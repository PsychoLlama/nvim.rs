#!/usr/bin/env bash
# Read-base oracle: decode on-disk artifacts a FIXED binary wrote.
#
#   readbase.sh <nvim-binary> <vimruntime> <goldens> <outdir> <label>
#   readbase.sh --record <nvim-binary> <vimruntime> <goldens>
#
# The `pers` and `undo` rows compare what the binary under test WRITES,
# and read back only its own files -- a reader and a writer that moved
# together pass them.  This row closes that: <goldens> holds `.swp`,
# `.shada` and `.un~` files written once by a reference binary (the
# `pers` and `undo` corpora, scrubbed and masked exactly as those rows
# leave them), and every later binary reads them:
#
#   swap/*.swp       `:recover` it            -> messages, the buffer
#   shada/*.shada    `:rshada!` it            -> registers, history, marks,
#                                                vars, oldfiles, and a
#                                                `:wshada!` of what was kept,
#                                                tokenised by shadatok.py
#   undo/*.un~       `:rundo` it over undo/*.txt (the text its tree
#                    hashes)                  -> undotree(), and the buffer
#                                                at every sequence number
#
# Produces <outdir>/<label>.txt.  `--record` builds <goldens> from the
# given binary: it runs the two sweeps, keeps their artifacts, and writes
# <goldens>/expected.txt -- the same binary reading its own artifacts --
# which is what readbaseverify.sh compares against.  The goldens are not
# committed (they belong to one recording, not to the tree); see
# readbaseverify.sh.
#
# One nvim process per artifact, in a fresh work directory, so state one
# read leaves behind cannot reach the next.
set -euo pipefail

HERE=$(cd "$(dirname "$0")" && pwd)
# Short on purpose, as perssweep's: messages are elided to the screen width.
WORK=${READBASE_WORK:-/tmp/rbase}

if [[ ${1:-} == --record ]]; then
  [[ $# -eq 4 ]] || {
    sed -n '2,30p' "$0" >&2
    exit 2
  }
  NVIM=$(realpath "$2")
  RUNTIME=$(realpath "$3")
  GOLD=$(realpath -m "$4")
  rm -rf "$GOLD"
  mkdir -p "$GOLD/swap" "$GOLD/shada" "$GOLD/undo" "$GOLD/sweeps"
  # The sweeps' own work directories: their artifacts are scrubbed IN
  # PLACE there (swapscrub, shadamask, undoscrub), which is the form kept.
  PERSSWEEP_WORK=$GOLD/sweeps/psweep \
    "$HERE/perssweep.sh" "$NVIM" "$RUNTIME" "$GOLD/sweeps/pers" base
  UNDOGOLD_WORK=$GOLD/sweeps/ugold \
    "$HERE/undogold.sh" "$NVIM" "$RUNTIME" "$GOLD/sweeps/undo" base
  cp "$GOLD"/sweeps/psweep/artifacts/*.swp "$GOLD/swap/"
  cp "$GOLD"/sweeps/psweep/artifacts/*.shada "$GOLD/shada/"
  cp "$GOLD"/sweeps/ugold/artifacts/*.un~ "$GOLD"/sweeps/ugold/artifacts/*.txt \
    "$GOLD/undo/"
  # The swaps name their original files under psweep/ (b0_fname).  Gone,
  # those paths never exist again, so `:recover` cannot find an original
  # to compare against on one run and not on another.
  rm -rf "$GOLD/sweeps/psweep" "$GOLD/sweeps/ugold"
  "$0" "$NVIM" "$RUNTIME" "$GOLD" "$GOLD" expected
  (cd "$GOLD" && find swap shada undo -type f | LC_ALL=C sort | xargs sha256sum) \
    >"$GOLD/MANIFEST"
  printf 'recorded %s: %d swap, %d shada, %d undo\n' "$GOLD" \
    "$(find "$GOLD/swap" -type f | wc -l)" "$(find "$GOLD/shada" -type f | wc -l)" \
    "$(find "$GOLD/undo" -name '*.un~' | wc -l)"
  exit 0
fi

if [[ $# -ne 5 ]]; then
  sed -n '2,30p' "$0" >&2
  exit 2
fi
NVIM=$(realpath "$1")
RUNTIME=$(realpath "$2")
GOLD=$(realpath "$3")
OUT=$(realpath -m "$4")
LABEL=$5
mkdir -p "$OUT"
REPORT=$OUT/$LABEL.txt

# The goldens are read-only input: a run must not be able to change what
# the next run reads.  The manifest says what was recorded.
if [[ -f $GOLD/MANIFEST ]] && ! (cd "$GOLD" && sha256sum --quiet -c MANIFEST); then
  echo "readbase: $GOLD no longer matches its MANIFEST" >&2
  exit 1
fi

read_one() { # kind artifact [text]
  local kind=$1 src=$2 text=${3:-}
  rm -rf "$WORK"
  mkdir -p "$WORK/home"
  # Read from a copy under the work directory, so every message that
  # names the file names <WORK>, wherever the goldens live.
  local name
  name=$(basename "$src")
  cp "$src" "$WORK/$name"
  if [[ -n $text ]]; then cp "$text" "$WORK/text"; fi
  printf '\n== %s %s\n' "$kind" "$name"
  # `timeout` outside `env -i`: the scrubbed PATH need not hold it.
  timeout -k 2 30 env -i \
    HOME="$WORK/home" \
    PATH=/usr/bin:/bin \
    TERM=dumb \
    SHELL=/bin/bash \
    LANG=C.UTF-8 \
    VIMRUNTIME="$RUNTIME" \
    XDG_CONFIG_HOME="$WORK/home/.config" \
    XDG_DATA_HOME="$WORK/home/.local/share" \
    XDG_STATE_HOME="$WORK/home/.local/state" \
    XDG_CACHE_HOME="$WORK/home/.cache" \
    READBASE_WORK="$WORK" \
    "$NVIM" --headless -u NONE -i NONE \
    -l "$HERE/readbase.lua" "$kind" "$WORK/$name" ${text:+"$WORK/text"} 2>&1 ||
    echo "  exit $?"
  if [[ $kind == shada && -f $WORK/rewrite.shada ]]; then
    "$HERE/shadamask.py" --quiet "$WORK/rewrite.shada" >/dev/null 2>&1 || true
    (cd "$WORK" && "$HERE/shadatok.py" rewrite.shada) | sed 's/^/  tok | /'
  fi
}

{
  for f in "$GOLD"/swap/*.swp; do read_one swap "$f"; done
  for f in "$GOLD"/shada/*.shada; do read_one shada "$f"; done
  for f in "$GOLD"/undo/*.un~; do read_one undo "$f" "${f%.un~}.txt"; done
} >"$REPORT"
rm -rf "$WORK"

printf '%s: %d lines\n' "$LABEL" "$(wc -l <"$REPORT")"
