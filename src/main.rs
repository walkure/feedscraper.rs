use atom_syndication::{Feed, FixedDateTime};
use url::{Url, ParseError};
use clap::Parser;
use std::path::{Path,PathBuf};

#[derive(Debug, Parser)]
struct Args {
    // The base path of output files
    #[arg(short='b', long="basePath", default_value = "./")]
    base_path: String,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {

    let args = Args::parse();
    let base_path = Path::new(&args.base_path);

    let feed = Scraper{
        target: "https://www.spf.org/iina/articles/",
        title_selector: scraper::Selector::parse("h3.title > a").unwrap(),
        author_selector: scraper::Selector::parse("div.author > a").unwrap(),
        category_selector: Some(scraper::Selector::parse("div.category > a").unwrap()),
        date_selector: scraper::Selector::parse("div.date").unwrap(),
        date_format: "%Y/%m/%d",
        feed_title_selector: scraper::Selector::parse("div.logo > a").unwrap(),
        column_selector: scraper::Selector::parse("section.sec-articles div.article-list > div.row > div.col").unwrap(),
    }.scrape()?;

    let path = base_path.join("./iina.atom");
    write_file(path, &feed.to_string())?;

    let feed = Scraper{
        target: "https://www.spf.org/jpus-insights/spf-america-monitor/",
        title_selector: scraper::Selector::parse("a[class='card-news-featured js-card-news-featured']").unwrap(),
        author_selector: scraper::Selector::parse("p.author").unwrap(),
        category_selector: None,
        date_selector: scraper::Selector::parse("p.date").unwrap(),
        date_format: "%Y.%m.%d",
        feed_title_selector: scraper::Selector::parse("title").unwrap(),
        column_selector: scraper::Selector::parse("div[id^='extSeries_'].extSeries").unwrap(),
    }.scrape()?;

    let path = base_path.join("./jpus.atom");
    write_file(path, &feed.to_string())?;

    let feed = Scraper{
        target: "https://www.nri.com/jp/knowledge/blog/lst?page=1&pageSize=30",
        title_selector: scraper::Selector::parse("p._title > a").unwrap(),
        author_selector: scraper::Selector::parse("p.author").unwrap(),
        category_selector: None,
        date_selector: scraper::Selector::parse("div._day > p").unwrap(),
        date_format: "%Y/%m/%d",
        feed_title_selector: scraper::Selector::parse("title").unwrap(),
        column_selector: scraper::Selector::parse("div.l-news > ul > li").unwrap(),
    }.scrape()?;

    let path = base_path.join("./nri.atom");
    write_file(path, &feed.to_string())?;

    Ok(())
}

fn write_file(path: PathBuf, content: &str) -> Result<(), Box<dyn std::error::Error>> {
    use std::fs::File;
    use std::io::Write;

    println!("write file: {}", path.display());
    let mut file = File::create(path)?;
    write!(file, "{}", content)?;
    file.flush()?;

    Ok(())
}

fn remove_whitespace(s: String) -> String {
    s.chars().filter(|c| !c.is_whitespace()).collect()
}

pub fn _rel2abs(base_url: &str, input: &str) -> Result<String, ParseError> {

    if input.starts_with("http://") || input.starts_with("https://") {
        return Ok(input.to_string());
    }

    let base = Url::parse(base_url)?;
    let result = base.join(&input)?.to_string();
    
    Ok(result)
}

struct Scraper<'a>{
    target: &'a str,
    title_selector: scraper::Selector,
    author_selector: scraper::Selector,
    category_selector: Option<scraper::Selector>,
    date_selector: scraper::Selector,
    date_format: &'a str,
    feed_title_selector: scraper::Selector,
    column_selector: scraper::Selector,
}

impl Scraper<'_> {
    fn scrape(&self) ->  Result<Feed, Box<dyn std::error::Error>>{

    let body = reqwest::blocking::get(self.target)?.text()?;
    let document = scraper::Html::parse_document(&body);

    let feed_title_element = document.select(&self.feed_title_selector).next().ok_or("feed title tag not found")?;

    // generate Atom feed
    let mut feed = Feed::default();
    println!("URL:{}\nTITLE:{}",self.target.to_string(), feed_title_element.inner_html());
    feed.set_title(feed_title_element.inner_html());
    feed.set_links(vec![atom_syndication::Link {
        href: self.target.to_string(),
        mime_type: None,
        hreflang: None,
        title: None,
        length: None,
        rel: "alternate".to_string(),
    }]);

    let jp = chrono::offset::FixedOffset::east_opt(9 * 3600).unwrap();

    let fixed_now = FixedDateTime::from_naive_utc_and_offset(chrono::Utc::now().naive_utc(), jp);   
    feed.set_updated(fixed_now);

    let column_element = document.select(&self.column_selector);

    // walk elements 
    for element in column_element{

        let mut entry = atom_syndication::Entry::default();

        if let Some(author_element) = element.select(&self.author_selector).next() {
            entry.set_authors(vec![atom_syndication::Person {
                name: remove_whitespace(author_element.inner_html()),
                email: None,
                uri: None,
            }]);
        }

        let date_element = element.select(&self.date_selector).next().ok_or("date tag not found")?;

        // parse date string to chrono::NaiveDateTime and convert to chrono::DateTime<chrono::offset::FixedOffset>
        // and_hms_opt(0,0,0) and east_opt(9*3600) MUST success.
        let updated = chrono::NaiveDate::parse_from_str(&remove_whitespace(date_element.inner_html())
        , self.date_format)?.and_hms_opt(0,0,0).unwrap();

        entry.set_updated(FixedDateTime::from_naive_utc_and_offset(updated, jp));

        let title_element = element.select(&self.title_selector).next().ok_or("title tag not found")?;
        let title = title_element.attr("title").map_or_else(
            || title_element.inner_html(),
            |s| s.to_string()
        ).replace("<br>", "\n");

        if let Some(cs) = &self.category_selector {
            let category_element = element.select(cs).next().ok_or("category tag not found")?;

            entry.set_categories(vec![atom_syndication::Category {
                term: remove_whitespace(category_element.inner_html()),
                scheme: None,
                label: None,
            }]);
            entry.set_title(format!("{}: {}", remove_whitespace(category_element.inner_html()), title));
        }else{
            entry.set_title(title);
        }


        let href  = title_element.attr("href").ok_or("href attr not found")?;
        let abs_href = _rel2abs(self.target, href)?;

        entry.set_id(href);
        entry.set_links(vec![atom_syndication::Link {
            href: abs_href,
            mime_type: None,
            hreflang: None,
            title: None,
            length: None,
            rel: "alternate".to_string(),
        }]);

        feed.entries.push(entry);

    }

    Ok(feed)
    }
}

