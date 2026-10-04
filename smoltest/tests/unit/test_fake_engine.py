"""The FakeEngine behaves like the real thing where the ladder relies on it."""

from __future__ import annotations

import json
import socket
from pathlib import Path
from typing import Any

import pytest

from smoltest._ports import is_port_free, pick_free_port
from smoltest.boot.spec import LOG_PATH
from smoltest.errors import BootError, InvalidConfig, NotSupportedError, SmoltestError
from smoltest.testing import DEFAULT_LOG_PATH, FakeEngine, FakeMachine, RelayBridge
from smoltest.transport.base import (
    Bridge,
    CheckpointRef,
    Engine,
    MachineHandle,
    MachineSpec,
    PortMapping,
    StaticBridge,
)


def spec(port: int | None = None, image: str = "postgres:16", **kw: Any) -> MachineSpec:
    ports = (PortMapping(port, 5432),)
    return MachineSpec(image=image, argv=("docker-entrypoint.sh", "postgres"), ports=ports, **kw)


def connect(port: int) -> socket.socket:
    s = socket.create_connection(("127.0.0.1", port), timeout=2)
    s.settimeout(0.2)
    return s


def test_protocol_conformance(fake_engine: FakeEngine) -> None:
    assert isinstance(fake_engine, Engine)
    m = fake_engine.create(spec(), "local")
    assert isinstance(m, MachineHandle)
    assert isinstance(fake_engine.open_tunnel(m.id, 5432, "local"), Bridge)
    m.delete()


def test_create_binds_a_real_listener_that_accepts_and_stays_silent(
    fake_engine: FakeEngine,
) -> None:
    m = fake_engine.create(spec(), "local")
    assert isinstance(m, FakeMachine)
    host, port = m.host_port(5432)
    assert host == "127.0.0.1" and not is_port_free(port)
    with connect(port) as s, pytest.raises(TimeoutError):
        s.recv(1, socket.MSG_PEEK)  # no EOF, no bytes: the probe sees a live service
    assert m.ready() and m.state() == "running"
    assert fake_engine.ops("create")[0]["target"] == "local"
    m.delete()
    assert is_port_free(port)
    assert m.state() == "deleted" and m not in fake_engine.live_machines


def test_create_pins_or_picks_ports_and_rejects_busy(fake_engine: FakeEngine) -> None:
    pinned = pick_free_port()
    m = fake_engine.create(spec(pinned), "local")
    assert m.host_port(5432) == ("127.0.0.1", pinned)
    with pytest.raises(BootError, match="port in use"):
        fake_engine.create(spec(pinned), "local")
    assert len(fake_engine.live_machines) == 1
    m.delete()


def test_exec_dispatch(fake_engine: FakeEngine) -> None:
    pg = fake_engine.create(spec(), "local")
    other = fake_engine.create(spec(image="nginx:1.27-alpine"), "local")
    assert isinstance(pg, FakeMachine)
    assert pg.exec(["pg_isready", "-U", "test"]).exit_code == 0
    assert other.exec(["pg_isready"]).exit_code == 2
    out = pg.exec(["psql", "-U", "test", "-tA", "-c", "SELECT 1"])
    assert out.ok and out.text.strip() == "1"
    assert pg.sql_log == ["SELECT 1"] and fake_engine.sql_log == [(pg.id, "SELECT 1")]
    assert pg.exec(["psql", "-tA", "-c", "SHOW fsync"]).text.strip() == "off"
    fake_engine.show_values["max_connections"] = "100"
    assert pg.exec(["psql", "-c", "show max_connections;"]).text.strip() == "100"
    pg.sql_responses["SELECT count(*) FROM t"] = "42"
    assert pg.exec(["psql", "-c", "SELECT count(*) FROM t"]).text.strip() == "42"
    assert other.exec(["psql", "-c", "SELECT 1"]).exit_code == 2
    assert pg.exec(["date", "-s", "@1700000000"]).ok and pg.clock_sets == ["@1700000000"]
    log = pg.exec(["cat", fake_engine.log_path])
    assert log.ok and b"ready to accept connections" in log.stdout
    assert pg.exec(["cat", "/nope"]).exit_code == 1
    assert pg.exec(["frobnicate"]).exit_code == 127
    assert pg.exec([]).exit_code == 127
    pg.write_file("/tmp/x", b"hi")
    assert pg.read_file("/tmp/x") == b"hi"
    with pytest.raises(FileNotFoundError):
        pg.read_file("/tmp/y")
    fake_engine.exec_handlers["uname"] = lambda _m, argv: pg.exec(["true"]) if argv else None
    assert pg.exec(["uname"]).ok
    assert [op for op, _ in fake_engine.calls].count("exec") >= 10
    pg.delete()
    other.delete()


def test_checkpoint_writes_shape_and_restore_rebinds_same_port(
    fake_engine: FakeEngine, tmp_path: Path
) -> None:
    m = fake_engine.create(spec(env={"POSTGRES_USER": "u"}), "local")
    assert isinstance(m, FakeMachine)
    port = m.host_port(5432)[1]
    m.exec(["psql", "-c", "CREATE TABLE t()"])
    out = tmp_path / "k" / f"{port}.smolcheckpoint"
    info = m.checkpoint(str(out), store=str(tmp_path / "store"))
    assert info.ref == CheckpointRef("file", str(out)) and info.size_bytes == out.stat().st_size
    shape = json.loads(out.read_text())
    assert shape["image"] == "postgres:16" and shape["ports"] == [[port, 5432]]
    assert shape["services"] == ["postgres"] and shape["ram_bytes"] == 512 * 2**20
    assert shape["env"]["POSTGRES_USER"] == "u" and shape["argv"][0] == "docker-entrypoint.sh"
    assert list((tmp_path / "store").glob("*.chunk"))
    with pytest.raises(InvalidConfig, match="already exists"):
        m.checkpoint(str(out))
    with pytest.raises(InvalidConfig, match="output"):
        m.checkpoint(None)
    with pytest.raises(BootError, match="port in use"):
        fake_engine.restore_checkpoint(info.ref, "clash", "local")
    m.delete()
    r = fake_engine.restore_checkpoint(info.ref, "restored", "local")
    assert isinstance(r, FakeMachine)
    assert r.host_port(5432) == ("127.0.0.1", port) and r.name == "restored"
    assert (
        r.branchable is True
        and "postgres" in r.services
        and r.inherited_sql == ["CREATE TABLE t()"]
    )
    assert r.exec(["pg_isready"]).ok
    r.delete()


def test_checkpoint_requires_branchable_and_support(
    fake_engine: FakeEngine, tmp_path: Path
) -> None:
    m = fake_engine.create(spec(branchable=False), "local")
    with pytest.raises(NotSupportedError):
        m.checkpoint(str(tmp_path / "c"))
    with pytest.raises(NotSupportedError):
        m.branch("child")
    m.delete()
    no = FakeEngine(supports_checkpoints=False, supports_branch=False)
    m2 = no.create(spec(), "local")
    with pytest.raises(NotSupportedError):
        m2.checkpoint(str(tmp_path / "c"))
    with pytest.raises(NotSupportedError):
        m2.branch("child")
    m2.delete()
    assert not no.live_machines


def test_restore_errors(fake_engine: FakeEngine, tmp_path: Path) -> None:
    bad = tmp_path / "bad.smolcheckpoint"
    bad.write_text("{not json")
    with pytest.raises(BootError, match="unreadable"):
        fake_engine.restore_checkpoint(CheckpointRef("file", str(bad)), "x", "local")
    bad.write_text(json.dumps({"format": 99}))
    with pytest.raises(BootError, match="format"):
        fake_engine.restore_checkpoint(CheckpointRef("file", str(bad)), "x", "local")
    with pytest.raises(SmoltestError):
        fake_engine.restore_checkpoint(CheckpointRef("cloud", "ckpt-404"), "x", "cloud")


def test_branch_inherits_services_with_new_ports(fake_engine: FakeEngine) -> None:
    golden = fake_engine.create(spec(), "local")
    assert isinstance(golden, FakeMachine)
    golden.exec(["psql", "-c", "CREATE TABLE t()"])
    auto = golden.branch("auto")
    pinned_port = pick_free_port(exclude=[golden.host_port(5432)[1], auto.host_port(5432)[1]])
    pinned = golden.branch("pinned", [PortMapping(pinned_port, 5432)])
    assert isinstance(auto, FakeMachine) and isinstance(pinned, FakeMachine)
    assert auto.host_port(5432)[1] != golden.host_port(5432)[1]
    assert pinned.host_port(5432) == ("127.0.0.1", pinned_port)
    for child in (auto, pinned):
        assert child.services == {"postgres"} and child.parent_id == golden.id
        assert child.exec(["pg_isready"]).ok and child.inherited_sql == ["CREATE TABLE t()"]
        assert child.branchable is True and child.argv == golden.argv
        assert not is_port_free(child.host_port(5432)[1])
    with pytest.raises(InvalidConfig):
        golden.branch("bad", [PortMapping(None, 9999)])
    for m in (auto, pinned, golden):
        m.delete()
    assert not fake_engine.live_machines


def test_fail_next_and_latency(fake_engine: FakeEngine) -> None:
    fake_engine.fail_next("create", BootError("simulated", stage="create"))
    with pytest.raises(BootError, match="simulated"):
        fake_engine.create(spec(), "local")
    m = fake_engine.create(spec(), "local")  # only the next call fails
    fake_engine.fail_next("branch", NotSupportedError("nope"))
    with pytest.raises(NotSupportedError):
        m.branch("c")
    fake_engine.latency["checkpoint"] = 0.01
    fake_engine.fail_next("checkpoint", RuntimeError("disk full"))
    with pytest.raises(RuntimeError):
        m.checkpoint("/dev/null/never")
    assert [op for op, _ in fake_engine.calls] == ["create", "create", "branch", "checkpoint"]
    m.delete()


def test_cloud_target_tunnel_relays_to_listener(fake_engine: FakeEngine) -> None:
    m = fake_engine.create(spec(None), "cloud")
    assert isinstance(m, FakeMachine)
    with pytest.raises(NotSupportedError):
        m.host_port(5432)
    bridge = fake_engine.open_tunnel(m.id, 5432, "cloud")
    assert isinstance(bridge, RelayBridge)
    before = bridge.endpoint
    assert before is None
    ep = bridge.open()
    after = bridge.endpoint
    assert after == ep and ep.host == "127.0.0.1"
    assert bridge.open() == ep
    with connect(ep.port) as s:
        s.sendall(b"hello")
        with pytest.raises(TimeoutError):
            s.recv(1, socket.MSG_PEEK)
    bridge.close()
    assert bridge.endpoint is None and is_port_free(ep.port)
    assert ("tunnel.open", {"machine": m.id, "guest": 5432}) in fake_engine.calls
    assert ("tunnel.close", {"machine": m.id, "guest": 5432}) in fake_engine.calls
    info = m.checkpoint()
    assert info.ref.kind == "cloud"
    r = fake_engine.restore_checkpoint(info.ref, "r", "cloud")
    assert r.exec(["pg_isready"]).ok
    r.delete()
    m.delete()


def test_local_tunnel_is_static_bridge(fake_engine: FakeEngine) -> None:
    m = fake_engine.create(spec(), "local")
    bridge = fake_engine.open_tunnel(m.id, 5432, "local")
    assert isinstance(bridge, StaticBridge)
    assert bridge.endpoint is None
    assert bridge.open().port == m.host_port(5432)[1]
    bridge.close()
    assert bridge.endpoint is None
    with pytest.raises(SmoltestError):
        fake_engine.open_tunnel(m.id, 80, "local")
    with pytest.raises(SmoltestError):
        fake_engine.open_tunnel("fake-9999", 5432, "local")
    m.delete()


def test_export_and_prune_store(fake_engine: FakeEngine, tmp_path: Path) -> None:
    store = tmp_path / "store"
    m = fake_engine.create(spec(), "local")
    a = tmp_path / "a.smolcheckpoint"
    info = m.checkpoint(str(a), store=str(store))
    m.exec(["psql", "-c", "CREATE TABLE t()"])
    b = tmp_path / "b.smolcheckpoint"
    m.checkpoint(str(b), store=str(store))
    assert len(list(store.glob("*.chunk"))) == 2
    exported = tmp_path / "out" / "copy.smolcheckpoint"
    assert fake_engine.export_checkpoint(info.ref.locator, str(exported)) == a.stat().st_size
    assert exported.read_bytes() == a.read_bytes()
    with pytest.raises(SmoltestError):
        fake_engine.export_checkpoint(str(tmp_path / "missing"), str(exported))
    assert fake_engine.prune_checkpoint_store(str(store)) == 0
    a.unlink()
    assert fake_engine.prune_checkpoint_store(str(store)) == 1
    assert len(list(store.glob("*.chunk"))) == 1
    assert fake_engine.prune_checkpoint_store(str(tmp_path / "nostore")) == 0
    m.delete()


def test_connect_pause_resume_stop(fake_engine: FakeEngine) -> None:
    m = fake_engine.create(spec(), "local")
    assert fake_engine.connect(m.id, "local") is m
    m.pause()
    assert m.state() == "paused" and not m.ready()
    with pytest.raises(SmoltestError):
        m.exec(["true"])
    m.resume()
    m.wait_until_ready(1, 0.01)
    port = m.host_port(5432)[1]
    m.stop()
    assert is_port_free(port)
    m.delete()
    with pytest.raises(SmoltestError):
        fake_engine.connect(m.id, "local")


def test_leak_check_and_close_all() -> None:
    engine = FakeEngine()
    engine.create(spec(), "local")
    engine.create(spec(), "local")
    assert len(engine.live_machines) == 2
    engine.close_all()
    assert not engine.live_machines


def test_factory_and_availability() -> None:
    engine = FakeEngine(availability=(False, "KVM_UNAVAILABLE", "nope"), sdk_version="x")
    assert engine.local_availability() == (False, "KVM_UNAVAILABLE", "nope")
    assert engine.sdk_version() == "x"
    assert isinstance(FakeEngine.factory(), FakeEngine)
    FakeEngine.set_default(engine)
    try:
        assert FakeEngine.factory() is engine
    finally:
        FakeEngine.set_default(None)
    assert FakeEngine.factory() is not engine


def test_fake_log_path_matches_the_spec() -> None:
    assert DEFAULT_LOG_PATH == LOG_PATH == "/tmp/postgresql.log"
    assert FakeEngine().log_path == LOG_PATH
