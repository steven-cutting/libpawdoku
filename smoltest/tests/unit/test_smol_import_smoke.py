"""The one test module that imports ``smol``: the SDK surface smoltest relies on exists.

Nothing here boots a machine. ``SmolEngine`` is exercised against the real
``MachineConfig``/``PortSpec``/``ExecOptions`` types and against a stub that
quacks like ``smol.Machine``.
"""

from __future__ import annotations

import dataclasses
import importlib.metadata
import importlib.util
import inspect
from typing import Any

import pytest

from smoltest._ports import PORT_RANGE, is_port_free
from smoltest.errors import (
    BootError,
    ExecError,
    InvalidConfig,
    NotSupportedError,
    ReadinessTimeout,
    SmoltestError,
)
from smoltest.transport import DEFAULT_ENGINE, load_engine_factory
from smoltest.transport.base import (
    CheckpointRef,
    Engine,
    MachineHandle,
    MachineSpec,
    PortMapping,
    StaticBridge,
)
from smoltest.transport.smol_engine import (
    DEFAULT_EXEC_TIMEOUT_S,
    SmolEngine,
    SmolHandle,
    _to_machine_config,
    _translate,
)
from smoltest.transport.tunnel import TunnelBridge

smol = pytest.importorskip("smol")

LOCAL_CODES = {
    "UNSUPPORTED_PLATFORM",
    "RUNTIME_NOT_INSTALLED",
    "KVM_UNAVAILABLE",
    "HYPERVISOR_UNAVAILABLE",
}


def params(fn: Any) -> list[str]:
    return [p for p in inspect.signature(fn).parameters if p != "self"]


# -- the SDK surface ------------------------------------------------------------------


def test_machine_class_has_the_entry_points_we_call() -> None:
    machine = smol.Machine
    assert params(machine.create)[:2] == ["config", "conn"]
    assert params(machine.connect)[:2] == ["machine_id", "conn"]
    assert params(machine.restore_checkpoint)[:3] == ["checkpoint_id", "name", "conn"]
    assert params(machine.export_checkpoint) == ["source", "output"]
    assert params(machine.prune_checkpoint_store) == ["store"]
    for name in ("create", "connect", "restore_checkpoint"):
        assert inspect.ismethod(getattr(machine, name)), f"{name} must be a classmethod"


def test_machine_instance_methods_have_the_shapes_we_use() -> None:
    machine = smol.Machine
    assert params(machine.branch)[:2] == ["name", "ports"]
    assert "branchable" in params(machine.branch)
    assert params(machine.checkpoint) == ["output", "store"]
    assert params(machine.endpoint)[:1] == ["port"]
    assert params(machine.exec) == ["command", "opts"]
    assert params(machine.wait_until_ready) == ["timeout_s", "interval_s"]
    assert params(machine.write_file) == ["path", "data", "mode"]
    for name in ("delete", "state", "ready", "read_file", "pause", "resume", "stop"):
        assert callable(getattr(machine, name)), name
    assert isinstance(inspect.getattr_static(machine, "name"), property)
    assert isinstance(inspect.getattr_static(machine, "id"), property)


def test_config_types_have_the_fields_we_set() -> None:
    config_fields = {f.name for f in dataclasses.fields(smol.MachineConfig)}
    assert config_fields >= {
        "name",
        "image",
        "command",
        "env",
        "ports",
        "resources",
        "network",
        "persistent",
        "branchable",
        "wait_for_ports",
        "ready_timeout_seconds",
        "auto_stop_seconds",
        "ttl_seconds",
    }
    assert [f.name for f in dataclasses.fields(smol.PortSpec)] == ["host", "guest"]
    assert {f.name for f in dataclasses.fields(smol.ResourceSpec)} >= {
        "cpus",
        "memory_mb",
        "network",
        "storage_gb",
    }
    assert {f.name for f in dataclasses.fields(smol.ExecOptions)} >= {"timeout", "user"}
    assert {f.name for f in dataclasses.fields(smol.ExecResult)} >= {
        "exit_code",
        "stdout",
        "stderr",
        "stdout_bytes",
        "stderr_bytes",
    }
    assert {f.name for f in dataclasses.fields(smol.ConnectOptions)} >= {"target"}
    assert {f.name for f in dataclasses.fields(smol.PortableCheckpointInfo)} >= {
        "id",
        "path",
        "size_bytes",
        "machine_id",
        "elapsed_ms",
        "source_pause_ms",
    }


def test_tunnel_errors_and_availability_exist() -> None:
    from smol import tunnel

    assert params(tunnel.open_tunnel) == ["machine", "port"]
    assert inspect.isfunction(tunnel.open_tunnel)
    assert callable(smol.local_availability)
    for name in ("SmolError", "NotSupportedError", "InvalidConfigError", "ExecutionError"):
        assert issubclass(getattr(smol, name), Exception), name
    assert issubclass(smol.NotSupportedError, smol.SmolError)
    assert smol.SmolError("SOME_CODE", "message").code == "SOME_CODE"
    assert smol.NotSupportedError("x").code == "NOT_SUPPORTED"
    assert smol.InvalidConfigError("x").code == "INVALID_CONFIG"


# -- SmolEngine without a machine ---------------------------------------------------


def test_default_engine_path_builds_a_smol_engine() -> None:
    engine = load_engine_factory(DEFAULT_ENGINE)()
    assert isinstance(engine, SmolEngine)
    assert isinstance(engine, Engine)


def test_local_availability_is_a_triple_and_sdk_version_matches() -> None:
    engine = SmolEngine()
    result = engine.local_availability()
    assert isinstance(result, tuple) and len(result) == 3
    ok, code, reason = result
    assert isinstance(ok, bool)
    if ok:
        assert code is None and reason is None
    else:
        assert code in LOCAL_CODES and isinstance(reason, str) and reason
    assert engine.sdk_version() == importlib.metadata.version("smolmachines")
    assert engine.supports_checkpoints("local") and engine.supports_branch("cloud")


def test_to_machine_config_translates_every_field_without_creating_a_machine() -> None:
    spec = MachineSpec(
        image="postgres:16",
        argv=("docker-entrypoint.sh", "postgres", "-c", "fsync=off"),
        env={"POSTGRES_PASSWORD": "test", "POSTGRES_USER": "test"},
        ports=(PortMapping(None, 5432), PortMapping(24001, 8080)),
        cpus=2,
        memory_mb=768,
        storage_gb=4,
        network=False,
        name="golden",
        ready_timeout_s=42.0,
        auto_stop_seconds=1800,
        ttl_seconds=7200,
    )
    picks = iter([24000])
    config = _to_machine_config(spec, "cloud", pick_port=lambda: next(picks))
    assert isinstance(config, smol.MachineConfig)
    assert config.name == "golden"
    assert config.image == "postgres:16"
    assert config.command == ["docker-entrypoint.sh", "postgres", "-c", "fsync=off"]
    assert config.env == {"POSTGRES_PASSWORD": "test", "POSTGRES_USER": "test"}
    assert [(p.host, p.guest) for p in config.ports] == [(24000, 5432), (24001, 8080)]
    assert all(isinstance(p, smol.PortSpec) for p in config.ports)
    assert isinstance(config.resources, smol.ResourceSpec)
    assert (config.resources.cpus, config.resources.memory_mb) == (2, 768)
    assert (config.resources.network, config.resources.storage_gb) == (False, 4)
    assert config.network is False
    assert config.branchable is True and config.forkable is True
    assert config.persistent is False
    assert config.wait_for_ports is True
    assert config.ready_timeout_seconds == 42.0
    assert (config.auto_stop_seconds, config.ttl_seconds) == (1800, 7200)


def test_to_machine_config_local_defaults_and_free_port_pick() -> None:
    spec = MachineSpec(image="postgres:16", ports=(PortMapping(None, 5432),))
    config = _to_machine_config(spec, "local")
    assert config.command is None and config.env is None and config.name is None
    assert config.auto_stop_seconds is None and config.ttl_seconds is None
    [port] = config.ports
    assert port.guest == 5432
    assert PORT_RANGE[0] <= port.host <= PORT_RANGE[1] and is_port_free(port.host)
    empty = _to_machine_config(MachineSpec(image="nginx:1.27-alpine"), "local")
    assert empty.ports is None and empty.branchable is True


def test_to_machine_config_rejects_what_the_sdk_rejects() -> None:
    with pytest.raises(BootError, match=r"\[config\]") as info:
        _to_machine_config(MachineSpec(image="postgres:16", argv=("",)), "local")
    assert info.value.stage == "config"


def test_translate_maps_sdk_errors_into_the_smoltest_hierarchy() -> None:
    mapped = _translate(smol.NotSupportedError("branch is cloud-only here"), stage="branch")
    assert isinstance(mapped, NotSupportedError) and mapped.code == "NOT_SUPPORTED"
    assert not isinstance(mapped, BootError)

    config = _translate(smol.InvalidConfigError("bad config"), stage="create")
    assert isinstance(config, BootError) and config.stage == "config"
    assert config.code == "INVALID_CONFIG" and "bad config" in str(config)

    timeout = _translate(smol.SmolError("TIMEOUT", "took too long"), stage="wait")
    assert isinstance(timeout, ReadinessTimeout) and "took too long" in str(timeout)

    generic = _translate(smol.SmolError("KVM_UNAVAILABLE", "no /dev/kvm"), stage="create")
    assert isinstance(generic, BootError) and generic.stage == "create"
    assert generic.code == "KVM_UNAVAILABLE"
    assert str(generic).startswith("[create] KVM_UNAVAILABLE: no /dev/kvm")

    execution = _translate(smol.ExecutionError(["false"], 1, "", "boom"), stage="exec")
    assert isinstance(execution, ExecError) and execution.code == "COMMAND_FAILED"

    already = InvalidConfig("mine")
    assert _translate(already, stage="create") is already

    foreign = _translate(ValueError("plain"), stage="exec")
    assert isinstance(foreign, BootError) and "plain" in str(foreign)


# -- SmolHandle over a stub machine ---------------------------------------------------


class StubMachine:
    """Quacks like ``smol.Machine`` for one local machine publishing 5432 on 24321."""

    def __init__(self, name: str = "stub-1") -> None:
        self.name = name
        self.id = name
        self.calls: list[tuple[str, Any]] = []
        self.deleted = 0

    def state(self) -> str:
        return "running"

    def ready(self) -> bool:
        return True

    def wait_until_ready(self, timeout_s: float, interval_s: float) -> None:
        self.calls.append(("wait", (timeout_s, interval_s)))
        raise smol.SmolError("TIMEOUT", f"not ready after {timeout_s:g}s")

    def endpoint(self, port: int, path: str | None = None) -> Any:
        if port != 5432:
            raise smol.InvalidConfigError(f"guest port {port} is not published")
        return smol.PortEndpoint(
            http_url="http://127.0.0.1:24321", ws_url="ws://127.0.0.1:24321", headers={}
        )

    def exec(self, command: list[str], opts: Any = None) -> Any:
        self.calls.append(("exec", (command, opts)))
        return smol.ExecResult(
            exit_code=3,
            stdout="out",
            stderr="err",
            stdout_bytes=b"out-bytes",
            stderr_bytes=b"err-bytes",
        )

    def read_file(self, path: str) -> bytes:
        return path.encode()

    def write_file(self, path: str, data: bytes, mode: int | None = None) -> None:
        self.calls.append(("write", (path, data, mode)))

    def branch(self, name: str, ports: Any = None, *, branchable: bool = False) -> StubMachine:
        self.calls.append(("branch", (name, ports, branchable)))
        return StubMachine(name)

    def checkpoint(self, output: str | None = None, *, store: str | None = None) -> Any:
        self.calls.append(("checkpoint", (output, store)))
        return smol.PortableCheckpointInfo(
            id=output or "ckpt-9",
            machine_id=self.name,
            status="available",
            size_bytes=4096,
            arch="x86_64",
            created_at="2026-10-03T00:00:00Z",
            download_url="",
            path=output,
            source_pause_ms=12.5,
            elapsed_ms=250.0,
        )

    def pause(self) -> None:
        self.calls.append(("pause", None))

    def resume(self) -> None:
        self.calls.append(("resume", None))

    def stop(self) -> None:
        self.calls.append(("stop", None))

    def delete(self) -> None:
        self.deleted += 1


def test_smol_handle_maps_every_call(tmp_path: Any) -> None:
    engine = SmolEngine()
    stub = StubMachine()
    handle = engine._adopt(stub, "local")
    assert isinstance(handle, SmolHandle) and isinstance(handle, MachineHandle)
    assert (handle.name, handle.id, handle.target) == ("stub-1", "stub-1", "local")
    assert handle.machine is stub
    assert handle.state() == "running" and handle.ready()

    with pytest.raises(ReadinessTimeout) as timeout:
        handle.wait_until_ready(1.5, 0.1)
    assert timeout.value.timeout_s == 1.5

    outcome = handle.exec(["psql", "-c", "SELECT 1"], timeout_s=2.0, user="postgres")
    assert (outcome.exit_code, outcome.stdout, outcome.stderr) == (3, b"out-bytes", b"err-bytes")
    assert outcome.argv == ("psql", "-c", "SELECT 1")
    _, (command, opts) = stub.calls[-1]
    assert command == ["psql", "-c", "SELECT 1"]
    assert isinstance(opts, smol.ExecOptions) and (opts.timeout, opts.user) == (2.0, "postgres")
    handle.exec(("true",))
    _, (command, opts) = stub.calls[-1]
    assert command == ["true"]
    assert isinstance(opts, smol.ExecOptions)
    assert (opts.timeout, opts.user) == (DEFAULT_EXEC_TIMEOUT_S, None)

    assert handle.read_file("/etc/hostname") == b"/etc/hostname"
    handle.write_file("/tmp/x", b"data", 0o600)
    assert stub.calls[-1] == ("write", ("/tmp/x", b"data", 0o600))

    assert handle.host_port(5432) == ("127.0.0.1", 24321)
    with pytest.raises(BootError, match="not published") as unpublished:
        handle.host_port(80)
    assert unpublished.value.stage == "config"

    child = handle.branch("child", [PortMapping(24400, 5432)])
    assert isinstance(child, SmolHandle) and child.id == "child"
    _, (name, ports, branchable) = stub.calls[-1]
    assert name == "child" and branchable is True
    assert [(p.host, p.guest) for p in ports] == [(24400, 5432)]
    assert all(isinstance(p, smol.PortSpec) for p in ports)
    handle.branch("auto")
    assert stub.calls[-1] == ("branch", ("auto", None, True))

    with pytest.raises(InvalidConfig):
        handle.checkpoint()
    output = str(tmp_path / "golden.smolcheckpoint")
    info = handle.checkpoint(output, store=str(tmp_path / "store"))
    assert info.ref == CheckpointRef("file", output)
    assert (info.size_bytes, info.machine_id, info.store) == (
        4096,
        "stub-1",
        str(tmp_path / "store"),
    )
    assert (info.elapsed_ms, info.source_pause_ms) == (250.0, 12.5)

    handle.pause()
    handle.resume()
    handle.stop()
    assert [op for op, _ in stub.calls[-3:]] == ["pause", "resume", "stop"]

    assert engine._lookup("stub-1") is handle
    handle.delete()
    handle.delete()
    assert stub.deleted == 1, "delete is idempotent"
    assert engine._lookup("stub-1") is None
    child.delete()


def test_cloud_handle_refuses_host_port_and_keys_checkpoints_by_id() -> None:
    engine = SmolEngine()
    handle = engine._adopt(StubMachine("mach-1"), "cloud")
    with pytest.raises(NotSupportedError):
        handle.host_port(5432)
    info = handle.checkpoint()
    assert info.ref == CheckpointRef("cloud", "ckpt-9") and info.store is None
    with pytest.raises(InvalidConfig):
        engine.restore_checkpoint(CheckpointRef("file", "/x.smolcheckpoint"), "r", "cloud")
    with pytest.raises(InvalidConfig):
        engine.restore_checkpoint(CheckpointRef("cloud", "ckpt-9"), "r", "local")


def test_open_tunnel_returns_static_bridge_locally_and_tunnel_bridge_on_cloud() -> None:
    engine = SmolEngine()
    local = engine._adopt(StubMachine("local-1"), "local")
    bridge = engine.open_tunnel(local.id, 5432, "local")
    assert isinstance(bridge, StaticBridge)
    assert bridge.open().port == 24321

    cloud = engine._adopt(StubMachine("mach-2"), "cloud")
    if importlib.util.find_spec("websockets") is None:
        with pytest.raises(NotSupportedError, match="websockets"):
            engine.open_tunnel(cloud.id, 5432, "cloud")
    else:
        assert isinstance(engine.open_tunnel(cloud.id, 5432, "cloud"), TunnelBridge)


def test_errors_raised_by_the_machine_are_translated() -> None:
    class Exploding(StubMachine):
        def delete(self) -> None:
            raise smol.SmolError("NOT_FOUND", "already gone")

        def exec(self, command: list[str], opts: Any = None) -> Any:
            raise smol.NotSupportedError("exec is not available here")

    engine = SmolEngine()
    handle = engine._adopt(Exploding("boom"), "local")
    with pytest.raises(NotSupportedError):
        handle.exec(["true"])
    with pytest.raises(BootError, match=r"\[delete\] NOT_FOUND: already gone") as info:
        handle.delete()
    assert isinstance(info.value, SmoltestError) and info.value.code == "NOT_FOUND"
    assert engine._lookup("boom") is handle, "a failed delete keeps the handle registered"
