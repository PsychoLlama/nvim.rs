#!/usr/bin/env bash
# Differential oracle for the insert-completion family --
# crates/nvim/src/insexpand/*.rs and the `Insstart`/`compl_*` globals
# behind it.  See the header of inssweep.lua for what it
# covers and why every case runs in an `--embed` child.
#
#   inssweep.sh <nvim-binary> <vimruntime> <outdir> <label>
#
# Produces, under <outdir>:
#
#   <label>.txt     canonical report: one line per OBSERVATION (that is,
#                   per keystroke, not per case), a `## <section> rows=N`
#                   line per section, `## TOTAL` and a final `exit N`.
#   <label>.dump    the BULK layer: the full match list per observation
#                   (word|abbr|kind|menu|user_data|hl_group), the buffer
#                   after every keystroke, `v:completed_item` in full,
#                   `complete_info()` verbatim for every key subset, and
#                   a dying child's stderr.  Moving ALONE means an item
#                   FIELD moved with every scalar answer still right.
#   <label>.stderr  what the parent and its children wrote to the
#                   prompt.  EMPTY at the baseline; that is the
#                   assertion.
#
# SANDBOX.  $WORK is a fresh `mktemp -d /tmp/inssweep.XXXXXXXXXX` unless
# INSSWEEP_WORK says otherwise -- and it must stay SHORT, because the
# children are started with `--listen $WORK/cN.sock` and a unix socket
# path over 108 bytes is `connection refused` with no other explanation.
# cwd is $WORK, $HOME is $WORK/home, $TMPDIR is $WORK/tmp and $PATH is
# $WORK/bin, which holds a copy of the binary under test and nothing
# else.  `env -i`.
#
# THE FIXTURE IS BUILT HERE, not in Lua, so it is one byte-stable tree a
# later slice can read: four buffers with disjoint word sets (so which
# `'complete'` source answered is readable off the words alone), a
# dictionary, a thesaurus, a tags file, an include tree, a directory for
# CTRL-X CTRL-F and a generated spell file for `kspell` / CTRL-X CTRL-S.
#
# ORPHANS.  i91 starts one FRESH child per case and reaps each before
# its row is printed; the standing child is stopped at the end of the
# driver.  If a run is killed, look for children with ppid 1 and KILL
# THEM BY PID -- never by pattern, which would take out the operator's
# own editor.
set -euo pipefail

if [[ $# -ne 4 ]]; then
  sed -n '2,45p' "$0" >&2
  exit 2
fi

NVIM=$(realpath "$1")
RUNTIME=$(realpath "$2")
OUT=$(realpath -m "$3")
LABEL=$4
HERE=$(cd "$(dirname "$0")" && pwd)

LIMIT=${INSSWEEP_TIMEOUT:-900}

OWNED=0
if [[ -n ${INSSWEEP_WORK:-} ]]; then
  WORK=$INSSWEEP_WORK
  rm -rf "$WORK"
  mkdir -p "$WORK"
else
  WORK=$(mktemp -d /tmp/inssweep.XXXXXXXXXX)
  OWNED=1
fi

mkdir -p "$OUT"
umask 022
mkdir -p "$WORK/home" "$WORK/bin" "$WORK/tmp" "$WORK/inc" "$WORK/files" "$WORK/spell"
chmod 755 "$WORK" "$WORK/home" "$WORK/bin" "$WORK/tmp"

cp "$NVIM" "$WORK/bin/nvim"
chmod 755 "$WORK/bin/nvim"
: >"$WORK/empty"

# ----------------------------------------------------------- the fixture
#
# Disjoint word sets, one per source.  `alpha`/`alp` live in the current
# buffer; every other source answers with a prefix nothing else uses, so
# a row's `words=` says which source produced it without a second lookup.

cat >"$WORK/win.txt" <<'EOF'
winalpha winbeta wingamma
alwin alwindow
EOF

cat >"$WORK/hid.txt" <<'EOF'
hidalpha hidbeta hidgamma
alhid alhidden
EOF

cat >"$WORK/unl.txt" <<'EOF'
unlalpha unlbeta unlgamma
alunl alunloaded
EOF

cat >"$WORK/uls.txt" <<'EOF'
ulsalpha ulsbeta ulsgamma
aluls alunlisted
EOF

cat >"$WORK/words.dict" <<'EOF'
alabaster
albatross
alchemy
alcove
aldebaran
zeppelin
EOF

printf 'alpha\tfirst\tprimary\tinitial\n' >"$WORK/thes.txt"
printf 'alcove\tnook\trecess\n' >>"$WORK/thes.txt"
printf 'bravo\tcheer\tapplaud\n' >>"$WORK/thes.txt"

cat >"$WORK/inc/one.h" <<'EOF'
#define INCONEMACRO 1
#define INCONEOTHER 2
int inconefunc(void);
EOF

cat >"$WORK/inc/two.h" <<'EOF'
#include "one.h"
#define INCTWOMACRO 3
int inctwofunc(void);
EOF

cat >"$WORK/src.c" <<'EOF'
#include "one.h"
#include "two.h"
#define SRCMACRO 4
EOF

for f in aardvark abacus abalone; do
  printf 'file %s\n' "$f" >"$WORK/files/$f.txt"
done

# Tags: the classic three-column form, sorted, so `]`/`t` and
# CTRL-X CTRL-] all read the same table.
{
  printf 'tagalpha\tsrc.c\t/^#define SRCMACRO/\n'
  printf 'tagbravo\tinc/one.h\t/^int inconefunc/\n'
  printf 'tagcharlie\tinc/two.h\t/^int inctwofunc/\n'
} >"$WORK/tags"

# The spell file for `kspell` and CTRL-X CTRL-S.  Generated rather than
# vendored: `:mkspell` is deterministic for a fixed word list, and a
# vendored `.spl` would pin a file format this fork is free to change.
cat >"$WORK/wordlist" <<'EOF'
alpha
alphabet
alpine
album
bravo
charlie
delta
EOF

env -i HOME="$WORK/home" PATH="$WORK/bin" TMPDIR="$WORK/tmp" TERM=dumb \
  LANG=C.UTF-8 VIMRUNTIME="$RUNTIME" \
  "$WORK/bin/nvim" --headless -u NONE -i NONE \
  --cmd "cd $WORK" \
  -c "silent mkspell! $WORK/spell/ins $WORK/wordlist" \
  -c 'qa!' >"$WORK/mkspell.log" 2>&1 || true

# ----------------------------------------------------------------- run

set +e
timeout -k 5 "$LIMIT" \
  env -i \
  HOME="$WORK/home" \
  PATH="$WORK/bin" \
  TMPDIR="$WORK/tmp" \
  TERM=dumb \
  SHELL=/bin/sh \
  LANG=C.UTF-8 \
  VIMRUNTIME="$RUNTIME" \
  XDG_CONFIG_HOME="$WORK/home/.config" \
  XDG_DATA_HOME="$WORK/home/.local/share" \
  XDG_STATE_HOME="$WORK/home/.local/state" \
  XDG_CACHE_HOME="$WORK/home/.cache" \
  NVIM_TEST=1 \
  INS_WORK="$WORK" \
  INS_NVIM="$WORK/bin/nvim" \
  INS_DUMP="$OUT/$LABEL.dump.raw" \
  INSSWEEP_ONLY="${INSSWEEP_ONLY:-}" \
  INSSWEEP_TRACE="${INSSWEEP_TRACE:-}" \
  "$WORK/bin/nvim" --headless -u NONE -i NONE \
  --cmd "cd $WORK" \
  --cmd 'set noswapfile shell=/bin/sh undolevels=1000' \
  -c 'set columns=80 lines=24 report=9999 nomore shortmess=aoOtTIcCF' \
  -c "luafile $HERE/inssweep.lua" \
  -c 'qa!' \
  <"$WORK/empty" \
  >"$OUT/$LABEL.txt" 2>"$OUT/$LABEL.stderr.raw"
status=$?
set -e

scrub() { # in out
  sed -E \
    -e "s#$WORK#<WORK>#g" \
    -e "s#$HERE/inssweep\.lua#<SCRIPT>#g" \
    -e "s#\.\.\.[^ \"]*/inssweep\.lua#<SCRIPT>#g" \
    -e "s#$RUNTIME#<RT>#g" \
    -e 's#0x[0-9a-f]+#<ADDR>#g' \
    -e 's#\.rs:[0-9]+:[0-9]+#.rs:<LINE>#g' \
    -e 's#\.rs:[0-9]+#.rs:<LINE>#g' \
    <"$1" >"$2"
}

scrub "$OUT/$LABEL.stderr.raw" "$OUT/$LABEL.stderr"
rm -f "$OUT/$LABEL.stderr.raw"

touch "$OUT/$LABEL.dump.raw"
scrub "$OUT/$LABEL.dump.raw" "$OUT/$LABEL.dump"
rm -f "$OUT/$LABEL.dump.raw"

scrub "$OUT/$LABEL.txt" "$OUT/$LABEL.txt.scrubbed"
mv "$OUT/$LABEL.txt.scrubbed" "$OUT/$LABEL.txt"

printf 'exit %d\n' "$status" >>"$OUT/$LABEL.txt"

if [[ $OWNED -eq 1 && -z ${INSSWEEP_KEEP:-} ]]; then
  rm -rf "$WORK"
fi

printf '%s: report %d lines, dump %d lines, stderr %d lines, exit %d\n' \
  "$LABEL" "$(wc -l <"$OUT/$LABEL.txt")" "$(wc -l <"$OUT/$LABEL.dump")" \
  "$(wc -l <"$OUT/$LABEL.stderr")" "$status"
