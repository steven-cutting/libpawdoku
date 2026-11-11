"""``benchmarks/bench_boot.py --fake`` runs in-process on the FakeEngine."""

from __future__ import annotations

import importlib.util
import json
import sys
from pathlib import Path
from types import ModuleType

import pytest

from smoltest.config import Settings
from smoltest.testing import FakeEngine

BENCH_PATH = Path(__file__).resolve().parents[2] / "benchmarks" / "bench_boot.py"
SCENARIOS = ("cold", "restore", "branch")


@pytest.fixture(scope="module")
def bench() -> ModuleType:
    """The benchmark script loaded as a module (it is not part of the package)."""
    spec = importlib.util.spec_from_file_location("bench_boot", BENCH_PATH)
    assert spec is not None and spec.loader is not None
    module = importlib.util.module_from_spec(spec)
    # Registered first: dataclasses resolve postponed annotations through sys.modules.
    sys.modules[spec.name] = module
    try:
        spec.loader.exec_module(module)
    except BaseException:
        sys.modules.pop(spec.name, None)
        raise
    return module


def table_rows(stdout: str) -> dict[str, list[str]]:
    """``scenario -> cells`` of every markdown body row (the first table wins)."""
    rows: dict[str, list[str]] = {}
    for line in stdout.splitlines():
        if not line.startswith("| ") or line.startswith("| scenario"):
            continue
        cells = [cell.strip() for cell in line.strip("|").split("|")]
        rows.setdefault(cells[0], cells)
    return rows


def test_fake_run_prints_all_three_scenarios(
    bench: ModuleType, capsys: pytest.CaptureFixture[str], tmp_path: Path
) -> None:
    out = tmp_path / "bench.json"
    status = bench.main(["--fake", "--rounds", "2", "--json", str(out)])
    assert status == 0
    captured = capsys.readouterr()
    assert "FakeEngine" in captured.out
    rows = table_rows(captured.out)
    assert set(SCENARIOS) <= set(rows)
    for name in SCENARIOS:
        cells = rows[name]
        assert cells[1] == "2"  # n
        assert cells[7] == name  # via: every round booted the way its scenario says
        assert float(cells[2]) <= float(cells[3]) <= float(cells[5])  # min <= median <= max
    assert rows["cold"][6] == "1.0x"
    assert rows["restore"][6].endswith("x") and float(rows["restore"][6][:-1]) > 1.0
    assert rows["branch"][6].endswith("x") and float(rows["branch"][6][:-1]) > 1.0
    assert "restore 1/2" in captured.err and "branch 2/2" in captured.err

    data = json.loads(out.read_text())
    assert data["config"]["fake"] is True
    assert data["config"]["rounds"] == 2
    assert data["config"]["sql_probe"] == "psql (in-guest)"
    assert set(data["scenarios"]) == set(SCENARIOS)
    for name, scenario in data["scenarios"].items():
        assert scenario["stats"]["n"] == 2
        assert [r["via"] for r in scenario["rounds"]] == [name, name]
        for r in scenario["rounds"]:
            assert 0.0 < r["to_tcp_s"] <= r["to_pg_isready_s"] <= r["to_select1_s"]


def test_run_benchmark_releases_every_machine(bench: ModuleType, tmp_path: Path) -> None:
    engine = FakeEngine(latency=bench.FAKE_LATENCY)
    settings = Settings(target="local", cache_dir=tmp_path / "cache")
    report = bench.run_benchmark(bench.BenchConfig(rounds=1, fake=True), engine, settings)
    assert engine.live_machines == set()
    # cold round + population; the restore round and the golden both restore the checkpoint.
    assert len(engine.ops("create")) == 2
    assert len(engine.ops("restore")) == 2
    assert len(engine.ops("checkpoint")) == 1
    assert len(engine.ops("branch")) == 1
    assert [s.vias for s in report.scenarios.values()] == list(SCENARIOS)
    assert all(len(s.rounds) == 1 for s in report.scenarios.values())
    # The fake sleeps per operation, so the medians order like the real ladder.
    medians = {name: s.stats()["median"] for name, s in report.scenarios.items()}
    assert medians["cold"] > medians["restore"] > medians["branch"]
    assert bench.render_table(report).count("\n") >= 7
