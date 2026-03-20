---
# Synthesis document
#
# Fill in the front-matter fields before writing the body sections.
# Delete this comment block once the doc is finalized.
kind: "synthesis"               # cross_module_architecture | gap_execution_order
analyst: "<github-handle>"
date: "<YYYY-MM-DD>"
issue: "#<number>"
status: "draft"                 # draft | in_review | accepted
depends_on: []                  # list the Round 1 or Round 2 docs this synthesizes
---

# Synthesis — <title>

> **Analyst:** @<github-handle>
> **Date:** <YYYY-MM-DD>
> **Issue:** [#<number>](https://github.com/Atena-IT/open-hub/issues/<number>)

---

## 1. Inputs

<!-- List the documents that were synthesized to produce this artifact. -->

| Document | Issue |
| --- | --- |
| | |

---

## 2. Terminology normalization

<!-- If this is the cross-module architecture synthesis, align on shared names,
     concepts, and module boundaries here before writing the synthesis body.
     Skip or abbreviate this section for the execution-order synthesis. -->

---

## 3. Dependency graph

<!-- ASCII or Mermaid diagram showing the dependency relationships between
     modules / areas / decisions discovered during analysis. -->

```
<module A>
  └─ depends on <module B>
       └─ depends on <module C>
```

---

## 4. Key findings

<!-- Numbered list of the most important cross-cutting observations. -->

1. ...

---

## 5. Recommended sequencing

<!-- In dependency order, what should be analysed / implemented / decided first?
     Explain the rationale briefly for each ordering choice. -->

1. **<first>** — reason
2. **<second>** — reason

---

## 6. Open questions

<!-- Anything that remains unresolved after synthesizing the inputs. -->

- [ ] List any cross-module interactions that could not be confirmed from the input documents alone.
- [ ] List any sequencing assumptions that should be validated before the decision round.
