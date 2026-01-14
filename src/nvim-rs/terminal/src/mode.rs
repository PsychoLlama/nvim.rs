//! Terminal mode handling
//!
//! This module provides Rust implementations for terminal mode state machine,
//! including terminal-normal mode, terminal-insert mode, and mode transitions.

#![allow(clippy::must_use_candidate)]
#![allow(clippy::doc_markdown)]
#![allow(clippy::use_self)]
#![allow(clippy::derivable_impls)]
#![allow(clippy::unnested_or_patterns)]
#![allow(clippy::match_same_arms)]

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

/// Opaque handle to a window (win_T).
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct WinHandle(*mut c_void);

impl WinHandle {
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

#[allow(dead_code)]
extern "C" {
    fn nvim_terminal_get_closed(term: TerminalHandle) -> c_int;
    fn nvim_terminal_get_cursor_visible(term: TerminalHandle) -> c_int;
    fn nvim_terminal_get_cursor_shape(term: TerminalHandle) -> c_int;
    fn nvim_terminal_get_cursor_blink(term: TerminalHandle) -> c_int;
    fn nvim_terminal_get_forward_mouse(term: TerminalHandle) -> c_int;
}

// =============================================================================
// Terminal Mode Types
// =============================================================================

/// Terminal mode enumeration.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TerminalMode {
    /// Not in terminal mode.
    None = 0,
    /// Terminal-Normal mode (can navigate, select, etc.).
    Normal = 1,
    /// Terminal-Insert mode (input goes to terminal).
    Insert = 2,
}

impl TerminalMode {
    /// Check if in any terminal mode.
    pub const fn is_terminal(self) -> bool {
        !matches!(self, TerminalMode::None)
    }

    /// Check if in terminal insert mode.
    pub const fn is_insert(self) -> bool {
        matches!(self, TerminalMode::Insert)
    }

    /// Check if in terminal normal mode.
    pub const fn is_normal(self) -> bool {
        matches!(self, TerminalMode::Normal)
    }
}

// =============================================================================
// Cursor State
// =============================================================================

/// Cursor shape types (matching VTermProp cursor shape values).
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CursorShape {
    /// Block cursor.
    Block = 1,
    /// Underline cursor.
    Underline = 2,
    /// Bar (vertical line) cursor.
    Bar = 3,
}

impl CursorShape {
    /// Create from raw value.
    pub const fn from_raw(val: c_int) -> Option<Self> {
        match val {
            1 => Some(CursorShape::Block),
            2 => Some(CursorShape::Underline),
            3 => Some(CursorShape::Bar),
            _ => None,
        }
    }

    /// Get the raw value.
    pub const fn as_raw(self) -> c_int {
        self as c_int
    }
}

impl Default for CursorShape {
    fn default() -> Self {
        CursorShape::Block
    }
}

/// Terminal cursor state.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct TerminalCursor {
    /// Whether the cursor is visible.
    pub visible: bool,
    /// The cursor shape.
    pub shape: CursorShape,
    /// Whether the cursor is blinking.
    pub blink: bool,
}

impl TerminalCursor {
    /// Create a default cursor state.
    pub const fn default_state() -> Self {
        Self {
            visible: true,
            shape: CursorShape::Block,
            blink: false,
        }
    }
}

impl Default for TerminalCursor {
    fn default() -> Self {
        Self::default_state()
    }
}

/// Get the cursor state from a terminal.
pub fn get_terminal_cursor(term: TerminalHandle) -> TerminalCursor {
    if term.is_null() {
        return TerminalCursor::default_state();
    }

    unsafe {
        let shape_raw = nvim_terminal_get_cursor_shape(term);
        TerminalCursor {
            visible: nvim_terminal_get_cursor_visible(term) != 0,
            shape: CursorShape::from_raw(shape_raw).unwrap_or(CursorShape::Block),
            blink: nvim_terminal_get_cursor_blink(term) != 0,
        }
    }
}

// =============================================================================
// Terminal State
// =============================================================================

/// Full terminal mode state.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct TerminalState {
    /// Current terminal mode.
    pub mode: TerminalMode,
    /// Whether the terminal is closed.
    pub closed: bool,
    /// Cursor state.
    pub cursor: TerminalCursor,
    /// Whether mouse events should be forwarded to the terminal.
    pub forward_mouse: bool,
}

impl TerminalState {
    /// Create an inactive state.
    pub const fn inactive() -> Self {
        Self {
            mode: TerminalMode::None,
            closed: true,
            cursor: TerminalCursor::default_state(),
            forward_mouse: false,
        }
    }
}

impl Default for TerminalState {
    fn default() -> Self {
        Self::inactive()
    }
}

/// Get the full terminal state.
pub fn get_terminal_state(term: TerminalHandle, mode: TerminalMode) -> TerminalState {
    if term.is_null() {
        return TerminalState::inactive();
    }

    unsafe {
        TerminalState {
            mode,
            closed: nvim_terminal_get_closed(term) != 0,
            cursor: get_terminal_cursor(term),
            forward_mouse: nvim_terminal_get_forward_mouse(term) != 0,
        }
    }
}

// =============================================================================
// Mode Transitions
// =============================================================================

/// Result of a mode transition.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModeTransitionResult {
    /// Transition successful.
    Success = 0,
    /// Terminal is null.
    NullTerminal = 1,
    /// Terminal is closed.
    Closed = 2,
    /// Already in the target mode.
    AlreadyInMode = 3,
    /// Invalid transition.
    InvalidTransition = 4,
}

/// Check if a mode transition is valid.
pub const fn is_valid_transition(from: TerminalMode, to: TerminalMode) -> bool {
    match (from, to) {
        // Same mode is not a valid transition
        (TerminalMode::None, TerminalMode::None)
        | (TerminalMode::Normal, TerminalMode::Normal)
        | (TerminalMode::Insert, TerminalMode::Insert) => false,
        // Can transition to None (exit terminal) from non-None
        (TerminalMode::Normal, TerminalMode::None) | (TerminalMode::Insert, TerminalMode::None) => {
            true
        }
        // Can enter terminal from None
        (TerminalMode::None, TerminalMode::Normal) | (TerminalMode::None, TerminalMode::Insert) => {
            true
        }
        // Can switch between normal and insert
        (TerminalMode::Normal, TerminalMode::Insert)
        | (TerminalMode::Insert, TerminalMode::Normal) => true,
    }
}

/// Validate a mode transition for a terminal.
pub fn validate_mode_transition(
    term: TerminalHandle,
    from: TerminalMode,
    to: TerminalMode,
) -> ModeTransitionResult {
    if term.is_null() {
        return ModeTransitionResult::NullTerminal;
    }

    unsafe {
        if nvim_terminal_get_closed(term) != 0 {
            return ModeTransitionResult::Closed;
        }
    }

    if from == to {
        return ModeTransitionResult::AlreadyInMode;
    }

    if !is_valid_transition(from, to) {
        return ModeTransitionResult::InvalidTransition;
    }

    ModeTransitionResult::Success
}

// =============================================================================
// Focus State
// =============================================================================

/// Focus state for terminal windows.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FocusState {
    /// Terminal does not have focus.
    Unfocused = 0,
    /// Terminal has focus.
    Focused = 1,
}

/// Terminal focus change event.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct FocusChange {
    /// Previous focus state.
    pub previous: FocusState,
    /// New focus state.
    pub current: FocusState,
}

impl FocusChange {
    /// Check if focus was gained.
    pub const fn gained_focus(&self) -> bool {
        matches!(
            (self.previous, self.current),
            (FocusState::Unfocused, FocusState::Focused)
        )
    }

    /// Check if focus was lost.
    pub const fn lost_focus(&self) -> bool {
        matches!(
            (self.previous, self.current),
            (FocusState::Focused, FocusState::Unfocused)
        )
    }

    /// Check if focus state changed.
    pub const fn changed(&self) -> bool {
        !matches!(
            (self.previous, self.current),
            (FocusState::Focused, FocusState::Focused)
                | (FocusState::Unfocused, FocusState::Unfocused)
        )
    }
}

// =============================================================================
// FFI Exports
// =============================================================================

/// FFI export: Get terminal cursor state.
#[no_mangle]
pub extern "C" fn rs_get_terminal_cursor(term: TerminalHandle) -> TerminalCursor {
    get_terminal_cursor(term)
}

/// FFI export: Get terminal state.
#[no_mangle]
pub extern "C" fn rs_get_terminal_state(term: TerminalHandle, mode: c_int) -> TerminalState {
    let mode = match mode {
        0 => TerminalMode::None,
        1 => TerminalMode::Normal,
        2 => TerminalMode::Insert,
        _ => TerminalMode::None,
    };
    get_terminal_state(term, mode)
}

/// FFI export: Check if mode transition is valid.
#[no_mangle]
pub extern "C" fn rs_is_valid_mode_transition(from: c_int, to: c_int) -> c_int {
    let from_mode = match from {
        0 => TerminalMode::None,
        1 => TerminalMode::Normal,
        2 => TerminalMode::Insert,
        _ => return 0,
    };
    let to_mode = match to {
        0 => TerminalMode::None,
        1 => TerminalMode::Normal,
        2 => TerminalMode::Insert,
        _ => return 0,
    };
    c_int::from(is_valid_transition(from_mode, to_mode))
}

/// FFI export: Validate mode transition.
#[no_mangle]
pub extern "C" fn rs_validate_mode_transition(
    term: TerminalHandle,
    from: c_int,
    to: c_int,
) -> ModeTransitionResult {
    let from_mode = match from {
        0 => TerminalMode::None,
        1 => TerminalMode::Normal,
        2 => TerminalMode::Insert,
        _ => return ModeTransitionResult::InvalidTransition,
    };
    let to_mode = match to {
        0 => TerminalMode::None,
        1 => TerminalMode::Normal,
        2 => TerminalMode::Insert,
        _ => return ModeTransitionResult::InvalidTransition,
    };
    validate_mode_transition(term, from_mode, to_mode)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_terminal_mode_values() {
        assert_eq!(TerminalMode::None as c_int, 0);
        assert_eq!(TerminalMode::Normal as c_int, 1);
        assert_eq!(TerminalMode::Insert as c_int, 2);
    }

    #[test]
    fn test_terminal_mode_checks() {
        assert!(!TerminalMode::None.is_terminal());
        assert!(TerminalMode::Normal.is_terminal());
        assert!(TerminalMode::Insert.is_terminal());

        assert!(TerminalMode::Insert.is_insert());
        assert!(!TerminalMode::Normal.is_insert());

        assert!(TerminalMode::Normal.is_normal());
        assert!(!TerminalMode::Insert.is_normal());
    }

    #[test]
    fn test_cursor_shape() {
        assert_eq!(CursorShape::from_raw(1), Some(CursorShape::Block));
        assert_eq!(CursorShape::from_raw(2), Some(CursorShape::Underline));
        assert_eq!(CursorShape::from_raw(3), Some(CursorShape::Bar));
        assert_eq!(CursorShape::from_raw(0), None);
        assert_eq!(CursorShape::from_raw(4), None);
    }

    #[test]
    fn test_valid_transitions() {
        // Can exit to None from anywhere
        assert!(is_valid_transition(
            TerminalMode::Normal,
            TerminalMode::None
        ));
        assert!(is_valid_transition(
            TerminalMode::Insert,
            TerminalMode::None
        ));

        // Can enter terminal from None
        assert!(is_valid_transition(
            TerminalMode::None,
            TerminalMode::Normal
        ));
        assert!(is_valid_transition(
            TerminalMode::None,
            TerminalMode::Insert
        ));

        // Can switch between normal and insert
        assert!(is_valid_transition(
            TerminalMode::Normal,
            TerminalMode::Insert
        ));
        assert!(is_valid_transition(
            TerminalMode::Insert,
            TerminalMode::Normal
        ));

        // Same mode is not a valid transition
        assert!(!is_valid_transition(TerminalMode::None, TerminalMode::None));
        assert!(!is_valid_transition(
            TerminalMode::Normal,
            TerminalMode::Normal
        ));
        assert!(!is_valid_transition(
            TerminalMode::Insert,
            TerminalMode::Insert
        ));
    }

    #[test]
    fn test_mode_transition_result_values() {
        assert_eq!(ModeTransitionResult::Success as c_int, 0);
        assert_eq!(ModeTransitionResult::NullTerminal as c_int, 1);
        assert_eq!(ModeTransitionResult::Closed as c_int, 2);
        assert_eq!(ModeTransitionResult::AlreadyInMode as c_int, 3);
        assert_eq!(ModeTransitionResult::InvalidTransition as c_int, 4);
    }

    #[test]
    fn test_focus_change() {
        let gain = FocusChange {
            previous: FocusState::Unfocused,
            current: FocusState::Focused,
        };
        assert!(gain.gained_focus());
        assert!(!gain.lost_focus());
        assert!(gain.changed());

        let lose = FocusChange {
            previous: FocusState::Focused,
            current: FocusState::Unfocused,
        };
        assert!(!lose.gained_focus());
        assert!(lose.lost_focus());
        assert!(lose.changed());

        let same = FocusChange {
            previous: FocusState::Focused,
            current: FocusState::Focused,
        };
        assert!(!same.gained_focus());
        assert!(!same.lost_focus());
        assert!(!same.changed());
    }

    #[test]
    fn test_terminal_state_inactive() {
        let state = TerminalState::inactive();
        assert_eq!(state.mode, TerminalMode::None);
        assert!(state.closed);
        assert!(!state.forward_mouse);
    }

    #[test]
    fn test_terminal_cursor_default() {
        let cursor = TerminalCursor::default_state();
        assert!(cursor.visible);
        assert_eq!(cursor.shape, CursorShape::Block);
        assert!(!cursor.blink);
    }

    #[test]
    fn test_terminal_mode_methods() {
        assert!(!TerminalMode::None.is_terminal());
        assert!(TerminalMode::Normal.is_terminal());
        assert!(TerminalMode::Insert.is_terminal());

        assert!(!TerminalMode::None.is_insert());
        assert!(!TerminalMode::Normal.is_insert());
        assert!(TerminalMode::Insert.is_insert());

        assert!(!TerminalMode::None.is_normal());
        assert!(TerminalMode::Normal.is_normal());
        assert!(!TerminalMode::Insert.is_normal());
    }

    #[test]
    fn test_cursor_shape_roundtrip() {
        // All cursor shapes should roundtrip through raw values
        assert_eq!(
            CursorShape::from_raw(CursorShape::Block.as_raw()),
            Some(CursorShape::Block)
        );
        assert_eq!(
            CursorShape::from_raw(CursorShape::Underline.as_raw()),
            Some(CursorShape::Underline)
        );
        assert_eq!(
            CursorShape::from_raw(CursorShape::Bar.as_raw()),
            Some(CursorShape::Bar)
        );
    }

    #[test]
    fn test_opaque_handle_null_checks() {
        let null_term = TerminalHandle::null();
        let null_win = WinHandle::null();

        assert!(null_term.is_null());
        assert!(null_win.is_null());

        // Non-null handles
        let fake_term = TerminalHandle(std::ptr::dangling_mut::<std::ffi::c_void>());
        let fake_win = WinHandle(std::ptr::dangling_mut::<std::ffi::c_void>());

        assert!(!fake_term.is_null());
        assert!(!fake_win.is_null());
    }

    #[test]
    fn test_validate_mode_transition_null() {
        // Validate with null terminal should return NullTerminal error
        let result = validate_mode_transition(
            TerminalHandle::null(),
            TerminalMode::None,
            TerminalMode::Insert,
        );
        assert_eq!(result, ModeTransitionResult::NullTerminal);
    }
}
