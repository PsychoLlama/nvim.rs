//! Command line mode state machine
//!
//! This module provides helpers for managing command-line mode transitions,
//! key action classification, and mode-specific state tracking.

#![allow(unsafe_code)]
#![allow(clippy::doc_markdown)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::missing_const_for_fn)]
#![allow(clippy::match_same_arms)]
#![allow(clippy::if_same_then_else)]

use std::ffi::c_int;

// =============================================================================
// Command Line Mode States
// =============================================================================

/// Command line mode state.
///
/// Represents the different states the command line can be in during editing.
#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CmdlineMode {
    /// Normal command line editing
    #[default]
    Normal = 0,
    /// Incremental search active
    Incsearch = 1,
    /// Wildcard completion active
    Wildmenu = 2,
    /// History navigation active
    History = 3,
    /// Digraph input (Ctrl-K)
    Digraph = 4,
    /// Special character input (Ctrl-V)
    SpecialChar = 5,
    /// Expression evaluation (Ctrl-\ e)
    Expression = 6,
    /// Command window editing
    Cmdwin = 7,
}

impl CmdlineMode {
    /// Convert from raw C integer.
    #[must_use]
    pub const fn from_raw(value: c_int) -> Option<Self> {
        match value {
            0 => Some(Self::Normal),
            1 => Some(Self::Incsearch),
            2 => Some(Self::Wildmenu),
            3 => Some(Self::History),
            4 => Some(Self::Digraph),
            5 => Some(Self::SpecialChar),
            6 => Some(Self::Expression),
            7 => Some(Self::Cmdwin),
            _ => None,
        }
    }

    /// Convert to raw C integer.
    #[must_use]
    pub const fn to_raw(self) -> c_int {
        self as c_int
    }

    /// Check if this mode allows typing.
    #[must_use]
    pub const fn allows_typing(self) -> bool {
        matches!(
            self,
            Self::Normal | Self::Incsearch | Self::Digraph | Self::SpecialChar
        )
    }

    /// Check if this mode needs special key handling.
    #[must_use]
    pub const fn needs_special_key_handling(self) -> bool {
        matches!(
            self,
            Self::Wildmenu | Self::History | Self::Digraph | Self::SpecialChar
        )
    }

    /// Check if this mode shows a menu.
    #[must_use]
    pub const fn shows_menu(self) -> bool {
        matches!(self, Self::Wildmenu)
    }
}

// =============================================================================
// Key Action Classification
// =============================================================================

/// Action to take for a key in command line mode.
#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyAction {
    /// Insert the character
    Insert = 0,
    /// Delete character before cursor
    Backspace = 1,
    /// Delete character at cursor
    Delete = 2,
    /// Move cursor left
    CursorLeft = 3,
    /// Move cursor right
    CursorRight = 4,
    /// Move to start of line
    Home = 5,
    /// Move to end of line
    End = 6,
    /// Delete word before cursor
    DeleteWord = 7,
    /// Delete to start of line
    DeleteToStart = 8,
    /// Delete to end of line
    DeleteToEnd = 9,
    /// Navigate history up
    HistoryUp = 10,
    /// Navigate history down
    HistoryDown = 11,
    /// Trigger completion
    Complete = 12,
    /// Accept command
    Accept = 13,
    /// Cancel command
    Cancel = 14,
    /// Toggle overstrike mode
    ToggleOvr = 15,
    /// Start digraph input
    StartDigraph = 16,
    /// Start special char input
    StartSpecial = 17,
    /// Move word left
    WordLeft = 18,
    /// Move word right
    WordRight = 19,
    /// Paste from register
    PasteRegister = 20,
    /// Open command window
    OpenCmdwin = 21,
    /// Add char from match (Ctrl-L)
    AddMatchChar = 22,
    /// Do nothing
    Ignore = 23,
    /// Unknown - pass through
    Unknown = 24,
}

impl KeyAction {
    /// Convert from raw C integer.
    #[must_use]
    pub const fn from_raw(value: c_int) -> Option<Self> {
        match value {
            0 => Some(Self::Insert),
            1 => Some(Self::Backspace),
            2 => Some(Self::Delete),
            3 => Some(Self::CursorLeft),
            4 => Some(Self::CursorRight),
            5 => Some(Self::Home),
            6 => Some(Self::End),
            7 => Some(Self::DeleteWord),
            8 => Some(Self::DeleteToStart),
            9 => Some(Self::DeleteToEnd),
            10 => Some(Self::HistoryUp),
            11 => Some(Self::HistoryDown),
            12 => Some(Self::Complete),
            13 => Some(Self::Accept),
            14 => Some(Self::Cancel),
            15 => Some(Self::ToggleOvr),
            16 => Some(Self::StartDigraph),
            17 => Some(Self::StartSpecial),
            18 => Some(Self::WordLeft),
            19 => Some(Self::WordRight),
            20 => Some(Self::PasteRegister),
            21 => Some(Self::OpenCmdwin),
            22 => Some(Self::AddMatchChar),
            23 => Some(Self::Ignore),
            24 => Some(Self::Unknown),
            _ => None,
        }
    }

    /// Convert to raw C integer.
    #[must_use]
    pub const fn to_raw(self) -> c_int {
        self as c_int
    }

    /// Check if this action modifies the command line.
    #[must_use]
    pub const fn modifies_cmdline(self) -> bool {
        matches!(
            self,
            Self::Insert
                | Self::Backspace
                | Self::Delete
                | Self::DeleteWord
                | Self::DeleteToStart
                | Self::DeleteToEnd
                | Self::PasteRegister
                | Self::AddMatchChar
        )
    }

    /// Check if this action moves the cursor.
    #[must_use]
    pub const fn moves_cursor(self) -> bool {
        matches!(
            self,
            Self::CursorLeft
                | Self::CursorRight
                | Self::Home
                | Self::End
                | Self::WordLeft
                | Self::WordRight
        )
    }

    /// Check if this action is a history navigation.
    #[must_use]
    pub const fn is_history_nav(self) -> bool {
        matches!(self, Self::HistoryUp | Self::HistoryDown)
    }

    /// Check if this action terminates command line mode.
    #[must_use]
    pub const fn terminates_cmdline(self) -> bool {
        matches!(self, Self::Accept | Self::Cancel)
    }
}

// =============================================================================
// Key Code Constants
// =============================================================================

/// Key codes for command line (matching keycodes.h)
pub mod keys {
    use std::ffi::c_int;

    // Control characters
    pub const CTRL_A: c_int = 1;
    pub const CTRL_B: c_int = 2;
    pub const CTRL_C: c_int = 3;
    pub const CTRL_D: c_int = 4;
    pub const CTRL_E: c_int = 5;
    pub const CTRL_F: c_int = 6;
    pub const CTRL_G: c_int = 7;
    pub const CTRL_H: c_int = 8; // Backspace
    pub const CTRL_I: c_int = 9; // Tab
    pub const CTRL_J: c_int = 10; // Newline
    pub const CTRL_K: c_int = 11; // Digraph
    pub const CTRL_L: c_int = 12; // Add match char
    pub const CTRL_M: c_int = 13; // Enter
    pub const CTRL_N: c_int = 14; // Next
    pub const CTRL_O: c_int = 15;
    pub const CTRL_P: c_int = 16; // Previous
    pub const CTRL_Q: c_int = 17; // Ctrl-V alias
    pub const CTRL_R: c_int = 18; // Paste register
    pub const CTRL_S: c_int = 19;
    pub const CTRL_T: c_int = 20;
    pub const CTRL_U: c_int = 21; // Delete to start
    pub const CTRL_V: c_int = 22; // Special char
    pub const CTRL_W: c_int = 23; // Delete word
    pub const CTRL_X: c_int = 24;
    pub const CTRL_Y: c_int = 25;
    pub const CTRL_Z: c_int = 26;
    pub const ESC: c_int = 27;
    pub const CTRL_BSL: c_int = 28; // Ctrl-backslash

    // Special keys (SPECIAL + code)
    pub const K_SPECIAL: c_int = 0x100;
    pub const K_UP: c_int = K_SPECIAL + 0x48;
    pub const K_DOWN: c_int = K_SPECIAL + 0x49;
    pub const K_LEFT: c_int = K_SPECIAL + 0x4B;
    pub const K_RIGHT: c_int = K_SPECIAL + 0x4C;
    pub const K_HOME: c_int = K_SPECIAL + 0x47;
    pub const K_END: c_int = K_SPECIAL + 0x4F;
    pub const K_DEL: c_int = K_SPECIAL + 0x53;
    pub const K_BS: c_int = K_SPECIAL + 0x5E + 2;
    pub const K_INS: c_int = K_SPECIAL + 0x52;
    pub const K_PAGEUP: c_int = K_SPECIAL + 0x5E + 7;
    pub const K_PAGEDOWN: c_int = K_SPECIAL + 0x5E + 8;

    // Shifted keys
    pub const K_S_UP: c_int = K_SPECIAL + 0x5E + 11;
    pub const K_S_DOWN: c_int = K_SPECIAL + 0x5E + 12;
    pub const K_S_LEFT: c_int = K_SPECIAL + 0x5E + 13;
    pub const K_S_RIGHT: c_int = K_SPECIAL + 0x5E + 14;
    pub const K_S_HOME: c_int = K_SPECIAL + 0x5E + 15;
    pub const K_S_END: c_int = K_SPECIAL + 0x5E + 16;

    // Control+Arrow
    pub const K_C_LEFT: c_int = K_SPECIAL + 0x5E + 39;
    pub const K_C_RIGHT: c_int = K_SPECIAL + 0x5E + 40;
    pub const K_C_HOME: c_int = K_SPECIAL + 0x5E + 41;
    pub const K_C_END: c_int = K_SPECIAL + 0x5E + 42;

    // Keypad
    pub const K_KENTER: c_int = K_SPECIAL + 0x5E;
    pub const K_KDEL: c_int = K_SPECIAL + 0x5E + 3;

    // Special events
    pub const K_IGNORE: c_int = K_SPECIAL + 0x5E + 4;
    pub const K_NOP: c_int = K_SPECIAL + 0x5E + 6;
    pub const K_EVENT: c_int = K_SPECIAL + 0x5E + 57;
    pub const K_COMMAND: c_int = K_SPECIAL + 0x5E + 58;
    pub const K_LUA: c_int = K_SPECIAL + 0x5E + 59;

    // Wild key
    pub const K_WILD: c_int = K_SPECIAL + 0x5E + 75;
}

/// Classify a key into an action for normal command line mode.
#[must_use]
pub const fn classify_key(key: c_int) -> KeyAction {
    match key {
        // Control characters
        keys::CTRL_H | keys::K_BS => KeyAction::Backspace,
        keys::K_DEL | keys::K_KDEL => KeyAction::Delete,
        keys::CTRL_W => KeyAction::DeleteWord,
        keys::CTRL_U => KeyAction::DeleteToStart,
        keys::CTRL_K => KeyAction::StartDigraph,
        keys::CTRL_V | keys::CTRL_Q => KeyAction::StartSpecial,
        keys::CTRL_R => KeyAction::PasteRegister,
        keys::CTRL_L => KeyAction::AddMatchChar,
        keys::CTRL_M | keys::CTRL_J | keys::K_KENTER => KeyAction::Accept,
        keys::ESC | keys::CTRL_C => KeyAction::Cancel,

        // Arrow keys
        keys::K_LEFT | keys::K_S_LEFT => KeyAction::CursorLeft,
        keys::K_RIGHT | keys::K_S_RIGHT => KeyAction::CursorRight,
        keys::K_UP | keys::K_S_UP | keys::K_PAGEUP => KeyAction::HistoryUp,
        keys::K_DOWN | keys::K_S_DOWN | keys::K_PAGEDOWN => KeyAction::HistoryDown,

        // Home/End
        keys::K_HOME | keys::K_S_HOME | keys::K_C_HOME | keys::CTRL_B => KeyAction::Home,
        keys::K_END | keys::K_S_END | keys::K_C_END | keys::CTRL_E => KeyAction::End,

        // Word movement
        keys::K_C_LEFT => KeyAction::WordLeft,
        keys::K_C_RIGHT => KeyAction::WordRight,

        // Insert toggle
        keys::K_INS => KeyAction::ToggleOvr,

        // Completion
        keys::CTRL_I | keys::K_WILD => KeyAction::Complete,

        // Ctrl-N/P for history
        keys::CTRL_N => KeyAction::HistoryDown,
        keys::CTRL_P => KeyAction::HistoryUp,

        // Ignore these
        keys::K_IGNORE | keys::K_NOP | keys::K_EVENT | keys::K_COMMAND | keys::K_LUA => {
            KeyAction::Ignore
        }

        // Everything else is either insert or unknown
        _ => {
            if key >= 32 && key < 256 {
                KeyAction::Insert
            } else if key > 256 {
                // Special key we don't handle
                KeyAction::Unknown
            } else {
                // Control character we don't handle
                KeyAction::Unknown
            }
        }
    }
}

/// Classify a key for wildmenu mode.
#[must_use]
pub const fn classify_key_wildmenu(key: c_int) -> KeyAction {
    match key {
        // Navigation within wildmenu
        keys::K_LEFT | keys::CTRL_P => KeyAction::CursorLeft, // Previous item
        keys::K_RIGHT | keys::CTRL_N => KeyAction::CursorRight, // Next item
        keys::K_UP => KeyAction::HistoryUp,                   // Exit wildmenu, history up
        keys::K_DOWN => KeyAction::HistoryDown,               // Exit wildmenu, history down

        // Accept current selection
        keys::CTRL_M | keys::CTRL_J | keys::K_KENTER | keys::CTRL_I => KeyAction::Accept,

        // Cancel wildmenu
        keys::ESC | keys::CTRL_C => KeyAction::Cancel,

        // Regular key ends wildmenu and inserts
        _ => classify_key(key),
    }
}

// =============================================================================
// Mode Transition Logic
// =============================================================================

/// Result of a mode transition.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ModeTransition {
    /// The new mode
    pub new_mode: CmdlineMode,
    /// Whether the command line was changed
    pub changed: bool,
    /// Whether to continue processing
    pub continue_processing: bool,
}

impl ModeTransition {
    /// Stay in current mode, mark as changed.
    #[must_use]
    pub const fn changed(mode: CmdlineMode) -> Self {
        Self {
            new_mode: mode,
            changed: true,
            continue_processing: true,
        }
    }

    /// Stay in current mode, not changed.
    #[must_use]
    pub const fn unchanged(mode: CmdlineMode) -> Self {
        Self {
            new_mode: mode,
            changed: false,
            continue_processing: true,
        }
    }

    /// Transition to new mode.
    #[must_use]
    pub const fn transition(new_mode: CmdlineMode, changed: bool) -> Self {
        Self {
            new_mode,
            changed,
            continue_processing: true,
        }
    }

    /// Stop processing (accept or cancel).
    #[must_use]
    pub const fn stop(changed: bool) -> Self {
        Self {
            new_mode: CmdlineMode::Normal,
            changed,
            continue_processing: false,
        }
    }
}

/// Handle a key action and determine mode transition.
#[must_use]
pub const fn handle_action_transition(
    current_mode: CmdlineMode,
    action: KeyAction,
) -> ModeTransition {
    match action {
        KeyAction::Accept | KeyAction::Cancel => ModeTransition::stop(false),

        KeyAction::StartDigraph => ModeTransition::transition(CmdlineMode::Digraph, false),

        KeyAction::StartSpecial => ModeTransition::transition(CmdlineMode::SpecialChar, false),

        KeyAction::Complete => ModeTransition::transition(CmdlineMode::Wildmenu, false),

        KeyAction::HistoryUp | KeyAction::HistoryDown => {
            // History navigation always transitions to History mode
            // Note: In wildmenu mode, this will also exit wildmenu
            ModeTransition::transition(CmdlineMode::History, true)
        }

        KeyAction::Insert
        | KeyAction::Backspace
        | KeyAction::Delete
        | KeyAction::DeleteWord
        | KeyAction::DeleteToStart
        | KeyAction::DeleteToEnd
        | KeyAction::PasteRegister
        | KeyAction::AddMatchChar => {
            // Modifying actions return to normal mode
            ModeTransition::changed(CmdlineMode::Normal)
        }

        KeyAction::CursorLeft
        | KeyAction::CursorRight
        | KeyAction::Home
        | KeyAction::End
        | KeyAction::WordLeft
        | KeyAction::WordRight
        | KeyAction::ToggleOvr => {
            // Cursor movement doesn't change mode
            ModeTransition::unchanged(current_mode)
        }

        KeyAction::OpenCmdwin => ModeTransition::transition(CmdlineMode::Cmdwin, false),

        KeyAction::Ignore => ModeTransition::unchanged(current_mode),

        KeyAction::Unknown => ModeTransition::unchanged(current_mode),
    }
}

// =============================================================================
// FFI Exports
// =============================================================================

/// Classify a key into an action.
#[unsafe(no_mangle)]
pub extern "C" fn rs_cmdline_classify_key(key: c_int) -> c_int {
    classify_key(key).to_raw()
}

/// Classify a key for wildmenu mode.
#[unsafe(no_mangle)]
pub extern "C" fn rs_cmdline_classify_key_wildmenu(key: c_int) -> c_int {
    classify_key_wildmenu(key).to_raw()
}

/// Check if an action modifies the command line.
#[unsafe(no_mangle)]
pub extern "C" fn rs_cmdline_action_modifies(action: c_int) -> c_int {
    KeyAction::from_raw(action).map_or(0, |a| c_int::from(a.modifies_cmdline()))
}

/// Check if an action moves the cursor.
#[unsafe(no_mangle)]
pub extern "C" fn rs_cmdline_action_moves_cursor(action: c_int) -> c_int {
    KeyAction::from_raw(action).map_or(0, |a| c_int::from(a.moves_cursor()))
}

/// Check if an action terminates command line mode.
#[unsafe(no_mangle)]
pub extern "C" fn rs_cmdline_action_terminates(action: c_int) -> c_int {
    KeyAction::from_raw(action).map_or(0, |a| c_int::from(a.terminates_cmdline()))
}

/// Check if mode allows typing.
#[unsafe(no_mangle)]
pub extern "C" fn rs_cmdline_mode_allows_typing(mode: c_int) -> c_int {
    CmdlineMode::from_raw(mode).map_or(0, |m| c_int::from(m.allows_typing()))
}

/// Check if mode shows a menu.
#[unsafe(no_mangle)]
pub extern "C" fn rs_cmdline_mode_shows_menu(mode: c_int) -> c_int {
    CmdlineMode::from_raw(mode).map_or(0, |m| c_int::from(m.shows_menu()))
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cmdline_mode_conversion() {
        for mode in [
            CmdlineMode::Normal,
            CmdlineMode::Incsearch,
            CmdlineMode::Wildmenu,
            CmdlineMode::History,
            CmdlineMode::Digraph,
            CmdlineMode::SpecialChar,
            CmdlineMode::Expression,
            CmdlineMode::Cmdwin,
        ] {
            let raw = mode.to_raw();
            assert_eq!(CmdlineMode::from_raw(raw), Some(mode));
        }
    }

    #[test]
    fn test_key_action_conversion() {
        for action in [
            KeyAction::Insert,
            KeyAction::Backspace,
            KeyAction::Delete,
            KeyAction::Accept,
            KeyAction::Cancel,
        ] {
            let raw = action.to_raw();
            assert_eq!(KeyAction::from_raw(raw), Some(action));
        }
    }

    #[test]
    fn test_classify_key_basic() {
        assert_eq!(classify_key(keys::CTRL_H), KeyAction::Backspace);
        assert_eq!(classify_key(keys::K_BS), KeyAction::Backspace);
        assert_eq!(classify_key(keys::K_DEL), KeyAction::Delete);
        assert_eq!(classify_key(keys::CTRL_W), KeyAction::DeleteWord);
        assert_eq!(classify_key(keys::CTRL_U), KeyAction::DeleteToStart);
        assert_eq!(classify_key(keys::CTRL_M), KeyAction::Accept);
        assert_eq!(classify_key(keys::ESC), KeyAction::Cancel);
        assert_eq!(classify_key(keys::K_LEFT), KeyAction::CursorLeft);
        assert_eq!(classify_key(keys::K_RIGHT), KeyAction::CursorRight);
        assert_eq!(classify_key(keys::K_UP), KeyAction::HistoryUp);
        assert_eq!(classify_key(keys::K_DOWN), KeyAction::HistoryDown);
        assert_eq!(classify_key(keys::K_HOME), KeyAction::Home);
        assert_eq!(classify_key(keys::K_END), KeyAction::End);
    }

    #[test]
    fn test_classify_key_printable() {
        assert_eq!(classify_key(c_int::from(b'a')), KeyAction::Insert);
        assert_eq!(classify_key(c_int::from(b'Z')), KeyAction::Insert);
        assert_eq!(classify_key(c_int::from(b' ')), KeyAction::Insert);
        assert_eq!(classify_key(c_int::from(b'~')), KeyAction::Insert);
    }

    #[test]
    fn test_action_properties() {
        assert!(KeyAction::Insert.modifies_cmdline());
        assert!(KeyAction::Backspace.modifies_cmdline());
        assert!(!KeyAction::CursorLeft.modifies_cmdline());
        assert!(!KeyAction::Accept.modifies_cmdline());

        assert!(KeyAction::CursorLeft.moves_cursor());
        assert!(KeyAction::Home.moves_cursor());
        assert!(!KeyAction::Insert.moves_cursor());

        assert!(KeyAction::HistoryUp.is_history_nav());
        assert!(KeyAction::HistoryDown.is_history_nav());
        assert!(!KeyAction::Insert.is_history_nav());

        assert!(KeyAction::Accept.terminates_cmdline());
        assert!(KeyAction::Cancel.terminates_cmdline());
        assert!(!KeyAction::Insert.terminates_cmdline());
    }

    #[test]
    fn test_mode_transition_modifying() {
        let result = handle_action_transition(CmdlineMode::Normal, KeyAction::Insert);
        assert_eq!(result.new_mode, CmdlineMode::Normal);
        assert!(result.changed);
        assert!(result.continue_processing);
    }

    #[test]
    fn test_mode_transition_digraph() {
        let result = handle_action_transition(CmdlineMode::Normal, KeyAction::StartDigraph);
        assert_eq!(result.new_mode, CmdlineMode::Digraph);
        assert!(!result.changed);
    }

    #[test]
    fn test_mode_transition_accept() {
        let result = handle_action_transition(CmdlineMode::Normal, KeyAction::Accept);
        assert!(!result.continue_processing);
    }

    #[test]
    fn test_mode_allows_typing() {
        assert!(CmdlineMode::Normal.allows_typing());
        assert!(CmdlineMode::Incsearch.allows_typing());
        assert!(!CmdlineMode::Wildmenu.allows_typing());
        assert!(!CmdlineMode::History.allows_typing());
    }
}
