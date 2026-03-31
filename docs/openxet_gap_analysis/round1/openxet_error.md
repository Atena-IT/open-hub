# Round 1 Module Map — `error`

**OpenXet commit:** `a5b6dfcdf5e806a5a29cb893f0d79d733bea309b`
**Analyst:** DiTo97
**Date:** 2026-03-31
**Tracking issue:** [#45](https://github.com/Atena-IT/open-hub/issues/45)

---

## Scope

This document maps the OpenXet `error` module. It covers only the upstream OpenXet source — no `xet-backend` references appear here. Round 2 will introduce the comparison.

## Source location

```
src/error.rs
```

The module is a single flat file declared as `mod error;` in `src/main.rs`. There are no sub-directories or nested modules.

Key entry points:
- `src/error.rs`

## Public surface

| Symbol | Kind | File | Brief description |
| --- | --- | --- | --- |
| `ServerError` | `enum` | `src/error.rs` | Unified error type for all server-side failures |
| `Result<T>` | `type` | `src/error.rs` | Alias for `std::result::Result<T, ServerError>` used throughout the codebase |

`ServerError` also implements `IntoResponse` (axum), which is an implicitly public integration point consumed by axum route handlers at compile-time.

## Internal structure

### Sub-modules

None. The module is a single `error.rs` file with no further decomposition.

### Key types

#### `ServerError`

A `thiserror`-derived enum with 13 variants covering the full error space of the server. Each variant maps to one or more HTTP status codes in the `IntoResponse` implementation.

| Variant | Payload | HTTP status |
| --- | --- | --- |
| `NotFound` | — | 404 |
| `RepoNotFound(String)` | repo identifier | 404 |
| `RepoAlreadyExists(String)` | repo identifier | 409 |
| `InvalidRef(String)` | ref name | 400 |
| `ObjectNotFound(String)` | object identifier | 404 |
| `GitProtocol(String)` | description | 400 |
| `AuthRequired` | — | 401 |
| `AuthFailed` | — | 403 |
| `PermissionDenied` | — | 403 |
| `InvalidRequest(String)` | description | 400 |
| `InvalidPath(String)` | path | 400 |
| `Internal(String)` | description (masked in response) | 500 |
| `Io(#[from] std::io::Error)` | underlying IO error (masked) | 500 |

`Io` uses `#[from]` for automatic conversion via the `?` operator from `std::io::Error`.

#### `Result<T>`

A crate-level type alias re-exported publicly:

```rust
pub type Result<T> = std::result::Result<T, ServerError>;
```

All internal modules import `crate::error::{Result, ServerError}` and use this alias in their function signatures.

## Intra-module dependencies

The `error` module has no imports from other OpenXet modules. It depends only on external crates and the standard library.

| Dependency | What is used |
| --- | --- |
| `axum::http::StatusCode` | Maps variants to HTTP status codes in `IntoResponse` |
| `axum::response::{IntoResponse, Response}` | Trait impl that lets axum convert `ServerError` directly into an HTTP response |
| `thiserror::Error` (derive macro) | Generates `std::error::Error` impl and `Display` messages for each variant |
| `std::io::Error` | Source type for the `Io` variant via `#[from]` |

The following modules depend on `error` (reverse dependencies):

| Module | Symbols imported |
| --- | --- |
| `api::auth` | `Result`, `ServerError` |
| `api::handlers` | `Result`, `ServerError` |
| `cas::store` | `Result`, `ServerError` |
| `git::storage` | `Result`, `ServerError` |
| `git::pack` | `Result`, `ServerError` |
| `git::protocol` | `Result`, `ServerError` |

## Notable implementation details

- **Error message masking for 5xx responses.** The `IntoResponse` arm for `Internal` and `Io` both return the static string `"Internal server error"` regardless of the inner message. The detailed message is available through the `Display` impl (used in tracing) but is never sent to the client. This is the only place where the error payload diverges from `self.to_string()`.

- **`AuthFailed` vs `PermissionDenied` both map to 403.** `AuthRequired` is the only variant that yields 401. Callers must choose carefully between `AuthFailed` (credentials wrong) and `PermissionDenied` (credentials valid, access denied) since both produce the same status code in the HTTP response; the distinction is semantic and visible only in the body text.

- **Single `#[from]` conversion.** Only `std::io::Error` has automatic `From` conversion. All other external error types (database errors, S3 errors, etc.) are converted manually to `ServerError::Internal(e.to_string())` at the call site. This pattern is used consistently across `api::auth` and `cas::store`.

- **Response body format is plain text.** `(status, message).into_response()` produces a plain-text body, not JSON. Clients that expect `{"error": "..."}` will receive raw strings instead.

- **No tracing/logging inside the module.** Error logging is left entirely to callers and to axum's `TraceLayer`. The error module itself is a pure data/conversion layer.

## Open questions

- The response body is plain text for all error variants. Is this intentional for all endpoints, or are some consumers (e.g., the HF-compatible API) expected to wrap errors in a JSON envelope? If so, the current `IntoResponse` impl would need per-context overrides.

- `AuthFailed` and `PermissionDenied` both yield HTTP 403. HuggingFace Hub clients may distinguish between 401 (unauthenticated) and 403 (forbidden). Would returning 401 for `AuthFailed` and 403 only for `PermissionDenied` better match expected client behavior?

- There is no variant for rate-limiting (429) or service unavailability (503). Are these expected to surface as `Internal`, or will new variants be added as the feature set grows?

- The `Io` variant wraps `std::io::Error` but the inner message is masked in HTTP responses. If callers want to surface a meaningful message for IO errors (e.g., "disk full"), they must convert to `Internal(message)` manually before returning. Is this the intended pattern?
