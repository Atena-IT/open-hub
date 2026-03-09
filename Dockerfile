# ── Chef stage ───────────────────────────────────────────────────────────────
FROM rust:1-bookworm AS chef
RUN apt-get update && apt-get install -y pkg-config libssl-dev && rm -rf /var/lib/apt/lists/*
RUN cargo install cargo-chef --locked
WORKDIR /build

# ── Planner stage ────────────────────────────────────────────────────────────
FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

# ── Builder stage ────────────────────────────────────────────────────────────
FROM chef AS builder
COPY --from=planner /build/recipe.json recipe.json
# Build dependencies - this layer is cached as long as dependencies don't change
RUN cargo chef cook --release --recipe-path recipe.json

# Build application
COPY . .
RUN cargo build --release -p server

# ── Runtime stage ─────────────────────────────────────────────────────────────
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y ca-certificates curl && rm -rf /var/lib/apt/lists/*

COPY --from=builder /build/target/release/xet-server /usr/local/bin/xet-server
COPY templates/ /app/templates/
COPY static/ /app/static/

WORKDIR /app
EXPOSE 3000

ENV TEMPLATE_DIR=/app/templates

CMD ["xet-server"]
