FROM rust:1.54-alpine as planner
WORKDIR /app/
RUN apk update && apk add --no-cache musl-dev
RUN cargo install cargo-chef && rm -rf /usr/local/cargo/registry/
COPY Cargo.toml Cargo.lock ./
COPY src/ ./src/
RUN cargo chef prepare --recipe-path recipe.json

FROM rust:1.54-alpine as cacher
WORKDIR /app/
RUN apk update && apk add --no-cache musl-dev protoc
RUN cargo install cargo-chef && rm -rf /usr/local/cargo/registry/
COPY --from=planner /app/recipe.json ./recipe.json
RUN cargo chef cook --release --no-default-features --recipe-path recipe.json

FROM rust:1.54-alpine as builder
WORKDIR /app/
COPY Cargo.toml Cargo.lock ./
COPY src/ ./src/
COPY --from=cacher /app/target target
COPY --from=cacher /usr/local/cargo /usr/local/cargo
RUN cargo build --release --no-default-features

FROM alpine:latest as runtime
COPY --from=builder /app/target/release/smolbotbot /usr/local/bin/sbb
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
