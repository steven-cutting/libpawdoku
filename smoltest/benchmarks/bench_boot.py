#!/usr/bin/env python3
"""Boot-time benchmark: cold boot vs checkpoint restore vs branch.

Each scenario is timed from the start of a round until ``SELECT 1`` succeeds,
with the intermediate marks "TCP accepts", "``pg_isready`` passes" and
"``SELECT 1`` answers" (through a host driver when one is installed, else
in-guest ``psql``). The result is a markdown table with the speedup of each
scenario over the cold median, and optionally a JSON file.

Scenarios:

- ``cold``: the cache is disabled; every round creates a machine on a fresh port.
- ``restore``: one boot populates the checkpoint cache, then every round
  restores it; the machine is deleted between rounds.
- ``branch``: one golden machine, then every round branches it; the child is
  deleted between rounds. ``branch_from`` already waits for ``pg_isready``, so
  the TCP and ``pg_isready`` marks of a branch coincide with its return.

``--fake`` runs the whole script on :class:`smoltest.testing.FakeEngine` with
small latencies, which is how CI checks the script without KVM.

Usage::

    python benchmarks/bench_boot.py --rounds 5 [--image postgres:16] [--cpus 1]
        [--memory-mb 512] [--cache-dir DIR] [--json out.json] [--fake]
"""

from __future__ import annotations

import argparse
import contextlib
import json
import statistics
import sys
import tempfile
import time
import uuid
from collections.abc import Callable, Iterator, Sequence
from dataclasses import dataclass, field
from pathlib import Path
from typing import Any

from smoltest.boot.spec import PostgresSpec
from smoltest.boot.strategy import BootResult, boot_postgres, branch_from
from smoltest.config import Settings
from smoltest.errors import SmoltestError, TargetUnavailable
from smoltest.testing import FakeEngine
from smoltest.transport import get_engine
from smoltest.transport.base import Engine
from smoltest.wait.strategies import (
    DRIVERS,
    PgIsReadyWaitStrategy,
    PortWaitStrategy,
    ReadinessView,
    SqlWaitStrategy,
)

EXIT_OK = 0
EXIT_FAILURE = 1
EXIT_TARGET_UNAVAILABLE = 3

SCENARIOS: tuple[str, ...] = ("cold", "restore", "branch")
FAKE_LATENCY: dict[str, float] = {"create": 0.3, "restore": 0.05, "branch": 0.02}
"""Per-operation sleeps of the fake engine: scaled down so CI stays fast."""

DEFAULT_ROUNDS = 5


@dataclass(frozen=True)
class BenchConfig:
    """What to measure; ``None`` means "the settings' default"."""

    rounds: int = DEFAULT_ROUNDS
    image: str | None = None
    cpus: int | None = None
    memory_mb: int | None = None
    fake: bool = False
    cache_dir: Path | None = None
    json_out: Path | None = None


@dataclass(frozen=True)
class Round:
    """One timed boot: seconds from the start of the round to each mark."""

    via: str
    to_tcp_s: float
    to_pg_isready_s: float
    to_select1_s: float
    stages: dict[str, float] = field(default_factory=dict)

    def to_dict(self) -> dict[str, Any]:
        return {
            "via": self.via,
            "to_tcp_s": self.to_tcp_s,
            "to_pg_isready_s": self.to_pg_isready_s,
            "to_select1_s": self.to_select1_s,
            "stages": dict(self.stages),
        }


@dataclass
class ScenarioResult:
    """The rounds of one scenario and their summary statistics."""

    name: str
    rounds: list[Round] = field(default_factory=list)

    @property
    def totals(self) -> list[float]:
        """Time to ``SELECT 1`` of every round."""
        return [r.to_select1_s for r in self.rounds]

    def stats(self) -> dict[str, float]:
        """``n``, ``min``, ``median``, ``mean`` and ``max`` of :attr:`totals`."""
        totals = self.totals
        if not totals:
            return {"n": 0, "min": 0.0, "median": 0.0, "mean": 0.0, "max": 0.0}
        return {
            "n": len(totals),
            "min": min(totals),
            "median": statistics.median(totals),
            "mean": statistics.fmean(totals),
            "max": max(totals),
        }

    def median_of(self, mark: str) -> float:
        """Median of one mark (``to_tcp_s``, ``to_pg_isready_s`` or ``to_select1_s``)."""
        values = [float(getattr(r, mark)) for r in self.rounds]
        return statistics.median(values) if values else 0.0

    @property
    def vias(self) -> str:
        """How the rounds actually booted (``restore`` may fall back to ``cold``)."""
        return ",".join(sorted({r.via for r in self.rounds})) or "-"

    def to_dict(self) -> dict[str, Any]:
        return {
            "name": self.name,
            "stats": self.stats(),
            "rounds": [r.to_dict() for r in self.rounds],
        }


@dataclass
class Report:
    """Everything one benchmark run produced."""

    config: dict[str, Any]
    scenarios: dict[str, ScenarioResult] = field(default_factory=dict)

    def to_dict(self) -> dict[str, Any]:
        return {
            "config": dict(self.config),
            "scenarios": {name: s.to_dict() for name, s in self.scenarios.items()},
        }


class Prober:
    """Records when a booted machine answers TCP, ``pg_isready`` and ``SELECT 1``."""

    def __init__(self, spec: PostgresSpec, settings: Settings, *, in_guest_sql: bool) -> None:
        self._spec = spec
        self._settings = settings
        self._tcp = PortWaitStrategy()
        self._pg_isready = PgIsReadyWaitStrategy(spec.username, port=spec.guest_port)
        prefer = ("psql",) if in_guest_sql else DRIVERS
        self._sql = SqlWaitStrategy("SELECT 1", prefer=prefer)

    @property
    def sql_probe(self) -> str:
        """Which driver answers the ``SELECT 1`` mark."""
        driver = self._sql.select_driver()
        return "psql (in-guest)" if driver is None else driver[0]

    def measure(self, result: BootResult, started: float) -> Round:
        """Wait for each mark in turn and return the elapsed times since ``started``."""
        spec = self._spec
        view = ReadinessView(
            handle=result.handle,
            endpoint=result.endpoint,
            username=spec.username,
            password=spec.password,
            dbname=spec.dbname,
            guest_port=spec.guest_port,
        )
        timeout, poll = self._settings.ready_timeout_s, self._settings.poll_interval_s
        self._tcp.wait_until_ready(view, timeout, poll)
        to_tcp = time.perf_counter() - started
        self._pg_isready.wait_until_ready(view, timeout, poll)
        to_pg = time.perf_counter() - started
        self._sql.wait_until_ready(view, timeout, poll)
        to_sql = time.perf_counter() - started
        return Round(result.info.via, to_tcp, to_pg, to_sql, dict(result.info.timings))


def _name(scenario: str) -> str:
    return f"smoltest-bench-{scenario}-{uuid.uuid4().hex[:6]}"


def _progress(scenario: str, index: int, rounds: int, measured: Round) -> None:
    print(
        f"{scenario} {index}/{rounds}: {measured.to_select1_s:.3f}s to SELECT 1 "
        f"(tcp {measured.to_tcp_s:.3f}s, pg_isready {measured.to_pg_isready_s:.3f}s) "
        f"via {measured.via}",
        file=sys.stderr,
        flush=True,
    )


def _timed_round(
    scenario: ScenarioResult,
    index: int,
    rounds: int,
    prober: Prober,
    boot: Callable[[], BootResult],
) -> None:
    """Run ``boot()``, measure the marks, release the machine, record the round."""
    started = time.perf_counter()
    result = boot()
    try:
        measured = prober.measure(result, started)
    finally:
        result.release()
    scenario.rounds.append(measured)
    _progress(scenario.name, index, rounds, measured)


def run_cold(
    config: BenchConfig, spec: PostgresSpec, settings: Settings, engine: Engine, prober: Prober
) -> ScenarioResult:
    """Cold boots with the cache disabled; a fresh host port each round."""
    scenario = ScenarioResult("cold")
    cold_settings = settings.replace(disable_cache=True)
    for index in range(1, config.rounds + 1):
        _timed_round(
            scenario,
            index,
            config.rounds,
            prober,
            lambda: boot_postgres(
                spec, cold_settings, engine, wait=PortWaitStrategy(), name=_name("cold")
            ),
        )
    return scenario


def run_restore(
    config: BenchConfig, spec: PostgresSpec, settings: Settings, engine: Engine, prober: Prober
) -> ScenarioResult:
    """One population (not timed), then ``rounds`` restores with a delete in between."""
    scenario = ScenarioResult("restore")
    populate = boot_postgres(spec, settings, engine, name=_name("populate"))
    print(
        f"restore: populated the cache via {populate.info.via} in {populate.info.elapsed_s:.3f}s",
        file=sys.stderr,
        flush=True,
    )
    populate.release()
    for index in range(1, config.rounds + 1):
        _timed_round(
            scenario,
            index,
            config.rounds,
            prober,
            lambda: boot_postgres(
                spec, settings, engine, wait=PortWaitStrategy(), name=_name("restore")
            ),
        )
    return scenario


def run_branch(
    config: BenchConfig, spec: PostgresSpec, settings: Settings, engine: Engine, prober: Prober
) -> ScenarioResult:
    """One golden (not timed), then ``rounds`` branches with a delete in between."""
    scenario = ScenarioResult("branch")
    golden = boot_postgres(spec, settings, engine, role="golden", name=_name("golden"))
    print(
        f"branch: golden booted via {golden.info.via} in {golden.info.elapsed_s:.3f}s",
        file=sys.stderr,
        flush=True,
    )
    try:
        for index in range(1, config.rounds + 1):
            _timed_round(
                scenario,
                index,
                config.rounds,
                prober,
                lambda: branch_from(golden, _name("branch"), settings, engine),
            )
    finally:
        golden.release()
    return scenario


def make_settings(config: BenchConfig, cache_dir: Path) -> Settings:
    """Settings for the run: the environment's (real) or plain defaults (fake)."""
    base = Settings(target="local") if config.fake else Settings.from_env()
    return base.replace(
        cache_dir=cache_dir,
        postgres_image=config.image,
        cpus=config.cpus,
        memory_mb=config.memory_mb,
    )


def make_engine(config: BenchConfig, settings: Settings) -> Engine:
    """The fake engine with :data:`FAKE_LATENCY`, or the engine the settings select."""
    if config.fake:
        return FakeEngine(latency=FAKE_LATENCY)
    return get_engine(settings)


def run_benchmark(config: BenchConfig, engine: Engine, settings: Settings) -> Report:
    """Run the three scenarios on ``engine`` and return the :class:`Report`."""
    spec = PostgresSpec()
    prober = Prober(spec, settings, in_guest_sql=config.fake)
    report = Report(
        config={
            "rounds": config.rounds,
            "image": spec.resolved_image(settings),
            "cpus": settings.cpus,
            "memory_mb": settings.memory_mb,
            "fast_mode": settings.fast_mode,
            "fake": config.fake,
            "cache_dir": str(settings.cache_dir),
            "sdk_version": engine.sdk_version(),
            "sql_probe": prober.sql_probe,
        }
    )
    report.scenarios["cold"] = run_cold(config, spec, settings, engine, prober)
    report.scenarios["restore"] = run_restore(config, spec, settings, engine, prober)
    report.scenarios["branch"] = run_branch(config, spec, settings, engine, prober)
    return report


def _speedup(cold_median: float, median: float) -> str:
    if median <= 0.0 or cold_median <= 0.0:
        return "n/a"
    return f"{cold_median / median:.1f}x"


def render_table(report: Report) -> str:
    """The markdown tables: totals with speedup, then the median marks."""
    cold = report.scenarios.get("cold")
    cold_median = cold.stats()["median"] if cold is not None else 0.0
    lines = [
        "| scenario | n | min s | median s | mean s | max s | vs cold | via |",
        "|---|---:|---:|---:|---:|---:|---:|---|",
    ]
    for name in SCENARIOS:
        scenario = report.scenarios.get(name)
        if scenario is None:
            continue
        s = scenario.stats()
        lines.append(
            f"| {name} | {int(s['n'])} | {s['min']:.3f} | {s['median']:.3f} | {s['mean']:.3f} "
            f"| {s['max']:.3f} | {_speedup(cold_median, s['median'])} | {scenario.vias} |"
        )
    lines.append("")
    lines.append(
        "| scenario | to TCP (median s) | to pg_isready (median s) | to SELECT 1 (median s) |"
    )
    lines.append("|---|---:|---:|---:|")
    for name in SCENARIOS:
        scenario = report.scenarios.get(name)
        if scenario is None:
            continue
        lines.append(
            f"| {name} | {scenario.median_of('to_tcp_s'):.3f} "
            f"| {scenario.median_of('to_pg_isready_s'):.3f} "
            f"| {scenario.median_of('to_select1_s'):.3f} |"
        )
    return "\n".join(lines)


def render_header(report: Report) -> str:
    c = report.config
    engine = "FakeEngine" if c["fake"] else f"Smol SDK {c['sdk_version']}"
    return (
        f"smoltest boot benchmark: {c['rounds']} rounds, image {c['image']}, "
        f"{c['cpus']} vCPU, {c['memory_mb']} MiB, fast_mode={c['fast_mode']}, "
        f"engine {engine}, SELECT 1 via {c['sql_probe']}"
    )


def _positive_int(text: str) -> int:
    try:
        value = int(text)
    except ValueError:
        raise argparse.ArgumentTypeError(f"expected an integer, got {text!r}") from None
    if value < 1:
        raise argparse.ArgumentTypeError(f"expected an integer >= 1, got {text!r}")
    return value


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(
        prog="bench_boot",
        description="Time cold boots, checkpoint restores and branches of PostgreSQL machines.",
    )
    parser.add_argument(
        "--rounds",
        type=_positive_int,
        default=DEFAULT_ROUNDS,
        metavar="N",
        help=f"timed boots per scenario (default {DEFAULT_ROUNDS})",
    )
    parser.add_argument("--image", metavar="IMAGE", help="PostgreSQL image (default postgres:16)")
    parser.add_argument("--cpus", type=_positive_int, metavar="N", help="guest vCPUs (default 1)")
    parser.add_argument(
        "--memory-mb", type=_positive_int, metavar="MIB", help="guest memory in MiB (default 512)"
    )
    parser.add_argument(
        "--cache-dir",
        type=Path,
        metavar="DIR",
        help="checkpoint cache to use (default: a temporary directory removed afterwards)",
    )
    parser.add_argument("--json", type=Path, metavar="OUT", help="also write the report as JSON")
    parser.add_argument(
        "--fake",
        action="store_true",
        help="run on smoltest.testing.FakeEngine with small latencies (no KVM needed)",
    )
    return parser


def parse_args(argv: Sequence[str] | None = None) -> BenchConfig:
    args = build_parser().parse_args(argv)
    return BenchConfig(
        rounds=args.rounds,
        image=args.image,
        cpus=args.cpus,
        memory_mb=args.memory_mb,
        fake=args.fake,
        cache_dir=args.cache_dir,
        json_out=args.json,
    )


@contextlib.contextmanager
def _cache_dir(config: BenchConfig) -> Iterator[Path]:
    if config.cache_dir is not None:
        yield config.cache_dir
        return
    with tempfile.TemporaryDirectory(prefix="smoltest-bench-") as tmp:
        yield Path(tmp)


def main(argv: Sequence[str] | None = None) -> int:
    """Run the benchmark and print the tables; returns the exit status."""
    config = parse_args(argv)
    try:
        with _cache_dir(config) as cache_dir:
            settings = make_settings(config, cache_dir)
            engine = make_engine(config, settings)
            report = run_benchmark(config, engine, settings)
    except TargetUnavailable as exc:
        print(f"bench_boot: {exc}", file=sys.stderr)
        return EXIT_TARGET_UNAVAILABLE
    except SmoltestError as exc:
        print(f"bench_boot: error: {exc}", file=sys.stderr)
        return EXIT_FAILURE
    print(render_header(report))
    print()
    print(render_table(report))
    if config.json_out is not None:
        config.json_out.write_text(json.dumps(report.to_dict(), indent=2, sort_keys=True) + "\n")
        print(f"wrote {config.json_out}", file=sys.stderr)
    return EXIT_OK


if __name__ == "__main__":
    sys.exit(main())
