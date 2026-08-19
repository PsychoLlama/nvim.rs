//! One atom of a pattern: the smallest thing a multi can repeat.
//!
//! This is only the dispatch. The three families that need more than a node
//! or two live next door: [`super::collection`] for `[...]`,
//! [`super::escape`] for the `\z` and `\%` escapes, and [`super::literal`]
//! for a run of ordinary characters.

#![forbid(unsafe_code)]

use core::ffi::c_int;

use super::collection::{Collection, collection};
use super::compile::{regc, regnode, seen_endbrace};
use super::escape::{percent_atom, z_atom};
use super::literal::{class_shorthand, is_class_shorthand, literal_run, previous_substitute};
use super::piece::reg;
use crate::main::rc_did_emsg;
use crate::regexp::{
    ADD_NL, BACKREF, BOL, BOW, EOL, EOW, EXACTLY, HASLOOKBH, HASNL, HASWIDTH, MAGIC_ALL, MAGIC_ON,
    NEWL, NL, REG_PAREN, Rex, SIMPLE, SPSTART, WORST, getchr, had_eol, magic, magic_prefix,
    one_exactly, prev_at_start, reg_magic, reg_string, unmagic,
};
use crate::semsg;
use crate::types::{NUL, uint8_t};

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
const M_0: c_int = magic(b'0');
const M_1: c_int = magic(b'1');
const M_9: c_int = magic(b'9');

/// `\%[abc]` compiles each of its members as a single atom, so a member that
/// is itself a group or an alternation cannot work. Reports E369 and returns
/// true when we are inside one.
pub(crate) fn denied_in_optional_sequence() -> bool {
    if one_exactly.get() == 0 {
        return false;
    }
    let prefix = magic_prefix();
    semsg!("E369: Invalid item in {prefix}%[]");
    rc_did_emsg.set(true);
    true
}

/// Parse one atom, emit its nodes and describe it in `*flagp`.
///
/// Returns null when an error has already been reported.
pub(crate) fn regatom(rex: Rex, flagp: &mut c_int) -> *mut uint8_t {
    // `\%23l` restores the "still at the start of the pattern" flag, because
    // a position assertion consumes no input; [`percent_atom`] needs the
    // value from before this atom was read.
    let save_prev_at_start = prev_at_start.get();
    *flagp = WORST;
    let mut c = getchr();

    // `\_x` is "x, or a line break". The atom is built as usual and `ADD_NL`
    // shifts its opcode to the newline-accepting variant of itself.
    if c == M_UNDERSCORE {
        c = unmagic(getchr());
        return match c as u8 {
            b'^' => regnode(BOL),
            b'$' => {
                had_eol.set(1);
                regnode(EOL)
            }
            _ => {
                *flagp |= HASNL;
                if c == b'[' as c_int {
                    bracketed(rex, flagp, ADD_NL, c)
                } else {
                    class_shorthand(flagp, c, ADD_NL)
                }
            }
        };
    }

    match c {
        M_CARET => regnode(BOL),
        M_DOLLAR => {
            had_eol.set(1);
            regnode(EOL)
        }
        M_LT => regnode(BOW),
        M_GT => regnode(EOW),

        // `\n` is a line break, except in a string match, where there are no
        // lines and it is just the byte.
        M_N => {
            if reg_string.get() != 0 {
                let ret = regnode(EXACTLY);
                regc(NL);
                regc(NUL);
                *flagp |= HASWIDTH | SIMPLE;
                ret
            } else {
                let ret = regnode(NEWL);
                *flagp |= HASWIDTH | HASNL;
                ret
            }
        }

        M_PAREN_OPEN => {
            if denied_in_optional_sequence() {
                return core::ptr::null_mut();
            }
            let mut flags = 0;
            let ret = reg(rex, REG_PAREN, &mut flags);
            if !ret.is_null() {
                *flagp |= flags & (HASWIDTH | SPSTART | HASNL | HASLOOKBH);
            }
            ret
        }

        // These end an alternative, so `regconcat` should already have
        // stopped; reaching one here means the parser lost track.
        NUL | M_BAR | M_AMP | M_PAREN_CLOSE => {
            if denied_in_optional_sequence() {
                return core::ptr::null_mut();
            }
            semsg!("E473: Internal error in regexp");
            rc_did_emsg.set(true);
            core::ptr::null_mut()
        }

        // A multi with no atom in front of it.
        M_EQUAL | M_QUESTION | M_PLUS | M_AT | M_BRACE | M_STAR => {
            let c = unmagic(c);
            // As in E61: `*` is magic one level sooner than the rest, so the
            // backslash this message shows is decided by a looser test.
            let bare = if c == b'*' as c_int {
                reg_magic.get() >= MAGIC_ON
            } else {
                reg_magic.get() == MAGIC_ALL
            };
            let prefix = if bare { "" } else { "\\" };
            let c = c as u8 as char;
            semsg!("E64: {prefix}{c} follows nothing");
            rc_did_emsg.set(true);
            core::ptr::null_mut()
        }

        M_TILDE => previous_substitute(flagp),

        M_1..=M_9 => {
            let refnum = c - M_0;
            if !seen_endbrace(refnum) {
                return core::ptr::null_mut();
            }
            regnode(BACKREF + refnum)
        }

        M_Z => z_atom(rex, flagp),
        M_PERCENT => percent_atom(rex, flagp, save_prev_at_start),
        M_BRACKET => bracketed(rex, flagp, 0, c),

        // `\d`, `\w`, `\s`, … in their magic form.
        c if is_class_shorthand(c) => class_shorthand(flagp, c, 0),

        _ => literal_run(flagp, c),
    }
}

/// A `[` that may open a collection. If it does not close, it is an ordinary
/// character — unless 'regexpengine' strictness is on, where the missing `]`
/// is an error.
fn bracketed(rex: Rex, flagp: &mut c_int, extra: c_int, c: c_int) -> *mut uint8_t {
    match collection(rex, flagp, extra) {
        Collection::Node(node) => node,
        Collection::Failed => core::ptr::null_mut(),
        Collection::NotACollection => literal_run(flagp, c),
    }
}
