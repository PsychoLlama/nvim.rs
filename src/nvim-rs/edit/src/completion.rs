//! Completion integration for insert mode.
//!
//! This module provides Rust infrastructure for completion triggering
//! and state management in insert mode. The actual completion logic
//! is in the `insexpand` module, but this provides helpers for
//! detecting completion triggers and managing completion state.

use std::ffi::c_int;

// C accessor functions for completion state.
extern "C" {
    fn nvim_get_compl_busy() -> c_int;
    // ctrl_x_mode is now a non-static C global (Phase 1 migration).
    #[link_name = "ctrl_x_mode"]
    static mut g_ctrl_x_mode: c_int;
}

/// CTRL-X mode values (from `insexpand.h`).
///
/// These represent different completion modes triggered by CTRL-X.
pub mod ctrl_x_modes {
    /// Not in CTRL-X mode.
    pub const CTRL_X_NORMAL: i32 = 0;
    /// CTRL-X CTRL-N/CTRL-P (scroll through matches).
    pub const CTRL_X_SCROLL: i32 = 1;
    /// CTRL-X mode (waiting for second key).
    pub const CTRL_X_NOT_DEFINED_YET: i32 = 2;
    /// CTRL-X CTRL-E/CTRL-Y (insert char from line above/below).
    pub const CTRL_X_WHOLE_LINE: i32 = 3;
    /// CTRL-X CTRL-F (filename completion).
    pub const CTRL_X_FILES: i32 = 4;
    /// CTRL-X CTRL-] (tag completion).
    pub const CTRL_X_TAGS: i32 = 5;
    /// CTRL-X CTRL-P (path pattern completion).
    pub const CTRL_X_PATH_PATTERNS: i32 = 6;
    /// CTRL-X CTRL-D (define/macro completion).
    pub const CTRL_X_PATH_DEFINES: i32 = 7;
    /// CTRL-X CTRL-I (complete from included files).
    pub const CTRL_X_FINISHED: i32 = 8;
    /// CTRL-X CTRL-K (dictionary completion).
    pub const CTRL_X_DICTIONARY: i32 = 9;
    /// CTRL-X CTRL-T (thesaurus completion).
    pub const CTRL_X_THESAURUS: i32 = 10;
    /// CTRL-X CTRL-L (whole line completion).
    pub const CTRL_X_CMDLINE: i32 = 11;
    /// CTRL-X CTRL-U (user-defined completion via 'completefunc').
    pub const CTRL_X_FUNCTION: i32 = 12;
    /// CTRL-X CTRL-O (omni completion).
    pub const CTRL_X_OMNI: i32 = 13;
    /// CTRL-X s (spelling suggestions).
    pub const CTRL_X_SPELL: i32 = 14;
    /// Local additions (reserved).
    pub const CTRL_X_LOCAL_MSG: i32 = 15;
    /// CTRL-X CTRL-V (Vim command completion).
    pub const CTRL_X_EVAL: i32 = 16;
    /// CTRL-X CTRL-R (register completion).
    pub const CTRL_X_REGISTER: i32 = 17;
}

/// Completion trigger type.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompletionTrigger {
    /// CTRL-N: Next match.
    CtrlN,
    /// CTRL-P: Previous match.
    CtrlP,
    /// CTRL-X: Enter completion mode.
    CtrlX,
    /// Character that may trigger auto-completion.
    AutoChar,
}

/// Check if completion is busy (preventing recursion).
#[inline]
#[must_use]
pub fn compl_busy() -> bool {
    // SAFETY: Simple global accessor
    unsafe { nvim_get_compl_busy() != 0 }
}

/// Get the current CTRL-X mode.
#[inline]
#[must_use]
pub fn ctrl_x_mode() -> i32 {
    // SAFETY: ctrl_x_mode is a C global; Neovim is single-threaded for completion.
    unsafe { g_ctrl_x_mode }
}

/// Check if we're in CTRL-X mode (any completion sub-mode).
#[inline]
#[must_use]
pub fn in_ctrl_x_mode() -> bool {
    ctrl_x_mode() != ctrl_x_modes::CTRL_X_NORMAL
}

/// Check if we're in CTRL-X normal scroll mode.
#[inline]
#[must_use]
pub fn ctrl_x_mode_scroll() -> bool {
    ctrl_x_mode() == ctrl_x_modes::CTRL_X_SCROLL
}

/// Check if we're in CTRL-X mode waiting for next key.
#[inline]
#[must_use]
pub fn ctrl_x_mode_not_defined() -> bool {
    ctrl_x_mode() == ctrl_x_modes::CTRL_X_NOT_DEFINED_YET
}

/// Check if we're in whole line completion mode.
#[inline]
#[must_use]
pub fn ctrl_x_mode_line() -> bool {
    ctrl_x_mode() == ctrl_x_modes::CTRL_X_WHOLE_LINE
}

/// Check if we're in filename completion mode.
#[inline]
#[must_use]
pub fn ctrl_x_mode_files() -> bool {
    ctrl_x_mode() == ctrl_x_modes::CTRL_X_FILES
}

/// Check if we're in tag completion mode.
#[inline]
#[must_use]
pub fn ctrl_x_mode_tags() -> bool {
    ctrl_x_mode() == ctrl_x_modes::CTRL_X_TAGS
}

/// Check if we're in path pattern completion mode.
#[inline]
#[must_use]
pub fn ctrl_x_mode_path_patterns() -> bool {
    ctrl_x_mode() == ctrl_x_modes::CTRL_X_PATH_PATTERNS
}

/// Check if we're in path defines completion mode.
#[inline]
#[must_use]
pub fn ctrl_x_mode_path_defines() -> bool {
    ctrl_x_mode() == ctrl_x_modes::CTRL_X_PATH_DEFINES
}

/// Check if we're in dictionary completion mode.
#[inline]
#[must_use]
pub fn ctrl_x_mode_dictionary() -> bool {
    ctrl_x_mode() == ctrl_x_modes::CTRL_X_DICTIONARY
}

/// Check if we're in thesaurus completion mode.
#[inline]
#[must_use]
pub fn ctrl_x_mode_thesaurus() -> bool {
    ctrl_x_mode() == ctrl_x_modes::CTRL_X_THESAURUS
}

/// Check if we're in user-defined function completion mode.
#[inline]
#[must_use]
pub fn ctrl_x_mode_function() -> bool {
    ctrl_x_mode() == ctrl_x_modes::CTRL_X_FUNCTION
}

/// Check if we're in omni completion mode.
#[inline]
#[must_use]
pub fn ctrl_x_mode_omni() -> bool {
    ctrl_x_mode() == ctrl_x_modes::CTRL_X_OMNI
}

/// Check if we're in spell completion mode.
#[inline]
#[must_use]
pub fn ctrl_x_mode_spell() -> bool {
    ctrl_x_mode() == ctrl_x_modes::CTRL_X_SPELL
}

/// Check if we're in Vim command completion mode.
#[inline]
#[must_use]
pub fn ctrl_x_mode_eval() -> bool {
    ctrl_x_mode() == ctrl_x_modes::CTRL_X_EVAL
}

/// Check if we're in register completion mode.
#[inline]
#[must_use]
pub fn ctrl_x_mode_register() -> bool {
    ctrl_x_mode() == ctrl_x_modes::CTRL_X_REGISTER
}

/// Check if a key can trigger completion.
///
/// Returns the trigger type if the key can trigger completion.
#[must_use]
pub const fn classify_completion_key(key: i32) -> Option<CompletionTrigger> {
    match key {
        14 => Some(CompletionTrigger::CtrlN), // CTRL-N
        16 => Some(CompletionTrigger::CtrlP), // CTRL-P
        24 => Some(CompletionTrigger::CtrlX), // CTRL-X
        _ => None,
    }
}

/// Check if a character might trigger auto-completion.
///
/// This is a simplified check - the full auto-completion triggering
/// depends on the 'complete' option and current context.
#[must_use]
pub const fn is_auto_completion_char(c: i32) -> bool {
    // Word characters can trigger keyword completion
    // ASCII letters
    (c >= 65 && c <= 90) || // A-Z
    (c >= 97 && c <= 122) || // a-z
    // Digits
    (c >= 48 && c <= 57) || // 0-9
    // Underscore
    c == 95 || // _
    // Non-ASCII (multi-byte) characters
    c >= 128
}

// FFI exports
//
// Note: Most CTRL-X mode checkers are in the insexpand crate to avoid duplication.
// This module only exports edit-specific completion functions.

/// FFI: Check if completion is busy.
#[no_mangle]
pub extern "C" fn rs_edit_compl_busy() -> c_int {
    c_int::from(compl_busy())
}

/// FFI: Check if in CTRL-X mode (any sub-mode).
#[no_mangle]
pub extern "C" fn rs_edit_in_ctrl_x_mode() -> c_int {
    c_int::from(in_ctrl_x_mode())
}

/// FFI: Classify key for completion (returns trigger type or -1).
#[no_mangle]
pub const extern "C" fn rs_edit_classify_completion_key(key: c_int) -> c_int {
    match classify_completion_key(key) {
        Some(CompletionTrigger::CtrlN) => 0,
        Some(CompletionTrigger::CtrlP) => 1,
        Some(CompletionTrigger::CtrlX) => 2,
        Some(CompletionTrigger::AutoChar) => 3,
        None => -1,
    }
}

/// FFI: Check if character can trigger auto-completion.
#[no_mangle]
pub const extern "C" fn rs_edit_is_auto_completion_char(c: c_int) -> c_int {
    if is_auto_completion_char(c) {
        1
    } else {
        0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ctrl_x_mode_constants() {
        assert_eq!(ctrl_x_modes::CTRL_X_NORMAL, 0);
        assert_eq!(ctrl_x_modes::CTRL_X_SCROLL, 1);
        assert_eq!(ctrl_x_modes::CTRL_X_NOT_DEFINED_YET, 2);
    }

    #[test]
    fn test_classify_completion_key() {
        assert_eq!(classify_completion_key(14), Some(CompletionTrigger::CtrlN));
        assert_eq!(classify_completion_key(16), Some(CompletionTrigger::CtrlP));
        assert_eq!(classify_completion_key(24), Some(CompletionTrigger::CtrlX));
        assert_eq!(classify_completion_key(i32::from(b'a')), None);
    }

    #[test]
    fn test_is_auto_completion_char() {
        // Letters
        assert!(is_auto_completion_char(i32::from(b'a')));
        assert!(is_auto_completion_char(i32::from(b'z')));
        assert!(is_auto_completion_char(i32::from(b'A')));
        assert!(is_auto_completion_char(i32::from(b'Z')));

        // Digits
        assert!(is_auto_completion_char(i32::from(b'0')));
        assert!(is_auto_completion_char(i32::from(b'9')));

        // Underscore
        assert!(is_auto_completion_char(i32::from(b'_')));

        // Non-word chars don't trigger
        assert!(!is_auto_completion_char(i32::from(b' ')));
        assert!(!is_auto_completion_char(i32::from(b'.')));
        assert!(!is_auto_completion_char(i32::from(b',')));

        // Multi-byte chars do trigger
        assert!(is_auto_completion_char(128));
        assert!(is_auto_completion_char(0x100));
    }
}
