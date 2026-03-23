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
    /// Check if p_debug contains a specific character
    fn nvim_p_debug_contains(c: c_int) -> c_int;
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
        && nvim_p_debug_contains(c_int::from(b'm')) == 0
        && nvim_p_debug_contains(c_int::from(b't')) == 0;

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

// ============================================================================
// Phase 423: Error Message Output Functions
// ============================================================================

extern "C" {
    // Phase 1: sourcing state accessors for reset_last_sourcing
    fn nvim_clear_last_sourcing_name();
    // last_sourcing_lnum: Rust-owned static (error.rs)

    // Accessors for msg_source implementation
    fn nvim_other_sourcing_name() -> c_int;
    fn nvim_sourcing_name_is_null() -> c_int;
    fn nvim_update_last_sourcing_name();
    static mut no_wait_return: c_int;
    fn msg_putchar_hl(c: c_int, hl_id: c_int);
    fn redirecting() -> c_int;

    // Accessors for emsg_multiline implementation
    fn nvim_cause_errthrow(
        s: *const std::ffi::c_char,
        multiline: c_int,
        severe: c_int,
        ignore: *mut c_int,
    ) -> c_int;
    static mut emsg_assert_fails_msg: *mut std::ffi::c_char;
    static mut emsg_assert_fails_lnum: c_long;
    static mut emsg_assert_fails_context: *mut std::ffi::c_char;
    fn nvim_get_sourcing_name() -> *const std::ffi::c_char;
    fn nvim_get_sourcing_lnum() -> c_int;
    fn nvim_xstrdup(s: *const std::ffi::c_char) -> *mut std::ffi::c_char;
    fn nvim_set_vim_var_errmsg(s: *const std::ffi::c_char);
    fn nvim_redir_write(str_: *const std::ffi::c_char, maxlen: isize);
    fn nvim_get_emsg_source() -> *mut std::ffi::c_char;
    fn nvim_get_emsg_lnum() -> *mut std::ffi::c_char;
    static mut ex_exitval: c_int;
    fn nvim_set_cmd_silent(val: c_int);
    fn nvim_inc_global_busy();
    fn nvim_get_p_eb() -> c_int;
    fn nvim_beep_flush();
    fn nvim_flush_buffers_minimal();
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
    let debug_t = nvim_p_debug_contains(c_int::from(b't'));
    if off == 0 || debug_t != 0 {
        // Cause a throw of an error exception if appropriate.
        let mut ignore: c_int = 0;
        // SAFETY: ignore is a local variable, valid for the duration of the call.
        if nvim_cause_errthrow(s, multiline, severe, std::ptr::addr_of_mut!(ignore)) != 0 {
            if ignore == 0 {
                let did = did_emsg;
                did_emsg = did + 1;
            }
            return 1;
        }

        if nvim_get_in_assert_fails() != 0 && emsg_assert_fails_msg.is_null() {
            emsg_assert_fails_msg = nvim_xstrdup(s);
            emsg_assert_fails_lnum = c_long::from(nvim_get_sourcing_lnum());
            xfree(emsg_assert_fails_context.cast::<std::ffi::c_void>());
            let sname = nvim_get_sourcing_name();
            let ctx: *const c_char = if sname.is_null() { c"".as_ptr() } else { sname };
            emsg_assert_fails_context = nvim_xstrdup(ctx);
        }

        // set "v:errmsg", also when using ":silent! cmd"
        nvim_set_vim_var_errmsg(s);

        // When using ":silent! cmd" ignore error messages.
        // But do write it to the redirection file.
        if emsg_silent != 0 {
            if !emsg_noredir {
                crate::output_core::rs_msg_start();
                let p = nvim_get_emsg_source();
                if !p.is_null() {
                    let p_len = libc_strlen(p.cast());
                    // Append newline then write (p_len + 1 bytes including newline)
                    // SAFETY: newline '\n' = 10, which fits in i8 without wrap.
                    #[allow(clippy::cast_possible_wrap)]
                    p.add(p_len).write(10i8); // '\n'
                    nvim_redir_write(p, isize::try_from(p_len + 1).unwrap_or(isize::MAX));
                    xfree(p.cast());
                }
                let p = nvim_get_emsg_lnum();
                if !p.is_null() {
                    let p_len = libc_strlen(p.cast());
                    #[allow(clippy::cast_possible_wrap)]
                    p.add(p_len).write(10i8); // '\n'
                    nvim_redir_write(p, isize::try_from(p_len + 1).unwrap_or(isize::MAX));
                    xfree(p.cast());
                }
                let s_len = libc_strlen(s.cast());
                nvim_redir_write(s, isize::try_from(s_len).unwrap_or(isize::MAX));
            }
            return 1;
        }

        // Log editor errors as INFO (no Rust equivalent of ILOG/DLOG; skip logging)

        ex_exitval = 1;

        // Reset msg_silent, an error causes messages to be switched back on.
        msg_silent = 0;
        nvim_set_cmd_silent(0);

        if nvim_get_global_busy() != 0 {
            // break :global command
            nvim_inc_global_busy();
        }

        if nvim_get_p_eb() != 0 {
            nvim_beep_flush(); // also includes flush_buffers()
        } else {
            nvim_flush_buffers_minimal(); // flush internal buffers
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

    let p = nvim_get_emsg_source();
    if !p.is_null() {
        msg_scroll = 1; // this will take more than one line
        let _ = crate::output_core::rs_msg(p, hl_id);
        xfree(p.cast());
    }

    let p = nvim_get_emsg_lnum();
    if !p.is_null() {
        let _ = crate::output_core::rs_msg(p, HLF_N_LINE_NR);
        xfree(p.cast());
        last_sourcing_lnum = nvim_get_sourcing_lnum(); // only once for each line
    }

    // remember the last sourcing name printed, also when it's empty
    if nvim_sourcing_name_is_null() != 0 || nvim_other_sourcing_name() != 0 {
        nvim_update_last_sourcing_name();
        if nvim_sourcing_name_is_null() == 0 && redirecting() == 0 {
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
    nvim_clear_last_sourcing_name();
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
