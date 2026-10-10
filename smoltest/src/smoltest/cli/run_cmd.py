"""``smoltest run postgres``: boot one machine and keep it up until interrupted.

Prints the connection URL and how the machine booted, then blocks until
``SIGINT``/``SIGTERM`` arrives or the ``stop_event`` given to ``main`` is set,
and finally deletes the machine.
"""

from __future__ import annotations

import argparse
import contextlib
import signal
import sys
import threading
from collections.abc import Iterator
from types import FrameType

from ..config import resolve_target
from ..errors import SmoltestError
from ..postgres import PostgresMachine
from ..transport import get_engine
from . import (
    EXIT_FAILURE,
    EXIT_OK,
    Context,
    add_postgres_options,
    load_seed,
    note,
    parse_env_assignments,
    port_arg,
    print_err,
)

STOP_SIGNALS = (signal.SIGINT, signal.SIGTERM)
POLL_S = 0.5


def register(
    commands: argparse._SubParsersAction[argparse.ArgumentParser],
    common: argparse.ArgumentParser,
) -> None:
    """Add the ``run`` command."""
    parser = commands.add_parser(
        "run",
        parents=[common],
        help="boot one machine and keep it up until interrupted",
        description="Boot a machine, print how to reach it and delete it on Ctrl-C.",
    )
    services = parser.add_subparsers(dest="service", metavar="SERVICE", required=True)
    postgres = services.add_parser(
        "postgres",
        parents=[common],
        help="PostgreSQL",
        description="Prints a postgresql:// URL, then blocks until SIGINT or SIGTERM.",
    )
    add_postgres_options(postgres)
    postgres.add_argument(
        "--port",
        type=port_arg,
        metavar="HOST_PORT",
        help="host port to publish PostgreSQL on (default: a free one)",
    )
    postgres.add_argument(
        "--no-cache",
        action="store_true",
        help="always boot cold; never restore or populate the checkpoint cache",
    )
    postgres.set_defaults(func=run_postgres)


@contextlib.contextmanager
def stop_on_signals(stop: threading.Event) -> Iterator[None]:
    """Set ``stop`` on ``SIGINT``/``SIGTERM`` while the block runs (main thread only)."""
    if threading.current_thread() is not threading.main_thread():
        yield
        return

    def handler(signum: int, frame: FrameType | None) -> None:
        del frame
        note(f"received {signal.Signals(signum).name}, stopping")
        stop.set()

    previous = {sig: signal.signal(sig, handler) for sig in STOP_SIGNALS}
    try:
        yield
    finally:
        for sig, old in previous.items():
            signal.signal(sig, signal.SIG_DFL if old is None else old)


@contextlib.contextmanager
def interrupt_on_signals() -> Iterator[None]:
    """Turn ``SIGINT``/``SIGTERM`` into :class:`KeyboardInterrupt` while the block runs.

    Used around startup, where nothing polls a stop event: the exception unwinds
    the boot, whose cleanup deletes whatever machine it had created. Main thread
    only; elsewhere the block runs with the handlers it finds.
    """
    if threading.current_thread() is not threading.main_thread():
        yield
        return

    def handler(signum: int, frame: FrameType | None) -> None:
        del frame
        note(f"received {signal.Signals(signum).name} during startup, aborting")
        raise KeyboardInterrupt

    previous = {sig: signal.signal(sig, handler) for sig in STOP_SIGNALS}
    try:
        yield
    finally:
        for sig, old in previous.items():
            signal.signal(sig, signal.SIG_DFL if old is None else old)


def run_postgres(args: argparse.Namespace, ctx: Context) -> int:
    """Boot, print the URL and boot info, wait for a stop, delete."""
    settings = ctx.settings.replace(
        postgres_image=args.image,
        fast_mode=False if args.no_fast else None,
        disable_cache=True if args.no_cache else None,
    )
    env = parse_env_assignments(args.env)
    seed = load_seed(args.seed_sql)
    engine = get_engine(settings)
    if args.port is not None and resolve_target(settings, engine) == "cloud":
        print_err(
            "error: --port is not supported on the cloud target: the tunnel chooses its "
            "loopback port; drop --port or use --target local"
        )
        return EXIT_FAILURE
    machine = PostgresMachine(settings=settings, engine=engine, seed=seed)
    for key, value in env.items():
        machine.with_env(key, value)
    if args.port is not None:
        machine.with_bind_ports(machine.port, args.port)
    stop = ctx.stop_event if ctx.stop_event is not None else threading.Event()
    try:
        with interrupt_on_signals():
            machine.start()
    except KeyboardInterrupt:
        machine.stop()  # the boot already tore its machine down; this covers the gap after it
        note("startup aborted; nothing left running")
        return EXIT_FAILURE
    with stop_on_signals(stop):
        info = machine.boot_info
        if info is None:
            raise SmoltestError(f"{machine.name} started without boot information")
        print(machine.get_connection_url(driver=None))
        print(
            f"booted via {info.via} in {info.elapsed_s:.2f}s on {info.target} "
            f"(machine {machine.name}, image {machine.image})"
        )
        sys.stdout.flush()
        note("press Ctrl-C to stop and delete the machine")
        try:
            while not stop.wait(POLL_S):
                pass
        finally:
            note(f"deleting {machine.name}")
            machine.stop()
    note("machine deleted")
    return EXIT_OK


__all__ = [
    "POLL_S",
    "STOP_SIGNALS",
    "interrupt_on_signals",
    "register",
    "run_postgres",
    "stop_on_signals",
]
