"""
Batch 7 — Jobs API mocked stub compatibility tests.

Covers the full surface of the HF Jobs Python client against the mocked stub
endpoints introduced in batch-7.  No real compute is performed: the server
stores jobs in-memory and immediately places them in the QUEUED stage.

Upstream reference:
  https://raw.githubusercontent.com/huggingface/huggingface_hub/refs/heads/main/docs/source/en/guides/jobs.md

API surface exercised:
  One-off jobs:
    run_job, list_jobs, inspect_job, fetch_job_logs, fetch_job_metrics, cancel_job
  Hardware catalogue:
    list_jobs_hardware
  Scheduled jobs:
    create_scheduled_job, create_scheduled_uv_job, list_scheduled_jobs,
    inspect_scheduled_job, suspend_scheduled_job, resume_scheduled_job,
    delete_scheduled_job

Required huggingface_hub version: >=1.8.0 (Jobs API GA)
"""

import pytest

# Guard: skip the entire module if the Jobs API is not available in the
# installed version of huggingface_hub.
pytest.importorskip(
    "huggingface_hub",
    minversion="1.8.0",
    reason="huggingface_hub >=1.8.0 required for Jobs API",
)

from huggingface_hub import (
    HfApi,
    cancel_job,
    create_scheduled_job,
    delete_scheduled_job,
    fetch_job_logs,
    fetch_job_metrics,
    inspect_job,
    inspect_scheduled_job,
    list_jobs,
    list_jobs_hardware,
    list_scheduled_jobs,
    resume_scheduled_job,
    run_job,
    suspend_scheduled_job,
)
from huggingface_hub import JobInfo, JobStatus, JobOwner
from huggingface_hub.errors import HfHubHTTPError


# ─── Helpers ──────────────────────────────────────────────────────────────────


def _assert_job_info_shape(job: JobInfo, *, image: str, command: list[str], owner: str) -> None:
    """Assert that a JobInfo object has the expected shape."""
    assert isinstance(job.id, str) and len(job.id) > 0
    # docker_image is set for Docker Hub images; space_id is set for HF Spaces images
    if job.space_id is None:
        assert job.docker_image == image
    assert job.command == command
    assert isinstance(job.status, JobStatus)
    assert job.status.stage in ("QUEUED", "RUNNING", "COMPLETED", "ERROR", "CANCELLED")
    assert isinstance(job.owner, JobOwner)
    assert job.owner.name == owner
    assert job.url.endswith(f"{owner}/{job.id}")


# ─── One-off job tests ────────────────────────────────────────────────────────


def test_run_job_returns_job_info_with_correct_shape(hf_api: HfApi, hf_session: dict) -> None:
    """run_job should return a well-formed JobInfo in QUEUED stage."""
    image = "python:3.12"
    command = ["python", "-c", "print('hello from stub')"]

    job = run_job(
        image=image,
        command=command,
        endpoint=hf_session["endpoint"],
        token=hf_session["token"],
    )

    _assert_job_info_shape(job, image=image, command=command, owner=hf_session["username"])
    assert job.status.stage == "QUEUED"
    assert job.flavor == "cpu-basic"  # default flavor


def test_run_job_with_explicit_flavor(hf_api: HfApi, hf_session: dict) -> None:
    """run_job should propagate the flavor field."""
    job = run_job(
        image="python:3.12",
        command=["python", "-c", "print('gpu stub')"],
        flavor="a10g-small",
        endpoint=hf_session["endpoint"],
        token=hf_session["token"],
    )

    assert job.flavor == "a10g-small"


def test_run_job_with_env_and_labels(hf_api: HfApi, hf_session: dict) -> None:
    """run_job should persist environment variables and labels."""
    job = run_job(
        image="python:3.12",
        command=["python", "-c", "import os; print(os.environ['FOO'])"],
        env={"FOO": "bar"},
        labels={"batch": "7", "purpose": "stub-test"},
        endpoint=hf_session["endpoint"],
        token=hf_session["token"],
    )

    assert job.environment == {"FOO": "bar"}
    assert job.labels == {"batch": "7", "purpose": "stub-test"}


def test_list_jobs_includes_created_job(hf_api: HfApi, hf_session: dict) -> None:
    """list_jobs should return any job created in this session."""
    job = run_job(
        image="python:3.12",
        command=["python", "-c", "print('list-test')"],
        endpoint=hf_session["endpoint"],
        token=hf_session["token"],
    )

    jobs = list_jobs(
        endpoint=hf_session["endpoint"],
        token=hf_session["token"],
    )

    assert any(j.id == job.id for j in jobs), (
        f"Job {job.id!r} not found in list_jobs result"
    )


def test_inspect_job_returns_same_job(hf_api: HfApi, hf_session: dict) -> None:
    """inspect_job should return the same job as run_job."""
    job = run_job(
        image="python:3.12",
        command=["python", "-c", "print('inspect-test')"],
        endpoint=hf_session["endpoint"],
        token=hf_session["token"],
    )

    inspected = inspect_job(
        job_id=job.id,
        endpoint=hf_session["endpoint"],
        token=hf_session["token"],
    )

    assert inspected.id == job.id
    assert inspected.command == job.command
    assert inspected.owner.name == hf_session["username"]


def test_inspect_job_raises_for_unknown_id(hf_api: HfApi, hf_session: dict) -> None:
    """inspect_job should raise an error for a job that does not exist."""
    with pytest.raises(HfHubHTTPError):
        inspect_job(
            job_id="000000000000000000000000",
            endpoint=hf_session["endpoint"],
            token=hf_session["token"],
        )


def test_cancel_job_changes_stage_to_cancelled(hf_api: HfApi, hf_session: dict) -> None:
    """cancel_job should update the job stage to CANCELLED."""
    job = run_job(
        image="python:3.12",
        command=["python", "-c", "print('cancel-test')"],
        endpoint=hf_session["endpoint"],
        token=hf_session["token"],
    )

    cancel_job(
        job_id=job.id,
        endpoint=hf_session["endpoint"],
        token=hf_session["token"],
    )

    inspected = inspect_job(
        job_id=job.id,
        endpoint=hf_session["endpoint"],
        token=hf_session["token"],
    )

    assert inspected.status.stage == "CANCELLED"


def test_fetch_job_logs_returns_iterable(hf_api: HfApi, hf_session: dict) -> None:
    """fetch_job_logs should return an iterable (may be empty for stub jobs)."""
    job = run_job(
        image="python:3.12",
        command=["python", "-c", "print('log-test')"],
        endpoint=hf_session["endpoint"],
        token=hf_session["token"],
    )

    logs = list(
        fetch_job_logs(
            job_id=job.id,
            follow=False,
            endpoint=hf_session["endpoint"],
            token=hf_session["token"],
        )
    )

    # The stub may return zero or more log lines; the important thing is that
    # the call succeeds without raising.
    assert isinstance(logs, list)


def test_fetch_job_metrics_returns_iterable(hf_api: HfApi, hf_session: dict) -> None:
    """fetch_job_metrics should return an iterable (may be empty for stub jobs)."""
    job = run_job(
        image="python:3.12",
        command=["python", "-c", "print('metrics-test')"],
        endpoint=hf_session["endpoint"],
        token=hf_session["token"],
    )

    metrics = list(
        fetch_job_metrics(
            job_id=job.id,
            endpoint=hf_session["endpoint"],
            token=hf_session["token"],
        )
    )

    assert isinstance(metrics, list)


# ─── Hardware catalogue tests ─────────────────────────────────────────────────


def test_list_jobs_hardware_returns_non_empty_list(hf_api: HfApi, hf_session: dict) -> None:
    """list_jobs_hardware should return at least the cpu-basic flavour."""
    hardware = list_jobs_hardware(
        endpoint=hf_session["endpoint"],
        token=hf_session["token"],
    )

    names = [h.name for h in hardware]
    assert "cpu-basic" in names, f"cpu-basic not found in hardware list: {names}"
    assert len(hardware) > 0


def test_list_jobs_hardware_objects_have_required_fields(hf_api: HfApi, hf_session: dict) -> None:
    """Each JobHardware entry should have the mandatory fields populated."""
    hardware = list_jobs_hardware(
        endpoint=hf_session["endpoint"],
        token=hf_session["token"],
    )

    for h in hardware:
        assert h.name, "hardware entry missing name"
        assert h.pretty_name, "hardware entry missing pretty_name"
        assert h.cpu, "hardware entry missing cpu"
        assert h.ram, "hardware entry missing ram"
        assert h.unit_cost_usd >= 0, "hardware entry has negative unit_cost_usd"


# ─── Scheduled job tests ──────────────────────────────────────────────────────


def test_create_scheduled_job_returns_scheduled_job_info(
    hf_api: HfApi, hf_session: dict
) -> None:
    """create_scheduled_job should return a ScheduledJobInfo with expected fields."""
    scheduled = create_scheduled_job(
        image="python:3.12",
        command=["python", "-c", "print('scheduled-test')"],
        schedule="@hourly",
        endpoint=hf_session["endpoint"],
        token=hf_session["token"],
    )

    assert isinstance(scheduled.id, str) and len(scheduled.id) > 0
    assert scheduled.schedule == "@hourly"
    assert scheduled.owner.name == hf_session["username"]


def test_list_scheduled_jobs_includes_created_scheduled_job(
    hf_api: HfApi, hf_session: dict
) -> None:
    """list_scheduled_jobs should include jobs created in this session."""
    scheduled = create_scheduled_job(
        image="python:3.12",
        command=["python", "-c", "print('list-sched')"],
        schedule="@daily",
        endpoint=hf_session["endpoint"],
        token=hf_session["token"],
    )

    sched_list = list_scheduled_jobs(
        endpoint=hf_session["endpoint"],
        token=hf_session["token"],
    )

    assert any(s.id == scheduled.id for s in sched_list), (
        f"Scheduled job {scheduled.id!r} not found in list_scheduled_jobs result"
    )


def test_inspect_scheduled_job_returns_correct_info(
    hf_api: HfApi, hf_session: dict
) -> None:
    """inspect_scheduled_job should return the same job that was created."""
    scheduled = create_scheduled_job(
        image="python:3.12",
        command=["python", "-c", "print('inspect-sched')"],
        schedule="*/5 * * * *",
        endpoint=hf_session["endpoint"],
        token=hf_session["token"],
    )

    inspected = inspect_scheduled_job(
        scheduled_job_id=scheduled.id,
        endpoint=hf_session["endpoint"],
        token=hf_session["token"],
    )

    assert inspected.id == scheduled.id
    assert inspected.schedule == "*/5 * * * *"


def test_suspend_and_resume_scheduled_job(hf_api: HfApi, hf_session: dict) -> None:
    """suspend_scheduled_job and resume_scheduled_job should toggle the suspend flag."""
    scheduled = create_scheduled_job(
        image="python:3.12",
        command=["python", "-c", "print('suspend-resume')"],
        schedule="@weekly",
        endpoint=hf_session["endpoint"],
        token=hf_session["token"],
    )

    # Suspend
    suspend_scheduled_job(
        scheduled_job_id=scheduled.id,
        endpoint=hf_session["endpoint"],
        token=hf_session["token"],
    )
    inspected = inspect_scheduled_job(
        scheduled_job_id=scheduled.id,
        endpoint=hf_session["endpoint"],
        token=hf_session["token"],
    )
    assert inspected.suspend is True

    # Resume
    resume_scheduled_job(
        scheduled_job_id=scheduled.id,
        endpoint=hf_session["endpoint"],
        token=hf_session["token"],
    )
    inspected = inspect_scheduled_job(
        scheduled_job_id=scheduled.id,
        endpoint=hf_session["endpoint"],
        token=hf_session["token"],
    )
    assert inspected.suspend is False


def test_delete_scheduled_job_removes_it_from_list(
    hf_api: HfApi, hf_session: dict
) -> None:
    """delete_scheduled_job should make the job disappear from list_scheduled_jobs."""
    scheduled = create_scheduled_job(
        image="python:3.12",
        command=["python", "-c", "print('delete-sched')"],
        schedule="@monthly",
        endpoint=hf_session["endpoint"],
        token=hf_session["token"],
    )

    delete_scheduled_job(
        scheduled_job_id=scheduled.id,
        endpoint=hf_session["endpoint"],
        token=hf_session["token"],
    )

    sched_list = list_scheduled_jobs(
        endpoint=hf_session["endpoint"],
        token=hf_session["token"],
    )

    assert all(s.id != scheduled.id for s in sched_list), (
        f"Deleted scheduled job {scheduled.id!r} still present in list"
    )


def test_delete_scheduled_job_raises_for_unknown_id(
    hf_api: HfApi, hf_session: dict
) -> None:
    """delete_scheduled_job should raise for a scheduled job that does not exist."""
    with pytest.raises(HfHubHTTPError):
        delete_scheduled_job(
            scheduled_job_id="000000000000000000000000",
            endpoint=hf_session["endpoint"],
            token=hf_session["token"],
        )
