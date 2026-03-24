//! User-defined and shell-command expansion, migrated from `cmdexpand.c`.
//!
//! Implements `ExpandUserDefined`, `ExpandUserList`, `ExpandUserLua`.

use libc::{c_char, c_int, c_void};

use crate::ExpandT;

// =============================================================================
// `fuzmatch_str_T` repr(C) mirror (same layout as in expand.rs)
// =============================================================================

/// Matches `fuzmatch_str_T` from `fuzzy.h`.
#[repr(C)]
struct FuzmatchStr {
    idx: c_int,
    _pad: c_int,
    str_: *mut c_char,
    score: c_int,
    _pad2: c_int,
}

// Re-use `RegMatch` from expand.rs to avoid duplicate type definitions.
pub use crate::expand::RegMatch;

// =============================================================================
// External C functions
// =============================================================================

type ListHandle = *mut c_void;

extern "C" {
    /// `call_user_expand_func(call_func_retlist, xp)` wrapper.
    fn nvim_cmdexpand_call_user_expand_retlist(xp: *mut ExpandT) -> ListHandle;

    /// `call_user_expand_func(call_func_retstr, xp)` wrapper.
    fn nvim_cmdexpand_call_user_expand_retstr(xp: *mut ExpandT) -> *mut c_char;

    /// `nlua_call_user_expand_func` wrapper — returns list or NULL.
    fn nvim_cmdexpand_nlua_call_user_expand_retlist(xp: *mut ExpandT) -> ListHandle;

    /// Convert a `list_T *` to `char **` array (takes ownership of list, unrefs it).
    fn nvim_cmdexpand_list_to_string_matches(
        list: ListHandle,
        matches: *mut *mut *mut c_char,
        num_matches: *mut c_int,
    );

    fn vim_regexec(rmp: *mut RegMatch, line: *const c_char, col: c_int) -> bool;
    fn fuzzy_match_str(str_: *mut c_char, pat: *const c_char) -> c_int;
    fn fuzzymatches_to_strmatches(
        fuzmatch: *mut FuzmatchStr,
        matches: *mut *mut *mut c_char,
        count: c_int,
        funcsort: bool,
    );
    fn xfree(ptr: *mut c_void);
    fn xmemdupz(s: *const c_char, len: usize) -> *mut c_char;
    fn vim_strchr(s: *const c_char, c: c_int) -> *const c_char;
}

// =============================================================================
// Constants
// =============================================================================

const OK: c_int = 1;
const FAIL: c_int = 0;
const FUZZY_SCORE_NONE: c_int = c_int::MIN;
const NUL: u8 = 0;

// =============================================================================
// ExpandUserDefined
// =============================================================================

/// Expand names with a user-defined function (`EXPAND_USER_DEFINED`).
///
/// Mirrors `ExpandUserDefined` from `cmdexpand.c`.
///
/// # Safety
///
/// All pointer arguments must be valid.
pub unsafe extern "C" fn rs_expand_user_defined(
    pat: *const c_char,
    xp: *mut ExpandT,
    regmatch: *mut RegMatch,
    matches: *mut *mut *mut c_char,
    num_matches: *mut c_int,
) -> c_int {
    let pat_str = std::ffi::CStr::from_ptr(pat).to_bytes();
    let pat_rust = std::str::from_utf8_unchecked(pat_str);
    let fuzzy = crate::cmdline_fuzzy_complete(pat_rust);

    *matches = std::ptr::null_mut();
    *num_matches = 0;

    let retstr = nvim_cmdexpand_call_user_expand_retstr(xp);
    if retstr.is_null() {
        return FAIL;
    }

    let mut str_matches: Vec<*mut c_char> = Vec::new();
    let mut fuz_matches: Vec<FuzmatchStr> = Vec::new();

    let mut s = retstr;
    loop {
        if *s == NUL as c_char {
            break;
        }
        // Find end of this item (newline or NUL)
        let e = vim_strchr(s, c_int::from(b'\n'));
        let e: *const c_char = if e.is_null() {
            s.add(libc::strlen(s))
        } else {
            e
        };

        let keep = *e;
        // Temporarily NUL-terminate this item
        let e_mut = e.cast_mut();
        *e_mut = 0;

        let mut score: c_int = 0;
        let xp_pattern = (*xp).xp_pattern;
        let is_match = if !xp_pattern.is_null() && *xp_pattern != 0 {
            if fuzzy {
                score = fuzzy_match_str(s, pat);
                score != FUZZY_SCORE_NONE
            } else {
                vim_regexec(regmatch, s, 0)
            }
        } else {
            true
        };

        // Restore the original character
        *e_mut = keep;

        if is_match {
            let item_len = (e as usize) - (s as usize);
            let owned = xmemdupz(s, item_len);
            if fuzzy {
                fuz_matches.push(FuzmatchStr {
                    idx: fuz_matches.len() as c_int,
                    _pad: 0,
                    str_: owned,
                    score,
                    _pad2: 0,
                });
            } else {
                str_matches.push(owned);
            }
        }

        if keep == 0 {
            break;
        }
        s = e.add(1).cast_mut();
    }
    xfree(retstr.cast());

    let count = if fuzzy {
        fuz_matches.len()
    } else {
        str_matches.len()
    };

    if count == 0 {
        return OK;
    }

    if fuzzy {
        fuzzymatches_to_strmatches(
            fuz_matches.as_mut_ptr(),
            matches,
            fuz_matches.len() as c_int,
            false,
        );
        *num_matches = fuz_matches.len() as c_int;
        std::mem::forget(fuz_matches);
    } else {
        let boxed = str_matches.into_boxed_slice();
        let len = boxed.len();
        let ptr = Box::into_raw(boxed).cast::<*mut c_char>();
        *matches = ptr;
        *num_matches = len as c_int;
    }

    OK
}

// =============================================================================
// ExpandUserList
// =============================================================================

/// Expand names with a list returned by a user-defined function.
///
/// Mirrors `ExpandUserList` from `cmdexpand.c`.
///
/// # Safety
///
/// All pointer arguments must be valid.
pub unsafe extern "C" fn rs_expand_user_list(
    xp: *mut ExpandT,
    matches: *mut *mut *mut c_char,
    num_matches: *mut c_int,
) -> c_int {
    *matches = std::ptr::null_mut();
    *num_matches = 0;

    let retlist = nvim_cmdexpand_call_user_expand_retlist(xp);
    if retlist.is_null() {
        return FAIL;
    }

    // C function handles TV_LIST_ITER_CONST + xstrdup + tv_list_unref
    nvim_cmdexpand_list_to_string_matches(retlist, matches, num_matches);
    OK
}

// =============================================================================
// ExpandUserLua
// =============================================================================

/// Expand from a Lua user completion function.
///
/// Mirrors `ExpandUserLua` from `cmdexpand.c`.
///
/// # Safety
///
/// All pointer arguments must be valid.
pub unsafe extern "C" fn rs_expand_user_lua(
    xp: *mut ExpandT,
    num_matches: *mut c_int,
    matches: *mut *mut *mut c_char,
) -> c_int {
    *matches = std::ptr::null_mut();
    *num_matches = 0;

    let retlist = nvim_cmdexpand_nlua_call_user_expand_retlist(xp);
    if retlist.is_null() {
        return FAIL;
    }

    nvim_cmdexpand_list_to_string_matches(retlist, matches, num_matches);
    OK
}
