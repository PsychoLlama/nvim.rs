//! Option value processing functions
//!
//! This module provides Rust implementations for option value construction,
//! conversion, and bounds checking. These functions are used during :set
//! command processing to create and validate new option values.

use std::ffi::{c_char, c_int};
use std::ptr;

use crate::OptInt;

// =============================================================================
// Type Definitions
// =============================================================================

/// Option value type enum (matches kOptValType* in C)
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OptValType {
    Nil = -1,
    Boolean = 0,
    Number = 1,
    String = 2,
}

/// Bounds check result
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct BoundsCheckResult {
    /// Adjusted value (may be clamped to bounds)
    pub value: OptInt,
    /// Error message (NULL if no error)
    pub errmsg: *const c_char,
    /// Whether value was modified
    pub modified: c_int,
}

impl Default for BoundsCheckResult {
    fn default() -> Self {
        Self {
            value: 0,
            errmsg: ptr::null(),
            modified: 0,
        }
    }
}

// =============================================================================
// Error Messages
// =============================================================================

static E_POSITIVE: &[u8] = b"E487: Argument must be positive\0";
static E_INVARG: &[u8] = b"E474: Invalid argument\0";
static E_SCROLL: &[u8] = b"E49: Invalid scroll size\0";
static E_WINHEIGHT: &[u8] = b"E591: 'winheight' cannot be smaller than 'winminheight'\0";
static E_WINWIDTH: &[u8] = b"E592: 'winwidth' cannot be smaller than 'winminwidth'\0";

// =============================================================================
// C External Functions
// =============================================================================

extern "C" {
    // Global state variables
    static mut full_screen: bool;
    static mut Rows: c_int;

    // Window accessor for view height
    fn nvim_option_win_get_view_height(win: *mut std::ffi::c_void) -> c_int;

    // Current window (note: *const to match setcmd.rs declaration; cast to *mut for Rust APIs)
    fn nvim_get_curwin() -> *const std::ffi::c_void;

    // Min rows for all tabpages
    #[link_name = "rs_min_rows_for_all_tabpages"]
    fn min_rows_for_all_tabpages() -> c_int;

    // Window default scroll
    #[link_name = "rs_win_default_scroll"]
    fn win_default_scroll(win: *mut std::ffi::c_void) -> OptInt;
}

// =============================================================================
// Boolean Value Processing
// =============================================================================

/// Compute new boolean value based on prefix and current value.
///
/// Handles:
/// - `:set opt` -> true
/// - `:set noopt` -> false
/// - `:set invopt` -> toggle
/// - `:set opt!` -> toggle (nextchar == '!')
#[no_mangle]
pub extern "C" fn rs_compute_boolean_newval(
    current: c_int,
    prefix: c_int,
    nextchar: c_int,
) -> c_int {
    // `:set opt!` - invert
    if nextchar == i32::from(b'!') {
        return i32::from(current == 0);
    }

    // Handle prefix
    match prefix {
        0 => 0,                       // PREFIX_NO -> false
        2 => i32::from(current == 0), // PREFIX_INV -> toggle
        _ => 1,                       // PREFIX_NONE -> true
    }
}

/// Compute new TriState value for tri-state boolean options.
#[no_mangle]
pub extern "C" fn rs_compute_tristate_newval(
    current: c_int,
    prefix: c_int,
    nextchar: c_int,
) -> c_int {
    // `:set opt!` - invert (kNone stays kNone)
    if nextchar == i32::from(b'!') {
        return match current {
            0 => 0, // kNone -> kNone (stays unchanged)
            1 => 2, // kFalse -> kTrue
            2 => 1, // kTrue -> kFalse
            _ => current,
        };
    }

    // Handle prefix
    match prefix {
        0 => 1, // PREFIX_NO -> kFalse
        2 => {
            if current == 2 {
                1
            } else {
                2
            }
        } // PREFIX_INV -> toggle
        _ => 2, // PREFIX_NONE -> kTrue
    }
}

// =============================================================================
// Number Value Processing
// =============================================================================

/// Apply numeric operator to compute new value.
///
/// Operators:
/// - OP_NONE (0): newval = operand
/// - OP_ADDING (1): newval = oldval + operand
/// - OP_PREPENDING (2): newval = oldval * operand
/// - OP_REMOVING (3): newval = oldval - operand
#[no_mangle]
pub extern "C" fn rs_apply_number_op(oldval: OptInt, operand: OptInt, op: c_int) -> OptInt {
    match op {
        1 => oldval + operand, // OP_ADDING
        2 => oldval * operand, // OP_PREPENDING
        3 => oldval - operand, // OP_REMOVING
        _ => operand,          // OP_NONE
    }
}

// =============================================================================
// Number Bounds Checking
// =============================================================================

/// Constants for number validation
const MIN_COLUMNS: OptInt = 12;
const INT_MAX: OptInt = i32::MAX as OptInt;

/// Check and bound 'lines' option value.
#[no_mangle]
pub unsafe extern "C" fn rs_check_lines_bounds(
    value: OptInt,
    errbuf: *mut c_char,
    errbuflen: usize,
) -> BoundsCheckResult {
    let mut result = BoundsCheckResult {
        value,
        errmsg: ptr::null(),
        modified: 0,
    };

    let min_rows = OptInt::from(min_rows_for_all_tabpages());

    if value < min_rows && full_screen {
        // Format error message
        if !errbuf.is_null() && errbuflen > 0 {
            let msg = format!("E593: Need at least {min_rows} lines\0");
            let bytes = msg.as_bytes();
            let copy_len = bytes.len().min(errbuflen - 1);
            ptr::copy_nonoverlapping(bytes.as_ptr(), errbuf.cast::<u8>(), copy_len);
            *errbuf.add(copy_len) = 0;
            result.errmsg = errbuf;
        }
        result.value = min_rows;
        result.modified = 1;
    }

    // Clamp to INT_MAX
    if result.value > INT_MAX {
        result.value = INT_MAX;
        result.modified = 1;
    }

    result
}

/// Check and bound 'columns' option value.
#[no_mangle]
pub unsafe extern "C" fn rs_check_columns_bounds(
    value: OptInt,
    errbuf: *mut c_char,
    errbuflen: usize,
) -> BoundsCheckResult {
    let mut result = BoundsCheckResult {
        value,
        errmsg: ptr::null(),
        modified: 0,
    };

    if value < MIN_COLUMNS && full_screen {
        // Format error message
        if !errbuf.is_null() && errbuflen > 0 {
            let msg = format!("E594: Need at least {MIN_COLUMNS} columns\0");
            let bytes = msg.as_bytes();
            let copy_len = bytes.len().min(errbuflen - 1);
            ptr::copy_nonoverlapping(bytes.as_ptr(), errbuf.cast::<u8>(), copy_len);
            *errbuf.add(copy_len) = 0;
            result.errmsg = errbuf;
        }
        result.value = MIN_COLUMNS;
        result.modified = 1;
    }

    // Clamp to INT_MAX
    if result.value > INT_MAX {
        result.value = INT_MAX;
        result.modified = 1;
    }

    result
}

/// Check and bound 'pumblend' option value (0-100).
#[no_mangle]
pub extern "C" fn rs_check_pumblend_bounds(value: OptInt) -> BoundsCheckResult {
    let clamped = value.clamp(0, 100);
    BoundsCheckResult {
        value: clamped,
        errmsg: ptr::null(),
        modified: i32::from(clamped != value),
    }
}

/// Check and bound 'scrolljump' option value.
#[no_mangle]
pub unsafe extern "C" fn rs_check_scrolljump_bounds(value: OptInt) -> BoundsCheckResult {
    let mut result = BoundsCheckResult {
        value,
        errmsg: ptr::null(),
        modified: 0,
    };

    let rows = OptInt::from(Rows);

    if full_screen && (value < -100 || value >= rows) {
        result.errmsg = E_SCROLL.as_ptr().cast();
        result.value = 1;
        result.modified = 1;
    }

    result
}

/// Check and bound 'scroll' option value.
#[no_mangle]
pub unsafe extern "C" fn rs_check_scroll_bounds(
    value: OptInt,
    win: *mut std::ffi::c_void,
) -> BoundsCheckResult {
    let mut result = BoundsCheckResult {
        value,
        errmsg: ptr::null(),
        modified: 0,
    };

    let view_height = if win.is_null() {
        0
    } else {
        OptInt::from(nvim_option_win_get_view_height(win))
    };

    if full_screen && (value <= 0 || (value > view_height && view_height > 0)) {
        if value != 0 {
            result.errmsg = E_SCROLL.as_ptr().cast();
        }
        result.value = if win.is_null() {
            value
        } else {
            win_default_scroll(win)
        };
        result.modified = 1;
    }

    result
}

// =============================================================================
// Number Validation -- Full Implementation
// =============================================================================

// Option index constants (from opt_index.rs)
use crate::opt_index::{
    K_OPT_CHANNEL, K_OPT_CHISTORY, K_OPT_CMDHEIGHT, K_OPT_CMDWINHEIGHT, K_OPT_COLUMNS,
    K_OPT_CONCEALLEVEL, K_OPT_FOLDLEVEL, K_OPT_HELPHEIGHT, K_OPT_HISTORY, K_OPT_IMINSERT,
    K_OPT_IMSEARCH, K_OPT_LHISTORY, K_OPT_LINES, K_OPT_MAXCOMBINE, K_OPT_MAXSEARCHCOUNT,
    K_OPT_NUMBERWIDTH, K_OPT_PUMBLEND, K_OPT_PYXVERSION, K_OPT_REGEXPENGINE, K_OPT_REPORT,
    K_OPT_SCROLL, K_OPT_SCROLLBACK, K_OPT_SCROLLJUMP, K_OPT_SCROLLOFF, K_OPT_SHIFTWIDTH,
    K_OPT_SIDESCROLL, K_OPT_SIDESCROLLOFF, K_OPT_TABSTOP, K_OPT_TEXTWIDTH, K_OPT_TIMEOUTLEN,
    K_OPT_TITLELEN, K_OPT_UPDATECOUNT, K_OPT_UPDATETIME, K_OPT_WINHEIGHT, K_OPT_WINMINHEIGHT,
    K_OPT_WINMINWIDTH, K_OPT_WINWIDTH, K_OPT_WRITEDELAY,
};

/// Constants matching C definitions
const MAX_MCO: OptInt = 6;
const B_IMODE_LAST: OptInt = 1;
const SB_MAX: OptInt = 1_000_000;
const TABSTOP_MAX: OptInt = 9999;
const MAX_NUMBERWIDTH: OptInt = 20;
const MAX_SEARCH_COUNT: OptInt = 9999;

/// Static error messages for quickfix option validation
/// (these are local to option_shim.c in C, so we define them in Rust)
static E_CANNOT_HAVE_NEGATIVE_OR_ZERO_QUICKFIX: &[u8] =
    b"E1542: Cannot have a negative or zero number of quickfix/location lists\0";
static E_CANNOT_HAVE_MORE_THAN_HUNDRED_QUICKFIX: &[u8] =
    b"E1543: Cannot have more than a hundred quickfix/location lists\0";

/// Write a formatted error message to errbuf.
/// Returns errbuf cast to *const c_char.
unsafe fn write_errbuf(errbuf: *mut c_char, errbuflen: usize, msg: &str) -> *const c_char {
    if errbuf.is_null() || errbuflen == 0 {
        return errbuf.cast();
    }
    let bytes = msg.as_bytes();
    let copy_len = bytes.len().min(errbuflen - 1);
    ptr::copy_nonoverlapping(bytes.as_ptr(), errbuf.cast::<u8>(), copy_len);
    *errbuf.add(copy_len) = 0;
    errbuf.cast()
}

/// Validate and bounds-check a numeric option value.
///
/// Full implementation of C's `validate_num_option` + `check_num_option_bounds`.
///
/// # Arguments
/// * `opt_idx` - Option index (matches kOpt* enum in C)
/// * `newval` - Pointer to the new value (may be modified to clamp/correct)
/// * `errbuf` - Buffer for formatted error messages
/// * `errbuflen` - Size of errbuf
///
/// # Returns
/// NULL on success, or pointer to error message string on failure.
#[no_mangle]
#[allow(clippy::too_many_lines)]
pub unsafe extern "C" fn rs_validate_num_option(
    opt_idx: c_int,
    newval: *mut OptInt,
    errbuf: *mut c_char,
    errbuflen: usize,
) -> *const c_char {
    if newval.is_null() {
        return ptr::null();
    }

    let value = *newval;

    // Many number options assume their value is in the signed int range.
    if !(OptInt::from(i32::MIN)..=OptInt::from(i32::MAX)).contains(&value) {
        return E_INVARG.as_ptr().cast();
    }

    // Option-specific validation switch
    match opt_idx {
        // Options requiring value >= 0
        idx if idx == K_OPT_HELPHEIGHT
            || idx == K_OPT_TITLELEN
            || idx == K_OPT_UPDATECOUNT
            || idx == K_OPT_REPORT
            || idx == K_OPT_UPDATETIME
            || idx == K_OPT_SIDESCROLL
            || idx == K_OPT_FOLDLEVEL
            || idx == K_OPT_SHIFTWIDTH
            || idx == K_OPT_TEXTWIDTH
            || idx == K_OPT_WRITEDELAY
            || idx == K_OPT_TIMEOUTLEN =>
        {
            if value < 0 {
                return E_POSITIVE.as_ptr().cast();
            }
        }

        idx if idx == K_OPT_WINHEIGHT => {
            if value < 1 {
                return E_POSITIVE.as_ptr().cast();
            } else if crate::p_wmh > value {
                return E_WINHEIGHT.as_ptr().cast();
            }
        }

        idx if idx == K_OPT_WINMINHEIGHT => {
            if value < 0 {
                return E_POSITIVE.as_ptr().cast();
            } else if value > crate::p_wh {
                return E_WINHEIGHT.as_ptr().cast();
            }
        }

        idx if idx == K_OPT_WINWIDTH => {
            if value < 1 {
                return E_POSITIVE.as_ptr().cast();
            } else if crate::p_wmw > value {
                return E_WINWIDTH.as_ptr().cast();
            }
        }

        idx if idx == K_OPT_WINMINWIDTH => {
            if value < 0 {
                return E_POSITIVE.as_ptr().cast();
            } else if value > crate::p_wiw {
                return E_WINWIDTH.as_ptr().cast();
            }
        }

        idx if idx == K_OPT_MAXCOMBINE => {
            *newval = MAX_MCO;
        }

        idx if idx == K_OPT_CMDHEIGHT => {
            if value < 0 {
                return E_POSITIVE.as_ptr().cast();
            }
        }

        idx if idx == K_OPT_HISTORY => {
            if value < 0 {
                return E_POSITIVE.as_ptr().cast();
            } else if value > 10000 {
                return E_INVARG.as_ptr().cast();
            }
        }

        idx if idx == K_OPT_PYXVERSION => {
            if value == 0 {
                *newval = 3;
            } else if value != 3 {
                return E_INVARG.as_ptr().cast();
            }
        }

        idx if idx == K_OPT_REGEXPENGINE => {
            if !(0..=2).contains(&value) {
                return E_INVARG.as_ptr().cast();
            }
        }

        idx if idx == K_OPT_SCROLLOFF => {
            if value < 0 && full_screen {
                return E_POSITIVE.as_ptr().cast();
            }
        }

        idx if idx == K_OPT_SIDESCROLLOFF => {
            if value < 0 && full_screen {
                return E_POSITIVE.as_ptr().cast();
            }
        }

        idx if idx == K_OPT_CMDWINHEIGHT => {
            if value < 1 {
                return E_POSITIVE.as_ptr().cast();
            }
        }

        idx if idx == K_OPT_CONCEALLEVEL => {
            if value < 0 {
                return E_POSITIVE.as_ptr().cast();
            } else if value > 3 {
                return E_INVARG.as_ptr().cast();
            }
        }

        idx if idx == K_OPT_NUMBERWIDTH => {
            if value < 1 {
                return E_POSITIVE.as_ptr().cast();
            } else if value > MAX_NUMBERWIDTH {
                return E_INVARG.as_ptr().cast();
            }
        }

        idx if idx == K_OPT_IMINSERT => {
            if !(0..=B_IMODE_LAST).contains(&value) {
                return E_INVARG.as_ptr().cast();
            }
        }

        idx if idx == K_OPT_IMSEARCH => {
            if !(-1..=B_IMODE_LAST).contains(&value) {
                return E_INVARG.as_ptr().cast();
            }
        }

        idx if idx == K_OPT_CHANNEL => {
            return E_INVARG.as_ptr().cast();
        }

        idx if idx == K_OPT_SCROLLBACK => {
            if !(-1..=SB_MAX).contains(&value) {
                return E_INVARG.as_ptr().cast();
            }
        }

        idx if idx == K_OPT_TABSTOP => {
            if value < 1 {
                return E_POSITIVE.as_ptr().cast();
            } else if value > TABSTOP_MAX {
                return E_INVARG.as_ptr().cast();
            }
        }

        idx if idx == K_OPT_CHISTORY || idx == K_OPT_LHISTORY => {
            if value < 1 {
                return E_CANNOT_HAVE_NEGATIVE_OR_ZERO_QUICKFIX.as_ptr().cast();
            } else if value > 100 {
                return E_CANNOT_HAVE_MORE_THAN_HUNDRED_QUICKFIX.as_ptr().cast();
            }
        }

        idx if idx == K_OPT_MAXSEARCHCOUNT => {
            if value <= 0 {
                return E_POSITIVE.as_ptr().cast();
            } else if value > MAX_SEARCH_COUNT {
                return E_INVARG.as_ptr().cast();
            }
        }

        _ => {}
    }

    // check_num_option_bounds inline

    match opt_idx {
        idx if idx == K_OPT_LINES => {
            let min_rows = OptInt::from(min_rows_for_all_tabpages());
            if *newval < min_rows && full_screen {
                let msg = format!("E593: Need at least {min_rows} lines");
                let errmsg = write_errbuf(errbuf, errbuflen, &msg);
                *newval = min_rows;
                return errmsg;
            }
            // Clamp to INT_MAX (true max size is defined by check_screensize())
            if *newval > OptInt::from(i32::MAX) {
                *newval = OptInt::from(i32::MAX);
            }
        }

        idx if idx == K_OPT_COLUMNS => {
            if *newval < MIN_COLUMNS && full_screen {
                let msg = format!("E594: Need at least {MIN_COLUMNS} columns");
                let errmsg = write_errbuf(errbuf, errbuflen, &msg);
                *newval = MIN_COLUMNS;
                return errmsg;
            }
            // Clamp to INT_MAX (true max size is defined by check_screensize())
            if *newval > OptInt::from(i32::MAX) {
                *newval = OptInt::from(i32::MAX);
            }
        }

        idx if idx == K_OPT_PUMBLEND => {
            *newval = (*newval).clamp(0, 100);
        }

        idx if idx == K_OPT_SCROLLJUMP => {
            let rows = OptInt::from(Rows);
            if (*newval < -100 || *newval >= rows) && full_screen {
                *newval = 1;
                return E_SCROLL.as_ptr().cast();
            }
        }

        idx if idx == K_OPT_SCROLL => {
            // nvim_get_curwin returns *const but we need *mut for C calls
            let win = nvim_get_curwin().cast_mut();
            let view_height = if win.is_null() {
                0
            } else {
                OptInt::from(nvim_option_win_get_view_height(win))
            };
            if (*newval <= 0 || (*newval > view_height && view_height > 0)) && full_screen {
                let errmsg = if *newval != 0 {
                    E_SCROLL.as_ptr().cast()
                } else {
                    ptr::null()
                };
                *newval = if win.is_null() {
                    *newval
                } else {
                    win_default_scroll(win)
                };
                return errmsg;
            }
        }

        _ => {}
    }

    ptr::null()
}

/// Validate 'winheight' against 'winminheight' with cross-check.
#[no_mangle]
pub unsafe extern "C" fn rs_validate_winheight_cross(value: OptInt) -> *const c_char {
    if value < 1 {
        return E_POSITIVE.as_ptr().cast();
    }

    let wmh = crate::p_wmh;
    if wmh > value {
        return E_WINHEIGHT.as_ptr().cast();
    }

    ptr::null()
}

/// Validate 'winminheight' against 'winheight' with cross-check.
#[no_mangle]
pub unsafe extern "C" fn rs_validate_winminheight_cross(value: OptInt) -> *const c_char {
    if value < 0 {
        return E_POSITIVE.as_ptr().cast();
    }

    let wh = crate::p_wh;
    if value > wh {
        return E_WINHEIGHT.as_ptr().cast();
    }

    ptr::null()
}

/// Validate 'winwidth' against 'winminwidth' with cross-check.
#[no_mangle]
pub unsafe extern "C" fn rs_validate_winwidth_cross(value: OptInt) -> *const c_char {
    if value < 1 {
        return E_POSITIVE.as_ptr().cast();
    }

    let wmw = crate::p_wmw;
    if wmw > value {
        return E_WINWIDTH.as_ptr().cast();
    }

    ptr::null()
}

/// Validate 'winminwidth' against 'winwidth' with cross-check.
#[no_mangle]
pub unsafe extern "C" fn rs_validate_winminwidth_cross(value: OptInt) -> *const c_char {
    if value < 0 {
        return E_POSITIVE.as_ptr().cast();
    }

    let wiw = crate::p_wiw;
    if value > wiw {
        return E_WINWIDTH.as_ptr().cast();
    }

    ptr::null()
}

/// Validate 'history' option value with bounds.
#[no_mangle]
pub extern "C" fn rs_validate_history_bounds(value: OptInt) -> *const c_char {
    if value < 0 {
        E_POSITIVE.as_ptr().cast()
    } else if value > 10000 {
        E_INVARG.as_ptr().cast()
    } else {
        ptr::null()
    }
}

/// Validate 'regexpengine' option value with bounds.
#[no_mangle]
pub extern "C" fn rs_validate_regexpengine_bounds(value: OptInt) -> *const c_char {
    if (0..=2).contains(&value) {
        ptr::null()
    } else {
        E_INVARG.as_ptr().cast()
    }
}

// =============================================================================
// String Option Helpers
// =============================================================================

/// Check if a string option value is empty.
#[no_mangle]
pub unsafe extern "C" fn rs_string_option_is_empty(val: *const c_char) -> c_int {
    i32::from(val.is_null() || *val == 0)
}

/// Check string against empty option value.
/// Used by check_string_option to replace NULL with empty string.
#[no_mangle]
pub unsafe extern "C" fn rs_check_string_option(valp: *mut *mut c_char, empty: *mut c_char) {
    if valp.is_null() {
        return;
    }

    if (*valp).is_null() {
        *valp = empty;
    }
}

// =============================================================================
// Option Reset Helpers
// =============================================================================

/// Check if nextchar indicates reset to default ('&').
#[no_mangle]
pub extern "C" fn rs_is_reset_to_default(nextchar: c_int) -> c_int {
    i32::from(nextchar == i32::from(b'&'))
}

/// Check if nextchar indicates reset to global ('<').
#[no_mangle]
pub extern "C" fn rs_is_reset_to_global(nextchar: c_int) -> c_int {
    i32::from(nextchar == i32::from(b'<'))
}

/// Check if nextchar indicates invert ('!').
#[no_mangle]
pub extern "C" fn rs_is_invert(nextchar: c_int) -> c_int {
    i32::from(nextchar == i32::from(b'!'))
}

// =============================================================================
// Phase 9 Value Manipulation Helpers
// =============================================================================

use crate::storage::{OptVal, OptValData, String_};
// Use StorageOptValType as an alias to distinguish from the local OptValType enum
use crate::OptValType as StorageOptValType;

// C external functions used by Phase 9 helpers
extern "C" {
    fn nvim_varp_is_curbuf_b_changed(varp: *const std::ffi::c_void) -> c_int;
    fn nvim_curbufIsChanged() -> c_int;
    fn nvim_get_option_type(opt_idx: c_int) -> c_int;
    fn rs_optval_free(o: OptVal);
    fn rs_optval_equal(o1: OptVal, o2: OptVal) -> c_int;
    fn rs_option_is_global_local(opt_idx: c_int) -> c_int;
    fn rs_get_option_unset_value(opt_idx: c_int) -> OptVal;
    fn nvim_get_varp_scope_by_idx(opt_idx: c_int, opt_flags: c_int) -> *mut std::ffi::c_void;
    fn nvim_get_option_ptr_by_idx(opt_idx: c_int) -> *mut std::ffi::c_void;
    fn rs_optval_copy(o: OptVal) -> OptVal;
    #[link_name = "is_option_hidden"]
    fn rs_option_is_hidden(opt_idx: c_int) -> c_int;
    fn rs_get_option_flags(opt_idx: c_int) -> u32;
    fn nvim_get_sandbox() -> c_int;
    fn nvim_get_e_sandbox() -> *const c_char;
    static e_unknown_option2: c_char;
    fn emsg(msg: *const c_char) -> c_int;
    fn gettext(s: *const c_char) -> *const c_char;
    fn rs_is_tty_option(name: *const c_char) -> c_int;
    fn rs_set_option_impl(
        opt_idx: c_int,
        value: OptVal,
        opt_flags: c_int,
        set_sid: c_int,
        direct: c_int,
        value_replaced: c_int,
        errbuf: *mut c_char,
        errbuflen: usize,
    ) -> *const c_char;
    static mut curbuf: crate::BufHandle;
    static mut curwin: crate::WinHandle;
    fn nvim_win_get_w_buffer(win: *const std::ffi::c_void) -> *mut std::ffi::c_void;
    fn xmalloc(size: usize) -> *mut c_char;
    fn xstrdup(s: *const c_char) -> *mut c_char;
    fn snprintf(buf: *mut c_char, size: usize, fmt: *const c_char, ...) -> c_int;
}

/// TriState values matching C definitions
const K_NONE: c_int = -1;
const K_FALSE: c_int = 0;
const K_TRUE: c_int = 1;

/// OPT_LOCAL flag (matches option.h)
const OPT_LOCAL: c_int = 0x02;

/// kOptValType* constants (must match C enum)
const K_OPT_VAL_TYPE_NIL: c_int = -1;
const K_OPT_VAL_TYPE_BOOLEAN: c_int = 0;
const K_OPT_VAL_TYPE_NUMBER: c_int = 1;

/// NUMBUFLEN constant matching C definition
const NUMBUFLEN: usize = 65;

/// Rust implementation of optval_from_varp.
///
/// Creates an OptVal from a variable pointer based on option type.
/// Handles the special case for 'modified' (b_changed).
#[no_mangle]
pub unsafe extern "C" fn rs_optval_from_varp(
    opt_idx: c_int,
    varp: *mut std::ffi::c_void,
) -> OptVal {
    // Special case: 'modified' is b_changed, but we also want to consider it set
    // when 'ff' or 'fenc' changed.
    if nvim_varp_is_curbuf_b_changed(varp) != 0 {
        let changed = nvim_curbufIsChanged();
        // Returns BOOLEAN_OPTVAL(curbufIsChanged()) where curbufIsChanged() is bool -> kTrue/kFalse
        return OptVal {
            type_: StorageOptValType::Boolean,
            data: OptValData {
                boolean: if changed != 0 { K_TRUE } else { K_FALSE },
            },
        };
    }

    let type_ = nvim_get_option_type(opt_idx);

    if type_ == K_OPT_VAL_TYPE_NIL {
        OptVal {
            type_: StorageOptValType::Nil,
            data: OptValData { number: 0 },
        }
    } else if type_ == K_OPT_VAL_TYPE_BOOLEAN {
        // Read int* and convert to TriState (kNone=-1, kFalse=0, kTrue=1)
        let raw = *varp.cast::<c_int>();
        let tristate = if raw == 0 {
            K_FALSE
        } else if raw >= 1 {
            K_TRUE
        } else {
            K_NONE
        };
        OptVal {
            type_: StorageOptValType::Boolean,
            data: OptValData { boolean: tristate },
        }
    } else if type_ == K_OPT_VAL_TYPE_NUMBER {
        let num = *varp.cast::<OptInt>();
        OptVal {
            type_: StorageOptValType::Number,
            data: OptValData { number: num },
        }
    } else {
        // String: read char** and build String_ with cstr_as_string semantics (no copy, just pointer)
        let data_ptr = *varp.cast::<*mut c_char>();
        let size = if data_ptr.is_null() {
            0
        } else {
            libc_strlen(data_ptr)
        };
        OptVal {
            type_: StorageOptValType::String,
            data: OptValData {
                string: String_ {
                    data: data_ptr,
                    size,
                },
            },
        }
    }
}

/// Measure the length of a C string without libc dependency.
/// Replicates strlen for use in no_std contexts.
unsafe fn libc_strlen(s: *const c_char) -> usize {
    let mut len = 0usize;
    let mut p = s;
    while *p != 0 {
        len += 1;
        p = p.add(1);
    }
    len
}

/// Rust implementation of set_option_varp.
///
/// Writes an OptVal into a variable pointer, optionally freeing the old value.
#[no_mangle]
pub unsafe extern "C" fn rs_set_option_varp(
    opt_idx: c_int,
    varp: *mut std::ffi::c_void,
    value: OptVal,
    free_oldval: c_int,
) {
    if free_oldval != 0 {
        let old = rs_optval_from_varp(opt_idx, varp);
        rs_optval_free(old);
    }

    match value.type_ {
        StorageOptValType::Nil => {
            // abort() in C - should never happen
            std::process::abort();
        }
        StorageOptValType::Boolean => {
            *varp.cast::<c_int>() = value.data.boolean;
        }
        StorageOptValType::Number => {
            *varp.cast::<OptInt>() = value.data.number;
        }
        StorageOptValType::String => {
            *varp.cast::<*mut c_char>() = value.data.string.data;
        }
    }
}

/// Rust implementation of is_option_local_value_unset.
///
/// Returns 1 if the local value of a global-local option equals the unset sentinel.
/// Returns 0 for options that aren't global-local.
#[no_mangle]
pub unsafe extern "C" fn rs_is_option_local_value_unset(opt_idx: c_int) -> c_int {
    // Local value of option that isn't global-local is always considered set.
    if rs_option_is_global_local(opt_idx) == 0 {
        return 0;
    }

    let varp_local = nvim_get_varp_scope_by_idx(opt_idx, OPT_LOCAL);
    let local_value = rs_optval_from_varp(opt_idx, varp_local);
    let unset_value = rs_get_option_unset_value(opt_idx);

    rs_optval_equal(local_value, unset_value)
}

/// Rust implementation of optval_to_cstr.
///
/// Returns an allocated C string representation of an OptVal.
/// Caller must free the returned string.
#[no_mangle]
pub unsafe extern "C" fn rs_optval_to_cstr(o: OptVal) -> *mut c_char {
    match o.type_ {
        StorageOptValType::Nil => xstrdup(c"".as_ptr()),
        StorageOptValType::Boolean => {
            // In C: o.data.boolean ? "true" : "false"
            // boolean == kFalse (0) or kNone (-1) -> "false"; kTrue (1) -> "true"
            let s = if o.data.boolean == K_TRUE {
                c"true".as_ptr()
            } else {
                c"false".as_ptr()
            };
            xstrdup(s)
        }
        StorageOptValType::Number => {
            let buf = xmalloc(NUMBUFLEN);
            snprintf(buf, NUMBUFLEN, c"%lld".as_ptr(), o.data.number);
            buf
        }
        StorageOptValType::String => {
            // Format as "\"<string>\""
            let size = o.data.string.size;
            let buf = xmalloc(size + 3);
            // Write opening quote
            *buf = b'"' as c_char;
            if size > 0 && !o.data.string.data.is_null() {
                std::ptr::copy_nonoverlapping(o.data.string.data, buf.add(1), size);
            }
            *buf.add(size + 1) = b'"' as c_char;
            *buf.add(size + 2) = 0;
            buf
        }
    }
}

// =============================================================================
// Phase 15: get_option_value, get_option_ptr
// =============================================================================

/// OPT_INVALID sentinel
const OPT_INVALID: c_int = -1;

/// IOSIZE matching C definition (1024+1 = 1025)
const IOSIZE: usize = 1025;

/// OptScope constants matching C enum (kOptScope*)
#[allow(dead_code)]
const K_OPT_SCOPE_GLOBAL: c_int = 0;
const K_OPT_SCOPE_WIN: c_int = 1;
const K_OPT_SCOPE_BUF: c_int = 2;

// =============================================================================
// Phase 15: set_option_direct, set_option_direct_for
// =============================================================================

/// Rust implementation of set_option_direct.
///
/// Sets an option value directly, without processing any side effects.
#[export_name = "set_option_direct"]
pub unsafe extern "C" fn rs_set_option_direct(
    opt_idx: c_int,
    value: OptVal,
    opt_flags: c_int,
    set_sid: c_int,
) {
    if rs_option_is_hidden(opt_idx) != 0 {
        return;
    }

    let mut errbuf = [0i8; IOSIZE];
    let errmsg = rs_set_option_impl(
        opt_idx,
        rs_optval_copy(value),
        opt_flags,
        set_sid,
        1, // direct = true
        1, // value_replaced = true
        errbuf.as_mut_ptr(),
        IOSIZE,
    );
    debug_assert!(errmsg.is_null(), "set_option_direct should not fail");
    let _ = errmsg;
}

/// Rust implementation of set_option_direct_for.
///
/// Sets an option value directly for a buffer/window, without side effects.
/// Saves/restores curbuf and curwin around the call.
#[export_name = "set_option_direct_for"]
pub unsafe extern "C" fn rs_set_option_direct_for(
    opt_idx: c_int,
    value: OptVal,
    opt_flags: c_int,
    set_sid: c_int,
    scope: c_int,
    from: *mut std::ffi::c_void,
) {
    let save_curbuf = curbuf;
    let save_curwin = curwin;

    // Adjust curbuf/curwin for the target scope.
    // Don't use switch_option_context (that calls aucmd_prepbuf with side effects).
    if scope == K_OPT_SCOPE_WIN {
        curwin = from;
        let buf = nvim_win_get_w_buffer(from.cast_const());
        curbuf = buf;
    } else if scope == K_OPT_SCOPE_BUF {
        curbuf = from;
    }
    // K_OPT_SCOPE_GLOBAL: no change to curbuf/curwin

    rs_set_option_direct(opt_idx, value, opt_flags, set_sid);

    curwin = save_curwin;
    curbuf = save_curbuf;
}

// =============================================================================
// Phase 15: get_option_value, get_option_ptr
// =============================================================================

/// Rust implementation of get_option_value.
///
/// Gets the value for an option, returning NIL_OPTVAL for invalid index.
#[allow(clippy::must_use_candidate)]
#[export_name = "get_option_value"]
pub unsafe extern "C" fn rs_get_option_value(opt_idx: c_int, opt_flags: c_int) -> OptVal {
    if opt_idx == OPT_INVALID {
        return OptVal {
            type_: StorageOptValType::Nil,
            data: OptValData { number: 0 },
        };
    }
    let varp = nvim_get_varp_scope_by_idx(opt_idx, opt_flags);
    let val = rs_optval_from_varp(opt_idx, varp);
    rs_optval_copy(val)
}

/// Rust implementation of get_option (returns pointer to vimoption_T).
///
/// Returns a pointer to &options[opt_idx].
#[allow(clippy::must_use_candidate)]
#[export_name = "get_option"]
pub unsafe extern "C" fn rs_get_option_ptr(opt_idx: c_int) -> *mut std::ffi::c_void {
    nvim_get_option_ptr_by_idx(opt_idx)
}

// =============================================================================
// Phase 15: set_option_value, unset_option_local_value, set_option_value_handle_tty,
//           set_option_value_give_err
// =============================================================================

/// kOptFlagSecure flag (matches option_defs.h kOptFlagSecure)
const K_OPT_FLAG_SECURE: u32 = 1 << 14;

/// Thread-local static error buffer for set_option_value_handle_tty.
/// Using a Cell<[u8; IOSIZE]> so we can write to it in unsafe code.
use std::cell::UnsafeCell;
thread_local! {
    static TTY_ERRBUF: UnsafeCell<[u8; IOSIZE]> = const { UnsafeCell::new([0u8; IOSIZE]) };
}

/// Rust implementation of set_option_value.
///
/// Sets the value of an option. Checks sandbox/secure flags before setting.
///
/// Returns NULL on success, untranslated error message on error.
#[allow(clippy::must_use_candidate)]
#[export_name = "set_option_value"]
pub unsafe extern "C" fn rs_set_option_value(
    opt_idx: c_int,
    value: OptVal,
    opt_flags: c_int,
) -> *const c_char {
    let flags = rs_get_option_flags(opt_idx);

    // Disallow changing some options in the sandbox
    if nvim_get_sandbox() > 0 && (flags & K_OPT_FLAG_SECURE) != 0 {
        return nvim_get_e_sandbox();
    }

    let mut errbuf = [0i8; IOSIZE];
    rs_set_option_impl(
        opt_idx,
        rs_optval_copy(value),
        opt_flags,
        0, // set_sid = 0 (use current)
        0, // direct = false
        1, // value_replaced = true
        errbuf.as_mut_ptr(),
        IOSIZE,
    )
}

/// Rust implementation of unset_option_local_value.
///
/// Unsets the local value of a global-local option.
///
/// Returns NULL on success, untranslated error message on error.
#[no_mangle]
pub unsafe extern "C" fn rs_unset_option_local_value(opt_idx: c_int) -> *const c_char {
    let unset_val = rs_get_option_unset_value(opt_idx);
    rs_set_option_value(opt_idx, unset_val, OPT_LOCAL)
}

/// Rust implementation of set_option_value_handle_tty.
///
/// Like set_option_value but also handles TTY options.
/// If opt_idx is OPT_INVALID, checks if it's a TTY option (silently succeeds)
/// or formats an unknown-option error message into a static buffer.
///
/// Returns NULL on success, error message pointer on error.
#[allow(clippy::must_use_candidate)]
#[export_name = "set_option_value_handle_tty"]
pub unsafe extern "C" fn rs_set_option_value_handle_tty(
    name: *const c_char,
    opt_idx: c_int,
    value: OptVal,
    opt_flags: c_int,
) -> *const c_char {
    if opt_idx == OPT_INVALID {
        if rs_is_tty_option(name) != 0 {
            return std::ptr::null(); // Fail silently; many old vimrcs set t_xx options.
        }
        // Format error message into thread-local buffer
        return TTY_ERRBUF.with(|cell| {
            let buf = cell.get().cast::<c_char>();
            let fmt = std::ptr::addr_of!(e_unknown_option2).cast::<c_char>();
            snprintf(buf, IOSIZE, fmt, name);
            buf.cast_const()
        });
    }

    rs_set_option_value(opt_idx, value, opt_flags)
}

/// Rust implementation of set_option_value_give_err.
///
/// Calls set_option_value and reports any error via emsg.
#[export_name = "set_option_value_give_err"]
pub unsafe extern "C" fn rs_set_option_value_give_err(
    opt_idx: c_int,
    value: OptVal,
    opt_flags: c_int,
) {
    let errmsg = rs_set_option_value(opt_idx, value, opt_flags);
    if !errmsg.is_null() {
        let _ = emsg(gettext(errmsg));
    }
}

// =============================================================================
// OptVal <-> Object conversions
// =============================================================================

/// Minimal Object representation matching C layout (api/private/defs.h).
///
/// We define this inline to avoid depending on the api crate from option.
#[repr(C)]
#[derive(Clone, Copy)]
pub union ObjectData {
    pub boolean: bool,
    pub integer: i64,
    pub string: String_,
    // Other variants (float, array, dict, luaref) occupy the same space.
    // We only need boolean, integer, and string for OptVal conversions.
    pub _padding: [u8; 16], // ensure union is large enough
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct Object {
    pub obj_type: c_int,
    pub data: ObjectData,
}

// kObjectType* constants (must match C enum in api/private/defs.h)
const K_OBJECT_TYPE_NIL: c_int = 0;
const K_OBJECT_TYPE_BOOLEAN: c_int = 1;
const K_OBJECT_TYPE_INTEGER: c_int = 2;
const K_OBJECT_TYPE_STRING: c_int = 4;

/// Convert an OptVal to an API Object.
///
/// Mirrors `optval_as_object` in option_shim.c.
#[no_mangle]
pub unsafe extern "C" fn rs_optval_as_object(o: OptVal) -> Object {
    match o.type_ {
        StorageOptValType::Nil => Object {
            obj_type: K_OBJECT_TYPE_NIL,
            data: ObjectData { integer: 0 },
        },
        StorageOptValType::Boolean => {
            let val = o.data.boolean;
            if val == K_NONE {
                // kNone -> NIL
                Object {
                    obj_type: K_OBJECT_TYPE_NIL,
                    data: ObjectData { integer: 0 },
                }
            } else {
                Object {
                    obj_type: K_OBJECT_TYPE_BOOLEAN,
                    data: ObjectData {
                        boolean: val != K_FALSE,
                    },
                }
            }
        }
        StorageOptValType::Number => Object {
            obj_type: K_OBJECT_TYPE_INTEGER,
            data: ObjectData {
                integer: o.data.number,
            },
        },
        StorageOptValType::String => Object {
            obj_type: K_OBJECT_TYPE_STRING,
            data: ObjectData {
                string: o.data.string,
            },
        },
    }
}

/// Convert an API Object to an OptVal.
///
/// Mirrors `object_as_optval` in option_shim.c.
///
/// # Safety
/// `error` must be a valid pointer to a bool.
#[no_mangle]
pub unsafe extern "C" fn rs_object_as_optval(o: Object, error: *mut bool) -> OptVal {
    match o.obj_type {
        K_OBJECT_TYPE_NIL => OptVal::nil(),
        K_OBJECT_TYPE_BOOLEAN => OptVal::boolean(if o.data.boolean { K_TRUE } else { K_FALSE }),
        K_OBJECT_TYPE_INTEGER => OptVal::number(o.data.integer),
        K_OBJECT_TYPE_STRING => OptVal {
            type_: StorageOptValType::String,
            data: OptValData {
                string: o.data.string,
            },
        },
        _ => {
            *error = true;
            OptVal::nil()
        }
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compute_boolean_newval() {
        // PREFIX_NONE (1) -> true
        assert_eq!(rs_compute_boolean_newval(0, 1, 0), 1);
        assert_eq!(rs_compute_boolean_newval(1, 1, 0), 1);

        // PREFIX_NO (0) -> false
        assert_eq!(rs_compute_boolean_newval(0, 0, 0), 0);
        assert_eq!(rs_compute_boolean_newval(1, 0, 0), 0);

        // PREFIX_INV (2) -> toggle
        assert_eq!(rs_compute_boolean_newval(0, 2, 0), 1);
        assert_eq!(rs_compute_boolean_newval(1, 2, 0), 0);

        // '!' -> toggle
        assert_eq!(rs_compute_boolean_newval(0, 1, i32::from(b'!')), 1);
        assert_eq!(rs_compute_boolean_newval(1, 1, i32::from(b'!')), 0);
    }

    #[test]
    fn test_apply_number_op() {
        // OP_NONE
        assert_eq!(rs_apply_number_op(10, 5, 0), 5);

        // OP_ADDING
        assert_eq!(rs_apply_number_op(10, 5, 1), 15);

        // OP_PREPENDING (multiply)
        assert_eq!(rs_apply_number_op(10, 5, 2), 50);

        // OP_REMOVING
        assert_eq!(rs_apply_number_op(10, 5, 3), 5);
    }

    #[test]
    fn test_check_pumblend_bounds() {
        let result = rs_check_pumblend_bounds(50);
        assert_eq!(result.value, 50);
        assert_eq!(result.modified, 0);

        let result = rs_check_pumblend_bounds(-10);
        assert_eq!(result.value, 0);
        assert_eq!(result.modified, 1);

        let result = rs_check_pumblend_bounds(150);
        assert_eq!(result.value, 100);
        assert_eq!(result.modified, 1);
    }

    #[test]
    fn test_validate_history_bounds() {
        assert!(rs_validate_history_bounds(0).is_null());
        assert!(rs_validate_history_bounds(500).is_null());
        assert!(rs_validate_history_bounds(10000).is_null());
        assert!(!rs_validate_history_bounds(-1).is_null());
        assert!(!rs_validate_history_bounds(10001).is_null());
    }

    #[test]
    fn test_validate_regexpengine_bounds() {
        assert!(rs_validate_regexpengine_bounds(0).is_null());
        assert!(rs_validate_regexpengine_bounds(1).is_null());
        assert!(rs_validate_regexpengine_bounds(2).is_null());
        assert!(!rs_validate_regexpengine_bounds(-1).is_null());
        assert!(!rs_validate_regexpengine_bounds(3).is_null());
    }

    #[test]
    fn test_nextchar_helpers() {
        assert_eq!(rs_is_reset_to_default(i32::from(b'&')), 1);
        assert_eq!(rs_is_reset_to_default(i32::from(b'=')), 0);

        assert_eq!(rs_is_reset_to_global(i32::from(b'<')), 1);
        assert_eq!(rs_is_reset_to_global(i32::from(b'=')), 0);

        assert_eq!(rs_is_invert(i32::from(b'!')), 1);
        assert_eq!(rs_is_invert(i32::from(b'=')), 0);
    }
}
