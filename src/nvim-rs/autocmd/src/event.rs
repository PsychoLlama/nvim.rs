//! Autocommand event management helpers
//!
//! This module provides helpers for working with autocommand events,
//! including event classification, matching, and execution control.

#![allow(clippy::missing_const_for_fn)]
#![allow(clippy::missing_safety_doc)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_possible_wrap)]
#![allow(clippy::cast_sign_loss)]

use std::ffi::c_int;

// =============================================================================
// Event Categories
// =============================================================================

/// Categories of autocommand events.
#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum EventCategory {
    /// Unknown or invalid event
    #[default]
    Unknown = 0,
    /// Buffer-related events (BufEnter, BufLeave, etc.)
    Buffer = 1,
    /// File-related events (FileRead, FileWrite, etc.)
    File = 2,
    /// Window-related events (WinEnter, WinLeave, etc.)
    Window = 3,
    /// Tab-related events (TabEnter, TabLeave, etc.)
    Tab = 4,
    /// Cursor-related events (CursorMoved, CursorHold, etc.)
    Cursor = 5,
    /// Insert mode events (InsertEnter, InsertLeave, etc.)
    Insert = 6,
    /// Command-line events (CmdlineEnter, CmdlineLeave, etc.)
    Cmdline = 7,
    /// Terminal events (TermOpen, TermClose, etc.)
    Terminal = 8,
    /// UI events (ColorScheme, VimResized, etc.)
    Ui = 9,
    /// Session events (VimEnter, VimLeave, etc.)
    Session = 10,
    /// Text change events (TextChanged, TextYankPost, etc.)
    TextChange = 11,
    /// Completion events (CompleteChanged, CompleteDone, etc.)
    Completion = 12,
    /// User-defined events
    User = 13,
}

impl EventCategory {
    /// Convert from raw integer.
    #[must_use]
    pub const fn from_raw(value: c_int) -> Self {
        match value {
            1 => Self::Buffer,
            2 => Self::File,
            3 => Self::Window,
            4 => Self::Tab,
            5 => Self::Cursor,
            6 => Self::Insert,
            7 => Self::Cmdline,
            8 => Self::Terminal,
            9 => Self::Ui,
            10 => Self::Session,
            11 => Self::TextChange,
            12 => Self::Completion,
            13 => Self::User,
            _ => Self::Unknown,
        }
    }

    /// Convert to raw integer.
    #[must_use]
    pub const fn to_raw(self) -> c_int {
        self as c_int
    }
}

// =============================================================================
// Event Flags
// =============================================================================

/// Flags for autocommand event execution.
pub mod event_flags {
    use std::ffi::c_int;

    /// Event is currently being executed
    pub const AU_EXECUTING: c_int = 0x01;
    /// Event should be nested (allow triggering more autocmds)
    pub const AU_NESTED: c_int = 0x02;
    /// Event execution was blocked
    pub const AU_BLOCKED: c_int = 0x04;
    /// Event is from a pattern match
    pub const AU_PATTERN: c_int = 0x08;
    /// Event is buffer-local
    pub const AU_BUFLOCAL: c_int = 0x10;
    /// Event is once-only (delete after execution)
    pub const AU_ONCE: c_int = 0x20;
}

/// Check if event flags have a specific flag set.
#[must_use]
#[inline]
pub const fn has_event_flag(flags: c_int, flag: c_int) -> bool {
    (flags & flag) != 0
}

/// Set an event flag.
#[must_use]
#[inline]
pub const fn set_event_flag(flags: c_int, flag: c_int) -> c_int {
    flags | flag
}

/// Clear an event flag.
#[must_use]
#[inline]
pub const fn clear_event_flag(flags: c_int, flag: c_int) -> c_int {
    flags & !flag
}

// =============================================================================
// Event State
// =============================================================================

/// State for autocommand event execution.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct EventState {
    /// Current event number
    pub event: c_int,
    /// Event execution flags
    pub flags: c_int,
    /// Nesting level (how many autocmds deep)
    pub nesting_level: c_int,
    /// Buffer number for buffer-local events
    pub bufnr: c_int,
}

impl EventState {
    /// Create a new event state.
    #[must_use]
    pub const fn new(event: c_int) -> Self {
        Self {
            event,
            flags: 0,
            nesting_level: 0,
            bufnr: 0,
        }
    }

    /// Check if event is executing.
    #[must_use]
    pub const fn is_executing(&self) -> bool {
        has_event_flag(self.flags, event_flags::AU_EXECUTING)
    }

    /// Check if event is nested.
    #[must_use]
    pub const fn is_nested(&self) -> bool {
        has_event_flag(self.flags, event_flags::AU_NESTED)
    }

    /// Check if event is blocked.
    #[must_use]
    pub const fn is_blocked(&self) -> bool {
        has_event_flag(self.flags, event_flags::AU_BLOCKED)
    }

    /// Check if event is buffer-local.
    #[must_use]
    pub const fn is_buflocal(&self) -> bool {
        has_event_flag(self.flags, event_flags::AU_BUFLOCAL)
    }

    /// Check if event is once-only.
    #[must_use]
    pub const fn is_once(&self) -> bool {
        has_event_flag(self.flags, event_flags::AU_ONCE)
    }

    /// Set executing flag.
    pub fn set_executing(&mut self, executing: bool) {
        if executing {
            self.flags = set_event_flag(self.flags, event_flags::AU_EXECUTING);
        } else {
            self.flags = clear_event_flag(self.flags, event_flags::AU_EXECUTING);
        }
    }

    /// Set nested flag.
    pub fn set_nested(&mut self, nested: bool) {
        if nested {
            self.flags = set_event_flag(self.flags, event_flags::AU_NESTED);
        } else {
            self.flags = clear_event_flag(self.flags, event_flags::AU_NESTED);
        }
    }

    /// Increment nesting level.
    pub fn push_nesting(&mut self) {
        self.nesting_level += 1;
    }

    /// Decrement nesting level.
    pub fn pop_nesting(&mut self) {
        if self.nesting_level > 0 {
            self.nesting_level -= 1;
        }
    }
}

// =============================================================================
// Nesting Control
// =============================================================================

/// Maximum allowed nesting depth for autocommands.
pub const MAX_NESTING_DEPTH: c_int = 10;

/// Check if more nesting is allowed.
#[must_use]
pub const fn can_nest(current_depth: c_int) -> bool {
    current_depth < MAX_NESTING_DEPTH
}

/// Result of nesting check.
#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NestingResult {
    /// Nesting is allowed
    Allowed = 0,
    /// Nesting is not allowed (++nested flag not set)
    NotNested = 1,
    /// Nesting depth exceeded
    TooDeep = 2,
    /// Event is blocked
    Blocked = 3,
}

impl NestingResult {
    /// Check if nesting is allowed.
    #[must_use]
    pub const fn is_allowed(&self) -> bool {
        matches!(self, Self::Allowed)
    }
}

/// Check if an event can trigger nested autocommands.
#[must_use]
pub const fn check_nesting(state: &EventState) -> NestingResult {
    if state.is_blocked() {
        return NestingResult::Blocked;
    }
    if !state.is_nested() {
        return NestingResult::NotNested;
    }
    if !can_nest(state.nesting_level) {
        return NestingResult::TooDeep;
    }
    NestingResult::Allowed
}

// =============================================================================
// Event Matching
// =============================================================================

/// Result of event matching.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct EventMatch {
    /// Whether a match was found
    pub matched: bool,
    /// Event number that matched (if any)
    pub event: c_int,
    /// Match score (higher is better)
    pub score: c_int,
}

impl EventMatch {
    /// Create a non-match.
    #[must_use]
    pub const fn no_match() -> Self {
        Self {
            matched: false,
            event: -1,
            score: 0,
        }
    }

    /// Create a match.
    #[must_use]
    pub const fn matched(event: c_int, score: c_int) -> Self {
        Self {
            matched: true,
            event,
            score,
        }
    }
}

// =============================================================================
// FFI Exports
// =============================================================================

/// Get event category from event number.
#[unsafe(no_mangle)]
pub extern "C" fn rs_get_event_category(event: c_int) -> c_int {
    // Event ranges based on auevents_enum.generated.h
    // Buffer events: 0-20
    // File events: 21-35
    // Cursor events: 36-40
    // etc.
    let category = if event < 0 {
        EventCategory::Unknown
    } else if event <= 20 {
        EventCategory::Buffer
    } else if event <= 35 {
        EventCategory::File
    } else if event <= 40 {
        EventCategory::Cursor
    } else if event <= 50 {
        EventCategory::Insert
    } else if event <= 60 {
        EventCategory::Cmdline
    } else if event <= 70 {
        EventCategory::Window
    } else if event <= 80 {
        EventCategory::Tab
    } else if event <= 90 {
        EventCategory::Terminal
    } else if event <= 100 {
        EventCategory::TextChange
    } else if event <= 110 {
        EventCategory::Completion
    } else if event <= 130 {
        EventCategory::Session
    } else if event <= 140 {
        EventCategory::Ui
    } else {
        EventCategory::User
    };
    category.to_raw()
}

/// Check if event flags have a specific flag.
#[unsafe(no_mangle)]
pub extern "C" fn rs_event_has_flag(flags: c_int, flag: c_int) -> c_int {
    c_int::from(has_event_flag(flags, flag))
}

/// Set an event flag.
#[unsafe(no_mangle)]
pub extern "C" fn rs_event_set_flag(flags: c_int, flag: c_int) -> c_int {
    set_event_flag(flags, flag)
}

/// Clear an event flag.
#[unsafe(no_mangle)]
pub extern "C" fn rs_event_clear_flag(flags: c_int, flag: c_int) -> c_int {
    clear_event_flag(flags, flag)
}

/// Check if more nesting is allowed.
#[unsafe(no_mangle)]
pub extern "C" fn rs_can_nest(current_depth: c_int) -> c_int {
    c_int::from(can_nest(current_depth))
}

/// Get maximum nesting depth.
#[unsafe(no_mangle)]
pub extern "C" fn rs_max_nesting_depth() -> c_int {
    MAX_NESTING_DEPTH
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event_category() {
        assert_eq!(EventCategory::from_raw(0), EventCategory::Unknown);
        assert_eq!(EventCategory::from_raw(1), EventCategory::Buffer);
        assert_eq!(EventCategory::from_raw(13), EventCategory::User);
        assert_eq!(EventCategory::from_raw(99), EventCategory::Unknown);

        assert_eq!(EventCategory::Buffer.to_raw(), 1);
        assert_eq!(EventCategory::User.to_raw(), 13);
    }

    #[test]
    fn test_event_flags() {
        let flags = 0;
        assert!(!has_event_flag(flags, event_flags::AU_EXECUTING));

        let flags = set_event_flag(flags, event_flags::AU_EXECUTING);
        assert!(has_event_flag(flags, event_flags::AU_EXECUTING));

        let flags = set_event_flag(flags, event_flags::AU_NESTED);
        assert!(has_event_flag(flags, event_flags::AU_EXECUTING));
        assert!(has_event_flag(flags, event_flags::AU_NESTED));

        let flags = clear_event_flag(flags, event_flags::AU_EXECUTING);
        assert!(!has_event_flag(flags, event_flags::AU_EXECUTING));
        assert!(has_event_flag(flags, event_flags::AU_NESTED));
    }

    #[test]
    fn test_event_state() {
        let mut state = EventState::new(10);
        assert_eq!(state.event, 10);
        assert!(!state.is_executing());
        assert!(!state.is_nested());
        assert_eq!(state.nesting_level, 0);

        state.set_executing(true);
        assert!(state.is_executing());

        state.set_nested(true);
        assert!(state.is_nested());

        state.push_nesting();
        assert_eq!(state.nesting_level, 1);
        state.push_nesting();
        assert_eq!(state.nesting_level, 2);

        state.pop_nesting();
        assert_eq!(state.nesting_level, 1);
    }

    #[test]
    fn test_nesting() {
        assert!(can_nest(0));
        assert!(can_nest(9));
        assert!(!can_nest(10));
        assert!(!can_nest(100));
    }

    #[test]
    fn test_check_nesting() {
        let mut state = EventState::new(0);

        // Not nested by default
        assert_eq!(check_nesting(&state), NestingResult::NotNested);

        // Enable nesting
        state.set_nested(true);
        assert_eq!(check_nesting(&state), NestingResult::Allowed);

        // Too deep
        state.nesting_level = MAX_NESTING_DEPTH;
        assert_eq!(check_nesting(&state), NestingResult::TooDeep);

        // Blocked
        state.flags = set_event_flag(state.flags, event_flags::AU_BLOCKED);
        assert_eq!(check_nesting(&state), NestingResult::Blocked);
    }

    #[test]
    fn test_event_match() {
        let no_match = EventMatch::no_match();
        assert!(!no_match.matched);
        assert_eq!(no_match.event, -1);

        let matched = EventMatch::matched(42, 100);
        assert!(matched.matched);
        assert_eq!(matched.event, 42);
        assert_eq!(matched.score, 100);
    }
}
