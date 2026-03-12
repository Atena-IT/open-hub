import os
import time
import uuid
from pathlib import Path

import requests

DEFAULT_ENDPOINT = os.environ.get("HF_ENDPOINT", "http://localhost:8080")
DEFAULT_PASSWORD = os.environ.get("HF_TEST_PASSWORD", "pytest-hf-hub-password")


def wait_for_server(endpoint: str, timeout: int = 60) -> None:
    deadline = time.time() + timeout
    while time.time() < deadline:
        try:
            response = requests.get(f"{endpoint}/health", timeout=3)
            response.raise_for_status()
            if response.json().get("status") == "ok":
                return
        except requests.RequestException:
            pass
        time.sleep(1)
    raise RuntimeError(f"Server not ready at {endpoint} after {timeout}s")


def login_user(endpoint: str, username: str, password: str) -> str:
    response = requests.post(
        f"{endpoint}/api/auth/login",
        json={"username": username, "password": password},
        timeout=10,
    )
    response.raise_for_status()
    return response.json()["token"]


def register_or_login(endpoint: str, username: str, password: str) -> str:
    response = requests.post(
        f"{endpoint}/api/auth/register",
        json={"username": username, "password": password},
        timeout=10,
    )
    if response.status_code == 409:
        return login_user(endpoint, username, password)
    response.raise_for_status()
    return response.json()["token"]


def configure_environment(endpoint: str, token: str | None = None) -> None:
    os.environ["HF_ENDPOINT"] = endpoint
    os.environ["CURL_CA_BUNDLE"] = ""
    os.environ["HF_HUB_DISABLE_SYMLINKS_WARNING"] = "1"
    if token is not None:
        os.environ["HF_TOKEN"] = token


def make_repo_id(owner: str, prefix: str) -> str:
    suffix = uuid.uuid4().hex[:8]
    return f"{owner}/{prefix}-{int(time.time())}-{suffix}"


def delete_repo_direct(endpoint: str, token: str, repo_id: str, repo_type: str = "model") -> None:
    response = requests.delete(
        f"{endpoint}/api/repos/delete",
        json={"name": repo_id, "type": repo_type},
        headers={"Authorization": f"Bearer {token}"},
        timeout=10,
    )
    if response.status_code not in {200, 404}:
        response.raise_for_status()


def write_tree(root: Path, files: dict[str, str | bytes]) -> None:
    for relative_path, content in files.items():
        path = root / relative_path
        path.parent.mkdir(parents=True, exist_ok=True)
        if isinstance(content, str):
            path.write_text(content, encoding="utf-8")
        else:
            path.write_bytes(content)
