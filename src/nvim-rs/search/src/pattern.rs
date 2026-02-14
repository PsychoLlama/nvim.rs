//! Search pattern storage and retrieval
//!
//! This module provides functions for managing search patterns,
//! including the main search pattern, substitute pattern, and
//! the last used pattern selection.

use std::ffi::{c_char, c_int};

use crate::state;

// =============================================================================
// C Functions
// =============================================================================

extern "C" {
    /// Get the pattern string from spats array.
    fn nvim_get_spat_pat(idx: c_int) -> *const c_char;

    // Batch accessors for ShaDa pattern get/set
    fn nvim_spat_memcpy_out(idx: c_int, out: *mut SearchPatternC);
    fn nvim_spat_memcpy_in(idx: c_int, inp: *const SearchPatternC);
    fn nvim_free_spat(idx: c_int);
    fn nvim_clear_spat_off(idx: c_int);
    fn nvim_clear_spats();
    fn nvim_free_mr_pattern();
    fn nvim_call_set_vv_searchforward();

    // Batch save/restore helpers (Phase 3)
    fn nvim_inc_save_level() -> c_int;
    fn nvim_dec_save_level() -> c_int;
    fn nvim_save_search_patterns_batch();
    fn nvim_restore_search_patterns_batch();
    fn nvim_inc_did_save() -> c_int;
    fn nvim_dec_did_save() -> c_int;
    fn nvim_save_last_search_spat_batch();
    fn nvim_restore_last_search_spat_batch();
    fn nvim_call_iemsg_restore_mismatch();

    // Batch helpers for search_regcomp and pattern compilation (Phase 4)
    fn nvim_emsg_noprevre();
    fn nvim_emsg_nopresub();
    fn nvim_set_rc_did_emsg();
    fn nvim_clear_rc_did_emsg();
    fn nvim_search_add_to_history(pat: *const c_char, patlen: usize);
    fn nvim_set_mr_pattern(pat: *const c_char, patlen: usize);
    fn nvim_get_cmdmod_keeppatterns() -> c_int;
    fn nvim_save_re_pat_batch(idx: c_int, pat: *const c_char, patlen: usize, magic: c_int);
    fn nvim_search_regcomp_compile(
        pat: *const c_char,
        magic: c_int,
        regmatch: *mut std::ffi::c_void,
    ) -> c_int;
    fn nvim_set_last_search_pat_batch(
        s: *const c_char,
        idx: c_int,
        magic: c_int,
        setlast: c_int,
    );
    fn nvim_inc_emsg_off();
    fn nvim_dec_emsg_off();
    fn nvim_spats_pat_is_null(idx: c_int) -> c_int;
    fn nvim_spats_get_pat_and_len(
        idx: c_int,
        patlen: *mut usize,
        magic: *mut c_int,
        no_scs: *mut c_int,
    ) -> *const c_char;
    fn nvim_set_no_smartcase(val: c_int);
}

/// Opaque representation of C SearchPattern struct.
/// Must match the C struct layout exactly.
#[repr(C)]
pub struct SearchPatternC {
    pub pat: *mut c_char,
    pub patlen: usize,
    pub magic: bool,
    pub no_scs: bool,
    pub timestamp: u64, // Timestamp
    pub off: SearchOffsetC,
    pub additional_data: *mut std::ffi::c_void,
}

/// Opaque representation of C SearchOffset struct.
#[repr(C)]
pub struct SearchOffsetC {
    pub dir: i8,
    pub line: bool,
    pub end: bool,
    pub off: i64,
}

// =============================================================================
// Pattern Index Functions
// =============================================================================

/// Get the pattern string for the search pattern (index 0).
///
/// # Safety
/// Returns a pointer to allocated memory that may be freed.
#[inline]
pub unsafe fn get_search_pattern() -> *const c_char {
    nvim_get_spat_pat(state::RE_SEARCH)
}

/// Get the pattern string for the substitute pattern (index 1).
///
/// # Safety
/// Returns a pointer to allocated memory that may be freed.
#[inline]
pub unsafe fn get_subst_pattern() -> *const c_char {
    nvim_get_spat_pat(state::RE_SUBST)
}

/// Get the pattern length for the search pattern.
#[inline]
pub fn get_search_pattern_len() -> usize {
    state::get_spat_patlen(state::RE_SEARCH)
}

/// Get the pattern length for the substitute pattern.
#[inline]
pub fn get_subst_pattern_len() -> usize {
    state::get_spat_patlen(state::RE_SUBST)
}

/// Check if the search pattern is empty or NULL.
#[inline]
pub fn search_pattern_is_empty() -> bool {
    // SAFETY: Just checking for NULL pointer
    unsafe {
        let pat = get_search_pattern();
        pat.is_null() || get_search_pattern_len() == 0
    }
}

/// Check if the substitute pattern is empty or NULL.
#[inline]
pub fn subst_pattern_is_empty() -> bool {
    // SAFETY: Just checking for NULL pointer
    unsafe {
        let pat = get_subst_pattern();
        pat.is_null() || get_subst_pattern_len() == 0
    }
}

/// Get the pattern for the last used index (search or substitute).
///
/// # Safety
/// Returns a pointer to allocated memory that may be freed.
#[inline]
pub unsafe fn get_last_used_pattern() -> *const c_char {
    nvim_get_spat_pat(state::get_last_idx())
}

/// Get the length of the last used pattern.
#[inline]
pub fn get_last_used_pattern_len() -> usize {
    state::get_spat_patlen(state::get_last_idx())
}

/// Check if the last used pattern was the search pattern.
#[inline]
pub fn last_was_search() -> bool {
    state::get_last_idx() == state::RE_SEARCH
}

/// Check if the last used pattern was the substitute pattern.
#[inline]
pub fn last_was_subst() -> bool {
    state::get_last_idx() == state::RE_SUBST
}

// =============================================================================
// Pattern Attributes
// =============================================================================

/// Check if magic mode is enabled for the search pattern.
#[inline]
pub fn search_pattern_magic() -> bool {
    state::get_spat_magic(state::RE_SEARCH)
}

/// Check if magic mode is enabled for the substitute pattern.
#[inline]
pub fn subst_pattern_magic() -> bool {
    state::get_spat_magic(state::RE_SUBST)
}

/// Check if no_smartcase is set for the search pattern.
#[inline]
pub fn search_pattern_no_scs() -> bool {
    state::get_spat_no_scs(state::RE_SEARCH)
}

/// Check if no_smartcase is set for the substitute pattern.
#[inline]
pub fn subst_pattern_no_scs() -> bool {
    state::get_spat_no_scs(state::RE_SUBST)
}

// =============================================================================
// Match Result Pattern (mr_pattern)
// =============================================================================

/// Get the pattern used by search_regcomp().
///
/// This is the pattern that was actually used for the last search,
/// which may have been reversed if 'rl' option is set.
///
/// # Safety
/// Returns a pointer to static memory that may be invalidated.
#[inline]
pub unsafe fn get_mr_pattern() -> *const c_char {
    state::get_mr_pattern()
}

/// Get the length of the mr_pattern.
#[inline]
pub fn get_mr_pattern_len() -> usize {
    state::get_mr_patternlen()
}

/// Check if mr_pattern is empty or NULL.
#[inline]
pub fn mr_pattern_is_empty() -> bool {
    // SAFETY: Just checking for NULL pointer
    unsafe {
        let pat = get_mr_pattern();
        pat.is_null() || get_mr_pattern_len() == 0
    }
}

// =============================================================================
// Pattern String Helpers
// =============================================================================

/// Get the search pattern as a Rust string slice, if available.
///
/// Returns None if the pattern is NULL or invalid UTF-8.
///
/// # Safety
/// The returned string slice is only valid as long as the pattern
/// is not modified in C code.
pub unsafe fn search_pattern_as_str() -> Option<&'static str> {
    let ptr = get_search_pattern();
    if ptr.is_null() {
        return None;
    }
    let len = get_search_pattern_len();
    if len == 0 {
        return Some("");
    }
    let slice = std::slice::from_raw_parts(ptr as *const u8, len);
    std::str::from_utf8(slice).ok()
}

/// Get the substitute pattern as a Rust string slice, if available.
///
/// # Safety
/// The returned string slice is only valid as long as the pattern
/// is not modified in C code.
pub unsafe fn subst_pattern_as_str() -> Option<&'static str> {
    let ptr = get_subst_pattern();
    if ptr.is_null() {
        return None;
    }
    let len = get_subst_pattern_len();
    if len == 0 {
        return Some("");
    }
    let slice = std::slice::from_raw_parts(ptr as *const u8, len);
    std::str::from_utf8(slice).ok()
}

/// Get the mr_pattern as a Rust string slice, if available.
///
/// # Safety
/// The returned string slice is only valid until the next search operation.
pub unsafe fn mr_pattern_as_str() -> Option<&'static str> {
    let ptr = get_mr_pattern();
    if ptr.is_null() {
        return None;
    }
    let len = get_mr_pattern_len();
    if len == 0 {
        return Some("");
    }
    let slice = std::slice::from_raw_parts(ptr as *const u8, len);
    std::str::from_utf8(slice).ok()
}

// =============================================================================
// FFI Exports
// =============================================================================

/// FFI: Get the search pattern pointer.
///
/// # Safety
/// Returns a pointer to allocated memory that may be freed by C code.
#[no_mangle]
pub unsafe extern "C" fn rs_get_search_pattern() -> *const c_char {
    get_search_pattern()
}

/// FFI: Get the substitute pattern pointer.
///
/// # Safety
/// Returns a pointer to allocated memory that may be freed by C code.
#[no_mangle]
pub unsafe extern "C" fn rs_get_subst_pattern() -> *const c_char {
    get_subst_pattern()
}

/// FFI: Get the search pattern length.
#[no_mangle]
pub extern "C" fn rs_get_search_pattern_len() -> usize {
    get_search_pattern_len()
}

/// FFI: Get the substitute pattern length.
#[no_mangle]
pub extern "C" fn rs_get_subst_pattern_len() -> usize {
    get_subst_pattern_len()
}

/// FFI: Check if search pattern is empty.
#[no_mangle]
pub extern "C" fn rs_search_pattern_is_empty() -> c_int {
    c_int::from(search_pattern_is_empty())
}

/// FFI: Check if substitute pattern is empty.
#[no_mangle]
pub extern "C" fn rs_subst_pattern_is_empty() -> c_int {
    c_int::from(subst_pattern_is_empty())
}

/// FFI: Get the last used pattern pointer.
///
/// # Safety
/// Returns a pointer to allocated memory that may be freed by C code.
#[no_mangle]
pub unsafe extern "C" fn rs_get_last_used_pattern() -> *const c_char {
    get_last_used_pattern()
}

/// FFI: Get the last used pattern length.
#[no_mangle]
pub extern "C" fn rs_get_last_used_pattern_len() -> usize {
    get_last_used_pattern_len()
}

/// FFI: Check if last pattern was search.
#[no_mangle]
pub extern "C" fn rs_last_was_search() -> c_int {
    c_int::from(last_was_search())
}

/// FFI: Check if last pattern was substitute.
#[no_mangle]
pub extern "C" fn rs_last_was_subst() -> c_int {
    c_int::from(last_was_subst())
}

/// FFI: Check if search pattern has magic.
#[no_mangle]
pub extern "C" fn rs_search_pattern_magic() -> c_int {
    c_int::from(search_pattern_magic())
}

/// FFI: Check if substitute pattern has magic.
#[no_mangle]
pub extern "C" fn rs_subst_pattern_magic() -> c_int {
    c_int::from(subst_pattern_magic())
}

/// FFI: Get mr_pattern pointer.
///
/// # Safety
/// Returns a pointer to static memory that may be invalidated by subsequent searches.
#[no_mangle]
pub unsafe extern "C" fn rs_get_mr_pattern() -> *const c_char {
    get_mr_pattern()
}

/// FFI: Get mr_pattern length.
#[no_mangle]
pub extern "C" fn rs_get_mr_pattern_len() -> usize {
    get_mr_pattern_len()
}

/// FFI: Check if mr_pattern is empty.
#[no_mangle]
pub extern "C" fn rs_mr_pattern_is_empty() -> c_int {
    c_int::from(mr_pattern_is_empty())
}

// =============================================================================
// ShaDa Pattern Get/Set (Phase 2)
// =============================================================================

/// Get last search pattern (memcpy spats[0] out).
///
/// # Safety
/// `pat` must point to a valid, properly aligned SearchPattern-sized buffer.
#[no_mangle]
pub unsafe extern "C" fn rs_get_search_pattern_shada(pat: *mut SearchPatternC) {
    nvim_spat_memcpy_out(state::RE_SEARCH, pat);
}

/// Get last substitute pattern (memcpy spats[1] out, then clear off).
///
/// # Safety
/// `pat` must point to a valid, properly aligned SearchPattern-sized buffer.
#[no_mangle]
pub unsafe extern "C" fn rs_get_substitute_pattern_shada(pat: *mut SearchPatternC) {
    nvim_spat_memcpy_out(state::RE_SUBST, pat);
    // Clear the offset fields, matching original get_substitute_pattern behavior
    if !pat.is_null() {
        let off = &mut (*pat).off;
        off.dir = b'/' as i8;
        off.line = false;
        off.end = false;
        off.off = 0;
    }
}

/// Set last search pattern (free old, memcpy in, update vv_searchforward).
///
/// # Safety
/// `pat` must point to a valid SearchPattern.
#[no_mangle]
pub unsafe extern "C" fn rs_set_search_pattern_shada(pat: *const SearchPatternC) {
    nvim_spat_memcpy_in(state::RE_SEARCH, pat);
    nvim_call_set_vv_searchforward();
}

/// Set last substitute pattern (free old, memcpy in, clear off).
///
/// # Safety
/// `pat` must point to a valid SearchPattern.
#[no_mangle]
pub unsafe extern "C" fn rs_set_substitute_pattern_shada(pat: *const SearchPatternC) {
    nvim_spat_memcpy_in(state::RE_SUBST, pat);
    nvim_clear_spat_off(state::RE_SUBST);
}

// rs_set_last_used_pattern is already defined in substitute.rs

// =============================================================================
// search_regcomp and Pattern Compilation (Phase 4)
// =============================================================================

/// search_regcomp: translate search pattern for vim_regcomp().
///
/// This is the Rust implementation of `search_regcomp()` from search.c.
/// Returns FAIL (0) or OK (1). Sets `*used_pat_out` to the pattern actually used.
///
/// # Safety
/// All pointer arguments must be valid.
#[no_mangle]
pub unsafe extern "C" fn rs_search_regcomp(
    pat: *mut c_char,
    patlen: usize,
    used_pat_out: *mut *mut c_char,
    pat_save: c_int,
    pat_use: c_int,
    options: c_int,
    regmatch: *mut std::ffi::c_void,
) -> c_int {
    const SEARCH_HIS: c_int = 0x20;
    const SEARCH_KEEP: c_int = 0x400;
    const FAIL: c_int = 0;
    const OK: c_int = 1;
    const RE_SEARCH: c_int = 0;
    const RE_SUBST: c_int = 1;
    const RE_LAST: c_int = 2;
    const RE_BOTH: c_int = 3;

    nvim_clear_rc_did_emsg();

    let magic_isset = crate::magic_isset_impl();
    let mut magic = c_int::from(magic_isset);

    let mut actual_pat = pat;
    let mut actual_patlen = patlen;

    // If no pattern given, use a previously defined pattern.
    if pat.is_null() || (!pat.is_null() && *pat == 0) {
        let i = if pat_use == RE_LAST {
            crate::state::get_last_idx()
        } else {
            pat_use
        };

        if nvim_spats_pat_is_null(i) != 0 {
            // Pattern was never defined
            if pat_use == RE_SUBST {
                nvim_emsg_nopresub();
            } else {
                nvim_emsg_noprevre();
            }
            nvim_set_rc_did_emsg();
            return FAIL;
        }

        let mut spat_magic: c_int = 0;
        let mut spat_no_scs: c_int = 0;
        actual_pat = nvim_spats_get_pat_and_len(
            i,
            &mut actual_patlen,
            &mut spat_magic,
            &mut spat_no_scs,
        ) as *mut c_char;
        magic = spat_magic;
        nvim_set_no_smartcase(spat_no_scs);
    } else if (options & SEARCH_HIS) != 0 {
        // Put new pattern in history
        nvim_search_add_to_history(pat, patlen);
    }

    if !used_pat_out.is_null() {
        *used_pat_out = actual_pat;
    }

    // Set mr_pattern
    nvim_set_mr_pattern(actual_pat, actual_patlen);

    // Save pattern unless SEARCH_KEEP or keeppatterns
    if (options & SEARCH_KEEP) == 0 && nvim_get_cmdmod_keeppatterns() == 0 {
        if pat_save == RE_SEARCH || pat_save == RE_BOTH {
            nvim_save_re_pat_batch(RE_SEARCH, actual_pat, actual_patlen, magic);
        }
        if pat_save == RE_SUBST || pat_save == RE_BOTH {
            nvim_save_re_pat_batch(RE_SUBST, actual_pat, actual_patlen, magic);
        }
    }

    // Compile the pattern
    if nvim_search_regcomp_compile(actual_pat, magic, regmatch) == 0 {
        return FAIL;
    }
    OK
}

/// save_re_pat: update spats[idx] with a new pattern.
///
/// # Safety
/// `pat` must be a valid pointer.
#[no_mangle]
pub unsafe extern "C" fn rs_save_re_pat(
    idx: c_int,
    pat: *const c_char,
    patlen: usize,
    magic: c_int,
) {
    nvim_save_re_pat_batch(idx, pat, patlen, magic);
}

/// set_last_search_pat: Set the last search pattern (for ":let @/ =" and ShaDa).
///
/// # Safety
/// `s` must be a valid null-terminated C string.
#[no_mangle]
pub unsafe extern "C" fn rs_set_last_search_pat(
    s: *const c_char,
    idx: c_int,
    magic: c_int,
    setlast: c_int,
) {
    nvim_set_last_search_pat_batch(s, idx, magic, setlast);
}

/// last_pat_prog: Get a regexp program for the last used search pattern.
///
/// # Safety
/// `regmatch` must be a valid pointer to a regmmatch_T.
#[no_mangle]
pub unsafe extern "C" fn rs_last_pat_prog(regmatch: *mut std::ffi::c_void) {
    let last_idx = crate::state::get_last_idx();
    if nvim_spats_pat_is_null(last_idx) != 0 {
        // Set regmatch->regprog = NULL
        // regprog is the first field of regmmatch_T (it's a pointer)
        let regprog_ptr = regmatch as *mut *mut std::ffi::c_void;
        *regprog_ptr = std::ptr::null_mut();
        return;
    }
    nvim_inc_emsg_off();
    // Call search_regcomp with empty pat, using last_idx pattern
    rs_search_regcomp(
        c"".as_ptr() as *mut c_char,
        0,
        std::ptr::null_mut(),
        0,
        last_idx,
        0x400, // SEARCH_KEEP
        regmatch,
    );
    nvim_dec_emsg_off();
}

/// set_vv_searchforward: Update vim variable.
/// This is already handled by nvim_call_set_vv_searchforward in C.
/// Exposed here for completeness.
#[no_mangle]
pub extern "C" fn rs_set_vv_searchforward() {
    unsafe {
        nvim_call_set_vv_searchforward();
    }
}

// =============================================================================
// Pattern Save/Restore (Phase 3)
// =============================================================================

/// Save search patterns (for autocmds/user functions).
///
/// Uses nesting via save_level: only acts at the top level.
#[no_mangle]
pub extern "C" fn rs_save_search_patterns() {
    unsafe {
        // Increment save_level; only do the actual save if it was 0
        if nvim_inc_save_level() != 0 {
            return;
        }
        nvim_save_search_patterns_batch();
    }
}

/// Restore search patterns (for autocmds/user functions).
///
/// Uses nesting via save_level: only acts when it reaches 0.
#[no_mangle]
pub extern "C" fn rs_restore_search_patterns() {
    unsafe {
        // Decrement save_level; only do the actual restore if it reaches 0
        if nvim_dec_save_level() != 0 {
            return;
        }
        nvim_restore_search_patterns_batch();
    }
}

/// Save last search pattern for incremental search.
///
/// Uses nesting via did_save_last_search_spat.
#[no_mangle]
pub extern "C" fn rs_save_last_search_pattern() {
    unsafe {
        // Increment counter; only save if first call (old value was 0)
        if nvim_inc_did_save() != 0 {
            return;
        }
        nvim_save_last_search_spat_batch();
    }
}

/// Restore last search pattern for incremental search.
///
/// Uses nesting via did_save_last_search_spat.
#[no_mangle]
pub extern "C" fn rs_restore_last_search_pattern() {
    unsafe {
        // Decrement counter
        let new_val = nvim_dec_did_save();
        if new_val > 0 {
            // Nested call, nothing to do
            return;
        }
        if new_val != 0 {
            // Called more often than save
            nvim_call_iemsg_restore_mismatch();
            return;
        }
        nvim_restore_last_search_spat_batch();
    }
}

/// Free spat at given index.
#[no_mangle]
pub extern "C" fn rs_free_spat(idx: c_int) {
    unsafe {
        nvim_free_spat(idx);
    }
}

/// Free all search patterns (for EXITFREE).
#[no_mangle]
pub extern "C" fn rs_free_search_patterns() {
    unsafe {
        nvim_free_spat(state::RE_SEARCH);
        nvim_free_spat(state::RE_SUBST);
        nvim_clear_spats();
        nvim_free_mr_pattern();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pattern_indices() {
        assert_eq!(state::RE_SEARCH, 0);
        assert_eq!(state::RE_SUBST, 1);
    }
}
