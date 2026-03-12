//! Shell command utilities
//!
//! Implements shell argument building, argv management, and stream callbacks.
//! Migrated from src/nvim/os/shell.c.

use std::ffi::{c_char, c_int, c_void, CStr};
use std::ptr;

// =============================================================================
// FFI declarations
// =============================================================================

extern "C" {
    #[link_name = "xfree"]
    fn nvim_xfree(ptr: *mut c_char);
    #[link_name = "xmalloc"]
    fn nvim_xmalloc(size: usize) -> *mut c_char;
    #[link_name = "xcalloc"]
    fn nvim_xcalloc(count: usize, size: usize) -> *mut c_char;
    #[link_name = "xstrlcat"]
    fn nvim_xstrlcat(dst: *mut c_char, src: *const c_char, dstsize: usize) -> usize;
    #[link_name = "xstrdup"]
    fn nvim_xstrdup(s: *const c_char) -> *mut c_char;
    #[link_name = "vim_strnsave_unquoted"]
    fn c_vim_strnsave_unquoted(string: *const c_char, length: usize) -> *mut c_char;
    #[link_name = "skipwhite"]
    fn c_skipwhite(s: *const c_char) -> *mut c_char;
    #[link_name = "vim_strsave_escaped_ext"]
    fn c_vim_strsave_escaped_ext(
        string: *const c_char,
        esc_chars: *const c_char,
        cc: c_char,
        bsl: bool,
    ) -> *mut c_char;
    #[link_name = "vim_snprintf"]
    fn c_vim_snprintf(str_: *mut c_char, str_m: usize, fmt: *const c_char, ...) -> c_int;
    #[link_name = "msg_schedule_semsg"]
    fn c_msg_schedule_semsg(fmt: *const c_char, ...);
    #[link_name = "stream_may_close"]
    fn c_stream_may_close(stream: *mut c_void);
    #[link_name = "uv_err_name"]
    fn c_uv_err_name(err: c_int) -> *const c_char;

    #[link_name = "xrealloc"]
    fn nvim_xrealloc(ptr: *mut c_void, size: usize) -> *mut c_void;
    #[link_name = "nvim_stream_get_fd"]
    fn c_nvim_stream_get_fd(stream: *mut c_void) -> c_int;
    #[link_name = "os_hrtime"]
    fn c_os_hrtime() -> u64;
    #[link_name = "msg_puts"]
    fn c_msg_puts(s: *const c_char);
    #[link_name = "msg_putchar"]
    fn c_msg_putchar(c: c_int) -> c_int;
    #[link_name = "ui_flush"]
    fn c_ui_flush();
    #[link_name = "msg_ext_set_kind"]
    fn c_msg_ext_set_kind(kind: *const c_char);
    #[link_name = "msg_putchar_hl"]
    fn c_msg_putchar_hl(c: c_int, hl_id: c_int);
    #[link_name = "utfc_ptr2len_len"]
    fn c_utfc_ptr2len_len(p: *const c_char, size: c_int) -> c_int;
    #[link_name = "msg_outtrans_len"]
    fn c_msg_outtrans_len(msgstr: *const c_char, len: c_int, hl_id: c_int, hist: bool) -> c_int;

    // Option globals
    static p_sh: *mut c_char;
    static p_shcf: *mut c_char;
    static p_sxq: *mut c_char;
    static p_sxe: *mut c_char;

    // UTF-8 length table (0 means invalid lead byte)
    static utf8len_tab_zero: [u8; 256];
}

// TAB character (must match C's TAB macro = 9)
const TAB: c_char = 9;
// Bell character (must match C's BELL macro = 7)
const BELL: c_char = 7;

// Throttle threshold: 10KB (must match C's OUT_DATA_THRESHOLD)
const OUT_DATA_THRESHOLD: usize = 1024 * 10;
// Ring buffer size: half the threshold
const MAX_CHUNK_SIZE: usize = OUT_DATA_THRESHOLD / 2;
// 1 second in nanoseconds
const NS_1_SECOND: u64 = 1_000_000_000;

// Highlight group IDs for shell output (from hlf_T enum in highlight_defs.h).
// HLF_SE = 72, HLF_SO = 73. These must match the C enum values.
const HLF_SE: c_int = 72; // stderr messages (from shell)
const HLF_SO: c_int = 73; // stdout messages (from shell)

// Standard file descriptor numbers
const STDOUT_FILENO: c_int = 1;
const STDERR_FILENO: c_int = 2;

/// StringBuilder: C kvec_t(char) layout.
///
/// Must match: struct { size_t size; size_t capacity; char *items; }
#[repr(C)]
struct StringBuilder {
    size: usize,
    capacity: usize,
    items: *mut c_char,
}

// =============================================================================
// Internal helpers (not exported - were `static` in C)
// =============================================================================

/// Computes the length of a null-terminated C string pointer.
/// Returns 0 for null pointers.
unsafe fn cstr_len(s: *const c_char) -> usize {
    if s.is_null() {
        return 0;
    }
    // SAFETY: caller guarantees s is a valid null-terminated C string
    unsafe { CStr::from_ptr(s) }.to_bytes().len()
}

/// Calculates the length of a shell word.
///
/// A word ends at unquoted whitespace. Double-quoted regions suppress
/// whitespace. A backslash inside a quoted region escapes the next char.
fn word_length(str: *const c_char) -> usize {
    let mut p = str;
    let mut inquote = false;
    let mut length = 0usize;

    while unsafe { *p } != 0
        && (inquote || (unsafe { *p } != b' ' as c_char && unsafe { *p } != TAB))
    {
        if unsafe { *p } == b'"' as c_char {
            inquote = !inquote;
        } else if unsafe { *p } == b'\\' as c_char && inquote {
            p = unsafe { p.add(1) };
            length += 1;
        }
        p = unsafe { p.add(1) };
        length += 1;
    }

    length
}

/// Parses a command string into a sequence of words.
///
/// If `argv` is non-null, fills it with allocated copies of each word.
/// Returns the number of words parsed.
fn tokenize(str: *const c_char, argv: *mut *mut c_char) -> usize {
    let mut argc = 0usize;
    let mut p = str;

    while unsafe { *p } != 0 {
        let len = word_length(p);

        if !argv.is_null() {
            let word = unsafe { c_vim_strnsave_unquoted(p, len) };
            unsafe {
                *argv.add(argc) = word;
            }
        }

        argc += 1;
        p = unsafe { c_skipwhite(p.add(len)) };
    }

    argc
}

/// Applies 'shellxescape' (p_sxe) and 'shellxquote' (p_sxq) to a command.
///
/// Returns an allocated string. Caller must free with `xfree`.
fn shell_xescape_xquote(cmd: *const c_char) -> *mut c_char {
    let sxq = unsafe { p_sxq };
    if sxq.is_null() || unsafe { *sxq } == 0 {
        return unsafe { nvim_xstrdup(cmd) };
    }

    let sxe = unsafe { p_sxe };

    let ecmd_allocated;
    let ecmd: *const c_char = {
        let sxq_is_open_paren =
            !sxq.is_null() && unsafe { *sxq } == b'(' as c_char && unsafe { *sxq.add(1) } == 0;
        if !sxe.is_null() && unsafe { *sxe } != 0 && sxq_is_open_paren {
            ecmd_allocated = true;
            unsafe { c_vim_strsave_escaped_ext(cmd, sxe, b'^' as c_char, false) }
        } else {
            ecmd_allocated = false;
            cmd
        }
    };

    // Compute size: ecmd + sxq * 2 + extra parens + NUL
    let ecmd_len = unsafe { cstr_len(ecmd) };
    let sxq_len = unsafe { cstr_len(sxq) };
    let ncmd_size = ecmd_len + sxq_len * 2 + 1;
    let ncmd = unsafe { nvim_xmalloc(ncmd_size) };

    let sxq_bytes = unsafe { CStr::from_ptr(sxq) }.to_bytes();

    if sxq_bytes == b"(" {
        unsafe {
            c_vim_snprintf(ncmd, ncmd_size, c"(%s)".as_ptr(), ecmd);
        }
    } else if sxq_bytes == b"\"(" {
        unsafe {
            c_vim_snprintf(ncmd, ncmd_size, c"\"(%s)\"".as_ptr(), ecmd);
        }
    } else {
        unsafe {
            c_vim_snprintf(ncmd, ncmd_size, c"%s%s%s".as_ptr(), sxq, ecmd, sxq);
        }
    }

    if ecmd_allocated {
        unsafe { nvim_xfree(ecmd.cast_mut()) };
    }

    ncmd
}

// =============================================================================
// Exported functions
// =============================================================================

/// Builds the argument vector for running the user-configured 'shell' (p_sh).
///
/// # Safety
///
/// `cmd` and `extra_args` must be valid C strings or NULL.
#[export_name = "shell_build_argv"]
pub unsafe extern "C" fn rs_shell_build_argv(
    cmd: *const c_char,
    extra_args: *const c_char,
) -> *mut *mut c_char {
    let sh = unsafe { p_sh };
    let shcf = unsafe { p_shcf };

    let argc = tokenize(sh, ptr::null_mut())
        + if cmd.is_null() {
            0
        } else {
            tokenize(shcf, ptr::null_mut())
        };

    #[allow(clippy::cast_ptr_alignment)]
    let rv = unsafe { nvim_xmalloc((argc + 4) * size_of::<*mut c_char>()) }.cast::<*mut c_char>();

    // Split 'shell' into argv[0..]
    let mut i = tokenize(sh, rv);

    if !extra_args.is_null() {
        unsafe {
            *rv.add(i) = nvim_xstrdup(extra_args);
        }
        i += 1;
    }

    if !cmd.is_null() {
        i += tokenize(shcf, unsafe { rv.add(i) });
        unsafe {
            *rv.add(i) = shell_xescape_xquote(cmd);
        }
        i += 1;
    }

    unsafe {
        *rv.add(i) = ptr::null_mut();
    }

    assert!(!unsafe { *rv }.is_null());

    rv
}

/// Releases the memory allocated by `shell_build_argv`.
///
/// # Safety
///
/// `argv` must be a null-terminated array of C-string pointers allocated with
/// xmalloc, or NULL.
#[export_name = "shell_free_argv"]
pub unsafe extern "C" fn rs_shell_free_argv(argv: *mut *mut c_char) {
    if argv.is_null() {
        return;
    }
    let mut p = argv;
    while !unsafe { *p }.is_null() {
        unsafe {
            nvim_xfree(*p);
            p = p.add(1);
        }
    }
    unsafe { nvim_xfree(argv.cast::<c_char>()) };
}

/// Joins shell arguments from `argv` into a new string.
///
/// If the result is too long it is truncated with ellipsis ("...").
///
/// # Safety
///
/// `argv` must be a null-terminated array of valid C strings.
#[export_name = "shell_argv_to_str"]
pub unsafe extern "C" fn rs_shell_argv_to_str(argv: *mut *mut c_char) -> *mut c_char {
    let maxsize: usize = 256;
    let rv = unsafe { nvim_xcalloc(maxsize, 1) };
    if argv.is_null() || unsafe { *argv }.is_null() {
        return rv;
    }

    let mut p = argv;
    let mut n = 0usize;
    while !unsafe { *p }.is_null() {
        unsafe {
            nvim_xstrlcat(rv, c"'".as_ptr(), maxsize);
            nvim_xstrlcat(rv, *p, maxsize);
            n = nvim_xstrlcat(rv, c"' ".as_ptr(), maxsize);
        }
        if n >= maxsize {
            break;
        }
        p = unsafe { p.add(1) };
    }

    if n < maxsize {
        // Remove trailing space
        if n > 0 {
            unsafe { *rv.add(n - 1) = 0 };
        }
    } else {
        // Truncate with ellipsis: ".../bin/bash 'foo' 'bar'..."
        unsafe {
            *rv.add(maxsize - 4) = b'.' as c_char;
            *rv.add(maxsize - 3) = b'.' as c_char;
            *rv.add(maxsize - 2) = b'.' as c_char;
            *rv.add(maxsize - 1) = 0;
        }
    }

    rv
}

/// Stream write callback: logs errors and closes the stream.
///
/// Called when a write to a shell's stdin stream completes (possibly with an
/// error).
///
/// # Safety
///
/// `stream` must be a valid `Stream *`.
#[export_name = "shell_write_cb"]
pub unsafe extern "C" fn rs_shell_write_cb(stream: *mut c_void, _data: *mut c_void, status: c_int) {
    if status != 0 {
        let err_name = unsafe { c_uv_err_name(status) };
        unsafe {
            c_msg_schedule_semsg(
                c"E5677: Error writing input to shell-command: %s".as_ptr(),
                err_name,
            );
        }
    }
    unsafe { c_stream_may_close(stream) };
}

// =============================================================================
// Phase 4: Output throttling and ring-buffer (migrated from shell.c)
// =============================================================================

/// Append `len` bytes from `data` to a StringBuilder (implements kv_concat_len).
///
/// # Safety
///
/// `sb` must be a valid non-null pointer to a `StringBuilder`. `data` must
/// point to at least `len` bytes.
unsafe fn sb_concat_len(sb: *mut StringBuilder, data: *const c_char, len: usize) {
    if len == 0 {
        return;
    }
    let sb = unsafe { &mut *sb };
    // Grow if needed (mirrors kv_ensure_space / kv_resize_full)
    if sb.capacity < sb.size + len {
        let mut new_cap = sb.size + len;
        // kv_roundup32: round up to next power of two
        new_cap = new_cap.next_power_of_two();
        sb.items = unsafe { nvim_xrealloc(sb.items.cast::<c_void>(), new_cap).cast::<c_char>() };
        sb.capacity = new_cap;
    }
    // Copy data into items[size..]
    unsafe {
        ptr::copy_nonoverlapping(data, sb.items.add(sb.size), len);
    }
    sb.size += len;
}

/// RStream callback for `system()`: appends data to the output StringBuilder.
///
/// # Safety
///
/// `data` must be a valid `StringBuilder *` allocated by `do_os_system`.
#[export_name = "system_data_cb"]
pub unsafe extern "C" fn rs_system_data_cb(
    _stream: *mut c_void,
    buf: *const c_char,
    count: usize,
    data: *mut c_void,
    _eof: bool,
) -> usize {
    unsafe { sb_concat_len(data.cast::<StringBuilder>(), buf, count) };
    count
}

/// Decides whether output should be throttled (skipped and pulsed).
///
/// size=0 resets internal state and returns the previous decision.
/// Returns true if output should be skipped and a pulse was displayed.
#[export_name = "out_data_decide_throttle"]
pub unsafe extern "C" fn out_data_decide_throttle(size: usize) -> bool {
    // Static mutable state (single-threaded main loop, matches C semantics)
    static mut STARTED: u64 = 0;
    static mut RECEIVED: usize = 0;
    static mut VISIT: usize = 0;
    static mut PULSE_MSG: [c_char; 4] = [b' ' as c_char, b' ' as c_char, b' ' as c_char, 0];

    if size == 0 {
        let previous = unsafe { VISIT > 0 };
        unsafe {
            STARTED = 0;
            RECEIVED = 0;
            VISIT = 0;
        }
        return previous;
    }

    unsafe { RECEIVED += size };
    let received = unsafe { RECEIVED };
    let started = unsafe { STARTED };
    let visit = unsafe { VISIT };

    if received < OUT_DATA_THRESHOLD
        // Display at least the first chunk even if it is big
        || (started == 0 && received < size + 1000)
    {
        return false;
    } else if visit == 0 {
        unsafe { STARTED = c_os_hrtime() };
    } else {
        let since = unsafe { c_os_hrtime() } - started;
        if since < visit as u64 * (NS_1_SECOND / 10) {
            return true;
        }
        if since > 3 * NS_1_SECOND {
            unsafe {
                RECEIVED = 0;
                VISIT = 0;
            }
            return false;
        }
    }

    unsafe { VISIT += 1 };
    let visit = unsafe { VISIT };
    let tick = visit % 4;
    unsafe {
        PULSE_MSG[0] = if tick > 0 {
            b'.' as c_char
        } else {
            b' ' as c_char
        };
        PULSE_MSG[1] = if tick > 1 {
            b'.' as c_char
        } else {
            b' ' as c_char
        };
        PULSE_MSG[2] = if tick > 2 {
            b'.' as c_char
        } else {
            b' ' as c_char
        };
    }
    if visit == 1 {
        unsafe { c_msg_puts(c"...\n".as_ptr()) };
    }
    unsafe { c_msg_putchar(c_int::from(b'\r')) }; // cursor to start of line
    unsafe { c_msg_puts((&raw const PULSE_MSG).cast::<c_char>()) };
    unsafe { c_msg_putchar(c_int::from(b'\r')) };
    unsafe { c_ui_flush() };
    true
}

/// Saves output in a quasi-ring-buffer so the last ~page is always displayed.
///
/// Init mode: output=NULL, size=0 → reset.
/// Print mode: output=NULL, size=SIZE_MAX → display saved data.
/// Save mode: output non-null → store/append data.
#[export_name = "out_data_ring"]
pub unsafe extern "C" fn out_data_ring(output: *const c_char, size: usize) {
    static mut LAST_SKIPPED: [c_char; MAX_CHUNK_SIZE] = [0; MAX_CHUNK_SIZE];
    static mut LAST_SKIPPED_LEN: usize = 0;

    if output.is_null() && size == 0 {
        // Init mode
        unsafe { LAST_SKIPPED_LEN = 0 };
        return;
    }

    if output.is_null() && size == usize::MAX {
        // Print mode
        unsafe {
            out_data_append_to_screen(
                (&raw const LAST_SKIPPED).cast::<c_char>(),
                &raw mut LAST_SKIPPED_LEN,
                STDOUT_FILENO,
                true,
            );
        }
        return;
    }

    // Save mode
    if size >= MAX_CHUNK_SIZE {
        let start = size - MAX_CHUNK_SIZE;
        unsafe {
            ptr::copy_nonoverlapping(
                output.add(start),
                (&raw mut LAST_SKIPPED).cast::<c_char>(),
                MAX_CHUNK_SIZE,
            );
            LAST_SKIPPED_LEN = MAX_CHUNK_SIZE;
        }
    } else if size > 0 {
        let last_len = unsafe { LAST_SKIPPED_LEN };
        let keep_len = last_len.min(MAX_CHUNK_SIZE - size);
        let keep_start = last_len - keep_len;
        unsafe {
            if keep_start > 0 {
                ptr::copy(
                    (&raw const LAST_SKIPPED).cast::<c_char>().add(keep_start),
                    (&raw mut LAST_SKIPPED).cast::<c_char>(),
                    keep_len,
                );
            }
            ptr::copy_nonoverlapping(
                output,
                (&raw mut LAST_SKIPPED).cast::<c_char>().add(keep_len),
                size,
            );
            LAST_SKIPPED_LEN = keep_len + size;
        }
    }
}

/// Appends shell output data to the screen.
///
/// # Safety
///
/// `output` must point to at least `*count` bytes. `count` is updated if
/// truncated due to incomplete UTF-8 at end.
#[allow(clippy::cast_sign_loss, clippy::cast_possible_truncation)]
unsafe fn out_data_append_to_screen(
    output: *const c_char,
    count: *mut usize,
    fd: c_int,
    eof: bool,
) {
    let mut p = output;
    let end = unsafe { output.add(*count) };
    let kind = if fd == STDERR_FILENO {
        c"shell_err".as_ptr()
    } else {
        c"shell_out".as_ptr()
    };
    unsafe { c_msg_ext_set_kind(kind) };
    while p < end {
        let ch = unsafe { *p };
        if ch == b'\n' as c_char || ch == b'\r' as c_char || ch == TAB || ch == BELL {
            let hl = if fd == STDERR_FILENO { HLF_SE } else { HLF_SO };
            unsafe { c_msg_putchar_hl(c_int::from(ch as u8), hl) };
            p = unsafe { p.add(1) };
        } else {
            let remaining = unsafe { end.offset_from(p) } as c_int;
            let i = if ch != 0 {
                unsafe { c_utfc_ptr2len_len(p, remaining) }
            } else {
                1
            };
            if !eof
                && i == 1
                && c_int::from(unsafe { utf8len_tab_zero[usize::from(*(p.cast::<u8>()))] })
                    > remaining
            {
                unsafe { *count = p.offset_from(output) as usize };
                unsafe { c_ui_flush() };
                return;
            }
            let hl = if fd == STDERR_FILENO { HLF_SE } else { HLF_SO };
            unsafe { c_msg_outtrans_len(p, i, hl, false) };
            p = unsafe { p.add(i as usize) };
        }
    }
    unsafe { c_ui_flush() };
}

/// RStream callback for `:!` shell output: throttle and display.
///
/// # Safety
///
/// `stream` must be a valid `RStream *`.
#[export_name = "out_data_cb"]
pub unsafe extern "C" fn rs_out_data_cb(
    stream: *mut c_void,
    ptr: *const c_char,
    count: usize,
    _data: *mut c_void,
    eof: bool,
) -> usize {
    let mut count = count;
    if count > 0 && unsafe { out_data_decide_throttle(count) } {
        // Skip; save for later display
        unsafe { out_data_ring(ptr, count) };
    } else if count > 0 {
        let fd = unsafe { c_nvim_stream_get_fd(stream) };
        unsafe { out_data_append_to_screen(ptr, &raw mut count, fd, eof) };
    }
    count
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_word_length_basic() {
        let s = b"hello world\0";
        assert_eq!(word_length(s.as_ptr().cast()), 5);
    }

    #[test]
    fn test_word_length_quoted() {
        // "hello world" is 13 chars including quotes
        let s = b"\"hello world\" foo\0";
        assert_eq!(word_length(s.as_ptr().cast()), 13);
    }

    #[test]
    fn test_word_length_empty() {
        let s = b"\0";
        assert_eq!(word_length(s.as_ptr().cast()), 0);
    }

    #[test]
    fn test_word_length_tab_separator() {
        let s = b"foo\tbar\0";
        assert_eq!(word_length(s.as_ptr().cast()), 3);
    }

    // Note: tokenize tests are disabled because tokenize calls c_skipwhite (C symbol)
    // which requires linking with nvim. These are tested via nvim's integration tests.
}
