---
# Decision record
#
# Fill in the front-matter fields before writing the body sections.
# Delete this comment block once the doc is finalized.
area: "<area>"                  # persistence | git_transport | lfs_cas_storage | api_auth_web | roadmap
analyst: "<github-handle>"
date: "<YYYY-MM-DD>"
issue: "#<number>"
status: "draft"                 # draft | in_review | accepted | superseded
depends_on: []                  # list synthesis or earlier decision docs this builds on
---

# Decision — <area>

> **Analyst:** @<github-handle>
> **Date:** <YYYY-MM-DD>
> **Issue:** [#<number>](https://github.com/Atena-IT/open-hub/issues/<number>)

---

## 1. Question

<!-- One sentence: what architectural question does this decision resolve? -->

---

## 2. Options considered

<!-- Two to four options with a brief description of each. -->

### Option A — <name>

...

### Option B — <name>

...

---

## 3. Evaluation criteria

<!-- How were the options assessed? List the criteria used. -->

| Criterion | Weight | Option A | Option B |
| --- | --- | --- | --- |
| | | | |

---

## 4. Decision

<!-- State the chosen option clearly. -->

**Chosen option:** Option A / Option B / ...

**Rationale:**
...

---

## 5. Phase ordering

<!-- Where does this decision fit in the implementation roadmap?
     List the phase and any prerequisite decisions. -->

| Phase | Prerequisite decisions |
| --- | --- |
| | |

---

## 6. Consequences

<!-- What changes, risks, or follow-on tasks does this decision create? -->

---

## 7. Open questions

<!-- Anything remaining unresolved after this decision. -->

- [ ] List any constraints or risks that were not fully resolved by this decision.
- [ ] List any follow-on decisions that this decision makes necessary.
