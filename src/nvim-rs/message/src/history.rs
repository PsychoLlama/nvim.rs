//! Message history management
//!
//! Implements the message history linked list for `:messages` command.
//! The history stores highlighted message chunks and supports iteration
//! for display and clearing.

use std::ffi::{c_char, c_int};
use std::ptr;

use nvim_api::{Object, ObjectData};

/// Layout-compatible representation of `HlMessageChunk` in C.
/// Size: 24 bytes (String=16, int=4, padding=4)
#[repr(C)]
#[derive(Copy, Clone)]
pub struct HlMessageChunk {
    pub text_data: *mut c_char, // String.data  (offset 0)
    pub text_size: usize,       // String.size  (offset 8)
    pub hl_id: c_int,           // offset 16
    _pad: c_int,                // offset 20 (alignment padding)
}

impl HlMessageChunk {
    /// Create a new chunk from a text pointer, size, and highlight ID.
    pub const fn new(text_data: *mut c_char, text_size: usize, hl_id: c_int) -> Self {
        Self {
            text_data,
            text_size,
            hl_id,
            _pad: 0,
        }
    }
}

/// Layout-compatible representation of `HlMessage` (kvec_t(HlMessageChunk)) in C.
/// Size: 24 bytes
#[repr(C)]
#[derive(Copy, Clone)]
pub struct HlMessage {
    pub size: usize,                // offset 0
    pub capacity: usize,            // offset 8
    pub items: *mut HlMessageChunk, // offset 16
}

// C accessor declarations
extern "C" {
    static mut msg_silent: c_int;
    /// `msg_hist_max` — Rust-owned static (misc.rs)
    static mut msg_hist_max: c_int;
    /// `msg_hist_off` — direct access to C global
    static mut msg_hist_off: bool;
    /// `msg_ext_append` — direct access to C global (message.h)
    static mut msg_ext_append: bool;
    /// xfree wrapper
    fn xfree(ptr: *mut std::ffi::c_void);
    fn xmalloc(size: usize) -> *mut c_char;
}

/// Free an `HlMessage` stored inside a `MessageHistoryEntry`.
///
/// Mirrors the C `hl_msg_free` logic: frees each chunk's text data,
/// then frees the items array and zeroes the kvec fields.
///
/// # Safety
/// `entry` must be a valid, non-null pointer to a `MessageHistoryEntry`.
unsafe fn hl_msg_free_entry(entry: *mut MessageHistoryEntry) {
    let msg = std::ptr::addr_of_mut!((*entry).msg);
    let size = (*msg).size;
    let items = (*msg).items;
    for i in 0..size {
        let chunk = items.add(i);
        xfree((*chunk).text_data.cast());
    }
    xfree(items.cast());
    (*msg).size = 0;
    (*msg).capacity = 0;
    (*msg).items = std::ptr::null_mut();
}

/// Layout-compatible representation of `MessageHistoryEntry` in C.
///
/// Field offsets verified against C struct:
/// - next:   offset 0  (8 bytes, pointer)
/// - prev:   offset 8  (8 bytes, pointer)
/// - msg:    offset 16 (24 bytes, HlMessage = kvec_t(HlMessageChunk))
/// - kind:   offset 40 (8 bytes, const char*)
/// - temp:   offset 48 (1 byte, bool)
/// - append: offset 49 (1 byte, bool)
/// - padding: 6 bytes (total: 56 bytes)
#[repr(C)]
pub struct MessageHistoryEntry {
    pub next: *mut MessageHistoryEntry,
    pub prev: *mut MessageHistoryEntry,
    pub msg: HlMessage,
    pub kind: *const c_char,
    pub temp: bool,
    pub append: bool,
    _pad: [u8; 6],
}

// ============================================================================
// Rust-owned statics (previously file-local in message.c)
// ============================================================================

/// First message in history linked list (replaces C static msg_hist_first)
#[no_mangle]
pub static mut msg_hist_first: *mut MessageHistoryEntry = std::ptr::null_mut();

/// Last message in history linked list (replaces C non-static msg_hist_last)
#[no_mangle]
pub static mut msg_hist_last: *mut MessageHistoryEntry = std::ptr::null_mut();

/// First potentially temporary message (replaces C static msg_hist_temp)
#[no_mangle]
pub static mut msg_hist_temp: *mut MessageHistoryEntry = std::ptr::null_mut();

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
pub unsafe extern "C" fn rs_msg_hist_first() -> *mut MessageHistoryEntry {
    msg_hist_first
}

/// Get the last entry in the message history.
///
/// # Safety
/// Calls C accessor function for `msg_hist_last`.
#[no_mangle]
pub unsafe extern "C" fn rs_msg_hist_last() -> *mut MessageHistoryEntry {
    msg_hist_last
}

/// Get the next entry in the message history.
///
/// # Safety
/// Direct field access on repr(C) struct.
#[no_mangle]
pub unsafe extern "C" fn rs_msg_hist_next(
    entry: *mut MessageHistoryEntry,
) -> *mut MessageHistoryEntry {
    if entry.is_null() {
        return ptr::null_mut();
    }
    (*entry).next
}

/// Get the previous entry in the message history.
///
/// # Safety
/// Direct field access on repr(C) struct.
#[no_mangle]
pub unsafe extern "C" fn rs_msg_hist_prev(
    entry: *mut MessageHistoryEntry,
) -> *mut MessageHistoryEntry {
    if entry.is_null() {
        return ptr::null_mut();
    }
    (*entry).prev
}

/// Check if an entry is temporary.
///
/// # Safety
/// Direct field access on repr(C) struct.
#[no_mangle]
pub unsafe extern "C" fn rs_msg_hist_entry_is_temp(entry: *mut MessageHistoryEntry) -> c_int {
    if entry.is_null() {
        return 0;
    }
    c_int::from((*entry).temp)
}

/// Check if an entry should be appended to the previous message.
///
/// # Safety
/// Direct field access on repr(C) struct.
#[no_mangle]
pub unsafe extern "C" fn rs_msg_hist_entry_is_append(entry: *mut MessageHistoryEntry) -> c_int {
    if entry.is_null() {
        return 0;
    }
    c_int::from((*entry).append)
}

/// Get the message kind for an entry.
///
/// # Safety
/// Direct field access on repr(C) struct.
#[no_mangle]
pub unsafe extern "C" fn rs_msg_hist_entry_kind(entry: *mut MessageHistoryEntry) -> *const c_char {
    if entry.is_null() {
        return ptr::null();
    }
    (*entry).kind
}

/// Free a single message history entry, unlinking it from the list.
///
/// # Safety
/// - `entry` must be a valid pointer or NULL.
/// - After this call, the entry is freed and must not be used.
#[no_mangle]
pub unsafe extern "C" fn rs_msg_hist_free_entry(entry: *mut MessageHistoryEntry) {
    if entry.is_null() {
        return;
    }

    let next = (*entry).next;
    let prev = (*entry).prev;

    // Update next's prev pointer
    if next.is_null() {
        msg_hist_last = prev;
    } else {
        (*next).prev = prev;
    }

    // Update prev's next pointer
    if prev.is_null() {
        msg_hist_first = next;
    } else {
        (*prev).next = next;
    }

    // Update msg_hist_temp if needed
    let temp = msg_hist_temp;
    if entry == temp {
        msg_hist_temp = next;
    }

    // Free the message content and entry
    hl_msg_free_entry(entry);
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
        if !first.is_null() && !(*first).temp {
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
        let next = (*current).next;

        if (*current).temp {
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
        entry = (*entry).next;
    }
    count
}

/// Count temporary entries in the history.
///
/// # Safety
/// Direct field access on repr(C) struct.
#[no_mangle]
pub unsafe extern "C" fn rs_msg_hist_temp_count() -> c_int {
    let mut count = 0;
    let mut entry = msg_hist_temp;
    while !entry.is_null() {
        if (*entry).temp {
            count += 1;
        }
        entry = (*entry).next;
    }
    count
}

/// Get the first temporary entry in the history.
///
/// # Safety
/// Calls C accessor function.
#[no_mangle]
pub unsafe extern "C" fn rs_msg_hist_temp() -> *mut MessageHistoryEntry {
    msg_hist_temp
}

/// Set the first temporary entry in the history.
///
/// # Safety
/// Calls C mutator function.
#[no_mangle]
pub unsafe extern "C" fn rs_set_msg_hist_temp(entry: *mut MessageHistoryEntry) {
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
pub unsafe extern "C" fn rs_set_msg_hist_first(entry: *mut MessageHistoryEntry) {
    msg_hist_first = entry;
}

/// Set the last history entry directly.
///
/// # Safety
/// Calls C mutator function.
#[no_mangle]
pub unsafe extern "C" fn rs_set_msg_hist_last(entry: *mut MessageHistoryEntry) {
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
    c_int::from((*first).next.is_null())
}

/// Check if a specific entry is the last one in history.
///
/// # Safety
/// Calls C accessor function.
#[no_mangle]
pub unsafe extern "C" fn rs_msg_hist_is_last(entry: *mut MessageHistoryEntry) -> c_int {
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
pub unsafe extern "C" fn rs_msg_hist_is_first(entry: *mut MessageHistoryEntry) -> c_int {
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
pub unsafe extern "C" fn rs_msg_hist_nth(n: c_int) -> *mut MessageHistoryEntry {
    if n < 0 {
        return ptr::null_mut();
    }

    let mut entry = msg_hist_first;
    let mut i = 0;

    while !entry.is_null() && i < n {
        entry = (*entry).next;
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
pub unsafe extern "C" fn rs_msg_hist_nth_last(n: c_int) -> *mut MessageHistoryEntry {
    if n < 0 {
        return ptr::null_mut();
    }

    let mut entry = msg_hist_last;
    let mut i = 0;

    while !entry.is_null() && i < n {
        entry = (*entry).prev;
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
pub unsafe extern "C" fn rs_msg_hist_skip(skip: c_int) -> *mut MessageHistoryEntry {
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

// ============================================================================
// Phase 11: hl_msg_free, msg_hist_add, msg_hist_add_multihl migrated to Rust
// ============================================================================

/// Free an `HlMessage` by value: free each chunk's text data, then the items array.
///
/// Equivalent to the C `hl_msg_free()` function.
///
/// # Safety
/// `hl_msg` must be a valid `HlMessage` whose chunks and items pointer were heap-allocated.
#[export_name = "hl_msg_free"]
pub unsafe extern "C" fn rs_hl_msg_free(hl_msg: HlMessage) {
    for i in 0..hl_msg.size {
        xfree((*hl_msg.items.add(i)).text_data.cast::<std::ffi::c_void>());
    }
    xfree(hl_msg.items.cast::<std::ffi::c_void>());
}

/// Add a plain-text string message to history.
///
/// Strips leading/trailing newlines, creates a single-chunk `HlMessage`,
/// and delegates to `msg_hist_add_multihl`.
///
/// Equivalent to the C `msg_hist_add()` function.
///
/// # Safety
/// `s` must be a valid C string pointer; `len` is the byte count or -1.
#[allow(clippy::cast_sign_loss, clippy::cast_ptr_alignment)]
#[export_name = "msg_hist_add"]
pub unsafe extern "C" fn rs_msg_hist_add(s: *const c_char, len: c_int, hl_id: c_int) {
    let raw_len: usize = if len < 0 {
        std::ffi::CStr::from_ptr(s).to_bytes().len()
    } else {
        len as usize
    };

    let mut text_ptr = s;
    let mut text_size = raw_len;
    let newline: c_char = 10; // '\n'

    // Strip leading newlines
    while text_size > 0 && *text_ptr == newline {
        text_size -= 1;
        text_ptr = text_ptr.add(1);
    }
    // Strip trailing newlines
    while text_size > 0 && *text_ptr.add(text_size - 1) == newline {
        text_size -= 1;
    }

    if text_size == 0 {
        return;
    }

    // Duplicate the text
    let data = xmalloc(text_size + 1);
    std::ptr::copy_nonoverlapping(text_ptr, data, text_size);
    *data.add(text_size) = 0; // NUL-terminate

    // Build a single-chunk HlMessage
    let chunk = HlMessageChunk::new(data, text_size, hl_id);
    let items = xmalloc(std::mem::size_of::<HlMessageChunk>()).cast::<HlMessageChunk>();
    std::ptr::write(items, chunk);
    let msg = HlMessage {
        size: 1,
        capacity: 1,
        items,
    };

    // INTEGER_OBJ(0) = { type=kObjectTypeInteger, data.integer=0 }
    rs_msg_hist_add_multihl(
        Object {
            obj_type: 2, // kObjectTypeInteger
            data: ObjectData { integer: 0 },
        },
        msg,
        false,
        ptr::null_mut(),
    );
}

/// Add a highlighted multi-chunk message to history.
///
/// Equivalent to the C `msg_hist_add_multihl()` function.
///
/// # Safety
/// Accesses global message history state and Rust-owned statics.
#[allow(clippy::cast_ptr_alignment)]
#[export_name = "msg_hist_add_multihl"]
pub unsafe extern "C" fn rs_msg_hist_add_multihl(
    _msg_id: Object,
    msg: HlMessage,
    temp: bool,
    _msg_data: *mut std::ffi::c_void,
) {
    if crate::scrollback::do_clear_hist_temp {
        rs_msg_hist_clear_temp();
        crate::scrollback::do_clear_hist_temp = false;
    }

    if msg_hist_off || msg_silent != 0 {
        rs_hl_msg_free(msg);
        return;
    }

    // Allocate a new history entry
    let entry = xmalloc(std::mem::size_of::<MessageHistoryEntry>()).cast::<MessageHistoryEntry>();
    (*entry).msg = msg;
    (*entry).temp = temp;
    (*entry).kind = crate::display::msg_ext_kind;
    (*entry).prev = msg_hist_last;
    (*entry).next = ptr::null_mut();
    (*entry).append = msg_ext_append;

    if msg_hist_first.is_null() {
        msg_hist_first = entry;
    }
    if !msg_hist_last.is_null() {
        (*msg_hist_last).next = entry;
    }
    if msg_hist_temp.is_null() {
        msg_hist_temp = entry;
    }

    if !temp {
        msg_hist_len += 1;
    }
    msg_hist_last = entry;
    crate::display::msg_ext_history = true;

    rs_msg_hist_clear(msg_hist_max);
}

#[cfg(test)]
mod tests {
    // Integration tests would require mocking C functions
    // Unit tests for pure Rust logic go here
}
