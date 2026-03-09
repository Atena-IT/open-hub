# Xet Backend Examples

This folder contains Python scripts demonstrating the capabilities of the Xet Hub Backend.
To run these examples, you must have the local Docker Compose stack running.

## Basic API Roundtrip
`roundtrip.py`: Registers a test user, creates a repository, uploads a small JSON file, and downloads it back, checking the SHA-256 hash.

## Content-Defined Chunking (CDC) Demonstrations
These scripts demonstrate Xet's block-level deduplication. By enabling `HF_HUB_ENABLE_HF_TRANSFER=1`, the scripts use the `hf_xet` client to upload large files.
The second upload modifies the file and proves that the backend only transfers the delta.

1. **`cdc_csv_demo.py`**: Best example! Uploads a 250MB+ CSV file, appends 1 million rows, and re-uploads it. The second upload is nearly instantaneous as it skips transferring the base 250MB.
2. **`cdc_binary_demo.py`**: Generates a 20MB random binary blob, uploads it, appends 100 bytes, and re-uploads it. Shows how binary appends are deduplicated.
3. **`cdc_parquet_demo.py`**: Generates a Parquet dataset and rewrites it. (Note: Parquet often shifts bits unpredictably when rewritten by PyArrow/Pandas, which can foil CDC depending on the chunking boundaries).

### How to Run

Ensure the backend is running (`docker compose up -d`).

```bash
# Install dependencies
pip install huggingface_hub[hf_transfer] requests pandas numpy

# Run the basic roundtrip
python roundtrip.py

# Run the CDC demonstration
python cdc_csv_demo.py
```
