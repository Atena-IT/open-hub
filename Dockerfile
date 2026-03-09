# ── Build stage ──────────────────────────────────────────────────────────────
FROM rust:1-bookworm AS builder

RUN apt-get update && apt-get install -y pkg-config libssl-dev && rm -rf /var/lib/apt/lists/*

WORKDIR /build

# Cache dependencies
COPY Cargo.toml ./
COPY crates/common/Cargo.toml      crates/common/Cargo.toml
COPY crates/db-layer/Cargo.toml    crates/db-layer/Cargo.toml
COPY crates/s3-storage/Cargo.toml  crates/s3-storage/Cargo.toml
COPY crates/shard-parser/Cargo.toml crates/shard-parser/Cargo.toml
COPY crates/cas-server/Cargo.toml  crates/cas-server/Cargo.toml
COPY crates/hub-api/Cargo.toml     crates/hub-api/Cargo.toml
COPY crates/web-ui/Cargo.toml      crates/web-ui/Cargo.toml
COPY crates/server/Cargo.toml      crates/server/Cargo.toml

# Create stub lib.rs / main.rs files so cargo can fetch deps
RUN for d in common db-layer s3-storage shard-parser hub-api web-ui; do \
      mkdir -p crates/$d/src && echo "// stub" > crates/$d/src/lib.rs; \
    done && \
    mkdir -p crates/cas-server/src && \
    echo "// stub" > crates/cas-server/src/lib.rs && \
    echo "fn main(){}" > crates/cas-server/src/main.rs && \
    mkdir -p crates/server/src && \
    echo "fn main(){}" > crates/server/src/main.rs

RUN cargo fetch

# Copy actual source and build
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
