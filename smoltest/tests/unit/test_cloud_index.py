"""The cloud index only drops or rewrites an entry while it still records the claimed id."""

from __future__ import annotations

import json
from pathlib import Path

import pytest

from smoltest.cache.cloud_index import CloudCheckpointIndex
from smoltest.cache.store import VariantClaim

KEY = "a" * 32


@pytest.fixture
def index(tmp_path: Path) -> CloudCheckpointIndex:
    return CloudCheckpointIndex(
        tmp_path / "cloud" / "idx" / "index.json", base_url="https://api.test", lock_timeout_s=5
    )


def claim_on(index: CloudCheckpointIndex, checkpoint_id: str) -> VariantClaim:
    index.set(KEY, checkpoint_id, sdk_version="fake")
    claim = index.claim_variant(KEY)
    assert claim is not None and claim.ref is not None
    assert claim.ref.locator == checkpoint_id
    return claim


def test_stale_invalidate_leaves_the_replacement_in_place(index: CloudCheckpointIndex) -> None:
    stale = claim_on(index, "ckpt-X")
    index.set(KEY, "ckpt-Y")  # another process replaced the entry meanwhile
    index.invalidate(stale)
    assert index.get(KEY) == "ckpt-Y", "a stale claim must not erase the replacement"
    assert stale.released


def test_stale_touch_and_branch_ok_do_not_resurrect_the_old_id(
    index: CloudCheckpointIndex,
) -> None:
    stale = claim_on(index, "ckpt-X")
    assert stale.meta is not None
    index.set(KEY, "ckpt-Y")
    before = json.loads(index.path.read_text())["entries"][KEY]
    index.touch(stale)
    index.record_branch_ok(stale, True)
    after = json.loads(index.path.read_text())["entries"][KEY]
    assert after == before, "a stale claim rewrites nothing"
    assert index.get(KEY) == "ckpt-Y"
    assert stale.meta.ref.locator == "ckpt-X" and stale.meta.branch_ok is None
    index.drop(KEY)
    index.touch(stale)  # no entry at all: still a no-op
    assert index.get(KEY) is None


def test_fresh_claim_still_touches_records_and_invalidates(index: CloudCheckpointIndex) -> None:
    claim = claim_on(index, "ckpt-X")
    assert claim.meta is not None
    raw = json.loads(index.path.read_text())
    raw["entries"][KEY]["last_used_at"] = 1.0
    index.path.write_text(json.dumps(raw))
    index.touch(claim)
    assert claim.meta.last_used_at > 1.0
    assert index.entries()[KEY].last_used_at == claim.meta.last_used_at
    index.record_branch_ok(claim, False)
    assert claim.meta.branch_ok is False and index.entries()[KEY].branch_ok is False
    index.invalidate(claim)
    assert index.get(KEY) is None and claim.released


def test_rewrite_merges_into_the_current_entry_not_the_claims_copy(
    index: CloudCheckpointIndex,
) -> None:
    first = claim_on(index, "ckpt-X")
    second = index.claim_variant(KEY)
    assert second is not None and second.meta is not None
    index.record_branch_ok(first, True)
    index.touch(second)  # second's meta still says branch_ok=None
    current = index.entries()[KEY]
    assert current.branch_ok is True, "touch merges into the current entry"
    assert current.ref.locator == "ckpt-X"
    assert second.meta.branch_ok is True, "the claim learns the merged result"
    assert json.loads(index.path.read_text())["entries"][KEY]["sdk_version"] == "fake"


def test_unpopulated_claim_invalidate_drops_nothing(index: CloudCheckpointIndex) -> None:
    index.set(KEY, "ckpt-Y")
    reserved = index.reserve_new_variant(KEY, None)
    index.invalidate(reserved)
    assert index.get(KEY) == "ckpt-Y" and reserved.released
    index.touch(reserved)
    assert index.get(KEY) == "ckpt-Y"


KEY2 = "b" * 32


def rewrite_entries(index: CloudCheckpointIndex, mutate) -> None:  # type: ignore[no-untyped-def]
    raw = json.loads(index.path.read_text())
    mutate(raw["entries"])
    index.path.write_text(json.dumps(raw))


def test_entry_recorded_under_another_key_is_dropped_not_claimed(
    index: CloudCheckpointIndex,
) -> None:
    index.set(KEY2, "ckpt-other", sdk_version="fake")
    index.set(KEY, "ckpt-mine", sdk_version="fake")
    # KEY's record is replaced by a copy of KEY2's: its embedded key says KEY2.
    rewrite_entries(index, lambda entries: entries.__setitem__(KEY, dict(entries[KEY2])))
    assert index.claim_variant(KEY) is None
    assert index.get(KEY) is None, "the swapped record is dropped"
    assert index.get(KEY2) == "ckpt-other", "the genuine record survives"
    assert KEY not in index.entries() and KEY2 in index.entries()


@pytest.mark.parametrize(
    "mutate",
    [
        lambda entries: entries[KEY].__setitem__("port", 20808),
        lambda entries: entries[KEY]["ref"].__setitem__("kind", "file"),
    ],
    ids=["port", "file-ref"],
)
def test_entry_with_a_port_or_a_file_ref_is_dropped(index: CloudCheckpointIndex, mutate) -> None:  # type: ignore[no-untyped-def]
    index.set(KEY, "ckpt-mine", sdk_version="fake")
    rewrite_entries(index, mutate)
    assert index.claim_variant(KEY) is None
    assert index.get(KEY) is None
