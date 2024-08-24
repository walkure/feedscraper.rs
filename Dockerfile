FROM rust:1.80.0-alpine3.19 as builder

WORKDIR /build

RUN apk update && apk add --no-cache musl-dev libressl-dev pkgconfig ca-certificates && update-ca-certificates

COPY ./Cargo.lock ./Cargo.lock
COPY ./Cargo.toml ./Cargo.toml
#RUN mkdir src
COPY ./src ./src

RUN cargo build --release --target=x86_64-unknown-linux-musl

FROM scratch
COPY --from=builder /build/target/x86_64-unknown-linux-musl/release/feedscraper .
# libressl ignores envrion SSL_CERT_FILE and set to /etc/ssl/cert.pem
COPY --from=builder /etc/ssl/certs/ca-certificates.crt /etc/ssl/cert.pem

ENTRYPOINT ["/feedscraper"]
