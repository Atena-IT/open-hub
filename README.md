# Xet Storage Backend (Hub Alternative)

A private Content-Addressable Storage (CAS) backend and Hugging Face Hub alternative written in Rust. It implements the open [Xet protocol](https://huggingface.co/docs/xet/index) for chunk-level deduplicated storage, complete with a Hub API compatible with the `huggingface_hub` Python library, an LFS server, and a Web UI.

## Features

- **Xet CAS Server**: Fully implements the 7 CAS protocol endpoints (upload, download, shard parsing, chunk deduplication).
- **Hugging Face Hub API Compatibility**: Natively supports `huggingface_hub` and `hf_xet` Python clients (`api.create_repo`, `api.upload_file`, `api.hf_hub_download`, etc.).
- **Smart Deduplication**: Utilizes Content-Defined Chunking (CDC) via Gearhash and Blake3 to identify duplicate blocks, drastically accelerating re-uploads of datasets (like Parquet/CSV) and models.
- **Git LFS Support**: Fully functional `/info/lfs` API with batch, upload, download, and verify endpoints, acting as a standard Git LFS server.
- **S3 & PostgreSQL**: Uses MinIO/S3 for highly scalable blob storage (with pre-signed URLs for direct downloads) and PostgreSQL for fast metadata and chunk mappings.
- **Web UI**: Built-in server-rendered (Tera) UI to browse users, repositories, files, and manage settings.

## Architecture

The project is structured as a Rust workspace with the following monorepo layout:

```text
crates/
  common/        # Shared config, error types, hash utilities
  db-layer/      # sqlx PostgreSQL queries and migrations (CAS + Hub tables)
  s3-storage/    # aws-sdk-s3 with pre-signed GET/PUT support
  shard-parser/  # MDB shard binary format parser + writer for Xet
  cas-server/    # Xet CAS API endpoints (/v1/xorbs, /v1/shards, /v1/reconstructions)
  hub-api/       # HF-compatible Hub API (/api/whoami, NDJSON commits, LFS, Auth)
  web-ui/        # Tera templates and server-rendered frontend (/-/ system routes)
  server/        # Unified binary that mounts all routers together

experiments/     # Test scripts for verifying CDC deduplication, LFS, and Hub API
```

## Quick Start (Docker Compose)

The easiest way to spin up the entire stack locally (Postgres, MinIO S3, minio-init, and the unified `xet-server`) is using Docker Compose.

```bash
# 1. Prepare environment variables
cp deployment/.env.example .env

# 2. Start the services
docker compose up -d --build
```

### Services Started:
- **Xet Server (Hub & CAS)**: http://localhost:8080
- **MinIO Console**: http://localhost:9001 (minioadmin / minioadmin)
- **PostgreSQL**: localhost:5432 (xet / xet / xetdb)

*Note: The server handles DB migrations and S3 bucket initialization automatically on startup.*

## Using with Python (`huggingface_hub`)

You can point the official Hugging Face Python client directly to your local instance.

```python
import os
from huggingface_hub import HfApi

# Tell the library to use your custom backend
os.environ["HF_ENDPOINT"] = "http://localhost:8080"
# Enable Xet transfer for lightning-fast deduplicated uploads
os.environ["HF_HUB_ENABLE_HF_TRANSFER"] = "1"

api = HfApi()

# Login (or use the web UI to register a token)
# api.token = "ox_your_token_here"

# Create a repository
api.create_repo(repo_id="testuser/my-model", repo_type="model")

# Upload a file (Will utilize LFS or Xet CAS for large files)
api.upload_file(
    path_or_fileobj="local_model.bin",
    path_in_repo="model.bin",
    repo_id="testuser/my-model"
)
```

Check the `experiments/` folder for full Python scripts that demonstrate CDC (Content-Defined Chunking) deduplication with large CSV and Parquet datasets.

## Manual Local Development

If you wish to run the server locally outside of Docker (e.g., for `cargo check` or `cargo run`):

```bash
# 1. Spin up the datastores
docker compose up -d postgres minio minio-init

# 2. Build the workspace
cargo build --workspace

# 3. Run the unified server
cargo run -p server
```

Run integration and unit tests:
```bash
cargo test --workspace
# Run full E2E tests including DB operations (ensure DATABASE_URL/S3_ENDPOINT are set)
cargo test --workspace -- --include-ignored
```

## Protocol Details
- **Upload Order**: Xorbs (data blobs) are pushed to S3 via pre-signed URLs before the Shard (chunk metadata) that references them.
- **Deduplication**: Driven by `/v1/chunks`. The client calculates chunk hashes locally and skips uploading chunks the server already has.
- **Download**: Clients retrieve reconstruction maps (Xorb byte ranges) from the CAS server, then download binary data directly from S3 via pre-signed GET URLs, keeping the Rust server out of the hot data path.

## Roadmap & Feature Support
We aim to provide drop-in support for the core lifecycle operations of Models and Datasets. For a detailed breakdown of which `huggingface_hub` features are supported natively (✅), partially supported (⚠️), or out of scope (❌), please see the **[Feature Support Matrix](docs/feature_support_matrix.md)**.
