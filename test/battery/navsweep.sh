#!/usr/bin/env bash
# Differential oracle for the search and navigation subsystem (batch B11):
# search.rs, tag.rs, path.rs, file_search.rs, fuzzy.rs and quickfix.rs.
#
#   navsweep.sh <nvim-binary> <vimruntime> <outdir> <label>
#
# Produces, under <outdir>:
#
#   <label>.txt       canonical report -- every message, every jump, every
#                     expanded name, every quickfix and tag listing as the
#                     user would see it
#   <label>.struct    the same answers as sorted-key JSON: every dict field
#                     `getqflist()`/`getloclist()`/`taglist()`/
#                     `gettagstack()`/`searchcount()`/`matchfuzzypos()`
#                     returns, including the ones the readable report
#                     elides.  This is the field-level gate; the report
#                     alone is blind to a wrong `qfbufnr` or a dropped
#                     `changedtick` (the B10 `max_kbyte` lesson).
#
# Run once per binary, then `diff` the two <label> outputs.  Both runs use
# the same fixed work directory, because absolute paths land in quickfix
# entries, tag stacks and expanded names; a differing work directory would
# report every case as changed.
#
# VIMRUNTIME is passed explicitly rather than left to exe-relative
# resolution, so a baseline worktree does not compare its own runtime/
# tree against the working one.  :helpgrep and :help are pointed at a
# fixture doc/ directory instead of the real runtime, so the report does
# not move when runtime/doc does.

set -euo pipefail

if [[ $# -ne 4 ]]; then
  sed -n '2,28p' "$0" >&2
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
WORK=${NAVSWEEP_WORK:-/tmp/nsweep}

mkdir -p "$OUT"
rm -rf "$WORK"
mkdir -p "$WORK/home"

"$HERE/navfixture.sh" "$WORK"

# `env -i` so the report cannot pick up the caller's locale, editor
# configuration or XDG directories.  SHELL is set explicitly: nushell is
# the login shell here and `:!`/'grepprg'/'makeprg' would behave
# differently under it.  /bin/sh, not /bin/bash: this is NixOS and only
# the former exists at a fixed path.
env -i \
  HOME="$WORK/home" \
  PATH=/usr/bin:/bin \
  TERM=dumb \
  SHELL=/bin/sh \
  LANG=C.UTF-8 \
  VIMRUNTIME="$RUNTIME" \
  XDG_CONFIG_HOME="$WORK/home/.config" \
  XDG_DATA_HOME="$WORK/home/.local/share" \
  XDG_STATE_HOME="$WORK/home/.local/state" \
  XDG_CACHE_HOME="$WORK/home/.cache" \
  NAV_WORK="$WORK" \
  NAV_STRUCT="$OUT/$LABEL.struct" \
  "$NVIM" --headless -u NONE -i NONE \
  -l "$HERE/navsweep.lua" \
  >"$OUT/$LABEL.txt" 2>"$OUT/$LABEL.stderr"

printf '%s: report %d lines, struct %d lines, stderr %d lines\n' \
  "$LABEL" "$(wc -l <"$OUT/$LABEL.txt")" "$(wc -l <"$OUT/$LABEL.struct")" \
  "$(wc -l <"$OUT/$LABEL.stderr")"
