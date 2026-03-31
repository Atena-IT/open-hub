# OpenXet Storage Module Map

## Metadata
- Module: `storage`
- Source paths:
  - `../OpenXet-reference/src/storage/mod.rs`
  - `../OpenXet-reference/src/storage/backend.rs`
  - `../OpenXet-reference/src/storage/config.rs`
  - `../OpenXet-reference/src/storage/local.rs`
  - `../OpenXet-reference/src/storage/s3.rs`
- Author: Claude
- Status: review-ready

## Build shape
- `mod.rs` re-exports the public storage surface.
- `backend.rs` defines the `StorageBackend` trait, storage errors, and namespace constants.
- `config.rs` defines `StorageType` and `StorageConfig`, including local/S3/MinIO builders.
- `local.rs` provides a filesystem backend with sharded key layout and tests.
- `s3.rs` wraps the AWS SDK and translates storage operations onto bucket/prefix + range requests.

## Responsibilities
- Provide a backend-agnostic object store for Git objects, blocks, chunks, LFS raw data, and repo metadata.
- Expose the primitives required by both Git and CAS paths: full get/put, existence checks, size, listing, range reads, and streaming/file helpers.
- Encapsulate local-vs-S3 differences behind a single trait.

## Explicit non-goals
- No access control or auth decisions at the storage layer.
- No higher-level repository semantics; keys are opaque and organized only by namespace.
- No versioning, lifecycle policies, or deep storage orchestration.
- No local caching or compression policy beyond what callers already store.

## Key contracts
### Types / entities
- `StorageBackend` is the central trait with full-object and byte-range operations.
- `StorageError` separates not-found, invalid-range, IO, and generic failures.
- `StorageConfig` and `StorageType` provide runtime backend selection.

### Routes / protocol surfaces
- The module is route-agnostic, but its range-read contract is essential for reconstruction-heavy APIs and download paths.
- Namespace constants provide a shared convention for how higher layers lay out data in storage.

### Storage / persistence contracts
- Local storage uses sharded directory layout under namespace prefixes.
- S3 storage uses bucket + optional prefix + namespace/key composition.
- Both implementations expose the same object/range contract so CAS and Git can remain backend-agnostic.

## Integration points
- Upstream: used directly by `cas` and `git`.
- Downstream: relies on tokio/fs for local operations and the AWS SDK for S3/MinIO-compatible remote storage.
- Adjacent: configuration is selected during startup and fed into higher-level store constructors.

## Runtime and configuration assumptions
- Local mode assumes writable filesystem access.
- S3 mode assumes AWS-style credentials/config are available through the environment or resolved SDK chain.
- S3 range semantics use HTTP byte ranges, which are critical for reconstruction and efficient download paths.
- Initialization differs by backend: local can be built synchronously, while remote backends require async construction.

## Questions for comparison round
- How closely does OpenXet’s namespace-oriented backend abstraction line up with `xet-backend`’s existing S3/Xorb/Reconstruction storage model?
- Is OpenXet’s storage trait a useful shape to adapt in `xet-backend`, or does the existing `s3-storage` crate already provide the right seam?
- Where do the two systems differ most on range semantics, object naming, and presigned-URL ownership?
- Could OpenXet’s backend split be reused for Git-native object storage while preserving `xet-backend`’s current CAS/Xorb handling?

## Evidence
- `../OpenXet-reference/src/storage/mod.rs`
- `../OpenXet-reference/src/storage/backend.rs`
- `../OpenXet-reference/src/storage/config.rs`
- `../OpenXet-reference/src/storage/local.rs`
- `../OpenXet-reference/src/storage/s3.rs`
- `../OpenXet-reference/src/cas/store.rs`
- `../OpenXet-reference/src/git/storage.rs`
