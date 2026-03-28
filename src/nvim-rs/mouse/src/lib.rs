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

use nvim_normal::types::CmdargT;
use std::ffi::{c_char, c_int};

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

// =============================================================================
// C accessors for mouse state
// =============================================================================

#[allow(dead_code)]
extern "C" {
    static Rows: c_int;
    static Columns: c_int;

    /// `p_mousescroll_vert` option value (`OptInt` = `int64_t`).
    static p_mousescroll_vert: i64;

    /// `p_mousescroll_hor` option value (`OptInt` = `int64_t`).
    static p_mousescroll_hor: i64;

    /// Get whether a click was received.
    fn nvim_get_got_click() -> bool;

    /// Set whether a click was received.
    fn nvim_set_got_click(val: bool);

    /// Get the window being dragged.
    fn nvim_get_dragwin() -> WinHandle;

    /// Set the window being dragged.
    fn nvim_set_dragwin(wp: WinHandle);

    /// Check if a window is being dragged.
    fn nvim_is_dragging() -> bool;

    // --- Window field accessors ---

    /// Get `w_topline` field from a window.
    fn nvim_win_get_topline(wp: WinHandle) -> linenr_T;

    /// Get `w_topfill` field from a window.
    fn nvim_win_get_topfill(wp: WinHandle) -> c_int;

    // --- Mouse globals ---

    /// Get `mouse_col` global.
    fn nvim_get_mouse_col() -> c_int;

    // --- Tabpage operations ---

    /// Get `tabnr` from `tab_page_click_defs` at given column.
    fn nvim_mouse_get_tab_click_tabnr(col: c_int) -> c_int;

    /// Get the current tabpage.
    fn nvim_get_curtab() -> TabpageHandle;

    /// Get the first tabpage.
    fn nvim_get_first_tabpage() -> TabpageHandle;

    /// Check if there is more than one tabpage.
    fn nvim_first_tabpage_has_next() -> c_int;

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
pub const extern "C" fn rs_compute_selection_mode(mod_mask: c_int) -> c_int {
    let multi_click = mod_mask & MOD_MASK_MULTI_CLICK;
    let alt_pressed = (mod_mask & MOD_MASK_ALT) != 0;

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
pub const extern "C" fn rs_get_click_count(mod_mask: c_int) -> c_int {
    let multi_click = mod_mask & MOD_MASK_MULTI_CLICK;

    match multi_click {
        MOD_MASK_4CLICK => 4,
        MOD_MASK_3CLICK => 3,
        MOD_MASK_2CLICK => 2,
        _ => 1,
    }
}

/// Check if this is a multi-click (double, triple, or quadruple click).
#[no_mangle]
pub const extern "C" fn rs_is_multi_click(mod_mask: c_int) -> bool {
    (mod_mask & MOD_MASK_MULTI_CLICK) != 0
}

/// Check if this is specifically a double-click.
#[no_mangle]
pub const extern "C" fn rs_is_double_click(mod_mask: c_int) -> bool {
    (mod_mask & MOD_MASK_MULTI_CLICK) == MOD_MASK_2CLICK
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
    ORIG_TOPLINE = nvim_win_get_topline(wp);
    ORIG_TOPFILL = nvim_win_get_topfill(wp);
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
    nvim_set_dragwin(std::ptr::null_mut());
}

/// Move the current tab to the tab in the same column as the mouse,
/// or to the end of the tabline if there is no tab there.
///
/// # Safety
/// Requires valid `tab_page_click_defs` array and valid `mouse_col`.
#[no_mangle]
pub unsafe extern "C" fn rs_move_tab_to_mouse() {
    let tabnr = nvim_mouse_get_tab_click_tabnr(nvim_get_mouse_col());
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
        if nvim_first_tabpage_has_next() != 0 {
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
    fn nvim_win_get_p_rl(wp: WinHandle) -> c_int;

    /// Get `w_view_width` field.
    fn nvim_win_get_view_width(wp: WinHandle) -> c_int;

    /// Get `w_skipcol` field.
    fn nvim_win_get_skipcol(wp: WinHandle) -> c_int;

    /// Check if window may have filler lines.
    fn nvim_win_may_fill(wp: WinHandle) -> c_int;

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
    fn nvim_win_col_off(wp: WinHandle) -> c_int;

    /// Get secondary column offset.
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

    if nvim_win_get_p_rl(win) != 0 {
        col = nvim_win_get_view_width(win) - 1 - col;
    }

    let topline = nvim_win_get_topline(win);
    let mut lnum = topline;
    let line_count = nvim_win_buf_line_count(win);
    let view_width = nvim_win_get_view_width(win);

    while row > 0 {
        // Don't include filler lines in "count"
        let count = if nvim_win_may_fill(win) != 0 {
            let fill = if lnum == topline {
                nvim_win_get_topfill(win)
            } else {
                nvim_win_get_fill(win, lnum)
            };
            row -= fill;
            nvim_plines_win_nofill(win, lnum, 0)
        } else {
            nvim_plines_win(win, lnum, 0)
        };

        let mut adjusted_count = count;
        if nvim_win_get_skipcol(win) > 0 && lnum == topline {
            let width1 = view_width - nvim_win_col_off(win);
            if width1 > 0 {
                let skipcol = nvim_win_get_skipcol(win);
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
            col += nvim_win_get_skipcol(win);
        }
    }

    if nvim_win_get_p_wrap(win) == 0 {
        col += nvim_win_get_leftcol(win);
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

    /// Get `w_p_wrap` field.
    fn nvim_win_get_p_wrap(wp: WinHandle) -> c_int;

    /// Get `w_leftcol` field.
    fn nvim_win_get_leftcol(wp: WinHandle) -> c_int;

    /// Get `w_topline` field.
    fn nvim_win_get_botline(wp: WinHandle) -> linenr_T;

    /// Get `w_cursor.lnum` field.
    fn nvim_win_get_cursor_lnum(wp: WinHandle) -> linenr_T;

    /// Set `w_cursor.lnum` field.
    fn nvim_win_set_cursor_lnum(wp: WinHandle, lnum: linenr_T);

    /// Set `w_cursor.col` field.
    fn nvim_win_set_cursor_col(wp: WinHandle, col: c_int);

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
    let topline = nvim_win_get_topline(curwin);
    let botline = nvim_win_get_botline(curwin);
    let cursor_lnum = nvim_win_get_cursor_lnum(curwin);
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

    if nvim_win_get_p_wrap(curwin) != 0 {
        return false; // no horizontal scrolling when wrapping
    }
    if nvim_win_get_leftcol(curwin) == leftcol {
        return false; // already there
    }

    // When the line of the cursor is too short, move the cursor to the
    // longest visible line.
    if nvim_win_virtual_active(curwin) == 0
        && leftcol > rs_scroll_line_len(nvim_win_get_cursor_lnum(curwin))
    {
        nvim_win_set_cursor_lnum(curwin, rs_find_longest_lnum());
        nvim_win_set_cursor_col(curwin, 0);
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
    /// C accessor: resolve grid handle to window and adjust row/col.
    fn nvim_mouse_find_grid_win(gridp: *mut c_int, rowp: *mut c_int, colp: *mut c_int)
        -> WinHandle;

    /// C accessor: traverse frame tree to find window at row/col.
    fn nvim_frame_find_win(rowp: *mut c_int, colp: *mut c_int) -> WinHandle;

    /// Get `w_winrow_off` field.
    fn nvim_win_get_winrow_off(wp: WinHandle) -> c_int;

    /// Get `w_wincol_off` field.
    fn nvim_win_get_wincol_off(wp: WinHandle) -> c_int;
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
    let wp_grid = nvim_mouse_find_grid_win(gridp, rowp, colp);
    if !wp_grid.is_null() {
        return wp_grid;
    }
    if *gridp > 1 {
        return std::ptr::null_mut();
    }

    // Fall back to frame tree traversal
    nvim_frame_find_win(rowp, colp)
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
        *rowp += nvim_win_get_winrow_off(wp);
        *colp += nvim_win_get_wincol_off(wp);
    }
    wp
}

// =============================================================================
// Phase 5 — Mid-level Functions
// =============================================================================

extern "C" {
    /// Get `mouse_grid` global.
    fn nvim_get_mouse_grid() -> c_int;

    /// Get `mouse_row` global.
    fn nvim_get_mouse_row() -> c_int;

    /// Get `w_p_stc` field (statuscolumn option string).
    fn nvim_win_get_p_stc(wp: WinHandle) -> *const c_char;

    /// Get `w_view_height` field.
    fn nvim_win_get_view_height(wp: WinHandle) -> c_int;

    /// Get `w_status_height` field.
    fn nvim_win_get_status_height(wp: WinHandle) -> c_int;

    /// Get `w_winbar_height` field.
    fn nvim_win_get_winbar_height(wp: WinHandle) -> c_int;

    /// Get `Rows` global.

    /// Get `p_ch` global (command height).
    static mut p_ch: i64;

    /// Get `global_stl_height()`.
    #[link_name = "rs_global_stl_height"]
    fn nvim_global_stl_height() -> c_int;

    /// C accessor: grid-based vcol/flags lookup for mouse click.
    fn nvim_mouse_check_grid_impl(vcolp: *mut colnr_T, flagsp: *mut c_int);

    /// C accessor: `ins_mousescroll` logic.
    fn nvim_ins_mousescroll_impl(dir: c_int);

    // --- Phase 2 accessors ---

    /// Set `curwin`.
    fn nvim_set_curwin(wp: WinHandle);

    /// Set `curbuf` (`buf_T*`).
    fn nvim_set_curbuf(buf: *mut std::ffi::c_void);

    /// Get `w_buffer` from a window.
    fn nvim_win_get_w_buffer(wp: WinHandle) -> *mut std::ffi::c_void;

    /// Set `w_redr_status` field.
    fn nvim_win_set_redr_status(wp: WinHandle, val: c_int);

    /// Get `mod_mask` global.
    fn nvim_get_mod_mask() -> c_int;

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
}

// =============================================================================
// Phase 3 — Popup Menu Implementation (extern declarations)
// =============================================================================

extern "C" {
    /// `VIsual_active` global.
    static VIsual_active: bool;

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

/// Check the mouse click grid for virtual column and fold flags.
///
/// # Safety
/// All pointers must be valid.
#[no_mangle]
pub unsafe extern "C" fn rs_mouse_check_grid(vcolp: *mut colnr_T, flagsp: *mut c_int) {
    nvim_mouse_check_grid_impl(vcolp, flagsp);
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
    let row = nvim_get_mouse_row();
    let col = nvim_get_mouse_col();
    if row < 0 || col < 0 {
        return IN_UNKNOWN;
    }

    let mut grid = nvim_get_mouse_grid();
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
            if nvim_win_get_p_rl(wp) != 0 {
                if wincol >= nvim_win_get_view_width(wp) - col_off {
                    return MOUSE_STATUSCOL;
                }
            } else if wincol < col_off {
                return MOUSE_STATUSCOL;
            }
        }
    }

    let view_height = nvim_win_get_view_height(wp);
    let status_height = nvim_win_get_status_height(wp);

    if winrow >= view_height + status_height {
        // Below window — check for global status line
        let mouse_grid_val = nvim_get_mouse_grid();
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

    if winrow < 0 && winrow + nvim_win_get_winbar_height(wp) >= 0 {
        return MOUSE_WINBAR;
    }

    if wincol >= nvim_win_get_view_width(wp) {
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
    nvim_set_got_click(false); // ignore release events
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
            let old_buf = nvim_win_get_w_buffer(old_curwin);
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
            nvim_set_curbuf(nvim_win_get_w_buffer(new_curwin));
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
    let mod_mask = nvim_get_mod_mask();
    // MOD_MASK_SHIFT = 0x02, MOD_MASK_CTRL = 0x04
    let shift_or_ctrl = (mod_mask & (0x02 | 0x04)) != 0;
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
                nvim_win_get_botline(curwin) - nvim_win_get_topline(curwin)
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
            nvim_win_get_view_width(curwin)
        } else {
            p_mousescroll_hor as c_int
        };
        let leftcol =
            nvim_win_get_leftcol(curwin) + if cap_arg == MSCR_RIGHT { -step } else { step };
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

    let mouse_row = nvim_get_mouse_row();
    let mouse_col = nvim_get_mouse_col();
    if mouse_row >= 0 && mouse_col >= 0 {
        // Find the window at the mouse pointer coordinates.
        // NOTE: Must restore curwin to old_curwin before returning!
        let mut grid = nvim_get_mouse_grid();
        let mut row = mouse_row;
        let mut col = mouse_col;
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
        nvim_set_curbuf(nvim_win_get_w_buffer(wp));
    }

    // Call the common mouse scroll function shared with other modes.
    rs_do_mousescroll(cap);

    let curwin = nvim_get_curwin();
    nvim_win_set_redr_status(curwin, 1);
    nvim_set_curwin(old_curwin);
    nvim_set_curbuf(nvim_win_get_w_buffer(old_curwin));
}

/// Insert mode mouse scroll handler.
///
/// # Safety
/// Requires valid window and insert mode state.
#[export_name = "ins_mousescroll"]
pub unsafe extern "C" fn rs_ins_mousescroll(dir: c_int) {
    nvim_ins_mousescroll_impl(dir);
}

// =============================================================================
// Phase 6 — jump_to_mouse
// =============================================================================

extern "C" {
    /// C accessor: the actual `jump_to_mouse` logic with static state.
    fn nvim_jump_to_mouse_impl(flags: c_int, inclusive: *mut bool, which_button: c_int) -> c_int;
}

/// Move the cursor to the specified row and column on the screen.
///
/// Change current window if necessary. Returns an integer with the
/// `CURSOR_MOVED` bit set if the cursor has moved or unset otherwise.
///
/// # Safety
/// `inclusive` may be null. Otherwise must be a valid pointer.
#[export_name = "jump_to_mouse"]
pub unsafe extern "C" fn rs_jump_to_mouse(
    flags: c_int,
    inclusive: *mut bool,
    which_button: c_int,
) -> c_int {
    nvim_jump_to_mouse_impl(flags, inclusive, which_button)
}

// =============================================================================
// Phase 7 — do_mouse and nv_mouse
// =============================================================================

extern "C" {
    /// C accessor: the actual `do_mouse` logic.
    fn nvim_do_mouse_impl(
        oap: OpargHandle,
        c: c_int,
        dir: c_int,
        count: c_int,
        fixindent: bool,
    ) -> bool;
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
    nvim_do_mouse_impl(oap, c, dir, count, fixindent)
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

extern "C" {
    /// C bridge: build `typval_T` args and call the `VimL` click callback.
    ///
    /// Takes pre-computed `click_count`, `button_str`, and `modifier_str`
    /// so that typval construction stays in C.
    fn nvim_call_stl_click_func(
        click_defs: StlClickDefinitionHandle,
        col: c_int,
        click_count: c_int,
        button_str: *const c_char,
        modifier_str: *const c_char,
    );
}

/// Call the `VimL` function registered for a statusline/winbar/tabline click.
///
/// Computes the click count, button string, and modifier string from the
/// current `mod_mask` global and the given `which_button`, then delegates to
/// the C helper `nvim_call_stl_click_func` for the actual `call_vim_function`
/// invocation (which requires `typval_T` construction that stays in C).
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
    let mod_mask = nvim_get_mod_mask();

    // Click count from multi-click mask bits.
    let click_count = rs_get_click_count(mod_mask);

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

    nvim_call_stl_click_func(
        click_defs,
        col,
        click_count,
        button_str.as_ptr().cast::<c_char>(),
        modifier_str.as_ptr().cast::<c_char>(),
    );

    // Ensure the next click is not treated as a drag.
    nvim_set_got_click(false);
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
