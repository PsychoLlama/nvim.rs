//! Main syntax matching engine.
//!
//! This module provides the core syntax matching loop and line-based
//! syntax state management. It coordinates pattern matching, state
//! transitions, and highlight attribute assignment.

use std::ffi::c_int;

// =============================================================================
// Syntax engine state
// =============================================================================

/// Current state of the syntax engine for a line.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct EngineState {
    /// Current position in line (column).
    pub col: c_int,
    /// End column of current match.
    pub end_col: c_int,
    /// Current syntax ID being matched.
    pub syn_id: c_int,
    /// Highlight attribute ID.
    pub attr_id: c_int,
    /// Whether we're inside a match.
    pub in_match: bool,
    /// Whether state changed.
    pub state_changed: bool,
    /// Whether we're at end of line.
    pub at_eol: bool,
    /// Number of items pushed to stack.
    pub stack_depth: c_int,
}

impl EngineState {
    /// Create a new engine state.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            col: 0,
            end_col: 0,
            syn_id: 0,
            attr_id: 0,
            in_match: false,
            state_changed: false,
            at_eol: false,
            stack_depth: 0,
        }
    }

    /// Reset state for a new line.
    pub fn reset_for_line(&mut self, start_col: c_int) {
        self.col = start_col;
        self.end_col = 0;
        self.syn_id = 0;
        self.attr_id = 0;
        self.in_match = false;
        self.state_changed = true;
        self.at_eol = false;
    }

    /// Advance position.
    pub fn advance(&mut self, count: c_int) {
        self.col += count;
        if self.col >= self.end_col {
            self.in_match = false;
        }
    }

    /// Set current match.
    pub fn set_match(&mut self, syn_id: c_int, attr_id: c_int, end_col: c_int) {
        self.syn_id = syn_id;
        self.attr_id = attr_id;
        self.end_col = end_col;
        self.in_match = true;
        self.state_changed = true;
    }

    /// Clear current match.
    pub fn clear_match(&mut self) {
        self.syn_id = 0;
        self.attr_id = 0;
        self.in_match = false;
    }

    /// Check if we're in an active match.
    #[must_use]
    pub const fn has_match(&self) -> bool {
        self.in_match && self.syn_id > 0
    }

    /// Get highlight attribute for current position.
    #[must_use]
    pub const fn get_attr(&self) -> c_int {
        if self.in_match {
            self.attr_id
        } else {
            0
        }
    }
}

// =============================================================================
// Engine configuration
// =============================================================================

/// Configuration flags for the syntax engine.
pub mod engine_flags {
    use std::ffi::c_int;

    /// Enable spell checking integration.
    pub const SPELL: c_int = 0x01;
    /// Enable concealing.
    pub const CONCEAL: c_int = 0x02;
    /// Fold detection enabled.
    pub const FOLD: c_int = 0x04;
    /// Extend patterns to end of line.
    pub const EXTEND: c_int = 0x08;
    /// Skip blanks.
    pub const SKIPWHITE: c_int = 0x10;
    /// Skip newlines.
    pub const SKIPNL: c_int = 0x20;
    /// Skip empty lines.
    pub const SKIPEMPTY: c_int = 0x40;
}

/// Check if engine flag is set.
#[inline]
pub const fn has_engine_flag(flags: c_int, flag: c_int) -> bool {
    (flags & flag) != 0
}

/// Set engine flag.
#[inline]
pub const fn set_engine_flag(flags: c_int, flag: c_int) -> c_int {
    flags | flag
}

/// Clear engine flag.
#[inline]
pub const fn clear_engine_flag(flags: c_int, flag: c_int) -> c_int {
    flags & !flag
}

// =============================================================================
// Match result
// =============================================================================

/// Result of a syntax match attempt.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct MatchResult {
    /// Whether a match was found.
    pub found: bool,
    /// Syntax ID of the match.
    pub syn_id: c_int,
    /// Start column of match.
    pub start_col: c_int,
    /// End column of match.
    pub end_col: c_int,
    /// Highlight attribute ID.
    pub attr_id: c_int,
    /// Stack depth after match.
    pub new_depth: c_int,
    /// Whether match opens a region.
    pub opens_region: bool,
    /// Whether match closes a region.
    pub closes_region: bool,
}

impl MatchResult {
    /// Create a "no match" result.
    #[must_use]
    pub const fn no_match() -> Self {
        Self {
            found: false,
            syn_id: 0,
            start_col: 0,
            end_col: 0,
            attr_id: 0,
            new_depth: 0,
            opens_region: false,
            closes_region: false,
        }
    }

    /// Create a successful match result.
    #[must_use]
    pub const fn found(syn_id: c_int, start_col: c_int, end_col: c_int, attr_id: c_int) -> Self {
        Self {
            found: true,
            syn_id,
            start_col,
            end_col,
            attr_id,
            new_depth: 0,
            opens_region: false,
            closes_region: false,
        }
    }

    /// Set as region opener.
    pub fn as_region_start(&mut self) {
        self.opens_region = true;
    }

    /// Set as region closer.
    pub fn as_region_end(&mut self) {
        self.closes_region = true;
    }
}

impl Default for MatchResult {
    fn default() -> Self {
        Self::no_match()
    }
}

// =============================================================================
// Line context
// =============================================================================

/// Context for processing a line of syntax.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct LineContext {
    /// Line number (1-based).
    pub lnum: c_int,
    /// Line length in bytes.
    pub line_len: c_int,
    /// Current column being processed.
    pub col: c_int,
    /// Whether this line continues from previous.
    pub is_continuation: bool,
    /// Whether spell checking is enabled for this line.
    pub spell_check: bool,
    /// Fold level of this line.
    pub fold_level: c_int,
    /// Conceal level.
    pub conceal_level: c_int,
}

impl LineContext {
    /// Create a new line context.
    #[must_use]
    pub const fn new(lnum: c_int, line_len: c_int) -> Self {
        Self {
            lnum,
            line_len,
            col: 0,
            is_continuation: false,
            spell_check: false,
            fold_level: 0,
            conceal_level: 0,
        }
    }

    /// Check if at end of line.
    #[must_use]
    pub const fn at_eol(&self) -> bool {
        self.col >= self.line_len
    }

    /// Advance position.
    pub fn advance(&mut self, count: c_int) {
        self.col += count;
    }

    /// Reset to start of line.
    pub fn reset(&mut self) {
        self.col = 0;
    }
}

// =============================================================================
// FFI exports
// =============================================================================

/// Create a new engine state.
#[no_mangle]
pub extern "C" fn rs_syntax_engine_state_new() -> EngineState {
    EngineState::new()
}
/// Create match result indicating no match.
#[no_mangle]
pub extern "C" fn rs_match_result_no_match() -> MatchResult {
    MatchResult::no_match()
}

/// Create match result for a successful match.
#[no_mangle]
pub extern "C" fn rs_match_result_found(
    syn_id: c_int,
    start_col: c_int,
    end_col: c_int,
    attr_id: c_int,
) -> MatchResult {
    MatchResult::found(syn_id, start_col, end_col, attr_id)
}
/// Create a new line context.
#[no_mangle]
pub extern "C" fn rs_line_context_new(lnum: c_int, line_len: c_int) -> LineContext {
    LineContext::new(lnum, line_len)
}
/// Check engine flag.
#[no_mangle]
pub extern "C" fn rs_syntax_has_engine_flag(flags: c_int, flag: c_int) -> c_int {
    c_int::from(has_engine_flag(flags, flag))
}

/// Set engine flag.
#[no_mangle]
pub extern "C" fn rs_syntax_set_engine_flag(flags: c_int, flag: c_int) -> c_int {
    set_engine_flag(flags, flag)
}

/// Clear engine flag.
#[no_mangle]
pub extern "C" fn rs_syntax_clear_engine_flag(flags: c_int, flag: c_int) -> c_int {
    clear_engine_flag(flags, flag)
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_engine_state() {
        let mut state = EngineState::new();
        assert!(!state.has_match());
        assert_eq!(state.get_attr(), 0);

        state.set_match(5, 10, 20);
        assert!(state.has_match());
        assert_eq!(state.syn_id, 5);
        assert_eq!(state.get_attr(), 10);

        state.advance(15);
        assert!(state.has_match());

        state.advance(10);
        assert!(!state.has_match());
    }

    #[test]
    fn test_engine_reset() {
        let mut state = EngineState::new();
        state.set_match(5, 10, 20);
        state.stack_depth = 3;

        state.reset_for_line(5);
        assert!(!state.has_match());
        assert_eq!(state.col, 5);
        assert!(state.state_changed);
    }

    #[test]
    fn test_match_result() {
        let no_match = MatchResult::no_match();
        assert!(!no_match.found);

        let found = MatchResult::found(5, 10, 20, 15);
        assert!(found.found);
        assert_eq!(found.syn_id, 5);
        assert_eq!(found.start_col, 10);
        assert_eq!(found.end_col, 20);
        assert_eq!(found.attr_id, 15);
    }

    #[test]
    fn test_line_context() {
        let mut ctx = LineContext::new(10, 80);
        assert_eq!(ctx.lnum, 10);
        assert_eq!(ctx.line_len, 80);
        assert!(!ctx.at_eol());

        ctx.advance(40);
        assert!(!ctx.at_eol());
        assert_eq!(ctx.col, 40);

        ctx.advance(40);
        assert!(ctx.at_eol());
    }

    #[test]
    fn test_engine_flags() {
        let flags = 0;
        assert!(!has_engine_flag(flags, engine_flags::SPELL));

        let flags = set_engine_flag(flags, engine_flags::SPELL);
        assert!(has_engine_flag(flags, engine_flags::SPELL));
        assert!(!has_engine_flag(flags, engine_flags::CONCEAL));

        let flags = set_engine_flag(flags, engine_flags::CONCEAL);
        assert!(has_engine_flag(flags, engine_flags::SPELL));
        assert!(has_engine_flag(flags, engine_flags::CONCEAL));

        let flags = clear_engine_flag(flags, engine_flags::SPELL);
        assert!(!has_engine_flag(flags, engine_flags::SPELL));
        assert!(has_engine_flag(flags, engine_flags::CONCEAL));
    }
}
