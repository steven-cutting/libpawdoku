"""Seed the golden database once per session; every test then gets a branch of it.

Copy the ``smoltest_seed_postgres`` fixture into your ``conftest.py``: the
smoltest pytest plugin (registered through the ``pytest11`` entry point, so
nothing needs enabling) uses it as the override point for seeding. The seeded
state is checkpointed under the seed's content hash, so a later session
restores it instead of running the SQL again, and each ``postgres`` branch
starts from the seeded schema with its own copy-on-write disk.

The fixture reads ``schema.sql`` next to this file; create it with, say::

    CREATE TABLE users (id serial PRIMARY KEY, name text NOT NULL);

Then run ``pytest examples/conftest_seed_example.py`` on a host where
``smoltest doctor`` reports a usable target, with ``psycopg`` installed.
"""

from __future__ import annotations

from pathlib import Path

import psycopg
import pytest

from smoltest import Seed

SCHEMA = Path(__file__).with_name("schema.sql")


@pytest.fixture(scope="session")
def smoltest_seed_postgres() -> Seed:
    """Run ``schema.sql`` once on the golden machine (cached by content hash)."""
    # Several files run in order: Seed.from_sql_files("schema.sql", "fixtures.sql").
    # Migrations or fixtures in code: Seed.from_callable(run_migrations, key=alembic_head).
    return Seed.from_sql_files(SCHEMA)


def _count_users(url: str) -> int:
    with psycopg.connect(url) as conn:
        row = conn.execute("SELECT count(*) FROM users").fetchone()
    return int(row[0]) if row is not None else 0


def test_seeded_schema_is_present(postgres_url: str) -> None:
    """The branch starts with the seeded schema; this test's insert is its own."""
    with psycopg.connect(postgres_url) as conn:
        conn.execute("INSERT INTO users (name) VALUES ('ada')")
    assert _count_users(postgres_url) == 1


def test_writes_do_not_leak_between_tests(postgres_url: str) -> None:
    """A new branch of the golden: the previous test's row is not here."""
    assert _count_users(postgres_url) == 0
