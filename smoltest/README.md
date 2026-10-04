# smoltest

PostgreSQL test databases that boot in [Smol Machines](https://smolmachines.com)
microVMs instead of Docker containers, behind the `PostgresContainer` API of
testcontainers-python. The first boot of an image is a cold start that gets
checkpointed with the server already running; every later boot restores that
checkpoint (Smol measures a restore at about 0.3 s), and inside one test
session each test gets its own copy-on-write **branch** of a golden machine in
under 200 ms. No Docker daemon: the engine ships inside the `smolmachines`
wheel and runs on Linux with `/dev/kvm` or on Apple Silicon, or on Smol Cloud
with a token.

```python
from smoltest import PostgresContainer

with PostgresContainer("postgres:16") as postgres:
    url = postgres.get_connection_url()  # postgresql+psycopg2://test:test@127.0.0.1:2xxxx/test
```

## Install

```sh
pip install smoltest                 # the library, the pytest plugin and the `smoltest` CLI
pip install "smoltest[psycopg]"      # + psycopg 3 for host-side connections and SQL readiness probes
pip install "smoltest[cloud]"        # + websockets, needed for the TCP tunnel to Smol Cloud machines
```

Until the package is on PyPI, install it from a checkout: `pip install /path/to/smoltest`.
pytest 7 or newer is installed with it: the plugin registers itself through the `pytest11`
entry point and needs pytest's `StashKey`, so an older pytest would abort at startup while
loading the plugin.

Requirements: Python 3.10 or newer, and one of

- **local**: Linux with `/dev/kvm` (x86_64 or aarch64, glibc 2.34+), or macOS on Apple Silicon.
  Both are served by the `smolmachines` wheel; there is nothing else to install or run.
- **cloud**: `SMOL_CLOUD_TOKEN` in the environment (and the `cloud` extra). Machines then run on
  Smol Cloud and PostgreSQL is reached through a loopback tunnel.

`smoltest doctor` tells you which one you have.

## Quickstart

The same shape as testcontainers' `PostgresContainer` example:

```python
import sqlalchemy

from smoltest import PostgresContainer  # an alias of smoltest.PostgresMachine

with PostgresContainer("postgres:16", driver="psycopg") as postgres:
    engine = sqlalchemy.create_engine(postgres.get_connection_url())
    with engine.begin() as connection:
        (version,) = connection.execute(sqlalchemy.text("select version()")).fetchone()
```

Or with psycopg directly, plus a look at how the machine came up:

```python
import psycopg

from smoltest import PostgresMachine

with PostgresMachine("postgres:16") as postgres:
    print(postgres.boot_info)  # BootInfo(via='restore', target='local', elapsed_s=0.41, ...)
    with psycopg.connect(postgres.get_connection_url(driver=None)) as conn:
        print(conn.execute("SELECT version()").fetchone())
```

What you get back from `PostgresMachine` (the whole testcontainers surface, plus a few
microVM-only calls):

| call | what it does |
|---|---|
| `start()` / `stop()` / `with ...:` | boot (branch, restore or cold) and tear down; `stop()` twice is harmless |
| `get_connection_url(host=None, driver=...)`, `.url` | `postgresql+<driver>://user:pw@host:port/db`; `driver=None` gives `postgresql://` |
| `get_container_host_ip()`, `get_exposed_port(port=None)`, `.endpoint` | `127.0.0.1` and the host port locally; the tunnel endpoint on the cloud |
| `with_env`, `with_envs`, `with_command`, `with_name`, `with_exposed_ports`, `with_bind_ports`, `waiting_for`, `with_startup_timeout` | the testcontainers builders, all returning `self` |
| `with_init_sql(*paths)`, `with_seed(Seed)`, `with_resources(cpus, memory_mb)` | load data, size the guest |
| `exec(command)` | runs in the guest; the `ExecResult` unpacks as `(exit_code, output)` like testcontainers |
| `psql(sql, dbname=None)` | in-guest `psql -tA` over the trusted unix socket; returns stdout |
| `branch(name=None)` | a copy-on-write child with its own host port; the parent keeps running |
| `checkpoint(output=None)` | snapshot the running machine (`output` is required on the local target) |
| `boot_info`, `is_running`, `get_logs()`, `get_wrapped_container()` | introspection |

`AsyncPostgresMachine` mirrors the constructor and makes `start`, `stop`, `exec`, `psql`,
`branch` and `checkpoint` awaitable (they run in a worker thread); `async with` works, and
`.sync` reaches the builders.

Seeds describe data the golden machine should hold; the result is cached under the seed's key:

```python
from smoltest import Seed

Seed.from_sql("CREATE TABLE t (id int)")  # key = sha256 of the text
Seed.from_sql_files("schema.sql", "fixtures.sql")  # key = sha256 of the joined files, in order
Seed.from_callable(run_migrations, key=alembic_head)  # any code; change the key when it changes
```

## pytest plugin

The plugin registers itself through the `pytest11` entry point, so installing `smoltest` is
enough. Tests ask for a database by fixture:

```python
def test_users(postgres_url):  # postgresql://test:test@127.0.0.1:2xxxx/test
    with psycopg.connect(postgres_url) as conn:
        conn.execute("INSERT INTO users (name) VALUES ('ada')")


def test_machine(postgres):  # the PostgresMachine itself
    assert postgres.psql("SELECT count(*) FROM users") == "0"
```

Each test gets a machine that is isolated from every other test, and the session boots the
underlying **golden** once.

Fixtures:

| fixture | scope | what it is |
|---|---|---|
| `postgres` | function | a `PostgresMachine` for this test: a branch of the golden by default |
| `postgres_url` | function | its `postgresql://` URL (driver-neutral) |
| `postgres_golden_url` | session | the golden's own `postgresql://` URL |
| `smoltest_postgres_golden` | session | the `PostgresGolden` (one per session, or per xdist worker) |
| `smoltest_postgres_template` | session | the unstarted `PostgresMachine` the golden boots from; override to apply builders |
| `smoltest_seed_postgres` | session | the seed applied to the golden; `None` by default. **Override this** to load a schema |
| `smoltest_settings`, `smoltest_engine` | session | the effective `Settings` and the engine in use |

Seeding, in `conftest.py` (see [`examples/conftest_seed_example.py`](examples/conftest_seed_example.py)):

```python
import pytest
from smoltest import Seed


@pytest.fixture(scope="session")
def smoltest_seed_postgres():
    return Seed.from_sql_files("schema.sql", "fixtures.sql")
```

The seeded golden is checkpointed under `seed.key`, so the next session restores the seeded
state instead of running the SQL again, and every branch inherits the data.

Isolation modes, set with the `smoltest_isolation` ini key or per test with the marker:

- `branch` (default): the test gets a copy-on-write branch of the golden, stopped after the test.
  When the engine or target cannot branch, the plugin warns once and falls back to `fresh`.
- `fresh`: a separate machine booted from the same template and seed (a restore of another cache
  variant, or a cold boot).
- `shared`: the golden itself, with no teardown; your test is responsible for cleaning up.

```python
@pytest.mark.smoltest(isolation="shared")
def test_read_only(postgres): ...


@pytest.mark.smoltest(image="postgres:15", env={"TZ": "UTC"}, seed=Seed.from_sql("..."))
def test_on_another_golden(postgres): ...  # image / env / seed select a distinct golden
```

Options, resolved as command line > `SMOLTEST_*` environment > ini file > defaults:

| command line | ini key | setting |
|---|---|---|
| `--smoltest-target {auto,local,cloud}` | `smoltest_target` | `target` |
| `--smoltest-image IMAGE` | `smoltest_postgres_image` | `postgres_image` |
| `--smoltest-cache-dir DIR` | `smoltest_cache_dir` (relative to the ini file) | `cache_dir` |
| `--smoltest-no-cache` | `smoltest_disable_cache` | `disable_cache` |
| `--smoltest-no-branch` | `smoltest_disable_branch` | `disable_branch` |
| `--smoltest-cpus N` | `smoltest_cpus` | `cpus` |
| `--smoltest-memory-mb MIB` | `smoltest_memory_mb` | `memory_mb` |
| – | `smoltest_isolation` (`branch`, `fresh`, `shared`) | what `postgres` hands a test |
| – | `smoltest_skip_if_unavailable` (default `false`) | skip instead of fail when no target can boot |

The report header prints the target, image, cache directory and isolation mode, then one line
per golden (`booted via restore in 0.38s on local`); the terminal summary counts the branched
and fresh machines tests used and their mean latency. With `pytest-xdist` each worker boots its
own golden; run `smoltest warm postgres --variants N` first so every worker restores instead of
booting cold (see below).

## Command line

```text
smoltest [--cache-dir DIR] [--target {auto,local,cloud}] [-v] COMMAND
```

- `smoltest doctor [--json]` reports versions, the local engine's availability
  (`KVM_UNAVAILABLE`, `HYPERVISOR_UNAVAILABLE`, ...), whether `/dev/kvm` exists, whether
  `SMOL_CLOUD_TOKEN` is set (never its value) and `websockets` is installed, the resolved
  target and the cache's size and claims. Exit 0 when a target is usable, else 1.
- `smoltest warm postgres [--image IMAGE] [--no-fast] [--seed-sql FILE ...] [--env K=V ...]
  [--cpus N] [--memory-mb MIB] [--variants K]` boots K machines with the cache enabled, keeping
  each alive until the last is up so each lands on its own host port, then deletes them. Run it
  before `pytest -n K` so every worker restores. Prints one row per boot (via, key, port, size,
  populate time (checkpoint write and cache bookkeeping), elapsed, seed state). Exit 0 only when
  every requested variant is in the cache, populated and intact; a boot that finished uncached
  (busy key lock, failed capture) is reported on stderr and the exit status is 1.
- `smoltest cache ls [--json]`, `smoltest cache prune [--older-than 14d] [--keep-latest N]
  [--max-bytes 2G] [--stale] [-y]`, `smoltest cache clear [-y]` and
  `smoltest cache export KEY[:PORT] OUT` manage the local store; `--target cloud` switches to the
  cloud checkpoint index (which cannot be exported). Variants claimed by a live process are never
  pruned; `--stale` removes those that cannot restore here (other host or SDK version, corrupt,
  unfinished). `export` warns that the copy contains guest RAM.
- `smoltest run postgres [--image IMAGE] [--no-fast] [--seed-sql FILE ...] [--env K=V ...]
  [--port HOST_PORT] [--no-cache]` boots one machine, prints its `postgresql://` URL and how it
  booted, blocks until Ctrl-C (SIGINT/SIGTERM) and deletes it. `--port` is refused on the cloud
  target (exit 1 before anything boots): the tunnel chooses its loopback port.

Exit status: 0 ok, 1 failure, 2 usage error, 3 no Smol target available.

## How it boots fast

Every machine smoltest creates is `branchable`. A boot climbs a three-rung ladder, fastest first:

| rung | mechanism | expected | when |
|---|---|---|---|
| branch | `machine.branch(name, ports=[fresh host port])`: a copy-on-write clone of RAM and disk of the running golden | under 200 ms (Smol's stated figure) | every test inside one process |
| restore | `Machine.restore_checkpoint(cached .smolcheckpoint)`: RAM and processes resume, PostgreSQL is already running | about 0.3 s (Smol measured 0.26 s on Linux) | the first machine of a process, cache hit |
| cold | `Machine.create(image, command, ...)` + wait for `pg_isready` + checkpoint into the cache | a few seconds (our estimate, 2–5 s, pending `benchmarks/bench_boot.py` on a KVM host; the first boot of an image also pulls it) | cache miss |

The branch and restore figures come from Smol's own measurements, not ours yet: this
repository was built in a sandbox without `/dev/kvm`, so the real-machine numbers will be
recorded here once `uv run python benchmarks/bench_boot.py --rounds 5` has run on a KVM host.

After a restore the guest wakes up with the clock it was checkpointed with; smoltest sets it
to the host's time (`date -s` inside the guest, best effort, `boot_info.clock_resynced`) and
then runs the readiness wait again before handing the machine out.

**Port variants.** A checkpoint preserves the machine's shape, published host port included,
so two restores of the same checkpoint on one host would collide. A cache key therefore owns
one or more *variants*, each a checkpoint recorded with the host port it binds. A boot claims
a variant whose port is free (file lock, a pid claim file, and a bind probe) and restores it;
when none is free it cold-boots on a fresh port from the 20000–29999 range and adds a variant.
`smoltest warm postgres --variants K` pre-creates K of them for K concurrent workers.

**Seeds.** A seed's result is a second checkpoint under `key + seed.key`. The ladder tries the
seeded key first, then the base key: on a base hit it restores, applies the seed, terminates
the seed's client connections so no host connection is captured, re-waits, and checkpoints the
seeded variant on the same port. A cold boot with a seed writes both checkpoints.

**The cache key** is the SHA-256 (32 hex chars) of canonical JSON holding: a format number,
smoltest's major version, the SDK version, the target, the host signature (OS, kernel major,
architecture, a hash of the CPU feature flags, libc) for local or the cloud base URL for cloud,
the image, the full workload argv, the guest environment (credentials included, sorted), the
guest ports, the pinned host ports of extra guest ports (`extra_ports=(PortMapping(29080,
9080),)` restores on host port 29080, so it is part of the shape; an engine-chosen extra port
is not), cpus, memory, storage and network, and the seed key or `None`. The PostgreSQL host
port is deliberately not in it: it is the variant's attribute. The key format is now 2.
Anything that would make a restored server differ from a fresh one changes the key; a key
that no longer matches simply misses and boots cold.

**What is on disk.** Local checkpoints live under the user cache directory
(`~/.cache/smoltest` by default, see the configuration reference) as

```text
<cache_dir>/postgres/.lock                      store-wide lock
<cache_dir>/postgres/<key>.lock                 per-key lock
<cache_dir>/postgres/store/                     the engine's dedup store
<cache_dir>/postgres/<key>/inputs.json          what produced the key; env values hashed
<cache_dir>/postgres/<key>/<port>.smolcheckpoint
<cache_dir>/postgres/<key>/<port>.meta.json     commit marker, written last
<cache_dir>/postgres/<key>/<port>.claim         pid + token of the process using the variant
<cache_dir>/cloud/<sha(base_url)>/index.json    key -> cloud checkpoint id
```

Directories are `0o700` and files `0o600`. **A checkpoint contains the guest's RAM**, so it
holds the database, its credentials and whatever else the guest had in memory: keep the cache
private, never commit a `.smolcheckpoint` (the repository's `.gitignore` already excludes them)
and treat `smoltest cache export` output the same way. When those modes cannot be applied (a
filesystem that refuses or ignores `chmod`), the checkpoint just written is removed again and
the operation fails with code `CHECKPOINT_PERMISSIONS` rather than leaving a readable copy; this
holds for cache variants (during a boot the `BootError`'s cause carries the code), `cache export`
and `PostgresMachine.checkpoint()` alike. The cache is
pruned least-recently-used down to `cache_max_bytes` (10 GiB by default); a variant that fails
to restore is invalidated and replaced by a cold boot.

## Compared with testcontainers-python

| | testcontainers-python | smoltest |
|---|---|---|
| Isolation unit | a Docker container sharing the host kernel | a microVM with its own kernel (KVM or Hypervisor.framework), or a Smol Cloud machine |
| Boot path | `docker run` + wait for the ready log line, every time | branch (< 200 ms) → restore a checkpoint (~0.3 s) → cold boot once per image and settings |
| Per-test isolation | a container per test, or one container with manual cleanup | a copy-on-write branch of the golden per test (`fresh` and `shared` modes available) |
| Host dependencies | a Docker daemon (or compatible socket) | none beyond the `smolmachines` wheel; `/dev/kvm` or Apple Silicon locally, or a cloud token |
| Readiness | log message by default | in-guest `pg_isready` by default; TCP, exec, SQL, log and composite strategies available |
| Logs | `get_logs()` reads the container's stdout | the microVM workload has no stdout pipe; `capture_logs=True` reads the server log from the guest |
| Volumes | bind mounts, `with_volume_mapping` | not available; `with_init_sql(*paths)` / `with_seed(Seed)` load data |
| Network | Docker networks, `with_kwargs(network=...)` | the guest has outbound network; published ports bind to `127.0.0.1`; no user-defined networks |
| Remote execution | `DOCKER_HOST` | `SMOL_CLOUD_TOKEN`; TCP reaches the machine through a loopback tunnel |
| Images | any | any PostgreSQL image whose entrypoint is `docker-entrypoint.sh` (the official ones) |

## Configuration reference

`Settings` is a frozen dataclass; `Settings.from_env()` reads the variables below (overrides
passed as keyword arguments win over the environment), `settings.replace(...)` derives a
variant, and `PostgresMachine(settings=...)` or the `smoltest_settings` fixture use it.

| `Settings` field | Environment variable | Default | Meaning |
|---|---|---|---|
| `target` | `SMOLTEST_TARGET` | `'auto'` | `auto` picks cloud when `SMOL_CLOUD_TOKEN` is set, else local when the engine can boot here |
| `cache_dir` | `SMOLTEST_CACHE_DIR` | `$SMOLTEST_CACHE_DIR`, `$XDG_CACHE_HOME/smoltest`, `~/Library/Caches/smoltest` (macOS) or `~/.cache/smoltest` | checkpoint cache directory |
| `disable_cache` | `SMOLTEST_DISABLE_CACHE` | `False` | never restore or populate checkpoints (always cold boot) |
| `disable_branch` | `SMOLTEST_DISABLE_BRANCH` | `False` | never branch the golden; every test gets a fresh machine |
| `postgres_image` | `SMOLTEST_POSTGRES_IMAGE` | `'postgres:16'` | default PostgreSQL image |
| `cpus` | `SMOLTEST_CPUS` | `1` | guest vCPUs (>= 1) |
| `memory_mb` | `SMOLTEST_MEMORY_MB` | `512` | guest memory in MiB (>= 64) |
| `storage_gb` | – | `None` | guest disk in GiB; `None` leaves the engine default |
| `network` | `SMOLTEST_NETWORK` | `True` | give the guest outbound network |
| `fast_mode` | `SMOLTEST_FAST` | `True` | `fsync=off`, `synchronous_commit=off`, `full_page_writes=off`, `initdb --no-sync` |
| `ready_timeout_s` | `SMOLTEST_READY_TIMEOUT` | `120.0` | readiness timeout in seconds (also `TC_MAX_TRIES * TC_POOLING_INTERVAL` when unset) |
| `poll_interval_s` | – | `0.05` | readiness poll interval in seconds |
| `exec_timeout_s` | `SMOLTEST_EXEC_TIMEOUT` | `600.0` | default timeout for in-guest commands (seeds, `psql`, `exec`) when the call passes none, on cold, restored and branched machines; the command is killed when it expires |
| `cache_max_bytes` | `SMOLTEST_CACHE_MAX_BYTES` | `10 GiB` | LRU budget of the local checkpoint cache |
| `cloud_auto_stop_seconds` | `SMOLTEST_CLOUD_AUTO_STOP` | `1800` | `auto_stop_seconds` given to every cloud machine smoltest creates (cold boots) |
| `cloud_ttl_seconds` | `SMOLTEST_CLOUD_TTL` | `7200` | `ttl_seconds` given to every cloud machine smoltest creates (cold boots) |
| `engine_path` | `SMOLTEST_ENGINE` | `None` | engine factory `pkg.mod:callable` (testing: `smoltest.testing:FakeEngine`) |

Booleans accept `1/true/yes/on` and `0/false/no/off`. Other variables smoltest reads:
`SMOL_CLOUD_TOKEN` (selects the cloud target when `target` is `auto`), `SMOL_CLOUD_URL`
(the API base URL, part of cloud cache keys), `POSTGRES_USER` / `POSTGRES_PASSWORD` /
`POSTGRES_DB` (default credentials, else `test`/`test`/`test`, as in testcontainers) and
`TC_MAX_TRIES` / `TC_POOLING_INTERVAL` (testcontainers' readiness knobs).

Per machine, `PostgresMachine(..., target=, cpus=, memory_mb=, cache=, fast=, wait=, name=,
seed=, capture_logs=)` override the settings for that machine only.

## Compatibility notes

- **`fast_mode`** (on by default) starts the server with `-c fsync=off -c synchronous_commit=off
  -c full_page_writes=off` and runs `initdb --no-sync`. A test database in a throwaway VM
  loses nothing it cares about, and it makes every write cheaper. It is part of the cache key.
  Turn it off with `fast=False`, `SMOLTEST_FAST=0` or `--no-fast` when you need durability
  semantics, e.g. to test crash recovery.
- The workload argv is always explicit: `docker-entrypoint.sh postgres` plus the `-c` switches
  for fast mode and log capture. `with_command(...)` replaces it entirely.
- A non-default `port=` moves the server itself: `-c port=<port>` is appended right after
  `docker-entrypoint.sh postgres`, and every in-guest client smoltest runs (`psql()`, SQL seeds,
  `pg_isready`, the in-guest `SqlWaitStrategy` probe and the pre-checkpoint backend
  termination) passes `-p <port>`. `with_command(...)` replaces the whole argv, so with
  `port != 5432` set the port yourself (`-c port=<port>`); otherwise the readiness probes on
  that port never pass.
- The default readiness check is in-guest `pg_isready` (`default_wait`), which proves the final
  server, not initdb's temporary one, is accepting TCP connections. `SqlWaitStrategy` probes
  from the host with the first importable driver (`psycopg`, `psycopg2`, `asyncpg`) and closes
  its connection, or runs `psql` in the guest; `PortWaitStrategy`, `ExecWaitStrategy` and
  `CompositeWaitStrategy` work as in testcontainers, and `with_startup_timeout` /
  `with_poll_interval` accept seconds or a `timedelta`. Every in-guest probe is bounded by
  `EXEC_PROBE_TIMEOUT_S` (10 s) so a stalled guest agent cannot hold the wait past its
  deadline. On the cloud target `PortWaitStrategy` only proves the tunnel is open (its
  endpoint is the local end of the relay); use `PgIsReadyWaitStrategy` or `SqlWaitStrategy`
  there to prove the server is up.
- The readiness strategy runs on every boot path, including `cache=False` / `--no-cache`
  and engines without checkpoints, before any seed is applied.
- `LogMessageWaitStrategy` and `get_logs()` need `capture_logs=True`, which turns on
  `logging_collector` and writes the server log to `/tmp/postgresql.log` in the guest
  (`/tmp` exists and is writable in both the Debian and the alpine official images, unlike
  `/var/log/postgresql`, which alpine lacks); without it `get_logs()` returns empty output
  with one warning and the log strategy raises `NotSupportedError`. `HttpWaitStrategy`
  always raises `NotSupportedError`.
- `POSTGRES_USER` / `POSTGRES_PASSWORD` / `POSTGRES_DB` set through `with_env`, `with_envs`,
  the marker's `env=` or `--env` are folded into the machine's `username` / `password` /
  `dbname`, so `get_connection_url()`, `psql()`, seeds and readiness probes always match
  the server that boots (testcontainers instead overrides such variables from its
  constructor arguments).
- `with_volume_mapping()` raises `NotSupportedError`: a microVM has no host mounts. Use
  `with_init_sql(*paths)` or `with_seed(Seed)`.
- `with_kwargs(...)` and unknown constructor keyword arguments (Docker `run` options) are
  accepted, warned about once per name (`SmoltestWarning`) and ignored.
- `stop(force=...)` ignores `force` (a microVM is always stopped hard); `stop(delete=False)`
  stops the guest and leaves deletion to interpreter exit. A machine that is garbage collected
  or still alive at exit is deleted by a finalizer, children before parents.
- `get_exposed_port(port)` for a port other than PostgreSQL's raises `NotSupportedError` on the
  cloud target, where only the tunnelled PostgreSQL port is reachable. Locally, published ports
  bind to `127.0.0.1` only.
- `exec()` returns an `ExecResult`; `exit_code, output = postgres.exec("...")` and
  `postgres.exec("...")[1]` work as with docker's result tuple, and `.stdout`, `.stderr`,
  `.text` and `.ok` are there too. Without `timeout_s`, in-guest commands (also seeds and
  `psql()`) run under `Settings.exec_timeout_s` (600 s by default) on both targets, on cold,
  restored and branched machines alike; the Smol SDK would otherwise cap a cloud exec at its
  30 s HTTP read timeout. A machine attached with `Engine.connect()` (made elsewhere) keeps
  the engine default of 600 s.
- URLs: the default `driver="psycopg2"` yields `postgresql+psycopg2://`, `driver="psycopg"`
  yields `postgresql+psycopg://`, `driver=None` yields `postgresql://`; the password is
  percent-encoded exactly as testcontainers does, and the user and database names are
  percent-encoded too, so `/`, `@`, `#` or `?` in any of them keeps the URL well-formed.
- Errors are a small hierarchy under `SmoltestError`: `TargetUnavailable` (nothing can boot here;
  the message points at `smoltest doctor`), `BootError` with a `.stage` (`create`, `restore`,
  `tunnel`, `wait`, `seed`, ...), `ReadinessTimeout` (also a `TimeoutError`), `CacheError`,
  `NotSupportedError`, `InvalidConfig` (also a `ValueError`) and `ExecError`.

## Limitations

- The local engine needs hardware virtualisation: `/dev/kvm` on Linux (not available on most
  hosted CI runners) or Apple Silicon. Without it, use Smol Cloud (`SMOL_CLOUD_TOKEN`).
- No Docker-style volumes or bind mounts; data goes in through SQL seeds or `exec`.
- Floating tags: the cache key hashes the image reference as written (`postgres:16`), not the
  image digest. After the registry moves a tag, run `smoltest cache prune --stale` or
  `smoltest cache clear` to boot the new image.
- Local checkpoints are host-local: they restore only on the same OS, architecture, CPU feature
  set and SDK version (all part of the key), and they contain guest RAM.
- The cloud target needs the `cloud` extra (`websockets`) for the TCP tunnel (checked before
  any cloud machine is created) and keeps the PostgreSQL port only (no `host_port` for extra
  ports). A fixed PostgreSQL host port (`with_bind_ports(5432, N)`, `--port N`) is refused there
  with `NotSupportedError`: the tunnel chooses its loopback port, and silently booting on another
  port would break the clients configured for the requested one.
- The `auto_stop_seconds` / `ttl_seconds` safety net is sent only when smoltest *creates* a
  cloud machine (a cold boot). Machines produced by a checkpoint restore (warm boots) and by
  branching the golden (the default pytest isolation) carry whatever Smol Cloud assigns them;
  the SDK offers no way to set a TTL on those, so a crash or `SIGKILL` that skips the
  `atexit` reaper can leave such billable machines running until you delete them from the
  Smol console or CLI (they are named with the `smoltest-pg-` prefix). A `Ctrl-C` during a
  boot or a branch does tear the machine down before propagating.
- A golden restored from a checkpoint may not be a branch source: the SDK's restore paths
  start the machine without the `forkable` flag. When the engine refuses to branch such a
  golden, smoltest warns once and falls back to fresh machines for every test, exactly as it
  does for a target that cannot branch. A seeded variant that cannot be captured for the same
  reason is handed out working and simply not cached.
- A restored checkpoint whose server never becomes ready, or whose guest cannot run the
  readiness probe at all, is invalidated and the boot moves on to the next variant or a cold
  boot (a wait strategy that refuses the machine raises instead and keeps the variant); a
  second process still populating a cache key is waited
  for, and when that wait runs out the boot proceeds cold without caching instead of failing.
- PostgreSQL is the only service today; the images must use the official
  `docker-entrypoint.sh` conventions (`POSTGRES_USER` / `POSTGRES_PASSWORD` / `POSTGRES_DB`,
  `POSTGRES_INITDB_ARGS`).

## Development

```sh
uv sync --all-extras --group dev

uv run ruff check . && uv run ruff format --check .   # lint and formatting
uv run mypy                                           # strict, over src/
uv run pytest -q                                      # unit suite on the FakeEngine (no KVM needed)
uv run python benchmarks/bench_boot.py --fake         # the benchmark script on the FakeEngine

uv run pytest -m local_e2e -q                         # real machines; needs /dev/kvm or Apple Silicon
uv run pytest -m cloud_e2e -q                         # real machines on Smol Cloud; needs SMOL_CLOUD_TOKEN
uv run python benchmarks/bench_boot.py --rounds 5     # cold vs restore vs branch, on a real target
```

The unit suite selects `smoltest.testing.FakeEngine` through `SMOLTEST_ENGINE`
(`tests/conftest.py`) and never imports `smol`; the fake binds real loopback listeners, so
port collisions and readiness probes behave as they do against real machines. The e2e
markers are excluded by `addopts` and opt in with `-m`. Downstream projects can use the same
fake to test their fixtures without KVM: `SMOLTEST_ENGINE=smoltest.testing:FakeEngine pytest`.

This repository was assembled in a sandbox with no `/dev/kvm` and no Docker, so the e2e
suites and the benchmark on real machines have not been run here yet; `.github/workflows/ci.yml`
runs lint, the unit matrix (3.10–3.13) and the fake benchmark, and gates the two e2e jobs
behind a self-hosted KVM runner and the `SMOL_CLOUD_TOKEN` secret.

## License

Apache-2.0; see [LICENSE](LICENSE).
