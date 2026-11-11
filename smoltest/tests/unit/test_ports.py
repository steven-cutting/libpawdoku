"""Port picking stays outside the ephemeral range and bind-probes."""

from __future__ import annotations

import random
import socket
from pathlib import Path

import pytest

from smoltest._ports import (
    PORT_RANGE,
    HostEndpoint,
    candidate_ports,
    ephemeral_range,
    is_port_free,
    pick_free_port,
)
from smoltest.errors import SmoltestError


def test_host_endpoint_is_frozen_and_prints() -> None:
    ep = HostEndpoint("127.0.0.1", 5432)
    assert str(ep) == "127.0.0.1:5432"
    with pytest.raises(AttributeError):
        ep.port = 1  # type: ignore[misc]


def test_pick_free_port_in_range_and_free() -> None:
    port = pick_free_port()
    low, high = PORT_RANGE
    assert low <= port <= high
    assert is_port_free(port)
    eph = ephemeral_range()
    if eph is not None:
        assert not (eph[0] <= port <= eph[1])


def test_pick_free_port_skips_busy_and_excluded() -> None:
    rng = random.Random(1)
    with socket.socket() as busy:
        busy.bind(("127.0.0.1", 0))
        busy.listen(1)
        busy_port = busy.getsockname()[1]
        assert not is_port_free(busy_port)
        first = pick_free_port(rng=random.Random(1))
        assert pick_free_port(exclude=[first], rng=rng) != first


def test_pick_free_port_gives_up(monkeypatch: pytest.MonkeyPatch) -> None:
    monkeypatch.setattr("smoltest._ports.is_port_free", lambda *_: False)
    with pytest.raises(SmoltestError, match="no free port"):
        pick_free_port()


def test_ephemeral_range_reads_proc_file(tmp_path: Path) -> None:
    f = tmp_path / "range"
    f.write_text("32768\t60999\n")
    assert ephemeral_range(f) == (32768, 60999)
    assert ephemeral_range(tmp_path / "missing") is None
    f.write_text("garbage")
    assert ephemeral_range(f) is None


@pytest.mark.parametrize(
    ("ephemeral", "expected"),
    [
        (None, (20000, 29999)),
        ((32768, 60999), (20000, 29999)),
        ((25000, 60999), (20000, 24999)),
        ((1024, 25000), (25001, 29999)),
        ((21000, 22000), (20000, 20999)),
    ],
)
def test_candidate_ports_carves_out_ephemeral(
    ephemeral: tuple[int, int] | None, expected: tuple[int, int]
) -> None:
    assert candidate_ports(ephemeral=ephemeral) == expected


def test_candidate_ports_rejects_total_overlap() -> None:
    with pytest.raises(SmoltestError):
        candidate_ports(ephemeral=(1024, 65535))
