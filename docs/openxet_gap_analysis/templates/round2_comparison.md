---
# Round 2 comparison — <area>
#
# Fill in the front-matter fields before writing the body sections.
# Delete this comment block once the doc is finalized.
area: "<area>"                  # e.g. api, git, cas, db, storage, web_ui, error, main
openxet_commit: "<sha>"         # git rev-parse HEAD of the openxet-ref clone used
analyst: "<github-handle>"
date: "<YYYY-MM-DD>"
issue: "#<number>"
status: "draft"                 # draft | in_review | accepted
depends_on:
  - "docs/openxet_gap_analysis/synthesis/openxet_cross_module_architecture.md"
---

# Round 2 comparison — `<area>`

> **OpenXet commit:** `<sha>`
> **Analyst:** @<github-handle>
> **Date:** <YYYY-MM-DD>
> **Issue:** [#<number>](https://github.com/Atena-IT/open-hub/issues/<number>)

---

## 1. OpenXet behavior (summary)

<!-- One paragraph or short bullet list: what does OpenXet do in this area?
     Reference the Round 1 doc for the full map. -->

---

## 2. xet-backend (open-hub) current state

<!-- Describe what open-hub currently implements in this area.
     Point to specific crates/files/routes where relevant. -->

---

## 3. Gap analysis

<!-- Table or structured list of gaps between OpenXet and open-hub. -->

| Feature / behavior | OpenXet | xet-backend | Gap severity |
| --- | --- | --- | --- |
| | | | |

**Gap severity scale:** `none` / `minor` / `moderate` / `major` / `blocker`

---

## 4. Dependency implications

<!-- Does closing any gap here unblock or require changes in another area?
     List cross-area dependencies discovered during this comparison. -->

---

## 5. Opportunities

<!-- Features or patterns in OpenXet that could improve xet-backend even if they
     are not strictly required for compatibility. -->

---

## 6. Risks

<!-- Anything in OpenXet that conflicts with xet-backend's current architecture
     or that would require invasive changes. -->

---

## 7. Recommendation for decision round

<!-- One-sentence forward pointer: what question should the decision round answer
     for this area? -->

---

## 8. Open questions

<!-- Anything unclear or requiring further investigation. -->

- [ ] List any gaps whose severity could not be confirmed without running both systems.
- [ ] List any behaviors that require clarification from the OpenXet authors or maintainers.
