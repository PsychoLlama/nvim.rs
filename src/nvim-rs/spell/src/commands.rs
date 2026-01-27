//! Spell command utilities for Neovim
//!
//! This module provides support functions for spell-related commands:
//! - Navigation commands (`]s`, `[s`, `]S`, `[S`)
//! - Word modification commands (`zg`, `zw`, `zug`, `zuw`, `zG`, `zW`)
//! - Ex commands (`:spellinfo`, `:spelldump`, `:spellrepall`)
//!
//! The actual command execution remains in C due to deep integration with
//! Neovim's buffer, window, and undo systems. This module provides the
//! supporting logic and data structures.

#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_possible_wrap)]

use std::ffi::c_int;

// =============================================================================
// Navigation Direction and Types
// =============================================================================

/// Direction of spell navigation movement.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SpellMoveDirection {
    /// Move forward in the buffer
    #[default]
    Forward = 0,
    /// Move backward in the buffer
    Backward = 1,
}

impl SpellMoveDirection {
    /// Convert from C integer (FORWARD = 1, BACKWARD = -1 in Neovim).
    #[must_use]
    pub const fn from_vim_direction(dir: c_int) -> Self {
        if dir < 0 {
            Self::Backward
        } else {
            Self::Forward
        }
    }

    /// Convert to Vim's direction constant.
    #[must_use]
    pub const fn to_vim_direction(self) -> c_int {
        match self {
            Self::Forward => 1,
            Self::Backward => -1,
        }
    }

    /// Check if this is forward movement.
    #[must_use]
    pub const fn is_forward(self) -> bool {
        matches!(self, Self::Forward)
    }

    /// Check if this is backward movement.
    #[must_use]
    pub const fn is_backward(self) -> bool {
        matches!(self, Self::Backward)
    }
}

/// FFI wrapper to create direction from Vim's direction constant.
#[no_mangle]
pub extern "C" fn rs_spell_direction_from_vim(dir: c_int) -> SpellMoveDirection {
    SpellMoveDirection::from_vim_direction(dir)
}

/// FFI wrapper to convert direction to Vim's constant.
#[no_mangle]
pub extern "C" fn rs_spell_direction_to_vim(dir: SpellMoveDirection) -> c_int {
    dir.to_vim_direction()
}

/// FFI wrapper to check if direction is forward.
#[no_mangle]
pub extern "C" fn rs_spell_direction_is_forward(dir: SpellMoveDirection) -> bool {
    dir.is_forward()
}

// =============================================================================
// Spell Navigation Behavior
// =============================================================================

/// Behavior for spell navigation (what types of errors to find).
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SpellMoveBehavior {
    /// Find all types of spelling errors
    #[default]
    All = 0,
    /// Find only bad (wrong) words
    Bad = 1,
    /// Find only rare words
    Rare = 2,
}

impl SpellMoveBehavior {
    /// Convert from C integer.
    #[must_use]
    pub const fn from_c_int(value: c_int) -> Option<Self> {
        match value {
            0 => Some(Self::All),
            1 => Some(Self::Bad),
            2 => Some(Self::Rare),
            _ => None,
        }
    }

    /// Check if a spell result matches this behavior.
    ///
    /// # Arguments
    /// * `hlf` - The highlight group (HLF_SPB = bad, HLF_SPR = rare, HLF_SPC = cap, HLF_SPL = local)
    #[must_use]
    pub const fn matches_highlight(self, hlf: c_int) -> bool {
        // HLF values from highlight.h
        const HLF_SPB: c_int = 0x2B; // 43 - SpellBad
        const HLF_SPR: c_int = 0x2E; // 46 - SpellRare
        const HLF_SPC: c_int = 0x2C; // 44 - SpellCap
        const HLF_SPL: c_int = 0x2D; // 45 - SpellLocal

        match self {
            Self::All => hlf == HLF_SPB || hlf == HLF_SPR || hlf == HLF_SPC || hlf == HLF_SPL,
            Self::Bad => hlf == HLF_SPB,
            Self::Rare => hlf == HLF_SPR,
        }
    }
}

/// FFI wrapper to convert integer to SpellMoveBehavior.
#[no_mangle]
pub extern "C" fn rs_spell_move_behavior_from_int(value: c_int) -> c_int {
    SpellMoveBehavior::from_c_int(value).map_or(0, |b| b as c_int)
}

/// FFI wrapper to check if a highlight matches the behavior.
#[no_mangle]
pub extern "C" fn rs_spell_move_behavior_matches(behavior: c_int, hlf: c_int) -> bool {
    SpellMoveBehavior::from_c_int(behavior).is_some_and(|b| b.matches_highlight(hlf))
}

// =============================================================================
// Word Add/Remove Commands
// =============================================================================

/// Type of spell word add command.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SpellWordCommand {
    /// zg - Add word as good
    #[default]
    AddGood = 0,
    /// zw - Add word as wrong/bad
    AddWrong = 1,
    /// zug - Undo zg (remove good word)
    UndoGood = 2,
    /// zuw - Undo zw (remove wrong word)
    UndoWrong = 3,
    /// zG - Add to internal wordlist as good
    AddGoodInternal = 4,
    /// zW - Add to internal wordlist as wrong
    AddWrongInternal = 5,
    /// zuG - Undo zG
    UndoGoodInternal = 6,
    /// zuW - Undo zW
    UndoWrongInternal = 7,
}

impl SpellWordCommand {
    /// Check if this is an add operation (vs undo/remove).
    #[must_use]
    pub const fn is_add(self) -> bool {
        matches!(
            self,
            Self::AddGood | Self::AddWrong | Self::AddGoodInternal | Self::AddWrongInternal
        )
    }

    /// Check if this is an undo/remove operation.
    #[must_use]
    pub const fn is_undo(self) -> bool {
        matches!(
            self,
            Self::UndoGood | Self::UndoWrong | Self::UndoGoodInternal | Self::UndoWrongInternal
        )
    }

    /// Check if this targets the internal wordlist (zG/zW/zuG/zuW).
    #[must_use]
    pub const fn is_internal(self) -> bool {
        matches!(
            self,
            Self::AddGoodInternal
                | Self::AddWrongInternal
                | Self::UndoGoodInternal
                | Self::UndoWrongInternal
        )
    }

    /// Check if this is a "good word" operation (zg/zG/zug/zuG).
    #[must_use]
    pub const fn is_good(self) -> bool {
        matches!(
            self,
            Self::AddGood | Self::AddGoodInternal | Self::UndoGood | Self::UndoGoodInternal
        )
    }

    /// Check if this is a "wrong word" operation (zw/zW/zuw/zuW).
    #[must_use]
    pub const fn is_wrong(self) -> bool {
        matches!(
            self,
            Self::AddWrong | Self::AddWrongInternal | Self::UndoWrong | Self::UndoWrongInternal
        )
    }

    /// Get the spell add type for this command.
    #[must_use]
    pub const fn add_type(self) -> c_int {
        if self.is_good() {
            0 // SpellAddType::Good
        } else {
            1 // SpellAddType::Bad
        }
    }

    /// Get the spellfile index for this command.
    ///
    /// Returns 0 for internal wordlist commands, -1 to indicate "use default".
    #[must_use]
    pub const fn spellfile_idx(self) -> c_int {
        if self.is_internal() {
            0
        } else {
            -1 // Use default from 'spellfile' option
        }
    }
}

/// FFI wrapper to check if command is an add operation.
#[no_mangle]
pub extern "C" fn rs_spell_word_cmd_is_add(cmd: SpellWordCommand) -> bool {
    cmd.is_add()
}

/// FFI wrapper to check if command is an undo operation.
#[no_mangle]
pub extern "C" fn rs_spell_word_cmd_is_undo(cmd: SpellWordCommand) -> bool {
    cmd.is_undo()
}

/// FFI wrapper to check if command targets internal wordlist.
#[no_mangle]
pub extern "C" fn rs_spell_word_cmd_is_internal(cmd: SpellWordCommand) -> bool {
    cmd.is_internal()
}

/// FFI wrapper to get add type for command.
#[no_mangle]
pub extern "C" fn rs_spell_word_cmd_add_type(cmd: SpellWordCommand) -> c_int {
    cmd.add_type()
}

/// FFI wrapper to get spellfile index for command.
#[no_mangle]
pub extern "C" fn rs_spell_word_cmd_spellfile_idx(cmd: SpellWordCommand) -> c_int {
    cmd.spellfile_idx()
}

// =============================================================================
// Spell Dump Options
// =============================================================================

/// Options for :spelldump command.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct SpellDumpOptions {
    /// Include only words from this region (empty = all regions)
    pub region: [u8; 3],
    /// Include word counts
    pub include_counts: bool,
    /// Include rare words
    pub include_rare: bool,
    /// Include banned words (preceded by /)
    pub include_banned: bool,
}

impl SpellDumpOptions {
    /// Create default options (dump all words).
    #[must_use]
    pub const fn new() -> Self {
        Self {
            region: [0, 0, 0],
            include_counts: false,
            include_rare: true,
            include_banned: true,
        }
    }

    /// Set the region filter.
    #[must_use]
    pub fn with_region(mut self, region: &[u8]) -> Self {
        if region.len() >= 2 {
            self.region[0] = region[0];
            self.region[1] = region[1];
            self.region[2] = 0;
        }
        self
    }

    /// Enable word counts.
    #[must_use]
    pub const fn with_counts(mut self) -> Self {
        self.include_counts = true;
        self
    }

    /// Check if a region filter is set.
    #[must_use]
    pub const fn has_region_filter(&self) -> bool {
        self.region[0] != 0
    }
}

/// FFI wrapper to create default SpellDumpOptions.
#[no_mangle]
pub extern "C" fn rs_spell_dump_options_new() -> SpellDumpOptions {
    SpellDumpOptions::new()
}

/// FFI wrapper to check if options have a region filter.
///
/// # Safety
/// `opts` must be a valid pointer to a SpellDumpOptions struct.
#[no_mangle]
pub unsafe extern "C" fn rs_spell_dump_has_region_filter(opts: *const SpellDumpOptions) -> bool {
    if opts.is_null() {
        return false;
    }
    (*opts).has_region_filter()
}

// =============================================================================
// Spell Info Formatting
// =============================================================================

/// Maximum length of a spell language info line.
pub const SPELL_INFO_MAX_LINE: usize = 256;

/// Format flags for spell info output.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct SpellInfoFlags {
    /// Show file path
    pub show_path: bool,
    /// Show word counts
    pub show_counts: bool,
    /// Show region information
    pub show_regions: bool,
    /// Show compound rules
    pub show_compound: bool,
}

impl SpellInfoFlags {
    /// Create with all flags enabled.
    #[must_use]
    pub const fn all() -> Self {
        Self {
            show_path: true,
            show_counts: true,
            show_regions: true,
            show_compound: true,
        }
    }

    /// Create with minimal info (path only).
    #[must_use]
    pub const fn minimal() -> Self {
        Self {
            show_path: true,
            show_counts: false,
            show_regions: false,
            show_compound: false,
        }
    }
}

/// FFI wrapper to create all SpellInfoFlags.
#[no_mangle]
pub extern "C" fn rs_spell_info_flags_all() -> SpellInfoFlags {
    SpellInfoFlags::all()
}

/// FFI wrapper to create minimal SpellInfoFlags.
#[no_mangle]
pub extern "C" fn rs_spell_info_flags_minimal() -> SpellInfoFlags {
    SpellInfoFlags::minimal()
}

// =============================================================================
// Spellrepall Support
// =============================================================================

/// State for :spellrepall command.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct SpellRepallState {
    /// Number of replacements made
    pub count: c_int,
    /// Line number of last replacement
    pub last_lnum: c_int,
    /// Whether any errors occurred
    pub had_error: bool,
}

impl SpellRepallState {
    /// Create new state.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            count: 0,
            last_lnum: 0,
            had_error: false,
        }
    }

    /// Record a successful replacement.
    pub fn record_replacement(&mut self, lnum: c_int) {
        self.count += 1;
        self.last_lnum = lnum;
    }

    /// Record an error.
    pub fn record_error(&mut self) {
        self.had_error = true;
    }
}

/// FFI wrapper to create SpellRepallState.
#[no_mangle]
pub extern "C" fn rs_spell_repall_state_new() -> SpellRepallState {
    SpellRepallState::new()
}

/// FFI wrapper to record a replacement.
///
/// # Safety
/// `state` must be a valid pointer to a SpellRepallState struct.
#[no_mangle]
pub unsafe extern "C" fn rs_spell_repall_record(state: *mut SpellRepallState, lnum: c_int) {
    if !state.is_null() {
        (*state).record_replacement(lnum);
    }
}

/// FFI wrapper to record an error.
///
/// # Safety
/// `state` must be a valid pointer to a SpellRepallState struct.
#[no_mangle]
pub unsafe extern "C" fn rs_spell_repall_error(state: *mut SpellRepallState) {
    if !state.is_null() {
        (*state).record_error();
    }
}

/// FFI wrapper to get replacement count.
///
/// # Safety
/// `state` must be a valid pointer to a SpellRepallState struct.
#[no_mangle]
pub unsafe extern "C" fn rs_spell_repall_count(state: *const SpellRepallState) -> c_int {
    if state.is_null() {
        return 0;
    }
    (*state).count
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_spell_direction() {
        assert!(SpellMoveDirection::Forward.is_forward());
        assert!(SpellMoveDirection::Backward.is_backward());
        assert_eq!(
            SpellMoveDirection::from_vim_direction(1),
            SpellMoveDirection::Forward
        );
        assert_eq!(
            SpellMoveDirection::from_vim_direction(-1),
            SpellMoveDirection::Backward
        );
        assert_eq!(SpellMoveDirection::Forward.to_vim_direction(), 1);
        assert_eq!(SpellMoveDirection::Backward.to_vim_direction(), -1);
    }

    #[test]
    fn test_spell_move_behavior() {
        assert_eq!(
            SpellMoveBehavior::from_c_int(0),
            Some(SpellMoveBehavior::All)
        );
        assert_eq!(
            SpellMoveBehavior::from_c_int(1),
            Some(SpellMoveBehavior::Bad)
        );
        assert_eq!(
            SpellMoveBehavior::from_c_int(2),
            Some(SpellMoveBehavior::Rare)
        );
        assert_eq!(SpellMoveBehavior::from_c_int(99), None);
    }

    #[test]
    fn test_spell_word_command() {
        assert!(SpellWordCommand::AddGood.is_add());
        assert!(SpellWordCommand::AddGood.is_good());
        assert!(!SpellWordCommand::AddGood.is_undo());
        assert!(!SpellWordCommand::AddGood.is_internal());

        assert!(SpellWordCommand::UndoGood.is_undo());
        assert!(SpellWordCommand::UndoGood.is_good());
        assert!(!SpellWordCommand::UndoGood.is_add());

        assert!(SpellWordCommand::AddGoodInternal.is_internal());
        assert!(SpellWordCommand::AddWrongInternal.is_internal());

        assert!(SpellWordCommand::AddWrong.is_wrong());
        assert!(SpellWordCommand::UndoWrong.is_wrong());
    }

    #[test]
    fn test_spell_dump_options() {
        let opts = SpellDumpOptions::new();
        assert!(!opts.has_region_filter());
        assert!(opts.include_rare);
        assert!(opts.include_banned);

        let opts = opts.with_region(b"us").with_counts();
        assert!(opts.has_region_filter());
        assert!(opts.include_counts);
        assert_eq!(opts.region[0], b'u');
        assert_eq!(opts.region[1], b's');
    }

    #[test]
    fn test_spell_info_flags() {
        let all = SpellInfoFlags::all();
        assert!(all.show_path);
        assert!(all.show_counts);
        assert!(all.show_regions);
        assert!(all.show_compound);

        let minimal = SpellInfoFlags::minimal();
        assert!(minimal.show_path);
        assert!(!minimal.show_counts);
    }

    #[test]
    fn test_spell_repall_state() {
        let mut state = SpellRepallState::new();
        assert_eq!(state.count, 0);
        assert!(!state.had_error);

        state.record_replacement(10);
        assert_eq!(state.count, 1);
        assert_eq!(state.last_lnum, 10);

        state.record_replacement(20);
        assert_eq!(state.count, 2);
        assert_eq!(state.last_lnum, 20);

        state.record_error();
        assert!(state.had_error);
    }
}
