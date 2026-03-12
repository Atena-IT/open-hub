from pathlib import Path

import pytest
from huggingface_hub import hf_hub_download, snapshot_download
from huggingface_hub.errors import RevisionNotFoundError



def _create_refs_repo(hf_api, repo_factory, tmp_path):
    repo_id = repo_factory("refs")
    upload_dir = tmp_path / "upload"
    upload_dir.mkdir()
    (upload_dir / "config.json").write_text('{"refs": 1}', encoding="utf-8")
    (upload_dir / "nested.txt").write_text("nested", encoding="utf-8")
    hf_api.upload_folder(
        folder_path=upload_dir,
        repo_id=repo_id,
        repo_type="model",
        commit_message="Upload refs fixture",
    )
    head_sha = hf_api.model_info(repo_id).sha
    return repo_id, head_sha



def test_list_repo_refs_includes_main_branch_and_created_named_refs(hf_api, repo_factory, tmp_path):
    repo_id, head_sha = _create_refs_repo(hf_api, repo_factory, tmp_path)

    hf_api.create_branch(repo_id, branch="release")
    hf_api.create_tag(repo_id, tag="v1.0", revision=head_sha)

    refs = hf_api.list_repo_refs(repo_id)
    branches = {ref.name: ref for ref in refs.branches}
    tags = {ref.name: ref for ref in refs.tags}

    assert branches["main"].ref == "refs/heads/main"
    assert branches["main"].target_commit == head_sha
    assert branches["release"].ref == "refs/heads/release"
    assert branches["release"].target_commit == head_sha
    assert tags["v1.0"].ref == "refs/tags/v1.0"
    assert tags["v1.0"].target_commit == head_sha



def test_revision_exists_model_info_download_and_snapshot_accept_named_refs(
    hf_api, hf_session, repo_factory, tmp_path
):
    repo_id, head_sha = _create_refs_repo(hf_api, repo_factory, tmp_path)

    hf_api.create_branch(repo_id, branch="release")
    hf_api.create_tag(repo_id, tag="v1.0", revision=head_sha)

    info = hf_api.model_info(repo_id, revision="release")
    downloaded_path = Path(
        hf_hub_download(
            repo_id=repo_id,
            filename="config.json",
            repo_type="model",
            revision="v1.0",
            endpoint=hf_session["endpoint"],
            token=hf_session["token"],
            cache_dir=tmp_path / "cache-download",
        )
    )
    snapshot_path = Path(
        snapshot_download(
            repo_id=repo_id,
            repo_type="model",
            revision="release",
            endpoint=hf_session["endpoint"],
            token=hf_session["token"],
            cache_dir=tmp_path / "cache-snapshot",
        )
    )

    assert hf_api.revision_exists(repo_id, "release")
    assert hf_api.revision_exists(repo_id, "v1.0")
    assert info.sha == head_sha
    assert downloaded_path.read_text(encoding="utf-8") == '{"refs": 1}'
    assert (snapshot_path / "config.json").read_text(encoding="utf-8") == '{"refs": 1}'
    assert (snapshot_path / "nested.txt").read_text(encoding="utf-8") == "nested"



def test_delete_branch_and_tag_remove_named_refs(hf_api, repo_factory, tmp_path):
    repo_id, head_sha = _create_refs_repo(hf_api, repo_factory, tmp_path)

    hf_api.create_branch(repo_id, branch="release")
    hf_api.create_tag(repo_id, tag="v1.0", revision=head_sha)

    hf_api.delete_branch(repo_id, branch="release")
    hf_api.delete_tag(repo_id, tag="v1.0")

    refs = hf_api.list_repo_refs(repo_id)

    assert [ref.name for ref in refs.branches] == ["main"]
    assert refs.tags == []
    assert not hf_api.revision_exists(repo_id, "release")
    assert not hf_api.revision_exists(repo_id, "v1.0")



def test_create_branch_and_tag_reject_missing_revision(hf_api, repo_factory, tmp_path):
    repo_id, _ = _create_refs_repo(hf_api, repo_factory, tmp_path)

    with pytest.raises(RevisionNotFoundError):
        hf_api.create_branch(repo_id, branch="release", revision="does-not-exist")

    with pytest.raises(RevisionNotFoundError):
        hf_api.create_tag(repo_id, tag="v1.0", revision="does-not-exist")
