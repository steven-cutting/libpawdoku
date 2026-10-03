"""In-process fakes of the engine protocols for test suites that must not need KVM.

:class:`FakeEngine` binds *real* loopback listeners for every published port, so
port collisions, bind probes and port-wait strategies behave as they do against
real machines, while ``exec`` is a dispatch table and a checkpoint is a small
JSON "shape" file. Select it with ``SMOLTEST_ENGINE=smoltest.testing:FakeEngine``.
"""

from __future__ import annotations

import asyncio
import contextlib
import hashlib
import itertools
import json
import shutil
import socket
import threading
import time
from collections.abc import Callable, Mapping, Sequence
from pathlib import Path
from typing import Any, ClassVar

from ._ports import LOOPBACK, HostEndpoint, pick_free_port
from .errors import (
    BootError,
    InvalidConfig,
    NotSupportedError,
    ReadinessTimeout,
    SmoltestError,
)
from .transport.base import (
    Bridge,
    CheckpointInfo,
    CheckpointRef,
    ExecOutcome,
    MachineHandle,
    MachineSpec,
    PortMapping,
    StaticBridge,
    Target,
)

DEFAULT_LOG_PATH = "/tmp/postgresql.log"
"""Must equal ``smoltest.boot.spec.LOG_PATH`` (asserted by the unit suite)."""
SHAPE_FORMAT = 1


class _Listener:
    """A loopback listener that accepts connections, holds them open and never writes."""

    def __init__(self, port: int, host: str = LOOPBACK) -> None:
        self.host = host
        self.port = port
        self._sock = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
        self._sock.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)
        try:
            self._sock.bind((host, port))
        except OSError as exc:
            self._sock.close()
            raise BootError(f"port in use: {host}:{port}", stage="bind", cause=exc) from exc
        self._sock.listen(16)
        self._sock.settimeout(0.1)
        self._clients: list[socket.socket] = []
        self._stop = threading.Event()
        self.accepted = 0
        self._thread = threading.Thread(target=self._loop, name=f"fake-listen-{port}", daemon=True)
        self._thread.start()

    def _loop(self) -> None:
        while not self._stop.is_set():
            try:
                client, _ = self._sock.accept()
            except TimeoutError:
                continue
            except OSError:
                break
            self.accepted += 1
            self._clients.append(client)

    def close(self) -> None:
        if self._stop.is_set():
            return
        self._stop.set()
        self._thread.join(timeout=2)
        with contextlib.suppress(OSError):
            self._sock.close()
        for client in self._clients:
            with contextlib.suppress(OSError):
                client.close()
        self._clients.clear()


class FakeAsyncTunnel:
    """Async context manager mimicking ``smol.tunnel.open_tunnel(machine, port)``.

    Entering it starts an asyncio loopback relay that forwards to the fake
    machine's listener and yields a :class:`HostEndpoint`; exiting closes it.
    """

    def __init__(self, machine: FakeMachine, guest_port: int) -> None:
        self.machine = machine
        self.guest_port = guest_port
        self._server: asyncio.base_events.Server | None = None
        self._tasks: set[asyncio.Task[None]] = set()
        self.relayed_bytes = 0

    async def _pump(self, reader: asyncio.StreamReader, writer: asyncio.StreamWriter) -> None:
        try:
            while True:
                chunk = await reader.read(65536)
                if not chunk:
                    break
                self.relayed_bytes += len(chunk)
                writer.write(chunk)
                await writer.drain()
        except (ConnectionError, asyncio.CancelledError, OSError):
            pass
        finally:
            with contextlib.suppress(OSError):
                writer.close()

    async def _handle(self, reader: asyncio.StreamReader, writer: asyncio.StreamWriter) -> None:
        target_port = self.machine.host_ports[self.guest_port]
        try:
            up_reader, up_writer = await asyncio.open_connection(LOOPBACK, target_port)
        except OSError:
            writer.close()
            return
        t1 = asyncio.ensure_future(self._pump(reader, up_writer))
        t2 = asyncio.ensure_future(self._pump(up_reader, writer))
        self._tasks.update({t1, t2})
        await asyncio.gather(t1, t2, return_exceptions=True)
        self._tasks.difference_update({t1, t2})

    async def __aenter__(self) -> HostEndpoint:
        self._server = await asyncio.start_server(self._handle, LOOPBACK, 0)
        port = self._server.sockets[0].getsockname()[1]
        self.machine.engine.record("tunnel.open", machine=self.machine.id, guest=self.guest_port)
        return HostEndpoint(LOOPBACK, port)

    async def __aexit__(self, *exc: object) -> None:
        if self._server is not None:
            self._server.close()
            await self._server.wait_closed()
        for task in list(self._tasks):
            task.cancel()
        if self._tasks:
            await asyncio.gather(*self._tasks, return_exceptions=True)
        self.machine.engine.record("tunnel.close", machine=self.machine.id, guest=self.guest_port)


class RelayBridge:
    """A :class:`Bridge` that keeps a :class:`FakeAsyncTunnel` open on a daemon thread.

    This is the fake counterpart of the real ``TunnelBridge``; the cloud code
    path of smoltest can be exercised through it without any SDK.
    """

    def __init__(self, opener: Callable[[], FakeAsyncTunnel]) -> None:
        self._opener = opener
        self._endpoint: HostEndpoint | None = None
        self._ready = threading.Event()
        self._stop: asyncio.Event | None = None
        self._loop: asyncio.AbstractEventLoop | None = None
        self._thread: threading.Thread | None = None
        self._error: BaseException | None = None

    @property
    def endpoint(self) -> HostEndpoint | None:
        return self._endpoint

    def open(self) -> HostEndpoint:
        if self._thread is None:
            self._thread = threading.Thread(target=self._run, name="fake-relay-bridge", daemon=True)
            self._thread.start()
            self._ready.wait(timeout=10)
        if self._error is not None:
            raise BootError("fake tunnel failed", stage="tunnel", cause=self._error)
        endpoint = self._endpoint
        if endpoint is None:
            raise BootError("fake tunnel did not come up", stage="tunnel")
        return endpoint

    def _run(self) -> None:
        loop = asyncio.new_event_loop()
        self._loop = loop
        try:
            loop.run_until_complete(self._serve())
        except BaseException as exc:
            self._error = exc
            self._ready.set()
        finally:
            loop.close()

    async def _serve(self) -> None:
        self._stop = asyncio.Event()
        async with self._opener() as endpoint:
            self._endpoint = endpoint
            self._ready.set()
            await self._stop.wait()

    def close(self) -> None:
        loop, stop, thread = self._loop, self._stop, self._thread
        if loop is not None and stop is not None and thread is not None and thread.is_alive():
            loop.call_soon_threadsafe(stop.set)
            thread.join(timeout=5)
        self._endpoint = None
        self._thread = None
        self._ready.clear()


def _split_sql(script: str) -> list[str]:
    """Split a script on ``;`` into stripped, non-empty statements (no quoting rules)."""
    return [part.strip() for part in script.split(";") if part.strip()]


class FakeMachine:
    """A fake :class:`~smoltest.transport.base.MachineHandle` with real listeners."""

    def __init__(
        self,
        engine: FakeEngine,
        machine_id: str,
        name: str,
        spec: MachineSpec,
        target: Target,
        *,
        services: set[str],
        host_ports: dict[int, int],
        parent_id: str | None = None,
        via: str = "create",
    ) -> None:
        self.engine = engine
        self.id = machine_id
        self.name = name
        self.spec = spec
        self.target: Target = target
        self.services: set[str] = set(services)
        self.host_ports: dict[int, int] = dict(host_ports)
        self.parent_id = parent_id
        self.via = via
        self.branchable = spec.branchable
        self.argv: tuple[str, ...] = tuple(spec.argv or ())
        self.env: dict[str, str] = dict(spec.env)
        self.sql_log: list[str] = []
        self.inherited_sql: list[str] = []
        self.clock_sets: list[str] = []
        self.exec_log: list[tuple[str, ...]] = []
        self.files: dict[str, bytes] = {}
        self.sql_responses: dict[str, str] = {}
        self.ram_bytes = spec.memory_mb * 2**20
        self._state = "created"
        self._listeners: dict[int, _Listener] = {}

    # -- lifecycle -----------------------------------------------------------------

    def _bind(self) -> None:
        try:
            for guest, host in self.host_ports.items():
                self._listeners[guest] = _Listener(host)
        except BootError:
            self._close_listeners()
            raise
        self._state = "running"

    def _close_listeners(self) -> None:
        for listener in self._listeners.values():
            listener.close()
        self._listeners.clear()

    def accepted_connections(self, guest: int) -> int:
        """How many TCP connections the listener for ``guest`` has accepted."""
        listener = self._listeners.get(guest)
        return listener.accepted if listener else 0

    def state(self) -> str:
        return self._state

    def ready(self) -> bool:
        return self._state == "running"

    def wait_until_ready(self, timeout_s: float, interval_s: float) -> None:
        deadline = time.monotonic() + timeout_s
        while not self.ready():
            if time.monotonic() >= deadline:
                raise ReadinessTimeout(
                    f"{self.name} not ready after {timeout_s}s", timeout_s=timeout_s
                )
            time.sleep(interval_s)

    def pause(self) -> None:
        self.engine.record("pause", machine=self.id)
        self._state = "paused"

    def resume(self) -> None:
        self.engine.record("resume", machine=self.id)
        self._state = "running"

    def stop(self) -> None:
        self.engine.record("stop", machine=self.id)
        self._state = "stopped"
        self._close_listeners()

    def delete(self) -> None:
        self.engine.record("delete", machine=self.id)
        self._state = "deleted"
        self._close_listeners()
        self.engine.live_machines.discard(self)

    def __hash__(self) -> int:
        return hash(self.id)

    def __repr__(self) -> str:
        return f"FakeMachine({self.id!r}, {self.name!r}, {self._state}, ports={self.host_ports})"

    # -- guest operations ----------------------------------------------------------

    def exec(
        self,
        argv: Sequence[str],
        *,
        timeout_s: float | None = None,
        user: str | None = None,
    ) -> ExecOutcome:
        argv = tuple(argv)
        self.engine.record("exec", machine=self.id, argv=argv, timeout_s=timeout_s, user=user)
        self.exec_log.append(argv)
        if self._state != "running":
            raise SmoltestError(f"machine {self.name} is {self._state}")
        if not argv:
            return ExecOutcome(127, stderr=b"empty command", argv=argv)
        handler = self.engine.exec_handlers.get(argv[0], self.engine.exec_handlers.get("*"))
        if handler is not None:
            custom = handler(self, argv)
            if custom is not None:
                return custom
        return self._dispatch(argv)

    def _dispatch(self, argv: tuple[str, ...]) -> ExecOutcome:
        builtin = self._BUILTINS.get(argv[0])
        if builtin is None:
            return ExecOutcome(127, b"", f"sh: {argv[0]}: not found\n".encode(), argv=argv)
        return builtin(self, argv)

    def _pg_isready(self, argv: tuple[str, ...]) -> ExecOutcome:
        if "postgres" in self.services:
            out = b"/var/run/postgresql:5432 - accepting connections\n"
            return ExecOutcome(0, out, argv=argv)
        return ExecOutcome(2, b"", b"no response\n", argv=argv)

    def _date(self, argv: tuple[str, ...]) -> ExecOutcome:
        if len(argv) >= 3 and argv[1] == "-s":
            self.clock_sets.append(argv[2])
            return ExecOutcome(0, argv[2].encode() + b"\n", argv=argv)
        return ExecOutcome(0, str(int(time.time())).encode() + b"\n", argv=argv)

    def _cat(self, argv: tuple[str, ...]) -> ExecOutcome:
        if len(argv) >= 2 and argv[1] in self.files:
            return ExecOutcome(0, self.files[argv[1]], argv=argv)
        name = argv[1] if len(argv) > 1 else ""
        err = f"cat: {name}: No such file or directory\n".encode()
        return ExecOutcome(1, b"", err, argv=argv)

    def _true(self, argv: tuple[str, ...]) -> ExecOutcome:
        return ExecOutcome(0, argv=argv)

    def _psql(self, argv: tuple[str, ...]) -> ExecOutcome:
        if "postgres" not in self.services:
            return ExecOutcome(2, b"", b"psql: error: connection refused\n", argv=argv)
        statements = [argv[i + 1] for i, a in enumerate(argv[:-1]) if a in ("-c", "--command")]
        for i, a in enumerate(argv[:-1]):
            if a in ("-f", "--file"):
                script = self.files.get(argv[i + 1])
                if script is None:
                    err = f"psql: error: {argv[i + 1]}: No such file or directory\n".encode()
                    return ExecOutcome(1, b"", err, argv=argv)
                statements.extend(_split_sql(script.decode("utf-8", "replace")))
        if not statements:
            return ExecOutcome(1, b"", b"psql: no -c or -f given to the fake\n", argv=argv)
        out: list[str] = []
        for sql in statements:
            self.sql_log.append(sql)
            self.engine.sql_log.append((self.id, sql))
            if self.engine.sql_error is not None and self.engine.sql_error.search(sql):
                return ExecOutcome(1, b"", f"ERROR:  {sql}\n".encode(), argv=argv)
            out.append(self.sql_response(sql))
        return ExecOutcome(0, ("\n".join(out) + "\n").encode(), argv=argv)

    _BUILTINS: ClassVar[dict[str, Callable[[FakeMachine, tuple[str, ...]], ExecOutcome]]] = {
        "pg_isready": _pg_isready,
        "psql": _psql,
        "date": _date,
        "cat": _cat,
        "true": _true,
    }

    def sql_response(self, sql: str) -> str:
        """Canned stdout for ``sql``: machine, then engine responses, then the default."""
        key = sql.strip().rstrip(";").lower()
        for table in (self.sql_responses, self.engine.sql_responses):
            if sql in table:
                return table[sql]
            for pattern, value in table.items():
                if key.startswith(pattern.strip().rstrip(";").lower()):
                    return value
        if key.startswith("show "):
            return self.engine.show_values.get(key[5:].strip(), "on")
        if key.startswith("select 1"):
            return "1"
        return self.engine.default_sql_response

    def read_file(self, path: str) -> bytes:
        self.engine.record("read_file", machine=self.id, path=path)
        try:
            return self.files[path]
        except KeyError:
            raise FileNotFoundError(path) from None

    def write_file(self, path: str, data: bytes, mode: int | None = None) -> None:
        self.engine.record("write_file", machine=self.id, path=path, mode=mode)
        self.files[path] = bytes(data)

    def host_port(self, guest: int) -> tuple[str, int]:
        if self.target == "cloud":
            raise NotSupportedError("cloud machines publish no host port; open a tunnel")
        try:
            return LOOPBACK, self.host_ports[guest]
        except KeyError:
            raise SmoltestError(f"guest port {guest} is not published") from None

    # -- branch / checkpoint -------------------------------------------------------

    def branch(self, name: str, ports: Sequence[PortMapping] | None = None) -> MachineHandle:
        self.engine.tick("branch", machine=self.id, name=name, ports=ports)
        if not self.engine.supports_branch(self.target):
            raise NotSupportedError(f"branching is not supported on {self.target}")
        if not self.branchable:
            raise NotSupportedError(f"{self.name} was not created with branchable=True")
        if self._state != "running":
            raise SmoltestError(f"cannot branch {self.name} while {self._state}")
        requested = {p.guest: p.host for p in ports} if ports is not None else {}
        unknown = set(requested) - set(self.host_ports)
        if unknown:
            raise InvalidConfig(f"branch ports {sorted(unknown)} are not published by the source")
        new_ports: dict[int, int] = {}
        for guest in self.host_ports:
            host = requested.get(guest)
            new_ports[guest] = (
                host if host is not None else pick_free_port(exclude=new_ports.values())
            )
        spec = MachineSpec(
            image=self.spec.image,
            argv=self.argv,
            env=self.env,
            ports=tuple(PortMapping(h, g) for g, h in new_ports.items()),
            cpus=self.spec.cpus,
            memory_mb=self.spec.memory_mb,
            storage_gb=self.spec.storage_gb,
            network=self.spec.network,
            branchable=True,  # like the real engine: children can be checkpointed too
            name=name,
            auto_stop_seconds=self.spec.auto_stop_seconds,
            ttl_seconds=self.spec.ttl_seconds,
        )
        child = self.engine.new_machine(
            spec,
            self.target,
            name,
            services=self.services,
            host_ports=new_ports,
            parent_id=self.id,
            via="branch",
        )
        child.files = dict(self.files)
        child.sql_responses = dict(self.sql_responses)
        child.inherited_sql = list(self.sql_log)
        child._bind()
        self.engine.live_machines.add(child)
        return child

    def shape(self) -> dict[str, Any]:
        """The JSON-serialisable shape a fake checkpoint records."""
        return {
            "format": SHAPE_FORMAT,
            "image": self.spec.image,
            "argv": list(self.argv),
            "env": dict(self.env),
            "ports": [[host, guest] for guest, host in self.host_ports.items()],
            "services": sorted(self.services),
            "ram_bytes": self.ram_bytes,
            "cpus": self.spec.cpus,
            "memory_mb": self.spec.memory_mb,
            "storage_gb": self.spec.storage_gb,
            "network": self.spec.network,
            "target": self.target,
            "source": self.id,
            "sql_log": list(self.inherited_sql) + list(self.sql_log),
            "files": {k: v.decode("utf-8", "replace") for k, v in self.files.items()},
        }

    def checkpoint(self, output: str | None = None, store: str | None = None) -> CheckpointInfo:
        self.engine.tick("checkpoint", machine=self.id, output=output, store=store)
        if not self.engine.supports_checkpoints(self.target):
            raise NotSupportedError(f"checkpoints are not supported on {self.target}")
        if not self.branchable:
            raise NotSupportedError(f"{self.name} was not created with branchable=True")
        if self._state != "running":
            raise SmoltestError(f"cannot checkpoint {self.name} while {self._state}")
        shape = self.shape()
        payload = json.dumps(shape, sort_keys=True, indent=1).encode()
        if self.target == "cloud":
            ckpt_id = f"ckpt-{next(self.engine.checkpoint_ids)}"
            self.engine.cloud_checkpoints[ckpt_id] = shape
            return CheckpointInfo(
                CheckpointRef("cloud", ckpt_id), size_bytes=len(payload), machine_id=self.id
            )
        if output is None:
            raise InvalidConfig("local checkpoints require an output path")
        out = Path(output)
        if out.exists():
            raise InvalidConfig(f"checkpoint output already exists: {out}")
        out.parent.mkdir(parents=True, exist_ok=True)
        if store is not None:
            store_dir = Path(store)
            store_dir.mkdir(parents=True, exist_ok=True)
            digest = hashlib.sha256(payload).hexdigest()[:24]
            chunk = store_dir / f"{digest}.chunk"
            if not chunk.exists():
                chunk.write_bytes(payload)
            shape = {**shape, "chunks": [chunk.name], "store": str(store_dir)}
            payload = json.dumps(shape, sort_keys=True, indent=1).encode()
            self.engine.store_refs.setdefault(str(store_dir), {}).setdefault(chunk.name, set()).add(
                str(out)
            )
        out.write_bytes(payload)
        return CheckpointInfo(
            CheckpointRef("file", str(out)),
            size_bytes=len(payload),
            store=store,
            machine_id=self.id,
        )


ExecHandler = Callable[[FakeMachine, tuple[str, ...]], "ExecOutcome | None"]


class FakeEngine:
    """An in-process :class:`~smoltest.transport.base.Engine`.

    Knobs: ``availability`` (what :meth:`local_availability` reports),
    ``supports_checkpoints`` / ``supports_branch`` flags, ``latency`` (seconds to
    sleep per operation name), :meth:`fail_next` fault injection, ``show_values`` /
    ``sql_responses`` for canned ``psql`` output, ``exec_handlers`` for custom
    programs and ``log_text`` returned for ``cat <log_path>``. Observe behaviour
    through ``calls``, ``sql_log`` and ``live_machines``.
    """

    _default: ClassVar[FakeEngine | None] = None

    def __init__(
        self,
        *,
        availability: tuple[bool, str | None, str | None] = (True, None, None),
        supports_checkpoints: bool = True,
        supports_branch: bool = True,
        latency: Mapping[str, float] | None = None,
        sdk_version: str = "fake-1.22.2",
        restored_branchable: bool = True,
        log_path: str = DEFAULT_LOG_PATH,
        log_text: str = "database system is ready to accept connections\n",
    ) -> None:
        self.availability = availability
        self._supports_checkpoints = supports_checkpoints
        self._supports_branch = supports_branch
        self.latency: dict[str, float] = dict(latency or {})
        self._sdk_version = sdk_version
        self.restored_branchable = restored_branchable
        self.log_path = log_path
        self.log_text = log_text
        self.calls: list[tuple[str, dict[str, Any]]] = []
        self.sql_log: list[tuple[str, str]] = []
        self.live_machines: set[FakeMachine] = set()
        self.machines: dict[str, FakeMachine] = {}
        self.cloud_checkpoints: dict[str, dict[str, Any]] = {}
        self.store_refs: dict[str, dict[str, set[str]]] = {}
        self.show_values: dict[str, str] = {"fsync": "off", "synchronous_commit": "off"}
        self.sql_responses: dict[str, str] = {}
        self.default_sql_response = ""
        self.sql_error: Any = None
        self.exec_handlers: dict[str, ExecHandler] = {}
        self.tunnel_bridge_factory: Callable[[Callable[[], FakeAsyncTunnel]], Bridge] | None = None
        self._fail_next: dict[str, list[BaseException]] = {}
        self._ids = itertools.count(1)
        self.checkpoint_ids = itertools.count(1)
        self._lock = threading.Lock()

    # -- selection through SMOLTEST_ENGINE ----------------------------------------

    @classmethod
    def factory(cls) -> FakeEngine:
        """Return the engine installed with :meth:`set_default`, else a new one.

        Point ``SMOLTEST_ENGINE`` at ``smoltest.testing:FakeEngine.factory`` so code
        that calls ``get_engine()`` shares the instance a test is inspecting.
        """
        return cls._default if cls._default is not None else cls()

    @classmethod
    def set_default(cls, engine: FakeEngine | None) -> None:
        """Install (or clear) the engine :meth:`factory` returns."""
        cls._default = engine

    # -- instrumentation -----------------------------------------------------------

    def record(self, op: str, **details: Any) -> None:
        """Append ``(op, details)`` to :attr:`calls`."""
        self.calls.append((op, details))

    def fail_next(self, op: str, exc: BaseException) -> None:
        """Make the next ``op`` (``create``, ``restore``, ``branch``, ...) raise ``exc``."""
        self._fail_next.setdefault(op, []).append(exc)

    def tick(self, op: str, **details: Any) -> None:
        """Record ``op``, sleep its configured latency and raise any injected fault."""
        self.record(op, **details)
        delay = self.latency.get(op, 0.0)
        if delay:
            time.sleep(delay)
        with self._lock:
            queue = self._fail_next.get(op)
            exc = queue.pop(0) if queue else None
        if exc is not None:
            raise exc

    def ops(self, name: str) -> list[dict[str, Any]]:
        """Details of every recorded call named ``name``."""
        return [details for op, details in self.calls if op == name]

    # -- Engine protocol -----------------------------------------------------------

    def sdk_version(self) -> str:
        return self._sdk_version

    def local_availability(self) -> tuple[bool, str | None, str | None]:
        return self.availability

    def supports_checkpoints(self, target: Target) -> bool:
        return self._supports_checkpoints

    def supports_branch(self, target: Target) -> bool:
        return self._supports_branch

    def new_machine(
        self,
        spec: MachineSpec,
        target: Target,
        name: str | None,
        *,
        services: set[str],
        host_ports: dict[int, int],
        parent_id: str | None = None,
        via: str = "create",
    ) -> FakeMachine:
        """Allocate an unbound :class:`FakeMachine` and register it in :attr:`machines`."""
        machine_id = f"fake-{next(self._ids):04d}"
        machine = FakeMachine(
            self,
            machine_id,
            name or machine_id,
            spec,
            target,
            services=services,
            host_ports=host_ports,
            parent_id=parent_id,
            via=via,
        )
        machine.files[self.log_path] = self.log_text.encode()
        self.machines[machine_id] = machine
        return machine

    def create(self, spec: MachineSpec, target: Target) -> MachineHandle:
        self.tick("create", spec=spec, target=target)
        host_ports: dict[int, int] = {}
        for mapping in spec.ports:
            if mapping.guest in host_ports:
                raise InvalidConfig(f"guest port {mapping.guest} published twice")
            host = mapping.host
            host_ports[mapping.guest] = (
                host if host is not None else pick_free_port(exclude=host_ports.values())
            )
        services = {"postgres"} if "postgres" in spec.image else set()
        machine = self.new_machine(
            spec, target, spec.name, services=services, host_ports=host_ports
        )
        machine._bind()
        self.live_machines.add(machine)
        return machine

    def connect(self, machine_id: str, target: Target) -> MachineHandle:
        self.tick("connect", machine_id=machine_id, target=target)
        machine = self.machines.get(machine_id)
        if machine is None or machine not in self.live_machines:
            raise SmoltestError(f"no such machine: {machine_id}")
        return machine

    def _load_shape(self, ref: CheckpointRef) -> dict[str, Any]:
        if ref.kind == "cloud":
            try:
                return self.cloud_checkpoints[ref.locator]
            except KeyError:
                raise SmoltestError(f"unknown cloud checkpoint {ref.locator}") from None
        path = Path(ref.locator)
        try:
            shape: dict[str, Any] = json.loads(path.read_bytes())
        except (OSError, ValueError) as exc:
            raise BootError(f"unreadable checkpoint {path}", stage="restore", cause=exc) from exc
        if shape.get("format") != SHAPE_FORMAT:
            raise BootError(f"unsupported checkpoint format in {path}", stage="restore")
        return shape

    def restore_checkpoint(self, ref: CheckpointRef, name: str, target: Target) -> MachineHandle:
        self.tick("restore", ref=ref, name=name, target=target)
        shape = self._load_shape(ref)
        # Local checkpoints preserve the published host port (the real collision
        # smoltest's port variants exist for); the cloud assigns fresh ones.
        host_ports = {int(guest): int(host) for host, guest in shape["ports"]}
        if target == "cloud":
            host_ports = {guest: pick_free_port() for guest in host_ports}
        spec = MachineSpec(
            image=shape["image"],
            argv=tuple(shape["argv"]),
            env=shape["env"],
            ports=tuple(PortMapping(h, g) for g, h in host_ports.items()),
            cpus=shape["cpus"],
            memory_mb=shape["memory_mb"],
            storage_gb=shape["storage_gb"],
            network=shape["network"],
            branchable=self.restored_branchable,
            name=name,
        )
        machine = self.new_machine(
            spec,
            target,
            name,
            services=set(shape["services"]),
            host_ports=host_ports,
            via="restore",
        )
        machine.inherited_sql = list(shape.get("sql_log", []))
        machine.files.update({k: v.encode() for k, v in shape.get("files", {}).items()})
        try:
            machine._bind()
        except BootError as exc:
            raise BootError(str(exc), stage="restore", cause=exc.cause) from exc.cause
        self.live_machines.add(machine)
        return machine

    def export_checkpoint(self, source: str, output: str) -> int:
        self.tick("export", source=source, output=output)
        src, dst = Path(source), Path(output)
        if not src.is_file():
            raise SmoltestError(f"no such checkpoint: {src}")
        dst.parent.mkdir(parents=True, exist_ok=True)
        shutil.copyfile(src, dst)
        return dst.stat().st_size

    def prune_checkpoint_store(self, store: str) -> int:
        self.tick("prune_store", store=store)
        store_dir = Path(store)
        refs = self.store_refs.get(str(store_dir), {})
        removed = 0
        for chunk in sorted(store_dir.glob("*.chunk")) if store_dir.is_dir() else []:
            holders = {p for p in refs.get(chunk.name, set()) if Path(p).exists()}
            if not holders:
                chunk.unlink()
                refs.pop(chunk.name, None)
                removed += 1
            else:
                refs[chunk.name] = holders
        return removed

    def open_tunnel(self, machine_id: str, guest_port: int, target: Target) -> Bridge:
        self.tick("open_tunnel", machine_id=machine_id, guest_port=guest_port, target=target)
        machine = self.machines.get(machine_id)
        if machine is None:
            raise SmoltestError(f"no such machine: {machine_id}")
        if guest_port not in machine.host_ports:
            raise SmoltestError(f"guest port {guest_port} is not published by {machine.name}")
        if target == "local":
            return StaticBridge(HostEndpoint(LOOPBACK, machine.host_ports[guest_port]))

        def opener() -> FakeAsyncTunnel:
            return FakeAsyncTunnel(machine, guest_port)

        if self.tunnel_bridge_factory is not None:
            return self.tunnel_bridge_factory(opener)
        return RelayBridge(opener)

    # -- test helpers --------------------------------------------------------------

    def close_all(self) -> None:
        """Delete every live machine (teardown safety net)."""
        for machine in list(self.live_machines):
            machine.delete()


__all__ = [
    "DEFAULT_LOG_PATH",
    "FakeAsyncTunnel",
    "FakeEngine",
    "FakeMachine",
    "RelayBridge",
]
