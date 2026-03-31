# Feature Support Matrix

This matrix details the features of the Hugging Face Hub ecosystem and their current support in our custom Xet Storage Backend. Our goal is to provide a complete drop-in replacement for the core features needed to manage datasets and models.

> This matrix stays intentionally high level. Detailed `huggingface_hub` client compatibility is tracked batch-by-batch in `docs/hf_hub_testing_roadmap.md` and `resources/hf_hub_compat/checklist.json`.
>
> Broader Git/LFS/Xet-native server expansion and OpenXet comparison work are tracked separately in `docs/openxet_gap_analysis/`.

| Feature / Use Case | Description | In Roadmap | Implemented |
| :--- | :--- | :---: | :---: |
| **Download files** | `hf_hub_download`, `snapshot_download`, specific revisions, and allow/ignore patterns. | Yes | ✅ |
| **Upload files** | Single file uploads, streaming bytes, folder uploads, and LFS chunking. | Yes | ✅ |
| **Commits** | Atomic transactions adding, modifying, or deleting multiple files via NDJSON. | Yes | ✅ |
| **Manage Repositories** | Create, delete, and fetch metadata for Models and Datasets. | Yes | ✅ |
| **Search / Listing** | `api.list_models()` and `api.list_datasets()` iteration support. | Yes | ✅ |
| **Model/Dataset Cards** | Auto-parsing YAML frontmatter in `README.md` and pushing cards to the Hub. | Yes | ✅ |
| **HF File System (`hf_fs`)** | Interacting with the Hub via the `fsspec` filesystem abstraction. | Yes | ✅ |
| **CLI tools** | Compatible with `huggingface-cli` login, upload, and download commands. | Yes | ✅ |
| **Branching / PRs** | Creating Git branches, making PRs, and reviewing via the API. | Yes | ❌ |
| **Manage Spaces** | Hosting interactive Gradio / Streamlit applications. | No | ❌ |
| **Buckets** | Native S3-like interface separate from Git repositories. | No | ❌ |
| **Inference API** | Serverless endpoints to query models directly via HTTP. | No | ❌ |
| **Jobs / Compute** | AutoTrain and dedicated compute clusters. | No | ❌ |
| **Community** | Pull requests discussions, comments, and community boards. | No | ❌ |
| **Collections** | Grouping repositories into curated lists. | No | ❌ |
| **Webhooks** | Emitting event hooks when repositories change. | No | ❌ |

### Key
- ✅ **Fully Supported:** Complete drop-in compatibility.
- ⚠️ **Partial Support:** Core functionality works, but some advanced client-side expectations (e.g., metadata parsing) might be stubbed or ignored.
- ❌ **Not Supported:** Either out of scope for the roadmap or pending future development.
