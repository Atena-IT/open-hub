# Round 1 Module Map — `api`

**OpenXet commit:** `a5b6dfc`
**Analyst:** DiTo97
**Date:** 2026-03-31
**Tracking issue:** [#39](https://github.com/Atena-IT/open-hub/issues/39)

---

## Scope

This document maps the OpenXet `api` module. It covers only the upstream OpenXet source — no `xet-backend` references appear here. Round 2 will introduce the comparison.

## Source location

```
src/api/
```

Key entry points:
- `src/api/mod.rs` — module root: re-exports, `git_router()` factory
- `src/api/auth.rs` — `AuthManager`, `Token`, password/token helpers
- `src/api/handlers.rs` — `AppState`, Git Smart HTTP handlers, auth/org/token HTTP handlers
- `src/api/lfs.rs` — Git LFS Batch API handlers (upload, download, multipart, verify)
- `src/api/hf_api/mod.rs` — HuggingFace-compatible sub-module root
- `src/api/hf_api/routes.rs` — `router()` factory for HF endpoints
- `src/api/hf_api/handlers.rs` — HF endpoint implementations
- `src/api/hf_api/types.rs` — HF request/response structs

## Public surface

| Symbol | Kind | File | Brief description |
| --- | --- | --- | --- |
| `AuthManager` | `struct` | `src/api/auth.rs` | Database-backed manager for users, orgs, session tokens, and access tokens |
| `Token` | `struct` | `src/api/auth.rs` | In-memory session token carrying user identity and expiry |
| `AppState` | `struct` | `src/api/handlers.rs` | Axum shared state: repos, CAS store, auth, background-processing channel, DB |
| `git_router` | `fn` | `src/api/mod.rs` | Builds the `Router<Arc<AppState>>` for Git Smart HTTP and LFS endpoints |
| `hf_router` | `fn` (re-export of `hf_api::routes::router`) | `src/api/mod.rs` | Builds the `Router<Arc<AppState>>` for HuggingFace-compatible API endpoints |
| `health` | `fn` | `src/api/handlers.rs` | `GET /health` — returns `{status, version}` |
| `list_repos` | `fn` | `src/api/handlers.rs` | `GET /api/repos` |
| `create_repo` | `fn` | `src/api/handlers.rs` | `POST /api/repos/:owner/:repo` |
| `delete_repo` | `fn` | `src/api/handlers.rs` | `DELETE /api/repos/:owner/:repo` |
| `list_refs` | `fn` | `src/api/handlers.rs` | `GET /api/repos/:owner/:repo/refs` |
| `git_info_refs` | `fn` | `src/api/handlers.rs` | `GET /:owner/:repo/info/refs` (Git Smart HTTP) |
| `git_upload_pack` | `fn` | `src/api/handlers.rs` | `POST /:owner/:repo/git-upload-pack` (fetch/clone) |
| `git_receive_pack` | `fn` | `src/api/handlers.rs` | `POST /:owner/:repo/git-receive-pack` (push) |
| `register` | `fn` | `src/api/handlers.rs` | `POST /api/auth/register` |
| `login` | `fn` | `src/api/handlers.rs` | `POST /api/auth/login` |
| `create_org` | `fn` | `src/api/handlers.rs` | `POST /api/orgs` |
| `add_org_member` | `fn` | `src/api/handlers.rs` | `POST /api/orgs/:org/members` |
| `remove_org_member` | `fn` | `src/api/handlers.rs` | `DELETE /api/orgs/:org/members/:username` |
| `get_org_members` | `fn` | `src/api/handlers.rs` | `GET /api/orgs/:org/members` |
| `create_access_token` | `fn` | `src/api/handlers.rs` | `POST /api/tokens` |
| `list_access_tokens` | `fn` | `src/api/handlers.rs` | `GET /api/tokens` |
| `revoke_access_token` | `fn` | `src/api/handlers.rs` | `DELETE /api/tokens/:id` (soft delete) |
| `delete_access_token` | `fn` | `src/api/handlers.rs` | `DELETE /api/tokens/:id/permanent` (hard delete) |
| `cas_stats` | `fn` | `src/api/handlers.rs` | `GET /api/cas/stats` — CAS and LFS object statistics |
| `lfs_batch` | `fn` | `src/api/lfs.rs` | `POST /:owner/:repo/info/lfs/objects/batch` — LFS Batch API |
| `lfs_upload` | `fn` | `src/api/lfs.rs` | `PUT /:owner/:repo/info/lfs/objects/:oid` |
| `lfs_download` | `fn` | `src/api/lfs.rs` | `GET /:owner/:repo/info/lfs/objects/:oid` |
| `lfs_verify` | `fn` | `src/api/lfs.rs` | `POST /:owner/:repo/info/lfs/verify` (header auth) |
| `lfs_verify_signed` | `fn` | `src/api/lfs.rs` | `POST /:owner/:repo/info/lfs/verify/:oid` (signed URL auth) |
| `lfs_multipart_complete` | `fn` | `src/api/lfs.rs` | `POST /:owner/:repo/info/lfs/multipart/:oid` |

## Internal structure

### Sub-modules

| Sub-module | Purpose |
| --- | --- |
| `auth` | Password hashing (Argon2id + SHA-256 legacy), session token map, access token DB CRUD, org/user queries, permission checks |
| `handlers` | `AppState` definition; Git Smart HTTP handlers; registration/login; repo and org management; access token management; CAS stats; health check |
| `lfs` | LFS Batch API (basic + multipart transfer modes); streaming upload/download; signed URL generation and verification; background CAS queueing |
| `hf_api` | HuggingFace Hub API compatibility: whoami, repo CRUD, tree listing, preupload, NDJSON commit, file resolution |
| `hf_api::routes` | Axum `Router` assembly for all HF API endpoints |
| `hf_api::handlers` | Handler implementations for every HF API route |
| `hf_api::types` | Serde request/response structs for the HF API |

### Key types

#### `AppState`

Central Axum shared state, held behind `Arc<AppState>`.

```
repos:      RepositoryStore                        — in-memory/DB git repo registry
cas:        Arc<CasStore>                          — content-addressable object store
auth:       AuthManager                            — auth, user, org, token management
process_tx: mpsc::UnboundedSender<ContentHash>     — channel to background chunking worker
db:         Option<Arc<DatabaseConnection>>        — Sea-ORM connection (when DB is enabled)
```

Two constructors exist: `with_storage_path` (no DB; ephemeral) and `with_db` (loads repos + CAS from DB on startup).

#### `AuthManager`

```
db:                  Option<Arc<DatabaseConnection>>
tokens:              RwLock<HashMap<String, Token>>   — in-memory session token map
token_duration:      Duration                          — 24 h default, not configurable
allow_anonymous_read: bool                            — true by default, not env-configurable
```

Responsible for all authentication paths:

- `authenticate` / `authenticate_basic` / `validate_bearer` — HTTP auth dispatch
- `validate_access_token` — looks up `ox_*` tokens by SHA-256 hash in DB; updates `last_used_at`
- `check_permission` / `can_write_to_repo` — owner or org-member check
- `register_user` / `create_org` / `add_org_member` / `remove_org_member` / `get_org_members` — user/org lifecycle
- `ensure_admin_user_secure` — bootstraps admin on first start from `GIT_XET_ADMIN_PASSWORD` env or generated password

#### `Token`

```
token:      String       — raw token value (base64-encoded 32 random bytes for sessions; ox_* for access tokens)
user_id:    i32
username:   String
is_org:     bool
expires_at: SystemTime
```

Used uniformly across session tokens and access tokens after validation.

#### `CreatedAccessToken` / `AccessTokenInfo`

`CreatedAccessToken` carries the raw `ox_*` token (returned only once). `AccessTokenInfo` carries only `prefix` (`ox_XXXXXXXX`), metadata, and `is_active` — the raw token is never returned again.

#### LFS request/response types (`src/api/lfs.rs`)

| Type | Purpose |
| --- | --- |
| `BatchRequest` | `POST .../lfs/objects/batch` body; holds `operation`, optional `transfers`, and `objects` |
| `LfsObject` | `{oid: String, size: u64}` — the SHA-256 OID and declared file size |
| `BatchResponse` | `{transfer, objects[]}` returned to client |
| `ObjectResponse` | Per-object result: actions or error |
| `ObjectActions` | Optional `download`, `upload`, `verify` `ActionSpec`s |
| `ActionSpec` | `{href, header?, expires_in?}` — signed URL payload |
| `LfsUrlParams` | Query params `expires`, `sig`, `part` for signed URL verification |
| `MultipartCompleteRequest` / `MultipartPart` | Body for multipart completion: sorted list of `{partNumber, etag}` |

#### HF API types (`src/api/hf_api/types.rs`)

| Type | Purpose |
| --- | --- |
| `WhoamiResponse` | `/api/whoami` response; returns placeholder email/fullname when unauthenticated |
| `CreateRepoRequest` / `CreateRepoResponse` | Repo creation; response includes `url` field required by the Python client |
| `PreuploadRequest` / `PreuploadResponse` / `PreuploadFileResponse` | Pre-upload check; response carries `uploadMode` (`lfs` or `regular`) and `shouldIgnore: false` per file |
| `CommitResponse` | `{success, commitOid, commitUrl, hookOutput}` |
| `RepoInfoResponse` | Repo metadata with `siblings[]` (file list) and static timestamps |
| `RepoSibling` | `{rfilename, size?, blob_id?}` — flat file entry |
| `TreeEntry` | `{type, path, size?, oid}` for tree listing |
| `ValidateYamlResponse` | Always `{errors:[], warnings:[]}` — validation is not implemented |

## Intra-module dependencies

| Dependency module | What is used |
| --- | --- |
| `crate::cas` | `CasStore` (store, stats, background worker, LFS object lookup/registration, raw object path, reconstruct stream), `ContentHash`, `store::LfsObjectSource` |
| `crate::db::entities` | `access_token`, `org_member`, `user` (Sea-ORM entity models and active models) |
| `crate::error` | `Result`, `ServerError` (AuthFailed, PermissionDenied, AuthRequired, NotFound, InvalidRequest, Internal) |
| `crate::git` | `RepositoryStore`, `Repository`, `ObjectId`, `ObjectType`, `TreeEntry`, `GitService`, `generate_ref_advertisement`, `handle_upload_pack`, `handle_receive_pack` |

## Notable implementation details

### Bearer token tri-dispatch

`validate_bearer` dispatches in order: (1) `ox_*` prefix → DB access token lookup; (2) `username:password` colon format → `authenticate()`; (3) raw string → in-memory session token map. This allows the Python `huggingface_hub` client to pass `username:password` as a Bearer token without a separate login step.

### Dual token storage

Session tokens are kept only in a `RwLock<HashMap>` — they are lost on restart. Persistent `ox_*` access tokens are stored in the DB as `SHA-256(raw_token)`, never as plaintext. The SHA-256 is acceptable here because the tokens carry 256 bits of entropy; Argon2 is not needed.

### Password migration path

`verify_password` detects legacy SHA-256 hashes (64 hex chars, no `$` prefix) produced by `hash_password_legacy`, which uses the fixed salt `git-xet-server-salt:`. New passwords are hashed with Argon2id (PHC format). Constant-time comparison (`subtle::ConstantTimeEq`) is used for both paths.

### LFS URL signing

Signed URLs carry `?expires=<unix>&sig=<32-hex-chars>`. The signature is `SHA-256(LFS_URL_SECRET || oid || operation || expires_le_bytes)[..16]`. Multipart part URLs additionally include `&part=<N>` with a separate signature that covers the part number. Secret is read from `LFS_URL_SECRET` env var (≥32 chars); a random in-process secret is generated if the env var is absent (warned loudly, not persisted across restarts).

### LFS upload flow

1. `POST .../batch` — client declares files; server returns signed upload URLs.
2. `PUT .../objects/:oid` — body is streamed to disk without hashing (maximum throughput). Hash is computed after write in a `spawn_blocking` task. On mismatch, the file is deleted.
3. On success, object is registered with `CasStore::register_raw_object` and queued to the background worker via `process_tx` for chunking and deduplication.

### LFS multipart upload

Triggered when the client advertises the `multipart` transfer type and at least one object is ≥ 10 MiB. Parts are stored as `<storage>/multipart/<oid[:2]>/<oid>/part_<NNNNN>` (zero-padded to 5 digits). The completion endpoint (`POST .../multipart/:oid`) assembles parts in order using an 8 MiB buffer, verifies the full SHA-256 hash, registers the object, queues it for processing, and cleans up part files. ETags returned per part are dummy values (`"part<N>"`).

### LFS download dual path

`lfs_download` checks `CasStore::get_lfs_object_source`:
- `LfsObjectSource::RawFile(path)` — the object was uploaded but not yet deduplicated; served via `tokio_util::ReaderStream`.
- `LfsObjectSource::Blocks(file_hash)` — deduplicated; served via `CasStore::reconstruct_file_stream` with up to 32 parallel fetch streams (claimed ~5 GB/s).

### NDJSON commit format

`POST .../commit/:revision` reads a line-delimited JSON body. Each line is `{"key": "<type>", "value": {...}}`:
- `"header"` — extracts `value.summary` as the commit message.
- `"file"` — decodes `value.content` (base64 or raw) and adds the blob to the git tree.
- `"lfsFile"` — writes a Git LFS pointer text (`version … oid sha256:… size …`) into the tree; actual LFS content must already be uploaded via the Batch API.

### Repo type unification

All of `/api/models/`, `/api/datasets/`, `/api/spaces/` resolve to the same storage namespace (`owner/repo`). The repo type segment is stripped by `normalize_repo_path`. No actual distinction exists at the storage or permission level.

### Tree hardcoded to `refs/heads/main`

`collect_repo_files` (used by `repo_info`) always resolves `refs/heads/main`. The `tree_list` handler also resolves `refs/heads/main` even though a `:revision` path parameter is present. Only `tree_list_path` and `repo_info_revision` attempt to resolve the passed revision (as SHA or branch name).

### Static timestamps in repo info

`RepoInfoResponse.created_at` and `last_modified` are hardcoded to `"2024-01-01T00:00:00.000Z"`. No real timestamps are tracked or returned.

### Admin bootstrap

`ensure_admin_user_secure` creates an `admin` user on startup if one does not exist. Password source priority: `GIT_XET_ADMIN_PASSWORD` env var (≥12 chars required) → randomly generated 24-char password (printed to logs). Registration for regular users can be disabled via `DISABLE_REGISTRATION=1`.

### Git Smart HTTP LFS route duplication

`git_router()` registers LFS routes four times: once without a type prefix, and once each for `/datasets/`, `/models/`, `/spaces/`. This mirrors HuggingFace's typed URL scheme without any actual routing logic difference.

## Open questions

- Where are `git_router()` and `hf_router()` actually mounted in `main.rs`? The exact prefix (if any) is not visible from within this module and affects the full URL space.
- `AuthManager.allow_anonymous_read` is hardcoded to `true` with no env var or config hook — is public read intentional and permanent, or is configuration planned?
- `tree_list` ignores the `:revision` path parameter and always resolves `refs/heads/main`. Is this a known limitation or a bug?
- `repo_info` and `repo_info_revision` return hardcoded `created_at`/`last_modified` dates — will real timestamps come from DB metadata or git commits?
- `validate_yaml` is a no-op stub — is implementation planned, or is it permanently a pass-through to satisfy client protocol requirements?
- `preupload` ignores both the `_repo` and `_revision` path segments; permission is checked only against the owner. Is cross-repo preupload isolation intentional?
- The `lfsFile` commit entry writes a pointer into the git tree. Does `resolve_file` then serve that pointer text verbatim, or does it transparently redirect to LFS content? (The current implementation reads blob content directly, so pointer text would be served as-is to clients that call `resolve_file`.)
- `hash_password_legacy` uses the salt prefix `git-xet-server-salt:` — what was the prior system this is migrating from, and is there a migration timeline?
- The in-memory session token map means all sessions are lost on server restart. Is this an accepted trade-off, or is persistent session storage planned?
- `OrganizationMembership` role values (`"owner"`, `"member"`) are free-form strings — is there a defined set of roles and any enforcement of role-based capabilities (e.g. only owners can add members)?
