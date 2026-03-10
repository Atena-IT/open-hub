# /// script
# requires-python = ">=3.8"
# dependencies = [
#   "huggingface_hub",
#   "requests"
# ]
# ///
import os
os.environ["HF_ENDPOINT"] = os.environ.get("HF_ENDPOINT", "http://localhost:8080")
os.environ["CURL_CA_BUNDLE"] = ""

import io
import tempfile
import requests
from huggingface_hub import HfApi

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
    
    repo_id = "test-user/test-upload-file-streams"
    print(f"Creating repo {repo_id}...")
    try:
        api.create_repo(repo_id, exist_ok=True, repo_type="model")
    except Exception as e:
        pass

    # 1. Upload file from disk
    with tempfile.NamedTemporaryFile(mode="w", delete=False) as f:
        f.write("Hello from disk!")
        tmp_path = f.name
    
    print("Uploading file from disk...")
    api.upload_file(
        path_or_fileobj=tmp_path,
        path_in_repo="file_from_disk.txt",
        repo_id=repo_id,
        commit_message="Upload from disk"
    )
    os.remove(tmp_path)

    # 2. Upload file from stream/bytes
    print("Uploading file from bytes/stream...")
    stream = io.BytesIO(b"Hello from stream!")
    api.upload_file(
        path_or_fileobj=stream,
        path_in_repo="file_from_stream.txt",
        repo_id=repo_id,
        commit_message="Upload from stream"
    )

    # Verification
    print("Listing files...")
    files_main = api.list_repo_files(repo_id)
    print(f"Found files: {files_main}")
    assert "file_from_disk.txt" in files_main
    assert "file_from_stream.txt" in files_main

    print("Upload file test passed!")

if __name__ == "__main__":
    main()
