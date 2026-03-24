//! Verbose and redirection message handling
//!
//! Provides Rust implementations for verbose message output and
//! redirection state management.

use std::ffi::{c_char, c_int, c_void};

// ============================================================================
// Rust-owned statics (previously file-local in message.c)
// ============================================================================

/// Verbose output file handle (replaces C static verbose_fd)
#[no_mangle]
pub static mut verbose_fd: *mut c_void = std::ptr::null_mut();

/// Whether verbose file was already attempted to open (replaces C static verbose_did_open)
#[no_mangle]
pub static mut verbose_did_open: bool = false;

/// Column position tracked across redir_write calls (replaces C static cur_col)
#[allow(non_upper_case_globals)]
static mut redir_write_cur_col: c_int = 0;

// C function declarations for verbose operations
extern "C" {
    static mut msg_silent: c_int;
    // msg_ext_set_kind is implemented in Rust (display.rs)
    fn msg_ext_set_kind(kind: *const c_char);
    // Verbose state (defined as Rust statics in display.rs)
    static mut msg_ext_kind: *const c_char;
    static mut verbose_kind: *const c_char;
    static mut pre_verbose_kind: *const c_char;

    // State accessors
    static mut msg_scroll: c_int;
    static mut msg_row: c_int;
    static mut cmdline_row: c_int;

    // Redirection state
    static mut redir_off: bool;
    static mut redir_fd: *mut c_void;
    static mut redir_reg: c_int;
    static mut redir_vname: bool;
    static mut capture_ga: *mut c_void;
    static mut p_vfile: *mut c_char;

    // For verbose_open_impl
    fn os_fopen(fname: *const c_char, mode: *const c_char) -> *mut c_void;
    fn fclose(f: *mut c_void) -> c_int;
    fn emsg(s: *const c_char) -> bool;
    fn gettext(s: *const c_char) -> *const c_char;
    fn xstrdup(s: *const c_char) -> *mut c_char;
    fn xfree(ptr: *mut c_void);
    static e_notopen: [c_char; 0];

    // For redir_write
    static mut msg_col: c_int;
    fn redirecting() -> c_int;
    fn ga_concat_len(gap: *mut c_void, s: *const c_char, len: usize);
    fn write_reg_contents(regname: c_int, str_: *const c_char, str_len: isize, must_append: bool);
    fn var_redir_str(s: *const c_char, maxlen: c_int);
    fn fputs(s: *const c_char, stream: *mut c_void) -> c_int;
    fn putc(c: c_int, stream: *mut c_void) -> c_int;
}

// ============================================================================
// Verbose Message Functions
// ============================================================================

/// Returns true if 'verbosefile' option is set (non-empty string).
///
/// # Safety
/// Reads the p_vfile global.
unsafe fn p_vfile_not_empty() -> bool {
    !p_vfile.is_null() && *p_vfile != 0
}

/// Enter verbose message mode.
///
/// Silences messages if 'verbosefile' is set and sets message kind.
/// Must be paired with `rs_verbose_leave()`.
///
/// # Safety
/// Calls C accessor functions that modify global state.
#[export_name = "verbose_enter"]
pub unsafe extern "C" fn rs_verbose_enter() {
    if p_vfile_not_empty() {
        let silent = msg_silent;
        msg_silent = silent + 1;
    }
    // Save pre_verbose_kind if not already in verbose mode, then set verbose kind.
    if msg_ext_kind != verbose_kind {
        pre_verbose_kind = msg_ext_kind;
    }
    msg_ext_set_kind(verbose_kind);
}

/// Leave verbose message mode.
///
/// Restores message silence and message kind.
/// Must be paired with `rs_verbose_enter()`.
///
/// # Safety
/// Calls C accessor functions that modify global state.
#[export_name = "verbose_leave"]
pub unsafe extern "C" fn rs_verbose_leave() {
    if p_vfile_not_empty() {
        let silent = msg_silent;
        if silent > 0 {
            msg_silent = silent - 1;
        } else {
            msg_silent = 0;
        }
    }
    if !pre_verbose_kind.is_null() {
        msg_ext_set_kind(pre_verbose_kind);
        pre_verbose_kind = std::ptr::null();
    }
}

/// Enter verbose message mode with scroll.
///
/// Like `rs_verbose_enter()` but also sets msg_scroll.
/// Must be paired with `rs_verbose_leave_scroll()`.
///
/// # Safety
/// Calls C accessor functions that modify global state.
#[export_name = "verbose_enter_scroll"]
pub unsafe extern "C" fn rs_verbose_enter_scroll() {
    rs_verbose_enter();
    if !p_vfile_not_empty() {
        // always scroll up, don't overwrite
        msg_scroll = 1;
    }
}

/// Leave verbose message mode with scroll.
///
/// Like `rs_verbose_leave()` but also updates cmdline_row.
/// Must be paired with `rs_verbose_enter_scroll()`.
///
/// # Safety
/// Calls C accessor functions that modify global state.
#[export_name = "verbose_leave_scroll"]
pub unsafe extern "C" fn rs_verbose_leave_scroll() {
    rs_verbose_leave();
    if !p_vfile_not_empty() {
        cmdline_row = msg_row;
    }
}

/// Stop verbose file output.
///
/// Closes the verbose file if open.
///
/// # Safety
/// Modifies verbose_fd and verbose_did_open globals; calls fclose().
#[export_name = "verbose_stop"]
pub unsafe extern "C" fn rs_verbose_stop() {
    if !verbose_fd.is_null() {
        fclose(verbose_fd);
        verbose_fd = std::ptr::null_mut();
    }
    verbose_did_open = false;
}

// C FAIL/OK constants
const FAIL: c_int = 0;
const OK: c_int = 1;

/// Open the verbose file ('verbosefile').
///
/// Returns FAIL or OK.
///
/// # Safety
/// Modifies verbose_fd and verbose_did_open globals; calls os_fopen(), emsg().
#[export_name = "verbose_open"]
#[must_use]
pub unsafe extern "C" fn rs_verbose_open() -> c_int {
    if !verbose_fd.is_null() || verbose_did_open {
        return OK;
    }
    verbose_did_open = true;
    verbose_fd = os_fopen(p_vfile, c"a".as_ptr());
    if verbose_fd.is_null() {
        // Format "E484: Can't open file <filename>" and emit as error
        let fmt_ptr = gettext(e_notopen.as_ptr());
        let fmt = std::ffi::CStr::from_ptr(fmt_ptr).to_string_lossy();
        let fname = std::ffi::CStr::from_ptr(p_vfile).to_string_lossy();
        let msg = fmt.replacen("%s", &fname, 1);
        let cmsg = std::ffi::CString::new(msg).unwrap_or_default();
        let duped = xstrdup(cmsg.as_ptr());
        emsg(duped);
        xfree(duped.cast());
        return FAIL;
    }
    OK
}

// ============================================================================
// Redirection State Functions
// ============================================================================

/// Check if redirection is temporarily disabled.
///
/// # Safety
/// Calls C accessor function.
#[no_mangle]
pub unsafe extern "C" fn rs_redir_off() -> c_int {
    c_int::from(redir_off)
}

/// Set redirection off state.
///
/// # Safety
/// Calls C mutator function.
#[no_mangle]
pub unsafe extern "C" fn rs_set_redir_off(val: c_int) {
    redir_off = val != 0;
}

/// Temporarily disable redirection.
///
/// # Safety
/// Calls C mutator function.
#[no_mangle]
pub unsafe extern "C" fn rs_redir_disable() {
    redir_off = true;
}

/// Re-enable redirection.
///
/// # Safety
/// Calls C mutator function.
#[no_mangle]
pub unsafe extern "C" fn rs_redir_enable() {
    redir_off = false;
}

/// Check if any redirection is active.
///
/// Returns true if redirecting to file, register, variable, or capture.
///
/// # Safety
/// Calls C accessor functions.
#[no_mangle]
pub unsafe extern "C" fn rs_is_redirecting() -> c_int {
    c_int::from(
        !redir_fd.is_null()
            || p_vfile_not_empty()
            || redir_reg != 0
            || redir_vname
            || !capture_ga.is_null(),
    )
}

/// Check if redirecting to a file.
///
/// # Safety
/// Calls C accessor function.
#[no_mangle]
pub unsafe extern "C" fn rs_redir_to_file() -> c_int {
    c_int::from(!redir_fd.is_null())
}

/// Check if redirecting to a register.
///
/// Returns the register number (0 if not redirecting to register).
///
/// # Safety
/// Calls C accessor function.
#[no_mangle]
pub unsafe extern "C" fn rs_redir_to_reg() -> c_int {
    redir_reg
}

/// Check if redirecting to a variable.
///
/// # Safety
/// Calls C accessor function.
#[no_mangle]
pub unsafe extern "C" fn rs_redir_to_var() -> c_int {
    c_int::from(redir_vname)
}

/// Check if capturing to ga buffer.
///
/// # Safety
/// Calls C accessor function.
#[no_mangle]
pub unsafe extern "C" fn rs_redir_capturing() -> c_int {
    c_int::from(!capture_ga.is_null())
}

/// Check if verbose file is in use.
///
/// # Safety
/// Calls C accessor function.
#[no_mangle]
pub unsafe extern "C" fn rs_verbose_file_active() -> c_int {
    c_int::from(p_vfile_not_empty())
}

// ============================================================================
// Message Silent State Helpers
// ============================================================================
//
// Note: rs_msg_silent() and rs_is_msg_silent() are defined in format.rs

/// Set the message silent counter.
///
/// # Safety
/// Calls C mutator function.
#[no_mangle]
pub unsafe extern "C" fn rs_set_msg_silent(val: c_int) {
    msg_silent = val;
}

/// Increment the message silent counter.
///
/// # Safety
/// Calls C accessor/mutator functions.
#[no_mangle]
pub unsafe extern "C" fn rs_msg_silent_enter() {
    let val = msg_silent;
    msg_silent = val + 1;
}

/// Decrement the message silent counter.
///
/// # Safety
/// Calls C accessor/mutator functions.
#[no_mangle]
pub unsafe extern "C" fn rs_msg_silent_leave() {
    let val = msg_silent;
    if val > 0 {
        msg_silent = val - 1;
    }
}

/// Check if output should be suppressed.
///
/// Returns true if silenced or redirecting and redir_off is not set.
///
/// # Safety
/// Calls C accessor functions.
#[no_mangle]
pub unsafe extern "C" fn rs_output_suppressed() -> c_int {
    let silent = msg_silent > 0;
    c_int::from(silent)
}

// ============================================================================
// Redirection Write
// ============================================================================

/// Write a string to any active redirection targets.
///
/// Mirrors the C `redir_write()` function. Writes to capture buffer, register,
/// variable, redirection file, and/or verbose file as appropriate.
///
/// The `maxlen` argument mirrors the C `ptrdiff_t maxlen`: -1 means write the
/// whole NUL-terminated string; any other value is the byte count.
///
/// # Safety
/// Accesses many global state variables; calls C functions for writing.
#[allow(clippy::cast_sign_loss)]
#[export_name = "redir_write"]
pub unsafe extern "C" fn rs_redir_write(str: *const c_char, maxlen: isize) {
    if maxlen == 0 || redir_off {
        return;
    }

    // If 'verbosefile' is set, open it for appending.
    if p_vfile_not_empty() && verbose_fd.is_null() {
        let _ = rs_verbose_open();
    }

    if redirecting() == 0 {
        return;
    }

    let s = str;
    let first_byte = *s.cast::<u8>();
    let space = c" ".as_ptr();

    // Advance to msg_col if string doesn't start with CR or NL.
    if first_byte != b'\n' && first_byte != b'\r' {
        while redir_write_cur_col < msg_col {
            if !capture_ga.is_null() {
                ga_concat_len(capture_ga, space, 1);
            }
            if redir_reg != 0 {
                write_reg_contents(redir_reg, space, 1, true);
            } else if redir_vname {
                var_redir_str(space, -1);
            } else if !redir_fd.is_null() {
                fputs(space, redir_fd);
            }
            if !verbose_fd.is_null() {
                fputs(space, verbose_fd);
            }
            redir_write_cur_col += 1;
        }
    }

    // Write string to bulk targets (capture/reg/var).
    let len: usize = if maxlen == -1 {
        std::ffi::CStr::from_ptr(str).to_bytes().len()
    } else {
        maxlen as usize
    };

    if !capture_ga.is_null() {
        ga_concat_len(capture_ga, str, len);
    }
    if redir_reg != 0 {
        #[allow(clippy::cast_possible_wrap)]
        write_reg_contents(redir_reg, s, len as isize, true);
    }
    if redir_vname {
        #[allow(clippy::cast_possible_truncation)]
        var_redir_str(s, maxlen as c_int);
    }

    // Write char-by-char to file/verbose targets, tracking column position.
    let mut p = str;
    let mut pos: isize = 0;
    loop {
        let ch = *p.cast::<u8>();
        if ch == 0 {
            break;
        }
        if maxlen >= 0 && pos >= maxlen {
            break;
        }
        if redir_reg == 0 && !redir_vname && capture_ga.is_null() && !redir_fd.is_null() {
            putc(c_int::from(ch), redir_fd);
        }
        if !verbose_fd.is_null() {
            putc(c_int::from(ch), verbose_fd);
        }
        if ch == b'\r' || ch == b'\n' {
            redir_write_cur_col = 0;
        } else if ch == b'\t' {
            redir_write_cur_col += 8 - redir_write_cur_col % 8;
        } else {
            redir_write_cur_col += 1;
        }
        p = p.add(1);
        pos += 1;
    }

    if msg_silent != 0 {
        msg_col = redir_write_cur_col;
    }
}

#[cfg(test)]
mod tests {
    // Integration tests would require mocking C functions
}
