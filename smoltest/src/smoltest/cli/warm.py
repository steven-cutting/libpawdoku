"""``smoltest warm postgres``: fill the checkpoint cache ahead of a test run.

Each machine is kept alive until the last one has booted: a live machine holds
its cache variant's claim, so the next boot cannot restore it and cold-boots on
a fresh host port instead, which is what leaves ``K`` distinct port variants
behind for ``K`` concurrent test workers.
"""

from __future__ import annotations

import argparse
from collections.abc import Mapping, Sequence

from ..boot.strategy import BootInfo
from ..cache import CacheSummary, open_checkpoint_backend
from ..config import Settings, resolve_target
from ..errors import SmoltestError
from ..postgres import PostgresMachine, Seed
from ..transport import get_engine
from ..transport.base import Engine
from . import (
    EXIT_FAILURE,
    EXIT_OK,
    Context,
    add_postgres_options,
    format_table,
    human_bytes,
    load_seed,
    note,
    parse_env_assignments,
    positive_int,
    print_err,
    short_key,
)

COLUMNS = ("#", "via", "key", "port", "size", "populate ms", "elapsed s", "seed")
ALIGN = "rllrrrrl"


def register(
    commands: argparse._SubParsersAction[argparse.ArgumentParser],
    common: argparse.ArgumentParser,
) -> None:
    """Add the ``warm`` command."""
    parser = commands.add_parser(
        "warm",
        parents=[common],
        help="populate the checkpoint cache ahead of a test run",
        description="Boot once now so later boots restore a checkpoint instead of booting cold.",
    )
    services = parser.add_subparsers(dest="service", metavar="SERVICE", required=True)
    postgres = services.add_parser(
        "postgres",
        parents=[common],
        help="PostgreSQL",
        description=(
            "Boot K PostgreSQL machines with the cache enabled, apply the seed SQL, "
            "checkpoint each on its own host port and delete them again. Exit 1 when a "
            "requested variant is not in the cache afterwards."
        ),
    )
    add_postgres_options(postgres)
    postgres.add_argument(
        "--cpus", type=positive_int, help="guest vCPUs (default: $SMOLTEST_CPUS, else 1)"
    )
    postgres.add_argument(
        "--memory-mb",
        type=positive_int,
        metavar="MIB",
        help="guest memory in MiB (default: $SMOLTEST_MEMORY_MB, else 512)",
    )
    postgres.add_argument(
        "--variants",
        type=positive_int,
        default=1,
        metavar="K",
        help="port variants to leave in the cache: one per concurrent machine, e.g. pytest -n K",
    )
    postgres.set_defaults(func=run_warm)


def _stopped(machine: PostgresMachine) -> bool:
    """Stop and delete ``machine``; ``False`` (with a warning) when the engine refused."""
    try:
        machine.stop()
    except SmoltestError as exc:
        print_err(f"warning: could not delete {machine.name}: {exc}")
        return False
    return True


def _stop_all(machines: Sequence[PostgresMachine]) -> list[str]:
    """Stop every machine, most recent first; the names of those that could not be deleted."""
    return [
        machine.name or repr(machine) for machine in reversed(machines) if not _stopped(machine)
    ]


def warm_postgres(
    settings: Settings,
    engine: Engine,
    *,
    variants: int,
    seed: Seed | None,
    env: Mapping[str, str],
) -> list[BootInfo]:
    """Boot ``variants`` machines, keeping each alive until the last is up, then stop all.

    A machine that cannot be deleted is an error, raised after every other
    machine was stopped: on the cloud it is still billed, and the caller must not
    report a successful warm. (A boot failure, if one happened, wins.)
    """
    machines: list[PostgresMachine] = []
    infos: list[BootInfo] = []
    booted = False
    try:
        for index in range(1, variants + 1):
            machine = PostgresMachine(settings=settings, engine=engine, seed=seed)
            for key, value in env.items():
                machine.with_env(key, value)
            machine.start()
            machines.append(machine)
            info = machine.boot_info
            if info is None:
                raise SmoltestError(f"{machine.name} started without boot information")
            infos.append(info)
            port = "-" if info.variant_port is None else info.variant_port
            note(f"[{index}/{variants}] {info.via} on port {port} in {info.elapsed_s:.2f}s")
        booted = True
    finally:
        failed = _stop_all(machines)
    if booted and failed:
        raise SmoltestError(
            f"could not delete {len(failed)} machine(s): {', '.join(failed)}; they may still be "
            "running (and, on the cloud, billed)",
            code="DELETE_FAILED",
        )
    return infos


def _seed_text(info: BootInfo) -> str:
    if info.seeded_key is None:
        return "-"
    return "applied" if info.seeded else "cached"


def table_rows(infos: Sequence[BootInfo], summary: CacheSummary) -> list[tuple[object, ...]]:
    """One row per boot: the variant it used or wrote, with the sizes the cache reports."""
    sizes = {(v.key, v.port): v.size_bytes for key in summary.keys for v in key.variants}
    rows: list[tuple[object, ...]] = []
    for index, info in enumerate(infos, 1):
        keys = [key for key in (info.cache_key, info.seeded_key) if key is not None]
        size = sum(sizes.get((key, info.variant_port), 0) for key in keys)
        populate_ms = sum(info.timings.get(stage, 0.0) for stage in ("populate", "populate_seed"))
        rows.append(
            (
                index,
                info.via,
                short_key(info.cache_key),
                "-" if info.variant_port is None else info.variant_port,
                human_bytes(size),
                f"{populate_ms * 1000:.0f}" if info.populated else "-",
                f"{info.elapsed_s:.2f}",
                _seed_text(info),
            )
        )
    return rows


def missing_variants(
    infos: Sequence[BootInfo], summary: CacheSummary, *, seeded: bool
) -> list[tuple[str | None, int | None]]:
    """``(key, port)`` pairs a later identical boot would restore but the cache lacks.

    For each boot, the key that boot path restores first (the seeded key when a
    seed was given, else the base key) must have a populated, intact variant on
    the boot's port. A boot that ran uncached (the key lock was busy, or the
    capture failed and the ladder continued) has no entry there and is missing
    as well; a boot without a key (the cache disabled) always is.
    """
    entries = {(v.key, v.port): v for key in summary.keys for v in key.variants}
    missing: list[tuple[str | None, int | None]] = []
    for info in infos:
        key = info.seeded_key if seeded else info.cache_key
        entry = entries.get((key, info.variant_port)) if key is not None else None
        if entry is None or not entry.populated or entry.corrupt:
            missing.append((key, info.variant_port))
    return missing


def run_warm(args: argparse.Namespace, ctx: Context) -> int:
    """Boot ``--variants`` machines with the cache on, print what the cache holds, verify it.

    Exit 0 only when every requested variant is in the cache, populated and
    intact; a boot the ladder finished uncached is reported and makes this 1.
    """
    settings = ctx.settings.replace(
        postgres_image=args.image,
        cpus=args.cpus,
        memory_mb=args.memory_mb,
        fast_mode=False if args.no_fast else None,
        disable_cache=False,
    )
    env = parse_env_assignments(args.env)
    seed = load_seed(args.seed_sql)
    engine = get_engine(settings)
    target = resolve_target(settings, engine)
    if not engine.supports_checkpoints(target):
        print_err(f"error: the engine cannot checkpoint on {target}; there is nothing to warm")
        return EXIT_FAILURE
    variants: int = args.variants
    if target == "cloud" and variants > 1:
        note("cloud checkpoints restore on fresh ports and need no port variants; booting once")
        variants = 1
    note(
        f"warming {variants} variant(s) of {settings.postgres_image} on {target} "
        f"into {settings.cache_dir}"
    )
    infos = warm_postgres(settings, engine, variants=variants, seed=seed, env=env)
    summary = open_checkpoint_backend(settings, target).ls()  # one scan for table and verdict
    print(format_table(COLUMNS, table_rows(infos, summary), align=ALIGN))
    missing = missing_variants(infos, summary, seeded=seed is not None)
    if missing:
        for key, port in missing:
            where = "" if port is None else f" on port {port}"
            print_err(f"error: no populated checkpoint for {short_key(key)}{where}")
        print_err(
            f"error: {len(missing)} of {len(infos)} requested variant(s) are not in the cache; "
            "the warnings above say why (a busy key lock or a failed checkpoint capture), "
            "rerun `smoltest warm` once other boots have finished"
        )
        return EXIT_FAILURE
    return EXIT_OK


__all__ = [
    "ALIGN",
    "COLUMNS",
    "missing_variants",
    "register",
    "run_warm",
    "table_rows",
    "warm_postgres",
]
