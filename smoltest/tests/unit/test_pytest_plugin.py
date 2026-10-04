"""The pytest plugin, driven through pytester in this process on the FakeEngine.

Every inner session runs ``runpytest_inprocess`` so it shares the ``fake_engine``
fixture's engine (``SMOLTEST_ENGINE=smoltest.testing:FakeEngine.factory``) and the
outer test can inspect calls, SQL and leaked machines afterwards.
"""

from __future__ import annotations

import json
from pathlib import Path

import pytest

# Pin every module the inner sessions touch so pytester's sys.modules snapshot keeps the
# classes (and the GoldenRegistry / Reaper singletons) the outer assertions look at.
import smoltest.boot.golden  # noqa: F401
import smoltest.pytest_plugin as plugin
from smoltest._log import reset_warn_once
from smoltest.config import Settings
from smoltest.errors import BootError
from smoltest.postgres import Seed
from smoltest.testing import FakeEngine

RECORD_CONFTEST = """
import json
import os
from pathlib import Path

import pytest


@pytest.fixture
def record():
    out = Path(os.environ["SMOLTEST_TEST_OUT"])

    def _record(machine, **extra):
        info = machine.boot_info
        parent = machine.parent
        row = {
            "port": machine.get_exposed_port(),
            "via": info.via,
            "name": machine.name,
            "golden_port": parent.get_exposed_port() if parent is not None else None,
        }
        row.update(extra)
        with out.open("a") as fh:
            fh.write(json.dumps(row) + "\\n")

    return _record
"""


@pytest.fixture(autouse=True)
def _fresh_warnings() -> None:
    reset_warn_once()


@pytest.fixture
def records(pytester: pytest.Pytester, monkeypatch: pytest.MonkeyPatch) -> Path:
    """Install a ``record`` fixture in the inner session that appends JSON rows to a file."""
    out = pytester.path / "records.jsonl"
    monkeypatch.setenv("SMOLTEST_TEST_OUT", str(out))
    pytester.makeconftest(RECORD_CONFTEST)
    return out


def rows(path: Path) -> list[dict[str, object]]:
    return [json.loads(line) for line in path.read_text().splitlines()] if path.exists() else []


def run(pytester: pytest.Pytester, *args: str) -> pytest.RunResult:
    return pytester.runpytest_inprocess("-p", "no:cacheprovider", *args)


def created_images(engine: FakeEngine) -> list[str]:
    return sorted(str(call["spec"].image) for call in engine.ops("create"))


# -- fixtures ------------------------------------------------------------------------------


def test_postgres_fixture_branches_a_distinct_machine_per_test(
    pytester: pytest.Pytester, fake_engine: FakeEngine, records: Path
) -> None:
    pytester.makepyfile(
        """
        def test_one(postgres, record):
            assert postgres.is_running and postgres.parent is not None
            record(postgres)

        def test_two(postgres, postgres_url, record):
            assert postgres_url == postgres.get_connection_url(driver=None)
            assert postgres_url.startswith("postgresql://test:test@127.0.0.1:")
            record(postgres)

        def test_three(postgres, smoltest_postgres_golden, postgres_golden_url, record):
            golden = smoltest_postgres_golden.machine
            assert postgres.parent is golden and golden.is_running
            assert postgres_golden_url == golden.get_connection_url(driver=None)
            assert postgres.get_exposed_port() != golden.get_exposed_port()
            record(postgres)
        """
    )
    result = run(pytester)
    result.assert_outcomes(passed=3)
    seen = rows(records)
    assert [row["via"] for row in seen] == ["branch", "branch", "branch"]
    # Three distinct machines; a child never shares its live golden's port, while the
    # port of a child that already stopped may legitimately be picked again later.
    assert len({row["name"] for row in seen}) == 3
    assert all(row["port"] != row["golden_port"] for row in seen), seen
    assert len(fake_engine.ops("create")) == 1 and len(fake_engine.ops("branch")) == 3
    assert len(fake_engine.ops("delete")) == 4  # three children, then the golden
    assert fake_engine.live_machines == set()
    result.stdout.fnmatch_lines(
        [
            "smoltest: target=auto image=postgres:16 cache=* isolation=branch",
            "*golden postgres:16: booted via cold in *s on local",
            "*per-test machines: 3 branched (mean * ms), 0 fresh",
        ]
    )


def test_seed_override_is_applied_once_and_inherited(
    pytester: pytest.Pytester, fake_engine: FakeEngine
) -> None:
    pytester.makeconftest(
        """
        import pytest
        from smoltest.postgres import Seed

        @pytest.fixture(scope="session")
        def smoltest_seed_postgres():
            return Seed.from_sql("CREATE TABLE seeded (id int);")
        """
    )
    pytester.makepyfile(
        """
        def inherited(machine):
            return [s for s in machine.get_machine().inherited_sql if "seeded" in s]

        def test_a(postgres):
            assert inherited(postgres) == ["CREATE TABLE seeded (id int)"]
            assert postgres.get_machine().sql_log == []

        def test_b(postgres, smoltest_postgres_golden):
            golden = smoltest_postgres_golden
            assert golden.boot_info.seeded and golden.seed is not None
            assert postgres.boot_info.seeded_key == golden.boot_info.seeded_key
            assert inherited(postgres) == ["CREATE TABLE seeded (id int)"]
        """
    )
    result = run(pytester)
    result.assert_outcomes(passed=2)
    applied = [sql for _, sql in fake_engine.sql_log if sql.startswith("CREATE TABLE seeded")]
    assert len(applied) == 1
    assert len(fake_engine.ops("checkpoint")) == 2  # base variant, then the seeded one
    result.stdout.fnmatch_lines(["*golden postgres:16 (seed *): booted via cold in *"])
    assert fake_engine.live_machines == set()


def test_template_override_is_honoured_by_goldens(
    pytester: pytest.Pytester, fake_engine: FakeEngine
) -> None:
    pytester.makeconftest(
        """
        import pytest
        from smoltest.postgres import PostgresMachine

        @pytest.fixture(scope="session")
        def smoltest_postgres_template(smoltest_settings, smoltest_engine):
            return (
                PostgresMachine(settings=smoltest_settings, engine=smoltest_engine, driver=None)
                .with_env("TZ", "UTC")
                .with_name("golden-x")
            )
        """
    )
    pytester.makepyfile(
        """
        import pytest

        def test_default(postgres, smoltest_postgres_golden):
            assert smoltest_postgres_golden.machine.name == "golden-x"
            assert postgres.get_machine().env["TZ"] == "UTC"
            assert postgres.url.startswith("postgresql://")

        @pytest.mark.smoltest(image="postgres:15")
        def test_derived(postgres):
            assert postgres.image == "postgres:15"
            assert postgres.get_machine().env["TZ"] == "UTC"
            assert postgres.parent.name != "golden-x"
        """
    )
    result = run(pytester)
    result.assert_outcomes(passed=2)
    assert created_images(fake_engine) == ["postgres:15", "postgres:16"]
    names = [call["spec"].name for call in fake_engine.ops("create")]
    assert names.count("golden-x") == 1
    assert fake_engine.live_machines == set()


# -- isolation -----------------------------------------------------------------------------


def test_isolation_fresh_boots_standalone_machines(
    pytester: pytest.Pytester, fake_engine: FakeEngine, records: Path
) -> None:
    pytester.makeini("[pytest]\nsmoltest_isolation = fresh\n")
    pytester.makepyfile(
        """
        def test_a(postgres, smoltest_postgres_golden, record):
            golden = smoltest_postgres_golden.machine
            assert postgres.parent is None
            assert postgres.get_exposed_port() != golden.get_exposed_port()
            record(postgres)

        def test_b(postgres, smoltest_postgres_golden, record):
            golden = smoltest_postgres_golden.machine
            assert postgres.get_exposed_port() != golden.get_exposed_port()
            record(postgres)
        """
    )
    result = run(pytester)
    result.assert_outcomes(passed=2)
    seen = rows(records)
    # The first fresh machine cold-boots a second cache variant (the golden holds the first);
    # the second restores it once the first child released its claim and port.
    assert [row["via"] for row in seen] == ["cold", "restore"]
    assert fake_engine.ops("branch") == []
    result.stdout.fnmatch_lines(["*per-test machines: 0 branched, 2 fresh (mean * s)"])
    assert fake_engine.live_machines == set()


def test_isolation_shared_hands_out_the_golden(
    pytester: pytest.Pytester, fake_engine: FakeEngine, records: Path
) -> None:
    pytester.makeini("[pytest]\nsmoltest_isolation = shared\n")
    pytester.makepyfile(
        """
        def test_a(postgres, smoltest_postgres_golden, record):
            assert postgres is smoltest_postgres_golden.machine
            record(postgres)

        def test_b(postgres, record):
            assert postgres.is_running
            record(postgres)
        """
    )
    result = run(pytester)
    result.assert_outcomes(passed=2)
    seen = rows(records)
    assert len({row["port"] for row in seen}) == 1 and seen[0]["via"] == "cold"
    assert len(fake_engine.ops("create")) == 1 and fake_engine.ops("branch") == []
    result.stdout.fnmatch_lines(["*per-test machines: 0 branched, 0 fresh, 2 shared"])
    assert fake_engine.live_machines == set()


def test_marker_isolation_overrides_the_ini(
    pytester: pytest.Pytester, fake_engine: FakeEngine, records: Path
) -> None:
    pytester.makeini("[pytest]\nsmoltest_isolation = shared\n")
    pytester.makepyfile(
        """
        import pytest

        @pytest.mark.smoltest(isolation="branch")
        def test_branch(postgres, smoltest_postgres_golden, record):
            assert postgres.parent is smoltest_postgres_golden.machine
            record(postgres)

        @pytest.mark.smoltest(isolation="fresh")
        def test_fresh(postgres, record):
            assert postgres.parent is None
            record(postgres)

        def test_shared(postgres, smoltest_postgres_golden, record):
            assert postgres is smoltest_postgres_golden.machine
            record(postgres)
        """
    )
    result = run(pytester)
    result.assert_outcomes(passed=3)
    seen = rows(records)
    assert [row["via"] == "branch" for row in seen] == [True, False, False]
    # Three distinct machines; a child never shares its live golden's port, while the
    # port of a child that already stopped may legitimately be picked again later.
    assert len({row["name"] for row in seen}) == 3
    assert all(row["port"] != row["golden_port"] for row in seen), seen
    result.stdout.fnmatch_lines(["*per-test machines: 1 branched (mean * ms), 1 fresh*, 1 shared"])
    assert fake_engine.live_machines == set()


def test_no_branch_option_uses_fresh_machines(
    pytester: pytest.Pytester, fake_engine: FakeEngine, records: Path
) -> None:
    pytester.makepyfile(
        """
        def test_a(postgres, record):
            record(postgres)

        def test_b(postgres, smoltest_settings, record):
            assert smoltest_settings.disable_branch
            record(postgres)
        """
    )
    result = run(pytester, "--smoltest-no-branch")
    result.assert_outcomes(passed=2)
    assert all(row["via"] != "branch" for row in rows(records))
    assert fake_engine.ops("branch") == []
    result.stdout.fnmatch_lines(["smoltest: * isolation=branch (no branch)"])
    assert fake_engine.live_machines == set()


# -- marker goldens --------------------------------------------------------------------------


def test_marker_image_and_env_boot_distinct_goldens(
    pytester: pytest.Pytester, fake_engine: FakeEngine
) -> None:
    pytester.makepyfile(
        """
        import pytest

        def test_default(postgres, smoltest_postgres_golden):
            assert postgres.image == "postgres:16"
            assert postgres.parent is smoltest_postgres_golden.machine

        @pytest.mark.smoltest(image="postgres:15")
        def test_other_image(postgres, smoltest_postgres_golden):
            assert postgres.image == "postgres:15"
            assert postgres.parent is not smoltest_postgres_golden.machine

        @pytest.mark.smoltest(env={"PGOPTIONS": "-c work_mem=64MB"})
        def test_env(postgres, smoltest_postgres_golden):
            assert postgres.get_machine().env["PGOPTIONS"] == "-c work_mem=64MB"
            assert "PGOPTIONS" not in smoltest_postgres_golden.machine.get_machine().env
            assert postgres.parent is not smoltest_postgres_golden.machine

        @pytest.mark.smoltest(image="postgres:15")
        def test_other_image_again(postgres):
            assert postgres.image == "postgres:15"

        @pytest.mark.smoltest(env={"POSTGRES_PASSWORD": "secret", "POSTGRES_DB": "app"})
        def test_credential_env(postgres):
            env = postgres.get_machine().env
            assert (env["POSTGRES_PASSWORD"], env["POSTGRES_DB"]) == ("secret", "app")
            assert postgres.get_connection_url().startswith("postgresql+psycopg2://test:secret@")
            assert postgres.get_connection_url().endswith("/app")
            assert postgres.dbname == "app" and postgres.spec.env == {}
        """
    )
    result = run(pytester)
    result.assert_outcomes(passed=5)
    assert created_images(fake_engine) == [
        "postgres:15",
        "postgres:16",
        "postgres:16",
        "postgres:16",
    ]
    assert len(fake_engine.ops("branch")) == 5
    result.stdout.fnmatch_lines(
        [
            "*golden postgres:16: booted via cold*",
            "*golden postgres:15: booted via cold*",
            "*golden postgres:16: booted via cold*",
            "*golden postgres:16: booted via cold*",
            "*per-test machines: 5 branched*",
        ]
    )
    assert fake_engine.live_machines == set()


def test_marker_seed_boots_a_seeded_golden(
    pytester: pytest.Pytester, fake_engine: FakeEngine
) -> None:
    pytester.makepyfile(
        """
        import pytest
        from smoltest.postgres import Seed

        MARKED = Seed.from_sql("CREATE TABLE marked (id int);")

        def inherited(machine):
            return [s for s in machine.get_machine().inherited_sql if "marked" in s]

        def test_plain(postgres):
            assert inherited(postgres) == []

        @pytest.mark.smoltest(seed=MARKED)
        def test_marked(postgres):
            assert inherited(postgres) == ["CREATE TABLE marked (id int)"]

        @pytest.mark.smoltest(seed=None)
        def test_explicit_none(postgres, smoltest_postgres_golden):
            assert inherited(postgres) == []
            assert postgres.parent is smoltest_postgres_golden.machine
        """
    )
    result = run(pytester)
    result.assert_outcomes(passed=3)
    assert len(fake_engine.ops("create")) == 2
    applied = [sql for _, sql in fake_engine.sql_log if sql.startswith("CREATE TABLE marked")]
    assert len(applied) == 1
    assert fake_engine.live_machines == set()


def test_malformed_markers_fail_before_anything_boots(
    pytester: pytest.Pytester, fake_engine: FakeEngine
) -> None:
    pytester.makepyfile(
        """
        import pytest

        @pytest.mark.smoltest("postgres:15")
        def test_positional(postgres):
            pass

        @pytest.mark.smoltest(isolation="weird")
        def test_isolation(postgres):
            pass

        @pytest.mark.smoltest(bogus=1)
        def test_unknown(postgres):
            pass

        @pytest.mark.smoltest(env={"A": 1})
        def test_env(postgres):
            pass

        @pytest.mark.smoltest(seed="not a seed")
        def test_seed(postgres):
            pass
        """
    )
    result = run(pytester)
    result.assert_outcomes(errors=5)
    text = result.stdout.str()
    for fragment in (
        "takes keyword arguments only",
        "isolation must be one of branch, fresh, shared; got 'weird'",
        "unknown arguments: bogus",
        "env must be a mapping of str to str",
        "seed must be a smoltest.Seed",
    ):
        assert fragment in text
    assert fake_engine.calls == []


# -- options and ini -------------------------------------------------------------------------


def test_settings_precedence_cli_env_ini_defaults(
    pytester: pytest.Pytester, monkeypatch: pytest.MonkeyPatch
) -> None:
    pytester.makeini(
        """
        [pytest]
        smoltest_target = local
        smoltest_postgres_image = postgres:14
        smoltest_cache_dir = ini-cache
        smoltest_disable_branch = true
        smoltest_cpus = 2
        smoltest_memory_mb = 1024
        """
    )
    monkeypatch.setenv("SMOLTEST_POSTGRES_IMAGE", "postgres:15")
    monkeypatch.setenv("SMOLTEST_CPUS", "3")
    monkeypatch.delenv("SMOLTEST_CACHE_DIR")  # let the ini supply it
    config = pytester.parseconfig("--smoltest-image", "postgres:16", "--smoltest-no-cache")
    settings = plugin.settings_from_config(config)
    assert settings.postgres_image == "postgres:16"  # CLI beats env beats ini
    assert settings.cpus == 3  # env beats ini
    assert settings.memory_mb == 1024 and settings.target == "local"  # ini beats defaults
    assert settings.disable_branch and settings.disable_cache
    assert settings.cache_dir == pytester.path / "ini-cache"
    assert settings.engine_path == "smoltest.testing:FakeEngine"
    # Without the ini key the environment, then the default, supplies the cache directory.
    monkeypatch.setenv("SMOLTEST_CACHE_DIR", "/elsewhere")
    assert plugin.settings_from_config(config).cache_dir == Path("/elsewhere")


def test_every_option_reaches_settings(pytester: pytest.Pytester, tmp_path: Path) -> None:
    config = pytester.parseconfig(
        "--smoltest-target",
        "cloud",
        "--smoltest-image",
        "postgres:17",
        "--smoltest-cache-dir",
        str(tmp_path / "cli-cache"),
        "--smoltest-no-cache",
        "--smoltest-no-branch",
        "--smoltest-cpus",
        "2",
        "--smoltest-memory-mb",
        "2048",
    )
    settings = plugin.settings_from_config(config)
    assert settings == Settings(
        target="cloud",
        postgres_image="postgres:17",
        cache_dir=tmp_path / "cli-cache",
        disable_cache=True,
        disable_branch=True,
        cpus=2,
        memory_mb=2048,
        engine_path="smoltest.testing:FakeEngine",
    )
    assert plugin.settings_from_config(pytester.parseconfig()).postgres_image == "postgres:16"


def test_settings_fixture_and_header_reflect_the_options(
    pytester: pytest.Pytester, fake_engine: FakeEngine
) -> None:
    pytester.makeini("[pytest]\nsmoltest_postgres_image = postgres:14\n")
    pytester.makepyfile(
        """
        def test_settings(smoltest_settings, smoltest_engine, smoltest_postgres_template):
            assert smoltest_settings.postgres_image == "postgres:15"
            assert smoltest_settings.cpus == 2
            assert smoltest_postgres_template.image == "postgres:15"
            assert not smoltest_postgres_template.is_running
            assert smoltest_engine.sdk_version() == "fake-1.22.2"
        """
    )
    result = run(pytester, "--smoltest-image", "postgres:15", "--smoltest-cpus", "2")
    result.assert_outcomes(passed=1)
    result.stdout.fnmatch_lines(
        ["smoltest: target=auto image=postgres:15 cache=* isolation=branch"]
    )
    assert fake_engine.calls == []  # the template never boots on its own


def test_invalid_values_are_usage_errors(pytester: pytest.Pytester) -> None:
    pytester.makepyfile("def test_nothing(): pass")
    result = run(pytester, "--smoltest-cpus", "0")
    assert result.ret == pytest.ExitCode.USAGE_ERROR
    result.stderr.fnmatch_lines(["*smoltest: cpus must be >= 1; got 0*"])

    pytester.makeini("[pytest]\nsmoltest_isolation = sometimes\n")
    result = run(pytester)
    assert result.ret == pytest.ExitCode.USAGE_ERROR
    result.stderr.fnmatch_lines(["*smoltest_isolation must be one of branch, fresh, shared*"])

    pytester.makeini("[pytest]\nsmoltest_memory_mb = lots\n")
    result = run(pytester)
    assert result.ret == pytest.ExitCode.USAGE_ERROR
    result.stderr.fnmatch_lines(["*smoltest_memory_mb: expected an integer, got 'lots'*"])


def test_plugin_is_quiet_when_no_fixture_is_used(
    pytester: pytest.Pytester, fake_engine: FakeEngine
) -> None:
    pytester.makepyfile("def test_nothing(): pass")
    result = run(pytester)
    result.assert_outcomes(passed=1)
    result.stdout.fnmatch_lines(
        ["smoltest: target=auto image=postgres:16 cache=* isolation=branch"]
    )
    assert "golden" not in result.stdout.str() and "per-test machines" not in result.stdout.str()
    assert fake_engine.calls == []


def test_marker_is_registered_for_strict_markers(pytester: pytest.Pytester) -> None:
    pytester.makepyfile(
        """
        import pytest

        @pytest.mark.smoltest(isolation="shared")
        def test_marked():
            pass
        """
    )
    result = run(pytester, "--strict-markers")
    result.assert_outcomes(passed=1)
    result = run(pytester, "--markers")
    result.stdout.fnmatch_lines(["@pytest.mark.smoltest(image=None, env=None, isolation=None*"])


# -- unavailable target ----------------------------------------------------------------------


UNAVAILABLE = (False, "KVM_UNAVAILABLE", "/dev/kvm is missing")


def test_unavailable_target_fails_with_the_doctor_hint(
    pytester: pytest.Pytester, fake_engine: FakeEngine
) -> None:
    fake_engine.availability = UNAVAILABLE
    pytester.makepyfile(
        """
        def test_a(postgres):
            pass

        def test_b(postgres_url):
            pass
        """
    )
    result = run(pytester)
    result.assert_outcomes(errors=2)
    text = result.stdout.str()
    assert "KVM_UNAVAILABLE" in text and "/dev/kvm is missing" in text
    assert "smoltest doctor" in text
    assert fake_engine.ops("create") == []


def test_unavailable_target_skips_when_configured(
    pytester: pytest.Pytester, fake_engine: FakeEngine
) -> None:
    fake_engine.availability = UNAVAILABLE
    pytester.makeini("[pytest]\nsmoltest_skip_if_unavailable = true\n")
    pytester.makepyfile(
        """
        import pytest

        def test_a(postgres):
            pass

        @pytest.mark.smoltest(image="postgres:15")
        def test_b(postgres):
            pass

        def test_c():
            pass
        """
    )
    result = run(pytester, "-rs")
    result.assert_outcomes(skipped=2, passed=1)
    result.stdout.fnmatch_lines(["*SKIPPED*smoltest: no Smol target available (KVM_UNAVAILABLE)*"])
    assert fake_engine.ops("create") == []


def test_explicit_target_ignores_availability(
    pytester: pytest.Pytester, fake_engine: FakeEngine
) -> None:
    fake_engine.availability = UNAVAILABLE
    pytester.makepyfile(
        """
        def test_a(postgres):
            assert postgres.boot_info.target == "local"
        """
    )
    result = run(pytester, "--smoltest-target", "local")
    result.assert_outcomes(passed=1)
    assert fake_engine.live_machines == set()


# -- leaks and teardown ----------------------------------------------------------------------


def test_boot_failure_leaves_nothing_behind(
    pytester: pytest.Pytester, fake_engine: FakeEngine
) -> None:
    fake_engine.fail_next("create", BootError("disk full", stage="create"))
    pytester.makepyfile(
        """
        def test_a(postgres):
            pass

        def test_b(postgres):
            pass
        """
    )
    result = run(pytester)
    result.assert_outcomes(errors=2)
    assert "[create] disk full" in result.stdout.str()
    assert len(fake_engine.ops("create")) == 1  # the session fixture's failure is cached
    assert fake_engine.live_machines == set()


def test_marker_goldens_are_closed_at_session_end_children_first(
    pytester: pytest.Pytester, fake_engine: FakeEngine
) -> None:
    pytester.makepyfile(
        """
        import pytest

        @pytest.mark.smoltest(image="postgres:15", isolation="shared")
        def test_shared_other(postgres):
            assert postgres.is_running

        @pytest.mark.smoltest(image="postgres:15")
        def test_branch_other(postgres):
            assert postgres.parent is not None
        """
    )
    result = run(pytester)
    result.assert_outcomes(passed=2)
    machines = [call["machine"] for call in fake_engine.ops("delete")]
    created = [call["spec"].image for call in fake_engine.ops("create")]
    assert created == ["postgres:15"]  # no session golden was booted for these tests
    assert len(machines) == 2  # the branch child, then its golden
    assert fake_engine.live_machines == set()
    assert fake_engine.machines[machines[-1]].via == "create"


def test_seed_type_is_the_one_the_marker_checks() -> None:
    options = plugin.MarkerOptions(seed=Seed.from_sql("SELECT 1"), seed_given=True)
    assert options.wants_own_golden
    assert not plugin.MarkerOptions(isolation="fresh").wants_own_golden
    assert plugin.ISOLATIONS == ("branch", "fresh", "shared")
