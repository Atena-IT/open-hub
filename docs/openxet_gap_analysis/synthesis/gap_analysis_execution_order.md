# Gap Analysis Execution Order Synthesis

Status: review-ready

## Inputs
- `round2/compare_api.md`
- `round2/compare_git.md`
- `round2/compare_cas.md`
- `round2/compare_db.md`
- `round2/compare_storage.md`
- `round2/compare_web_ui.md`
- `round2/compare_error.md`
- `round2/compare_main.md`

## Cross-module dependency order

1. **Decide the repository/object/history foundation**
   - Primary modules: `git`, `db`
   - Why first: every meaningful Git/LFS/server expansion depends on whether refs, objects, commits, and history become first-class persisted concepts.

2. **Define the storage seam that will support the richer model**
   - Primary modules: `storage`, with `cas` implications
   - Why second: Git-native object storage and richer read paths need a clear storage abstraction and better range semantics, but should build on the chosen data model.

3. **Introduce Git-native transport over the chosen foundation**
   - Primary module: `git`
   - Why third: Smart HTTP, ref advertisement, upload-pack, and receive-pack only make sense after the object/ref/history model exists.

4. **Reconcile Git transport with the existing Xet/CAS/LFS path**
   - Primary modules: `cas`, `storage`, `api`
   - Why fourth: `xet-backend` already has the stronger CAS model, so the job is integration and coexistence, not replacement.

5. **Adapt the existing Hub API over the richer backend semantics**
   - Primary modules: `api`, `error`
   - Why fifth: the current Hub layer is valuable and should be preserved, but it needs the new foundation underneath before it can expose richer history/ref behavior.

6. **Adjust runtime composition and route ordering**
   - Primary module: `main`
   - Why sixth: composition work should be incremental and follow the addition of new transport/storage/state requirements.

7. **Revisit browser/product layers only after the backend model is stable**
   - Primary module: `web_ui`
   - Why last: most Web UI gaps are downstream of missing repo/history/product entities and should not drive the foundation decisions.

## Shared blockers

- No Git-native object model in current `xet-backend`
- No Smart HTTP transport
- Lightweight refs/history semantics compared with OpenXet’s Git-native foundation
- Incomplete range semantics in the current reconstruction/read path
- Web/product gaps that are really persistence/model gaps in disguise

## Recommended epic sequencing

### Epic 1 — repository/object-model expansion
- extend Postgres/sqlx schema with the missing Git-native persistence concepts
- preserve the existing query-layer style
- define how current Hub commits/refs map onto the richer model

### Epic 2 — storage seam and read-path cleanup
- add or formalize a storage abstraction seam around the current S3 layer
- complete range-read behavior needed by future Git/CAS reads
- keep current Xorb/Shard flows intact

### Epic 3 — Git transport foundation
- adapt OpenXet-like protocol/pack/repository structure onto `xet-backend`
- add Git Smart HTTP routes and ref advertisement

### Epic 4 — Git + CAS/LFS integration
- ensure Git-native pushes/reads coexist with current Xet/LFS/CAS behavior
- define how large-file flows bridge Git, Hub, and Xet clients

### Epic 5 — Hub adapter evolution
- preserve existing HF compatibility while exposing richer ref/history semantics where appropriate
- add transport-aware error shaping if Git and HF clients diverge

### Epic 6 — browser/admin follow-on
- only after the backend model is agreed and stable
- limit to the pieces that align with chosen product scope

## Verification areas to design next

- Git smart HTTP clone/fetch/push tests
- historical snapshot/revision tests
- PR/ref workflow tests after the Git-native foundation is designed and approved
- Git LFS CLI interoperability tests
- mixed Git + HF Hub + Xet workflow tests
- migration and backward-compat tests for existing HF behavior
