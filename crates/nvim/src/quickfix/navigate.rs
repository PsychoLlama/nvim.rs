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
use crate::types::CmdIdx;
use core::cmp::Ordering;
use core::ffi::c_int;

/// One entry of a list together with its number, which the adjacency search
/// tracks in step with the entry it walks to.
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
    // SAFETY: the caller's promise -- a live `exarg_T`.
    let eap = unsafe { Ea::new(eap) };
    let Some(qi) = qf_cmd_stack(eap, true) else {
        return;
    };

    let mut errornr = if eap.addr_count > 0 {
        eap.line2 as c_int
    } else {
        match eap.cmdidx {
            // The current entry.
            CmdIdx::cc | CmdIdx::ll => 0,
            CmdIdx::crewind | CmdIdx::lrewind | CmdIdx::cfirst | CmdIdx::lfirst => 1,
            // :clast/:llast: past the end, which qf_jump clamps.
            _ => 32767,
        }
    };

    // :cdo/:ldo jump to the nth valid entry, :cfdo/:lfdo to the first
    // valid entry of the nth file.
    let is_do = matches!(
        eap.cmdidx,
        CmdIdx::cdo | CmdIdx::ldo | CmdIdx::cfdo | CmdIdx::lfdo
    );
    if is_do {
        let n = if eap.addr_count > 0 {
            debug_assert!(eap.line1 >= 0);
            eap.line1 as size_t
        } else {
            1
        };
        let per_file = matches!(eap.cmdidx, CmdIdx::cfdo | CmdIdx::lfdo);
        let valid_entry = qf_get_nth_valid_entry(qf_current_list(qi), n, per_file);
        debug_assert!(valid_entry <= c_int::MAX as size_t);
        errornr = valid_entry as c_int;
    }

    qf_goto(qi, 0, errornr, eap.forceit);
}

/// `:cnext`, `:cprevious`, `:cnfile`, `:cpfile` and their `:l…` twins, plus
/// the `:cdo`/`:cfdo` family's step to the next entry or file.
///
/// # Safety
///
/// `eap` must be a live command.
pub unsafe fn ex_cnext(eap: *mut exarg_T) {
    // SAFETY: the caller's promise -- a live `exarg_T`.
    let eap = unsafe { Ea::new(eap) };
    let Some(qi) = qf_cmd_stack(eap, true) else {
        return;
    };

    // A count says how many entries to move — except for the :cdo
    // family, whose count is the entry it started at.
    let is_do = matches!(
        eap.cmdidx,
        CmdIdx::cdo | CmdIdx::ldo | CmdIdx::cfdo | CmdIdx::lfdo
    );
    let errornr = if eap.addr_count > 0 && !is_do {
        eap.line2 as c_int
    } else {
        1
    };

    // Depending on the command, jump to either the next or the previous
    // entry, or to one in the next or previous file.
    let dir = match eap.cmdidx {
        CmdIdx::cprevious | CmdIdx::lprevious | CmdIdx::cNext | CmdIdx::lNext => BACKWARD,
        CmdIdx::cnfile | CmdIdx::lnfile | CmdIdx::cfdo | CmdIdx::lfdo => FORWARD_FILE,
        CmdIdx::cpfile | CmdIdx::lpfile | CmdIdx::cNfile | CmdIdx::lNfile => BACKWARD_FILE,
        // CmdIdx::cnext, CmdIdx::lnext, CmdIdx::cdo, CmdIdx::ldo and anything else.
        _ => FORWARD,
    };

    qf_goto(qi, dir, errornr, eap.forceit);
}

/// The first entry of the list that belongs to buffer `bnr`.
///
/// # Safety
///
/// `qfl` must be a live list.
unsafe fn first_entry_in_buf(qfl: *mut qf_list_T, bnr: c_int) -> Option<At> {
    // SAFETY: the caller's promise -- a live `qf_list_T`.
    let qfl = unsafe { Qfl::new(qfl) };
    // SAFETY: forwarded from the caller.
    let mut at = At {
        entry: qfl.qf_start,
        nr: 1,
    };
    while !got_int.get() && at.nr <= qfl.qf_count && !at.entry.is_null() {
        if unsafe { (*at.entry).qf_fnum } == bnr {
            return Some(at);
        }
        at.nr += 1;
        at.entry = unsafe { (*at.entry).qf_next };
    }
    None
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
    while !got_int.get() && !unsafe { (*at.entry).qf_prev.is_null() } {
        let prev = unsafe { (*at.entry).qf_prev };
        if unsafe { (*prev).qf_fnum } != unsafe { (*at.entry).qf_fnum }
            || unsafe { (*prev).qf_lnum } != unsafe { (*at.entry).qf_lnum }
        {
            break;
        }
        at = At {
            entry: prev,
            nr: at.nr - 1,
        };
    }
    at
}

/// The last entry on the same line of the same file as `at`.
///
/// # Safety
///
/// `at.entry` must be a live entry.
unsafe fn last_entry_on_line(mut at: At) -> At {
    // SAFETY: forwarded from the caller.
    while !got_int.get() && !unsafe { (*at.entry).qf_next.is_null() } {
        let next = unsafe { (*at.entry).qf_next };
        if unsafe { (*next).qf_fnum } != unsafe { (*at.entry).qf_fnum }
            || unsafe { (*next).qf_lnum } != unsafe { (*at.entry).qf_lnum }
        {
            break;
        }
        at = At {
            entry: next,
            nr: at.nr + 1,
        };
    }
    at
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
    // SAFETY: the caller's promise -- a live `qfline_T`.
    let qfp = unsafe { Qfe::new(qfp.cast_mut()) };
    // SAFETY: forwarded from the caller.
    let cols = if linewise {
        (0, 0)
    } else {
        (qfp.qf_col, unsafe { (*pos).col })
    };
    (qfp.qf_lnum, cols.0).cmp(&(unsafe { (*pos).lnum }, cols.1))
}

/// The first entry of buffer `bnr` after `pos`, starting the walk at `at`,
/// which must be the buffer's first entry.
///
/// # Safety
///
/// `at.entry` must be a live entry and `pos` a live position.
unsafe fn entry_after_pos(bnr: c_int, pos: *const pos_T, linewise: bool, mut at: At) -> Option<At> {
    // SAFETY: forwarded from the caller.
    if unsafe { compare_to_pos(at.entry, pos, linewise) } == Ordering::Greater {
        // The buffer's first entry is already after the position.
        return Some(at);
    }
    // Walk past the entries on or before the position; the first one
    // that is not is the answer, and running out of them means there is
    // none.
    loop {
        let next = unsafe { (*at.entry).qf_next };
        if next.is_null() || unsafe { (*next).qf_fnum } != bnr {
            return None;
        }
        at = At {
            entry: next,
            nr: at.nr + 1,
        };
        if unsafe { compare_to_pos(next, pos, linewise) } == Ordering::Greater {
            return Some(at);
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
    while !unsafe { (*at.entry).qf_next.is_null() } {
        let next = unsafe { (*at.entry).qf_next };
        if unsafe { (*next).qf_fnum } != bnr
            || unsafe { compare_to_pos(next, pos, linewise) } != Ordering::Less
        {
            break;
        }
        at = At {
            entry: next,
            nr: at.nr + 1,
        };
    }
    if unsafe { compare_to_pos(at.entry, pos, linewise) } != Ordering::Less {
        return None;
    }
    if linewise {
        // Entries on one line count as one, so answer the first.
        at = unsafe { first_entry_on_line(at) };
    }
    Some(at)
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
    let first = unsafe { first_entry_in_buf(qfl, bnr) }?;
    if dir == FORWARD {
        unsafe { entry_after_pos(bnr, pos, linewise, first) }
    } else {
        unsafe { entry_before_pos(bnr, pos, linewise, first) }
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
    let mut left = n;
    while left > 0 && !got_int.get() {
        left -= 1;
        let first_nr = at.nr;
        if linewise {
            // Treat all the entries on one line of this file as one.
            at = unsafe { last_entry_on_line(at) };
        }
        let next = unsafe { (*at.entry).qf_next };
        if next.is_null() || unsafe { (*next).qf_fnum } != unsafe { (*at.entry).qf_fnum } {
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

/// The number of the `n`th entry of the same file above `at`, or of the
/// first one there is.
///
/// # Safety
///
/// `at.entry` must be a live entry.
unsafe fn nth_entry_above(mut at: At, n: linenr_T, linewise: bool) -> c_int {
    // SAFETY: forwarded from the caller.
    let mut left = n;
    while left > 0 && !got_int.get() {
        left -= 1;
        let prev = unsafe { (*at.entry).qf_prev };
        if prev.is_null() || unsafe { (*prev).qf_fnum } != unsafe { (*at.entry).qf_fnum } {
            break;
        }
        at = At {
            entry: prev,
            nr: at.nr - 1,
        };
        if linewise {
            at = unsafe { first_entry_on_line(at) };
        }
    }
    at.nr
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
    let closest = unsafe { closest_entry(qfl, bnr, pos, dir, linewise) };
    let Some(at) = closest else {
        return 0;
    };
    // The closest entry is the first one; a count asks for further ones
    // in the same file.
    if n - 1 > 0 {
        if dir == FORWARD {
            return unsafe { nth_entry_below(at, n - 1, linewise) };
        }
        return unsafe { nth_entry_above(at, n - 1, linewise) };
    }
    at.nr
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
    // SAFETY: the caller's promise -- a live `exarg_T`.
    let eap = unsafe { Ea::new(eap) };
    // SAFETY: forwarded from the caller.
    if eap.addr_count > 0 && eap.line2 <= 0 {
        qf_emsg(e_invrange.as_ptr());
        return;
    }

    // Does the current buffer have any entry of the right kind?
    let quickfix = matches!(
        eap.cmdidx,
        CmdIdx::cabove | CmdIdx::cbelow | CmdIdx::cbefore | CmdIdx::cafter
    );
    let buf_has_flag = if quickfix {
        BUF_HAS_QF_ENTRY
    } else {
        BUF_HAS_LL_ENTRY
    };
    if cur_buf().b_has_qf_entry & buf_has_flag == 0 {
        qf_emsg(e_no_errors.as_ptr());
        return;
    }

    let Some(qi) = qf_cmd_stack(eap, true) else {
        return;
    };
    let qfl = qf_current_list(qi);
    if !unsafe { qf_list_has_valid_entries(qfl.raw().cast_const()) } {
        qf_emsg(e_no_errors.as_ptr());
        return;
    }

    let dir = if matches!(
        eap.cmdidx,
        CmdIdx::cbelow | CmdIdx::lbelow | CmdIdx::cafter | CmdIdx::lafter
    ) {
        FORWARD
    } else {
        BACKWARD
    };
    let linewise = matches!(
        eap.cmdidx,
        CmdIdx::cbelow | CmdIdx::lbelow | CmdIdx::cabove | CmdIdx::labove
    );

    let mut pos = cur_win().w_cursor;
    // An entry's column is 1 based where the cursor's is 0 based.
    pos.col += 1;
    let bnr2 = cur_buf().handle;
    let pos2 = &raw const pos;
    let n2 = if eap.addr_count > 0 { eap.line2 } else { 0 };
    let errornr = unsafe { nth_adjacent_entry(qfl.raw(), bnr2, pos2, n2, dir, linewise) };

    if errornr > 0 {
        qf_goto(qi, 0, errornr, false as c_int);
    } else {
        qf_emsg(E_NO_MORE_ITEMS.as_ptr());
    }
}
