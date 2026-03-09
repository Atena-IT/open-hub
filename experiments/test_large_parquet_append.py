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
    print(f"=== Xet Hub Large Parquet Deduplication Test (Append Mode) ===")
    print(f"Server: {SERVER}")

    api = HfApi(endpoint=SERVER)

    # 1. Register a test user if not exists
    try:
        resp = requests.post(f"{SERVER}/api/auth/register", json={
            "username": "testuser_pq2",
            "password": "testpassword123"
        })
        resp.raise_for_status()
        token = resp.json()["token"]
    except requests.exceptions.HTTPError as e:
        if e.response.status_code == 409: # Conflict
            resp = requests.post(f"{SERVER}/api/auth/login", json={
                "username": "testuser_pq2",
                "password": "testpassword123"
            })
            token = resp.json()["token"]
        else:
            raise

    api.token = token
    me = api.whoami()

    # 3. Create repo
    repo_name = f"parquet-dataset-append-{int(time.time())}"
    repo_id = f"{me['name']}/{repo_name}"
    api.create_repo(repo_id=repo_id, repo_type="dataset")
    print(f"[OK] Created dataset repo {repo_id}")

    # 4. Generate a large Parquet file using fastparquet to allow appending later
    print("[*] Generating Large Parquet File (~100MB)...")
    large_file_path = "dataset_append.parquet"
    
    # Create 5 million rows in chunks
    for i in range(5):
        df = pd.DataFrame({
            'id': np.arange(i*1000000, (i+1)*1000000),
            'value1': np.random.randn(1000000),
            'value2': np.random.randn(1000000),
            'category': [f'cat_{x%100}' for x in range(1000000)]
        })
        # Write first chunk normally, append subsequent chunks
        df.to_parquet(large_file_path, engine='fastparquet', append=(i > 0))
            
    file_size_mb = os.path.getsize(large_file_path) / (1024 * 1024)
    print(f"[*] Generated {large_file_path}: {file_size_mb:.2f} MB")

    # 5. Upload V1
    print("[*] Uploading V1 (initial upload)...")
    start_time = time.time()
    api.upload_file(
        path_or_fileobj=large_file_path,
        path_in_repo="data.parquet",
        repo_id=repo_id,
        repo_type="dataset",
    )
    v1_duration = time.time() - start_time
    print(f"[OK] Uploaded V1 in {v1_duration:.2f} seconds")

    # 6. Append new rows (this only appends bytes to the file, keeping previous chunks identical)
    print("[*] Modifying dataset (appending 1,000,000 rows)...")
    
    df_new = pd.DataFrame({
        'id': np.arange(5000000, 6000000),
        'value1': np.random.randn(1000000),
        'value2': np.random.randn(1000000),
        'category': [f'cat_new_{x%10}' for x in range(1000000)]
    })
    
    # Append directly to the same file!
    df_new.to_parquet(large_file_path, engine='fastparquet', append=True)
    
    mod_size_mb = os.path.getsize(large_file_path) / (1024 * 1024)
    print(f"[*] Generated modified {large_file_path}: {mod_size_mb:.2f} MB")

    # 7. Upload V2
    print("[*] Uploading V2 (modified dataset)...")
    start_time = time.time()
    api.upload_file(
        path_or_fileobj=large_file_path,
        path_in_repo="data.parquet",
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
