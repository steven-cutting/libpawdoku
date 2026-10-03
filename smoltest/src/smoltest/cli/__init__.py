"""The ``smoltest`` command-line interface (argparse only).

``doctor`` reports which Smol target can boot here, ``warm`` fills the
checkpoint cache, ``cache`` inspects and trims it and ``run`` keeps one
PostgreSQL machine up until interrupted. This module holds what the command
modules share: exit codes, the :class:`Context` every command receives, and
small parsing and formatting helpers. :mod:`smoltest.cli.main` wires the
commands together.
"""

from __future__ import annotations

import argparse
import re
import sys
import threading
from collections.abc import Iterable, Sequence
from dataclasses import dataclass

from ..config import Settings
from ..errors import InvalidConfig
from ..postgres import Seed

EXIT_OK = 0
EXIT_FAILURE = 1
EXIT_USAGE = 2
EXIT_TARGET_UNAVAILABLE = 3

KEY_WIDTH = 12
"""Characters of a cache key shown in tables; a prefix resolves with ``find_key``."""


@dataclass(frozen=True)
class Context:
    """What every command receives next to its parsed arguments."""

    settings: Settings
    stop_event: threading.Event | None = None
    verbose: int = 0


# -- output -------------------------------------------------------------------------


def print_err(message: str) -> None:
    """Print ``smoltest: <message>`` to stderr."""
    print(f"smoltest: {message}", file=sys.stderr, flush=True)


def note(message: str) -> None:
    """Print a progress line to stderr; stdout is reserved for results."""
    print(message, file=sys.stderr, flush=True)


def short_key(key: str | None) -> str:
    """The first :data:`KEY_WIDTH` characters of a cache key, or ``-``."""
    return key[:KEY_WIDTH] if key else "-"


def human_bytes(count: int | None) -> str:
    """``1536`` as ``1.5 KiB``; ``None`` as ``-``."""
    if count is None:
        return "-"
    if count < 1024:
        return f"{count} B"
    value = float(count)
    for unit in ("KiB", "MiB", "GiB"):
        value /= 1024
        if value < 1024:
            return f"{value:.1f} {unit}"
    return f"{value / 1024:.1f} TiB"


def human_age(seconds: float | None) -> str:
    """An age in seconds as ``45s``, ``3m``, ``5h`` or ``2d``; ``None`` as ``-``."""
    if seconds is None:
        return "-"
    seconds = max(seconds, 0.0)
    if seconds < 60:
        return f"{int(seconds)}s"
    if seconds < 3600:
        return f"{int(seconds // 60)}m"
    if seconds < 86400:
        return f"{int(seconds // 3600)}h"
    return f"{int(seconds // 86400)}d"


def format_table(
    headers: Sequence[str],
    rows: Iterable[Sequence[object]],
    *,
    align: str = "",
) -> str:
    """Plain-text columns separated by two spaces.

    ``align`` holds one ``l`` or ``r`` per column (missing entries mean left).
    """
    table = [[str(cell) for cell in row] for row in rows]
    widths = [len(header) for header in headers]
    for row in table:
        for index, cell in enumerate(row):
            widths[index] = max(widths[index], len(cell))

    def render(cells: Sequence[str]) -> str:
        parts = []
        for index, cell in enumerate(cells):
            right = index < len(align) and align[index] == "r"
            parts.append(cell.rjust(widths[index]) if right else cell.ljust(widths[index]))
        return "  ".join(parts).rstrip()

    lines = [render(headers)]
    lines.extend(render(row) for row in table)
    return "\n".join(lines)


def confirm(question: str, *, assume_yes: bool) -> int:
    """Ask ``question`` on a terminal.

    Returns :data:`EXIT_OK` to proceed, :data:`EXIT_USAGE` when stdin is not a
    terminal and ``--yes`` was not given, and :data:`EXIT_FAILURE` when the
    user declined.
    """
    if assume_yes:
        return EXIT_OK
    stdin = getattr(sys, "stdin", None)
    if stdin is None or not stdin.isatty():
        print_err("error: refusing without --yes because standard input is not a terminal")
        return EXIT_USAGE
    try:
        answer = input(f"{question} [y/N] ")
    except EOFError:
        answer = ""
    if answer.strip().lower() in ("y", "yes"):
        return EXIT_OK
    note("aborted")
    return EXIT_FAILURE


# -- parsing ------------------------------------------------------------------------

_DURATION_RE = re.compile(r"^\s*(\d+(?:\.\d+)?)\s*([smhdw]?)\s*$", re.IGNORECASE)
_DURATION_UNITS = {"": 1.0, "s": 1.0, "m": 60.0, "h": 3600.0, "d": 86400.0, "w": 604800.0}
_SIZE_RE = re.compile(r"^\s*(\d+(?:\.\d+)?)\s*([kmgt]?)(?:i?b)?\s*$", re.IGNORECASE)
_SIZE_UNITS = {"": 1, "k": 2**10, "m": 2**20, "g": 2**30, "t": 2**40}


def parse_duration(text: str) -> float:
    """``14d``, ``36h``, ``90m``, ``45s``, ``2w`` or bare seconds, as seconds."""
    match = _DURATION_RE.match(text)
    if match is None:
        raise InvalidConfig(f"expected a duration like 14d, 36h, 90m or 45s; got {text!r}")
    value, unit = match.groups()
    return float(value) * _DURATION_UNITS[unit.lower()]


def parse_size(text: str) -> int:
    """``2G``, ``500M``, ``10GiB``, ``64KB`` or bare bytes, as bytes (binary multiples)."""
    match = _SIZE_RE.match(text)
    if match is None:
        raise InvalidConfig(f"expected a size like 2G, 500M or 1024; got {text!r}")
    value, unit = match.groups()
    return int(float(value) * _SIZE_UNITS[unit.lower()])


def parse_env_assignments(items: Iterable[str]) -> dict[str, str]:
    """``KEY=VALUE`` strings as a mapping; a later key overrides an earlier one."""
    env: dict[str, str] = {}
    for item in items:
        key, sep, value = item.partition("=")
        if not sep or not key.strip():
            raise InvalidConfig(f"expected KEY=VALUE for --env, got {item!r}")
        env[key.strip()] = value
    return env


def duration_arg(text: str) -> float:
    """argparse ``type`` for :func:`parse_duration`."""
    try:
        return parse_duration(text)
    except InvalidConfig as exc:
        raise argparse.ArgumentTypeError(str(exc)) from None


def size_arg(text: str) -> int:
    """argparse ``type`` for :func:`parse_size`."""
    try:
        return parse_size(text)
    except InvalidConfig as exc:
        raise argparse.ArgumentTypeError(str(exc)) from None


def _int_arg(text: str, *, minimum: int, maximum: int | None = None) -> int:
    try:
        value = int(text)
    except ValueError:
        raise argparse.ArgumentTypeError(f"expected an integer, got {text!r}") from None
    if value < minimum or (maximum is not None and value > maximum):
        bound = f"{minimum}..{maximum}" if maximum is not None else f">= {minimum}"
        raise argparse.ArgumentTypeError(f"expected an integer {bound}, got {text!r}")
    return value


def positive_int(text: str) -> int:
    """argparse ``type``: an integer ``>= 1``."""
    return _int_arg(text, minimum=1)


def nonnegative_int(text: str) -> int:
    """argparse ``type``: an integer ``>= 0``."""
    return _int_arg(text, minimum=0)


def port_arg(text: str) -> int:
    """argparse ``type``: a TCP port number."""
    return _int_arg(text, minimum=1, maximum=65535)


# -- shared PostgreSQL options ---------------------------------------------------------


def add_postgres_options(parser: argparse.ArgumentParser) -> None:
    """Options ``warm postgres`` and ``run postgres`` have in common."""
    parser.add_argument(
        "--image",
        metavar="IMAGE",
        help="PostgreSQL image (default: $SMOLTEST_POSTGRES_IMAGE, else postgres:16)",
    )
    parser.add_argument(
        "--no-fast",
        action="store_true",
        help="keep fsync and synchronous commits on (slower, durable)",
    )
    parser.add_argument(
        "--seed-sql",
        action="extend",
        nargs="+",
        default=[],
        metavar="FILE",
        help="SQL files to run once the server is ready, in order; cached by content hash",
    )
    parser.add_argument(
        "--env",
        action="extend",
        nargs="+",
        default=[],
        metavar="KEY=VALUE",
        help="extra guest environment variables",
    )


def load_seed(paths: Sequence[str]) -> Seed | None:
    """A :class:`~smoltest.postgres.Seed` for ``--seed-sql`` files, or ``None`` without any."""
    if not paths:
        return None
    try:
        return Seed.from_sql_files(*paths)
    except OSError as exc:
        raise InvalidConfig(f"cannot read seed file: {exc}") from exc


__all__ = [
    "EXIT_FAILURE",
    "EXIT_OK",
    "EXIT_TARGET_UNAVAILABLE",
    "EXIT_USAGE",
    "KEY_WIDTH",
    "Context",
    "add_postgres_options",
    "confirm",
    "duration_arg",
    "format_table",
    "human_age",
    "human_bytes",
    "load_seed",
    "nonnegative_int",
    "note",
    "parse_duration",
    "parse_env_assignments",
    "parse_size",
    "port_arg",
    "positive_int",
    "print_err",
    "short_key",
    "size_arg",
]
