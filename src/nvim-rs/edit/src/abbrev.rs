//! Abbreviation system for insert mode.
//!
//! This module provides Rust infrastructure for abbreviation handling
//! in insert mode. The actual abbreviation lookup and expansion is
//! performed by the C code in `mapping.c`, but this module provides
//! helper functions for triggering and context management.

use std::ffi::c_int;

/// Abbreviation offset added to characters before checking abbreviations.
///
/// When characters like ESC, TAB, or multi-byte characters are checked for
/// abbreviations, `ABBR_OFF` is added to avoid special handling.
/// Multi-byte characters also have `ABBR_OFF` added, thus are above 0x0200.
pub const ABBR_OFF: i32 = 0x100;

// C accessor functions for abbreviation state.
extern "C" {
    // Global state
    fn nvim_get_no_abbr() -> c_int;
    fn nvim_get_p_paste() -> c_int;
    fn nvim_get_arrow_used() -> c_int;
}

/// Check if abbreviations are currently disabled.
///
/// Abbreviations are disabled when:
/// - `no_abbr` is true (no abbreviations loaded)
/// - Paste mode is active
/// - Arrow keys have been used (cursor moved)
#[inline]
#[must_use]
pub fn abbr_disabled() -> bool {
    // SAFETY: These are simple global accessors
    unsafe { nvim_get_p_paste() != 0 || nvim_get_no_abbr() != 0 || nvim_get_arrow_used() != 0 }
}

/// Check if abbreviations are loaded.
#[inline]
#[must_use]
pub fn abbr_loaded() -> bool {
    // SAFETY: Simple global accessor
    unsafe { nvim_get_no_abbr() == 0 }
}

/// Add abbreviation offset to a character for checking.
///
/// Used for special characters (ESC, TAB, etc.) and multi-byte characters
/// to avoid special handling during abbreviation lookup.
#[inline]
#[must_use]
pub const fn add_abbr_off(c: i32) -> i32 {
    c + ABBR_OFF
}

/// Remove abbreviation offset from a character.
///
/// Used to get the original character after abbreviation processing.
#[inline]
#[must_use]
pub const fn remove_abbr_off(c: i32) -> i32 {
    c - ABBR_OFF
}

/// Check if a character has the abbreviation offset applied.
#[inline]
#[must_use]
pub const fn has_abbr_off(c: i32) -> bool {
    c >= ABBR_OFF
}

/// Characters that trigger abbreviation expansion.
///
/// Non-word characters that follow a word character trigger abbreviation checking.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AbbrTrigger {
    /// Space character.
    Space,
    /// Tab character.
    Tab,
    /// Enter/newline.
    Enter,
    /// Escape key.
    Escape,
    /// Ctrl-] (explicit abbreviation expansion).
    CtrlRsb,
    /// Other non-word character.
    Other,
}

impl AbbrTrigger {
    /// Classify a character as an abbreviation trigger.
    ///
    /// Returns `Some(trigger)` if the character can trigger abbreviation expansion,
    /// `None` if it cannot (e.g., word characters).
    #[must_use]
    pub const fn from_char(c: i32) -> Option<Self> {
        // Ctrl-] always triggers abbreviation
        if c == 29 {
            // Ctrl_RSB
            return Some(Self::CtrlRsb);
        }

        // Space
        if c == 32 {
            return Some(Self::Space);
        }

        // Tab
        if c == 9 {
            return Some(Self::Tab);
        }

        // Enter (CR or NL)
        if c == 13 || c == 10 {
            return Some(Self::Enter);
        }

        // Escape
        if c == 27 {
            return Some(Self::Escape);
        }

        // Other punctuation and special chars trigger abbreviation
        // This is a simplified check - full check uses vim_iswordc()
        if c > 0 && c < 128 {
            // ASCII range - check if it's not alphanumeric or underscore
            let is_word_char = (c >= 48 && c <= 57)      // 0-9
                || (c >= 65 && c <= 90)                   // A-Z
                || (c >= 97 && c <= 122)                  // a-z
                || c == 95;                               // _
            if !is_word_char {
                return Some(Self::Other);
            }
        }

        None
    }

    /// Check if this trigger should have `ABBR_OFF` added.
    ///
    /// Some special triggers (ESC, TAB, etc.) need offset added.
    #[must_use]
    pub const fn needs_abbr_off(&self) -> bool {
        matches!(self, Self::Escape | Self::Tab | Self::Enter)
    }
}

/// Context for abbreviation checking.
#[derive(Debug, Clone, Copy)]
pub struct AbbrContext {
    /// The character that triggered the check.
    pub trigger_char: i32,
    /// Whether `ABBR_OFF` has been added to the trigger.
    pub has_offset: bool,
    /// Column position where insert started on this line.
    pub min_col: i32,
}

impl AbbrContext {
    /// Create a new abbreviation context.
    #[must_use]
    pub const fn new(trigger_char: i32, has_offset: bool, min_col: i32) -> Self {
        Self {
            trigger_char,
            has_offset,
            min_col,
        }
    }

    /// Get the character for abbreviation checking.
    ///
    /// Returns the trigger character with offset added if needed.
    #[must_use]
    pub const fn check_char(&self) -> i32 {
        if self.has_offset {
            self.trigger_char + ABBR_OFF
        } else {
            self.trigger_char
        }
    }

    /// Get the original character (without offset).
    #[must_use]
    pub const fn original_char(&self) -> i32 {
        if self.has_offset && self.trigger_char >= ABBR_OFF {
            self.trigger_char - ABBR_OFF
        } else {
            self.trigger_char
        }
    }
}

// FFI exports

/// FFI: Check if abbreviations are disabled.
#[no_mangle]
pub extern "C" fn rs_abbr_disabled() -> c_int {
    c_int::from(abbr_disabled())
}

/// FFI: Check if abbreviations are loaded.
#[no_mangle]
pub extern "C" fn rs_abbr_loaded() -> c_int {
    c_int::from(abbr_loaded())
}

/// FFI: Add abbreviation offset to character.
#[no_mangle]
pub const extern "C" fn rs_add_abbr_off(c: c_int) -> c_int {
    add_abbr_off(c)
}

/// FFI: Remove abbreviation offset from character.
#[no_mangle]
pub const extern "C" fn rs_remove_abbr_off(c: c_int) -> c_int {
    remove_abbr_off(c)
}

/// FFI: Check if character has abbreviation offset.
#[no_mangle]
pub const extern "C" fn rs_has_abbr_off(c: c_int) -> c_int {
    if has_abbr_off(c) { 1 } else { 0 }
}

/// FFI: Check if character can trigger abbreviation (returns trigger type or -1).
#[no_mangle]
pub const extern "C" fn rs_abbr_trigger_type(c: c_int) -> c_int {
    match AbbrTrigger::from_char(c) {
        Some(AbbrTrigger::Space) => 0,
        Some(AbbrTrigger::Tab) => 1,
        Some(AbbrTrigger::Enter) => 2,
        Some(AbbrTrigger::Escape) => 3,
        Some(AbbrTrigger::CtrlRsb) => 4,
        Some(AbbrTrigger::Other) => 5,
        None => -1,
    }
}

/// FFI: Check if trigger type needs `ABBR_OFF`.
#[no_mangle]
pub const extern "C" fn rs_abbr_trigger_needs_offset(trigger_type: c_int) -> c_int {
    match trigger_type {
        1..=3 => 1, // Tab (1), Enter (2), Escape (3) need offset
        _ => 0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_abbr_off_constant() {
        assert_eq!(ABBR_OFF, 0x100);
        assert_eq!(ABBR_OFF, 256);
    }

    #[test]
    fn test_add_abbr_off() {
        assert_eq!(add_abbr_off(27), 27 + ABBR_OFF); // ESC
        assert_eq!(add_abbr_off(9), 9 + ABBR_OFF); // TAB
        assert_eq!(add_abbr_off(i32::from(b'a')), i32::from(b'a') + ABBR_OFF);
    }

    #[test]
    fn test_remove_abbr_off() {
        assert_eq!(remove_abbr_off(27 + ABBR_OFF), 27);
        assert_eq!(remove_abbr_off(ABBR_OFF), 0);
    }

    #[test]
    fn test_has_abbr_off() {
        assert!(!has_abbr_off(27));
        assert!(!has_abbr_off(255));
        assert!(has_abbr_off(ABBR_OFF));
        assert!(has_abbr_off(ABBR_OFF + 1));
        assert!(has_abbr_off(500));
    }

    #[test]
    fn test_abbr_trigger_from_char() {
        // Word characters don't trigger
        assert_eq!(AbbrTrigger::from_char(i32::from(b'a')), None);
        assert_eq!(AbbrTrigger::from_char(i32::from(b'Z')), None);
        assert_eq!(AbbrTrigger::from_char(i32::from(b'5')), None);
        assert_eq!(AbbrTrigger::from_char(i32::from(b'_')), None);

        // Special triggers
        assert_eq!(AbbrTrigger::from_char(32), Some(AbbrTrigger::Space));
        assert_eq!(AbbrTrigger::from_char(9), Some(AbbrTrigger::Tab));
        assert_eq!(AbbrTrigger::from_char(13), Some(AbbrTrigger::Enter));
        assert_eq!(AbbrTrigger::from_char(10), Some(AbbrTrigger::Enter));
        assert_eq!(AbbrTrigger::from_char(27), Some(AbbrTrigger::Escape));
        assert_eq!(AbbrTrigger::from_char(29), Some(AbbrTrigger::CtrlRsb));

        // Other punctuation
        assert_eq!(AbbrTrigger::from_char(i32::from(b'.')), Some(AbbrTrigger::Other));
        assert_eq!(AbbrTrigger::from_char(i32::from(b',')), Some(AbbrTrigger::Other));
        assert_eq!(AbbrTrigger::from_char(i32::from(b'!')), Some(AbbrTrigger::Other));
    }

    #[test]
    fn test_abbr_trigger_needs_offset() {
        assert!(!AbbrTrigger::Space.needs_abbr_off());
        assert!(AbbrTrigger::Tab.needs_abbr_off());
        assert!(AbbrTrigger::Enter.needs_abbr_off());
        assert!(AbbrTrigger::Escape.needs_abbr_off());
        assert!(!AbbrTrigger::CtrlRsb.needs_abbr_off());
        assert!(!AbbrTrigger::Other.needs_abbr_off());
    }

    #[test]
    fn test_abbr_context() {
        // Without offset
        let ctx = AbbrContext::new(32, false, 0);
        assert_eq!(ctx.check_char(), 32);
        assert_eq!(ctx.original_char(), 32);

        // With offset
        let ctx = AbbrContext::new(27, true, 5);
        assert_eq!(ctx.check_char(), 27 + ABBR_OFF);
        assert_eq!(ctx.original_char(), 27);
        assert_eq!(ctx.min_col, 5);
    }
}
