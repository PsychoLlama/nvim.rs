//! Setting up a compile and reading back what the finished machine implies:
//! whether it is anchored, what character it must start with, and any
//! literal text the whole pattern reduces to.
//!
//! The last three are the shortcuts `nfa_regexec_both` uses to skip ahead in
//! the line without running the machine at all, so each walks the states
//! conservatively and gives up — returning "no shortcut" — the moment it
//! meets anything it cannot reason about.

#![deny(unsafe_op_in_unsafe_fn)]

use core::ffi::c_int;

use super::postfix;
use crate::mbyte::{utf_char2bytes, utf_char2len};
use crate::memory::xmalloc;
use crate::regexp::{
    CLASS_AF, CLASS_AZ, CLASS_af, CLASS_az, CLASS_not, CLASS_o7, CLASS_o9, CLASS_underscore,
    NFA_ADD_NL, NFA_ALPHA, NFA_BOF, NFA_BOL, NFA_BOW, NFA_CURSOR, NFA_DIGIT, NFA_EOW, NFA_HEAD,
    NFA_HEX, NFA_LOWER_IC, NFA_MATCH, NFA_MCLOSE, NFA_MOPEN, NFA_MOPEN9, NFA_NALPHA, NFA_NDIGIT,
    NFA_NHEAD, NFA_NHEX, NFA_NLOWER_IC, NFA_NOCTAL, NFA_NOPEN, NFA_NUPPER_IC, NFA_NWORD, NFA_OCTAL,
    NFA_SPLIT, NFA_UPPER_IC, NFA_VISUAL, NFA_WORD, NFA_ZEND, NFA_ZOPEN, NFA_ZOPEN9, NFA_ZSTART,
    Rex, istate, nfa_state_T, nstate, regcomp_start, wants_nfa,
};
use crate::types::{FAIL, NUL, uint8_t};
use ::libc::strlen;

/// Reset the compile-time state and reserve the postfix program.
///
/// # Safety
///
/// `expr` must be the NUL-terminated pattern about to be parsed.
pub(crate) unsafe fn nfa_regcomp_start(rex: Rex, expr: *mut uint8_t, re_flags: c_int) {
    nstate.set(0);
    istate.set(0);
    // SAFETY: the caller's NUL-terminated pattern.
    postfix::start(unsafe { strlen(expr.cast()) });
    wants_nfa.set(false);
    rex.set_nfa_has_zend(0);
    rex.set_nfa_has_backref(0);
    regcomp_start(expr, re_flags);
}

/// How far the walks below follow a `NFA_SPLIT` before giving up.
const MAX_DEPTH: c_int = 4;

/// The capture brackets and `\zs`/`\ze` markers: they consume no input, so a
/// walk looking for the first *character* steps straight through them.
fn is_bracket(c: c_int) -> bool {
    matches!(c, NFA_ZSTART | NFA_ZEND | NFA_NOPEN)
        || (NFA_MOPEN..=NFA_MOPEN9).contains(&c)
        || (NFA_ZOPEN..=NFA_ZOPEN9).contains(&c)
}

/// Must every match start at the beginning of a line?
///
/// # Safety
///
/// `start` must be null or a state of a live program.
pub(crate) unsafe fn nfa_get_reganch(start: *mut nfa_state_T, depth: c_int) -> c_int {
    if depth > MAX_DEPTH {
        return 0;
    }
    // SAFETY: the caller's program; `out`/`out1` stay inside it.
    let mut p = start;
    while !p.is_null() {
        match unsafe { (*p).c } {
            NFA_BOL | NFA_BOF => return 1,
            // Zero-width, so the anchor is whatever follows. Note that
            // `\%23l` and its neighbours are *not* here: unlike
            // `nfa_get_regstart` below, a position assertion stops this
            // walk.
            NFA_CURSOR | NFA_VISUAL => p = unsafe { (*p).out },
            c if is_bracket(c) => p = unsafe { (*p).out },
            // Anchored only if both alternatives are.
            NFA_SPLIT => {
                return (unsafe { nfa_get_reganch((*p).out, depth + 1) } != 0
                    && unsafe { nfa_get_reganch((*p).out1, depth + 1) } != 0)
                    as c_int;
            }
            _ => return 0,
        }
    }
    0
}

/// The character every match must start with, or 0 if there is no single
/// such character.
///
/// # Safety
///
/// `start` must be null or a state of a live program.
pub(crate) unsafe fn nfa_get_regstart(start: *mut nfa_state_T, depth: c_int) -> c_int {
    if depth > MAX_DEPTH {
        return 0;
    }
    // SAFETY: the caller's program; `out`/`out1` stay inside it.
    let mut p = start;
    while !p.is_null() {
        match unsafe { (*p).c } {
            // Zero-width: the start character is whatever follows.
            // `NFA_CURSOR..=NFA_VISUAL` is the whole position-assertion
            // block, `\%#` through `\%V`.
            NFA_BOL | NFA_BOF | NFA_BOW | NFA_EOW => p = unsafe { (*p).out },
            NFA_CURSOR..=NFA_VISUAL => p = unsafe { (*p).out },
            c if is_bracket(c) => p = unsafe { (*p).out },
            // Only if both alternatives agree.
            NFA_SPLIT => {
                let c1 = unsafe { nfa_get_regstart((*p).out, depth + 1) };
                let c2 = unsafe { nfa_get_regstart((*p).out1, depth + 1) };
                return if c1 == c2 { c1 } else { 0 };
            }
            // A literal character is one; any other opcode is not.
            c => return if c > 0 { c } else { 0 },
        }
    }
    0
}

/// The literal text the whole pattern matches, as a fresh NUL-terminated
/// allocation, or null when it is not one plain run of characters.
///
/// # Safety
///
/// `start` must be a state of a live program.
pub(crate) unsafe fn nfa_get_match_text(start: *mut nfa_state_T) -> *mut uint8_t {
    // SAFETY: the caller's program. The measuring walk proves the chain
    // ends in `NFA_MCLOSE` -> `NFA_MATCH` before the writing walk follows
    // it again.
    if unsafe { (*start).c } != NFA_MOPEN {
        return core::ptr::null_mut();
    }
    let mut p = unsafe { (*start).out };
    let mut len = 0;
    while unsafe { (*p).c } > 0 {
        len += utf_char2len(unsafe { (*p).c });
        p = unsafe { (*p).out };
    }
    if unsafe { (*p).c } != NFA_MCLOSE || unsafe { (*(*p).out).c } != NFA_MATCH {
        return core::ptr::null_mut();
    }

    // `len` counted the first character too, and the write below skips
    // it (it is reported separately as the program's `regstart`), so
    // there is always at least one spare byte for the terminator.
    let text = unsafe { xmalloc(len as usize) } as *mut uint8_t;
    let mut out = text;
    let mut p = unsafe { (*(*start).out).out };
    while unsafe { (*p).c } > 0 {
        out = unsafe { out.offset(utf_char2bytes((*p).c, out.cast()) as isize) };
        p = unsafe { (*p).out };
    }
    unsafe { *out = NUL as uint8_t };
    text
}

/// The masks of `[a-z0-9_]`-shaped pieces that add up to one of the
/// `\w`-style classes. Recognising them lets such a collection compile to a
/// single state instead of one per member.
const CLASS_NDIGIT: c_int = CLASS_not | CLASS_o9;
const CLASS_HEX: c_int = CLASS_o9 | CLASS_af | CLASS_AF;
const CLASS_NHEX: c_int = CLASS_not | CLASS_HEX;
const CLASS_NOCTAL: c_int = CLASS_not | CLASS_o7;
const CLASS_WORD: c_int = CLASS_underscore | CLASS_o9 | CLASS_az | CLASS_AZ;
const CLASS_NWORD: c_int = CLASS_not | CLASS_WORD;
const CLASS_HEAD: c_int = CLASS_underscore | CLASS_az | CLASS_AZ;
const CLASS_NHEAD: c_int = CLASS_not | CLASS_HEAD;
const CLASS_ALPHA: c_int = CLASS_az | CLASS_AZ;
const CLASS_NALPHA: c_int = CLASS_not | CLASS_ALPHA;
const CLASS_NAZ: c_int = CLASS_not | CLASS_az;
const CLASS_NUP: c_int = CLASS_not | CLASS_AZ;

/// The opcode an accumulated mask stands for, if any.
fn class_opcode(config: c_int) -> Option<c_int> {
    Some(match config {
        CLASS_o9 => NFA_DIGIT,
        CLASS_NDIGIT => NFA_NDIGIT,
        CLASS_HEX => NFA_HEX,
        CLASS_NHEX => NFA_NHEX,
        CLASS_o7 => NFA_OCTAL,
        CLASS_NOCTAL => NFA_NOCTAL,
        CLASS_WORD => NFA_WORD,
        CLASS_NWORD => NFA_NWORD,
        CLASS_HEAD => NFA_HEAD,
        CLASS_NHEAD => NFA_NHEAD,
        CLASS_ALPHA => NFA_ALPHA,
        CLASS_NALPHA => NFA_NALPHA,
        CLASS_az => NFA_LOWER_IC,
        CLASS_NAZ => NFA_NLOWER_IC,
        CLASS_AZ => NFA_UPPER_IC,
        CLASS_NUP => NFA_NUPPER_IC,
        _ => return None,
    })
}

/// Is the collection running from `start` up to `end` (its closing `]`) one
/// of the character classes? Returns the opcode, raised by `NFA_ADD_NL` when
/// it also accepts a line break, or `FAIL`.
///
/// # Safety
///
/// `start..=end` must be one collection inside the pattern being parsed.
pub(crate) unsafe fn nfa_recognize_char_class(
    start: *mut uint8_t,
    end: *const uint8_t,
    extra_newl: c_int,
) -> c_int {
    let end = end.cast_mut();
    // SAFETY: the caller's collection; every read is between `start` and
    // `end`, which is where the loop keeps `p`.
    if unsafe { *end } as c_int != ']' as c_int {
        return FAIL;
    }
    let mut config = 0;
    let mut newl = extra_newl == 1;
    let mut p = start;
    if unsafe { *p } == b'^' {
        config |= CLASS_not;
        p = unsafe { p.add(1) };
    }
    while p < end {
        if unsafe { p.add(2) } < end && unsafe { *p.add(1) } == b'-' {
            config |= match (unsafe { *p }, unsafe { *p.add(2) }) {
                (b'0', b'9') => CLASS_o9,
                (b'0', b'7') => CLASS_o7,
                (b'a', b'z') => CLASS_az,
                (b'a', b'f') => CLASS_af,
                (b'A', b'Z') => CLASS_AZ,
                (b'A', b'F') => CLASS_AF,
                _ => return FAIL,
            };
            p = unsafe { p.add(3) };
        } else if unsafe { p.add(1) } < end
            && unsafe { *p } == b'\\'
            && unsafe { *p.add(1) } == b'n'
        {
            newl = true;
            p = unsafe { p.add(2) };
        } else if unsafe { *p } == b'_' {
            config |= CLASS_underscore;
            p = unsafe { p.add(1) };
        } else if unsafe { *p } == b'\n' {
            newl = true;
            p = unsafe { p.add(1) };
        } else {
            return FAIL;
        }
    }
    // A range that straddled `end` would have left `p` past it.
    if p != end {
        return FAIL;
    }
    let extra = if newl { NFA_ADD_NL } else { extra_newl };
    match class_opcode(config) {
        Some(op) => extra + op,
        None => FAIL,
    }
}
