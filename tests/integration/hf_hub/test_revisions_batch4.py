from pathlib import Path

import pytest
from huggingface_hub import hf_hub_download, snapshot_download
from huggingface_hub.errors import RevisionNotFoundError



def _create_revision_repo(hf_api, repo_factory, tmp_path):
    repo_id = repo_factory("revisions")
    upload_dir = tmp_path / "upload"
    upload_dir.mkdir()
    (upload_dir / "config.json").write_text('{"revision": 1}', encoding="utf-8")
    (upload_dir / "nested.txt").write_text("nested", encoding="utf-8")
    hf_api.upload_folder(
        folder_path=upload_dir,
        repo_id=repo_id,
        repo_type="model",
        commit_message="Upload revision fixture",
    )
    head_sha = hf_api.model_info(repo_id).sha
    return repo_id, head_sha



def test_revision_exists_for_main_and_current_head_sha(hf_api, repo_factory, tmp_path):
    repo_id, head_sha = _create_revision_repo(hf_api, repo_factory, tmp_path)

    assert hf_api.revision_exists(repo_id, "main")
    assert hf_api.revision_exists(repo_id, head_sha)



def test_revision_exists_false_for_missing_revision(hf_api, repo_factory, tmp_path):
    repo_id, _ = _create_revision_repo(hf_api, repo_factory, tmp_path)

    assert not hf_api.revision_exists(repo_id, "does-not-exist")



def test_model_info_and_download_accept_current_head_sha(hf_api, hf_session, repo_factory, tmp_path):
    repo_id, head_sha = _create_revision_repo(hf_api, repo_factory, tmp_path)

    info = hf_api.model_info(repo_id, revision=head_sha)
    downloaded_path = Path(
        hf_hub_download(
            repo_id=repo_id,
            filename="config.json",
            repo_type="model",
            revision=head_sha,
            endpoint=hf_session["endpoint"],
            token=hf_session["token"],
            cache_dir=tmp_path / "cache-download",
        )
    )
    snapshot_path = Path(
        snapshot_download(
            repo_id=repo_id,
            repo_type="model",
            revision=head_sha,
            endpoint=hf_session["endpoint"],
            token=hf_session["token"],
            cache_dir=tmp_path / "cache-snapshot",
        )
    )

    assert info.sha == head_sha
    assert downloaded_path.read_text(encoding="utf-8") == '{"revision": 1}'
    assert (snapshot_path / "config.json").read_text(encoding="utf-8") == '{"revision": 1}'
    assert (snapshot_path / "nested.txt").read_text(encoding="utf-8") == "nested"



def test_model_info_download_and_snapshot_reject_missing_revision(
    hf_api, hf_session, repo_factory, tmp_path
):
    repo_id, _ = _create_revision_repo(hf_api, repo_factory, tmp_path)

    with pytest.raises(RevisionNotFoundError):
        hf_api.model_info(repo_id, revision="does-not-exist")

    with pytest.raises(RevisionNotFoundError):
        hf_hub_download(
            repo_id=repo_id,
            filename="config.json",
            repo_type="model",
            revision="does-not-exist",
            endpoint=hf_session["endpoint"],
            token=hf_session["token"],
            cache_dir=tmp_path / "cache-missing-download",
        )

    with pytest.raises(RevisionNotFoundError):
        snapshot_download(
            repo_id=repo_id,
            repo_type="model",
            revision="does-not-exist",
            endpoint=hf_session["endpoint"],
            token=hf_session["token"],
            cache_dir=tmp_path / "cache-missing-snapshot",
        )
