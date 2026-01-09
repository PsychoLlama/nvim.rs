//! Keyword handling for syntax highlighting.
//!
//! This module handles:
//! - Keyword hash table management
//! - Keyword matching functions
//! - Case-sensitive/insensitive handling
//! - Keyword entry accessors

use std::ffi::{c_char, c_int, c_void};

use crate::types::{
    IdListHandle, KeyEntryHandle, StateItemHandle, SynBlockHandle, WinHandle, HL_CONTAINED,
    MAXKEYWLEN,
};

// =============================================================================
// FFI declarations for keyword operations
// =============================================================================

extern "C" {
    // Keyword entry accessors
    fn nvim_keyentry_get_next(ke: KeyEntryHandle) -> KeyEntryHandle;
    fn nvim_keyentry_get_syn_id(ke: KeyEntryHandle) -> i16;
    fn nvim_keyentry_get_syn_inc_tag(ke: KeyEntryHandle) -> c_int;
    fn nvim_keyentry_get_flags(ke: KeyEntryHandle) -> c_int;
    fn nvim_keyentry_get_char(ke: KeyEntryHandle) -> c_int;
    fn nvim_keyentry_get_keyword(ke: KeyEntryHandle) -> *const c_char;
    fn nvim_keyentry_get_next_list(ke: KeyEntryHandle) -> IdListHandle;
    fn nvim_keyentry_get_cont_in_list(ke: KeyEntryHandle) -> IdListHandle;
    fn nvim_keyentry_has_next_list(ke: KeyEntryHandle) -> c_int;
    fn nvim_keyentry_has_cont_in_list(ke: KeyEntryHandle) -> c_int;

    // Keyword matching functions
    fn nvim_syn_keyword_find(keyword: *mut c_char, use_ic: c_int) -> KeyEntryHandle;
    fn nvim_syn_match_keyword(
        keyword: *mut c_char,
        use_ic: c_int,
        cur_si: StateItemHandle,
    ) -> KeyEntryHandle;
    fn nvim_syn_keyword_foldcase(src: *mut c_char, srclen: c_int, dst: *mut c_char, dstlen: c_int);
    fn nvim_syn_utfc_ptr2len(p: *mut c_char) -> c_int;
    fn nvim_syn_get_maxkeywlen() -> c_int;

    // Keyword table accessors
    fn nvim_syn_has_keywords() -> c_int;
    fn nvim_syn_has_keywords_ic() -> c_int;
    fn nvim_synblock_has_keywords(block: SynBlockHandle) -> c_int;
    fn nvim_synblock_has_keywords_ic(block: SynBlockHandle) -> c_int;
    fn nvim_synblock_keywtab_count(block: SynBlockHandle) -> usize;
    fn nvim_synblock_keywtab_ic_count(block: SynBlockHandle) -> usize;
    fn nvim_synblock_get_keywtab(block: SynBlockHandle) -> *mut c_void;
    fn nvim_synblock_get_keywtab_ic(block: SynBlockHandle) -> *mut c_void;

    // Window-level keyword accessors
    fn nvim_win_get_keywtab_used(win: WinHandle) -> c_int;
    fn nvim_win_get_keywtab_ic_used(win: WinHandle) -> c_int;
}

// =============================================================================
// Keyword entry accessors
// =============================================================================

/// Get the next keyword entry in the hash collision chain.
#[must_use]
pub fn keyentry_next(ke: KeyEntryHandle) -> KeyEntryHandle {
    if ke.is_null() {
        return KeyEntryHandle::null();
    }
    unsafe { nvim_keyentry_get_next(ke) }
}

/// Get the syntax ID (highlight group ID) for a keyword entry.
#[must_use]
pub fn keyentry_syn_id(ke: KeyEntryHandle) -> i16 {
    if ke.is_null() {
        return 0;
    }
    unsafe { nvim_keyentry_get_syn_id(ke) }
}

/// Get the include tag for a keyword entry.
#[must_use]
pub fn keyentry_inc_tag(ke: KeyEntryHandle) -> i32 {
    if ke.is_null() {
        return 0;
    }
    unsafe { nvim_keyentry_get_syn_inc_tag(ke) }
}

/// Get the flags for a keyword entry.
#[must_use]
pub fn keyentry_flags(ke: KeyEntryHandle) -> i32 {
    if ke.is_null() {
        return 0;
    }
    unsafe { nvim_keyentry_get_flags(ke) }
}

/// Get the conceal character for a keyword entry.
#[must_use]
pub fn keyentry_cchar(ke: KeyEntryHandle) -> i32 {
    if ke.is_null() {
        return 0;
    }
    unsafe { nvim_keyentry_get_char(ke) }
}

/// Get the keyword string for a keyword entry.
/// Returns a raw pointer that should not be freed.
#[must_use]
pub fn keyentry_keyword_ptr(ke: KeyEntryHandle) -> *const c_char {
    if ke.is_null() {
        return std::ptr::null();
    }
    unsafe { nvim_keyentry_get_keyword(ke) }
}

/// Get the next_list for a keyword entry.
#[must_use]
pub fn keyentry_next_list(ke: KeyEntryHandle) -> IdListHandle {
    if ke.is_null() {
        return IdListHandle::null();
    }
    unsafe { nvim_keyentry_get_next_list(ke) }
}

/// Get the cont_in_list for a keyword entry.
#[must_use]
pub fn keyentry_cont_in_list(ke: KeyEntryHandle) -> IdListHandle {
    if ke.is_null() {
        return IdListHandle::null();
    }
    unsafe { nvim_keyentry_get_cont_in_list(ke) }
}

/// Check if a keyword entry has a next_list.
#[must_use]
pub fn keyentry_has_next_list(ke: KeyEntryHandle) -> bool {
    if ke.is_null() {
        return false;
    }
    unsafe { nvim_keyentry_has_next_list(ke) != 0 }
}

/// Check if a keyword entry has a cont_in_list.
#[must_use]
pub fn keyentry_has_cont_in_list(ke: KeyEntryHandle) -> bool {
    if ke.is_null() {
        return false;
    }
    unsafe { nvim_keyentry_has_cont_in_list(ke) != 0 }
}

/// Check if a keyword entry is contained (not top-level).
#[must_use]
pub fn keyentry_is_contained(ke: KeyEntryHandle) -> bool {
    if ke.is_null() {
        return false;
    }
    let flags = unsafe { nvim_keyentry_get_flags(ke) };
    (flags & HL_CONTAINED) != 0
}

// =============================================================================
// Keyword iteration
// =============================================================================

/// Iterator over keyword entries in a hash collision chain.
pub struct KeywordIter {
    current: KeyEntryHandle,
}

impl KeywordIter {
    /// Create a new iterator starting at the given keyword entry.
    #[must_use]
    pub fn new(start: KeyEntryHandle) -> Self {
        Self { current: start }
    }
}

impl Iterator for KeywordIter {
    type Item = KeyEntryHandle;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current.is_null() {
            return None;
        }
        let result = self.current;
        self.current = keyentry_next(self.current);
        Some(result)
    }
}

// =============================================================================
// Keyword matching
// =============================================================================

/// Find a keyword in the hash table.
///
/// # Arguments
/// * `keyword` - The keyword to search for (C string pointer)
/// * `use_ic` - If true, use case-insensitive matching
///
/// # Safety
/// The keyword pointer must be valid and null-terminated.
#[must_use]
pub unsafe fn keyword_find(keyword: *mut c_char, use_ic: bool) -> KeyEntryHandle {
    nvim_syn_keyword_find(keyword, if use_ic { 1 } else { 0 })
}

/// Match a keyword considering the current syntax state.
/// This respects contained/containedin lists.
///
/// # Arguments
/// * `keyword` - The keyword to match (C string pointer)
/// * `use_ic` - If true, use case-insensitive matching
/// * `cur_si` - Current state item (may be null)
///
/// # Safety
/// The keyword pointer must be valid and null-terminated.
#[must_use]
pub unsafe fn match_keyword(
    keyword: *mut c_char,
    use_ic: bool,
    cur_si: StateItemHandle,
) -> KeyEntryHandle {
    nvim_syn_match_keyword(keyword, if use_ic { 1 } else { 0 }, cur_si)
}

/// Fold case for keyword comparison.
///
/// # Safety
/// Both src and dst must be valid pointers with sufficient space.
pub unsafe fn keyword_foldcase(src: *mut c_char, srclen: i32, dst: *mut c_char, dstlen: i32) {
    nvim_syn_keyword_foldcase(src, srclen, dst, dstlen);
}

/// Get the length of a UTF-8 character at the given position.
///
/// # Safety
/// The pointer must be valid.
#[must_use]
pub unsafe fn utfc_ptr2len(p: *mut c_char) -> i32 {
    nvim_syn_utfc_ptr2len(p)
}

/// Get the maximum keyword length constant.
#[must_use]
pub fn max_keyword_len() -> i32 {
    unsafe { nvim_syn_get_maxkeywlen() }
}

/// The maximum keyword length (constant).
pub const MAX_KEYWORD_LEN: i32 = MAXKEYWLEN;

// =============================================================================
// Keyword table queries
// =============================================================================

/// Check if the current synblock has any case-sensitive keywords.
#[must_use]
pub fn has_keywords() -> bool {
    unsafe { nvim_syn_has_keywords() != 0 }
}

/// Check if the current synblock has any case-insensitive keywords.
#[must_use]
pub fn has_keywords_ic() -> bool {
    unsafe { nvim_syn_has_keywords_ic() != 0 }
}

/// Check if a synblock has any case-sensitive keywords.
#[must_use]
pub fn synblock_has_keywords(block: SynBlockHandle) -> bool {
    if block.is_null() {
        return false;
    }
    unsafe { nvim_synblock_has_keywords(block) != 0 }
}

/// Check if a synblock has any case-insensitive keywords.
#[must_use]
pub fn synblock_has_keywords_ic(block: SynBlockHandle) -> bool {
    if block.is_null() {
        return false;
    }
    unsafe { nvim_synblock_has_keywords_ic(block) != 0 }
}

/// Get the count of case-sensitive keywords in a synblock.
#[must_use]
pub fn synblock_keyword_count(block: SynBlockHandle) -> usize {
    if block.is_null() {
        return 0;
    }
    unsafe { nvim_synblock_keywtab_count(block) }
}

/// Get the count of case-insensitive keywords in a synblock.
#[must_use]
pub fn synblock_keyword_ic_count(block: SynBlockHandle) -> usize {
    if block.is_null() {
        return 0;
    }
    unsafe { nvim_synblock_keywtab_ic_count(block) }
}

/// Get the case-sensitive keyword hash table for a synblock.
/// Returns a raw pointer to the hashtab_T.
#[must_use]
pub fn synblock_keywtab(block: SynBlockHandle) -> *mut c_void {
    if block.is_null() {
        return std::ptr::null_mut();
    }
    unsafe { nvim_synblock_get_keywtab(block) }
}

/// Get the case-insensitive keyword hash table for a synblock.
/// Returns a raw pointer to the hashtab_T.
#[must_use]
pub fn synblock_keywtab_ic(block: SynBlockHandle) -> *mut c_void {
    if block.is_null() {
        return std::ptr::null_mut();
    }
    unsafe { nvim_synblock_get_keywtab_ic(block) }
}

// =============================================================================
// Window-level keyword queries
// =============================================================================

/// Get the count of case-sensitive keywords used in a window's synblock.
#[must_use]
pub fn win_keyword_count(win: WinHandle) -> i32 {
    if win.is_null() {
        return 0;
    }
    unsafe { nvim_win_get_keywtab_used(win) }
}

/// Get the count of case-insensitive keywords used in a window's synblock.
#[must_use]
pub fn win_keyword_ic_count(win: WinHandle) -> i32 {
    if win.is_null() {
        return 0;
    }
    unsafe { nvim_win_get_keywtab_ic_used(win) }
}

// =============================================================================
// High-level keyword matching result
// =============================================================================

/// Result of a keyword match operation.
#[derive(Debug, Clone, Copy)]
pub struct KeywordMatch {
    /// The syntax ID (highlight group) of the matched keyword.
    pub syn_id: i16,
    /// Flags for the keyword.
    pub flags: i32,
    /// The conceal character (0 if none).
    pub cchar: i32,
    /// The next_list for this keyword (may be null).
    pub next_list: IdListHandle,
}

impl KeywordMatch {
    /// Create a keyword match result from a keyword entry.
    #[must_use]
    pub fn from_entry(ke: KeyEntryHandle) -> Option<Self> {
        if ke.is_null() {
            return None;
        }
        Some(Self {
            syn_id: keyentry_syn_id(ke),
            flags: keyentry_flags(ke),
            cchar: keyentry_cchar(ke),
            next_list: keyentry_next_list(ke),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_null_handle_checks() {
        // Test null handle creation and checking
        // Note: We can only test is_null(), not functions that call extern FFI
        let null_ke = KeyEntryHandle::null();
        assert!(null_ke.is_null());

        let null_block = SynBlockHandle(std::ptr::null_mut());
        assert!(null_block.is_null());

        let null_win = WinHandle(std::ptr::null_mut());
        assert!(null_win.is_null());

        // Non-null handle creation (for testing purposes)
        let fake_ptr = std::ptr::dangling_mut::<std::ffi::c_void>();
        let non_null_ke = KeyEntryHandle(fake_ptr);
        assert!(!non_null_ke.is_null());
    }

    #[test]
    fn test_max_keyword_len_constant() {
        // Test the constant value matches
        assert_eq!(MAX_KEYWORD_LEN, MAXKEYWLEN);
    }
}
