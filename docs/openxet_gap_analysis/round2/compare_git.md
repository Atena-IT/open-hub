# Round 2 Comparison — `git`

**OpenXet commit:** `a5b6dfcdf5e806a5a29cb893f0d79d733bea309b`
**Analyst:** DiTo97
**Date:** 2026-03-31
**Tracking issue:** [#49](https://github.com/Atena-IT/open-hub/issues/49)
**Round 1 source:** [`round1/openxet_git.md`](../round1/openxet_git.md)

---

## Scope

This document compares the OpenXet `git` module (mapped in Round 1) against the corresponding implementation in `xet-backend`. Its goal is to enumerate gaps, alignment, and surfaces that need decision.

## xet-backend counterpart

xet-backend has **no dedicated git module**. There is no `crates/git/` crate and no git object model. The responsibilities covered by OpenXet's `git` module are spread across the `db-layer` and `hub-api` crates using a fundamentally different architecture: a **DB-primary flat-file model** backed by PostgreSQL and S3, rather than an **in-memory git object store** with async DB write-through.

| Responsibility | xet-backend location | Notes |
| --- | --- | --- |
| Repository registry | `crates/db-layer/src/queries/repositories.rs` | `RepoRow` in PostgreSQL; no in-memory `DashMap` registry |
| Repository CRUD | `crates/hub-api/src/routes/repos.rs` | `create_repo`, `delete_repo`, `repo_info` via Hub API |
| Object storage (blobs, trees, commits, tags) | **Not implemented** | No git object model; files stored in `repo_files` table + S3 |
| Object identity (SHA-1 `ObjectId`) | **Not implemented** | Commits use synthetic SHA-256 hashes; files use content SHA-256 |
| Commit creation | `crates/hub-api/src/routes/files.rs:206-374` | NDJSON-based commit via Hub API; writes `commits` DB row |
| Commit history traversal | `crates/db-layer/src/queries/commits.rs` | `get_latest_commit`, `find_commit_by_sha`; no graph walk |
| Tree structure | `crates/hub-api/src/routes/files.rs:78-130` | `build_tree_entries` derives virtual tree from flat `repo_files` rows |
| Ref management | `crates/db-layer/src/queries/repo_refs.rs` | `repo_refs` table with `name`, `ref_type`, `target_sha` |
| Ref resolution | `crates/hub-api/src/auth.rs:116-144` | `resolve_repo_revision`: checks `main` -> `head_sha` -> named refs |
| Branch/tag CRUD | `crates/hub-api/src/routes/repos.rs:350-500` | `create_branch`, `delete_branch`, `create_tag`, `delete_tag` |
| Git Smart HTTP protocol | **Not implemented** | No `info/refs`, no upload-pack, no receive-pack |
| Pack file format | **Not implemented** | No pack generation or parsing |
| Pkt-line codec | **Not implemented** | No pkt-line encoding or decoding |

## Surface comparison

Status values: `covered` | `partial` | `missing` | `out-of-scope`

| OpenXet symbol / responsibility | Status | xet-backend equivalent | Notes |
| --- | --- | --- | --- |
| `ObjectId` (20-byte SHA-1) | `missing` | N/A | xet-backend uses SHA-256 content hashes for files and synthetic SHA-256 hashes for commits; no SHA-1 anywhere |
| `ObjectType` (Blob, Tree, Commit, Tag) | `missing` | N/A | No git object type system; files are flat DB rows, commits are DB rows |
| `GitObject` (type + raw bytes) | `missing` | N/A | No in-memory git object representation |
| `GitRef` (name, target, symbolic) | `partial` | `crates/db-layer/src/queries/repo_refs.rs:4-13` (`RepoRefRow`) | DB-backed ref with `name`, `ref_type`, `target_sha`; no symbolic ref support; no `HEAD` ref row (HEAD is `repositories.head_sha`) |
| `Repository` (disk-backed, in-memory index) | `partial` | `crates/db-layer/src/queries/repositories.rs:4-16` (`RepoRow`) | PostgreSQL row with `head_sha`; no in-memory object index, no disk-backed object store, no `DashMap` |
| `RepositoryStore` (in-memory registry) | `partial` | `crates/db-layer/src/queries/repositories.rs` | All lookups go to PostgreSQL directly; no in-memory `DashMap` cache |
| `TreeEntry` (name, oid, mode, type) | `partial` | `crates/hub-api/src/routes/files.rs:55-71` (`TreeEntry`) | Virtual tree entries derived from flat `repo_files` rows; no git mode bits, no real oid for directories |
| `CommitInfo` (tree, parents, author, message) | `partial` | `crates/db-layer/src/queries/commits.rs:4-13` (`CommitRow`) | DB row with `sha`, `message`, `author_id`, `parent_sha`; no tree reference, no committer field, single parent only |
| `Repository::store_object` | `missing` | N/A | Files uploaded to S3 via `files.rs:commit`; no git object storage |
| `Repository::get_object` | `missing` | N/A | File content retrieved from S3 via `files.rs:resolve_file` |
| `Repository::has_object` | `missing` | N/A | No object existence check; file existence via `repo_files::find_file` |
| `Repository::update_ref` | `covered` | `crates/db-layer/src/queries/repo_refs.rs:40-58` | `create_ref` inserts a new ref; HEAD updated via `repositories::update_head_sha` |
| `Repository::delete_ref` | `covered` | `crates/db-layer/src/queries/repo_refs.rs:60-75` | `delete_ref` removes by repo_id + name + ref_type |
| `Repository::list_refs` | `covered` | `crates/db-layer/src/queries/repo_refs.rs:15-23` | `list_refs` returns all refs for a repo |
| `Repository::resolve_ref` | `covered` | `crates/hub-api/src/auth.rs:116-144` | `resolve_repo_revision` resolves `main` to `head_sha`, then checks named refs |
| `Repository::parse_tree` | `missing` | N/A | No git tree parsing; tree structure synthesized from flat DB file list |
| `Repository::walk_commits` | `missing` | N/A | Only `get_latest_commit` and `find_commit_by_sha`; no graph traversal |
| `Repository::create_commit` | `partial` | `crates/hub-api/src/routes/files.rs:343-363` | Creates a `commits` DB row with synthetic SHA-256 hash; no tree object, no real git commit format |
| `Repository::resolve_path` | `partial` | `crates/db-layer/src/queries/repo_files.rs:48-61` | `find_file` does exact-match path lookup in DB; no tree-walk |
| `Repository::initialize_empty` | `missing` | N/A | No initial empty commit/tree on repo creation; repo starts with no commits |
| `parse_repo_path` | `partial` | `crates/hub-api/src/routes/repos.rs:145-149` | Path splitting done inline in `create_repo`; no `.git` suffix stripping |
| `GitService` enum | `missing` | N/A | No Git Smart HTTP protocol; no upload-pack or receive-pack service type |
| `generate_ref_advertisement` | `missing` | N/A | No `info/refs` endpoint; ref listing is HF API only (`/api/.../refs`) |
| `handle_upload_pack` | `missing` | N/A | No git clone/fetch support; file download is via HF `resolve` endpoint only |
| `handle_receive_pack` | `missing` | N/A | No git push support; file upload is via HF NDJSON `commit` endpoint only |
| `CAPABILITIES` const | `missing` | N/A | No git protocol capabilities advertised |
| `PackObjectType` enum | `missing` | N/A | No pack file format support |
| `generate_pack` | `missing` | N/A | No pack file generation |
| `parse_pack` | `missing` | N/A | No pack file parsing |
| `pktline::encode` / `flush` / `delim` / `parse` / `parse_all` | `missing` | N/A | No pkt-line codec |

## Gaps

### Missing

- **Git Smart HTTP protocol.** xet-backend has no `info/refs`, `git-upload-pack`, or `git-receive-pack` endpoints. Standard git clients (`git clone`, `git push`, `git fetch`) cannot interact with the server. All file operations go through the HF Hub API (NDJSON commit for writes, `resolve` for reads).

- **Git object model.** No `ObjectId`, `ObjectType`, `GitObject`, or any representation of git objects (blobs, trees, commits, tags) as content-addressed data. Files are stored as flat rows in the `repo_files` table with S3 content, not as git tree/blob structures.

- **Pack file format.** No generation, parsing, or pkt-line codec. Without Smart HTTP protocol support these are not needed, but they are a prerequisite if git client interop is ever required.

- **SHA-1 object identity.** xet-backend uses SHA-256 throughout (content hashes for files, synthetic hashes for commits). There is no SHA-1 usage. This is arguably a strength (avoiding SHA-1 weaknesses), but it means the commit SHA produced by xet-backend is not a valid git commit hash.

- **Commit graph structure.** Commits are flat DB rows with a single `parent_sha` column. There is no support for merge commits (multiple parents), no tree-id reference in the commit, and no graph traversal (BFS or otherwise). `walk_commits` has no counterpart.

- **`initialize_empty`.** New repositories start with no commits and no objects. There is no initial empty tree/commit bootstrap, which sidesteps the OpenXet bug of orphaned commits on restart but also means a newly created repo has `head_sha = NULL`.

### Partial

- **Ref management.** Refs exist as `repo_refs` DB rows with `name`, `ref_type`, `target_sha`. Basic CRUD (create/list/delete) and resolution are implemented. However: (a) no symbolic ref support; (b) HEAD is stored as `repositories.head_sha` rather than as a ref pointing to `refs/heads/main`; (c) `main` branch is implicit and not stored in `repo_refs` — it is derived from `head_sha`; (d) no ref update (only create + delete); (e) `resolve_repo_revision` does not look up commit SHAs directly in the `commits` table, only named refs plus `head_sha`.

- **Commit creation.** The NDJSON commit handler in `files.rs` creates a `CommitRow` with a synthetic SHA-256 hash (`SHA256(timestamp:message)[..40]`). This has no relationship to git's commit hash algorithm. There is no tree object, no committer metadata, and no support for multi-parent merges.

- **Tree structure.** `build_tree_entries` in `files.rs` derives virtual directory/file entries from the flat `repo_files` table using path prefix matching. This approximates `ls-tree` output for the HF API, but (a) directories have no real object IDs (empty string), (b) there are no git mode bits, (c) tree structure is not versioned per-commit — it reflects the latest state of `repo_files` regardless of revision.

- **Repository model.** `RepoRow` captures repository identity (`id`, `owner_id`, `name`, `full_name`, `repo_type`, `head_sha`) in PostgreSQL. This covers the registry role of `RepositoryStore` but with a fundamentally different architecture: DB-primary with no in-memory cache, no object index, no local disk storage path.

- **Path resolution.** `repo_files::find_file` does exact-match path lookup, functionally equivalent to `resolve_path` for leaf files. However, it does not walk tree entries, cannot resolve intermediate directories to tree objects, and operates on the latest file state (not a specific commit's snapshot).

## Already covered

- **Ref CRUD operations.** `create_ref`, `delete_ref`, `list_refs`, and `find_ref_by_name` in `repo_refs.rs` provide the same logical operations as `Repository::update_ref`, `delete_ref`, `list_refs`, and `resolve_ref`. The implementation is DB-primary (synchronous, with error propagation) rather than in-memory-primary (fire-and-forget async DB writes), which is architecturally stronger for data integrity.

- **Revision resolution.** `resolve_repo_revision` in `auth.rs` resolves `"main"` to `head_sha`, checks if the revision matches `head_sha` directly, then falls back to named ref lookup. This covers the revision-resolution path exercised by the current `huggingface_hub` compatibility slice in this repo.

- **Branch and tag management.** The Hub API exposes `create_branch`, `delete_branch`, `create_tag`, `delete_tag` via `/api/{type}s/{owner}/{repo}/branch/{name}` and `/api/{type}s/{owner}/{repo}/tag/{value}`. These are fully implemented and tested.

- **Repository deletion cascades.** The `ON DELETE CASCADE` constraint on `repo_refs`, `repo_files`, `commits`, and `lfs_objects` ensures referential integrity when a repository is deleted. OpenXet's `RepositoryStore::delete_repo` removes from the in-memory map and fires an async DB delete, but does not cascade to related tables.

## Out-of-scope items

- **Git Smart HTTP protocol (upload-pack, receive-pack, ref advertisement).** xet-backend is explicitly a HuggingFace Hub-compatible server, not a general-purpose git server. File operations use the HF API (NDJSON commit, tree listing, resolve). The xet-core client handles CAS-level data transfer via the separate CAS protocol (`/v1/xorbs`, `/v1/shards`, `/v1/reconstructions`). Adding Git Smart HTTP would require implementing the full git object model first. Per the project roadmap, this is not a current compatibility target.

- **Pack file generation and parsing.** Without Git Smart HTTP, there is no current consumer for pack files in the compatibility slice exercised here.

- **Delta compression (OFS_DELTA, REF_DELTA).** Relevant only to Git Smart HTTP pack exchange.

- **SHA-1 object identity.** xet-backend's use of SHA-256 throughout is a deliberate design choice aligned with the Xet CAS protocol. Introducing SHA-1 would add a second hash namespace and bring known cryptographic weaknesses.

- **On-disk git object storage.** OpenXet's non-standard on-disk format (`[type_code][raw_data]`) is itself incompatible with standard git tooling. xet-backend's S3-backed storage avoids this problem entirely by not pretending to be a git object store.

## Recommendations for synthesis

- **The absence of Git Smart HTTP is not a gap for the current HF Hub compatibility target.** The compatibility slice exercised in this repo uses the HF API for commits, tree listing, and file resolution rather than native `git clone`/`git push` flows, and xet-backend implements that surface. This should be explicitly documented as a design boundary, not a missing feature.

- **Commit integrity.** xet-backend's synthetic commit SHA (`SHA256(timestamp:message)[..40]`) is a weak identifier: it has no relationship to the commit content, no tree reference, and is truncated. If commit SHAs are ever used for deduplication, caching, or cross-system references, they should be derived from actual commit content (files, parent, metadata). This is a low-priority improvement unless commit portability is needed.

- **Tree versioning per commit is absent.** The current `repo_files` table stores only the latest file state. There is no snapshot of the tree at a specific commit. This means `resolve_file` with a non-HEAD revision will still return the latest file content, not the file as it existed at that commit. This is a functional gap if multi-branch or historical file access is needed, but does not affect the single-branch `huggingface_hub` roundtrip.

- **Ref update vs. create-only.** The `repo_refs` table has `UNIQUE(repo_id, name)` but the query module only exposes `create_ref` and `delete_ref` — there is no `update_ref` to advance a branch pointer to a new commit. Currently, non-main branches are created pointing at a commit but never advanced. An `update_ref` query would be needed if branch-level commit workflows are added.

- **xet-backend's DB-primary model avoids several OpenXet bugs.** The fire-and-forget async DB writes in OpenXet (`tokio::spawn(async { let _ = ... })`) silently drop errors. xet-backend's synchronous DB writes with error propagation avoid: (a) the `repo_id = None` race in `create_repo`; (b) silently skipped object/ref inserts; (c) orphaned commits from `initialize_empty` on every restart. This is an architectural advantage.

- **If Git Smart HTTP is ever required**, it should be treated as a standalone project, not an incremental addition. It requires: git object model (SHA-1 ObjectId, blob/tree/commit/tag objects), a content-addressed object store, pack file generation/parsing, pkt-line codec, and side-band multiplexing. None of these components exist in xet-backend today.

## Open questions

- Is multi-branch file versioning (file content at a specific commit, not just the latest) a requirement for any planned `huggingface_hub` API endpoint? If so, the flat `repo_files` table needs a commit-level snapshot mechanism (e.g., a `commit_id` FK on `repo_files`, or a separate `commit_files` join table).

- Should the synthetic commit SHA be strengthened to be content-derived (e.g., `SHA-256(parent_sha + tree_manifest + author + message)`) for forward compatibility, or is the current `SHA-256(timestamp:message)` sufficient for the HF Hub compat scope?

- Does any downstream consumer (xet-core client, CI pipeline, model card tooling) depend on commit SHAs being stable across server restarts? The current algorithm includes a timestamp, so replaying the same commit at a different time produces a different SHA.

- Is there a plan to support `git clone`/`git push` against xet-backend for use cases beyond the `huggingface_hub` Python client (e.g., DVC, git-annex, or direct git workflows)?

- The OpenXet `RepositoryStore::load_from_db` rebuilds the full in-memory state from the database on every restart. xet-backend has no equivalent warm-up phase because it queries the DB on every request. If query latency becomes a concern at scale, should xet-backend introduce an in-memory caching layer (e.g., for hot repos or recently resolved refs)?
