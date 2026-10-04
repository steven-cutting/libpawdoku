"""Reaper ordering and idempotence, at exit and across a fork."""

from __future__ import annotations

import gc
import multiprocessing
import subprocess
import sys
from pathlib import Path
from typing import Any

import pytest

from smoltest._lifecycle import Reaper, get_reaper


class Thing:
    def __init__(self, name: str) -> None:
        self.name = name


def test_run_is_lifo_children_before_parents_and_idempotent() -> None:
    reaper = Reaper(install_atexit=False)
    order: list[str] = []
    a, b, c, d = (Thing(n) for n in "abcd")
    ta = reaper.register(a, lambda: order.append("a"))
    reaper.register(b, lambda: order.append("b"), parent=ta)
    reaper.register(c, lambda: order.append("c"))
    reaper.register(d, lambda: order.append("d"), parent=c)  # parent given as the object
    assert reaper.pending == 4
    reaper.run()
    assert order == ["d", "c", "b", "a"]
    assert reaper.pending == 0
    reaper.run()
    assert order == ["d", "c", "b", "a"]
    del a, b, c, d


def test_child_registered_before_parent_still_runs_first() -> None:
    reaper = Reaper(install_atexit=False)
    order: list[str] = []
    parent, child = Thing("p"), Thing("c")
    tp = reaper.register(parent, lambda: order.append("p"))
    reaper.register(child, lambda: order.append("c"), parent=tp)
    reaper.run()
    assert order == ["c", "p"]


def test_finalizer_runs_on_gc_and_not_again_at_exit() -> None:
    reaper = Reaper(install_atexit=False)
    order: list[str] = []
    thing = Thing("x")
    token = reaper.register(thing, lambda: order.append("x"))
    assert token.atexit is False
    del thing
    gc.collect()
    assert order == ["x"]
    reaper.run()
    assert order == ["x"]


def test_token_can_be_called_early_or_detached() -> None:
    reaper = Reaper(install_atexit=False)
    order: list[str] = []
    early, cancelled = Thing("e"), Thing("k")
    t1 = reaper.register(early, lambda: order.append("e"))
    t2 = reaper.register(cancelled, lambda: order.append("k"))
    t1()
    t2.detach()
    reaper.run()
    assert order == ["e"]
    del early, cancelled


def test_failing_cleanup_does_not_stop_the_others() -> None:
    reaper = Reaper(install_atexit=False)
    order: list[str] = []
    bad, good = Thing("bad"), Thing("good")

    def explode() -> None:
        raise RuntimeError("boom")

    reaper.register(good, lambda: order.append("good"))
    reaper.register(bad, explode)
    reaper.run()
    assert order == ["good"]
    del bad, good


def test_unknown_parent_object_is_ignored() -> None:
    reaper = Reaper(install_atexit=False)
    thing = Thing("t")
    token = reaper.register(thing, lambda: None, parent=object())
    assert token.alive
    reaper.run()


def test_get_reaper_is_a_singleton() -> None:
    assert get_reaper() is get_reaper()


def test_fired_and_detached_registrations_are_forgotten() -> None:
    reaper = Reaper(install_atexit=False)
    parent = Thing("p")
    tp = reaper.register(parent, lambda: None)
    children = [Thing(f"c{i}") for i in range(50)]
    tokens = [reaper.register(child, lambda: None, parent=tp) for child in children]
    assert reaper.pending == 51 and len(reaper._entries[tp].children) == 50
    for token in tokens[:25]:
        token()  # fired early (what a GC'd or explicitly closed child does)
    for token in tokens[25:]:
        assert reaper.detach(token) is True  # cancelled (what stop() does)
    assert reaper.pending == 1
    assert list(reaper._entries) == [tp] and list(reaper._order) == [tp]
    assert reaper._entries[tp].children == []
    assert len(reaper._by_obj) == 1
    assert reaper.detach(tokens[0]) is False  # already fired: nothing left to cancel
    reaper.run()
    assert reaper.pending == 0 and reaper._entries == {} and reaper._by_obj == {}
    del parent, children


def test_exit_hook_runs_even_when_the_reaper_creates_the_first_finalizer(tmp_path: Path) -> None:
    """weakref's exit hook must be registered before ours or it disables our tokens first."""
    marker = tmp_path / "cleaned"
    code = (
        "import sys\n"
        "from pathlib import Path\n"
        "from smoltest._lifecycle import get_reaper\n"
        "class Thing: pass\n"
        "thing = Thing()\n"
        "marker = Path(sys.argv[1])\n"
        "get_reaper().register(thing, lambda: marker.write_text('cleaned'))\n"
    )
    subprocess.run([sys.executable, "-c", code, str(marker)], check=True, timeout=120)
    assert marker.read_text() == "cleaned", "the exit hook skipped the registered cleanup"


def _forked_child(parent_marker: str, own_marker: str, results: Any) -> None:
    """What a forked child sees: no inherited registrations, a reaper that still works."""
    reaper = get_reaper()
    inherited = reaper.pending
    gc.collect()  # on the old code this fires the parent's finalizers
    reaper.run()  # what a normal exit in the child would do

    class Own:
        pass

    own = Own()
    reaper.register(own, lambda: Path(own_marker).write_text("own"))
    del own
    gc.collect()
    own_done = Path(own_marker).read_text() if Path(own_marker).exists() else None
    results.put((inherited, Path(parent_marker).exists(), own_done))


@pytest.mark.skipif(
    "fork" not in multiprocessing.get_all_start_methods(), reason="needs the fork start method"
)
@pytest.mark.filterwarnings("ignore::DeprecationWarning")  # fork() with live threads, 3.12+
def test_forked_child_does_not_run_the_parents_cleanups(tmp_path: Path) -> None:
    parent_marker, own_marker = tmp_path / "parent", tmp_path / "child"
    reaper = get_reaper()
    thing = Thing("parent-owned")
    token = reaper.register(thing, lambda: parent_marker.write_text("released by the child"))
    try:
        ctx = multiprocessing.get_context("fork")
        results = ctx.Queue()
        proc = ctx.Process(
            target=_forked_child, args=(str(parent_marker), str(own_marker), results), daemon=True
        )
        proc.start()
        inherited, parent_marker_exists, own_done = results.get(timeout=60)
        proc.join(timeout=30)
        assert proc.exitcode == 0
        assert inherited == 0, "the child must start with no inherited registrations"
        assert parent_marker_exists is False, "the child ran a parent-owned cleanup"
        assert own_done == "own", "child-owned registrations must still work"
        assert token.alive and not parent_marker.exists(), "the parent's registration is intact"
    finally:
        reaper.detach(token)
    del thing


def test_gc_removes_the_entry() -> None:
    reaper = Reaper(install_atexit=False)
    thing = Thing("x")
    reaper.register(thing, lambda: None)
    assert len(reaper._entries) == 1 and len(reaper._by_obj) == 1
    del thing
    gc.collect()
    assert reaper._entries == {} and reaper._order == {} and reaper._by_obj == {}
    assert reaper.pending == 0
