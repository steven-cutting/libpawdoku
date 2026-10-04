"""Settings, their environment variables and target resolution."""

from __future__ import annotations

import dataclasses
import math
import os
import sys
from collections.abc import Callable, Mapping
from dataclasses import dataclass, field
from pathlib import Path
from typing import TYPE_CHECKING, Any, Literal

from .errors import InvalidConfig, TargetUnavailable

if TYPE_CHECKING:
    from .transport.base import Engine

Target = Literal["auto", "local", "cloud"]
ResolvedTarget = Literal["local", "cloud"]

_TARGETS: tuple[Target, ...] = ("auto", "local", "cloud")
_TRUE = frozenset({"1", "true", "yes", "on"})
_FALSE = frozenset({"0", "false", "no", "off", ""})


def default_cache_dir(env: Mapping[str, str] | None = None) -> Path:
    """Return the checkpoint cache directory for this user.

    Precedence: ``$SMOLTEST_CACHE_DIR``, ``$XDG_CACHE_HOME/smoltest``,
    ``~/Library/Caches/smoltest`` on macOS, else ``~/.cache/smoltest``.
    """
    env = os.environ if env is None else env
    explicit = env.get("SMOLTEST_CACHE_DIR")
    if explicit:
        return Path(explicit).expanduser()
    xdg = env.get("XDG_CACHE_HOME")
    if xdg:
        return Path(xdg).expanduser() / "smoltest"
    home = Path(env.get("HOME") or "~").expanduser()
    if sys.platform == "darwin":
        return home / "Library" / "Caches" / "smoltest"
    return home / ".cache" / "smoltest"


def parse_bool(raw: str) -> bool:
    """Parse an environment boolean (``1/true/yes/on`` or ``0/false/no/off``)."""
    value = raw.strip().lower()
    if value in _TRUE:
        return True
    if value in _FALSE:
        return False
    raise InvalidConfig(f"expected a boolean, got {raw!r}")


def _parse_int(raw: str) -> int:
    try:
        return int(raw.strip())
    except ValueError as exc:
        raise InvalidConfig(f"expected an integer, got {raw!r}") from exc


def _parse_float(raw: str) -> float:
    try:
        value = float(raw.strip())
    except ValueError as exc:
        raise InvalidConfig(f"expected a number, got {raw!r}") from exc
    if not math.isfinite(value):
        raise InvalidConfig(f"expected a finite number, got {raw!r}")
    return value


def _parse_target(raw: str) -> Target:
    value = raw.strip().lower()
    if value not in _TARGETS:
        raise InvalidConfig(f"SMOLTEST_TARGET must be one of {', '.join(_TARGETS)}; got {raw!r}")
    return value


def _parse_path(raw: str) -> Path:
    return Path(raw).expanduser()


def _parse_str(raw: str) -> str:
    return raw


@dataclass(frozen=True)
class EnvVar:
    """One environment variable smoltest reads, and the setting it feeds."""

    name: str
    field: str
    parse: Callable[[str], Any]
    description: str


ENV_VARS: tuple[EnvVar, ...] = (
    EnvVar("SMOLTEST_TARGET", "target", _parse_target, "auto | local | cloud"),
    EnvVar("SMOLTEST_CACHE_DIR", "cache_dir", _parse_path, "checkpoint cache directory"),
    EnvVar("SMOLTEST_POSTGRES_IMAGE", "postgres_image", _parse_str, "default PostgreSQL image"),
    EnvVar("SMOLTEST_DISABLE_CACHE", "disable_cache", parse_bool, "never restore or populate"),
    EnvVar("SMOLTEST_DISABLE_BRANCH", "disable_branch", parse_bool, "never branch a golden"),
    EnvVar("SMOLTEST_CPUS", "cpus", _parse_int, "guest vCPUs"),
    EnvVar("SMOLTEST_MEMORY_MB", "memory_mb", _parse_int, "guest memory in MiB"),
    EnvVar("SMOLTEST_NETWORK", "network", parse_bool, "give the guest outbound network"),
    EnvVar("SMOLTEST_FAST", "fast_mode", parse_bool, "fsync=off and friends"),
    EnvVar("SMOLTEST_READY_TIMEOUT", "ready_timeout_s", _parse_float, "readiness timeout (s)"),
    EnvVar("SMOLTEST_EXEC_TIMEOUT", "exec_timeout_s", _parse_float, "in-guest command timeout (s)"),
    EnvVar("SMOLTEST_CACHE_MAX_BYTES", "cache_max_bytes", _parse_int, "LRU cache budget"),
    EnvVar("SMOLTEST_CLOUD_AUTO_STOP", "cloud_auto_stop_seconds", _parse_int, "cloud auto-stop"),
    EnvVar("SMOLTEST_CLOUD_TTL", "cloud_ttl_seconds", _parse_int, "cloud time-to-live"),
    EnvVar("SMOLTEST_ENGINE", "engine_path", _parse_str, "engine factory 'pkg.mod:callable'"),
)
"""Every ``SMOLTEST_*`` variable, in documentation order."""

ENV_BY_FIELD: Mapping[str, EnvVar] = {var.field: var for var in ENV_VARS}


@dataclass(frozen=True)
class Settings:
    """Process-wide knobs; immutable, with ``replace`` for variants."""

    target: Target = "auto"
    cache_dir: Path = field(default_factory=default_cache_dir)
    disable_cache: bool = False
    disable_branch: bool = False
    postgres_image: str = "postgres:16"
    cpus: int = 1
    memory_mb: int = 512
    storage_gb: int | None = None
    network: bool = True
    fast_mode: bool = True
    ready_timeout_s: float = 120.0
    poll_interval_s: float = 0.05
    exec_timeout_s: float = 600.0
    cache_max_bytes: int = 10 * 2**30
    cloud_auto_stop_seconds: int = 1800
    cloud_ttl_seconds: int = 7200
    engine_path: str | None = None

    def __post_init__(self) -> None:
        if self.target not in _TARGETS:
            raise InvalidConfig(f"target must be one of {', '.join(_TARGETS)}; got {self.target!r}")
        if self.cpus < 1:
            raise InvalidConfig(f"cpus must be >= 1; got {self.cpus}")
        if self.memory_mb < 64:
            raise InvalidConfig(f"memory_mb must be >= 64; got {self.memory_mb}")
        for name in ("ready_timeout_s", "poll_interval_s", "exec_timeout_s"):
            value = getattr(self, name)
            # NaN compares false with everything and infinity never expires: a
            # deadline built from either would wait forever, so both are rejected.
            if not math.isfinite(value) or value <= 0:
                raise InvalidConfig(f"{name} must be a finite number > 0; got {value}")
        object.__setattr__(self, "cache_dir", Path(self.cache_dir).expanduser())

    @classmethod
    def from_env(cls, env: Mapping[str, str] | None = None, /, **overrides: Any) -> Settings:
        """Build settings from ``env`` (default ``os.environ``) with ``overrides`` on top.

        Precedence is overrides > environment > defaults. The testcontainers
        variables ``TC_MAX_TRIES`` and ``TC_POOLING_INTERVAL`` set the readiness
        timeout when ``SMOLTEST_READY_TIMEOUT`` is absent. The default cache
        directory is derived from ``env`` too (see :func:`default_cache_dir`),
        never from the process environment when a mapping is supplied.
        """
        env = os.environ if env is None else env
        values: dict[str, Any] = {}
        for var in ENV_VARS:
            raw = env.get(var.name)
            if raw is None:
                continue
            try:
                values[var.field] = var.parse(raw)
            except InvalidConfig as exc:
                raise InvalidConfig(f"{var.name}: {exc}") from None
        values.setdefault("cache_dir", default_cache_dir(env))
        if "ready_timeout_s" not in values and "TC_MAX_TRIES" in env:
            tries = _parse_int(env["TC_MAX_TRIES"])
            interval = _parse_float(env.get("TC_POOLING_INTERVAL", "1"))
            values["ready_timeout_s"] = tries * interval
        unknown = set(overrides) - {f.name for f in dataclasses.fields(cls)}
        if unknown:
            raise InvalidConfig(f"unknown settings: {', '.join(sorted(unknown))}")
        values.update({k: v for k, v in overrides.items() if v is not None})
        return cls(**values)

    def replace(self, **changes: Any) -> Settings:
        """Return a copy with ``changes`` applied; ``None`` values are ignored."""
        return dataclasses.replace(self, **{k: v for k, v in changes.items() if v is not None})


def resolve_target(
    settings: Settings,
    engine: Engine,
    env: Mapping[str, str] | None = None,
) -> ResolvedTarget:
    """Pick ``local`` or ``cloud`` for ``settings``.

    An explicit target wins; otherwise ``SMOL_CLOUD_TOKEN`` selects the cloud,
    then a usable local engine selects ``local``; else :class:`TargetUnavailable`
    carries the engine's availability code and reason.
    """
    if settings.target != "auto":
        return settings.target
    env = os.environ if env is None else env
    if env.get("SMOL_CLOUD_TOKEN"):
        return "cloud"
    ok, code, reason = engine.local_availability()
    if ok:
        return "local"
    raise TargetUnavailable(code, reason)


__all__ = [
    "ENV_BY_FIELD",
    "ENV_VARS",
    "EnvVar",
    "ResolvedTarget",
    "Settings",
    "Target",
    "default_cache_dir",
    "parse_bool",
    "resolve_target",
]
