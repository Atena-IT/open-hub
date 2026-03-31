# DB Comparison

## Metadata
- Module: `db`
- OpenXet map: `round1/openxet_db.md`
- Local anchors:
  - `crates/db-layer/migrations/002_hub_schema.sql`
  - `crates/db-layer/migrations/003_repo_refs.sql`
  - `crates/db-layer/src/queries/commits.rs`
  - `crates/db-layer/src/queries/repo_files.rs`
  - `crates/db-layer/src/queries/repo_refs.rs`
  - `crates/db-layer/src/queries/repositories.rs`
- Author: Claude
- Status: review-ready

## What OpenXet does
- Uses a Git- and product-native DB model with first-class entities for repos, refs, Git objects, CAS/LFS metadata, tokens, org membership, discussions, and PRs.
- Couples the application closely to SeaORM entities and inline schema creation.
- Encodes richer repository semantics directly into persistence.

## What xet-backend currently does
- Uses versioned sqlx migrations and a dedicated query layer over Postgres.
- Persists users, org membership, tokens, repos, repo files, commits, LFS objects, and lightweight repo refs.
- Keeps the repository model file-centric, with current-head metadata and no explicit Git object table.
- Separates CAS metadata (migration 001) from Hub/repo metadata (migration 002+) in a cleaner operational style than OpenXet’s inline DDL.

## Gap inventory
| Area | OpenXet behavior | Current behavior | Gap type | Notes |
| --- | --- | --- | --- | --- |
| Repo/object persistence | Git objects and richer refs modeled explicitly | Repo files + commits + lightweight refs | persistence | Core mismatch |
| Schema evolution | Inline schema creation | Versioned migrations | persistence | xet-backend approach is operationally stronger |
| Query style | Direct entity usage | Dedicated query modules | persistence | xet-backend style is worth preserving |
| Product-layer entities | Discussions/PRs/likes first-class | Not modeled | persistence, ui | Likely later-phase work |
| Reconstruction metadata shape | More relational file/block metadata | JSONB file mappings + CAS tables | persistence, storage | Different but not inherently worse |

Gap type uses one or more of: `protocol`, `persistence`, `storage`, `routing`, `auth`, `ui`.

## Recommendation
- Decision: **extend ours**
- Why this is the best next move:
  - `xet-backend`’s Postgres + sqlx + migration workflow is already a better fit for the current Axum/sqlx/S3 workspace than adopting SeaORM + inline schema creation.
  - The real work is to extend the schema with the missing Git-native concepts, not to replace the persistence stack.
  - OpenXet’s data model is valuable as a comparison target and source of missing entities/relationships, especially around refs/objects/history.
- Dependencies or blockers:
  - Need a clear Git foundation target from `compare_git.md`
  - Need to decide which product-layer entities are in scope for this epic versus later

## Verification to design later
- Migration tests for any added object/ref/history tables
- Query-layer tests for ref/history resolution under the richer model
- Backward-compat tests proving existing HF Hub behaviors still pass after schema expansion

## Evidence
- OpenXet anchors: `round1/openxet_db.md`, `round1/openxet_git.md`
- xet-backend anchors:
  - `crates/db-layer/migrations/002_hub_schema.sql`
  - `crates/db-layer/migrations/003_repo_refs.sql`
  - `crates/db-layer/src/queries/commits.rs`
  - `crates/db-layer/src/queries/repo_files.rs`
  - `crates/db-layer/src/queries/repo_refs.rs`
  - `crates/db-layer/src/queries/repositories.rs`
