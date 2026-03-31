# Git Comparison

## Metadata
- Module: `git`
- OpenXet map: `round1/openxet_git.md`
- Local anchors:
  - `crates/db-layer/src/queries/commits.rs`
  - `crates/db-layer/src/queries/repo_refs.rs`
  - `crates/hub-api/src/auth.rs`
  - `crates/hub-api/src/routes/repos.rs`
  - `crates/server/src/main.rs`
- Author: Claude
- Status: review-ready

## What OpenXet does
- Implements a Git-native foundation: object IDs, object types, refs, repository store, Smart HTTP transport, and pack parsing/generation.
- Treats branches/tags/HEAD as first-class ref semantics.
- Builds protocol behavior over explicit Git objects rather than file snapshots.

## What xet-backend currently does
- Stores commits and lightweight refs in Postgres, but does not implement Git object storage or Git transport.
- Uses `commits.rs` and `repo_refs.rs` to support Hub-compatible revision and named-ref behavior.
- Resolves revisions through current-head and lightweight refs in `hub-api/src/auth.rs` and route handlers.
- Has no Smart HTTP endpoints, pack handling, or persisted blob/tree/tag model.

## Gap inventory
| Area | OpenXet behavior | Current behavior | Gap type | Notes |
| --- | --- | --- | --- | --- |
| Transport | Smart HTTP fetch/push implemented | No Git transport | protocol, routing | Critical gap |
| Object model | Git objects are first-class | No git_objects layer | persistence, storage | Current repo model is file-centric |
| Ref semantics | Symbolic/direct refs and advertisement | Lightweight branch/tag rows only | protocol, persistence | Enough for HF refs, not for Git-native workflows |
| History traversal | Commit/tree/object walks and pack generation | Minimal commit metadata only | protocol, persistence | Current ancestry model is too shallow |
| Repo abstraction | RepositoryStore over Git objects | Hub routes operate over repo files and head SHA | persistence | Architectural mismatch |

Gap type uses one or more of: `protocol`, `persistence`, `storage`, `routing`, `auth`, `ui`.

## Recommendation
- Decision: **adapt/adopt theirs**
- Why this is the best next move:
  - `xet-backend` is missing the entire Git-native layer, so this is not a small extension of the current route code.
  - OpenXet already demonstrates a clean split between protocol, pack, and repository/object storage responsibilities.
  - The best path is to adapt that structural split onto `xet-backend`’s own Postgres/S3/CAS foundation rather than inventing the Git layer shape from scratch.
- Dependencies or blockers:
  - DB/object-model decisions in `compare_db.md`
  - Storage seam decisions in `compare_storage.md`
  - CAS coexistence decisions in `compare_cas.md`

## Verification to design later
- Git Smart HTTP clone/fetch/push tests
- Ref advertisement and branch/tag visibility tests
- Commit/history traversal tests
- Cross-protocol tests proving Git pushes and HF reads resolve the same refs and revisions

## Evidence
- OpenXet anchors: `round1/openxet_git.md`, `round1/openxet_main.md`
- xet-backend anchors:
  - `crates/db-layer/src/queries/commits.rs`
  - `crates/db-layer/src/queries/repo_refs.rs`
  - `crates/hub-api/src/auth.rs`
  - `crates/hub-api/src/routes/repos.rs`
  - `crates/server/src/main.rs`
