# Round 2 Comparison — `storage`

**OpenXet commit:** `a5b6dfcdf5e806a5a29cb893f0d79d733bea309b`
**Analyst:** DiTo97
**Date:** 2026-03-31
**Tracking issue:** [#52](https://github.com/Atena-IT/open-hub/issues/52)
**Round 1 source:** [`round1/openxet_storage.md`](../round1/openxet_storage.md)

---

## Scope

This document compares the OpenXet `storage` module (mapped in Round 1) against the corresponding implementation in `xet-backend`. Its goal is to enumerate gaps, alignment, and surfaces that need decision. The OpenXet storage module provides a `StorageBackend` trait with two implementations (local filesystem and S3), namespace constants, configuration factories, and key-generation helpers. xet-backend distributes equivalent functionality across the `s3-storage` crate, the `common` crate, and inline S3 calls in `cas-server` and `hub-api` route handlers.

## xet-backend counterpart

xet-backend does not have a single unified storage abstraction equivalent to OpenXet's `StorageBackend` trait. Instead, it uses a concrete `S3Client` struct as the sole object-storage backend, with S3 key-construction helpers in the `s3-storage` crate and direct `S3Client` method calls scattered across the `cas-server` and `hub-api` crates.

| Responsibility | xet-backend location | Notes |
| --- | --- | --- |
| S3 object storage client | `crates/s3-storage/src/lib.rs` (`S3Client`) | Concrete struct, not a trait. Holds two SDK `Client` instances (internal + presigning). |
| S3 key generation (xorbs) | `crates/s3-storage/src/lib.rs` (`xorb_key`) | `xorbs/{hex[..2]}/{hex[2..4]}/{hex}` -- three-level prefix sharding. |
| S3 key generation (shards) | `crates/s3-storage/src/lib.rs` (`shard_key`) | `shards/{hex[..2]}/{hex}` -- two-level prefix sharding. |
| S3 key generation (LFS) | `crates/s3-storage/src/lib.rs` (`lfs_key`) | `lfs/{oid[..2]}/{oid[2..4]}/{oid}` -- three-level prefix sharding. |
| S3 key generation (repo files) | `crates/s3-storage/src/lib.rs` (`repo_file_key`) | `files/{repo_full_name}/{sha256}` -- no shard prefix. |
| Storage configuration | `crates/common/src/config.rs` (`AppConfig`) | S3 endpoint, region, bucket sourced from env vars. No `StorageType` enum; S3 is the only backend. |
| Dual endpoint (internal vs. presign) | `crates/s3-storage/src/lib.rs` (`with_public_endpoint`) | Separate presign client for external URLs, vs. internal endpoint for server-side ops. |
| Hash encoding (API wire format) | `crates/common/src/types.rs` (`hash_to_api_string`, `api_string_to_hash`) | Xet-protocol-specific LE-u64-group encoding. Not present in OpenXet. |
| Error types for storage failures | `crates/common/src/error.rs` (`AppError`) | Unified application error enum, not a storage-specific type. |
| CAS route: upload xorb | `crates/cas-server/src/routes/xorbs.rs` | Calls `S3Client::put_object` directly. |
| CAS route: upload shard | `crates/cas-server/src/routes/shards.rs` | Calls `S3Client::put_object` and `S3Client::object_exists`. |
| CAS route: reconstruction | `crates/cas-server/src/routes/reconstructions.rs` | Calls `S3Client::presign_get` to generate presigned URLs. |
| CAS route: dedup query | `crates/cas-server/src/routes/chunks.rs` | Pure DB lookup; does not call storage directly. |
| Hub: LFS upload/download | `crates/hub-api/src/routes/lfs.rs` | Presigned PUT for upload; presigned GET + redirect for download. |
| Hub: file commit (regular) | `crates/hub-api/src/routes/files.rs` (`commit`) | Inline `S3Client::put_object` for each file in the NDJSON body. |
| Hub: file resolve | `crates/hub-api/src/routes/files.rs` (`resolve_file`) | `S3Client::get_object` for regular files; presigned redirect for LFS. |
| DB: xorb metadata | `crates/db-layer/src/queries/xorbs.rs` | `xorbs` table stores hash, s3_key, size_bytes. Idempotent upsert. |
| DB: chunk metadata | `crates/db-layer/src/queries/xorbs.rs` | `chunks` table with FK to xorbs. Row-by-row upsert (no batch insert). |
| DB: file reconstruction | `crates/db-layer/src/queries/file_mappings.rs` | `file_mappings` table with JSONB reconstruction terms. |
| DB: LFS objects | `crates/db-layer/src/queries/lfs_objects.rs` | `lfs_objects` table keyed by (repo_id, oid). |
| DB: repo files | `crates/db-layer/src/queries/repo_files.rs` | `repo_files` table tracks current file state per repo. |

## Surface comparison

Status values: `covered` | `partial` | `missing` | `out-of-scope`

| OpenXet symbol / responsibility | Status | xet-backend equivalent | Notes |
| --- | --- | --- | --- |
| `StorageBackend` trait (10 async methods) | `missing` | `S3Client` struct (5 methods) | No trait abstraction. Only `put_object`, `get_object`, `object_exists`, `presign_get`, `presign_put` exposed. |
| `StorageBackend::get` | `covered` | `S3Client::get_object` | Full object fetch. |
| `StorageBackend::get_range` | `missing` | -- | No range-read method on `S3Client`. Reconstruction route works around this by issuing full presigned GET URLs. |
| `StorageBackend::put` | `covered` | `S3Client::put_object` | Single-part upload. |
| `StorageBackend::delete` | `missing` | -- | No delete method on `S3Client`. |
| `StorageBackend::exists` | `covered` | `S3Client::object_exists` | Uses `HeadObject` with typed SDK error check (`is_not_found()`). |
| `StorageBackend::list` | `missing` | -- | No list/enumerate method. |
| `StorageBackend::size` | `missing` | -- | No size-only query; `object_exists` discards content-length. |
| `StorageBackend::get_to_file` | `missing` | -- | No file-download helper. |
| `StorageBackend::put_from_file` | `missing` | -- | No file-upload helper; all uploads pass through `Bytes`. |
| `StorageBackend::get_stream` | `missing` | -- | No streaming reader. `get_object` collects the full body into `Bytes`. |
| `LocalStorage` (filesystem backend) | `out-of-scope` | -- | xet-backend targets production S3 only; local storage was never part of the design. |
| `StorageConfig` / `StorageType` enum | `partial` | `AppConfig` | S3-only configuration via env vars. No `StorageType` enum or factory pattern. |
| `StorageConfig::build` / `build_local` factory | `missing` | `S3Client::new` / `S3Client::with_public_endpoint` | No polymorphic factory; callers always construct an `S3Client` directly. |
| `namespaces::GIT_OBJECTS` | `out-of-scope` | -- | xet-backend does not store git objects in the storage layer. |
| `namespaces::GIT_PACKS` | `out-of-scope` | -- | xet-backend does not handle git pack files. |
| `namespaces::CAS_CHUNKS` (deprecated) | `out-of-scope` | -- | Deprecated in OpenXet; xet-backend uses xorb-level storage only. |
| `namespaces::BLOCKS` | `covered` | `xorb_key()` | xet-backend stores CAS blocks under the `xorbs/` S3 prefix. Different naming (Xorb vs. Block), same concept. |
| `namespaces::LFS_RAW` | `covered` | `lfs_key()` | xet-backend stores LFS objects under the `lfs/` S3 prefix. |
| `namespaces::REPO_META` | `out-of-scope` | -- | Unused in OpenXet; no counterpart in xet-backend. |
| `StorageError` (4 variants) | `partial` | `AppError` (8 variants) | `AppError` is application-wide, not storage-specific. Maps: `NotFound` -> `AppError::NotFound`, `Other` -> `AppError::Internal`. No `InvalidRange` or `Io` variants. |
| `S3Config` (bucket, region, endpoint, force_path_style) | `covered` | `AppConfig` fields + `S3Client` constructor | Config fields: `s3_bucket`, `s3_region`, `s3_endpoint`. `force_path_style` is always `true`. |
| `S3Storage::with_credentials` (testing constructor) | `covered` | `S3Client::new` (always takes credentials) | xet-backend always uses explicit credentials; no env-chain fallback. |
| S3 NotFound detection via string matching | `covered` | `S3Client::object_exists` uses `is_not_found()` | xet-backend uses typed SDK error variant, not string matching -- this is an improvement. |
| Dual endpoint (internal vs. public presigning) | `covered` | `S3Client::with_public_endpoint` | Separate presign client built when `S3_PUBLIC_ENDPOINT` differs from `S3_ENDPOINT`. |
| Shard binary parsing | `covered` | `crates/shard-parser/` | Full MDB shard parser with reader and writer. Separate crate. |
| Content-hash encoding (Xet LE-u64 groups) | `covered` | `crates/common/src/types.rs` | `hash_to_api_string` / `api_string_to_hash`. Not present in OpenXet. |

## Gaps

### Missing

- **`StorageBackend` trait abstraction.** xet-backend has no pluggable storage interface. `S3Client` is a concrete struct, and every caller imports it directly. Adding a second backend (e.g., local filesystem for testing, GCS, or Azure Blob) would require refactoring all call sites. OpenXet's `Arc<dyn StorageBackend>` allows runtime backend selection.

- **Range-read (`get_range`).** `S3Client` has no partial-fetch method. The reconstruction endpoint generates presigned GET URLs for full xorb downloads and delegates range selection to the client. Server-side range reads (needed for partial reconstruction or serving byte ranges) cannot be done without adding this method.

- **Object deletion (`delete`).** No S3 `DeleteObject` wrapper exists. There is no way to remove an orphaned xorb, shard, or LFS object from S3 through the application layer. This blocks garbage collection and retention policy features.

- **Object enumeration (`list`).** No `ListObjectsV2` wrapper. Auditing, garbage collection, and data migration scenarios that need to enumerate S3 contents are not supported.

- **Size query (`size`).** `object_exists` uses `HeadObject` but discards the content-length. A separate `size()` method or returning metadata from `exists()` would fill this gap.

- **Streaming read (`get_stream`).** `get_object` collects the full S3 response body into a `Bytes` buffer. For large objects this allocates the entire content in memory. A streaming variant returning `impl AsyncRead` or `impl Stream<Item = Bytes>` is absent.

- **File I/O helpers (`get_to_file`, `put_from_file`).** No convenience methods for local-file-to-S3 or S3-to-local-file transfers. These are low-priority in a server context where all I/O is network-to-network.

- **Multipart upload.** `S3Client::put_object` is single-part only (5 GiB ceiling), matching OpenXet's limitation. Neither system has multipart support, but xet-backend's Xorb max is 64 MiB, making this a lower-risk gap than for LFS raw objects (which can be arbitrarily large).

### Partial

- **Configuration / factory pattern.** `AppConfig` covers all S3 connection parameters but lacks the polymorphic `StorageConfig::build` factory. There is no way to switch backends at runtime or via configuration. The `StorageType::Local` path has no counterpart; integration tests always need an S3 endpoint (MinIO via Docker Compose).

- **Error classification.** `AppError` covers `NotFound` and `Internal` but has no dedicated `InvalidRange` or `Io` variants for storage-specific failures. Storage errors are uniformly wrapped as `AppError::Internal(e.to_string())`, losing error type information. OpenXet's `StorageError` preserves distinctions useful for retry logic and HTTP status mapping.

- **Namespace organization.** xet-backend uses four implicit S3 prefix namespaces (`xorbs/`, `shards/`, `lfs/`, `files/`) defined in free functions rather than a centralized namespace module. The mapping is correct but the lack of a single namespace definition point makes it harder to audit or extend.

## Already covered

- **S3 object put/get.** Core put and get operations are implemented and exercised by the CAS upload/download flow and the Hub file commit/resolve flow.

- **Presigned URL generation.** Both presigned GET and PUT are implemented, with dual-endpoint support for separating internal and client-facing URLs. This matches OpenXet's S3 layer and adds the public-endpoint separation that OpenXet does not have.

- **S3 existence check.** `object_exists` uses typed SDK error detection (`is_not_found()`), which is more robust than OpenXet's string-matching approach.

- **Shard binary format.** The `shard-parser` crate implements full MDB shard parsing (reader) and dedup response shard construction (writer), covering the binary wire format that OpenXet handles through `cas::store`.

- **Hash encoding.** The Xet-protocol-specific hash encoding (LE-u64 groups to 64-char hex) is implemented and tested with known vectors. OpenXet uses plain SHA-256 hex and does not need this encoding.

- **Content-addressed key sharding.** S3 keys use two- or three-level hex prefix sharding (`xorb_key`, `shard_key`, `lfs_key`), distributing objects evenly across S3 partition prefixes. OpenXet's `S3Storage` uses a simpler `{namespace}/{key}` layout.

- **DB-backed metadata.** Xorb, chunk, file mapping, and LFS object metadata are persisted in PostgreSQL with proper schemas, indexes, and FK constraints (`chunks.xorb_hash` references `xorbs.hash`). This is structurally stronger than OpenXet's in-memory DashMaps with fire-and-forget SQLite writes.

- **LFS basic transfer.** The full LFS batch/upload/verify/download flow works via presigned S3 URLs, with DB tracking of object state. The `xet` transfer type is negotiated in the batch response.

## Out-of-scope items

- **`LocalStorage` (filesystem backend).** xet-backend is S3-only by design. Local filesystem storage was never part of the architecture. OpenXet's `LocalStorage` with its sharded two-level directory tree has no counterpart and is not planned. Integration tests use MinIO in Docker Compose as the S3 target.

- **`namespaces::GIT_OBJECTS` / `GIT_PACKS`.** xet-backend does not implement a custom git object store. Git operations are handled at the Hub API level (repo files in PostgreSQL + S3) rather than through a storage-backend-backed git layer.

- **`namespaces::CAS_CHUNKS` (deprecated).** Already deprecated in OpenXet in favor of `BLOCKS`. xet-backend skipped this entirely and only implements block-level (Xorb) storage.

- **`namespaces::REPO_META`.** Declared but unused in OpenXet. No counterpart needed.

## Recommendations for synthesis

- **Trait abstraction decision.** The most significant architectural difference is the absence of a `StorageBackend` trait in xet-backend. If the project needs to support alternative backends (local testing without Docker, GCS, Azure Blob) or wants to enable mock-based unit testing of routes that touch S3, introducing a trait analogous to OpenXet's would be the highest-leverage change. If S3 remains the sole target, the concrete `S3Client` approach is simpler and sufficient. This should be decided before any attempt to port OpenXet storage features.

- **Add `get_range` before range-aware reconstruction.** The `reconstructions.rs` endpoint currently returns presigned URLs with the expectation that clients do their own range selection. If server-side partial-file serving is needed (e.g., for the `Range` header support TODO already flagged in the code at `reconstructions.rs:112-118`), `S3Client` needs a `get_range` method wrapping the S3 `Range` header.

- **Add `delete` before any GC or retention work.** Without a delete method, orphaned objects accumulate in S3. This should be added early in any operations or lifecycle management work.

- **Chunk batch insert.** `upsert_chunks` in `crates/db-layer/src/queries/xorbs.rs:45-63` inserts chunks row-by-row in a loop. For xorbs with many chunks, this is N round-trips to PostgreSQL. A batch insert (using `UNNEST` arrays or a CTE) would reduce this to a single round-trip. This is an implementation quality issue, not a feature gap versus OpenXet.

- **Implementation ordering for convergence.** If the goal is to converge with OpenXet's storage surface: (1) `get_range`, (2) `delete`, (3) `list`, (4) optionally `size`. Streaming (`get_stream`) and file I/O helpers are lower priority in a server context.

## Open questions

- Should xet-backend introduce a `StorageBackend` trait to enable backend polymorphism and testability, or is the concrete `S3Client` approach the long-term design?

- Is local-filesystem storage needed for integration tests that run without Docker/MinIO, or is the current Docker Compose setup the permanent test infrastructure?

- The `files/` S3 prefix namespace (for Hub file commits) has no counterpart in OpenXet. Is this namespace permanent, or is it a transitional shim until all uploads go through the CAS/LFS pipeline?

- `S3Client` always sets `force_path_style(true)`. This is correct for MinIO but may cause issues with AWS S3 in non-us-east-1 regions where virtual-hosted-style is required. Should this be configurable?

- The `reconstructions.rs` TODO at line 112-118 notes that `offset_into_first_range` is always 0. Is implementing partial-range reconstruction a near-term priority, and if so, does it depend on a server-side `get_range` for xorbs?

- LFS objects uploaded via presigned PUT have no size ceiling enforced by xet-backend (the 5 GiB limit is an S3 constraint). Should multipart upload support be added for large LFS objects, or is the 5 GiB single-part ceiling acceptable?
