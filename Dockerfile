# Stage 1: Setup build environment with cargo-chef
FROM --platform=$BUILDPLATFORM lukemathwalker/cargo-chef:latest-rust-1.95-slim-trixie AS chef
WORKDIR /app

# Stage 2: Prepare recipe.json from Cargo files
FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

# Stage 3: Build dependencies (with caching)
FROM chef AS builder
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --recipe-path recipe.json --release

# Stage 4: Build the application
COPY . .
ARG APP_VERSION=0.0.0
RUN APP_VERSION=${APP_VERSION} cargo build --release

# Stage 5: Install speedtest CLI and runtime image
FROM debian:trixie-slim

RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates curl gnupg2 && \
    curl -s https://packagecloud.io/install/repositories/ookla/speedtest-cli/script.deb.sh | bash && \
    apt-get install --no-install-recommends -y speedtest && \
    rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/speedtest-exporter /usr/local/bin/

EXPOSE 9516
ENTRYPOINT ["speedtest-exporter"]
