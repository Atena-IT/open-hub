# OpenXet Main Module Map

## Metadata
- Module: `main`
- Source paths:
  - `../OpenXet-reference/src/main.rs`
- Author: Claude
- Status: review-ready

## Build shape
- A single Tokio entrypoint that initializes logging, storage path, DB, app state, admin bootstrap, and the unified router before binding the server.
- Imports all top-level modules and is the composition root for the binary.

## Responsibilities
- Own startup ordering for logging, storage path resolution, DB initialization, state construction, admin bootstrap, router assembly, security headers, and listener binding.
- Merge the API, HF adapter, Git/LFS, and Web UI routers into one Axum application.
- Apply request limits and standard HTTP security headers.
- Bridge environment configuration into concrete runtime components.

## Explicit non-goals
- No multi-binary or microservice composition; everything is assembled into one process.
- No external orchestration of background workers or services beyond what `AppState` creates internally.
- No distributed coordination or clustering logic.
- No deep product logic; `main` wires components together but does not implement domain behavior itself.

## Key contracts
### Types / entities
- Relies on `AppState::with_db(...)` to return a fully wired application state.
- Uses the DB init function as the persistence bootstrap boundary.

### Routes / protocol surfaces
- Router merge order matters: API/HF/Git layers are composed before the Web UI.
- Applies large request-body limits and security headers at the application boundary.

### Storage / persistence contracts
- Resolves storage path and DB path up front, then loads repo/CAS/auth state before serving traffic.
- Ties together the optional/removable modules by providing the one canonical runtime wiring.

## Integration points
- Upstream: none; this is the binary entrypoint.
- Downstream: initializes `db`, `api`, `git`, `cas`, `storage`, `error`, and `web_ui`.
- Adjacent: a key adapter module rather than a domain foundation.

## Runtime and configuration assumptions
- Uses environment variables such as storage path, host, and port.
- Assumes the storage path is writable and the DB can be created/loaded during boot.
- Assumes a Tokio runtime and a single merged HTTP server process.
- Assumes startup is the correct place to create the default/admin bootstrap state.

## Questions for comparison round
- How closely does OpenXet’s single-binary composition root align with `xet-backend`’s current unified Axum/sqlx/S3 server wiring?
- Which startup and router-ordering patterns are directly reusable in `xet-backend`, and which depend on OpenXet’s different repo/data model?
- Does `xet-backend` need any new composition seams before Git-native transport can be integrated cleanly?
- Which concerns should remain in `main` versus moving into dedicated initialization modules as this epic grows?

## Evidence
- `../OpenXet-reference/src/main.rs`
- `../OpenXet-reference/src/api/mod.rs`
- `../OpenXet-reference/src/web_ui/mod.rs`
