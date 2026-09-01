//! The character classes the match loop tests one input character against:
//! `\i`, `\w`, `\d` and the rest, in their `\I`-style negated forms too.
//!
//! Each is a pure test on the character at the input, so they answer `bool`
//! and the loop decides what to do about it.

#![deny(unsafe_op_in_unsafe_fn)]

use crate::regexp::NfaOp;
use core::ffi::{c_char, c_int};

use crate::ascii::{ascii_isdigit, ascii_iswhite};
use crate::charset::{vim_is_ident_char, vim_isfilec, vim_isprintc};
use crate::mbyte::utf_ptr2char;
use crate::regexp::{
    RI_ALPHA, RI_DIGIT, RI_FLAGS, RI_HEAD, RI_HEX, RI_LOWER, RI_OCTAL, RI_UPPER, RI_WORD, Rex,
};
use crate::types::NUL;

/// Is `c` in the Latin-1 class `flag` names? The table only covers the
/// first 256 code points, so anything above them is out by construction.
fn ri(c: c_int, flag: c_int) -> bool {
    (0..0x100).contains(&c) && RI_FLAGS[c as usize] as c_int & flag != 0
}

/// The character at the input, which for the keyword and file-name classes
/// has to be read from the text rather than taken as a code point: they are
/// buffer-option-driven and look at the bytes.
fn input(rex: Rex) -> *mut c_char {
    rex.input_str()
}

fn is_word_char(rex: Rex) -> bool {
    rex.iswordp()
}

fn is_printable(rex: Rex) -> bool {
    // SAFETY: the cursor is inside the line being matched.
    unsafe { vim_isprintc(utf_ptr2char(input(rex))) }
}

fn ignoring_case(rex: Rex) -> bool {
    rex.reg_ic()
}

/// `vim_is_ident_char` and `vim_isfilec` are pure tests on a code point that read
/// only option state.
fn is_ident_char(c: c_int) -> bool {
    // SAFETY: a pure test on a code point.
    unsafe { vim_is_ident_char(c) }
}

fn is_file_char(c: c_int) -> bool {
    // SAFETY: as `is_ident_char`.
    unsafe { vim_isfilec(c) }
}

/// Does the class opcode `op` accept the character `curc` at the input?
///
/// The negated forms all also require a character to be there: at the end of
/// a line there is nothing for `\I` to match.
pub(crate) fn class_matches(rex: Rex, op: NfaOp, curc: c_int) -> bool {
    match op {
        NfaOp::Ident => is_ident_char(curc),
        NfaOp::Sident => !ascii_isdigit(curc) && is_ident_char(curc),
        NfaOp::Kword => is_word_char(rex),
        NfaOp::Skword => !ascii_isdigit(curc) && is_word_char(rex),
        NfaOp::Fname => is_file_char(curc),
        NfaOp::Sfname => !ascii_isdigit(curc) && is_file_char(curc),
        NfaOp::Print => is_printable(rex),
        NfaOp::Sprint => !ascii_isdigit(curc) && is_printable(rex),
        NfaOp::White => ascii_iswhite(curc),
        NfaOp::Nwhite => curc != NUL && !ascii_iswhite(curc),
        NfaOp::Digit => ri(curc, RI_DIGIT),
        NfaOp::Ndigit => curc != NUL && !ri(curc, RI_DIGIT),
        NfaOp::Hex => ri(curc, RI_HEX),
        NfaOp::Nhex => curc != NUL && !ri(curc, RI_HEX),
        NfaOp::Octal => ri(curc, RI_OCTAL),
        NfaOp::Noctal => curc != NUL && !ri(curc, RI_OCTAL),
        NfaOp::Word => ri(curc, RI_WORD),
        NfaOp::Nword => curc != NUL && !ri(curc, RI_WORD),
        NfaOp::Head => ri(curc, RI_HEAD),
        NfaOp::Nhead => curc != NUL && !ri(curc, RI_HEAD),
        NfaOp::Alpha => ri(curc, RI_ALPHA),
        NfaOp::Nalpha => curc != NUL && !ri(curc, RI_ALPHA),
        NfaOp::Lower => ri(curc, RI_LOWER),
        NfaOp::Nlower => curc != NUL && !ri(curc, RI_LOWER),
        NfaOp::Upper => ri(curc, RI_UPPER),
        NfaOp::Nupper => curc != NUL && !ri(curc, RI_UPPER),
        // The `_IC` forms are what a `[a-z]` collection recognised as a
        // class compiles to, and they honour 'ignorecase' where the bare
        // `\l`/`\u` do not.
        NfaOp::LowerIc => lower_ic(rex, curc),
        NfaOp::NlowerIc => curc != NUL && !lower_ic(rex, curc),
        NfaOp::UpperIc => upper_ic(rex, curc),
        NfaOp::NupperIc => curc != NUL && !upper_ic(rex, curc),
        _ => false,
    }
}

fn lower_ic(rex: Rex, curc: c_int) -> bool {
    ri(curc, RI_LOWER) || (ignoring_case(rex) && ri(curc, RI_UPPER))
}

fn upper_ic(rex: Rex, curc: c_int) -> bool {
    ri(curc, RI_UPPER) || (ignoring_case(rex) && ri(curc, RI_LOWER))
}
