/// script
requires-python = ">=3.8"
dependencies = [
  "requests",
  "pandas",
  "pyarrow",
  "fastparquet",
]
///

import os
import time
import requests
import pandas as pd
import numpy as np
import pyarrow as pa
import pyarrow.parquet as pq
from huggingface_hub import HfApi

# Enable hf_transfer for potentially testing xet/lfs fast paths
os.environ["HF_HUB_ENABLE_HF_TRANSFER"] = "1"

SERVER = os.environ.get("HF_ENDPOINT", "http://localhost:8080")

def main():
    print(f"=== Xet Hub Large Parquet Deduplication Test ===")
    print(f"Server: {SERVER}")

    api = HfApi(endpoint=SERVER)

    # 1. Register a test user if not exists
    try:
        resp = requests.post(f"{SERVER}/api/auth/register", json={
            "username": "testuser_pq",
            "password": "testpassword123"
        })
        resp.raise_for_status()
        token = resp.json()["token"]
        print(f"[OK] Registered user 'testuser_pq', got token")
    except requests.exceptions.HTTPError as e:
        if e.response.status_code == 409: # Conflict
            resp = requests.post(f"{SERVER}/api/auth/login", json={
                "username": "testuser_pq",
                "password": "testpassword123"
            })
            token = resp.json()["token"]
            print(f"[OK] Logged in as 'testuser_pq'")
        else:
            raise

    # 2. Login
    api.token = token
    me = api.whoami()
    print(f"[OK] whoami: {me['name']}")

    # 3. Create repo
    repo_name = f"parquet-dataset-{int(time.time())}"
    repo_id = f"{me['name']}/{repo_name}"
    api.create_repo(repo_id=repo_id, repo_type="dataset")
    print(f"[OK] Created dataset repo {repo_id}")

    # 4. Generate a large Parquet file (~150MB uncompressed, ~80-100MB compressed)
    # We will write it in chunks so we can easily append later without rewriting the whole file
    # This guarantees maximum binary overlap for CDC.
    print("[*] Generating Large Parquet File...")
    large_file_path = "dataset.parquet"
    
    # Create 5 chunks of 1 million rows each
    schema = pa.schema([
        ('id', pa.int64()),
        ('value1', pa.float64()),
        ('value2', pa.float64()),
        ('category', pa.string())
    ])
    
    with pq.ParquetWriter(large_file_path, schema) as writer:
        for i in range(5):
            df = pd.DataFrame({
                'id': np.arange(i*1000000, (i+1)*1000000),
                'value1': np.random.randn(1000000),
                'value2': np.random.randn(1000000),
                'category': [f'cat_{x%100}' for x in range(1000000)]
            })
            table = pa.Table.from_pandas(df, schema=schema)
            writer.write_table(table)
            
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

    # 6. Append new rows (simulate "adding a few rows to the dataset")
    print("[*] Modifying dataset (appending 50,000 rows)...")
    # Instead of rewriting the whole file (which might alter metadata at the beginning),
    # parquet allows appending row groups, though standard writers overwrite the footer.
    # To append to an existing parquet safely using pyarrow, it's easier to read it as a Dataset,
    # or just append bytes if it were CSV. For parquet, rewriting usually shifts compression blocks slightly.
    # But Xet's Content-Defined Chunking (CDC) handles byte shifts! 
    # Let's do a pure read, add rows, rewrite to test CDC's robustness.
    
    df_existing = pd.read_parquet(large_file_path)
    df_new = pd.DataFrame({
        'id': np.arange(5000000, 5050000),
        'value1': np.random.randn(50000),
        'value2': np.random.randn(50000),
        'category': [f'cat_new_{x%10}' for x in range(50000)]
    })
    df_combined = pd.concat([df_existing, df_new], ignore_index=True)
    
    # Write to a new file (this represents the modified user file)
    modified_file_path = "dataset_v2.parquet"
    df_combined.to_parquet(modified_file_path, engine='pyarrow', index=False)
    
    mod_size_mb = os.path.getsize(modified_file_path) / (1024 * 1024)
    print(f"[*] Generated {modified_file_path}: {mod_size_mb:.2f} MB")

    # 7. Upload V2
    print("[*] Uploading V2 (modified dataset)...")
    start_time = time.time()
    api.upload_file(
        path_or_fileobj=modified_file_path,
        path_in_repo="data.parquet", # overwrite the same path
        repo_id=repo_id,
        repo_type="dataset",
    )
    v2_duration = time.time() - start_time
    print(f"[OK] Uploaded V2 in {v2_duration:.2f} seconds")
    
    print(f"[INFO] V1 Time: {v1_duration:.2f}s | V2 Time: {v2_duration:.2f}s")
    if v2_duration < v1_duration * 0.75:
        print("[PASS] V2 was significantly faster! CDC deduplication successfully found common blocks despite full file rewrite.")
    else:
        print("[WARN] V2 did not show massive speedup. The Parquet rewrite might have changed too much binary structure for CDC, or upload is bound by parsing.")

if __name__ == "__main__":
    main()
