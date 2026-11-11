"""pytest plugin loaded through the ``pytest11`` entry point.

It adds the ``--smoltest-*`` options and ``smoltest_*`` ini keys (resolved as
CLI > environment > ini > defaults into one :class:`~smoltest.config.Settings`),
the ``@pytest.mark.smoltest(...)`` marker and the fixtures below:

``smoltest_settings``, ``smoltest_engine``, ``smoltest_seed_postgres`` (the
override point for seeding), ``smoltest_postgres_template`` (an unstarted
:class:`~smoltest.postgres.PostgresMachine`), ``smoltest_postgres_golden`` (one
booted golden per session, or per xdist worker), ``postgres`` (an isolated
machine per test: a branch, a fresh machine or the shared golden),
``postgres_url`` and ``postgres_golden_url``.

Importing this module must stay cheap and side-effect free: pytest loads it in
every run of every project that has smoltest installed. Nothing boots until a
fixture asks for it, and the rest of the package is imported lazily.
"""

from __future__ import annotations

import dataclasses
import os
import statistics
from collections.abc import Iterator, Mapping
from dataclasses import dataclass, field
from pathlib import Path
from typing import TYPE_CHECKING, Any, Literal

import pytest

from ._log import logger
from .config import ENV_BY_FIELD, Settings
from .errors import InvalidConfig, TargetUnavailable

if TYPE_CHECKING:
    from .boot.golden import PostgresGolden
    from .postgres import PostgresMachine, Seed
    from .transport.base import Engine

Isolation = Literal["branch", "fresh", "shared"]
"""What the ``postgres`` fixture hands a test: a branch, a fresh machine or the golden."""

ISOLATIONS: tuple[Isolation, ...] = ("branch", "fresh", "shared")
MARKER = "smoltest"
"""Name of the per-test marker: ``@pytest.mark.smoltest(image=, env=, isolation=, seed=)``."""

_ISOLATION_BY_NAME: dict[str, Isolation] = {name: name for name in ISOLATIONS}
_MARKER_KEYS = frozenset({"image", "env", "isolation", "seed"})
_TARGETS = ("auto", "local", "cloud")

# (option dest, Settings field) and (ini key, Settings field).
_OPTIONS: tuple[tuple[str, str], ...] = (
    ("smoltest_target", "target"),
    ("smoltest_image", "postgres_image"),
    ("smoltest_cache_dir", "cache_dir"),
    ("smoltest_no_cache", "disable_cache"),
    ("smoltest_no_branch", "disable_branch"),
    ("smoltest_cpus", "cpus"),
    ("smoltest_memory_mb", "memory_mb"),
)
_INI: tuple[tuple[str, str], ...] = (
    ("smoltest_target", "target"),
    ("smoltest_postgres_image", "postgres_image"),
    ("smoltest_cache_dir", "cache_dir"),
    ("smoltest_disable_cache", "disable_cache"),
    ("smoltest_disable_branch", "disable_branch"),
    ("smoltest_cpus", "cpus"),
    ("smoltest_memory_mb", "memory_mb"),
)
_INT_FIELDS = frozenset({"cpus", "memory_mb"})


# -- options and settings ----------------------------------------------------------------


def pytest_addoption(parser: pytest.Parser) -> None:
    """Register the ``--smoltest-*`` options and the ``smoltest_*`` ini keys."""
    group = parser.getgroup("smoltest", "smoltest: PostgreSQL in Smol Machines microVMs")
    group.addoption(
        "--smoltest-target",
        dest="smoltest_target",
        default=None,
        choices=_TARGETS,
        help="where machines boot (default: SMOLTEST_TARGET, ini smoltest_target, else auto)",
    )
    group.addoption(
        "--smoltest-image",
        dest="smoltest_image",
        default=None,
        metavar="IMAGE",
        help="PostgreSQL image to boot (default: postgres:16)",
    )
    group.addoption(
        "--smoltest-cache-dir",
        dest="smoltest_cache_dir",
        default=None,
        metavar="DIR",
        help="checkpoint cache directory",
    )
    group.addoption(
        "--smoltest-no-cache",
        dest="smoltest_no_cache",
        action="store_true",
        default=False,
        help="never restore or populate checkpoints (always cold boot)",
    )
    group.addoption(
        "--smoltest-no-branch",
        dest="smoltest_no_branch",
        action="store_true",
        default=False,
        help="boot a fresh machine per test instead of branching the golden",
    )
    group.addoption(
        "--smoltest-cpus",
        dest="smoltest_cpus",
        type=int,
        default=None,
        metavar="N",
        help="guest vCPUs",
    )
    group.addoption(
        "--smoltest-memory-mb",
        dest="smoltest_memory_mb",
        type=int,
        default=None,
        metavar="MIB",
        help="guest memory in MiB",
    )
    parser.addini("smoltest_target", "auto | local | cloud", default=None)
    parser.addini("smoltest_postgres_image", "PostgreSQL image to boot", default=None)
    parser.addini(
        "smoltest_cache_dir",
        "checkpoint cache directory (a relative path is taken from the ini file)",
        default=None,
    )
    parser.addini(
        "smoltest_disable_cache",
        "never restore or populate checkpoints",
        type="bool",
        default=False,
    )
    parser.addini(
        "smoltest_disable_branch", "never branch the golden machine", type="bool", default=False
    )
    parser.addini("smoltest_cpus", "guest vCPUs", default=None)
    parser.addini("smoltest_memory_mb", "guest memory in MiB", default=None)
    parser.addini(
        "smoltest_isolation",
        "branch | fresh | shared: what the postgres fixture hands each test (default branch)",
        default="branch",
    )
    parser.addini(
        "smoltest_skip_if_unavailable",
        "skip tests instead of failing them when no Smol target is available",
        type="bool",
        default=False,
    )


def _ini_dir(config: pytest.Config) -> Path:
    return config.inipath.parent if config.inipath is not None else config.rootpath


def _coerce_ini(config: pytest.Config, name: str, field_name: str, raw: Any) -> Any:
    if field_name in ("disable_cache", "disable_branch"):
        return bool(raw)
    text = str(raw).strip()
    if field_name == "cache_dir":
        path = Path(text).expanduser()
        return path if path.is_absolute() else _ini_dir(config) / path
    if field_name in _INT_FIELDS:
        try:
            return int(text)
        except ValueError:
            raise InvalidConfig(f"{name}: expected an integer, got {text!r}") from None
    return text.lower() if field_name == "target" else text


def _ini_values(config: pytest.Config) -> dict[str, Any]:
    """Settings fields the ini file sets (unset keys and empty strings are skipped)."""
    values: dict[str, Any] = {}
    for name, field_name in _INI:
        raw = config.getini(name)
        if raw is None or raw is False or (isinstance(raw, str) and not raw.strip()):
            continue
        values[field_name] = _coerce_ini(config, name, field_name, raw)
    return values


def _option_values(config: pytest.Config) -> dict[str, Any]:
    """Settings fields given on the command line."""
    values: dict[str, Any] = {}
    for dest, field_name in _OPTIONS:
        value = config.getoption(dest, default=None)
        if value is None or value is False:
            continue
        values[field_name] = Path(str(value)).expanduser() if field_name == "cache_dir" else value
    return values


def settings_from_config(config: pytest.Config, env: Mapping[str, str] | None = None) -> Settings:
    """Resolve the session's :class:`~smoltest.config.Settings`.

    Precedence is command line > environment (``SMOLTEST_*``) > ini file >
    defaults. Malformed values raise :class:`pytest.UsageError`.
    """
    env = os.environ if env is None else env
    try:
        ini = _ini_values(config)
        below_env = {f: v for f, v in ini.items() if ENV_BY_FIELD[f].name not in env}
        # Command-line values go in as overrides, so a malformed SMOLTEST_* variable
        # they replace is never parsed.
        return Settings.from_env(env, **{**below_env, **_option_values(config)})
    except InvalidConfig as exc:
        raise pytest.UsageError(f"smoltest: {exc}") from exc


def _ini_isolation(config: pytest.Config) -> Isolation:
    raw = str(config.getini("smoltest_isolation") or "branch").strip().lower()
    isolation = _ISOLATION_BY_NAME.get(raw)
    if isolation is None:
        raise pytest.UsageError(
            f"smoltest: smoltest_isolation must be one of {', '.join(ISOLATIONS)}; got {raw!r}"
        )
    return isolation


# -- per-session state -----------------------------------------------------------------


@dataclass(frozen=True)
class _ChildBoot:
    via: str
    elapsed_s: float


@dataclass
class _State:
    """What the plugin knows about one session: settings, goldens booted and children made."""

    settings: Settings
    isolation: Isolation
    goldens: list[PostgresGolden] = field(default_factory=list)
    children: list[_ChildBoot] = field(default_factory=list)
    shared_uses: int = 0

    def track(self, golden: PostgresGolden) -> None:
        if golden not in self.goldens:
            self.goldens.append(golden)

    def record_child(self, machine: PostgresMachine) -> None:
        info = machine.boot_info
        if info is not None:
            self.children.append(_ChildBoot(info.via, info.elapsed_s))

    def close(self) -> None:
        """Close every golden this session booted, most recent first; idempotent."""
        for golden in reversed(self.goldens):
            _close_quietly(golden)


def _close_quietly(golden: PostgresGolden) -> None:
    try:
        golden.close()
    except Exception:
        logger.exception("closing %r failed", golden)


_STATE_KEY: pytest.StashKey[_State] = pytest.StashKey()


def _state(config: pytest.Config) -> _State:
    state = config.stash.get(_STATE_KEY, None)
    if state is None:
        state = _State(settings_from_config(config), _ini_isolation(config))
        config.stash[_STATE_KEY] = state
    return state


def pytest_configure(config: pytest.Config) -> None:
    """Register the marker and resolve (and validate) the settings for this session."""
    config.addinivalue_line(
        "markers",
        "smoltest(image=None, env=None, isolation=None, seed=None): per-test overrides for "
        "the postgres fixture; image/env/seed select a distinct golden machine, isolation is "
        "one of branch, fresh or shared",
    )
    config.stash[_STATE_KEY] = _State(settings_from_config(config), _ini_isolation(config))


def pytest_sessionfinish(session: pytest.Session) -> None:
    """Close the goldens this session booted (children first) before the summary prints."""
    state = session.config.stash.get(_STATE_KEY, None)
    if state is not None:
        state.close()


# -- reporting -------------------------------------------------------------------------


def _golden_line(golden: PostgresGolden) -> str:
    info = golden.boot_info
    seed = golden.seed
    label = golden.machine.image + (f" (seed {seed.key[:12]})" if seed is not None else "")
    return f"golden {label}: booted via {info.via} in {info.elapsed_s:.2f}s on {info.target}"


def pytest_report_header(config: pytest.Config) -> list[str]:
    """One line with the target, image, cache directory and isolation (plus goldens, if any)."""
    state = _state(config)
    s = state.settings
    cache = f"{s.cache_dir}" + (" (disabled)" if s.disable_cache else "")
    parts = [f"target={s.target}", f"image={s.postgres_image}", f"cache={cache}"]
    parts.append(f"isolation={state.isolation}" + (" (no branch)" if s.disable_branch else ""))
    return ["smoltest: " + " ".join(parts)] + [_golden_line(g) for g in state.goldens]


def pytest_terminal_summary(
    terminalreporter: pytest.TerminalReporter, config: pytest.Config
) -> None:
    """How the goldens booted and how many branch / fresh machines tests used."""
    state = config.stash.get(_STATE_KEY, None)
    if state is None or not state.goldens:
        return
    terminalreporter.write_sep("-", "smoltest")
    for golden in state.goldens:
        terminalreporter.write_line(_golden_line(golden))
    branches = [c.elapsed_s for c in state.children if c.via == "branch"]
    fresh = [c.elapsed_s for c in state.children if c.via != "branch"]
    line = f"per-test machines: {len(branches)} branched"
    if branches:
        line += f" (mean {statistics.fmean(branches) * 1000:.0f} ms)"
    line += f", {len(fresh)} fresh"
    if fresh:
        line += f" (mean {statistics.fmean(fresh):.2f} s)"
    if state.shared_uses:
        line += f", {state.shared_uses} shared"
    terminalreporter.write_line(line)


# -- marker ----------------------------------------------------------------------------


@dataclass(frozen=True)
class MarkerOptions:
    """The validated arguments of ``@pytest.mark.smoltest(...)`` closest to a test."""

    image: str | None = None
    env: Mapping[str, str] | None = None
    isolation: Isolation | None = None
    seed: Seed | None = None
    seed_given: bool = False

    @property
    def wants_own_golden(self) -> bool:
        """``True`` when the test needs a golden other than the session's."""
        return self.image is not None or self.env is not None or self.seed_given


def _marker_error(node: pytest.Item, text: str) -> None:
    pytest.fail(f"@pytest.mark.{MARKER} on {node.nodeid}: {text}", pytrace=False)


def _marker_env(node: pytest.Item, env: Any) -> dict[str, str] | None:
    if env is None:
        return None
    if not isinstance(env, Mapping) or not all(
        isinstance(k, str) and isinstance(v, str) for k, v in env.items()
    ):
        _marker_error(node, "env must be a mapping of str to str")
    return dict(env)


def _marker_isolation(node: pytest.Item, raw: Any) -> Isolation | None:
    if raw is None:
        return None
    isolation = _ISOLATION_BY_NAME.get(str(raw).strip().lower())
    if isolation is None:
        _marker_error(node, f"isolation must be one of {', '.join(ISOLATIONS)}; got {raw!r}")
    return isolation


def marker_options(node: pytest.Item) -> MarkerOptions:
    """Parse the ``smoltest`` marker on ``node``; a malformed marker fails the test."""
    marker = node.get_closest_marker(MARKER)
    if marker is None:
        return MarkerOptions()
    if marker.args:
        _marker_error(node, "takes keyword arguments only (image=, env=, isolation=, seed=)")
    unknown = sorted(set(marker.kwargs) - _MARKER_KEYS)
    if unknown:
        _marker_error(node, f"unknown arguments: {', '.join(unknown)}")
    image = marker.kwargs.get("image")
    if image is not None and (not isinstance(image, str) or not image.strip()):
        _marker_error(node, "image must be a non-empty string")
    seed = marker.kwargs.get("seed")
    if seed is not None and not isinstance(seed, _postgres().Seed):
        _marker_error(node, "seed must be a smoltest.Seed (or None)")
    return MarkerOptions(
        image=image,
        env=_marker_env(node, marker.kwargs.get("env")),
        isolation=_marker_isolation(node, marker.kwargs.get("isolation")),
        seed=seed,
        seed_given="seed" in marker.kwargs,
    )


# -- goldens ---------------------------------------------------------------------------


def _postgres() -> Any:
    """The ``smoltest.postgres`` module, imported on first use to keep plugin import cheap."""
    from . import postgres  # noqa: PLC0415  (lazy: never pay for the boot stack when unused)

    return postgres


def _acquire_golden(
    config: pytest.Config, template: PostgresMachine, seed: Seed | None
) -> PostgresGolden:
    """Boot (or reuse) the golden for ``template`` + ``seed`` through the process registry."""
    from .boot.golden import GoldenRegistry  # noqa: PLC0415  (lazy, see _postgres)

    state = _state(config)
    try:
        golden = GoldenRegistry.instance().get_or_boot(template, seed)
    except TargetUnavailable as exc:
        message = f"smoltest: {exc}"
        if config.getini("smoltest_skip_if_unavailable"):
            pytest.skip(message)
        pytest.fail(message, pytrace=False)
    state.track(golden)
    return golden


def _derive_template(
    template: PostgresMachine, options: MarkerOptions, seed: Seed | None
) -> PostgresMachine:
    """A copy of ``template`` with the marker's image/env applied and no name or port pin."""
    spec = template.spec
    changes: dict[str, Any] = {}
    if options.image is not None:
        changes["image"] = options.image
    if options.env is not None:
        changes["env"] = {**spec.env, **options.env}
    derived: PostgresMachine = _postgres().PostgresMachine(
        image=spec.image,
        port=spec.guest_port,
        username=spec.username,
        password=spec.password,
        dbname=spec.dbname,
        driver=template.driver,
        settings=template.settings,
        engine=template.engine,
        wait=template.wait_strategy,
        seed=seed,
        capture_logs=spec.capture_logs,
    )
    # The builders cannot copy command, extra ports or storage; the spec is immutable anyway.
    derived._spec = dataclasses.replace(spec, **changes)
    return derived


def _golden_for(request: pytest.FixtureRequest, options: MarkerOptions) -> PostgresGolden:
    if not options.wants_own_golden:
        shared: PostgresGolden = request.getfixturevalue("smoltest_postgres_golden")
        return shared
    template: PostgresMachine = request.getfixturevalue("smoltest_postgres_template")
    session_seed: Seed | None = request.getfixturevalue("smoltest_seed_postgres")
    if options.seed_given:
        seed = options.seed
    else:
        seed = session_seed if session_seed is not None else template.seed
    return _acquire_golden(request.config, _derive_template(template, options, seed), seed)


# -- fixtures --------------------------------------------------------------------------


@pytest.fixture(scope="session")
def smoltest_settings(pytestconfig: pytest.Config) -> Settings:
    """Effective :class:`~smoltest.config.Settings` (CLI > environment > ini > defaults)."""
    return _state(pytestconfig).settings


@pytest.fixture(scope="session")
def smoltest_engine(smoltest_settings: Settings) -> Engine:
    """The engine every machine of the session boots on (``SMOLTEST_ENGINE`` or the Smol SDK)."""
    from .transport import get_engine  # noqa: PLC0415  (lazy, see _postgres)

    return get_engine(smoltest_settings)


@pytest.fixture(scope="session")
def smoltest_seed_postgres() -> Seed | None:
    """Seed applied once to the golden database; ``None`` (the default) leaves it empty.

    This is the override point. In ``conftest.py``::

        @pytest.fixture(scope="session")
        def smoltest_seed_postgres():
            return Seed.from_sql_files("schema.sql", "fixtures.sql")

    or ``Seed.from_callable(run_migrations, key=alembic_head)``. The seeded state
    is checkpointed under ``seed.key``, so later sessions restore it instead of
    running the seed again, and every branch inherits the data.
    """
    return None


@pytest.fixture(scope="session")
def smoltest_postgres_template(
    smoltest_settings: Settings, smoltest_engine: Engine
) -> PostgresMachine:
    """The unstarted machine goldens boot from; override to apply builders.

    For example ``PostgresMachine(settings=smoltest_settings, engine=smoltest_engine)
    .with_env("TZ", "UTC").waiting_for(SqlWaitStrategy())``.
    """
    template: PostgresMachine = _postgres().PostgresMachine(
        settings=smoltest_settings, engine=smoltest_engine
    )
    return template


@pytest.fixture(scope="session")
def smoltest_postgres_golden(
    request: pytest.FixtureRequest,
    smoltest_postgres_template: PostgresMachine,
    smoltest_seed_postgres: Seed | None,
) -> PostgresGolden:
    """The session's golden PostgreSQL machine, booted once and closed at session end.

    Fails (or skips, with ``smoltest_skip_if_unavailable = true``) when no Smol
    target is available.
    """
    golden = _acquire_golden(request.config, smoltest_postgres_template, smoltest_seed_postgres)
    request.addfinalizer(golden.close)
    return golden


@pytest.fixture
def postgres(request: pytest.FixtureRequest) -> Iterator[PostgresMachine]:
    """A PostgreSQL machine for this test.

    Isolation comes from the ``smoltest`` marker, else ``smoltest_isolation``:
    ``branch`` forks the golden (copy-on-write, its own port, stopped after the
    test), ``fresh`` boots a separate machine, ``shared`` hands out the golden
    itself with no teardown. ``image=``, ``env=`` or ``seed=`` on the marker pick
    a distinct golden for the test.
    """
    options = marker_options(request.node)
    state = _state(request.config)
    golden = _golden_for(request, options)
    isolation = options.isolation or state.isolation
    if isolation == "shared":
        state.shared_uses += 1
        yield golden.machine
        return
    child = golden.branch() if isolation == "branch" else golden.fresh()
    state.record_child(child)
    try:
        yield child
    finally:
        child.stop()


@pytest.fixture
def postgres_url(postgres: PostgresMachine) -> str:
    """``postgresql://user:password@host:port/db`` of the test's machine (any driver)."""
    return postgres.get_connection_url(driver=None)


@pytest.fixture(scope="session")
def postgres_golden_url(smoltest_postgres_golden: PostgresGolden) -> str:
    """``postgresql://`` URL of the shared golden machine."""
    return smoltest_postgres_golden.machine.get_connection_url(driver=None)


__all__ = [
    "ISOLATIONS",
    "MARKER",
    "Isolation",
    "MarkerOptions",
    "marker_options",
    "postgres",
    "postgres_golden_url",
    "postgres_url",
    "pytest_addoption",
    "pytest_configure",
    "pytest_report_header",
    "pytest_sessionfinish",
    "pytest_terminal_summary",
    "settings_from_config",
    "smoltest_engine",
    "smoltest_postgres_golden",
    "smoltest_postgres_template",
    "smoltest_seed_postgres",
    "smoltest_settings",
]
