#!/usr/bin/env bash
# Golden oracle for the undofile (`.un~`), i.e. batch B-undo's on-disk
# surface: undo/write.rs + undo/file.rs.
#
#   undogold.sh <nvim-binary> <vimruntime> <outdir> <label>
#
# Produces, under <outdir>:
#
#   <label>.txt       canonical report -- every case's buffer text,
#                     undotree(), :undolist and every message
#   <label>.struct    field-by-field decode of each .un~: file header,
#                     every uh record, every uep, every extmark
#   <label>.hashes    sha256 of every generated .un~ after the two
#                     wall-clock fields are masked, so the write path is
#                     byte-compared
#   <label>-files/    the scrubbed bytes themselves, for diffing a
#                     mismatch down to the offending record
#
# Run once per binary, then `diff -r` the two <label> outputs; or use
# undoverify.sh, which does that against the stored baseline.
#
# Both runs reuse the same work directory path, because that path is what
# 'undodir' munges into the undo file's *name* and what every message
# prints -- a differing work directory would report every case as
# changed.
#
# VIMRUNTIME is passed explicitly rather than left to exe-relative
# resolution, so a baseline worktree does not compare its own runtime/
# tree against the working one.

set -euo pipefail

if [[ $# -ne 4 ]]; then
  sed -n '2,30p' "$0" >&2
  exit 2
fi

NVIM=$(realpath "$1")
RUNTIME=$(realpath "$2")
OUT=$(realpath -m "$3")
LABEL=$4
HERE=$(cd "$(dirname "$0")" && pwd)

# Short on purpose: nvim truncates messages to fit the (headless, 80
# column) screen, and 'undodir' turns the whole path into one file name,
# so a long work directory would leave the report full of half-elided
# names the scrub cannot recognise.
WORK=${UNDOGOLD_WORK:-/tmp/ugold}
ART=$WORK/artifacts

mkdir -p "$OUT"
rm -rf "$WORK"
mkdir -p "$ART" "$WORK/home"

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
  -l "$HERE/undogold.lua" \
  >"$OUT/$LABEL.txt" 2>"$OUT/$LABEL.stderr"

# Anything the driver could not capture in-process still has to be
# compared, so the stderr stream is scrubbed the same two ways the Lua
# does -- the work directory and the "N seconds ago" phrases -- and
# appended to the report.  Left out, a message that moved would be a
# silent pass.
{
  printf '\n== stderr\n'
  sed -e "s#${WORK//\#/\\\#}#<WORK>#g" \
      -e 's/[0-9][0-9]* [a-z][a-z]*s\? ago/<AGO>/g' \
      "$OUT/$LABEL.stderr"
} >>"$OUT/$LABEL.txt"

# --- scrub, decode, hash --------------------------------------------------
# The scrubber rewrites in place (masking the two wall-clock fields) and
# prints the structural decode.  It announces a file it cannot parse
# rather than passing it through, so a format change is a report
# difference and not a silent pass.
: >"$OUT/$LABEL.struct"
mapfile -t undos < <(find "$ART" -name '*.un~' | sort)
if ((${#undos[@]})); then
  "$HERE/undoscrub.py" "${undos[@]}" >>"$OUT/$LABEL.struct"
fi

rm -rf "${OUT:?}/$LABEL-files"
mkdir -p "$OUT/$LABEL-files"
: >"$OUT/$LABEL.hashes"
for f in "${undos[@]}"; do
  name=$(basename "$f")
  cp "$f" "$OUT/$LABEL-files/$name"
  printf '%s  %s\n' "$(sha256sum <"$f" | cut -d' ' -f1)" "$name" \
    >>"$OUT/$LABEL.hashes"
done

# Files whose scrubbed bytes are identical, as one line per group.  The
# round-trip pair (write, read back, write again with nothing walked in
# between) is meant to land in one group, so a read path that stopped
# being the write path's inverse shows up here as a group that came
# apart -- not only as two hashes that both moved.
sort "$OUT/$LABEL.hashes" | awk '
  { if ($1 == prev) { group = group " " $2 }
    else { if (n > 1) print "identical:" group; group = " " $2; n = 0 }
    prev = $1; n++ }
  END { if (n > 1) print "identical:" group }
' >>"$OUT/$LABEL.hashes"

printf '%s: %d undo artifacts, report %d lines, struct %d lines\n' \
  "$LABEL" "${#undos[@]}" \
  "$(wc -l <"$OUT/$LABEL.txt")" "$(wc -l <"$OUT/$LABEL.struct")"
