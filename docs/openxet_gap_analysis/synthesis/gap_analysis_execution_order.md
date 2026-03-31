# Synthesis — Gap Analysis Execution Order

**Analyst:** DiTo97
**Date:** 2026-03-31
**Tracking issue:** [#56](https://github.com/Atena-IT/open-hub/issues/56)
**Input documents:**
- [`synthesis/openxet_cross_module_architecture.md`](openxet_cross_module_architecture.md)
- [`round2/compare_api.md`](../round2/compare_api.md)
- [`round2/compare_git.md`](../round2/compare_git.md)
- [`round2/compare_cas.md`](../round2/compare_cas.md)
- [`round2/compare_db.md`](../round2/compare_db.md)
- [`round2/compare_storage.md`](../round2/compare_storage.md)
- [`round2/compare_web_ui.md`](../round2/compare_web_ui.md)
- [`round2/compare_error.md`](../round2/compare_error.md)
- [`round2/compare_main.md`](../round2/compare_main.md)

---

## Purpose

This document synthesizes the eight Round 2 module comparisons and the cross-module architecture synthesis into a cross-module dependency order and epic sequencing recommendation. It identifies which gaps must be closed before others, which are independent, and which depend on architectural decisions that the later decision-round issues (#57-#60) must resolve first. The output is an execution ordering, not a decision matrix: it answers "in what order can work proceed" rather than "what should be built."

## Methodology

Each Round 2 comparison document was analyzed for: (1) gaps classified as `missing` or `partial`, (2) explicit dependency relationships between gaps (e.g., "requires session auth before any state-changing web UI handler"), (3) cross-module references where a gap in one module blocks or enables work in another, and (4) the "recommendations for synthesis" section which often names sequencing constraints directly.

Gaps were then classified into dependency tiers by asking: "Can this gap be closed without first closing any other gap?" Tier 0 gaps have no prerequisite. Higher tiers depend on at least one gap in a lower tier being resolved first.

Architectural decision points were extracted where multiple comparisons converge on the same unresolved question, since these decisions constrain the order of downstream implementation work.

---

## Findings

### Architectural decisions that gate implementation

Four architectural decisions emerged as blocking constraints from multiple Round 2 comparisons. Each must be resolved before the dependent implementation tiers can proceed. These are inputs to decision issues #57-#60 and are labeled D1-D4 in the dependency graph below.

**D1: Storage trait abstraction** (referenced by: compare_storage.md, compare_cas.md)
- Should xet-backend introduce a `StorageBackend` trait analogous to OpenXet's, or continue with the concrete `S3Client` approach?
- Trade-off: Concrete S3Client is simpler and matches the current Docker Compose deployment. A trait enables mock-based unit tests and future cloud portability.

**D2: LFS-CAS integration** (referenced by: compare_cas.md, compare_db.md, compare_api.md)
- Should LFS objects be integrated into the CAS deduplication pipeline (server-side chunking into xorbs), or should LFS and CAS remain separate storage paths?
- Trade-off: The separate-paths model is simpler and avoids the unbounded-channel / fire-and-forget risks from OpenXet. Integration enables dedup of large model files uploaded via LFS.

**D3: Git object model vs. flat-file DB** (referenced by: compare_git.md, compare_db.md, compare_web_ui.md, compare_main.md)
- Should xet-backend adopt a git object model (blobs, trees, commits, tags as content-addressed objects) or continue with the flat `repo_files` + `commits` DB tables?
- Trade-off: The flat-file model is sufficient for the single-branch HF Hub roundtrip. A git object model is required for multi-branch file versioning, diff computation, and Git Smart HTTP. The flat model avoids the orphaned-commits and fire-and-forget bugs identified in OpenXet.
- Evidence: compare_git.md documents that xet-backend's tree view currently shows the latest file state regardless of the requested revision, which is a functional gap for multi-branch workflows.

**D4: Community features scope** (referenced by: compare_web_ui.md, compare_db.md, cross-module architecture synthesis)
- Are discussions, pull requests, likes, and trending in scope for xet-backend?
- Trade-off: These features have no API exposure in OpenXet (web UI only). Building them in xet-backend is greenfield work, not a parity gap. If in scope, they should be designed with referential integrity from the start.

### Cross-module dependency graph

The dependency graph below shows how gap-closure work items depend on each other. An arrow `A --> B` means "B cannot be started until A is resolved."

```
                ┌──────────────────────────────────────────────────┐
                │  TIER 0 -- No prerequisites, independent         │
                │                                                  │
                │  E0.1  5xx error message masking (error)         │
                │  E0.2  Security response headers (main)          │
                │  E0.3  Registration gating env var (main)        │
                │  E0.4  Admin bootstrap on startup (main)         │
                │  E0.5  S3Client::delete method (storage)         │
                │  E0.6  S3Client::get_range method (storage)      │
                │  E0.7  S3Client::list method (storage)           │
                │  E0.8  Typed NotFound sub-variants (error)       │
                │  E0.9  Result<T> type alias (error)              │
                │  E0.10 Chunk batch insert optimization (db)      │
                │  E0.11 Token expiry + soft-revocation (db+api)   │
                └───┬──────────────┬────────────┬──────────────────┘
                    │              │            │
                    ▼              ▼            ▼
        ┌───────────────┐  ┌────────────┐  ┌──────────────────────┐
        │ TIER 1         │  │ TIER 1     │  │ DECISIONS            │
        │                │  │            │  │                      │
        │ E1.1 Cookie-   │  │ E1.2 Range-│  │ [D1] StorageBackend  │
        │ based session  │  │ aware re-  │  │ trait decision       │
        │ auth (web_ui)  │  │ construct- │  │                      │
        │ [needs E0.11   │  │ ion (cas)  │  │ [D2] LFS-CAS        │
        │  for token     │  │ [needs     │  │ integration decision │
        │  infra]        │  │  E0.6]     │  │                      │
        └───┬────────────┘  └────────────┘  │ [D3] Git object      │
            │                               │ model decision       │
            ▼                               │                      │
        ┌───────────────┐                   │ [D4] Community       │
        │ TIER 2         │                   │ features scope       │
        │                │                   └──────────────────────┘
        │ E2.1 CSRF      │
        │ protection     │
        │ (web_ui)       │
        │ [needs E1.1]   │
        │                │
        │ E2.2 Login/    │
        │ signup submit  │
        │ [needs E1.1]   │
        └───┬────────────┘
            │
            ▼
        ┌───────────────────────────────────────────────────┐
        │ TIER 3 -- Depends on sessions + decisions          │
        │                                                   │
        │ E3.1  Token management UI (web_ui) [E2.2]         │
        │ E3.2  Web-based file editor (web_ui) [D3, E2.1]   │
        │ E3.3  Blob view page (web_ui) [E2.1]              │
        │ E3.4  Commit history page (web_ui) [E2.1]         │
        │ E3.5  Org management (api+web_ui) [E2.1]          │
        │ E3.6  Bearer tri-dispatch (api) [E1.1]            │
        │ E3.7  LFS background chunking (cas) [D2]          │
        │ E3.8  Branch creation from web UI [E2.1]          │
        └───┬───────────────────────────────────────────────┘
            │
            ▼
        ┌───────────────────────────────────────────────────┐
        │ TIER 4 -- Depends on Tier 3 + decisions            │
        │                                                   │
        │ E4.1  Discussion system (web_ui+db) [D4, E3.5]    │
        │ E4.2  Pull request system (web_ui+db) [D3, D4]    │
        │ E4.3  Diff engine (web_ui) [D3]                   │
        │ E4.4  Trending/likes (web_ui+db) [D4]             │
        │ E4.5  Git Smart HTTP protocol [D3]                │
        │ E4.6  Markdown rendering (web_ui) [E3.3]          │
        └───────────────────────────────────────────────────┘
```

### Decision ordering

The Round 2 comparisons converge on one dominant dependency pattern: the persistence/object-model choice sits underneath everything else. The decisions should be resolved in this order:

```
#57 persistence + object-model direction
        |
        v
#58 git transport + refs/history direction
        |
        +-------------------+
        |                   |
        v                   v
#59 LFS/CAS/storage     independent tactical
   direction            hardening items (Tier 0)
        |
        v
#60 API/auth/web integration direction
        |
        v
#61 phased implementation roadmap approval
```

| Decision | Depends on | Used by |
| --- | --- | --- |
| `#57` persistence + object-model | Round 2 findings (complete) | `#58`, `#59`, `#60`, `#61` |
| `#58` git transport + refs/history | `#57` | `#59`, `#60`, `#61` |
| `#59` LFS/CAS/storage | `#57`, partially `#58` | `#60`, `#61` |
| `#60` API/auth/web integration | `#57`, `#58`, `#59` | `#61` |
| `#61` phased implementation roadmap | `#57`-`#60` | implementation begins |

### Gap summary by module

| Module | Round 2 done | Gap severity | Missing count | Partial count | Key blocking gaps |
| --- | --- | --- | --- | --- | --- |
| `error` | yes | **low** | 3 (Result alias, GitProtocol, Io variant) | 4 | 5xx masking (E0.1), typed NotFound (E0.8) |
| `storage` | yes | **medium** | 7 (trait, get_range, delete, list, size, get_stream, file I/O) | 2 | get_range (E0.6), delete (E0.5), trait decision (D1) |
| `db` | yes | **medium** | 4 (git_objects, lfs_chunks, entity re-exports, preload) | 4 | git_objects depends on D3; token expiry (E0.11) |
| `cas` | yes | **medium** | 7 (server-side chunking, LFS pipeline, reconstruction, dedup index, stats, chunk/block types, streaming) | 3 | LFS-CAS integration (D2), range reconstruction (E1.2) |
| `api` | yes | **high** | 10 (Git Smart HTTP, orgs, sessions, tri-dispatch, LFS multipart, LFS URL signing, background worker, stats, admin bootstrap, token revoke) | 7 | Session auth (E1.1), org support (E3.5), Git Smart HTTP (E4.5) |
| `git` | yes | **high** | 10 (Smart HTTP, object model, pack format, SHA-1, commit graph, initialize_empty, pkt-line, ref advertisement, upload-pack, receive-pack) | 5 | All depend on D3; out-of-scope for current HF Hub compat target |
| `web_ui` | yes | **high** | 18 (session auth, CSRF, blob view, commits, editor, discussions, PRs, orgs, tokens, likes, trending, search, markdown, git tree, LFS annotation, diff, branch UI, tree ops) | 6 | Session auth (E1.1) blocks nearly everything |
| `main` | yes | **medium** | 6 (Git Smart HTTP, background worker, in-memory state, admin bootstrap, security headers, registration gate) | 3 | Security headers (E0.2), admin bootstrap (E0.4) |

### Implementation order recommendation

The following order minimizes blocking and maximizes parallelism. Items at the same tier can be worked in parallel.

**Tier 0 -- Immediate, no prerequisites:**

1. `E0.1` **Mask 5xx error messages** -- The `IntoResponse` implementation for `AppError::Internal` and `AppError::Config` currently exposes raw internal error messages (DB errors, S3 SDK errors, Argon2 failures) to API clients. Change to return `{"error": "Internal server error"}` for 500-class responses. Affects: `crates/common/src/error.rs`. Source: compare_error.md "Missing" section. Independent of all other work.

2. `E0.2` **Add security response headers** -- `X-Frame-Options: DENY`, `X-Content-Type-Options: nosniff`, `Referrer-Policy`, and a CSP via `SetResponseHeaderLayer` in the composition root. Affects: `crates/server/src/main.rs`. Source: compare_main.md "Missing" section.

3. `E0.5` **Add `S3Client::delete` method** -- Wraps `DeleteObject`. Required before any garbage collection, orphan cleanup, or retention policy work. Affects: `crates/s3-storage/src/lib.rs`. Source: compare_storage.md "Missing" section.

4. `E0.6` **Add `S3Client::get_range` method** -- Wraps the S3 `Range` header for partial object fetches. Prerequisite for range-aware reconstruction (E1.2). Affects: `crates/s3-storage/src/lib.rs`. Source: compare_storage.md "Missing" section.

5. `E0.8` **Introduce typed NotFound sub-variants** -- Replace the fragile `classify_not_found_error` string-matching with explicit `RepoNotFound`, `RevisionNotFound`, `EntryNotFound` variants on `AppError`. Affects: `crates/common/src/error.rs` and call sites in `hub-api` and `cas-server`. Source: compare_error.md "Partial" section.

6. `E0.11` **Add token expiry and soft-revocation** -- Add `expires_at` (TIMESTAMPTZ, nullable) and `is_active` (BOOLEAN, default true) columns to `access_tokens`. Add migration, query functions, and enforcement in `resolve_bearer_token`. Affects: `crates/db-layer/`, `crates/hub-api/src/auth.rs`. Source: compare_db.md "Partial" section and compare_api.md "Missing" section. Security-relevant for multi-user deployment.

7. `E0.3` **Registration gating env var** -- Add a `DISABLE_REGISTRATION` flag to `AppConfig`. When set, `/api/auth/register` returns 403. Affects: `crates/common/src/config.rs`, `crates/hub-api/src/routes/auth.rs`. Source: compare_main.md "Missing" section.

8. `E0.4` **Admin bootstrap on startup** -- Implement an `ensure_admin_user` function called from `main.rs` after pool creation. If no admin user exists, create one with a random password logged to `tracing::warn!`. Affects: `crates/server/src/main.rs`, `crates/hub-api/src/auth.rs`. Source: compare_main.md "Missing" section.

9. `E0.7` **Add `S3Client::list` method** -- Wraps `ListObjectsV2` for object enumeration. Lower priority than delete/get_range but needed for audit and GC tooling. Affects: `crates/s3-storage/src/lib.rs`. Source: compare_storage.md "Missing" section.

10. `E0.9` **Result type alias** -- `pub type Result<T> = std::result::Result<T, AppError>` in the `common` crate. Ergonomic, no behavioral change. Affects: `crates/common/src/error.rs`. Source: compare_error.md "Missing" section.

11. `E0.10` **Chunk batch insert** -- Replace row-by-row `upsert_chunks` loop with a single batch insert using `UNNEST` arrays. Affects: `crates/db-layer/src/queries/xorbs.rs`. Source: compare_storage.md "Recommendations" section.

**Tier 1 -- Depends on Tier 0 items:**

12. `E1.1` **Cookie-based session auth for web UI** -- Implement session token generation, a `token=` cookie with `HttpOnly; SameSite=Lax`, and session-to-user resolution in the web UI layer. Can use DB-backed sessions (leveraging the PostgreSQL pool already available) or in-memory. Depends on E0.11 for the token infrastructure pattern. Affects: `crates/web-ui/src/pages.rs`, `crates/hub-api/src/auth.rs`. Source: compare_web_ui.md "Missing" section. **This is the single largest blocker for web UI progress.**

13. `E1.2` **Range-aware reconstruction** -- Wire E0.6 (`get_range`) into the reconstruction endpoint to support `offset_into_first_range` and partial-file downloads. Depends on E0.6. Affects: `crates/cas-server/src/routes/reconstructions.rs`. Source: compare_cas.md "Recommendations" section.

**Tier 2 -- Depends on session auth (E1.1):**

14. `E2.1` **CSRF protection** -- Per-process ephemeral HMAC secret, timestamp-bound tokens in every state-changing form. Must accompany session auth. Depends on E1.1. Affects: `crates/web-ui/`. Source: compare_web_ui.md "Missing" section.

15. `E2.2` **Login and signup submit handlers** -- Replace "coming soon" stubs with actual credential verification and session creation. Depends on E1.1. Affects: `crates/web-ui/src/pages.rs`. Source: compare_web_ui.md "Partial" section.

**Tier 3 -- Depends on Tier 2 and/or architectural decisions:**

16. `E3.1` **Token management UI** -- Web page to list, create, and delete `ox_*` access tokens. Depends on E2.2 (requires authenticated session). Affects: `crates/web-ui/`, `templates/`. Source: compare_web_ui.md "Missing" section.

17. `E3.3` **Blob view page** -- File content viewer at `/:owner/:repo/blob/:ref/*path`. Depends on E2.1 (for auth-guarded views). Affects: `crates/web-ui/`, `templates/`. Source: compare_web_ui.md "Missing" section.

18. `E3.4` **Commit history page** -- Paginated commit list and single-commit detail view. Depends on E2.1. Affects: `crates/web-ui/`, `templates/`, may need `crates/db-layer/src/queries/commits.rs` enhancements. Source: compare_web_ui.md "Missing" section.

19. `E3.5` **Organization management** -- Org CRUD, membership management, org-scoped permissions. Depends on E2.1. Affects: `crates/hub-api/src/routes/`, `crates/db-layer/`, `crates/web-ui/`. Source: compare_api.md "Missing" section. Prerequisite for collaborative repos and the discussion system.

20. `E3.6` **Bearer tri-dispatch** -- Extend `resolve_bearer_token` to fall back from `ox_*` to `username:password` colon format and optionally session tokens. Depends on E1.1 (session token infrastructure). Affects: `crates/hub-api/src/auth.rs`. Source: compare_api.md "Missing" section.

21. `E3.7` **LFS background chunking** -- Implement the `Raw -> Processing -> Chunked` pipeline for LFS objects through the CAS layer. Depends on D2 (must decide whether LFS-CAS dedup is in scope). Affects: `crates/hub-api/src/routes/lfs.rs`, `crates/db-layer/`, potentially new background worker. Source: compare_cas.md "Missing" section.

22. `E3.8` **Branch creation from web UI** -- Route and handler for `POST /:owner/:repo/branches/new`. Depends on E2.1. Affects: `crates/web-ui/`. Source: compare_web_ui.md "Missing" section.

23. `E3.2` **Web-based file editor** -- Create/edit/rename/delete files from the browser, creating commits through the Hub API. Depends on D3 (must decide whether commits write to DB flat-files or git objects) and E2.1. Affects: `crates/web-ui/`, `templates/`. Source: compare_web_ui.md "Missing" section.

**Tier 4 -- Depends on Tier 3 and/or multiple decisions:**

24. `E4.1` **Discussion system** -- Full CRUD with threaded comments, close/reopen lifecycle. Depends on D4 (scope decision) and E3.5 (org support for author/moderator roles). Affects: new DB migrations, `crates/hub-api/`, `crates/web-ui/`. Source: compare_web_ui.md "Missing" section.

25. `E4.2` **Pull request system** -- PR CRUD, merge, ref tracking. Depends on D3 (git object model decision, since PR merge requires ref manipulation) and D4. Affects: new DB migrations, `crates/hub-api/`, `crates/web-ui/`. Source: compare_web_ui.md "Missing" section.

26. `E4.3` **Diff engine** -- LCS-based line-level diff for commit and PR views. Depends on D3 (diff source: git objects or DB file snapshots). Affects: `crates/web-ui/`. Source: compare_web_ui.md "Missing" section.

27. `E4.4` **Trending and likes** -- Repo like/unlike, trending page. Depends on D4 (scope). Affects: new DB migrations, `crates/web-ui/`. Source: compare_web_ui.md "Missing" section.

28. `E4.5` **Git Smart HTTP protocol** -- `info/refs`, `upload-pack`, `receive-pack`. Depends on D3 (requires a full git object model if adopted). Affects: would require a new `git` crate or major additions to `hub-api`. Source: compare_git.md "Missing" section. This is the largest single body of work and is out-of-scope for the current HF Hub compatibility target.

29. `E4.6` **Markdown rendering in repo detail** -- README display using `pulldown-cmark` + `ammonia` (dependencies already declared). Depends on E3.3 (blob view provides the rendering context). Affects: `crates/web-ui/`, `templates/repo_detail.html`. Source: compare_web_ui.md "Missing" section.

---

## Inputs to the decision round

The following summarizes what each decision issue needs to resolve, as derived from the Round 2 findings.

- **Issue #57 must decide the repository/persistence foundation.** Should xet-backend stay DB-primary with `repo_files`/`commits`/`repo_refs`, or should it adopt more of OpenXet's git-object / in-memory-primary model? This decision shapes every downstream module.

- **Issue #58 must decide transport and history scope.** Is Git Smart HTTP intentionally out of scope, or does the project want ref/history fidelity beyond the current HF-compatible API slice? This determines whether the `git` module's 10 missing items are gaps or deliberate exclusions.

- **Issue #59 must decide whether LFS and CAS stay parallel or converge.** Is the long-term direction direct-to-S3 LFS plus client-driven CAS, or server-side LFS chunking and richer storage abstractions? This gates E3.7 and the `lfs_objects` schema extensions.

- **Issue #60 must decide the adapter/auth/web layer over the chosen backend semantics.** That includes org support, session model, bearer-token behavior, web UI scope, and cross-surface error contracts. This is the broadest decision but the least load-bearing for ordering because its scope is determined by the answers to #57-#59.

- **Issue #61 should convert those answers into phases.** The roadmap should separate immediate hardening (Tier 0), foundation changes (Tiers 1-2), protocol changes (Tier 3), and optional UI/community follow-ups (Tier 4).

## Open questions

- E1.1 (session auth) is listed as depending on E0.11 (token expiry). In practice, session auth could be implemented with in-memory tokens and no DB-backed expiry. Should E1.1 wait for E0.11, or proceed independently?

- compare_main.md flags the missing explicit body limit (Axum defaults to 2 MiB). Since LFS uploads use presigned S3 URLs, is there any current endpoint that needs a larger limit? The NDJSON commit endpoint receives file content in-band for non-LFS files.

- compare_storage.md flags the missing `get_stream` method. `resolve_file` currently loads the full S3 object into memory. For large regular (non-LFS) files, this could cause OOM. Should streaming reads be elevated to Tier 0?

- compare_cas.md notes that xet-backend queries PostgreSQL for every dedup lookup while OpenXet uses an in-memory DashMap. Under high-frequency dedup traffic from xet-core clients, is PostgreSQL indexing sufficient, or does an in-memory cache need to be planned?

- compare_main.md asks whether the current merge order (CAS -> Hub API -> LFS -> Web UI) has precedence risks between HF API resolve routes (`/{owner}/{repo}/resolve/...`) and Web UI routes (`/{owner}/{repo}`). This needs verification as a correctness concern.

- Does any future target workflow require native git clone/fetch/push, or is the current HF-compatible API + xet-core CAS slice the permanent boundary?

- Is the `files/` object-storage namespace in xet-backend a permanent part of the architecture, or a transitional shim that would disappear if the object/persistence model changes?
