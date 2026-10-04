"""The ``smoltest`` CLI against the FakeEngine: exit codes, output shapes, cleanup."""

from __future__ import annotations

import importlib.util
import io
import json
import os
import signal
import sys
import threading
import time
from collections.abc import Callable, Iterator
from pathlib import Path
from typing import Any

import pytest

from smoltest._log import logger, reset_warn_once
from smoltest._ports import pick_free_port
from smoltest.boot.spec import PostgresSpec
from smoltest.boot.strategy import boot_postgres
from smoltest.cache import CheckpointCache
from smoltest.cache.store import pid_alive
from smoltest.cli import (
    EXIT_FAILURE,
    EXIT_OK,
    EXIT_TARGET_UNAVAILABLE,
    EXIT_USAGE,
    format_table,
    human_age,
    human_bytes,
    parse_duration,
    parse_env_assignments,
    parse_size,
)
from smoltest.cli.main import main
from smoltest.config import Settings
from smoltest.errors import BootError, InvalidConfig
from smoltest.testing import FakeEngine, FakeMachine

UNAVAILABLE = (False, "KVM_UNAVAILABLE", "no /dev/kvm")
DEAD_PID = 2**22 - 1
WEBSOCKETS_INSTALLED = importlib.util.find_spec("websockets") is not None
SEED_OPTIONS = (
    "--env",
    "FOO=bar",
    "BAZ=1",
    "--image",
    "postgres:17-alpine",
    "--no-fast",
    "--cpus",
    "2",
    "--memory-mb",
    "1024",
)


@pytest.fixture(autouse=True)
def _fresh_warnings() -> Iterator[None]:
    reset_warn_once()
    yield
    reset_warn_once()


@pytest.fixture
def cache_dir(tmp_path: Path) -> Path:
    """Where the autouse conftest fixture points ``SMOLTEST_CACHE_DIR``."""
    return tmp_path / "cache"


def run(*argv: str, stop_event: threading.Event | None = None) -> int:
    return main(list(argv), stop_event=stop_event)


def warm(variants: int, *extra: str) -> int:
    return run("warm", "postgres", "--variants", str(variants), *extra)


def only_machine(engine: FakeEngine) -> FakeMachine:
    assert len(engine.machines) == 1
    return next(iter(engine.machines.values()))


def when_live(engine: FakeEngine, action: Callable[[], None]) -> threading.Thread:
    """Run ``action`` from another thread once the engine has a live machine."""

    def wait_then_act() -> None:
        deadline = time.monotonic() + 20
        while not engine.live_machines and time.monotonic() < deadline:
            time.sleep(0.01)
        action()

    thread = threading.Thread(target=wait_then_act, daemon=True)
    thread.start()
    return thread


# -- usage ------------------------------------------------------------------------------


def test_no_command_is_a_usage_error(capsys: pytest.CaptureFixture[str]) -> None:
    assert run() == EXIT_USAGE
    assert "usage:" in capsys.readouterr().err


@pytest.mark.parametrize(
    "argv",
    [
        ("bogus",),
        ("cache",),
        ("warm",),
        ("run", "mysql"),
        ("cache", "prune", "--older-than", "soon", "--yes"),
        ("cache", "prune", "--max-bytes", "lots", "--yes"),
        ("warm", "postgres", "--variants", "0"),
        ("run", "postgres", "--port", "70000"),
        ("--target", "mars", "doctor"),
    ],
)
def test_bad_command_lines_exit_2(argv: tuple[str, ...]) -> None:
    assert run(*argv) == EXIT_USAGE


def test_help_and_version(capsys: pytest.CaptureFixture[str]) -> None:
    assert run("--help") == EXIT_OK
    out = capsys.readouterr().out
    assert "doctor" in out and "warm" in out and "cache" in out and "run" in out
    assert run("--version") == EXIT_OK
    assert capsys.readouterr().out.startswith("smoltest ")
    assert run("cache", "prune", "--help") == EXIT_OK
    assert "--keep-latest" in capsys.readouterr().out


def test_malformed_env_and_missing_seed_file_exit_2(
    fake_engine: FakeEngine, tmp_path: Path, capsys: pytest.CaptureFixture[str]
) -> None:
    assert run("warm", "postgres", "--env", "NOVALUE") == EXIT_USAGE
    assert "KEY=VALUE" in capsys.readouterr().err
    assert run("run", "postgres", "--seed-sql", str(tmp_path / "missing.sql")) == EXIT_USAGE
    assert "seed file" in capsys.readouterr().err
    assert fake_engine.ops("create") == []


def test_bad_environment_setting_exits_2(
    monkeypatch: pytest.MonkeyPatch, capsys: pytest.CaptureFixture[str]
) -> None:
    monkeypatch.setenv("SMOLTEST_CPUS", "lots")
    assert run("doctor") == EXIT_USAGE
    assert "SMOLTEST_CPUS" in capsys.readouterr().err


def test_logging_handler_is_removed_after_each_call(fake_engine: FakeEngine) -> None:
    before = list(logger.handlers)
    assert run("doctor") == EXIT_OK
    assert logger.handlers == before


# -- helpers ----------------------------------------------------------------------------


@pytest.mark.parametrize(
    ("text", "seconds"),
    [
        ("14d", 14 * 86400),
        ("36h", 36 * 3600),
        ("90m", 5400),
        ("45s", 45),
        ("120", 120),
        ("2w", 1209600),
    ],
)
def test_parse_duration(text: str, seconds: float) -> None:
    assert parse_duration(text) == seconds


@pytest.mark.parametrize(
    ("text", "size"),
    [
        ("2G", 2 * 2**30),
        ("500M", 500 * 2**20),
        ("10GiB", 10 * 2**30),
        ("64KB", 65536),
        ("1024", 1024),
    ],
)
def test_parse_size(text: str, size: int) -> None:
    assert parse_size(text) == size


@pytest.mark.parametrize("bad", ["", "soon", "-1d", "1y"])
def test_parse_rejects_garbage(bad: str) -> None:
    with pytest.raises(InvalidConfig):
        parse_duration(bad)
    with pytest.raises(InvalidConfig):
        parse_size(bad)


def test_parse_env_assignments() -> None:
    assert parse_env_assignments(["A=1", "B=x=y", "A=2", "EMPTY="]) == {
        "A": "2",
        "B": "x=y",
        "EMPTY": "",
    }
    with pytest.raises(InvalidConfig):
        parse_env_assignments(["=novalue"])


def test_formatting_helpers() -> None:
    assert human_bytes(None) == "-" and human_bytes(512) == "512 B"
    assert human_bytes(1536) == "1.5 KiB" and human_bytes(10 * 2**30) == "10.0 GiB"
    assert human_bytes(3 * 2**40) == "3.0 TiB"
    assert human_age(None) == "-" and human_age(45) == "45s" and human_age(200) == "3m"
    assert human_age(7200) == "2h" and human_age(3 * 86400) == "3d"
    table = format_table(("a", "bb"), [(1, "x"), (22, "yyy")], align="rl")
    assert table.splitlines() == [" a  bb", " 1  x", "22  yyy"]


# -- doctor -----------------------------------------------------------------------------


def test_doctor_reports_a_usable_local_target(
    fake_engine: FakeEngine, cache_dir: Path, capsys: pytest.CaptureFixture[str]
) -> None:
    assert run("doctor") == EXIT_OK
    out = capsys.readouterr().out
    assert "target: auto -> local" in out
    assert "local engine: available" in out
    assert "ok: machines can boot on local" in out
    assert "SMOL_CLOUD_TOKEN absent" in out
    assert not cache_dir.exists(), "doctor must not create the cache directory"


def test_doctor_json_shape(fake_engine: FakeEngine, capsys: pytest.CaptureFixture[str]) -> None:
    assert run("doctor", "--json") == EXIT_OK
    data = json.loads(capsys.readouterr().out)
    assert set(data) == {
        "ok",
        "smoltest",
        "smolmachines",
        "platform",
        "local",
        "cloud",
        "target",
        "cache",
        "problems",
    }
    assert data["ok"] is True and data["problems"] == []
    assert data["target"] == {"requested": "auto", "resolved": "local", "error": None}
    assert data["local"]["available"] is True and isinstance(data["local"]["kvm_device"], bool)
    assert data["smolmachines"]["sdk_version"] == "fake-1.22.2"
    assert data["smolmachines"]["engine"] == "smoltest.testing.FakeEngine"
    assert data["smoltest"]["version"]
    assert data["platform"]["python"] and data["platform"]["machine"]
    assert data["cloud"]["token_present"] is False
    assert data["cache"]["keys"] == 0 and data["cache"]["exists"] is False


def test_doctor_without_any_target_exits_1(
    fake_engine: FakeEngine, capsys: pytest.CaptureFixture[str]
) -> None:
    fake_engine.availability = UNAVAILABLE
    assert run("doctor") == EXIT_FAILURE
    out = capsys.readouterr().out
    assert "target: auto -> none" in out and "KVM_UNAVAILABLE" in out and "problems:" in out
    assert run("doctor", "--json") == EXIT_FAILURE
    data = json.loads(capsys.readouterr().out)
    assert data["ok"] is False and data["target"]["resolved"] is None
    assert data["target"]["error"] == {"code": "KVM_UNAVAILABLE", "reason": "no /dev/kvm"}
    assert any("KVM_UNAVAILABLE" in problem for problem in data["problems"])


def test_doctor_detects_the_cloud_token_but_never_prints_it(
    fake_engine: FakeEngine, monkeypatch: pytest.MonkeyPatch, capsys: pytest.CaptureFixture[str]
) -> None:
    fake_engine.availability = UNAVAILABLE
    monkeypatch.setenv("SMOL_CLOUD_TOKEN", "sekrit-token-value")
    expected = EXIT_OK if WEBSOCKETS_INSTALLED else EXIT_FAILURE
    assert run("doctor", "--json") == expected
    out = capsys.readouterr().out
    assert "sekrit" not in out
    data = json.loads(out)
    assert data["cloud"]["token_present"] is True
    assert data["cloud"]["websockets"] is WEBSOCKETS_INSTALLED
    assert data["target"]["resolved"] == "cloud"
    assert run("doctor") == expected
    assert "sekrit" not in capsys.readouterr().out


def test_doctor_explicit_local_target_that_is_unavailable(
    fake_engine: FakeEngine, capsys: pytest.CaptureFixture[str]
) -> None:
    fake_engine.availability = UNAVAILABLE
    assert run("--target", "local", "doctor", "--json") == EXIT_FAILURE
    data = json.loads(capsys.readouterr().out)
    assert data["target"]["resolved"] == "local" and data["ok"] is False
    assert data["problems"] == ["local engine unavailable (KVM_UNAVAILABLE: no /dev/kvm)"]


def test_doctor_counts_cache_contents(
    fake_engine: FakeEngine, cache_dir: Path, capsys: pytest.CaptureFixture[str]
) -> None:
    assert warm(2) == EXIT_OK
    capsys.readouterr()
    assert run("doctor", "--json") == EXIT_OK
    cache = json.loads(capsys.readouterr().out)["cache"]
    assert cache["exists"] is True and cache["keys"] == 1 and cache["variants"] == 2
    assert cache["populated"] == 2 and cache["live_claims"] == 0 and cache["stale_claims"] == 0
    assert cache["bytes"] > 0
    # A claim file of a dead process counts as stale.
    assert not pid_alive(DEAD_PID)
    store = CheckpointCache(cache_dir)
    key = store.keys()[0]
    port = store.variants(key)[0].port
    assert port is not None
    store.paths(key, port).claim.write_text(json.dumps({"pid": DEAD_PID, "token": "x"}))
    assert run("doctor", "--json") == EXIT_OK
    cache = json.loads(capsys.readouterr().out)["cache"]
    assert cache["stale_claims"] == 1 and cache["live_claims"] == 0


# -- warm -------------------------------------------------------------------------------


def test_warm_two_variants_on_distinct_ports(
    fake_engine: FakeEngine, cache_dir: Path, capsys: pytest.CaptureFixture[str]
) -> None:
    assert warm(2) == EXIT_OK
    captured = capsys.readouterr()
    assert len(fake_engine.ops("create")) == 2
    assert len(fake_engine.ops("checkpoint")) == 2
    assert fake_engine.live_machines == set()
    summary = CheckpointCache(cache_dir).ls()
    assert len(summary.keys) == 1
    variants = summary.keys[0].variants
    assert len(variants) == 2
    ports = {v.port for v in variants}
    assert len(ports) == 2 and all(v.populated and v.claimed_by is None for v in variants)
    lines = captured.out.splitlines()
    assert lines[0].split() == [
        "#",
        "via",
        "key",
        "port",
        "size",
        "populate",
        "ms",
        "elapsed",
        "s",
        "seed",
    ]
    assert len(lines) == 3
    assert all("cold" in line for line in lines[1:])
    assert {int(line.split()[3]) for line in lines[1:]} == ports
    assert all(line.split()[-1] == "-" for line in lines[1:])
    assert "[1/2] cold" in captured.err and "[2/2] cold" in captured.err


def test_warm_restores_an_existing_variant_and_adds_one(
    fake_engine: FakeEngine, cache_dir: Path, capsys: pytest.CaptureFixture[str]
) -> None:
    assert warm(1) == EXIT_OK
    capsys.readouterr()
    assert warm(2) == EXIT_OK
    out = capsys.readouterr().out
    assert "restore" in out and "cold" in out
    assert len(fake_engine.ops("create")) == 2 and len(fake_engine.ops("restore")) == 1
    assert CheckpointCache(cache_dir).ls().variant_count == 2
    assert fake_engine.live_machines == set()


def test_warm_applies_seed_env_and_options(
    fake_engine: FakeEngine, cache_dir: Path, tmp_path: Path, capsys: pytest.CaptureFixture[str]
) -> None:
    seed_file = tmp_path / "schema.sql"
    seed_file.write_text("CREATE TABLE t (id int);\nINSERT INTO t VALUES (1);\n")
    assert warm(1, "--seed-sql", str(seed_file), *SEED_OPTIONS) == EXIT_OK
    out = capsys.readouterr().out
    machine = only_machine(fake_engine)
    assert machine.env["FOO"] == "bar" and machine.env["BAZ"] == "1"
    assert machine.spec.image == "postgres:17-alpine"
    assert machine.spec.cpus == 2 and machine.spec.memory_mb == 1024
    assert "fsync=off" not in machine.argv
    assert any("CREATE TABLE t" in sql for sql in machine.sql_log)
    summary = CheckpointCache(cache_dir).ls()
    assert len(summary.keys) == 2  # the base key and the seeded key
    assert len(fake_engine.ops("checkpoint")) == 2
    assert out.splitlines()[1].split()[-1] == "applied"
    # A second warm restores the seeded checkpoint and applies nothing.
    capsys.readouterr()
    assert warm(1, "--seed-sql", str(seed_file), *SEED_OPTIONS) == EXIT_OK
    assert capsys.readouterr().out.splitlines()[1].split()[-1] == "cached"
    assert len(fake_engine.ops("checkpoint")) == 2


def test_warm_without_checkpoint_support_exits_1(
    fake_engine: FakeEngine, monkeypatch: pytest.MonkeyPatch, capsys: pytest.CaptureFixture[str]
) -> None:
    monkeypatch.setattr(fake_engine, "supports_checkpoints", lambda target: False)
    assert warm(1) == EXIT_FAILURE
    assert "nothing to warm" in capsys.readouterr().err
    assert fake_engine.ops("create") == []


def test_warm_without_a_target_exits_3(
    fake_engine: FakeEngine, capsys: pytest.CaptureFixture[str]
) -> None:
    fake_engine.availability = UNAVAILABLE
    assert warm(1) == EXIT_TARGET_UNAVAILABLE
    err = capsys.readouterr().err
    assert "KVM_UNAVAILABLE" in err and "smoltest doctor" in err
    assert fake_engine.ops("create") == []


def test_warm_on_the_cloud_boots_once(
    fake_engine: FakeEngine, monkeypatch: pytest.MonkeyPatch, capsys: pytest.CaptureFixture[str]
) -> None:
    monkeypatch.setenv("SMOL_CLOUD_TOKEN", "x")
    assert warm(3) == EXIT_OK
    captured = capsys.readouterr()
    assert "booting once" in captured.err
    assert len(fake_engine.ops("create")) == 1
    assert fake_engine.ops("create")[0]["target"] == "cloud"
    rows = captured.out.splitlines()[1:]
    assert len(rows) == 1 and rows[0].split()[3] == "-"
    assert fake_engine.live_machines == set()
    assert run("--target", "cloud", "cache", "ls", "--json") == EXIT_OK
    listing = json.loads(capsys.readouterr().out)
    assert listing["kind"] == "cloud" and len(listing["keys"]) == 1


def test_warm_boot_failure_stops_earlier_machines(
    fake_engine: FakeEngine, capsys: pytest.CaptureFixture[str]
) -> None:
    assert warm(1) == EXIT_OK
    capsys.readouterr()
    fake_engine.fail_next("create", BootError("boom", stage="create"))
    assert warm(2) == EXIT_FAILURE
    captured = capsys.readouterr()
    assert "boom" in captured.err and captured.out == ""
    assert len(fake_engine.ops("restore")) == 1
    assert fake_engine.live_machines == set()


# -- cache ------------------------------------------------------------------------------


def test_cache_ls_on_an_empty_cache(cache_dir: Path, capsys: pytest.CaptureFixture[str]) -> None:
    assert run("cache", "ls") == EXIT_OK
    assert "empty" in capsys.readouterr().out
    assert run("cache", "ls", "--json") == EXIT_OK
    data = json.loads(capsys.readouterr().out)
    assert data["keys"] == [] and data["variant_count"] == 0 and data["kind"] == "file"
    assert not cache_dir.exists()


def test_cache_ls_lists_variants(
    fake_engine: FakeEngine, cache_dir: Path, capsys: pytest.CaptureFixture[str]
) -> None:
    assert warm(2) == EXIT_OK
    capsys.readouterr()
    summary = CheckpointCache(cache_dir).ls()
    key = summary.keys[0].key
    ports = sorted(v.port for v in summary.keys[0].variants if v.port is not None)
    assert run("cache", "ls") == EXIT_OK
    out = capsys.readouterr().out
    assert "2 variants" in out and key[:12] in out and "populated" in out
    assert all(str(port) in out for port in ports)
    assert run("cache", "ls", "--json") == EXIT_OK
    data = json.loads(capsys.readouterr().out)
    assert [k["key"] for k in data["keys"]] == [key]
    assert sorted(v["port"] for v in data["keys"][0]["variants"]) == ports
    assert data["variant_count"] == 2 and data["total_bytes"] > 0


def test_cache_prune_requires_a_criterion(capsys: pytest.CaptureFixture[str]) -> None:
    assert run("cache", "prune", "--yes") == EXIT_USAGE
    assert "--older-than" in capsys.readouterr().err


def test_cache_prune_refuses_without_yes_on_a_non_terminal(
    fake_engine: FakeEngine,
    cache_dir: Path,
    monkeypatch: pytest.MonkeyPatch,
    capsys: pytest.CaptureFixture[str],
) -> None:
    assert warm(1) == EXIT_OK
    monkeypatch.setattr(sys, "stdin", io.StringIO())
    assert run("cache", "prune", "--keep-latest", "0") == EXIT_USAGE
    assert "--yes" in capsys.readouterr().err
    assert run("cache", "clear") == EXIT_USAGE
    assert CheckpointCache(cache_dir).ls().variant_count == 1


class _Terminal(io.StringIO):
    def isatty(self) -> bool:
        return True


def test_cache_prune_asks_on_a_terminal(
    fake_engine: FakeEngine,
    cache_dir: Path,
    monkeypatch: pytest.MonkeyPatch,
    capsys: pytest.CaptureFixture[str],
) -> None:
    assert warm(1) == EXIT_OK
    monkeypatch.setattr(sys, "stdin", _Terminal())
    answers = iter(["n", "y"])
    monkeypatch.setattr("builtins.input", lambda prompt: next(answers))
    assert run("cache", "clear") == EXIT_FAILURE
    assert CheckpointCache(cache_dir).ls().variant_count == 1
    assert run("cache", "clear") == EXIT_OK
    assert CheckpointCache(cache_dir).ls().variant_count == 0


def test_cache_prune_keep_latest(
    fake_engine: FakeEngine, cache_dir: Path, capsys: pytest.CaptureFixture[str]
) -> None:
    assert warm(2) == EXIT_OK
    capsys.readouterr()
    assert run("cache", "prune", "--keep-latest", "1", "--yes") == EXIT_OK
    out = capsys.readouterr().out
    assert out.startswith("pruned 1 variant(s)")
    assert CheckpointCache(cache_dir).ls().variant_count == 1
    assert len(fake_engine.ops("prune_store")) == 1


def test_cache_prune_by_age_size_and_staleness(
    fake_engine: FakeEngine, cache_dir: Path, capsys: pytest.CaptureFixture[str]
) -> None:
    assert warm(2) == EXIT_OK
    assert run("cache", "prune", "--stale", "--yes") == EXIT_OK
    assert CheckpointCache(cache_dir).ls().variant_count == 2
    assert run("cache", "prune", "--max-bytes", "1", "--yes") == EXIT_OK
    assert CheckpointCache(cache_dir).ls().variant_count == 0
    assert warm(1) == EXIT_OK
    assert run("cache", "prune", "--older-than", "0s", "--yes") == EXIT_OK
    assert CheckpointCache(cache_dir).ls().variant_count == 0
    assert "pruned 1 variant(s)" in capsys.readouterr().out


def test_cache_clear(
    fake_engine: FakeEngine, cache_dir: Path, capsys: pytest.CaptureFixture[str]
) -> None:
    assert warm(1) == EXIT_OK
    capsys.readouterr()
    assert run("cache", "clear", "--yes") == EXIT_OK
    assert capsys.readouterr().out.startswith("removed 1 variant(s)")
    assert CheckpointCache(cache_dir).keys() == ()
    assert run("cache", "ls", "--json") == EXIT_OK
    assert json.loads(capsys.readouterr().out)["keys"] == []


def test_cache_export(
    fake_engine: FakeEngine, cache_dir: Path, tmp_path: Path, capsys: pytest.CaptureFixture[str]
) -> None:
    assert warm(1) == EXIT_OK
    capsys.readouterr()
    store = CheckpointCache(cache_dir)
    key = store.keys()[0]
    port = store.variants(key)[0].port
    assert port is not None
    out_file = tmp_path / "exported.smolcheckpoint"
    assert run("cache", "export", key[:8], str(out_file)) == EXIT_OK
    captured = capsys.readouterr()
    assert out_file.read_bytes() == store.paths(key, port).checkpoint.read_bytes()
    assert f"exported {key}:latest" in captured.out
    assert "guest RAM" in captured.err
    assert run("cache", "export", f"{key}:{port}", str(tmp_path / "second")) == EXIT_OK
    assert (tmp_path / "second").is_file()
    assert run("cache", "export", "zzz", str(tmp_path / "none")) == EXIT_USAGE
    assert run("cache", "export", f"{key}:1", str(tmp_path / "none")) == EXIT_USAGE
    assert run("cache", "export", key, str(out_file)) == EXIT_FAILURE
    assert "already exists" in capsys.readouterr().err
    assert run("--target", "cloud", "cache", "export", key, str(tmp_path / "cloud")) == EXIT_FAILURE


def test_cache_commands_on_the_cloud_index(
    fake_engine: FakeEngine, monkeypatch: pytest.MonkeyPatch, capsys: pytest.CaptureFixture[str]
) -> None:
    monkeypatch.setenv("SMOL_CLOUD_TOKEN", "x")
    boot_postgres(PostgresSpec(), Settings.from_env(), fake_engine).release()
    assert run("--target", "cloud", "cache", "ls") == EXIT_OK
    out = capsys.readouterr().out
    assert "(cloud)" in out and "1 keys" in out
    assert run("--target", "cloud", "cache", "clear", "--yes") == EXIT_OK
    assert capsys.readouterr().out.startswith("removed 1 variant(s)")
    assert run("--target", "cloud", "cache", "ls", "--json") == EXIT_OK
    assert json.loads(capsys.readouterr().out)["keys"] == []


# -- run --------------------------------------------------------------------------------


def test_run_prints_the_url_and_stops_on_the_event(
    fake_engine: FakeEngine, cache_dir: Path, capsys: pytest.CaptureFixture[str]
) -> None:
    port = pick_free_port()
    stop = threading.Event()
    thread = when_live(fake_engine, stop.set)
    assert run("run", "postgres", "--port", str(port), stop_event=stop) == EXIT_OK
    thread.join(timeout=5)
    captured = capsys.readouterr()
    lines = captured.out.splitlines()
    assert lines[0] == f"postgresql://test:test@127.0.0.1:{port}/test"
    assert lines[1].startswith("booted via cold in ") and " on local " in lines[1]
    assert "machine deleted" in captured.err
    assert fake_engine.live_machines == set() and len(fake_engine.ops("delete")) == 1
    assert len(fake_engine.ops("checkpoint")) == 1
    assert CheckpointCache(cache_dir).ls().variant_count == 1


def test_run_with_no_cache_seed_and_env(
    fake_engine: FakeEngine, cache_dir: Path, tmp_path: Path, capsys: pytest.CaptureFixture[str]
) -> None:
    seed_file = tmp_path / "seed.sql"
    seed_file.write_text("CREATE TABLE s (id int);")
    stop = threading.Event()
    stop.set()  # an already set event ends the run right after boot
    status = run(
        "run",
        "postgres",
        "--no-cache",
        "--seed-sql",
        str(seed_file),
        "--env",
        "K=V",
        stop_event=stop,
    )
    assert status == EXIT_OK
    machine = only_machine(fake_engine)
    assert machine.env["K"] == "V"
    assert any("CREATE TABLE s" in sql for sql in machine.sql_log)
    assert fake_engine.ops("checkpoint") == []
    assert not (cache_dir / "postgres").exists()
    assert fake_engine.live_machines == set()
    assert "booted via cold" in capsys.readouterr().out


def test_run_stops_on_sigint_and_restores_the_handler(
    fake_engine: FakeEngine, capsys: pytest.CaptureFixture[str]
) -> None:
    if threading.current_thread() is not threading.main_thread():
        pytest.skip("signal handlers can only be installed from the main thread")
    before = signal.getsignal(signal.SIGINT)
    thread = when_live(fake_engine, lambda: os.kill(os.getpid(), signal.SIGINT))
    assert run("run", "postgres") == EXIT_OK
    thread.join(timeout=5)
    assert signal.getsignal(signal.SIGINT) is before
    captured = capsys.readouterr()
    assert "received SIGINT" in captured.err and "machine deleted" in captured.err
    assert fake_engine.live_machines == set()


def test_run_boot_failure_exits_1(
    fake_engine: FakeEngine, capsys: pytest.CaptureFixture[str]
) -> None:
    fake_engine.fail_next("create", BootError("no room", stage="create"))
    assert run("run", "postgres", stop_event=threading.Event()) == EXIT_FAILURE
    captured = capsys.readouterr()
    assert "no room" in captured.err and captured.out == ""
    assert fake_engine.live_machines == set()


def test_verbose_shows_library_progress(
    fake_engine: FakeEngine, capsys: pytest.CaptureFixture[str]
) -> None:
    stop = threading.Event()
    stop.set()
    assert run("-v", "run", "postgres", "--no-cache", stop_event=stop) == EXIT_OK
    err = capsys.readouterr().err
    assert "smoltest: info:" in err


def test_global_options_after_the_command(
    fake_engine: FakeEngine, tmp_path: Path, capsys: pytest.CaptureFixture[str]
) -> None:
    elsewhere = tmp_path / "elsewhere"
    assert run("warm", "postgres", "--cache-dir", str(elsewhere)) == EXIT_OK
    capsys.readouterr()
    assert CheckpointCache(elsewhere).ls().variant_count == 1
    assert run("cache", "ls", "--json", "--cache-dir", str(elsewhere)) == EXIT_OK
    data: dict[str, Any] = json.loads(capsys.readouterr().out)
    assert data["variant_count"] == 1
