# Agent instructions for smoltest

`smoltest` is a **standalone** Python project inside the `libpawdoku`
repository, carved out by the root `AGENTS.md` ("Standalone subprojects"). The
root's safety and authority rules apply here unchanged; its engine workflow
(`just`, `cargo`, `pixi`, `prek`, the Allium specifications) does not, and this
project's checks are the `uv` commands below plus
`.github/workflows/smoltest.yml`. The hook gate (`.pre-commit-config.yaml`),
taplo, markdownlint, typos and lychee each exclude `smoltest/` explicitly.
Never run `just`, `cargo`, `pixi` or `prek` for this project, and never touch
files outside this directory on its behalf.

## What it is

A testcontainers-python replacement that boots PostgreSQL inside Smol Machines
microVMs (PyPI `smolmachines`, import name `smol`). The README is the user
manual; this file is for working on the code.

Layout of `src/smoltest/`:

| module | role |
|---|---|
| `postgres.py` | `PostgresMachine` / `PostgresContainer`, `AsyncPostgresMachine`, `Seed`, `ExecResult`: the user-facing facade |
| `boot/spec.py`, `boot/strategy.py`, `boot/golden.py` | the machine spec, the branch → restore → cold ladder (`boot_postgres`, `branch_from`), goldens and the registry |
| `cache/` | cache keys, file locks, the local checkpoint store with port variants, the cloud index |
| `wait/strategies.py` | readiness strategies (`pg_isready` by default) |
| `transport/base.py` | the `Engine` / `MachineHandle` / `Bridge` protocols everything else codes against |
| `transport/smol_engine.py`, `transport/tunnel.py` | the only modules that touch the Smol SDK, lazily |
| `testing.py` | `FakeEngine`: real loopback listeners, exec dispatch table, JSON "checkpoints", fault injection |
| `config.py`, `errors.py`, `_ports.py`, `_lifecycle.py`, `_log.py` | settings and `SMOLTEST_*` variables, the error hierarchy, host port picking, the exit-time reaper, logging |
| `pytest_plugin.py`, `cli/` | the `pytest11` plugin and the `smoltest` command |

## Setup and checks

```sh
uv sync --all-extras --group dev
uv run ruff check . && uv run ruff format --check .
uv run mypy                                        # strict, over src/
uv run pytest -q                                   # unit suite on the FakeEngine, no KVM needed
uv run python benchmarks/bench_boot.py --fake      # benchmark script smoke test
uv run pytest -m local_e2e -q                      # real machines; needs /dev/kvm or Apple Silicon
uv run pytest -m cloud_e2e -q                      # real machines on Smol Cloud; needs SMOL_CLOUD_TOKEN
```

Run all four of the first block before handing back. The e2e markers are
excluded by `addopts`; a sandbox without `/dev/kvm`, Docker or a cloud token
cannot run them, and `smoltest doctor` says so.

## Conventions

- Python >= 3.10 syntax with `from __future__ import annotations`, complete
  type hints (`mypy --strict`), ruff rules from `pyproject.toml`, concise
  docstrings on public items, no runtime dependencies beyond `pyproject.toml`.
- Unit tests never import `smol`; only `tests/unit/test_smol_import_smoke.py`
  does, and it skips when the SDK is absent. Tests select the fake through
  `SMOLTEST_ENGINE` (set in `tests/conftest.py`) and use the `fake_engine`
  fixture, which fails a test that leaks a machine.
- Everything Smol-specific stays behind `transport/base.py`; add behaviour to
  the fake and the real engine together.
- Behaviour, its test and its documentation (README, CHANGELOG) land in the
  same change. Do not commit, push, tag or publish unless asked.

## Checkpoints are sensitive

A `.smolcheckpoint` is a copy of **guest RAM**, credentials and data included.
They live in the user cache directory (`~/.cache/smoltest` by default, or
wherever `SMOLTEST_CACHE_DIR` points, such as a gitignored `.smoltest-cache/`),
with `0o700` directories and `0o600` files. Never commit, upload or attach one,
and never print the contents of `SMOL_CLOUD_TOKEN`.
