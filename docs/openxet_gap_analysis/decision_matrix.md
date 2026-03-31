# OpenXet Gap Analysis Decision Matrix

This matrix is completed after Round 1 maps, Round 1.5 synthesis, and Round 2 comparisons.

## Rubric

Each area is judged against the same factors:
- protocol correctness
- fit with current Axum/sqlx/S3 workspace
- migration complexity
- backward compatibility with existing HF Hub behavior
- security/auth impact
- operational complexity
- testability and maintainability

## Per-area decisions

| Area | Foundation or adapter | Current gap summary | Protocol correctness | Workspace fit | Migration complexity | HF compatibility impact | Security/auth impact | Operational complexity | Testability | Recommendation | Initial phase |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| api | adapter | Strong HF adapter over a lightweight repo model; lacks Git-native transport assumptions | Depends on richer backend semantics | Strong fit with current Hub stack | Moderate once foundation exists | Must preserve current HF behavior | Moderate due to mixed client types | Moderate | Strong if cross-protocol tests are added | **extend ours** | 5 |
| git | foundation | Missing Git objects, Smart HTTP, pack handling, and richer ref/history semantics | OpenXet is much stronger here | Needs adaptation to current stack | High | High impact if done incorrectly | Moderate | High | Moderate once structure is chosen | **adapt/adopt theirs** | 3 |
| cas | foundation | xet-backend already has a stronger Xorb/Shard-specific CAS path | Current system is already strong | Strong fit | Low-to-moderate | Must not regress Xet/HF flows | Moderate | Moderate | Strong with current protocol tests | **keep current** | 4 |
| db | foundation | Current schema is operationally strong but lacks Git-native object/history depth | Needs richer repo/object/ref persistence | Strong fit with Postgres/sqlx | High but incremental | High | Moderate | Moderate | Strong with migrations + query tests | **extend ours** | 1 |
| storage | foundation | Strong S3 integration, but weak abstraction seam and incomplete range semantics | Needs better support for future Git/CAS reads | Strong fit with current S3 layer | Moderate | Moderate | Low-to-moderate | Moderate | Strong if seam/range tests are added | **extend ours** | 2 |
| web_ui | adapter | Current UI is intentionally minimal compared with OpenXet’s product-rich browser layer | Downstream of deeper model gaps | Current minimal UI fits scope | Moderate if expanded later | Low immediate impact | Moderate if browser auth grows | Moderate | Moderate | **keep current** | 6 |
| error | cross-cutting | Current error shaping is HF-optimized but not yet transport-aware for future Git clients | Strong for HF today; incomplete for mixed transports | Strong fit with current API stack | Low-to-moderate | High because HF client behavior depends on it | Moderate | Low | Strong | **extend ours** | 5 |
| main | adapter | Composition root is already clean; missing future Git route/state composition only | Good enough today | Strong fit | Low | Low immediate impact | Low | Low | Strong | **extend ours** | 5 |

## Proposed implementation order

1. repository persistence and object model
2. storage seam and range/read-path cleanup
3. Git transport and Git-native server behavior
4. LFS/CAS/storage integration under the richer Git model
5. Hub API, error, and runtime adaptation over the new backend semantics
6. web/admin surfaces
7. broader verification and support-doc updates

## Decision notes

### Preserve-and-evolve candidates
- `cas`

### Likely architecture-defining areas
- `git`
- `db`
- `storage`

### Likely adapter or seam-heavy areas
- `api`
- `main`
- `web_ui`
- `error`

### Working conclusion

The best path is **not** to migrate wholesale toward OpenXet. Instead:
- keep `xet-backend`’s stronger CAS/Xet path
- keep Postgres/sqlx/query-layer foundations
- adapt OpenXet’s Git-native structure and lessons where `xet-backend` currently has no equivalent layer
- preserve the current HF-compatible API surface and evolve it after the foundation work lands
