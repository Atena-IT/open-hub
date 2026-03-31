# OpenXet Cross-Module Architecture Synthesis

Status: review-ready

## Inputs
- `round1/openxet_api.md`
- `round1/openxet_git.md`
- `round1/openxet_cas.md`
- `round1/openxet_db.md`
- `round1/openxet_storage.md`
- `round1/openxet_web_ui.md`
- `round1/openxet_error.md`
- `round1/openxet_main.md`

## Terminology normalization

### Foundational terms
- **Repository model**: OpenXet is fundamentally Git-native. Repos, refs, objects, commits, and trees are first-class concepts, not compatibility veneers.
- **CAS model**: large-file content is handled by a separate CAS/LFS subsystem that chunks content, bundles chunks into blocks, and persists reconstruction metadata.
- **Storage backend**: a pluggable local/S3 abstraction used by both Git and CAS.
- **Persistence layer**: a single SeaORM/SQLite domain model that stores auth, repos, refs, objects, CAS metadata, and collaboration/product entities.

### Adapter terms
- **API layer**: HTTP adapters over the Git/CAS/Auth/DB foundations, including Git Smart HTTP, LFS, and HF-compatible routes.
- **Web UI layer**: browser-facing adapter over the same domain primitives and product entities.
- **Composition root**: `main.rs` wires the process together, sets router order, and applies global runtime policy.

### Cross-cutting term
- **Error layer**: a shared HTTP-oriented error taxonomy used by multiple modules but not itself a data or protocol foundation.

## Dependency graph

```text
main
├── api
│   ├── auth (inside api)
│   ├── git
│   ├── cas
│   ├── db
│   ├── storage
│   └── error
├── web_ui
│   ├── api state/auth helpers
│   ├── git
│   ├── db
│   ├── cas (selected stats/integration points)
│   └── error
├── git
│   ├── storage
│   ├── db
│   └── error
├── cas
│   ├── storage
│   ├── db
│   └── error
└── db / storage / error
```

## Foundational modules

### `git`
- Defines the repository/object/ref/history model.
- Exposes the primitives that higher-level routes and UI actions build on.
- Strongly influences what “repo compatibility” means across the system.

### `db`
- Encodes the product and protocol domain model directly in persistence.
- Establishes whether discussions, PRs, refs, objects, tokens, and CAS metadata are first-class persisted concepts.
- Is a major predictor of migration difficulty when comparing to `xet-backend`.

### `storage`
- Provides the concrete object-store seam that both Git and CAS rely on.
- Makes byte-range semantics and backend abstraction first-class.
- Sits below both Git-native and CAS-native paths.

### `cas`
- Separate from Git, but still foundational for large-file/Xet behavior.
- Owns chunking, block bundling, reconstruction, and LFS-object processing.
- Likely a preserve-and-evolve candidate when compared against `xet-backend`’s stronger existing CAS model.

## Adapter modules

### `api`
- Mostly a translation layer over underlying Git/CAS/Auth/DB primitives.
- Important for protocol compatibility, but many decisions here are downstream of the persistence and repo models.

### `web_ui`
- A product adapter that depends on the richer DB + Git model already existing underneath it.
- Large in scope, but unlikely to be the right starting point for implementation.

### `main`
- A composition adapter and runtime seam.
- Important for integration order, but not itself architecture-defining unless startup composition prevents deeper changes.

## Cross-cutting module

### `error`
- Shared contract for HTTP response shaping.
- Important for client compatibility and transport correctness, but not a primary driver of repo/object/history design.

## Architectural observations

1. **OpenXet starts from a Git-native core, not from an HF-compatible current-state file model.**
   - This is the biggest structural difference from current `xet-backend`.
2. **OpenXet’s DB schema already assumes richer product semantics.**
   - Discussions, PRs, tokens, orgs, refs, Git objects, and CAS metadata are all modeled explicitly.
3. **CAS is present but remains conceptually distinct from Git.**
   - The design is “Git foundation + CAS/LFS large-file subsystem,” not “CAS-first storage with Git compatibility layered over it.”
4. **The API and Web UI layers are downstream of the data/repo model.**
   - Their behavior is meaningful only because the repo/object/ref/history model already exists.
5. **`main` is cleanly unified.**
   - Router order, startup, and security policy are centralized, which should make comparison to `xet-backend` straightforward.

## Gaps or weak spots in Round 1 docs

- The first-pass maps are strong on structure and responsibilities, but the next round still needs direct side-by-side anchor work against `xet-backend` to determine whether similarities are conceptual or implementation-level.
- `main` and `error` are intentionally thinner modules; the real architectural weight for those areas will emerge only in comparison with `xet-backend`’s router/error handling.
- `web_ui` is broad and feature-rich, but many of its most important implications are actually DB- and Git-model implications rather than purely UI concerns.

## Ready-state for Round 2

Round 2 should treat the module families as:
- **Foundation-first comparison**: `git`, `db`, `storage`, `cas`
- **Adapter comparison**: `api`, `web_ui`, `main`
- **Cross-cutting comparison**: `error`

This ordering should also inform implementation sequencing later: compare and decide the data/repo/history foundation before attempting endpoint or browser parity.
