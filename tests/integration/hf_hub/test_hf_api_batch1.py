from io import BytesIO
from pathlib import Path

import pytest
from huggingface_hub import CommitOperationAdd, CommitOperationDelete, HfApi, hf_hub_download
from huggingface_hub.errors import EntryNotFoundError

from helpers import make_repo_id, write_tree


def test_repo_exists_file_exists_and_revision_exists(hf_api, hf_session, repo_factory, tmp_path):
    repo_id = repo_factory("hf-api-exists")
    hf_api.upload_file(
        path_or_fileobj=b"content",
        path_in_repo="file.txt",
        repo_id=repo_id,
        commit_message="Add file.txt",
    )

    missing_repo_id = make_repo_id(hf_session["username"], "missing-repo")

    assert hf_api.repo_exists(repo_id)
    assert not hf_api.repo_exists(missing_repo_id)
    assert hf_api.file_exists(repo_id, "file.txt")
    assert not hf_api.file_exists(repo_id, "missing.txt")
    assert hf_api.revision_exists(repo_id, "main")

    downloaded_path = hf_hub_download(
        repo_id=repo_id,
        filename="file.txt",
        endpoint=hf_session["endpoint"],
        token=hf_session["token"],
        cache_dir=tmp_path / "cache",
    )
    assert downloaded_path


def test_whoami_with_passing_token(hf_session):
    api = HfApi(endpoint=hf_session["endpoint"])
    info = api.whoami(token=hf_session["token"])

    assert info["name"] == hf_session["username"]
    assert info["fullname"] == hf_session["username"]
    assert isinstance(info["orgs"], list)


def test_whoami_with_caching(hf_session):
    api = HfApi(endpoint=hf_session["endpoint"], token=hf_session["token"])
    assert api._whoami_cache == {}

    info = api.whoami(cache=True)
    assert info["name"] == hf_session["username"]
    assert hf_session["token"] in api._whoami_cache

    cached_value = {"name": "cached-value"}
    api._whoami_cache[hf_session["token"]] = cached_value
    assert api.whoami(cache=True) == cached_value

    api_bis = HfApi(endpoint=hf_session["endpoint"], token=hf_session["token"])
    assert api_bis._whoami_cache == {}


def test_delete_repo_missing_ok(hf_api, hf_session):
    missing_repo_id = make_repo_id(hf_session["username"], "missing-delete")
    hf_api.delete_repo(missing_repo_id, missing_ok=True)


def test_upload_file_from_path(hf_api, hf_session, repo_factory, tmp_path):
    repo_id = repo_factory("upload-from-path")
    source_path = tmp_path / "source.txt"
    source_path.write_text("Hello from path", encoding="utf-8")

    commit_info = hf_api.upload_file(
        path_or_fileobj=source_path,
        path_in_repo="temp/new_file.md",
        repo_id=repo_id,
        commit_message="Upload from path",
    )

    assert commit_info.oid
    downloaded_path = hf_hub_download(
        repo_id=repo_id,
        filename="temp/new_file.md",
        endpoint=hf_session["endpoint"],
        token=hf_session["token"],
        cache_dir=tmp_path / "cache",
    )
    assert downloaded_path
    assert (tmp_path / "cache").exists()
    assert Path(downloaded_path).read_text(encoding="utf-8") == "Hello from path"


def test_upload_file_from_file_object(hf_api, hf_session, repo_factory, tmp_path):
    repo_id = repo_factory("upload-from-fileobj")
    source_path = tmp_path / "source.bin"
    source_path.write_bytes(b"Hello from file object")

    with source_path.open("rb") as handle:
        commit_info = hf_api.upload_file(
            path_or_fileobj=handle,
            path_in_repo="temp/new_file.md",
            repo_id=repo_id,
            commit_message="Upload from file object",
        )

    assert commit_info.oid
    downloaded_path = hf_hub_download(
        repo_id=repo_id,
        filename="temp/new_file.md",
        endpoint=hf_session["endpoint"],
        token=hf_session["token"],
        cache_dir=tmp_path / "cache",
    )
    assert Path(downloaded_path).read_bytes() == b"Hello from file object"


def test_upload_file_from_bytesio(hf_api, hf_session, repo_factory, tmp_path):
    repo_id = repo_factory("upload-from-bytesio")

    commit_info = hf_api.upload_file(
        path_or_fileobj=BytesIO(b"Hello from BytesIO"),
        path_in_repo="temp/new_file.md",
        repo_id=repo_id,
        commit_message="Upload from BytesIO",
    )

    assert commit_info.oid
    downloaded_path = hf_hub_download(
        repo_id=repo_id,
        filename="temp/new_file.md",
        endpoint=hf_session["endpoint"],
        token=hf_session["token"],
        cache_dir=tmp_path / "cache",
    )
    assert Path(downloaded_path).read_bytes() == b"Hello from BytesIO"


def test_upload_folder_basic(hf_api, hf_session, repo_factory, tmp_path):
    repo_id = repo_factory("upload-folder")
    upload_dir = tmp_path / "upload"
    write_tree(
        upload_dir,
        {
            "config.json": '{"model_type": "dummy"}',
            "nested/weights.bin": b"weights",
        },
    )

    commit_info = hf_api.upload_folder(
        folder_path=upload_dir,
        repo_id=repo_id,
        repo_type="model",
        commit_message="Upload folder",
    )

    assert commit_info.oid
    config_path = hf_hub_download(
        repo_id=repo_id,
        filename="config.json",
        endpoint=hf_session["endpoint"],
        token=hf_session["token"],
        cache_dir=tmp_path / "cache-config",
    )
    nested_path = hf_hub_download(
        repo_id=repo_id,
        filename="nested/weights.bin",
        endpoint=hf_session["endpoint"],
        token=hf_session["token"],
        cache_dir=tmp_path / "cache-nested",
    )
    assert Path(config_path).read_text(encoding="utf-8") == '{"model_type": "dummy"}'
    assert Path(nested_path).read_bytes() == b"weights"


def test_delete_file(hf_api, hf_session, repo_factory, tmp_path):
    repo_id = repo_factory("delete-file")
    hf_api.upload_file(
        path_or_fileobj=b"temporary content",
        path_in_repo="temp/new_file.md",
        repo_id=repo_id,
    )

    commit_info = hf_api.delete_file(
        path_in_repo="temp/new_file.md",
        repo_id=repo_id,
        commit_message="Delete file",
    )

    assert commit_info.oid
    assert not hf_api.file_exists(repo_id, "temp/new_file.md")
    with pytest.raises(EntryNotFoundError):
        hf_hub_download(
            repo_id=repo_id,
            filename="temp/new_file.md",
            endpoint=hf_session["endpoint"],
            token=hf_session["token"],
            cache_dir=tmp_path / "cache",
        )


def test_create_commit_add_and_delete(hf_api, repo_factory):
    repo_id = repo_factory("create-commit")

    first_commit = hf_api.create_commit(
        repo_id=repo_id,
        commit_message="Initial files",
        operations=[
            CommitOperationAdd(path_in_repo="file_to_keep.txt", path_or_fileobj=b"Keep me"),
            CommitOperationAdd(path_in_repo="file_to_delete.txt", path_or_fileobj=b"Delete me later"),
        ],
    )
    assert first_commit.oid

    second_commit = hf_api.create_commit(
        repo_id=repo_id,
        commit_message="Add and delete",
        operations=[
            CommitOperationAdd(path_in_repo="new_file.txt", path_or_fileobj=b"I am new"),
            CommitOperationDelete(path_in_repo="file_to_delete.txt"),
        ],
    )

    assert second_commit.oid
    files = set(hf_api.list_repo_files(repo_id))
    assert "file_to_keep.txt" in files
    assert "new_file.txt" in files
    assert "file_to_delete.txt" not in files
