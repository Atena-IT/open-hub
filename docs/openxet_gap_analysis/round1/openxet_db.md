# Round 1 Module Map — `db`

**OpenXet commit:** `a5b6dfcdf5e806a5a29cb893f0d79d733bea309b`
**Analyst:** DiTo97
**Date:** 2026-03-31
**Tracking issue:** [#42](https://github.com/Atena-IT/open-hub/issues/42)

---

## Scope

This document maps the OpenXet `db` module. It covers only the upstream OpenXet source — no `xet-backend` references appear here. Round 2 will introduce the comparison.

## Source location

```
src/db/
```

Key entry points:
- `src/db/mod.rs` — module root: `init_database` (public), `create_tables` (private), full DDL for all 18 tables
- `src/db/entities/mod.rs` — re-exports all SeaORM entity types under short aliases
- `src/db/entities/user.rs` — `users` table entity (dual-purpose: users and orgs)
- `src/db/entities/org_member.rs` — `org_members` join table
- `src/db/entities/repository.rs` — `repositories` table entity
- `src/db/entities/git_ref.rs` — `git_refs` table entity
- `src/db/entities/git_object.rs` — `git_objects` table entity
- `src/db/entities/lfs_object.rs` — `lfs_objects` table entity
- `src/db/entities/lfs_chunk.rs` — `lfs_chunks` table entity
- `src/db/entities/cas_block.rs` — `cas_blocks` table entity
- `src/db/entities/cas_chunk.rs` — `cas_chunks` table entity
- `src/db/entities/file_segment.rs` — `file_segments` table entity
- `src/db/entities/discussion.rs` — `discussions` table entity
- `src/db/entities/discussion_comment.rs` — `discussion_comments` table entity
- `src/db/entities/discussion_event.rs` — `discussion_events` table entity
- `src/db/entities/repo_like.rs` — `repo_likes` table entity
- `src/db/entities/pull_request.rs` — `pull_requests` table entity
- `src/db/entities/pr_comment.rs` — `pr_comments` table entity
- `src/db/entities/pr_event.rs` — `pr_events` table entity
- `src/db/entities/access_token.rs` — `access_tokens` table entity

## Public surface

| Symbol | Kind | File | Brief description |
| --- | --- | --- | --- |
| `init_database` | `async fn` | `src/db/mod.rs` | Opens (or creates) the SQLite file, runs `create_tables`, and returns a `DatabaseConnection` |
| `entities` | `mod` | `src/db/mod.rs` | Public sub-module containing all SeaORM entity definitions |
| `entities::User` | `type alias` | `src/db/entities/mod.rs` | Re-export of `user::Entity` |
| `entities::OrgMember` | `type alias` | `src/db/entities/mod.rs` | Re-export of `org_member::Entity` |
| `entities::Repository` | `type alias` | `src/db/entities/mod.rs` | Re-export of `repository::Entity` |
| `entities::GitRef` | `type alias` | `src/db/entities/mod.rs` | Re-export of `git_ref::Entity` |
| `entities::GitObject` | `type alias` | `src/db/entities/mod.rs` | Re-export of `git_object::Entity` |
| `entities::LfsObject` | `type alias` | `src/db/entities/mod.rs` | Re-export of `lfs_object::Entity` |
| `entities::LfsChunk` | `type alias` | `src/db/entities/mod.rs` | Re-export of `lfs_chunk::Entity` |
| `entities::CasBlock` | `type alias` | `src/db/entities/mod.rs` | Re-export of `cas_block::Entity` |
| `entities::CasChunk` | `type alias` | `src/db/entities/mod.rs` | Re-export of `cas_chunk::Entity` |
| `entities::FileSegment` | `type alias` | `src/db/entities/mod.rs` | Re-export of `file_segment::Entity` |
| `entities::Discussion` | `type alias` | `src/db/entities/mod.rs` | Re-export of `discussion::Entity` |
| `entities::DiscussionComment` | `type alias` | `src/db/entities/mod.rs` | Re-export of `discussion_comment::Entity` |
| `entities::DiscussionEvent` | `type alias` | `src/db/entities/mod.rs` | Re-export of `discussion_event::Entity` |
| `entities::RepoLike` | `type alias` | `src/db/entities/mod.rs` | Re-export of `repo_like::Entity` |
| `entities::PullRequest` | `type alias` | `src/db/entities/mod.rs` | Re-export of `pull_request::Entity` |
| `entities::PrComment` | `type alias` | `src/db/entities/mod.rs` | Re-export of `pr_comment::Entity` |
| `entities::PrEvent` | `type alias` | `src/db/entities/mod.rs` | Re-export of `pr_event::Entity` |
| `entities::AccessToken` | `type alias` | `src/db/entities/mod.rs` | Re-export of `access_token::Entity` |

## Internal structure

### Sub-modules

| Sub-module | Purpose |
| --- | --- |
| `entities` | One SeaORM entity per table; re-exports all `Entity` type aliases for convenient external access |
| `entities::user` | `users` table (dual-purpose: regular users and organisations) |
| `entities::org_member` | `org_members` join table linking users to organisations |
| `entities::repository` | `repositories` table |
| `entities::git_ref` | `git_refs` table storing branch/tag/HEAD references per repo |
| `entities::git_object` | `git_objects` table tracking which objects belong to which repo |
| `entities::lfs_object` | `lfs_objects` table with upload lifecycle state |
| `entities::lfs_chunk` | `lfs_chunks` ordered chunk map for LFS object reconstruction |
| `entities::cas_block` | `cas_blocks` table — metadata for bundled content blocks |
| `entities::cas_chunk` | `cas_chunks` table — individual chunk positions within blocks |
| `entities::file_segment` | `file_segments` table — byte-range map for file reconstruction |
| `entities::discussion` | `discussions` table — community discussion threads |
| `entities::discussion_comment` | `discussion_comments` table |
| `entities::discussion_event` | `discussion_events` table — lifecycle audit trail for discussions |
| `entities::repo_like` | `repo_likes` table — per-user repository star/like records |
| `entities::pull_request` | `pull_requests` table |
| `entities::pr_comment` | `pr_comments` table — supports optional inline file/line annotations |
| `entities::pr_event` | `pr_events` table — lifecycle audit trail for pull requests |
| `entities::access_token` | `access_tokens` table — persisted `ox_*` API tokens |

### Key types

#### `user::Model` (`users` table)

Dual-purpose: stores both regular users and organisations in a single table. An `is_org: bool` flag differentiates the two kinds.

| Column | Type | Notes |
| --- | --- | --- |
| `id` | `i32` PK autoincrement | |
| `username` | `String` UNIQUE | also used as the org name |
| `password_hash` | `String` | Argon2id PHC string or legacy SHA-256 hex; empty string for orgs |
| `display_name` | `Option<String>` | |
| `email` | `Option<String>` | |
| `is_org` | `bool` | `false` = user, `true` = organisation |
| `created_at` | `i64` | Unix epoch seconds |

SeaORM relation: `has_many -> repository::Entity`.

#### `org_member::Model` (`org_members` table)

| Column | Type | Notes |
| --- | --- | --- |
| `id` | `i32` PK autoincrement | |
| `org_id` | `i32` | FK → `users.id` (must have `is_org=1`) |
| `user_id` | `i32` | FK → `users.id` |
| `role` | `String` | free-form; comment lists `"owner"`, `"admin"`, `"member"` |
| `created_at` | `i64` | |

Unique constraint on `(org_id, user_id)`. No `impl Related` blocks are defined in the entity — relations exist only as `DeriveRelation` enum variants.

#### `repository::Model` (`repositories` table)

| Column | Type | Notes |
| --- | --- | --- |
| `id` | `i32` PK autoincrement | |
| `owner_id` | `i32` | FK → `users.id` ON DELETE CASCADE |
| `name` | `String` | bare repo name |
| `head` | `String` | symbolic HEAD (e.g., `refs/heads/main`) |
| `created_at` | `i64` | |

Unique constraint on `(owner_id, name)`. Relations: `belongs_to User`, `has_many GitRef`, `has_many GitObject`.

#### `git_ref::Model` (`git_refs` table)

| Column | Type | Notes |
| --- | --- | --- |
| `id` | `String` PK (non-autoincrement) | composite key: `"{repo_id}/{ref_name}"` |
| `repo_id` | `i32` | FK → `repositories.id` ON DELETE CASCADE |
| `ref_name` | `String` | full ref name, e.g., `refs/heads/main` |
| `target_hash` | `String` | 40-char hex SHA-1 (for direct refs) |
| `is_symbolic` | `bool` | `true` if ref is a symbolic ref (e.g., `HEAD`) |
| `symbolic_target` | `Option<String>` | target ref name when `is_symbolic` is true |

#### `git_object::Model` (`git_objects` table)

| Column | Type | Notes |
| --- | --- | --- |
| `id` | `String` PK (non-autoincrement) | composite key: `"{repo_id}/{object_hash}"` |
| `repo_id` | `i32` | FK → `repositories.id` ON DELETE CASCADE |
| `object_hash` | `String` | 40-char hex SHA-1 |
| `object_type` | `i32` | `0`=blob, `1`=tree, `2`=commit, `3`=tag |

#### `lfs_object::Model` (`lfs_objects` table)

| Column | Type | Notes |
| --- | --- | --- |
| `oid` | `String` PK (non-autoincrement) | 64-char hex SHA-256 |
| `size` | `i64` | declared size in bytes |
| `status` | `i32` | `0`=Raw, `1`=Processing, `2`=Chunked |
| `raw_path` | `Option<String>` | filesystem path while status is Raw/Processing |

Relation: `has_many LfsChunk`.

#### `lfs_chunk::Model` (`lfs_chunks` table)

| Column | Type | Notes |
| --- | --- | --- |
| `id` | `i64` PK autoincrement | |
| `lfs_oid` | `String` | FK → `lfs_objects.oid` ON DELETE CASCADE |
| `chunk_index` | `i32` | zero-based position in reconstruction order |
| `chunk_hash` | `String` | 64-char hex; comment says "BLAKE3" |

#### `cas_block::Model` (`cas_blocks` table)

| Column | Type | Notes |
| --- | --- | --- |
| `hash` | `String` PK (non-autoincrement) | 64-char hex; comment says "BLAKE3" |
| `size` | `i64` | total byte size of the block |
| `chunk_count` | `i32` | number of chunks bundled in this block |
| `storage_key` | `String` | path or object key in the storage backend |

Relation: `has_many CasChunk`.

#### `cas_chunk::Model` (`cas_chunks` table)

| Column | Type | Notes |
| --- | --- | --- |
| `hash` | `String` PK (non-autoincrement) | 64-char hex; comment says "BLAKE3" |
| `size` | `i64` | byte size of this chunk |
| `block_hash` | `String` | FK → `cas_blocks.hash` ON DELETE CASCADE |
| `offset_in_block` | `i64` | byte offset where this chunk starts within its block |

#### `file_segment::Model` (`file_segments` table)

| Column | Type | Notes |
| --- | --- | --- |
| `id` | `i32` PK autoincrement | |
| `file_hash` | `String` | 64-char hex identifying the logical file (LFS OID) |
| `segment_index` | `i32` | zero-based position in reconstruction order |
| `block_hash` | `String` | FK → `cas_blocks.hash` ON DELETE CASCADE |
| `byte_start` | `i64` | start byte offset within the block |
| `byte_end` | `i64` | end byte offset within the block |
| `segment_size` | `i64` | size of this byte range |

`file_hash` has no FK constraint; there is no parent table for the logical file itself beyond the LFS object OID stored in `lfs_objects`.

#### Community / collaboration tables

| Entity | Table | Key columns |
| --- | --- | --- |
| `discussion::Model` | `discussions` | `id`, `repo_name TEXT`, `author_id`, `title`, `status` (`"open"`/`"closed"`), `created_at`, `updated_at` |
| `discussion_comment::Model` | `discussion_comments` | `id`, `discussion_id`, `author_id`, `content`, `created_at`, `updated_at` |
| `discussion_event::Model` | `discussion_events` | `id`, `discussion_id`, `actor_id`, `event_type`, `old_value`, `new_value`, `created_at` |
| `repo_like::Model` | `repo_likes` | `id`, `repo_name TEXT`, `user_id`, `created_at`; unique on `(repo_name, user_id)` |
| `pull_request::Model` | `pull_requests` | `id`, `repo_name TEXT`, `number`, `author_id`, `title`, `description`, `source_branch`, `target_branch`, `source_commit?`, `target_commit?`, `status` (`"open"`/`"closed"`/`"merged"`), `merged_by?`, `merged_at?`, `created_at`, `updated_at` |
| `pr_comment::Model` | `pr_comments` | `id`, `pr_id`, `author_id`, `content`, `file_path?`, `line_number?`, `created_at`, `updated_at` |
| `pr_event::Model` | `pr_events` | `id`, `pr_id`, `actor_id`, `event_type`, `old_value?`, `new_value?`, `created_at` |

`Discussion`, `RepoLike`, and `PullRequest` all reference repositories via a bare `repo_name TEXT` column rather than a FK to `repositories.id`.

#### `access_token::Model` (`access_tokens` table)

| Column | Type | Notes |
| --- | --- | --- |
| `id` | `i32` PK autoincrement | |
| `user_id` | `i32` | FK → `users.id` ON DELETE CASCADE |
| `name` | `String` | human label (e.g., "CI/CD") |
| `token_hash` | `String` | SHA-256 of the raw `ox_*` token; never stored in plain |
| `token_prefix` | `String` | first 8 chars for display/identification (`ox_XXXX…`) |
| `description` | `Option<String>` | |
| `scopes` | `String` | comma-separated permissions or `"*"` for all |
| `last_used_at` | `i64` | Unix epoch; `0` if never used |
| `created_at` | `i64` | |
| `expires_at` | `i64` | Unix epoch; `0` means no expiry |
| `is_active` | `bool` | soft-delete / revoke flag |

Indexes on `token_hash` and `token_prefix` support fast lookup and display without exposing raw tokens.

## Intra-module dependencies

The `db` module itself depends on no other `crate::` modules. All inter-module dependency flows in the opposite direction: `crate::cas`, `crate::api`, and `crate::git` import from `crate::db::entities`.

| Dependency | What is used |
| --- | --- |
| `sea_orm` (external) | `Database`, `DatabaseConnection`, `DbErr`, `ConnectionTrait`, `Statement`, `DeriveEntityModel`, `DeriveRelation`, `EnumIter`, `Related`, `ActiveModelBehavior`, entity prelude |

## Notable implementation details

### Schema managed as inline raw SQL, not SeaORM migrations

`create_tables` in `src/db/mod.rs` executes 18 `CREATE TABLE IF NOT EXISTS` statements and their accompanying `CREATE INDEX IF NOT EXISTS` statements as raw SQL strings via `Statement::from_string`. There is no migration framework (no SeaORM migrations, no Diesel schema files, no Flyway/Liquibase). SeaORM is used only as an ORM query layer after the schema is established.

### In-place column migration via `ALTER TABLE ... ADD COLUMN`

Two `ALTER TABLE` statements are executed unconditionally at startup to add `password_hash` and `is_org` to `users` for databases created before those columns existed. Failures are silently discarded with `let _ = db.execute(…).await;`. This is the only migration mechanism.

### Dual-purpose `users` table

Both regular users and organisations are stored in a single `users` table differentiated by the `is_org` boolean. Organisations have an empty string as `password_hash`. `org_members` links users to orgs using two FKs that both point at `users.id`.

### String composite PKs for git tables

`git_refs.id` and `git_objects.id` are `String` primary keys constructed as `"{repo_id}/{ref_name}"` and `"{repo_id}/{object_hash}"` respectively. This avoids a separate surrogate key but ties the PK to a naming convention not enforced by the schema.

### Denormalised repository references in collaboration tables

`discussions`, `repo_likes`, and `pull_requests` reference repositories via a bare `repo_name TEXT` column rather than a FK to `repositories.id`. There is no referential integrity constraint between these tables and `repositories`. The column appears to hold an owner-scoped path such as `"owner/repo"` based on usage context in the API layer, but no constraint or comment confirms the format.

### `object_type` stored as an integer discriminant

`git_objects.object_type` uses the raw integers `0`=blob, `1`=tree, `2`=commit, `3`=tag. No Rust enum is defined in the entity to enforce the discriminant set; callers must cast manually.

### `file_segments.file_hash` has no FK parent

`file_segments` links to `cas_blocks` via `block_hash` (with ON DELETE CASCADE), but `file_hash` — which logically corresponds to an LFS object OID — has no FK constraint. If an `lfs_objects` row is deleted, its associated `file_segments` rows are orphaned.

### Hash algorithm mismatch in entity comments

The comments in `cas_block.rs`, `cas_chunk.rs`, and `lfs_chunk.rs` annotate the hash columns as "64-char hex BLAKE3", but the CAS store module (`src/cas/store.rs`) computes all hashes with `sha2::Sha256`. The database schema is agnostic (the column is `TEXT`), but the discrepancy between the entity comments and the actual algorithm used by the writer is misleading.

### All timestamps are bare `i64` Unix epoch integers

No `DateTime` or `chrono` types appear in any entity. Timestamps across all tables are stored and retrieved as raw `i64` values (seconds since Unix epoch). Time zone, precision, and formatting are all delegated to callers.

### Token security design

Access tokens are stored only by their SHA-256 hash (`token_hash`) and a short display prefix (`token_prefix`). The raw `ox_*` token is returned to the client exactly once at creation time and never persisted. Soft-delete via `is_active = false` allows revocation without data loss; a permanent hard-delete path also exists in the API layer.

### `#[allow(unused_imports)]` on all entity re-exports

Every `pub use … Entity as …;` line in `entities/mod.rs` is annotated with `#[allow(unused_imports)]`. This suppresses warnings for entities that are declared but not yet consumed by any other module, indicating the schema is ahead of the implemented query layer in some areas.

## Open questions

- `org_member::Model` has two `belongs_to` variants in its `DeriveRelation` enum (`Organization` and `User`) but implements no `Related<T>` blocks. Is this intentional (callers use raw queries), or was the `impl Related` omitted accidentally?
- `discussions`, `repo_likes`, and `pull_requests` reference `repo_name TEXT` with no FK. What is the canonical format of this value — is it `"owner/repo"`, `"repo"`, or something else? Is referential integrity planned?
- `file_segments.file_hash` has no FK to `lfs_objects`. Is this intentional (file segments may outlive LFS object rows) or an oversight that could lead to orphaned rows?
- `lfs_chunk.chunk_hash` and `cas_block.hash` / `cas_chunk.hash` are annotated as "BLAKE3" in source comments but the CAS store uses SHA-256. Which algorithm is authoritative, and will the comments or the store be corrected?
- `git_objects.object_type` is a raw `i32` discriminant. Is a Rust enum planned to make invalid object types unrepresentable, or will callers continue to cast integers manually?
- `org_member.role` is a free-form `TEXT` column. The comment lists `"owner"`, `"admin"`, `"member"` as the intended values — is enforcement (a `CHECK` constraint or a Rust enum) planned?
- `access_tokens.scopes` is a comma-separated `TEXT` column. Is a normalised scopes table or a well-defined token set being considered, or is the free-form comma list the long-term design?
- `create_tables` runs unconditionally on every server start. For a production deployment with a large database, is this acceptable, or is a proper migration system (SeaORM migrations or similar) planned?
- The `ALTER TABLE … ADD COLUMN` migration lines for `password_hash` and `is_org` are executed without checking whether the column already exists (relying on SQLite silently failing the `ALTER TABLE`). Are additional in-place migrations expected to follow the same pattern, and is this considered the migration strategy going forward?
- No table stores repository-level metadata such as visibility (`public`/`private`), description, or tags. Is this metadata intended to live elsewhere (e.g., in-memory only), or is schema expansion planned?
