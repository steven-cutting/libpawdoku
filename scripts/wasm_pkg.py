"""Finish the npm package wasm-pack wrote into crates/pawdoku-wasm/pkg/.

`just wasm-build` runs this after `wasm-pack build`, with the pixi environment's
Python and `-I`. Standard library only. It writes inside pkg/ alone, which Git
ignores, so the gate's worktree check never sees it (decision 0020).

Every module in docs/specs/ is copied into pkg/specs/, whatever the directory
holds on the day, after any stale copy is removed; the crate's own CHANGELOG.md
is copied into pkg/; and package.json gains `specs` and `CHANGELOG.md` in
`files`, and an `exports` map that gives the module, the `.wasm` and every
specification module a subpath.

wasm-pack copies README.md and LICENSE itself; npm packs a LICENSE whatever
`files` says.
"""

import json
import shutil
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
CRATE = ROOT / "crates" / "pawdoku-wasm"
PKG = CRATE / "pkg"
SPECS = ROOT / "docs" / "specs"

# The module and its types first: a resolver takes the first condition that
# matches, and TypeScript reads `types` only when it precedes `default`.
EXPORTS = {
    ".": {"types": "./pawdoku_wasm.d.ts", "default": "./pawdoku_wasm.js"},
    "./pawdoku_wasm_bg.wasm": "./pawdoku_wasm_bg.wasm",
    "./specs/*": "./specs/*",
}


def copy_specs() -> list[str]:
    """Replace pkg/specs/ with every module docs/specs/ holds, and name them."""
    modules = sorted(SPECS.glob("*.allium"))
    if not modules:
        sys.exit(f"wasm_pkg: no .allium module in {SPECS}")
    target = PKG / "specs"
    shutil.rmtree(target, ignore_errors=True)
    target.mkdir()
    for module in modules:
        shutil.copyfile(module, target / module.name)
    return [module.name for module in modules]


def amend_manifest() -> dict:
    """Add the files and the exports map to pkg/package.json, and return it."""
    path = PKG / "package.json"
    manifest = json.loads(path.read_text(encoding="utf-8"))
    files = manifest.setdefault("files", [])
    for entry in ("specs", "CHANGELOG.md"):
        if entry not in files:
            files.append(entry)
    manifest["exports"] = EXPORTS
    path.write_text(json.dumps(manifest, indent=2) + "\n", encoding="utf-8")
    return manifest


def main() -> None:
    if not (PKG / "package.json").is_file():
        sys.exit(f"wasm_pkg: {PKG / 'package.json'} is missing; run wasm-pack build first")
    modules = copy_specs()
    shutil.copyfile(CRATE / "CHANGELOG.md", PKG / "CHANGELOG.md")
    manifest = amend_manifest()
    print(
        f"wasm_pkg: {manifest['name']} {manifest['version']}: "
        f"{len(modules)} modules in specs/ ({', '.join(modules)}), CHANGELOG.md, exports {sorted(EXPORTS)}"
    )


if __name__ == "__main__":
    main()
