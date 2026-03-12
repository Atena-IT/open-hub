from pathlib import Path

from huggingface_hub import hf_hub_download, hf_hub_url
from huggingface_hub.file_download import get_hf_file_metadata, try_to_load_from_cache


def _create_download_repo(hf_api, repo_factory):
    repo_id = repo_factory("file-download")
    content = b'{"model_type": "gpt2", "batch": 1}'
    hf_api.upload_file(
        path_or_fileobj=content,
        path_in_repo="config.json",
        repo_id=repo_id,
        repo_type="model",
        commit_message="Upload config.json",
    )
    return repo_id, content


def test_hf_hub_download_current_head(hf_api, hf_session, repo_factory, tmp_path):
    repo_id, expected_content = _create_download_repo(hf_api, repo_factory)

    downloaded_path = hf_hub_download(
        repo_id=repo_id,
        filename="config.json",
        repo_type="model",
        endpoint=hf_session["endpoint"],
        token=hf_session["token"],
        cache_dir=tmp_path / "cache",
    )

    assert Path(downloaded_path).read_bytes() == expected_content


def test_hf_hub_download_revision_main(hf_api, hf_session, repo_factory, tmp_path):
    repo_id, expected_content = _create_download_repo(hf_api, repo_factory)

    downloaded_path = hf_hub_download(
        repo_id=repo_id,
        filename="config.json",
        repo_type="model",
        revision="main",
        endpoint=hf_session["endpoint"],
        token=hf_session["token"],
        cache_dir=tmp_path / "cache",
    )

    assert Path(downloaded_path).read_bytes() == expected_content


def test_get_hf_file_metadata_basic(hf_api, hf_session, repo_factory):
    repo_id, expected_content = _create_download_repo(hf_api, repo_factory)
    repo_info = hf_api.model_info(repo_id)
    file_url = hf_hub_url(
        repo_id=repo_id,
        filename="config.json",
        repo_type="model",
        endpoint=hf_session["endpoint"],
    )

    metadata = get_hf_file_metadata(file_url, token=hf_session["token"])

    assert metadata.commit_hash == repo_info.sha
    assert metadata.etag is not None
    assert metadata.size == len(expected_content)


def test_try_to_load_from_cache_after_download(hf_api, hf_session, repo_factory, tmp_path):
    repo_id, _ = _create_download_repo(hf_api, repo_factory)
    cache_dir = tmp_path / "cache"

    downloaded_path = hf_hub_download(
        repo_id=repo_id,
        filename="config.json",
        repo_type="model",
        endpoint=hf_session["endpoint"],
        token=hf_session["token"],
        cache_dir=cache_dir,
    )

    assert try_to_load_from_cache(repo_id, filename="config.json", cache_dir=cache_dir) == downloaded_path
    assert (
        try_to_load_from_cache(repo_id, filename="config.json", cache_dir=cache_dir, revision="main")
        == downloaded_path
    )
