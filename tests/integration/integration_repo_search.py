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
import urllib.request
import urllib.error
import os
os.environ['HF_ENDPOINT'] = os.environ.get('HF_ENDPOINT', 'http://localhost:8080')
from huggingface_hub import HfApi

ENDPOINT = os.environ.get("HF_ENDPOINT", "http://localhost:8080")
os.environ["HF_ENDPOINT"] = ENDPOINT
os.environ["CURL_CA_BUNDLE"] = ""

def login_user(base_url, username, password):
    data = json.dumps({"username": username, "password": password}).encode()
    req = urllib.request.Request(
        f"{base_url}/api/auth/login",
        data=data,
        headers={"Content-Type": "application/json"},
    )
    try:
        resp = urllib.request.urlopen(req)
        result = json.loads(resp.read())
        return result["token"]
    except urllib.error.HTTPError as e:
        print(f"Login failed: {e.code} {e.read().decode()}")
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
        print(f"Register failed: {e.code} {e.read().decode()}")
        sys.exit(1)

def main():
    print(f"Using endpoint: {ENDPOINT}")
    token = register_user(ENDPOINT, "testuser", "testpassword123")
    os.environ["HF_TOKEN"] = token
    
    api = HfApi()
    repo_id = "testuser/test-repo-management"
    
    print("\n=== Testing Repository Management ===")
    
    # 1. Create Repo
    print(f"\n[1] Creating repo: {repo_id}")
    try:
        url = api.create_repo(repo_id, exist_ok=True, repo_type="model")
        print(f"Success! Repo URL: {url}")
    except Exception as e:
        print(f"Failed to create repo: {e}")

    # 2. Repo Info
    print(f"\n[2] Fetching repo_info for: {repo_id}")
    try:
        info = api.repo_info(repo_id, repo_type="model")
        print(f"Success! Repo info retrieved. Private: {info.private}")
    except Exception as e:
        print(f"Failed to fetch repo info: {e}")

    # 3. Update Repo Visibility
    print(f"\n[3] Updating repo visibility to private for: {repo_id}")
    try:
        if hasattr(api, 'update_repo_settings'):
            api.update_repo_settings(repo_id=repo_id, private=True, repo_type="model")
        else:
            api.update_repo_visibility(repo_id=repo_id, private=True, repo_type="model")
        info = api.repo_info(repo_id, repo_type="model")
        print(f"Success! Repo is now private: {info.private}")
    except Exception as e:
        print(f"Failed to update repo visibility: {e}. (This might not be implemented yet)")

    # 4. Delete Repo
    print(f"\n[4] Deleting repo: {repo_id}")
    try:
        api.delete_repo(repo_id, repo_type="model")
        print("Success! Repo deleted.")
    except Exception as e:
        print(f"Failed to delete repo: {e}. (This might not be implemented yet)")

    print("\n=== Testing Search APIs ===")
    
    # 5. List Models
    print("\n[5] Listing models (limit=2)")
    try:
        models = list(api.list_models(limit=2))
        print(f"Success! Found {len(models)} models.")
        for m in models:
            print(f" - {m.id}")
    except Exception as e:
        print(f"Failed to list models: {e}. (This might not be implemented yet)")

    # 6. List Datasets
    print("\n[6] Listing datasets (limit=2)")
    try:
        datasets = list(api.list_datasets(limit=2))
        print(f"Success! Found {len(datasets)} datasets.")
        for d in datasets:
            print(f" - {d.id}")
    except Exception as e:
        print(f"Failed to list datasets: {e}. (This might not be implemented yet)")

    print("\nIntegration test script completed.")

if __name__ == "__main__":
    main()
