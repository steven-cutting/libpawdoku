# Changelog

All notable changes to smoltest are recorded here. The format follows
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/) and the project uses
[Semantic Versioning](https://semver.org/).

## Unreleased

### Fixed

- The readiness strategy (default `pg_isready` or `waiting_for(...)`) now runs on every boot
  path: with `cache=False` / `--no-cache` / `SMOLTEST_DISABLE_CACHE` and on engines without
  checkpoints the machine was previously handed out, and seeded, without a single probe.
- A restored checkpoint whose server never becomes ready is invalidated and the boot falls
  through to the next variant or a cold boot instead of being retried forever; a tunnel
  failure after a restore still propagates without invalidating the variant.
- Waiting on another process's cold boot no longer fails the boot when the key lock wait
  runs out (the holder's image pull is unbounded): the boot proceeds cold without caching.
  The lock wait itself grew to `2 * ready_timeout_s + 120 s`.
- A failed capture of a seeded variant (for example a restored machine the SDK started
  non-branchable, or a full disk) no longer deletes a working, seeded machine and fails
  every seeded boot; the machine is handed out uncached and the half-written variant dropped.
- `KeyboardInterrupt` / `SystemExit` during a boot or a branch tears the machine and its
  tunnel down before propagating.
- The Smol SDK's refusals to branch an unbranchable source (the native "is not running
  forkable" error and the cloud 409 on `/branches`) are classified as `NotSupportedError`, so
  `PostgresGolden.branch()` falls back to fresh machines as documented instead of erroring on
  every test.
- `capture_logs=True` writes the server log to `/tmp/postgresql.log` instead of
  `/var/log/postgresql/postgresql.log`, which the `postgres:*-alpine` images lack (the server
  failed to start and the boot ended in a bare `ReadinessTimeout`). Cached checkpoints for
  `capture_logs=True` specs are invalidated by the argv change.
- `POSTGRES_USER` / `POSTGRES_PASSWORD` / `POSTGRES_DB` given through `with_env`, `with_envs`,
  the marker's `env=` or `--env` are folded into the spec's credential fields, so the
  connection URL, `psql()`, seeds and probes match the server.
- Readiness probes and housekeeping execs are bounded (`EXEC_PROBE_TIMEOUT_S`, 10 s) so a
  stalled guest agent ends in `ReadinessTimeout` rather than a hang.
- `ExecResult` is indexable (`result[0]`, `result[1]`) like docker's exec result tuple.
- `with_startup_timeout` / `with_poll_interval` (strategies) and
  `PostgresMachine.with_startup_timeout` accept `timedelta` values.
- The cache root chosen by the user keeps its permissions when it already exists; only
  directories smoltest creates are made `0o700`. `smoltest cache export` writes the copy
  as `0o600`.
- On the cloud target the missing `websockets` extra is reported before a billed machine is
  created.

### Added

- `Settings.exec_timeout_s` / `SMOLTEST_EXEC_TIMEOUT` (default 600 s): the timeout the Smol
  engine applies to in-guest commands run without one (seeds, `psql()`, `exec()`), on both
  targets; previously a cloud exec was cut off by the SDK's 30 s HTTP read timeout and a
  local one was unbounded. `PostgresMachine.psql(..., timeout_s=)` overrides it per call.

### Changed

- README: the cloud `auto_stop` / `ttl` safety net is documented as applying only to machines
  smoltest creates; restored and branched cloud machines carry no TTL smoltest can set.

## 0.1.0 - 2026-10-03

First release: PostgreSQL test databases in Smol Machines microVMs behind the
testcontainers-python `PostgresContainer` API.

### Added

- `PostgresMachine` (alias `PostgresContainer`) with testcontainers' constructor,
  builders (`with_env`, `with_envs`, `with_command`, `with_name`, `with_exposed_ports`,
  `with_bind_ports`, `waiting_for`, `with_startup_timeout`, `with_kwargs`), lifecycle
  (`start`, `stop`, context manager), `get_connection_url`, `get_container_host_ip`,
  `get_exposed_port`, `exec` (tuple-unpackable `ExecResult`) and `get_logs`; plus
  microVM-only `with_init_sql`, `with_seed`, `with_resources`, `psql`, `branch`,
  `checkpoint`, `boot_info` and `get_machine`. `AsyncPostgresMachine` wraps it for asyncio.
- `Seed.from_sql`, `Seed.from_sql_files` and `Seed.from_callable`: deterministic database
  population identified by a key, applied once to a golden machine and cached as a checkpoint.
- The boot ladder (`smoltest.boot.strategy.boot_postgres`): branch a golden, else restore a
  cached checkpoint (seeded key first, then base key), else cold boot and populate the cache;
  guest clock resync after a restore; readiness wait; teardown on any failure with a
  `BootError` carrying the failing stage.
- `PostgresGolden` and the process-wide `GoldenRegistry`: boot once per (spec, seed), hand
  out copy-on-write branches on fresh host ports, fall back to fresh machines when the
  engine cannot branch, close children before parents at exit.
- The local checkpoint cache (`smoltest.cache`): canonical cache keys with a host signature,
  per-key and store-wide file locks, port variants with pid claims and bind probes, a commit
  marker written last, LRU eviction to a byte budget, prune by age / count / size /
  staleness, export, and a cloud checkpoint index keyed by API base URL.
- Readiness strategies mirroring testcontainers: `PgIsReadyWaitStrategy` (default),
  `PortWaitStrategy`, `ExecWaitStrategy`, `SqlWaitStrategy` (host driver or in-guest `psql`),
  `LogMessageWaitStrategy` (with `capture_logs=True`), `CompositeWaitStrategy`, and
  `with_startup_timeout` / `with_poll_interval`.
- `Settings` with the `SMOLTEST_*` environment variables, `TC_MAX_TRIES` /
  `TC_POOLING_INTERVAL` compatibility and automatic target resolution (`SMOL_CLOUD_TOKEN`
  selects the cloud, else a usable local engine, else `TargetUnavailable`).
- The Smol SDK transport (`smoltest.transport.smol_engine`) over `smolmachines` 1.22 with
  lazy imports, SDK error translation and a threaded `TunnelBridge` for cloud machines; the
  `Engine` / `MachineHandle` protocols it implements; and `smoltest.testing.FakeEngine`, an
  in-process fake with real loopback listeners for test suites that must not need KVM.
- The pytest plugin (`pytest11` entry point): `postgres`, `postgres_url`,
  `postgres_golden_url`, `smoltest_postgres_golden`, `smoltest_postgres_template`,
  `smoltest_seed_postgres`, `smoltest_settings` and `smoltest_engine` fixtures; the
  `@pytest.mark.smoltest(image=, env=, isolation=, seed=)` marker; `branch` / `fresh` /
  `shared` isolation; `--smoltest-*` options and `smoltest_*` ini keys; a report header and
  terminal summary with boot paths and branch latency.
- The `smoltest` CLI: `doctor [--json]`, `warm postgres`, `cache ls|prune|clear|export`
  and `run postgres`, with exit status 3 when no Smol target is available.
- Examples (`examples/`), a GitHub Actions workflow (lint, unit matrix on Python
  3.10–3.13, fake benchmark smoke test, gated local and cloud e2e jobs) and this changelog.

### Known gaps

- Only PostgreSQL is supported.
- Real-machine benchmarks have not been recorded yet: the release was assembled on a host
  without `/dev/kvm`; the boot figures in the README are Smol's own measurements.
- Not yet published to PyPI; install from a checkout.
