"""Engine selection: the Smol SDK by default, anything else via ``SMOLTEST_ENGINE``."""

from __future__ import annotations

import importlib
import os
from collections.abc import Callable
from typing import Any

from ..config import Settings
from ..errors import InvalidConfig
from .base import Engine

ENGINE_ENV = "SMOLTEST_ENGINE"
DEFAULT_ENGINE = "smoltest.transport.smol_engine:SmolEngine"


def load_engine_factory(path: str) -> Callable[[], Engine]:
    """Resolve ``"pkg.mod:callable"`` (dotted attributes allowed) to a zero-arg factory."""
    module_name, sep, attr_path = path.partition(":")
    if not sep or not module_name or not attr_path:
        raise InvalidConfig(f"{ENGINE_ENV} must look like 'pkg.mod:callable'; got {path!r}")
    try:
        obj: Any = importlib.import_module(module_name)
    except ImportError as exc:
        raise InvalidConfig(f"{ENGINE_ENV}: cannot import {module_name!r}: {exc}") from exc
    for part in attr_path.split("."):
        obj = getattr(obj, part, None)
        if obj is None:
            raise InvalidConfig(f"{ENGINE_ENV}: {path!r} has no attribute {part!r}")
    if not callable(obj):
        raise InvalidConfig(f"{ENGINE_ENV}: {path!r} is not callable")
    factory: Callable[[], Engine] = obj
    return factory


def get_engine(settings: Settings | None = None) -> Engine:
    """Instantiate the engine ``settings.engine_path`` or ``$SMOLTEST_ENGINE`` names.

    Without either, the Smol SDK engine is imported lazily so test suites that
    only ever use a fake never import ``smol``.
    """
    path = (settings.engine_path if settings is not None else None) or os.environ.get(ENGINE_ENV)
    return load_engine_factory(path or DEFAULT_ENGINE)()


__all__ = ["DEFAULT_ENGINE", "ENGINE_ENV", "Engine", "get_engine", "load_engine_factory"]
