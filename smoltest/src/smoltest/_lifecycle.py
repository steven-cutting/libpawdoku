"""Process-exit cleanup of machines and bridges.

The :class:`Reaper` wraps :func:`weakref.finalize` so an object is cleaned up
when it is garbage collected *or* at interpreter exit, whichever comes first,
in LIFO order with children closed before their parents.
"""

from __future__ import annotations

import atexit
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
    children: list[Finalizer] = field(default_factory=list)


class Reaper:
    """Registry of close functions run at exit in a deterministic order."""

    def __init__(self, *, install_atexit: bool = True) -> None:
        self._lock = threading.RLock()
        self._entries: dict[Finalizer, _Entry] = {}
        self._order: list[Finalizer] = []
        self._by_obj: dict[int, Finalizer] = {}
        if install_atexit:
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
        called to clean up early or ``detach()``-ed to cancel.
        """
        token = weakref.finalize(obj, self._fire, close_fn)
        token.atexit = False  # our own atexit hook orders the calls
        with self._lock:
            parent_token = self._resolve_parent(parent)
            self._entries[token] = _Entry(token, parent_token)
            self._order.append(token)
            self._by_obj[id(obj)] = token
            if parent_token is not None and parent_token in self._entries:
                self._entries[parent_token].children.append(token)
        return token

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

    def _fire(self, close_fn: Callable[[], None]) -> None:
        try:
            close_fn()
        except Exception:
            logger.exception("smoltest cleanup failed")

    @property
    def pending(self) -> int:
        """Registrations whose object is alive and not yet cleaned up."""
        with self._lock:
            return sum(1 for token in self._order if token.alive)

    def run(self) -> None:
        """Run every pending cleanup: LIFO, children before parents. Idempotent."""
        with self._lock:
            order = list(reversed(self._order))
        for token in order:
            self._run_tree(token)
        with self._lock:
            dead = [t for t in self._order if not t.alive]
            for token in dead:
                self._forget(token)

    def _run_tree(self, token: Finalizer) -> None:
        with self._lock:
            entry = self._entries.get(token)
            children = list(reversed(entry.children)) if entry else []
        for child in children:
            self._run_tree(child)
        if token.alive:
            token()

    def _forget(self, token: Finalizer) -> None:
        self._entries.pop(token, None)
        if token in self._order:
            self._order.remove(token)
        for key, value in list(self._by_obj.items()):
            if value is token:
                del self._by_obj[key]


_default: Reaper | None = None
_default_lock = threading.Lock()


def get_reaper() -> Reaper:
    """Return the process-wide :class:`Reaper`, creating it on first use."""
    global _default  # noqa: PLW0603 - process-wide singleton by design
    with _default_lock:
        if _default is None:
            _default = Reaper()
        return _default


__all__ = ["Finalizer", "Reaper", "get_reaper"]
