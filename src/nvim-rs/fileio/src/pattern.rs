//! File pattern conversion and matching utilities.
//!
//! This module provides:
//! - `rs_file_pat_to_reg_pat`: Convert shell glob to Vim regex
//! - `rs_match_file_pat`: Match a filename against a pattern or pre-compiled regex
//! - `rs_match_file_list`: Match a filename against a comma-separated pattern list

#![allow(unsafe_code)]

use std::ffi::{c_char, c_int, c_void};

// RE_MAGIC flag for vim_regcomp
const RE_MAGIC: c_int = 1;
// colnr_T = int32
type ColnrT = i32;

extern "C" {
    static mut p_fic: c_int;
}

extern "C" {
    fn xmalloc(size: usize) -> *mut c_char;
    fn xstrdup(s: *const c_char) -> *mut c_char;
    fn xfree(ptr: *mut c_void);

    /// vim_ispathsep: returns nonzero if c is a path separator.
    fn vim_ispathsep(c: c_int) -> c_int;

    /// Compile a Vim regex pattern. Returns NULL on failure.
    fn vim_regcomp(pat: *const c_char, flags: c_int) -> *mut c_void;
    /// Free a compiled regex program.
    fn vim_regfree(prog: *mut c_void);
    /// Match `line` against `*prog` (no regmatch_T needed).
    fn vim_regexec_prog(
        prog: *mut *mut c_void,
        ignore_case: bool,
        line: *const c_char,
        col: ColnrT,
    ) -> bool;

    /// Copy one comma-separated option part into buf.
    #[link_name = "copy_option_part"]
    fn rs_copy_option_part(
        option: *mut *mut c_char,
        buf: *mut c_char,
        maxlen: usize,
        sep_chars: *const c_char,
    ) -> usize;

    /// path_tail: return pointer to the filename part of a path.
    fn path_tail(p: *const c_char) -> *mut c_char;
}

// Maximum path length
const MAXPATHL: usize = 4096;

// =============================================================================
// file_pat_to_reg_pat
// =============================================================================

/// Convert a shell glob pattern `pat[..pat_end]` to a Vim regular expression.
///
/// If `pat_end` is NULL, the whole NUL-terminated string is used.
/// If `allow_dirs` is non-NULL, it is set to true when the pattern contains
/// a path separator (so directory matching is needed).
/// `no_bslash` is only meaningful on Windows; on Linux it is ignored.
///
/// Returns an xmalloc-allocated string or NULL on failure. Caller must xfree().
///
/// # Safety
/// `pat` must be a valid non-null C string.
/// `pat_end` may be null (meaning use strlen).
/// `allow_dirs` may be null.
#[export_name = "file_pat_to_reg_pat"]
pub unsafe extern "C" fn rs_file_pat_to_reg_pat(
    pat: *const c_char,
    pat_end: *const c_char,
    allow_dirs: *mut c_char,
    _no_bslash: c_int,
) -> *mut c_char {
    if pat.is_null() {
        return std::ptr::null_mut();
    }

    if !allow_dirs.is_null() {
        unsafe { *allow_dirs = 0 };
    }

    let pat_end = if pat_end.is_null() {
        let len = unsafe { libc::strlen(pat as *const libc::c_char) };
        unsafe { pat.add(len) }
    } else {
        pat_end
    };

    // Empty pattern → "^$"
    if pat_end == pat {
        return unsafe { xstrdup(c"^$".as_ptr()) };
    }

    let pat_len = unsafe { pat_end.offset_from(pat) } as usize;
    let pat_slice = unsafe { std::slice::from_raw_parts(pat as *const u8, pat_len) };

    // First pass: calculate output buffer size.
    // Base: 2 bytes for '^' and '$', plus 1 for NUL.
    let mut size: usize = 2;
    for &b in pat_slice {
        match b {
            b'*' | b'.' | b',' | b'{' | b'}' | b'~' => size += 2, // extra backslash
            _ => size += 1,
        }
    }
    size += 1; // NUL terminator

    let reg_pat = unsafe { xmalloc(size) };
    if reg_pat.is_null() {
        return std::ptr::null_mut();
    }

    // Second pass: build the output.
    let out = unsafe { std::slice::from_raw_parts_mut(reg_pat as *mut u8, size) };
    let mut i: usize = 0;

    // Skip leading '*'s (unless it's the only char)
    let mut pat_start = 0usize;
    if pat_slice[0] == b'*' {
        while pat_start < pat_len - 1 && pat_slice[pat_start] == b'*' {
            pat_start += 1;
        }
        // If pat still starts with '*', don't emit '^'
        if pat_slice[pat_start] == b'*' {
            // all stars - don't emit '^'
        }
    } else {
        out[i] = b'^';
        i += 1;
    }

    // Strip trailing '*'s
    let mut pat_end_idx = pat_len;
    let mut add_dollar = true;
    if pat_end_idx > 0 && pat_slice[pat_end_idx - 1] == b'*' {
        while pat_end_idx > pat_start + 1 && pat_slice[pat_end_idx - 1] == b'*' {
            pat_end_idx -= 1;
        }
        add_dollar = false;
    }

    let mut nested: i32 = 0;
    let mut p = pat_start;

    while p < pat_end_idx && nested >= 0 {
        let c = pat_slice[p];
        match c {
            b'*' => {
                out[i] = b'.';
                i += 1;
                out[i] = b'*';
                i += 1;
                // "**" matches like "*"
                while p + 1 < pat_end_idx && pat_slice[p + 1] == b'*' {
                    p += 1;
                }
            }
            b'.' | b'~' => {
                out[i] = b'\\';
                i += 1;
                out[i] = c;
                i += 1;
            }
            b'?' => {
                out[i] = b'.';
                i += 1;
            }
            b'\\' => {
                // At end of pattern: skip
                if p + 1 >= pat_end_idx {
                    // nothing
                } else {
                    let next = pat_slice[p + 1];
                    p += 1;
                    // On Linux (no BACKSLASH_IN_FILENAME), no_bslash is irrelevant.
                    // Undo escaping from ExpandEscape():
                    // \? -> ?   (since BACKSLASH_IN_FILENAME_BOOL is false)
                    // \, \% \# \  \{ \} -> literal
                    // \\\{ -> \{
                    if next == b'?' {
                        out[i] = b'?';
                        i += 1;
                    } else if next == b','
                        || next == b'%'
                        || next == b'#'
                        || next == b' '
                        || next == b'\t'
                        || next == b'{'
                        || next == b'}'
                    {
                        out[i] = next;
                        i += 1;
                    } else if next == b'\\' && p + 1 < pat_end_idx && pat_slice[p + 1] == b'{' {
                        out[i] = b'\\';
                        i += 1;
                        out[i] = b'{';
                        i += 1;
                        p += 1;
                    } else {
                        // Check for path separator
                        if !allow_dirs.is_null() && unsafe { vim_ispathsep(next as c_int) } != 0 {
                            unsafe { *allow_dirs = 1 };
                        }
                        out[i] = b'\\';
                        i += 1;
                        out[i] = next;
                        i += 1;
                    }
                }
            }
            b'{' => {
                out[i] = b'\\';
                i += 1;
                out[i] = b'(';
                i += 1;
                nested += 1;
            }
            b'}' => {
                out[i] = b'\\';
                i += 1;
                out[i] = b')';
                i += 1;
                nested -= 1;
            }
            b',' => {
                if nested != 0 {
                    out[i] = b'\\';
                    i += 1;
                    out[i] = b'|';
                    i += 1;
                } else {
                    out[i] = b',';
                    i += 1;
                }
            }
            _ => {
                if !allow_dirs.is_null() && unsafe { vim_ispathsep(c as c_int) } != 0 {
                    unsafe { *allow_dirs = 1 };
                }
                out[i] = c;
                i += 1;
            }
        }
        p += 1;
    }

    if add_dollar {
        out[i] = b'$';
        i += 1;
    }
    out[i] = 0; // NUL terminate

    if nested != 0 {
        if nested < 0 {
            unsafe { emsg(c"E219: Missing {.".as_ptr()) };
        } else {
            unsafe { emsg(c"E220: Missing }.".as_ptr()) };
        }
        unsafe { xfree(reg_pat as *mut c_void) };
        return std::ptr::null_mut();
    }

    reg_pat
}

extern "C" {
    fn emsg(s: *const c_char);
}

// =============================================================================
// match_file_pat
// =============================================================================

/// Match a filename against a pattern or pre-compiled regex.
///
/// If `prog` is non-NULL, the compiled regex is stored/reused via `*prog`.
/// Otherwise, `pattern` is compiled each call and freed afterward.
///
/// Returns true if there is a match.
///
/// # Safety
/// Pointers must be valid or null per the C contract.
#[export_name = "match_file_pat"]
pub unsafe extern "C" fn rs_match_file_pat(
    pattern: *const c_char,
    prog: *mut *mut c_void,
    fname: *const c_char,
    sfname: *const c_char,
    tail: *const c_char,
    allow_dirs: c_int,
) -> bool {
    let use_icase = unsafe { p_fic } != 0;

    // Get (or compile) the regex program.
    let local_prog: *mut c_void;
    let use_prog_ptr: *mut *mut c_void;

    if !prog.is_null() {
        // Caller owns the prog (may be NULL on first call → compiled lazily by vim_regexec_prog)
        use_prog_ptr = prog;
        local_prog = std::ptr::null_mut(); // not used
    } else {
        // Compile locally
        if pattern.is_null() {
            return false;
        }
        let compiled = unsafe { vim_regcomp(pattern, RE_MAGIC) };
        if compiled.is_null() {
            return false;
        }
        // Put it in a local slot
        local_prog = compiled;
        use_prog_ptr = std::ptr::null_mut(); // sentinel: use local path
    };

    let result;

    if !prog.is_null() {
        // Using the caller's prog slot - vim_regexec_prog handles compilation if *prog is NULL
        // but we need pattern for that case. The C original sets regmatch.regprog = *prog
        // then calls vim_regexec. Here we use vim_regexec_prog which does the same:
        // if *prog_ptr is NULL it means it was already freed or never compiled.
        // But the C code compiles once at registration time (autocmd.c line 595).
        // At match time, *prog is non-NULL. We just call vim_regexec_prog.
        if unsafe { *prog }.is_null() {
            // prog slot is NULL (never compiled) - nothing to match
            result = false;
        } else {
            let mut matched = false;
            if allow_dirs != 0 {
                // Try full path
                if !fname.is_null()
                    && unsafe { vim_regexec_prog(use_prog_ptr, use_icase, fname, 0) }
                {
                    matched = true;
                }
                // Try short name
                if !matched
                    && !sfname.is_null()
                    && unsafe { vim_regexec_prog(use_prog_ptr, use_icase, sfname, 0) }
                {
                    matched = true;
                }
            } else if !tail.is_null()
                && unsafe { vim_regexec_prog(use_prog_ptr, use_icase, tail, 0) }
            {
                matched = true;
            }
            result = matched;
        }
    } else {
        // Local prog
        let mut lp = local_prog;
        let lp_ptr: *mut *mut c_void = &raw mut lp;
        let mut matched = false;
        if allow_dirs != 0 {
            if !fname.is_null() && unsafe { vim_regexec_prog(lp_ptr, use_icase, fname, 0) } {
                matched = true;
            }
            if !matched
                && !sfname.is_null()
                && unsafe { vim_regexec_prog(lp_ptr, use_icase, sfname, 0) }
            {
                matched = true;
            }
        } else if !tail.is_null() && unsafe { vim_regexec_prog(lp_ptr, use_icase, tail, 0) } {
            matched = true;
        }
        // Free the locally-compiled prog (lp may have been updated by vim_regexec_prog)
        unsafe { vim_regfree(lp) };
        result = matched;
    }

    result
}

// =============================================================================
// match_file_list
// =============================================================================

/// Check if a file matches any pattern in a comma-separated list.
///
/// `list` is like `'wildignore'`: comma-separated glob patterns.
/// `sfname` is the short filename, `ffname` the full path.
///
/// Returns true if any pattern matched.
///
/// # Safety
/// `list` and `ffname` must be valid non-null C strings. `sfname` may be null.
#[export_name = "match_file_list"]
pub unsafe extern "C" fn rs_match_file_list(
    list: *const c_char,
    sfname: *const c_char,
    ffname: *const c_char,
) -> bool {
    let tail = if sfname.is_null() {
        std::ptr::null()
    } else {
        unsafe { path_tail(sfname) as *const c_char }
    };

    let sep = c",".as_ptr();
    let mut p = list as *mut c_char;

    while unsafe { *p } != 0 {
        let mut buf = [0u8; MAXPATHL + 1];
        unsafe { rs_copy_option_part(&raw mut p, buf.as_mut_ptr() as *mut c_char, MAXPATHL, sep) };
        let mut allow_dirs: c_char = 0;
        let regpat = unsafe {
            rs_file_pat_to_reg_pat(
                buf.as_ptr() as *const c_char,
                std::ptr::null(),
                &raw mut allow_dirs,
                0,
            )
        };
        if regpat.is_null() {
            break;
        }
        let matched = unsafe {
            rs_match_file_pat(
                regpat,
                std::ptr::null_mut(),
                ffname,
                sfname,
                tail,
                allow_dirs as c_int,
            )
        };
        unsafe { xfree(regpat as *mut c_void) };
        if matched {
            return true;
        }
    }
    false
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_constants() {
        assert_eq!(super::RE_MAGIC, 1);
        assert_eq!(super::MAXPATHL, 4096);
    }
}
