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


def test_gc_removes_the_entry() -> None:
    reaper = Reaper(install_atexit=False)
    thing = Thing("x")
    reaper.register(thing, lambda: None)
    assert len(reaper._entries) == 1 and len(reaper._by_obj) == 1
    del thing
    gc.collect()
    assert reaper._entries == {} and reaper._order == {} and reaper._by_obj == {}
    assert reaper.pending == 0
