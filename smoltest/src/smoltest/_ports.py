"""Loopback port selection outside the kernel's ephemeral range."""

from __future__ import annotations

import random
import socket
from collections.abc import Iterable
from dataclasses import dataclass
from pathlib import Path

from .errors import SmoltestError

LOOPBACK = "127.0.0.1"
PORT_RANGE: tuple[int, int] = (20000, 29999)
"""Inclusive range smoltest picks host ports from."""

_EPHEMERAL_FILE = Path("/proc/sys/net/ipv4/ip_local_port_range")
_ATTEMPTS = 200


@dataclass(frozen=True)
class HostEndpoint:
    """A host-side TCP endpoint that reaches a guest port."""

    host: str
    port: int

    def __str__(self) -> str:
        return f"{self.host}:{self.port}"


def ephemeral_range(path: Path = _EPHEMERAL_FILE) -> tuple[int, int] | None:
    """Return the kernel's ephemeral port range when readable, else ``None``."""
    try:
        low, high = path.read_text().split()
        return int(low), int(high)
    except (OSError, ValueError):
        return None


def is_port_free(port: int, host: str = LOOPBACK) -> bool:
    """Bind-probe ``host:port``; ``True`` when nothing listens there."""
    with socket.socket(socket.AF_INET, socket.SOCK_STREAM) as sock:
        sock.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)
        try:
            sock.bind((host, port))
        except OSError:
            return False
        return True


def candidate_ports(
    port_range: tuple[int, int] = PORT_RANGE,
    ephemeral: tuple[int, int] | None = None,
) -> tuple[int, int]:
    """Shrink ``port_range`` so it does not overlap the ephemeral range."""
    low, high = port_range
    if ephemeral is None:
        return low, high
    e_low, e_high = ephemeral
    if e_high < low or e_low > high:
        return low, high
    if e_low > low:
        return low, min(high, e_low - 1)
    if e_high < high:
        return max(low, e_high + 1), high
    raise SmoltestError(
        f"ephemeral port range {e_low}-{e_high} covers the whole smoltest range {low}-{high}"
    )


def pick_free_port(
    *,
    exclude: Iterable[int] = (),
    host: str = LOOPBACK,
    rng: random.Random | None = None,
) -> int:
    """Return a free loopback port in :data:`PORT_RANGE` outside the ephemeral range.

    The pick is random so concurrent processes rarely collide; callers still
    bind promptly and treat ``EADDRINUSE`` as a retryable condition.
    """
    low, high = candidate_ports(ephemeral=ephemeral_range())
    excluded = set(exclude)
    chooser = rng or random
    for _ in range(_ATTEMPTS):
        port = chooser.randint(low, high)
        if port in excluded:
            continue
        if is_port_free(port, host):
            return port
    raise SmoltestError(f"no free port found in {low}-{high} after {_ATTEMPTS} attempts")


__all__ = [
    "LOOPBACK",
    "PORT_RANGE",
    "HostEndpoint",
    "candidate_ports",
    "ephemeral_range",
    "is_port_free",
    "pick_free_port",
]
