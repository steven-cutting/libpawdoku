"""The pytest plugin under ``pytest -n 2`` against real local machines (``-m local_e2e``).

A throwaway pytest project is written into ``tmp_path`` and run in a subprocess
(``uv run pytest`` when ``uv`` is on the path, else this interpreter). Each
xdist worker is its own process, so each boots its own golden and branches it
per test; the inner tests record which worker and golden served them.
"""

from __future__ import annotations

import json
import os
import shutil
import subprocess
import sys
from pathlib import Path

import pytest

pytestmark = pytest.mark.local_e2e

PROJECT_ROOT = Path(__file__).resolve().parents[2]
RECORD_ENV = "SMOLTEST_E2E_RECORDS"
SUBPROCESS_TIMEOUT_S = 1800.0

CONFTEST = """
import json
import os

import pytest


@pytest.fixture
def record(smoltest_postgres_golden):
    # The worker's golden is read from the plugin, not from machine.parent: a fresh
    # fallback machine (when the golden cannot be branched) has no parent but
    # still came from this golden.
    out = os.environ["SMOLTEST_E2E_RECORDS"]
    golden = smoltest_postgres_golden

    def _record(machine):
        info = machine.boot_info
        row = {
            "worker": os.environ.get("PYTEST_XDIST_WORKER", "main"),
            "via": info.via,
            "port": machine.get_exposed_port(),
            "golden": golden.machine.name,
            "golden_via": golden.boot_info.via,
            "golden_port": golden.machine.get_exposed_port(),
        }
        with open(out, "a") as fh:
            fh.write(json.dumps(row) + "\\n")

    return _record
"""

TEST_MODULE = """
import psycopg


def scalar(url, sql):
    with psycopg.connect(url, connect_timeout=15, autocommit=True) as conn:
        return conn.execute(sql).fetchone()[0]


def test_select_one(postgres, postgres_url, record):
    assert scalar(postgres_url, "SELECT 1") == 1
    record(postgres)


def test_private_table(postgres, postgres_url, record):
    # The same table name in every test: it only works if each test gets its own machine.
    with psycopg.connect(postgres_url, connect_timeout=15, autocommit=True) as conn:
        conn.execute("CREATE TABLE probe (id int)")
        conn.execute("INSERT INTO probe VALUES (1)")
        assert conn.execute("SELECT count(*) FROM probe").fetchone()[0] == 1
    record(postgres)
"""

PYTEST_INI = """
[pytest]
addopts = -p no:cacheprovider
smoltest_isolation = branch
"""


def pytest_command() -> list[str]:
    """``uv run --project <root> pytest`` when uv is available, else this interpreter."""
    uv = shutil.which("uv")
    if uv is not None:
        return [uv, "run", "--project", str(PROJECT_ROOT), "pytest"]
    return [sys.executable, "-m", "pytest"]


def write_project(root: Path) -> None:
    """A two-file mini project so ``--dist loadfile`` gives each worker one file."""
    root.mkdir(parents=True, exist_ok=True)
    (root / "pytest.ini").write_text(PYTEST_INI.lstrip())
    (root / "conftest.py").write_text(CONFTEST.lstrip())
    (root / "test_alpha.py").write_text(TEST_MODULE.lstrip())
    (root / "test_beta.py").write_text(TEST_MODULE.lstrip())


def test_two_workers_each_boot_a_golden_and_branch_per_test(tmp_path: Path) -> None:
    pytest.importorskip("psycopg")
    pytest.importorskip("xdist")
    project = tmp_path / "project"
    write_project(project)
    records = tmp_path / "records.jsonl"
    # os.environ already carries the e2e fixture's choices (real engine, cache dir, target).
    env = {**os.environ, RECORD_ENV: str(records)}
    env.pop("PYTEST_ADDOPTS", None)
    cmd = [*pytest_command(), "-q", "-n", "2", "--dist", "loadfile", str(project)]

    completed = subprocess.run(
        cmd,
        cwd=project,
        env=env,
        capture_output=True,
        text=True,
        timeout=SUBPROCESS_TIMEOUT_S,
        check=False,
    )
    output = completed.stdout + completed.stderr
    assert completed.returncode == 0, output
    assert "4 passed" in completed.stdout, output

    rows = [json.loads(line) for line in records.read_text().splitlines()]
    assert len(rows) == 4, rows
    assert {row["worker"] for row in rows} == {"gw0", "gw1"}, rows
    goldens_by_worker: dict[str, set[str | None]] = {}
    for row in rows:
        goldens_by_worker.setdefault(row["worker"], set()).add(row["golden"])
    # Every worker booted exactly one golden of its own, and every test used it.
    assert all(len(goldens) == 1 for goldens in goldens_by_worker.values()), goldens_by_worker
    goldens = set().union(*goldens_by_worker.values())
    assert None not in goldens and len(goldens) == 2, goldens_by_worker
    assert all(row["golden_via"] in ("cold", "restore") for row in rows), rows
    assert all(row["via"] in ("branch", "restore", "cold") for row in rows), rows
    # Branching guarantees a child never shares its golden's host port while both are live;
    # ports of machines that have already stopped may legitimately be reused.
    assert all(row["golden_port"] is not None for row in rows), rows
    assert all(row["port"] != row["golden_port"] for row in rows), rows
