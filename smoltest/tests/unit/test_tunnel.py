"""TunnelBridge keeps an async tunnel open on a thread and hands out a plain endpoint.

The tunnels here are this file's own ``asynccontextmanager`` relays to a local
TCP echo server, shaped like ``smol.tunnel.open_tunnel``; nothing imports ``smol``.
"""

from __future__ import annotations

import asyncio
import contextlib
import socket
import threading
from collections.abc import AsyncIterator, Iterator
from dataclasses import dataclass

import pytest

from smoltest._ports import HostEndpoint, is_port_free
from smoltest.errors import BootError
from smoltest.testing import FakeEngine
from smoltest.transport.base import Bridge, MachineSpec, PortMapping
from smoltest.transport.base import StaticBridge as BaseStaticBridge
from smoltest.transport.tunnel import StaticBridge, TunnelBridge, TunnelOpener

THREAD_NAME = "smoltest-tunnel"


class EchoServer:
    """A threaded loopback TCP server that writes back whatever it reads."""

    def __init__(self) -> None:
        self._sock = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
        self._sock.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)
        self._sock.bind(("127.0.0.1", 0))
        self._sock.listen(8)
        self._sock.settimeout(0.1)
        self.port: int = self._sock.getsockname()[1]
        self._stop = threading.Event()
        self._threads: list[threading.Thread] = []
        self._acceptor = threading.Thread(target=self._accept, name="echo-accept", daemon=True)
        self._acceptor.start()

    def _accept(self) -> None:
        while not self._stop.is_set():
            try:
                client, _ = self._sock.accept()
            except TimeoutError:
                continue
            except OSError:
                break
            worker = threading.Thread(target=self._echo, args=(client,), daemon=True)
            self._threads.append(worker)
            worker.start()

    @staticmethod
    def _echo(client: socket.socket) -> None:
        with client:
            while True:
                try:
                    chunk = client.recv(65536)
                except OSError:
                    return
                if not chunk:
                    return
                client.sendall(chunk)

    def close(self) -> None:
        self._stop.set()
        self._acceptor.join(timeout=2)
        with contextlib.suppress(OSError):
            self._sock.close()


@pytest.fixture
def echo() -> Iterator[EchoServer]:
    server = EchoServer()
    yield server
    server.close()


@dataclass(frozen=True)
class Ep:
    """Duck-typed endpoint, shaped like ``smol.tunnel.TunnelEndpoint``."""

    host: str
    port: int


def relay_to(target_port: int, log: list[str]) -> TunnelOpener:
    """An opener whose context manager relays a fresh loopback listener to ``target_port``."""

    async def pump(reader: asyncio.StreamReader, writer: asyncio.StreamWriter) -> None:
        try:
            while chunk := await reader.read(65536):
                writer.write(chunk)
                await writer.drain()
        finally:
            writer.close()

    async def handle(reader: asyncio.StreamReader, writer: asyncio.StreamWriter) -> None:
        up_reader, up_writer = await asyncio.open_connection("127.0.0.1", target_port)
        await asyncio.gather(
            pump(reader, up_writer), pump(up_reader, writer), return_exceptions=True
        )

    @contextlib.asynccontextmanager
    async def relay() -> AsyncIterator[Ep]:
        server = await asyncio.start_server(handle, "127.0.0.1", 0)
        log.append("open")
        try:
            yield Ep("127.0.0.1", server.sockets[0].getsockname()[1])
        finally:
            server.close()
            await server.wait_closed()
            log.append("close")

    return relay


def round_trip(endpoint: HostEndpoint, payload: bytes) -> bytes:
    with socket.create_connection((endpoint.host, endpoint.port), timeout=5) as sock:
        sock.settimeout(5)
        sock.sendall(payload)
        received = b""
        while len(received) < len(payload):
            chunk = sock.recv(65536)
            if not chunk:
                break
            received += chunk
        return received


def tunnel_threads() -> list[threading.Thread]:
    return [t for t in threading.enumerate() if t.name == THREAD_NAME]


def always() -> bool:
    """``True`` in a way the type checker cannot see through."""
    return True


def test_round_trip_through_the_bridge_and_close_joins(echo: EchoServer) -> None:
    log: list[str] = []
    bridge = TunnelBridge(relay_to(echo.port, log))
    assert isinstance(bridge, Bridge)
    before = bridge.endpoint
    assert before is None
    ep = bridge.open()
    assert ep.host == "127.0.0.1" and ep.port != echo.port
    during = bridge.endpoint
    assert during == ep
    assert bridge.open() == ep, "open() is idempotent while the tunnel is up"
    assert round_trip(ep, b"ping" * 1000) == b"ping" * 1000
    assert log == ["open"]
    assert tunnel_threads()
    bridge.close()
    after = bridge.endpoint
    assert after is None
    assert log == ["open", "close"]
    assert not tunnel_threads(), "close() joins the loop thread"
    assert is_port_free(ep.port)


def test_context_manager_opens_and_closes(echo: EchoServer) -> None:
    log: list[str] = []
    bridge = TunnelBridge(relay_to(echo.port, log))
    with bridge as ep:
        assert round_trip(ep, b"hello") == b"hello"
    assert bridge.endpoint is None and log == ["open", "close"]


def test_reopen_after_close_gives_a_working_tunnel(echo: EchoServer) -> None:
    log: list[str] = []
    bridge = TunnelBridge(relay_to(echo.port, log))
    first = bridge.open()
    bridge.close()
    second = bridge.open()
    assert round_trip(second, b"again") == b"again"
    bridge.close()
    assert log == ["open", "close", "open", "close"]
    assert is_port_free(first.port) and is_port_free(second.port)


def test_open_failure_raises_boot_error_in_caller_and_thread_exits() -> None:
    @contextlib.asynccontextmanager
    async def broken() -> AsyncIterator[Ep]:
        if always():
            raise RuntimeError("no tunnel for you")
        yield Ep("127.0.0.1", 0)  # never reached; makes this an async generator

    bridge = TunnelBridge(broken)
    with pytest.raises(BootError, match="no tunnel for you") as info:
        bridge.open()
    assert info.value.stage == "tunnel"
    assert isinstance(info.value.__cause__, RuntimeError)
    assert bridge.endpoint is None
    assert not tunnel_threads()
    bridge.close()  # nothing to do, must not raise


def test_open_timeout_cancels_a_tunnel_that_never_comes_up() -> None:
    events: list[str] = []

    @contextlib.asynccontextmanager
    async def stuck() -> AsyncIterator[Ep]:
        try:
            await asyncio.Event().wait()
            yield Ep("127.0.0.1", 0)  # pragma: no cover - never reached
        finally:
            events.append("cancelled")

    bridge = TunnelBridge(stuck, connect_timeout_s=0.2)
    with pytest.raises(BootError, match="did not open within") as info:
        bridge.open()
    assert info.value.stage == "tunnel"
    assert events == ["cancelled"]
    assert not tunnel_threads()
    assert bridge.endpoint is None


def test_double_close_and_close_before_open_are_safe(echo: EchoServer) -> None:
    bridge = TunnelBridge(relay_to(echo.port, []))
    bridge.close()
    bridge.close()
    bridge.open()
    bridge.close()
    bridge.close()
    assert bridge.endpoint is None and not tunnel_threads()


def test_error_while_closing_does_not_escape_close(echo: EchoServer) -> None:
    @contextlib.asynccontextmanager
    async def fragile() -> AsyncIterator[Ep]:
        server = await asyncio.start_server(lambda r, w: w.close(), "127.0.0.1", 0)
        try:
            yield Ep("127.0.0.1", server.sockets[0].getsockname()[1])
        finally:
            server.close()
            await server.wait_closed()
            raise RuntimeError("teardown exploded")

    bridge = TunnelBridge(fragile)
    ep = bridge.open()
    during = bridge.endpoint
    assert during == ep
    bridge.close()
    after = bridge.endpoint
    assert after is None
    assert not tunnel_threads()


def test_repr_reports_state(echo: EchoServer) -> None:
    bridge = TunnelBridge(relay_to(echo.port, []))
    assert "open=False" in repr(bridge)
    with bridge:
        assert "open=True" in repr(bridge)


def test_static_bridge_is_reexported_from_base() -> None:
    assert StaticBridge is BaseStaticBridge
    bridge = StaticBridge(HostEndpoint("127.0.0.1", 5432))
    assert bridge.endpoint is None
    assert bridge.open() == HostEndpoint("127.0.0.1", 5432)
    bridge.close()
    assert bridge.endpoint is None


def test_fake_engine_cloud_tunnel_through_the_real_bridge(fake_engine: FakeEngine) -> None:
    """The fake's async tunnel has the SDK's shape, so the real bridge must drive it."""
    fake_engine.tunnel_bridge_factory = TunnelBridge
    spec = MachineSpec(image="postgres:16", ports=(PortMapping(None, 5432),))
    machine = fake_engine.create(spec, "cloud")
    bridge = fake_engine.open_tunnel(machine.id, 5432, "cloud")
    assert isinstance(bridge, TunnelBridge)
    ep = bridge.open()
    with socket.create_connection((ep.host, ep.port), timeout=5) as sock:
        sock.settimeout(0.3)
        sock.sendall(b"startup packet")
        with pytest.raises(TimeoutError):
            sock.recv(1, socket.MSG_PEEK)  # the fake listener accepts and stays silent
    bridge.close()
    assert bridge.endpoint is None
    assert ("tunnel.open", {"machine": machine.id, "guest": 5432}) in fake_engine.calls
    assert ("tunnel.close", {"machine": machine.id, "guest": 5432}) in fake_engine.calls
    machine.delete()
