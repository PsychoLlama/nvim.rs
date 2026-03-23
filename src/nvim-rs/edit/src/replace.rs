//! Replace mode helpers and replace stack for edit mode
//!
//! This module owns the replace stack (`Vec<u8>`) which tracks replaced
//! characters during Replace and Virtual Replace modes. The stack is
//! accessed from C via `rs_replace_*` FFI functions.

#![allow(clippy::missing_const_for_fn)]
#![allow(clippy::missing_safety_doc)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_possible_wrap)]
#![allow(clippy::cast_sign_loss)]

use std::ffi::{c_char, c_int, c_void};
use std::sync::Mutex;

// =============================================================================
// Replace Mode Types
// =============================================================================

/// Replace mode types.
#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ReplaceMode {
    /// Normal insert mode (not replace)
    #[default]
    Insert = 0,
    /// Replace mode (R)
    Replace = 1,
    /// Virtual replace mode (gR)
    VirtualReplace = 2,
}

impl ReplaceMode {
    /// Convert from raw integer.
    #[must_use]
    pub const fn from_raw(value: c_int) -> Self {
        match value {
            1 => Self::Replace,
            2 => Self::VirtualReplace,
            _ => Self::Insert,
        }
    }

    /// Convert to raw integer.
    #[must_use]
    pub const fn to_raw(self) -> c_int {
        self as c_int
    }

    /// Check if in any replace mode.
    #[must_use]
    pub const fn is_replace(&self) -> bool {
        !matches!(self, Self::Insert)
    }

    /// Check if in virtual replace mode.
    #[must_use]
    pub const fn is_virtual(&self) -> bool {
        matches!(self, Self::VirtualReplace)
    }
}

// =============================================================================
// Replace Stack Entry
// =============================================================================

/// Entry in the replace stack (tracks replaced characters).
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct ReplaceEntry {
    /// Original character that was replaced
    pub orig_char: i32,
    /// Whether this was a multi-byte character
    pub is_multibyte: bool,
    /// Extra bytes for multi-byte characters
    pub extra_bytes: u8,
}

impl ReplaceEntry {
    /// Create a new replace entry for a single-byte character.
    #[must_use]
    pub const fn single(c: i32) -> Self {
        Self {
            orig_char: c,
            is_multibyte: false,
            extra_bytes: 0,
        }
    }

    /// Create a new replace entry for a multi-byte character.
    #[must_use]
    pub const fn multibyte(c: i32, extra: u8) -> Self {
        Self {
            orig_char: c,
            is_multibyte: true,
            extra_bytes: extra,
        }
    }

    /// Check if this is a NUL entry (no replacement).
    #[must_use]
    pub const fn is_nul(&self) -> bool {
        self.orig_char == 0 && !self.is_multibyte
    }

    /// Get total byte count for this entry.
    #[must_use]
    pub const fn byte_count(&self) -> usize {
        if self.is_multibyte {
            1 + self.extra_bytes as usize
        } else {
            1
        }
    }
}

// =============================================================================
// Replace State
// =============================================================================

/// Maximum size of replace stack.
pub const REPLACE_STACK_MAX: usize = 1024;

/// State for replace mode.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct ReplaceState {
    /// Current replace mode
    pub mode: c_int,
    /// Stack pointer (number of entries)
    pub stack_ptr: c_int,
    /// Column where replace started
    pub start_col: c_int,
    /// Virtual column for virtual replace
    pub vcol: c_int,
    /// Whether we've replaced past end of line
    pub past_eol: bool,
}

impl ReplaceState {
    /// Create a new replace state.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            mode: 0,
            stack_ptr: 0,
            start_col: 0,
            vcol: 0,
            past_eol: false,
        }
    }

    /// Get the replace mode.
    #[must_use]
    pub const fn get_mode(&self) -> ReplaceMode {
        ReplaceMode::from_raw(self.mode)
    }

    /// Check if in replace mode.
    #[must_use]
    pub const fn is_replace(&self) -> bool {
        self.mode != 0
    }

    /// Check if stack is empty.
    #[must_use]
    pub const fn stack_empty(&self) -> bool {
        self.stack_ptr == 0
    }

    /// Check if stack has room.
    #[must_use]
    pub const fn stack_has_room(&self) -> bool {
        (self.stack_ptr as usize) < REPLACE_STACK_MAX
    }

    /// Push to stack (increment pointer).
    pub fn stack_push(&mut self) {
        if self.stack_has_room() {
            self.stack_ptr += 1;
        }
    }

    /// Pop from stack (decrement pointer).
    pub fn stack_pop(&mut self) -> bool {
        if self.stack_ptr > 0 {
            self.stack_ptr -= 1;
            true
        } else {
            false
        }
    }

    /// Enter replace mode.
    pub fn enter(&mut self, mode: ReplaceMode, col: c_int) {
        self.mode = mode.to_raw();
        self.start_col = col;
        self.stack_ptr = 0;
        self.past_eol = false;
    }

    /// Exit replace mode.
    pub fn exit(&mut self) {
        self.mode = 0;
        self.stack_ptr = 0;
    }
}

// =============================================================================
// Virtual Replace Helpers
// =============================================================================

/// Calculate how many screen columns a character takes.
///
/// This is a simplified version - the full implementation uses
/// the actual character and 'tabstop' setting.
#[must_use]
pub const fn char_cells(c: i32, vcol: c_int, ts: c_int) -> c_int {
    if c == b'\t' as i32 {
        // Tab: expands to next tab stop
        ts - (vcol % ts)
    } else if c < 32 || c == 127 {
        // Control character: ^X format
        2
    } else {
        // Regular character
        1
    }
}

/// State for virtual replace column tracking.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct VirtualReplaceState {
    /// Current virtual column
    pub vcol: c_int,
    /// Virtual column at start of replaced character
    pub vcol_start: c_int,
    /// Number of screen columns being replaced
    pub replace_width: c_int,
}

impl VirtualReplaceState {
    /// Create a new virtual replace state.
    #[must_use]
    pub const fn new(vcol: c_int) -> Self {
        Self {
            vcol,
            vcol_start: vcol,
            replace_width: 0,
        }
    }

    /// Set the width of the character being replaced.
    pub fn set_replace_width(&mut self, width: c_int) {
        self.vcol_start = self.vcol;
        self.replace_width = width;
    }

    /// Advance virtual column by the given width.
    pub fn advance(&mut self, width: c_int) {
        self.vcol += width;
    }

    /// Check if replacement needs padding.
    #[must_use]
    pub const fn needs_padding(&self, new_width: c_int) -> bool {
        new_width < self.replace_width
    }

    /// Get padding needed for replacement.
    #[must_use]
    pub const fn padding_needed(&self, new_width: c_int) -> c_int {
        if new_width < self.replace_width {
            self.replace_width - new_width
        } else {
            0
        }
    }
}

// =============================================================================
// Replace Stack (Rust-owned)
// =============================================================================

/// The global replace stack, protected by a Mutex for safety.
/// In practice, Neovim is single-threaded so contention never occurs.
static REPLACE_STACK: Mutex<Vec<u8>> = Mutex::new(Vec::new());

/// Access the replace stack (panics if poisoned, which cannot happen
/// in single-threaded Neovim).
fn with_stack<F, R>(f: F) -> R
where
    F: FnOnce(&mut Vec<u8>) -> R,
{
    let mut stack = REPLACE_STACK.lock().unwrap();
    f(&mut stack)
}

// C accessor functions for replace_do_bs
extern "C" {
    static mut State: c_int;
    fn nvim_get_replace_offset() -> c_int;
    fn nvim_get_curwin() -> *mut c_void;
    fn nvim_curwin_get_cursor_col() -> i32;
    fn nvim_curwin_set_cursor_col(col: i32);
    fn nvim_get_cursor_pos_ptr() -> *const c_char;
    fn get_cursor_pos_ptr() -> *mut c_char;
    fn utf_head_off(base: *const c_char, p: *const c_char) -> c_int;
    fn ins_bytes_len(p: *const c_char, len: usize);
    fn dec_cursor() -> c_int;
    fn del_char(fixpos: c_int) -> c_int;
    fn getvcol(wp: *mut c_void, pos: *const c_void, scol: *mut i32, ccol: *mut i32, ecol: *mut i32);
    fn nvim_curwin_get_cursor_ptr() -> *const c_void;
    fn win_chartabsize(wp: *mut c_void, p: *const c_char, vcol: i32) -> c_int;
    fn nvim_cursor_get_pos_len() -> i32;
    fn gchar_cursor() -> c_int;
    fn changed_bytes(lnum: i32, col: i32);
    fn nvim_curwin_get_cursor_lnum() -> i32;
    fn del_char_after_col(limit_col: c_int) -> c_int;
    // Use *const u8 to match insert.rs declaration in same crate
    fn utfc_ptr2len(p: *const u8) -> c_int;
}

/// Constants verified by `_Static_assert` in `edit.c`.
const VREPLACE_FLAG: c_int = 0x200;
const MODE_NORMAL: c_int = 0x01;

// =============================================================================
// replace_push
// =============================================================================

/// Push bytes onto the replace stack.
///
/// `replace_offset` controls insertion point: 0 means append at end,
/// non-zero means insert that many bytes from the end.
unsafe fn replace_push_impl(data: *const u8, len: usize) {
    let offset = nvim_get_replace_offset() as usize;
    with_stack(|stack| {
        if stack.len() < offset {
            return; // nothing to do
        }
        let insert_pos = stack.len() - offset;
        // Make room
        stack.reserve(len);
        // Extend first so we have space
        let old_len = stack.len();
        stack.resize(old_len + len, 0);
        // Move the tail (offset bytes) to make room
        if offset > 0 {
            stack.copy_within(insert_pos..old_len, insert_pos + len);
        }
        // Copy new data in
        let src = std::slice::from_raw_parts(data, len);
        stack[insert_pos..insert_pos + len].copy_from_slice(src);
    });
}

/// Exported as the canonical C symbol, replacing the thin wrapper in `edit.c`.
#[unsafe(export_name = "replace_push")]
pub unsafe extern "C" fn rs_replace_push(str_ptr: *const c_char, len: usize) {
    replace_push_impl(str_ptr.cast::<u8>(), len);
}

// =============================================================================
// replace_push_nul
// =============================================================================

/// Push a NUL separator onto the replace stack.
#[unsafe(export_name = "replace_push_nul")]
pub unsafe extern "C" fn rs_replace_push_nul() {
    let nul: u8 = 0;
    replace_push_impl(std::ptr::addr_of!(nul), 1);
}

// =============================================================================
// replace_pop_if_nul
// =============================================================================

/// Check top of replace stack, pop it if it was NUL.
///
/// Returns -1 if stack is empty, the last byte otherwise.
/// If the last byte is NUL (0), it is popped.
fn replace_pop_if_nul_impl() -> c_int {
    with_stack(|stack| {
        if stack.is_empty() {
            return -1;
        }
        let ch = c_int::from(stack[stack.len() - 1]);
        if ch == 0 {
            stack.pop();
        }
        ch
    })
}

#[must_use]
#[unsafe(export_name = "replace_pop_if_nul")]
pub extern "C" fn rs_replace_pop_if_nul() -> c_int {
    replace_pop_if_nul_impl()
}

// =============================================================================
// replace_join
// =============================================================================

/// Join the top two items on the replace stack by removing the `off`-th NUL
/// encountered (searching from the end).
fn replace_join_impl(mut off: c_int) {
    with_stack(|stack| {
        let len = stack.len();
        for i in (0..len).rev() {
            if stack[i] == 0 {
                if off <= 0 {
                    stack.remove(i);
                    return;
                }
                off -= 1;
            }
        }
    });
}

#[unsafe(export_name = "replace_join")]
pub extern "C" fn rs_replace_join(off: c_int) {
    replace_join_impl(off);
}

// =============================================================================
// replace_pop_ins
// =============================================================================

/// Pop bytes from the replace stack until a NUL is found, and insert them
/// before the cursor. Temporarily sets State to `MODE_NORMAL`.
unsafe fn replace_pop_ins_impl() {
    let old_state = State;
    State = MODE_NORMAL;
    while replace_pop_if_nul_impl() > 0 {
        mb_replace_pop_ins_impl();
        dec_cursor();
    }
    State = old_state;
}

#[unsafe(export_name = "replace_pop_ins")]
pub unsafe extern "C" fn rs_replace_pop_ins() {
    replace_pop_ins_impl();
}

// =============================================================================
// mb_replace_pop_ins
// =============================================================================

/// Insert multibyte char popped from the replace stack.
///
/// The caller must have already checked the top of the stack is not NUL.
unsafe fn mb_replace_pop_ins_impl() {
    with_stack(|stack| {
        if stack.is_empty() {
            return;
        }
        let base = stack.as_ptr().cast::<c_char>();
        let last = base.add(stack.len() - 1);
        let head = utf_head_off(base, last);
        let len = (head + 1) as usize;
        let new_len = stack.len() - len;
        let ptr = stack.as_ptr().add(new_len).cast::<c_char>();
        ins_bytes_len(ptr, len);
        stack.truncate(new_len);
    });
}

#[unsafe(export_name = "mb_replace_pop_ins")]
pub unsafe extern "C" fn rs_mb_replace_pop_ins() {
    mb_replace_pop_ins_impl();
}

// =============================================================================
// replace_do_bs
// =============================================================================

/// Handle backspace for one replaced character.
///
/// - `cc < 0`: replace stack empty, just move cursor
/// - `cc == 0`: character was inserted, delete it
/// - `cc > 0`: character was replaced, put original char back
unsafe fn replace_do_bs_impl(limit_col: c_int) {
    let l_state = State;
    let cc = replace_pop_if_nul_impl();

    if cc > 0 {
        let is_vreplace = l_state & VREPLACE_FLAG != 0;

        let mut orig_vcols: c_int = if is_vreplace {
            let mut start_vcol: i32 = 0;
            getvcol(
                nvim_get_curwin(),
                nvim_curwin_get_cursor_ptr(),
                std::ptr::null_mut(),
                std::ptr::addr_of_mut!(start_vcol),
                std::ptr::null_mut(),
            );
            win_chartabsize(nvim_get_curwin(), get_cursor_pos_ptr(), start_vcol)
        } else {
            0
        };

        del_char_after_col(limit_col);

        let orig_len = if is_vreplace {
            nvim_cursor_get_pos_len()
        } else {
            0
        };

        replace_pop_ins_impl();

        if is_vreplace {
            let p = nvim_get_cursor_pos_ptr();
            let ins_len = nvim_cursor_get_pos_len() - orig_len;
            let mut start_vcol: i32 = 0;
            getvcol(
                nvim_get_curwin(),
                nvim_curwin_get_cursor_ptr(),
                std::ptr::null_mut(),
                std::ptr::addr_of_mut!(start_vcol),
                std::ptr::null_mut(),
            );
            let mut vcol = start_vcol;
            let mut i: c_int = 0;
            while i < ins_len {
                vcol += win_chartabsize(nvim_get_curwin(), p.add(i as usize), vcol);
                i += utfc_ptr2len(p.cast::<u8>()) - 1;
                i += 1;
            }
            vcol -= start_vcol;

            // Delete spaces that were inserted after cursor to keep text aligned
            let cursor_col = nvim_curwin_get_cursor_col();
            nvim_curwin_set_cursor_col(cursor_col + ins_len);
            while vcol > orig_vcols && gchar_cursor() == c_int::from(b' ') {
                del_char(0);
                orig_vcols += 1;
            }
            nvim_curwin_set_cursor_col(nvim_curwin_get_cursor_col() - ins_len);
        }

        changed_bytes(nvim_curwin_get_cursor_lnum(), nvim_curwin_get_cursor_col());
    } else if cc == 0 {
        del_char_after_col(limit_col);
    }
}

#[unsafe(export_name = "replace_do_bs")]
pub unsafe extern "C" fn rs_replace_do_bs(limit_col: c_int) {
    replace_do_bs_impl(limit_col);
}

// =============================================================================
// replace_stack_clear
// =============================================================================

/// Clear (destroy) the replace stack. Called from `stop_insert`.
#[unsafe(no_mangle)]
pub extern "C" fn rs_replace_stack_clear() {
    with_stack(|stack| {
        stack.clear();
        stack.shrink_to_fit();
    });
}

// =============================================================================
// FFI Exports (pre-existing)
// =============================================================================

/// Get replace mode from raw value.
#[unsafe(no_mangle)]
pub extern "C" fn rs_replace_mode(value: c_int) -> c_int {
    ReplaceMode::from_raw(value).to_raw()
}

/// Check if value indicates replace mode.
#[unsafe(no_mangle)]
pub extern "C" fn rs_is_replace_mode(value: c_int) -> c_int {
    c_int::from(ReplaceMode::from_raw(value).is_replace())
}

/// Check if value indicates virtual replace mode.
#[unsafe(no_mangle)]
pub extern "C" fn rs_is_virtual_replace(value: c_int) -> c_int {
    c_int::from(ReplaceMode::from_raw(value).is_virtual())
}

/// Calculate character cell width.
#[unsafe(no_mangle)]
pub extern "C" fn rs_char_cells(c: c_int, vcol: c_int, ts: c_int) -> c_int {
    char_cells(c, vcol, ts)
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_replace_mode() {
        assert_eq!(ReplaceMode::from_raw(0), ReplaceMode::Insert);
        assert_eq!(ReplaceMode::from_raw(1), ReplaceMode::Replace);
        assert_eq!(ReplaceMode::from_raw(2), ReplaceMode::VirtualReplace);
        assert_eq!(ReplaceMode::from_raw(99), ReplaceMode::Insert);

        assert!(!ReplaceMode::Insert.is_replace());
        assert!(ReplaceMode::Replace.is_replace());
        assert!(ReplaceMode::VirtualReplace.is_replace());

        assert!(!ReplaceMode::Replace.is_virtual());
        assert!(ReplaceMode::VirtualReplace.is_virtual());
    }

    #[test]
    fn test_replace_entry() {
        let single = ReplaceEntry::single(i32::from(b'x'));
        assert!(!single.is_multibyte);
        assert_eq!(single.byte_count(), 1);
        assert!(!single.is_nul());

        let multi = ReplaceEntry::multibyte(0xC3, 1);
        assert!(multi.is_multibyte);
        assert_eq!(multi.byte_count(), 2);

        let nul = ReplaceEntry::single(0);
        assert!(nul.is_nul());
    }

    #[test]
    fn test_replace_state() {
        let mut state = ReplaceState::new();
        assert!(!state.is_replace());
        assert!(state.stack_empty());

        state.enter(ReplaceMode::Replace, 10);
        assert!(state.is_replace());
        assert_eq!(state.start_col, 10);
        assert!(state.stack_empty());

        state.stack_push();
        assert!(!state.stack_empty());
        assert_eq!(state.stack_ptr, 1);

        assert!(state.stack_pop());
        assert!(state.stack_empty());

        assert!(!state.stack_pop()); // Can't pop from empty stack

        state.exit();
        assert!(!state.is_replace());
    }

    #[test]
    fn test_char_cells() {
        // Regular character
        assert_eq!(char_cells(i32::from(b'a'), 0, 8), 1);

        // Tab at column 0 with tabstop 8
        assert_eq!(char_cells(i32::from(b'\t'), 0, 8), 8);

        // Tab at column 5 with tabstop 8
        assert_eq!(char_cells(i32::from(b'\t'), 5, 8), 3);

        // Control character (e.g., ^A)
        assert_eq!(char_cells(1, 0, 8), 2);
    }

    #[test]
    fn test_virtual_replace_state() {
        let mut state = VirtualReplaceState::new(10);
        assert_eq!(state.vcol, 10);

        state.set_replace_width(4);
        assert_eq!(state.replace_width, 4);

        assert!(state.needs_padding(2));
        assert_eq!(state.padding_needed(2), 2);

        assert!(!state.needs_padding(5));
        assert_eq!(state.padding_needed(5), 0);

        state.advance(3);
        assert_eq!(state.vcol, 13);
    }

    #[test]
    fn test_replace_stack_push_pop() {
        // Clear any leftover state
        rs_replace_stack_clear();

        // Test push and pop_if_nul
        with_stack(|stack| {
            stack.push(b'A');
            stack.push(0); // NUL
            assert_eq!(stack.len(), 2);
        });

        // pop_if_nul should pop the NUL
        let ch = replace_pop_if_nul_impl();
        assert_eq!(ch, 0);
        with_stack(|stack| assert_eq!(stack.len(), 1));

        // pop_if_nul on non-NUL should NOT pop
        let ch = replace_pop_if_nul_impl();
        assert_eq!(ch, c_int::from(b'A'));
        with_stack(|stack| assert_eq!(stack.len(), 1));

        // pop_if_nul on empty should return -1
        with_stack(Vec::clear);
        let ch = replace_pop_if_nul_impl();
        assert_eq!(ch, -1);

        rs_replace_stack_clear();
    }

    #[test]
    fn test_replace_join() {
        rs_replace_stack_clear();

        // Build: [a, NUL, b, NUL, c]
        with_stack(|stack| {
            stack.extend_from_slice(&[b'a', 0, b'b', 0, b'c']);
        });

        // Join off=0 should remove the last NUL (index 3)
        replace_join_impl(0);
        with_stack(|stack| {
            assert_eq!(stack.as_slice(), &[b'a', 0, b'b', b'c']);
        });

        // Join off=0 again should remove the remaining NUL (index 1)
        replace_join_impl(0);
        with_stack(|stack| {
            assert_eq!(stack.as_slice(), b"abc");
        });

        rs_replace_stack_clear();
    }

    #[test]
    fn test_replace_stack_clear() {
        with_stack(|stack| {
            stack.extend_from_slice(b"hello");
        });
        rs_replace_stack_clear();
        with_stack(|stack| {
            assert!(stack.is_empty());
            assert_eq!(stack.capacity(), 0);
        });
    }

    #[test]
    fn test_constants() {
        assert_eq!(VREPLACE_FLAG, 0x200);
        assert_eq!(MODE_NORMAL, 0x01);
    }
}
