//! Rust implementations of getsourceline and related helpers.
//!
//! These functions read lines from sourced script files, handling:
//! - continuation lines (backslash-joined lines)
//! - encoding conversion
//! - debugger breakpoints
//! - profiling hooks

use std::ffi::{c_char, c_int, c_void};

use crate::constants::{CONV_NONE, CPO_CONCAT, PROF_YES};
use crate::globals;
use crate::globals::GarrayT;

// =============================================================================
// External C functions
// =============================================================================

extern "C" {
    // Memory management
    fn xfree(p: *mut c_void);

    // garray operations
    fn ga_init(gap: *mut GarrayT, itemsize: c_int, growsize: c_int);
    fn ga_grow(gap: *mut GarrayT, n: c_int);
    fn ga_concat(gap: *mut GarrayT, s: *const c_char);
    fn ga_concat_len(gap: *mut GarrayT, s: *const c_char, len: usize);
    fn ga_append(gap: *mut GarrayT, byte: u8);
    fn ga_set_growsize(gap: *mut GarrayT, size: c_int);

    // Charset / path
    #[link_name = "skipwhite"]
    fn nvim_rt_skipwhite(p: *const c_char) -> *mut c_char;
    #[link_name = "skipwhite_len"]
    fn nvim_rt_skipwhite_len(p: *const c_char, len: usize) -> *const c_char;
    // p_cpo declared in static block below
    #[link_name = "vim_strchr"]
    fn nvim_rt_vim_strchr(buf: *const c_char, c: c_int) -> *mut c_char;

    // Debugger
    #[link_name = "dbg_find_breakpoint"]
    fn nvim_rt_dbg_find_breakpoint(file: bool, fname: *mut c_char, after: c_int) -> c_int;
    #[link_name = "dbg_breakpoint"]
    fn nvim_rt_dbg_breakpoint(fname: *mut c_char, lnum: c_int);

    // Profiling
    #[link_name = "script_line_start"]
    fn nvim_rt_script_line_start();
    #[link_name = "script_line_end"]
    fn nvim_rt_script_line_end();

    // SOURCING_LNUM access
    fn nvim_rt_get_sourcing_lnum() -> c_int;
    fn nvim_rt_set_sourcing_lnum(lnum: c_int);

    // source_cookie_T accessors (in runtime.c)
    fn nvim_rt_cookie_get_dbg_tick(cookie: *mut c_void) -> c_int;
    fn nvim_rt_cookie_get_src_from_buf_or_str(cookie: *mut c_void) -> bool;
    fn nvim_rt_cookie_get_fname(cookie: *mut c_void) -> *const c_char;
    fn nvim_rt_cookie_get_finished(cookie: *mut c_void) -> bool;
    fn nvim_rt_cookie_get_fp(cookie: *mut c_void) -> *mut c_void;
    fn nvim_rt_cookie_get_nextline(cookie: *mut c_void) -> *mut c_char;
    fn nvim_rt_cookie_set_nextline(cookie: *mut c_void, val: *mut c_char);
    fn nvim_rt_cookie_get_sourcing_lnum(cookie: *mut c_void) -> c_int;
    fn nvim_rt_cookie_inc_sourcing_lnum(cookie: *mut c_void);
    fn nvim_rt_cookie_dec_sourcing_lnum(cookie: *mut c_void);
    fn nvim_rt_cookie_get_breakpoint(cookie: *mut c_void) -> c_int;
    fn nvim_rt_cookie_set_breakpoint(cookie: *mut c_void, val: c_int);
    fn nvim_rt_cookie_set_dbg_tick(cookie: *mut c_void, val: c_int);
    fn nvim_rt_cookie_get_buf_lnum(cookie: *mut c_void) -> c_int;
    fn nvim_rt_cookie_inc_buf_lnum(cookie: *mut c_void);
    fn nvim_rt_cookie_get_buflines_len(cookie: *mut c_void) -> c_int;
    fn nvim_rt_cookie_get_bufline(cookie: *mut c_void, idx: c_int) -> *const c_char;
    fn nvim_rt_cookie_get_conv(cookie: *mut c_void) -> *mut c_void;
    #[link_name = "line_breakcheck"]
    fn nvim_rt_line_breakcheck();

    // Encoding conversion
    #[link_name = "string_convert"]
    fn nvim_rt_string_convert(vcp: *mut c_void, s: *mut c_char, len: *mut usize) -> *mut c_char;
    fn nvim_rt_conv_get_type(vcp: *const c_void) -> c_int;

}

extern "C" {
    #[link_name = "p_cpo"]
    static nvim_rt_p_cpo: *const c_char;
}

// =============================================================================
// Constants
// =============================================================================

const NUL: u8 = 0;
const CTRL_V: u8 = 22; // Ctrl_V character

// =============================================================================
// Helper: concat_continued_line (static)
// =============================================================================

/// Concatenate a Vimscript continuation line into a growarray.
///
/// Returns `true` if this line started with a continuation character (next
/// line should be checked too). Skips comment-continuation lines `"\ `.
///
/// # Safety
/// Operates on raw C pointers. `ga` must point to an initialized garray.
unsafe fn concat_continued_line(
    ga: *mut GarrayT,
    init_growsize: c_int,
    p: *const c_char,
    len: usize,
) -> bool {
    let line = nvim_rt_skipwhite_len(p, len);
    let trimmed_len = len - (line as usize - p as usize);

    // Skip comment-continuation lines starting with '"\ '
    if trimmed_len >= 3 {
        let b0 = *line.cast::<u8>();
        let b1 = *line.add(1).cast::<u8>();
        let b2 = *line.add(2).cast::<u8>();
        if b0 == b'"' && b1 == b'\\' && b2 == b' ' {
            return true;
        }
    }

    // Non-continuation: empty line or not starting with '\'
    if trimmed_len == 0 || *line.cast::<u8>() != b'\\' {
        return false;
    }

    if (*ga).ga_len > init_growsize {
        ga_set_growsize(ga, c_int::min((*ga).ga_len, 8000));
    }
    ga_concat_len(ga, line.add(1), trimmed_len - 1);
    true
}

// =============================================================================
// Helper: get_one_sourceline (static)
// =============================================================================

/// Read a single raw line from the sourced file or buffer.
///
/// Handles escaped newlines (Ctrl_V before `\n`), EOF, and EINTR for file
/// mode. Returns an allocated string, or NULL for end-of-file.
///
/// # Safety
/// Operates on raw C cookie pointers and C file handles.
unsafe fn get_one_sourceline(cookie: *mut c_void) -> *mut c_char {
    let mut ga = GarrayT::zeroed();
    let mut have_read = false;

    // Use a growarray to store the sourced line (item size 1 byte).
    ga_init(&raw mut ga, 1, 250);

    // Increment sourcing line number for each call.
    nvim_rt_cookie_inc_sourcing_lnum(cookie);

    loop {
        // Make room to read at least 120 more characters.
        ga_grow(&raw mut ga, 120);

        let len: c_int = if nvim_rt_cookie_get_src_from_buf_or_str(cookie) {
            // Buffer/string mode: pull one line from buflines.
            let buf_lnum = nvim_rt_cookie_get_buf_lnum(cookie);
            if buf_lnum >= nvim_rt_cookie_get_buflines_len(cookie) {
                break; // all lines processed
            }
            let src_line = nvim_rt_cookie_get_bufline(cookie, buf_lnum);
            ga_concat(&raw mut ga, src_line);
            nvim_rt_cookie_inc_buf_lnum(cookie);

            // Append a NUL terminator.
            ga_grow(&raw mut ga, 1);
            let data = ga.ga_data.cast::<u8>();
            *data.add(ga.ga_len as usize) = NUL;
            ga.ga_len += 1;
            ga.ga_len
        } else {
            // File mode: use fgets.
            let fp = nvim_rt_cookie_get_fp(cookie).cast::<libc::FILE>();
            let buf_start = ga.ga_data.cast::<c_char>().add(ga.ga_len as usize);
            let buf_room = ga.ga_maxlen - ga.ga_len;

            *libc::__errno_location() = 0;
            if libc::fgets(buf_start, buf_room, fp).is_null() {
                if *libc::__errno_location() == libc::EINTR {
                    continue; // retry on EINTR
                }
                break;
            }

            ga.ga_len + libc::strlen(buf_start.cast()) as c_int
        };

        have_read = true;
        ga.ga_len = len;

        // If the line was longer than the buffer, read more.
        if ga.ga_maxlen - ga.ga_len == 1 {
            let last = *ga.ga_data.cast::<u8>().add(len as usize - 1);
            if last != b'\n' {
                continue;
            }
        }

        // Remove trailing NL and check for escaped newline.
        if len >= 1 {
            let data = ga.ga_data.cast::<u8>();
            if *data.add(len as usize - 1) == b'\n' {
                // Check for odd number of Ctrl_V before the NL.
                let mut c = len as isize - 2;
                while c >= 0 && *data.add(c as usize) == CTRL_V {
                    c -= 1;
                }
                // The '\n' is escaped if there is an odd number of Ctrl_V's
                // just before it. Check by parity: (len & 1) != (c & 1).
                if (len as usize & 1) != (c as usize & 1) {
                    // Escaped NL: read more on next line.
                    nvim_rt_cookie_inc_sourcing_lnum(cookie);
                    continue;
                }
                // Remove the trailing NL.
                *data.add(len as usize - 1) = NUL;
                ga.ga_len = len - 1;
            }
        }

        // Check for ^C interrupts periodically.
        nvim_rt_line_breakcheck();
        break;
    }

    if have_read {
        ga.ga_data.cast::<c_char>()
    } else {
        xfree(ga.ga_data);
        std::ptr::null_mut()
    }
}

// =============================================================================
// rs_getsourceline: the main exported function
// =============================================================================

/// Get one full line from a sourced file.
///
/// Called by `do_cmdline()` when sourcing a script. Returns a pointer to an
/// allocated line, or NULL for end-of-file.
///
/// # Safety
/// Called from C via function pointer. Manipulates raw pointers.
///
/// # Panics
/// Panics if `size_of::<c_char>()` does not fit in `c_int` (impossible on all
/// supported platforms).
#[unsafe(export_name = "getsourceline")]
pub unsafe extern "C" fn rs_getsourceline(
    _c: c_int,
    cookie: *mut c_void,
    _indent: c_int,
    do_concat: bool,
) -> *mut c_char {
    // If breakpoints have been added/deleted since last check, refresh.
    if nvim_rt_cookie_get_dbg_tick(cookie) < globals::debug_tick
        && !nvim_rt_cookie_get_src_from_buf_or_str(cookie)
    {
        let fname = nvim_rt_cookie_get_fname(cookie).cast_mut();
        let new_bp = nvim_rt_dbg_find_breakpoint(true, fname, nvim_rt_get_sourcing_lnum());
        nvim_rt_cookie_set_breakpoint(cookie, new_bp);
        nvim_rt_cookie_set_dbg_tick(cookie, globals::debug_tick);
    }

    if globals::do_profiling == PROF_YES {
        nvim_rt_script_line_end();
    }

    // Update the current sourcing line number.
    nvim_rt_set_sourcing_lnum(nvim_rt_cookie_get_sourcing_lnum(cookie) + 1);

    // Get the current line. If there is a read-ahead line, use it; otherwise
    // read one now. fp is NULL if actually using a string/buffer.
    let mut line: *mut c_char = if nvim_rt_cookie_get_finished(cookie)
        || (!nvim_rt_cookie_get_src_from_buf_or_str(cookie)
            && nvim_rt_cookie_get_fp(cookie).is_null())
    {
        std::ptr::null_mut()
    } else if nvim_rt_cookie_get_nextline(cookie).is_null() {
        get_one_sourceline(cookie)
    } else {
        let nl = nvim_rt_cookie_get_nextline(cookie);
        nvim_rt_cookie_set_nextline(cookie, std::ptr::null_mut());
        nvim_rt_cookie_inc_sourcing_lnum(cookie);
        nl
    };

    if !line.is_null() && globals::do_profiling == PROF_YES {
        nvim_rt_script_line_start();
    }

    // Concatenate continuation lines (backslash-continued) unless 'C' is in
    // 'cpoptions'.
    let no_cpo_concat = nvim_rt_vim_strchr(nvim_rt_p_cpo, CPO_CONCAT).is_null();
    if !line.is_null() && do_concat && no_cpo_concat {
        // Compensate for the one line read-ahead.
        nvim_rt_cookie_dec_sourcing_lnum(cookie);

        // Get the next line and check if it starts with backslash or '"\ '.
        nvim_rt_cookie_set_nextline(cookie, get_one_sourceline(cookie));
        let nextline = nvim_rt_cookie_get_nextline(cookie);
        if !nextline.is_null() {
            let p = nvim_rt_skipwhite(nextline);
            if *p.cast::<u8>() == b'\\'
                || (*p.cast::<u8>() == b'"'
                    && *p.add(1).cast::<u8>() == b'\\'
                    && *p.add(2).cast::<u8>() == b' ')
            {
                let mut ga = GarrayT::zeroed();
                ga_init(
                    &raw mut ga,
                    c_int::try_from(size_of::<c_char>()).unwrap(),
                    400,
                );
                ga_concat(&raw mut ga, line);

                while !nvim_rt_cookie_get_nextline(cookie).is_null() {
                    let nl = nvim_rt_cookie_get_nextline(cookie);
                    let nl_len = libc::strlen(nl.cast());
                    if !concat_continued_line(&raw mut ga, 400, nl, nl_len) {
                        break;
                    }
                    xfree(nl.cast::<c_void>());
                    nvim_rt_cookie_set_nextline(cookie, get_one_sourceline(cookie));
                }
                ga_append(&raw mut ga, NUL);
                xfree(line.cast::<c_void>());
                line = ga.ga_data.cast::<c_char>();
            }
        }
    }

    // Convert encoding if needed.
    if !line.is_null() {
        let conv = nvim_rt_cookie_get_conv(cookie);
        if nvim_rt_conv_get_type(conv) != CONV_NONE {
            let s = nvim_rt_string_convert(conv, line, std::ptr::null_mut());
            if !s.is_null() {
                xfree(line.cast::<c_void>());
                line = s;
            }
        }
    }

    // Handle breakpoints.
    if !nvim_rt_cookie_get_src_from_buf_or_str(cookie) {
        let bp = nvim_rt_cookie_get_breakpoint(cookie);
        if bp != 0 && bp <= nvim_rt_get_sourcing_lnum() {
            let fname = nvim_rt_cookie_get_fname(cookie).cast_mut();
            nvim_rt_dbg_breakpoint(fname, nvim_rt_get_sourcing_lnum());
            // Find next breakpoint.
            let new_bp = nvim_rt_dbg_find_breakpoint(true, fname, nvim_rt_get_sourcing_lnum());
            nvim_rt_cookie_set_breakpoint(cookie, new_bp);
            nvim_rt_cookie_set_dbg_tick(cookie, globals::debug_tick);
        }
    }

    line
}
