# Round 1 Module Map — `storage`

**OpenXet commit:** `a5b6dfcdf5e806a5a29cb893f0d79d733bea309b`
**Analyst:** DiTo97
**Date:** 2026-03-31
**Tracking issue:** [#43](https://github.com/Atena-IT/open-hub/issues/43)

---

## Scope

This document maps the OpenXet `storage` module. It covers only the upstream OpenXet source — no `xet-backend` references appear here. Round 2 will introduce the comparison.

## Source location

```
src/storage/
```

Key entry points:
- `src/storage/mod.rs` — module root: `pub use` re-exports; module-level `#![allow(dead_code)]`
- `src/storage/backend.rs` — `StorageBackend` trait definition, `StorageError`, `StorageResult`, `namespaces` constants
- `src/storage/config.rs` — `StorageConfig`, `StorageType` enum, and factory constructors
- `src/storage/local.rs` — `LocalStorage` implementation (filesystem-backed)
- `src/storage/s3.rs` — `S3Storage` implementation, `S3Config` (AWS S3 / MinIO / R2)

## Public surface

| Symbol | Kind | File | Brief description |
| --- | --- | --- | --- |
| `StorageBackend` | `trait` | `src/storage/backend.rs` | Async, pluggable storage abstraction with 10 methods covering get, put, delete, range, list, stream, and file I/O |
| `namespaces` | `mod` | `src/storage/backend.rs` | Six `&str` constants naming logical storage partitions |
| `namespaces::GIT_OBJECTS` | `const` | `src/storage/backend.rs` | `"git-objects"` — Git loose object store |
| `namespaces::GIT_PACKS` | `const` | `src/storage/backend.rs` | `"git-packs"` — Git pack files |
| `namespaces::CAS_CHUNKS` | `const` | `src/storage/backend.rs` | `"cas-chunks"` — per-chunk storage (deprecated; prefer `BLOCKS`) |
| `namespaces::BLOCKS` | `const` | `src/storage/backend.rs` | `"blocks"` — bundled ~64 MiB block objects; primary CAS storage unit |
| `namespaces::LFS_RAW` | `const` | `src/storage/backend.rs` | `"lfs-raw"` — raw LFS objects awaiting chunking |
| `namespaces::REPO_META` | `const` | `src/storage/backend.rs` | `"repo-meta"` — repository metadata (defined but unused in current codebase) |
| `StorageConfig` | `struct` | `src/storage/config.rs` | Storage configuration; wraps a `StorageType`; provides factory methods and `build`/`build_local` |
| `StorageType` | `enum` | `src/storage/config.rs` | `Local { path }` or `S3(S3Config)` |
| `LocalStorage` | `struct` | `src/storage/local.rs` | Filesystem backend: sharded two-level directory tree |
| `S3Config` | `struct` | `src/storage/s3.rs` | S3 connection parameters: bucket, region, optional endpoint, `force_path_style` |
| `S3Storage` | `struct` | `src/storage/s3.rs` | S3-compatible backend using `aws-sdk-s3` |

`StorageError` and `StorageResult` are **not** re-exported by `mod.rs`; they are internal to `backend.rs` and consumed only within the storage sub-modules.

## Internal structure

### Sub-modules

| Sub-module | Purpose |
| --- | --- |
| `backend` | Core trait (`StorageBackend`), error type (`StorageError`), result alias, and namespace constants |
| `config` | `StorageConfig` and `StorageType`; sync (`build_local`) and async (`build`) factory functions that produce a `Arc<dyn StorageBackend>` |
| `local` | Filesystem implementation of `StorageBackend` using tokio async I/O; sharded directory layout |
| `s3` | S3-compatible implementation of `StorageBackend` using `aws-sdk-s3`; supports AWS, MinIO, R2 |

### Key types

#### `StorageBackend`

The central abstraction. Declared `#[async_trait]` with `Send + Sync` bounds, making all implementors thread-safe and usable behind `Arc<dyn StorageBackend>`. The 10 methods are:

| Method | Signature | Notes |
| --- | --- | --- |
| `get` | `(namespace, key) -> StorageResult<Bytes>` | Full object fetch |
| `get_range` | `(namespace, key, Range<u64>) -> StorageResult<Bytes>` | Partial fetch; range is `[start, end)` |
| `put` | `(namespace, key, Bytes) -> StorageResult<()>` | Store object |
| `delete` | `(namespace, key) -> StorageResult<()>` | Remove object |
| `exists` | `(namespace, key) -> StorageResult<bool>` | Presence check |
| `list` | `(namespace, prefix?) -> StorageResult<Vec<String>>` | Enumerate keys, optionally filtered by prefix |
| `size` | `(namespace, key) -> StorageResult<u64>` | Metadata-only size query |
| `get_to_file` | `(namespace, key, &Path) -> StorageResult<()>` | Download object directly to a local file |
| `put_from_file` | `(namespace, key, &Path) -> StorageResult<()>` | Upload object from a local file |
| `get_stream` | `(namespace, key) -> StorageResult<Box<dyn AsyncRead + Unpin + Send>>` | Return a streaming reader |

#### `StorageError`

Private to `backend.rs`. Four variants:

| Variant | Payload | Trigger |
| --- | --- | --- |
| `NotFound(String)` | key description | Object absent in backend |
| `Io(std::io::Error)` | wrapped IO error | Filesystem errors other than NotFound |
| `InvalidRange(String)` | description | Range out of bounds or malformed |
| `Other(String)` | description | SDK or protocol errors (S3 path) |

`From<std::io::Error>` is implemented: `ErrorKind::NotFound` maps to `StorageError::NotFound`; all others map to `StorageError::Io`.

#### `StorageConfig` / `StorageType`

`StorageConfig` holds a single `storage_type: StorageType` field. Factory methods:

| Method | Backend produced |
| --- | --- |
| `local(path: PathBuf)` | `LocalStorage` at the given path |
| `s3(bucket, region)` | `S3Storage` for AWS S3 (no custom endpoint) |
| `minio(bucket, endpoint)` | `S3Storage` with custom endpoint and `force_path_style: true` |

`build_local() -> Option<Arc<dyn StorageBackend>>` is sync and returns `None` for `StorageType::S3` (S3 requires async initialisation). `build() -> Arc<dyn StorageBackend>` is async and supports all variants. Default is `Local { path: $TMPDIR/git-xet-storage }`.

#### `LocalStorage`

Single field: `base_path: PathBuf`.

**Directory layout:**
```
{base_path}/{namespace}/{key[..2]}/{key[2..]}
```
Keys shorter than 2 characters skip sharding: `{base_path}/{namespace}/{key}`.

`get_range` seeks to `range.start`, validates that both `start < file_size` and `end <= file_size` and `start < end`, then issues a single `read_exact` for the requested length.

`delete` is idempotent: `ErrorKind::NotFound` is suppressed and returns `Ok(())`.

`list` walks the two-level shard directory, reconstructing the full key by concatenating the shard directory name (first 2 chars) with the file name. Keys stored without sharding (len < 2) are invisible to `list` because it only recurses into subdirectories.

`get_stream` returns a `Box<tokio::fs::File>` opened for reading.

#### `S3Storage`

Fields: `client: aws_sdk_s3::Client`, `bucket: String`, `prefix: Option<String>`.

Key format: `{prefix}{namespace}/{key}` when a prefix is set, else `{namespace}/{key}`. The prefix is not a path separator — callers must include a trailing `/` if they want one.

`get_range` translates the half-open Rust range `[start, end)` to the S3 inclusive range header `bytes=start-(end-1)`.

`list` uses `ListObjectsV2` with pagination (continuation token loop) and strips the `{ns_prefix}` before returning keys, so callers see unqualified keys.

`exists` uses `HeadObject`; `size` also uses `HeadObject.content_length`.

`put_from_file` uses `ByteStream::from_path` — a single-part upload with no multipart support.

`get_stream` returns `result.body.into_async_read()` from the S3 response.

Credential resolution: `aws_config::load_from_env()` is called at construction; a separate `with_credentials(access_key, secret_key)` constructor is provided for testing.

## Intra-module dependencies

The `storage` module has no imports from other OpenXet modules. All dependencies are external crates and the standard library.

| External crate | Consuming file | What is used |
| --- | --- | --- |
| `async_trait` | `backend.rs`, `local.rs`, `s3.rs` | `#[async_trait]` macro |
| `bytes` | `backend.rs`, `local.rs`, `s3.rs` | `Bytes` type for zero-copy data |
| `tokio` | `local.rs` | `tokio::fs`, `AsyncReadExt`, `AsyncSeekExt` |
| `aws_sdk_s3` | `s3.rs` | `Client`, `Builder`, `Credentials`, `Region`, `ByteStream` |
| `aws_config` | `s3.rs` | `load_from_env()` for credential chain |

Reverse dependencies (other OpenXet modules that directly import from `storage` in the visible source):

| Module | Symbols imported |
| --- | --- |
| `cas::store` | `namespaces`, `StorageBackend`, `StorageConfig` |

## Notable implementation details

**`#![allow(dead_code)]` is module-wide.** The annotation in `mod.rs` suppresses unused-method warnings for the entire storage module. Several `StorageBackend` methods (e.g. `get_to_file`, `put_from_file`, `get_stream`, `list`, `size`) appear unused by the current callers in `cas::store`, consistent with the dead-code suppression.

**`REPO_META` namespace is declared but unused.** The constant `"repo-meta"` is defined in `namespaces` with no call sites in the visible codebase, suggesting it is reserved for future repository metadata persistence.

**`CAS_CHUNKS` is deprecated in-source.** The inline comment `// deprecated, use BLOCKS` documents a completed or in-progress migration from per-chunk storage to bundled block storage.

**`list` on `LocalStorage` misses short-key objects.** The implementation walks only subdirectories of the namespace directory. Objects stored with keys shorter than 2 characters are written directly into the namespace directory (no shard sub-dir) and are therefore invisible to `list`. This is a latent bug if keys < 2 chars are ever used (currently only hex content hashes — 64 chars — are used, so the bug is dormant).

**S3 error detection uses string matching.** `S3Storage` detects `NotFound` by checking `e.to_string().contains("NoSuchKey")` or `".contains("404")`. This is fragile: SDK version changes or non-standard S3 implementations may produce different message strings, causing `NotFound` to be misclassified as `Other`.

**Single-part S3 upload only.** `put_from_file` uses `ByteStream::from_path`, which issues a single `PutObject` call with no multipart support. S3 rejects single-part uploads larger than 5 GiB. No multipart path exists for blocks uploaded via this module.

**`S3Config::minio` hardcodes region `"us-east-1"`.** The comment "MinIO doesn't care about region" explains the choice. Other S3-compatible stores (e.g. GCS S3-interop, R2) may require a real region value; using `minio()` as a generic "custom-endpoint" constructor may fail against those systems.

**Range semantics are consistent but must be coordinated.** The `StorageBackend` trait documents `[start, end)` (exclusive end). `LocalStorage` enforces this with an explicit validity check. `S3Storage` converts to S3's inclusive range header (`end - 1`). Any third-party implementation must follow the same convention.

**`LocalStorage::exists` is synchronous under the hood.** The async method calls `path.exists()` (a sync `std::path::Path` method) inside an async context — no `spawn_blocking` wrapper. For hot paths this may inadvertently block a Tokio worker thread on slow filesystems.

**`put` on `LocalStorage` uses `fs::write` with no atomic swap.** Concurrent puts to the same key can produce partial writes visible to concurrent readers. There is no write-then-rename pattern to make puts atomic.

**Tests are local-only.** `local.rs` contains four `#[tokio::test]` integration tests covering basic put/get, list, streaming, and range reads. There are no tests for `S3Storage` in the source tree — S3 correctness must be validated against a live endpoint (MinIO or AWS).

## Open questions

- `StorageError` and `StorageResult` are internal to `backend.rs` and not re-exported. Callers in `cas::store` that receive a `StorageError` must convert it to `crate::error::ServerError` manually. Is this the intended boundary, or should `StorageError` become part of the public API?
- `REPO_META` is the only namespace with no known callers. Is repository metadata currently stored elsewhere (e.g. SQLite), or is a future migration to `StorageBackend`-backed metadata planned?
- `namespaces::CAS_CHUNKS` is marked deprecated. Has all production data migrated to `BLOCKS`, or do deployments still need to read from `cas-chunks`?
- `LocalStorage::exists` calls `path.exists()` synchronously inside an async context. Is a `spawn_blocking` wrapper needed, or is local storage considered low-latency enough to call synchronously?
- `put` on `LocalStorage` has no atomic write-then-rename. Can concurrent puts to the same key corrupt data, and is this a real concern given the immutable-content (hash-keyed) usage pattern?
- `put_from_file` on `S3Storage` uses single-part upload. Is multipart upload planned for block objects that exceed 5 GiB, and if so, would it live in this module or in `cas::store`?
- The `with_credentials` constructor on `S3Storage` is documented "for testing". Is there a deployment scenario where static credentials are preferred over the environment credential chain, or is this constructor expected to be removed?
- S3 `NotFound` detection relies on string matching. Is there a plan to use typed SDK error variants instead, which would be more robust across SDK versions and compatible S3 implementations?
- `LocalStorage::list` is invisible to short-key objects. Is the constraint that all keys are always long hex strings (≥ 2 chars) formally enforced anywhere, or is this an implicit assumption?
- The `get_stream` return type is `Box<dyn AsyncRead + Unpin + Send>`. Should it be `impl futures::Stream<Item = Bytes>` to integrate more cleanly with axum response bodies without an intermediate `ReaderStream` adapter?
