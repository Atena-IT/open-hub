import requests

url = "http://localhost:9000/xet-storage/xorbs/1a/77/1a770946010248dc85ba3e2534bd70513d0d155b3c2eb84f195593702c6d87d4?x-id=GetObject&X-Amz-Algorithm=AWS4-HMAC-SHA256&X-Amz-Credential=minioadmin%2F20260309%2Fus-east-1%2Fs3%2Faws4_request&X-Amz-Date=20260309T134528Z&X-Amz-Expires=3600&X-Amz-SignedHeaders=host&X-Amz-Signature=1b1f5b7d2e1ade1aaa97929088de9392fe1e002f35eb8c515fdc6c46542da8ad"

resp = requests.get(url)
print("Status:", resp.status_code)
print(resp.text[:200])
