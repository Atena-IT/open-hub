# Error Comparison

## Metadata
- Module: `error`
- OpenXet map: `round1/openxet_error.md`
- Local anchors:
  - `crates/common/src/error.rs`
  - `crates/hub-api/src/auth.rs`
  - `crates/hub-api/src/routes/repos.rs`
  - `crates/hub-api/src/routes/files.rs`
  - `crates/hub-api/src/routes/lfs.rs`
- Author: Claude
- Status: review-ready

## What OpenXet does
- Uses a small HTTP-oriented error taxonomy and plain-text response mapping.
- Keeps error shaping simple and relatively protocol-agnostic.
- Does not embed richer HF-specific response headers or response envelopes.

## What xet-backend currently does
- Uses a richer `AppError` shape with JSON bodies, header enrichment, and targeted not-found classification for HF-compatible client behavior.
- Logs errors centrally and is optimized for Hub API expectations.
- Does not yet have a differentiated Git-native error path because there is no Git Smart HTTP surface yet.

## Gap inventory
| Area | OpenXet behavior | Current behavior | Gap type | Notes |
| --- | --- | --- | --- | --- |
| Error envelope | Plain-text/simple status mapping | JSON + HF-specific headers | routing, auth | Current behavior fits HF clients better |
| Protocol differentiation | One simple HTTP-facing style | Hub-optimized style only | protocol, routing | Will matter if Git transport is added |
| Error classification | Caller chooses simple variants | More opinionated centralized shaping | routing | Stronger for current Hub compatibility |

Gap type uses one or more of: `protocol`, `persistence`, `storage`, `routing`, `auth`, `ui`.

## Recommendation
- Decision: **extend ours**
- Why this is the best next move:
  - `xet-backend`’s current error shaping is better aligned with the existing HF compatibility goal.
  - The missing work is not to simplify toward OpenXet, but to add transport-aware behavior later if Git clients are introduced.
  - A future Git-native surface may need a second response-shaping mode, but the current Hub-mode error stack should be preserved.
- Dependencies or blockers:
  - Git transport decision in `compare_git.md`
  - API surface evolution in `compare_api.md`

## Verification to design later
- HF error regression tests for repo/revision/file not found semantics
- Transport-aware error tests once Git Smart HTTP is added
- LFS and Xet auth failure-shape tests

## Evidence
- OpenXet anchors: `round1/openxet_error.md`
- xet-backend anchors:
  - `crates/common/src/error.rs`
  - `crates/hub-api/src/auth.rs`
  - `crates/hub-api/src/routes/repos.rs`
  - `crates/hub-api/src/routes/files.rs`
  - `crates/hub-api/src/routes/lfs.rs`
