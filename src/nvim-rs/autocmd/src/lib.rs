//! Autocommand state checking for Neovim
//!
//! This module provides Rust implementations for checking autocommand state.

#![warn(clippy::all, clippy::pedantic)]
#![allow(clippy::missing_safety_doc)]
#![allow(clippy::doc_markdown)]
#![allow(clippy::cast_possible_wrap)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::missing_const_for_fn)]

pub mod event;
pub mod group;
pub mod pattern;

use std::ffi::{c_char, c_void};
use std::os::raw::c_int;

/// Opaque handle to a Neovim window (`win_T*`).
#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WinHandle(*mut c_void);

// Mode constants from state_defs.h
const MODE_NORMAL: c_int = 0x01;
const MODE_NORMAL_BUSY: c_int = 0x1000 | MODE_NORMAL;
const MODE_INSERT: c_int = 0x10;

// Event constants from auevents_enum.generated.h
const EVENT_CURSORHOLD: c_int = 37;
const EVENT_CURSORHOLDI: c_int = 38;
const NUM_EVENTS: c_int = 141;

// Buffer-local pattern constants from autocmd.h
const BUFLOCAL_PAT_LEN: usize = 25;

// C accessors for static data
extern "C" {
    fn nvim_get_autocmd_blocked() -> c_int;
    fn nvim_get_event_name(event: c_int) -> *const c_char;
    fn nvim_get_autocmds_count(event: c_int) -> usize;

    // Accessors for aucmd_win array
    fn nvim_get_aucmd_win_count() -> c_int;
    fn nvim_aucmd_win_used(idx: c_int) -> c_int;
    fn nvim_aucmd_win_get_win(idx: c_int) -> WinHandle;

    // From event crate - get the real editor state
    fn rs_get_real_state() -> c_int;

    // Accessors for trigger_cursorhold
    fn nvim_get_did_cursorhold() -> c_int;
    fn nvim_get_reg_recording() -> c_int;
    fn nvim_get_typebuf_len() -> c_int;

    // From insexpand crate - check if completion is active
    fn rs_ins_compl_active() -> c_int;

    // Buffer/autocmd state accessors (Phase 1)
    fn nvim_get_curbuf_fnum() -> c_int;
    fn nvim_get_curbuf_handle() -> c_int;
    fn nvim_get_autocmd_bufnr() -> c_int;

    // Event name resolution (Phase 1)
    fn nvim_event_name2nr(start: *const c_char, len: usize) -> c_int;

    // Phase 4: Autocmd deletion + cleanup accessors
    fn nvim_autocmd_del_at(event: c_int, idx: usize);
    fn nvim_autocmd_pat_is_null(event: c_int, idx: usize) -> c_int;
    fn nvim_autocmd_get_pat_group(event: c_int, idx: usize) -> c_int;
    fn nvim_autocmd_get_pat_buflocal_nr(event: c_int, idx: usize) -> c_int;
    fn nvim_autocmd_compact_event(event: c_int);
    fn nvim_get_au_need_clean() -> c_int;
    fn nvim_set_au_need_clean(val: c_int);
    fn nvim_get_autocmd_busy() -> bool;
    fn nvim_apc_invalidate_bufnr(bufnr: c_int);
    fn nvim_verbose_buflocal_remove(event: c_int, bufnr: c_int);

    // Phase 5: :augroup command + arg parsing accessors
    fn nvim_autocmd_set_current_augroup(val: c_int);
    fn nvim_autocmd_list_group_names();
    fn nvim_autocmd_emsg(msg: *const c_char);
    fn nvim_autocmd_semsg_str(fmt: *const c_char, arg: *const c_char);
    fn nvim_autocmd_get_e_argreq() -> *const c_char;
    fn nvim_autocmd_get_e216_no_such_event() -> *const c_char;
    fn nvim_autocmd_get_e216_no_such_group_or_event() -> *const c_char;
    fn nvim_autocmd_get_e215() -> *const c_char;
    fn nvim_autocmd_get_e_duparg2() -> *const c_char;
    fn nvim_skipwhite(p: *const c_char) -> *mut c_char;
    fn nvim_autocmd_xmemdupz(src: *const c_char, len: usize) -> *mut c_char;
    fn nvim_autocmd_xfree(ptr: *mut c_char);

    // Rust functions from other modules (called via FFI)
    fn rs_augroup_find(name: *const c_char) -> c_int;
    fn rs_augroup_add(name: *const c_char) -> c_int;
}

// Static "Unknown" string for invalid events
static UNKNOWN_EVENT: &[u8] = b"Unknown\0";

/// Check if autocommands are blocked.
///
/// Returns true if autocmd_blocked != 0.
#[no_mangle]
pub unsafe extern "C" fn rs_is_autocmd_blocked() -> c_int {
    c_int::from(nvim_get_autocmd_blocked() != 0)
}

/// Return the name for an event.
///
/// Returns "Unknown" for invalid or out-of-range events.
///
/// # Safety
/// The returned pointer is valid for the lifetime of the program (static data).
#[no_mangle]
pub unsafe extern "C" fn rs_event_nr2name(event: c_int, num_events: c_int) -> *const c_char {
    if event >= 0 && event < num_events {
        let name = nvim_get_event_name(event);
        if !name.is_null() {
            return name;
        }
    }
    UNKNOWN_EVENT.as_ptr().cast()
}

/// Check if there are any autocommands registered for an event.
///
/// Returns 1 if the event has at least one autocommand, 0 otherwise.
#[no_mangle]
pub unsafe extern "C" fn rs_has_event(event: c_int, num_events: c_int) -> c_int {
    if event >= 0 && event < num_events {
        c_int::from(nvim_get_autocmds_count(event) > 0)
    } else {
        0
    }
}

/// Internal helper to check if an event has autocommands.
fn has_event_impl(event: c_int) -> bool {
    if (0..NUM_EVENTS).contains(&event) {
        unsafe { nvim_get_autocmds_count(event) > 0 }
    } else {
        false
    }
}

/// Check if there is a CursorHold/CursorHoldI autocommand defined for
/// the current mode.
///
/// Returns 1 if there is a cursorhold autocommand, 0 otherwise.
#[no_mangle]
pub unsafe extern "C" fn rs_has_cursorhold() -> c_int {
    let state = rs_get_real_state();
    let event = if state == MODE_NORMAL_BUSY {
        EVENT_CURSORHOLD
    } else {
        EVENT_CURSORHOLDI
    };
    c_int::from(has_event_impl(event))
}

/// Check if the CursorHold/CursorHoldI event can be triggered.
///
/// Returns 1 if the event can be triggered, 0 otherwise.
#[no_mangle]
pub unsafe extern "C" fn rs_trigger_cursorhold() -> c_int {
    // Check preconditions: cursorhold not yet triggered, has autocommand,
    // not recording, type-ahead buffer empty, and completion not active
    if nvim_get_did_cursorhold() != 0
        || rs_has_cursorhold() == 0
        || nvim_get_reg_recording() != 0
        || nvim_get_typebuf_len() != 0
        || rs_ins_compl_active() != 0
    {
        return 0;
    }

    // Check if we're in the right mode (normal-busy or insert)
    let state = rs_get_real_state();
    if state == MODE_NORMAL_BUSY || (state & MODE_INSERT) != 0 {
        return 1;
    }
    0
}

/// Check if "win" is an active entry in the aucmd_win array.
///
/// Returns 1 if the window is found in the autocmd window array and is in use, 0 otherwise.
///
/// # Safety
/// The `win` parameter must be a valid `win_T*` pointer or null.
#[no_mangle]
pub unsafe extern "C" fn rs_is_aucmd_win(win: WinHandle) -> c_int {
    let count = nvim_get_aucmd_win_count();
    for i in 0..count {
        if nvim_aucmd_win_used(i) != 0 && nvim_aucmd_win_get_win(i) == win {
            return 1;
        }
    }
    0
}

/// Returns the length of the first pattern in a comma-separated pattern list.
///
/// Handles brace groups like `*.{obj,o}` where the comma is not a separator.
/// Returns 0 if the pattern is empty (NUL).
///
/// # Safety
/// `pat` must be a valid NUL-terminated C string.
#[no_mangle]
pub unsafe extern "C" fn rs_aucmd_pattern_length(pat: *const c_char) -> usize {
    if pat.is_null() {
        return 0;
    }

    let mut p = pat;

    // Check for empty string
    if *p == 0 {
        return 0;
    }

    loop {
        let endpat_start = p;

        // Ignore single comma at start
        if *p == b',' as c_char {
            p = p.add(1);
            if *p == 0 {
                break;
            }
            continue;
        }

        // Find end of the pattern, watching for comma in braces
        let mut endpat = p;
        let mut brace_level = 0i32;

        loop {
            let c = *endpat;
            if c == 0 {
                break;
            }
            if c == b',' as c_char && brace_level == 0 {
                // Check if previous char was backslash (escaped comma)
                if endpat > endpat_start && *endpat.sub(1) != b'\\' as c_char {
                    break;
                }
            }
            if c == b'{' as c_char {
                brace_level += 1;
            } else if c == b'}' as c_char {
                brace_level -= 1;
            }
            endpat = endpat.add(1);
        }

        // Return length of this pattern segment
        return endpat.offset_from(p) as usize;
    }

    // Fallback: return strlen of remaining pattern
    let mut len = 0usize;
    while *p.add(len) != 0 {
        len += 1;
    }
    len
}

/// Returns a pointer to the next pattern in a comma-separated pattern list.
///
/// Given a pattern `pat` and its length `patlen`, returns a pointer to the
/// start of the next pattern (skipping the comma separator if present).
///
/// # Safety
/// `pat` must be a valid pointer within a NUL-terminated C string, and
/// `patlen` must not exceed the remaining length of the string.
#[no_mangle]
pub unsafe extern "C" fn rs_aucmd_next_pattern(pat: *const c_char, patlen: usize) -> *const c_char {
    let mut p = pat.add(patlen);
    if *p == b',' as c_char {
        p = p.add(1);
    }
    p
}

/// Checks if an autocommand pattern is buffer-local.
///
/// A pattern is buffer-local if it starts with "<buffer" and ends with ">".
/// Examples: "<buffer>", "<buffer=1>", "<buffer=abuf>"
///
/// # Safety
/// `pat` must be a valid pointer to at least `patlen` bytes.
#[no_mangle]
pub unsafe extern "C" fn rs_aupat_is_buflocal(pat: *const c_char, patlen: c_int) -> c_int {
    if pat.is_null() || patlen < 8 {
        return 0;
    }

    let patlen = patlen as usize;

    // Check starts with "<buffer" (7 chars)
    let buffer_prefix = b"<buffer";
    for (i, &expected) in buffer_prefix.iter().enumerate() {
        let c = *pat.add(i) as u8;
        // Case-insensitive comparison for 'b', 'u', 'f', 'e', 'r'
        if i == 0 {
            if c != b'<' {
                return 0;
            }
        } else if c.to_ascii_lowercase() != expected {
            return 0;
        }
    }

    // Check ends with ">"
    let last = *pat.add(patlen - 1) as u8;
    c_int::from(last == b'>')
}

/// Get the buffer number from a buffer-local pattern.
///
/// Patterns: `<buffer>` → curbuf fnum, `<buffer=abuf>` → autocmd_bufnr,
/// `<buffer=N>` → N. Returns 0 if the pattern is invalid.
///
/// # Safety
/// `pat` must be a valid pointer to at least `patlen` bytes.
/// The pattern must be buffer-local (caller asserts this).
#[no_mangle]
pub unsafe extern "C" fn rs_aupat_get_buflocal_nr(pat: *const c_char, patlen: c_int) -> c_int {
    let patlen = patlen as usize;

    // "<buffer>" — bare pattern means current buffer
    if patlen == 8 {
        return nvim_get_curbuf_fnum();
    }

    // Need at least "<buffer=X>" (10 chars) and '=' at position 7
    if patlen > 9 && *pat.add(7) == b'=' as c_char {
        // "<buffer=abuf>" — use the autocmd buffer number
        if patlen == 13 {
            let slice = std::slice::from_raw_parts(pat.cast::<u8>(), 13);
            if slice.eq_ignore_ascii_case(b"<buffer=abuf>") {
                return nvim_get_autocmd_bufnr();
            }
        }

        // "<buffer=123>" — parse the digits
        // Check that characters at positions 8..patlen-1 are all digits
        let digits_start = 8;
        let digits_end = patlen - 1; // last char should be '>'
        let mut all_digits = digits_start < digits_end;
        for i in digits_start..digits_end {
            let c = *pat.add(i) as u8;
            if !c.is_ascii_digit() {
                all_digits = false;
                break;
            }
        }
        if all_digits {
            // Parse the number using atoi-equivalent
            let slice = std::slice::from_raw_parts(pat.add(8).cast::<u8>(), digits_end - 8);
            if let Ok(s) = std::str::from_utf8(slice) {
                if let Ok(n) = s.parse::<c_int>() {
                    return n;
                }
            }
        }
    }

    0
}

/// Normalize a buffer-local pattern to standard `<buffer=N>` form.
///
/// If `buflocal_nr` is 0, uses `curbuf->handle` instead.
/// Writes a NUL-terminated string into `dest` (must be at least `BUFLOCAL_PAT_LEN` bytes).
///
/// # Safety
/// `dest` must be a valid writable pointer to at least `BUFLOCAL_PAT_LEN` bytes.
/// `pat` must be a valid pointer to at least `patlen` bytes.
#[no_mangle]
pub unsafe extern "C" fn rs_aupat_normalize_buflocal_pat(
    dest: *mut c_char,
    _pat: *const c_char,
    _patlen: c_int,
    mut buflocal_nr: c_int,
) {
    if buflocal_nr == 0 {
        buflocal_nr = nvim_get_curbuf_handle();
    }

    // Format "<buffer=N>" into dest
    // Use a stack buffer and copy
    let mut buf = [0u8; BUFLOCAL_PAT_LEN];
    let formatted = format!("<buffer={buflocal_nr}>");
    let bytes = formatted.as_bytes();
    let copy_len = bytes.len().min(BUFLOCAL_PAT_LEN - 1);
    buf[..copy_len].copy_from_slice(&bytes[..copy_len]);
    buf[copy_len] = 0;
    std::ptr::copy_nonoverlapping(buf.as_ptr(), dest.cast::<u8>(), copy_len + 1);
}

// =============================================================================
// Phase 4: Autocmd Deletion + Cleanup
// =============================================================================

/// Delete all autocommands for a specific event and group, then cleanup.
///
/// # Safety
/// `event` must be a valid event number (0..NUM_EVENTS).
#[no_mangle]
pub unsafe extern "C" fn rs_aucmd_del_for_event_and_group(event: c_int, group: c_int) {
    let size = nvim_get_autocmds_count(event);
    for i in 0..size {
        if nvim_autocmd_pat_is_null(event, i) == 0 && nvim_autocmd_get_pat_group(event, i) == group
        {
            nvim_autocmd_del_at(event, i);
        }
    }
    rs_au_cleanup();
}

/// Cleanup autocommands that have been deleted.
/// Only runs when not executing autocommands and cleanup is needed.
#[no_mangle]
pub unsafe extern "C" fn rs_au_cleanup() {
    if nvim_get_autocmd_busy() || nvim_get_au_need_clean() == 0 {
        return;
    }

    for event in 0..NUM_EVENTS {
        nvim_autocmd_compact_event(event);
    }

    nvim_set_au_need_clean(0);
}

/// Remove/invalidate buffer-local autocommands when a buffer is freed.
///
/// # Safety
/// `bufnr` must be the buffer's file number (`buf->b_fnum`).
#[no_mangle]
pub unsafe extern "C" fn rs_aubuflocal_remove(bufnr: c_int) {
    // Invalidate currently executing autocommands
    nvim_apc_invalidate_bufnr(bufnr);

    // Invalidate buffer-local autocommands across all events
    for event in 0..NUM_EVENTS {
        let size = nvim_get_autocmds_count(event);
        for i in 0..size {
            if nvim_autocmd_pat_is_null(event, i) != 0 {
                continue;
            }
            if nvim_autocmd_get_pat_buflocal_nr(event, i) != bufnr {
                continue;
            }
            nvim_autocmd_del_at(event, i);
            nvim_verbose_buflocal_remove(event, bufnr);
        }
    }
    rs_au_cleanup();
}

// =============================================================================
// Phase 5: :augroup Command + Arg Parsing
// =============================================================================

/// Handle `:augroup` command.
///
/// - `del_group` true: delete the named group
/// - arg is "end": switch back to default group
/// - arg is non-empty: switch to (or create) the named group
/// - arg is empty: list all group names
///
/// # Safety
/// `arg` must be a valid NUL-terminated C string.
#[no_mangle]
pub unsafe extern "C" fn rs_do_augroup(arg: *mut c_char, del_group: bool) {
    if del_group {
        if *arg == 0 {
            nvim_autocmd_emsg(nvim_autocmd_get_e_argreq());
        } else {
            // augroup_del is still in C (not yet migrated)
            // We call it through the C wrapper
            extern "C" {
                fn augroup_del(name: *mut c_char, stupid_legacy_mode: bool);
            }
            augroup_del(arg, true);
        }
    } else if strnicmp_3(arg, b"end") && {
        let after = *arg.add(3) as u8;
        after == 0
    } {
        // ":aug end": back to group 0
        nvim_autocmd_set_current_augroup(group::AUGROUP_DEFAULT);
    } else if *arg != 0 {
        // ":aug xxx": switch to group xxx
        let id = rs_augroup_add(arg);
        nvim_autocmd_set_current_augroup(id);
    } else {
        // ":aug": list the group names (msg_start, msg_ext_set_kind, iteration, msg_clr_eos, msg_end all in one C call)
        nvim_autocmd_list_group_names();
    }
}

/// Parse group name from `:autocmd` / `:doautocmd` argument.
///
/// Advances `*argp` past the group name if found.
/// Returns the group ID or `AUGROUP_ALL`.
///
/// # Safety
/// `argp` must be a valid pointer to a `*const c_char` pointing into a NUL-terminated string.
#[no_mangle]
pub unsafe extern "C" fn rs_arg_augroup_get(argp: *mut *const c_char) -> c_int {
    let arg = *argp;

    // Scan to end of token (whitespace, pipe, or NUL)
    let mut p = arg;
    while *p != 0 && !is_ascii_white(*p as u8) && *p != b'|' as c_char {
        p = p.add(1);
    }

    if p == arg {
        return group::AUGROUP_ALL;
    }

    let len = p.offset_from(arg) as usize;
    let group_name = nvim_autocmd_xmemdupz(arg, len);
    let group_id = rs_augroup_find(group_name);
    nvim_autocmd_xfree(group_name);

    if group_id == group::AUGROUP_ERROR {
        group::AUGROUP_ALL
    } else {
        *argp = nvim_skipwhite(p);
        group_id
    }
}

/// Validate and skip over event names in an autocmd argument.
///
/// Returns a pointer to the character after the last valid event name,
/// or NULL if an error is encountered.
///
/// # Safety
/// `arg` must be a valid NUL-terminated C string.
#[no_mangle]
pub unsafe extern "C" fn rs_arg_event_skip(arg: *const c_char, have_group: bool) -> *const c_char {
    if *arg == b'*' as c_char {
        // Check for illegal character after *
        if *arg.add(1) != 0 && !is_ascii_white(*arg.add(1) as u8) {
            nvim_autocmd_semsg_str(nvim_autocmd_get_e215(), arg);
            return std::ptr::null();
        }
        return arg.add(1);
    }

    let mut pat = arg;
    while *pat != 0 && *pat != b'|' as c_char && !is_ascii_white(*pat as u8) {
        let result = event::rs_event_name2nr(pat);
        if result.event >= NUM_EVENTS {
            if have_group {
                nvim_autocmd_semsg_str(nvim_autocmd_get_e216_no_such_event(), pat);
            } else {
                nvim_autocmd_semsg_str(nvim_autocmd_get_e216_no_such_group_or_event(), pat);
            }
            return std::ptr::null();
        }
        pat = result.end_ptr;
    }

    pat
}

/// Parse `++once`, `++nested` flags from autocmd command.
///
/// If the flag matches and was not already set, sets it and advances `*cmd_ptr`.
/// Returns true on duplicate flag error, false normally.
///
/// # Safety
/// All pointers must be valid.
#[no_mangle]
pub unsafe extern "C" fn rs_arg_autocmd_flag_get(
    flag: *mut bool,
    cmd_ptr: *mut *const c_char,
    pattern: *const c_char,
    len: c_int,
) -> bool {
    let cmd = *cmd_ptr;
    let len = len as usize;

    // Check if cmd starts with pattern followed by whitespace
    let mut matches = true;
    for i in 0..len {
        if *cmd.add(i) != *pattern.add(i) {
            matches = false;
            break;
        }
    }

    if matches && is_ascii_white(*cmd.add(len) as u8) {
        if *flag {
            nvim_autocmd_semsg_str(nvim_autocmd_get_e_duparg2(), pattern);
            return true;
        }
        *flag = true;
        *cmd_ptr = nvim_skipwhite(cmd.add(len));
    }

    false
}

/// Check for `<nomodeline>` marker in argument.
///
/// If found, advances `*argp` past it and returns false (no modeline).
/// Otherwise returns true (process modeline).
///
/// # Safety
/// `argp` must be a valid pointer to a `*const c_char`.
#[no_mangle]
pub unsafe extern "C" fn rs_check_nomodeline(argp: *mut *const c_char) -> bool {
    let arg = *argp;
    let marker = b"<nomodeline>";
    let mut matches = true;
    for (i, &expected) in marker.iter().enumerate() {
        if *arg.add(i) as u8 != expected {
            matches = false;
            break;
        }
    }
    if matches {
        *argp = nvim_skipwhite(arg.add(12));
        return false;
    }
    true
}

/// Helper: case-insensitive 3-byte prefix check against a known ASCII lowercase pattern.
unsafe fn strnicmp_3(s: *const c_char, prefix: &[u8; 3]) -> bool {
    for (i, &expected) in prefix.iter().enumerate() {
        let c = *s.add(i) as u8;
        if c.to_ascii_lowercase() != expected {
            return false;
        }
    }
    true
}

/// Helper: check if a byte is ASCII whitespace.
#[inline]
fn is_ascii_white(c: u8) -> bool {
    c == b' ' || c == b'\t'
}

/// Check whether a given autocommand event name is supported.
///
/// Returns 1 if the event name is recognized, 0 otherwise.
///
/// # Safety
/// `event` must be a valid NUL-terminated C string.
#[no_mangle]
pub unsafe extern "C" fn rs_autocmd_supported(event: *const c_char) -> c_int {
    if event.is_null() {
        return 0;
    }

    // Find the length of the event name (up to NUL, whitespace, comma, or pipe)
    let mut len = 0usize;
    loop {
        let c = *event.add(len) as u8;
        if c == 0 || c == b' ' || c == b'\t' || c == b',' || c == b'|' {
            break;
        }
        len += 1;
    }

    let result = nvim_event_name2nr(event, len);
    c_int::from(result != NUM_EVENTS)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::CString;

    #[test]
    fn test_aucmd_pattern_length() {
        unsafe {
            // Empty pattern
            let empty = CString::new("").unwrap();
            assert_eq!(rs_aucmd_pattern_length(empty.as_ptr()), 0);

            // Simple pattern
            let simple = CString::new("*.c").unwrap();
            assert_eq!(rs_aucmd_pattern_length(simple.as_ptr()), 3);

            // Pattern with comma
            let with_comma = CString::new("*.c,*.h").unwrap();
            assert_eq!(rs_aucmd_pattern_length(with_comma.as_ptr()), 3);

            // Pattern with braces containing comma
            let with_braces = CString::new("*.{c,h}").unwrap();
            assert_eq!(rs_aucmd_pattern_length(with_braces.as_ptr()), 7);
        }
    }

    #[test]
    fn test_aucmd_next_pattern() {
        unsafe {
            let patterns = CString::new("*.c,*.h").unwrap();
            let ptr = patterns.as_ptr();

            // First pattern is "*.c" (length 3)
            let next = rs_aucmd_next_pattern(ptr, 3);
            // Should point to "*.h"
            assert_eq!(*next, b'*' as c_char);
        }
    }

    #[test]
    fn test_aupat_is_buflocal() {
        unsafe {
            // Valid buffer-local patterns
            let buf = CString::new("<buffer>").unwrap();
            assert_eq!(rs_aupat_is_buflocal(buf.as_ptr(), 8), 1);

            let buf_eq = CString::new("<buffer=1>").unwrap();
            assert_eq!(rs_aupat_is_buflocal(buf_eq.as_ptr(), 10), 1);

            let buf_abuf = CString::new("<buffer=abuf>").unwrap();
            assert_eq!(rs_aupat_is_buflocal(buf_abuf.as_ptr(), 13), 1);

            // Case insensitive
            let buf_upper = CString::new("<BUFFER>").unwrap();
            assert_eq!(rs_aupat_is_buflocal(buf_upper.as_ptr(), 8), 1);

            // Invalid patterns
            let short = CString::new("<buf>").unwrap();
            assert_eq!(rs_aupat_is_buflocal(short.as_ptr(), 5), 0);

            let no_end = CString::new("<buffer").unwrap();
            assert_eq!(rs_aupat_is_buflocal(no_end.as_ptr(), 7), 0);

            let wrong_start = CString::new("buffer>").unwrap();
            assert_eq!(rs_aupat_is_buflocal(wrong_start.as_ptr(), 7), 0);

            let normal = CString::new("*.c").unwrap();
            assert_eq!(rs_aupat_is_buflocal(normal.as_ptr(), 3), 0);
        }
    }

    #[test]
    fn test_aucmd_pattern_length_escaped_comma() {
        unsafe {
            // Escaped comma should not be treated as separator
            let escaped = CString::new("*.\\,c,*.h").unwrap();
            // Length should include the escaped comma
            assert_eq!(rs_aucmd_pattern_length(escaped.as_ptr()), 5);
        }
    }

    #[test]
    fn test_aucmd_pattern_length_nested_braces() {
        unsafe {
            // Nested braces pattern
            let nested = CString::new("*.{{a,b},{c,d}}").unwrap();
            assert_eq!(rs_aucmd_pattern_length(nested.as_ptr()), 15);
        }
    }

    #[test]
    fn test_aucmd_pattern_length_null() {
        unsafe {
            // Null pointer should return 0
            assert_eq!(rs_aucmd_pattern_length(std::ptr::null()), 0);
        }
    }

    #[test]
    fn test_aupat_is_buflocal_null() {
        unsafe {
            // Null pointer should return 0
            assert_eq!(rs_aupat_is_buflocal(std::ptr::null(), 8), 0);
        }
    }

    #[test]
    fn test_unknown_event_string() {
        // Verify UNKNOWN_EVENT is properly null-terminated
        assert!(UNKNOWN_EVENT.ends_with(&[0]));
        assert_eq!(&UNKNOWN_EVENT[..7], b"Unknown");
    }

    #[test]
    fn test_mode_constants() {
        // Verify mode constants match expected values from state_defs.h
        assert_eq!(MODE_NORMAL, 0x01);
        assert_eq!(MODE_NORMAL_BUSY, 0x1001);
        assert_eq!(MODE_INSERT, 0x10);
    }

    #[test]
    fn test_event_constants() {
        // Verify event constants match expected values from auevents_enum.generated.h
        assert_eq!(EVENT_CURSORHOLD, 37);
        assert_eq!(EVENT_CURSORHOLDI, 38);
        assert_eq!(NUM_EVENTS, 141);
    }
}
