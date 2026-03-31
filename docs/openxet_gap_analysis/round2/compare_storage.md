# Storage Comparison

## Metadata
- Module: `storage`
- OpenXet map: `round1/openxet_storage.md`
- Local anchors:
  - `crates/server/src/main.rs`
  - `crates/hub-api/src/routes/files.rs`
  - `crates/hub-api/src/routes/lfs.rs`
  - `crates/hub-api/src/routes/xet_auth.rs`
  - `crates/s3-storage/src/lib.rs`
  - `crates/cas-server/src/routes/reconstructions.rs`
- Author: Claude
- Status: review-ready

## What OpenXet does
- Defines a backend-agnostic storage contract used by both Git and CAS.
- Treats namespaces and byte-range reads as foundational storage capabilities.
- Supports local and S3-compatible backends behind the same trait.

## What xet-backend currently does
- Uses a concrete `S3Client` abstraction with internal/public endpoints, presigned URLs, and typed key builders.
- Already has strong S3 integration for Xorbs, shards, repo files, and LFS objects.
- Handles reconstruction largely through metadata + presigned URL generation rather than a generic pluggable storage trait.
- Is more storage-specific than OpenXet, but also more mature in the Xet/CAS path.

## Gap inventory
| Area | OpenXet behavior | Current behavior | Gap type | Notes |
| --- | --- | --- | --- | --- |
| Backend abstraction | Local/S3 behind one trait | Concrete S3 client only | storage | Useful future seam |
| Range semantics | First-class storage capability | Partial/incomplete range handling in reconstruction path | storage, protocol | Important for future Git/CAS reads |
| Namespace strategy | Generic namespace contracts | Typed key-builder helpers | storage | Different style, similar outcome |
| Presign ownership | Mostly lower-layer capability | Strongly exposed through current route flows | storage, routing | Fits current architecture |
| Git object storage readiness | Storage seam already generic | Would need a new Git-object use of the current S3 layer | storage, persistence | Not a blocker if an abstraction is added |

Gap type uses one or more of: `protocol`, `persistence`, `storage`, `routing`, `auth`, `ui`.

## Recommendation
- Decision: **extend ours**
- Why this is the best next move:
  - `xet-backend`’s S3/presign integration is already a strength and should not be replaced.
  - The main improvement suggested by OpenXet is adding a cleaner backend/storage seam and completing range semantics, not adopting a different storage stack.
  - A trait-shaped wrapper around the current storage layer could make future Git-object support cleaner without disrupting existing CAS flows.
- Dependencies or blockers:
  - CAS decisions in `compare_cas.md`
  - Git foundation decisions in `compare_git.md`

## Verification to design later
- Range-read tests over reconstruction/download paths
- Storage-seam tests if a backend trait is introduced
- Mixed Git/CAS/LFS read-path tests once Git-native object storage is added

## Evidence
- OpenXet anchors: `round1/openxet_storage.md`, `round1/openxet_cas.md`
- xet-backend anchors:
  - `crates/s3-storage/src/lib.rs`
  - `crates/cas-server/src/routes/reconstructions.rs`
  - `crates/hub-api/src/routes/files.rs`
  - `crates/hub-api/src/routes/lfs.rs`
  - `crates/hub-api/src/routes/xet_auth.rs`
  - `crates/server/src/main.rs`
