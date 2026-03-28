//! Autocommand state checking for Neovim
//!
//! This module provides Rust implementations for checking autocommand state.

#![warn(clippy::all, clippy::pedantic)]
#![allow(clippy::missing_safety_doc)]
#![allow(clippy::doc_markdown)]
#![allow(clippy::cast_possible_wrap)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::missing_const_for_fn)]
#![allow(clippy::must_use_candidate)]

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
const EVENT_FOCUSGAINED: c_int = 66;
const EVENT_FOCUSLOST: c_int = 67;
const EVENT_TERMRESPONSE: c_int = 120;
const NUM_EVENTS: c_int = 141;

// C constants from vim_defs.h
const FAIL: c_int = 0;
const OK: c_int = 1;

// Buffer-local pattern constants from autocmd.h
const BUFLOCAL_PAT_LEN: usize = 25;

// Vim variable index from eval_defs.h
const VV_TERMRESPONSE: c_int = 11;

// C accessors for static data
extern "C" {
    fn nvim_get_autocmd_blocked() -> c_int;
    fn nvim_get_event_name(event: c_int) -> *const c_char;
    fn nvim_get_event_sign(event: c_int) -> c_int;
    fn nvim_get_autocmds_count(event: c_int) -> usize;
    fn nvim_get_next_augroup_id() -> c_int;
    fn nvim_get_deleted_augroup() -> *const c_char;

    // Accessors for aucmd_win array
    fn nvim_get_aucmd_win_count() -> c_int;
    fn nvim_aucmd_win_used(idx: c_int) -> c_int;
    fn nvim_aucmd_win_get_win(idx: c_int) -> WinHandle;

    // From event crate - get the real editor state
    #[link_name = "get_real_state"]
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
    #[link_name = "emsg"]
    fn nvim_autocmd_emsg(msg: *const c_char);
    fn nvim_autocmd_semsg_str(fmt: *const c_char, arg: *const c_char);
    fn nvim_autocmd_get_e_argreq() -> *const c_char;
    fn nvim_autocmd_get_e216_no_such_event() -> *const c_char;
    fn nvim_autocmd_get_e216_no_such_group_or_event() -> *const c_char;
    fn nvim_autocmd_get_e215() -> *const c_char;
    fn nvim_autocmd_get_e_duparg2() -> *const c_char;
    fn skipwhite(p: *const c_char) -> *mut c_char;
    #[link_name = "xmemdupz"]
    fn nvim_autocmd_xmemdupz(src: *const c_char, len: usize) -> *mut c_char;
    #[link_name = "xfree"]
    fn nvim_autocmd_xfree(ptr: *mut c_char);

    // Phase 6: Display + Query accessors
    fn nvim_autocmd_get_pat_str(event: c_int, idx: usize) -> *const c_char;
    fn nvim_autocmd_get_pat_patlen(event: c_int, idx: usize) -> c_int;
    fn nvim_autocmd_get_pat_id(event: c_int, idx: usize) -> usize;
    fn nvim_autocmd_get_handler_str(event: c_int, idx: usize) -> *mut c_char;
    fn nvim_autocmd_get_desc(event: c_int, idx: usize) -> *const c_char;
    fn nvim_autocmd_has_handler_cmd(event: c_int, idx: usize) -> bool;
    fn nvim_autocmd_show_last_set(event: c_int, idx: usize);
    #[link_name = "msg_putchar"]
    fn nvim_autocmd_msg_putchar(c: c_int);
    #[link_name = "msg_puts_hl"]
    fn nvim_autocmd_msg_puts_hl(s: *const c_char, hlf: c_int, append: bool);
    fn nvim_autocmd_msg_outtrans(s: *const c_char);
    static mut msg_col: c_int;
    fn nvim_autocmd_get_got_int() -> c_int;
    fn nvim_autocmd_get_p_verbose() -> c_int;
    fn nvim_autocmd_match_file(
        event: c_int,
        idx: usize,
        fname: *const c_char,
        sfname: *const c_char,
        tail: *const c_char,
        buf_fnum: c_int,
    ) -> bool;
    #[link_name = "path_tail"]
    fn nvim_autocmd_path_tail(fname: *const c_char) -> *const c_char;
    fn nvim_autocmd_fullname_save(fname: *const c_char) -> *mut c_char;
    #[link_name = "path_fnamecmp"]
    fn nvim_autocmd_path_fnamecmp(a: *const c_char, b: *const c_char) -> c_int;
    #[link_name = "nvim_get_curbuf_fnum"]
    fn nvim_autocmd_get_curbuf_fnum() -> c_int;
    #[link_name = "msg_puts"]
    fn nvim_autocmd_msg_puts(s: *const c_char);
    #[link_name = "xmallocz"]
    fn nvim_autocmd_xmallocz(len: usize) -> *mut c_char;
    #[link_name = "xstrdup"]
    fn nvim_autocmd_xstrdup(s: *const c_char) -> *mut c_char;

    // Phase 7: :autocmd command + registration accessors
    fn nvim_autocmd_eap_set_nextcmd(eap: *mut c_void, val: *mut c_char);
    #[link_name = "vim_strchr"]
    fn nvim_autocmd_vim_strchr(s: *const c_char, c: c_int) -> *const c_char;
    #[link_name = "expand_env_save"]
    fn nvim_autocmd_expand_env_save(pat: *const c_char) -> *mut c_char;
    fn nvim_docmd_expand_sfile_impl(cmd: *const c_char) -> *mut c_char;
    fn nvim_autocmd_show_header();
    fn nvim_autocmd_get_e_cannot_define_for_all() -> *const c_char;
    fn nvim_autocmd_get_current_augroup() -> c_int;
    fn nvim_autocmd_del_matching(event: c_int, findgroup: c_int, pat: *const c_char, patlen: c_int);
    fn nvim_autocmd_register_cmd(
        event: c_int,
        pat: *const c_char,
        patlen: c_int,
        group: c_int,
        once: bool,
        nested: bool,
        cmd: *const c_char,
    ) -> c_int;
    fn nvim_autocmd_ok() -> c_int;

    // Phase 8a: Simple wrappers + blocking accessors
    #[link_name = "get_vim_var_str"]
    fn nvim_autocmd_get_vim_var_str(vv: c_int) -> *const c_char;
    fn nvim_autocmd_get_old_termresponse() -> *const c_char;
    fn nvim_autocmd_set_old_termresponse(ptr: *const c_char);
    fn nvim_autocmd_inc_blocked();
    fn nvim_autocmd_dec_blocked();
    fn nvim_autocmd_apply_autocmds_group(
        event: c_int,
        fname: *mut c_char,
        fname_io: *mut c_char,
        force: bool,
        group: c_int,
        buf: *mut c_void,
        eap: *mut c_void,
        data: *mut c_void,
    ) -> bool;
    #[link_name = "should_abort"]
    fn nvim_autocmd_should_abort(retval: c_int) -> bool;
    #[link_name = "aborting"]
    fn nvim_autocmd_aborting() -> bool;
    fn nvim_autocmd_get_curbuf_ptr() -> *mut c_void;

    // Phase 8e: Event triggers + doautocmd accessors
    fn nvim_autocmd_get_e217() -> *const c_char;
    fn nvim_autocmd_smsg_no_matching(arg_start: *const c_char);

    // Phase 3: Event trigger helpers
    fn check_timestamps(focus: c_int);
    fn os_now() -> u64;

    // Rust functions from group module (exported under C names)
    #[link_name = "augroup_find"]
    fn rs_augroup_find(name: *const c_char) -> c_int;
    #[link_name = "augroup_add"]
    fn rs_augroup_add(name: *const c_char) -> c_int;
    #[link_name = "augroup_name"]
    fn rs_augroup_name(group: c_int) -> *const c_char;
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

/// Single-arg version of has_event exported under the C name `has_event`.
/// Uses the global NUM_EVENTS constant.
#[unsafe(export_name = "has_event")]
pub unsafe extern "C" fn rs_has_event_c(event: c_int) -> c_int {
    rs_has_event(event, NUM_EVENTS)
}

/// Exported as `is_autocmd_blocked` for C callers.
#[unsafe(export_name = "is_autocmd_blocked")]
pub unsafe extern "C" fn rs_is_autocmd_blocked_c() -> c_int {
    rs_is_autocmd_blocked()
}

/// Exported as `trigger_cursorhold` for C callers.
#[unsafe(export_name = "trigger_cursorhold")]
pub unsafe extern "C" fn rs_trigger_cursorhold_c() -> c_int {
    rs_trigger_cursorhold()
}

/// Exported as `has_cursorhold` for C callers.
#[unsafe(export_name = "has_cursorhold")]
pub unsafe extern "C" fn rs_has_cursorhold_c() -> c_int {
    rs_has_cursorhold()
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
#[export_name = "is_aucmd_win"]
pub unsafe extern "C" fn rs_is_aucmd_win(win: WinHandle) -> bool {
    let count = nvim_get_aucmd_win_count();
    for i in 0..count {
        if nvim_aucmd_win_used(i) != 0 && nvim_aucmd_win_get_win(i) == win {
            return true;
        }
    }
    false
}

/// Returns the length of the first pattern in a comma-separated pattern list.
///
/// Handles brace groups like `*.{obj,o}` where the comma is not a separator.
/// Returns 0 if the pattern is empty (NUL).
///
/// # Safety
/// `pat` must be a valid NUL-terminated C string.
#[unsafe(export_name = "aucmd_pattern_length")]
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
#[unsafe(export_name = "aucmd_next_pattern")]
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
#[export_name = "aupat_is_buflocal"]
pub unsafe extern "C" fn rs_aupat_is_buflocal(pat: *const c_char, patlen: c_int) -> bool {
    if pat.is_null() || patlen < 8 {
        return false;
    }

    let patlen = patlen as usize;

    // Check starts with "<buffer" (7 chars)
    let buffer_prefix = b"<buffer";
    for (i, &expected) in buffer_prefix.iter().enumerate() {
        let c = *pat.add(i) as u8;
        // Case-insensitive comparison for 'b', 'u', 'f', 'e', 'r'
        if i == 0 {
            if c != b'<' {
                return false;
            }
        } else if c.to_ascii_lowercase() != expected {
            return false;
        }
    }

    // Check ends with ">"
    let last = *pat.add(patlen - 1) as u8;
    last == b'>'
}

/// Get the buffer number from a buffer-local pattern.
///
/// Patterns: `<buffer>` → curbuf fnum, `<buffer=abuf>` → autocmd_bufnr,
/// `<buffer=N>` → N. Returns 0 if the pattern is invalid.
///
/// # Safety
/// `pat` must be a valid pointer to at least `patlen` bytes.
/// The pattern must be buffer-local (caller asserts this).
#[unsafe(export_name = "aupat_get_buflocal_nr")]
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
#[unsafe(export_name = "aupat_normalize_buflocal_pat")]
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
#[unsafe(export_name = "aucmd_del_for_event_and_group")]
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
#[unsafe(export_name = "au_cleanup")]
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
#[unsafe(export_name = "do_augroup")]
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
#[export_name = "arg_augroup_get"]
#[allow(clippy::must_use_candidate)]
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
        *argp = skipwhite(p);
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
#[unsafe(export_name = "arg_autocmd_flag_get")]
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
        *cmd_ptr = skipwhite(cmd.add(len));
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
#[unsafe(export_name = "check_nomodeline")]
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
        *argp = skipwhite(arg.add(12));
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
#[export_name = "autocmd_supported"]
pub unsafe extern "C" fn rs_autocmd_supported(event: *const c_char) -> bool {
    if event.is_null() {
        return false;
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
    result != NUM_EVENTS
}

// =============================================================================
// Phase 6: Display + Query Functions
// =============================================================================

// Highlight constants from highlight_defs.h (verified with _Static_assert in autocmd.c)
const HLF_8: c_int = 1; // Meta & special keys
const HLF_E: c_int = 6; // Error messages
const HLF_T: c_int = 23; // Titles

/// Helper: compute C string length.
unsafe fn c_strlen(s: *const c_char) -> usize {
    let mut len = 0;
    while *s.add(len) != 0 {
        len += 1;
    }
    len
}

/// Helper: compare two C strings for `len` bytes.
unsafe fn c_strncmp(a: *const c_char, b: *const c_char, len: usize) -> bool {
    for i in 0..len {
        if *a.add(i) != *b.add(i) {
            return false;
        }
    }
    true
}

/// Display autocommands for a specific event, optionally filtered by group and pattern.
///
/// # Safety
/// `pat` must be a valid NUL-terminated C string (or empty string).
#[unsafe(export_name = "au_show_for_event")]
#[allow(clippy::too_many_lines)]
pub unsafe extern "C" fn rs_au_show_for_event(group: c_int, event: c_int, pat: *const c_char) {
    // Return early if there are no autocmds for this event
    if nvim_get_autocmds_count(event) == 0 {
        return;
    }

    let mut buflocal_pat = [0u8; BUFLOCAL_PAT_LEN];
    let mut pat = pat;
    let mut patlen: c_int;

    if !pat.is_null() && *pat != 0 {
        #[allow(clippy::cast_possible_truncation)]
        {
            patlen = rs_aucmd_pattern_length(pat) as c_int;
        }

        // detect special <buffer[=X]> buffer-local patterns
        if rs_aupat_is_buflocal(pat, patlen) {
            let buflocal_nr = rs_aupat_get_buflocal_nr(pat, patlen);
            rs_aupat_normalize_buflocal_pat(
                buflocal_pat.as_mut_ptr().cast(),
                pat,
                patlen,
                buflocal_nr,
            );
            pat = buflocal_pat.as_ptr().cast();
            #[allow(clippy::cast_possible_truncation)]
            {
                patlen = c_strlen(pat) as c_int;
            }
        }

        if patlen == 0 {
            return;
        }
    } else {
        pat = std::ptr::null();
        patlen = 0;
    }

    // Loop through all the specified patterns
    loop {
        au_show_event_inner(group, event, pat, patlen);

        // If a pattern is provided, find next pattern. Otherwise exit after single iteration.
        if pat.is_null() {
            break;
        }
        pat = rs_aucmd_next_pattern(pat, patlen as usize);
        #[allow(clippy::cast_possible_truncation)]
        {
            patlen = rs_aucmd_pattern_length(pat) as c_int;
        }
        if patlen == 0 {
            break;
        }
    }
}

/// Inner loop for `rs_au_show_for_event` — iterates autocmds for one pattern.
#[allow(clippy::too_many_lines)]
unsafe fn au_show_event_inner(group: c_int, event: c_int, pat: *const c_char, patlen: c_int) {
    let mut last_pat_id: usize = 0;
    let mut last_group = group::AUGROUP_ERROR;

    let size = nvim_get_autocmds_count(event);
    for i in 0..size {
        // Skip deleted autocommands
        if nvim_autocmd_pat_is_null(event, i) != 0 {
            continue;
        }

        let ac_group = nvim_autocmd_get_pat_group(event, i);
        let ac_patlen = nvim_autocmd_get_pat_patlen(event, i);
        let ac_pat = nvim_autocmd_get_pat_str(event, i);

        // Filter by group and pattern
        if (group != group::AUGROUP_ALL && ac_group != group)
            || (!pat.is_null() && (ac_patlen != patlen || !c_strncmp(pat, ac_pat, patlen as usize)))
        {
            continue;
        }

        // Show event name and group only if one of them changed
        if ac_group != last_group {
            last_group = ac_group;
            if !show_group_and_event(ac_group, event) {
                return;
            }
        }

        // Show pattern only if it changed
        let pat_id = nvim_autocmd_get_pat_id(event, i);
        if pat_id != last_pat_id {
            last_pat_id = pat_id;
            nvim_autocmd_msg_putchar(c_int::from(b'\n'));
            if nvim_autocmd_get_got_int() != 0 {
                return;
            }
            msg_col = 4;
            nvim_autocmd_msg_outtrans(ac_pat);
        }

        if nvim_autocmd_get_got_int() != 0 {
            return;
        }

        if msg_col >= 14 {
            nvim_autocmd_msg_putchar(c_int::from(b'\n'));
        }
        msg_col = 14;
        if nvim_autocmd_get_got_int() != 0 {
            return;
        }

        show_handler_and_desc(event, i);

        if nvim_autocmd_get_p_verbose() > 0 {
            nvim_autocmd_show_last_set(event, i);
        }

        if nvim_autocmd_get_got_int() != 0 {
            return;
        }
    }
}

/// Show group name and event name when group changes. Returns false if got_int.
unsafe fn show_group_and_event(ac_group: c_int, event: c_int) -> bool {
    let group_name = rs_augroup_name(ac_group);

    if nvim_autocmd_get_got_int() != 0 {
        return false;
    }
    nvim_autocmd_msg_putchar(c_int::from(b'\n'));
    if nvim_autocmd_get_got_int() != 0 {
        return false;
    }

    // Show group name if not the default group
    if ac_group != group::AUGROUP_DEFAULT {
        if group_name.is_null() {
            extern "C" {
                fn nvim_get_deleted_augroup() -> *const c_char;
            }
            nvim_autocmd_msg_puts_hl(nvim_get_deleted_augroup(), HLF_E, false);
        } else {
            nvim_autocmd_msg_puts_hl(group_name, HLF_T, false);
        }
        nvim_autocmd_msg_puts(c"  ".as_ptr());
    }
    // Show the event name
    let event_name = rs_event_nr2name(event, NUM_EVENTS);
    nvim_autocmd_msg_puts_hl(event_name, HLF_T, false);
    true
}

/// Display handler string and description for one autocmd entry.
unsafe fn show_handler_and_desc(event: c_int, idx: usize) {
    let handler_str = nvim_autocmd_get_handler_str(event, idx);
    let desc = nvim_autocmd_get_desc(event, idx);
    let has_cmd = nvim_autocmd_has_handler_cmd(event, idx);

    if !desc.is_null() {
        let msglen: usize = 100;
        let msg = nvim_autocmd_xmallocz(msglen);
        if has_cmd {
            format_handler_desc(msg, msglen, handler_str, desc);
        } else {
            nvim_autocmd_msg_puts_hl(handler_str, HLF_8, false);
            format_desc_only(msg, msglen, desc);
        }
        nvim_autocmd_msg_outtrans(msg);
        nvim_autocmd_xfree(msg);
    } else if has_cmd {
        nvim_autocmd_msg_outtrans(handler_str);
    } else {
        nvim_autocmd_msg_puts_hl(handler_str, HLF_8, false);
    }
    nvim_autocmd_xfree(handler_str);
}

/// Format "{handler_str} [{desc}]" into msg buffer.
unsafe fn format_handler_desc(
    msg: *mut c_char,
    msglen: usize,
    handler_str: *const c_char,
    desc: *const c_char,
) {
    let handler_len = c_strlen(handler_str);
    let desc_len = c_strlen(desc);
    let mut pos = 0usize;
    let h_copy = handler_len.min(msglen);
    std::ptr::copy_nonoverlapping(handler_str, msg, h_copy);
    pos += h_copy;
    if pos + 2 <= msglen {
        *msg.add(pos) = b' ' as c_char;
        *msg.add(pos + 1) = b'[' as c_char;
        pos += 2;
    }
    let d_copy = desc_len.min(msglen.saturating_sub(pos));
    std::ptr::copy_nonoverlapping(desc, msg.add(pos), d_copy);
    pos += d_copy;
    if pos < msglen {
        *msg.add(pos) = b']' as c_char;
        pos += 1;
    }
    *msg.add(pos.min(msglen)) = 0;
}

/// Format " [{desc}]" into msg buffer.
unsafe fn format_desc_only(msg: *mut c_char, msglen: usize, desc: *const c_char) {
    let desc_len = c_strlen(desc);
    let mut pos = 0usize;
    if pos + 2 <= msglen {
        *msg.add(pos) = b' ' as c_char;
        *msg.add(pos + 1) = b'[' as c_char;
        pos += 2;
    }
    let d_copy = desc_len.min(msglen.saturating_sub(pos));
    std::ptr::copy_nonoverlapping(desc, msg.add(pos), d_copy);
    pos += d_copy;
    if pos < msglen {
        *msg.add(pos) = b']' as c_char;
        pos += 1;
    }
    *msg.add(pos.min(msglen)) = 0;
}

/// Display autocommands for all events, optionally filtered by group and pattern.
///
/// # Safety
/// `pat` must be a valid NUL-terminated C string (or empty string).
#[unsafe(export_name = "au_show_for_all_events")]
pub unsafe extern "C" fn rs_au_show_for_all_events(group: c_int, pat: *const c_char) {
    for event in 0..NUM_EVENTS {
        rs_au_show_for_event(group, event, pat);
    }
}

/// Check if there is a matching autocommand for a filename.
///
/// # Safety
/// `sfname` must be a valid NUL-terminated C string.
#[no_mangle]
pub unsafe extern "C" fn rs_has_autocmd(
    event: c_int,
    sfname: *const c_char,
    buf_fnum: c_int,
) -> bool {
    let tail = nvim_autocmd_path_tail(sfname);
    let fname = nvim_autocmd_fullname_save(sfname);
    if fname.is_null() {
        return false;
    }

    let size = nvim_get_autocmds_count(event);
    let mut retval = false;
    for i in 0..size {
        if nvim_autocmd_match_file(event, i, fname, sfname, tail, buf_fnum) {
            retval = true;
            break;
        }
    }

    nvim_autocmd_xfree(fname);
    retval
}

/// Check if an autocommand exists matching the given specification.
///
/// Parses comma-separated group#event#pattern syntax.
///
/// # Safety
/// `arg` must be a valid NUL-terminated C string.
#[unsafe(export_name = "au_exists")]
pub unsafe extern "C" fn rs_au_exists(arg: *const c_char) -> bool {
    // Make a copy so we can modify '#' to NUL
    let arg_save = nvim_autocmd_xstrdup(arg);

    // Find first '#'
    let mut p = arg_save;
    while *p != 0 && *p != b'#' as c_char {
        p = p.add(1);
    }
    let has_hash = *p == b'#' as c_char;
    if has_hash {
        *p = 0;
        p = p.add(1);
    }

    // First, look for an autocmd group name
    let group_id = rs_augroup_find(arg_save);

    let retval = if group_id == group::AUGROUP_ERROR {
        // Didn't match a group name, assume the first argument is an event
        let event_name: *const c_char = arg_save;
        let pattern = if has_hash {
            p.cast_const()
        } else {
            std::ptr::null()
        };
        au_exists_inner(event_name, pattern, group::AUGROUP_ALL)
    } else if !has_hash {
        // "Group": group name is present and it's recognized
        true
    } else {
        // Must be "Group#Event" or "Group#Event#pat"
        let event_name: *const c_char = p.cast_const();
        // Find second '#'
        let mut p2 = p;
        while *p2 != 0 && *p2 != b'#' as c_char {
            p2 = p2.add(1);
        }
        let pattern = if *p2 == b'#' as c_char {
            *p2 = 0;
            p2 = p2.add(1);
            p2.cast_const()
        } else {
            std::ptr::null()
        };
        au_exists_inner(event_name, pattern, group_id)
    };

    nvim_autocmd_xfree(arg_save);
    retval
}

/// Inner helper for `rs_au_exists`: checks if an autocommand exists for event_name + pattern.
unsafe fn au_exists_inner(event_name: *const c_char, pattern: *const c_char, group: c_int) -> bool {
    // Find the event from the name
    let result = event::rs_event_name2nr(event_name);
    if result.event >= NUM_EVENTS {
        return false;
    }
    let event = result.event;

    // Check if there are any autocmds for this event
    let size = nvim_get_autocmds_count(event);
    if size == 0 {
        return false;
    }

    // If no pattern given and we have autocmds, check if any match the group
    if pattern.is_null() {
        // Need at least one autocmd matching the group
        for i in 0..size {
            if nvim_autocmd_pat_is_null(event, i) != 0 {
                continue;
            }
            if group == group::AUGROUP_ALL || nvim_autocmd_get_pat_group(event, i) == group {
                return true;
            }
        }
        return false;
    }

    // Check for "<buffer>" pattern - use curbuf's fnum
    let buflocal_fnum = if strnicmp_prefix_8(pattern, b"<buffer>") {
        nvim_autocmd_get_curbuf_fnum()
    } else {
        0
    };

    // Check if there is an autocommand with the given pattern
    for i in 0..size {
        if nvim_autocmd_pat_is_null(event, i) != 0 {
            continue;
        }
        let ac_group = nvim_autocmd_get_pat_group(event, i);
        if group != group::AUGROUP_ALL && ac_group != group {
            continue;
        }

        let ac_pat = nvim_autocmd_get_pat_str(event, i);
        if buflocal_fnum != 0 {
            // Buffer-local: compare buffer numbers
            if nvim_autocmd_get_pat_buflocal_nr(event, i) == buflocal_fnum {
                return true;
            }
        } else if nvim_autocmd_path_fnamecmp(ac_pat, pattern) == 0 {
            return true;
        }
    }

    false
}

/// Helper: case-insensitive check for "<buffer>" (8 bytes, NUL-terminated).
unsafe fn strnicmp_prefix_8(s: *const c_char, prefix: &[u8; 8]) -> bool {
    for (i, &expected) in prefix.iter().enumerate() {
        let c = *s.add(i) as u8;
        if !c.eq_ignore_ascii_case(&expected) {
            return false;
        }
    }
    // Must be exactly 8 chars (NUL-terminated)
    *s.add(8) == 0
}

/// Get an allocated string representation of the handler for autocmd at (event, idx).
///
/// # Safety
/// `event` and `idx` must be valid.
#[no_mangle] // keep rs_ name since it's internal
pub unsafe extern "C" fn rs_aucmd_handler_to_string(event: c_int, idx: usize) -> *mut c_char {
    nvim_autocmd_get_handler_str(event, idx)
}

// =============================================================================
// Phase 7: :autocmd Command + Registration
// =============================================================================

/// Handle the `:autocmd` command.
///
/// # Safety
/// `eap` must be a valid `exarg_T*`, `arg_in` must be a valid NUL-terminated mutable string.
#[unsafe(export_name = "do_autocmd")]
#[allow(clippy::too_many_lines)]
pub unsafe extern "C" fn rs_do_autocmd(eap: *mut c_void, arg_in: *mut c_char, forceit: c_int) {
    let mut arg = arg_in;
    let mut envpat: *mut c_char = std::ptr::null_mut();
    let mut cmd: *mut c_char;
    let mut need_free = false;
    let mut nested = false;
    let mut once = false;
    let group: c_int;

    if *arg == b'|' as c_char {
        nvim_autocmd_eap_set_nextcmd(eap, arg.add(1));
        arg = c"".as_ptr().cast_mut();
        group = group::AUGROUP_ALL;
    } else {
        let mut arg_ptr: *const c_char = arg;
        group = rs_arg_augroup_get(&raw mut arg_ptr);
        arg = arg_ptr.cast_mut();
    }

    // Scan over the events
    let pat_result = rs_arg_event_skip(arg, group != group::AUGROUP_ALL);
    if pat_result.is_null() {
        return;
    }
    let mut pat: *mut c_char = skipwhite(pat_result);

    if *pat == b'|' as c_char {
        nvim_autocmd_eap_set_nextcmd(eap, pat.add(1));
        pat = c"".as_ptr().cast_mut();
        cmd = c"".as_ptr().cast_mut();
    } else {
        // Scan over the pattern. Put a NUL at the end.
        cmd = pat;
        while *cmd != 0 && (!is_ascii_white(*cmd as u8) || *cmd.sub(1) == b'\\' as c_char) {
            cmd = cmd.add(1);
        }
        if *cmd != 0 {
            *cmd = 0;
            cmd = cmd.add(1);
        }

        // Expand environment variables in the pattern
        if !nvim_autocmd_vim_strchr(pat, c_int::from(b'$')).is_null()
            || !nvim_autocmd_vim_strchr(pat, c_int::from(b'~')).is_null()
        {
            envpat = nvim_autocmd_expand_env_save(pat);
            if !envpat.is_null() {
                pat = envpat;
            }
        }

        cmd = skipwhite(cmd);

        // Parse ++once, ++nested flags
        let mut invalid_flags = false;
        let mut cmd_const: *const c_char = cmd;
        for _ in 0..2 {
            if *cmd_const == 0 {
                continue;
            }
            invalid_flags |=
                rs_arg_autocmd_flag_get(&raw mut once, &raw mut cmd_const, c"++once".as_ptr(), 6);
            invalid_flags |= rs_arg_autocmd_flag_get(
                &raw mut nested,
                &raw mut cmd_const,
                c"++nested".as_ptr(),
                8,
            );
            // Check the deprecated "nested" flag
            invalid_flags |=
                rs_arg_autocmd_flag_get(&raw mut nested, &raw mut cmd_const, c"nested".as_ptr(), 6);
        }
        cmd = cmd_const.cast_mut();

        if invalid_flags {
            return;
        }

        // Expand <sfile> in command
        if *cmd != 0 {
            cmd = nvim_docmd_expand_sfile_impl(cmd);
            if cmd.is_null() {
                return;
            }
            need_free = true;
        }
    }

    let is_showing = forceit == 0 && *cmd == 0;

    if is_showing {
        nvim_autocmd_show_header();
        if *arg == b'*' as c_char || *arg == b'|' as c_char || *arg == 0 {
            rs_au_show_for_all_events(group, pat);
        } else {
            let result = event::rs_event_name2nr(arg);
            rs_au_show_for_event(group, result.event, pat);
        }
    } else if *arg == b'*' as c_char || *arg == 0 || *arg == b'|' as c_char {
        if *cmd != 0 {
            nvim_autocmd_emsg(nvim_autocmd_get_e_cannot_define_for_all());
        } else {
            rs_do_all_autocmd_events(pat, once, c_int::from(nested), cmd, forceit != 0, group);
        }
    } else {
        while *arg != 0 && *arg != b'|' as c_char && !is_ascii_white(*arg as u8) {
            let result = event::rs_event_name2nr(arg);
            arg = result.end_ptr.cast_mut();
            if rs_do_autocmd_event(result.event, pat, once, nested, cmd, forceit != 0, group)
                != nvim_autocmd_ok()
            {
                break;
            }
        }
    }

    if need_free {
        nvim_autocmd_xfree(cmd);
    }
    nvim_autocmd_xfree(envpat);
}

/// Execute `do_autocmd_event` for all events.
///
/// # Safety
/// All pointers must be valid NUL-terminated strings.
#[unsafe(export_name = "do_all_autocmd_events")]
pub unsafe extern "C" fn rs_do_all_autocmd_events(
    pat: *const c_char,
    once: bool,
    nested: c_int,
    cmd: *mut c_char,
    del: bool,
    group: c_int,
) {
    for event in 0..NUM_EVENTS {
        if rs_do_autocmd_event(event, pat, once, nested != 0, cmd, del, group) != nvim_autocmd_ok()
        {
            return;
        }
    }
}

/// Define or delete an autocommand for one event.
///
/// # Safety
/// `pat` and `cmd` must be valid NUL-terminated C strings.
#[export_name = "do_autocmd_event"]
#[allow(clippy::too_many_lines)]
pub unsafe extern "C" fn rs_do_autocmd_event(
    event: c_int,
    pat: *const c_char,
    once: bool,
    nested: bool,
    cmd: *const c_char,
    del: bool,
    group: c_int,
) -> c_int {
    let is_adding_cmd = *cmd != 0;
    let findgroup = if group == group::AUGROUP_ALL {
        nvim_autocmd_get_current_augroup()
    } else {
        group
    };

    // Delete all aupat for an event
    if *pat == 0 && del {
        rs_aucmd_del_for_event_and_group(event, findgroup);
        return nvim_autocmd_ok();
    }

    let mut buflocal_pat = [0u8; BUFLOCAL_PAT_LEN];
    let mut pat = pat;

    #[allow(clippy::cast_possible_truncation)]
    let mut patlen = rs_aucmd_pattern_length(pat) as c_int;

    while patlen != 0 {
        // Detect special <buffer[=X]> buffer-local patterns
        if rs_aupat_is_buflocal(pat, patlen) {
            let buflocal_nr = rs_aupat_get_buflocal_nr(pat, patlen);
            rs_aupat_normalize_buflocal_pat(
                buflocal_pat.as_mut_ptr().cast(),
                pat,
                patlen,
                buflocal_nr,
            );
            pat = buflocal_pat.as_ptr().cast();
            #[allow(clippy::cast_possible_truncation)]
            {
                patlen = c_strlen(pat) as c_int;
            }
        }

        if del {
            nvim_autocmd_del_matching(event, findgroup, pat, patlen);
        }

        if is_adding_cmd {
            nvim_autocmd_register_cmd(event, pat, patlen, group, once, nested, cmd);
        }

        pat = rs_aucmd_next_pattern(pat, patlen as usize);
        #[allow(clippy::cast_possible_truncation)]
        {
            patlen = rs_aucmd_pattern_length(pat) as c_int;
        }
    }

    rs_au_cleanup();
    nvim_autocmd_ok()
}

// =============================================================================
// Phase 8a: Simple Wrappers + Blocking
// =============================================================================

/// Block triggering autocommands until `unblock_autocmds` is called.
/// Can be used recursively, so long as it's symmetric.
#[unsafe(export_name = "block_autocmds")]
pub unsafe extern "C" fn rs_block_autocmds() {
    // Remember the value of v:termresponse.
    if rs_is_autocmd_blocked() == 0 {
        nvim_autocmd_set_old_termresponse(nvim_autocmd_get_vim_var_str(VV_TERMRESPONSE));
    }
    nvim_autocmd_inc_blocked();
}

/// Unblock autocommands. When v:termresponse was set while autocommands
/// were blocked, trigger the autocommands now.
#[unsafe(export_name = "unblock_autocmds")]
pub unsafe extern "C" fn rs_unblock_autocmds() {
    nvim_autocmd_dec_blocked();

    // When v:termresponse was set while autocommands were blocked, trigger
    // the autocommands now. Esp. useful when executing a shell command
    // during startup (nvim -d).
    if rs_is_autocmd_blocked() == 0
        && nvim_autocmd_get_vim_var_str(VV_TERMRESPONSE) != nvim_autocmd_get_old_termresponse()
    {
        rs_apply_autocmds(
            EVENT_TERMRESPONSE,
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            false,
            nvim_autocmd_get_curbuf_ptr(),
        );
    }
}

/// Execute autocommands for "event" and file name "fname".
#[unsafe(export_name = "apply_autocmds")]
pub unsafe extern "C" fn rs_apply_autocmds(
    event: c_int,
    fname: *mut c_char,
    fname_io: *mut c_char,
    force: bool,
    buf: *mut c_void,
) -> bool {
    nvim_autocmd_apply_autocmds_group(
        event,
        fname,
        fname_io,
        force,
        group::AUGROUP_ALL,
        buf,
        std::ptr::null_mut(),
        std::ptr::null_mut(),
    )
}

/// Like `apply_autocmds`, but with extra "eap" argument.
#[unsafe(export_name = "apply_autocmds_exarg")]
pub unsafe extern "C" fn rs_apply_autocmds_exarg(
    event: c_int,
    fname: *mut c_char,
    fname_io: *mut c_char,
    force: bool,
    buf: *mut c_void,
    eap: *mut c_void,
) -> bool {
    nvim_autocmd_apply_autocmds_group(
        event,
        fname,
        fname_io,
        force,
        group::AUGROUP_ALL,
        buf,
        eap,
        std::ptr::null_mut(),
    )
}

/// Like `apply_autocmds`, but handles the caller's retval.
///
/// If the script processing is being aborted or if retval is FAIL when inside
/// a try conditional, no autocommands are executed. If otherwise the
/// autocommands cause the script to be aborted, retval is set to FAIL.
#[unsafe(export_name = "apply_autocmds_retval")]
pub unsafe extern "C" fn rs_apply_autocmds_retval(
    event: c_int,
    fname: *mut c_char,
    fname_io: *mut c_char,
    force: bool,
    buf: *mut c_void,
    retval: *mut c_int,
) -> bool {
    if nvim_autocmd_should_abort(*retval) {
        return false;
    }

    let did_cmd = nvim_autocmd_apply_autocmds_group(
        event,
        fname,
        fname_io,
        force,
        group::AUGROUP_ALL,
        buf,
        std::ptr::null_mut(),
        std::ptr::null_mut(),
    );
    if did_cmd && nvim_autocmd_aborting() {
        *retval = FAIL;
    }
    did_cmd
}

// =============================================================================
// Phase 8e: Event Triggers + doautocmd
// =============================================================================

/// Check if a character ends an Ex command.
///
/// Returns true for NUL, '|', '"', '\n'.
#[inline]
fn ends_excmd(c: u8) -> bool {
    c == 0 || c == b'|' || c == b'"' || c == b'\n'
}

/// Execute `:doautocmd` — trigger autocommands for a given set of events.
///
/// Returns OK for success, FAIL for failure.
#[unsafe(export_name = "do_doautocmd")]
pub unsafe extern "C" fn rs_do_doautocmd(
    arg_start: *mut c_char,
    do_msg: bool,
    did_something: *mut bool,
) -> c_int {
    let mut arg: *const c_char = arg_start;
    let mut nothing_done = true;

    if !did_something.is_null() {
        *did_something = false;
    }

    // Check for a legal group name. If not, use AUGROUP_ALL.
    let group = rs_arg_augroup_get(&raw mut arg);

    if *arg == b'*' as c_char {
        nvim_autocmd_emsg(nvim_autocmd_get_e217());
        return FAIL;
    }

    // Scan over the events.
    // If we find an illegal name, return here, don't do anything.
    let fname = rs_arg_event_skip(arg, group != group::AUGROUP_ALL);
    if fname.is_null() {
        return FAIL;
    }

    let fname = skipwhite(fname);

    // Loop over the events.
    while *arg != 0 && !ends_excmd(*arg as u8) && !is_ascii_white(*arg as u8) {
        let result = event::rs_event_name2nr(arg);
        arg = result.end_ptr;
        if nvim_autocmd_apply_autocmds_group(
            result.event,
            fname,
            std::ptr::null_mut(),
            true,
            group,
            nvim_autocmd_get_curbuf_ptr(),
            std::ptr::null_mut(),
            std::ptr::null_mut(),
        ) {
            nothing_done = false;
        }
    }

    if nothing_done && do_msg && !nvim_autocmd_aborting() {
        nvim_autocmd_smsg_no_matching(arg_start);
    }
    if !did_something.is_null() {
        *did_something = !nothing_done;
    }

    if nvim_autocmd_aborting() {
        FAIL
    } else {
        OK
    }
}

// Phase 3: Migrated event trigger functions

/// Static state for do_autocmd_focusgained.
static FOCUS_RECURSIVE: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
static FOCUS_LAST_TIME: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);

/// Tracks whether to include group names during event-name expansion.
/// Replaces the C `static bool autocmd_include_groups`.
static AUTOCMD_INCLUDE_GROUPS: std::sync::atomic::AtomicBool =
    std::sync::atomic::AtomicBool::new(false);

/// Trigger FocusGained or FocusLost autocommand, and check timestamps when gaining focus.
///
/// # Safety
/// Calls into C. Must be called from the main Neovim thread.
#[unsafe(export_name = "do_autocmd_focusgained")]
pub unsafe extern "C" fn rs_do_autocmd_focusgained(gained: bool) {
    use std::sync::atomic::Ordering;

    if FOCUS_RECURSIVE.load(Ordering::Relaxed) {
        return; // disallow recursion
    }
    FOCUS_RECURSIVE.store(true, Ordering::Relaxed);

    let event = if gained {
        EVENT_FOCUSGAINED
    } else {
        EVENT_FOCUSLOST
    };
    rs_apply_autocmds(
        event,
        std::ptr::null_mut(),
        std::ptr::null_mut(),
        false,
        nvim_autocmd_get_curbuf_ptr(),
    );

    if gained {
        let last = FOCUS_LAST_TIME.load(Ordering::Relaxed);
        let now = os_now();
        if last + 2000 < now {
            check_timestamps(1);
            FOCUS_LAST_TIME.store(os_now(), Ordering::Relaxed);
        }
    }

    FOCUS_RECURSIVE.store(false, Ordering::Relaxed);
}

// xp_context values from cmdexpand_defs.h
const EXPAND_NOTHING: c_int = 0;
const EXPAND_FILES: c_int = 2;
const EXPAND_EVENTS: c_int = 10;

/// Set the completion context for `:autocmd` / `:doautocmd`.
///
/// Port of the C `set_context_in_autocmd` function.
/// Exported under `set_context_in_autocmd` to replace the C function.
///
/// # Safety
/// `xp` must be a valid `expand_T*`. `arg` must be a NUL-terminated C string.
/// Accesses `expand_T` fields via raw pointer offsets (xp_pattern@0, xp_context@8).
#[unsafe(export_name = "set_context_in_autocmd")]
pub unsafe extern "C" fn rs_set_context_in_autocmd(
    xp: *mut c_void,
    arg: *mut c_char,
    doautocmd: bool,
) -> *mut c_char {
    use std::sync::atomic::Ordering;

    // xp_pattern is at offset 0, xp_context is at offset 8 (64-bit ABI).
    // We use write_unaligned to avoid strict-alignment warnings for the c_int field.
    let xp_pattern_ptr = xp.cast::<*mut c_char>();
    let xp_context_raw = xp.cast::<u8>().add(8);
    macro_rules! set_xp_context {
        ($val:expr) => {
            std::ptr::write_unaligned(xp_context_raw.cast::<c_int>(), $val)
        };
    }

    AUTOCMD_INCLUDE_GROUPS.store(false, Ordering::Relaxed);

    // Save original position; arg_augroup_get advances arg past group name.
    let p_orig = arg;
    let mut argp: *const c_char = arg.cast_const();
    let group = rs_arg_augroup_get(&raw mut argp);
    let mut arg = argp.cast_mut();

    // If there is only a group name (not yet followed by whitespace), expand the group.
    if *arg == 0 && group != group::AUGROUP_ALL && !is_ascii_white(*arg.sub(1) as u8) {
        arg = p_orig;
        // group = AUGROUP_ALL; (not needed - only used for include_groups check below)
        // Fall through: arg is back at start, group effectively treated as AUGROUP_ALL
        // by setting include_groups based on AUGROUP_ALL condition
        AUTOCMD_INCLUDE_GROUPS.store(true, Ordering::Relaxed);
        set_xp_context!(EXPAND_EVENTS);
        *xp_pattern_ptr = arg;
        return std::ptr::null_mut();
    }

    // Skip over event name(s), tracking start of last event after a comma.
    let mut p = arg;
    while *p != 0 && !is_ascii_white(*p as u8) {
        if *p == b',' as c_char {
            arg = p.add(1); // next event starts after comma
        }
        p = p.add(1);
    }
    if *p == 0 {
        if group == group::AUGROUP_ALL {
            AUTOCMD_INCLUDE_GROUPS.store(true, Ordering::Relaxed);
        }
        set_xp_context!(EXPAND_EVENTS);
        *xp_pattern_ptr = arg;
        return std::ptr::null_mut();
    }

    // Skip over pattern (non-whitespace, or backslash-escaped whitespace).
    arg = skipwhite(p);
    while *arg != 0 && (!is_ascii_white(*arg as u8) || *arg.sub(1) == b'\\' as c_char) {
        arg = arg.add(1);
    }
    if *arg != 0 {
        return arg; // expand (next) command
    }

    if doautocmd {
        set_xp_context!(EXPAND_FILES);
    } else {
        set_xp_context!(EXPAND_NOTHING);
    }
    std::ptr::null_mut()
}

/// Return the augroup name or event name at `idx` for `:autocmd` completion.
///
/// Exported under `expand_get_event_name` to replace the C function.
///
/// # Safety
/// `xp` is unused but required for `CompleteListItemGetter` signature.
#[unsafe(export_name = "expand_get_event_name")]
pub unsafe extern "C" fn rs_expand_get_event_name(_xp: *mut c_void, idx: c_int) -> *mut c_char {
    use std::sync::atomic::Ordering;

    // Try to get a group name at position idx+1
    let name = rs_augroup_name(idx + 1);
    if !name.is_null() {
        if !AUTOCMD_INCLUDE_GROUPS.load(Ordering::Relaxed) || name == nvim_get_deleted_augroup() {
            // Return empty string (not NULL) to skip this entry
            return c"".as_ptr().cast_mut().cast();
        }
        return name.cast_mut();
    }

    // Past all groups: compute event index
    let next_id = nvim_get_next_augroup_id();
    let i = idx - next_id;
    if !(0..NUM_EVENTS).contains(&i) {
        return std::ptr::null_mut();
    }
    nvim_get_event_name(i).cast_mut()
}

/// Return the augroup name at `idx` for `:autocmd` augroup completion.
///
/// Exported under `expand_get_augroup_name` to replace the C function.
///
/// # Safety
/// `xp` is unused but required for `CompleteListItemGetter` signature.
#[unsafe(export_name = "expand_get_augroup_name")]
pub unsafe extern "C" fn rs_expand_get_augroup_name(_xp: *mut c_void, idx: c_int) -> *mut c_char {
    rs_augroup_name(idx + 1).cast_mut()
}

/// Return the event name at `idx` for `eventignorewin` / no-group event completion.
///
/// When `win` is true, only returns events with a non-positive sign (window-level events).
/// Exported under `get_event_name_no_group` to replace the C function.
///
/// # Safety
/// `xp` is unused but required for function signature compatibility.
#[unsafe(export_name = "get_event_name_no_group")]
pub unsafe extern "C" fn rs_get_event_name_no_group(
    _xp: *mut c_void,
    idx: c_int,
    win: bool,
) -> *mut c_char {
    if !(0..NUM_EVENTS).contains(&idx) {
        return std::ptr::null_mut();
    }

    if !win {
        return nvim_get_event_name(idx).cast_mut();
    }

    // Filter to window-level events (sign <= 0)
    let mut j: c_int = 0;
    for i in 0..NUM_EVENTS {
        let sign = nvim_get_event_sign(i);
        if sign <= 0 {
            j += 1;
            if j == idx + 1 {
                return nvim_get_event_name(i).cast_mut();
            }
        }
    }
    std::ptr::null_mut()
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
            assert!(rs_aupat_is_buflocal(buf.as_ptr(), 8));

            let buf_eq = CString::new("<buffer=1>").unwrap();
            assert!(rs_aupat_is_buflocal(buf_eq.as_ptr(), 10));

            let buf_abuf = CString::new("<buffer=abuf>").unwrap();
            assert!(rs_aupat_is_buflocal(buf_abuf.as_ptr(), 13));

            // Case insensitive
            let buf_upper = CString::new("<BUFFER>").unwrap();
            assert!(rs_aupat_is_buflocal(buf_upper.as_ptr(), 8));

            // Invalid patterns
            let short = CString::new("<buf>").unwrap();
            assert!(!rs_aupat_is_buflocal(short.as_ptr(), 5));

            let no_end = CString::new("<buffer").unwrap();
            assert!(!rs_aupat_is_buflocal(no_end.as_ptr(), 7));

            let wrong_start = CString::new("buffer>").unwrap();
            assert!(!rs_aupat_is_buflocal(wrong_start.as_ptr(), 7));

            let normal = CString::new("*.c").unwrap();
            assert!(!rs_aupat_is_buflocal(normal.as_ptr(), 3));
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
            // Null pointer should return false
            assert!(!rs_aupat_is_buflocal(std::ptr::null(), 8));
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
