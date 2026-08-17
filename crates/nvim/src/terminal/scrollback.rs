//! The lines that have scrolled off the top of a `:terminal`.
//!
//! The emulator's screen holds only as many rows as the window is tall.
//! Everything above that lives here, oldest last, capped by `'scrollback'`.
//! vterm hands rows over as they scroll away ([`term_sb_push`]) and asks for
//! them back when the screen grows or leaves the alternate screen
//! ([`term_sb_pop`]); the refresh path then mirrors what is stored into the
//! buffer's lines.
//!
//! Two numbers connect this to the buffer. `pending` counts rows pushed but
//! not yet appended to the buffer, and `deleted` counts rows evicted over
//! the terminal's whole life — buffer line numbers are only meaningful
//! relative to it, which is why [`super::row_to_linenr`] exists.

#![deny(unsafe_op_in_unsafe_fn)]

use crate::change::{appended_lines_buf, deleted_lines_buf};
use crate::grid::{MAX_SCHAR_SIZE, schar_get_adv};
use crate::mark::mark_adjust_buf;
use crate::memline::{ml_append_buf, ml_delete_buf};
use crate::types::{
    OptInt, VTermColor, VTermPos, VTermScreenCell, VTermScreenCellAttrs, buf_T, colnr_T, linenr_T,
    schar_T,
};
use crate::vterm::screen::vterm_screen_get_cell;
use crate::winlayer::Buf;
use core::ffi::{c_char, c_int, c_void};

use super::refresh::invalidate_terminal;
use super::{NUL, SB_MAX, Term, kExtmarkUndo, kMarkAdjustTerm};
use crate::pos::MAXLNUM;

/// A cell holding nothing. vterm reports an empty cell as a zero `schar`;
/// the width still has to be 1 or the row scan below would not advance.
fn blank_cell() -> VTermScreenCell {
    VTermScreenCell {
        schar: 0 as schar_T,
        width: 1,
        attrs: VTermScreenCellAttrs {
            bold_underline_italic_blink_reverse_conceal_strike_font_dwl_dhl_small_baseline_dim_overline: [0; 3],
            c2rust_padding: [0; 1],
        },
        fg: VTermColor { type_0: 0 },
        bg: VTermColor { type_0: 0 },
        uri: 0,
    }
}

/// Size the scrollback from `'scrollback'`, if it has not been sized yet.
///
/// Returns false only when there is no buffer to read the option from,
/// which happens if the buffer was wiped while the terminal still lives.
pub fn term_may_alloc_scrollback(mut term: Term, buf: Option<Buf>) -> bool {
    if term.sb.is_sized() {
        return true;
    }
    let Some(buf) = buf.or_else(|| term.buf()) else {
        return false;
    };
    term.sb.set_capacity(scrollback_limit(buf));
    true
}

/// `'scrollback'` as a row count. The option's "unlimited" spelling is a
/// negative value, which stands for a cap large enough never to be reached.
fn scrollback_limit(mut buf: Buf) -> usize {
    if buf.b_p_scbk < 1 as OptInt {
        buf.b_p_scbk = SB_MAX as OptInt;
    }
    buf.b_p_scbk as usize
}

pub unsafe extern "C" fn term_sb_push(
    cols: c_int,
    cells: *const VTermScreenCell,
    data: *mut c_void,
) -> c_int {
    // SAFETY: vterm hands back the terminal registered alongside this table.
    let mut term = unsafe { Term::new(data.cast()) };
    // SAFETY: `cells` points at `cols` cells vterm owns.
    let row = unsafe { ::core::slice::from_raw_parts(cells, cols as usize) };
    if !term_may_alloc_scrollback(term, None) {
        return 0;
    }
    term.sb.push(row);
    if !term.synchronized_output {
        invalidate_terminal(term, None);
    }
    1
}

pub unsafe extern "C" fn term_sb_pop(
    cols: c_int,
    cells: *mut VTermScreenCell,
    data: *mut c_void,
) -> c_int {
    // SAFETY: as above.
    let mut term = unsafe { Term::new(data.cast()) };
    // SAFETY: `cells` is vterm's own row of `cols` cells, to fill in.
    let row = unsafe { ::core::slice::from_raw_parts_mut(cells, cols as usize) };
    let mut old_height = term.old_height;
    if !term.sb.pop(row, &mut old_height) {
        return 0;
    }
    term.old_height = old_height;
    if !term.synchronized_output {
        invalidate_terminal(term, None);
    }
    1
}

pub unsafe extern "C" fn term_sb_clear(data: *mut c_void) -> c_int {
    // SAFETY: vterm hands back the terminal registered alongside this table.
    let mut term = unsafe { Term::new(data.cast()) };
    // On the alternate screen the scrollback belongs to the screen
    // underneath, which is about to be restored; clearing it there would
    // lose history the program never touched.
    if term.in_altscreen || !term.sb.is_sized() || term.sb.is_empty() {
        return 1;
    }
    term.sb.clear();
    invalidate_terminal(term, None);
    1
}

/// Render row `row` of the screen into the terminal's line buffer.
///
/// Negative rows come from the scrollback, counting back from -1. Cells
/// vterm reports as empty become spaces, and trailing spaces are dropped by
/// terminating at the last cell that actually held something.
pub fn fetch_row(mut term: Term, row: c_int, end_col: c_int) {
    // Worst case is one maximum-length grapheme cluster per column, plus
    // the terminator. C sized this buffer once, at 8191 bytes, which a wide
    // enough terminal full of clusters overruns.
    let needed = end_col.max(0) as usize * MAX_SCHAR_SIZE as usize + 1;
    if term.textbuf.len() < needed {
        term.textbuf.resize(needed, 0);
    }
    // Stable for the whole loop: nothing below touches `textbuf`.
    let start = term.textbuf.as_mut_ptr();
    let mut ptr = start;
    let mut line_len = 0usize;
    let mut col = 0;
    while col < end_col {
        let mut cell = blank_cell();
        fetch_cell(term, row, col, &mut cell);
        if cell.schar != 0 {
            // SAFETY: `textbuf` holds room for a maximum-length cluster at
            // every column, and `ptr` has advanced one cluster per column.
            unsafe { schar_get_adv(&raw mut ptr, cell.schar) };
            // SAFETY: as above; both pointers are into `textbuf`.
            line_len = unsafe { ptr.offset_from(start) } as usize;
        } else {
            // Written but not counted, so that trailing blanks are dropped
            // by terminating at the last cell with content.
            //
            // SAFETY: as above.
            unsafe { *ptr = b' ' as c_char };
            // SAFETY: as above.
            ptr = unsafe { ptr.add(1) };
        }
        col += cell.width as c_int;
    }
    // SAFETY: as above; `line_len` is at most what the loop wrote.
    unsafe { *start.add(line_len) = NUL as c_char };
}

/// One cell of the screen or of the scrollback.
///
/// Returns false for a scrollback cell past the end of a row that was
/// stored while the terminal was narrower; the cell is left blank.
pub fn fetch_cell(term: Term, row: c_int, col: c_int, cell: &mut VTermScreenCell) -> bool {
    if row >= 0 {
        let vts = term.vts;
        // SAFETY: the terminal's own screen, and a cell of the caller's.
        unsafe { vterm_screen_get_cell(vts, VTermPos { row, col }, cell) };
        return true;
    }
    let stored = term
        .sb
        .row((-row - 1) as usize)
        .and_then(|stored| stored.get(col as usize));
    match stored {
        Some(found) => {
            *cell = *found;
            true
        }
        None => {
            *cell = blank_cell();
            false
        }
    }
}

/// Bring the scrollback back within `'scrollback'` after the option changed
/// or rows were pushed.
///
/// Trimming deletes from the top of the buffer, so the marks that pointed
/// into those lines have to move with them.
pub fn adjust_scrollback(mut term: Term, buf: Buf) {
    let limit = scrollback_limit(buf);
    assert!(
        term.sb.pending() == 0,
        "scrollback trimmed while rows were still owed to the buffer"
    );
    if limit < term.sb.len() {
        let diff = term.sb.len() - limit;
        for _ in 0..diff {
            // SAFETY: a live buffer, deleting the line the row that is
            // about to be dropped was mirrored onto.
            unsafe { ml_delete_buf(buf.raw(), 1 as linenr_T, false) };
            term.sb.drop_oldest();
        }
        let (buf, diff) = (buf.raw(), diff as linenr_T);
        // SAFETY: as above; the marks that pointed into the deleted lines
        // move with them.
        unsafe { mark_adjust_term(buf, 1 as linenr_T, diff, -diff) };
        // SAFETY: as above, reporting what the deletion took away.
        unsafe { deleted_lines_buf(buf, 1 as linenr_T, diff) };
    }
    term.sb.set_capacity(limit);
}

/// `mark_adjust_buf` as this module always calls it: a deletion at the top
/// of a terminal buffer, running to the end of it.
///
/// # Safety
/// `buf` must be a live buffer.
unsafe fn mark_adjust_term(buf: *mut buf_T, line1: linenr_T, line2: linenr_T, amount: linenr_T) {
    let (end, after) = (MAXLNUM as linenr_T, true);
    let (mode, op) = (kMarkAdjustTerm, kExtmarkUndo);
    // SAFETY: the caller's promise.
    unsafe { mark_adjust_buf(buf, line1, line2, end, amount, after, mode, op) };
}

/// Mirror everything the scrollback gained or lost into the buffer's lines.
///
/// Reading is paused for the duration: appending lines can run autocommands,
/// and more terminal output arriving in the middle would be appended at the
/// wrong place.
pub fn refresh_scrollback(mut term: Term, buf: Buf) {
    let read_pause = term.opts.read_pause_cb.expect("non-null function pointer");
    let data = term.opts.data;
    // SAFETY: the callback the channel registered, taking the data it
    // registered with it.
    unsafe { read_pause(true, data) };

    // Rows evicted since the last refresh are gone from the buffer's top;
    // move the marks that were pointing at them.
    let mut deleted = (term.sb.deleted() - term.old_sb_deleted) as linenr_T;
    deleted = deleted.min(buf.line_count());
    // SAFETY: a live buffer.
    unsafe { mark_adjust_term(buf.raw(), 1 as linenr_T, deleted, -deleted) };
    term.old_sb_deleted = term.sb.deleted();

    let mut old_height = term.old_height;
    let (height, width) = term.size();

    while deleted > 0 && buf.line_count() > old_height as linenr_T {
        // SAFETY: a live buffer, deleting a line the scrollback no longer
        // holds.
        unsafe { ml_delete_buf(buf.raw(), 1 as linenr_T, false) };
        // SAFETY: as above, reporting what the deletion took away.
        unsafe { deleted_lines_buf(buf.raw(), 1 as linenr_T, 1 as linenr_T) };
        deleted -= 1;
    }
    old_height = old_height.min(buf.line_count() as c_int);

    // Each owed row is appended just above the rows that make up the
    // screen, which sit at the end of the buffer.
    while term.sb.pending() > 0 {
        fetch_row(term, -term.sb.pending(), width);
        let at = (buf.line_count() as c_int - old_height) as linenr_T;
        let text = term.textbuf.as_mut_ptr();
        // SAFETY: a live buffer, taking the row this terminal's own line
        // buffer holds.
        unsafe { ml_append_buf(buf.raw(), at, text, 0 as colnr_T, false) };
        // SAFETY: as above, reporting the line just appended.
        unsafe { appended_lines_buf(buf.raw(), at, 1 as linenr_T) };
        term.sb.mark_mirrored();
    }

    // Anything past the scrollback plus one screen is stale.
    let max_line_count = (term.sb.len() as c_int + height) as linenr_T;
    while buf.line_count() > max_line_count {
        let last = buf.line_count();
        // SAFETY: a live buffer, deleting its own last line.
        unsafe { ml_delete_buf(buf.raw(), last, false) };
        // SAFETY: as above, reporting what the deletion took away.
        unsafe { deleted_lines_buf(buf.raw(), buf.line_count(), 1 as linenr_T) };
    }

    adjust_scrollback(term, buf);
    let data = term.opts.data;
    // SAFETY: the callback the channel registered, as above.
    unsafe { read_pause(false, data) };
}
