//! The character classes the match loop tests one input character against:
//! `\i`, `\w`, `\d` and the rest, in their `\I`-style negated forms too.
//!
//! Each is a pure test on the character at the input, so they answer `bool`
//! and the loop decides what to do about it.

#![deny(unsafe_op_in_unsafe_fn)]

use core::ffi::{c_char, c_int};

use crate::src::nvim::ascii::{ascii_isdigit, ascii_iswhite};
use crate::src::nvim::charset::{vim_isIDc, vim_isfilec, vim_isprintc, vim_iswordp_buf};
use crate::src::nvim::mbyte::utf_ptr2char;
use crate::src::nvim::regexp::{
    NFA_ALPHA, NFA_DIGIT, NFA_FNAME, NFA_HEAD, NFA_HEX, NFA_IDENT, NFA_KWORD, NFA_LOWER,
    NFA_LOWER_IC, NFA_NALPHA, NFA_NDIGIT, NFA_NHEAD, NFA_NHEX, NFA_NLOWER, NFA_NLOWER_IC,
    NFA_NOCTAL, NFA_NUPPER, NFA_NUPPER_IC, NFA_NWHITE, NFA_NWORD, NFA_OCTAL, NFA_PRINT, NFA_SFNAME,
    NFA_SIDENT, NFA_SKWORD, NFA_SPRINT, NFA_UPPER, NFA_UPPER_IC, NFA_WHITE, NFA_WORD, NUL,
    RI_ALPHA, RI_DIGIT, RI_FLAGS, RI_HEAD, RI_HEX, RI_LOWER, RI_OCTAL, RI_UPPER, RI_WORD, rex,
};

/// Is `c` in the Latin-1 class `flag` names? The table only covers the
/// first 256 code points, so anything above them is out by construction.
fn ri(c: c_int, flag: c_int) -> bool {
    (0..0x100).contains(&c) && RI_FLAGS[c as usize] as c_int & flag != 0
}

/// The character at the input, which for the keyword and file-name classes
/// has to be read from the text rather than taken as a code point: they are
/// buffer-option-driven and look at the bytes.
fn input() -> *mut c_char {
    // SAFETY: `rex.input` points into the line being matched.
    unsafe { (*rex.ptr()).input as *mut c_char }
}

fn is_word_char() -> bool {
    // SAFETY: `reg_buf` is the buffer whose 'iskeyword' applies.
    unsafe { vim_iswordp_buf(input(), (*rex.ptr()).reg_buf) }
}

fn is_printable() -> bool {
    // SAFETY: as `input`.
    unsafe { vim_isprintc(utf_ptr2char(input())) }
}

fn ignoring_case() -> bool {
    // SAFETY: reads the match context.
    unsafe { (*rex.ptr()).reg_ic }
}

/// `vim_isIDc` and `vim_isfilec` are pure tests on a code point that read
/// only option state.
fn is_ident_char(c: c_int) -> bool {
    // SAFETY: a pure test on a code point.
    unsafe { vim_isIDc(c) }
}

fn is_file_char(c: c_int) -> bool {
    // SAFETY: as `is_ident_char`.
    unsafe { vim_isfilec(c) }
}

/// Does the class opcode `c` accept the character `curc` at the input?
///
/// The negated forms all also require a character to be there: at the end of
/// a line there is nothing for `\I` to match.
pub(crate) fn class_matches(c: c_int, curc: c_int) -> bool {
    match c {
        NFA_IDENT => is_ident_char(curc),
        NFA_SIDENT => !ascii_isdigit(curc) && is_ident_char(curc),
        NFA_KWORD => is_word_char(),
        NFA_SKWORD => !ascii_isdigit(curc) && is_word_char(),
        NFA_FNAME => is_file_char(curc),
        NFA_SFNAME => !ascii_isdigit(curc) && is_file_char(curc),
        NFA_PRINT => is_printable(),
        NFA_SPRINT => !ascii_isdigit(curc) && is_printable(),
        NFA_WHITE => ascii_iswhite(curc),
        NFA_NWHITE => curc != NUL && !ascii_iswhite(curc),
        NFA_DIGIT => ri(curc, RI_DIGIT),
        NFA_NDIGIT => curc != NUL && !ri(curc, RI_DIGIT),
        NFA_HEX => ri(curc, RI_HEX),
        NFA_NHEX => curc != NUL && !ri(curc, RI_HEX),
        NFA_OCTAL => ri(curc, RI_OCTAL),
        NFA_NOCTAL => curc != NUL && !ri(curc, RI_OCTAL),
        NFA_WORD => ri(curc, RI_WORD),
        NFA_NWORD => curc != NUL && !ri(curc, RI_WORD),
        NFA_HEAD => ri(curc, RI_HEAD),
        NFA_NHEAD => curc != NUL && !ri(curc, RI_HEAD),
        NFA_ALPHA => ri(curc, RI_ALPHA),
        NFA_NALPHA => curc != NUL && !ri(curc, RI_ALPHA),
        NFA_LOWER => ri(curc, RI_LOWER),
        NFA_NLOWER => curc != NUL && !ri(curc, RI_LOWER),
        NFA_UPPER => ri(curc, RI_UPPER),
        NFA_NUPPER => curc != NUL && !ri(curc, RI_UPPER),
        // The `_IC` forms are what a `[a-z]` collection recognised as a
        // class compiles to, and they honour 'ignorecase' where the bare
        // `\l`/`\u` do not.
        NFA_LOWER_IC => lower_ic(curc),
        NFA_NLOWER_IC => curc != NUL && !lower_ic(curc),
        NFA_UPPER_IC => upper_ic(curc),
        NFA_NUPPER_IC => curc != NUL && !upper_ic(curc),
        _ => false,
    }
}

fn lower_ic(curc: c_int) -> bool {
    ri(curc, RI_LOWER) || (ignoring_case() && ri(curc, RI_UPPER))
}

fn upper_ic(curc: c_int) -> bool {
    ri(curc, RI_UPPER) || (ignoring_case() && ri(curc, RI_LOWER))
}
