"""Boot PostgreSQL in a Smol microVM and run one query, testcontainers style.

Run it on a host where ``smoltest doctor`` reports a usable target (Linux with
``/dev/kvm``, Apple Silicon, or ``SMOL_CLOUD_TOKEN`` set) with a host driver
installed: ``pip install "smoltest[psycopg]"``.

The first run boots cold and checkpoints the ready server into the cache; every
later run with the same image and settings restores that checkpoint instead.
"""

from __future__ import annotations

import psycopg

from smoltest import PostgresMachine


def main() -> None:
    """Start a machine, print how it booted and ``SELECT version()`` through psycopg."""
    with PostgresMachine("postgres:16") as postgres:
        info = postgres.boot_info
        if info is not None:
            print(f"booted via {info.via} in {info.elapsed_s:.2f}s on {info.target}")
        # driver=None gives a plain postgresql:// URL, which psycopg (and libpq) accept.
        with psycopg.connect(postgres.get_connection_url(driver=None)) as conn:
            row = conn.execute("SELECT version()").fetchone()
            print(row[0] if row is not None else "no row")


if __name__ == "__main__":
    main()
