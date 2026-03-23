//! Option initialization utilities
//!
//! This module provides Rust implementations of helper functions used during
//! Neovim's option initialization. The actual initialization sequence remains
//! in C due to global state dependencies.

#![allow(clippy::missing_safety_doc)] // FFI functions safety is implicit
#![allow(clippy::cast_possible_wrap)] // FFI with C char types

use std::ffi::{c_char, c_int};

use crate::opt_index::{
    K_OPT_BACKUPDIR, K_OPT_DIRECTORY, K_OPT_PACKPATH, K_OPT_RUNTIMEPATH, K_OPT_UNDODIR,
    K_OPT_VIEWDIR,
};

// =============================================================================
// C Function Declarations
// =============================================================================

#[allow(dead_code)] // FFI functions used when linked with C
extern "C" {
    fn xmalloc(size: usize) -> *mut c_char;
    fn xfree(ptr: *mut c_char);
}

// =============================================================================
// Help Language Default
// =============================================================================

/// Result of processing a language string for 'helplang'.
#[repr(C)]
pub struct HelplangResult {
    /// Two-letter language code (or empty if invalid)
    pub code: [c_char; 3],
    /// Whether the result is valid
    pub valid: c_int,
}

/// Process a language string to extract the 'helplang' default.
///
/// Converts locale strings to two-letter language codes:
/// - "zh_CN" -> "cn"
/// - "zh_TW" -> "tw"
/// - "C", "C.UTF-8", etc. -> "en"
/// - "en_US" -> "en"
/// - Other -> first two letters lowercased
///
/// # Arguments
/// * `lang` - The locale/language string (e.g., from LANG environment)
///
/// # Returns
/// A struct containing the two-letter code and validity flag.
#[no_mangle]
pub unsafe extern "C" fn rs_compute_helplang(lang: *const c_char) -> HelplangResult {
    let mut result = HelplangResult {
        code: [0, 0, 0],
        valid: 0,
    };

    if lang.is_null() || *lang == 0 {
        return result;
    }

    // Get length
    let mut len: usize = 0;
    let mut p = lang;
    while *p != 0 {
        len += 1;
        p = p.add(1);
    }

    let b0 = *lang as u8;

    // Check for C locale (C, C.UTF-8, etc.) - handle single char case
    if b0 == b'C' && (len == 1 || *lang.add(1) as u8 == b'.') {
        result.code[0] = b'e' as c_char;
        result.code[1] = b'n' as c_char;
        result.valid = 1;
        return result;
    }

    // Need at least 2 characters for other checks
    if len < 2 {
        return result;
    }

    let b1 = *lang.add(1) as u8;

    // Check for zh_CN or zh_TW
    if len >= 5 && b0 == b'z' && b1 == b'h' && *lang.add(2) as u8 == b'_' {
        // zh_CN -> cn, zh_TW -> tw
        result.code[0] = (*lang.add(3) as u8).to_ascii_lowercase() as c_char;
        result.code[1] = (*lang.add(4) as u8).to_ascii_lowercase() as c_char;
        result.valid = 1;
        return result;
    }

    // POSIX locale
    if len >= 5
        && b0 == b'P'
        && b1 == b'O'
        && *lang.add(2) as u8 == b'S'
        && *lang.add(3) as u8 == b'I'
        && *lang.add(4) as u8 == b'X'
    {
        result.code[0] = b'e' as c_char;
        result.code[1] = b'n' as c_char;
        result.valid = 1;
        return result;
    }

    // Default: use first two letters, lowercased
    result.code[0] = b0.to_ascii_lowercase() as c_char;
    result.code[1] = b1.to_ascii_lowercase() as c_char;
    result.valid = 1;
    result
}

// =============================================================================
// Shell Default
// =============================================================================

/// Check if a shell path needs quoting (contains spaces).
///
/// # Returns
/// 1 if the shell path contains spaces and needs quoting, 0 otherwise.
#[no_mangle]
pub unsafe extern "C" fn rs_shell_needs_quoting(shell: *const c_char) -> c_int {
    if shell.is_null() || *shell == 0 {
        return 0;
    }

    let mut p = shell;
    while *p != 0 {
        if *p as u8 == b' ' {
            return 1;
        }
        p = p.add(1);
    }
    0
}

/// Compute the length needed for a quoted shell path.
///
/// If the shell path contains spaces, it will be quoted with double quotes.
/// Returns the length including quotes and null terminator.
#[no_mangle]
pub unsafe extern "C" fn rs_quoted_shell_len(shell: *const c_char) -> usize {
    if shell.is_null() || *shell == 0 {
        return 1; // Just the null terminator
    }

    let mut len: usize = 0;
    let mut p = shell;
    while *p != 0 {
        len += 1;
        p = p.add(1);
    }

    if rs_shell_needs_quoting(shell) != 0 {
        len + 3 // Two quotes + null terminator
    } else {
        len + 1 // Just null terminator
    }
}

// =============================================================================
// CDPATH Processing
// =============================================================================

/// Compute the length needed for a converted CDPATH value.
///
/// CDPATH uses ':' as separator on Unix (';' on Windows), which gets
/// converted to ',' for Vim's internal format. Spaces and commas in
/// paths get escaped with backslash.
///
/// Returns the length needed including null terminator.
#[no_mangle]
pub unsafe extern "C" fn rs_cdpath_converted_len(cdpath: *const c_char) -> usize {
    if cdpath.is_null() || *cdpath == 0 {
        return 2; // Just ",\0" (current dir)
    }

    let mut len: usize = 2; // Start with "," for current dir, plus null
    let mut p = cdpath;

    while *p != 0 {
        let c = *p as u8;
        // Path list separator becomes comma
        if c == b':' || c == b';' {
            len += 1;
        } else if c == b' ' || c == b',' {
            // Spaces and commas need escaping
            len += 2;
        } else {
            len += 1;
        }
        p = p.add(1);
    }

    len
}

/// Convert a CDPATH value to Vim's internal format.
///
/// - Adds leading comma (current directory first)
/// - Converts path separators (':' or ';') to ','
/// - Escapes spaces and commas with backslash
///
/// # Arguments
/// * `cdpath` - The CDPATH environment variable value
/// * `buf` - Buffer to write the converted value
/// * `buflen` - Length of the buffer
///
/// # Returns
/// The length of the converted string (not including null terminator).
#[no_mangle]
pub unsafe extern "C" fn rs_convert_cdpath(
    cdpath: *const c_char,
    buf: *mut c_char,
    buflen: usize,
) -> usize {
    if buf.is_null() || buflen == 0 {
        return 0;
    }

    // Start with comma for current directory
    *buf = b',' as c_char;
    let mut j: usize = 1;

    if cdpath.is_null() || *cdpath == 0 {
        if buflen > 1 {
            *buf.add(1) = 0;
        }
        return 1;
    }

    let mut p = cdpath;
    while *p != 0 && j < buflen - 1 {
        let c = *p as u8;

        if c == b':' || c == b';' {
            // Path separator becomes comma
            *buf.add(j) = b',' as c_char;
            j += 1;
        } else if c == b' ' || c == b',' {
            // Escape spaces and commas
            if j < buflen - 2 {
                *buf.add(j) = b'\\' as c_char;
                j += 1;
                *buf.add(j) = c as c_char;
                j += 1;
            }
        } else {
            *buf.add(j) = c as c_char;
            j += 1;
        }

        p = p.add(1);
    }

    if j < buflen {
        *buf.add(j) = 0;
    }

    j
}

// =============================================================================
// Backupskip Default Helpers
// =============================================================================

/// Check if a path should be added to backupskip.
///
/// Returns 1 if the path is valid and non-empty, 0 otherwise.
#[no_mangle]
pub unsafe extern "C" fn rs_valid_backupskip_path(path: *const c_char) -> c_int {
    if path.is_null() || *path == 0 {
        return 0;
    }
    1
}

/// Check if a path has a trailing path separator.
///
/// # Returns
/// 1 if path ends with '/' (Unix) or '\\' (Windows), 0 otherwise.
#[no_mangle]
pub unsafe extern "C" fn rs_has_trailing_pathsep(path: *const c_char) -> c_int {
    if path.is_null() || *path == 0 {
        return 0;
    }

    // Find end of string
    let mut p = path;
    while *p.add(1) != 0 {
        p = p.add(1);
    }

    let last = *p as u8;
    c_int::from(last == b'/' || last == b'\\')
}

/// Compute the pattern for a backupskip entry.
///
/// Creates a pattern like "/tmp/*" from a path like "/tmp".
/// Handles trailing separators correctly.
///
/// # Arguments
/// * `path` - The directory path
/// * `buf` - Buffer to write the pattern
/// * `buflen` - Length of the buffer
///
/// # Returns
/// The length of the pattern (not including null terminator).
#[no_mangle]
pub unsafe extern "C" fn rs_make_backupskip_pattern(
    path: *const c_char,
    buf: *mut c_char,
    buflen: usize,
) -> usize {
    if buf.is_null() || buflen == 0 || path.is_null() {
        return 0;
    }

    // Copy path
    let mut j: usize = 0;
    let mut p = path;
    while *p != 0 && j < buflen - 3 {
        *buf.add(j) = *p;
        j += 1;
        p = p.add(1);
    }

    // Add path separator if not present
    if j > 0 && rs_has_trailing_pathsep(path) == 0 && j < buflen - 2 {
        *buf.add(j) = b'/' as c_char;
        j += 1;
    }

    // Add wildcard
    if j < buflen - 1 {
        *buf.add(j) = b'*' as c_char;
        j += 1;
    }

    // Null terminate
    if j < buflen {
        *buf.add(j) = 0;
    }

    j
}

// =============================================================================
// Default Detection Utilities
// =============================================================================

/// Detect if we're in a Unix-like environment.
///
/// This is a compile-time constant exposed to Rust code.
#[no_mangle]
pub extern "C" fn rs_is_unix() -> c_int {
    #[cfg(unix)]
    {
        1
    }
    #[cfg(not(unix))]
    {
        0
    }
}

/// Get the default temporary directory for the current platform.
///
/// Returns a pointer to a static string:
/// - macOS: "/private/tmp"
/// - Other Unix: "/tmp"
/// - Windows: NULL (use environment variable)
#[no_mangle]
pub extern "C" fn rs_default_tmpdir() -> *const c_char {
    #[cfg(target_os = "macos")]
    {
        c"/private/tmp".as_ptr()
    }
    #[cfg(all(unix, not(target_os = "macos")))]
    {
        c"/tmp".as_ptr()
    }
    #[cfg(not(unix))]
    {
        std::ptr::null()
    }
}

// =============================================================================
// Helplang Default
// =============================================================================

extern "C" {
    #[link_name = "rs_get_option_flags"]
    fn nvim_hlg_get_option_flags(opt_idx: c_int) -> u32;
    static mut p_hlg: *mut c_char;
    #[link_name = "free_string_option"]
    fn nvim_free_string_option_hlg(p: *mut c_char);
    fn xstrdup(s: *const c_char) -> *mut c_char;
}

/// Set 'helplang' to a default value derived from a locale string, if it
/// has not already been set by the user.
///
/// Converts locale strings to two-letter codes (same logic as
/// `rs_compute_helplang`), then stores them via `nvim_set_p_hlg_from_code`.
#[export_name = "set_helplang_default"]
pub unsafe extern "C" fn rs_set_helplang_default(lang: *const c_char) {
    const K_OPT_FLAG_WAS_SET: u32 = 1 << 3;
    if lang.is_null() || *lang == 0 {
        return;
    }
    if (nvim_hlg_get_option_flags(crate::opt_index::K_OPT_HELPLANG) & K_OPT_FLAG_WAS_SET) != 0 {
        return;
    }

    let result = rs_compute_helplang(lang);
    if result.valid == 0 {
        return;
    }

    // Set p_hlg from 2-char code, freeing old value (matches C: p_hlg = code[0] ? xstrdup(code) : xstrdup(""))
    nvim_free_string_option_hlg(p_hlg);
    p_hlg = if result.code[0] != 0 {
        xstrdup(result.code.as_ptr())
    } else {
        xstrdup(c"".as_ptr())
    };
}

// =============================================================================
// Phase 1: Init helper function implementations (option_shim.c migration)
// =============================================================================

use crate::opt_index::{K_OPT_BACKUPSKIP, K_OPT_CDPATH, K_OPT_ICON, K_OPT_SHELL, K_OPT_TITLE};
use crate::storage::{OptVal, OptValData, String_};
use crate::OptValType;

extern "C" {
    // Phase 1 accessors (added to option_shim.c)
    fn enc_locale() -> *mut c_char;
    static mut fenc_default: *mut c_char;
    static mut p_title: c_int;
    static mut p_icon: c_int;
    fn os_getenv(name: *const c_char) -> *mut c_char;
    fn vim_getenv(name: *const c_char) -> *mut c_char;
    fn after_pathsep(b: *const c_char, p: *const c_char) -> c_int;
    #[link_name = "rs_get_option_flags"]
    fn nvim_get_option_flags(opt_idx: c_int) -> u32;
    fn xmemdupz(src: *const c_char, len: usize) -> *mut c_char;
    fn rs_find_dup_item(
        origval: *const c_char,
        newval: *const c_char,
        newvallen: usize,
        flags: u32,
    ) -> *const c_char;
}

/// C flag: option was set
const K_OPT_FLAG_WAS_SET: u32 = 1 << 3;

/// Equivalent of CSTR_AS_OPTVAL(ptr): creates a String OptVal wrapping a C pointer.
/// The pointer must be a valid C string allocated (or borrowed) for the lifetime of the OptVal.
unsafe fn cstr_as_optval(ptr: *mut c_char) -> OptVal {
    let len = libc::strlen(ptr.cast_const());
    OptVal {
        type_: OptValType::String,
        data: OptValData {
            string: String_ {
                data: ptr,
                size: len,
            },
        },
    }
}

/// Equivalent of BOOLEAN_OPTVAL(b)
fn boolean_optval(b: bool) -> OptVal {
    OptVal {
        type_: OptValType::Boolean,
        data: OptValData {
            boolean: c_int::from(b),
        },
    }
}

/// Initialize the 'shell' option to a default value (Rust implementation).
///
/// Replaces C `set_init_default_shell()`.
#[no_mangle]
pub unsafe extern "C" fn rs_set_init_default_shell() {
    let shell_env = c"SHELL";
    let shell = os_getenv(shell_env.as_ptr());
    if shell.is_null() {
        return;
    }

    if rs_shell_needs_quoting(shell) != 0 {
        // Shell path contains spaces; wrap in double quotes.
        let len = rs_quoted_shell_len(shell);
        let cmd = xmalloc(len);
        // Write `"<shell>"` into cmd
        *cmd = b'"' as c_char;
        let mut j: usize = 1;
        let mut p = shell;
        while *p != 0 {
            *cmd.add(j) = *p;
            j += 1;
            p = p.add(1);
        }
        *cmd.add(j) = b'"' as c_char;
        j += 1;
        *cmd.add(j) = 0;
        // Ownership of cmd transferred to set_string_default (allocated=true)
        crate::defaults::rs_set_string_default_opt(K_OPT_SHELL, cmd, 1);
    } else {
        // No quoting needed; shell is not owned after call (allocated=false)
        crate::defaults::rs_set_string_default_opt(K_OPT_SHELL, shell, 0);
    }
    xfree(shell);
}

/// Initialize the 'backupskip' option default from TMPDIR/TEMP/TMP env vars.
///
/// Replaces C `set_init_default_backupskip()`.
#[no_mangle]
pub unsafe extern "C" fn rs_set_init_default_backupskip() {
    // On Unix we also include the platform default temp directory.
    #[cfg(unix)]
    let tmpdir_literal: Option<*const c_char> = Some(rs_default_tmpdir());
    #[cfg(not(unix))]
    let tmpdir_literal: Option<*const c_char> = None;

    let env_names: &[&[u8]] = &[b"TMPDIR\0", b"TEMP\0", b"TMP\0"];

    let opt_idx = K_OPT_BACKUPSKIP;
    let flags = nvim_get_option_flags(opt_idx);

    // We build the result as a Vec<u8> (comma-separated patterns).
    let mut result: Vec<u8> = Vec::with_capacity(256);

    // Helper closure: try to add a path to result.
    let mut try_add_path = |path: *const c_char, mustfree: bool| {
        if path.is_null() || *path == 0 {
            if mustfree {
                xfree(path.cast_mut());
            }
            return;
        }

        let plen = libc::strlen(path);
        let has_trailing = after_pathsep(path, path.add(plen)) != 0;

        // Build pattern: path[/]* where / is path separator on current platform
        let sep: &[u8] = if has_trailing { b"" } else { b"/" };
        let mut item: Vec<u8> = Vec::with_capacity(plen + 2 + 1);
        let slice = std::slice::from_raw_parts(path.cast::<u8>(), plen);
        item.extend_from_slice(slice);
        item.extend_from_slice(sep);
        item.push(b'*');

        // Check for duplicates using rs_find_dup_item
        let existing_ptr: *const c_char = if result.is_empty() {
            std::ptr::null()
        } else {
            result.as_ptr().cast::<c_char>()
        };
        let dup = rs_find_dup_item(
            existing_ptr,
            item.as_ptr().cast::<c_char>(),
            item.len(),
            flags,
        );

        if dup.is_null() {
            // Not a duplicate: append separator if needed, then item
            if !result.is_empty() {
                result.push(b',');
            }
            result.extend_from_slice(&item);
        }

        if mustfree {
            xfree(path.cast_mut());
        }
    };

    // On Unix, add platform default tmp dir first
    #[cfg(unix)]
    {
        if let Some(default_tmp) = tmpdir_literal {
            if !default_tmp.is_null() {
                try_add_path(default_tmp, false);
            }
        }
    }

    // Add TMPDIR, TEMP, TMP from environment
    for name in env_names {
        let env_name = name.as_ptr().cast::<c_char>();
        let p = vim_getenv(env_name);
        try_add_path(p, true);
    }

    if !result.is_empty() {
        // Null-terminate and hand off to set_string_default (allocated=true)
        result.push(0);
        let len = result.len();
        let buf = xmalloc(len);
        std::ptr::copy_nonoverlapping(result.as_ptr().cast::<c_char>(), buf, len);
        crate::defaults::rs_set_string_default_opt(opt_idx, buf, 1);
    }
}

/// Initialize the 'cdpath' option default from CDPATH env var.
///
/// Replaces C `set_init_default_cdpath()`.
#[no_mangle]
pub unsafe extern "C" fn rs_set_init_default_cdpath() {
    let cdpath_key = c"CDPATH";
    let cdpath = vim_getenv(cdpath_key.as_ptr());
    if cdpath.is_null() {
        return;
    }

    // Compute needed buffer length and allocate
    let needed = rs_cdpath_converted_len(cdpath);
    let buf = xmalloc(needed);

    rs_convert_cdpath(cdpath, buf, needed);
    xfree(cdpath);

    // change_option_default takes ownership via CSTR_AS_OPTVAL semantics
    let val = cstr_as_optval(buf);
    crate::defaults::rs_change_option_default(K_OPT_CDPATH, val);
}

/// Initialize the encoding used for "default" in 'fileencodings'.
///
/// Replaces C `set_init_fenc_default()`.
#[no_mangle]
pub unsafe extern "C" fn rs_set_init_fenc_default() {
    let mut p = enc_locale();
    if p.is_null() {
        // Use utf-8 as "default" if locale encoding can't be detected.
        let utf8 = c"utf-8";
        let len = 5; // "utf-8\0" = 6, but xmemdupz adds NUL
        p = xmemdupz(utf8.as_ptr(), len);
    }
    fenc_default = p;
}

/// Set default values for 'title' and 'icon'.
///
/// Replaces C `set_title_defaults()`.
#[export_name = "set_title_defaults"]
pub unsafe extern "C" fn rs_set_title_defaults() {
    let title_flags = nvim_get_option_flags(K_OPT_TITLE);
    if (title_flags & K_OPT_FLAG_WAS_SET) == 0 {
        crate::defaults::rs_change_option_default(K_OPT_TITLE, boolean_optval(false));
        p_title = 0;
    }
    let icon_flags = nvim_get_option_flags(K_OPT_ICON);
    if (icon_flags & K_OPT_FLAG_WAS_SET) == 0 {
        crate::defaults::rs_change_option_default(K_OPT_ICON, boolean_optval(false));
        p_icon = 0;
    }
}

// =============================================================================
// set_init_2 and set_init_3 (option pass 7 phase 2)
// =============================================================================

use crate::opt_index::{
    K_OPT_FILEFORMATS, K_OPT_SCROLL, K_OPT_SHELLPIPE, K_OPT_SHELLREDIR, K_OPT_WINDOW,
};
use crate::OptInt;

extern "C" {
    // Direct C globals
    static mut Rows: c_int;
    static mut p_sh: *mut c_char;
    static mut p_window: OptInt;

    // set_init_2 accessors
    fn nvim_option_ilog_rtp();
    fn comp_col();
    fn option_was_set(opt_idx: c_int) -> bool;

    // set_init_3 accessors
    fn parse_shape_opt(what: c_int) -> *const c_char;
    fn invocation_path_tail(p: *const c_char, lenp: *mut usize) -> *const c_char;
    fn path_fnamecmp(a: *const c_char, b: *const c_char) -> c_int;
    fn set_option_direct(opt_idx: c_int, val: OptVal, opt_flags: c_int, set_sid: c_int);
    #[link_name = "rs_buf_is_empty"]
    fn nvim_curbuf_is_empty_via_buf(buf: *mut core::ffi::c_void) -> bool;
    fn rs_default_fileformat() -> c_int;
    #[link_name = "set_fileformat"]
    fn rs_set_fileformat(eol_style: c_int, opt_flags: c_int);
    fn rs_optval_copy(o: OptVal) -> OptVal;
}

/// OPT_LOCAL flag value (from option.h)
const OPT_LOCAL: c_int = 0x02;

/// kOptFlagWasSet: bit 3 of option flags
const K_OPT_FLAG_WAS_SET_2: u32 = 1 << 3;

/// Create a string OptVal from a static byte literal.
/// The string is NOT heap-allocated; use rs_optval_copy to allocate.
///
/// # Safety
/// `bytes` must be a valid NUL-terminated byte string with `len` chars before NUL.
unsafe fn static_cstr_as_optval(bytes: *const u8, len: usize) -> OptVal {
    OptVal {
        type_: OptValType::String,
        data: OptValData {
            string: String_ {
                data: bytes as *mut c_char,
                size: len,
            },
        },
    }
}

/// Create a number OptVal.
fn number_optval(n: OptInt) -> OptVal {
    OptVal {
        type_: OptValType::Number,
        data: OptValData { number: n },
    }
}

/// Initialize the options, part two: After getting Rows and Columns.
///
/// Corresponds to C's `set_init_2`.
#[export_name = "set_init_2"]
pub unsafe extern "C" fn rs_set_init_2(_headless: c_int) {
    // set in set_init_1 but logging is not allowed there
    nvim_option_ilog_rtp();

    // 'scroll' defaults to half the window height. The stored default is zero,
    // which results in the actual value computed from the window height.
    let scroll_flags = nvim_get_option_flags(K_OPT_SCROLL);
    if (scroll_flags & K_OPT_FLAG_WAS_SET_2) == 0 {
        crate::defaults::rs_set_option_default(K_OPT_SCROLL, OPT_LOCAL);
    }
    comp_col();

    // 'window' is only for backwards compatibility with Vi.
    // Default is Rows - 1.
    if !option_was_set(K_OPT_WINDOW) {
        p_window = OptInt::from(Rows) - 1;
    }
    crate::defaults::rs_change_option_default(K_OPT_WINDOW, number_optval(OptInt::from(Rows) - 1));
}

/// Compare a shell basename (from `p`) to a known shell name.
///
/// Helper used by `rs_set_init_3`.
unsafe fn shell_is(p: *const c_char, name: *const c_char) -> bool {
    path_fnamecmp(p, name) == 0
}

/// Initialize the options, part three: After reading the .vimrc.
///
/// Corresponds to C's `set_init_3`.
#[export_name = "set_init_3"]
pub unsafe extern "C" fn rs_set_init_3() {
    parse_shape_opt(2); // SHAPE_CURSOR = 2

    // Set 'shellpipe' and 'shellredir', depending on the 'shell' option.
    let do_srr = (nvim_get_option_flags(K_OPT_SHELLREDIR) & K_OPT_FLAG_WAS_SET_2) == 0;
    let do_sp = (nvim_get_option_flags(K_OPT_SHELLPIPE) & K_OPT_FLAG_WAS_SET_2) == 0;

    let mut len: usize = 0;
    let tail_ptr = invocation_path_tail(p_sh.cast_const(), &raw mut len);
    // Duplicate just the basename so we can compare safely.
    let p = xmemdupz(tail_ptr, len);

    let is_csh = shell_is(p, c"csh".as_ptr()) || shell_is(p, c"tcsh".as_ptr());
    let is_known_shell = shell_is(p, c"sh".as_ptr())
        || shell_is(p, c"ksh".as_ptr())
        || shell_is(p, c"mksh".as_ptr())
        || shell_is(p, c"pdksh".as_ptr())
        || shell_is(p, c"zsh".as_ptr())
        || shell_is(p, c"zsh-beta".as_ptr())
        || shell_is(p, c"bash".as_ptr())
        || shell_is(p, c"fish".as_ptr())
        || shell_is(p, c"ash".as_ptr())
        || shell_is(p, c"dash".as_ptr());

    if is_csh || is_known_shell {
        if do_sp {
            let sp_str: &[u8] = if is_csh { b"|& tee" } else { b"2>&1| tee" };
            let sp = static_cstr_as_optval(sp_str.as_ptr(), sp_str.len());
            set_option_direct(K_OPT_SHELLPIPE, sp, 0, -6 /* SID_NONE */);
            crate::defaults::rs_change_option_default(K_OPT_SHELLPIPE, rs_optval_copy(sp));
        }
        if do_srr {
            let srr_str: &[u8] = if is_csh { b">&" } else { b">%s 2>&1" };
            let srr = static_cstr_as_optval(srr_str.as_ptr(), srr_str.len());
            set_option_direct(K_OPT_SHELLREDIR, srr, 0, -6 /* SID_NONE */);
            crate::defaults::rs_change_option_default(K_OPT_SHELLREDIR, rs_optval_copy(srr));
        }
    }
    xfree(p);

    if nvim_curbuf_is_empty_via_buf(curbuf) {
        // Apply the first entry of 'fileformats' to the initial buffer.
        if (nvim_get_option_flags(K_OPT_FILEFORMATS) & K_OPT_FLAG_WAS_SET_2) != 0 {
            rs_set_fileformat(rs_default_fileformat(), OPT_LOCAL);
        }
    }

    rs_set_title_defaults();
}

// =============================================================================
// Phase 11 (pass 11): set_init_1, set_init_expand_env migration
// =============================================================================

extern "C" {
    fn langmap_init();
    fn stdpaths_user_state_subpath(
        fname: *const c_char,
        trailing_pathseps: usize,
        escape_commas: bool,
    ) -> *mut c_char;
    fn runtimepath_default(clean_arg: bool) -> *mut c_char;
    fn nvim_buf_set_b_p_initialized(buf: *mut core::ffi::c_void, val: c_int);
    fn nvim_buf_set_b_p_ac_minus1(buf: *mut core::ffi::c_void);
    fn nvim_buf_set_b_p_ar_minus1(buf: *mut core::ffi::c_void);
    fn nvim_buf_set_b_p_ul_no_local(buf: *mut core::ffi::c_void);
    static mut curwin: *mut core::ffi::c_void;
    #[link_name = "check_options"]
    fn rs_check_options();
    #[link_name = "rs_last_status"]
    fn nvim_last_status_0(morewin: c_int);
    fn init_spell_chartab();
    fn save_file_ff(buf: *mut core::ffi::c_void);
    fn os_env_exists(name: *const c_char, nonempty: bool) -> bool;
    fn lang_init();
    fn nvim_call_bind_textdomain_codeset();
    fn check_buf_options(buf: *mut core::ffi::c_void);
    static mut curbuf: *mut core::ffi::c_void;
    fn get_mess_lang() -> *mut c_char;

}

/// Rust implementation of `set_init_1`.
///
/// Initialize the options, first part. Called only once from main(),
/// just after creating the first buffer. If `clean_arg` is 1, Nvim was
/// started with --clean.
///
/// NOTE: ELOG() etc calls are not allowed here, as log location depends on
/// env var expansion which depends on expression evaluation and other
/// editor state initialized here. Do logging in set_init_2 or later.
///
/// # Safety
/// Must only be called once during startup, from C main().
#[export_name = "set_init_1"]
pub unsafe extern "C" fn rs_set_init_1(clean_arg: c_int) {
    langmap_init();

    // Allocate the default option values.
    crate::defaults::rs_alloc_options_default();

    // Set defaults for shell, backupskip, cdpath (already Rust functions).
    rs_set_init_default_shell();
    rs_set_init_default_backupskip();
    rs_set_init_default_cdpath();

    // backupdir: prepend ".," to the state subpath
    let backupdir_raw = stdpaths_user_state_subpath(c"backup".as_ptr(), 2, true);
    let backupdir = prepend_dot_comma(backupdir_raw);
    crate::defaults::rs_set_string_default_opt(K_OPT_BACKUPDIR, backupdir, 1);

    let viewdir = stdpaths_user_state_subpath(c"view".as_ptr(), 2, true);
    crate::defaults::rs_set_string_default_opt(K_OPT_VIEWDIR, viewdir, 1);

    let directory = stdpaths_user_state_subpath(c"swap".as_ptr(), 2, true);
    crate::defaults::rs_set_string_default_opt(K_OPT_DIRECTORY, directory, 1);

    let undodir = stdpaths_user_state_subpath(c"undo".as_ptr(), 2, true);
    crate::defaults::rs_set_string_default_opt(K_OPT_UNDODIR, undodir, 1);

    // Set default for &runtimepath. All necessary expansions are performed in
    // runtimepath_default().
    let rtp = runtimepath_default(clean_arg != 0);
    if !rtp.is_null() {
        crate::defaults::rs_set_string_default_opt(K_OPT_RUNTIMEPATH, rtp, 1);
        // Make a copy of rtp for packpath (allocated=false means a copy is made)
        crate::defaults::rs_set_string_default_opt(K_OPT_PACKPATH, rtp, 0);
        // rtp ownership was taken by runtimepath default; packpath copied it
    }

    // Set all options (except terminal options) to their default value.
    crate::defaults::rs_set_options_default(0);

    nvim_buf_set_b_p_initialized(curbuf, 1);
    nvim_buf_set_b_p_ac_minus1(curbuf);
    nvim_buf_set_b_p_ar_minus1(curbuf);
    nvim_buf_set_b_p_ul_no_local(curbuf);
    check_buf_options(curbuf);
    // check_win_options(curwin): call rs_check_winopt for w_onebuf_opt (offset 784)
    // and w_allbuf_opt (offset 2424), validated by window_struct_check.c _Static_asserts.
    crate::winopt::rs_check_winopt(curwin.byte_add(784));
    crate::winopt::rs_check_winopt(curwin.byte_add(2424));
    rs_check_options();

    // Set 'laststatus'
    nvim_last_status_0(0);

    // Must be before option_expand(), because that one needs vim_isIDc()
    crate::sideeffect::rs_didset_options();

    // Use the current chartab for the generic chartab.
    // This is not in didset_options() because it only depends on 'encoding'.
    init_spell_chartab();

    // Expand environment variables and things like "~" for the defaults.
    crate::defaults::rs_set_init_expand_env();

    save_file_ff(curbuf);

    // Detect use of mlterm.
    // Mlterm is a terminal emulator akin to xterm that has some special
    // abilities (bidi namely).
    if os_env_exists(c"MLTERM".as_ptr(), false) {
        crate::value::rs_set_option_value_give_err(
            crate::opt_index::K_OPT_TERMBIDI as c_int,
            boolean_optval(true),
            0,
        );
    }

    crate::sideeffect::rs_didset_options2();

    lang_init();
    rs_set_init_fenc_default();

    // GNU gettext: set codeset for translated messages
    nvim_call_bind_textdomain_codeset();

    // Set the default for 'helplang'.
    rs_set_helplang_default(get_mess_lang());
}

/// Prepend ".," to a heap-allocated C string.
///
/// The input pointer `s` must be a heap-allocated NUL-terminated string.
/// The returned pointer is a new heap allocation; `s` is freed.
///
/// # Safety
/// `s` must be a valid heap-allocated C string or NULL.
unsafe fn prepend_dot_comma(s: *mut c_char) -> *mut c_char {
    if s.is_null() {
        // Allocate just ".," + NUL
        let buf = xmalloc(3);
        *buf = b'.' as c_char;
        *buf.add(1) = b',' as c_char;
        *buf.add(2) = 0;
        return buf;
    }

    let orig_len = libc::strlen(s.cast_const());
    // Reallocate with 2 extra bytes at the front
    let new_size = orig_len + 3; // ".," + original + NUL
    let buf = xmalloc(new_size);

    // Write ".," then the original string
    *buf = b'.' as c_char;
    *buf.add(1) = b',' as c_char;
    core::ptr::copy_nonoverlapping(s.cast_const(), buf.add(2), orig_len + 1);

    // Free the old allocation
    xfree(s);

    buf
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::CString;
    use std::ptr;

    #[test]
    fn test_compute_helplang() {
        unsafe {
            // Test zh_CN -> cn
            let zh_cn = CString::new("zh_CN.UTF-8").unwrap();
            let result = rs_compute_helplang(zh_cn.as_ptr());
            assert_eq!(result.valid, 1);
            assert_eq!(result.code[0] as u8, b'c');
            assert_eq!(result.code[1] as u8, b'n');

            // Test zh_TW -> tw
            let zh_tw = CString::new("zh_TW").unwrap();
            let result = rs_compute_helplang(zh_tw.as_ptr());
            assert_eq!(result.valid, 1);
            assert_eq!(result.code[0] as u8, b't');
            assert_eq!(result.code[1] as u8, b'w');

            // Test C locale -> en
            let c_locale = CString::new("C").unwrap();
            let result = rs_compute_helplang(c_locale.as_ptr());
            assert_eq!(result.valid, 1);
            assert_eq!(result.code[0] as u8, b'e');
            assert_eq!(result.code[1] as u8, b'n');

            // Test C.UTF-8 -> en
            let c_utf8 = CString::new("C.UTF-8").unwrap();
            let result = rs_compute_helplang(c_utf8.as_ptr());
            assert_eq!(result.valid, 1);
            assert_eq!(result.code[0] as u8, b'e');
            assert_eq!(result.code[1] as u8, b'n');

            // Test POSIX -> en
            let posix = CString::new("POSIX").unwrap();
            let result = rs_compute_helplang(posix.as_ptr());
            assert_eq!(result.valid, 1);
            assert_eq!(result.code[0] as u8, b'e');
            assert_eq!(result.code[1] as u8, b'n');

            // Test en_US -> en
            let en_us = CString::new("en_US.UTF-8").unwrap();
            let result = rs_compute_helplang(en_us.as_ptr());
            assert_eq!(result.valid, 1);
            assert_eq!(result.code[0] as u8, b'e');
            assert_eq!(result.code[1] as u8, b'n');

            // Test de_DE -> de
            let de_de = CString::new("de_DE").unwrap();
            let result = rs_compute_helplang(de_de.as_ptr());
            assert_eq!(result.valid, 1);
            assert_eq!(result.code[0] as u8, b'd');
            assert_eq!(result.code[1] as u8, b'e');

            // Test invalid (too short)
            let short = CString::new("x").unwrap();
            let result = rs_compute_helplang(short.as_ptr());
            assert_eq!(result.valid, 0);

            // Test null
            let result = rs_compute_helplang(ptr::null());
            assert_eq!(result.valid, 0);
        }
    }

    #[test]
    fn test_shell_needs_quoting() {
        unsafe {
            let with_space = CString::new("/bin/my shell").unwrap();
            let without_space = CString::new("/bin/bash").unwrap();

            assert_eq!(rs_shell_needs_quoting(with_space.as_ptr()), 1);
            assert_eq!(rs_shell_needs_quoting(without_space.as_ptr()), 0);
            assert_eq!(rs_shell_needs_quoting(ptr::null()), 0);
        }
    }

    #[test]
    fn test_quoted_shell_len() {
        unsafe {
            let with_space = CString::new("/bin/my shell").unwrap();
            let without_space = CString::new("/bin/bash").unwrap();

            // "/bin/my shell" (13) + 2 quotes + null = 16
            assert_eq!(rs_quoted_shell_len(with_space.as_ptr()), 16);
            // "/bin/bash" (9) + null = 10
            assert_eq!(rs_quoted_shell_len(without_space.as_ptr()), 10);
        }
    }

    #[test]
    fn test_cdpath_converted_len() {
        unsafe {
            // "/foo:/bar" -> ",/foo,/bar" (10) + null = 11
            let cdpath = CString::new("/foo:/bar").unwrap();
            assert_eq!(rs_cdpath_converted_len(cdpath.as_ptr()), 11);

            // "/path with space" -> ",/path\\ with\\ space" (needs escaping)
            let with_space = CString::new("/path with space").unwrap();
            // 16 chars + 2 escapes + 1 leading comma + null = 20
            assert_eq!(rs_cdpath_converted_len(with_space.as_ptr()), 20);

            // Empty -> just ",\0"
            assert_eq!(rs_cdpath_converted_len(ptr::null()), 2);
        }
    }

    #[test]
    fn test_convert_cdpath() {
        unsafe {
            let cdpath = CString::new("/foo:/bar").unwrap();
            let mut buf = [0i8; 32];

            let len = rs_convert_cdpath(cdpath.as_ptr(), buf.as_mut_ptr(), 32);
            assert_eq!(len, 10); // ",/foo,/bar"

            // Check the result
            assert_eq!(buf[0] as u8, b',');
            assert_eq!(buf[1] as u8, b'/');
            assert_eq!(buf[2] as u8, b'f');
            assert_eq!(buf[5] as u8, b','); // Separator converted
        }
    }

    #[test]
    fn test_has_trailing_pathsep() {
        unsafe {
            let with_sep = CString::new("/tmp/").unwrap();
            let without_sep = CString::new("/tmp").unwrap();

            assert_eq!(rs_has_trailing_pathsep(with_sep.as_ptr()), 1);
            assert_eq!(rs_has_trailing_pathsep(without_sep.as_ptr()), 0);
            assert_eq!(rs_has_trailing_pathsep(ptr::null()), 0);
        }
    }

    #[test]
    fn test_make_backupskip_pattern() {
        unsafe {
            let path = CString::new("/tmp").unwrap();
            let mut buf = [0i8; 32];

            let len = rs_make_backupskip_pattern(path.as_ptr(), buf.as_mut_ptr(), 32);
            assert_eq!(len, 6); // "/tmp/*"

            // Check result
            assert_eq!(buf[0] as u8, b'/');
            assert_eq!(buf[1] as u8, b't');
            assert_eq!(buf[2] as u8, b'm');
            assert_eq!(buf[3] as u8, b'p');
            assert_eq!(buf[4] as u8, b'/');
            assert_eq!(buf[5] as u8, b'*');

            // With trailing separator
            let path_sep = CString::new("/tmp/").unwrap();
            let len = rs_make_backupskip_pattern(path_sep.as_ptr(), buf.as_mut_ptr(), 32);
            assert_eq!(len, 6); // "/tmp/*" (no double separator)
        }
    }

    #[test]
    fn test_is_unix() {
        #[cfg(unix)]
        assert_eq!(rs_is_unix(), 1);
        #[cfg(not(unix))]
        assert_eq!(rs_is_unix(), 0);
    }

    #[test]
    fn test_default_tmpdir() {
        let tmpdir = rs_default_tmpdir();
        #[cfg(unix)]
        assert!(!tmpdir.is_null());
        #[cfg(not(unix))]
        assert!(tmpdir.is_null());
    }
}
