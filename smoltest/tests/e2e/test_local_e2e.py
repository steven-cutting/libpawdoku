"""Real PostgreSQL machines on the local Smol engine (``-m local_e2e``).

Needs ``/dev/kvm`` (Linux) or Apple Silicon; skipped elsewhere. The first boot
of a session may pull the image, so readiness timeouts are generous. Every
test tears its machines down through the ``cleanup`` stack.
"""

from __future__ import annotations

import contextlib
import json
import logging
import threading
import time
import uuid
import warnings

import pytest

from smoltest.boot.golden import PostgresGolden
from smoltest.cli.main import main
from smoltest.config import Settings
from smoltest.postgres import PostgresMachine, Seed
from smoltest.transport.base import Engine

from .conftest import Pg, plain_url

pytestmark = pytest.mark.local_e2e

log = logging.getLogger("smoltest.e2e")

CLOCK_SKEW_LIMIT_S = 60.0
BRANCH_SOFT_LIMIT_S = 2.0
READY_MESSAGE = b"database system is ready to accept connections"


def clock_skew_s(pg: Pg, url: str) -> float:
    """Seconds between the guest's ``now()`` and the host clock."""
    guest = float(pg.scalar(url, "SELECT extract(epoch FROM now())"))
    return abs(guest - time.time())


def test_cold_boot_runs_queries_in_fast_mode(
    e2e_settings: Settings, engine: Engine, pg: Pg, cleanup: contextlib.ExitStack
) -> None:
    machine = PostgresMachine(settings=e2e_settings, engine=engine)
    cleanup.callback(machine.stop)
    machine.start()
    info = machine.boot_info
    assert info is not None
    assert info.target == "local"
    assert info.via in ("cold", "restore")
    assert info.populated or info.via == "restore"
    assert info.cache_key is not None
    log.info("first boot via %s in %.2fs: %s", info.via, info.elapsed_s, info.timings)

    url = plain_url(machine)
    assert machine.get_container_host_ip() == "127.0.0.1"
    assert machine.get_exposed_port() == machine.endpoint.port
    assert pg.scalar(url, "SELECT 1") == 1
    # Fast mode: the explicit argv replaced the image's command and reached the server.
    assert pg.scalar(url, "SHOW fsync") == "off"
    assert pg.scalar(url, "SHOW synchronous_commit") == "off"
    assert pg.scalar(url, "SHOW full_page_writes") == "off"
    assert clock_skew_s(pg, url) < CLOCK_SKEW_LIMIT_S

    # testcontainers-style exec and in-guest psql.
    exit_code, output = machine.exec(["pg_isready", "-U", machine.username])
    assert exit_code == 0, output
    assert machine.psql("SELECT 2 + 2") == "4"


def test_second_start_restores_the_checkpoint(
    e2e_settings: Settings, engine: Engine, pg: Pg, cleanup: contextlib.ExitStack
) -> None:
    first = PostgresMachine(settings=e2e_settings, engine=engine)
    cleanup.callback(first.stop)
    first.start()
    first_info = first.boot_info
    assert first_info is not None and first_info.cache_key is not None
    first.stop()

    second = PostgresMachine(settings=e2e_settings, engine=engine)
    cleanup.callback(second.stop)
    second.start()
    info = second.boot_info
    assert info is not None
    assert info.via == "restore", info
    assert info.restored_key == first_info.cache_key
    assert info.restored_from is not None and info.restored_from.kind == "file"
    assert info.populated is False
    log.info("restore took %.3fs (clock resynced: %s)", info.elapsed_s, info.clock_resynced)

    # The checkpoint predates any user data; what matters is that the restored
    # server is live, writable and on the right clock.
    url = plain_url(second)
    pg.execute(url, "CREATE TABLE after_restore (id int PRIMARY KEY)")
    pg.execute(url, "INSERT INTO after_restore VALUES (1), (2), (3)")
    assert pg.scalar(url, "SELECT count(*) FROM after_restore") == 3
    skew = clock_skew_s(pg, url)
    assert skew < CLOCK_SKEW_LIMIT_S, (
        f"guest clock is {skew:.0f}s off after restore (clock_resynced={info.clock_resynced})"
    )
    if not info.clock_resynced:
        warnings.warn(
            f"the guest clock was not resynced after restore (date -s failed); skew is {skew:.1f}s",
            stacklevel=1,
        )


def test_golden_branches_are_isolated(
    e2e_settings: Settings, engine: Engine, pg: Pg, cleanup: contextlib.ExitStack
) -> None:
    template = PostgresMachine(settings=e2e_settings, engine=engine)
    golden = PostgresGolden.boot(template)
    cleanup.callback(golden.close)
    golden_url = plain_url(golden.machine)
    pg.execute(golden_url, "CREATE TABLE shared (v text)")

    t0 = time.perf_counter()
    a = golden.branch()
    branch_s = time.perf_counter() - t0
    b = golden.branch()
    for child in (a, b):
        assert child.boot_info is not None
        assert child.parent is golden.machine
    a_url, b_url = plain_url(a), plain_url(b)
    ports = {golden.machine.get_exposed_port(), a.get_exposed_port(), b.get_exposed_port()}
    assert len(ports) == 3

    # Both children inherit the table; a write in A never shows up in B (or the golden).
    pg.execute(a_url, "INSERT INTO shared VALUES ('from-a')")
    assert pg.fetch(a_url, "SELECT v FROM shared") == [("from-a",)]
    assert pg.fetch(b_url, "SELECT v FROM shared") == []
    assert pg.fetch(golden_url, "SELECT v FROM shared") == []

    # Both reachable at the same time, over concurrently open connections.
    with pg.connect(a_url) as conn_a, pg.connect(b_url) as conn_b:
        assert conn_a.execute("SELECT count(*) FROM shared").fetchone()[0] == 1
        assert conn_b.execute("SELECT count(*) FROM shared").fetchone()[0] == 0

    via = a.boot_info.via if a.boot_info is not None else None
    log.info(
        "first child via %s in %.3fs (branch_supported=%s)", via, branch_s, golden.branch_supported
    )
    if via != "branch":
        warnings.warn(
            f"the golden fell back to fresh machines (via={via}); branching is unsupported here",
            stacklevel=1,
        )
    elif branch_s >= BRANCH_SOFT_LIMIT_S:
        warnings.warn(
            f"branch took {branch_s:.3f}s, above the {BRANCH_SOFT_LIMIT_S:g}s target",
            stacklevel=1,
        )
    assert golden.branch_count + golden.fresh_count == 2

    a.stop()
    assert not a.is_running
    assert golden.children == (b,)
    b.stop()
    assert not golden.children


def test_two_standalone_machines_boot_concurrently(
    e2e_settings: Settings, engine: Engine, pg: Pg, cleanup: contextlib.ExitStack
) -> None:
    machines = [PostgresMachine(settings=e2e_settings, engine=engine) for _ in range(2)]
    for machine in machines:
        cleanup.callback(machine.stop)
    errors: list[BaseException] = []

    def start(machine: PostgresMachine) -> None:
        try:
            machine.start()
        except BaseException as exc:  # reported by the main thread
            errors.append(exc)

    threads = [
        threading.Thread(target=start, args=(m,), name=f"e2e-start-{i}")
        for i, m in enumerate(machines)
    ]
    for thread in threads:
        thread.start()
    for thread in threads:
        thread.join()
    assert not errors, errors

    ports = {machine.get_exposed_port() for machine in machines}
    assert len(ports) == 2
    for machine in machines:
        assert pg.scalar(plain_url(machine), "SELECT 1") == 1
    vias = sorted(m.boot_info.via for m in machines if m.boot_info is not None)
    log.info("concurrent boots: %s on ports %s", vias, sorted(ports))
    # Two live machines of one spec mean two distinct cache variants (or one plus a cold boot).
    keys = {m.boot_info.cache_key for m in machines if m.boot_info is not None}
    assert len(keys) == 1


def test_seed_is_applied_once_and_restored(
    e2e_settings: Settings, engine: Engine, pg: Pg, cleanup: contextlib.ExitStack
) -> None:
    table = f"seeded_{uuid.uuid4().hex[:8]}"
    seed = Seed.from_sql(
        f"CREATE TABLE {table} (id int PRIMARY KEY);\n"
        f"INSERT INTO {table} SELECT generate_series(1, 5);\n"
    )
    first = PostgresGolden.boot(PostgresMachine(settings=e2e_settings, engine=engine, seed=seed))
    cleanup.callback(first.close)
    info = first.boot_info
    assert info.seeded is True
    assert info.seeded_key is not None and info.seeded_key != info.cache_key
    assert info.populated is True
    assert pg.scalar(plain_url(first.machine), f"SELECT count(*) FROM {table}") == 5
    first.close()

    second = PostgresGolden.boot(PostgresMachine(settings=e2e_settings, engine=engine, seed=seed))
    cleanup.callback(second.close)
    info2 = second.boot_info
    assert info2.via == "restore", info2
    assert info2.restored_key == info.seeded_key
    assert info2.seeded is False
    assert pg.scalar(plain_url(second.machine), f"SELECT count(*) FROM {table}") == 5
    child = second.branch()
    assert pg.scalar(plain_url(child), f"SELECT count(*) FROM {table}") == 5
    child.stop()


@pytest.mark.parametrize("image", ["postgres:16", "postgres:16-alpine"])
def test_get_logs_with_capture(
    image: str, e2e_settings: Settings, engine: Engine, pg: Pg, cleanup: contextlib.ExitStack
) -> None:
    # The alpine image has no /var/log/postgresql; the log goes to /tmp in both families.
    machine = PostgresMachine(image, settings=e2e_settings, engine=engine, capture_logs=True)
    cleanup.callback(machine.stop)
    machine.start()
    assert pg.scalar(plain_url(machine), "SELECT 1") == 1
    stdout, stderr = machine.get_logs()
    assert stdout, "capture_logs=True produced no server log"
    assert stderr == b""
    if READY_MESSAGE not in stdout:
        warnings.warn(
            f"server log lacks {READY_MESSAGE!r}; first lines: {stdout[:200]!r}", stacklevel=1
        )


def test_cli_warm_ls_prune_clear(
    tmp_path_factory: pytest.TempPathFactory, capsys: pytest.CaptureFixture[str]
) -> None:
    cache_dir = tmp_path_factory.mktemp("cli-cache")
    base = ["--cache-dir", str(cache_dir), "--target", "local"]

    assert main([*base, "warm", "postgres", "--variants", "1"]) == 0
    captured = capsys.readouterr()
    assert "cold" in captured.out, captured.out

    assert main([*base, "cache", "ls", "--json"]) == 0
    summary = json.loads(capsys.readouterr().out)
    variants = [v for key in summary["keys"] for v in key["variants"]]
    assert len(variants) == 1
    assert variants[0]["populated"] is True
    assert variants[0]["claimed_by"] is None
    assert variants[0]["size_bytes"] > 0

    # A second warm with two variants restores the first and cold-boots another port.
    assert main([*base, "warm", "postgres", "--variants", "2"]) == 0
    out = capsys.readouterr().out
    assert "restore" in out and "cold" in out, out
    assert main([*base, "cache", "ls"]) == 0
    listing = capsys.readouterr().out
    assert "2 variants" in listing, listing

    assert main([*base, "cache", "prune", "--keep-latest", "1", "--yes"]) == 0
    assert "pruned 1 variant" in capsys.readouterr().out
    assert main([*base, "cache", "prune", "--stale", "--yes"]) == 0
    assert "pruned 0 variant" in capsys.readouterr().out
    assert main([*base, "cache", "clear", "--yes"]) == 0
    assert "removed 1 variant" in capsys.readouterr().out
    assert main([*base, "cache", "ls", "--json"]) == 0
    assert json.loads(capsys.readouterr().out)["keys"] == []
