# Epic 1 Showcase PRD — Replacement Example for the End-of-Epic Demo

## Summary

Replace the current `dlt` showcase example with a more immediately recognizable example for the Epic 1 demo. The new example should help internal engineers understand the value of Open Hub quickly, while staying inside workflows the product already supports today.

## Problem statement

The current `dlt` example is too domain-specific to carry the showcase narrative on its own. For the Epic 1 demo, the audience should not need prior context to understand what the repository contains, why it matters, or what Open Hub is enabling.

## Audience

**Primary audience:** internal engineers

This audience assumption was explicitly chosen during issue clarification for `#71`. If the expected showcase audience broadens beyond internal engineers, the candidate scores should be re-run before locking the example.

This audience is technical, but the example still needs to be legible at a glance. The strongest choice is one that is:
- recognizable in under a minute
- clearly aligned with the current Open Hub product surface
- realistic enough to feel like a true repository, not a toy file drop

## Goal

Select one primary showcase example and one fallback that:
1. are easier to recognize than `dlt`
2. fit the current Open Hub product capabilities
3. can support a short end-of-epic demo without requiring unsupported platform features

## Non-goals

- building a full benchmark suite of future showcase examples
- expanding product scope to support a specific example
- depending on inference, Spaces, PR workflows, or other out-of-scope Hub features
- deciding the long-term examples strategy for all future demos

## Current product constraints

The example should only rely on capabilities that are already implemented or explicitly validated in current project docs:

- repository creation and metadata fetch
- single-file and folder upload flows
- atomic commit flow for file additions/modifications
- file download and snapshot-style retrieval
- model/dataset card persistence via `README.md`
- Hugging Face-style filesystem access (`hf_fs`)
- private-Hub style repository browsing through the existing Open Hub surface

The example should **not** depend on:
- hosted inference
- Spaces or app hosting
- branch/PR workflows
- community or review features
- advanced historical or collaborative Git semantics beyond the current roadmap

## Decision criteria

Weighted for the approved audience of internal engineers:

| Criterion | Weight | What it means |
| --- | ---: | --- |
| Audience familiarity | 30% | How quickly an internal engineer will recognize the domain |
| Narrative clarity | 25% | How easily the demo story explains why the repo exists |
| Fit with current product capabilities | 25% | How well the example maps to supported Open Hub flows today |
| Demo/setup effort | 10% | How much prep work is needed to make the demo reliable |
| Reuse after the showcase | 10% | How useful the example remains for future smoke tests, docs, or sales engineering |

Scoring uses a 1–5 scale, where 5 is strongest.

## Candidate shortlist

### Candidate A — Tiny sentiment-analysis model repo

**What it is:** a small, clearly labeled text-classification model repository with a model card, tokenizer/config files, example inputs, and a compact weight artifact.

**Why it is attractive:**
- instantly understandable use case: classify text as positive or negative
- strongly aligned with Hugging Face-style model repositories
- lets the demo show both metadata (`README.md`) and real repository assets
- works well with upload, browse, download, and `hf_hub_download` / `hf_fs` workflows

**Trade-offs:**
- best version benefits from a realistic weight artifact and polished README/model card
- if live inference is expected, the audience could over-assume product scope, so the script must keep the focus on repository management rather than hosted execution

### Candidate B — MNIST digit-classification model repo

**What it is:** a model repository for handwritten-digit classification with sample images, model files, and a short card explaining the task.

**Why it is attractive:**
- widely known ML example among engineers
- visually intuitive assets make the repository easy to browse in a demo
- still fits a standard model-repository shape

**Trade-offs:**
- slightly weaker narrative outside ML-familiar audiences than sentiment analysis
- assets and story are more “classic ML demo” than “practical repository someone would really manage day to day”

### Candidate C — Titanic dataset repo

**What it is:** a dataset repository containing a familiar tabular dataset, a dataset card, and a few simple usage examples.

**Why it is attractive:**
- broadly recognizable even beyond ML specialists
- lowest setup risk of the shortlist
- easy to demo upload, list, download, and file-structure browsing

**Trade-offs:**
- demonstrates Open Hub more as managed dataset storage than as a strong private-Hub alternative for model teams
- weaker fit if the goal is to spotlight model-style repository flows and richer artifact packaging

### Candidate D — LoRA adapter repo

**What it is:** a repository for a lightweight fine-tuning adapter plus README, metadata, and usage notes.

**Why it is attractive:**
- modern and relevant to current AI engineering workflows
- strong reuse potential for future examples that target ML practitioners

**Trade-offs:**
- less immediately legible than sentiment analysis or Titanic
- requires more context to explain what the asset is and why the audience should care
- more likely to turn into a domain explanation instead of a product showcase

## Scoring matrix

| Candidate | Familiarity (30%) | Narrative (25%) | Product fit (25%) | Setup (10%) | Reuse (10%) | Weighted score |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| A. Tiny sentiment-analysis model repo | 5 | 5 | 5 | 4 | 4 | **4.8 / 5** |
| B. MNIST digit-classification model repo | 4 | 4 | 4 | 3 | 4 | **3.9 / 5** |
| C. Titanic dataset repo | 4 | 4 | 4 | 5 | 3 | **4.0 / 5** |
| D. LoRA adapter repo | 3 | 3 | 3 | 3 | 5 | **3.2 / 5** |

## Recommendation

### Primary recommendation

Use **Candidate A — Tiny sentiment-analysis model repo** as the Epic 1 showcase example.

**Why this is the best fit:**
- it is the fastest for the approved audience to understand
- it looks and feels like a real Hugging Face-style repository
- it exercises the product surfaces we already support without expanding scope
- it gives the demo a clean story: create repo, upload meaningful assets, inspect metadata, and retrieve them through standard Hub-style workflows

**Why it is better than `dlt`:**
The use case is self-explanatory. The audience does not need to know a specific framework or company context to understand the repository contents.

### Fallback recommendation

Use **Candidate C — Titanic dataset repo** if the team wants the lowest-risk path with the smallest asset-prep burden.

**Why this is the fallback instead of the primary:**
- it is easier to prepare and demo reliably
- it is still broadly understandable
- but it undersells Open Hub’s value as a private Hub for richer model-style repositories

## Proposed demo storyline

Target a **5–7 minute** showcase segment.

### Demo arc

1. **Open with the problem in one sentence**
   - “We want Open Hub to feel immediately familiar for teams managing model and dataset repositories internally.”

2. **Show the chosen repository at a glance**
   - repository name, short description, and model card title
   - make the task obvious from the first screen: sentiment analysis

3. **Show the repository contents**
   - `README.md` / model card
   - config/tokenizer files
   - one compact weight artifact
   - one small examples file with sample text inputs

4. **Show a standard workflow, not a custom one**
   - create the repo
   - upload the folder through a supported Open Hub path
   - browse the resulting files

5. **Show retrieval through familiar client behavior**
   - run a prepared `hf_hub_download(repo_id="<demo-owner>/epic1-sentiment-demo", filename="examples.json")` call as the retrieval proof point
   - keep this step fixed in the dry run so the presenter is not choosing between multiple client paths live

6. **Close on product value**
   - Open Hub already supports the repository lifecycle engineers expect for a private Hub-style environment
   - the showcase is about recognizable repository management, not about hosted inference or unrelated platform features

## Required assets and dependencies

### Required assets

For the primary recommendation:
- repository name and short description
- polished `README.md` / model card
- sample `config.json`
- sample tokenizer/config companion files as needed
- one compact weight artifact suitable for demo upload/download
- one `examples.json` or `samples.md` file showing a few example inputs and expected labels
- one screenshot or backup terminal transcript for recovery if the live path fails

For the fallback dataset option:
- dataset card in `README.md`
- one canonical CSV or parquet file
- one tiny examples/preview file
- one backup screenshot or transcript

### Runtime / tooling dependencies

- one prepared local asset directory checked before the demo
- one repeatable upload path using already supported client/server workflows
- one repeatable retrieval path (`hf_hub_download`, snapshot-style download, or `hf_fs`)
- valid credentials for the demo environment
- a stable demo repository name created ahead of time or scripted deterministically

### Assumptions

- the demo should stay inside currently supported repository-management flows
- no live inference is required
- no unsupported Hub features need to be simulated
- the example assets can be small enough to keep the demo reliable while still looking realistic

## Risks and mitigations

| Risk | Why it matters | Mitigation |
| --- | --- | --- |
| Example looks too toy-like | Weakens stakeholder confidence | Use a polished card, realistic file names, and a believable artifact set |
| Demo drifts into unsupported features | Creates confusion about current scope | Keep the script centered on repo creation, upload, browse, and download |
| Asset prep takes too long | Threatens showcase readiness | Keep the fallback dataset repo ready in parallel |
| Live environment is flaky | Can derail a short showcase slot | Prepare a backup transcript or screenshots from the same workflow |

## Success criteria

The showcase example is ready when:
- an internal engineer can understand the repository purpose in under one minute
- the demo uses only currently supported Open Hub capabilities
- the repository contents look realistic enough to feel production-adjacent
- the team has both a primary and fallback example ready before the showcase
- the presenter can complete the scripted flow without improvising around missing features

## Follow-up implementation shape

Recommend **splitting the remaining work into subtasks**, not keeping it as one undifferentiated task.

### Suggested follow-up split

1. **Prepare showcase assets**
   - assemble the primary sentiment-analysis repo contents
   - prepare the fallback Titanic dataset repo contents

2. **Script and validate the repository flow**
   - create repo
   - upload contents
   - verify browse/download behavior through supported paths

3. **Create presenter-ready materials**
   - final demo script
   - backup screenshots / transcript
   - short “what this proves” closing statement

4. **Run one dry run in the target environment**
   - verify credentials, asset paths, repo naming, and retrieval commands

This split reduces last-minute risk and keeps the issue actionable for internal engineers without inflating scope.

## Final decision

- **Primary example:** Tiny sentiment-analysis model repo
- **Fallback example:** Titanic dataset repo
- **Recommended execution shape:** split follow-up implementation into subtasks

This gives the Epic 1 showcase a recognizable story, keeps the demo aligned with current Open Hub capabilities, and avoids expanding scope to fit the example.