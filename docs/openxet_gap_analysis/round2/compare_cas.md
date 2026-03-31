# CAS Comparison

## Metadata
- Module: `cas`
- OpenXet map: `round1/openxet_cas.md`
- Local anchors:
  - `crates/cas-server/src/routes/reconstructions.rs`
  - `crates/cas-server/src/routes/xorbs.rs`
  - `crates/cas-server/src/routes/shards.rs`
  - `crates/db-layer/migrations/001_initial.sql`
  - `crates/s3-storage/src/lib.rs`
- Author: Claude
- Status: review-ready

## What OpenXet does
- Provides a generalized CAS/LFS subsystem with chunking, block bundling, reconstruction metadata, and background processing.
- Uses a separate CAS foundation that coexists with, but is distinct from, the Git layer.
- Relies on a storage trait and DB-backed metadata for chunk/block/file reconstruction.

## What xet-backend currently does
- Implements a more Xet-specific CAS path around Xorbs, Shards, chunks, and reconstruction terms.
- Exposes explicit CAS protocol routes for Xorb upload, Shard ingestion, chunk lookup, and reconstruction lookup.
- Uses Postgres + S3 + typed formats rather than OpenXet’s chunk/block abstraction.
- Already has stronger domain-specific behavior for Xet/CAS flows than OpenXet’s more generic CAS layer.

## Gap inventory
| Area | OpenXet behavior | Current behavior | Gap type | Notes |
| --- | --- | --- | --- | --- |
| CAS abstraction | Chunk/block/reconstruction subsystem | Xorb/Shard/Reconstruction subsystem | storage, persistence | Different abstractions for similar goals |
| Background processing | Raw-to-chunked lifecycle with worker | Direct protocol ingestion of pre-shaped artifacts | storage | OpenXet has richer ingest lifecycle |
| LFS object lifecycle | Explicit state transitions | LFS support exists but is lighter | storage, auth | More of a feature-shape difference than a core blocker |
| Reconstruction path | Multiple reconstruction strategies | Metadata + presigned URL reconstruction flow | storage, protocol | xet-backend is already strong here |
| Git coupling | CAS coexists with Git-native repo model | CAS is stronger than Git layer today | persistence | Important for later integration sequencing |

Gap type uses one or more of: `protocol`, `persistence`, `storage`, `routing`, `auth`, `ui`.

## Recommendation
- Decision: **keep current**
- Why this is the best next move:
  - `xet-backend`’s CAS/Xorb/Shard model is already the stronger, more domain-specific implementation for the current product goal.
  - OpenXet’s CAS is useful as a reference for lifecycle ideas and storage seams, but not as a replacement target.
  - The major work in this epic lies in bringing the repo/history/Git transport model up to the CAS layer, not replacing CAS itself.
- Dependencies or blockers:
  - Need future mixed Git + CAS integration decisions once the Git foundation is chosen
  - Range/read-path cleanup from `compare_storage.md` still matters

## Verification to design later
- Mixed Git + HF + Xet roundtrip tests
- Reconstruction/range-read tests
- Large-file and LFS interoperability tests once Git-native transport exists

## Evidence
- OpenXet anchors: `round1/openxet_cas.md`, `round1/openxet_storage.md`
- xet-backend anchors:
  - `crates/cas-server/src/routes/reconstructions.rs`
  - `crates/cas-server/src/routes/xorbs.rs`
  - `crates/cas-server/src/routes/shards.rs`
  - `crates/db-layer/migrations/001_initial.sql`
  - `crates/s3-storage/src/lib.rs`
