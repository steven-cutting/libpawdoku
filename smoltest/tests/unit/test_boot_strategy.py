"""The boot ladder against the FakeEngine: cold, restore, seeding, cloud, cleanup, branch."""

from __future__ import annotations

import json
import re
import socket
import threading
from collections.abc import Iterator
from pathlib import Path
from typing import Any

import pytest

from smoltest._ports import LOOPBACK, is_port_free, pick_free_port
from smoltest.boot import strategy
from smoltest.boot.spec import PostgresSpec
from smoltest.boot.strategy import (
    TERMINATE_FOREIGN_BACKENDS_SQL,
    BootResult,
    boot_postgres,
    branch_from,
)
from smoltest.cache import CacheKey, CheckpointCache, CloudCheckpointIndex
from smoltest.cache.lock import FileLock
from smoltest.config import Settings
from smoltest.errors import (
    BootError,
    ExecError,
    NotSupportedError,
    ReadinessTimeout,
    SmoltestError,
    TargetUnavailable,
)
from smoltest.postgres import PostgresMachine, Seed
from smoltest.testing import FakeEngine, FakeMachine
from smoltest.transport.base import ExecOutcome, MachineSpec, PortMapping
from smoltest.transport.tunnel import TunnelBridge
from smoltest.wait.strategies import (
    EXEC_PROBE_TIMEOUT_S,
    LogMessageWaitStrategy,
    PortWaitStrategy,
    ReadinessTarget,
    WaitStrategy,
)

SPEC = PostgresSpec()


def fake_of(result: BootResult) -> FakeMachine:
    assert isinstance(result.handle, FakeMachine)
    return result.handle


def claim_files(settings: Settings) -> list[Path]:
    return sorted(settings.cache_dir.rglob("*.claim"))


def base_key(settings: Settings, engine: FakeEngine, spec: PostgresSpec = SPEC) -> CacheKey:
    return CacheKey.compute(spec, settings, "local", engine)


class RecordingWait(WaitStrategy):
    """Counts probes and remembers the SQL the fake had run at each one."""

    def __init__(self) -> None:
        self.probes = 0
        self.sql_seen: list[list[str]] = []

    def probe(self, target: ReadinessTarget) -> bool:
        self.probes += 1
        handle = target.handle
        assert isinstance(handle, FakeMachine)
        self.sql_seen.append(list(handle.sql_log))
        return True


def poison_variant(settings: Settings, key: CacheKey, port: int) -> None:
    """Rewrite a cached fake checkpoint so the restored machine runs no postgres."""
    checkpoint = CheckpointCache.open(settings).paths(key.key, port).checkpoint
    shape = json.loads(checkpoint.read_bytes())
    shape["services"] = []
    checkpoint.write_bytes(json.dumps(shape).encode())


@pytest.fixture
def listener() -> Iterator[socket.socket]:
    sock = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
    sock.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)
    yield sock
    sock.close()


# -- cold and restore ---------------------------------------------------------------------


def test_cold_boot_populates_the_cache(fake_engine: FakeEngine, settings: Settings) -> None:
    result = boot_postgres(SPEC, settings, fake_engine)
    try:
        info, fake = result.info, fake_of(result)
        key = base_key(settings, fake_engine)
        assert info.via == "cold" and info.target == "local" and info.populated
        assert info.cache_key == key.key and info.seeded_key is None and not info.seeded
        assert info.variant_port == result.endpoint.port == fake.host_ports[5432]
        assert info.machine_id == fake.id and not info.clock_resynced
        assert {"create", "tunnel", "wait", "populate"} <= set(info.timings)
        assert len(fake_engine.ops("create")) == 1 and len(fake_engine.ops("checkpoint")) == 1
        assert fake.clock_sets == []  # no resync on a cold boot
        cache = CheckpointCache.open(settings)
        paths = cache.paths(key.key, result.endpoint.port)
        assert paths.checkpoint.is_file() and paths.meta.is_file() and paths.claim.is_file()
        assert cache.inputs_path(key.key).is_file()
        assert result.claim is not None and result.claim.populated
        assert isinstance(result.backend, CheckpointCache)
        assert result.bridge.endpoint == result.endpoint
    finally:
        result.release()
    assert not paths.claim.exists() and paths.meta.is_file()
    bridge = result.bridge
    assert result.released and bridge.endpoint is None
    assert fake_engine.live_machines == set() and len(fake_engine.ops("delete")) == 1
    result.release()  # idempotent
    assert len(fake_engine.ops("delete")) == 1


def test_second_boot_restores_and_resyncs_the_clock(
    fake_engine: FakeEngine, settings: Settings
) -> None:
    first = boot_postgres(SPEC, settings, fake_engine)
    port = first.endpoint.port
    first.release()
    second = boot_postgres(SPEC, settings, fake_engine)
    try:
        info, fake = second.info, fake_of(second)
        assert info.via == "restore" and not info.populated and info.clock_resynced
        assert info.restored_key == info.cache_key and info.restored_from is not None
        assert info.restored_from.kind == "file"
        assert second.endpoint.port == port == info.variant_port
        assert len(fake.clock_sets) == 1 and fake.clock_sets[0].startswith("@")
        assert len(fake_engine.ops("create")) == 1 and len(fake_engine.ops("restore")) == 1
        assert len(fake_engine.ops("checkpoint")) == 1
        assert fake.exec_log[0][:2] == ("date", "-s")
        assert fake.exec_log[1][0] == "pg_isready"
    finally:
        second.release()
    assert fake_engine.live_machines == set()


def test_busy_variant_port_boots_a_new_variant(
    fake_engine: FakeEngine, settings: Settings, listener: socket.socket
) -> None:
    first = boot_postgres(SPEC, settings, fake_engine)
    port = first.endpoint.port
    first.release()
    listener.bind((LOOPBACK, port))
    listener.listen(1)
    second = boot_postgres(SPEC, settings, fake_engine)
    try:
        assert second.info.via == "cold" and second.endpoint.port != port
        key = base_key(settings, fake_engine)
        ports = {v.port for v in CheckpointCache.open(settings).variants(key.key)}
        assert ports == {port, second.endpoint.port}
        assert len(fake_engine.ops("restore")) == 0 and len(fake_engine.ops("create")) == 2
    finally:
        second.release()


def test_restore_failure_invalidates_and_falls_back_to_cold(
    fake_engine: FakeEngine, settings: Settings
) -> None:
    first = boot_postgres(SPEC, settings, fake_engine)
    old_port = first.endpoint.port
    first.release()
    fake_engine.fail_next("restore", SmoltestError("checkpoint rotten"))
    second = boot_postgres(SPEC, settings, fake_engine)
    try:
        assert second.info.via == "cold" and second.info.populated
        assert len(fake_engine.ops("restore")) == 1 and len(fake_engine.ops("create")) == 2
        key = base_key(settings, fake_engine)
        variants = CheckpointCache.open(settings).variants(key.key)
        assert [v.port for v in variants] == [second.endpoint.port]
        if second.endpoint.port != old_port:
            assert not CheckpointCache.open(settings).paths(key.key, old_port).checkpoint.exists()
    finally:
        second.release()
    assert claim_files(settings) == []


def test_restore_port_conflict_skips_without_invalidating(
    fake_engine: FakeEngine, settings: Settings, monkeypatch: pytest.MonkeyPatch
) -> None:
    first = boot_postgres(SPEC, settings, fake_engine)
    port = first.endpoint.port
    first.release()
    monkeypatch.setattr(strategy, "pick_free_port", lambda **kw: pick_free_port(exclude=[port]))
    fake_engine.fail_next("restore", BootError("port in use: 127.0.0.1:0", stage="bind"))
    second = boot_postgres(SPEC, settings, fake_engine)
    try:
        assert second.info.via == "cold"
        key = base_key(settings, fake_engine)
        ports = {v.port for v in CheckpointCache.open(settings).variants(key.key)}
        assert ports == {port, second.endpoint.port}  # the old variant survived
    finally:
        second.release()


def test_create_port_conflict_retries_once(
    fake_engine: FakeEngine,
    settings: Settings,
    listener: socket.socket,
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    busy = pick_free_port()
    listener.bind((LOOPBACK, busy))
    listener.listen(1)
    picks = iter([busy, pick_free_port(exclude=[busy])])
    monkeypatch.setattr(strategy, "pick_free_port", lambda **kw: next(picks))
    result = boot_postgres(SPEC, settings, fake_engine)
    try:
        assert result.info.via == "cold" and result.endpoint.port != busy
        assert len(fake_engine.ops("create")) == 2
        key = base_key(settings, fake_engine)
        ports = {v.port for v in CheckpointCache.open(settings).variants(key.key)}
        assert ports == {result.endpoint.port}
    finally:
        result.release()
    assert claim_files(settings) == []


def test_pinned_host_port_is_used_and_restored(fake_engine: FakeEngine, settings: Settings) -> None:
    port = pick_free_port()
    first = boot_postgres(SPEC, settings, fake_engine, host_port=port, name="golden-1")
    try:
        assert first.endpoint.port == port and first.handle.name == "golden-1"
    finally:
        first.release()
    second = boot_postgres(SPEC, settings, fake_engine, host_port=port)
    try:
        assert second.info.via == "restore" and second.endpoint.port == port
    finally:
        second.release()


def test_disable_cache_writes_nothing(fake_engine: FakeEngine, settings: Settings) -> None:
    result = boot_postgres(SPEC, settings.replace(disable_cache=True), fake_engine)
    try:
        assert result.info.via == "cold" and result.info.cache_key is None
        assert result.claim is None and result.backend is None and not result.info.populated
        assert fake_engine.ops("checkpoint") == []
        assert not (settings.cache_dir / "postgres").exists()
    finally:
        result.release()
    again = boot_postgres(SPEC, settings.replace(disable_cache=True), fake_engine)
    try:
        assert again.info.via == "cold" and len(fake_engine.ops("create")) == 2
    finally:
        again.release()


def test_engine_without_checkpoints_boots_cold(settings: Settings) -> None:
    engine = FakeEngine(supports_checkpoints=False)
    result = boot_postgres(SPEC, settings, engine)
    try:
        assert result.info.via == "cold" and result.claim is None
    finally:
        result.release()
    assert engine.live_machines == set()


def test_custom_wait_strategy_is_used(fake_engine: FakeEngine, settings: Settings) -> None:
    result = boot_postgres(SPEC, settings, fake_engine, wait=PortWaitStrategy())
    try:
        fake = fake_of(result)
        assert fake.accepted_connections(5432) >= 1
        assert all(argv[0] != "pg_isready" for argv in fake.exec_log)
    finally:
        result.release()


# -- seeding ------------------------------------------------------------------------------


def test_seed_miss_on_base_hit_restores_seeds_and_populates(
    fake_engine: FakeEngine, settings: Settings
) -> None:
    plain = boot_postgres(SPEC, settings, fake_engine)
    plain.release()
    seed = Seed.from_sql("CREATE TABLE t (id int);")
    key = base_key(settings, fake_engine)
    seeded_key = key.with_seed(seed.key)
    result = boot_postgres(SPEC, settings, fake_engine, seed=seed)
    try:
        info, fake = result.info, fake_of(result)
        assert info.via == "restore" and info.seeded and info.populated
        assert info.restored_key == key.key and info.seeded_key == seeded_key.key
        assert fake.sql_log == ["CREATE TABLE t (id int)", TERMINATE_FOREIGN_BACKENDS_SQL]
        assert len(fake_engine.ops("checkpoint")) == 2  # base on the first boot, seeded now
        cache = CheckpointCache.open(settings)
        assert [v.port for v in cache.variants(seeded_key.key)] == [result.endpoint.port]
        assert result.seed_claim is not None and result.seed_claim.populated
        assert result.claim is not None and result.claim.key == key.key
        assert {"seed", "populate_seed"} <= set(info.timings)
    finally:
        result.release()
    assert claim_files(settings) == []
    hit = boot_postgres(SPEC, settings, fake_engine, seed=seed)
    try:
        info, fake = hit.info, fake_of(hit)
        assert info.via == "restore" and not info.seeded and not info.populated
        assert info.restored_key == seeded_key.key
        assert fake.sql_log == [] and "CREATE TABLE t (id int)" in fake.inherited_sql
        assert hit.seed_claim is None and hit.claim is not None
        assert hit.claim.key == seeded_key.key
    finally:
        hit.release()
    assert fake_engine.live_machines == set()


def test_cold_boot_with_seed_populates_base_and_seeded_variants(
    fake_engine: FakeEngine, settings: Settings
) -> None:
    applied: list[str] = []

    def apply(machine: PostgresMachine) -> None:
        applied.append(machine.psql("CREATE TABLE s (id int)"))
        assert machine.is_running and machine.boot_info is not None

    seed = Seed.from_callable(apply, key="schema-v1")
    result = boot_postgres(SPEC, settings, fake_engine, seed=seed)
    try:
        assert result.info.via == "cold" and result.info.seeded and result.info.populated
        assert applied == [""]
        assert len(fake_engine.ops("checkpoint")) == 2
        key = base_key(settings, fake_engine)
        cache = CheckpointCache.open(settings)
        assert len(cache.variants(key.key)) == 1
        assert len(cache.variants(key.with_seed("schema-v1").key)) == 1
        assert len(fake_engine.live_machines) == 1  # the seeding facade owns nothing
    finally:
        result.release()
    assert fake_engine.live_machines == set()


def test_seed_failure_tears_everything_down(fake_engine: FakeEngine, settings: Settings) -> None:
    def explode(machine: PostgresMachine) -> None:
        raise RuntimeError("migration failed")

    seed = Seed.from_callable(explode, key="bad")
    with pytest.raises(BootError, match=r"\[seed\] RuntimeError: migration failed") as info:
        boot_postgres(SPEC, settings, fake_engine, seed=seed)
    assert info.value.stage == "seed" and isinstance(info.value.cause, RuntimeError)
    assert fake_engine.live_machines == set() and len(fake_engine.ops("delete")) == 1
    assert claim_files(settings) == []


def test_sql_seed_failure_is_a_boot_error(fake_engine: FakeEngine, settings: Settings) -> None:
    fake_engine.sql_error = re.compile("DROP")
    seed = Seed.from_sql("DROP TABLE nope;")
    with pytest.raises(BootError, match=r"\[seed\] .*psql exited") as info:
        boot_postgres(SPEC, settings, fake_engine, seed=seed)
    assert info.value.stage == "seed"
    assert fake_engine.live_machines == set()


# -- failure cleanup ----------------------------------------------------------------------


def test_tunnel_failure_deletes_machine_and_releases_claim(
    fake_engine: FakeEngine, settings: Settings
) -> None:
    fake_engine.fail_next("open_tunnel", SmoltestError("no tunnel for you"))
    with pytest.raises(BootError, match=r"\[tunnel\] SmoltestError: no tunnel") as info:
        boot_postgres(SPEC, settings, fake_engine)
    assert info.value.stage == "tunnel"
    assert fake_engine.live_machines == set() and len(fake_engine.ops("delete")) == 1
    assert claim_files(settings) == [] and fake_engine.ops("checkpoint") == []
    key = base_key(settings, fake_engine)
    assert not CheckpointCache.open(settings).variants(key.key)


def test_wait_timeout_deletes_machine_and_releases_claim(
    fake_engine: FakeEngine, settings: Settings
) -> None:
    quick = settings.replace(ready_timeout_s=0.2, poll_interval_s=0.01)
    spec = PostgresSpec(image="alpine:3")  # the fake runs no postgres service in it
    with pytest.raises(ReadinessTimeout) as info:
        boot_postgres(spec, quick, fake_engine)
    assert info.value.stage == "wait" and isinstance(info.value, BootError)
    assert fake_engine.live_machines == set() and len(fake_engine.ops("delete")) == 1
    assert claim_files(settings) == [] and fake_engine.ops("checkpoint") == []


def test_create_failure_is_wrapped(fake_engine: FakeEngine, settings: Settings) -> None:
    fake_engine.fail_next("create", SmoltestError("quota exceeded"))
    with pytest.raises(BootError, match=r"\[create\] SmoltestError: quota") as info:
        boot_postgres(SPEC, settings, fake_engine)
    assert info.value.stage == "create"
    assert claim_files(settings) == [] and fake_engine.live_machines == set()


def test_populate_failure_cleans_up(fake_engine: FakeEngine, settings: Settings) -> None:
    fake_engine.fail_next("checkpoint", SmoltestError("disk full"))
    with pytest.raises(BootError, match=r"\[populate\]") as info:
        boot_postgres(SPEC, settings, fake_engine)
    assert info.value.stage == "populate"
    assert fake_engine.live_machines == set() and claim_files(settings) == []


def test_target_unavailable_propagates_unchanged(settings: Settings) -> None:
    engine = FakeEngine(availability=(False, "KVM_UNAVAILABLE", "no /dev/kvm"))
    with pytest.raises(TargetUnavailable) as info:
        boot_postgres(SPEC, settings.replace(target="auto"), engine)
    assert info.value.code == "KVM_UNAVAILABLE" and "smoltest doctor" in str(info.value)
    assert engine.calls == []


# -- cloud --------------------------------------------------------------------------------


def test_cloud_refuses_a_pinned_host_port(fake_engine: FakeEngine, settings: Settings) -> None:
    """Booting on another port than the one asked for would break the clients configured for it."""
    cloud = settings.replace(target="cloud")
    with pytest.raises(NotSupportedError, match=r"fixed host port .* cloud"):
        boot_postgres(SPEC, cloud, fake_engine, host_port=24000)
    assert fake_engine.ops("create") == [] and fake_engine.live_machines == set()
    machine = PostgresMachine(settings=cloud, engine=fake_engine).with_bind_ports(5432, 24000)
    with pytest.raises(NotSupportedError, match="cloud"):
        machine.start()
    assert fake_engine.ops("create") == [] and fake_engine.live_machines == set()


def test_cloud_uses_index_tunnel_and_safety_net(
    fake_engine: FakeEngine, settings: Settings
) -> None:
    cloud = settings.replace(target="cloud")
    fake_engine.tunnel_bridge_factory = TunnelBridge
    result = boot_postgres(SPEC, cloud, fake_engine)
    try:
        info = result.info
        assert info.via == "cold" and info.target == "cloud" and info.variant_port is None
        assert isinstance(result.bridge, TunnelBridge)
        assert result.endpoint.port != fake_of(result).host_ports[5432]  # relayed, not direct
        assert not is_port_free(result.endpoint.port)
        created: MachineSpec = fake_engine.ops("create")[0]["spec"]
        assert created.auto_stop_seconds == cloud.cloud_auto_stop_seconds
        assert created.ttl_seconds == cloud.cloud_ttl_seconds
        assert created.ports[0].host is None
        assert isinstance(result.backend, CloudCheckpointIndex)
        assert result.claim is not None and result.claim.kind == "cloud"
        assert result.claim.port is None and result.claim.populated
        index_files = list((settings.cache_dir / "cloud").rglob("index.json"))
        assert len(index_files) == 1
        assert not (settings.cache_dir / "postgres").exists()
    finally:
        result.release()
    assert fake_engine.ops("tunnel.close")
    second = boot_postgres(SPEC, cloud, fake_engine)
    try:
        assert second.info.via == "restore" and second.info.clock_resynced
        assert second.info.restored_from is not None
        assert second.info.restored_from.kind == "cloud"
        assert fake_engine.ops("restore")[0]["target"] == "cloud"
    finally:
        second.release()
    assert fake_engine.live_machines == set()


def test_cloud_restore_failure_drops_the_index_entry(
    fake_engine: FakeEngine, settings: Settings
) -> None:
    cloud = settings.replace(target="cloud")
    first = boot_postgres(SPEC, cloud, fake_engine)
    first.release()
    fake_engine.fail_next("restore", SmoltestError("checkpoint expired"))
    second = boot_postgres(SPEC, cloud, fake_engine)
    try:
        assert second.info.via == "cold"
        key = CacheKey.compute(SPEC, cloud, "cloud", fake_engine)
        index = CloudCheckpointIndex.open(cloud)
        assert index.get(key.key) == "ckpt-2"
    finally:
        second.release()


# -- branch_from --------------------------------------------------------------------------


def test_branch_from_pins_a_fresh_port_and_inherits(
    fake_engine: FakeEngine, settings: Settings
) -> None:
    golden = boot_postgres(SPEC, settings, fake_engine, role="golden")
    fake_of(golden).exec(["psql", "-c", "CREATE TABLE g (id int)"])
    child = branch_from(golden, "child-1", settings, fake_engine)
    try:
        info, fake = child.info, fake_of(child)
        assert info.via == "branch" and info.target == "local" and info.machine_id == fake.id
        assert info.cache_key == golden.info.cache_key and info.variant_port == child.endpoint.port
        assert child.endpoint.port != golden.endpoint.port
        assert fake.parent_id == golden.handle.id and fake.services == {"postgres"}
        assert fake.inherited_sql == ["CREATE TABLE g (id int)"] and fake.name == "child-1"
        assert child.claim is None and child.spec is golden.spec
        assert {"branch", "tunnel", "wait"} <= set(info.timings)
        assert not is_port_free(child.endpoint.port)
        ports = fake_engine.ops("branch")[0]["ports"]
        assert [p.guest for p in ports] == [5432] and ports[0].host == child.endpoint.port
    finally:
        child.release()
        golden.release()
    assert fake_engine.live_machines == set()


def test_branch_from_retries_once_on_port_conflict(
    fake_engine: FakeEngine,
    settings: Settings,
    listener: socket.socket,
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    busy = pick_free_port()
    listener.bind((LOOPBACK, busy))
    listener.listen(1)
    golden = boot_postgres(SPEC, settings, fake_engine)
    picks = iter([busy, pick_free_port(exclude=[busy, golden.endpoint.port])])
    monkeypatch.setattr(strategy, "pick_free_port", lambda **kw: next(picks))
    child = branch_from(golden, "child", settings, fake_engine)
    try:
        assert child.endpoint.port != busy and len(fake_engine.ops("branch")) == 2
    finally:
        child.release()
        golden.release()


def test_branch_from_passes_not_supported_through(
    fake_engine: FakeEngine, settings: Settings
) -> None:
    golden = boot_postgres(SPEC, settings, fake_engine)
    fake_engine.fail_next("branch", NotSupportedError("branching is off"))
    try:
        with pytest.raises(NotSupportedError):
            branch_from(golden, "child", settings, fake_engine)
        fake_engine.fail_next("branch", SmoltestError("other"))
        with pytest.raises(BootError, match=r"\[branch\] SmoltestError: other"):
            branch_from(golden, "child", settings, fake_engine)
        assert len(fake_engine.live_machines) == 1
    finally:
        golden.release()


def test_branch_from_cleans_up_when_the_tunnel_fails(
    fake_engine: FakeEngine, settings: Settings
) -> None:
    golden = boot_postgres(SPEC, settings, fake_engine)
    fake_engine.fail_next("open_tunnel", SmoltestError("tunnel down"))
    try:
        with pytest.raises(BootError, match=r"\[branch\] SmoltestError: tunnel down"):
            branch_from(golden, "child", settings, fake_engine)
        assert len(fake_engine.live_machines) == 1
        assert len(fake_engine.ops("delete")) == 1
    finally:
        golden.release()


def test_branch_from_extra_ports_get_fresh_host_ports(
    fake_engine: FakeEngine, settings: Settings
) -> None:
    spec = PostgresSpec(extra_ports=(PortMapping(None, 8080),))
    golden = boot_postgres(spec, settings, fake_engine)
    child = branch_from(golden, "child", settings, fake_engine)
    try:
        ports: list[Any] = fake_engine.ops("branch")[0]["ports"]
        assert [p.guest for p in ports] == [5432, 8080]
        assert len({p.host for p in ports}) == 2
        assert fake_of(child).host_port(8080)[1] == ports[1].host
    finally:
        child.release()
        golden.release()


# -- readiness on every rung ---------------------------------------------------------------


@pytest.mark.parametrize("mode", ["disable_cache", "no_checkpoints"])
def test_uncached_cold_boot_runs_the_wait_strategy(settings: Settings, mode: str) -> None:
    engine = FakeEngine(supports_checkpoints=mode != "no_checkpoints")
    chosen = settings.replace(disable_cache=mode == "disable_cache")
    recording = RecordingWait()
    result = boot_postgres(SPEC, chosen, engine, wait=recording)
    try:
        assert result.info.via == "cold" and result.claim is None
        assert recording.probes >= 1 and "wait" in result.info.timings
    finally:
        result.release()
    assert engine.live_machines == set()


def test_uncached_cold_boot_waits_before_seeding(
    fake_engine: FakeEngine, settings: Settings
) -> None:
    recording = RecordingWait()
    seed = Seed.from_sql("CREATE TABLE t (id int);")
    result = boot_postgres(
        SPEC, settings.replace(disable_cache=True), fake_engine, seed=seed, wait=recording
    )
    try:
        assert result.info.seeded and recording.probes >= 2
        assert recording.sql_seen[0] == [], "the first probe happens before the seed runs"
        assert "CREATE TABLE t (id int)" in recording.sql_seen[-1]
    finally:
        result.release()


def test_non_default_guest_port_reaches_the_server_and_every_client(
    fake_engine: FakeEngine, settings: Settings
) -> None:
    seed = Seed.from_sql("CREATE TABLE t (id int);")
    machine = PostgresMachine(port=5433, settings=settings, seed=seed).start()
    try:
        created: MachineSpec = fake_engine.ops("create")[0]["spec"]
        assert created.argv is not None and "port=5433" in created.argv
        assert created.argv[:4] == ("docker-entrypoint.sh", "postgres", "-c", "port=5433")
        assert created.ports[0].guest == 5433 and machine.port == 5433
        machine.psql("SELECT 1")
        execs = [tuple(c["argv"]) for c in fake_engine.ops("exec")]
        psqls = [argv for argv in execs if argv[0] == "psql"]
        seeds = [argv for argv in psqls if "-f" in argv]
        terminates = [argv for argv in psqls if TERMINATE_FOREIGN_BACKENDS_SQL in argv]
        probes = [argv for argv in execs if argv[0] == "pg_isready"]
        assert seeds and terminates and probes and len(psqls) >= 3
        assert all(argv[argv.index("-p") + 1] == "5433" for argv in psqls)
        assert all("-p 5433" in " ".join(argv) for argv in probes)
    finally:
        machine.stop()
    assert fake_engine.live_machines == set()


def test_exec_timeout_reaches_cold_and_restored_machines(
    fake_engine: FakeEngine, settings: Settings
) -> None:
    tuned = settings.replace(exec_timeout_s=30.0)
    first = boot_postgres(SPEC, tuned, fake_engine)
    first.release()
    second = boot_postgres(SPEC, tuned, fake_engine)
    try:
        assert second.info.via == "restore"
        assert fake_engine.ops("create")[0]["spec"].exec_timeout_s == 30.0
        assert fake_engine.ops("restore")[0]["exec_timeout_s"] == 30.0
        assert fake_of(second).spec.exec_timeout_s == 30.0
        child = branch_from(second, "child", tuned, fake_engine)
        try:
            assert fake_of(child).spec.exec_timeout_s == 30.0
        finally:
            child.release()
    finally:
        second.release()
    assert fake_engine.live_machines == set()


def test_default_wait_and_housekeeping_execs_are_bounded(
    fake_engine: FakeEngine, settings: Settings
) -> None:
    seed = Seed.from_sql("CREATE TABLE t (id int);")
    first = boot_postgres(SPEC, settings, fake_engine, seed=seed)
    first.release()
    second = boot_postgres(SPEC, settings, fake_engine)  # restores: clock resync + wait
    second.release()
    execs = fake_engine.ops("exec")
    probes = [c for c in execs if c["argv"][0] in ("pg_isready", "date")]
    terminates = [c for c in execs if TERMINATE_FOREIGN_BACKENDS_SQL in c["argv"]]
    seeds = [c for c in execs if "-f" in c["argv"]]
    assert probes and terminates and seeds
    assert all(c["timeout_s"] == EXEC_PROBE_TIMEOUT_S for c in probes + terminates)
    assert all(c["timeout_s"] is None for c in seeds), "seeds get the engine's default timeout"


# -- poisoned variants ----------------------------------------------------------------------


def test_restored_variant_that_never_becomes_ready_is_invalidated(
    fake_engine: FakeEngine, settings: Settings
) -> None:
    first = boot_postgres(SPEC, settings, fake_engine)
    old_port = first.endpoint.port
    first.release()
    key = base_key(settings, fake_engine)
    poison_variant(settings, key, old_port)
    quick = settings.replace(ready_timeout_s=0.2, poll_interval_s=0.01)
    second = boot_postgres(SPEC, quick, fake_engine)
    try:
        assert second.info.via == "cold" and second.info.populated
        assert len(fake_engine.ops("restore")) == 1 and len(fake_engine.ops("create")) == 2
        variants = CheckpointCache.open(settings).variants(key.key)
        assert [v.port for v in variants] == [second.endpoint.port]
        if second.endpoint.port != old_port:
            assert not CheckpointCache.open(settings).paths(key.key, old_port).checkpoint.exists()
        assert len(fake_engine.live_machines) == 1
    finally:
        second.release()
    assert fake_engine.live_machines == set() and claim_files(settings) == []
    third = boot_postgres(SPEC, quick, fake_engine)
    try:
        assert third.info.via == "restore" and third.endpoint.port == second.endpoint.port
    finally:
        third.release()


def test_poisoned_seeded_variant_falls_through_to_the_base_key(
    fake_engine: FakeEngine, settings: Settings
) -> None:
    seed = Seed.from_sql("CREATE TABLE t (id int);")
    first = boot_postgres(SPEC, settings, fake_engine, seed=seed)
    port = first.endpoint.port
    first.release()
    seeded_key = base_key(settings, fake_engine).with_seed(seed.key)
    poison_variant(settings, seeded_key, port)
    quick = settings.replace(ready_timeout_s=0.2, poll_interval_s=0.01)
    second = boot_postgres(SPEC, quick, fake_engine, seed=seed)
    try:
        info = second.info
        assert info.via == "restore" and info.restored_key == info.cache_key and info.seeded
        assert len(fake_engine.ops("restore")) == 2 and len(fake_engine.ops("create")) == 1
        assert [v.port for v in CheckpointCache.open(settings).variants(seeded_key.key)] == [port]
        assert len(fake_engine.live_machines) == 1
    finally:
        second.release()
    assert fake_engine.live_machines == set() and claim_files(settings) == []


def test_wait_strategy_refusal_after_restore_does_not_invalidate(
    fake_engine: FakeEngine, settings: Settings
) -> None:
    first = boot_postgres(SPEC, settings, fake_engine)
    port = first.endpoint.port
    first.release()
    key = base_key(settings, fake_engine)
    cache = CheckpointCache.open(settings)
    # LogMessageWaitStrategy needs capture_logs: it raises NotSupportedError, not a timeout.
    with pytest.raises(BootError, match=r"\[wait\] NotSupportedError") as info:
        boot_postgres(SPEC, settings, fake_engine, wait=LogMessageWaitStrategy("ready"))
    assert isinstance(info.value.cause, NotSupportedError) and info.value.stage == "wait"
    assert len(fake_engine.ops("restore")) == 1 and len(fake_engine.ops("create")) == 1
    assert [v.port for v in cache.variants(key.key)] == [port]
    assert cache.paths(key.key, port).meta.is_file()
    assert fake_engine.live_machines == set() and claim_files(settings) == []
    # The variant is intact: the next boot restores it.
    again = boot_postgres(SPEC, settings, fake_engine)
    try:
        assert again.info.via == "restore" and again.endpoint.port == port
    finally:
        again.release()


@pytest.mark.parametrize(
    "error",
    [
        ExecError("agent unreachable", code="EXEC_FAILED"),
        BootError("agent unreachable", stage="exec", code="EXEC_FAILED"),
    ],
    ids=["ExecError", "BootError[exec]"],
)
def test_restored_machine_that_cannot_exec_is_invalidated(
    fake_engine: FakeEngine, settings: Settings, error: SmoltestError
) -> None:
    """A guest that refuses every command is the checkpoint's fault, like a timeout."""
    first = boot_postgres(SPEC, settings, fake_engine)
    old_port = first.endpoint.port
    first.release()
    key = base_key(settings, fake_engine)

    def agent_down(machine: FakeMachine, argv: tuple[str, ...]) -> ExecOutcome | None:
        if machine.via == "restore":
            raise error
        return None

    fake_engine.exec_handlers["pg_isready"] = agent_down
    second = boot_postgres(SPEC, settings, fake_engine)
    try:
        assert second.info.via == "cold" and second.info.populated
        assert len(fake_engine.ops("restore")) == 1 and len(fake_engine.ops("create")) == 2
        cache = CheckpointCache.open(settings)
        assert [v.port for v in cache.variants(key.key)] == [second.endpoint.port]
        if second.endpoint.port != old_port:
            assert not cache.paths(key.key, old_port).checkpoint.exists()
        assert len(fake_engine.live_machines) == 1
    finally:
        second.release()
    assert fake_engine.live_machines == set() and claim_files(settings) == []


def test_unready_restores_stop_at_the_limit_and_boot_cold(
    fake_engine: FakeEngine,
    settings: Settings,
    listener: socket.socket,
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    key = base_key(settings, fake_engine)
    first = boot_postgres(SPEC, settings, fake_engine)
    port_a = first.endpoint.port
    first.release()
    listener.bind((LOOPBACK, port_a))  # keep A busy so the next boot makes variant B
    listener.listen(1)
    second = boot_postgres(SPEC, settings, fake_engine)
    port_b = second.endpoint.port
    second.release()
    listener.close()
    assert port_a != port_b
    for port in (port_a, port_b):
        poison_variant(settings, key, port)
    monkeypatch.setattr(strategy, "MAX_UNREADY_RESTORES", 1)
    quick = settings.replace(ready_timeout_s=0.1, poll_interval_s=0.01)
    third = boot_postgres(SPEC, quick, fake_engine)
    try:
        assert third.info.via == "cold" and third.info.populated
        # One poisoned variant was tried and invalidated; the cap stopped the second try.
        assert len(fake_engine.ops("restore")) == 1 and len(fake_engine.ops("create")) == 3
        remaining = {v.port for v in CheckpointCache.open(settings).variants(key.key)}
        assert third.endpoint.port in remaining and len(remaining) == 2
    finally:
        third.release()
    assert fake_engine.live_machines == set() and claim_files(settings) == []


def test_tunnel_failure_after_restore_does_not_invalidate(
    fake_engine: FakeEngine, settings: Settings
) -> None:
    first = boot_postgres(SPEC, settings, fake_engine)
    port = first.endpoint.port
    first.release()
    fake_engine.fail_next("open_tunnel", SmoltestError("relay down"))
    with pytest.raises(BootError, match=r"\[tunnel\]"):
        boot_postgres(SPEC, settings, fake_engine)
    key = base_key(settings, fake_engine)
    assert [v.port for v in CheckpointCache.open(settings).variants(key.key)] == [port]
    assert fake_engine.live_machines == set() and claim_files(settings) == []


# -- key lock contention ---------------------------------------------------------------------


def test_key_lock_timeout_falls_back_to_an_uncached_cold_boot(
    fake_engine: FakeEngine, settings: Settings, monkeypatch: pytest.MonkeyPatch
) -> None:
    key = base_key(settings, fake_engine)
    # The boot would wait 2 * ready_timeout_s + 120 s for the holder; make it impatient.
    real_open = CheckpointCache.open

    def impatient(settings: Settings, target: str) -> CheckpointCache:
        cache = real_open(settings, target)  # type: ignore[arg-type]
        cache.lock_timeout_s = 0.3
        return cache

    monkeypatch.setattr(CheckpointCache, "open", staticmethod(impatient))
    # Another process in effect: a lock on the key file whose hold the cache's lock shares.
    lock = FileLock.for_path(settings.cache_dir / "postgres" / f"{key.key}.lock", timeout_s=5)
    held, let_go = threading.Event(), threading.Event()

    def holder() -> None:
        with lock:
            held.set()
            let_go.wait(10)

    thread = threading.Thread(target=holder, daemon=True)
    thread.start()
    assert held.wait(5)
    try:
        result = boot_postgres(SPEC, settings, fake_engine)
    finally:
        let_go.set()
        thread.join(5)
    try:
        info = result.info
        assert info.via == "cold" and not info.populated and result.claim is None
        assert "wait" in info.timings and info.cache_key == key.key
        assert fake_engine.ops("restore") == [] and fake_engine.ops("checkpoint") == []
        assert not (settings.cache_dir / "postgres" / key.key).exists()
    finally:
        result.release()
    assert fake_engine.live_machines == set()
    # With the lock free again the next boot populates as usual.
    later = boot_postgres(SPEC, settings, fake_engine)
    try:
        assert later.info.via == "cold" and later.info.populated
    finally:
        later.release()


# -- seeded capture failures -----------------------------------------------------------------


def test_unbranchable_restored_machine_is_seeded_and_handed_out_uncached(
    settings: Settings,
) -> None:
    engine = FakeEngine(restored_branchable=False)
    plain = boot_postgres(SPEC, settings, engine)
    plain.release()
    seed = Seed.from_sql("CREATE TABLE t (id int);")
    key = base_key(settings, engine)
    result = boot_postgres(SPEC, settings, engine, seed=seed)
    try:
        info, fake = result.info, fake_of(result)
        assert info.via == "restore" and info.seeded and not info.populated
        assert result.seed_claim is None and result.claim is not None
        assert fake.sql_log[0] == "CREATE TABLE t (id int)" and fake.exec(["pg_isready"]).ok
        assert len(engine.ops("checkpoint")) == 2  # the base capture, then the refused one
        cache = CheckpointCache.open(settings)
        assert cache.variants(key.with_seed(seed.key).key) == ()
        assert claim_files(settings) == [cache.paths(key.key, result.endpoint.port).claim]
    finally:
        result.release()
    assert claim_files(settings) == [] and engine.live_machines == set()


def test_seeded_capture_error_is_a_cache_miss_not_a_boot_failure(
    fake_engine: FakeEngine, settings: Settings
) -> None:
    plain = boot_postgres(SPEC, settings, fake_engine)
    plain.release()
    fake_engine.fail_next("checkpoint", SmoltestError("disk full"))
    seed = Seed.from_sql("CREATE TABLE t (id int);")
    result = boot_postgres(SPEC, settings, fake_engine, seed=seed)
    try:
        assert result.info.via == "restore" and result.info.seeded
        assert result.seed_claim is None and not result.info.populated
    finally:
        result.release()
    assert claim_files(settings) == [] and fake_engine.live_machines == set()
    # The next seeded boot tries the capture again and caches it this time.
    again = boot_postgres(SPEC, settings, fake_engine, seed=seed)
    try:
        assert again.info.seeded and again.info.populated and again.seed_claim is not None
    finally:
        again.release()


# -- interrupts -----------------------------------------------------------------------------


def test_keyboard_interrupt_during_boot_tears_the_machine_down(
    fake_engine: FakeEngine, settings: Settings
) -> None:
    fake_engine.fail_next("open_tunnel", KeyboardInterrupt())
    with pytest.raises(KeyboardInterrupt):
        boot_postgres(SPEC, settings, fake_engine)
    assert fake_engine.live_machines == set() and len(fake_engine.ops("delete")) == 1
    assert claim_files(settings) == []


def test_keyboard_interrupt_during_branch_tears_the_child_down(
    fake_engine: FakeEngine, settings: Settings
) -> None:
    golden = boot_postgres(SPEC, settings, fake_engine)
    fake_engine.fail_next("open_tunnel", KeyboardInterrupt())
    try:
        with pytest.raises(KeyboardInterrupt):
            branch_from(golden, "child", settings, fake_engine)
        assert fake_engine.live_machines == {fake_of(golden)}
    finally:
        golden.release()
