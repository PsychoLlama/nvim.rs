//! Message chunk structures for scrollback buffer
//!
//! Implements the `msgchunk_T` equivalent for storing text chunks
//! in the scrollback buffer used by "more" and "hit-enter" prompts.

use std::ffi::{c_char, c_int};
use std::ptr;

// C accessor declarations
extern "C" {
    // last_msgchunk: Rust-owned static in scrollback.rs
    static mut last_msgchunk: *mut MsgChunk;
    /// xfree wrapper
    fn xfree(ptr: *mut std::ffi::c_void);
}

/// Layout-compatible representation of `msgchunk_T` in C.
///
/// Field offsets verified against C struct:
/// - sb_next:    offset 0  (8 bytes)
/// - sb_prev:    offset 8  (8 bytes)
/// - sb_eol:     offset 16 (1 byte, char)
/// - _pad:       offset 17 (3 bytes padding)
/// - sb_msg_col: offset 20 (4 bytes, int)
/// - sb_hl_id:   offset 24 (4 bytes, int)
/// - sb_text[]:  offset 28 (flexible array, not in Rust struct)
#[repr(C)]
pub struct MsgChunk {
    pub sb_next: *mut MsgChunk,
    pub sb_prev: *mut MsgChunk,
    pub sb_eol: c_char,
    _pad: [u8; 3],
    pub sb_msg_col: c_int,
    pub sb_hl_id: c_int,
}

/// Get the last message chunk in the scrollback buffer.
///
/// # Safety
/// Calls C accessor function.
#[no_mangle]
pub unsafe extern "C" fn rs_msg_sb_last() -> *mut MsgChunk {
    last_msgchunk
}

/// Get the next chunk in the scrollback buffer.
///
/// # Safety
/// Direct field access on repr(C) struct.
#[no_mangle]
pub unsafe extern "C" fn rs_msgchunk_next(chunk: *mut MsgChunk) -> *mut MsgChunk {
    if chunk.is_null() {
        return ptr::null_mut();
    }
    (*chunk).sb_next
}

/// Get the previous chunk in the scrollback buffer.
///
/// # Safety
/// Direct field access on repr(C) struct.
#[no_mangle]
pub unsafe extern "C" fn rs_msgchunk_prev(chunk: *mut MsgChunk) -> *mut MsgChunk {
    if chunk.is_null() {
        return ptr::null_mut();
    }
    (*chunk).sb_prev
}

/// Check if a chunk marks end-of-line.
///
/// # Safety
/// Direct field access on repr(C) struct.
#[no_mangle]
pub unsafe extern "C" fn rs_msgchunk_is_eol(chunk: *mut MsgChunk) -> c_int {
    if chunk.is_null() {
        return 0;
    }
    c_int::from((*chunk).sb_eol != 0)
}

/// Get the message column for a chunk.
///
/// # Safety
/// Direct field access on repr(C) struct.
#[no_mangle]
pub unsafe extern "C" fn rs_msgchunk_col(chunk: *mut MsgChunk) -> c_int {
    if chunk.is_null() {
        return 0;
    }
    (*chunk).sb_msg_col
}

/// Get the highlight ID for a chunk.
///
/// # Safety
/// Direct field access on repr(C) struct.
#[no_mangle]
pub unsafe extern "C" fn rs_msgchunk_hl_id(chunk: *mut MsgChunk) -> c_int {
    if chunk.is_null() {
        return 0;
    }
    (*chunk).sb_hl_id
}

/// Get the text pointer for a chunk.
///
/// The `sb_text[]` flexible array immediately follows the struct fields.
/// `size_of::<MsgChunk>()` is 28, matching the C layout.
///
/// # Safety
/// Direct pointer arithmetic on repr(C) struct.
#[no_mangle]
pub const unsafe extern "C" fn rs_msgchunk_text(chunk: *mut MsgChunk) -> *const c_char {
    if chunk.is_null() {
        return ptr::null();
    }
    // sb_text[] flexible array starts immediately after the struct fields
    (chunk as *const u8)
        .add(std::mem::size_of::<MsgChunk>())
        .cast::<c_char>()
}

/// Find the start of a screen line in the scrollback buffer.
///
/// Given a chunk, walks backwards through the linked list until it finds
/// a chunk where the previous chunk has `sb_eol` set (or there is no previous).
///
/// # Safety
/// Direct field access on repr(C) struct.
#[no_mangle]
pub unsafe extern "C" fn rs_msg_sb_start(mps: *mut MsgChunk) -> *mut MsgChunk {
    if mps.is_null() {
        return ptr::null_mut();
    }

    let mut mp = mps;

    loop {
        let prev = (*mp).sb_prev;
        if prev.is_null() {
            break;
        }
        if (*prev).sb_eol != 0 {
            break;
        }
        mp = prev;
    }

    mp
}

/// Mark the end of a line in the scrollback buffer.
///
/// Sets `sb_eol` to true on the last chunk if it exists.
///
/// # Safety
/// Direct field access on repr(C) struct.
#[no_mangle]
pub unsafe extern "C" fn rs_msg_sb_eol() {
    let last = last_msgchunk;
    if !last.is_null() {
        (*last).sb_eol = 1;
    }
}

/// Clear all scrollback buffer content.
///
/// Frees all chunks from the buffer, walking from the end to the beginning.
///
/// # Safety
/// Direct field access on repr(C) struct, frees memory.
#[no_mangle]
pub unsafe extern "C" fn rs_msg_sb_clear_all() {
    let mut chunk = last_msgchunk;

    while !chunk.is_null() {
        let prev = (*chunk).sb_prev;
        xfree(chunk.cast());
        chunk = prev;
    }

    last_msgchunk = ptr::null_mut();
}

/// Clear `count` lines from the beginning of the scrollback buffer.
///
/// # Arguments
/// * `count` - Number of lines to clear
///
/// # Safety
/// Direct field access on repr(C) struct, frees memory.
#[no_mangle]
pub unsafe extern "C" fn rs_msg_sb_clear_lines(count: c_int) {
    let last = last_msgchunk;
    if last.is_null() {
        return;
    }

    // Find the start of the scrollback
    let mp = rs_msg_sb_start(last);
    if mp.is_null() {
        return;
    }

    // Walk back to find the line to clear
    let prev = (*mp).sb_prev;
    if prev.is_null() {
        return;
    }

    let mut target = rs_msg_sb_start(prev);
    let mut lines_to_clear = count;

    while lines_to_clear > 0 && !target.is_null() {
        let target_prev = (*target).sb_prev;
        if target_prev.is_null() {
            break;
        }
        target = rs_msg_sb_start(target_prev);
        lines_to_clear -= 1;
    }

    // Free chunks from target to the start
    if !target.is_null() {
        let target_prev = (*target).sb_prev;
        let mut to_free = target;

        // Disconnect from the rest of the list
        if !target_prev.is_null() {
            (*target_prev).sb_next = ptr::null_mut();
        }

        // Free the chunks
        while !to_free.is_null() {
            let next = (*to_free).sb_next;
            xfree(to_free.cast());
            to_free = next;
        }
    }
}

/// Check if there are any lines in the scrollback before the current position.
///
/// # Safety
/// Direct field access on repr(C) struct.
#[no_mangle]
pub unsafe extern "C" fn rs_msg_sb_has_prev_line() -> c_int {
    let last = last_msgchunk;
    if last.is_null() {
        return 0;
    }

    let start = rs_msg_sb_start(last);
    if start.is_null() {
        return 0;
    }

    let prev = (*start).sb_prev;
    c_int::from(!prev.is_null())
}

#[cfg(test)]
mod tests {
    // Integration tests would require mocking C functions
    // Unit tests for pure Rust logic go here
}
