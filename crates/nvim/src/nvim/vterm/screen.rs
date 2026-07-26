//! The screen: the grid of cells the terminal state paints onto.
//!
//! The state below reports what changed — a glyph here, a scroll there — and
//! the screen keeps the resulting cells, batches the damage at whatever
//! granularity the host asked for, and hands cells back on demand. It also
//! owns the resize, which is where the interesting work is: rows reflow to
//! the new width, spare lines go out to scrollback, and lines come back from
//! scrollback to fill what is left.
//!
//! Anything reachable from the state's callback table takes the screen as a
//! raw pointer rather than a reference: those callbacks re-enter the screen
//! freely, and a live borrow across such a call would not hold.

use core::ffi::{c_int, c_void};

use crate::src::nvim::global_cell::GlobalCell;
use crate::src::nvim::os::libc::{abort, fprintf, memmove, stderr};
use crate::src::nvim::types::{
    ScreenCell, VTerm, VTermAttr, VTermColor, VTermDamageSize, VTermGlyphInfo, VTermLineInfo,
    VTermPos, VTermProp, VTermRect, VTermScreen, VTermScreenCallbacks, VTermScreenCell,
    VTermStateCallbacks, VTermStateFallbacks, VTermStateFields, VTermValue, size_t,
};
use crate::src::nvim::vterm::cell::{
    SCHAR_CONTINUATION, blank_cells, erased_pen, export_pen, import_row,
};
use crate::src::nvim::vterm::damage::{
    Damage, NO_RECT, VTERM_DAMAGE_CELL, VTERM_DAMAGE_SCROLL, follow_scroll, intersects,
    merge_damage,
};
use crate::src::nvim::vterm::pen::{
    VTERM_ATTR_BACKGROUND, VTERM_ATTR_BASELINE, VTERM_ATTR_BLINK, VTERM_ATTR_BOLD,
    VTERM_ATTR_CONCEAL, VTERM_ATTR_DIM, VTERM_ATTR_FONT, VTERM_ATTR_FOREGROUND, VTERM_ATTR_ITALIC,
    VTERM_ATTR_OVERLINE, VTERM_ATTR_REVERSE, VTERM_ATTR_SMALL, VTERM_ATTR_STRIKE,
    VTERM_ATTR_UNDERLINE, VTERM_ATTR_URI, convert_color_to_rgb,
};
use crate::src::nvim::vterm::state::{
    vterm_obtain_state, vterm_state_get_lineinfo, vterm_state_reset, vterm_state_set_callbacks,
    vterm_state_set_unrecognised_fallbacks,
};
use crate::src::nvim::vterm::vterm::{
    vterm_allocator_free, vterm_allocator_malloc, vterm_get_size, vterm_scroll_rect,
};

/// The terminal property that swaps the alternate screen buffer in and out.
const VTERM_PROP_ALTSCREEN: VTermProp = 3;
/// The terminal property that reverses the whole screen at once.
const VTERM_PROP_REVERSE: VTermProp = 6;
pub const BUFIDX_PRIMARY: usize = 0;
pub const BUFIDX_ALTSCREEN: usize = 1;

// ------------------------------------------------------------ the cell grid

/// The cell at `row`/`col`, or null outside the grid.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn getcell(
    screen: *const VTermScreen,
    row: c_int,
    col: c_int,
) -> *mut ScreenCell {
    if row < 0 || row >= (*screen).rows || col < 0 || col >= (*screen).cols {
        return core::ptr::null_mut();
    }
    (*screen)
        .buffer
        .offset(((*screen).cols * row + col) as isize)
}

/// `count` cells starting at `first`, as a slice.
unsafe fn cells_mut<'a>(first: *mut ScreenCell, count: c_int) -> &'a mut [ScreenCell] {
    core::slice::from_raw_parts_mut(first, count as usize)
}

/// A freshly allocated, fully blanked cell grid.
unsafe fn alloc_buffer(screen: *mut VTermScreen, rows: c_int, cols: c_int) -> *mut ScreenCell {
    let bytes = size_of::<ScreenCell>() * rows as size_t * cols as size_t;
    let buffer = vterm_allocator_malloc((*screen).vt, bytes) as *mut ScreenCell;
    blank_cells(cells_mut(buffer, rows * cols), &(*screen).pen);
    buffer
}

/// How many leading cells of a row are non-blank, i.e. where its trailing run
/// of blanks starts.
unsafe fn line_popcount(buffer: *const ScreenCell, row: c_int, cols: c_int) -> c_int {
    let mut col = cols - 1;
    while col >= 0 && (*buffer.offset((row * cols + col) as isize)).schar == 0 {
        col -= 1;
    }
    col + 1
}

// -------------------------------------------------------------------- damage

/// Records damage to `rect`, telling the host at once or holding it back,
/// according to the merge level.
unsafe fn damage_rect(screen: *mut VTermScreen, rect: VTermRect) {
    let merge = (*screen).damage_merge;
    let emit = match merge_damage(&mut (*screen).damaged, rect, merge) {
        Damage::Pending => return,
        Damage::Emit(pending) => pending,
        Damage::FlushFirst(pending) => {
            vterm_screen_flush_damage(screen);
            pending
        }
    };
    if let Some(callbacks) = (*screen).callbacks.as_ref()
        && let Some(damage) = callbacks.damage
    {
        damage(emit, (*screen).cbdata);
    }
}

/// Records damage to every cell.
unsafe fn damage_screen(screen: *mut VTermScreen) {
    let whole = VTermRect {
        start_row: 0,
        end_row: (*screen).rows,
        start_col: 0,
        end_col: (*screen).cols,
    };
    damage_rect(screen, whole);
}

// -------------------------------------------------------- state callbacks

unsafe extern "C" fn putglyph(
    info: *mut VTermGlyphInfo,
    pos: VTermPos,
    user: *mut c_void,
) -> c_int {
    let screen = user as *mut VTermScreen;
    let cell = getcell(screen, pos.row, pos.col);
    if cell.is_null() {
        return 0;
    }
    (*cell).schar = (*info).schar;
    // An erasing glyph keeps the cell's existing pen.
    if (*info).schar != 0 {
        (*cell).pen = (*screen).pen;
    }
    for col in 1..(*info).width {
        (*getcell(screen, pos.row, pos.col + col)).schar = SCHAR_CONTINUATION;
    }
    (*cell).pen.set_protected_cell((*info).protected_cell());
    (*cell).pen.set_dwl((*info).dwl());
    (*cell).pen.set_dhl((*info).dhl());
    damage_rect(
        screen,
        VTermRect {
            start_row: pos.row,
            end_row: pos.row + 1,
            start_col: pos.col,
            end_col: pos.col + (*info).width,
        },
    );
    1
}

unsafe extern "C" fn movecursor(
    pos: VTermPos,
    oldpos: VTermPos,
    visible: c_int,
    user: *mut c_void,
) -> c_int {
    let screen = user as *mut VTermScreen;
    if let Some(callbacks) = (*screen).callbacks.as_ref()
        && let Some(on_movecursor) = callbacks.movecursor
    {
        return on_movecursor(pos, oldpos, visible, (*screen).cbdata);
    }
    0
}

unsafe extern "C" fn setpenattr(attr: VTermAttr, val: *mut VTermValue, user: *mut c_void) -> c_int {
    let pen = &mut (*(user as *mut VTermScreen)).pen;
    match attr {
        VTERM_ATTR_BOLD => pen.set_bold((*val).boolean as u32),
        VTERM_ATTR_UNDERLINE => pen.set_underline((*val).number as u32),
        VTERM_ATTR_ITALIC => pen.set_italic((*val).boolean as u32),
        VTERM_ATTR_BLINK => pen.set_blink((*val).boolean as u32),
        VTERM_ATTR_REVERSE => pen.set_reverse((*val).boolean as u32),
        VTERM_ATTR_CONCEAL => pen.set_conceal((*val).boolean as u32),
        VTERM_ATTR_STRIKE => pen.set_strike((*val).boolean as u32),
        VTERM_ATTR_FONT => pen.set_font((*val).number as u32),
        VTERM_ATTR_FOREGROUND => pen.fg = (*val).color,
        VTERM_ATTR_BACKGROUND => pen.bg = (*val).color,
        VTERM_ATTR_SMALL => pen.set_small((*val).boolean as u32),
        VTERM_ATTR_BASELINE => pen.set_baseline((*val).number as u32),
        VTERM_ATTR_URI => pen.uri = (*val).number,
        VTERM_ATTR_DIM => pen.set_dim((*val).boolean as u32),
        VTERM_ATTR_OVERLINE => pen.set_overline((*val).boolean as u32),
        _ => return 0,
    }
    1
}

unsafe extern "C" fn settermprop(
    prop: VTermProp,
    val: *mut VTermValue,
    user: *mut c_void,
) -> c_int {
    let screen = user as *mut VTermScreen;
    match prop {
        VTERM_PROP_ALTSCREEN => {
            let want_altscreen = (*val).boolean != 0;
            if want_altscreen && (*screen).buffers[BUFIDX_ALTSCREEN].is_null() {
                return 0;
            }
            (*screen).buffer = if want_altscreen {
                (*screen).buffers[BUFIDX_ALTSCREEN]
            } else {
                (*screen).buffers[BUFIDX_PRIMARY]
            };
            // Only on disable: enabling is followed by an erase, which
            // reports the damage anyway.
            if !want_altscreen {
                damage_screen(screen);
            }
        }
        VTERM_PROP_REVERSE => {
            (*screen).set_global_reverse((*val).boolean as u32);
            damage_screen(screen);
        }
        _ => {}
    }
    if let Some(callbacks) = (*screen).callbacks.as_ref()
        && let Some(on_settermprop) = callbacks.settermprop
    {
        return on_settermprop(prop, val, (*screen).cbdata);
    }
    1
}

unsafe extern "C" fn bell(user: *mut c_void) -> c_int {
    let screen = user as *mut VTermScreen;
    if let Some(callbacks) = (*screen).callbacks.as_ref()
        && let Some(on_bell) = callbacks.bell
    {
        return on_bell((*screen).cbdata);
    }
    0
}

unsafe extern "C" fn theme(dark: *mut bool, user: *mut c_void) -> c_int {
    let screen = user as *mut VTermScreen;
    if let Some(callbacks) = (*screen).callbacks.as_ref()
        && let Some(on_theme) = callbacks.theme
    {
        return on_theme(dark, (*screen).cbdata);
    }
    1
}

unsafe extern "C" fn sb_clear(user: *mut c_void) -> c_int {
    let screen = user as *mut VTermScreen;
    if let Some(callbacks) = (*screen).callbacks.as_ref()
        && let Some(on_sb_clear) = callbacks.sb_clear
        && on_sb_clear((*screen).cbdata) != 0
    {
        return 1;
    }
    0
}

/// A line's double-width or double-height mark changed: restamp the row's
/// cells and report the damage. Going double-width halves the usable row, so
/// the right half is erased outright.
unsafe extern "C" fn setlineinfo(
    row: c_int,
    newinfo: *const VTermLineInfo,
    oldinfo: *const VTermLineInfo,
    user: *mut c_void,
) -> c_int {
    let screen = user as *mut VTermScreen;
    if (*newinfo).doublewidth() == (*oldinfo).doublewidth()
        && (*newinfo).doubleheight() == (*oldinfo).doubleheight()
    {
        return 1;
    }
    for col in 0..(*screen).cols {
        let cell = getcell(screen, row, col);
        (*cell).pen.set_dwl((*newinfo).doublewidth());
        (*cell).pen.set_dhl((*newinfo).doubleheight());
    }
    let doublewidth = (*newinfo).doublewidth() != 0;
    let mut rect = VTermRect {
        start_row: row,
        end_row: row + 1,
        start_col: 0,
        end_col: if doublewidth {
            (*screen).cols / 2
        } else {
            (*screen).cols
        },
    };
    damage_rect(screen, rect);
    if doublewidth {
        rect.start_col = (*screen).cols / 2;
        rect.end_col = (*screen).cols;
        erase_internal(rect, 0, user);
    }
    1
}

// ------------------------------------------------- moving and erasing cells

/// Copies `screen`'s row `row` into the scrollback buffer and hands it over.
unsafe fn sb_pushline_from_row(screen: *mut VTermScreen, row: c_int) {
    let mut pos = VTermPos { row, col: 0 };
    while pos.col < (*screen).cols {
        vterm_screen_get_cell(screen, pos, (*screen).sb_buffer.offset(pos.col as isize));
        pos.col += 1;
    }
    (*(*screen).callbacks)
        .sb_pushline
        .expect("non-null function pointer")(
        (*screen).cols, (*screen).sb_buffer, (*screen).cbdata
    );
}

/// Moves cells within the grid. Rows scrolled off the top of the primary
/// buffer go out to scrollback on the way.
unsafe extern "C" fn moverect_internal(
    dest: VTermRect,
    src: VTermRect,
    user: *mut c_void,
) -> c_int {
    let screen = user as *mut VTermScreen;
    let full_width_from_top = dest.start_row == 0
        && dest.start_col == 0
        && dest.end_col == (*screen).cols
        && (*screen).buffer == (*screen).buffers[BUFIDX_PRIMARY];
    if full_width_from_top
        && let Some(callbacks) = (*screen).callbacks.as_ref()
        && callbacks.sb_pushline.is_some()
    {
        for row in 0..src.start_row {
            sb_pushline_from_row(screen, row);
        }
    }
    let cols = src.end_col - src.start_col;
    let downward = src.start_row - dest.start_row;
    // Overlapping ranges: copy away from the direction of travel.
    let (mut row, limit, step) = if downward < 0 {
        (dest.end_row - 1, dest.start_row - 1, -1)
    } else {
        (dest.start_row, dest.end_row, 1)
    };
    while row != limit {
        memmove(
            getcell(screen, row, dest.start_col) as *mut c_void,
            getcell(screen, row + downward, src.start_col) as *const c_void,
            cols as size_t * size_of::<ScreenCell>(),
        );
        row += step;
    }
    1
}

/// Tells the host about a move it may be able to perform itself, falling back
/// to reporting the destination as damaged.
unsafe extern "C" fn moverect_user(dest: VTermRect, src: VTermRect, user: *mut c_void) -> c_int {
    let screen = user as *mut VTermScreen;
    if let Some(callbacks) = (*screen).callbacks.as_ref()
        && let Some(moverect) = callbacks.moverect
    {
        // Flushing under scroll merging would recurse back into here.
        if (*screen).damage_merge != VTERM_DAMAGE_SCROLL {
            vterm_screen_flush_damage(screen);
        }
        if moverect(dest, src, (*screen).cbdata) != 0 {
            return 1;
        }
    }
    damage_rect(screen, dest);
    1
}

/// Blanks the cells in `rect`, keeping the screen's current colours. A
/// selective erase spares cells the host marked protected.
unsafe extern "C" fn erase_internal(rect: VTermRect, selective: c_int, user: *mut c_void) -> c_int {
    let screen = user as *mut VTermScreen;
    let mut row = rect.start_row;
    while row < (*(*screen).state).rows && row < rect.end_row {
        let info = vterm_state_get_lineinfo((*screen).state, row);
        for col in rect.start_col..rect.end_col {
            let cell = getcell(screen, row, col);
            if selective != 0 && (*cell).pen.protected_cell() != 0 {
                continue;
            }
            (*cell).schar = 0;
            (*cell).pen = erased_pen((*screen).pen.fg, (*screen).pen.bg);
            (*cell).pen.set_dwl((*info).doublewidth());
            (*cell).pen.set_dhl((*info).doubleheight());
        }
        row += 1;
    }
    1
}

/// The reporting half of an erase: the cells themselves are another pass.
unsafe extern "C" fn erase_user(rect: VTermRect, _selective: c_int, user: *mut c_void) -> c_int {
    damage_rect(user as *mut VTermScreen, rect);
    1
}

unsafe extern "C" fn erase(rect: VTermRect, selective: c_int, user: *mut c_void) -> c_int {
    erase_internal(rect, selective, user);
    erase_user(rect, 0, user)
}

/// Scrolls a region. Under cell or row merging the move happens at once, in
/// two passes so that the host sees the cells settle before it is told. Under
/// scroll merging the move is coalesced with whatever is already pending.
unsafe extern "C" fn scrollrect(
    region: VTermRect,
    downward: c_int,
    rightward: c_int,
    user: *mut c_void,
) -> c_int {
    let screen = user as *mut VTermScreen;
    if (*screen).damage_merge != VTERM_DAMAGE_SCROLL {
        vterm_scroll_rect(
            region,
            downward,
            rightward,
            Some(moverect_internal),
            Some(erase_internal),
            user,
        );
        vterm_screen_flush_damage(screen);
        vterm_scroll_rect(
            region,
            downward,
            rightward,
            Some(moverect_user),
            Some(erase_user),
            user,
        );
        return 1;
    }
    if (*screen).damaged.start_row != NO_RECT && !intersects(&region, &(*screen).damaged) {
        vterm_screen_flush_damage(screen);
    }
    let pending_matches = (*screen).pending_scrollrect == region
        && ((*screen).pending_scroll_downward == 0 && downward == 0
            || (*screen).pending_scroll_rightward == 0 && rightward == 0);
    if (*screen).pending_scrollrect.start_row == NO_RECT {
        (*screen).pending_scrollrect = region;
        (*screen).pending_scroll_downward = downward;
        (*screen).pending_scroll_rightward = rightward;
    } else if pending_matches {
        (*screen).pending_scroll_downward += downward;
        (*screen).pending_scroll_rightward += rightward;
    } else {
        vterm_screen_flush_damage(screen);
        (*screen).pending_scrollrect = region;
        (*screen).pending_scroll_downward = downward;
        (*screen).pending_scroll_rightward = rightward;
    }
    vterm_scroll_rect(
        region,
        downward,
        rightward,
        Some(moverect_internal),
        Some(erase_internal),
        user,
    );
    if (*screen).damaged.start_row != NO_RECT {
        follow_scroll(&mut (*screen).damaged, &region, downward, rightward);
    }
    1
}

// -------------------------------------------------------------- resizing

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

    let new_buffer = vterm_allocator_malloc(
        (*screen).vt,
        size_of::<ScreenCell>() * new_rows as size_t * new_cols as size_t,
    ) as *mut ScreenCell;
    let new_lineinfo = vterm_allocator_malloc(
        (*screen).vt,
        size_of::<VTermLineInfo>() * new_rows as size_t,
    ) as *mut VTermLineInfo;

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

    vterm_allocator_free((*screen).vt, old_buffer as *mut c_void);
    (*screen).buffers[bufidx] = new_buffer;
    vterm_allocator_free((*screen).vt, old_lineinfo as *mut c_void);
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

unsafe extern "C" fn resize(
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
        vterm_allocator_free(
            (*screen).vt,
            (*fields).lineinfos[BUFIDX_ALTSCREEN] as *mut c_void,
        );
        let lineinfo = vterm_allocator_malloc(
            (*screen).vt,
            size_of::<VTermLineInfo>() * new_rows as size_t,
        ) as *mut VTermLineInfo;
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
unsafe fn realloc_sb_buffer(screen: *mut VTermScreen, cols: c_int) {
    if !(*screen).sb_buffer.is_null() {
        vterm_allocator_free((*screen).vt, (*screen).sb_buffer as *mut c_void);
    }
    (*screen).sb_buffer =
        vterm_allocator_malloc((*screen).vt, size_of::<VTermScreenCell>() * cols as size_t)
            as *mut VTermScreenCell;
}

static STATE_CALLBACKS: GlobalCell<VTermStateCallbacks> = GlobalCell::new(VTermStateCallbacks {
    putglyph: Some(putglyph),
    movecursor: Some(movecursor),
    scrollrect: Some(scrollrect),
    moverect: None,
    erase: Some(erase),
    initpen: None,
    setpenattr: Some(setpenattr),
    settermprop: Some(settermprop),
    bell: Some(bell),
    resize: Some(resize),
    theme: Some(theme),
    setlineinfo: Some(setlineinfo),
    sb_clear: Some(sb_clear),
});

// ------------------------------------------------------------ the interface

unsafe fn screen_new(vt: *mut VTerm) -> *mut VTermScreen {
    let state = vterm_obtain_state(vt);
    if state.is_null() {
        return core::ptr::null_mut();
    }
    let screen = vterm_allocator_malloc(vt, size_of::<VTermScreen>()) as *mut VTermScreen;
    let mut rows = 0;
    let mut cols = 0;
    vterm_get_size(vt, &mut rows, &mut cols);
    (*screen).vt = vt;
    (*screen).state = state;
    (*screen).damage_merge = VTERM_DAMAGE_CELL;
    (*screen).damaged.start_row = NO_RECT;
    (*screen).pending_scrollrect.start_row = NO_RECT;
    (*screen).rows = rows;
    (*screen).cols = cols;
    (*screen).set_global_reverse(0);
    (*screen).set_reflow(0);
    (*screen).callbacks = core::ptr::null();
    (*screen).cbdata = core::ptr::null_mut();
    (*screen).buffers[BUFIDX_PRIMARY] = alloc_buffer(screen, rows, cols);
    (*screen).buffer = (*screen).buffers[BUFIDX_PRIMARY];
    (*screen).sb_buffer = core::ptr::null_mut();
    realloc_sb_buffer(screen, cols);
    vterm_state_set_callbacks(state, STATE_CALLBACKS.ptr(), screen as *mut c_void);
    screen
}

/// The terminal's screen, creating it on first use.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn vterm_obtain_screen(vt: *mut VTerm) -> *mut VTermScreen {
    if (*vt).screen.is_null() {
        (*vt).screen = screen_new(vt);
    }
    (*vt).screen
}

pub unsafe fn vterm_screen_free(screen: *mut VTermScreen) {
    vterm_allocator_free(
        (*screen).vt,
        (*screen).buffers[BUFIDX_PRIMARY] as *mut c_void,
    );
    if !(*screen).buffers[BUFIDX_ALTSCREEN].is_null() {
        vterm_allocator_free(
            (*screen).vt,
            (*screen).buffers[BUFIDX_ALTSCREEN] as *mut c_void,
        );
    }
    vterm_allocator_free((*screen).vt, (*screen).sb_buffer as *mut c_void);
    vterm_allocator_free((*screen).vt, screen as *mut c_void);
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn vterm_screen_reset(screen: *mut VTermScreen, hard: c_int) {
    (*screen).damaged.start_row = NO_RECT;
    (*screen).pending_scrollrect.start_row = NO_RECT;
    vterm_state_reset((*screen).state, hard);
    vterm_screen_flush_damage(screen);
}

/// Copies the cell at `pos` into its reported form. Returns 0 for a position
/// outside the screen.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn vterm_screen_get_cell(
    screen: *const VTermScreen,
    pos: VTermPos,
    cell: *mut VTermScreenCell,
) -> c_int {
    let intcell = getcell(screen, pos.row, pos.col);
    if intcell.is_null() {
        return 0;
    }
    // The gap behind a double-width glyph reports as empty.
    (*cell).schar = if (*intcell).schar == SCHAR_CONTINUATION {
        0
    } else {
        (*intcell).schar
    };
    export_pen(&(*intcell).pen, (*screen).global_reverse() != 0, &mut *cell);
    let followed_by_gap = pos.col < (*screen).cols - 1
        && (*getcell(screen, pos.row, pos.col + 1)).schar == SCHAR_CONTINUATION;
    (*cell).width = if followed_by_gap { 2 } else { 1 };
    1
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn vterm_screen_enable_reflow(screen: *mut VTermScreen, reflow: bool) {
    (*screen).set_reflow(reflow as u32);
}

/// Allocates the alternate screen buffer, which the terminal starts without.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn vterm_screen_enable_altscreen(screen: *mut VTermScreen, altscreen: c_int) {
    if (*screen).buffers[BUFIDX_ALTSCREEN].is_null() && altscreen != 0 {
        let mut rows = 0;
        let mut cols = 0;
        vterm_get_size((*screen).vt, &mut rows, &mut cols);
        (*screen).buffers[BUFIDX_ALTSCREEN] = alloc_buffer(screen, rows, cols);
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn vterm_screen_set_callbacks(
    screen: *mut VTermScreen,
    callbacks: *const VTermScreenCallbacks,
    user: *mut c_void,
) {
    (*screen).callbacks = callbacks;
    (*screen).cbdata = user;
}

pub unsafe fn vterm_screen_set_unrecognised_fallbacks(
    screen: *mut VTermScreen,
    fallbacks: *const VTermStateFallbacks,
    user: *mut c_void,
) {
    vterm_state_set_unrecognised_fallbacks((*screen).state, fallbacks, user);
}

/// Hands the host everything held back by the merge level.
pub unsafe fn vterm_screen_flush_damage(screen: *mut VTermScreen) {
    if (*screen).pending_scrollrect.start_row != NO_RECT {
        vterm_scroll_rect(
            (*screen).pending_scrollrect,
            (*screen).pending_scroll_downward,
            (*screen).pending_scroll_rightward,
            Some(moverect_user),
            Some(erase_user),
            screen as *mut c_void,
        );
        (*screen).pending_scrollrect.start_row = NO_RECT;
    }
    if (*screen).damaged.start_row != NO_RECT {
        if let Some(callbacks) = (*screen).callbacks.as_ref()
            && let Some(damage) = callbacks.damage
        {
            damage((*screen).damaged, (*screen).cbdata);
        }
        (*screen).damaged.start_row = NO_RECT;
    }
}

pub unsafe fn vterm_screen_set_damage_merge(screen: *mut VTermScreen, size: VTermDamageSize) {
    vterm_screen_flush_damage(screen);
    (*screen).damage_merge = size;
}

/// [`convert_color_to_rgb`] against a screen rather than a state.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn vterm_screen_convert_color_to_rgb(
    screen: *const VTermScreen,
    col: *mut VTermColor,
) {
    convert_color_to_rgb(&*(*screen).state, &mut *col);
}
