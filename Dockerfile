FROM rust:1.67 AS builder
WORKDIR /usr/src/$APP
COPY . .
RUN cargo install --path .

FROM debian:stable-slim
RUN apt-get update && DEBIAN_FRONTEND=noninteractive apt-get install -qq ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=builder /usr/local/cargo/bin/mastodon-wsj-bot /usr/local/bin/mastodon-wsj-bot
CMD ["mastodon-wsj-bot"]





