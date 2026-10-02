//! One atom of a pattern: the smallest thing a multi can repeat.
//!
//! This is only the dispatch. The three families that need more than a node
//! or two live next door: [`super::collection`] for `[...]`,
//! [`super::escape`] for the `\z` and `\%` escapes, and [`super::literal`]
//! for a run of ordinary characters.

#![forbid(unsafe_code)]

use super::compile::Node;
use crate::regexp::RegCompiler;
use core::ffi::c_int;

use super::collection::{Collection, collection};
use super::compile::seen_endbrace;
use super::escape::{percent_atom, z_atom};
use super::literal::{class_shorthand, is_class_shorthand, literal_run, previous_substitute};
use super::op::BtOp;
use super::piece::reg;
use crate::regexp::state::rc_did_emsg;
use crate::regexp::{
    HASLOOKBH, HASNL, HASWIDTH, MAGIC_ALL, MAGIC_ON, NL, REG_PAREN, SIMPLE, SPSTART, WORST, getchr,
    magic, magic_prefix, unmagic,
};
use crate::semsg;
use crate::types::NUL;

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
pub(crate) fn denied_in_optional_sequence(rc: &RegCompiler) -> bool {
    if rc.one_exactly == 0 {
        return false;
    }
    let prefix = magic_prefix(rc);
    semsg!("E369: Invalid item in {prefix}%[]");
    rc_did_emsg.set(true);
    true
}

/// Parse one atom, emit its nodes and describe it in `*flagp`.
///
/// Returns null when an error has already been reported.
pub(crate) fn regatom(rc: &mut RegCompiler, flagp: &mut c_int) -> Option<Node> {
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
        return match c as u8 {
            b'^' => Some(rc.code.node(BtOp::Bol)),
            b'$' => {
                rc.had_eol = 1;
                Some(rc.code.node(BtOp::Eol))
            }
            _ => {
                *flagp |= HASNL;
                if c == b'[' as c_int {
                    bracketed(rc, flagp, true, c)
                } else {
                    class_shorthand(rc, flagp, c, true)
                }
            }
        };
    }

    match c {
        M_CARET => Some(rc.code.node(BtOp::Bol)),
        M_DOLLAR => {
            rc.had_eol = 1;
            Some(rc.code.node(BtOp::Eol))
        }
        M_LT => Some(rc.code.node(BtOp::Bow)),
        M_GT => Some(rc.code.node(BtOp::Eow)),

        // `\n` is a line break, except in a string match, where there are no
        // lines and it is just the byte.
        M_N => {
            if rc.string_match != 0 {
                let ret = rc.code.node(BtOp::Exactly);
                rc.code.byte(NL);
                rc.code.byte(NUL);
                *flagp |= HASWIDTH | SIMPLE;
                Some(ret)
            } else {
                let ret = rc.code.node(BtOp::Newl);
                *flagp |= HASWIDTH | HASNL;
                Some(ret)
            }
        }

        M_PAREN_OPEN => {
            if denied_in_optional_sequence(rc) {
                return None;
            }
            let mut flags = 0;
            let ret = reg(rc, REG_PAREN, &mut flags);
            if !ret.is_none() {
                *flagp |= flags & (HASWIDTH | SPSTART | HASNL | HASLOOKBH);
            }
            ret
        }

        // These end an alternative, so `regconcat` should already have
        // stopped; reaching one here means the parser lost track.
        NUL | M_BAR | M_AMP | M_PAREN_CLOSE => {
            if denied_in_optional_sequence(rc) {
                return None;
            }
            semsg!("E473: Internal error in regexp");
            rc_did_emsg.set(true);
            None
        }

        // A multi with no atom in front of it.
        M_EQUAL | M_QUESTION | M_PLUS | M_AT | M_BRACE | M_STAR => {
            let c = unmagic(c);
            // As in E61: `*` is magic one level sooner than the rest, so the
            // backslash this message shows is decided by a looser test.
            let bare = if c == b'*' as c_int {
                rc.magic >= MAGIC_ON
            } else {
                rc.magic == MAGIC_ALL
            };
            let prefix = if bare { "" } else { "\\" };
            let c = c as u8 as char;
            semsg!("E64: {prefix}{c} follows nothing");
            rc_did_emsg.set(true);
            None
        }

        M_TILDE => previous_substitute(rc, flagp),

        M_1..=M_9 => {
            let refnum = c - M_0;
            if !seen_endbrace(rc, refnum) {
                return None;
            }
            Some(rc.code.node(BtOp::BACKREF[group_index(refnum)]))
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
) -> Option<Node> {
    match collection(rc, flagp, crosses_lines) {
        Collection::Node(node) => Some(node),
        Collection::Failed => None,
        Collection::Literal => literal_run(rc, flagp, c),
    }
}

/// A `\\1`..`\\9` reference number as an index into [`BtOp::BACKREF`]. The
/// parser only ever passes one through `M_1..=M_9`.
fn group_index(refnum: c_int) -> usize {
    usize::try_from(refnum).expect("a back-reference number")
}
