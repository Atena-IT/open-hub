from pathlib import Path

import pytest
from huggingface_hub import snapshot_download
from huggingface_hub.errors import LocalEntryNotFoundError

from helpers import write_tree



def _create_snapshot_repo(hf_api, repo_factory, tmp_path):
    repo_id = repo_factory("snapshot-download-batch2")
    upload_dir = tmp_path / "upload"
    write_tree(
        upload_dir,
        {
            "config.json": '{"model_type": "gpt2", "batch": 2}',
            "vocab.json": '{"a": 1, "b": 2}',
            "data/train.csv": "col1,col2\n1,2\n",
        },
    )
    hf_api.upload_folder(
        folder_path=upload_dir,
        repo_id=repo_id,
        repo_type="model",
        commit_message="Upload snapshot fixture",
    )
    return repo_id



def test_snapshot_download_local_files_only_reuses_existing_local_dir(
    hf_api, hf_session, repo_factory, tmp_path
):
    repo_id = _create_snapshot_repo(hf_api, repo_factory, tmp_path)
    cache_dir = tmp_path / "cache"
    local_dir = tmp_path / "local-snapshot"

    snapshot_path = Path(
        snapshot_download(
            repo_id=repo_id,
            repo_type="model",
            endpoint=hf_session["endpoint"],
            token=hf_session["token"],
            cache_dir=cache_dir,
            local_dir=local_dir,
        )
    )
    local_only_path = Path(
        snapshot_download(
            repo_id=repo_id,
            repo_type="model",
            endpoint=hf_session["endpoint"],
            token=hf_session["token"],
            cache_dir=cache_dir,
            local_dir=local_dir,
            local_files_only=True,
        )
    )

    assert snapshot_path == local_dir
    assert local_only_path == local_dir
    assert (local_only_path / "config.json").read_text(encoding="utf-8") == '{"model_type": "gpt2", "batch": 2}'
    assert (local_only_path / "data" / "train.csv").read_text(encoding="utf-8") == "col1,col2\n1,2\n"



def test_snapshot_download_local_files_only_missing_cache_raises(hf_session, repo_factory, tmp_path):
    repo_id = repo_factory("snapshot-local-only-miss")

    with pytest.raises(LocalEntryNotFoundError):
        snapshot_download(
            repo_id=repo_id,
            repo_type="model",
            endpoint=hf_session["endpoint"],
            token=hf_session["token"],
            cache_dir=tmp_path / "cache",
            local_files_only=True,
        )



def test_snapshot_download_refreshes_existing_local_dir_after_new_commit(
    hf_api, hf_session, repo_factory, tmp_path
):
    repo_id = _create_snapshot_repo(hf_api, repo_factory, tmp_path)
    cache_dir = tmp_path / "cache"
    local_dir = tmp_path / "local-snapshot"

    snapshot_download(
        repo_id=repo_id,
        repo_type="model",
        endpoint=hf_session["endpoint"],
        token=hf_session["token"],
        cache_dir=cache_dir,
        local_dir=local_dir,
    )

    hf_api.upload_file(
        path_or_fileobj=b'{"model_type": "gpt2", "batch": 2, "version": 2}',
        path_in_repo="config.json",
        repo_id=repo_id,
        repo_type="model",
        commit_message="Update config.json",
    )
    hf_api.upload_file(
        path_or_fileobj=b"new content\n",
        path_in_repo="notes/new.txt",
        repo_id=repo_id,
        repo_type="model",
        commit_message="Add notes/new.txt",
    )

    refreshed_path = Path(
        snapshot_download(
            repo_id=repo_id,
            repo_type="model",
            endpoint=hf_session["endpoint"],
            token=hf_session["token"],
            cache_dir=cache_dir,
            local_dir=local_dir,
        )
    )

    assert refreshed_path == local_dir
    assert (refreshed_path / "config.json").read_text(encoding="utf-8") == '{"model_type": "gpt2", "batch": 2, "version": 2}'
    assert (refreshed_path / "notes" / "new.txt").read_text(encoding="utf-8") == "new content\n"
