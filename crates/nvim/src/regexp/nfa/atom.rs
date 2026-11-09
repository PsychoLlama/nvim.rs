//! One atom of a pattern: the smallest thing a repeat can apply to.
//!
//! This is only the dispatch. The families that need more than an item or
//! two live next door: [`super::collection`] for `[...]`, [`super::escape`]
//! for the `\z` and `\%` escapes, and [`super::literal`] for the class
//! shorthands, back-references and ordinary characters.

#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use crate::regexp::chars::{
    M_1, M_9, M_AMP, M_AT, M_BAR, M_BRACE, M_BRACKET, M_CARET, M_DOLLAR, M_EQUAL, M_GT, M_LT, M_N,
    M_PAREN_CLOSE, M_PAREN_OPEN, M_PERCENT, M_PLUS, M_QUESTION, M_STAR, M_TILDE, M_UNDERSCORE, M_Z,
};
use crate::regexp::{NfaOp, RegCompiler};
use core::ffi::c_int;

use super::collection::{Collection, collection};
use super::escape::{percent_atom, z_atom};
use super::literal::{
    back_reference, class_shorthand, is_class_shorthand, literal, previous_substitute,
};
use super::parse::nfa_reg;
use super::{Parsed, cursor};
use crate::ascii::NUL;
use crate::regexp::{CompileError, NL, REG_PAREN, RF_HASNL, getchr, unmagic};

/// Parse one atom and append it to the postfix program.
pub(crate) fn nfa_regatom(rc: &mut RegCompiler) -> Parsed {
    // `\%23l` restores the "still at the start of the pattern" flag, because
    // a position assertion consumes no input; [`percent_atom`] needs the
    // value from before this atom was read.
    let save_prev_at_start = rc.prev_at_start;
    // Where this atom starts in the pattern. The character reader hands back
    // a code point, but a grapheme with combining marks on it has to be read
    // from the pattern text, and a collection walks it byte by byte.
    let atom_start = cursor::here(rc);
    let c = getchr(rc);

    // `\_x` is "x, or a line break". The atom is built as usual and the line
    // break is offered as an alternative to it.
    if c == M_UNDERSCORE {
        let c = unmagic(getchr(rc));
        if c == NUL {
            return nul_found();
        }
        return match u8::try_from(c) {
            Ok(b'^') => {
                rc.post.emit_op(NfaOp::Bol);
                Ok(())
            }
            Ok(b'$') => {
                rc.post.emit_op(NfaOp::Eol);
                rc.had_eol = 1;
                Ok(())
            }
            Ok(b'[') => bracketed(rc, true, atom_start, c),
            _ => class_shorthand(rc, c, true),
        };
    }

    match c {
        NUL => nul_found(),
        M_CARET => {
            rc.post.emit_op(NfaOp::Bol);
            Ok(())
        }
        M_DOLLAR => {
            rc.post.emit_op(NfaOp::Eol);
            rc.had_eol = 1;
            Ok(())
        }
        M_LT => {
            rc.post.emit_op(NfaOp::Bow);
            Ok(())
        }
        M_GT => {
            rc.post.emit_op(NfaOp::Eow);
            Ok(())
        }

        // `\n` is a line break, except in a string match, where there are no
        // lines and it is just the byte.
        M_N => {
            if rc.string_match != 0 {
                rc.post.emit(NL);
            } else {
                rc.post.emit_op(NfaOp::Newl);
                rc.flags |= RF_HASNL as u32;
            }
            Ok(())
        }

        M_PAREN_OPEN => nfa_reg(rc, REG_PAREN),

        // The first three end an alternative, so `nfa_regconcat` should
        // already have stopped; reaching one here means the parser lost
        // track. The rest are repeats with no atom in front of them.
        M_BAR | M_AMP | M_PAREN_CLOSE | M_EQUAL | M_QUESTION | M_PLUS | M_AT | M_STAR | M_BRACE => {
            // Every arm above is a magic token, so `unmagic` answers the one
            // ASCII byte it was built from; the low byte is that byte.
            Err(CompileError::Misplaced(
                unmagic(c).cast_unsigned().to_le_bytes()[0],
            ))
        }

        M_TILDE => previous_substitute(rc),
        M_1..=M_9 => back_reference(rc, c),
        M_Z => z_atom(rc),
        M_PERCENT => percent_atom(rc, save_prev_at_start),
        M_BRACKET => bracketed(rc, false, atom_start, c),

        // `\d`, `\w`, `\s`, … in their magic form.
        c if is_class_shorthand(c) => class_shorthand(rc, c, false),

        _ => literal(rc, c, atom_start),
    }
}

fn nul_found() -> Parsed {
    Err(CompileError::PrematureEnd)
}

/// A `[` that may open a collection. If it does not close, it is an ordinary
/// character — unless 'regexpengine' strictness is on, where the missing `]`
/// is an error.
fn bracketed(rc: &mut RegCompiler, accepts_newline: bool, atom_start: usize, c: c_int) -> Parsed {
    match collection(rc, accepts_newline, atom_start)? {
        Collection::Done => Ok(()),
        Collection::Literal => literal(rc, c, atom_start),
    }
}
