#!/usr/bin/env bash
# Differential oracle for the message pipeline: `message/` and its
# callers -- `:echo`/`:echon`/`:echomsg`/`:echoerr`/`:echohl`, the `emsg`
# family under `shortmess`, `msg_trunc` at three widths and four
# 'cmdheight's, the history (`:messages`, a count, `clear`, 'msghistory'),
# the three message variables, `:redir`/`execute()` capture, `nvim_echo`
# chunks and kinds (`ext_messages`), `confirm()` with scripted answers, the
# display of every value type, and control/multibyte text through
# `msg_outtrans`/`str2special`.
#
#   msgsweep.sh <nvim-binary> <outdir> <label>
#
# The gap this closes: `message/` had no differential beyond `test_messages`
# and the UI specs, and the work after it moves every message's bytes from
# raw C strings to slices.
#
# THE HARNESS IS SCRSWEEP'S: `-ll` runs msgsweep.lua, which drives children
# through the functional suite's `testnvim` + `ui/screen` (a real UI client,
# so the hit-enter and more prompts, truncation and 'cmdheight' are real).
# The suite's default `--cmd` set applies (`nomore`, `shortmess+=IS`, the
# `vim` colour scheme); the cases that need otherwise say so.
#
# Produces, under <outdir>:
#
#   <label>.txt     the grid per screen case, plus ext_ state and the mode
#   <label>.attrs   the attribute table each grid's ids refer to
#   <label>.vals    one row per value case (captured text / value / error)
#
# Every record is numbered; `.vals` ends with `## LAST <n>`, which
# msgverify.sh checks against the expected count so a driver that stopped
# early cannot read IDENTICAL against a base that stopped at the same place.
#
# The work directory (the children's cwd, where the file cases write) is a
# constant length under the build dir, and only relative names reach a
# message.  TAKES ~25 s debug.
#
# MSGSWEEP_ONLY is a Lua pattern over section names (iteration only);
# MSGSWEEP_TRACE=1 names each section on stderr.
set -uo pipefail

nvim_bin=$(realpath "$1")
outdir=$(realpath -m "$2")
label=$3

here=$(dirname "$(realpath "$0")")
root=${MSGSWEEP_REPO:-$(dirname "$(dirname "$here")")}

mkdir -p "$outdir"

build_dir=$root/target/msgsweep
xdg=$build_dir/xdg
tmpdir=${XDG_RUNTIME_DIR:-/tmp}/nvim.rs-msgsweep-$$
chmod -R u+rwX "$xdg" 2>/dev/null || true
rm -rf "$xdg"
mkdir -p "$xdg/work" "$tmpdir"
ln -sfn "$root/runtime" "$xdg/runtime"
ln -sfn "$root/test" "$xdg/test"

cd "$xdg" || exit 1
NVIMRS_ROOT=$root \
MSGSWEEP_WORK=$xdg/work \
MSGSWEEP_ONLY=${MSGSWEEP_ONLY:-} \
MSGSWEEP_TRACE=${MSGSWEEP_TRACE:-} \
NVIM_TEST=1 \
LC_ALL=en_US.UTF-8 \
VIMRUNTIME=$root/runtime \
XDG_CONFIG_HOME=$xdg/config \
XDG_DATA_HOME=$xdg/share \
XDG_STATE_HOME=$xdg/state \
NVIM_LOG_FILE=$build_dir/nvim.log \
NVIM_PRG=$nvim_bin \
TMPDIR=$tmpdir \
HISTFILE=/dev/null \
SHELL=sh \
SYSTEM_NAME=$(uname -s) \
  timeout -k 5 "${MSGSWEEP_TIMEOUT:-300}" \
  "$nvim_bin" -ll "$here/msgsweep.lua" "$outdir" "$label"
rc=$?
if [ "$rc" -ge 124 ]; then
  echo "msgsweep $label: TIMED OUT after ${MSGSWEEP_TIMEOUT:-300}s" >&2
  pkill -f "msgsweep.lua $outdir $label" 2>/dev/null
fi

rm -rf "$tmpdir"
exit $rc
