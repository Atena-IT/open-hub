# Round 1 Module Map — `main`

**OpenXet commit:** `a5b6dfcdf5e806a5a29cb893f0d79d733bea309b`
**Analyst:** DiTo97
**Date:** 2026-03-31
**Tracking issue:** [#46](https://github.com/Atena-IT/open-hub/issues/46)

---

## Scope

This document maps the OpenXet `main` module (the binary entry point and runtime composition root). It covers only the upstream OpenXet source — no `xet-backend` references appear here. Round 2 will introduce the comparison.

## Source location

```
../OpenXet-reference/src/main.rs
```

Key entry points:
- `src/main.rs` — sole file; declares all sub-modules, builds the Axum router, and calls `tokio::main`

## Public surface

The `main` module has no library-visible public exports (it is a binary crate entry point). Everything wired here is consumed indirectly via the composed `Router` at runtime.

| Symbol | Kind | File | Brief description |
| --- | --- | --- | --- |
| `main` | `fn` (async, `#[tokio::main]`) | `src/main.rs` | Async entry point: initialises tracing, DB, state, router, and starts the TCP listener |

## Internal structure

### Sub-modules (declared in main.rs)

| Sub-module | Purpose |
| --- | --- |
| `api` | HTTP handlers, `AppState`, auth, Git Smart HTTP, LFS, HF-compatible router |
| `cas` | Content-addressable store (chunks, blocks, reconstructions) |
| `db` | SQLite database initialisation via SeaORM |
| `error` | Shared `ServerError` type and `Result` alias |
| `git` | Git Smart HTTP protocol implementation (`RepositoryStore`, pack encoding) |
| `storage` | Low-level file/block storage abstraction |
| `web_ui` | Server-rendered web interface (Tera templates) |

### Key types

#### `AppState` (defined in `src/api/handlers.rs`, re-exported from `src/api/mod.rs`)

Central shared state, wrapped in `Arc<AppState>` and passed to every Axum handler via `State<Arc<AppState>>`.

Fields:
- `repos: RepositoryStore` — in-memory + DB-backed repository index
- `cas: Arc<CasStore>` — content-addressable store (chunks and LFS objects)
- `auth: AuthManager` — user/org/token management backed by Argon2id + SQLite
- `process_tx: mpsc::UnboundedSender<ContentHash>` — channel to a background Tokio worker that performs chunking and deduplication
- `db: Option<Arc<DatabaseConnection>>` — optional SeaORM connection (always `Some` in the `with_db` boot path)

Two constructors:
- `AppState::with_storage_path(path)` — in-process, no DB (test/dev)
- `AppState::with_db(path, db)` — async, loads repos and CAS chunks from DB, used in production boot

#### `AuthManager` (defined in `src/api/auth.rs`, re-exported from `src/api/mod.rs`)

Handles user registration, login, organisation membership, and long-lived access tokens. `ensure_admin_user_secure` is called at startup to bootstrap a default admin on first run; if the admin already exists the method returns `None` (no password is printed).

## Intra-module dependencies

`main.rs` imports from the following modules directly:

| Dependency module | What is used |
| --- | --- |
| `api` | `AppState`, all HTTP handler functions, `hf_router()`, `git_router()` |
| `db` | `db::init_database(&db_path)` |
| `web_ui` | `web_ui::router()` |

`cas`, `error`, `git`, `storage` are declared as modules but not directly imported in `main.rs`; they are consumed transitively through `api` and `db`.

## Notable implementation details

**Router composition order matters.** The call order in `main.rs` is intentional:
1. Explicit `/api/*` routes are registered first.
2. `api::hf_router()` is merged next (catches `/api/*` HF paths before the generic git router can).
3. `api::git_router()` is merged after (uses `/:owner/:repo/*` wildcard patterns).
4. `web_ui::router()` is merged last (also uses `/:owner/:repo` patterns but non-overlapping paths).

**Body limit.** `DefaultBodyLimit::max(10 * 1024 * 1024 * 1024)` — 10 GiB limit applied globally to support large LFS uploads.

**Security headers applied via `tower_http::set_header::SetResponseHeaderLayer`:**
- `X-Frame-Options: DENY`
- `X-Content-Type-Options: nosniff`
- `X-XSS-Protection: 1; mode=block`
- `Referrer-Policy: strict-origin-when-cross-origin`
- `Content-Security-Policy` restricting scripts/styles to `'self'` + cdnjs

**Configuration via environment variables (no config file):**

| Variable | Default | Purpose |
| --- | --- | --- |
| `GIT_XET_STORAGE_PATH` | `$TMPDIR/git-xet-storage` | Root for local storage and SQLite DB |
| `PORT` | `8080` | TCP listen port |
| `HOST` | `127.0.0.1` | TCP listen host |
| `DISABLE_REGISTRATION` | unset / `false` | When `1` or `true`, blocks new user registration |

**Tracing initialisation.** Uses `tracing_subscriber` with `EnvFilter` defaulting to `git_xet_server=debug,tower_http=debug`. No structured/JSON logging emitter is configured.

**Background CAS worker.** `CasStore::start_background_worker(cas.clone())` is called inside `AppState::with_db`. It receives `ContentHash` values over an unbounded mpsc channel (`process_tx`) and performs chunking/deduplication asynchronously. The channel is never explicitly shut down before `axum::serve` exits.

**Admin bootstrap.** On every startup `state.auth.ensure_admin_user_secure()` is called. It creates the `admin` user only if absent, using either the password from an environment variable (not visible in `main.rs` — logic is inside `AuthManager`) or a randomly generated one printed to `tracing::warn!`. This means the random password is visible in logs only on the very first boot.

**SQLite path construction.** DB file is placed at `$GIT_XET_STORAGE_PATH/git-xet.db`. No migration guard is visible in `main.rs`; `db::init_database` is expected to run migrations internally.

## Open questions

- `ensure_admin_user_secure` reads from an env var for the admin password — the variable name is not visible in `main.rs`. What is the variable name and does it interact with `DISABLE_REGISTRATION`?
- `CasStore::start_background_worker` uses an unbounded channel. Is there a shutdown signal sent before the process exits, or are in-flight items dropped silently?
- `AppState::with_storage_path` (no-DB path) is still in scope — is it exercised in tests or is `with_db` the only production path?
- The `storage` and `git` modules are declared but never directly imported in `main.rs`. Are they only used via `api`/`cas`, or do they expose types that `main.rs` will import in future work?
- No TLS configuration is present — is TLS handled by a reverse proxy (e.g., nginx/Caddy) in all deployment topologies, or is `rustls` integration planned?
- The HF-compatible router routes are merged before the Git router to avoid path conflicts; it is unclear whether there are any actual overlapping routes or whether the ordering is purely defensive.
