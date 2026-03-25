//! Buffer name expansion for Neovim command-line completion.
//!
//! Implements `ExpandBufnames()` which finds all buffer names matching a
//! pattern (regex or fuzzy) for `:buf`, `:sbuf`, `:diffget`, `:diffput`
//! tab completion.
//!
//! Also provides `buflist_regex_match` (Rust port of C `fname_match` +
//! `buflist_match`), used by both `ExpandBufnames` and `buflist_findpat`.

#![allow(clippy::missing_safety_doc)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_possible_wrap)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::too_many_lines)]

use std::ffi::{c_char, c_int, c_void};

use crate::BufHandle;

// Return values matching C OK/FAIL
const OK: c_int = 0;
const FAIL: c_int = -1;

// Wild option constants (from cmdexpand.h)
const WILD_HOME_REPLACE: c_int = 0x02;
const WILD_BUFLASTUSED: c_int = 0x1000;
const BUF_DIFF_FILTER: c_int = 0x2000;

// FUZZY_SCORE_NONE = INT_MIN
const FUZZY_SCORE_NONE: c_int = c_int::MIN;

// =============================================================================
// External C Functions
// =============================================================================

extern "C" {
    fn nvim_get_curbuf() -> BufHandle;
    fn nvim_get_firstbuf() -> BufHandle;
    fn nvim_buf_get_next(buf: BufHandle) -> BufHandle;
    fn nvim_buf_get_b_p_bl(buf: BufHandle) -> c_int;
    fn nvim_buf_get_last_used(buf: BufHandle) -> i64;
    fn nvim_buf_get_b_sfname(buf: BufHandle) -> *const c_char;
    fn nvim_buf_get_b_ffname(buf: BufHandle) -> *const c_char;
    fn rs_diff_mode_buf(buf: BufHandle) -> bool;

    /// Check if pattern should use fuzzy matching.
    #[link_name = "cmdline_fuzzy_complete"]
    fn nvim_cmdline_fuzzy_complete(pat: *const c_char) -> bool;

    /// Compile a regex pattern for buffer name matching. Returns opaque handle or NULL.
    fn nvim_bufname_regex_compile(pat: *mut c_char) -> *mut c_void;
    /// Check if the compiled regex is still valid.
    fn nvim_bufname_regex_valid(handle: *mut c_void) -> c_int;
    /// Free a compiled regex handle.
    fn nvim_bufname_regex_free(handle: *mut c_void);

    /// Get `p_wic` (wildignorecase) option value.
    fn nvim_get_p_wic() -> c_int;

    /// Get `p_fic` (fileignorecase) option value.
    fn nvim_get_p_fic() -> bool;
    /// Check if a regmatch handle has a valid regprog.
    fn nvim_regmatch_has_regprog(handle: *mut c_void) -> bool;
    /// Set `rm_ic` on a regmatch handle.
    fn nvim_regmatch_set_rm_ic(handle: *mut c_void, val: c_int);
    /// Execute `vim_regexec` on a regmatch handle against name.
    fn nvim_regmatch_exec(handle: *mut c_void, name: *const c_char) -> bool;

    /// Get curwin->w_p_diff value.
    fn nvim_curwin_get_p_diff() -> c_int;

    /// Fuzzy match a string against a pattern. Returns score or `FUZZY_SCORE_NONE`.
    #[link_name = "fuzzy_match_str"]
    fn nvim_fuzzy_match_str(str_: *mut c_char, pat: *const c_char) -> c_int;

    /// `home_replace_save()` for a buffer -- caller must free with `nvim_xfree`.
    #[link_name = "home_replace_save"]
    fn nvim_home_replace_save_buf(buf: BufHandle, src: *const c_char) -> *mut c_char;
    fn nvim_xstrdup(s: *const c_char) -> *mut c_char;
    fn nvim_xfree(p: *mut c_void);
    fn nvim_xmalloc(size: usize) -> *mut c_void;

    /// Convert fuzzy matches to a string array. Frees `fuzmatch`.
    #[link_name = "fuzzymatches_to_strmatches"]
    fn nvim_fuzzymatches_to_strmatches(
        fuzmatch: *mut c_void,
        file: *mut *mut *mut c_char,
        count: c_int,
        escape: bool,
    );
}

// =============================================================================
// FuzmatchStr -- matches C fuzmatch_str_T layout
// =============================================================================

/// Mirrors C `fuzmatch_str_T` layout. Must match exactly for FFI.
#[repr(C)]
struct FuzmatchStr {
    idx: c_int,
    str_ptr: *mut c_char,
    score: c_int,
}

// =============================================================================
// Buffer name regex match (Rust port of fname_match / buflist_match)
// =============================================================================

/// Match `name` against the compiled regex `handle`, with home-replace fallback.
///
/// Mirrors C `fname_match()`. Returns `name` on match, NULL on no match.
/// Sets `rm_ic` from `p_fic` | `ignore_case`.
///
/// # Safety
/// `handle` must be a valid `regmatch_T`* from `nvim_bufname_regex_compile` or
/// `nvim_blfp_regex_compile`. `name` must be a valid NUL-terminated C string or NULL.
unsafe fn fname_match_rs(
    handle: *mut c_void,
    name: *const c_char,
    ignore_case: bool,
) -> *const c_char {
    if name.is_null() || !nvim_regmatch_has_regprog(handle) {
        return std::ptr::null();
    }
    // Set ignore-case: p_fic OR caller-requested
    nvim_regmatch_set_rm_ic(handle, c_int::from(nvim_get_p_fic() || ignore_case));
    if nvim_regmatch_exec(handle, name) {
        return name;
    }
    // If regprog became NULL (engine switch), stop here
    if !nvim_regmatch_has_regprog(handle) {
        return std::ptr::null();
    }
    // Replace $HOME with '~' and try again
    let p = nvim_home_replace_save_buf(BufHandle::from_ptr(std::ptr::null_mut()), name);
    let matched = if nvim_regmatch_exec(handle, p) {
        name
    } else {
        std::ptr::null()
    };
    nvim_xfree(p.cast::<c_void>());
    matched
}

/// Match buffer `buf` against the compiled regex `handle`.
///
/// Mirrors C `buflist_match()`: tries `b_sfname` first, then `b_ffname`.
/// Returns matched name or NULL.
///
/// Used by both `ExpandBufnames` (via `nvim_bufname_regex_compile`) and
/// `buflist_findpat` (via `nvim_blfp_regex_compile`); both produce a `regmatch_T`*.
///
/// # Safety
/// Same requirements as `fname_match_rs`.
pub(crate) unsafe fn buflist_regex_match(
    handle: *mut c_void,
    buf: BufHandle,
    ignore_case: bool,
) -> *const c_char {
    if handle.is_null() {
        return std::ptr::null();
    }
    let sfname = nvim_buf_get_b_sfname(buf);
    let m = fname_match_rs(handle, sfname, ignore_case);
    if !m.is_null() {
        return m;
    }
    // regprog may have gone NULL if engine switched
    if !nvim_regmatch_has_regprog(handle) {
        return std::ptr::null();
    }
    let ffname = nvim_buf_get_b_ffname(buf);
    fname_match_rs(handle, ffname, ignore_case)
}

// =============================================================================
// ExpandBufnames implementation
// =============================================================================

/// Find all buffer names matching `pat` for command-line expansion.
///
/// Returns `OK` if at least one match was found, `FAIL` otherwise.
/// On success, sets `*num_file` and `*file` (caller must free).
///
/// # Safety
///
/// Must be called on the main thread with valid Neovim state.
pub unsafe fn expand_bufnames_impl(
    pat: *mut c_char,
    num_file: *mut c_int,
    file: *mut *mut *mut c_char,
    options: c_int,
) -> c_int {
    *num_file = 0;
    *file = std::ptr::null_mut();

    // BUF_DIFF_FILTER: only diff buffers, and curwin must be in diff mode
    if (options & BUF_DIFF_FILTER) != 0 && nvim_curwin_get_p_diff() == 0 {
        return FAIL;
    }

    let fuzzy = nvim_cmdline_fuzzy_complete(pat);
    let p_wic = nvim_get_p_wic() != 0;
    let curbuf = nvim_get_curbuf();

    // Determine pattern to compile:
    // '^' at start means "match from separator or start", so strip it and
    // let the regex match anywhere (the C code did this trick too).
    let mut patc: *mut c_char = pat;
    let mut patc_to_free: *mut c_char = std::ptr::null_mut();

    let mut regex_handle: *mut c_void = std::ptr::null_mut();

    if !fuzzy {
        if *pat == b'^' as c_char {
            if *pat.add(1) != 0 {
                // Strip leading '^': xstrdup(pat + 1)
                patc = nvim_xstrdup(pat.add(1));
                patc_to_free = patc;
            } else {
                // Pattern is just "^": match everything
                patc = c"".as_ptr().cast_mut();
            }
        }
        regex_handle = nvim_bufname_regex_compile(patc);
        if !patc_to_free.is_null() {
            nvim_xfree(patc_to_free.cast::<c_void>());
        }
        // regex_handle may be NULL if pattern is invalid
    }

    // Collect matches in a Vec -- single-pass (no need for C's two-round approach)
    let mut str_matches: Vec<*mut c_char> = Vec::new();
    let mut fuz_matches: Vec<FuzmatchStr> = Vec::new();
    // For WILD_BUFLASTUSED sorting: track (last_used, buf_handle) alongside str
    let mut str_with_buf: Vec<(i64, BufHandle, *mut c_char)> = Vec::new();
    let want_lastused = !fuzzy && (options & WILD_BUFLASTUSED) != 0;

    let mut buf = nvim_get_firstbuf();
    while !buf.is_null() {
        // Skip unlisted buffers
        if nvim_buf_get_b_p_bl(buf) == 0 {
            buf = nvim_buf_get_next(buf);
            continue;
        }

        // BUF_DIFF_FILTER: skip non-diff and curbuf
        if (options & BUF_DIFF_FILTER) != 0 && (buf == curbuf || !rs_diff_mode_buf(buf)) {
            buf = nvim_buf_get_next(buf);
            continue;
        }

        let matched_name: *const c_char = if fuzzy {
            // Try short name first, then full name
            let sfname = nvim_buf_get_b_sfname(buf);
            let score = if sfname.is_null() {
                FUZZY_SCORE_NONE
            } else {
                nvim_fuzzy_match_str(sfname.cast_mut(), pat)
            };
            if score != FUZZY_SCORE_NONE {
                let idx = fuz_matches.len() as c_int;
                let p = if (options & WILD_HOME_REPLACE) != 0 {
                    nvim_home_replace_save_buf(buf, sfname)
                } else {
                    nvim_xstrdup(sfname)
                };
                fuz_matches.push(FuzmatchStr {
                    idx,
                    str_ptr: p,
                    score,
                });
                buf = nvim_buf_get_next(buf);
                continue;
            }
            let ffname = nvim_buf_get_b_ffname(buf);
            let score = if ffname.is_null() {
                FUZZY_SCORE_NONE
            } else {
                nvim_fuzzy_match_str(ffname.cast_mut(), pat)
            };
            if score != FUZZY_SCORE_NONE {
                let idx = fuz_matches.len() as c_int;
                let p = if (options & WILD_HOME_REPLACE) != 0 {
                    nvim_home_replace_save_buf(buf, ffname)
                } else {
                    nvim_xstrdup(ffname)
                };
                fuz_matches.push(FuzmatchStr {
                    idx,
                    str_ptr: p,
                    score,
                });
            }
            buf = nvim_buf_get_next(buf);
            continue;
        } else {
            // Regex path: validity check to detect engine switch
            if nvim_bufname_regex_valid(regex_handle) == 0 {
                // Regex became invalid (engine switch): abort
                nvim_bufname_regex_free(regex_handle);
                return FAIL;
            }
            buflist_regex_match(regex_handle, buf, p_wic)
        };

        if matched_name.is_null() {
            buf = nvim_buf_get_next(buf);
            continue;
        }

        // Got a regex match
        let p = if (options & WILD_HOME_REPLACE) != 0 {
            nvim_home_replace_save_buf(buf, matched_name)
        } else {
            nvim_xstrdup(matched_name)
        };

        if want_lastused {
            str_with_buf.push((nvim_buf_get_last_used(buf), buf, p));
        } else {
            str_matches.push(p);
        }

        buf = nvim_buf_get_next(buf);
    }

    nvim_bufname_regex_free(regex_handle);

    // Build output
    if fuzzy {
        let count = fuz_matches.len() as c_int;
        if count == 0 {
            return FAIL;
        }
        // Allocate a C-compatible fuzmatch_str_T array for the C sorter
        let fuz_c = nvim_xmalloc(fuz_matches.len() * std::mem::size_of::<FuzmatchStr>())
            .cast::<FuzmatchStr>();
        for (i, fm) in fuz_matches.into_iter().enumerate() {
            fuz_c.add(i).write(fm);
        }
        nvim_fuzzymatches_to_strmatches(fuz_c.cast::<c_void>(), file, count, false);
        *num_file = count;
        return OK;
    }

    // Regex path: sort by last_used if requested, move curbuf to end
    if want_lastused {
        // Sort descending by b_last_used (most recently used first)
        str_with_buf.sort_by(|a, b| b.0.cmp(&a.0));
        // If curbuf is first, rotate it to the end
        if !str_with_buf.is_empty() && str_with_buf[0].1 == curbuf {
            let first = str_with_buf.remove(0);
            str_with_buf.push(first);
        }
        str_matches = str_with_buf.into_iter().map(|(_, _, p)| p).collect();
    }

    let count = str_matches.len();
    if count == 0 {
        return FAIL;
    }

    // Allocate C array and fill it
    let arr = nvim_xmalloc(count * std::mem::size_of::<*mut c_char>()).cast::<*mut c_char>();
    for (i, p) in str_matches.into_iter().enumerate() {
        arr.add(i).write(p);
    }
    *file = arr;
    *num_file = count as c_int;
    OK
}

// =============================================================================
// FFI Exports
// =============================================================================

/// Find all buffer names matching `pat` for command-line expansion.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_ExpandBufnames(
    pat: *mut c_char,
    num_file: *mut c_int,
    file: *mut *mut *mut c_char,
    options: c_int,
) -> c_int {
    expand_bufnames_impl(pat, num_file, file, options)
}

/// C export: `ExpandBufnames`.
#[unsafe(export_name = "ExpandBufnames")]
pub unsafe extern "C" fn expand_bufnames_export(
    pat: *mut c_char,
    num_file: *mut c_int,
    file: *mut *mut *mut c_char,
    options: c_int,
) -> c_int {
    expand_bufnames_impl(pat, num_file, file, options)
}
