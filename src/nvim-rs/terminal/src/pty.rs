//! PTY interaction layer
//!
//! This module provides Rust implementations for PTY (pseudo-terminal) interaction,
//! including data sending, filtering, and terminal-to-process communication.

#![allow(clippy::must_use_candidate)]
#![allow(clippy::doc_markdown)]
#![allow(clippy::use_self)]
#![allow(clippy::derivable_impls)]
#![allow(clippy::return_self_not_must_use)]
#![allow(clippy::missing_const_for_fn)]

use std::ffi::c_void;
use std::os::raw::c_int;

// =============================================================================
// Opaque Handles
// =============================================================================

/// Opaque handle to Terminal struct.
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TerminalHandle(*mut c_void);

impl TerminalHandle {
    /// Check if the handle is null.
    #[inline]
    pub const fn is_null(self) -> bool {
        self.0.is_null()
    }

    /// Create a null handle.
    #[inline]
    pub const fn null() -> Self {
        Self(std::ptr::null_mut())
    }
}

// =============================================================================
// External C Functions
// =============================================================================

/// Helper: get shared reference to CTerminal from a handle.
/// # Safety: handle must be non-null and valid.
#[inline]
unsafe fn term_ref(term: TerminalHandle) -> &'static crate::CTerminal {
    unsafe { &*(term.0 as *const crate::CTerminal) }
}

// =============================================================================
// Character Filtering
// =============================================================================

/// Terminal paste flags for character filtering.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TermPasteFlags {
    flags: c_int,
}

/// Filter flag constants (must match TPF_* in terminal.c)
pub const TPF_BS: c_int = 0x001;
pub const TPF_HT: c_int = 0x002;
pub const TPF_FF: c_int = 0x004;
pub const TPF_ESC: c_int = 0x008;
pub const TPF_DEL: c_int = 0x010;
pub const TPF_C0: c_int = 0x020;
pub const TPF_C1: c_int = 0x040;

impl TermPasteFlags {
    /// Create with no filtering.
    pub const fn none() -> Self {
        Self { flags: 0 }
    }

    /// Create with all filters enabled.
    pub const fn all() -> Self {
        Self {
            flags: TPF_BS | TPF_HT | TPF_FF | TPF_ESC | TPF_DEL | TPF_C0 | TPF_C1,
        }
    }

    /// Create from raw flags.
    pub const fn from_raw(flags: c_int) -> Self {
        Self { flags }
    }

    /// Get raw flags value.
    pub const fn as_raw(self) -> c_int {
        self.flags
    }

    /// Check if backspace filtering is enabled.
    pub const fn filter_backspace(self) -> bool {
        (self.flags & TPF_BS) != 0
    }

    /// Check if horizontal tab filtering is enabled.
    pub const fn filter_tab(self) -> bool {
        (self.flags & TPF_HT) != 0
    }

    /// Check if form feed filtering is enabled.
    pub const fn filter_formfeed(self) -> bool {
        (self.flags & TPF_FF) != 0
    }

    /// Check if escape filtering is enabled.
    pub const fn filter_escape(self) -> bool {
        (self.flags & TPF_ESC) != 0
    }

    /// Check if DEL filtering is enabled.
    pub const fn filter_del(self) -> bool {
        (self.flags & TPF_DEL) != 0
    }

    /// Check if C0 control code filtering is enabled.
    pub const fn filter_c0(self) -> bool {
        (self.flags & TPF_C0) != 0
    }

    /// Check if C1 control code filtering is enabled.
    pub const fn filter_c1(self) -> bool {
        (self.flags & TPF_C1) != 0
    }
}

/// Check if a character should be filtered when pasting.
///
/// Returns true if the character should be filtered out.
pub const fn is_filter_char(c: c_int, flags: TermPasteFlags) -> bool {
    // Check specific characters first
    if c == 0x08 && flags.filter_backspace() {
        // BS (backspace)
        return true;
    }
    if c == 0x09 && flags.filter_tab() {
        // HT (horizontal tab)
        return true;
    }
    if c == 0x0C && flags.filter_formfeed() {
        // FF (form feed)
        return true;
    }
    if c == 0x1B && flags.filter_escape() {
        // ESC
        return true;
    }
    if c == 0x7F && flags.filter_del() {
        // DEL
        return true;
    }

    // C0 control codes (0x00-0x1F excluding already checked)
    if flags.filter_c0() && c >= 0x00 && c <= 0x1F {
        // Exclude BS, HT, FF, ESC which have their own flags
        if c != 0x08 && c != 0x09 && c != 0x0C && c != 0x1B {
            return true;
        }
    }

    // C1 control codes (0x80-0x9F)
    if flags.filter_c1() && c >= 0x80 && c <= 0x9F {
        return true;
    }

    false
}

// =============================================================================
// PTY Data Types
// =============================================================================

/// Result of sending data to the terminal.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SendResult {
    /// Data sent successfully.
    Success = 0,
    /// Terminal is null.
    NullTerminal = 1,
    /// Terminal is closed.
    Closed = 2,
    /// Data is empty.
    EmptyData = 3,
}

/// Result of a key send operation.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct KeySendResult {
    /// The result status.
    pub status: SendResult,
    /// Number of bytes that would be sent.
    pub bytes: c_int,
}

impl KeySendResult {
    /// Create a success result.
    pub const fn success(bytes: c_int) -> Self {
        Self {
            status: SendResult::Success,
            bytes,
        }
    }

    /// Create a failure result.
    pub const fn failure(status: SendResult) -> Self {
        Self { status, bytes: 0 }
    }
}

// =============================================================================
// Key Conversion
// =============================================================================

/// Modifier key flags for terminal input.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TermModifiers {
    flags: c_int,
}

pub const TERM_MOD_NONE: c_int = 0;
pub const TERM_MOD_SHIFT: c_int = 1;
pub const TERM_MOD_ALT: c_int = 2;
pub const TERM_MOD_CTRL: c_int = 4;

impl TermModifiers {
    /// No modifiers.
    pub const fn none() -> Self {
        Self { flags: 0 }
    }

    /// Create from raw flags.
    pub const fn from_raw(flags: c_int) -> Self {
        Self { flags }
    }

    /// Get raw flags value.
    pub const fn as_raw(self) -> c_int {
        self.flags
    }

    /// Check if shift is pressed.
    pub const fn has_shift(self) -> bool {
        (self.flags & TERM_MOD_SHIFT) != 0
    }

    /// Check if alt is pressed.
    pub const fn has_alt(self) -> bool {
        (self.flags & TERM_MOD_ALT) != 0
    }

    /// Check if ctrl is pressed.
    pub const fn has_ctrl(self) -> bool {
        (self.flags & TERM_MOD_CTRL) != 0
    }

    /// Add shift modifier.
    pub const fn with_shift(self) -> Self {
        Self {
            flags: self.flags | TERM_MOD_SHIFT,
        }
    }

    /// Add alt modifier.
    pub const fn with_alt(self) -> Self {
        Self {
            flags: self.flags | TERM_MOD_ALT,
        }
    }

    /// Add ctrl modifier.
    pub const fn with_ctrl(self) -> Self {
        Self {
            flags: self.flags | TERM_MOD_CTRL,
        }
    }
}

/// Convert a Neovim modifier key to terminal modifier.
///
/// This handles the conversion of Neovim's MOD_MASK_* to VTerm modifiers.
pub const fn convert_nvim_modifier(nvim_mod: c_int) -> TermModifiers {
    let mut flags = 0;

    // MOD_MASK_SHIFT = 0x02, MOD_MASK_CTRL = 0x04, MOD_MASK_ALT = 0x08
    if (nvim_mod & 0x02) != 0 {
        flags |= TERM_MOD_SHIFT;
    }
    if (nvim_mod & 0x04) != 0 {
        flags |= TERM_MOD_CTRL;
    }
    if (nvim_mod & 0x08) != 0 {
        flags |= TERM_MOD_ALT;
    }

    TermModifiers { flags }
}

// =============================================================================
// PTY State
// =============================================================================

/// PTY connection state.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PtyState {
    /// PTY is not connected.
    Disconnected = 0,
    /// PTY is connecting.
    Connecting = 1,
    /// PTY is connected and ready.
    Connected = 2,
    /// PTY is closing.
    Closing = 3,
    /// PTY is closed.
    Closed = 4,
}

/// Check if a terminal's PTY is ready for communication.
pub fn is_pty_ready(term: TerminalHandle) -> bool {
    if term.is_null() {
        return false;
    }
    !unsafe { term_ref(term).closed }
}

// =============================================================================
// FFI Exports
// =============================================================================

/// FFI export: Check if character should be filtered with specific flags.
#[no_mangle]
pub extern "C" fn rs_terminal_is_filter_char_flags(c: c_int, flags: c_int) -> c_int {
    c_int::from(is_filter_char(c, TermPasteFlags::from_raw(flags)))
}

/// FFI export: Convert Neovim modifier to terminal modifier.
#[no_mangle]
pub extern "C" fn rs_terminal_convert_modifier(nvim_mod: c_int) -> c_int {
    convert_nvim_modifier(nvim_mod).as_raw()
}

/// FFI export: Check if PTY is ready.
#[no_mangle]
pub extern "C" fn rs_terminal_is_pty_ready(term: TerminalHandle) -> c_int {
    c_int::from(is_pty_ready(term))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_term_paste_flags() {
        let none = TermPasteFlags::none();
        assert!(!none.filter_backspace());
        assert!(!none.filter_tab());
        assert!(!none.filter_escape());

        let all = TermPasteFlags::all();
        assert!(all.filter_backspace());
        assert!(all.filter_tab());
        assert!(all.filter_escape());
        assert!(all.filter_del());
        assert!(all.filter_c0());
        assert!(all.filter_c1());
    }

    #[test]
    fn test_is_filter_char() {
        let flags = TermPasteFlags::all();

        // Backspace
        assert!(is_filter_char(0x08, flags));
        // Tab
        assert!(is_filter_char(0x09, flags));
        // Escape
        assert!(is_filter_char(0x1B, flags));
        // DEL
        assert!(is_filter_char(0x7F, flags));
        // C1 control
        assert!(is_filter_char(0x80, flags));

        // Regular character should not be filtered
        assert!(!is_filter_char(0x41, flags)); // 'A'
        assert!(!is_filter_char(0x20, flags)); // space

        // With no flags, nothing filtered
        let none = TermPasteFlags::none();
        assert!(!is_filter_char(0x08, none));
        assert!(!is_filter_char(0x1B, none));
    }

    #[test]
    fn test_term_modifiers() {
        let none = TermModifiers::none();
        assert!(!none.has_shift());
        assert!(!none.has_alt());
        assert!(!none.has_ctrl());

        let shift = none.with_shift();
        assert!(shift.has_shift());
        assert!(!shift.has_alt());

        let ctrl_alt = TermModifiers::none().with_ctrl().with_alt();
        assert!(ctrl_alt.has_ctrl());
        assert!(ctrl_alt.has_alt());
        assert!(!ctrl_alt.has_shift());
    }

    #[test]
    fn test_convert_nvim_modifier() {
        // No modifiers
        let mods = convert_nvim_modifier(0);
        assert_eq!(mods.as_raw(), 0);

        // Shift (0x02)
        let mods = convert_nvim_modifier(0x02);
        assert!(mods.has_shift());

        // Ctrl (0x04)
        let mods = convert_nvim_modifier(0x04);
        assert!(mods.has_ctrl());

        // Alt (0x08)
        let mods = convert_nvim_modifier(0x08);
        assert!(mods.has_alt());

        // All
        let mods = convert_nvim_modifier(0x0E);
        assert!(mods.has_shift());
        assert!(mods.has_ctrl());
        assert!(mods.has_alt());
    }

    #[test]
    fn test_send_result_values() {
        assert_eq!(SendResult::Success as c_int, 0);
        assert_eq!(SendResult::NullTerminal as c_int, 1);
        assert_eq!(SendResult::Closed as c_int, 2);
        assert_eq!(SendResult::EmptyData as c_int, 3);
    }

    #[test]
    fn test_pty_state_values() {
        assert_eq!(PtyState::Disconnected as c_int, 0);
        assert_eq!(PtyState::Connecting as c_int, 1);
        assert_eq!(PtyState::Connected as c_int, 2);
        assert_eq!(PtyState::Closing as c_int, 3);
        assert_eq!(PtyState::Closed as c_int, 4);
    }

    #[test]
    fn test_key_send_result() {
        let success = KeySendResult::success(5);
        assert_eq!(success.status, SendResult::Success);
        assert_eq!(success.bytes, 5);

        let failure = KeySendResult::failure(SendResult::Closed);
        assert_eq!(failure.status, SendResult::Closed);
        assert_eq!(failure.bytes, 0);
    }
}
