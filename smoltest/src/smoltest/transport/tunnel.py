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
from dataclasses import dataclass, field
from typing import Any

from .._log import logger
from .._ports import HostEndpoint
from ..errors import BootError
from .base import StaticBridge

TunnelOpener = Callable[[], AbstractAsyncContextManager[Any]]
"""Zero-argument callable returning an async context manager that yields an
object with ``.host`` and ``.port`` (``smol.tunnel.TunnelEndpoint``,
:class:`~smoltest._ports.HostEndpoint`, …)."""


@dataclass
class _Run:
    """The state of one ``open()``: a loop thread, what it reports and how to stop it.

    Every run owns its fields, so a worker that outlived ``close(timeout_s)`` and
    keeps running writes into a run the bridge has already let go of, never into
    the run that a later ``open()`` started.
    """

    thread: threading.Thread = field(init=False)
    loop: asyncio.AbstractEventLoop | None = None
    task: asyncio.Task[None] | None = None
    stop: asyncio.Event | None = None
    endpoint: HostEndpoint | None = None
    error: BaseException | None = None
    ready: threading.Event = field(default_factory=threading.Event)
    stop_requested: bool = False
    """Set by ``_request_stop`` before it looks for a loop to signal, so a request that
    arrives while the worker is still creating its loop and task is honoured by the
    worker once they exist instead of being lost."""


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
        self._run: _Run | None = None

    # -- public surface --------------------------------------------------------------

    @property
    def endpoint(self) -> HostEndpoint | None:
        """Where the tunnel listens while it is open and healthy, else ``None``."""
        run = self._run
        if run is None or not run.thread.is_alive() or run.error is not None:
            return None
        return run.endpoint

    def open(self) -> HostEndpoint:
        """Open the tunnel (idempotent) and return its host endpoint."""
        with self._lock:
            live = self.endpoint
            if live is not None:
                return live
            run = _Run()
            run.thread = threading.Thread(
                target=self._serve_thread, args=(run,), name="smoltest-tunnel", daemon=True
            )
            self._run = run
            run.thread.start()
            if not run.ready.wait(self._connect_timeout_s):
                self._request_stop(run)
                run.thread.join(timeout=5.0)
                self._run = None
                raise BootError(
                    f"tunnel did not open within {self._connect_timeout_s:g}s", stage="tunnel"
                )
            error, endpoint = run.error, run.endpoint
            if error is not None:
                run.thread.join(timeout=5.0)
                self._run = None
                raise BootError(f"tunnel failed to open: {error}", stage="tunnel", cause=error)
            if endpoint is None or not run.thread.is_alive():
                self._request_stop(run)
                run.thread.join(timeout=5.0)
                self._run = None
                raise BootError("tunnel closed before it became reachable", stage="tunnel")
            return endpoint

    def close(self, timeout_s: float = 5.0) -> None:
        """Leave the tunnel context and join the thread; safe to call repeatedly.

        A thread that does not stop within ``timeout_s`` is abandoned together
        with its run: whatever it still writes lands there, not on this bridge.
        """
        with self._lock:
            run = self._run
            if run is None:
                return
            self._request_stop(run)
            run.thread.join(timeout_s)
            if run.thread.is_alive():
                logger.warning("tunnel thread did not stop within %.1fs; abandoning it", timeout_s)
            self._run = None

    def __enter__(self) -> HostEndpoint:
        return self.open()

    def __exit__(self, *exc: object) -> None:
        self.close()

    def __repr__(self) -> str:
        return f"TunnelBridge(endpoint={self.endpoint}, open={self.endpoint is not None})"

    # -- background loop -------------------------------------------------------------

    @staticmethod
    def _request_stop(run: _Run) -> None:
        """Ask ``run``'s loop thread to leave the tunnel; harmless once it has finished.

        The flag goes first: if the worker has not assigned ``loop`` and ``task``
        yet (``open()`` timed out during its start-up), it sees the flag right
        after creating the task and cancels it, so no tunnel is left open on a
        run the bridge has already discarded.
        """
        run.stop_requested = True
        loop, task = run.loop, run.task
        if loop is None or task is None or loop.is_closed():
            return

        def signal() -> None:
            if run.stop is not None and run.endpoint is not None:
                run.stop.set()
            else:
                task.cancel()

        with contextlib.suppress(RuntimeError):  # the loop closed between check and call
            loop.call_soon_threadsafe(signal)

    async def _serve(self, run: _Run) -> None:
        run.stop = asyncio.Event()
        async with self._open_tunnel() as raw:
            run.endpoint = HostEndpoint(str(raw.host), int(raw.port))
            run.ready.set()
            await run.stop.wait()

    def _serve_thread(self, run: _Run) -> None:
        loop = asyncio.new_event_loop()
        run.loop = loop
        try:
            run.task = loop.create_task(self._serve(run))
            if run.stop_requested:  # a stop that arrived before the task existed
                run.task.cancel()
            loop.run_until_complete(run.task)
        except asyncio.CancelledError:
            pass
        except BaseException as exc:  # surfaced to the opener in open()
            run.error = exc
        finally:
            try:
                loop.run_until_complete(loop.shutdown_asyncgens())
            finally:
                loop.close()
                run.ready.set()


__all__ = ["StaticBridge", "TunnelBridge", "TunnelOpener"]
