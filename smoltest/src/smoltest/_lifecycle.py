"""Process-exit cleanup of machines and bridges.

The :class:`Reaper` wraps :func:`weakref.finalize` so an object is cleaned up
when it is garbage collected *or* at interpreter exit, whichever comes first,
in LIFO order with children closed before their parents. A registration is
forgotten as soon as it has run or been detached, so the registry only ever
holds the live ones. A forked child forgets them all: the parent's machines
and claims are the parent's to close.
"""

from __future__ import annotations

import atexit
import os
import threading
import weakref
from collections.abc import Callable
from dataclasses import dataclass, field
from typing import Any, TypeAlias

from ._log import logger

Finalizer: TypeAlias = "weakref.finalize[Any, Any]"
"""The token :meth:`Reaper.register` returns."""


@dataclass
class _Entry:
    token: Finalizer
    parent: Finalizer | None
    obj_id: int
    children: list[Finalizer] = field(default_factory=list)


def _noop() -> None:
    """Callback of the throwaway finalizer that pins weakref's exit hook ahead of ours."""


class Reaper:
    """Registry of close functions run at exit in a deterministic order."""

    def __init__(self, *, install_atexit: bool = True) -> None:
        self._lock = threading.RLock()
        self._entries: dict[Finalizer, _Entry] = {}
        self._order: dict[Finalizer, None] = {}  # insertion-ordered set
        self._by_obj: dict[int, Finalizer] = {}
        _reapers.add(self)
        if install_atexit:
            # weakref registers its own exit hook when the first ``finalize`` of
            # the process is created, and atexit runs hooks LIFO. Registered after
            # ours, that hook would run first, set ``finalize._shutdown`` and turn
            # every ``token()`` in :meth:`run` into a no-op, leaving machines
            # running. Creating (and dropping) a finalizer here pins weakref's
            # hook ahead of ours.
            weakref.finalize(self, _noop).detach()
            atexit.register(self.run)

    def register(
        self,
        obj: object,
        close_fn: Callable[[], None],
        parent: Finalizer | object | None = None,
    ) -> Finalizer:
        """Arrange for ``close_fn()`` to run when ``obj`` dies or the process exits.

        ``parent`` is the token (or the object) of a registration that must outlive
        this one; at exit children run before parents. The returned token can be
        called to clean up early or cancelled with :meth:`detach`.
        """
        # The finalizer needs its own token to forget itself once it has run; the
        # token does not exist until ``weakref.finalize`` returns, hence the box.
        box: list[Finalizer] = []
        token = weakref.finalize(obj, self._fire, close_fn, box)
        token.atexit = False  # our own atexit hook orders the calls
        box.append(token)
        with self._lock:
            parent_token = self._resolve_parent(parent)
            self._entries[token] = _Entry(token, parent_token, id(obj))
            self._order[token] = None
            self._by_obj[id(obj)] = token
            if parent_token is not None and parent_token in self._entries:
                self._entries[parent_token].children.append(token)
        return token

    def detach(self, token: Finalizer) -> bool:
        """Cancel ``token`` and forget it; ``True`` when its cleanup had not run yet."""
        detached = token.detach() is not None
        with self._lock:
            self._forget(token)
        return detached

    def _resolve_parent(self, parent: Finalizer | object | None) -> Finalizer | None:
        if parent is None:
            return None
        if isinstance(parent, weakref.finalize):
            return parent
        token = self._by_obj.get(id(parent))
        if token is None:
            return None
        peeked = token.peek()
        if peeked is None or peeked[0] is not parent:
            return None
        return token

    def _fire(self, close_fn: Callable[[], None], box: list[Finalizer]) -> None:
        try:
            close_fn()
        except Exception:
            logger.exception("smoltest cleanup failed")
        finally:
            if box:
                with self._lock:
                    self._forget(box[0])

    @property
    def pending(self) -> int:
        """Registrations whose object is alive and not yet cleaned up."""
        with self._lock:
            tokens = list(self._order)
        return sum(1 for token in tokens if token.alive)

    def run(self) -> None:
        """Run every pending cleanup: LIFO, children before parents. Idempotent."""
        with self._lock:
            order = list(reversed(self._order))
        for token in order:
            self._run_tree(token)
        with self._lock:
            for token in list(self._order):
                if not token.alive:
                    self._forget(token)

    def _run_tree(self, token: Finalizer) -> None:
        with self._lock:
            entry = self._entries.get(token)
            children = list(reversed(entry.children)) if entry else []
        for child in children:
            self._run_tree(child)
        if token.alive:
            token()

    def _reset_after_fork(self) -> None:
        """Forget the parent's registrations; runs in a forked child.

        The child inherits every finalizer and this reaper's exit hook, so a
        garbage collection or a normal exit there would run the parent's close
        functions: delete its machines, release its cache claims. Detaching
        leaves the inherited objects alone and keeps the reaper usable for the
        child's own registrations.
        """
        self._lock = threading.RLock()  # a parent thread may have held it at the fork
        for token in list(self._order):
            token.detach()
        self._entries.clear()
        self._order.clear()
        self._by_obj.clear()

    def _forget(self, token: Finalizer) -> None:
        """Drop every trace of ``token``; the caller holds the lock."""
        entry = self._entries.pop(token, None)
        self._order.pop(token, None)
        if entry is None:
            return
        if self._by_obj.get(entry.obj_id) is token:
            del self._by_obj[entry.obj_id]
        if entry.parent is not None:
            parent_entry = self._entries.get(entry.parent)
            if parent_entry is not None and token in parent_entry.children:
                parent_entry.children.remove(token)


_reapers: weakref.WeakSet[Reaper] = weakref.WeakSet()
_default: Reaper | None = None
_default_lock = threading.Lock()


def get_reaper() -> Reaper:
    """Return the process-wide :class:`Reaper`, creating it on first use."""
    global _default  # noqa: PLW0603 - process-wide singleton by design
    with _default_lock:
        if _default is None:
            _default = Reaper()
        return _default


def _after_fork_in_child() -> None:
    global _default_lock  # noqa: PLW0603 - replaced, a parent thread may hold the old one
    _default_lock = threading.Lock()
    for reaper in list(_reapers):
        reaper._reset_after_fork()


if hasattr(os, "register_at_fork"):  # POSIX; elsewhere there is no fork to survive
    os.register_at_fork(after_in_child=_after_fork_in_child)


__all__ = ["Finalizer", "Reaper", "get_reaper"]
