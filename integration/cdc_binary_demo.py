# /// script
# requires-python = ">=3.8"
# dependencies = [
#   "requests",
# ]
# ///

import os
import secrets
import time
import os
os.environ['HF_ENDPOINT'] = os.environ.get('HF_ENDPOINT', 'http://localhost:8080')
from huggingface_hub import HfApi
import requests

# Enable hf_transfer for potentially testing xet/lfs fast paths
os.environ["HF_HUB_ENABLE_HF_TRANSFER"] = "1"

SERVER = os.environ.get("HF_ENDPOINT", "http://localhost:8080")

def main():
    print(f"=== Xet Hub Deduplication Test ===")
    print(f"Server: {SERVER}")

    api = HfApi(endpoint=SERVER)

    # 1. Register a test user if not exists
    try:
        resp = requests.post(f"{SERVER}/api/auth/register", json={
            "username": "testuser_dedup",
            "password": "testpassword123"
        })
        resp.raise_for_status()
        token = resp.json()["token"]
        print(f"[OK] Registered user 'testuser_dedup', got token")
    except requests.exceptions.HTTPError as e:
        if e.response.status_code == 409: # Conflict
            resp = requests.post(f"{SERVER}/api/auth/login", json={
                "username": "testuser_dedup",
                "password": "testpassword123"
            })
            token = resp.json()["token"]
            print(f"[OK] Logged in as 'testuser_dedup'")
        else:
            raise

    # 2. Login
    api.token = token

    me = api.whoami()
    print(f"[OK] whoami: {me['name']}")

    # 3. Create repo
    repo_name = f"dedup-test-{int(time.time())}"
    repo_id = f"{me['name']}/{repo_name}"
    api.create_repo(repo_id=repo_id, repo_type="model")
    print(f"[OK] Created repo {repo_id}")

    # 4. Generate a large file (> 20MB to ensure multiple chunks)
    print("[*] Generating 20MB file...")
    large_file_path = "dedup_model.bin"
    # Keep some predictable data to improve chance of chunk boundary stability
    base_data = os.urandom(20 * 1024 * 1024)
    with open(large_file_path, "wb") as f:
        f.write(base_data)
    
    print("[*] Uploading v1 (initial upload)...")
    start_time = time.time()
    api.upload_file(
        path_or_fileobj=large_file_path,
        path_in_repo="model.bin",
        repo_id=repo_id,
        repo_type="model",
    )
    v1_duration = time.time() - start_time
    print(f"[OK] Uploaded v1 in {v1_duration:.2f} seconds")

    # 5. Modify the file slightly (append 100 bytes)
    print("[*] Modifying file (appending 100 bytes)...")
    with open(large_file_path, "ab") as f:
        f.write(secrets.token_bytes(100))
    
    print("[*] Uploading v2 (modified file)...")
    start_time = time.time()
    api.upload_file(
        path_or_fileobj=large_file_path,
        path_in_repo="model_v2.bin",
        repo_id=repo_id,
        repo_type="model",
    )
    v2_duration = time.time() - start_time
    print(f"[OK] Uploaded v2 in {v2_duration:.2f} seconds")
    print(f"[INFO] V1 Time: {v1_duration:.2f}s | V2 Time: {v2_duration:.2f}s")
    if v2_duration < v1_duration / 2:
        print("[PASS] V2 was significantly faster, deduplication likely working!")
    else:
        print("[WARN] V2 took a similar amount of time, check server logs to see if full xorbs were uploaded.")

if __name__ == "__main__":
    main()
