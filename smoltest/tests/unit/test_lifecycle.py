"""Reaper ordering and idempotence."""

from __future__ import annotations

import gc

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
