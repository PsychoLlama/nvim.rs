//! Resizing the screen: rebuilding both cell buffers at a new size.
//!
//! This is where the screen does its heaviest pointer work, so it sits apart
//! from the callback table that drives it. A resize walks the old grid from
//! the bottom up, so that the content nearest the cursor survives; with
//! reflow on it treats a run of continuation rows as one logical line and
//! re-wraps it to the new width; what falls off the top goes out to the
//! host's scrollback, and lines come back from scrollback to fill whatever
//! room is left at the bottom.
//!
//! Everything here takes the screen as a raw pointer for the reason the
//! parent module gives: the host callbacks a resize drives may re-enter the
//! screen, so no borrow of it may live across one.
//!
//! Ported from libvterm, Copyright (c) 2008 Paul Evans, under the MIT
//! license; the notice is reproduced in licenses/libvterm-LICENSE.txt.

use core::ffi::{c_int, c_void};

use crate::src::nvim::os::libc::{abort, fprintf, memmove, stderr};
use crate::src::nvim::types::{
    ScreenCell, VTermLineInfo, VTermPos, VTermScreen, VTermScreenCell, VTermStateFields, size_t,
};
use crate::src::nvim::vterm::cell::{blank_cells, import_row};
use crate::src::nvim::vterm::vterm::{vterm_alloc, vterm_dealloc};

use super::{
    BUFIDX_ALTSCREEN, BUFIDX_PRIMARY, cells_mut, damage_screen, line_popcount, sb_pushline_from_row,
};

/// Rebuilds one of the screen's buffers at a new size.
///
/// Rows are laid out from the bottom up, so that the content nearest the
/// cursor survives. With reflow on, a run of continuation rows is one logical
/// line and is re-wrapped to the new width; otherwise every row stays a row.
/// Content that falls off the top goes to scrollback, and if there is room
/// left at the bottom, scrollback is popped back in to fill it. `active`
/// marks the buffer holding the cursor, whose position is rewritten.
unsafe fn resize_buffer(
    screen: *mut VTermScreen,
    bufidx: usize,
    new_rows: c_int,
    new_cols: c_int,
    active: bool,
    statefields: *mut VTermStateFields,
) {
    let old_rows = (*screen).rows;
    let old_cols = (*screen).cols;
    let old_buffer = (*screen).buffers[bufidx];
    let old_lineinfo = (*statefields).lineinfos[bufidx];

    let new_buffer = vterm_alloc(size_of::<ScreenCell>() * new_rows as size_t * new_cols as size_t)
        as *mut ScreenCell;
    let new_lineinfo =
        vterm_alloc(size_of::<VTermLineInfo>() * new_rows as size_t) as *mut VTermLineInfo;

    let mut old_row = old_rows - 1;
    let mut new_row = new_rows - 1;
    let old_cursor = (*statefields).pos;
    let mut new_cursor = VTermPos { row: -1, col: -1 };
    // The topmost row known to be blank, i.e. how much room there is to
    // scroll content down into.
    let mut final_blank_row = new_rows;
    let do_reflow = (*screen).reflow() != 0 && bufidx == BUFIDX_PRIMARY;

    while old_row >= 0 {
        // Walk back over the continuation rows of one logical line.
        let old_row_end = old_row;
        while do_reflow
            && !old_lineinfo.is_null()
            && old_row > 0
            && (*old_lineinfo.offset(old_row as isize)).continuation() != 0
        {
            old_row -= 1;
        }
        let old_row_start = old_row;

        let mut width = 0;
        for row in old_row_start..=old_row_end {
            let wrapped = do_reflow
                && row < old_rows - 1
                && (*old_lineinfo.offset((row + 1) as isize)).continuation() != 0;
            width += if wrapped {
                old_cols
            } else {
                line_popcount(old_buffer, row, old_cols)
            };
        }

        if final_blank_row == new_row + 1 && width == 0 {
            final_blank_row = new_row;
        }

        let new_height = if do_reflow && width != 0 {
            (width + new_cols - 1) / new_cols
        } else {
            1
        };
        let mut new_row_end = new_row;
        let mut new_row_start = new_row - new_height + 1;
        let spare_rows = new_rows - final_blank_row;

        if new_row_start < 0
            && spare_rows >= 0
            && (!active || new_cursor.row == -1 || new_cursor.row - new_row_start < new_rows)
        {
            // The line would fall off the top; push what is already placed
            // down into the blank rows at the bottom to make room.
            let downwards = (-new_row_start).min(spare_rows);
            let rowcount = new_rows - downwards;
            memmove(
                new_buffer.offset((downwards * new_cols) as isize) as *mut c_void,
                new_buffer as *const c_void,
                rowcount as size_t * new_cols as size_t * size_of::<ScreenCell>(),
            );
            memmove(
                new_lineinfo.offset(downwards as isize) as *mut c_void,
                new_lineinfo as *const c_void,
                rowcount as size_t * size_of::<VTermLineInfo>(),
            );
            new_row += downwards;
            new_row_start += downwards;
            new_row_end += downwards;
            if new_cursor.row >= 0 {
                new_cursor.row += downwards;
            }
            final_blank_row += downwards;
        }

        if new_row_start < 0 {
            // Out of room: this line and everything above it is scrollback.
            if old_row_start <= old_cursor.row && old_cursor.row <= old_row_end {
                new_cursor.row = 0;
                new_cursor.col = old_cursor.col.min(new_cols - 1);
            }
            break;
        }

        old_row = old_row_start;
        let mut old_col = 0;
        new_row = new_row_start;
        while new_row <= new_row_end {
            let mut count = width.min(new_cols);
            width -= count;
            let mut new_col = 0;
            while count != 0 {
                *new_buffer.offset((new_row * new_cols + new_col) as isize) =
                    *old_buffer.offset((old_row * old_cols + old_col) as isize);
                if old_cursor.row == old_row && old_cursor.col == old_col {
                    new_cursor = VTermPos {
                        row: new_row,
                        col: new_col,
                    };
                }
                old_col += 1;
                if old_col == old_cols {
                    old_row += 1;
                    if !do_reflow {
                        new_col += 1;
                        break;
                    }
                    old_col = 0;
                }
                new_col += 1;
                count -= 1;
            }
            // The cursor sat in the blank tail of the old row.
            if old_cursor.row == old_row && old_cursor.col >= old_col {
                new_cursor.row = new_row;
                new_cursor.col = (old_cursor.col - old_col + new_col).min(new_cols - 1);
            }
            let row_start = new_buffer.offset((new_row * new_cols) as isize);
            let row_cells = cells_mut(row_start, new_cols);
            blank_cells(&mut row_cells[new_col as usize..], &(*screen).pen);
            (*new_lineinfo.offset(new_row as isize))
                .set_continuation((new_row > new_row_start) as u32);
            new_row += 1;
        }

        old_row = old_row_start - 1;
        new_row = new_row_start - 1;
    }

    if old_cursor.row <= old_row {
        // The cursor was on a row that fell off the top; bring it into range.
        new_cursor.row = 0;
        new_cursor.col = old_cursor.col.min(new_cols - 1);
    }
    if active && (new_cursor.row == -1 || new_cursor.col == -1) {
        fprintf(
            stderr,
            c"screen_resize failed to update cursor position\n".as_ptr(),
        );
        abort();
    }

    if old_row >= 0 && bufidx == BUFIDX_PRIMARY {
        if let Some(callbacks) = (*screen).callbacks.as_ref()
            && callbacks.sb_pushline.is_some()
        {
            for row in 0..=old_row {
                sb_pushline_from_row(screen, row);
            }
        }
        if active {
            (*statefields).pos.row -= old_row + 1;
        }
    }
    if new_row >= 0 && bufidx == BUFIDX_PRIMARY {
        backfill_from_scrollback(
            screen,
            new_buffer,
            &mut new_row,
            new_cols,
            old_cols,
            active,
            statefields,
        );
    }
    if new_row >= 0 {
        // Content ended up low in the buffer; slide it up to the top and
        // blank whatever is left at the bottom.
        let moverows = new_rows - new_row - 1;
        memmove(
            new_buffer as *mut c_void,
            new_buffer.offset(((new_row + 1) * new_cols) as isize) as *const c_void,
            moverows as size_t * new_cols as size_t * size_of::<ScreenCell>(),
        );
        memmove(
            new_lineinfo as *mut c_void,
            new_lineinfo.offset((new_row + 1) as isize) as *const c_void,
            moverows as size_t * size_of::<VTermLineInfo>(),
        );
        new_cursor.row -= new_row + 1;
        for row in moverows..new_rows {
            let row_start = new_buffer.offset((row * new_cols) as isize);
            blank_cells(cells_mut(row_start, new_cols), &(*screen).pen);
            *new_lineinfo.offset(row as isize) = blank_lineinfo();
        }
    }

    vterm_dealloc(old_buffer as *mut c_void);
    (*screen).buffers[bufidx] = new_buffer;
    vterm_dealloc(old_lineinfo as *mut c_void);
    (*statefields).lineinfos[bufidx] = new_lineinfo;
    if active {
        (*statefields).pos = new_cursor;
    }
}

/// Pops lines off the host's scrollback into the rows above `*new_row`, until
/// the host runs out or the space does. Leaves `*new_row` one above the
/// topmost row it filled.
unsafe fn backfill_from_scrollback(
    screen: *mut VTermScreen,
    new_buffer: *mut ScreenCell,
    new_row: &mut c_int,
    new_cols: c_int,
    old_cols: c_int,
    active: bool,
    statefields: *mut VTermStateFields,
) {
    let Some(callbacks) = (*screen).callbacks.as_ref() else {
        return;
    };
    let Some(sb_popline) = callbacks.sb_popline else {
        return;
    };
    while *new_row >= 0 {
        if sb_popline(old_cols, (*screen).sb_buffer, (*screen).cbdata) == 0 {
            break;
        }
        let popped = core::slice::from_raw_parts((*screen).sb_buffer, old_cols as usize);
        let row_start = new_buffer.offset((*new_row * new_cols) as isize);
        let global_reverse = (*screen).global_reverse() != 0;
        let pen = (*screen).pen;
        import_row(popped, cells_mut(row_start, new_cols), global_reverse, &pen);
        *new_row -= 1;
        if active {
            (*statefields).pos.row += 1;
        }
    }
}

/// A line with no double-width, double-height or continuation marks.
fn blank_lineinfo() -> VTermLineInfo {
    VTermLineInfo {
        doublewidth_doubleheight_continuation: [0; 1],
        c2rust_padding: [0; 3],
    }
}

pub(super) unsafe extern "C" fn resize(
    new_rows: c_int,
    new_cols: c_int,
    fields: *mut VTermStateFields,
    user: *mut c_void,
) -> c_int {
    let screen = user as *mut VTermScreen;
    let altscreen = (*screen).buffers[BUFIDX_ALTSCREEN];
    let altscreen_active = !altscreen.is_null() && (*screen).buffer == altscreen;
    let old_rows = (*screen).rows;
    let old_cols = (*screen).cols;

    // The scrollback staging buffer has to hold a row of either width, so it
    // is grown before the resize and shrunk after it.
    if new_cols > old_cols {
        realloc_sb_buffer(screen, new_cols);
    }
    resize_buffer(
        screen,
        BUFIDX_PRIMARY,
        new_rows,
        new_cols,
        !altscreen_active,
        fields,
    );
    if !altscreen.is_null() {
        resize_buffer(
            screen,
            BUFIDX_ALTSCREEN,
            new_rows,
            new_cols,
            altscreen_active,
            fields,
        );
    } else if new_rows != old_rows {
        // The altscreen itself is not allocated, but its line info still has
        // to match the new height.
        vterm_dealloc((*fields).lineinfos[BUFIDX_ALTSCREEN] as *mut c_void);
        let lineinfo =
            vterm_alloc(size_of::<VTermLineInfo>() * new_rows as size_t) as *mut VTermLineInfo;
        for row in 0..new_rows {
            *lineinfo.offset(row as isize) = blank_lineinfo();
        }
        (*fields).lineinfos[BUFIDX_ALTSCREEN] = lineinfo;
    }

    (*screen).buffer = if altscreen_active {
        (*screen).buffers[BUFIDX_ALTSCREEN]
    } else {
        (*screen).buffers[BUFIDX_PRIMARY]
    };
    (*screen).rows = new_rows;
    (*screen).cols = new_cols;
    if new_cols <= old_cols {
        realloc_sb_buffer(screen, new_cols);
    }

    damage_screen(screen);
    if let Some(callbacks) = (*screen).callbacks.as_ref()
        && let Some(on_resize) = callbacks.resize
    {
        return on_resize(new_rows, new_cols, (*screen).cbdata);
    }
    1
}

/// Resizes the one-row staging buffer that carries cells to and from the
/// host's scrollback.
pub(super) unsafe fn realloc_sb_buffer(screen: *mut VTermScreen, cols: c_int) {
    if !(*screen).sb_buffer.is_null() {
        vterm_dealloc((*screen).sb_buffer as *mut c_void);
    }
    (*screen).sb_buffer =
        vterm_alloc(size_of::<VTermScreenCell>() * cols as size_t) as *mut VTermScreenCell;
}
