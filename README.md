# feedscraper

WebサイトをクロールしてAtom feedを吐きます。

## build

``cargo build --release``

## exec

```:shell
docker run --rm -it -v "$(pwd)"/atom:/atom ghcr.io/walkure/feedscraper.rs:latest -b /atom
```

## targets

- [国際情報ネットワーク分析 IINA](https://www.spf.org/iina/articles/)
- [SPFアメリカ現状モニター](https://www.spf.org/jpus-insights/spf-america-monitor/)
- [NRIコラム](https://www.nri.com/jp/knowledge/blog)

## License

MIT

## Author

walkure <walkure at 3pf.jp>
