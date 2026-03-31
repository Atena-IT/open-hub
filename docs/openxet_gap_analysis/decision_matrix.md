# OpenXet Gap Analysis — Decision Matrix

**Date:** 2026-03-31  
**Current issue:** [#61](https://github.com/Atena-IT/open-hub/issues/61)  
**Primary input:** [`synthesis/gap_analysis_execution_order.md`](synthesis/gap_analysis_execution_order.md)

This document accumulates the decision-round outputs for issues #57-#61. Issues #57-#61 now establish the repository foundation, the transport/history boundary, the storage direction, the adapter-layer scope, and the phased roadmap. The goal is to keep the final direction, rationale, and ordering in one place instead of scattering them across issue comments.

## Decision tracker

| Issue | Decision area | Status | Selected direction | Downstream effect |
| --- | --- | --- | --- | --- |
| [#57](https://github.com/Atena-IT/open-hub/issues/57) | Persistence and object-model foundation | decided | Keep xet-backend's DB-primary repository model as the system of record; do not adopt OpenXet's in-memory-primary git-object foundation | Constrains #58, #59, #60, and the roadmap in #61 |
| [#58](https://github.com/Atena-IT/open-hub/issues/58) | Git transport and refs/history scope | decided | Keep the HF-compatible API plus xet-core CAS as the supported transport boundary; treat stronger refs/history behavior as targeted DB-native follow-up work rather than Git Smart HTTP work | Constrains #59, #60, and the roadmap in #61 |
| [#59](https://github.com/Atena-IT/open-hub/issues/59) | LFS/CAS/storage direction | decided | Keep the concrete `S3Client` and the current split between direct-to-S3 LFS and client-driven CAS; prioritize targeted storage helpers and performance work over StorageBackend abstraction or LFS-CAS convergence | Constrains #60 and the roadmap in #61 |
| [#60](https://github.com/Atena-IT/open-hub/issues/60) | API/auth/web integration direction | decided | Keep API/auth/web as a thin adapter over the selected backend: `ox_*` bearer tokens for API, cookie-backed sessions for web UI, org support in scope, and community features out of scope for the current roadmap | Constrains the roadmap in #61 |
| [#61](https://github.com/Atena-IT/open-hub/issues/61) | Phased implementation roadmap approval | decided | Approve a bounded roadmap: Tier 0 hardening first, then auth/session and range foundations, then minimum operational web/API surfaces; keep git-native, storage-convergence, and community expansions off the committed path | Marks the implementation order after the decision round |

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

## Issue #59 — LFS/CAS/storage direction

Issue #59 resolves the two storage-side decision points surfaced by the synthesis: D1 (storage abstraction) and D2 (LFS-CAS integration).

### Decision statement

xet-backend should keep its concrete `S3Client`-based storage layer and its current split between direct-to-S3 LFS uploads and client-driven CAS/xorb storage. It should **not** introduce an OpenXet-style `StorageBackend` trait or a server-side LFS-to-CAS chunking pipeline as part of the current roadmap.

Within that boundary, storage work should focus on targeted helper methods and DB/query improvements that support the chosen DB-primary and HF-compatible transport model. That includes concrete S3 helpers (`delete`, `get_range`, `list`, and possibly `get_stream`) plus DB-backed performance work such as chunk batch inserts and range-aware reconstruction.

### Options considered

| Option | Summary | Strengths | Costs / risks | Decision |
| --- | --- | --- | --- | --- |
| A | Keep the concrete `S3Client` and the current split between direct-to-S3 LFS and client-driven CAS | Matches the #57 DB-primary foundation and the #58 HF-compatible transport boundary; keeps the storage model simple; focuses work on concrete helpers and measured performance fixes | No backend abstraction today; no automatic LFS/CAS dedup convergence; some helper methods still need to be added explicitly | **Selected** |
| B | Preserve the current architecture but revisit narrow storage abstractions or targeted storage enhancements later | Leaves room for helper-method expansion, `get_stream`, caching, or a future trait if a real second backend or testability pressure appears | Adds design and maintenance cost if introduced too early; should be triggered by concrete needs rather than parity goals | Deferred follow-up within the selected boundary |
| C | Introduce a StorageBackend abstraction and converge LFS into a server-side CAS chunking pipeline | Would move xet-backend closer to OpenXet's storage surface and could enable future LFS dedup integration | Highest complexity; adds worker/lifecycle coordination; not required by the current DB-primary + HF-compatible architecture | Rejected for this decision round |

### Rationale

1. **#57 and #58 already fixed the architectural boundaries that storage must serve.** The repository remains DB-primary, and the transport boundary remains the HF-compatible API plus xet-core CAS. That sharply reduces the case for a more general storage abstraction or git-oriented storage pipeline.

2. **The current deployment model is concrete, not polymorphic.** The synthesis already notes that the concrete `S3Client` approach matches the current Docker Compose and MinIO deployment. There is no accepted second backend that justifies paying the abstraction cost now.

3. **LFS and CAS currently solve different transport paths.** LFS uses direct-to-S3 flows, while CAS assumes client-side chunking and xorb upload. Converging them would add background-worker and lifecycle complexity that the current compatibility boundary does not require.

4. **The highest-value storage gaps are targeted, not architectural.** The concrete missing items already identified in the synthesis — `S3Client::delete`, `get_range`, `list`, chunk batch inserts, and range-aware reconstruction — can be addressed without introducing a new abstraction layer or changing the LFS/CAS split.

### Consequences for downstream issues

- **As a constraint from #59, #60 should assume direct-to-S3 LFS and client-driven CAS remain separate backend flows.** Any API/auth/web behavior built on top of storage should describe those paths explicitly instead of implying a unified chunking pipeline.

- **As a constraint from #59, #61 should phase concrete storage hardening and performance work ahead of any optional abstraction or convergence experiments.** The selected direction makes helper methods and bounded query/storage improvements the near-term path.

### Phase ordering unlocked by this decision

1. The storage-focused Tier 0 and Tier 1 items in `gap_analysis_execution_order.md` (E0.5, E0.6, E0.7, E0.10, and E1.2) are now unblocked and can proceed immediately.
2. Treat `get_stream`, dedup-query caching, and other low-level storage optimizations as targeted follow-up items only when backed by concrete need; for now the potential regular-file OOM concern remains tracked as an open question rather than a Tier 0 commitment.
3. Keep `StorageBackend` abstraction and LFS-CAS convergence off the planned path unless a real second backend or dedup requirement appears.
4. Resolve [#60](https://github.com/Atena-IT/open-hub/issues/60) assuming the selected direct-to-S3 LFS + client-driven CAS split remains in place.
5. Let [#61](https://github.com/Atena-IT/open-hub/issues/61) encode this as a roadmap boundary rather than reopening storage abstraction or convergence by default.

### Explicitly not decided here

- Whether `get_stream` should be promoted into the near-term implementation list.
- Whether a second storage backend will ever enter scope.
- Whether dedup-query caching beyond PostgreSQL indexes is needed.
- Whether GC/audit tooling should be built now or later.
- Whether LFS-CAS convergence should ever be reconsidered if workflow scope changes.

### Open questions carried forward

- Should `get_stream` be elevated because some regular-file reads still load the full S3 object into memory?
- Do dedup-query hot paths need caching beyond PostgreSQL indexes at the expected scale?
- Is there any planned workflow that benefits enough from LFS-CAS dedup to justify reopening the convergence decision?

## Issue #60 — API/auth/web integration direction

Issue #60 resolves the remaining adapter-layer question from the synthesis, including the scope boundary for web/community features.

### Decision statement

xet-backend should keep API/auth/web as a thin adapter layer over the already-selected DB-primary, HF-compatible backend. The API should remain centered on HF-compatible behavior and `ox_*` bearer tokens. The web UI should use separate cookie-backed sessions for interactive pages, with CSRF protection and explicit error handling, rather than a unified multi-mode bearer model.

Within that boundary, organization support is in scope as an access-control and collaboration primitive, but community/product features such as discussions, pull requests, likes, and trending remain out of scope for the current roadmap. The web UI should prioritize utility surfaces — authentication flows, token management, repository browsing, and other bounded operational pages — over platform/community features. Cross-surface error contracts should be standardized as part of the same adapter-layer foundation, advancing alongside session/auth work rather than being deferred behind richer UI features.

### Options considered

| Option | Summary | Strengths | Costs / risks | Decision |
| --- | --- | --- | --- | --- |
| A | Keep API/auth/web as a thin adapter: `ox_*` bearer tokens for API, cookie sessions for web UI, org support in scope, community features out of scope | Matches the #57-#59 backend decisions; keeps auth modes purpose-built; prioritizes the highest-value missing adapter features; resolves D4 without expanding into a full product surface | No unified tri-dispatch auth baseline; community/web platform features are deferred; some richer UI surfaces remain optional follow-up work | **Selected** |
| B | Expand the adapter layer gradually with broader auth modes (including bearer tri-dispatch if justified) and richer utility surfaces while still deferring community features | Leaves room for bearer fallback modes or additional web pages if concrete consumers appear | Still adds auth and UI complexity; should be triggered by identified clients or workflows, not by parity pressure | Deferred follow-up within the selected boundary |
| C | Aim for an OpenXet-like full product/community surface | Would move xet-backend closer to OpenXet's broader web/community surface | Largest scope expansion; weak alignment with the current HF-compatible target; pushes #60 into roadmap/product-planning territory | Rejected for this decision round |

### Rationale

1. **#57-#59 already fixed the backend semantics the adapter layer must expose.** The repository is DB-primary, the supported transport boundary is HF-compatible API plus xet-core CAS, and storage remains concrete-S3 plus separate LFS/CAS flows. The adapter layer should express those decisions clearly, not hide them behind a broader product surface.

2. **API and web have different auth needs.** `ox_*` bearer tokens already match the API/client compatibility surface, while the web UI needs cookie-backed sessions plus CSRF protection. Treating those as separate, purpose-built modes is simpler than adopting a unified tri-dispatch auth scheme without a concrete consumer.

3. **Organization support is infrastructure; community features are product-surface expansion.** Org management directly supports ownership, permissions, and multi-user collaboration on repositories. Discussions, pull requests, likes, and trending do not serve the current compatibility target and should not be smuggled into the roadmap as if they were required parity items.

4. **The highest-value adapter gaps are foundational.** Session auth, CSRF, login/signup submit flows, token management, org management, and consistent cross-surface error contracts all unlock practical use of the existing backend. They should take precedence over richer UI or community work.

### Consequences for the roadmap issue

- **As a constraint from #60, #61 should phase adapter-layer foundation work ahead of richer UI features.** The near-term path is session auth, CSRF, login/signup submit, token management, org support, and consistent error handling.

- **As a constraint from #60, #61 should keep community/product features out of the committed roadmap.** Discussions, pull requests, likes, and trending remain outside the selected scope unless a later scope change deliberately reopens that choice.

- **As a constraint from #60, #61 should treat `ox_*` bearer auth for API and cookie-backed sessions for web UI as the default auth split.** Bearer tri-dispatch stays a deferred follow-up, not a baseline requirement.

### Phase ordering unlocked by this decision

1. The adapter-layer foundation items already identified in `gap_analysis_execution_order.md` — especially session auth (E1.1), CSRF (E2.1), login/signup submit (E2.2), token management UI (E3.1), org management (E3.5), and the cross-surface error-contract hardening from Tier 0 — should be treated as the near-term path.
2. Keep `ox_*` bearer auth for API and cookie-backed sessions for web UI as separate default modes; do not treat bearer tri-dispatch as a prerequisite for the roadmap.
3. Treat richer blob/history/editor pages as follow-up utility features only after the auth/session foundation exists and only within the selected history/storage boundaries.
4. Keep community/product features off the planned path unless scope changes reopen the decision.
5. Let [#61](https://github.com/Atena-IT/open-hub/issues/61) convert these boundaries into the final phased roadmap.

### Explicitly not decided here

- Whether bearer tri-dispatch will ever be needed for a concrete client.
- Which optional utility pages beyond auth/token/org management should ship first.
- Whether blob/history/editor pages are all needed in the initial roadmap.
- Whether community/product features should ever be revisited if project scope changes.
- Whether the web UI should remain primarily a utility/admin surface long-term.

### Open questions carried forward

- Does any planned client actually require username:password bearer fallback or session-token bearer support?
- What is the minimum useful web UI beyond auth flows, token management, org management, and bounded repository views?
- Should cross-surface error-contract cleanup be treated as a prerequisite for session/auth rollout, or can those items advance in parallel?

## Issue #61 — Phased implementation roadmap approval

Issue #61 converts the settled decisions from #57-#60 into the approved implementation sequence for the next execution phase.

### Decision statement

xet-backend should adopt a bounded phased roadmap that starts with independent Tier 0 hardening and concrete storage/DB improvements, then establishes the auth/session and partial-content foundations, then delivers the minimum operational web/API surface selected in #60. After those phases, only bounded utility follow-up work should remain on the table, and only when it fits the already-selected DB-primary, HF-compatible, concrete-S3 architecture.

The committed roadmap should **not** schedule Git Smart HTTP, a git-object migration, StorageBackend abstraction, LFS-CAS convergence, or community/product features such as discussions, pull requests, likes, and trending. Those remain outside the approved path unless a later scope change explicitly reopens the earlier decisions.

### Options considered

| Option | Summary | Strengths | Costs / risks | Decision |
| --- | --- | --- | --- | --- |
| A | Approve a bounded roadmap: Tier 0 hardening first, then auth/session and range foundations, then the minimum operational web/API surface, with later utility work kept conditional | Matches the settled decisions from #57-#60; gets immediate correctness/security/performance wins; sequences work around the strongest known blockers; keeps the roadmap aligned with the current HF-compatible target | Some richer history/UI work remains deferred and must be justified by concrete consumers later | **Selected** |
| B | Approve the same near-term phases but also treat broader DB-native history, bearer tri-dispatch, and more utility surfaces as committed near-term work | Would produce a more ambitious follow-up roadmap without immediately reopening the rejected architectural decisions | Risks overcommitting optional work before the foundation phases land; blurs the line between committed scope and conditional backlog | Deferred follow-up within the selected boundary |
| C | Approve a parity-oriented roadmap that also schedules git-native transport, storage abstraction/LFS-CAS convergence, and community/product features | Would move the roadmap closer to OpenXet's broader surface area | Directly contradicts the selected boundaries from #57-#60; largest blast radius; weak fit for the current project target | Rejected for this decision round |

### Rationale

1. **#57-#60 already removed the major architectural uncertainty.** The repository foundation, transport boundary, storage direction, and adapter-layer scope are now explicit. The roadmap should exploit that clarity by sequencing implementation, not by reopening those choices.

2. **Tier 0 delivers immediate value with the lowest coordination cost.** The hardening, security, concrete S3 helper, token-lifecycle, and DB performance items improve correctness and operational readiness immediately, and most of them can proceed in parallel.

3. **Session/auth work is the practical gate for almost all in-scope web progress.** Once #60 selected cookie-backed sessions plus CSRF-protected interactive pages, the roadmap became structurally dependent on that foundation. Token management, org management, and any useful authenticated web surface should follow it rather than compete with it.

4. **The roadmap should reflect the chosen scope boundaries, not the full comparison backlog.** Git Smart HTTP, git-object migration, StorageBackend abstraction, LFS-CAS convergence, and community/product features were not merely deferred for capacity reasons; they were kept off the selected path by the decision round itself.

### Consequences for implementation

- **As a constraint from #61, implementation should start with the independent Tier 0 batch and parallelize those items where practical.** That is now the approved immediate work queue.

- **As a constraint from #61, the first committed web/auth phase is session/auth/CSRF/login foundation work plus the minimum operational surfaces unlocked by it.** That includes token management and organization support once the prerequisites exist.

- **As a constraint from #61, storage and protocol follow-up work should stay inside the selected concrete-S3 and HF-compatible boundaries.** Range-aware reconstruction stays on the path; storage abstraction and LFS-CAS convergence do not.

- **As a constraint from #61, any later blob/history/editor work must remain explicit about current DB-backed history limits and must not smuggle Git Smart HTTP or git-object assumptions back into scope.**

### Phase ordering approved by this decision

1. **Phase 0 — Immediate hardening and storage/DB groundwork.** Start with the independent Tier 0 items from `gap_analysis_execution_order.md`: error-contract hardening (E0.1, E0.8, E0.9), server/security controls (E0.2, E0.3, E0.4), token expiry and soft-revocation (E0.11), concrete S3 helpers (E0.5, E0.6, E0.7), and chunk batch insert optimization (E0.10). These are the first committed implementation batch and should be parallelized where practical.

2. **Phase 1 — Foundation features unlocked by Phase 0.** Deliver cookie-backed session auth for the web UI (E1.1), range-aware reconstruction (E1.2), and any cross-surface error-contract standardization needed to apply the Tier 0 hardening consistently across the selected API/web split. This phase establishes the auth and partial-content foundations without expanding the transport or storage boundaries.

3. **Phase 2 — Minimum operational web/API surface within the approved scope.** Deliver CSRF protection (E2.1), real login/signup submit flows (E2.2), token management UI (E3.1), and organization management (E3.5). Within this phase, CSRF and login/signup submit should land before the token and organization surfaces that depend on those prerequisites. Although some of these items were placed in later synthesis tiers, #60 makes them part of the minimum useful adapter-layer surface once the session/auth prerequisites exist.

4. **Phase 3 — Bounded utility follow-ups, only after Phases 0-2 are stable and only when a concrete consumer exists.** Candidate items include blob views (E3.3) and commit/history pages (E3.4) that clearly document current DB-backed history limits. Web editing (E3.2), branch creation (E3.8), stronger revision semantics, `get_stream`, dedup-query caching, and bearer tri-dispatch remain conditional follow-up items rather than baseline commitments.

5. **Outside the committed roadmap.** Git Smart HTTP (E4.5), git-object migration, StorageBackend abstraction, LFS-CAS convergence / LFS background chunking (E3.7), and community/product features such as discussions, pull requests, likes, and trending remain out of scope for the approved roadmap unless a later decision round explicitly reopens those boundaries.

### Explicitly not decided here

- The exact internal ordering of independent Phase 0 items.
- Which conditional Phase 3 utility items, if any, should ship first after the committed phases.
- Whether `get_stream` or dedup-query caching should be elevated later based on observed scale or memory pressure.
- Whether DB-native history follow-up work ever becomes necessary for a concrete workflow.
- Whether any future scope change should reopen the excluded git-native, storage-convergence, or community/product work.

### Open questions carried forward

- Is `get_stream` significant enough to pull forward if large regular-file reads become a real operational problem?
- Do expected workflows require non-HEAD views or stronger commit identifiers beyond the currently selected DB-backed model?
- Does any concrete client ever require bearer tri-dispatch rather than the selected `ox_*` API tokens plus cookie-backed web sessions?
- Which bounded repository utility surface is most valuable after the committed phases: blob view, history view, editor flows, or branch management?
