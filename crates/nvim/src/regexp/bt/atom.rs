//! One atom of a pattern: the smallest thing a multi can repeat.
//!
//! This is only the dispatch. The three families that need more than a node
//! or two live next door: [`super::collection`] for `[...]`,
//! [`super::escape`] for the `\z` and `\%` escapes, and [`super::literal`]
//! for a run of ordinary characters.

#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use super::compile::Node;
use crate::regexp::chars::{
    M_0, M_1, M_9, M_AMP, M_AT, M_BAR, M_BRACE, M_BRACKET, M_CARET, M_DOLLAR, M_EQUAL, M_GT, M_LT,
    M_N, M_PAREN_CLOSE, M_PAREN_OPEN, M_PERCENT, M_PLUS, M_QUESTION, M_STAR, M_TILDE, M_UNDERSCORE,
    M_Z,
};
use crate::regexp::{RegCompiler, low_byte};
use core::ffi::c_int;

use super::collection::{Collection, collection};
use super::compile::seen_endbrace;
use super::escape::{percent_atom, z_atom};
use super::literal::{class_shorthand, is_class_shorthand, literal_run, previous_substitute};
use super::op::BtOp;
use super::piece::reg;
use crate::regexp::{
    CompileError, Compiled, HASLOOKBH, HASNL, HASWIDTH, MAGIC_ALL, MAGIC_ON, NL, REG_PAREN, SIMPLE,
    SPSTART, Spelled, WORST, getchr, spelled, unmagic,
};

/// `\%[abc]` compiles each of its members as a single atom, so a member that
/// is itself a group or an alternation cannot work. Refuses with E369 when
/// we are inside one.
pub(crate) fn denied_in_optional_sequence(rc: &RegCompiler) -> Compiled {
    if rc.one_exactly == 0 {
        return Ok(());
    }
    Err(CompileError::InvalidInOptional(spelled(rc)))
}

/// Parse one atom, emit its nodes and describe it in `*flagp`.
///
pub(crate) fn regatom(rc: &mut RegCompiler, flagp: &mut c_int) -> Compiled<Node> {
    // `\%23l` restores the "still at the start of the pattern" flag, because
    // a position assertion consumes no input; [`percent_atom`] needs the
    // value from before this atom was read.
    let save_prev_at_start = rc.prev_at_start;
    *flagp = WORST;
    let mut c = getchr(rc);

    // `\_x` is "x, or a line break". The atom is built as usual and `ADD_NL`
    // shifts its opcode to the newline-accepting variant of itself.
    if c == M_UNDERSCORE {
        c = unmagic(getchr(rc));
        return match low_byte(c) {
            b'^' => Ok(rc.code.node(BtOp::Bol)),
            b'$' => {
                rc.had_eol = 1;
                Ok(rc.code.node(BtOp::Eol))
            }
            _ => {
                *flagp |= HASNL;
                if c == c_int::from(b'[') {
                    bracketed(rc, flagp, true, c)
                } else {
                    class_shorthand(rc, flagp, c, true)
                }
            }
        };
    }

    match c {
        M_CARET => Ok(rc.code.node(BtOp::Bol)),
        M_DOLLAR => {
            rc.had_eol = 1;
            Ok(rc.code.node(BtOp::Eol))
        }
        M_LT => Ok(rc.code.node(BtOp::Bow)),
        M_GT => Ok(rc.code.node(BtOp::Eow)),

        // `\n` is a line break, except in a string match, where there are no
        // lines and it is just the byte.
        M_N => {
            if rc.string_match != 0 {
                let ret = rc.code.node(BtOp::Exactly);
                rc.code.byte(NL);
                rc.code.byte(b'\0');
                *flagp |= HASWIDTH | SIMPLE;
                Ok(ret)
            } else {
                let ret = rc.code.node(BtOp::Newl);
                *flagp |= HASWIDTH | HASNL;
                Ok(ret)
            }
        }

        M_PAREN_OPEN => {
            denied_in_optional_sequence(rc)?;
            let mut flags = 0;
            let ret = reg(rc, REG_PAREN, &mut flags)?;
            *flagp |= flags & (HASWIDTH | SPSTART | HASNL | HASLOOKBH);
            Ok(ret)
        }

        // These end an alternative, so `regconcat` should already have
        // stopped; reaching one here means the parser lost track.
        0 | M_BAR | M_AMP | M_PAREN_CLOSE => {
            denied_in_optional_sequence(rc)?;
            Err(CompileError::Internal)
        }

        // A multi with no atom in front of it.
        M_EQUAL | M_QUESTION | M_PLUS | M_AT | M_BRACE | M_STAR => {
            let c = unmagic(c);
            // As in E61: `*` is magic one level sooner than the rest, so the
            // backslash this message shows is decided by a looser test.
            let bare = if c == c_int::from(b'*') {
                rc.magic >= MAGIC_ON
            } else {
                rc.magic == MAGIC_ALL
            };
            Err(CompileError::FollowsNothing(
                Spelled::bare_if(bare),
                low_byte(c),
            ))
        }

        M_TILDE => previous_substitute(rc, flagp),

        M_1..=M_9 => {
            let refnum = group_index(c - M_0);
            seen_endbrace(rc, refnum)?;
            Ok(rc.code.node(BtOp::BACKREF[refnum]))
        }

        M_Z => z_atom(rc, flagp),
        M_PERCENT => percent_atom(rc, flagp, save_prev_at_start),
        M_BRACKET => bracketed(rc, flagp, false, c),

        // `\d`, `\w`, `\s`, … in their magic form.
        c if is_class_shorthand(c) => class_shorthand(rc, flagp, c, false),

        _ => literal_run(rc, flagp, c),
    }
}

/// A `[` that may open a collection. If it does not close, it is an ordinary
/// character — unless 'regexpengine' strictness is on, where the missing `]`
/// is an error.
fn bracketed(
    rc: &mut RegCompiler,
    flagp: &mut c_int,
    crosses_lines: bool,
    c: c_int,
) -> Compiled<Node> {
    match collection(rc, flagp, crosses_lines)? {
        Collection::Node(node) => Ok(node),
        Collection::Literal => literal_run(rc, flagp, c),
    }
}

/// A `\\1`..`\\9` reference number as an index into [`BtOp::BACKREF`]. The
/// parser only ever passes one through `M_1..=M_9`.
fn group_index(refnum: c_int) -> usize {
    usize::try_from(refnum).expect("a back-reference number")
}
