//! Mouse event handling for Neovim
//!
//! This crate provides Rust implementations of mouse-related functions
//! from `src/nvim/mouse.c`. It handles:
//! - Mouse button state tracking
//! - Click position computation
//! - Drag state machine
//! - Mouse mode flags
//!
//! The crate uses the opaque handle pattern for window access.

#![warn(clippy::all, clippy::pedantic, clippy::nursery)]
#![allow(clippy::missing_safety_doc)]
#![allow(unsafe_code)]
#![allow(clippy::missing_const_for_fn)] // extern "C" functions cannot be const

use nvim_buffer::buf_struct::BufStruct;
use nvim_normal::types::{CmdargT, OpargT};
use nvim_window::win_struct::WinStruct;
use std::ffi::{c_char, c_int};

/// Convert raw `WinHandle` pointer to `&WinStruct`.
#[inline]
unsafe fn win_ref<'a>(wp: WinHandle) -> &'a WinStruct {
    nvim_window::win_struct::win_ref(nvim_window::WinHandle::from_ptr(wp))
}

/// Access `BufStruct` fields from a raw `buf_T` pointer.
#[inline]
unsafe fn bref_raw(buf: *mut std::ffi::c_void) -> &'static BufStruct {
    &*(buf.cast::<BufStruct>())
}

/// Convert raw `WinHandle` pointer to `&mut WinStruct`.
#[inline]
unsafe fn win_mut<'a>(wp: WinHandle) -> &'a mut WinStruct {
    nvim_window::win_struct::win_mut(nvim_window::WinHandle::from_ptr(wp))
}

// =============================================================================
// Mouse button constants (from mouse.h)
// =============================================================================

/// Left mouse button
pub const MOUSE_LEFT: c_int = 0x00;
/// Middle mouse button
pub const MOUSE_MIDDLE: c_int = 0x01;
/// Right mouse button
pub const MOUSE_RIGHT: c_int = 0x02;
/// Mouse button release
pub const MOUSE_RELEASE: c_int = 0x03;
/// Mouse button X1 (6th button)
pub const MOUSE_X1: c_int = 0x300;
/// Mouse button X2
pub const MOUSE_X2: c_int = 0x400;

// =============================================================================
// jump_to_mouse() return values (from mouse.h)
// =============================================================================

/// Unknown position
pub const IN_UNKNOWN: c_int = 0;
/// In buffer text
pub const IN_BUFFER: c_int = 1;
/// On status or command line
pub const IN_STATUS_LINE: c_int = 2;
/// On vertical separator line
pub const IN_SEP_LINE: c_int = 4;
/// In other window but can't go there
pub const IN_OTHER_WIN: c_int = 8;
/// Cursor has moved
pub const CURSOR_MOVED: c_int = 0x100;
/// Clicked on '-' in fold column
pub const MOUSE_FOLD_CLOSE: c_int = 0x200;
/// Clicked on '+' in fold column
pub const MOUSE_FOLD_OPEN: c_int = 0x400;
/// In window toolbar
pub const MOUSE_WINBAR: c_int = 0x800;
/// In 'statuscolumn'
pub const MOUSE_STATUSCOL: c_int = 0x1000;

// =============================================================================
// Flags for jump_to_mouse() (from mouse.h)
// =============================================================================

/// Need to stay in this window
pub const MOUSE_FOCUS: c_int = 0x01;
/// May start Visual mode
pub const MOUSE_MAY_VIS: c_int = 0x02;
/// Only act when mouse has moved
pub const MOUSE_DID_MOVE: c_int = 0x04;
/// Only set current mouse position
pub const MOUSE_SETPOS: c_int = 0x08;
/// May stop Visual mode
pub const MOUSE_MAY_STOP_VIS: c_int = 0x10;
/// Button was released
pub const MOUSE_RELEASED: c_int = 0x20;

// =============================================================================
// Scroll direction constants (from mouse.h)
// =============================================================================

/// Scroll down (must be false/0)
pub const MSCR_DOWN: c_int = 0;
/// Scroll up
pub const MSCR_UP: c_int = 1;
/// Scroll left
pub const MSCR_LEFT: c_int = -1;
/// Scroll right
pub const MSCR_RIGHT: c_int = -2;

// =============================================================================
// Character class for word selection
// =============================================================================

/// Character class for mouse selection:
/// - 0: blank (space, tab)
/// - 1: punctuation groups (-+*/%<>&|^!=)
/// - 2: normal word character
/// - >2: multi-byte word character class
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
pub struct CharClass(pub c_int);

impl CharClass {
    /// Blank character (space or tab)
    pub const BLANK: Self = Self(0);
    /// Punctuation group
    pub const PUNCTUATION: Self = Self(1);
    /// Normal word character
    pub const WORD: Self = Self(2);
}

// =============================================================================
// ASCII constants
// =============================================================================

const NUL: u8 = 0;
const SPACE: u8 = b' ';
const TAB: u8 = b'\t';

// =============================================================================
// Imports from other crates
// =============================================================================

// Re-use existing Rust implementations from mbyte and charset crates
use nvim_charset::rs_vim_iswordc;
// Re-use typval layout from eval crate
use nvim_eval::typval::{TypvalT, TypvalVval};

// =============================================================================
// Type aliases for C types
// =============================================================================

/// Line number type (from `pos_defs.h` — `int32_t`)
#[allow(non_camel_case_types)]
pub type linenr_T = i32;

/// Opaque handle for window pointer
pub type WinHandle = *mut std::ffi::c_void;

/// Opaque handle for tabpage pointer
pub type TabpageHandle = *mut std::ffi::c_void;

/// Opaque handle for `yankreg_T *` (yank register).
pub type YankregHandle = std::ffi::c_void;

// =============================================================================
// Rust-owned static state (moved from C)
// =============================================================================

/// Original topline for double-click detection (was `orig_topline` in mouse.c).
static mut ORIG_TOPLINE: linenr_T = 0;

/// Original topfill for double-click detection (was `orig_topfill` in mouse.c).
static mut ORIG_TOPFILL: c_int = 0;

/// Saved cursor position for `start_arrow` pattern (was `mouse_saved_tpos` in mouse.c).
static mut MOUSE_SAVED_TPOS: PosT = PosT {
    lnum: 0,
    col: 0,
    coladd: 0,
};

/// Whether a mouse click has been received (was `got_click` static in mouse.c).
static mut GOT_CLICK: bool = false;

/// Window currently being dragged, or null (was `dragwin` static in mouse.c).
static mut DRAGWIN: WinHandle = std::ptr::null_mut();

// =============================================================================
// Rust-owned mouse state accessors (exported for C callers)
// =============================================================================

/// Get whether a click was received.
#[no_mangle]
pub unsafe extern "C" fn rs_get_got_click() -> bool {
    GOT_CLICK
}

/// Set whether a click was received.
#[no_mangle]
pub unsafe extern "C" fn rs_set_got_click(val: bool) {
    GOT_CLICK = val;
}

/// Get the window being dragged (opaque handle, may be null).
#[no_mangle]
pub unsafe extern "C" fn rs_get_dragwin() -> WinHandle {
    DRAGWIN
}

/// Set the window being dragged.
#[no_mangle]
pub unsafe extern "C" fn rs_set_dragwin(wp: WinHandle) {
    DRAGWIN = wp;
}

/// Return true if a window is currently being dragged.
#[no_mangle]
pub unsafe extern "C" fn rs_is_dragging() -> bool {
    !DRAGWIN.is_null()
}

/// Get the `mouse_dragging` global value from C.
#[no_mangle]
pub unsafe extern "C" fn rs_get_mouse_dragging() -> c_int {
    nvim_get_mouse_dragging()
}

// =============================================================================
// C accessors for mouse state
// =============================================================================

#[allow(dead_code)]
extern "C" {
    static Rows: c_int;
    static Columns: c_int;

    /// `p_mousescroll_vert` option value (`OptInt` = `int64_t`).
    static p_mousescroll_vert: i64;

    /// Get `mouse_dragging` global value from C.
    fn nvim_get_mouse_dragging() -> c_int;

    /// `p_mousescroll_hor` option value (`OptInt` = `int64_t`).
    static p_mousescroll_hor: i64;

    // --- Mouse globals ---

    /// `mouse_col` global.
    static mouse_col: c_int;

    // --- Tabpage operations ---

    /// Get the current tabpage.
    fn nvim_get_curtab() -> TabpageHandle;

    /// Get the first tabpage.
    fn nvim_get_first_tabpage() -> TabpageHandle;

    // nvim_first_tabpage_has_next replaced by nvim_get_first_tabpage + direct tp_next check

    /// Move current tab to position nr.
    #[link_name = "tabpage_move"]
    fn tabpage_move(nr: c_int);

    /// Close the current tabpage.
    #[link_name = "tabpage_close"]
    fn tabpage_close(forceit: c_int);

    /// Close another tabpage.
    #[link_name = "tabpage_close_other"]
    fn tabpage_close_other(tp: TabpageHandle, forceit: c_int);

    /// Get tabpage index (Rust impl in window crate).
    fn rs_tabpage_index(ftp: TabpageHandle) -> c_int;

    /// Find tabpage by number (Rust impl in window crate).
    fn rs_find_tabpage(n: c_int) -> TabpageHandle;

    // --- UI operations ---

    /// Call `ui_cursor_shape()`.
    #[link_name = "ui_cursor_shape"]
    fn ui_cursor_shape();

    /// Call `ui_check_mouse()`.
    #[link_name = "ui_check_mouse"]
    fn ui_check_mouse();
}

// =============================================================================
// Character Classification Functions
// =============================================================================

/// Get class of a character for mouse word selection.
///
/// Returns:
/// - 0: blank (space, tab)
/// - 1: punctuation groups
/// - 2: normal word character
/// - >2: multi-byte word character class
///
/// # Safety
/// `p` must be a valid pointer to a NUL-terminated string.
#[no_mangle]
pub unsafe extern "C" fn rs_get_mouse_class(p: *const c_char) -> c_int {
    if p.is_null() {
        return 0;
    }

    // Check for multi-byte character
    let first_byte = *p.cast::<u8>();
    if utf_ptr2len(p) > 1 {
        return mb_get_class(p);
    }

    // Single-byte character checks
    if first_byte == SPACE || first_byte == TAB {
        return CharClass::BLANK.0;
    }

    if rs_vim_iswordc(c_int::from(first_byte)) {
        return CharClass::WORD.0;
    }

    // Check for punctuation groups (-+*/%<>&|^!=)
    if first_byte != NUL {
        let punct_chars = b"-+*/%<>&|^!=";
        if punct_chars.contains(&first_byte) {
            return CharClass::PUNCTUATION.0;
        }
    }

    // Each character is its own class
    c_int::from(first_byte)
}

/// Check if 'mousemodel' is set to "popup" or "`popup_setpos`".
///
/// Returns true when the first character of 'mousem' is 'p'.
///
/// # Safety
/// `p_mousem` must be a valid pointer to a NUL-terminated string.
#[no_mangle]
pub unsafe extern "C" fn rs_mouse_model_popup(p_mousem: *const c_char) -> bool {
    if p_mousem.is_null() {
        return false;
    }
    *p_mousem.cast::<u8>() == b'p'
}

// =============================================================================
// External UTF-8 helpers from mbyte crate
// =============================================================================

extern "C" {
    /// Get length of UTF-8 character at pointer (including composing chars)
    fn utfc_ptr2len(p: *const c_char) -> c_int;

    /// Get length of UTF-8 character at pointer (not including composing chars)
    fn utf_ptr2len(p: *const c_char) -> c_int;

    /// Get character class of multibyte character
    fn mb_get_class(p: *const c_char) -> c_int;

    /// Get offset to start of UTF-8 character
    fn utf_head_off(base: *const c_char, p: *const c_char) -> c_int;
}

// =============================================================================
// Word Boundary Detection Functions
// =============================================================================

/// Find the start of the word at the given column.
///
/// Given a line and starting column, returns the column position of the
/// start of the word that contains the starting column position.
///
/// # Safety
/// - `line` must be a valid pointer to a NUL-terminated string.
/// - `col` must be a valid byte offset within the line.
#[no_mangle]
#[allow(clippy::cast_sign_loss)]
pub unsafe extern "C" fn rs_find_start_of_word(line: *const c_char, col: c_int) -> c_int {
    if line.is_null() || col <= 0 {
        return 0;
    }

    let mut pos_col = col;
    let cclass = rs_get_mouse_class(line.add(pos_col as usize));

    while pos_col > 0 {
        // Move back one character
        let mut new_col = pos_col - 1;
        // Adjust for multi-byte character start
        new_col -= utf_head_off(line, line.add(new_col as usize));

        // Check if character class changed
        if rs_get_mouse_class(line.add(new_col as usize)) != cclass {
            break;
        }
        pos_col = new_col;
    }

    pos_col
}

/// Find the end of the word at the given column.
///
/// Given a line and starting column, returns the column position of the
/// end of the word that contains the starting column position.
/// If `sel_exclusive` is true, the position is just after the word (for exclusive selection).
///
/// # Safety
/// - `line` must be a valid pointer to a NUL-terminated string.
/// - `col` must be a valid byte offset within the line.
#[no_mangle]
#[allow(clippy::cast_sign_loss)]
pub unsafe extern "C" fn rs_find_end_of_word(
    line: *const c_char,
    col: c_int,
    sel_exclusive: bool,
) -> c_int {
    if line.is_null() {
        return col;
    }

    let mut pos_col = col;

    // For exclusive selection, adjust start position if col > 0
    if sel_exclusive && pos_col > 0 {
        pos_col -= 1;
        pos_col -= utf_head_off(line, line.add(pos_col as usize));
    }

    let cclass = rs_get_mouse_class(line.add(pos_col as usize));

    // Scan forward while same character class
    while *line.add(pos_col as usize) != 0 {
        let next_col = pos_col + utfc_ptr2len(line.add(pos_col as usize));
        if rs_get_mouse_class(line.add(next_col as usize)) != cclass {
            // For exclusive selection, move past the last character
            if sel_exclusive {
                return next_col;
            }
            break;
        }
        pos_col = next_col;
    }

    pos_col
}

// =============================================================================
// Fold Column Click Detection
// =============================================================================

/// Virtual column value indicating a fold open marker was clicked.
pub const VCOL_FOLD_OPEN: c_int = -2;

/// Virtual column value indicating a fold close marker was clicked.
pub const VCOL_FOLD_CLOSE: c_int = -3;

/// Result of checking a click for fold column interaction.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FoldClickResult {
    /// The flags to add based on the click (e.g., `MOUSE_FOLD_OPEN` or `MOUSE_FOLD_CLOSE`)
    pub flags: c_int,
    /// Whether to use the vcol value
    pub use_vcol: bool,
}

/// Check if a virtual column value indicates a fold column click.
///
/// Returns fold flags to add and whether to use the vcol value for cursor positioning.
/// - If vcol >= 0: use the value for cursor positioning
/// - If vcol == -2: fold open marker clicked
/// - If vcol == -3: fold close marker clicked
/// - Otherwise: no special handling
#[no_mangle]
pub const extern "C" fn rs_check_fold_click(vcol: c_int) -> FoldClickResult {
    match vcol {
        x if x >= 0 => FoldClickResult {
            flags: 0,
            use_vcol: true,
        },
        VCOL_FOLD_OPEN => FoldClickResult {
            flags: MOUSE_FOLD_OPEN,
            use_vcol: false,
        },
        VCOL_FOLD_CLOSE => FoldClickResult {
            flags: MOUSE_FOLD_CLOSE,
            use_vcol: false,
        },
        _ => FoldClickResult {
            flags: 0,
            use_vcol: false,
        },
    }
}

// =============================================================================
// Mouse Button Parsing
// =============================================================================

/// Result of parsing a mouse button event.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MouseButtonResult {
    /// The button that was pressed (`MOUSE_LEFT`, `MOUSE_MIDDLE`, `MOUSE_RIGHT`, etc.)
    pub button: c_int,
    /// Whether this is a click event (vs drag or release)
    pub is_click: bool,
    /// Whether this is a drag event
    pub is_drag: bool,
}

// =============================================================================
// Visual Mode Selection Type
// =============================================================================

/// Visual mode character selection ('v')
pub const VISUAL_CHAR: c_int = b'v' as c_int;

/// Visual mode line selection ('V')
pub const VISUAL_LINE: c_int = b'V' as c_int;

/// Visual mode block selection (Ctrl-V = 0x16)
pub const VISUAL_BLOCK: c_int = 0x16;

/// Multi-click mask bits
pub const MOD_MASK_MULTI_CLICK: c_int = 0x60;

/// Double-click mask
pub const MOD_MASK_2CLICK: c_int = 0x20;

/// Triple-click mask
pub const MOD_MASK_3CLICK: c_int = 0x40;

/// Quadruple-click mask
pub const MOD_MASK_4CLICK: c_int = 0x60;

/// Alt modifier mask
pub const MOD_MASK_ALT: c_int = 0x08;

/// Determine the visual selection mode based on multi-click count and modifiers.
///
/// - Double-click: character-wise ('v'), or block-wise (Ctrl-V) if ALT is pressed
/// - Triple-click: line-wise ('V')
/// - Quadruple-click: block-wise (Ctrl-V)
///
/// Returns 0 if the click count doesn't correspond to a selection mode change.
#[no_mangle]
pub const extern "C" fn rs_compute_selection_mode(mmask: c_int) -> c_int {
    let multi_click = mmask & MOD_MASK_MULTI_CLICK;
    let alt_pressed = (mmask & MOD_MASK_ALT) != 0;

    match multi_click {
        MOD_MASK_2CLICK => {
            if alt_pressed {
                VISUAL_BLOCK
            } else {
                VISUAL_CHAR
            }
        }
        MOD_MASK_3CLICK => VISUAL_LINE,
        MOD_MASK_4CLICK => VISUAL_BLOCK,
        _ => 0, // No selection mode change
    }
}

/// Get the click count from a modifier mask (1, 2, 3, or 4).
#[no_mangle]
pub const extern "C" fn rs_get_click_count(mmask: c_int) -> c_int {
    let multi_click = mmask & MOD_MASK_MULTI_CLICK;

    match multi_click {
        MOD_MASK_4CLICK => 4,
        MOD_MASK_3CLICK => 3,
        MOD_MASK_2CLICK => 2,
        _ => 1,
    }
}

/// Check if this is a multi-click (double, triple, or quadruple click).
#[no_mangle]
pub const extern "C" fn rs_is_multi_click(mmask: c_int) -> bool {
    (mmask & MOD_MASK_MULTI_CLICK) != 0
}

/// Check if this is specifically a double-click.
#[no_mangle]
pub const extern "C" fn rs_is_double_click(mmask: c_int) -> bool {
    (mmask & MOD_MASK_MULTI_CLICK) == MOD_MASK_2CLICK
}

// =============================================================================
// Scroll Computation
// =============================================================================

/// Column number type
#[allow(non_camel_case_types)]
pub type colnr_T = c_int;

/// Compute the new left column for horizontal scrolling.
///
/// # Arguments
/// * `current_leftcol` - Current left column of the window
/// * `scroll_dir` - Scroll direction (`MSCR_LEFT` or `MSCR_RIGHT`)
/// * `step` - Number of columns to scroll
///
/// # Returns
/// The new left column value, clamped to >= 0
#[no_mangle]
pub const extern "C" fn rs_compute_horiz_scroll(
    current_leftcol: colnr_T,
    scroll_dir: c_int,
    step: c_int,
) -> colnr_T {
    let delta = if scroll_dir == MSCR_RIGHT {
        -step
    } else {
        step
    };

    let new_col = current_leftcol + delta;
    if new_col < 0 {
        0
    } else {
        new_col
    }
}

/// Compute the scroll line count for mouse wheel scrolling.
///
/// # Arguments
/// * `shift_or_ctrl` - Whether shift or ctrl is pressed
/// * `visible_lines` - Number of visible lines (`w_botline` - `w_topline`)
/// * `default_scroll` - Default scroll amount from `p_mousescroll_vert`
///
/// # Returns
/// The number of lines to scroll
#[no_mangle]
pub const extern "C" fn rs_compute_scroll_lines(
    shift_or_ctrl: bool,
    visible_lines: c_int,
    default_scroll: c_int,
) -> c_int {
    if shift_or_ctrl {
        // Scroll whole page
        visible_lines
    } else {
        default_scroll
    }
}

/// Check if scroll direction is vertical (up or down).
#[no_mangle]
pub const extern "C" fn rs_is_vertical_scroll(scroll_dir: c_int) -> bool {
    scroll_dir == MSCR_UP || scroll_dir == MSCR_DOWN
}

/// Check if scroll direction is horizontal (left or right).
#[no_mangle]
pub const extern "C" fn rs_is_horizontal_scroll(scroll_dir: c_int) -> bool {
    scroll_dir == MSCR_LEFT || scroll_dir == MSCR_RIGHT
}

// =============================================================================
// Phase 1 — Simple Leaf Helpers
// =============================================================================

/// Set `orig_topline` and `orig_topfill` from the given window.
/// Used when jumping to another window so that double-click detection works.
///
/// # Safety
/// `wp` must be a valid window handle.
#[no_mangle]
pub unsafe extern "C" fn rs_set_mouse_topline(wp: WinHandle) {
    ORIG_TOPLINE = win_ref(wp).w_topline;
    ORIG_TOPFILL = win_ref(wp).w_topfill;
}

/// Set UI mouse depending on current mode and 'mouse'.
///
/// Emits `mouse_on`/`mouse_off` UI event (unless 'mouse' is empty).
#[export_name = "setmouse"]
pub unsafe extern "C" fn rs_setmouse() {
    ui_cursor_shape();
    ui_check_mouse();
}

/// Reset the window being dragged to NULL.
/// Called when switching tab page.
#[export_name = "reset_dragwin"]
pub unsafe extern "C" fn rs_reset_dragwin() {
    DRAGWIN = std::ptr::null_mut();
}

/// Move the current tab to the tab in the same column as the mouse,
/// or to the end of the tabline if there is no tab there.
///
/// # Safety
/// Requires valid `tab_page_click_defs` array and valid `mouse_col`.
#[no_mangle]
pub unsafe extern "C" fn rs_move_tab_to_mouse() {
    let tab_defs = nvim_get_tab_page_click_defs_ptr();
    let col = mouse_col;
    #[allow(clippy::cast_sign_loss)]
    let tabnr = if tab_defs.is_null() || col < 0 {
        0
    } else {
        (*tab_defs.cast::<StlClickDefinition>().add(col as usize)).tabnr
    };
    if tabnr <= 0 {
        tabpage_move(9999);
    } else if tabnr < rs_tabpage_index(nvim_get_curtab()) {
        tabpage_move(tabnr - 1);
    } else {
        tabpage_move(tabnr);
    }
}

/// Close the current or specified tab page.
///
/// # Arguments
/// * `c1` - tabpage number, or 999 for the current tabpage
///
/// # Safety
/// Requires valid tabpage state.
#[no_mangle]
pub unsafe extern "C" fn rs_mouse_tab_close(c1: c_int) {
    let tp = if c1 == 999 {
        nvim_get_curtab()
    } else {
        rs_find_tabpage(c1)
    };

    let curtab = nvim_get_curtab();
    if tp == curtab {
        let ft = nvim_get_first_tabpage();
        let has_next_tab = !ft.is_null()
            && !(*(ft as *const nvim_window::tabpage_struct::TabpageStruct))
                .tp_next
                .is_null();
        if has_next_tab {
            tabpage_close(0); // false
        }
    } else if !tp.is_null() {
        tabpage_close_other(tp, 0); // false
    }
}

// =============================================================================
// Phase 3 — Position Computation
// =============================================================================

extern "C" {
    /// Get `w_p_rl` (right-to-left) field.
    /// Check if window may have filler lines.
    fn win_may_fill(wp: WinHandle) -> bool;

    /// Get filler lines for a line in a window.
    fn nvim_win_get_fill(wp: WinHandle, lnum: linenr_T) -> c_int;

    /// Get number of physical lines a buffer line takes (no fill).
    fn nvim_plines_win_nofill(wp: WinHandle, lnum: linenr_T, limit: c_int) -> c_int;

    /// Get number of physical lines a buffer line takes (with fill).
    fn nvim_plines_win(wp: WinHandle, lnum: linenr_T, limit: c_int) -> c_int;

    /// Check if line is folded, optionally get first/last line of fold.
    fn nvim_hasFolding(
        wp: WinHandle,
        lnum: linenr_T,
        firstp: *mut linenr_T,
        lastp: *mut linenr_T,
    ) -> c_int;

    /// Check if a line is concealed by decoration.
    fn nvim_decor_conceal_line(wp: WinHandle, row: c_int, check_cursor: c_int) -> c_int;

    /// Get column offset for line numbers etc.
    #[link_name = "win_col_off"]
    fn nvim_win_col_off(wp: WinHandle) -> c_int;

    /// Get secondary column offset.
    #[link_name = "win_col_off2"]
    fn nvim_win_col_off2(wp: WinHandle) -> c_int;

    /// Convert virtual column to character column (C impl with charsize).
    fn nvim_vcol2col(wp: WinHandle, lnum: linenr_T, vcol: c_int, coladdp: *mut c_int) -> c_int;
}

/// Convert a virtual (screen) column to a character column.
///
/// Delegates to the C `nvim_vcol2col` which has the charsize iteration
/// infrastructure (inline functions not accessible from Rust).
///
/// # Safety
/// `wp` must be a valid window handle.
#[export_name = "vcol2col"]
pub unsafe extern "C" fn rs_vcol2col(
    wp: WinHandle,
    lnum: linenr_T,
    vcol: c_int,
    coladdp: *mut c_int,
) -> c_int {
    nvim_vcol2col(wp, lnum, vcol, coladdp)
}

/// Compute the buffer line position from the screen position.
///
/// Given a window and row/col screen coordinates, computes the buffer
/// line number and column. Handles folds, wrapping, smoothscroll,
/// right-to-left mode, and concealed lines.
///
/// Returns true if the position is below the last line.
///
/// # Safety
/// All pointers must be valid. `win` must be a valid window handle.
#[export_name = "mouse_comp_pos"]
pub unsafe extern "C" fn rs_mouse_comp_pos(
    win: WinHandle,
    rowp: *mut c_int,
    colp: *mut c_int,
    lnump: *mut linenr_T,
) -> bool {
    let mut col = *colp;
    let mut row = *rowp;
    let mut retval = false;

    if win_ref(win).w_p_rl() != 0 {
        col = win_ref(win).w_view_width - 1 - col;
    }

    let topline = win_ref(win).w_topline;
    let mut lnum = topline;
    let line_count = nvim_win_buf_line_count(win);
    let view_width = win_ref(win).w_view_width;

    while row > 0 {
        // Don't include filler lines in "count"
        let count = if win_may_fill(win) {
            let fill = if lnum == topline {
                win_ref(win).w_topfill
            } else {
                nvim_win_get_fill(win, lnum)
            };
            row -= fill;
            nvim_plines_win_nofill(win, lnum, 0)
        } else {
            nvim_plines_win(win, lnum, 0)
        };

        let mut adjusted_count = count;
        if win_ref(win).w_skipcol > 0 && lnum == topline {
            let width1 = view_width - nvim_win_col_off(win);
            if width1 > 0 {
                let skipcol = win_ref(win).w_skipcol;
                let skip_lines = if skipcol > width1 {
                    (skipcol - width1) / (width1 + nvim_win_col_off2(win)) + 1
                } else {
                    i32::from(skipcol > 0)
                };
                adjusted_count -= skip_lines;
            }
        }

        if adjusted_count > row {
            break; // Position is in this buffer line.
        }

        nvim_hasFolding(
            win,
            lnum,
            std::ptr::null_mut(),
            std::ptr::addr_of_mut!(lnum),
        );

        if lnum == line_count {
            retval = true;
            break; // past end of file
        }
        row -= adjusted_count;
        lnum += 1;
    }

    // Mouse row reached, adjust lnum for concealed lines.
    while lnum < line_count && nvim_decor_conceal_line(win, lnum - 1, 0) != 0 {
        lnum += 1;
        nvim_hasFolding(
            win,
            lnum,
            std::ptr::null_mut(),
            std::ptr::addr_of_mut!(lnum),
        );
    }

    if !retval {
        // Compute the column without wrapping.
        let off = nvim_win_col_off(win) - nvim_win_col_off2(win);
        if col < off {
            col = off;
        }
        col += row * (view_width - off);

        // Add skip column for the topline.
        if lnum == topline {
            col += win_ref(win).w_skipcol;
        }
    }

    if win_ref(win).w_p_wrap() == 0 {
        col += win_ref(win).w_leftcol;
    }

    // skip line number and fold column in front of the line
    col -= nvim_win_col_off(win);
    if col < 0 {
        col = 0;
    }

    *colp = col;
    *rowp = row;
    *lnump = lnum;
    retval
}

// =============================================================================
// Phase 2 — Scroll Helpers
// =============================================================================

extern "C" {
    /// Get current window.
    fn nvim_get_curwin() -> WinHandle;

    /// Get buffer line count for this window.
    fn nvim_win_buf_line_count(wp: WinHandle) -> linenr_T;

    /// Get line text from current buffer.
    fn nvim_ml_get(lnum: linenr_T) -> *const c_char;

    /// Check if virtual editing is active for window.
    fn nvim_win_virtual_active(wp: WinHandle) -> c_int;

    /// Compute char screen width at position in window.
    #[link_name = "win_chartabsize"]
    fn rs_win_chartabsize(wp: WinHandle, p: *const c_char, col: c_int) -> c_int;

    /// Set `w_leftcol` and adjust cursor. Returns true if cursor moved.
    #[link_name = "set_leftcol"]
    fn rs_set_leftcol(leftcol: c_int) -> bool;
}

/// Return length of line `lnum` for horizontal scrolling.
///
/// Iterates characters using `win_chartabsize` and advances with
/// `utfc_ptr2len`, stopping before the last character.
///
/// # Safety
/// Requires valid current window and buffer state.
#[no_mangle]
#[allow(clippy::cast_sign_loss)]
pub unsafe extern "C" fn rs_scroll_line_len(lnum: linenr_T) -> c_int {
    let curwin = nvim_get_curwin();
    let mut col: c_int = 0;
    let line = nvim_ml_get(lnum);
    if line.is_null() || *line == 0 {
        return 0;
    }

    let mut p = line;
    loop {
        let numchar = rs_win_chartabsize(curwin, p, col);
        let len = utfc_ptr2len(p);
        p = p.offset(len as isize);
        if *p == 0 {
            // Don't count the last character
            break;
        }
        col += numchar;
    }
    col
}

/// Find the longest visible line number.
///
/// Returns the line number of the longest visible line, closest to the
/// cursor line. Used for horizontal scrolling.
///
/// # Safety
/// Requires valid current window and buffer state.
#[no_mangle]
pub unsafe extern "C" fn rs_find_longest_lnum() -> linenr_T {
    let curwin = nvim_get_curwin();
    let topline = win_ref(curwin).w_topline;
    let botline = win_ref(curwin).w_botline;
    let cursor_lnum = win_ref(curwin).w_cursor.lnum;
    let line_count = nvim_win_buf_line_count(curwin);

    // Check for reasonable line numbers
    if topline > cursor_lnum || botline <= cursor_lnum || botline > line_count + 1 {
        // Use cursor line only
        return cursor_lnum;
    }

    let mut ret: linenr_T = 0;
    let mut max: c_int = 0;

    let mut lnum = topline;
    while lnum < botline {
        let len = rs_scroll_line_len(lnum);
        if len > max {
            max = len;
            ret = lnum;
        } else if len == max && (lnum - cursor_lnum).abs() < (ret - cursor_lnum).abs() {
            ret = lnum;
        }
        lnum += 1;
    }

    if ret == 0 {
        cursor_lnum
    } else {
        ret
    }
}

/// Make a horizontal scroll to `leftcol`.
///
/// Returns true if the cursor moved, false otherwise.
///
/// # Safety
/// Requires valid current window state.
#[no_mangle]
pub unsafe extern "C" fn rs_do_mousescroll_horiz(leftcol: c_int) -> bool {
    let curwin = nvim_get_curwin();

    if win_ref(curwin).w_p_wrap() != 0 {
        return false; // no horizontal scrolling when wrapping
    }
    if win_ref(curwin).w_leftcol == leftcol {
        return false; // already there
    }

    // When the line of the cursor is too short, move the cursor to the
    // longest visible line.
    if nvim_win_virtual_active(curwin) == 0
        && leftcol > rs_scroll_line_len(win_ref(curwin).w_cursor.lnum)
    {
        win_mut(curwin).w_cursor.lnum = rs_find_longest_lnum();
        win_mut(curwin).w_cursor.col = 0;
    }

    rs_set_leftcol(leftcol)
}

// =============================================================================
// pos_T representation
// =============================================================================

/// Buffer position: line number, column, and column add.
/// Matches C `pos_T` layout exactly.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct PosT {
    pub lnum: linenr_T,
    pub col: colnr_T,
    pub coladd: colnr_T,
}

/// Opaque handle for `cmdarg_T *`
pub type CmdargHandle = *mut nvim_normal::types::CmdargT;

/// Opaque handle for `oparg_T *`
pub type OpargHandle = *mut nvim_normal::types::OpargT;

// =============================================================================
// Phase 4 — Window Find Functions
// =============================================================================

extern "C" {
    // Grid/frame accessors used by rs_mouse_find_grid_win and rs_frame_find_win.

    /// Get `msg_grid.handle`.
    fn nvim_msg_grid_get_handle() -> c_int;

    /// Get `msg_grid_pos` global.
    fn nvim_mouse_get_msg_grid_pos() -> c_int;

    /// `get_win_by_grid_handle(handle)` — find window by grid handle.
    fn nvim_get_win_by_grid_handle(handle: c_int) -> WinHandle;

    /// Return true if `wp->w_config.mouse`.
    fn nvim_win_config_get_mouse(wp: WinHandle) -> bool;

    /// Return true if grid is `pum_grid`.
    fn nvim_grid_is_pum_grid(grid: *mut std::ffi::c_void) -> bool;

    /// Call `ui_comp_mouse_focus(row, col)` and return opaque `ScreenGrid*`.
    fn nvim_ui_comp_mouse_focus(row: c_int, col: c_int) -> *mut std::ffi::c_void;

    /// Find window in current tab whose `w_grid_alloc` matches `grid`.
    /// Adjusts `*rowp` and `*colp` relative to that window's grid.
    fn nvim_curtab_find_win_by_grid_alloc(
        grid: *mut std::ffi::c_void,
        rowp: *mut c_int,
        colp: *mut c_int,
    ) -> WinHandle;

    /// Get `fp->fr_layout`.
    fn nvim_ses_frame_get_layout(fp: *mut std::ffi::c_void) -> c_int;

    /// Get `fp->fr_child`.
    fn nvim_ses_frame_get_child(fp: *mut std::ffi::c_void) -> *mut std::ffi::c_void;

    /// Get `fp->fr_next`.
    fn nvim_ses_frame_get_next(fp: *mut std::ffi::c_void) -> *mut std::ffi::c_void;

    /// Get `fp->fr_width`.
    fn nvim_ses_frame_get_width(fp: *mut std::ffi::c_void) -> c_int;

    /// Get `fp->fr_height`.
    fn nvim_ses_frame_get_height(fp: *mut std::ffi::c_void) -> c_int;

    /// Get `topframe` global.
    fn nvim_ses_get_topframe() -> *mut std::ffi::c_void;

    /// Get `firstwin->w_winrow` (row of the first non-floating window).
    fn nvim_get_firstwin_winrow() -> c_int;

    /// Find window in current tab matching `fp->fr_win`.
    fn nvim_curtab_find_win_for_frame(fp: *mut std::ffi::c_void) -> WinHandle;
}

// Frame layout constants (from `buffer_defs.h`).
const FR_LEAF: c_int = 0;
const FR_ROW: c_int = 1;

/// Resolve grid handle to window and adjust row/col.
///
/// Port of `nvim_mouse_find_grid_win` from mouse.c. The C function is now
/// deleted; this Rust version is the authoritative implementation.
///
/// # Safety
/// All pointers must be valid.
unsafe fn rs_mouse_find_grid_win(
    gridp: *mut c_int,
    rowp: *mut c_int,
    colp: *mut c_int,
) -> WinHandle {
    if *gridp == nvim_msg_grid_get_handle() {
        // Message grid: adjust row to the default grid's coordinate space.
        *rowp += nvim_mouse_get_msg_grid_pos();
        *gridp = DEFAULT_GRID_HANDLE;
    } else if *gridp > 1 {
        // Specific grid handle: find the owning window.
        let wp = nvim_get_win_by_grid_handle(*gridp);
        if !wp.is_null()
            && nvim_window::win_struct::win_grid_alloc_has_chars(nvim_window::WinHandle::from_ptr(
                wp,
            ))
            && (!win_ref(wp).w_floating || nvim_win_config_get_mouse(wp))
        {
            let view_height = win_ref(wp).w_view_height;
            let view_width = win_ref(wp).w_view_width;
            let row_offset = nvim_gridview_get_row_offset(nvim_win_get_grid(wp));
            let col_offset = nvim_gridview_get_col_offset(nvim_win_get_grid(wp));
            *rowp = (*rowp - row_offset).min(view_height - 1);
            *colp = (*colp - col_offset).min(view_width - 1);
            return wp;
        }
    } else if *gridp == 0 {
        // Compositor grid: find which window/grid the mouse is over.
        let grid = nvim_ui_comp_mouse_focus(*rowp, *colp);
        if nvim_grid_is_pum_grid(grid) {
            *gridp = nvim_screengrid_get_handle(grid);
            *rowp -= nvim_screengrid_get_comp_row(grid);
            *colp -= nvim_screengrid_get_comp_col(grid);
            // The popup menu doesn't have a window.
            return std::ptr::null_mut();
        }
        let wp = nvim_curtab_find_win_by_grid_alloc(grid, rowp, colp);
        if !wp.is_null() {
            *gridp = nvim_screengrid_get_handle(grid);
            return wp;
        }
        // No grid found — fall back to default grid (e.g., split separator).
        *gridp = DEFAULT_GRID_HANDLE;
    }
    std::ptr::null_mut()
}

/// Traverse frame tree to find window at row/col position.
///
/// Port of `nvim_frame_find_win` from mouse.c. The C function is now
/// deleted; this Rust version is the authoritative implementation.
///
/// # Safety
/// All pointers must be valid.
unsafe fn rs_frame_find_win(rowp: *mut c_int, colp: *mut c_int) -> WinHandle {
    let mut fp = nvim_ses_get_topframe();
    *rowp -= nvim_get_firstwin_winrow();

    loop {
        let layout = nvim_ses_frame_get_layout(fp);
        if layout == FR_LEAF {
            break;
        }
        let mut child = nvim_ses_frame_get_child(fp);
        if layout == FR_ROW {
            loop {
                let next = nvim_ses_frame_get_next(child);
                if next.is_null() {
                    break;
                }
                if *colp < nvim_ses_frame_get_width(child) {
                    break;
                }
                *colp -= nvim_ses_frame_get_width(child);
                child = next;
            }
        } else {
            // FR_COL
            loop {
                let next = nvim_ses_frame_get_next(child);
                if next.is_null() {
                    break;
                }
                if *rowp < nvim_ses_frame_get_height(child) {
                    break;
                }
                *rowp -= nvim_ses_frame_get_height(child);
                child = next;
            }
        }
        fp = child;
    }

    let wp = nvim_curtab_find_win_for_frame(fp);
    if !wp.is_null() {
        *rowp -= win_ref(wp).w_winbar_height;
    }
    wp
}

/// Find the window at grid position `*rowp`, `*colp`.
///
/// Updates positions to be relative to the top-left of the window inner area.
/// Returns NULL when something is wrong.
///
/// # Safety
/// All pointers must be valid.
#[export_name = "mouse_find_win_inner"]
pub unsafe extern "C" fn rs_mouse_find_win_inner(
    gridp: *mut c_int,
    rowp: *mut c_int,
    colp: *mut c_int,
) -> WinHandle {
    // First try grid-based resolution (floating windows, multigrid, etc.)
    let wp_grid = rs_mouse_find_grid_win(gridp, rowp, colp);
    if !wp_grid.is_null() {
        return wp_grid;
    }
    if *gridp > 1 {
        return std::ptr::null_mut();
    }

    // Fall back to frame tree traversal
    rs_frame_find_win(rowp, colp)
}

/// Find the window at grid position `*rowp`, `*colp`.
///
/// Updates positions to be relative to the top-left of the window
/// (including winbar, sign column, etc.).
/// Returns NULL when something is wrong.
///
/// # Safety
/// All pointers must be valid.
#[export_name = "mouse_find_win_outer"]
pub unsafe extern "C" fn rs_mouse_find_win_outer(
    gridp: *mut c_int,
    rowp: *mut c_int,
    colp: *mut c_int,
) -> WinHandle {
    let wp = rs_mouse_find_win_inner(gridp, rowp, colp);
    if !wp.is_null() {
        *rowp += win_ref(wp).w_winrow_off;
        *colp += win_ref(wp).w_wincol_off;
    }
    wp
}

// =============================================================================
// Phase 5 — Mid-level Functions
// =============================================================================

extern "C" {
    /// `mouse_grid` global.
    static mouse_grid: c_int;

    /// `mouse_row` global.
    static mouse_row: c_int;

    /// Get `w_p_stc` field (statuscolumn option string).
    fn nvim_win_get_p_stc(wp: WinHandle) -> *const c_char;

    /// Get `Rows` global.

    /// Get `p_ch` global (command height).
    static mut p_ch: i64;

    /// Get `global_stl_height()`.
    #[link_name = "rs_global_stl_height"]
    fn nvim_global_stl_height() -> c_int;

    // --- Phase 2 grid accessors ---

    /// Get `&wp->w_grid` (`GridView` pointer).
    fn nvim_win_get_grid(wp: WinHandle) -> *mut std::ffi::c_void;

    /// Adjust `GridView` and return target `ScreenGrid`.
    fn rs_grid_adjust(
        view: *mut std::ffi::c_void,
        row_off: *mut c_int,
        col_off: *mut c_int,
    ) -> *mut std::ffi::c_void;

    /// Get `grid->handle`.
    fn nvim_screengrid_get_handle(grid: *mut std::ffi::c_void) -> c_int;

    /// Get `grid->chars` (null means no cells).
    fn nvim_screengrid_get_chars(grid: *mut std::ffi::c_void) -> *mut std::ffi::c_void;

    /// Get `grid->rows`.
    fn nvim_screengrid_get_rows(grid: *mut std::ffi::c_void) -> c_int;

    /// Get `grid->cols`.
    fn nvim_screengrid_get_cols(grid: *mut std::ffi::c_void) -> c_int;

    /// Get `grid->line_offset` array pointer.
    fn nvim_screengrid_get_line_offset(grid: *mut std::ffi::c_void) -> *const usize;

    /// Get `grid->vcols` array pointer.
    fn nvim_screengrid_get_vcols(grid: *mut std::ffi::c_void) -> *const colnr_T;

    // --- Phase 2 accessors ---

    /// Set `curwin`.
    fn nvim_set_curwin(wp: WinHandle);

    /// Set `curbuf` (`buf_T*`).
    fn nvim_set_curbuf(buf: *mut std::ffi::c_void);

    /// `mod_mask` global.
    static mod_mask: c_int;

    /// Get `State` global.
    fn nvim_get_state() -> c_int;

    /// Call `pagescroll(dir, count, half)`.
    #[link_name = "pagescroll"]
    fn pagescroll(dir: c_int, count: c_int, half: c_int) -> c_int;

    /// Call `undisplay_dollar()`.
    #[link_name = "nvim_textfmt_undisplay_dollar"]
    fn nvim_undisplay_dollar();

    /// Get `&curwin->w_cursor` pointer.
    fn nvim_win_get_cursor_ptr(wp: WinHandle) -> *mut PosT;

    /// Call `start_arrow(end_insert_pos)` (Rust impl in edit crate).
    fn start_arrow(end_insert_pos: *mut PosT);

    /// Call `set_can_cindent(val)`.
    fn nvim_set_can_cindent(val: c_int);

    /// Call `redraw_statuslines()`.
    fn redraw_statuslines();

    /// Check if a window's buffer is a prompt buffer.
    fn nvim_win_bt_prompt(wp: WinHandle) -> c_int;

    /// Set `buf->b_prompt_insert` field.
    fn nvim_buf_set_prompt_insert(buf: *mut std::ffi::c_void, val: c_int);

    /// Check if a window pointer is valid.
    fn rs_win_valid(win: WinHandle) -> c_int;

    /// Call `rs_nv_scroll_line(cap)`.
    fn rs_nv_scroll_line(cap: CmdargHandle);

    /// Check if popup menu is visible.
    fn pum_visible() -> bool;
}

// =============================================================================
// Phase 3 — Popup Menu Implementation (extern declarations)
// =============================================================================

extern "C" {
    /// `VIsual_active` global.
    static mut VIsual_active: bool;

    /// `VIsual_mode` global (character: 'v', 'V', or Ctrl-V).
    static VIsual_mode: c_int;

    /// `VIsual` global (start of visual selection, `pos_T`).
    static VIsual: PosT;

    /// `p_mousem` option (`char *`).
    #[link_name = "p_mousem"]
    static P_MOUSEM: *const c_char;

    /// `getvcols(wp, pos1, pos2, left, right)` — Rust impl in plines crate.
    fn rs_getvcols(wp: WinHandle, pos1: PosT, pos2: PosT, left: *mut colnr_T, right: *mut colnr_T);

    /// `getvcol(wp, pos, scol, ccol, ecol)`.
    #[link_name = "getvcol"]
    fn nvim_getvcol(
        wp: WinHandle,
        pos: *mut PosT,
        scol: *mut colnr_T,
        ccol: *mut colnr_T,
        ecol: *mut colnr_T,
    );

    /// Show the popup menu.
    fn show_popupmenu();

    /// Update the screen display.
    fn update_screen() -> c_int;

    /// Set cursor position on screen.
    fn setcursor();

    /// Flush UI events.
    fn ui_flush();

    /// Mark current buffer for redraw with given type.
    fn redraw_curbuf_later(typ: c_int);
}

/// Screen update type: buffer not changed (from drawscreen.h).
const UPD_VALID: c_int = 10;

/// Screen update type: redisplay inverted part (from drawscreen.h).
const UPD_INVERTED: c_int = 20;

/// Ctrl-V character code for block-visual mode detection.
const CTRL_V: c_int = 0x16;

/// Compare two `PosT` values: return true if `a < b`.
#[inline]
fn pos_lt(a: PosT, b: PosT) -> bool {
    a.lnum < b.lnum || (a.lnum == b.lnum && a.col < b.col)
}

/// Compare two `PosT` values: return true if `a <= b`.
#[inline]
fn pos_ltoreq(a: PosT, b: PosT) -> bool {
    a.lnum < b.lnum || (a.lnum == b.lnum && a.col <= b.col)
}

// =============================================================================
// Phase 6 — jump_to_mouse_impl extern declarations
// =============================================================================

extern "C" {
    /// Set `mouse_past_bottom` global.
    fn nvim_set_mouse_past_bottom(val: bool);

    /// Set `mouse_past_eol` global.
    fn nvim_set_mouse_past_eol(val: bool);

    /// Exit Visual mode.
    fn end_visual_mode();

    /// Mark the current window for later redraw.
    fn redraw_later(wp: WinHandle, upd: c_int);

    /// Enter a window (possibly changing `curwin`/`curbuf`).
    fn win_enter(wp: WinHandle, undo_sync: bool);

    /// Check and fix `w_topfill`.
    fn check_topfill(wp: WinHandle, down: c_int);

    /// Get physical line count (`plines_win` without filler lines).
    fn plines_win(wp: WinHandle, lnum: linenr_T, winheight: bool) -> c_int;

    /// Get filler line count for `lnum` in `wp` (`win_get_fill`).
    fn win_get_fill(wp: WinHandle, lnum: linenr_T) -> c_int;

    /// Advance the cursor column.
    fn coladvance(wp: WinHandle, col: c_int) -> c_int;

    /// Get `w_buffer` for curwin (the current buffer handle).
    fn nvim_get_curbuf() -> *mut std::ffi::c_void;

    /// Clear valid bits: `w_valid &= ~bits`.
    fn nvim_win_clear_valid_bits(wp: WinHandle, bits: c_int);

    /// Get `cmdwin_type` global.
    fn nvim_get_cmdwin_type() -> c_int;

    /// Get `cmdwin_win` global.
    fn nvim_get_cmdwin_win() -> WinHandle;

    /// Set `VIsual_reselect`.
    static mut VIsual_reselect: bool;

    /// Check `stl_connected(wp)`.
    fn stl_connected(wp: WinHandle) -> c_int;

    /// Get `GridView::comp_row` for `w_grid_alloc`.
    fn nvim_screengrid_get_comp_row(gp: *mut std::ffi::c_void) -> c_int;

    /// Get `GridView::comp_col` for `w_grid_alloc`.
    fn nvim_screengrid_get_comp_col(gp: *mut std::ffi::c_void) -> c_int;

    /// Get `GridView::row_offset` from a `GridView*`.
    fn nvim_gridview_get_row_offset(view: *mut std::ffi::c_void) -> c_int;

    /// Get `GridView::col_offset` from a `GridView*`.
    fn nvim_gridview_get_col_offset(view: *mut std::ffi::c_void) -> c_int;

    /// `msg_silent` global — direct static access.
    static mut msg_silent: c_int;

    /// Set `redraw_cmdline = true`.
    fn nvim_set_redraw_cmdline(val: bool);

    /// `setmouse()` — update UI mouse state.
    #[link_name = "setmouse"]
    fn setmouse_global();

    /// `rs_may_start_select(c)` — check if we may start Select mode.
    fn rs_may_start_select(c: c_int);

    // VIsual_active already declared mutable above

    /// Set `VIsual` position fields.
    fn nvim_set_VIsual_pos(lnum: linenr_T, col: c_int, coladd: c_int);

    /// `p_smd` option (showmode).
    static p_smd: c_int;

    // static VIsual: PosT;  — already declared in Phase 3 block
    // static VIsual_active: bool;  — already declared in Phase 3 block
}

/// Valid-bits masks (from `buffer_defs.h`).
const VALID_WROW: c_int = 0x01;
const VALID_CROW: c_int = 0x10;
const VALID_BOTLINE: c_int = 0x20;
const VALID_BOTLINE_AP: c_int = 0x40;
const VALID_TOPLINE: c_int = 0x80;

/// `DEFAULT_GRID_HANDLE` from `grid.h`
const DEFAULT_GRID_HANDLE: c_int = 1;

// `IN_OTHER_WIN` is already defined above as `IN_OTHER_WIN: c_int = 8`.

/// Check the mouse click grid for virtual column and fold flags.
///
/// Port of `nvim_mouse_check_grid_impl` from mouse.c. Looks up the virtual
/// column (`vcols[]`) at the mouse click position using the current window's
/// grid, and sets fold-open/close flags if appropriate.
///
/// # Safety
/// All pointers must be valid.
#[no_mangle]
#[allow(clippy::cast_sign_loss)]
pub unsafe extern "C" fn rs_mouse_check_grid(vcolp: *mut colnr_T, flagsp: *mut c_int) {
    let mut click_grid = mouse_grid;
    let mut click_row = mouse_row;
    let mut click_col = mouse_col;

    let curwin = nvim_get_curwin();

    // Only act when the grid corresponds to curwin and it has been redrawn.
    if rs_mouse_find_win_inner(
        std::ptr::addr_of_mut!(click_grid),
        std::ptr::addr_of_mut!(click_row),
        std::ptr::addr_of_mut!(click_col),
    ) != curwin
        || win_ref(curwin).w_redr_type != 0
    {
        return;
    }

    let mut start_row: c_int = 0;
    let mut start_col: c_int = 0;
    let gp = rs_grid_adjust(
        nvim_win_get_grid(curwin),
        std::ptr::addr_of_mut!(start_row),
        std::ptr::addr_of_mut!(start_col),
    );

    if gp.is_null()
        || nvim_screengrid_get_handle(gp) != click_grid
        || nvim_screengrid_get_chars(gp).is_null()
    {
        return;
    }

    click_row += start_row;
    click_col += start_col;

    let rows = nvim_screengrid_get_rows(gp);
    let cols = nvim_screengrid_get_cols(gp);
    if click_row < 0 || click_row >= rows || click_col < 0 || click_col >= cols {
        return;
    }

    let line_offset = nvim_screengrid_get_line_offset(gp);
    let vcol_arr = nvim_screengrid_get_vcols(gp);
    if line_offset.is_null() || vcol_arr.is_null() {
        return;
    }

    let off = *line_offset.add(click_row as usize) + click_col as usize;
    let col_from_screen = *vcol_arr.add(off);

    if col_from_screen >= 0 {
        *vcolp = col_from_screen;
    }
    if col_from_screen == -2 {
        *flagsp |= MOUSE_FOLD_OPEN;
    } else if col_from_screen == -3 {
        *flagsp |= MOUSE_FOLD_CLOSE;
    }
}

/// Get the file position of the mouse click.
///
/// Returns `IN_BUFFER` if the click is on a valid buffer position,
/// or one of the other `IN_*` / `MOUSE_*` flags.
///
/// # Safety
/// `mpos` must be a valid pointer.
#[no_mangle]
pub unsafe extern "C" fn rs_get_fpos_of_mouse(mpos: *mut PosT) -> c_int {
    let row = mouse_row;
    let col = mouse_col;
    if row < 0 || col < 0 {
        return IN_UNKNOWN;
    }

    let mut grid = mouse_grid;
    let mut frow = row;
    let mut fcol = col;
    let wp = rs_mouse_find_win_inner(
        std::ptr::addr_of_mut!(grid),
        std::ptr::addr_of_mut!(frow),
        std::ptr::addr_of_mut!(fcol),
    );
    if wp.is_null() {
        return IN_UNKNOWN;
    }
    let winrow = frow;
    let wincol = fcol;

    let below_buffer = rs_mouse_comp_pos(
        wp,
        std::ptr::addr_of_mut!(frow),
        std::ptr::addr_of_mut!(fcol),
        std::ptr::addr_of_mut!((*mpos).lnum),
    );

    // Check for statuscolumn click
    if !below_buffer {
        let p_stc = nvim_win_get_p_stc(wp);
        if !p_stc.is_null() && *p_stc != 0 {
            let col_off = nvim_win_col_off(wp);
            if win_ref(wp).w_p_rl() != 0 {
                if wincol >= win_ref(wp).w_view_width - col_off {
                    return MOUSE_STATUSCOL;
                }
            } else if wincol < col_off {
                return MOUSE_STATUSCOL;
            }
        }
    }

    let view_height = win_ref(wp).w_view_height;
    let status_height = win_ref(wp).w_status_height;

    if winrow >= view_height + status_height {
        // Below window — check for global status line
        let mouse_grid_val = mouse_grid;
        let rows = Rows;
        #[allow(clippy::cast_possible_truncation)]
        let p_ch_int = p_ch as c_int;
        if mouse_grid_val <= 1
            && row < rows - p_ch_int
            && row >= rows - p_ch_int - nvim_global_stl_height()
        {
            return IN_STATUS_LINE;
        }
        return IN_UNKNOWN;
    } else if winrow >= view_height {
        return IN_STATUS_LINE;
    }

    if winrow < 0 && winrow + win_ref(wp).w_winbar_height >= 0 {
        return MOUSE_WINBAR;
    }

    if wincol >= win_ref(wp).w_view_width {
        return IN_SEP_LINE;
    }

    let curwin = nvim_get_curwin();
    if wp != curwin || below_buffer {
        return IN_UNKNOWN;
    }

    (*mpos).col = rs_vcol2col(
        wp,
        (*mpos).lnum,
        fcol,
        std::ptr::addr_of_mut!((*mpos).coladd),
    );
    IN_BUFFER
}

/// Handle popup menu action for mouse click.
///
/// Port of `nvim_do_popup_impl` from mouse.c. Sets cursor position if
/// `mousemodel=popup_setpos` and the click is outside the current visual
/// selection, then shows the popup menu.
///
/// # Safety
/// Requires valid mouse and visual mode state.
#[no_mangle]
#[allow(clippy::similar_names)]
pub unsafe extern "C" fn rs_do_popup(
    which_button: c_int,
    m_pos_flag: c_int,
    mut m_pos: PosT,
) -> c_int {
    let mut jump_flags: c_int = 0;

    // Compare p_mousem to "popup_setpos"
    let popup_setpos = b"popup_setpos\0";
    if !P_MOUSEM.is_null() && {
        let n = popup_setpos.len();
        let s = std::slice::from_raw_parts(P_MOUSEM.cast::<u8>(), n);
        s == popup_setpos
    } {
        // First set the cursor position before showing the popup menu.
        if VIsual_active {
            // Set MOUSE_MAY_STOP_VIS if we are outside the selection
            // or the current window (might have false negative here).
            if m_pos_flag == IN_BUFFER {
                let curwin = nvim_get_curwin();
                let cursor = *nvim_win_get_cursor_ptr(curwin);
                if VIsual_mode == c_int::from(b'V') {
                    if (cursor.lnum <= VIsual.lnum
                        && (m_pos.lnum < cursor.lnum || VIsual.lnum < m_pos.lnum))
                        || (VIsual.lnum < cursor.lnum
                            && (m_pos.lnum < VIsual.lnum || cursor.lnum < m_pos.lnum))
                    {
                        jump_flags = MOUSE_MAY_STOP_VIS;
                    }
                } else if (pos_ltoreq(cursor, VIsual)
                    && (pos_lt(m_pos, cursor) || pos_lt(VIsual, m_pos)))
                    || (pos_lt(VIsual, cursor) && (pos_lt(m_pos, VIsual) || pos_lt(cursor, m_pos)))
                {
                    jump_flags = MOUSE_MAY_STOP_VIS;
                } else if VIsual_mode == CTRL_V {
                    let mut leftcol: colnr_T = 0;
                    let mut rightcol: colnr_T = 0;
                    rs_getvcols(curwin, cursor, VIsual, &raw mut leftcol, &raw mut rightcol);
                    nvim_getvcol(
                        curwin,
                        &raw mut m_pos,
                        std::ptr::null_mut(),
                        &raw mut m_pos.col,
                        std::ptr::null_mut(),
                    );
                    if m_pos.col < leftcol || m_pos.col > rightcol {
                        jump_flags = MOUSE_MAY_STOP_VIS;
                    }
                }
            } else {
                jump_flags = MOUSE_MAY_STOP_VIS;
            }
        } else {
            jump_flags = MOUSE_MAY_STOP_VIS;
        }
    }

    if jump_flags != 0 {
        jump_flags = rs_jump_to_mouse(jump_flags, std::ptr::null_mut(), which_button);
        redraw_curbuf_later(if VIsual_active {
            UPD_INVERTED
        } else {
            UPD_VALID
        });
        update_screen();
        setcursor();
        ui_flush(); // Update before showing popup menu
    }

    show_popupmenu();
    GOT_CLICK = false; // ignore release events
    jump_flags
}

/// Handle mouse click in Insert mode.
///
/// # Safety
/// Requires valid window and insert mode state.
#[export_name = "ins_mouse"]
pub unsafe extern "C" fn rs_ins_mouse(c: c_int) {
    let old_curwin = nvim_get_curwin();

    nvim_undisplay_dollar();
    // Save curwin->w_cursor as tpos before do_mouse changes it.
    MOUSE_SAVED_TPOS = *nvim_win_get_cursor_ptr(old_curwin);

    if rs_do_mouse(std::ptr::null_mut(), c, -1 /* BACKWARD */, 1, false) {
        let new_curwin = nvim_get_curwin();

        if nvim_get_curwin() != old_curwin && rs_win_valid(old_curwin) != 0 {
            // Mouse took us to another window. Go back to the previous one
            // to stop insert there properly.
            nvim_set_curwin(old_curwin);
            let old_buf = win_ref(old_curwin).w_buffer;
            nvim_set_curbuf(old_buf);
            if nvim_win_bt_prompt(old_curwin) != 0 {
                // Restart Insert mode when re-entering the prompt buffer.
                nvim_buf_set_prompt_insert(old_buf, c_int::from(b'A'));
            }
        }
        let same_window = nvim_get_curwin() == old_curwin;
        let tpos_ptr = if same_window {
            std::ptr::addr_of_mut!(MOUSE_SAVED_TPOS)
        } else {
            std::ptr::null_mut()
        };
        start_arrow(tpos_ptr);
        if nvim_get_curwin() != new_curwin && rs_win_valid(new_curwin) != 0 {
            nvim_set_curwin(new_curwin);
            nvim_set_curbuf(win_ref(new_curwin).w_buffer);
        }
        nvim_set_can_cindent(1);
    }

    // Redraw status lines (in case another window became active).
    redraw_statuslines();
}

// MODE_NORMAL value from state_defs.h
const MODE_NORMAL: c_int = 0x01;
// FORWARD/BACKWARD directions
const FORWARD: c_int = 1;
// BACKWARD is -1 (same as MSCR_LEFT = -1, different from MSCR_UP = 1)
// Defined explicitly here to match C's BACKWARD constant
const BACKWARD_DIR: c_int = 0; // pagescroll BACKWARD is 0 in C

/// Common mouse wheel scrolling for Normal/Visual modes.
///
/// Scroll direction is from `cap->arg`:
/// - `MSCR_UP` / `MSCR_DOWN`: vertical scroll
/// - `MSCR_LEFT` / `MSCR_RIGHT`: horizontal scroll
///
/// # Safety
/// `cap` must be a valid `cmdarg_T` pointer.
#[export_name = "do_mousescroll"]
#[allow(clippy::cast_possible_truncation)]
pub unsafe extern "C" fn rs_do_mousescroll(cap: CmdargHandle) {
    let mm = mod_mask;
    // MOD_MASK_SHIFT = 0x02, MOD_MASK_CTRL = 0x04
    let shift_or_ctrl = (mm & (0x02 | 0x04)) != 0;
    let cap_arg = (*cap.cast::<CmdargT>()).arg;
    let curwin = nvim_get_curwin();

    if cap_arg == MSCR_UP || cap_arg == MSCR_DOWN {
        // Vertical scrolling
        let state = nvim_get_state();
        if (state & MODE_NORMAL) != 0 && shift_or_ctrl {
            // Whole page up or down
            let dir = if cap_arg != 0 { FORWARD } else { BACKWARD_DIR };
            pagescroll(dir, 1, 0);
        } else {
            let count = if shift_or_ctrl {
                // Whole page up or down: botline - topline
                win_ref(curwin).w_botline - win_ref(curwin).w_topline
            } else {
                p_mousescroll_vert as c_int
            };
            if count > 0 {
                (*cap.cast::<CmdargT>()).count1 = count;
                (*cap.cast::<CmdargT>()).count0 = count;
                rs_nv_scroll_line(cap);
            }
        }
    } else {
        // Horizontal scrolling
        let step = if shift_or_ctrl {
            win_ref(curwin).w_view_width
        } else {
            p_mousescroll_hor as c_int
        };
        let leftcol = win_ref(curwin).w_leftcol + if cap_arg == MSCR_RIGHT { -step } else { step };
        let leftcol = if leftcol < 0 { 0 } else { leftcol };
        rs_do_mousescroll_horiz(leftcol);
    }
}

/// Normal/Visual mode mouse scroll handler.
///
/// Finds the window at the mouse position, scrolls it, then restores `curwin`.
///
/// # Safety
/// `cap` must be a valid `cmdarg_T` pointer.
#[export_name = "nv_mousescroll"]
pub unsafe extern "C" fn rs_nv_mousescroll(cap: CmdargHandle) {
    let old_curwin = nvim_get_curwin();

    let mr = mouse_row;
    let mc = mouse_col;
    if mr >= 0 && mc >= 0 {
        // Find the window at the mouse pointer coordinates.
        // NOTE: Must restore curwin to old_curwin before returning!
        let mut grid = mouse_grid;
        let mut row = mr;
        let mut col = mc;
        let wp = rs_mouse_find_win_inner(
            std::ptr::addr_of_mut!(grid),
            std::ptr::addr_of_mut!(row),
            std::ptr::addr_of_mut!(col),
        );
        if wp.is_null() {
            nvim_set_curwin(old_curwin);
            return;
        }
        nvim_set_curwin(wp);
        nvim_set_curbuf(win_ref(wp).w_buffer);
    }

    // Call the common mouse scroll function shared with other modes.
    rs_do_mousescroll(cap);

    let curwin = nvim_get_curwin();
    win_mut(curwin).w_redr_status = (1) != 0;
    nvim_set_curwin(old_curwin);
    nvim_set_curbuf(win_ref(old_curwin).w_buffer);
}

// =============================================================================
// Phase 4 — ins_mousescroll implementation
// Key constants computed via TERMCAP2KEY(KS_EXTRA, KE_xxx):
//   TERMCAP2KEY(a, b) = -(a + (b << 8))
//   KS_EXTRA = 253
// =============================================================================

/// `K_MOUSEUP` key code (scroll wheel up).
const K_MOUSEUP: c_int = -(253 + (76i32 << 8)); // -19709

/// `K_MOUSEDOWN` key code (scroll wheel down).
const K_MOUSEDOWN: c_int = -(253 + (75i32 << 8)); // -19453

/// `K_MOUSELEFT` key code (scroll wheel left).
const K_MOUSELEFT: c_int = -(253 + (77i32 << 8)); // -19965

/// `K_MOUSERIGHT` key code (scroll wheel right).
const K_MOUSERIGHT: c_int = -(253 + (78i32 << 8)); // -20221

/// Insert mode mouse scroll handler.
///
/// Port of `nvim_ins_mousescroll_impl` from mouse.c. Constructs a `CmdargT`,
/// optionally changes `curwin` to the window under the mouse, scrolls, and
/// calls `start_arrow` if the cursor moved.
///
/// # Safety
/// Requires valid window and insert mode state.
#[export_name = "ins_mousescroll"]
#[allow(clippy::field_reassign_with_default)]
pub unsafe extern "C" fn rs_ins_mousescroll(dir: c_int) {
    let mut oa = OpargT::default();
    let mut cap = CmdargT::default();
    cap.oap = &raw mut oa;
    cap.arg = dir;
    cap.cmdchar = match dir {
        MSCR_UP => K_MOUSEUP,
        MSCR_DOWN => K_MOUSEDOWN,
        MSCR_LEFT => K_MOUSELEFT,
        MSCR_RIGHT => K_MOUSERIGHT,
        _ => {
            // Invalid argument — silently ignore (mirrors siemsg path without crashing)
            return;
        }
    };

    let old_curwin = nvim_get_curwin();
    let mr = mouse_row;
    let mc = mouse_col;
    if mr >= 0 && mc >= 0 {
        // Find the window at the mouse pointer coordinates.
        // NOTE: Must restore curwin to old_curwin before returning!
        let mut grid = mouse_grid;
        let mut row = mr;
        let mut col = mc;
        let wp = rs_mouse_find_win_inner(
            std::ptr::addr_of_mut!(grid),
            std::ptr::addr_of_mut!(row),
            std::ptr::addr_of_mut!(col),
        );
        if wp.is_null() {
            return;
        }
        nvim_set_curwin(wp);
        nvim_set_curbuf(win_ref(wp).w_buffer);
    }

    let curwin = nvim_get_curwin();
    if curwin == old_curwin {
        // Don't scroll the current window if the popup menu is visible.
        if pum_visible() {
            return;
        }
        nvim_undisplay_dollar();
    }

    let orig_cursor = *nvim_win_get_cursor_ptr(curwin);

    // Call the common mouse scroll function shared with other modes.
    rs_do_mousescroll(std::ptr::addr_of_mut!(cap));

    let curwin_after = nvim_get_curwin();
    win_mut(curwin_after).w_redr_status = (1) != 0;
    nvim_set_curwin(old_curwin);
    nvim_set_curbuf(win_ref(old_curwin).w_buffer);

    // If cursor moved, notify insert mode via start_arrow.
    let new_cursor = *nvim_win_get_cursor_ptr(old_curwin);
    if new_cursor.lnum != orig_cursor.lnum
        || new_cursor.col != orig_cursor.col
        || new_cursor.coladd != orig_cursor.coladd
    {
        let mut orig = orig_cursor;
        start_arrow(std::ptr::addr_of_mut!(orig));
        nvim_set_can_cindent(1);
    }
}

// =============================================================================
// Phase 6 — jump_to_mouse
// =============================================================================

extern "C" {
    /// Get `w_grid_alloc` pointer (`ScreenGrid`\*) from a window.
    fn nvim_win_get_w_grid_alloc(wp: WinHandle) -> *mut std::ffi::c_void;

    /// Count foldcolumn characters for window `wp`.
    fn rs_win_fdccol_count(wp: WinHandle) -> c_int;

    /// Drag the status line for `dragwin` by `offset` rows.
    fn rs_win_drag_status_line(dragwin: WinHandle, offset: c_int);

    /// Drag the vertical separator for `dragwin` by `offset` columns.
    fn rs_win_drag_vsep_line(dragwin: WinHandle, offset: c_int);
}

// FAIL / OK return codes (from vim.h).
const FAIL: c_int = 0;

// Static locals for jump_to_mouse — preserved across calls.
static mut STATUS_LINE_OFFSET: c_int = 0;
static mut SEP_LINE_OFFSET: c_int = 0;
static mut ON_STATUS_LINE: bool = false;
static mut ON_SEP_LINE: bool = false;
static mut ON_WINBAR: bool = false;
static mut ON_STATUSCOL: bool = false;
static mut PREV_ROW: c_int = -1;
static mut PREV_COL: c_int = -1;
static mut DID_DRAG: c_int = 0;

/// Handle the `goto retnomove` case: return the appropriate IN_* constant
/// without moving the cursor. May stop Visual mode if needed.
///
/// # Safety
/// Reads static state; must only be called from `rs_jump_to_mouse`.
unsafe fn retnomove(flags: c_int) -> c_int {
    if STATUS_LINE_OFFSET != 0 {
        return IN_STATUS_LINE;
    }
    if SEP_LINE_OFFSET != 0 {
        return IN_SEP_LINE;
    }
    if ON_WINBAR {
        return IN_OTHER_WIN | MOUSE_WINBAR;
    }
    if ON_STATUSCOL {
        return IN_OTHER_WIN | MOUSE_STATUSCOL;
    }
    if (flags & MOUSE_MAY_STOP_VIS) != 0 {
        end_visual_mode();
        redraw_curbuf_later(UPD_INVERTED);
    }
    IN_BUFFER
}

/// Move the cursor to the specified row and column on the screen.
///
/// Change current window if necessary. Returns an integer with the
/// `CURSOR_MOVED` bit set if the cursor has moved or unset otherwise.
///
/// # Safety
/// `inclusive` may be null. Otherwise must be a valid pointer.
#[export_name = "jump_to_mouse"]
#[allow(clippy::too_many_lines)]
pub unsafe extern "C" fn rs_jump_to_mouse(
    mut flags: c_int,
    inclusive: *mut bool,
    which_button: c_int,
) -> c_int {
    let mut row = mouse_row;
    let mut col = mouse_col;
    let mut grid = mouse_grid;
    #[allow(unused_assignments)]
    let mut fdc: c_int = 0;
    let keep_focus = (flags & MOUSE_FOCUS) != 0;

    nvim_set_mouse_past_bottom(false);
    nvim_set_mouse_past_eol(false);

    if (flags & MOUSE_RELEASED) != 0 {
        // On button release we may change window focus if positioned on a
        // status line and no dragging happened.
        if rs_is_dragging() && DID_DRAG == 0 {
            flags &= !(MOUSE_FOCUS | MOUSE_DID_MOVE);
        }
        rs_set_dragwin(std::ptr::null_mut());
        DID_DRAG = 0;
    }

    if (flags & MOUSE_DID_MOVE) != 0 && PREV_ROW == mouse_row && PREV_COL == mouse_col {
        return retnomove(flags);
    }

    PREV_ROW = mouse_row;
    PREV_COL = mouse_col;

    if (flags & MOUSE_SETPOS) != 0 {
        return retnomove(flags);
    }

    if row < 0 || col < 0 {
        return IN_UNKNOWN;
    }

    let wp = rs_mouse_find_win_inner(
        std::ptr::addr_of_mut!(grid),
        std::ptr::addr_of_mut!(row),
        std::ptr::addr_of_mut!(col),
    );
    if wp.is_null() {
        return IN_UNKNOWN;
    }

    let winbar_height = win_ref(wp).w_winbar_height;
    let w_height = win_ref(wp).w_height;
    let w_width = win_ref(wp).w_width;
    let w_view_width = win_ref(wp).w_view_width;
    let w_p_rl = win_ref(wp).w_p_rl() != 0;
    let p_stc = nvim_win_get_p_stc(wp);
    let col_off = nvim_win_col_off(wp);

    let below_window = grid == DEFAULT_GRID_HANDLE && row + winbar_height >= w_height;
    ON_STATUS_LINE = below_window && row + winbar_height - w_height + 1 == 1;
    ON_SEP_LINE = grid == DEFAULT_GRID_HANDLE && col >= w_width && col - w_width + 1 == 1;
    ON_WINBAR = row < 0 && row + winbar_height >= 0;
    ON_STATUSCOL = !below_window
        && !ON_STATUS_LINE
        && !ON_SEP_LINE
        && !ON_WINBAR
        && !p_stc.is_null()
        && *p_stc != 0
        && (if w_p_rl {
            col >= w_view_width - col_off
        } else {
            col < col_off
        });

    // The rightmost character of the status line might be a vertical
    // separator character if there is no connecting window to the right.
    if ON_STATUS_LINE && ON_SEP_LINE {
        if stl_connected(wp) != 0 {
            ON_SEP_LINE = false;
        } else {
            ON_STATUS_LINE = false;
        }
    }

    if keep_focus {
        row = mouse_row;
        col = mouse_col;
        grid = mouse_grid;
    }

    let curwin = nvim_get_curwin();
    let old_curwin = curwin;
    let old_cursor = *nvim_win_get_cursor_ptr(curwin);

    // `do_foldclick` tracks whether we should jump directly to the foldclick
    // section without the normal cursor movement logic.
    let do_foldclick: bool;

    if !keep_focus {
        if ON_WINBAR {
            return IN_OTHER_WIN | MOUSE_WINBAR;
        }

        if ON_STATUSCOL {
            do_foldclick = true;
        } else {
            fdc = rs_win_fdccol_count(wp);
            rs_set_dragwin(std::ptr::null_mut());

            if below_window {
                // In (or below) status line
                STATUS_LINE_OFFSET = row + winbar_height - w_height + 1;
                rs_set_dragwin(wp);
            } else {
                STATUS_LINE_OFFSET = 0;
            }

            if grid == DEFAULT_GRID_HANDLE && col >= w_width {
                // In separator line
                SEP_LINE_OFFSET = col - w_width + 1;
                rs_set_dragwin(wp);
            } else {
                SEP_LINE_OFFSET = 0;
            }

            // The rightmost character of the status line might be a vertical
            // separator character if there is no connecting window to the right.
            if STATUS_LINE_OFFSET != 0 && SEP_LINE_OFFSET != 0 {
                if stl_connected(wp) != 0 {
                    SEP_LINE_OFFSET = 0;
                } else {
                    STATUS_LINE_OFFSET = 0;
                }
            }

            // Before jumping to another buffer, or moving the cursor for a left
            // click, stop Visual mode.
            if VIsual_active
                && (win_ref(wp).w_buffer != win_ref(curwin).w_buffer
                    || (STATUS_LINE_OFFSET == 0
                        && SEP_LINE_OFFSET == 0
                        && (if w_p_rl {
                            col < w_view_width - fdc
                        } else {
                            let cmdwin_win = nvim_get_cmdwin_win();
                            col >= fdc + i32::from(wp == cmdwin_win)
                        })
                        && (flags & MOUSE_MAY_STOP_VIS) != 0))
            {
                end_visual_mode();
                redraw_curbuf_later(UPD_INVERTED);
            }

            if nvim_get_cmdwin_type() != 0 && wp != nvim_get_cmdwin_win() {
                // A click outside the command-line window: Use modeless
                // selection if possible. Allow dragging the status lines.
                SEP_LINE_OFFSET = 0;
                row = 0;
                col += win_ref(wp).w_wincol;
                // wp = cmdwin_win (win_enter will handle this)
            }

            // Only change window focus when not clicking on or dragging the
            // status line. Do change focus when releasing the mouse button
            // (MOUSE_FOCUS was set above if we dragged first).
            if !rs_is_dragging() || (flags & MOUSE_RELEASED) != 0 {
                win_enter(wp, true); // can make wp invalid!
            }
            let curwin = nvim_get_curwin();
            if curwin != old_curwin {
                rs_set_mouse_topline(curwin);
            }
            if STATUS_LINE_OFFSET != 0 {
                // In (or below) status line
                // Don't use start_arrow() if we're in the same window
                if curwin == old_curwin {
                    return IN_STATUS_LINE;
                }
                return IN_STATUS_LINE | CURSOR_MOVED;
            }
            if SEP_LINE_OFFSET != 0 {
                // In (or below) status line
                // Don't use start_arrow() if we're in the same window
                if curwin == old_curwin {
                    return IN_SEP_LINE;
                }
                return IN_SEP_LINE | CURSOR_MOVED;
            }

            let curwin = nvim_get_curwin();
            let topline = win_ref(curwin).w_topline;
            win_mut(curwin).w_cursor.lnum = topline;
            do_foldclick = false;
        }
    } else if STATUS_LINE_OFFSET != 0 {
        let dw = rs_get_dragwin();
        if which_button == MOUSE_LEFT && !dw.is_null() {
            // Drag the status line
            let count = row - win_ref(dw).w_winrow - win_ref(dw).w_height + 1 - STATUS_LINE_OFFSET;
            rs_win_drag_status_line(dw, count);
            DID_DRAG |= count;
        }
        return IN_STATUS_LINE; // Cursor didn't move
    } else if SEP_LINE_OFFSET != 0 && which_button == MOUSE_LEFT {
        let dw = rs_get_dragwin();
        if !dw.is_null() {
            // Drag the separator column
            let count = col - win_ref(dw).w_wincol - win_ref(dw).w_width + 1 - SEP_LINE_OFFSET;
            rs_win_drag_vsep_line(dw, count);
            DID_DRAG |= count;
        }
        return IN_SEP_LINE; // Cursor didn't move
    } else if ON_STATUS_LINE && which_button == MOUSE_RIGHT {
        return IN_STATUS_LINE;
    } else if ON_WINBAR && which_button == MOUSE_RIGHT {
        // After a click on the window bar don't start Visual mode.
        return IN_OTHER_WIN | MOUSE_WINBAR;
    } else if ON_STATUSCOL && which_button == MOUSE_RIGHT {
        // After a click on the status column don't start Visual mode.
        return IN_OTHER_WIN | MOUSE_STATUSCOL;
    } else {
        if (flags & MOUSE_MAY_STOP_VIS) != 0 {
            end_visual_mode();
            redraw_curbuf_later(UPD_INVERTED);
        }

        let curwin = nvim_get_curwin();
        let grid_alloc = nvim_win_get_w_grid_alloc(curwin);
        let w_grid = nvim_win_get_grid(curwin);

        if grid == 0 {
            row -= nvim_screengrid_get_comp_row(grid_alloc) + nvim_gridview_get_row_offset(w_grid);
            col -= nvim_screengrid_get_comp_col(grid_alloc) + nvim_gridview_get_col_offset(w_grid);
        } else if grid != DEFAULT_GRID_HANDLE {
            row -= nvim_gridview_get_row_offset(w_grid);
            col -= nvim_gridview_get_col_offset(w_grid);
        }

        let view_height = win_ref(curwin).w_view_height;
        let topline = win_ref(curwin).w_topline;
        let topfill = win_ref(curwin).w_topfill;

        if row < 0 {
            let mut count: c_int = 0;
            let mut first = true;
            let mut tline = topline;
            let mut tfill = topfill;
            while tline > 1 {
                if tfill < win_get_fill(curwin, tline) {
                    count += 1;
                } else {
                    count += plines_win(curwin, tline - 1, true);
                }
                if !first && count > -row {
                    break;
                }
                first = false;
                // hasFolding may update tline
                let mut tline_out = tline;
                nvim_hasFolding(
                    curwin,
                    tline,
                    std::ptr::addr_of_mut!(tline_out),
                    std::ptr::null_mut(),
                );
                tline = tline_out;
                if tfill < win_get_fill(curwin, tline) {
                    tfill += 1;
                } else {
                    tline -= 1;
                    tfill = 0;
                }
                win_mut(curwin).w_topline = tline;
                win_mut(curwin).w_topfill = tfill;
            }
            // Final sync: write back topline/topfill in case loop didn't run
            win_mut(curwin).w_topline = tline;
            win_mut(curwin).w_topfill = tfill;
            check_topfill(curwin, 0);
            nvim_win_clear_valid_bits(
                curwin,
                VALID_WROW | VALID_CROW | VALID_BOTLINE | VALID_BOTLINE_AP,
            );
            redraw_later(curwin, UPD_VALID);
            row = 0;
        } else if row >= view_height {
            let mut count: c_int = 0;
            let mut first = true;
            let line_count = bref_raw(nvim_get_curbuf()).ml_line_count;
            let mut tline = topline;
            let mut tfill = topfill;
            while tline < line_count {
                if tfill > 0 {
                    count += 1;
                } else {
                    count += plines_win(curwin, tline, true);
                }
                if !first && count > row - view_height + 1 {
                    break;
                }
                first = false;
                if tfill > 0 {
                    tfill -= 1;
                } else {
                    let mut tline_end = tline;
                    nvim_hasFolding(
                        curwin,
                        tline,
                        std::ptr::null_mut(),
                        std::ptr::addr_of_mut!(tline_end),
                    );
                    tline = tline_end;
                    if tline == line_count {
                        break;
                    }
                    tline += 1;
                    tfill = win_get_fill(curwin, tline);
                }
            }
            win_mut(curwin).w_topline = tline;
            win_mut(curwin).w_topfill = tfill;
            check_topfill(curwin, 0);
            redraw_later(curwin, UPD_VALID);
            nvim_win_clear_valid_bits(
                curwin,
                VALID_WROW | VALID_CROW | VALID_BOTLINE | VALID_BOTLINE_AP,
            );
            row = view_height - 1;
        } else if row == 0 {
            // When dragging the mouse, while the text has been scrolled up as
            // far as it goes, moving the mouse in the top line should scroll
            // the text down (done later when recomputing w_topline).
            let cursor_lnum = win_ref(curwin).w_cursor.lnum;
            let line_count = bref_raw(nvim_get_curbuf()).ml_line_count;
            if rs_get_mouse_dragging() > 0 && cursor_lnum == line_count && cursor_lnum == topline {
                nvim_win_clear_valid_bits(curwin, VALID_TOPLINE);
            }
        }

        do_foldclick = false;
    }

    // === foldclick section ===
    let curwin = nvim_get_curwin();
    let mut col_from_screen: colnr_T = -1;
    let mut mouse_fold_flags: c_int = 0;
    rs_mouse_check_grid(
        std::ptr::addr_of_mut!(col_from_screen),
        std::ptr::addr_of_mut!(mouse_fold_flags),
    );

    let cursor_ptr = nvim_win_get_cursor_ptr(curwin);
    if rs_mouse_comp_pos(
        curwin,
        std::ptr::addr_of_mut!(row),
        std::ptr::addr_of_mut!(col),
        std::ptr::addr_of_mut!((*cursor_ptr).lnum),
    ) {
        nvim_set_mouse_past_bottom(true);
    }

    if (flags & MOUSE_MAY_VIS) != 0 && !VIsual_active {
        // VIsual = old_cursor
        nvim_set_VIsual_pos(
            old_cursor.lnum,
            old_cursor.col as c_int,
            old_cursor.coladd as c_int,
        );
        VIsual_active = true;
        VIsual_reselect = true;
        rs_may_start_select(c_int::from(b'o'));
        setmouse_global();
        if p_smd != 0 && msg_silent == 0 {
            nvim_set_redraw_cmdline(true); // show visual mode later
        }
    }

    if col_from_screen >= 0 {
        col = col_from_screen;
    }

    win_mut(curwin).w_curswant = col;
    win_mut(curwin).w_set_curswant = 0; // May still have been true
    if coladvance(curwin, col) == FAIL {
        // Mouse click beyond end of line
        if !inclusive.is_null() {
            *inclusive = true;
        }
        nvim_set_mouse_past_eol(true);
    } else if !inclusive.is_null() {
        *inclusive = false;
    }

    let _ = do_foldclick; // used to control flow only; ON_STATUSCOL captures the same condition
    let mut count = if ON_STATUSCOL {
        IN_OTHER_WIN | MOUSE_STATUSCOL
    } else {
        IN_BUFFER
    };

    let new_cursor = *nvim_win_get_cursor_ptr(curwin);
    if curwin != old_curwin
        || new_cursor.lnum != old_cursor.lnum
        || new_cursor.col != old_cursor.col
    {
        count |= CURSOR_MOVED; // Cursor has moved
    }

    count |= mouse_fold_flags;
    count
}

// =============================================================================
// Phase 4 — f_getmousepos
// =============================================================================

extern "C" {
    /// Initialize `rettv` as a dict and return its `dict_T*`.
    fn tv_dict_alloc_ret(rettv: *mut std::ffi::c_void);

    /// Get dict pointer from a typval (reads `tv->vval.v_dict`).
    #[link_name = "nvim_tv_get_dict"]
    fn nvim_tv_get_dict_ptr(tv: *const std::ffi::c_void) -> *const std::ffi::c_void;

    /// Add a number entry to a dict.
    fn tv_dict_add_nr(
        d: *mut std::ffi::c_void,
        key: *const c_char,
        key_len: usize,
        nr: i64,
    ) -> c_int;

}

/// `getmousepos()` function.
///
/// Returns a dict with mouse position fields: `screenrow`, `screencol`,
/// `winid`, `winrow`, `wincol`, `line`, `column`, `coladd`.
///
/// # Safety
/// `argvars`, `rettv`, and `fptr` must be valid typval pointers.
#[export_name = "f_getmousepos"]
pub unsafe extern "C" fn rs_f_getmousepos(
    _argvars: *const std::ffi::c_void,
    rettv: *mut std::ffi::c_void,
    _fptr: *mut std::ffi::c_void,
) {
    let mr = mouse_row;
    let mc = mouse_col;
    let mut grid = mouse_grid;
    let mut row = mr;
    let mut col = mc;
    let mut winid: i64 = 0;
    let mut winrow: i64 = 0;
    let mut wincol: i64 = 0;
    let mut lnum: linenr_T = 0;
    let mut column: i64 = 0;
    let mut coladd: colnr_T = 0;

    tv_dict_alloc_ret(rettv);
    let dict = nvim_tv_get_dict_ptr(rettv).cast_mut();

    tv_dict_add_nr(dict, c"screenrow".as_ptr(), 9, i64::from(mouse_row) + 1);
    tv_dict_add_nr(dict, c"screencol".as_ptr(), 9, i64::from(mouse_col) + 1);

    let wp = rs_mouse_find_win_inner(
        std::ptr::addr_of_mut!(grid),
        std::ptr::addr_of_mut!(row),
        std::ptr::addr_of_mut!(col),
    );
    if !wp.is_null() {
        let height = win_ref(wp).w_height + win_ref(wp).w_hsep_height + win_ref(wp).w_status_height;
        // The height is adjusted by 1 when there is a bottom border.
        if row < height + win_ref(wp).w_border_adj[2] {
            winid = i64::from(win_ref(wp).handle);
            winrow = i64::from(row) + 1 + i64::from(win_ref(wp).w_winrow_off);
            wincol = i64::from(col) + 1 + i64::from(win_ref(wp).w_wincol_off);
            if row >= 0 && row < win_ref(wp).w_height && col >= 0 && col < win_ref(wp).w_width {
                rs_mouse_comp_pos(
                    wp,
                    std::ptr::addr_of_mut!(row),
                    std::ptr::addr_of_mut!(col),
                    std::ptr::addr_of_mut!(lnum),
                );
                col = rs_vcol2col(wp, lnum, col, std::ptr::addr_of_mut!(coladd));
                column = i64::from(col) + 1;
            }
        }
    }

    tv_dict_add_nr(dict, c"winid".as_ptr(), 5, winid);
    tv_dict_add_nr(dict, c"winrow".as_ptr(), 6, winrow);
    tv_dict_add_nr(dict, c"wincol".as_ptr(), 6, wincol);
    tv_dict_add_nr(dict, c"line".as_ptr(), 4, i64::from(lnum));
    tv_dict_add_nr(dict, c"column".as_ptr(), 6, column);
    tv_dict_add_nr(dict, c"coladd".as_ptr(), 6, i64::from(coladd));
}

// =============================================================================
// Phase 9 — full rs_do_mouse_impl (formerly nvim_do_mouse_impl in C)
// =============================================================================

// ---------------------------------------------------------------------------
// Extern declarations for rs_do_mouse_impl
// ---------------------------------------------------------------------------
extern "C" {
    // Input helpers
    fn get_mouse_button(code: c_int, is_click: *mut bool, is_drag: *mut bool) -> c_int;
    fn nvim_vpeekc() -> c_int;
    fn nvim_safe_vgetc() -> c_int;
    fn nvim_vungetc(c: c_int);
    fn nvim_stuffcharReadbuff(c: c_int);
    fn nvim_stuffnumReadbuff(n: c_int);
    fn nvim_stuffReadbuff(s: *const c_char);
    // nvim_AppendCharToRedobuff: now in the insert/put block below

    // Keyboard globals
    static KeyStuffed: c_int;

    // Mouse global setters
    fn nvim_set_mouse_grid(val: c_int);
    fn nvim_set_mouse_row(val: c_int);
    fn nvim_set_mouse_col(val: c_int);
    static mut mouse_dragging: c_int;

    // Mode/state accessors
    fn nvim_get_state_mouse() -> c_int;
    fn nvim_get_mod_mask_mouse() -> c_int;
    fn nvim_get_VIsual_active_mouse() -> bool;
    fn nvim_get_VIsual_mode_mouse() -> c_int;
    fn nvim_set_VIsual_mode_mouse(val: c_int);
    fn nvim_get_VIsual_select_mouse() -> bool;
    fn nvim_get_mode_displayed_mouse() -> bool;
    fn nvim_get_p_smd_mouse() -> c_int;
    fn nvim_get_msg_silent_mouse() -> c_int;
    fn nvim_get_p_sel() -> *const c_char;
    fn nvim_get_p_mousem() -> *const c_char;
    fn nvim_get_Columns_mouse() -> c_int;
    fn nvim_get_firstwin_winrow_mouse() -> c_int;
    fn nvim_get_cmdwin_type_mouse() -> c_int;
    fn nvim_tab_page_click_defs_valid() -> bool;
    fn nvim_get_tab_page_click_defs_ptr() -> StlClickDefinitionHandle;
    fn nvim_get_restart_edit_mouse() -> c_int;

    // Insert/put/register operations
    fn nvim_eval_has_provider(feat: *const c_char) -> bool;
    fn nvim_set_where_paste_started_to_cursor();
    fn nvim_get_mouse_past_bottom() -> bool;
    fn nvim_get_mouse_past_eol() -> bool;
    fn nvim_AppendCharToRedobuff(c: c_int);
    fn insert_reg(regname: c_int, reg: *mut YankregHandle, literally_arg: bool) -> c_int;
    fn do_put(regname: c_int, reg: *mut YankregHandle, dir: c_int, count: c_int, flags: c_int);
    fn yank_register_mline(regname: c_int, reg: *mut *mut YankregHandle) -> bool;

    // Click defs accessors
    fn nvim_win_get_status_click_defs(wp: WinHandle) -> StlClickDefinitionHandle;
    fn nvim_win_get_winbar_click_defs(wp: WinHandle) -> StlClickDefinitionHandle;
    fn nvim_win_get_statuscol_click_defs(wp: WinHandle) -> StlClickDefinitionHandle;
    fn nvim_win_get_status_click_defs_size(wp: WinHandle) -> c_int;
    fn nvim_win_get_statuscol_click_defs_size(wp: WinHandle) -> c_int;

    // Navigation/tag/quickfix
    fn nvim_curwin_is_qf() -> c_int;
    fn nvim_curwin_is_ll() -> c_int;
    fn nvim_do_cmdline_cmd_mouse(cmd: *const c_char) -> c_int;
    fn nvim_curbuf_is_help_mouse() -> c_int;

    // Position operations
    fn gchar_pos(pos: *const PosT) -> c_int;
    fn inc(pos: *mut PosT) -> c_int;
    fn nvim_findmatch_nul(
        oap: OpargHandle,
        lnum: *mut linenr_T,
        col: *mut c_int,
        coladd: *mut c_int,
        motion_type_out: *mut c_int,
    ) -> bool;
    fn nvim_ml_get_line(lnum: linenr_T) -> *const c_char;
    fn nvim_ascii_iswhite_mouse(c: c_int) -> c_int;
    fn nvim_vim_iswordc_mouse(c: c_int) -> c_int;
    fn nvim_get_cursor_pos_ptr_mouse() -> *const c_char;
    fn nvim_utfc_ptr2len_at_cursor() -> c_int;

    // Visual/cursor operations
    fn nvim_curwin_coladvance(col: c_int) -> c_int;
    fn nvim_set_curswant_flag();
    fn nvim_set_VIsual_to_cursor();
    fn nvim_get_VIsual_pos(lnum: *mut linenr_T, col: *mut c_int, coladd: *mut c_int);
    fn nvim_set_VIsual_lnum_col_coladd(lnum: linenr_T, col: c_int, coladd: c_int);
    fn nvim_set_VIsual_col_only(col: c_int);
    fn nvim_inc_VIsual_col();
    fn nvim_inc_cursor_col();
    fn nvim_set_cursor_pos(lnum: linenr_T, col: c_int, coladd: c_int);
    fn nvim_set_cursor_col(col: c_int);

    // Tabpage ops
    fn nvim_goto_tabpage(n: c_int);
    fn nvim_tabpage_new();

    // Scroll
    fn nvim_scroll_redraw(up: bool, count: c_int);

    // Cross-crate Rust functions
    fn rs_prep_redo(
        regname: c_int,
        num: c_int,
        cmd1: c_int,
        cmd2: c_int,
        cmd3: c_int,
        cmd4: c_int,
        cmd5: c_int,
    );
    fn rs_get_scrolloff_value(wp: WinHandle) -> c_int;
    fn rs_setFoldRepeat(lnum: linenr_T, count: c_int, do_open: bool);
    fn rs_clearop(oap: OpargHandle);
    fn rs_clearopbeep(oap: OpargHandle);

    #[link_name = "nvim_set_redraw_cmdline"]
    fn nvim_set_redraw_cmdline_mouse(val: bool);
}

// ---------------------------------------------------------------------------
// Constants for rs_do_mouse_impl
// ---------------------------------------------------------------------------

/// `K_MOUSEMOVE` = `TERMCAP2KEY(KS_EXTRA=253`, `KE_MOUSEMOVE=100`)
const K_MOUSEMOVE: c_int = -(253 + (100i32 << 8));

/// KEY2TERMCAP1(x): extract the termcap1 byte from a negative key code.
#[inline]
#[allow(clippy::cast_sign_loss, clippy::cast_possible_wrap)]
fn key2termcap1(x: c_int) -> c_int {
    ((-x as u32) >> 8) as c_int & 0xff
}

/// NUL key code value (used in `rs_prep_redo` calls).
const NUL_KEY: c_int = 0;

// Mode bits
const MODE_INSERT_IMPL: c_int = 0x10;
const REPLACE_FLAG: c_int = 0x100;

// Direction constants
const FORWARD_IMPL: c_int = 1;
const BACKWARD_IMPL: c_int = -1;

// do_put flags (from register_defs.h)
const PUT_FIXINDENT: c_int = 1;
const PUT_CURSEND: c_int = 2;

// Ctrl key codes
const CTRL_O_CODE: c_int = 15;
const CTRL_P_CODE: c_int = 16;
const CTRL_R_CODE: c_int = 18;
const CTRL_T_CODE: c_int = 20;
const CTRL_G_CODE: c_int = 7;
const CTRL_RSB_CODE: c_int = 29;

// K_MIDDLEMOUSE = TERMCAP2KEY(KS_EXTRA=253, KE_MIDDLEMOUSE=47)
const K_MIDDLEMOUSE_IMPL: c_int = -(253 + (47i32 << 8));

// kStlClickType enum values (match C)
const STL_CLICK_DISABLED: c_int = 0;
const STL_CLICK_TAB_SWITCH: c_int = 1;
const STL_CLICK_TAB_CLOSE: c_int = 2;
const STL_CLICK_FUNC_RUN: c_int = 3;

// op_type OP_NOP = 0, motion kMTCharWise = 0, kMTLineWise = 1
const OP_NOP_IMPL: c_int = 0;
const MT_CHAR_WISE: c_int = 0;
const MT_LINE_WISE: c_int = 1;

// Static state for in_tab_line and orig_cursor (from C's static locals in nvim_do_mouse_impl)
static mut IN_TAB_LINE: bool = false;
static mut ORIG_CURSOR: PosT = PosT {
    lnum: 0,
    col: 0,
    coladd: 0,
};

// ---------------------------------------------------------------------------
// Helpers for oparg_T field access (opaque pointer)
// ---------------------------------------------------------------------------

/// Get `op_type` from oparg handle (first field, `c_int` at offset 0).
#[inline]
unsafe fn oap_op_type(oap: OpargHandle) -> c_int {
    if oap.is_null() {
        OP_NOP_IMPL
    } else {
        *(oap.cast::<c_int>())
    }
}

/// Get `regname` from oparg handle (second field after `op_type`).
#[inline]
unsafe fn oap_regname(oap: OpargHandle) -> c_int {
    use nvim_normal::types::OpargT;
    if oap.is_null() {
        0
    } else {
        (*oap.cast::<OpargT>()).regname
    }
}

/// Set `motion_type` field of oparg handle.
#[inline]
unsafe fn oap_set_motion_type(oap: OpargHandle, val: c_int) {
    use nvim_normal::types::OpargT;
    if !oap.is_null() {
        (*oap.cast::<OpargT>()).motion_type = val;
    }
}

/// Get `VIsual` position as a `PosT`.
#[inline]
unsafe fn get_visual_pos() -> PosT {
    let mut lnum: linenr_T = 0;
    let mut col: c_int = 0;
    let mut coladd: c_int = 0;
    nvim_get_VIsual_pos(&raw mut lnum, &raw mut col, &raw mut coladd);
    PosT { lnum, col, coladd }
}

/// Check if mousemodel is popup (first char of `p_mousem` is 'p').
#[inline]
unsafe fn mouse_model_popup_impl() -> bool {
    let p = nvim_get_p_mousem();
    !p.is_null() && *p.cast::<u8>() == b'p'
}

/// Check equality of two `PosT` values.
#[inline]
fn pos_equal(a: PosT, b: PosT) -> bool {
    a.lnum == b.lnum && a.col == b.col && a.coladd == b.coladd
}

// ---------------------------------------------------------------------------
// The main migration: rs_do_mouse_impl
// ---------------------------------------------------------------------------

/// Full implementation of `do_mouse` logic (was `nvim_do_mouse_impl` in C).
///
/// # Safety
/// `oap` may be null. Otherwise must be valid `oparg_T*`.
#[allow(clippy::too_many_lines)]
#[allow(clippy::cognitive_complexity)]
unsafe fn rs_do_mouse_impl(
    oap: OpargHandle,
    c: c_int,
    mut dir: c_int,
    count: c_int,
    fixindent: bool,
) -> bool {
    // ------------------------------------------------------------------
    // Drag coalescing loop
    // ------------------------------------------------------------------
    let which_button: c_int;
    let is_click: bool;
    let is_drag: bool;

    {
        let mut wb: c_int;
        let mut ic = false;
        let mut id = false;
        loop {
            wb = get_mouse_button(key2termcap1(c), &raw mut ic, &raw mut id);
            if id && KeyStuffed == 0 && nvim_vpeekc() != 0 {
                let save_grid = mouse_grid;
                let save_row = mouse_row;
                let save_col = mouse_col;
                let nc = nvim_safe_vgetc();
                if c == nc {
                    continue;
                }
                nvim_vungetc(nc);
                nvim_set_mouse_grid(save_grid);
                nvim_set_mouse_row(save_row);
                nvim_set_mouse_col(save_col);
            }
            break;
        }
        which_button = wb;
        is_click = ic;
        is_drag = id;
    }

    if c == K_MOUSEMOVE {
        return false;
    }

    // ------------------------------------------------------------------
    // got_click tracking
    // ------------------------------------------------------------------
    if is_click {
        GOT_CLICK = true;
    } else {
        if !GOT_CLICK {
            return false;
        }
        if !is_drag {
            GOT_CLICK = false;
            if IN_TAB_LINE {
                IN_TAB_LINE = false;
                return false;
            }
        }
    }

    let mm = nvim_get_mod_mask_mouse();

    // ------------------------------------------------------------------
    // CTRL + right mouse = CTRL-T
    // ------------------------------------------------------------------
    if is_click && (mm & MOD_MASK_CTRL) != 0 && which_button == MOUSE_RIGHT {
        if (nvim_get_state_mouse() & MODE_INSERT_IMPL) != 0 {
            nvim_stuffcharReadbuff(CTRL_O_CODE);
        }
        if count > 1 {
            nvim_stuffnumReadbuff(count);
        }
        nvim_stuffcharReadbuff(CTRL_T_CODE);
        GOT_CLICK = false;
        return false;
    }

    // CTRL only works with left mouse button
    if (mm & MOD_MASK_CTRL) != 0 && which_button != MOUSE_LEFT {
        return false;
    }

    // ------------------------------------------------------------------
    // Modifier filtering
    // ------------------------------------------------------------------
    if (mm & (MOD_MASK_SHIFT | MOD_MASK_CTRL | MOD_MASK_ALT | MOD_MASK_META)) != 0
        && (!is_click || (mm & MOD_MASK_MULTI_CLICK) != 0 || which_button == MOUSE_MIDDLE)
        && !((mm & (MOD_MASK_SHIFT | MOD_MASK_ALT)) != 0
            && mouse_model_popup_impl()
            && which_button == MOUSE_LEFT)
        && !((mm & MOD_MASK_ALT) != 0 && !mouse_model_popup_impl() && which_button == MOUSE_RIGHT)
    {
        return false;
    }

    // Ignore drag/release with middle button
    if !is_click && which_button == MOUSE_MIDDLE {
        return false;
    }

    // ------------------------------------------------------------------
    // Middle mouse button (before jump_to_mouse)
    // ------------------------------------------------------------------
    let mut regname = oap_regname(oap);
    if which_button == MOUSE_MIDDLE {
        let state = nvim_get_state_mouse();
        if state == MODE_NORMAL {
            if !oap.is_null() && oap_op_type(oap) != OP_NOP_IMPL {
                rs_clearopbeep(oap);
                return false;
            }
            if nvim_get_VIsual_active_mouse() {
                if nvim_get_VIsual_select_mouse() {
                    nvim_stuffcharReadbuff(CTRL_G_CODE);
                    nvim_stuffReadbuff(c"\"+p".as_ptr());
                } else {
                    nvim_stuffcharReadbuff(c_int::from(b'y'));
                    nvim_stuffcharReadbuff(K_MIDDLEMOUSE_IMPL);
                }
                return false;
            }
            // The rest is below jump_to_mouse
        } else if (state & MODE_INSERT_IMPL) == 0 {
            return false;
        }
        if (state & MODE_INSERT_IMPL) != 0 {
            // Inline of C's nvim_mouse_middle_insert_mode.
            if regname == c_int::from(b'.') {
                insert_reg(regname, std::ptr::null_mut(), true);
            } else {
                if regname == 0 && nvim_eval_has_provider(c"clipboard".as_ptr()) {
                    regname = c_int::from(b'*');
                }
                let mut reg: *mut YankregHandle = std::ptr::null_mut();
                if (state & REPLACE_FLAG) != 0 && !yank_register_mline(regname, &raw mut reg) {
                    insert_reg(regname, reg, true);
                } else {
                    do_put(
                        regname,
                        reg,
                        BACKWARD_IMPL,
                        1,
                        (if fixindent { PUT_FIXINDENT } else { 0 }) | PUT_CURSEND,
                    );
                    nvim_AppendCharToRedobuff(CTRL_R_CODE);
                    nvim_AppendCharToRedobuff(if fixindent { CTRL_P_CODE } else { CTRL_O_CODE });
                    nvim_AppendCharToRedobuff(if regname == 0 {
                        c_int::from(b'"')
                    } else {
                        regname
                    });
                }
            }
            return false;
        }
    }

    // ------------------------------------------------------------------
    // Tab line handling
    // ------------------------------------------------------------------
    let mut jump_flags: c_int = if is_click {
        0
    } else {
        MOUSE_FOCUS | MOUSE_DID_MOVE
    };
    let old_curwin = nvim_get_curwin();

    if nvim_tab_page_click_defs_valid() {
        let fwwinrow = nvim_get_firstwin_winrow_mouse();
        if mouse_grid <= 1 && mouse_row == 0 && fwwinrow > 0 {
            if is_drag {
                if IN_TAB_LINE {
                    rs_move_tab_to_mouse();
                }
                return false;
            }
            if is_click && nvim_get_cmdwin_type_mouse() == 0 && mouse_col < nvim_get_Columns_mouse()
            {
                let tab_defs = nvim_get_tab_page_click_defs_ptr();
                #[allow(clippy::cast_sign_loss)]
                let tab_def = if tab_defs.is_null() || mouse_col < 0 {
                    None
                } else {
                    Some(
                        &*tab_defs
                            .cast::<StlClickDefinition>()
                            .add(mouse_col as usize),
                    )
                };
                let tabnr = tab_def.map_or(0, |d| d.tabnr);
                IN_TAB_LINE = true;
                let click_type = tab_def.map_or(0, |d| d.click_type);
                if click_type == STL_CLICK_TAB_SWITCH {
                    if which_button == MOUSE_MIDDLE {
                        rs_mouse_tab_close(tabnr);
                    } else if (mm & MOD_MASK_MULTI_CLICK) == MOD_MASK_2CLICK {
                        end_visual_mode();
                        nvim_tabpage_new();
                        tabpage_move(if tabnr == 0 { 9999 } else { tabnr - 1 });
                    } else {
                        nvim_goto_tabpage(tabnr);
                        if nvim_get_curwin() != old_curwin {
                            end_visual_mode();
                        }
                    }
                } else if click_type == STL_CLICK_TAB_CLOSE {
                    rs_mouse_tab_close(tabnr);
                } else if click_type == STL_CLICK_FUNC_RUN {
                    rs_call_click_def_func(
                        nvim_get_tab_page_click_defs_ptr(),
                        mouse_col,
                        which_button,
                    );
                }
                // kStlClickDisabled: do nothing
            }
            return true;
        } else if is_drag && IN_TAB_LINE {
            rs_move_tab_to_mouse();
            return false;
        }
    }

    // ------------------------------------------------------------------
    // Popup model translation
    // ------------------------------------------------------------------
    let mut m_pos_flag: c_int = 0;
    let mut m_pos = PosT::default();
    let mut which_button = which_button; // shadow as mutable

    if mouse_model_popup_impl() {
        m_pos_flag = rs_get_fpos_of_mouse(&raw mut m_pos);
        let not_in_ui = (m_pos_flag & (IN_STATUS_LINE | MOUSE_WINBAR | MOUSE_STATUSCOL)) == 0;
        if not_in_ui && which_button == MOUSE_RIGHT && (mm & (MOD_MASK_SHIFT | MOD_MASK_CTRL)) == 0
        {
            if !is_click {
                return false;
            }
            return (rs_do_popup(which_button, m_pos_flag, m_pos) & CURSOR_MOVED) != 0;
        }
        if not_in_ui && which_button == MOUSE_LEFT && (mm & (MOD_MASK_SHIFT | MOD_MASK_ALT)) != 0 {
            which_button = MOUSE_RIGHT;
            // Note: the C code also does mod_mask &= ~MOD_MASK_SHIFT here
            // mod_mask is a C global; we can't mutate it, but only the
            // jump_flags computation below uses it and we re-read with nvim_get_mod_mask_mouse()
        }
    }

    // ------------------------------------------------------------------
    // Visual mode flag setup
    // ------------------------------------------------------------------
    let mm = nvim_get_mod_mask_mouse();
    let state = nvim_get_state_mouse();
    let mut start_visual = PosT::default();
    let mut end_visual = PosT::default();

    if (state & (MODE_NORMAL | MODE_INSERT_IMPL)) != 0
        && (mm & (MOD_MASK_SHIFT | MOD_MASK_CTRL)) == 0
    {
        if which_button == MOUSE_LEFT {
            if is_click {
                if nvim_get_VIsual_active_mouse() {
                    jump_flags |= MOUSE_MAY_STOP_VIS;
                }
            } else {
                jump_flags |= MOUSE_MAY_VIS;
            }
        } else if which_button == MOUSE_RIGHT {
            if is_click && nvim_get_VIsual_active_mouse() {
                let curwin = nvim_get_curwin();
                let cursor = *nvim_win_get_cursor_ptr(curwin);
                let visual = get_visual_pos();
                if pos_lt(cursor, visual) {
                    start_visual = cursor;
                    end_visual = visual;
                } else {
                    start_visual = visual;
                    end_visual = cursor;
                }
            }
            jump_flags |= MOUSE_FOCUS;
            jump_flags |= MOUSE_MAY_VIS;
        }
    }

    // If operator pending, ignore drags/releases
    if !is_drag && !oap.is_null() && oap_op_type(oap) != OP_NOP_IMPL {
        GOT_CLICK = false;
        oap_set_motion_type(oap, MT_CHAR_WISE);
    }

    // Releasing button: tell jump_to_mouse
    if !is_click && !is_drag {
        jump_flags |= MOUSE_RELEASED;
    }

    // ------------------------------------------------------------------
    // jump_to_mouse
    // ------------------------------------------------------------------
    let old_active = nvim_get_VIsual_active_mouse();
    let curwin = nvim_get_curwin();
    let save_cursor = *nvim_win_get_cursor_ptr(curwin);

    let inclusive_ptr: *mut bool = if oap.is_null() {
        std::ptr::null_mut()
    } else {
        use nvim_normal::types::OpargT;
        std::ptr::addr_of_mut!((*oap.cast::<OpargT>()).inclusive)
    };

    jump_flags = rs_jump_to_mouse(jump_flags, inclusive_ptr, which_button);

    let moved = (jump_flags & CURSOR_MOVED) != 0;
    let in_winbar = (jump_flags & MOUSE_WINBAR) != 0;
    let in_statuscol = (jump_flags & MOUSE_STATUSCOL) != 0;
    let in_status_line = (jump_flags & IN_STATUS_LINE) != 0;
    let in_global_statusline = in_status_line && nvim_global_stl_height() > 0;
    let in_sep_line = (jump_flags & IN_SEP_LINE) != 0;

    // ------------------------------------------------------------------
    // Status line / winbar / statuscol click dispatch
    // ------------------------------------------------------------------
    if (in_winbar || in_status_line || in_statuscol) && is_click {
        let mut click_grid = mouse_grid;
        let mut click_row = mouse_row;
        let mut click_col = mouse_col;
        let wp =
            rs_mouse_find_win_inner(&raw mut click_grid, &raw mut click_row, &raw mut click_col);
        if wp.is_null() {
            return false;
        }

        let click_defs_raw = if in_status_line {
            nvim_win_get_status_click_defs(wp)
        } else if in_winbar {
            nvim_win_get_winbar_click_defs(wp)
        } else {
            nvim_win_get_statuscol_click_defs(wp)
        };

        let (click_defs, click_col) = if in_global_statusline {
            let cw = nvim_get_curwin();
            (nvim_win_get_status_click_defs(cw), mouse_col)
        } else {
            (click_defs_raw, click_col)
        };

        let click_col = if in_statuscol && win_ref(wp).w_p_rl() != 0 {
            win_ref(wp).w_view_width - click_col - 1
        } else {
            click_col
        };

        // Bounds checks
        if in_statuscol && click_col >= nvim_win_get_statuscol_click_defs_size(wp) {
            return false;
        }
        if in_status_line {
            let check_wp = if in_global_statusline {
                nvim_get_curwin()
            } else {
                wp
            };
            if click_col >= nvim_win_get_status_click_defs_size(check_wp) {
                return false;
            }
        }

        if !click_defs.is_null() {
            #[allow(clippy::cast_sign_loss)]
            let click_type = (*click_defs
                .cast::<StlClickDefinition>()
                .add(click_col as usize))
            .click_type;
            if click_type == STL_CLICK_DISABLED {
                if in_statuscol
                    && mouse_model_popup_impl()
                    && which_button == MOUSE_RIGHT
                    && (mm & (MOD_MASK_SHIFT | MOD_MASK_CTRL)) == 0
                {
                    rs_do_popup(which_button, m_pos_flag, m_pos);
                }
            } else if click_type == STL_CLICK_FUNC_RUN {
                rs_call_click_def_func(click_defs, click_col, which_button);
            }
        }

        if !(in_statuscol && (jump_flags & (MOUSE_FOLD_CLOSE | MOUSE_FOLD_OPEN)) != 0) {
            return false;
        }
    } else if in_winbar || in_statuscol {
        return false;
    }

    // ------------------------------------------------------------------
    // Operator clear on window change
    // ------------------------------------------------------------------
    if nvim_get_curwin() != old_curwin && !oap.is_null() && oap_op_type(oap) != OP_NOP_IMPL {
        rs_clearop(oap);
    }

    // ------------------------------------------------------------------
    // Fold open/close
    // ------------------------------------------------------------------
    let mm = nvim_get_mod_mask_mouse();
    if mm == 0
        && !is_drag
        && (jump_flags & (MOUSE_FOLD_CLOSE | MOUSE_FOLD_OPEN)) != 0
        && which_button == MOUSE_LEFT
    {
        let curwin = nvim_get_curwin();
        let lnum = (*nvim_win_get_cursor_ptr(curwin)).lnum;
        if (jump_flags & MOUSE_FOLD_OPEN) != 0 {
            rs_setFoldRepeat(lnum, 1, true);
        } else {
            rs_setFoldRepeat(lnum, 1, false);
        }
        if nvim_get_curwin() == old_curwin {
            nvim_set_cursor_pos(save_cursor.lnum, save_cursor.col, save_cursor.coladd);
        }
    }

    // ------------------------------------------------------------------
    // Scroll dragging
    // ------------------------------------------------------------------
    if nvim_get_VIsual_active_mouse() && is_drag && rs_get_scrolloff_value(nvim_get_curwin()) > 0 {
        if mouse_row == 0 {
            mouse_dragging = 2;
        } else {
            mouse_dragging = 1;
        }
    }

    // Drag above window: scroll down
    if is_drag && mouse_row < 0 && !in_status_line {
        nvim_scroll_redraw(false, 1);
        nvim_set_mouse_row(0);
    }

    // ------------------------------------------------------------------
    // Visual extend (right-click in visual mode)
    // ------------------------------------------------------------------
    let old_mode = nvim_get_VIsual_mode_mouse();
    let state = nvim_get_state_mouse();

    if start_visual.lnum != 0 {
        let mm = nvim_get_mod_mask_mouse();
        if (mm & MOD_MASK_ALT) != 0 {
            nvim_set_VIsual_mode_mouse(VISUAL_BLOCK);
        }
        let vmode = nvim_get_VIsual_mode_mouse();
        let curwin = nvim_get_curwin();
        let cursor = *nvim_win_get_cursor_ptr(curwin);

        if vmode == VISUAL_BLOCK {
            let mut leftcol: colnr_T = 0;
            let mut rightcol: colnr_T = 0;
            // Inline of C's nvim_getvcols_mouse.
            rs_getvcols(
                curwin,
                start_visual,
                end_visual,
                &raw mut leftcol,
                &raw mut rightcol,
            );
            let curswant = win_ref(nvim_get_curwin()).w_curswant;
            let target_col = if curswant > i32::midpoint(leftcol, rightcol) {
                leftcol
            } else {
                rightcol
            };
            let target_lnum = if cursor.lnum >= i32::midpoint(start_visual.lnum, end_visual.lnum) {
                start_visual.lnum
            } else {
                end_visual.lnum
            };
            // Move VIsual to the right column
            let save_curs = cursor;
            nvim_set_cursor_pos(target_lnum, target_col, 0);
            nvim_curwin_coladvance(target_col);
            // VIsual = curwin->w_cursor (after coladvance)
            let curwin = nvim_get_curwin();
            let new_curs = *nvim_win_get_cursor_ptr(curwin);
            nvim_set_VIsual_lnum_col_coladd(new_curs.lnum, new_curs.col, new_curs.coladd);
            nvim_set_cursor_pos(save_curs.lnum, save_curs.col, save_curs.coladd);
        } else if pos_lt(cursor, start_visual) {
            nvim_set_VIsual_lnum_col_coladd(end_visual.lnum, end_visual.col, end_visual.coladd);
        } else if pos_lt(end_visual, cursor) {
            nvim_set_VIsual_lnum_col_coladd(
                start_visual.lnum,
                start_visual.col,
                start_visual.coladd,
            );
        } else if end_visual.lnum == start_visual.lnum {
            if (cursor.col - start_visual.col) > (end_visual.col - cursor.col) {
                nvim_set_VIsual_lnum_col_coladd(
                    start_visual.lnum,
                    start_visual.col,
                    start_visual.coladd,
                );
            } else {
                nvim_set_VIsual_lnum_col_coladd(end_visual.lnum, end_visual.col, end_visual.coladd);
            }
        } else {
            let diff = (cursor.lnum - start_visual.lnum) - (end_visual.lnum - cursor.lnum);
            // diff > 0: cursor is closest to end, anchor at start
            // diff < 0: cursor is closest to start, anchor at end
            // diff == 0 (middle line): compare column
            if diff > 0
                || (diff == 0 && cursor.col >= i32::midpoint(start_visual.col, end_visual.col))
            {
                nvim_set_VIsual_lnum_col_coladd(
                    start_visual.lnum,
                    start_visual.col,
                    start_visual.coladd,
                );
            } else {
                nvim_set_VIsual_lnum_col_coladd(end_visual.lnum, end_visual.col, end_visual.coladd);
            }
        }
    } else if (state & MODE_INSERT_IMPL) != 0 && nvim_get_VIsual_active_mouse() {
        nvim_stuffcharReadbuff(CTRL_O_CODE);
    }

    // ------------------------------------------------------------------
    // Middle click put, Ctrl-click tag, shift-click search,
    // multi-click word/line/block selection
    // ------------------------------------------------------------------
    let mm = nvim_get_mod_mask_mouse();

    if which_button == MOUSE_MIDDLE {
        if regname == 0 && nvim_eval_has_provider(c"clipboard".as_ptr()) {
            regname = c_int::from(b'*');
        }
        // Inline of C's nvim_do_put_middle_click: check mline, adjust dir, redo, put.
        let mut reg: *mut YankregHandle = std::ptr::null_mut();
        let is_mline = yank_register_mline(regname, &raw mut reg);
        if is_mline {
            if nvim_get_mouse_past_bottom() {
                dir = FORWARD_IMPL;
            }
        } else if nvim_get_mouse_past_eol() {
            dir = FORWARD_IMPL;
        }
        let fixindent_char = if fixindent {
            if dir == BACKWARD_IMPL {
                c_int::from(b'[')
            } else {
                c_int::from(b']')
            }
        } else if dir == FORWARD_IMPL {
            c_int::from(b'p')
        } else {
            c_int::from(b'P')
        };
        let fixindent_char2 = if fixindent {
            c_int::from(b'p')
        } else {
            NUL_KEY
        };
        rs_prep_redo(
            regname,
            count,
            NUL_KEY,
            fixindent_char,
            NUL_KEY,
            fixindent_char2,
            NUL_KEY,
        );
        if nvim_get_restart_edit_mouse() != 0 {
            nvim_set_where_paste_started_to_cursor();
        }
        do_put(
            regname,
            reg,
            dir,
            count,
            (if fixindent { PUT_FIXINDENT } else { 0 }) | PUT_CURSEND,
        );
    } else if ((mm & MOD_MASK_CTRL) != 0 || (mm & MOD_MASK_MULTI_CLICK) == MOD_MASK_2CLICK)
        && nvim_curwin_is_qf() != 0
    {
        nvim_do_cmdline_cmd_mouse(c".cc".as_ptr());
        GOT_CLICK = false;
    } else if ((mm & MOD_MASK_CTRL) != 0 || (mm & MOD_MASK_MULTI_CLICK) == MOD_MASK_2CLICK)
        && nvim_curwin_is_ll() != 0
    {
        nvim_do_cmdline_cmd_mouse(c".ll".as_ptr());
        GOT_CLICK = false;
    } else if (mm & MOD_MASK_CTRL) != 0
        || (nvim_curbuf_is_help_mouse() != 0 && (mm & MOD_MASK_MULTI_CLICK) == MOD_MASK_2CLICK)
    {
        let state = nvim_get_state_mouse();
        if (state & MODE_INSERT_IMPL) != 0 {
            nvim_stuffcharReadbuff(CTRL_O_CODE);
        }
        nvim_stuffcharReadbuff(CTRL_RSB_CODE);
        GOT_CLICK = false;
    } else if (mm & MOD_MASK_SHIFT) != 0 {
        let state = nvim_get_state_mouse();
        if (state & MODE_INSERT_IMPL) != 0 || nvim_get_VIsual_select_mouse() {
            nvim_stuffcharReadbuff(CTRL_O_CODE);
        }
        if which_button == MOUSE_LEFT {
            nvim_stuffcharReadbuff(c_int::from(b'*'));
        } else {
            nvim_stuffcharReadbuff(c_int::from(b'#'));
        }
    } else if in_status_line || in_sep_line {
        // Do nothing
    } else if (mm & MOD_MASK_MULTI_CLICK) != 0 && (state & (MODE_NORMAL | MODE_INSERT_IMPL)) != 0 {
        if is_click || !nvim_get_VIsual_active_mouse() {
            if nvim_get_VIsual_active_mouse() {
                ORIG_CURSOR = get_visual_pos();
            } else {
                nvim_set_VIsual_to_cursor();
                let curwin = nvim_get_curwin();
                ORIG_CURSOR = *nvim_win_get_cursor_ptr(curwin);
                VIsual_active = true;
                VIsual_reselect = true;
                rs_may_start_select(c_int::from(b'o'));
                setmouse_global();
            }
            let mm = nvim_get_mod_mask_mouse();
            if (mm & MOD_MASK_MULTI_CLICK) == MOD_MASK_2CLICK {
                if (mm & MOD_MASK_ALT) != 0 {
                    nvim_set_VIsual_mode_mouse(VISUAL_BLOCK);
                } else {
                    nvim_set_VIsual_mode_mouse(c_int::from(b'v'));
                }
            } else if (mm & MOD_MASK_MULTI_CLICK) == MOD_MASK_3CLICK {
                nvim_set_VIsual_mode_mouse(c_int::from(b'V'));
            } else if (mm & MOD_MASK_MULTI_CLICK) == MOD_MASK_4CLICK {
                nvim_set_VIsual_mode_mouse(VISUAL_BLOCK);
            }
        }

        // Double-click: select word or block
        let mm = nvim_get_mod_mask_mouse();
        if (mm & MOD_MASK_MULTI_CLICK) == MOD_MASK_2CLICK {
            let mut found_match = false;

            if is_click {
                // Skip white space, then check for bracket match
                let curwin = nvim_get_curwin();
                let mut ep = *nvim_win_get_cursor_ptr(curwin);

                loop {
                    let gc = gchar_pos(&raw const ep);
                    if nvim_ascii_iswhite_mouse(gc) == 0 {
                        break;
                    }
                    if inc(&raw mut ep) < 0 {
                        break;
                    }
                }
                nvim_set_cursor_pos(ep.lnum, ep.col, ep.coladd);
                oap_set_motion_type(oap, MT_CHAR_WISE);

                let curwin = nvim_get_curwin();
                let ep = *nvim_win_get_cursor_ptr(curwin);
                let vis = get_visual_pos();
                let vmode = nvim_get_VIsual_mode_mouse();
                let gc = gchar_pos(&raw const ep);

                if !oap.is_null()
                    && vmode == c_int::from(b'v')
                    && nvim_vim_iswordc_mouse(gc) == 0
                    && pos_equal(ep, vis)
                {
                    let mut m_lnum: linenr_T = 0;
                    let mut m_col: c_int = 0;
                    let mut m_coladd: c_int = 0;
                    let mut mot: c_int = 0;
                    if nvim_findmatch_nul(
                        oap,
                        &raw mut m_lnum,
                        &raw mut m_col,
                        &raw mut m_coladd,
                        &raw mut mot,
                    ) {
                        nvim_set_cursor_pos(m_lnum, m_col, m_coladd);
                        if mot == MT_LINE_WISE {
                            nvim_set_VIsual_mode_mouse(c_int::from(b'V'));
                        } else {
                            let psel = nvim_get_p_sel();
                            if !psel.is_null() && *psel.cast::<u8>() == b'e' {
                                let curwin = nvim_get_curwin();
                                let cursor = *nvim_win_get_cursor_ptr(curwin);
                                let vis = get_visual_pos();
                                if pos_lt(cursor, vis) {
                                    nvim_inc_VIsual_col();
                                } else {
                                    nvim_inc_cursor_col();
                                }
                            }
                        }
                        found_match = true;
                    }
                }
            }

            if !found_match && (is_click || is_drag) {
                let curwin = nvim_get_curwin();
                let cursor = *nvim_win_get_cursor_ptr(curwin);
                let orig = ORIG_CURSOR;
                let psel = nvim_get_p_sel();
                let sel_exclusive = !psel.is_null() && *psel.cast::<u8>() == b'e';

                if pos_lt(cursor, orig) {
                    let line = nvim_ml_get_line(cursor.lnum);
                    nvim_set_cursor_col(rs_find_start_of_word(line, cursor.col));
                    let vis = get_visual_pos();
                    let line = nvim_ml_get_line(vis.lnum);
                    nvim_set_VIsual_col_only(rs_find_end_of_word(line, vis.col, sel_exclusive));
                } else {
                    let vis = get_visual_pos();
                    let line = nvim_ml_get_line(vis.lnum);
                    nvim_set_VIsual_col_only(rs_find_start_of_word(line, vis.col));
                    if sel_exclusive {
                        let cp = nvim_get_cursor_pos_ptr_mouse();
                        if !cp.is_null() && *cp.cast::<u8>() != 0 {
                            let len = nvim_utfc_ptr2len_at_cursor();
                            let curwin = nvim_get_curwin();
                            let cursor = *nvim_win_get_cursor_ptr(curwin);
                            nvim_set_cursor_col(cursor.col + len);
                        }
                    }
                    let curwin = nvim_get_curwin();
                    let cursor = *nvim_win_get_cursor_ptr(curwin);
                    let line = nvim_ml_get_line(cursor.lnum);
                    nvim_set_cursor_col(rs_find_end_of_word(line, cursor.col, sel_exclusive));
                }
            }
            nvim_set_curswant_flag();
        }

        if is_click {
            redraw_curbuf_later(UPD_INVERTED);
        }
    } else if nvim_get_VIsual_active_mouse() && !old_active {
        let mm = nvim_get_mod_mask_mouse();
        if (mm & MOD_MASK_ALT) != 0 {
            nvim_set_VIsual_mode_mouse(VISUAL_BLOCK);
        } else {
            nvim_set_VIsual_mode_mouse(c_int::from(b'v'));
        }
    }

    // ------------------------------------------------------------------
    // Show visual mode change later
    // ------------------------------------------------------------------
    let vis_active = nvim_get_VIsual_active_mouse();
    if (!vis_active && old_active && nvim_get_mode_displayed_mouse())
        || (vis_active
            && nvim_get_p_smd_mouse() != 0
            && nvim_get_msg_silent_mouse() == 0
            && (!old_active || nvim_get_VIsual_mode_mouse() != old_mode))
    {
        nvim_set_redraw_cmdline_mouse(true);
    }

    moved
}

/// Do the appropriate action for the current mouse click in the current mode.
///
/// # Safety
/// `oap` may be null. Otherwise must be a valid `oparg_T` pointer.
#[export_name = "do_mouse"]
pub unsafe extern "C" fn rs_do_mouse(
    oap: OpargHandle,
    c: c_int,
    dir: c_int,
    count: c_int,
    fixindent: bool,
) -> bool {
    rs_do_mouse_impl(oap, c, dir, count, fixindent)
}

/// Mouse clicks and drags (Normal/Visual mode entry point).
///
/// # Safety
/// `cap` must be a valid `cmdarg_T` pointer.
#[export_name = "nv_mouse"]
pub unsafe extern "C" fn rs_nv_mouse(cap: CmdargHandle) {
    let oap = (*cap.cast::<CmdargT>()).oap;
    let cmdchar = (*cap.cast::<CmdargT>()).cmdchar;
    let count1 = (*cap.cast::<CmdargT>()).count1;
    rs_do_mouse(oap, cmdchar, -1 /* BACKWARD */, count1, false);
}

// =============================================================================
// Phase 3 — call_click_def_func
// =============================================================================

/// Shift modifier mask
const MOD_MASK_SHIFT: c_int = 0x02;
/// Ctrl modifier mask
const MOD_MASK_CTRL: c_int = 0x04;
/// Meta modifier mask (distinct from Alt)
const MOD_MASK_META: c_int = 0x10;

/// Opaque handle for `StlClickDefinition *` (statusline/winbar/tabline click defs).
type StlClickDefinitionHandle = *mut std::ffi::c_void;

/// Rust mirror of C `StlClickDefinition` (layout verified against `statusline_defs.h`).
/// Fields: type(i32), tabnr(i32), func(*char)
#[repr(C)]
struct StlClickDefinition {
    /// Click type enum (kStlClickDisabled=0..kStlClickFuncRun=3)
    click_type: c_int,
    /// Tab page number
    tabnr: c_int,
    /// `VimL` function name (NUL-terminated)
    func: *mut c_char,
}

// typval_T type constants (from eval/typval_defs.h)
const VAR_NUMBER: c_int = 4;
const VAR_STRING: c_int = 2;
// VarLockStatus VAR_FIXED = 1
const VAR_FIXED: c_int = 1;

extern "C" {
    /// Call a `VimL` function: `call_vim_function(func, argc, argv, rettv)`.
    fn call_vim_function(
        func: *const c_char,
        argc: c_int,
        argv: *mut TypvalT,
        rettv: *mut TypvalT,
    ) -> c_int;
    /// Clear a typval (frees its contents).
    fn tv_clear(tv: *mut TypvalT);
}

/// Call the `VimL` function registered for a statusline/winbar/tabline click.
///
/// Computes the click count, button string, and modifier string from the
/// current `mod_mask` global and the given `which_button`, then builds
/// `typval_T` args in Rust and calls `call_vim_function` directly.
///
/// After the callback returns, `got_click` is cleared so that the next click
/// is not mistakenly treated as a drag.
///
/// # Safety
/// `click_defs` must be a valid pointer into the click-definition array for
/// the relevant UI element, and `col` must be a valid index into that array.
#[no_mangle]
pub unsafe extern "C" fn rs_call_click_def_func(
    click_defs: StlClickDefinitionHandle,
    col: c_int,
    which_button: c_int,
) {
    let mm = mod_mask;

    // Click count from multi-click mask bits.
    let click_count = rs_get_click_count(mm);

    // Button string: single char for l/r/m, two chars for x1/x2.
    let button_str: &[u8] = match which_button {
        MOUSE_LEFT => b"l\0",
        MOUSE_RIGHT => b"r\0",
        MOUSE_MIDDLE => b"m\0",
        MOUSE_X1 => b"x1\0",
        MOUSE_X2 => b"x2\0",
        _ => b"?\0",
    };

    // Modifier string: four chars (s/c/a/m) followed by NUL.
    let modifier_str: [u8; 5] = [
        if (mod_mask & MOD_MASK_SHIFT) != 0 {
            b's'
        } else {
            b' '
        },
        if (mod_mask & MOD_MASK_CTRL) != 0 {
            b'c'
        } else {
            b' '
        },
        if (mod_mask & MOD_MASK_ALT) != 0 {
            b'a'
        } else {
            b' '
        },
        if (mod_mask & MOD_MASK_META) != 0 {
            b'm'
        } else {
            b' '
        },
        0u8, // NUL terminator
    ];

    // Access the click definition for this column (col >= 0 is enforced by callers).
    #[allow(clippy::cast_sign_loss)]
    let def = &*click_defs.cast::<StlClickDefinition>().add(col as usize);

    // Build typval_T args on the stack (mirrors C's nvim_call_stl_click_func).
    let mut argv: [TypvalT; 4] = [
        TypvalT {
            v_type: VAR_NUMBER,
            v_lock: VAR_FIXED,
            vval: TypvalVval {
                v_number: i64::from(def.tabnr),
            },
        },
        TypvalT {
            v_type: VAR_NUMBER,
            v_lock: VAR_FIXED,
            vval: TypvalVval {
                v_number: i64::from(click_count),
            },
        },
        TypvalT {
            v_type: VAR_STRING,
            v_lock: VAR_FIXED,
            vval: TypvalVval {
                v_string: button_str.as_ptr().cast::<c_char>().cast_mut(),
            },
        },
        TypvalT {
            v_type: VAR_STRING,
            v_lock: VAR_FIXED,
            vval: TypvalVval {
                v_string: modifier_str.as_ptr().cast::<c_char>().cast_mut(),
            },
        },
    ];
    let mut rettv: TypvalT = TypvalT {
        v_type: 0,
        v_lock: 0,
        vval: TypvalVval { v_number: 0 },
    };
    call_vim_function(def.func, 4, argv.as_mut_ptr(), &raw mut rettv);
    tv_clear(&raw mut rettv);

    // Ensure the next click is not treated as a drag.
    GOT_CLICK = false;
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mouse_button_constants() {
        assert_eq!(MOUSE_LEFT, 0x00);
        assert_eq!(MOUSE_MIDDLE, 0x01);
        assert_eq!(MOUSE_RIGHT, 0x02);
        assert_eq!(MOUSE_RELEASE, 0x03);
        assert_eq!(MOUSE_X1, 0x300);
        assert_eq!(MOUSE_X2, 0x400);
    }

    #[test]
    fn test_jump_to_mouse_constants() {
        assert_eq!(IN_UNKNOWN, 0);
        assert_eq!(IN_BUFFER, 1);
        assert_eq!(IN_STATUS_LINE, 2);
        assert_eq!(IN_SEP_LINE, 4);
        assert_eq!(IN_OTHER_WIN, 8);
        assert_eq!(CURSOR_MOVED, 0x100);
        assert_eq!(MOUSE_FOLD_CLOSE, 0x200);
        assert_eq!(MOUSE_FOLD_OPEN, 0x400);
        assert_eq!(MOUSE_WINBAR, 0x800);
        assert_eq!(MOUSE_STATUSCOL, 0x1000);
    }

    #[test]
    fn test_mouse_flags_constants() {
        assert_eq!(MOUSE_FOCUS, 0x01);
        assert_eq!(MOUSE_MAY_VIS, 0x02);
        assert_eq!(MOUSE_DID_MOVE, 0x04);
        assert_eq!(MOUSE_SETPOS, 0x08);
        assert_eq!(MOUSE_MAY_STOP_VIS, 0x10);
        assert_eq!(MOUSE_RELEASED, 0x20);
    }

    #[test]
    fn test_scroll_constants() {
        assert_eq!(MSCR_DOWN, 0);
        assert_eq!(MSCR_UP, 1);
        assert_eq!(MSCR_LEFT, -1);
        assert_eq!(MSCR_RIGHT, -2);
    }

    #[test]
    fn test_char_class() {
        assert_eq!(CharClass::BLANK.0, 0);
        assert_eq!(CharClass::PUNCTUATION.0, 1);
        assert_eq!(CharClass::WORD.0, 2);
    }

    #[test]
    fn test_mouse_model_popup_null() {
        unsafe {
            assert!(!rs_mouse_model_popup(std::ptr::null()));
        }
    }

    #[test]
    fn test_mouse_model_popup_popup() {
        let popup = b"popup\0";
        unsafe {
            assert!(rs_mouse_model_popup(popup.as_ptr().cast()));
        }
    }

    #[test]
    fn test_mouse_model_popup_other() {
        let extend = b"extend\0";
        unsafe {
            assert!(!rs_mouse_model_popup(extend.as_ptr().cast()));
        }
    }

    #[test]
    fn test_fold_click_result_positive_vcol() {
        let result = rs_check_fold_click(10);
        assert_eq!(result.flags, 0);
        assert!(result.use_vcol);
    }

    #[test]
    fn test_fold_click_result_zero_vcol() {
        let result = rs_check_fold_click(0);
        assert_eq!(result.flags, 0);
        assert!(result.use_vcol);
    }

    #[test]
    fn test_fold_click_result_fold_open() {
        let result = rs_check_fold_click(VCOL_FOLD_OPEN);
        assert_eq!(result.flags, MOUSE_FOLD_OPEN);
        assert!(!result.use_vcol);
    }

    #[test]
    fn test_fold_click_result_fold_close() {
        let result = rs_check_fold_click(VCOL_FOLD_CLOSE);
        assert_eq!(result.flags, MOUSE_FOLD_CLOSE);
        assert!(!result.use_vcol);
    }

    #[test]
    fn test_fold_click_result_other_negative() {
        let result = rs_check_fold_click(-1);
        assert_eq!(result.flags, 0);
        assert!(!result.use_vcol);
    }

    #[test]
    fn test_fold_vcol_constants() {
        assert_eq!(VCOL_FOLD_OPEN, -2);
        assert_eq!(VCOL_FOLD_CLOSE, -3);
    }

    #[test]
    fn test_visual_mode_constants() {
        assert_eq!(VISUAL_CHAR, c_int::from(b'v'));
        assert_eq!(VISUAL_LINE, c_int::from(b'V'));
        assert_eq!(VISUAL_BLOCK, 0x16);
    }

    #[test]
    fn test_selection_mode_double_click() {
        let mode = rs_compute_selection_mode(MOD_MASK_2CLICK);
        assert_eq!(mode, VISUAL_CHAR);
    }

    #[test]
    fn test_selection_mode_double_click_with_alt() {
        let mode = rs_compute_selection_mode(MOD_MASK_2CLICK | MOD_MASK_ALT);
        assert_eq!(mode, VISUAL_BLOCK);
    }

    #[test]
    fn test_selection_mode_triple_click() {
        let mode = rs_compute_selection_mode(MOD_MASK_3CLICK);
        assert_eq!(mode, VISUAL_LINE);
    }

    #[test]
    fn test_selection_mode_quadruple_click() {
        let mode = rs_compute_selection_mode(MOD_MASK_4CLICK);
        assert_eq!(mode, VISUAL_BLOCK);
    }

    #[test]
    fn test_selection_mode_single_click() {
        let mode = rs_compute_selection_mode(0);
        assert_eq!(mode, 0);
    }

    #[test]
    fn test_click_count() {
        assert_eq!(rs_get_click_count(0), 1);
        assert_eq!(rs_get_click_count(MOD_MASK_2CLICK), 2);
        assert_eq!(rs_get_click_count(MOD_MASK_3CLICK), 3);
        assert_eq!(rs_get_click_count(MOD_MASK_4CLICK), 4);
    }

    #[test]
    fn test_is_multi_click() {
        assert!(!rs_is_multi_click(0));
        assert!(rs_is_multi_click(MOD_MASK_2CLICK));
        assert!(rs_is_multi_click(MOD_MASK_3CLICK));
        assert!(rs_is_multi_click(MOD_MASK_4CLICK));
    }

    #[test]
    fn test_is_double_click() {
        assert!(!rs_is_double_click(0));
        assert!(rs_is_double_click(MOD_MASK_2CLICK));
        assert!(!rs_is_double_click(MOD_MASK_3CLICK));
        assert!(!rs_is_double_click(MOD_MASK_4CLICK));
    }

    #[test]
    fn test_horiz_scroll_left() {
        // Scrolling left increases leftcol
        let result = rs_compute_horiz_scroll(10, MSCR_LEFT, 5);
        assert_eq!(result, 15);
    }

    #[test]
    fn test_horiz_scroll_right() {
        // Scrolling right decreases leftcol
        let result = rs_compute_horiz_scroll(10, MSCR_RIGHT, 5);
        assert_eq!(result, 5);
    }

    #[test]
    fn test_horiz_scroll_clamp_to_zero() {
        // Cannot go below 0
        let result = rs_compute_horiz_scroll(3, MSCR_RIGHT, 10);
        assert_eq!(result, 0);
    }

    #[test]
    fn test_scroll_lines_with_modifier() {
        // With shift/ctrl, scroll whole page
        let result = rs_compute_scroll_lines(true, 25, 3);
        assert_eq!(result, 25);
    }

    #[test]
    fn test_scroll_lines_without_modifier() {
        // Without modifier, use default
        let result = rs_compute_scroll_lines(false, 25, 3);
        assert_eq!(result, 3);
    }

    #[test]
    fn test_is_vertical_scroll() {
        assert!(rs_is_vertical_scroll(MSCR_UP));
        assert!(rs_is_vertical_scroll(MSCR_DOWN));
        assert!(!rs_is_vertical_scroll(MSCR_LEFT));
        assert!(!rs_is_vertical_scroll(MSCR_RIGHT));
    }

    #[test]
    fn test_is_horizontal_scroll() {
        assert!(!rs_is_horizontal_scroll(MSCR_UP));
        assert!(!rs_is_horizontal_scroll(MSCR_DOWN));
        assert!(rs_is_horizontal_scroll(MSCR_LEFT));
        assert!(rs_is_horizontal_scroll(MSCR_RIGHT));
    }
}
