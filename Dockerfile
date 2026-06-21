# stage 1: frontend
FROM oven/bun:1 AS frontend-builder
WORKDIR /app
COPY frontend/package.json frontend/bun.lock ./
RUN bun install --frozen-lockfile
COPY frontend/ ./
RUN bun run build

# stage 2: api
FROM rust:1.95 AS api-builder
WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY crates ./crates

RUN cargo build --release

# stage 3: runtime
FROM debian:trixie-slim
RUN apt-get update && apt-get install -y ca-certificates openssh-client && rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY --from=api-builder /app/target/release/dashboard_api ./
COPY --from=frontend-builder /app/dist ./dist

EXPOSE 8080
CMD ["./dashboard_api"]
