#!/usr/bin/env python3
"""Generate src/nvim/utf8proc/tables.rs from a utf8proc source tree.

Usage: gen.py <utf8proc-src-dir> <output.rs>

Translates the four tables the in-tree port consumes (utf8proc_sequences,
utf8proc_stage1table, utf8proc_stage2table, utf8proc_properties) from
utf8proc_data.c into Rust statics. Enum names appearing in the property
initializers are resolved to their numeric values by scanning utf8proc.h.
The combination tables (utf8proc_combinations_*) are canonical-composition
data the port does not use and are deliberately skipped.

Upstream utf8proc_data.c is itself generated (data/data_generator.jl over
the Unicode data files), so this is a translation of generated data, not of
source; regenerate by pointing this script at a newer utf8proc release.
Run rustfmt on the output before committing.
"""

import re
import sys
from pathlib import Path

# Field order of a utf8proc_property_t initializer in utf8proc_data.c,
# with the Rust type each field gets. `pad` exists only to fill the C
# bitfield and is dropped.
FIELDS = [
    ("category", "i16"),
    ("combining_class", "i16"),
    ("bidi_class", "i16"),
    ("decomp_type", "i16"),
    ("decomp_seqindex", "u16"),
    ("casefold_seqindex", "u16"),
    ("uppercase_seqindex", "u16"),
    ("lowercase_seqindex", "u16"),
    ("titlecase_seqindex", "u16"),
    ("comb_index", "u16"),
    ("comb_length", "u8"),
    ("comb_issecond", "bool"),
    ("bidi_mirrored", "bool"),
    ("comp_exclusion", "bool"),
    ("ignorable", "bool"),
    ("control_boundary", "bool"),
    ("charwidth", "u8"),
    ("ambiguous_width", "bool"),
    ("pad", "drop"),
    ("boundclass", "u8"),
    ("indic_conjunct_break", "u8"),
]


def parse_enums(header: str) -> dict[str, int]:
    """Every UTF8PROC_* enum constant in utf8proc.h carries an explicit
    value, so a flat scan is sufficient."""
    values = {}
    for name, value in re.findall(
        r"\b(UTF8PROC_[A-Z0-9_]+)\s*=\s*(-?(?:0x[0-9A-Fa-f]+|\d+))", header
    ):
        values[name] = int(value, 0)
    return values


def array_body(data: str, name: str) -> str:
    m = re.search(
        rf"static const \w+ {re.escape(name)}\[\] = \{{(.*?)\}};", data, re.DOTALL
    )
    if not m:
        sys.exit(f"array {name} not found")
    return m.group(1)


def int_array(data: str, name: str) -> list[int]:
    return [
        int(tok, 0)
        for tok in array_body(data, name).replace("\n", " ").split(",")
        if tok.strip()
    ]


def parse_properties(data: str, enums: dict[str, int]) -> list[list]:
    entries = []
    for raw in re.findall(r"\{([^}]*)\}", array_body(data, "utf8proc_properties")):
        tokens = [t.strip() for t in raw.split(",")]
        if len(tokens) != len(FIELDS):
            sys.exit(
                f"property entry has {len(tokens)} fields, expected {len(FIELDS)}: {raw}"
            )
        entry = []
        for tok, (field, ty) in zip(tokens, FIELDS):
            if ty == "drop":
                continue
            if tok == "UINT16_MAX":
                value = 0xFFFF
            elif tok in ("false", "true"):
                value = tok == "true"
            elif tok in enums:
                value = enums[tok]
            else:
                value = int(tok, 0)
            if ty == "bool" and not isinstance(value, bool):
                if value not in (0, 1):
                    sys.exit(f"non-boolean value {value} for {field}")
                value = bool(value)
            entry.append(value)
        entries.append(entry)
    return entries


def emit_ints(out, name: str, ty: str, values: list[int]):
    out.write(
        f"#[rustfmt::skip]\npub(super) static {name}: [{ty}; {len(values)}] = [\n"
    )
    for i in range(0, len(values), 16):
        out.write("    " + ", ".join(str(v) for v in values[i : i + 16]) + ",\n")
    out.write("];\n\n")


def main():
    if len(sys.argv) != 3:
        sys.exit(__doc__)
    srcdir = Path(sys.argv[1])
    enums = parse_enums((srcdir / "utf8proc.h").read_text())
    data = (srcdir / "utf8proc_data.c").read_text()

    unicode_version = re.search(
        r'utf8proc_unicode_version\(void\) \{\s*return "([^"]+)"',
        (srcdir / "utf8proc.c").read_text(),
    ).group(1)
    utf8proc_version = ".".join(
        re.search(
            rf"#define UTF8PROC_VERSION_{part} (\d+)",
            (srcdir / "utf8proc.h").read_text(),
        ).group(1)
        for part in ("MAJOR", "MINOR", "PATCH")
    )

    sequences = int_array(data, "utf8proc_sequences")
    stage1 = int_array(data, "utf8proc_stage1table")
    stage2 = int_array(data, "utf8proc_stage2table")
    properties = parse_properties(data, enums)

    with open(sys.argv[2], "w") as out:
        out.write(f"""\
//! Unicode {unicode_version} character property tables, translated from
//! utf8proc v{utf8proc_version}'s `utf8proc_data.c` by `tools/utf8proc-tables/gen.py`.
//! Generated file -- do not edit by hand.
//!
//! The data derives from the Unicode data files, Copyright (c) 1991-2007
//! Unicode, Inc., distributed under the Unicode Terms of Use; utf8proc's
//! processing of them is MIT-licensed. Both notices are reproduced in
//! licenses/utf8proc-LICENSE.md.

use super::utf8proc_property_t;

/// Compact constructor keeping the generated entries to one line each; the
/// argument order is the field order of the C initializers.
#[rustfmt::skip]
#[allow(clippy::too_many_arguments, clippy::fn_params_excessive_bools)]
const fn p(category: i16, combining_class: i16, bidi_class: i16, decomp_type: i16,
           decomp_seqindex: u16, casefold_seqindex: u16, uppercase_seqindex: u16,
           lowercase_seqindex: u16, titlecase_seqindex: u16, comb_index: u16,
           comb_length: u8, comb_issecond: bool, bidi_mirrored: bool,
           comp_exclusion: bool, ignorable: bool, control_boundary: bool,
           charwidth: u8, ambiguous_width: bool, boundclass: u8,
           indic_conjunct_break: u8) -> utf8proc_property_t {{
    utf8proc_property_t {{
        category, combining_class, bidi_class, decomp_type, decomp_seqindex,
        casefold_seqindex, uppercase_seqindex, lowercase_seqindex,
        titlecase_seqindex, comb_index, comb_length, comb_issecond,
        bidi_mirrored, comp_exclusion, ignorable, control_boundary, charwidth,
        ambiguous_width, boundclass, indic_conjunct_break,
    }}
}}

""")
        emit_ints(out, "SEQUENCES", "u16", sequences)
        emit_ints(out, "STAGE1", "u16", stage1)
        emit_ints(out, "STAGE2", "u16", stage2)
        out.write(
            f"#[rustfmt::skip]\npub(super) static PROPERTIES: [utf8proc_property_t; {len(properties)}] = [\n"
        )
        for entry in properties:
            args = ", ".join(
                "true" if v is True else "false" if v is False else str(v)
                for v in entry
            )
            out.write(f"    p({args}),\n")
        out.write("];\n")


if __name__ == "__main__":
    main()
