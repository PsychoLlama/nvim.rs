#!/usr/bin/env bash
# sessgold -- the `:mksession`/`:mkview`/`:mkvimrc`/`:mkexrc` byte-golden.
#
#   sessgold.sh <nvim-binary> <vimruntime> <outdir> <label>
#
# `:mksession` is an on-disk format and the roadmap's byte-compat ask;
# cmdsweep's s17s normalises every digit and so cannot see what makeopens
# (453 lines) and put_view (230) actually emit.  This one keeps the bytes:
# eight fixed scenes, one child each, every `:mk*` form run against the
# scene and the emitted file reported verbatim.
#
# Produces, under <outdir>:
#   <label>.txt      the report: `F <scene> <form> <n> <line>` per emitted
#                    line, plus scene/layout/this_session/viewdir/loadview
#                    and the ex_mkrc error arms.
#   <label>.stderr   what nvim wrote to the prompt.  Thin by construction:
#                    the forms are `pcall`ed so a failing one is a recorded
#                    row rather than a dead child, and a pcall catches the
#                    error before emsg reaches stderr (B16-5).  The `!` and
#                    `X` rows in .txt are the error artifact.
#
# SANDBOX.  Everything is under $WORK (default /tmp/sessgold, removed and
# recreated per run); `env -i` keeps the caller's locale and XDG dirs out.
# $WORK is scrubbed to <WORK> in three spellings -- plain, get_view_file's
# `=+` encoding, and via $VIMRUNTIME/$HOME -- so two work directories of
# different NAME LENGTHS must produce byte-identical artifacts.  That is
# the determinism test, and the reason nothing else is normalised: every
# digit in a session file is load-bearing.
#
# SESSGOLD_WORK overrides the work directory (used by the determinism run).

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

WORK=${SESSGOLD_WORK:-/tmp/sessgold}
LIMIT=${SESSGOLD_TIMEOUT:-300}

mkdir -p "$OUT"
rm -rf "$WORK"
mkdir -p "$WORK/home" "$WORK/out" "$WORK/deep" "$WORK/vdir"

# Fixture files.  Fixed content, indented so that 'foldmethod=indent' has
# something to find, and one name that needs ses_escape_fname().
cat >"$WORK/a.txt" <<'EOF'
The quick brown fox
    second line indented
    third line indented
	fourth line tabbed
fifth line plain
    sixth line indented
    seventh line indented
        eighth line deeper
ninth line plain
tenth line plain
EOF
cat >"$WORK/b.txt" <<'EOF'
alpha
    beta
    gamma
        delta
epsilon
EOF
printf 'one\ntwo\nthree\n' >"$WORK/c.txt"
printf 'deep one\ndeep two\n' >"$WORK/deep/d.txt"
printf 'odd one\nodd two with a percent %% and a hash #\n' >"$WORK/odd name%#.txt"
: >"$WORK/empty"

: >"$OUT/$LABEL.txt"
: >"$OUT/$LABEL.stderr.raw"

run_scene() {
  local scene=$1
  set +e
  timeout -k 5 "$LIMIT" \
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
    NVIM_TEST=1 \
    SG_WORK="$WORK" \
    SG_RUNTIME="$RUNTIME" \
    SG_SCENE="$scene" \
    "$NVIM" --headless -u NONE -i NONE \
    --cmd "cd $WORK" \
    -l "$HERE/sessgold.lua" \
    <"$WORK/empty" \
    >>"$OUT/$LABEL.txt" 2>>"$OUT/$LABEL.stderr.raw"
  local st=$?
  set -e
  printf 'exit\t%s\t%d\n' "$scene" "$st" >>"$OUT/$LABEL.txt"
  # Each scene starts from a clean viewdir and output directory: 'viewdir'
  # accumulates across scenes otherwise and the listing stops being a
  # function of the scene.
  rm -rf "$WORK/vdir" "$WORK/out"
  mkdir -p "$WORK/vdir" "$WORK/out"
}

SCENES=$(
  env -i HOME="$WORK/home" PATH=/usr/bin:/bin TERM=dumb LANG=C.UTF-8 \
    VIMRUNTIME="$RUNTIME" SG_WORK="$WORK" SG_SCENE=LIST \
    "$NVIM" --headless -u NONE -i NONE -l "$HERE/sessgold.lua" \
    <"$WORK/empty" 2>/dev/null
)

for scene in $SCENES; do run_scene "$scene"; done

sed -E \
  -e "s#$WORK#<WORK>#g" \
  -e "s#$RUNTIME#<RT>#g" \
  -e "s#$HERE/sessgold\.lua#<SCRIPT>#g" \
  -e "s#\.\.\.[^ \"]*/sessgold\.lua#<SCRIPT>#g" \
  -e 's#[^ ]*/target/debug/nvim#<NVIM>#g' \
  -e 's#[^ ]*/nvim-[0-9a-f]+#<NVIM>#g' \
  -e 's#[^ ]*/vim/_core/#<CORE>/#g' \
  <"$OUT/$LABEL.stderr.raw" >"$OUT/$LABEL.stderr"
rm -f "$OUT/$LABEL.stderr.raw"

printf '%s: report %d lines, stderr %d lines, scenes %d\n' \
  "$LABEL" "$(wc -l <"$OUT/$LABEL.txt")" "$(wc -l <"$OUT/$LABEL.stderr")" \
  "$(echo "$SCENES" | wc -w)"
