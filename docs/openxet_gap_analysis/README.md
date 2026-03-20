# OpenXet Gap Analysis

This directory contains the discovery workstream that compares **OpenXet** against the current
`xet-backend` (open-hub) implementation, with the goal of choosing the right Git/LFS/Xet-native
architecture direction before any implementation work begins.

See the macro epic in [issue #35](https://github.com/Atena-IT/open-hub/issues/35).

---

## Directory layout

```text
docs/openxet_gap_analysis/
  README.md                          ← this file — index + reference-clone policy
  github_tracking.md                 ← issue/PR ledger and discovery checklist
  decision_matrix.md                 ← final decision record (populated in decision round)
  templates/
    round1_module_map.md             ← template for Round 1 module-map docs
    round2_comparison.md             ← template for Round 2 comparison docs
    synthesis.md                     ← template for synthesis docs
    decision.md                      ← template for decision docs
  round1/                            ← one file per OpenXet module
    openxet_api.md
    openxet_git.md
    openxet_cas.md
    openxet_db.md
    openxet_storage.md
    openxet_web_ui.md
    openxet_error.md
    openxet_main.md
  round2/                            ← one file per comparison area
    compare_api.md
    compare_git.md
    compare_cas.md
    compare_db.md
    compare_storage.md
    compare_web_ui.md
    compare_error.md
    compare_main.md
  synthesis/                         ← cross-module aggregation
    openxet_cross_module_architecture.md
    gap_analysis_execution_order.md
```

---

## Reference-clone policy

All OpenXet source reading is done through a **read-only sibling clone** that lives
**outside** this repository.  This keeps the open-hub working tree clean and avoids
any risk of accidentally mixing upstream OpenXet commits into our history.

### Rules

1. **Never clone OpenXet inside `open-hub/`.** The sibling clone lives alongside the
   open-hub checkout, not inside it, e.g.:

   ```text
   ~/projects/
     open-hub/           ← this repo (open-hub)
     openxet-ref/        ← read-only reference clone of OpenXet
   ```

2. **Clone as read-only (no-push remote).**  Use bare or mirror-less clones and
   immediately set the push URL to `no-push` to prevent accidental pushes:

   ```bash
   git clone https://github.com/<org>/openxet openxet-ref
   cd openxet-ref
   git remote set-url --push origin no-push
   ```

3. **Never commit OpenXet source files into open-hub.** Only analysis documents,
   summaries, and references belong in this directory.

4. **Update the reference clone before each analysis session** to ensure you are
   working against the latest upstream state:

   ```bash
   cd ../openxet-ref && git fetch origin && git checkout main && git pull
   ```

5. **Record the OpenXet commit SHA** used for each analysis doc in its YAML front
   matter (see the templates) so the analysis is reproducible.

6. **`.gitignore` any local scratch files** you create during exploration.  Only
   finalized analysis documents go into version control.

---

## Document lifecycle

| Stage | Input | Output | Issues |
| --- | --- | --- | --- |
| Round 1 | OpenXet reference clone | `round1/openxet_<module>.md` | #39 – #46 |
| Cross-module synthesis 1 | Round 1 docs | `synthesis/openxet_cross_module_architecture.md` | #47 |
| Round 2 | Round 1 docs + synthesis | `round2/compare_<area>.md` | #48 – #55 |
| Cross-module synthesis 2 | Round 2 docs | `synthesis/gap_analysis_execution_order.md` | #56 |
| Decision round | Synthesis docs | `decision_matrix.md` (entries) | #57 – #60 |
| Implementation approval | Decision matrix | `decision_matrix.md` (approved order) | #61 |

---

## Naming rules

- Round 1 files: `round1/openxet_<module>.md` (lowercase, underscores)
- Round 2 files: `round2/compare_<area>.md` (lowercase, underscores)
- Synthesis files: `synthesis/<descriptive_name>.md`
- Decision matrix: `decision_matrix.md` (single file, append entries per decision issue)
- Templates: `templates/<kind>.md` (do not modify after initial creation without updating
  all existing docs that used them)
