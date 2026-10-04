"""Advisory file locks (``fcntl.flock``) that also exclude threads of this process.

A :class:`FileLock` is re-entrant for the thread that holds it, so a caller may
hold a key lock across a whole cold boot while the store's own methods take the
same lock again underneath. The *hold* (thread lock, descriptor, depth) is shared
per path among every lock :meth:`FileLock.for_path` returns, which is what makes
that re-entrancy work across separately opened caches; the timeout and poll
interval stay the caller's, so one cache's patience never binds another's.

A forked child starts with every hold forgotten (see
:meth:`FileLock._reset_after_fork`): a lock the parent held is the parent's, and
the child takes its own ``flock`` when it asks for one.
"""

from __future__ import annotations

import contextlib
import errno
import fcntl
import os
import threading
import time
import weakref
from collections.abc import Iterator
from contextlib import contextmanager
from pathlib import Path
from typing import Any

from ..errors import CacheError

DEFAULT_TIMEOUT_S = 300.0
DEFAULT_POLL_S = 0.05
_WOULD_BLOCK = frozenset({errno.EAGAIN, errno.EWOULDBLOCK, errno.EACCES})
_UNSET: Any = object()


class LockTimeout(CacheError, TimeoutError):
    """The lock stayed held by someone else for longer than the timeout."""

    def __init__(self, path: Path, timeout_s: float) -> None:
        message = f"timed out after {timeout_s:g}s waiting for lock {path}"
        super().__init__(message, code="LOCK_TIMEOUT")
        self.path = path
        self.timeout_s = timeout_s


class _Hold:
    """The state every :class:`FileLock` on one path shares: who holds it, and how."""

    __slots__ = ("__weakref__", "depth", "fd", "shared", "tlock")

    def __init__(self) -> None:
        self.tlock = threading.RLock()
        self.fd: int | None = None
        self.depth = 0
        self.shared = False

    def reset(self) -> None:
        """Forget the hold without unlocking: what a forked child must do."""
        fd, self.fd = self.fd, None
        self.depth = 0
        self.shared = False
        self.tlock = threading.RLock()
        if fd is not None:
            with contextlib.suppress(OSError):
                os.close(fd)


class FileLock:
    """A re-entrant, thread-aware ``flock`` on ``path``; a context manager.

    ``acquire`` blocks (polling every ``poll_interval_s``) until the lock is free
    or ``timeout_s`` elapses, then raises :class:`LockTimeout`. ``None`` waits
    forever. The lock file is created with mode ``0o600`` and never deleted.
    """

    _registry: weakref.WeakValueDictionary[str, _Hold] = weakref.WeakValueDictionary()
    _registry_lock = threading.Lock()
    # Every live hold, registered or not, so a fork can reset each one.
    _holds: weakref.WeakSet[_Hold] = weakref.WeakSet()
    _holds_lock = threading.Lock()

    def __init__(
        self,
        path: str | Path,
        *,
        timeout_s: float | None = DEFAULT_TIMEOUT_S,
        poll_interval_s: float = DEFAULT_POLL_S,
        _hold: _Hold | None = None,
    ) -> None:
        self.path = Path(path)
        self.timeout_s = timeout_s
        self.poll_interval_s = poll_interval_s
        self._hold = _hold if _hold is not None else self._new_hold()

    @classmethod
    def _new_hold(cls) -> _Hold:
        hold = _Hold()
        with cls._holds_lock:
            cls._holds.add(hold)
        return hold

    @classmethod
    def for_path(
        cls,
        path: str | Path,
        *,
        timeout_s: float | None = DEFAULT_TIMEOUT_S,
        poll_interval_s: float = DEFAULT_POLL_S,
    ) -> FileLock:
        """A lock on ``path`` whose hold is shared with every other lock from here.

        Sharing the hold per path is what lets the same thread take a lock it
        already holds through a different :class:`CheckpointCache` object. The
        returned object carries *this* caller's ``timeout_s`` and
        ``poll_interval_s``: a later caller with a shorter timeout waits only
        that long, whatever an earlier caller asked for.
        """
        # Keyed and built from the resolved path, never the one given: the shared
        # hold outlives the caller's working directory, so a relative path would
        # lock a different file after a chdir.
        key = str(Path(path).resolve())
        with cls._registry_lock:
            hold = cls._registry.get(key)
            if hold is None:
                hold = cls._new_hold()
                cls._registry[key] = hold
        return cls(key, timeout_s=timeout_s, poll_interval_s=poll_interval_s, _hold=hold)

    def shares_hold_with(self, other: FileLock) -> bool:
        """``True`` when ``self`` and ``other`` are the same lock in all but their timeouts."""
        return self._hold is other._hold

    @classmethod
    def _reset_after_fork(cls) -> None:
        """Forget every hold the parent had; runs in the child right after ``fork``.

        The child inherits each hold's depth, thread lock and descriptor, which
        would make its first ``acquire`` look re-entrant and skip ``flock``, and
        a ``release`` there would ``LOCK_UN`` the open file description it shares
        with the parent. So: descriptors are closed *without* unlocking (the
        parent's copy keeps the lock), the holds are reset in place so a cache
        object the child inherited takes a fresh lock through the same per-path
        hold, and the class-level locks are replaced in case another thread of
        the parent held one at the moment of the fork. A ``with lock:`` block
        that straddles a plain ``os.fork()`` therefore leaves the child outside
        the lock, and a ``release()`` there raises; ``multiprocessing`` children
        never return into such a block.
        """
        cls._registry_lock = threading.Lock()
        cls._holds_lock = threading.Lock()
        for hold in list(cls._holds):
            hold.reset()

    def __repr__(self) -> str:
        mode = "shared" if self._hold.shared else "exclusive"
        return f"FileLock({str(self.path)!r}, held={self.held}, mode={mode})"

    @property
    def held(self) -> bool:
        """``True`` while some thread of this process holds the lock."""
        return self._hold.depth > 0

    @property
    def is_shared(self) -> bool:
        """``True`` when the current hold is a shared (read) lock."""
        return self.held and self._hold.shared

    def acquire(self, timeout_s: float | None = _UNSET, *, shared: bool = False) -> None:
        """Take the lock; ``shared=True`` takes a read lock other readers may share.

        Re-entrant: a thread that holds the lock may take it again (an exclusive
        hold satisfies a nested shared request; the reverse raises
        :class:`~smoltest.errors.CacheError`).
        """
        hold = self._hold
        timeout = self.timeout_s if timeout_s is _UNSET else timeout_s
        deadline = None if timeout is None else time.monotonic() + max(timeout, 0.0)
        if not hold.tlock.acquire(timeout=-1 if timeout is None else max(timeout, 0.0)):
            raise LockTimeout(self.path, timeout or 0.0)
        try:
            if hold.depth == 0:
                hold.fd = self._flock(deadline, shared, timeout)
                hold.shared = shared
            elif hold.shared and not shared:
                raise CacheError(f"cannot upgrade shared lock {self.path} to exclusive")
            hold.depth += 1
        except BaseException:
            hold.tlock.release()
            raise

    def _flock(self, deadline: float | None, shared: bool, timeout: float | None) -> int:
        self.path.parent.mkdir(parents=True, exist_ok=True, mode=0o700)
        fd = os.open(self.path, os.O_RDWR | os.O_CREAT | os.O_CLOEXEC, 0o600)
        op = (fcntl.LOCK_SH if shared else fcntl.LOCK_EX) | fcntl.LOCK_NB
        try:
            while not self._try_flock(fd, op):
                if deadline is not None and time.monotonic() >= deadline:
                    raise LockTimeout(self.path, timeout or 0.0)
                time.sleep(self.poll_interval_s)
        except BaseException:
            os.close(fd)
            raise
        return fd

    def _try_flock(self, fd: int, op: int) -> bool:
        try:
            fcntl.flock(fd, op)
        except OSError as exc:
            if exc.errno not in _WOULD_BLOCK:
                raise CacheError(f"cannot lock {self.path}: {exc}") from exc
            return False
        return True

    def release(self) -> None:
        """Give the lock back; the file lock drops with the outermost release."""
        hold = self._hold
        if hold.depth == 0:
            raise CacheError(f"lock {self.path} is not held")
        hold.depth -= 1
        if hold.depth == 0 and hold.fd is not None:
            # Cleared before the close: a fork in between must not see a number
            # that another thread has since reused and close that in the child.
            fd, hold.fd = hold.fd, None
            try:
                fcntl.flock(fd, fcntl.LOCK_UN)
            finally:
                os.close(fd)
                hold.shared = False
        hold.tlock.release()

    def __enter__(self) -> FileLock:
        self.acquire()
        return self

    def __exit__(self, *exc: object) -> None:
        self.release()

    @contextmanager
    def shared_lock(self, timeout_s: float | None = _UNSET) -> Iterator[FileLock]:
        """``with lock.shared_lock():`` takes and releases a read lock."""
        self.acquire(timeout_s, shared=True)
        try:
            yield self
        finally:
            self.release()


if hasattr(os, "register_at_fork"):  # POSIX only, like ``fcntl`` above
    os.register_at_fork(after_in_child=FileLock._reset_after_fork)


__all__ = ["DEFAULT_POLL_S", "DEFAULT_TIMEOUT_S", "FileLock", "LockTimeout"]
