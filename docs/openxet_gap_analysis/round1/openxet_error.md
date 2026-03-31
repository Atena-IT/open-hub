# OpenXet Error Module Map

## Metadata
- Module: `error`
- Source paths:
  - `../OpenXet-reference/src/error.rs`
- Author: Claude
- Status: review-ready

## Build shape
- A single-file module defining `ServerError`, `IntoResponse`, and a `Result<T>` alias.
- Uses `thiserror` for typed errors and Axum’s `IntoResponse` for HTTP translation.

## Responsibilities
- Classify server failures into a small domain-specific error taxonomy.
- Map those failures onto HTTP status codes and response bodies.
- Provide a uniform error type consumed by Git, API, and handler code.

## Explicit non-goals
- No structured JSON error envelopes, custom headers, or rich machine-readable error codes.
- No logging or tracing inside the error module itself.
- No deep error chaining beyond the small enum surface.
- No product-specific response shaping for different clients.

## Key contracts
### Types / entities
- `ServerError` variants cover not-found, repo/ref/object failures, auth failures, invalid requests/paths, internal errors, and IO.
- `Result<T>` is a simple alias over `ServerError`.

### Routes / protocol surfaces
- The module’s main outward contract is HTTP status/body mapping via `IntoResponse`.
- Git, API, and auth code can return `ServerError` directly and let Axum translate it.

### Storage / persistence contracts
- No direct persistence contracts; DB/storage-specific failures are usually wrapped higher up and funneled into `Internal` or related variants.

## Integration points
- Upstream: used by API, auth, and Git code.
- Downstream: depends only on Axum and `thiserror`.
- Adjacent: sets the baseline HTTP error semantics seen by protocol and browser clients.

## Runtime and configuration assumptions
- Assumes Axum is the HTTP layer.
- Assumes no environment-dependent error formatting or debug/release behavior split.
- Assumes callers choose the correct error variant rather than relying on central classification logic.

## Questions for comparison round
- How do OpenXet’s plain-text error responses compare to `xet-backend`’s current JSON/error-header behavior for HF compatibility?
- Which error classification choices in `xet-backend` are stronger for Hub clients, and where are OpenXet’s simpler semantics sufficient for Git/LFS flows?
- If Git-native transport is added to `xet-backend`, should it reuse current Hub-style error shaping or adopt a more OpenXet-like split?
- Where are auth, ref, and not-found errors most sensitive to client expectations across Git, HF, and browser surfaces?

## Evidence
- `../OpenXet-reference/src/error.rs`
- `../OpenXet-reference/src/api/auth.rs`
- `../OpenXet-reference/src/git/protocol.rs`
