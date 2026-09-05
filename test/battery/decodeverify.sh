#!/usr/bin/env bash
# Build the working tree and diff its decode corpora against the stored
# baseline.  Run from anywhere; everything is absolute.
#
#   decodeverify.sh [label]       # default label: cur
#
# This is `1785820781-decodediff.sh` turned into a BATTERY ROW.  That
# script is a *paired* differential: it takes two nvim binaries and
# diffs them against each other, which is why it was never in the
# battery -- the battery's contract is "one binary, one stored
# baseline".  The pairing turned out not to be fundamental.  Measured at
# 068df58956: all five corpora are byte-identical across three
# consecutive runs of the same binary AND across a change of working
# directory (they were run from the repo root and from /tmp).  Nothing
# in any artifact carries a path, a pid, a version string, a timestamp
# or a raw handle -- the rpc corpus already scrubs channel ids to `N`
# and stream pointers to `PTR`, which is exactly what a baseline needs
# and is why it was worth checking rather than assuming.
#
# So the five sections keep their driver scripts and their corpora
# verbatim; this file only replaces "the other binary" with a stored
# answer.  Use `decodediff.sh` when you have two binaries (a rewrite
# against its pre-rewrite build); use this one from the battery.
#
# WHAT IT WATCHES, and why it matters to the typval/value layer:
#
#   json / msgpack   eval/decode.rs -- every parser error arm, every
#                    escape and surrogate path, every truncated document
#                    and every msgpack token width.  evalsweep drives
#                    json and msgpack only over ENCODER-GENERATED values,
#                    so all of that is invisible to it.
#   luajson          crates/nvim/src/cjson/ (incl. `fpconv.rs`, a
#                    hand-rolled grisu whose exact output is a byte
#                    contract no test asserts).
#   lumpack          crates/nvim/src/mpack/ -- every token width, the ext
#                    types, the Packer/Unpacker objects.
#   rpc              THE FULL Object TYPE MATRIX, in both directions,
#                    over a real msgpack-RPC job channel -- which makes
#                    this the only stored-baseline oracle in the tree
#                    that round-trips `api/private/helpers/value.rs`'s
#                    copy/free/convert surface over every Object kind.
#                    Plus forty malformed framings fed as raw bytes to a
#                    fresh `--embed` child each, and `nvim__unpack`.
#
# Baseline: decodebase/base.{json,msgpack,luajson,lumpack,rpc},
# produced at 068df58956.  Line counts 269 / 122 / 1622 / 651 / 164.
# Regenerate only when a behaviour change is *intended* and reviewed:
#
#   DECODE_REGEN=1 decodeverify.sh
#
# ... and `just build` first -- a mutation harness leaves the binary
# built from its last mutant, and a baseline taken from that compares
# mutant against mutant forever after.
#
# TAKES ~40 s, almost all of it the rpc corpus's forty child processes.
# That section is also the only one here with any plausible flake
# surface (it reaps children and reads their logs); if it ever DIFFERS
# on child bookkeeping alone, re-run before believing it.
set -uo pipefail

HERE=$(cd "$(dirname "$0")" && pwd)
BASELINE=${DECODE_BASELINE:-$HERE/decodebase}
OUT=${SWEEP_OUT:-/tmp/decodesweep-out}
REPO=${REPO:-$(cd "$HERE/../.." && pwd)}
LABEL=${1:-cur}
REGEN=${DECODE_REGEN:-}
LOG=$OUT/build-$LABEL.log
# Every other sweep bounds its children; this one did not, so a mutant that
# wedged an rpc child wedged the whole battery row. The rpc corpus is ~30 s
# of forty children on a good day, so 180 s is generous.
LIMIT=${DECODE_TIMEOUT:-180}

mkdir -p "$OUT"
cd "$REPO" || exit 1
if ! just build >"$LOG" 2>&1; then
  echo "BUILD FAILED -- see $LOG" >&2
  grep -E '^(error|warning)' "$LOG" | head -60 >&2
  exit 1
fi
NVIM=$REPO/target/debug/nvim

# The corpora are cwd-independent (verified), but run them from a fixed
# short directory anyway: that is the rule every other sweep follows and
# it costs nothing.
WORK=/tmp/decodesweep-work
rm -rf "$WORK"; mkdir -p "$WORK"
cd "$WORK" || exit 1

fail=0
for corpus in json msgpack luajson lumpack rpc; do
  out=$OUT/$LABEL.$corpus
  rm -f "$out"
  case $corpus in
    json|msgpack)
      timeout -k 5 "$LIMIT" \
        "$NVIM" --headless -u NONE -S "$HERE/decodecorpus-$corpus.vim" \
        -c qa >"$out" 2>&1 ;;
    *)
      timeout -k 5 "$LIMIT" \
        "$NVIM" --headless -u NONE -i NONE \
        -l "$HERE/decodecorpus-$corpus.lua" >"$out" 2>&1 ;;
  esac
  # 124 is `timeout`'s own status; say so rather than letting it read as a
  # content diff against a truncated artifact.
  if [ $? -eq 124 ]; then
    echo "$corpus: TIMED OUT after ${LIMIT}s"
    fail=1
    continue
  fi
  if [ -n "$REGEN" ]; then
    mkdir -p "$BASELINE"
    cp "$out" "$BASELINE/base.$corpus"
    echo "$corpus: REGENERATED ($(wc -l <"$out") lines)"
    continue
  fi
  if diff -q "$BASELINE/base.$corpus" "$out" >/dev/null 2>&1; then
    echo "$corpus: IDENTICAL ($(wc -l <"$out") lines)"
  else
    echo "$corpus: DIFFERS"
    # -a: the artifacts carry escaped high bytes, and diff would
    # otherwise call them binary and print nothing useful.
    diff -a "$BASELINE/base.$corpus" "$out" | head -40
    fail=1
  fi
done
exit $fail
