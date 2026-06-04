//! Window functions for VimL.
//!
//! This module implements window-related functions from `src/nvim/eval/funcs.c`:
//! - Window identification helpers
//! - Window number/id conversion utilities
//!
//! ## Note
//!
//! These are helper functions that work with window identifiers.
//! The actual window operations require C FFI calls that access window state.

#![allow(clippy::must_use_candidate)]

use std::ffi::c_int;

// =============================================================================
// Window Identifier Types
// =============================================================================

/// Window identifier types in VimL.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(i32)]
pub enum WinIdType {
    /// Window number (1-based in tab)
    Number = 0,
    /// Window ID (global unique)
    Id = 1,
    /// Current window
    Current = 2,
    /// Invalid window
    Invalid = -1,
}

impl WinIdType {
    /// Create from C int.
    pub const fn from_c_int(val: c_int) -> Self {
        match val {
            0 => Self::Number,
            1 => Self::Id,
            2 => Self::Current,
            _ => Self::Invalid,
        }
    }

    /// Convert to C int.
    pub const fn to_c_int(self) -> c_int {
        self as c_int
    }
}

// =============================================================================
// Window Number Helpers
// =============================================================================

/// Special window number values.
pub mod special_winnr {
    /// Current window
    pub const CURRENT: i64 = 0;
    /// Previous window (#)
    pub const PREVIOUS: i64 = -1;
}

/// Check if a window number is valid (positive).
pub const fn is_valid_winnr(winnr: i64) -> bool {
    winnr > 0
}

/// Check if a window ID is valid (positive).
pub const fn is_valid_winid(winid: i64) -> bool {
    winid > 0
}

/// FFI export: check if window number is valid.
#[no_mangle]
pub extern "C" fn rs_win_is_valid_winnr(winnr: i64) -> bool {
    is_valid_winnr(winnr)
}

/// FFI export: check if window ID is valid.
#[no_mangle]
pub extern "C" fn rs_win_is_valid_winid(winid: i64) -> bool {
    is_valid_winid(winid)
}

// =============================================================================
// Window Position Helpers
// =============================================================================

/// Window position relative to viewport.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(i32)]
pub enum WinRelPos {
    /// Absolute position
    Absolute = 0,
    /// Relative to cursor
    Cursor = 1,
    /// Relative to window
    Window = 2,
    /// Relative to editor grid
    Editor = 3,
}

impl WinRelPos {
    /// Create from C int.
    pub const fn from_c_int(val: c_int) -> Self {
        match val {
            1 => Self::Cursor,
            2 => Self::Window,
            3 => Self::Editor,
            _ => Self::Absolute,
        }
    }

    /// Convert to C int.
    pub const fn to_c_int(self) -> c_int {
        self as c_int
    }
}

// =============================================================================
// Window Layout Helpers
// =============================================================================

/// Window split direction.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(i32)]
pub enum WinSplitDir {
    /// Split horizontally (above/below)
    Horizontal = 0,
    /// Split vertically (left/right)
    Vertical = 1,
}

impl WinSplitDir {
    /// Create from C int.
    pub const fn from_c_int(val: c_int) -> Self {
        match val {
            1 => Self::Vertical,
            _ => Self::Horizontal,
        }
    }

    /// Convert to C int.
    pub const fn to_c_int(self) -> c_int {
        self as c_int
    }
}

// =============================================================================
// Window Size and Position
// =============================================================================

/// Window dimensions.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct WinDimensions {
    /// Window width in columns
    pub width: c_int,
    /// Window height in rows
    pub height: c_int,
    /// Window row position
    pub row: c_int,
    /// Window column position
    pub col: c_int,
}

impl WinDimensions {
    /// Create new dimensions.
    pub const fn new(width: c_int, height: c_int, row: c_int, col: c_int) -> Self {
        Self {
            width,
            height,
            row,
            col,
        }
    }

    /// Check if dimensions are valid.
    pub const fn is_valid(&self) -> bool {
        self.width > 0 && self.height > 0
    }
}

/// FFI: Create window dimensions.
#[no_mangle]
pub extern "C" fn rs_win_dimensions_new(
    width: c_int,
    height: c_int,
    row: c_int,
    col: c_int,
) -> WinDimensions {
    WinDimensions::new(width, height, row, col)
}

/// FFI: Check if dimensions are valid.
///
/// # Safety
/// `dims` must be valid or null.
#[no_mangle]
pub unsafe extern "C" fn rs_win_dimensions_is_valid(dims: *const WinDimensions) -> c_int {
    if dims.is_null() {
        return 0;
    }
    c_int::from((*dims).is_valid())
}

// =============================================================================
// Window View State
// =============================================================================

/// Window view state for getwininfo().
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct WinViewState {
    /// Top line visible in window
    pub topline: i64,
    /// Bottom line visible in window
    pub botline: i64,
    /// Cursor line
    pub curline: i64,
    /// Cursor column
    pub curcol: i64,
    /// Text width available
    pub textoff: c_int,
}

impl WinViewState {
    /// Create new view state.
    pub const fn new() -> Self {
        Self {
            topline: 1,
            botline: 1,
            curline: 1,
            curcol: 1,
            textoff: 0,
        }
    }

    /// Check if line is visible in window.
    pub const fn is_line_visible(&self, lnum: i64) -> bool {
        lnum >= self.topline && lnum <= self.botline
    }
}

/// FFI: Create default view state.
#[no_mangle]
pub extern "C" fn rs_win_view_state_new() -> WinViewState {
    WinViewState::new()
}

/// FFI: Check if line is visible.
///
/// # Safety
/// `state` must be valid or null.
#[no_mangle]
pub unsafe extern "C" fn rs_win_is_line_visible(state: *const WinViewState, lnum: i64) -> c_int {
    if state.is_null() {
        return 0;
    }
    c_int::from((*state).is_line_visible(lnum))
}

// =============================================================================
// Window Type
// =============================================================================

/// Window type classification.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(i32)]
pub enum WinType {
    /// Normal editing window
    Normal = 0,
    /// Floating window
    Floating = 1,
    /// Preview window
    Preview = 2,
    /// Quickfix window
    Quickfix = 3,
    /// Location list window
    Loclist = 4,
    /// Command-line window
    Cmdwin = 5,
}

impl WinType {
    /// Create from C int.
    pub const fn from_c_int(val: c_int) -> Self {
        match val {
            1 => Self::Floating,
            2 => Self::Preview,
            3 => Self::Quickfix,
            4 => Self::Loclist,
            5 => Self::Cmdwin,
            _ => Self::Normal,
        }
    }

    /// Convert to C int.
    pub const fn to_c_int(self) -> c_int {
        self as c_int
    }

    /// Check if this is a special window type.
    pub const fn is_special(self) -> bool {
        !matches!(self, Self::Normal)
    }
}

/// FFI: Check if window type is special.
#[no_mangle]
pub extern "C" fn rs_win_type_is_special(win_type: c_int) -> c_int {
    c_int::from(WinType::from_c_int(win_type).is_special())
}

// =============================================================================
// Window Navigation
// =============================================================================

/// Window navigation direction.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(i32)]
pub enum WinNavDir {
    /// Move up
    Up = 0,
    /// Move down
    Down = 1,
    /// Move left
    Left = 2,
    /// Move right
    Right = 3,
    /// Next window
    Next = 4,
    /// Previous window
    Prev = 5,
}

impl WinNavDir {
    /// Create from C int.
    pub const fn from_c_int(val: c_int) -> Self {
        match val {
            1 => Self::Down,
            2 => Self::Left,
            3 => Self::Right,
            4 => Self::Next,
            5 => Self::Prev,
            _ => Self::Up,
        }
    }

    /// Convert to C int.
    pub const fn to_c_int(self) -> c_int {
        self as c_int
    }

    /// Check if direction is horizontal.
    pub const fn is_horizontal(self) -> bool {
        matches!(self, Self::Left | Self::Right)
    }

    /// Check if direction is vertical.
    pub const fn is_vertical(self) -> bool {
        matches!(self, Self::Up | Self::Down)
    }
}

/// FFI: Check if nav direction is horizontal.
#[no_mangle]
pub extern "C" fn rs_win_nav_is_horizontal(dir: c_int) -> c_int {
    c_int::from(WinNavDir::from_c_int(dir).is_horizontal())
}

/// FFI: Check if nav direction is vertical.
#[no_mangle]
pub extern "C" fn rs_win_nav_is_vertical(dir: c_int) -> c_int {
    c_int::from(WinNavDir::from_c_int(dir).is_vertical())
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_win_id_type() {
        assert_eq!(WinIdType::from_c_int(0), WinIdType::Number);
        assert_eq!(WinIdType::from_c_int(-1), WinIdType::Invalid);
    }

    #[test]
    fn test_is_valid_winnr() {
        assert!(is_valid_winnr(1));
        assert!(!is_valid_winnr(0));
        assert!(!is_valid_winnr(-1));
    }

    #[test]
    fn test_is_valid_winid() {
        assert!(is_valid_winid(1000));
        assert!(!is_valid_winid(0));
        assert!(!is_valid_winid(-1));
    }

    #[test]
    fn test_win_dimensions() {
        let dims = WinDimensions::new(80, 24, 0, 0);
        assert!(dims.is_valid());

        let invalid = WinDimensions::new(0, 24, 0, 0);
        assert!(!invalid.is_valid());
    }

    #[test]
    fn test_win_view_state() {
        let mut state = WinViewState::new();
        state.topline = 10;
        state.botline = 30;

        assert!(state.is_line_visible(10));
        assert!(state.is_line_visible(20));
        assert!(state.is_line_visible(30));
        assert!(!state.is_line_visible(9));
        assert!(!state.is_line_visible(31));
    }

    #[test]
    fn test_win_type() {
        assert_eq!(WinType::from_c_int(0), WinType::Normal);
        assert_eq!(WinType::from_c_int(1), WinType::Floating);
        assert!(!WinType::Normal.is_special());
        assert!(WinType::Floating.is_special());
        assert!(WinType::Quickfix.is_special());
    }

    #[test]
    fn test_win_nav_dir() {
        assert!(WinNavDir::Left.is_horizontal());
        assert!(WinNavDir::Right.is_horizontal());
        assert!(!WinNavDir::Up.is_horizontal());

        assert!(WinNavDir::Up.is_vertical());
        assert!(WinNavDir::Down.is_vertical());
        assert!(!WinNavDir::Left.is_vertical());
    }
}

// =============================================================================
// f_line / f_virtcol VimL built-in implementations
// =============================================================================

// These are appended to window.rs because they are logically window-position
// functions, and they reuse the switchwin_T opaque buffer defined here.

#[allow(clippy::too_many_lines)]
#[allow(clippy::cast_possible_truncation)]
#[allow(clippy::cast_possible_wrap)]
#[allow(clippy::cast_sign_loss)]
#[allow(clippy::cast_ptr_alignment)]
#[allow(clippy::ptr_as_ptr)]
#[allow(clippy::ptr_cast_constness)]
#[allow(clashing_extern_declarations)]
mod window_funcs {
    use std::ffi::{c_int, c_void};

    use crate::indexing::PosT;
    use crate::typval::TypvalT;

    // ─── Constants ────────────────────────────────────────────────────────────

    const VAR_UNKNOWN: c_int = 0;
    const OK: c_int = 1;

    /// Size of switchwin_T opaque buffer.
    /// Pinned by `_Static_assert(sizeof(switchwin_T) == SWITCHWIN_T_SIZE)` in C.
    const SWITCHWIN_T_SIZE: usize = 24;

    /// EvalFuncData — opaque union passed by value
    type EvalFuncData = *mut c_void;

    // ─── FFI imports ──────────────────────────────────────────────────────────

    extern "C" {
        fn tv_get_number(tv: *const c_void) -> i64;
        fn tv_get_bool(tv: *const c_void) -> c_int;
        fn win_id2wp_tp(id: c_int, tpp: *mut *mut c_void) -> *mut c_void;
        fn nvim_option_switch_win_noblock(
            switchwin: *mut c_void,
            win: *mut c_void,
            tabpage: *mut c_void,
        ) -> c_int;
        fn nvim_option_restore_win_noblock(switchwin: *mut c_void);
        fn check_cursor(wp: *mut c_void);
        fn var2fpos(tv: *const c_void, dolpos: bool, fnum: *mut c_int, charcol: bool) -> *mut PosT;
        fn nvim_eval_curwin() -> *mut c_void;
        fn nvim_eval_line_should_skip_topline(sw: *mut c_void, wp: *mut c_void) -> bool;
        fn nvim_eval_set_skip_update_topline(v: bool);
        fn getvvcol(
            wp: *mut c_void,
            pos: *mut PosT,
            start: *mut c_int,
            cursor: *mut c_int,
            end: *mut c_int,
        );
        fn ml_get_len(lnum: i32) -> c_int;
        fn nvim_eval_curbuf_fnum() -> c_int;
        fn nvim_eval_curbuf_line_count() -> c_int;
        fn tv_list_alloc_ret(rettv: *mut c_void, len: isize) -> *mut c_void;
        fn tv_list_append_number(l: *mut c_void, n: i64);
    }

    /// Return a pointer to `argvars[i]` (each typval_T is 16 bytes).
    #[inline]
    unsafe fn argvar(argvars: *const c_void, i: usize) -> *const c_void {
        argvars.cast::<u8>().add(i * 16).cast::<c_void>()
    }

    // ─── f_line ──────────────────────────────────────────────────────────────

    /// `line()` VimL function — line number for a position, optionally in a window.
    ///
    /// # Safety
    ///
    /// `argvars` and `rettv` must be valid `typval_T *`.
    #[export_name = "f_line"]
    pub unsafe extern "C" fn rs_f_line(
        argvars: *const c_void,
        rettv: *mut c_void,
        _fptr: EvalFuncData,
    ) {
        let mut lnum: i32 = 0;
        let mut fp: *mut PosT = std::ptr::null_mut();
        let mut fnum: c_int = 0;

        let arg1 = argvar(argvars, 1);
        if (*arg1.cast::<TypvalT>()).v_type == VAR_UNKNOWN {
            // use current window
            fp = var2fpos(argvar(argvars, 0), true, &raw mut fnum, false);
        } else {
            // use window specified in the second argument
            let id = tv_get_number(arg1) as c_int;
            let mut tp: *mut c_void = std::ptr::null_mut();
            let wp = win_id2wp_tp(id, &raw mut tp);
            if !wp.is_null() && !tp.is_null() {
                // Stack-allocate switchwin_T as an opaque byte buffer
                let mut switchwin = [0u8; SWITCHWIN_T_SIZE];
                let sw_ptr = switchwin.as_mut_ptr().cast::<c_void>();
                if nvim_option_switch_win_noblock(sw_ptr, wp, tp) == OK {
                    // With 'splitkeep' != cursor and in diff mode, prevent window scroll
                    if nvim_eval_line_should_skip_topline(sw_ptr, wp) {
                        nvim_eval_set_skip_update_topline(true);
                    }
                    check_cursor(nvim_eval_curwin());
                    fp = var2fpos(argvar(argvars, 0), true, &raw mut fnum, false);
                }
                nvim_eval_set_skip_update_topline(false);
                nvim_option_restore_win_noblock(sw_ptr);
            }
        }

        if !fp.is_null() {
            lnum = (*fp).lnum;
        }
        (*rettv.cast::<TypvalT>()).vval.v_number = i64::from(lnum);
    }

    // ─── f_virtcol ───────────────────────────────────────────────────────────

    /// `virtcol()` VimL function — virtual column of a position.
    ///
    /// # Safety
    ///
    /// `argvars` and `rettv` must be valid `typval_T *`.
    #[export_name = "f_virtcol"]
    pub unsafe extern "C" fn rs_f_virtcol(
        argvars: *const c_void,
        rettv: *mut c_void,
        _fptr: EvalFuncData,
    ) {
        let mut vcol_start: c_int = 0;
        let mut vcol_end: c_int = 0;
        let mut switchwin = [0u8; SWITCHWIN_T_SIZE];
        let sw_ptr = switchwin.as_mut_ptr().cast::<c_void>();
        let mut winchanged = false;

        let arg1 = argvar(argvars, 1);
        let arg2 = argvar(argvars, 2);

        if (*arg1.cast::<TypvalT>()).v_type != VAR_UNKNOWN
            && (*arg2.cast::<TypvalT>()).v_type != VAR_UNKNOWN
        {
            // use the window specified in the third argument
            let mut tp: *mut c_void = std::ptr::null_mut();
            let wp = win_id2wp_tp(tv_get_number(arg2) as c_int, &raw mut tp);
            if wp.is_null() || tp.is_null() {
                // goto theend
                virtcol_theend(argvars, rettv, vcol_start, vcol_end, winchanged, sw_ptr);
                return;
            }

            if nvim_option_switch_win_noblock(sw_ptr, wp, tp) != OK {
                // goto theend
                virtcol_theend(argvars, rettv, vcol_start, vcol_end, winchanged, sw_ptr);
                return;
            }

            check_cursor(nvim_eval_curwin());
            winchanged = true;
        }

        let cur_fnum = nvim_eval_curbuf_fnum();
        let mut fnum = cur_fnum;
        let arg0 = argvar(argvars, 0);
        let fp = var2fpos(arg0, false, &raw mut fnum, false);

        if !fp.is_null() && (*fp).lnum <= nvim_eval_curbuf_line_count() && fnum == cur_fnum {
            // Limit column to a valid value
            if (*fp).col < 0 {
                (*fp).col = 0;
            } else {
                let len = ml_get_len((*fp).lnum);
                if (*fp).col > len {
                    (*fp).col = len;
                }
            }
            getvvcol(
                nvim_eval_curwin(),
                fp,
                &raw mut vcol_start,
                std::ptr::null_mut(),
                &raw mut vcol_end,
            );
            vcol_start += 1;
            vcol_end += 1;
        }

        virtcol_theend(argvars, rettv, vcol_start, vcol_end, winchanged, sw_ptr);
    }

    /// Helper for the `theend:` label in `f_virtcol` — set return value and restore window.
    #[inline]
    unsafe fn virtcol_theend(
        argvars: *const c_void,
        rettv: *mut c_void,
        vcol_start: c_int,
        vcol_end: c_int,
        winchanged: bool,
        sw_ptr: *mut c_void,
    ) {
        let arg1 = argvar(argvars, 1);
        if (*arg1.cast::<TypvalT>()).v_type != VAR_UNKNOWN && tv_get_bool(arg1) != 0 {
            // list form: [start, end]
            let retlist = tv_list_alloc_ret(rettv, 2);
            tv_list_append_number(retlist, i64::from(vcol_start));
            tv_list_append_number(retlist, i64::from(vcol_end));
        } else {
            (*rettv.cast::<TypvalT>()).vval.v_number = i64::from(vcol_end);
        }

        if winchanged {
            nvim_option_restore_win_noblock(sw_ptr);
        }
    }
}

pub use window_funcs::{rs_f_line, rs_f_virtcol};
