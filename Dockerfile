# FROM lukemathwalker/cargo-chef:latest-rust-1.59.0-alpine AS chef
# WORKDIR /app/

# FROM chef AS planner
# COPY Cargo.toml Cargo.lock ./
# COPY src/ ./src/
# RUN cargo chef prepare --recipe-path recipe.json

# FROM chef AS builder
# COPY --from=planner /app/recipe.json ./recipe.json
# RUN apk update && apk add --no-cache musl-dev protoc
# # RUN cargo chef cook --release --recipe-path recipe.json
# RUN cargo chef cook --no-default-features --recipe-path recipe.json
# COPY src/ ./src/
# # RUN cargo build --release --no-default-features
# RUN cargo build --no-default-features

FROM rust:1.59-alpine as builder
WORKDIR /app/
RUN apk update && apk add --no-cache musl-dev protoc
COPY Cargo.toml Cargo.lock ./
COPY src/ ./src/
RUN cargo build --no-default-features

FROM alpine:latest as runtime
COPY --from=builder /app/target/debug/smolbotbot /usr/local/bin/sbb
# COPY --from=builder /app/target/release/smolbotbot /usr/local/bin/sbb
WORKDIR /sbb/
COPY docker_runtime/ ./
RUN chmod 0555 *.sh
ARG USER_ID=12000 GROUP_ID=12000
RUN addgroup -S -g "$GROUP_ID" sbb && adduser -SDH -u "$USER_ID" -g sbb sbb
RUN mkdir -p /var/lib/smolbotbot/bootstrap \
    && mkdir -p /var/lib/smolbotbot/images \
    && chown -R sbb:sbb /var/lib/smolbotbot/bootstrap \
    && chown -R sbb:sbb /var/lib/smolbotbot/images
VOLUME /var/lib/smolbotbot/bootstrap /var/lib/smolbotbot/images
USER sbb
ENTRYPOINT ["/bin/sh"]
CMD ["timeline.sh"]
