# /// script
# requires-python = ">=3.8"
# dependencies = [
#   "huggingface_hub",
#   "requests",
# ]
# ///

import os
import sys
import json
import time
import tempfile
import urllib.request

SERVER = os.environ.get("HF_ENDPOINT", "http://localhost:8080")
os.environ["HF_ENDPOINT"] = SERVER

import os
os.environ['HF_ENDPOINT'] = os.environ.get('HF_ENDPOINT', 'http://localhost:8080')
from huggingface_hub import HfApi, hf_hub_download, snapshot_download

def wait_for_server(url, timeout=60):
    deadline = time.time() + timeout
    while time.time() < deadline:
        try:
            resp = urllib.request.urlopen(f"{url}/health", timeout=3)
            data = json.loads(resp.read())
            if data.get("status") == "ok":
                return
        except Exception:
            pass
        time.sleep(1)
    print(f"[FAIL] Server not ready at {url} after {timeout}s")
    sys.exit(1)

def register_user(base_url, username, password):
    data = json.dumps({"username": username, "password": password}).encode()
    req = urllib.request.Request(
        f"{base_url}/api/auth/register",
        data=data,
        headers={"Content-Type": "application/json"},
    )
    try:
        resp = urllib.request.urlopen(req)
        result = json.loads(resp.read())
        return result["token"]
    except urllib.error.HTTPError as e:
        if e.code == 409:
            return login_user(base_url, username, password)
        print(f"[FAIL] Register failed: {e.code} {e.read().decode()}")
        sys.exit(1)

def login_user(base_url, username, password):
    data = json.dumps({"username": username, "password": password}).encode()
    req = urllib.request.Request(
        f"{base_url}/api/auth/login",
        data=data,
        headers={"Content-Type": "application/json"},
    )
    resp = urllib.request.urlopen(req)
    result = json.loads(resp.read())
    return result["token"]

def main():
    print(f"=== Testing Download Scenarios ===")
    print(f"Server: {SERVER}")

    wait_for_server(SERVER)
    token = register_user(SERVER, "testuser", "testpassword123")

    os.environ["HF_ENDPOINT"] = SERVER
    os.environ["HF_TOKEN"] = token

    api = HfApi(endpoint=SERVER, token=token)

    repo_name = f"test-downloads-{int(time.time())}"
    repo_id = f"testuser/{repo_name}"

    print(f"Creating repo: {repo_id}")
    api.create_repo(repo_id=repo_id, repo_type="model")

    # Upload dummy files via folder to ensure they are all in the same commit
    files_to_upload = {
        "config.json": b'{"model_type": "gpt2"}',
        "vocab.json": b'{"a": 1, "b": 2}',
        "model.safetensors": b"fake weights data",
        "dummy.txt": b"# Test Model\n",
        "data/train.csv": b"col1,col2\n1,2",
    }

    with tempfile.TemporaryDirectory() as upload_dir:
        for filename, content in files_to_upload.items():
            file_path = os.path.join(upload_dir, filename)
            os.makedirs(os.path.dirname(file_path), exist_ok=True)
            with open(file_path, "wb") as f:
                f.write(content)

        api.upload_folder(
            folder_path=upload_dir,
            repo_id=repo_id,
            repo_type="model",
        )

    print(f"Uploaded {len(files_to_upload)} files to {repo_id}")

    # 1. Download single file
    print("\nTesting hf_hub_download (single file)...")
    path = hf_hub_download(repo_id=repo_id, filename="config.json", repo_type="model")
    with open(path, "rb") as f:
        assert f.read() == b'{"model_type": "gpt2"}', "Content mismatch for single file download"
    print("  [OK] Single file download passed")

    # 2. Download specific revision
    print("\nTesting hf_hub_download (specific revision)...")
    path_rev = hf_hub_download(repo_id=repo_id, filename="config.json", repo_type="model", revision="main")
    with open(path_rev, "rb") as f:
        assert f.read() == b'{"model_type": "gpt2"}', "Content mismatch for specific revision download"
    print("  [OK] Specific revision download passed")

    # 3. Download entire repository
    print("\nTesting snapshot_download (entire repo)...")
    snap_path = snapshot_download(repo_id=repo_id, repo_type="model")

    # Debug: print out the downloaded files
    print(f"Downloaded snapshot to {snap_path}")
    for root, dirs, files in os.walk(snap_path):
        for name in files:
            print(f"  found: {os.path.relpath(os.path.join(root, name), snap_path)}")

    assert os.path.exists(os.path.join(snap_path, "config.json")), "Missing config.json in snapshot"
    assert os.path.exists(os.path.join(snap_path, "model.safetensors")), "Missing model.safetensors in snapshot"
    assert os.path.exists(os.path.join(snap_path, "data", "train.csv")), "Missing data/train.csv in snapshot"
    print("  [OK] Entire repo download passed")

    # 4. Download with allow_patterns
    print("\nTesting snapshot_download (allow_patterns)...")
    with tempfile.TemporaryDirectory() as temp_dir:
        allow_path = snapshot_download(repo_id=repo_id, repo_type="model", allow_patterns="*.json", local_dir=temp_dir)
        assert os.path.exists(os.path.join(allow_path, "config.json")), "Missing config.json in allow_patterns snapshot"
        assert os.path.exists(os.path.join(allow_path, "vocab.json")), "Missing vocab.json in allow_patterns snapshot"
        assert not os.path.exists(os.path.join(allow_path, "model.safetensors")), "model.safetensors should not be in allow_patterns snapshot"
        print("  [OK] allow_patterns passed")

    # 5. Download with ignore_patterns
    print("\nTesting snapshot_download (ignore_patterns)...")
    with tempfile.TemporaryDirectory() as temp_dir:
        ignore_path = snapshot_download(repo_id=repo_id, repo_type="model", ignore_patterns=["*.safetensors", "data/*"], local_dir=temp_dir)
        assert os.path.exists(os.path.join(ignore_path, "config.json")), "Missing config.json in ignore_patterns snapshot"
        assert not os.path.exists(os.path.join(ignore_path, "model.safetensors")), "model.safetensors should not be in ignore_patterns snapshot"
        assert not os.path.exists(os.path.join(ignore_path, "data", "train.csv")), "data/train.csv should not be in ignore_patterns snapshot"
        print("  [OK] ignore_patterns passed")

    print("\n[SUCCESS] All tests passed!")

if __name__ == "__main__":
    main()
