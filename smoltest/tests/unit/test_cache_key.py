"""Cache keys are deterministic, flip on every shaping input and never on the host port."""

from __future__ import annotations

import hashlib
import json
from collections.abc import Callable
from pathlib import Path
from typing import Any

import pytest

from smoltest.boot.spec import PostgresSpec
from smoltest.cache.key import (
    DEFAULT_CLOUD_URL,
    KEY_LENGTH,
    CacheKey,
    HostSignature,
    canonical_json,
    cloud_base_url,
    compute_host_signature,
    digest,
    host_signature,
    is_cache_key,
    read_cpu_flags,
    redact_inputs,
    smoltest_major,
)
from smoltest.config import Settings
from smoltest.testing import FakeEngine
from smoltest.transport.base import MachineSpec, PortMapping

HOST = HostSignature("linux", "6", "x86_64", "a" * 16, "glibc-2.39")
OTHER_HOST = HostSignature("linux", "6", "x86_64", "b" * 16, "glibc-2.39")


def key_for(
    spec: PostgresSpec | None = None,
    *,
    settings: Settings | None = None,
    target: str = "local",
    engine: FakeEngine | None = None,
    seed_key: str | None = None,
    env: dict[str, str] | None = None,
) -> CacheKey:
    settings = settings or Settings(target="local", cache_dir=Path("/tmp/unused"))
    return CacheKey.compute(
        spec or PostgresSpec(),
        settings,
        target,  # type: ignore[arg-type]
        engine or FakeEngine(),
        seed_key,
        host=HOST,
        env=env or {},
    )


def test_key_is_deterministic_and_well_formed() -> None:
    a, b = key_for(), key_for()
    assert a == b and a.key == b.key and str(a) == a.key
    assert len(a.key) == KEY_LENGTH and is_cache_key(a.key)
    assert a.key == hashlib.sha256(a.canonical_json().encode()).hexdigest()[:KEY_LENGTH]
    assert a.canonical_json() == canonical_json(a.inputs)
    assert json.loads(a.canonical_json())["format"] == 1
    assert a.target == "local" and a.seed_key is None


@pytest.mark.parametrize(
    ("label", "mutate"),
    [
        ("image", lambda: key_for(PostgresSpec(image="postgres:15"))),
        ("fast", lambda: key_for(PostgresSpec(fast=False))),
        ("capture_logs", lambda: key_for(PostgresSpec(capture_logs=True))),
        ("command", lambda: key_for(PostgresSpec(command=("postgres",)))),
        ("env", lambda: key_for(PostgresSpec(env={"TZ": "UTC"}))),
        ("username", lambda: key_for(PostgresSpec(username="alice"))),
        ("password", lambda: key_for(PostgresSpec(password="hunter2"))),
        ("dbname", lambda: key_for(PostgresSpec(dbname="app"))),
        ("guest_port", lambda: key_for(PostgresSpec(guest_port=5433))),
        ("extra_ports", lambda: key_for(PostgresSpec(extra_ports=(PortMapping(None, 8080),)))),
        ("cpus", lambda: key_for(PostgresSpec(cpus=2))),
        ("memory_mb", lambda: key_for(PostgresSpec(memory_mb=1024))),
        ("storage_gb", lambda: key_for(PostgresSpec(storage_gb=4))),
        ("network", lambda: key_for(PostgresSpec(network=False))),
        ("settings.image", lambda: key_for(settings=Settings(postgres_image="postgres:17"))),
        ("settings.cpus", lambda: key_for(settings=Settings(cpus=3))),
        ("settings.fast", lambda: key_for(settings=Settings(fast_mode=False))),
        ("seed_key", lambda: key_for(seed_key="schema-v1")),
        ("sdk_version", lambda: key_for(engine=FakeEngine(sdk_version="fake-9.9.9"))),
        ("target", lambda: key_for(target="cloud")),
    ],
)
def test_every_shaping_input_flips_the_key(label: str, mutate: Callable[[], CacheKey]) -> None:
    base = key_for()
    changed = mutate()
    assert changed.key != base.key, label
    assert changed.inputs != base.inputs, label


def test_seed_keys_differ_from_each_other_and_from_base() -> None:
    base, s1, s2 = key_for(), key_for(seed_key="a"), key_for(seed_key="b")
    assert len({base.key, s1.key, s2.key}) == 3
    assert s1.seed_key == "a" and base.with_seed("a") == s1 and s1.with_seed(None) == base


def test_host_signature_matters_locally_but_not_on_cloud() -> None:
    settings = Settings(target="local", cache_dir=Path("/tmp/unused"))
    engine = FakeEngine()
    local_a = CacheKey.compute(PostgresSpec(), settings, "local", engine, host=HOST, env={})
    local_b = CacheKey.compute(PostgresSpec(), settings, "local", engine, host=OTHER_HOST, env={})
    assert local_a.key != local_b.key
    assert local_a.inputs["host"] == HOST.as_dict() and local_a.inputs["base_url"] is None
    cloud_a = CacheKey.compute(PostgresSpec(), settings, "cloud", engine, host=HOST, env={})
    cloud_b = CacheKey.compute(PostgresSpec(), settings, "cloud", engine, host=OTHER_HOST, env={})
    assert cloud_a.key == cloud_b.key
    assert cloud_a.inputs["host"] is None and cloud_a.inputs["base_url"] == DEFAULT_CLOUD_URL


def test_cloud_base_url_is_part_of_cloud_keys() -> None:
    default = key_for(target="cloud")
    other = key_for(target="cloud", env={"SMOL_CLOUD_URL": "https://eu.example.test/"})
    assert other.inputs["base_url"] == "https://eu.example.test"
    assert other.key != default.key
    assert cloud_base_url({}) == DEFAULT_CLOUD_URL
    assert cloud_base_url({"SMOL_CLOUD_URL": " https://x.test// "}) == "https://x.test"
    assert cloud_base_url({"SMOL_CLOUD_URL": ""}) == DEFAULT_CLOUD_URL


def test_host_port_is_not_part_of_the_key() -> None:
    def mspec(host: int | None) -> MachineSpec:
        return MachineSpec(
            image="postgres:16",
            argv=("docker-entrypoint.sh", "postgres"),
            env={"POSTGRES_PASSWORD": "test"},
            ports=(PortMapping(host, 5432),),
        )

    a = CacheKey.from_machine_spec(mspec(21000), "local", "fake", host=HOST)
    b = CacheKey.from_machine_spec(mspec(22000), "local", "fake", host=HOST)
    c = CacheKey.from_machine_spec(mspec(None), "local", "fake", host=HOST)
    assert a.key == b.key == c.key
    assert a.inputs["guest_ports"] == [5432]
    assert "ports" not in a.inputs and "host_port" not in a.inputs


def test_env_order_does_not_matter() -> None:
    a = key_for(PostgresSpec(env={"A": "1", "B": "2"}))
    b = key_for(PostgresSpec(env={"B": "2", "A": "1"}))
    assert a.key == b.key


def test_redaction_hashes_password_like_values_only() -> None:
    key = key_for(PostgresSpec(password="s3cret", env={"API_TOKEN": "t", "TZ": "UTC"}))
    env = key.inputs["env"]
    assert env["POSTGRES_PASSWORD"] == "s3cret"  # unredacted in memory
    redacted = key.redacted_inputs()
    assert redacted["env"]["POSTGRES_PASSWORD"] == f"sha256:{digest('s3cret', 8)}"
    assert redacted["env"]["API_TOKEN"] == f"sha256:{digest('t', 8)}"
    assert redacted["env"]["TZ"] == "UTC"
    assert redacted["env"]["POSTGRES_USER"] == "test"
    assert redacted["image"] == key.inputs["image"]
    assert "s3cret" not in json.dumps(redacted)
    assert len(redacted["env"]["POSTGRES_PASSWORD"]) == len("sha256:") + 8


def test_redact_inputs_recurses_and_copies() -> None:
    inputs: dict[str, Any] = {
        "nested": {"Passwd": "x", "list": [{"secret_key": "y"}, "z"], "n": 1},
        "tuple": ("a", "b"),
        "password": 42,  # not a string: left alone
    }
    out = redact_inputs(inputs)
    assert out["nested"]["Passwd"].startswith("sha256:")
    assert out["nested"]["list"][0]["secret_key"].startswith("sha256:")
    assert out["nested"]["list"][1] == "z" and out["nested"]["n"] == 1
    assert out["tuple"] == ["a", "b"] and out["password"] == 42
    assert inputs["nested"]["Passwd"] == "x"  # the original is untouched


def test_compute_host_signature_from_cpuinfo(tmp_path: Path) -> None:
    cpuinfo = tmp_path / "cpuinfo"
    cpuinfo.write_text("processor\t: 0\nflags\t\t: sse2 avx fpu avx\nbugs\t: none\n")
    flags = read_cpu_flags(cpuinfo)
    assert flags == ("avx", "fpu", "sse2")
    sig = compute_host_signature(cpuinfo)
    assert sig.cpu_flags_hash == digest("avx fpu sse2", 16)
    assert sig.os and sig.arch and sig.kernel_major.isdigit()
    assert set(sig.as_dict()) == {"os", "kernel_major", "arch", "cpu_flags_hash", "libc"}
    # arm64 spells the line differently
    cpuinfo.write_text("Features\t: fp asimd evtstrm\n")
    assert read_cpu_flags(cpuinfo) == ("asimd", "evtstrm", "fp")
    # the hash feeds the key: different flags, different key
    a = compute_host_signature(cpuinfo)
    assert a.cpu_flags_hash != sig.cpu_flags_hash


def test_host_signature_falls_back_without_cpuinfo(tmp_path: Path) -> None:
    missing = tmp_path / "nope"
    assert read_cpu_flags(missing) is None
    sig = compute_host_signature(missing)
    assert len(sig.cpu_flags_hash) == 16
    assert compute_host_signature(missing) == sig
    assert isinstance(host_signature(), HostSignature) and host_signature() is host_signature()


def test_smoltest_major_parsing() -> None:
    assert smoltest_major("0.1.0") == 0
    assert smoltest_major("12.3.4rc1") == 12
    assert smoltest_major("0.0.0+unknown") == 0
    assert smoltest_major("garbage") == 0
    assert isinstance(key_for().inputs["smoltest_major"], int)


def test_from_inputs_round_trip() -> None:
    key = key_for()
    again = CacheKey.from_inputs(dict(key.inputs))
    assert again == key
    assert CacheKey.from_inputs({**key.inputs, "image": "other"}).key != key.key
