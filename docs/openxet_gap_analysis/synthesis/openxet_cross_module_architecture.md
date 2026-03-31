# Synthesis — OpenXet Cross-Module Architecture

**Analyst:** DiTo97
**Date:** 2026-03-31
**Tracking issue:** [#47](https://github.com/Atena-IT/open-hub/issues/47)
**OpenXet commit:** `a5b6dfcdf5e806a5a29cb893f0d79d733bea309b`
**Input documents:**
- [`round1/openxet_api.md`](../round1/openxet_api.md)
- [`round1/openxet_git.md`](../round1/openxet_git.md)
- [`round1/openxet_cas.md`](../round1/openxet_cas.md)
- [`round1/openxet_db.md`](../round1/openxet_db.md)
- [`round1/openxet_storage.md`](../round1/openxet_storage.md)
- [`round1/openxet_web_ui.md`](../round1/openxet_web_ui.md)
- [`round1/openxet_error.md`](../round1/openxet_error.md)
- [`round1/openxet_main.md`](../round1/openxet_main.md)

---

## Purpose

This document normalizes terminology across the eight Round 1 module maps and constructs a dependency-aware architectural view of OpenXet. It explains how the eight modules compose into a single server binary, how requests flow through the layers, and how cross-cutting concerns (error propagation, persistence, auth, background work) are handled. The findings here are the primary input for Round 2 comparisons, which will evaluate later implementation and architecture differences against this normalized OpenXet baseline.

## Methodology

Each Round 1 map was read for: the module's public surface, its declared imports from other OpenXet modules, its key types, and its notable implementation details and open questions. Dependencies were extracted from the "Intra-module dependencies" sections. Terminology was normalized by cross-referencing the same concept as named in each map. Architectural layers and cross-cutting patterns were identified by grouping modules by role rather than by name. The dependency graph was assembled bottom-up: modules that import no other OpenXet modules form the foundation; `main` (which imports from all others) forms the apex.

---

## Findings

### Terminology normalization

The eight Round 1 maps use overlapping vocabulary that requires disambiguation before Round 2 comparisons can be precise.

**Content-addressed storage units.** The CAS module uses the term `Block` for a bundled collection of content-defined chunks (~64 MiB). The xet-core protocol calls the equivalent unit an "Xorb". In OpenXet, `Block` is the canonical term. The older namespace constant `CAS_CHUNKS` is deprecated in-source (replaced by `BLOCKS`), and per-chunk storage is no longer the primary storage unit. The logical file-level mapping is called `FileReconstruction` in OpenXet; the xet-core protocol calls the analogous structure a "shard". Use `Block` and `FileReconstruction` when referring to OpenXet internals.

**Hash types and namespaces.** Two distinct hash types coexist in the codebase:
- `ObjectId` — a 20-byte SHA-1 digest, used exclusively for git objects (blobs, trees, commits, tags). Represented as a 40-char hex string.
- `ContentHash` — a 32-byte SHA-256 digest, used for CAS content: chunks, blocks, LFS objects, and file reconstructions. Represented as a 64-char hex string.

The db map notes that entity comments in `cas_block`, `cas_chunk`, and `lfs_chunk` label their hash columns as "BLAKE3", but the CAS store uses `sha2::Sha256`. This is a source-level inconsistency; SHA-256 is what is actually written. The two hash namespaces are never mixed: git layer uses `ObjectId`/SHA-1, CAS and LFS layer uses `ContentHash`/SHA-256.

**Token types.** Two token kinds exist:
- Session token — a base64-encoded 32-byte random value, held only in an in-memory `RwLock<HashMap>` on `AuthManager`. Lost on server restart. Used as the value of the web UI `token=` cookie.
- Access token (`ox_*`) — a long-lived token with a persistent DB record. Stored in `access_tokens` as `SHA-256(raw_token)`. The raw `ox_*` string is returned once at creation time and never re-readable. Used in the `Authorization: Bearer ox_*` header from API clients (including the `huggingface_hub` Python library).

**LFS object lifecycle states.** The `LfsObjectStatus` enum (CAS module) defines three states: `Raw` (uploaded to disk, not yet chunked), `Processing` (background worker has picked it up), `Chunked` (split into `Block`s and indexed). The DB `lfs_objects.status` column stores this as an integer discriminant (0/1/2). The web UI uses these states to annotate tree entries as "raw / processing / chunked".

**Repo identifier conventions.** Repositories are always keyed by `"owner/repo"` as a single compound string across all in-memory maps (`RepositoryStore.repos`, `like_handlers`, `discussion_handlers`). The DB stores this split across `owner_id` (FK to `users`) and `name` columns in `repositories`, but the collaboration tables (`discussions`, `repo_likes`, `pull_requests`) store only a bare `repo_name TEXT` column whose canonical format is `"owner/repo"` by convention (not enforced by a DB constraint).

**Repo types.** The HF-compatible API accepts paths with type prefixes (`/api/models/`, `/api/datasets/`, `/api/spaces/`). These are stripped by `normalize_repo_path` in the API layer. No actual type distinction is stored or enforced at any layer; all repos are the same.

**Dual construction path naming.** Both `CasStore` and `Repository`/`RepositoryStore` offer a `with_storage_path` constructor (local, no DB) and a `with_db` constructor (local + DB persistence). The same names appear in `AppState`. When this document refers to the "production boot path" it means `with_db`; when it refers to the "no-DB path" it means `with_storage_path`, which is used in tests and single-process development.

---

### Layered architecture

OpenXet is a single monolithic binary. The eight modules form four recognizable layers, listed from outermost to innermost:

```
┌─────────────────────────────────────────────────────┐
│  Composition root: main                             │
│  (bootstraps all subsystems, assembles the router)  │
├─────────────────────┬───────────────────────────────┤
│  Request layer      │  Web/UI surface               │
│  api                │  web_ui                       │
│  (Git Smart HTTP,   │  (Tera templates, browser     │
│   HF API, LFS,      │   sessions, PR/discussion/    │
│   auth, health)     │   editor, trending, likes)    │
├─────────────────────┴───────────────────────────────┤
│  Domain layers                                      │
│  git                         cas                    │
│  (RepositoryStore,            (CasStore, Chunker,   │
│   protocol, pack)              Block, Reconstruction)│
├─────────────────────────────────────────────────────┤
│  Infrastructure layers                              │
│  storage                      db                    │
│  (StorageBackend trait,        (SeaORM entities,    │
│   LocalStorage, S3Storage)      SQLite init/DDL)    │
├─────────────────────────────────────────────────────┤
│  Cross-cutting: error                               │
│  (ServerError enum, Result alias)                   │
└─────────────────────────────────────────────────────┘
```

---

### Cross-module dependency graph

```
error ──────────────────────────────────┐
                                        │ (imported by all)
storage ─────────────────────┐          │
                             │          │
db ──────────────────────────┤          │
                             ▼          ▼
                           git ──────► api ──────► main
                           cas ──────►      ──────► web_ui
                                      └──────────► main
```

Directed edges (A imports from B):

| Consumer | Imports from | Key symbols consumed |
| --- | --- | --- |
| `git` | `db` | `git_object`, `git_ref`, `repository`, `user` entities |
| `git` | `storage` | `namespaces::GIT_OBJECTS`, `StorageBackend`, `StorageConfig` |
| `git` | `error` | `Result`, `ServerError` |
| `cas` | `db` | `lfs_object`, `lfs_chunk`, `cas_block`, `cas_chunk`, `file_segment` entities |
| `cas` | `storage` | `namespaces` (BLOCKS, CAS_CHUNKS, LFS_RAW), `StorageBackend`, `StorageConfig` |
| `cas` | `error` | `Result`, `ServerError` |
| `api` | `git` | `RepositoryStore`, `Repository`, `ObjectId`, `ObjectType`, `TreeEntry`, `GitService`, `generate_ref_advertisement`, `handle_upload_pack`, `handle_receive_pack` |
| `api` | `cas` | `CasStore`, `ContentHash`, `LfsObjectSource`, `CasStats`, `LfsStats` |
| `api` | `db` | `access_token`, `org_member`, `user` entities |
| `api` | `error` | `Result`, `ServerError` |
| `web_ui` | `api` | `AppState` (via `State<Arc<AppState>>`) |
| `web_ui` | `cas` | `ContentHash`, `LfsObjectStatus` |
| `web_ui` | `git` | `Repository`, `ObjectId`, `ObjectType`, `TreeEntry`, `CommitInfo` |
| `web_ui` | `db` | `user`, `org_member`, `discussion`, `discussion_comment`, `discussion_event`, `repo_like`, `pull_request`, `pr_comment`, `pr_event` entities |
| `main` | `api` | `AppState`, `git_router()`, `hf_router()` |
| `main` | `db` | `init_database()` |
| `main` | `web_ui` | `router()` |

The `storage` and `error` modules have no OpenXet imports. The `db` module has no OpenXet imports. These three are the dependency leaves.

Tabular form:

| Module | Depends on (OpenXet modules) | Consumed by |
| --- | --- | --- |
| `error` | — | `api`, `cas`, `git` (and transitively `web_ui`, `main`) |
| `storage` | — | `cas`, `git` |
| `db` | — | `api`, `cas`, `git`, `web_ui` |
| `git` | `db`, `storage`, `error` | `api`, `web_ui` |
| `cas` | `db`, `storage`, `error` | `api`, `web_ui` |
| `api` | `git`, `cas`, `db`, `error` | `main`, `web_ui` |
| `web_ui` | `api`, `git`, `cas`, `db` | `main` |
| `main` | `api`, `db`, `web_ui` | — (composition root) |

---

### Composition root and startup sequence

`main.rs` is the only binary entry point. Its startup sequence is:

1. Tracing initialised (`EnvFilter` defaulting to debug for the server crate and tower_http).
2. `db::init_database` opens (or creates) the SQLite file at `$GIT_XET_STORAGE_PATH/git-xet.db` and executes 18 `CREATE TABLE IF NOT EXISTS` statements plus in-place `ALTER TABLE ADD COLUMN` migrations.
3. `AppState::with_db` constructs `CasStore` (loads DB state into DashMaps), `RepositoryStore` (rebuilds in-memory repo map from DB), `AuthManager` (no DB preload; lookups are live queries), and starts the background CAS chunking worker.
4. `state.auth.ensure_admin_user_secure()` creates the `admin` user if absent.
5. Axum router assembled in a fixed precedence order: explicit `/api/*` routes → `hf_router()` (HF-compatible API) → `git_router()` (Git Smart HTTP + LFS) → `web_ui::router()`. The ordering is load-bearing: later routers use `/:owner/:repo` wildcards that would shadow earlier specific routes if reversed.
6. Security headers applied (`X-Frame-Options`, CSP, etc.) via `SetResponseHeaderLayer`.
7. TCP listener bound (`HOST:PORT`, default `127.0.0.1:8080`); `axum::serve` begins.

No TLS is configured. TLS is expected to be handled upstream.

---

### Request-facing layers: `api` and `web_ui`

Both layers access server state exclusively through `Arc<AppState>` passed via Axum's `State` extractor. `AppState` is defined in `api::handlers` and is the sole shared-state type for the entire server.

**`api` layer** handles three distinct protocol surfaces:
- Git Smart HTTP: `git_info_refs`, `git_upload_pack`, `git_receive_pack` — delegates to `git::protocol`.
- LFS Batch API: `lfs_batch`, `lfs_upload`, `lfs_download`, `lfs_verify`, `lfs_multipart_complete` — manages two-phase LFS upload lifecycle through `CasStore`.
- HF-compatible API: `hf_api::router` / `hf_api::handlers` — implements `whoami`, repo CRUD, tree listing, `preupload`, NDJSON commit, and `resolve_file` for the `huggingface_hub` Python client.

`api::auth` handles authentication across all three surfaces using a tri-dispatch bearer validation: `ox_*` prefix → DB lookup, `username:password` colon format → `authenticate()`, raw string → in-memory session map. This means the Python `huggingface_hub` client can pass `username:password` as a Bearer token without a prior login step.

**`web_ui` layer** handles browser requests via cookie-based session auth. It does not use the `Authorization` header. The session cookie (`token=`) is looked up in `AuthManager`'s in-memory session map. CSRF tokens are generated and verified per form submission using a per-process ephemeral secret (lost on restart). The layer provides the complete community surface: discussions, pull requests, repo likes, trending, branch management, web-based file editing, and access token management.

The two layers share `AppState` but do not otherwise call into each other. The `web_ui` module documentation explicitly describes a three-point removal contract (remove the module declaration, router merge, and Tera dependency) — the coupling is intentionally minimal.

---

### Git and CAS domain layers

`git` and `cas` are symmetric in structure: both maintain a large in-memory state (DashMaps plus `RwLock`-protected maps), write through to SQLite asynchronously (fire-and-forget), and share a local-disk or S3 storage backend. Neither module calls into the other.

**`git` domain.** Manages git repositories as first-class objects: object storage (blobs, trees, commits, tags in a custom binary format), reference management, and Git Smart HTTP protocol (upload-pack and receive-pack). Object IDs are SHA-1 (20 bytes). On-disk format is non-standard: a single `[type_code: u8][raw_data...]` byte sequence, not git's zlib-compressed loose-object format. Delta types (OFS_DELTA, REF_DELTA) in incoming packs are not implemented; they yield `ServerError::GitProtocol`.

**`cas` domain.** Manages content-addressed storage for LFS objects and file reconstruction. Content hashes are SHA-256 (32 bytes). Objects are chunked by a gear-hash CDC algorithm and bundled into ~64 MiB Blocks. File reconstruction is recorded as an ordered `Vec<FileSegment>`, each pointing to a byte range within a Block, enabling parallel S3 Range fetches. The "streaming" chunker API is a misnomer — it buffers all bytes in memory before chunking.

Both domains expose dual construction paths (`with_storage_path` / `with_db`) and share the pattern of constructing in-memory state first, then optionally connecting a DB. The `git` module has an additional bug in this area: `Repository::initialize_empty` runs unconditionally on every constructor including `with_storage_path` (called from `load_from_db`), producing orphaned initial commits on every restart.

---

### Infrastructure layers: `storage` and `db`

**`storage` module.** A pure trait abstraction (`StorageBackend`) with 10 async methods covering get, put, delete, range-read, list, stream, file-level I/O, size, and exists. Two implementations are provided: `LocalStorage` (sharded two-level directory tree) and `S3Storage` (AWS/MinIO/R2). The module has no OpenXet module imports — it is the lowest leaf. Key constraints: no atomic writes on `LocalStorage` (no write-then-rename), single-part S3 upload only (no multipart; 5 GiB object limit), S3 NotFound detection via string matching (fragile). The `REPO_META` namespace is declared but has no callers.

**`db` module.** Provides `init_database` (schema bootstrap) and 18 SeaORM entity types. The schema is defined entirely as inline raw SQL (18 `CREATE TABLE IF NOT EXISTS` plus accompanying indexes); there is no migration framework. In-place schema evolution uses unconditional `ALTER TABLE ADD COLUMN` calls with silent error suppression. Five entity groups serve different consumers:
- Core entities: `user`, `org_member`, `repository`, `access_token` → consumed by `api::auth`
- Git entities: `git_ref`, `git_object` → consumed by `git::storage`
- CAS entities: `cas_block`, `cas_chunk`, `file_segment`, `lfs_object`, `lfs_chunk` → consumed by `cas::store`
- Community entities: `discussion`, `discussion_comment`, `discussion_event`, `repo_like`, `pull_request`, `pr_comment`, `pr_event` → consumed by `web_ui` only (no API endpoints expose these)

Community entities reference repositories via a bare `repo_name TEXT` column with no FK constraint, creating a denormalized island isolated from the core `repositories` table.

---

### Cross-cutting: `error` module

`ServerError` is a 13-variant `thiserror`-derived enum with `IntoResponse` implemented for axum. It is the single shared error type for the entire server. Direct importers identified in the Round 1 maps are `api::auth`, `api::handlers`, `cas::store`, `git::storage`, `git::pack`, and `git::protocol`; the rest of the stack depends on it transitively through those modules.

Key behavioral properties that affect all layers:
- Internal errors (variants `Internal` and `Io`) are masked to the string `"Internal server error"` in HTTP responses; detailed messages reach only the tracing log.
- Response bodies are plain text for all variants — no JSON envelope. HF-compatible API clients that expect JSON error bodies receive plain text.
- `AuthFailed` and `PermissionDenied` both map to HTTP 403. The 401 vs 403 distinction meaningful to HuggingFace clients is only partially implemented (`AuthRequired` → 401; all auth failures → 403).
- Only `std::io::Error` has an automatic `#[from]` conversion. All other external errors (DB errors, S3 SDK errors) are manually converted to `Internal(e.to_string())` at the call site.

---

### Cross-cutting implementation patterns

The following patterns appear across multiple modules and represent architectural decisions that Round 2 comparisons should evaluate explicitly.

**Fire-and-forget DB writes.** Every DB mutation in `git::storage`, `cas::store`, and `api::auth` is wrapped in `tokio::spawn(async move { let _ = ... })`. Failures are silently discarded. There is no error propagation, no retry, and no mechanism to detect divergence between in-memory state and the DB. This pattern is consistent and intentional, but it creates a class of silent data loss on DB failure. Affected operations: ref updates, object inserts, block/chunk inserts, file segment inserts, LFS object status updates, user/org mutations.

**In-memory primary, DB as write-through index.** All read paths in `git` and `cas` go to in-memory DashMaps and `RwLock<HashMap>` structures. The DB is populated at startup (`load_from_db`) and written asynchronously thereafter. This means the DB is not the source of truth at runtime — it is a durable snapshot of in-memory state as of the last successful async write. After a crash, the DB may lag behind the last in-memory state.

**Dual construction paths throughout.** `CasStore`, `Repository`, `RepositoryStore`, and `AppState` all expose `with_storage_path` (test/dev, no DB) and `with_db` (production). The no-DB path uses the same in-memory DashMaps but skips all async persistence. Tests can run the full request stack without a database.

**Module-wide `#[allow(dead_code)]`.** Four modules (`cas`, `git::storage`, `storage`, `web_ui`) carry file-scope or module-scope `#[allow(dead_code)]` annotations. This indicates the public API surface of each module is larger than what the current codebase exercises, and that future callers are expected.

**Background CAS worker.** `CasStore::start_background_worker` consumes an `mpsc::UnboundedSender<ContentHash>`. LFS uploads are staged to raw disk immediately (low latency response) and queued for deferred chunking/deduplication. The channel is unbounded (no back-pressure) and is not shut down before `axum::serve` exits — in-flight items may be dropped on shutdown.

**No shutdown coordination.** Neither the background CAS worker nor any in-flight async DB writes have a shutdown signal or drain mechanism. Process exit may drop unflushed data.

**Hash algorithm comments do not match code.** The `db` map records that `cas_block`, `cas_chunk`, and `lfs_chunk` entity comments say "BLAKE3" but the CAS store uses SHA-256. The same pattern exists in `git::storage`, which has a comment claiming SHA-256 but uses SHA-1. These documentation errors are latent mislead risks for future maintainers.

**Admin bootstrap on every startup.** `ensure_admin_user_secure` is called unconditionally on every boot. If the admin user already exists, the call is a no-op (the DB query confirms existence); if not, the user is created. The admin password is sourced from `GIT_XET_ADMIN_PASSWORD` env var or randomly generated and printed to `tracing::warn!` (logs only, not stdout). The env var name is defined inside `api::auth`, not visible in `main.rs`.

---

### Key module interaction flows

**LFS upload flow (cross-module):**
```
client
  └─► api::lfs::lfs_batch           (signs upload URLs)
  └─► api::lfs::lfs_upload          (streams body → disk via CasStore)
       └─► cas::store::store_lfs_object_stream  (writes to LFS_RAW namespace via StorageBackend)
            └─► fire process_tx (ContentHash)
                 └─► cas::store::process_raw_object  (background: chunk, bundle into Block, persist)
                      └─► db: fire-and-forget inserts for cas_block, cas_chunk, file_segment, lfs_object
  └─► api::lfs::lfs_verify          (confirms hash; CasStore registers and queues)
```

**LFS download flow (cross-module):**
```
client
  └─► api::lfs::lfs_batch           (returns download URL)
  └─► api::lfs::lfs_download
       └─► cas::store::get_lfs_object_source
            ├─► LfsObjectSource::RawFile(path)  → tokio ReaderStream from disk
            └─► LfsObjectSource::Blocks(hash)   → ReconstructionStream
                 └─► storage::S3Storage::get_range  (up to 32 parallel)
```

**Git push flow (cross-module):**
```
client (git push)
  └─► api::handlers::git_receive_pack
       └─► git::protocol::handle_receive_pack
            └─► git::pack::parse_pack         (decode pack file objects)
            └─► git::storage::Repository::store_object  (write each object)
                 └─► storage::LocalStorage::put / StorageBackend::put
                 └─► db: fire-and-forget git_object insert
            └─► git::storage::Repository::update_ref
                 └─► db: fire-and-forget git_ref update
```

**HF-compatible commit flow (cross-module):**
```
client (huggingface_hub commit)
  └─► api::hf_api::handlers::commit_file  (NDJSON body)
       └─► parse header line → commit message
       └─► parse file line → base64 content → git blob object
            └─► git::storage::Repository::store_object
       └─► parse lfsFile line → LFS pointer text → git blob object
       └─► git::storage::Repository::create_commit  (new tree + commit)
       └─► git::storage::Repository::update_ref
```

**Web UI file browser flow (cross-module):**
```
browser GET /:owner/:repo/blob/:ref/*path
  └─► web_ui::routes::repo_handlers::blob_view
       └─► AppState::repos::get_repo
       └─► git::storage::Repository::resolve_ref
       └─► git::storage::Repository::parse_tree / resolve_path
       └─► git::storage::Repository::get_object  (read blob bytes)
       └─► web_ui::routes::lfs::check_lfs_file
            └─► cas::store::LfsObjectMeta lookup  (for LFS-pointer blobs)
       └─► templates::render("blob.html", context)
```

---

### Architectural tensions and open questions for Round 2

The following tensions were identified by synthesizing open questions and implementation notes across all eight maps. Each represents a design decision where OpenXet's current choice may differ from xet-backend's approach.

**1. Persistence model: in-memory primary vs. DB primary.**
OpenXet holds all live state in DashMaps with async DB write-through. If the DB write fails silently, the in-memory state diverges. A DB-first model (or at least synchronous confirmation before responding to the client) would be more durable but slower. Round 2 should examine which model xet-backend uses for git refs, CAS blocks, and LFS object state.

**2. DB technology: SQLite with no migration framework.**
OpenXet uses a single SQLite file with schema-as-raw-SQL and in-place `ALTER TABLE ADD COLUMN` for evolution. xet-backend uses PostgreSQL with sqlx migrations (per the project memory). The schema group, migration strategy, and entity-layer approach differ materially. Round 2 should map the schema surface-by-surface and identify which tables exist in one system but not the other.

**3. Git object storage: custom format vs. standard.**
OpenXet stores git objects as `[type_code][raw_data]` with no zlib compression, not the standard loose-object format. This makes objects incompatible with standard git tooling. Round 2 should determine whether xet-backend shares this format or uses a different approach.

**4. Error response format: plain text vs. JSON.**
All `ServerError` variants produce plain-text HTTP response bodies. HF-compatible API clients (including the Python `huggingface_hub` library) typically expect JSON error envelopes. Round 2 should check whether xet-backend wraps errors in JSON for HF-facing endpoints.

**5. Auth boundary: 401 vs. 403 semantics.**
`AuthFailed` and `PermissionDenied` both map to HTTP 403. HuggingFace clients distinguish 401 (unauthenticated) from 403 (forbidden). Round 2 should evaluate whether xet-backend returns 401 for authentication failures and whether this matters for HF Hub compatibility.

**6. Community features: DB schema exists, no API surface.**
`discussions`, `pull_requests`, `repo_likes`, `discussion_events`, `pr_events` are fully specified in the DB schema and are exercised by `web_ui` handlers, but no `api` endpoints expose them programmatically. Round 2 should determine whether xet-backend implements API-level community features or also restricts them to the web UI.

**7. LFS chunking: two-phase with background worker.**
OpenXet uses a two-phase LFS upload (raw disk immediately, background chunking deferred). The background worker uses an unbounded channel with no back-pressure or shutdown drain. Round 2 should compare this to xet-backend's LFS processing model.

**8. Tree revision handling: hardcoded `refs/heads/main`.**
Several handlers in the `api::hf_api` layer ignore the `:revision` path parameter and always resolve `refs/heads/main`. This is a known limitation that breaks multi-branch workflows. Round 2 should test whether xet-backend resolves revisions correctly.

**9. Session persistence: in-memory session tokens.**
Session tokens are lost on restart. This affects both the web UI and the Python client's `username:password` Bearer flow. Round 2 should determine whether xet-backend persists sessions or shares this limitation.

**10. S3 upload limit: single-part only.**
`StorageBackend::put_from_file` (S3 path) uses a single `PutObject`, capping uploads at 5 GiB. For LFS objects larger than 5 GiB this would fail. Round 2 should determine whether xet-backend's S3 layer uses multipart upload and whether xet-backend's LFS tests exercise large objects.

**11. Orphaned commits on restart.**
`Repository::initialize_empty` writes an initial commit on every constructor call, including calls from `load_from_db`. Every server restart of a DB-backed repository produces an orphaned commit in the object store. Round 2 should verify whether xet-backend exhibits the same behavior or guards against it.

**12. Missing referential integrity in collaboration tables.**
`discussions`, `repo_likes`, and `pull_requests` reference repositories via `repo_name TEXT` with no FK constraint. Deleting a repository does not cascade to these tables. Round 2 should check whether xet-backend has stronger referential integrity here.

---

## Inputs to the decision round

- Decide which persistence model xet-backend should target for Round 2 parity with OpenXet: whether to document the DB-primary (synchronous confirmation) vs. in-memory-primary (async write-through) difference explicitly or treat it as a gap to close.
- Decide whether the plain-text error body in OpenXet's HF API surface is a gap that xet-backend should avoid, or whether both systems should share the same behavior.
- Decide whether community features (discussions, PRs, likes) are in scope for Round 2 comparisons, given they have no OpenXet API surface (web UI only).
- Decide whether the single-part S3 upload ceiling (5 GiB) should be flagged as a Round 2 implementation gap or treated as a known shared limitation.
- Decide whether the tree-always-resolves-to-main bug in the HF API layer should be captured as a compatibility test target for Round 2.

## Open questions

- The admin password env var name (`GIT_XET_ADMIN_PASSWORD` is referenced in the api map but not in `main.rs`) — is this variable documented in a README or deployment guide not captured in these maps?
- Whether `RepositoryStore::get_or_create_user` (which creates a user with `password_hash: ""`) creates users that conflict with the `DISABLE_REGISTRATION` gate. The interaction is not visible from the maps alone.
- Whether the `REPO_META` storage namespace (`"repo-meta"`) is reserved for an upcoming feature or is permanently vestigial.
- Whether the community entity tables (`discussions`, `pull_requests`, etc.) have any test coverage, given the `#[allow(unused_imports)]` on all entity re-exports in `db/entities/mod.rs` suggests some entity types are not yet exercised.
- Whether the `CAS_CHUNKS` deprecated namespace still has data in any deployed instance, requiring a backward-compatible read path during the transition to `BLOCKS`.
