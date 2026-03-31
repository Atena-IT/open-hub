# Main Comparison

## Metadata
- Module: `main`
- OpenXet map: `round1/openxet_main.md`
- Local anchors:
  - `crates/server/src/main.rs`
  - `docs/hf_hub_testing_roadmap.md`
  - `docs/feature_support_matrix.md`
- Author: Claude
- Status: review-ready

## What OpenXet does
- Uses a single Tokio composition root to initialize storage, DB, auth/admin bootstrap, and merge Git/LFS/HF/Web UI routes in one process.
- Treats main as a clean adapter/composition seam rather than a domain module.
- Router order is important because protocol surfaces overlap.

## What xet-backend currently does
- Also uses a unified Axum server composition root in `crates/server/src/main.rs`.
- Initializes config, Postgres, S3, shared CAS/Hub state, and merges CAS, Hub, LFS, and Web UI routes.
- Already has a good composition seam for extending the server with additional routes.
- Explicitly scopes current behavior around HF compatibility rather than Git-native transport.

## Gap inventory
| Area | OpenXet behavior | Current behavior | Gap type | Notes |
| --- | --- | --- | --- | --- |
| Route composition | Git/LFS/HF/Web UI in one root | CAS/HF/LFS/Web UI in one root | routing | Structures are already similar |
| Startup dependencies | SQLite + SeaORM + app bootstrap | Postgres + S3 + hub/cas bootstrap | routing, persistence | Different foundations, similar composition role |
| Git-native route integration | Already present | Missing | routing | This is the main future extension point |
| Product boundary | Git-native server with optional UI | HF-compatible server with lightweight repo model | routing | Boundary difference belongs to deeper modules |

Gap type uses one or more of: `protocol`, `persistence`, `storage`, `routing`, `auth`, `ui`.

## Recommendation
- Decision: **extend ours**
- Why this is the best next move:
  - `xet-backend` already has a clean unified composition root and does not need a main-module redesign.
  - The likely future change is adding a Git-native router and any supporting middleware/state once foundation decisions are made.
  - Main should remain an adapter seam, not the place where this epic is solved.
- Dependencies or blockers:
  - Git/API/storage decisions determine what new routers and shared state must be composed here

## Verification to design later
- Startup smoke tests for any richer shared state
- Route ordering tests once Git Smart HTTP is added alongside Hub/LFS/CAS routes
- End-to-end mixed workflow tests through the unified binary

## Evidence
- OpenXet anchors: `round1/openxet_main.md`, `synthesis/openxet_cross_module_architecture.md`
- xet-backend anchors:
  - `crates/server/src/main.rs`
  - `docs/hf_hub_testing_roadmap.md`
  - `docs/feature_support_matrix.md`
