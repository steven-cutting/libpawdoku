"""Settings precedence, environment parsing and target resolution."""

from __future__ import annotations

from pathlib import Path

import pytest

from smoltest.config import ENV_VARS, Settings, default_cache_dir, parse_bool, resolve_target
from smoltest.errors import InvalidConfig, TargetUnavailable
from smoltest.testing import FakeEngine
from smoltest.transport import get_engine, load_engine_factory


def test_defaults_match_plan() -> None:
    s = Settings()
    assert s.target == "auto"
    assert s.postgres_image == "postgres:16"
    assert (s.cpus, s.memory_mb, s.storage_gb, s.network) == (1, 512, None, True)
    assert s.fast_mode is True
    assert s.ready_timeout_s == 120.0
    assert s.cache_max_bytes == 10 * 2**30
    assert (s.cloud_auto_stop_seconds, s.cloud_ttl_seconds) == (1800, 7200)
    assert s.engine_path is None


def test_from_env_reads_every_variable() -> None:
    env = {
        "SMOLTEST_TARGET": "cloud",
        "SMOLTEST_CACHE_DIR": "/tmp/x",
        "SMOLTEST_POSTGRES_IMAGE": "postgres:15",
        "SMOLTEST_DISABLE_CACHE": "1",
        "SMOLTEST_DISABLE_BRANCH": "yes",
        "SMOLTEST_CPUS": "2",
        "SMOLTEST_MEMORY_MB": "1024",
        "SMOLTEST_NETWORK": "false",
        "SMOLTEST_FAST": "off",
        "SMOLTEST_READY_TIMEOUT": "7.5",
        "SMOLTEST_EXEC_TIMEOUT": "42",
        "SMOLTEST_CACHE_MAX_BYTES": "123",
        "SMOLTEST_CLOUD_AUTO_STOP": "60",
        "SMOLTEST_CLOUD_TTL": "120",
        "SMOLTEST_ENGINE": "smoltest.testing:FakeEngine",
    }
    assert {v.name for v in ENV_VARS} == set(env)
    s = Settings.from_env(env)
    assert s.target == "cloud"
    assert s.cache_dir == Path("/tmp/x")
    assert s.postgres_image == "postgres:15"
    assert s.disable_cache and s.disable_branch
    assert (s.cpus, s.memory_mb) == (2, 1024)
    assert s.network is False and s.fast_mode is False
    assert s.ready_timeout_s == 7.5
    assert s.exec_timeout_s == 42.0
    assert s.cache_max_bytes == 123
    assert (s.cloud_auto_stop_seconds, s.cloud_ttl_seconds) == (60, 120)
    assert s.engine_path == "smoltest.testing:FakeEngine"


@pytest.mark.parametrize(
    ("env", "override", "expected"),
    [
        ({}, None, "postgres:16"),
        ({"SMOLTEST_POSTGRES_IMAGE": "postgres:15"}, None, "postgres:15"),
        ({"SMOLTEST_POSTGRES_IMAGE": "postgres:15"}, "postgres:14", "postgres:14"),
        ({}, "postgres:14", "postgres:14"),
    ],
)
def test_precedence_override_beats_env_beats_default(
    env: dict[str, str], override: str | None, expected: str
) -> None:
    assert Settings.from_env(env, postgres_image=override).postgres_image == expected


def test_replace_ignores_none_and_keeps_frozen() -> None:
    s = Settings(cpus=2)
    r = s.replace(cpus=None, memory_mb=256)
    assert (r.cpus, r.memory_mb) == (2, 256)
    assert s.memory_mb == 512
    with pytest.raises(AttributeError):
        s.cpus = 3  # type: ignore[misc]


def test_tc_compat_sets_ready_timeout_only_when_unset() -> None:
    assert Settings.from_env({"TC_MAX_TRIES": "30"}).ready_timeout_s == 30.0
    assert (
        Settings.from_env({"TC_MAX_TRIES": "30", "TC_POOLING_INTERVAL": "0.5"}).ready_timeout_s
        == 15.0
    )
    both = {"TC_MAX_TRIES": "30", "SMOLTEST_READY_TIMEOUT": "9"}
    assert Settings.from_env(both).ready_timeout_s == 9.0


@pytest.mark.parametrize(
    "env",
    [
        {"SMOLTEST_TARGET": "docker"},
        {"SMOLTEST_CPUS": "two"},
        {"SMOLTEST_FAST": "maybe"},
        {"SMOLTEST_READY_TIMEOUT": "soon"},
    ],
)
def test_malformed_env_raises_invalid_config_naming_the_variable(env: dict[str, str]) -> None:
    with pytest.raises(InvalidConfig, match=next(iter(env))):
        Settings.from_env(env)


def test_unknown_override_and_bad_values_raise() -> None:
    with pytest.raises(InvalidConfig, match="unknown settings"):
        Settings.from_env({}, bogus=1)
    with pytest.raises(InvalidConfig):
        Settings(cpus=0)
    with pytest.raises(InvalidConfig):
        Settings(target="docker")  # type: ignore[arg-type]


@pytest.mark.parametrize(
    ("raw", "expected"),
    [("1", True), ("TRUE", True), ("on", True), ("0", False), ("", False), ("No", False)],
)
def test_parse_bool(raw: str, expected: bool) -> None:
    assert parse_bool(raw) is expected


def test_default_cache_dir_precedence() -> None:
    assert default_cache_dir(
        {"SMOLTEST_CACHE_DIR": "/c", "XDG_CACHE_HOME": "/x", "HOME": "/h"}
    ) == Path("/c")
    assert default_cache_dir({"XDG_CACHE_HOME": "/x", "HOME": "/h"}) == Path("/x/smoltest")
    fallback = default_cache_dir({"HOME": "/h"})
    assert fallback in (Path("/h/.cache/smoltest"), Path("/h/Library/Caches/smoltest"))


def test_from_env_mapping_decides_the_default_cache_dir(monkeypatch: pytest.MonkeyPatch) -> None:
    monkeypatch.setenv("SMOLTEST_CACHE_DIR", "/from-the-process")
    assert Settings.from_env({"XDG_CACHE_HOME": "/isolated"}).cache_dir == Path(
        "/isolated/smoltest"
    ), "a supplied mapping, not the process environment, decides the default"
    assert Settings.from_env({"HOME": "/h"}).cache_dir == default_cache_dir({"HOME": "/h"})
    assert Settings.from_env({"SMOLTEST_CACHE_DIR": "/e", "XDG_CACHE_HOME": "/x"}).cache_dir == (
        Path("/e")
    )
    assert Settings.from_env({"XDG_CACHE_HOME": "/isolated"}, cache_dir="/o").cache_dir == Path(
        "/o"
    ), "an explicit override still wins"
    assert Settings.from_env({"XDG_CACHE_HOME": "/isolated"}, cache_dir=None).cache_dir == Path(
        "/isolated/smoltest"
    ), "a None override means no override"
    assert Settings.from_env().cache_dir == Path("/from-the-process")
    assert Settings().cache_dir == Path("/from-the-process")


def test_resolve_target_explicit_wins() -> None:
    engine = FakeEngine(availability=(False, "KVM_UNAVAILABLE", "no kvm"))
    assert resolve_target(Settings(target="local"), engine, {"SMOL_CLOUD_TOKEN": "t"}) == "local"
    assert resolve_target(Settings(target="cloud"), engine, {}) == "cloud"


def test_resolve_target_token_selects_cloud() -> None:
    engine = FakeEngine(availability=(True, None, None))
    assert resolve_target(Settings(), engine, {"SMOL_CLOUD_TOKEN": "t"}) == "cloud"
    assert resolve_target(Settings(), engine, {"SMOL_CLOUD_TOKEN": ""}) == "local"


def test_resolve_target_availability_selects_local() -> None:
    assert resolve_target(Settings(), FakeEngine(), {}) == "local"


def test_resolve_target_neither_raises_with_code_and_hint() -> None:
    engine = FakeEngine(availability=(False, "KVM_UNAVAILABLE", "KVM not available"))
    with pytest.raises(TargetUnavailable) as info:
        resolve_target(Settings(), engine, {})
    assert info.value.code == "KVM_UNAVAILABLE"
    assert info.value.reason == "KVM not available"
    assert "doctor" in str(info.value)
    assert "KVM_UNAVAILABLE" in str(info.value)


def test_get_engine_honours_settings_then_env(monkeypatch: pytest.MonkeyPatch) -> None:
    assert isinstance(get_engine(Settings(engine_path="smoltest.testing:FakeEngine")), FakeEngine)
    monkeypatch.setenv("SMOLTEST_ENGINE", "smoltest.testing:FakeEngine")
    assert isinstance(get_engine(Settings()), FakeEngine)
    assert isinstance(get_engine(), FakeEngine)


def test_get_engine_shares_the_default_fake(fake_engine: FakeEngine, settings: Settings) -> None:
    assert get_engine(settings) is fake_engine
    assert get_engine() is fake_engine


@pytest.mark.parametrize(
    "path",
    [
        "nocolon",
        "smoltest.testing",
        "smoltest.nope:FakeEngine",
        "smoltest.testing:Nope",
        "smoltest.testing:DEFAULT_LOG_PATH",
    ],
)
def test_load_engine_factory_rejects_bad_paths(path: str) -> None:
    with pytest.raises(InvalidConfig):
        load_engine_factory(path)


@pytest.mark.parametrize("raw", ["nan", "inf", "-inf"])
def test_non_finite_timeouts_are_rejected(raw: str) -> None:
    """A NaN or infinite timeout would make the readiness wait poll forever."""
    with pytest.raises(InvalidConfig, match="SMOLTEST_READY_TIMEOUT"):
        Settings.from_env({"SMOLTEST_READY_TIMEOUT": raw})
    for name in ("ready_timeout_s", "poll_interval_s", "exec_timeout_s"):
        with pytest.raises(InvalidConfig, match="finite"):
            Settings(**{name: float(raw)})


@pytest.mark.parametrize(
    ("env", "override"),
    [
        ({"SMOLTEST_TARGET": "docker"}, {"target": "local"}),
        ({"SMOLTEST_READY_TIMEOUT": "soon"}, {"ready_timeout_s": 5.0}),
        ({"TC_MAX_TRIES": "many"}, {"ready_timeout_s": 5.0}),
        ({"SMOLTEST_CPUS": "lots"}, {"cpus": 2}),
    ],
)
def test_an_explicit_override_skips_the_malformed_variable_it_replaces(
    env: dict[str, str], override: dict[str, object]
) -> None:
    settings = Settings.from_env(env, **override)
    for name, value in override.items():
        assert getattr(settings, name) == value
    with pytest.raises(InvalidConfig, match=next(iter(env))):
        Settings.from_env(env)  # without the override the variable still fails loudly


def test_a_none_override_does_not_shield_a_malformed_variable() -> None:
    with pytest.raises(InvalidConfig, match="SMOLTEST_TARGET"):
        Settings.from_env({"SMOLTEST_TARGET": "docker"}, target=None)
