/// script
requires-python = ">=3.8"
dependencies = [
  "huggingface_hub[hf_transfer]",
]
///
import os
from huggingface_hub import HfApi, ModelCard

os.environ["HF_ENDPOINT"] = os.environ.get("HF_ENDPOINT", "http://localhost:8080")
os.environ["CURL_CA_BUNDLE"] = ""

def main():
    api = HfApi()
    repo_id = "test-user/test-model-card"
    print(f"Creating repo {repo_id}...")
    try:
        api.create_repo(repo_id, exist_ok=True, repo_type="model")
    except Exception as e:
        print(f"Failed to create repo: {e}")
        return

    content = """
---
language:
- en
license: mit
tags:
- text-classification
---

# Test Model Card
This is a mock model card for integration testing.
"""
    print("Creating and pushing model card...")
    card = ModelCard(content)
    card.push_to_hub(repo_id)
    
    print("Listing files...")
    files = api.list_repo_files(repo_id)
    print(f"Files in repo: {files}")
    assert "README.md" in files
    
    print("Fetching model card...")
    fetched_card = ModelCard.load(repo_id)
    print(f"Fetched card tags: {fetched_card.data.tags}")
    assert "text-classification" in fetched_card.data.tags
    print("Model card test passed!")

if __name__ == "__main__":
    main()
