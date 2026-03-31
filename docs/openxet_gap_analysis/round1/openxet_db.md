# OpenXet DB Module Map

## Metadata
- Module: `db`
- Source paths:
  - `../OpenXet-reference/src/db/mod.rs`
  - `../OpenXet-reference/src/db/entities/`
- Author: Claude
- Status: review-ready

## Build shape
- `mod.rs` owns database initialization and inline schema creation.
- `entities/` contains SeaORM models for users, org membership, repositories, refs, Git objects, LFS/CAS metadata, discussions, PRs, comments, likes, and tokens.
- There is no separate query/repository layer; consumers are expected to use SeaORM entities directly.

## Responsibilities
- Initialize the SQLite database and create all required tables/indexes at startup.
- Model the full server domain: auth, repos, Git metadata, CAS/LFS metadata, and collaboration primitives.
- Provide relational structure for repo ownership, tokens, org membership, discussion timelines, PRs, and storage metadata.

## Explicit non-goals
- No migration framework or versioned SQL files; schema evolution is embedded in startup code.
- No dedicated query layer or repo abstraction comparable to `xet-backend`’s `queries/` modules.
- No production-grade DB portability; the implementation is clearly optimized for SQLite + SeaORM.
- No byte-optimized hash encoding strategy; many hashes are persisted as hex/text values.

## Key contracts
### Types / entities
- Core identity/repo entities: `user`, `org_member`, `repository`, `git_ref`, `git_object`, `access_token`.
- Storage entities: `lfs_object`, `lfs_chunk`, `cas_block`, `cas_chunk`, `file_segment`.
- Product/collab entities: `discussion`, `discussion_comment`, `discussion_event`, `pull_request`, `pr_comment`, `pr_event`, `repo_like`.

### Routes / protocol surfaces
- The DB layer itself exposes no routes, but its schema directly supports the API, Git, Web UI, and token-management surfaces.
- Many higher-level features in OpenXet exist because the DB module already models them directly.

### Storage / persistence contracts
- Repos, refs, and Git objects are persisted alongside storage metadata instead of relying only on object storage.
- CAS/LFS reconstruction depends on relational metadata like chunk lists, block membership, and file segments.
- Access tokens and org membership are DB-backed and feed namespace-aware auth decisions.

## Integration points
- Upstream: initialized by `main.rs` before `AppState` is built.
- Downstream: consumed directly by `api`, `auth`, `git`, `cas`, and `web_ui` through SeaORM entities.
- Adjacent: the DB schema is one of the strongest signals that OpenXet is built from a Git- and product-native foundation rather than a thin compatibility layer.

## Runtime and configuration assumptions
- Uses SQLite and opens/creates the DB file during startup.
- Assumes schema creation is safe to run on each boot.
- Assumes a single-process deployment model that can rely on inline schema initialization.
- Expects SeaORM as the persistence API throughout the codebase.

## Questions for comparison round
- Which OpenXet entities correspond directly to existing `xet-backend` tables, and where are the deepest mismatches in repo/object/ref/history semantics?
- Would `xet-backend` benefit more from extending its current sqlx/Postgres schema or from adapting parts of OpenXet’s data model structure?
- Which tables in OpenXet are foundational for a Git-native server, and which are product-layer additions that can wait?
- How much of OpenXet’s direct-entity usage would need to be translated into `xet-backend`’s existing query-layer style?

## Evidence
- `../OpenXet-reference/src/db/mod.rs`
- `../OpenXet-reference/src/db/entities/mod.rs`
- `../OpenXet-reference/src/db/entities/user.rs`
- `../OpenXet-reference/src/db/entities/repository.rs`
- `../OpenXet-reference/src/db/entities/git_ref.rs`
- `../OpenXet-reference/src/db/entities/git_object.rs`
- `../OpenXet-reference/src/db/entities/lfs_object.rs`
- `../OpenXet-reference/src/db/entities/cas_chunk.rs`
- `../OpenXet-reference/src/db/entities/file_segment.rs`
