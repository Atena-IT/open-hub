# OpenXet Gap Analysis Workstream

This docs area tracks the discovery-first epic for expanding `xet-backend` from lightweight `huggingface_hub` compatibility into a fuller Git/LFS/Xet-compatible server model.

## Scope and boundaries

- This workstream is separate from the HF Hub compatibility batches in `docs/hf_hub_testing_roadmap.md`.
- Discovery artifacts live here and are the canonical in-repo review surface for the long-lived draft PR.
- No server implementation should begin until Round 1 maps, Round 2 comparisons, cross-module synthesis, and the decision matrix are complete and reviewed.
- The OpenXet checkout is reference-only and should remain outside this repository's tracked tree.

## Local reference checkouts

- Current worktree: `xet-backend/.claude/worktrees/openxet-git-server-gap-analysis`
- Reference checkout: sibling repo at `../OpenXet-reference`

## Directory layout

- `templates/module_template.md` — required structure for Round 1 OpenXet module maps
- `templates/comparison_template.md` — required structure for Round 2 comparisons
- `round1/openxet_<module>.md` — first-pass maps for each OpenXet module
- `synthesis/openxet_cross_module_architecture.md` — normalized terminology and dependency graph
- `round2/compare_<module>.md` — per-module comparison against local xet-backend surfaces
- `synthesis/gap_analysis_execution_order.md` — dependency-aware synthesis of cross-module gaps
- `decision_matrix.md` — final per-area recommendation and phased implementation order

## Module naming rules

Use the following module names consistently across documents and GitHub issues:

- `api`
- `git`
- `cas`
- `db`
- `storage`
- `web_ui`
- `error`
- `main`

## Required workflow

### Round 1 — OpenXet module maps
Each module document must answer:
- how the module is built
- what responsibilities it owns
- what it explicitly does not do
- key types, routes, and storage contracts
- integration points with adjacent modules
- runtime and config assumptions
- questions to carry into Round 2

### Round 1.5 — cross-module synthesis
The synthesis doc must:
- normalize terminology across the Round 1 docs
- build a dependency graph
- identify foundational vs adapter modules
- flag weak or incomplete module docs before comparison starts

### Round 2 — xet-backend comparisons
Each comparison document must answer:
- what OpenXet does in that area
- what `xet-backend` currently does
- where behavior is missing, stubbed, or intentionally lightweight
- whether the gap is mainly protocol, persistence, storage, routing, auth, or UI
- whether the best next move is **extend ours**, **adapt/adopt theirs**, or **keep current**
- what tests should prove the area later

### Decision round
`decision_matrix.md` must apply a consistent rubric across modules:
- protocol correctness
- fit with the current Axum/sqlx/S3 workspace
- migration complexity
- backward compatibility with existing HF Hub behavior
- security and auth impact
- operational complexity
- testability and maintainability

## Status checklist

- [x] Round 1 module docs complete
- [x] Cross-module architecture synthesis complete
- [x] Round 2 comparison docs complete
- [x] Gap/dependency-order synthesis complete
- [x] Decision matrix complete
- [ ] Implementation order approved
