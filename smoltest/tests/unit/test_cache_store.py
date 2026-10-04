"""The checkpoint store: layout, claims, commit markers, eviction, prune and locking."""

from __future__ import annotations

import dataclasses
import gc
import json
import multiprocessing
import os
import re
import socket
import stat
import threading
import time
from pathlib import Path
from typing import Any

import pytest

from smoltest._log import reset_warn_once
from smoltest._ports import is_port_free, pick_free_port
from smoltest.cache import open_checkpoint_backend
from smoltest.cache.cloud_index import CloudCheckpointIndex
from smoltest.cache.key import host_signature
from smoltest.cache.lock import FileLock, LockTimeout
from smoltest.cache.store import (
    CacheSummary,
    CheckpointBackend,
    CheckpointCache,
    PruneReport,
    VariantClaim,
    VariantMeta,
    path_size,
    pid_alive,
    read_claim,
)
from smoltest.config import Settings
from smoltest.errors import (
    CacheCorrupt,
    CacheError,
    NotSupportedError,
    SmoltestError,
    SmoltestWarning,
)
from smoltest.testing import FakeEngine, FakeMachine
from smoltest.transport.base import CheckpointRef, MachineSpec, PortMapping

KEY = "a" * 32
KEY2 = "b" * 32
INPUTS: dict[str, Any] = {
    "format": 1,
    "image": "postgres:16",
    "env": {"POSTGRES_USER": "test", "POSTGRES_PASSWORD": "s3cret"},
    "sdk_version": "fake-1.22.2",
    "host": host_signature().as_dict(),
}
DEAD_PID = 2**22 - 1


def mspec(port: int | None, image: str = "postgres:16") -> MachineSpec:
    return MachineSpec(
        image=image, argv=("docker-entrypoint.sh", "postgres"), ports=(PortMapping(port, 5432),)
    )


def populate(
    cache: CheckpointCache,
    engine: FakeEngine,
    key: str = KEY,
    port: int | None = None,
    *,
    keep_machine: bool = False,
    inputs: dict[str, Any] | None = INPUTS,
) -> tuple[VariantClaim, FakeMachine]:
    """Cold-boot a fake machine on ``port`` and populate ``key`` with it.

    Without an explicit ``port`` one is picked that no variant of this cache
    already uses: the machine is deleted on return, which frees its host port,
    and a later call that drew the same port would collide with the claim the
    first call left behind. The machine is also deleted when the claim or the
    checkpoint fails, so a failure never shows up as a leaked machine.
    """
    if port is None:
        taken = {v.port for k in cache.ls().keys for v in k.variants if v.port is not None}
        port = pick_free_port(exclude=taken)
    machine = engine.create(mspec(port), "local")
    assert isinstance(machine, FakeMachine)
    try:
        claim = cache.reserve_new_variant(key, port)
        cache.populate(key, claim, machine, inputs, engine=engine)
    except BaseException:
        machine.delete()
        raise
    if not keep_machine:
        machine.delete()
    return claim, machine


def set_last_used(cache: CheckpointCache, key: str, port: int, when: float) -> None:
    meta_path = cache.paths(key, port).meta
    data = json.loads(meta_path.read_text())
    data["last_used_at"] = when
    meta_path.write_text(json.dumps(data))


def ports_of(cache: CheckpointCache, key: str = KEY) -> set[int]:
    return {v.port for v in cache.variants(key) if v.populated and v.port is not None}


def is_held(lock: FileLock) -> bool:
    """Read ``lock.held`` through a call so mypy does not narrow it between asserts."""
    return lock.held


@pytest.fixture
def cache(tmp_path: Path) -> CheckpointCache:
    return CheckpointCache(tmp_path / "cache", max_bytes=10**9, lock_timeout_s=10)


# -- layout and permissions ---------------------------------------------------------------


def test_open_from_settings_and_dispatch(settings: Settings) -> None:
    cache = CheckpointCache.open(settings, "local")
    assert cache.root == settings.cache_dir and cache.max_bytes == settings.cache_max_bytes
    assert cache.lock_timeout_s == 2 * settings.ready_timeout_s + 120.0
    assert cache.family_dir.is_dir() and cache.family_dir == settings.cache_dir / "postgres"
    assert stat.S_IMODE(cache.family_dir.stat().st_mode) == 0o700
    with pytest.raises(NotSupportedError):
        CheckpointCache.open(settings, "cloud")
    assert isinstance(open_checkpoint_backend(settings, "local"), CheckpointCache)
    cloud = open_checkpoint_backend(settings, "cloud", {"SMOL_CLOUD_URL": "https://x.test"})
    assert isinstance(cloud, CloudCheckpointIndex) and cloud.base_url == "https://x.test"
    assert cloud.path.parent.parent == settings.cache_dir / "cloud"
    assert isinstance(cache, CheckpointBackend) and isinstance(cloud, CheckpointBackend)


def test_populate_writes_the_layout_with_private_modes(
    cache: CheckpointCache, fake_engine: FakeEngine
) -> None:
    claim, machine = populate(cache, fake_engine)
    port = claim.port
    assert port is not None and claim.populated and claim.meta is not None
    paths = cache.paths(KEY, port)
    assert paths.checkpoint.name == f"{port}.smolcheckpoint"
    assert paths.meta.name == f"{port}.meta.json" and paths.claim.name == f"{port}.claim"
    for p in (paths.checkpoint, paths.meta, paths.claim, cache.inputs_path(KEY)):
        assert p.is_file(), p
        assert stat.S_IMODE(p.stat().st_mode) == 0o600, p
    for d in (cache.root, cache.family_dir, cache.key_dir(KEY), cache.store_dir):
        assert stat.S_IMODE(d.stat().st_mode) == 0o700, d
    assert (cache.family_dir / f"{KEY}.lock").is_file() and (cache.family_dir / ".lock").is_file()
    assert list(cache.store_dir.glob("*.chunk")), "checkpoint went through the dedup store"
    meta = claim.meta
    assert meta.ref == CheckpointRef("file", str(paths.checkpoint)) == claim.ref
    assert meta.key == KEY and meta.port == port and meta.machine_id == machine.id
    assert meta.sdk_version == "fake-1.22.2" and meta.size_bytes == paths.checkpoint.stat().st_size
    assert VariantMeta.load(paths.meta) == meta
    assert fake_engine.ops("checkpoint")[0]["store"] == str(cache.store_dir)
    claim.release()


def test_inputs_json_is_redacted_and_written_once(
    cache: CheckpointCache, fake_engine: FakeEngine
) -> None:
    c1, _ = populate(cache, fake_engine)
    data = json.loads(cache.inputs_path(KEY).read_text())
    assert data["format"] == 1 and data["key"] == KEY
    assert data["inputs"]["env"]["POSTGRES_PASSWORD"].startswith("sha256:")
    assert "s3cret" not in cache.inputs_path(KEY).read_text()
    assert data["inputs"]["env"]["POSTGRES_USER"] == "test"
    first = cache.inputs_path(KEY).read_bytes()
    c2, _ = populate(cache, fake_engine, inputs={"image": "different"})
    assert cache.inputs_path(KEY).read_bytes() == first, "inputs.json is written only if missing"
    assert cache.ls().keys[0].inputs == data["inputs"]
    c1.release()
    c2.release()


# -- claims -----------------------------------------------------------------------------


def test_claim_lifecycle(cache: CheckpointCache, fake_engine: FakeEngine) -> None:
    assert cache.claim_variant(KEY) is None, "nothing cached yet"
    claim, _ = populate(cache, fake_engine)
    port = claim.port
    assert port is not None
    record = read_claim(cache.paths(KEY, port).claim)
    assert record is not None and record.pid == os.getpid() and record.alive
    assert cache.claim_variant(KEY) is None, "our own live claim excludes the variant"
    claim.release()
    assert claim.released and not cache.paths(KEY, port).claim.exists()
    claim.release()  # idempotent

    again = cache.claim_variant(KEY)
    assert again is not None and again.port == port and again.populated
    assert again.ref == CheckpointRef("file", str(cache.paths(KEY, port).checkpoint))
    assert cache.claim_variant(KEY) is None
    restored = fake_engine.restore_checkpoint(again.ref, "restored", "local")
    assert restored.host_port(5432) == ("127.0.0.1", port)
    cache.touch(again)
    assert again.meta is not None and again.meta.last_used_at >= again.meta.created_at
    restored.delete()
    with again:
        pass
    assert again.released
    assert cache.claim_variant(KEY) is not None


def test_claim_is_released_when_garbage_collected(
    cache: CheckpointCache, fake_engine: FakeEngine
) -> None:
    claim, _ = populate(cache, fake_engine)
    port = claim.port
    assert port is not None
    claim_path = cache.paths(KEY, port).claim
    del claim
    gc.collect()
    assert not claim_path.exists()


def test_release_leaves_a_foreign_claim_alone(
    cache: CheckpointCache, fake_engine: FakeEngine
) -> None:
    claim, _ = populate(cache, fake_engine)
    port = claim.port
    assert port is not None
    claim.release()
    other = cache.claim_variant(KEY)
    assert other is not None
    claim.release()  # a stale release must not steal the newer claim file
    assert cache.paths(KEY, port).claim.exists()
    other.release()


def test_stale_pid_claim_is_ignored(cache: CheckpointCache, fake_engine: FakeEngine) -> None:
    assert not pid_alive(DEAD_PID), "pid cannot exist on this host"
    claim, _ = populate(cache, fake_engine)
    port = claim.port
    assert port is not None
    claim.release()
    claim_path = cache.paths(KEY, port).claim
    claim_path.write_text(json.dumps({"pid": DEAD_PID, "token": "x", "created_at": 0}))
    taken = cache.claim_variant(KEY)
    assert taken is not None and taken.port == port
    record = read_claim(claim_path)
    assert record is not None and record.pid == os.getpid()
    taken.release()
    # legacy bare-pid claim files and garbage are read leniently
    claim_path.write_text(str(DEAD_PID))
    assert cache.claim_variant(KEY) is not None
    claim_path.write_text("not json")
    assert read_claim(claim_path) is None


def test_variant_without_meta_is_deleted_on_scan_unless_live_claimed(
    cache: CheckpointCache, fake_engine: FakeEngine
) -> None:
    good, _ = populate(cache, fake_engine)
    good.release()
    garbage_port = pick_free_port(exclude=[good.port or 0])
    cache.paths(KEY, garbage_port).checkpoint.write_bytes(b"half-written")
    taken = cache.claim_variant(KEY)
    assert taken is not None and taken.port == good.port
    assert not cache.paths(KEY, garbage_port).checkpoint.exists(), "garbage removed"
    taken.release()
    # a variant somebody else is populating right now (live claim) is left alone
    reserved = cache.reserve_new_variant(KEY, garbage_port)
    cache.paths(KEY, garbage_port).checkpoint.write_bytes(b"in progress")
    taken = cache.claim_variant(KEY)
    assert taken is not None and taken.port == good.port
    assert cache.paths(KEY, garbage_port).checkpoint.exists()
    taken.release()
    reserved.release()
    # meta without a checkpoint is garbage too
    paths = cache.paths(KEY, garbage_port)
    paths.checkpoint.unlink()
    assert good.meta is not None
    paths.meta.write_bytes(good.meta.to_json())
    assert cache.claim_variant(KEY, pinned_port=garbage_port) is None
    assert not paths.meta.exists()


def test_busy_port_variant_is_skipped(cache: CheckpointCache, fake_engine: FakeEngine) -> None:
    claim, _ = populate(cache, fake_engine)
    port = claim.port
    assert port is not None
    claim.release()
    blocker = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
    blocker.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)
    blocker.bind(("127.0.0.1", port))
    blocker.listen(1)
    try:
        assert not is_port_free(port)
        assert cache.claim_variant(KEY) is None
        assert cache.paths(KEY, port).checkpoint.exists(), "busy is not garbage"
    finally:
        blocker.close()
    taken = cache.claim_variant(KEY)
    assert taken is not None and taken.port == port
    taken.release()


def test_claim_prefers_most_recently_used_and_honours_pinned_port(
    cache: CheckpointCache, fake_engine: FakeEngine
) -> None:
    c1, _ = populate(cache, fake_engine)
    c2, _ = populate(cache, fake_engine)
    p1, p2 = c1.port, c2.port
    assert p1 is not None and p2 is not None
    c1.release()
    c2.release()
    set_last_used(cache, KEY, p1, 2000.0)
    set_last_used(cache, KEY, p2, 1000.0)
    hot = cache.claim_variant(KEY)
    assert hot is not None and hot.port == p1
    pinned = cache.claim_variant(KEY, pinned_port=p2)
    assert pinned is not None and pinned.port == p2
    assert cache.claim_variant(KEY, pinned_port=p1) is None, "already claimed"
    assert cache.claim_variant(KEY, pinned_port=p1 + 1) is None, "no such variant"
    hot.release()
    pinned.release()


def test_reserve_new_variant_rejects_live_claims_and_populated_ports(
    cache: CheckpointCache, fake_engine: FakeEngine
) -> None:
    claim, _ = populate(cache, fake_engine)
    port = claim.port
    assert port is not None
    with pytest.raises(CacheError, match="claimed by live pid"):
        cache.reserve_new_variant(KEY, port)
    claim.release()
    with pytest.raises(CacheError, match="already populated"):
        cache.reserve_new_variant(KEY, port)
    with pytest.raises(CacheError, match="needs a host port"):
        cache.reserve_new_variant(KEY, None)
    for bad in ("", "store", ".hidden", "a/b"):
        with pytest.raises(CacheError, match="invalid cache key"):
            cache.reserve_new_variant(bad, port)
    # a dead claim on a garbage variant is swept and the port reused
    other = pick_free_port(exclude=[port])
    cache.paths(KEY, other).checkpoint.write_bytes(b"junk")
    cache.paths(KEY, other).claim.write_text(json.dumps({"pid": DEAD_PID, "token": "t"}))
    fresh = cache.reserve_new_variant(KEY, other)
    assert not fresh.populated and fresh.ref == CheckpointRef(
        "file", str(cache.paths(KEY, other).checkpoint)
    )
    assert not cache.paths(KEY, other).checkpoint.exists()
    fresh.release()


# -- populate / invalidate ----------------------------------------------------------------


def test_populate_validates_claim_and_cleans_up_on_failure(
    cache: CheckpointCache, fake_engine: FakeEngine
) -> None:
    port = pick_free_port()
    machine = fake_engine.create(mspec(port), "local")
    claim = cache.reserve_new_variant(KEY, port)
    with pytest.raises(CacheError, match="not"):
        cache.populate(KEY2, claim, machine)
    fake_engine.fail_next("checkpoint", SmoltestError("disk full"))
    with pytest.raises(SmoltestError, match="disk full"):
        cache.populate(KEY, claim, machine, INPUTS)
    paths = cache.paths(KEY, port)
    assert not paths.checkpoint.exists() and not paths.meta.exists()
    assert paths.claim.exists(), "the claim survives a failed populate"
    assert not claim.populated
    cache.populate(KEY, claim, machine, INPUTS)  # retry works
    assert claim.meta is not None and paths.meta.exists()
    claim.release()
    with pytest.raises(CacheError, match="released"):
        cache.populate(KEY, claim, machine, INPUTS)
    machine.delete()


def test_populate_removes_leftover_output_and_meta_is_the_commit_marker(
    cache: CheckpointCache, fake_engine: FakeEngine
) -> None:
    port = pick_free_port()
    machine = fake_engine.create(mspec(port), "local")
    claim = cache.reserve_new_variant(KEY, port)
    leftover = cache.paths(KEY, port).checkpoint
    leftover.write_bytes(b"crashed half-way")  # the fake refuses to overwrite an output
    meta = cache.populate(KEY, claim, machine, INPUTS)
    assert json.loads(leftover.read_bytes())["format"] == 1
    assert meta.to_dict() == json.loads(cache.paths(KEY, port).meta.read_text())
    machine.delete()
    claim.release()


def test_invalidate_removes_variant_and_empty_key_dir(
    cache: CheckpointCache, fake_engine: FakeEngine
) -> None:
    c1, _ = populate(cache, fake_engine)
    c2, _ = populate(cache, fake_engine)
    assert c1.port is not None and c2.port is not None
    cache.invalidate(c1)
    assert c1.released
    paths = cache.paths(KEY, c1.port)
    assert not paths.checkpoint.exists() and not paths.meta.exists() and not paths.claim.exists()
    assert cache.key_dir(KEY).is_dir() and ports_of(cache) == {c2.port}
    cache.invalidate(c2)
    assert not cache.key_dir(KEY).exists()
    assert cache.keys() == ()


def test_corrupt_meta_raises_cache_corrupt_and_is_invalidated_on_claim(
    cache: CheckpointCache, fake_engine: FakeEngine
) -> None:
    claim, _ = populate(cache, fake_engine)
    port = claim.port
    assert port is not None
    claim.release()
    paths = cache.paths(KEY, port)
    paths.meta.write_text("{not json")
    with pytest.raises(CacheCorrupt):
        VariantMeta.load(paths.meta)
    with pytest.raises(CacheCorrupt, match="format"):
        VariantMeta.from_dict({"format": 99})
    with pytest.raises(CacheCorrupt):
        VariantMeta.from_dict({"format": 1, "key": KEY})  # missing fields
    with pytest.raises(CacheCorrupt):
        VariantMeta.from_dict([])
    listed = cache.variants(KEY)
    assert len(listed) == 1 and listed[0].corrupt and not listed[0].populated
    assert cache.claim_variant(KEY) is None
    assert not paths.meta.exists() and not paths.checkpoint.exists(), "invalidated"


def test_meta_copied_from_another_variant_or_key_is_invalidated(
    cache: CheckpointCache, fake_engine: FakeEngine
) -> None:
    first, _ = populate(cache, fake_engine)
    other, _ = populate(cache, fake_engine, KEY2)
    port, other_port = first.port, other.port
    assert port is not None and other_port is not None
    first.release()
    other.release()
    paths = cache.paths(KEY, port)
    genuine = paths.meta.read_bytes()

    def check_rejected(label: str) -> None:
        listed = cache.variants(KEY)
        assert len(listed) == 1 and listed[0].corrupt and not listed[0].populated, label
        assert cache.claim_variant(KEY) is None, label
        assert not paths.meta.exists() and not paths.checkpoint.exists(), f"{label}: invalidated"

    def rewrite(**changes: Any) -> None:
        data = json.loads(genuine)
        data.update(changes)
        paths.meta.write_bytes(json.dumps(data).encode())
        paths.checkpoint.write_bytes(b"checkpoint")

    # A meta copied from another key (its key, ref and port all say so).
    paths.meta.write_bytes(cache.paths(KEY2, other_port).meta.read_bytes())
    paths.checkpoint.write_bytes(b"checkpoint")
    check_rejected("foreign key")
    rewrite(port=port + 1)
    check_rejected("foreign port")
    rewrite(ref={"kind": "file", "locator": str(cache.paths(KEY, port + 1).checkpoint)})
    check_rejected("foreign checkpoint path")
    rewrite(ref={"kind": "cloud", "locator": "ckpt-1"})
    check_rejected("cloud ref in a local cache")
    # The genuine meta is still accepted, and a symlinked cache root resolves alike.
    other_claim = cache.claim_variant(KEY2)
    assert other_claim is not None and other_claim.port == other_port
    other_claim.release()
    link = cache.root.parent / "link"
    link.symlink_to(cache.root, target_is_directory=True)
    linked = CheckpointCache(link, max_bytes=10**9, lock_timeout_s=10)
    assert ports_of(linked, KEY2) == {other_port}, "realpath comparison tolerates a symlinked root"
    assert not cache.variants(KEY2)[0].corrupt


def test_populate_records_the_variant_path_however_the_engine_spells_it(
    cache: CheckpointCache, fake_engine: FakeEngine, monkeypatch: pytest.MonkeyPatch
) -> None:
    """The meta's ref is the variant's own path, so the scan check never depends on the SDK."""
    port = pick_free_port()
    machine = fake_engine.create(mspec(port), "local")
    assert isinstance(machine, FakeMachine)
    paths = cache.paths(KEY, port)
    alias = cache.root.parent / "alias"
    alias.symlink_to(cache.root, target_is_directory=True)
    echoed = alias / paths.checkpoint.relative_to(cache.root)
    real_checkpoint = machine.checkpoint

    def checkpoint_via_alias(output: str | None = None, store: str | None = None) -> Any:
        info = real_checkpoint(output, store)
        return dataclasses.replace(info, ref=CheckpointRef("file", str(echoed)))

    monkeypatch.setattr(machine, "checkpoint", checkpoint_via_alias)
    claim = cache.reserve_new_variant(KEY, port)
    meta = cache.populate(KEY, claim, machine, INPUTS, engine=fake_engine)
    machine.delete()  # frees the port for the bind probe in claim_variant
    assert meta.ref == CheckpointRef("file", str(paths.checkpoint)) == claim.ref
    assert VariantMeta.load(paths.meta).ref.locator == str(paths.checkpoint)
    claim.release()
    assert [v.corrupt for v in cache.variants(KEY)] == [False]
    claimed = cache.claim_variant(KEY)
    assert claimed is not None and claimed.port == port
    claimed.release()

    # An engine that writes somewhere else does not populate the variant.
    other = fake_engine.create(mspec(port + 1), "local")
    assert isinstance(other, FakeMachine)
    elsewhere = cache.root.parent / "elsewhere.smolcheckpoint"
    other_checkpoint = other.checkpoint

    def checkpoint_elsewhere(output: str | None = None, store: str | None = None) -> Any:
        return other_checkpoint(str(elsewhere), store)

    monkeypatch.setattr(other, "checkpoint", checkpoint_elsewhere)
    stray = cache.reserve_new_variant(KEY2, port + 1)
    with pytest.raises(CacheCorrupt, match=re.escape(f"wrote checkpoint {elsewhere}, not")):
        cache.populate(KEY2, stray, other, INPUTS, engine=fake_engine)
    assert not cache.paths(KEY2, port + 1).meta.exists()
    stray.release()
    assert not cache.variants(KEY2)
    other.delete()


def test_touch_and_record_branch_ok_rewrite_meta(
    cache: CheckpointCache, fake_engine: FakeEngine
) -> None:
    claim, _ = populate(cache, fake_engine)
    port = claim.port
    assert port is not None and claim.meta is not None
    set_last_used(cache, KEY, port, 1.0)
    claim.meta = VariantMeta.load(cache.paths(KEY, port).meta)
    cache.touch(claim)
    assert claim.meta.last_used_at > 1.0
    assert VariantMeta.load(cache.paths(KEY, port).meta).last_used_at == claim.meta.last_used_at
    cache.record_branch_ok(claim, False)
    assert VariantMeta.load(cache.paths(KEY, port).meta).branch_ok is False
    assert cache.variants(KEY)[0].branch_ok is False
    cache.invalidate(claim)
    cache.touch(claim)  # meta gone: silently a no-op
    unpopulated = cache.reserve_new_variant(KEY, port)
    cache.touch(unpopulated)
    unpopulated.release()


# -- size accounting, eviction, prune, clear, export --------------------------------------


def test_path_size_handles_files_and_directories(tmp_path: Path) -> None:
    f = tmp_path / "f"
    f.write_bytes(b"x" * 10)
    d = tmp_path / "d"
    (d / "sub").mkdir(parents=True)
    (d / "a").write_bytes(b"x" * 3)
    (d / "sub" / "b").write_bytes(b"x" * 4)
    assert path_size(f) == 10 and path_size(d) == 7 and path_size(tmp_path / "nope") == 0


def test_lru_eviction_by_budget_protects_the_current_claim(
    tmp_path: Path, fake_engine: FakeEngine
) -> None:
    big = CheckpointCache(tmp_path / "cache", max_bytes=10**9, lock_timeout_s=10)
    claims = [populate(big, fake_engine)[0] for _ in range(3)]
    ports = [c.port for c in claims]
    assert all(p is not None for p in ports)
    for c in claims:
        c.release()
    for i, p in enumerate(ports):
        assert p is not None
        set_last_used(big, KEY, p, 1000.0 * (i + 1))
    total = big.size_bytes()
    assert total == big.ls().total_bytes > 0
    small = CheckpointCache(tmp_path / "cache", max_bytes=total + 64, lock_timeout_s=10)
    fourth, _ = populate(small, fake_engine)
    remaining = ports_of(small)
    assert ports[0] not in remaining, "least recently used variant evicted"
    assert remaining == {ports[1], ports[2], fourth.port}
    assert small.size_bytes() <= total + 64
    assert fake_engine.ops("prune_store"), "the dedup store was pruned after eviction"
    fourth.release()
    # a budget smaller than one checkpoint still keeps the variant being populated
    tiny = CheckpointCache(tmp_path / "cache", max_bytes=1, lock_timeout_s=10)
    fifth, _ = populate(tiny, fake_engine)
    assert ports_of(tiny) == {fifth.port}
    fifth.release()


def test_eviction_never_removes_live_claimed_variants(
    tmp_path: Path, fake_engine: FakeEngine
) -> None:
    big = CheckpointCache(tmp_path / "cache", max_bytes=10**9, lock_timeout_s=10)
    held, _ = populate(big, fake_engine)  # claim kept: a process is using it
    tiny = CheckpointCache(tmp_path / "cache", max_bytes=1, lock_timeout_s=10)
    new, _ = populate(tiny, fake_engine)
    assert ports_of(tiny) == {held.port, new.port}
    held.release()
    new.release()


def test_prune_by_age_count_budget_and_stale(
    cache: CheckpointCache, fake_engine: FakeEngine
) -> None:
    claims = [populate(cache, fake_engine)[0] for _ in range(3)]
    other, _ = populate(cache, fake_engine, key=KEY2)
    for c in (*claims, other):
        c.release()
    ports = [c.port for c in claims]
    now = time.time()
    for i, p in enumerate(ports):
        assert p is not None
        set_last_used(cache, KEY, p, now - 3600 * (i + 1))
    assert cache.prune() == PruneReport(), "no criteria, nothing removed"
    report = cache.prune(older_than_s=2.5 * 3600, engine=fake_engine)
    assert {v.port for v in report.removed} == {ports[2]} and report.freed_bytes > 0
    assert report.store_chunks_pruned >= 1
    report = cache.prune(keep_latest=1)
    assert {v.port for v in report.removed} == {ports[1]}
    assert ports_of(cache) == {ports[0]} and ports_of(cache, KEY2) == {other.port}
    report = cache.prune(max_bytes=0)
    assert {(v.key, v.port) for v in report.removed} == {(KEY, ports[0]), (KEY2, other.port)}
    assert set(report.removed_keys) == {KEY, KEY2}
    assert cache.keys() == () and cache.ls().keys == ()


def test_prune_stale_removes_garbage_dead_claims_sdk_and_host_mismatches(
    cache: CheckpointCache, fake_engine: FakeEngine
) -> None:
    fresh, _ = populate(cache, fake_engine)
    fresh.release()
    old_sdk, _ = populate(cache, fake_engine, key=KEY2)
    old_sdk.release()
    assert old_sdk.port is not None
    meta_path = cache.paths(KEY2, old_sdk.port).meta
    data = json.loads(meta_path.read_text())
    data["sdk_version"] = "fake-0.0.1"
    meta_path.write_text(json.dumps(data))
    garbage_port = pick_free_port(exclude=[fresh.port or 0, old_sdk.port])
    cache.paths(KEY, garbage_port).checkpoint.write_bytes(b"junk")
    cache.paths(KEY, garbage_port).claim.write_text(json.dumps({"pid": DEAD_PID, "token": ""}))
    report = cache.prune(stale=True, engine=fake_engine)
    assert {(v.key, v.port) for v in report.removed} == {(KEY, garbage_port), (KEY2, old_sdk.port)}
    assert report.claims_removed == 1 and report.removed_keys == (KEY2,)
    assert ports_of(cache) == {fresh.port}
    # a different host signature in inputs.json marks the whole key stale
    inputs_path = cache.inputs_path(KEY)
    recorded = json.loads(inputs_path.read_text())
    recorded["inputs"]["host"]["cpu_flags_hash"] = "0" * 16
    inputs_path.write_text(json.dumps(recorded))
    report = cache.prune(stale=True)
    assert {v.port for v in report.removed} == {fresh.port}
    assert cache.keys() == ()


def test_prune_protects_live_claims(cache: CheckpointCache, fake_engine: FakeEngine) -> None:
    held, _ = populate(cache, fake_engine)
    assert held.port is not None
    set_last_used(cache, KEY, held.port, 0.0)
    assert cache.prune(older_than_s=1, max_bytes=0, keep_latest=0, stale=True).removed == ()
    assert ports_of(cache) == {held.port}
    held.release()
    assert {v.port for v in cache.prune(older_than_s=1).removed} == {held.port}


def test_clear_removes_everything_but_lock_files(
    cache: CheckpointCache, fake_engine: FakeEngine
) -> None:
    c1, _ = populate(cache, fake_engine)
    c2, _ = populate(cache, fake_engine, key=KEY2)
    report = cache.clear()
    assert len(report.removed) == 2 and set(report.removed_keys) == {KEY, KEY2}
    assert report.claims_removed == 2 and report.freed_bytes > 0
    assert not cache.store_dir.exists() and cache.keys() == ()
    assert (cache.family_dir / ".lock").exists()
    assert cache.ls().total_bytes == 0 and cache.claim_variant(KEY) is None
    c1.release()
    c2.release()
    assert cache.clear() == PruneReport()


def test_ls_summary_is_structured_and_json_ready(
    cache: CheckpointCache, fake_engine: FakeEngine
) -> None:
    assert cache.ls() == CacheSummary("file", cache.family_dir, (), 0, cache.max_bytes)
    claim, machine = populate(cache, fake_engine)
    summary = cache.ls()
    assert summary.kind == "file" and summary.variant_count == 1 and summary.max_bytes == 10**9
    (entry,) = summary.keys
    assert entry.key == KEY and entry.inputs is not None and entry.size_bytes > 0
    (variant,) = entry.variants
    assert variant.port == claim.port and variant.populated and not variant.corrupt
    assert variant.claimed_by == os.getpid() and variant.claim_live
    assert variant.machine_id == machine.id and variant.ref == claim.ref
    assert summary.store_bytes > 0 and summary.total_bytes == entry.size_bytes + summary.store_bytes
    as_json = json.dumps(summary.to_dict())
    assert KEY in as_json and "s3cret" not in as_json
    assert json.loads(as_json)["variant_count"] == 1
    assert json.dumps(cache.prune().to_dict())
    claim.release()


def test_find_key_resolves_unique_prefixes(cache: CheckpointCache, fake_engine: FakeEngine) -> None:
    c1, _ = populate(cache, fake_engine)
    c2, _ = populate(cache, fake_engine, key="ab" + "c" * 30)
    assert cache.find_key("aa") == KEY and cache.find_key("abc") == "ab" + "c" * 30
    with pytest.raises(CacheError, match="ambiguous"):
        cache.find_key("a")
    with pytest.raises(CacheError, match="no cache key"):
        cache.find_key("zz")
    c1.release()
    c2.release()


def test_export_copies_a_variant_and_warns_once(
    cache: CheckpointCache, fake_engine: FakeEngine, tmp_path: Path
) -> None:
    reset_warn_once()
    claim, _ = populate(cache, fake_engine)
    port = claim.port
    assert port is not None
    out = tmp_path / "out" / "pg.smolcheckpoint"
    with pytest.warns(SmoltestWarning, match="guest RAM"):
        written = cache.export(KEY, port, out, fake_engine)
    assert written == out.stat().st_size == cache.paths(KEY, port).checkpoint.stat().st_size
    assert out.read_bytes() == cache.paths(KEY, port).checkpoint.read_bytes()
    assert stat.S_IMODE(out.stat().st_mode) == 0o600, "an exported guest-RAM copy stays private"
    with pytest.raises(CacheError, match="already exists"):
        cache.export(KEY, None, out, fake_engine)
    out2 = tmp_path / "out2.smolcheckpoint"
    assert cache.export(KEY, None, out2, fake_engine) > 0, "port defaults to the newest variant"
    with pytest.raises(CacheError, match="no populated variant"):
        cache.export(KEY, port + 1, tmp_path / "x", fake_engine)
    with pytest.raises(CacheError, match="no populated variant"):
        cache.export(KEY2, None, tmp_path / "y", fake_engine)
    restored = fake_engine.restore_checkpoint(CheckpointRef("file", str(out)), "r", "local")
    restored.delete()
    claim.release()


# -- file locks -----------------------------------------------------------------------------


def test_file_lock_is_reentrant_shared_by_path_and_times_out(tmp_path: Path) -> None:
    lock = FileLock.for_path(tmp_path / "sub" / "x.lock", timeout_s=0.3, poll_interval_s=0.01)
    assert FileLock.for_path(tmp_path / "sub" / "x.lock") is lock
    assert not is_held(lock)
    with lock:
        assert is_held(lock) and stat.S_IMODE(lock.path.stat().st_mode) == 0o600
        with lock:  # same thread, nested
            assert is_held(lock)
        assert is_held(lock)
        with lock.shared_lock():  # exclusive satisfies a nested shared request
            assert is_held(lock) and not lock.is_shared
    assert not is_held(lock)
    with pytest.raises(CacheError, match="not held"):
        lock.release()

    acquired = threading.Event()
    release = threading.Event()

    def holder() -> None:
        other = FileLock(tmp_path / "sub" / "x.lock")  # a distinct instance, same file
        other.acquire()
        acquired.set()
        release.wait(5)
        other.release()

    t = threading.Thread(target=holder)
    t.start()
    assert acquired.wait(5)
    started = time.monotonic()
    with pytest.raises(LockTimeout) as info:
        lock.acquire()
    assert 0.2 <= time.monotonic() - started < 5
    assert info.value.code == "LOCK_TIMEOUT" and isinstance(info.value, TimeoutError)
    with pytest.raises(LockTimeout):
        lock.acquire(0)
    release.set()
    t.join(5)
    lock.acquire(timeout_s=5)
    lock.release()


def test_shared_locks_coexist_but_exclude_writers(tmp_path: Path) -> None:
    path = tmp_path / "s.lock"
    reader = FileLock(path, timeout_s=0.2, poll_interval_s=0.01)
    other_reader = FileLock(path, timeout_s=0.2, poll_interval_s=0.01)
    writer = FileLock(path, timeout_s=0.2, poll_interval_s=0.01)
    with reader.shared_lock():
        assert reader.is_shared
        with pytest.raises(CacheError, match="upgrade"):
            reader.acquire()
        with other_reader.shared_lock():
            assert other_reader.is_shared
        with pytest.raises(LockTimeout):
            writer.acquire()
    with writer, pytest.raises(LockTimeout):
        reader.acquire(shared=True)
    assert "FileLock(" in repr(writer)


def test_key_lock_held_across_claim_and_populate_is_reentrant(
    cache: CheckpointCache, fake_engine: FakeEngine
) -> None:
    port = pick_free_port()
    with cache.key_lock(KEY) as lock:
        assert is_held(lock)
        assert cache.claim_variant(KEY) is None
        claim = cache.reserve_new_variant(KEY, port)
        machine = fake_engine.create(mspec(port), "local")
        cache.populate(KEY, claim, machine, INPUTS)
        machine.delete()
        claim.release()
        assert is_held(lock)
    assert not is_held(lock)
    taken = cache.claim_variant(KEY)
    assert taken is not None and taken.port == port
    taken.release()


# -- two processes race for one key ---------------------------------------------------------


def _race_worker(root: str, key: str, port: int, barrier: Any, results: Any) -> None:
    """One contender: lock the key, restore if a variant exists, else cold boot and populate."""
    from smoltest.cache.store import CheckpointCache
    from smoltest.testing import FakeEngine
    from smoltest.transport.base import MachineSpec, PortMapping

    engine = FakeEngine()
    cache = CheckpointCache(root, max_bytes=10**9, lock_timeout_s=60, poll_interval_s=0.005)
    barrier.wait(timeout=30)
    with cache.key_lock(key):
        claim = cache.claim_variant(key, pinned_port=port)
        if claim is not None:
            via = "restore"
            handle = engine.restore_checkpoint(claim.ref, "restored", "local")  # type: ignore[arg-type]
            cache.touch(claim)
        else:
            via = "cold"
            claim = cache.reserve_new_variant(key, port)
            spec = MachineSpec(
                image="postgres:16",
                argv=("docker-entrypoint.sh", "postgres"),
                ports=(PortMapping(port, 5432),),
            )
            handle = engine.create(spec, "local")
            cache.populate(key, claim, handle, {"image": "postgres:16"}, engine=engine)
        handle.delete()
        claim.release()
    results.put((os.getpid(), via, len(engine.ops("create")), len(engine.ops("restore"))))


def test_two_processes_racing_for_one_key_boot_cold_exactly_once(tmp_path: Path) -> None:
    ctx = multiprocessing.get_context("spawn")
    barrier = ctx.Barrier(2)
    results = ctx.Queue()
    port = pick_free_port()
    root = str(tmp_path / "cache")
    procs = [
        ctx.Process(target=_race_worker, args=(root, KEY, port, barrier, results), daemon=True)
        for _ in range(2)
    ]
    for p in procs:
        p.start()
    outcomes = [results.get(timeout=60) for _ in procs]
    for p in procs:
        p.join(timeout=30)
        assert p.exitcode == 0
    vias = sorted(o[1] for o in outcomes)
    assert vias == ["cold", "restore"], outcomes
    assert sum(o[2] for o in outcomes) == 1, "exactly one cold boot"
    assert sum(o[3] for o in outcomes) == 1, "the other restored the checkpoint"
    cache = CheckpointCache(root, max_bytes=10**9)
    assert ports_of(cache) == {port}
    assert not cache.paths(KEY, port).claim.exists()


# -- the cloud index ------------------------------------------------------------------------


@pytest.fixture
def index(tmp_path: Path) -> CloudCheckpointIndex:
    return CloudCheckpointIndex(
        tmp_path / "cloud" / "idx" / "index.json", base_url="https://api.test", lock_timeout_s=5
    )


def test_cloud_index_map_operations(index: CloudCheckpointIndex) -> None:
    assert index.kind == "cloud" and index.get(KEY) is None and index.ls().keys == ()
    meta = index.set(KEY, "ckpt-7", inputs=INPUTS, sdk_version="fake", machine_id="m1")
    assert index.get(KEY) == "ckpt-7" and meta.ref == CheckpointRef("cloud", "ckpt-7")
    assert meta.port is None and meta.sdk_version == "fake"
    assert stat.S_IMODE(index.path.stat().st_mode) == 0o600
    raw = json.loads(index.path.read_text())
    assert raw["format"] == 1 and raw["base_url"] == "https://api.test"
    assert raw["entries"][KEY]["inputs"]["env"]["POSTGRES_PASSWORD"].startswith("sha256:")
    assert "s3cret" not in index.path.read_text()
    index.set(KEY, "ckpt-8")
    assert index.get(KEY) == "ckpt-8"
    assert json.loads(index.path.read_text())["entries"][KEY]["inputs"], "inputs kept"
    assert index.entries()[KEY].ref.locator == "ckpt-8"
    summary = index.ls()
    assert summary.kind == "cloud" and summary.variant_count == 1
    assert summary.keys[0].variants[0].ref == CheckpointRef("cloud", "ckpt-8")
    assert json.dumps(summary.to_dict())
    assert index.drop(KEY) and not index.drop(KEY) and index.get(KEY) is None
    index.set(KEY, "x")
    index.set(KEY2, "y")
    report = index.clear()
    assert set(report.removed_keys) == {KEY, KEY2} and not index.path.exists()
    assert index.clear() == PruneReport()
    with pytest.raises(NotSupportedError):
        index.export(KEY, None, "out", FakeEngine())


def test_cloud_index_backend_interface_with_fake_engine(
    index: CloudCheckpointIndex, fake_engine: FakeEngine
) -> None:
    assert isinstance(index, CheckpointBackend)
    assert index.claim_variant(KEY) is None
    claim = index.reserve_new_variant(KEY, None)
    assert claim.kind == "cloud" and claim.port is None and not claim.populated
    machine = fake_engine.create(mspec(None), "cloud")
    with index.key_lock(KEY):
        meta = index.populate(KEY, claim, machine, INPUTS, engine=fake_engine)
    machine.delete()
    assert meta.ref.kind == "cloud" and claim.ref == meta.ref and claim.meta is not None
    assert index.get(KEY) == meta.ref.locator and meta.sdk_version == "fake-1.22.2"
    assert meta.machine_id == machine.id
    found = index.claim_variant(KEY, pinned_port=12345)
    assert found is not None and found.ref == meta.ref and found.populated
    restored = fake_engine.restore_checkpoint(found.ref, "r", "cloud")
    restored.delete()
    index.touch(found)
    index.record_branch_ok(found, True)
    assert found.meta is not None and found.meta.branch_ok is True
    assert index.entries()[KEY].branch_ok is True
    assert index.entries()[KEY].last_used_at == found.meta.last_used_at
    found.release()
    index.invalidate(found)
    assert index.get(KEY) is None and index.claim_variant(KEY) is None
    with pytest.raises(CacheError, match="not"):
        index.populate(KEY2, claim, machine)


def test_cloud_index_prune_and_corruption(index: CloudCheckpointIndex) -> None:
    index.set(KEY, "old", sdk_version="fake-1.22.2")
    index.set(KEY2, "other-sdk", sdk_version="fake-0.1")
    raw = json.loads(index.path.read_text())
    raw["entries"][KEY]["last_used_at"] = 1.0
    index.path.write_text(json.dumps(raw))
    assert index.prune(keep_latest=0, max_bytes=0) == PruneReport(), "not applicable on cloud"
    report = index.prune(stale=True, engine=FakeEngine())
    assert report.removed_keys == (KEY2,)
    report = index.prune(older_than_s=60)
    assert report.removed_keys == (KEY,) and index.ls().keys == ()
    index.set(KEY, "c")
    index.path.write_text("{broken")
    with pytest.raises(CacheCorrupt):
        index.get(KEY)
    assert index.ls().keys == ()
    assert index.claim_variant(KEY) is None, "corrupt index is quarantined, not fatal"
    assert not index.path.exists() and index.path.with_name("index.json.corrupt").exists()
    index.set(KEY, "again")
    assert index.get(KEY) == "again"
    index.path.write_text(json.dumps({"format": 1, "entries": {KEY: {"format": 1}}}))
    assert index.entries() == {} and index.ls().keys[0].variants[0].corrupt
    assert index.prune(stale=True).removed_keys == (KEY,)


def test_existing_cache_root_keeps_its_permissions(tmp_path: Path) -> None:
    shared = tmp_path / "shared"
    shared.mkdir()
    shared.chmod(0o755)
    cache = CheckpointCache(shared, lock_timeout_s=10)
    cache.ensure_layout()
    assert stat.S_IMODE(shared.stat().st_mode) == 0o755, "a user-chosen root is left alone"
    assert stat.S_IMODE(cache.family_dir.stat().st_mode) == 0o700
    fresh = tmp_path / "fresh" / "cache"
    CheckpointCache(fresh, lock_timeout_s=10).ensure_layout()
    assert stat.S_IMODE(fresh.stat().st_mode) == 0o700, "a root smoltest creates is private"
