"""SmolEngine behaviour that needs no SDK: error classification, exec defaults, extras.

``smol`` is never imported here; a stub module stands in where the adapter needs one.
"""

from __future__ import annotations

import importlib.util
import sys
import types
from dataclasses import dataclass
from typing import Any

import pytest

from smoltest.errors import BootError, NotSupportedError
from smoltest.transport import smol_engine
from smoltest.transport.base import MachineSpec, PortMapping
from smoltest.transport.smol_engine import DEFAULT_EXEC_TIMEOUT_S, SmolEngine, _translate


class SmolError(Exception):
    """Shaped like the SDK's ``SmolError``: matched by class name and ``.code``."""

    def __init__(self, code: str, message: str) -> None:
        super().__init__(message)
        self.code = code


NATIVE_REFUSAL = "'g' is not running forkable; start it with `machine start --forkable --name g`"
CLOUD_REFUSAL = "cloud POST /v1/machines/m-1/branches → 409 Conflict: machine is not forkable"


def test_translate_turns_unbranchable_refusals_into_not_supported() -> None:
    native = SmolError("SMOLVM_ERROR", NATIVE_REFUSAL)
    mapped = _translate(native, stage="branch")
    assert isinstance(mapped, NotSupportedError) and not isinstance(mapped, BootError)
    assert mapped.__cause__ is native and "not running forkable" in str(mapped)

    cloud = _translate(SmolError("SMOLVM_ERROR", CLOUD_REFUSAL), stage="branch")
    assert isinstance(cloud, NotSupportedError)

    other = _translate(SmolError("INVALID_STATE", "machine is paused"), stage="branch")
    assert isinstance(other, BootError) and other.stage == "branch"

    elsewhere = _translate(SmolError("SMOLVM_ERROR", NATIVE_REFUSAL), stage="checkpoint")
    assert isinstance(elsewhere, BootError) and elsewhere.stage == "checkpoint"


@dataclass
class ExecOptions:
    timeout: float | None = None
    user: str | None = None


@dataclass
class ExecResult:
    exit_code: int = 0
    stdout: str = ""
    stderr: str = ""


class StubMachine:
    def __init__(self, machine_id: str = "stub-1") -> None:
        self.id = machine_id
        self.name = machine_id
        self.execs: list[tuple[list[str], Any]] = []

    def exec(self, argv: list[str], opts: Any) -> ExecResult:
        self.execs.append((argv, opts))
        return ExecResult()


@pytest.fixture
def stub_smol(monkeypatch: pytest.MonkeyPatch) -> types.ModuleType:
    """A minimal ``smol`` in ``sys.modules`` so the adapter's lazy import finds it."""
    module = types.ModuleType("smol")
    created: list[Any] = []

    @dataclass
    class PortSpec:
        host: int | None
        guest: int

    @dataclass
    class ResourceSpec:
        cpus: int
        memory_mb: int
        network: bool
        storage_gb: int | None

    @dataclass
    class MachineConfig:
        name: str | None
        image: str
        command: list[str] | None
        env: dict[str, str] | None
        ports: list[Any] | None
        resources: Any

    @dataclass
    class ConnectOptions:
        target: str

    class Machine:
        @staticmethod
        def create(config: Any, conn: Any) -> StubMachine:
            created.append(config)
            return StubMachine()

    module.ExecOptions = ExecOptions  # type: ignore[attr-defined]
    module.PortSpec = PortSpec  # type: ignore[attr-defined]
    module.ResourceSpec = ResourceSpec  # type: ignore[attr-defined]
    module.MachineConfig = MachineConfig  # type: ignore[attr-defined]
    module.ConnectOptions = ConnectOptions  # type: ignore[attr-defined]
    module.Machine = Machine  # type: ignore[attr-defined]
    module.created = created  # type: ignore[attr-defined]
    monkeypatch.setitem(sys.modules, "smol", module)
    return module


def test_exec_applies_the_default_timeout_when_given_none(stub_smol: types.ModuleType) -> None:
    engine = SmolEngine()
    stub = StubMachine()
    handle = engine._adopt(stub, "local")
    assert handle.exec_timeout_s == DEFAULT_EXEC_TIMEOUT_S
    handle.exec(["psql", "-c", "SELECT 1"])
    handle.exec(["true"], timeout_s=5.0, user="postgres")
    assert [(opts.timeout, opts.user) for _, opts in stub.execs] == [
        (DEFAULT_EXEC_TIMEOUT_S, None),
        (5.0, "postgres"),
    ]
    custom = engine._adopt(StubMachine("stub-2"), "local", exec_timeout_s=42.0)
    custom.exec(["true"])
    assert custom.machine.execs[-1][1].timeout == 42.0


def test_create_hands_the_spec_exec_timeout_to_the_handle(stub_smol: types.ModuleType) -> None:
    spec = MachineSpec(image="postgres:16", ports=(PortMapping(24321, 5432),), exec_timeout_s=99.0)
    handle = SmolEngine().create(spec, "local")
    assert isinstance(handle, smol_engine.SmolHandle) and handle.exec_timeout_s == 99.0
    handle.exec(["true"])
    assert handle.machine.execs[-1][1].timeout == 99.0


def test_cloud_create_refuses_before_a_machine_exists_without_websockets(
    stub_smol: types.ModuleType, monkeypatch: pytest.MonkeyPatch
) -> None:
    real_find_spec = importlib.util.find_spec

    def find_spec(name: str, *args: Any, **kwargs: Any) -> Any:
        return None if name == "websockets" else real_find_spec(name, *args, **kwargs)

    monkeypatch.setattr(importlib.util, "find_spec", find_spec)
    engine = SmolEngine()
    spec = MachineSpec(image="postgres:16", ports=(PortMapping(None, 5432),))
    with pytest.raises(NotSupportedError, match="websockets") as info:
        engine.create(spec, "cloud")
    assert info.value.code == "TUNNEL_EXTRA_MISSING"
    assert stub_smol.created == [], "no billed cloud machine was created"  # type: ignore[attr-defined]
    assert isinstance(engine.create(spec, "local"), smol_engine.SmolHandle)
