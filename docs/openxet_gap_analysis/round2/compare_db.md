# Round 2 Comparison — `db`

**OpenXet commit:** `a5b6dfcdf5e806a5a29cb893f0d79d733bea309b`
**Analyst:** DiTo97
**Date:** 2026-03-31
**Tracking issue:** [#51](https://github.com/Atena-IT/open-hub/issues/51)
**Round 1 source:** `docs/openxet_gap_analysis/round1/openxet_db.md` (on branch `worktree-issue-42-openxet-db-map`; not yet merged to this branch)

---

## Scope

This document compares the OpenXet `db` module (mapped in Round 1) against the corresponding implementation in `xet-backend`. It covers schema design, entity models, query patterns, migration strategy, and persistence architecture. Community/collaboration tables (discussions, pull requests, repo likes) are classified as out-of-scope per the current compatibility target.

## xet-backend counterpart

The xet-backend database layer lives in the `db-layer` crate. It uses PostgreSQL via `sqlx` (compile-time-checked queries) instead of OpenXet's SQLite via SeaORM (ORM entity model). The schema is managed through `sqlx::migrate!` with numbered SQL migration files, not inline raw DDL.

| Responsibility | xet-backend location | Notes |
| --- | --- | --- |
| Connection pool & migration runner | `crates/db-layer/src/lib.rs` | `create_pool` opens PgPool (max 20 connections) and runs `sqlx::migrate!("./migrations")` |
| CAS schema (xorbs, chunks, shards, file_mappings) | `crates/db-layer/migrations/001_initial.sql` | First migration; CAS-only tables |
| Hub schema (users, repos, tokens, files, commits, LFS) | `crates/db-layer/migrations/002_hub_schema.sql` | Second migration; all Hub platform tables |
| Repo refs schema | `crates/db-layer/migrations/003_repo_refs.sql` | Third migration; `repo_refs` table |
| User queries | `crates/db-layer/src/queries/users.rs` | `create_user`, `find_user_by_username`, `find_user_by_id`, `create_org` |
| Repository queries | `crates/db-layer/src/queries/repositories.rs` | CRUD + `list_repos_for_owner`, `list_repos_by_type` |
| Access token queries | `crates/db-layer/src/queries/access_tokens.rs` | `create_token`, `find_token_by_hash`, `list_tokens_for_user`, `delete_token`, `touch_token` |
| Xorb queries | `crates/db-layer/src/queries/xorbs.rs` | `upsert_xorb`, `xorb_exists`, `get_xorb_s3_key` |
| Chunk queries | `crates/db-layer/src/queries/xorbs.rs` | `upsert_chunks`, `find_chunk`, `chunks_for_xorb`; re-exported via `queries/chunks.rs` |
| Commit queries | `crates/db-layer/src/queries/commits.rs` | `create_commit`, `get_latest_commit`, `find_commit_by_sha` |
| File mapping queries | `crates/db-layer/src/queries/file_mappings.rs` | `upsert_file_mapping`, `get_file_mapping` + `ReconstructionTerm` |
| LFS object queries | `crates/db-layer/src/queries/lfs_objects.rs` | `upsert_lfs_object`, `find_lfs_object`, `list_lfs_objects` |
| Repo file queries | `crates/db-layer/src/queries/repo_files.rs` | `upsert_file`, `find_file`, `list_files`, `delete_file` |
| Repo ref queries | `crates/db-layer/src/queries/repo_refs.rs` | `list_refs`, `find_ref_by_name`, `create_ref`, `delete_ref` |

## Surface comparison

Status values: `covered` | `partial` | `missing` | `out-of-scope`

### Infrastructure and bootstrap

| OpenXet symbol / responsibility | Status | xet-backend equivalent | Notes |
| --- | --- | --- | --- |
| `init_database` (opens SQLite, runs DDL) | `covered` | `create_pool` in `crates/db-layer/src/lib.rs:6` | Replaced by PgPool + sqlx migrations |
| `create_tables` (18 inline `CREATE TABLE` statements) | `covered` | `sqlx::migrate!("./migrations")` in `crates/db-layer/src/lib.rs:13` | Proper migration framework vs. inline raw SQL |
| `ALTER TABLE ADD COLUMN` in-place migration | `covered` | sqlx numbered migrations (`001_`, `002_`, `003_`) | Declarative evolution; no silent error suppression |
| SeaORM entity layer | `partial` | sqlx `FromRow` derive structs | No ORM; hand-written SQL queries with compile-time checking. Achieves type safety differently |
| `entities` module (re-exports) | `missing` | (no equivalent) | xet-backend uses per-module query functions, not entity type aliases |

### Database engine and type system

| OpenXet choice | xet-backend choice | Implications |
| --- | --- | --- |
| SQLite (single-file, embedded) | PostgreSQL (server-mode, via PgPool) | Full concurrent-write support, row-level locking, production-grade; requires separate process |
| `i32` autoincrement PKs | `UUID` PKs (`gen_random_uuid()`) | Globally unique, no sequence contention; larger storage |
| `i64` Unix-epoch timestamps | `TIMESTAMPTZ` via `chrono::DateTime<Utc>` | Time-zone-aware, queryable, no manual conversion |
| Hash columns as `TEXT` (64-char hex) | Hash columns as `BYTEA` (raw 32 bytes) | Compact storage, correct binary comparison; requires hex encoding at API boundary |
| `String` comma-separated scopes | `TEXT[]` PostgreSQL array | Queryable, type-safe; no parsing needed |
| SeaORM `DeriveEntityModel` | `sqlx::FromRow` derive | Compile-time SQL verification vs. runtime ORM |

### Table-by-table comparison

| OpenXet table | Status | xet-backend table | Notes |
| --- | --- | --- | --- |
| `users` | `covered` | `users` (002) | UUID PK, added `full_name`, `avatar_url`, `updated_at`. Dual-purpose user/org pattern preserved (`is_org` flag). Missing: `display_name` (replaced by `full_name`). Email is UNIQUE in xet-backend |
| `org_members` | `covered` | `org_members` (002) | Composite PK `(org_id, user_id)` instead of surrogate. Same role model. CASCADE on both FKs |
| `repositories` | `partial` | `repositories` (002) | Added `full_name` (UNIQUE, `"owner/repo"`), `repo_type` (model/dataset/space), `private` flag, `description`, `updated_at`. Uses `head_sha` (nullable) instead of `head` (string). No symbolic HEAD |
| `git_refs` | `partial` | `repo_refs` (003) | UUID PK (not string composite). Added `ref_type` column (branch/tag). No `is_symbolic` / `symbolic_target` columns. UNIQUE on `(repo_id, name)` |
| `git_objects` | `missing` | (none) | xet-backend does not track individual git objects in the DB. No blob/tree/commit/tag object store at the DB level |
| `lfs_objects` | `partial` | `lfs_objects` (002) | Scoped to `repo_id` (FK) instead of global OID PK. No `status`/`raw_path` columns -- no background chunking lifecycle. Stores `s3_key` directly |
| `lfs_chunks` | `missing` | (none) | xet-backend does not maintain an ordered chunk map for LFS objects. LFS lifecycle is simpler (direct S3) |
| `cas_blocks` | `covered` | `xorbs` (001) | Renamed from "block" to "xorb" (matching xet-core terminology). BYTEA hash PK. Stores `s3_key` + `size_bytes`. No `chunk_count` column |
| `cas_chunks` | `covered` | `chunks` (001) | BYTEA hash PK. FK to `xorbs(hash)` with ON DELETE RESTRICT (not CASCADE). Stores `chunk_index_in_xorb`, `byte_range_start`, `unpacked_segment_bytes` instead of `offset_in_block` + `size` |
| `file_segments` | `covered` | `file_mappings` (001) | Different model: xet-backend stores reconstruction as a single JSONB array per file (via `ReconstructionTerm`), not one row per segment. More compact, less relational |
| `access_tokens` | `partial` | `access_tokens` (002) | BYTEA `token_hash` (not hex TEXT). `TEXT[]` scopes instead of comma-separated. No `token_prefix`, `description`, `expires_at`, `is_active` columns. Hard-delete only (no soft-delete revocation) |
| `discussions` | `out-of-scope` | (none) | Community feature; not in current compatibility target |
| `discussion_comments` | `out-of-scope` | (none) | Community feature |
| `discussion_events` | `out-of-scope` | (none) | Community feature |
| `repo_likes` | `out-of-scope` | (none) | Community feature |
| `pull_requests` | `out-of-scope` | (none) | Community feature |
| `pr_comments` | `out-of-scope` | (none) | Community feature |
| `pr_events` | `out-of-scope` | (none) | Community feature |

### Tables present in xet-backend but absent from OpenXet

| xet-backend table | Migration | Purpose |
| --- | --- | --- |
| `shards` | 001 | Tracks shard blobs in S3 by BYTEA hash. OpenXet has no shard-level metadata table (shards are a protocol concept not stored in its DB) |
| `repo_files` | 002 | Flat file manifest per repo: path, size, sha256, s3_key, LFS flag. OpenXet resolves files by walking the git tree from `git_objects`; xet-backend stores files directly |
| `commits` | 002 | Explicit commit log (sha, message, author_id, parent_sha). OpenXet models commits as `git_objects` with type=2; xet-backend models them as first-class entities with metadata |
| `repo_refs` | 003 | Equivalent of OpenXet's `git_refs` but with a separate migration and `ref_type` column |

### Query layer comparison

| OpenXet pattern | Status | xet-backend pattern | Notes |
| --- | --- | --- | --- |
| SeaORM `ActiveModel` insert/update | `covered` | Raw sqlx `query` / `query_as` with `$N` bind params | Hand-written SQL; compile-time verified |
| Fire-and-forget DB writes (`tokio::spawn`) | `partial` | Synchronous `await?` with `anyhow::Result` propagation | Most DB writes are awaited with error propagation. Exception: `touch_token` in `hub-api/src/auth.rs` is awaited but its `Result` is discarded via `let _ =`, silently dropping errors |
| In-memory DashMap as primary, DB as write-through | `partial` | DB-primary model; all reads hit PostgreSQL | No in-memory cache layer. Simpler consistency model |
| `load_from_db` startup preload | `missing` | (none) | xet-backend does not preload state into memory at startup. All queries go directly to the DB |
| Dual construction paths (`with_db` / `with_storage_path`) | `missing` | (none) | Single construction path via `create_pool`. Tests require a real (or test-scoped) PostgreSQL database |

## Gaps

### Missing

- **`git_objects` table** -- OpenXet tracks individual git objects (blobs, trees, commits, tags) in a DB table with per-object `object_type` discriminant. xet-backend has no equivalent. Git objects in xet-backend are handled through the `repo_files` flat manifest and `commits` table, not through a full git object graph. This means xet-backend cannot store or query arbitrary git trees, tags, or non-file blobs at the DB level.

- **`lfs_chunks` table** -- OpenXet maintains an ordered chunk map linking each LFS object to its constituent BLAKE3 chunks. xet-backend does not track LFS-to-chunk relationships. LFS objects go directly to S3 (`s3_key`); the CAS chunking path (`xorbs` + `chunks`) is separate and used for the Xet protocol, not for LFS reconstruction.

- **SeaORM entity re-exports** -- OpenXet exposes type-aliased entities (`entities::User`, `entities::Repository`, etc.) for convenient access across modules. xet-backend has no entity module; callers import row structs and query functions from per-domain modules (`queries::users::UserRow`, `queries::repositories::RepoRow`).

- **In-memory state preload** -- OpenXet's `load_from_db` populates DashMaps at startup for fast reads. xet-backend has no in-memory caching; it queries PostgreSQL for every read. This is architecturally intentional (PostgreSQL handles caching and concurrency natively) but represents a design divergence.

### Partial

- **`lfs_objects` -- no lifecycle state machine** -- OpenXet models LFS objects with a three-state lifecycle (`Raw` -> `Processing` -> `Chunked`) and a `raw_path` for temporary disk storage during the chunking pipeline. xet-backend's `lfs_objects` has no `status` or `raw_path` columns; objects are stored directly in S3. If a background chunking pipeline is needed, the schema would need to be extended.

- **`access_tokens` -- fewer metadata columns** -- xet-backend's token table lacks `token_prefix` (display prefix), `description`, `expires_at`, and `is_active` (soft-delete). Tokens can only be hard-deleted. This means: (a) no user-friendly token identification in the UI without the prefix, (b) no token expiry enforcement, (c) no soft-revocation (delete is permanent).

- **`git_refs` / `repo_refs` -- no symbolic refs** -- OpenXet's `git_refs` supports symbolic references (`is_symbolic`, `symbolic_target`) for modeling HEAD as a pointer to another ref. xet-backend's `repo_refs` stores only direct `target_sha` values and a `ref_type` (branch/tag). Symbolic ref resolution must be handled at the application layer if needed (e.g., HEAD stored on `repositories.head_sha`).

- **`repositories` -- `head_sha` vs. symbolic `head`** -- OpenXet stores `head` as a symbolic ref string (e.g., `refs/heads/main`). xet-backend stores `head_sha` as the resolved commit SHA (nullable). This means xet-backend cannot represent an unresolved HEAD pointing to a branch name; it must eagerly resolve on every commit.

## Already covered

- **User/org dual-purpose model** -- Both systems use a single `users` table with an `is_org` flag. xet-backend adds `full_name` and `avatar_url` but preserves the same pattern. Empty `password_hash` for orgs in both.

- **Org membership** -- Both systems use a join table (`org_members`) with an FK pair and a free-form `role` column. xet-backend uses a composite PK directly instead of a surrogate ID.

- **Repository metadata columns** -- xet-backend's `repositories` table adds `full_name` (unique `"owner/repo"`), `repo_type`, `private`, `description`, and `updated_at` beyond OpenXet's schema. The `full_name` column eliminates the denormalization problem OpenXet has in its collaboration tables. (Note: the table is `partial` overall due to the `head_sha` vs. symbolic `head` limitation documented in the Gaps section.)

- **CAS block/chunk storage** -- The `xorbs` and `chunks` tables map directly to OpenXet's `cas_blocks` and `cas_chunks`. Naming follows xet-core convention (`xorb` instead of `block`). Hashes are stored as BYTEA (correct binary type) instead of hex TEXT. FK constraint is ON DELETE RESTRICT (safer than CASCADE for content-addressed data).

- **File reconstruction** -- The `file_mappings` table covers the same purpose as OpenXet's `file_segments`, but uses a single JSONB column (`reconstruction_terms`) instead of one row per segment. Both approaches map a file hash to an ordered sequence of xorb/block ranges.

- **Access token security model** -- Both systems store only a hash of the token, never the raw value. Both return the raw token once at creation. xet-backend stores the hash as BYTEA (binary) rather than hex TEXT. (Note: the table is `partial` overall due to missing metadata columns documented in the Gaps section.)

- **Proper FK constraints throughout** -- xet-backend uses ON DELETE CASCADE for parent-child relationships (users -> repos, repos -> files, repos -> commits, repos -> lfs_objects, repos -> refs) and ON DELETE RESTRICT for content-addressed data (chunks -> xorbs). OpenXet's collaboration tables lack FK constraints; xet-backend avoids this problem by not implementing those tables.

## Out-of-scope items

- **Community / collaboration tables** (`discussions`, `discussion_comments`, `discussion_events`, `repo_likes`, `pull_requests`, `pr_comments`, `pr_events`) -- These 7 tables exist in OpenXet but serve only the web UI layer. They are not exposed via the HF-compatible API and are excluded from the current xet-backend compatibility target. In OpenXet's codebase, these tables also have referential integrity issues (bare `repo_name TEXT` without FK constraints) that would need architectural resolution before adoption.

- **SQLite-specific patterns** -- OpenXet's `ALTER TABLE ADD COLUMN` silent-failure migration strategy and inline raw-SQL schema are SQLite-specific implementation choices. xet-backend uses PostgreSQL with sqlx migrations, making these patterns irrelevant.

- **In-memory primary persistence model** -- OpenXet's DashMap + fire-and-forget architecture is a design choice tied to its single-process SQLite model. xet-backend's DB-primary model is an intentional architectural divergence, not a gap. This is not flagged as something to reconcile.

## Recommendations for synthesis

- **`git_objects` gap is architectural, not incremental.** OpenXet stores individual git objects (blobs, trees, commits, tags) and navigates them via tree-walking. xet-backend replaces this with a flat `repo_files` manifest and explicit `commits` table. These are fundamentally different file-resolution strategies. The synthesis round should decide whether xet-backend needs to adopt git-object-level tracking (for Git Smart HTTP fidelity) or can continue with the flat-file approach (sufficient for the HF Hub API surface).

- **LFS lifecycle simplification is intentional but limits dedup.** xet-backend's LFS path skips the `Raw -> Processing -> Chunked` pipeline and stores objects directly in S3. This means LFS objects are not automatically chunked and deduplicated through the CAS pipeline. If CAS-level deduplication of LFS content is a goal, the `lfs_objects` schema needs `status` and the background worker pattern from OpenXet.

- **Token expiry and soft-delete are absent.** The missing `expires_at` and `is_active` columns on `access_tokens` mean tokens live indefinitely and cannot be revoked without permanent deletion. This is a security-relevant divergence from OpenXet's model for multi-user deployments.

- **`repo_files` and `commits` are xet-backend innovations not present in OpenXet.** These tables simplify Hub API file listing and commit history without requiring full git-object tree-walking. The synthesis should consider whether these tables remain sufficient or whether OpenXet's `git_objects` approach is needed for correctness in edge cases (e.g., directory listing from non-HEAD refs, tag objects).

- **BYTEA hashes are a strict improvement.** xet-backend's choice to store hashes as raw bytes is more space-efficient and avoids hex-encoding ambiguities. No action needed; this is a positive divergence from OpenXet's TEXT hashes.

- **Migration framework is a strict improvement.** xet-backend's sqlx numbered migrations are production-ready. No action needed; OpenXet's inline DDL with silent `ALTER TABLE` failures is a known weakness documented in the Round 1 map.

## Open questions

- Does xet-backend need a `git_objects` table (or equivalent) to support Git Smart HTTP protocol operations (upload-pack, receive-pack)? The current `repo_files` + `commits` model may be insufficient if full pack negotiation is a compatibility target.

- Should `lfs_objects` gain a `status` column and background worker to enable CAS-level dedup of LFS content, or is the direct-to-S3 model the intended long-term design?

- The `shards` table in xet-backend (migration 001) has no corresponding query module in `crates/db-layer/src/queries/`. Is this table actively used, or is it vestigial from an earlier design?

- xet-backend's `repo_refs` has no support for symbolic refs. If `HEAD` needs to track a branch name (not just a resolved SHA), should a `symbolic_target` column be added to `repo_refs`, or is `repositories.head_sha` sufficient?

- The `chunks` FK to `xorbs` uses ON DELETE RESTRICT. If xorb garbage collection is planned, this constraint will block deletion of xorbs that still have chunk references. Is this intentional (requiring explicit chunk cleanup first), or should the constraint be relaxed?

- `access_tokens` has no `expires_at` or `is_active` column. Is permanent, hard-delete-only token management the intended design, or are these columns planned for a future migration?
