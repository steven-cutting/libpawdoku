"""Exception hierarchy and the mapping from Smol SDK errors."""

from __future__ import annotations

from typing import Any


class SmoltestWarning(UserWarning):
    """Category of every warning smoltest emits."""


class SmoltestError(Exception):
    """Base class of every error smoltest raises."""

    code: str | None

    def __init__(self, message: str = "", *, code: str | None = None) -> None:
        super().__init__(message)
        self.code = code


class InvalidConfig(SmoltestError, ValueError):
    """A setting, environment variable or spec value is malformed."""


class TargetUnavailable(SmoltestError):
    """Neither a local engine nor cloud credentials are available."""

    def __init__(
        self,
        code: str | None,
        reason: str | None,
        *,
        hint: str = "run `smoltest doctor` for details",
    ) -> None:
        self.reason = reason
        self.hint = hint
        text = f"no Smol target available ({code or 'UNKNOWN'})"
        if reason:
            text += f": {reason}"
        super().__init__(f"{text}; {hint}", code=code)


class BootError(SmoltestError):
    """Booting a machine failed at ``stage`` because of ``cause``."""

    def __init__(
        self,
        message: str,
        *,
        stage: str = "boot",
        cause: BaseException | None = None,
        code: str | None = None,
    ) -> None:
        super().__init__(f"[{stage}] {message}", code=code)
        self.stage = stage
        self.cause = cause
        if cause is not None:
            self.__cause__ = cause


class ReadinessTimeout(BootError, TimeoutError):
    """The readiness wait strategy did not succeed within its timeout."""

    def __init__(self, message: str, *, timeout_s: float | None = None) -> None:
        super().__init__(message, stage="wait")
        self.timeout_s = timeout_s


class CacheError(SmoltestError):
    """The checkpoint cache could not be read or written."""


class CacheCorrupt(CacheError):
    """A cache entry is unreadable or inconsistent and should be invalidated."""


class NotSupportedError(SmoltestError):
    """The operation is not supported by the engine, target or machine."""


class ExecError(SmoltestError):
    """An in-guest command could not be executed at all (not a non-zero exit)."""


_SMOL_BY_NAME: dict[str, type[SmoltestError]] = {
    "NotSupportedError": NotSupportedError,
    "InvalidConfigError": InvalidConfig,
    "ExecutionError": ExecError,
}


def map_smol_error(exc: BaseException, *, stage: str | None = None) -> SmoltestError:
    """Translate a Smol SDK exception into the smoltest hierarchy.

    The SDK is matched by class name so this module never imports ``smol``. A
    ``stage`` wraps the result in :class:`BootError`. Exceptions that already are
    smoltest errors are returned unchanged.
    """
    if isinstance(exc, SmoltestError):
        return exc
    code: Any = getattr(exc, "code", None)
    code_str = str(code) if code is not None else None
    for klass in type(exc).__mro__:
        target = _SMOL_BY_NAME.get(klass.__name__)
        if target is not None:
            mapped = target(str(exc) or klass.__name__, code=code_str)
            break
    else:
        mapped = SmoltestError(f"{type(exc).__name__}: {exc}", code=code_str)
    mapped.__cause__ = exc
    if stage is not None:
        return BootError(str(mapped), stage=stage, cause=exc, code=code_str)
    return mapped


__all__ = [
    "BootError",
    "CacheCorrupt",
    "CacheError",
    "ExecError",
    "InvalidConfig",
    "NotSupportedError",
    "ReadinessTimeout",
    "SmoltestError",
    "SmoltestWarning",
    "TargetUnavailable",
    "map_smol_error",
]
