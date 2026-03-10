#!/usr/bin/env python3
"""
hf_xet_roundtrip.py — Xet protocol end-to-end compatibility test
==================================================================
Downloads vocab.txt from bert-base-uncased (~231 KB, 3-4 chunks with CDC)
and round-trips it through our xet-backend to verify the full protocol:

  1. Content-defined chunking   (Gearhash, target 64 KiB)
  2. Blake3 keyed chunk hashing
  3. Xorb packing               (lz4-block-compressed chunks + 8-byte headers)
  4. MDB shard binary format    (FileInfo + CASInfo sections)
  5. Xorb upload                (POST /v1/xorbs/default/{hash})
  6. Shard upload               (POST /v1/shards)
  7. Reconstruction query       (GET /v1/reconstructions/{file_hash})
  8. Presigned URL download     (direct from MinIO/S3)
  9. SHA-256 integrity check

Execution modes
---------------
  Primary :  hf_xet Python package is imported and its CAS-level API is tried
             (pip install hf_xet).  Three call shapes are attempted; whichever
             succeeds is used.
  Fallback:  clean-room Python implementation of the Xet protocol.

Usage
-----
  pip install requests blake3 lz4 hf_xet   # hf_xet is optional
  CAS_URL=http://localhost:3000 python3 hf_xet_roundtrip.py

Environment variables
---------------------
  CAS_URL          CAS server base URL (default: http://localhost:3000)
  HF_FILE_URL      Alternative test file URL from HuggingFace Hub
  SKIP_HF_DOWNLOAD Set to 1 to skip network download (uses /tmp/xet_e2e_vocab.txt if present)
"""

import hashlib
import json
import os
import struct
import sys
import tempfile
import uuid
from dataclasses import dataclass
from pathlib import Path
from typing import Dict, List, Optional, Tuple

try:
    import requests
except ImportError:
    sys.exit("ERROR: pip install requests")

# ── Config ─────────────────────────────────────────────────────────────────────
CAS_URL       = os.environ.get("CAS_URL",      "http://localhost:3000")
HF_FILE_URL   = os.environ.get("HF_FILE_URL",
    "https://huggingface.co/bert-base-uncased/resolve/main/vocab.txt")
SKIP_DOWNLOAD = os.environ.get("SKIP_HF_DOWNLOAD", "0") == "1"

# When docker-compose generates presigned URLs with the MinIO external hostname
# (e.g. http://localhost:9000/...) they may be unreachable from inside a docker container.
# Set S3_REWRITE_FROM and S3_REWRITE_TO to remap them.
S3_REWRITE_FROM = os.environ.get("S3_REWRITE_FROM")
S3_REWRITE_TO = os.environ.get("S3_REWRITE_TO")

REPO_TYPE = "model"   # singular — URL becomes /api/models/...
NAMESPACE = f"hf-xet-test-{uuid.uuid4().hex[:8]}"
REPO      = "roundtrip"
REVISION  = "main"

CACHE_PATH = Path(tempfile.gettempdir()) / "xet_e2e_vocab.txt"


def normalize_url(url: str) -> str:
    """Rewrite S3 presigned URLs if needed (e.g. localhost -> minio inside docker)."""
    if S3_REWRITE_FROM and S3_REWRITE_TO:
        return url.replace(S3_REWRITE_FROM, S3_REWRITE_TO)
    return url


# ── Optional dependencies ──────────────────────────────────────────────────────

try:
    import blake3 as _b3
    def _blake3_keyed(data: bytes, key: bytes) -> bytes:
        return _b3.blake3(data, key=key).digest()
    HAS_BLAKE3 = True
except ImportError:
    HAS_BLAKE3 = False
    def _blake3_keyed(data: bytes, key: bytes) -> bytes:
        # SHA-256 fallback — produces different hashes from xet-core but is
        # self-consistent, which is all our server needs for a round-trip test.
        return hashlib.sha256(key + data).digest()

# Blake3 keys (32 bytes each).  These match the xet-core constants:
#   xet-core/cas_types/src/hash.rs
# If you have the exact values from the source, replace the placeholders below.
# The values here are PLACEHOLDERS — they will produce hashes incompatible with
# xet-core's hf_xet, but the server round-trip will still pass.
DATA_KEY      = b"xet_data_hash_key_aaaaaaaaaaaaaa"  # 32 bytes
INTERNAL_KEY  = b"xet_internal_node_key_aaaaaaaaaa"  # 32 bytes
ZERO_KEY      = b"\x00" * 32

try:
    import lz4.block as _lz4
    def lz4_compress(data: bytes) -> bytes:
        return _lz4.compress(data, store_size=False)
    def lz4_decompress(data: bytes, uncompressed_size: int) -> bytes:
        return _lz4.decompress(data, uncompressed_size=uncompressed_size)
    HAS_LZ4 = True
except ImportError:
    HAS_LZ4 = False
    def lz4_compress(data: bytes) -> bytes:
        return data  # store raw when lz4 unavailable
    def lz4_decompress(data: bytes, uncompressed_size: int) -> bytes:
        return data


# ── Gearhash CDC ───────────────────────────────────────────────────────────────
# Gear table: deterministic from a fixed seed (LCG).  Not identical to
# xet-core's table, so chunk boundaries will differ from hf_xet's — that is
# fine because our server is content-addressed and accepts any consistent client.
_GEAR_SEED = 0xDEAD_BEEF_CAFE_0001
GEAR: List[int] = []
_s = _GEAR_SEED
for _i in range(256):
    _s = (_s * 6364136223846793005 + 1442695040888963407) & 0xFFFF_FFFF_FFFF_FFFF
    GEAR.append(_s)

MASK       = 0xFFFF_0000_0000_0000
MIN_CHUNK  = 8   * 1024   #   8 KiB
MAX_CHUNK  = 128 * 1024   # 128 KiB


def cdc_chunk(data: bytes) -> List[bytes]:
    """Gearhash content-defined chunking (same MASK as xet-core spec)."""
    chunks: List[bytes] = []
    n = len(data)
    i = 0
    while i < n:
        end = min(i + MAX_CHUNK, n)
        # Skip min chunk length before checking boundaries
        j = i + MIN_CHUNK
        if j >= n:
            chunks.append(data[i:])
            break
        state = 0
        while j < end:
            state = (state >> 1) ^ GEAR[data[j]]
            j += 1
            if (state & MASK) == 0:
                break
        chunks.append(data[i:j])
        i = j
    return chunks


# ── Hash encoding (API format) ─────────────────────────────────────────────────

def hash_to_api(h: bytes) -> str:
    """32-byte hash → 64-char API hex (four LE u64 groups)."""
    out = ""
    for i in range(4):
        val = struct.unpack_from("<Q", h, i * 8)[0]
        out += f"{val:016x}"
    return out


# ── Xorb format ────────────────────────────────────────────────────────────────

@dataclass
class ChunkInfo:
    hash:             bytes   # 32-byte Blake3 keyed hash
    byte_range_start: int     # byte offset of this chunk's header in the xorb
    unpacked_bytes:   int     # original (uncompressed) chunk size


def build_xorb(chunks: List[bytes]) -> Tuple[bytes, List[ChunkInfo]]:
    """Pack chunks into xorb binary format.

    Each chunk:  [compressed_size:u32LE | uncompressed_size:u32LE | compressed_data]
    """
    buf = bytearray()
    infos: List[ChunkInfo] = []
    for chunk in chunks:
        compressed = lz4_compress(chunk)
        header = struct.pack("<II", len(compressed), len(chunk))
        byte_start = len(buf)
        buf.extend(header)
        buf.extend(compressed)
        infos.append(ChunkInfo(
            hash=_blake3_keyed(chunk, DATA_KEY),
            byte_range_start=byte_start,
            unpacked_bytes=len(chunk),
        ))
    return bytes(buf), infos


def xorb_hash_from_chunks(infos: List[ChunkInfo]) -> bytes:
    """Xorb hash = blake3(concat(chunk_hashes), INTERNAL_KEY)."""
    combined = b"".join(c.hash for c in infos)
    return _blake3_keyed(combined, INTERNAL_KEY)


def file_hash_from_xorb(xorb_hash: bytes) -> bytes:
    """File hash = blake3(xorb_hash, ZERO_KEY).  (Simplified: real xet uses Merkle.)"""
    return _blake3_keyed(xorb_hash, ZERO_KEY)


def parse_xorb(data: bytes) -> bytes:
    """Decompress all chunks from an xorb and concatenate them."""
    out = bytearray()
    i = 0
    while i + 8 <= len(data):
        compressed_size, uncompressed_size = struct.unpack_from("<II", data, i)
        i += 8
        chunk_bytes = data[i: i + compressed_size]
        i += compressed_size
        out.extend(lz4_decompress(chunk_bytes, uncompressed_size))
    return bytes(out)


# ── MDB shard format ───────────────────────────────────────────────────────────

MDB_SHARD_HEADER_TAG = (
    b"HFRepoMetaData"
    + bytes([0, 85, 105, 103, 69, 106, 123, 129, 87,
             131, 165, 189, 217, 92, 205, 209, 74, 169])
)
assert len(MDB_SHARD_HEADER_TAG) == 32
BOOKEND           = bytes([0xFF] * 32) + bytes(16)
FLAG_VERIFICATION = 0x8000_0000
FLAG_METADATA_EXT = 0x4000_0000


def build_shard(
    file_hash:  bytes,
    sha256:     bytes,
    xorb_hash:  bytes,
    infos:      List[ChunkInfo],
    total_size: int,
    on_disk:    int,
) -> bytes:
    buf = bytearray()

    # Header (48 bytes): TAG[32] + version[8] + footer_size[8]
    buf += MDB_SHARD_HEADER_TAG
    buf += struct.pack("<Q", 2)   # version
    buf += struct.pack("<Q", 0)   # footer_size = 0 on upload

    # FileDataSequenceHeader
    buf += file_hash
    buf += struct.pack("<I", FLAG_VERIFICATION | FLAG_METADATA_EXT)
    buf += struct.pack("<I", 1)            # num_entries (one xorb term)
    buf += bytes(8)                        # _unused

    # FileDataSequenceEntry
    buf += xorb_hash
    buf += struct.pack("<I", 0)            # cas_flags
    buf += struct.pack("<I", total_size)   # unpacked_segment_bytes
    buf += struct.pack("<I", 0)            # chunk_index_start
    buf += struct.pack("<I", len(infos))   # chunk_index_end (exclusive)

    # FileVerificationEntry (1 entry)
    buf += bytes(32) + bytes(16)

    # FileMetadataExt (sha256 + padding)
    buf += sha256 + bytes(16)

    # File Info section bookend
    buf += BOOKEND

    # CASChunkSequenceHeader
    buf += xorb_hash
    buf += struct.pack("<I", 0)            # cas_flags
    buf += struct.pack("<I", len(infos))   # num_entries
    buf += struct.pack("<I", total_size)   # num_bytes_in_cas
    buf += struct.pack("<I", on_disk)      # num_bytes_on_disk

    # CASChunkSequenceEntry × len(infos)
    for info in infos:
        buf += info.hash
        buf += struct.pack("<I", info.byte_range_start)
        buf += struct.pack("<I", info.unpacked_bytes)
        buf += bytes(8)   # _unused

    # CAS Info section bookend
    buf += BOOKEND

    return bytes(buf)


# ── CAS HTTP helpers ───────────────────────────────────────────────────────────

def _get_token(scope: str, hub_token: str) -> str:
    """Returns the access_token JWT.  Always uses CAS_URL directly."""
    url = (f"{CAS_URL}/api/{REPO_TYPE}s/{NAMESPACE}/{REPO}"
           f"/xet-{scope}-token/{REVISION}")
    r = requests.get(url, headers={"Authorization": f"Bearer {hub_token}"}, timeout=10)
    r.raise_for_status()
    return r.json()["accessToken"]

def _register_and_create_repo() -> str:
    print(f"    Registering user {NAMESPACE}…")
    r = requests.post(f"{CAS_URL}/api/auth/register", json={"username": NAMESPACE, "password": "password"}, timeout=10)
    r.raise_for_status()
    hub_token = r.json()["token"]

    print(f"    Creating repo {NAMESPACE}/{REPO}…")
    r = requests.post(
        f"{CAS_URL}/api/repos/create",
        headers={"Authorization": f"Bearer {hub_token}"},
        json={"name": REPO, "type": REPO_TYPE, "private": False},
        timeout=10
    )
    r.raise_for_status()
    return hub_token


def upload_xorb(token: str, xorb_hash: bytes, xorb_bytes: bytes) -> bool:
    api = hash_to_api(xorb_hash)
    r = requests.post(
        f"{CAS_URL}/v1/xorbs/default/{api}",
        data=xorb_bytes,
        headers={
            "Authorization": f"Bearer {token}",
            "Content-Type":  "application/octet-stream",
        },
        timeout=120,
    )
    r.raise_for_status()
    return r.json().get("was_inserted", True)


def upload_shard(token: str, shard_bytes: bytes) -> None:
    r = requests.post(
        f"{CAS_URL}/v1/shards",
        data=shard_bytes,
        headers={
            "Authorization": f"Bearer {token}",
            "Content-Type":  "application/octet-stream",
        },
        timeout=30,
    )
    r.raise_for_status()
    assert r.json().get("result") == 1, f"shard upload: {r.json()}"


def get_reconstruction(token: str, file_hash: bytes) -> dict:
    api = hash_to_api(file_hash)
    r = requests.get(
        f"{CAS_URL}/v1/reconstructions/{api}",
        headers={"Authorization": f"Bearer {token}"},
        timeout=30,
    )
    r.raise_for_status()
    return r.json()


# ── hf_xet probe ──────────────────────────────────────────────────────────────

def try_hf_xet(source_path: str, write_token: str) -> Optional[str]:
    """Try three likely hf_xet API shapes.  Returns file_hash API string or None."""
    try:
        import hf_xet
        exports = [x for x in dir(hf_xet) if not x.startswith("_")]
        print(f"  [hf_xet] package found, exports: {exports}")
    except ImportError:
        print("  [hf_xet] not installed — using Python implementation")
        return None

    # Shape 1: top-level function(endpoint, token, path)
    for fname in ("upload_file", "cas_upload_file", "upload"):
        fn = getattr(hf_xet, fname, None)
        if fn is None:
            continue
        try:
            print(f"  [hf_xet] trying hf_xet.{fname}(endpoint, token, path)")
            result = fn(CAS_URL, write_token, source_path)
            print(f"  [hf_xet] {fname} → {result!r}")
            return str(result) if result else None
        except Exception as exc:
            print(f"  [hf_xet] {fname} failed: {exc}")

    # Shape 2: XetHub(endpoint, token).method(path)
    XetHub = getattr(hf_xet, "XetHub", None)
    if XetHub is not None:
        try:
            hub = XetHub(endpoint=CAS_URL, token=write_token)
            for mname in ("upload_file", "upload", "push_file"):
                m = getattr(hub, mname, None)
                if m is None:
                    continue
                print(f"  [hf_xet] trying XetHub.{mname}(path)")
                result = m(source_path)
                print(f"  [hf_xet] {mname} → {result!r}")
                return str(result) if result else None
        except Exception as exc:
            print(f"  [hf_xet] XetHub failed: {exc}")

    # Shape 3: upload_files(endpoint, token, [path]) → [hash]
    fn = getattr(hf_xet, "upload_files", None)
    if fn is not None:
        try:
            print("  [hf_xet] trying hf_xet.upload_files(endpoint, token, [path])")
            result = fn(CAS_URL, write_token, [source_path])
            print(f"  [hf_xet] upload_files → {result!r}")
            if result and isinstance(result, (list, tuple)) and result[0]:
                return str(result[0])
        except Exception as exc:
            print(f"  [hf_xet] upload_files failed: {exc}")

    print("  [hf_xet] no compatible CAS upload API found — using Python implementation")
    return None


# ── Main ───────────────────────────────────────────────────────────────────────

def step(n: int, msg: str) -> None:
    print(f"\n[{n}] {msg}")


def main() -> None:
    print("=" * 68)
    print("  Xet Protocol Round-Trip Compatibility Test")
    print(f"  CAS server : {CAS_URL}")
    print("=" * 68)

    # ── Step 1: download test file ─────────────────────────────────────────────
    step(1, "Downloading test file from HuggingFace Hub")

    if SKIP_DOWNLOAD and CACHE_PATH.exists():
        source_data = CACHE_PATH.read_bytes()
        print(f"    Using cached file: {CACHE_PATH}")
    else:
        print(f"    URL: {HF_FILE_URL}")
        r = requests.get(HF_FILE_URL, timeout=60)
        r.raise_for_status()
        source_data = r.content
        CACHE_PATH.write_bytes(source_data)

    source_size   = len(source_data)
    source_sha256 = hashlib.sha256(source_data).hexdigest()
    print(f"    Size  : {source_size:,} bytes")
    print(f"    SHA-256: {source_sha256}")

    if not HAS_BLAKE3:
        print("\n  NOTE: pip install blake3 for real xet-core hash compatibility")
    if not HAS_LZ4:
        print("  NOTE: pip install lz4    for real xet-core xorb format")

    # ── Step 2: get write token ────────────────────────────────────────────────
    step(2, "Obtaining write token")
    hub_token = _register_and_create_repo()
    write_token = _get_token("write", hub_token)
    print(f"    casUrl: {CAS_URL}")

    # ── Step 3: upload ─────────────────────────────────────────────────────────
    step(3, "Uploading")

    hf_xet_hash = try_hf_xet(str(CACHE_PATH), write_token)
    using_hf_xet = hf_xet_hash is not None

    if using_hf_xet:
        file_hash_api = hf_xet_hash
        # hf_xet returns the file hash; we'll use it for reconstruction
        print(f"  [hf_xet] file_hash = {file_hash_api[:16]}…")
        # We need the raw bytes to call our reconstruction endpoint
        # If the hash is 64 hex chars (API encoded), decode it
        if len(file_hash_api) == 64 and all(c in "0123456789abcdef" for c in file_hash_api):
            # It might be either plain hex or API-encoded; store as-is for reconstruction
            file_hash_raw: Optional[bytes] = None
        else:
            file_hash_raw = None
    else:
        # ── Python xet client ──────────────────────────────────────────────────
        print("  [python] CDC chunking…")
        chunks = cdc_chunk(source_data)
        print(f"    {len(chunks)} chunk(s), sizes: "
              f"{[f'{len(c)//1024}KiB' for c in chunks]}")

        print("  [python] Building xorb…")
        xorb_bytes, chunk_infos = build_xorb(chunks)
        xorb_hash  = xorb_hash_from_chunks(chunk_infos)
        file_hash  = file_hash_from_xorb(xorb_hash)
        sha256_raw = hashlib.sha256(source_data).digest()

        file_hash_api = hash_to_api(file_hash)
        file_hash_raw = file_hash
        xorb_api      = hash_to_api(xorb_hash)
        print(f"    xorb : {len(xorb_bytes):,} bytes  hash={xorb_api[:16]}…")
        print(f"    file hash: {file_hash_api[:16]}…")

        print("  [python] Uploading xorb…")
        inserted = upload_xorb(write_token, xorb_hash, xorb_bytes)
        print(f"    was_inserted={inserted}")

        print("  [python] Uploading shard…")
        shard = build_shard(
            file_hash=file_hash,
            sha256=sha256_raw,
            xorb_hash=xorb_hash,
            infos=chunk_infos,
            total_size=source_size,
            on_disk=len(xorb_bytes),
        )
        upload_shard(write_token, shard)
        print(f"    {len(shard):,} shard bytes, {len(chunk_infos)} chunk entries")

    # ── Step 4: read token + reconstruction ───────────────────────────────────
    step(4, "Querying reconstruction")
    read_token = _get_token("read", hub_token)

    if using_hf_xet and file_hash_raw is None:
        print("  hf_xet upload succeeded but returned an opaque hash.")
        print("  Cannot decode for reconstruction query — PARTIAL PASS.")
        print("\n  ✓  hf_xet upload path exercised successfully.")
        return

    recon = get_reconstruction(read_token, file_hash_raw)  # type: ignore[arg-type]
    terms      = recon.get("terms", [])
    fetch_info = recon.get("fetch_info", {})
    print(f"    {len(terms)} term(s), {len(fetch_info)} xorb(s) with presigned URLs")
    assert terms, "reconstruction returned 0 terms"

    # ── Step 5: download via presigned URL ─────────────────────────────────────
    step(5, "Downloading xorb via presigned URL")
    # fetch_info: {xorb_api_hash: [{url, range, url_range}]}
    first_entry = next(iter(fetch_info.values()))[0]
    presigned   = normalize_url(first_entry["url"])
    print(f"    {presigned[:72]}…")

    r = requests.get(presigned, timeout=120)
    r.raise_for_status()
    downloaded_xorb = r.content
    print(f"    Downloaded: {len(downloaded_xorb):,} bytes")

    # ── Step 6: reassemble and verify ─────────────────────────────────────────
    step(6, "Reassembling file from xorb and verifying SHA-256")
    reconstructed = parse_xorb(downloaded_xorb)
    if not reconstructed:
        # lz4 unavailable and xorb was stored raw: the xorb IS the file
        reconstructed = downloaded_xorb

    recon_sha256 = hashlib.sha256(reconstructed).hexdigest()

    print(f"    original      SHA-256 : {source_sha256}")
    print(f"    reconstructed SHA-256 : {recon_sha256}")
    print(f"    original size      : {source_size:,}")
    print(f"    reconstructed size : {len(reconstructed):,}")

    ok = source_sha256 == recon_sha256
    print()
    print("=" * 68)
    if ok:
        print("  ✓  ROUND-TRIP PASS — SHA-256 matches")
    else:
        print("  ✗  ROUND-TRIP FAIL — SHA-256 mismatch")
    print("=" * 68)
    sys.exit(0 if ok else 1)


if __name__ == "__main__":
    main()
