"""Cache keys: canonical JSON of everything that shapes a checkpoint, hashed.

A key identifies a machine *shape*, never the PostgreSQL host port: that port is
a per-variant attribute of the store. Pinned host ports of *extra* guest ports
are part of the shape, because a restored machine binds them exactly as the
checkpointed one did. Local keys carry a host signature because a checkpoint
restores only on the OS, architecture and CPU feature set that produced it;
cloud keys carry the API base URL instead.
"""

from __future__ import annotations

import functools
import hashlib
import json
import os
import platform
import re
from collections.abc import Mapping
from dataclasses import dataclass
from pathlib import Path
from typing import Any

from .._version import __version__
from ..boot.spec import PostgresSpec, build_machine_spec
from ..config import Settings
from ..transport.base import Engine, MachineSpec, Target

KEY_FORMAT = 2
"""Bump when the key inputs change shape; old entries then simply miss.

Format 2 added ``pinned_ports`` (the pinned host ports of extra guest ports).
"""

KEY_LENGTH = 32
"""Hex characters kept from the SHA-256 digest."""

DEFAULT_CLOUD_URL = "https://api.smolmachines.com"
CLOUD_URL_ENV = "SMOL_CLOUD_URL"
CPUINFO_PATH = Path("/proc/cpuinfo")

_SENSITIVE = re.compile(
    r"PASSWORD|PASSWD|PASSPHRASE|SECRET|TOKEN|CREDENTIAL|API_?KEY|PRIVATE_?KEY",
    re.IGNORECASE,
)
PLAIN_ENV_NAMES = frozenset(
    {
        "LANG",
        "LANGUAGE",
        "LC_ALL",
        "LC_COLLATE",
        "LC_CTYPE",
        "LC_MESSAGES",
        "LC_MONETARY",
        "LC_NUMERIC",
        "LC_TIME",
        "PGDATA",
        "PGDATABASE",
        "PGPORT",
        "PGTZ",
        "PGUSER",
        "POSTGRES_DB",
        "POSTGRES_HOST_AUTH_METHOD",
        "POSTGRES_INITDB_ARGS",
        "POSTGRES_INITDB_WALDIR",
        "POSTGRES_USER",
        "TZ",
    }
)
"""Guest environment variables :func:`redact_inputs` keeps in clear: the PostgreSQL
image's documented settings that carry no credential. Every other value is hashed."""
_KEY_RE = re.compile(rf"^[0-9a-f]{{{KEY_LENGTH}}}$")


def cloud_base_url(env: Mapping[str, str] | None = None) -> str:
    """The Smol Cloud API URL in effect (``$SMOL_CLOUD_URL`` or the default), normalised."""
    env = os.environ if env is None else env
    return (env.get(CLOUD_URL_ENV) or DEFAULT_CLOUD_URL).strip().rstrip("/")


def smoltest_major(version: str = __version__) -> int:
    """The leading integer of a version string (``0`` when it has none)."""
    match = re.match(r"\d+", version)
    return int(match.group(0)) if match else 0


def is_cache_key(text: str) -> bool:
    """``True`` when ``text`` has the exact shape of a key this module produces."""
    return bool(_KEY_RE.match(text))


def canonical_json(value: Any) -> str:
    """Deterministic JSON: sorted keys, no whitespace, ASCII only."""
    return json.dumps(value, sort_keys=True, separators=(",", ":"), ensure_ascii=True)


def digest(text: str, length: int = KEY_LENGTH) -> str:
    """Hex SHA-256 of ``text`` truncated to ``length`` characters."""
    return hashlib.sha256(text.encode("utf-8")).hexdigest()[:length]


def redact_inputs(inputs: Mapping[str, Any]) -> dict[str, Any]:
    """Copy ``inputs`` with secrets replaced by ``sha256:<8 hex>``.

    Under ``env`` every value is hashed unless its name is in
    :data:`PLAIN_ENV_NAMES`: ``with_env`` carries arbitrary credentials
    (``API_KEY``, ``DATABASE_URL``, ``AUTH``, ...) that no name pattern can know
    in full. Elsewhere a value is hashed when its key names a secret
    (``PASSWORD``, ``PASSWD``, ``PASSPHRASE``, ``SECRET``, ``TOKEN``,
    ``CREDENTIAL``, ``API_KEY``, ``PRIVATE_KEY``; any case), at any depth, and a
    string inside any other list is hashed when its own text contains one of those
    words. Every string of ``argv`` is hashed: a command line can carry a secret
    in a form no keyword finds (``--password`` followed by the value,
    ``postgresql://user:pw@host/db``, ``Authorization: Bearer ...``). The digest
    lets two redacted files be compared without exposing the value; the cache key
    itself is computed from the real inputs.
    """
    return _redact_mapping(inputs)


def _redact_mapping(value: Mapping[Any, Any], *, env: bool = False) -> dict[str, Any]:
    out: dict[str, Any] = {}
    for k, v in value.items():
        key = str(k)
        secret = _SENSITIVE.search(key) is not None or (env and key not in PLAIN_ENV_NAMES)
        if secret and isinstance(v, str):
            out[key] = f"sha256:{digest(v, 8)}"
        elif key == "env" and isinstance(v, Mapping):
            out[key] = _redact_mapping(v, env=True)
        elif key == "argv" and isinstance(v, (list, tuple)):
            out[key] = [f"sha256:{digest(a, 8)}" if isinstance(a, str) else a for a in v]
        else:
            out[key] = _redact_value(v)
    return out


def _redact_value(value: Any) -> Any:
    if isinstance(value, Mapping):
        return _redact_mapping(value)
    if isinstance(value, (list, tuple)):
        return [_redact_item(v) for v in value]
    return value


def _redact_item(value: Any) -> Any:
    """A list element: a string that names a secret is hashed, everything else recurses."""
    if isinstance(value, str) and _SENSITIVE.search(value):
        return f"sha256:{digest(value, 8)}"
    return _redact_value(value)


# -- host signature ---------------------------------------------------------------


@dataclass(frozen=True)
class HostSignature:
    """What a local checkpoint depends on besides the machine spec."""

    os: str
    kernel_major: str
    arch: str
    cpu_flags_hash: str
    libc: str

    def as_dict(self) -> dict[str, str]:
        """A JSON-ready mapping with stable key names."""
        return {
            "os": self.os,
            "kernel_major": self.kernel_major,
            "arch": self.arch,
            "cpu_flags_hash": self.cpu_flags_hash,
            "libc": self.libc,
        }


def read_cpu_flags(cpuinfo: Path = CPUINFO_PATH) -> tuple[str, ...] | None:
    """The sorted, de-duplicated CPU feature flags from ``/proc/cpuinfo``, or ``None``.

    Reads the first ``flags`` (x86) or ``Features`` (arm64) line; missing or
    unreadable files yield ``None`` so callers can fall back.
    """
    try:
        text = cpuinfo.read_text(errors="replace")
    except OSError:
        return None
    for line in text.splitlines():
        name, sep, value = line.partition(":")
        if sep and name.strip().lower() in ("flags", "features"):
            return tuple(sorted(set(value.split())))
    return None


def compute_host_signature(cpuinfo: Path = CPUINFO_PATH) -> HostSignature:
    """Build the :class:`HostSignature` of this machine (uncached)."""
    flags = read_cpu_flags(cpuinfo)
    if flags is not None:
        flags_hash = digest(" ".join(flags), 16)
    else:
        flags_hash = digest(platform.processor() or platform.machine() or "unknown", 16)
    libc_name, libc_version = platform.libc_ver()
    libc = f"{libc_name}-{libc_version}" if libc_name or libc_version else ""
    return HostSignature(
        os=platform.system().lower(),
        kernel_major=platform.release().split(".")[0],
        arch=platform.machine(),
        cpu_flags_hash=flags_hash,
        libc=libc,
    )


@functools.lru_cache(maxsize=1)
def host_signature() -> HostSignature:
    """This host's signature, computed once per process."""
    return compute_host_signature()


# -- the key ------------------------------------------------------------------------


@dataclass(frozen=True)
class CacheKey:
    """A computed key together with the canonical inputs it hashes.

    ``inputs`` is unredacted (it holds the guest environment, credentials
    included); write it to disk only through :meth:`redacted_inputs`.
    """

    key: str
    inputs: Mapping[str, Any]

    def __str__(self) -> str:
        return self.key

    @property
    def seed_key(self) -> str | None:
        """The seed key folded into this key, if any."""
        value = self.inputs.get("seed_key")
        return str(value) if value is not None else None

    @property
    def target(self) -> str:
        """The target the key was computed for."""
        return str(self.inputs["target"])

    def canonical_json(self) -> str:
        """The exact text whose SHA-256 is :attr:`key`."""
        return canonical_json(self.inputs)

    def redacted_inputs(self) -> dict[str, Any]:
        """The inputs with secrets replaced by short digests (see :func:`redact_inputs`)."""
        return redact_inputs(self.inputs)

    def with_seed(self, seed_key: str | None) -> CacheKey:
        """The key of the same shape with ``seed_key`` applied (``None`` = base key)."""
        return CacheKey.from_inputs({**self.inputs, "seed_key": seed_key})

    @classmethod
    def from_inputs(cls, inputs: Mapping[str, Any]) -> CacheKey:
        """Hash an already assembled canonical input mapping."""
        canonical = dict(inputs)
        return cls(digest(canonical_json(canonical)), canonical)

    @classmethod
    def from_machine_spec(
        cls,
        spec: MachineSpec,
        target: Target,
        sdk_version: str,
        seed_key: str | None = None,
        *,
        host: HostSignature | None = None,
        env: Mapping[str, str] | None = None,
    ) -> CacheKey:
        """Compute the key of an engine-level :class:`~smoltest.transport.base.MachineSpec`.

        The first mapping in ``spec.ports`` is the PostgreSQL port (by construction
        of :func:`~smoltest.boot.spec.build_machine_spec`); its host side is the
        variant port and is ignored, only its guest port counts. Every later
        mapping is an extra port: its guest port counts, and so does its host port
        when it is pinned (``pinned_ports``, sorted ``[guest, host]`` pairs),
        because a restored machine publishes the extra port on the host port the
        checkpoint recorded. ``host`` overrides the detected host signature
        (tests); ``env`` is the process environment the cloud URL is read from.
        """
        local = target == "local"
        signature = (host or host_signature()).as_dict() if local else None
        pinned_ports = sorted(
            [mapping.guest, mapping.host] for mapping in spec.ports[1:] if mapping.host is not None
        )
        inputs: dict[str, Any] = {
            "format": KEY_FORMAT,
            "smoltest_major": smoltest_major(),
            "sdk_version": sdk_version,
            "target": target,
            "host": signature,
            "base_url": None if local else cloud_base_url(env),
            "image": spec.image,
            "argv": list(spec.argv) if spec.argv is not None else None,
            "env": dict(sorted(spec.env.items())),
            "guest_ports": sorted(spec.guest_ports),
            "pinned_ports": pinned_ports,
            "cpus": spec.cpus,
            "memory_mb": spec.memory_mb,
            "storage_gb": spec.storage_gb,
            "network": spec.network,
            "seed_key": seed_key,
        }
        return cls.from_inputs(inputs)

    @classmethod
    def compute(
        cls,
        spec: PostgresSpec,
        settings: Settings,
        target: Target,
        engine: Engine,
        seed_key: str | None = None,
        *,
        host: HostSignature | None = None,
        env: Mapping[str, str] | None = None,
    ) -> CacheKey:
        """Compute the key for booting ``spec`` under ``settings`` on ``target``.

        Resolves the PostgreSQL spec to a machine spec exactly as the boot does
        (image, argv, environment, ports, resources) and hashes it with the
        engine's SDK version, the host signature (local) or cloud URL, and
        ``seed_key`` when the checkpoint will contain seeded data.
        """
        machine = build_machine_spec(spec, settings, target, host_port=None)
        return cls.from_machine_spec(
            machine, target, engine.sdk_version(), seed_key, host=host, env=env
        )


__all__ = [
    "CLOUD_URL_ENV",
    "CPUINFO_PATH",
    "DEFAULT_CLOUD_URL",
    "KEY_FORMAT",
    "KEY_LENGTH",
    "PLAIN_ENV_NAMES",
    "CacheKey",
    "HostSignature",
    "canonical_json",
    "cloud_base_url",
    "compute_host_signature",
    "digest",
    "host_signature",
    "is_cache_key",
    "read_cpu_flags",
    "redact_inputs",
    "smoltest_major",
]
