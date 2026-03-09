/// script
requires-python = ">=3.8"
dependencies = [
  "huggingface_hub[hf_transfer]",
]
///
import os
from huggingface_hub import HfApi, CommitOperationAdd

os.environ["HF_ENDPOINT"] = os.environ.get("HF_ENDPOINT", "http://localhost:8080")
os.environ["CURL_CA_BUNDLE"] = ""

def main():
    api = HfApi()
    repo_id = "test-user/test-create-commit"
    print(f"Creating repo {repo_id}...")
    try:
        api.create_repo(repo_id, exist_ok=True, repo_type="model")
    except Exception as e:
        print(f"Failed to create repo: {e}")
        return

    operations = [
        CommitOperationAdd(path_in_repo="test1.txt", path_or_fileobj=b"Content 1"),
        CommitOperationAdd(path_in_repo="test2.txt", path_or_fileobj=b"Content 2")
    ]
    
    print("Creating commit...")
    api.create_commit(
        repo_id=repo_id,
        operations=operations,
        commit_message="Test create_commit"
    )
    
    print("Listing files...")
    files = api.list_repo_files(repo_id)
    print(f"Files in repo: {files}")
    assert "test1.txt" in files
    assert "test2.txt" in files
    print("Create commit test passed!")

if __name__ == "__main__":
    main()
