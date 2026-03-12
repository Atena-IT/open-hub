import hashlib

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



def test_xet_write_token_requires_auth_write_access_and_valid_revision(
    hf_api, hf_session, repo_factory, second_user, tmp_path
):
    repo_id = repo_factory("xet-write-token", private=True)
    _seed_repo_head(hf_api, repo_id, tmp_path)
    hf_api.create_branch(repo_id, branch="release")
    token_url = f"{hf_session['endpoint']}/api/models/{repo_id}/xet-write-token/release"

    unauthenticated = requests.get(token_url, timeout=10)
    assert unauthenticated.status_code == 401

    forbidden = requests.get(
        token_url,
        headers=_auth_headers(second_user["token"]),
        timeout=10,
    )
    assert forbidden.status_code == 403

    missing_revision = requests.get(
        f"{hf_session['endpoint']}/api/models/{repo_id}/xet-write-token/does-not-exist",
        headers=_auth_headers(hf_session["token"]),
        timeout=10,
    )
    assert missing_revision.status_code == 404

    response = requests.get(
        token_url,
        headers=_auth_headers(hf_session["token"]),
        timeout=10,
    )
    assert response.status_code == 200
    assert response.json()["accessToken"]
    assert response.json()["exp"] > 0
    assert response.json()["casUrl"]



def test_lfs_batch_upload_negotiates_xet_or_basic_based_on_requested_transfers(
    hf_session, repo_factory
):
    repo_id = repo_factory("xet-upload-negotiation")
    batch_url = f"{hf_session['endpoint']}/{repo_id}.git/info/lfs/objects/batch"

    xet_response = requests.post(
        batch_url,
        headers=_auth_headers(hf_session["token"]),
        json={
            "operation": "upload",
            "transfers": ["basic", "xet"],
            "objects": [{"oid": "0" * 64, "size": 5}],
        },
        timeout=10,
    )
    xet_response.raise_for_status()
    assert xet_response.json()["transfer"] == "xet"

    basic_response = requests.post(
        batch_url,
        headers=_auth_headers(hf_session["token"]),
        json={
            "operation": "upload",
            "transfers": ["basic"],
            "objects": [{"oid": "1" * 64, "size": 5}],
        },
        timeout=10,
    )
    basic_response.raise_for_status()
    assert basic_response.json()["transfer"] == "basic"



def test_direct_lfs_upload_and_verify_require_write_access(
    hf_session, repo_factory, second_user
):
    repo_id = repo_factory("xet-upload-private", private=True)
    payload = b"batch6-direct-lfs"
    oid = hashlib.sha256(payload).hexdigest()
    upload_url = f"{hf_session['endpoint']}/{repo_id}.git/info/lfs/objects/{oid}"
    verify_url = f"{hf_session['endpoint']}/{repo_id}.git/info/lfs/verify"
    batch_url = f"{hf_session['endpoint']}/{repo_id}.git/info/lfs/objects/batch"

    forbidden_upload = requests.put(
        upload_url,
        headers=_auth_headers(second_user["token"]),
        data=payload,
        timeout=10,
    )
    assert forbidden_upload.status_code == 403

    owner_upload = requests.put(
        upload_url,
        headers=_auth_headers(hf_session["token"]),
        data=payload,
        timeout=10,
    )
    assert owner_upload.status_code == 200

    forbidden_verify = requests.post(
        verify_url,
        headers=_auth_headers(second_user["token"]),
        json={"oid": oid, "size": len(payload)},
        timeout=10,
    )
    assert forbidden_verify.status_code == 403

    owner_verify = requests.post(
        verify_url,
        headers=_auth_headers(hf_session["token"]),
        json={"oid": oid, "size": len(payload)},
        timeout=10,
    )
    assert owner_verify.status_code == 200

    download_batch = requests.post(
        batch_url,
        headers=_auth_headers(hf_session["token"]),
        json={
            "operation": "download",
            "transfers": ["basic"],
            "objects": [{"oid": oid, "size": len(payload)}],
        },
        timeout=10,
    )
    download_batch.raise_for_status()
    body = download_batch.json()
    assert body["transfer"] == "basic"
    assert body["objects"][0]["actions"]["download"]["href"]
