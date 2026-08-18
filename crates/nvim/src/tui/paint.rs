//! Getting the editor's screen onto the terminal.
//!
//! The editor describes what the screen should look like; the TUI keeps a
//! shadow copy of what it believes is already there ([`UGrid`]) and writes
//! only the difference. Two costs shape everything here: bytes on the wire,
//! and the terminal's own work. So the cursor is moved with whatever
//! sequence is shortest ([`cursor_goto`]), runs of blanks at the end of a
//! line are erased rather than overwritten ([`clear_region`]), and scrolling
//! is handed to the terminal when it can do it ([`tui_grid_scroll`]).
//!
//! Repainting is deferred: damage is accumulated as rectangles by
//! [`invalidate`] and drawn at the next [`tui_flush`], so a burst of updates
//! costs one write.

#![deny(unsafe_op_in_unsafe_fn)]

use crate::event::r#loop::{loop_purge, loop_size};
use crate::grid::{schar_cache_clear_if_full, schar_get, schar_get_ascii};
use crate::log::{LOGLVL_WRN, logmsg_c};
use crate::mbyte::{utf_ambiguous_width, utf_char2cells, utf_ptr2char};
use crate::tui::attrs::{attrs_differ, update_attrs};
use crate::tui::events::tui_busy_stop;
use crate::tui::negotiate::{LEFT_AND_RIGHT_MARGINS, tui_set_term_mode};
use crate::tui::output::{
    flush, out, out_cstr, out_fmt, out_repeat, terminfo_out, terminfo_print_nums,
};
use crate::tui::terminfo::caps::{
    TerminfoDef, kTerm_carriage_return, kTerm_change_scroll_region, kTerm_clear_screen,
    kTerm_clr_eol, kTerm_clr_eos, kTerm_cursor_address, kTerm_cursor_down, kTerm_cursor_home,
    kTerm_cursor_left, kTerm_cursor_right, kTerm_cursor_up, kTerm_delete_line, kTerm_erase_chars,
    kTerm_exit_attribute_mode, kTerm_insert_line, kTerm_parm_delete_line, kTerm_parm_down_cursor,
    kTerm_parm_insert_line, kTerm_parm_left_cursor, kTerm_parm_right_cursor, kTerm_parm_up_cursor,
    kTerm_set_lr_margin,
};
use crate::types::{FILE, Integer, LineFlags, Rect, String_0, TUIData, UCell, sattr_T, schar_T};
use ::libc::{fclose, fopen, fprintf};
use core::ffi::{c_char, c_int};

/// The `schar_T` value of a cell nothing has been drawn into, and of the
/// second half of a double-width character.
const NOTHING: schar_T = 0;

/// The line flag saying this row is the continuation of the one above.
const WRAPPED: c_int = 1;

/// How many `cursor_left`/`cursor_up` sequences are worth emitting before a
/// parameterised move is shorter. Left and up are cheaper to repeat than
/// right and down, which is why the two limits differ.
const MAX_REPEATED_BACK: c_int = 4;
const MAX_REPEATED_FORWARD: c_int = 2;

// ------------------------------------------------------------------ cursor

/// Follow the terminal's own wrap after a cell was printed in the last
/// column.
///
/// Terminals do not move the cursor out of the final column until the *next*
/// character arrives, so the shadow grid tracks a column one past the end
/// and resolves it here — either as soon as the cell is printed or just
/// before the next one, depending on which the terminal does.
pub(crate) fn final_column_wrap(tui: &mut TUIData) {
    let (width, height) = (tui.width, tui.height);
    let grid = &mut tui.grid;
    if grid.row != -1 && grid.col == width {
        grid.col = 0;
        // The last row does not wrap: the terminal scrolls instead, and the
        // editor is the one that decides when that happens.
        if grid.row < height.min(grid.height - 1) {
            grid.row += 1;
        }
    }
}

/// Can `next` cells starting at `col` be reprinted more cheaply than the
/// cursor can be moved past them?
///
/// Only if they are plain ASCII in the attributes already set: anything else
/// costs an escape sequence of its own, which is what we were avoiding.
fn cheap_to_print(tui: &TUIData, row: c_int, col: c_int, next: c_int) -> bool {
    for i in 0..next {
        let cell = tui.grid.cell(row, col + i);
        if attrs_differ(tui, cell.attr as c_int, tui.print_attr_id, tui.rgb) && tui.default_attr {
            return false;
        }
        // SAFETY: a grapheme handle is always readable.
        if schar_get_ascii(cell.data) == 0 {
            return false;
        }
    }
    true
}

/// Would a carriage return help get from the cursor's column to `col`?
///
/// A CR costs one byte and lands on column 0, from which the first columns
/// can be reached by reprinting the cells that are already there. That is
/// only a win when the cursor is far enough to the right, and — except for
/// column 0, which CR reaches on any row — only within the same row.
fn carriage_return_helps(tui: &TUIData, row: c_int, col: c_int) -> bool {
    let grid = &tui.grid;
    match col {
        0 => col != grid.col,
        _ if row != grid.row => false,
        1 => grid.col > 2 && cheap_to_print(tui, grid.row, 0, col),
        2 => grid.col > 5 && cheap_to_print(tui, grid.row, 0, col),
        _ => false,
    }
}

/// Move the cursor to `row`/`col` using the shortest sequence that gets
/// there.
///
/// Absolute addressing always works and is the fallback; relative moves are
/// tried first because they are usually shorter, and repeated single-step
/// moves beat a parameterised one only for very short distances.
pub(crate) fn cursor_goto(tui: &mut TUIData, row: c_int, col: c_int) {
    if row == tui.grid.row && col == tui.grid.col {
        return;
    }
    // A hyperlink must not be left open across a jump: the cells in between
    // would join the link.
    if tui.url >= 0 {
        out(tui, b"\x1b]8;;\x1b\\");
        tui.url = -1;
        tui.print_attr_id = -1;
    }
    if row == 0 && col == 0 {
        emit(tui, kTerm_cursor_home, &[]);
        tui.grid.goto(row, col);
        return;
    }
    // A grid row of -1 means the cursor's position is not known, so nothing
    // relative to it can be trusted.
    if tui.grid.row != -1 {
        if carriage_return_helps(tui, row, col) {
            emit(tui, kTerm_carriage_return, &[]);
            let grid_row = tui.grid.row;
            tui.grid.goto(grid_row, 0);
        }
        if row == tui.grid.row {
            // Moving left past the final column is only safe on terminals
            // that have already wrapped out of it.
            if col < tui.grid.col
                && (tui.immediate_wrap_after_last_column || tui.grid.col < tui.width)
            {
                step(
                    tui,
                    tui.grid.col - col,
                    MAX_REPEATED_BACK,
                    kTerm_cursor_left,
                    kTerm_parm_left_cursor,
                );
                tui.grid.goto(row, col);
                return;
            } else if col > tui.grid.col {
                step(
                    tui,
                    col - tui.grid.col,
                    MAX_REPEATED_FORWARD,
                    kTerm_cursor_right,
                    kTerm_parm_right_cursor,
                );
                tui.grid.goto(row, col);
                return;
            }
        }
        if col == tui.grid.col {
            if row > tui.grid.row {
                step(
                    tui,
                    row - tui.grid.row,
                    MAX_REPEATED_BACK,
                    kTerm_cursor_down,
                    kTerm_parm_down_cursor,
                );
                tui.grid.goto(row, col);
                return;
            } else if row < tui.grid.row {
                step(
                    tui,
                    tui.grid.row - row,
                    MAX_REPEATED_FORWARD,
                    kTerm_cursor_up,
                    kTerm_parm_up_cursor,
                );
                tui.grid.goto(row, col);
                return;
            }
        }
    }
    emit(tui, kTerm_cursor_address, &[row, col]);
    tui.grid.goto(row, col);
}

/// Move `count` cells in one direction, as repeated single steps while that
/// is shorter than the parameterised form.
fn step(tui: &mut TUIData, count: c_int, max_repeats: c_int, one: TerminfoDef, many: TerminfoDef) {
    if count <= max_repeats {
        for _ in 0..count {
            emit(tui, one, &[]);
        }
    } else {
        emit(tui, many, &[count]);
    }
}

/// Stage capability `what` with `params`.
fn emit(tui: &mut TUIData, what: TerminfoDef, params: &[c_int]) {
    terminfo_print_nums(tui, what, params);
}

// ------------------------------------------------------------------- cells

/// Print one cell's bytes in `attr`, advancing the shadow cursor.
fn print_cell(tui: &mut TUIData, text: &[u8], attr: sattr_T) {
    if !tui.immediate_wrap_after_last_column {
        final_column_wrap(tui);
    }
    update_attrs(tui, attr as c_int);
    out(tui, text);
    tui.grid.col += 1;
    if tui.immediate_wrap_after_last_column {
        final_column_wrap(tui);
    }
}

/// Print `width` spaces in the current attributes.
fn print_spaces(tui: &mut TUIData, width: c_int) {
    out_repeat(tui, b' ', width as usize);
    tui.grid.col += width;
    if tui.immediate_wrap_after_last_column {
        final_column_wrap(tui);
    }
}

/// Move to `row`/`col` and print `cell` there.
///
/// A double-width cell whose character the terminal will draw narrow would
/// leave the following cells one column out of place. There is no fixing
/// that from here, so the pair is blanked first and the shadow cursor is
/// marked unknown afterwards, forcing an absolute move for the next cell.
fn print_cell_at_pos(tui: &mut TUIData, row: c_int, col: c_int, cell: UCell, is_doublewidth: bool) {
    if tui.grid.row == -1 && cell.data == NOTHING {
        return;
    }
    cursor_goto(tui, row, col);
    let mut buf = [0u8; 32];
    // SAFETY: `schar_get` writes at most 32 bytes including its NUL, and the
    // three readers stop at that NUL.
    let (len, c, mut is_ambiwidth) = unsafe {
        let text = buf.as_mut_ptr().cast::<c_char>();
        (
            schar_get(text, cell.data),
            utf_ptr2char(text),
            utf_ambiguous_width(text),
        )
    };
    // SAFETY: `c` is the character just decoded.
    if is_doublewidth && (is_ambiwidth || unsafe { utf_char2cells(c) } == 1) {
        is_ambiwidth = true;
        update_attrs(tui, cell.attr as c_int);
        print_spaces(tui, 2);
        cursor_goto(tui, row, col);
    }
    print_cell(tui, &buf[..len], cell.attr);
    if is_ambiwidth {
        tui.grid.row = -1;
    }
}

// ---------------------------------------------------------------- clearing

/// Blank the rectangle `[top, bot) x [left, right)` on the terminal.
///
/// Erasing is cheaper than printing spaces, but only says anything about the
/// background colour on terminals whose `bce` promises it — which is what
/// `can_clear_attr` tracks.
pub(crate) fn clear_region(
    tui: &mut TUIData,
    top: c_int,
    bot: c_int,
    left: c_int,
    right: c_int,
    attr_id: c_int,
) {
    if tui.set_default_colors {
        update_attrs(tui, attr_id);
    } else {
        // Default colours are not known yet, so nothing can be assumed about
        // what a clear would paint; fall back to the terminal's own.
        emit_out(tui, kTerm_exit_attribute_mode);
    }
    if tui.can_clear_attr && left == 0 && right == tui.width && bot == tui.height {
        if top == 0 {
            emit_out(tui, kTerm_clear_screen);
            tui.grid.goto(top, left);
        } else {
            cursor_goto(tui, top, 0);
            emit_out(tui, kTerm_clr_eos);
        }
        return;
    }
    let width = right - left;
    for row in top..bot {
        cursor_goto(tui, row, left);
        if tui.can_clear_attr && right == tui.width {
            emit_out(tui, kTerm_clr_eol);
        } else if tui.can_erase_chars && tui.can_clear_attr && width >= 5 {
            emit(tui, kTerm_erase_chars, &[width]);
        } else {
            print_spaces(tui, width);
        }
    }
}

/// Stage a capability that takes no parameters.
fn emit_out(tui: &mut TUIData, what: TerminfoDef) {
    terminfo_out(tui, what);
}

// --------------------------------------------------------------- scrolling

/// Confine the terminal's own scrolling to `[top, bot] x [left, right]`.
fn set_scroll_region(tui: &mut TUIData, top: c_int, bot: c_int, left: c_int, right: c_int) {
    emit(tui, kTerm_change_scroll_region, &[top, bot]);
    if left != 0 || right != tui.width - 1 {
        tui_set_term_mode(tui, LEFT_AND_RIGHT_MARGINS, true);
        emit(tui, kTerm_set_lr_margin, &[left, right]);
    }
    // Terminals differ on where the cursor lands after this.
    tui.grid.row = -1;
}

/// Give the whole screen back to the terminal's scrolling.
fn reset_scroll_region(tui: &mut TUIData, fullwidth: bool) {
    if let Some(cap) = tui.terminfo_ext.reset_scroll_region {
        out_cstr(tui, Some(cap));
    } else {
        emit(tui, kTerm_change_scroll_region, &[0, tui.height - 1]);
    }
    if !fullwidth {
        emit(tui, kTerm_set_lr_margin, &[0, tui.width - 1]);
        tui_set_term_mode(tui, LEFT_AND_RIGHT_MARGINS, false);
    }
    tui.grid.row = -1;
}

// -------------------------------------------------------------- invalidation

/// Mark `[top, bot) x [left, right)` as needing a repaint at the next flush.
///
/// Overlapping damage is merged into one rectangle rather than kept apart:
/// repainting a little extra is cheaper than tracking the exact shape.
pub(crate) fn invalidate(tui: &mut TUIData, top: c_int, bot: c_int, left: c_int, right: c_int) {
    if let Some(r) = tui
        .invalid_regions
        .iter_mut()
        .find(|r| top <= r.bot && bot >= r.top && left <= r.right && right >= r.left)
    {
        r.top = r.top.min(top);
        r.bot = r.bot.max(bot);
        r.left = r.left.min(left);
        r.right = r.right.max(right);
        return;
    }
    tui.invalid_regions.push(Rect {
        top,
        bot,
        left,
        right,
    });
}

// ------------------------------------------------------------- the UI sinks

/// The editor's screen changed size.
///
pub fn tui_grid_resize(tui: &mut TUIData, _grid: Integer, width: Integer, height: Integer) {
    tui.grid.resize(width as c_int, height as c_int);
    let (grid_width, grid_height) = (tui.grid.width, tui.grid.height);
    for r in &mut tui.invalid_regions {
        r.bot = r.bot.min(grid_height);
        r.right = r.right.min(grid_width);
    }
    if tui.pending_resize_events == 0 && !tui.is_starting {
        // The editor resized itself, so ask the terminal to follow.
        out_fmt(tui, format_args!("\x1b[8;{height};{width}t"));
    } else {
        // This is the echo of a resize the terminal already made.
        tui.pending_resize_events = (tui.pending_resize_events - 1).max(0);
        tui.grid.row = -1;
    }
}

/// Blank the whole screen.
///
pub fn tui_grid_clear(tui: &mut TUIData, _grid: Integer) {
    tui.grid.clear();
    // SAFETY: no grapheme handle is held across this call.
    unsafe { schar_cache_clear_if_full() };
    // Nothing that was damaged matters once everything is repainted.
    tui.invalid_regions.clear();
    clear_region(tui, 0, tui.height, 0, tui.width, 0);
}

/// Where the cursor should be left after the next flush.
///
pub fn tui_grid_cursor_goto(tui: &mut TUIData, _grid: Integer, row: Integer, col: Integer) {
    tui.row = row as c_int;
    tui.col = col as c_int;
}

/// Scroll `rows` rows out of the region `[startrow, endrow) x [startcol,
/// endcol)`; a positive `rows` moves text up.
///
/// The terminal does the work when it can — it has the old contents already,
/// so nothing has to be re-sent. That needs either the whole screen or a
/// scroll region, and margins as well when the region is not full width.
/// Failing that the region is simply repainted.
///
#[expect(clippy::too_many_arguments, reason = "the UI event's own shape")]
pub fn tui_grid_scroll(
    tui: &mut TUIData,
    _grid: Integer,
    startrow: Integer,
    endrow: Integer,
    startcol: Integer,
    endcol: Integer,
    rows: Integer,
    _cols: Integer,
) {
    let (top, bot) = (startrow as c_int, endrow as c_int - 1);
    let (left, right) = (startcol as c_int, endcol as c_int - 1);
    let fullwidth = left == 0 && right == tui.width - 1;
    let full_screen = fullwidth && top == 0 && bot == tui.height - 1;
    tui.grid.scroll(top, bot, left, right, rows as c_int);

    let has_margins = tui.has_left_and_right_margin_mode && tui.can_set_lr_margin;
    let can_scroll = tui.can_scroll
        && (full_screen || (tui.can_change_scroll_region && (fullwidth || has_margins)));
    if !can_scroll {
        // Repaint the rows that changed. The region shrinks by the scroll
        // distance: the rows scrolled in from outside are painted by the
        // editor's own line updates.
        let (mut startrow, mut endrow) = (startrow, endrow);
        if rows > 0 {
            endrow -= rows;
        } else {
            startrow -= rows;
        }
        invalidate(
            tui,
            startrow as c_int,
            endrow as c_int,
            startcol as c_int,
            endcol as c_int,
        );
        return;
    }

    if !full_screen {
        set_scroll_region(tui, top, bot, left, right);
    }
    cursor_goto(tui, top, left);
    update_attrs(tui, 0);
    let rows = rows as c_int;
    match rows {
        1 => emit_out(tui, kTerm_delete_line),
        -1 => emit_out(tui, kTerm_insert_line),
        _ if rows > 0 => emit(tui, kTerm_parm_delete_line, &[rows]),
        _ => emit(tui, kTerm_parm_insert_line, &[-rows]),
    }
    if !full_screen {
        reset_scroll_region(tui, fullwidth);
    }
}

/// One row of the editor's screen changed.
///
/// `chunk`/`attrs` describe `[startcol, endcol)`; everything from there to
/// `clearcol` is blank in `clearattr`.
///
/// # Safety
/// `chunk` and `attrs` must each hold `endcol - startcol` entries.
#[expect(clippy::too_many_arguments, reason = "the UI event's own shape")]
pub unsafe fn tui_raw_line(
    tui: &mut TUIData,
    _grid: Integer,
    linerow: Integer,
    startcol: Integer,
    endcol: Integer,
    clearcol: Integer,
    clearattr: Integer,
    flags: LineFlags,
    chunk: *const schar_T,
    attrs: *const sattr_T,
) {
    let len = (endcol - startcol) as usize;
    // SAFETY: the caller guarantees the two arrays' length. An empty range
    // says nothing about the pointers, which may be null.
    let (chunk, attrs) = unsafe {
        if len == 0 {
            ([].as_slice(), [].as_slice())
        } else {
            (
                core::slice::from_raw_parts(chunk, len),
                core::slice::from_raw_parts(attrs, len),
            )
        }
    };
    let row = linerow as c_int;
    for (i, (&data, &attr)) in chunk.iter().zip(attrs).enumerate() {
        assert!(
            (attr as usize) < tui.attrs.len(),
            "undefined attribute {attr}"
        );
        let col = startcol as c_int + i as c_int;
        tui.grid.set_cell(row, col, UCell { data, attr });
    }
    for col in startcol as c_int..endcol as c_int {
        let cell = tui.grid.cell(row, col);
        let doublewidth = col < endcol as c_int - 1 && tui.grid.cell(row, col + 1).data == NOTHING;
        print_cell_at_pos(tui, row, col, cell, doublewidth);
    }
    if clearcol > endcol {
        tui.grid.clear_chunk(
            row,
            endcol as c_int,
            clearcol as c_int,
            clearattr as sattr_T,
        );
        clear_region(
            tui,
            row,
            row + 1,
            endcol as c_int,
            clearcol as c_int,
            clearattr as c_int,
        );
    }
    // A wrapped row's last cell must actually be printed for the terminal to
    // wrap by itself, which is how a long line stays one line for whoever
    // copies it out of the terminal.
    if flags as c_int & WRAPPED != 0
        && tui.width == tui.grid.width
        && linerow + 1 < tui.grid.height as Integer
    {
        if endcol != tui.grid.width as Integer {
            let width = tui.grid.width;
            let size = if tui.grid.cell(row, width - 1).data == NOTHING {
                2
            } else {
                1
            };
            let cell = tui.grid.cell(row, width - size);
            print_cell_at_pos(tui, row, width - size, cell, size == 2);
        }
        final_column_wrap(tui);
    }
}

/// Repaint everything that was invalidated and put the cursor where the
/// editor last asked for it.
///
pub fn tui_flush(tui: &mut TUIData) {
    // SAFETY: the loop is the TUI's own, alive for as long as the TUI is.
    unsafe {
        let queued = loop_size(tui.loop_0);
        if queued > TOO_MANY_EVENTS {
            // The editor is producing updates faster than they can be drawn;
            // drawing every one of them would only fall further behind.
            logmsg_c!(
                LOGLVL_WRN,
                core::ptr::null(),
                c"tui_flush".as_ptr(),
                0,
                true,
                c"TUI event-queue flooded (thread_events=%zu); purging".as_ptr(),
                queued,
            );
            loop_purge(tui.loop_0);
            tui_busy_stop(tui);
        }
    }

    // Taken from the back, and taken off the list before it is painted:
    // painting must not see damage it is in the middle of repairing.
    while let Some(r) = tui.invalid_regions.pop() {
        assert!(
            r.bot <= tui.grid.height && r.right <= tui.grid.width,
            "damage outside the grid"
        );
        for row in r.top..r.bot {
            repaint_row(tui, row, r.left, r.right);
        }
    }
    cursor_goto(tui, tui.row, tui.col);
    flush(tui);
}

/// How many queued events count as the editor having run away with itself.
const TOO_MANY_EVENTS: usize = 1000000;

/// Repaint `[left, right)` of one row.
///
/// The trailing run of blanks is erased rather than printed, so a mostly
/// empty line costs a couple of bytes instead of one per column.
fn repaint_row(tui: &mut TUIData, row: c_int, left: c_int, right: c_int) {
    let clear_attr = tui.grid.cell(row, right - 1).attr;
    let mut clear_col = right;
    while clear_col > 0 {
        let cell = tui.grid.cell(row, clear_col - 1);
        if cell.data != b' ' as schar_T || cell.attr != clear_attr {
            break;
        }
        clear_col -= 1;
    }
    for col in left..clear_col {
        let cell = tui.grid.cell(row, col);
        let doublewidth = col < clear_col - 1 && tui.grid.cell(row, col + 1).data == NOTHING;
        print_cell_at_pos(tui, row, col, cell, doublewidth);
    }
    if clear_col < right {
        clear_region(tui, row, row + 1, clear_col, right, clear_attr as c_int);
    }
}

/// Write what the screen looks like to `path`, in the form the functional
/// tests read back.
///
/// The whole grid is printed unconditionally with the write path pointed at
/// the file, so the result is the sequences a terminal would have received,
/// not a rendering of them.
///
/// # Safety
/// `path` must be a valid API string.
pub unsafe fn tui_screenshot(tui: &mut TUIData, path: String_0) {
    // SAFETY: the caller guarantees `path`.
    let file = unsafe { fopen(path.data, c"w".as_ptr()) };
    if file.is_null() {
        return;
    }
    flush(tui);
    let (height, width) = (tui.grid.height, tui.grid.width);
    // SAFETY: `file` is the handle just opened, and the format string holds
    // the two `%d` these arguments fill.
    unsafe { fprintf(file, c"%d,%d\n".as_ptr(), height, width) };
    tui.grid.goto(0, 0);
    tui.screenshot = file.cast::<FILE>();
    emit_out(tui, kTerm_clear_screen);
    for row in 0..tui.grid.height {
        cursor_goto(tui, row, 0);
        for col in 0..tui.grid.width {
            let cell = tui.grid.cell(row, col);
            let mut buf = [0u8; 32];
            // SAFETY: `schar_get` writes at most 32 bytes including its NUL.
            let len = unsafe { schar_get(buf.as_mut_ptr().cast::<c_char>(), cell.data) };
            print_cell(tui, &buf[..len], cell.attr);
        }
    }
    flush(tui);
    tui.screenshot = core::ptr::null_mut();
    // SAFETY: `file` is the handle opened above and not closed since.
    unsafe { fclose(file) };
}
