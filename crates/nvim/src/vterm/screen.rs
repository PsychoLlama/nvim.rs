//! The screen: the grid of cells the terminal state paints onto.
//!
//! The state below reports what changed — a glyph here, a scroll there — and
//! the screen keeps the resulting cells, batches the damage at whatever
//! granularity the host asked for, and hands cells back on demand. This
//! module holds the cell grid, the damage bookkeeping, the callback table the
//! state paints through and the interface the host uses; the resize, which is
//! where the interesting work is, lives in [`resize`](self::resize).
//!
//! Everything the state and the host call arrives as a raw pointer and is
//! wrapped in a [`Screen`] on the way in. The wrapper is what carries the
//! promise; past it the whole module is ordinary code. It may not be
//! *borrowed* across a call that hands control away, though: the state's
//! callbacks re-enter the screen freely, and so do `sb_pushline`,
//! `sb_popline`, `damage` and `resize` on the host's side. Every such call
//! here copies what it needs out first and reaches the callback with nothing
//! outstanding.
//!
//! Ported from libvterm, Copyright (c) 2008 Paul Evans, under the MIT
//! license; the notice is reproduced in licenses/libvterm-LICENSE.txt.

#![deny(unsafe_op_in_unsafe_fn)]

pub mod resize;

use core::ffi::{c_int, c_void};
use core::ops::{Deref, DerefMut};

use self::resize::{realloc_sb_buffer, resize};
use crate::global_cell::GlobalCell;
use crate::os::libc::memmove;
use crate::types::{
    ScreenCell, ScreenPen, VTerm, VTermAttr, VTermColor, VTermDamageSize, VTermGlyphInfo,
    VTermLineInfo, VTermPos, VTermProp, VTermRect, VTermScreen, VTermScreenCallbacks,
    VTermScreenCell, VTermStateCallbacks, VTermStateFallbacks, VTermValue, size_t,
};
use crate::vterm::cell::{SCHAR_CONTINUATION, blank_cells, erased_pen, export_pen};
use crate::vterm::damage::{Damage, NO_RECT, follow_scroll, intersects, merge_damage};
use crate::vterm::pen::convert_color_to_rgb;
use crate::vterm::state::entry::{
    vterm_obtain_state, vterm_state_get_lineinfo, vterm_state_reset, vterm_state_set_callbacks,
    vterm_state_set_unrecognised_fallbacks,
};
use crate::vterm::vterm::{
    VTERM_ATTR_BACKGROUND, VTERM_ATTR_BASELINE, VTERM_ATTR_BLINK, VTERM_ATTR_BOLD,
    VTERM_ATTR_CONCEAL, VTERM_ATTR_DIM, VTERM_ATTR_FONT, VTERM_ATTR_FOREGROUND, VTERM_ATTR_ITALIC,
    VTERM_ATTR_OVERLINE, VTERM_ATTR_REVERSE, VTERM_ATTR_SMALL, VTERM_ATTR_STRIKE,
    VTERM_ATTR_UNDERLINE, VTERM_ATTR_URI, VTERM_DAMAGE_CELL, VTERM_DAMAGE_SCROLL,
    VTERM_PROP_ALTSCREEN, VTERM_PROP_REVERSE, vterm_alloc, vterm_dealloc, vterm_get_size,
    vterm_scroll_rect,
};

/// The two handler shapes `vterm_scroll_rect` drives a scroll through: one
/// pass moves the cells, the other tells the host. Spelled out because a
/// function *item* does not coerce to the pointer the table wants on its own.
type MoveRect = unsafe extern "C" fn(VTermRect, VTermRect, *mut c_void) -> c_int;
type EraseRect = unsafe extern "C" fn(VTermRect, c_int, *mut c_void) -> c_int;
type Handlers = (Option<MoveRect>, Option<EraseRect>);

/// Which of the screen's two cell buffers a call means: the one the child
/// normally paints into, or the alternate one `VTERM_PROP_ALTSCREEN` swaps
/// in.
pub const BUFIDX_PRIMARY: usize = 0;
pub const BUFIDX_ALTSCREEN: usize = 1;

// -------------------------------------------------------------- the wrapper

/// A terminal's screen, reached through the pointer the state and the host
/// pass around.
struct Screen(*mut VTermScreen);

impl Deref for Screen {
    type Target = VTermScreen;

    fn deref(&self) -> &VTermScreen {
        // SAFETY: the wrapper promised the screen stays live, and `&self`
        // bounds the borrow to code that cannot hand control away.
        unsafe { &*self.0 }
    }
}

impl DerefMut for Screen {
    fn deref_mut(&mut self) -> &mut VTermScreen {
        // SAFETY: as for `deref`, and `&mut self` rules out a second borrow
        // through this wrapper.
        unsafe { &mut *self.0 }
    }
}

impl Screen {
    /// The screen `screen` points at.
    ///
    /// # Safety
    ///
    /// `screen` must point at a live `VTermScreen` for as long as the wrapper
    /// is used, and the buffers, state and callback table hanging off it must
    /// stay valid for that long.
    ///
    /// The wrapper may not be *borrowed* across a call that hands control to
    /// the state or to the host: both re-enter the screen and wrap it again,
    /// so a `&`/`&mut` taken through [`Deref`] would alias. Copy out what the
    /// call needs first — the module docs say the same thing at length.
    unsafe fn at(screen: *mut VTermScreen) -> Self {
        Screen(screen)
    }

    /// The screen a state callback's `user` pointer means.
    ///
    /// # Safety
    ///
    /// As for [`Screen::at`]. `user` must be the pointer [`screen_new`]
    /// installed in the state's callback table.
    unsafe fn of(user: *mut c_void) -> Self {
        Screen(user.cast())
    }

    // ---------------------------------------------------------- the cells

    /// The cell at `row`/`col`, or `None` outside the grid.
    fn cell(&self, row: c_int, col: c_int) -> Option<&ScreenCell> {
        // SAFETY: `getcell` bounds-checks and answers null outside the grid;
        // the wrapper promised the grid it indexes is live.
        unsafe { getcell(self.0, row, col).as_ref() }
    }

    /// [`Screen::cell`], to write through.
    fn cell_mut(&mut self, row: c_int, col: c_int) -> Option<&mut ScreenCell> {
        // SAFETY: as for `cell`, and `&mut self` rules out a second borrow.
        unsafe { getcell(self.0, row, col).as_mut() }
    }

    // ------------------------------------------------- the host's callbacks

    /// The host's callback table and its data pointer, if it installed one.
    ///
    /// Copied out by value on purpose: a host callback may re-enter the
    /// screen, so neither the table nor the screen it hangs off may be
    /// borrowed while one runs.
    fn host(&self) -> Option<(VTermScreenCallbacks, *mut c_void)> {
        // SAFETY: the host installed this table through
        // `vterm_screen_set_callbacks` and promised it outlives the screen.
        let callbacks = unsafe { self.callbacks.as_ref() }?;
        Some((*callbacks, self.cbdata))
    }

    /// Tells the host `rect` has to be redrawn.
    fn report_damage(&mut self, rect: VTermRect) {
        let Some((host, data)) = self.host() else {
            return;
        };
        let Some(damage) = host.damage else {
            return;
        };
        // SAFETY: the host's own callback, reached with nothing borrowed.
        unsafe { damage(rect, data) };
    }

    /// Offers the host a move it may be able to perform itself. False if it
    /// did not, and the destination has to be reported as damaged instead.
    fn report_moverect(&mut self, dest: VTermRect, src: VTermRect) -> bool {
        let Some((host, data)) = self.host() else {
            return false;
        };
        let Some(moverect) = host.moverect else {
            return false;
        };
        // Flushing under scroll merging would recurse back into here.
        if self.damage_merge != VTERM_DAMAGE_SCROLL {
            self.flush_damage();
        }
        // SAFETY: the host's own callback, reached with nothing borrowed.
        unsafe { moverect(dest, src, data) != 0 }
    }

    /// Passes a state callback the host may also want to see straight on.
    /// `unhandled` is what the state is told when the host is not listening.
    fn report<T>(
        &mut self,
        pick: impl FnOnce(&VTermScreenCallbacks) -> Option<T>,
        call: impl FnOnce(T, *mut c_void) -> c_int,
        unhandled: c_int,
    ) -> c_int {
        let Some((host, data)) = self.host() else {
            return unhandled;
        };
        match pick(&host) {
            Some(slot) => call(slot, data),
            None => unhandled,
        }
    }
}

// ------------------------------------------------------------ the cell grid

/// The cell at `row`/`col`, or null outside the grid.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn getcell(
    screen: *const VTermScreen,
    row: c_int,
    col: c_int,
) -> *mut ScreenCell {
    // SAFETY: the caller promised a live screen, whose grid is as large as
    // its own `rows` and `cols` say.
    let grid = unsafe { &*screen };
    if row < 0 || row >= grid.rows || col < 0 || col >= grid.cols {
        return core::ptr::null_mut();
    }
    // SAFETY: bounds-checked just above, so the offset lands in the grid.
    unsafe { grid.buffer.offset((grid.cols * row + col) as isize) }
}

/// `count` cells starting at `first`, as a slice.
///
/// # Safety
///
/// `first` must point at `count` live, initialised cells that nothing else
/// borrows for the lifetime the caller picks.
unsafe fn cells_mut<'a>(first: *mut ScreenCell, count: c_int) -> &'a mut [ScreenCell] {
    // SAFETY: the caller promised exactly that.
    unsafe { core::slice::from_raw_parts_mut(first, count as usize) }
}

/// Row `row` of a grid `cols` columns wide, as a slice.
///
/// # Safety
///
/// As for [`cells_mut`]: `buffer` must hold at least `(row + 1) * cols` live
/// cells.
unsafe fn row_cells<'a>(buffer: *mut ScreenCell, row: c_int, cols: c_int) -> &'a mut [ScreenCell] {
    // SAFETY: the caller promised the row is inside the grid.
    unsafe { cells_mut(buffer.offset((row * cols) as isize), cols) }
}

/// A freshly allocated, fully blanked cell grid.
///
/// # Safety
///
/// `rows * cols` cells must fit in `size_t`.
unsafe fn alloc_buffer(pen: ScreenPen, rows: c_int, cols: c_int) -> *mut ScreenCell {
    let bytes = size_of::<ScreenCell>() * rows as size_t * cols as size_t;
    // SAFETY: `vterm_alloc` answers a live block of `bytes`, which is exactly
    // `rows * cols` cells and nothing else has a pointer to it yet.
    let buffer = unsafe { vterm_alloc(bytes) } as *mut ScreenCell;
    blank_cells(unsafe { cells_mut(buffer, rows * cols) }, &pen);
    buffer
}

/// How many leading cells of `row` are non-blank, i.e. where its trailing run
/// of blanks starts.
fn line_popcount(row: &[ScreenCell]) -> c_int {
    let mut col = row.len() as c_int - 1;
    while col >= 0 && row[col as usize].schar == 0 {
        col -= 1;
    }
    col + 1
}

// -------------------------------------------------------------------- damage

impl Screen {
    /// Records damage to `rect`, telling the host at once or holding it back,
    /// according to the merge level.
    fn damage(&mut self, rect: VTermRect) {
        let merge = self.damage_merge;
        let emit = match merge_damage(&mut self.damaged, rect, merge) {
            Damage::Pending => return,
            Damage::Emit(pending) => pending,
            Damage::FlushFirst(pending) => {
                self.flush_damage();
                pending
            }
        };
        self.report_damage(emit);
    }

    /// Records damage to every cell.
    fn damage_screen(&mut self) {
        let whole = VTermRect {
            start_row: 0,
            end_row: self.rows,
            start_col: 0,
            end_col: self.cols,
        };
        self.damage(whole);
    }

    /// Hands the host everything held back by the merge level.
    fn flush_damage(&mut self) {
        if self.pending_scrollrect.start_row != NO_RECT {
            let (region, downward) = (self.pending_scrollrect, self.pending_scroll_downward);
            let (rightward, user) = (self.pending_scroll_rightward, self.0.cast::<c_void>());
            let (moved, erased): Handlers = (Some(moverect_user), Some(erase_user));
            // SAFETY: the scroll only calls back into those two handlers,
            // which take the `user` pointer this screen is.
            unsafe { vterm_scroll_rect(region, downward, rightward, moved, erased, user) };
            self.pending_scrollrect.start_row = NO_RECT;
        }
        if self.damaged.start_row != NO_RECT {
            let damaged = self.damaged;
            self.report_damage(damaged);
            self.damaged.start_row = NO_RECT;
        }
    }
}

// -------------------------------------------------------- state callbacks

unsafe extern "C" fn putglyph(
    info: *mut VTermGlyphInfo,
    pos: VTermPos,
    user: *mut c_void,
) -> c_int {
    // SAFETY: the state hands back the pointer `screen_new` installed and
    // owns the glyph for the length of the call.
    let (mut screen, info) = unsafe { (Screen::of(user), &*info) };
    screen.put_glyph(info, pos)
}

unsafe extern "C" fn movecursor(
    pos: VTermPos,
    oldpos: VTermPos,
    visible: c_int,
    user: *mut c_void,
) -> c_int {
    // SAFETY: the state hands back the pointer `screen_new` installed.
    let mut screen = unsafe { Screen::of(user) };
    screen.report(
        |host| host.movecursor,
        // SAFETY: the host's own callback, reached with nothing borrowed.
        |movecursor, data| unsafe { movecursor(pos, oldpos, visible, data) },
        0,
    )
}

unsafe extern "C" fn setpenattr(attr: VTermAttr, val: *mut VTermValue, user: *mut c_void) -> c_int {
    // SAFETY: the state hands back the pointer `screen_new` installed and
    // owns the value for the length of the call.
    let (mut screen, val) = unsafe { (Screen::of(user), &*val) };
    screen.set_pen_attr(attr, val)
}

unsafe extern "C" fn settermprop(
    prop: VTermProp,
    val: *mut VTermValue,
    user: *mut c_void,
) -> c_int {
    // SAFETY: as for `setpenattr`; the raw value is passed on to the host,
    // which is why it is kept alongside the boolean arm.
    let (mut screen, boolean) = unsafe { (Screen::of(user), (*val).boolean) };
    screen.set_termprop(prop, boolean, val)
}

unsafe extern "C" fn bell(user: *mut c_void) -> c_int {
    // SAFETY: the state hands back the pointer `screen_new` installed.
    let mut screen = unsafe { Screen::of(user) };
    screen.report(
        |host| host.bell,
        // SAFETY: the host's own callback, reached with nothing borrowed.
        |bell, data| unsafe { bell(data) },
        0,
    )
}

unsafe extern "C" fn theme(dark: *mut bool, user: *mut c_void) -> c_int {
    // SAFETY: the state hands back the pointer `screen_new` installed and
    // owns the flag for the length of the call.
    let mut screen = unsafe { Screen::of(user) };
    screen.report(
        |host| host.theme,
        // SAFETY: the host's own callback, reached with nothing borrowed.
        |theme, data| unsafe { theme(dark, data) },
        1,
    )
}

unsafe extern "C" fn sb_clear(user: *mut c_void) -> c_int {
    // SAFETY: the state hands back the pointer `screen_new` installed.
    let mut screen = unsafe { Screen::of(user) };
    let cleared = screen.report(
        |host| host.sb_clear,
        // SAFETY: the host's own callback, reached with nothing borrowed.
        |sb_clear, data| unsafe { sb_clear(data) },
        0,
    );
    (cleared != 0) as c_int
}

unsafe extern "C" fn setlineinfo(
    row: c_int,
    newinfo: *const VTermLineInfo,
    oldinfo: *const VTermLineInfo,
    user: *mut c_void,
) -> c_int {
    // SAFETY: the state hands back the pointer `screen_new` installed and
    // owns both line infos for the length of the call.
    let (mut screen, new, old) = unsafe { (Screen::of(user), &*newinfo, &*oldinfo) };
    screen.set_lineinfo(row, new, old)
}

unsafe extern "C" fn moverect_internal(
    dest: VTermRect,
    src: VTermRect,
    user: *mut c_void,
) -> c_int {
    // SAFETY: `vterm_scroll_rect` passes on the pointer it was handed, which
    // is the screen this callback was installed for.
    let mut screen = unsafe { Screen::of(user) };
    screen.move_cells(dest, src);
    1
}

/// Tells the host about a move it may be able to perform itself, falling back
/// to reporting the destination as damaged.
unsafe extern "C" fn moverect_user(dest: VTermRect, src: VTermRect, user: *mut c_void) -> c_int {
    // SAFETY: as for `moverect_internal`.
    let mut screen = unsafe { Screen::of(user) };
    if !screen.report_moverect(dest, src) {
        screen.damage(dest);
    }
    1
}

unsafe extern "C" fn erase_internal(rect: VTermRect, selective: c_int, user: *mut c_void) -> c_int {
    // SAFETY: as for `moverect_internal`.
    let mut screen = unsafe { Screen::of(user) };
    screen.erase_cells(rect, selective != 0);
    1
}

/// The reporting half of an erase: the cells themselves are another pass.
unsafe extern "C" fn erase_user(rect: VTermRect, _selective: c_int, user: *mut c_void) -> c_int {
    // SAFETY: as for `moverect_internal`.
    let mut screen = unsafe { Screen::of(user) };
    screen.damage(rect);
    1
}

unsafe extern "C" fn erase(rect: VTermRect, selective: c_int, user: *mut c_void) -> c_int {
    // SAFETY: the state hands back the pointer `screen_new` installed.
    let mut screen = unsafe { Screen::of(user) };
    screen.erase_cells(rect, selective != 0);
    screen.damage(rect);
    1
}

unsafe extern "C" fn scrollrect(
    region: VTermRect,
    downward: c_int,
    rightward: c_int,
    user: *mut c_void,
) -> c_int {
    // SAFETY: the state hands back the pointer `screen_new` installed.
    let mut screen = unsafe { Screen::of(user) };
    screen.scroll_rect(region, downward, rightward);
    1
}

// ------------------------------------------------- painting, moving, erasing

impl Screen {
    /// Paints one glyph, marking the cells a wide one covers.
    fn put_glyph(&mut self, info: &VTermGlyphInfo, pos: VTermPos) -> c_int {
        let pen = self.pen;
        let Some(cell) = self.cell_mut(pos.row, pos.col) else {
            return 0;
        };
        cell.schar = info.schar;
        // An erasing glyph keeps the cell's existing pen.
        if info.schar != 0 {
            cell.pen = pen;
        }
        cell.pen.set_protected_cell(info.protected_cell());
        cell.pen.set_dwl(info.dwl());
        cell.pen.set_dhl(info.dhl());
        // A wide glyph can reach the last column with no cell to continue
        // into: on a one-column grid there is no next column for the state to
        // wrap it away into first, so it is placed and its continuation runs
        // off the end. Truncate the glyph to the cells that exist, which is
        // what the caller's clipped damage rect already claims.
        for col in 1..info.width {
            let Some(continuation) = self.cell_mut(pos.row, pos.col + col) else {
                break;
            };
            continuation.schar = SCHAR_CONTINUATION;
        }
        self.damage(VTermRect {
            start_row: pos.row,
            end_row: pos.row + 1,
            start_col: pos.col,
            end_col: pos.col + info.width,
        });
        1
    }

    /// Applies one SGR attribute to the pen the screen paints with.
    fn set_pen_attr(&mut self, attr: VTermAttr, val: &VTermValue) -> c_int {
        // SAFETY: the state checks an attribute's value type before setting
        // it, so the arm each match arm reads is the one it wrote. The others
        // are stale rather than uninitialised — the terminal is zeroed at
        // `vterm_alloc` — and every arm here is plain scalar data.
        let number = || unsafe { val.number };
        let color = || unsafe { val.color };
        let pen = &mut self.pen;
        match attr {
            VTERM_ATTR_BOLD => pen.set_bold(number() as u32),
            VTERM_ATTR_UNDERLINE => pen.set_underline(number() as u32),
            VTERM_ATTR_ITALIC => pen.set_italic(number() as u32),
            VTERM_ATTR_BLINK => pen.set_blink(number() as u32),
            VTERM_ATTR_REVERSE => pen.set_reverse(number() as u32),
            VTERM_ATTR_CONCEAL => pen.set_conceal(number() as u32),
            VTERM_ATTR_STRIKE => pen.set_strike(number() as u32),
            VTERM_ATTR_FONT => pen.set_font(number() as u32),
            VTERM_ATTR_FOREGROUND => pen.fg = color(),
            VTERM_ATTR_BACKGROUND => pen.bg = color(),
            VTERM_ATTR_SMALL => pen.set_small(number() as u32),
            VTERM_ATTR_BASELINE => pen.set_baseline(number() as u32),
            VTERM_ATTR_URI => pen.uri = number(),
            VTERM_ATTR_DIM => pen.set_dim(number() as u32),
            VTERM_ATTR_OVERLINE => pen.set_overline(number() as u32),
            _ => return 0,
        }
        1
    }

    /// Acts on the two terminal properties the screen itself keeps, then
    /// passes every property on to the host.
    fn set_termprop(&mut self, prop: VTermProp, boolean: c_int, val: *mut VTermValue) -> c_int {
        match prop {
            VTERM_PROP_ALTSCREEN => {
                let want_altscreen = boolean != 0;
                if want_altscreen && self.buffers[BUFIDX_ALTSCREEN].is_null() {
                    return 0;
                }
                self.buffer = if want_altscreen {
                    self.buffers[BUFIDX_ALTSCREEN]
                } else {
                    self.buffers[BUFIDX_PRIMARY]
                };
                // Only on disable: enabling is followed by an erase, which
                // reports the damage anyway.
                if !want_altscreen {
                    self.damage_screen();
                }
            }
            VTERM_PROP_REVERSE => {
                self.set_global_reverse(boolean as u32);
                self.damage_screen();
            }
            _ => {}
        }
        self.report(
            |host| host.settermprop,
            // SAFETY: the host's own callback, reached with nothing
            // borrowed; the value is the state's and outlives the call.
            |settermprop, data| unsafe { settermprop(prop, val, data) },
            1,
        )
    }

    /// A line's double-width or double-height mark changed: restamp the row's
    /// cells and report the damage. Going double-width halves the usable row,
    /// so the right half is erased outright.
    fn set_lineinfo(&mut self, row: c_int, new: &VTermLineInfo, old: &VTermLineInfo) -> c_int {
        if new.doublewidth() == old.doublewidth() && new.doubleheight() == old.doubleheight() {
            return 1;
        }
        let (dwl, dhl) = (new.doublewidth(), new.doubleheight());
        for col in 0..self.cols {
            let Some(cell) = self.cell_mut(row, col) else {
                continue;
            };
            cell.pen.set_dwl(dwl);
            cell.pen.set_dhl(dhl);
        }
        let doublewidth = dwl != 0;
        let mut rect = VTermRect {
            start_row: row,
            end_row: row + 1,
            start_col: 0,
            end_col: if doublewidth {
                self.cols / 2
            } else {
                self.cols
            },
        };
        self.damage(rect);
        if doublewidth {
            rect.start_col = self.cols / 2;
            rect.end_col = self.cols;
            self.erase_cells(rect, false);
        }
        1
    }

    /// Copies the screen's row `row` into the scrollback staging buffer and
    /// hands it over to the host.
    fn push_line(&mut self, row: c_int) {
        let (cols, sb_buffer, this) = (self.cols, self.sb_buffer, self.0);
        for col in 0..cols {
            let pos = VTermPos { row, col };
            // SAFETY: the staging buffer holds a row of `cols` cells —
            // `realloc_sb_buffer` keeps it at the wider of the two widths a
            // resize is between — and `vterm_screen_get_cell` writes one.
            unsafe { vterm_screen_get_cell(this, pos, sb_buffer.offset(col as isize)) };
        }
        let Some((host, data)) = self.host() else {
            return;
        };
        let Some(sb_pushline) = host.sb_pushline else {
            return;
        };
        // SAFETY: the host's own callback, reached with nothing borrowed; it
        // reads `cols` cells out of the staging buffer and keeps neither.
        unsafe { sb_pushline(cols, sb_buffer, data) };
    }

    /// Moves cells within the grid. Rows scrolled off the top of the primary
    /// buffer go out to scrollback on the way.
    fn move_cells(&mut self, dest: VTermRect, src: VTermRect) {
        let full_width_from_top = dest.start_row == 0
            && dest.start_col == 0
            && dest.end_col == self.cols
            && self.buffer == self.buffers[BUFIDX_PRIMARY];
        let host_takes_lines = self
            .host()
            .is_some_and(|(host, _)| host.sb_pushline.is_some());
        if full_width_from_top && host_takes_lines {
            for row in 0..src.start_row {
                self.push_line(row);
            }
        }
        let bytes = (src.end_col - src.start_col) as size_t * size_of::<ScreenCell>();
        let downward = src.start_row - dest.start_row;
        // Overlapping ranges: copy away from the direction of travel.
        let (mut row, limit, step) = if downward < 0 {
            (dest.end_row - 1, dest.start_row - 1, -1)
        } else {
            (dest.start_row, dest.end_row, 1)
        };
        while row != limit {
            // SAFETY: `getcell` bounds-checks both ends against the grid, and
            // `memmove` is the overlap-tolerant copy the two rows need.
            let to = unsafe { getcell(self.0, row, dest.start_col) } as *mut c_void;
            let from = unsafe { getcell(self.0, row + downward, src.start_col) } as *const c_void;
            unsafe { memmove(to, from, bytes) };
            row += step;
        }
    }

    /// Blanks the cells in `rect`, keeping the screen's current colours. A
    /// selective erase spares cells the host marked protected.
    fn erase_cells(&mut self, rect: VTermRect, selective: bool) {
        let state = self.state;
        // SAFETY: the state is the one `screen_new` obtained from the
        // terminal, and it outlives the screen.
        let state_rows = unsafe { (*state).rows };
        let (fg, bg) = (self.pen.fg, self.pen.bg);
        let mut row = rect.start_row;
        while row < state_rows && row < rect.end_row {
            // SAFETY: `row` is inside the state's own row count, which is
            // what `vterm_state_get_lineinfo` indexes.
            let info = unsafe { &*vterm_state_get_lineinfo(state, row) };
            let (dwl, dhl) = (info.doublewidth(), info.doubleheight());
            for col in rect.start_col..rect.end_col {
                let Some(cell) = self.cell_mut(row, col) else {
                    continue;
                };
                if selective && cell.pen.protected_cell() != 0 {
                    continue;
                }
                cell.schar = 0;
                cell.pen = erased_pen(fg, bg);
                cell.pen.set_dwl(dwl);
                cell.pen.set_dhl(dhl);
            }
            row += 1;
        }
    }

    /// Scrolls a region. Under cell or row merging the move happens at once,
    /// in two passes so that the host sees the cells settle before it is
    /// told. Under scroll merging the move is coalesced with whatever is
    /// already pending.
    fn scroll_rect(&mut self, region: VTermRect, downward: c_int, rightward: c_int) {
        let user = self.0.cast::<c_void>();
        let (moved, erased): Handlers = (Some(moverect_internal), Some(erase_internal));
        if self.damage_merge != VTERM_DAMAGE_SCROLL {
            // SAFETY: the scroll only calls back into the handlers it is
            // given, which take the `user` pointer this screen is.
            unsafe { vterm_scroll_rect(region, downward, rightward, moved, erased, user) };
            self.flush_damage();
            let (moved, erased): Handlers = (Some(moverect_user), Some(erase_user));
            // SAFETY: as above, for the reporting pass.
            unsafe { vterm_scroll_rect(region, downward, rightward, moved, erased, user) };
            return;
        }
        if self.damaged.start_row != NO_RECT && !intersects(&region, &self.damaged) {
            self.flush_damage();
        }
        let pending_matches = self.pending_scrollrect == region
            && (self.pending_scroll_downward == 0 && downward == 0
                || self.pending_scroll_rightward == 0 && rightward == 0);
        if self.pending_scrollrect.start_row != NO_RECT && pending_matches {
            self.pending_scroll_downward += downward;
            self.pending_scroll_rightward += rightward;
        } else {
            if self.pending_scrollrect.start_row != NO_RECT {
                self.flush_damage();
            }
            self.pending_scrollrect = region;
            self.pending_scroll_downward = downward;
            self.pending_scroll_rightward = rightward;
        }
        // SAFETY: as for the immediate pass above.
        unsafe { vterm_scroll_rect(region, downward, rightward, moved, erased, user) };
        if self.damaged.start_row != NO_RECT {
            follow_scroll(&mut self.damaged, &region, downward, rightward);
        }
    }
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

/// # Safety
///
/// `vt` must point at a live terminal that has no screen yet.
unsafe fn screen_new(vt: *mut VTerm) -> *mut VTermScreen {
    // SAFETY: the caller promised a live terminal.
    let state = unsafe { vterm_obtain_state(vt) };
    if state.is_null() {
        return core::ptr::null_mut();
    }
    let (mut rows, mut cols) = (0, 0);
    // SAFETY: the same terminal, and both out-scalars are live locals.
    unsafe { vterm_get_size(vt, &mut rows, &mut cols) };
    // SAFETY: `vterm_alloc` answers a live zeroed block of that size, and
    // nothing else has a pointer to it yet.
    let raw = unsafe { vterm_alloc(size_of::<VTermScreen>()) } as *mut VTermScreen;
    // SAFETY: the block just allocated, which nothing else wraps.
    let mut screen = unsafe { Screen::at(raw) };
    screen.vt = vt;
    screen.state = state;
    screen.damage_merge = VTERM_DAMAGE_CELL;
    screen.damaged.start_row = NO_RECT;
    screen.pending_scrollrect.start_row = NO_RECT;
    screen.rows = rows;
    screen.cols = cols;
    screen.set_global_reverse(0);
    screen.set_reflow(0);
    screen.callbacks = core::ptr::null();
    screen.cbdata = core::ptr::null_mut();
    let pen = screen.pen;
    // SAFETY: `rows * cols` is the terminal's own size, which fits.
    screen.buffers[BUFIDX_PRIMARY] = unsafe { alloc_buffer(pen, rows, cols) };
    screen.buffer = screen.buffers[BUFIDX_PRIMARY];
    screen.sb_buffer = core::ptr::null_mut();
    realloc_sb_buffer(&mut screen, cols);
    // SAFETY: the table is static and the data pointer is the screen just
    // built, which outlives the state it is installed in.
    unsafe { vterm_state_set_callbacks(state, STATE_CALLBACKS.ptr(), raw.cast()) };
    raw
}

/// The terminal's screen, creating it on first use.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn vterm_obtain_screen(vt: *mut VTerm) -> *mut VTermScreen {
    // SAFETY: the caller promised a live terminal. The read and the write are
    // separate operations because `screen_new` re-enters the terminal to
    // obtain its state, so no borrow of it may straddle the call.
    let mut screen = unsafe { (*vt).screen };
    if screen.is_null() {
        // SAFETY: the same terminal, which has no screen yet.
        screen = unsafe { screen_new(vt) };
        // SAFETY: as above.
        unsafe { (*vt).screen = screen };
    }
    screen
}

/// # Safety
///
/// `screen` must point at a live screen that nothing uses again, and it and
/// its buffers must have come from [`vterm_alloc`].
pub unsafe fn vterm_screen_free(screen: *mut VTermScreen) {
    // SAFETY: the caller handed the screen over.
    let this = unsafe { Screen::at(screen) };
    let owned = [
        this.buffers[BUFIDX_PRIMARY].cast::<c_void>(),
        this.buffers[BUFIDX_ALTSCREEN].cast::<c_void>(),
        this.sb_buffer.cast::<c_void>(),
        screen.cast::<c_void>(),
    ];
    for block in owned {
        if !block.is_null() {
            // SAFETY: every one of these came from `vterm_alloc` and this is
            // its last use; the screen itself goes last.
            unsafe { vterm_dealloc(block) };
        }
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn vterm_screen_reset(screen: *mut VTermScreen, hard: c_int) {
    // SAFETY: the caller promised a live screen.
    let mut this = unsafe { Screen::at(screen) };
    this.damaged.start_row = NO_RECT;
    this.pending_scrollrect.start_row = NO_RECT;
    let state = this.state;
    // SAFETY: the state `screen_new` obtained. It repaints through the
    // callback table, re-entering this screen, so nothing is borrowed here.
    unsafe { vterm_state_reset(state, hard) };
    this.flush_damage();
}

/// Copies the cell at `pos` into its reported form. Returns 0 for a position
/// outside the screen.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn vterm_screen_get_cell(
    screen: *const VTermScreen,
    pos: VTermPos,
    cell: *mut VTermScreenCell,
) -> c_int {
    // SAFETY: the caller promised a live screen and one cell to write. The
    // wrapper only reads through the pointer — every method used below takes
    // `&self` — so handing it a `*mut` writes nothing.
    let (this, out) = unsafe { (Screen::at(screen.cast_mut()), &mut *cell) };
    this.export_cell(pos, out)
}

impl Screen {
    /// The reported form of the cell at `pos`.
    fn export_cell(&self, pos: VTermPos, out: &mut VTermScreenCell) -> c_int {
        let Some(cell) = self.cell(pos.row, pos.col) else {
            return 0;
        };
        // The gap behind a double-width glyph reports as empty.
        out.schar = if cell.schar == SCHAR_CONTINUATION {
            0
        } else {
            cell.schar
        };
        export_pen(&cell.pen, self.global_reverse() != 0, out);
        let gap = self.cell(pos.row, pos.col + 1);
        let followed_by_gap =
            pos.col < self.cols - 1 && gap.is_some_and(|next| next.schar == SCHAR_CONTINUATION);
        out.width = if followed_by_gap { 2 } else { 1 };
        1
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn vterm_screen_enable_reflow(screen: *mut VTermScreen, reflow: bool) {
    // SAFETY: the caller promised a live screen.
    let mut this = unsafe { Screen::at(screen) };
    this.set_reflow(reflow as u32);
}

/// Allocates the alternate screen buffer, which the terminal starts without.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn vterm_screen_enable_altscreen(screen: *mut VTermScreen, altscreen: c_int) {
    // SAFETY: the caller promised a live screen.
    let mut this = unsafe { Screen::at(screen) };
    if this.buffers[BUFIDX_ALTSCREEN].is_null() && altscreen != 0 {
        let (mut rows, mut cols) = (0, 0);
        let (vt, pen) = (this.vt, this.pen);
        // SAFETY: the terminal that owns this screen, and two live locals.
        unsafe { vterm_get_size(vt, &mut rows, &mut cols) };
        // SAFETY: `rows * cols` is the terminal's own size, which fits.
        this.buffers[BUFIDX_ALTSCREEN] = unsafe { alloc_buffer(pen, rows, cols) };
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn vterm_screen_set_callbacks(
    screen: *mut VTermScreen,
    callbacks: *const VTermScreenCallbacks,
    user: *mut c_void,
) {
    // SAFETY: the caller promised a live screen, and that the table and data
    // pointer it is installing outlive it.
    let mut this = unsafe { Screen::at(screen) };
    this.callbacks = callbacks;
    this.cbdata = user;
}

/// # Safety
///
/// As for [`vterm_screen_set_callbacks`], for the state's fallback table.
pub unsafe fn vterm_screen_set_unrecognised_fallbacks(
    screen: *mut VTermScreen,
    fallbacks: *const VTermStateFallbacks,
    user: *mut c_void,
) {
    // SAFETY: the caller promised a live screen.
    let state = unsafe { Screen::at(screen) }.state;
    // SAFETY: the state `screen_new` obtained, and the caller's promise about
    // the table carries through to it.
    unsafe { vterm_state_set_unrecognised_fallbacks(state, fallbacks, user) };
}

/// Hands the host everything held back by the merge level.
///
/// # Safety
///
/// `screen` must point at a live screen.
pub unsafe fn vterm_screen_flush_damage(screen: *mut VTermScreen) {
    // SAFETY: the caller promised exactly that.
    let mut this = unsafe { Screen::at(screen) };
    this.flush_damage();
}

/// # Safety
///
/// `screen` must point at a live screen.
pub unsafe fn vterm_screen_set_damage_merge(screen: *mut VTermScreen, size: VTermDamageSize) {
    // SAFETY: the caller promised exactly that.
    let mut this = unsafe { Screen::at(screen) };
    this.flush_damage();
    this.damage_merge = size;
}

/// [`convert_color_to_rgb`] against a screen rather than a state.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn vterm_screen_convert_color_to_rgb(
    screen: *const VTermScreen,
    col: *mut VTermColor,
) {
    // SAFETY: the caller promised a live screen, whose state outlives it, and
    // one colour to rewrite in place.
    let state = unsafe { Screen::at(screen.cast_mut()) }.state;
    // SAFETY: as above.
    unsafe { convert_color_to_rgb(&*state, &mut *col) };
}
