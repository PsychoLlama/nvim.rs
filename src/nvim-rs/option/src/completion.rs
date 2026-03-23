//! Option completion utilities
//!
//! This module provides helper functions for option name and value completion
//! used by the command-line expansion code in `:set` command handling.

#![allow(clippy::missing_safety_doc)] // FFI functions safety is implicit
#![allow(clippy::cast_possible_wrap)] // FFI with C char types

use std::ffi::{c_char, c_int};

use crate::opt_index::K_OPT_COUNT;

// =============================================================================
// Completion Context Types
// =============================================================================

/// Option completion context type.
/// Corresponds to expansion context values in cmdexpand.h.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OptionExpandContext {
    /// Expand option names
    Settings = 37,
    /// Expand boolean option names only
    BoolSettings = 38,
    /// Expand option values
    SettingValue = 39,
}

// =============================================================================
// String Matching Utilities
// =============================================================================

/// Check if an option name matches a prefix (case-insensitive).
/// Returns 1 if the name starts with the prefix, 0 otherwise.
#[no_mangle]
pub unsafe extern "C" fn rs_option_name_matches_prefix(
    name: *const c_char,
    prefix: *const c_char,
) -> c_int {
    if name.is_null() || prefix.is_null() {
        return 0;
    }

    let mut n = name;
    let mut p = prefix;

    while *p != 0 {
        let nc = (*n as u8).to_ascii_lowercase();
        let pc = (*p as u8).to_ascii_lowercase();

        if nc != pc {
            return 0;
        }

        n = n.add(1);
        p = p.add(1);
    }

    1
}

/// Check if an option name contains a substring (case-insensitive).
/// Returns 1 if found, 0 otherwise.
#[no_mangle]
pub unsafe extern "C" fn rs_option_name_contains(
    name: *const c_char,
    substr: *const c_char,
) -> c_int {
    if name.is_null() || substr.is_null() {
        return 0;
    }

    // Get substring length
    let mut substr_len: usize = 0;
    let mut s = substr;
    while *s != 0 {
        substr_len += 1;
        s = s.add(1);
    }

    if substr_len == 0 {
        return 1; // Empty substring always matches
    }

    let mut n = name;
    while *n != 0 {
        // Try to match substring starting at this position
        let mut matched = true;
        let mut ni = n;
        let mut si = substr;

        for _ in 0..substr_len {
            if *ni == 0 {
                matched = false;
                break;
            }
            let nc = (*ni as u8).to_ascii_lowercase();
            let sc = (*si as u8).to_ascii_lowercase();
            if nc != sc {
                matched = false;
                break;
            }
            ni = ni.add(1);
            si = si.add(1);
        }

        if matched {
            return 1;
        }

        n = n.add(1);
    }

    0
}

/// Get the length of an option name string.
#[no_mangle]
pub unsafe extern "C" fn rs_option_name_len(name: *const c_char) -> usize {
    if name.is_null() {
        return 0;
    }

    let mut len: usize = 0;
    let mut p = name;
    while *p != 0 {
        len += 1;
        p = p.add(1);
    }
    len
}

/// Compare two option names for sorting (case-insensitive).
/// Returns: -1 if a < b, 0 if a == b, 1 if a > b
#[no_mangle]
pub unsafe extern "C" fn rs_option_name_compare(a: *const c_char, b: *const c_char) -> c_int {
    if a.is_null() && b.is_null() {
        return 0;
    }
    if a.is_null() {
        return -1;
    }
    if b.is_null() {
        return 1;
    }

    let mut pa = a;
    let mut pb = b;

    loop {
        let ca = (*pa as u8).to_ascii_lowercase();
        let cb = (*pb as u8).to_ascii_lowercase();

        if ca < cb {
            return -1;
        }
        if ca > cb {
            return 1;
        }
        if ca == 0 {
            return 0; // Both ended at same time
        }

        pa = pa.add(1);
        pb = pb.add(1);
    }
}

// =============================================================================
// Prefix Handling for Boolean Options
// =============================================================================

/// Get the base option name, stripping any "no" or "inv" prefix.
/// Returns a pointer to the start of the base name within the original string.
#[no_mangle]
pub unsafe extern "C" fn rs_option_strip_bool_prefix(name: *const c_char) -> *const c_char {
    if name.is_null() {
        return name;
    }

    let b0 = *name as u8;
    let b1 = *name.add(1) as u8;

    // Check for "no" prefix
    if b0 == b'n' && b1 == b'o' {
        return name.add(2);
    }

    // Check for "inv" prefix
    let b2 = *name.add(2) as u8;
    if b0 == b'i' && b1 == b'n' && b2 == b'v' {
        return name.add(3);
    }

    name
}

/// Check if a name starts with "no" and the rest matches an option.
/// This is useful for completing "no<option>" for boolean options.
/// Returns 1 if name starts with "no", 0 otherwise.
#[no_mangle]
pub unsafe extern "C" fn rs_option_has_no_completion_prefix(name: *const c_char) -> c_int {
    if name.is_null() {
        return 0;
    }

    let b0 = *name as u8;
    let b1 = *name.add(1) as u8;

    c_int::from(b0 == b'n' && b1 == b'o' && *name.add(2) != 0)
}

/// Check if a name starts with "inv" and the rest matches an option.
/// Returns 1 if name starts with "inv", 0 otherwise.
#[no_mangle]
pub unsafe extern "C" fn rs_option_has_inv_completion_prefix(name: *const c_char) -> c_int {
    if name.is_null() {
        return 0;
    }

    let b0 = *name as u8;
    let b1 = *name.add(1) as u8;
    let b2 = *name.add(2) as u8;

    c_int::from(b0 == b'i' && b1 == b'n' && b2 == b'v' && *name.add(3) != 0)
}

// =============================================================================
// Value Completion Helpers
// =============================================================================

/// Check if a character is valid in an option value (for :set).
/// Some characters need escaping in option values.
#[no_mangle]
pub extern "C" fn rs_option_char_needs_escape(ch: c_char) -> c_int {
    let c = ch as u8;
    // Characters that need escaping in option values
    c_int::from(c == b'\\' || c == b' ' || c == b'\t' || c == b'|' || c == b'"' || c == b'#')
}

/// Check if a string contains characters that need escaping for :set.
/// Returns 1 if escaping is needed, 0 otherwise.
#[no_mangle]
pub unsafe extern "C" fn rs_option_value_needs_escape(value: *const c_char) -> c_int {
    if value.is_null() {
        return 0;
    }

    let mut p = value;
    while *p != 0 {
        if rs_option_char_needs_escape(*p) != 0 {
            return 1;
        }
        p = p.add(1);
    }
    0
}

// =============================================================================
// List Flag Expansion Helpers
// =============================================================================

/// Get the list of valid flags for an option as a null-terminated string.
/// Used by expand_set_opt_listflag in C.
/// Get valid flags for 'concealcursor' option.
#[no_mangle]
pub extern "C" fn rs_expand_concealcursor_flags() -> *const c_char {
    c"nvic".as_ptr()
}

/// Get valid flags for 'cpoptions' option.
#[no_mangle]
pub extern "C" fn rs_expand_cpoptions_flags() -> *const c_char {
    c"aAbBcCdDeEfFiIJKlLmMnoOpPqrRsStuvWxXyZ$!%+>;~_".as_ptr()
}

/// Get valid flags for 'formatoptions' option.
#[no_mangle]
pub extern "C" fn rs_expand_formatoptions_flags() -> *const c_char {
    c"tcro/q2vlb1mMBn,aw]jp".as_ptr()
}

/// Get valid flags for 'mouse' option.
#[no_mangle]
pub extern "C" fn rs_expand_mouse_flags() -> *const c_char {
    c"anvichr".as_ptr()
}

/// Get valid flags for 'shortmess' option.
#[no_mangle]
pub extern "C" fn rs_expand_shortmess_flags() -> *const c_char {
    c"rwoOstTWIcCqaAFnlxfiS".as_ptr()
}

/// Get valid flags for 'whichwrap' option.
#[no_mangle]
pub extern "C" fn rs_expand_whichwrap_flags() -> *const c_char {
    c"bshl<>[]~".as_ptr()
}

// =============================================================================
// Flag Expansion Utilities
// =============================================================================

/// Check if a flag character is in the current option value.
/// Used to filter out already-set flags during completion.
/// Returns 1 if flag is present, 0 otherwise.
#[no_mangle]
pub unsafe extern "C" fn rs_flag_in_value(value: *const c_char, flag: c_char) -> c_int {
    if value.is_null() {
        return 0;
    }

    let mut p = value;
    while *p != 0 {
        if *p == flag {
            return 1;
        }
        p = p.add(1);
    }
    0
}

/// Count how many flags from the allowed set are NOT in the current value.
/// Used to allocate the correct size for expansion results.
#[no_mangle]
pub unsafe extern "C" fn rs_count_missing_flags(
    value: *const c_char,
    allowed: *const c_char,
) -> usize {
    if allowed.is_null() {
        return 0;
    }

    let mut count: usize = 0;
    let mut p = allowed;

    while *p != 0 {
        if rs_flag_in_value(value, *p) == 0 {
            count += 1;
        }
        p = p.add(1);
    }

    count
}

/// Get the Nth missing flag from the allowed set.
/// Returns the flag character, or 0 if idx is out of range.
#[no_mangle]
pub unsafe extern "C" fn rs_get_missing_flag(
    value: *const c_char,
    allowed: *const c_char,
    idx: usize,
) -> c_char {
    if allowed.is_null() {
        return 0;
    }

    let mut found: usize = 0;
    let mut p = allowed;

    while *p != 0 {
        if rs_flag_in_value(value, *p) == 0 {
            if found == idx {
                return *p;
            }
            found += 1;
        }
        p = p.add(1);
    }

    0
}

// =============================================================================
// Value List Expansion
// =============================================================================

/// Result of parsing a comma-separated list for expansion.
#[repr(C)]
pub struct ExpandListResult {
    /// Pointer to the start of the current item
    pub item_start: *const c_char,
    /// Length of the current item
    pub item_len: usize,
    /// Pointer to the next item (after comma) or NULL if end
    pub next: *const c_char,
}

/// Parse the next item from a comma-separated list.
/// Used for expanding values like 'backspace=indent,eol,start'.
#[no_mangle]
pub unsafe extern "C" fn rs_expand_next_item(s: *const c_char) -> ExpandListResult {
    let mut result = ExpandListResult {
        item_start: s,
        item_len: 0,
        next: std::ptr::null(),
    };

    if s.is_null() || *s == 0 {
        return result;
    }

    let mut p = s;

    // Find end of item (comma or end of string)
    while *p != 0 && *p as u8 != b',' {
        // Handle backslash escapes
        if *p as u8 == b'\\' && *p.add(1) != 0 {
            p = p.add(1);
            result.item_len += 1;
        }
        p = p.add(1);
        result.item_len += 1;
    }

    // Set next pointer
    if *p as u8 == b',' {
        result.next = p.add(1);
    }

    result
}

/// Check if an item is in a comma-separated list.
/// Returns 1 if found, 0 otherwise.
#[no_mangle]
pub unsafe extern "C" fn rs_item_in_list(
    list: *const c_char,
    item: *const c_char,
    item_len: usize,
) -> c_int {
    if list.is_null() || item.is_null() || item_len == 0 {
        return 0;
    }

    let mut s = list;

    while *s != 0 {
        let result = rs_expand_next_item(s);

        if result.item_len == item_len {
            // Compare items
            let mut matches = true;
            for i in 0..item_len {
                if *result.item_start.add(i) != *item.add(i) {
                    matches = false;
                    break;
                }
            }
            if matches {
                return 1;
            }
        }

        if result.next.is_null() {
            break;
        }
        s = result.next;
    }

    0
}

// =============================================================================
// Prefix/Suffix Handling for Completion
// =============================================================================

/// Find the start of the last item in a comma-separated list.
/// Useful for completing after 'set opt+=value1,'.
#[no_mangle]
pub unsafe extern "C" fn rs_find_last_item_start(s: *const c_char) -> *const c_char {
    if s.is_null() || *s == 0 {
        return s;
    }

    let mut last_start = s;
    let mut p = s;

    while *p != 0 {
        if *p as u8 == b',' && *p.add(1) != 0 {
            last_start = p.add(1);
        }
        p = p.add(1);
    }

    last_start
}

/// Check if the string ends with a comma (ready for new item).
#[no_mangle]
pub unsafe extern "C" fn rs_ends_with_comma(s: *const c_char) -> c_int {
    if s.is_null() || *s == 0 {
        return 0;
    }

    let mut p = s;
    let mut last: c_char = 0;

    while *p != 0 {
        last = *p;
        p = p.add(1);
    }

    c_int::from(last as u8 == b',')
}

// =============================================================================
// Phase 2: ExpandSettings / match_str
// =============================================================================

// FFI declarations for ExpandSettings migration
extern "C" {
    fn nvim_option_get_fullname(opt_idx: c_int) -> *const c_char;
    fn nvim_option_get_shortname(opt_idx: c_int) -> *const c_char;
    #[link_name = "option_has_type"]
    fn nvim_option_has_type(opt_idx: c_int, type_: c_int) -> c_int;
    #[link_name = "is_option_hidden"]
    fn nvim_opt_is_hidden(opt_idx: c_int) -> c_int;
    fn nvim_xp_get_context(xp: *mut std::ffi::c_void) -> c_int;
    fn nvim_regmatch_get_rm_ic(regmatch: *const std::ffi::c_void) -> c_int;
    fn nvim_regmatch_set_rm_ic(regmatch: *mut std::ffi::c_void, val: c_int);
    fn nvim_excmds_regexec(rm: *mut std::ffi::c_void, line: *const c_char) -> c_int;
    fn fuzzy_match_str(str_: *mut c_char, pat: *const c_char) -> c_int;
    fn nvim_option_fuzzymatches_to_strmatches(
        fuzmatch: *mut std::ffi::c_void,
        matches: *mut *mut *mut c_char,
        count: c_int,
    );
    fn cmdline_fuzzy_complete(fuzzystr: *const c_char) -> bool;
    fn nvim_option_get_fuzmatch_size() -> usize;
    fn nvim_option_fuzmatch_set(
        fuzmatch: *mut std::ffi::c_void,
        idx: c_int,
        str_: *const c_char,
        score: c_int,
    );
    fn xstrdup(s: *const c_char) -> *mut c_char;
    fn xmalloc(size: usize) -> *mut c_char;
}

/// Option boolean type value (kOptValTypeBoolean = 0)
const K_OPT_VAL_TYPE_BOOLEAN: c_int = 0;

/// EXPAND_BOOL_SETTINGS context value
const EXPAND_BOOL_SETTINGS: c_int = 5;

/// FUZZY_SCORE_NONE value (INT_MIN)
const FUZZY_SCORE_NONE: c_int = i32::MIN;

/// Match a string against a regex or fuzzy pattern.
///
/// Fuzzy match context passed to `match_str_impl`.
struct FuzzyCtx {
    fuzzystr: *const c_char,
    fuzmatch: *mut std::ffi::c_void,
}

/// If not fuzzy: calls vim_regexec; on match, stores str in matches[idx] (unless test_only).
/// If fuzzy: calls fuzzy_match_str; on match, stores fuzmatch entry (unless test_only).
///
/// Returns true if matched.
///
/// # Safety
/// All pointers must be valid.
#[allow(clippy::cast_possible_truncation)]
unsafe fn match_str_impl(
    str_: *const c_char,
    regmatch: *mut std::ffi::c_void,
    matches: *mut *mut c_char,
    idx: c_int,
    test_only: bool,
    fuzzy_ctx: Option<&FuzzyCtx>,
) -> bool {
    if let Some(ctx) = fuzzy_ctx {
        let score = fuzzy_match_str(str_.cast_mut(), ctx.fuzzystr);
        if score != FUZZY_SCORE_NONE {
            if !test_only && !ctx.fuzmatch.is_null() {
                nvim_option_fuzmatch_set(ctx.fuzmatch, idx, str_, score);
            }
            return true;
        }
    } else if nvim_excmds_regexec(regmatch, str_) != 0 {
        if !test_only && !matches.is_null() {
            *matches.add(idx as usize) = xstrdup(str_);
        }
        return true;
    }
    false
}

/// Expand option names for command-line completion (translation of C ExpandSettings).
///
/// Two-pass algorithm: first count matches, then fill allocated arrays.
///
/// # Safety
/// All pointers must be valid.
#[export_name = "ExpandSettings"]
#[allow(clippy::must_use_candidate)]
#[allow(clippy::cast_possible_truncation)]
#[allow(clippy::cast_possible_wrap)]
#[allow(clippy::cast_sign_loss)]
pub unsafe extern "C" fn rs_expand_option_settings(
    xp: *mut std::ffi::c_void,
    regmatch: *mut std::ffi::c_void,
    fuzzystr: *const c_char,
    num_matches: *mut c_int,
    matches: *mut *mut *mut c_char,
    can_fuzzy: c_int,
) -> c_int {
    // "all" is the only keyword we offer outside of option names
    static KEYWORD_ALL: &[u8] = b"all\0";

    let mut num_normal: c_int = 0;
    let mut count: c_int = 0;
    let ic = nvim_regmatch_get_rm_ic(regmatch);
    let fuzzy = can_fuzzy != 0 && cmdline_fuzzy_complete(fuzzystr);
    let mut fuzmatch: *mut std::ffi::c_void = std::ptr::null_mut();
    // Two-pass loop: loop==0 counts, loop==1 fills
    let mut loop_: c_int = 0;
    while loop_ <= 1 {
        nvim_regmatch_set_rm_ic(regmatch, ic);
        let fuzzy_ctx = fuzzy.then_some(FuzzyCtx { fuzzystr, fuzmatch });
        let fill_matches = if loop_ == 1 {
            *matches
        } else {
            std::ptr::null_mut()
        };
        let test_only = loop_ == 0;

        // Match "all" keyword (only for non-bool-settings context)
        if nvim_xp_get_context(xp) != EXPAND_BOOL_SETTINGS {
            let all_ptr = KEYWORD_ALL.as_ptr().cast::<c_char>();
            let kw_matched = match_str_impl(
                all_ptr,
                regmatch,
                fill_matches,
                count,
                test_only,
                fuzzy_ctx.as_ref(),
            );
            if kw_matched {
                if loop_ == 0 {
                    num_normal += 1;
                } else {
                    count += 1;
                }
            }
        }

        // Match option names
        for opt_idx in 0..K_OPT_COUNT {
            if nvim_opt_is_hidden(opt_idx) != 0 {
                continue;
            }
            // Bool-only context: skip non-boolean options
            if nvim_xp_get_context(xp) == EXPAND_BOOL_SETTINGS
                && nvim_option_has_type(opt_idx, K_OPT_VAL_TYPE_BOOLEAN) == 0
            {
                continue;
            }

            let fullname = nvim_option_get_fullname(opt_idx);

            let name_matched = match_str_impl(
                fullname,
                regmatch,
                fill_matches,
                count,
                test_only,
                fuzzy_ctx.as_ref(),
            );
            if name_matched {
                if loop_ == 0 {
                    num_normal += 1;
                } else {
                    count += 1;
                }
            } else if fuzzy_ctx.is_none() {
                // Also try shortname for regex (not fuzzy - fuzzy already matches both)
                let shortname = nvim_option_get_shortname(opt_idx);
                if !shortname.is_null() && nvim_excmds_regexec(regmatch, shortname) != 0 {
                    if loop_ == 0 {
                        num_normal += 1;
                    } else {
                        // Store fullname (not shortname) in matches
                        *(*matches).add(count as usize) = xstrdup(fullname);
                        count += 1;
                    }
                }
            }
        }

        if loop_ == 0 {
            if num_normal == 0 {
                return 0; // OK
            }
            *num_matches = num_normal;
            if fuzzy {
                let fm_size = nvim_option_get_fuzmatch_size();
                fuzmatch = xmalloc((*num_matches as usize) * fm_size).cast();
            } else {
                #[allow(clippy::cast_ptr_alignment)]
                {
                    *matches =
                        xmalloc((*num_matches as usize) * std::mem::size_of::<*mut c_char>())
                            .cast::<*mut c_char>();
                }
            }
        }
        loop_ += 1;
    }

    if fuzzy {
        nvim_option_fuzzymatches_to_strmatches(fuzmatch, matches, count);
    }

    0 // OK
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::CString;

    #[test]
    fn test_option_name_matches_prefix() {
        unsafe {
            let name = CString::new("autoindent").unwrap();
            let prefix1 = CString::new("auto").unwrap();
            let prefix2 = CString::new("AUTO").unwrap();
            let prefix3 = CString::new("indent").unwrap();

            assert_eq!(
                rs_option_name_matches_prefix(name.as_ptr(), prefix1.as_ptr()),
                1
            );
            assert_eq!(
                rs_option_name_matches_prefix(name.as_ptr(), prefix2.as_ptr()),
                1
            );
            assert_eq!(
                rs_option_name_matches_prefix(name.as_ptr(), prefix3.as_ptr()),
                0
            );
        }
    }

    #[test]
    fn test_option_name_contains() {
        unsafe {
            let name = CString::new("autoindent").unwrap();
            let sub1 = CString::new("ind").unwrap();
            let sub2 = CString::new("xyz").unwrap();
            let empty = CString::new("").unwrap();

            assert_eq!(rs_option_name_contains(name.as_ptr(), sub1.as_ptr()), 1);
            assert_eq!(rs_option_name_contains(name.as_ptr(), sub2.as_ptr()), 0);
            assert_eq!(rs_option_name_contains(name.as_ptr(), empty.as_ptr()), 1);
        }
    }

    #[test]
    fn test_option_name_compare() {
        unsafe {
            let a = CString::new("autoindent").unwrap();
            let b = CString::new("expandtab").unwrap();
            let c = CString::new("AUTOINDENT").unwrap();

            assert!(rs_option_name_compare(a.as_ptr(), b.as_ptr()) < 0);
            assert!(rs_option_name_compare(b.as_ptr(), a.as_ptr()) > 0);
            assert_eq!(rs_option_name_compare(a.as_ptr(), c.as_ptr()), 0);
        }
    }

    #[test]
    fn test_option_strip_bool_prefix() {
        unsafe {
            let no_number = CString::new("nonumber").unwrap();
            let inv_number = CString::new("invnumber").unwrap();
            let number = CString::new("number").unwrap();

            let stripped_no = rs_option_strip_bool_prefix(no_number.as_ptr());
            let stripped_inv = rs_option_strip_bool_prefix(inv_number.as_ptr());
            let stripped_plain = rs_option_strip_bool_prefix(number.as_ptr());

            assert_eq!(*stripped_no as u8, b'n');
            assert_eq!(*stripped_no.add(1) as u8, b'u');
            assert_eq!(*stripped_inv as u8, b'n');
            assert_eq!(*stripped_plain as u8, b'n');
        }
    }

    #[test]
    fn test_option_char_needs_escape() {
        assert_eq!(rs_option_char_needs_escape(b'\\' as c_char), 1);
        assert_eq!(rs_option_char_needs_escape(b' ' as c_char), 1);
        assert_eq!(rs_option_char_needs_escape(b'|' as c_char), 1);
        assert_eq!(rs_option_char_needs_escape(b'a' as c_char), 0);
        assert_eq!(rs_option_char_needs_escape(b'z' as c_char), 0);
    }

    #[test]
    fn test_option_value_needs_escape() {
        unsafe {
            let needs = CString::new("some value").unwrap();
            let no_needs = CString::new("value").unwrap();
            let with_pipe = CString::new("a|b").unwrap();

            assert_eq!(rs_option_value_needs_escape(needs.as_ptr()), 1);
            assert_eq!(rs_option_value_needs_escape(no_needs.as_ptr()), 0);
            assert_eq!(rs_option_value_needs_escape(with_pipe.as_ptr()), 1);
        }
    }

    // =========================================================================
    // Flag Expansion Tests
    // =========================================================================

    #[test]
    fn test_expand_flags_returns_valid_strings() {
        unsafe {
            // Just check that the pointers are non-null and the first char is valid
            let cocu = rs_expand_concealcursor_flags();
            assert!(!cocu.is_null());
            assert_eq!(*cocu as u8, b'n');

            let cpo = rs_expand_cpoptions_flags();
            assert!(!cpo.is_null());
            assert_eq!(*cpo as u8, b'a');

            let fo = rs_expand_formatoptions_flags();
            assert!(!fo.is_null());
            assert_eq!(*fo as u8, b't');

            let mouse = rs_expand_mouse_flags();
            assert!(!mouse.is_null());
            assert_eq!(*mouse as u8, b'a');

            let shm = rs_expand_shortmess_flags();
            assert!(!shm.is_null());
            assert_eq!(*shm as u8, b'r');

            let ww = rs_expand_whichwrap_flags();
            assert!(!ww.is_null());
            assert_eq!(*ww as u8, b'b');
        }
    }

    #[test]
    fn test_flag_in_value() {
        unsafe {
            let value = CString::new("nvic").unwrap();

            assert_eq!(rs_flag_in_value(value.as_ptr(), b'n' as c_char), 1);
            assert_eq!(rs_flag_in_value(value.as_ptr(), b'v' as c_char), 1);
            assert_eq!(rs_flag_in_value(value.as_ptr(), b'i' as c_char), 1);
            assert_eq!(rs_flag_in_value(value.as_ptr(), b'c' as c_char), 1);
            assert_eq!(rs_flag_in_value(value.as_ptr(), b'x' as c_char), 0);
            assert_eq!(rs_flag_in_value(std::ptr::null(), b'n' as c_char), 0);
        }
    }

    #[test]
    fn test_count_missing_flags() {
        unsafe {
            let value = CString::new("ac").unwrap();
            let allowed = CString::new("abcd").unwrap();

            // 'a' and 'c' are present, 'b' and 'd' are missing
            assert_eq!(rs_count_missing_flags(value.as_ptr(), allowed.as_ptr()), 2);

            // Empty value - all flags are missing
            let empty = CString::new("").unwrap();
            assert_eq!(rs_count_missing_flags(empty.as_ptr(), allowed.as_ptr()), 4);

            // All flags present
            let full = CString::new("abcd").unwrap();
            assert_eq!(rs_count_missing_flags(full.as_ptr(), allowed.as_ptr()), 0);
        }
    }

    #[test]
    fn test_get_missing_flag() {
        unsafe {
            let value = CString::new("ac").unwrap();
            let allowed = CString::new("abcd").unwrap();

            // Missing flags are 'b' (idx 0) and 'd' (idx 1)
            assert_eq!(
                rs_get_missing_flag(value.as_ptr(), allowed.as_ptr(), 0) as u8,
                b'b'
            );
            assert_eq!(
                rs_get_missing_flag(value.as_ptr(), allowed.as_ptr(), 1) as u8,
                b'd'
            );
            // Out of range
            assert_eq!(rs_get_missing_flag(value.as_ptr(), allowed.as_ptr(), 2), 0);
        }
    }

    // =========================================================================
    // List Expansion Tests
    // =========================================================================

    #[test]
    fn test_expand_next_item() {
        unsafe {
            // Simple comma-separated list
            let list = CString::new("indent,eol,start").unwrap();

            let r1 = rs_expand_next_item(list.as_ptr());
            assert_eq!(r1.item_len, 6); // "indent"
            assert!(!r1.next.is_null());

            let r2 = rs_expand_next_item(r1.next);
            assert_eq!(r2.item_len, 3); // "eol"
            assert!(!r2.next.is_null());

            let r3 = rs_expand_next_item(r2.next);
            assert_eq!(r3.item_len, 5); // "start"
            assert!(r3.next.is_null());
        }
    }

    #[test]
    fn test_expand_next_item_with_escape() {
        unsafe {
            // Item with escaped comma
            let list = CString::new("a\\,b,c").unwrap();

            let r1 = rs_expand_next_item(list.as_ptr());
            assert_eq!(r1.item_len, 4); // "a\\,b" (escaped comma is part of item)
            assert!(!r1.next.is_null());

            let r2 = rs_expand_next_item(r1.next);
            assert_eq!(r2.item_len, 1); // "c"
            assert!(r2.next.is_null());
        }
    }

    #[test]
    fn test_item_in_list() {
        unsafe {
            let list = CString::new("indent,eol,start").unwrap();
            let indent = CString::new("indent").unwrap();
            let eol = CString::new("eol").unwrap();
            let missing = CString::new("nostop").unwrap();

            assert_eq!(rs_item_in_list(list.as_ptr(), indent.as_ptr(), 6), 1);
            assert_eq!(rs_item_in_list(list.as_ptr(), eol.as_ptr(), 3), 1);
            assert_eq!(rs_item_in_list(list.as_ptr(), missing.as_ptr(), 6), 0);
        }
    }

    #[test]
    fn test_find_last_item_start() {
        unsafe {
            let items = CString::new("a,b,c").unwrap();
            let last_ptr = rs_find_last_item_start(items.as_ptr());
            assert_eq!(*last_ptr as u8, b'c');

            let single = CString::new("only").unwrap();
            let single_ptr = rs_find_last_item_start(single.as_ptr());
            assert_eq!(*single_ptr as u8, b'o');
        }
    }

    #[test]
    fn test_ends_with_comma() {
        unsafe {
            let with_comma = CString::new("a,b,").unwrap();
            let without = CString::new("a,b,c").unwrap();
            let empty = CString::new("").unwrap();

            assert_eq!(rs_ends_with_comma(with_comma.as_ptr()), 1);
            assert_eq!(rs_ends_with_comma(without.as_ptr()), 0);
            assert_eq!(rs_ends_with_comma(empty.as_ptr()), 0);
        }
    }
}
