# OpenXet Gap Analysis — GitHub Tracking

Issue-by-issue status for epic [#35](https://github.com/Atena-IT/open-hub/issues/35).

Status values: `done` | `in_progress` | `planned` | `blocked`

## Setup and governance

| Issue | Title | Status | Notes |
| --- | --- | --- | --- |
| [#36](https://github.com/Atena-IT/open-hub/issues/36) | Bootstrap gap-analysis docs and reference-clone policy | `in_progress` | Scaffold drafted locally; pending commit/PR. |
| [#37](https://github.com/Atena-IT/open-hub/issues/37) | Open draft PR and tracking checklist | `planned` | |
| [#38](https://github.com/Atena-IT/open-hub/issues/38) | Define analysis templates and naming rules | `in_progress` | Templates drafted locally; pending commit/PR. |

## Round 1: OpenXet module maps

Target directory: `docs/openxet_gap_analysis/round1/`

| Issue | Module | Status | Notes |
| --- | --- | --- | --- |
| [#39](https://github.com/Atena-IT/open-hub/issues/39) | Map OpenXet `api` | `planned` | Output: `docs/openxet_gap_analysis/round1/openxet_api.md` |
| [#40](https://github.com/Atena-IT/open-hub/issues/40) | Map OpenXet `git` | `planned` | Output: `docs/openxet_gap_analysis/round1/openxet_git.md` |
| [#41](https://github.com/Atena-IT/open-hub/issues/41) | Map OpenXet `cas` | `planned` | Output: `docs/openxet_gap_analysis/round1/openxet_cas.md` |
| [#42](https://github.com/Atena-IT/open-hub/issues/42) | Map OpenXet `db` | `planned` | Output: `docs/openxet_gap_analysis/round1/openxet_db.md` |
| [#43](https://github.com/Atena-IT/open-hub/issues/43) | Map OpenXet `storage` | `planned` | Output: `docs/openxet_gap_analysis/round1/openxet_storage.md` |
| [#44](https://github.com/Atena-IT/open-hub/issues/44) | Map OpenXet `web_ui` | `planned` | Output: `docs/openxet_gap_analysis/round1/openxet_web_ui.md` |
| [#45](https://github.com/Atena-IT/open-hub/issues/45) | Map OpenXet `error` | `planned` | Output: `docs/openxet_gap_analysis/round1/openxet_error.md` |
| [#46](https://github.com/Atena-IT/open-hub/issues/46) | Map OpenXet `main` | `planned` | Output: `docs/openxet_gap_analysis/round1/openxet_main.md` |
| [#47](https://github.com/Atena-IT/open-hub/issues/47) | Synthesize cross-module dependency graph | `planned` | Output: `docs/openxet_gap_analysis/synthesis/openxet_cross_module_architecture.md` |

## Round 2: xet-backend vs OpenXet comparisons

Target directory: `docs/openxet_gap_analysis/round2/`

| Issue | Module | Status | Notes |
| --- | --- | --- | --- |
| [#48](https://github.com/Atena-IT/open-hub/issues/48) | Compare `api` | `planned` | Blocked on [#39](https://github.com/Atena-IT/open-hub/issues/39) |
| [#49](https://github.com/Atena-IT/open-hub/issues/49) | Compare `git` | `planned` | Blocked on [#40](https://github.com/Atena-IT/open-hub/issues/40) |
| [#50](https://github.com/Atena-IT/open-hub/issues/50) | Compare `cas` | `planned` | Blocked on [#41](https://github.com/Atena-IT/open-hub/issues/41) |
| [#51](https://github.com/Atena-IT/open-hub/issues/51) | Compare `db` | `planned` | Blocked on [#42](https://github.com/Atena-IT/open-hub/issues/42) |
| [#52](https://github.com/Atena-IT/open-hub/issues/52) | Compare `storage` | `planned` | Blocked on [#43](https://github.com/Atena-IT/open-hub/issues/43) |
| [#53](https://github.com/Atena-IT/open-hub/issues/53) | Compare `web_ui` | `planned` | Blocked on [#44](https://github.com/Atena-IT/open-hub/issues/44) |
| [#54](https://github.com/Atena-IT/open-hub/issues/54) | Compare `error` | `planned` | Blocked on [#45](https://github.com/Atena-IT/open-hub/issues/45) |
| [#55](https://github.com/Atena-IT/open-hub/issues/55) | Compare `main` | `planned` | Blocked on [#46](https://github.com/Atena-IT/open-hub/issues/46) |
| [#56](https://github.com/Atena-IT/open-hub/issues/56) | Synthesize gap/dependency order | `planned` | Output: `docs/openxet_gap_analysis/synthesis/gap_analysis_execution_order.md` |

## Decision round

| Issue | Decision | Status | Notes |
| --- | --- | --- | --- |
| [#57](https://github.com/Atena-IT/open-hub/issues/57) | Persistence / object model direction | `planned` | Blocked on [#56](https://github.com/Atena-IT/open-hub/issues/56) |
| [#58](https://github.com/Atena-IT/open-hub/issues/58) | Git transport / ref / history direction | `planned` | Blocked on [#56](https://github.com/Atena-IT/open-hub/issues/56) |
| [#59](https://github.com/Atena-IT/open-hub/issues/59) | LFS / CAS / storage direction | `planned` | Blocked on [#56](https://github.com/Atena-IT/open-hub/issues/56) |
| [#60](https://github.com/Atena-IT/open-hub/issues/60) | API / auth / web integration direction | `planned` | Blocked on [#56](https://github.com/Atena-IT/open-hub/issues/56) |
| [#61](https://github.com/Atena-IT/open-hub/issues/61) | Approve phased implementation roadmap | `planned` | Final gate before any implementation issues open |

## Update instructions

When an issue changes state, update the `Status` column in the relevant table above and add a brief note if needed. Keep this file in sync with the epic description in issue [#35](https://github.com/Atena-IT/open-hub/issues/35).
