# OpenXet CAS Module Map

## Metadata
- Module: `cas`
- Source paths:
  - `../OpenXet-reference/src/cas/mod.rs`
  - `../OpenXet-reference/src/cas/store.rs`
- Author: Claude
- Status: review-ready

## Build shape
- `mod.rs` is a thin re-export layer.
- `store.rs` is the substantive module and contains chunking, block bundling, reconstruction metadata, LFS object lifecycle handling, background processing, and DB loading/persistence.

## Responsibilities
- Own the Xet/CAS-style content-addressed storage flow for large file data.
- Chunk uploaded content, deduplicate chunks, bundle them into larger blocks, and persist reconstruction metadata.
- Track LFS object lifecycle state and bridge raw uploads into chunked/block-backed storage.
- Provide multiple reconstruction strategies including buffered, parallel, and streaming reads.

## Explicit non-goals
- No Git protocol or ref/history semantics; it is a storage subsystem, not a Git server by itself.
- No product-layer auth/token negotiation; access control belongs to higher layers.
- No rich migration/repair/scrubbing workflows over stored CAS data.
- No full delta-compression or history-oriented artifact model; the focus is chunk/block reconstruction efficiency.

## Key contracts
### Types / entities
- `ContentHash` is the core CAS identifier.
- `Chunk`, `Block`, `FileSegment`, and `FileReconstruction` model chunk storage, block bundling, and file rebuild metadata.
- LFS objects move through explicit states such as raw, processing, and chunked.

### Routes / protocol surfaces
- The CAS module itself is not route-heavy, but it powers upload/download/LFS flows exposed by the API layer.
- Background-worker integration is part of the module contract: upload handlers can enqueue raw objects for later chunking.

### Storage / persistence contracts
- Depends on `StorageBackend` for local or S3-backed object persistence.
- Persists chunk, block, LFS, and reconstruction metadata into the DB.
- Uses namespace-oriented storage layout instead of embedding file semantics directly into paths.

## Integration points
- Upstream: the API/LFS layer queues raw uploads and queries stats/reconstruction behavior.
- Downstream: depends on `storage` for object persistence and `db` for metadata persistence.
- Adjacent: can coexist with the Git module, but remains conceptually separate from Git object/ref semantics.

## Runtime and configuration assumptions
- Requires Tokio for async upload, reconstruction, and worker processing.
- Assumes storage and optional DB are initialized before `CasStore` is used.
- Relies on configured chunking/block-size constants that bias the implementation toward storage efficiency.
- Assumes a single-process background worker model rather than distributed processing.

## Questions for comparison round
- How close are OpenXet’s chunk/block/reconstruction abstractions to `xet-backend`’s Xorb/Shard/Reconstruction model?
- Which CAS pieces look preserve-and-evolve compatible with `xet-backend`, and where are the format/protocol mismatches too deep to reuse directly?
- Should `xet-backend` adopt any of OpenXet’s background-processing or reconstruction patterns, or does its current CAS pipeline already occupy that role?
- What evidence in Round 2 will decide whether `cas` remains a preserve-and-evolve area versus a convergence target?

## Evidence
- `../OpenXet-reference/src/cas/mod.rs`
- `../OpenXet-reference/src/cas/store.rs`
- `../OpenXet-reference/src/storage/backend.rs`
- `../OpenXet-reference/src/api/handlers.rs`
