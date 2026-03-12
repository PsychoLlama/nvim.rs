//! Syntime profiling subsystem for Neovim syntax highlighting.
//!
//! Migrated from syntax_accessors.c: ex_syntime, syntime_clear,
//! syntime_report, syn_clear_time, syn_compare_syntime, get_syntime_arg.

use std::ffi::{c_char, c_int, c_void};

use crate::types::{SynBlockHandle, SynPatHandle, WinHandle};

// =============================================================================
// FFI declarations
// =============================================================================

/// Proftime type (nanoseconds, matches C proftime_T which is uint64_t)
type Proftime = u64;

extern "C" {
    // syn_time_on toggle
    fn nvim_syn_get_syn_time_on() -> c_int;
    fn nvim_syn_set_syn_time_on(val: c_int);

    // synblock pattern access (synblock_T is not repr(C) yet)
    fn nvim_synblock_get_pattern_count(block: SynBlockHandle) -> c_int;
    fn nvim_synblock_get_pattern(block: SynBlockHandle, idx: c_int) -> SynPatHandle;

    // syntax_present(curwin)
    fn nvim_syn_syntax_present_curwin() -> c_int;

    // curwin synblock
    fn nvim_get_curwin() -> WinHandle;
    fn nvim_syn_get_curwin_synblock() -> SynBlockHandle;

    // got_int flag
    fn nvim_syn_get_got_int() -> c_int;

    // Columns
    fn nvim_syn_get_columns() -> c_int;

    // Profile arithmetic (from profile Rust crate, exported via #[export_name])
    fn profile_zero() -> Proftime;
    fn profile_add(tm1: Proftime, tm2: Proftime) -> Proftime;
    fn profile_divide(tm: Proftime, count: c_int) -> Proftime;
    fn profile_cmp(tm1: Proftime, tm2: Proftime) -> c_int;
    fn profile_msg(tm: Proftime) -> *const c_char;

    // Highlight group name
    fn nvim_syn_highlight_group_name(idx: c_int) -> *mut c_char;

    // Message output
    fn msg_puts(s: *const c_char);
    fn msg_puts_title(s: *const c_char);
    fn msg_outnum(n: c_int);
    fn msg_advance(col: c_int);
    fn msg_outtrans(s: *const c_char, hl_id: c_int, hist: bool) -> c_int;
    fn msg_outtrans_len(s: *const c_char, len: c_int, hl_id: c_int, hist: bool) -> c_int;
    fn msg(s: *const c_char, hl_id: c_int) -> c_int;

    // Error message
    fn semsg(fmt: *const c_char, ...);

    // EAP accessors
    fn nvim_syn_get_eap_arg(eap: *const c_void) -> *mut c_char;
}

// Static message strings
static MSG_NO_ITEMS: &[u8] = b"No Syntax items defined for this buffer\0";
static EMSG_INVARG2: &[u8] = b"E474: Invalid argument: %s\0";
static MSG_SYNTIME_HEADER: &[u8] =
    b"  TOTAL      COUNT  MATCH   SLOWEST     AVERAGE   NAME               PATTERN\0";
static MSG_NEWLINE: &[u8] = b"\n\0";
static MSG_SPACE: &[u8] = b" \0";

// Syntime argument names (for tab completion)
static SYNTIME_ARGS: [&[u8]; 4] = [b"on\0", b"off\0", b"clear\0", b"report\0"];

// =============================================================================
// Entry representing one pattern's timing data (replaces C time_entry_T)
// =============================================================================

struct TimeEntry {
    total: Proftime,
    count: i32,
    match_count: i32,
    slowest: Proftime,
    average: Proftime,
    id: i16,
    pattern: *const c_char,
}

// =============================================================================
// Public Rust functions exported to C
// =============================================================================

/// Handle `:syntime {on|off|clear|report}` command.
///
/// # Safety
/// Must be called from main thread with valid eap pointer.
#[export_name = "ex_syntime"]
pub unsafe extern "C" fn rs_ex_syntime(eap: *mut c_void) {
    let arg = nvim_syn_get_eap_arg(eap);
    if arg.is_null() {
        return;
    }

    // Compare arg string
    let arg_bytes = std::ffi::CStr::from_ptr(arg).to_bytes();

    if arg_bytes == b"on" {
        nvim_syn_set_syn_time_on(1);
    } else if arg_bytes == b"off" {
        nvim_syn_set_syn_time_on(0);
    } else if arg_bytes == b"clear" {
        syntime_clear_impl();
    } else if arg_bytes == b"report" {
        syntime_report_impl();
    } else {
        semsg(EMSG_INVARG2.as_ptr().cast(), arg);
    }
}

/// Return tab-completion argument string for `:syntime` by index.
///
/// # Safety
/// Returned pointer is to a static string, valid forever.
#[export_name = "get_syntime_arg"]
#[must_use]
pub unsafe extern "C" fn rs_get_syntime_arg(_xp: *mut c_void, idx: c_int) -> *mut c_char {
    if idx >= 0 && (idx as usize) < SYNTIME_ARGS.len() {
        // Cast away const for C API compatibility (callers treat as const)
        SYNTIME_ARGS[idx as usize]
            .as_ptr()
            .cast::<c_char>()
            .cast_mut()
    } else {
        std::ptr::null_mut()
    }
}

// =============================================================================
// Internal implementation
// =============================================================================

/// Clear timing data for all patterns in the current buffer.
unsafe fn syntime_clear_impl() {
    if nvim_syn_syntax_present_curwin() == 0 {
        msg(MSG_NO_ITEMS.as_ptr().cast(), 0);
        return;
    }
    let block = nvim_syn_get_curwin_synblock();
    let count = nvim_synblock_get_pattern_count(block);
    for idx in 0..count {
        let pat = nvim_synblock_get_pattern(block, idx);
        if !pat.is_null() {
            let st_ptr = &mut (*pat.as_ptr()).sp_time as *mut _ as *mut c_void;
            crate::line_init::rs_syn_clear_time(st_ptr);
        }
    }
}

/// Collect, sort, and display timing report for the current buffer.
unsafe fn syntime_report_impl() {
    if nvim_syn_syntax_present_curwin() == 0 {
        msg(MSG_NO_ITEMS.as_ptr().cast(), 0);
        return;
    }

    let block = nvim_syn_get_curwin_synblock();
    let pat_count = nvim_synblock_get_pattern_count(block);

    // Collect entries with count > 0
    let mut entries: Vec<TimeEntry> = Vec::new();
    let mut total_total: Proftime = profile_zero();
    let mut total_count: i32 = 0;

    for idx in 0..pat_count {
        let pat = nvim_synblock_get_pattern(block, idx);
        if pat.is_null() {
            continue;
        }
        let pp = pat.as_ptr();
        let count = (*pp).sp_time.count;
        if count > 0 {
            let total = (*pp).sp_time.total;
            total_total = profile_add(total_total, total);
            let average = profile_divide(total, count);
            entries.push(TimeEntry {
                total,
                count,
                match_count: (*pp).sp_time.match_,
                slowest: (*pp).sp_time.slowest,
                average,
                id: (*pp).sp_syn.id,
                pattern: (*pp).sp_pattern as *const c_char,
            });
            total_count += count;
        }
    }

    // Sort by total time descending (profile_cmp returns <0 if tm2 < tm1)
    entries.sort_by(|a, b| {
        let cmp = profile_cmp(a.total, b.total);
        cmp.cmp(&0)
    });

    // Output header
    msg_puts_title(MSG_SYNTIME_HEADER.as_ptr().cast());
    msg_puts(MSG_NEWLINE.as_ptr().cast());

    for entry in &entries {
        if nvim_syn_get_got_int() != 0 {
            break;
        }

        msg_puts(profile_msg(entry.total));
        msg_puts(MSG_SPACE.as_ptr().cast());
        msg_advance(13);
        msg_outnum(entry.count);
        msg_puts(MSG_SPACE.as_ptr().cast());
        msg_advance(20);
        msg_outnum(entry.match_count);
        msg_puts(MSG_SPACE.as_ptr().cast());
        msg_advance(26);
        msg_puts(profile_msg(entry.slowest));
        msg_puts(MSG_SPACE.as_ptr().cast());
        msg_advance(38);
        msg_puts(profile_msg(entry.average));
        msg_puts(MSG_SPACE.as_ptr().cast());
        msg_advance(50);
        let group_name = nvim_syn_highlight_group_name((entry.id as c_int) - 1);
        msg_outtrans(group_name, 0, false);
        msg_puts(MSG_SPACE.as_ptr().cast());

        msg_advance(69);
        let columns = nvim_syn_get_columns();
        let len = if columns < 80 {
            20 // will wrap anyway
        } else {
            columns - 70
        };
        let pattern = entry.pattern;
        let patlen = if pattern.is_null() {
            0
        } else {
            libc_strlen(pattern) as c_int
        };
        let len = len.min(patlen);
        msg_outtrans_len(pattern, len, 0, false);
        msg_puts(MSG_NEWLINE.as_ptr().cast());
    }

    if nvim_syn_get_got_int() == 0 {
        msg_puts(MSG_NEWLINE.as_ptr().cast());
        msg_puts(profile_msg(total_total));
        msg_advance(13);
        msg_outnum(total_count);
        msg_puts(MSG_NEWLINE.as_ptr().cast());
    }
}

/// Compute strlen of a C string without using libc crate.
unsafe fn libc_strlen(s: *const c_char) -> usize {
    let mut len = 0usize;
    while *s.add(len) != 0 {
        len += 1;
    }
    len
}
