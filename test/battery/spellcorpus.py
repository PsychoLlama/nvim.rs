#!/usr/bin/env python3
"""Generate the .aff/.dic corpus that the spell sweep compiles.

The tree ships no spell fixtures at all -- every .aff/.dic in the test
suite is written inline by a .vim file and deleted again -- so the sweep
needs its own corpus.  One directory per case under <out>/, each holding
`t.aff` and `t.dic` (plus region siblings where the case needs them),
a `words` file of spellings to check and a `sugs` file of misspellings
to ask for suggestions on.

Cases were chosen to cover every keyword `spell_read_aff` recognises:

    ./spellcorpus.py <outdir>

`meta` in each directory carries the case's options, one `key=value`
per line, read by the sweep:

    enc=<latin1|utf8>     encoding to compile under (default utf8)
    inputs=<a> <b> ...    `:mkspell` input basenames (default `t`)
    ascii=1               compile with `-ascii`
"""

import shutil
import sys
from pathlib import Path

# Word lists reused across cases.  Deliberately mixes correct spellings,
# misspellings, case variants and words that only exist through affixes,
# so a verdict table distinguishes "not in the trie" from "in the trie
# but flagged".
COMMON_WORDS = "foo Foo FOO fooo bar Bar baz qux xyzzy the The".split()
COMMON_SUGS = "fooo brr bazz quux teh".split()

CASES: dict[str, dict] = {}


def case(name, aff, dic, words=None, sugs=None, extra=None, **meta):
    """Register a corpus case.  `dic` is the word list without its count
    line -- .dic files open with a line holding the word count, which
    mkspell reads but only warns about when wrong."""
    CASES[name] = {
        "aff": aff,
        "dic": dic,
        "words": words or COMMON_WORDS,
        "sugs": sugs or COMMON_SUGS,
        "extra": extra or {},
        "meta": meta,
    }


# --- baseline -------------------------------------------------------------

case(
    "basic",
    "",
    ["foo", "bar", "baz", "quux"],
)

# --- affixes --------------------------------------------------------------

# Y on the PFX/SFX header means "combines with the other kind", i.e. the
# prefix/suffix cross-product is generated; N suppresses it.  Both are
# present so the sweep sees a cross-product and a non-cross-product tree.
case(
    "pfxsfx",
    """\
PFX A Y 1
PFX A 0 re .

SFX B Y 2
SFX B 0 ed [^y]
SFX B y ied y

SFX C N 1
SFX C 0 ly .
""",
    ["work/AB", "carry/B", "quick/C", "play/AB"],
    words="work rework worked reworked carry carried quick quickly "
    "play replay played replayed reworkedly".split(),
    sugs="worked reworkd carryed quikly".split(),
)

# A suffix that strips characters before appending is a different trie
# shape from one that only appends.
case(
    "sfxstrip",
    """\
SFX S Y 3
SFX S e ing e
SFX S 0 ing [^e]
SFX S 0 s .
""",
    ["make/S", "walk/S", "bake/S"],
    words="make making makes walk walking walks bake baking makeing".split(),
    sugs="makeing walkin bakeing".split(),
)

# --- FLAG types -----------------------------------------------------------

# The four flag encodings pick four different parsers for the affix
# field, and change how flags are packed into the .spl word tree.
case(
    "flagnum",
    """\
FLAG num

SFX 101 Y 1
SFX 101 0 s .

PFX 102 Y 1
PFX 102 0 un .
""",
    ["fit/101,102", "tidy/102"],
    words="fit fits unfit unfits tidy untidy".split(),
    sugs="fitts unfitt tidyy".split(),
)

case(
    "flaglong",
    """\
FLAG long

SFX Sx Y 1
SFX Sx 0 s .

PFX Px Y 1
PFX Px 0 un .
""",
    ["fit/SxPx", "tidy/Px"],
    words="fit fits unfit unfits tidy untidy".split(),
    sugs="fitts unfitt tidyy".split(),
)

case(
    "flagcaplong",
    """\
FLAG caplong

SFX Sx Y 1
SFX Sx 0 s .

PFX Px Y 1
PFX Px 0 un .
""",
    ["fit/SxPx", "tidy/Px"],
    words="fit fits unfit unfits tidy untidy".split(),
    sugs="fitts unfitt tidyy".split(),
)

# --- compounding ----------------------------------------------------------

case(
    "compoundflag",
    """\
COMPOUNDFLAG c
COMPOUNDMIN 3
COMPOUNDWORDMAX 3
""",
    ["foo/c", "bar/c", "baz/c", "ab/c"],
    words="foo foobar foobarbaz foobarbazfoo abfoo ab abab".split(),
    sugs="foobarr fooba bazfooo".split(),
)

case(
    "compoundrule",
    """\
COMPOUNDRULE mn*t
COMPOUNDRULE ab?c
COMPOUNDMIN 1
""",
    ["one/m", "two/n", "three/t", "aa/a", "bb/b", "cc/c"],
    words="onetwothree onethree one two onetwotwothree aacc aabbcc aabb bbcc".split(),
    sugs="onetwothre aabcc onethre".split(),
)

case(
    "checkcompound",
    """\
COMPOUNDFLAG c
COMPOUNDMIN 2
CHECKCOMPOUNDPATTERN oo ba
CHECKCOMPOUNDDUP
CHECKCOMPOUNDTRIPLE
CHECKCOMPOUNDREP
CHECKCOMPOUNDCASE
REP oo o
""",
    ["foo/c", "bar/c", "buzz/c", "ll/c"],
    words="foo foobar foobuzz foofoo barbar llbar buzzbar".split(),
    sugs="foobuz barbuzz".split(),
)

case(
    "compoundmisc",
    """\
COMPOUNDRULE xy
COMPOUNDFLAG c
COMPOUNDFORBIDFLAG f
COMPOUNDPERMITFLAG p
COMPOUNDROOT r
COMPOUNDSYLMAX 3
COMPOUNDRULES 2
SYLLABLE aeiou
COMPOUNDMIN 2

SFX P Y 1
SFX P 0 s/p .
""",
    ["head/xc", "tail/yc", "stop/cf", "root/cr", "bit/cP"],
    words="head tail headtail stoptail headstop bits headbits roottail".split(),
    sugs="headtale headtai".split(),
)

# --- word flags -----------------------------------------------------------

# Every per-word flag that changes a verdict or a suggestion, in one
# case: the verdict table is the only thing that separates them.
case(
    "wordflags",
    """\
RARE R
BAD B
FORBIDDENWORD X
NOSUGGEST N
KEEPCASE K
NEEDAFFIX Z
NEEDCOMPOUND M
ONLYINCOMPOUND O
COMMON foo bar
COMPOUNDFLAG c

SFX A Y 1
SFX A 0 s .
""",
    [
        "foo",
        "bar",
        "seldom/R",
        "wrong/B",
        "banned/X",
        "rude/N",
        "McCoy/K",
        "stem/ZA",
        "half/Mc",
        "piece/Oc",
        "whole/c",
    ],
    words="foo bar seldom wrong banned rude McCoy mccoy MCCOY stem stems "
    "half halfwhole piece piecewhole whole".split(),
    sugs="seldon rued stemm mccoi".split(),
)

case(
    "circumfix",
    """\
CIRCUMFIX X

PFX A Y 1
PFX A 0 pre/X .

SFX B Y 2
SFX B 0 ing .
SFX B 0 ed/X .
""",
    ["view/AB"],
    words="view viewing viewed preview previewing previewed".split(),
    sugs="previewd viewwing".split(),
)

# --- suggestion tables ----------------------------------------------------

case(
    "rep",
    """\
TRY esianrtolcdugmphbyfvkwzESIANRTOLCDUGMPHBYFVKWZ'
REP 4
REP f ph
REP ph f
REP shun tion
REP ck k
""",
    ["phone", "fish", "nation", "back"],
    words="phone fone fish nation back bak".split(),
    sugs="fone phish nashun bak".split(),
)

case(
    "sal",
    """\
TRY esianrtolcdugmphbyfvkwz
SAL AH(AEIOUY)-^ *H
SAL AR(AEIOUY)-^ *R
SAL A^ *
SAL CK K
SAL PH F
SAL SCH- SK
SAL TH0 T
""",
    ["phone", "school", "nation", "think", "back"],
    words="phone fone school skool think tink back bak".split(),
    sugs="fone skool tink bak nashun".split(),
)

case(
    "sofo",
    """\
TRY esianrtolcdugmphbyfvkwz
SOFOFROM abcdefghijklmnopqrstuvwxyz
SOFOTO   ebctefghejklmnepkrstevwxyz
""",
    ["phone", "school", "nation", "think"],
    words="phone fone school skool think tink".split(),
    sugs="fone skool tink nashun".split(),
)

case(
    "repsal",
    """\
TRY esianrtolcdugmphbyfvkwz
SAL PH F
SAL CK K
REPSAL 2
REPSAL f ph
REPSAL k ck
""",
    ["phone", "back", "fish"],
    words="phone fone back bak fish".split(),
    sugs="fone bak phish".split(),
)

case(
    "map",
    """\
MAP 3
MAP aA
MAP oO
MAP eE
TRY aeiourtns
""",
    ["foo", "bar", "cafe"],
    words="foo bar cafe".split(),
    sugs="fOo bAr cafE fooo".split(),
)

case(
    "midword",
    """\
MIDWORD '-
""",
    ["don't", "well-known", "foo"],
    words="don't dont well-known wellknown foo".split(),
    sugs="dont wellknown".split(),
)

case(
    "sugopts",
    """\
NOSPLITSUGS
NOCOMPOUNDSUGS
COMPOUNDFLAG c
TRY esianrtolcdugmphbyfvkwz
SAL PH F
""",
    ["foo/c", "bar/c", "baz"],
    words="foo bar foobar baz".split(),
    sugs="foobarr fooba bazz".split(),
)

# --- the .sug reader at scale ---------------------------------------------

# Every other SAL case here compiles a handful of words, which leaves the
# .sug companion two to six entries long.  That is enough to prove the
# suggestion tree is read at all, but not enough to reach most of the code
# that reads it: the word numbers in a .sug line are stored as
# variable-length deltas, and a number under 127 fits the one-byte form,
# so a small corpus never leaves `bytes2offset`'s first branch and never
# makes `add_sound_suggest` walk more than a step or two of the
# case-folded tree counting words.
#
# So one case compiles a language big enough to matter: ~24,000 words
# under a sound-folding table lossy enough to collapse them onto ~1,000
# distinct soundfolds.  Measured against the generated file, that spans
# all three reachable branches (19,564 one-byte, 3,974 two-byte, 402
# three-byte encodings; the four-byte form needs a word number past 248
# million and no dictionary reaches it).  The words are generated rather
# than listed so the case costs a dozen lines: consonant-vowel-consonant
# syllables, plus every two-syllable compound of the first 150 of them.
# They compress to a 137-node tree, which is itself the point -- heavy
# node sharing is what makes the word-count walk in `add_sound_suggest`
# do real work.
_ONSETS = "b br d dr f fl g gr k kl l m n p pl r s sl sp st t tr v w".split()
_NUCLEI = "a e i o u".split()
_CODAS = ["", "b", "d", "g", "k", "l", "m", "n", "p", "r", "s", "t"]
_SYLLABLES = sorted({o + n + c for o in _ONSETS for n in _NUCLEI for c in _CODAS})
_SUGTREE_WORDS = sorted(
    set(_SYLLABLES) | {a + b for a in _SYLLABLES[:150] for b in _SYLLABLES[:150]}
)

# The misspellings are deliberately unreachable by editing: nothing in the
# dictionary is within four edits of "gzzkkl", so every suggestion for it
# comes from the soundfold pass, and deleting the .sug empties the list.
case(
    "sugtree",
    """\
TRY esianrtolcdugmphbyfvkwz
SAL CK K
SAL C K
SAL Q K
SAL X K
SAL PH F
SAL V F
SAL Z S
SAL D T
SAL P B
SAL G K
SAL M N
SAL R L
SAL A E
SAL I E
SAL O E
SAL U E
SAL Y E
""",
    _SUGTREE_WORDS,
    words="ba bad brat babrat brabbrat wut wutwut zzz stum stumstum".split(),
    sugs="brack flomp cknuzz dogglebop gzzkkl kwaddle mnwqxz phishnet "
    "quicksand sprocketing vulmpster wrackspurt zenophobe zzzqq".split(),
)

# NOSUGFILE suppresses the .sug companion even though SAL is present --
# the case exists to prove the SN_SUGFILE section is absent from .spl.
case(
    "nosugfile",
    """\
NOSUGFILE
TRY esianrtolcdugmphbyfvkwz
SAL PH F
SAL CK K
""",
    ["phone", "back"],
    words="phone fone back bak".split(),
    sugs="fone bak".split(),
)

case(
    "nobreak",
    """\
NOBREAK
""",
    ["foo", "bar", "baz"],
    words="foo foobar foobarbaz barbaz qux fooqux".split(),
    sugs="foobarr".split(),
)

# --- header/metadata sections --------------------------------------------

# SN_INFO is the only section built out of these; a case exists so the
# byte comparison covers it.
case(
    "info",
    """\
NAME Test Dictionary
VERSION 1.2.3
AUTHOR A. Tester
EMAIL tester@example.invalid
COPYRIGHT Public Domain
HOME https://example.invalid/
""",
    ["foo", "bar"],
)

case(
    "pfxpostpone",
    """\
PFXPOSTPONE
IGNOREEXTRA

PFX A Y 1
PFX A 0 un .

SFX B Y 1
SFX B 0 ed .
""",
    ["do/AB extra fields ignored", "tie/A"],
    words="do undo doed undoed tie untie".split(),
    sugs="undoo tiee".split(),
)

# --- encodings ------------------------------------------------------------

# FOL/LOW/UPP are the case tables the .spl SN_CHARFLAGS section is built
# from; they only apply to single-byte encodings.
case(
    "charflags",
    """\
SET ISO8859-1

FOL  àáâãäåæçèéêëìíîï
LOW  àáâãäåæçèéêëìíîï
UPP  ÀÁÂÃÄÅÆÇÈÉÊËÌÍÎÏ
""",
    ["café", "naïve", "élan", "foo"],
    words="café Café CAFÉ cafe naïve naive élan Élan foo".split(),
    sugs="cafe naive elan".split(),
    enc="latin1",
)

case(
    "utf8",
    """\
SET UTF-8

FOL  àéîõüšž
LOW  àéîõüšž
UPP  ÀÉÎÕÜŠŽ

SFX A Y 1
SFX A 0 s .
""",
    ["café/A", "škoda", "naïve", "中文"],
    words="café cafés Café CAFÉ cafe škoda Škoda skoda naïve 中文".split(),
    sugs="cafe skoda naive".split(),
)

# `-ascii` drops every word holding a non-ASCII character; the same
# input compiled twice is the clearest way to see that in the bytes.
case(
    "asciiflag",
    """\
SET UTF-8
""",
    ["foo", "café", "bar", "naïve"],
    words="foo café bar naïve cafe".split(),
    sugs="cafe fooo".split(),
    ascii="1",
)

# --- regions --------------------------------------------------------------

# Region support comes from passing several input basenames to one
# :mkspell.  The region name is the two characters after the `_` in each
# input's basename, so every input in a multi-region build has to carry
# one -- there is no unregioned base file to add them to.  The basename
# must also be at least five characters (`_xx` plus two), which is why
# these are `reg_us` and not `t_us`.
AFF_S = "SFX A Y 1\nSFX A 0 s .\n"
CASES["regions"] = {
    "aff": None,
    "dic": None,
    "words": "colour colours color colors shared lorry truck theater theatre".split(),
    "sugs": "colur lorrie theatr".split(),
    "extra": {
        "reg_us.aff": AFF_S,
        "reg_us.dic": "4\ncolor/A\nshared/A\ntruck\ntheater\n",
        "reg_ca.aff": AFF_S,
        "reg_ca.dic": "4\ncolour/A\nshared/A\nlorry\ntheatre\n",
    },
    "meta": {"inputs": "reg_us reg_ca"},
}

# --- plain wordlist -------------------------------------------------------

# An input with no .aff sibling is read as a plain word list: no count
# line, no flags, a `/` suffix for rare/region markers, and a different
# reader entirely (`spell_read_wordfile`).
CASES["wordlist"] = {
    "aff": None,
    "dic": None,
    "words": "apple banana cherry damson aple bannana rarity".split(),
    "sugs": "aple bannana cherrie".split(),
    "extra": {
        "t": "apple\nbanana\ncherry\ndamson\nelderberry\nrarity/?\n",
    },
    "meta": {},
}

# A .add file is the format `zg` appends to and `:mkspell` compiles into
# the companion .add.spl -- a third writer path, distinct from both the
# aff/dic build and the plain word list.
CASES["addfile"] = {
    "aff": None,
    "dic": None,
    "words": "gizmo widget doohickey gizmoo".split(),
    "sugs": "gizmoo widgit".split(),
    "extra": {"t.utf-8.add": "gizmo\nwidget\ndoohickey\n"},
    "meta": {"inputs": "t.utf-8.add"},
}


def write(root: Path) -> None:
    if root.exists():
        shutil.rmtree(root)
    root.mkdir(parents=True)
    for name, spec in sorted(CASES.items()):
        d = root / name
        d.mkdir()
        enc = spec["meta"].get("enc", "utf8")
        codec = "latin-1" if enc == "latin1" else "utf-8"
        if spec["aff"] is not None:
            (d / "t.aff").write_text(spec["aff"], encoding=codec)
        if spec["dic"] is not None:
            body = "\n".join(spec["dic"])
            (d / "t.dic").write_text(f"{len(spec['dic'])}\n{body}\n", encoding=codec)
        for fname, text in spec["extra"].items():
            (d / fname).write_text(text, encoding=codec)
        (d / "words").write_text("\n".join(spec["words"]) + "\n", encoding=codec)
        (d / "sugs").write_text("\n".join(spec["sugs"]) + "\n", encoding=codec)
        (d / "meta").write_text(
            "".join(f"{k}={v}\n" for k, v in sorted(spec["meta"].items())),
            encoding="utf-8",
        )
    print(f"{root}: {len(CASES)} cases")


if __name__ == "__main__":
    write(Path(sys.argv[1]))
