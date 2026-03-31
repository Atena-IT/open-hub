# API Comparison

## Metadata
- Module: `api`
- OpenXet map: `round1/openxet_api.md`
- Local anchors:
  - `crates/hub-api/src/routes/repos.rs`
  - `crates/hub-api/src/routes/files.rs`
  - `crates/hub-api/src/routes/lfs.rs`
  - `crates/hub-api/src/routes/xet_auth.rs`
  - `crates/hub-api/src/auth.rs`
  - `crates/server/src/main.rs`
  - `docs/feature_support_matrix.md`
  - `docs/hf_hub_testing_roadmap.md`
- Author: Claude
- Status: review-ready

## What OpenXet does
- Exposes a unified API layer over a Git-native foundation.
- Serves Git Smart HTTP, LFS, and HF-compatible routes from the same application state.
- Treats the HF adapter as a translation layer over repos, refs, objects, and CAS/LFS primitives that already exist underneath.
- Uses auth, repo access, and route composition patterns that assume a richer persistence and object model.

## What xet-backend currently does
- Exposes Hub API, LFS, and Xet-auth endpoints over a lightweight Hub-compatible repo model.
- Provides strong HF-specific compatibility in `repos.rs`, `files.rs`, `lfs.rs`, and `xet_auth.rs`.
- Uses `resolve_repo_revision` and repo/file/ref queries to emulate enough behavior for `huggingface_hub`, but not to expose a Git-native transport.
- Merges Hub/LFS/Web UI routes into a clean unified server in `crates/server/src/main.rs`.

## Gap inventory
| Area | OpenXet behavior | Current behavior | Gap type | Notes |
| --- | --- | --- | --- | --- |
| Git transport surface | Smart HTTP is first-class | No Smart HTTP routes | protocol, routing | Biggest adapter-layer difference |
| Repo/history expectations | API assumes Git-native refs/objects underneath | API adapts onto current-state files + lightweight refs | protocol, persistence | Current model is intentionally lightweight |
| HF adapter depth | Thin adapter over Git/CAS foundations | Hub routes do most of the behavior shaping themselves | routing, persistence | xet-backend carries more compatibility logic in the route layer |
| Auth surface | Unified auth manager over Git-native repos | Hub auth tuned for current Hub compatibility | auth | Similar token style, different foundation |
| Product scope | API coexists with broader Git server | API is scoped around Hub compatibility | routing | This is a deliberate repo boundary today |

Gap type uses one or more of: `protocol`, `persistence`, `storage`, `routing`, `auth`, `ui`.

## Recommendation
- Decision: **extend ours**
- Why this is the best next move:
  - `xet-backend` already has a working HF-compatible API surface and a clean unified Axum composition.
  - The biggest gaps are downstream of the missing Git-native foundation, not proof that the current route organization is wrong.
  - The right move is to preserve the existing Hub API adapter and extend it after the repo/object/history model is enriched, rather than adopting OpenXet’s API layer wholesale.
- Dependencies or blockers:
  - Git/object/ref/history decisions in `compare_git.md` and `compare_db.md`
  - CAS/storage decisions in `compare_cas.md` and `compare_storage.md`

## Verification to design later
- HF regression: `uv run --with pytest --with huggingface_hub --with requests pytest tests/integration/hf_hub -q`
- Future Git transport tests should verify Git and HF APIs see the same repo/ref state.
- Add cross-protocol tests for branch creation, revision resolution, and LFS/Xet negotiation once the richer repo model exists.

## Evidence
- OpenXet anchors: `round1/openxet_api.md`, `round1/openxet_git.md`, `round1/openxet_main.md`
- xet-backend anchors:
  - `crates/hub-api/src/routes/repos.rs`
  - `crates/hub-api/src/routes/files.rs`
  - `crates/hub-api/src/routes/lfs.rs`
  - `crates/hub-api/src/routes/xet_auth.rs`
  - `crates/hub-api/src/auth.rs`
  - `crates/server/src/main.rs`
  - `docs/feature_support_matrix.md`
  - `docs/hf_hub_testing_roadmap.md`
