# HF Hub Compatibility Testing Roadmap

This document is the curated source of truth for `huggingface_hub` client compatibility work in this repository.

It distills the raw upstream test analysis from [issue #11](https://github.com/Atena-IT/xet-backend/issues/11), [issue #13](https://github.com/Atena-IT/xet-backend/issues/13), [issue #14](https://github.com/Atena-IT/xet-backend/issues/14), [issue #15](https://github.com/Atena-IT/xet-backend/issues/15), [issue #16](https://github.com/Atena-IT/xet-backend/issues/16), and the analysis-only [PR #12](https://github.com/Atena-IT/xet-backend/pull/12) into an ordered delivery queue that can be maintained on `master`.

## Operating rules

- Track compatibility work by user workflow, not by raw upstream file count.
- Keep exactly one active compatibility batch issue/branch/PR at a time.
- Update this roadmap and `resources/hf_hub_compat/checklist.json` in the same PR as the executable work.
- Reuse the existing pytest harness under `tests/integration/hf_hub/` and the `uv`-based CI entrypoint.
- Treat PR #12 as source material only; do not merge the raw inventory artifacts into mainline.

## Status model

- `done`: merged and passing in CI
- `in_progress`: currently active issue/branch/PR
- `planned`: queued work with defined scope
- `blocked`: intentionally paused behind a dependency or architectural gap
- `out_of_scope`: explicitly excluded from the current compatibility target

## Current sequence

| Batch | Status | Scope summary | Upstream modules | Issue | PR |
| --- | --- | --- | --- | --- | --- |
| Batch 1 | done | Core `HfApi` CRUD, basic `hf_hub_download`, metadata/cache basics, and core `snapshot_download` flows with pattern filters. | `tests/test_hf_api.py`, `tests/test_file_download.py`, `tests/test_snapshot_download.py` | [#17](https://github.com/Atena-IT/xet-backend/issues/17) | [#23](https://github.com/Atena-IT/xet-backend/pull/23) |
| Batch 2 | done | Deeper download and cache semantics for `hf_hub_download` and `snapshot_download`, including cache reuse and local-dir/cache edge cases that fit the current architecture. | `tests/test_file_download.py`, `tests/test_snapshot_download.py`, `tests/test_cache_layout.py`, `tests/test_utils_cache.py` | [#24](https://github.com/Atena-IT/xet-backend/issues/24) | [#25](https://github.com/Atena-IT/xet-backend/pull/25) |
| Batch 3 | done | Private repo enforcement, token/no-token behavior, and correct 401/403/404 semantics for protected resources. | `tests/test_hf_api.py`, `tests/test_file_download.py` | [#26](https://github.com/Atena-IT/xet-backend/issues/26) | [#27](https://github.com/Atena-IT/xet-backend/pull/27) |
| Batch 4 | done | Revision and history semantics, including commit SHA handling and non-HEAD lookup behavior. | `tests/test_hf_api.py`, `tests/test_snapshot_download.py` | [#28](https://github.com/Atena-IT/xet-backend/issues/28) | [#29](https://github.com/Atena-IT/xet-backend/pull/29) |
| Batch 5 | done | Lightweight named refs for branches and tags, plus revision-aware reads through those refs when they resolve to `main` or current head. | `tests/test_hf_api.py` | [#30](https://github.com/Atena-IT/xet-backend/issues/30) | [#31](https://github.com/Atena-IT/xet-backend/pull/31) |
| Batch 6 | in_progress | Basic `HfFileSystem` compatibility, README/RepoCard flows, and Xet/LFS compatibility headers and negotiation that fit the current storage model. | `tests/test_hf_file_system.py`, `tests/test_xet_upload.py`, `tests/test_xet_download.py`, `tests/test_repocard.py` | [#32](https://github.com/Atena-IT/xet-backend/issues/32) | [#33](https://github.com/Atena-IT/xet-backend/pull/33) |

## Batch details

### Batch 1 — core Hub CRUD and basic download workflows

**Status:** `done`

**What it covers**
- `HfApi` basics: `repo_exists`, `file_exists`, `revision_exists`, `whoami`, delete-missing repo behavior
- write-path happy paths: `upload_file`, `upload_folder`, `delete_file`, `create_commit`
- `hf_hub_download` against current head and `main`
- file metadata and cache basics via `get_hf_file_metadata` and `try_to_load_from_cache`
- `snapshot_download` happy paths, `local_dir`, `allow_patterns`, and `ignore_patterns`
- CI execution of the pytest-based compatibility slice

**Local test files**
- `tests/integration/hf_hub/test_hf_api_batch1.py`
- `tests/integration/hf_hub/test_file_download_batch1.py`
- `tests/integration/hf_hub/test_snapshot_download_batch1.py`
- `.github/workflows/integration-tests.yml`

**Server surfaces already exercised**
- `crates/hub-api/src/routes/repos.rs`
- `crates/hub-api/src/routes/files.rs`
- `crates/common/src/error.rs`

**Merged implementation**
- Issue: [#17](https://github.com/Atena-IT/xet-backend/issues/17)
- PR: [#23](https://github.com/Atena-IT/xet-backend/pull/23)

### Batch 2 — deeper download and cache semantics

**Status:** `done`
- Issue: [#24](https://github.com/Atena-IT/xet-backend/issues/24)
- PR: [#25](https://github.com/Atena-IT/xet-backend/pull/25)

**Target**
- extend `hf_hub_download` and `snapshot_download` coverage beyond batch-1 happy paths
- add cache reuse, metadata, and local-dir/cache edge cases that fit the current architecture
- keep the work focused on read-path compatibility rather than expanding unrelated API surface

**Likely local files**
- `tests/integration/hf_hub/test_file_download_batch2.py`
- `tests/integration/hf_hub/test_snapshot_download_batch2.py`
- `tests/integration/hf_hub/helpers.py` (narrow fixture/helper updates only if needed)

**Likely server surfaces**
- `crates/hub-api/src/routes/files.rs`

**Exit criteria**
- targeted batch-2 pytest modules pass locally with `uv`
- full `tests/integration/hf_hub` passes locally
- overlapping smoke downloads still pass
- CI is green for the batch PR

### Batch 3 — private repo and auth correctness

**Status:** `done`
- Issue: [#26](https://github.com/Atena-IT/xet-backend/issues/26)
- PR: [#27](https://github.com/Atena-IT/xet-backend/pull/27)

**Target**
- enforce private/public behavior consistently
- verify token/no-token behavior across protected reads and metadata calls
- return the correct 401/403/404 shapes expected by `huggingface_hub`

**Likely local files**
- `tests/integration/hf_hub/test_auth_batch3.py`
- `tests/integration/hf_hub/test_repo_visibility_batch3.py`

**Likely server surfaces**
- `crates/hub-api/src/routes/repos.rs`
- `crates/hub-api/src/routes/files.rs`
- auth-adjacent query/route files as required

**Exit criteria**
- targeted auth and visibility tests pass locally
- full `tests/integration/hf_hub` passes locally
- CI is green for the batch PR

### Batch 4 — revision and history semantics

**Status:** `done`
- Issue: [#28](https://github.com/Atena-IT/xet-backend/issues/28)
- PR: [#29](https://github.com/Atena-IT/xet-backend/pull/29)

**Target**
- verify commit SHA and revision correctness beyond always returning latest head
- accept `main` and the current head commit SHA as valid read revisions in the current storage model
- reject missing revisions instead of silently serving the latest head

**Likely local files**
- `tests/integration/hf_hub/test_revisions_batch4.py`

**Likely server surfaces**
- `crates/hub-api/src/routes/repos.rs`
- commit and file lookup paths involved in revision resolution

**Exit criteria**
- targeted revision tests pass locally
- full `tests/integration/hf_hub` passes locally
- CI is green for the batch PR

### Batch 5 — named refs: branches and tags

**Status:** `done`
- Issue: [#30](https://github.com/Atena-IT/xet-backend/issues/30)
- PR: [#31](https://github.com/Atena-IT/xet-backend/pull/31)

**Target**
- add lightweight named refs for branches and tags via `list_repo_refs`, `create_branch`, `delete_branch`, `create_tag`, and `delete_tag`
- allow branch/tag names as read revisions when they resolve to `main` or the current head commit in the current storage model
- keep pull-request refs and historical snapshots out of scope for this batch

**Likely local files**
- `tests/integration/hf_hub/test_refs_batch5.py`

**Likely server surfaces**
- `crates/hub-api/src/routes/repos.rs`
- `crates/hub-api/src/routes/files.rs`
- `crates/hub-api/src/auth.rs`
- `crates/db-layer/src/queries/repo_refs.rs`
- `crates/db-layer/migrations/003_repo_refs.sql`

**Exit criteria**
- targeted batch-5 named-ref tests pass locally
- full `tests/integration/hf_hub` passes locally
- Rust tests pass if server code changes
- CI is green for the batch PR

### Batch 6 — filesystem, README, and Xet compatibility

**Status:** `in_progress`
- Issue: [#32](https://github.com/Atena-IT/xet-backend/issues/32)
- PR: [#33](https://github.com/Atena-IT/xet-backend/pull/33)

**Target**
- basic `HfFileSystem` compatibility that maps to current repo read/write semantics
- simple RepoCard / README flows that operate through the existing file APIs
- Xet/LFS protocol awareness: token issuance, access control, transfer negotiation, and compatibility headers

**Likely local files**
- `tests/integration/hf_hub/test_hf_filesystem_batch6.py`
- `tests/integration/hf_hub/test_repocard_batch6.py`
- `tests/integration/hf_hub/test_xet_download_batch6.py`
- `tests/integration/hf_hub/test_xet_upload_batch6.py`

**Likely server surfaces**
- `crates/hub-api/src/routes/files.rs`
- `crates/hub-api/src/routes/xet_auth.rs`
- `crates/hub-api/src/routes/lfs.rs`
- `crates/hub-api/src/routes/repos.rs` only if tests prove a small response-shape mismatch

**Exit criteria**
- targeted batch-6 tests pass locally
- full `tests/integration/hf_hub` passes locally
- relevant legacy smoke scripts still pass when overlapping behavior changes
- Rust tests pass if server code changes
- CI is green for the batch PR

## Explicit non-goals

The following areas are not part of the current compatibility queue unless the roadmap is intentionally expanded later:

- Spaces
- Inference APIs and Inference Endpoints
- Jobs / compute workflows
- community/discussions beyond what is strictly necessary for repository compatibility
- buckets, unless they are later promoted into scope as a deliberate product goal

## Delivery loop for every batch after batch 1

1. Create a GitHub issue with exact scope, exclusions, upstream source modules, and exit criteria.
2. Create an issue-tied branch from updated `main`.
3. Open a draft PR immediately.
4. Implement only that batch’s pytest modules plus the minimum server fixes required.
5. Run focused batch tests, then full `tests/integration/hf_hub`, then overlapping smoke scripts when relevant.
6. Update this roadmap and `resources/hf_hub_compat/checklist.json` in the same PR.
7. Wait for GitHub Actions to go green.
8. Merge the PR and delete the branch.
9. Close the issue.
10. Move the pointer to the next planned batch and repeat.
