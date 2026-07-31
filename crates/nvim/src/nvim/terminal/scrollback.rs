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

use crate::src::nvim::change::{appended_lines_buf, deleted_lines_buf};
use crate::src::nvim::grid::{MAX_SCHAR_SIZE, schar_get_adv};
use crate::src::nvim::mark::mark_adjust_buf;
use crate::src::nvim::memline::{ml_append_buf, ml_delete_buf};
use crate::src::nvim::types::{
    OptInt, Terminal, VTermColor, VTermPos, VTermScreenCell, VTermScreenCellAttrs, buf_T, colnr_T,
    linenr_T, schar_T,
};
use crate::src::nvim::vterm::screen::vterm_screen_get_cell;
use crate::src::nvim::vterm::vterm::vterm_get_size;

use super::refresh::invalidate_terminal;
use super::{MAXLNUM, NUL, SB_MAX, buf_for_handle, kExtmarkUndo, kMarkAdjustTerm};

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
pub unsafe fn term_may_alloc_scrollback(term: *mut Terminal, buf: *mut buf_T) -> bool {
    unsafe {
        if (*term).sb.is_sized() {
            return true;
        }
        let buf = if buf.is_null() {
            buf_for_handle((*term).buf_handle)
        } else {
            buf
        };
        if buf.is_null() {
            return false;
        }
        (*term).sb.set_capacity(scrollback_limit(buf));
        true
    }
}

/// `'scrollback'` as a row count. The option's "unlimited" spelling is a
/// negative value, which stands for a cap large enough never to be reached.
unsafe fn scrollback_limit(buf: *mut buf_T) -> usize {
    unsafe {
        if (*buf).b_p_scbk < 1 as OptInt {
            (*buf).b_p_scbk = SB_MAX as OptInt;
        }
        (*buf).b_p_scbk as usize
    }
}

pub unsafe extern "C" fn term_sb_push(
    cols: ::core::ffi::c_int,
    cells: *const VTermScreenCell,
    data: *mut ::core::ffi::c_void,
) -> ::core::ffi::c_int {
    unsafe {
        let term = data as *mut Terminal;
        if !term_may_alloc_scrollback(term, ::core::ptr::null_mut()) {
            return 0;
        }
        (*term)
            .sb
            .push(::core::slice::from_raw_parts(cells, cols as usize));
        if !(*term).synchronized_output {
            invalidate_terminal(term, None);
        }
        1
    }
}

pub unsafe extern "C" fn term_sb_pop(
    cols: ::core::ffi::c_int,
    cells: *mut VTermScreenCell,
    data: *mut ::core::ffi::c_void,
) -> ::core::ffi::c_int {
    unsafe {
        let term = data as *mut Terminal;
        let cells = ::core::slice::from_raw_parts_mut(cells, cols as usize);
        let old_height = &raw mut (*term).old_height;
        if !(*term).sb.pop(cells, &mut *old_height) {
            return 0;
        }
        if !(*term).synchronized_output {
            invalidate_terminal(term, None);
        }
        1
    }
}

pub unsafe extern "C" fn term_sb_clear(data: *mut ::core::ffi::c_void) -> ::core::ffi::c_int {
    unsafe {
        let term = data as *mut Terminal;
        // On the alternate screen the scrollback belongs to the screen
        // underneath, which is about to be restored; clearing it there
        // would lose history the program never touched.
        if (*term).in_altscreen || !(*term).sb.is_sized() || (*term).sb.is_empty() {
            return 1;
        }
        (*term).sb.clear();
        invalidate_terminal(term, None);
        1
    }
}

/// Render row `row` of the screen into the terminal's line buffer.
///
/// Negative rows come from the scrollback, counting back from -1. Cells
/// vterm reports as empty become spaces, and trailing spaces are dropped by
/// terminating at the last cell that actually held something.
pub unsafe fn fetch_row(term: *mut Terminal, row: ::core::ffi::c_int, end_col: ::core::ffi::c_int) {
    unsafe {
        // Worst case is one maximum-length grapheme cluster per column,
        // plus the terminator. C sized this buffer once, at 8191 bytes,
        // which a wide enough terminal full of clusters overruns.
        let needed = end_col.max(0) as usize * MAX_SCHAR_SIZE as usize + 1;
        if (*term).textbuf.len() < needed {
            (*term).textbuf.resize(needed, 0);
        }
        // Stable for the whole loop: nothing below touches `textbuf`.
        let start = (*term).textbuf.as_mut_ptr();
        let mut ptr = start;
        let mut line_len = 0usize;
        let mut col = 0;
        while col < end_col {
            let mut cell = blank_cell();
            fetch_cell(term, row, col, &raw mut cell);
            if cell.schar != 0 {
                schar_get_adv(&raw mut ptr, cell.schar);
                line_len = ptr.offset_from(start) as usize;
            } else {
                // Written but not counted, so that trailing blanks are
                // dropped by terminating at the last cell with content.
                *ptr = b' ' as ::core::ffi::c_char;
                ptr = ptr.add(1);
            }
            col += cell.width as ::core::ffi::c_int;
        }
        *start.add(line_len) = NUL as ::core::ffi::c_char;
    }
}

/// One cell of the screen or of the scrollback.
///
/// Returns false for a scrollback cell past the end of a row that was
/// stored while the terminal was narrower; the cell is left blank.
pub unsafe fn fetch_cell(
    term: *mut Terminal,
    row: ::core::ffi::c_int,
    col: ::core::ffi::c_int,
    cell: *mut VTermScreenCell,
) -> bool {
    unsafe {
        if row >= 0 {
            vterm_screen_get_cell((*term).vts, VTermPos { row, col }, cell);
            return true;
        }
        let stored = (*term)
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
}

/// Bring the scrollback back within `'scrollback'` after the option changed
/// or rows were pushed.
///
/// Trimming deletes from the top of the buffer, so the marks that pointed
/// into those lines have to move with them.
pub unsafe fn adjust_scrollback(term: *mut Terminal, buf: *mut buf_T) {
    unsafe {
        let limit = scrollback_limit(buf);
        assert!(
            (*term).sb.pending() == 0,
            "scrollback trimmed while rows were still owed to the buffer"
        );
        if limit < (*term).sb.len() {
            let diff = (*term).sb.len() - limit;
            for _ in 0..diff {
                ml_delete_buf(buf, 1 as linenr_T, false);
                (*term).sb.drop_oldest();
            }
            mark_adjust_buf(
                buf,
                1 as linenr_T,
                diff as linenr_T,
                MAXLNUM as linenr_T,
                -(diff as linenr_T),
                true,
                kMarkAdjustTerm,
                kExtmarkUndo,
            );
            deleted_lines_buf(buf, 1 as linenr_T, diff as linenr_T);
        }
        (*term).sb.set_capacity(limit);
    }
}

/// Mirror everything the scrollback gained or lost into the buffer's lines.
///
/// Reading is paused for the duration: appending lines can run autocommands,
/// and more terminal output arriving in the middle would be appended at the
/// wrong place.
pub unsafe fn refresh_scrollback(term: *mut Terminal, buf: *mut buf_T) {
    unsafe {
        let read_pause = (*term)
            .opts
            .read_pause_cb
            .expect("non-null function pointer");
        read_pause(true, (*term).opts.data);

        // Rows evicted since the last refresh are gone from the buffer's
        // top; move the marks that were pointing at them.
        let mut deleted = ((*term).sb.deleted() - (*term).old_sb_deleted) as linenr_T;
        deleted = deleted.min((*buf).b_ml.ml_line_count);
        mark_adjust_buf(
            buf,
            1 as linenr_T,
            deleted,
            MAXLNUM as linenr_T,
            -deleted,
            true,
            kMarkAdjustTerm,
            kExtmarkUndo,
        );
        (*term).old_sb_deleted = (*term).sb.deleted();

        let mut old_height = (*term).old_height;
        let mut width = 0;
        let mut height = 0;
        vterm_get_size((*term).vt, &raw mut height, &raw mut width);

        while deleted > 0 && (*buf).b_ml.ml_line_count > old_height as linenr_T {
            ml_delete_buf(buf, 1 as linenr_T, false);
            deleted_lines_buf(buf, 1 as linenr_T, 1 as linenr_T);
            deleted -= 1;
        }
        old_height = old_height.min((*buf).b_ml.ml_line_count as ::core::ffi::c_int);

        // Each owed row is appended just above the rows that make up the
        // screen, which sit at the end of the buffer.
        while (*term).sb.pending() > 0 {
            fetch_row(term, -(*term).sb.pending(), width);
            let at = (*buf).b_ml.ml_line_count as ::core::ffi::c_int - old_height;
            ml_append_buf(
                buf,
                at as linenr_T,
                (*term).textbuf.as_mut_ptr(),
                0 as colnr_T,
                false,
            );
            appended_lines_buf(buf, at as linenr_T, 1 as linenr_T);
            (*term).sb.mark_mirrored();
        }

        // Anything past the scrollback plus one screen is stale.
        let max_line_count = ((*term).sb.len() as ::core::ffi::c_int + height) as linenr_T;
        // Not immutable: ml_delete_buf() mutates (*buf).b_ml behind the raw pointer.
        #[allow(clippy::while_immutable_condition)]
        while (*buf).b_ml.ml_line_count > max_line_count {
            ml_delete_buf(buf, (*buf).b_ml.ml_line_count, false);
            deleted_lines_buf(buf, (*buf).b_ml.ml_line_count, 1 as linenr_T);
        }

        adjust_scrollback(term, buf);
        read_pause(false, (*term).opts.data);
    }
}
