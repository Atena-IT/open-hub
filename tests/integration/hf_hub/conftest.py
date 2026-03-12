import os
import uuid

from helpers import (
    DEFAULT_ENDPOINT,
    DEFAULT_PASSWORD,
    configure_environment,
    delete_repo_direct,
    make_repo_id,
    register_or_login,
    wait_for_server,
)

os.environ.setdefault("HF_ENDPOINT", DEFAULT_ENDPOINT)
os.environ.setdefault("HF_HUB_DISABLE_SYMLINKS_WARNING", "1")

import pytest
import huggingface_hub.file_download as hf_file_download
from huggingface_hub import HfApi

if os.name == "nt":
    hf_file_download.are_symlinks_supported = lambda cache_dir=None: False
    hf_file_download._are_symlinks_supported_in_dir.clear()


@pytest.fixture(scope="session")
def hf_endpoint() -> str:
    return os.environ.get("HF_ENDPOINT", DEFAULT_ENDPOINT)


@pytest.fixture(scope="session")
def hf_session(hf_endpoint: str) -> dict[str, str]:
    wait_for_server(hf_endpoint)
    username = os.environ.get("HF_TEST_USERNAME", "pytest-hf-hub")
    password = os.environ.get("HF_TEST_PASSWORD", DEFAULT_PASSWORD)
    token = register_or_login(hf_endpoint, username, password)
    configure_environment(hf_endpoint, token)
    return {
        "endpoint": hf_endpoint,
        "username": username,
        "password": password,
        "token": token,
    }


@pytest.fixture
def hf_api(hf_session: dict[str, str]) -> HfApi:
    configure_environment(hf_session["endpoint"], hf_session["token"])
    return HfApi(endpoint=hf_session["endpoint"], token=hf_session["token"])


@pytest.fixture
def second_user(hf_endpoint: str) -> dict[str, str]:
    username = os.environ.get("HF_TEST_SECOND_USERNAME") or f"pytest-hf-hub-alt-{uuid.uuid4().hex[:8]}"
    password = os.environ.get("HF_TEST_SECOND_PASSWORD", DEFAULT_PASSWORD)
    token = register_or_login(hf_endpoint, username, password)
    return {
        "endpoint": hf_endpoint,
        "username": username,
        "password": password,
        "token": token,
    }


@pytest.fixture
def repo_factory(hf_api: HfApi, hf_session: dict[str, str]):
    created_repos: list[tuple[str, str]] = []

    def create_repo(prefix: str, repo_type: str = "model", private: bool = False) -> str:
        repo_id = make_repo_id(hf_session["username"], prefix)
        hf_api.create_repo(repo_id=repo_id, repo_type=repo_type, private=private)
        created_repos.append((repo_id, repo_type))
        return repo_id

    yield create_repo

    for repo_id, repo_type in reversed(created_repos):
        try:
            hf_api.delete_repo(repo_id=repo_id, repo_type=repo_type, missing_ok=True)
        except Exception:
            delete_repo_direct(
                endpoint=hf_session["endpoint"],
                token=hf_session["token"],
                repo_id=repo_id,
                repo_type=repo_type,
            )
