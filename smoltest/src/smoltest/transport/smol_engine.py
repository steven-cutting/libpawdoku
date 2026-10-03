"""The real engine: :class:`Engine` and :class:`MachineHandle` over the Smol SDK.

Importing this module does **not** import ``smol``; every SDK symbol is
resolved lazily inside the method that needs it, so ``smoltest doctor`` and
the unit suite work on hosts where the SDK is missing or cannot load. Every
SDK exception is translated into the :mod:`smoltest.errors` hierarchy.

Call shapes were taken from ``smolmachines`` 1.22.2:
``Machine.create(config, conn)``, ``Machine.connect(id, conn)``,
``Machine.restore_checkpoint(checkpoint_id, name, conn)``,
``machine.branch(name, ports, *, branchable)``,
``machine.checkpoint(output, *, store) -> PortableCheckpointInfo``,
``machine.exec(argv, ExecOptions) -> ExecResult`` and
``smol.tunnel.open_tunnel(machine, port)`` (an async context manager over a
*sync* ``Machine``).
"""

from __future__ import annotations

import dataclasses
import importlib
import importlib.metadata
import importlib.util
import threading
from collections.abc import Callable, Sequence
from pathlib import Path
from typing import Any
from urllib.parse import urlsplit

from .._ports import LOOPBACK, HostEndpoint, pick_free_port
from ..errors import (
    BootError,
    ExecError,
    InvalidConfig,
    NotSupportedError,
    ReadinessTimeout,
    SmoltestError,
    map_smol_error,
)
from .base import (
    CheckpointInfo,
    CheckpointRef,
    ExecOutcome,
    MachineHandle,
    MachineSpec,
    PortMapping,
    StaticBridge,
    Target,
)
from .tunnel import TunnelBridge

SDK_DISTRIBUTION = "smolmachines"
"""PyPI distribution name of the SDK whose import name is ``smol``."""

DEFAULT_EXEC_TIMEOUT_S = 600.0
"""Exec timeout handed to the SDK when the caller passes none.

Without ``ExecOptions.timeout`` the SDK bounds a cloud exec by its 30 s HTTP read
timeout, which aborts long seeds and migrations while psql keeps running in the
guest, and leaves a local exec unbounded. With it, the SDK sizes the cloud read
timeout to ``timeout + 30 s`` and kills the command on both targets when it expires.
"""

_UNBRANCHABLE_MARKERS = (
    "not running forkable",
    "not forkable",
    "not branchable",
    "not_forkable",
    "not_branchable",
)

# MachineConfig fields smoltest sets when the installed SDK has them; older or
# newer releases that lack one simply do not receive it (``forkable`` is the
# pre-1.20 spelling of ``branchable``).
_OPTIONAL_CONFIG_FIELDS = frozenset(
    {
        "branchable",
        "forkable",
        "wait_for_ports",
        "ready_timeout_seconds",
        "auto_stop_seconds",
        "ttl_seconds",
        "persistent",
        "network",
    }
)


def _smol() -> Any:
    """Import and return the ``smol`` package, or raise :class:`NotSupportedError`."""
    try:
        return importlib.import_module("smol")
    except ImportError as exc:
        raise NotSupportedError(
            f"the Smol SDK ({SDK_DISTRIBUTION}) is not importable: {exc}",
            code="RUNTIME_NOT_INSTALLED",
        ) from exc


def _is_unbranchable_source(exc: BaseException) -> bool:
    """``True`` for the SDK's refusals to branch a machine that is not a branch source.

    The native engine says ``'<name>' is not running forkable; start it with
    ``machine start --forkable``; Smol Cloud answers ``POST .../branches`` with a
    409 for a machine stored non-forkable. Neither arrives as the SDK's
    ``NotSupportedError``, which it reserves for cloud-vs-local feature gaps.
    """
    text = str(exc).lower()
    if any(marker in text for marker in _UNBRANCHABLE_MARKERS):
        return True
    return "409" in text and ("/branches" in text or "/fork" in text)


def _translate(exc: BaseException, *, stage: str) -> SmoltestError:
    """Map an SDK exception to smoltest's hierarchy for an operation at ``stage``.

    ``NotSupportedError`` and ``ExecutionError`` keep their own classes because
    callers branch on them; a refusal to branch an unbranchable source at stage
    ``branch`` becomes :class:`NotSupportedError` too, so goldens fall back to
    fresh machines; ``InvalidConfigError`` becomes ``BootError(stage="config")``;
    a ``TIMEOUT`` while waiting becomes :class:`ReadinessTimeout`; everything else
    is a :class:`BootError` at ``stage`` whose message starts with the SDK error code.
    """
    if isinstance(exc, SmoltestError):
        return exc
    mapped = map_smol_error(exc)
    if isinstance(mapped, (NotSupportedError, ExecError)):
        return mapped
    if isinstance(mapped, InvalidConfig):
        return BootError(str(mapped), stage="config", cause=exc, code=mapped.code)
    if stage == "branch" and _is_unbranchable_source(exc):
        unsupported = NotSupportedError(
            f"{mapped.code or 'SMOLVM_ERROR'}: {exc}", code=mapped.code or "NOT_BRANCHABLE"
        )
        unsupported.__cause__ = exc
        return unsupported
    code = mapped.code
    if code == "TIMEOUT" and stage == "wait":
        return ReadinessTimeout(str(exc))
    message = f"{code}: {exc}" if code else str(mapped)
    return BootError(message, stage=stage, cause=exc, code=code)


def _to_port_specs(
    ports: Sequence[PortMapping], pick_port: Callable[[], int] | None = None
) -> list[Any]:
    """Build ``smol.PortSpec`` objects, picking a free host port where none is pinned."""
    smol = _smol()
    chosen: list[int] = []

    def pick() -> int:
        if pick_port is not None:
            return pick_port()
        return pick_free_port(exclude=chosen)

    specs: list[Any] = []
    for mapping in ports:
        host = mapping.host if mapping.host is not None else pick()
        chosen.append(host)
        specs.append(smol.PortSpec(host=host, guest=mapping.guest))
    return specs


def _to_machine_config(
    spec: MachineSpec, target: Target, *, pick_port: Callable[[], int] | None = None
) -> Any:
    """Translate an engine-neutral :class:`MachineSpec` into ``smol.MachineConfig``.

    Pure apart from the host-port pick for unpinned ports (injectable through
    ``pick_port`` so tests stay deterministic). ``argv`` becomes ``command``,
    which replaces the image's ENTRYPOINT and CMD; resources go through
    ``ResourceSpec``; the machine is always a branch source. ``target`` only
    affects which optional fields matter (``auto_stop``/``ttl`` are cloud-only
    but harmless locally) and is kept in the signature for forward compatibility.
    """
    smol = _smol()
    known = {f.name for f in dataclasses.fields(smol.MachineConfig)}
    kwargs: dict[str, Any] = {
        "name": spec.name,
        "image": spec.image,
        "command": list(spec.argv) if spec.argv is not None else None,
        "env": dict(spec.env) or None,
        "ports": _to_port_specs(spec.ports, pick_port) or None,
        "resources": smol.ResourceSpec(
            cpus=spec.cpus,
            memory_mb=spec.memory_mb,
            network=spec.network,
            storage_gb=spec.storage_gb,
        ),
    }
    optional: dict[str, Any] = {
        "network": spec.network,
        "persistent": False,
        "branchable": spec.branchable,
        "forkable": spec.branchable,
        "wait_for_ports": spec.wait_for_ports,
        "ready_timeout_seconds": spec.ready_timeout_s,
        "auto_stop_seconds": spec.auto_stop_seconds if target == "cloud" else None,
        "ttl_seconds": spec.ttl_seconds if target == "cloud" else None,
    }
    for name in _OPTIONAL_CONFIG_FIELDS:
        if name in known:
            kwargs[name] = optional[name]
    try:
        return smol.MachineConfig(**kwargs)
    except (TypeError, ValueError) as exc:
        raise BootError(f"invalid machine config: {exc}", stage="config", cause=exc) from exc


def _exec_outcome(result: Any, argv: Sequence[str]) -> ExecOutcome:
    """Convert ``smol.ExecResult`` (text plus byte-exact fields) to :class:`ExecOutcome`."""
    stdout = getattr(result, "stdout_bytes", b"") or str(result.stdout).encode("utf-8", "replace")
    stderr = getattr(result, "stderr_bytes", b"") or str(result.stderr).encode("utf-8", "replace")
    return ExecOutcome(
        exit_code=int(result.exit_code),
        stdout=bytes(stdout),
        stderr=bytes(stderr),
        argv=tuple(argv),
    )


class SmolHandle:
    """A :class:`MachineHandle` wrapping one ``smol.Machine`` on a known target.

    ``exec_timeout_s`` is what :meth:`exec` passes to the SDK when the caller
    gives no timeout (see :data:`DEFAULT_EXEC_TIMEOUT_S`).
    """

    def __init__(
        self,
        engine: SmolEngine,
        machine: Any,
        target: Target,
        *,
        exec_timeout_s: float = DEFAULT_EXEC_TIMEOUT_S,
    ) -> None:
        self._engine = engine
        self._machine = machine
        self._target: Target = target
        self._exec_timeout_s = exec_timeout_s
        self._deleted = False

    @property
    def machine(self) -> Any:
        """The underlying synchronous ``smol.Machine``."""
        return self._machine

    @property
    def target(self) -> Target:
        """Which backend runs this machine."""
        return self._target

    @property
    def exec_timeout_s(self) -> float:
        """The default timeout :meth:`exec` applies when given none."""
        return self._exec_timeout_s

    @property
    def name(self) -> str:
        return str(self._machine.name)

    @property
    def id(self) -> str:
        return str(self._machine.id)

    def state(self) -> str:
        try:
            return str(self._machine.state())
        except Exception as exc:
            raise _translate(exc, stage="state") from exc

    def ready(self) -> bool:
        try:
            return bool(self._machine.ready())
        except Exception as exc:
            raise _translate(exc, stage="wait") from exc

    def wait_until_ready(self, timeout_s: float, interval_s: float) -> None:
        try:
            self._machine.wait_until_ready(timeout_s, interval_s)
        except Exception as exc:
            mapped = _translate(exc, stage="wait")
            if isinstance(mapped, ReadinessTimeout):
                mapped.timeout_s = timeout_s
            raise mapped from exc

    def exec(
        self,
        argv: Sequence[str],
        *,
        timeout_s: float | None = None,
        user: str | None = None,
    ) -> ExecOutcome:
        smol = _smol()
        effective = self._exec_timeout_s if timeout_s is None else timeout_s
        opts = smol.ExecOptions(timeout=effective, user=user)
        try:
            result = self._machine.exec(list(argv), opts)
        except Exception as exc:
            raise _translate(exc, stage="exec") from exc
        return _exec_outcome(result, argv)

    def read_file(self, path: str) -> bytes:
        try:
            return bytes(self._machine.read_file(path))
        except Exception as exc:
            raise _translate(exc, stage="read_file") from exc

    def write_file(self, path: str, data: bytes, mode: int | None = None) -> None:
        try:
            self._machine.write_file(path, data, mode)
        except Exception as exc:
            raise _translate(exc, stage="write_file") from exc

    def host_port(self, guest: int) -> tuple[str, int]:
        if self._target == "cloud":
            raise NotSupportedError(
                "cloud machines publish no loopback host port; use Engine.open_tunnel()"
            )
        try:
            endpoint = self._machine.endpoint(guest)
        except Exception as exc:
            raise _translate(exc, stage="endpoint") from exc
        return _parse_local_endpoint(str(endpoint.http_url), guest)

    def branch(self, name: str, ports: Sequence[PortMapping] | None = None) -> MachineHandle:
        port_specs = _to_port_specs(ports) if ports else None
        try:
            child = self._machine.branch(name, ports=port_specs, branchable=True)
        except Exception as exc:
            raise _translate(exc, stage="branch") from exc
        return self._engine._adopt(child, self._target, exec_timeout_s=self._exec_timeout_s)

    def checkpoint(self, output: str | None = None, store: str | None = None) -> CheckpointInfo:
        if self._target == "local" and not output:
            raise InvalidConfig("a local checkpoint needs an output .smolcheckpoint path")
        try:
            info = self._machine.checkpoint(output, store=store)
        except Exception as exc:
            raise _translate(exc, stage="checkpoint") from exc
        if self._target == "local":
            locator = str(getattr(info, "path", None) or Path(str(output)).absolute())
            ref = CheckpointRef("file", locator)
        else:
            ref = CheckpointRef("cloud", str(info.id))
        return CheckpointInfo(
            ref=ref,
            size_bytes=_opt_int(getattr(info, "size_bytes", None)),
            store=store,
            machine_id=str(getattr(info, "machine_id", None) or self.id),
            elapsed_ms=_opt_float(getattr(info, "elapsed_ms", None)),
            source_pause_ms=_opt_float(getattr(info, "source_pause_ms", None)),
        )

    def pause(self) -> None:
        try:
            self._machine.pause()
        except Exception as exc:
            raise _translate(exc, stage="pause") from exc

    def resume(self) -> None:
        try:
            self._machine.resume()
        except Exception as exc:
            raise _translate(exc, stage="resume") from exc

    def stop(self) -> None:
        try:
            self._machine.stop()
        except Exception as exc:
            raise _translate(exc, stage="stop") from exc

    def delete(self) -> None:
        if self._deleted:
            return
        try:
            self._machine.delete()
        except Exception as exc:
            raise _translate(exc, stage="delete") from exc
        self._deleted = True
        self._engine._forget(self.id)

    def __repr__(self) -> str:
        return f"SmolHandle({self.id!r}, {self.name!r}, target={self._target!r})"


def _parse_local_endpoint(http_url: str, guest: int) -> tuple[str, int]:
    """Extract ``(host, port)`` from a local ``PortEndpoint.http_url``."""
    parts = urlsplit(http_url)
    try:
        port = parts.port
    except ValueError:
        port = None
    if parts.hostname is None or port is None:
        raise BootError(
            f"cannot read the host port for guest port {guest} from {http_url!r}", stage="endpoint"
        )
    return parts.hostname, port


def _opt_int(value: Any) -> int | None:
    return None if value is None else int(value)


def _opt_float(value: Any) -> float | None:
    return None if value is None else float(value)


class SmolEngine:
    """The :class:`Engine` backed by the Smol SDK; zero-argument so ``get_engine`` can build it.

    Handles created, connected, restored or branched through this engine are
    remembered by machine id so :meth:`open_tunnel` can reach the live
    ``smol.Machine`` a cloud tunnel needs; :meth:`SmolHandle.delete` forgets them.
    """

    def __init__(self) -> None:
        self._handles: dict[str, SmolHandle] = {}
        self._lock = threading.Lock()

    # -- host facts ------------------------------------------------------------------

    def sdk_version(self) -> str:
        try:
            return importlib.metadata.version(SDK_DISTRIBUTION)
        except importlib.metadata.PackageNotFoundError:
            pass
        try:
            return str(_smol().__version__)
        except (NotSupportedError, AttributeError):
            return "unknown"

    def local_availability(self) -> tuple[bool, str | None, str | None]:
        try:
            smol = _smol()
        except NotSupportedError as exc:
            return (False, "RUNTIME_NOT_INSTALLED", str(exc))
        try:
            ok, code, reason = smol.local_availability()
        except Exception as exc:  # the SDK promises never to raise; stay defensive
            return (False, "RUNTIME_NOT_INSTALLED", f"smol.local_availability() failed: {exc}")
        return (
            bool(ok),
            None if code is None else str(code),
            None if reason is None else str(reason),
        )

    def supports_checkpoints(self, target: Target) -> bool:
        return True

    def supports_branch(self, target: Target) -> bool:
        return True

    # -- machines ----------------------------------------------------------------------

    def _conn(self, target: Target) -> Any:
        return _smol().ConnectOptions(target=target)

    def _adopt(
        self, machine: Any, target: Target, *, exec_timeout_s: float = DEFAULT_EXEC_TIMEOUT_S
    ) -> SmolHandle:
        handle = SmolHandle(self, machine, target, exec_timeout_s=exec_timeout_s)
        with self._lock:
            self._handles[handle.id] = handle
        return handle

    @staticmethod
    def _require_tunnel_extra(target: Target) -> None:
        """Fail before a billed cloud machine exists when its tunnel could never open."""
        if target == "cloud" and importlib.util.find_spec("websockets") is None:
            raise NotSupportedError(
                "cloud tunnels need the 'websockets' package; install smoltest[cloud]",
                code="TUNNEL_EXTRA_MISSING",
            )

    def _forget(self, machine_id: str) -> None:
        with self._lock:
            self._handles.pop(machine_id, None)

    def _lookup(self, machine_id: str) -> SmolHandle | None:
        with self._lock:
            return self._handles.get(machine_id)

    def create(self, spec: MachineSpec, target: Target) -> MachineHandle:
        smol = _smol()
        self._require_tunnel_extra(target)
        config = _to_machine_config(spec, target)
        try:
            machine = smol.Machine.create(config, self._conn(target))
        except Exception as exc:
            raise _translate(exc, stage="create") from exc
        return self._adopt(machine, target, exec_timeout_s=spec.exec_timeout_s)

    def connect(self, machine_id: str, target: Target) -> MachineHandle:
        smol = _smol()
        try:
            machine = smol.Machine.connect(machine_id, self._conn(target))
        except Exception as exc:
            raise _translate(exc, stage="connect") from exc
        return self._adopt(machine, target)

    def restore_checkpoint(self, ref: CheckpointRef, name: str, target: Target) -> MachineHandle:
        expected = "file" if target == "local" else "cloud"
        if ref.kind != expected:
            raise InvalidConfig(
                f"a {ref.kind!r} checkpoint cannot be restored on the {target} target"
            )
        smol = _smol()
        self._require_tunnel_extra(target)
        try:
            machine = smol.Machine.restore_checkpoint(ref.locator, name, self._conn(target))
        except Exception as exc:
            raise _translate(exc, stage="restore") from exc
        return self._adopt(machine, target)

    def export_checkpoint(self, source: str, output: str) -> int:
        smol = _smol()
        try:
            return int(smol.Machine.export_checkpoint(source, output))
        except Exception as exc:
            raise _translate(exc, stage="export") from exc

    def prune_checkpoint_store(self, store: str) -> int:
        smol = _smol()
        try:
            return int(smol.Machine.prune_checkpoint_store(store))
        except Exception as exc:
            raise _translate(exc, stage="prune") from exc

    def open_tunnel(
        self, machine_id: str, guest_port: int, target: Target
    ) -> StaticBridge | TunnelBridge:
        handle = self._lookup(machine_id)
        if handle is None:
            # A machine made elsewhere (another process, the console): attach to it.
            attached = self.connect(machine_id, target)
            assert isinstance(attached, SmolHandle)
            handle = attached
        if target == "local":
            host, port = handle.host_port(guest_port)
            return StaticBridge(HostEndpoint(host or LOOPBACK, port))
        self._require_tunnel_extra(target)
        tunnel = importlib.import_module("smol.tunnel")
        machine = handle.machine

        def opener() -> Any:
            return tunnel.open_tunnel(machine, guest_port)

        return TunnelBridge(opener)


__all__ = ["DEFAULT_EXEC_TIMEOUT_S", "SDK_DISTRIBUTION", "SmolEngine", "SmolHandle"]
