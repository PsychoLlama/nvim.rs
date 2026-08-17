//! One atom of a pattern: the smallest thing a repeat can apply to.
//!
//! This is only the dispatch. The families that need more than an item or
//! two live next door: [`super::collection`] for `[...]`, [`super::escape`]
//! for the `\z` and `\%` escapes, and [`super::literal`] for the class
//! shorthands, back-references and ordinary characters.

#![forbid(unsafe_code)]

use core::ffi::{c_char, c_int};

use super::collection::{Collection, collection};
use super::escape::{percent_atom, z_atom};
use super::literal::{
    back_reference, class_shorthand, is_class_shorthand, literal, previous_substitute,
};
use super::parse::nfa_reg;
use super::{cursor, postfix};
use crate::main::rc_did_emsg;
use crate::regexp::{
    FAIL, NFA_ADD_NL, NFA_BOL, NFA_BOW, NFA_EOL, NFA_EOW, NFA_NEWL, NL, NUL, OK, REG_PAREN,
    RF_HASNL, Rex, getchr, had_eol, magic, prev_at_start, reg_string, regflags, unmagic,
};
use crate::semsg;

const M_AMP: c_int = magic(b'&');
const M_AT: c_int = magic(b'@');
const M_BAR: c_int = magic(b'|');
const M_BRACE: c_int = magic(b'{');
const M_BRACKET: c_int = magic(b'[');
const M_CARET: c_int = magic(b'^');
const M_DOLLAR: c_int = magic(b'$');
const M_EQUAL: c_int = magic(b'=');
const M_GT: c_int = magic(b'>');
const M_LT: c_int = magic(b'<');
const M_N: c_int = magic(b'n');
const M_PAREN_CLOSE: c_int = magic(b')');
const M_PAREN_OPEN: c_int = magic(b'(');
const M_PERCENT: c_int = magic(b'%');
const M_PLUS: c_int = magic(b'+');
const M_QUESTION: c_int = magic(b'?');
const M_STAR: c_int = magic(b'*');
const M_TILDE: c_int = magic(b'~');
const M_UNDERSCORE: c_int = magic(b'_');
const M_Z: c_int = magic(b'z');
const M_1: c_int = magic(b'1');
const M_9: c_int = magic(b'9');

/// Parse one atom and append it to the postfix program.
pub(crate) fn nfa_regatom(rex: Rex) -> c_int {
    // `\%23l` restores the "still at the start of the pattern" flag, because
    // a position assertion consumes no input; [`percent_atom`] needs the
    // value from before this atom was read.
    let save_prev_at_start = prev_at_start.get();
    // Where this atom starts in the pattern. The character reader hands back
    // a code point, but a grapheme with combining marks on it has to be read
    // from the pattern text, and a collection walks it byte by byte.
    let atom_start = cursor::here();
    let c = getchr();

    // `\_x` is "x, or a line break". The atom is built as usual and the line
    // break is offered as an alternative to it.
    if c == M_UNDERSCORE {
        let c = unmagic(getchr());
        if c == NUL {
            return nul_found();
        }
        return match u8::try_from(c) {
            Ok(b'^') => {
                postfix::emit(NFA_BOL);
                OK
            }
            Ok(b'$') => {
                postfix::emit(NFA_EOL);
                had_eol.set(1);
                OK
            }
            Ok(b'[') => bracketed(NFA_ADD_NL, atom_start, c),
            _ => class_shorthand(c, NFA_ADD_NL),
        };
    }

    match c {
        NUL => nul_found(),
        M_CARET => {
            postfix::emit(NFA_BOL);
            OK
        }
        M_DOLLAR => {
            postfix::emit(NFA_EOL);
            had_eol.set(1);
            OK
        }
        M_LT => {
            postfix::emit(NFA_BOW);
            OK
        }
        M_GT => {
            postfix::emit(NFA_EOW);
            OK
        }

        // `\n` is a line break, except in a string match, where there are no
        // lines and it is just the byte.
        M_N => {
            if reg_string.get() != 0 {
                postfix::emit(NL);
            } else {
                postfix::emit(NFA_NEWL);
                regflags.set(regflags.get() | RF_HASNL as u32);
            }
            OK
        }

        M_PAREN_OPEN => nfa_reg(rex, REG_PAREN),

        // The first three end an alternative, so `nfa_regconcat` should
        // already have stopped; reaching one here means the parser lost
        // track. The rest are repeats with no atom in front of them.
        M_BAR | M_AMP | M_PAREN_CLOSE | M_EQUAL | M_QUESTION | M_PLUS | M_AT | M_STAR | M_BRACE => {
            let c = unmagic(c) as u8 as char;
            semsg!("E866: (NFA regexp) Misplaced {c}");
            FAIL
        }

        M_TILDE => previous_substitute(),
        M_1..=M_9 => back_reference(rex, c),
        M_Z => z_atom(rex),
        M_PERCENT => percent_atom(rex, save_prev_at_start),
        M_BRACKET => bracketed(0, atom_start, c),

        // `\d`, `\w`, `\s`, … in their magic form.
        c if is_class_shorthand(c) => class_shorthand(c, 0),

        _ => literal(c, atom_start),
    }
}

fn nul_found() -> c_int {
    semsg!("E865: (NFA) Regexp end encountered prematurely");
    rc_did_emsg.set(true);
    FAIL
}

/// A `[` that may open a collection. If it does not close, it is an ordinary
/// character — unless 'regexpengine' strictness is on, where the missing `]`
/// is an error.
fn bracketed(extra: c_int, atom_start: *mut c_char, c: c_int) -> c_int {
    match collection(extra, atom_start) {
        Collection::Done => OK,
        Collection::Failed => FAIL,
        Collection::NotACollection => literal(c, atom_start),
    }
}
