//! Setting up a compile and reading back what the finished machine implies:
//! whether it is anchored, what character it must start with, and any
//! literal text the whole pattern reduces to.
//!
//! The last three are the shortcuts `nfa_regexec_both` uses to skip ahead in
//! the line without running the machine at all, so each walks the states
//! conservatively and gives up — returning "no shortcut" — the moment it
//! meets anything it cannot reason about.

#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use crate::cstr;
use crate::regexp::NfaOp;
use core::ffi::c_int;

use super::program::{NfaState, StateId};
use crate::mbyte::encode_char;
use crate::types::MB_MAXBYTES;

/// How far the walks below follow a `NFA_SPLIT` before giving up.
const MAX_DEPTH: c_int = 4;

/// The capture brackets and `\zs`/`\ze` markers: they consume no input, so a
/// walk looking for the first *character* steps straight through them.
fn is_bracket(c: NfaOp) -> bool {
    matches!(c, NfaOp::Zstart | NfaOp::Zend | NfaOp::Nopen) || c.opens_capture()
}

/// Must every match start at the beginning of a line?
pub(crate) fn nfa_get_reganch(states: &[NfaState], start: StateId, depth: c_int) -> bool {
    if depth > MAX_DEPTH {
        return false;
    }
    let mut p = start;
    while !p.is_none() {
        let s = &states[p.index()];
        match NfaOp::try_from(s.c) {
            Ok(NfaOp::Bol | NfaOp::Bof) => return true,
            // Zero-width, so the anchor is whatever follows. Note that
            // `\%23l` and its neighbours are *not* here: unlike
            // `nfa_get_regstart` below, a position assertion stops this
            // walk.
            Ok(NfaOp::Cursor | NfaOp::Visual) => p = s.out,
            Ok(c) if is_bracket(c) => p = s.out,
            // Anchored only if both alternatives are.
            Ok(NfaOp::Split) => {
                return nfa_get_reganch(states, s.out, depth + 1)
                    && nfa_get_reganch(states, s.out1, depth + 1);
            }
            _ => return false,
        }
    }
    false
}

/// The character every match must start with, or 0 if there is no single
/// such character.
pub(crate) fn nfa_get_regstart(states: &[NfaState], start: StateId, depth: c_int) -> c_int {
    if depth > MAX_DEPTH {
        return 0;
    }
    let mut p = start;
    while !p.is_none() {
        let s = &states[p.index()];
        let code = s.c;
        match NfaOp::try_from(code) {
            // Zero-width: the start character is whatever follows.
            // `Cursor`..=`Visual` is the whole position-assertion block,
            // `\%#` through `\%V`.
            Ok(NfaOp::Bol | NfaOp::Bof | NfaOp::Bow | NfaOp::Eow) => p = s.out,
            Ok(
                NfaOp::Cursor
                | NfaOp::Lnum
                | NfaOp::LnumGt
                | NfaOp::LnumLt
                | NfaOp::Col
                | NfaOp::ColGt
                | NfaOp::ColLt
                | NfaOp::Vcol
                | NfaOp::VcolGt
                | NfaOp::VcolLt
                | NfaOp::Mark
                | NfaOp::MarkGt
                | NfaOp::MarkLt
                | NfaOp::Visual,
            ) => p = s.out,
            Ok(c) if is_bracket(c) => p = s.out,
            // Only if both alternatives agree.
            Ok(NfaOp::Split) => {
                let c1 = nfa_get_regstart(states, s.out, depth + 1);
                let c2 = nfa_get_regstart(states, s.out1, depth + 1);
                return if c1 == c2 { c1 } else { 0 };
            }
            // A literal character is one; any other opcode is not.
            _ => return if code > 0 { code } else { 0 },
        }
    }
    0
}

/// The literal text the whole pattern matches after its first character
/// (which the program reports as `regstart`), NUL-terminated, or `None`
/// when the pattern is not one plain run of characters.
pub(crate) fn nfa_get_match_text(states: &[NfaState], start: StateId) -> Option<Vec<u8>> {
    let state = |id: StateId| &states[id.index()];
    if state(start).c != NfaOp::Mopen.code() {
        return None;
    }
    let mut p = state(start).out;
    while state(p).c > 0 {
        p = state(p).out;
    }
    if state(p).c != NfaOp::Mclose.code() || state(state(p).out).c != NfaOp::Match.code() {
        return None;
    }
    let mut text = Vec::new();
    let mut buf = [0u8; MB_MAXBYTES];
    let mut p = state(state(start).out).out;
    while state(p).c > 0 {
        let len = encode_char(state(p).c, &mut buf);
        text.extend_from_slice(&buf[..len]);
        p = state(p).out;
    }
    text.push(b'\0');
    Some(text)
}

crate::flag_set! {
    /// The `[a-z0-9_]`-shaped pieces a `[]` collection is built from --
    /// upstream's `CLASS_*`. A collection made only of these adds up to one
    /// of the `\w`-style classes, and recognising that lets it compile to a
    /// single state instead of one per member.
    ///
    /// The combinations below are the sums that have a name; anything else
    /// is a collection that has to be compiled member by member.
    struct CollParts;

    /// A leading `^`.
    const NOT = 0x80;
    const AF = 0x40;
    const AF_UPPER = 0x20;
    const AZ = 0x10;
    const AZ_UPPER = 0x8;
    const O7 = 0x4;
    const O9 = 0x2;
    const UNDERSCORE = 0x1;

    const NDIGIT = Self::NOT.bits() | Self::O9.bits();
    const HEX = Self::O9.bits() | Self::AF.bits() | Self::AF_UPPER.bits();
    const NHEX = Self::NOT.bits() | Self::HEX.bits();
    const NOCTAL = Self::NOT.bits() | Self::O7.bits();
    const WORD = Self::UNDERSCORE.bits()
        | Self::O9.bits()
        | Self::AZ.bits()
        | Self::AZ_UPPER.bits();
    const NWORD = Self::NOT.bits() | Self::WORD.bits();
    const HEAD = Self::UNDERSCORE.bits() | Self::AZ.bits() | Self::AZ_UPPER.bits();
    const NHEAD = Self::NOT.bits() | Self::HEAD.bits();
    const ALPHA = Self::AZ.bits() | Self::AZ_UPPER.bits();
    const NALPHA = Self::NOT.bits() | Self::ALPHA.bits();
    const NAZ = Self::NOT.bits() | Self::AZ.bits();
    const NUP = Self::NOT.bits() | Self::AZ_UPPER.bits();
}

/// The opcode an accumulated set of parts stands for, if any.
fn class_opcode(config: CollParts) -> Option<NfaOp> {
    Some(match config {
        CollParts::O9 => NfaOp::Digit,
        CollParts::NDIGIT => NfaOp::Ndigit,
        CollParts::HEX => NfaOp::Hex,
        CollParts::NHEX => NfaOp::Nhex,
        CollParts::O7 => NfaOp::Octal,
        CollParts::NOCTAL => NfaOp::Noctal,
        CollParts::WORD => NfaOp::Word,
        CollParts::NWORD => NfaOp::Nword,
        CollParts::HEAD => NfaOp::Head,
        CollParts::NHEAD => NfaOp::Nhead,
        CollParts::ALPHA => NfaOp::Alpha,
        CollParts::NALPHA => NfaOp::Nalpha,
        CollParts::AZ => NfaOp::LowerIc,
        CollParts::NAZ => NfaOp::NlowerIc,
        CollParts::AZ_UPPER => NfaOp::UpperIc,
        CollParts::NUP => NfaOp::NupperIc,
        _ => return None,
    })
}

/// Is the collection running from `text[start]` up to `text[end]` (its
/// closing `]`) one of the character classes? Answers the class and whether
/// it also accepts a line break, which the `\_[` form and a `\n` member both
/// ask for.
pub(crate) fn nfa_recognize_char_class(
    text: &[u8],
    start: usize,
    end: usize,
    extra_newl: bool,
) -> Option<(NfaOp, bool)> {
    let byte = |i: usize| cstr::byte_at(text, i);
    if byte(end) != b']' {
        return None;
    }
    let mut config = CollParts::NONE;
    let mut newl = extra_newl;
    let mut p = start;
    if byte(p) == b'^' {
        config |= CollParts::NOT;
        p += 1;
    }
    while p < end {
        if p + 2 < end && byte(p + 1) == b'-' {
            config |= match (byte(p), byte(p + 2)) {
                (b'0', b'9') => CollParts::O9,
                (b'0', b'7') => CollParts::O7,
                (b'a', b'z') => CollParts::AZ,
                (b'a', b'f') => CollParts::AF,
                (b'A', b'Z') => CollParts::AZ_UPPER,
                (b'A', b'F') => CollParts::AF_UPPER,
                _ => return None,
            };
            p += 3;
        } else if p + 1 < end && byte(p) == b'\\' && byte(p + 1) == b'n' {
            newl = true;
            p += 2;
        } else if byte(p) == b'_' {
            config |= CollParts::UNDERSCORE;
            p += 1;
        } else if byte(p) == b'\n' {
            newl = true;
            p += 1;
        } else {
            return None;
        }
    }
    // A range that straddled `end` would have left `p` past it.
    if p != end {
        return None;
    }
    class_opcode(config).map(|op| (op, newl))
}
