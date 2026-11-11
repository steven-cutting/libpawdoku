"""``smoltest doctor``: which Smol target can boot here, and the state of the cache.

The report never prints a secret: ``SMOL_CLOUD_TOKEN`` is reported as present
or absent only. Nothing is created on disk.
"""

from __future__ import annotations

import argparse
import importlib.util
import json
import os
import platform
import sys
from collections.abc import Mapping
from importlib.metadata import PackageNotFoundError, version
from pathlib import Path
from typing import Any

from .._version import __version__
from ..cache import CheckpointCache, cloud_base_url
from ..config import Settings, resolve_target
from ..errors import TargetUnavailable
from ..transport import DEFAULT_ENGINE, ENGINE_ENV, get_engine
from ..transport.base import Engine
from . import EXIT_FAILURE, EXIT_OK, Context, human_bytes

KVM_DEVICE = Path("/dev/kvm")
TOKEN_ENV = "SMOL_CLOUD_TOKEN"
SDK_DISTRIBUTION = "smolmachines"
HINT = "set SMOL_CLOUD_TOKEN to use Smol Cloud, or run on Linux with /dev/kvm or on Apple Silicon"


def register(
    commands: argparse._SubParsersAction[argparse.ArgumentParser],
    common: argparse.ArgumentParser,
) -> None:
    """Add the ``doctor`` command."""
    parser = commands.add_parser(
        "doctor",
        parents=[common],
        help="check what can boot here and report the cache",
        description=(
            "Report versions, the local engine's availability, cloud credentials "
            "(present or absent, never the value), the resolved target and the "
            "checkpoint cache. Exit 0 when a target is usable, else 1."
        ),
    )
    parser.add_argument("--json", action="store_true", help="print the report as JSON")
    parser.set_defaults(func=run_doctor)


def distribution_version(name: str) -> str | None:
    """The installed version of distribution ``name``, or ``None``."""
    try:
        return version(name)
    except PackageNotFoundError:
        return None


def engine_name(engine: Engine) -> str:
    """``module.Class`` of the engine in use."""
    cls = type(engine)
    return f"{cls.__module__}.{cls.__qualname__}"


def cache_report(settings: Settings) -> dict[str, Any]:
    """Counts and sizes of the local cache, read without creating anything."""
    cache = CheckpointCache(settings.cache_dir, max_bytes=settings.cache_max_bytes)
    summary = cache.ls()
    variants = [variant for key in summary.keys for variant in key.variants]
    return {
        "dir": str(settings.cache_dir),
        "exists": settings.cache_dir.is_dir(),
        "bytes": summary.total_bytes,
        "max_bytes": settings.cache_max_bytes,
        "keys": len(summary.keys),
        "variants": len(variants),
        "populated": sum(1 for v in variants if v.populated),
        "live_claims": sum(1 for v in variants if v.claim_live),
        "stale_claims": sum(1 for v in variants if v.claimed_by is not None and not v.claim_live),
    }


def one_line(text: str | None) -> str | None:
    """Collapse the whitespace of an engine's reason, which may span lines."""
    return None if text is None else " ".join(text.split())


def _problems(
    resolved: str | None,
    error: Mapping[str, Any] | None,
    *,
    available: bool,
    code: str | None,
    reason: str | None,
    token_present: bool,
    websockets: bool,
) -> list[str]:
    problems: list[str] = []
    if resolved is None:
        why = error or {}
        problems.append(
            f"no Smol target available ({why.get('code') or 'UNKNOWN'}: "
            f"{one_line(why.get('reason')) or 'local engine unavailable'}); {HINT}"
        )
    elif resolved == "local" and not available:
        problems.append(f"local engine unavailable ({code}: {one_line(reason)})")
    elif resolved == "cloud":
        if not token_present:
            problems.append(f"{TOKEN_ENV} is not set")
        if not websockets:
            problems.append("websockets is not importable; install smoltest[cloud] for tunnels")
    return problems


def collect(settings: Settings, env: Mapping[str, str] | None = None) -> dict[str, Any]:
    """Gather the report as a JSON-ready mapping; prints and creates nothing."""
    env = os.environ if env is None else env
    engine = get_engine(settings)
    available, code, reason = engine.local_availability()
    token_present = bool(env.get(TOKEN_ENV))
    websockets = importlib.util.find_spec("websockets") is not None
    error: dict[str, Any] | None = None
    resolved: str | None
    try:
        resolved = resolve_target(settings, engine, env)
    except TargetUnavailable as exc:
        resolved = None
        error = {"code": exc.code, "reason": exc.reason}
    problems = _problems(
        resolved,
        error,
        available=available,
        code=code,
        reason=reason,
        token_present=token_present,
        websockets=websockets,
    )
    libc_name, libc_version = platform.libc_ver()
    return {
        "ok": resolved is not None and not problems,
        "smoltest": {"version": __version__},
        "smolmachines": {
            "version": distribution_version(SDK_DISTRIBUTION),
            "sdk_version": engine.sdk_version(),
            "engine": engine_name(engine),
            "engine_path": settings.engine_path or env.get(ENGINE_ENV) or DEFAULT_ENGINE,
        },
        "platform": {
            "system": platform.system(),
            "release": platform.release(),
            "machine": platform.machine(),
            "python": platform.python_version(),
            "executable": sys.executable,
            "libc": f"{libc_name} {libc_version}".strip() or None,
        },
        "local": {
            "available": available,
            "code": code,
            "reason": reason,
            "kvm_device": KVM_DEVICE.exists(),
        },
        "cloud": {
            "token_present": token_present,
            "url": cloud_base_url(env),
            "websockets": websockets,
        },
        "target": {"requested": settings.target, "resolved": resolved, "error": error},
        "cache": cache_report(settings),
        "problems": problems,
    }


def render_text(report: Mapping[str, Any]) -> str:
    """The human-readable form of :func:`collect`'s report."""
    sdk, pf = report["smolmachines"], report["platform"]
    local, cloud, target, cache = (
        report["local"],
        report["cloud"],
        report["target"],
        report["cache"],
    )
    local_text = (
        "available"
        if local["available"]
        else f"unavailable ({local['code']}: {one_line(local['reason'])})"
    )
    libc = f", {pf['libc']}" if pf["libc"] else ""
    lines = [
        f"smoltest {report['smoltest']['version']} (Python {pf['python']})",
        f"smolmachines {sdk['version'] or 'not installed'}, engine {sdk['engine']} "
        f"(sdk {sdk['sdk_version']})",
        f"platform {pf['system']} {pf['release']} {pf['machine']}{libc}",
        f"local engine: {local_text}",
        f"/dev/kvm: {'present' if local['kvm_device'] else 'absent'}",
        f"cloud: {TOKEN_ENV} {'present' if cloud['token_present'] else 'absent'}, "
        f"url {cloud['url']}, websockets {'installed' if cloud['websockets'] else 'missing'}",
        f"target: {target['requested']} -> {target['resolved'] or 'none'}",
        f"cache: {cache['dir']} ({'exists' if cache['exists'] else 'missing'}), "
        f"{human_bytes(cache['bytes'])} of {human_bytes(cache['max_bytes'])}, "
        f"{cache['keys']} keys, {cache['variants']} variants ({cache['populated']} populated), "
        f"{cache['live_claims']} live claims, {cache['stale_claims']} stale claims",
    ]
    if report["ok"]:
        lines.append(f"ok: machines can boot on {target['resolved']}")
    else:
        lines.append("problems:")
        lines.extend(f"  - {problem}" for problem in report["problems"])
    return "\n".join(lines)


def run_doctor(args: argparse.Namespace, ctx: Context) -> int:
    """Print the report; exit 0 when a target is usable, else 1."""
    report = collect(ctx.settings)
    if args.json:
        print(json.dumps(report, indent=2, sort_keys=True))
    else:
        print(render_text(report))
    return EXIT_OK if report["ok"] else EXIT_FAILURE


__all__ = [
    "HINT",
    "KVM_DEVICE",
    "SDK_DISTRIBUTION",
    "TOKEN_ENV",
    "cache_report",
    "collect",
    "distribution_version",
    "engine_name",
    "one_line",
    "register",
    "render_text",
    "run_doctor",
]
