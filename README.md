# Xet Storage Backend

A private Content-Addressable Storage (CAS) backend in Rust implementing the open [Xet protocol](https://huggingface.co/docs/xet/index), compatible with the `xet-core` / `hf_xet` client.

## Architecture

```
crates/
  common/        # Config, error types, hash utilities (hash_to_api_string / api_string_to_hash)
  db-layer/      # sqlx PostgreSQL: chunks, xorbs, file_mappings tables
  s3-storage/    # aws-sdk-s3: put_object, presign_get (MinIO / S3 compatible)
  shard-parser/  # Full MDB shard binary format parser + writer
  cas-server/    # axum HTTP server with all 7 CAS protocol endpoints
```

## Quick Start (local)

```bash
cp .env.example .env
docker-compose up --build
```

Services:
- **cas-server**: http://localhost:3000
- **MinIO console**: http://localhost:9001 (minioadmin / minioadmin)
- **PostgreSQL**: localhost:5432 (xet / xet / xetdb)

## API Endpoints

| Method | Path | Description |
|--------|------|-------------|
| GET | `/health` | Health check |
| GET | `/api/{type}s/{id}/xet-read-token/{rev}` | Issue read JWT |
| GET | `/api/{type}s/{id}/xet-write-token/{rev}` | Issue write JWT |
| POST | `/v1/xorbs/default/{hash}` | Upload a Xorb |
| POST | `/v1/shards` | Upload a Shard (registers files) |
| GET | `/v1/chunks/default-merkledb/{hash}` | Global dedup query |
| GET | `/v1/reconstructions/{file_id}` | Get file reconstruction |

## Connecting xet-core

Point the client to this server by obtaining a token from the auth endpoint:

```
GET http://localhost:3000/api/models/my-org/my-model/xet-read-token/main
```

Response:
```json
{
  "accessToken": "<jwt>",
  "exp": 1848535668,
  "casUrl": "http://localhost:3000"
}
```

## Development

```bash
# Build all crates
cargo build --workspace

# Run tests
cargo test --workspace

# Run server directly (with .env)
cargo run -p cas-server
```

## Key Protocol Details

- **Hash encoding**: 32-byte hashes in API paths use little-endian u64 reversal per 8-byte group — see `common::hash_to_api_string`.
- **Chunking**: Gearhash CDC, target 64 KiB, min 8 KiB, max 128 KiB.
- **Hashing**: Blake3 keyed hash (DATA_KEY for chunks, INTERNAL_NODE_KEY for Xorb MerkleTree, VERIFICATION_KEY for terms).
- **Shard upload order**: All Xorbs MUST be uploaded before the Shard that references them.
- **Download**: Clients download Xorb data directly from S3 via pre-signed URLs — the server is not in the download hot path.
