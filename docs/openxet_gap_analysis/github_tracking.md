# GitHub Tracking Plan for the OpenXet Gap-Analysis Epic

This file captures the GitHub issue set and draft PR structure for the discovery-first OpenXet workstream.

Current state:
- macro issue and child issues have been created remotely
- the long-lived draft PR body below is ready, but the PR itself has not yet been opened from this worktree

## Macro issue

### Proposed title
`Epic: OpenXet gap analysis for Git/LFS/Xet-native server expansion`

### Proposed body
```md
## Summary

Expand `xet-backend` beyond lightweight `huggingface_hub` compatibility by running a discovery-first comparison against OpenXet and using that analysis to choose the right Git/LFS/Xet-native architecture direction.

## Why this is separate from the HF Hub roadmap

The current HF Hub roadmap is intentionally batch-oriented and scoped around lightweight compatibility over the existing current-state repository model. This epic tracks the broader Git-native foundation work that may reshape repository persistence, refs/history semantics, Git transport, and storage integration.

## Discovery gates before implementation

- [x] Round 1 module maps complete under `docs/openxet_gap_analysis/round1/`
- [x] Cross-module architecture synthesis complete
- [x] Round 2 comparison docs complete under `docs/openxet_gap_analysis/round2/`
- [x] Gap/dependency-order synthesis complete
- [x] Decision matrix complete
- [ ] Implementation order approved

## Child issues

### Setup / governance
- [x] Bootstrap worktree + reference clone policy
- [ ] Open draft PR and tracking checklist
- [x] Define document templates and naming rules

### Round 1: OpenXet maps
- [x] Map OpenXet `api`
- [x] Map OpenXet `git`
- [x] Map OpenXet `cas`
- [x] Map OpenXet `db`
- [x] Map OpenXet `storage`
- [x] Map OpenXet `web_ui`
- [x] Map OpenXet `error`
- [x] Map OpenXet `main`
- [x] Synthesize cross-module dependency graph

### Round 2: comparisons
- [x] Compare `api`
- [x] Compare `git`
- [x] Compare `cas`
- [x] Compare `db`
- [x] Compare `storage`
- [x] Compare `web_ui`
- [x] Compare `error`
- [x] Compare `main`
- [x] Synthesize cross-module gap/dependency order

### Decision round
- [x] Decide persistence/object-model direction
- [x] Decide Git transport/ref/history direction
- [x] Decide LFS/CAS/storage direction
- [x] Decide API/auth/web integration direction
- [ ] Approve phased implementation roadmap

## Local reference artifacts
- `docs/openxet_gap_analysis/README.md`
- `docs/openxet_gap_analysis/synthesis/openxet_cross_module_architecture.md`
- `docs/openxet_gap_analysis/synthesis/gap_analysis_execution_order.md`
- `docs/openxet_gap_analysis/decision_matrix.md`
```

## Child issue set

### Setup / governance
1. `docs(openxet): bootstrap gap-analysis docs and reference-clone policy`
2. `docs(openxet): open draft PR ledger and tracking checklist`
3. `docs(openxet): define analysis templates and naming rules`

### Round 1: module maps
4. `docs(openxet): map api module`
5. `docs(openxet): map git module`
6. `docs(openxet): map cas module`
7. `docs(openxet): map db module`
8. `docs(openxet): map storage module`
9. `docs(openxet): map web_ui module`
10. `docs(openxet): map error module`
11. `docs(openxet): map main module`
12. `docs(openxet): synthesize cross-module architecture`

### Round 2: comparisons
13. `docs(openxet): compare api surfaces`
14. `docs(openxet): compare git foundations`
15. `docs(openxet): compare cas flows`
16. `docs(openxet): compare db and persistence model`
17. `docs(openxet): compare storage contracts`
18. `docs(openxet): compare web_ui surfaces`
19. `docs(openxet): compare error handling`
20. `docs(openxet): compare main/runtime composition`
21. `docs(openxet): synthesize gap dependency order`

### Decision round
22. `design(openxet): choose persistence and object-model direction`
23. `design(openxet): choose Git transport and refs/history direction`
24. `design(openxet): choose LFS/CAS/storage direction`
25. `design(openxet): choose API/auth/web integration direction`
26. `design(openxet): approve phased implementation roadmap`

## Draft PR

### Proposed title
`Draft: OpenXet Git/LFS/Xet-native gap analysis`

### Proposed body
```md
## Summary
- establish a dedicated in-repo review surface for the OpenXet discovery-first epic
- track Round 1 module maps, Round 2 comparisons, and the decision matrix in one long-lived draft PR
- keep broader Git-native expansion separate from the lightweight HF Hub batch tracker

## Ledger
- [x] Round 1 module maps complete
- [x] Cross-module architecture synthesis complete
- [x] Round 2 comparisons complete
- [x] Gap/dependency-order synthesis complete
- [x] Decision matrix complete
- [ ] Implementation order approved

## Key docs
- `docs/openxet_gap_analysis/README.md`
- `docs/openxet_gap_analysis/synthesis/openxet_cross_module_architecture.md`
- `docs/openxet_gap_analysis/synthesis/gap_analysis_execution_order.md`
- `docs/openxet_gap_analysis/decision_matrix.md`

## Test plan
- [ ] Discovery docs reviewed for completeness
- [ ] `cargo fmt --all`
- [ ] `cargo test --workspace`
- [ ] `uv run --with pytest --with huggingface_hub --with requests pytest tests/integration/hf_hub -q`

🤖 Generated with [Claude Code](https://claude.com/claude-code)
```
