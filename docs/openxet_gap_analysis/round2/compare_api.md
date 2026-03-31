# Round 2 Comparison — `api`

**OpenXet commit:** `a5b6dfcdf5e806a5a29cb893f0d79d733bea309b`
**Analyst:** DiTo97
**Date:** 2026-03-31
**Tracking issue:** [#48](https://github.com/Atena-IT/open-hub/issues/48)
**Round 1 source:** [`round1/openxet_api.md`](../round1/openxet_api.md)

---

## Scope

This document compares the OpenXet `api` module (mapped in Round 1) against the corresponding implementation in `xet-backend`. The OpenXet `api` module covers five protocol surfaces: Git Smart HTTP, LFS Batch API, HuggingFace-compatible API, authentication/authorization, and health. The comparison evaluates each surface for parity, partial coverage, and gaps.

## xet-backend counterpart

xet-backend splits what OpenXet consolidates in a single `src/api/` module across three crates and an error crate:

| Responsibility | xet-backend location | Notes |
| --- | --- | --- |
| HF-compatible API (whoami, repo CRUD, tree, preupload, commit, resolve) | `crates/hub-api/src/routes/` (whoami.rs, repos.rs, files.rs) | Separate crate; equivalent to OpenXet `hf_api` sub-module |
| Authentication & authorization | `crates/hub-api/src/auth.rs` | Covers ox_* token resolution, password hashing, repo access checks |
| LFS Batch API (batch, upload, download, verify) | `crates/hub-api/src/routes/lfs.rs` | Same crate as HF API; no multipart support |
| Access token management | `crates/hub-api/src/routes/tokens.rs` | Create, list, delete |
| Xet CAS token vending (read/write tokens) | `crates/hub-api/src/routes/xet_auth.rs` + `crates/cas-server/src/routes/auth.rs` | JWT-based; both Hub API and CAS server expose token endpoints |
| CAS protocol routes (xorbs, shards, chunks, reconstructions) | `crates/cas-server/src/routes/` | Separate crate; JWT middleware on protected routes |
| Shared application state | `crates/hub-api/src/state.rs` (HubState) + `crates/cas-server/src/state.rs` (AppState) | Both hold `Arc<PgPool>`, `Arc<S3Client>`, `Arc<AppConfig>` |
| Error type / HTTP error responses | `crates/common/src/error.rs` (AppError) | JSON error bodies with X-Error-Code header |
| Unified server composition | `crates/server/src/main.rs` | Merges CAS, Hub API, LFS, Web UI routers |
| Git Smart HTTP protocol | **not present** | No git protocol handlers exist in xet-backend |
| Organization management | **not present** | No org CRUD, no org membership |
| CAS stats endpoint | **not present** | No `/api/cas/stats` equivalent |
| Health check | `crates/cas-server/src/routes/health.rs` (`/health`) | CAS-level only; no Hub API health endpoint |
| Web UI (browser sessions) | `crates/web-ui/` | Separate crate; Tera templates; login/signup/repo browse |

## Surface comparison

Status values: `covered` | `partial` | `missing` | `out-of-scope`

### Application state

| OpenXet symbol / responsibility | Status | xet-backend equivalent | Notes |
| --- | --- | --- | --- |
| `AppState` (unified shared state) | `partial` | `HubState` + CAS `AppState` | xet-backend splits state into two structs; both hold pool/S3/config but lack `RepositoryStore`, background `process_tx`, and in-memory git/CAS caches |
| `AppState::repos` (in-memory repo registry) | `missing` | — | xet-backend uses DB queries exclusively; no in-memory repo map |
| `AppState::cas` (CAS store with DashMaps) | `missing` | — | xet-backend delegates to S3/DB directly; no in-memory CAS index |
| `AppState::process_tx` (background worker channel) | `missing` | — | No background chunking pipeline exists |
| `AppState::db` (optional DB connection) | `covered` | `HubState.pool: Arc<PgPool>` | Always present; not optional |
| Dual construction paths (`with_storage_path` / `with_db`) | `out-of-scope` | — | xet-backend always requires a DB; no in-memory-only mode |

### Authentication

| OpenXet symbol / responsibility | Status | xet-backend equivalent | Notes |
| --- | --- | --- | --- |
| `AuthManager` | `partial` | `crates/hub-api/src/auth.rs` (free functions) | No struct; functions operate directly on PgPool |
| `Token` (session token struct) | `missing` | — | No in-memory session token map; no session tokens |
| `authenticate` / `authenticate_basic` | `missing` | — | No Basic auth dispatch; no username:password auth |
| `validate_bearer` (tri-dispatch: ox_* / user:pass / session) | `partial` | `resolve_bearer_token` | Only ox_* token path implemented; no user:pass or session fallback |
| `validate_access_token` (DB lookup by SHA-256 hash) | `covered` | `resolve_bearer_token` in `auth.rs:49-64` | Same approach: SHA-256 hash lookup, touch last_used |
| `check_permission` / `can_write_to_repo` | `partial` | `ensure_repo_write_access` / `ensure_repo_read_access` | Owner-only write check; no org membership check |
| `register_user` | `covered` | `auth_routes::register` | Username validation, Argon2id hash, auto-creates ox_* token |
| `create_org` / `add_org_member` / `remove_org_member` / `get_org_members` | `missing` | — | No organization support at all |
| `ensure_admin_user_secure` (admin bootstrap) | `missing` | — | No admin bootstrap on startup |
| Password hashing (Argon2id) | `covered` | `auth::hash_password` / `auth::verify_password` | Same Argon2id approach |
| SHA-256 legacy password migration | `missing` | — | No legacy hash detection or migration path |
| `allow_anonymous_read` flag | `partial` | `ensure_repo_read_access` | Anonymous read allowed for public repos; private repos require auth. Behavior is implicit via `repo.private` check, not a configurable flag |

### Registration & login

| OpenXet symbol / responsibility | Status | xet-backend equivalent | Notes |
| --- | --- | --- | --- |
| `register` (`POST /api/auth/register`) | `covered` | `auth_routes::register` | Returns ox_* token (not session token); auto-creates "default" token |
| `login` (`POST /api/auth/login`) | `covered` | `auth_routes::login` | Returns new ox_* token per login (creates "login" token in DB) |

### Health & stats

| OpenXet symbol / responsibility | Status | xet-backend equivalent | Notes |
| --- | --- | --- | --- |
| `health` (`GET /health`) | `covered` | `cas-server/routes/health.rs` | Present on CAS router; not on Hub API router |
| `cas_stats` (`GET /api/cas/stats`) | `missing` | — | No CAS or LFS statistics endpoint |

### Repo management

| OpenXet symbol / responsibility | Status | xet-backend equivalent | Notes |
| --- | --- | --- | --- |
| `create_repo` (`POST /api/repos/:owner/:repo`) | `covered` | `repos::create_repo` (`POST /api/repos/create`) | Different URL pattern; xet-backend uses request body for name, not path params |
| `delete_repo` (`DELETE /api/repos/:owner/:repo`) | `covered` | `repos::delete_repo` (`DELETE /api/repos/delete`) | Different URL pattern; uses request body for name |
| `list_repos` (`GET /api/repos`) | `partial` | `repos::list_models` / `repos::list_datasets` | Separate endpoints per type; no unified listing |
| `list_refs` (`GET /api/repos/:owner/:repo/refs`) | `covered` | `repos::list_repo_refs` | Returns branches and tags from DB; includes `main` from head_sha |
| `validate_yaml` (`POST /api/validate-yaml`) | `covered` | `repos::validate_yaml` | xet-backend actually parses YAML frontmatter; OpenXet is a no-op stub |
| Repo info by revision | `covered` | `repos::repo_info_revision` | Resolves revision via DB query |
| `update_repo_settings` | `partial` | `repos::update_repo_settings` | Route exists but visibility update is a stub (returns success without DB write) |
| Branch CRUD | `covered` | `repos::create_branch` / `repos::delete_branch` | Creates refs in DB; prevents deleting main |
| Tag CRUD | `covered` | `repos::create_tag` / `repos::delete_tag` | Creates refs in DB |
| Repo type unification (models/datasets/spaces) | `covered` | Route triplication in `routes/mod.rs` | Same pattern: three type prefixes map to same handlers |

### HF-compatible API

| OpenXet symbol / responsibility | Status | xet-backend equivalent | Notes |
| --- | --- | --- | --- |
| `hf_router` (sub-router factory) | `covered` | `hub_api_router()` in `routes/mod.rs` | Part of main hub router, not a separate sub-router |
| `whoami` (`GET /api/whoami`) | `covered` | `whoami::whoami` | Returns user info; also registered at `/api/whoami-v2` |
| `repo_info` | `covered` | `repos::repo_info` | Includes siblings with LFS metadata; uses real `created_at` from DB |
| `tree_list` | `covered` | `files::tree_list` | Resolves revision parameter; builds directory/file entries from DB |
| `preupload` | `covered` | `files::preupload` | Size threshold 10 MiB for LFS; validates repo write access |
| `commit` (NDJSON) | `covered` | `files::commit` | Handles header, file, lfsFile, deletedFile, deletedFolder operations |
| `resolve_file` (download) | `covered` | `files::resolve_file` | Proxies content for regular files; presigns S3 redirect for LFS; returns Xet headers |
| `WhoamiResponse` | `partial` | `whoami::WhoamiResponse` | No org membership in response; `orgs` always empty |
| `CreateRepoRequest` / `CreateRepoResponse` | `covered` | `repos::CreateRepoRequest` / `RepoInfoResponse` | Response includes `url` field required by Python client |
| `PreuploadRequest` / `PreuploadResponse` | `covered` | `files::PreuploadRequest` / `PreuploadResponse` | Same structure; `shouldIgnore: false` per file |
| `CommitResponse` | `partial` | `files::CommitResponse` | Missing `success` and `hookOutput` fields that OpenXet returns |
| `RepoInfoResponse` | `partial` | `repos::RepoInfoResponse` | Missing `last_modified` field; includes real `created_at` (improvement over OpenXet's hardcoded date) |
| `TreeEntry` | `covered` | `files::TreeEntry` | Includes LFS pointer info |
| `ValidateYamlResponse` | `covered` | `repos::ValidateYamlResponse` | Functionally superior: actually validates YAML |
| Route without type prefix (`/{owner}/{repo}/resolve/...`) | `covered` | `routes/mod.rs` | Registered for resolve and LFS endpoints |

### Git Smart HTTP

| OpenXet symbol / responsibility | Status | xet-backend equivalent | Notes |
| --- | --- | --- | --- |
| `git_router` (router factory) | `missing` | — | No git protocol router |
| `git_info_refs` (`GET /:owner/:repo/info/refs`) | `missing` | — | No Smart HTTP advertisement |
| `git_upload_pack` (`POST /:owner/:repo/git-upload-pack`) | `missing` | — | No clone/fetch support |
| `git_receive_pack` (`POST /:owner/:repo/git-receive-pack`) | `missing` | — | No push support |

### LFS Batch API

| OpenXet symbol / responsibility | Status | xet-backend equivalent | Notes |
| --- | --- | --- | --- |
| `lfs_batch` (`POST .../lfs/objects/batch`) | `covered` | `lfs::lfs_batch` | Supports basic and xet transfer modes; uses presigned S3 URLs |
| `lfs_upload` (`PUT .../objects/:oid`) | `covered` | `lfs::lfs_upload` | Direct upload to S3; records in DB |
| `lfs_download` (`GET .../objects/:oid`) | `covered` | `lfs::lfs_download` | Redirects to presigned S3 URL |
| `lfs_verify` (`POST .../lfs/verify`) | `covered` | `lfs::lfs_verify` | Checks S3 object existence; updates DB record |
| `lfs_verify_signed` (`POST .../lfs/verify/:oid`) | `missing` | — | No signed-URL verify variant |
| `lfs_multipart_complete` (`POST .../lfs/multipart/:oid`) | `missing` | — | No multipart upload support |
| LFS URL signing (SHA-256 HMAC) | `missing` | — | No custom URL signing; relies on S3 presigned URLs |
| LFS `BatchRequest` / `BatchResponse` types | `covered` | `lfs::LfsBatchRequest` / `LfsBatchResponse` | Same structure |
| LFS multipart types | `missing` | — | No `MultipartCompleteRequest` / `MultipartPart` |
| Background CAS chunking after upload | `missing` | — | No background dedup worker; LFS objects stored as-is in S3 |
| Dual download path (raw file vs. reconstructed blocks) | `missing` | — | Only single S3 presigned redirect; no reconstruction stream |

### Access token management

| OpenXet symbol / responsibility | Status | xet-backend equivalent | Notes |
| --- | --- | --- | --- |
| `create_access_token` (`POST /api/tokens`) | `covered` | `tokens::create_token` | Same pattern: generate ox_*, store SHA-256 hash |
| `list_access_tokens` (`GET /api/tokens`) | `covered` | `tokens::list_tokens` | Returns metadata only; no raw token |
| `revoke_access_token` (`DELETE /api/tokens/:id`) | `missing` | — | No soft delete / revocation; only hard delete |
| `delete_access_token` (`DELETE /api/tokens/:id/permanent`) | `covered` | `tokens::delete_token` (`DELETE /api/tokens/:token_id`) | Hard delete only |
| `CreatedAccessToken` / `AccessTokenInfo` types | `partial` | `tokens::TokenResponse` | Single type for both create (with token) and list (without token) |

### Xet CAS token vending

| OpenXet symbol / responsibility | Status | xet-backend equivalent | Notes |
| --- | --- | --- | --- |
| Xet read/write token endpoints | `covered` | `xet_auth.rs` (Hub API) + `cas-server/routes/auth.rs` | Both crates implement token vending; Hub API version adds X-Xet-* response headers |
| JWT-based CAS auth | `covered` | `cas-server/middleware.rs` | HS256 JWT with Claims struct (sub, scope, revision, exp) |

### Organization management

| OpenXet symbol / responsibility | Status | xet-backend equivalent | Notes |
| --- | --- | --- | --- |
| `create_org` (`POST /api/orgs`) | `missing` | — | No org support |
| `add_org_member` (`POST /api/orgs/:org/members`) | `missing` | — | — |
| `remove_org_member` (`DELETE /api/orgs/:org/members/:username`) | `missing` | — | — |
| `get_org_members` (`GET /api/orgs/:org/members`) | `missing` | — | — |

## Gaps

### Missing

- **Git Smart HTTP protocol.** xet-backend has no `git_info_refs`, `git_upload_pack`, or `git_receive_pack` handlers. Standard `git clone` / `git push` over HTTP is not supported. All content flows through the HF API (NDJSON commit) and LFS/Xet protocols.

- **Organization management.** No org creation, membership management, or org-scoped permissions. Write access is owner-only; there is no mechanism for collaborative repos.

- **In-memory session tokens.** xet-backend has no session token concept. Every login and registration creates a persistent ox_* access token in the DB. There is no ephemeral session flow for the web UI (the web UI relies on cookie-based sessions handled separately in `web-ui/pages.rs`).

- **Bearer token tri-dispatch.** Only `ox_*` tokens are accepted. The `username:password` colon format that OpenXet supports for `huggingface_hub` Bearer tokens is not implemented. If a non-ox_* token is presented, it is rejected immediately.

- **LFS multipart upload.** No multipart completion endpoint. Large LFS objects (>5 GiB) that require multipart transfer cannot be uploaded through the LFS API.

- **LFS URL signing.** xet-backend uses S3 presigned URLs directly instead of custom HMAC-signed URLs. This means the server does not intermediate upload/download authorization after the batch response; the S3 credential scope is the authorization boundary.

- **Background CAS chunking worker.** No `process_tx` channel or background dedup pipeline. LFS objects remain as opaque S3 blobs; they are not chunked into Blocks or indexed for file reconstruction via the CAS layer.

- **CAS stats endpoint.** No `/api/cas/stats` equivalent. There is no way to query CAS or LFS storage statistics through the API.

- **Admin bootstrap.** No `ensure_admin_user_secure` mechanism. There is no automatic admin user creation on startup.

- **Token soft revocation.** Only hard delete of access tokens is supported. No `revoke_access_token` for soft-disable.

- **Legacy password migration.** No SHA-256 legacy hash detection or migration path. Only Argon2id is supported.

### Partial

- **AppState structure.** xet-backend splits state into `HubState` and CAS `AppState`, both DB-backed. The OpenXet pattern of in-memory DashMaps for repos, CAS objects, and session tokens is absent. xet-backend is DB-primary for all reads.

- **Bearer token resolution.** `resolve_bearer_token` only handles `ox_*` prefix tokens. OpenXet's tri-dispatch (ox_* -> user:pass -> session map) is reduced to a single path, so xet-backend does not currently match the broader bearer-token surface that OpenXet exposes. The practical downstream impact of the missing `username:password` shortcut is not verified here.

- **CommitResponse.** Missing `success` boolean and `hookOutput` string fields that OpenXet returns. Downstream dependence on those fields is not verified here.

- **WhoamiResponse.** The `orgs` field is always an empty array. OpenXet populates this from the org membership database.

- **RepoInfoResponse.** Missing `last_modified` field. Uses real DB `created_at` (an improvement), but also falls back to `"main"` when `head_sha` is None for the `sha` field, which is non-standard.

- **update_repo_settings.** The route exists and returns a 200 response, but the visibility change is not persisted to the database (the DB write is commented out as a TODO).

- **Repo listing.** Separate `list_models` and `list_datasets` endpoints instead of a unified `list_repos`. No `list_spaces` endpoint.

## Already covered

- **Repo CRUD.** Create, delete, info, info-by-revision all work against the DB. The response format includes the `url` field required by the Python client.
- **HF-compatible file operations.** `tree_list`, `preupload`, NDJSON `commit`, and `resolve_file` are all implemented. `resolve_file` returns the correct headers (`ETag`, `X-Linked-ETag`, `X-Linked-Size`, `X-Repo-Commit`) and handles both regular files and LFS files.
- **LFS Batch API (basic flow).** `lfs_batch`, `lfs_upload`, `lfs_download`, and `lfs_verify` cover the standard basic transfer. S3 presigned URLs are used for upload and download actions.
- **Xet transfer negotiation.** `lfs_batch` detects the `xet` transfer mode in the client's `transfers` list and returns `"xet"` as the transfer type when requested.
- **Access token lifecycle.** Create, list, and delete (hard) are implemented with the same ox_*/SHA-256 storage pattern as OpenXet.
- **Xet CAS token vending.** Both read and write token endpoints exist in the Hub API crate, producing JWT tokens with `sub`, `scope`, `revision`, `exp` claims. The CAS server validates these via middleware.
- **Branch and tag management.** Create and delete branches/tags, list refs, resolve revisions by name or SHA.
- **Revision resolution.** `resolve_repo_revision` handles "main" as a special case (maps to `head_sha`), SHA matches, and DB ref lookups. OpenXet's `tree_list` was hardcoded to `refs/heads/main`; xet-backend resolves the actual revision parameter.
- **YAML validation.** xet-backend actually parses YAML frontmatter and returns errors, while OpenXet's `validate_yaml` is a no-op stub.
- **Repo type normalization.** All three type prefixes (`models`, `datasets`, `spaces`) route to the same handlers, consistent with OpenXet's `normalize_repo_path` approach.
- **Error responses.** xet-backend returns JSON error bodies (`{"error": "..."}`) with `X-Error-Code` and `X-Error-Message` headers. This is an improvement over OpenXet's plain-text error bodies. The `Unauthorized` vs `Forbidden` distinction (401 vs 403) is correctly separated, whereas OpenXet maps both `AuthFailed` and `PermissionDenied` to 403.
- **Registration and login.** Both endpoints work and return ox_* tokens. Registration includes username length validation (3-64 chars) and duplicate checking.

## Out-of-scope items

- **Git Smart HTTP protocol.** xet-backend is designed as a compatibility layer over the Xet CAS protocol, not as a general-purpose Git server. `git clone`/`push` over Smart HTTP is not part of the current compatibility target. Content flows through the HF API (NDJSON commit + resolve) and the Xet/LFS protocols.

- **In-memory state caching (DashMaps).** OpenXet's pattern of in-memory primary state with async DB write-through is a fundamentally different persistence architecture. xet-backend is DB-primary with synchronous query paths. Replicating the in-memory caching pattern is not a gap to close but a deliberate architectural difference.

- **CAS chunking and deduplication pipeline.** The background worker that chunks LFS uploads into Blocks and builds file reconstructions is part of the CAS domain layer (`crate::cas`), not the API layer. This will be evaluated in the `compare_cas.md` document. The API-level gap is the absence of the `process_tx` channel plumbing.

- **Community features (discussions, PRs, likes).** OpenXet has DB schema and web UI handlers for these but no API endpoints. xet-backend's web UI is also a separate crate with no API-level exposure. These are web UI concerns.

## Recommendations for synthesis

- **Bearer tri-dispatch is a compatibility decision point.** OpenXet accepts a broader set of bearer token formats (`ox_*`, `username:password`, session token) than xet-backend, which only accepts `ox_*`. If compatibility work needs the OpenXet-style colon-format shortcut, the missing piece is fallback parsing in `resolve_bearer_token`; practical downstream reliance still needs verification.

- **Organization support is a prerequisite for collaborative repos.** Without org membership, write access is limited to the repo owner. Any multi-user workflow requires org CRUD and membership-based permission checks.

- **LFS multipart is needed for large objects.** The lack of multipart upload support limits LFS object size to the S3 single-part maximum (5 GiB). If large model files are expected, multipart support should be prioritized.

- **CommitResponse completeness.** `commitOid` is present, so the core response shape is already useful. OpenXet also returns `success` and `hookOutput`; if later compatibility testing shows callers expect them, adding those fields would be a small follow-up.

- **Token revocation.** Soft revocation (marking a token as inactive without deleting it) is useful for audit trails. Consider adding a revoke endpoint alongside the existing hard delete.

- **Error response format is already improved.** xet-backend's JSON error bodies and proper 401/403 distinction are better aligned with HuggingFace client expectations than OpenXet's plain-text errors. No action needed here.

- **CAS token duplication.** Both `hub-api/xet_auth.rs` and `cas-server/routes/auth.rs` implement Xet token vending with slightly different response formats. The Hub API version adds `X-Xet-*` response headers; the CAS server version does not. Consider consolidating or ensuring both paths return identical payloads to avoid client confusion depending on which endpoint is hit.

## Open questions

- Does the `huggingface_hub` Python client ever rely on the `username:password` Bearer token format in practice, or does it always go through a login flow first? This determines whether the tri-dispatch gap is a practical blocker.

- The web UI uses cookie-based sessions (visible in `web-ui/pages.rs` login/signup routes), but the cookie-to-user resolution mechanism is not visible in the files read. How does the web UI authenticate users after login? Does it store an ox_* token in the cookie, or is there a separate session mechanism?

- xet-backend creates a new ox_* token on every login. OpenXet creates an ephemeral session token. Does the accumulation of "login" tokens in the DB cause any practical issues (e.g., stale token growth)?

- The `update_repo_settings` endpoint returns 200 without actually persisting the visibility change. Is there a tracking issue for completing this implementation?

- OpenXet's `lfs_download` has a dual path (raw file vs. reconstructed blocks). xet-backend always redirects to S3. If CAS chunking is eventually implemented, will `lfs_download` need to support the reconstruction stream path?
