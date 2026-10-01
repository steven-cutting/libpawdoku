---
title: "Maintain dependencies"
kind: "how-to"
audience: [maintainer, agent]
canonical_for: [dependency_maintenance]
requires: []
---

# Maintain dependencies

The manifests write full `x.y.z` caret ranges, and `Cargo.lock` is the pin: committed,
marked `linguist-generated`, and passed `--locked` by every gate, so no recipe that checks
can rewrite it. This is [decision 0007](../decisions/0007-dependency-policy.md), a stated
deviation from the games' exact-pin rule, because a library that writes `=1.2.3` poisons
every downstream resolution: two crates pinning different patch releases of one
dependency cannot be built together. The tools are the other way round: `pyproject.toml`
pins each one exactly, and `pixi.lock`, committed beside it, records the resolution.
Nothing updates either lockfile for you. There is no Dependabot or Renovate here until
ticket S01 decides how updates should arrive.

## Check that the lockfiles still match

```console
just lock-check
```

This runs `cargo update --workspace --locked --offline` and
`pixi lock --check --offline --dry-run`, neither of which writes. It is gate 2 of
`just check`, so a manifest edited without relocking fails the gate rather than
drifting.

## Update deliberately

```console
just lock            # relock after a manifest edit
just lock-upgrade    # move every crate to the newest version its range admits
```

`just lock` relocks the workspace against what the manifests state, then runs
`pixi lock`. It is not a no-op: a dependency whose range was edited, or that is new,
resolves to the newest version its range admits, not to the version the range names.
`just lock-upgrade` moves every crate, transitive ones included, to the newest version
its range admits, which under caret ranges changes plenty, and runs `pixi update`. Every
tool pin is exact, so that moves transitive packages and one direct one: Python, pinned
`3.14.*`, whose patch release can move. Read both lockfile diffs before accepting them.

## Upgrading a crate

1. The version lives in `[workspace.dependencies]` in the root `Cargo.toml` and nowhere
   else; a member crate names the dependency with `workspace = true`. Edit the range there.
2. Run `just lock`, then read the `Cargo.lock` diff before accepting it.
3. Run `just sync`, so the new sources are fetched while the network is there.
4. Run `just deny`: licences, bans and sources, answered offline.
5. Run `just check`. A clippy or rustdoc upgrade usually surfaces new findings; fix them
   rather than pinning back, unless the finding is wrong for this project.
6. Run `just audit` for security advisories. It needs the RustSec database over the
   network, which is why it sits outside `just check`; the scheduled `audit.yml` workflow
   runs it too.

A new crate, rather than a new version of one, is a decision-0007 note in the pull
request: what it is for, and why the engine cannot do without it.

## The tools in `pyproject.toml`

The `[tool.pixi.dependencies]` table pins every tool, Python and prek exactly, and
`pixi.lock` records the resolution for each platform (decision 0011). To move one, edit
its line, relock that one package, read the lockfile diff, and commit `pixi.lock` with the
manifest:

```console
pixi update <name>
```

Two lists hold what conda-forge lacks, one `name@version` per line, and
`just install-tools` reads both. It skips a version already installed.

- `tools.txt` (cargo-hack today) is binary-only: its pass never compiles
  (`--disable-strategies compile`), so a tool with no prebuilt binary fails loudly
  rather than building at bootstrap.
- `tools-source.txt` (rustqual today) is for a tool with no prebuilt binary at all. Its
  pass may compile, from crates.io source with `--locked`, or take a signed
  cargo-quickinstall build, but never an unsigned upstream release asset
  (`--disable-strategies crate-meta-data`). A line goes here only on a
  decision record that accepts the source build, as
  [decision 0013](../decisions/0013-metrics-gate.md) does for rustqual.

Bump either by editing the line and running `just install-tools`. Move a
`tools-source.txt` pin one release at a time and run `just check` on each, because the
tool is a gate: rustqual's pin is held until the solver has landed under it, and the exit
that decision 0013 states counts the bumps that needed a suppression or a threshold change.

Outside both files sit the bootstrap pins: pixi's own version, a floor in `requires-pixi`
in `pyproject.toml` and an exact `pixi-version` on the `setup-pixi` step in
`.github/actions/setup/action.yml`. They move together.

## The toolchain

`channel` in `rust-toolchain.toml` and `rust-version` in the root `Cargo.toml` move
together. After a bump run `just install-toolchain`, then `just clippy`, because the lint
set moves with the release and `-D warnings` turns every new lint into a failure.

## Hook pins

Every remote hook in `.pre-commit-config.yaml` and `.pre-commit-fix.yaml` is pinned to a
full commit SHA, with the release tag as a comment. lychee also carries `LYCHEE_VERSION`
at the head of both its argument lists, which must match the comment; the comment beside
the hook says why. The `builtin` hygiene hooks carry no pin.

To move a pin, resolve the release's commit and replace both the SHA and the comment,
in every file that names the hook. Tags are often annotated, so ask for the commit: the
SHA a tag reference answers with names the tag object, which neither a `rev:` nor a
`uses:` line accepts. For typos, the worked example:

```console
gh api repos/crate-ci/typos/commits/<tag> --jq .sha
```

The same line, as `gh api repos/<owner>/<repo>/commits/<tag> --jq .sha`, resolves any hook
or action.

## Moving the tooling package

The documentation, agent and specification gates, the allium installer and the runner
behind `just check` are console scripts of `biscuit-games-tooling`, which
`[tool.pixi.pypi-dependencies]` in `pyproject.toml` pins to a release tag of
`steven-cutting/biscuit_games_tooling`. `pixi.lock` records the commit behind the tag, and
`just lock-check` fails when the tag moves without a relock.

1. Read the package's `CHANGELOG.md` in that repository for the release you are taking, and
   the level its README gives the change. A Major release can fail a tree that passed.
2. Edit the `tag` in the `biscuit-games-tooling` line of `pyproject.toml`, then relock that
   one package and read the lockfile diff before committing `pixi.lock`:

   ```console
   pixi update biscuit-games-tooling
   ```

3. Run `just sync`, because the lock installs nothing, and `just install-allium`, which
   replaces the binary if the release moved its pin.
4. Run `just check` before committing, whatever the release level. The package supplies
   the gate runner and the validators, and the hooks that read `pyproject.toml`, among
   them `check-specs` and `analyse-specs`, re-run the contracts and the specifications
   but not `lock-check` or the whole gate.

A tag the package has released is never moved, so relocking without editing the tag
changes nothing.

## Moving the Allium binary

`allium` is a checksummed binary, not a package, so no lockfile accounts for it and
`just lock-check` cannot speak for it. The `biscuit-games-tooling` package holds the
version and the SHA-256 of each supported artefact; see
[decision 0005](../decisions/0005-project-managed-allium-cli.md). Moving the allium pin is
a release of that package, and this repository takes it by moving the package pin, as the
section above describes. The package's README carries the recomputation of the checksums,
because upstream publishes none for these files.

After taking a release that moves it, reinstall and confirm:

```console
just install-allium
just check-specs
```

Reinstalling is always safe to retry. The download lands beside the installed copy under a
temporary name and is asked for both its checksum and its version there, so a failed
download, a mismatched checksum or a binary that will not run leaves the working
installation exactly where it was. `just install-allium` also replaces a binary that no
longer runs, so an installation damaged by other means repairs itself rather than needing
`.tools/` cleared by hand.

A version change can move what the checker reports, in both directions. After moving the
pin, run `just check-specs` and `just analyse-specs`: a new version can report something
the modules were clean of, and it can also stop needing a waiver they carry. The modules
carry none at present, but the directive leans on behaviour upstream documents nowhere and
was verified against 3.6.1 only, so any waiver added later must be re-verified on the
commit that moves the pin, dropped where the new version no longer needs it, and its count
and shape updated in [Work with the specifications](work-with-the-specs.md) in that same
commit. Moving the package pin in `pyproject.toml` is itself a trigger for both
specification hooks, so the gate re-reads the modules against the new version on the
commit that moves the pin — but only after `just install-allium` has actually installed it.

## Actions in the workflows

`ci.yml`, `audit.yml` and the composite `setup` action pin every remote action in a
`uses:` line to a full commit SHA with its version comment, never to a tag. The one
exception is the local `uses: ./.github/actions/setup`, a path in this repository that
moves with the commit that calls it. The same `gh api` line resolves a tag to
the commit to paste. The toolchain the jobs build with is not pinned here: the setup
action runs `just install-toolchain`, so CI reads `rust-toolchain.toml` like everyone else.

`actionlint` runs inside `just lint`, so a malformed workflow fails locally.

## Related pages

- [Configuration](../reference/configuration.md)
- [Quality gates](../reference/quality-gates.md)
- [Maintenance](../operations/maintenance.md)
- [Decision 0007: dependency policy](../decisions/0007-dependency-policy.md)
