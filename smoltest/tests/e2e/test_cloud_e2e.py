"""Real machines on Smol Cloud (``-m cloud_e2e``); needs ``SMOL_CLOUD_TOKEN``.

Cloud machines are reached through a loopback tunnel, so every URL points at
``127.0.0.1``. Each test deletes what it created (``cleanup`` runs LIFO) and
checks the machine is gone where the SDK lets it.
"""

from __future__ import annotations

import contextlib
import dataclasses
import logging
import uuid
from collections.abc import Iterable, Iterator
from typing import Any

import pytest

from smoltest.config import Settings
from smoltest.errors import SmoltestError
from smoltest.postgres import PostgresMachine
from smoltest.transport.base import Engine, MachineHandle
from smoltest.wait.strategies import PgIsReadyWaitStrategy, ReadinessView

from .conftest import Pg, plain_url

pytestmark = pytest.mark.cloud_e2e

log = logging.getLogger("smoltest.e2e")

TTL_KEYS = ("ttl_seconds", "ttlSeconds", "ttl_secs", "ttlSecs")
AUTO_STOP_KEYS = ("auto_stop_seconds", "autoStopSeconds", "auto_stop_secs", "autoStopSecs")
METADATA_ATTRS = ("config", "metadata", "info", "describe", "spec")


def assert_gone(handle: MachineHandle) -> None:
    """After delete, the SDK either refuses to report a state or reports a dead one."""
    try:
        state = handle.state()
    except SmoltestError as exc:
        log.info("state() after delete raised, as expected: %s", exc)
        return
    assert "run" not in state.lower(), f"machine {handle.id} still reports {state!r}"


def boot(
    settings: Settings, engine: Engine, cleanup: contextlib.ExitStack, **kwargs: Any
) -> PostgresMachine:
    machine = PostgresMachine(settings=settings, engine=engine, **kwargs)
    cleanup.callback(machine.stop)
    machine.start()
    info = machine.boot_info
    assert info is not None and info.target == "cloud"
    return machine


def test_create_tunnel_and_select_one(
    e2e_settings: Settings, engine: Engine, pg: Pg, cleanup: contextlib.ExitStack
) -> None:
    machine = boot(e2e_settings, engine, cleanup)
    info = machine.boot_info
    assert info is not None
    assert info.variant_port is None
    assert "tunnel" in info.timings
    log.info("cloud boot via %s in %.2fs: %s", info.via, info.elapsed_s, info.timings)
    assert machine.get_container_host_ip() == "127.0.0.1"
    url = plain_url(machine)
    assert pg.scalar(url, "SELECT 1") == 1
    assert pg.scalar(url, "SHOW fsync") == "off"
    exit_code, output = machine.exec(["pg_isready", "-U", machine.username])
    assert exit_code == 0, output
    handle = machine.get_machine()
    machine.stop()
    assert not machine.is_running
    assert_gone(handle)


def test_branch_on_cloud_yields_a_usable_second_url(
    e2e_settings: Settings, engine: Engine, pg: Pg, cleanup: contextlib.ExitStack
) -> None:
    parent = boot(e2e_settings, engine, cleanup)
    parent_url = plain_url(parent)
    pg.execute(parent_url, "CREATE TABLE shared (v text)")

    child = parent.branch()
    cleanup.callback(child.stop)
    child_info = child.boot_info
    assert child_info is not None and child_info.via == "branch"
    child_url = plain_url(child)
    assert child_url != parent_url
    assert child.get_container_host_ip() == "127.0.0.1"
    assert pg.scalar(child_url, "SELECT 1") == 1
    log.info("cloud branch in %.3fs: %s", child_info.elapsed_s, child_info.timings)

    pg.execute(parent_url, "INSERT INTO shared VALUES ('parent')")
    pg.execute(child_url, "INSERT INTO shared VALUES ('child')")
    assert pg.fetch(parent_url, "SELECT v FROM shared") == [("parent",)]
    assert pg.fetch(child_url, "SELECT v FROM shared") == [("child",)]

    child_handle, parent_handle = child.get_machine(), parent.get_machine()
    child.stop()
    parent.stop()
    assert_gone(child_handle)
    assert_gone(parent_handle)


def test_checkpoint_then_restore_by_id(
    e2e_settings: Settings, engine: Engine, pg: Pg, cleanup: contextlib.ExitStack
) -> None:
    machine = boot(e2e_settings, engine, cleanup, cache=False)
    url = plain_url(machine)
    pg.execute(url, "CREATE TABLE marker (v text)")
    pg.execute(url, "INSERT INTO marker VALUES ('before-checkpoint')")

    info = machine.checkpoint()
    assert info.ref.kind == "cloud"
    assert info.ref.locator
    log.info(
        "cloud checkpoint %s (%s bytes, %s ms)", info.ref.locator, info.size_bytes, info.elapsed_ms
    )

    name = f"smoltest-e2e-restore-{uuid.uuid4().hex[:8]}"
    restored = engine.restore_checkpoint(info.ref, name, "cloud")
    cleanup.callback(restored.delete)
    assert restored.id != machine.get_machine().id
    bridge = engine.open_tunnel(restored.id, machine.port, "cloud")
    cleanup.callback(bridge.close)
    endpoint = bridge.open()
    view = ReadinessView(
        handle=restored,
        endpoint=endpoint,
        username=machine.username,
        password=machine.password,
        dbname=machine.dbname,
        guest_port=machine.port,
    )
    PgIsReadyWaitStrategy(machine.username, port=machine.port).wait_until_ready(
        view, e2e_settings.ready_timeout_s, e2e_settings.poll_interval_s
    )
    restored_url = (
        f"postgresql://{machine.username}:{machine.password}@{endpoint.host}:{endpoint.port}"
        f"/{machine.dbname}"
    )
    assert pg.fetch(restored_url, "SELECT v FROM marker") == [("before-checkpoint",)]
    bridge.close()
    restored.delete()
    assert_gone(restored)


def test_second_boot_restores_from_the_cloud_index(
    e2e_settings: Settings, engine: Engine, pg: Pg, cleanup: contextlib.ExitStack
) -> None:
    first = boot(e2e_settings, engine, cleanup)
    first_info = first.boot_info
    assert first_info is not None and first_info.cache_key is not None
    first.stop()

    second = boot(e2e_settings, engine, cleanup)
    info = second.boot_info
    assert info is not None
    assert info.via == "restore", info
    assert info.restored_from is not None and info.restored_from.kind == "cloud"
    assert info.restored_key == first_info.cache_key
    url = plain_url(second)
    pg.execute(url, "CREATE TABLE after_restore (id int)")
    assert pg.scalar(url, "SELECT count(*) FROM after_restore") == 0
    index = e2e_settings.cache_dir / "cloud"
    assert any(index.rglob("index.json")), f"no cloud index under {index}"


def _candidates(obj: Any) -> Iterator[Any]:
    """Metadata-like values the SDK machine object exposes, if any."""
    for attr in METADATA_ATTRS:
        value = getattr(obj, attr, None)
        if value is None:
            continue
        if callable(value):
            try:
                value = value()
            except Exception as exc:  # the SDK may refuse; that is "not exposed"
                log.info("%s() raised %s", attr, exc)
                continue
        yield value


def _lookup(value: Any, keys: Iterable[str]) -> int | None:
    """Find one of ``keys`` on a mapping, dataclass or attribute bag, recursively."""
    if dataclasses.is_dataclass(value) and not isinstance(value, type):
        value = dataclasses.asdict(value)
    if isinstance(value, dict):
        for key in keys:
            if key in value and value[key] is not None:
                return int(value[key])
        for nested in value.values():
            if isinstance(nested, dict):
                found = _lookup(nested, keys)
                if found is not None:
                    return found
        return None
    for key in keys:
        found = getattr(value, key, None)
        if found is not None:
            return int(found)
    return None


def find_metadata(obj: Any, keys: Iterable[str]) -> int | None:
    """The first of ``keys`` found on the SDK machine's metadata, or ``None``."""
    wanted = tuple(keys)
    for candidate in _candidates(obj):
        found = _lookup(candidate, wanted)
        if found is not None:
            return found
    return None


def test_cloud_machine_carries_ttl_and_auto_stop(
    e2e_settings: Settings, engine: Engine, cleanup: contextlib.ExitStack
) -> None:
    machine = boot(e2e_settings, engine, cleanup)
    handle = machine.get_machine()
    sdk_machine = getattr(handle, "machine", handle)
    ttl = find_metadata(sdk_machine, TTL_KEYS)
    auto_stop = find_metadata(sdk_machine, AUTO_STOP_KEYS)
    machine.stop()
    if ttl is None and auto_stop is None:
        pytest.skip(
            "the SDK's Machine exposes no config metadata to read ttl/auto_stop back from; "
            "they are asserted on the MachineSpec in tests/unit/test_boot_strategy.py"
        )
    if ttl is not None:
        assert ttl == e2e_settings.cloud_ttl_seconds
    if auto_stop is not None:
        assert auto_stop == e2e_settings.cloud_auto_stop_seconds
