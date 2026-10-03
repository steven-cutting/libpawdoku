"""smoltest: testcontainers-style PostgreSQL fixtures on Smol Machines microVMs.

Public names are imported lazily so ``import smoltest`` stays cheap and never
pulls in the Smol SDK.
"""

from __future__ import annotations

import importlib
from typing import TYPE_CHECKING, Any

from ._version import __version__

if TYPE_CHECKING:
    from .boot.golden import GoldenRegistry, PostgresGolden
    from .boot.strategy import BootInfo
    from .config import Settings
    from .errors import (
        BootError,
        CacheCorrupt,
        CacheError,
        ExecError,
        InvalidConfig,
        NotSupportedError,
        ReadinessTimeout,
        SmoltestError,
        SmoltestWarning,
        TargetUnavailable,
    )
    from .postgres import AsyncPostgresMachine, ExecResult, PostgresContainer, PostgresMachine, Seed
    from .wait.strategies import (
        CompositeWaitStrategy,
        ExecWaitStrategy,
        LogMessageWaitStrategy,
        PgIsReadyWaitStrategy,
        PortWaitStrategy,
        SqlWaitStrategy,
        WaitStrategy,
        default_wait,
    )

_LAZY: dict[str, str] = {
    # facades
    "PostgresMachine": ".postgres",
    "PostgresContainer": ".postgres",
    "AsyncPostgresMachine": ".postgres",
    "Seed": ".postgres",
    "ExecResult": ".postgres",
    # golden machines
    "PostgresGolden": ".boot.golden",
    "GoldenRegistry": ".boot.golden",
    "BootInfo": ".boot.strategy",
    # settings
    "Settings": ".config",
    # wait strategies
    "WaitStrategy": ".wait.strategies",
    "PortWaitStrategy": ".wait.strategies",
    "ExecWaitStrategy": ".wait.strategies",
    "PgIsReadyWaitStrategy": ".wait.strategies",
    "SqlWaitStrategy": ".wait.strategies",
    "LogMessageWaitStrategy": ".wait.strategies",
    "CompositeWaitStrategy": ".wait.strategies",
    "default_wait": ".wait.strategies",
    # errors
    "SmoltestError": ".errors",
    "SmoltestWarning": ".errors",
    "InvalidConfig": ".errors",
    "TargetUnavailable": ".errors",
    "BootError": ".errors",
    "ReadinessTimeout": ".errors",
    "CacheError": ".errors",
    "CacheCorrupt": ".errors",
    "NotSupportedError": ".errors",
    "ExecError": ".errors",
}

__all__ = [
    "AsyncPostgresMachine",
    "BootError",
    "BootInfo",
    "CacheCorrupt",
    "CacheError",
    "CompositeWaitStrategy",
    "ExecError",
    "ExecResult",
    "ExecWaitStrategy",
    "GoldenRegistry",
    "InvalidConfig",
    "LogMessageWaitStrategy",
    "NotSupportedError",
    "PgIsReadyWaitStrategy",
    "PortWaitStrategy",
    "PostgresContainer",
    "PostgresGolden",
    "PostgresMachine",
    "ReadinessTimeout",
    "Seed",
    "Settings",
    "SmoltestError",
    "SmoltestWarning",
    "SqlWaitStrategy",
    "TargetUnavailable",
    "WaitStrategy",
    "__version__",
    "default_wait",
]


def __getattr__(name: str) -> Any:
    module_name = _LAZY.get(name)
    if module_name is None:
        raise AttributeError(f"module 'smoltest' has no attribute {name!r}")
    module = importlib.import_module(module_name, __name__)
    value = getattr(module, name)
    globals()[name] = value
    return value


def __dir__() -> list[str]:
    return sorted(set(globals()) | set(_LAZY))
