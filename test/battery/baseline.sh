#!/usr/bin/env bash
# Resolve one row's baseline directory, cutting it on a cache miss.
#
#   baseline.sh key   ->  .../target/battery/base/<sha>/keybase
#
# Nothing generated is committed.  The baselines used to be 276 files and
# 53 MiB of `*base/` in git; they are now cut from the binary
# `test/battery/BASE` pins, with the CURRENT tree's sweep scripts and
# corpora -- only the binary comes from BASE, because the scripts and
# corpora are versioned with the tree and a row must always run the corpus
# it was just edited to run.
#
# The cache key is the sha, so a stale cache is impossible: bump BASE and
# every row misses and re-cuts against the new binary; revert it and the old
# cut is still there.  `rm -rf target/battery` is always safe.
#
# `$<ROW>_BASELINE` still overrides, and `<row>verify.sh` still honours it --
# that is how a hand-cut baseline is compared against.
set -euo pipefail

HERE=$(cd "$(dirname "$0")" && pwd)
REPO=${REPO:-$(cd "$HERE/../.." && pwd)}
ROW=${1:?usage: baseline.sh <row>}

SHA=$(grep -m1 -oE '\b[0-9a-f]{7,40}\b' "$HERE/BASE" || true)
if [[ -z $SHA ]]; then
  echo "baseline: no commit in $HERE/BASE" >&2
  exit 1
fi

DIR=$REPO/target/battery/base/$SHA
CACHE=$DIR/${ROW}base
# The stamp, not the directory, is the cache hit: a cut interrupted halfway
# leaves a directory full of half-written artifacts, and a row that trusted
# the directory would compare against them forever.
STAMP=$CACHE/.cut

mkdir -p "$DIR"
exec 9>"$DIR/.$ROW.lock"
flock 9

if [[ -f $STAMP ]]; then
  echo "$CACHE"
  exit 0
fi

BIN=$("$HERE/refbin.sh")
rm -rf "$CACHE"
mkdir -p "$CACHE"
echo "baseline: cutting $ROW at $SHA" >&2

if [[ $ROW == probe ]]; then
  # The probe has no verify.sh -- battery.sh runs it inline.  It exits
  # nonzero when a case's nvim does, which is data, not a failure of the cut.
  python3 "$HERE/startprobe.py" "$BIN" "$CACHE/base.txt" >&2 || true
else
  "$HERE/${ROW}verify.sh" --cut "$BIN" "$CACHE" >&2
fi

# `compgen -G` would be the obvious test and is not available: bash's
# programmable-completion builtins are compiled out of a non-interactive
# shell on this platform.  A nullglob array is portable.
shopt -s nullglob
cut_artifacts=("$CACHE"/base*)
if [[ ${#cut_artifacts[@]} -eq 0 ]]; then
  echo "baseline: cutting $ROW produced no artifacts in $CACHE" >&2
  exit 1
fi

echo "$SHA" >"$STAMP"
echo "$CACHE"
