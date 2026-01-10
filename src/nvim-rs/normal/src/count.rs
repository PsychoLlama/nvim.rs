//! Count and repeat handling helpers
//!
//! This module provides helpers for managing command counts,
//! repeat (.) command state, and command multipliers.

#![allow(clippy::missing_const_for_fn)]
#![allow(clippy::missing_safety_doc)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_possible_wrap)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::cast_lossless)]
#![allow(clippy::unreadable_literal)]

use std::ffi::c_int;

// =============================================================================
// Count Constants
// =============================================================================

/// Maximum count value (to prevent overflow).
pub const MAX_COUNT: c_int = 999999999;

/// Default count when not specified.
pub const DEFAULT_COUNT: c_int = 1;

// =============================================================================
// Count State
// =============================================================================

/// State for tracking command count.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct CountState {
    /// First count (before operator)
    pub count1: c_int,
    /// Second count (before motion)
    pub count2: c_int,
    /// Whether count1 was explicitly set
    pub count1_set: bool,
    /// Whether count2 was explicitly set
    pub count2_set: bool,
    /// Current digit accumulator
    pub accumulator: c_int,
    /// Whether we're reading digits
    pub reading: bool,
}

impl CountState {
    /// Create a new count state.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            count1: 0,
            count2: 0,
            count1_set: false,
            count2_set: false,
            accumulator: 0,
            reading: false,
        }
    }

    /// Start reading a count.
    pub fn start_reading(&mut self, digit: c_int) {
        self.reading = true;
        self.accumulator = digit;
    }

    /// Add a digit to the count.
    pub fn add_digit(&mut self, digit: c_int) -> bool {
        if !self.reading {
            return false;
        }

        // Check for overflow
        if self.accumulator > MAX_COUNT / 10 {
            self.accumulator = MAX_COUNT;
            return false;
        }

        self.accumulator = self.accumulator * 10 + digit;
        if self.accumulator > MAX_COUNT {
            self.accumulator = MAX_COUNT;
        }
        true
    }

    /// Finish reading and set count1.
    pub fn finish_count1(&mut self) {
        if self.reading && self.accumulator > 0 {
            self.count1 = self.accumulator;
            self.count1_set = true;
        }
        self.reading = false;
        self.accumulator = 0;
    }

    /// Finish reading and set count2.
    pub fn finish_count2(&mut self) {
        if self.reading && self.accumulator > 0 {
            self.count2 = self.accumulator;
            self.count2_set = true;
        }
        self.reading = false;
        self.accumulator = 0;
    }

    /// Get the effective count (count1 * count2, with defaults).
    #[must_use]
    pub const fn effective(&self) -> c_int {
        let c1 = if self.count1_set && self.count1 > 0 {
            self.count1
        } else {
            DEFAULT_COUNT
        };
        let c2 = if self.count2_set && self.count2 > 0 {
            self.count2
        } else {
            DEFAULT_COUNT
        };

        // Prevent overflow
        if c1 > MAX_COUNT / c2 {
            MAX_COUNT
        } else {
            c1 * c2
        }
    }

    /// Get count1 with default.
    #[must_use]
    pub const fn count1_or_default(&self) -> c_int {
        if self.count1_set && self.count1 > 0 {
            self.count1
        } else {
            DEFAULT_COUNT
        }
    }

    /// Get count2 with default.
    #[must_use]
    pub const fn count2_or_default(&self) -> c_int {
        if self.count2_set && self.count2 > 0 {
            self.count2
        } else {
            DEFAULT_COUNT
        }
    }

    /// Check if any count was set.
    #[must_use]
    pub const fn any_set(&self) -> bool {
        self.count1_set || self.count2_set
    }

    /// Clear the state.
    pub fn clear(&mut self) {
        *self = Self::new();
    }
}

// =============================================================================
// Repeat State
// =============================================================================

/// State for the repeat (.) command.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct RepeatState {
    /// The operator for repeat
    pub op_type: c_int,
    /// The motion character
    pub motion_char: c_int,
    /// Extra character (for text objects, etc.)
    pub extra_char: c_int,
    /// Count for repeat
    pub count: c_int,
    /// Register for repeat
    pub register: c_int,
    /// Whether repeat is valid
    pub valid: bool,
    /// Whether this was a line-wise operation
    pub linewise: bool,
}

impl RepeatState {
    /// Create a new repeat state.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            op_type: 0,
            motion_char: 0,
            extra_char: 0,
            count: 0,
            register: 0,
            valid: false,
            linewise: false,
        }
    }

    /// Set up for a simple command repeat.
    pub fn set_simple(&mut self, op: c_int, count: c_int) {
        self.op_type = op;
        self.motion_char = 0;
        self.extra_char = 0;
        self.count = count;
        self.valid = true;
    }

    /// Set up for an operator + motion repeat.
    pub fn set_motion(&mut self, op: c_int, motion: c_int, count: c_int) {
        self.op_type = op;
        self.motion_char = motion;
        self.extra_char = 0;
        self.count = count;
        self.valid = true;
    }

    /// Set up for a text object repeat.
    pub fn set_text_object(&mut self, op: c_int, prefix: c_int, obj: c_int, count: c_int) {
        self.op_type = op;
        self.motion_char = prefix; // 'i' or 'a'
        self.extra_char = obj; // 'w', 'W', etc.
        self.count = count;
        self.valid = true;
    }

    /// Check if repeat is valid.
    #[must_use]
    pub const fn is_valid(&self) -> bool {
        self.valid
    }

    /// Get the effective count (use provided or stored).
    #[must_use]
    pub const fn get_count(&self, provided: c_int) -> c_int {
        if provided > 0 {
            provided
        } else if self.count > 0 {
            self.count
        } else {
            DEFAULT_COUNT
        }
    }

    /// Clear the state.
    pub fn clear(&mut self) {
        *self = Self::new();
    }

    /// Invalidate (mark as not repeatable).
    pub fn invalidate(&mut self) {
        self.valid = false;
    }
}

// =============================================================================
// Command Repeat Buffer
// =============================================================================

/// Maximum size of repeat buffer.
pub const REPEAT_BUFFER_MAX: usize = 256;

/// State for buffering keystrokes for repeat.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct RepeatBuffer {
    /// Number of characters stored
    pub len: usize,
    /// Whether buffer is active (recording)
    pub recording: bool,
    /// Whether buffer is valid for replay
    pub valid: bool,
}

impl Default for RepeatBuffer {
    fn default() -> Self {
        Self::new()
    }
}

impl RepeatBuffer {
    /// Create a new repeat buffer.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            len: 0,
            recording: false,
            valid: false,
        }
    }

    /// Start recording.
    pub fn start_recording(&mut self) {
        self.len = 0;
        self.recording = true;
        self.valid = false;
    }

    /// Stop recording.
    pub fn stop_recording(&mut self) {
        self.recording = false;
        if self.len > 0 {
            self.valid = true;
        }
    }

    /// Add a character to the buffer.
    pub fn add_char(&mut self) -> bool {
        if !self.recording {
            return false;
        }
        if self.len >= REPEAT_BUFFER_MAX {
            return false;
        }
        self.len += 1;
        true
    }

    /// Check if buffer is valid for replay.
    #[must_use]
    pub const fn can_replay(&self) -> bool {
        self.valid && self.len > 0
    }

    /// Clear the buffer.
    pub fn clear(&mut self) {
        *self = Self::new();
    }
}

// =============================================================================
// Register for Repeat
// =============================================================================

/// Special register values.
pub mod registers {
    use std::ffi::c_int;

    /// No register specified
    pub const REG_NONE: c_int = -1;
    /// Unnamed register (")
    pub const REG_UNNAMED: c_int = 0;
    /// Black hole register (_)
    pub const REG_BLACKHOLE: c_int = b'_' as c_int;
    /// Expression register (=)
    pub const REG_EXPR: c_int = b'=' as c_int;
    /// Clipboard register (+)
    pub const REG_CLIPBOARD: c_int = b'+' as c_int;
    /// Selection register (*)
    pub const REG_SELECTION: c_int = b'*' as c_int;
    /// Last inserted text register (.)
    pub const REG_INSERT: c_int = b'.' as c_int;
}

/// Check if a register is valid.
#[must_use]
pub const fn is_valid_register(reg: c_int) -> bool {
    if reg == registers::REG_NONE {
        return true; // No register is valid
    }
    if reg >= b'a' as c_int && reg <= b'z' as c_int {
        return true; // a-z
    }
    if reg >= b'A' as c_int && reg <= b'Z' as c_int {
        return true; // A-Z (append)
    }
    if reg >= b'0' as c_int && reg <= b'9' as c_int {
        return true; // 0-9
    }
    matches!(
        reg,
        registers::REG_UNNAMED
            | registers::REG_BLACKHOLE
            | registers::REG_EXPR
            | registers::REG_CLIPBOARD
            | registers::REG_SELECTION
            | registers::REG_INSERT
    )
}

/// Check if a register appends (A-Z).
#[must_use]
pub const fn is_append_register(reg: c_int) -> bool {
    reg >= b'A' as c_int && reg <= b'Z' as c_int
}

// =============================================================================
// FFI Exports
// =============================================================================

/// Get effective count from count1 and count2.
#[unsafe(no_mangle)]
pub extern "C" fn rs_effective_count(count1: c_int, count2: c_int) -> c_int {
    let c1 = if count1 > 0 { count1 } else { 1 };
    let c2 = if count2 > 0 { count2 } else { 1 };
    if c1 > MAX_COUNT / c2 {
        MAX_COUNT
    } else {
        c1 * c2
    }
}

/// Check if register is valid.
#[unsafe(no_mangle)]
pub extern "C" fn rs_normal_is_valid_register(reg: c_int) -> c_int {
    c_int::from(is_valid_register(reg))
}

/// Check if register appends.
#[unsafe(no_mangle)]
pub extern "C" fn rs_normal_is_append_register(reg: c_int) -> c_int {
    c_int::from(is_append_register(reg))
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_count_state() {
        let mut state = CountState::new();
        assert!(!state.any_set());
        assert_eq!(state.effective(), 1);

        // Read count 5
        state.start_reading(5);
        assert!(state.reading);
        state.finish_count1();
        assert!(state.count1_set);
        assert_eq!(state.count1, 5);
        assert_eq!(state.effective(), 5);

        // Read count 3 for motion
        state.start_reading(3);
        state.finish_count2();
        assert!(state.count2_set);
        assert_eq!(state.count2, 3);
        assert_eq!(state.effective(), 15); // 5 * 3
    }

    #[test]
    fn test_count_digits() {
        let mut state = CountState::new();
        state.start_reading(1);
        state.add_digit(2);
        state.add_digit(3);
        state.finish_count1();
        assert_eq!(state.count1, 123);
    }

    #[test]
    fn test_count_overflow() {
        let mut state = CountState::new();
        state.start_reading(9);
        for _ in 0..20 {
            state.add_digit(9);
        }
        state.finish_count1();
        assert!(state.count1 <= MAX_COUNT);
    }

    #[test]
    fn test_repeat_state() {
        let mut state = RepeatState::new();
        assert!(!state.is_valid());

        state.set_simple(1, 5); // Delete with count 5
        assert!(state.is_valid());
        assert_eq!(state.op_type, 1);
        assert_eq!(state.count, 5);

        assert_eq!(state.get_count(0), 5); // Use stored
        assert_eq!(state.get_count(3), 3); // Use provided

        state.invalidate();
        assert!(!state.is_valid());
    }

    #[test]
    fn test_repeat_state_text_object() {
        let mut state = RepeatState::new();
        state.set_text_object(1, b'i' as c_int, b'w' as c_int, 2);
        assert!(state.is_valid());
        assert_eq!(state.motion_char, b'i' as c_int);
        assert_eq!(state.extra_char, b'w' as c_int);
    }

    #[test]
    fn test_repeat_buffer() {
        let mut buffer = RepeatBuffer::new();
        assert!(!buffer.can_replay());

        buffer.start_recording();
        assert!(buffer.recording);

        buffer.add_char();
        buffer.add_char();
        buffer.add_char();
        assert_eq!(buffer.len, 3);

        buffer.stop_recording();
        assert!(!buffer.recording);
        assert!(buffer.can_replay());
    }

    #[test]
    fn test_registers() {
        assert!(is_valid_register(registers::REG_NONE));
        assert!(is_valid_register(registers::REG_UNNAMED));
        assert!(is_valid_register(b'a' as c_int));
        assert!(is_valid_register(b'z' as c_int));
        assert!(is_valid_register(b'A' as c_int));
        assert!(is_valid_register(b'0' as c_int));
        assert!(is_valid_register(registers::REG_CLIPBOARD));

        assert!(!is_valid_register(b'!' as c_int));

        assert!(!is_append_register(b'a' as c_int));
        assert!(is_append_register(b'A' as c_int));
        assert!(is_append_register(b'Z' as c_int));
    }
}
