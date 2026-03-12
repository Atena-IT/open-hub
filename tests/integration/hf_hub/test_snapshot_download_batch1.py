from pathlib import Path

from huggingface_hub import snapshot_download

from helpers import write_tree


def _create_snapshot_repo(hf_api, repo_factory, tmp_path):
    repo_id = repo_factory("snapshot-download")
    upload_dir = tmp_path / "upload"
    write_tree(
        upload_dir,
        {
            "config.json": '{"model_type": "gpt2"}',
            "vocab.json": '{"a": 1, "b": 2}',
            "model.safetensors": b"fake weights data",
            "dummy.txt": "# Test Model\n",
            "data/train.csv": "col1,col2\n1,2\n",
        },
    )
    hf_api.upload_folder(
        folder_path=upload_dir,
        repo_id=repo_id,
        repo_type="model",
        commit_message="Upload snapshot fixture",
    )
    return repo_id


def test_snapshot_download_current_head_preserves_nested_files(hf_api, hf_session, repo_factory, tmp_path):
    repo_id = _create_snapshot_repo(hf_api, repo_factory, tmp_path)

    snapshot_path = Path(
        snapshot_download(
            repo_id=repo_id,
            repo_type="model",
            endpoint=hf_session["endpoint"],
            token=hf_session["token"],
            cache_dir=tmp_path / "cache",
        )
    )

    assert (snapshot_path / "config.json").read_text(encoding="utf-8") == '{"model_type": "gpt2"}'
    assert (snapshot_path / "model.safetensors").read_bytes() == b"fake weights data"
    assert (snapshot_path / "data" / "train.csv").read_text(encoding="utf-8") == "col1,col2\n1,2\n"


def test_snapshot_download_to_local_dir_is_correct(hf_api, hf_session, repo_factory, tmp_path):
    repo_id = _create_snapshot_repo(hf_api, repo_factory, tmp_path)
    local_dir = tmp_path / "local-snapshot"

    snapshot_path = snapshot_download(
        repo_id=repo_id,
        repo_type="model",
        endpoint=hf_session["endpoint"],
        token=hf_session["token"],
        cache_dir=tmp_path / "cache",
        local_dir=local_dir,
    )

    assert Path(snapshot_path) == local_dir
    assert (local_dir / "config.json").is_file()
    assert (local_dir / "data" / "train.csv").is_file()
    assert (local_dir / "dummy.txt").read_text(encoding="utf-8") == "# Test Model\n"


def test_snapshot_download_allow_patterns(hf_api, hf_session, repo_factory, tmp_path):
    repo_id = _create_snapshot_repo(hf_api, repo_factory, tmp_path)
    local_dir = tmp_path / "allow-local"

    snapshot_path = Path(
        snapshot_download(
            repo_id=repo_id,
            repo_type="model",
            endpoint=hf_session["endpoint"],
            token=hf_session["token"],
            local_dir=local_dir,
            allow_patterns="*.json",
        )
    )

    assert (snapshot_path / "config.json").is_file()
    assert (snapshot_path / "vocab.json").is_file()
    assert not (snapshot_path / "model.safetensors").exists()
    assert not (snapshot_path / "data" / "train.csv").exists()


def test_snapshot_download_ignore_patterns(hf_api, hf_session, repo_factory, tmp_path):
    repo_id = _create_snapshot_repo(hf_api, repo_factory, tmp_path)
    local_dir = tmp_path / "ignore-local"

    snapshot_path = Path(
        snapshot_download(
            repo_id=repo_id,
            repo_type="model",
            endpoint=hf_session["endpoint"],
            token=hf_session["token"],
            local_dir=local_dir,
            ignore_patterns=["*.safetensors", "data/*"],
        )
    )

    assert (snapshot_path / "config.json").is_file()
    assert (snapshot_path / "vocab.json").is_file()
    assert not (snapshot_path / "model.safetensors").exists()
    assert not (snapshot_path / "data" / "train.csv").exists()
