//! Command line editing operations
//!
//! This module provides the core editing operations for command-line mode,
//! including character insertion, deletion, word operations, and clipboard
//! integration.

#![allow(unsafe_code)]
#![allow(clippy::doc_markdown)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::must_use_candidate)]

use std::ffi::{c_char, c_int};

use crate::state::{next_char_pos, prev_char_pos, utf8_char_len, NUL};

// =============================================================================
// C Function Declarations
// =============================================================================

#[allow(dead_code)]
extern "C" {
    // Buffer management
    fn nvim_get_ccline_cmdbuff() -> *mut c_char;
    fn nvim_get_ccline_cmdbufflen() -> c_int;
    fn nvim_get_ccline_cmdpos() -> c_int;
    fn nvim_get_ccline_cmdlen() -> c_int;
    fn nvim_set_ccline_cmdpos(pos: c_int);
    fn nvim_set_ccline_cmdlen(len: c_int);

    // Reallocation (calls C realloc_cmdbuff)
    fn realloc_cmdbuff(len: c_int) -> c_int;

    // Multibyte utilities
    fn mb_char2len(c: c_int) -> c_int;
    fn utf_char2bytes(c: c_int, buf: *mut c_char) -> c_int;
    fn mb_get_class(p: *const c_char) -> c_int;
    fn mb_prevptr(start: *const c_char, p: *const c_char) -> *mut c_char;
    fn utfc_ptr2len(p: *const c_char) -> c_int;

    // Redraw
    fn redrawcmd();
}

// =============================================================================
// Character Class Constants
// =============================================================================

/// Character class for word operations
pub mod char_class {
    /// Whitespace
    pub const WHITE: i32 = 0;
    /// Word character (alphanumeric + '_')
    pub const WORD: i32 = 2;
}

// =============================================================================
// Edit Result
// =============================================================================

/// Result of an editing operation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EditResult {
    /// Command line was changed
    Changed,
    /// Command line was not changed
    NotChanged,
    /// Operation failed
    Failed,
}

// =============================================================================
// Buffer Management
// =============================================================================

/// Ensure the command buffer has enough space.
///
/// # Safety
///
/// Calls C function to reallocate the buffer.
pub unsafe fn ensure_buffer_space(needed: usize) -> bool {
    let current_len = nvim_get_ccline_cmdbufflen();
    if current_len < 0 {
        return false;
    }

    #[allow(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]
    if (current_len as usize) < needed {
        realloc_cmdbuff((needed + 1) as c_int) == 0
    } else {
        true
    }
}

/// Get the command buffer as a mutable slice.
///
/// # Safety
///
/// Caller must ensure no other references to the buffer exist.
unsafe fn get_buffer_mut() -> Option<&'static mut [u8]> {
    let ptr = nvim_get_ccline_cmdbuff();
    if ptr.is_null() {
        return None;
    }
    let bufflen = nvim_get_ccline_cmdbufflen();
    if bufflen < 0 {
        return None;
    }
    Some(std::slice::from_raw_parts_mut(
        ptr.cast::<u8>(),
        bufflen as usize,
    ))
}

/// Get the command buffer as an immutable slice up to cmdlen.
///
/// # Safety
///
/// Caller must ensure buffer is valid.
unsafe fn get_cmdline_slice() -> Option<&'static [u8]> {
    let ptr = nvim_get_ccline_cmdbuff();
    if ptr.is_null() {
        return None;
    }
    let len = nvim_get_ccline_cmdlen();
    if len < 0 {
        return None;
    }
    Some(std::slice::from_raw_parts(ptr.cast::<u8>(), len as usize))
}

// =============================================================================
// Character Insertion
// =============================================================================

/// Insert a character at the current cursor position.
///
/// # Arguments
///
/// * `c` - The Unicode codepoint to insert
/// * `overstrike` - If true, replace the character at cursor position
///
/// # Safety
///
/// Calls C functions to manipulate the command buffer.
///
/// # Returns
///
/// `EditResult::Changed` if successful, `EditResult::Failed` otherwise.
pub unsafe fn insert_char(c: i32, overstrike: bool) -> EditResult {
    if c == 0 {
        return EditResult::NotChanged;
    }

    // Get character length
    let char_len = mb_char2len(c);
    if char_len <= 0 {
        return EditResult::Failed;
    }
    let char_len = char_len as usize;

    let cmdpos = nvim_get_ccline_cmdpos() as usize;
    let cmdlen = nvim_get_ccline_cmdlen() as usize;

    // Calculate how much space we need
    let old_char_len = if overstrike && cmdpos < cmdlen {
        let Some(buf) = get_cmdline_slice() else {
            return EditResult::Failed;
        };
        utf8_char_len(buf, cmdpos)
    } else {
        0
    };

    let new_len = cmdlen + char_len - old_char_len;

    // Ensure we have enough space
    if !ensure_buffer_space(new_len + 1) {
        return EditResult::Failed;
    }

    let Some(buf) = get_buffer_mut() else {
        return EditResult::Failed;
    };

    // Move existing text if needed
    if overstrike && old_char_len != char_len {
        // Shift text after cursor
        let src_start = cmdpos + old_char_len;
        let dst_start = cmdpos + char_len;
        if src_start < cmdlen {
            buf.copy_within(src_start..cmdlen, dst_start);
        }
    } else if !overstrike && cmdpos < cmdlen {
        // Insert mode: shift everything after cursor
        buf.copy_within(cmdpos..cmdlen, cmdpos + char_len);
    }

    // Write the character
    #[allow(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]
    {
        let ptr = buf.as_mut_ptr().add(cmdpos).cast::<c_char>();
        utf_char2bytes(c, ptr);
    }

    // Update positions
    #[allow(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]
    {
        nvim_set_ccline_cmdlen(new_len as c_int);
        nvim_set_ccline_cmdpos((cmdpos + char_len) as c_int);
        buf[new_len] = NUL;
    }

    EditResult::Changed
}

/// Insert a string at the current cursor position.
///
/// # Arguments
///
/// * `s` - The string bytes to insert
///
/// # Safety
///
/// Calls C functions to manipulate the command buffer.
pub unsafe fn insert_str(s: &[u8]) -> EditResult {
    if s.is_empty() {
        return EditResult::NotChanged;
    }

    let cmdpos = nvim_get_ccline_cmdpos() as usize;
    let cmdlen = nvim_get_ccline_cmdlen() as usize;
    let new_len = cmdlen + s.len();

    // Ensure we have enough space
    if !ensure_buffer_space(new_len + 1) {
        return EditResult::Failed;
    }

    let Some(buf) = get_buffer_mut() else {
        return EditResult::Failed;
    };

    // Shift existing text
    if cmdpos < cmdlen {
        buf.copy_within(cmdpos..cmdlen, cmdpos + s.len());
    }

    // Copy the string
    buf[cmdpos..cmdpos + s.len()].copy_from_slice(s);

    // Update positions
    #[allow(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]
    {
        nvim_set_ccline_cmdlen(new_len as c_int);
        nvim_set_ccline_cmdpos((cmdpos + s.len()) as c_int);
        buf[new_len] = NUL;
    }

    EditResult::Changed
}

// =============================================================================
// Character Deletion
// =============================================================================

/// Delete the character before the cursor (backspace).
///
/// # Safety
///
/// Calls C functions to manipulate the command buffer.
pub unsafe fn delete_char_before() -> EditResult {
    let cmdpos = nvim_get_ccline_cmdpos() as usize;
    if cmdpos == 0 {
        return EditResult::NotChanged;
    }

    let cmdlen = nvim_get_ccline_cmdlen() as usize;
    let Some(buf) = get_cmdline_slice() else {
        return EditResult::Failed;
    };

    // Find the start of the previous character
    let prev_pos = prev_char_pos(buf, cmdpos);
    let char_len = cmdpos - prev_pos;

    let Some(buf) = get_buffer_mut() else {
        return EditResult::Failed;
    };

    // Shift remaining text
    if cmdpos < cmdlen {
        buf.copy_within(cmdpos..cmdlen, prev_pos);
    }

    let new_len = cmdlen - char_len;
    buf[new_len] = NUL;

    #[allow(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]
    {
        nvim_set_ccline_cmdlen(new_len as c_int);
        nvim_set_ccline_cmdpos(prev_pos as c_int);
    }

    EditResult::Changed
}

/// Delete the character at the cursor (delete key).
///
/// # Safety
///
/// Calls C functions to manipulate the command buffer.
pub unsafe fn delete_char_at() -> EditResult {
    let cmdpos = nvim_get_ccline_cmdpos() as usize;
    let cmdlen = nvim_get_ccline_cmdlen() as usize;

    if cmdpos >= cmdlen {
        return EditResult::NotChanged;
    }

    let Some(buf) = get_cmdline_slice() else {
        return EditResult::Failed;
    };
    let char_len = utf8_char_len(buf, cmdpos);

    let Some(buf) = get_buffer_mut() else {
        return EditResult::Failed;
    };

    // Shift remaining text
    let after_char = cmdpos + char_len;
    if after_char < cmdlen {
        buf.copy_within(after_char..cmdlen, cmdpos);
    }

    let new_len = cmdlen - char_len;
    buf[new_len] = NUL;

    #[allow(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]
    nvim_set_ccline_cmdlen(new_len as c_int);

    EditResult::Changed
}

/// Delete the word before the cursor (Ctrl-W).
///
/// This deletes:
/// 1. Any trailing whitespace
/// 2. The word before the cursor
///
/// # Safety
///
/// Calls C functions to manipulate the command buffer.
pub unsafe fn delete_word_before() -> EditResult {
    let cmdpos = nvim_get_ccline_cmdpos() as usize;
    if cmdpos == 0 {
        return EditResult::NotChanged;
    }

    let buf_ptr = nvim_get_ccline_cmdbuff();
    if buf_ptr.is_null() {
        return EditResult::Failed;
    }

    // Use C's mb_prevptr to find previous positions
    let mut p = buf_ptr.add(cmdpos);

    // Skip trailing whitespace
    while p > buf_ptr {
        let prev = mb_prevptr(buf_ptr, p);
        let class = mb_get_class(prev);
        if class != char_class::WHITE {
            break;
        }
        p = prev;
    }

    // Skip the word
    if p > buf_ptr {
        let word_class = mb_get_class(mb_prevptr(buf_ptr, p));
        while p > buf_ptr {
            let prev = mb_prevptr(buf_ptr, p);
            let class = mb_get_class(prev);
            if class != word_class {
                break;
            }
            p = prev;
        }
    }

    #[allow(clippy::cast_sign_loss)]
    let new_pos = p.offset_from(buf_ptr) as usize;
    let delete_len = cmdpos - new_pos;

    if delete_len == 0 {
        return EditResult::NotChanged;
    }

    let cmdlen = nvim_get_ccline_cmdlen() as usize;
    let Some(buf) = get_buffer_mut() else {
        return EditResult::Failed;
    };

    // Shift remaining text
    if cmdpos < cmdlen {
        buf.copy_within(cmdpos..cmdlen, new_pos);
    }

    let new_len = cmdlen - delete_len;
    buf[new_len] = NUL;

    #[allow(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]
    {
        nvim_set_ccline_cmdlen(new_len as c_int);
        nvim_set_ccline_cmdpos(new_pos as c_int);
    }

    EditResult::Changed
}

/// Delete from the cursor to the beginning of the line (Ctrl-U).
///
/// # Safety
///
/// Calls C functions to manipulate the command buffer.
pub unsafe fn delete_to_start() -> EditResult {
    let cmdpos = nvim_get_ccline_cmdpos() as usize;
    if cmdpos == 0 {
        return EditResult::NotChanged;
    }

    let cmdlen = nvim_get_ccline_cmdlen() as usize;
    let Some(buf) = get_buffer_mut() else {
        return EditResult::Failed;
    };

    // Shift remaining text to the beginning
    if cmdpos < cmdlen {
        buf.copy_within(cmdpos..cmdlen, 0);
    }

    let new_len = cmdlen - cmdpos;
    buf[new_len] = NUL;

    #[allow(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]
    {
        nvim_set_ccline_cmdlen(new_len as c_int);
        nvim_set_ccline_cmdpos(0);
    }

    EditResult::Changed
}

/// Delete from the cursor to the end of the line (Ctrl-K in some modes).
///
/// # Safety
///
/// Calls C functions to manipulate the command buffer.
pub unsafe fn delete_to_end() -> EditResult {
    let cmdpos = nvim_get_ccline_cmdpos() as usize;
    let cmdlen = nvim_get_ccline_cmdlen() as usize;

    if cmdpos >= cmdlen {
        return EditResult::NotChanged;
    }

    let Some(buf) = get_buffer_mut() else {
        return EditResult::Failed;
    };
    buf[cmdpos] = NUL;

    #[allow(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]
    nvim_set_ccline_cmdlen(cmdpos as c_int);

    EditResult::Changed
}

// =============================================================================
// Cursor Movement
// =============================================================================

/// Move cursor left by one character.
///
/// # Safety
///
/// Calls C functions to access the command buffer.
pub unsafe fn cursor_left() -> EditResult {
    let cmdpos = nvim_get_ccline_cmdpos() as usize;
    if cmdpos == 0 {
        return EditResult::NotChanged;
    }

    let Some(buf) = get_cmdline_slice() else {
        return EditResult::Failed;
    };
    let new_pos = prev_char_pos(buf, cmdpos);

    #[allow(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]
    nvim_set_ccline_cmdpos(new_pos as c_int);

    EditResult::Changed
}

/// Move cursor right by one character.
///
/// # Safety
///
/// Calls C functions to access the command buffer.
pub unsafe fn cursor_right() -> EditResult {
    let cmdpos = nvim_get_ccline_cmdpos() as usize;
    let cmdlen = nvim_get_ccline_cmdlen() as usize;

    if cmdpos >= cmdlen {
        return EditResult::NotChanged;
    }

    let Some(buf) = get_cmdline_slice() else {
        return EditResult::Failed;
    };
    let new_pos = next_char_pos(buf, cmdpos);

    #[allow(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]
    nvim_set_ccline_cmdpos(new_pos as c_int);

    EditResult::Changed
}

/// Move cursor to the beginning of the line.
///
/// # Safety
///
/// Calls C functions to access the command buffer.
pub unsafe fn cursor_home() -> EditResult {
    let cmdpos = nvim_get_ccline_cmdpos();
    if cmdpos == 0 {
        return EditResult::NotChanged;
    }

    nvim_set_ccline_cmdpos(0);
    EditResult::Changed
}

/// Move cursor to the end of the line.
///
/// # Safety
///
/// Calls C functions to access the command buffer.
pub unsafe fn cursor_end() -> EditResult {
    let cmdpos = nvim_get_ccline_cmdpos();
    let cmdlen = nvim_get_ccline_cmdlen();

    if cmdpos >= cmdlen {
        return EditResult::NotChanged;
    }

    nvim_set_ccline_cmdpos(cmdlen);
    EditResult::Changed
}

/// Move cursor left by one word.
///
/// # Safety
///
/// Calls C functions to access the command buffer.
pub unsafe fn cursor_word_left() -> EditResult {
    let cmdpos = nvim_get_ccline_cmdpos() as usize;
    if cmdpos == 0 {
        return EditResult::NotChanged;
    }

    let buf_ptr = nvim_get_ccline_cmdbuff();
    if buf_ptr.is_null() {
        return EditResult::Failed;
    }

    let mut p = buf_ptr.add(cmdpos);

    // Skip whitespace
    while p > buf_ptr {
        let prev = mb_prevptr(buf_ptr, p);
        if mb_get_class(prev) != char_class::WHITE {
            break;
        }
        p = prev;
    }

    // Skip word
    if p > buf_ptr {
        let word_class = mb_get_class(mb_prevptr(buf_ptr, p));
        while p > buf_ptr {
            let prev = mb_prevptr(buf_ptr, p);
            if mb_get_class(prev) != word_class {
                break;
            }
            p = prev;
        }
    }

    #[allow(
        clippy::cast_sign_loss,
        clippy::cast_possible_truncation,
        clippy::cast_possible_wrap
    )]
    {
        let new_pos = p.offset_from(buf_ptr) as usize;
        nvim_set_ccline_cmdpos(new_pos as c_int);
    }

    EditResult::Changed
}

/// Move cursor right by one word.
///
/// # Safety
///
/// Calls C functions to access the command buffer.
pub unsafe fn cursor_word_right() -> EditResult {
    let cmdpos = nvim_get_ccline_cmdpos() as usize;
    let cmdlen = nvim_get_ccline_cmdlen() as usize;

    if cmdpos >= cmdlen {
        return EditResult::NotChanged;
    }

    let Some(buf) = get_cmdline_slice() else {
        return EditResult::Failed;
    };
    let buf_ptr = nvim_get_ccline_cmdbuff();

    // Get current character class
    let mut pos = cmdpos;
    let start_class = mb_get_class(buf_ptr.add(pos));

    // Skip current word
    while pos < cmdlen && mb_get_class(buf_ptr.add(pos)) == start_class {
        pos += utf8_char_len(buf, pos);
    }

    // Skip whitespace
    while pos < cmdlen && mb_get_class(buf_ptr.add(pos)) == char_class::WHITE {
        pos += utf8_char_len(buf, pos);
    }

    #[allow(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]
    nvim_set_ccline_cmdpos(pos as c_int);

    EditResult::Changed
}

// =============================================================================
// FFI Exports
// =============================================================================

/// Insert a character at the current cursor position (FFI).
///
/// # Safety
///
/// Calls C functions to manipulate the command buffer.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_cmdline_insert_char(c: c_int, overstrike: c_int) -> c_int {
    match insert_char(c, overstrike != 0) {
        EditResult::Changed => 1,
        EditResult::NotChanged => 0,
        EditResult::Failed => -1,
    }
}

/// Delete the character before the cursor (FFI).
///
/// # Safety
///
/// Calls C functions to manipulate the command buffer.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_cmdline_delete_char_before() -> c_int {
    match delete_char_before() {
        EditResult::Changed => 1,
        EditResult::NotChanged => 0,
        EditResult::Failed => -1,
    }
}

/// Delete the character at the cursor (FFI).
///
/// # Safety
///
/// Calls C functions to manipulate the command buffer.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_cmdline_delete_char_at() -> c_int {
    match delete_char_at() {
        EditResult::Changed => 1,
        EditResult::NotChanged => 0,
        EditResult::Failed => -1,
    }
}

/// Delete the word before the cursor (FFI).
///
/// # Safety
///
/// Calls C functions to manipulate the command buffer.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_cmdline_delete_word_before() -> c_int {
    match delete_word_before() {
        EditResult::Changed => 1,
        EditResult::NotChanged => 0,
        EditResult::Failed => -1,
    }
}

/// Delete from cursor to start of line (FFI).
///
/// # Safety
///
/// Calls C functions to manipulate the command buffer.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_cmdline_delete_to_start() -> c_int {
    match delete_to_start() {
        EditResult::Changed => 1,
        EditResult::NotChanged => 0,
        EditResult::Failed => -1,
    }
}

/// Move cursor left by one character (FFI).
///
/// # Safety
///
/// Calls C functions to access the command buffer.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_cmdline_cursor_left() -> c_int {
    match cursor_left() {
        EditResult::Changed => 1,
        EditResult::NotChanged => 0,
        EditResult::Failed => -1,
    }
}

/// Move cursor right by one character (FFI).
///
/// # Safety
///
/// Calls C functions to access the command buffer.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_cmdline_cursor_right() -> c_int {
    match cursor_right() {
        EditResult::Changed => 1,
        EditResult::NotChanged => 0,
        EditResult::Failed => -1,
    }
}

/// Move cursor to the beginning of the line (FFI).
///
/// # Safety
///
/// Calls C functions to access the command buffer.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_cmdline_cursor_home() -> c_int {
    match cursor_home() {
        EditResult::Changed => 1,
        EditResult::NotChanged => 0,
        EditResult::Failed => -1,
    }
}

/// Move cursor to the end of the line (FFI).
///
/// # Safety
///
/// Calls C functions to access the command buffer.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_cmdline_cursor_end() -> c_int {
    match cursor_end() {
        EditResult::Changed => 1,
        EditResult::NotChanged => 0,
        EditResult::Failed => -1,
    }
}

/// Move cursor left by one word (FFI).
///
/// # Safety
///
/// Calls C functions to access the command buffer.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_cmdline_cursor_word_left() -> c_int {
    match cursor_word_left() {
        EditResult::Changed => 1,
        EditResult::NotChanged => 0,
        EditResult::Failed => -1,
    }
}

/// Move cursor right by one word (FFI).
///
/// # Safety
///
/// Calls C functions to access the command buffer.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_cmdline_cursor_word_right() -> c_int {
    match cursor_word_right() {
        EditResult::Changed => 1,
        EditResult::NotChanged => 0,
        EditResult::Failed => -1,
    }
}

/// Insert a string at the current cursor position (FFI).
///
/// # Safety
///
/// `s` must be a valid pointer to `len` bytes.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_cmdline_insert_str(s: *const c_char, len: usize) -> c_int {
    if s.is_null() {
        return -1;
    }

    let slice = std::slice::from_raw_parts(s.cast::<u8>(), len);
    match insert_str(slice) {
        EditResult::Changed => 1,
        EditResult::NotChanged => 0,
        EditResult::Failed => -1,
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_edit_result() {
        assert_ne!(EditResult::Changed, EditResult::NotChanged);
        assert_ne!(EditResult::Changed, EditResult::Failed);
        assert_ne!(EditResult::NotChanged, EditResult::Failed);
    }
}
