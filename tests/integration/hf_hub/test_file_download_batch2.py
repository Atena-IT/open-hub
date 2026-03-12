from pathlib import Path

import pytest
from huggingface_hub import hf_hub_download
from huggingface_hub.errors import LocalEntryNotFoundError
from huggingface_hub.file_download import try_to_load_from_cache



def _create_download_repo(hf_api, repo_factory):
    repo_id = repo_factory("file-download-batch2")
    content = b'{"model_type": "gpt2", "batch": 2}'
    hf_api.upload_file(
        path_or_fileobj=content,
        path_in_repo="config.json",
        repo_id=repo_id,
        repo_type="model",
        commit_message="Upload config.json",
    )
    return repo_id, content



def test_hf_hub_download_to_local_dir_after_cache_seed(hf_api, hf_session, repo_factory, tmp_path):
    repo_id, expected_content = _create_download_repo(hf_api, repo_factory)
    cache_dir = tmp_path / "cache"
    local_dir = tmp_path / "local"

    cached_path = Path(
        hf_hub_download(
            repo_id=repo_id,
            filename="config.json",
            repo_type="model",
            endpoint=hf_session["endpoint"],
            token=hf_session["token"],
            cache_dir=cache_dir,
        )
    )
    local_path = Path(
        hf_hub_download(
            repo_id=repo_id,
            filename="config.json",
            repo_type="model",
            endpoint=hf_session["endpoint"],
            token=hf_session["token"],
            cache_dir=cache_dir,
            local_dir=local_dir,
        )
    )

    assert cached_path.read_bytes() == expected_content
    assert local_path == local_dir / "config.json"
    assert local_path.read_bytes() == expected_content
    assert try_to_load_from_cache(repo_id, filename="config.json", cache_dir=cache_dir) == str(cached_path)



def test_hf_hub_download_local_files_only_returns_cached_path(hf_api, hf_session, repo_factory, tmp_path):
    repo_id, expected_content = _create_download_repo(hf_api, repo_factory)
    cache_dir = tmp_path / "cache"

    cached_path = hf_hub_download(
        repo_id=repo_id,
        filename="config.json",
        repo_type="model",
        endpoint=hf_session["endpoint"],
        token=hf_session["token"],
        cache_dir=cache_dir,
    )
    local_only_path = hf_hub_download(
        repo_id=repo_id,
        filename="config.json",
        repo_type="model",
        endpoint=hf_session["endpoint"],
        token=hf_session["token"],
        cache_dir=cache_dir,
        local_files_only=True,
    )

    assert local_only_path == cached_path
    assert Path(local_only_path).read_bytes() == expected_content



def test_hf_hub_download_local_files_only_missing_cache_raises(hf_session, repo_factory, tmp_path):
    repo_id = repo_factory("file-download-local-only-miss")

    with pytest.raises(LocalEntryNotFoundError):
        hf_hub_download(
            repo_id=repo_id,
            filename="config.json",
            repo_type="model",
            endpoint=hf_session["endpoint"],
            token=hf_session["token"],
            cache_dir=tmp_path / "cache",
            local_files_only=True,
        )



def test_hf_hub_download_refreshes_cached_main_after_new_commit(hf_api, hf_session, repo_factory, tmp_path):
    repo_id, first_content = _create_download_repo(hf_api, repo_factory)
    cache_dir = tmp_path / "cache"

    first_cached_path = Path(
        hf_hub_download(
            repo_id=repo_id,
            filename="config.json",
            repo_type="model",
            endpoint=hf_session["endpoint"],
            token=hf_session["token"],
            cache_dir=cache_dir,
        )
    )

    updated_content = b'{"model_type": "gpt2", "batch": 2, "version": 2}'
    hf_api.upload_file(
        path_or_fileobj=updated_content,
        path_in_repo="config.json",
        repo_id=repo_id,
        repo_type="model",
        commit_message="Update config.json",
    )

    refreshed_cached_path = Path(
        hf_hub_download(
            repo_id=repo_id,
            filename="config.json",
            repo_type="model",
            endpoint=hf_session["endpoint"],
            token=hf_session["token"],
            cache_dir=cache_dir,
        )
    )

    assert first_cached_path.read_bytes() == first_content
    assert refreshed_cached_path.read_bytes() == updated_content
    assert refreshed_cached_path != first_cached_path
    assert try_to_load_from_cache(repo_id, filename="config.json", cache_dir=cache_dir) == str(
        refreshed_cached_path
    )



def test_hf_hub_download_overwrites_stale_local_dir_file(hf_api, hf_session, repo_factory, tmp_path):
    repo_id, expected_content = _create_download_repo(hf_api, repo_factory)
    cache_dir = tmp_path / "cache"
    local_dir = tmp_path / "local"
    local_dir.mkdir(parents=True, exist_ok=True)
    stale_path = local_dir / "config.json"
    stale_path.write_text("stale", encoding="utf-8")

    local_path = Path(
        hf_hub_download(
            repo_id=repo_id,
            filename="config.json",
            repo_type="model",
            endpoint=hf_session["endpoint"],
            token=hf_session["token"],
            cache_dir=cache_dir,
            local_dir=local_dir,
        )
    )

    assert local_path == stale_path
    assert local_path.read_bytes() == expected_content
