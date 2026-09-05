#!/usr/bin/env bash
# Differential oracle for the runtime family (batch B16):
# runtime/{estack,search,cache,pack,expand,rtp,source,script}.rs -- the
# 'runtimepath'/'packpath' construction, the rtp search and its cached
# path, :runtime / :packadd / :packloadall, ExpandRTDir, :source and the
# script registry (:scriptnames, getscriptinfo(), <sfile>, getstacktrace).
#
#   rtsweep.sh <nvim-binary> <vimruntime> <outdir> <label>
#
# The gap this closes: oldtest test_source/test_packadd/test_scriptnames
# and the runtime_spec/source_spec/startup_spec functional specs total
# ~4.1k lines and not one of them asserts the *search order* -- which
# file of several candidates wins, where `after/` lands, where a
# packadd'd directory is inserted into 'runtimepath'.  That is this
# subsystem's silent-wrong-answer class, so every fixture script here
# appends its own path to `g:RTM` and the answer is the whole list, in
# order.
#
# Produces, under <outdir>:
#
#   <label>.txt       canonical report, read top to bottom and diffed
#                     as-is.  One or more tagged lines per case:
#                       M  the marker list -- which scripts ran, in order
#                       R  a resolved path list (rtp, packpath, glob)
#                       O  captured output
#                       !  the error a command raised
#                       A  the case's own extra question
#   <label>.struct    canonical (sorted-key) JSON, one line per labelled
#                     answer, LC_ALL=C sorted AFTER the scrub, so the
#                     diff is independent of where in the run an answer
#                     was produced.
#   <label>.stderr    what nvim wrote to the prompt.  s20 runs a block of
#                     commands *uncaptured* so the message path
#                     (:scriptnames' listing, E-numbers, the "line N:"
#                     prefix a sourced script's errors carry) reaches an
#                     artifact.  It is the only view of msg_* here.
#
# SANDBOX.  Everything is confined to $WORK (default /tmp/rtsweep,
# removed and recreated on every run): cwd is $WORK, HOME is $WORK/home,
# every XDG directory is under it, and `env -i` means the report cannot
# pick up the caller's locale or configuration.
#
# THE BINARY IS COPIED TO $WORK/bin/nvim and run from there.  This
# subsystem derives paths from argv[0] (`get_lib_dir`, v:progpath, and
# the exe-relative fallback in `runtimepath_default`), so two binaries
# compared from different directories would differ for a reason that is
# not behaviour.  A fixed path removes the whole class.
#
# 'runtimepath' is set to the FIXTURE TREE ONLY for every section but
# s01: with $VIMRUNTIME in it, `:runtime plugin/*.vim` sources the real
# runtime/ tree and the artifact becomes a function of files this repo
# edits for unrelated reasons.  s01 is the section that asks what the
# default 'runtimepath' *is*, and it asks it in a child per environment.
#
# The nvim invocation is wrapped in `timeout` and its exit status is
# appended to the report as a final `exit <code>` line: 124 is the
# harness's verdict on a wedge, 134 an abort.  s91 runs the inputs that
# may kill the editor in a *child* each, so a crash there is one
# diffable ABORTED row rather than a truncated report.
#
# RTSWEEP_ONLY is a Lua pattern matched against each section name; it
# exists for iterating on one section, not for gating -- script IDs are
# monotonic, so a partial run renumbers s08's answers.
# RTSWEEP_TRACE=1 mirrors each section name to stderr, which is the only
# way to see where a wedged run stopped, and must be off for a baseline
# because it writes into the .stderr artifact.

set -euo pipefail

if [[ $# -ne 4 ]]; then
  sed -n '2,70p' "$0" >&2
  exit 2
fi

NVIM=$(realpath "$1")
RUNTIME=$(realpath "$2")
OUT=$(realpath -m "$3")
LABEL=$4
HERE=$(cd "$(dirname "$0")" && pwd)

# Short on purpose, and the same for every run: messages are truncated to
# the (headless, 80 column) screen and a long work directory leaves
# half-elided paths behind that the scrub cannot recognise.
WORK=${RTSWEEP_WORK:-/tmp/rtsweep}
LIMIT=${RTSWEEP_TIMEOUT:-900}

mkdir -p "$OUT"
rm -rf "$WORK"
mkdir -p "$WORK/home" "$WORK/bin"

cp "$NVIM" "$WORK/bin/nvim"
chmod 755 "$WORK/bin/nvim"

# stdin, explicitly.  In `-l` script mode a case that ends up asking the
# real input stream reads whatever the caller's terminal has; an empty
# file makes that EOF, deterministically, on every machine.
: >"$WORK/empty"

# ------------------------------------------------------------- fixtures
#
# Generated here rather than checked in, and generated the same way every
# time: no timestamps, no random names, no mtime-ordered directory reads
# that a `glob()` could return in a different order.  Every script's body
# is its own path, so the marker list *is* the search order.

mkfile() {
  mkdir -p "$(dirname "$WORK/$1")"
  printf '%s\n' "$2" >"$WORK/$1"
}

# A Vimscript file that announces itself.
vm() { mkfile "$1" "call add(g:RTM, '$1')${2:+
$2}"; }

# The same for Lua.  `vim.cmd` rather than `vim.g`, because `vim.g.RTM`
# copies the list out and back and the ORDER of two Lua files appending
# to it would survive a bug that lost one of them.
lm() { mkfile "$1" "vim.cmd(\"call add(g:RTM, '$1')\")${2:+
$2}"; }

# -- the runtimepath tree.  d1/d2/d3 overlap on plugin/p1.vim so that
# "which one wins" and "all of them, in order" are different answers.
vm rt/d1/plugin/p1.vim
lm rt/d1/plugin/p1.lua
vm rt/d1/plugin/p2.vim
vm rt/d1/plugin/sub/deep.vim
vm rt/d1/colors/one.vim
vm rt/d1/syntax/sx.vim
vm rt/d1/ftplugin/ft.vim
vm rt/d1/autoload/al.vim "function! al#f() abort
  return 'al-from-d1'
endfunction"
vm rt/d1/autoload/nest/ed.vim "function! nest#ed#g() abort
  return 'nested-autoload'
endfunction"
# A second autoload name, used once and only under a reversed
# 'runtimepath'.  Re-using `al#f` there would prove nothing: the first
# call defines the function for the rest of the process, so every later
# case answers from whichever tree won the FIRST time.
vm rt/d1/autoload/al2.vim "function! al2#f() abort
  return 'al2-from-d1'
endfunction"
vm rt/d1/after/plugin/ap.vim
vm rt/d1/after/plugin/p1.vim
mkfile rt/d1/lua/modone.lua "vim.cmd(\"call add(g:RTM, 'rt/d1/lua/modone.lua')\")
return { where = 'd1' }"

vm rt/d2/plugin/p1.vim
lm rt/d2/plugin/p3.lua
vm rt/d2/colors/two.vim
vm rt/d2/autoload/al.vim "function! al#f() abort
  return 'al-from-d2'
endfunction"
vm rt/d2/autoload/al2.vim "function! al2#f() abort
  return 'al2-from-d2'
endfunction"
vm rt/d2/after/plugin/ap.vim

vm rt/d3/plugin/p1.vim
vm rt/d3/plugin/only3.vim
mkfile rt/d3/lua/modthree.lua "return { where = 'd3' }"

# A RELATIVE rtp entry that is exactly the five characters "after".
# `path_is_after` guards its suffix compare with a length test, and every
# absolute fixture path is far past it; this is the only entry short
# enough to reach the guard.  The sweep's cwd is $WORK, so it resolves.
vm after/plugin/rel.vim

# An rtp entry that IS an `after` directory -- `path_is_after` sorts the
# cached search path on exactly this suffix, and nothing else in the
# fixture reaches it.
vm rt/xa/after/plugin/xa.vim

# -- the packpath tree.
vm pk/pack/alpha/start/s1/plugin/s1.vim
vm pk/pack/alpha/start/s1/after/plugin/s1a.vim
vm pk/pack/alpha/start/s1/ftdetect/s1d.vim
lm pk/pack/alpha/start/s2/plugin/s2.lua
vm pk/pack/alpha/opt/o1/plugin/o1.vim
vm pk/pack/alpha/opt/o1/after/plugin/o1a.vim
vm pk/pack/beta/start/s3/plugin/s3.vim
vm pk/pack/beta/opt/o2/plugin/o2.vim
mkfile pk/pack/beta/opt/o2/lua/o2mod.lua "return { where = 'o2' }"
# A pack directory with no loadable entry at all: `pack_has_entries` is
# what decides whether it reaches 'runtimepath'.
mkdir -p "$WORK/pk/pack/gamma/opt/hollow"
: >"$WORK/pk/pack/gamma/opt/hollow/README"

# -- a second packpath, so DIP_START/DIP_OPT ordering is over two roots.
vm pk2/pack/delta/start/s4/plugin/s4.vim
vm pk2/pack/delta/opt/o3/plugin/o3.vim

# -- scripts for :source.
vm src/plain.vim
vm src/nest.vim "source $WORK/src/plain.vim"
vm src/fin.vim "finish
call add(g:RTM, 'src/fin.vim UNREACHABLE')"
mkfile src/cont.vim "call add(g:RTM,
      \\ 'src/cont.vim')
let g:CONT = 'a'
      \\ . 'b'
      \\ . 'c'"
mkfile src/here.vim "let g:HERE =<< trim EOT
  one
    two
  three
EOT
call add(g:RTM, 'src/here.vim')"
# A REAL 0xe9 byte, not the four characters that spell one: the whole
# point of the case is that `:scriptencoding` converts it.
printf 'scriptencoding latin1\nlet g:ENC = "caf\xe9"\ncall add(g:RTM, "src/enc.vim")\n' \
  >"$WORK/src/enc.vim"
vm src/err.vim "call NoSuchFunction()
call add(g:RTM, 'src/err.vim after-error')"
vm src/sfile.vim "let g:SF = [expand('<sfile>'), expand('<sflnum>'), expand('<slnum>'), expand('<script>'), expand('<stack>')]"
vm src/func.vim "function! Deep() abort
  return [expand('<sfile>'), expand('<slnum>'), expand('<stack>'), getstacktrace()]
endfunction
function! Outer() abort
  return Deep()
endfunction"
lm src/plain.lua
mkfile src/err.lua "error('lua source exploded')"
mkfile src/vars.vim "let s:local = 'script-local'
function! s:hidden() abort
  return s:local
endfunction
function! Visible() abort
  return s:hidden()
endfunction
call add(g:RTM, 'src/vars.vim')"
mkfile src/trace.vim "function! T3() abort
  return getstacktrace()
endfunction
function! T2() abort
  return T3()
endfunction
function! T1() abort
  return T2()
endfunction"
printf 'call add(g:RTM, "src/crlf.vim")\r\nlet g:CRLF = 1\r\n' >"$WORK/src/crlf.vim"
printf '\xef\xbb\xbfcall add(g:RTM, "src/bom.vim")\nlet g:BOM = 1\n' >"$WORK/src/bom.vim"
printf 'call add(g:RTM, "src/noeol.vim")' >"$WORK/src/noeol.vim"
vm src/self.vim "source $WORK/src/self.vim"

# -- XDG trees for s01.  Only their existence and their names matter.
for d in cfg1 cfg2 data1 data2 state1 cache1; do
  mkdir -p "$WORK/xdg/$d/nvim"
done
mkdir -p "$WORK/xdg/co,mma/nvim" "$WORK/xdg/app/myapp"

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
  RT_WORK="$WORK" \
  RT_STRUCT="$OUT/$LABEL.struct.raw" \
  RTSWEEP_ONLY="${RTSWEEP_ONLY:-}" \
  RTSWEEP_TRACE="${RTSWEEP_TRACE:-}" \
  "$WORK/bin/nvim" --headless -u NONE -i NONE \
  --cmd "cd $WORK" \
  -l "$HERE/rtsweep.lua" \
  <"$WORK/empty" \
  >"$OUT/$LABEL.txt" 2>"$OUT/$LABEL.stderr.raw"
status=$?
set -e

# Scrubs the driver has to do rather than the Lua, because they land in
# messages the sweep never gets to touch.  The work directory reaches
# messages through file names; the Lua core modules carry a *relative*
# chunk name and resolve against the cwd; and `:scriptnames` prints the
# script path of this file itself.
sed -E \
  -e "s#$WORK#<WORK>#g" \
  -e "s#$HERE/rtsweep\.lua#<SCRIPT>#g" \
  -e "s#\.\.\.[^ \"]*/rtsweep\.lua#<SCRIPT>#g" \
  -e "s#$RUNTIME#<RT>#g" \
  -e 's#[^ ]*/vim/_core/#<CORE>/#g' \
  -e 's#[0-9]+ (second|minute|hour|day)s? ago#N ago#g' \
  <"$OUT/$LABEL.stderr.raw" >"$OUT/$LABEL.stderr"
rm -f "$OUT/$LABEL.stderr.raw"

printf 'exit %d\n' "$status" >>"$OUT/$LABEL.txt"

touch "$OUT/$LABEL.struct.raw"
LC_ALL=C sort "$OUT/$LABEL.struct.raw" >"$OUT/$LABEL.struct"
rm -f "$OUT/$LABEL.struct.raw"

printf '%s: report %d lines, struct %d lines, stderr %d lines, exit %d\n' \
  "$LABEL" "$(wc -l <"$OUT/$LABEL.txt")" "$(wc -l <"$OUT/$LABEL.struct")" \
  "$(wc -l <"$OUT/$LABEL.stderr")" "$status"
