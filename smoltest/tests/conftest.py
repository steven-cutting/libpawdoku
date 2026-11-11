"""Shared fixtures: every test runs on the FakeEngine with a throwaway cache dir."""

from __future__ import annotations

from collections.abc import Iterator
from pathlib import Path

import pytest

from smoltest.config import Settings
from smoltest.testing import FakeEngine

pytest_plugins = ["pytester"]


@pytest.fixture(autouse=True)
def _fake_environment(monkeypatch: pytest.MonkeyPatch, tmp_path: Path) -> None:
    """Route engine selection to the fake and the cache to a temporary directory."""
    monkeypatch.setenv("SMOLTEST_ENGINE", "smoltest.testing:FakeEngine")
    monkeypatch.setenv("SMOLTEST_CACHE_DIR", str(tmp_path / "cache"))
    monkeypatch.delenv("SMOL_CLOUD_TOKEN", raising=False)
    monkeypatch.delenv("SMOLTEST_TARGET", raising=False)


@pytest.fixture
def fake_engine(monkeypatch: pytest.MonkeyPatch) -> Iterator[FakeEngine]:
    """A FakeEngine that ``get_engine()`` also returns; fails the test on leaked machines."""
    engine = FakeEngine()
    monkeypatch.setattr(FakeEngine, "_default", engine)
    monkeypatch.setenv("SMOLTEST_ENGINE", "smoltest.testing:FakeEngine.factory")
    yield engine
    leaked = list(engine.live_machines)
    engine.close_all()
    assert not leaked, f"machines leaked: {leaked}"


@pytest.fixture
def settings(tmp_path: Path) -> Settings:
    """Settings pinned to the local target and a temporary cache directory."""
    return Settings(
        target="local",
        cache_dir=tmp_path / "cache",
        engine_path="smoltest.testing:FakeEngine.factory",
    )
