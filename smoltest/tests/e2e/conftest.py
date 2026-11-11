"""End-to-end fixtures: real machines on the real Smol engine.

The unit suite's root ``conftest.py`` routes every test to the fake engine; the
autouse fixture here undoes that for the tests in this directory (real
``SmolEngine``, an isolated session-wide cache directory, the target the
marker asks for, and the cloud token the root fixture cleared). Tests marked
``local_e2e`` are skipped unless the local engine can boot here; ``cloud_e2e``
tests are skipped without ``SMOL_CLOUD_TOKEN``. Both markers are deselected by
default (``addopts``); run them with ``-m local_e2e`` or ``-m cloud_e2e``.
"""

from __future__ import annotations

import contextlib
import functools
import os
from collections.abc import Iterator
from pathlib import Path
from typing import Any

import pytest

from smoltest.config import Settings
from smoltest.postgres import PostgresMachine
from smoltest.transport import get_engine
from smoltest.transport.base import Engine

LOCAL_MARK = "local_e2e"
CLOUD_MARK = "cloud_e2e"
TOKEN_ENV = "SMOL_CLOUD_TOKEN"
E2E_READY_TIMEOUT_S = 600.0
"""Readiness timeout for e2e boots: the first run of a session may pull the image."""

ORIGINAL_ENV: dict[str, str] = dict(os.environ)
"""The process environment before any fixture monkeypatched it (read at import)."""


@functools.lru_cache(maxsize=1)
def local_availability() -> tuple[bool, str | None, str | None]:
    """``(ok, code, reason)`` of the real local engine, computed once per process."""
    from smoltest.transport.smol_engine import SmolEngine

    return SmolEngine().local_availability()


def cloud_token_present() -> bool:
    """``True`` when the session started with ``SMOL_CLOUD_TOKEN`` set."""
    return bool(ORIGINAL_ENV.get(TOKEN_ENV))


def pytest_runtest_setup(item: pytest.Item) -> None:
    """Skip e2e tests whose target cannot boot here."""
    if item.get_closest_marker(LOCAL_MARK) is not None:
        ok, code, reason = local_availability()
        if not ok:
            pytest.skip(f"local Smol engine unavailable ({code}: {reason})")
    if item.get_closest_marker(CLOUD_MARK) is not None and not cloud_token_present():
        pytest.skip(f"{TOKEN_ENV} is not set")


@pytest.fixture(scope="session")
def e2e_cache_dir(tmp_path_factory: pytest.TempPathFactory) -> Path:
    """One checkpoint cache for the whole e2e session, so later boots can restore."""
    return tmp_path_factory.mktemp("smoltest-e2e-cache")


@pytest.fixture(autouse=True)
def _real_engine_environment(
    request: pytest.FixtureRequest,
    monkeypatch: pytest.MonkeyPatch,
    e2e_cache_dir: Path,
    _fake_environment: None,
) -> None:
    """Point the process at the real engine, the e2e cache and the marker's target.

    Depends on the root ``_fake_environment`` fixture so it runs after it and
    overrides what that fixture set. Subprocesses (the xdist test) inherit the
    result through ``os.environ``.
    """
    monkeypatch.delenv("SMOLTEST_ENGINE", raising=False)
    monkeypatch.setenv("SMOLTEST_CACHE_DIR", str(e2e_cache_dir))
    if "SMOLTEST_READY_TIMEOUT" not in ORIGINAL_ENV:
        monkeypatch.setenv("SMOLTEST_READY_TIMEOUT", f"{E2E_READY_TIMEOUT_S:g}")
    if request.node.get_closest_marker(CLOUD_MARK) is not None:
        monkeypatch.setenv("SMOLTEST_TARGET", "cloud")
        token = ORIGINAL_ENV.get(TOKEN_ENV)
        if token:
            monkeypatch.setenv(TOKEN_ENV, token)
    elif request.node.get_closest_marker(LOCAL_MARK) is not None:
        monkeypatch.setenv("SMOLTEST_TARGET", "local")


@pytest.fixture
def e2e_settings() -> Settings:
    """Settings resolved from the (patched) environment: real cache dir, marker target."""
    return Settings.from_env()


@pytest.fixture
def engine(e2e_settings: Settings) -> Engine:
    """The real engine (``SmolEngine`` unless the user set ``SMOLTEST_ENGINE``)."""
    return get_engine(e2e_settings)


@pytest.fixture
def cleanup() -> Iterator[contextlib.ExitStack]:
    """Register ``machine.stop`` / ``golden.close`` here; runs LIFO at teardown."""
    with contextlib.ExitStack() as stack:
        yield stack


class Pg:
    """Short-lived psycopg connections: every call opens, runs and closes.

    Nothing stays connected between calls, so a checkpoint or branch taken
    afterwards never captures a host client.
    """

    def __init__(self, psycopg: Any, *, connect_timeout_s: int = 15) -> None:
        self._psycopg = psycopg
        self._connect_timeout_s = connect_timeout_s

    def connect(self, url: str) -> Any:
        """An autocommit connection (a context manager that closes on exit)."""
        return self._psycopg.connect(url, connect_timeout=self._connect_timeout_s, autocommit=True)

    def fetch(self, url: str, sql: str) -> list[tuple[Any, ...]]:
        """All rows of ``sql``."""
        with self.connect(url) as conn:
            cursor = conn.execute(sql)
            rows: list[tuple[Any, ...]] = [tuple(row) for row in cursor.fetchall()]
            return rows

    def scalar(self, url: str, sql: str) -> Any:
        """The first column of the first row of ``sql``."""
        rows = self.fetch(url, sql)
        assert rows, f"{sql!r} returned no rows"
        return rows[0][0]

    def execute(self, url: str, sql: str) -> None:
        """Run ``sql`` for its effect."""
        with self.connect(url) as conn:
            conn.execute(sql)


@pytest.fixture
def pg() -> Pg:
    """A :class:`Pg` helper; skips the test when psycopg (v3) is not installed."""
    psycopg = pytest.importorskip("psycopg")
    return Pg(psycopg)


def plain_url(machine: PostgresMachine) -> str:
    """``postgresql://`` URL (no SQLAlchemy driver suffix) of a running machine."""
    return machine.get_connection_url(driver=None)
