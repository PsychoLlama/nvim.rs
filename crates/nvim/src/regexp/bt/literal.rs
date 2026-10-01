//! The atoms that stand for characters rather than for structure: a run of
//! ordinary text, one of the `\d`/`\w`/`\s` class shorthands, and `\~` — the
//! previous `:substitute` replacement, spliced in as literal text.

#![deny(unsafe_op_in_unsafe_fn)]
#![allow(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use crate::regexp::RegCompiler;
use core::ffi::c_int;

use super::compile::{regc, regmbc, regnode, regnode_nl, use_multibytecode};
use super::op::BtOp;
use crate::mbyte::{utf_composinglike, utf_iscomposing_legacy, utf_ptr2char, utf_ptr2len};
use crate::message::e_nopresub;
use crate::message::emsg;
use crate::os::cshim::gettext;
use crate::regexp::state::rc_did_emsg;
use crate::regexp::{
    GRAPHEME_STATE_INIT, HASWIDTH, NOT_MULTI, SIMPLE, getchr, magic, peekchr, re_multi_type,
    reg_prev_sub, skipchr, ungetchr, unmagic,
};
use crate::semsg;
use crate::types::{GraphemeState, NUL, uint8_t};

/// The `\x` class shorthands, in the order upstream's two parallel tables
/// (`classchars` and `classcodes`) paired them. The `\_x` form of each is the
/// same opcode with its `\_` form.
static CLASS_SHORTHANDS: [(u8, BtOp); 27] = [
    (b'.', BtOp::Any),
    (b'i', BtOp::Ident),
    (b'I', BtOp::Sident),
    (b'k', BtOp::Kword),
    (b'K', BtOp::Skword),
    (b'f', BtOp::Fname),
    (b'F', BtOp::Sfname),
    (b'p', BtOp::Print),
    (b'P', BtOp::Sprint),
    (b's', BtOp::White),
    (b'S', BtOp::Nwhite),
    (b'd', BtOp::Digit),
    (b'D', BtOp::Ndigit),
    (b'x', BtOp::Hex),
    (b'X', BtOp::Nhex),
    (b'o', BtOp::Octal),
    (b'O', BtOp::Noctal),
    (b'w', BtOp::Word),
    (b'W', BtOp::Nword),
    (b'h', BtOp::Head),
    (b'H', BtOp::Nhead),
    (b'a', BtOp::Alpha),
    (b'A', BtOp::Nalpha),
    (b'l', BtOp::Lower),
    (b'L', BtOp::Nlower),
    (b'u', BtOp::Upper),
    (b'U', BtOp::Nupper),
];

/// Is `c` the magic form of a class shorthand? Those are the atoms that
/// [`regatom`](super::atom::regatom) hands straight to [`class_shorthand`].
pub(crate) fn is_class_shorthand(c: c_int) -> bool {
    c < 0 && CLASS_SHORTHANDS.iter().any(|(name, _)| magic(*name) == c)
}

/// One of the class shorthands. `crosses_lines` is the `\_d` form, which
/// also matches a line break.
///
/// Reached both from a magic `\d` and from a `\_d`, which is why the lookup
/// is on the unmagicked character.
pub(crate) fn class_shorthand(
    rc: &mut RegCompiler,
    flagp: &mut c_int,
    c: c_int,
    crosses_lines: bool,
) -> *mut uint8_t {
    let Some(&(_, code)) = CLASS_SHORTHANDS
        .iter()
        .find(|(name, _)| c_int::from(*name) == unmagic(c))
    else {
        // The only way to get here is `\_` followed by something that is not
        // a class.
        semsg!("E63: Invalid use of \\_");
        rc_did_emsg.set(true);
        return core::ptr::null_mut();
    };

    // `.` followed by a combining character is that grapheme, not the "any"
    // class — but only for the magic `.`; `\_.` stays the class.
    if c == magic(b'.') && utf_iscomposing_legacy(peekchr(rc)) {
        let c = getchr(rc);
        return multibyte_node(rc, flagp, c);
    }

    let ret = regnode_nl(rc, code, crosses_lines);
    *flagp |= HASWIDTH | SIMPLE;
    ret
}

/// A single character that has to be matched as a whole rather than as
/// bytes, because a multi may follow it or it can carry combining marks.
fn multibyte_node(rc: &mut RegCompiler, flagp: &mut c_int, c: c_int) -> *mut uint8_t {
    let ret = regnode(rc, BtOp::Multibytecode);
    regmbc(rc, c);
    *flagp |= HASWIDTH | SIMPLE;
    ret
}

/// A run of ordinary characters, emitted as one `EXACTLY` node.
///
/// The run stops before a character a multi could apply to, so that `abc*`
/// repeats only the `c`: everything but the last character of the run is
/// safe to swallow. `one_exactly` — set while parsing a `\%[...]` member —
/// caps the run at one character.
pub(crate) fn literal_run(rc: &mut RegCompiler, flagp: &mut c_int, mut c: c_int) -> *mut uint8_t {
    if use_multibytecode(rc, c) {
        return multibyte_node(rc, flagp, c);
    }

    let ret = regnode(rc, BtOp::Exactly);
    let mut len = 0;
    // A negative `c` is a metacharacter, which only the first iteration may
    // take (as a literal): stopping before one is what leaves it for the
    // next atom.
    while c != NUL
        && (len == 0 || (re_multi_type(peekchr(rc)) == NOT_MULTI && rc.one_exactly == 0 && c >= 0))
    {
        regmbc(rc, unmagic(c));
        emit_combining_marks(rc);
        c = getchr(rc);
        len += 1;
    }
    ungetchr(rc);
    regc(rc, NUL);
    *flagp |= HASWIDTH;
    if len == 1 {
        *flagp |= SIMPLE;
    }
    ret
}

/// Swallow the combining characters that belong with the character just
/// emitted, so that the grapheme stays one `EXACTLY` operand.
fn emit_combining_marks(rc: &mut RegCompiler) {
    let mut state: GraphemeState = GRAPHEME_STATE_INIT as GraphemeState;
    // SAFETY: `regparse` points into the NUL-terminated pattern, and
    // `utf_composinglike` stops the walk at its end.
    loop {
        let len = unsafe { utf_ptr2len(rc.cursor) };
        let len = usize::try_from(len).expect("a character is at least one byte");
        if !unsafe { utf_composinglike(rc.cursor, rc.cursor.add(len), &mut state) } {
            break;
        }
        regmbc(rc, unsafe { utf_ptr2char(rc.cursor) });
        skipchr(rc);
    }
}

/// `\~`: the text of the last `:substitute` replacement, as literal
/// characters.
pub(crate) fn previous_substitute(rc: &mut RegCompiler, flagp: &mut c_int) -> *mut uint8_t {
    let Some(sub) = reg_prev_sub.with(Clone::clone) else {
        emsg(gettext(e_nopresub));
        rc_did_emsg.set(true);
        return core::ptr::null_mut();
    };
    let ret = regnode(rc, BtOp::Exactly);
    for &byte in sub.iter() {
        regc(rc, c_int::from(byte));
    }
    regc(rc, NUL);
    if !sub.is_empty() {
        *flagp |= HASWIDTH;
        if sub.len() == 1 {
            *flagp |= SIMPLE;
        }
    }
    ret
}
