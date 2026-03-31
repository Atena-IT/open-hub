# Round 2 Comparison — `cas`

**OpenXet commit:** `a5b6dfcdf5e806a5a29cb893f0d79d733bea309b`
**Analyst:** DiTo97
**Date:** 2026-03-31
**Tracking issue:** [#50](https://github.com/Atena-IT/open-hub/issues/50)
**Round 1 source:** [`round1/openxet_cas.md`](../round1/openxet_cas.md)

---

## Scope

This document compares the OpenXet `cas` module (mapped in Round 1) against the corresponding implementation in `xet-backend`. The comparison covers content-addressed storage operations: xorb storage, shard parsing and persistence, chunk-level deduplication, file reconstruction, and LFS object management. It identifies where the two implementations are aligned, where they diverge, and where gaps exist.

## xet-backend counterpart

The OpenXet `cas` module is a monolithic ~2079-line `store.rs` that bundles chunking, hashing, block assembly, dedup indexing, reconstruction, and LFS lifecycle into a single `CasStore` struct. In xet-backend, these responsibilities are distributed across four crates with separate concerns:

| Responsibility | xet-backend location | Notes |
| --- | --- | --- |
| Xorb upload and storage | `crates/cas-server/src/routes/xorbs.rs` | Receives pre-built xorbs from the client; no server-side chunking |
| Shard parsing (binary format) | `crates/shard-parser/src/reader.rs` | Parses MDB shard binary (Header + FileInfo + CASInfo) |
| Shard response writing | `crates/shard-parser/src/writer.rs` | Builds minimal dedup-response shards with footer |
| Shard upload handler | `crates/cas-server/src/routes/shards.rs` | Orchestrates parse -> verify xorbs -> persist metadata -> archive |
| Chunk dedup query | `crates/cas-server/src/routes/chunks.rs` | `GET /v1/chunks/{prefix}/{hash}` returns shard binary |
| File reconstruction query | `crates/cas-server/src/routes/reconstructions.rs` | `GET /v1/reconstructions/{file_id}` returns JSON with presigned S3 URLs |
| Xorb/chunk DB persistence | `crates/db-layer/src/queries/xorbs.rs` | PostgreSQL upserts for xorbs and chunks tables |
| File mapping DB persistence | `crates/db-layer/src/queries/file_mappings.rs` | PostgreSQL upsert with JSONB reconstruction_terms |
| Object storage (S3) | `crates/s3-storage/src/lib.rs` | AWS SDK S3 client with presigning, dual endpoint |
| Hash encoding/decoding | `crates/common/src/types.rs` | `MerkleHash`, `hash_to_api_string`, `api_string_to_hash` |
| Token vending (CAS auth) | `crates/cas-server/src/routes/auth.rs` | JWT-based xet-read-token / xet-write-token |
| JWT middleware | `crates/cas-server/src/middleware.rs` | Bearer token validation for CAS routes |
| LFS object management | `crates/hub-api/src/routes/lfs.rs` + `crates/db-layer/src/queries/lfs_objects.rs` | Separate from CAS; lives in hub-api crate |

## Surface comparison

Status values: `covered` | `partial` | `missing` | `out-of-scope`

| OpenXet symbol / responsibility | Status | xet-backend equivalent | Notes |
| --- | --- | --- | --- |
| `TARGET_CDC_CHUNK_SIZE` (16 KiB) | `missing` | No server-side chunking | xet-backend delegates chunking to the xet-core client; server receives pre-chunked xorbs |
| `MIN_CHUNK_SIZE` (4 KiB) | `missing` | No server-side chunking | Same reason as above |
| `MAX_CHUNK_SIZE` (128 KiB) | `missing` | No server-side chunking | Same reason as above |
| `STREAM_BUFFER_SIZE` (64 KiB) | `missing` | No server-side chunking | Not needed; no streaming chunker on the server |
| `TARGET_BLOCK_SIZE` (64 MiB) | `missing` | No server-side block assembly | Xorbs arrive pre-assembled from the client |
| `DEFAULT_FETCH_PARALLELISM` (32) | `missing` | Not applicable | Reconstruction returns presigned URLs; clients do their own parallel fetches |
| `ContentHash` (SHA-256 newtype) | `partial` | `crates/common/src/types.rs:3` `MerkleHash` | `MerkleHash` is a 32-byte newtype but is hash-algorithm-agnostic. No `from_data()` hashing method. Encoding uses the xet-core little-endian-grouped API format, not plain big-endian hex as in OpenXet |
| `Chunk` (hash + bytes + size) | `missing` | No server-side type | Server does not hold chunk data in memory; chunks are only metadata in DB |
| `BlockChunkEntry` | `partial` | `crates/db-layer/src/queries/xorbs.rs:97` `ChunkRecord` | `ChunkRecord` stores the same fields (hash, xorb_hash, index, byte_range_start, unpacked_segment_bytes) in PostgreSQL |
| `Block` (~64 MiB chunk bundle) | `partial` | `xorbs` DB table + S3 objects | xet-backend stores xorbs as opaque S3 objects with a DB index (hash, s3_key, size_bytes). No in-memory `Block` struct |
| `FileSegment` (block range) | `covered` | `crates/db-layer/src/queries/file_mappings.rs:7` `ReconstructionTerm` | Maps to `ReconstructionTerm` with xorb_hash, chunk_index_start, chunk_index_end, unpacked_length |
| `FileReconstruction` (ordered segments) | `covered` | `file_mappings` DB table + `get_reconstruction` handler | Stored as JSONB array in PostgreSQL; served as JSON with presigned URLs |
| `ReconstructionStream` (async parallel fetch) | `missing` | Not applicable | xet-backend returns presigned URLs; the client reconstructs. No server-side streaming reconstruction |
| `ChunkLocation` (chunk -> block range) | `covered` | `chunks` DB table | `find_chunk` query returns the xorb_hash and byte range for a given chunk hash |
| `CasStore` (central engine) | `partial` | `crates/cas-server/src/state.rs:7` `AppState` | `AppState` holds `PgPool + S3Client + AppConfig`. No in-memory DashMaps; all state is DB-primary |
| `LfsObjectStatus` (Raw/Processing/Chunked) | `missing` | No lifecycle states | xet-backend LFS objects are stored directly to S3 with no deferred chunking pipeline |
| `LfsObjectMeta` | `partial` | `crates/db-layer/src/queries/lfs_objects.rs:4` `LfsObjectRow` | `LfsObjectRow` stores repo_id, oid, size, s3_key. No status field, no chunk_hashes |
| `LfsObjectSource` (RawFile/Blocks) | `missing` | Not applicable | No two-phase LFS; objects are served directly from S3 |
| `LfsStats` / `CasStats` | `missing` | No statistics endpoints | No aggregate storage statistics API |
| `Chunker` (gear-hash CDC) | `missing` | No server-side chunking | Client-side responsibility in xet-backend architecture |
| `StreamingChunker<R>` | `missing` | No server-side chunking | Same as above |
| `store_lfs_object` / `store_lfs_object_stream` | `missing` | Hub API `lfs.rs` handles LFS upload differently | LFS objects go directly to S3 via hub-api crate, not through CAS chunking pipeline |
| `process_raw_object` (background chunking) | `missing` | No background worker | No deferred chunking; xorbs arrive pre-chunked |
| `reconstruct_file_sync` / `reconstruct_file` | `missing` | Not applicable | Server does not reconstruct files; it provides presigned URLs |
| `get_lfs_object` / `get_lfs_object_source` | `partial` | Hub API LFS download | LFS download in hub-api serves the S3 object directly; no reconstruction from chunks |
| Dedup query endpoint | `covered` | `crates/cas-server/src/routes/chunks.rs:27` `dedup_query` | Returns MDB shard binary containing the xorb that holds the queried chunk |
| Shard upload endpoint | `covered` | `crates/cas-server/src/routes/shards.rs:19` `upload_shard` | Parses shard, verifies xorbs exist, persists chunk and file mapping metadata |
| Xorb upload endpoint | `covered` | `crates/cas-server/src/routes/xorbs.rs:27` `upload_xorb` | Receives raw xorb bytes, stores to S3, records in DB |
| File reconstruction endpoint | `covered` | `crates/cas-server/src/routes/reconstructions.rs:31` `get_reconstruction` | Returns JSON with terms and presigned S3 URLs per xorb |
| Token vending (xet-read/write-token) | `covered` | `crates/cas-server/src/routes/auth.rs:23-35` | JWT-based; returns accessToken, exp, casUrl |
| Health check | `covered` | `crates/cas-server/src/routes/health.rs:5` | DB ping, returns ok/degraded |

## Gaps

### Missing

- **Server-side CDC chunking.** OpenXet performs content-defined chunking on the server (gear-hash CDC with configurable min/max/target sizes). xet-backend has no server-side chunker; it relies entirely on the xet-core client to chunk, compress, and assemble xorbs before upload. This is an intentional architectural decision, not an oversight, but it means the server cannot independently chunk LFS objects or arbitrary file uploads.

- **Two-phase LFS upload pipeline.** OpenXet has a full LFS lifecycle (Raw -> Processing -> Chunked) with a background Tokio worker that asynchronously chunks raw LFS objects into Blocks. xet-backend stores LFS objects as-is in S3 (via `hub-api/routes/lfs.rs`). There is no background processing, no lifecycle states, and LFS objects are not integrated into the CAS deduplication layer.

- **Server-side file reconstruction.** OpenXet's `ReconstructionStream` fetches block ranges from S3 in parallel and streams the assembled file to the client. xet-backend returns presigned URLs in the reconstruction response, delegating all fetching and assembly to the client. This is lighter on server resources but means the server cannot serve file content directly from the CAS layer.

- **In-memory dedup index.** OpenXet maintains a `DashMap<ContentHash, u64>` `chunk_index` for O(1) dedup checks without hitting the DB. xet-backend queries PostgreSQL for every dedup lookup (`find_chunk`). This is simpler and crash-safe but potentially slower under high dedup query load.

- **Aggregate storage statistics.** `CasStats` and `LfsStats` in OpenXet provide dedup ratios, object counts, and size summaries. xet-backend has no equivalent endpoints or query functions.

- **Chunk/Block/Xorb data types as first-class structs.** OpenXet has `Chunk`, `Block`, `BlockChunkEntry` as rich in-memory types. xet-backend represents these only as DB rows (`ChunkRecord`) and S3 objects. There are no in-memory domain types for CAS data beyond what the shard parser provides during request processing.

### Partial

- **`ContentHash` vs. `MerkleHash`.** Both are 32-byte newtypes, but `MerkleHash` in xet-backend is purely a container with no hashing methods (`from_data`, `from_hex`, `to_hex`). Hash encoding uses the xet-core API format (little-endian u64 groups) via standalone functions `hash_to_api_string` / `api_string_to_hash`, whereas OpenXet's `ContentHash` uses plain big-endian hex. The hash algorithm also differs: OpenXet uses SHA-256; xet-backend is algorithm-agnostic at the `MerkleHash` level (the actual hashing is done client-side with Blake3 as per the xet-core protocol).

- **LFS object metadata.** `LfsObjectRow` in xet-backend stores (repo_id, oid, size, s3_key) but lacks the `status`, `raw_path`, and `chunk_hashes` fields from OpenXet's `LfsObjectMeta`. LFS objects in xet-backend are not associated with CAS chunks.

- **`CasStore` vs `AppState`.** OpenXet's `CasStore` is a rich domain object holding DashMaps, a chunker, a background worker channel, and an optional DB connection. xet-backend's `AppState` is a thin container holding `PgPool`, `S3Client`, and `AppConfig` -- the database is the source of truth, not an async write-through cache. This is a fundamental architectural difference, not a gap to close, but it changes how all CAS operations behave.

## Already covered

- **CAS protocol endpoints.** All four core xet-core CAS API endpoints are implemented: xorb upload (`POST /v1/xorbs/{prefix}/{hash}`), shard upload (`POST /v1/shards`), chunk dedup query (`GET /v1/chunks/{prefix}/{hash}`), and file reconstruction (`GET /v1/reconstructions/{file_id}`). These match the protocol surface that xet-core clients expect.

- **MDB shard binary parsing.** The `shard-parser` crate correctly parses the MDB shard format (Header + FileInfo + CASInfo sections) including optional footer, file verification entries, and FileMetadataExt (SHA-256). The writer produces valid dedup-response shards with footer.

- **Token vending.** The xet-read-token / xet-write-token API matches the auth flow expected by xet-core clients: returns `{accessToken, exp, casUrl}` with JWT bearer tokens validated by middleware on CAS routes.

- **Xorb existence verification.** The shard upload handler verifies that all referenced xorbs already exist in S3 before persisting metadata, matching the protocol requirement that xorbs must be uploaded before shards.

- **S3 presigned URL generation.** The reconstruction endpoint generates presigned GET URLs for xorbs, allowing clients to download content directly from S3.

- **Database-backed persistence.** All CAS metadata (xorbs, chunks, file_mappings) is stored in PostgreSQL with proper upsert semantics. Unlike OpenXet's fire-and-forget async DB writes, xet-backend writes are synchronous and confirmed before responding to the client.

## Out-of-scope items

- **Local-disk `StorageBackend`.** OpenXet supports `LocalStorage` with a sharded two-level directory tree for single-process development. xet-backend uses S3 exclusively; there is no local-disk storage backend. This is consistent with the project's deployment model (Docker Compose with MinIO). Adding local-disk storage is not in the current roadmap.

- **SQLite / SeaORM compatibility.** OpenXet uses SQLite with SeaORM entities and fire-and-forget persistence. xet-backend uses PostgreSQL with sqlx and synchronous writes. This is a deliberate technology choice, not a gap.

- **`#[allow(dead_code)]` surface.** OpenXet has a file-wide `#[allow(dead_code)]` annotation indicating its public API is larger than what is exercised. xet-backend's CAS crate exposes only the routes and types that are actively used.

- **Sync construction paths (`with_storage_path`).** OpenXet's dual `with_storage_path` (no DB) / `with_db` construction paths are not replicated. xet-backend always requires a database connection.

## Recommendations for synthesis

- **LFS-CAS integration decision.** The largest functional gap is the absence of a server-side LFS -> CAS chunking pipeline. If deduplication of LFS objects is a target, xet-backend would need either (a) a background worker similar to OpenXet's, or (b) client-side chunking of LFS content before upload. The current architecture treats LFS and CAS as parallel, non-intersecting storage paths. This should be an explicit decision: is LFS-CAS deduplication in scope?

- **Dedup query performance.** xet-backend's dedup queries hit PostgreSQL on every lookup. Under high-frequency dedup traffic (the xet-core client can issue thousands of dedup queries during a single upload), this may become a bottleneck. Consider whether an in-memory cache (e.g., a Bloom filter or LRU cache for known chunk hashes) is needed, or whether PostgreSQL indexing is sufficient at the expected scale.

- **Range-based reconstruction.** The `offset_into_first_range` field in reconstruction responses is hardcoded to 0 (noted in a TODO comment at `reconstructions.rs:113`). Partial-file downloads via Range headers are parsed but not acted upon. If range-based access is needed (e.g., for streaming large files), this needs implementation.

- **Shard archival key.** The shard upload handler archives raw shards to S3 using a non-keyed Blake3 hash as the key (`shards.rs:114`). This is not part of the xet-core protocol spec and serves only as an internal archival mechanism. The choice of Blake3 for this purpose is fine but should be documented as an internal convention.

- **Persistence model advantage.** xet-backend's DB-primary model (synchronous writes, no in-memory caches) is simpler and avoids the data-loss risk of OpenXet's fire-and-forget pattern. This should be recognized as a deliberate improvement, not something to change for parity.

## Open questions

- Is server-side LFS -> CAS chunking (deduplication of LFS objects) a planned feature, or is the current design (LFS and CAS as separate storage paths) the intended long-term architecture?

- The reconstruction endpoint returns chunk index ranges (`chunk_index_start`, `chunk_index_end`) but not byte ranges within the xorb. The xet-core client needs byte ranges to issue S3 Range requests. Does the client recompute byte ranges from the chunk metadata, or is this a gap in the response format?

- The `file_mappings.get_file_mapping` query matches on both `file_hash` and `sha256` (`WHERE file_hash = $1 OR sha256 = $1`). Is this dual-key lookup intentional for supporting lookups by either hash, or is it a workaround for hash format ambiguity?

- The chunks table has `ON CONFLICT (hash) DO NOTHING` for upserts, meaning if two xorbs contain the same chunk hash, only the first xorb's mapping is kept. Is this the correct dedup semantic, or should later uploads be able to update the chunk's xorb association?

- The shard upload handler does not persist a record in the `shards` DB table for the uploaded shard itself (it only stores the raw bytes in S3). The `shards` table exists in the migration but has no query functions that write to it. Is this intentional, or is the shard table meant to be populated during upload?
