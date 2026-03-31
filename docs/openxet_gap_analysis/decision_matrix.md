# OpenXet Gap Analysis — Decision Matrix

**Date:** 2026-03-31  
**Current issue:** [#57](https://github.com/Atena-IT/open-hub/issues/57)  
**Primary input:** [`synthesis/gap_analysis_execution_order.md`](synthesis/gap_analysis_execution_order.md)

This document accumulates the decision-round outputs for issues #57-#61. Issue #57 establishes the repository and persistence foundation; later issues will extend this file with transport/history, storage, adapter-layer, and roadmap decisions. The goal is to keep the final direction, rationale, and ordering in one place instead of scattering them across issue comments.

## Decision tracker

| Issue | Decision area | Status | Selected direction | Downstream effect |
| --- | --- | --- | --- | --- |
| [#57](https://github.com/Atena-IT/open-hub/issues/57) | Persistence and object-model foundation | decided | Keep xet-backend's DB-primary repository model as the system of record; do not adopt OpenXet's in-memory-primary git-object foundation | Constrains #58, #59, #60, and the roadmap in #61 |
| [#58](https://github.com/Atena-IT/open-hub/issues/58) | Git transport and refs/history scope | pending | — | Depends on #57 |
| [#59](https://github.com/Atena-IT/open-hub/issues/59) | LFS/CAS/storage direction | pending | — | Depends on #57 and partially on #58 |
| [#60](https://github.com/Atena-IT/open-hub/issues/60) | API/auth/web integration direction | pending | — | Depends on #57-#59 |
| [#61](https://github.com/Atena-IT/open-hub/issues/61) | Phased implementation roadmap approval | pending | — | Depends on #57-#60 |

## Issue #57 — Persistence and object-model direction

### Decision statement

xet-backend should keep its current DB-primary repository model (`repositories`, `repo_files`, `commits`, `repo_refs`, plus S3-backed file content) as the system of record. It should **not** adopt OpenXet's in-memory-primary `RepositoryStore` + git-object model as the new architectural foundation.

If future requirements demand richer history semantics, tree snapshots, or diff inputs, this decision prefers extending the current PostgreSQL/S3 model directly rather than introducing a second, parallel git-object store.

### Options considered

| Option | Summary | Strengths | Costs / risks | Decision |
| --- | --- | --- | --- | --- |
| A | Preserve the existing DB-primary flat-file repository model | Aligned with the current HF-compatible API target; minimal migration cost; preserves synchronous write semantics; fits the existing `repo_files`/`commits`/`repo_refs` model | No native git object model; no historical tree snapshots today; commit identifiers remain synthetic | **Selected** |
| B | Keep DB-primary storage but add DB-native snapshot/versioning support later | Allows historical file views and richer diff inputs without rewriting the whole backend around git objects | Adds schema and query complexity; still requires explicit snapshot design; does not by itself provide Git Smart HTTP | Deferred follow-up, not a foundation change |
| C | Re-found the backend on OpenXet-style git objects plus in-memory-primary stores | Would align more closely with Git Smart HTTP, object-addressed trees, and native git semantics | Largest migration blast radius; conflicts with the current repo boundary; reintroduces the async write-through / state-divergence risks identified in the comparisons | Rejected for this decision round |

### Rationale

1. **The current product boundary does not require a git-native foundation.** The issue-56 synthesis and the Round 2 comparisons converge on the same boundary: xet-backend currently targets the HF-compatible API plus the xet-core CAS protocol, not native `git clone` / `git push` flows. Replacing the persistence foundation now would optimize for a scope that the project has not yet accepted.

2. **DB-primary writes are a structural advantage, not a temporary compromise.** The comparisons repeatedly identified OpenXet's fire-and-forget persistence model as carrying write-loss, async write-through state-divergence, and orphaned-object risks. xet-backend's synchronous PostgreSQL writes and direct DB lookups avoid those failure modes. Replacing that foundation would discard one of the clearest architectural advantages surfaced by the analysis.

3. **A git-object migration would couple too many modules at once.** The git, db, web_ui, and main comparisons are directly tied to this choice in the synthesis, and the api layer is indirectly affected through later Git- and history-scoped work. Switching foundations before #58-#60 are resolved would turn every later decision into a moving target.

4. **The known gaps do not justify a foundation rewrite by themselves.** The lack of historical tree snapshots, merge-parent support, and content-derived commit identifiers are real gaps, but they are narrower than a full git-object migration. They can be revisited as scoped follow-up design questions once #58 clarifies the transport/history target.

### Consequences for downstream issues

- **As a constraint from #57, #58 should treat Git Smart HTTP, SHA-1 object identities, pack files, pkt-line, and full commit-graph semantics as explicit scope expansions rather than assumed next steps.** With #57 decided, those items are no longer baseline parity obligations.

- **As a constraint from #57, #59 should evaluate storage changes under the assumption that PostgreSQL + S3 remain the primary persistence model.** Any StorageBackend trait, LFS/CAS convergence, or new helper methods should be justified within that model, not as stepping stones toward a git-object store.

- **As a constraint from #57, #60 should design API/auth/web behavior around DB-backed repository semantics.** Any future blob view, commit page, editor, or organization flow should clearly state when it relies on HEAD-only data versus a true historical snapshot.

- **As a constraint from #57, #61 should phase independent Tier 0 hardening ahead of any optional history expansion.** This decision leaves the immediate hardening work valid and unblocked.

### Phase ordering unlocked by this decision

1. Proceed immediately with the independent Tier 0 hardening items from `gap_analysis_execution_order.md`.
2. Resolve [#58](https://github.com/Atena-IT/open-hub/issues/58) within the selected DB-primary boundary.
3. Resolve [#59](https://github.com/Atena-IT/open-hub/issues/59) using the #57 foundation and the #58 history/transport decision.
4. Resolve [#60](https://github.com/Atena-IT/open-hub/issues/60) over the now-explicit backend semantics.
5. Convert the resulting decisions into the final phased roadmap in [#61](https://github.com/Atena-IT/open-hub/issues/61).

### Explicitly not decided here

- Whether historical file snapshots should be added to the DB model.
- Whether Git Smart HTTP should ever enter scope.
- Whether LFS and CAS should remain separate or converge.
- Whether community/web-ui features beyond HF compatibility are in scope.
- Whether a storage trait abstraction is worth introducing.

### Open questions carried forward

- Do any target workflows require historical file contents at non-HEAD revisions, or is HEAD-only behavior acceptable for the intended compatibility scope?
- Does any downstream consumer require stable, content-derived commit identifiers rather than the current synthetic SHA-style identifiers?
- If richer history becomes necessary later, is DB-native snapshot/versioning support sufficient, or would that requirement justify reopening the git-object decision?
