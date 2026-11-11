"""Checkpoint caching: keys, locks, the local store and the cloud index.

Both backends implement :class:`CheckpointBackend`; :func:`open_checkpoint_backend`
picks the one for a resolved target.
"""

from __future__ import annotations

from collections.abc import Mapping

from ..config import Settings
from ..transport.base import Target
from .cloud_index import CloudCheckpointIndex
from .key import CacheKey, HostSignature, cloud_base_url, host_signature, redact_inputs
from .lock import FileLock, LockTimeout
from .store import (
    CacheSummary,
    CheckpointBackend,
    CheckpointCache,
    KeyEntry,
    PruneReport,
    VariantClaim,
    VariantEntry,
    VariantMeta,
)


def open_checkpoint_backend(
    settings: Settings,
    target: Target,
    env: Mapping[str, str] | None = None,
) -> CheckpointBackend:
    """A :class:`CheckpointCache` for ``local``, a :class:`CloudCheckpointIndex` for ``cloud``."""
    if target == "cloud":
        return CloudCheckpointIndex.open(settings, env=env)
    return CheckpointCache.open(settings, target)


__all__ = [
    "CacheKey",
    "CacheSummary",
    "CheckpointBackend",
    "CheckpointCache",
    "CloudCheckpointIndex",
    "FileLock",
    "HostSignature",
    "KeyEntry",
    "LockTimeout",
    "PruneReport",
    "VariantClaim",
    "VariantEntry",
    "VariantMeta",
    "cloud_base_url",
    "host_signature",
    "open_checkpoint_backend",
    "redact_inputs",
]
