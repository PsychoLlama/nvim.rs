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
    static Rows: c_int;
    static Columns: c_int;
    // Buffer management
    fn nvim_get_ccline_cmdbuff() -> *mut c_char;
    fn nvim_get_ccline_cmdbufflen() -> c_int;
    fn nvim_get_ccline_cmdpos() -> c_int;
    fn nvim_get_ccline_cmdlen() -> c_int;
    fn nvim_set_ccline_cmdpos(pos: c_int);
    fn nvim_set_ccline_cmdlen(len: c_int);
    fn nvim_get_ccline_cmdspos() -> c_int;
    fn nvim_set_ccline_cmdspos(val: c_int);
    fn nvim_get_ccline_overstrike() -> c_int;

    // Reallocation (calls C realloc_cmdbuff)
    fn realloc_cmdbuff(len: c_int) -> c_int;

    // Multibyte utilities
    fn mb_char2len(c: c_int) -> c_int;
    fn utf_char2bytes(c: c_int, buf: *mut c_char) -> c_int;
    fn mb_get_class(p: *const c_char) -> c_int;
    fn mb_prevptr(start: *const c_char, p: *const c_char) -> *mut c_char;
    fn utfc_ptr2len(p: *const c_char) -> c_int;
    fn utf_head_off(base: *const c_char, p: *const c_char) -> c_int;

    // Redraw
    fn redrawcmd();

    // Screen
    fn cmd_screencol(bytepos: c_int) -> c_int;
    fn rs_cmdline_charsize(idx: c_int) -> c_int;
    fn cursorcmd();
    fn correct_screencol(idx: c_int, cells: c_int, col: *mut c_int);
    fn draw_cmdline(start: c_int, len: c_int);
    fn msg_clr_eos();
    static cmd_silent: bool;
    static mut cmdline_row: c_int;

    // Globals
    static mut KeyTyped: bool;
    static mut msg_no_more: bool;
    fn msg_check();

    // Abbreviation support
    fn nvim_get_p_paste() -> c_int;
    fn nvim_get_no_abbr() -> c_int;
    fn check_abbr(c: c_int, ptr: *mut c_char, col: c_int, mincol: c_int) -> c_int;

    // Paste support
    fn mb_cptr2char_adv(pp: *mut *const c_char) -> c_int;
    fn stuffcharReadbuff(c: c_int);

    // putcmdline / unputcmdline support
    fn ui_has(what: c_int) -> c_int;
    fn msg_putchar(c: c_int);
    fn ui_cursor_shape();
    fn nvim_get_ccline_level() -> c_int;
    fn nvim_get_ccline_redraw_state() -> c_int;
    fn nvim_set_ccline_special_char(c: c_int);
    fn nvim_set_ccline_special_shift(shift: c_int);
    fn nvim_ui_cmdline_special_char(c: c_int, shift: bool, level: c_int);

    // cmdline_paste support
    static mut got_int: bool;
    fn valid_yank_reg(regname: c_int, writing: bool) -> bool;
    fn get_spec_reg(
        regname: c_int,
        argp: *mut *mut c_char,
        allocated: *mut bool,
        errmsg: bool,
    ) -> bool;
    fn cmdline_paste_reg(regname: c_int, literally: bool, remcr: bool) -> bool;
    fn line_breakcheck();
    fn nvim_inc_textlock();
    fn nvim_dec_textlock();
    fn nvim_get_p_is() -> c_int;
    fn nvim_get_p_ic() -> c_int;
    fn utf_ptr2char(p: *const c_char) -> c_int;
    fn vim_iswordc(c: c_int) -> bool;
    fn xfree(ptr: *mut c_char);
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
// Paste String
// =============================================================================

/// Control character constants
const CTRL_V: c_int = 22;
const CTRL_BSL: c_int = 28;
const CTRL_C: c_int = 3;
const CTRL_L: c_int = 12;
const CTRL_N: c_int = 14;
const ESC: c_int = 27;
const CAR: c_int = 13;
const NL: c_int = 10;

/// Put a string on the command line.
///
/// When `literally` is true, insert literally.
/// When `literally` is false, insert as typed, but don't leave the command line.
///
/// Rust equivalent of `cmdline_paste_str()` from ex_getln.c.
///
/// # Safety
///
/// `s` must be a valid NUL-terminated string.
#[unsafe(export_name = "cmdline_paste_str")]
pub unsafe extern "C" fn rs_cmdline_paste_str(s: *const c_char, literally: bool) {
    if literally {
        put_on_cmdline_rs(s, -1, true);
    } else {
        let mut p = s;
        while *p != NUL as c_char {
            let cv = (*p as u8) as c_int;
            if cv == CTRL_V && (*p.add(1) != NUL as c_char) {
                p = p.add(1);
            }
            let c = mb_cptr2char_adv(&raw mut p);
            if cv == CTRL_V
                || c == ESC
                || c == CTRL_C
                || c == CAR
                || c == NL
                || c == CTRL_L
                || (c == CTRL_BSL && *p == CTRL_N as c_char)
            {
                stuffcharReadbuff(CTRL_V);
            }
            stuffcharReadbuff(c);
        }
    }
}

// =============================================================================
// putcmdline / unputcmdline
// =============================================================================

/// kUICmdline UIExtension value
const K_UI_CMDLINE: c_int = 0;
/// kCmdRedrawAll CmdRedraw value
const K_CMD_REDRAW_ALL: c_int = 2;

/// Put a special character on the command line.
///
/// Shifts the following text to the right when `shift` is true.
/// Used for CTRL-V, CTRL-K, etc. `c` must be printable (fit in one display cell).
///
/// Rust replacement for `putcmdline()` from ex_getln.c.
///
/// # Safety
///
/// Calls C functions to access ccline state and UI.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_putcmdline(c: c_int, shift: bool) {
    if cmd_silent {
        return;
    }
    if ui_has(K_UI_CMDLINE) == 0 {
        msg_no_more = true;
        msg_putchar(c);
        if shift {
            draw_cmdline(
                nvim_get_ccline_cmdpos(),
                nvim_get_ccline_cmdlen() - nvim_get_ccline_cmdpos(),
            );
        }
        msg_no_more = false;
    } else if nvim_get_ccline_redraw_state() != K_CMD_REDRAW_ALL {
        nvim_ui_cmdline_special_char(c, shift, nvim_get_ccline_level());
    }
    cursorcmd();
    nvim_set_ccline_special_char(c);
    nvim_set_ccline_special_shift(shift as c_int);
    ui_cursor_shape();
}

/// Undo a `putcmdline(c, false)`.
///
/// Rust replacement for `unputcmdline()` from ex_getln.c.
///
/// # Safety
///
/// Calls C functions to access ccline state and UI.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_unputcmdline() {
    if cmd_silent {
        return;
    }
    msg_no_more = true;
    let cmdpos = nvim_get_ccline_cmdpos();
    let cmdlen = nvim_get_ccline_cmdlen();
    let cmdbuff = nvim_get_ccline_cmdbuff();
    if cmdlen == cmdpos && ui_has(K_UI_CMDLINE) == 0 {
        msg_putchar(b' ' as c_int);
    } else {
        let char_len = utfc_ptr2len(cmdbuff.add(cmdpos as usize));
        draw_cmdline(cmdpos, char_len);
    }
    msg_no_more = false;
    cursorcmd();
    nvim_set_ccline_special_char(NUL as c_int);
    ui_cursor_shape();
}

// =============================================================================
// Register Paste
// =============================================================================

/// Ctrl-F, Ctrl-P, Ctrl-W, Ctrl-A, Ctrl-L special register codes
const CTRL_F: c_int = 6;
const CTRL_P: c_int = 16;
const CTRL_W: c_int = 23;
const CTRL_A: c_int = 1;

/// Paste a register into the command line.
///
/// Handles CTRL-R <regname>. When `regname` is CTRL-W the current word
/// under the cursor is used, deduplicating any overlap with what is already
/// on the command line when 'incsearch' is active.
///
/// Rust replacement for `nvim_cmdline_paste()` from ex_getln.c.
///
/// # Safety
///
/// Calls C functions to access globals and register content.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_cmdline_paste(regname: c_int, literally: bool, remcr: bool) -> bool {
    // Check for valid regname; also accept special codes used in the command line
    if regname != CTRL_F
        && regname != CTRL_P
        && regname != CTRL_W
        && regname != CTRL_A
        && regname != CTRL_L
        && !valid_yank_reg(regname, false)
    {
        return false;
    }

    // A register containing CTRL-R can cause an endless loop.
    // Allow CTRL-C to break it.
    line_breakcheck();
    if got_int {
        return false;
    }

    // Set textlock to avoid nasty things like jumping to another buffer
    // when evaluating an expression.
    let mut arg: *mut c_char = std::ptr::null_mut();
    let mut allocated = false;
    nvim_inc_textlock();
    let found = get_spec_reg(regname, &raw mut arg, &raw mut allocated, true);
    nvim_dec_textlock();

    if found {
        if arg.is_null() {
            return false;
        }

        // When 'incsearch' is set and CTRL-R CTRL-W is used: skip the
        // duplicate part of the word already on the command line.
        let mut p = arg;
        if nvim_get_p_is() != 0 && regname == CTRL_W {
            let cmdbuff = nvim_get_ccline_cmdbuff();
            let cmdpos = nvim_get_ccline_cmdpos();
            let mut w = cmdbuff.add(cmdpos as usize);
            while w > cmdbuff {
                let len = utf_head_off(cmdbuff, w.sub(1)) + 1;
                if !vim_iswordc(utf_ptr2char(w.sub(len as usize))) {
                    break;
                }
                w = w.sub(len as usize);
            }
            let len = cmdbuff.add(cmdpos as usize).offset_from(w) as c_int;
            let skip = if nvim_get_p_ic() != 0 {
                libc::strncasecmp(
                    w.cast::<libc::c_char>(),
                    arg.cast::<libc::c_char>(),
                    len as libc::size_t,
                ) == 0
            } else {
                libc::strncmp(
                    w.cast::<libc::c_char>(),
                    arg.cast::<libc::c_char>(),
                    len as libc::size_t,
                ) == 0
            };
            if skip {
                p = p.add(len as usize);
            }
        }

        rs_cmdline_paste_str(p, literally);
        if allocated {
            xfree(arg);
        }
        return true;
    }

    cmdline_paste_reg(regname, literally, remcr)
}

// =============================================================================
// Abbreviation Check
// =============================================================================

/// Check if there is an abbreviation to be expanded at the cursor.
///
/// Rust equivalent of `ccheck_abbr()` from ex_getln.c.
///
/// # Safety
///
/// Calls C functions to access ccline state and check abbreviations.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_ccheck_abbr(c: c_int) -> c_int {
    // No abbreviations in paste mode
    if nvim_get_p_paste() != 0 || nvim_get_no_abbr() != 0 {
        return 0;
    }

    let cmdbuff = nvim_get_ccline_cmdbuff();
    let cmdlen = nvim_get_ccline_cmdlen();
    let cmdpos = nvim_get_ccline_cmdpos();

    if cmdbuff.is_null() {
        return 0;
    }

    // Find start position: skip leading whitespace for '<,'> ranges
    let mut spos: c_int = 0;

    // Skip leading whitespace
    while spos < cmdlen {
        let ch = *cmdbuff.add(spos as usize) as u8;
        if ch != b' ' && ch != b'\t' {
            break;
        }
        spos += 1;
    }

    // Check if we have a '<,'> range marker (any mark like 'a,'b or '<,'>)
    if cmdlen - spos > 5
        && *cmdbuff.add(spos as usize) == b'\'' as c_char
        && *cmdbuff.add((spos + 2) as usize) == b',' as c_char
        && *cmdbuff.add((spos + 3) as usize) == b'\'' as c_char
    {
        spos += 5;
    } else {
        spos = 0;
    }

    check_abbr(c, cmdbuff, cmdpos, spos)
}

// =============================================================================
// put_on_cmdline: Insert a string into the command line buffer
// =============================================================================

/// Maximum column constant
const MAXCOL: c_int = 0x7FFF_FFFF;

/// Put the given string onto the command line.
///
/// If `len` is -1, `strlen` is used. If `redraw` is true, the new part and
/// remaining part are redrawn.
///
/// Direct replacement for C `put_on_cmdline()`.
///
/// # Safety
///
/// `str` must be a valid pointer to at least `len` bytes (or NUL-terminated if len == -1).
#[allow(clippy::too_many_lines)]
#[export_name = "put_on_cmdline"]
pub unsafe extern "C" fn put_on_cmdline_rs(str: *const c_char, mut len: c_int, redraw: bool) {
    if len < 0 {
        len = libc_strlen(str) as c_int;
    }

    let initial_cmdlen = nvim_get_ccline_cmdlen();
    if nvim_get_ccline_cmdbuff().is_null() {
        return;
    }

    realloc_cmdbuff(initial_cmdlen + len + 1);
    // Re-fetch after possible realloc
    let cmdbuff = nvim_get_ccline_cmdbuff();
    let pos0 = nvim_get_ccline_cmdpos();
    let buf_len = nvim_get_ccline_cmdlen();

    let overstrike = nvim_get_ccline_overstrike() != 0;
    if overstrike {
        // Overstrike mode: count chars being replaced
        let mut m: c_int = 0;
        let mut i: c_int = 0;
        while i < len {
            let char_len = utfc_ptr2len(str.add(i as usize));
            m += 1;
            i += char_len;
        }
        // Count bytes in cmdline overwritten
        i = pos0;
        while i < buf_len && m > 0 {
            let char_len = utfc_ptr2len(cmdbuff.add(i as usize));
            m -= 1;
            i += char_len;
        }
        let cur_len = nvim_get_ccline_cmdlen();
        if i < cur_len {
            std::ptr::copy(
                cmdbuff.add(i as usize),
                cmdbuff.add((pos0 + len) as usize),
                (cur_len - i) as usize,
            );
            nvim_set_ccline_cmdlen(cur_len + pos0 + len - i);
        } else {
            nvim_set_ccline_cmdlen(pos0 + len);
        }
    } else {
        // Insert mode: shift rest of buffer right
        std::ptr::copy(
            cmdbuff.add(pos0 as usize),
            cmdbuff.add((pos0 + len) as usize),
            (buf_len - pos0) as usize,
        );
        nvim_set_ccline_cmdlen(buf_len + len);
    }

    // Re-fetch after changes
    let cmdbuff = nvim_get_ccline_cmdbuff();
    let pos1 = nvim_get_ccline_cmdpos();
    let new_len = nvim_get_ccline_cmdlen();

    // Copy the new string in
    std::ptr::copy_nonoverlapping(str, cmdbuff.add(pos1 as usize), len as usize);
    // NUL-terminate
    *cmdbuff.add(new_len as usize) = 0;

    // When inserted text starts with a composing character, backup before it
    if pos1 > 0 && (*cmdbuff.add(pos1 as usize) as u8) >= 0x80 {
        let head = utf_head_off(cmdbuff, cmdbuff.add(pos1 as usize));
        if head != 0 {
            let new_pos = pos1 - head;
            nvim_set_ccline_cmdpos(new_pos);
            let new_cmdspos = cmd_screencol(new_pos);
            nvim_set_ccline_cmdspos(new_cmdspos);
        }
    }

    let cur_pos = nvim_get_ccline_cmdpos();

    if redraw && !cmd_silent {
        msg_no_more = true;
        let old_row = cmdline_row;
        cursorcmd();
        let draw_len = nvim_get_ccline_cmdlen();
        draw_cmdline(cur_pos, draw_len - cur_pos);
        // Avoid clearing the rest of the line too often
        if cmdline_row != old_row || nvim_get_ccline_overstrike() != 0 {
            msg_clr_eos();
        }
        msg_no_more = false;
    }

    let screen_limit = if KeyTyped {
        let cols = Columns;
        let rows = Rows;
        let prod = cols.saturating_mul(rows);
        if prod < 0 {
            MAXCOL
        } else {
            prod
        }
    } else {
        MAXCOL
    };

    let mut i: c_int = 0;
    let mut pos = nvim_get_ccline_cmdpos();
    let buf = nvim_get_ccline_cmdbuff();
    while i < len {
        let mut screen_pos = nvim_get_ccline_cmdspos();
        let c = rs_cmdline_charsize(pos);
        // count ">" for a double-wide char that doesn't fit
        correct_screencol(pos, c, &raw mut screen_pos);
        // Stop cursor at end of screen, but still advance position
        if screen_pos + c < screen_limit {
            screen_pos += c;
        }
        nvim_set_ccline_cmdspos(screen_pos);
        let char_len = utfc_ptr2len(buf.add(pos as usize)) - 1;
        let advance = char_len.min(len - i - 1);
        pos += advance;
        i += advance;
        pos += 1;
        i += 1;
    }
    nvim_set_ccline_cmdpos(pos);

    if redraw {
        msg_check();
    }
}

/// strlen wrapper (avoids importing libc)
const unsafe fn libc_strlen(s: *const c_char) -> usize {
    let mut len = 0;
    while *s.add(len) != 0 {
        len += 1;
    }
    len
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
