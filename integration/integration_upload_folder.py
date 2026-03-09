/// script
requires-python = ">=3.8"
dependencies = [
  "huggingface_hub[hf_transfer]",
]
///
import os
import shutil
import tempfile
from huggingface_hub import HfApi

os.environ["HF_ENDPOINT"] = os.environ.get("HF_ENDPOINT", "http://localhost:8080")
# Disable SSL verification for local dev
os.environ["CURL_CA_BUNDLE"] = ""

def main():
    api = HfApi()
    repo_id = "test-user/test-upload-folder"
    print(f"Creating repo {repo_id}...")
    try:
        api.create_repo(repo_id, exist_ok=True, repo_type="model")
    except Exception as e:
        print(f"Failed to create repo: {e}")
        return

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
    print(f"Files in repo: {files}")
    assert "config.json" in files
    assert "pytorch_model.bin" in files
    print("Upload folder test passed!")

if __name__ == "__main__":
    main()
