//! The mouse: which window and character a screen position names, and what a
//! button does there.
//!
//! Carved by the stage:
//!
//! | child | what |
//! | --- | --- |
//! | [`geom`] | the position a event names, and the family's pure arithmetic |
//! | [`click`] | `%@Func@` click definitions and the popup menu |
//! | [`domouse`] | `do_mouse()`, the Normal/Visual-mode mouse command |
//! | [`scroll`] | the wheel, and the mouse in Insert mode |
//! | [`jump`] | `jump_to_mouse()`, screen position to buffer position |
//! | [`find`] | which window, line and column a screen position names |
//! | [`visual`] | what a click does to the Visual selection |
//!
//! What stays here is the flag alphabet the six share (`MOUSE_*`, `IN_*`,
//! `MOD_MASK_*`), the word-boundary helpers a double click uses, the tab-page
//! click actions, `setmouse()`, `getmousepos()` and the "longest line" scan
//! `'mousescroll'` needs -- plus the [`Win`] and [`ClickDefs`] wrappers, which
//! are where the family's raw pointers are dereferenced.  Everything below
//! them is safe code.
//!
//! Original: `src/nvim/mouse.c`, Vim/Neovim, Vim license.

#![deny(unsafe_op_in_unsafe_fn)]

use core::ffi::{CStr, c_char, c_int};
use core::{ptr, slice};

use crate::src::nvim::charset::vim_iswordc;
use crate::src::nvim::cursor::set_leftcol;
use crate::src::nvim::eval::typval::{tv_dict_add_nr, tv_dict_alloc_ret};
use crate::src::nvim::ex_docmd::{tabpage_close, tabpage_close_other};
use crate::src::nvim::global_cell::GlobalCell;
use crate::src::nvim::grid::grid_adjust;
use crate::src::nvim::main::{
    curbuf, curtab, curwin, first_tabpage, mouse_col, mouse_row, p_sel, tab_page_click_defs,
};
use crate::src::nvim::mbyte::{mb_get_class, utf_head_off, utf8len_tab, utfc_ptr2len};
use crate::src::nvim::plines::{getvcols, win_chartabsize};
use crate::src::nvim::search::BACKWARD;
use crate::src::nvim::state::virtual_active;
use crate::src::nvim::statusline::stl_connected;
use crate::src::nvim::strings::vim_strchr;
use crate::src::nvim::types::{
    EvalFuncData, MotionType, StlClickDefinition, cmdarg_T, colnr_T, linenr_T, pos_T, size_t,
    tabpage_T, typval_T, varnumber_T, win_T,
};
use crate::src::nvim::ui::{ui_check_mouse, ui_cursor_shape};
use crate::src::nvim::window::{
    self, find_tabpage, tabpage_index, tabpage_move, win_drag_status_line, win_drag_vsep_line,
    win_enter, win_valid,
};
use crate::src::nvim::winlayer::{Buf, Pos, Win};

// The carve of the transpiled module; see each child's docs.
mod click;
mod domouse;
mod find;
mod geom;
mod jump;
mod scroll;
mod visual;

pub use self::click::*;
pub use self::domouse::*;
pub use self::find::*;
pub use self::geom::*;
pub use self::jump::*;
pub use self::scroll::*;
pub use self::visual::*;

// ---------------------------------------------------------------------------
// The flag alphabet

pub const kMTCharWise: MotionType = 0;
pub const kMTLineWise: MotionType = 1;

/// Where a click landed, as [`jump_to_mouse`] and `get_fpos_of_mouse()` report
/// it: one `IN_*` value, plus the `MOUSE_*`/`CURSOR_MOVED` bits above them.
pub const IN_UNKNOWN: c_int = 0;
pub const IN_BUFFER: c_int = 1;
pub const IN_STATUS_LINE: c_int = 2;
pub const IN_SEP_LINE: c_int = 4;
pub const IN_OTHER_WIN: c_int = 8;
pub const CURSOR_MOVED: c_int = 256;
pub const MOUSE_FOLD_CLOSE: c_int = 512;
pub const MOUSE_FOLD_OPEN: c_int = 1024;
pub const MOUSE_WINBAR: c_int = 2048;
pub const MOUSE_STATUSCOL: c_int = 4096;

/// What [`jump_to_mouse`] is allowed to do about the event.
pub const MOUSE_FOCUS: c_int = 1;
pub const MOUSE_MAY_VIS: c_int = 2;
pub const MOUSE_DID_MOVE: c_int = 4;
pub const MOUSE_SETPOS: c_int = 8;
pub const MOUSE_MAY_STOP_VIS: c_int = 16;
pub const MOUSE_RELEASED: c_int = 32;

/// Which button the event carries.  `MOUSE_X1`/`MOUSE_X2` sit above the mask
/// the modifiers use, which is why they are not 3 and 4.
pub const MOUSE_LEFT: c_int = 0;
pub const MOUSE_MIDDLE: c_int = 1;
pub const MOUSE_RIGHT: c_int = 2;
pub const MOUSE_X1: c_int = 768;
pub const MOUSE_X2: c_int = 1024;

/// A wheel event's direction, as `cmdarg_T::arg` carries it.
pub const MSCR_DOWN: c_int = 0;
pub const MSCR_UP: c_int = 1;
pub const MSCR_LEFT: c_int = -1;
pub const MSCR_RIGHT: c_int = -2;

pub const MOD_MASK_SHIFT: c_int = 0x2;
pub const MOD_MASK_CTRL: c_int = 0x4;
pub const MOD_MASK_ALT: c_int = 0x8;
pub const MOD_MASK_META: c_int = 0x10;
pub const MOD_MASK_2CLICK: c_int = 0x20;
pub const MOD_MASK_3CLICK: c_int = 0x40;
pub const MOD_MASK_4CLICK: c_int = 0x60;
pub const MOD_MASK_MULTI_CLICK: c_int = MOD_MASK_2CLICK | MOD_MASK_3CLICK | MOD_MASK_4CLICK;

const NUL: c_int = 0;
const VALID_WROW: c_int = 0x1;
const VALID_CROW: c_int = 0x10;
const VALID_BOTLINE: c_int = 0x20;
const VALID_BOTLINE_AP: c_int = 0x40;
const VALID_TOPLINE: c_int = 0x80;
const FR_LEAF: c_int = 0;
const FR_ROW: c_int = 1;
/// The `'mouse'` flag a drag needs before it may start Visual mode.
const MOUSE_VISUAL: c_int = 'v' as c_int;
/// The handle of the screen-wide grid every window draws on without
/// `ext_multigrid`.
const DEFAULT_GRID_HANDLE: c_int = 1;

/// Which of the window's decorations [`jump_to_mouse`] says the click landed
/// on.
#[derive(Clone, Copy)]
pub struct Landed {
    pub winbar: bool,
    pub statuscol: bool,
    pub status_line: bool,
    pub global_status_line: bool,
    pub sep_line: bool,
}

impl Landed {
    pub fn of(jump_flags: c_int) -> Self {
        let status_line = jump_flags & IN_STATUS_LINE != 0;
        Self {
            winbar: jump_flags & MOUSE_WINBAR != 0,
            statuscol: jump_flags & MOUSE_STATUSCOL != 0,
            status_line,
            global_status_line: status_line && global_stl_height() > 0,
            sep_line: jump_flags & IN_SEP_LINE != 0,
        }
    }
}

// ---------------------------------------------------------------------------
// The mouse's own state

/// The top line of the window a click moved focus to, remembered so that the
/// double click that follows still counts as one.
static orig_topline: GlobalCell<linenr_T> = GlobalCell::new(0);
static orig_topfill: GlobalCell<c_int> = GlobalCell::new(0);

/// Whether a press was seen; drags and releases without one are ignored.
static got_click: GlobalCell<bool> = GlobalCell::new(false);

/// The window a drag started in, whose status line or separator the drag
/// moves.
static dragwin: GlobalCell<*mut win_T> = GlobalCell::new(ptr::null_mut());

/// Reset the window being dragged.  To be called when switching tab page.
pub fn reset_dragwin() {
    dragwin.set(ptr::null_mut());
}

/// Remember the window's top line, so a later double click can tell whether
/// the text moved under the pointer.
fn set_mouse_topline(win: Win) {
    orig_topline.set(win.w_topline);
    orig_topfill.set(win.w_topfill);
}

// ---------------------------------------------------------------------------
// The window, as this family reads it
//
// One method per projection off the window pointer or call into a neighbouring
// module, each resting on the promise `Win`'s constructor took.

/// The rows a global status line takes, zero when each window has its own.
fn global_stl_height() -> c_int {
    // SAFETY: reads the 'laststatus' option and the window list.
    unsafe { window::global_stl_height() }
}

/// The leftmost and rightmost virtual column two positions span.
fn vcols_between(win: Win, mut first: pos_T, mut second: pos_T) -> (colnr_T, colnr_T) {
    let (mut left, mut right) = (0, 0);
    let (a, b) = (&raw mut first, &raw mut second);
    // SAFETY: a live window and two local copies of positions in its buffer,
    // which `getvcols` only reads.
    unsafe { getvcols(win.raw(), a, b, &raw mut left, &raw mut right) };
    (left, right)
}

impl Win {
    /// Whether screen column `col` is inside this window's `'statuscolumn'`,
    /// which sits on the right in a 'rightleft' window.
    fn in_statuscolumn(self, col: c_int) -> bool {
        if self.w_onebuf_opt.wo_rl != 0 {
            col >= self.w_view_width - self.col_off()
        } else {
            col < self.col_off()
        }
    }

    /// Whether `'statuscolumn'` is unset for this window.
    fn statuscolumn_empty(self) -> bool {
        // SAFETY: an option string is NUL-terminated, never null.
        unsafe { *self.w_onebuf_opt.wo_stc == NUL as c_char }
    }

    /// Whether the window's status line runs into a window to its right, so
    /// that its last cell is not a vertical separator.
    fn status_line_connected(self) -> bool {
        // SAFETY: a live window.
        unsafe { stl_connected(self.raw()) }
    }

    /// Make this the current window.  Can make the pointer invalid!
    fn enter(self) {
        // SAFETY: a live window.
        unsafe { win_enter(self.raw(), true) };
    }

    /// Whether the window is still in a tab page's list.
    fn is_valid(self) -> bool {
        // SAFETY: `win_valid` only compares the pointer against the lists.
        unsafe { win_valid(self.raw()) }
    }

    /// Move this window's status line down by `count` rows.
    fn drag_status_line(self, count: c_int) {
        // SAFETY: a live window.
        unsafe { win_drag_status_line(self.raw(), count) };
    }

    /// Move this window's vertical separator right by `count` columns.
    fn drag_sep_line(self, count: c_int) {
        // SAFETY: a live window.
        unsafe { win_drag_vsep_line(self.raw(), count) };
    }

    /// The click definitions drawn for this window's status line, winbar and
    /// `'statuscolumn'`, with the number of columns each covers.
    fn status_click_defs(self) -> (Option<ClickDefs>, c_int) {
        // SAFETY: the statusline builder sizes the array as it records it.
        (
            unsafe { ClickDefs::new(self.w_status_click_defs) },
            self.w_status_click_defs_size as c_int,
        )
    }

    fn winbar_click_defs(self) -> Option<ClickDefs> {
        // SAFETY: as `status_click_defs`.
        unsafe { ClickDefs::new(self.w_winbar_click_defs) }
    }

    fn statuscol_click_defs(self) -> (Option<ClickDefs>, c_int) {
        // SAFETY: as `status_click_defs`.
        (
            unsafe { ClickDefs::new(self.w_statuscol_click_defs) },
            self.w_statuscol_click_defs_size as c_int,
        )
    }
}

/// A `%@Func@` click-definition array: one entry per screen column of the line
/// it describes.
#[derive(Clone, Copy)]
pub struct ClickDefs(*mut StlClickDefinition);

impl ClickDefs {
    /// # Safety
    /// `defs` must cover every column [`ClickDefs::at`] is asked for.
    unsafe fn new(defs: *mut StlClickDefinition) -> Option<Self> {
        (!defs.is_null()).then_some(Self(defs))
    }

    /// The tab page line's click definitions, once the tabline has been drawn.
    ///
    /// # Safety
    /// The array must cover every column asked for, which it does for columns
    /// below `Columns`.
    unsafe fn tabline() -> Option<Self> {
        // SAFETY: the caller's promise.
        unsafe { Self::new(tab_page_click_defs.get()) }
    }

    fn raw(self) -> *mut StlClickDefinition {
        self.0
    }

    /// The definition recorded for screen column `col`.
    fn at(self, col: c_int) -> StlClickDefinition {
        // SAFETY: the constructor's promise.
        unsafe { *self.0.offset(col as isize) }
    }
}

// ---------------------------------------------------------------------------
// Words, for the double click

/// Run `f` over line `lnum` of the current buffer, the terminating NUL
/// included: a click may land on the position just past the last character,
/// and [`mouse_class`] answers for the NUL there.
///
/// # Safety
/// `lnum` must be a line of the current buffer.
unsafe fn with_line<R>(lnum: linenr_T, f: impl FnOnce(&[u8]) -> R) -> R {
    // SAFETY: the caller's promise; `ml_get_buf` answers a NUL-terminated
    // line of `ml_get_buf_len` bytes, so one byte more is still in bounds.
    let line = unsafe {
        let buf = Buf::current();
        slice::from_raw_parts(buf.line(lnum).raw().cast(), buf.line_len(lnum) as usize + 1)
    };
    f(line)
}

/// Class of the character at `idx` for the purpose of selection: same class
/// means same word.  0 is blank, 1 a punctuation group, 2 a word character and
/// anything above that a multi-byte word character.
fn mouse_class(line: &[u8], idx: usize) -> c_int {
    let rest = &line[idx..];
    if utf8len_tab[rest[0] as usize] > 1 {
        // SAFETY: a NUL-terminated line, indexed within it.
        return unsafe { mb_get_class(rest.as_ptr().cast()) };
    }

    let c = rest[0] as c_int;
    if c == ' ' as c_int || c == '\t' as c_int {
        return 0;
    }
    // SAFETY: reads the character-class tables, not `c`.
    if unsafe { vim_iswordc(c) } {
        return 2;
    }

    // There are a few special cases where we want certain combinations of
    // characters to be considered as a single word.  These are things like
    // "->", "/ *", "*=", "+=", "&=", "<=", ">=", "!=" etc.  Otherwise, each
    // character is in its own class.
    // SAFETY: a NUL-terminated literal.
    if c != NUL && unsafe { !vim_strchr(c"-+*/%<>&|^!=".as_ptr(), c).is_null() } {
        return 1;
    }
    c
}

/// Bytes the character at `idx` takes, composing characters included.
fn char_len(line: &[u8], idx: usize) -> c_int {
    // SAFETY: a NUL-terminated line, indexed within it.
    unsafe { utfc_ptr2len(line[idx..].as_ptr().cast()) }
}

/// Bytes `idx` sits past the start of the character it is inside.
fn head_off(line: &[u8], idx: usize) -> c_int {
    // SAFETY: a NUL-terminated line, indexed within it.
    unsafe { utf_head_off(line.as_ptr().cast(), line[idx..].as_ptr().cast()) }
}

/// Whether `'selection'` is "exclusive", so that a selection ends just after
/// the last selected character.
fn selection_exclusive() -> bool {
    // SAFETY: an option string is NUL-terminated, never null.
    unsafe { *p_sel.get() == 'e' as c_char }
}

/// Move `pos` back to the start of the word it is in.
///
/// # Safety
/// `pos` must be a live position in the current buffer.
unsafe fn find_start_of_word(pos: Pos) {
    // SAFETY: the caller's promise.
    unsafe { with_line(pos.lnum, |line| start_of_word(pos, line)) };
}

fn start_of_word(mut pos: Pos, line: &[u8]) {
    let cclass = mouse_class(line, pos.col as usize);
    while pos.col > 0 {
        let col = pos.col - 1 - head_off(line, pos.col as usize - 1);
        if mouse_class(line, col as usize) != cclass {
            break;
        }
        pos.col = col;
    }
}

/// Move `pos` forward to the end of the word it is in.  When `'selection'` is
/// "exclusive", to just after the word.
///
/// # Safety
/// `pos` must be a live position in the current buffer.
unsafe fn find_end_of_word(pos: Pos) {
    // SAFETY: the caller's promise.
    unsafe { with_line(pos.lnum, |line| end_of_word(pos, line)) };
}

fn end_of_word(mut pos: Pos, line: &[u8]) {
    let exclusive = selection_exclusive();
    if exclusive && pos.col > 0 {
        pos.col = pos.col - 1 - head_off(line, pos.col as usize - 1);
    }
    let cclass = mouse_class(line, pos.col as usize);
    while line[pos.col as usize] != 0 {
        let col = pos.col + char_len(line, pos.col as usize);
        if mouse_class(line, col as usize) != cclass {
            if exclusive {
                pos.col = col;
            }
            break;
        }
        pos.col = col;
    }
}

// ---------------------------------------------------------------------------
// The tab page line

/// Move the current tab to the tab in the same column as the mouse, or to the
/// end of the tabline if there is no tab there.
fn move_tab_to_mouse(defs: ClickDefs) {
    let tabnr = defs.at(mouse_col.get()).tabnr;
    // The index is read even where the C would not ask for it, which is a
    // pure walk of the tab page list.
    // SAFETY: the tab page list is live from startup to exit.
    let current = unsafe { tabpage_index(curtab.get()) };
    let target = if tabnr <= 0 {
        9999
    } else if tabnr < current {
        tabnr - 1
    } else {
        tabnr
    };
    // SAFETY: as above.
    unsafe { tabpage_move(target) };
}

/// Close tab page `c1`, or the current one when it is 999.
fn mouse_tab_close(c1: c_int) {
    // SAFETY: the tab page globals and list are live from startup to exit.
    let tp: *mut tabpage_T = unsafe {
        if c1 == 999 {
            curtab.get()
        } else {
            find_tabpage(c1)
        }
    };
    if tp == curtab.get() {
        // SAFETY: as above.
        if unsafe { !(*first_tabpage.get()).tp_next.is_null() } {
            // SAFETY: as above.
            unsafe { tabpage_close(false as c_int) };
        }
    } else if !tp.is_null() {
        // SAFETY: as above.
        unsafe { tabpage_close_other(tp, false as c_int) };
    }
}

// ---------------------------------------------------------------------------
// Horizontal scrolling

/// Length of line `lnum` in screen cells, for horizontal scrolling.  The last
/// character is deliberately not counted.
fn scroll_line_len(win: Win, lnum: linenr_T) -> colnr_T {
    // SAFETY: a live window, and a line of the buffer it shows -- so the walk
    // below stays inside that NUL-terminated line.
    unsafe {
        let mut p = win.buffer().line(lnum).raw();
        let mut col: colnr_T = 0;
        while *p != NUL as c_char {
            let numchar = win_chartabsize(win.raw(), p, col);
            p = p.offset(utfc_ptr2len(p) as isize);
            if *p == NUL as c_char {
                break; // don't count the last character
            }
            col += numchar;
        }
        col
    }
}

/// The longest line on screen, closest to the cursor line when several tie.
///
/// Topline and botline can be invalid when displaying is postponed, which is
/// what the range check is for; then only the cursor line is considered.
fn find_longest_lnum(win: Win) -> linenr_T {
    let cursor = win.w_cursor.lnum;
    if !(win.w_topline <= cursor
        && win.w_botline > cursor
        && win.w_botline <= win.buffer().line_count() + 1)
    {
        return cursor;
    }

    let mut ret: linenr_T = 0;
    let mut max: colnr_T = 0;
    for lnum in win.w_topline..win.w_botline {
        let len = scroll_line_len(win, lnum);
        if len > max {
            max = len;
            ret = lnum;
        } else if len == max && (lnum - cursor).abs() < (ret - cursor).abs() {
            ret = lnum;
        }
    }
    ret
}

/// Make a horizontal scroll to `leftcol`.  Answers whether the cursor moved.
fn do_mousescroll_horiz(mut win: Win, leftcol: colnr_T) -> bool {
    if win.w_onebuf_opt.wo_wrap != 0 {
        return false; // no horizontal scrolling when wrapping
    }
    if win.w_leftcol == leftcol {
        return false; // already there
    }

    // When the line of the cursor is too short, move the cursor to the
    // longest visible line.
    // SAFETY: a live window.
    if !unsafe { virtual_active(win.raw()) } && leftcol > scroll_line_len(win, win.w_cursor.lnum) {
        win.w_cursor.lnum = find_longest_lnum(win);
        win.w_cursor.col = 0;
    }

    // SAFETY: a live window.
    unsafe { set_leftcol(leftcol) }
}

// ---------------------------------------------------------------------------
// The drawn screen

/// The virtual column the *drawn screen* records for the clicked cell, and the
/// fold flags it carries.
///
/// `vcols[]` is only meaningful after the window was redrawn -- mainly matters
/// for tests, a user would not click before redrawing -- so the answer is
/// `None` whenever the click is not on a drawn cell of the current window.
fn mouse_check_grid() -> (Option<colnr_T>, c_int) {
    let mut pos = MousePos::current();
    // XXX: this doesn't change `pos.grid` if it is 1, even with multigrid.
    let mut win = match find_win_inner(&mut pos) {
        Some(win) if win.is_current() && win.w_redr_type == 0 => win,
        _ => return (None, 0),
    };

    let (mut start_row, mut start_col) = (0, 0);
    // SAFETY: a live window whose view has been allocated; `grid_adjust`
    // writes the two offsets and answers the grid the view draws on.
    let gp = unsafe { grid_adjust(&raw mut win.w_grid, &raw mut start_row, &raw mut start_col) };
    // SAFETY: a live grid.
    let (handle, drawn, rows, cols) =
        unsafe { ((*gp).handle, !(*gp).chars.is_null(), (*gp).rows, (*gp).cols) };
    let (row, col) = (pos.row + start_row, pos.col + start_col);
    if handle != pos.grid || !drawn || row < 0 || row >= rows || col < 0 || col >= cols {
        return (None, 0);
    }
    // SAFETY: `line_offset` and `vcols` are allocated for the grid's rows x
    // cols, and the position is checked just above.
    let vcol = unsafe {
        *(*gp)
            .vcols
            .add((*(*gp).line_offset.offset(row as isize)).wrapping_add(col as size_t))
    };
    // Use the virtual column from vcols[], it is accurate also after
    // concealed characters.  -2 and -3 are the fold column's markers.
    let flags = match vcol {
        -2 => MOUSE_FOLD_OPEN,
        -3 => MOUSE_FOLD_CLOSE,
        _ => 0,
    };
    ((vcol >= 0).then_some(vcol), flags)
}

// ---------------------------------------------------------------------------
// Entry points

/// Normal and Visual modes implementation for scrolling in direction
/// `cap->arg`, which is one of the `MSCR_` values.
///
/// # Safety
/// `cap` must be a live command argument.
pub unsafe fn nv_mousescroll(cap: *mut cmdarg_T) {
    let old_curwin = curwin.get();

    if mouse_row.get() >= 0 && mouse_col.get() >= 0 {
        // Find the window at the mouse pointer coordinates.
        // NOTE: Must restore "curwin" to "old_curwin" before returning!
        let mut pos = MousePos::current();
        let Some(win) = find_win_inner(&mut pos) else {
            return;
        };
        curwin.set(win.raw());
        curbuf.set(win.buffer().raw());
    }

    // SAFETY: the caller's promise, and `curwin` is a live window.
    unsafe {
        do_mousescroll(cap);
        Win::current().w_redr_status = true;
    }
    curwin.set(old_curwin);
    // SAFETY: `old_curwin` was live and nothing above closes a window.
    curbuf.set(unsafe { Win::current() }.buffer().raw());
}

/// Mouse clicks and drags.
///
/// # Safety
/// `cap` must be a live command argument.
pub unsafe fn nv_mouse(cap: *mut cmdarg_T) {
    // SAFETY: the caller's promise.
    let (oap, cmdchar, count1) = unsafe { ((*cap).oap, (*cap).cmdchar, (*cap).count1) };
    // SAFETY: `oap` is the live operator the command carries, or null.
    unsafe { do_mouse(oap, cmdchar, BACKWARD as c_int, count1, false) };
}

/// Set UI mouse depending on current mode and `'mouse'`.
///
/// Emits mouse_on/mouse_off UI events (unless `'mouse'` is empty).
pub fn setmouse() {
    // SAFETY: both read the editor's mode and the UI list.
    unsafe { ui_cursor_shape() };
    // SAFETY: as above.
    unsafe { ui_check_mouse() };
}

/// `getmousepos()` -- where the pointer last was, in every coordinate system
/// the editor knows.
///
/// # Safety
/// `rettv` must be a live, unset return value.
pub unsafe extern "C" fn f_getmousepos(
    _argvars: *mut typval_T,
    rettv: *mut typval_T,
    _fptr: EvalFuncData,
) {
    // SAFETY: the caller's promise.
    let d = unsafe {
        tv_dict_alloc_ret(rettv);
        (*rettv).vval.v_dict
    };
    let add = |key: &CStr, value: varnumber_T| {
        // SAFETY: the dict just allocated, and a NUL-terminated literal key.
        unsafe { tv_dict_add_nr(d, key.as_ptr(), key.to_bytes().len() as size_t, value) };
    };
    add(c"screenrow", mouse_row.get() as varnumber_T + 1);
    add(c"screencol", mouse_col.get() as varnumber_T + 1);

    let mut pos = MousePos::current();
    let mut winid: varnumber_T = 0;
    let mut winrow: varnumber_T = 0;
    let mut wincol: varnumber_T = 0;
    let mut lnum: linenr_T = 0;
    let mut column: varnumber_T = 0;
    let mut coladd: colnr_T = 0;

    if let Some(win) = find_win_inner(&mut pos) {
        // `w_height` / `w_width` here, where the rest of the family reads
        // `w_view_height` / `w_view_width`: upstream's own mixture, kept.
        let height = win.w_height + win.w_hsep_height + win.w_status_height;
        // The height is adjusted by 1 when there is a bottom border. This is
        // not necessary for a top border since `row` starts at -1 in that
        // case.
        if pos.row < height + win.w_border_adj[2] {
            winid = win.handle as varnumber_T;
            // Adjust by 1 for a top/left border.
            winrow = (pos.row + 1 + win.w_winrow_off) as varnumber_T;
            wincol = (pos.col + 1 + win.w_wincol_off) as varnumber_T;
            if pos.row >= 0 && pos.row < win.w_height && pos.col >= 0 && pos.col < win.w_width {
                (lnum, _) = comp_pos(win, &mut pos.row, &mut pos.col);
                let col;
                (col, coladd) = vcol_to_col(win, lnum, pos.col);
                column = col as varnumber_T + 1;
            }
        }
    }

    add(c"winid", winid);
    add(c"winrow", winrow);
    add(c"wincol", wincol);
    add(c"line", lnum as varnumber_T);
    add(c"column", column);
    add(c"coladd", coladd as varnumber_T);
}
