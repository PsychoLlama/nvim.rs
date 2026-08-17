//! Choosing which entry to go to next.
//!
//! [`ex_cc`] is `:cc`/`:ll`, [`ex_cnext`] the `:cnext`/`:cprev`/`:cfirst`
//! family, and [`ex_cbelow`] the position-relative
//! `:cabove`/`:cbelow`/`:cbefore`/`:cafter`, which need the adjacent-entry
//! search in this file to decide what "next" means relative to the cursor.
//!
//! All three end in `qf_jump`, which takes the *number* of an entry, so the
//! search here answers a number rather than an entry.

#![deny(unsafe_op_in_unsafe_fn)]

use super::*;
use crate::types::{
    CMD_cNext, CMD_cNfile, CMD_cabove, CMD_cafter, CMD_cbefore, CMD_cbelow, CMD_cc, CMD_cdo,
    CMD_cfdo, CMD_cfirst, CMD_cnfile, CMD_cpfile, CMD_cprevious, CMD_crewind, CMD_lNext,
    CMD_lNfile, CMD_labove, CMD_lafter, CMD_lbelow, CMD_ldo, CMD_lfdo, CMD_lfirst, CMD_ll,
    CMD_lnfile, CMD_lpfile, CMD_lprevious, CMD_lrewind,
};
use core::cmp::Ordering;
use core::ffi::{c_char, c_int};

/// One entry of a list together with its number, which the adjacency search
/// tracks in step with the entry it walks to.
#[derive(Clone, Copy)]
struct At {
    entry: *mut qfline_T,
    /// The entry's position in the list, counted from 1.
    nr: c_int,
}

/// `:cc`, `:ll`, `:crewind`, `:cfirst`, `:clast` and their `:l…` twins,
/// plus `:cdo`/`:cfdo`, which start by jumping to the entry they run on.
///
/// # Safety
///
/// `eap` must be a live command.
pub unsafe fn ex_cc(eap: *mut exarg_T) {
    // SAFETY: forwarded from the caller.
    unsafe {
        let qi = qf_cmd_get_stack(eap, true);
        if qi.is_null() {
            return;
        }

        let mut errornr = if (*eap).addr_count > 0 {
            (*eap).line2 as c_int
        } else {
            match (*eap).cmdidx {
                // The current entry.
                CMD_cc | CMD_ll => 0,
                CMD_crewind | CMD_lrewind | CMD_cfirst | CMD_lfirst => 1,
                // :clast/:llast: past the end, which qf_jump clamps.
                _ => 32767,
            }
        };

        // :cdo/:ldo jump to the nth valid entry, :cfdo/:lfdo to the first
        // valid entry of the nth file.
        let is_do = matches!((*eap).cmdidx, CMD_cdo | CMD_ldo | CMD_cfdo | CMD_lfdo);
        if is_do {
            let n = if (*eap).addr_count > 0 {
                debug_assert!((*eap).line1 >= 0);
                (*eap).line1 as size_t
            } else {
                1
            };
            let per_file = matches!((*eap).cmdidx, CMD_cfdo | CMD_lfdo);
            let valid_entry = qf_get_nth_valid_entry(qf_get_curlist(qi), n, per_file);
            debug_assert!(valid_entry <= c_int::MAX as size_t);
            errornr = valid_entry as c_int;
        }

        qf_jump(qi, 0, errornr, (*eap).forceit);
    }
}

/// `:cnext`, `:cprevious`, `:cnfile`, `:cpfile` and their `:l…` twins, plus
/// the `:cdo`/`:cfdo` family's step to the next entry or file.
///
/// # Safety
///
/// `eap` must be a live command.
pub unsafe fn ex_cnext(eap: *mut exarg_T) {
    // SAFETY: forwarded from the caller.
    unsafe {
        let qi = qf_cmd_get_stack(eap, true);
        if qi.is_null() {
            return;
        }

        // A count says how many entries to move — except for the :cdo
        // family, whose count is the entry it started at.
        let is_do = matches!((*eap).cmdidx, CMD_cdo | CMD_ldo | CMD_cfdo | CMD_lfdo);
        let errornr = if (*eap).addr_count > 0 && !is_do {
            (*eap).line2 as c_int
        } else {
            1
        };

        // Depending on the command, jump to either the next or the previous
        // entry, or to one in the next or previous file.
        let dir = match (*eap).cmdidx {
            CMD_cprevious | CMD_lprevious | CMD_cNext | CMD_lNext => BACKWARD,
            CMD_cnfile | CMD_lnfile | CMD_cfdo | CMD_lfdo => FORWARD_FILE,
            CMD_cpfile | CMD_lpfile | CMD_cNfile | CMD_lNfile => BACKWARD_FILE,
            // CMD_cnext, CMD_lnext, CMD_cdo, CMD_ldo and anything else.
            _ => FORWARD,
        };

        qf_jump(qi, dir, errornr, (*eap).forceit);
    }
}

/// The first entry of the list that belongs to buffer `bnr`.
///
/// # Safety
///
/// `qfl` must be a live list.
unsafe fn first_entry_in_buf(qfl: *mut qf_list_T, bnr: c_int) -> Option<At> {
    // SAFETY: forwarded from the caller.
    unsafe {
        let mut at = At {
            entry: (*qfl).qf_start,
            nr: 1,
        };
        while !got_int.get() && at.nr <= (*qfl).qf_count && !at.entry.is_null() {
            if (*at.entry).qf_fnum == bnr {
                return Some(at);
            }
            at.nr += 1;
            at.entry = (*at.entry).qf_next;
        }
        None
    }
}

/// The first entry on the same line of the same file as `at`.
///
/// The entries of a list are in line order, so the run of entries sharing a
/// line is contiguous.
///
/// # Safety
///
/// `at.entry` must be a live entry.
unsafe fn first_entry_on_line(mut at: At) -> At {
    // SAFETY: forwarded from the caller.
    unsafe {
        while !got_int.get() && !(*at.entry).qf_prev.is_null() {
            let prev = (*at.entry).qf_prev;
            if (*prev).qf_fnum != (*at.entry).qf_fnum || (*prev).qf_lnum != (*at.entry).qf_lnum {
                break;
            }
            at = At {
                entry: prev,
                nr: at.nr - 1,
            };
        }
        at
    }
}

/// The last entry on the same line of the same file as `at`.
///
/// # Safety
///
/// `at.entry` must be a live entry.
unsafe fn last_entry_on_line(mut at: At) -> At {
    // SAFETY: forwarded from the caller.
    unsafe {
        while !got_int.get() && !(*at.entry).qf_next.is_null() {
            let next = (*at.entry).qf_next;
            if (*next).qf_fnum != (*at.entry).qf_fnum || (*next).qf_lnum != (*at.entry).qf_lnum {
                break;
            }
            at = At {
                entry: next,
                nr: at.nr + 1,
            };
        }
        at
    }
}

/// Where an entry sits relative to a position: `Greater` is after it.
///
/// With `linewise` the column is not compared at all, which is how
/// `:cabove`/`:cbelow` treat every entry on a line as one.
///
/// # Safety
///
/// `qfp` and `pos` must be live.
unsafe fn compare_to_pos(qfp: *const qfline_T, pos: *const pos_T, linewise: bool) -> Ordering {
    // SAFETY: forwarded from the caller.
    unsafe {
        let cols = if linewise {
            (0, 0)
        } else {
            ((*qfp).qf_col, (*pos).col)
        };
        ((*qfp).qf_lnum, cols.0).cmp(&((*pos).lnum, cols.1))
    }
}

/// The first entry of buffer `bnr` after `pos`, starting the walk at `at`,
/// which must be the buffer's first entry.
///
/// # Safety
///
/// `at.entry` must be a live entry and `pos` a live position.
unsafe fn entry_after_pos(bnr: c_int, pos: *const pos_T, linewise: bool, mut at: At) -> Option<At> {
    // SAFETY: forwarded from the caller.
    unsafe {
        if compare_to_pos(at.entry, pos, linewise) == Ordering::Greater {
            // The buffer's first entry is already after the position.
            return Some(at);
        }
        // Walk past the entries on or before the position; the first one
        // that is not is the answer, and running out of them means there is
        // none.
        loop {
            let next = (*at.entry).qf_next;
            if next.is_null() || (*next).qf_fnum != bnr {
                return None;
            }
            at = At {
                entry: next,
                nr: at.nr + 1,
            };
            if compare_to_pos(next, pos, linewise) == Ordering::Greater {
                return Some(at);
            }
        }
    }
}

/// The last entry of buffer `bnr` before `pos`, starting the walk at `at`,
/// which must be the buffer's first entry.
///
/// # Safety
///
/// `at.entry` must be a live entry and `pos` a live position.
unsafe fn entry_before_pos(
    bnr: c_int,
    pos: *const pos_T,
    linewise: bool,
    mut at: At,
) -> Option<At> {
    // SAFETY: forwarded from the caller.
    unsafe {
        while !(*at.entry).qf_next.is_null() {
            let next = (*at.entry).qf_next;
            if (*next).qf_fnum != bnr || compare_to_pos(next, pos, linewise) != Ordering::Less {
                break;
            }
            at = At {
                entry: next,
                nr: at.nr + 1,
            };
        }
        if compare_to_pos(at.entry, pos, linewise) != Ordering::Less {
            return None;
        }
        if linewise {
            // Entries on one line count as one, so answer the first.
            at = first_entry_on_line(at);
        }
        Some(at)
    }
}

/// The entry of buffer `bnr` closest to `pos` in the direction `dir`.
///
/// # Safety
///
/// `qfl` must be a live list and `pos` a live position.
unsafe fn closest_entry(
    qfl: *mut qf_list_T,
    bnr: c_int,
    pos: *const pos_T,
    dir: Direction,
    linewise: bool,
) -> Option<At> {
    // SAFETY: forwarded from the caller.
    unsafe {
        let first = first_entry_in_buf(qfl, bnr)?;
        if dir == FORWARD {
            entry_after_pos(bnr, pos, linewise, first)
        } else {
            entry_before_pos(bnr, pos, linewise, first)
        }
    }
}

/// The number of the `n`th entry of the same file below `at`, or of the
/// last one there is.
///
/// # Safety
///
/// `at.entry` must be a live entry.
unsafe fn nth_entry_below(mut at: At, n: linenr_T, linewise: bool) -> c_int {
    // SAFETY: forwarded from the caller.
    unsafe {
        let mut left = n;
        while left > 0 && !got_int.get() {
            left -= 1;
            let first_nr = at.nr;
            if linewise {
                // Treat all the entries on one line of this file as one.
                at = last_entry_on_line(at);
            }
            let next = (*at.entry).qf_next;
            if next.is_null() || (*next).qf_fnum != (*at.entry).qf_fnum {
                if linewise {
                    at.nr = first_nr;
                }
                break;
            }
            at = At {
                entry: next,
                nr: at.nr + 1,
            };
        }
        at.nr
    }
}

/// The number of the `n`th entry of the same file above `at`, or of the
/// first one there is.
///
/// # Safety
///
/// `at.entry` must be a live entry.
unsafe fn nth_entry_above(mut at: At, n: linenr_T, linewise: bool) -> c_int {
    // SAFETY: forwarded from the caller.
    unsafe {
        let mut left = n;
        while left > 0 && !got_int.get() {
            left -= 1;
            let prev = (*at.entry).qf_prev;
            if prev.is_null() || (*prev).qf_fnum != (*at.entry).qf_fnum {
                break;
            }
            at = At {
                entry: prev,
                nr: at.nr - 1,
            };
            if linewise {
                at = first_entry_on_line(at);
            }
        }
        at.nr
    }
}

/// The number of the `n`th entry adjacent to `pos` in buffer `bnr`, or 0
/// when there is none.
///
/// # Safety
///
/// `qfl` must be a live list and `pos` a live position.
unsafe fn nth_adjacent_entry(
    qfl: *mut qf_list_T,
    bnr: c_int,
    pos: *const pos_T,
    n: linenr_T,
    dir: Direction,
    linewise: bool,
) -> c_int {
    // SAFETY: forwarded from the caller.
    unsafe {
        let Some(at) = closest_entry(qfl, bnr, pos, dir, linewise) else {
            return 0;
        };
        // The closest entry is the first one; a count asks for further ones
        // in the same file.
        if n - 1 > 0 {
            if dir == FORWARD {
                return nth_entry_below(at, n - 1, linewise);
            }
            return nth_entry_above(at, n - 1, linewise);
        }
        at.nr
    }
}

/// `:cabove`, `:cbelow`, `:cbefore`, `:cafter` and their `:l…` twins: jump
/// to the entry of the current file nearest the cursor.
///
/// `:cabove`/`:cbelow` work in whole lines, `:cbefore`/`:cafter` in
/// line-and-column positions.
///
/// # Safety
///
/// `eap` must be a live command.
pub unsafe fn ex_cbelow(eap: *mut exarg_T) {
    // SAFETY: forwarded from the caller.
    unsafe {
        if (*eap).addr_count > 0 && (*eap).line2 <= 0 {
            emsg(gettext(&raw const e_invrange as *const c_char));
            return;
        }

        // Does the current buffer have any entry of the right kind?
        let quickfix = matches!(
            (*eap).cmdidx,
            CMD_cabove | CMD_cbelow | CMD_cbefore | CMD_cafter
        );
        let buf_has_flag = if quickfix {
            BUF_HAS_QF_ENTRY
        } else {
            BUF_HAS_LL_ENTRY
        };
        if (*curbuf.get()).b_has_qf_entry & buf_has_flag == 0 {
            emsg(gettext(&raw const e_no_errors as *const c_char));
            return;
        }

        let qi = qf_cmd_get_stack(eap, true);
        if qi.is_null() {
            return;
        }
        let qfl = qf_get_curlist(qi);
        if !qf_list_has_valid_entries(qfl) {
            emsg(gettext(&raw const e_no_errors as *const c_char));
            return;
        }

        let dir = if matches!(
            (*eap).cmdidx,
            CMD_cbelow | CMD_lbelow | CMD_cafter | CMD_lafter
        ) {
            FORWARD
        } else {
            BACKWARD
        };
        let linewise = matches!(
            (*eap).cmdidx,
            CMD_cbelow | CMD_lbelow | CMD_cabove | CMD_labove
        );

        let mut pos = (*curwin.get()).w_cursor;
        // An entry's column is 1 based where the cursor's is 0 based.
        pos.col += 1;
        let errornr = nth_adjacent_entry(
            qfl,
            (*curbuf.get()).handle,
            &raw const pos,
            if (*eap).addr_count > 0 {
                (*eap).line2
            } else {
                0
            },
            dir,
            linewise,
        );

        if errornr > 0 {
            qf_jump(qi, 0, errornr, false as c_int);
        } else {
            emsg(gettext(E_NO_MORE_ITEMS.as_ptr()));
        }
    }
}
