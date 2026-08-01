//! The cterm colour names, per `'t_Co'`.
//!
//! `ctermfg=`/`ctermbg=` accept a small set of names rather than the RGB
//! table, and which number each one means depends on how many colours the
//! terminal claims ([`lookup_color`]). The 8-colour case also has to fake
//! the light half by setting `bold`.

#![forbid(unsafe_code)]

use core::ffi::{CStr, c_int};

use crate::src::nvim::main::t_colors;
use crate::src::nvim::types::TriState;

use super::{kFalse, kNone, kTrue};

/// The names `ctermfg=`/`ctermbg=` accept, in the order the number tables
/// below are indexed. `NONE` is last and maps to -1 in every table.
static COLOR_NAMES: [&CStr; 28] = [
    c"Black",
    c"DarkBlue",
    c"DarkGreen",
    c"DarkCyan",
    c"DarkRed",
    c"DarkMagenta",
    c"Brown",
    c"DarkYellow",
    c"Gray",
    c"Grey",
    c"LightGray",
    c"LightGrey",
    c"DarkGray",
    c"DarkGrey",
    c"Blue",
    c"LightBlue",
    c"Green",
    c"LightGreen",
    c"Cyan",
    c"LightCyan",
    c"Red",
    c"LightRed",
    c"Magenta",
    c"LightMagenta",
    c"Yellow",
    c"LightYellow",
    c"White",
    c"NONE",
];

/// The number each name means on a 16-colour terminal. Also the validity
/// test: a name whose entry here is negative is not a colour at all.
static COLOR_NUMBERS_16: [c_int; 28] = [
    0, 1, 2, 3, 4, 5, 6, 6, 7, 7, 7, 7, 8, 8, 9, 9, 10, 10, 11, 11, 12, 12, 13, 13, 14, 14, 15, -1,
];

/// xterm with 88 colours.
static COLOR_NUMBERS_88: [c_int; 28] = [
    0, 4, 2, 6, 1, 5, 32, 72, 84, 84, 7, 7, 82, 82, 12, 43, 10, 61, 14, 63, 9, 74, 13, 75, 11, 78,
    15, -1,
];

/// xterm with 256 colours.
static COLOR_NUMBERS_256: [c_int; 28] = [
    0, 4, 2, 6, 1, 5, 130, 3, 248, 248, 7, 7, 242, 242, 12, 81, 10, 121, 14, 159, 9, 224, 13, 225,
    11, 229, 15, -1,
];

/// Fewer than 16 colours: the light half of the palette is the dark half
/// with bit 3 set (8, 12, 10, 14, 9, 13, 11, 15), which [`lookup_color`]
/// turns into `bold` plus the dark colour.
static COLOR_NUMBERS_8: [c_int; 28] = [
    0, 4, 2, 6, 1, 5, 3, 3, 7, 7, 7, 7, 8, 8, 12, 12, 10, 10, 14, 14, 9, 9, 13, 13, 11, 11, 15, -1,
];

/// The index in [`COLOR_NAMES`] of a cterm colour name, case-insensitively.
///
/// Upstream compares the first byte against the uppercased first byte of the
/// argument before calling `STRICMP` on the rest, "to reduce calls to
/// STRICMP, it can be slow" — every table entry starts with an uppercase
/// letter, so that is the same test as folding both sides.
pub(crate) fn cterm_color_index(name: &CStr) -> Option<usize> {
    let name = name.to_bytes();
    let (&first, rest) = name.split_first()?;
    let first = first.to_ascii_uppercase();
    // Reverse order, as upstream; every name is distinct case-insensitively,
    // so the direction cannot change which one is found.
    (0..COLOR_NAMES.len()).rev().find(|&i| {
        let entry = COLOR_NAMES[i].to_bytes();
        entry[0] == first && entry[1..].eq_ignore_ascii_case(rest)
    })
}

/// The cterm number for [`COLOR_NAMES`]`[idx]` on this terminal, and whether
/// `bold` has to be turned on or off to get it.
///
/// Answers -1 for `NONE`, the one entry that is not a colour. `bold` is only
/// decided for a foreground colour on an 8-colour terminal, where the light
/// half of the palette is reached by making the dark half bold; every other
/// case leaves it [`kNone`], meaning "don't touch it".
pub(crate) fn lookup_color(idx: usize, foreground: bool) -> (c_int, TriState) {
    // The _16 table doubles as the validity check.
    if COLOR_NUMBERS_16[idx] < 0 {
        return (-1, kNone);
    }
    match t_colors.get() {
        8 => {
            let color = COLOR_NUMBERS_8[idx];
            let bold = if !foreground {
                kNone
            } else if color & 8 != 0 {
                kTrue
            } else {
                kFalse
            };
            (color & 7, bold)
        }
        16 => (COLOR_NUMBERS_8[idx], kNone),
        88 => (COLOR_NUMBERS_88[idx], kNone),
        n if n >= 256 => (COLOR_NUMBERS_256[idx], kNone),
        _ => (COLOR_NUMBERS_16[idx], kNone),
    }
}

/// The cterm number a colour name means, or -1 if it is not one.
pub fn name_to_ctermcolor(name: &CStr) -> c_int {
    match cterm_color_index(name) {
        Some(idx) => lookup_color(idx, false).0,
        None => -1,
    }
}
