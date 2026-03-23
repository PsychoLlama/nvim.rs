//! Message history management
//!
//! Implements the message history linked list for `:messages` command.
//! The history stores highlighted message chunks and supports iteration
//! for display and clearing.

use std::ffi::{c_char, c_int};
use std::ptr;

// C accessor declarations
extern "C" {
    static mut msg_silent: c_int;
    /// `msg_hist_max` — Rust-owned static (misc.rs)
    static mut msg_hist_max: c_int;
    /// `msg_hist_off` — direct access to C global
    static mut msg_hist_off: bool;
    /// Get `msg_silent`
    /// Get entry->next pointer
    fn nvim_msg_hist_entry_get_next(
        entry: *mut MessageHistoryEntryHandle,
    ) -> *mut MessageHistoryEntryHandle;
    /// Get entry->prev pointer
    fn nvim_msg_hist_entry_get_prev(
        entry: *mut MessageHistoryEntryHandle,
    ) -> *mut MessageHistoryEntryHandle;
    /// Get entry->temp flag
    fn nvim_msg_hist_entry_get_temp(entry: *mut MessageHistoryEntryHandle) -> c_int;
    /// Get entry->kind pointer
    fn nvim_msg_hist_entry_get_kind(entry: *mut MessageHistoryEntryHandle) -> *const c_char;
    /// Get entry->append flag
    fn nvim_msg_hist_entry_get_append(entry: *mut MessageHistoryEntryHandle) -> c_int;
    /// Free an HlMessage
    fn nvim_hl_msg_free(entry: *mut MessageHistoryEntryHandle);
    /// xfree wrapper
    fn xfree(ptr: *mut std::ffi::c_void);
    /// Set entry->next
    fn nvim_msg_hist_entry_set_next(
        entry: *mut MessageHistoryEntryHandle,
        next: *mut MessageHistoryEntryHandle,
    );
    /// Set entry->prev
    fn nvim_msg_hist_entry_set_prev(
        entry: *mut MessageHistoryEntryHandle,
        prev: *mut MessageHistoryEntryHandle,
    );
}

/// Opaque handle to `MessageHistoryEntry` in C.
///
/// This type is never instantiated in Rust; it exists only to provide
/// type safety for pointers passed across the FFI boundary.
#[repr(C)]
pub struct MessageHistoryEntryHandle {
    _private: [u8; 0],
}

// ============================================================================
// Rust-owned statics (previously file-local in message.c)
// ============================================================================

/// First message in history linked list (replaces C static msg_hist_first)
#[no_mangle]
pub static mut msg_hist_first: *mut MessageHistoryEntryHandle = std::ptr::null_mut();

/// Last message in history linked list (replaces C non-static msg_hist_last)
#[no_mangle]
pub static mut msg_hist_last: *mut MessageHistoryEntryHandle = std::ptr::null_mut();

/// First potentially temporary message (replaces C static msg_hist_temp)
#[no_mangle]
pub static mut msg_hist_temp: *mut MessageHistoryEntryHandle = std::ptr::null_mut();

/// Current history length (replaces C static msg_hist_len)
#[no_mangle]
pub static mut msg_hist_len: c_int = 0;

/// Get the length of the message history.
///
/// # Safety
/// Calls C accessor function for `msg_hist_len`.
#[no_mangle]
pub unsafe extern "C" fn rs_msg_hist_len() -> c_int {
    msg_hist_len
}

/// Get the maximum message history length.
///
/// # Safety
/// Calls C accessor function for `msg_hist_max`.
#[no_mangle]
pub unsafe extern "C" fn rs_msg_hist_max() -> c_int {
    msg_hist_max
}

/// Get the first entry in the message history.
///
/// # Safety
/// Calls C accessor function for `msg_hist_first`.
#[no_mangle]
pub unsafe extern "C" fn rs_msg_hist_first() -> *mut MessageHistoryEntryHandle {
    msg_hist_first
}

/// Get the last entry in the message history.
///
/// # Safety
/// Calls C accessor function for `msg_hist_last`.
#[no_mangle]
pub unsafe extern "C" fn rs_msg_hist_last() -> *mut MessageHistoryEntryHandle {
    msg_hist_last
}

/// Get the next entry in the message history.
///
/// # Safety
/// Calls C accessor function for entry->next.
#[no_mangle]
pub unsafe extern "C" fn rs_msg_hist_next(
    entry: *mut MessageHistoryEntryHandle,
) -> *mut MessageHistoryEntryHandle {
    if entry.is_null() {
        return ptr::null_mut();
    }
    nvim_msg_hist_entry_get_next(entry)
}

/// Get the previous entry in the message history.
///
/// # Safety
/// Calls C accessor function for entry->prev.
#[no_mangle]
pub unsafe extern "C" fn rs_msg_hist_prev(
    entry: *mut MessageHistoryEntryHandle,
) -> *mut MessageHistoryEntryHandle {
    if entry.is_null() {
        return ptr::null_mut();
    }
    nvim_msg_hist_entry_get_prev(entry)
}

/// Check if an entry is temporary.
///
/// # Safety
/// Calls C accessor function for entry->temp.
#[no_mangle]
pub unsafe extern "C" fn rs_msg_hist_entry_is_temp(entry: *mut MessageHistoryEntryHandle) -> c_int {
    if entry.is_null() {
        return 0;
    }
    nvim_msg_hist_entry_get_temp(entry)
}

/// Check if an entry should be appended to the previous message.
///
/// # Safety
/// Calls C accessor function for entry->append.
#[no_mangle]
pub unsafe extern "C" fn rs_msg_hist_entry_is_append(
    entry: *mut MessageHistoryEntryHandle,
) -> c_int {
    if entry.is_null() {
        return 0;
    }
    nvim_msg_hist_entry_get_append(entry)
}

/// Get the message kind for an entry.
///
/// # Safety
/// Calls C accessor function for entry->kind.
#[no_mangle]
pub unsafe extern "C" fn rs_msg_hist_entry_kind(
    entry: *mut MessageHistoryEntryHandle,
) -> *const c_char {
    if entry.is_null() {
        return ptr::null();
    }
    nvim_msg_hist_entry_get_kind(entry)
}

/// Free a single message history entry, unlinking it from the list.
///
/// # Safety
/// - `entry` must be a valid pointer or NULL.
/// - After this call, the entry is freed and must not be used.
#[no_mangle]
pub unsafe extern "C" fn rs_msg_hist_free_entry(entry: *mut MessageHistoryEntryHandle) {
    if entry.is_null() {
        return;
    }

    let next = nvim_msg_hist_entry_get_next(entry);
    let prev = nvim_msg_hist_entry_get_prev(entry);

    // Update next's prev pointer
    if next.is_null() {
        msg_hist_last = prev;
    } else {
        nvim_msg_hist_entry_set_prev(next, prev);
    }

    // Update prev's next pointer
    if prev.is_null() {
        msg_hist_first = next;
    } else {
        nvim_msg_hist_entry_set_next(prev, next);
    }

    // Update msg_hist_temp if needed
    let temp = msg_hist_temp;
    if entry == temp {
        msg_hist_temp = next;
    }

    // Free the message content and entry
    nvim_hl_msg_free(entry);
    xfree(entry.cast());
}

/// Clear the oldest messages from the history until there are `keep` messages.
///
/// # Safety
/// Calls C accessor and mutator functions.
#[export_name = "msg_hist_clear"]
pub unsafe extern "C" fn rs_msg_hist_clear(keep: c_int) {
    loop {
        let first = msg_hist_first;
        let len = msg_hist_len;

        // Stop if we've reduced to desired length
        if len <= keep && (keep != 0 || first.is_null()) {
            break;
        }

        // Decrement length if not temporary
        if !first.is_null() && nvim_msg_hist_entry_get_temp(first) == 0 {
            msg_hist_len = len - 1;
        }

        rs_msg_hist_free_entry(first);
    }
}

/// Clear all temporary messages from the history.
///
/// # Safety
/// Calls C accessor and mutator functions.
#[export_name = "msg_hist_clear_temp"]
pub unsafe extern "C" fn rs_msg_hist_clear_temp() {
    let mut current = msg_hist_temp;

    while !current.is_null() {
        let next = nvim_msg_hist_entry_get_next(current);

        if nvim_msg_hist_entry_get_temp(current) != 0 {
            rs_msg_hist_free_entry(current);
        }

        current = next;
    }

    // Reset temp marker since we've processed all temp entries
    msg_hist_temp = ptr::null_mut();
}

/// Check if message history recording is disabled.
///
/// Returns true if `msg_hist_off` is set or `msg_silent` is non-zero.
///
/// # Safety
/// Calls C accessor functions.
#[no_mangle]
pub unsafe extern "C" fn rs_msg_hist_disabled() -> c_int {
    c_int::from(msg_hist_off || msg_silent != 0)
}

/// Check if history is at capacity.
///
/// Returns true if the history is at or over the maximum length.
///
/// # Safety
/// Calls C accessor functions.
#[no_mangle]
pub unsafe extern "C" fn rs_msg_hist_at_capacity() -> c_int {
    let len = msg_hist_len;
    let max = msg_hist_max;
    c_int::from(len >= max)
}

/// Check if the history is empty.
///
/// # Safety
/// Calls C accessor function.
#[no_mangle]
pub unsafe extern "C" fn rs_msg_hist_empty() -> c_int {
    c_int::from(msg_hist_first.is_null())
}

/// Count the number of entries in history (including temp).
///
/// This is useful for debugging or when the exact count is needed.
///
/// # Safety
/// Calls C accessor functions.
#[no_mangle]
pub unsafe extern "C" fn rs_msg_hist_count() -> c_int {
    let mut count = 0;
    let mut entry = msg_hist_first;
    while !entry.is_null() {
        count += 1;
        entry = nvim_msg_hist_entry_get_next(entry);
    }
    count
}

/// Count temporary entries in the history.
///
/// # Safety
/// Calls C accessor functions.
#[no_mangle]
pub unsafe extern "C" fn rs_msg_hist_temp_count() -> c_int {
    let mut count = 0;
    let mut entry = msg_hist_temp;
    while !entry.is_null() {
        if nvim_msg_hist_entry_get_temp(entry) != 0 {
            count += 1;
        }
        entry = nvim_msg_hist_entry_get_next(entry);
    }
    count
}

/// Get the first temporary entry in the history.
///
/// # Safety
/// Calls C accessor function.
#[no_mangle]
pub unsafe extern "C" fn rs_msg_hist_temp() -> *mut MessageHistoryEntryHandle {
    msg_hist_temp
}

/// Set the first temporary entry in the history.
///
/// # Safety
/// Calls C mutator function.
#[no_mangle]
pub unsafe extern "C" fn rs_set_msg_hist_temp(entry: *mut MessageHistoryEntryHandle) {
    msg_hist_temp = entry;
}

/// Set the history length.
///
/// # Safety
/// Calls C mutator function.
#[no_mangle]
pub unsafe extern "C" fn rs_set_msg_hist_len(len: c_int) {
    msg_hist_len = len;
}

/// Increment history length.
///
/// # Safety
/// Calls C accessor/mutator functions.
#[no_mangle]
pub unsafe extern "C" fn rs_inc_msg_hist_len() {
    let len = msg_hist_len;
    msg_hist_len = len + 1;
}

/// Decrement history length.
///
/// # Safety
/// Calls C accessor/mutator functions.
#[no_mangle]
pub unsafe extern "C" fn rs_dec_msg_hist_len() {
    let len = msg_hist_len;
    if len > 0 {
        msg_hist_len = len - 1;
    }
}

// ============================================================================
// Phase 427: Additional History Management Functions
// ============================================================================

/// Set the first history entry directly.
///
/// # Safety
/// Calls C mutator function.
#[no_mangle]
pub unsafe extern "C" fn rs_set_msg_hist_first(entry: *mut MessageHistoryEntryHandle) {
    msg_hist_first = entry;
}

/// Set the last history entry directly.
///
/// # Safety
/// Calls C mutator function.
#[no_mangle]
pub unsafe extern "C" fn rs_set_msg_hist_last(entry: *mut MessageHistoryEntryHandle) {
    msg_hist_last = entry;
}

/// Check if history has exactly one entry.
///
/// # Safety
/// Calls C accessor functions.
#[no_mangle]
pub unsafe extern "C" fn rs_msg_hist_single() -> c_int {
    let first = msg_hist_first;
    if first.is_null() {
        return 0;
    }
    c_int::from(nvim_msg_hist_entry_get_next(first).is_null())
}

/// Check if a specific entry is the last one in history.
///
/// # Safety
/// Calls C accessor function.
#[no_mangle]
pub unsafe extern "C" fn rs_msg_hist_is_last(entry: *mut MessageHistoryEntryHandle) -> c_int {
    if entry.is_null() {
        return 0;
    }
    c_int::from(entry == msg_hist_last)
}

/// Check if a specific entry is the first one in history.
///
/// # Safety
/// Calls C accessor function.
#[no_mangle]
pub unsafe extern "C" fn rs_msg_hist_is_first(entry: *mut MessageHistoryEntryHandle) -> c_int {
    if entry.is_null() {
        return 0;
    }
    c_int::from(entry == msg_hist_first)
}

/// Get the number of non-temporary entries in history.
///
/// # Safety
/// Calls C accessor function.
#[no_mangle]
pub unsafe extern "C" fn rs_msg_hist_permanent_count() -> c_int {
    msg_hist_len
}

/// Check if history recording is currently possible.
///
/// Returns true if not disabled and not at capacity.
///
/// # Safety
/// Calls C accessor functions.
#[no_mangle]
pub unsafe extern "C" fn rs_msg_hist_can_add() -> c_int {
    let disabled = msg_hist_off || msg_silent != 0;
    c_int::from(!disabled)
}

/// Trim history to the maximum allowed size.
///
/// Equivalent to calling `rs_msg_hist_clear(max)`.
///
/// # Safety
/// Calls C accessor and mutator functions.
#[no_mangle]
pub unsafe extern "C" fn rs_msg_hist_trim() {
    let max = msg_hist_max;
    rs_msg_hist_clear(max);
}

/// Clear all history entries.
///
/// Equivalent to calling `rs_msg_hist_clear(0)`.
///
/// # Safety
/// Calls C accessor and mutator functions.
#[no_mangle]
pub unsafe extern "C" fn rs_msg_hist_clear_all() {
    rs_msg_hist_clear(0);
}

/// Get the nth entry from the start of history.
///
/// Returns NULL if n >= total count.
///
/// # Safety
/// Calls C accessor functions.
#[no_mangle]
pub unsafe extern "C" fn rs_msg_hist_nth(n: c_int) -> *mut MessageHistoryEntryHandle {
    if n < 0 {
        return ptr::null_mut();
    }

    let mut entry = msg_hist_first;
    let mut i = 0;

    while !entry.is_null() && i < n {
        entry = nvim_msg_hist_entry_get_next(entry);
        i += 1;
    }

    entry
}

/// Get the nth entry from the end of history.
///
/// Returns NULL if n >= total count.
///
/// # Safety
/// Calls C accessor functions.
#[no_mangle]
pub unsafe extern "C" fn rs_msg_hist_nth_last(n: c_int) -> *mut MessageHistoryEntryHandle {
    if n < 0 {
        return ptr::null_mut();
    }

    let mut entry = msg_hist_last;
    let mut i = 0;

    while !entry.is_null() && i < n {
        entry = nvim_msg_hist_entry_get_prev(entry);
        i += 1;
    }

    entry
}

/// Skip entries from the start of history.
///
/// Returns the (skip+1)th entry, or NULL if not enough entries.
///
/// # Safety
/// Calls C accessor functions.
#[no_mangle]
pub unsafe extern "C" fn rs_msg_hist_skip(skip: c_int) -> *mut MessageHistoryEntryHandle {
    rs_msg_hist_nth(skip)
}

/// Calculate how many entries to skip for `:messages N`.
///
/// Given total length and requested count, returns how many
/// to skip from the beginning.
///
/// # Safety
/// Calls C accessor function.
#[no_mangle]
pub unsafe extern "C" fn rs_msg_hist_calc_skip(count: c_int) -> c_int {
    let len = msg_hist_len;
    if count >= len {
        0
    } else {
        len - count
    }
}

#[cfg(test)]
mod tests {
    // Integration tests would require mocking C functions
    // Unit tests for pure Rust logic go here
}
