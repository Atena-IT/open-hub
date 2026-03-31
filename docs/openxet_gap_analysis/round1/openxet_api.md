# OpenXet API Module Map

## Metadata
- Module: `api`
- Source paths:
  - `../OpenXet-reference/src/api/mod.rs`
  - `../OpenXet-reference/src/api/auth.rs`
  - `../OpenXet-reference/src/api/handlers.rs`
  - `../OpenXet-reference/src/api/lfs.rs`
  - `../OpenXet-reference/src/api/hf_api/mod.rs`
  - `../OpenXet-reference/src/api/hf_api/routes.rs`
  - `../OpenXet-reference/src/api/hf_api/types.rs`
  - `../OpenXet-reference/src/api/hf_api/handlers.rs`
- Author: Claude
- Status: review-ready

## Build shape
- `mod.rs` is the entry point and exports the router factories for the Git/LFS and HF-compatible API surfaces.
- `handlers.rs` defines `AppState` and the core request handlers for repo, auth, health, and CAS-adjacent endpoints.
- `auth.rs` owns password hashing, bearer-token generation, and namespace-aware permission checks.
- `lfs.rs` implements Git LFS batch/upload/download/verify and multipart flows.
- `hf_api/` is a thin Hugging Face compatibility layer with its own routes, DTOs, and handlers.

## Responsibilities
- Compose the HTTP-facing server API over the Git, CAS, DB, and storage subsystems.
- Normalize Git, LFS, and Hugging Face client requests onto the same repository model.
- Own request parsing, auth extraction, namespace checks, and response-shape compatibility.
- Initialize and pass around `AppState` containing `RepositoryStore`, `CasStore`, `AuthManager`, optional DB state, and the background CAS worker channel.

## Explicit non-goals
- It does not model separate model/dataset/space persistence; HF repo types are normalized onto a unified repo model.
- It does not make the HF layer deeply Git-native; many HF operations translate onto current repo/ref assumptions.
- It does not expose CAS internals through HF commit flows; the HF layer treats most files as Git- or LFS-level artifacts.
- It does not centralize all auth in middleware; many permission checks are handler-local.

## Key contracts
### Types / entities
- `AppState` is the central integration point for repos, CAS, auth, optional DB, and async background processing.
- `AuthManager` issues `ox_*` tokens, hashes passwords with Argon2id, and checks namespace write permissions.
- HF DTOs in `hf_api/types.rs` define request/response compatibility contracts such as `CreateRepoRequest`, `PreuploadRequest`, `WhoamiResponse`, and tree/file response shapes.

### Routes / protocol surfaces
- Git Smart HTTP routes are assembled via `git_router()` for ref advertisement, upload-pack, and receive-pack.
- LFS routes cover batch, object transfer, verify, and multipart upload negotiation.
- HF routes cover `whoami`, repo creation, preupload, commit, tree listing, resolve/download, and YAML validation.
- The HF resolve path deliberately proxies content instead of redirecting, so client libraries see the expected headers directly.

### Storage / persistence contracts
- The API layer depends on the Git/storage layer for object and ref persistence, and on CAS/LFS persistence for large-file workflows.
- HF commit handlers write Git objects and LFS pointers rather than invoking CAS directly.
- Auth and org membership depend on SeaORM-backed DB entities, but the handler surface can still run with optional DB in some initialization modes.

## Integration points
- Inbound: `src/main.rs` merges `api::hf_router()` and `api::git_router()` before the Web UI router.
- Downstream: calls into `crate::git`, `crate::cas`, `crate::storage`, and `crate::db`.
- Adjacent modules: `auth.rs` bridges DB-backed identities to repo namespace checks; `hf_api` is an adapter layer over the core Git/LFS state.

## Runtime and configuration assumptions
- Expects a Tokio runtime and an initialized `AppState`.
- Uses `GIT_XET_STORAGE_PATH`, `HOST`, and `PORT` from the process environment through main initialization.
- Assumes the DB, when present, has already been initialized and loaded before handlers are called.
- Relies on router ordering so `/api/*` HF routes win before Git or Web UI routes.

## Questions for comparison round
- How much of OpenXet’s HF adapter logic can map onto `xet-backend`’s current `hub-api` crate without rewriting the repo model underneath?
- Where do OpenXet’s permissive auth shortcuts diverge from `xet-backend`’s more explicit Hub auth behavior?
- Which API surfaces are thin adapters over Git primitives versus places where OpenXet embeds product semantics that `xet-backend` currently lacks?
- How should LFS and HF commit flows interact with the richer CAS/Xet model that already exists in `xet-backend`?

## Evidence
- `../OpenXet-reference/src/api/mod.rs`
- `../OpenXet-reference/src/api/handlers.rs`
- `../OpenXet-reference/src/api/auth.rs`
- `../OpenXet-reference/src/api/lfs.rs`
- `../OpenXet-reference/src/api/hf_api/routes.rs`
- `../OpenXet-reference/src/api/hf_api/types.rs`
- `../OpenXet-reference/src/api/hf_api/handlers.rs`
- `../OpenXet-reference/src/main.rs`
