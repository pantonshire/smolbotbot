FROM rust:1.54-alpine as build
WORKDIR /app/
COPY Cargo.toml Cargo.lock ./
COPY src/ ./src/
RUN apk update \
    && apk add --no-cache musl-dev protoc
RUN cargo build --release --no-default-features

FROM alpine:latest as runtime
COPY --from=build /app/target/release/smolbotbot /usr/local/bin/sbb
WORKDIR /sbb/
COPY docker_runtime/ ./
RUN chmod 0555 *.sh
ENV USER_ID=12000 GROUP_ID=12000
RUN addgroup -S -g "$GROUP_ID" sbb \
    && adduser -SDH -u "$USER_ID" -g sbb sbb
RUN mkdir -p /var/lib/smolbotbot/bootstrap \
    && mkdir -p /var/lib/smolbotbot/images \
    && chown -R sbb:sbb /var/lib/smolbotbot/bootstrap \
    && chown -R sbb:sbb /var/lib/smolbotbot/images
VOLUME /var/lib/smolbotbot/bootstrap /var/lib/smolbotbot/images
USER sbb
ENTRYPOINT ["/bin/sh"]
CMD ["timeline.sh"]
