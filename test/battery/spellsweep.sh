#!/usr/bin/env bash
# Differential oracle for the spell subsystem: both halves of the
# on-disk format plus everything the reader exposes.
#
#   spellsweep.sh <nvim-binary> <vimruntime> <outdir> <label>
#
# Produces, under <outdir>:
#
#   <label>.txt          canonical report -- mkspell messages, :spellinfo,
#                        spellbadword verdicts, spellsuggest results
#   <label>.hashes       sha256 of every generated .spl/.sug, timestamps
#                        scrubbed, so the write path is byte-compared
#   <label>-files/       the scrubbed bytes themselves, for diffing a
#                        mismatch down to the offending section
#
# Run it once per binary, then `diff -r` the two <label> outputs.  Both
# runs reuse the same work directory path so no path text can differ.
#
# VIMRUNTIME is passed explicitly rather than left to exe-relative
# resolution: a baseline worktree resolves to its own runtime/ tree, and
# runtime/spell/en.utf-8.spl has to be the same 621 KB file on both
# sides or the shipped-dictionary case compares two dictionaries.

set -euo pipefail

if [[ $# -ne 4 ]]; then
  sed -n '2,20p' "$0" >&2
  exit 2
fi

NVIM=$(realpath "$1")
RUNTIME=$(realpath "$2")
OUT=$(realpath -m "$3")
LABEL=$4
HERE=$(cd "$(dirname "$0")" && pwd)

WORK=${SPELLSWEEP_WORK:-/tmp/spellsweep-work}
CORPUS=$WORK/corpus

mkdir -p "$OUT"
rm -rf "$WORK"
mkdir -p "$WORK"

"$HERE/spellcorpus.py" "$CORPUS" >&2

# The shipped en.utf-8.spl gets its own word lists: the corpus cases are
# a few dozen words each and cannot reach the parts of the reader that
# only a real dictionary has (deep tries, populated SAL/REP tables, the
# region and compound sections a generated corpus keeps small).
cat >"$WORK/enwords" <<'WORDS'
the quick brown fox jumps over lazy dog
The Quick BROWN
teh quik borwn jumpps ovre lasy dogg
receive recieve believe beleive weird wierd
separate seperate definitely definately occurrence occurence
accommodate acommodate embarrass embarass necessary neccessary
rhythm rythm conscience concience privilege priviledge
misspell mispell restaurant restarant
colour color favourite favorite
don't dont can't cant won't wont it's its
e-mail email co-operate cooperate
Washington washington WASHINGTON
i I a A an
1234 v1 x86 utf-8
antidisestablishmentarianism supercalifragilisticexpialidocious
zzzzz qqqqq xyzzy
run running runner ran runs
child children childs
mouse mice mouses
be is are was were been being
WORDS
tr ' ' '\n' <"$WORK/enwords" | grep -v '^$' >"$WORK/enwords.tmp"
mv "$WORK/enwords.tmp" "$WORK/enwords"

cat >"$WORK/ensugs" <<'SUGS'
teh
recieve
seperate
definately
occurence
acommodate
embarass
neccessary
rythm
concience
priviledge
mispell
restarant
wierd
beleive
quik
borwn
jumpps
ovre
lasy
dogg
speling
langauge
enviroment
goverment
independant
publically
truely
untill
wich
thier
SUGS

env -i \
  HOME="$WORK/home" \
  PATH=/usr/bin:/bin \
  TERM=dumb \
  SHELL=/bin/bash \
  VIMRUNTIME="$RUNTIME" \
  XDG_CONFIG_HOME="$WORK/home/.config" \
  XDG_DATA_HOME="$WORK/home/.local/share" \
  XDG_STATE_HOME="$WORK/home/.local/state" \
  XDG_CACHE_HOME="$WORK/home/.cache" \
  SWEEP_CORPUS="$CORPUS" \
  "$NVIM" --headless -u NONE -i NONE -n \
  -l "$HERE/spellsweep.lua" \
  >"$OUT/$LABEL.txt" 2>"$OUT/$LABEL.stderr"

# Scrub before hashing: :mkspell stamps time(NULL) into the SN_SUGFILE
# section of the .spl and into the .sug header, and the .sug is only
# loaded when the two agree.  Those eight bytes are the only thing in
# either format that is not a function of the input.
mapfile -t generated < <(find "$CORPUS" -name '*.spl' -o -name '*.sug' | sort)
"$HERE/splscrub.py" "${generated[@]}" >"$OUT/$LABEL.scrub"

rm -rf "${OUT:?}/$LABEL-files"
mkdir -p "$OUT/$LABEL-files"
: >"$OUT/$LABEL.hashes"
for f in "${generated[@]}"; do
  rel=${f#"$CORPUS"/}
  cp "$f" "$OUT/$LABEL-files/${rel//\//__}"
  printf '%s  %s\n' "$(sha256sum <"$f" | cut -d' ' -f1)" "$rel" \
    >>"$OUT/$LABEL.hashes"
done

printf '%s: %d generated files, report %d lines\n' \
  "$LABEL" "${#generated[@]}" "$(wc -l <"$OUT/$LABEL.txt")"
