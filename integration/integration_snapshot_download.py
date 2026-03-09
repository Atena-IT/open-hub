# /// script
# requires-python = ">=3.8"
# dependencies = [
#   "huggingface_hub[hf_transfer]",
# ]
# ///
import os
import tempfile
import os
os.environ['HF_ENDPOINT'] = os.environ.get('HF_ENDPOINT', 'http://localhost:8080')
from huggingface_hub import HfApi, snapshot_download

os.environ["HF_ENDPOINT"] = os.environ.get("HF_ENDPOINT", "http://localhost:8080")
os.environ["CURL_CA_BUNDLE"] = ""

def main():
    api = HfApi()
    repo_id = "test-user/test-snapshot-download"
    print(f"Creating repo {repo_id}...")
    try:
        api.create_repo(repo_id, exist_ok=True, repo_type="model")
    except Exception as e:
        print(f"Failed to create repo: {e}")
        return

    api.upload_file(
        path_or_fileobj=b"Snapshot content 1",
        path_in_repo="snap1.txt",
        repo_id=repo_id,
    )
    api.upload_file(
        path_or_fileobj=b"Snapshot content 2",
        path_in_repo="snap2.txt",
        repo_id=repo_id,
    )
    
    print("Downloading snapshot...")
    with tempfile.TemporaryDirectory() as cache_dir:
        path = snapshot_download(repo_id=repo_id, cache_dir=cache_dir)
        print(f"Snapshot downloaded to {path}")
        
        files = os.listdir(path)
        print(f"Downloaded files: {files}")
        assert "snap1.txt" in files or os.path.exists(os.path.join(path, "snap1.txt"))
        assert "snap2.txt" in files or os.path.exists(os.path.join(path, "snap2.txt"))
        print("Snapshot download test passed!")

if __name__ == "__main__":
    main()
