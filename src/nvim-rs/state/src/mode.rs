//! Mode tracking
//!
//! This module provides mode tracking infrastructure for Neovim,
//! including mode identification, transitions, and state.

use std::ffi::c_int;

// =============================================================================
// Mode Types
// =============================================================================

/// Neovim editing modes.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Mode {
    /// Normal mode
    #[default]
    Normal = 0,
    /// Visual mode (characterwise)
    Visual = 1,
    /// Visual line mode
    VisualLine = 2,
    /// Visual block mode
    VisualBlock = 3,
    /// Select mode (characterwise)
    Select = 4,
    /// Select line mode
    SelectLine = 5,
    /// Select block mode
    SelectBlock = 6,
    /// Insert mode
    Insert = 7,
    /// Replace mode
    Replace = 8,
    /// Virtual replace mode
    VReplace = 9,
    /// Command-line mode
    CmdLine = 10,
    /// Ex mode
    Ex = 11,
    /// Operator-pending mode
    OpPending = 12,
    /// Terminal mode
    Terminal = 13,
    /// Lang-arg mode
    LangArg = 14,
}

impl Mode {
    /// Create from C int.
    #[must_use]
    pub const fn from_c_int(val: c_int) -> Self {
        match val {
            1 => Self::Visual,
            2 => Self::VisualLine,
            3 => Self::VisualBlock,
            4 => Self::Select,
            5 => Self::SelectLine,
            6 => Self::SelectBlock,
            7 => Self::Insert,
            8 => Self::Replace,
            9 => Self::VReplace,
            10 => Self::CmdLine,
            11 => Self::Ex,
            12 => Self::OpPending,
            13 => Self::Terminal,
            14 => Self::LangArg,
            _ => Self::Normal,
        }
    }

    /// Convert to C int.
    #[must_use]
    pub const fn to_c_int(self) -> c_int {
        self as c_int
    }

    /// Check if mode is a visual mode.
    #[must_use]
    pub const fn is_visual(self) -> bool {
        matches!(self, Self::Visual | Self::VisualLine | Self::VisualBlock)
    }

    /// Check if mode is a select mode.
    #[must_use]
    pub const fn is_select(self) -> bool {
        matches!(self, Self::Select | Self::SelectLine | Self::SelectBlock)
    }

    /// Check if mode allows editing.
    #[must_use]
    pub const fn allows_editing(self) -> bool {
        matches!(self, Self::Insert | Self::Replace | Self::VReplace)
    }

    /// Check if mode is command-line related.
    #[must_use]
    pub const fn is_cmdline(self) -> bool {
        matches!(self, Self::CmdLine | Self::Ex)
    }

    /// Get the mode character for statusline.
    #[must_use]
    pub const fn char(self) -> u8 {
        match self {
            Self::Normal => b'n',
            Self::Visual => b'v',
            Self::VisualLine => b'V',
            Self::VisualBlock => b'\x16', // Ctrl-V
            Self::Select => b's',
            Self::SelectLine => b'S',
            Self::SelectBlock => b'\x13', // Ctrl-S
            Self::Insert => b'i',
            Self::Replace => b'R',
            Self::VReplace => b'r',
            Self::CmdLine | Self::Ex => b'c',
            Self::OpPending => b'o',
            Self::Terminal => b't',
            Self::LangArg => b'l',
        }
    }
}

/// FFI: Check if mode is visual.
#[no_mangle]
pub extern "C" fn rs_mode_is_visual(mode: c_int) -> c_int {
    c_int::from(Mode::from_c_int(mode).is_visual())
}

/// FFI: Check if mode is select.
#[no_mangle]
pub extern "C" fn rs_mode_is_select(mode: c_int) -> c_int {
    c_int::from(Mode::from_c_int(mode).is_select())
}

/// FFI: Check if mode allows editing.
#[no_mangle]
pub extern "C" fn rs_mode_allows_editing(mode: c_int) -> c_int {
    c_int::from(Mode::from_c_int(mode).allows_editing())
}

/// FFI: Check if mode is cmdline.
#[no_mangle]
pub extern "C" fn rs_mode_is_cmdline(mode: c_int) -> c_int {
    c_int::from(Mode::from_c_int(mode).is_cmdline())
}

/// FFI: Get mode character.
#[no_mangle]
pub extern "C" fn rs_mode_char(mode: c_int) -> c_int {
    c_int::from(Mode::from_c_int(mode).char())
}

// =============================================================================
// Mode Modifiers
// =============================================================================

/// Modifiers that affect mode behavior.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct ModeModifiers {
    /// Langmap is active
    pub langmap: bool,
    /// Insert mode is from "i" (not "a", "o", etc.)
    pub insert_i: bool,
    /// Replace mode is from "R" (not "gR")
    pub replace_r: bool,
    /// Completion is active
    pub completion: bool,
    /// Recording a macro
    pub recording: bool,
    /// Executing a macro
    pub executing: bool,
}

impl ModeModifiers {
    /// Create new modifiers.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            langmap: false,
            insert_i: false,
            replace_r: false,
            completion: false,
            recording: false,
            executing: false,
        }
    }

    /// Check if any modifier is active.
    #[must_use]
    pub const fn any_active(&self) -> bool {
        self.langmap
            || self.insert_i
            || self.replace_r
            || self.completion
            || self.recording
            || self.executing
    }
}

/// FFI: Create mode modifiers.
#[no_mangle]
pub extern "C" fn rs_mode_modifiers_new() -> ModeModifiers {
    ModeModifiers::new()
}

/// FFI: Check if any modifier active.
///
/// # Safety
/// `mods` must be valid or null.
#[no_mangle]
pub unsafe extern "C" fn rs_mode_modifiers_any_active(mods: *const ModeModifiers) -> c_int {
    if mods.is_null() {
        return 0;
    }
    c_int::from((*mods).any_active())
}

// =============================================================================
// Mode State
// =============================================================================

/// Current mode state.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct ModeState {
    /// Current mode
    pub mode: c_int,
    /// Previous mode
    pub prev_mode: c_int,
    /// Mode modifiers
    pub modifiers: ModeModifiers,
    /// Mode entry count (for nesting)
    pub entry_count: c_int,
    /// Time mode was entered (ms)
    pub entry_time_ms: i64,
}

impl ModeState {
    /// Create new mode state.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            mode: Mode::Normal as c_int,
            prev_mode: Mode::Normal as c_int,
            modifiers: ModeModifiers::new(),
            entry_count: 0,
            entry_time_ms: 0,
        }
    }

    /// Get current mode.
    #[must_use]
    pub const fn get_mode(&self) -> Mode {
        Mode::from_c_int(self.mode)
    }

    /// Get previous mode.
    #[must_use]
    pub const fn get_prev_mode(&self) -> Mode {
        Mode::from_c_int(self.prev_mode)
    }

    /// Change to a new mode.
    pub fn change_to(&mut self, new_mode: Mode, time_ms: i64) {
        self.prev_mode = self.mode;
        self.mode = new_mode as c_int;
        self.entry_time_ms = time_ms;
        self.entry_count += 1;
    }

    /// Check if mode changed recently.
    #[must_use]
    pub const fn just_changed(&self) -> bool {
        self.mode != self.prev_mode
    }
}

/// FFI: Create mode state.
#[no_mangle]
pub extern "C" fn rs_mode_state_new() -> ModeState {
    ModeState::new()
}

/// FFI: Get current mode.
///
/// # Safety
/// `state` must be valid or null.
#[no_mangle]
pub unsafe extern "C" fn rs_mode_state_get_mode(state: *const ModeState) -> c_int {
    if state.is_null() {
        return 0;
    }
    (*state).mode
}

/// FFI: Change mode.
///
/// # Safety
/// `state` must be valid or null.
#[no_mangle]
pub unsafe extern "C" fn rs_mode_state_change_to(
    state: *mut ModeState,
    new_mode: c_int,
    time_ms: i64,
) {
    if !state.is_null() {
        (*state).change_to(Mode::from_c_int(new_mode), time_ms);
    }
}

// =============================================================================
// Mode Transition
// =============================================================================

/// Describes a mode transition.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct ModeTransition {
    /// Mode transitioning from
    pub from: c_int,
    /// Mode transitioning to
    pub to: c_int,
    /// Transition is allowed
    pub allowed: bool,
    /// Transition requires cleanup
    pub needs_cleanup: bool,
}

impl ModeTransition {
    /// Create a transition.
    #[must_use]
    pub const fn new(from: Mode, to: Mode) -> Self {
        Self {
            from: from as c_int,
            to: to as c_int,
            allowed: true,
            needs_cleanup: false,
        }
    }

    /// Check if transitioning to insert mode.
    #[must_use]
    pub const fn entering_insert(&self) -> bool {
        let to = Mode::from_c_int(self.to);
        to.allows_editing()
    }

    /// Check if leaving insert mode.
    #[must_use]
    pub const fn leaving_insert(&self) -> bool {
        let from = Mode::from_c_int(self.from);
        from.allows_editing()
    }
}

/// FFI: Create mode transition.
#[no_mangle]
pub extern "C" fn rs_mode_transition_new(from: c_int, to: c_int) -> ModeTransition {
    ModeTransition::new(Mode::from_c_int(from), Mode::from_c_int(to))
}

/// FFI: Check if entering insert.
///
/// # Safety
/// `trans` must be valid or null.
#[no_mangle]
pub unsafe extern "C" fn rs_mode_entering_insert(trans: *const ModeTransition) -> c_int {
    if trans.is_null() {
        return 0;
    }
    c_int::from((*trans).entering_insert())
}

/// FFI: Check if leaving insert.
///
/// # Safety
/// `trans` must be valid or null.
#[no_mangle]
pub unsafe extern "C" fn rs_mode_leaving_insert(trans: *const ModeTransition) -> c_int {
    if trans.is_null() {
        return 0;
    }
    c_int::from((*trans).leaving_insert())
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mode() {
        assert!(Mode::Visual.is_visual());
        assert!(Mode::VisualLine.is_visual());
        assert!(!Mode::Normal.is_visual());

        assert!(Mode::Insert.allows_editing());
        assert!(Mode::Replace.allows_editing());
        assert!(!Mode::Normal.allows_editing());

        assert_eq!(Mode::Normal.char(), b'n');
        assert_eq!(Mode::Insert.char(), b'i');
    }

    #[test]
    fn test_mode_modifiers() {
        let mut mods = ModeModifiers::new();
        assert!(!mods.any_active());

        mods.recording = true;
        assert!(mods.any_active());
    }

    #[test]
    fn test_mode_state() {
        let mut state = ModeState::new();
        assert_eq!(state.get_mode(), Mode::Normal);

        state.change_to(Mode::Insert, 1000);
        assert_eq!(state.get_mode(), Mode::Insert);
        assert_eq!(state.get_prev_mode(), Mode::Normal);
        assert!(state.just_changed());
    }

    #[test]
    fn test_mode_transition() {
        let trans = ModeTransition::new(Mode::Normal, Mode::Insert);
        assert!(trans.entering_insert());
        assert!(!trans.leaving_insert());

        let trans = ModeTransition::new(Mode::Insert, Mode::Normal);
        assert!(!trans.entering_insert());
        assert!(trans.leaving_insert());
    }
}
