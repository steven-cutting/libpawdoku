"""Key → cloud checkpoint id, kept in ``<cache_dir>/cloud/<sha(base_url)>/index.json``.

Cloud checkpoints live on Smol's side and restore on fresh ports, so there are
no variants, claims or sizes to manage; the index only remembers which id a
key produced. It implements the same :class:`~smoltest.cache.store.CheckpointBackend`
interface as the local :class:`~smoltest.cache.store.CheckpointCache`, so the
boot ladder treats both alike.
"""

from __future__ import annotations

import json
import time
from collections.abc import Mapping
from dataclasses import replace
from pathlib import Path
from typing import Any

from .._log import logger
from ..config import Settings
from ..errors import CacheCorrupt, CacheError, NotSupportedError
from ..transport.base import CheckpointKind, CheckpointRef, Engine, MachineHandle
from .key import cloud_base_url, digest, redact_inputs
from .lock import DEFAULT_POLL_S, DEFAULT_TIMEOUT_S, FileLock
from .store import (
    CacheSummary,
    KeyEntry,
    PruneReport,
    VariantClaim,
    VariantEntry,
    VariantMeta,
    ensure_dir,
    remove_path,
    write_private,
)

INDEX_FORMAT = 1
INDEX_FILENAME = "index.json"
INDEX_LOCK_NAME = "index.lock"
CLOUD_DIRNAME = "cloud"


class CloudCheckpointIndex:
    """A small JSON map from cache key to cloud checkpoint id."""

    kind: CheckpointKind = "cloud"

    def __init__(
        self,
        path: str | Path,
        *,
        base_url: str,
        lock_timeout_s: float | None = DEFAULT_TIMEOUT_S,
        poll_interval_s: float = DEFAULT_POLL_S,
    ) -> None:
        self.path = Path(path).expanduser()
        self.base_url = base_url
        self.lock_timeout_s = lock_timeout_s
        self.poll_interval_s = poll_interval_s

    @classmethod
    def open(
        cls,
        settings: Settings,
        base_url: str | None = None,
        env: Mapping[str, str] | None = None,
    ) -> CloudCheckpointIndex:
        """Open (creating) the index for ``base_url`` (default: ``$SMOL_CLOUD_URL``)."""
        url = base_url if base_url is not None else cloud_base_url(env)
        path = settings.cache_dir / CLOUD_DIRNAME / digest(url, 16) / INDEX_FILENAME
        index = cls(
            path,
            base_url=url,
            lock_timeout_s=settings.ready_timeout_s + 60.0,
            poll_interval_s=settings.poll_interval_s,
        )
        ensure_dir(index.dir)
        return index

    def __repr__(self) -> str:
        return f"CloudCheckpointIndex({str(self.path)!r}, base_url={self.base_url!r})"

    @property
    def dir(self) -> Path:
        """The directory holding the index and its lock files."""
        return self.path.parent

    # -- locking and persistence -----------------------------------------------------

    def key_lock(self, key: str) -> FileLock:
        """Per-key lock, so concurrent processes do not both cold-boot the same key."""
        if not key or "/" in key or key.startswith("."):
            raise CacheError(f"invalid cache key {key!r}")
        ensure_dir(self.dir)
        return FileLock.for_path(
            self.dir / f"{key}.lock",
            timeout_s=self.lock_timeout_s,
            poll_interval_s=self.poll_interval_s,
        )

    def _lock(self) -> FileLock:
        ensure_dir(self.dir)
        return FileLock.for_path(
            self.dir / INDEX_LOCK_NAME,
            timeout_s=self.lock_timeout_s,
            poll_interval_s=self.poll_interval_s,
        )

    def _load(self) -> dict[str, dict[str, Any]]:
        try:
            raw = self.path.read_text("utf-8")
        except FileNotFoundError:
            return {}
        except OSError as exc:
            raise CacheCorrupt(f"{self.path}: unreadable: {exc}", code="INDEX_UNREADABLE") from exc
        try:
            data = json.loads(raw)
        except ValueError as exc:
            raise CacheCorrupt(f"{self.path}: not JSON: {exc}", code="INDEX_JSON") from exc
        if not isinstance(data, dict) or data.get("format") != INDEX_FORMAT:
            raise CacheCorrupt(f"{self.path}: unsupported index format", code="INDEX_FORMAT")
        entries = data.get("entries")
        if not isinstance(entries, dict):
            raise CacheCorrupt(f"{self.path}: entries is not an object", code="INDEX_SHAPE")
        return {str(k): v for k, v in entries.items() if isinstance(v, dict)}

    def _save(self, entries: Mapping[str, Mapping[str, Any]]) -> None:
        payload = {
            "format": INDEX_FORMAT,
            "base_url": self.base_url,
            "updated_at": time.time(),
            "entries": {k: dict(v) for k, v in entries.items()},
        }
        write_private(self.path, (json.dumps(payload, sort_keys=True, indent=1) + "\n").encode())

    def _quarantine(self, exc: CacheCorrupt) -> None:
        logger.warning("resetting corrupt cloud index %s: %s", self.path, exc)
        aside = self.path.with_name(f"{self.path.name}.corrupt")
        remove_path(aside)
        try:
            self.path.rename(aside)
        except OSError:
            remove_path(self.path)

    @staticmethod
    def _meta(key: str, entry: Mapping[str, Any]) -> VariantMeta:
        """The meta an index entry records, or :class:`CacheCorrupt` if it is not ``key``'s.

        A record copied or swapped under another key, carrying a port or a non-cloud
        ref, would otherwise restore a different database state under this key.
        """
        source = f"index entry {key}"
        meta = VariantMeta.from_dict(dict(entry), source=source)
        if meta.key != key:
            raise CacheCorrupt(f"{source}: records key {meta.key}", code="INDEX_MISMATCH")
        if meta.port is not None:
            raise CacheCorrupt(f"{source}: records port {meta.port}", code="INDEX_MISMATCH")
        if meta.ref.kind != "cloud":
            raise CacheCorrupt(
                f"{source}: records a {meta.ref.kind} checkpoint", code="INDEX_MISMATCH"
            )
        return meta

    @classmethod
    def _meta_or_none(cls, key: str, entry: Mapping[str, Any]) -> VariantMeta | None:
        try:
            return cls._meta(key, entry)
        except CacheCorrupt as exc:
            logger.warning("skipping corrupt cloud index entry %s: %s", key, exc)
            return None

    @staticmethod
    def _entry_locator(entry: Mapping[str, Any]) -> str | None:
        """The checkpoint id an index entry records, or ``None`` when it has none."""
        ref = entry.get("ref")
        locator = ref.get("locator") if isinstance(ref, Mapping) else None
        return None if locator is None else str(locator)

    @staticmethod
    def _claim_locator(claim: VariantClaim) -> str | None:
        """The checkpoint id ``claim`` was taken on, or ``None`` for an unpopulated claim."""
        ref = claim.ref if claim.ref is not None else (claim.meta.ref if claim.meta else None)
        return None if ref is None else ref.locator

    @staticmethod
    def _entry(key: str, meta: VariantMeta | None) -> VariantEntry:
        if meta is None:
            return VariantEntry(key, None, None, 0, populated=False, corrupt=True)
        return VariantEntry(
            key=key,
            port=None,
            ref=meta.ref,
            size_bytes=meta.size_bytes,
            populated=True,
            created_at=meta.created_at,
            last_used_at=meta.last_used_at,
            machine_id=meta.machine_id,
            sdk_version=meta.sdk_version,
            branch_ok=meta.branch_ok,
        )

    # -- the plain map ---------------------------------------------------------------

    def get(self, key: str) -> str | None:
        """The checkpoint id recorded for ``key``, if any (raises on a corrupt index)."""
        entry = self._load().get(key)
        return None if entry is None else str(entry["ref"]["locator"])

    def set(
        self,
        key: str,
        checkpoint_id: str,
        *,
        inputs: Mapping[str, Any] | None = None,
        sdk_version: str | None = None,
        machine_id: str | None = None,
        size_bytes: int | None = None,
    ) -> VariantMeta:
        """Record ``checkpoint_id`` for ``key`` and return its meta."""
        now = time.time()
        meta = VariantMeta(
            key=key,
            port=None,
            ref=CheckpointRef("cloud", checkpoint_id),
            created_at=now,
            last_used_at=now,
            size_bytes=size_bytes or 0,
            machine_id=machine_id,
            sdk_version=sdk_version,
        )
        with self._lock():
            try:
                entries = self._load()
            except CacheCorrupt as exc:
                self._quarantine(exc)
                entries = {}
            record = meta.to_dict()
            if inputs is not None:
                record["inputs"] = redact_inputs(inputs)
            else:
                previous = entries.get(key, {})
                if "inputs" in previous:
                    record["inputs"] = previous["inputs"]
            entries[key] = record
            self._save(entries)
        return meta

    def drop(self, key: str) -> bool:
        """Forget ``key``; ``True`` when it was recorded."""
        with self._lock():
            try:
                entries = self._load()
            except CacheCorrupt as exc:
                self._quarantine(exc)
                return False
            if key not in entries:
                return False
            del entries[key]
            self._save(entries)
            return True

    def entries(self) -> dict[str, VariantMeta]:
        """Every recorded key with its meta."""
        metas = ((key, self._meta_or_none(key, entry)) for key, entry in self._load().items())
        return {key: meta for key, meta in metas if meta is not None}

    # -- the CheckpointBackend interface -----------------------------------------------

    def claim_variant(self, key: str, pinned_port: int | None = None) -> VariantClaim | None:
        """A claim on the recorded checkpoint of ``key``, or ``None`` (ports are irrelevant)."""
        with self.key_lock(key):
            try:
                entry = self._load().get(key)
            except CacheCorrupt as exc:
                with self._lock():
                    self._quarantine(exc)
                return None
            if entry is None:
                return None
            try:
                meta = self._meta(key, entry)
            except CacheCorrupt as exc:
                # One bad record is not a bad index: drop it, keep the rest.
                logger.warning("dropping corrupt cloud index entry %s: %s", key, exc)
                self.drop(key)
                return None
        return VariantClaim(key, None, meta.ref, meta, "cloud")

    def reserve_new_variant(self, key: str, port: int | None = None) -> VariantClaim:
        """An empty claim :meth:`populate` fills in; nothing is written yet."""
        return VariantClaim(key, None, None, None, "cloud")

    def populate(
        self,
        key: str,
        claim: VariantClaim,
        handle: MachineHandle,
        inputs: Mapping[str, Any] | None = None,
        *,
        engine: Engine | None = None,
    ) -> VariantMeta:
        """Checkpoint ``handle`` on the cloud and record the id under ``key``."""
        if claim.key != key:
            raise CacheError(f"claim is for key {claim.key}, not {key}")
        with self.key_lock(key):
            info = handle.checkpoint(None, None)
            if info.ref.kind != "cloud":
                raise CacheCorrupt(f"engine returned a {info.ref.kind} checkpoint for the cloud")
            sdk_version = engine.sdk_version() if engine is not None else None
            if sdk_version is None and inputs is not None and inputs.get("sdk_version") is not None:
                sdk_version = str(inputs["sdk_version"])
            meta = self.set(
                key,
                info.ref.locator,
                inputs=inputs,
                sdk_version=sdk_version,
                machine_id=info.machine_id or handle.id,
                size_bytes=info.size_bytes,
            )
        claim.meta = meta
        claim.ref = meta.ref
        return meta

    def invalidate(self, claim: VariantClaim) -> None:
        """Forget the key behind ``claim`` (the cloud checkpoint itself is left alone).

        Compare-and-set: the entry is dropped only while it still records the
        checkpoint id ``claim`` was taken on. A replacement another process
        recorded in the meantime is left in place.
        """
        locator = self._claim_locator(claim)
        if locator is not None:
            with self._lock():
                try:
                    entries = self._load()
                except CacheCorrupt as exc:
                    self._quarantine(exc)
                else:
                    entry = entries.get(claim.key)
                    if entry is not None and self._entry_locator(entry) == locator:
                        del entries[claim.key]
                        self._save(entries)
                    else:
                        logger.debug(
                            "cloud index entry %s no longer records %s; not dropping it",
                            claim.key,
                            locator,
                        )
        claim.release()

    def touch(self, claim: VariantClaim) -> None:
        """Bump ``last_used_at`` for the key behind ``claim``."""
        self._rewrite(claim, last_used_at=time.time())

    def record_branch_ok(self, claim: VariantClaim, ok: bool) -> None:
        """Remember whether the restored machine could be branched."""
        self._rewrite(claim, branch_ok=ok)

    def _rewrite(self, claim: VariantClaim, **changes: Any) -> None:
        """Merge ``changes`` into the *current* entry of ``claim.key``, compare-and-set.

        The entry is rewritten only while it still records the checkpoint id
        ``claim`` was taken on, and from the entry as it is now, never from the
        claim's possibly stale copy; otherwise nothing is written.
        """
        locator = self._claim_locator(claim)
        if claim.meta is None or locator is None:
            return
        with self._lock():
            try:
                entries = self._load()
            except CacheCorrupt as exc:
                self._quarantine(exc)
                return
            entry = entries.get(claim.key)
            current = None if entry is None else self._meta_or_none(claim.key, entry)
            if entry is None or current is None or current.ref.locator != locator:
                logger.debug(
                    "cloud index entry %s no longer records %s; not rewriting it",
                    claim.key,
                    locator,
                )
                return
            meta = replace(current, **changes)
            entries[claim.key] = {**entry, **meta.to_dict()}
            self._save(entries)
        claim.meta = meta

    def ls(self) -> CacheSummary:
        """Every recorded key as a one-variant entry."""
        try:
            raw = self._load()
        except CacheCorrupt as exc:
            logger.warning("cloud index %s is corrupt: %s", self.path, exc)
            raw = {}
        keys: list[KeyEntry] = []
        for key in sorted(raw):
            entry = raw[key]
            inputs = entry.get("inputs")
            variant = self._entry(key, self._meta_or_none(key, entry))
            keys.append(KeyEntry(key, inputs if isinstance(inputs, dict) else None, (variant,)))
        return CacheSummary(kind="cloud", root=self.dir, keys=tuple(keys))

    def prune(
        self,
        older_than_s: float | None = None,
        keep_latest: int | None = None,
        max_bytes: int | None = None,
        stale: bool = False,
        engine: Engine | None = None,
    ) -> PruneReport:
        """Drop entries by age or staleness (corrupt, or another SDK version with ``engine``).

        ``keep_latest`` and ``max_bytes`` do not apply: a key holds one id and
        the cloud owns the bytes.
        """
        with self._lock():
            try:
                entries = self._load()
            except CacheCorrupt as exc:
                self._quarantine(exc)
                return PruneReport()
            now = time.time()
            sdk = engine.sdk_version() if engine is not None else None
            removed: list[VariantEntry] = []
            for key in sorted(entries):
                meta = self._meta_or_none(key, entries[key])
                if meta is None:
                    if stale:
                        removed.append(self._entry(key, None))
                    continue
                too_old = older_than_s is not None and now - meta.last_used_at > older_than_s
                other_sdk = sdk is not None and meta.sdk_version not in (None, sdk)
                if too_old or (stale and other_sdk):
                    removed.append(self._entry(key, meta))
            if removed:
                for variant in removed:
                    entries.pop(variant.key, None)
                self._save(entries)
        return PruneReport(
            removed=tuple(removed),
            freed_bytes=sum(v.size_bytes for v in removed),
            removed_keys=tuple(v.key for v in removed),
        )

    def clear(self) -> PruneReport:
        """Forget every key."""
        summary = self.ls()
        removed = tuple(v for k in summary.keys for v in k.variants)
        with self._lock():
            remove_path(self.path)
        return PruneReport(
            removed=removed,
            freed_bytes=sum(v.size_bytes for v in removed),
            removed_keys=tuple(k.key for k in summary.keys),
        )

    def export(self, key: str, port: int | None, output: str | Path, engine: Engine) -> int:
        """Cloud checkpoints cannot be exported to a file."""
        raise NotSupportedError("cloud checkpoints live on Smol's side and cannot be exported")


__all__ = ["CLOUD_DIRNAME", "INDEX_FILENAME", "INDEX_FORMAT", "CloudCheckpointIndex"]
