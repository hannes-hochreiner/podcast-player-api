FROM rust:slim AS builder
RUN apt update && apt install librust-openssl-dev -y
RUN mkdir -p /opt/rss-json-service
COPY src /opt/rss-json-service/src
COPY Cargo.* /opt/rss-json-service/
RUN cd /opt/rss-json-service && cargo build --release

FROM debian:stable-slim AS rss-json-service
MAINTAINER Hannes Hochreiner <hannes@hochreiner.net>
COPY --from=builder /opt/rss-json-service/target/release/rss-json-service /opt/rss-json-service
CMD ["/opt/rss-json-service"]

FROM debian:stable-slim AS updater
MAINTAINER Hannes Hochreiner <hannes@hochreiner.net>
COPY --from=builder /opt/rss-json-service/target/release/updater /opt/updater
CMD ["/opt/updater"]
