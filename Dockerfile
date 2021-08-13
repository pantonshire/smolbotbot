FROM rust:1.54-alpine as build
WORKDIR /app/
COPY Cargo.toml Cargo.lock ./
COPY src/ ./src/
RUN apk update
RUN apk add --no-cache musl-dev protoc
RUN cargo build --release --no-default-features

FROM alpine:latest as runtime
COPY --from=build /app/target/release/smolbotbot /usr/local/bin/sbb
WORKDIR /sbb/
RUN mkdir -p /var/lib/smolbotbot/images
COPY docker/runtime/entry.sh ./

# Temporary
COPY tweet_ids ./

ENTRYPOINT ["/bin/sh", "entry.sh"]
