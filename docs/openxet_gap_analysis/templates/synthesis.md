# Synthesis — `<document-title>`

<!-- Naming rule: save this file as docs/openxet_gap_analysis/synthesis/<issue-defined-name>.md -->
<!--   Current epic examples: synthesis/openxet_cross_module_architecture.md -->
<!--                          synthesis/gap_analysis_execution_order.md      -->
<!-- Replace every <...> placeholder and remove all HTML comments before marking the issue done. -->

**Analyst:** <!-- GitHub handle -->
**Date:** <!-- YYYY-MM-DD -->
**Tracking issue:** [#<!-- number -->](https://github.com/Atena-IT/open-hub/issues/<!-- number -->)
**Input documents:**
<!-- List every Round 1 map or Round 2 comparison this synthesis draws from. -->
- [`round1/<module-name>_map.md`](../round1/<module-name>_map.md)

---

## Purpose

<!-- One paragraph describing what question this synthesis document answers and what decision-round artifact it feeds into. -->

## Methodology

<!-- How were the input documents aggregated? What criteria were used to identify dependencies, gaps, or order constraints? -->

## Findings

<!-- The main content. Use sub-sections as needed. Two common structures are shown below — use whichever fits, or define your own. -->

### Cross-module dependency graph

<!-- Prose or ASCII diagram showing how OpenXet modules depend on each other. -->

```
<!-- module-a --> module-b --> module-c
              \-> module-d
```

| Module | Depends on | Used by |
| --- | --- | --- |
| `<!-- module -->` | `<!-- deps -->` | `<!-- consumers -->` |

### Gap summary by module

<!-- Aggregated view of gap status across all compared modules. -->

| Module | Round 1 done | Round 2 done | Gap severity | Notes |
| --- | --- | --- | --- | --- |
| `<!-- module -->` | <!-- yes / no --> | <!-- yes / no --> | `<!-- high / medium / low / none -->` | <!-- notes --> |

### Implementation order recommendation

<!-- If this synthesis is about execution order, list modules or work items in the recommended order. Include the rationale for each ordering constraint. -->

1. `<!-- item -->` — <!-- reason -->

## Inputs to the decision round

<!-- What architectural or product decisions does this synthesis surface? Each item should be actionable by the decision-round issues. -->

- <!-- decision required -->

## Open questions

<!-- Anything that could not be resolved from the available Round 1 or Round 2 material. -->

- <!-- question -->
