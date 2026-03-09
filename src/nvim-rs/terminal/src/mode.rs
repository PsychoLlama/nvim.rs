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
// Internal Helpers
// =============================================================================

/// Helper: get shared reference to CTerminal from a handle.
/// # Safety: handle must be non-null and valid.
#[inline]
const unsafe fn term_ref(term: TerminalHandle) -> &'static crate::CTerminal {
    unsafe { &*(term.0 as *const crate::CTerminal) }
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

    let t = unsafe { term_ref(term) };
    TerminalCursor {
        visible: t.cursor.visible,
        shape: CursorShape::from_raw(t.cursor.shape).unwrap_or(CursorShape::Block),
        blink: t.cursor.blink,
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

    let t = unsafe { term_ref(term) };
    TerminalState {
        mode,
        closed: t.closed,
        cursor: get_terminal_cursor(term),
        forward_mouse: t.forward_mouse,
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

    if unsafe { term_ref(term).closed } {
        return ModeTransitionResult::Closed;
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
// Job State Integration
// =============================================================================

/// Job state for terminal processes.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JobState {
    /// Job has not started.
    Pending = 0,
    /// Job is running.
    Running = 1,
    /// Job is paused/stopped.
    Stopped = 2,
    /// Job has exited normally.
    Exited = 3,
    /// Job was killed by signal.
    Killed = 4,
    /// Job failed to start.
    Failed = 5,
}

impl JobState {
    /// Check if the job is active (running or stopped).
    pub const fn is_active(self) -> bool {
        matches!(self, JobState::Running | JobState::Stopped)
    }

    /// Check if the job has terminated.
    pub const fn is_terminated(self) -> bool {
        matches!(self, JobState::Exited | JobState::Killed | JobState::Failed)
    }

    /// Check if the job is running.
    pub const fn is_running(self) -> bool {
        matches!(self, JobState::Running)
    }
}

/// Terminal job info.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct JobInfo {
    /// Current job state.
    pub state: JobState,
    /// Exit code (valid only when state is Exited).
    pub exit_code: c_int,
    /// Signal number (valid only when state is Killed).
    pub signal: c_int,
}

impl JobInfo {
    /// Create a pending job info.
    pub const fn pending() -> Self {
        Self {
            state: JobState::Pending,
            exit_code: -1,
            signal: 0,
        }
    }

    /// Create a running job info.
    pub const fn running() -> Self {
        Self {
            state: JobState::Running,
            exit_code: -1,
            signal: 0,
        }
    }

    /// Create an exited job info.
    pub const fn exited(code: c_int) -> Self {
        Self {
            state: JobState::Exited,
            exit_code: code,
            signal: 0,
        }
    }

    /// Create a killed job info.
    pub const fn killed(signal: c_int) -> Self {
        Self {
            state: JobState::Killed,
            exit_code: -1,
            signal,
        }
    }

    /// Check if job exited successfully (exit code 0).
    pub const fn exited_success(&self) -> bool {
        matches!(self.state, JobState::Exited) && self.exit_code == 0
    }
}

impl Default for JobInfo {
    fn default() -> Self {
        Self::pending()
    }
}

// =============================================================================
// Mode Entry/Exit Helpers
// =============================================================================

/// Reason for entering terminal mode.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModeEntryReason {
    /// Explicit user action (e.g., i in terminal window).
    UserAction = 0,
    /// Terminal buffer opened.
    BufferOpen = 1,
    /// Window focus change.
    WindowFocus = 2,
    /// API call.
    Api = 3,
}

/// Reason for exiting terminal mode.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModeExitReason {
    /// User pressed escape or similar.
    UserAction = 0,
    /// Job terminated.
    JobTerminated = 1,
    /// Window focus lost.
    WindowBlur = 2,
    /// API call.
    Api = 3,
    /// Buffer closed.
    BufferClosed = 4,
}

/// Mode entry request.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct ModeEntry {
    /// Target mode.
    pub target: TerminalMode,
    /// Reason for entry.
    pub reason: ModeEntryReason,
}

impl ModeEntry {
    /// Create an insert mode entry.
    pub const fn insert(reason: ModeEntryReason) -> Self {
        Self {
            target: TerminalMode::Insert,
            reason,
        }
    }

    /// Create a normal mode entry.
    pub const fn normal(reason: ModeEntryReason) -> Self {
        Self {
            target: TerminalMode::Normal,
            reason,
        }
    }
}

/// Mode exit request.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct ModeExit {
    /// Current mode being exited.
    pub from: TerminalMode,
    /// Reason for exit.
    pub reason: ModeExitReason,
}

impl ModeExit {
    /// Create an exit request.
    pub const fn new(from: TerminalMode, reason: ModeExitReason) -> Self {
        Self { from, reason }
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

/// FFI export: Check if job is active.
#[no_mangle]
pub extern "C" fn rs_terminal_job_is_active(state: c_int) -> c_int {
    let job_state = match state {
        0 => JobState::Pending,
        1 => JobState::Running,
        2 => JobState::Stopped,
        3 => JobState::Exited,
        4 => JobState::Killed,
        5 => JobState::Failed,
        _ => return 0,
    };
    c_int::from(job_state.is_active())
}

/// FFI export: Check if job is terminated.
#[no_mangle]
pub extern "C" fn rs_terminal_job_is_terminated(state: c_int) -> c_int {
    let job_state = match state {
        0 => JobState::Pending,
        1 => JobState::Running,
        2 => JobState::Stopped,
        3 => JobState::Exited,
        4 => JobState::Killed,
        5 => JobState::Failed,
        _ => return 0,
    };
    c_int::from(job_state.is_terminated())
}

/// FFI export: Create exited job info.
#[no_mangle]
#[allow(clippy::missing_const_for_fn)]
pub extern "C" fn rs_terminal_job_exited(exit_code: c_int) -> JobInfo {
    JobInfo::exited(exit_code)
}

/// FFI export: Create killed job info.
#[no_mangle]
#[allow(clippy::missing_const_for_fn)]
pub extern "C" fn rs_terminal_job_killed(signal: c_int) -> JobInfo {
    JobInfo::killed(signal)
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

    #[test]
    fn test_job_state_values() {
        assert_eq!(JobState::Pending as c_int, 0);
        assert_eq!(JobState::Running as c_int, 1);
        assert_eq!(JobState::Stopped as c_int, 2);
        assert_eq!(JobState::Exited as c_int, 3);
        assert_eq!(JobState::Killed as c_int, 4);
        assert_eq!(JobState::Failed as c_int, 5);
    }

    #[test]
    fn test_job_state_checks() {
        assert!(!JobState::Pending.is_active());
        assert!(JobState::Running.is_active());
        assert!(JobState::Stopped.is_active());
        assert!(!JobState::Exited.is_active());
        assert!(!JobState::Killed.is_active());
        assert!(!JobState::Failed.is_active());

        assert!(!JobState::Pending.is_terminated());
        assert!(!JobState::Running.is_terminated());
        assert!(!JobState::Stopped.is_terminated());
        assert!(JobState::Exited.is_terminated());
        assert!(JobState::Killed.is_terminated());
        assert!(JobState::Failed.is_terminated());

        assert!(!JobState::Pending.is_running());
        assert!(JobState::Running.is_running());
        assert!(!JobState::Stopped.is_running());
    }

    #[test]
    fn test_job_info_creation() {
        let pending = JobInfo::pending();
        assert_eq!(pending.state, JobState::Pending);
        assert_eq!(pending.exit_code, -1);

        let running = JobInfo::running();
        assert_eq!(running.state, JobState::Running);

        let exited = JobInfo::exited(42);
        assert_eq!(exited.state, JobState::Exited);
        assert_eq!(exited.exit_code, 42);
        assert!(!exited.exited_success());

        let success = JobInfo::exited(0);
        assert!(success.exited_success());

        let killed = JobInfo::killed(9);
        assert_eq!(killed.state, JobState::Killed);
        assert_eq!(killed.signal, 9);
    }

    #[test]
    fn test_mode_entry_reason_values() {
        assert_eq!(ModeEntryReason::UserAction as c_int, 0);
        assert_eq!(ModeEntryReason::BufferOpen as c_int, 1);
        assert_eq!(ModeEntryReason::WindowFocus as c_int, 2);
        assert_eq!(ModeEntryReason::Api as c_int, 3);
    }

    #[test]
    fn test_mode_exit_reason_values() {
        assert_eq!(ModeExitReason::UserAction as c_int, 0);
        assert_eq!(ModeExitReason::JobTerminated as c_int, 1);
        assert_eq!(ModeExitReason::WindowBlur as c_int, 2);
        assert_eq!(ModeExitReason::Api as c_int, 3);
        assert_eq!(ModeExitReason::BufferClosed as c_int, 4);
    }

    #[test]
    fn test_mode_entry_creation() {
        let insert = ModeEntry::insert(ModeEntryReason::UserAction);
        assert_eq!(insert.target, TerminalMode::Insert);
        assert_eq!(insert.reason, ModeEntryReason::UserAction);

        let normal = ModeEntry::normal(ModeEntryReason::WindowFocus);
        assert_eq!(normal.target, TerminalMode::Normal);
        assert_eq!(normal.reason, ModeEntryReason::WindowFocus);
    }

    #[test]
    fn test_mode_exit_creation() {
        let exit = ModeExit::new(TerminalMode::Insert, ModeExitReason::UserAction);
        assert_eq!(exit.from, TerminalMode::Insert);
        assert_eq!(exit.reason, ModeExitReason::UserAction);
    }

    #[test]
    fn test_struct_sizes() {
        use std::mem::size_of;
        // JobInfo should be reasonable size
        assert!(size_of::<JobInfo>() <= 16);
        // ModeEntry should be small
        assert!(size_of::<ModeEntry>() <= 8);
        // ModeExit should be small
        assert!(size_of::<ModeExit>() <= 8);
    }
}
