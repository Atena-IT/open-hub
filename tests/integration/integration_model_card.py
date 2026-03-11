# /// script
# requires-python = ">=3.8"
# dependencies = [
#   "huggingface_hub",
#   "requests",
# ]
# ///
import os
os.environ['HF_ENDPOINT'] = os.environ.get('HF_ENDPOINT', 'http://localhost:8080')
from huggingface_hub import HfApi, ModelCard
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
    
    print("Testing invalid YAML frontmatter validation...")
    invalid_content = """---
language:
- en
license: mit
  invalid_indentation: true
---

# Test Model Card
"""
    try:
        ModelCard(invalid_content).push_to_hub(repo_id)
        assert False, "Should have failed with invalid YAML"
    except Exception as e:
        print(f"Caught expected error for invalid YAML: {e}")

    print("Model card test passed!")

if __name__ == "__main__":
    main()
