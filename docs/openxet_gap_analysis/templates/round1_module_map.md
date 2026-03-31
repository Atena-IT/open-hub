# Round 1 Module Map — `<module-name>`

<!-- Naming rule: save this file as docs/openxet_gap_analysis/round1/openxet_<module>.md -->
<!-- Replace every <...> placeholder and remove all HTML comments before marking the issue done. -->

**OpenXet commit:** `<!-- paste the exact SHA from git -C ../openxet rev-parse HEAD -->`
**Analyst:** <!-- GitHub handle -->
**Date:** <!-- YYYY-MM-DD -->
**Tracking issue:** [#<!-- number -->](https://github.com/Atena-IT/open-hub/issues/<!-- number -->)

---

## Scope

This document maps the OpenXet `<module-name>` module. It covers only the upstream OpenXet source — no `xet-backend` references appear here. Round 2 will introduce the comparison.

## Source location

```
../openxet/<path-to-module>/
```

Key entry points:
- `<!-- file.rs or mod.rs -->`

## Public surface

<!-- List every publicly exported type, trait, function, or constant. One item per row. -->

| Symbol | Kind | File | Brief description |
| --- | --- | --- | --- |
| `<!-- Name -->` | `<!-- struct / enum / trait / fn / const -->` | `<!-- src/file.rs -->` | <!-- one-line description --> |

## Internal structure

<!-- Summarise the internal organisation: sub-modules, major structs, significant private helpers. -->

### Sub-modules

| Sub-module | Purpose |
| --- | --- |
| `<!-- name -->` | <!-- description --> |

### Key types

<!-- Describe the most important internal types, their fields, and their role. Use a subsection per type if needed. -->

#### `<!-- TypeName -->`

<!-- Role and notable fields. -->

## Intra-module dependencies

<!-- Which other OpenXet modules does this module import from? List only direct imports. -->

| Dependency module | What is used |
| --- | --- |
| `<!-- module -->` | `<!-- symbol(s) -->` |

## Notable implementation details

<!-- Anything non-obvious: algorithms, protocol specifics, error handling patterns, compile-time config, unsafe blocks, etc. Keep snippets short — quote only what is necessary to understand the design. -->

## Open questions

<!-- Questions that arose during mapping and could not be resolved from the source alone. These feed into Round 2 or synthesis. -->

- <!-- question -->
