# /// script
# requires-python = ">=3.8"
# dependencies = [
#   "huggingface_hub[hf_transfer]",
#   "fsspec",
#   "requests",
# ]
# ///
import os
os.environ['HF_ENDPOINT'] = os.environ.get('HF_ENDPOINT', 'http://localhost:8080')
from huggingface_hub import HfApi, HfFileSystem
import requests

os.environ["CURL_CA_BUNDLE"] = ""

def get_token(username="test-user", password="password"):
    endpoint = os.environ["HF_ENDPOINT"]
    resp = requests.post(f"{endpoint}/api/auth/register", json={"username": username, "password": password})
    if resp.status_code == 409:
        resp = requests.post(f"{endpoint}/api/auth/login", json={"username": username, "password": password})
    resp.raise_for_status()
    return resp.json()["token"]

def main():
    token = get_token()
    os.environ["HF_TOKEN"] = token
    api = HfApi(token=token)
    repo_id = "test-user/test-hffs"
    print(f"Creating repo {repo_id}...")
    try:
        api.create_repo(repo_id, exist_ok=True, repo_type="model")
    except Exception as e:
        pass # Might already exist

    fs = HfFileSystem(token=token)
    
    print("Writing files using HfFileSystem...")
    with fs.open(f"{repo_id}/test_fs.txt", "w") as f:
        f.write("Hello from HfFileSystem")
        
    print("Globbing files...")
    files = fs.glob(f"{repo_id}/*")
    print(f"Globbed files: {files}")
    
    assert any("test_fs.txt" in f for f in files)
    
    print("Reading file...")
    with fs.open(f"{repo_id}/test_fs.txt", "r") as f:
        content = f.read()
        print(f"Content: {content}")
        assert content == "Hello from HfFileSystem"
        
    print("HfFileSystem test passed!")

if __name__ == "__main__":
    main()
