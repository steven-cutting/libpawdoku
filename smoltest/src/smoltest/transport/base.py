"""Engine-neutral protocols and value types every other layer codes against.

``smoltest.transport.smol_engine`` implements them over the Smol SDK and
``smoltest.testing`` over in-process fakes; nothing above this module imports
``smol``.
"""

from __future__ import annotations

from collections.abc import Mapping, Sequence
from dataclasses import dataclass, field
from typing import Literal, Protocol, runtime_checkable

from .._ports import HostEndpoint

Target = Literal["local", "cloud"]
CheckpointKind = Literal["file", "cloud"]


@dataclass(frozen=True)
class PortMapping:
    """Publish guest port ``guest`` on host port ``host`` (``None`` = let the engine pick)."""

    host: int | None
    guest: int


@dataclass(frozen=True)
class MachineSpec:
    """Engine-neutral description of a machine to create.

    Sequences are normalised to tuples and ``env`` to a plain ``dict`` so a spec
    can be compared and serialised. ``argv`` replaces the image's ENTRYPOINT and
    CMD; ``None`` runs the image's own entrypoint.
    """

    image: str
    argv: tuple[str, ...] | None = None
    env: Mapping[str, str] = field(default_factory=dict)
    ports: tuple[PortMapping, ...] = ()
    cpus: int = 1
    memory_mb: int = 512
    storage_gb: int | None = None
    network: bool = True
    branchable: bool = True
    name: str | None = None
    ready_timeout_s: float = 120.0
    wait_for_ports: bool = True
    auto_stop_seconds: int | None = None
    ttl_seconds: int | None = None
    exec_timeout_s: float = 600.0

    def __post_init__(self) -> None:
        # Coerce leniently: callers may pass lists; equality and hashing need tuples.
        object.__setattr__(self, "argv", None if self.argv is None else tuple(self.argv))
        object.__setattr__(self, "ports", tuple(self.ports))
        object.__setattr__(self, "env", dict(self.env))

    @property
    def guest_ports(self) -> tuple[int, ...]:
        """Guest ports in declaration order."""
        return tuple(p.guest for p in self.ports)


@dataclass(frozen=True)
class CheckpointRef:
    """Where a checkpoint lives: a ``file`` path on this host or a ``cloud`` id."""

    kind: CheckpointKind
    locator: str


@dataclass(frozen=True)
class CheckpointInfo:
    """What the engine reports after taking a checkpoint."""

    ref: CheckpointRef
    size_bytes: int | None = None
    store: str | None = None
    machine_id: str | None = None
    elapsed_ms: float | None = None
    source_pause_ms: float | None = None


@dataclass(frozen=True)
class ExecOutcome:
    """Result of an in-guest command."""

    exit_code: int
    stdout: bytes = b""
    stderr: bytes = b""
    argv: tuple[str, ...] = ()

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


@runtime_checkable
class Bridge(Protocol):
    """Something that makes a guest port reachable from the host and can be closed."""

    @property
    def endpoint(self) -> HostEndpoint | None:
        """The host endpoint while open, else ``None``."""
        ...

    def open(self) -> HostEndpoint:
        """Make the guest port reachable and return where."""
        ...

    def close(self) -> None:
        """Tear the bridge down; safe to call twice."""
        ...


class StaticBridge:
    """A :class:`Bridge` over a port the host already publishes (the local target)."""

    def __init__(self, endpoint: HostEndpoint) -> None:
        self._endpoint = endpoint
        self._open = False

    @property
    def endpoint(self) -> HostEndpoint | None:
        return self._endpoint if self._open else None

    def open(self) -> HostEndpoint:
        self._open = True
        return self._endpoint

    def close(self) -> None:
        self._open = False

    def __repr__(self) -> str:
        return f"StaticBridge({self._endpoint}, open={self._open})"


@runtime_checkable
class MachineHandle(Protocol):
    """A running (or paused) machine, however it was obtained."""

    @property
    def name(self) -> str: ...

    @property
    def id(self) -> str: ...

    def state(self) -> str:
        """Engine state string, e.g. ``running``, ``paused``, ``stopped``."""
        ...

    def ready(self) -> bool:
        """``True`` when the guest agent answers and every published port accepts."""
        ...

    def wait_until_ready(self, timeout_s: float, interval_s: float) -> None:
        """Poll :meth:`ready`; raise :class:`smoltest.errors.ReadinessTimeout` on expiry."""
        ...

    def exec(
        self,
        argv: Sequence[str],
        *,
        timeout_s: float | None = None,
        user: str | None = None,
    ) -> ExecOutcome:
        """Run ``argv`` in the guest and return its outcome.

        ``timeout_s=None`` means the engine's default (``MachineSpec.exec_timeout_s``
        for the Smol engine); a command still running when it expires is killed.
        """
        ...

    def read_file(self, path: str) -> bytes:
        """Read a guest file."""
        ...

    def write_file(self, path: str, data: bytes, mode: int | None = None) -> None:
        """Write a guest file."""
        ...

    def host_port(self, guest: int) -> tuple[str, int]:
        """Host ``(ip, port)`` publishing ``guest``; cloud raises ``NotSupportedError``."""
        ...

    def branch(self, name: str, ports: Sequence[PortMapping] | None = None) -> MachineHandle:
        """Copy-on-write clone of this running machine with its own host ports."""
        ...

    def checkpoint(self, output: str | None = None, store: str | None = None) -> CheckpointInfo:
        """Snapshot RAM and disks; ``output`` is required on the local target."""
        ...

    def pause(self) -> None: ...

    def resume(self) -> None: ...

    def stop(self) -> None: ...

    def delete(self) -> None:
        """Destroy the machine and free its ports; idempotent."""
        ...


@runtime_checkable
class Engine(Protocol):
    """Factory and host-level operations of one machine backend."""

    def sdk_version(self) -> str: ...

    def local_availability(self) -> tuple[bool, str | None, str | None]:
        """``(ok, code, reason)`` for the local target."""
        ...

    def supports_checkpoints(self, target: Target) -> bool: ...

    def supports_branch(self, target: Target) -> bool: ...

    def create(self, spec: MachineSpec, target: Target) -> MachineHandle:
        """Create a machine and return once it is ready."""
        ...

    def connect(self, machine_id: str, target: Target) -> MachineHandle:
        """Attach to an existing machine.

        The machine was made elsewhere, so its handle gets the engine's default
        exec timeout rather than any :class:`MachineSpec` of ours.
        """
        ...

    def restore_checkpoint(
        self,
        ref: CheckpointRef,
        name: str,
        target: Target,
        *,
        exec_timeout_s: float | None = None,
    ) -> MachineHandle:
        """Resume a checkpoint as a new machine named ``name``.

        ``exec_timeout_s`` is the default :meth:`MachineHandle.exec` timeout of the
        restored machine (and of the branches made from it); ``None`` means the
        engine's own default, the same one a cold ``create`` applies when its spec
        says nothing.
        """
        ...

    def export_checkpoint(self, source: str, output: str) -> int:
        """Copy a checkpoint to ``output``; return bytes written."""
        ...

    def prune_checkpoint_store(self, store: str) -> int:
        """Drop unreferenced chunks from a dedup store; return how many."""
        ...

    def open_tunnel(self, machine_id: str, guest_port: int, target: Target) -> Bridge:
        """Return an unopened :class:`Bridge` to ``guest_port``."""
        ...


__all__ = [
    "Bridge",
    "CheckpointInfo",
    "CheckpointKind",
    "CheckpointRef",
    "Engine",
    "ExecOutcome",
    "HostEndpoint",
    "MachineHandle",
    "MachineSpec",
    "PortMapping",
    "StaticBridge",
    "Target",
]
