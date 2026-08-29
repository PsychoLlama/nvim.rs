//! The atoms that stand for characters rather than for structure: a run of
//! ordinary text, one of the `\d`/`\w`/`\s` class shorthands, and `\~` — the
//! previous `:substitute` replacement, spliced in as literal text.

#![deny(unsafe_op_in_unsafe_fn)]

use core::ffi::c_int;

use super::compile::{regc, regmbc, regnode, use_multibytecode};
use crate::main::{e_nopresub, rc_did_emsg};
use crate::mbyte::{utf_composinglike, utf_iscomposing_legacy, utf_ptr2char, utf_ptr2len};
use crate::message::emsg;
use crate::os::cshim::gettext;
use crate::regexp::{
    ALPHA, ANY, DIGIT, EXACTLY, FNAME, GRAPHEME_STATE_INIT, HASWIDTH, HEAD, HEX, IDENT, KWORD,
    LOWER, MULTIBYTECODE, NALPHA, NDIGIT, NHEAD, NHEX, NLOWER, NOCTAL, NOT_MULTI, NUPPER, NWHITE,
    NWORD, OCTAL, PRINT, SFNAME, SIDENT, SIMPLE, SKWORD, SPRINT, UPPER, WHITE, WORD, getchr, magic,
    one_exactly, peekchr, re_multi_type, reg_prev_sub, regparse, skipchr, ungetchr, unmagic,
};
use crate::semsg;
use crate::types::{GraphemeState, NUL, uint8_t};

/// The `\x` class shorthands, in the order upstream's two parallel tables
/// (`classchars` and `classcodes`) paired them. The `\_x` form of each is the
/// same opcode plus `ADD_NL`.
static CLASS_SHORTHANDS: [(u8, c_int); 27] = [
    (b'.', ANY),
    (b'i', IDENT),
    (b'I', SIDENT),
    (b'k', KWORD),
    (b'K', SKWORD),
    (b'f', FNAME),
    (b'F', SFNAME),
    (b'p', PRINT),
    (b'P', SPRINT),
    (b's', WHITE),
    (b'S', NWHITE),
    (b'd', DIGIT),
    (b'D', NDIGIT),
    (b'x', HEX),
    (b'X', NHEX),
    (b'o', OCTAL),
    (b'O', NOCTAL),
    (b'w', WORD),
    (b'W', NWORD),
    (b'h', HEAD),
    (b'H', NHEAD),
    (b'a', ALPHA),
    (b'A', NALPHA),
    (b'l', LOWER),
    (b'L', NLOWER),
    (b'u', UPPER),
    (b'U', NUPPER),
];

/// Is `c` the magic form of a class shorthand? Those are the atoms that
/// [`regatom`](super::atom::regatom) hands straight to [`class_shorthand`].
pub(crate) fn is_class_shorthand(c: c_int) -> bool {
    c < 0 && CLASS_SHORTHANDS.iter().any(|(name, _)| magic(*name) == c)
}

/// One of the class shorthands, with `extra` either 0 or `ADD_NL`.
///
/// Reached both from a magic `\d` and from a `\_d`, which is why the lookup
/// is on the unmagicked character.
pub(crate) fn class_shorthand(flagp: &mut c_int, c: c_int, extra: c_int) -> *mut uint8_t {
    let Some(&(_, code)) = CLASS_SHORTHANDS
        .iter()
        .find(|(name, _)| *name as c_int == unmagic(c))
    else {
        // The only way to get here is `\_` followed by something that is not
        // a class.
        semsg!("E63: Invalid use of \\_");
        rc_did_emsg.set(true);
        return core::ptr::null_mut();
    };

    // `.` followed by a combining character is that grapheme, not the "any"
    // class — but only for the magic `.`; `\_.` stays the class.
    if c == magic(b'.') && utf_iscomposing_legacy(peekchr()) {
        return multibyte_node(flagp, getchr());
    }

    let ret = regnode(code + extra);
    *flagp |= HASWIDTH | SIMPLE;
    ret
}

/// A single character that has to be matched as a whole rather than as
/// bytes, because a multi may follow it or it can carry combining marks.
fn multibyte_node(flagp: &mut c_int, c: c_int) -> *mut uint8_t {
    let ret = regnode(MULTIBYTECODE);
    regmbc(c);
    *flagp |= HASWIDTH | SIMPLE;
    ret
}

/// A run of ordinary characters, emitted as one `EXACTLY` node.
///
/// The run stops before a character a multi could apply to, so that `abc*`
/// repeats only the `c`: everything but the last character of the run is
/// safe to swallow. `one_exactly` — set while parsing a `\%[...]` member —
/// caps the run at one character.
pub(crate) fn literal_run(flagp: &mut c_int, mut c: c_int) -> *mut uint8_t {
    if use_multibytecode(c) {
        return multibyte_node(flagp, c);
    }

    let ret = regnode(EXACTLY);
    let mut len = 0;
    // A negative `c` is a metacharacter, which only the first iteration may
    // take (as a literal): stopping before one is what leaves it for the
    // next atom.
    while c != NUL
        && (len == 0 || (re_multi_type(peekchr()) == NOT_MULTI && one_exactly.get() == 0 && c >= 0))
    {
        regmbc(unmagic(c));
        emit_combining_marks();
        c = getchr();
        len += 1;
    }
    ungetchr();
    regc(NUL);
    *flagp |= HASWIDTH;
    if len == 1 {
        *flagp |= SIMPLE;
    }
    ret
}

/// Swallow the combining characters that belong with the character just
/// emitted, so that the grapheme stays one `EXACTLY` operand.
fn emit_combining_marks() {
    let mut state: GraphemeState = GRAPHEME_STATE_INIT as GraphemeState;
    // SAFETY: `regparse` points into the NUL-terminated pattern, and
    // `utf_composinglike` stops the walk at its end.
    loop {
        let len = unsafe { utf_ptr2len(regparse.get()) };
        if !unsafe {
            utf_composinglike(regparse.get(), regparse.get().add(len as usize), &mut state)
        } {
            break;
        }
        regmbc(unsafe { utf_ptr2char(regparse.get()) });
        skipchr();
    }
}

/// `\~`: the text of the last `:substitute` replacement, as literal
/// characters.
pub(crate) fn previous_substitute(flagp: &mut c_int) -> *mut uint8_t {
    // SAFETY: `reg_prev_sub` is either null or a NUL-terminated copy of the
    // replacement, owned by the substitute code.
    let sub = reg_prev_sub.get().cast::<uint8_t>();
    if sub.is_null() {
        unsafe { emsg(gettext(e_nopresub.as_ptr())) };
        rc_did_emsg.set(true);
        return core::ptr::null_mut();
    }
    let ret = regnode(EXACTLY);
    let mut end = sub;
    while unsafe { *end } as c_int != NUL {
        regc(unsafe { *end } as c_int);
        end = unsafe { end.add(1) };
    }
    regc(NUL);
    if unsafe { *sub } as c_int != NUL {
        *flagp |= HASWIDTH;
        if unsafe { end.offset_from(sub) } == 1 {
            *flagp |= SIMPLE;
        }
    }
    ret
}
