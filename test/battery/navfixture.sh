#!/usr/bin/env bash
# Build the deterministic fixture tree the B11 navigation sweep searches
# over.  Called by navsweep.sh with the work directory as $1;
# separated out so the corpus can be inspected and extended without
# re-reading the sweep driver.
#
# Everything here is byte-fixed: no timestamps, no host names, no
# generated ordering.  `find`-order is never relied on -- the sweep asks
# nvim to sort whatever it globs.
set -euo pipefail

WORK=$1
T=$WORK/tree
mkdir -p "$T"

# --- the searchable file tree --------------------------------------------
# Shapes that matter: a name that exists at several depths (upward and
# downward 'path' searches), an extension family ('suffixesadd'), a name
# needing 'wildignore'/'suffixes' to rank, a space and a non-ASCII byte in
# a name, a dotfile, and an empty directory.
mkdir -p "$T"/{inc,src,sub/deep,sub/other,dir1,dir2/nested,empty,link-target}
printf 'alpha\nbeta\ngamma\n'            >"$T/a.txt"
printf 'beta\nalpha\n'                   >"$T/b.txt"
printf 'one\ntwo\n'                      >"$T/a.log"
printf 'obj\n'                           >"$T/a.o"
printf 'backup\n'                        >"$T/a.txt~"
printf 'deep alpha\n'                    >"$T/sub/a.txt"
printf 'sub c\n'                         >"$T/sub/c.txt"
printf 'deeper alpha\n'                  >"$T/sub/deep/a.txt"
printf 'deeper x\n'                      >"$T/sub/deep/x.h"
printf 'other\n'                         >"$T/sub/other/a.txt"
printf 'has space\n'                     >"$T/sp ace.txt"
printf 'accented\n'                      >"$T/u-\xc3\xbc.txt"
printf 'hidden\n'                        >"$T/.hidden"
printf 'nested\n'                        >"$T/dir2/nested/a.txt"
# A deliberately deep chain, so a `**N` search can be limited by N and by
# FF_MAX_STAR_STAR_EXPAND: without it every downward search bottoms out
# at depth 3 and the limits are unobservable.
deep=$T/chain
mkdir -p "$deep"
for i in 1 2 3 4 5 6 7 8 9 10 11 12; do
  deep=$deep/d$i
  mkdir -p "$deep"
  printf 'chain depth %d\n' "$i" >"$deep/a.txt"
done
printf 'bottom\n' >"$deep/bottom.txt"
ln -sfn ../link-target "$T/sub/alink"
ln -sfn ./a.txt "$T/a-link.txt"
ln -sfn ./nowhere "$T/dangling"

# --- C-ish sources, for 'include'/'define' and :checkpath ------------------
cat >"$T/src/main.c" <<'EOF'
#include "util.h"
#include "missing.h"
#define MAIN_LIMIT 10
#define MAIN_NAME "main"
int main(void) { return util_run(MAIN_LIMIT); }
EOF
cat >"$T/src/util.c" <<'EOF'
#include "util.h"
int util_run(int n) { return n * 2; }
EOF
cat >"$T/src/util.h" <<'EOF'
#ifndef UTIL_H
#define UTIL_H
#include "inner.h"
#define UTIL_LIMIT 20
int util_run(int n);
#endif
EOF
cat >"$T/inc/inner.h" <<'EOF'
#define INNER_LIMIT 30
#define MAIN_LIMIT 99
EOF
cat >"$T/inc/two.h" <<'EOF'
#include "inner.h"
#define TWO 2
EOF

# --- a deliberate include tree, for find_pattern_in_path ------------------
# Its only suite coverage is test_checkpath + test_find_complete, so every
# shape the walk distinguishes gets a file here.  Kept in its own
# directory with its own vocabulary (ip*, cmt*) so a 'path' pointed at it
# reaches nothing else, and so no word here collides with the search
# corpus in hay.txt.
mkdir -p "$T/ipath"
# top.c is the include-tree root.  Line by line: a two-deep chain, an
# <>-form include, the *same* header the chain already pulled in (the
# "already searched" branch, which prints "(Already listed)" for
# :checkpath! and "(includes previously listed match)" for :ilist), a
# missing ""-form include and a missing <>-form one (the two halves of
# CHECK_PATH's name isolation, which re-attaches the surrounding
# quote/bracket), and a bare "#include" with nothing after it (the
# "nothing found, use the rest of the line" branch).
cat >"$T/ipath/top.c" <<'EOF'
#include "chain1.h"
#include <angle.h>
#include "diamond.h"
#include "ipgone.h"
#include <ipgone.h>
#include
#define IP_TOP 1
int ipmain(void) { return ipleaf(IP_TOP); }
EOF
cat >"$T/ipath/chain1.h" <<'EOF'
#ifndef CHAIN1_H
#define CHAIN1_H
#include "chain2.h"
#define IP_LIMIT 11
int ipleaf(int n);
#endif
EOF
cat >"$T/ipath/chain2.h" <<'EOF'
#include "diamond.h"
#define IP_LIMIT 22
EOF
# The chain's leaf, and the only backslash-continued #define in the
# corpus: show_pat_in_path keeps printing while a FIND_DEFINE line ends
# in a backslash, and that loop has a branch per line source (buffer vs
# included file).
cat >"$T/ipath/diamond.h" <<'EOF'
#define IP_LEAF 33
#define IP_CONT (1 + \
                 2 + \
                 3)
int ipleaf(int n);
EOF
cat >"$T/ipath/angle.h" <<'EOF'
#define IP_ANGLE 44
int ipangle(void);
EOF
# Reached by editing it, not by including it: the comment rules apply to
# the current buffer as well as to included files, and :ilist vs :ilist!
# is exactly the skip_comments flag.
cat >"$T/ipath/cmt.c" <<'EOF'
int cmtword = 1;
/* cmtword inside a block comment */
// cmtword after a line comment
int one;  /* cmtword after code */
 * cmtword in a continued block comment
int two; /* x */ cmtword after the comment closed
#define cmtword 7
# define cmtword 8
#hash cmtword after a hash leader
EOF
# A second root, so the continued #define is also reached at depth >= 0.
cat >"$T/ipath/cont.c" <<'EOF'
#include "diamond.h"
int ipcont(void) { return IP_CONT; }
EOF

# --- bracket-matching corpora --------------------------------------------
# findmatchlimit has four modes the pairs.txt corpus never reaches: C
# comment ends ([/ ]/ [* ]*), FM_BLOCKSTOP ([m ]m), raw strings (only
# from 'cindent'), and the whole Lisp branch of check_linecomment.
mkdir -p "$T/mp"
cat >"$T/mp/cmt.c" <<'EOF'
int a; /* one comment */ int b;
/* a comment that
   runs over lines */
int c;   // a line comment with /* inside it
/*/ a comment that starts and ends with a slash-star-slash /*/
int d; /* nested-looking /* start */ int e;
int f;  /* trailing */
EOF
cat >"$T/mp/blocks.c" <<'EOF'
int outer(void)
{
  if (x) {
    body();
  }
  return 0;
}
int second(void)
{
  return 1;
}
EOF
# Raw strings are reached through 'cindent' only, which is why this file
# is indented rather than searched.
cat >"$T/mp/raw.cpp" <<'EOF'
void f()
{
const char *s = R"delim(a ) not the end
still inside )delim";
int after = 1;
}
EOF
# The last line's "#x;" is deliberately *not* a comment start (a ';'
# preceded by '#' and one other character is a character literal), and
# the brackets straddle it -- so whether check_linecomment honours that
# rule decides whether '%' can cross the line at all.
cat >"$T/mp/code.lisp" <<'EOF'
(defun outer (a b)
  (let ((x (+ a b)))   ; a comment with ( in it
    (list x #\( #\) "a string with ; and ( in it")))
(defun second ()
  ;; leading comment ( and ) here
  (values))
(princ #\" ) ; a hash-quote then a comment with ( in it
(a #x; b )
EOF

# --- matchable text, for '%'/searchpair/searchc ---------------------------
cat >"$T/pairs.txt" <<'EOF'
outer ( first [ second { third } second ] first ) outer
if ( a && b ) { c( d[e] ); }
/* ( not a real open */ ) tail
// { line comment brace
"string ( in quotes )" plain ( real )
#if 0
#else
#endif
<tag attr="v"> body </tag>
EOF

# --- a body of text with predictable match positions ----------------------
cat >"$T/hay.txt" <<'EOF'
The quick brown Fox jumps
over the lazy dog. FOX
fox Fox fOx foX
alpha beta gamma alpha
    indented alpha here
tab	separated	alpha
multi
line
alpha at start
end alpha
EOF

# --- tag corpora ----------------------------------------------------------
# Tabs are load bearing; printf keeps them literal and keeps the trailing
# newline exact.
tags_sorted=$T/tags-sorted
{
  printf '!_TAG_FILE_FORMAT\t2\t//\n'
  printf '!_TAG_FILE_SORTED\t1\t//\n'
  printf 'Alpha\tsrc/util.c\t/^int util_run/;"\tf\n'
  printf 'alpha\tsrc/main.c\t/^int main/;"\tf\n'
  printf 'alphabet\tsrc/util.h\t/^int util_run/;"\tp\n'
  printf 'beta\tsrc/util.c\t2;"\tv\tfile:\n'
  printf 'beta\tsrc/util.h\t/^int util_run/;"\tp\n'
  printf 'gamma\tsrc/main.c\t/^#define MAIN_NAME/;"\td\tline:4\n'
  printf 'zeta\tsrc/missing.c\t/^nothing/\n'
} >"$tags_sorted"

# Same content, deliberately out of order and with the sorted flag off:
# the linear scan and the binary search must agree on the answers.
tags_unsorted=$T/tags-unsorted
{
  printf '!_TAG_FILE_SORTED\t0\t//\n'
  printf 'zeta\tsrc/missing.c\t/^nothing/\n'
  printf 'beta\tsrc/util.h\t/^int util_run/;"\tp\n'
  printf 'alpha\tsrc/main.c\t/^int main/;"\tf\n'
  printf 'Alpha\tsrc/util.c\t/^int util_run/;"\tf\n'
} >"$tags_unsorted"

# The "static" tag form: FILENAME:tagname.
tags_static=$T/tags-static
{
  printf 'src/util.c:local\tsrc/util.c\t/^int util_run/\n'
  printf 'plain\tsrc/util.c\t1\n'
} >"$tags_static"

# Extra fields, a search pattern containing the field separator, and a
# tag whose address is an ex command rather than a line number.
tags_fields=$T/tags-fields
{
  printf 'fields\tsrc/main.c\t/^int main(void) { return util_run(MAIN_LIMIT); }$/;"\tf\tsignature:(void)\ttyperef:typename:int\n'
  printf 'excmd\tsrc/util.c\t:call cursor(2, 1)\n'
  printf 'kindonly\tsrc/util.h\t3;"\tv\n'
} >"$tags_fields"

# Emacs format: ^L, filename,size, then "text\x7ftag\x01line,offset".
printf '\x0csrc/main.c,64\nint main(void)\x7femacstag\x011,0\n\x0csrc/util.c,32\nint util_run\x7fetwo\x012,0\n' \
  >"$T/TAGS"

# Rejection corpus.  Each is a *different* rejection path.
printf 'nofields\n'                              >"$T/tags-nofields"
printf 'trunc\t'                                 >"$T/tags-truncated"
printf 'sortedlie\tsrc/main.c\t1\n!_TAG_FILE_SORTED\t1\t//\nAAA\tsrc/main.c\t1\n' \
  >"$T/tags-sortedlie"
printf 'withnul\tsrc/ma\x00in.c\t1\n'             >"$T/tags-nul"
: >"$T/tags-empty"
head -c 2000 /dev/zero | tr '\0' 'x' >"$T/tags-huge-line"
printf '\n' >>"$T/tags-huge-line"

# --- a help fixture, so :helpgrep does not read the real runtime ----------
mkdir -p "$T/rtp/doc"
cat >"$T/rtp/doc/fix.txt" <<'EOF'
*fix.txt*	A fixture help file

			FIXTURE HELP

alpha appears here, and again: alpha.
*fix-one*	the first fixture tag
beta lives on this line.
*fix-two*	the second fixture tag
alpha one more time.
EOF
cat >"$T/rtp/doc/tags" <<'EOF'
fix-one	fix.txt	/*fix-one*
fix-two	fix.txt	/*fix-two*
fix.txt	fix.txt	/*fix.txt*
EOF

# --- a fixed 'makeprg', so :make is a pure function ----------------------
cat >"$T/mk.sh" <<'EOF'
#!/bin/sh
printf '%s\n' \
  "src/main.c:5:12: error: made-up failure" \
  "src/util.c:2:1: warning: made-up warning" \
  "make: *** [all] Error 1"
EOF
chmod +x "$T/mk.sh"

# --- an errorformat corpus, read by :cfile ------------------------------
cat >"$T/errors-basic.txt" <<'EOF'
src/main.c:5:12: error: basic one
src/util.c:2: basic two
src/util.h:3:4: warning: basic three
EOF
cat >"$T/errors-multiline.txt" <<'EOF'
Error in src/main.c line 5
  context line one
  context line two
end of error
Error in src/util.c line 2
  only context
end of error
EOF
cat >"$T/errors-dirstack.txt" <<'EOF'
Entering dir `src'
main.c:5: in the subdirectory
Leaving dir `src'
util.h:1: back at the top
EOF
printf 'plain text with no location at all\n' >"$T/errors-none.txt"
