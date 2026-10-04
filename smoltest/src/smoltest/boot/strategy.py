"""The boot ladder: branch, restore, cold.

:func:`boot_postgres` resolves the target, tries to restore a cached checkpoint
(seeded key first, then the base key), falls back to a cold ``engine.create``
and populates the cache, opens the host bridge, resyncs the guest clock after a
restore, waits for readiness, seeds, and tears everything down when any step
fails. :func:`branch_from` forks a booted golden into an isolated child.
"""

from __future__ import annotations

import contextlib
import threading
import time
import uuid
from collections.abc import Iterator, Mapping
from dataclasses import dataclass, field
from typing import TYPE_CHECKING, Literal

from .._log import logger
from .._ports import HostEndpoint, pick_free_port
from ..cache import CacheKey, CheckpointBackend, VariantClaim, open_checkpoint_backend
from ..cache.lock import LockTimeout
from ..config import Settings, resolve_target
from ..errors import (
    BootError,
    CacheError,
    InvalidConfig,
    NotSupportedError,
    SmoltestError,
)
from ..transport.base import Bridge, CheckpointRef, Engine, MachineHandle, PortMapping, Target
from ..wait.strategies import (
    EXEC_PROBE_TIMEOUT_S,
    PgIsReadyWaitStrategy,
    ReadinessView,
    WaitStrategy,
    default_wait,
    guest_log_reader,
)
from .spec import LOG_PATH, PostgresSpec, build_machine_spec

if TYPE_CHECKING:
    from ..postgres import Seed

Via = Literal["cold", "restore", "branch"]
Role = Literal["standalone", "golden", "branch"]

TERMINATE_FOREIGN_BACKENDS_SQL = (
    "SELECT pg_terminate_backend(pid) FROM pg_stat_activity "
    "WHERE pid <> pg_backend_pid() AND backend_type = 'client backend'"
)
"""Closes every host connection so a checkpoint never captures one."""

BRANCH_READY_TIMEOUT_S = 30.0
"""Upper bound on the readiness wait after a branch (the child was already running)."""

MAX_UNREADY_RESTORES = 3
"""Restored variants that never become ready before a boot stops trying and boots cold."""

_PORT_CONFLICT_MARKERS = ("port in use", "address already in use", "eaddrinuse", "[bind]")


@dataclass(frozen=True)
class BootInfo:
    """How a machine came to be, for reports, headers and benchmarks."""

    via: Via
    target: Target
    elapsed_s: float
    cache_key: str | None = None
    seeded_key: str | None = None
    variant_port: int | None = None
    restored_from: CheckpointRef | None = None
    populated: bool = False
    seeded: bool = False
    clock_resynced: bool = False
    machine_id: str | None = None
    timings: Mapping[str, float] = field(default_factory=dict)
    restored_key: str | None = None


@dataclass
class BootResult:
    """Everything a facade needs to use and later tear down a booted machine.

    ``claim`` is the cache variant the machine was restored from or populated
    into; ``seed_claim`` the seeded variant populated during this boot. Both are
    released by :meth:`release`, after the bridge is closed and the machine deleted.
    """

    handle: MachineHandle
    bridge: Bridge
    endpoint: HostEndpoint
    info: BootInfo
    spec: PostgresSpec
    target: Target
    claim: VariantClaim | None = None
    seed_claim: VariantClaim | None = None
    backend: CheckpointBackend | None = field(default=None, repr=False)
    _released: bool = field(default=False, init=False, repr=False)
    _lock: threading.Lock = field(default_factory=threading.Lock, init=False, repr=False)

    @property
    def released(self) -> bool:
        """``True`` once :meth:`release` ran."""
        return self._released

    def release(self) -> None:
        """Close the bridge, delete the machine and release the cache claims (idempotent)."""
        with self._lock:
            if self._released:
                return
            self._released = True
        _teardown(self.bridge, self.handle, self.claim, self.seed_claim)


def _is_port_conflict(exc: BaseException) -> bool:
    text = str(exc).lower()
    return any(marker in text for marker in _PORT_CONFLICT_MARKERS)


def _teardown(
    bridge: Bridge | None,
    handle: MachineHandle | None,
    *claims: VariantClaim | None,
) -> None:
    """Best-effort cleanup in the only safe order: bridge, machine, claims."""
    if bridge is not None:
        try:
            bridge.close()
        except Exception:
            logger.exception("closing the bridge failed")
    if handle is not None:
        try:
            handle.delete()
        except Exception:
            logger.exception("deleting machine %s failed", handle.name)
    for claim in claims:
        if claim is not None:
            try:
                claim.release()
            except Exception:
                logger.exception("releasing cache claim %s failed", claim.key)


def _resync_clock(handle: MachineHandle) -> bool:
    """Set the guest clock to the host's; a restored guest wakes up in the past."""
    try:
        outcome = handle.exec(
            ["date", "-s", f"@{int(time.time())}"], timeout_s=EXEC_PROBE_TIMEOUT_S
        )
    except Exception as exc:
        logger.debug("clock resync failed on %s: %s", handle.name, exc)
        return False
    if not outcome.ok:
        logger.debug("clock resync exited %d on %s", outcome.exit_code, handle.name)
    return outcome.ok


def _terminate_foreign_backends(handle: MachineHandle, spec: PostgresSpec) -> None:
    """Close every client connection (seeds may have left one open) before a checkpoint."""
    argv = ["psql", "-tA", "-U", spec.username, "-d", spec.dbname, "-p", str(spec.guest_port)]
    argv.extend(("-c", TERMINATE_FOREIGN_BACKENDS_SQL))
    try:
        outcome = handle.exec(argv, timeout_s=EXEC_PROBE_TIMEOUT_S)
    except Exception as exc:
        logger.warning("could not terminate client backends on %s: %s", handle.name, exc)
        return
    if not outcome.ok:
        logger.warning(
            "terminating client backends on %s exited %d: %s",
            handle.name,
            outcome.exit_code,
            outcome.stderr_text.strip(),
        )


def _machine_name(name: str | None, role: Role) -> str:
    return name or f"smoltest-pg-{role}-{uuid.uuid4().hex[:8]}"


class _Timer:
    """Accumulates per-stage durations for :attr:`BootInfo.timings`."""

    def __init__(self) -> None:
        self.started = time.perf_counter()
        self.timings: dict[str, float] = {}

    @contextlib.contextmanager
    def stage(self, name: str) -> Iterator[None]:
        t0 = time.perf_counter()
        try:
            yield
        finally:
            self.timings[name] = self.timings.get(name, 0.0) + (time.perf_counter() - t0)

    @property
    def elapsed(self) -> float:
        return time.perf_counter() - self.started


class _Boot:
    """One run of the ladder; state shared by its steps and the cleanup."""

    def __init__(
        self,
        spec: PostgresSpec,
        settings: Settings,
        engine: Engine,
        *,
        seed: Seed | None,
        wait: WaitStrategy | None,
        role: Role,
        name: str | None,
        host_port: int | None,
    ) -> None:
        self.spec = spec
        self.settings = settings
        self.engine = engine
        self.seed = seed
        self.wait = wait if wait is not None else default_wait(spec.username, port=spec.guest_port)
        self.role = role
        self.name = _machine_name(name, role)
        self.target: Target = resolve_target(settings, engine)
        self.local = self.target == "local"
        if host_port is not None and not self.local:
            # Silently booting on another port would break every client configured
            # for the one asked for; the tunnel chooses its own loopback port.
            raise NotSupportedError(
                f"a fixed host port ({host_port}) is not supported on the cloud target: "
                "the tunnel chooses its loopback port; drop the port pin or use the local target"
            )
        self.pinned_port = host_port
        use_cache = not settings.disable_cache and engine.supports_checkpoints(self.target)
        self.backend: CheckpointBackend | None = (
            open_checkpoint_backend(settings, self.target) if use_cache else None
        )
        self.base_key: CacheKey | None = (
            CacheKey.compute(spec, settings, self.target, engine) if use_cache else None
        )
        self.seeded_key: CacheKey | None = (
            self.base_key.with_seed(seed.key) if self.base_key is not None and seed else None
        )
        self.timer = _Timer()
        self.stage = "resolve"
        self.handle: MachineHandle | None = None
        self.bridge: Bridge | None = None
        self.endpoint: HostEndpoint | None = None
        self.claim: VariantClaim | None = None
        self.seed_claim: VariantClaim | None = None
        self.via: Via = "cold"
        self.restored_key: CacheKey | None = None
        self.populated = False
        self.seeded = False
        self.clock_resynced = False
        self.port: int | None = None
        self.cache_variant = True  # False: boot cold without writing the base key
        self.unready_restores = 0

    # -- the ladder --------------------------------------------------------------------

    def run(self) -> BootResult:
        try:
            if self.seeded_key is not None and self._try_restore(self.seeded_key):
                self._ready_restored()  # on failure falls through to the base key
            while self.handle is None:
                self._restore_or_create_base()
                if self.via == "restore":
                    self._ready_restored()  # on failure: next variant, else cold boot
            handle = self.handle
            assert handle is not None
            self._open_bridge(handle)  # a no-op when the ladder already opened it
            if self.via == "restore" and self.backend is not None and self.claim is not None:
                with contextlib.suppress(SmoltestError):
                    self.backend.touch(self.claim)
            result = self._result()
            if self.seed is not None and self._needs_seed():
                self._seed(result)
                result.seed_claim = self.seed_claim
            result.info = self._info(handle)
            logger.info(
                "postgres %s booted via %s on %s in %.2fs",
                handle.name,
                self.via,
                self.target,
                result.info.elapsed_s,
            )
        except BootError:
            self._cleanup()
            raise
        except Exception as exc:
            self._cleanup()
            raise BootError(f"{type(exc).__name__}: {exc}", stage=self.stage, cause=exc) from exc
        except BaseException:  # KeyboardInterrupt / SystemExit: release, then propagate
            self._cleanup()
            raise
        return result

    def _cleanup(self) -> None:
        _teardown(self.bridge, self.handle, self.claim, self.seed_claim)
        self.bridge = self.handle = self.claim = self.seed_claim = None
        self.endpoint = None

    def _needs_seed(self) -> bool:
        restored, seeded = self.restored_key, self.seeded_key
        return restored is None or seeded is None or restored.key != seeded.key

    def _try_restore(self, key: CacheKey) -> bool:
        """Claim a variant of ``key`` and restore it; ``True`` when a machine came up."""
        backend = self.backend
        if backend is None or self.unready_restores >= MAX_UNREADY_RESTORES:
            return False
        self.stage = "claim"
        claim = backend.claim_variant(key.key, self.pinned_port)
        if claim is None:
            return False
        if claim.ref is None:
            claim.release()
            return False
        self.stage = "restore"
        try:
            with self.timer.stage("restore"):
                handle = self.engine.restore_checkpoint(
                    claim.ref,
                    self.name,
                    self.target,
                    exec_timeout_s=self.settings.exec_timeout_s,
                )
        except SmoltestError as exc:
            if _is_port_conflict(exc):
                logger.info("variant %s:%s lost its port, skipping: %s", key, claim.port, exc)
                claim.release()
            else:
                logger.warning("restoring %s:%s failed, invalidating: %s", key, claim.port, exc)
                backend.invalidate(claim)
            return False
        self.handle = handle
        self.claim = claim
        self.via = "restore"
        self.restored_key = key
        self.port = claim.port
        return True

    def _ready_restored(self) -> bool:
        """Bridge, clock and readiness for a restored machine.

        A :class:`~smoltest.errors.ReadinessTimeout`, or any other smoltest error
        the probe raises (a guest agent that refuses every command, a machine that
        died on resume), means the variant is poisoned: the machine is deleted,
        the variant invalidated and the ladder state reset so the caller tries the
        next rung. A strategy refusing this machine
        (:class:`~smoltest.errors.NotSupportedError`,
        :class:`~smoltest.errors.InvalidConfig`), a bridge that will not open or a
        non-smoltest exception is not the checkpoint's fault: the machine is
        deleted and the claim released, nothing is invalidated and the error
        propagates.
        """
        handle = self.handle
        assert handle is not None
        self._open_bridge(handle)
        self.stage = "clock"
        self.clock_resynced = _resync_clock(handle)
        try:
            self._wait(handle)
        except (NotSupportedError, InvalidConfig):
            self._discard_restored(invalidate=False)
            raise
        except SmoltestError as exc:
            self.unready_restores += 1
            logger.warning(
                "restored %s:%s is unusable (%s: %s), invalidating",
                self.restored_key,
                self.port,
                type(exc).__name__,
                exc,
            )
            self._discard_restored(invalidate=True)
            return False
        except Exception:
            self._discard_restored(invalidate=False)
            raise
        return True

    def _discard_restored(self, *, invalidate: bool) -> None:
        """Delete the restored machine, release its claim (invalidating it if asked), reset."""
        backend, claim = self.backend, self.claim
        _teardown(self.bridge, self.handle)
        if claim is not None:
            try:
                if invalidate and backend is not None:
                    backend.invalidate(claim)
            except SmoltestError as exc:
                logger.debug("could not invalidate %s:%s: %s", claim.key, claim.port, exc)
            finally:
                claim.release()
        self.handle = self.bridge = self.endpoint = self.claim = None
        self.via, self.restored_key, self.port, self.clock_resynced = "cold", None, None, False

    def _restore_or_create_base(self) -> None:
        backend, key = self.backend, self.base_key
        if backend is None or key is None:
            self._create_and_wait()
            return
        # Holding the key lock across claim, create and populate means a second
        # process booting the same key waits and then restores instead of booting
        # cold too. The lock is re-entrant, so the backend's own locking nests.
        self.stage = "lock"
        lock = backend.key_lock(key.key)
        try:
            lock.acquire()
        except LockTimeout as exc:
            # The holder's cold boot (image pull included) is not bounded by any
            # timeout of ours. Boot our own machine and leave the key alone:
            # reservation and population would need the same lock.
            logger.warning(
                "cache key %s stayed locked for %gs (another process is populating it); "
                "booting cold without caching: %s",
                key,
                exc.timeout_s,
                exc,
            )
            self.cache_variant = False
            self._create_and_wait()
            return
        try:
            if self._try_restore(key):
                return
            self._create_and_wait()
            handle = self.handle
            assert handle is not None
            self._populate(key, handle)
        finally:
            lock.release()

    def _create_and_wait(self) -> None:
        """Cold boot, open the bridge and run the readiness strategy."""
        self._create()
        handle = self.handle
        assert handle is not None
        self._open_bridge(handle)
        self._wait(handle)

    def _reserve(self, port: int | None) -> VariantClaim | None:
        backend, key = self.backend, self.base_key
        if backend is None or key is None or not self.cache_variant:
            return None
        return backend.reserve_new_variant(key.key, port)

    def _create(self) -> None:
        self.stage = "create"
        attempts = 2 if self.local and self.pinned_port is None else 1
        for attempt in range(1, attempts + 1):
            port = self.pinned_port
            if port is None and self.local:
                port = pick_free_port()
            try:
                self.claim = self._reserve(port)
                mspec = build_machine_spec(self.spec, self.settings, self.target, port, self.name)
                with self.timer.stage("create"):
                    self.handle = self.engine.create(mspec, self.target)
            except (CacheError, BootError) as exc:
                if self.claim is not None:
                    self.claim.release()
                    self.claim = None
                retry = attempt < attempts and (
                    isinstance(exc, CacheError) or _is_port_conflict(exc)
                )
                if not retry:
                    raise
                logger.info("host port %s was taken while booting, retrying: %s", port, exc)
                continue
            self.via = "cold"
            self.port = port
            return

    def _open_bridge(self, handle: MachineHandle) -> None:
        if self.bridge is not None:
            return
        self.stage = "tunnel"
        with self.timer.stage("tunnel"):
            bridge = self.engine.open_tunnel(handle.id, self.spec.guest_port, self.target)
            self.bridge = bridge
            self.endpoint = bridge.open()

    def _view(self, handle: MachineHandle) -> ReadinessView:
        endpoint = self.endpoint
        assert endpoint is not None
        return ReadinessView(
            handle=handle,
            endpoint=endpoint,
            username=self.spec.username,
            password=self.spec.password,
            dbname=self.spec.dbname,
            guest_port=self.spec.guest_port,
            logs=guest_log_reader(handle, LOG_PATH) if self.spec.capture_logs else None,
        )

    def _wait(self, handle: MachineHandle) -> None:
        self.stage = "wait"
        with self.timer.stage("wait"):
            self.wait.wait_until_ready(
                self._view(handle), self.settings.ready_timeout_s, self.settings.poll_interval_s
            )

    def _populate(self, key: CacheKey, handle: MachineHandle) -> None:
        backend, claim = self.backend, self.claim
        if backend is None or claim is None:
            return
        self.stage = "populate"
        with self.timer.stage("populate"):
            backend.populate(key.key, claim, handle, key.inputs, engine=self.engine)
        self.populated = True

    def _seed(self, result: BootResult) -> None:
        seed = self.seed
        assert seed is not None
        handle = result.handle
        # A local import: postgres.py imports this module at load time.
        from ..postgres import PostgresMachine  # noqa: PLC0415

        facade = PostgresMachine._from_boot(
            result, self.spec, self.settings, self.engine, register_finalizer=False
        )
        self.stage = "seed"
        with self.timer.stage("seed"):
            seed.apply(facade)
            _terminate_foreign_backends(handle, self.spec)
            self._wait(handle)
        self.seeded = True
        backend, key = self.backend, self.seeded_key
        if backend is None or key is None:
            return
        self.stage = "populate_seed"
        with self.timer.stage("populate_seed"):
            lock = backend.key_lock(key.key)
            try:
                lock.acquire()
            except LockTimeout as exc:
                logger.warning(
                    "not caching seeded variant %s:%s (key busy): %s", key, self.port, exc
                )
                return
            try:
                self._populate_seeded(backend, key, handle)
            finally:
                lock.release()

    def _populate_seeded(
        self, backend: CheckpointBackend, key: CacheKey, handle: MachineHandle
    ) -> None:
        try:
            claim = backend.reserve_new_variant(key.key, self.port)
        except CacheError as exc:
            logger.warning("not caching seeded variant %s:%s: %s", key, self.port, exc)
            return
        try:
            backend.populate(key.key, claim, handle, key.inputs, engine=self.engine)
        except SmoltestError as exc:
            # The machine is seeded and ready; a failed capture is a cache miss, not
            # a boot failure. Drop the half-written variant so the next boot does not
            # find a stale claim, and hand the machine out uncached.
            logger.warning(
                "could not cache seeded variant %s:%s, continuing uncached: %s",
                key,
                self.port,
                exc,
            )
            try:
                backend.invalidate(claim)
            except SmoltestError as inner:
                logger.debug("could not invalidate %s:%s: %s", key, self.port, inner)
            finally:
                claim.release()
            return
        self.seed_claim = claim
        self.populated = True

    # -- results -----------------------------------------------------------------------

    def _info(self, handle: MachineHandle) -> BootInfo:
        restored = self.restored_key
        return BootInfo(
            via=self.via,
            target=self.target,
            elapsed_s=self.timer.elapsed,
            cache_key=self.base_key.key if self.base_key is not None else None,
            seeded_key=self.seeded_key.key if self.seeded_key is not None else None,
            variant_port=self.port if self.local else None,
            restored_from=self.claim.ref if restored is not None and self.claim else None,
            populated=self.populated,
            seeded=self.seeded,
            clock_resynced=self.clock_resynced,
            machine_id=handle.id,
            timings=dict(self.timer.timings),
            restored_key=restored.key if restored is not None else None,
        )

    def _result(self) -> BootResult:
        handle, bridge, endpoint = self.handle, self.bridge, self.endpoint
        assert handle is not None and bridge is not None and endpoint is not None
        return BootResult(
            handle=handle,
            bridge=bridge,
            endpoint=endpoint,
            info=self._info(handle),
            spec=self.spec,
            target=self.target,
            claim=self.claim,
            seed_claim=self.seed_claim,
            backend=self.backend,
        )


def boot_postgres(
    spec: PostgresSpec,
    settings: Settings,
    engine: Engine,
    *,
    seed: Seed | None = None,
    wait: WaitStrategy | None = None,
    role: Role = "standalone",
    name: str | None = None,
    host_port: int | None = None,
) -> BootResult:
    """Boot PostgreSQL the fastest way available and return it ready.

    Resolves the target, tries cached checkpoint variants (the seeded key, then
    the base key), falls back to a cold ``engine.create`` on ``host_port`` (or a
    free port) and populates the cache, opens the bridge, resyncs the guest clock
    after a restore, runs ``wait`` (default ``pg_isready``), applies ``seed`` when
    the restored checkpoint did not already include it and caches the result, and
    tears everything down on failure, raising :class:`~smoltest.errors.BootError`.
    :class:`~smoltest.errors.TargetUnavailable` propagates unchanged.
    """
    boot = _Boot(
        spec,
        settings,
        engine,
        seed=seed,
        wait=wait,
        role=role,
        name=name,
        host_port=host_port,
    )
    return boot.run()


def _branch_ports(spec: PostgresSpec, target: Target) -> list[PortMapping] | None:
    if target != "local":
        return None
    picked: list[int] = []
    ports: list[PortMapping] = []
    for guest in (spec.guest_port, *(p.guest for p in spec.extra_ports)):
        port = pick_free_port(exclude=picked)
        picked.append(port)
        ports.append(PortMapping(port, guest))
    return ports


def _branch_once(result: BootResult, name: str) -> MachineHandle:
    """One ``handle.branch`` with fresh host ports; errors become ``BootError``.

    :class:`~smoltest.errors.NotSupportedError` passes through untouched.
    """
    try:
        return result.handle.branch(name, _branch_ports(result.spec, result.target))
    except (NotSupportedError, BootError):
        raise
    except SmoltestError as exc:
        raise BootError(f"{type(exc).__name__}: {exc}", stage="branch", cause=exc) from exc


def _branch_handle(result: BootResult, name: str) -> MachineHandle:
    """``handle.branch``, retried once when the picked host port was taken meanwhile."""
    try:
        return _branch_once(result, name)
    except BootError as exc:
        if not _is_port_conflict(exc):
            raise
        logger.info("branch port was taken, retrying: %s", exc)
    return _branch_once(result, name)


def branch_from(
    result: BootResult,
    name: str,
    settings: Settings,
    engine: Engine,
) -> BootResult:
    """Branch a booted golden into an isolated child with its own host port.

    Pins a fresh ``PortMapping(pick_free_port(), guest_port)`` on the local target
    (auto on cloud), retries once on a port conflict, opens a bridge to the child,
    waits briefly for ``pg_isready`` and returns a :class:`BootResult` whose
    ``info.via`` is ``"branch"``. :class:`~smoltest.errors.NotSupportedError`
    propagates unchanged so callers can fall back to a fresh machine.
    """
    spec, target = result.spec, result.target
    timer = _Timer()
    with timer.stage("branch"):
        child = _branch_handle(result, name)
    bridge: Bridge | None = None
    try:
        with timer.stage("tunnel"):
            bridge = engine.open_tunnel(child.id, spec.guest_port, target)
            endpoint = bridge.open()
        view = ReadinessView(
            handle=child,
            endpoint=endpoint,
            username=spec.username,
            password=spec.password,
            dbname=spec.dbname,
            guest_port=spec.guest_port,
            logs=guest_log_reader(child, LOG_PATH) if spec.capture_logs else None,
        )
        with timer.stage("wait"):
            PgIsReadyWaitStrategy(spec.username, port=spec.guest_port).wait_until_ready(
                view,
                min(settings.ready_timeout_s, BRANCH_READY_TIMEOUT_S),
                settings.poll_interval_s,
            )
    except BootError:
        _teardown(bridge, child)
        raise
    except Exception as exc:
        _teardown(bridge, child)
        raise BootError(f"{type(exc).__name__}: {exc}", stage="branch", cause=exc) from exc
    except BaseException:  # KeyboardInterrupt / SystemExit: release, then propagate
        _teardown(bridge, child)
        raise
    info = BootInfo(
        via="branch",
        target=target,
        elapsed_s=timer.elapsed,
        cache_key=result.info.cache_key,
        seeded_key=result.info.seeded_key,
        variant_port=endpoint.port if target == "local" else None,
        seeded=result.info.seeded,
        machine_id=child.id,
        timings=dict(timer.timings),
        restored_key=result.info.restored_key,
    )
    logger.info(
        "postgres %s branched from %s in %.3fs", child.name, result.handle.name, info.elapsed_s
    )
    return BootResult(child, bridge, endpoint, info, spec, target)


__all__ = [
    "BRANCH_READY_TIMEOUT_S",
    "MAX_UNREADY_RESTORES",
    "TERMINATE_FOREIGN_BACKENDS_SQL",
    "BootInfo",
    "BootResult",
    "Role",
    "Via",
    "boot_postgres",
    "branch_from",
]
