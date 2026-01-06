//! Message chunk structures for scrollback buffer
//!
//! Implements the `msgchunk_T` equivalent for storing text chunks
//! in the scrollback buffer used by "more" and "hit-enter" prompts.

use std::ffi::c_int;
use std::ptr;

// C accessor declarations
extern "C" {
    /// Get `last_msgchunk` pointer
    fn nvim_get_last_msgchunk() -> *mut MsgChunkHandle;
    /// Set `last_msgchunk` pointer
    fn nvim_set_last_msgchunk(chunk: *mut MsgChunkHandle);
    /// Get chunk->sb_next
    fn nvim_msgchunk_get_next(chunk: *mut MsgChunkHandle) -> *mut MsgChunkHandle;
    /// Set chunk->sb_next
    fn nvim_msgchunk_set_next(chunk: *mut MsgChunkHandle, next: *mut MsgChunkHandle);
    /// Get chunk->sb_prev
    fn nvim_msgchunk_get_prev(chunk: *mut MsgChunkHandle) -> *mut MsgChunkHandle;
    /// Set chunk->sb_prev (kept for future use)
    #[allow(dead_code)]
    fn nvim_msgchunk_set_prev(chunk: *mut MsgChunkHandle, prev: *mut MsgChunkHandle);
    /// Get chunk->sb_eol
    fn nvim_msgchunk_get_eol(chunk: *mut MsgChunkHandle) -> c_int;
    /// Set chunk->sb_eol
    fn nvim_msgchunk_set_eol(chunk: *mut MsgChunkHandle, eol: c_int);
    /// Get chunk->sb_msg_col
    fn nvim_msgchunk_get_msg_col(chunk: *mut MsgChunkHandle) -> c_int;
    /// Get chunk->sb_hl_id
    fn nvim_msgchunk_get_hl_id(chunk: *mut MsgChunkHandle) -> c_int;
    /// Get chunk->sb_text pointer
    fn nvim_msgchunk_get_text(chunk: *mut MsgChunkHandle) -> *const std::ffi::c_char;
    /// xfree wrapper
    fn xfree(ptr: *mut std::ffi::c_void);
}

/// Opaque handle to `msgchunk_T` in C.
///
/// This type is never instantiated in Rust; it exists only to provide
/// type safety for pointers passed across the FFI boundary.
#[repr(C)]
pub struct MsgChunkHandle {
    _private: [u8; 0],
}

/// Get the last message chunk in the scrollback buffer.
///
/// # Safety
/// Calls C accessor function.
#[no_mangle]
pub unsafe extern "C" fn rs_msg_sb_last() -> *mut MsgChunkHandle {
    nvim_get_last_msgchunk()
}

/// Get the next chunk in the scrollback buffer.
///
/// # Safety
/// Calls C accessor function for chunk->sb_next.
#[no_mangle]
pub unsafe extern "C" fn rs_msgchunk_next(chunk: *mut MsgChunkHandle) -> *mut MsgChunkHandle {
    if chunk.is_null() {
        return ptr::null_mut();
    }
    nvim_msgchunk_get_next(chunk)
}

/// Get the previous chunk in the scrollback buffer.
///
/// # Safety
/// Calls C accessor function for chunk->sb_prev.
#[no_mangle]
pub unsafe extern "C" fn rs_msgchunk_prev(chunk: *mut MsgChunkHandle) -> *mut MsgChunkHandle {
    if chunk.is_null() {
        return ptr::null_mut();
    }
    nvim_msgchunk_get_prev(chunk)
}

/// Check if a chunk marks end-of-line.
///
/// # Safety
/// Calls C accessor function for chunk->sb_eol.
#[no_mangle]
pub unsafe extern "C" fn rs_msgchunk_is_eol(chunk: *mut MsgChunkHandle) -> c_int {
    if chunk.is_null() {
        return 0;
    }
    nvim_msgchunk_get_eol(chunk)
}

/// Get the message column for a chunk.
///
/// # Safety
/// Calls C accessor function for chunk->sb_msg_col.
#[no_mangle]
pub unsafe extern "C" fn rs_msgchunk_col(chunk: *mut MsgChunkHandle) -> c_int {
    if chunk.is_null() {
        return 0;
    }
    nvim_msgchunk_get_msg_col(chunk)
}

/// Get the highlight ID for a chunk.
///
/// # Safety
/// Calls C accessor function for chunk->sb_hl_id.
#[no_mangle]
pub unsafe extern "C" fn rs_msgchunk_hl_id(chunk: *mut MsgChunkHandle) -> c_int {
    if chunk.is_null() {
        return 0;
    }
    nvim_msgchunk_get_hl_id(chunk)
}

/// Get the text pointer for a chunk.
///
/// # Safety
/// Calls C accessor function for chunk->sb_text.
#[no_mangle]
pub unsafe extern "C" fn rs_msgchunk_text(chunk: *mut MsgChunkHandle) -> *const std::ffi::c_char {
    if chunk.is_null() {
        return ptr::null();
    }
    nvim_msgchunk_get_text(chunk)
}

/// Find the start of a screen line in the scrollback buffer.
///
/// Given a chunk, walks backwards through the linked list until it finds
/// a chunk where the previous chunk has `sb_eol` set (or there is no previous).
///
/// # Safety
/// Calls C accessor functions.
#[no_mangle]
pub unsafe extern "C" fn rs_msg_sb_start(mps: *mut MsgChunkHandle) -> *mut MsgChunkHandle {
    if mps.is_null() {
        return ptr::null_mut();
    }

    let mut mp = mps;

    loop {
        let prev = nvim_msgchunk_get_prev(mp);
        if prev.is_null() {
            break;
        }
        if nvim_msgchunk_get_eol(prev) != 0 {
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
/// Calls C accessor and mutator functions.
#[no_mangle]
pub unsafe extern "C" fn rs_msg_sb_eol() {
    let last = nvim_get_last_msgchunk();
    if !last.is_null() {
        nvim_msgchunk_set_eol(last, 1);
    }
}

/// Clear all scrollback buffer content.
///
/// Frees all chunks from the buffer, walking from the end to the beginning.
///
/// # Safety
/// Calls C accessor and mutator functions, frees memory.
#[no_mangle]
pub unsafe extern "C" fn rs_msg_sb_clear_all() {
    let mut chunk = nvim_get_last_msgchunk();

    while !chunk.is_null() {
        let prev = nvim_msgchunk_get_prev(chunk);
        xfree(chunk.cast());
        chunk = prev;
    }

    nvim_set_last_msgchunk(ptr::null_mut());
}

/// Clear `count` lines from the beginning of the scrollback buffer.
///
/// # Arguments
/// * `count` - Number of lines to clear
///
/// # Safety
/// Calls C accessor and mutator functions, frees memory.
#[no_mangle]
pub unsafe extern "C" fn rs_msg_sb_clear_lines(count: c_int) {
    let last = nvim_get_last_msgchunk();
    if last.is_null() {
        return;
    }

    // Find the start of the scrollback
    let mp = rs_msg_sb_start(last);
    if mp.is_null() {
        return;
    }

    // Walk back to find the line to clear
    let prev = nvim_msgchunk_get_prev(mp);
    if prev.is_null() {
        return;
    }

    let mut target = rs_msg_sb_start(prev);
    let mut lines_to_clear = count;

    while lines_to_clear > 0 && !target.is_null() {
        let target_prev = nvim_msgchunk_get_prev(target);
        if target_prev.is_null() {
            break;
        }
        target = rs_msg_sb_start(target_prev);
        lines_to_clear -= 1;
    }

    // Free chunks from target to the start
    if !target.is_null() {
        let target_prev = nvim_msgchunk_get_prev(target);
        let mut to_free = target;

        // Disconnect from the rest of the list
        if !target_prev.is_null() {
            nvim_msgchunk_set_next(target_prev, ptr::null_mut());
        }

        // Free the chunks
        while !to_free.is_null() {
            let next = nvim_msgchunk_get_next(to_free);
            xfree(to_free.cast());
            to_free = next;
        }
    }
}

/// Check if there are any lines in the scrollback before the current position.
///
/// # Safety
/// Calls C accessor functions.
#[no_mangle]
pub unsafe extern "C" fn rs_msg_sb_has_prev_line() -> c_int {
    let last = nvim_get_last_msgchunk();
    if last.is_null() {
        return 0;
    }

    let start = rs_msg_sb_start(last);
    if start.is_null() {
        return 0;
    }

    let prev = nvim_msgchunk_get_prev(start);
    c_int::from(!prev.is_null())
}

#[cfg(test)]
mod tests {
    // Integration tests would require mocking C functions
    // Unit tests for pure Rust logic go here
}
