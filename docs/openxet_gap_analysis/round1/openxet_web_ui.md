# OpenXet Web UI Module Map

## Metadata
- Module: `web_ui`
- Source paths:
  - `../OpenXet-reference/src/web_ui/mod.rs`
  - `../OpenXet-reference/src/web_ui/routes.rs`
  - `../OpenXet-reference/src/web_ui/routes/`
  - `../OpenXet-reference/src/web_ui/templates.rs`
  - `../OpenXet-reference/src/web_ui/templates/`
- Author: Claude
- Status: review-ready

## Build shape
- `mod.rs` exposes the Web UI router and explicitly documents that the UI is removable without changing the core server.
- `routes.rs` wires a GitHub-like route tree.
- `routes/` is feature-sliced into auth, repo browsing, editing, discussions, PRs, org management, tokens, likes, trending, diffs, tree operations, and shared utils.
- `templates.rs` initializes Tera and provides Markdown rendering/sanitization.
- `templates/` contains a large server-rendered page set plus reusable components.

## Responsibilities
- Provide a browser-facing product layer over the same Git, auth, DB, and CAS foundations used by the API surface.
- Render repository browsing, file viewing, editing, discussions, PRs, org management, and token management via Tera templates.
- Enforce browser-oriented auth, CSRF, and permission checks around form workflows.
- Adapt repo, discussion, and PR data into page-level UX instead of protocol responses.

## Explicit non-goals
- No SPA/client-side state architecture; rendering is server-side and request/response driven.
- No real-time collaboration or notification system.
- No deep platform features like CI, full repo settings, or advanced review workflows.
- No requirement to exist at all: the module is treated as removable adapter functionality rather than a foundation.

## Key contracts
### Types / entities
- Shared route utilities own auth extraction, CSRF generation/verification, permission helpers, and rendering helpers.
- Feature handlers rely on DB-backed discussion/PR/token/org entities plus repo/Git access.
- Templates define the browser contract for navigation, forms, timelines, and repo views.

### Routes / protocol surfaces
- System routes under `/-/` cover login, signup, repo/org creation, search, stats, trending, and token management.
- User/org routes expose profile and membership management.
- Repo routes cover tree/blob/commit browsing, file editing, discussions, PRs, branch actions, and likes.

### Storage / persistence contracts
- The Web UI itself is not a storage layer, but it depends heavily on the DB schema and Git/CAS layers for page data and mutations.
- Markdown content is sanitized before rendering.
- Form workflows assume DB and repo mutation paths are available synchronously enough for browser redirects.

## Integration points
- Upstream: merged by `main.rs` after the API routes.
- Downstream: depends on `api` state/auth, `git` repo access, `db` entities, and selected CAS stats.
- Adjacent: serves as a product-layer adapter that exercises much more of the domain model than the pure protocol surfaces do.

## Runtime and configuration assumptions
- Requires Tera templates to be available at runtime.
- Uses cookie/session-style auth flows and CSRF protection for form submissions.
- Assumes the DB-backed product entities for discussions, PRs, likes, tokens, and org membership exist and are populated.
- Assumes a single-server request/response model with server-rendered HTML.

## Questions for comparison round
- Which OpenXet UI features represent core Git-native product requirements versus optional browser-layer functionality for `xet-backend`?
- How far can `xet-backend`’s current placeholder Web UI evolve on top of the existing Hub model before the repo/history foundation becomes the blocker?
- Which Web UI areas should be deferred until after the data-model and Git-transport decisions are made?
- Are there any handler/template patterns worth adapting now even if the broader UI scope remains out of phase?

## Evidence
- `../OpenXet-reference/src/web_ui/mod.rs`
- `../OpenXet-reference/src/web_ui/routes.rs`
- `../OpenXet-reference/src/web_ui/routes/utils.rs`
- `../OpenXet-reference/src/web_ui/routes/repo_handlers.rs`
- `../OpenXet-reference/src/web_ui/routes/edit_handlers.rs`
- `../OpenXet-reference/src/web_ui/routes/discussion_handlers.rs`
- `../OpenXet-reference/src/web_ui/routes/pr_handlers.rs`
- `../OpenXet-reference/src/web_ui/templates.rs`
- `../OpenXet-reference/src/web_ui/templates/base.html`
