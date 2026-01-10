//! Syntax attribute computation.
//!
//! This module provides utilities for computing syntax highlighting attributes,
//! including:
//! - Current attribute calculation based on state stack
//! - Conceal character handling
//! - Spell checking interaction
//! - Transparent group handling

use std::ffi::c_int;

// =============================================================================
// Constants
// =============================================================================

/// No syntax highlighting.
pub const SYN_ATTR_NONE: c_int = 0;

/// Conceal attribute flag.
pub const SYN_ATTR_CONCEAL: c_int = 0x01;

/// Spell checking enabled attribute.
pub const SYN_ATTR_SPELL: c_int = 0x02;

/// No spell checking attribute.
pub const SYN_ATTR_NOSPELL: c_int = 0x04;

/// Transparent attribute (inherit from parent).
pub const SYN_ATTR_TRANSPARENT: c_int = 0x08;

/// Combined spell mask.
pub const SYN_ATTR_SPELL_MASK: c_int = SYN_ATTR_SPELL | SYN_ATTR_NOSPELL;

// =============================================================================
// Attribute State
// =============================================================================

/// Current syntax attribute state.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct SynAttrState {
    /// Current highlight attribute ID.
    pub attr: c_int,
    /// Current syntax ID (for spell checking decisions).
    pub syn_id: c_int,
    /// Conceal character (if any).
    pub conceal_char: u32,
    /// Flags (SYN_ATTR_*).
    pub flags: c_int,
    /// Start column of current match.
    pub start_col: c_int,
    /// End column of current match.
    pub end_col: c_int,
}

impl SynAttrState {
    /// Create a new empty state.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            attr: 0,
            syn_id: 0,
            conceal_char: 0,
            flags: 0,
            start_col: 0,
            end_col: 0,
        }
    }

    /// Check if there's a current attribute.
    #[must_use]
    pub const fn has_attr(&self) -> bool {
        self.attr != 0
    }

    /// Check if conceal is active.
    #[must_use]
    pub const fn is_concealed(&self) -> bool {
        self.flags & SYN_ATTR_CONCEAL != 0
    }

    /// Check if spell checking is enabled.
    #[must_use]
    pub const fn spell_enabled(&self) -> bool {
        self.flags & SYN_ATTR_SPELL != 0
    }

    /// Check if spell checking is disabled.
    #[must_use]
    pub const fn spell_disabled(&self) -> bool {
        self.flags & SYN_ATTR_NOSPELL != 0
    }

    /// Check if this is a transparent group.
    #[must_use]
    pub const fn is_transparent(&self) -> bool {
        self.flags & SYN_ATTR_TRANSPARENT != 0
    }

    /// Set the conceal flag and character.
    pub fn set_conceal(&mut self, char: u32) {
        self.flags |= SYN_ATTR_CONCEAL;
        self.conceal_char = char;
    }

    /// Clear the conceal flag.
    pub fn clear_conceal(&mut self) {
        self.flags &= !SYN_ATTR_CONCEAL;
        self.conceal_char = 0;
    }

    /// Set spell checking enabled.
    pub fn enable_spell(&mut self) {
        self.flags = (self.flags & !SYN_ATTR_SPELL_MASK) | SYN_ATTR_SPELL;
    }

    /// Set spell checking disabled.
    pub fn disable_spell(&mut self) {
        self.flags = (self.flags & !SYN_ATTR_SPELL_MASK) | SYN_ATTR_NOSPELL;
    }

    /// Clear spell flags.
    pub fn clear_spell(&mut self) {
        self.flags &= !SYN_ATTR_SPELL_MASK;
    }
}

// =============================================================================
// Attribute Stack
// =============================================================================

/// Maximum nesting depth for attribute computation.
pub const MAX_ATTR_STACK: usize = 64;

/// Stack of syntax attributes for nested regions.
#[repr(C)]
#[derive(Debug, Clone)]
pub struct SynAttrStack {
    /// Stack entries.
    entries: Vec<SynAttrState>,
    /// Current top (number of entries).
    top: usize,
}

impl Default for SynAttrStack {
    fn default() -> Self {
        Self::new()
    }
}

impl SynAttrStack {
    /// Create a new empty stack.
    #[must_use]
    pub fn new() -> Self {
        Self {
            entries: Vec::with_capacity(MAX_ATTR_STACK),
            top: 0,
        }
    }

    /// Clear the stack.
    pub fn clear(&mut self) {
        self.entries.clear();
        self.top = 0;
    }

    /// Push an attribute state onto the stack.
    pub fn push(&mut self, state: SynAttrState) -> bool {
        if self.top >= MAX_ATTR_STACK {
            return false;
        }
        if self.top >= self.entries.len() {
            self.entries.push(state);
        } else {
            self.entries[self.top] = state;
        }
        self.top += 1;
        true
    }

    /// Pop an attribute state from the stack.
    pub fn pop(&mut self) -> Option<SynAttrState> {
        if self.top == 0 {
            return None;
        }
        self.top -= 1;
        Some(self.entries[self.top])
    }

    /// Get the current top state.
    #[must_use]
    pub fn current(&self) -> Option<&SynAttrState> {
        if self.top == 0 {
            None
        } else {
            Some(&self.entries[self.top - 1])
        }
    }

    /// Get the current top state mutably.
    pub fn current_mut(&mut self) -> Option<&mut SynAttrState> {
        if self.top == 0 {
            None
        } else {
            Some(&mut self.entries[self.top - 1])
        }
    }

    /// Get the stack depth.
    #[must_use]
    pub const fn depth(&self) -> usize {
        self.top
    }

    /// Check if stack is empty.
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.top == 0
    }

    /// Compute effective attribute considering transparency.
    ///
    /// Walks up the stack to find a non-transparent attribute.
    #[must_use]
    pub fn effective_attr(&self) -> c_int {
        for i in (0..self.top).rev() {
            let state = &self.entries[i];
            if !state.is_transparent() && state.has_attr() {
                return state.attr;
            }
        }
        0
    }

    /// Compute effective spell state.
    ///
    /// Returns: 1 = spell enabled, -1 = spell disabled, 0 = default
    #[must_use]
    pub fn effective_spell(&self) -> c_int {
        for i in (0..self.top).rev() {
            let state = &self.entries[i];
            if state.spell_enabled() {
                return 1;
            }
            if state.spell_disabled() {
                return -1;
            }
        }
        0
    }

    /// Compute effective conceal.
    ///
    /// Returns the conceal character if concealing is active, 0 otherwise.
    #[must_use]
    pub fn effective_conceal(&self) -> u32 {
        for i in (0..self.top).rev() {
            if self.entries[i].is_concealed() {
                return self.entries[i].conceal_char;
            }
        }
        0
    }
}

// =============================================================================
// Attribute Computation Helpers
// =============================================================================

/// Combine two highlight attributes.
///
/// If `base` is non-zero, it takes precedence. Otherwise use `override_attr`.
#[must_use]
pub const fn combine_attrs(base: c_int, override_attr: c_int) -> c_int {
    if base != 0 {
        base
    } else {
        override_attr
    }
}

/// Check if spell checking should be done based on syntax state.
///
/// `syn_spell` is the buffer's b_syn_spell setting (SYNSPL_*).
/// `state_spell` is from effective_spell() (1, -1, or 0).
#[must_use]
pub const fn should_spell_check(syn_spell: c_int, state_spell: c_int) -> bool {
    // SYNSPL_DEFAULT = 0, SYNSPL_TOP = 1, SYNSPL_NOTOP = 2
    match state_spell {
        1 => true,   // Explicit @Spell
        -1 => false, // Explicit @NoSpell
        _ => {
            // Default: depends on syn_spell setting
            // SYNSPL_TOP means spell check at top level
            syn_spell == 1 // SYNSPL_TOP
        }
    }
}

// =============================================================================
// Change Detection
// =============================================================================

/// Result of checking for syntax changes.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct SynChangeResult {
    /// Whether any change was detected.
    pub changed: bool,
    /// First line that changed (1-based).
    pub first_lnum: i32,
    /// Last line that changed (1-based, 0 if unknown).
    pub last_lnum: i32,
}

impl SynChangeResult {
    /// No change detected.
    #[must_use]
    pub const fn no_change() -> Self {
        Self {
            changed: false,
            first_lnum: 0,
            last_lnum: 0,
        }
    }

    /// Single line changed.
    #[must_use]
    pub const fn single_line(lnum: i32) -> Self {
        Self {
            changed: true,
            first_lnum: lnum,
            last_lnum: lnum,
        }
    }

    /// Range of lines changed.
    #[must_use]
    pub const fn range(first: i32, last: i32) -> Self {
        Self {
            changed: true,
            first_lnum: first,
            last_lnum: last,
        }
    }

    /// All lines from lnum to end of buffer changed.
    #[must_use]
    pub const fn to_end(lnum: i32) -> Self {
        Self {
            changed: true,
            first_lnum: lnum,
            last_lnum: 0,
        }
    }

    /// Merge with another change result.
    pub fn merge(&mut self, other: &Self) {
        if !other.changed {
            return;
        }
        if !self.changed {
            *self = *other;
            return;
        }
        // Both have changes - expand range
        if other.first_lnum < self.first_lnum {
            self.first_lnum = other.first_lnum;
        }
        if other.last_lnum == 0 || self.last_lnum == 0 {
            self.last_lnum = 0; // Unknown end
        } else if other.last_lnum > self.last_lnum {
            self.last_lnum = other.last_lnum;
        }
    }
}

// =============================================================================
// Invalidation Types
// =============================================================================

/// Reasons for syntax invalidation.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InvalidationReason {
    /// Buffer text was modified.
    TextChange = 0,
    /// Syntax rules changed (e.g., :syntax command).
    RulesChange = 1,
    /// Option changed (e.g., 'syntax', 'filetype').
    OptionChange = 2,
    /// Window changed (for window-local syntax).
    WindowChange = 3,
    /// Full invalidation (e.g., :syntax clear).
    FullClear = 4,
}

impl InvalidationReason {
    /// Check if this invalidation requires full re-sync.
    #[must_use]
    pub const fn needs_full_sync(&self) -> bool {
        matches!(self, Self::RulesChange | Self::FullClear)
    }
}

// =============================================================================
// FFI Exports
// =============================================================================

/// Create a new syntax attribute state.
#[no_mangle]
pub extern "C" fn rs_syn_attr_state_new() -> SynAttrState {
    SynAttrState::new()
}

/// Check if syntax attribute state has an attribute.
///
/// # Safety
/// `state` must be valid if not null.
#[no_mangle]
pub unsafe extern "C" fn rs_syn_attr_state_has_attr(state: *const SynAttrState) -> c_int {
    if state.is_null() {
        return 0;
    }
    c_int::from((*state).has_attr())
}

/// Check if syntax attribute state is concealed.
///
/// # Safety
/// `state` must be valid if not null.
#[no_mangle]
pub unsafe extern "C" fn rs_syn_attr_state_is_concealed(state: *const SynAttrState) -> c_int {
    if state.is_null() {
        return 0;
    }
    c_int::from((*state).is_concealed())
}

/// Get conceal character from syntax attribute state.
///
/// # Safety
/// `state` must be valid if not null.
#[no_mangle]
pub unsafe extern "C" fn rs_syn_attr_state_conceal_char(state: *const SynAttrState) -> u32 {
    if state.is_null() {
        return 0;
    }
    (*state).conceal_char
}

/// Create a new syntax attribute stack.
#[no_mangle]
pub extern "C" fn rs_syn_attr_stack_new() -> *mut SynAttrStack {
    Box::into_raw(Box::new(SynAttrStack::new()))
}

/// Free a syntax attribute stack.
///
/// # Safety
/// `stack` must be valid or null.
#[no_mangle]
pub unsafe extern "C" fn rs_syn_attr_stack_free(stack: *mut SynAttrStack) {
    if !stack.is_null() {
        drop(Box::from_raw(stack));
    }
}

/// Clear a syntax attribute stack.
///
/// # Safety
/// `stack` must be valid.
#[no_mangle]
pub unsafe extern "C" fn rs_syn_attr_stack_clear(stack: *mut SynAttrStack) {
    if !stack.is_null() {
        (*stack).clear();
    }
}

/// Push state onto syntax attribute stack.
///
/// # Safety
/// `stack` must be valid.
#[no_mangle]
pub unsafe extern "C" fn rs_syn_attr_stack_push(
    stack: *mut SynAttrStack,
    state: SynAttrState,
) -> c_int {
    if stack.is_null() {
        return 0;
    }
    c_int::from((*stack).push(state))
}

/// Pop state from syntax attribute stack.
///
/// # Safety
/// `stack` must be valid.
/// `out` must be valid if not null.
#[no_mangle]
pub unsafe extern "C" fn rs_syn_attr_stack_pop(
    stack: *mut SynAttrStack,
    out: *mut SynAttrState,
) -> c_int {
    if stack.is_null() {
        return 0;
    }
    match (*stack).pop() {
        Some(state) => {
            if !out.is_null() {
                *out = state;
            }
            1
        }
        None => 0,
    }
}

/// Get stack depth.
///
/// # Safety
/// `stack` must be valid.
#[no_mangle]
pub unsafe extern "C" fn rs_syn_attr_stack_depth(stack: *const SynAttrStack) -> c_int {
    if stack.is_null() {
        return 0;
    }
    (*stack).depth() as c_int
}

/// Get effective attribute from stack.
///
/// # Safety
/// `stack` must be valid.
#[no_mangle]
pub unsafe extern "C" fn rs_syn_attr_stack_effective_attr(stack: *const SynAttrStack) -> c_int {
    if stack.is_null() {
        return 0;
    }
    (*stack).effective_attr()
}

/// Get effective spell state from stack.
///
/// # Safety
/// `stack` must be valid.
#[no_mangle]
pub unsafe extern "C" fn rs_syn_attr_stack_effective_spell(stack: *const SynAttrStack) -> c_int {
    if stack.is_null() {
        return 0;
    }
    (*stack).effective_spell()
}

/// Combine two highlight attributes.
#[no_mangle]
pub extern "C" fn rs_combine_attrs(base: c_int, override_attr: c_int) -> c_int {
    combine_attrs(base, override_attr)
}

/// Check if spell checking should be done.
#[no_mangle]
pub extern "C" fn rs_should_spell_check(syn_spell: c_int, state_spell: c_int) -> c_int {
    c_int::from(should_spell_check(syn_spell, state_spell))
}

/// Create a no-change result.
#[no_mangle]
pub extern "C" fn rs_syn_change_no_change() -> SynChangeResult {
    SynChangeResult::no_change()
}

/// Create a single-line change result.
#[no_mangle]
pub extern "C" fn rs_syn_change_single_line(lnum: i32) -> SynChangeResult {
    SynChangeResult::single_line(lnum)
}

/// Create a range change result.
#[no_mangle]
pub extern "C" fn rs_syn_change_range(first: i32, last: i32) -> SynChangeResult {
    SynChangeResult::range(first, last)
}

/// Merge two change results.
///
/// # Safety
/// `result` must be valid.
#[no_mangle]
pub unsafe extern "C" fn rs_syn_change_merge(
    result: *mut SynChangeResult,
    other: *const SynChangeResult,
) {
    if result.is_null() || other.is_null() {
        return;
    }
    (*result).merge(&*other);
}

/// Check if invalidation reason needs full sync.
#[no_mangle]
pub extern "C" fn rs_invalidation_needs_full_sync(reason: c_int) -> c_int {
    let reason = match reason {
        0 => InvalidationReason::TextChange,
        1 => InvalidationReason::RulesChange,
        2 => InvalidationReason::OptionChange,
        3 => InvalidationReason::WindowChange,
        4 => InvalidationReason::FullClear,
        _ => return 0,
    };
    c_int::from(reason.needs_full_sync())
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_syn_attr_state() {
        let mut state = SynAttrState::new();
        assert!(!state.has_attr());
        assert!(!state.is_concealed());
        assert!(!state.spell_enabled());

        state.attr = 42;
        assert!(state.has_attr());

        state.set_conceal('x' as u32);
        assert!(state.is_concealed());
        assert_eq!(state.conceal_char, 'x' as u32);

        state.clear_conceal();
        assert!(!state.is_concealed());

        state.enable_spell();
        assert!(state.spell_enabled());
        assert!(!state.spell_disabled());

        state.disable_spell();
        assert!(!state.spell_enabled());
        assert!(state.spell_disabled());

        state.clear_spell();
        assert!(!state.spell_enabled());
        assert!(!state.spell_disabled());
    }

    #[test]
    fn test_syn_attr_stack() {
        let mut stack = SynAttrStack::new();
        assert!(stack.is_empty());
        assert_eq!(stack.depth(), 0);

        let mut state1 = SynAttrState::new();
        state1.attr = 1;
        stack.push(state1);
        assert_eq!(stack.depth(), 1);
        assert!(!stack.is_empty());

        let mut state2 = SynAttrState::new();
        state2.attr = 2;
        state2.flags |= SYN_ATTR_TRANSPARENT;
        stack.push(state2);
        assert_eq!(stack.depth(), 2);

        // Effective attr should skip transparent
        assert_eq!(stack.effective_attr(), 1);

        let popped = stack.pop().unwrap();
        assert_eq!(popped.attr, 2);
        assert_eq!(stack.depth(), 1);

        stack.clear();
        assert!(stack.is_empty());
    }

    #[test]
    fn test_effective_spell() {
        let mut stack = SynAttrStack::new();

        // Default is 0
        assert_eq!(stack.effective_spell(), 0);

        let mut state1 = SynAttrState::new();
        state1.enable_spell();
        stack.push(state1);
        assert_eq!(stack.effective_spell(), 1);

        let mut state2 = SynAttrState::new();
        state2.disable_spell();
        stack.push(state2);
        // Top wins
        assert_eq!(stack.effective_spell(), -1);
    }

    #[test]
    fn test_combine_attrs() {
        assert_eq!(combine_attrs(5, 10), 5);
        assert_eq!(combine_attrs(0, 10), 10);
        assert_eq!(combine_attrs(0, 0), 0);
    }

    #[test]
    fn test_should_spell_check() {
        // Explicit @Spell
        assert!(should_spell_check(0, 1));
        assert!(should_spell_check(1, 1));
        assert!(should_spell_check(2, 1));

        // Explicit @NoSpell
        assert!(!should_spell_check(0, -1));
        assert!(!should_spell_check(1, -1));

        // Default depends on syn_spell
        assert!(!should_spell_check(0, 0)); // SYNSPL_DEFAULT
        assert!(should_spell_check(1, 0)); // SYNSPL_TOP
        assert!(!should_spell_check(2, 0)); // SYNSPL_NOTOP
    }

    #[test]
    fn test_syn_change_result() {
        let no_change = SynChangeResult::no_change();
        assert!(!no_change.changed);

        let single = SynChangeResult::single_line(5);
        assert!(single.changed);
        assert_eq!(single.first_lnum, 5);
        assert_eq!(single.last_lnum, 5);

        let range = SynChangeResult::range(10, 20);
        assert!(range.changed);
        assert_eq!(range.first_lnum, 10);
        assert_eq!(range.last_lnum, 20);

        let mut result = SynChangeResult::single_line(5);
        let other = SynChangeResult::range(3, 15);
        result.merge(&other);
        assert_eq!(result.first_lnum, 3);
        assert_eq!(result.last_lnum, 15);
    }

    #[test]
    fn test_invalidation_reason() {
        assert!(!InvalidationReason::TextChange.needs_full_sync());
        assert!(InvalidationReason::RulesChange.needs_full_sync());
        assert!(!InvalidationReason::OptionChange.needs_full_sync());
        assert!(InvalidationReason::FullClear.needs_full_sync());
    }
}
