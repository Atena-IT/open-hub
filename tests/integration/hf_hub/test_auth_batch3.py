import pytest
import requests
from huggingface_hub import HfApi
from huggingface_hub.errors import HfHubHTTPError


def test_whoami_token_false_raises_value_error(hf_session):
    api = HfApi(endpoint=hf_session["endpoint"], token=hf_session["token"])

    with pytest.raises(ValueError, match="token=False"):
        api.whoami(token=False)


def test_whoami_without_or_with_invalid_token_returns_401(hf_session):
    endpoint = hf_session["endpoint"]

    no_token_response = requests.get(f"{endpoint}/api/whoami-v2", timeout=10)
    invalid_token_response = requests.get(
        f"{endpoint}/api/whoami-v2",
        headers={"Authorization": "Bearer ox_invalid"},
        timeout=10,
    )

    assert no_token_response.status_code == 401
    assert no_token_response.headers["X-Error-Message"] == "missing bearer token"
    assert invalid_token_response.status_code == 401
    assert invalid_token_response.headers["X-Error-Message"] == "invalid token"


def test_private_repo_non_owner_upload_file_is_forbidden(hf_api, hf_session, second_user, repo_factory):
    repo_id = repo_factory("auth-private-upload", private=True)
    other_api = HfApi(endpoint=hf_session["endpoint"], token=second_user["token"])

    with pytest.raises(HfHubHTTPError, match="403"):
        other_api.upload_file(
            path_or_fileobj=b"blocked",
            path_in_repo="blocked.txt",
            repo_id=repo_id,
            repo_type="model",
            commit_message="Blocked upload",
        )

    assert not hf_api.file_exists(repo_id, "blocked.txt")
