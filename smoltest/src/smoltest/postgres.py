"""The PostgreSQL facade: a testcontainers-compatible surface over the boot ladder.

:class:`PostgresMachine` (alias :class:`PostgresContainer`) is what users hold.
It builds a :class:`~smoltest.boot.spec.PostgresSpec`, hands it to
:func:`smoltest.boot.strategy.boot_postgres` on :meth:`~PostgresMachine.start`
and wraps the returned :class:`~smoltest.boot.strategy.BootResult`. This module
never imports the Smol SDK.
"""

from __future__ import annotations

import asyncio
import dataclasses
import hashlib
import itertools
import os
import shlex
import threading
from collections.abc import Callable, Iterator, Sequence
from dataclasses import dataclass
from datetime import timedelta
from pathlib import Path
from typing import Any
from urllib.parse import quote

from ._lifecycle import Finalizer, get_reaper
from ._log import logger, warn_once
from ._ports import HostEndpoint
from .boot import strategy as _boot
from .boot.spec import LOG_PATH, PostgresSpec
from .boot.strategy import BootInfo, BootResult
from .config import Settings
from .errors import InvalidConfig, NotSupportedError, SmoltestError
from .transport import get_engine
from .transport.base import CheckpointInfo, Engine, ExecOutcome, MachineHandle, PortMapping
from .wait.strategies import EXEC_PROBE_TIMEOUT_S, WaitStrategy, default_wait

SEED_DIR = "/tmp"
"""Guest directory SQL seeds are written to before ``psql -f`` runs them."""


class _DefaultDriver:
    """Sentinel: ``get_connection_url(driver=_DEFAULT)`` uses the constructor's driver."""

    __slots__ = ()

    def __repr__(self) -> str:
        return "<constructor driver>"


_DEFAULT = _DefaultDriver()


@dataclass(frozen=True)
class ExecResult:
    """Outcome of :meth:`PostgresMachine.exec`.

    Unpacks and indexes like testcontainers' ``(exit_code, output)`` tuple, where
    ``output`` is stdout followed by stderr.
    """

    exit_code: int
    stdout: bytes = b""
    stderr: bytes = b""
    argv: tuple[str, ...] = ()

    @classmethod
    def from_outcome(cls, outcome: ExecOutcome) -> ExecResult:
        """Wrap an engine :class:`~smoltest.transport.base.ExecOutcome`."""
        return cls(outcome.exit_code, outcome.stdout, outcome.stderr, tuple(outcome.argv))

    @property
    def output(self) -> bytes:
        """``stdout`` followed by ``stderr``."""
        return self.stdout + self.stderr

    @property
    def ok(self) -> bool:
        """``True`` when the command exited with status 0."""
        return self.exit_code == 0

    @property
    def text(self) -> str:
        """``stdout`` decoded as UTF-8 with replacement."""
        return self.stdout.decode("utf-8", "replace")

    @property
    def stderr_text(self) -> str:
        """``stderr`` decoded as UTF-8 with replacement."""
        return self.stderr.decode("utf-8", "replace")

    def __iter__(self) -> Iterator[Any]:
        yield self.exit_code
        yield self.output

    def __len__(self) -> int:
        return 2

    def __getitem__(self, index: int) -> Any:
        return (self.exit_code, self.output)[index]


SeedFn = Callable[["PostgresMachine"], None]
"""A seeding step: receives the ready machine and populates it."""


def _sha256(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def apply_sql_script(machine: PostgresMachine, sql: bytes, *, label: str = "seed") -> None:
    """Write ``sql`` into the guest and run it with ``psql -v ON_ERROR_STOP=1 -f``.

    Runs over the trusted unix socket inside the guest, so no host connection is
    opened and a checkpoint taken afterwards sees no client.
    """
    handle = machine.get_machine()
    path = f"{SEED_DIR}/smoltest-{_sha256(sql)[:16]}.sql"
    handle.write_file(path, sql, 0o600)
    argv = [
        "psql",
        "-v",
        "ON_ERROR_STOP=1",
        "-U",
        machine.username,
        "-d",
        machine.dbname,
        "-f",
        path,
    ]
    outcome = handle.exec(argv)
    if not outcome.ok:
        detail = outcome.stderr_text.strip() or outcome.text.strip()
        raise SmoltestError(
            f"{label} failed: psql exited {outcome.exit_code}: {detail}", code="SEED_FAILED"
        )


@dataclass(frozen=True)
class Seed:
    """Deterministic database population, identified by ``key`` for the checkpoint cache.

    Two seeds with the same key are assumed to produce the same database, so a
    checkpoint taken after one is reused for the other.
    """

    key: str
    apply: SeedFn

    @classmethod
    def from_sql(cls, sql: str, key: str | None = None) -> Seed:
        """Run ``sql`` through in-guest ``psql``; ``key`` defaults to its SHA-256."""
        data = sql.encode("utf-8")
        seed_key = key or _sha256(data)

        def apply(machine: PostgresMachine) -> None:
            apply_sql_script(machine, data, label=f"seed {seed_key[:12]}")

        return cls(seed_key, apply)

    @classmethod
    def from_sql_files(cls, *paths: str | os.PathLike[str], key: str | None = None) -> Seed:
        """Run the files in order; ``key`` defaults to the SHA-256 of their joined contents."""
        if not paths:
            raise InvalidConfig("Seed.from_sql_files needs at least one path")
        data = b"\n".join(Path(p).read_bytes() for p in paths)
        seed_key = key or _sha256(data)
        names = ", ".join(Path(p).name for p in paths)

        def apply(machine: PostgresMachine) -> None:
            apply_sql_script(machine, data, label=f"seed {names}")

        return cls(seed_key, apply)

    @classmethod
    def from_callable(cls, fn: SeedFn, key: str) -> Seed:
        """Seed with arbitrary code (migrations, fixtures); ``key`` must change when it does."""
        if not key:
            raise InvalidConfig("Seed.from_callable needs a non-empty key")
        return cls(key, fn)


def _env_default(var: str, fallback: str) -> str:
    return os.environ.get(var) or fallback


class PostgresMachine:
    """A PostgreSQL server in a Smol microVM, with testcontainers' ``PostgresContainer`` API.

    Credentials default to ``POSTGRES_USER`` / ``POSTGRES_PASSWORD`` / ``POSTGRES_DB``
    from the environment, else ``test``. Docker-only keyword arguments are
    accepted, warned about once and ignored.
    """

    def __init__(
        self,
        image: str | None = None,
        port: int = 5432,
        username: str | None = None,
        password: str | None = None,
        dbname: str | None = None,
        driver: str | None = "psycopg2",
        *,
        settings: Settings | None = None,
        target: str | None = None,
        cpus: int | None = None,
        memory_mb: int | None = None,
        cache: bool | None = None,
        fast: bool | None = None,
        wait: WaitStrategy | None = None,
        name: str | None = None,
        seed: Seed | None = None,
        capture_logs: bool = False,
        engine: Engine | None = None,
        **kwargs: Any,
    ) -> None:
        base = settings if settings is not None else Settings.from_env()
        self._settings = base.replace(
            target=target,
            disable_cache=None if cache is None else not cache,
        )
        self._spec = PostgresSpec(
            image=image,
            username=username or _env_default("POSTGRES_USER", "test"),
            password=password or _env_default("POSTGRES_PASSWORD", "test"),
            dbname=dbname or _env_default("POSTGRES_DB", "test"),
            guest_port=port,
            fast=fast,
            capture_logs=capture_logs,
            cpus=cpus,
            memory_mb=memory_mb,
        )
        self._driver = driver
        self._wait = wait
        self._name = name
        self._seed = seed
        self._engine = engine
        self._host_port: int | None = None
        self._result: BootResult | None = None
        self._finalizer: Finalizer | None = None
        self._parent: PostgresMachine | None = None
        self._halted = False
        self._branch_ids = itertools.count(1)
        self._lock = threading.RLock()
        for key in kwargs:
            self._ignore_kwarg(key)

    # -- construction helpers --------------------------------------------------------

    @classmethod
    def _from_boot(
        cls,
        result: BootResult,
        spec: PostgresSpec,
        settings: Settings,
        engine: Engine,
        parent: PostgresMachine | None = None,
        *,
        driver: str | None = "psycopg2",
        register_finalizer: bool = True,
    ) -> PostgresMachine:
        """Wrap an already booted machine (used by ``branch`` and the pytest plugin).

        ``register_finalizer=False`` builds a facade that does not own the machine,
        e.g. the one ``boot_postgres`` hands to ``Seed.apply`` mid-boot: collecting
        it never tears the machine down.
        """
        machine = cls(
            image=spec.image,
            port=spec.guest_port,
            username=spec.username,
            password=spec.password,
            dbname=spec.dbname,
            driver=driver,
            settings=settings,
            engine=engine,
        )
        machine._spec = spec
        machine._attach(result, parent, register_finalizer=register_finalizer)
        return machine

    @staticmethod
    def _ignore_kwarg(key: str) -> None:
        warn_once(
            f"postgres.kwarg.{key}",
            f"PostgresMachine ignores the Docker-only argument {key!r}",
            stacklevel=3,
        )

    def _attach(
        self, result: BootResult, parent: PostgresMachine | None, *, register_finalizer: bool = True
    ) -> None:
        self._result = result
        self._parent = parent
        self._halted = False
        if register_finalizer:
            # The finalizer must not reference ``self`` or it would never be collected.
            self._finalizer = get_reaper().register(self, result.release, parent=parent)
        logger.info(
            "postgres %s ready via %s in %.2fs at %s",
            result.handle.name,
            result.info.via,
            result.info.elapsed_s,
            result.endpoint,
        )

    def _require(self) -> BootResult:
        if self._result is None:
            raise SmoltestError("the machine is not running; call start() first")
        return self._result

    def _unstarted(self) -> None:
        if self._result is not None:
            raise SmoltestError("configure the machine before start()")

    def _update_spec(self, **changes: Any) -> PostgresMachine:
        self._unstarted()
        self._spec = dataclasses.replace(self._spec, **changes)
        return self

    # -- builders (testcontainers style, return self) ---------------------------------

    def with_env(self, key: str, value: str) -> PostgresMachine:
        """Set one guest environment variable."""
        return self._update_spec(env={**self._spec.env, key: value})

    def with_envs(self, **envs: str) -> PostgresMachine:
        """Set several guest environment variables."""
        return self._update_spec(env={**self._spec.env, **envs})

    def with_command(self, command: str | Sequence[str]) -> PostgresMachine:
        """Replace the workload argv (``docker-entrypoint.sh postgres ...`` by default)."""
        argv = tuple(shlex.split(command)) if isinstance(command, str) else tuple(command)
        if not argv:
            raise InvalidConfig("with_command needs a non-empty command")
        return self._update_spec(command=argv)

    def with_name(self, name: str) -> PostgresMachine:
        """Name the machine (shown by the engine and in logs)."""
        self._unstarted()
        self._name = name
        return self

    def with_exposed_ports(self, *ports: int) -> PostgresMachine:
        """Publish extra guest ports on engine-chosen host ports."""
        extra = list(self._spec.extra_ports)
        known = {p.guest for p in extra} | {self._spec.guest_port}
        for guest in ports:
            if int(guest) not in known:
                extra.append(PortMapping(None, int(guest)))
                known.add(int(guest))
        return self._update_spec(extra_ports=tuple(extra))

    def with_bind_ports(self, container: int, host: int | None = None) -> PostgresMachine:
        """Publish guest port ``container`` on a fixed ``host`` port (``None`` = any)."""
        self._unstarted()
        if int(container) == self._spec.guest_port:
            self._host_port = host
            return self
        extra = [p for p in self._spec.extra_ports if p.guest != int(container)]
        extra.append(PortMapping(host, int(container)))
        return self._update_spec(extra_ports=tuple(extra))

    def with_volume_mapping(self, host: str, container: str, mode: str = "ro") -> PostgresMachine:
        """Unsupported: a microVM has no host mounts; load SQL with :meth:`with_init_sql`."""
        raise NotSupportedError(
            f"volume mappings are not supported ({host} -> {container}, {mode}); "
            "use with_init_sql(*paths) or with_seed(Seed) to load data"
        )

    def with_kwargs(self, **kwargs: Any) -> PostgresMachine:
        """Accept Docker ``run`` keyword arguments for compatibility; warn and ignore them."""
        for key in kwargs:
            self._ignore_kwarg(key)
        return self

    def with_init_sql(self, *paths: str | os.PathLike[str]) -> PostgresMachine:
        """Seed the database from SQL files (replaces any earlier seed)."""
        return self.with_seed(Seed.from_sql_files(*paths))

    def with_seed(self, seed: Seed | None) -> PostgresMachine:
        """Seed the database with ``seed`` after it is ready (cached by ``seed.key``)."""
        self._unstarted()
        self._seed = seed
        return self

    def with_resources(
        self, cpus: int | None = None, memory_mb: int | None = None
    ) -> PostgresMachine:
        """Size the guest; ``None`` keeps the current value."""
        changes: dict[str, int] = {}
        if cpus is not None:
            changes["cpus"] = cpus
        if memory_mb is not None:
            changes["memory_mb"] = memory_mb
        return self._update_spec(**changes)

    def waiting_for(self, strategy: WaitStrategy) -> PostgresMachine:
        """Replace the readiness strategy (default: in-guest ``pg_isready``)."""
        self._unstarted()
        self._wait = strategy
        return self

    def with_startup_timeout(self, seconds: float | timedelta) -> PostgresMachine:
        """Bound how long boot and readiness may take (seconds or a ``timedelta``)."""
        self._unstarted()
        value = seconds.total_seconds() if isinstance(seconds, timedelta) else float(seconds)
        self._settings = self._settings.replace(ready_timeout_s=value)
        return self

    # -- lifecycle ---------------------------------------------------------------------

    def start(self) -> PostgresMachine:
        """Boot (branch, restore or cold) and wait until PostgreSQL accepts connections."""
        with self._lock:
            if self._result is not None:
                if self._halted:
                    raise SmoltestError("the machine was stopped; create a new PostgresMachine")
                return self
            wait = (
                self._wait
                if self._wait is not None
                else default_wait(self.username, port=self.port)
            )
            extra: dict[str, Any] = {}
            if self._name is not None:
                extra["name"] = self._name
            if self._host_port is not None:
                extra["host_port"] = self._host_port
            result = _boot.boot_postgres(
                self._spec,
                self._settings,
                self.engine,
                seed=self._seed,
                wait=wait,
                role="standalone",
                **extra,
            )
            self._attach(result, parent=None)
            return self

    def stop(self, force: bool = True, delete: bool = True) -> None:
        """Tear the machine down: close the bridge, delete it and release its cache claim.

        ``force`` exists for testcontainers compatibility (a microVM is always
        stopped hard). ``delete=False`` only stops the guest; it is deleted at exit.
        Calling ``stop`` twice is harmless.
        """
        with self._lock:
            result = self._result
            if result is None:
                return
            if not delete:
                if not self._halted:
                    result.bridge.close()
                    result.handle.stop()
                    self._halted = True
                return
            self._result = None
            self._halted = False
            self._parent = None
            token, self._finalizer = self._finalizer, None
        # Detach first so GC or exit cannot run the release a second time.
        if token is None or token.detach() is not None:
            result.release()

    def __enter__(self) -> PostgresMachine:
        return self.start()

    def __exit__(self, *exc: object) -> None:
        self.stop()

    def __repr__(self) -> str:
        state = "running" if self.is_running else ("stopped" if self._halted else "new")
        return f"PostgresMachine({self.image!r}, {self.username}@{self.dbname}, {state})"

    # -- connection details ------------------------------------------------------------

    def get_connection_url(
        self, host: str | None = None, driver: str | _DefaultDriver | None = _DEFAULT
    ) -> str:
        """SQLAlchemy-style URL: ``postgresql+<driver>://user:pw@host:port/db``.

        ``driver=None`` gives a plain ``postgresql://`` URL (libpq, psycopg3 and
        asyncpg accept it); ``host`` overrides the endpoint host.
        """
        endpoint = self._require().endpoint
        chosen = self._driver if isinstance(driver, _DefaultDriver) else driver
        dialect = "postgresql" if chosen is None else f"postgresql+{chosen}"
        password = quote(self.password, safe=" +")
        return (
            f"{dialect}://{self.username}:{password}@{host or endpoint.host}:{endpoint.port}"
            f"/{self.dbname}"
        )

    @property
    def url(self) -> str:
        """:meth:`get_connection_url` with the constructor's driver."""
        return self.get_connection_url()

    def get_container_host_ip(self) -> str:
        """Host address PostgreSQL is reachable at (``127.0.0.1`` on the local target)."""
        return self._require().endpoint.host

    def get_exposed_port(self, port: int | None = None) -> int:
        """Host port publishing guest ``port`` (default: the PostgreSQL port)."""
        result = self._require()
        if port is None or int(port) == self._spec.guest_port:
            return result.endpoint.port
        return result.handle.host_port(int(port))[1]

    @property
    def endpoint(self) -> HostEndpoint:
        """Where the host reaches PostgreSQL."""
        return self._require().endpoint

    # -- guest operations --------------------------------------------------------------

    def exec(
        self,
        command: str | Sequence[str],
        *,
        timeout_s: float | None = None,
        user: str | None = None,
    ) -> ExecResult:
        """Run a command in the guest; the result unpacks as ``(exit_code, output)``.

        ``timeout_s=None`` applies the engine default (``Settings.exec_timeout_s``
        for the Smol engine); the command is killed when the timeout expires.
        """
        argv = shlex.split(command) if isinstance(command, str) else list(command)
        if not argv:
            raise InvalidConfig("exec needs a non-empty command")
        outcome = self._require().handle.exec(argv, timeout_s=timeout_s, user=user)
        return ExecResult.from_outcome(outcome)

    def get_logs(self) -> tuple[bytes, bytes]:
        """``(stdout, stderr)`` of the server.

        A microVM workload has no stdout pipe, so this is empty (with one warning)
        unless the machine was created with ``capture_logs=True``, in which case the
        ``logging_collector`` file is read from the guest.
        """
        result = self._require()
        if not self._spec.capture_logs:
            warn_once(
                "postgres.get_logs",
                "PostgresMachine.get_logs() is empty unless capture_logs=True: a microVM "
                "workload has no stdout pipe; with capture_logs the server log is read "
                f"from {LOG_PATH} in the guest",
            )
            return b"", b""
        outcome = result.handle.exec(["cat", LOG_PATH], timeout_s=EXEC_PROBE_TIMEOUT_S)
        if not outcome.ok:
            logger.debug("no server log yet: %s", outcome.stderr_text.strip())
            return b"", b""
        return outcome.stdout, b""

    def get_wrapped_container(self) -> MachineHandle:
        """The engine handle (testcontainers name for :meth:`get_machine`)."""
        return self._require().handle

    def get_machine(self) -> MachineHandle:
        """The engine handle of the running machine."""
        return self._require().handle

    def psql(self, sql: str, *, dbname: str | None = None, timeout_s: float | None = None) -> str:
        """Run ``sql`` with in-guest ``psql -tA`` (trust auth over the unix socket).

        Returns stdout without its trailing newline; raises
        :class:`~smoltest.errors.SmoltestError` when psql exits non-zero.
        ``timeout_s=None`` applies the engine default (``Settings.exec_timeout_s``).
        """
        argv = ["psql", "-tA", "-U", self.username, "-d", dbname or self.dbname, "-c", sql]
        outcome = self._require().handle.exec(argv, timeout_s=timeout_s)
        if not outcome.ok:
            detail = outcome.stderr_text.strip() or outcome.text.strip()
            raise SmoltestError(f"psql exited {outcome.exit_code}: {detail}", code="PSQL_FAILED")
        text = outcome.text
        return text[:-1] if text.endswith("\n") else text

    # -- branch / checkpoint -----------------------------------------------------------

    def branch(self, name: str | None = None) -> PostgresMachine:
        """Fork a copy-on-write child with its own host port; the parent keeps running.

        The child holds a reference to its parent and is torn down before it at exit.
        """
        result = self._require()
        child_name = name or f"{result.handle.name}-b{next(self._branch_ids)}"
        child_result = _boot.branch_from(result, child_name, self._settings, self.engine)
        return PostgresMachine._from_boot(
            child_result,
            child_result.spec,
            self._settings,
            self.engine,
            parent=self,
            driver=self._driver,
        )

    def checkpoint(self, output: str | os.PathLike[str] | None = None) -> CheckpointInfo:
        """Snapshot the running machine (``output`` is required on the local target)."""
        target = None if output is None else os.fspath(output)
        return self._require().handle.checkpoint(output=target, store=None)

    # -- state -------------------------------------------------------------------------

    @property
    def boot_info(self) -> BootInfo | None:
        """How the machine was booted, once started."""
        return None if self._result is None else self._result.info

    @property
    def is_running(self) -> bool:
        """``True`` between a successful :meth:`start` and :meth:`stop`."""
        return self._result is not None and not self._halted

    @property
    def spec(self) -> PostgresSpec:
        """The immutable description the builders have produced."""
        return self._spec

    @property
    def settings(self) -> Settings:
        """Effective settings."""
        return self._settings

    @property
    def engine(self) -> Engine:
        """The engine, resolved from settings on first use."""
        if self._engine is None:
            self._engine = get_engine(self._settings)
        return self._engine

    @property
    def image(self) -> str:
        """The image that will be (or was) booted."""
        return self._spec.resolved_image(self._settings)

    @property
    def username(self) -> str:
        return self._spec.username

    @property
    def password(self) -> str:
        return self._spec.password

    @property
    def dbname(self) -> str:
        return self._spec.dbname

    @property
    def port(self) -> int:
        """The guest port PostgreSQL listens on."""
        return self._spec.guest_port

    @property
    def driver(self) -> str | None:
        """Default driver for :meth:`get_connection_url`."""
        return self._driver

    @property
    def name(self) -> str | None:
        """The machine name: the engine's once running, else the configured one."""
        return self._result.handle.name if self._result is not None else self._name

    @property
    def seed(self) -> Seed | None:
        return self._seed

    @property
    def pinned_host_port(self) -> int | None:
        """The host port ``with_bind_ports`` pinned for the PostgreSQL port, if any."""
        return self._host_port

    @property
    def wait_strategy(self) -> WaitStrategy | None:
        """The configured readiness strategy (``None`` = default ``pg_isready``)."""
        return self._wait

    @property
    def parent(self) -> PostgresMachine | None:
        """The machine this one was branched from, if any."""
        return self._parent


PostgresContainer = PostgresMachine
"""testcontainers' name for :class:`PostgresMachine`."""


class AsyncPostgresMachine:
    """``await``-able wrapper: blocking calls run in a worker thread via ``asyncio.to_thread``.

    Configure through the constructor or the builders on :attr:`sync`.
    """

    def __init__(
        self,
        image: str | None = None,
        port: int = 5432,
        username: str | None = None,
        password: str | None = None,
        dbname: str | None = None,
        driver: str | None = "psycopg2",
        *,
        settings: Settings | None = None,
        target: str | None = None,
        cpus: int | None = None,
        memory_mb: int | None = None,
        cache: bool | None = None,
        fast: bool | None = None,
        wait: WaitStrategy | None = None,
        name: str | None = None,
        seed: Seed | None = None,
        capture_logs: bool = False,
        engine: Engine | None = None,
        **kwargs: Any,
    ) -> None:
        self._sync = PostgresMachine(
            image,
            port,
            username,
            password,
            dbname,
            driver,
            settings=settings,
            target=target,
            cpus=cpus,
            memory_mb=memory_mb,
            cache=cache,
            fast=fast,
            wait=wait,
            name=name,
            seed=seed,
            capture_logs=capture_logs,
            engine=engine,
            **kwargs,
        )

    @classmethod
    def wrap(cls, machine: PostgresMachine) -> AsyncPostgresMachine:
        """Wrap an existing (possibly running) :class:`PostgresMachine`."""
        wrapper = cls.__new__(cls)
        wrapper._sync = machine
        return wrapper

    @property
    def sync(self) -> PostgresMachine:
        """The underlying synchronous machine."""
        return self._sync

    async def start(self) -> AsyncPostgresMachine:
        await asyncio.to_thread(self._sync.start)
        return self

    async def stop(self, force: bool = True, delete: bool = True) -> None:
        await asyncio.to_thread(self._sync.stop, force, delete)

    async def exec(
        self,
        command: str | Sequence[str],
        *,
        timeout_s: float | None = None,
        user: str | None = None,
    ) -> ExecResult:
        return await asyncio.to_thread(self._sync.exec, command, timeout_s=timeout_s, user=user)

    async def psql(self, sql: str, *, dbname: str | None = None) -> str:
        return await asyncio.to_thread(self._sync.psql, sql, dbname=dbname)

    async def branch(self, name: str | None = None) -> AsyncPostgresMachine:
        child = await asyncio.to_thread(self._sync.branch, name)
        return AsyncPostgresMachine.wrap(child)

    async def checkpoint(self, output: str | os.PathLike[str] | None = None) -> CheckpointInfo:
        return await asyncio.to_thread(self._sync.checkpoint, output)

    async def __aenter__(self) -> AsyncPostgresMachine:
        return await self.start()

    async def __aexit__(self, *exc: object) -> None:
        await self.stop()

    def get_connection_url(
        self, host: str | None = None, driver: str | _DefaultDriver | None = _DEFAULT
    ) -> str:
        return self._sync.get_connection_url(host, driver)

    def get_container_host_ip(self) -> str:
        return self._sync.get_container_host_ip()

    def get_exposed_port(self, port: int | None = None) -> int:
        return self._sync.get_exposed_port(port)

    @property
    def url(self) -> str:
        return self._sync.url

    @property
    def boot_info(self) -> BootInfo | None:
        return self._sync.boot_info

    @property
    def is_running(self) -> bool:
        return self._sync.is_running

    def __repr__(self) -> str:
        return f"Async{self._sync!r}"


__all__ = [
    "SEED_DIR",
    "AsyncPostgresMachine",
    "ExecResult",
    "PostgresContainer",
    "PostgresMachine",
    "Seed",
    "SeedFn",
    "apply_sql_script",
]
