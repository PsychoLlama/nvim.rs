//! `\%C` and a pattern character that carries combining marks: an
//! `NFA_COMPOSING` group, which matches one whole grapheme.
//!
//! The group holds the base character and each combining mark as its own
//! state. Matching it means checking that every mark the group names is
//! somewhere in the grapheme at the input — in any order, which is why this
//! collects the input's marks first and then looks each state's up.

#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use crate::regexp::NfaOp;
use core::ffi::c_int;

use super::program::{StateId, States};
use crate::mbyte::{utf_char2len, utf_iscomposing_legacy};
use crate::regexp::Rex;
use crate::types::MAX_MCO;

/// Does the grapheme at the input match the group whose first member state
/// is `sta`, a member of an `NFA_COMPOSING` group of `prog`? `clen` is the
/// grapheme's encoded length.
pub(crate) fn matches_composing(
    rex: Rex,
    prog: States,
    mut sta: StateId,
    curc: c_int,
    clen: c_int,
) -> bool {
    let mut mc = curc;
    let mut len = 0;
    // A group whose first member is itself a combining character has no
    // base character of its own: `\%C` matched one already.
    if utf_iscomposing_legacy(prog.op(sta)) {
        len += utf_char2len(mc);
    }

    if rex.reg_icombine() && len == 0 {
        // 'regexpengine' combining-insensitive: only the base character
        // has to match, and the marks are skipped over.
        return prog.op(sta) == curc;
    }
    if len == 0 && mc != prog.op(sta) {
        return false;
    }
    if len == 0 {
        // The base character matched; the marks follow it.
        len += utf_char2len(mc);
        sta = prog.out(sta);
    }

    // The marks actually present at the input, capped at what a
    // grapheme may carry.
    let mut marks = [0; MAX_MCO as usize];
    let mut count = 0;
    while len < clen {
        mc = rex.char_at_offset(len);
        marks[count] = mc;
        count += 1;
        len += utf_char2len(mc);
        if count == MAX_MCO as usize {
            break;
        }
    }

    // Every mark the group names has to be one of them.
    while prog.op(sta) != NfaOp::EndComposing.code() {
        if !marks[..count].contains(&prog.op(sta)) {
            return false;
        }
        sta = prog.out(sta);
    }
    true
}
