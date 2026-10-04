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
- `PostgresMachine(port=N)` with `N != 5432` booted a server still listening on 5432 and ended
  in a startup timeout; the argv now sets `-c port=N` and every in-guest client (`psql()`, SQL
  seeds, `pg_isready`, the in-guest `SqlWaitStrategy` probe, the pre-checkpoint backend
  termination) passes `-p N`. Default-port argvs are unchanged.
- `Settings.exec_timeout_s` reached only cold machines; restored and branched handles got the
  600 s engine default. `Engine.restore_checkpoint(..., exec_timeout_s=)` now carries it and
  branches inherit it.
- Children of a golden (`branch()` and `fresh()`) and `PostgresMachine.branch()` now carry the
  seed, so a `stop()` / `start()` of such a child boots seeded (restores the seeded checkpoint
  or re-seeds) instead of unseeded.
- `TunnelBridge`: a worker thread abandoned by `close(timeout_s)` can no longer corrupt a
  bridge reopened afterwards; its late error and endpoint writes land in an orphaned run.
- The exit reaper forgot nothing until exit: completed and detached registrations are now
  dropped immediately (`Reaper.detach(token)`), so memory no longer grows per test and the
  exit sweep is linear.
- The cloud checkpoint index only drops or rewrites an entry while it still records the
  checkpoint id the claim was taken on, so a stale claim can neither erase a replacement
  another process recorded nor resurrect the old id.
- A local variant whose meta file does not describe it (another key, port or checkpoint path,
  or a cloud ref) is treated as corrupt and invalidated instead of being restored. `populate`
  records the variant's own checkpoint path in the meta (and refuses an engine that wrote the
  checkpoint anywhere else), so that check never depends on how the SDK spells the path.
- A restored variant is invalidated when readiness times out or when the readiness probe
  fails with any other smoltest error (a guest agent that refuses every command, a machine
  that died on resume); a wait strategy refusing the machine (`NotSupportedError` or
  `InvalidConfig`, for example `LogMessageWaitStrategy` without `capture_logs=True`) surfaces
  as `BootError` with the cause and leaves the cached variant intact.
- `Settings.from_env(mapping)` derives the default cache directory from the supplied mapping
  instead of the process environment; explicit `cache_dir` overrides still win.
- `GoldenRegistry` keyed goldens by checkpoint key and seed only, so a template pinned to
  another host port, or asking for another driver, name, wait strategy or settings (branching
  policy, exec timeout, ...), was handed the first golden. The registry identity now carries
  those runtime options too; checkpoint reuse is unchanged.
- `PostgresGolden.fresh()` now prunes stopped children like `branch()` does, so the fresh
  isolation mode no longer retains every finished test's machine until session teardown.
- `PostgresMachine.checkpoint(path)` makes the local checkpoint private (`0o600` files, `0o700`
  directories) before returning it, as the cache and `cache export` already did; the engine
  writes it with the process umask.
- `FileLock.for_path` registered instances under the resolved path but built them from the path
  as given, so a relative lock path locked a different file after a `chdir`. The shared
  instance is now built from the resolved path.
- `smoltest run postgres` could not be interrupted while the machine was still booting (the
  signal handler only set an event the boot never checked). `SIGINT`/`SIGTERM` during startup
  now abort the boot, which deletes whatever it had created, and exit 1.
- `GoldenRegistry` also keys goldens by the engine instance, so templates that inject
  different engines never share a golden.
- `Settings` and the wait-strategy builders reject non-finite timeouts (`nan`, `inf`), which would
  otherwise make a readiness wait poll forever.
- The cloud checkpoint index validates each record against its own key (embedded key, no port,
  cloud ref) before claiming it and drops a mismatched record instead of restoring another key's
  checkpoint, mirroring the local metadata check.

### Added

- `Settings.exec_timeout_s` / `SMOLTEST_EXEC_TIMEOUT` (default 600 s): the timeout the Smol
  engine applies to in-guest commands run without one (seeds, `psql()`, `exec()`), on both
  targets; previously a cloud exec was cut off by the SDK's 30 s HTTP read timeout and a
  local one was unbounded. `PostgresMachine.psql(..., timeout_s=)` overrides it per call.

### Changed

- Cache keys changed (key format 2): the pinned host ports of extra guest ports are now part of
  the key, so a restore can no longer bind a different extra host port than the one asked for.
  Existing checkpoints miss once and are re-created on the next boot.
- `smoltest warm` table: the `pause ms` column is now `populate ms`; it reports the populate
  time (checkpoint write and cache bookkeeping), which is what it always measured.
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
