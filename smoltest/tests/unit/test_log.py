"""``warn_once``: once per key, and safe across a fork."""

from __future__ import annotations

import multiprocessing
import threading
import warnings
from typing import Any

import pytest

from smoltest import _log
from smoltest._log import reset_warn_once, warn_once
from smoltest.errors import SmoltestWarning


def test_warn_once_warns_once_per_key() -> None:
    reset_warn_once()
    with pytest.warns(SmoltestWarning, match="first"):
        assert warn_once("log-test", "first") is True
    with warnings.catch_warnings():
        warnings.simplefilter("error")
        assert warn_once("log-test", "again") is False
    reset_warn_once()
    with pytest.warns(SmoltestWarning):
        assert warn_once("log-test", "after reset") is True


def _child_warns(results: Any) -> None:
    with warnings.catch_warnings(record=True):
        warnings.simplefilter("always")
        results.put(warn_once("fork-child-key", "from the child"))


@pytest.mark.skipif(
    "fork" not in multiprocessing.get_all_start_methods(), reason="needs the fork start method"
)
@pytest.mark.filterwarnings("ignore::DeprecationWarning")  # fork() with live threads, 3.12+
def test_forked_child_does_not_inherit_a_held_warning_lock() -> None:
    """A parent thread holding the lock at the fork must not hang the child's warn_once."""
    reset_warn_once()
    holding, release = threading.Event(), threading.Event()

    def hold() -> None:
        with _log._warned_lock:
            holding.set()
            release.wait(30)

    thread = threading.Thread(target=hold, daemon=True)
    thread.start()
    try:
        assert holding.wait(5)
        ctx = multiprocessing.get_context("fork")
        results = ctx.Queue()
        proc = ctx.Process(target=_child_warns, args=(results,), daemon=True)
        proc.start()
        try:
            assert results.get(timeout=30) is True, "the child's warn_once must not block"
        finally:
            proc.join(timeout=5)
            if proc.is_alive():
                proc.kill()
        assert proc.exitcode == 0
    finally:
        release.set()
        thread.join(5)
