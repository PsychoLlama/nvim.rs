//! Completion types for insert-mode completion.
//!
//! This module defines the core types for the insert-mode completion system,
//! including completion modes, state, and item representations.

#![allow(dead_code, unused_imports)]
#![allow(clippy::missing_const_for_fn)]

use std::ffi::c_int;

// =============================================================================
// CTRL-X Mode Constants (must match C enum values exactly)
// =============================================================================

/// Flag indicating the mode wants an identifier.
pub const CTRL_X_WANT_IDENT: c_int = 0x100;

/// CTRL-X completion mode enumeration.
///
/// These values must match the C enum in insexpand.c exactly.
#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Hash)]
pub enum CtrlXMode {
    /// CTRL-N CTRL-P completion, default
    #[default]
    Normal = 0,
    /// CTRL-X was typed but no submode selected yet
    NotDefinedYet = 1,
    /// Scroll mode (CTRL-E/CTRL-Y)
    Scroll = 2,
    /// Whole line completion (CTRL-L)
    WholeLine = 3,
    /// File name completion (CTRL-F)
    Files = 4,
    /// Tag completion (CTRL-])
    Tags = 5 + CTRL_X_WANT_IDENT,
    /// Path pattern completion (CTRL-P in path)
    PathPatterns = 6 + CTRL_X_WANT_IDENT,
    /// Definition completion (CTRL-D)
    PathDefines = 7 + CTRL_X_WANT_IDENT,
    /// Completion finished state
    Finished = 8,
    /// Dictionary completion (CTRL-K)
    Dictionary = 9 + CTRL_X_WANT_IDENT,
    /// Thesaurus completion (CTRL-T)
    Thesaurus = 10 + CTRL_X_WANT_IDENT,
    /// Command-line completion (CTRL-V)
    Cmdline = 11,
    /// User-defined function completion (CTRL-U)
    Function = 12,
    /// Omni completion (CTRL-O)
    Omni = 13,
    /// Spell completion (CTRL-S)
    Spell = 14,
    /// Eval/builtin complete() function
    Eval = 16,
    /// CTRL-X typed in CTRL_X_CMDLINE mode
    CmdlineCtrlX = 17,
    /// Buffer name completion
    Bufnames = 18,
    /// Register completion (CTRL-R)
    Register = 19,
}

impl CtrlXMode {
    /// Create from raw C int value.
    #[must_use]
    pub const fn from_raw(value: c_int) -> Option<Self> {
        match value {
            0 => Some(Self::Normal),
            1 => Some(Self::NotDefinedYet),
            2 => Some(Self::Scroll),
            3 => Some(Self::WholeLine),
            4 => Some(Self::Files),
            v if v == 5 + CTRL_X_WANT_IDENT => Some(Self::Tags),
            v if v == 6 + CTRL_X_WANT_IDENT => Some(Self::PathPatterns),
            v if v == 7 + CTRL_X_WANT_IDENT => Some(Self::PathDefines),
            8 => Some(Self::Finished),
            v if v == 9 + CTRL_X_WANT_IDENT => Some(Self::Dictionary),
            v if v == 10 + CTRL_X_WANT_IDENT => Some(Self::Thesaurus),
            11 => Some(Self::Cmdline),
            12 => Some(Self::Function),
            13 => Some(Self::Omni),
            14 => Some(Self::Spell),
            16 => Some(Self::Eval),
            17 => Some(Self::CmdlineCtrlX),
            18 => Some(Self::Bufnames),
            19 => Some(Self::Register),
            _ => None,
        }
    }

    /// Convert to raw C int value.
    #[must_use]
    pub const fn to_raw(self) -> c_int {
        self as c_int
    }

    /// Check if this mode wants an identifier.
    #[must_use]
    pub const fn wants_ident(self) -> bool {
        (self.to_raw() & CTRL_X_WANT_IDENT) != 0
    }

    /// Get the base mode value without the WANT_IDENT flag.
    #[must_use]
    pub const fn base_mode(self) -> c_int {
        self.to_raw() & !CTRL_X_WANT_IDENT
    }

    /// Check if this is the normal/default completion mode.
    #[must_use]
    pub const fn is_normal(self) -> bool {
        matches!(self, Self::Normal)
    }

    /// Check if this is a scroll mode.
    #[must_use]
    pub const fn is_scroll(self) -> bool {
        matches!(self, Self::Scroll)
    }

    /// Check if this is whole line or eval mode.
    #[must_use]
    pub const fn is_line_or_eval(self) -> bool {
        matches!(self, Self::WholeLine | Self::Eval)
    }

    /// Check if this is a cmdline mode.
    #[must_use]
    pub const fn is_cmdline(self) -> bool {
        matches!(self, Self::Cmdline | Self::CmdlineCtrlX)
    }

    /// Get the mode name for display.
    #[must_use]
    pub const fn name(&self) -> &'static str {
        match self {
            Self::Normal => "keyword",
            Self::NotDefinedYet => "ctrl_x",
            Self::Scroll => "scroll",
            Self::WholeLine => "whole_line",
            Self::Files => "files",
            Self::Tags => "tags",
            Self::PathPatterns => "path_patterns",
            Self::PathDefines => "path_defines",
            Self::Finished => "unknown",
            Self::Dictionary => "dictionary",
            Self::Thesaurus => "thesaurus",
            Self::Cmdline | Self::CmdlineCtrlX => "cmdline",
            Self::Function => "function",
            Self::Omni => "omni",
            Self::Spell => "spell",
            Self::Eval => "eval",
            Self::Bufnames => "bufnames",
            Self::Register => "register",
        }
    }
}

// =============================================================================
// Completion Status Flags
// =============================================================================

/// Flags for completion continuation status.
pub mod compl_status {
    use std::ffi::c_int;

    /// "normal" or "adding" expansion
    pub const CONT_ADDING: c_int = 1;
    /// A ^X interrupted the current expansion (set only if N_ADDS is set)
    pub const CONT_INTRPT: c_int = 2 + 4;
    /// Next ^X<> will add-new or expand-current
    pub const CONT_N_ADDS: c_int = 4;
    /// Next ^X<> will set initial_pos
    pub const CONT_S_IPOS: c_int = 8;
    /// Pattern includes start of line (for word-wise expansion)
    pub const CONT_SOL: c_int = 16;
    /// For ctrl_x_mode 0, ^X^P/^X^N do a local expansion
    pub const CONT_LOCAL: c_int = 32;
}

// =============================================================================
// Match Item Flags
// =============================================================================

/// Flags for completion match items (cp_flags).
#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum MatchFlags {
    /// No special flags
    #[default]
    None = 0,
    /// The original text when the expansion begun
    OriginalText = 1,
    /// cp_fname is allocated
    FreeFname = 2,
    /// Use CONT_S_IPOS for compl_cont_status
    ContSIpos = 4,
    /// ins_compl_equal() always returns true
    Equal = 8,
    /// ins_compl_equal ignores case
    Icase = 16,
    /// Use fast_breakcheck instead of os_breakcheck
    Fast = 32,
}

impl MatchFlags {
    /// Create from raw value.
    #[must_use]
    pub const fn from_raw(value: c_int) -> Self {
        match value {
            1 => Self::OriginalText,
            2 => Self::FreeFname,
            4 => Self::ContSIpos,
            8 => Self::Equal,
            16 => Self::Icase,
            32 => Self::Fast,
            _ => Self::None,
        }
    }

    /// Convert to raw value.
    #[must_use]
    pub const fn to_raw(self) -> c_int {
        self as c_int
    }
}

// =============================================================================
// Completion State
// =============================================================================

/// Current state of the insert-mode completion system.
///
/// This struct mirrors the static variables in insexpand.c that track
/// completion state.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct CompletionState {
    /// Which CTRL-X mode we're in
    pub ctrl_x_mode: c_int,
    /// Whether completion has started
    pub started: bool,
    /// Whether completion was interrupted
    pub interrupted: bool,
    /// Whether time slice expired (for autocomplete timeout)
    pub time_slice_expired: bool,
    /// Whether completion was interrupted (for restart check)
    pub was_interrupted: bool,
    /// Whether Enter selects the current match
    pub enter_selects: bool,
    /// Whether a match was used/selected
    pub used_match: bool,
    /// Completion continuation status flags
    pub cont_status: c_int,
    /// Number of completion matches
    pub matches: c_int,
    /// Length of text being completed
    pub length: c_int,
    /// Column where completion text starts
    pub col: c_int,
    /// Line number where completion started
    pub lnum: c_int,
    /// End column of inserted completion text
    pub ins_end_col: c_int,
    /// Currently selected item in popup (-1 if none)
    pub selected_item: c_int,
    /// Whether autocomplete mode is active
    pub autocomplete: bool,
    /// Whether completion is restarting
    pub restarting: bool,
    /// Direction of completion (FORWARD=1, BACKWARD=-1)
    pub direction: c_int,
    /// Direction currently being shown
    pub shows_dir: c_int,
    /// Whether refresh_always is set for function/omni completion
    pub opt_refresh_always: bool,
}

impl CompletionState {
    /// Create a new default completion state.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            ctrl_x_mode: 0,
            started: false,
            interrupted: false,
            time_slice_expired: false,
            was_interrupted: false,
            enter_selects: false,
            used_match: false,
            cont_status: 0,
            matches: 0,
            length: 0,
            col: 0,
            lnum: 0,
            ins_end_col: 0,
            selected_item: -1,
            autocomplete: false,
            restarting: false,
            direction: 1, // FORWARD
            shows_dir: 1, // FORWARD
            opt_refresh_always: false,
        }
    }

    /// Check if completion is active.
    #[must_use]
    pub const fn is_active(&self) -> bool {
        self.started
    }

    /// Check if in adding mode.
    #[must_use]
    pub const fn is_adding(&self) -> bool {
        (self.cont_status & compl_status::CONT_ADDING) != 0
    }

    /// Check if pattern includes start of line.
    #[must_use]
    pub const fn includes_sol(&self) -> bool {
        (self.cont_status & compl_status::CONT_SOL) != 0
    }

    /// Check if doing local completion.
    #[must_use]
    pub const fn is_local(&self) -> bool {
        (self.cont_status & compl_status::CONT_LOCAL) != 0
    }

    /// Get the current CTRL-X mode as an enum.
    #[must_use]
    pub const fn mode(&self) -> Option<CtrlXMode> {
        CtrlXMode::from_raw(self.ctrl_x_mode)
    }

    /// Check if we need to restart completion.
    #[must_use]
    pub const fn needs_restart(&self) -> bool {
        self.was_interrupted || self.opt_refresh_always
    }

    /// Check if going forward.
    #[must_use]
    pub const fn is_forward(&self) -> bool {
        self.direction == 1
    }

    /// Check if going backward.
    #[must_use]
    pub const fn is_backward(&self) -> bool {
        self.direction == -1
    }
}

// =============================================================================
// Completion Item
// =============================================================================

/// Text indices for completion item extra fields (cp_text array).
#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompletionTextField {
    /// Abbreviation to display in menu
    Abbr = 0,
    /// Menu text
    Menu = 1,
    /// Kind of completion (variable, function, etc.)
    Kind = 2,
    /// Additional info text
    Info = 3,
}

impl CompletionTextField {
    /// Number of text fields in a completion item.
    pub const COUNT: usize = 4;

    /// Convert from index.
    #[must_use]
    pub const fn from_index(idx: usize) -> Option<Self> {
        match idx {
            0 => Some(Self::Abbr),
            1 => Some(Self::Menu),
            2 => Some(Self::Kind),
            3 => Some(Self::Info),
            _ => None,
        }
    }

    /// Convert to index.
    #[must_use]
    pub const fn to_index(self) -> usize {
        self as usize
    }
}

/// A completion item that can be shown in the popup menu.
///
/// This corresponds to compl_T in the C code.
#[derive(Debug, Clone, Default)]
pub struct CompletionItem {
    /// The matched text (cp_str)
    pub text: String,
    /// Abbreviation (for menu display)
    pub abbr: Option<String>,
    /// Menu text
    pub menu: Option<String>,
    /// Kind of completion
    pub kind: Option<String>,
    /// Additional info
    pub info: Option<String>,
    /// File containing the match
    pub fname: Option<String>,
    /// Match flags
    pub flags: c_int,
    /// Sequence number
    pub number: c_int,
    /// Fuzzy match score or proximity score
    pub score: c_int,
    /// Whether this item is in the match array
    pub in_match_array: bool,
    /// Highlight attribute for abbr
    pub user_abbr_hlattr: c_int,
    /// Highlight attribute for kind
    pub user_kind_hlattr: c_int,
    /// Index of this match's source in 'cpt' option
    pub cpt_source_idx: c_int,
}

impl CompletionItem {
    /// Create a new completion item with the given text.
    #[must_use]
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            abbr: None,
            menu: None,
            kind: None,
            info: None,
            fname: None,
            flags: 0,
            number: -1,
            score: 0,
            in_match_array: false,
            user_abbr_hlattr: -1,
            user_kind_hlattr: -1,
            cpt_source_idx: -1,
        }
    }

    /// Check if this is the original text entry.
    #[must_use]
    pub const fn is_original_text(&self) -> bool {
        (self.flags & MatchFlags::OriginalText.to_raw()) != 0
    }

    /// Check if the file name should be freed.
    #[must_use]
    pub const fn has_free_fname(&self) -> bool {
        (self.flags & MatchFlags::FreeFname.to_raw()) != 0
    }

    /// Check if case should be ignored in comparison.
    #[must_use]
    pub const fn is_icase(&self) -> bool {
        (self.flags & MatchFlags::Icase.to_raw()) != 0
    }

    /// Check if equality is always true.
    #[must_use]
    pub const fn is_equal(&self) -> bool {
        (self.flags & MatchFlags::Equal.to_raw()) != 0
    }

    /// Get the display text (abbr if set, otherwise text).
    #[must_use]
    pub fn display_text(&self) -> &str {
        self.abbr.as_deref().unwrap_or(&self.text)
    }
}

// =============================================================================
// Completeopt Flags
// =============================================================================

/// Flags for the 'completeopt' option.
pub mod cot_flags {
    use std::ffi::c_uint;

    /// Show popup menu
    pub const MENU: c_uint = 0x01;
    /// Show popup menu even for single match
    pub const MENUONE: c_uint = 0x02;
    /// Insert longest common text
    pub const LONGEST: c_uint = 0x04;
    /// Show preview window
    pub const PREVIEW: c_uint = 0x08;
    /// Show popup for preview
    pub const POPUP: c_uint = 0x10;
    /// Don't insert any text
    pub const NOINSERT: c_uint = 0x20;
    /// Don't select any match
    pub const NOSELECT: c_uint = 0x40;
    /// Use fuzzy matching
    pub const FUZZY: c_uint = 0x80;
    /// Don't sort matches
    pub const NOSORT: c_uint = 0x100;
    /// Pre-insert match
    pub const PREINSERT: c_uint = 0x200;
    /// Sort by proximity (nearest)
    pub const NEAREST: c_uint = 0x400;
}

/// Parsed completeopt settings.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct CompleteoptFlags {
    /// Raw flags value
    pub flags: u32,
}

impl CompleteoptFlags {
    /// Create from raw flags.
    #[must_use]
    pub const fn from_raw(flags: u32) -> Self {
        Self { flags }
    }

    /// Check if menu is enabled.
    #[must_use]
    pub const fn has_menu(&self) -> bool {
        (self.flags & cot_flags::MENU) != 0
    }

    /// Check if menuone is enabled.
    #[must_use]
    pub const fn has_menuone(&self) -> bool {
        (self.flags & cot_flags::MENUONE) != 0
    }

    /// Check if longest is enabled.
    #[must_use]
    pub const fn has_longest(&self) -> bool {
        (self.flags & cot_flags::LONGEST) != 0
    }

    /// Check if preview is enabled.
    #[must_use]
    pub const fn has_preview(&self) -> bool {
        (self.flags & cot_flags::PREVIEW) != 0
    }

    /// Check if popup preview is enabled.
    #[must_use]
    pub const fn has_popup(&self) -> bool {
        (self.flags & cot_flags::POPUP) != 0
    }

    /// Check if noinsert is enabled.
    #[must_use]
    pub const fn has_noinsert(&self) -> bool {
        (self.flags & cot_flags::NOINSERT) != 0
    }

    /// Check if noselect is enabled.
    #[must_use]
    pub const fn has_noselect(&self) -> bool {
        (self.flags & cot_flags::NOSELECT) != 0
    }

    /// Check if fuzzy matching is enabled.
    #[must_use]
    pub const fn has_fuzzy(&self) -> bool {
        (self.flags & cot_flags::FUZZY) != 0
    }

    /// Check if nosort is enabled.
    #[must_use]
    pub const fn has_nosort(&self) -> bool {
        (self.flags & cot_flags::NOSORT) != 0
    }

    /// Check if preinsert is enabled.
    #[must_use]
    pub const fn has_preinsert(&self) -> bool {
        (self.flags & cot_flags::PREINSERT) != 0
    }

    /// Check if nearest sorting is enabled.
    #[must_use]
    pub const fn has_nearest(&self) -> bool {
        (self.flags & cot_flags::NEAREST) != 0
    }

    /// Check if popup menu should be shown.
    #[must_use]
    pub const fn should_show_pum(&self) -> bool {
        self.has_menu() || self.has_menuone()
    }
}

// =============================================================================
// FFI Exports
// =============================================================================

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ctrl_x_mode_values() {
        assert_eq!(CtrlXMode::Normal.to_raw(), 0);
        assert_eq!(CtrlXMode::NotDefinedYet.to_raw(), 1);
        assert_eq!(CtrlXMode::Scroll.to_raw(), 2);
        assert_eq!(CtrlXMode::WholeLine.to_raw(), 3);
        assert_eq!(CtrlXMode::Files.to_raw(), 4);
        assert_eq!(CtrlXMode::Tags.to_raw(), 5 + CTRL_X_WANT_IDENT);
        assert_eq!(CtrlXMode::Cmdline.to_raw(), 11);
        assert_eq!(CtrlXMode::Register.to_raw(), 19);
    }

    #[test]
    fn test_ctrl_x_mode_from_raw() {
        assert_eq!(CtrlXMode::from_raw(0), Some(CtrlXMode::Normal));
        assert_eq!(CtrlXMode::from_raw(11), Some(CtrlXMode::Cmdline));
        assert_eq!(
            CtrlXMode::from_raw(5 + CTRL_X_WANT_IDENT),
            Some(CtrlXMode::Tags)
        );
        assert_eq!(CtrlXMode::from_raw(999), None);
    }

    #[test]
    fn test_ctrl_x_mode_wants_ident() {
        assert!(CtrlXMode::Tags.wants_ident());
        assert!(CtrlXMode::Dictionary.wants_ident());
        assert!(CtrlXMode::PathPatterns.wants_ident());
        assert!(!CtrlXMode::Normal.wants_ident());
        assert!(!CtrlXMode::Files.wants_ident());
        assert!(!CtrlXMode::Cmdline.wants_ident());
    }

    #[test]
    fn test_ctrl_x_mode_predicates() {
        assert!(CtrlXMode::Normal.is_normal());
        assert!(!CtrlXMode::Scroll.is_normal());

        assert!(CtrlXMode::Scroll.is_scroll());
        assert!(!CtrlXMode::Normal.is_scroll());

        assert!(CtrlXMode::WholeLine.is_line_or_eval());
        assert!(CtrlXMode::Eval.is_line_or_eval());
        assert!(!CtrlXMode::Normal.is_line_or_eval());

        assert!(CtrlXMode::Cmdline.is_cmdline());
        assert!(CtrlXMode::CmdlineCtrlX.is_cmdline());
        assert!(!CtrlXMode::Normal.is_cmdline());
    }

    #[test]
    fn test_completion_state() {
        let state = CompletionState::new();
        assert!(!state.is_active());
        assert!(!state.is_adding());
        assert!(state.is_forward());
        assert!(!state.is_backward());
        assert_eq!(state.mode(), Some(CtrlXMode::Normal));
    }

    #[test]
    fn test_completion_state_cont_status() {
        let mut state = CompletionState::new();
        state.cont_status = compl_status::CONT_ADDING;
        assert!(state.is_adding());

        state.cont_status = compl_status::CONT_SOL;
        assert!(state.includes_sol());

        state.cont_status = compl_status::CONT_LOCAL;
        assert!(state.is_local());
    }

    #[test]
    fn test_completion_item() {
        let item = CompletionItem::new("test");
        assert_eq!(item.text, "test");
        assert!(!item.is_original_text());
        assert_eq!(item.display_text(), "test");

        let mut item_with_abbr = CompletionItem::new("long_function_name");
        item_with_abbr.abbr = Some("long_fn".to_string());
        assert_eq!(item_with_abbr.display_text(), "long_fn");
    }

    #[test]
    fn test_completion_item_flags() {
        let mut item = CompletionItem::new("test");
        item.flags = MatchFlags::OriginalText.to_raw();
        assert!(item.is_original_text());

        item.flags = MatchFlags::Icase.to_raw();
        assert!(item.is_icase());
    }

    #[test]
    fn test_completeopt_flags() {
        let flags = CompleteoptFlags::from_raw(cot_flags::MENU | cot_flags::MENUONE);
        assert!(flags.has_menu());
        assert!(flags.has_menuone());
        assert!(flags.should_show_pum());
        assert!(!flags.has_fuzzy());

        let fuzzy_flags = CompleteoptFlags::from_raw(cot_flags::FUZZY | cot_flags::NOSORT);
        assert!(fuzzy_flags.has_fuzzy());
        assert!(fuzzy_flags.has_nosort());
    }

    #[test]
    fn test_completion_text_field() {
        assert_eq!(
            CompletionTextField::from_index(0),
            Some(CompletionTextField::Abbr)
        );
        assert_eq!(
            CompletionTextField::from_index(3),
            Some(CompletionTextField::Info)
        );
        assert_eq!(CompletionTextField::from_index(4), None);

        assert_eq!(CompletionTextField::Abbr.to_index(), 0);
        assert_eq!(CompletionTextField::Info.to_index(), 3);
    }
}
