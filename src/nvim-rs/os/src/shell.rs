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

    // Option globals
    static p_sh: *mut c_char;
    static p_shcf: *mut c_char;
    static p_sxq: *mut c_char;
    static p_sxe: *mut c_char;
}

// TAB character (must match C's TAB macro = 9)
const TAB: c_char = 9;

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
