import os
import time
import requests
import pandas as pd
import numpy as np
from huggingface_hub import HfApi

# Enable hf_transfer for potentially testing xet/lfs fast paths
os.environ["HF_HUB_ENABLE_HF_TRANSFER"] = "1"

SERVER = os.environ.get("HF_ENDPOINT", "http://localhost:8080")

def main():
    print(f"=== Xet Hub Large CSV Deduplication Test (CDC) ===")
    print(f"Server: {SERVER}")

    api = HfApi(endpoint=SERVER)

    try:
        resp = requests.post(f"{SERVER}/api/auth/register", json={
            "username": "testuser_csv",
            "password": "testpassword123"
        })
        token = resp.json()["token"]
    except requests.exceptions.HTTPError as e:
        if e.response.status_code == 409: # Conflict
            resp = requests.post(f"{SERVER}/api/auth/login", json={
                "username": "testuser_csv",
                "password": "testpassword123"
            })
            token = resp.json()["token"]
        else:
            raise

    api.token = token
    me = api.whoami()

    repo_name = f"csv-dataset-append-{int(time.time())}"
    repo_id = f"{me['name']}/{repo_name}"
    api.create_repo(repo_id=repo_id, repo_type="dataset")
    print(f"[OK] Created dataset repo {repo_id}")

    print("[*] Generating Large CSV File (~150MB)...")
    large_file_path = "dataset.csv"
    
    # Create ~5 million rows in chunks
    with open(large_file_path, "w") as f:
        f.write("id,value1,value2,category\n")
        for i in range(5):
            df = pd.DataFrame({
                'id': np.arange(i*1000000, (i+1)*1000000),
                'value1': np.random.randn(1000000),
                'value2': np.random.randn(1000000),
                'category': [f'cat_{x%100}' for x in range(1000000)]
            })
            df.to_csv(f, header=False, index=False)
            
    file_size_mb = os.path.getsize(large_file_path) / (1024 * 1024)
    print(f"[*] Generated {large_file_path}: {file_size_mb:.2f} MB")

    print("[*] Uploading V1 (initial upload)...")
    start_time = time.time()
    api.upload_file(
        path_or_fileobj=large_file_path,
        path_in_repo="data.csv",
        repo_id=repo_id,
        repo_type="dataset",
    )
    v1_duration = time.time() - start_time
    print(f"[OK] Uploaded V1 in {v1_duration:.2f} seconds")

    print("[*] Modifying dataset (appending 1,000,000 rows)...")
    
    with open(large_file_path, "a") as f:
        df_new = pd.DataFrame({
            'id': np.arange(5000000, 6000000),
            'value1': np.random.randn(1000000),
            'value2': np.random.randn(1000000),
            'category': [f'cat_new_{x%10}' for x in range(1000000)]
        })
        df_new.to_csv(f, header=False, index=False)
    
    mod_size_mb = os.path.getsize(large_file_path) / (1024 * 1024)
    print(f"[*] Generated modified {large_file_path}: {mod_size_mb:.2f} MB")

    print("[*] Uploading V2 (modified dataset)...")
    start_time = time.time()
    api.upload_file(
        path_or_fileobj=large_file_path,
        path_in_repo="data.csv",
        repo_id=repo_id,
        repo_type="dataset",
    )
    v2_duration = time.time() - start_time
    print(f"[OK] Uploaded V2 in {v2_duration:.2f} seconds")
    
    print(f"[INFO] V1 Time: {v1_duration:.2f}s | V2 Time: {v2_duration:.2f}s")
    if v2_duration < v1_duration * 0.5:
        print("[PASS] V2 was significantly faster! CDC deduplication successfully found common blocks!")
    else:
        print("[WARN] V2 did not show massive speedup.")

if __name__ == "__main__":
    main()
