"""PostgresSpec -> argv/env/MachineSpec translation."""

from __future__ import annotations

from typing import Any

import pytest

from smoltest.boot.spec import (
    CAPTURE_LOGS_ARGS,
    CREDENTIAL_VARS,
    FAST_ARGS,
    LOG_PATH,
    PostgresSpec,
    build_machine_spec,
    default_argv,
)
from smoltest.config import Settings
from smoltest.errors import InvalidConfig
from smoltest.transport.base import MachineSpec, PortMapping


def test_default_argv_fast_and_capture_combinations() -> None:
    base = ("docker-entrypoint.sh", "postgres")
    assert default_argv(PostgresSpec(fast=False)) == base
    assert default_argv(PostgresSpec(fast=True)) == base + FAST_ARGS
    assert default_argv(PostgresSpec(fast=False, capture_logs=True)) == base + CAPTURE_LOGS_ARGS
    assert default_argv(PostgresSpec(capture_logs=True)) == base + FAST_ARGS + CAPTURE_LOGS_ARGS
    assert LOG_PATH == "/tmp/postgresql.log"  # /tmp exists in the Debian and alpine images
    assert (
        "-c" in CAPTURE_LOGS_ARGS
        and f"log_directory={LOG_PATH.rsplit('/', 1)[0]}" in CAPTURE_LOGS_ARGS
    )


def test_fast_none_follows_settings() -> None:
    spec = PostgresSpec()
    assert FAST_ARGS[1] in default_argv(spec, Settings(fast_mode=True))
    assert FAST_ARGS[1] not in default_argv(spec, Settings(fast_mode=False))
    assert FAST_ARGS[1] in default_argv(PostgresSpec(fast=True), Settings(fast_mode=False))


def test_effective_env_layers_credentials_initdb_and_user_env() -> None:
    settings = Settings()
    env = PostgresSpec(username="u", password="p", dbname="d").effective_env(settings)
    assert env == {
        "POSTGRES_USER": "u",
        "POSTGRES_PASSWORD": "p",
        "POSTGRES_DB": "d",
        "POSTGRES_INITDB_ARGS": "--no-sync",
    }
    slow = PostgresSpec(fast=False, env={"TZ": "UTC"}).effective_env(settings)
    assert "POSTGRES_INITDB_ARGS" not in slow and slow["TZ"] == "UTC"
    merged = PostgresSpec(env={"POSTGRES_INITDB_ARGS": "--data-checksums"}).effective_env(settings)
    assert merged["POSTGRES_INITDB_ARGS"] == "--data-checksums --no-sync"
    again = PostgresSpec(env={"POSTGRES_INITDB_ARGS": "--no-sync"}).effective_env(settings)
    assert again["POSTGRES_INITDB_ARGS"] == "--no-sync"


def test_command_overrides_default_argv() -> None:
    spec = PostgresSpec(command=["postgres", "-c", "max_connections=5"])  # type: ignore[arg-type]
    assert spec.command == ("postgres", "-c", "max_connections=5")
    assert spec.effective_argv(Settings()) == spec.command


def test_spec_validation() -> None:
    with pytest.raises(InvalidConfig):
        PostgresSpec(guest_port=0)
    with pytest.raises(InvalidConfig):
        PostgresSpec(extra_ports=[PortMapping(None, 5432)])  # type: ignore[arg-type]


def test_build_machine_spec_local() -> None:
    settings = Settings(cpus=2, memory_mb=256, ready_timeout_s=30)
    spec = PostgresSpec(image="postgres:15", extra_ports=(PortMapping(None, 8080),))
    ms = build_machine_spec(spec, settings, "local", 24001, "golden")
    assert isinstance(ms, MachineSpec)
    assert ms.image == "postgres:15"
    assert ms.argv == default_argv(spec, settings)
    assert ms.env["POSTGRES_USER"] == "test"
    assert ms.ports == (PortMapping(24001, 5432), PortMapping(None, 8080))
    assert ms.guest_ports == (5432, 8080)
    assert (ms.cpus, ms.memory_mb, ms.storage_gb, ms.network) == (2, 256, None, True)
    assert ms.branchable is True and ms.name == "golden"
    assert ms.ready_timeout_s == 30 and ms.wait_for_ports is True
    assert ms.auto_stop_seconds is None and ms.ttl_seconds is None


def test_build_machine_spec_cloud_and_overrides() -> None:
    settings = Settings(cloud_auto_stop_seconds=10, cloud_ttl_seconds=20)
    spec = PostgresSpec(cpus=4, memory_mb=2048, storage_gb=3, network=False)
    ms = build_machine_spec(spec, settings, "cloud", None, branchable=False, extra_ports=[])
    assert ms.ports == (PortMapping(None, 5432),)
    assert (ms.cpus, ms.memory_mb, ms.storage_gb, ms.network) == (4, 2048, 3, False)
    assert ms.branchable is False
    assert (ms.auto_stop_seconds, ms.ttl_seconds) == (10, 20)
    assert ms.image == "postgres:16"


def test_machine_spec_coerces_sequences() -> None:
    lenient: dict[str, Any] = {"argv": ["a", "b"], "ports": [PortMapping(1, 2)], "env": {"k": "v"}}
    ms = MachineSpec(image="postgres:16", **lenient)
    assert ms.argv == ("a", "b") and ms.ports == (PortMapping(1, 2),)
    assert ms == MachineSpec(
        image="postgres:16", argv=("a", "b"), ports=(PortMapping(1, 2),), env={"k": "v"}
    )


def test_credential_env_vars_fold_into_the_spec_fields() -> None:
    assert set(CREDENTIAL_VARS) == {"POSTGRES_USER", "POSTGRES_PASSWORD", "POSTGRES_DB"}
    spec = PostgresSpec(env={"POSTGRES_PASSWORD": "secret", "POSTGRES_DB": "app", "TZ": "UTC"})
    assert (spec.username, spec.password, spec.dbname) == ("test", "secret", "app")
    assert spec.env == {"TZ": "UTC"}
    env = spec.effective_env(Settings())
    assert (env["POSTGRES_USER"], env["POSTGRES_PASSWORD"], env["POSTGRES_DB"]) == (
        "test",
        "secret",
        "app",
    )
    assert env["TZ"] == "UTC"
    # dataclasses.replace (the builders and the pytest marker) goes through the same fold.
    import dataclasses

    again = dataclasses.replace(spec, env={**spec.env, "POSTGRES_USER": "alice"})
    assert again.username == "alice" and "POSTGRES_USER" not in again.env
    assert again.password == "secret"


def test_build_machine_spec_carries_the_exec_timeout() -> None:
    mspec = build_machine_spec(PostgresSpec(), Settings(exec_timeout_s=33.0), "local", None)
    assert mspec.exec_timeout_s == 33.0
    with pytest.raises(InvalidConfig, match="exec_timeout_s"):
        Settings(exec_timeout_s=0)
