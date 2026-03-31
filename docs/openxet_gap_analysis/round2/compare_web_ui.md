# Web UI Comparison

## Metadata
- Module: `web_ui`
- OpenXet map: `round1/openxet_web_ui.md`
- Local anchors:
  - `crates/web-ui/src/lib.rs`
  - `crates/web-ui/src/pages.rs`
  - `templates/`
  - `docs/feature_support_matrix.md`
- Author: Claude
- Status: review-ready

## What OpenXet does
- Ships a broad GitHub-like server-rendered product layer with auth flows, repo browsing, editing, discussions, PRs, org management, token management, and product discovery pages.
- Assumes a richer DB and Git model underneath and exercises that model through browser workflows.
- Treats the Web UI as removable, but still much more feature-complete than a placeholder browser layer.

## What xet-backend currently does
- Provides a small Tera-based browser layer for home, login/signup stubs, user pages, repo detail, and tree browsing.
- Uses `HubState` and the existing Hub DB schema to render simple pages.
- Leaves login/signup/new-repo submit paths as stubs and does not implement PR/discussion/token-management/browser-write workflows.
- Is deliberately much lighter and aligned with current HF-compatibility scope.

## Gap inventory
| Area | OpenXet behavior | Current behavior | Gap type | Notes |
| --- | --- | --- | --- | --- |
| Auth/browser workflows | Full browser auth + CSRF + sessions | Mostly stubbed form flows | ui, auth | Achievable incrementally |
| Repo UX | Rich repo/history/edit views | Read-only-ish repo/tree pages | ui | Blocked partly by Git/history depth |
| Product entities | Discussions/PRs/org/token pages | Not modeled or not implemented | ui, persistence | Depends on broader data-model decisions |
| Scope | Full product layer | Minimal companion UI | ui | Current scope difference is intentional |

Gap type uses one or more of: `protocol`, `persistence`, `storage`, `routing`, `auth`, `ui`.

## Recommendation
- Decision: **keep current**
- Why this is the best next move:
  - The Web UI is not the right place to start this epic; it depends on decisions in Git, DB, and API foundations.
  - `xet-backend` should preserve its minimal UI until the richer repo/history model is settled.
  - After foundation work, selective OpenXet-inspired browser workflows can be adapted incrementally where they map cleanly onto the chosen backend model.
- Dependencies or blockers:
  - Git/history decisions in `compare_git.md`
  - Product/entity scope decisions in `compare_db.md`
  - API/auth/browser session decisions in `compare_api.md`

## Verification to design later
- Basic browser flow tests for login/signup/new-repo once forms are made real
- Repo browsing/history UI tests after Git-native history exists
- Token-management UI tests if browser auth is expanded

## Evidence
- OpenXet anchors: `round1/openxet_web_ui.md`, `round1/openxet_db.md`, `round1/openxet_git.md`
- xet-backend anchors:
  - `crates/web-ui/src/lib.rs`
  - `crates/web-ui/src/pages.rs`
  - `templates/`
  - `docs/feature_support_matrix.md`
