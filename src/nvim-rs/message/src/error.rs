//! Error and warning message handling
//!
//! Provides utilities for managing error message state including
//! suppression flags, error counters, and display state.

use std::ffi::{c_int, c_long};

// C accessor declarations
extern "C" {
    static mut msg_silent: c_int;
    /// Get `emsg_off` counter
    static mut emsg_off: c_int;
    /// Set `emsg_off` counter
    static mut emsg_skip: c_int;
    /// Get `emsg_silent` counter
    static mut emsg_silent: c_int;
    /// Set `emsg_silent` counter
    /// Get `emsg_severe` flag
    static mut emsg_severe: bool;
    /// Set `emsg_severe` flag
    static mut emsg_noredir: bool;
    /// Get `did_emsg` counter
    static mut did_emsg: c_int;
    /// Set `did_emsg` counter
    /// Get `called_emsg` counter
    static mut called_emsg: c_int;
    /// Set `called_emsg` counter
    /// Get `emsg_on_display` flag
    static mut emsg_on_display: bool;
    /// Set `emsg_on_display` flag
    static mut p_debug: *mut std::ffi::c_char;
    fn vim_strchr(string: *const std::ffi::c_char, c: c_int) -> *mut std::ffi::c_char;
    static mut need_wait_return: bool;
}

/// Get the emsg_off counter (error messages disabled).
///
/// When > 0, error messages are not displayed.
///
/// # Safety
/// Calls C accessor function.
#[no_mangle]
pub unsafe extern "C" fn rs_emsg_off() -> c_int {
    emsg_off
}

/// Set the emsg_off counter.
///
/// # Safety
/// Calls C accessor function.
#[no_mangle]
pub unsafe extern "C" fn rs_set_emsg_off(val: c_int) {
    emsg_off = val;
}

/// Increment emsg_off to disable error messages.
///
/// # Safety
/// Calls C accessor function.
#[no_mangle]
pub unsafe extern "C" fn rs_emsg_off_enter() {
    let val = emsg_off;
    emsg_off = val + 1;
}

/// Decrement emsg_off to re-enable error messages.
///
/// # Safety
/// Calls C accessor function.
#[no_mangle]
pub unsafe extern "C" fn rs_emsg_off_leave() {
    let val = emsg_off;
    if val > 0 {
        emsg_off = val - 1;
    }
}

/// Check if error messages are currently disabled.
///
/// # Safety
/// Calls C accessor function.
#[no_mangle]
pub unsafe extern "C" fn rs_is_emsg_off() -> c_int {
    c_int::from(emsg_off > 0)
}

/// Get the emsg_skip counter.
///
/// When > 0, error messages are never displayed.
///
/// # Safety
/// Calls C accessor function.
#[no_mangle]
pub unsafe extern "C" fn rs_emsg_skip() -> c_int {
    emsg_skip
}

/// Set the emsg_skip counter.
///
/// # Safety
/// Calls C accessor function.
#[no_mangle]
pub unsafe extern "C" fn rs_set_emsg_skip(val: c_int) {
    emsg_skip = val;
}

/// Increment emsg_skip to skip error messages.
///
/// # Safety
/// Calls C accessor function.
#[no_mangle]
pub unsafe extern "C" fn rs_emsg_skip_enter() {
    let val = emsg_skip;
    emsg_skip = val + 1;
}

/// Decrement emsg_skip.
///
/// # Safety
/// Calls C accessor function.
#[no_mangle]
pub unsafe extern "C" fn rs_emsg_skip_leave() {
    let val = emsg_skip;
    if val > 0 {
        emsg_skip = val - 1;
    }
}

/// Get the emsg_silent counter.
///
/// When > 0, error messages are silent (not printed).
///
/// # Safety
/// Calls C accessor function.
#[no_mangle]
pub unsafe extern "C" fn rs_emsg_silent() -> c_int {
    emsg_silent
}

/// Set the emsg_silent counter.
///
/// # Safety
/// Calls C accessor function.
#[no_mangle]
pub unsafe extern "C" fn rs_set_emsg_silent(val: c_int) {
    emsg_silent = val;
}

/// Increment emsg_silent to silence error messages.
///
/// # Safety
/// Calls C accessor function.
#[no_mangle]
pub unsafe extern "C" fn rs_emsg_silent_enter() {
    let val = emsg_silent;
    emsg_silent = val + 1;
}

/// Decrement emsg_silent.
///
/// # Safety
/// Calls C accessor function.
#[no_mangle]
pub unsafe extern "C" fn rs_emsg_silent_leave() {
    let val = emsg_silent;
    if val > 0 {
        emsg_silent = val - 1;
    }
}

/// Check if error messages are currently silent.
///
/// # Safety
/// Calls C accessor function.
#[no_mangle]
pub unsafe extern "C" fn rs_is_emsg_silent() -> c_int {
    c_int::from(emsg_silent > 0)
}

/// Get the emsg_severe flag.
///
/// When true, prefer this error message over previous ones for exceptions.
///
/// # Safety
/// Calls C accessor function.
#[no_mangle]
pub unsafe extern "C" fn rs_emsg_severe() -> c_int {
    c_int::from(emsg_severe)
}

/// Set the emsg_severe flag.
///
/// # Safety
/// Calls C accessor function.
#[no_mangle]
pub unsafe extern "C" fn rs_set_emsg_severe(val: c_int) {
    emsg_severe = (val) != 0;
}

/// Get the emsg_noredir flag.
///
/// When true, don't redirect error messages.
///
/// # Safety
/// Calls C accessor function.
#[no_mangle]
pub unsafe extern "C" fn rs_emsg_noredir() -> c_int {
    c_int::from(emsg_noredir)
}

/// Set the emsg_noredir flag.
///
/// # Safety
/// Calls C accessor function.
#[no_mangle]
pub unsafe extern "C" fn rs_set_emsg_noredir(val: c_int) {
    emsg_noredir = val != 0;
}

/// Get the did_emsg counter.
///
/// Incremented by emsg() when a message is displayed or thrown.
///
/// # Safety
/// Calls C accessor function.
#[no_mangle]
pub unsafe extern "C" fn rs_did_emsg() -> c_int {
    did_emsg
}

/// Set the did_emsg counter.
///
/// # Safety
/// Calls C accessor function.
#[no_mangle]
pub unsafe extern "C" fn rs_set_did_emsg(val: c_int) {
    did_emsg = val;
}

/// Increment did_emsg.
///
/// # Safety
/// Calls C accessor function.
#[no_mangle]
pub unsafe extern "C" fn rs_inc_did_emsg() {
    let val = did_emsg;
    did_emsg = val + 1;
}

/// Check if any error message was displayed.
///
/// # Safety
/// Calls C accessor function.
#[no_mangle]
pub unsafe extern "C" fn rs_has_did_emsg() -> c_int {
    c_int::from(did_emsg > 0)
}

/// Get the called_emsg counter.
///
/// Always incremented by emsg(), even if message is not displayed.
///
/// # Safety
/// Calls C accessor function.
#[no_mangle]
pub unsafe extern "C" fn rs_called_emsg() -> c_int {
    called_emsg
}

/// Set the called_emsg counter.
///
/// # Safety
/// Calls C accessor function.
#[no_mangle]
pub unsafe extern "C" fn rs_set_called_emsg(val: c_int) {
    called_emsg = val;
}

/// Increment called_emsg.
///
/// # Safety
/// Calls C accessor function.
#[no_mangle]
pub unsafe extern "C" fn rs_inc_called_emsg() {
    let val = called_emsg;
    called_emsg = val + 1;
}

/// Get the emsg_on_display flag.
///
/// True if there is an error message currently displayed.
///
/// # Safety
/// Calls C accessor function.
#[no_mangle]
pub unsafe extern "C" fn rs_emsg_on_display_get() -> c_int {
    c_int::from(emsg_on_display)
}

/// Set the emsg_on_display flag.
///
/// # Safety
/// Calls C accessor function.
#[no_mangle]
pub unsafe extern "C" fn rs_emsg_on_display_set(val: c_int) {
    emsg_on_display = (val) != 0;
}

/// Check if error messages should not be shown right now.
///
/// Returns true if:
/// - emsg_off is set and debug mode is not 'm' or 't', or
/// - emsg_skip is set
///
/// This matches the C function `emsg_not_now()` exactly.
///
/// # Safety
/// Calls C accessor functions.
#[no_mangle]
pub unsafe extern "C" fn rs_emsg_not_now() -> c_int {
    let off = emsg_off;
    let skip = emsg_skip;

    // If emsg_off > 0 and debug doesn't contain 'm' or 't', skip messages
    // If emsg_skip > 0, always skip messages
    let skip_due_to_off = off > 0
        && vim_strchr(p_debug, c_int::from(b'm')).is_null()
        && vim_strchr(p_debug, c_int::from(b't')).is_null();

    c_int::from(skip_due_to_off || skip > 0)
}

/// Simplified check if error messages should not be shown.
///
/// Only checks emsg_skip, not the full emsg_not_now logic.
/// Use `rs_emsg_not_now` for the full check.
///
/// # Safety
/// Calls C accessor functions.
#[no_mangle]
pub unsafe extern "C" fn rs_emsg_not_now_simple() -> c_int {
    // Simplified version - only check emsg_skip
    c_int::from(emsg_skip > 0)
}

/// Check if any error suppression is active.
///
/// Returns true if emsg_off, emsg_skip, or emsg_silent is > 0.
///
/// # Safety
/// Calls C accessor functions.
#[no_mangle]
pub unsafe extern "C" fn rs_is_emsg_suppressed() -> c_int {
    let off = emsg_off;
    let skip = emsg_skip;
    let silent = emsg_silent;
    c_int::from(off > 0 || skip > 0 || silent > 0)
}

/// Save error message state for later restoration.
///
/// Returns a packed state value containing emsg_off, emsg_skip, emsg_silent.
///
/// # Safety
/// Calls C accessor functions.
#[no_mangle]
pub unsafe extern "C" fn rs_emsg_save_state() -> c_int {
    // Pack the three counters into a single int
    // Each gets 10 bits (max 1023)
    let off = emsg_off & 0x3FF;
    let skip = emsg_skip & 0x3FF;
    let silent = emsg_silent & 0x3FF;
    off | (skip << 10) | (silent << 20)
}

/// Restore error message state from a saved value.
///
/// # Safety
/// Calls C accessor functions.
#[no_mangle]
pub unsafe extern "C" fn rs_emsg_restore_state(state: c_int) {
    let off = state & 0x3FF;
    let skip = (state >> 10) & 0x3FF;
    let silent = (state >> 20) & 0x3FF;
    emsg_off = off;
    emsg_skip = skip;
    emsg_silent = silent;
}

// Additional C accessor declarations for warning functionality
extern "C" {
    /// `did_emsg_syntax` — direct access to C global
    static mut did_emsg_syntax: bool;
}

/// Check if did_emsg was set because of a syntax error.
///
/// # Safety
/// Calls C accessor function.
#[no_mangle]
pub unsafe extern "C" fn rs_did_emsg_syntax() -> c_int {
    c_int::from(did_emsg_syntax)
}

/// Check if error should be output now.
///
/// Returns false (0) if error messages should not be shown because:
/// - emsg_off is set OR
/// - emsg_skip is set
///
/// # Safety
/// Calls C accessor functions.
#[no_mangle]
pub unsafe extern "C" fn rs_emsg_now() -> c_int {
    let off = emsg_off;
    let skip = emsg_skip;
    c_int::from(off == 0 && skip == 0)
}

/// Reset error counters for a fresh start.
///
/// Clears did_emsg, called_emsg, and emsg_on_display.
///
/// # Safety
/// Calls C accessor functions.
#[no_mangle]
pub unsafe extern "C" fn rs_emsg_reset_counters() {
    did_emsg = 0;
    called_emsg = 0;
    emsg_on_display = false;
}

/// Combined check for all error suppression.
///
/// Returns the total "depth" of error suppression (sum of counters).
///
/// # Safety
/// Calls C accessor functions.
#[no_mangle]
pub unsafe extern "C" fn rs_emsg_suppression_depth() -> c_int {
    emsg_off + emsg_skip + emsg_silent
}

/// Last sourcing line number (replaces C static last_sourcing_lnum)
#[no_mangle]
pub static mut last_sourcing_lnum: c_int = 0;

/// Last sourcing name (replaces C static last_sourcing_name in message.c)
#[no_mangle]
pub static mut last_sourcing_name: *mut std::ffi::c_char = std::ptr::null_mut();

// ============================================================================
// Phase 423: Error Message Output Functions
// ============================================================================

extern "C" {
    // last_sourcing_lnum/last_sourcing_name: Rust-owned statics (error.rs)
    static mut no_wait_return: c_int;
    fn msg_putchar_hl(c: c_int, hl_id: c_int);
    fn redirecting() -> c_int;
    fn strcmp(s1: *const std::ffi::c_char, s2: *const std::ffi::c_char) -> c_int;

    // Accessors for emsg_multiline implementation
    fn cause_errthrow(
        s: *const std::ffi::c_char,
        multiline: bool,
        severe: bool,
        ignore: *mut bool,
    ) -> bool;
    static mut emsg_assert_fails_msg: *mut std::ffi::c_char;
    static mut emsg_assert_fails_lnum: c_long;
    static mut emsg_assert_fails_context: *mut std::ffi::c_char;
    fn nvim_get_sourcing_name() -> *const std::ffi::c_char;
    fn nvim_get_sourcing_lnum() -> c_int;
    fn xstrdup(s: *const std::ffi::c_char) -> *mut std::ffi::c_char;
    fn set_vim_var_string(idx: c_int, val: *const std::ffi::c_char, len: c_int);
    fn redir_write(str_: *const std::ffi::c_char, maxlen: isize);
    fn estack_sfile(which: c_int) -> *mut std::ffi::c_char;
    fn gettext(s: *const std::ffi::c_char) -> *const std::ffi::c_char;
    static mut ex_exitval: c_int;
    static mut cmd_silent: bool;
    fn nvim_inc_global_busy();
    static mut p_eb: c_int;
    #[link_name = "beep_flush"]
    fn nvim_beep_flush();
    fn flush_buffers(flush_typeahead: c_int); // FLUSH_MINIMAL = 0
    static mut msg_nowait: bool;
    static mut msg_scroll: c_int;
    static mut msg_ext_skip_flush: bool;
}

// These are also declared elsewhere; allow the type-compatible clashes.
#[allow(clashing_extern_declarations)]
extern "C" {
    fn nvim_get_in_assert_fails() -> c_int;
    fn nvim_get_global_busy() -> c_int;
    static mut msg_scrolled: c_int;
    fn xfree(ptr: *mut std::ffi::c_void);
}

use std::ffi::c_char;

/// Display error message with full control over kind, highlight, and multiline.
///
/// This is the core error display function. It handles:
/// - Exception throwing via cause_errthrow()
/// - assert_fails() tracking
/// - v:errmsg setting
/// - Silent error redirection
/// - Source info display via msg_source()
/// - Message display via msg_keep()
///
/// # Arguments
/// * `s` - The error message string (NUL-terminated)
/// * `kind` - Message kind for ext_messages (e.g., "emsg")
/// * `hl_id` - Highlight group ID (typically HLF_E for errors)
/// * `multiline` - If true, handle embedded newlines specially
///
/// # Returns
/// * `true` (1) if wait_return() was not called
/// * `false` (0) if wait_return() was called
///
/// # Safety
/// - `s` and `kind` must be valid NUL-terminated C strings
#[export_name = "emsg_multiline"]
#[must_use]
pub unsafe extern "C" fn rs_emsg_multiline(
    s: *const c_char,
    kind: *const c_char,
    hl_id: c_int,
    multiline: c_int,
) -> c_int {
    // Skip this if not giving error messages at the moment.
    if rs_emsg_not_now() != 0 {
        return 1;
    }

    let called = called_emsg;
    called_emsg = called + 1;

    // If "emsg_severe" is true: When an error exception is to be thrown,
    // prefer this message over previous messages for the same command.
    let severe = c_int::from(emsg_severe);
    emsg_severe = false;

    let off = emsg_off;
    let debug_t = !vim_strchr(p_debug, c_int::from(b't')).is_null();
    if off == 0 || debug_t {
        // Cause a throw of an error exception if appropriate.
        let mut ignore: bool = false;
        // SAFETY: ignore is a local variable, valid for the duration of the call.
        if cause_errthrow(
            s,
            multiline != 0,
            severe != 0,
            std::ptr::addr_of_mut!(ignore),
        ) {
            if !ignore {
                let did = did_emsg;
                did_emsg = did + 1;
            }
            return 1;
        }

        if nvim_get_in_assert_fails() != 0 && emsg_assert_fails_msg.is_null() {
            emsg_assert_fails_msg = xstrdup(s);
            emsg_assert_fails_lnum = c_long::from(nvim_get_sourcing_lnum());
            xfree(emsg_assert_fails_context.cast::<std::ffi::c_void>());
            let sname = nvim_get_sourcing_name();
            let ctx: *const c_char = if sname.is_null() { c"".as_ptr() } else { sname };
            emsg_assert_fails_context = xstrdup(ctx);
        }

        // set "v:errmsg", also when using ":silent! cmd"
        set_vim_var_string(3, s, -1); // VV_ERRMSG = 3

        // When using ":silent! cmd" ignore error messages.
        // But do write it to the redirection file.
        if emsg_silent != 0 {
            if !emsg_noredir {
                crate::output_core::rs_msg_start();
                let p = emsg_source_string();
                if !p.is_null() {
                    let p_len = libc_strlen(p.cast());
                    // Append newline then write (p_len + 1 bytes including newline)
                    // SAFETY: newline '\n' = 10, which fits in i8 without wrap.
                    #[allow(clippy::cast_possible_wrap)]
                    p.add(p_len).write(10i8); // '\n'
                    redir_write(p, isize::try_from(p_len + 1).unwrap_or(isize::MAX));
                    xfree(p.cast());
                }
                let p = emsg_lnum_string();
                if !p.is_null() {
                    let p_len = libc_strlen(p.cast());
                    #[allow(clippy::cast_possible_wrap)]
                    p.add(p_len).write(10i8); // '\n'
                    redir_write(p, isize::try_from(p_len + 1).unwrap_or(isize::MAX));
                    xfree(p.cast());
                }
                let s_len = libc_strlen(s.cast());
                redir_write(s, isize::try_from(s_len).unwrap_or(isize::MAX));
            }
            return 1;
        }

        // Log editor errors as INFO (no Rust equivalent of ILOG/DLOG; skip logging)

        ex_exitval = 1;

        // Reset msg_silent, an error causes messages to be switched back on.
        msg_silent = 0;
        cmd_silent = false;

        if nvim_get_global_busy() != 0 {
            // break :global command
            nvim_inc_global_busy();
        }

        if p_eb != 0 {
            nvim_beep_flush(); // also includes flush_buffers()
        } else {
            flush_buffers(0); // FLUSH_MINIMAL = 0, flush internal buffers
        }

        let did = did_emsg;
        did_emsg = did + 1; // flag for DoOneCmd()
    }

    emsg_on_display = true; // remember there is an error message
    if msg_scrolled != 0 {
        need_wait_return = true; // needed in case emsg() is called after
                                 // wait_return() has reset need_wait_return
    }
    crate::display::rs_msg_ext_set_kind(kind);

    // Display name and line number for the source of the error.
    msg_scroll = 1;
    let save_skip_flush = msg_ext_skip_flush;
    msg_ext_skip_flush = true;
    rs_msg_source(hl_id);

    // Display the error message itself.
    msg_nowait = false; // Wait for this msg.
    let rv = crate::output_core::msg_keep_impl(s, hl_id, 0, multiline);
    msg_ext_skip_flush = save_skip_flush;
    rv
}

/// Compute the length of a NUL-terminated byte string.
///
/// # Safety
/// `s` must point to a valid NUL-terminated sequence of bytes.
const unsafe fn libc_strlen(s: *const u8) -> usize {
    let mut len = 0;
    while unsafe { *s.add(len) } != 0 {
        len += 1;
    }
    len
}

/// Display a simple error message.
///
/// This is the standard function for displaying error messages to the user.
/// It uses the default error highlight (HLF_E) and "emsg" kind.
///
/// Equivalent to C's `emsg()` function.
///
/// # Arguments
/// * `s` - The error message string (NUL-terminated)
///
/// # Returns
/// * `true` (1) if wait_return() was not called
/// * `false` (0) if wait_return() was called
///
/// # Safety
/// - `s` must be a valid NUL-terminated C string
#[export_name = "emsg"]
#[must_use]
pub unsafe extern "C" fn rs_emsg(s: *const c_char) -> c_int {
    static KIND: &[u8] = b"emsg\0";
    rs_emsg_multiline(s, KIND.as_ptr().cast::<c_char>(), HLF_E, 0)
}

/// Display an error message with kind and highlight.
///
/// Full version of emsg with all options. Used for multiline error
/// messages and special highlighting.
///
/// # Arguments
/// * `s` - The error message string (NUL-terminated)
/// * `kind` - Message kind for ext_messages (e.g., "emsg")
/// * `hl_id` - Highlight group ID (typically HLF_E for errors)
/// * `multiline` - If true, handle embedded newlines specially
///
/// # Returns
/// * `true` (1) if wait_return() was not called
/// * `false` (0) if wait_return() was called
///
/// # Safety
/// - `s` and `kind` must be valid NUL-terminated C strings
#[no_mangle]
pub unsafe extern "C" fn rs_emsg_multiline_full(
    s: *const c_char,
    kind: *const c_char,
    hl_id: c_int,
    multiline: c_int,
) -> c_int {
    rs_emsg_multiline(s, kind, hl_id, multiline)
}

/// Display an internal error message.
///
/// For internal Neovim errors that shouldn't normally occur.
/// Like emsg() but skips when error messages are suppressed.
///
/// # Arguments
/// * `s` - The error message string (NUL-terminated)
///
/// # Safety
/// - `s` must be a valid NUL-terminated C string
#[export_name = "iemsg"]
pub unsafe extern "C" fn rs_iemsg(s: *const c_char) {
    if rs_emsg_not_now() != 0 {
        return;
    }
    let _ = rs_emsg(s);
    // Note: ABORT_ON_INTERNAL_ERROR path omitted (fuzzing builds only)
}

// Static recursion guard for msg_source
static MSG_SOURCE_RECURSIVE: std::sync::atomic::AtomicBool =
    std::sync::atomic::AtomicBool::new(false);

/// HLF_N: LineNr highlight face (for msg_source)
const HLF_N_LINE_NR: c_int = 12;

/// Display source info before an error message.
///
/// Shows the script name and line number where the error occurred.
/// Remembers the last source shown so it is only displayed when it changes.
///
/// # Arguments
/// * `hl_id` - Highlight group ID for the source info
///
/// # Safety
/// Calls C accessor functions.
#[export_name = "msg_source"]
pub unsafe extern "C" fn rs_msg_source(hl_id: c_int) {
    use std::sync::atomic::Ordering;

    // Bail out if something called here causes an error.
    if MSG_SOURCE_RECURSIVE
        .compare_exchange(false, true, Ordering::AcqRel, Ordering::Relaxed)
        .is_err()
    {
        return;
    }

    no_wait_return += 1;

    let p = emsg_source_string();
    if !p.is_null() {
        msg_scroll = 1; // this will take more than one line
        let _ = crate::output_core::rs_msg(p, hl_id);
        xfree(p.cast());
    }

    let p = emsg_lnum_string();
    if !p.is_null() {
        let _ = crate::output_core::rs_msg(p, HLF_N_LINE_NR);
        xfree(p.cast());
        last_sourcing_lnum = nvim_get_sourcing_lnum(); // only once for each line
    }

    // remember the last sourcing name printed, also when it's empty
    let sourcing_name = nvim_get_sourcing_name();
    let sourcing_is_null = sourcing_name.is_null();
    let other_name = !sourcing_is_null
        && (last_sourcing_name.is_null() || strcmp(sourcing_name, last_sourcing_name) != 0);
    if sourcing_is_null || other_name {
        // update last_sourcing_name
        xfree(last_sourcing_name.cast());
        last_sourcing_name = if sourcing_name.is_null() {
            std::ptr::null_mut()
        } else {
            xstrdup(sourcing_name)
        };
        if !sourcing_is_null && redirecting() == 0 {
            msg_putchar_hl(c_int::from(b'\n'), hl_id);
        }
    }

    if no_wait_return > 0 {
        no_wait_return -= 1;
    }

    MSG_SOURCE_RECURSIVE.store(false, Ordering::Release);
}

/// Reset the last sourcing info.
///
/// Clears the cached source name/line so it will be
/// displayed again for the next error.
///
/// Equivalent to the C function `reset_last_sourcing()`.
///
/// # Safety
/// Calls C accessor functions that manage allocated memory.
#[export_name = "reset_last_sourcing"]
pub unsafe extern "C" fn rs_reset_last_sourcing() {
    xfree(last_sourcing_name.cast());
    last_sourcing_name = std::ptr::null_mut();
    last_sourcing_lnum = 0;
}

// ============================================================================
// Error Message Constants
// ============================================================================

/// Highlight face for error messages (HLF_E)
pub const HLF_E: c_int = 6;

/// Highlight face for warning messages (HLF_W)
pub const HLF_W: c_int = 26;

/// Internal error message prefix
pub const IEMSG_PREFIX: &[u8] = b"internal error: \0";

// ============================================================================
// Convenience Functions
// ============================================================================

/// Display an error with multiline handling.
///
/// Convenience wrapper for multiline error messages with standard error highlight.
///
/// # Arguments
/// * `s` - The error message string
/// * `kind` - Message kind for ext_messages
///
/// # Returns
/// Result from `rs_emsg_multiline_full`
///
/// # Safety
/// - `s` and `kind` must be valid NUL-terminated C strings
#[no_mangle]
pub unsafe extern "C" fn rs_emsg_multiline_hl(s: *const c_char, kind: *const c_char) -> c_int {
    rs_emsg_multiline(s, kind, HLF_E, 1)
}

/// Check if error output is currently possible.
///
/// Returns true if errors can be displayed right now.
/// Uses the simple check (emsg_off and emsg_skip).
///
/// # Safety
/// Calls C accessor functions.
#[no_mangle]
pub unsafe extern "C" fn rs_emsg_can_output() -> c_int {
    // Use the simple check - same as rs_emsg_now
    let off = emsg_off;
    let skip = emsg_skip;
    c_int::from(off == 0 && skip == 0)
}

/// Begin an error message sequence.
///
/// Increments error counters to track that an error is being processed.
///
/// # Safety
/// Calls C accessor functions.
#[no_mangle]
pub unsafe extern "C" fn rs_emsg_begin() {
    rs_inc_called_emsg();
}

/// End an error message sequence.
///
/// Sets the emsg_on_display flag to indicate an error was shown.
///
/// # Safety
/// Calls C accessor functions.
#[no_mangle]
pub unsafe extern "C" fn rs_emsg_end() {
    emsg_on_display = true;
}

// ESTACK_NONE constant (from runtime_defs.h)
const ESTACK_NONE_VAL: c_int = 0;

/// Rust implementation of get_emsg_source().
///
/// Returns an xmalloc-allocated "Error in <source>:" string, or NULL.
///
/// # Safety
/// Calls C accessor functions and manages allocated memory.
unsafe fn emsg_source_string() -> *mut c_char {
    let sourcing_name = nvim_get_sourcing_name();
    if sourcing_name.is_null() {
        return std::ptr::null_mut();
    }
    // Check if this is a different source name than last displayed.
    let other_name = last_sourcing_name.is_null() || strcmp(sourcing_name, last_sourcing_name) != 0;
    if !other_name {
        return std::ptr::null_mut();
    }
    // Get the full sfile representation (may be NULL).
    let sname = estack_sfile(ESTACK_NONE_VAL);
    let effective: *const c_char = if sname.is_null() {
        sourcing_name
    } else {
        sname.cast()
    };
    // Translate and format "Error in %s:".
    let fmt_ptr = gettext(c"Error in %s:".as_ptr());
    let fmt = std::ffi::CStr::from_ptr(fmt_ptr).to_string_lossy();
    let name = std::ffi::CStr::from_ptr(effective).to_string_lossy();
    let result = fmt.replacen("%s", &name, 1);
    if !sname.is_null() {
        xfree(sname.cast());
    }
    let cstr = std::ffi::CString::new(result).unwrap_or_default();
    xstrdup(cstr.as_ptr())
}

/// Export get_emsg_source for C callers.
///
/// # Safety
/// Calls C accessor functions and manages allocated memory.
#[must_use]
#[export_name = "get_emsg_source"]
pub unsafe extern "C" fn rs_get_emsg_source() -> *mut c_char {
    emsg_source_string()
}

/// Rust implementation of get_emsg_lnum().
///
/// Returns an xmalloc-allocated "line   42:" string, or NULL.
///
/// # Safety
/// Calls C accessor functions and manages allocated memory.
unsafe fn emsg_lnum_string() -> *mut c_char {
    let sourcing_name = nvim_get_sourcing_name();
    if sourcing_name.is_null() {
        return std::ptr::null_mut();
    }
    let sourcing_lnum = nvim_get_sourcing_lnum();
    // Check if this is a different source name than last displayed.
    let other_name = last_sourcing_name.is_null() || strcmp(sourcing_name, last_sourcing_name) != 0;
    // Only show lnum if name or lnum changed, and lnum is nonzero.
    if sourcing_lnum == 0 || (!other_name && sourcing_lnum == last_sourcing_lnum) {
        return std::ptr::null_mut();
    }
    // Translate and format "line %4d:".
    let fmt_ptr = gettext(c"line %4d:".as_ptr());
    let fmt = std::ffi::CStr::from_ptr(fmt_ptr).to_string_lossy();
    // Replace the printf-style %4d with the right-justified number.
    let num_str = format!("{sourcing_lnum:>4}");
    let result = fmt.replacen("%4d", &num_str, 1);
    let cstr = std::ffi::CString::new(result).unwrap_or_default();
    xstrdup(cstr.as_ptr())
}

/// Export get_emsg_lnum for C callers.
///
/// # Safety
/// Calls C accessor functions and manages allocated memory.
#[must_use]
#[export_name = "get_emsg_lnum"]
pub unsafe extern "C" fn rs_get_emsg_lnum() -> *mut c_char {
    emsg_lnum_string()
}

// ============================================================================
// Phase 2: emsg_invreg and internal_error — migrated from message.c
// ============================================================================

extern "C" {
    /// Wraps transchar_buf(NULL, c) — returns pointer to static transchar_charbuf.
    fn nvim_transchar_null_buf(c: c_int) -> *const c_char;
    /// Wraps siemsg(_(e_intern2), where) — needed because siemsg is variadic.
    fn nvim_siemsg_intern2(where_: *const c_char);
}

/// Print "invalid register name" error.
///
/// Replaces C `emsg_invreg(int name)`.
///
/// # Safety
/// Calls C transchar_buf and emsg.
#[export_name = "emsg_invreg"]
pub unsafe extern "C" fn rs_emsg_invreg(name: c_int) {
    let ch = nvim_transchar_null_buf(name);
    // Format: E354: Invalid register name: '<ch>'
    // Use a static format and call emsg with pre-formatted string
    // We avoid gettext here since emsg_invreg is a one-shot internal call.
    let mut buf = [0u8; 256];
    let ch_str = std::ffi::CStr::from_ptr(ch).to_bytes();
    let prefix = b"E354: Invalid register name: '";
    let suffix = b"'";
    let len = prefix.len().min(buf.len());
    buf[..len].copy_from_slice(&prefix[..len]);
    let mut pos = len;
    let ch_len = ch_str.len().min(buf.len() - pos - suffix.len() - 1);
    buf[pos..pos + ch_len].copy_from_slice(&ch_str[..ch_len]);
    pos += ch_len;
    let suf_len = suffix.len().min(buf.len() - pos - 1);
    buf[pos..pos + suf_len].copy_from_slice(&suffix[..suf_len]);
    pos += suf_len;
    buf[pos] = 0;
    let _ = rs_emsg(buf.as_ptr().cast::<c_char>());
}

/// Give an "Internal error" message.
///
/// Replaces C `internal_error(const char *where)`.
///
/// # Safety
/// - `where_` must be a valid NUL-terminated C string.
#[export_name = "internal_error"]
pub unsafe extern "C" fn rs_internal_error(where_: *const c_char) {
    nvim_siemsg_intern2(where_);
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_state_packing() {
        // Test that state packing/unpacking is reversible
        let off = 5;
        let skip = 3;
        let silent = 7;
        let packed = off | (skip << 10) | (silent << 20);
        assert_eq!(packed & 0x3FF, 5);
        assert_eq!((packed >> 10) & 0x3FF, 3);
        assert_eq!((packed >> 20) & 0x3FF, 7);
    }

    #[test]
    fn test_max_packed_values() {
        // Test edge cases for packing - max values
        let off = 1023; // 0x3FF
        let skip = 1023;
        let silent = 1023;
        let packed = off | (skip << 10) | (silent << 20);
        assert_eq!(packed & 0x3FF, 1023);
        assert_eq!((packed >> 10) & 0x3FF, 1023);
        assert_eq!((packed >> 20) & 0x3FF, 1023);
    }

    #[test]
    fn test_zero_packed_values() {
        let packed = 0;
        assert_eq!(packed & 0x3FF, 0);
        assert_eq!((packed >> 10) & 0x3FF, 0);
        assert_eq!((packed >> 20) & 0x3FF, 0);
    }
}
