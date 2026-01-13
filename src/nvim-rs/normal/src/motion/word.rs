//! Word motion commands.
//!
//! This module provides helper functions for word motions:
//! - nv_word/bck/end (w/W, b/B, e/E)
//! - word boundary detection
//! - WORD vs word distinction

#![allow(clippy::missing_const_for_fn)]

use std::ffi::c_int;

// =============================================================================
// Word Class Constants
// =============================================================================

/// White space character class.
pub const CLASS_WHITE: c_int = 0;
/// Keyword character class (letters, digits, _).
pub const CLASS_WORD: c_int = 1;
/// Punctuation character class.
pub const CLASS_PUNCT: c_int = 2;

/// Big WORD mode (ignores keyword vs punct distinction).
pub const WORD_BIG: c_int = 1;
/// Small word mode (respects keyword vs punct).
pub const WORD_SMALL: c_int = 0;

// =============================================================================
// Word Motion Helpers (Pure Rust)
// =============================================================================

/// Check if character classes differ (boundary check).
fn classes_differ(class1: c_int, class2: c_int) -> bool {
    class1 != class2
}

/// Check if class is whitespace.
fn class_is_white(class: c_int) -> bool {
    class == CLASS_WHITE
}

/// Check if class is word.
fn class_is_word(class: c_int) -> bool {
    class == CLASS_WORD
}

/// Check if class is punctuation.
fn class_is_punct(class: c_int) -> bool {
    class == CLASS_PUNCT
}

/// Check if should stop at word end for 'e' motion.
fn stop_at_word_end(cur_class: c_int, next_class: c_int) -> bool {
    // Stop when current is non-white and next is white or different class
    cur_class != CLASS_WHITE && (next_class == CLASS_WHITE || next_class != cur_class)
}

// =============================================================================
// FFI Exports
// =============================================================================

/// FFI: Get CLASS_WHITE constant.
#[unsafe(no_mangle)]
pub extern "C" fn rs_motion_class_white() -> c_int {
    CLASS_WHITE
}

/// FFI: Get CLASS_WORD constant.
#[unsafe(no_mangle)]
pub extern "C" fn rs_motion_class_word() -> c_int {
    CLASS_WORD
}

/// FFI: Get CLASS_PUNCT constant.
#[unsafe(no_mangle)]
pub extern "C" fn rs_motion_class_punct() -> c_int {
    CLASS_PUNCT
}

/// FFI: Get WORD_BIG constant.
#[unsafe(no_mangle)]
pub extern "C" fn rs_motion_word_big() -> c_int {
    WORD_BIG
}

/// FFI: Get WORD_SMALL constant.
#[unsafe(no_mangle)]
pub extern "C" fn rs_motion_word_small() -> c_int {
    WORD_SMALL
}

/// FFI: Check if classes are different (boundary check).
#[unsafe(no_mangle)]
pub extern "C" fn rs_motion_classes_differ(class1: c_int, class2: c_int) -> c_int {
    c_int::from(classes_differ(class1, class2))
}

/// FFI: Check if class is whitespace.
#[unsafe(no_mangle)]
pub extern "C" fn rs_motion_class_is_white(class: c_int) -> c_int {
    c_int::from(class_is_white(class))
}

/// FFI: Check if class is word.
#[unsafe(no_mangle)]
pub extern "C" fn rs_motion_class_is_word(class: c_int) -> c_int {
    c_int::from(class_is_word(class))
}

/// FFI: Check if class is punctuation.
#[unsafe(no_mangle)]
pub extern "C" fn rs_motion_class_is_punct(class: c_int) -> c_int {
    c_int::from(class_is_punct(class))
}

/// FFI: Check if should stop at word end for 'e' motion.
#[unsafe(no_mangle)]
pub extern "C" fn rs_motion_stop_at_word_end(cur_class: c_int, next_class: c_int) -> c_int {
    c_int::from(stop_at_word_end(cur_class, next_class))
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_class_constants() {
        assert_eq!(CLASS_WHITE, 0);
        assert_eq!(CLASS_WORD, 1);
        assert_eq!(CLASS_PUNCT, 2);
    }

    #[test]
    fn test_word_constants() {
        assert_eq!(WORD_BIG, 1);
        assert_eq!(WORD_SMALL, 0);
    }

    #[test]
    fn test_classes_differ() {
        assert!(classes_differ(CLASS_WHITE, CLASS_WORD));
        assert!(!classes_differ(CLASS_WORD, CLASS_WORD));
        assert!(classes_differ(CLASS_PUNCT, CLASS_WORD));
    }

    #[test]
    fn test_class_checks() {
        assert!(class_is_white(CLASS_WHITE));
        assert!(!class_is_white(CLASS_WORD));
        assert!(class_is_word(CLASS_WORD));
        assert!(class_is_punct(CLASS_PUNCT));
    }

    #[test]
    fn test_stop_at_word_end() {
        // In word, next is white - stop
        assert!(stop_at_word_end(CLASS_WORD, CLASS_WHITE));
        // In word, next is punct - stop
        assert!(stop_at_word_end(CLASS_WORD, CLASS_PUNCT));
        // In word, next is word - don't stop
        assert!(!stop_at_word_end(CLASS_WORD, CLASS_WORD));
        // In white, anything - don't stop
        assert!(!stop_at_word_end(CLASS_WHITE, CLASS_WORD));
    }
}
