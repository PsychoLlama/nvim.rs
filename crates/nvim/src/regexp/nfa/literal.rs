//! The atoms that stand for characters rather than for structure: one of the
//! `\d`/`\w`/`\s` class shorthands, a back-reference, `\~` — the previous
//! `:substitute` replacement — and an ordinary character, which may be a
//! grapheme with combining marks on it.

#![deny(unsafe_op_in_unsafe_fn)]

use crate::siemsg_c;
use core::ffi::{c_char, c_int};

use super::{Parsed, Rejected, cursor, postfix};
use crate::main::{e_nopresub, rc_did_emsg};
use crate::mbyte::{utf_char2len, utf_ptr2char, utf_ptr2len};
use crate::message::emsg;
use crate::os::cshim::gettext;
use crate::regexp::{
    NFA_ADD_NL, NFA_ALPHA, NFA_ANY, NFA_BACKREF1, NFA_COMPOSING, NFA_CONCAT, NFA_DIGIT, NFA_FNAME,
    NFA_HEAD, NFA_HEX, NFA_IDENT, NFA_KWORD, NFA_LOWER, NFA_NALPHA, NFA_NDIGIT, NFA_NEWL,
    NFA_NHEAD, NFA_NHEX, NFA_NLOWER, NFA_NOCTAL, NFA_NOPEN, NFA_NUPPER, NFA_NWHITE, NFA_NWORD,
    NFA_OCTAL, NFA_OR, NFA_PRINT, NFA_SFNAME, NFA_SIDENT, NFA_SKWORD, NFA_SPRINT, NFA_UPPER,
    NFA_WHITE, NFA_WORD, RF_HASNL, Rex, getchr, magic, peekchr, reg_prev_sub, regflags,
    seen_endbrace, unmagic,
};
use crate::semsg;
use crate::types::NUL;

/// The `\x` class shorthands, in the order upstream's two parallel tables
/// (`classchars` and `nfa_classcodes`) paired them. The `\_x` form of each is
/// the same opcode with a line break offered as an alternative.
static CLASS_SHORTHANDS: [(u8, c_int); 27] = [
    (b'.', NFA_ANY),
    (b'i', NFA_IDENT),
    (b'I', NFA_SIDENT),
    (b'k', NFA_KWORD),
    (b'K', NFA_SKWORD),
    (b'f', NFA_FNAME),
    (b'F', NFA_SFNAME),
    (b'p', NFA_PRINT),
    (b'P', NFA_SPRINT),
    (b's', NFA_WHITE),
    (b'S', NFA_NWHITE),
    (b'd', NFA_DIGIT),
    (b'D', NFA_NDIGIT),
    (b'x', NFA_HEX),
    (b'X', NFA_NHEX),
    (b'o', NFA_OCTAL),
    (b'O', NFA_NOCTAL),
    (b'w', NFA_WORD),
    (b'W', NFA_NWORD),
    (b'h', NFA_HEAD),
    (b'H', NFA_NHEAD),
    (b'a', NFA_ALPHA),
    (b'A', NFA_NALPHA),
    (b'l', NFA_LOWER),
    (b'L', NFA_NLOWER),
    (b'u', NFA_UPPER),
    (b'U', NFA_NUPPER),
];

/// Is `c` the magic form of a class shorthand?
pub(crate) fn is_class_shorthand(c: c_int) -> bool {
    c < 0 && CLASS_SHORTHANDS.iter().any(|(name, _)| magic(*name) == c)
}

/// One of the class shorthands, with `extra` either 0 or `NFA_ADD_NL`.
///
/// Reached both from a magic `\d` and from a `\_d`, which is why the lookup
/// is on the unmagicked character. `atom_start` is where this atom began,
/// which the `.`-plus-combining-character case needs.
pub(crate) fn class_shorthand(c: c_int, extra: c_int) -> Parsed {
    let Some(&(_, code)) = CLASS_SHORTHANDS
        .iter()
        .find(|(name, _)| *name as c_int == unmagic(c))
    else {
        // Reachable two ways: `\_` followed by something that is not a
        // class, and — only in principle — a dispatch that sent a character
        // here that is not one either.
        if extra == NFA_ADD_NL {
            let c = c as i64;
            semsg!("E877: (NFA regexp) Invalid character class: {c}");
            rc_did_emsg.set(true);
        } else {
            // `siemsg`, not `semsg`: an internal-error report, which is
            // dropped while messages are being held back. Unreachable in
            // practice — the dispatch only sends class characters here.
            // SAFETY: a NUL-terminated literal and the int its `%d` takes.
            unsafe { siemsg_c!(c"INTERNAL: Unknown character class char: %d".as_ptr(), c) };
        }
        return Err(Rejected);
    };

    // `.` followed by a combining character is that grapheme, not the "any"
    // class — but only for the magic `.`; `\_.` stays the class.
    if c == magic(b'.') && cursor::is_composing(peekchr()) {
        let atom_start = cursor::here();
        return literal(getchr(), atom_start);
    }

    postfix::emit(code);
    if extra == NFA_ADD_NL {
        postfix::emit(NFA_NEWL);
        postfix::emit(NFA_OR);
        regflags.set(regflags.get() | RF_HASNL as u32);
    }
    Ok(())
}

/// `\1` .. `\9`: match what that capture group matched.
pub(crate) fn back_reference(rex: Rex, c: c_int) -> Parsed {
    let refnum = unmagic(c) - b'1' as c_int;
    // A back-reference to a group that has not been closed yet cannot work.
    if !seen_endbrace(refnum + 1) {
        return Err(Rejected);
    }
    postfix::emit(NFA_BACKREF1 + refnum);
    rex.set_nfa_has_backref(1);
    Ok(())
}

/// `\~`: the text of the last `:substitute` replacement, as literal
/// characters wrapped in a group so that a repeat after it applies to the
/// whole run.
pub(crate) fn previous_substitute() -> Parsed {
    // SAFETY: `reg_prev_sub` is either null or a NUL-terminated copy of the
    // replacement, owned by the substitute code.
    let sub = reg_prev_sub.get();
    if sub.is_null() {
        unsafe { emsg(gettext(&raw const e_nopresub as *const c_char)) };
        return Err(Rejected);
    }
    let mut p = sub;
    while unsafe { *p } as c_int != NUL {
        postfix::emit(unsafe { utf_ptr2char(p) });
        // The join goes after the second and every later character, so
        // the run reads as `a b CONCAT c CONCAT …`.
        if p != sub {
            postfix::emit(NFA_CONCAT);
        }
        p = unsafe { p.add(utf_ptr2len(p) as usize) };
    }
    postfix::emit(NFA_NOPEN);
    Ok(())
}

/// An ordinary character. `atom_start` is where it begins in the pattern,
/// which is what says whether it carries combining marks.
///
/// A grapheme becomes an `NFA_COMPOSING` group holding each of its code
/// points; a plain character is one item.
pub(crate) fn literal(mut c: c_int, atom_start: *mut c_char) -> Parsed {
    let plen = cursor::grapheme_len(atom_start);
    if utf_char2len(c) == plen && !cursor::is_composing(c) {
        postfix::emit(unmagic(c));
        return Ok(());
    }

    let mut i = 0;
    loop {
        postfix::emit(c);
        if i > 0 {
            postfix::emit(NFA_CONCAT);
        }
        i += utf_char2len(c);
        if i >= plen {
            break;
        }
        c = cursor::char_at(atom_start, i);
    }
    postfix::emit(NFA_COMPOSING);
    cursor::seek_to(atom_start.wrapping_offset(plen as isize));
    Ok(())
}
