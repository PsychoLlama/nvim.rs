//! The character classes the match loop tests one input character against:
//! `\i`, `\w`, `\d` and the rest, in their `\I`-style negated forms too.
//!
//! Each is a pure test on the character at the input, so they answer `bool`
//! and the loop decides what to do about it.

#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use crate::mbyte::Codepoint;
use crate::regexp::NfaOp;
use core::ffi::c_int;

use crate::ascii::{ascii_isdigit, ascii_iswhite};
use crate::charset::{vim_is_ident_char, vim_isfilec, vim_isprintc};
use crate::regexp::{ByteClass, RI_FLAGS, Rex};

/// Is `c` in the Latin-1 class `flag` names? The table only covers the
/// first 256 code points, so anything above them is out by construction.
fn ri(c: c_int, flag: ByteClass) -> bool {
    usize::try_from(c).is_ok_and(|c| c < 0x100 && RI_FLAGS[c].has(flag))
}

/// The character at the input, which for the keyword and file-name classes
/// has to be read from the text rather than taken as a code point: they are
/// buffer-option-driven and look at the bytes.
fn input(rex: Rex) -> c_int {
    rex.char_here()
}

fn is_word_char(rex: Rex) -> bool {
    rex.iswordp()
}

/// Whether the character at the input is printable. Its own function, as
/// the twin of [`is_word_char`], because inlining it into [`class_matches`]
/// costs the NFA's class states: measured +0.13 % on rebench's `nfa` phase
/// and +0.21 % on `search` with the two `vim_isprintc` calls in place.
fn is_printable(rex: Rex) -> bool {
    vim_isprintc(Codepoint::new(input(rex)))
}

fn ignoring_case(rex: Rex) -> bool {
    rex.reg_ic()
}

/// Does the class opcode `op` accept the character `curc` at the input?
///
/// The negated forms all also require a character to be there: at the end of
/// a line there is nothing for `\I` to match.
// Out of line: inlined into the match loop, its tests of `curc` are hoisted
// into the per-character prologue and paid for at every character, class
// state or not.
#[inline(never)]
pub(crate) fn class_matches(rex: Rex, op: NfaOp, curc: c_int) -> bool {
    match op {
        NfaOp::Ident => vim_is_ident_char(Codepoint::new(curc)),
        NfaOp::Sident => !ascii_isdigit(curc) && vim_is_ident_char(Codepoint::new(curc)),
        NfaOp::Kword => is_word_char(rex),
        NfaOp::Skword => !ascii_isdigit(curc) && is_word_char(rex),
        NfaOp::Fname => vim_isfilec(Codepoint::new(curc)),
        NfaOp::Sfname => !ascii_isdigit(curc) && vim_isfilec(Codepoint::new(curc)),
        NfaOp::Print => is_printable(rex),
        NfaOp::Sprint => !ascii_isdigit(curc) && is_printable(rex),
        NfaOp::White => ascii_iswhite(curc),
        NfaOp::Nwhite => curc != 0 && !ascii_iswhite(curc),
        NfaOp::Digit => ri(curc, ByteClass::DIGIT),
        NfaOp::Ndigit => curc != 0 && !ri(curc, ByteClass::DIGIT),
        NfaOp::Hex => ri(curc, ByteClass::HEX),
        NfaOp::Nhex => curc != 0 && !ri(curc, ByteClass::HEX),
        NfaOp::Octal => ri(curc, ByteClass::OCTAL),
        NfaOp::Noctal => curc != 0 && !ri(curc, ByteClass::OCTAL),
        NfaOp::Word => ri(curc, ByteClass::WORD),
        NfaOp::Nword => curc != 0 && !ri(curc, ByteClass::WORD),
        NfaOp::Head => ri(curc, ByteClass::HEAD),
        NfaOp::Nhead => curc != 0 && !ri(curc, ByteClass::HEAD),
        NfaOp::Alpha => ri(curc, ByteClass::ALPHA),
        NfaOp::Nalpha => curc != 0 && !ri(curc, ByteClass::ALPHA),
        NfaOp::Lower => ri(curc, ByteClass::LOWER),
        NfaOp::Nlower => curc != 0 && !ri(curc, ByteClass::LOWER),
        NfaOp::Upper => ri(curc, ByteClass::UPPER),
        NfaOp::Nupper => curc != 0 && !ri(curc, ByteClass::UPPER),
        // The `_IC` forms are what a `[a-z]` collection recognised as a
        // class compiles to, and they honour 'ignorecase' where the bare
        // `\l`/`\u` do not.
        NfaOp::LowerIc => lower_ic(rex, curc),
        NfaOp::NlowerIc => curc != 0 && !lower_ic(rex, curc),
        NfaOp::UpperIc => upper_ic(rex, curc),
        NfaOp::NupperIc => curc != 0 && !upper_ic(rex, curc),
        _ => false,
    }
}

fn lower_ic(rex: Rex, curc: c_int) -> bool {
    ri(curc, ByteClass::LOWER) || (ignoring_case(rex) && ri(curc, ByteClass::UPPER))
}

fn upper_ic(rex: Rex, curc: c_int) -> bool {
    ri(curc, ByteClass::UPPER) || (ignoring_case(rex) && ri(curc, ByteClass::LOWER))
}
