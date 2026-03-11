# Xet Backend Integration Tests & Examples

This folder contains Python scripts demonstrating the capabilities of the Xet Hub Backend and acting as integration tests against the `huggingface_hub` Python library.

## Setup

To run these integration tests, you must have the local Docker Compose stack running, which spins up the Xet Hub server, PostgreSQL, and MinIO.

```bash
# 1. Ensure you are in the repository root
cd ..

# 2. Start the Docker Compose stack in the background
docker compose up -d --build --wait

# The Hub is now running at http://localhost:8080
```

## Running the Tests

All tests are written as **self-contained Python scripts** using PEP 723 inline metadata. This means you do NOT need to manually manage virtual environments or pip installs. We highly recommend using [`uv`](https://github.com/astral-sh/uv) to run them.

```bash
# Run a specific integration test using uv (it will auto-install dependencies)
uv run integration/cdc_csv_demo.py

# Or run the basic roundtrip
uv run integration/roundtrip.py
```

## Test Categories

### Basic Hub API
- `roundtrip.py`: Registers a test user, creates a repository, uploads a small JSON file, and downloads it back.
- `integration_create_commit.py`: Tests batched, atomic commit operations.
- `integration_upload_folder.py`: Tests recursive folder uploads.
- `integration_snapshot_download.py`: Tests downloading an entire repository to the local cache.
- `integration_model_card.py`: Tests the `ModelCard` abstraction and YAML frontmatter writing.
- `integration_hffilesystem.py`: Tests the `HfFileSystem` (fsspec) interface.

### Content-Defined Chunking (CDC) Demonstrations
These scripts demonstrate Xet's block-level deduplication using the `hf_xet` client:
1. **`cdc_csv_demo.py`**: Best example! Uploads a 250MB+ CSV file, appends 1M rows, and re-uploads it instantly.
2. **`cdc_binary_demo.py`**: Shows how binary appends are deduplicated.
3. **`cdc_parquet_demo.py`**: Generates a Parquet dataset and rewrites it to test CDC against compression shifts.

