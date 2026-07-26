# stage 1: ui
FROM rust:1.95 AS ui-builder
WORKDIR /app

COPY scripts/dependencies.sh /usr/local/bin/dependencies.sh

ARG TARGETARCH
ARG TRUNK_VERSION=0.21.14

RUN rustup target add wasm32-unknown-unknown
RUN chmod +x /usr/local/bin/dependencies.sh
RUN /usr/local/bin/dependencies.sh "$TRUNK_VERSION" "$TARGETARCH"

COPY Cargo.toml Cargo.lock ./
COPY crates ./crates

WORKDIR /app/crates/dashboard_ui
RUN trunk build --release

# stage 2: api
FROM rust:1.95 AS api-builder
WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY crates ./crates

RUN cargo build -p dashboard_api --release

# stage 3: runtime
FROM debian:trixie-slim
RUN apt-get update && apt-get install -y ca-certificates openssh-client && rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY --from=api-builder /app/target/release/dashboard_api ./
COPY --from=ui-builder /app/crates/dashboard_ui/dist ./dist

EXPOSE 8080
CMD ["./dashboard_api"]
