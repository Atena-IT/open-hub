import os
import secrets
from huggingface_hub import HfApi, login, whoami
import requests

# Enable hf_transfer for potentially testing xet/lfs fast paths
os.environ["HF_HUB_ENABLE_HF_TRANSFER"] = "1"

SERVER = os.environ.get("HF_ENDPOINT", "http://localhost:8080")

def main():
    print(f"=== Xet Hub Large File Test ===")
    print(f"Server: {SERVER}")

    api = HfApi(endpoint=SERVER)

    # 1. Register a test user if not exists
    try:
        resp = requests.post(f"{SERVER}/api/auth/register", json={
            "username": "testuser_lfs",
            "password": "testpassword123"
        })
        resp.raise_for_status()
        token = resp.json()["token"]
        print(f"[OK] Registered user 'testuser_lfs', got token")
    except requests.exceptions.HTTPError as e:
        if e.response.status_code == 409: # Conflict
            resp = requests.post(f"{SERVER}/api/auth/login", json={
                "username": "testuser_lfs",
                "password": "testpassword123"
            })
            token = resp.json()["token"]
            print(f"[OK] Logged in as 'testuser_lfs'")
        else:
            raise

    # 2. Login
    api.token = token

    me = api.whoami()
    print(f"[OK] whoami: {me['name']}")

    # 3. Create repo
    repo_name = "large-model-test"
    repo_id = f"{me['name']}/{repo_name}"
    try:
        api.create_repo(repo_id=repo_id, repo_type="model")
        print(f"[OK] Created repo {repo_id}")
    except Exception as e:
        if "already exists" in str(e):
            print(f"[OK] Repo {repo_id} already exists")
        else:
            raise

    # 4. Generate a large file (> 10MB to trigger LFS logic)
    print("[*] Generating 15MB file...")
    large_file_path = "large_model.bin"
    with open(large_file_path, "wb") as f:
        f.write(secrets.token_bytes(15 * 1024 * 1024))
    
    print("[*] Uploading large file...")
    api.upload_file(
        path_or_fileobj=large_file_path,
        path_in_repo="model.bin",
        repo_id=repo_id,
        repo_type="model",
    )
    print(f"[OK] Uploaded {large_file_path}")

    # 5. Download the file
    print("[*] Downloading large file...")
    downloaded_path = api.hf_hub_download(
        repo_id=repo_id,
        filename="model.bin",
        repo_type="model",
        force_download=True
    )
    
    # 6. Verify contents
    with open(large_file_path, "rb") as f1, open(downloaded_path, "rb") as f2:
        if f1.read() == f2.read():
            print("[PASS] File contents match! Large file roundtrip successful.")
        else:
            print("[FAIL] File contents DO NOT match!")

if __name__ == "__main__":
    main()
