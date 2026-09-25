#!/bin/sh
set -eu

project_root=$(CDPATH='' cd -- "$(dirname -- "$0")/.." && pwd -P)
cd "$project_root"

git rev-parse --is-inside-work-tree >/dev/null

# The toolchain rust-toolchain.toml pins, then proof that `cargo` is rustup's
# proxy and not a pixi, Homebrew or distribution cargo that ignores the pin.
just install-toolchain
just check-toolchain

# cargo-binstall is a per-machine bootstrap pin, like rustup itself: it cannot
# install itself from tools.txt, so it is compiled once when absent.
command -v cargo-binstall >/dev/null || cargo install cargo-binstall@1.23.0 --locked

# The pinned binaries tools.txt lists, into the gitignored .tools/bin.
just install-tools

# The Allium checker for docs/specs/, pinned and checksummed in
# the biscuit-games-tooling package that pyproject.toml pins, landing in the
# gitignored .tools/bin. `just check-specs` and `just analyse-specs` run it, and
# both the hook gate and `just check` run those, so a worktree without it cannot
# reach a green gate.
just install-allium

test -f Cargo.lock || cargo generate-lockfile
uv lock
just sync

# Formatting is normalised once here rather than leaving the first `just check`
# to fail on it.
just format

# Hooks are installed only from the primary checkout. Every worktree of this
# repository shares one .git/hooks directory, and `prek install` writes a shim
# naming an absolute path into whichever worktree ran it — so a hook installed
# from a secondary worktree runs that worktree's virtual environment for commits
# made anywhere, and keeps doing so after the worktree is deleted, at which point
# every commit fails on a `prek` that is not on PATH. The comparison is the test
# git itself uses: in a secondary worktree the common directory and the git
# directory differ.
if [ "$(git rev-parse --git-common-dir)" = "$(git rev-parse --git-dir)" ]; then
    just install-hooks
else
    printf '%s\n' 'Secondary worktree: skipping install-hooks.' >&2
    printf '%s\n' 'Run just install-hooks once from the primary checkout.' >&2
fi

printf '\n%s\n' 'Ready. Next: just check.'
printf '%s\n' 'Nothing has been staged, committed, tagged, or pushed.'
