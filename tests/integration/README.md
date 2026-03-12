# Xet Backend Integration Tests & Examples

This folder contains Python scripts demonstrating the capabilities of the Xet Hub Backend and acting as integration tests against the `huggingface_hub` Python library.

The long-running `huggingface_hub` compatibility rollout is tracked in:
- `docs/hf_hub_testing_roadmap.md` — the human-readable source of truth for batch scope and status
- `resources/hf_hub_compat/checklist.json` — the machine-readable tracker used to keep batch state aligned with implementation work

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

Use [`uv`](https://github.com/astral-sh/uv) for all Python integration coverage.

### Script-based smoke tests

The original smoke tests remain self-contained Python scripts using PEP 723 inline metadata.

```bash
HF_ENDPOINT=http://localhost:8080 uv run tests/integration/roundtrip.py
HF_ENDPOINT=http://localhost:8080 uv run tests/integration/test_downloads.py
HF_ENDPOINT=http://localhost:8080 uv run tests/integration/integration_create_commit.py
```

### Pytest-based `hf_hub` compatibility slice

Batch 1 of the upstream-derived compatibility suite lives under `tests/integration/hf_hub/`.
These tests reuse the same bootstrap flow as the smoke scripts but run under pytest for better isolation and selective execution.
Future batches should continue the additive naming pattern `test_<feature>_batch<N>.py` and must update the roadmap/checklist in the same PR as any new compatibility work.

```bash
HF_ENDPOINT=http://localhost:8080 uv run --with pytest --with huggingface_hub --with requests pytest tests/integration/hf_hub/test_hf_api_batch1.py -q
HF_ENDPOINT=http://localhost:8080 uv run --with pytest --with huggingface_hub --with requests pytest tests/integration/hf_hub/test_snapshot_download_batch1.py -q
HF_ENDPOINT=http://localhost:8080 uv run --with pytest --with huggingface_hub --with requests pytest tests/integration/hf_hub/test_file_download_batch1.py -q
HF_ENDPOINT=http://localhost:8080 uv run --with pytest --with huggingface_hub --with requests pytest tests/integration/hf_hub -q
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

