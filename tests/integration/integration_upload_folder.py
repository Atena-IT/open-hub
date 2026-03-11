# /// script
# requires-python = ">=3.8"
# dependencies = [
#   "huggingface_hub",
#   "requests"
# ]
# ///
import os
import shutil
import tempfile
import requests
os.environ["HF_ENDPOINT"] = os.environ.get("HF_ENDPOINT", "http://localhost:8080")
os.environ["CURL_CA_BUNDLE"] = ""

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
    
    repo_id = "test-user/test-upload-folder"
    print(f"Creating repo {repo_id}...")
    try:
        api.create_repo(repo_id, exist_ok=True, repo_type="model")
    except Exception as e:
        pass

    # Create dummy folder
    with tempfile.TemporaryDirectory() as tmpdir:
        with open(os.path.join(tmpdir, "config.json"), "w") as f:
            f.write('{"model_type": "dummy"}')
        with open(os.path.join(tmpdir, "pytorch_model.bin"), "wb") as f:
            f.write(b"dummy binary content 12345")
        
        print("Uploading folder...")
        api.upload_folder(
            folder_path=tmpdir,
            repo_id=repo_id,
            repo_type="model",
            commit_message="Upload folder test"
        )
    
    print("Listing files...")
    files = api.list_repo_files(repo_id)
    print(f"Files in main: {files}")
    assert "config.json" in files
    assert "pytorch_model.bin" in files
    
    print("Upload folder test passed!")

if __name__ == "__main__":
    main()
