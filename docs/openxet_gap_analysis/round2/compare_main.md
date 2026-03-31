# Round 2 Comparison — `main`

**OpenXet commit:** `a5b6dfcdf5e806a5a29cb893f0d79d733bea309b`
**Analyst:** DiTo97
**Date:** 2026-03-31
**Tracking issue:** [#55](https://github.com/Atena-IT/open-hub/issues/55)
**Round 1 source:** `round1/openxet_main.md` (on branch `worktree-issue-46-openxet-main-map`; not yet merged to `main`)

---

## Scope

This document compares the OpenXet `main` module (the binary entry point and runtime composition root, mapped in Round 1) against the corresponding implementation in `xet-backend`. It covers startup sequence, shared state construction, router composition, configuration, tracing, and the overall crate dependency graph.

## xet-backend counterpart

The composition root lives in the `server` crate (`crates/server/src/main.rs`), which depends on six internal crates to assemble the unified binary. The xet-backend workspace splits responsibilities into eight crates, compared to OpenXet's seven `mod` declarations in a single binary crate.

| Responsibility | xet-backend location | Notes |
| --- | --- | --- |
| Binary entry point / composition root | `crates/server/src/main.rs` | `#[tokio::main]` async entry; binary name `xet-server` |
| Shared configuration | `crates/common/src/config.rs` — `AppConfig` | Loaded from env vars via `AppConfig::from_env()` |
| Database initialisation | `crates/db-layer/src/lib.rs` — `create_pool()` | PostgreSQL via sqlx; runs embedded migrations at pool creation |
| S3 / object storage client | `crates/s3-storage/src/lib.rs` — `S3Client` | Dual-endpoint (internal + presign) AWS SDK client |
| CAS protocol state and routes | `crates/cas-server/src/{state,router}.rs` | `AppState` (CAS-specific), JWT-protected CAS routes |
| Hub API state and routes | `crates/hub-api/src/{state,routes/mod}.rs` | `HubState`, HF-compatible API + LFS routes |
| Web UI routes | `crates/web-ui/src/lib.rs` | `WebState` wrapping `HubState` + Tera templates |
| Shared error type | `crates/common/src/error.rs` — `AppError` | 8-variant enum, JSON response bodies |
| Hash types | `crates/common/src/types.rs` — `MerkleHash` | 32-byte Blake3-keyed hashes |
| Shard binary parser | `crates/shard-parser/` | Standalone parsing library, no runtime role in main |

## Surface comparison

Status values: `covered` | `partial` | `missing` | `out-of-scope`

| OpenXet symbol / responsibility | Status | xet-backend equivalent | Notes |
| --- | --- | --- | --- |
| `main` fn (async tokio entry point) | `covered` | `crates/server/src/main.rs:11` | Same pattern: `#[tokio::main]`, anyhow `Result` |
| `AppState` (single shared state for all routes) | `partial` | `CasState` + `HubState` + `WebState` | xet-backend splits state into three types sharing `Arc<PgPool>` and `Arc<S3Client>` rather than one monolithic `AppState` |
| `AppState::with_db` (production constructor) | `covered` | `CasState::new()`, `HubState::from_arcs()` in `main.rs:54-56` | No in-memory preloading; DB is the live source of truth |
| `AppState::with_storage_path` (no-DB test path) | `missing` | — | xet-backend has no DB-less construction path; `DATABASE_URL` is required |
| `RepositoryStore` (in-memory repo index) | `missing` | — | xet-backend queries PostgreSQL directly per-request; no in-memory repo map |
| `CasStore` (in-memory CAS state) | `missing` | — | xet-backend queries PostgreSQL directly; no DashMap-backed in-memory CAS store |
| `AuthManager` (user/org/token management) | `partial` | `crates/hub-api/src/auth.rs` | Stateless functions against PgPool; no `AuthManager` object, no in-memory session map |
| `process_tx` (background CAS worker channel) | `missing` | — | xet-backend has no background chunking worker; LFS objects are persisted synchronously to S3 |
| `db::init_database` (SQLite schema bootstrap) | `covered` | `db_layer::create_pool()` at `main.rs:32` | PostgreSQL with sqlx embedded migrations (3 migration files) instead of raw DDL |
| Router composition (4 sub-routers in precedence order) | `covered` | `main.rs:62-67` — 4 `.merge()` calls | CAS router, Hub API router, LFS router, Web UI router |
| `hf_router()` (HF-compatible API) | `covered` | `hub_api::hub_router()` at `main.rs:64` | Equivalent scope: whoami, repos, tree, preupload, commit, resolve |
| `git_router()` (Git Smart HTTP) | `missing` | — | xet-backend has no Git Smart HTTP protocol (no upload-pack / receive-pack) |
| `web_ui::router()` (server-rendered pages) | `covered` | `web_ui::web_router()` at `main.rs:66` | Tera templates; similar route set but smaller scope |
| LFS batch API (part of `api` module in OpenXet) | `covered` | `hub_api::lfs_router()` at `main.rs:65` | Structural divergence: OpenXet bundles LFS routes inside the `api` module; xet-backend extracts them into a dedicated LFS router |
| Security headers (X-Frame-Options, CSP, etc.) | `missing` | — | xet-backend applies only `TraceLayer`; no security headers in the composition root |
| `DefaultBodyLimit::max(10 GiB)` | `missing` | — | No explicit body limit configured; relies on Axum defaults |
| Tracing initialisation (`EnvFilter`, debug defaults) | `covered` | `main.rs:13-19` | Uses `tracing_subscriber::fmt` with `EnvFilter`; defaults to `LOG_LEVEL` env var (default `info`) vs OpenXet's `debug` |
| `ensure_admin_user_secure()` (admin bootstrap) | `missing` | — | No admin bootstrap on startup; users created via `/api/auth/register` |
| `GIT_XET_STORAGE_PATH` env var (local storage root) | `out-of-scope` | — | xet-backend uses S3 exclusively; no local filesystem storage path |
| `PORT` / `HOST` env vars | `covered` | `BIND_ADDR` env var (default `0.0.0.0:3000`) | Single combined address vs OpenXet's separate `HOST`+`PORT` |
| `DISABLE_REGISTRATION` env var | `missing` | — | No registration gate in xet-backend; `/api/auth/register` is always open |

## Gaps

### Missing

- **Git Smart HTTP protocol.** OpenXet's `git_router()` provides `git-upload-pack`, `git-receive-pack`, and `git-info-refs` handlers. xet-backend has no equivalent; repositories are accessed exclusively via the HF-compatible API and LFS endpoints. This is an intentional architectural difference: xet-backend treats the Hub API as the sole client-facing protocol surface, not a compatibility shim over a git server.

- **Background CAS worker.** OpenXet decouples LFS upload latency from chunking via an unbounded mpsc channel and a `start_background_worker` task. xet-backend writes LFS objects directly to S3 in the request path with no background processing or chunking queue. The CAS xorb/shard/chunk pipeline is driven by the xet-core client, not the server.

- **In-memory state layer (DashMaps / RwLocks).** OpenXet's `RepositoryStore`, `CasStore`, and `AuthManager` hold all live state in-memory with fire-and-forget DB writes. xet-backend has no in-memory caches; every read and write goes to PostgreSQL. This eliminates the in-memory/DB divergence risk identified in the cross-module synthesis but adds per-request DB latency.

- **Admin bootstrap on startup.** OpenXet's `ensure_admin_user_secure()` creates a default admin user on first boot with a random password printed to logs. xet-backend has no equivalent; all user creation is via the registration API.

- **Security response headers.** OpenXet applies `X-Frame-Options: DENY`, `X-Content-Type-Options: nosniff`, `X-XSS-Protection`, `Referrer-Policy`, and a `Content-Security-Policy` via `SetResponseHeaderLayer`. xet-backend's composition root applies only `TraceLayer` for request logging. No hardening headers are set.

- **Body size limit.** OpenXet sets a 10 GiB global body limit to support large LFS uploads. xet-backend does not configure an explicit limit, defaulting to Axum's built-in 2 MiB limit. Large LFS uploads bypass this because they use presigned S3 URLs (client uploads directly to S3), so the missing limit is not a functional gap for the current LFS flow, but it would affect any future direct-upload endpoint.

- **No-DB construction path.** OpenXet's `with_storage_path` constructors allow running the full request stack without a database (useful for tests). xet-backend requires `DATABASE_URL` unconditionally; integration tests depend on a running PostgreSQL instance.

- **Registration gating (`DISABLE_REGISTRATION`).** OpenXet can disable new user registration via an env var. xet-backend's `/api/auth/register` is always available.

### Partial

- **Shared state model.** OpenXet uses a single `AppState` struct (defined in `api::handlers`) that is passed to every handler. xet-backend splits this into three state types: `CasState` (pool + s3 + config), `HubState` (pool + s3 + config, constructed from shared `Arc`s), and `WebState` (HubState + Tera). All three share the same underlying `Arc<PgPool>` and `Arc<S3Client>`. The surface area is equivalent, but the boundaries are different: OpenXet's `AppState` contains domain objects (`RepositoryStore`, `CasStore`, `AuthManager`); xet-backend's state types contain only infrastructure handles (pool, s3, config).

- **Auth system.** OpenXet's `AuthManager` is a stateful object with an in-memory session map, tri-dispatch bearer validation (`ox_*` / `username:password` / session token), and organisation membership management. xet-backend's auth is a set of stateless functions in `hub-api/src/auth.rs` that support only `ox_*` token validation via SHA-256 DB lookup. The `username:password` Bearer format and in-memory session tokens are not implemented. CAS routes use a separate JWT middleware (`cas-server/src/middleware.rs`) for xet-core client auth.

- **Configuration surface.** Both load configuration from environment variables, but the sets differ materially:

  | OpenXet | xet-backend | Overlap |
  | --- | --- | --- |
  | `GIT_XET_STORAGE_PATH` | — | none (S3-only in xet-backend) |
  | `PORT`, `HOST` | `BIND_ADDR` | equivalent, different shape |
  | `DISABLE_REGISTRATION` | — | none |
  | — | `DATABASE_URL` | PostgreSQL (required) |
  | — | `S3_ENDPOINT`, `S3_PUBLIC_ENDPOINT`, `S3_BUCKET`, `S3_REGION` | S3 configuration |
  | — | `JWT_SECRET`, `JWT_EXPIRY_SECS` | CAS JWT auth |
  | — | `HUB_TOKEN_SECRET` | ox_* token HMAC |
  | — | `CAS_BASE_URL`, `HUB_BASE_URL` | Service base URLs |
  | — | `LOG_LEVEL` | Tracing filter |

## Already covered

- **Async entry point pattern.** Both use `#[tokio::main]` with `anyhow::Result` return type, tracing init, and TCP listener binding via `tokio::net::TcpListener` + `axum::serve`.

- **Router composition with precedence ordering.** Both compose multiple sub-routers in a deliberate order using `Router::merge`. OpenXet: API -> HF -> Git -> Web UI. xet-backend: CAS -> Hub API -> LFS -> Web UI.

- **Database initialisation before state construction.** Both connect to the database and run schema setup before constructing shared state. OpenXet uses `db::init_database` (inline DDL); xet-backend uses `db_layer::create_pool` (sqlx migrations).

- **Separate construction of S3/storage client.** Both create the storage client as an independent step before wiring it into state (though OpenXet initialises `StorageBackend` inside `CasStore::with_db`, while xet-backend creates `S3Client` explicitly in `main.rs`).

- **Tera-based web UI as a separate composable router.** Both mount a Tera template-driven web UI as the last router layer with minimal coupling to the API layer.

- **`tracing_subscriber` with `EnvFilter`.** Both use the same tracing setup pattern. xet-backend defaults to `info` vs OpenXet's `debug`.

## Out-of-scope items

- **Git Smart HTTP protocol.** xet-backend does not implement Git Smart HTTP (upload-pack, receive-pack). This is an intentional design boundary per `CLAUDE.md`: "Treat the current Hub layer as compatibility-oriented behavior over the existing storage model, not as a fully featured Git server." Repositories are managed through the HF-compatible API and the xet-core CAS protocol, not native git operations.

- **Local filesystem storage (`StorageBackend` trait with `LocalStorage`).** OpenXet's storage module provides `LocalStorage` and `S3Storage` behind a trait. xet-backend uses S3 exclusively via `S3Client` (no trait abstraction, no local filesystem fallback). This is by design: the Docker Compose deployment always runs MinIO.

- **Community entity surface (discussions, pull requests, likes).** OpenXet's DB schema includes community tables exercised by `web_ui` but not by any API endpoint. xet-backend's DB schema has no community tables. These features are outside the current compatibility target.

- **SQLite / SeaORM.** OpenXet uses SQLite + SeaORM + raw DDL for schema evolution. xet-backend uses PostgreSQL + sqlx + embedded migration files. The database technology divergence is an architectural decision, not a gap to close.

## Recommendations for synthesis

- **State architecture divergence is load-bearing.** The split between OpenXet's in-memory-primary model and xet-backend's DB-primary model is the most consequential structural difference surfaced by this comparison. It affects every other module comparison (CAS, git, auth) because it changes how state is initialised, how failures propagate, and how tests are structured. Synthesis should document this as a first-order architectural decision, not a per-module gap.

- **Security headers should be evaluated as a single cross-cutting item.** The absence of `X-Frame-Options`, CSP, and other response headers in xet-backend is not specific to the `main` module; it affects all routes. Synthesis should track this as a global hardening task rather than a module-level gap.

- **Auth surface comparison deserves its own Round 2 document.** The auth differences (stateful AuthManager with sessions vs. stateless ox_* only; tri-dispatch bearer vs. single-dispatch; JWT for CAS vs. none in OpenXet CAS) are substantial enough that the `api` comparison should dedicate a section to auth parity mapping.

- **Git Smart HTTP absence is a compatibility boundary.** xet-backend intentionally omits native git operations. Any future compatibility target that depends on Smart HTTP-style clone/fetch/push flows would need that protocol surface, so the synthesis should flag this as a decision point.

- **Registration gating and admin bootstrap are deployment-level concerns.** Both are absent from xet-backend. If multi-tenant or production deployment is in scope, these should be tracked as operational gaps rather than protocol gaps.

## Open questions

- Does xet-backend need an explicit body size limit for any endpoint, or is the presigned-URL-to-S3 pattern sufficient for all large upload cases (including future ones)?
- Should xet-backend implement a registration gate (`DISABLE_REGISTRATION` equivalent) before any multi-tenant deployment, or is this deferred indefinitely?
- Is the absence of Git Smart HTTP a permanent architectural boundary, or is it expected to be revisited if git-based workflows ever enter scope?
- Should the three separate state types (`CasState`, `HubState`, `WebState`) be unified into a single `AppState` for consistency, or is the current split preferred for crate isolation?
- OpenXet's router ordering is load-bearing because `git_router` uses `/:owner/:repo/*` wildcards. With no git router in xet-backend, does the current merge order still have precedence risks between HF API resolve routes and Web UI `/{owner}/{repo}` routes?
