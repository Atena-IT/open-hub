# Round 2 Comparison — `web_ui`

**OpenXet commit:** `a5b6dfcdf5e806a5a29cb893f0d79d733bea309b`
**Analyst:** DiTo97
**Date:** 2026-03-31
**Tracking issue:** [#53](https://github.com/Atena-IT/open-hub/issues/53)
**Round 1 source:** `docs/openxet_gap_analysis/round1/openxet_web_ui.md` (on branch `issue-38-gap-analysis-templates`; not present on this branch)

---

## Scope

This document compares the OpenXet `web_ui` module (mapped in Round 1) against the corresponding implementation in `xet-backend`. Its goal is to enumerate gaps, alignment, and surfaces that need decision.

## xet-backend counterpart

The xet-backend web UI lives in the `web-ui` crate and a top-level `templates/` directory, with static assets in `static/`.

| Responsibility | xet-backend location | Notes |
| --- | --- | --- |
| Router assembly and state | `crates/web-ui/src/lib.rs` | `web_router()` returns `Router`; `WebState` wraps `HubState` + `Arc<Tera>` |
| Page handlers | `crates/web-ui/src/pages.rs` | Single flat file with all handlers; no sub-module decomposition |
| Template engine | `crates/web-ui/src/lib.rs` | Tera loaded at runtime from `TEMPLATE_DIR` env var (default `"templates"`) |
| HTML templates | `templates/*.html` | 8 files: `base.html`, `home.html`, `login.html`, `signup.html`, `new_repo.html`, `user_profile.html`, `repo_detail.html`, `repo_tree.html` |
| Static CSS | `static/style.css` | Single minified CSS file |
| Server integration | `crates/server/src/main.rs:66` | Merged last in the unified router via `.merge(web_ui::web_router(...))` |

## Surface comparison

Status values: `covered` | `partial` | `missing` | `out-of-scope`

### Routing and router assembly

| OpenXet symbol / responsibility | Status | xet-backend equivalent | Notes |
| --- | --- | --- | --- |
| `router()` (module entry point) | `covered` | `web_router()` in `crates/web-ui/src/lib.rs:16` | Different signature: accepts `HubState` + `template_dir` instead of zero-arg |
| `create_router()` (route table) | `partial` | Route definitions in `web_router()` body | xet-backend registers 10 routes vs. OpenXet's 45+ |
| `/-/` system-route prefix | `covered` | Used for `/-/login`, `/-/signup`, `/-/logout`, `/-/new` | Same collision-avoidance pattern |
| Static file serving | `covered` | `nest_service("/static", ServeDir::new("static"))` | OpenXet inlines CSS in templates; xet-backend uses `tower-http` `ServeDir` |

### Template engine

| OpenXet symbol / responsibility | Status | xet-backend equivalent | Notes |
| --- | --- | --- | --- |
| `TEMPLATES` (lazy global Tera) | `partial` | `Arc<Tera>` in `WebState` | xet-backend uses runtime-configurable `TEMPLATE_DIR` env var instead of compile-time `CARGO_MANIFEST_DIR` path; gracefully falls back to `Tera::default()` on error instead of `exit(1)` |
| `render()` helper | `covered` | `render()` in `crates/web-ui/src/pages.rs:11` | Local function; returns HTML response or 500 with error detail |
| `render_markdown()` helper | `missing` | — | `pulldown-cmark` and `ammonia` are in `Cargo.toml` dependencies but not used in any handler |
| 26+ templates + 2 component partials | `partial` | 8 templates, no component partials | Only `base.html`, `home.html`, `login.html`, `signup.html`, `new_repo.html`, `user_profile.html`, `repo_detail.html`, `repo_tree.html` exist |

### Auth handlers

| OpenXet symbol / responsibility | Status | xet-backend equivalent | Notes |
| --- | --- | --- | --- |
| `GET /-/login` / `POST /-/login` | `partial` | `login_page` / `login_submit` in `pages.rs:31,43` | Form renders but submit is a stub: returns "Login functionality coming soon" message. No cookie-based session. |
| `GET /-/signup` / `POST /-/signup` | `partial` | `signup_page` / `signup_submit` in `pages.rs:55,68` | Form renders but submit is a stub: returns "Registration functionality coming soon" message. |
| `GET /-/logout` | `partial` | `logout` in `pages.rs:81` | Redirects to `/-/login` but does not clear any cookie (no session exists to clear). |
| Cookie-based session auth (`get_current_user`) | `missing` | — | No session cookie, no `token=` header parsing, no in-memory session map. Web UI is completely unauthenticated. |
| CSRF protection (`CSRF_SECRET`, `generate_csrf_token`, `verify_csrf_token`) | `missing` | — | No CSRF tokens in any form template. No CSRF secret generation. |
| `LoginForm` / `SignupForm` with CSRF field | `partial` | `LoginForm` / `SignupForm` in `pages.rs:37,62` | Structs exist but have no `csrf_token` field |

### Home and profile handlers

| OpenXet symbol / responsibility | Status | xet-backend equivalent | Notes |
| --- | --- | --- | --- |
| `GET /` (home/index) | `covered` | `home` in `pages.rs:25` | Static welcome page; no user-aware content, no repo listing. OpenXet shows auth-aware landing. |
| `GET /:owner` (user profile) | `partial` | `user_profile` in `pages.rs:110` | Queries DB for user's repos and renders list. No visibility filtering: `list_repos_for_owner` returns private repos to unauthenticated visitors. |
| `GET /:owner/:repo` (repo detail) | `partial` | `repo_detail` in `pages.rs:141` | Shows file table from DB (`repo_files` table). No read-access check: exposes private repo metadata and file listings without auth. No tabbed view, no README rendering, no branch selector. |
| `GET /-/stats` | `missing` | — | No stats page |
| `GET /-/search` | `missing` | — | No search page |

### Repository handlers

| OpenXet symbol / responsibility | Status | xet-backend equivalent | Notes |
| --- | --- | --- | --- |
| `GET /:owner/:repo/tree/:ref` | `partial` | `repo_tree` in `pages.rs:175` | Queries `repo_files` from DB, not git tree objects. No read-access check on private repos. Path parameter extraction uses `Vec<(String, String)>` instead of typed struct. No branch resolution — `revision` param is accepted but not used to resolve git refs. |
| `GET /:owner/:repo/tree/:ref/*path` | `partial` | Same `repo_tree` handler | Path-filtered DB query via `list_files(pool, repo_id, path)`. No git tree parsing. |
| `GET /:owner/:repo/blob/:ref/*path` (blob view) | `missing` | — | No blob/file content viewer |
| `GET /:owner/:repo/commits/:ref` (commit list) | `missing` | — | No commit history page |
| `GET /:owner/:repo/commit/:sha` (commit detail) | `missing` | — | No single-commit view with diff |
| `POST /:owner/:repo/branches/new` (branch creation) | `missing` | — | No branch creation from web UI |
| `Breadcrumb` / `BranchInfo` types | `missing` | — | No breadcrumb trail or branch switcher UI |
| `CommitInfoView` type | `missing` | — | No commit rendering |
| `TreeEntryInfo` with LFS annotation | `partial` | Inline `serde_json::json!` in `repo_detail` | DB-sourced `is_lfs` boolean; no LFS status (raw/processing/chunked), no LFS size/oid fields |

### File editor handlers

| OpenXet symbol / responsibility | Status | xet-backend equivalent | Notes |
| --- | --- | --- | --- |
| `GET/POST /:owner/:repo/edit/:ref/*path` | `missing` | — | No web-based file editing |
| `GET/POST /:owner/:repo/new/:ref` | `missing` | — | No web-based file creation |
| Web-based git commit cycle (blob -> tree -> commit -> ref update) | `missing` | — | No git object creation from web UI |

### Discussion handlers

| OpenXet symbol / responsibility | Status | xet-backend equivalent | Notes |
| --- | --- | --- | --- |
| `GET /:owner/:repo/community` | `missing` | — | No community page |
| Discussion CRUD (list, create, view, comment, close/reopen) | `missing` | — | No discussion feature |
| `DiscussionInfo` / `TimelineItem` types | `missing` | — | No discussion data types |

### Pull request handlers

| OpenXet symbol / responsibility | Status | xet-backend equivalent | Notes |
| --- | --- | --- | --- |
| PR lifecycle (list, create, view, comment, close/reopen, merge) | `missing` | — | No pull request feature |
| Fast-forward merge via `update_ref` | `missing` | — | No merge capability |
| PR ref tracking (`refs/pull/{number}/head`) | `missing` | — | No PR ref creation |
| `PrInfo` / `PrTimelineItem` / `FileChange` types | `missing` | — | No PR data types |

### Organization handlers

| OpenXet symbol / responsibility | Status | xet-backend equivalent | Notes |
| --- | --- | --- | --- |
| `GET/POST /-/new` (repo creation form) | `partial` | `new_repo_page` / `new_repo_submit` in `pages.rs:85,97` | Form renders with name + private checkbox, but submit is a stub. No namespace/owner selector. |
| `GET/POST /-/new-org` (org creation) | `missing` | — | No organization creation |
| `POST /:owner/members` / `POST /:owner/members/:username/remove` | `missing` | — | No org membership management |

### Token handlers

| OpenXet symbol / responsibility | Status | xet-backend equivalent | Notes |
| --- | --- | --- | --- |
| `GET /-/settings/tokens` | `missing` | — | No token management UI |
| Token CRUD (create, revoke, delete) | `missing` | — | Token operations exist in hub-api (`auth.rs`) but have no web UI |
| `TokenDisplay` type | `missing` | — | No token display rendering |

### Trending and likes handlers

| OpenXet symbol / responsibility | Status | xet-backend equivalent | Notes |
| --- | --- | --- | --- |
| `GET /-/trending` | `missing` | — | No trending page |
| `POST /:owner/:repo/like` / `POST /:owner/:repo/unlike` | `missing` | — | No like feature |
| AJAX-aware dual JSON/redirect response | `missing` | — | No progressive-enhancement pattern |
| `TrendingRepo` type | `missing` | — | No trending data type |

### Utility modules

| OpenXet symbol / responsibility | Status | xet-backend equivalent | Notes |
| --- | --- | --- | --- |
| `routes::utils` (auth cookie, CSRF, display formatters, `Breadcrumb`, `BranchInfo`, `can_user_write_repo`) | `missing` | — | No shared handler utilities |
| `routes::lfs` (LFS pointer parsing, tree-entry LFS annotation) | `missing` | — | No git-level LFS annotation; `is_lfs` comes from DB |
| `routes::diff` (LCS-based line-level diff) | `missing` | — | No diff engine |
| `routes::tree_ops` (git tree serialization and mutation) | `missing` | — | No git tree manipulation from web layer |

### Shared state and dependencies

| OpenXet symbol / responsibility | Status | xet-backend equivalent | Notes |
| --- | --- | --- | --- |
| `AppState` (shared server state) | `partial` | `WebState` in `crates/web-ui/src/lib.rs:11` | Wraps `HubState` (PgPool + S3Client + AppConfig) + `Arc<Tera>`. No git `RepositoryStore`, no `CasStore`, no `AuthManager` with session map. |
| `crate::git` dependency (Repository, ObjectId, TreeEntry, CommitInfo) | `missing` | — | Web UI has no git module dependency; all data comes from PostgreSQL queries |
| `crate::cas` dependency (ContentHash, LfsObjectStatus) | `missing` | — | Web UI has no CAS module dependency |
| `crate::db::entities` community entities | `missing` | — | No discussion, PR, like entities imported; DB schema has no community tables |

## Gaps

### Missing

- **Repo read-access checks in browse handlers.** The `user_profile`, `repo_detail`, and `repo_tree` handlers perform no authentication and no visibility filtering. `list_repos_for_owner` (`repositories.rs:64`) returns all repos (including those with `private = true`), and `find_repo_by_full_name` (`repositories.rs:41`) returns private repos without checking the caller's identity. This means unauthenticated visitors can see private repo metadata (name, description, file paths, sizes) via `GET /:owner`, `GET /:owner/:repo`, and `GET /:owner/:repo/tree/:ref`. OpenXet gates browse handlers on the requesting user's read access to the repository.

- **Cookie-based session auth.** The web UI has no session management. All pages are unauthenticated. OpenXet uses a `token=` cookie with `HttpOnly; SameSite=Lax; Secure` flags and an in-memory session map via `AuthManager`.

- **CSRF protection.** No CSRF tokens in any form. OpenXet implements a custom CSRF scheme with per-process ephemeral HMAC secret, timestamp-bound tokens (1-hour TTL), and constant-time verification.

- **Blob view (file content viewer).** No route or handler to display individual file contents. OpenXet provides `/:owner/:repo/blob/:ref/*path` with syntax-aware rendering and LFS status annotation.

- **Commit history and diff viewing.** No commit list page, no single-commit detail page, no inline diff rendering. OpenXet provides paginated commit lists, per-commit diff views with a custom LCS diff engine, and HTML sanitization via ammonia.

- **Web-based file editor.** No create/edit/rename/delete file functionality from the browser. OpenXet implements a full git commit cycle (blob creation, tree mutation, commit object, ref update) from the web editor.

- **Discussion system.** No community/discussions feature. OpenXet provides full CRUD, threaded comments, close/reopen lifecycle with event logging, and author-only moderation.

- **Pull request system.** No PR feature. OpenXet provides PR CRUD, inline comments with file/line references, close/reopen/merge lifecycle, fast-forward merge via ref update, and PR ref tracking (`refs/pull/{number}/head`).

- **Organization management.** No org creation or membership management UI. OpenXet provides org creation, member add/remove, and namespace selection for repo creation.

- **Token management UI.** No web page to list, create, revoke, or delete `ox_*` access tokens. The API-level token operations exist in `hub-api/auth.rs` but have no web frontend.

- **Likes, trending, and search.** No repository like/unlike, no trending page, no search page, no stats page. OpenXet provides all four with AJAX-aware dual-response patterns for likes.

- **Markdown rendering.** `pulldown-cmark` and `ammonia` are declared in `Cargo.toml` but are not imported or used in any handler. No README rendering on repo detail pages.

- **Git-backed tree browsing.** The tree view queries the `repo_files` PostgreSQL table rather than parsing git tree objects. This means file listings depend on DB state, not the actual git repository content.

- **LFS pointer parsing and annotation.** No git-level LFS pointer detection. The `is_lfs` flag comes from the DB `repo_files` table. OpenXet parses LFS pointer text from blob content and annotates tree entries with status (raw/processing/chunked), size, and OID.

- **Diff engine.** No diff computation at any level. OpenXet implements a full LCS-based line-level diff with tree-recursive descent, binary detection, and HTML sanitization.

- **Git tree serialization and mutation.** No git tree binary format writing. OpenXet's `tree_ops` module provides `build_updated_tree`, `build_tree_with_addition`, `build_tree_with_deletion`, and `serialize_tree` for web-initiated commits.

- **Branch creation and branch selector.** No route to create branches from the web UI. No branch switcher component in templates. OpenXet provides `POST /:owner/:repo/branches/new` and a `BranchInfo`-populated dropdown in repo views.

### Partial

- **Login/signup forms.** HTML forms render correctly with proper input validation (minlength, maxlength, required). However, submit handlers are stubs that return "coming soon" messages. No actual credential verification, no session cookie creation. The `LoginForm` struct lacks a `csrf_token` field.

- **User profile page.** `user_profile` queries the DB for the owner's repositories and renders them with name, type badge, private badge, and description. However, `list_repos_for_owner` does not filter by the `private` column, so unauthenticated visitors see private repos. OpenXet gates profile listings on the requesting user's read access.

- **Repo detail page.** Displays owner, repo name, description, and file table. However, no read-access check on private repos, no tabbed interface (files/community/settings), no README rendering, no branch selector, no like count.

- **Tree browsing.** Route exists and accepts revision and path parameters. However, the revision parameter is ignored (no git ref resolution), and files come from DB rather than git objects. No breadcrumb navigation.

- **New repository form.** Form renders with name + private checkbox. Submit is a stub. No namespace/owner selector (OpenXet allows choosing between personal and org namespaces).

- **Template engine.** Tera is used in both. xet-backend uses a runtime-configurable `TEMPLATE_DIR` env var instead of OpenXet's compile-time `CARGO_MANIFEST_DIR` path, and falls back to `Tera::default()` on parse errors rather than calling `exit(1)`.

## Already covered

- **Tera template engine usage.** Both systems use Tera for server-rendered HTML. xet-backend uses a runtime-configurable template path; OpenXet uses a compile-time `CARGO_MANIFEST_DIR` path.

- **`/-/` system route prefix.** Both use the `/-/` prefix for system routes to avoid collision with `/:owner` dynamic segments.

- **Static asset serving.** xet-backend serves static files via `tower-http::ServeDir` at `/static`. OpenXet inlines CSS in templates.

- **Server integration pattern.** The web UI router is merged last in the unified router (after CAS, Hub API, and LFS routers), matching OpenXet's precedence ordering where specific routes come before wildcard `/:owner` routes.

- **Decoupled module design.** The `web-ui` crate has minimal coupling: it depends on `hub-api` (for `HubState`) and `db-layer` (for queries), and is merged into the server via a single `.merge()` call. This mirrors OpenXet's explicit removal contract.

## Out-of-scope items

- **PR merge conflict detection and three-way merge.** OpenXet itself only implements fast-forward merge with no conflict detection. This is a known limitation in both systems, not a gap unique to xet-backend. The repository boundaries document treats the current Hub layer as compatibility-oriented behavior over the existing storage model, not as a fully featured Git server.

- **Trending via raw SQL.** OpenXet's `get_trending_repos` uses non-parameterized SQL with integer-clamped limits. This is an OpenXet implementation detail; xet-backend's DB layer uses sqlx with parameterized queries throughout, so if trending is implemented, it would not share this pattern.

- **Discussion/PR close authorization model.** OpenXet's strict username string comparison for close/reopen authorization is an OpenXet-specific design choice. If these features are implemented in xet-backend, the authorization model should be designed independently.

## Recommendations for synthesis

- **Session auth blocks state-changing handlers.** Without cookie-based sessions, no state-changing web UI handler can distinguish users. Session management (cookie + in-memory or DB-backed map) is a prerequisite for all other web UI work that requires authentication.

- **CSRF protection depends on session auth.** Once sessions exist, every state-changing form needs CSRF tokens. xet-backend already has `sha2` in the dependency tree (via `hub-api/auth.rs`).

- **Git-backed tree browsing is an architectural decision.** xet-backend's tree view queries PostgreSQL `repo_files`, while OpenXet parses git tree objects directly. If xet-backend continues with DB-backed file metadata (as opposed to a git object store), many OpenXet web UI features (blob view, diff, web editor commits) would need different implementations.

- **Markdown rendering dependencies are declared but unused.** The `pulldown-cmark` and `ammonia` dependencies are already in `Cargo.toml`. Adding README rendering to the repo detail page would stay within the existing web UI stack.

- **Community features (discussions, PRs, likes) depend on DB schema.** The xet-backend DB schema (managed by sqlx migrations in `db-layer`) does not include community tables (`discussions`, `pull_requests`, `repo_likes`, etc.). Adding these features requires new migrations before handlers can be written.

- **Token management UI has API backing.** The API-level token operations (generate, hash, verify) already exist in `hub-api/auth.rs`. A web frontend would need a new page handler and template, plus session auth.

- **Template path approach differs.** xet-backend's runtime `TEMPLATE_DIR` env var avoids a compile-time path dependency. OpenXet's `CARGO_MANIFEST_DIR`-based path is absent in release images that do not ship the source tree.

## Open questions

- Should xet-backend's web UI read file trees from the PostgreSQL `repo_files` table (current approach) or from git objects (OpenXet's approach)? This is an architectural fork that affects every file-browsing, diff, and editing feature.

- Is the web UI intended to reach feature parity with OpenXet's community features (discussions, PRs, likes, trending), or is it scoped to basic repo browsing and management? The roadmap and feature support matrix do not explicitly state the target scope for web UI features.

- Should session tokens be DB-backed (surviving restarts) or in-memory only (matching OpenXet's model)? xet-backend uses PostgreSQL, which makes DB-backed sessions straightforward, but OpenXet's ephemeral model was an intentional simplicity trade-off.

- The `pulldown-cmark` and `ammonia` dependencies in `web-ui/Cargo.toml` are currently unused. Were they added in anticipation of README rendering, or should they be removed until needed to keep the dependency tree clean?

- Should the web UI implement its own write-access check (analogous to OpenXet's `can_user_write_repo`), or should it delegate entirely to `hub-api/auth.rs::ensure_repo_write_access`? The current auth module only checks owner equality, with no org-member or collaborator support.
