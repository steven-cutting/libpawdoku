"""``smoltest cache``: list, prune, clear and export checkpoints.

The commands act on the local store; ``--target cloud`` selects the cloud
checkpoint index instead (which cannot be exported).
"""

from __future__ import annotations

import argparse
import json
import time
from pathlib import Path

from .._log import warn_once
from ..cache import (
    CacheSummary,
    CheckpointBackend,
    CheckpointCache,
    CloudCheckpointIndex,
    PruneReport,
    VariantEntry,
)
from ..config import Settings
from ..errors import CacheError
from ..transport import get_engine
from ..transport.base import Target
from . import (
    EXIT_FAILURE,
    EXIT_OK,
    EXIT_USAGE,
    Context,
    confirm,
    duration_arg,
    format_table,
    human_age,
    human_bytes,
    nonnegative_int,
    print_err,
    short_key,
    size_arg,
)

LS_COLUMNS = ("key", "port", "size", "state", "age", "claim", "image", "seed")
LS_ALIGN = "lrrllll"
USAGE_CODES = frozenset({"NO_KEY", "AMBIGUOUS", "NO_VARIANT"})
"""``CacheError`` codes that mean the command line named something that is not there."""


def register(
    commands: argparse._SubParsersAction[argparse.ArgumentParser],
    common: argparse.ArgumentParser,
) -> None:
    """Add the ``cache`` command and its actions."""
    parser = commands.add_parser(
        "cache",
        parents=[common],
        help="inspect and trim the checkpoint cache",
        description="Acts on the local store; pass --target cloud for the cloud checkpoint index.",
    )
    actions = parser.add_subparsers(dest="action", metavar="ACTION", required=True)

    ls = actions.add_parser("ls", parents=[common], help="list keys and variants")
    ls.add_argument("--json", action="store_true", help="print the listing as JSON")
    ls.set_defaults(func=run_ls)

    prune = actions.add_parser(
        "prune",
        parents=[common],
        help="remove variants by age, count, size or staleness",
        description="Variants claimed by a live process are never removed; dead claims always are.",
    )
    prune.add_argument(
        "--older-than",
        type=duration_arg,
        metavar="AGE",
        help="variants not used for AGE (e.g. 14d, 36h, 90m)",
    )
    prune.add_argument(
        "--keep-latest",
        type=nonnegative_int,
        metavar="N",
        help="keep only the N most recently used variants of each key",
    )
    prune.add_argument(
        "--max-bytes",
        type=size_arg,
        metavar="SIZE",
        help="evict least recently used variants until the cache fits SIZE (e.g. 2G, 500M)",
    )
    prune.add_argument(
        "--stale",
        action="store_true",
        help="variants that cannot restore here (other host or SDK version, corrupt, unfinished)",
    )
    prune.add_argument("-y", "--yes", action="store_true", help="do not ask for confirmation")
    prune.set_defaults(func=run_prune)

    clear = actions.add_parser("clear", parents=[common], help="remove every checkpoint")
    clear.add_argument("-y", "--yes", action="store_true", help="do not ask for confirmation")
    clear.set_defaults(func=run_clear)

    export = actions.add_parser(
        "export",
        parents=[common],
        help="copy a checkpoint out of the cache",
        description="The copy contains the guest's RAM, credentials and data included.",
    )
    export.add_argument(
        "key",
        metavar="KEY[:PORT]",
        help="cache key (a unique prefix is enough), optionally with the variant's host port",
    )
    export.add_argument("output", type=Path, metavar="OUT", help="destination (must not exist)")
    export.set_defaults(func=run_export)


def cache_target(settings: Settings) -> Target:
    """``cloud`` only when asked for explicitly; the local store otherwise."""
    return "cloud" if settings.target == "cloud" else "local"


def local_cache(settings: Settings) -> CheckpointCache:
    """The local store at ``settings.cache_dir``, opened without creating anything."""
    return CheckpointCache(settings.cache_dir, max_bytes=settings.cache_max_bytes)


def open_backend(settings: Settings) -> CheckpointBackend:
    """The local store, or the cloud index for ``--target cloud``."""
    if cache_target(settings) == "cloud":
        return CloudCheckpointIndex.open(settings)
    return local_cache(settings)


def split_variant(text: str) -> tuple[str, int | None]:
    """``KEY[:PORT]`` as ``(key, port)``."""
    prefix, sep, port = text.rpartition(":")
    if sep and port.isdigit():
        return prefix, int(port)
    return text, None


def variant_state(variant: VariantEntry) -> str:
    """``populated``, ``corrupt`` or ``pending`` (reserved, not yet checkpointed)."""
    if variant.corrupt:
        return "corrupt"
    return "populated" if variant.populated else "pending"


def claim_text(variant: VariantEntry) -> str:
    """Who claims the variant, if anyone."""
    if variant.claimed_by is None:
        return "-"
    return f"pid {variant.claimed_by}" + ("" if variant.claim_live else " (dead)")


def render_summary(summary: CacheSummary, now: float | None = None) -> str:
    """A header line plus one row per variant."""
    now = time.time() if now is None else now
    budget = "" if summary.max_bytes is None else f" of {human_bytes(summary.max_bytes)}"
    store = f" (dedup store {human_bytes(summary.store_bytes)})" if summary.store_bytes else ""
    head = (
        f"{summary.root} ({summary.kind}): {len(summary.keys)} keys, "
        f"{summary.variant_count} variants, {human_bytes(summary.total_bytes)}{budget}{store}"
    )
    if not summary.keys:
        return f"{head}\nempty"
    rows: list[tuple[object, ...]] = []
    for key in summary.keys:
        inputs = key.inputs or {}
        image = str(inputs.get("image") or "-")
        seed = short_key(inputs.get("seed_key"))
        for variant in key.variants:
            age = None if variant.last_used_at is None else now - variant.last_used_at
            rows.append(
                (
                    short_key(variant.key),
                    "-" if variant.port is None else variant.port,
                    human_bytes(variant.size_bytes),
                    variant_state(variant),
                    human_age(age),
                    claim_text(variant),
                    image,
                    seed,
                )
            )
    return f"{head}\n{format_table(LS_COLUMNS, rows, align=LS_ALIGN)}"


def render_report(verb: str, report: PruneReport) -> str:
    """What a prune or clear did, one line per removed variant."""
    lines = [
        f"{verb} {len(report.removed)} variant(s), freed {human_bytes(report.freed_bytes)}; "
        f"{len(report.removed_keys)} key(s) emptied, {report.claims_removed} claim file(s) "
        f"removed, {report.store_chunks_pruned} store chunk(s) pruned"
    ]
    lines.extend(
        f"  {short_key(v.key)}:{'-' if v.port is None else v.port}  {human_bytes(v.size_bytes)}"
        for v in report.removed
    )
    return "\n".join(lines)


def run_ls(args: argparse.Namespace, ctx: Context) -> int:
    """List keys and variants, as text or JSON."""
    summary = open_backend(ctx.settings).ls()
    if args.json:
        print(json.dumps(summary.to_dict(), indent=2, sort_keys=True))
    else:
        print(render_summary(summary))
    return EXIT_OK


def run_prune(args: argparse.Namespace, ctx: Context) -> int:
    """Remove variants matching the given criteria (at least one is required)."""
    criteria = (args.older_than, args.keep_latest, args.max_bytes)
    if all(value is None for value in criteria) and not args.stale:
        print_err(
            "error: prune needs at least one of --older-than, --keep-latest, --max-bytes, --stale"
        )
        return EXIT_USAGE
    backend = open_backend(ctx.settings)
    status = confirm(f"prune checkpoints under {ctx.settings.cache_dir}?", assume_yes=args.yes)
    if status != EXIT_OK:
        return status
    report = backend.prune(
        older_than_s=args.older_than,
        keep_latest=args.keep_latest,
        max_bytes=args.max_bytes,
        stale=args.stale,
        engine=get_engine(ctx.settings),
    )
    print(render_report("pruned", report))
    return EXIT_OK


def run_clear(args: argparse.Namespace, ctx: Context) -> int:
    """Remove every key, variant and the dedup store."""
    backend = open_backend(ctx.settings)
    status = confirm(
        f"remove every checkpoint under {ctx.settings.cache_dir}?", assume_yes=args.yes
    )
    if status != EXIT_OK:
        return status
    print(render_report("removed", backend.clear()))
    return EXIT_OK


def run_export(args: argparse.Namespace, ctx: Context) -> int:
    """Copy one variant (default: the most recently used of the key) to ``OUT``."""
    if cache_target(ctx.settings) == "cloud":
        print_err("error: cloud checkpoints live on Smol's side and cannot be exported")
        return EXIT_FAILURE
    cache = local_cache(ctx.settings)
    prefix, port = split_variant(args.key)
    output: Path = args.output
    try:
        key = cache.find_key(prefix)
        # The same key the store uses, so the file is announced exactly once.
        warn_once(
            "cache.export",
            f"{output} will contain a copy of guest RAM, credentials and data included; "
            "keep it private and never commit it",
        )
        written = cache.export(key, port, output, get_engine(ctx.settings))
    except CacheError as exc:
        print_err(f"error: {exc}")
        return EXIT_USAGE if exc.code in USAGE_CODES else EXIT_FAILURE
    which = "latest" if port is None else str(port)
    print(f"exported {key}:{which} to {output} ({human_bytes(written)})")
    return EXIT_OK


__all__ = [
    "LS_ALIGN",
    "LS_COLUMNS",
    "USAGE_CODES",
    "cache_target",
    "claim_text",
    "local_cache",
    "open_backend",
    "register",
    "render_report",
    "render_summary",
    "run_clear",
    "run_export",
    "run_ls",
    "run_prune",
    "split_variant",
    "variant_state",
]
