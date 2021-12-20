FROM fedora:34 AS builder
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
RUN dnf install gcc openssl-devel -y
RUN mkdir -p /opt/rss-json-service
COPY src /opt/rss-json-service/src
COPY Cargo.* /opt/rss-json-service/
RUN source $HOME/.cargo/env && cd /opt/rss-json-service && cargo build --release

FROM fedora:34 AS rss-json-service
MAINTAINER Hannes Hochreiner <hannes@hochreiner.net>
COPY --from=builder /opt/rss-json-service/target/release/rss-json-service /opt/rss-json-service
CMD ["/opt/rss-json-service"]

FROM fedora:34 AS updater
MAINTAINER Hannes Hochreiner <hannes@hochreiner.net>
COPY --from=builder /opt/rss-json-service/target/release/updater /opt/updater
CMD ["/opt/updater"]
