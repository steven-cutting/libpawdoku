"""Entry point of the ``smoltest`` command.

Global options (``--cache-dir``, ``--target``, ``-v``) are accepted before or
after the command name. Exit status: 0 ok, 1 failure, 2 usage error, 3 no Smol
target available.
"""

from __future__ import annotations

import argparse
import contextlib
import logging
import sys
import threading
import warnings
from collections.abc import Iterator, Sequence
from pathlib import Path

from .._log import logger
from .._version import __version__
from ..config import ENV_VARS, Settings
from ..errors import InvalidConfig, SmoltestError, SmoltestWarning, TargetUnavailable
from . import (
    EXIT_FAILURE,
    EXIT_OK,
    EXIT_TARGET_UNAVAILABLE,
    EXIT_USAGE,
    Context,
    cache_cmd,
    doctor,
    print_err,
    run_cmd,
    warm,
)

TARGETS = ("auto", "local", "cloud")


def global_options() -> argparse.ArgumentParser:
    """The parent parser every (sub)command shares.

    Defaults are suppressed so an option given before the command name is not
    overwritten by the subcommand's default.
    """
    parent = argparse.ArgumentParser(add_help=False)
    group = parent.add_argument_group("global options")
    group.add_argument(
        "--cache-dir",
        type=Path,
        metavar="DIR",
        default=argparse.SUPPRESS,
        help="checkpoint cache directory (default: $SMOLTEST_CACHE_DIR, else ~/.cache/smoltest)",
    )
    group.add_argument(
        "--target",
        choices=TARGETS,
        default=argparse.SUPPRESS,
        help="where machines run (default: $SMOLTEST_TARGET, else auto)",
    )
    group.add_argument(
        "-v",
        "--verbose",
        action="count",
        default=argparse.SUPPRESS,
        help="show the library's progress on stderr (-vv for debug output)",
    )
    return parent


def build_parser() -> argparse.ArgumentParser:
    """The complete ``smoltest`` argument parser."""
    common = global_options()
    env_names = ", ".join(var.name for var in ENV_VARS)
    parser = argparse.ArgumentParser(
        prog="smoltest",
        description="PostgreSQL test machines in Smol Machines microVMs.",
        epilog=(
            "exit status: 0 ok, 1 failure, 2 usage error, 3 no Smol target available\n"
            f"environment: {env_names}, SMOL_CLOUD_TOKEN, SMOL_CLOUD_URL"
        ),
        formatter_class=argparse.RawDescriptionHelpFormatter,
        parents=[common],
    )
    parser.add_argument("--version", action="version", version=f"%(prog)s {__version__}")
    commands = parser.add_subparsers(dest="command", metavar="COMMAND", required=True)
    doctor.register(commands, common)
    warm.register(commands, common)
    cache_cmd.register(commands, common)
    run_cmd.register(commands, common)
    return parser


class _Formatter(logging.Formatter):
    """``smoltest: <level>: <message>`` lines for the library's log records."""

    def format(self, record: logging.LogRecord) -> str:
        text = f"smoltest: {record.levelname.lower()}: {record.getMessage()}"
        if record.exc_info:
            text = f"{text}\n{self.formatException(record.exc_info)}"
        return text


@contextlib.contextmanager
def cli_output(verbose: int) -> Iterator[None]:
    """Route the library's log records to stderr while a command runs.

    ``SmoltestWarning`` is silenced meanwhile because :func:`smoltest._log.warn_once`
    also logs every warning, which would print it twice.
    """
    level = logging.DEBUG if verbose >= 2 else logging.INFO if verbose == 1 else logging.WARNING
    handler = logging.StreamHandler(sys.stderr)
    handler.setFormatter(_Formatter())
    handler.setLevel(level)
    previous_level = logger.level
    logger.addHandler(handler)
    if logger.getEffectiveLevel() > level:
        logger.setLevel(level)
    try:
        with warnings.catch_warnings():
            warnings.simplefilter("ignore", SmoltestWarning)
            yield
    finally:
        logger.removeHandler(handler)
        logger.setLevel(previous_level)


def _exit_status(code: object) -> int:
    """The exit status a ``SystemExit`` raised by argparse stands for."""
    if code is None:
        return EXIT_OK
    if isinstance(code, int):
        return code
    print(code, file=sys.stderr)
    return EXIT_FAILURE


def _dispatch(args: argparse.Namespace, stop_event: threading.Event | None) -> int:
    verbose: int = getattr(args, "verbose", 0)
    try:
        settings = Settings.from_env(
            cache_dir=getattr(args, "cache_dir", None),
            target=getattr(args, "target", None),
        )
        status: int = args.func(args, Context(settings, stop_event, verbose))
    except TargetUnavailable as exc:
        print_err(f"error: {exc}")
        return EXIT_TARGET_UNAVAILABLE
    except InvalidConfig as exc:
        print_err(f"error: {exc}")
        return EXIT_USAGE
    except SmoltestError as exc:
        print_err(f"error: {exc}")
        return EXIT_FAILURE
    except KeyboardInterrupt:
        print_err("interrupted")
        return EXIT_FAILURE
    return status


def main(argv: Sequence[str] | None = None, stop_event: threading.Event | None = None) -> int:
    """Run the CLI with ``argv`` (default ``sys.argv[1:]``) and return an exit status.

    ``stop_event`` lets a caller end the blocking ``run`` command without a signal.
    """
    parser = build_parser()
    try:
        args = parser.parse_args(argv)
    except SystemExit as exc:  # argparse: usage errors exit 2, --help and --version exit 0
        return _exit_status(exc.code)
    with cli_output(getattr(args, "verbose", 0)):
        return _dispatch(args, stop_event)


if __name__ == "__main__":  # pragma: no cover
    sys.exit(main())


__all__ = [
    "EXIT_FAILURE",
    "EXIT_OK",
    "EXIT_TARGET_UNAVAILABLE",
    "EXIT_USAGE",
    "TARGETS",
    "build_parser",
    "cli_output",
    "global_options",
    "main",
]
