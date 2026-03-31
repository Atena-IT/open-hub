# Round 1 Module Map — `web_ui`

**OpenXet commit:** `a5b6dfcdf5e806a5a29cb893f0d79d733bea309b`
**Analyst:** DiTo97
**Date:** 2026-03-31
**Tracking issue:** [#44](https://github.com/Atena-IT/open-hub/issues/44)

---

## Scope

This document maps the OpenXet `web_ui` module. It covers only the upstream OpenXet source — no `xet-backend` references appear here. Round 2 will introduce the comparison.

## Source location

```
src/web_ui/
```

Key entry points:
- `src/web_ui/mod.rs` — module root; exposes the single public symbol `router()`
- `src/web_ui/routes.rs` — route table assembly (`create_router()`); re-exports `get_like_count` and `has_user_liked`
- `src/web_ui/templates.rs` — Tera template engine bootstrap; `TEMPLATES` lazy global; `render()` and `render_markdown()` helpers
- `src/web_ui/routes/utils.rs` — shared handler utilities: auth cookie extraction, CSRF generation/verification, display formatters, `Breadcrumb`, `BranchInfo`, `can_user_write_repo`
- `src/web_ui/routes/auth_handlers.rs` — login, signup, logout
- `src/web_ui/routes/home_handlers.rs` — home page, user/org profile page, repository detail (tabbed), stats, search
- `src/web_ui/routes/repo_handlers.rs` — tree browsing, blob view, commit list, commit detail, branch creation, README rendering
- `src/web_ui/routes/edit_handlers.rs` — web-based file editor: edit existing, create new, commit changes
- `src/web_ui/routes/discussion_handlers.rs` — community discussions: list, create, view thread, comment, close/reopen
- `src/web_ui/routes/pr_handlers.rs` — pull requests: list, create, view, comment, close/reopen, merge (fast-forward)
- `src/web_ui/routes/org_handlers.rs` — repository creation form, organization creation, org membership management
- `src/web_ui/routes/token_handlers.rs` — access token management: list, create, revoke, delete
- `src/web_ui/routes/trending_handlers.rs` — trending repositories page
- `src/web_ui/routes/like_handlers.rs` — repository likes/stars: like, unlike, counts, trending query
- `src/web_ui/routes/lfs.rs` — LFS pointer parsing and tree-entry LFS status checks
- `src/web_ui/routes/diff.rs` — LCS-based line-level diff computation
- `src/web_ui/routes/tree_ops.rs` — git tree manipulation: update, add, delete entries

HTML templates live at:

```
src/web_ui/templates/
```

Twenty-six top-level templates plus two component partials (`components/dropdown.html`, `components/modal.html`).

## Public surface

The `web_ui` module exposes a minimal surface. `routes` and `templates` are private sub-modules; callers outside `web_ui` only see:

| Symbol | Kind | File | Brief description |
| --- | --- | --- | --- |
| `router` | `fn` | `src/web_ui/mod.rs` | Returns `Router<Arc<AppState>>` for all web UI routes; intended to be mounted with `.merge(web_ui::router())` |

Within `routes.rs`, two items are re-exported for use by sibling handlers inside the `routes` sub-module:

| Symbol | Kind | File | Brief description |
| --- | --- | --- | --- |
| `get_like_count` | `fn` (re-export from `like_handlers`) | `src/web_ui/routes.rs` | Queries DB for the like count of a repository |
| `has_user_liked` | `fn` (re-export from `like_handlers`) | `src/web_ui/routes.rs` | Returns whether a given user has liked a repository |

These are not accessible from outside `web_ui` because `routes` itself is not `pub`.

### Route table

`create_router()` registers the following routes:

| Method | Path | Handler |
| --- | --- | --- |
| `GET` | `/` | `home_handlers::index` |
| `GET` | `/-/login` | `auth_handlers::login_page` |
| `POST` | `/-/login` | `auth_handlers::login_submit` |
| `GET` | `/-/signup` | `auth_handlers::signup_page` |
| `POST` | `/-/signup` | `auth_handlers::signup_submit` |
| `GET` | `/-/logout` | `auth_handlers::logout` |
| `GET` | `/-/new` | `org_handlers::new_repo_page` |
| `POST` | `/-/new` | `org_handlers::create_repo` |
| `GET` | `/-/new-org` | `org_handlers::new_org_page` |
| `POST` | `/-/new-org` | `org_handlers::create_org` |
| `GET` | `/-/stats` | `home_handlers::stats` |
| `GET` | `/-/search` | `home_handlers::search` |
| `GET` | `/-/trending` | `trending_handlers::trending_page` |
| `GET` | `/-/settings/tokens` | `token_handlers::tokens_page` |
| `POST` | `/-/settings/tokens` | `token_handlers::create_token` |
| `POST` | `/-/settings/tokens/:id/revoke` | `token_handlers::revoke_token` |
| `POST` | `/-/settings/tokens/:id/delete` | `token_handlers::delete_token` |
| `GET` | `/:owner` | `home_handlers::user_profile` |
| `POST` | `/:owner/members` | `org_handlers::add_org_member` |
| `POST` | `/:owner/members/:username/remove` | `org_handlers::remove_org_member` |
| `GET` | `/:owner/:repo` | `home_handlers::repo_detail` |
| `GET` | `/:owner/:repo/tree/:ref` | `repo_handlers::tree_root` |
| `GET` | `/:owner/:repo/tree/:ref/*path` | `repo_handlers::tree_path` |
| `GET` | `/:owner/:repo/blob/:ref/*path` | `repo_handlers::blob_view` |
| `GET` | `/:owner/:repo/edit/:ref/*path` | `edit_handlers::edit_file` |
| `POST` | `/:owner/:repo/edit/:ref/*path` | `edit_handlers::commit_file` |
| `GET` | `/:owner/:repo/new/:ref` | `edit_handlers::new_file` |
| `POST` | `/:owner/:repo/new/:ref` | `edit_handlers::commit_new_file` |
| `GET` | `/:owner/:repo/new/:ref/*path` | `edit_handlers::new_file_in_dir` |
| `POST` | `/:owner/:repo/new/:ref/*path` | `edit_handlers::commit_new_file_in_dir` |
| `GET` | `/:owner/:repo/commits/:ref` | `repo_handlers::commits_list` |
| `GET` | `/:owner/:repo/commit/:sha` | `repo_handlers::commit_view` |
| `GET` | `/:owner/:repo/community` | `discussion_handlers::community_page` |
| `GET` | `/:owner/:repo/discussions` | `discussion_handlers::discussions_list` |
| `GET` | `/:owner/:repo/discussions/new` | `discussion_handlers::new_discussion_page` |
| `POST` | `/:owner/:repo/discussions/new` | `discussion_handlers::create_discussion` |
| `GET` | `/:owner/:repo/discussions/:id` | `discussion_handlers::discussion_detail` |
| `POST` | `/:owner/:repo/discussions/:id` | `discussion_handlers::post_comment` |
| `POST` | `/:owner/:repo/discussions/:id/close` | `discussion_handlers::close_discussion` |
| `POST` | `/:owner/:repo/discussions/:id/reopen` | `discussion_handlers::reopen_discussion` |
| `GET` | `/:owner/:repo/pulls` | `pr_handlers::pr_list` |
| `POST` | `/:owner/:repo/branches/new` | `repo_handlers::create_branch` |
| `GET` | `/:owner/:repo/pulls/new` | `pr_handlers::new_pr_page` |
| `POST` | `/:owner/:repo/pulls/new` | `pr_handlers::create_pr` |
| `GET` | `/:owner/:repo/pulls/:number` | `pr_handlers::pr_detail` |
| `POST` | `/:owner/:repo/pulls/:number` | `pr_handlers::post_pr_comment` |
| `POST` | `/:owner/:repo/pulls/:number/close` | `pr_handlers::close_pr` |
| `POST` | `/:owner/:repo/pulls/:number/reopen` | `pr_handlers::reopen_pr` |
| `POST` | `/:owner/:repo/pulls/:number/merge` | `pr_handlers::merge_pr` |
| `POST` | `/:owner/:repo/like` | `like_handlers::like_repo` |
| `POST` | `/:owner/:repo/unlike` | `like_handlers::unlike_repo` |

System routes use the `/-/` prefix to avoid collision with `/:owner` dynamic segments.

## Internal structure

### Sub-modules

| Sub-module | Purpose |
| --- | --- |
| `routes` | Route table assembly and all handler sub-modules |
| `templates` | Tera engine bootstrap and markdown rendering helpers |
| `routes::utils` | Shared handler helpers: auth cookie extraction, CSRF token lifecycle, display formatters (`format_size`, `format_time_ago`, `format_relative_time`), `Breadcrumb`, `BranchInfo`, `can_user_write_repo` |
| `routes::auth_handlers` | HTTP handlers for login, signup, logout; CSRF enforcement |
| `routes::home_handlers` | Home page, user/org profile, tabbed repo detail, stats, search (HTML + JSON dual-mode) |
| `routes::repo_handlers` | Tree browsing (root and path), blob view, paginated commit list, single commit with diff, branch creation |
| `routes::edit_handlers` | Web-based file editor for text files: GET (show editor), POST (commit change); handles edit, rename, and new file cases |
| `routes::discussion_handlers` | Per-repo discussion threads: CRUD, comments, close/reopen lifecycle with event log |
| `routes::pr_handlers` | Pull requests: CRUD, comment, close/reopen/merge with event log and inline diff |
| `routes::org_handlers` | Repository creation (with namespace selection), organization creation, org membership add/remove |
| `routes::token_handlers` | Access token lifecycle through the web UI: list, create, revoke (soft), delete (hard) |
| `routes::trending_handlers` | Trending page: queries like counts via raw SQL aggregate, delegates to `like_handlers::get_trending_repos` |
| `routes::like_handlers` | Repository like/unlike (AJAX-aware dual JSON/redirect responses), like count query, trending repo query |
| `routes::lfs` | LFS pointer text parsing (`parse_lfs_pointer`), per-tree-entry LFS status annotation (`check_lfs_file`) |
| `routes::diff` | LCS-based line-level file diff: `compute_diff` (tree-to-tree), `compute_diff_recursive` (subdirectory descent), `compute_file_diff` (single file), `generate_diff` / `compute_lcs` / `build_diff_from_lcs` |
| `routes::tree_ops` | Git tree binary serialization and recursive tree mutation: `build_updated_tree`, `build_tree_with_addition`, `build_tree_with_deletion`, `serialize_tree` |

### Key types

#### `TEMPLATES` (`src/web_ui/templates.rs`)

A `once_cell::sync::Lazy<Tera>` global. Tera is initialized once on first access by loading all files matching `src/web_ui/templates/**/*` using a path built from `env!("CARGO_MANIFEST_DIR")` at compile time. A parse error causes `std::process::exit(1)` — the process cannot start with broken templates.

#### `CSRF_SECRET` (`src/web_ui/routes/utils.rs`)

A `once_cell::sync::Lazy<[u8; 32]>` global initialized with 32 cryptographically random bytes via `rand::thread_rng().fill()`. The secret is per-process and not persisted. CSRF tokens are `SHA-256(secret || session_token_or_"anonymous" || timestamp_le_u64)[..16]` prepended by the 8-byte timestamp, base64url-encoded. Tokens are valid for 1 hour.

#### `Breadcrumb` / `BranchInfo` (`src/web_ui/routes/utils.rs`)

Lightweight `serde::Serialize` structs for template context injection:
- `Breadcrumb { name: String, path: String }` — one entry in a path breadcrumb trail
- `BranchInfo { name: String }` — a branch name for the branch switcher

#### `LoginForm` / `SignupForm` (`src/web_ui/routes/auth_handlers.rs`)

`serde::Deserialize` structs for HTML form data. Both include a `csrf_token: String` field; handlers verify it before processing credentials.

#### `CommitInfoView` (`src/web_ui/routes/repo_handlers.rs`)

A `serde::Serialize` projection of `crate::git::CommitInfo` for template rendering. Adds pre-formatted fields: `short_id` (7-char SHA prefix), `short_message` (first line only), `author_time_ago` (relative string). Constructed via `From<&CommitInfo>`.

#### `TreeEntryInfo` (`src/web_ui/routes/home_handlers.rs`, `src/web_ui/routes/repo_handlers.rs`)

Template context struct for file tree entries. Fields: `name`, `full_path`, `is_dir: bool`, `is_lfs: bool`, `lfs_status: Option<String>` (raw/processing/chunked), `lfs_size: Option<String>`, `lfs_oid: Option<String>`.

#### `DiscussionInfo` / `TimelineItem` (`src/web_ui/routes/discussion_handlers.rs`)

- `DiscussionInfo` — flat view of a discussion for list pages: `id`, `title`, `author`, `status`, `comment_count`, `created_at`, `updated_at` (relative time strings).
- `TimelineItem` — unified chronological entry in a discussion thread: `item_type` (`"op"`, `"comment"`, or `"event"`), optional `content`, `event_type`, `old_value`, `new_value`.

#### `PrInfo` / `PrTimelineItem` / `FileChange` (`src/web_ui/routes/pr_handlers.rs`)

- `PrInfo` — list-view PR summary struct.
- `PrTimelineItem` — PR thread entry; `item_type` is `"description"`, `"comment"`, or `"event"`; includes optional `file_path` and `line_number` for inline comments.
- `FileChange` — per-file diff result for PR detail view: `path`, `additions`, `deletions`, `diff_html` (pre-rendered HTML string with inline `<span class="diff-line add|del">` elements).

#### `LfsPointerInfo` (`src/web_ui/routes/lfs.rs`)

Internal struct `{ oid: ContentHash, size: u64 }` parsed from Git LFS pointer text.

#### `FileChange` / `DiffLine` (`src/web_ui/routes/diff.rs`)

- `FileChange` — diff output per file: `path`, `status` (added/modified/deleted), `additions`, `deletions`, `diff_lines: Vec<DiffLine>`, `is_binary: bool`.
- `DiffLine` — one diff line: `line_type` (add/del/context/header), `content` (ammonia-sanitized), `old_line: Option<usize>`, `new_line: Option<usize>`.

#### `TokenDisplay` (`src/web_ui/routes/token_handlers.rs`)

Template-facing token representation: `id`, `name`, `prefix` (first 8 chars of `ox_*` token), `description`, `scopes`, `last_used` / `created` / `expires` (relative time strings), `is_active: bool`.

#### `TrendingRepo` (`src/web_ui/routes/like_handlers.rs`)

`{ name, owner, repo, like_count: u64, description: Option<String> }` — one entry in the trending list; `description` is always `None` (no repo description storage exists).

## Intra-module dependencies

| Dependency module | What is used |
| --- | --- |
| `crate::api` | `AppState` (Axum shared state, accessed via `State<Arc<AppState>>` extractor in every handler) |
| `crate::cas` | `ContentHash` (LFS pointer OID type in `routes::lfs`); `store::LfsObjectStatus` (raw/processing/chunked enum for tree/blob LFS annotation) |
| `crate::db::entities` | `user`, `org_member` (auth, profile, org management); `pull_request`, `pr_comment`, `pr_event` (PR lifecycle); `discussion`, `discussion_comment`, `discussion_event` (discussion lifecycle); `repo_like` (likes/trending) |
| `crate::git` | `Repository` (tree parsing, blob reading, ref resolution, commit creation, object creation); `ObjectId` (object addressing); `ObjectType` (Blob/Tree when creating git objects); `TreeEntry` (tree entry struct used in diff and tree manipulation); `CommitInfo` (commit metadata for history views) |

## Notable implementation details

### Cookie-based session auth

User identity is propagated entirely via a `token=<session_token>` cookie. `get_current_user()` parses the `cookie` header (semicolon-split), strips the `token=` prefix, and calls `AuthManager::get_username_for_token()`. No `Authorization` header is used in the web UI path. The cookie is set with `HttpOnly; SameSite=Lax; Secure` on login and cleared with `Max-Age=0` on logout.

### Custom CSRF protection

Every state-changing form (login, signup, edit, branch create, PR create, discussion create, org management, token management, like/unlike) carries a `csrf_token` field. The token encodes an 8-byte Unix timestamp and a 16-byte HMAC (SHA-256 truncated), base64url-encoded. Verification checks: (1) correct length after decode; (2) token age ≤ 3600 s; (3) recomputed HMAC matches via `subtle::ConstantTimeEq`. The CSRF secret is random per process-start — secrets are lost on restart along with session tokens.

### Tera template engine and compile-time path

Templates are loaded at runtime from a path composed at compile time:

```rust
concat!(env!("CARGO_MANIFEST_DIR"), "/src/web_ui/templates/**/*")
```

This means the source tree must be present on the filesystem at the path captured during compilation. In Docker images built via multi-stage Dockerfile this path is typically absent in the final image. A template parse error calls `std::process::exit(1)`, so startup fails loudly rather than panicking at request time.

### Markdown rendering with sanitization

`render_markdown()` uses `pulldown-cmark` with `ENABLE_TABLES | ENABLE_STRIKETHROUGH | ENABLE_TASKLISTS`. The resulting HTML is sanitized by `ammonia` before insertion into the template. The same `ammonia::clean()` call is applied to every diff line's content in `generate_diff()`.

### Custom LCS diff

The diff engine in `routes::diff` implements the classic O(mn) dynamic-programming LCS algorithm from scratch (no external diff crate). All diff content is sanitized with `ammonia::clean()`. The diff is tree-recursive: `compute_diff` calls `compute_diff_recursive` for subdirectory entries. Null-byte detection (`iter().take(8000).any(|&b| b == 0)`) classifies files as binary; binary files receive a single `"Binary file changed"` header line.

### Git tree serialization

`routes::tree_ops::serialize_tree` writes git tree binary format: entries sorted with directory names suffixed by `/` for comparison (matching git's sort order), each written as `mode SP name NUL raw_oid_bytes`. This output is passed directly to `crate::git::Repository::create_object(ObjectType::Tree, &data)`.

### Web-based git commits

`edit_handlers::commit_file_impl` performs a full commit cycle through the git layer: create blob object → mutate tree (update/add/delete) → create commit object referencing the new tree and the previous HEAD commit → update the branch ref. The commit author is hardcoded to `"Web User"` / `"web@openxet.local"` regardless of the authenticated user.

### PR merge strategy

`pr_handlers::merge_pr` performs a fast-forward merge by calling `repo_handle.update_ref(&target_ref, source_commit_id)`. No three-way merge, no merge commit, no conflict detection. The source branch tip is resolved at merge time (not at PR creation time), so the effective merge point may differ from what the PR was opened against.

### PR ref tracking

`create_pr` creates a `refs/pull/{number}/head` git ref pointing to the source branch commit at PR creation time. This preserves the source commit even if the source branch is later deleted. Creation failures are logged as warnings but do not fail the PR creation.

### AJAX-aware like/unlike

`like_handlers::like_repo` and `unlike_repo` inspect the `Accept` header: if it contains `application/json`, they return `Json(LikeResponse)` (JSON with `success`, `liked`, `like_count`, optional `error`); otherwise they issue a redirect. This supports both progressive-enhancement HTML forms and JavaScript fetch calls from the same endpoint.

### Trending via raw SQL

`like_handlers::get_trending_repos` constructs a `COUNT(*) GROUP BY repo_name ORDER BY like_count DESC LIMIT {safe_limit}` query by string formatting. The limit is clamped to 1..100 using integer arithmetic before embedding (no SQL injection from user input), but the query is not parameterized.

### Discussion close/reopen authorization

`discussion_handlers::update_discussion_status` allows close/reopen only when `current_user == owner` (string comparison of cookie-extracted username against the URL `owner` path segment). PR status changes use a broader rule: owner or PR author can close/reopen, but only users with write permission (`can_user_write_repo`) can merge.

### Module removal contract

`src/web_ui/mod.rs` documents a removal procedure in comments: delete the directory, remove two lines from `main.rs`, and remove `tera = "1"` from `Cargo.toml`. The module is explicitly designed to have no coupling into the rest of the codebase beyond the three integration points (`mod web_ui;`, `.merge(web_ui::router())`, and the `tera` dependency).

## Open questions

- `TEMPLATES` is loaded from a compile-time path (`env!("CARGO_MANIFEST_DIR")`). In a Docker deployment where the source tree is absent from the final image, templates would not be found at startup. Is template embedding (e.g., `include_str!` or `rust-embed`) planned, or is the source tree expected to be present at runtime?

- The CSRF secret (`CSRF_SECRET`) and session tokens are in-memory only and lost on restart. Is this an accepted trade-off, or is persistent CSRF state (e.g., a DB-backed secret rotation) planned for production?

- The session cookie is flagged `Secure`, so it will not be sent over plain HTTP. Is TLS termination always handled upstream (e.g., by a reverse proxy), or could the flag prevent auth from working in HTTP-only development setups?

- `commit_file_impl` hardcodes the git commit author as `"Web User"` / `"web@openxet.local"`. Is using the authenticated user's identity (name and email) from the DB planned?

- PR merge is implemented as a bare fast-forward ref update — there is no divergence check, no conflict detection, and no merge commit. What is the intended behavior when source and target branches have diverged?

- `get_trending_repos` uses a raw SQL string with an integer-clamped `LIMIT`. The query is not parameterized; while the limit cannot introduce SQL injection, `repo_name` values from the result are passed to templates without further escaping. Is there a plan to migrate this to Sea-ORM query builder?

- `repos.html` exists in the templates directory but no route in `create_router()` renders it. Is this template orphaned, or is a "browse all repos" listing page planned?

- `discussion_event` and `pr_event` DB entities are imported from `crate::db::entities`. It is not visible from this module whether the corresponding migrations exist in `db-layer/migrations`. If they do not, the handlers that insert events would fail at runtime.

- Discussion close/reopen uses a strict username string comparison (`current_user != owner`), which means org members with write permission cannot moderate discussions in org-owned repositories — only the literal owner can. Is this intentional?

- `find_and_render_readme` searches for README files case-insensitively and returns only the first match. If a repository contains both `README.md` and `readme.md`, the first entry in the parsed tree array wins. Is the ordering of tree entries from `parse_tree` stable and consistent with git's canonical sort order?
