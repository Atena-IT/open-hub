# OpenXet Gap Analysis — Decision Matrix

**Date:** 2026-03-31  
**Current issue:** [#58](https://github.com/Atena-IT/open-hub/issues/58)  
**Primary input:** [`synthesis/gap_analysis_execution_order.md`](synthesis/gap_analysis_execution_order.md)

This document accumulates the decision-round outputs for issues #57-#61. Issues #57 and #58 now establish the repository foundation and the transport/history boundary; later issues will extend this file with storage, adapter-layer, and roadmap decisions. The goal is to keep the final direction, rationale, and ordering in one place instead of scattering them across issue comments.

## Decision tracker

| Issue | Decision area | Status | Selected direction | Downstream effect |
| --- | --- | --- | --- | --- |
| [#57](https://github.com/Atena-IT/open-hub/issues/57) | Persistence and object-model foundation | decided | Keep xet-backend's DB-primary repository model as the system of record; do not adopt OpenXet's in-memory-primary git-object foundation | Constrains #58, #59, #60, and the roadmap in #61 |
| [#58](https://github.com/Atena-IT/open-hub/issues/58) | Git transport and refs/history scope | decided | Keep the HF-compatible API plus xet-core CAS as the supported transport boundary; treat stronger refs/history behavior as targeted DB-native follow-up work rather than Git Smart HTTP work | Constrains #59, #60, and the roadmap in #61 |
| [#59](https://github.com/Atena-IT/open-hub/issues/59) | LFS/CAS/storage direction | pending | — | Depends on #57 and #58 |
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

## Issue #58 — Git transport and refs/history direction

### Decision statement

xet-backend should keep the HuggingFace-compatible API plus the xet-core CAS protocol as its supported transport boundary. Native Git Smart HTTP (`info/refs`, `upload-pack`, `receive-pack`, pack files, pkt-line, and SHA-1 object identity) remains out of scope for the current roadmap.

Within that boundary, refs/history improvements should be treated as targeted DB-native follow-up work only when a concrete workflow requires them. Possible follow-up items include branch-pointer advancement, stronger commit identifiers, or explicit non-HEAD snapshot support, but none of those imply a shift to native Git transport.

### Options considered

| Option | Summary | Strengths | Costs / risks | Decision |
| --- | --- | --- | --- | --- |
| A | Keep the HF-compatible API plus DB-backed refs/history as the target surface | Matches the issue-56 synthesis; fits the #57 DB-primary foundation; avoids reopening the persistence choice; aligns with the currently exercised compatibility slice | No native `git clone` / `git fetch` / `git push`; historical snapshots remain limited unless added separately | **Selected** |
| B | Expand DB-backed refs/history fidelity without adopting Git Smart HTTP | Allows future branch advancement, stronger revision semantics, or non-HEAD snapshot retrieval while staying within the selected DB-primary model | Still requires explicit schema/query design; should be triggered by concrete workflows, not assumed as baseline parity | Deferred follow-up within the selected boundary |
| C | Add Git Smart HTTP and full git-native history semantics | Would cover clone/fetch/push, packet-line exchange, pack files, and native git object identity | Reopens the foundation choice from #57; largest implementation blast radius; outside the current HF-compatible target | Rejected for this decision round |

### Rationale

1. **The synthesis already frames Smart HTTP as a boundary decision, not a missing baseline feature.** Issue #56 explicitly places the transport/history decision after the persistence choice and characterizes Git Smart HTTP as outside the current HF-compatible target scope while leaving the formal settlement to #58.

2. **#57 already removed the strongest argument for a git-native transport layer.** Once the repository foundation stays DB-primary, adding Smart HTTP would no longer be an incremental transport enhancement; it would become a cross-cutting architectural expansion.

3. **The currently exercised compatibility slice does not need native git transport.** The analysis already covers the active boundary: HF-style commits, ref listing, file resolution, and xet-core CAS flows. Those are the relevant transport surfaces today.

4. **The remaining refs/history gaps are narrower than a transport rewrite.** Branch-pointer advancement, non-HEAD file retrieval, and stronger commit identity semantics are real questions, but they can be evaluated as DB-native enhancements if a concrete consumer appears.

### Consequences for downstream issues

- **As a constraint from #58, #59 should evaluate LFS/CAS/storage without assuming pack files, git-object storage, or native git upload/download paths.** Storage decisions should start from the selected HF-compatible transport boundary.

- **As a constraint from #58, #60 should document DB-backed history limits explicitly in API/auth/web surfaces.** Features that imply branch semantics, commit history, or non-HEAD file views must state those assumptions directly instead of inheriting git-native expectations.

- **As a constraint from #58, #61 should keep Git Smart HTTP and full git-native history out of the committed phased roadmap unless a later scope change explicitly reopens this decision.**

### Phase ordering unlocked by this decision

1. Continue Tier 0 hardening and any #57-compatible DB-native improvements that have a clear consumer.
2. Treat branch advancement, non-HEAD snapshot support, and content-derived commit identifiers as optional follow-up design items, not transport commitments.
3. Resolve [#59](https://github.com/Atena-IT/open-hub/issues/59) assuming the HF-compatible API plus xet-core CAS remain the only supported transport surfaces.
4. Resolve [#60](https://github.com/Atena-IT/open-hub/issues/60) with explicit DB-backed history semantics and no implicit clone/push support.
5. Let [#61](https://github.com/Atena-IT/open-hub/issues/61) encode this as a roadmap boundary rather than reopening Smart HTTP by default.

### Explicitly not decided here

- Whether DB-native history improvements should be implemented immediately or deferred.
- Whether non-HEAD file retrieval is a real product requirement.
- Whether commit identifiers should become content-derived.
- Whether branch-pointer advancement should be added before web or org features.
- Whether a future scope change should reopen #57 and #58 together.

### Open questions carried forward

- Does any planned workflow require branch advancement or commit-history traversal beyond the current HF-compatible slice?
- Is non-HEAD file resolution a real product requirement, or only a theoretical parity gap?
- If a later scope change demands native git transport, should that reopen #57 and #58 together rather than incrementally?
