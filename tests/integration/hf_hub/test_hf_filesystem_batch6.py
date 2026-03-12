from huggingface_hub import HfFileSystem

from helpers import write_tree



def _create_filesystem_repo(hf_api, repo_factory, tmp_path):
    repo_id = repo_factory("hf-filesystem")
    upload_dir = tmp_path / "upload"
    upload_dir.mkdir()
    write_tree(
        upload_dir,
        {
            "README.md": "---\nlicense: mit\n---\n\nfilesystem fixture\n",
            "config.json": '{"batch": 6}',
            "nested/child.txt": "nested child\n",
            "nested/deeper/info.txt": "deep info\n",
        },
    )
    hf_api.upload_folder(
        folder_path=upload_dir,
        repo_id=repo_id,
        repo_type="model",
        commit_message="Upload filesystem fixture",
    )
    return repo_id



def test_hf_filesystem_lists_exists_and_reads_files(hf_api, hf_session, repo_factory, tmp_path):
    repo_id = _create_filesystem_repo(hf_api, repo_factory, tmp_path)
    hf_api.create_branch(repo_id, branch="release")

    fs = HfFileSystem(endpoint=hf_session["endpoint"], token=hf_session["token"])

    root_entries = fs.ls(repo_id, detail=False)

    assert f"{repo_id}/README.md" in root_entries
    assert f"{repo_id}/config.json" in root_entries
    assert f"{repo_id}/nested" in root_entries
    assert fs.exists(f"{repo_id}/README.md")
    assert fs.isfile(f"{repo_id}/README.md")
    assert fs.isdir(f"{repo_id}/nested")
    assert not fs.exists(f"{repo_id}/missing.txt")

    with fs.open(f"{repo_id}/config.json", "r") as handle:
        assert handle.read() == '{"batch": 6}'

    assert fs.read_text(f"{repo_id}/README.md") == "---\nlicense: mit\n---\n\nfilesystem fixture\n"
    assert fs.read_text(f"{repo_id}@release/nested/child.txt") == "nested child\n"

    nested_entries = fs.ls(f"{repo_id}/nested", detail=False, revision="release")

    assert f"{repo_id}/nested/child.txt" in nested_entries
    assert f"{repo_id}/nested/deeper" in nested_entries
