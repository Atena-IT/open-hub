# Xet Backend Repo Contract

## Working defaults
- Keep changes minimal and tightly scoped to the active task or batch.
- Prefer tests-first for `huggingface_hub` compatibility work: add or tighten pytest coverage, then implement only the server behavior required to pass.
- Do not widen scope into larger architectural work unless the roadmap or the user explicitly asks for it.

## Validation defaults
- Python HF Hub compatibility tests: `uv run --with pytest --with huggingface_hub --with requests pytest tests/integration/hf_hub -q`
- Rust validation: `cargo fmt --all` and `cargo test --workspace`
- Reuse the existing deployment stack in `deployment/docker-compose.yml`; avoid spinning a second compose project on the same ports unless that is intentional.

## HF Hub batch workflow
- Follow the batch lifecycle: issue -> issue branch -> draft PR -> targeted tests -> minimal implementation -> full suite -> tracker update.
- Update both trackers when a batch changes state:
  - `docs/hf_hub_testing_roadmap.md`
  - `resources/hf_hub_compat/checklist.json`

## Repository boundaries
- Treat the current Hub layer as compatibility-oriented behavior over the existing storage model, not as a fully featured Git server.
- Do not assume arbitrary git history, full ref semantics, or historical file snapshots unless the roadmap has been explicitly expanded to cover them.
- Keep local tool state out of commits unless explicitly requested (for example `.claude/worktrees/` and ad-hoc scratch files).

## Useful docs
- `docs/hf_hub_testing_roadmap.md` — active compatibility batches and scope
- `resources/hf_hub_compat/checklist.json` — machine-readable tracker
- `docs/testing_guide.md` — local testing and workflow notes
- `docs/feature_support_matrix.md` — current support overview
- `docs/hf_integration_plan.md` — broader HF integration context
