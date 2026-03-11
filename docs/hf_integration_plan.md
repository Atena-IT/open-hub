# HF Hub Integration Plan

## Overview
This document outlines the plan for building integration tests against the higher-level Python APIs provided by the `huggingface_hub` library. By testing these high-level APIs, we ensure that our custom Xet Storage Backend correctly implements the necessary Hub API endpoints to support realistic user workflows.

## Target APIs & Integration Scripts

### 1. `upload_folder`
- **Description**: Recursively uploads an entire local folder to a remote repository on the Hub.
- **Integration Script**:
  1. Create a new test repository using `create_repo()`.
  2. Generate a local directory containing dummy model files (e.g., `config.json`, `pytorch_model.bin`, `tokenizer.json`).
  3. Call `upload_folder()` with the local directory path.
  4. Use the CAS backend to verify that all chunks/xorbs were correctly uploaded and the commit successfully registered all files.

### 2. `create_commit`
- **Description**: Allows batching multiple file additions, modifications, and deletions into a single atomic commit.
- **Integration Script**:
  1. Instantiate an `HfApi` client and create a test repository.
  2. Use `create_commit()` providing a list of operations (`CommitOperationAdd`, `CommitOperationDelete`).
  3. Issue an API request to verify the exact commit history, ensuring the backend accurately resolved the NDJSON commit format and correctly represented the new repository state.

### 3. `snapshot_download`
- **Description**: Downloads an entire repository (or specific filtered files) at a given revision, storing them in the local cache.
- **Integration Script**:
  1. Populate a test repository with multiple files.
  2. Call `snapshot_download()` pointing to an empty local cache directory.
  3. Validate that the downloaded files match the source exactly (e.g., via SHA-256 hashes) and that the backend properly served the `resolve_file` proxy content with correct ETags and headers.

### 4. `ModelCard` and `DatasetCard` APIs
- **Description**: High-level object-oriented APIs for creating, parsing, and updating `README.md` files (and their YAML frontmatter) for models and datasets.
- **Integration Script**:
  1. Instantiate a `ModelCard` (or `DatasetCard`) with mock tags, metadata, and evaluation results.
  2. Call the `.push_to_hub()` method to commit the card.
  3. Fetch the `README.md` file back using `hf_hub_download()`.
  4. Parse the file locally to verify the YAML frontmatter and markdown body were successfully persisted by the backend without corruption.

### 5. `HfFileSystem`
- **Description**: A convenient, fsspec-compatible file system interface to interact with the Hub.
- **Integration Script**:
  1. Instantiate `HfFileSystem`.
  2. Use `fs.write_text()` to create a few files in a test repository.
  3. Use `fs.glob()` to verify the directory structure and `fs.open()` to stream the file contents directly, ensuring the backend supports these standard filesystem-like operations.
