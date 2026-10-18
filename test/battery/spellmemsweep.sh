#!/usr/bin/env bash
# Differential oracle for WHEN `:mkspell` compresses its word trees.
#
#   spellmemsweep.sh <nvim-binary> <vimruntime> <outdir> <label>
#
# `'mkspellmem'` (start KiB, increment KiB, words added) decides when the
# tree under construction is compressed, by the bytes its nodes would take
# in upstream's block allocator. The compression points are observable --
# each run prints "Compressed <tree>: n of m nodes" -- and the .spl/.sug
# bytes depend on them, so a port that charged the tree's memory
# differently would still write a valid file, just a different one. The
# `spell` row compiles small cases at the default setting and never
# reaches a second compression; this row compiles one 20,000-word input
# (`spellmemcorpus.py`) under settings that force hundreds of runs:
#
#   nosal 16,16,1      the tightest setting, no .sug pass
#   sal   48,16,2      .sug pass, compressing every couple of words
#   sal   160,32,8
#   sal   460000,2000,500  (the default)
#
# The tightest setting is not run with SAL: a .sug pass under 32 KiB makes
# upstream compress a keep-case tree it has already freed (the reference
# binary segfaults there; this tree drops the stale root).
#
# Produces <outdir>/<label>.txt: per setting the exit code, the message
# log with progress lines dropped, and the sha256 of the scrubbed
# .spl/.sug. Both sides reuse one work directory path, which the messages
# name.
set -euo pipefail

if [[ $# -ne 4 ]]; then
  sed -n '2,27p' "$0" >&2
  exit 2
fi

NVIM=$(realpath "$1")
RUNTIME=$(realpath "$2")
OUT=$(realpath -m "$3")
LABEL=$4
HERE=$(cd "$(dirname "$0")" && pwd)
WORK=${SPELLMEMSWEEP_WORK:-/tmp/spellmemsweep-work}

mkdir -p "$OUT"
rm -rf "$WORK"
mkdir -p "$WORK"
"$HERE/spellmemcorpus.py" "$WORK"

REPORT=$OUT/$LABEL.txt
: >"$REPORT"
for run in nosal:16,16,1 sal:48,16,2 sal:160,32,8 sal:460000,2000,500; do
  input=${run%%:*}
  msm=${run#*:}
  t=$WORK/${input}-${msm//,/_}
  mkdir -p "$t"
  rc=0
  (cd "$WORK" && timeout -k 5 300 env -i HOME="$WORK" PATH=/usr/bin:/bin TERM=dumb \
    VIMRUNTIME="$RUNTIME" "$NVIM" --clean --headless -n -i NONE \
    -c "set mkspellmem=$msm" -c "redir! > $t/msgs" \
    -c "mkspell! $t/out.utf-8.spl $input" -c 'redir END' -c 'qa!' \
    </dev/null >/dev/null 2>&1) || rc=$?
  for f in "$t"/out.utf-8.spl "$t"/out.utf-8.sug; do
    [[ -f $f ]] && "$HERE/splscrub.py" "$f" >/dev/null
  done
  {
    echo "=== $input mkspellmem=$msm rc=$rc"
    for f in "$t"/out.utf-8.spl "$t"/out.utf-8.sug; do
      [[ -f $f ]] && echo "$(basename "$f") $(sha256sum <"$f" | cut -c1-64)"
    done
    grep -av '^line \|^$' "$t/msgs" || true
  } >>"$REPORT"
done
echo "spellmemsweep: $(grep -c '^===' "$REPORT") settings, $(grep -c '^Compressed' "$REPORT") compressions"
