"""Wait strategies against FakeEngine machines: probes, timeouts and driver selection."""

from __future__ import annotations

import asyncio
import sys
import threading
import time
import types
from collections.abc import Callable, Iterator
from datetime import timedelta
from typing import Any

import pytest

from smoltest._ports import HostEndpoint, pick_free_port
from smoltest.errors import (
    BootError,
    InvalidConfig,
    NotSupportedError,
    ReadinessTimeout,
    SmoltestError,
)
from smoltest.testing import FakeEngine, FakeMachine
from smoltest.transport.base import MachineSpec, PortMapping
from smoltest.wait import strategies as ws
from smoltest.wait.strategies import (
    EXEC_PROBE_TIMEOUT_S,
    CompositeWaitStrategy,
    ExecWaitStrategy,
    HttpWaitStrategy,
    LogMessageWaitStrategy,
    PgIsReadyWaitStrategy,
    PortWaitStrategy,
    ReadinessTarget,
    ReadinessView,
    SqlWaitStrategy,
    WaitStrategy,
    default_wait,
    guest_log_reader,
)


def boot(engine: FakeEngine, image: str = "postgres:16") -> FakeMachine:
    spec = MachineSpec(
        image=image, argv=("docker-entrypoint.sh", "postgres"), ports=(PortMapping(None, 5432),)
    )
    machine = engine.create(spec, "local")
    assert isinstance(machine, FakeMachine)
    return machine


def view(machine: FakeMachine, **kw: Any) -> ReadinessView:
    host, port = machine.host_port(5432)
    return ReadinessView(handle=machine, endpoint=HostEndpoint(host, port), **kw)


@pytest.fixture
def pg(fake_engine: FakeEngine) -> Iterator[FakeMachine]:
    machine = boot(fake_engine)
    yield machine
    machine.delete()


@pytest.fixture
def not_pg(fake_engine: FakeEngine) -> Iterator[FakeMachine]:
    machine = boot(fake_engine, image="nginx:1.27-alpine")
    yield machine
    machine.delete()


def eventually(condition: Callable[[], bool], timeout_s: float = 2.0) -> bool:
    deadline = time.monotonic() + timeout_s
    while not condition():
        if time.monotonic() > deadline:
            return False
        time.sleep(0.01)
    return True


def flip_later(machine: FakeMachine, delay_s: float = 0.15) -> threading.Thread:
    def flip() -> None:
        machine.services.add("postgres")

    t = threading.Timer(delay_s, flip)
    t.start()
    return t


# -- base class -----------------------------------------------------------------------


def test_view_satisfies_protocol(pg: FakeMachine) -> None:
    target: ReadinessTarget = view(pg)
    assert target.username == "test" and target.guest_port == 5432 and target.logs is None


def test_builders_return_self_and_validate() -> None:
    s = PgIsReadyWaitStrategy()
    assert s.with_startup_timeout(3).with_poll_interval(0.5) is s
    assert s.startup_timeout_s == 3.0 and s.poll_interval_s == 0.5
    assert s.resolve(10, 1) == (3.0, 0.5)  # configured values win over the caller's
    assert PgIsReadyWaitStrategy().resolve(10, 1) == (10.0, 1.0)
    assert PgIsReadyWaitStrategy().resolve() == (ws.DEFAULT_TIMEOUT_S, ws.DEFAULT_POLL_S)
    with pytest.raises(InvalidConfig):
        s.with_startup_timeout(0)
    with pytest.raises(InvalidConfig):
        s.with_poll_interval(-1)


def test_base_probe_is_abstract(pg: FakeMachine) -> None:
    with pytest.raises(NotImplementedError):
        WaitStrategy().probe(view(pg))


# -- port -----------------------------------------------------------------------------


def test_port_wait_passes_on_live_listener(pg: FakeMachine) -> None:
    PortWaitStrategy().wait_until_ready(view(pg), 2, 0.01)
    PortWaitStrategy(5432).wait_until_ready(view(pg), 2, 0.01)
    assert eventually(lambda: pg.accepted_connections(5432) >= 2)


def test_port_wait_times_out_on_free_port(pg: FakeMachine) -> None:
    target = ReadinessView(handle=pg, endpoint=HostEndpoint("127.0.0.1", pick_free_port()))
    with pytest.raises(ReadinessTimeout) as info:
        PortWaitStrategy().wait_until_ready(target, 0.2, 0.02)
    assert info.value.timeout_s == 0.2
    assert "PortWaitStrategy" in str(info.value) and info.value.stage == "wait"
    assert isinstance(info.value, TimeoutError)


def test_port_wait_on_other_guest_port_uses_handle(fake_engine: FakeEngine) -> None:
    spec = MachineSpec(
        image="postgres:16", ports=(PortMapping(None, 5432), PortMapping(None, 8080))
    )
    m = fake_engine.create(spec, "local")
    assert isinstance(m, FakeMachine)
    PortWaitStrategy(8080).wait_until_ready(view(m), 2, 0.01)
    assert eventually(lambda: m.accepted_connections(8080) == 1)
    m.delete()


# -- exec / pg_isready ----------------------------------------------------------------


def test_pg_isready_flips_when_service_appears(not_pg: FakeMachine) -> None:
    strategy = PgIsReadyWaitStrategy()
    assert strategy.probe(view(not_pg)) is False
    timer = flip_later(not_pg)
    try:
        strategy.wait_until_ready(view(not_pg), 5, 0.01)
    finally:
        timer.join()
    assert not_pg.exec_log[-1] == ("pg_isready", "-h", "localhost", "-p", "5432", "-U", "test")
    assert len(not_pg.exec_log) > 1  # it really polled


def test_pg_isready_uses_target_user_or_override(pg: FakeMachine) -> None:
    PgIsReadyWaitStrategy().wait_until_ready(view(pg, username="alice"), 1, 0.01)
    assert pg.exec_log[-1][-1] == "alice"
    PgIsReadyWaitStrategy("bob", port=5433).wait_until_ready(view(pg), 1, 0.01)
    assert pg.exec_log[-1] == ("pg_isready", "-h", "localhost", "-p", "5433", "-U", "bob")


def test_default_wait_is_pg_isready(pg: FakeMachine) -> None:
    strategy = default_wait("carol")
    assert isinstance(strategy, PgIsReadyWaitStrategy)
    assert strategy.argv(view(pg))[-1] == "carol"
    other = default_wait(port=5433)
    assert isinstance(other, PgIsReadyWaitStrategy)
    assert other.argv(view(pg))[4:] == ["5433", "-U", "test"]


def test_exec_wait_string_and_list(pg: FakeMachine) -> None:
    pg.files["/etc/motd"] = b"hi"
    ExecWaitStrategy("cat /etc/motd").wait_until_ready(view(pg), 1, 0.01)
    ExecWaitStrategy(["cat", "/missing"], expected_exit_code=1).wait_until_ready(view(pg), 1, 0.01)
    assert ExecWaitStrategy("true").argv == ("true",)
    with pytest.raises(InvalidConfig):
        ExecWaitStrategy("")


def test_exec_wait_times_out_and_configured_timeout_wins(pg: FakeMachine) -> None:
    strategy = (
        ExecWaitStrategy("no-such-program").with_startup_timeout(0.1).with_poll_interval(0.01)
    )
    with pytest.raises(ReadinessTimeout, match="ExecWaitStrategy") as info:
        strategy.wait_until_ready(view(pg), 60, 5)  # the caller's 60 s is overridden
    assert info.value.timeout_s == 0.1
    assert 2 <= len(pg.exec_log) <= 30


def test_zero_timeout_probes_exactly_once(pg: FakeMachine) -> None:
    with pytest.raises(ReadinessTimeout):
        ExecWaitStrategy("no-such-program").wait_until_ready(view(pg), 0, 0.01)
    assert len(pg.exec_log) == 1


# -- log message ----------------------------------------------------------------------


def test_log_message_without_logs_is_unsupported(pg: FakeMachine) -> None:
    with pytest.raises(NotSupportedError, match="capture_logs"):
        LogMessageWaitStrategy("ready").wait_until_ready(view(pg), 1, 0.01)
    with pytest.raises(NotSupportedError):
        LogMessageWaitStrategy("ready").probe(view(pg))
    assert pg.exec_log == []  # failed fast, no polling


def test_log_message_waits_for_count(pg: FakeMachine) -> None:
    lines: list[bytes] = [b"database system is ready to accept connections\n"]

    def logs() -> bytes:
        return b"".join(lines)

    target = view(pg, logs=logs)
    LogMessageWaitStrategy(r"ready to accept").wait_until_ready(target, 1, 0.01)
    twice = LogMessageWaitStrategy("ready to accept", times=2)
    assert twice.probe(target) is False
    timer = threading.Timer(0.1, lambda: lines.append(lines[0]))
    timer.start()
    try:
        twice.wait_until_ready(target, 2, 0.01)
    finally:
        timer.join()
    with pytest.raises(InvalidConfig):
        LogMessageWaitStrategy("x", times=0)


def test_guest_log_reader_reads_file_or_empty(pg: FakeMachine) -> None:
    pg.files["/var/log/pg.log"] = b"LOG: ready\n"
    assert guest_log_reader(pg, "/var/log/pg.log")() == b"LOG: ready\n"
    assert guest_log_reader(pg, "/nope")() == b""
    LogMessageWaitStrategy("ready").wait_until_ready(
        view(pg, logs=guest_log_reader(pg, "/var/log/pg.log")), 1, 0.01
    )


# -- sql ------------------------------------------------------------------------------


def test_sql_in_guest_psql_path(pg: FakeMachine, not_pg: FakeMachine) -> None:
    strategy = SqlWaitStrategy(prefer=("psql",))
    assert strategy.select_driver() is None
    strategy.wait_until_ready(view(pg), 1, 0.01)
    assert pg.sql_log == ["SELECT 1"]
    assert pg.exec_log[-1] == ("psql", "-tA", "-U", "test", "-d", "test", "-c", "SELECT 1")
    with pytest.raises(ReadinessTimeout, match="SELECT 1"):
        strategy.wait_until_ready(view(not_pg), 0.1, 0.01)
    SqlWaitStrategy("SELECT 2", prefer=("psql",)).wait_until_ready(view(pg, dbname="app"), 1, 0.01)
    assert pg.exec_log[-1][5:] == ("app", "-c", "SELECT 2")


def test_sql_unknown_driver_falls_through_to_psql(pg: FakeMachine) -> None:
    strategy = SqlWaitStrategy(prefer=("smoltest_no_such_driver_xyz", "psql"))
    assert strategy.select_driver() is None
    strategy.wait_until_ready(view(pg), 1, 0.01)
    assert pg.sql_log == ["SELECT 1"]
    with pytest.raises(InvalidConfig):
        SqlWaitStrategy(prefer=())


class _FakeCursor:
    def __init__(self, conn: _FakeConn) -> None:
        self.conn = conn

    def execute(self, sql: str) -> None:
        self.conn.module.executed.append(sql)

    def close(self) -> None:
        self.conn.module.cursors_closed += 1


class _FakeConn:
    def __init__(self, module: FakeDbapi) -> None:
        self.module = module

    def cursor(self) -> _FakeCursor:
        return _FakeCursor(self)

    def close(self) -> None:
        self.module.closed += 1


class FakeDbapi(types.ModuleType):
    """A DB-API-ish driver module: ``connect(dsn)`` fails ``fail_first`` times, then works."""

    def __init__(self, name: str, *, fail_first: int = 0) -> None:
        super().__init__(name)
        self.dsns: list[str] = []
        self.executed: list[str] = []
        self.closed = 0
        self.cursors_closed = 0
        self.failures_left = fail_first

    def connect(self, dsn: str) -> _FakeConn:
        self.dsns.append(dsn)
        if self.failures_left > 0:
            self.failures_left -= 1
            raise ConnectionRefusedError("server starting up")
        return _FakeConn(self)


def fake_dbapi(name: str, *, fail_first: int = 0) -> FakeDbapi:
    return FakeDbapi(name, fail_first=fail_first)


def test_sql_host_driver_module_is_used_and_closed(
    pg: FakeMachine, monkeypatch: pytest.MonkeyPatch
) -> None:
    driver = fake_dbapi("smoltest_fake_pg", fail_first=2)
    monkeypatch.setitem(sys.modules, "smoltest_fake_pg", driver)
    target = view(pg, username="al ice", password="p@ss/wörd", dbname="my db")
    strategy = SqlWaitStrategy("SELECT 42", prefer=("smoltest_fake_pg", "psql"))
    selected = strategy.select_driver()
    assert selected is not None and selected[0] == "smoltest_fake_pg"
    strategy.wait_until_ready(target, 2, 0.01)
    assert len(driver.dsns) == 3 and driver.executed == ["SELECT 42"]
    assert driver.closed == 1 and driver.cursors_closed == 1
    host, port = pg.host_port(5432)
    assert driver.dsns[-1] == (
        f"postgresql://al%20ice:p%40ss%2Fw%C3%B6rd@{host}:{port}/my%20db?connect_timeout=3"
    )
    assert pg.sql_log == []  # never fell back to the guest


def test_sql_prefers_first_importable_driver(
    pg: FakeMachine, monkeypatch: pytest.MonkeyPatch
) -> None:
    first, second = fake_dbapi("smoltest_pg_a"), fake_dbapi("smoltest_pg_b")
    monkeypatch.setitem(sys.modules, "smoltest_pg_a", first)
    monkeypatch.setitem(sys.modules, "smoltest_pg_b", second)
    SqlWaitStrategy(prefer=("missing_driver_q", "smoltest_pg_a", "smoltest_pg_b")).wait_until_ready(
        view(pg), 1, 0.01
    )
    assert first.executed == ["SELECT 1"] and second.executed == []


class _FakeAsyncConn:
    def __init__(self, module: FakeAsyncpg) -> None:
        self.module = module

    async def execute(self, sql: str) -> None:
        self.module.executed.append(sql)

    async def close(self) -> None:
        self.module.closed += 1


class FakeAsyncpg(types.ModuleType):
    """An asyncpg-shaped module: ``await connect(dsn, timeout=...)``."""

    def __init__(self, name: str = "asyncpg") -> None:
        super().__init__(name)
        self.executed: list[str] = []
        self.closed = 0
        self.timeouts: list[float] = []

    async def connect(self, dsn: str, timeout: float) -> _FakeAsyncConn:
        self.timeouts.append(timeout)
        return _FakeAsyncConn(self)


def fake_asyncpg(name: str = "asyncpg") -> FakeAsyncpg:
    return FakeAsyncpg(name)


def test_sql_asyncpg_path_runs_its_own_loop(
    pg: FakeMachine, monkeypatch: pytest.MonkeyPatch
) -> None:
    module = fake_asyncpg()
    monkeypatch.setitem(sys.modules, "asyncpg", module)
    strategy = SqlWaitStrategy(prefer=("asyncpg", "psql"), connect_timeout_s=1.5)
    strategy.wait_until_ready(view(pg), 1, 0.01)
    assert module.executed == ["SELECT 1"] and module.closed == 1 and module.timeouts == [1.5]
    assert pg.sql_log == []


def test_sql_skips_asyncpg_inside_a_running_loop(
    pg: FakeMachine, monkeypatch: pytest.MonkeyPatch
) -> None:
    module = fake_asyncpg()
    monkeypatch.setitem(sys.modules, "asyncpg", module)
    strategy = SqlWaitStrategy(prefer=("asyncpg", "psql"))

    async def main() -> bool:
        assert strategy.select_driver() is None  # asyncpg would need asyncio.run
        return strategy.probe(view(pg))

    assert asyncio.run(main()) is True
    assert module.executed == [] and pg.sql_log == ["SELECT 1"]


def test_sql_driver_errors_mean_not_ready(pg: FakeMachine, monkeypatch: pytest.MonkeyPatch) -> None:
    module = fake_dbapi("smoltest_pg_err", fail_first=10**6)
    monkeypatch.setitem(sys.modules, "smoltest_pg_err", module)
    with pytest.raises(ReadinessTimeout):
        SqlWaitStrategy(prefer=("smoltest_pg_err",)).wait_until_ready(view(pg), 0.1, 0.01)
    assert len(module.dsns) >= 2


# -- composite / http -----------------------------------------------------------------


def test_composite_waits_for_each_in_turn(not_pg: FakeMachine) -> None:
    composite = CompositeWaitStrategy(PortWaitStrategy(), PgIsReadyWaitStrategy())
    assert composite.probe(view(not_pg)) is False
    timer = flip_later(not_pg)
    try:
        composite.wait_until_ready(view(not_pg), 5, 0.01)
    finally:
        timer.join()
    assert composite.probe(view(not_pg)) is True
    assert len(composite.strategies) == 2
    assert "PortWaitStrategy" in composite.describe() and "PgIsReady" in composite.describe()
    CompositeWaitStrategy().wait_until_ready(view(not_pg), 0, 0.01)  # nothing to wait for


def test_composite_shares_one_deadline(pg: FakeMachine) -> None:
    composite = CompositeWaitStrategy(PortWaitStrategy(), ExecWaitStrategy("no-such-program"))
    with pytest.raises(ReadinessTimeout, match="ExecWaitStrategy") as info:
        composite.wait_until_ready(view(pg), 0.2, 0.02)
    assert info.value.timeout_s is not None and info.value.timeout_s <= 0.2


def test_http_wait_is_unsupported() -> None:
    with pytest.raises(NotSupportedError, match="HttpWaitStrategy"):
        HttpWaitStrategy(80, "/health")


def test_package_reexports() -> None:
    import smoltest
    import smoltest.wait as pkg

    assert pkg.PgIsReadyWaitStrategy is PgIsReadyWaitStrategy
    assert smoltest.SqlWaitStrategy is SqlWaitStrategy and smoltest.default_wait is default_wait


def test_builders_accept_timedelta() -> None:
    strategy = (
        PgIsReadyWaitStrategy()
        .with_startup_timeout(timedelta(seconds=30))
        .with_poll_interval(timedelta(milliseconds=200))
    )
    assert (strategy.startup_timeout_s, strategy.poll_interval_s) == (30.0, 0.2)
    with pytest.raises(InvalidConfig):
        PgIsReadyWaitStrategy().with_startup_timeout(timedelta(0))


def hang(_machine: FakeMachine, _argv: tuple[str, ...]) -> None:
    raise BootError("agent stalled", stage="exec", code="TIMEOUT")


def test_probe_execs_are_bounded_and_a_timeout_means_not_ready(
    fake_engine: FakeEngine, pg: FakeMachine
) -> None:
    fake_engine.exec_handlers["pg_isready"] = hang
    fake_engine.exec_handlers["psql"] = hang
    fake_engine.exec_handlers["true"] = hang
    for strategy in (
        PgIsReadyWaitStrategy(),
        SqlWaitStrategy(prefer=("psql",)),
        ExecWaitStrategy("true"),
    ):
        with pytest.raises(ReadinessTimeout):
            strategy.wait_until_ready(view(pg), 0.1, 0.01)
    assert guest_log_reader(pg, "/nowhere")() == b""
    recorded = [c for c in fake_engine.ops("exec") if c["machine"] == pg.id]
    assert recorded and all(c["timeout_s"] == EXEC_PROBE_TIMEOUT_S for c in recorded)


def test_other_probe_errors_still_propagate(fake_engine: FakeEngine, pg: FakeMachine) -> None:
    def broken(_machine: FakeMachine, _argv: tuple[str, ...]) -> None:
        raise SmoltestError("agent gone", code="AGENT")

    fake_engine.exec_handlers["pg_isready"] = broken
    with pytest.raises(SmoltestError, match="agent gone"):
        PgIsReadyWaitStrategy().probe(view(pg))
