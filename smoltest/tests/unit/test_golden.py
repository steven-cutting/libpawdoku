"""PostgresGolden and GoldenRegistry against the FakeEngine."""

from __future__ import annotations

import warnings

import pytest

from smoltest._log import reset_warn_once
from smoltest._ports import pick_free_port
from smoltest.boot.golden import GoldenKey, GoldenRegistry, PostgresGolden, template_key
from smoltest.config import Settings
from smoltest.errors import NotSupportedError, SmoltestError, SmoltestWarning
from smoltest.postgres import PostgresMachine, Seed
from smoltest.testing import FakeEngine, FakeMachine
from smoltest.wait.strategies import PortWaitStrategy


@pytest.fixture(autouse=True)
def _fresh_warnings() -> None:
    reset_warn_once()


@pytest.fixture
def template(settings: Settings) -> PostgresMachine:
    return PostgresMachine(settings=settings)


def fake_of(machine: PostgresMachine) -> FakeMachine:
    handle = machine.get_machine()
    assert isinstance(handle, FakeMachine)
    return handle


def deleted_ids(engine: FakeEngine) -> list[str]:
    return [call["machine"] for call in engine.ops("delete")]


# -- PostgresGolden -----------------------------------------------------------------------


def test_boot_and_branch(fake_engine: FakeEngine, template: PostgresMachine) -> None:
    golden = PostgresGolden.boot(template)
    assert golden.machine.is_running and golden.boot_info.via == "cold"
    assert golden.branch_supported and not golden.children
    assert golden.machine.boot_info is golden.boot_info and golden.machine.driver == "psycopg2"
    golden.machine.psql("CREATE TABLE g (id int)")
    child = golden.branch()
    assert child.is_running and child.boot_info is not None
    assert child.boot_info.via == "branch" and child.parent is golden.machine
    assert child.get_exposed_port() != golden.machine.get_exposed_port()
    fake = fake_of(child)
    assert fake.services == {"postgres"} and fake.parent_id == fake_of(golden.machine).id
    assert fake.inherited_sql == ["CREATE TABLE g (id int)"]
    assert child.url.startswith("postgresql+psycopg2://test:test@127.0.0.1:")
    assert list(golden.children) == [child] and golden.branch_count == 1 and golden.fresh_count == 0
    named = golden.branch("named-child")
    assert named.name == "named-child" and len(golden.children) == 2
    child.stop()
    assert list(golden.children) == [named]
    machine = golden.machine
    golden.close()
    assert golden.closed and not machine.is_running and not named.is_running
    assert fake_engine.live_machines == set()
    golden.close()  # idempotent
    with pytest.raises(SmoltestError, match="closed"):
        golden.branch()


def test_boot_requires_an_unstarted_template(
    fake_engine: FakeEngine, template: PostgresMachine
) -> None:
    template.start()
    try:
        with pytest.raises(SmoltestError, match="unstarted"):
            PostgresGolden.boot(template)
    finally:
        template.stop()


def test_boot_honours_template_name_pin_and_wait(
    fake_engine: FakeEngine, settings: Settings
) -> None:
    from smoltest._ports import pick_free_port
    from smoltest.wait.strategies import PortWaitStrategy

    port = pick_free_port()
    template = (
        PostgresMachine(settings=settings, driver=None)
        .with_name("golden-x")
        .with_bind_ports(5432, port)
        .waiting_for(PortWaitStrategy())
    )
    with PostgresGolden.boot(template) as golden:
        assert golden.machine.name == "golden-x" and golden.machine.get_exposed_port() == port
        assert golden.machine.url.startswith("postgresql://")
        assert all(argv[0] != "pg_isready" for argv in fake_of(golden.machine).exec_log)
        assert fake_engine.ops("create")[0]["spec"].name == "golden-x"
    assert fake_engine.live_machines == set()


def test_seed_is_applied_once_and_inherited(fake_engine: FakeEngine, settings: Settings) -> None:
    seed = Seed.from_sql("CREATE TABLE s (id int);")
    template = PostgresMachine(settings=settings)
    with PostgresGolden.boot(template, seed=seed) as golden:
        assert golden.seed is seed and golden.machine.seed is seed
        assert golden.boot_info.seeded and golden.boot_info.seeded_key is not None
        child = golden.branch()
        assert "CREATE TABLE s (id int)" in fake_of(child).inherited_sql
        assert fake_of(child).sql_log == []
    assert fake_engine.live_machines == set()
    # A second golden with the same seed restores the seeded checkpoint instead.
    with PostgresGolden.boot(PostgresMachine(settings=settings, seed=seed)) as again:
        assert again.seed is seed
        assert again.boot_info.via == "restore" and not again.boot_info.seeded
        assert again.boot_info.restored_key == again.boot_info.seeded_key


def test_children_carry_the_seed_so_a_restart_boots_seeded(
    fake_engine: FakeEngine, settings: Settings
) -> None:
    seed = Seed.from_sql("CREATE TABLE s (id int);")
    with PostgresGolden.boot(PostgresMachine(settings=settings), seed=seed) as golden:
        branch = golden.branch()
        fresh = golden.fresh()
        assert branch.seed is seed and fresh.seed is seed
        branch.stop()
        fresh.stop()
        fresh.start()  # a stop()/start() cycle re-runs the ladder with the seed
        try:
            info = fresh.boot_info
            assert info is not None and info.seeded_key is not None
            assert info.seeded or info.restored_key == info.seeded_key
            assert "CREATE TABLE s (id int)" in (
                fake_of(fresh).sql_log + fake_of(fresh).inherited_sql
            )
        finally:
            fresh.stop()
    assert fake_engine.live_machines == set()


def test_not_supported_falls_back_to_fresh_with_one_warning(
    fake_engine: FakeEngine, template: PostgresMachine
) -> None:
    with PostgresGolden.boot(template) as golden:
        fake_engine.fail_next("branch", NotSupportedError("branching is off"))
        with pytest.warns(SmoltestWarning, match="falling back to fresh machines"):
            first = golden.branch()
        assert first.boot_info is not None and first.boot_info.via in ("cold", "restore")
        assert first.parent is None and first.is_running
        assert not golden.branch_supported and golden.fresh_count == 1
        with warnings.catch_warnings():
            warnings.simplefilter("error")
            second = golden.branch()
        assert second.boot_info is not None and second.boot_info.via in ("cold", "restore")
        assert len(fake_engine.ops("branch")) == 1 and golden.fresh_count == 2
        assert set(golden.children) == {first, second}
        assert len(fake_engine.live_machines) == 3
        # branch_ok=False was recorded on the golden's cache variant.
        claim = golden._result.claim
        assert claim is not None and claim.meta is not None and claim.meta.branch_ok is False
    assert fake_engine.live_machines == set()


def test_disable_branch_setting_uses_fresh_machines(
    fake_engine: FakeEngine, settings: Settings
) -> None:
    template = PostgresMachine(settings=settings.replace(disable_branch=True))
    with PostgresGolden.boot(template) as golden:
        assert not golden.branch_supported
        with warnings.catch_warnings():
            warnings.simplefilter("error")
            child = golden.branch()
        assert child.boot_info is not None and child.boot_info.via in ("cold", "restore")
        assert fake_engine.ops("branch") == [] and golden.fresh_count == 1
        assert child.get_exposed_port() != golden.machine.get_exposed_port()
    assert fake_engine.live_machines == set()


def test_engine_without_branch_support_warns_once(settings: Settings) -> None:
    engine = FakeEngine(supports_branch=False)
    template = PostgresMachine(settings=settings, engine=engine)
    with PostgresGolden.boot(template) as golden:
        assert not golden.branch_supported
        with pytest.warns(SmoltestWarning, match="not supported"):
            golden.branch()
        with warnings.catch_warnings():
            warnings.simplefilter("error")
            golden.branch()
        assert engine.ops("branch") == [] and golden.fresh_count == 2
    assert engine.live_machines == set()


def test_fresh_boots_from_the_template(fake_engine: FakeEngine, template: PostgresMachine) -> None:
    with PostgresGolden.boot(template) as golden:
        fresh = golden.fresh("fresh-1")
        assert fresh.name == "fresh-1" and fresh.boot_info is not None
        assert fresh.boot_info.via == "cold"  # the golden holds the only variant's claim
        assert fresh.boot_info.cache_key == golden.boot_info.cache_key
        assert list(golden.children) == [fresh] and golden.branch_supported
    assert fake_engine.live_machines == set()


def test_close_stops_children_before_the_golden(
    fake_engine: FakeEngine, template: PostgresMachine
) -> None:
    golden = PostgresGolden.boot(template)
    a, b = golden.branch(), golden.branch()
    fresh = golden.fresh()
    ids = {m: fake_of(m).id for m in (golden.machine, a, b, fresh)}
    golden.close()
    deleted = deleted_ids(fake_engine)
    assert deleted == [ids[fresh], ids[b], ids[a], ids[golden.machine]]
    assert fake_engine.live_machines == set()
    assert not a.is_running and not b.is_running and not fresh.is_running


def test_close_survives_a_failing_child(fake_engine: FakeEngine, template: PostgresMachine) -> None:
    golden = PostgresGolden.boot(template)
    child = golden.branch()
    fake_engine.fail_next("checkpoint", SmoltestError("unrelated"))  # not hit by stop()
    fake_of(child).delete()  # simulate the engine already having lost it
    golden.close()
    assert fake_engine.live_machines == set() and golden.closed


def test_golden_repr(fake_engine: FakeEngine, template: PostgresMachine) -> None:
    with PostgresGolden.boot(template) as golden:
        assert repr(golden).startswith("PostgresGolden(PostgresMachine(") and "open" in repr(golden)
    assert "closed" in repr(golden)


# -- GoldenRegistry -----------------------------------------------------------------------


def test_registry_dedups_by_spec_and_seed(fake_engine: FakeEngine, settings: Settings) -> None:
    registry = GoldenRegistry()
    seed = Seed.from_sql("CREATE TABLE r (id int);")
    plain = registry.get_or_boot(PostgresMachine(settings=settings))
    assert registry.get_or_boot(PostgresMachine(settings=settings)) is plain
    assert registry.get(PostgresMachine(settings=settings)) is plain
    seeded = registry.get_or_boot(PostgresMachine(settings=settings), seed)
    assert seeded is not plain and seeded.seed is seed
    assert registry.get_or_boot(PostgresMachine(settings=settings, seed=seed)) is seeded
    other = registry.get_or_boot(PostgresMachine("postgres:15", settings=settings))
    assert other is not plain and len(registry) == 3
    assert set(registry.goldens) == {plain, seeded, other}
    assert len(fake_engine.ops("create")) == 3 and fake_engine.ops("branch") == []
    assert registry.get(PostgresMachine("postgres:17", settings=settings)) is None
    registry.close_all()
    assert all(g.closed for g in (plain, seeded, other)) and len(registry) == 0
    assert fake_engine.live_machines == set()
    registry.close_all()  # idempotent
    reborn = registry.get_or_boot(PostgresMachine(settings=settings))
    assert reborn is not plain and reborn.boot_info.via == "restore"
    registry.close_all()
    assert fake_engine.live_machines == set()


def test_registry_key_uses_the_machine_spec(fake_engine: FakeEngine, settings: Settings) -> None:
    a = PostgresMachine(settings=settings)
    b = PostgresMachine(settings=settings).with_env("TZ", "UTC")
    assert template_key(a) == template_key(PostgresMachine(settings=settings))
    assert template_key(a) != template_key(b)
    key = GoldenRegistry.key_for(a, None)
    assert isinstance(key, GoldenKey)
    assert key.cache_key == template_key(a) and key.seed_key is None
    assert key == GoldenRegistry.key_for(PostgresMachine(settings=settings), None)
    assert GoldenRegistry.key_for(a, Seed.from_sql("x", key="k")).seed_key == "k"
    assert fake_engine.calls == []  # computing keys boots nothing


def test_registry_separates_templates_that_differ_at_runtime(
    fake_engine: FakeEngine, settings: Settings
) -> None:
    """The same checkpoint key is not enough: runtime options must match too."""
    registry = GoldenRegistry()
    base = registry.get_or_boot(PostgresMachine(settings=settings))
    port = pick_free_port()
    pinned = registry.get_or_boot(PostgresMachine(settings=settings).with_bind_ports(5432, port))
    assert pinned is not base and pinned.machine.get_exposed_port() == port
    no_branch = registry.get_or_boot(
        PostgresMachine(settings=settings.replace(disable_branch=True))
    )
    assert no_branch is not base and not no_branch.branch_supported and base.branch_supported
    bare = registry.get_or_boot(PostgresMachine(settings=settings, driver=None))
    assert bare is not base and bare.machine.url.startswith("postgresql://")
    quick = registry.get_or_boot(PostgresMachine(settings=settings.replace(exec_timeout_s=30.0)))
    assert quick is not base
    strategy = PortWaitStrategy()
    waiting = registry.get_or_boot(PostgresMachine(settings=settings).waiting_for(strategy))
    assert waiting is not base
    named = registry.get_or_boot(PostgresMachine(settings=settings).with_name("golden-x"))
    assert named is not base and named.machine.name == "golden-x"
    assert registry.get_or_boot(PostgresMachine(settings=settings)) is base
    assert len(registry) == 7
    # All of them share one checkpoint key: the cache, not the registry, decides reuse there.
    assert len({GoldenRegistry.key_for(g.machine, None).cache_key for g in registry.goldens}) == 1
    registry.close_all()
    assert fake_engine.live_machines == set()


def test_registry_separates_templates_with_different_engines(
    fake_engine: FakeEngine, settings: Settings
) -> None:
    """Two engines reporting the same version and target still get their own goldens."""
    other = FakeEngine()
    registry = GoldenRegistry()
    first = registry.get_or_boot(PostgresMachine(settings=settings, engine=fake_engine))
    second = registry.get_or_boot(PostgresMachine(settings=settings, engine=other))
    assert second is not first and second.engine is other and first.engine is fake_engine
    assert registry.get_or_boot(PostgresMachine(settings=settings, engine=other)) is second
    assert len(registry) == 2
    registry.close_all()
    assert fake_engine.live_machines == set() and other.live_machines == set()


def test_fresh_prunes_stopped_children(fake_engine: FakeEngine, template: PostgresMachine) -> None:
    golden = PostgresGolden.boot(template)
    first = golden.fresh()
    first.stop()
    second = golden.fresh()
    assert list(golden.children) == [second]
    golden.close()
    assert fake_engine.live_machines == set()


def test_registry_instance_is_a_singleton() -> None:
    assert GoldenRegistry.instance() is GoldenRegistry.instance()
    assert isinstance(GoldenRegistry.instance(), GoldenRegistry)


def test_registry_closes_all_through_the_reaper(
    fake_engine: FakeEngine, settings: Settings
) -> None:
    registry = GoldenRegistry()
    golden = registry.get_or_boot(PostgresMachine(settings=settings))
    child = golden.branch()
    registry._token()  # what the Reaper runs at exit
    assert golden.closed and not child.is_running and fake_engine.live_machines == set()
