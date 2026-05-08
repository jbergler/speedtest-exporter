FROM rust:1.95-alpine3.22 AS builder
WORKDIR /app

ARG TARGETARCH
ARG APP_VERSION=0.0.0

RUN apk add --no-cache build-base musl-dev

RUN set -eux; \
    case "$TARGETARCH" in \
        amd64) echo "x86_64-unknown-linux-musl" > /tmp/rust-target ;; \
        arm64) echo "aarch64-unknown-linux-musl" > /tmp/rust-target ;; \
        *) echo "Unsupported TARGETARCH: $TARGETARCH" >&2; exit 1 ;; \
    esac; \
    rustup target add "$(cat /tmp/rust-target)"

COPY . .

RUN set -eux; \
    RUST_TARGET="$(cat /tmp/rust-target)"; \
    APP_VERSION="$APP_VERSION" cargo build --release --target "$RUST_TARGET"; \
    strip "target/$RUST_TARGET/release/speedtest-exporter"

FROM alpine:3.22

ARG TARGETARCH
ARG OOKLA_VERSION=1.2.0
ARG OOKLA_SHA256_AMD64=5690596c54ff9bed63fa3732f818a05dbc2db19ad36ed68f21ca5f64d5cfeeb7
ARG OOKLA_SHA256_ARM64=3953d231da3783e2bf8904b6dd72767c5c6e533e163d3742fd0437affa431bd3

RUN set -eux; \
    apk add --no-cache ca-certificates; \
    apk add --no-cache --virtual .fetch-deps curl tar; \
    case "$TARGETARCH" in \
        amd64) OOKLA_ARCH="x86_64"; OOKLA_SHA256="$OOKLA_SHA256_AMD64" ;; \
        arm64) OOKLA_ARCH="aarch64"; OOKLA_SHA256="$OOKLA_SHA256_ARM64" ;; \
        *) echo "Unsupported TARGETARCH: $TARGETARCH" >&2; exit 1 ;; \
    esac; \
    curl -fsSL "https://install.speedtest.net/app/cli/ookla-speedtest-${OOKLA_VERSION}-linux-${OOKLA_ARCH}.tgz" -o /tmp/ookla.tgz; \
    echo "$OOKLA_SHA256  /tmp/ookla.tgz" | sha256sum -c -; \
    tar -xzf /tmp/ookla.tgz -C /usr/local/bin speedtest; \
    chmod +x /usr/local/bin/speedtest; \
    rm -f /tmp/ookla.tgz; \
    apk del .fetch-deps

COPY --from=builder /app/target/*/release/speedtest-exporter /usr/local/bin/speedtest-exporter

EXPOSE 9516
ENTRYPOINT ["speedtest-exporter"]
