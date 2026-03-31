# Round 1 Module Map — `git`

**OpenXet commit:** `a5b6dfcdf5e806a5a29cb893f0d79d733bea309b`
**Analyst:** DiTo97
**Date:** 2026-03-31
**Tracking issue:** [#40](https://github.com/Atena-IT/open-hub/issues/40)

---

## Scope

This document maps the OpenXet `git` module. It covers only the upstream OpenXet source — no `xet-backend` references appear here. Round 2 will introduce the comparison.

## Source location

```
src/git/
```

Key entry points:
- `src/git/mod.rs` — module root: re-exports from `protocol` and `storage`
- `src/git/storage.rs` — `ObjectId`, `ObjectType`, `GitObject`, `GitRef`, `Repository`, `RepositoryStore`, `TreeEntry`, `CommitInfo`
- `src/git/protocol.rs` — `GitService`, `generate_ref_advertisement`, `handle_upload_pack`, `handle_receive_pack`, `CAPABILITIES`
- `src/git/pack.rs` — `generate_pack`, `parse_pack`, `pktline` sub-module (pkt-line codec)

## Public surface

### Re-exported at `src/git/mod.rs`

| Symbol | Kind | File | Brief description |
| --- | --- | --- | --- |
| `GitService` | `enum` | `src/git/protocol.rs` | Two-variant enum: `UploadPack` (fetch/clone) and `ReceivePack` (push) |
| `generate_ref_advertisement` | `fn` | `src/git/protocol.rs` | Builds the Git Smart HTTP ref advertisement payload for `info/refs` |
| `handle_upload_pack` | `fn` | `src/git/protocol.rs` | Processes a `git-upload-pack` request body and returns a pack file response |
| `handle_receive_pack` | `fn` | `src/git/protocol.rs` | Processes a `git-receive-pack` request body, ingests the pack, and updates refs |
| `ObjectId` | `struct` | `src/git/storage.rs` | 20-byte SHA-1 object identifier with hex encode/decode |
| `ObjectType` | `enum` | `src/git/storage.rs` | Git object types: `Blob`, `Tree`, `Commit`, `Tag` |
| `Repository` | `struct` | `src/git/storage.rs` | Disk-backed single-repository instance with in-memory ref/object index |
| `RepositoryStore` | `struct` | `src/git/storage.rs` | Registry of all repositories, keyed by `"owner/repo"` string |
| `TreeEntry` | `struct` | `src/git/storage.rs` | A single git tree entry: name, oid, mode, type flags |
| `CommitInfo` | `struct` | `src/git/storage.rs` | Parsed commit metadata: tree id, parents, author, committer, message |

### Accessible through `pub mod` sub-modules (not flat-exported)

| Symbol | Kind | File | Brief description |
| --- | --- | --- | --- |
| `CAPABILITIES` | `const &[&str]` | `src/git/protocol.rs` | Advertised Git protocol capabilities: `report-status`, `delete-refs`, `side-band-64k`, `no-thin`, `agent=git-xet-server/0.1` |
| `GitObject` | `struct` | `src/git/storage.rs` | In-memory git object: `object_type` + raw `Bytes` payload |
| `GitRef` | `struct` | `src/git/storage.rs` | A git reference: name, target `ObjectId`, symbolic flag and optional symbolic target |
| `parse_repo_path` | `fn` | `src/git/storage.rs` | Splits `"owner/repo"` (with optional `.git` suffix) into `(owner, repo_name)` |
| `PackObjectType` | `enum` | `src/git/pack.rs` | Pack-format object type codes: Commit=1, Tree=2, Blob=3, Tag=4 |
| `generate_pack` | `fn` | `src/git/pack.rs` | Serialises a list of `ObjectId`s into a Git pack file (version 2, zlib-compressed) |
| `parse_pack` | `fn` | `src/git/pack.rs` | Parses a Git pack file and returns `Vec<(ObjectType, Bytes)>` |
| `pktline::encode` | `fn` | `src/git/pack.rs` | Encodes bytes as a Git pkt-line (4-hex-digit length prefix) |
| `pktline::flush` | `fn` | `src/git/pack.rs` | Returns the flush packet (`0000`) |
| `pktline::delim` | `fn` | `src/git/pack.rs` | Returns the delimiter packet (`0001`); marked `#[allow(dead_code)]` |
| `pktline::parse` | `fn` | `src/git/pack.rs` | Parses one pkt-line from a byte slice; returns `(Option<Bytes>, consumed)` |
| `pktline::parse_all` | `fn` | `src/git/pack.rs` | Iteratively parses all pkt-lines from a byte slice, skipping flush packets |

## Internal structure

### Sub-modules

| Sub-module | Purpose |
| --- | --- |
| `storage` | Git object and reference storage: `ObjectId`, `ObjectType`, `Repository`, `RepositoryStore`, commit/tree parsing, DB persistence |
| `protocol` | Git Smart HTTP protocol layer: ref advertisement, upload-pack (fetch/clone), receive-pack (push), side-band multiplexing |
| `pack` | Git pack file format: generation (`generate_pack`), parsing (`parse_pack`), pkt-line codec (`pktline`) |

### Key types

#### `ObjectId`

A 20-byte SHA-1 object identifier. Construction methods:
- `from_bytes(&[u8])` — SHA-1 hashes the input and wraps the digest (this is the standard Git hash-object path)
- `from_raw([u8; 20])` — wraps pre-computed bytes without hashing (used when reading tree entries and pack binary data)
- `from_hex(&str) -> Option<Self>` — decodes a 40-character hex string
- `to_hex() -> String` — produces the canonical 40-character lowercase hex string

Derives `Copy`, `Clone`, `PartialEq`, `Eq`, `Hash`. `Debug` prints only the first 8 hex characters; `Display` prints all 40.

#### `Repository`

Disk-backed repository instance. Fields:

| Field | Type | Role |
| --- | --- | --- |
| `name` | `String` | Canonical `"owner/repo"` identifier |
| `repo_id` | `Option<i32>` | Database primary key; `None` when the DB insert is still in flight |
| `storage` | `Arc<dyn StorageBackend>` | Pluggable storage backend (local only in sync context) |
| `object_index` | `DashMap<ObjectId, u8>` | In-memory index: object hash → type code (0–3); used for O(1) existence checks |
| `refs` | `RwLock<HashMap<String, GitRef>>` | In-memory ref map (branch names, HEAD) |
| `head` | `RwLock<String>` | Current HEAD ref target string (default `"refs/heads/main"`) |
| `storage_path` | `PathBuf` | Local filesystem root for object files |
| `db` | `Option<Arc<DatabaseConnection>>` | SeaORM connection for async persistence |

Construction paths:
- `new(name)` — in-memory only, ephemeral
- `with_storage_path(name, path)` — local disk, no DB
- `with_db(name, repo_id, path, db)` — local disk + DB persistence

Every constructor calls `initialize_empty()`, which unconditionally writes an initial empty tree and an initial commit (author/committer `Git-Xet Server <server@git-xet.local>`, timestamp 0), then inserts `refs/heads/main` and `HEAD` into the in-memory ref map.

Key methods on `Repository`:
- `store_object(GitObject) -> ObjectId` — computes SHA-1, writes object file, updates `object_index`, fires async DB insert
- `get_object(&ObjectId) -> Option<GitObject>` — reads raw object file from disk
- `has_object(&ObjectId) -> bool` — checks `object_index` first, falls back to disk existence check
- `update_ref / delete_ref / list_refs / resolve_ref` — in-memory ref operations, each fires an async DB mutation
- `parse_tree / get_commit_tree / get_commit_info` — parse standard git binary/text object formats in-memory
- `walk_commits(start, limit, skip) -> Vec<CommitInfo>` — BFS traversal of commit history
- `create_commit(tree, parents, message, author)` — builds commit object text, stores it, returns the new `ObjectId`
- `resolve_path(tree_id, path)` — walks tree entries by path components, returns `(ObjectId, is_dir)`

#### `RepositoryStore`

Registry of all repositories. Fields:

| Field | Type | Role |
| --- | --- | --- |
| `repos` | `DashMap<String, Arc<Repository>>` | In-memory map of `"owner/repo"` to repository instances |
| `storage_path` | `PathBuf` | Shared storage root passed to each `Repository` |
| `db` | `Option<Arc<DatabaseConnection>>` | Shared DB connection passed down to each `Repository` |

Key methods:
- `load_from_db() -> Result<()>` — async startup: loads all `repository`, `git_ref`, and `git_object` rows from DB to rebuild in-memory state
- `create_repo_async(owner, repo) -> Result<Arc<Repository>>` — async-context creation; inserts to DB synchronously before creating in-memory repo
- `create_repo(name) -> Result<Arc<Repository>>` — sync-context creation; inserts to DB via `tokio::spawn`, so `repo_id` is `None` initially
- `get_repo / get_or_create_repo` — lookup with optional auto-creation
- `delete_repo` — removes from map and fires async DB delete
- `list_repos / list_user_repos` — read from in-memory map only

#### `GitService`

Two-variant enum distinguishing fetch/clone (`UploadPack`) from push (`ReceivePack`). Provides helpers:
- `from_str` / `as_str` — string conversion (`"git-upload-pack"` / `"git-receive-pack"`)
- `content_type` / `result_content_type` — HTTP `Content-Type` values for the advertisement and result payloads

#### `PackObjectType`

`repr(u8)` enum mapping Git pack type codes 1–4 to `Commit`, `Tree`, `Blob`, `Tag`. Provides `From<ObjectType>` and `TryFrom<u8>` (returns `ServerError::GitProtocol` on unknown type codes). Delta types 6 (`OFS_DELTA`) and 7 (`REF_DELTA`) are explicitly not implemented.

## Intra-module dependencies

| Dependency module | What is used |
| --- | --- |
| `crate::db::entities::git_object` | SeaORM entity for persisting object hashes and type codes |
| `crate::db::entities::git_ref` | SeaORM entity for persisting ref name → target hash rows |
| `crate::db::entities::repository` | SeaORM entity for repository rows (owner_id, name, head, created_at) |
| `crate::db::entities::user` | SeaORM entity; queried during `load_from_db` and `create_repo` to map `owner_id → username` |
| `crate::error::{Result, ServerError}` | `ServerError::Internal`, `RepoNotFound`, `RepoAlreadyExists`, `InvalidPath`, `InvalidRef`, `GitProtocol`, `ObjectNotFound` |
| `crate::storage::{namespaces, StorageBackend, StorageConfig}` | `namespaces::GIT_OBJECTS` path prefix, `StorageBackend` trait, `StorageConfig` for local/S3 config |

External crate dependencies: `bytes`, `dashmap`, `flate2`, `parking_lot`, `sea_orm`, `sha1`, `tokio`.

## Notable implementation details

**SHA-1 for object IDs despite a misleading comment.** `ObjectId` is a 20-byte value computed with `sha1::Sha1`. The `compute_id` method's doc comment reads "Git uses SHA-1, but we use SHA-256 for security" yet the code calls `sha1::Sha1`. The actual hashing is standard Git SHA-1.

**Non-standard on-disk object format.** Objects are stored as a flat binary: `[type_code: u8][raw_data...]`. This is not the standard git loose-object format (which uses `zlib(header + NUL + content)`). Objects stored here cannot be read by standard git tooling, and objects produced by a git client's pack file (which are zlib-compressed) are stored decompressed. Round-tripping through a real git client would require re-encoding.

**Two-level fan-out directory layout.** Objects are stored at `{storage_path}/git_objects/{owner/repo}/{hex[:2]}/{hex[2:]}`, mirroring the standard git object directory layout, but using the custom flat binary format.

**All database writes are fire-and-forget.** Every DB mutation in `storage.rs` is wrapped in `tokio::spawn(async move { let _ = ... })` with no error propagation. Failures are silently dropped. This is the same pattern as `cas/store.rs`.

**`initialize_empty` runs on every constructor, including loads from DB.** `with_storage_path` (called from `load_from_db`) triggers `initialize_empty`, which writes a new initial commit and overwrites the in-memory refs. The subsequent code in `load_from_db` then re-populates the ref map from DB rows, so the empty commit is orphaned in the object index and on disk. Every restart of a DB-backed repo accumulates an extra orphaned initial commit.

**`create_repo` (sync) sets `repo_id = None`.** The in-memory `Repository` is created before the async DB insert completes. Any object or ref write that happens before the `tokio::spawn` finishes will attempt `self.db.insert(...)` with `repo_id = None`, causing those DB inserts to be skipped silently (the `if let (Some(db), Some(repo_id))` guard evaluates false). There is no mechanism to backfill these missed rows.

**`collect_objects_recursive` uses `Vec::contains` for deduplication.** The `exclude` and `collected` membership checks in `protocol.rs` are O(n) linear scans. For repositories with many objects, this makes `handle_upload_pack` quadratic in the number of objects returned.

**No delta compression in pack generation.** `generate_pack` writes each object as a full zlib-compressed payload with no delta base reference. This is correct and compatible with standard git clients (which must handle non-delta packs), but produces larger pack files than a delta-aware implementation.

**Side-band detection scans raw request bytes.** `handle_receive_pack` determines whether to use side-band by calling `request_body.windows(12).any(|w| w == b"side-band-64" || w == b"side-band\0")`. This searches the entire raw byte slice including the pack data itself, not just the parsed capability string. A push payload that happens to contain these bytes in object content could produce a false positive.

**Report-status is sent as a single side-band packet.** When side-band is active, the entire `report-status` payload (all `ok`/`ng` lines plus flush) is assembled into one `BytesMut` and then wrapped in a single side-band channel-1 pkt-line. The git protocol allows splitting the report across multiple pkt-lines, but this implementation sends it in one shot, which requires that the assembled report fits within the 65,520-byte pkt-line payload limit.

**`walk_commits` is BFS with post-traversal sort.** The commit graph is traversed with a `VecDeque` (BFS order). Skip and limit are applied during traversal, then the resulting set is sorted by `committer_time` descending. Because skip/limit are applied before sorting, the skip count refers to BFS discovery order, not chronological order. For non-linear histories this can produce counter-intuitive skip/limit semantics.

**`parse_tree` sorts entries with directories first, then alphabetically.** This matches the canonical git tree sort order for directory entries, but standard git's `ls-tree` sorts all entries together with a `\0` appended to directory names for comparison. The two orderings may diverge for names that collide across file/directory types.

**`RepositoryStore::get_or_create_repo` has a TOCTOU fallback without persistence.** On a concurrent creation race, if `create_repo` fails and the entry is still absent from the map, a fresh `Repository::with_storage_path` is constructed without a DB connection and inserted directly. This repo will never have its state persisted.

**`#[allow(dead_code)]` is file-scoped in `storage.rs`.** The annotation suppresses all unused-method warnings for the entire file, indicating a wider public API than what the rest of the codebase currently exercises.

## Open questions

- `ObjectId::from_bytes` hashes its input with SHA-1, but the `compute_id` comment claims SHA-256. Is there a plan to migrate to SHA-256 object IDs (as in the experimental Git SHA-256 object format), or is the comment simply incorrect?
- `initialize_empty` runs on every `with_storage_path` call (including within `load_from_db`), creating an orphaned initial commit for every loaded repository on every restart. Is cleanup of these orphaned objects planned, or is the initial commit intentionally idempotent?
- `create_repo` (sync) sets `repo_id = None`, causing all object and ref DB writes during that window to be silently skipped. Is there a mechanism to drain/flush or backfill those rows after the async DB insert completes?
- The on-disk object format (`[type_code][raw_data]`) is not compatible with the standard git loose-object or pack-object format. Can standard git tooling (`git fsck`, `git verify-pack`, etc.) be used against this storage at all, or is the storage intentionally opaque to git internals?
- Delta object types (OFS_DELTA=6, REF_DELTA=7) in incoming pack files cause `PackObjectType::try_from` to return `ServerError::GitProtocol`. A push from a client that sends a thin pack with delta objects will silently drop those objects (the error is only logged via `tracing::warn`). Is there a plan to implement delta resolution, or should the `no-thin` capability advertisement be expected to suppress deltas from all clients?
- `parse_pack` limits the loop to `data.len().saturating_sub(20)` to reserve space for the trailing SHA-1 checksum, but it does not verify that checksum. Is checksum verification intentionally omitted?
- `walk_commits` applies skip/limit in BFS discovery order, then sorts by `committer_time`. For clients that expect page-stable commit history (e.g. a web UI showing commit log page 2), the skip semantics may be surprising. Is BFS + sort the intended algorithm?
- `RepositoryStore::get_or_create_user` (called from `create_repo_async`) creates a user row with `password_hash: ""`. These implicit users bypass the normal registration flow and cannot log in. Is there a plan to reconcile implicit repo-owner users with registered users?
- The `storage_path` for S3-backed configs is set to `std::env::temp_dir().join("git-xet-cache")` in `with_storage_config`. All object reads/writes then use that temp path, which is local-only. How do object reads work when the `StorageBackend` is S3 — do they fall back to local disk, or is S3 object storage for git objects simply not implemented?
- Side-band detection searches raw request bytes rather than the parsed capability string. Is this an intentional heuristic or a known gap that should be fixed?
