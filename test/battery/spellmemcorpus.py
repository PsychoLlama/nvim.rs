#!/usr/bin/env python3
"""Generate the Hunspell input the `spellmem` row compiles.

    ./spellmemcorpus.py <outdir>

Writes `sal.aff`/`sal.dic` and `nosal.aff`/`nosal.dic` under <outdir>:
the same 20,000 made-up words (fixed seed, so every run writes the same
bytes) with S/D suffix and G prefix flags, REP items, and -- in `sal`
only -- SAL rules, which is what makes `:mkspell` write a .sug file and
run the soundfold pass over the trees.  The words are syllable soup with
a few capitalised, so both the case-folded and the keep-case tree grow.
"""

import random
import sys
from pathlib import Path

N_WORDS = 20000
SYL = [
    "ka",
    "ta",
    "ra",
    "mi",
    "no",
    "su",
    "be",
    "lo",
    "ve",
    "qui",
    "dra",
    "sto",
    "pen",
    "gor",
    "lim",
    "ash",
    "eth",
    "ung",
    "or",
    "el",
    "an",
    "ix",
    "um",
    "ez",
    "ou",
    "ch",
    "sh",
    "th",
    "ph",
    "é",
    "ö",
    "ñ",
]
FLAGS = ["", "", "", "/S", "/D", "/SD", "/G"]
AFF = (
    "SET UTF-8\nTRY esianrtolcdugmphbyfvkwzéöñ\n"
    "SFX S Y 2\nSFX S 0 s [^s]\nSFX S 0 es s\n"
    "SFX D Y 2\nSFX D 0 ed [^e]\nSFX D 0 d e\n"
    "PFX G Y 1\nPFX G 0 re .\n"
    "REP 2\nREP ph f\nREP k qu\n"
)
SAL = "SAL followup true\nSAL collapse_result true\nSAL PH F\nSAL K K\nSAL Q K\n"


def main() -> None:
    out = Path(sys.argv[1])
    out.mkdir(parents=True, exist_ok=True)
    rng = random.Random(37)
    words: set[str] = set()
    while len(words) < N_WORDS:
        w = "".join(rng.choice(SYL) for _ in range(rng.randint(2, 5)))
        if rng.random() < 0.05:
            w = w.capitalize()
        words.add(w)
    dic = [f"{len(words)}\n"] + [w + rng.choice(FLAGS) + "\n" for w in sorted(words)]
    for name, aff in (("sal", AFF + SAL), ("nosal", AFF)):
        (out / f"{name}.dic").write_text("".join(dic), encoding="utf-8")
        (out / f"{name}.aff").write_text(aff, encoding="utf-8")


main()
