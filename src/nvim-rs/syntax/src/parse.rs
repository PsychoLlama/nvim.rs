//! Syntax command parsing utilities.
//!
//! This module provides parsing for :syntax commands including
//! :syn match, :syn region, :syn keyword, and their arguments.

use std::ffi::c_int;

// =============================================================================
// Syntax command types
// =============================================================================

/// Type of syntax command being parsed.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SynCmdType {
    /// :syntax match
    Match = 0,
    /// :syntax region
    Region = 1,
    /// :syntax keyword
    Keyword = 2,
    /// :syntax cluster
    Cluster = 3,
    /// :syntax sync
    Sync = 4,
    /// :syntax clear
    Clear = 5,
    /// :syntax reset
    Reset = 6,
    /// :syntax on/off/enable/manual
    Toggle = 7,
    /// :syntax include
    Include = 8,
    /// :syntax spell
    Spell = 9,
    /// :syntax conceal
    Conceal = 10,
    /// :syntax foldlevel
    FoldLevel = 11,
    /// :syntax iskeyword
    IsKeyword = 12,
    /// Unknown command
    Unknown = 99,
}

impl SynCmdType {
    /// Convert from C integer.
    #[must_use]
    pub const fn from_c_int(val: c_int) -> Self {
        match val {
            0 => Self::Match,
            1 => Self::Region,
            2 => Self::Keyword,
            3 => Self::Cluster,
            4 => Self::Sync,
            5 => Self::Clear,
            6 => Self::Reset,
            7 => Self::Toggle,
            8 => Self::Include,
            9 => Self::Spell,
            10 => Self::Conceal,
            11 => Self::FoldLevel,
            12 => Self::IsKeyword,
            _ => Self::Unknown,
        }
    }

    /// Convert to C integer.
    #[must_use]
    pub const fn to_c_int(self) -> c_int {
        self as c_int
    }

    /// Check if this is a definition command (creates syntax items).
    #[must_use]
    pub const fn is_definition(self) -> bool {
        matches!(self, Self::Match | Self::Region | Self::Keyword | Self::Cluster)
    }

    /// Check if this command takes a group name.
    #[must_use]
    pub const fn takes_group_name(self) -> bool {
        matches!(self, Self::Match | Self::Region | Self::Keyword | Self::Cluster | Self::Clear)
    }

    /// Check if this command takes a pattern.
    #[must_use]
    pub const fn takes_pattern(self) -> bool {
        matches!(self, Self::Match | Self::Region | Self::Sync)
    }
}

// =============================================================================
// Parse state
// =============================================================================

/// State during syntax command parsing.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct SynParseState {
    /// Command type being parsed.
    pub cmd_type: c_int,
    /// Whether parsing succeeded so far.
    pub ok: bool,
    /// Whether we've seen the group name.
    pub has_group: bool,
    /// Whether we've seen the pattern.
    pub has_pattern: bool,
    /// Whether we've seen `contained`.
    pub contained: bool,
    /// Whether we've seen `display`.
    pub display: bool,
    /// Whether we've seen `oneline`.
    pub oneline: bool,
    /// Whether we've seen `fold`.
    pub fold: bool,
    /// Whether we've seen `conceal`.
    pub conceal: bool,
    /// Whether we've seen `transparent`.
    pub transparent: bool,
    /// Whether we've seen `skipwhite`.
    pub skipwhite: bool,
    /// Whether we've seen `skipnl`.
    pub skipnl: bool,
    /// Whether we've seen `skipempty`.
    pub skipempty: bool,
    /// Whether we've seen `extend`.
    pub extend: bool,
    /// Whether we've seen `excludenl`.
    pub excludenl: bool,
    /// Whether we've seen `keepend`.
    pub keepend: bool,
}

impl SynParseState {
    /// Create a new parse state.
    #[must_use]
    pub const fn new(cmd_type: SynCmdType) -> Self {
        Self {
            cmd_type: cmd_type.to_c_int(),
            ok: true,
            has_group: false,
            has_pattern: false,
            contained: false,
            display: false,
            oneline: false,
            fold: false,
            conceal: false,
            transparent: false,
            skipwhite: false,
            skipnl: false,
            skipempty: false,
            extend: false,
            excludenl: false,
            keepend: false,
        }
    }

    /// Mark parsing as failed.
    pub fn fail(&mut self) {
        self.ok = false;
    }

    /// Get command type.
    #[must_use]
    pub const fn get_cmd_type(&self) -> SynCmdType {
        SynCmdType::from_c_int(self.cmd_type)
    }

    /// Check if all required components have been parsed.
    #[must_use]
    pub fn is_complete(&self) -> bool {
        if !self.ok {
            return false;
        }
        let cmd = self.get_cmd_type();
        if cmd.takes_group_name() && !self.has_group {
            return false;
        }
        if cmd.takes_pattern() && !self.has_pattern {
            return false;
        }
        true
    }
}

// =============================================================================
// Argument keywords
// =============================================================================

/// Known syntax argument keywords.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SynArgKeyword {
    /// Unknown/invalid keyword.
    Unknown = 0,
    /// contained
    Contained = 1,
    /// containedin=
    ContainedIn = 2,
    /// nextgroup=
    NextGroup = 3,
    /// skipwhite
    SkipWhite = 4,
    /// skipnl
    SkipNl = 5,
    /// skipempty
    SkipEmpty = 6,
    /// contains=
    Contains = 7,
    /// oneline
    OneLine = 8,
    /// fold
    Fold = 9,
    /// display
    Display = 10,
    /// extend
    Extend = 11,
    /// conceal
    Conceal = 12,
    /// cchar=
    Cchar = 13,
    /// transparent
    Transparent = 14,
    /// excludenl
    ExcludeNl = 15,
    /// keepend
    KeepEnd = 16,
    /// start=
    Start = 17,
    /// skip=
    Skip = 18,
    /// end=
    End = 19,
    /// matchgroup=
    MatchGroup = 20,
    /// add=
    Add = 21,
    /// remove=
    Remove = 22,
}

impl SynArgKeyword {
    /// Convert to C integer.
    #[must_use]
    pub const fn to_c_int(self) -> c_int {
        self as c_int
    }

    /// Check if keyword takes a value (has =).
    #[must_use]
    pub const fn takes_value(self) -> bool {
        matches!(
            self,
            Self::ContainedIn
                | Self::NextGroup
                | Self::Contains
                | Self::Cchar
                | Self::Start
                | Self::Skip
                | Self::End
                | Self::MatchGroup
                | Self::Add
                | Self::Remove
        )
    }

    /// Check if keyword is a boolean flag (no value).
    #[must_use]
    pub const fn is_boolean(self) -> bool {
        !self.takes_value()
    }
}

// =============================================================================
// FFI exports
// =============================================================================

/// Create a new parse state.
#[no_mangle]
pub extern "C" fn rs_syn_parse_state_new(cmd_type: c_int) -> SynParseState {
    SynParseState::new(SynCmdType::from_c_int(cmd_type))
}

/// Check if parse state is OK.
///
/// # Safety
/// `state` must be valid.
#[no_mangle]
pub unsafe extern "C" fn rs_syn_parse_state_is_ok(state: *const SynParseState) -> c_int {
    if state.is_null() {
        return 0;
    }
    c_int::from((*state).ok)
}

/// Mark parse state as failed.
///
/// # Safety
/// `state` must be valid.
#[no_mangle]
pub unsafe extern "C" fn rs_syn_parse_state_fail(state: *mut SynParseState) {
    if !state.is_null() {
        (*state).fail();
    }
}

/// Check if parse state is complete.
///
/// # Safety
/// `state` must be valid.
#[no_mangle]
pub unsafe extern "C" fn rs_syn_parse_state_is_complete(state: *const SynParseState) -> c_int {
    if state.is_null() {
        return 0;
    }
    c_int::from((*state).is_complete())
}

/// Check if command type is a definition command.
#[no_mangle]
pub extern "C" fn rs_syn_cmd_type_is_definition(cmd_type: c_int) -> c_int {
    c_int::from(SynCmdType::from_c_int(cmd_type).is_definition())
}

/// Check if command type takes a group name.
#[no_mangle]
pub extern "C" fn rs_syn_cmd_type_takes_group(cmd_type: c_int) -> c_int {
    c_int::from(SynCmdType::from_c_int(cmd_type).takes_group_name())
}

/// Check if command type takes a pattern.
#[no_mangle]
pub extern "C" fn rs_syn_cmd_type_takes_pattern(cmd_type: c_int) -> c_int {
    c_int::from(SynCmdType::from_c_int(cmd_type).takes_pattern())
}

/// Check if argument keyword takes a value.
#[no_mangle]
pub extern "C" fn rs_syn_arg_takes_value(keyword: c_int) -> c_int {
    if keyword < 0 || keyword > SynArgKeyword::Remove as c_int {
        return 0;
    }
    let kw = unsafe { std::mem::transmute::<c_int, SynArgKeyword>(keyword) };
    c_int::from(kw.takes_value())
}

/// Check if argument keyword is boolean.
#[no_mangle]
pub extern "C" fn rs_syn_arg_is_boolean(keyword: c_int) -> c_int {
    if keyword < 0 || keyword > SynArgKeyword::Remove as c_int {
        return 0;
    }
    let kw = unsafe { std::mem::transmute::<c_int, SynArgKeyword>(keyword) };
    c_int::from(kw.is_boolean())
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_syn_cmd_type() {
        assert_eq!(SynCmdType::Match.to_c_int(), 0);
        assert_eq!(SynCmdType::Region.to_c_int(), 1);
        assert_eq!(SynCmdType::from_c_int(0), SynCmdType::Match);
        assert_eq!(SynCmdType::from_c_int(99), SynCmdType::Unknown);

        assert!(SynCmdType::Match.is_definition());
        assert!(SynCmdType::Region.is_definition());
        assert!(!SynCmdType::Clear.is_definition());

        assert!(SynCmdType::Match.takes_group_name());
        assert!(SynCmdType::Clear.takes_group_name());
        assert!(!SynCmdType::Reset.takes_group_name());

        assert!(SynCmdType::Match.takes_pattern());
        assert!(SynCmdType::Region.takes_pattern());
        assert!(!SynCmdType::Keyword.takes_pattern());
    }

    #[test]
    fn test_parse_state() {
        let mut state = SynParseState::new(SynCmdType::Match);
        assert!(state.ok);
        assert!(!state.is_complete());

        state.has_group = true;
        assert!(!state.is_complete());

        state.has_pattern = true;
        assert!(state.is_complete());

        state.fail();
        assert!(!state.is_complete());
    }

    #[test]
    fn test_arg_keyword() {
        assert!(SynArgKeyword::ContainedIn.takes_value());
        assert!(SynArgKeyword::NextGroup.takes_value());
        assert!(!SynArgKeyword::Contained.takes_value());
        assert!(!SynArgKeyword::Display.takes_value());

        assert!(SynArgKeyword::Contained.is_boolean());
        assert!(!SynArgKeyword::Contains.is_boolean());
    }
}
