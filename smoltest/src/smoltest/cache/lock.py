"""Advisory file locks (``fcntl.flock``) that also exclude threads of this process.

A :class:`FileLock` is re-entrant for the thread that holds it, so a caller may
hold a key lock across a whole cold boot while the store's own methods take the
same lock again underneath. Instances are shared per path through
:meth:`FileLock.for_path`, which is what makes that re-entrancy work across
separately opened caches.

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


class FileLock:
    """A re-entrant, thread-aware ``flock`` on ``path``; a context manager.

    ``acquire`` blocks (polling every ``poll_interval_s``) until the lock is free
    or ``timeout_s`` elapses, then raises :class:`LockTimeout`. ``None`` waits
    forever. The lock file is created with mode ``0o600`` and never deleted.
    """

    _registry: weakref.WeakValueDictionary[str, FileLock] = weakref.WeakValueDictionary()
    _registry_lock = threading.Lock()
    # Every live instance, registered or not, so a fork can reset each one.
    _instances: weakref.WeakSet[FileLock] = weakref.WeakSet()
    _instances_lock = threading.Lock()

    def __init__(
        self,
        path: str | Path,
        *,
        timeout_s: float | None = DEFAULT_TIMEOUT_S,
        poll_interval_s: float = DEFAULT_POLL_S,
    ) -> None:
        self.path = Path(path)
        self.timeout_s = timeout_s
        self.poll_interval_s = poll_interval_s
        self._tlock = threading.RLock()
        self._fd: int | None = None
        self._depth = 0
        self._shared = False
        with FileLock._instances_lock:
            FileLock._instances.add(self)

    @classmethod
    def for_path(
        cls,
        path: str | Path,
        *,
        timeout_s: float | None = DEFAULT_TIMEOUT_S,
        poll_interval_s: float = DEFAULT_POLL_S,
    ) -> FileLock:
        """Return the process-wide :class:`FileLock` instance for ``path``.

        Sharing one instance per path is what lets the same thread take a lock
        it already holds through a different :class:`CheckpointCache` object.
        """
        key = str(Path(path).resolve())
        with cls._registry_lock:
            lock = cls._registry.get(key)
            if lock is None:
                # Built from the resolved path, never the one given: the shared
                # instance outlives the caller's working directory, so a relative
                # path would lock a different file after a chdir.
                lock = cls(key, timeout_s=timeout_s, poll_interval_s=poll_interval_s)
                cls._registry[key] = lock
            return lock

    @classmethod
    def _reset_after_fork(cls) -> None:
        """Forget every hold the parent had; runs in the child right after ``fork``.

        The child inherits each instance's ``_depth``, thread lock and descriptor,
        which would make its first ``acquire`` look re-entrant and skip ``flock``,
        and a ``release`` there would ``LOCK_UN`` the open file description it
        shares with the parent. So: descriptors are closed *without* unlocking
        (the parent's copy keeps the lock), the state is reset in place so a cache
        object the child inherited takes a fresh lock through the same per-path
        instance, and the class-level locks are replaced in case another thread
        of the parent held one at the moment of the fork. A ``with lock:`` block
        that straddles a plain ``os.fork()`` therefore leaves the child outside
        the lock, and a ``release()`` there raises; ``multiprocessing`` children
        never return into such a block.
        """
        cls._registry_lock = threading.Lock()
        cls._instances_lock = threading.Lock()
        for lock in list(cls._instances):
            fd, lock._fd = lock._fd, None
            lock._depth = 0
            lock._shared = False
            lock._tlock = threading.RLock()
            if fd is not None:
                with contextlib.suppress(OSError):
                    os.close(fd)

    def __repr__(self) -> str:
        mode = "shared" if self._shared else "exclusive"
        return f"FileLock({str(self.path)!r}, held={self.held}, mode={mode})"

    @property
    def held(self) -> bool:
        """``True`` while some thread of this process holds the lock."""
        return self._depth > 0

    @property
    def is_shared(self) -> bool:
        """``True`` when the current hold is a shared (read) lock."""
        return self.held and self._shared

    def acquire(self, timeout_s: float | None = _UNSET, *, shared: bool = False) -> None:
        """Take the lock; ``shared=True`` takes a read lock other readers may share.

        Re-entrant: a thread that holds the lock may take it again (an exclusive
        hold satisfies a nested shared request; the reverse raises
        :class:`~smoltest.errors.CacheError`).
        """
        timeout = self.timeout_s if timeout_s is _UNSET else timeout_s
        deadline = None if timeout is None else time.monotonic() + max(timeout, 0.0)
        if not self._tlock.acquire(timeout=-1 if timeout is None else max(timeout, 0.0)):
            raise LockTimeout(self.path, timeout or 0.0)
        try:
            if self._depth == 0:
                self._flock(deadline, shared, timeout)
                self._shared = shared
            elif self._shared and not shared:
                raise CacheError(f"cannot upgrade shared lock {self.path} to exclusive")
            self._depth += 1
        except BaseException:
            self._tlock.release()
            raise

    def _flock(self, deadline: float | None, shared: bool, timeout: float | None) -> None:
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
        self._fd = fd

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
        if self._depth == 0:
            raise CacheError(f"lock {self.path} is not held")
        self._depth -= 1
        if self._depth == 0 and self._fd is not None:
            try:
                fcntl.flock(self._fd, fcntl.LOCK_UN)
            finally:
                os.close(self._fd)
                self._fd = None
                self._shared = False
        self._tlock.release()

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
