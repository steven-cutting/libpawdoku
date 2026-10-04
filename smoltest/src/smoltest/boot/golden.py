"""Golden machines: boot once, branch per test, fall back to fresh machines.

A :class:`PostgresGolden` wraps one booted :class:`~smoltest.postgres.PostgresMachine`
and hands out isolated children: copy-on-write branches when the engine supports
them, otherwise fresh machines booted from the same template (a restore of
another cache variant, or a cold boot). :class:`GoldenRegistry` keeps one golden
per (machine spec, seed) in the process and closes them all at exit.
"""

from __future__ import annotations

import threading
import weakref
from collections.abc import Callable
from dataclasses import dataclass, field
from typing import ClassVar

from .._lifecycle import Finalizer, get_reaper
from .._log import logger, warn_once
from ..cache import CacheKey
from ..config import Settings, resolve_target
from ..errors import NotSupportedError, SmoltestError
from ..postgres import PostgresMachine, Seed
from ..transport.base import Engine
from ..wait.strategies import WaitStrategy
from .strategy import BootInfo, BootResult, boot_postgres


@dataclass(frozen=True)
class GoldenKey:
    """What :class:`GoldenRegistry` dedups on.

    The checkpoint key says which *checkpoints* a template can reuse; a golden is
    reusable only if the template also asks for the same runtime: the same
    pinned PostgreSQL host port (a second template pinned elsewhere would never
    get its port published), the same URL driver, machine name, settings
    (branching policy, exec timeout, cache directory, ...), wait strategy and
    the very same engine instance: a golden booted through one engine must never
    be handed to a template that injected another.
    """

    cache_key: str
    seed_key: str | None
    pinned_host_port: int | None
    driver: str | None
    name: str | None
    settings: Settings
    wait: tuple[str, str] | None
    engine: Engine = field(compare=True, hash=True, repr=False)


def _wait_identity(strategy: WaitStrategy | None) -> tuple[str, str] | None:
    """A hashable description of a wait strategy: its class and configuration."""
    if strategy is None:
        return None
    kind = type(strategy)
    return f"{kind.__module__}.{kind.__qualname__}", repr(sorted(vars(strategy).items()))


def template_key(template: PostgresMachine) -> str:
    """The cache key of the machine ``template`` would boot (seed excluded)."""
    settings, engine = template.settings, template.engine
    target = resolve_target(settings, engine)
    return CacheKey.compute(template.spec, settings, target, engine).key


def _close_quietly(close: Callable[[], None], what: str) -> None:
    try:
        close()
    except Exception:
        logger.exception("closing %s failed", what)


class PostgresGolden:
    """One booted PostgreSQL machine that spawns isolated children.

    Create it with :meth:`boot`; get children with :meth:`branch` (falls back to
    :meth:`fresh` when branching is disabled or unsupported) and tear everything
    down with :meth:`close`, children first.
    """

    def __init__(
        self,
        machine: PostgresMachine,
        result: BootResult,
        *,
        template: PostgresMachine,
        seed: Seed | None = None,
    ) -> None:
        self._machine = machine
        self._result = result
        self._template = template
        self._seed = seed
        self._children: list[PostgresMachine] = []
        self._lock = threading.RLock()
        self._closed = False
        settings, engine = machine.settings, machine.engine
        self._branch_disabled = settings.disable_branch
        self._branch_supported = not settings.disable_branch and engine.supports_branch(
            result.target
        )
        self._branch_count = 0
        self._fresh_count = 0

    @classmethod
    def boot(cls, template: PostgresMachine, *, seed: Seed | None = None) -> PostgresGolden:
        """Boot the golden from an unstarted ``template`` (its spec, settings, wait and name).

        ``seed`` defaults to the template's seed; it is applied once (or restored
        from the seeded checkpoint) and every child inherits the data.
        """
        if template.is_running:
            raise SmoltestError("PostgresGolden.boot needs an unstarted template machine")
        chosen = seed if seed is not None else template.seed
        result = boot_postgres(
            template.spec,
            template.settings,
            template.engine,
            seed=chosen,
            wait=template.wait_strategy,
            role="golden",
            name=template.name,
            host_port=template.pinned_host_port,
        )
        machine = PostgresMachine._from_boot(
            result,
            template.spec,
            template.settings,
            template.engine,
            driver=template.driver,
            seed=chosen,
        )
        return cls(machine, result, template=template, seed=chosen)

    # -- state -------------------------------------------------------------------------

    @property
    def machine(self) -> PostgresMachine:
        """The golden machine itself (shared; do not mutate from tests)."""
        return self._machine

    @property
    def seed(self) -> Seed | None:
        return self._seed

    @property
    def settings(self) -> Settings:
        return self._machine.settings

    @property
    def engine(self) -> Engine:
        return self._machine.engine

    @property
    def boot_info(self) -> BootInfo:
        """How the golden was booted."""
        return self._result.info

    @property
    def branch_supported(self) -> bool:
        """``True`` while :meth:`branch` really branches (flips off on the first refusal)."""
        return self._branch_supported

    @property
    def closed(self) -> bool:
        return self._closed

    @property
    def children(self) -> tuple[PostgresMachine, ...]:
        """Children handed out and not yet stopped."""
        with self._lock:
            self._prune()
            return tuple(self._children)

    @property
    def branch_count(self) -> int:
        """How many children were branches (the rest came from :meth:`fresh`)."""
        return self._branch_count

    @property
    def fresh_count(self) -> int:
        return self._fresh_count

    def _prune(self) -> None:
        self._children = [c for c in self._children if c.is_running]

    def _require_open(self) -> None:
        if self._closed:
            raise SmoltestError("the golden machine is closed")

    # -- children ----------------------------------------------------------------------

    def branch(self, name: str | None = None) -> PostgresMachine:
        """An isolated child: a branch when possible, else :meth:`fresh`."""
        with self._lock:
            self._require_open()
            self._prune()
            if self._branch_supported:
                try:
                    child = self._machine.branch(name)
                except NotSupportedError as exc:
                    self._disable_branch(f"the engine refused to branch: {exc}")
                else:
                    self._branch_count += 1
                    self._record_branch_ok(True)
                    self._children.append(child)
                    return child
            elif self._branch_disabled:
                logger.info("branching disabled by settings; booting a fresh machine")
            else:
                self._disable_branch("branching is not supported on this target")
            return self.fresh(name)

    def _disable_branch(self, reason: str) -> None:
        if self._branch_supported:
            self._record_branch_ok(False)
        self._branch_supported = False
        warn_once(
            f"golden.branch.{self._result.handle.id}",
            f"smoltest: {reason}; falling back to fresh machines (restore or cold boot) "
            "for every test",
            stacklevel=3,
        )

    def _record_branch_ok(self, ok: bool) -> None:
        backend, claim = self._result.backend, self._result.claim
        if backend is None or claim is None or not claim.populated:
            return
        try:
            backend.record_branch_ok(claim, ok)
        except SmoltestError as exc:
            logger.debug("could not record branch_ok: %s", exc)

    def fresh(self, name: str | None = None) -> PostgresMachine:
        """A fresh machine from the same template and seed (restore or cold boot)."""
        with self._lock:
            self._require_open()
            self._prune()
            template = self._template
            result = boot_postgres(
                template.spec,
                template.settings,
                template.engine,
                seed=self._seed,
                wait=template.wait_strategy,
                role="standalone",
                name=name,
            )
            child = PostgresMachine._from_boot(
                result,
                template.spec,
                template.settings,
                template.engine,
                driver=template.driver,
                seed=self._seed,
            )
            self._fresh_count += 1
            self._children.append(child)
            return child

    # -- teardown ----------------------------------------------------------------------

    def close(self) -> None:
        """Stop every live child, then the golden; idempotent."""
        with self._lock:
            if self._closed:
                return
            self._closed = True
            children, self._children = self._children, []
        for child in reversed(children):
            _close_quietly(child.stop, f"child {child.name}")
        self._machine.stop()

    def __enter__(self) -> PostgresGolden:
        return self

    def __exit__(self, *exc: object) -> None:
        self.close()

    def __repr__(self) -> str:
        state = "closed" if self._closed else "open"
        return f"PostgresGolden({self._machine!r}, {state}, branch={self._branch_supported})"


class GoldenRegistry:
    """Process-wide goldens, one per :class:`GoldenKey` (spec, seed and runtime options).

    :meth:`instance` is the shared registry; :meth:`close_all` runs at interpreter
    exit through the :class:`~smoltest._lifecycle.Reaper`.
    """

    _instance: ClassVar[GoldenRegistry | None] = None
    _instance_lock: ClassVar[threading.Lock] = threading.Lock()

    def __init__(self) -> None:
        self._lock = threading.Lock()
        self._goldens: dict[GoldenKey, PostgresGolden] = {}
        ref = weakref.ref(self)

        def close_all() -> None:
            registry = ref()
            if registry is not None:
                registry.close_all()

        self._token: Finalizer = get_reaper().register(self, close_all)

    @classmethod
    def instance(cls) -> GoldenRegistry:
        """The shared registry, created on first use."""
        with cls._instance_lock:
            if cls._instance is None:
                cls._instance = cls()
            return cls._instance

    @staticmethod
    def key_for(template: PostgresMachine, seed: Seed | None) -> GoldenKey:
        """The dedup key for ``template`` with ``seed``: checkpoint key plus runtime options."""
        return GoldenKey(
            cache_key=template_key(template),
            seed_key=seed.key if seed is not None else None,
            pinned_host_port=template.pinned_host_port,
            driver=template.driver,
            name=template.name,
            settings=template.settings,
            wait=_wait_identity(template.wait_strategy),
            engine=template.engine,
        )

    def get_or_boot(self, template: PostgresMachine, seed: Seed | None = None) -> PostgresGolden:
        """The golden for ``template`` and ``seed``, booting it on first request."""
        chosen = seed if seed is not None else template.seed
        key = self.key_for(template, chosen)
        with self._lock:
            golden = self._goldens.get(key)
            if golden is not None and not golden.closed:
                return golden
            golden = PostgresGolden.boot(template, seed=chosen)
            self._goldens[key] = golden
            return golden

    def get(self, template: PostgresMachine, seed: Seed | None = None) -> PostgresGolden | None:
        """The golden for ``template`` and ``seed`` if it is booted and open."""
        chosen = seed if seed is not None else template.seed
        with self._lock:
            golden = self._goldens.get(self.key_for(template, chosen))
        return golden if golden is not None and not golden.closed else None

    @property
    def goldens(self) -> tuple[PostgresGolden, ...]:
        with self._lock:
            return tuple(self._goldens.values())

    def close_all(self) -> None:
        """Close every golden (children first), most recent first; idempotent."""
        with self._lock:
            goldens = list(self._goldens.values())
            self._goldens.clear()
        for golden in reversed(goldens):
            _close_quietly(golden.close, repr(golden))

    def __len__(self) -> int:
        with self._lock:
            return len(self._goldens)


__all__ = ["GoldenKey", "GoldenRegistry", "PostgresGolden", "template_key"]
