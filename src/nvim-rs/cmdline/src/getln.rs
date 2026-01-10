//! Main command line entry loop functionality
//!
//! This module provides the core types and utilities for the command-line
//! entry loop, including the `CommandLineState` structure, key handling,
//! and incremental search state management.

#![allow(clippy::doc_markdown)]
#![allow(clippy::cast_possible_wrap)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_sign_loss)]

use std::ffi::c_int;

use crate::viewstate::{IncsearchViewState, Position, ViewState};

// =============================================================================
// Return Codes
// =============================================================================

/// Return value when handling keys in command-line mode.
pub mod result {
    use std::ffi::c_int;

    /// Command line not changed - skip further processing
    pub const NOT_CHANGED: c_int = 1;
    /// Command line changed - update display
    pub const CHANGED: c_int = 2;
    /// Go back to normal mode
    pub const GOTO_NORMAL_MODE: c_int = 3;
    /// Process next key
    pub const PROCESS_NEXT_KEY: c_int = 4;
}

/// Key handling result type.
#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyResult {
    /// Command line not changed
    NotChanged = 1,
    /// Command line changed
    Changed = 2,
    /// Go to normal mode
    GotoNormalMode = 3,
    /// Process next key
    ProcessNextKey = 4,
}

impl KeyResult {
    /// Create from raw C integer.
    #[must_use]
    pub const fn from_raw(value: c_int) -> Option<Self> {
        match value {
            1 => Some(Self::NotChanged),
            2 => Some(Self::Changed),
            3 => Some(Self::GotoNormalMode),
            4 => Some(Self::ProcessNextKey),
            _ => None,
        }
    }

    /// Convert to raw C integer.
    #[must_use]
    pub const fn to_raw(self) -> c_int {
        self as c_int
    }

    /// Check if result indicates command line changed.
    #[must_use]
    pub const fn is_changed(self) -> bool {
        matches!(self, Self::Changed)
    }

    /// Check if result indicates goto normal mode.
    #[must_use]
    pub const fn is_goto_normal(self) -> bool {
        matches!(self, Self::GotoNormalMode)
    }
}

// =============================================================================
// Incsearch State
// =============================================================================

/// Magic override values for incsearch.
#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum MagicOverride {
    /// No override
    #[default]
    None = 0,
    /// Force magic on
    On = 1,
    /// Force magic off
    Off = 2,
}

impl MagicOverride {
    /// Create from raw C integer.
    #[must_use]
    pub const fn from_raw(value: c_int) -> Self {
        match value {
            1 => Self::On,
            2 => Self::Off,
            _ => Self::None,
        }
    }

    /// Convert to raw C integer.
    #[must_use]
    pub const fn to_raw(self) -> c_int {
        self as c_int
    }
}

/// Extended incremental search state for command-line mode.
///
/// This structure holds all state needed during incremental search,
/// including match positions, view state, and magic override.
#[derive(Debug, Clone, Copy, Default)]
#[repr(C)]
pub struct IncsearchState {
    /// Window handle where this state is valid
    pub winid: i64,
    /// Position where search started
    pub search_start: Position,
    /// Saved cursor position (for restoration)
    pub save_cursor: Position,
    /// Initial view state before incsearch
    pub init_viewstate: ViewState,
    /// Previous view state (for detecting changes)
    pub old_viewstate: ViewState,
    /// Start of current match
    pub match_start: Position,
    /// End of current match
    pub match_end: Position,
    /// Whether incsearch has been performed
    pub did_incsearch: bool,
    /// Whether incsearch is postponed (waiting for input)
    pub incsearch_postponed: bool,
    /// Saved magic_overruled value
    pub magic_overruled_save: MagicOverride,
}

impl IncsearchState {
    /// Create a new incsearch state.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            winid: 0,
            search_start: Position::invalid(),
            save_cursor: Position::invalid(),
            init_viewstate: ViewState::new(),
            old_viewstate: ViewState::new(),
            match_start: Position::invalid(),
            match_end: Position::invalid(),
            did_incsearch: false,
            incsearch_postponed: false,
            magic_overruled_save: MagicOverride::None,
        }
    }

    /// Reset the state to initial values.
    #[allow(clippy::missing_const_for_fn)]
    pub fn reset(&mut self) {
        *self = Self::new();
    }

    /// Check if a match has been found.
    #[must_use]
    pub const fn has_match(&self) -> bool {
        self.match_start.is_valid()
    }

    /// Clear match positions.
    pub fn clear_match(&mut self) {
        self.match_start.clear();
        self.match_end.clear();
    }

    /// Check if incsearch state is valid for the given window.
    #[must_use]
    pub const fn is_valid_for_window(&self, winid: i64) -> bool {
        self.winid == winid
    }

    /// Copy from IncsearchViewState.
    #[allow(clippy::missing_const_for_fn)]
    pub fn from_view_state(&mut self, vs: &IncsearchViewState) {
        self.winid = vs.winid;
        self.init_viewstate = vs.init_viewstate;
        self.old_viewstate = vs.old_viewstate;
        self.search_start = vs.search_start;
        self.save_cursor = vs.save_cursor;
        self.match_start = vs.match_start;
        self.match_end = vs.match_end;
        self.did_incsearch = vs.did_incsearch;
        self.incsearch_postponed = vs.incsearch_postponed;
    }

    /// Copy to IncsearchViewState.
    #[must_use]
    pub const fn to_view_state(&self) -> IncsearchViewState {
        IncsearchViewState {
            winid: self.winid,
            init_viewstate: self.init_viewstate,
            old_viewstate: self.old_viewstate,
            search_start: self.search_start,
            save_cursor: self.save_cursor,
            match_start: self.match_start,
            match_end: self.match_end,
            did_incsearch: self.did_incsearch,
            incsearch_postponed: self.incsearch_postponed,
        }
    }
}

// =============================================================================
// Command Line State Flags
// =============================================================================

/// Flags for command line state.
#[derive(Debug, Clone, Copy, Default)]
#[allow(clippy::struct_excessive_bools)]
pub struct CmdlineFlags {
    /// ESC was just typed
    pub gotesc: bool,
    /// Check for abbreviation when true
    pub do_abbr: bool,
    /// Did wild_list() recently
    pub did_wild_list: bool,
    /// Some key was typed (not from mapping)
    pub some_key_typed: bool,
    /// Ignore mouse drag/release events
    pub ignore_drag_release: bool,
    /// Break on Ctrl-C
    pub break_ctrl_c: bool,
    /// Skip PUM redraw
    pub skip_pum_redraw: bool,
    /// CmdlineLeavePre was triggered
    pub event_cmdlineleavepre_triggered: bool,
    /// History navigation occurred
    pub did_hist_navigate: bool,
}

impl CmdlineFlags {
    /// Create new flags with default values.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            gotesc: false,
            do_abbr: true,
            did_wild_list: false,
            some_key_typed: false,
            ignore_drag_release: true,
            break_ctrl_c: false,
            skip_pum_redraw: false,
            event_cmdlineleavepre_triggered: false,
            did_hist_navigate: false,
        }
    }

    /// Reset to initial state for new command line entry.
    #[allow(clippy::missing_const_for_fn)]
    pub fn reset(&mut self) {
        *self = Self::new();
    }
}

// =============================================================================
// Command Line State
// =============================================================================

/// Full command-line state structure.
///
/// This mirrors the C `CommandLineState` structure and provides the complete
/// state for command-line editing, including input processing, history,
/// completion, and incremental search.
#[derive(Debug, Clone, Copy, Default)]
pub struct CommandLineStateInfo {
    /// First character determining command type (: / ? = > @ or NUL)
    pub firstc: c_int,
    /// Count argument (for incremental search)
    pub count: c_int,
    /// Indent for inside conditionals
    pub indent: c_int,
    /// Current character being processed
    pub c: c_int,
    /// History count (index into history)
    pub hiscnt: c_int,
    /// Saved history count before navigation
    pub save_hiscnt: c_int,
    /// History type being used
    pub histype: c_int,
    /// Wildcard mode index
    pub wim_index: c_int,
    /// Saved msg_scroll value
    pub save_msg_scroll: c_int,
    /// Saved State value when called
    pub save_state: c_int,
    /// Previous cursor position in command buffer
    pub prev_cmdpos: c_int,
    /// Command-line type character for events
    pub cmdline_type: c_int,
    /// Flags
    pub flags: CmdlineFlags,
    /// Incremental search state
    pub is_state: IncsearchState,
}

impl CommandLineStateInfo {
    /// Create a new command line state.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            firstc: 0,
            count: 0,
            indent: 0,
            c: 0,
            hiscnt: 0,
            save_hiscnt: 0,
            histype: 0,
            wim_index: 0,
            save_msg_scroll: 0,
            save_state: 0,
            prev_cmdpos: -1,
            cmdline_type: 0,
            flags: CmdlineFlags::new(),
            is_state: IncsearchState::new(),
        }
    }

    /// Initialize for a new command line entry.
    pub fn init(&mut self, firstc: c_int, count: c_int, indent: c_int) {
        self.firstc = firstc;
        self.count = count;
        self.indent = indent;
        self.c = 0;
        self.prev_cmdpos = -1;
        self.flags.reset();
        self.is_state.reset();

        // Determine command-line type for events
        self.cmdline_type = if firstc > 0 {
            firstc
        } else {
            c_int::from(b'-')
        };
    }

    /// Get the effective first character.
    #[must_use]
    pub const fn effective_firstc(&self) -> c_int {
        if self.firstc == -1 {
            0
        } else {
            self.firstc
        }
    }

    /// Check if this is a search command.
    #[must_use]
    pub const fn is_search(&self) -> bool {
        let firstc = self.effective_firstc();
        firstc == b'/' as c_int || firstc == b'?' as c_int
    }

    /// Check if this is an input() function call.
    #[must_use]
    pub const fn is_input_fn(&self) -> bool {
        self.effective_firstc() == b'@' as c_int
    }

    /// Check if escape was used to abort.
    #[must_use]
    pub const fn was_escaped(&self) -> bool {
        self.flags.gotesc
    }
}

// =============================================================================
// History Navigation Helpers
// =============================================================================

/// History navigation direction.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HistoryDirection {
    /// Navigate to older history entries
    Older,
    /// Navigate to newer history entries
    Newer,
}

impl HistoryDirection {
    /// Determine direction from key code.
    #[must_use]
    pub const fn from_key(key: c_int) -> Option<Self> {
        // K_DOWN, K_S_DOWN, Ctrl_N, K_PAGEDOWN, K_KPAGEDOWN
        const K_DOWN: c_int = 0x100 + 0x48; // Approximate values
        const K_S_DOWN: c_int = 0x100 + 0x49;
        const K_PAGEDOWN: c_int = 0x100 + 0x4a;
        const K_KPAGEDOWN: c_int = 0x100 + 0x4b;
        const CTRL_N: c_int = 14;
        const K_UP: c_int = 0x100 + 0x44;
        const K_S_UP: c_int = 0x100 + 0x45;
        const K_PAGEUP: c_int = 0x100 + 0x46;
        const K_KPAGEUP: c_int = 0x100 + 0x47;
        const CTRL_P: c_int = 16;

        match key {
            K_DOWN | K_S_DOWN | CTRL_N | K_PAGEDOWN | K_KPAGEDOWN => Some(Self::Newer),
            K_UP | K_S_UP | CTRL_P | K_PAGEUP | K_KPAGEUP => Some(Self::Older),
            _ => None,
        }
    }

    /// Check if direction is newer.
    #[must_use]
    pub const fn is_newer(self) -> bool {
        matches!(self, Self::Newer)
    }

    /// Check if direction is older.
    #[must_use]
    pub const fn is_older(self) -> bool {
        matches!(self, Self::Older)
    }
}

// =============================================================================
// Cursor Movement Helpers
// =============================================================================

/// Movement direction for cursor operations.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CursorDirection {
    /// Move left
    Left,
    /// Move right
    Right,
}

/// Movement type for cursor operations.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MovementType {
    /// Move by character
    Char,
    /// Move by word
    Word,
    /// Move to start/end
    Edge,
}

/// Determine movement type from key code.
#[must_use]
pub const fn movement_type_from_key(key: c_int, mod_mask: c_int) -> MovementType {
    const MOD_MASK_SHIFT: c_int = 0x02;
    const MOD_MASK_CTRL: c_int = 0x04;

    const K_S_LEFT: c_int = 0x100 + 0x40;
    const K_S_RIGHT: c_int = 0x100 + 0x41;
    const K_C_LEFT: c_int = 0x100 + 0x42;
    const K_C_RIGHT: c_int = 0x100 + 0x43;

    if key == K_S_LEFT || key == K_S_RIGHT || key == K_C_LEFT || key == K_C_RIGHT {
        return MovementType::Word;
    }

    if (mod_mask & (MOD_MASK_SHIFT | MOD_MASK_CTRL)) != 0 {
        return MovementType::Word;
    }

    MovementType::Char
}

// =============================================================================
// Key Classification
// =============================================================================

/// Special key constants for command-line mode.
pub mod keys {
    use std::ffi::c_int;

    // Control keys
    pub const CTRL_A: c_int = 1;
    pub const CTRL_B: c_int = 2;
    pub const CTRL_C: c_int = 3;
    pub const CTRL_D: c_int = 4;
    pub const CTRL_E: c_int = 5;
    pub const CTRL_G: c_int = 7;
    pub const CTRL_H: c_int = 8;
    pub const CTRL_K: c_int = 11;
    pub const CTRL_L: c_int = 12;
    pub const CTRL_N: c_int = 14;
    pub const CTRL_P: c_int = 16;
    pub const CTRL_Q: c_int = 17;
    pub const CTRL_R: c_int = 18;
    pub const CTRL_T: c_int = 20;
    pub const CTRL_U: c_int = 21;
    pub const CTRL_V: c_int = 22;
    pub const CTRL_W: c_int = 23;
    pub const CTRL_Z: c_int = 26;
    pub const CTRL_BSL: c_int = 28;
    pub const CTRL_RSB: c_int = 29;
    pub const CTRL_HAT: c_int = 30;
    pub const CTRL_UNDERSCORE: c_int = 31;
    pub const ESC: c_int = 27;
    pub const NUL: c_int = 0;
    pub const NL: c_int = 10;

    // Special keys (approximate values)
    pub const K_BS: c_int = 0x100 + 0x01;
    pub const K_DEL: c_int = 0x100 + 0x02;
    pub const K_KDEL: c_int = 0x100 + 0x03;
    pub const K_INS: c_int = 0x100 + 0x04;
    pub const K_KINS: c_int = 0x100 + 0x05;
    pub const K_HOME: c_int = 0x100 + 0x06;
    pub const K_KHOME: c_int = 0x100 + 0x07;
    pub const K_END: c_int = 0x100 + 0x08;
    pub const K_KEND: c_int = 0x100 + 0x09;
    pub const K_LEFT: c_int = 0x100 + 0x10;
    pub const K_RIGHT: c_int = 0x100 + 0x11;
    pub const K_UP: c_int = 0x100 + 0x12;
    pub const K_DOWN: c_int = 0x100 + 0x13;
    pub const K_S_HOME: c_int = 0x100 + 0x14;
    pub const K_S_END: c_int = 0x100 + 0x15;
    pub const K_C_HOME: c_int = 0x100 + 0x16;
    pub const K_C_END: c_int = 0x100 + 0x17;
}

/// Check if a key is a delete/backspace key.
#[must_use]
pub const fn is_delete_key(key: c_int) -> bool {
    key == keys::K_BS
        || key == keys::CTRL_H
        || key == keys::K_DEL
        || key == keys::K_KDEL
        || key == keys::CTRL_W
}

/// Check if a key is a history navigation key.
#[must_use]
pub const fn is_history_key(key: c_int) -> bool {
    key == keys::K_UP || key == keys::K_DOWN || key == keys::CTRL_P || key == keys::CTRL_N
}

/// Check if a key is a cursor movement key.
#[must_use]
pub const fn is_cursor_key(key: c_int) -> bool {
    key == keys::K_LEFT
        || key == keys::K_RIGHT
        || key == keys::K_HOME
        || key == keys::K_END
        || key == keys::K_KHOME
        || key == keys::K_KEND
        || key == keys::CTRL_B
        || key == keys::CTRL_E
}

/// Check if a key is an incsearch navigation key.
#[must_use]
pub const fn is_incsearch_nav_key(key: c_int) -> bool {
    key == keys::CTRL_G || key == keys::CTRL_T
}

// =============================================================================
// FFI Exports
// =============================================================================

/// Get key result value for not changed.
#[no_mangle]
pub const extern "C" fn rs_getln_result_not_changed() -> c_int {
    result::NOT_CHANGED
}

/// Get key result value for changed.
#[no_mangle]
pub const extern "C" fn rs_getln_result_changed() -> c_int {
    result::CHANGED
}

/// Get key result value for goto normal mode.
#[no_mangle]
pub const extern "C" fn rs_getln_result_goto_normal() -> c_int {
    result::GOTO_NORMAL_MODE
}

/// Get key result value for process next key.
#[no_mangle]
pub const extern "C" fn rs_getln_result_process_next() -> c_int {
    result::PROCESS_NEXT_KEY
}

/// Check if key is a delete/backspace key.
#[no_mangle]
pub const extern "C" fn rs_getln_is_delete_key(key: c_int) -> c_int {
    is_delete_key(key) as c_int
}

/// Check if key is a history navigation key.
#[no_mangle]
pub const extern "C" fn rs_getln_is_history_key(key: c_int) -> c_int {
    is_history_key(key) as c_int
}

/// Check if key is a cursor movement key.
#[no_mangle]
pub const extern "C" fn rs_getln_is_cursor_key(key: c_int) -> c_int {
    is_cursor_key(key) as c_int
}

/// Check if key is an incsearch navigation key.
#[no_mangle]
pub const extern "C" fn rs_getln_is_incsearch_nav_key(key: c_int) -> c_int {
    is_incsearch_nav_key(key) as c_int
}

/// Check if history direction is newer.
#[no_mangle]
pub extern "C" fn rs_getln_history_direction_is_newer(key: c_int) -> c_int {
    c_int::from(HistoryDirection::from_key(key).is_some_and(HistoryDirection::is_newer))
}

/// Get effective firstc from raw value.
#[no_mangle]
pub const extern "C" fn rs_getln_effective_firstc(firstc: c_int) -> c_int {
    if firstc == -1 {
        0
    } else {
        firstc
    }
}

/// Get cmdline type character for events.
#[no_mangle]
pub const extern "C" fn rs_getln_cmdline_type(firstc: c_int) -> c_int {
    let effective = if firstc == -1 { 0 } else { firstc };
    if effective > 0 {
        effective
    } else {
        b'-' as c_int
    }
}

/// Check if firstc indicates a search command.
#[no_mangle]
pub const extern "C" fn rs_getln_is_search_firstc(firstc: c_int) -> c_int {
    let effective = if firstc == -1 { 0 } else { firstc };
    (effective == b'/' as c_int || effective == b'?' as c_int) as c_int
}

/// Check if firstc indicates input function.
#[no_mangle]
pub const extern "C" fn rs_getln_is_input_fn_firstc(firstc: c_int) -> c_int {
    let effective = if firstc == -1 { 0 } else { firstc };
    (effective == b'@' as c_int) as c_int
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_key_result() {
        assert_eq!(KeyResult::NotChanged.to_raw(), 1);
        assert_eq!(KeyResult::Changed.to_raw(), 2);
        assert_eq!(KeyResult::GotoNormalMode.to_raw(), 3);
        assert_eq!(KeyResult::ProcessNextKey.to_raw(), 4);

        assert_eq!(KeyResult::from_raw(1), Some(KeyResult::NotChanged));
        assert_eq!(KeyResult::from_raw(2), Some(KeyResult::Changed));
        assert_eq!(KeyResult::from_raw(5), None);

        assert!(KeyResult::Changed.is_changed());
        assert!(!KeyResult::NotChanged.is_changed());
        assert!(KeyResult::GotoNormalMode.is_goto_normal());
    }

    #[test]
    fn test_magic_override() {
        assert_eq!(MagicOverride::None.to_raw(), 0);
        assert_eq!(MagicOverride::On.to_raw(), 1);
        assert_eq!(MagicOverride::Off.to_raw(), 2);

        assert_eq!(MagicOverride::from_raw(0), MagicOverride::None);
        assert_eq!(MagicOverride::from_raw(1), MagicOverride::On);
        assert_eq!(MagicOverride::from_raw(2), MagicOverride::Off);
        assert_eq!(MagicOverride::from_raw(99), MagicOverride::None);
    }

    #[test]
    fn test_incsearch_state() {
        let mut state = IncsearchState::new();
        assert!(!state.has_match());
        assert_eq!(state.winid, 0);

        state.match_start = Position::new(1, 0);
        assert!(state.has_match());

        state.clear_match();
        assert!(!state.has_match());

        state.winid = 42;
        assert!(state.is_valid_for_window(42));
        assert!(!state.is_valid_for_window(1));
    }

    #[test]
    fn test_cmdline_flags() {
        let flags = CmdlineFlags::new();
        assert!(!flags.gotesc);
        assert!(flags.do_abbr);
        assert!(flags.ignore_drag_release);

        let mut flags2 = CmdlineFlags::new();
        flags2.gotesc = true;
        flags2.reset();
        assert!(!flags2.gotesc);
    }

    #[test]
    fn test_cmdline_state_info() {
        let mut state = CommandLineStateInfo::new();
        state.init(b':' as c_int, 1, 0);

        assert_eq!(state.firstc, b':' as c_int);
        assert_eq!(state.count, 1);
        assert!(!state.is_search());
        assert!(!state.is_input_fn());
        assert_eq!(state.cmdline_type, b':' as c_int);

        state.init(b'/' as c_int, 1, 0);
        assert!(state.is_search());

        state.init(b'@' as c_int, 1, 0);
        assert!(state.is_input_fn());

        state.init(-1, 1, 0);
        assert_eq!(state.effective_firstc(), 0);
        assert_eq!(state.cmdline_type, b'-' as c_int);
    }

    #[test]
    fn test_key_classification() {
        assert!(is_delete_key(keys::K_BS));
        assert!(is_delete_key(keys::CTRL_H));
        assert!(is_delete_key(keys::CTRL_W));
        assert!(!is_delete_key(keys::CTRL_A));

        assert!(is_history_key(keys::K_UP));
        assert!(is_history_key(keys::K_DOWN));
        assert!(is_history_key(keys::CTRL_P));
        assert!(is_history_key(keys::CTRL_N));
        assert!(!is_history_key(keys::CTRL_A));

        assert!(is_cursor_key(keys::K_LEFT));
        assert!(is_cursor_key(keys::K_RIGHT));
        assert!(is_cursor_key(keys::CTRL_B));
        assert!(is_cursor_key(keys::CTRL_E));
        assert!(!is_cursor_key(keys::CTRL_A));

        assert!(is_incsearch_nav_key(keys::CTRL_G));
        assert!(is_incsearch_nav_key(keys::CTRL_T));
        assert!(!is_incsearch_nav_key(keys::CTRL_A));
    }

    #[test]
    fn test_ffi_helpers() {
        assert_eq!(rs_getln_effective_firstc(-1), 0);
        assert_eq!(rs_getln_effective_firstc(b':' as c_int), b':' as c_int);

        assert_eq!(rs_getln_cmdline_type(-1), b'-' as c_int);
        assert_eq!(rs_getln_cmdline_type(0), b'-' as c_int);
        assert_eq!(rs_getln_cmdline_type(b':' as c_int), b':' as c_int);

        assert_eq!(rs_getln_is_search_firstc(b'/' as c_int), 1);
        assert_eq!(rs_getln_is_search_firstc(b'?' as c_int), 1);
        assert_eq!(rs_getln_is_search_firstc(b':' as c_int), 0);

        assert_eq!(rs_getln_is_input_fn_firstc(b'@' as c_int), 1);
        assert_eq!(rs_getln_is_input_fn_firstc(b':' as c_int), 0);
    }
}
