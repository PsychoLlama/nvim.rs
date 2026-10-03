//! Choosing which entry to go to next.
//!
//! [`ex_cc`] is `:cc`/`:ll`, [`ex_cnext`] the `:cnext`/`:cprev`/`:cfirst`
//! family, and [`ex_cbelow`] the position-relative
//! `:cabove`/`:cbelow`/`:cbefore`/`:cafter`, which need the adjacent-entry
//! search in this file to decide what "next" means relative to the cursor.
//!
//! All three end in `qf_jump`, which takes the *number* of an entry, so the
//! search here answers a number rather than an entry.

#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use super::*;
use crate::types::CmdIdx;
use core::cmp::Ordering;
use core::ffi::c_int;

/// `:cc`, `:ll`, `:crewind`, `:cfirst`, `:clast` and their `:l…` twins,
/// plus `:cdo`/`:cfdo`, which start by jumping to the entry they run on.
pub fn ex_cc(excmd: &mut ExArg) {
    let Some(qi) = stack_for_cmd(excmd.cmdidx, true) else {
        return;
    };

    let mut errornr = if excmd.addr_count > 0 {
        excmd.line2 as c_int
    } else {
        match excmd.cmdidx {
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
        excmd.cmdidx,
        CmdIdx::cdo | CmdIdx::ldo | CmdIdx::cfdo | CmdIdx::lfdo
    );
    if is_do {
        let n = if excmd.addr_count > 0 {
            size_t::try_from(excmd.line1).expect("an ex range line is never negative")
        } else {
            1
        };
        let per_file = matches!(excmd.cmdidx, CmdIdx::cfdo | CmdIdx::lfdo);
        let valid_entry = qf_get_nth_valid_entry(qi.current_list(), n, per_file);
        errornr = c_int::try_from(valid_entry).expect("a quickfix list is shorter than INT_MAX");
    }

    qf_jump(qi, 0, errornr, excmd.forceit);
}

/// `:cnext`, `:cprevious`, `:cnfile`, `:cpfile` and their `:l…` twins, plus
/// the `:cdo`/`:cfdo` family's step to the next entry or file.
pub fn ex_cnext(excmd: &mut ExArg) {
    let Some(qi) = stack_for_cmd(excmd.cmdidx, true) else {
        return;
    };

    // A count says how many entries to move — except for the :cdo
    // family, whose count is the entry it started at.
    let is_do = matches!(
        excmd.cmdidx,
        CmdIdx::cdo | CmdIdx::ldo | CmdIdx::cfdo | CmdIdx::lfdo
    );
    let errornr = if excmd.addr_count > 0 && !is_do {
        excmd.line2 as c_int
    } else {
        1
    };

    // Depending on the command, jump to either the next or the previous
    // entry, or to one in the next or previous file.
    let dir = match excmd.cmdidx {
        CmdIdx::cprevious | CmdIdx::lprevious | CmdIdx::cNext | CmdIdx::lNext => BACKWARD,
        CmdIdx::cnfile | CmdIdx::lnfile | CmdIdx::cfdo | CmdIdx::lfdo => FORWARD_FILE,
        CmdIdx::cpfile | CmdIdx::lpfile | CmdIdx::cNfile | CmdIdx::lNfile => BACKWARD_FILE,
        // CmdIdx::cnext, CmdIdx::lnext, CmdIdx::cdo, CmdIdx::ldo and anything else.
        _ => FORWARD,
    };

    qf_jump(qi, dir, errornr, excmd.forceit);
}

/// The entries of one list, and the position the adjacency search compares
/// them with. Nothing in the search can run user code, so it borrows the
/// list. An entry is named by its position; its number is one more.
struct Adjacent<'a> {
    entries: &'a [QfEntry],
    bnr: c_int,
    pos: Pos,
    linewise: bool,
}

impl Adjacent<'_> {
    /// The entry after `at`, if there is one.
    fn next(&self, at: usize) -> Option<usize> {
        (at + 1 < self.entries.len()).then_some(at + 1)
    }

    /// The entry before `at`, if there is one.
    fn prev(&self, at: usize) -> Option<usize> {
        at.checked_sub(1)
    }

    /// Whether two entries are in the same file.
    fn same_file(&self, a: usize, b: usize) -> bool {
        self.entries[a].fnum == self.entries[b].fnum
    }

    /// Whether two entries are on the same line of the same file.
    fn same_line(&self, a: usize, b: usize) -> bool {
        self.same_file(a, b) && self.entries[a].lnum == self.entries[b].lnum
    }

    /// The first entry of the list that belongs to the buffer.
    fn first_in_buf(&self) -> Option<usize> {
        self.entries
            .iter()
            .take_while(|_| !got_int.get())
            .position(|entry| entry.fnum == self.bnr)
    }

    /// Where an entry sits relative to the position: `Greater` is after it.
    ///
    /// Linewise the column is not compared at all, which is how
    /// `:cabove`/`:cbelow` treat every entry on a line as one.
    fn compare(&self, at: usize) -> Ordering {
        let entry = &self.entries[at];
        let cols = if self.linewise {
            (0, 0)
        } else {
            (entry.col, self.pos.col)
        };
        (entry.lnum, cols.0).cmp(&(self.pos.lnum, cols.1))
    }

    /// The first entry on the same line of the same file as `at`.
    ///
    /// The entries of a list are in line order, so the run of entries
    /// sharing a line is contiguous.
    fn first_on_line(&self, mut at: usize) -> usize {
        while !got_int.get()
            && let Some(prev) = self.prev(at)
            && self.same_line(prev, at)
        {
            at = prev;
        }
        at
    }

    /// The last entry on the same line of the same file as `at`.
    fn last_on_line(&self, mut at: usize) -> usize {
        while !got_int.get()
            && let Some(next) = self.next(at)
            && self.same_line(next, at)
        {
            at = next;
        }
        at
    }

    /// The first entry of the buffer after the position, starting the walk
    /// at `at`, which must be the buffer's first entry.
    fn after_pos(&self, mut at: usize) -> Option<usize> {
        if self.compare(at) == Ordering::Greater {
            // The buffer's first entry is already after the position.
            return Some(at);
        }
        // Walk past the entries on or before the position; the first one
        // that is not is the answer, and running out of them means there is
        // none.
        loop {
            let next = self.next(at)?;
            if self.entries[next].fnum != self.bnr {
                return None;
            }
            at = next;
            if self.compare(next) == Ordering::Greater {
                return Some(at);
            }
        }
    }

    /// The last entry of the buffer before the position, starting the walk
    /// at `at`, which must be the buffer's first entry.
    fn before_pos(&self, mut at: usize) -> Option<usize> {
        while let Some(next) = self.next(at)
            && self.entries[next].fnum == self.bnr
            && self.compare(next) == Ordering::Less
        {
            at = next;
        }
        if self.compare(at) != Ordering::Less {
            return None;
        }
        if self.linewise {
            // Entries on one line count as one, so answer the first.
            at = self.first_on_line(at);
        }
        Some(at)
    }

    /// The entry of the buffer closest to the position in `dir`.
    fn closest(&self, dir: Direction) -> Option<usize> {
        let first = self.first_in_buf()?;
        if dir == FORWARD {
            self.after_pos(first)
        } else {
            self.before_pos(first)
        }
    }

    /// The `n`th entry of the same file below `at`, or the last one there
    /// is.
    fn nth_below(&self, mut at: usize, n: LineNr) -> usize {
        let mut left = n;
        while left > 0 && !got_int.get() {
            left -= 1;
            let first = at;
            if self.linewise {
                // Treat all the entries on one line of this file as one.
                at = self.last_on_line(at);
            }
            let Some(next) = self.next(at).filter(|&next| self.same_file(next, at)) else {
                if self.linewise {
                    at = first;
                }
                break;
            };
            at = next;
        }
        at
    }

    /// The `n`th entry of the same file above `at`, or the first one there
    /// is.
    fn nth_above(&self, mut at: usize, n: LineNr) -> usize {
        let mut left = n;
        while left > 0 && !got_int.get() {
            left -= 1;
            let Some(prev) = self.prev(at).filter(|&prev| self.same_file(prev, at)) else {
                break;
            };
            at = prev;
            if self.linewise {
                at = self.first_on_line(at);
            }
        }
        at
    }

    /// The number of the `n`th entry adjacent to the position, or 0 when
    /// there is none.
    fn nth(&self, n: LineNr, dir: Direction) -> c_int {
        let Some(closest) = self.closest(dir) else {
            return 0;
        };
        // The closest entry is the first one; a count asks for further ones
        // in the same file.
        let at = if n - 1 > 0 {
            if dir == FORWARD {
                self.nth_below(closest, n - 1)
            } else {
                self.nth_above(closest, n - 1)
            }
        } else {
            closest
        };
        c_int::try_from(at + 1).unwrap_or(c_int::MAX)
    }
}

/// `:cabove`, `:cbelow`, `:cbefore`, `:cafter` and their `:l…` twins: jump
/// to the entry of the current file nearest the cursor.
///
/// `:cabove`/`:cbelow` work in whole lines, `:cbefore`/`:cafter` in
/// line-and-column positions.
pub fn ex_cbelow(excmd: &mut ExArg) {
    if excmd.addr_count > 0 && excmd.line2 <= 0 {
        qf_emsg(e_invrange);
        return;
    }

    // Does the current buffer have any entry of the right kind?
    let quickfix = matches!(
        excmd.cmdidx,
        CmdIdx::cabove | CmdIdx::cbelow | CmdIdx::cbefore | CmdIdx::cafter
    );
    let buf_has_flag = if quickfix {
        BUF_HAS_QF_ENTRY
    } else {
        BUF_HAS_LL_ENTRY
    };
    if Buf::current().b_has_qf_entry & buf_has_flag == 0 {
        qf_emsg(e_no_errors);
        return;
    }

    let Some(qi) = stack_for_cmd(excmd.cmdidx, true) else {
        return;
    };
    if !qi.current_list().has_valid_entries() {
        qf_emsg(e_no_errors);
        return;
    }

    let dir = if matches!(
        excmd.cmdidx,
        CmdIdx::cbelow | CmdIdx::lbelow | CmdIdx::cafter | CmdIdx::lafter
    ) {
        FORWARD
    } else {
        BACKWARD
    };
    let linewise = matches!(
        excmd.cmdidx,
        CmdIdx::cbelow | CmdIdx::lbelow | CmdIdx::cabove | CmdIdx::labove
    );

    let mut pos = Win::current().w_cursor;
    // An entry's column is 1 based where the cursor's is 0 based.
    pos.col += 1;
    let n = if excmd.addr_count > 0 { excmd.line2 } else { 0 };
    let errornr = Adjacent {
        entries: &qi.current_list().entries,
        bnr: Buf::current().handle,
        pos,
        linewise,
    }
    .nth(n, dir);

    if errornr > 0 {
        qf_jump(qi, 0, errornr, false);
    } else {
        qf_emsg(E_NO_MORE_ITEMS);
    }
}
