//! User command completion handling
//!
//! This module provides Rust implementations for user command completion,
//! including completion types, matching, and result handling.

#![allow(clippy::must_use_candidate)]
#![allow(clippy::doc_markdown)]
#![allow(clippy::missing_const_for_fn)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_possible_wrap)]
#![allow(clippy::derivable_impls)]
#![allow(clippy::match_same_arms)]
#![allow(clippy::if_same_then_else)]

use std::ffi::c_int;

// =============================================================================
// Completion Type Constants
// =============================================================================

/// No completion
pub const EXPAND_NOTHING: c_int = 0;
/// Command completion
pub const EXPAND_COMMANDS: c_int = 1;
/// File completion
pub const EXPAND_FILES: c_int = 2;
/// Directory completion
pub const EXPAND_DIRECTORIES: c_int = 3;
/// Settings completion
pub const EXPAND_SETTINGS: c_int = 4;
/// Boolean settings completion
pub const EXPAND_BOOL_SETTINGS: c_int = 5;
/// Tags completion
pub const EXPAND_TAGS: c_int = 6;
/// Tags files completion
pub const EXPAND_TAGS_LISTFILES: c_int = 7;
/// Help topics completion
pub const EXPAND_HELP: c_int = 8;
/// Buffer names completion
pub const EXPAND_BUFFERS: c_int = 9;
/// Events completion
pub const EXPAND_EVENTS: c_int = 10;
/// Menus completion
pub const EXPAND_MENUS: c_int = 11;
/// Syntax completion
pub const EXPAND_SYNTAX: c_int = 12;
/// Highlight groups completion
pub const EXPAND_HIGHLIGHT: c_int = 13;
/// Autocommand groups completion
pub const EXPAND_AUGROUP: c_int = 14;
/// User-defined vars completion
pub const EXPAND_USER_VARS: c_int = 15;
/// Mappings completion
pub const EXPAND_MAPPINGS: c_int = 16;
/// Functions completion
pub const EXPAND_FUNCTIONS: c_int = 17;
/// User functions completion
pub const EXPAND_USER_FUNC: c_int = 18;
/// Expression completion
pub const EXPAND_EXPRESSION: c_int = 19;
/// Menu names completion
pub const EXPAND_MENUNAMES: c_int = 20;
/// User commands completion
pub const EXPAND_USER_COMMANDS: c_int = 21;
/// User command nargs completion
pub const EXPAND_USER_CMD_FLAGS: c_int = 22;
/// User-defined completion
pub const EXPAND_USER_DEFINED: c_int = 23;
/// User list completion
pub const EXPAND_USER_LIST: c_int = 24;
/// Shellcmd completion
pub const EXPAND_SHELLCMD: c_int = 25;
/// Colors completion
pub const EXPAND_COLORS: c_int = 26;
/// Compiler completion
pub const EXPAND_COMPILER: c_int = 27;
/// User addr type completion
pub const EXPAND_USER_ADDR_TYPE: c_int = 28;
/// Packadd completion
pub const EXPAND_PACKADD: c_int = 29;
/// Lua completion
pub const EXPAND_LUA: c_int = 30;

// =============================================================================
// Completion Type Enum
// =============================================================================

/// Completion type for user commands
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompleteType {
    /// No completion
    None = 0,
    /// Argument list
    Arglist = 1,
    /// Autocommand groups
    Augroup = 2,
    /// Buffer names
    Buffer = 3,
    /// Behave suboptions
    Behave = 4,
    /// Color schemes
    Color = 5,
    /// Commands
    Command = 6,
    /// Compilers
    Compiler = 7,
    /// C type definitions
    Cscope = 8,
    /// Directories
    Dir = 9,
    /// Environment variables
    Environment = 10,
    /// Autocommand events
    Event = 11,
    /// Expression
    Expression = 12,
    /// Files and directories
    File = 13,
    /// Files in path
    FileInPath = 14,
    /// Filetype names
    Filetype = 15,
    /// Functions
    Function = 16,
    /// Help topics
    Help = 17,
    /// Highlight groups
    Highlight = 18,
    /// History types
    History = 19,
    /// Locale names
    Locale = 20,
    /// Lua expression
    Lua = 21,
    /// Mappings
    Mapping = 22,
    /// Menus
    Menu = 23,
    /// Messages
    Messages = 24,
    /// Options
    Option = 25,
    /// Packages
    Packadd = 26,
    /// Shell commands
    Shellcmd = 27,
    /// Signs
    Sign = 28,
    /// Syntax items
    Syntax = 29,
    /// Syntax file names
    Syntime = 30,
    /// Tags
    Tag = 31,
    /// Tag files
    TagListfiles = 32,
    /// User-defined
    User = 33,
    /// User variables
    Var = 34,
    /// Custom function (user-defined function)
    Custom = 35,
    /// Custom list (user-defined function returning list)
    CustomList = 36,
}

impl CompleteType {
    /// Create from raw integer value
    pub const fn from_raw(value: c_int) -> Option<Self> {
        match value {
            0 => Some(Self::None),
            1 => Some(Self::Arglist),
            2 => Some(Self::Augroup),
            3 => Some(Self::Buffer),
            4 => Some(Self::Behave),
            5 => Some(Self::Color),
            6 => Some(Self::Command),
            7 => Some(Self::Compiler),
            8 => Some(Self::Cscope),
            9 => Some(Self::Dir),
            10 => Some(Self::Environment),
            11 => Some(Self::Event),
            12 => Some(Self::Expression),
            13 => Some(Self::File),
            14 => Some(Self::FileInPath),
            15 => Some(Self::Filetype),
            16 => Some(Self::Function),
            17 => Some(Self::Help),
            18 => Some(Self::Highlight),
            19 => Some(Self::History),
            20 => Some(Self::Locale),
            21 => Some(Self::Lua),
            22 => Some(Self::Mapping),
            23 => Some(Self::Menu),
            24 => Some(Self::Messages),
            25 => Some(Self::Option),
            26 => Some(Self::Packadd),
            27 => Some(Self::Shellcmd),
            28 => Some(Self::Sign),
            29 => Some(Self::Syntax),
            30 => Some(Self::Syntime),
            31 => Some(Self::Tag),
            32 => Some(Self::TagListfiles),
            33 => Some(Self::User),
            34 => Some(Self::Var),
            35 => Some(Self::Custom),
            36 => Some(Self::CustomList),
            _ => None,
        }
    }

    /// Convert to raw integer value
    pub const fn as_raw(self) -> c_int {
        self as c_int
    }

    /// Check if this completion type uses files
    pub const fn uses_files(self) -> bool {
        matches!(self, Self::File | Self::Dir | Self::FileInPath)
    }

    /// Check if this is a custom completion
    pub const fn is_custom(self) -> bool {
        matches!(self, Self::Custom | Self::CustomList | Self::User)
    }

    /// Get the short name for this completion type
    pub const fn short_name(self) -> &'static str {
        match self {
            Self::None => "",
            Self::Arglist => "arglist",
            Self::Augroup => "augroup",
            Self::Buffer => "buffer",
            Self::Behave => "behave",
            Self::Color => "color",
            Self::Command => "command",
            Self::Compiler => "compiler",
            Self::Cscope => "cscope",
            Self::Dir => "dir",
            Self::Environment => "environment",
            Self::Event => "event",
            Self::Expression => "expression",
            Self::File => "file",
            Self::FileInPath => "file_in_path",
            Self::Filetype => "filetype",
            Self::Function => "function",
            Self::Help => "help",
            Self::Highlight => "highlight",
            Self::History => "history",
            Self::Locale => "locale",
            Self::Lua => "lua",
            Self::Mapping => "mapping",
            Self::Menu => "menu",
            Self::Messages => "messages",
            Self::Option => "option",
            Self::Packadd => "packadd",
            Self::Shellcmd => "shellcmd",
            Self::Sign => "sign",
            Self::Syntax => "syntax",
            Self::Syntime => "syntime",
            Self::Tag => "tag",
            Self::TagListfiles => "tag_listfiles",
            Self::User => "custom",
            Self::Var => "var",
            Self::Custom => "customlist",
            Self::CustomList => "customlist",
        }
    }
}

impl Default for CompleteType {
    fn default() -> Self {
        Self::None
    }
}

// =============================================================================
// Completion Context
// =============================================================================

/// Context for user command completion
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct CompleteContext {
    /// The completion type
    pub ctype: CompleteType,
    /// Argument position (for positional completion)
    pub argpos: c_int,
    /// Whether to expand environment variables
    pub expand_env: bool,
    /// Whether completion is fuzzy
    pub fuzzy: bool,
}

impl Default for CompleteContext {
    fn default() -> Self {
        Self {
            ctype: CompleteType::None,
            argpos: 0,
            expand_env: false,
            fuzzy: false,
        }
    }
}

impl CompleteContext {
    /// Create context for file completion
    pub const fn for_file() -> Self {
        Self {
            ctype: CompleteType::File,
            argpos: 0,
            expand_env: true,
            fuzzy: false,
        }
    }

    /// Create context for directory completion
    pub const fn for_dir() -> Self {
        Self {
            ctype: CompleteType::Dir,
            argpos: 0,
            expand_env: true,
            fuzzy: false,
        }
    }

    /// Create context for buffer completion
    pub const fn for_buffer() -> Self {
        Self {
            ctype: CompleteType::Buffer,
            argpos: 0,
            expand_env: false,
            fuzzy: false,
        }
    }

    /// Check if completion should expand env vars
    pub const fn should_expand_env(&self) -> bool {
        self.expand_env && self.ctype.uses_files()
    }
}

// =============================================================================
// Completion Result
// =============================================================================

/// Result of a completion match
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct CompleteMatch {
    /// Match score (higher is better)
    pub score: c_int,
    /// Whether this is an exact match
    pub exact: bool,
    /// Whether this match is case-sensitive
    pub case_sensitive: bool,
    /// Match index in the list
    pub index: c_int,
}

impl Default for CompleteMatch {
    fn default() -> Self {
        Self {
            score: 0,
            exact: false,
            case_sensitive: true,
            index: -1,
        }
    }
}

impl CompleteMatch {
    /// Create an exact match
    pub const fn exact(index: c_int) -> Self {
        Self {
            score: 1000,
            exact: true,
            case_sensitive: true,
            index,
        }
    }

    /// Create a prefix match
    pub const fn prefix(index: c_int, score: c_int) -> Self {
        Self {
            score,
            exact: false,
            case_sensitive: true,
            index,
        }
    }

    /// Check if this is a valid match
    pub const fn is_valid(&self) -> bool {
        self.index >= 0
    }
}

// =============================================================================
// Completion State
// =============================================================================

/// Completion operation state
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct CompleteState {
    /// Number of matches found
    pub match_count: c_int,
    /// Index of selected match
    pub selected: c_int,
    /// Whether completion is active
    pub active: bool,
    /// Whether all matches have been computed
    pub complete: bool,
}

impl Default for CompleteState {
    fn default() -> Self {
        Self {
            match_count: 0,
            selected: -1,
            active: false,
            complete: false,
        }
    }
}

impl CompleteState {
    /// Check if there are matches
    pub const fn has_matches(&self) -> bool {
        self.match_count > 0
    }

    /// Check if a match is selected
    pub const fn has_selection(&self) -> bool {
        self.selected >= 0 && self.selected < self.match_count
    }

    /// Get next match index (wrapping)
    pub const fn next_match(&self) -> c_int {
        if self.match_count == 0 {
            return -1;
        }
        if self.selected < 0 {
            0
        } else if self.selected >= self.match_count - 1 {
            0
        } else {
            self.selected + 1
        }
    }

    /// Get previous match index (wrapping)
    pub const fn prev_match(&self) -> c_int {
        if self.match_count == 0 {
            return -1;
        }
        if self.selected <= 0 {
            self.match_count - 1
        } else {
            self.selected - 1
        }
    }
}

// =============================================================================
// FFI Exports
// =============================================================================

/// FFI export: Check if completion type is valid
#[no_mangle]
pub extern "C" fn rs_usercmd_complete_type_valid(ctype: c_int) -> c_int {
    c_int::from(CompleteType::from_raw(ctype).is_some())
}

/// FFI export: Check if completion type uses files
#[no_mangle]
pub extern "C" fn rs_usercmd_complete_uses_files(ctype: c_int) -> c_int {
    CompleteType::from_raw(ctype).map_or(0, |t| c_int::from(t.uses_files()))
}

/// FFI export: Check if completion type is custom
#[no_mangle]
pub extern "C" fn rs_usercmd_complete_is_custom(ctype: c_int) -> c_int {
    CompleteType::from_raw(ctype).map_or(0, |t| c_int::from(t.is_custom()))
}

/// FFI export: Get next match index
#[no_mangle]
pub extern "C" fn rs_usercmd_complete_next_match(match_count: c_int, selected: c_int) -> c_int {
    let state = CompleteState {
        match_count,
        selected,
        active: true,
        complete: true,
    };
    state.next_match()
}

/// FFI export: Get previous match index
#[no_mangle]
pub extern "C" fn rs_usercmd_complete_prev_match(match_count: c_int, selected: c_int) -> c_int {
    let state = CompleteState {
        match_count,
        selected,
        active: true,
        complete: true,
    };
    state.prev_match()
}

/// FFI export: Create file completion context
#[no_mangle]
pub extern "C" fn rs_usercmd_complete_context_file() -> CompleteContext {
    CompleteContext::for_file()
}

/// FFI export: Create buffer completion context
#[no_mangle]
pub extern "C" fn rs_usercmd_complete_context_buffer() -> CompleteContext {
    CompleteContext::for_buffer()
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_complete_type_from_raw() {
        assert_eq!(CompleteType::from_raw(0), Some(CompleteType::None));
        assert_eq!(CompleteType::from_raw(13), Some(CompleteType::File));
        assert_eq!(CompleteType::from_raw(100), None);
    }

    #[test]
    fn test_complete_type_properties() {
        assert!(CompleteType::File.uses_files());
        assert!(CompleteType::Dir.uses_files());
        assert!(!CompleteType::Buffer.uses_files());

        assert!(CompleteType::Custom.is_custom());
        assert!(CompleteType::CustomList.is_custom());
        assert!(!CompleteType::File.is_custom());
    }

    #[test]
    fn test_complete_context() {
        let file = CompleteContext::for_file();
        assert!(file.should_expand_env());

        let buffer = CompleteContext::for_buffer();
        assert!(!buffer.should_expand_env());
    }

    #[test]
    fn test_complete_match() {
        let exact = CompleteMatch::exact(0);
        assert!(exact.is_valid());
        assert!(exact.exact);
        assert_eq!(exact.score, 1000);

        let prefix = CompleteMatch::prefix(1, 500);
        assert!(prefix.is_valid());
        assert!(!prefix.exact);
    }

    #[test]
    fn test_complete_state_navigation() {
        let state = CompleteState {
            match_count: 5,
            selected: 2,
            active: true,
            complete: true,
        };

        assert!(state.has_matches());
        assert!(state.has_selection());
        assert_eq!(state.next_match(), 3);
        assert_eq!(state.prev_match(), 1);

        // Test wrap around
        let at_end = CompleteState {
            match_count: 5,
            selected: 4,
            ..state
        };
        assert_eq!(at_end.next_match(), 0);

        let at_start = CompleteState {
            match_count: 5,
            selected: 0,
            ..state
        };
        assert_eq!(at_start.prev_match(), 4);
    }

    #[test]
    fn test_ffi_exports() {
        assert_eq!(rs_usercmd_complete_type_valid(13), 1);
        assert_eq!(rs_usercmd_complete_type_valid(100), 0);

        assert_eq!(rs_usercmd_complete_uses_files(13), 1);
        assert_eq!(rs_usercmd_complete_uses_files(3), 0);

        assert_eq!(rs_usercmd_complete_next_match(5, 2), 3);
        assert_eq!(rs_usercmd_complete_prev_match(5, 2), 1);
    }
}
