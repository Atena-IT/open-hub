# OpenXet Gap Analysis

This docs area is the source of truth for the OpenXet discovery workstream tracked by epic [#35](https://github.com/Atena-IT/open-hub/issues/35).

The goal is a structured comparison between `xet-backend` (this repo) and the upstream OpenXet implementation, producing a decision matrix and phased implementation roadmap for Git/LFS/Xet-native server expansion.

## Reference-only sibling clone policy

All OpenXet discovery work relies on a **local, read-only reference clone** of the upstream OpenXet repository.

### Rules

1. **Sibling placement.** Clone OpenXet at `../openxet/` relative to this repository root — i.e., as a sibling directory, not inside `xet-backend/`.
2. **Read-only use.** The clone is used for reading source code only. Do not push to it, do not open branches in it, do not modify tracked files.
3. **Not tracked here.** The OpenXet clone directory is never added to this repository's `.gitignore` exceptions or committed as a submodule. It remains a purely local artifact.
4. **One clone per workstation.** Keep a single clone; pull to update it before starting new analysis rounds. Do not create multiple clones at different paths on the same machine.
5. **No copy-paste of source.** Analysis documents quote short snippets for reference only. Do not copy OpenXet source files wholesale into this repository.
6. **Reproducibility.** Record the exact OpenXet commit SHA used for each analysis round in the corresponding round document so results are reproducible.

### Setup

Confirm the canonical upstream OpenXet reference repository URL before cloning, then place it at the agreed sibling path:

```bash
# From the xet-backend parent directory
git clone https://github.com/fakerybakery/OpenXet.git ../openxet
```

After cloning, verify the path resolves correctly from this repo:

```bash
ls ../openxet/   # should list the OpenXet workspace root
```

To update before a new analysis round:

```bash
git -C ../openxet pull --ff-only
```

## Directory layout

```
docs/openxet_gap_analysis/
  README.md              # this file — governance and reference-clone policy
  github_tracking.md     # issue-by-issue status tracker for the workstream
  round1/                # OpenXet module maps (populated by Round 1 issues)
  round2/                # xet-backend vs OpenXet comparisons (populated by Round 2 issues)
  synthesis/             # cross-module architecture and gap/dependency synthesis
  templates/             # reusable doc templates and naming rules (issue #38)
  decision_matrix.md     # final decision matrix (late-stage artifact)
```

Directories without entries yet are created by the issues that own them. Do not create placeholder files in those directories ahead of the owning issue.

## Workstream status

See `github_tracking.md` for the current issue-by-issue status.
