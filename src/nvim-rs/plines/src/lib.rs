//! Physical line calculations and display sizing for Neovim
//!
//! This crate provides Rust implementations of display calculation functions
//! from `src/nvim/drawscreen.c` and `src/nvim/plines.c`. It uses an opaque
//! handle pattern where `win_T*` pointers are treated as opaque handles,
//! with field access done through C accessor functions.

#![allow(unsafe_code)] // FFI requires unsafe
#![allow(clippy::doc_markdown)]
#![allow(clippy::cast_possible_wrap)] // Character literals are safe ASCII values
#![allow(clippy::cast_sign_loss)] // We know the values are non-negative
#![allow(clippy::cast_lossless)] // Character literals fit in c_int
#![allow(clippy::cast_possible_truncation)] // OptInt values fit in c_int for these options
#![allow(clippy::similar_names)] // p_nu and p_rnu are standard Vim option names

use std::ffi::{c_char, c_int, c_void};

use nvim_buffer::BufHandle;
use nvim_mark::PosT;
use nvim_window::WinHandle;

/// Opaque handle to a CharsizeArg structure.
#[repr(transparent)]
#[derive(Debug, Clone, Copy)]
pub struct CharsizeArgHandle(*mut c_void);

impl CharsizeArgHandle {
    const fn is_null(self) -> bool {
        self.0.is_null()
    }
}

// C accessor functions
extern "C" {
    // Window display properties
    fn nvim_win_get_view_width(wp: WinHandle) -> c_int;
    fn nvim_win_get_view_height(wp: WinHandle) -> c_int;
    fn nvim_win_fdccol_count(wp: WinHandle) -> c_int;
    fn nvim_win_is_curwin(wp: WinHandle) -> c_int;

    // Window options
    fn nvim_win_get_p_rnu(wp: WinHandle) -> c_int;
    fn nvim_win_get_p_nu(wp: WinHandle) -> c_int;
    fn nvim_win_get_p_nuw(wp: WinHandle) -> i64;
    fn nvim_win_get_p_stc(wp: WinHandle) -> *const c_char;
    fn nvim_win_get_p_cocu(wp: WinHandle) -> *const c_char;
    fn nvim_win_get_minscwidth(wp: WinHandle) -> c_int;
    fn nvim_win_get_p_wrap(wp: WinHandle) -> c_int;

    // Window cache fields
    fn nvim_win_get_nrwidth_line_count(wp: WinHandle) -> i64;
    fn nvim_win_set_nrwidth_line_count(wp: WinHandle, val: i64);
    fn nvim_win_get_nrwidth_width(wp: WinHandle) -> c_int;
    fn nvim_win_set_nrwidth_width(wp: WinHandle, val: c_int);
    fn nvim_win_set_statuscol_line_count(wp: WinHandle, val: i64);

    // Buffer properties
    fn nvim_win_buf_line_count(wp: WinHandle) -> i64;
    fn nvim_win_buf_meta_total_signtext(wp: WinHandle) -> c_int;

    // Global state
    fn nvim_get_p_wmw() -> i64;
    fn nvim_get_State() -> c_int;
    fn nvim_get_real_state() -> c_int;

    // String utilities
    fn nvim_vim_strchr(s: *const c_char, c: c_int) -> *const c_char;

    // Buffer properties for charsize
    fn nvim_buf_get_p_ts(buf: BufHandle) -> i64;
    fn nvim_buf_get_p_vts_array(buf: BufHandle) -> *const c_int;

    // Window buffer accessor
    fn nvim_win_get_w_buffer(wp: WinHandle) -> BufHandle;

    // Window properties for win_may_fill
    fn nvim_win_get_p_diff(wp: WinHandle) -> c_int;
    fn nvim_win_buf_meta_total_lines(wp: WinHandle) -> c_int;
    fn nvim_diffopt_filler() -> c_int;

    // Existing Rust functions we can call
    fn rs_tabstop_padding(col: c_int, ts: i64, vts: *const c_int) -> c_int;
    fn rs_ptr2cells(p: *const c_char) -> c_int;

    // Window properties for win_col_off
    fn nvim_win_is_cmdwin(wp: WinHandle) -> c_int;
    fn nvim_win_get_scwidth(wp: WinHandle) -> c_int;
    fn nvim_get_p_cpo() -> *const c_char;

    // Window properties for showbreak
    fn nvim_win_get_p_sbr(wp: WinHandle) -> *const c_char;
    fn nvim_get_p_sbr() -> *const c_char;
    fn nvim_get_empty_string_option() -> *const c_char;

    // Window properties for sms_marker_overlap
    fn nvim_win_get_p_list(wp: WinHandle) -> c_int;
    fn nvim_win_get_lcs_prec(wp: WinHandle) -> u32;
    fn nvim_win_get_lcs_tab1(wp: WinHandle) -> u32;
    fn nvim_win_get_w_p_list(wp: WinHandle) -> c_int;

    // Window properties for win_cursorline_standout
    fn nvim_win_get_p_cul(wp: WinHandle) -> c_int;
    fn nvim_win_get_p_cole(wp: WinHandle) -> i64;

    // Window properties for scrolloff
    fn nvim_win_get_p_so(wp: WinHandle) -> i64;
    fn nvim_win_get_p_siso(wp: WinHandle) -> i64;
    fn nvim_get_p_so() -> i64;
    fn nvim_get_p_siso() -> i64;

    // Terminal mode check
    fn nvim_win_buf_is_terminal(wp: WinHandle) -> c_int;

    // Global options for statusline/winbar
    fn nvim_get_p_ls() -> i64;
    fn nvim_get_p_wbr_empty() -> c_int;

    // Window pointers
    fn nvim_get_firstwin() -> WinHandle;

    // Already-migrated Rust functions
    fn rs_one_window_in_tab(win: WinHandle, tp: *const std::ffi::c_void) -> c_int;

    // Tabline-related accessors
    fn nvim_ui_has_tabline() -> c_int;
    fn nvim_get_p_stal() -> i64;
    fn nvim_first_tabpage_has_next() -> c_int;

    // Window border accessors
    fn nvim_win_get_border_adj(wp: WinHandle, idx: c_int) -> c_int;

    // CharsizeArg accessor functions
    fn nvim_csarg_get_win(csarg: CharsizeArgHandle) -> WinHandle;
    fn nvim_csarg_get_line(csarg: CharsizeArgHandle) -> *const c_char;
    fn nvim_csarg_get_virt_row(csarg: CharsizeArgHandle) -> c_int;
    fn nvim_csarg_get_use_tabstop(csarg: CharsizeArgHandle) -> c_int;
    fn nvim_csarg_get_max_head_vcol(csarg: CharsizeArgHandle) -> c_int;
    fn nvim_csarg_get_indent_width(csarg: CharsizeArgHandle) -> c_int;
    fn nvim_csarg_set_indent_width(csarg: CharsizeArgHandle, value: c_int);
    fn nvim_csarg_get_cur_text_width_left(csarg: CharsizeArgHandle) -> c_int;
    fn nvim_csarg_set_cur_text_width_left(csarg: CharsizeArgHandle, value: c_int);
    fn nvim_csarg_get_cur_text_width_right(csarg: CharsizeArgHandle) -> c_int;
    fn nvim_csarg_set_cur_text_width_right(csarg: CharsizeArgHandle, value: c_int);

    // Marktree iterator accessors
    fn nvim_csarg_itr_current_row(csarg: CharsizeArgHandle) -> c_int;
    fn nvim_csarg_itr_current_col(csarg: CharsizeArgHandle) -> c_int;
    fn nvim_csarg_itr_mark_invalid(csarg: CharsizeArgHandle) -> c_int;
    fn nvim_csarg_itr_mark_right(csarg: CharsizeArgHandle) -> c_int;
    fn nvim_csarg_itr_ns_in_win(csarg: CharsizeArgHandle) -> c_int;
    fn nvim_csarg_itr_get_virt_text_widths(
        csarg: CharsizeArgHandle,
        left_width: *mut c_int,
        right_width: *mut c_int,
    );
    fn nvim_csarg_itr_next(csarg: CharsizeArgHandle);

    // Additional accessors for charsize_regular
    fn nvim_virt_text_cursor_off(csarg: CharsizeArgHandle, on_NUL: c_int) -> c_int;
    fn nvim_vim_strsize(s: *const c_char) -> c_int;
    fn nvim_get_breakindent_win(wp: WinHandle, line: *const c_char) -> c_int;
    fn nvim_vim_isbreak(c: c_int) -> c_int;
    fn nvim_win_get_p_lbr(wp: WinHandle) -> c_int;
    fn nvim_win_get_p_bri(wp: WinHandle) -> c_int;
    fn nvim_win_get_lcs_eol(wp: WinHandle) -> c_int;

    // Character iteration accessors for linesize_regular
    fn nvim_str_char_info_init(
        line: *const c_char,
        ptr_out: *mut *const c_char,
        len_out: *mut c_int,
    ) -> i32;
    fn nvim_str_char_info_next(
        ptr_out: *mut *const c_char,
        len: c_int,
        value: i32,
        len_out: *mut c_int,
    ) -> i32;

    // Visual mode and virtual editing accessors for getvcol
    fn nvim_virtual_active(wp: WinHandle) -> c_int;
    fn nvim_get_VIsual_active() -> c_int;
    fn nvim_get_VIsual_lnum() -> c_int;
    fn nvim_get_VIsual_col() -> c_int;
    fn nvim_get_VIsual_coladd() -> c_int;
    fn nvim_get_p_sel_first() -> c_char;

    // Position comparison from mark crate
    fn rs_ltoreq(a: PosT, b: PosT) -> c_int;
}

// Mode constants (matching Neovim's state.h)
const MODE_VISUAL: c_int = 0x02;
const MODE_INSERT: c_int = 0x10;
const MODE_NORMAL: c_int = 0x01;
const MODE_CMDLINE: c_int = 0x04;
const MODE_TERMINAL: c_int = 0x1000;

// Statusline constants (matching Neovim's window.h)
const STATUS_HEIGHT: c_int = 1;

// Sign column constants (matching Neovim's optionstr.c)
const SCL_NUM: c_int = -1;

// Display constants
const SIGN_WIDTH: c_int = 2;
const CPO_NUMCOL: c_int = b'n' as c_int;

// ============================================================================
// Display Calculations
// ============================================================================

/// Compute the width of the foldcolumn.
///
/// Based on 'foldcolumn' and how much space is available for window "wp",
/// minus "col".
#[inline]
fn compute_foldcolumn_impl(wp: WinHandle, col: c_int) -> c_int {
    if wp.is_null() {
        return 0;
    }

    unsafe {
        let fdc = nvim_win_fdccol_count(wp);
        let is_curwin = nvim_win_is_curwin(wp) != 0;
        let p_wmw = nvim_get_p_wmw() as c_int;

        let wmw = if is_curwin && p_wmw == 0 { 1 } else { p_wmw };
        let view_width = nvim_win_get_view_width(wp);
        let n = view_width - (col + wmw);

        // MIN(fdc, n)
        if fdc < n {
            fdc
        } else {
            n
        }
    }
}

/// Return the width of the 'number' and 'relativenumber' column.
///
/// Caller may need to check if 'number' or 'relativenumber' is set.
/// Otherwise it depends on 'numberwidth' and the line count.
#[inline]
fn number_width_impl(wp: WinHandle) -> c_int {
    if wp.is_null() {
        return 0;
    }

    unsafe {
        let p_rnu = nvim_win_get_p_rnu(wp) != 0;
        let p_nu = nvim_win_get_p_nu(wp) != 0;

        // Determine the line count to use
        let lnum: i64 = if p_rnu && !p_nu {
            // cursor line shows "0"
            nvim_win_get_view_height(wp) as i64
        } else {
            // cursor line shows absolute line number
            nvim_win_buf_line_count(wp)
        };

        // Check cache
        let cached_line_count = nvim_win_get_nrwidth_line_count(wp);
        if lnum == cached_line_count {
            return nvim_win_get_nrwidth_width(wp);
        }
        nvim_win_set_nrwidth_line_count(wp, lnum);

        // Check for 'statuscolumn'
        let p_stc = nvim_win_get_p_stc(wp);
        if !p_stc.is_null() && *p_stc != 0 {
            nvim_win_set_statuscol_line_count(wp, 0); // make sure width is re-estimated
            let width = i32::from(p_nu || p_rnu) * (nvim_win_get_p_nuw(wp) as c_int);
            nvim_win_set_nrwidth_width(wp, width);
            return width;
        }

        // Count digits
        let mut temp_lnum = lnum;
        let mut n: c_int = 0;
        loop {
            temp_lnum /= 10;
            n += 1;
            if temp_lnum <= 0 {
                break;
            }
        }

        // 'numberwidth' gives the minimal width plus one
        let p_nuw = nvim_win_get_p_nuw(wp) as c_int;
        let nuw_minus_one = if p_nuw > 1 { p_nuw - 1 } else { 0 };
        if n < nuw_minus_one {
            n = nuw_minus_one;
        }

        // If 'signcolumn' is set to 'number' and there is a sign to display,
        // then the minimal width for the number column is 2.
        let has_signs = nvim_win_buf_meta_total_signtext(wp) != 0;
        let minscwidth = nvim_win_get_minscwidth(wp);
        if n < 2 && has_signs && minscwidth == SCL_NUM {
            n = 2;
        }

        nvim_win_set_nrwidth_width(wp, n);
        n
    }
}

/// Return true if the cursor line in window "wp" may be concealed,
/// according to the 'concealcursor' option.
#[inline]
fn conceal_cursor_line_impl(wp: WinHandle) -> bool {
    if wp.is_null() {
        return false;
    }

    unsafe {
        let p_cocu = nvim_win_get_p_cocu(wp);
        if p_cocu.is_null() || *p_cocu == 0 {
            return false;
        }

        let real_state = nvim_get_real_state();
        let state = nvim_get_State();

        let c: c_int = if (real_state & MODE_VISUAL) != 0 {
            b'v' as c_int
        } else if (state & MODE_INSERT) != 0 {
            b'i' as c_int
        } else if (state & MODE_NORMAL) != 0 {
            b'n' as c_int
        } else if (state & MODE_CMDLINE) != 0 {
            b'c' as c_int
        } else {
            return false;
        };

        !nvim_vim_strchr(p_cocu, c).is_null()
    }
}

// Tab character constant
const TAB: i32 = 0x09;

// Invalid byte display width (from mbyte.h)
const K_INVALID_BYTE_CELLS: c_int = 4;

/// CharSize struct for character size with head offset.
/// Matches C's CharSize struct.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct CharSize {
    pub width: c_int,
    pub head: c_int,
}

/// Get the number of cells taken up on the screen at given virtual column.
///
/// Handles tabs, invalid bytes, and normal characters.
#[inline]
fn charsize_nowrap_impl(
    buf: BufHandle,
    cur: *const c_char,
    use_tabstop: bool,
    vcol: c_int,
    cur_char: i32,
) -> c_int {
    if buf.is_null() {
        return 1;
    }

    unsafe {
        if cur_char == TAB && use_tabstop {
            let ts = nvim_buf_get_p_ts(buf);
            let vts = nvim_buf_get_p_vts_array(buf);
            rs_tabstop_padding(vcol, ts, vts)
        } else if cur_char < 0 {
            K_INVALID_BYTE_CELLS
        } else {
            rs_ptr2cells(cur)
        }
    }
}

/// Check if there may be filler lines anywhere in window "wp".
#[inline]
fn win_may_fill_impl(wp: WinHandle) -> bool {
    if wp.is_null() {
        return false;
    }

    unsafe {
        let p_diff = nvim_win_get_p_diff(wp) != 0;
        let diffopt_fill = nvim_diffopt_filler() != 0;
        let has_meta_lines = nvim_win_buf_meta_total_lines(wp) != 0;

        (p_diff && diffopt_fill) || has_meta_lines
    }
}

/// Return the offset for the window's first column.
///
/// Takes into account line numbers, fold column, sign column, and command-line window.
#[inline]
fn win_col_off_impl(wp: WinHandle) -> c_int {
    if wp.is_null() {
        return 0;
    }

    unsafe {
        let p_nu = nvim_win_get_p_nu(wp) != 0;
        let p_rnu = nvim_win_get_p_rnu(wp) != 0;
        let p_stc = nvim_win_get_p_stc(wp);
        let has_stc = !p_stc.is_null() && *p_stc != 0;

        // Number column contribution
        let num_col = if p_nu || p_rnu || has_stc {
            rs_number_width(wp) + c_int::from(!has_stc)
        } else {
            0
        };

        // Command-line window adds 1 column
        let cmdwin_col = c_int::from(nvim_win_is_cmdwin(wp) != 0);

        // Fold column
        let fdc = nvim_win_fdccol_count(wp);

        // Sign column
        let scwidth = nvim_win_get_scwidth(wp);

        num_col + cmdwin_col + fdc + (scwidth * SIGN_WIDTH)
    }
}

/// Return the offset for wrapped lines (second screen line onwards).
///
/// It's positive if 'number' or 'relativenumber' is on and 'n' is in 'cpoptions'.
#[inline]
fn win_col_off2_impl(wp: WinHandle) -> c_int {
    if wp.is_null() {
        return 0;
    }

    unsafe {
        let p_nu = nvim_win_get_p_nu(wp) != 0;
        let p_rnu = nvim_win_get_p_rnu(wp) != 0;
        let p_stc = nvim_win_get_p_stc(wp);
        let has_stc = !p_stc.is_null() && *p_stc != 0;

        if (p_nu || p_rnu || has_stc) && !nvim_vim_strchr(nvim_get_p_cpo(), CPO_NUMCOL).is_null() {
            rs_number_width(wp) + c_int::from(!has_stc)
        } else {
            0
        }
    }
}

/// Check that virtual column "vcol" is in the rightmost column of window "wp".
///
/// Used for determining if a double-width character wraps at the end of a line.
#[inline]
fn in_win_border_impl(wp: WinHandle, vcol: c_int) -> bool {
    if wp.is_null() {
        return false;
    }

    unsafe {
        let view_width = nvim_win_get_view_width(wp);
        if view_width == 0 {
            // there is no border
            return false;
        }

        // width of first line (after line number, etc.)
        let width1 = view_width - rs_win_col_off(wp);

        if vcol < width1 - 1 {
            return false;
        }

        if vcol == width1 - 1 {
            return true;
        }

        // width of further lines
        let width2 = width1 + rs_win_col_off2(wp);

        if width2 <= 0 {
            return false;
        }

        (vcol - width1) % width2 == width2 - 1
    }
}

/// Get the 'showbreak' value for a window.
///
/// Returns window-local showbreak if set, otherwise global showbreak.
/// Returns empty string if window showbreak is "NONE".
#[inline]
fn get_showbreak_value_impl(wp: WinHandle) -> *const c_char {
    if wp.is_null() {
        unsafe {
            return nvim_get_p_sbr();
        }
    }

    unsafe {
        let w_sbr = nvim_win_get_p_sbr(wp);

        // If window showbreak is NULL or empty, use global
        if w_sbr.is_null() || *w_sbr == 0 {
            return nvim_get_p_sbr();
        }

        // Check for "NONE" (case-sensitive)
        // "NONE" = 'N', 'O', 'N', 'E', '\0'
        if *w_sbr == b'N' as c_char
            && *w_sbr.add(1) == b'O' as c_char
            && *w_sbr.add(2) == b'N' as c_char
            && *w_sbr.add(3) == b'E' as c_char
            && *w_sbr.add(4) == 0
        {
            return nvim_get_empty_string_option();
        }

        w_sbr
    }
}

/// Calculate the smoothscroll marker overlap.
///
/// Calculates how much the 'listchars' "precedes" or 'smoothscroll' "<<<"
/// marker overlaps with buffer text for window "wp".
/// Parameter "extra2" should be the padding on the 2nd line, not the first
/// line. When "extra2" is -1 calculate the padding.
/// Returns the number of columns of overlap with buffer text, excluding the
/// extra padding on the ledge.
#[inline]
fn sms_marker_overlap_impl(wp: WinHandle, extra2: c_int) -> c_int {
    if wp.is_null() {
        return 0;
    }

    let extra2 = if extra2 == -1 {
        rs_win_col_off(wp) - rs_win_col_off2(wp)
    } else {
        extra2
    };

    // There is no marker overlap when in showbreak mode, thus no need to
    // account for it. See wlv_put_linebuf().
    unsafe {
        let sbr = get_showbreak_value_impl(wp);
        if !sbr.is_null() && *sbr != 0 {
            return 0;
        }

        // Overlap when 'list' and 'listchars' "precedes" are set is 1.
        let p_list = nvim_win_get_p_list(wp) != 0;
        let prec = nvim_win_get_lcs_prec(wp);
        if p_list && prec != 0 {
            return 1;
        }
    }

    // The marker is "<<<" which takes 3 columns, so overlap is 3 - extra2
    // but only when extra2 <= 3
    if extra2 > 3 {
        0
    } else {
        3 - extra2
    }
}

/// Whether cursorline is drawn in a special way.
///
/// If true, both old and new cursorline will need to be redrawn when moving cursor within windows.
#[inline]
fn win_cursorline_standout_impl(wp: WinHandle) -> bool {
    if wp.is_null() {
        return false;
    }

    unsafe {
        let cul = nvim_win_get_p_cul(wp) != 0;
        let is_curwin = nvim_win_is_curwin(wp) != 0;
        let cole = nvim_win_get_p_cole(wp);
        let conceal_cursor = rs_conceal_cursor_line(wp) != 0;

        cul || (is_curwin && cole > 0 && !conceal_cursor)
    }
}

/// Return the effective 'scrolloff' value for the current window.
///
/// Uses the global value when window value is negative.
/// Disallows scrolloff in terminal-mode for terminal buffers.
#[inline]
fn get_scrolloff_value_impl(wp: WinHandle) -> c_int {
    if wp.is_null() {
        return 0;
    }

    unsafe {
        let state = nvim_get_State();
        let is_terminal_buf = nvim_win_buf_is_terminal(wp) != 0;

        // Disallow scrolloff in terminal-mode for terminal buffers
        if (state & MODE_TERMINAL) != 0 && is_terminal_buf {
            return 0;
        }

        let w_so = nvim_win_get_p_so(wp);
        if w_so < 0 {
            nvim_get_p_so() as c_int
        } else {
            w_so as c_int
        }
    }
}

/// Return the effective 'sidescrolloff' value for the current window.
///
/// Uses the global value when window value is negative.
#[inline]
fn get_sidescrolloff_value_impl(wp: WinHandle) -> c_int {
    if wp.is_null() {
        return 0;
    }

    unsafe {
        let w_siso = nvim_win_get_p_siso(wp);
        if w_siso < 0 {
            nvim_get_p_siso() as c_int
        } else {
            w_siso as c_int
        }
    }
}

/// Return the number of lines used by the global statusline.
#[inline]
fn global_stl_height_impl() -> c_int {
    unsafe {
        if nvim_get_p_ls() == 3 {
            STATUS_HEIGHT
        } else {
            0
        }
    }
}

/// Return the number of lines used by default by the window bar.
#[inline]
fn global_winbar_height_impl() -> c_int {
    unsafe { c_int::from(nvim_get_p_wbr_empty() == 0) }
}

/// Return the height of the last window's statusline, or the global statusline if set.
///
/// @param morewin  pretend there are two or more windows if true.
#[inline]
fn last_stl_height_impl(morewin: bool) -> c_int {
    unsafe {
        let p_ls = nvim_get_p_ls();

        // p_ls > 1 means always show statusline
        // p_ls == 1 means show statusline when more than one window
        let show_stl = p_ls > 1
            || (p_ls == 1
                && (morewin || rs_one_window_in_tab(nvim_get_firstwin(), std::ptr::null()) == 0));

        if show_stl {
            STATUS_HEIGHT
        } else {
            0
        }
    }
}

/// Return the number of lines used by the tab page line.
#[inline]
fn tabline_height_impl() -> c_int {
    unsafe {
        // If UI provides tabline extension, don't draw our own
        if nvim_ui_has_tabline() != 0 {
            return 0;
        }

        let p_stal = nvim_get_p_stal();

        match p_stal {
            0 => 0,
            1 => {
                // Show tabline only if more than one tab
                c_int::from(nvim_first_tabpage_has_next() != 0)
            }
            _ => 1, // Always show tabline (p_stal == 2)
        }
    }
}

/// Return the height of a floating window's border (top + bottom).
#[inline]
fn win_border_height_impl(wp: WinHandle) -> c_int {
    if wp.is_null() {
        return 0;
    }
    unsafe {
        // w_border_adj indices: 0=top, 1=right, 2=bottom, 3=left
        nvim_win_get_border_adj(wp, 0) + nvim_win_get_border_adj(wp, 2)
    }
}

/// Return the width of a floating window's border (left + right).
#[inline]
fn win_border_width_impl(wp: WinHandle) -> c_int {
    if wp.is_null() {
        return 0;
    }
    unsafe {
        // w_border_adj indices: 0=top, 1=right, 2=bottom, 3=left
        nvim_win_get_border_adj(wp, 1) + nvim_win_get_border_adj(wp, 3)
    }
}

// ============================================================================
// FFI Exports
// ============================================================================

/// Compute the width of the foldcolumn.
///
/// # Safety
/// The `wp` parameter must be a valid `win_T*` pointer or null.
#[no_mangle]
pub extern "C" fn rs_compute_foldcolumn(wp: WinHandle, col: c_int) -> c_int {
    compute_foldcolumn_impl(wp, col)
}

/// Return the width of the 'number' and 'relativenumber' column.
///
/// # Safety
/// The `wp` parameter must be a valid `win_T*` pointer or null.
#[no_mangle]
pub extern "C" fn rs_number_width(wp: WinHandle) -> c_int {
    number_width_impl(wp)
}

/// Return true if the cursor line may be concealed.
///
/// # Safety
/// The `wp` parameter must be a valid `win_T*` pointer or null.
#[no_mangle]
pub extern "C" fn rs_conceal_cursor_line(wp: WinHandle) -> c_int {
    c_int::from(conceal_cursor_line_impl(wp))
}

/// Get the number of cells taken up on the screen at given virtual column.
///
/// # Safety
/// The `buf` parameter must be a valid `buf_T*` pointer or null.
/// The `cur` parameter must be a valid pointer to a character.
#[no_mangle]
pub extern "C" fn rs_charsize_nowrap(
    buf: BufHandle,
    cur: *const c_char,
    use_tabstop: c_int,
    vcol: c_int,
    cur_char: i32,
) -> c_int {
    charsize_nowrap_impl(buf, cur, use_tabstop != 0, vcol, cur_char)
}

/// Return number of display cells occupied by character at "p" in window "wp".
/// A TAB is counted as the number of cells to the next tab stop.
///
/// # Safety
/// The `wp` parameter must be a valid `win_T*` pointer.
/// The `p` parameter must be a valid pointer to a character.
#[no_mangle]
pub unsafe extern "C" fn rs_win_chartabsize(wp: WinHandle, p: *const c_char, col: c_int) -> c_int {
    if wp.is_null() || p.is_null() {
        return 1;
    }

    let buf = nvim_win_get_w_buffer(wp);
    let c = i32::from(*p as u8);

    // If the char is TAB and (not list mode OR tab1 character is set),
    // use tabstop_padding. Otherwise use ptr2cells.
    if c == TAB {
        let list = nvim_win_get_w_p_list(wp) != 0;
        let tab1 = nvim_win_get_lcs_tab1(wp);

        if !list || tab1 != 0 {
            let ts = nvim_buf_get_p_ts(buf);
            let vts = nvim_buf_get_p_vts_array(buf);
            return rs_tabstop_padding(col, ts, vts);
        }
    }

    rs_ptr2cells(p)
}

/// Check if there may be filler lines anywhere in window "wp".
///
/// # Safety
/// The `wp` parameter must be a valid `win_T*` pointer or null.
#[no_mangle]
pub extern "C" fn rs_win_may_fill(wp: WinHandle) -> c_int {
    c_int::from(win_may_fill_impl(wp))
}

/// Return the offset for the window's first column.
///
/// # Safety
/// The `wp` parameter must be a valid `win_T*` pointer or null.
#[no_mangle]
pub extern "C" fn rs_win_col_off(wp: WinHandle) -> c_int {
    win_col_off_impl(wp)
}

/// Return the offset for wrapped lines (second screen line onwards).
///
/// # Safety
/// The `wp` parameter must be a valid `win_T*` pointer or null.
#[no_mangle]
pub extern "C" fn rs_win_col_off2(wp: WinHandle) -> c_int {
    win_col_off2_impl(wp)
}

/// Check if vcol is in the rightmost column of window.
///
/// # Safety
/// The `wp` parameter must be a valid `win_T*` pointer or null.
#[no_mangle]
pub extern "C" fn rs_in_win_border(wp: WinHandle, vcol: c_int) -> c_int {
    c_int::from(in_win_border_impl(wp, vcol))
}

/// Get the 'showbreak' value for a window.
///
/// Returns window-local showbreak if set, otherwise global showbreak.
/// Returns empty string if window showbreak is "NONE".
///
/// # Safety
/// The `wp` parameter must be a valid `win_T*` pointer or null.
#[no_mangle]
pub extern "C" fn rs_get_showbreak_value(wp: WinHandle) -> *const c_char {
    get_showbreak_value_impl(wp)
}

/// Calculate the smoothscroll marker overlap.
///
/// # Safety
/// The `wp` parameter must be a valid `win_T*` pointer or null.
#[no_mangle]
pub extern "C" fn rs_sms_marker_overlap(wp: WinHandle, extra2: c_int) -> c_int {
    sms_marker_overlap_impl(wp, extra2)
}

/// Whether cursorline is drawn in a special way.
///
/// # Safety
/// The `wp` parameter must be a valid `win_T*` pointer or null.
#[no_mangle]
pub extern "C" fn rs_win_cursorline_standout(wp: WinHandle) -> c_int {
    c_int::from(win_cursorline_standout_impl(wp))
}

/// Return the effective 'scrolloff' value for the current window.
///
/// # Safety
/// The `wp` parameter must be a valid `win_T*` pointer or null.
#[no_mangle]
pub extern "C" fn rs_get_scrolloff_value(wp: WinHandle) -> c_int {
    get_scrolloff_value_impl(wp)
}

/// Return the effective 'sidescrolloff' value for the current window.
///
/// # Safety
/// The `wp` parameter must be a valid `win_T*` pointer or null.
#[no_mangle]
pub extern "C" fn rs_get_sidescrolloff_value(wp: WinHandle) -> c_int {
    get_sidescrolloff_value_impl(wp)
}

/// Return the number of lines used by the global statusline.
#[no_mangle]
pub extern "C" fn rs_global_stl_height() -> c_int {
    global_stl_height_impl()
}

/// Return the number of lines used by default by the window bar.
#[no_mangle]
pub extern "C" fn rs_global_winbar_height() -> c_int {
    global_winbar_height_impl()
}

/// Return the height of the last window's statusline, or the global statusline if set.
#[no_mangle]
pub extern "C" fn rs_last_stl_height(morewin: c_int) -> c_int {
    last_stl_height_impl(morewin != 0)
}

/// Return the number of lines used by the tab page line.
#[no_mangle]
pub extern "C" fn rs_tabline_height() -> c_int {
    tabline_height_impl()
}

/// Return the height of a floating window's border.
///
/// # Safety
/// The `wp` parameter must be a valid `win_T*` pointer or null.
#[no_mangle]
pub extern "C" fn rs_win_border_height(wp: WinHandle) -> c_int {
    win_border_height_impl(wp)
}

/// Return the width of a floating window's border.
///
/// # Safety
/// The `wp` parameter must be a valid `win_T*` pointer or null.
#[no_mangle]
pub extern "C" fn rs_win_border_width(wp: WinHandle) -> c_int {
    win_border_width_impl(wp)
}

// ============================================================================
// charsize_fast - Fast character size without virtual text
// ============================================================================

/// Like charsize_regular(), except it doesn't handle inline virtual text,
/// 'linebreak', 'breakindent' or 'showbreak'.
/// Handles normal characters, tabs and wrapping.
#[inline]
fn charsize_fast_impl(
    wp: WinHandle,
    cur: *const c_char,
    use_tabstop: bool,
    vcol: c_int,
    cur_char: i32,
) -> CharSize {
    if wp.is_null() {
        return CharSize { width: 1, head: 0 };
    }

    unsafe {
        // A tab gets expanded, depending on the current column
        if cur_char == TAB && use_tabstop {
            let buf = nvim_win_get_w_buffer(wp);
            let ts = nvim_buf_get_p_ts(buf);
            let vts = nvim_buf_get_p_vts_array(buf);
            return CharSize {
                width: rs_tabstop_padding(vcol, ts, vts),
                head: 0,
            };
        }

        let width = if cur_char < 0 {
            K_INVALID_BYTE_CELLS
        } else {
            rs_ptr2cells(cur)
        };

        // If a double-width char doesn't fit at the end of a line, it wraps to the next line,
        // and the last column displays a '>'.
        let p_wrap = nvim_win_get_p_wrap(wp) != 0;
        if width == 2 && cur_char >= 0x80 && p_wrap && in_win_border_impl(wp, vcol) {
            CharSize { width: 3, head: 1 }
        } else {
            CharSize { width, head: 0 }
        }
    }
}

/// Get the character size for fast path (no virtual text, linebreak, etc.).
///
/// Returns CharSize struct with width and head fields.
///
/// # Safety
/// The `wp` parameter must be a valid `win_T*` pointer.
/// The `cur` parameter must be a valid pointer to a character.
#[no_mangle]
pub extern "C" fn rs_charsize_fast(
    wp: WinHandle,
    cur: *const c_char,
    use_tabstop: c_int,
    vcol: c_int,
    cur_char: i32,
) -> CharSize {
    charsize_fast_impl(wp, cur, use_tabstop != 0, vcol, cur_char)
}

// ============================================================================
// linesize_fast - Fast line size calculation
// ============================================================================

/// Maximum column value (from Neovim's pos_defs.h MAXCOL)
const MAXCOL: c_int = 0x7fff_ffff;

/// Calculate the display width of a line using the fast path.
///
/// This function iterates through the line using UTF-8 aware iteration
/// and accumulates character widths using `charsize_fast_impl`.
/// It doesn't handle inline virtual text, 'linebreak', 'breakindent' or 'showbreak'.
///
/// # Arguments
/// * `wp` - Window handle
/// * `use_tabstop` - Whether to use tabstop for TAB characters
/// * `line` - Pointer to the line string
/// * `vcol_arg` - Initial virtual column
/// * `len` - Maximum length to process (in bytes)
#[inline]
fn linesize_fast_impl(
    wp: WinHandle,
    use_tabstop: bool,
    line: *const c_char,
    vcol_arg: c_int,
    len: c_int,
) -> c_int {
    if wp.is_null() || line.is_null() {
        return vcol_arg;
    }

    unsafe {
        // Create a slice from the line pointer with the given length
        // We need to be careful here - `len` may be MAXCOL which means "until NUL"
        // For safety, we'll iterate byte by byte

        let mut vcol: i64 = vcol_arg as i64;
        let mut vcol_for_charsize = vcol_arg;
        let mut offset: usize = 0;

        loop {
            // Check bounds
            let cur_ptr = line.add(offset);
            let cur_byte = *cur_ptr as u8;

            // Stop at NUL
            if cur_byte == 0 {
                break;
            }

            // Stop if we've processed enough bytes
            if offset as c_int >= len {
                break;
            }

            // Get character info using UTF-8 decoding
            // First determine the length of this UTF-8 character
            let char_len = utf8_char_len(cur_byte);

            // Decode the character value
            let chr_value = decode_utf8_char(cur_ptr, char_len);

            // Calculate character width using current vcol
            let cs = charsize_fast_impl(wp, cur_ptr, use_tabstop, vcol_for_charsize, chr_value);
            vcol += cs.width as i64;

            // Move to next character (including composing characters)
            offset += char_len;

            // Check for overflow and update vcol_for_charsize
            if vcol > MAXCOL as i64 {
                return MAXCOL;
            }
            vcol_for_charsize = vcol as c_int;
        }

        vcol_for_charsize
    }
}

/// Get the byte length of a UTF-8 character from its first byte.
#[inline]
const fn utf8_char_len(b: u8) -> usize {
    if b < 0x80 {
        1
    } else if b < 0xC0 {
        // Continuation byte - treat as 1 (invalid sequence)
        1
    } else if b < 0xE0 {
        2
    } else if b < 0xF0 {
        3
    } else if b < 0xF8 {
        4
    } else {
        // Invalid - treat as 1
        1
    }
}

/// Decode a UTF-8 character from a pointer.
///
/// Returns the codepoint value, or -1 for invalid sequences.
#[inline]
const unsafe fn decode_utf8_char(p: *const c_char, len: usize) -> i32 {
    let b0 = *p as u8;

    // ASCII fast path
    if b0 < 0x80 {
        return b0 as i32;
    }

    match len {
        2 => {
            let b1 = *p.add(1) as u8;
            if (b1 & 0xC0) != 0x80 {
                return -1;
            }
            let code = ((b0 as i32 & 0x1F) << 6) | (b1 as i32 & 0x3F);
            if code < 0x80 {
                -1 // Overlong
            } else {
                code
            }
        }
        3 => {
            let b1 = *p.add(1) as u8;
            let b2 = *p.add(2) as u8;
            if (b1 & 0xC0) != 0x80 || (b2 & 0xC0) != 0x80 {
                return -1;
            }
            let code = ((b0 as i32 & 0x0F) << 12) | ((b1 as i32 & 0x3F) << 6) | (b2 as i32 & 0x3F);
            if code < 0x800 {
                -1 // Overlong
            } else {
                code
            }
        }
        4 => {
            let b1 = *p.add(1) as u8;
            let b2 = *p.add(2) as u8;
            let b3 = *p.add(3) as u8;
            if (b1 & 0xC0) != 0x80 || (b2 & 0xC0) != 0x80 || (b3 & 0xC0) != 0x80 {
                return -1;
            }
            let code = ((b0 as i32 & 0x07) << 18)
                | ((b1 as i32 & 0x3F) << 12)
                | ((b2 as i32 & 0x3F) << 6)
                | (b3 as i32 & 0x3F);
            // Valid 4-byte UTF-8 range: U+10000 to U+10FFFF
            #[allow(clippy::manual_range_contains)]
            if code < 0x10000 || code > 0x0010_FFFF {
                -1 // Invalid or overlong
            } else {
                code
            }
        }
        _ => b0 as i32, // Treat single byte as its value
    }
}

/// Calculate the display width of a line using the fast path.
///
/// Returns the virtual column after processing the line.
///
/// # Safety
/// The `wp` parameter must be a valid `win_T*` pointer.
/// The `line` parameter must be a valid pointer to a NUL-terminated string.
#[no_mangle]
pub extern "C" fn rs_linesize_fast(
    wp: WinHandle,
    use_tabstop: c_int,
    line: *const c_char,
    vcol_arg: c_int,
    len: c_int,
) -> c_int {
    linesize_fast_impl(wp, use_tabstop != 0, line, vcol_arg, len)
}

// ============================================================================
// charsize_regular - Full character size with virtual text, linebreak, etc.
// ============================================================================

/// INT_MIN value (for indent_width sentinel)
const INT_MIN: c_int = c_int::MIN;

/// Get the number of cells taken up on the screen for the given arguments.
///
/// This handles:
/// - Normal characters, tabs, NUL
/// - Inline virtual text (via marktree iteration)
/// - Double-width character wrapping
/// - 'showbreak' and 'breakindent'
/// - 'linebreak' option
///
/// "csarg->cur_text_width_left" and "csarg->cur_text_width_right" are set
/// to the extra size for inline virtual text.
#[inline]
#[allow(clippy::too_many_lines)]
fn charsize_regular_impl(
    csarg: CharsizeArgHandle,
    cur: *const c_char,
    vcol: c_int,
    cur_char: i32,
) -> CharSize {
    if csarg.is_null() || cur.is_null() {
        return CharSize { width: 1, head: 0 };
    }

    unsafe {
        // Reset cur_text_width fields
        nvim_csarg_set_cur_text_width_left(csarg, 0);
        nvim_csarg_set_cur_text_width_right(csarg, 0);

        let wp = nvim_csarg_get_win(csarg);
        let line = nvim_csarg_get_line(csarg);
        let use_tabstop_flag = nvim_csarg_get_use_tabstop(csarg) != 0;
        let use_tabstop = cur_char == TAB && use_tabstop_flag;
        let mut mb_added: c_int = 0;

        // Check for 'list' and 'listchars' eol
        let p_list = nvim_win_get_p_list(wp) != 0;
        let lcs_eol = nvim_win_get_lcs_eol(wp);
        let has_lcs_eol = p_list && lcs_eol != 0;

        // Get buffer info for tabstop calculation
        let buf = nvim_win_get_w_buffer(wp);
        let ts = nvim_buf_get_p_ts(buf);
        let vts = nvim_buf_get_p_vts_array(buf);

        // First get normal size, without 'linebreak' or inline virtual text
        let mut size: c_int;
        let is_doublewidth: bool;

        if use_tabstop {
            size = rs_tabstop_padding(vcol, ts, vts);
            is_doublewidth = false;
        } else if *cur == 0 {
            // 1 cell for EOL list char (if present), as opposed to the two cell ^@
            // for a NUL character in the text.
            size = c_int::from(has_lcs_eol);
            is_doublewidth = false;
        } else if cur_char < 0 {
            size = K_INVALID_BYTE_CELLS;
            is_doublewidth = false;
        } else {
            size = rs_ptr2cells(cur);
            is_doublewidth = size == 2 && cur_char >= 0x80;
        }

        // Handle inline virtual text via marktree iteration
        let virt_row = nvim_csarg_get_virt_row(csarg);
        if virt_row >= 0 {
            let mut tab_size = size;
            let col = (cur as isize - line as isize) as c_int;

            loop {
                let mark_row = nvim_csarg_itr_current_row(csarg);
                let mark_col = nvim_csarg_itr_current_col(csarg);

                if mark_row != virt_row || mark_col > col {
                    break;
                } else if mark_col == col {
                    let mark_invalid = nvim_csarg_itr_mark_invalid(csarg) != 0;
                    let ns_visible = nvim_csarg_itr_ns_in_win(csarg) != 0;

                    if !mark_invalid && ns_visible {
                        let mut left_width: c_int = 0;
                        let mut right_width: c_int = 0;
                        nvim_csarg_itr_get_virt_text_widths(
                            csarg,
                            std::ptr::from_mut(&mut left_width),
                            std::ptr::from_mut(&mut right_width),
                        );

                        // Update cur_text_width fields
                        let mark_right = nvim_csarg_itr_mark_right(csarg) != 0;
                        if mark_right {
                            let cur_right = nvim_csarg_get_cur_text_width_right(csarg);
                            nvim_csarg_set_cur_text_width_right(csarg, cur_right + right_width);
                        } else {
                            let cur_left = nvim_csarg_get_cur_text_width_left(csarg);
                            nvim_csarg_set_cur_text_width_left(csarg, cur_left + left_width);
                        }

                        let total_width = left_width + right_width;
                        size += total_width;

                        if use_tabstop && total_width > 0 {
                            // Tab size changes because of the inserted text
                            size -= tab_size;
                            tab_size = rs_tabstop_padding(vcol + size, ts, vts);
                            size += tab_size;
                        }
                    }
                }
                nvim_csarg_itr_next(csarg);
            }
        }

        // Handle double-width character wrapping
        let p_wrap = nvim_win_get_p_wrap(wp) != 0;
        if is_doublewidth && p_wrap && in_win_border_impl(wp, vcol + size - 2) {
            // Count the ">" in the last column
            size += 1;
            mb_added = 1;
        }

        // Get showbreak value
        let sbr = get_showbreak_value_impl(wp);

        // May have to add something for 'breakindent' and/or 'showbreak'
        // string at the start of a screen line.
        let mut head = mb_added;
        let p_bri = nvim_win_get_p_bri(wp) != 0;
        let sbr_nonempty = !sbr.is_null() && *sbr != 0;

        // When "size" is 0, no new screen line is started.
        if size > 0 && p_wrap && (sbr_nonempty || p_bri) {
            let mut col_off_prev = rs_win_col_off(wp);
            let view_width = nvim_win_get_view_width(wp);
            let width2 = view_width - col_off_prev + rs_win_col_off2(wp);
            let mut wcol = vcol + col_off_prev;
            let max_head_vcol = nvim_csarg_get_max_head_vcol(csarg);
            let mut added: c_int = 0;

            // Cells taken by 'showbreak'/'breakindent' before current char
            let mut head_prev: c_int = 0;
            if wcol >= view_width {
                wcol -= view_width;
                col_off_prev = view_width - width2;
                if wcol >= width2 && width2 > 0 {
                    wcol %= width2;
                }
                head_prev = nvim_csarg_get_indent_width(csarg);
                if head_prev == INT_MIN {
                    head_prev = 0;
                    if sbr_nonempty {
                        head_prev += nvim_vim_strsize(sbr);
                    }
                    if p_bri {
                        head_prev += nvim_get_breakindent_win(wp, line);
                    }
                    nvim_csarg_set_indent_width(csarg, head_prev);
                }
                if wcol < head_prev {
                    head_prev -= wcol;
                    wcol += head_prev;
                    added += head_prev;
                    if max_head_vcol <= 0 || vcol < max_head_vcol {
                        head += head_prev;
                    }
                } else {
                    head_prev = 0;
                }
                wcol += col_off_prev;
            }

            if wcol + size > view_width {
                // Cells taken by 'showbreak'/'breakindent' halfway current char
                let mut head_mid = nvim_csarg_get_indent_width(csarg);
                if head_mid == INT_MIN {
                    head_mid = 0;
                    if sbr_nonempty {
                        head_mid += nvim_vim_strsize(sbr);
                    }
                    if p_bri {
                        head_mid += nvim_get_breakindent_win(wp, line);
                    }
                    nvim_csarg_set_indent_width(csarg, head_mid);
                }
                if head_mid > 0 {
                    // Calculate effective window width
                    let prev_rem = view_width - wcol;
                    let mut width = width2 - head_mid;

                    if width <= 0 {
                        width = 1;
                    }
                    // Divide "size - prev_rem" by "width", rounding up
                    let cnt = (size - prev_rem + width - 1) / width;
                    added += cnt * head_mid;

                    if max_head_vcol == 0 || vcol + size + added < max_head_vcol {
                        head += cnt * head_mid;
                    } else if max_head_vcol > vcol + head_prev + prev_rem {
                        head += (max_head_vcol - (vcol + head_prev + prev_rem) + width2 - 1)
                            / width2
                            * head_mid;
                    } else if max_head_vcol < 0 {
                        let on_nul = c_int::from(*cur == 0);
                        let off = mb_added + nvim_virt_text_cursor_off(csarg, on_nul);
                        if off >= prev_rem {
                            if size > off {
                                head += (1 + (off - prev_rem) / width) * head_mid;
                            } else {
                                head += (off - prev_rem + width - 1) / width * head_mid;
                            }
                        }
                    }
                }
            }

            size += added;
        }

        // Handle 'linebreak' option
        let p_lbr = nvim_win_get_p_lbr(wp) != 0;
        let view_width = nvim_win_get_view_width(wp);
        let mut need_lbr = false;

        // If 'linebreak' set check at a blank before a non-blank if the line
        // needs a break here.
        if p_lbr && p_wrap && view_width != 0 {
            let cur0 = *cur as u8 as c_int;
            let cur1 = *cur.add(1) as u8 as c_int;
            if nvim_vim_isbreak(cur0) != 0 && nvim_vim_isbreak(cur1) == 0 {
                // Check if we're not in leading whitespace
                let mut t = line;
                while nvim_vim_isbreak(*t as u8 as c_int) != 0 {
                    t = t.add(1);
                }
                // 'linebreak' is only needed when not in leading whitespace
                need_lbr = cur as usize >= t as usize;
            }
        }

        if need_lbr {
            let mut s = cur;
            // Count all characters from first non-blank after a blank up to next
            // non-blank after a blank.
            let numberextra = rs_win_col_off(wp);
            let col_adj = size - 1;
            let mut colmax = view_width - numberextra - col_adj;
            if vcol >= colmax {
                colmax += col_adj;
                let n = colmax + rs_win_col_off2(wp);
                if n > 0 {
                    colmax += (((vcol - colmax) / n) + 1) * n - col_adj;
                }
            }

            let mut vcol2 = vcol;
            loop {
                let ps = s;
                // Advance s by UTF-8 character length
                let char_len = utf8_char_len(*s as u8);
                s = s.add(char_len);

                let c = *s as u8 as c_int;
                let ps_byte = *ps as u8 as c_int;

                // Break condition: stop if we reach end or specific break conditions
                let continue_loop = c != 0
                    && (nvim_vim_isbreak(c) != 0
                        || vcol2 == vcol
                        || nvim_vim_isbreak(ps_byte) == 0);

                if !continue_loop {
                    break;
                }

                vcol2 += rs_win_chartabsize(wp, s, vcol2);
                if vcol2 >= colmax {
                    // Doesn't fit
                    size = colmax - vcol + col_adj;
                    break;
                }
            }
        }

        CharSize { width: size, head }
    }
}

/// Get the number of cells taken up on the screen for the given arguments.
///
/// This is the full implementation that handles:
/// - Inline virtual text
/// - 'linebreak', 'breakindent', 'showbreak'
/// - Double-width character wrapping
///
/// # Safety
/// The `csarg` parameter must be a valid `CharsizeArg*` pointer.
/// The `cur` parameter must be a valid pointer to a character within the line.
#[no_mangle]
pub extern "C" fn rs_charsize_regular(
    csarg: CharsizeArgHandle,
    cur: *const c_char,
    vcol: c_int,
    cur_char: i32,
) -> CharSize {
    charsize_regular_impl(csarg, cur, vcol, cur_char)
}

// ============================================================================
// linesize_regular - Full line size calculation with virtual text
// ============================================================================

/// Calculate the display width of a line using the regular path.
///
/// This function handles inline virtual text, linebreak, breakindent, etc.
///
/// # Arguments
/// * `csarg` - CharsizeArg handle (must have been initialized)
/// * `vcol_arg` - Starting virtual column
/// * `len` - First byte of end character, or MAXCOL
///
/// # Returns
/// Virtual column before the character at "len", or full size if len is MAXCOL.
#[inline]
fn linesize_regular_impl(csarg: CharsizeArgHandle, vcol_arg: c_int, len: c_int) -> c_int {
    if csarg.is_null() {
        return vcol_arg;
    }

    unsafe {
        let line = nvim_csarg_get_line(csarg);
        if line.is_null() {
            return vcol_arg;
        }

        let mut vcol: i64 = vcol_arg as i64;
        let mut vcol_for_charsize = vcol_arg;

        // Initialize character iteration
        let mut ptr: *const c_char = std::ptr::null();
        let mut char_len: c_int = 0;
        let mut char_value = nvim_str_char_info_init(
            line,
            std::ptr::from_mut(&mut ptr),
            std::ptr::from_mut(&mut char_len),
        );

        // Iterate through characters
        while (ptr as isize - line as isize) < len as isize && *ptr != 0 {
            let cs = charsize_regular_impl(csarg, ptr, vcol_for_charsize, char_value);
            vcol += cs.width as i64;

            // Advance to next character
            char_value = nvim_str_char_info_next(
                std::ptr::from_mut(&mut ptr),
                char_len,
                char_value,
                std::ptr::from_mut(&mut char_len),
            );

            // Check for overflow
            if vcol > MAXCOL as i64 {
                vcol_for_charsize = MAXCOL;
                break;
            }
            vcol_for_charsize = vcol as c_int;
        }

        // Check for inline virtual text after the end of the line
        let virt_row = nvim_csarg_get_virt_row(csarg);
        if len == MAXCOL && virt_row >= 0 && *ptr == 0 {
            let cs = charsize_regular_impl(csarg, ptr, vcol_for_charsize, char_value);
            let cur_text_left = nvim_csarg_get_cur_text_width_left(csarg);
            let cur_text_right = nvim_csarg_get_cur_text_width_right(csarg);
            vcol += (cur_text_left + cur_text_right + cs.head) as i64;
            vcol_for_charsize = if vcol > MAXCOL as i64 {
                MAXCOL
            } else {
                vcol as c_int
            };
        }

        vcol_for_charsize
    }
}

/// Calculate the display width of a line using the regular path.
///
/// Returns the virtual column at the end of the specified length.
///
/// # Safety
/// The `csarg` parameter must be a valid `CharsizeArg*` pointer.
#[no_mangle]
pub extern "C" fn rs_linesize_regular(
    csarg: CharsizeArgHandle,
    vcol_arg: c_int,
    len: c_int,
) -> c_int {
    linesize_regular_impl(csarg, vcol_arg, len)
}

// ============================================================================
// getvcol - Get virtual column of position
// ============================================================================

/// CSType enum matching C's CSType (only FAST used for comparison)
const CSTYPE_FAST: c_int = 0;

/// Get the virtual column for a position in a line.
///
/// This function calculates the virtual column (display position) for a given
/// byte position in a line. It handles:
/// - Tab expansion
/// - Inline virtual text
/// - Multi-byte characters
/// - Visual mode cursor positioning for tabs
///
/// # Arguments
/// * `csarg` - CharsizeArg handle (must be initialized with init_charsize_arg)
/// * `line` - Pointer to the start of the line
/// * `end_col` - Byte position to calculate vcol for
/// * `cstype` - 0 for fast path, 1 for regular path
/// * `pos_lnum` - Line number of position (for visual mode comparison)
/// * `pos_coladd` - Virtual column add of position
/// * `start_out` - Output: start vcol of character (NULL to skip)
/// * `cursor_out` - Output: cursor vcol position (NULL to skip)
/// * `end_out` - Output: end vcol of character (NULL to skip)
/// * `pos_col_out` - Output: updated pos->col if at NUL (NULL to skip)
#[inline]
#[allow(clippy::too_many_arguments)]
#[allow(clippy::too_many_lines)]
fn getvcol_impl(
    csarg: CharsizeArgHandle,
    line: *const c_char,
    end_col: c_int,
    cstype: c_int,
    pos_lnum: c_int,
    pos_coladd: c_int,
    start_out: *mut c_int,
    cursor_out: *mut c_int,
    end_out: *mut c_int,
    pos_col_out: *mut c_int,
) {
    if csarg.is_null() || line.is_null() {
        return;
    }

    unsafe {
        let wp = nvim_csarg_get_win(csarg);
        let mut vcol: c_int = 0;
        let mut char_size;
        let mut on_nul = false;

        // Initialize character iteration
        let mut ptr: *const c_char = std::ptr::null();
        let mut char_len: c_int = 0;
        let mut char_value = nvim_str_char_info_init(
            line,
            std::ptr::from_mut(&mut ptr),
            std::ptr::from_mut(&mut char_len),
        );

        if cstype == CSTYPE_FAST {
            let use_tabstop = nvim_csarg_get_use_tabstop(csarg) != 0;
            loop {
                if *ptr == 0 {
                    // if cursor is at NUL, it is treated like 1 cell char
                    char_size = CharSize { width: 1, head: 0 };
                    break;
                }
                char_size = charsize_fast_impl(wp, ptr, use_tabstop, vcol, char_value);

                // Get next character info
                let next_ptr = ptr;
                let next_value = nvim_str_char_info_next(
                    std::ptr::from_mut(&mut ptr),
                    char_len,
                    char_value,
                    std::ptr::from_mut(&mut char_len),
                );

                // Check if we've passed end_col
                if (ptr as isize - line as isize) > end_col as isize {
                    ptr = next_ptr;
                    char_value = next_value;
                    break;
                }
                char_value = next_value;
                vcol += char_size.width;
            }
        } else {
            loop {
                char_size = charsize_regular_impl(csarg, ptr, vcol, char_value);
                // make sure we don't go past the end of the line
                if *ptr == 0 {
                    // NUL at end of line only takes one column unless there is virtual text
                    let cur_text_left = nvim_csarg_get_cur_text_width_left(csarg);
                    let cur_text_right = nvim_csarg_get_cur_text_width_right(csarg);
                    char_size.width = 1 + cur_text_left + cur_text_right;
                    on_nul = true;
                    break;
                }

                // Get next character info
                let next_ptr = ptr;
                let next_value = nvim_str_char_info_next(
                    std::ptr::from_mut(&mut ptr),
                    char_len,
                    char_value,
                    std::ptr::from_mut(&mut char_len),
                );

                // Check if we've passed end_col
                if (ptr as isize - line as isize) > end_col as isize {
                    ptr = next_ptr;
                    char_value = next_value;
                    break;
                }
                char_value = next_value;
                vcol += char_size.width;
            }
        }

        // Handle pos->col update for NUL case
        if *ptr == 0
            && end_col < MAXCOL
            && end_col > (ptr as isize - line as isize) as c_int
            && !pos_col_out.is_null()
        {
            *pos_col_out = (ptr as isize - line as isize) as c_int;
        }

        let head = char_size.head;
        let incr = char_size.width;

        if !start_out.is_null() {
            *start_out = vcol + head;
        }

        if !end_out.is_null() {
            *end_out = vcol + incr - 1;
        }

        if !cursor_out.is_null() {
            // Complex cursor logic for tabs in visual mode
            let state = nvim_get_State();
            let p_list = nvim_win_get_p_list(wp) != 0;
            let virtual_active = nvim_virtual_active(wp) != 0;
            let visual_active = nvim_get_VIsual_active() != 0;
            let p_sel_first = nvim_get_p_sel_first();

            // Check if we should position cursor at end of tab
            // Condition: TAB, in NORMAL mode, not list mode, not virtual edit,
            // and not in visual mode with exclusive selection or pos <= VIsual
            let cursor_at_end = char_value == TAB
                && (state & MODE_NORMAL) != 0
                && !p_list
                && !virtual_active
                && !(visual_active && {
                    // Check if p_sel is 'e' (exclusive) or ltoreq(pos, VIsual)
                    if p_sel_first == b'e' as c_char {
                        true
                    } else {
                        // Construct the position and compare with VIsual
                        let pos = PosT {
                            lnum: pos_lnum,
                            col: end_col,
                            coladd: pos_coladd,
                        };
                        let visual_pos = PosT {
                            lnum: nvim_get_VIsual_lnum(),
                            col: nvim_get_VIsual_col(),
                            coladd: nvim_get_VIsual_coladd(),
                        };
                        rs_ltoreq(pos, visual_pos) != 0
                    }
                });

            if cursor_at_end {
                // cursor at end
                *cursor_out = vcol + incr - 1;
            } else {
                let on_nul_flag = c_int::from(on_nul);
                vcol += nvim_virt_text_cursor_off(csarg, on_nul_flag);
                // cursor at start
                *cursor_out = vcol + head;
            }
        }
    }
}

/// Get the virtual column for a position.
///
/// This is the Rust implementation of getvcol().
///
/// # Safety
/// All pointer parameters must be valid or null.
#[no_mangle]
#[allow(clippy::too_many_arguments)]
pub extern "C" fn rs_getvcol(
    csarg: CharsizeArgHandle,
    line: *const c_char,
    end_col: c_int,
    cstype: c_int,
    pos_lnum: c_int,
    pos_coladd: c_int,
    start_out: *mut c_int,
    cursor_out: *mut c_int,
    end_out: *mut c_int,
    pos_col_out: *mut c_int,
) {
    getvcol_impl(
        csarg,
        line,
        end_col,
        cstype,
        pos_lnum,
        pos_coladd,
        start_out,
        cursor_out,
        end_out,
        pos_col_out,
    );
}

// ============================================================================
// plines_win_nofold - Physical lines for a buffer line
// ============================================================================

/// Get number of window lines physical line will occupy in window.
/// Does not care about folding, 'wrap' or filler lines.
///
/// This function calculates how many screen lines a buffer line will take
/// based on the line width and window width.
///
/// # Arguments
/// * `csarg` - CharsizeArg handle (must be initialized with init_charsize_arg)
/// * `cstype` - 0 for fast path, 1 for regular path
/// * `first_char` - First character of the line (0 for NUL/empty)
#[inline]
fn plines_win_nofold_impl(csarg: CharsizeArgHandle, cstype: c_int, first_char: c_int) -> c_int {
    if csarg.is_null() {
        return 1;
    }

    unsafe {
        let wp = nvim_csarg_get_win(csarg);
        let virt_row = nvim_csarg_get_virt_row(csarg);

        // Empty line without virtual text
        if first_char == 0 && virt_row < 0 {
            return 1;
        }

        // Get line width using linesize_fast or linesize_regular
        let col: i64 = if cstype == CSTYPE_FAST {
            let use_tabstop = nvim_csarg_get_use_tabstop(csarg);
            let line = nvim_csarg_get_line(csarg);
            i64::from(rs_linesize_fast(wp, use_tabstop, line, 0, MAXCOL))
        } else {
            i64::from(linesize_regular_impl(csarg, 0, MAXCOL))
        };

        // If list mode is on, the '$' at the end may take up one extra column
        let p_list = nvim_win_get_p_list(wp) != 0;
        let lcs_eol = nvim_win_get_lcs_eol(wp);
        let col = if p_list && lcs_eol != 0 { col + 1 } else { col };

        // Add column offset for 'number', 'relativenumber' and 'foldcolumn'
        let view_width = nvim_win_get_view_width(wp);
        let width = view_width - rs_win_col_off(wp);
        if width <= 0 {
            return 32000; // bigger than the number of screen lines
        }

        if col <= i64::from(width) {
            return 1;
        }

        let col = col - i64::from(width);
        let width = width + rs_win_col_off2(wp);
        let lines = (col + i64::from(width - 1)) / i64::from(width) + 1;

        if lines > 0 && lines <= i64::from(c_int::MAX) {
            lines as c_int
        } else {
            c_int::MAX
        }
    }
}

/// Get number of window lines physical line will occupy.
///
/// # Safety
/// The `csarg` parameter must be a valid `CharsizeArg*` pointer.
#[no_mangle]
pub extern "C" fn rs_plines_win_nofold(
    csarg: CharsizeArgHandle,
    cstype: c_int,
    first_char: c_int,
) -> c_int {
    plines_win_nofold_impl(csarg, cstype, first_char)
}

// ============================================================================
// plines_win_col - Physical lines up to a column
// ============================================================================

/// Calculate physical screen lines from start of line to given column.
///
/// This iterates through characters up to `column`, then calculates
/// how many screen lines that takes.
///
/// # Arguments
/// * `csarg` - Character size argument handle
/// * `line` - Pointer to the line string
/// * `column` - Column to count up to
/// * `cstype` - Charsize type (0 = fast, 1 = regular)
/// * `fill_lines` - Number of filler lines (from win_get_fill)
///
/// # Returns
/// Total number of screen lines including fill lines
#[inline]
fn plines_win_col_impl(
    csarg: CharsizeArgHandle,
    line: *const c_char,
    column: c_int,
    cstype: c_int,
    fill_lines: c_int,
) -> c_int {
    if csarg.is_null() || line.is_null() {
        return fill_lines + 1;
    }

    unsafe {
        let wp = nvim_csarg_get_win(csarg);

        // Check for wrap off or zero width
        if nvim_win_get_p_wrap(wp) == 0 {
            return fill_lines + 1;
        }

        let view_width = nvim_win_get_view_width(wp);
        if view_width == 0 {
            return fill_lines + 1;
        }

        // Iterate through characters up to column
        let mut vcol: c_int = 0;
        let mut col_count = column;

        // Initialize character iteration
        let mut ptr = line;
        let mut char_len: c_int = 0;
        let mut char_value = nvim_str_char_info_init(
            line,
            std::ptr::from_mut(&mut ptr),
            std::ptr::from_mut(&mut char_len),
        );

        if cstype == CSTYPE_FAST {
            let use_tabstop = nvim_csarg_get_use_tabstop(csarg) != 0;
            while *ptr != 0 && {
                col_count -= 1;
                col_count >= 0
            } {
                let cs = charsize_fast_impl(wp, ptr, use_tabstop, vcol, char_value);
                vcol += cs.width;

                // Advance to next character
                char_value = nvim_str_char_info_next(
                    std::ptr::from_mut(&mut ptr),
                    char_len,
                    char_value,
                    std::ptr::from_mut(&mut char_len),
                );
            }
        } else {
            while *ptr != 0 && {
                col_count -= 1;
                col_count >= 0
            } {
                let cs = charsize_regular_impl(csarg, ptr, vcol, char_value);
                vcol += cs.width;

                // Advance to next character
                char_value = nvim_str_char_info_next(
                    std::ptr::from_mut(&mut ptr),
                    char_len,
                    char_value,
                    std::ptr::from_mut(&mut char_len),
                );
            }
        }

        // If current char is a TAB, and the TAB is not displayed as ^I, and we're not
        // in MODE_INSERT state, then col must be adjusted so that it represents the
        // last screen position of the TAB.
        let mut col = vcol;
        let state = nvim_get_State();
        let use_tabstop = nvim_csarg_get_use_tabstop(csarg) != 0;

        if char_value == TAB && (state & MODE_NORMAL) != 0 && use_tabstop {
            // Use appropriate charsize function
            let tab_size = if cstype == CSTYPE_FAST {
                charsize_fast_impl(wp, ptr, true, col, char_value)
            } else {
                charsize_regular_impl(csarg, ptr, col, char_value)
            };
            col += tab_size.width - 1;
        }

        // Add column offset for 'number', 'relativenumber', 'foldcolumn', etc.
        let width = view_width - rs_win_col_off(wp);
        if width <= 0 {
            return 9999;
        }

        let mut lines = fill_lines + 1;
        if col > width {
            lines += (col - width) / (width + rs_win_col_off2(wp)) + 1;
        }
        lines
    }
}

/// Get the number of physical screen lines used from start of line to column.
///
/// # Safety
/// The `csarg` parameter must be a valid `CharsizeArg*` pointer.
/// The `line` parameter must be a valid null-terminated string pointer.
#[no_mangle]
pub extern "C" fn rs_plines_win_col(
    csarg: CharsizeArgHandle,
    line: *const c_char,
    column: c_int,
    cstype: c_int,
    fill_lines: c_int,
) -> c_int {
    plines_win_col_impl(csarg, line, column, cstype, fill_lines)
}

// ============================================================================
// skipcol helper functions
// ============================================================================

extern "C" {
    fn nvim_win_get_skipcol(wp: WinHandle) -> c_int;
}

/// Get the number of screen lines skipped with "wp->w_skipcol".
///
/// This calculates how many screen lines are skipped when smooth scrolling
/// is active and the window has a skipcol value.
#[inline]
fn adjust_plines_for_skipcol_impl(wp: WinHandle) -> c_int {
    if wp.is_null() {
        return 0;
    }

    unsafe {
        let skipcol = nvim_win_get_skipcol(wp);
        if skipcol == 0 {
            return 0;
        }

        let width = nvim_win_get_view_width(wp) - rs_win_col_off(wp);
        let w2 = width + rs_win_col_off2(wp);

        if skipcol >= width && w2 > 0 {
            (skipcol - width) / w2 + 1
        } else {
            0
        }
    }
}

/// Get the number of screen lines skipped with "wp->w_skipcol".
///
/// # Safety
/// The `wp` parameter must be a valid window handle.
#[no_mangle]
pub extern "C" fn rs_adjust_plines_for_skipcol(wp: WinHandle) -> c_int {
    adjust_plines_for_skipcol_impl(wp)
}

/// Calculates the skipcol offset for window "wp" given how many
/// physical lines we want to scroll down.
#[inline]
fn skipcol_from_plines_impl(wp: WinHandle, plines_off: c_int) -> c_int {
    if wp.is_null() {
        return 0;
    }

    unsafe {
        let width1 = nvim_win_get_view_width(wp) - rs_win_col_off(wp);

        let mut skipcol = 0;
        if plines_off > 0 {
            skipcol += width1;
        }
        if plines_off > 1 {
            skipcol += (width1 + rs_win_col_off2(wp)) * (plines_off - 1);
        }
        skipcol
    }
}

/// Calculates the skipcol offset for window "wp" given how many
/// physical lines we want to scroll down.
///
/// # Safety
/// The `wp` parameter must be a valid window handle.
#[no_mangle]
pub extern "C" fn rs_skipcol_from_plines(wp: WinHandle, plines_off: c_int) -> c_int {
    skipcol_from_plines_impl(wp, plines_off)
}

#[cfg(test)]
mod tests {
    // Tests require FFI stubs which aren't available in pure Rust testing.
    // Integration testing is done via the full Neovim build.
}
