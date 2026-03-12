from pathlib import Path

import pytest
import requests
from huggingface_hub import HfApi, hf_hub_download
from huggingface_hub.errors import HfHubHTTPError, RepositoryNotFoundError



def _create_private_repo(hf_api, repo_factory):
    repo_id = repo_factory("repo-visibility-private", private=True)
    content = b'{"private": true, "batch": 3}'
    hf_api.upload_file(
        path_or_fileobj=content,
        path_in_repo="config.json",
        repo_id=repo_id,
        repo_type="model",
        commit_message="Upload private config",
    )
    return repo_id, content



def _repo_info_url(endpoint: str, repo_id: str) -> str:
    return f"{endpoint}/api/models/{repo_id}"



def _resolve_url(endpoint: str, repo_id: str) -> str:
    return f"{endpoint}/{repo_id}/resolve/main/config.json"



def _auth_headers(token: str | None) -> dict[str, str]:
    return {"Authorization": f"Bearer {token}"} if token else {}



def test_private_repo_owner_can_read_metadata_and_file(hf_api, hf_session, repo_factory, tmp_path):
    repo_id, expected_content = _create_private_repo(hf_api, repo_factory)

    info = hf_api.model_info(repo_id)
    downloaded_path = Path(
        hf_hub_download(
            repo_id=repo_id,
            filename="config.json",
            repo_type="model",
            endpoint=hf_session["endpoint"],
            token=hf_session["token"],
            cache_dir=tmp_path / "cache",
        )
    )

    assert info.private is True
    assert downloaded_path.read_bytes() == expected_content



def test_private_repo_anonymous_looks_missing_to_client_and_http(hf_api, hf_session, repo_factory):
    repo_id, _ = _create_private_repo(hf_api, repo_factory)
    api = HfApi(endpoint=hf_session["endpoint"], token=hf_session["token"])

    assert not api.repo_exists(repo_id, token=False)
    assert not api.file_exists(repo_id, "config.json", token=False)
    with pytest.raises(RepositoryNotFoundError):
        api.model_info(repo_id, token=False)

    repo_response = requests.get(_repo_info_url(hf_session["endpoint"], repo_id), timeout=10)
    file_response = requests.head(_resolve_url(hf_session["endpoint"], repo_id), timeout=10)

    assert repo_response.status_code == 404
    assert file_response.status_code == 404



def test_private_repo_invalid_token_is_unauthorized(hf_api, hf_session, repo_factory):
    repo_id, _ = _create_private_repo(hf_api, repo_factory)
    headers = _auth_headers("ox_invalid")

    repo_response = requests.get(
        _repo_info_url(hf_session["endpoint"], repo_id),
        headers=headers,
        timeout=10,
    )
    file_response = requests.head(
        _resolve_url(hf_session["endpoint"], repo_id),
        headers=headers,
        timeout=10,
    )

    assert repo_response.status_code == 401
    assert repo_response.headers["X-Error-Message"] == "invalid token"
    assert file_response.status_code == 401
    assert file_response.headers["X-Error-Message"] == "invalid token"



def test_private_repo_non_owner_gets_forbidden(hf_api, hf_session, second_user, repo_factory):
    repo_id, _ = _create_private_repo(hf_api, repo_factory)
    other_api = HfApi(endpoint=hf_session["endpoint"], token=second_user["token"])
    headers = _auth_headers(second_user["token"])

    with pytest.raises(HfHubHTTPError, match="403"):
        other_api.model_info(repo_id)
    with pytest.raises(HfHubHTTPError, match="403"):
        other_api.file_exists(repo_id, "config.json")

    repo_response = requests.get(
        _repo_info_url(hf_session["endpoint"], repo_id),
        headers=headers,
        timeout=10,
    )
    file_response = requests.head(
        _resolve_url(hf_session["endpoint"], repo_id),
        headers=headers,
        timeout=10,
    )

    assert repo_response.status_code == 403
    assert file_response.status_code == 403



def test_private_repo_cached_download_can_be_reused_without_token(
    hf_api, hf_session, repo_factory, tmp_path
):
    repo_id, expected_content = _create_private_repo(hf_api, repo_factory)
    cache_dir = tmp_path / "cache"

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
    local_only_path = Path(
        hf_hub_download(
            repo_id=repo_id,
            filename="config.json",
            repo_type="model",
            endpoint=hf_session["endpoint"],
            token=False,
            cache_dir=cache_dir,
            local_files_only=True,
        )
    )

    assert cached_path == local_only_path
    assert local_only_path.read_bytes() == expected_content
