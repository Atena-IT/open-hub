# OpenXet Git Module Map

## Metadata
- Module: `git`
- Source paths:
  - `../OpenXet-reference/src/git/mod.rs`
  - `../OpenXet-reference/src/git/pack.rs`
  - `../OpenXet-reference/src/git/protocol.rs`
  - `../OpenXet-reference/src/git/storage.rs`
- Author: Claude
- Status: review-ready

## Build shape
- `mod.rs` re-exports the public Git surface.
- `protocol.rs` implements Smart HTTP request handling, pkt-line exchange, ref advertisement, fetch, and push.
- `pack.rs` implements pack generation/parsing and pkt-line helpers.
- `storage.rs` defines `ObjectId`, `ObjectType`, `GitObject`, `GitRef`, `Repository`, and `RepositoryStore`.

## Responsibilities
- Serve a minimal Git-native repository model over Smart HTTP.
- Persist Git objects and refs through a storage backend plus optional DB metadata.
- Build and parse packfiles for fetch/push.
- Provide repo operations needed by higher layers: resolve refs, walk commits, parse trees, resolve paths, and create commits.

## Explicit non-goals
- No delta-object support in pack parsing/generation.
- No advanced Git server features such as hooks, shallow-clone semantics, conflict policies, or SSH transport.
- No full transactional validation of pushed object graphs before ref updates are accepted.
- No history-heavy product semantics beyond what the repository abstraction already exposes.

## Key contracts
### Types / entities
- `ObjectId` is a 20-byte SHA-1 wrapper with hex conversion helpers.
- `ObjectType` models blob/tree/commit/tag.
- `GitObject` computes Git-standard object IDs from `<type> <size>\0<data>`.
- `GitRef` stores ref name, target hash, and symbolic/direct-ref state.
- `Repository` is the core object/ref interface backed by storage, caches, and optional DB persistence.

### Routes / protocol surfaces
- `protocol.rs` handles ref advertisement, upload-pack, and receive-pack.
- The advertised capability surface is intentionally small and fetch/push oriented.
- Receive-pack processes create/update/delete ref commands and writes stored objects before ref updates.

### Storage / persistence contracts
- Objects are stored via `StorageBackend` under Git-oriented namespaces.
- Ref state is cached in memory and written through to SQLite/SeaORM metadata.
- `RepositoryStore` manages multiple `owner/repo` repositories.

## Integration points
- Upstream: called by the API module’s Git and HF adapter handlers.
- Downstream: depends on `storage` for blob storage, `db` for repo/ref/object metadata, and `error` for protocol/domain failures.
- Adjacent: feeds file/tree/commit operations into the Web UI and HF API layers.

## Runtime and configuration assumptions
- Requires Tokio because ref/object persistence uses async tasks and the HTTP protocol layer is async.
- Assumes a single process with in-memory caches for refs/object lookups.
- Assumes Smart HTTP is the canonical transport; there is no parallel SSH or external Git-daemon path.
- Expects repository storage and DB wiring to be initialized before requests arrive.

## Questions for comparison round
- Which Git protocol pieces could be reused or adapted directly in `xet-backend`, and which are blocked by the current lightweight repo/history model?
- How far does `xet-backend` need to move toward OpenXet’s explicit object/ref persistence before Git transport becomes viable?
- Does `xet-backend` need OpenXet-like pack handling as-is, or a richer layer that integrates with CAS/Xet chunking semantics?
- Where would transactional guarantees be required in `xet-backend` beyond OpenXet’s current last-write-wins ref updates?

## Evidence
- `../OpenXet-reference/src/git/mod.rs`
- `../OpenXet-reference/src/git/protocol.rs`
- `../OpenXet-reference/src/git/pack.rs`
- `../OpenXet-reference/src/git/storage.rs`
