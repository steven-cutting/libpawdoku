"""The on-disk checkpoint cache for the local target.

Layout under ``<cache_dir>/<family>/`` (``family`` is ``postgres``)::

    .lock                       store-wide lock (eviction, prune, clear)
    <key>.lock                  per-key lock (claims, populate, invalidate)
    store/                      the engine's dedup store
    <key>/inputs.json           what produced the key, passwords redacted
    <key>/<port>.smolcheckpoint the checkpoint (a file or a directory)
    <key>/<port>.meta.json      commit marker: written last, read first
    <key>/<port>.claim          pid + token of the process using the variant

A *variant* is one checkpoint of a key bound to one host port, because a local
checkpoint restores on the port it was taken with. Directories are ``0o700``
and files ``0o600``.

Lock order is always key lock, then store lock. Hold ``key_lock(key)`` across
``claim_variant`` → cold boot → ``populate`` so concurrent processes wait for the
checkpoint instead of booting cold as well.
"""

from __future__ import annotations

import contextlib
import json
import os
import shutil
import stat
import time
import uuid
import weakref
from collections.abc import Iterable, Mapping
from dataclasses import dataclass, field, replace
from pathlib import Path
from typing import Any, Protocol, runtime_checkable

from .._lifecycle import Finalizer
from .._log import logger, warn_once
from .._ports import is_port_free
from ..config import Settings
from ..errors import CacheCorrupt, CacheError, NotSupportedError, SmoltestError
from ..transport.base import (
    CheckpointInfo,
    CheckpointKind,
    CheckpointRef,
    Engine,
    MachineHandle,
    Target,
)
from .key import host_signature, redact_inputs
from .lock import DEFAULT_POLL_S, DEFAULT_TIMEOUT_S, FileLock

META_FORMAT = 1
INPUTS_FORMAT = 1
CHECKPOINT_SUFFIX = ".smolcheckpoint"
META_SUFFIX = ".meta.json"
CLAIM_SUFFIX = ".claim"
LOCK_SUFFIX = ".lock"
STORE_DIRNAME = "store"
STORE_LOCK_NAME = ".lock"
INPUTS_FILENAME = "inputs.json"
DIR_MODE = 0o700
FILE_MODE = 0o600

# -- filesystem helpers -------------------------------------------------------------


def ensure_dir(path: Path, *, chmod_existing: bool = True) -> Path:
    """Create ``path`` (and parents) and make it private to the user.

    ``chmod_existing=False`` leaves the permissions of a directory that already
    existed alone: the cache root may be a directory the user owns for other
    purposes, and smoltest must not strip group and world access from it.
    """
    existed = path.is_dir()
    path.mkdir(parents=True, exist_ok=True, mode=DIR_MODE)
    if chmod_existing or not existed:
        with contextlib.suppress(OSError):
            path.chmod(DIR_MODE)
    return path


def write_private(path: Path, data: bytes) -> None:
    """Atomically replace ``path`` with ``data`` in a ``0o600`` file."""
    tmp = path.with_name(f".{path.name}.{os.getpid()}.{uuid.uuid4().hex[:8]}.tmp")
    fd = os.open(tmp, os.O_WRONLY | os.O_CREAT | os.O_EXCL | os.O_CLOEXEC, FILE_MODE)
    try:
        with os.fdopen(fd, "wb") as fh:
            fh.write(data)
            fh.flush()
            os.fsync(fh.fileno())
        tmp.replace(path)
    except BaseException:
        with contextlib.suppress(OSError):
            tmp.unlink()
        raise


def make_private(path: Path) -> None:
    """Chmod ``path`` to ``0o600`` (files) / ``0o700`` (directories), recursively.

    Raises :class:`~smoltest.errors.CacheError` (code ``CHECKPOINT_PERMISSIONS``)
    when a ``chmod`` fails or does not take effect, which a re-``stat`` checks:
    every caller holds a copy of guest RAM that must not be handed out with the
    permissions the engine's umask gave it. Symbolic links inside a tree are left
    alone (their targets are not the checkpoint's).
    """

    def unreadable(exc: OSError) -> None:  # a directory the walk could not list
        raise CacheError(
            f"cannot make {path} private: {exc}", code="CHECKPOINT_PERMISSIONS"
        ) from exc

    targets: list[tuple[Path, int]] = []
    if path.is_dir() and not path.is_symlink():
        targets.append((path, DIR_MODE))
        for root, dirs, files in os.walk(path, onerror=unreadable):
            targets.extend((Path(root, name), DIR_MODE) for name in dirs)
            targets.extend((Path(root, name), FILE_MODE) for name in files)
    else:
        targets.append((path, FILE_MODE))
    for target, mode in targets:
        if target is not path and target.is_symlink():
            continue
        try:
            target.chmod(mode)
            actual = stat.S_IMODE(target.stat().st_mode)
        except OSError as exc:
            raise CacheError(
                f"cannot make {target} private: {exc}", code="CHECKPOINT_PERMISSIONS"
            ) from exc
        if actual & 0o077:  # group or world bits survived: a mount with a fixed mask, say
            raise CacheError(
                f"cannot make {target} private: mode is {actual:#o} after chmod {mode:#o}",
                code="CHECKPOINT_PERMISSIONS",
            )


def remove_path_strict(path: Path) -> None:
    """:func:`remove_path`, raising :class:`~smoltest.errors.CacheError` when it cannot.

    For a checkpoint that could not be made private: a removal that fails leaves
    guest RAM readable, which the caller must hear about rather than a bare
    ``OSError`` that hides the permission failure it was cleaning up after.
    """
    try:
        remove_path(path)
    except OSError as exc:
        raise CacheError(
            f"could not remove {path}, which is still readable: {exc}",
            code="CHECKPOINT_PERMISSIONS",
        ) from exc


def _private_or_removed(path: Path, remove: Path) -> None:
    """``make_private(path)``, deleting ``remove`` on failure: guest RAM never stays readable."""
    try:
        make_private(path)
    except CacheError:
        remove_path_strict(remove)
        raise


def path_size(path: Path) -> int:
    """Bytes used by ``path``: its size for a file, the sum of its files for a directory."""
    try:
        st = path.lstat()
    except OSError:
        return 0
    if not stat.S_ISDIR(st.st_mode):
        return st.st_size
    total = 0
    for root, _dirs, files in os.walk(path):
        for name in files:
            with contextlib.suppress(OSError):
                total += Path(root, name).lstat().st_size
    return total


def remove_path(path: Path) -> bool:
    """Delete a file, symlink or directory tree; ``True`` when something was removed."""
    try:
        if path.is_dir() and not path.is_symlink():
            shutil.rmtree(path)
        else:
            path.unlink()
    except FileNotFoundError:
        return False
    return True


def pid_alive(pid: int) -> bool:
    """``True`` when a process with ``pid`` exists (even one we may not signal)."""
    if pid <= 0:
        return False
    try:
        os.kill(pid, 0)
    except ProcessLookupError:
        return False
    except PermissionError:
        return True
    return True


def _json_bytes(value: Any) -> bytes:
    return (json.dumps(value, sort_keys=True, indent=1) + "\n").encode("utf-8")


# -- records ------------------------------------------------------------------------


@dataclass(frozen=True)
class VariantPaths:
    """The three files of one variant."""

    checkpoint: Path
    meta: Path
    claim: Path


@dataclass(frozen=True)
class ClaimRecord:
    """What a ``<port>.claim`` file says."""

    pid: int
    token: str
    created_at: float

    @property
    def alive(self) -> bool:
        """``True`` while the claiming process exists."""
        return pid_alive(self.pid)


def read_claim(path: Path) -> ClaimRecord | None:
    """Parse a claim file; ``None`` when it is missing or unreadable."""
    try:
        text = path.read_text("utf-8")
    except OSError:
        return None
    try:
        data = json.loads(text)
        if isinstance(data, dict):
            return ClaimRecord(
                int(data["pid"]), str(data.get("token", "")), float(data.get("created_at", 0.0))
            )
        return ClaimRecord(int(data), "", 0.0)
    except (ValueError, TypeError, KeyError):
        return None


def _unlink_claim(path: Path, token: str) -> None:
    record = read_claim(path)
    if record is not None and record.token == token:
        with contextlib.suppress(OSError):
            path.unlink()


_claims: weakref.WeakSet[VariantClaim] = weakref.WeakSet()
"""Every claim with a finalizer, so a forked child can disown the parent's."""


def _disown_inherited_claims() -> None:
    """In a forked child, the parent's claim files are the parent's to remove.

    The child inherits each claim's finalizer; garbage collection or weakref's
    exit hook there would unlink a claim file the parent still relies on.
    """
    for claim in list(_claims):
        if claim._finalizer is not None:
            claim._finalizer.detach()


if hasattr(os, "register_at_fork"):  # POSIX; elsewhere there is no fork to survive
    os.register_at_fork(after_in_child=_disown_inherited_claims)


@dataclass(frozen=True)
class VariantMeta:
    """The commit marker of a populated variant (``<port>.meta.json``)."""

    key: str
    port: int | None
    ref: CheckpointRef
    created_at: float
    last_used_at: float
    size_bytes: int
    machine_id: str | None = None
    sdk_version: str | None = None
    branch_ok: bool | None = None
    format: int = META_FORMAT

    def to_dict(self) -> dict[str, Any]:
        """JSON-ready mapping."""
        return {
            "format": self.format,
            "key": self.key,
            "port": self.port,
            "ref": {"kind": self.ref.kind, "locator": self.ref.locator},
            "created_at": self.created_at,
            "last_used_at": self.last_used_at,
            "size_bytes": self.size_bytes,
            "machine_id": self.machine_id,
            "sdk_version": self.sdk_version,
            "branch_ok": self.branch_ok,
        }

    def to_json(self) -> bytes:
        """Serialised :meth:`to_dict`."""
        return _json_bytes(self.to_dict())

    @classmethod
    def from_dict(cls, data: Any, *, source: str = "meta") -> VariantMeta:
        """Validate a parsed meta mapping; raise :class:`CacheCorrupt` when it is not one."""
        if not isinstance(data, dict):
            raise CacheCorrupt(f"{source}: expected an object", code="META_SHAPE")
        if data.get("format") != META_FORMAT:
            found = data.get("format")
            raise CacheCorrupt(f"{source}: unsupported format {found!r}", code="META_FORMAT")
        try:
            ref = data["ref"]
            kind = ref["kind"]
            if kind not in ("file", "cloud"):
                raise CacheCorrupt(f"{source}: bad checkpoint kind {kind!r}", code="META_SHAPE")
            port = data.get("port")
            return cls(
                key=str(data["key"]),
                port=None if port is None else int(port),
                ref=CheckpointRef(kind, str(ref["locator"])),
                created_at=float(data["created_at"]),
                last_used_at=float(data["last_used_at"]),
                size_bytes=int(data["size_bytes"]),
                machine_id=None if data.get("machine_id") is None else str(data["machine_id"]),
                sdk_version=None if data.get("sdk_version") is None else str(data["sdk_version"]),
                branch_ok=None if data.get("branch_ok") is None else bool(data["branch_ok"]),
            )
        except (KeyError, TypeError, ValueError) as exc:
            raise CacheCorrupt(f"{source}: {exc!r}", code="META_SHAPE") from exc

    @classmethod
    def load(cls, path: Path) -> VariantMeta:
        """Read and validate ``path``; raise :class:`CacheCorrupt` on any problem."""
        try:
            data = json.loads(path.read_text("utf-8"))
        except OSError as exc:
            raise CacheCorrupt(f"{path}: unreadable: {exc}", code="META_UNREADABLE") from exc
        except ValueError as exc:
            raise CacheCorrupt(f"{path}: not JSON: {exc}", code="META_JSON") from exc
        return cls.from_dict(data, source=str(path))


@dataclass(eq=False)
class VariantClaim:
    """Exclusive use of one variant by this process, until :meth:`release`.

    A local claim is backed by a ``<port>.claim`` file that holds this process's
    pid and a random token; the file is removed by :meth:`release`, when the
    claim object is garbage collected, or at interpreter exit. ``meta`` is
    ``None`` for a variant reserved but not yet populated.
    """

    key: str
    port: int | None
    ref: CheckpointRef | None
    meta: VariantMeta | None = None
    kind: CheckpointKind = "file"
    claim_path: Path | None = None
    token: str | None = None
    _finalizer: Finalizer | None = field(default=None, repr=False)
    _released: bool = field(default=False, repr=False)

    def __post_init__(self) -> None:
        if self.claim_path is not None and self.token is not None:
            self._finalizer = weakref.finalize(self, _unlink_claim, self.claim_path, self.token)
            _claims.add(self)

    @property
    def populated(self) -> bool:
        """``True`` when the variant holds a committed checkpoint."""
        return self.meta is not None

    @property
    def released(self) -> bool:
        """``True`` once :meth:`release` ran."""
        return self._released

    def release(self) -> None:
        """Drop the claim (idempotent). The checkpoint stays for the next claimant."""
        if self._finalizer is not None:
            self._finalizer()
        self._released = True

    def __enter__(self) -> VariantClaim:
        return self

    def __exit__(self, *exc: object) -> None:
        self.release()


# -- summaries ----------------------------------------------------------------------


@dataclass(frozen=True)
class VariantEntry:
    """One variant as :meth:`CheckpointCache.ls` reports it."""

    key: str
    port: int | None
    ref: CheckpointRef | None
    size_bytes: int
    populated: bool
    corrupt: bool = False
    created_at: float | None = None
    last_used_at: float | None = None
    claimed_by: int | None = None
    claim_live: bool = False
    machine_id: str | None = None
    sdk_version: str | None = None
    branch_ok: bool | None = None

    def to_dict(self) -> dict[str, Any]:
        """JSON-ready mapping."""
        ref = None if self.ref is None else {"kind": self.ref.kind, "locator": self.ref.locator}
        return {
            "key": self.key,
            "port": self.port,
            "ref": ref,
            "size_bytes": self.size_bytes,
            "populated": self.populated,
            "corrupt": self.corrupt,
            "created_at": self.created_at,
            "last_used_at": self.last_used_at,
            "claimed_by": self.claimed_by,
            "claim_live": self.claim_live,
            "machine_id": self.machine_id,
            "sdk_version": self.sdk_version,
            "branch_ok": self.branch_ok,
        }


@dataclass(frozen=True)
class KeyEntry:
    """One key with its (redacted) inputs and variants."""

    key: str
    inputs: Mapping[str, Any] | None
    variants: tuple[VariantEntry, ...]

    @property
    def size_bytes(self) -> int:
        """Bytes used by the key's variants."""
        return sum(v.size_bytes for v in self.variants)

    def to_dict(self) -> dict[str, Any]:
        """JSON-ready mapping."""
        return {
            "key": self.key,
            "inputs": None if self.inputs is None else dict(self.inputs),
            "size_bytes": self.size_bytes,
            "variants": [v.to_dict() for v in self.variants],
        }


@dataclass(frozen=True)
class CacheSummary:
    """What a cache or index holds, for ``smoltest cache ls`` and ``doctor``."""

    kind: CheckpointKind
    root: Path
    keys: tuple[KeyEntry, ...]
    store_bytes: int = 0
    max_bytes: int | None = None

    @property
    def total_bytes(self) -> int:
        """Variants plus the dedup store."""
        return sum(k.size_bytes for k in self.keys) + self.store_bytes

    @property
    def variant_count(self) -> int:
        """How many variants (populated or not) exist."""
        return sum(len(k.variants) for k in self.keys)

    def to_dict(self) -> dict[str, Any]:
        """JSON-ready mapping."""
        return {
            "kind": self.kind,
            "root": str(self.root),
            "keys": [k.to_dict() for k in self.keys],
            "store_bytes": self.store_bytes,
            "total_bytes": self.total_bytes,
            "variant_count": self.variant_count,
            "max_bytes": self.max_bytes,
        }


@dataclass(frozen=True)
class PruneReport:
    """What :meth:`CheckpointCache.prune` or :meth:`CheckpointCache.clear` removed."""

    removed: tuple[VariantEntry, ...] = ()
    freed_bytes: int = 0
    removed_keys: tuple[str, ...] = ()
    store_chunks_pruned: int = 0
    claims_removed: int = 0

    def to_dict(self) -> dict[str, Any]:
        """JSON-ready mapping."""
        return {
            "removed": [v.to_dict() for v in self.removed],
            "freed_bytes": self.freed_bytes,
            "removed_keys": list(self.removed_keys),
            "store_chunks_pruned": self.store_chunks_pruned,
            "claims_removed": self.claims_removed,
        }


# -- the common interface -----------------------------------------------------------


@runtime_checkable
class CheckpointBackend(Protocol):
    """What :class:`CheckpointCache` and ``CloudCheckpointIndex`` share.

    ``boot/strategy.py`` codes against this: claim a variant to restore, or
    reserve one and populate it after a cold boot, invalidate a variant that
    failed to restore, touch one that worked, and report or clear the lot.
    """

    @property
    def kind(self) -> CheckpointKind: ...

    def key_lock(self, key: str) -> FileLock: ...

    def claim_variant(self, key: str, pinned_port: int | None = None) -> VariantClaim | None: ...

    def reserve_new_variant(self, key: str, port: int | None) -> VariantClaim: ...

    def populate(
        self,
        key: str,
        claim: VariantClaim,
        handle: MachineHandle,
        inputs: Mapping[str, Any] | None = None,
        *,
        engine: Engine | None = None,
    ) -> VariantMeta: ...

    def invalidate(self, claim: VariantClaim) -> None: ...

    def touch(self, claim: VariantClaim) -> None: ...

    def record_branch_ok(self, claim: VariantClaim, ok: bool) -> None: ...

    def ls(self) -> CacheSummary: ...

    def prune(
        self,
        older_than_s: float | None = None,
        keep_latest: int | None = None,
        max_bytes: int | None = None,
        stale: bool = False,
        engine: Engine | None = None,
    ) -> PruneReport: ...

    def clear(self) -> PruneReport: ...


# -- the store ----------------------------------------------------------------------


@dataclass
class _Scanned:
    """A variant as found on disk (internal)."""

    key: str
    port: int
    paths: VariantPaths
    has_checkpoint: bool
    meta: VariantMeta | None
    corrupt: bool
    claim: ClaimRecord | None

    @property
    def claim_live(self) -> bool:
        return self.claim is not None and self.claim.alive

    @property
    def usable(self) -> bool:
        return self.meta is not None and self.has_checkpoint and not self.corrupt

    @property
    def size_bytes(self) -> int:
        if self.meta is not None:
            return self.meta.size_bytes
        return path_size(self.paths.checkpoint)

    def entry(self) -> VariantEntry:
        meta = self.meta
        return VariantEntry(
            key=self.key,
            port=self.port,
            ref=meta.ref if meta else None,
            size_bytes=self.size_bytes,
            populated=self.usable,
            corrupt=self.corrupt,
            created_at=meta.created_at if meta else None,
            last_used_at=meta.last_used_at if meta else None,
            claimed_by=self.claim.pid if self.claim else None,
            claim_live=self.claim_live,
            machine_id=meta.machine_id if meta else None,
            sdk_version=meta.sdk_version if meta else None,
            branch_ok=meta.branch_ok if meta else None,
        )


def _check_meta_matches(meta: VariantMeta, key: str, port: int, paths: VariantPaths) -> None:
    """Raise :class:`CacheCorrupt` unless ``meta`` describes the variant at ``paths``.

    A meta file copied or moved from another key or port would otherwise be
    restored as if it were this variant's: its ``key``, ``port`` and ``ref``
    (a ``file`` ref whose real path is this variant's checkpoint, whether or not
    that checkpoint exists yet) must all match.
    """
    source = str(paths.meta)
    if meta.key != key:
        raise CacheCorrupt(f"{source}: records key {meta.key}, not {key}", code="META_MISMATCH")
    if meta.port != port:
        raise CacheCorrupt(f"{source}: records port {meta.port}, not {port}", code="META_MISMATCH")
    if meta.ref.kind != "file":
        raise CacheCorrupt(
            f"{source}: records a {meta.ref.kind} checkpoint, not a file", code="META_MISMATCH"
        )
    if os.path.realpath(meta.ref.locator) != os.path.realpath(paths.checkpoint):
        raise CacheCorrupt(
            f"{source}: records checkpoint {meta.ref.locator}, not {paths.checkpoint}",
            code="META_MISMATCH",
        )


def _lru_order(variants: Iterable[_Scanned]) -> list[_Scanned]:
    """Least recently used first; ties broken by key then port for determinism."""
    return sorted(variants, key=lambda v: (v.meta.last_used_at if v.meta else 0.0, v.key, v.port))


class CheckpointCache:
    """The local checkpoint cache; see the module docstring for the layout."""

    kind: CheckpointKind = "file"

    def __init__(
        self,
        root: str | Path,
        *,
        max_bytes: int = 10 * 2**30,
        family: str = "postgres",
        lock_timeout_s: float | None = DEFAULT_TIMEOUT_S,
        poll_interval_s: float = DEFAULT_POLL_S,
    ) -> None:
        self.root = Path(root).expanduser()
        self.family = family
        self.max_bytes = max_bytes
        self.lock_timeout_s = lock_timeout_s
        self.poll_interval_s = poll_interval_s

    @classmethod
    def open(cls, settings: Settings, target: Target = "local") -> CheckpointCache:
        """Open (creating) the cache ``settings`` points at for the local target.

        The cloud target keeps checkpoints by id in a
        :class:`~smoltest.cache.cloud_index.CloudCheckpointIndex` instead; see
        :func:`smoltest.cache.open_checkpoint_backend` for the dispatch.
        """
        if target != "local":
            raise NotSupportedError(f"CheckpointCache serves the local target, not {target!r}")
        # A holder may spend ready_timeout_s in create, as much again in the
        # readiness wait, then checkpoint; boot/strategy.py still falls back to an
        # uncached cold boot when even this expires (the image pull is unbounded).
        cache = cls(
            settings.cache_dir,
            max_bytes=settings.cache_max_bytes,
            lock_timeout_s=2 * settings.ready_timeout_s + 120.0,
            poll_interval_s=settings.poll_interval_s,
        )
        cache.ensure_layout()
        return cache

    def __repr__(self) -> str:
        return f"CheckpointCache({str(self.family_dir)!r}, max_bytes={self.max_bytes})"

    # -- layout ----------------------------------------------------------------------

    @property
    def family_dir(self) -> Path:
        """``<cache_dir>/<family>``."""
        return self.root / self.family

    @property
    def store_dir(self) -> Path:
        """The engine's dedup store directory."""
        return self.family_dir / STORE_DIRNAME

    def ensure_layout(self) -> None:
        """Create the root and family directories with private permissions.

        The root's permissions are only set when smoltest creates it; a
        pre-existing, user-chosen directory keeps whatever mode it had.
        """
        ensure_dir(self.root, chmod_existing=False)
        ensure_dir(self.family_dir)

    def key_dir(self, key: str) -> Path:
        """Directory holding ``key``'s variants."""
        self._check_key(key)
        return self.family_dir / key

    def paths(self, key: str, port: int) -> VariantPaths:
        """The checkpoint, meta and claim paths of variant ``port`` under ``key``."""
        base = self.key_dir(key) / str(port)
        return VariantPaths(
            base.with_name(f"{port}{CHECKPOINT_SUFFIX}"),
            base.with_name(f"{port}{META_SUFFIX}"),
            base.with_name(f"{port}{CLAIM_SUFFIX}"),
        )

    def inputs_path(self, key: str) -> Path:
        """``<key>/inputs.json``."""
        return self.key_dir(key) / INPUTS_FILENAME

    def key_lock(self, key: str) -> FileLock:
        """The re-entrant per-key lock (``<key>.lock``); hold it across a cold boot."""
        self._check_key(key)
        self.ensure_layout()
        return FileLock.for_path(
            self.family_dir / f"{key}{LOCK_SUFFIX}",
            timeout_s=self.lock_timeout_s,
            poll_interval_s=self.poll_interval_s,
        )

    def store_lock(self) -> FileLock:
        """The store-wide lock (``.lock``), taken after any key lock."""
        self.ensure_layout()
        return FileLock.for_path(
            self.family_dir / STORE_LOCK_NAME,
            timeout_s=self.lock_timeout_s,
            poll_interval_s=self.poll_interval_s,
        )

    @staticmethod
    def _check_key(key: str) -> None:
        if not key or key in (STORE_DIRNAME, ".", "..") or key.startswith(".") or "/" in key:
            raise CacheError(f"invalid cache key {key!r}")

    # -- scanning --------------------------------------------------------------------

    def _key_names(self) -> list[str]:
        try:
            names = sorted(p.name for p in self.family_dir.iterdir() if p.is_dir())
        except OSError:
            return []
        return [n for n in names if n != STORE_DIRNAME and not n.startswith(".")]

    def _scan(self, key: str) -> list[_Scanned]:
        kdir = self.key_dir(key)
        ports: set[int] = set()
        try:
            names = [p.name for p in kdir.iterdir()]
        except OSError:
            return []
        for name in names:
            for suffix in (CHECKPOINT_SUFFIX, META_SUFFIX, CLAIM_SUFFIX):
                if name.endswith(suffix):
                    stem = name[: -len(suffix)]
                    if stem.isdigit():
                        ports.add(int(stem))
        return [self._scan_one(key, port) for port in sorted(ports)]

    def _scan_one(self, key: str, port: int) -> _Scanned:
        paths = self.paths(key, port)
        has_checkpoint = paths.checkpoint.exists() or paths.checkpoint.is_symlink()
        meta: VariantMeta | None = None
        corrupt = False
        if paths.meta.exists():
            try:
                meta = VariantMeta.load(paths.meta)
                _check_meta_matches(meta, key, port, paths)
            except CacheCorrupt as exc:
                logger.warning("corrupt cache entry %s: %s", paths.meta, exc)
                meta = None
                corrupt = True
        return _Scanned(key, port, paths, has_checkpoint, meta, corrupt, read_claim(paths.claim))

    def _all_variants(self) -> list[_Scanned]:
        out: list[_Scanned] = []
        for key in self._key_names():
            out.extend(self._scan(key))
        return out

    def _read_inputs(self, key: str) -> dict[str, Any] | None:
        try:
            data = json.loads(self.inputs_path(key).read_text("utf-8"))
        except (OSError, ValueError):
            return None
        inputs = data.get("inputs") if isinstance(data, dict) else None
        return dict(inputs) if isinstance(inputs, dict) else None

    def _remove_variant(self, paths: VariantPaths) -> None:
        # Meta first so a crash half-way leaves garbage, never a dangling marker.
        remove_path(paths.meta)
        remove_path(paths.checkpoint)
        remove_path(paths.claim)

    def _remove_key_dir_if_empty(self, key: str) -> bool:
        kdir = self.key_dir(key)
        if not kdir.is_dir() or self._scan(key):
            return False
        remove_path(kdir)
        return True

    def _write_claim(self, key: str, port: int, meta: VariantMeta | None) -> VariantClaim:
        paths = self.paths(key, port)
        token = f"{os.getpid()}-{uuid.uuid4().hex}"
        record = {
            "pid": os.getpid(),
            "token": token,
            "created_at": time.time(),
            "key": key,
            "port": port,
        }
        write_private(paths.claim, _json_bytes(record))
        ref = meta.ref if meta is not None else CheckpointRef("file", str(paths.checkpoint))
        return VariantClaim(key, port, ref, meta, "file", paths.claim, token)

    # -- the boot-facing API ---------------------------------------------------------

    def claim_variant(self, key: str, pinned_port: int | None = None) -> VariantClaim | None:
        """Claim a restorable variant of ``key`` whose port is free, or return ``None``.

        Under the key lock: variants without a commit marker and without a live
        claim are deleted as garbage, corrupt ones are invalidated, variants
        claimed by a live process or whose port is busy are skipped, and the
        most recently used of the rest is claimed (``pinned_port`` restricts the
        choice to that port).
        """
        self._check_key(key)
        with self.key_lock(key), self.store_lock().shared_lock():
            if not self.key_dir(key).is_dir():
                return None
            candidates: list[_Scanned] = []
            for variant in self._scan(key):
                if variant.claim_live:
                    continue
                if variant.corrupt or not variant.usable:
                    logger.info("removing unusable cache variant %s:%s", key, variant.port)
                    self._remove_variant(variant.paths)
                    continue
                if variant.claim is not None:
                    remove_path(variant.paths.claim)
                if pinned_port is not None and variant.port != pinned_port:
                    continue
                if not is_port_free(variant.port):
                    continue
                candidates.append(variant)
            if not candidates:
                return None
            best = max(
                candidates,
                key=lambda v: (v.meta.last_used_at if v.meta else 0.0, -v.port),
            )
            return self._write_claim(key, best.port, best.meta)

    def reserve_new_variant(self, key: str, port: int | None) -> VariantClaim:
        """Claim ``port`` under ``key`` for a checkpoint :meth:`populate` will write."""
        self._check_key(key)
        if port is None:
            raise CacheError("a local variant needs a host port")
        with self.key_lock(key), self.store_lock().shared_lock():
            ensure_dir(self.key_dir(key))
            existing = self._scan_one(key, port)
            if existing.claim_live and existing.claim is not None:
                raise CacheError(
                    f"variant {key}:{port} is claimed by live pid {existing.claim.pid}",
                    code="CLAIMED",
                )
            if existing.usable:
                raise CacheError(
                    f"variant {key}:{port} is already populated; claim it instead",
                    code="EXISTS",
                )
            self._remove_variant(existing.paths)
            return self._write_claim(key, port, None)

    def populate(
        self,
        key: str,
        claim: VariantClaim,
        handle: MachineHandle,
        inputs: Mapping[str, Any] | None = None,
        *,
        engine: Engine | None = None,
    ) -> VariantMeta:
        """Checkpoint ``handle`` into the variant ``claim`` reserved and commit it.

        Under the key and store locks: evict least recently used variants over
        ``max_bytes`` (never the current claim or a live-claimed variant), remove
        a leftover output, take the checkpoint into the dedup store (the engine
        must write it at the variant's own path, which the meta records), write
        ``inputs.json`` (redacted) if missing, write the meta marker last, then
        evict again now that the new size is known. ``engine`` lets the store be
        pruned after eviction and stamps the SDK version into the meta.
        """
        self._check_key(key)
        if claim.key != key:
            raise CacheError(f"claim is for key {claim.key}, not {key}")
        if claim.port is None:
            raise CacheError("a local variant needs a host port")
        if claim.released:
            raise CacheError(f"claim {key}:{claim.port} was already released")
        paths = self.paths(key, claim.port)
        protect = {(key, claim.port)}
        with self.key_lock(key), self.store_lock():
            ensure_dir(self.key_dir(key))
            ensure_dir(self.store_dir)
            self._evict_lru(self.max_bytes, protect, engine)
            remove_path(paths.meta)
            remove_path(paths.checkpoint)
            info = self._take_checkpoint(handle, paths)
            # Record the variant's own path, not the engine's spelling of it, so the
            # meta matches what _check_meta_matches expects whatever the SDK echoes.
            ref = CheckpointRef("file", str(paths.checkpoint))
            size = path_size(paths.checkpoint) or (info.size_bytes or 0)
            if inputs is not None and not self.inputs_path(key).exists():
                self._write_inputs(key, inputs)
            sdk_version = engine.sdk_version() if engine is not None else None
            if sdk_version is None and inputs is not None and inputs.get("sdk_version") is not None:
                sdk_version = str(inputs["sdk_version"])
            now = time.time()
            meta = VariantMeta(
                key=key,
                port=claim.port,
                ref=ref,
                created_at=now,
                last_used_at=now,
                size_bytes=size,
                machine_id=info.machine_id or handle.id,
                sdk_version=sdk_version,
            )
            write_private(paths.meta, meta.to_json())  # the commit marker, last
            self._evict_lru(self.max_bytes, protect, engine)
        claim.meta = meta
        claim.ref = ref
        return meta

    def _take_checkpoint(self, handle: MachineHandle, paths: VariantPaths) -> CheckpointInfo:
        """``handle.checkpoint`` into ``paths.checkpoint``, validated and made private.

        Whatever the engine wrote is a copy of guest RAM with the umask's
        permissions, so any failure after the call (an engine error, a reported
        ref that is not this file, a ``chmod`` that does not take) removes the
        requested output, and the file the engine reported if that differs,
        before the error propagates.
        """
        reported: Path | None = None
        try:
            info = handle.checkpoint(str(paths.checkpoint), str(self.store_dir))
            ref = info.ref
            if ref.kind != "file":
                raise CacheCorrupt(f"engine returned a {ref.kind} checkpoint for a local cache")
            reported = Path(ref.locator)
            if not (reported.exists() or reported.is_symlink()):
                raise CacheCorrupt(f"engine reported checkpoint {reported} but nothing is there")
            if os.path.realpath(reported) != os.path.realpath(paths.checkpoint):
                raise CacheCorrupt(f"engine wrote checkpoint {reported}, not {paths.checkpoint}")
            make_private(paths.checkpoint)  # the engine writes with the default umask
        except SmoltestError:
            self._discard_checkpoint(paths, reported)
            raise
        except Exception as exc:
            self._discard_checkpoint(paths, reported)
            raise CacheError(f"checkpoint of {handle.name} failed: {exc}") from exc
        except BaseException:  # KeyboardInterrupt / SystemExit: tidy, then propagate
            self._discard_checkpoint(paths, reported)
            raise
        return info

    @staticmethod
    def _discard_checkpoint(paths: VariantPaths, reported: Path | None) -> None:
        remove_path_strict(paths.checkpoint)
        if reported is not None and os.path.realpath(reported) != os.path.realpath(
            paths.checkpoint
        ):
            remove_path_strict(reported)

    def _write_inputs(self, key: str, inputs: Mapping[str, Any]) -> None:
        payload = {
            "format": INPUTS_FORMAT,
            "key": key,
            "created_at": time.time(),
            "inputs": redact_inputs(inputs),
        }
        write_private(self.inputs_path(key), _json_bytes(payload))

    def invalidate(self, claim: VariantClaim) -> None:
        """Delete the variant behind ``claim`` (checkpoint, meta, claim) and release it."""
        self._check_key(claim.key)
        with self.key_lock(claim.key):
            if claim.port is not None:
                self._remove_variant(self.paths(claim.key, claim.port))
            claim.release()
            self._remove_key_dir_if_empty(claim.key)

    def touch(self, claim: VariantClaim) -> None:
        """Record that the variant behind ``claim`` was just used (LRU bookkeeping)."""
        self._rewrite_meta(claim, last_used_at=time.time())

    def record_branch_ok(self, claim: VariantClaim, ok: bool) -> None:
        """Remember whether a machine restored from this variant could be branched."""
        self._rewrite_meta(claim, branch_ok=ok)

    def _rewrite_meta(self, claim: VariantClaim, **changes: Any) -> None:
        if claim.meta is None or claim.port is None:
            return
        paths = self.paths(claim.key, claim.port)
        with self.key_lock(claim.key):
            if not paths.meta.exists():
                return
            meta = replace(claim.meta, **changes)
            write_private(paths.meta, meta.to_json())
        claim.meta = meta

    # -- housekeeping ----------------------------------------------------------------

    def _evict_lru(
        self,
        budget: int,
        protect: set[tuple[str, int]],
        engine: Engine | None,
    ) -> list[VariantEntry]:
        removed: list[VariantEntry] = []
        while True:
            variants = self._all_variants()
            if sum(v.size_bytes for v in variants) + path_size(self.store_dir) <= budget:
                break
            candidates = _lru_order(
                v
                for v in variants
                if v.usable and (v.key, v.port) not in protect and not v.claim_live
            )
            if not candidates:
                break
            victim = candidates[0]
            logger.info("evicting cache variant %s:%s (LRU)", victim.key, victim.port)
            removed.append(victim.entry())
            self._remove_variant(victim.paths)
            self._remove_key_dir_if_empty(victim.key)
            if engine is not None and self.store_dir.is_dir():
                # Re-measure after the store drops the chunks only the victim used.
                engine.prune_checkpoint_store(str(self.store_dir))
        return removed

    def size_bytes(self) -> int:
        """Bytes used by every variant plus the dedup store."""
        return sum(v.size_bytes for v in self._all_variants()) + path_size(self.store_dir)

    def variants(self, key: str) -> tuple[VariantEntry, ...]:
        """Every variant of ``key`` as found on disk, populated or not."""
        return tuple(v.entry() for v in self._scan(key))

    def keys(self) -> tuple[str, ...]:
        """Keys with a directory in the cache."""
        return tuple(self._key_names())

    def find_key(self, prefix: str) -> str:
        """Resolve a unique key prefix (as typed on the CLI) to the full key."""
        matches = [k for k in self._key_names() if k.startswith(prefix)]
        if len(matches) == 1:
            return matches[0]
        if not matches:
            raise CacheError(f"no cache key starts with {prefix!r}", code="NO_KEY")
        raise CacheError(f"ambiguous key prefix {prefix!r}: {', '.join(matches)}", code="AMBIGUOUS")

    def ls(self) -> CacheSummary:
        """Describe every key and variant without taking any lock."""
        keys = tuple(
            KeyEntry(key, self._read_inputs(key), tuple(v.entry() for v in self._scan(key)))
            for key in self._key_names()
        )
        return CacheSummary(
            kind="file",
            root=self.family_dir,
            keys=keys,
            store_bytes=path_size(self.store_dir),
            max_bytes=self.max_bytes,
        )

    def prune(
        self,
        older_than_s: float | None = None,
        keep_latest: int | None = None,
        max_bytes: int | None = None,
        stale: bool = False,
        engine: Engine | None = None,
    ) -> PruneReport:
        """Remove variants by age, count per key, size budget and/or staleness.

        ``stale`` removes variants that can never restore here: no commit marker,
        corrupt or missing checkpoint, a host signature in ``inputs.json`` that
        differs from this host's, or (with ``engine``) a different SDK version.
        Variants claimed by a live process are never removed; claim files of dead
        processes always are. With ``engine`` the dedup store is pruned afterwards.
        """
        with self.store_lock():
            variants = self._all_variants()
            claims_removed = self._drop_dead_claims(variants)
            marked: dict[tuple[str, int], _Scanned] = {}
            self._mark_stale_and_old(variants, marked, older_than_s, stale, engine)
            if keep_latest is not None:
                self._mark_beyond_latest(variants, marked, keep_latest)
            if max_bytes is not None:
                self._mark_over_budget(variants, marked, max_bytes)
            removed = tuple(v.entry() for v in marked.values())
            for v in marked.values():
                self._remove_variant(v.paths)
            touched_keys = sorted({v.key for v in marked.values()})
            removed_keys = tuple(k for k in touched_keys if self._remove_key_dir_if_empty(k))
            pruned = 0
            if engine is not None and self.store_dir.is_dir():
                pruned = engine.prune_checkpoint_store(str(self.store_dir))
        return PruneReport(
            removed=removed,
            freed_bytes=sum(v.size_bytes for v in removed),
            removed_keys=removed_keys,
            store_chunks_pruned=pruned,
            claims_removed=claims_removed,
        )

    @staticmethod
    def _drop_dead_claims(variants: list[_Scanned]) -> int:
        dropped = 0
        for v in variants:
            if v.claim is not None and not v.claim_live:
                remove_path(v.paths.claim)
                v.claim = None
                dropped += 1
        return dropped

    def _mark_stale_and_old(
        self,
        variants: list[_Scanned],
        marked: dict[tuple[str, int], _Scanned],
        older_than_s: float | None,
        stale: bool,
        engine: Engine | None,
    ) -> None:
        now = time.time()
        host = host_signature().as_dict() if stale else None
        sdk = engine.sdk_version() if engine is not None else None
        for v in variants:
            if v.claim_live:
                continue
            too_old = (
                older_than_s is not None
                and v.meta is not None
                and now - v.meta.last_used_at > older_than_s
            )
            if too_old or (stale and self._is_stale(v, host, sdk)):
                marked[(v.key, v.port)] = v

    @staticmethod
    def _mark_beyond_latest(
        variants: list[_Scanned],
        marked: dict[tuple[str, int], _Scanned],
        keep_latest: int,
    ) -> None:
        by_key: dict[str, list[_Scanned]] = {}
        for v in variants:
            if v.usable and not v.claim_live and (v.key, v.port) not in marked:
                by_key.setdefault(v.key, []).append(v)
        for group in by_key.values():
            newest_first = list(reversed(_lru_order(group)))
            for v in newest_first[max(keep_latest, 0) :]:
                marked[(v.key, v.port)] = v

    def _mark_over_budget(
        self,
        variants: list[_Scanned],
        marked: dict[tuple[str, int], _Scanned],
        max_bytes: int,
    ) -> None:
        remaining = [v for v in variants if (v.key, v.port) not in marked]
        total = sum(v.size_bytes for v in remaining) + path_size(self.store_dir)
        for v in _lru_order(v for v in remaining if v.usable and not v.claim_live):
            if total <= max_bytes:
                break
            marked[(v.key, v.port)] = v
            total -= v.size_bytes

    def _is_stale(self, v: _Scanned, host: Mapping[str, str] | None, sdk: str | None) -> bool:
        if not v.usable or v.meta is None:
            return True
        if sdk is not None and v.meta.sdk_version is not None and v.meta.sdk_version != sdk:
            return True
        if host is not None:
            inputs = self._read_inputs(v.key)
            recorded = inputs.get("host") if inputs else None
            if isinstance(recorded, dict) and recorded != host:
                return True
        return False

    def clear(self) -> PruneReport:
        """Remove every key, variant and the dedup store (lock files stay)."""
        with self.store_lock():
            variants = self._all_variants()
            removed = tuple(v.entry() for v in variants)
            claims = sum(1 for v in variants if v.claim is not None)
            keys = tuple(self._key_names())
            for key in keys:
                remove_path(self.key_dir(key))
            remove_path(self.store_dir)
        return PruneReport(
            removed=removed,
            freed_bytes=sum(v.size_bytes for v in removed),
            removed_keys=keys,
            claims_removed=claims,
        )

    def export(self, key: str, port: int | None, output: str | Path, engine: Engine) -> int:
        """Copy variant ``port`` of ``key`` (default: most recently used) to ``output``.

        Returns the bytes written. Checkpoints contain guest RAM, so a warning is
        emitted once per process.
        """
        self._check_key(key)
        out = Path(output).expanduser()
        if out.exists():
            raise CacheError(f"export target already exists: {out}", code="EXISTS")
        with self.key_lock(key), self.store_lock().shared_lock():
            usable = [v for v in self._scan(key) if v.usable and (port is None or v.port == port)]
            if not usable:
                where = f"{key}:{port}" if port is not None else key
                raise CacheError(f"no populated variant for {where}", code="NO_VARIANT")
            chosen = max(usable, key=lambda v: v.meta.last_used_at if v.meta else 0.0)
            warn_once(
                "cache.export",
                "exported checkpoints contain a copy of guest RAM, including any "
                "credentials and data the guest held; keep them private",
            )
            written = engine.export_checkpoint(str(chosen.paths.checkpoint), str(out))
            _private_or_removed(out, out)  # the engine writes with the default umask
            return written


__all__ = [
    "CHECKPOINT_SUFFIX",
    "CLAIM_SUFFIX",
    "DIR_MODE",
    "FILE_MODE",
    "INPUTS_FILENAME",
    "META_FORMAT",
    "META_SUFFIX",
    "STORE_DIRNAME",
    "CacheSummary",
    "CheckpointBackend",
    "CheckpointCache",
    "ClaimRecord",
    "KeyEntry",
    "PruneReport",
    "VariantClaim",
    "VariantEntry",
    "VariantMeta",
    "VariantPaths",
    "ensure_dir",
    "make_private",
    "path_size",
    "pid_alive",
    "read_claim",
    "remove_path",
    "remove_path_strict",
    "write_private",
]
