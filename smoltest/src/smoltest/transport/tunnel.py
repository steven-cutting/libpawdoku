"""Bridges that make a guest TCP port reachable from synchronous host code.

The Smol SDK exposes cloud TCP access only as an *async* context manager
(``smol.tunnel.open_tunnel(machine, port)``) that keeps a loopback relay alive
while it is entered. :class:`TunnelBridge` hosts such a context manager on a
daemon thread running its own asyncio loop so blocking callers (psycopg, the
pytest plugin, the CLI) get a plain ``(host, port)`` and a ``close()``.
:class:`StaticBridge` covers the local target, where the host already publishes
the port. Nothing here imports ``smol``.
"""

from __future__ import annotations

import asyncio
import contextlib
import threading
from collections.abc import Callable
from contextlib import AbstractAsyncContextManager
from typing import Any

from .._log import logger
from .._ports import HostEndpoint
from ..errors import BootError
from .base import StaticBridge

TunnelOpener = Callable[[], AbstractAsyncContextManager[Any]]
"""Zero-argument callable returning an async context manager that yields an
object with ``.host`` and ``.port`` (``smol.tunnel.TunnelEndpoint``,
:class:`~smoltest._ports.HostEndpoint`, …)."""


class TunnelBridge:
    """Keep an async tunnel open on a background thread and expose its endpoint.

    ``open()`` starts a daemon thread, enters ``open_tunnel()`` inside a fresh
    asyncio loop and returns once the tunnel reports where it listens; a failure
    to open is re-raised in the calling thread as :class:`BootError` with
    ``stage="tunnel"`` and the thread exits. ``close()`` asks the loop to leave
    the context manager and joins the thread. Both are idempotent and
    thread-safe; the object is also a context manager.
    """

    def __init__(self, open_tunnel: TunnelOpener, *, connect_timeout_s: float = 30.0) -> None:
        self._open_tunnel = open_tunnel
        self._connect_timeout_s = connect_timeout_s
        self._lock = threading.Lock()
        self._ready = threading.Event()
        self._thread: threading.Thread | None = None
        self._loop: asyncio.AbstractEventLoop | None = None
        self._task: asyncio.Task[None] | None = None
        self._stop: asyncio.Event | None = None
        self._endpoint: HostEndpoint | None = None
        self._error: BaseException | None = None

    # -- public surface --------------------------------------------------------------

    @property
    def endpoint(self) -> HostEndpoint | None:
        """Where the tunnel listens while it is open and healthy, else ``None``."""
        thread = self._thread
        if thread is None or not thread.is_alive() or self._error is not None:
            return None
        return self._endpoint

    def open(self) -> HostEndpoint:
        """Open the tunnel (idempotent) and return its host endpoint."""
        with self._lock:
            live = self.endpoint
            if live is not None:
                return live
            self._reset()
            thread = threading.Thread(target=self._run, name="smoltest-tunnel", daemon=True)
            self._thread = thread
            thread.start()
            if not self._ready.wait(self._connect_timeout_s):
                self._request_stop()
                thread.join(timeout=5.0)
                self._reset()
                raise BootError(
                    f"tunnel did not open within {self._connect_timeout_s:g}s", stage="tunnel"
                )
            error, endpoint = self._error, self._endpoint
            if error is not None:
                thread.join(timeout=5.0)
                self._reset()
                raise BootError(f"tunnel failed to open: {error}", stage="tunnel", cause=error)
            if endpoint is None or not thread.is_alive():
                thread.join(timeout=5.0)
                self._reset()
                raise BootError("tunnel closed before it became reachable", stage="tunnel")
            return endpoint

    def close(self, timeout_s: float = 5.0) -> None:
        """Leave the tunnel context and join the thread; safe to call repeatedly."""
        with self._lock:
            thread = self._thread
            if thread is None:
                return
            self._request_stop()
            thread.join(timeout_s)
            if thread.is_alive():
                logger.warning("tunnel thread did not stop within %.1fs; abandoning it", timeout_s)
            self._reset()

    def __enter__(self) -> HostEndpoint:
        return self.open()

    def __exit__(self, *exc: object) -> None:
        self.close()

    def __repr__(self) -> str:
        return f"TunnelBridge(endpoint={self.endpoint}, open={self.endpoint is not None})"

    # -- background loop -------------------------------------------------------------

    def _reset(self) -> None:
        self._thread = None
        self._loop = None
        self._task = None
        self._stop = None
        self._endpoint = None
        self._error = None
        self._ready.clear()

    def _request_stop(self) -> None:
        """Ask the loop thread to leave the tunnel; harmless once it has finished."""
        loop, task = self._loop, self._task
        if loop is None or task is None or loop.is_closed():
            return

        def signal() -> None:
            stop = self._stop
            if stop is not None and self._endpoint is not None:
                stop.set()
            else:
                task.cancel()

        with contextlib.suppress(RuntimeError):  # the loop closed between check and call
            loop.call_soon_threadsafe(signal)

    async def _serve(self) -> None:
        self._stop = asyncio.Event()
        async with self._open_tunnel() as raw:
            self._endpoint = HostEndpoint(str(raw.host), int(raw.port))
            self._ready.set()
            await self._stop.wait()

    def _run(self) -> None:
        loop = asyncio.new_event_loop()
        self._loop = loop
        try:
            self._task = loop.create_task(self._serve())
            loop.run_until_complete(self._task)
        except asyncio.CancelledError:
            pass
        except BaseException as exc:  # surfaced to the opener in open()
            self._error = exc
        finally:
            try:
                loop.run_until_complete(loop.shutdown_asyncgens())
            finally:
                loop.close()
                self._ready.set()


__all__ = ["StaticBridge", "TunnelBridge", "TunnelOpener"]
