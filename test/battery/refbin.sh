#!/usr/bin/env bash
# Build the pinned reference binary and print its path.
#
# The battery is a base-vs-head differential: every row's baseline is CUT,
# not committed.  `test/battery/BASE` pins the commit the baselines speak
# for; this script materialises that commit's binary once per checkout and
# every row reuses it.
#
#   refbin.sh            # prints .../target/battery/ref/<sha>/target/debug/nvim
#
# Three things about the layout are load-bearing:
#
#   * The reference source is a DETACHED WORKTREE under target/, not a
#     `git checkout` -- the tree must not move while a row is running, and
#     `target/` is already ignored, so nothing here can reach a commit.
#   * It builds into its OWN CARGO_TARGET_DIR.  Sharing target/ with the
#     main tree would make the two builds evict each other's artifacts on
#     every alternation, and a row that ran `just build` while the
#     reference build held the lock would simply block.
#   * That directory is named `target/`, so the binary's path ends in
#     `/target/debug/nvim`.  Fourteen sweeps mask the binary out of their
#     artifacts with `%S*/target/debug/nvim` / `[^ ]*/target/debug/nvim`
#     (cmdsweep, ex-run, foldsweep, ...); a reference binary anywhere else
#     would survive the mask and make every one of those rows DIFFER on
#     nothing but its own path.
#
# The build is the whole added cost of the cache: ~4 min cold, zero warm.
set -euo pipefail

HERE=$(cd "$(dirname "$0")" && pwd)
REPO=${REPO:-$(cd "$HERE/../.." && pwd)}

# BASE is one line -- the sha -- but read the first hex word rather than the
# first line, so a future BASE can carry a comment above the pin.
SHA=$(grep -m1 -oE '\b[0-9a-f]{7,40}\b' "$HERE/BASE" || true)
if [[ -z $SHA ]]; then
  echo "refbin: no commit in $HERE/BASE" >&2
  exit 1
fi

ROOT=$REPO/target/battery/ref/$SHA
SRC=$ROOT/src
BIN=$ROOT/target/debug/nvim

mkdir -p "$ROOT"
# Rows run serially, but `test/battery/keyverify.sh` alone in one terminal
# and `just battery` in another must not both drive cargo through the same
# CARGO_TARGET_DIR.
exec 9>"$ROOT/.lock"
flock 9

if [[ -x $BIN ]]; then
  echo "$BIN"
  exit 0
fi

if [[ ! -f $SRC/Cargo.toml ]]; then
  rm -rf "$SRC"
  # A worktree left registered by an interrupted run (or by `rm -rf target`)
  # would make `worktree add` refuse the path.
  git -C "$REPO" worktree prune
  echo "refbin: checking out $SHA" >&2
  git -C "$REPO" worktree add --detach "$SRC" "$SHA" >&2
fi

echo "refbin: building $SHA (once per checkout)" >&2
(cd "$SRC" && CARGO_TARGET_DIR=$ROOT/target cargo build) >&2

echo "$BIN"
