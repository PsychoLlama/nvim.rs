#!/usr/bin/env bash
# Record a restartable Ex-command probe against one binary.
#
#   ex-run.sh <nvim-binary> <vimruntime> <outdir> <label> [probe]
#
# `probe` is `parse` (nvim_parse_cmd over 1,946 command lines -- pure, so it
# cannot crash and cannot leak) or `excmd` (1,368 (scene, command) pairs run
# for effect through do_cmdline).  Writes <outdir>/<label>-<probe>.{txt,cases,err}.
#
# Rehomed at B17-5 from test/battery/1785211338-phase-14-scripts/d/, where
# THREE things were wrong with the phase-14 runner:
#
#   * `S=` hardcoded a session scratchpad that has not existed since phase 14,
#     so the script only ever worked from the shell it was written in.  Both
#     the probes and the output directory are arguments now.
#   * the resume cursor was read with `grep -aP '^CASE\t'`, which matches the
#     EXCMD probe's rows and NOTHING in the parse probe's -- its rows begin
#     with a bare index.  So `.cases` came out empty for half the tool, its
#     count read 0, and a restart after a crash resumed from case 1 forever.
#     The pattern and the index column are per-probe now.
#   * `pwd` recorded the caller's working directory and `checkhealth` recorded
#     the nvim version, the build type, the ripgrep/git/curl versions on
#     $PATH, an ever-growing log size and seven /nix/store paths.  The run is
#     sandboxed under $WORK with `env -i` and both probes scrub $WORK,
#     $VIMRUNTIME and $HOME; the checkhealth case was narrowed to
#     `vim.deprecated` (see the probe).
#
# RESTART-RESUME.  The excmd probe runs real commands, so a case can kill the
# editor.  On a non-zero exit the runner finds the last index that completed,
# writes one ABORT row for the next one, and restarts past it -- so a crash is
# a diffable row, not a truncated file.  Advance on the row that RAN the
# command, never on a second line printed under the same index (P0.4b: doing
# so blames the following case and skips it).
set -u

if [[ $# -lt 4 ]]; then
  sed -n '2,34p' "$0" >&2
  exit 2
fi

NVIM=$(realpath "$1")
RUNTIME=$(realpath "$2")
OUT=$(realpath -m "$3")
LABEL=$4
PROBE=${5:-parse}
HERE=$(cd "$(dirname "$0")" && pwd)

case $PROBE in
  parse) SCRIPT=$HERE/parseprobe.lua; PAT='^[0-9]+\tparse\t'; IDX=1 ;;
  excmd) SCRIPT=$HERE/exprobe.lua;    PAT='^CASE\t';          IDX=2 ;;
  *) echo "unknown probe: $PROBE (parse|excmd)" >&2; exit 2 ;;
esac

WORK=${EX_RUN_WORK:-/tmp/exprobe}
mkdir -p "$OUT"
rm -rf "$WORK"
mkdir -p "$WORK/home"
: >"$WORK/empty"

TXT=$OUT/$LABEL-$PROBE.txt
ERR=$OUT/$LABEL-$PROBE.err
CASES=$OUT/$LABEL-$PROBE.list

run() {
  # PROBE_LIST must be ABSENT, not empty: Lua's os.getenv returns "" for an
  # empty variable and "" is TRUTHY, so `PROBE_LIST=""` put the probe in
  # list mode for its real run and every row came out three fields short.
  # `timeout` sits OUTSIDE `env -i`: with -i, env resolves the program name
  # against the environment it is building, and it does that before the
  # PATH= assignment takes effect -- so `env -i PATH=... timeout ...` exits
  # 127 with "env: 'timeout': No such file or directory", which the restart
  # loop then reads as 1,946 consecutive aborts.
  timeout 900 env -i \
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
    EX_WORK="$WORK" \
    EX_RUNTIME="$RUNTIME" \
    PROBE_FROM="${PROBE_FROM:-1}" \
    ${PROBE_LIST:+PROBE_LIST=1} \
    "$NVIM" --headless -u NONE -i NONE \
    --cmd "cd $WORK" --cmd "lua dofile('$SCRIPT')" \
    <"$WORK/empty"
}

PROBE_LIST=1 PROBE_FROM=1 run >"$CASES" 2>/dev/null

: >"$TXT"
from=1
restarts=0
while :; do
  PROBE_FROM=$from run >>"$TXT" 2>"$ERR"
  rc=$?
  [[ $rc == 0 ]] && break
  last=$(grep -aP "$PAT" "$TXT" | tail -n 1 | cut -f"$IDX")
  case $last in ''|*[!0-9]*) last=$((from - 1)) ;; esac
  ((last < from - 1)) && last=$((from - 1))
  ai=$((last + 1))
  ain=$(awk -F'\t' -v i="$ai" '$1 == i {print $3}' "$CASES")
  printf 'CASE\t%d\t%s\tABORT\tABORT\trc=%s\n' "$ai" "$ain" "$rc" >>"$TXT"
  from=$((last + 2))
  restarts=$((restarts + 1))
  if ((restarts > 300)); then echo 'TOO MANY RESTARTS' >>"$TXT"; break; fi
done

# The work directory reaches a handful of messages through file names even
# after the Lua scrub (an elided path, a name nvim built itself); the binary
# names itself in two more.
sed -i -E \
  -e "s#$WORK#<WORK>#g" \
  -e "s#$RUNTIME#<RT>#g" \
  -e "s#$SCRIPT#<SCRIPT>#g" \
  -e 's#[^ ]*/target/debug/nvim#<NVIM>#g' \
  -e 's#[^ ]*/nvim-[0-9a-f]+#<NVIM>#g' \
  "$TXT"

grep -aP "$PAT" "$TXT" >"$TXT.cases"
printf '%-6s %-10s rows=%s restarts=%s aborts=%s exit=%s\n' \
  "$PROBE" "$LABEL" "$(wc -l <"$TXT.cases")" "$restarts" \
  "$(grep -ac $'\tABORT\t' "$TXT.cases" || true)" "$rc"
