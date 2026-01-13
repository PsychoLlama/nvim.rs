//! Tag stack operations for Neovim C-to-Rust migration
//!
//! This module provides Rust implementations for tag stack push, pop, clear,
//! and navigation operations.

#![allow(clippy::cast_possible_wrap)]
#![allow(clippy::cast_lossless)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::missing_const_for_fn)]
#![allow(clippy::missing_safety_doc)]
#![allow(clippy::doc_markdown)]
#![allow(clippy::ptr_cast_constness)]

use std::ffi::{c_char, c_int, c_void};
use std::ptr;

use crate::TAGSTACKSIZE;

// =============================================================================
// Opaque handle types (use const pointers to match lib.rs declarations)
// =============================================================================

/// Opaque handle to win_T (window) - mutable version for stack operations
type WinHandle = *const c_void;

/// Opaque handle to taggy_T (tag stack entry) - mutable version for stack operations
type TaggyHandle = *const c_void;

/// Line number type
type LinenrT = i32;

/// Column number type
type ColnrT = c_int;

// =============================================================================
// External C functions
// =============================================================================

// Note: Using #[allow(clashing_extern_declarations)] because we declare
// these with const pointers but internally the C functions may modify
// the data through the pointer.
#[allow(clashing_extern_declarations)]
extern "C" {
    // Window tag stack accessors (getters use const)
    fn nvim_win_get_tagstacklen(wp: WinHandle) -> c_int;
    fn nvim_win_get_tagstackidx(wp: WinHandle) -> c_int;
    fn nvim_win_get_tagstack_entry(wp: WinHandle, idx: c_int) -> TaggyHandle;

    // Window tag stack setters (need mutable access)
    fn nvim_win_set_tagstacklen(wp: *mut c_void, len: c_int);
    fn nvim_win_set_tagstackidx(wp: *mut c_void, idx: c_int);

    // Taggy accessors (getters use const)
    fn nvim_taggy_get_tagname(tg: TaggyHandle) -> *const c_char;
    fn nvim_taggy_get_cur_match(tg: TaggyHandle) -> c_int;
    fn nvim_taggy_get_cur_fnum(tg: TaggyHandle) -> c_int;
    fn nvim_taggy_get_user_data(tg: TaggyHandle) -> *const c_char;

    // Taggy setters (need mutable access)
    fn nvim_taggy_set_tagname(tg: *mut c_void, name: *mut c_char);
    fn nvim_taggy_set_cur_match(tg: *mut c_void, match_idx: c_int);
    fn nvim_taggy_set_cur_fnum(tg: *mut c_void, fnum: c_int);
    fn nvim_taggy_set_user_data(tg: *mut c_void, data: *mut c_char);

    // Fmark accessors
    fn nvim_taggy_get_fmark_lnum(tg: TaggyHandle) -> LinenrT;
    fn nvim_taggy_get_fmark_col(tg: TaggyHandle) -> ColnrT;
    fn nvim_taggy_get_fmark_fnum(tg: TaggyHandle) -> c_int;
    fn nvim_taggy_set_fmark_lnum(tg: *mut c_void, lnum: LinenrT);
    fn nvim_taggy_set_fmark_col(tg: *mut c_void, col: ColnrT);
    fn nvim_taggy_set_fmark_fnum(tg: *mut c_void, fnum: c_int);

    // Memory functions
    fn xfree(ptr: *mut c_void);
}

// =============================================================================
// Tag stack entry operations
// =============================================================================

/// Clear a single tag stack entry, freeing its allocated memory.
///
/// This frees the tagname and user_data strings, then zeroes the pointers.
#[no_mangle]
pub unsafe extern "C" fn rs_tagstack_clear_entry(tg: TaggyHandle) {
    if tg.is_null() {
        return;
    }

    let tg_mut = tg as *mut c_void;

    let tagname = nvim_taggy_get_tagname(tg);
    if !tagname.is_null() {
        xfree(tagname as *mut c_void);
        nvim_taggy_set_tagname(tg_mut, ptr::null_mut());
    }

    let user_data = nvim_taggy_get_user_data(tg);
    if !user_data.is_null() {
        xfree(user_data as *mut c_void);
        nvim_taggy_set_user_data(tg_mut, ptr::null_mut());
    }
}

/// Copy a tag stack entry from source to destination.
///
/// This does NOT deep-copy the strings - it just copies the pointers.
/// Use with caution - the source entry should be invalidated after copying.
#[no_mangle]
pub unsafe extern "C" fn rs_tagstack_copy_entry(dest: TaggyHandle, src: TaggyHandle) {
    if dest.is_null() || src.is_null() {
        return;
    }

    let dest_mut = dest as *mut c_void;

    nvim_taggy_set_tagname(dest_mut, nvim_taggy_get_tagname(src) as *mut c_char);
    nvim_taggy_set_cur_match(dest_mut, nvim_taggy_get_cur_match(src));
    nvim_taggy_set_cur_fnum(dest_mut, nvim_taggy_get_cur_fnum(src));
    nvim_taggy_set_user_data(dest_mut, nvim_taggy_get_user_data(src) as *mut c_char);
    nvim_taggy_set_fmark_lnum(dest_mut, nvim_taggy_get_fmark_lnum(src));
    nvim_taggy_set_fmark_col(dest_mut, nvim_taggy_get_fmark_col(src));
    nvim_taggy_set_fmark_fnum(dest_mut, nvim_taggy_get_fmark_fnum(src));
}

/// Zero out a tag stack entry (without freeing memory).
#[no_mangle]
pub unsafe extern "C" fn rs_tagstack_zero_entry(tg: TaggyHandle) {
    if tg.is_null() {
        return;
    }

    let tg_mut = tg as *mut c_void;

    nvim_taggy_set_tagname(tg_mut, ptr::null_mut());
    nvim_taggy_set_cur_match(tg_mut, 0);
    nvim_taggy_set_cur_fnum(tg_mut, 0);
    nvim_taggy_set_user_data(tg_mut, ptr::null_mut());
    nvim_taggy_set_fmark_lnum(tg_mut, 0);
    nvim_taggy_set_fmark_col(tg_mut, 0);
    nvim_taggy_set_fmark_fnum(tg_mut, 0);
}

// =============================================================================
// Tag stack operations
// =============================================================================

/// Clear all entries in the window's tag stack.
#[no_mangle]
pub unsafe extern "C" fn rs_tagstack_clear(wp: WinHandle) {
    if wp.is_null() {
        return;
    }

    let wp_mut = wp as *mut c_void;

    let len = nvim_win_get_tagstacklen(wp);
    for i in 0..len {
        let entry = nvim_win_get_tagstack_entry(wp, i);
        rs_tagstack_clear_entry(entry);
    }

    nvim_win_set_tagstacklen(wp_mut, 0);
    nvim_win_set_tagstackidx(wp_mut, 0);
}

/// Shift the tag stack down by one, removing the oldest entry.
///
/// This frees entry[0], shifts all other entries down, and decrements the length.
#[no_mangle]
pub unsafe extern "C" fn rs_tagstack_shift(wp: WinHandle) {
    if wp.is_null() {
        return;
    }

    let wp_mut = wp as *mut c_void;

    let len = nvim_win_get_tagstacklen(wp);
    if len <= 0 {
        return;
    }

    // Clear the oldest entry
    let oldest = nvim_win_get_tagstack_entry(wp, 0);
    rs_tagstack_clear_entry(oldest);

    // Shift all entries down
    for i in 1..len {
        let src = nvim_win_get_tagstack_entry(wp, i);
        let dest = nvim_win_get_tagstack_entry(wp, i - 1);
        rs_tagstack_copy_entry(dest, src);
    }

    // Zero out the now-unused top entry
    let top = nvim_win_get_tagstack_entry(wp, len - 1);
    rs_tagstack_zero_entry(top);

    nvim_win_set_tagstacklen(wp_mut, len - 1);
}

/// Push a new entry onto the tag stack.
///
/// If the stack is full, removes the oldest entry first.
///
/// # Arguments
///
/// * `wp` - Window handle
/// * `tagname` - Tag name (takes ownership)
/// * `cur_fnum` - Buffer number for current match
/// * `cur_match` - Current match index
/// * `mark_lnum` - Cursor line before jump
/// * `mark_col` - Cursor column before jump
/// * `fnum` - File number for the mark
/// * `user_data` - Optional user data (takes ownership)
#[no_mangle]
pub unsafe extern "C" fn rs_tagstack_push(
    wp: WinHandle,
    tagname: *mut c_char,
    cur_fnum: c_int,
    cur_match: c_int,
    mark_lnum: LinenrT,
    mark_col: ColnrT,
    fnum: c_int,
    user_data: *mut c_char,
) {
    if wp.is_null() {
        return;
    }

    let wp_mut = wp as *mut c_void;

    let mut idx = nvim_win_get_tagstacklen(wp);

    // If stack is full, remove oldest entry
    if idx >= TAGSTACKSIZE {
        rs_tagstack_shift(wp);
        idx = TAGSTACKSIZE - 1;
    }

    let entry = nvim_win_get_tagstack_entry(wp, idx);
    if entry.is_null() {
        return;
    }

    let entry_mut = entry as *mut c_void;

    // Set all fields
    nvim_taggy_set_tagname(entry_mut, tagname);
    nvim_taggy_set_cur_fnum(entry_mut, cur_fnum);
    nvim_taggy_set_cur_match(entry_mut, cur_match.max(0));
    nvim_taggy_set_fmark_lnum(entry_mut, mark_lnum);
    nvim_taggy_set_fmark_col(entry_mut, mark_col);
    nvim_taggy_set_fmark_fnum(entry_mut, fnum);
    nvim_taggy_set_user_data(entry_mut, user_data);

    nvim_win_set_tagstacklen(wp_mut, idx + 1);
}

/// Pop an entry from the tag stack (go to older position).
///
/// Returns true if successful, false if already at bottom of stack.
#[no_mangle]
pub unsafe extern "C" fn rs_tagstack_pop(wp: WinHandle) -> bool {
    if wp.is_null() {
        return false;
    }

    let wp_mut = wp as *mut c_void;

    let idx = nvim_win_get_tagstackidx(wp);
    if idx <= 0 {
        return false;
    }

    nvim_win_set_tagstackidx(wp_mut, idx - 1);
    true
}

/// Get the number of entries in the tag stack.
#[no_mangle]
pub unsafe extern "C" fn rs_tagstack_len(wp: WinHandle) -> c_int {
    if wp.is_null() {
        return 0;
    }
    nvim_win_get_tagstacklen(wp)
}

/// Get the current index in the tag stack.
#[no_mangle]
pub unsafe extern "C" fn rs_tagstack_idx(wp: WinHandle) -> c_int {
    if wp.is_null() {
        return 0;
    }
    nvim_win_get_tagstackidx(wp)
}

/// Set the current index in the tag stack.
///
/// The index is clamped to valid range [0, len].
#[no_mangle]
pub unsafe extern "C" fn rs_tagstack_set_idx(wp: WinHandle, idx: c_int) {
    if wp.is_null() {
        return;
    }

    let wp_mut = wp as *mut c_void;

    let len = nvim_win_get_tagstacklen(wp);
    let clamped = idx.max(0).min(len);
    nvim_win_set_tagstackidx(wp_mut, clamped);
}

/// Get a tag stack entry by index.
///
/// Returns null if index is out of range.
#[no_mangle]
pub unsafe extern "C" fn rs_tagstack_get_entry(wp: WinHandle, idx: c_int) -> TaggyHandle {
    if wp.is_null() {
        return ptr::null();
    }

    let len = nvim_win_get_tagstacklen(wp);
    if idx < 0 || idx >= len {
        return ptr::null();
    }

    nvim_win_get_tagstack_entry(wp, idx)
}

/// Get the current tag stack entry (at w_tagstackidx - 1).
///
/// Returns null if the stack is empty or at the bottom.
#[no_mangle]
pub unsafe extern "C" fn rs_tagstack_current_entry(wp: WinHandle) -> TaggyHandle {
    if wp.is_null() {
        return ptr::null();
    }

    let idx = nvim_win_get_tagstackidx(wp);
    if idx <= 0 {
        return ptr::null();
    }

    nvim_win_get_tagstack_entry(wp, idx - 1)
}

/// Truncate the tag stack at the current index.
///
/// Removes all entries above the current position.
#[no_mangle]
pub unsafe extern "C" fn rs_tagstack_truncate(wp: WinHandle) {
    if wp.is_null() {
        return;
    }

    let wp_mut = wp as *mut c_void;

    let idx = nvim_win_get_tagstackidx(wp);
    let mut len = nvim_win_get_tagstacklen(wp);

    // Clear all entries above current index
    while idx < len {
        len -= 1;
        let entry = nvim_win_get_tagstack_entry(wp, len);
        rs_tagstack_clear_entry(entry);
    }

    nvim_win_set_tagstacklen(wp_mut, idx);
}

// =============================================================================
// Tag stack navigation helpers
// =============================================================================

/// Check if we can go to an older tag (pop).
#[no_mangle]
pub unsafe extern "C" fn rs_tagstack_can_pop(wp: WinHandle) -> bool {
    if wp.is_null() {
        return false;
    }
    nvim_win_get_tagstackidx(wp) > 0
}

/// Check if we can go to a newer tag (push direction).
#[no_mangle]
pub unsafe extern "C" fn rs_tagstack_can_push_nav(wp: WinHandle) -> bool {
    if wp.is_null() {
        return false;
    }
    nvim_win_get_tagstackidx(wp) < nvim_win_get_tagstacklen(wp)
}

/// Check if the tag stack is empty.
#[no_mangle]
pub unsafe extern "C" fn rs_tagstack_empty(wp: WinHandle) -> bool {
    if wp.is_null() {
        return true;
    }
    nvim_win_get_tagstacklen(wp) <= 0
}

/// Check if the tag stack is full.
#[no_mangle]
pub unsafe extern "C" fn rs_tagstack_full(wp: WinHandle) -> bool {
    if wp.is_null() {
        return true;
    }
    nvim_win_get_tagstacklen(wp) >= TAGSTACKSIZE
}

/// Navigate to an older tag (decrement index).
///
/// Returns the new index, or -1 on failure.
#[no_mangle]
pub unsafe extern "C" fn rs_tagstack_go_older(wp: WinHandle, count: c_int) -> c_int {
    if wp.is_null() {
        return -1;
    }

    let wp_mut = wp as *mut c_void;

    let idx = nvim_win_get_tagstackidx(wp);
    let new_idx = (idx - count).max(0);
    nvim_win_set_tagstackidx(wp_mut, new_idx);
    new_idx
}

/// Navigate to a newer tag (increment index).
///
/// Returns the new index, or -1 on failure.
#[no_mangle]
pub unsafe extern "C" fn rs_tagstack_go_newer(wp: WinHandle, count: c_int) -> c_int {
    if wp.is_null() {
        return -1;
    }

    let wp_mut = wp as *mut c_void;

    let idx = nvim_win_get_tagstackidx(wp);
    let len = nvim_win_get_tagstacklen(wp);
    let new_idx = (idx + count).min(len);
    nvim_win_set_tagstackidx(wp_mut, new_idx);
    new_idx
}

// =============================================================================
// Tag stack info accessors
// =============================================================================

/// Get the tag name from a stack entry.
#[no_mangle]
pub unsafe extern "C" fn rs_tagstack_entry_tagname(tg: TaggyHandle) -> *const c_char {
    if tg.is_null() {
        return ptr::null();
    }
    nvim_taggy_get_tagname(tg)
}

/// Get the match index from a stack entry.
#[no_mangle]
pub unsafe extern "C" fn rs_tagstack_entry_match(tg: TaggyHandle) -> c_int {
    if tg.is_null() {
        return 0;
    }
    nvim_taggy_get_cur_match(tg)
}

/// Get the buffer number from a stack entry.
#[no_mangle]
pub unsafe extern "C" fn rs_tagstack_entry_fnum(tg: TaggyHandle) -> c_int {
    if tg.is_null() {
        return 0;
    }
    nvim_taggy_get_cur_fnum(tg)
}

/// Get the user data from a stack entry.
#[no_mangle]
pub unsafe extern "C" fn rs_tagstack_entry_user_data(tg: TaggyHandle) -> *const c_char {
    if tg.is_null() {
        return ptr::null();
    }
    nvim_taggy_get_user_data(tg)
}

/// Get the mark line number from a stack entry.
#[no_mangle]
pub unsafe extern "C" fn rs_tagstack_entry_mark_lnum(tg: TaggyHandle) -> LinenrT {
    if tg.is_null() {
        return 0;
    }
    nvim_taggy_get_fmark_lnum(tg)
}

/// Get the mark column from a stack entry.
#[no_mangle]
pub unsafe extern "C" fn rs_tagstack_entry_mark_col(tg: TaggyHandle) -> ColnrT {
    if tg.is_null() {
        return 0;
    }
    nvim_taggy_get_fmark_col(tg)
}

/// Get the mark file number from a stack entry.
#[no_mangle]
pub unsafe extern "C" fn rs_tagstack_entry_mark_fnum(tg: TaggyHandle) -> c_int {
    if tg.is_null() {
        return 0;
    }
    nvim_taggy_get_fmark_fnum(tg)
}

/// Check if a stack entry has a valid tag name.
#[no_mangle]
pub unsafe extern "C" fn rs_tagstack_entry_has_name(tg: TaggyHandle) -> bool {
    if tg.is_null() {
        return false;
    }
    !nvim_taggy_get_tagname(tg).is_null()
}

// =============================================================================
// Phase 152: Additional Tag Stack Management FFI Exports
// =============================================================================

/// Get the TAGSTACKSIZE constant.
#[no_mangle]
pub extern "C" fn rs_tagstacksize() -> c_int {
    TAGSTACKSIZE
}

/// Check if a stack entry has user data.
#[no_mangle]
pub unsafe extern "C" fn rs_tagstack_entry_has_user_data(tg: TaggyHandle) -> bool {
    if tg.is_null() {
        return false;
    }
    !nvim_taggy_get_user_data(tg).is_null()
}

/// Check if a stack entry has a valid mark (line > 0).
#[no_mangle]
pub unsafe extern "C" fn rs_tagstack_entry_has_mark(tg: TaggyHandle) -> bool {
    if tg.is_null() {
        return false;
    }
    nvim_taggy_get_fmark_lnum(tg) > 0
}

/// Set the tag name for a stack entry.
#[no_mangle]
pub unsafe extern "C" fn rs_tagstack_entry_set_tagname(tg: TaggyHandle, name: *mut c_char) {
    if tg.is_null() {
        return;
    }
    nvim_taggy_set_tagname(tg as *mut c_void, name);
}

/// Set the match index for a stack entry.
#[no_mangle]
pub unsafe extern "C" fn rs_tagstack_entry_set_match(tg: TaggyHandle, match_idx: c_int) {
    if tg.is_null() {
        return;
    }
    nvim_taggy_set_cur_match(tg as *mut c_void, match_idx);
}

/// Set the buffer number for a stack entry.
#[no_mangle]
pub unsafe extern "C" fn rs_tagstack_entry_set_fnum(tg: TaggyHandle, fnum: c_int) {
    if tg.is_null() {
        return;
    }
    nvim_taggy_set_cur_fnum(tg as *mut c_void, fnum);
}

/// Set the user data for a stack entry.
#[no_mangle]
pub unsafe extern "C" fn rs_tagstack_entry_set_user_data(tg: TaggyHandle, data: *mut c_char) {
    if tg.is_null() {
        return;
    }
    nvim_taggy_set_user_data(tg as *mut c_void, data);
}

/// Set the mark line number for a stack entry.
#[no_mangle]
pub unsafe extern "C" fn rs_tagstack_entry_set_mark_lnum(tg: TaggyHandle, lnum: LinenrT) {
    if tg.is_null() {
        return;
    }
    nvim_taggy_set_fmark_lnum(tg as *mut c_void, lnum);
}

/// Set the mark column for a stack entry.
#[no_mangle]
pub unsafe extern "C" fn rs_tagstack_entry_set_mark_col(tg: TaggyHandle, col: ColnrT) {
    if tg.is_null() {
        return;
    }
    nvim_taggy_set_fmark_col(tg as *mut c_void, col);
}

/// Set the mark file number for a stack entry.
#[no_mangle]
pub unsafe extern "C" fn rs_tagstack_entry_set_mark_fnum(tg: TaggyHandle, fnum: c_int) {
    if tg.is_null() {
        return;
    }
    nvim_taggy_set_fmark_fnum(tg as *mut c_void, fnum);
}

/// Get the number of positions we can go older.
#[no_mangle]
pub unsafe extern "C" fn rs_tagstack_older_count(wp: WinHandle) -> c_int {
    if wp.is_null() {
        return 0;
    }
    nvim_win_get_tagstackidx(wp)
}

/// Get the number of positions we can go newer.
#[no_mangle]
pub unsafe extern "C" fn rs_tagstack_newer_count(wp: WinHandle) -> c_int {
    if wp.is_null() {
        return 0;
    }
    let idx = nvim_win_get_tagstackidx(wp);
    let len = nvim_win_get_tagstacklen(wp);
    len - idx
}

/// Check if the stack is at a specific index.
#[no_mangle]
pub unsafe extern "C" fn rs_tagstack_at_idx(wp: WinHandle, idx: c_int) -> bool {
    if wp.is_null() {
        return false;
    }
    nvim_win_get_tagstackidx(wp) == idx
}

/// Get remaining capacity in the tag stack.
#[no_mangle]
pub unsafe extern "C" fn rs_tagstack_remaining(wp: WinHandle) -> c_int {
    if wp.is_null() {
        return 0;
    }
    TAGSTACKSIZE - nvim_win_get_tagstacklen(wp)
}

/// Set both length and index atomically.
#[no_mangle]
pub unsafe extern "C" fn rs_tagstack_set_len_idx(wp: WinHandle, len: c_int, idx: c_int) {
    if wp.is_null() {
        return;
    }
    let wp_mut = wp as *mut c_void;
    let clamped_len = len.clamp(0, TAGSTACKSIZE);
    let clamped_idx = idx.clamp(0, clamped_len);
    nvim_win_set_tagstacklen(wp_mut, clamped_len);
    nvim_win_set_tagstackidx(wp_mut, clamped_idx);
}

/// Increment the current match index in a stack entry.
#[no_mangle]
pub unsafe extern "C" fn rs_tagstack_entry_inc_match(tg: TaggyHandle) -> c_int {
    if tg.is_null() {
        return 0;
    }
    let cur = nvim_taggy_get_cur_match(tg);
    let new = cur + 1;
    nvim_taggy_set_cur_match(tg as *mut c_void, new);
    new
}

/// Decrement the current match index in a stack entry.
#[no_mangle]
pub unsafe extern "C" fn rs_tagstack_entry_dec_match(tg: TaggyHandle) -> c_int {
    if tg.is_null() {
        return 0;
    }
    let cur = nvim_taggy_get_cur_match(tg);
    let new = (cur - 1).max(0);
    nvim_taggy_set_cur_match(tg as *mut c_void, new);
    new
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_null_safety() {
        unsafe {
            // Entry operations
            rs_tagstack_clear_entry(ptr::null_mut());
            rs_tagstack_copy_entry(ptr::null_mut(), ptr::null_mut());
            rs_tagstack_zero_entry(ptr::null_mut());

            // Stack operations
            rs_tagstack_clear(ptr::null_mut());
            rs_tagstack_shift(ptr::null_mut());
            rs_tagstack_push(
                ptr::null_mut(),
                ptr::null_mut(),
                0,
                0,
                0,
                0,
                0,
                ptr::null_mut(),
            );
            assert!(!rs_tagstack_pop(ptr::null_mut()));

            // Getters
            assert_eq!(rs_tagstack_len(ptr::null_mut()), 0);
            assert_eq!(rs_tagstack_idx(ptr::null_mut()), 0);
            rs_tagstack_set_idx(ptr::null_mut(), 5);
            assert!(rs_tagstack_get_entry(ptr::null_mut(), 0).is_null());
            assert!(rs_tagstack_current_entry(ptr::null_mut()).is_null());

            // Navigation
            assert!(!rs_tagstack_can_pop(ptr::null_mut()));
            assert!(!rs_tagstack_can_push_nav(ptr::null_mut()));
            assert!(rs_tagstack_empty(ptr::null_mut()));
            assert!(rs_tagstack_full(ptr::null_mut()));
            assert_eq!(rs_tagstack_go_older(ptr::null_mut(), 1), -1);
            assert_eq!(rs_tagstack_go_newer(ptr::null_mut(), 1), -1);

            // Entry accessors
            assert!(rs_tagstack_entry_tagname(ptr::null_mut()).is_null());
            assert_eq!(rs_tagstack_entry_match(ptr::null_mut()), 0);
            assert_eq!(rs_tagstack_entry_fnum(ptr::null_mut()), 0);
            assert!(rs_tagstack_entry_user_data(ptr::null_mut()).is_null());
            assert_eq!(rs_tagstack_entry_mark_lnum(ptr::null_mut()), 0);
            assert_eq!(rs_tagstack_entry_mark_col(ptr::null_mut()), 0);
            assert_eq!(rs_tagstack_entry_mark_fnum(ptr::null_mut()), 0);
            assert!(!rs_tagstack_entry_has_name(ptr::null_mut()));
        }
    }

    #[test]
    fn test_tagstacksize_constant() {
        assert_eq!(TAGSTACKSIZE, 20);
    }
}
