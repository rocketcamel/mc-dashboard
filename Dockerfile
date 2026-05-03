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
COPY api/Cargo.toml api/Cargo.lock ./
COPY api/templates ./templates
# dummy src to cache dependencies
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release
# Now copy real source and rebuild
RUN rm -rf src
COPY api/src ./src
RUN touch src/main.rs && cargo build --release

# stage 3: runtime
FROM debian:trixie-slim
RUN apt-get update && apt-get install -y ca-certificates openssh-client && rm -rf /var/lib/apt/lists/*
WORKDIR /app
COPY --from=api-builder /app/target/release/api ./
COPY --from=frontend-builder /app/dist ./dist
COPY api/templates ./templates
EXPOSE 8080
CMD ["./api"]
