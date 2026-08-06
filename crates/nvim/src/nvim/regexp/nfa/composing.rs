//! `\%C` and a pattern character that carries combining marks: an
//! `NFA_COMPOSING` group, which matches one whole grapheme.
//!
//! The group holds the base character and each combining mark as its own
//! state. Matching it means checking that every mark the group names is
//! somewhere in the grapheme at the input — in any order, which is why this
//! collects the input's marks first and then looks each state's up.

#![deny(unsafe_op_in_unsafe_fn)]

use core::ffi::{c_char, c_int};

use crate::src::nvim::mbyte::{utf_char2len, utf_iscomposing_legacy, utf_ptr2char};
use crate::src::nvim::regexp::{NFA_END_COMPOSING, nfa_state_T, rex};
use crate::src::nvim::types::MAX_MCO;

/// Does the grapheme at the input match the group whose first member state
/// is `sta`? `clen` is the grapheme's encoded length.
///
/// # Safety
///
/// `sta` must be a member state of a live `NFA_COMPOSING` group, and the
/// match context must be live.
pub(crate) unsafe fn matches_composing(
    mut sta: *mut nfa_state_T,
    curc: c_int,
    clen: c_int,
) -> bool {
    unsafe {
        let mut mc = curc;
        let mut len = 0;
        // A group whose first member is itself a combining character has no
        // base character of its own: `\%C` matched one already.
        if utf_iscomposing_legacy((*sta).c) {
            len += utf_char2len(mc);
        }

        if (*rex.ptr()).reg_icombine && len == 0 {
            // 'regexpengine' combining-insensitive: only the base character
            // has to match, and the marks are skipped over.
            let matched = (*sta).c == curc;
            return matched;
        }
        if len == 0 && mc != (*sta).c {
            return false;
        }
        if len == 0 {
            // The base character matched; the marks follow it.
            len += utf_char2len(mc);
            sta = (*sta).out;
        }

        // The marks actually present at the input, capped at what a
        // grapheme may carry.
        let mut marks = [0; MAX_MCO as usize];
        let mut count = 0;
        while len < clen {
            mc = utf_ptr2char(((*rex.ptr()).input as *mut c_char).offset(len as isize));
            marks[count] = mc;
            count += 1;
            len += utf_char2len(mc);
            if count == MAX_MCO as usize {
                break;
            }
        }

        // Every mark the group names has to be one of them.
        while (*sta).c != NFA_END_COMPOSING {
            if !marks[..count].contains(&(*sta).c) {
                return false;
            }
            sta = (*sta).out;
        }
        true
    }
}
