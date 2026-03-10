# /// script
# requires-python = ">=3.8"
# dependencies = [
#   "huggingface_hub",
#   "requests"
# ]
# ///
import os
import requests
os.environ["HF_ENDPOINT"] = os.environ.get("HF_ENDPOINT", "http://localhost:8080")
os.environ["CURL_CA_BUNDLE"] = ""

from huggingface_hub import HfApi, CommitOperationAdd, CommitOperationDelete

ENDPOINT = os.environ["HF_ENDPOINT"]

def get_token(username="test-user", password="password"):
    resp = requests.post(f"{ENDPOINT}/api/auth/register", json={"username": username, "password": password})
    if resp.status_code == 409:
        resp = requests.post(f"{ENDPOINT}/api/auth/login", json={"username": username, "password": password})
    resp.raise_for_status()
    return resp.json()["token"]

def main():
    token = get_token()
    os.environ["HF_TOKEN"] = token
    api = HfApi(token=token)
    
    repo_id = "test-user/test-create-commit-deletions"
    print(f"Creating repo {repo_id}...")
    try:
        api.create_repo(repo_id, exist_ok=True, repo_type="model")
    except Exception as e:
        pass

    # Phase 1: Add some files
    operations = [
        CommitOperationAdd(path_in_repo="file_to_keep.txt", path_or_fileobj=b"Keep me"),
        CommitOperationAdd(path_in_repo="file_to_delete.txt", path_or_fileobj=b"Delete me later"),
        CommitOperationAdd(path_in_repo="another_file.txt", path_or_fileobj=b"Just another file")
    ]
    
    print("Creating initial commit with files...")
    api.create_commit(
        repo_id=repo_id,
        operations=operations,
        commit_message="Initial files"
    )

    files_v1 = api.list_repo_files(repo_id)
    assert "file_to_delete.txt" in files_v1

    # Phase 2: Add and delete in the same commit
    operations2 = [
        CommitOperationAdd(path_in_repo="new_file.txt", path_or_fileobj=b"I am new"),
        CommitOperationDelete(path_in_repo="file_to_delete.txt")
    ]
    
    print("Creating second commit with additions and deletions...")
    api.create_commit(
        repo_id=repo_id,
        operations=operations2,
        commit_message="Add and delete"
    )

    files_v2 = api.list_repo_files(repo_id)
    print(f"Final files: {files_v2}")
    assert "file_to_delete.txt" not in files_v2
    assert "new_file.txt" in files_v2
    assert "file_to_keep.txt" in files_v2

    print("Create commit deletions test passed!")

if __name__ == "__main__":
    main()
