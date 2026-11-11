"""Logging helpers shared by every smoltest module."""

from __future__ import annotations

import logging
import os
import threading
import warnings

from .errors import SmoltestWarning

logger = logging.getLogger("smoltest")
"""The package logger; handlers are left to the application."""

_warned: set[str] = set()
_warned_lock = threading.Lock()


def warn_once(key: str, message: str, *, stacklevel: int = 2) -> bool:
    """Emit ``message`` as a :class:`SmoltestWarning` the first time ``key`` is seen.

    Returns ``True`` when the warning was emitted and ``False`` when it was suppressed
    because the same key already warned in this process.
    """
    with _warned_lock:
        if key in _warned:
            return False
        _warned.add(key)
    warnings.warn(message, SmoltestWarning, stacklevel=stacklevel + 1)
    logger.warning("%s", message)
    return True


def reset_warn_once() -> None:
    """Forget every key seen by :func:`warn_once` (for tests)."""
    with _warned_lock:
        _warned.clear()


def _reset_after_fork() -> None:
    """Give a forked child a fresh lock.

    Another thread of the parent may have held ``_warned_lock`` at the fork; the
    child would inherit it locked with no thread left to release it, and every
    later :func:`warn_once` would hang. The keys already seen are kept.
    """
    global _warned_lock  # noqa: PLW0603 - replaced, a parent thread may hold the old one
    _warned_lock = threading.Lock()


if hasattr(os, "register_at_fork"):  # POSIX; elsewhere there is no fork to survive
    os.register_at_fork(after_in_child=_reset_after_fork)


__all__ = ["logger", "reset_warn_once", "warn_once"]
