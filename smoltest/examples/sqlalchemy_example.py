"""Hand smoltest's connection URL to SQLAlchemy.

``PostgresMachine(driver="psycopg")`` makes ``get_connection_url()`` return a
``postgresql+psycopg://`` URL, which is the dialect string ``create_engine``
expects for psycopg 3. The default driver, ``psycopg2``, yields
``postgresql+psycopg2://`` exactly as testcontainers does.

Needs ``pip install sqlalchemy "smoltest[psycopg]"`` and a usable Smol target
(``smoltest doctor``).
"""

from __future__ import annotations

from sqlalchemy import create_engine, text

from smoltest import PostgresMachine


def main() -> None:
    """Create a table, insert a row and read it back through SQLAlchemy."""
    with PostgresMachine("postgres:16", driver="psycopg") as postgres:
        engine = create_engine(postgres.get_connection_url())
        try:
            with engine.begin() as conn:
                conn.execute(text("CREATE TABLE notes (id serial PRIMARY KEY, body text)"))
                conn.execute(text("INSERT INTO notes (body) VALUES (:body)"), {"body": "hello"})
                rows = conn.execute(text("SELECT id, body FROM notes ORDER BY id")).all()
            print(rows)
        finally:
            # Close pooled connections before the machine is deleted on __exit__.
            engine.dispose()


if __name__ == "__main__":
    main()
