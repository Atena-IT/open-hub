# Round 2 Comparison — `<module-name>`

<!-- Naming rule: save this file as docs/openxet_gap_analysis/round2/compare_<module>.md -->
<!-- Replace every <...> placeholder and remove all HTML comments before marking the issue done. -->

**OpenXet commit:** `<!-- paste the same SHA used in the Round 1 map -->`
**Analyst:** <!-- GitHub handle -->
**Date:** <!-- YYYY-MM-DD -->
**Tracking issue:** [#<!-- number -->](https://github.com/Atena-IT/open-hub/issues/<!-- number -->)
**Round 1 source:** [`round1/openxet_<module>.md`](../round1/openxet_<module>.md)

---

## Scope

This document compares the OpenXet `<module-name>` module (mapped in Round 1) against the corresponding implementation in `xet-backend`. Its goal is to enumerate gaps, alignment, and surfaces that need decision.

## xet-backend counterpart

<!-- Where in xet-backend does this module's functionality live? If there is no counterpart, say so explicitly. -->

| Responsibility | xet-backend location | Notes |
| --- | --- | --- |
| `<!-- responsibility -->` | `<!-- crates/crate-name/src/file.rs -->` | <!-- notes --> |

## Surface comparison

<!-- For each symbol or responsibility in the OpenXet module, classify its status in xet-backend. -->

Status values: `covered` | `partial` | `missing` | `out-of-scope`

| OpenXet symbol / responsibility | Status | xet-backend equivalent | Notes |
| --- | --- | --- | --- |
| `<!-- Name -->` | `<!-- status -->` | `<!-- crates/.../file.rs:line -->` | <!-- notes --> |

## Gaps

<!-- List only items with status `missing` or `partial`. Be specific about what is absent or incomplete. -->

### Missing

- <!-- description -->

### Partial

- <!-- description — what is present and what is absent -->

## Already covered

<!-- Brief confirmation of what is fully aligned, so Round 2 readers know what not to re-examine. -->

- <!-- description -->

## Out-of-scope items

<!-- Surfaces from OpenXet that are explicitly excluded from the current compatibility target. Reference the roadmap or a decision-round item if one exists. -->

- <!-- description -->

## Recommendations for synthesis

<!-- Inputs this comparison contributes to the synthesis round: blocking dependencies, implementation order hints, architectural decisions required. -->

- <!-- recommendation -->

## Open questions

<!-- Unresolved ambiguities that need clarification before implementation can begin. -->

- <!-- question -->
