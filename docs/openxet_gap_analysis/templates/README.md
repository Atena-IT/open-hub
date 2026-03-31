# OpenXet Gap Analysis — Templates and Naming Rules

This directory contains reusable document templates for the OpenXet gap analysis workstream (epic [#35](https://github.com/Atena-IT/open-hub/issues/35)).

## Templates

| File | Purpose | Target directory |
| --- | --- | --- |
| [`round1_module_map.md`](round1_module_map.md) | Per-module OpenXet source map | `docs/openxet_gap_analysis/round1/` |
| [`round2_comparison.md`](round2_comparison.md) | xet-backend vs OpenXet per-module comparison | `docs/openxet_gap_analysis/round2/` |
| [`synthesis.md`](synthesis.md) | Cross-module architecture or gap synthesis | `docs/openxet_gap_analysis/synthesis/` |

## Naming rules

### File names

- Use the exact lower-case snake_case file names defined by the owning issues for this epic.
- Round 1 module maps: `round1/openxet_<module>.md` — e.g., `round1/openxet_api.md`, `round1/openxet_web_ui.md`.
- Round 2 comparisons: `round2/compare_<module>.md` — e.g., `round2/compare_api.md`, `round2/compare_web_ui.md`.
- Synthesis documents: use the exact issue-defined names — currently `synthesis/openxet_cross_module_architecture.md` and `synthesis/gap_analysis_execution_order.md`.

### Round-by-round scope

- **Round 1** (`round1/`) — OpenXet source only. Each document maps a single OpenXet module: what it contains, what it exports, and its internal dependencies. Does not reference `xet-backend`.
- **Round 2** (`round2/`) — One document per module comparing the OpenXet module map (Round 1) against the corresponding `xet-backend` implementation. Identifies gaps, mismatches, and already-covered surfaces.
- **Synthesis** (`synthesis/`) — Cross-module documents that aggregate findings from Round 1 or Round 2 into dependency graphs, implementation-order recommendations, or architectural decision inputs.

### OpenXet commit SHA

Every Round 1 document must record the exact OpenXet commit SHA used so results are reproducible (see the reference-clone policy in `docs/openxet_gap_analysis/README.md`).

## How to use a template

1. Copy the relevant template file into the appropriate target directory.
2. Rename it to match the naming rules above.
3. Fill in all `<!-- ... -->` placeholder sections.
4. Remove all remaining HTML comments once the document is complete.
5. Update `docs/openxet_gap_analysis/github_tracking.md` as the owning issue moves through `planned`, `in_progress`, and `done`.
