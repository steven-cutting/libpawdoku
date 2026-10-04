"""Readiness strategies: how smoltest decides a booted PostgreSQL machine is usable.

Every strategy polls through the :class:`~smoltest.transport.base.MachineHandle`
protocol (in-guest ``exec``) or the host endpoint of the published port, so the
same code runs against the Smol SDK and against :class:`smoltest.testing.FakeEngine`.
The names and builder methods mirror ``testcontainers.core.wait_strategies``.
"""

from __future__ import annotations

import asyncio
import importlib
import math
import re
import shlex
import socket
import time
from collections.abc import Callable, Sequence
from dataclasses import dataclass
from datetime import timedelta
from typing import Any, Protocol, TypeVar
from urllib.parse import quote

from .._log import logger
from .._ports import HostEndpoint
from ..errors import InvalidConfig, NotSupportedError, ReadinessTimeout, SmoltestError
from ..transport.base import MachineHandle

DEFAULT_TIMEOUT_S = 120.0
"""Timeout used when neither the strategy nor the caller supplies one."""

DEFAULT_POLL_S = 0.05
"""Poll interval used when neither the strategy nor the caller supplies one."""

CONNECT_TIMEOUT_S = 3.0
"""Per-attempt TCP / driver connect timeout for host-side probes."""

EXEC_PROBE_TIMEOUT_S = 10.0
"""Per-attempt bound on an in-guest probe command, so a stalled guest agent cannot
hold the readiness loop past its own deadline."""

DRIVERS: tuple[str, ...] = ("psycopg", "psycopg2", "asyncpg", "psql")
"""Default host-driver preference of :class:`SqlWaitStrategy`; ``psql`` means in-guest."""

_W = TypeVar("_W", bound="WaitStrategy")


class ReadinessTarget(Protocol):
    """What a strategy may look at: the machine, where it is reachable and its credentials.

    ``logs`` returns the server log when the machine was booted with log capture,
    else it is ``None`` and :class:`LogMessageWaitStrategy` refuses to run.
    """

    @property
    def handle(self) -> MachineHandle: ...

    @property
    def endpoint(self) -> HostEndpoint: ...

    @property
    def username(self) -> str: ...

    @property
    def password(self) -> str: ...

    @property
    def dbname(self) -> str: ...

    @property
    def guest_port(self) -> int: ...

    @property
    def logs(self) -> Callable[[], bytes] | None: ...


@dataclass(frozen=True)
class ReadinessView:
    """A plain :class:`ReadinessTarget` the boot code builds before a facade exists."""

    handle: MachineHandle
    endpoint: HostEndpoint
    username: str = "test"
    password: str = "test"
    dbname: str = "test"
    guest_port: int = 5432
    logs: Callable[[], bytes] | None = None


def guest_log_reader(handle: MachineHandle, path: str) -> Callable[[], bytes]:
    """Return a ``logs`` callable that ``cat``s ``path`` in the guest (empty when absent)."""

    def read() -> bytes:
        outcome = handle.exec(["cat", path], timeout_s=EXEC_PROBE_TIMEOUT_S)
        return outcome.stdout if outcome.ok else b""

    return read


def exec_probe(handle: MachineHandle, argv: Sequence[str]) -> int | None:
    """Run a probe command bounded by :data:`EXEC_PROBE_TIMEOUT_S`; its exit code, or ``None``.

    ``None`` means the command did not finish in time, which a strategy treats
    as "not ready yet" so the outer deadline still ends the wait with
    :class:`~smoltest.errors.ReadinessTimeout` instead of hanging on one call.
    """
    try:
        return handle.exec(list(argv), timeout_s=EXEC_PROBE_TIMEOUT_S).exit_code
    except SmoltestError as exc:
        if exc.code == "TIMEOUT":
            logger.debug("probe %s timed out after %gs", argv[0], EXEC_PROBE_TIMEOUT_S)
            return None
        raise


def _argv(command: str | Sequence[str]) -> tuple[str, ...]:
    if isinstance(command, str):
        return tuple(shlex.split(command))
    return tuple(command)


def _seconds(value: float | timedelta) -> float:
    seconds = value.total_seconds() if isinstance(value, timedelta) else float(value)
    if not math.isfinite(seconds):
        # NaN never compares as expired and infinity never arrives: either would poll forever.
        raise InvalidConfig(f"expected a finite number of seconds; got {value!r}")
    return seconds


class WaitStrategy:
    """Base class: poll :meth:`probe` until it passes or the timeout expires.

    Subclasses implement :meth:`probe`, a single non-blocking-ish check. A timeout
    or poll interval set with :meth:`with_startup_timeout` / :meth:`with_poll_interval`
    wins over the values passed to :meth:`wait_until_ready` (which the boot code
    takes from :class:`~smoltest.config.Settings`), which win over the module defaults.
    """

    _startup_timeout_s: float | None = None
    _poll_interval_s: float | None = None

    def with_startup_timeout(self: _W, seconds: float | timedelta) -> _W:
        """Override the readiness timeout (seconds or ``timedelta``); returns ``self``."""
        value = _seconds(seconds)
        if value <= 0:
            raise InvalidConfig(f"startup timeout must be > 0; got {seconds}")
        self._startup_timeout_s = value
        return self

    def with_poll_interval(self: _W, seconds: float | timedelta) -> _W:
        """Override the poll interval (seconds or ``timedelta``); returns ``self``."""
        value = _seconds(seconds)
        if value <= 0:
            raise InvalidConfig(f"poll interval must be > 0; got {seconds}")
        self._poll_interval_s = value
        return self

    @property
    def startup_timeout_s(self) -> float | None:
        """The timeout set with :meth:`with_startup_timeout`, if any."""
        return self._startup_timeout_s

    @property
    def poll_interval_s(self) -> float | None:
        """The interval set with :meth:`with_poll_interval`, if any."""
        return self._poll_interval_s

    def describe(self) -> str:
        """Short name used in timeout messages."""
        return type(self).__name__

    def probe(self, target: ReadinessTarget) -> bool:
        """One readiness check; ``True`` when the target is ready right now."""
        raise NotImplementedError

    def resolve(
        self, timeout_s: float | None = None, poll_s: float | None = None
    ) -> tuple[float, float]:
        """The effective ``(timeout, poll)`` for a call with these arguments.

        Direct arguments are checked like the builders' values: a non-finite
        timeout or poll interval raises :class:`~smoltest.errors.InvalidConfig`
        here, before it could make the wait poll forever.
        """
        if timeout_s is not None:
            timeout_s = _seconds(timeout_s)
        if poll_s is not None:
            poll_s = _seconds(poll_s)
        timeout = (
            self._startup_timeout_s
            if self._startup_timeout_s is not None
            else (DEFAULT_TIMEOUT_S if timeout_s is None else max(0.0, timeout_s))
        )
        poll = (
            self._poll_interval_s
            if self._poll_interval_s is not None
            else (DEFAULT_POLL_S if poll_s is None else max(0.001, poll_s))
        )
        return timeout, poll

    def wait_until_ready(
        self,
        target: ReadinessTarget,
        timeout_s: float | None = None,
        poll_s: float | None = None,
    ) -> None:
        """Poll :meth:`probe` until it passes; raise :class:`ReadinessTimeout` otherwise."""
        timeout, poll = self.resolve(timeout_s, poll_s)
        deadline = time.monotonic() + timeout
        attempts = 0
        while True:
            attempts += 1
            if self.probe(target):
                logger.debug("%s ready after %d probe(s)", self.describe(), attempts)
                return
            remaining = deadline - time.monotonic()
            if remaining <= 0:
                raise ReadinessTimeout(
                    f"{self.describe()} not ready after {timeout:g}s ({attempts} probes)",
                    timeout_s=timeout,
                )
            time.sleep(min(poll, remaining))

    def __repr__(self) -> str:
        return self.describe()


class PortWaitStrategy(WaitStrategy):
    """Ready when a TCP connection to the published port succeeds from the host.

    ``port`` is a guest port; ``None`` means the PostgreSQL port the target's
    ``endpoint`` already reaches. On the cloud target that endpoint is the local
    end of the tunnel, which accepts as soon as the tunnel is open, so there the
    strategy only proves the tunnel is up; use :class:`PgIsReadyWaitStrategy`
    or :class:`SqlWaitStrategy` to prove the server is.
    """

    def __init__(self, port: int | None = None, *, connect_timeout_s: float = 1.0) -> None:
        self._port = port
        self._connect_timeout_s = connect_timeout_s

    def describe(self) -> str:
        return f"PortWaitStrategy(port={self._port if self._port is not None else 'endpoint'})"

    def _endpoint(self, target: ReadinessTarget) -> HostEndpoint:
        if self._port is None or self._port == target.guest_port:
            return target.endpoint
        host, port = target.handle.host_port(self._port)
        return HostEndpoint(host, port)

    def probe(self, target: ReadinessTarget) -> bool:
        endpoint = self._endpoint(target)
        try:
            with socket.create_connection(
                (endpoint.host, endpoint.port), timeout=self._connect_timeout_s
            ):
                return True
        except OSError:
            return False


class ExecWaitStrategy(WaitStrategy):
    """Ready when ``command`` exits with ``expected_exit_code`` inside the guest."""

    def __init__(self, command: str | Sequence[str], expected_exit_code: int = 0) -> None:
        self._argv = _argv(command)
        if not self._argv:
            raise InvalidConfig("ExecWaitStrategy needs a non-empty command")
        self._expected = expected_exit_code

    @property
    def argv(self) -> tuple[str, ...]:
        """The command run in the guest."""
        return self._argv

    def describe(self) -> str:
        return f"ExecWaitStrategy({shlex.join(self._argv)!r} -> {self._expected})"

    def probe(self, target: ReadinessTarget) -> bool:
        return exec_probe(target.handle, self._argv) == self._expected


class PgIsReadyWaitStrategy(WaitStrategy):
    """Ready when ``pg_isready`` in the guest reports the server accepting connections.

    This is the default: it needs no host driver and no password, and it reaches
    the server over TCP inside the guest so it proves the final server (not
    initdb's unix-socket-only temporary one) is up.
    """

    def __init__(self, username: str | None = None, port: int = 5432) -> None:
        self._username = username
        self._port = port

    def argv(self, target: ReadinessTarget) -> list[str]:
        """The ``pg_isready`` command for ``target``."""
        user = self._username if self._username is not None else target.username
        return ["pg_isready", "-h", "localhost", "-p", str(self._port), "-U", user]

    def describe(self) -> str:
        return f"PgIsReadyWaitStrategy(port={self._port})"

    def probe(self, target: ReadinessTarget) -> bool:
        return exec_probe(target.handle, self.argv(target)) == 0


class SqlWaitStrategy(WaitStrategy):
    """Ready when ``sql`` runs successfully.

    The first importable driver in ``prefer`` connects from the host to the
    target's endpoint with the target's credentials (and closes the connection
    again, so a checkpoint taken afterwards sees no client); ``"psql"`` runs the
    statement in the guest over the trusted unix socket instead. ``asyncpg`` is
    skipped when an event loop is already running in the calling thread.
    """

    def __init__(
        self,
        sql: str = "SELECT 1",
        prefer: Sequence[str] = DRIVERS,
        *,
        connect_timeout_s: float = CONNECT_TIMEOUT_S,
    ) -> None:
        self._sql = sql
        self._prefer = tuple(prefer)
        if not self._prefer:
            raise InvalidConfig("SqlWaitStrategy needs at least one driver preference")
        self._connect_timeout_s = connect_timeout_s

    @property
    def sql(self) -> str:
        """The probe statement."""
        return self._sql

    def describe(self) -> str:
        return f"SqlWaitStrategy({self._sql!r})"

    def dsn(self, target: ReadinessTarget) -> str:
        """The libpq URI a host driver uses to reach ``target``."""
        timeout = max(1, round(self._connect_timeout_s))
        return (
            f"postgresql://{quote(target.username, safe='')}:{quote(target.password, safe='')}"
            f"@{target.endpoint.host}:{target.endpoint.port}/{quote(target.dbname, safe='')}"
            f"?connect_timeout={timeout}"
        )

    def select_driver(self) -> tuple[str, Any] | None:
        """``(name, module)`` of the first usable host driver, or ``None`` for in-guest psql."""
        for name in self._prefer:
            if name == "psql":
                return None
            if name == "asyncpg" and _loop_running():
                continue
            try:
                module = importlib.import_module(name)
            except ImportError:
                continue
            return name, module
        return None

    def probe(self, target: ReadinessTarget) -> bool:
        driver = self.select_driver()
        if driver is None:
            return self._probe_psql(target)
        name, module = driver
        try:
            if name == "asyncpg":
                asyncio.run(self._probe_asyncpg(module, self.dsn(target)))
            else:
                self._probe_dbapi(module, self.dsn(target))
        except Exception as exc:
            logger.debug("%s via %s not ready: %s", self.describe(), name, exc)
            return False
        return True

    def _probe_psql(self, target: ReadinessTarget) -> bool:
        argv = [
            "psql",
            "-tA",
            "-U",
            target.username,
            "-d",
            target.dbname,
            "-p",
            str(target.guest_port),
            "-c",
            self._sql,
        ]
        return exec_probe(target.handle, argv) == 0

    def _probe_dbapi(self, module: Any, dsn: str) -> None:
        conn = module.connect(dsn)
        try:
            cursor = conn.cursor()
            try:
                cursor.execute(self._sql)
            finally:
                cursor.close()
        finally:
            conn.close()

    async def _probe_asyncpg(self, module: Any, dsn: str) -> None:
        conn = await module.connect(dsn, timeout=self._connect_timeout_s)
        try:
            await conn.execute(self._sql)
        finally:
            await conn.close()


def _loop_running() -> bool:
    try:
        asyncio.get_running_loop()
    except RuntimeError:
        return False
    return True


class LogMessageWaitStrategy(WaitStrategy):
    """Ready once ``message`` (a regular expression) appears ``times`` times in the log.

    Needs a target booted with log capture; otherwise the wait raises
    :class:`~smoltest.errors.NotSupportedError` because a microVM workload has no
    stdout pipe the host can read.
    """

    def __init__(self, message: str, times: int = 1) -> None:
        if times < 1:
            raise InvalidConfig(f"times must be >= 1; got {times}")
        self._pattern = re.compile(message)
        self._times = times

    def describe(self) -> str:
        return f"LogMessageWaitStrategy({self._pattern.pattern!r} x{self._times})"

    @staticmethod
    def _reader(target: ReadinessTarget) -> Callable[[], bytes]:
        reader = target.logs
        if reader is None:
            raise NotSupportedError(
                "LogMessageWaitStrategy needs the server log: boot with capture_logs=True "
                "or use PgIsReadyWaitStrategy / SqlWaitStrategy"
            )
        return reader

    def wait_until_ready(
        self,
        target: ReadinessTarget,
        timeout_s: float | None = None,
        poll_s: float | None = None,
    ) -> None:
        self._reader(target)  # fail fast, before the first poll
        super().wait_until_ready(target, timeout_s, poll_s)

    def probe(self, target: ReadinessTarget) -> bool:
        text = self._reader(target)().decode("utf-8", "replace")
        return len(self._pattern.findall(text)) >= self._times


class HttpWaitStrategy(WaitStrategy):
    """Placeholder for testcontainers' HTTP strategy; always unsupported here."""

    def __init__(self, *args: Any, **kwargs: Any) -> None:
        raise NotSupportedError(
            "HttpWaitStrategy is not supported: smoltest machines run PostgreSQL only; "
            "use PgIsReadyWaitStrategy, SqlWaitStrategy or PortWaitStrategy"
        )

    def probe(self, target: ReadinessTarget) -> bool:  # pragma: no cover - unreachable
        raise NotSupportedError("HttpWaitStrategy is not supported")


class CompositeWaitStrategy(WaitStrategy):
    """Wait for each strategy in turn, sharing one deadline.

    A child's own ``with_startup_timeout`` still wins for that child; otherwise it
    gets whatever time remains of the composite's timeout.
    """

    def __init__(self, *strategies: WaitStrategy) -> None:
        self._strategies = tuple(strategies)

    @property
    def strategies(self) -> tuple[WaitStrategy, ...]:
        """The strategies in order."""
        return self._strategies

    def describe(self) -> str:
        inner = ", ".join(s.describe() for s in self._strategies)
        return f"CompositeWaitStrategy({inner})"

    def probe(self, target: ReadinessTarget) -> bool:
        return all(strategy.probe(target) for strategy in self._strategies)

    def wait_until_ready(
        self,
        target: ReadinessTarget,
        timeout_s: float | None = None,
        poll_s: float | None = None,
    ) -> None:
        timeout, poll = self.resolve(timeout_s, poll_s)
        deadline = time.monotonic() + timeout
        for strategy in self._strategies:
            strategy.wait_until_ready(target, max(0.0, deadline - time.monotonic()), poll)


def default_wait(username: str | None = None, *, port: int = 5432) -> WaitStrategy:
    """The strategy smoltest uses when none is given: in-guest ``pg_isready``."""
    return PgIsReadyWaitStrategy(username, port)


__all__ = [
    "CONNECT_TIMEOUT_S",
    "DEFAULT_POLL_S",
    "DEFAULT_TIMEOUT_S",
    "DRIVERS",
    "EXEC_PROBE_TIMEOUT_S",
    "CompositeWaitStrategy",
    "ExecWaitStrategy",
    "HttpWaitStrategy",
    "LogMessageWaitStrategy",
    "PgIsReadyWaitStrategy",
    "PortWaitStrategy",
    "ReadinessTarget",
    "ReadinessView",
    "SqlWaitStrategy",
    "WaitStrategy",
    "default_wait",
    "exec_probe",
    "guest_log_reader",
]
