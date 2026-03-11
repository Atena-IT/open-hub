# /// script
# requires-python = ">=3.8"
# dependencies = [
#   "huggingface_hub",
#   "requests",
# ]
# ///

#!/usr/bin/env python3
"""
Roundtrip test: register user, create repo, upload file, download file, verify SHA-256.
Uses the huggingface_hub library pointed at our local Xet Hub server.
"""
import hashlib
import json
import os
import sys
import time
import tempfile
import urllib.request

SERVER = os.environ.get("HF_ENDPOINT", "http://localhost:8080")

def wait_for_server(url, timeout=60):
    """Wait for the server health endpoint."""
    deadline = time.time() + timeout
    while time.time() < deadline:
        try:
            resp = urllib.request.urlopen(f"{url}/health", timeout=3)
            data = json.loads(resp.read())
            if data.get("status") == "ok":
                print(f"[OK] Server is healthy at {url}")
                return
        except Exception:
            pass
        time.sleep(1)
    print(f"[FAIL] Server not ready at {url} after {timeout}s")
    sys.exit(1)

def register_user(base_url, username, password):
    """Register a user and return the API token."""
    data = json.dumps({"username": username, "password": password}).encode()
    req = urllib.request.Request(
        f"{base_url}/api/auth/register",
        data=data,
        headers={"Content-Type": "application/json"},
    )
    try:
        resp = urllib.request.urlopen(req)
        result = json.loads(resp.read())
        print(f"[OK] Registered user '{username}', got token: {result['token'][:20]}...")
        return result["token"]
    except urllib.error.HTTPError as e:
        body = e.read().decode()
        # If user already exists, try login
        if e.code == 409:
            return login_user(base_url, username, password)
        print(f"[FAIL] Register failed: {e.code} {body}")
        sys.exit(1)

def login_user(base_url, username, password):
    """Login and return the API token."""
    data = json.dumps({"username": username, "password": password}).encode()
    req = urllib.request.Request(
        f"{base_url}/api/auth/login",
        data=data,
        headers={"Content-Type": "application/json"},
    )
    resp = urllib.request.urlopen(req)
    result = json.loads(resp.read())
    print(f"[OK] Logged in as '{username}', got token: {result['token'][:20]}...")
    return result["token"]

def main():
    print(f"=== Xet Hub Roundtrip Test ===")
    print(f"Server: {SERVER}")

    wait_for_server(SERVER)

    # 1. Register
    token = register_user(SERVER, "testuser", "testpassword123")

    try:
        import os
        os.environ['HF_ENDPOINT'] = os.environ.get('HF_ENDPOINT', 'http://localhost:8080')
        from huggingface_hub import HfApi
    except ImportError:
        print("[SKIP] huggingface_hub not installed. Install with: pip install huggingface_hub")
        print("[OK] Auth roundtrip passed (register + login)")
        return

    # 2. Use huggingface_hub
    api = HfApi(endpoint=SERVER, token=token)

    # Whoami
    user_info = api.whoami()
    print(f"[OK] whoami: {user_info['name']}")

    # 3. Create repo
    repo_name = "test-model"
    try:
        api.create_repo(repo_id=f"testuser/{repo_name}", repo_type="model")
        print(f"[OK] Created repo testuser/{repo_name}")
    except Exception as e:
        if "already exists" in str(e) or "409" in str(e):
            print(f"[OK] Repo testuser/{repo_name} already exists")
        else:
            raise

    # 4. Create a test file
    test_content = b'{"model_type": "gpt2", "test": true, "timestamp": ' + str(time.time()).encode() + b'}'
    sha_upload = hashlib.sha256(test_content).hexdigest()
    print(f"[OK] Test content SHA-256: {sha_upload}")

    # 5. Upload via huggingface_hub
    with tempfile.NamedTemporaryFile(suffix=".json", delete=False, mode="wb") as f:
        f.write(test_content)
        tmp_path = f.name

    try:
        api.upload_file(
            path_or_fileobj=tmp_path,
            path_in_repo="config.json",
            repo_id=f"testuser/{repo_name}",
            repo_type="model",
        )
        print(f"[OK] Uploaded config.json")
    finally:
        os.unlink(tmp_path)

    # 6. Download and verify
    downloaded_path = api.hf_hub_download(
        repo_id=f"testuser/{repo_name}",
        filename="config.json",
        repo_type="model",
    )
    with open(downloaded_path, "rb") as f:
        downloaded_content = f.read()

    sha_download = hashlib.sha256(downloaded_content).hexdigest()
    print(f"[OK] Downloaded config.json, SHA-256: {sha_download}")

    if sha_upload == sha_download:
        print("[PASS] SHA-256 match! Roundtrip successful.")
    else:
        print(f"[FAIL] SHA-256 mismatch! Upload={sha_upload} Download={sha_download}")
        sys.exit(1)

if __name__ == "__main__":
    main()
