"""Package version, read from installed metadata."""

from __future__ import annotations

from importlib.metadata import PackageNotFoundError, version

try:
    __version__: str = version("smoltest")
except PackageNotFoundError:  # pragma: no cover - only when run from an unbuilt checkout
    __version__ = "0.0.0+unknown"

__all__ = ["__version__"]
