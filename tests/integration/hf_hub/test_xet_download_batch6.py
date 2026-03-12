import hashlib
import json

import requests



def _auth_headers(token: str) -> dict[str, str]:
    return {"Authorization": f"Bearer {token}"}



def _seed_repo_head(hf_api, repo_id: str, tmp_path, content: str = "seed") -> str:
    seed_path = tmp_path / f"{repo_id.rsplit('/', 1)[-1]}-seed.txt"
    seed_path.write_text(content, encoding="utf-8")
    hf_api.upload_file(
        path_or_fileobj=str(seed_path),
        path_in_repo="seed.txt",
        repo_id=repo_id,
        repo_type="model",
        commit_message="Seed repo head",
    )
    return hf_api.model_info(repo_id).sha



def _upload_lfs_object(endpoint: str, token: str, repo_id: str, payload: bytes) -> tuple[str, int]:
    oid = hashlib.sha256(payload).hexdigest()
    upload_response = requests.put(
        f"{endpoint}/{repo_id}.git/info/lfs/objects/{oid}",
        headers=_auth_headers(token),
        data=payload,
        timeout=10,
    )
    upload_response.raise_for_status()

    verify_response = requests.post(
        f"{endpoint}/{repo_id}.git/info/lfs/verify",
        headers=_auth_headers(token),
        json={"oid": oid, "size": len(payload)},
        timeout=10,
    )
    verify_response.raise_for_status()
    return oid, len(payload)



def _commit_lfs_pointer(endpoint: str, token: str, repo_id: str, path_in_repo: str, oid: str, size: int) -> None:
    payload = "\n".join(
        [
            json.dumps({"key": "header", "value": {"summary": "Add batch 6 LFS fixture"}}),
            json.dumps(
                {
                    "key": "lfsFile",
                    "value": {"path": path_in_repo, "oid": oid, "size": size},
                }
            ),
        ]
    )
    response = requests.post(
        f"{endpoint}/api/models/{repo_id}/commit/main",
        headers={**_auth_headers(token), "Content-Type": "application/x-ndjson"},
        data=f"{payload}\n",
        timeout=10,
    )
    response.raise_for_status()



def _create_lfs_backed_repo(hf_api, hf_session, repo_factory, tmp_path, private: bool = False):
    repo_id = repo_factory("xet-download", private=private)
    oid, size = _upload_lfs_object(
        hf_session["endpoint"],
        hf_session["token"],
        repo_id,
        b"batch6-xet-download-payload",
    )
    _commit_lfs_pointer(
        hf_session["endpoint"],
        hf_session["token"],
        repo_id,
        "weights.bin",
        oid,
        size,
    )
    head_sha = hf_api.model_info(repo_id).sha
    hf_api.create_branch(repo_id, branch="release")
    return repo_id, oid, size, head_sha



def test_xet_read_token_allows_main_for_empty_repo(hf_session, repo_factory):
    repo_id = repo_factory("xet-read-token-empty", private=False)

    response = requests.get(
        f"{hf_session['endpoint']}/api/models/{repo_id}/xet-read-token/main",
        headers=_auth_headers(hf_session["token"]),
        timeout=10,
    )

    assert response.status_code == 200
    assert response.json()["accessToken"]
    assert response.json()["exp"] > 0
    assert response.json()["casUrl"]



def test_xet_read_token_returns_headers_and_enforces_access_and_revision(
    hf_api, hf_session, repo_factory, second_user, tmp_path
):
    repo_id = repo_factory("xet-read-token", private=True)
    _seed_repo_head(hf_api, repo_id, tmp_path)
    hf_api.create_branch(repo_id, branch="release")

    response = requests.get(
        f"{hf_session['endpoint']}/api/models/{repo_id}/xet-read-token/release",
        headers=_auth_headers(hf_session["token"]),
        timeout=10,
    )

    assert response.status_code == 200
    assert response.json()["accessToken"]
    assert response.json()["exp"] > 0
    assert response.json()["casUrl"]
    assert response.headers["X-Xet-Cas-Url"] == response.json()["casUrl"]
    assert response.headers["X-Xet-Access-Token"] == response.json()["accessToken"]
    assert response.headers["X-Xet-Token-Expiration"] == str(response.json()["exp"])

    forbidden = requests.get(
        f"{hf_session['endpoint']}/api/models/{repo_id}/xet-read-token/release",
        headers=_auth_headers(second_user["token"]),
        timeout=10,
    )
    assert forbidden.status_code == 403

    missing_revision = requests.get(
        f"{hf_session['endpoint']}/api/models/{repo_id}/xet-read-token/does-not-exist",
        headers=_auth_headers(hf_session["token"]),
        timeout=10,
    )
    assert missing_revision.status_code == 404



def test_head_on_lfs_resolve_exposes_revision_aware_xet_headers(
    hf_api, hf_session, repo_factory, tmp_path
):
    repo_id, oid, size, head_sha = _create_lfs_backed_repo(
        hf_api, hf_session, repo_factory, tmp_path
    )

    response = requests.head(
        f"{hf_session['endpoint']}/{repo_id}/resolve/release/weights.bin",
        headers=_auth_headers(hf_session["token"]),
        timeout=10,
    )

    assert response.status_code == 200
    assert response.headers["X-Xet-Hash"] == oid
    assert response.headers["X-Xet-Refresh-Route"] == (
        f"{hf_session['endpoint']}/api/models/{repo_id}/xet-read-token/release"
    )
    assert response.headers["X-Linked-Size"] == str(size)
    assert response.headers["X-Repo-Commit"] == head_sha



def test_lfs_batch_download_prefers_xet_and_checks_private_repo_access(
    hf_api, hf_session, repo_factory, second_user, tmp_path
):
    repo_id, oid, size, _ = _create_lfs_backed_repo(
        hf_api, hf_session, repo_factory, tmp_path, private=True
    )
    batch_url = f"{hf_session['endpoint']}/{repo_id}.git/info/lfs/objects/batch"
    payload = {
        "operation": "download",
        "transfers": ["basic", "xet"],
        "objects": [{"oid": oid, "size": size}],
    }

    response = requests.post(
        batch_url,
        headers=_auth_headers(hf_session["token"]),
        json=payload,
        timeout=10,
    )
    response.raise_for_status()
    data = response.json()

    assert data["transfer"] == "xet"
    assert data["objects"][0]["authenticated"] is True
    assert data["objects"][0]["actions"]["download"]["href"]

    forbidden = requests.post(
        batch_url,
        headers=_auth_headers(second_user["token"]),
        json=payload,
        timeout=10,
    )
    assert forbidden.status_code == 403
