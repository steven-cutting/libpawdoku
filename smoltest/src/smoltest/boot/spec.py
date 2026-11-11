"""What a PostgreSQL machine should look like, before any engine is involved."""

from __future__ import annotations

from collections.abc import Mapping, Sequence
from dataclasses import dataclass, field

from ..config import Settings
from ..errors import InvalidConfig
from ..transport.base import MachineSpec, PortMapping, Target

ENTRYPOINT: tuple[str, ...] = ("docker-entrypoint.sh", "postgres")
DEFAULT_PORT = 5432
"""The port the image's server listens on unless the argv says otherwise."""
FAST_ARGS: tuple[str, ...] = (
    "-c",
    "fsync=off",
    "-c",
    "synchronous_commit=off",
    "-c",
    "full_page_writes=off",
)
LOG_DIRECTORY = "/tmp"
"""Where ``capture_logs`` sends the server log: world-writable in both the Debian and
the alpine official images, unlike ``/var/log/postgresql``, which alpine lacks."""
LOG_FILENAME = "postgresql.log"
LOG_PATH = f"{LOG_DIRECTORY}/{LOG_FILENAME}"
CAPTURE_LOGS_ARGS: tuple[str, ...] = (
    "-c",
    "logging_collector=on",
    "-c",
    f"log_directory={LOG_DIRECTORY}",
    "-c",
    f"log_filename={LOG_FILENAME}",
    "-c",
    "log_rotation_age=0",
    "-c",
    "log_rotation_size=0",
)
INITDB_NO_SYNC = "--no-sync"
CREDENTIAL_VARS: Mapping[str, str] = {
    "POSTGRES_USER": "username",
    "POSTGRES_PASSWORD": "password",
    "POSTGRES_DB": "dbname",
}
"""Guest variables that are really spec fields; ``env`` entries for them are folded in."""


@dataclass(frozen=True)
class PostgresSpec:
    """Immutable description of a PostgreSQL machine.

    ``None`` on ``fast``, ``cpus``, ``memory_mb`` and ``storage_gb`` means "take
    the value from :class:`~smoltest.config.Settings`". ``command`` overrides the
    default argv entirely. ``env`` is layered over the credential variables, except
    that ``POSTGRES_USER`` / ``POSTGRES_PASSWORD`` / ``POSTGRES_DB`` given in ``env``
    are folded into ``username`` / ``password`` / ``dbname``, so the connection URL,
    ``psql``, seeds and readiness probes always agree with the server that boots.
    """

    image: str | None = None
    username: str = "test"
    password: str = "test"
    dbname: str = "test"
    guest_port: int = 5432
    env: Mapping[str, str] = field(default_factory=dict)
    command: tuple[str, ...] | None = None
    fast: bool | None = None
    capture_logs: bool = False
    extra_ports: tuple[PortMapping, ...] = ()
    cpus: int | None = None
    memory_mb: int | None = None
    storage_gb: int | None = None
    network: bool | None = None

    def __post_init__(self) -> None:
        env = dict(self.env)
        for var, field_name in CREDENTIAL_VARS.items():
            if var in env:
                object.__setattr__(self, field_name, env.pop(var))
        object.__setattr__(self, "env", env)
        object.__setattr__(self, "command", None if self.command is None else tuple(self.command))
        object.__setattr__(self, "extra_ports", tuple(self.extra_ports))
        if not 0 < self.guest_port < 65536:
            raise InvalidConfig(f"guest_port out of range: {self.guest_port}")
        if any(p.guest == self.guest_port for p in self.extra_ports):
            raise InvalidConfig(
                f"extra_ports must not repeat the PostgreSQL port {self.guest_port}"
            )

    def resolved_image(self, settings: Settings) -> str:
        """The image to boot: the spec's or the settings' default."""
        return self.image or settings.postgres_image

    def resolved_fast(self, settings: Settings) -> bool:
        """Whether fast mode applies, honouring ``settings.fast_mode`` when unset."""
        return settings.fast_mode if self.fast is None else self.fast

    def effective_env(self, settings: Settings) -> dict[str, str]:
        """Credential variables, ``POSTGRES_INITDB_ARGS`` for fast mode, then ``env``."""
        base = {
            "POSTGRES_USER": self.username,
            "POSTGRES_PASSWORD": self.password,
            "POSTGRES_DB": self.dbname,
        }
        merged = {**base, **self.env}
        if self.resolved_fast(settings):
            existing = merged.get("POSTGRES_INITDB_ARGS", "")
            if INITDB_NO_SYNC not in existing.split():
                merged["POSTGRES_INITDB_ARGS"] = f"{existing} {INITDB_NO_SYNC}".strip()
        return merged

    def effective_argv(self, settings: Settings) -> tuple[str, ...]:
        """``command`` when given, else :func:`default_argv`."""
        return self.command if self.command is not None else default_argv(self, settings)


def default_argv(spec: PostgresSpec, settings: Settings | None = None) -> tuple[str, ...]:
    """The explicit workload argv smoltest always passes.

    Starts with ``docker-entrypoint.sh postgres`` so it works whether the engine's
    ``command`` replaces ENTRYPOINT+CMD or CMD only, moves the server to
    ``spec.guest_port`` when that is not :data:`DEFAULT_PORT` (the image's
    ``initdb`` phase keeps its temporary server on the default port), then adds
    the fast-mode and log-capture ``-c`` switches the spec asks for.
    """
    settings = settings or Settings()
    argv: list[str] = list(ENTRYPOINT)
    if spec.guest_port != DEFAULT_PORT:
        argv.extend(("-c", f"port={spec.guest_port}"))
    if spec.resolved_fast(settings):
        argv.extend(FAST_ARGS)
    if spec.capture_logs:
        argv.extend(CAPTURE_LOGS_ARGS)
    return tuple(argv)


def build_machine_spec(
    spec: PostgresSpec,
    settings: Settings,
    target: Target,
    host_port: int | None,
    name: str | None = None,
    *,
    extra_ports: Sequence[PortMapping] | None = None,
    branchable: bool = True,
) -> MachineSpec:
    """Translate a :class:`PostgresSpec` plus settings into an engine :class:`MachineSpec`.

    ``host_port`` pins the host side of the PostgreSQL port (``None`` lets the
    engine choose, which is what the cloud target wants). Cloud machines get the
    ``auto_stop``/``ttl`` safety net from settings.
    """
    ports = [PortMapping(host_port, spec.guest_port)]
    ports.extend(spec.extra_ports if extra_ports is None else extra_ports)
    cloud = target == "cloud"
    return MachineSpec(
        image=spec.resolved_image(settings),
        argv=spec.effective_argv(settings),
        env=spec.effective_env(settings),
        ports=tuple(ports),
        cpus=spec.cpus if spec.cpus is not None else settings.cpus,
        memory_mb=spec.memory_mb if spec.memory_mb is not None else settings.memory_mb,
        storage_gb=spec.storage_gb if spec.storage_gb is not None else settings.storage_gb,
        network=spec.network if spec.network is not None else settings.network,
        branchable=branchable,
        name=name,
        ready_timeout_s=settings.ready_timeout_s,
        wait_for_ports=True,
        auto_stop_seconds=settings.cloud_auto_stop_seconds if cloud else None,
        ttl_seconds=settings.cloud_ttl_seconds if cloud else None,
        exec_timeout_s=settings.exec_timeout_s,
    )


__all__ = [
    "CAPTURE_LOGS_ARGS",
    "CREDENTIAL_VARS",
    "DEFAULT_PORT",
    "ENTRYPOINT",
    "FAST_ARGS",
    "INITDB_NO_SYNC",
    "LOG_PATH",
    "PostgresSpec",
    "build_machine_spec",
    "default_argv",
]
