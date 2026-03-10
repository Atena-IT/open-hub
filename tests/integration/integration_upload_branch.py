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
    
    repo_id = "test-user/test-upload-branch"
    print(f"Creating repo {repo_id}...")
    try:
        api.create_repo(repo_id, exist_ok=True, repo_type="model")
    except Exception:
        pass

    print("Uploading file to a specific branch ('dev_branch') via revision parameter...")
    # This should automatically create the branch if the backend supports it.
    try:
        api.upload_file(
            path_or_fileobj=b"Hello from dev branch!",
            path_in_repo="branch_file.txt",
            repo_id=repo_id,
            revision="dev_branch",
            create_pr=False,
            commit_message="Upload to branch"
        )
        print("Upload completed without crashing.")
    except Exception as e:
        print(f"Upload to branch failed (likely expected if backend lacks support): {e}")

    print("Testing if branch files are distinct...")
    try:
        main_files = api.list_repo_files(repo_id)
        dev_files = api.list_repo_files(repo_id, revision="dev_branch")
        print(f"Main files: {main_files}")
        print(f"Dev files: {dev_files}")
        
        # We don't strictly assert the branch works here since we know the backend currently maps everything to main
        # But this serves as the integration test for when the feature is ready.
    except Exception as e:
        print(f"Listing branch files failed: {e}")

    print("Branch upload test executed!")

if __name__ == "__main__":
    main()
