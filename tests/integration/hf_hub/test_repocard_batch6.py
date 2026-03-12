from huggingface_hub import ModelCard, RepoCard



def _write_readme(path, content: str):
    path.write_text(content, encoding="utf-8")
    return path



def _initial_card_text() -> str:
    return """---
license: mit
tags:
- batch6
- compat
---

Initial batch 6 model card body.
"""



def _updated_card_text() -> str:
    return """---
license: apache-2.0
tags:
- batch6
- updated
---

Updated batch 6 model card body.
"""



def test_repocard_loads_metadata_and_body_and_pushes_update(hf_api, hf_session, repo_factory, tmp_path):
    repo_id = repo_factory("repocard")
    initial_path = _write_readme(tmp_path / "README.initial.md", _initial_card_text())

    hf_api.upload_file(
        path_or_fileobj=str(initial_path),
        path_in_repo="README.md",
        repo_id=repo_id,
        repo_type="model",
        commit_message="Add README fixture",
    )

    repo_card = RepoCard.load(repo_id, repo_type="model", token=hf_session["token"])
    model_card = ModelCard.load(repo_id, token=hf_session["token"])

    assert repo_card.data.license == "mit"
    assert model_card.data.license == "mit"
    assert model_card.data.tags == ["batch6", "compat"]
    assert model_card.text == "Initial batch 6 model card body.\n"

    updated_card = ModelCard(_updated_card_text())
    updated_card.push_to_hub(
        repo_id,
        token=hf_session["token"],
        commit_message="Update README fixture",
    )

    reloaded = ModelCard.load(repo_id, token=hf_session["token"])

    assert reloaded.data.license == "apache-2.0"
    assert reloaded.data.tags == ["batch6", "updated"]
    assert reloaded.text == "Updated batch 6 model card body.\n"
