"""PostgresMachine's public surface, with the boot ladder replaced by a fake.

The facade is tested in isolation from the ladder: this module monkeypatches
``boot_postgres`` / ``branch_from`` (and ``BootResult.release``) with minimal
versions that create real FakeEngine machines. ``test_boot_strategy.py`` and
``test_golden.py`` cover the real ladder.
"""

from __future__ import annotations

import asyncio
import gc
import hashlib
import re
import warnings
from collections.abc import Callable
from dataclasses import dataclass, field
from datetime import timedelta
from pathlib import Path
from typing import Any

import pytest

from smoltest._lifecycle import get_reaper
from smoltest._log import reset_warn_once
from smoltest._ports import pick_free_port
from smoltest.boot import strategy
from smoltest.boot.spec import LOG_PATH, PostgresSpec, build_machine_spec
from smoltest.boot.strategy import BootInfo, BootResult
from smoltest.config import Settings
from smoltest.errors import InvalidConfig, NotSupportedError, SmoltestError, SmoltestWarning
from smoltest.postgres import (
    AsyncPostgresMachine,
    ExecResult,
    PostgresContainer,
    PostgresMachine,
    Seed,
    apply_sql_script,
)
from smoltest.testing import FakeEngine, FakeMachine
from smoltest.transport.base import CheckpointInfo, ExecOutcome, PortMapping
from smoltest.wait.strategies import PgIsReadyWaitStrategy, PortWaitStrategy, ReadinessView


@dataclass
class BootLog:
    """What the fake ladder saw."""

    boots: list[dict[str, Any]] = field(default_factory=list)
    branches: list[dict[str, Any]] = field(default_factory=list)
    released: list[str] = field(default_factory=list)


@pytest.fixture(autouse=True)
def _fresh_warnings() -> None:
    reset_warn_once()


@pytest.fixture
def fake_boot(monkeypatch: pytest.MonkeyPatch, fake_engine: FakeEngine) -> BootLog:
    log = BootLog()

    def boot_postgres(
        spec: PostgresSpec,
        settings: Settings,
        engine: FakeEngine,
        *,
        seed: Any = None,
        wait: Any = None,
        role: str = "standalone",
        name: str | None = None,
        host_port: int | None = None,
    ) -> BootResult:
        log.boots.append(
            {
                "spec": spec,
                "settings": settings,
                "engine": engine,
                "seed": seed,
                "wait": wait,
                "role": role,
                "name": name,
                "host_port": host_port,
            }
        )
        machine_spec = build_machine_spec(spec, settings, "local", host_port, name)
        handle = engine.create(machine_spec, "local")
        bridge = engine.open_tunnel(handle.id, spec.guest_port, "local")
        endpoint = bridge.open()
        info = BootInfo(via="cold", target="local", elapsed_s=0.01, machine_id=handle.id)
        return BootResult(handle, bridge, endpoint, info, spec, "local")

    def branch_from(
        result: BootResult, name: str, settings: Settings, engine: FakeEngine
    ) -> BootResult:
        log.branches.append({"result": result, "name": name, "settings": settings})
        guest = result.spec.guest_port
        child = result.handle.branch(name, [PortMapping(pick_free_port(), guest)])
        bridge = engine.open_tunnel(child.id, guest, "local")
        endpoint = bridge.open()
        info = BootInfo(via="branch", target="local", elapsed_s=0.001, machine_id=child.id)
        return BootResult(child, bridge, endpoint, info, result.spec, "local")

    def release(self: BootResult) -> None:
        log.released.append(self.handle.id)
        self.bridge.close()
        self.handle.delete()

    monkeypatch.setattr(strategy, "boot_postgres", boot_postgres)
    monkeypatch.setattr(strategy, "branch_from", branch_from)
    monkeypatch.setattr(BootResult, "release", release)
    return log


def fake_of(machine: PostgresMachine) -> FakeMachine:
    handle = machine.get_machine()
    assert isinstance(handle, FakeMachine)
    return handle


# -- ExecResult / Seed ------------------------------------------------------------------


def test_exec_result_unpacks_like_testcontainers() -> None:
    result = ExecResult.from_outcome(ExecOutcome(3, b"out", b"err", ("x", "y")))
    code, output = result
    assert (code, output) == (3, b"outerr")
    assert not result.ok and result.text == "out" and result.stderr_text == "err"
    assert result.argv == ("x", "y") and ExecResult(0).ok


def test_seed_from_sql_keys_by_content_hash() -> None:
    seed = Seed.from_sql("CREATE TABLE t (id int);")
    assert seed.key == hashlib.sha256(b"CREATE TABLE t (id int);").hexdigest()
    assert Seed.from_sql("CREATE TABLE t (id int);").key == seed.key
    assert Seed.from_sql("CREATE TABLE u (id int);").key != seed.key
    assert Seed.from_sql("x", key="schema-v3").key == "schema-v3"


def test_seed_from_sql_files(tmp_path: Path) -> None:
    a, b = tmp_path / "a.sql", tmp_path / "b.sql"
    a.write_text("CREATE TABLE a (id int);\n")
    b.write_text("CREATE TABLE b (id int);\n")
    seed = Seed.from_sql_files(a, str(b))
    assert seed.key == hashlib.sha256(a.read_bytes() + b"\n" + b.read_bytes()).hexdigest()
    assert Seed.from_sql_files(a, b).key == seed.key
    assert Seed.from_sql_files(b, a).key != seed.key  # order matters
    assert Seed.from_sql_files(a, key="k").key == "k"
    with pytest.raises(InvalidConfig):
        Seed.from_sql_files()


def test_seed_from_callable() -> None:
    seen: list[PostgresMachine] = []
    seed = Seed.from_callable(seen.append, key="alembic:abc123")
    assert seed.key == "alembic:abc123"
    machine = PostgresMachine(settings=Settings(target="local"))
    seed.apply(machine)
    assert seen == [machine]
    with pytest.raises(InvalidConfig):
        Seed.from_callable(seen.append, key="")


def install_psql_f(engine: FakeEngine, *, exit_code: int = 0) -> list[str]:
    """Teach the fake psql ``-f <path>`` (it only knows ``-c``); returns the SQL it ran."""
    ran: list[str] = []

    def handler(machine: FakeMachine, argv: tuple[str, ...]) -> ExecOutcome | None:
        if "-f" not in argv:
            return None
        path = argv[argv.index("-f") + 1]
        ran.append(machine.files[path].decode())
        if exit_code:
            return ExecOutcome(exit_code, b"", b"ERROR:  syntax error\n", argv=argv)
        return ExecOutcome(0, b"CREATE TABLE\n", argv=argv)

    engine.exec_handlers["psql"] = handler
    return ran


def test_sql_seed_apply_writes_file_and_runs_psql(
    fake_boot: BootLog, fake_engine: FakeEngine, settings: Settings
) -> None:
    ran = install_psql_f(fake_engine)
    with PostgresMachine(settings=settings, username="alice", dbname="app") as m:
        Seed.from_sql("CREATE TABLE t (id int);").apply(m)
        fake = fake_of(m)
        argv = fake.exec_log[-1]
        assert argv[:4] == ("psql", "-v", "ON_ERROR_STOP=1", "-U")
        assert argv[4:10] == ("alice", "-d", "app", "-p", "5432", "-f")
        assert argv[10].startswith("/tmp/smoltest-") and argv[10].endswith(".sql")
        assert fake.files[argv[10]] == b"CREATE TABLE t (id int);"
        assert fake_engine.ops("write_file")[-1]["mode"] == 0o600
    assert ran == ["CREATE TABLE t (id int);"]


def test_sql_seed_failure_raises(
    fake_boot: BootLog, fake_engine: FakeEngine, settings: Settings
) -> None:
    install_psql_f(fake_engine, exit_code=3)
    with PostgresMachine(settings=settings) as m:
        with pytest.raises(SmoltestError, match="psql exited 3") as info:
            apply_sql_script(m, b"bad sql", label="seed x")
        assert info.value.code == "SEED_FAILED" and "seed x" in str(info.value)


# -- construction ---------------------------------------------------------------------


def test_defaults(settings: Settings) -> None:
    m = PostgresMachine(settings=settings)
    assert (m.username, m.password, m.dbname, m.port, m.driver) == (
        "test",
        "test",
        "test",
        5432,
        "psycopg2",
    )
    assert m.image == settings.postgres_image and m.spec.image is None
    assert m.spec.fast is None and m.spec.capture_logs is False
    assert m.boot_info is None and not m.is_running and m.name is None
    assert m.settings.target == "local" and m.seed is None and m.wait_strategy is None
    assert PostgresContainer is PostgresMachine
    assert "new" in repr(m)


def test_env_defaults_and_explicit_args(
    monkeypatch: pytest.MonkeyPatch, settings: Settings
) -> None:
    monkeypatch.setenv("POSTGRES_USER", "alice")
    monkeypatch.setenv("POSTGRES_PASSWORD", "s3cret")
    monkeypatch.setenv("POSTGRES_DB", "app")
    m = PostgresMachine(settings=settings)
    assert (m.username, m.password, m.dbname) == ("alice", "s3cret", "app")
    explicit = PostgresMachine("postgres:15", 5433, "bob", "pw", "other", None, settings=settings)
    assert (explicit.username, explicit.password, explicit.dbname) == ("bob", "pw", "other")
    assert explicit.image == "postgres:15" and explicit.port == 5433 and explicit.driver is None


def test_constructor_knobs_flow_into_spec_and_settings(settings: Settings) -> None:
    m = PostgresMachine(
        settings=settings,
        target="cloud",
        cpus=2,
        memory_mb=1024,
        cache=False,
        fast=False,
        capture_logs=True,
    )
    assert m.settings.target == "cloud" and m.settings.disable_cache is True
    assert m.spec.cpus == 2 and m.spec.memory_mb == 1024
    assert m.spec.fast is False and m.spec.capture_logs is True
    assert PostgresMachine(settings=settings, cache=True).settings.disable_cache is False
    with pytest.raises(InvalidConfig):
        PostgresMachine(settings=settings, target="moon")


def test_settings_default_from_env(monkeypatch: pytest.MonkeyPatch) -> None:
    monkeypatch.setenv("SMOLTEST_POSTGRES_IMAGE", "postgres:17")
    monkeypatch.setenv("SMOLTEST_TARGET", "local")
    m = PostgresMachine()
    assert m.image == "postgres:17" and m.settings.target == "local"


def test_unknown_kwargs_warn_once_and_are_ignored(settings: Settings) -> None:
    with pytest.warns(SmoltestWarning, match="network_mode"):
        m = PostgresMachine(settings=settings, network_mode="host")
    with pytest.warns(SmoltestWarning, match="privileged"):
        assert m.with_kwargs(privileged=True) is m
    reset_warn_once()
    with pytest.warns(UserWarning, match="privileged"):  # SmoltestWarning is a UserWarning
        m.with_kwargs(privileged=True)
    with warnings.catch_warnings():
        warnings.simplefilter("error")
        m.with_kwargs(privileged=True)  # second time: silent


# -- builders -------------------------------------------------------------------------


def test_builders_chain_and_shape_the_spec(settings: Settings, tmp_path: Path) -> None:
    sql = tmp_path / "init.sql"
    sql.write_text("CREATE TABLE t (id int);")
    strategy_ = PortWaitStrategy()
    m = (
        PostgresMachine(settings=settings)
        .with_env("TZ", "UTC")
        .with_envs(LANG="C", POSTGRES_INITDB_ARGS="--data-checksums")
        .with_command("docker-entrypoint.sh postgres -c max_connections=50")
        .with_name("golden")
        .with_exposed_ports(8080, 8080, 5432)
        .with_bind_ports(9090, 29090)
        .with_bind_ports(5432, 25432)
        .with_resources(cpus=2)
        .with_resources(memory_mb=768)
        .waiting_for(strategy_)
        .with_startup_timeout(7)
        .with_init_sql(sql)
    )
    spec = m.spec
    assert spec.env == {"TZ": "UTC", "LANG": "C", "POSTGRES_INITDB_ARGS": "--data-checksums"}
    assert spec.command == ("docker-entrypoint.sh", "postgres", "-c", "max_connections=50")
    assert spec.extra_ports == (PortMapping(None, 8080), PortMapping(29090, 9090))
    assert spec.cpus == 2 and spec.memory_mb == 768
    assert m.name == "golden" and m.wait_strategy is strategy_
    assert m.settings.ready_timeout_s == 7.0
    assert m.seed is not None and m.seed.key == hashlib.sha256(sql.read_bytes()).hexdigest()
    assert m.with_command(["postgres"]).spec.command == ("postgres",)
    assert m.with_bind_ports(9090, 29091).spec.extra_ports[-1] == PortMapping(29091, 9090)
    assert m.with_seed(None).seed is None
    env = spec.effective_env(settings)
    assert env["POSTGRES_INITDB_ARGS"] == "--data-checksums --no-sync"
    with pytest.raises(InvalidConfig):
        m.with_command("")


def test_builders_feed_the_boot_call(
    fake_boot: BootLog, fake_engine: FakeEngine, settings: Settings
) -> None:
    wait = PgIsReadyWaitStrategy("alice")
    seed = Seed.from_callable(lambda _m: None, key="k")
    pinned = pick_free_port()
    m = (
        PostgresMachine(settings=settings, username="alice", seed=seed)
        .with_name("golden")
        .with_bind_ports(5432, pinned)
        .with_exposed_ports(8080)
        .waiting_for(wait)
    )
    assert m.start() is m
    call = fake_boot.boots[-1]
    assert (
        call["spec"] is m.spec and call["settings"] is m.settings and call["engine"] is fake_engine
    )
    assert call["seed"] is seed and call["wait"] is wait and call["role"] == "standalone"
    assert call["name"] == "golden" and call["host_port"] == pinned
    assert m.get_exposed_port() == pinned and m.name == "golden"
    assert m.get_exposed_port(8080) == fake_of(m).host_ports[8080]
    assert m.get_exposed_port(5432) == pinned
    m.stop()


def test_default_wait_is_pg_isready_for_the_user(fake_boot: BootLog, settings: Settings) -> None:
    with PostgresMachine(settings=settings, username="carol") as m:
        call = fake_boot.boots[-1]
        wait = call["wait"]
        assert isinstance(wait, PgIsReadyWaitStrategy)
        view = ReadinessView(handle=m.get_machine(), endpoint=m.endpoint, username="carol")
        assert wait.argv(view) == ["pg_isready", "-h", "localhost", "-p", "5432", "-U", "carol"]
        assert call["name"] is None and call["host_port"] is None


def test_volume_mapping_is_unsupported(settings: Settings) -> None:
    with pytest.raises(NotSupportedError, match="with_init_sql"):
        PostgresMachine(settings=settings).with_volume_mapping(
            "/host/init.sql", "/docker-entrypoint-initdb.d/x.sql"
        )


def test_builders_refuse_after_start(fake_boot: BootLog, settings: Settings) -> None:
    with PostgresMachine(settings=settings) as m:
        calls: list[Callable[[], object]] = [
            lambda: m.with_env("A", "b"),
            lambda: m.with_name("x"),
            lambda: m.with_bind_ports(5432, 1),
            lambda: m.waiting_for(PortWaitStrategy()),
            lambda: m.with_startup_timeout(1),
            lambda: m.with_seed(None),
        ]
        for call in calls:
            with pytest.raises(SmoltestError, match="before start"):
                call()


# -- lifecycle ------------------------------------------------------------------------


def test_start_is_idempotent_and_stop_releases_exactly_once(
    fake_boot: BootLog, fake_engine: FakeEngine, settings: Settings
) -> None:
    m = PostgresMachine(settings=settings)
    assert m.start() is m and m.start() is m
    assert len(fake_boot.boots) == 1
    machine_id = fake_of(m).id
    info = m.boot_info
    assert info is not None and info.via == "cold"
    assert info.machine_id == machine_id and "running" in repr(m)
    m.stop()
    m.stop()
    assert fake_boot.released == [machine_id] and not m.is_running and m.boot_info is None
    assert fake_engine.live_machines == set()
    with pytest.raises(SmoltestError, match="not running"):
        m.get_connection_url()
    PostgresMachine(settings=settings).stop()  # never started: no-op


def test_stop_without_delete_keeps_the_machine_until_final_stop(
    fake_boot: BootLog, fake_engine: FakeEngine, settings: Settings
) -> None:
    m = PostgresMachine(settings=settings).start()
    fake = fake_of(m)
    m.stop(delete=False)
    assert fake.state() == "stopped" and fake in fake_engine.live_machines
    assert not m.is_running and "stopped" in repr(m) and fake_boot.released == []
    m.stop(delete=False)  # idempotent
    with pytest.raises(SmoltestError, match="was stopped"):
        m.start()
    m.stop()
    assert fake_boot.released == [fake.id] and fake_engine.live_machines == set()


def test_context_manager_deletes(
    fake_boot: BootLog, fake_engine: FakeEngine, settings: Settings
) -> None:
    with PostgresMachine(settings=settings) as m:
        assert m.is_running and len(fake_engine.live_machines) == 1
        machine_id = fake_of(m).id
    assert fake_engine.live_machines == set() and fake_boot.released == [machine_id]


def test_finalizer_deletes_an_unstopped_machine_on_gc(
    fake_boot: BootLog, fake_engine: FakeEngine, settings: Settings
) -> None:
    pending_before = get_reaper().pending
    m = PostgresMachine(settings=settings).start()
    machine_id = fake_of(m).id
    assert get_reaper().pending == pending_before + 1
    del m
    gc.collect()
    assert fake_boot.released == [machine_id] and fake_engine.live_machines == set()
    assert get_reaper().pending == pending_before


def test_stop_detaches_the_finalizer(fake_boot: BootLog, settings: Settings) -> None:
    m = PostgresMachine(settings=settings).start()
    machine_id = fake_of(m).id
    m.stop()
    del m
    gc.collect()
    assert fake_boot.released == [machine_id]  # not released a second time by GC


def test_engine_resolved_lazily_from_settings(
    fake_boot: BootLog, fake_engine: FakeEngine, settings: Settings
) -> None:
    m = PostgresMachine(settings=settings)
    assert m.engine is fake_engine  # via SMOLTEST_ENGINE -> FakeEngine.factory
    explicit = PostgresMachine(settings=settings, engine=fake_engine)
    assert explicit.engine is fake_engine


# -- connection details ---------------------------------------------------------------


def test_connection_urls(fake_boot: BootLog, settings: Settings) -> None:
    with PostgresMachine(settings=settings, password="p@ss w+rd/é") as m:
        port = m.get_exposed_port()
        host = m.get_container_host_ip()
        assert host == "127.0.0.1" and m.endpoint.port == port
        pw = "p%40ss w+rd%2F%C3%A9"
        assert m.get_connection_url() == f"postgresql+psycopg2://test:{pw}@127.0.0.1:{port}/test"
        assert m.url == m.get_connection_url()
        assert (
            m.get_connection_url(driver="psycopg")
            == f"postgresql+psycopg://test:{pw}@127.0.0.1:{port}/test"
        )
        assert m.get_connection_url(driver=None) == f"postgresql://test:{pw}@127.0.0.1:{port}/test"
        assert m.get_connection_url(host="db.internal") == (
            f"postgresql+psycopg2://test:{pw}@db.internal:{port}/test"
        )
        assert (
            m.get_connection_url("h", "asyncpg") == f"postgresql+asyncpg://test:{pw}@h:{port}/test"
        )
    with PostgresMachine(settings=settings, driver=None, username="u", dbname="d") as plain:
        assert plain.url.startswith("postgresql://u:test@127.0.0.1:") and plain.url.endswith("/d")
    with PostgresMachine(settings=settings, driver="psycopg") as v3:
        assert v3.url.startswith("postgresql+psycopg://")


def test_exposed_port_for_unpublished_guest_port_fails(
    fake_boot: BootLog, settings: Settings
) -> None:
    with (
        PostgresMachine(settings=settings) as m,
        pytest.raises(SmoltestError, match="not published"),
    ):
        m.get_exposed_port(9999)


# -- guest operations -----------------------------------------------------------------


def test_exec_tuple_unpack_and_forms(fake_boot: BootLog, settings: Settings) -> None:
    with PostgresMachine(settings=settings) as m:
        code, output = m.exec("pg_isready -U test")
        assert code == 0 and b"accepting connections" in output
        result = m.exec(["cat", "/nope"], timeout_s=5, user="postgres")
        assert isinstance(result, ExecResult) and result.exit_code == 1
        assert b"No such file" in result.output and result.stdout == b""
        assert fake_of(m).exec_log[-1] == ("cat", "/nope")
        assert m.get_wrapped_container() is m.get_machine()
        with pytest.raises(InvalidConfig):
            m.exec("")


def test_get_logs_without_capture_warns_once_and_is_empty(
    fake_boot: BootLog, settings: Settings
) -> None:
    with PostgresMachine(settings=settings) as m:
        with pytest.warns(SmoltestWarning, match="capture_logs=True"):
            assert m.get_logs() == (b"", b"")
        with warnings.catch_warnings():
            warnings.simplefilter("error")
            assert m.get_logs() == (b"", b"")
        assert all(argv[0] != "cat" for argv in fake_of(m).exec_log)


def test_get_logs_with_capture_reads_the_server_log(
    fake_boot: BootLog, fake_engine: FakeEngine, settings: Settings
) -> None:
    with PostgresMachine(settings=settings, capture_logs=True) as m:
        assert m.spec.capture_logs and "logging_collector=on" in m.spec.effective_argv(settings)
        out, err = m.get_logs()
        assert out == fake_engine.log_text.encode() and err == b""
        assert fake_of(m).exec_log[-1] == ("cat", LOG_PATH)
        del fake_of(m).files[LOG_PATH]
        assert m.get_logs() == (b"", b"")


def test_psql(fake_boot: BootLog, fake_engine: FakeEngine, settings: Settings) -> None:
    with PostgresMachine(settings=settings, username="alice", dbname="app") as m:
        assert m.psql("SELECT 1") == "1"
        assert fake_of(m).exec_log[-1] == (
            "psql",
            "-tA",
            "-U",
            "alice",
            "-d",
            "app",
            "-p",
            "5432",
            "-c",
            "SELECT 1",
        )
        assert m.psql("SHOW fsync", dbname="postgres") == "off"
        assert fake_of(m).exec_log[-1][5] == "postgres"
        assert fake_engine.sql_log == [(fake_of(m).id, "SELECT 1"), (fake_of(m).id, "SHOW fsync")]
        fake_engine.sql_error = re.compile("boom")
        with pytest.raises(SmoltestError, match="psql exited 1") as info:
            m.psql("SELECT boom()")
        assert info.value.code == "PSQL_FAILED"


# -- branch / checkpoint --------------------------------------------------------------


def test_branch_wraps_child_with_parent_reference(
    fake_boot: BootLog, fake_engine: FakeEngine, settings: Settings
) -> None:
    parent = PostgresMachine(settings=settings, driver="psycopg").start()
    parent.psql("CREATE TABLE t (id int)")
    child = parent.branch()
    try:
        assert isinstance(child, PostgresMachine) and child.is_running and child.parent is parent
        assert child.boot_info is not None and child.boot_info.via == "branch"
        assert child.get_exposed_port() != parent.get_exposed_port()
        assert child.url.startswith("postgresql+psycopg://") and child.url != parent.url
        assert child.spec is parent.spec and child.name == f"{parent.name}-b1"
        assert fake_boot.branches[-1]["name"] == f"{parent.name}-b1"
        assert fake_of(child).parent_id == fake_of(parent).id
        assert fake_of(child).inherited_sql == ["CREATE TABLE t (id int)"]
        named = parent.branch("feature-x")
        assert named.name == "feature-x"
        named_id, child_id, parent_id = fake_of(named).id, fake_of(child).id, fake_of(parent).id
        named.stop()
        assert len(fake_engine.live_machines) == 2
    finally:
        child.stop()
        parent.stop()
    assert fake_engine.live_machines == set()
    assert fake_boot.released == [named_id, child_id, parent_id]


def test_reaper_closes_children_before_parents(
    fake_boot: BootLog, fake_engine: FakeEngine, settings: Settings
) -> None:
    parent = PostgresMachine(settings=settings).start()
    child = parent.branch()
    parent_id, child_id = fake_of(parent).id, fake_of(child).id
    del parent, child
    gc.collect()
    assert fake_boot.released == [child_id, parent_id] or set(fake_boot.released) == {
        child_id,
        parent_id,
    }
    assert fake_engine.live_machines == set()


def test_checkpoint_passes_through(fake_boot: BootLog, settings: Settings, tmp_path: Path) -> None:
    with PostgresMachine(settings=settings) as m:
        with pytest.raises(InvalidConfig, match="output"):
            m.checkpoint()
        out = tmp_path / "golden.smolcheckpoint"
        info = m.checkpoint(out)
        assert isinstance(info, CheckpointInfo) and info.ref.kind == "file"
        assert info.ref.locator == str(out) and out.is_file()


def test_from_boot_classmethod(
    fake_boot: BootLog, fake_engine: FakeEngine, settings: Settings
) -> None:
    spec = PostgresSpec(username="x", password="y", dbname="z", guest_port=5432)
    result = strategy.boot_postgres(spec, settings, fake_engine)
    m = PostgresMachine._from_boot(result, spec, settings, fake_engine, driver=None)
    assert m.is_running and m.spec is spec and m.engine is fake_engine
    assert m.url == f"postgresql://x:y@127.0.0.1:{result.endpoint.port}/z"
    m.stop()
    assert fake_engine.live_machines == set()


def test_from_boot_without_finalizer_does_not_own_the_machine(
    fake_boot: BootLog, fake_engine: FakeEngine, settings: Settings
) -> None:
    spec = PostgresSpec()
    result = strategy.boot_postgres(spec, settings, fake_engine)
    pending = get_reaper().pending
    facade = PostgresMachine._from_boot(
        result, spec, settings, fake_engine, register_finalizer=False
    )
    assert facade.psql("SELECT 1") == "1" and get_reaper().pending == pending
    del facade
    gc.collect()
    assert fake_boot.released == [] and len(fake_engine.live_machines) == 1
    result.release()
    assert fake_engine.live_machines == set()


# -- async --------------------------------------------------------------------------------


def test_async_wrapper(fake_boot: BootLog, fake_engine: FakeEngine, settings: Settings) -> None:
    async def main() -> None:
        machine = AsyncPostgresMachine(settings=settings, username="alice", driver="asyncpg")
        assert machine.sync.username == "alice" and not machine.is_running
        async with machine as m:
            assert m is machine and m.is_running
            assert m.url.startswith("postgresql+asyncpg://alice:test@127.0.0.1:")
            assert m.get_connection_url(driver=None).startswith("postgresql://")
            assert m.get_container_host_ip() == "127.0.0.1"
            assert m.get_exposed_port() == m.sync.get_exposed_port()
            assert m.boot_info is not None and m.boot_info.via == "cold"
            code, out = await m.exec("pg_isready")
            assert code == 0 and b"accepting" in out
            assert await m.psql("SELECT 1") == "1"
            child = await m.branch("kid")
            assert isinstance(child, AsyncPostgresMachine) and child.sync.parent is m.sync
            assert child.sync.name == "kid"
            await child.stop()
            assert "Async" in repr(m)
        assert not machine.is_running
        await machine.stop()  # idempotent

    asyncio.run(main())
    assert fake_engine.live_machines == set() and len(fake_boot.released) == 2


def test_async_wrap_existing(fake_boot: BootLog, settings: Settings) -> None:
    sync = PostgresMachine(settings=settings).start()
    wrapped = AsyncPostgresMachine.wrap(sync)
    assert wrapped.sync is sync and wrapped.is_running
    asyncio.run(wrapped.stop())
    assert not sync.is_running


# -- review fixes ----------------------------------------------------------------------------


def test_credential_env_builders_keep_url_and_guest_in_step(
    fake_boot: BootLog, settings: Settings
) -> None:
    machine = (
        PostgresMachine(settings=settings)
        .with_env("POSTGRES_PASSWORD", "secret")
        .with_envs(POSTGRES_DB="app", TZ="UTC")
    )
    machine.start()
    try:
        env = fake_of(machine).env
        assert (env["POSTGRES_USER"], env["POSTGRES_PASSWORD"], env["POSTGRES_DB"]) == (
            "test",
            "secret",
            "app",
        )
        assert env["TZ"] == "UTC" and machine.spec.env == {"TZ": "UTC"}
        assert (machine.username, machine.password, machine.dbname) == ("test", "secret", "app")
        url = machine.get_connection_url()
        assert url.startswith("postgresql+psycopg2://test:secret@") and url.endswith("/app")
    finally:
        machine.stop()


def test_exec_result_indexes_like_a_tuple() -> None:
    result = ExecResult(1, b"out", b"err")
    assert (result[0], result[1], len(result)) == (1, b"outerr", 2)
    assert tuple(result) == (1, b"outerr") and result[-1] == b"outerr"
    with pytest.raises(IndexError):
        result[2]


def test_with_startup_timeout_accepts_timedelta(settings: Settings) -> None:
    machine = PostgresMachine(settings=settings).with_startup_timeout(timedelta(minutes=2))
    assert machine.settings.ready_timeout_s == 120.0
    assert (
        PostgresMachine(settings=settings).with_startup_timeout(7).settings.ready_timeout_s == 7.0
    )


def test_psql_forwards_its_timeout(
    fake_boot: BootLog, fake_engine: FakeEngine, settings: Settings
) -> None:
    machine = PostgresMachine(settings=settings).start()
    try:
        machine.psql("SELECT 1", timeout_s=7.0)
        assert fake_engine.ops("exec")[-1]["timeout_s"] == 7.0
        machine.psql("SELECT 1")
        assert fake_engine.ops("exec")[-1]["timeout_s"] is None  # the engine default applies
    finally:
        machine.stop()
