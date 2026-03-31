# Round 2 Comparison — `error`

**OpenXet commit:** `a5b6dfcdf5e806a5a29cb893f0d79d733bea309b`
**Analyst:** DiTo97
**Date:** 2026-03-31
**Tracking issue:** [#54](https://github.com/Atena-IT/open-hub/issues/54)
**Round 1 source:** `round1/openxet_error.md` (not present on this branch; the Round 1 document lives on the `worktree-issue-45-openxet-error-map` branch)

---

## Scope

This document compares the OpenXet `error` module (mapped in Round 1) against the corresponding implementation in `xet-backend`. Its goal is to enumerate gaps, alignment, and surfaces that need decision.

## xet-backend counterpart

The error module in xet-backend lives in the `common` crate as a single file. It is re-exported at the crate root and consumed by both `hub-api` and `cas-server`.

| Responsibility | xet-backend location | Notes |
| --- | --- | --- |
| Unified error enum | `crates/common/src/error.rs` (`AppError`) | 8 variants vs. OpenXet's 13 |
| `IntoResponse` impl | `crates/common/src/error.rs:35` | Returns JSON body, not plain text |
| Result type alias | (none) | No crate-level `Result<T>` alias; handlers use `Result<..., AppError>` explicitly |
| Error classification header | `crates/common/src/error.rs:72` (`classify_not_found_error`) | Adds `X-Error-Code` header for NotFound variants |
| Blanket conversion from anyhow | `crates/common/src/error.rs:84` (`From<anyhow::Error>`) | Maps any anyhow error to `Internal` |

xet-backend also defines two local-only variants with no direct OpenXet counterpart: `RangeNotSatisfiable` (416) and `Config` (500). They are xet-backend extensions, not coverage of upstream `error` surfaces.

## Surface comparison

Status values: `covered` | `partial` | `missing` | `out-of-scope`

| OpenXet symbol / responsibility | Status | xet-backend equivalent | Notes |
| --- | --- | --- | --- |
| `ServerError` enum | `covered` | `AppError` in `crates/common/src/error.rs:9` | Different name, different variant set |
| `Result<T>` type alias | `missing` | (none) | xet-backend does not define a crate-level Result alias; each handler spells out `Result<..., AppError>` |
| `IntoResponse` impl | `covered` | `crates/common/src/error.rs:35` | Both implement the trait; response format differs (JSON vs. plain text) |
| `NotFound` variant | `covered` | `AppError::NotFound(String)` | Same semantics, same HTTP 404 |
| `RepoNotFound(String)` variant | `partial` | `AppError::NotFound(String)` | xet-backend folds repo-not-found into generic `NotFound`; the `classify_not_found_error` function infers `RepoNotFound` from the message string at response time via `X-Error-Code` header |
| `RepoAlreadyExists(String)` variant | `covered` | `AppError::Conflict(String)` | Same HTTP 409; different variant name |
| `InvalidRef(String)` variant | `partial` | `AppError::BadRequest(String)` | No dedicated variant; ref validation errors use generic `BadRequest` |
| `ObjectNotFound(String)` variant | `partial` | `AppError::NotFound(String)` | Folded into generic `NotFound`; `classify_not_found_error` infers `EntryNotFound` from message |
| `GitProtocol(String)` variant | `missing` | (none) | xet-backend has no Git Smart HTTP protocol layer; no equivalent error surface |
| `AuthRequired` variant (401) | `covered` | `AppError::Unauthorized(String)` | Both map to HTTP 401 |
| `AuthFailed` variant (403) | `partial` | `AppError::Unauthorized(String)` | xet-backend has no dedicated `AuthFailed` variant; both "auth required" and "auth failed" collapse into `Unauthorized` (401), diverging from OpenXet's 403 |
| `PermissionDenied` variant (403) | `covered` | `AppError::Forbidden(String)` | Both map to HTTP 403 |
| `InvalidRequest(String)` variant | `covered` | `AppError::BadRequest(String)` | Same HTTP 400 |
| `InvalidPath(String)` variant | `partial` | `AppError::BadRequest(String)` | No dedicated variant; path errors use generic `BadRequest` |
| `Internal(String)` variant | `covered` | `AppError::Internal(String)` | Both map to HTTP 500; xet-backend exposes the message in JSON body; OpenXet masks it |
| `Io(std::io::Error)` variant | `missing` | (none) | No `#[from]` IO conversion; IO errors are converted manually to `AppError::Internal` at call sites |
| 5xx message masking | `missing` | (none) | xet-backend returns the actual error message in the JSON body for all variants including `Internal` and `Config` |
| No-logging inside module | `partial` | `crates/common/src/error.rs:55` | xet-backend logs via `tracing::error!` inside `IntoResponse` for errors routed through `AppError`; however, the CAS JWT middleware (`crates/cas-server/src/middleware.rs`) returns bare `StatusCode::UNAUTHORIZED` on auth failure, bypassing `AppError` and its logging entirely. OpenXet leaves logging to callers |

## Gaps

### Missing

- **`Result<T>` type alias.** OpenXet defines `pub type Result<T> = std::result::Result<T, ServerError>` and uses it across all modules. xet-backend has no equivalent alias; every handler writes out the full `Result<..., AppError>` type. This is ergonomic, not behavioral.

- **`GitProtocol(String)` variant.** OpenXet uses this for Git Smart HTTP protocol-level errors (unsupported delta types, malformed packs). xet-backend has no Git Smart HTTP layer and therefore has no counterpart. This gap will become relevant only if xet-backend adds native git protocol support.

- **`Io(#[from] std::io::Error)` variant with automatic `From` conversion.** OpenXet allows `?` on `std::io::Error` to produce `ServerError::Io`. xet-backend requires explicit `.map_err(|e| AppError::Internal(e.to_string()))` at every IO call site. The automatic conversion is a convenience pattern, not a behavioral difference, but it means xet-backend loses the original `std::io::Error` source chain.

- **5xx error message masking.** OpenXet's `IntoResponse` for `Internal` and `Io` variants always returns the static body `"Internal server error"`, hiding implementation details from clients. xet-backend's `IntoResponse` returns the full error message as `{"error": "<actual message>"}` for all variants, including `Internal` and `Config`. This is a security-relevant divergence: internal error messages (database errors, S3 SDK errors, password-hash failures) are currently exposed to API clients.

### Partial

- **`RepoNotFound` / `ObjectNotFound` — collapsed into generic `NotFound`.** OpenXet has separate typed variants (`RepoNotFound`, `ObjectNotFound`), making call sites self-documenting and the `IntoResponse` mapping trivial. xet-backend collapses all not-found cases into `AppError::NotFound(String)` and then uses `classify_not_found_error` to reverse-engineer the sub-type from the message string at response time. This is fragile: if a message does not contain the expected substring ("repo" or "revision"), the classification defaults to `EntryNotFound` regardless of the actual error. Any message wording change could silently change the `X-Error-Code` header value.

- **`InvalidRef` / `InvalidPath` — collapsed into generic `BadRequest`.** OpenXet has dedicated variants for invalid refs and invalid paths, each producing HTTP 400 with a specific `Display` message. xet-backend uses `AppError::BadRequest(String)` for both, losing the semantic distinction at the type level.

- **`AuthFailed` semantics diverge from upstream.** OpenXet maps `AuthFailed` to HTTP 403, distinguishing it from `AuthRequired`. xet-backend has no dedicated `AuthFailed` variant; both "auth required" and "credentials wrong" cases map to `AppError::Unauthorized` (401). This collapses two distinct failure modes into one variant and returns a different status code than OpenXet. Whether the 401/403 split is more HTTP-correct depends on interpretation — HTTP 401 signals missing or invalid credentials, while HTTP 403 signals the server understood the credentials but refuses access — and the right choice may depend on client expectations.

- **Logging location differs.** OpenXet keeps the error module as a pure data/response-conversion layer and leaves logging to callers. xet-backend logs via `tracing::error!` inside `AppError::IntoResponse`, but this only applies to errors routed through `AppError`. The CAS JWT middleware (`crates/cas-server/src/middleware.rs`) returns bare `StatusCode::UNAUTHORIZED` on auth failure, bypassing `AppError` and its logging. This means CAS auth failures are not logged through the centralized error path.

## Already covered

- **`IntoResponse` trait implementation.** Both systems implement axum's `IntoResponse` to convert their error enum into HTTP responses. The mechanism is identical; only the output format differs (JSON vs. plain text).

- **`thiserror::Error` derive.** Both error enums use `thiserror` for `Display` and `std::error::Error` implementation.

- **409 Conflict for duplicate repos.** `AppError::Conflict` covers the same semantics as `ServerError::RepoAlreadyExists`.

- **Blanket `anyhow::Error` conversion.** xet-backend's `From<anyhow::Error>` covers the same use case as OpenXet's manual `.map_err(|e| ServerError::Internal(e.to_string()))` pattern, with less boilerplate.

## Out-of-scope items

- **`GitProtocol(String)` variant.** xet-backend does not implement Git Smart HTTP and therefore has no need for git protocol error classification. Per the project roadmap, the Hub layer is compatibility-oriented behavior over the existing CAS storage model, not a fully featured Git server.

- **Plain-text response body format.** OpenXet returns plain-text error bodies for all variants. xet-backend already returns JSON, which is the expected format for HF-compatible API clients. Adopting OpenXet's plain-text format would be a regression.

## Recommendations for synthesis

- **Mask 5xx error messages.** The most significant security gap is that `AppError::Internal` and `AppError::Config` currently expose raw internal error messages (DB errors, S3 SDK errors, Argon2 failures) to API clients in the JSON body. The `IntoResponse` implementation should return a generic `{"error": "Internal server error"}` for 500-class responses while preserving the detailed message in the `tracing::error!` log line. This aligns with OpenXet's approach and is standard practice.

- **Consider typed NotFound sub-variants.** The current `classify_not_found_error` approach of string-matching the message to assign `X-Error-Code` is fragile. Introducing explicit sub-variants (e.g., `RepoNotFound`, `RevisionNotFound`, `EntryNotFound`) would make the classification type-safe and resistant to message wording changes. Whether downstream clients depend on the `X-Error-Code` header values is an open question (see below).

- **Evaluate adding a `Result<T>` alias.** A `pub type Result<T> = std::result::Result<T, AppError>` in the `common` crate would reduce boilerplate across all handler signatures. Low-priority ergonomic improvement.

- **Decide on the 401/403 split.** xet-backend collapses `AuthRequired` and `AuthFailed` into a single `Unauthorized` (401) variant, while OpenXet distinguishes them (401 vs 403). This is a behavioral divergence whose correctness depends on client expectations. If downstream clients (or `huggingface_hub`) rely on 403 to distinguish "bad credentials" from "no credentials," xet-backend would need a dedicated variant. Otherwise, the current mapping is acceptable.

- **Implementation ordering.** The 5xx masking fix is independent of all other modules and can be done immediately. Typed NotFound sub-variants touch the `hub-api` and `cas-server` call sites and should be planned as a follow-up.

## Open questions

- The `X-Error-Code` header (`RepoNotFound`, `RevisionNotFound`, `EntryNotFound`) is generated by string-matching the `NotFound` message. Which downstream clients depend on this header, and is it part of the intended external contract or just an internal HF-compatibility mechanism? The answer determines whether the header contract needs formal documentation.

- xet-backend exposes full error messages for all status codes, including 500. Is this intentional for developer convenience during the current development phase, or is it an oversight?

- OpenXet's `AuthFailed` (credentials wrong) and `PermissionDenied` (valid credentials, insufficient access) both map to HTTP 403. xet-backend collapses the "credentials wrong" case into `Unauthorized` (401). Should xet-backend maintain this 401 mapping even if it diverges from OpenXet's behavior, or should it add a dedicated variant to match the upstream 403 semantics?

- The CAS server's JWT middleware (`crates/cas-server/src/middleware.rs`) returns bare `StatusCode::UNAUTHORIZED` (no body, no JSON envelope, no logging) on auth failure, bypassing `AppError` entirely. This is a concrete gap in error formatting and observability for the CAS internal API.
