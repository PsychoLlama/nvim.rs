#!/usr/bin/env bash
# Differential oracle for the persistence subsystem (batch B10):
# memline.rs, memfile.rs, shada.rs, fileio.rs and bufwrite.rs.
#
#   perssweep.sh <nvim-binary> <vimruntime> <outdir> <label>
#
# Produces, under <outdir>:
#
#   <label>.txt       canonical report -- every message, every recovered
#                     buffer, every decoded read, every written file
#   <label>.struct    block-zero and block-tree decode of each .swp, and
#                     the msgpack token stream of each .shada
#   <label>.hashes    sha256 of every generated .swp, timestamps/pids
#                     scrubbed, so the write paths are byte-compared
#   <label>-files/    the scrubbed bytes themselves, for diffing a
#                     mismatch down to the offending block.  The .shada
#                     copies still carry the writer's pid, so they are
#                     NOT byte-comparable between runs -- <label>.struct
#                     is their comparison, and it masks the pid.
#   <label>.masklog   what the version mask rewrote.  It names pre-mask
#                     sizes, so it is version-dependent on purpose and is
#                     never compared.
#
# Run once per binary, then `diff -r` the two <label> outputs.  Both runs
# reuse the same work directory path, because that path is written into
# the swap file's b0_fname and into every ShaDa mark entry -- a differing
# work directory would report every case as changed.
#
# VIMRUNTIME is passed explicitly rather than left to exe-relative
# resolution, so a baseline worktree does not compare its own runtime/
# tree against the working one.

set -euo pipefail

if [[ $# -ne 4 ]]; then
  sed -n '2,25p' "$0" >&2
  exit 2
fi

NVIM=$(realpath "$1")
RUNTIME=$(realpath "$2")
OUT=$(realpath -m "$3")
LABEL=$4
HERE=$(cd "$(dirname "$0")" && pwd)

# Short on purpose: nvim truncates messages to fit the (headless, 80
# column) screen, and a long work directory would leave the report full
# of half-elided paths that the scrub cannot recognise.
WORK=${PERSSWEEP_WORK:-/tmp/psweep}
ART=$WORK/artifacts

mkdir -p "$OUT"
rm -rf "$WORK"
mkdir -p "$WORK/seeds" "$ART" "$WORK/home"

# --- the fixed ShaDa seed -----------------------------------------------
# Hand-packed rather than produced by nvim: the merge and read halves need
# an input that does not depend on the binary under test, and fixed
# timestamps so which side of a merge wins is a property of the merge and
# not of when the sweep ran.  Two of the entries are stamped in 2033 so
# they beat anything the live session has; the rest are stamped in 2001 so
# the live session beats them.
python3 - "$WORK" >"$WORK/seeds/seed.json" <<'PY'
import json, sys
work = sys.argv[1]
f1 = work + "/shada/w-one.txt"
f2 = work + "/shada/w-two.txt"
OLD, NEW = 1000000000, 2000000000
spec = [
    [1, OLD, {"generator": "nvim", "version": "seed", "max_kbyte": 10,
              "pid": 1, "encoding": "utf-8"}],
    [2, NEW, {"sp": "seedpat", "sm": True, "su": True}],
    [3, OLD, ["seedsub"]],
    [4, OLD, [0, "seedcmd"]],
    [4, OLD, [1, "seedsearch", 47]],
    [4, OLD, [2, "seedexpr"]],
    [5, NEW, {"n": 97, "rc": ["seedreg"], "rt": 0}],
    [5, OLD, {"n": 100, "rc": ["seed d one", "seed d two"], "rt": 1}],
    [6, OLD, ["SEEDVAR", "seed value"]],
    [6, OLD, ["SEEDLIST", [1, 2, 3]]],
    [7, OLD, {"n": 67, "f": f1, "l": 1, "c": 0}],
    [8, OLD, {"f": f2, "l": 2, "c": 1}],
    [9, OLD, [{"f": f1, "l": 3, "c": 2}, {"f": f2, "l": 1, "c": 0}]],
    [10, OLD, {"f": f1, "n": 122, "l": 3, "c": 1}],
    [11, OLD, {"f": f1, "l": 2, "c": 0}],
]
json.dump(spec, sys.stdout)
PY
"$HERE/shadatok.py" --pack "$WORK/seeds/seed.json" "$WORK/seeds/seed.shada"

# --- the sweep ------------------------------------------------------------
# `env -i` so the report cannot pick up the caller's locale, editor
# configuration or XDG directories.  SHELL is set explicitly: nushell is
# the login shell here and `:!` would behave differently under it.
env -i \
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
  SWEEP_WORK="$WORK" \
  SWEEP_ART="$ART" \
  "$NVIM" --headless -u NONE -i NONE \
  -l "$HERE/perssweep.lua" \
  >"$OUT/$LABEL.txt" 2>"$OUT/$LABEL.stderr"

# --- mask the version string ----------------------------------------------
# Every ShaDa file nvim writes embeds `nvim.rs <version>` in its header,
# and that string's *length* moves the header entry's payload length, the
# file size and the offset of every later entry -- so before this the
# shada half of the sweep could never be compared against a stored
# baseline, only against another build of the same-length version (see
# shadamask.py).  Masking here, after the run and before
# anything is decoded, makes the artifacts a function of the code alone.
# Both the kept artifacts and $WORK/shada are masked: the report quotes
# sizes for the damaged files too, which never reach $ART.  The seeds are
# left alone -- they are hand-packed input, not output.
# PERSSWEEP_NO_MASK=1 restores the pre-P20-4 behaviour.
if [[ -z ${PERSSWEEP_NO_MASK:-} ]]; then
  mapfile -t allshada < <(find "$WORK" -name '*.shada' -not -path "$WORK/seeds/*" | sort)
  if ((${#allshada[@]})); then
    # Its log names the *pre-mask* sizes, so it is version-dependent by
    # construction and must not land in a compared artifact.
    "$HERE/shadamask.py" --report "$OUT/$LABEL.txt" "${allshada[@]}" \
      >"$OUT/$LABEL.masklog" 2>&1
  fi
fi

# --- scrub, describe, hash ------------------------------------------------
# The swap scrubber rewrites in place and prints the structural decode;
# the ShaDa tokeniser only reads.  Both announce a file they cannot parse
# rather than passing it through, so a format change is a report
# difference and not a silent pass.
: >"$OUT/$LABEL.struct"
mapfile -t swaps < <(find "$ART" -name '*.swp' | sort)
if ((${#swaps[@]})); then
  "$HERE/swapscrub.py" "${swaps[@]}" >>"$OUT/$LABEL.struct"
fi
mapfile -t shadas < <(find "$ART" -name '*.shada' | sort)
if ((${#shadas[@]})); then
  "$HERE/shadatok.py" "${shadas[@]}" >>"$OUT/$LABEL.struct"
fi

rm -rf "${OUT:?}/$LABEL-files"
mkdir -p "$OUT/$LABEL-files"
: >"$OUT/$LABEL.hashes"
for f in "${swaps[@]}"; do
  name=$(basename "$f")
  cp "$f" "$OUT/$LABEL-files/$name"
  printf '%s  %s\n' "$(sha256sum <"$f" | cut -d' ' -f1)" "$name" \
    >>"$OUT/$LABEL.hashes"
done
# ShaDa files are deliberately NOT hashed raw.  A pid encodes to one, two
# or four msgpack bytes depending on its value, so the same writer
# produces files of differing length between runs; the token stream in
# <label>.struct is the byte-level comparison for them, and it records
# every format byte the raw hash would have covered.  The bytes are still
# copied out so a token difference can be inspected.
for f in "${shadas[@]}"; do
  cp "$f" "$OUT/$LABEL-files/$(basename "$f")"
done

printf '%s: %d swap + %d shada artifacts, report %d lines, struct %d lines\n' \
  "$LABEL" "${#swaps[@]}" "${#shadas[@]}" \
  "$(wc -l <"$OUT/$LABEL.txt")" "$(wc -l <"$OUT/$LABEL.struct")"
