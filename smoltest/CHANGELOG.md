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
- A process that forks while holding a `FileLock` no longer hands the child a lock it believes
  it holds: the child's inherited descriptors are closed without unlocking the parent, its hold
  state is reset, and its first `acquire()` takes its own `flock`.
- `make_private` (cache variants, `cache export`, `PostgresMachine.checkpoint()`) raises
  `CacheError` with code `CHECKPOINT_PERMISSIONS` when a `chmod` fails or does not take effect,
  instead of silently leaving a copy of guest RAM with the umask's permissions; the checkpoint
  that could not be protected is removed, and `checkpoint()` raises `SmoltestError` with the same
  code. A removal that fails in turn is reported with that code too (the path is still readable)
  rather than as a bare `OSError` hiding the permission failure. Group and world permission
  bits are what the check rejects, so a mount with a fixed owner-only mask still works.
- `GoldenRegistry` compares engines by identity without hashing them: an unhashable engine (a
  mutable dataclass) no longer fails `get_or_boot()` with `TypeError`, and two value-equal engine
  instances no longer share a golden. `GoldenKey.engine` is now an `EngineIdentity` wrapper (the
  engine itself is `key.engine.engine`).
- `get_connection_url()` percent-encodes the user and database names as well as the password,
  so `/`, `@`, `#` or `?` in them no longer corrupt the URL.
- `WaitStrategy.resolve()` / `wait_until_ready(target, timeout_s, poll_s)` reject non-finite
  direct arguments (`inf`, `nan`) with `InvalidConfig` instead of polling forever.
- `TunnelBridge`: a stop requested while the worker thread was still creating its event loop
  (the connect timeout beat it) is remembered and honoured once the task exists, so no tunnel is
  left open on a run the bridge has already discarded.
- `smoltest warm` exits 1, after printing its table, when a requested variant is not in the
  cache populated and intact afterwards (a boot that continued uncached after a busy key lock
  or a failed capture); previously it exited 0 and deleted the machines. The two seeded
  uncached paths now log at WARNING, so the CLI shows the cause without `-v`.
- The exit reaper pins weakref's own exit hook ahead of its own. When the reaper created the
  process's first `weakref.finalize`, atexit's LIFO order ran weakref's hook first, which
  disables every finalizer, so machines relying on exit cleanup were left running.
- A forked child no longer inherits the parent's cleanups: the reaper's registrations and the
  cache claims' finalizers are detached in the child, so a garbage collection or a normal exit
  there cannot delete the parent's machines or remove its claim files. The child's own
  registrations work as before.
- Locks returned by `FileLock.for_path` share the hold (re-entrancy across cache objects) but
  carry each caller's own timeout and poll interval; previously the first caller's settings
  stuck to the path, so a later caller with a shorter timeout waited the longer one (or forever).
- A fixed PostgreSQL host port (`with_bind_ports(5432, N)`, `smoltest run postgres --port N`)
  is refused on the cloud target with `NotSupportedError` (the CLI exits 1 before booting)
  instead of being silently dropped while the tunnel picks another port.
- A checkpoint the cache rejects after the engine wrote it (a ref that is not the requested
  file, a file that is missing or elsewhere) is removed instead of staying behind with the
  umask's permissions. A different path the engine reported is removed only when it is a stray
  inside the variant's own key directory; the dedup store, other variants' files and anything
  outside the cache are left in place with a warning, never deleted on the engine's word. An
  interrupt during that clean-up still propagates as the interrupt.
- `SqlWaitStrategy` builds the asyncpg DSN without `connect_timeout`: asyncpg forwards unknown
  URI parameters to the server as session settings, which rejected every probe until the
  readiness timeout. The libpq-based drivers keep the parameter; asyncpg gets `timeout=`.
- `inputs.json`, the cloud index and `smoltest cache ls --json` hash every guest environment
  value except the PostgreSQL image's documented non-secret settings (`POSTGRES_USER`,
  `POSTGRES_DB`, `POSTGRES_INITDB_ARGS`, `TZ`, `LANG`, `PG*`, ...); previously only names
  containing `PASSWORD`, `PASSWD`, `SECRET` or `TOKEN` were redacted, so an `API_KEY` or
  `DATABASE_URL` passed through `with_env` was persisted in clear. Strings in lists (the argv of
  `with_command`, such as `-c primary_conninfo=... password=...`) are hashed when they name a
  secret, and the secret names now also cover `PASSPHRASE`, `CREDENTIAL`, `API_KEY` and
  `PRIVATE_KEY`.
- `PostgresMachine.stop()` raises when the engine refuses to delete the machine, and keeps
  everything needed to try again: the machine stays registered and `is_running`, its cache claim
  stays held and its exit-time cleanup stays armed, so a second `stop()` (or interpreter exit)
  retries. Previously the error was only logged and a still-running (on the cloud, still billed)
  machine was forgotten. Cleanup at exit, on garbage collection and on a failed boot stays best
  effort.
- `FileLock.release()` from a thread that does not hold the lock raises `CacheError` before the
  hold is touched; it used to drop the owner's `flock` first, letting another process into the
  owner's critical section.
- An unusable lock path (an unwritable directory, or a directory where the lock file belongs)
  raises `CacheError` with code `LOCK_PATH`, so `smoltest cache prune` / `clear` print an error and
  exit 1 instead of a traceback.
- `warn_once` gets a fresh lock in a forked child; a parent thread holding it at the fork left
  every later warning in the child (the golden's branch-fallback warning among them) hanging.
- `SqlWaitStrategy` and `PortWaitStrategy` reject a non-finite or non-positive
  `connect_timeout_s` with `InvalidConfig` (they also accept a `timedelta`): an infinite
  per-attempt timeout kept a stalled probe from ever returning to the readiness deadline.

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
- `pytest>=7` is a runtime dependency: the plugin is loaded through the `pytest11` entry point
  whenever smoltest is installed and needs `pytest.StashKey`, so declaring it keeps an older
  pytest from aborting at startup while it loads the plugin.

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
