//! Implementation of `get_breakindent_win()` — breakindent calculation with caching.

use std::ffi::{c_char, c_int};

use nvim_buffer::BufHandle;
use nvim_memory::xfree;

use crate::{rs_indent_size_no_ts, rs_indent_size_ts, WinHandle};

// C accessor functions
extern "C" {
    // Existing accessors (window.c, move.c, message.c, option.c)
    fn nvim_win_get_view_width(wp: WinHandle) -> c_int;
    fn nvim_win_get_p_list(wp: WinHandle) -> c_int;
    fn nvim_win_get_lcs_tab1(wp: WinHandle) -> u32;
    fn nvim_win_get_briopt_sbr(wp: WinHandle) -> bool;
    fn nvim_win_get_briopt_list(wp: WinHandle) -> c_int;
    fn nvim_win_get_buffer(wp: WinHandle) -> BufHandle;
    fn nvim_win_col_off(wp: WinHandle) -> c_int;
    fn nvim_win_col_off2(wp: WinHandle) -> c_int;
    fn nvim_vim_strsize(s: *const c_char) -> c_int;

    // indent_ffi.c accessors
    fn nvim_win_get_briopt_shift(wp: WinHandle) -> c_int;
    fn nvim_win_get_briopt_min(wp: WinHandle) -> c_int;
    fn nvim_win_get_briopt_vcol(wp: WinHandle) -> c_int;
    fn nvim_buf_get_b_fnum(buf: BufHandle) -> c_int;
    fn nvim_buf_get_p_ts(buf: BufHandle) -> i64;
    fn nvim_buf_get_p_vts_array(buf: BufHandle) -> *const c_int;
    fn nvim_indent_buf_get_changedtick(buf: BufHandle) -> i64;
    fn nvim_get_flp_value(buf: BufHandle) -> *const c_char;
    fn nvim_get_dy_flags_uhex() -> u32;
    fn nvim_indent_get_showbreak_value(wp: WinHandle) -> *const c_char;

    fn nvim_breakindent_flp_match(
        wp: WinHandle,
        pat: *const c_char,
        line: *const c_char,
        briopt_list: c_int,
        out_width: *mut c_int,
    ) -> c_int;

    fn nvim_strcmp(s1: *const c_char, s2: *const c_char) -> c_int;
    fn nvim_xstrnsave(s: *const c_char, len: usize) -> *mut c_char;
    fn nvim_strlen(s: *const c_char) -> usize;
}

/// Static cache for breakindent calculation.
/// SAFETY: Single-threaded (Neovim guarantee).
struct BreakindentCache {
    prev_indent: c_int,
    prev_ts: i64,
    prev_vts: *const c_int,
    prev_fnum: c_int,
    prev_line: *mut c_char,
    prev_tick: i64,
    prev_list: c_int,
    prev_listopt: c_int,
    prev_no_ts: bool,
    prev_dy_uhex: u32,
    prev_flp: *mut c_char,
}

static mut CACHE: BreakindentCache = BreakindentCache {
    prev_indent: 0,
    prev_ts: 0,
    prev_vts: std::ptr::null(),
    prev_fnum: 0,
    prev_line: std::ptr::null_mut(),
    prev_tick: 0,
    prev_list: 0,
    prev_listopt: 0,
    prev_no_ts: false,
    prev_dy_uhex: 0,
    prev_flp: std::ptr::null_mut(),
};

/// Helper: duplicate a C string using xmalloc.
unsafe fn xstrdup(s: *const c_char) -> *mut c_char {
    let len = nvim_strlen(s);
    nvim_xstrnsave(s, len)
}

/// Return appropriate space number for breakindent.
///
/// Window must be specified, since it is not necessarily the current one.
///
/// # Safety
/// - `wp` must be a valid window handle.
/// - `line` must be a valid null-terminated C string.
/// - Accesses global editor state (single-threaded).
#[no_mangle]
pub unsafe extern "C" fn rs_get_breakindent_win(wp: WinHandle, line: *const c_char) -> c_int {
    let cache = &raw mut CACHE;

    let buf = nvim_win_get_buffer(wp);

    // window width minus window margin space
    let eff_wwidth = nvim_win_get_view_width(wp) - nvim_win_col_off(wp) + nvim_win_col_off2(wp);

    // In list mode, if 'listchars' "tab" isn't set, a TAB is displayed as ^I.
    let no_ts = nvim_win_get_p_list(wp) != 0 && nvim_win_get_lcs_tab1(wp) == 0;

    let b_p_ts = nvim_buf_get_p_ts(buf);
    let b_p_vts = nvim_buf_get_p_vts_array(buf);
    let b_fnum = nvim_buf_get_b_fnum(buf);
    let tick = nvim_indent_buf_get_changedtick(buf);
    let listopt = nvim_win_get_briopt_list(wp);
    let dy_uhex = nvim_get_dy_flags_uhex();
    let flp = nvim_get_flp_value(buf);
    let briopt_vcol = nvim_win_get_briopt_vcol(wp);

    // Use cached indent, unless something changed.
    let flp_changed = (*cache).prev_flp.is_null() || nvim_strcmp((*cache).prev_flp, flp) != 0;
    let line_changed = (*cache).prev_line.is_null() || nvim_strcmp((*cache).prev_line, line) != 0;

    if (*cache).prev_fnum != b_fnum
        || (*cache).prev_ts != b_p_ts
        || (*cache).prev_vts != b_p_vts
        || (*cache).prev_tick != tick
        || (*cache).prev_listopt != listopt
        || (*cache).prev_no_ts != no_ts
        || (*cache).prev_dy_uhex != dy_uhex
        || flp_changed
        || line_changed
    {
        (*cache).prev_fnum = b_fnum;
        xfree((*cache).prev_line.cast());
        (*cache).prev_line = xstrdup(line);
        (*cache).prev_ts = b_p_ts;
        (*cache).prev_vts = b_p_vts;

        if briopt_vcol == 0 {
            if no_ts {
                (*cache).prev_indent = rs_indent_size_no_ts(line);
            } else {
                (*cache).prev_indent = rs_indent_size_ts(line, b_p_ts, b_p_vts);
            }
        }

        (*cache).prev_tick = tick;
        (*cache).prev_listopt = listopt;
        (*cache).prev_list = 0;
        (*cache).prev_no_ts = no_ts;
        (*cache).prev_dy_uhex = dy_uhex;
        xfree((*cache).prev_flp.cast());
        (*cache).prev_flp = xstrdup(flp);

        // Add additional indent for numbered lists.
        if listopt != 0 && briopt_vcol == 0 {
            let mut width: c_int = 0;
            let matched =
                nvim_breakindent_flp_match(wp, (*cache).prev_flp, line, listopt, &mut width);
            if matched != 0 {
                if listopt > 0 {
                    (*cache).prev_list += listopt;
                } else {
                    (*cache).prev_indent = width;
                }
            }
        }
    }

    let mut bri: c_int;
    if briopt_vcol != 0 {
        bri = briopt_vcol;
        (*cache).prev_list = 0;
    } else {
        bri = (*cache).prev_indent + nvim_win_get_briopt_shift(wp);
    }

    // Add offset for number column, if 'n' is in 'cpoptions'
    bri += nvim_win_col_off2(wp);

    // Add additional indent for numbered lists
    if listopt > 0 {
        bri += (*cache).prev_list;
    }

    // Indent minus the length of the showbreak string
    if nvim_win_get_briopt_sbr(wp) {
        bri -= nvim_vim_strsize(nvim_indent_get_showbreak_value(wp));
    }

    // Never indent past left window margin
    let briopt_min = nvim_win_get_briopt_min(wp);
    if bri < 0 {
        bri = 0;
    } else if bri > eff_wwidth - briopt_min {
        bri = if eff_wwidth - briopt_min < 0 {
            0
        } else {
            eff_wwidth - briopt_min
        };
    }

    bri
}
