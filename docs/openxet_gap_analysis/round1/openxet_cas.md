# Round 1 Module Map — `cas`

**OpenXet commit:** `a5b6dfcdf5e806a5a29cb893f0d79d733bea309b`
**Analyst:** DiTo97
**Date:** 2026-03-31
**Tracking issue:** [#41](https://github.com/Atena-IT/open-hub/issues/41)

---

## Scope

This document maps the OpenXet `cas` module. It covers only the upstream OpenXet source — no `xet-backend` references appear here. Round 2 will introduce the comparison.

## Source location

```
src/cas/
```

Key entry points:
- `src/cas/mod.rs` — re-exports everything from `store` via `pub use store::*;`
- `src/cas/store.rs` — entire implementation (~2079 lines)

## Public surface

| Symbol | Kind | File | Brief description |
| --- | --- | --- | --- |
| `TARGET_CDC_CHUNK_SIZE` | `const` | `src/cas/store.rs` | Target CDC chunk size: 16 KiB |
| `MIN_CHUNK_SIZE` | `const` | `src/cas/store.rs` | Minimum chunk size: 4 KiB (target / 4) |
| `MAX_CHUNK_SIZE` | `const` | `src/cas/store.rs` | Maximum chunk size: 128 KiB (target × 8) |
| `STREAM_BUFFER_SIZE` | `const` | `src/cas/store.rs` | Read buffer size for streaming: 64 KiB |
| `TARGET_BLOCK_SIZE` | `const` | `src/cas/store.rs` | Target block bundle size: 64 MiB |
| `DEFAULT_FETCH_PARALLELISM` | `const` | `src/cas/store.rs` | Default concurrent S3 fetch count: 32 |
| `ContentHash` | `struct` | `src/cas/store.rs` | 32-byte SHA-256 hash newtype with hex encode/decode |
| `Chunk` | `struct` | `src/cas/store.rs` | A content-defined chunk: hash + raw bytes + size |
| `BlockChunkEntry` | `struct` | `src/cas/store.rs` | Chunk's position within a Block: hash, size, byte offset |
| `Block` | `struct` | `src/cas/store.rs` | ~64 MiB bundle of chunks; the unit of object storage |
| `FileSegment` | `struct` | `src/cas/store.rs` | Byte range within a Block for file reconstruction |
| `FileReconstruction` | `struct` | `src/cas/store.rs` | Ordered list of FileSegments that reconstitute a file |
| `ReconstructionStream` | `struct` | `src/cas/store.rs` | Async streaming reconstructor with parallel block fetches |
| `ChunkLocation` | `struct` | `src/cas/store.rs` | Fast lookup: which Block contains a chunk and at what range |
| `CasStore` | `struct` | `src/cas/store.rs` | Central CAS engine: chunking, dedup, block storage, reconstruction |
| `LfsObjectStatus` | `enum` | `src/cas/store.rs` | LFS object lifecycle: `Raw` → `Processing` → `Chunked` |
| `LfsObjectMeta` | `struct` | `src/cas/store.rs` | Metadata for an LFS object: status, size, raw path, chunk hashes |
| `LfsObjectSource` | `enum` | `src/cas/store.rs` | Where to read an LFS object: `RawFile(PathBuf)` or `Blocks(ContentHash)` |
| `LfsStats` | `struct` | `src/cas/store.rs` | Aggregate LFS storage statistics |
| `CasStats` | `struct` | `src/cas/store.rs` | Aggregate CAS storage statistics |
| `Chunker` | `struct` | `src/cas/store.rs` | Content-defined chunker using a gear-hash rolling hash |
| `StreamingChunker<R>` | `struct` | `src/cas/store.rs` | `Iterator`-based chunker that wraps any `Read` source |

## Internal structure

### Sub-modules

| Sub-module | Purpose |
| --- | --- |
| `store` | Entire CAS implementation; re-exported flat from `cas` |

The module has no further sub-module division — `store.rs` is a single monolithic file.

### Key types

#### `ContentHash`

A 32-byte SHA-256 digest newtype. Provides:
- `from_data(&[u8])` — hash raw bytes with `sha2::Sha256`
- `from_raw([u8; 32])` — wrap pre-computed bytes without hashing
- `from_hex(&str)` / `to_hex()` — hex string round-trip (64 chars, big-endian byte order)

Derives `Copy`, `Clone`, `PartialEq`, `Eq`, `Hash`. Display prints full 64-char hex; Debug prints first 16 chars.

#### `Block`

The primary unit of persistent storage, analogous to an Xorb in the xet-core protocol. A Block is assembled from a batch of content-defined chunks whose total size targets ~64 MiB. Its hash is the SHA-256 of all chunk bytes concatenated in order. Key method:

- `from_chunks(chunks: &[(ContentHash, Bytes)]) -> (Block, Bytes)` — builds the Block struct and returns the raw concatenated payload to store.
- `chunk_byte_range(idx)` / `chunk_range_bytes(start, end)` — look up byte ranges within the block for partial S3 fetches.

#### `FileReconstruction`

Analogous to a "shard": maps a file's `ContentHash` to an ordered `Vec<FileSegment>`. Each `FileSegment` points to `(block_hash, byte_start, byte_end)` so the server can issue HTTP Range requests against blocks to reconstruct the file without reading entire blocks. `total_size` is computed as the sum of all `segment_size` values.

#### `CasStore`

The central engine. All state is held in-memory using `dashmap::DashMap` for concurrent access, with optional async write-through to SQLite (SeaORM). Fields:

| Field | Type | Role |
| --- | --- | --- |
| `storage` | `Arc<dyn StorageBackend>` | Pluggable backend (local disk or S3) |
| `chunk_index` | `DashMap<ContentHash, u64>` | Dedup index: known chunk hash → size |
| `chunk_locations` | `DashMap<ContentHash, ChunkLocation>` | O(1) chunk → (block, range) lookup |
| `blocks` | `DashMap<ContentHash, Block>` | Block metadata in memory |
| `reconstructions` | `DashMap<ContentHash, FileReconstruction>` | File reconstruction records |
| `lfs_objects` | `DashMap<ContentHash, LfsObjectMeta>` | LFS object lifecycle state |
| `chunker` | `Chunker` | CDC chunker instance |
| `db` | `Option<Arc<DatabaseConnection>>` | SeaORM connection; absent until `set_db()` is called |
| `process_tx` | `Option<mpsc::UnboundedSender<ContentHash>>` | Channel to background processing worker |
| `pending_count` | `AtomicU64` | Inflight background jobs counter |

Important construction path: `new()` → `with_storage_path()` → `with_storage_config()` are sync and support local storage only. `with_storage_config_async()` must be used for S3. The database is injected post-construction via `set_db()` + `load_from_db()`.

#### `Chunker`

CDC chunker using a gear-hash rolling hash over a 256-entry lookup table (`GEAR_TABLE`) generated at compile time by a LCG. `Chunker::new()` hardcodes default parameters of min=8 KiB, max=64 KiB, target=16 KiB, which differ from the exported module constants `MIN_CHUNK_SIZE` (4 KiB) and `MAX_CHUNK_SIZE` (128 KiB). The boundary mask is `(1 << floor(log2(target))) - 1`.

Public methods:
- `chunk(&[u8]) -> Vec<Bytes>` — chunk in-memory slice
- `find_all_boundaries(&[u8]) -> Vec<usize>` — return all boundary offsets (used for parallel hashing)
- `chunk_streaming<R: Read>() -> StreamingChunker<R>` — wrap a reader in an iterator

#### `ReconstructionStream`

Wraps `Vec<FileSegment>` and drives async parallel block fetches using `futures::stream::buffer_unordered(parallelism)`. Maintains a `pending` HashMap for out-of-order arrivals so segments are yielded in correct order. Exposes `into_stream()` which converts to `impl futures::Stream` for use with axum response bodies.

## Intra-module dependencies

| Dependency module | What is used |
| --- | --- |
| `crate::db::entities::lfs_object` | SeaORM entity for LFS object rows |
| `crate::db::entities::lfs_chunk` | SeaORM entity for per-chunk hash rows tied to an LFS object |
| `crate::db::entities::cas_chunk` | SeaORM entity for chunk rows within a Block |
| `crate::db::entities::cas_block` | SeaORM entity for Block metadata rows |
| `crate::db::entities::file_segment` | SeaORM entity for FileReconstruction segment rows |
| `crate::error::{Result, ServerError}` | Unified error type (`ServerError::Internal`, `ObjectNotFound`) |
| `crate::storage::{namespaces, StorageBackend, StorageConfig}` | Storage trait, namespace constants (`BLOCKS`, `CAS_CHUNKS`, `LFS_RAW`), and config |

External crate dependencies: `bytes`, `dashmap`, `sea_orm`, `sha2`, `tokio`, `futures`.

## Notable implementation details

**Hashing algorithm is SHA-256, not Blake3.** All `ContentHash` values are plain `sha2::Sha256` digests. There is no keyed hashing (no HMAC, no Blake3 keyed mode). Block hashes are computed the same way: SHA-256 over the concatenation of all chunk bytes.

**Gear hash table is LCG-generated at compile time.** The `GEAR_TABLE` is a `static [u64; 256]` computed in a `const` block using the LCG `state = state * 6364136223846793005 + 1` seeded at `0x123456789abcdef0`. This is not a standard FastCDC table.

**"Streaming" chunker actually buffers everything in memory.** `chunk_and_store_streaming` calls `reader.read_to_end(&mut data)` before delegating to `chunk_and_store_parallel_from_bytes`. The streaming API is a misnomer — full file data is in memory before chunking begins.

**Parallel hashing via `std::thread`.** After boundaries are found (sequential), chunks are distributed across `min(available_parallelism, 8)` threads via `std::mpsc` channel. Data is shared read-only behind `Arc<Vec<u8>>`.

**Two-phase upload for LFS objects.** LFS objects are written to raw disk immediately (`store_lfs_object_stream` or `store_lfs_object`), then queued via `mpsc::UnboundedSender<ContentHash>` to a background Tokio task that calls `process_raw_object`. This keeps push latency low by deferring chunking.

**Database writes are fire-and-forget.** All SeaORM persistence is done inside `tokio::spawn(async move { ... })` calls with no error propagation back to the caller. Failures are silently swallowed (the `let _ = ...` pattern).

**`#[allow(dead_code)]` is file-wide.** The module annotation suppresses all unused-method warnings, indicating the public API is larger than what is currently exercised by the rest of the binary.

**Block boundary condition in parallel chunker.** `chunk_and_store_parallel_from_bytes` assembles `all_chunks` by first pushing `(0, first_boundary)` and then iterating windows. If `chunk_boundaries` has N entries, this produces N+1 ranges but the logic for handling the remainder after the last boundary may double-count or omit chunks depending on the boundaries list.

**`store_block_sync` writes blocks with a two-level directory prefix.** Block files are stored at `{storage_path}/{namespace}/{hex[..2]}/{hex[2..]}`, the same layout used for chunks. This is a local-only code path; the async `store_block` delegates to the `StorageBackend` trait.

**LFS reconstruction fallback.** `get_lfs_object` reads directly from the raw file path for `Raw` and `Processing` states, and calls `reconstruct_file_sync` for `Chunked`. The sync reconstruction reads full Block files from disk one at a time — no parallelism.

## Open questions

- Why SHA-256 instead of Blake3? The code comment says "aligned with xet-core" for chunk sizes, but xet-core uses Blake3 keyed hashing. Is the SHA-256 choice intentional for this implementation or a placeholder?
- The gear hash table is generated by a non-standard LCG; is the resulting chunking quality (boundary distribution) validated against benchmarks or known CDC tables?
- `chunk_and_store_streaming` reads all bytes into memory before chunking, despite its name. Is a true constant-memory streaming path planned?
- The `db` field is injected post-construction via `set_db()`. Is there a data race window where LFS objects written before `set_db()` is called are never persisted to SQLite?
- `store_block_sync` writes directly to `storage_path` for local storage, bypassing `StorageBackend`. Does this mean blocks written via the sync path are invisible to an S3-backed `StorageBackend::get_range` call later?
- The `chunk_and_store_parallel_from_bytes` boundary-assembly logic (`all_chunks` construction) appears to double-count or miscount chunk ranges in certain cases. Is this exercised by any test?
- All database writes use `let _ = ...` ignoring errors. Is there a durability guarantee (e.g., WAL checkpoint) before an upload response is sent to the client?
- `LfsStats::dedup_ratio` is computed as `total_logical_size / total_physical_size` but `total_physical_size` sums raw chunk sizes from `chunk_index` — this omits block overhead and counts only unique chunks, not all references. Is this the intended semantic?
