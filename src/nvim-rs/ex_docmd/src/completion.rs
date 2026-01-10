//! Command completion infrastructure for Ex commands.
//!
//! This module provides utilities for command-line completion,
//! including command name completion, argument completion,
//! and completion context management.

use std::ffi::{c_char, c_int};

// =============================================================================
// Completion types (Extended)
// =============================================================================

/// Extended completion type enumeration.
///
/// This extends the base CompletionType in table.rs with additional completion types.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExtendedCompletionType {
    /// No completion.
    None = 0,
    /// Command name completion.
    Command = 1,
    /// File/directory completion.
    File = 2,
    /// Directory only completion.
    Directory = 3,
    /// Buffer name completion.
    Buffer = 4,
    /// Help topic completion.
    Help = 5,
    /// Option name completion.
    Option = 6,
    /// Tag completion.
    Tag = 7,
    /// Function name completion.
    Function = 8,
    /// User-defined command completion.
    User = 9,
    /// Mapping completion.
    Mapping = 10,
    /// Menu completion.
    Menu = 11,
    /// Color scheme completion.
    Colorscheme = 12,
    /// Highlight group completion.
    Highlight = 13,
    /// Environment variable completion.
    Environment = 14,
    /// Event name completion.
    Event = 15,
    /// Expression completion.
    Expression = 16,
    /// Shell command completion.
    Shellcmd = 17,
    /// Sign completion.
    Sign = 18,
    /// Filetype completion.
    Filetype = 19,
    /// Package completion.
    Package = 20,
    /// Syntax completion.
    Syntax = 21,
    /// Custom function completion.
    Custom = 22,
    /// Custom list completion.
    CustomList = 23,
    /// User command completion.
    UserCmd = 24,
    /// User function completion.
    UserFunc = 25,
    /// Argument list completion.
    Arglist = 26,
    /// Compiler completion.
    Compiler = 27,
    /// Locale completion.
    Locale = 28,
    /// Messages completion.
    Messages = 29,
    /// History completion.
    History = 30,
    /// Lua completion.
    Lua = 31,
    /// Diff completion.
    Diff = 32,
    /// Terminal completion.
    Terminal = 33,
    /// Breakpoint completion.
    Breakpoint = 34,
    /// Scriptnames completion.
    Scriptnames = 35,
    /// Runtime completion.
    Runtime = 36,
    /// Checkhealth completion.
    Checkhealth = 37,
    /// MRU files completion.
    MruFiles = 38,
}

impl ExtendedCompletionType {
    /// Convert from C integer.
    #[inline]
    pub const fn from_c_int(val: c_int) -> Option<Self> {
        match val {
            0 => Some(Self::None),
            1 => Some(Self::Command),
            2 => Some(Self::File),
            3 => Some(Self::Directory),
            4 => Some(Self::Buffer),
            5 => Some(Self::Help),
            6 => Some(Self::Option),
            7 => Some(Self::Tag),
            8 => Some(Self::Function),
            9 => Some(Self::User),
            10 => Some(Self::Mapping),
            11 => Some(Self::Menu),
            12 => Some(Self::Colorscheme),
            13 => Some(Self::Highlight),
            14 => Some(Self::Environment),
            15 => Some(Self::Event),
            16 => Some(Self::Expression),
            17 => Some(Self::Shellcmd),
            18 => Some(Self::Sign),
            19 => Some(Self::Filetype),
            20 => Some(Self::Package),
            21 => Some(Self::Syntax),
            22 => Some(Self::Custom),
            23 => Some(Self::CustomList),
            24 => Some(Self::UserCmd),
            25 => Some(Self::UserFunc),
            26 => Some(Self::Arglist),
            27 => Some(Self::Compiler),
            28 => Some(Self::Locale),
            29 => Some(Self::Messages),
            30 => Some(Self::History),
            31 => Some(Self::Lua),
            32 => Some(Self::Diff),
            33 => Some(Self::Terminal),
            34 => Some(Self::Breakpoint),
            35 => Some(Self::Scriptnames),
            36 => Some(Self::Runtime),
            37 => Some(Self::Checkhealth),
            38 => Some(Self::MruFiles),
            _ => Option::None,
        }
    }

    /// Convert to C integer.
    #[inline]
    pub const fn to_c_int(self) -> c_int {
        self as c_int
    }

    /// Check if completion type is for files/directories.
    #[inline]
    pub const fn is_file_type(self) -> bool {
        matches!(self, Self::File | Self::Directory)
    }

    /// Check if completion type is user-defined.
    #[inline]
    pub const fn is_user_defined(self) -> bool {
        matches!(
            self,
            Self::User | Self::Custom | Self::CustomList | Self::UserCmd | Self::UserFunc
        )
    }
}

// =============================================================================
// Completion context
// =============================================================================

/// Completion context for command line.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct CompletionContext {
    /// Type of completion.
    pub comp_type: c_int,
    /// Start position of the argument to complete.
    pub arg_start: c_int,
    /// Current position in command line.
    pub cur_pos: c_int,
    /// Whether in command position (vs argument).
    pub in_command: bool,
    /// Whether after a range.
    pub after_range: bool,
    /// Command index if known.
    pub cmdidx: c_int,
    /// Number of arguments before current.
    pub arg_count: c_int,
    /// Whether command uses expression arguments.
    pub expr_args: bool,
}

impl CompletionContext {
    /// Create a new completion context.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            comp_type: 0,
            arg_start: 0,
            cur_pos: 0,
            in_command: true,
            after_range: false,
            cmdidx: -1,
            arg_count: 0,
            expr_args: false,
        }
    }

    /// Set completion type.
    pub fn set_type(&mut self, comp_type: ExtendedCompletionType) {
        self.comp_type = comp_type.to_c_int();
    }

    /// Get completion type.
    #[must_use]
    pub fn get_type(&self) -> Option<ExtendedCompletionType> {
        ExtendedCompletionType::from_c_int(self.comp_type)
    }

    /// Check if completing a command name.
    #[must_use]
    pub const fn is_command_completion(&self) -> bool {
        self.in_command
    }

    /// Check if completing an argument.
    #[must_use]
    pub const fn is_argument_completion(&self) -> bool {
        !self.in_command
    }

    /// Get argument length to complete.
    #[must_use]
    pub const fn arg_len(&self) -> c_int {
        if self.cur_pos >= self.arg_start {
            self.cur_pos - self.arg_start
        } else {
            0
        }
    }
}

impl Default for CompletionContext {
    fn default() -> Self {
        Self::new()
    }
}

// =============================================================================
// Completion flags
// =============================================================================

/// Flags for command completion behavior.
pub mod comp_flags {
    use std::ffi::c_int;

    /// Complete directories only.
    pub const DIRECTORY: c_int = 0x01;
    /// Add slash after directories.
    pub const ADD_SLASH: c_int = 0x02;
    /// Escape special characters.
    pub const ESCAPE: c_int = 0x04;
    /// Case insensitive matching.
    pub const ICASE: c_int = 0x08;
    /// Include hidden files.
    pub const HIDDEN: c_int = 0x10;
    /// Allow empty completion.
    pub const ALLOW_EMPTY: c_int = 0x20;
    /// Keep original text.
    pub const KEEP_ORIG: c_int = 0x40;
    /// No duplicates.
    pub const NO_DUP: c_int = 0x80;
}

/// Check if completion flags include a specific flag.
#[inline]
pub const fn has_flag(flags: c_int, flag: c_int) -> bool {
    (flags & flag) != 0
}

/// Set a completion flag.
#[inline]
pub const fn set_flag(flags: c_int, flag: c_int) -> c_int {
    flags | flag
}

/// Clear a completion flag.
#[inline]
pub const fn clear_flag(flags: c_int, flag: c_int) -> c_int {
    flags & !flag
}

// =============================================================================
// Command name matching
// =============================================================================

/// Check if a command name matches a prefix.
///
/// This performs case-insensitive prefix matching for command names.
#[inline]
pub fn cmd_matches_prefix(cmd: &[u8], prefix: &[u8]) -> bool {
    if prefix.is_empty() {
        return true;
    }
    if cmd.len() < prefix.len() {
        return false;
    }
    for (c, p) in cmd.iter().zip(prefix.iter()) {
        if !c.eq_ignore_ascii_case(p) {
            return false;
        }
    }
    true
}

/// Check if a string is a valid command name character.
#[inline]
pub const fn is_cmd_char(c: u8) -> bool {
    c.is_ascii_alphabetic()
}

/// Count command characters from the start of a string.
pub fn cmd_char_count(s: &[u8]) -> usize {
    s.iter().take_while(|&&c| is_cmd_char(c)).count()
}

// =============================================================================
// FFI exports
// =============================================================================

/// Create a new completion context.
#[no_mangle]
pub extern "C" fn rs_completion_context_new() -> CompletionContext {
    CompletionContext::new()
}

/// Set completion type.
///
/// # Safety
/// `ctx` must be valid.
#[no_mangle]
pub unsafe extern "C" fn rs_completion_context_set_type(
    ctx: *mut CompletionContext,
    comp_type: c_int,
) {
    if !ctx.is_null() {
        (*ctx).comp_type = comp_type;
    }
}

/// Get completion type.
///
/// # Safety
/// `ctx` must be valid.
#[no_mangle]
pub unsafe extern "C" fn rs_completion_context_get_type(ctx: *const CompletionContext) -> c_int {
    if ctx.is_null() {
        return 0;
    }
    (*ctx).comp_type
}

/// Check if in command completion.
///
/// # Safety
/// `ctx` must be valid.
#[no_mangle]
pub unsafe extern "C" fn rs_completion_context_is_command(ctx: *const CompletionContext) -> c_int {
    if ctx.is_null() {
        return 0;
    }
    c_int::from((*ctx).is_command_completion())
}

/// Check if in argument completion.
///
/// # Safety
/// `ctx` must be valid.
#[no_mangle]
pub unsafe extern "C" fn rs_completion_context_is_argument(ctx: *const CompletionContext) -> c_int {
    if ctx.is_null() {
        return 0;
    }
    c_int::from((*ctx).is_argument_completion())
}

/// Get argument length.
///
/// # Safety
/// `ctx` must be valid.
#[no_mangle]
pub unsafe extern "C" fn rs_completion_context_arg_len(ctx: *const CompletionContext) -> c_int {
    if ctx.is_null() {
        return 0;
    }
    (*ctx).arg_len()
}

/// Convert extended completion type from int.
#[no_mangle]
pub extern "C" fn rs_ext_completion_type_from_int(val: c_int) -> c_int {
    match ExtendedCompletionType::from_c_int(val) {
        Some(t) => t.to_c_int(),
        Option::None => -1,
    }
}

/// Check if extended completion type is file type.
#[no_mangle]
pub extern "C" fn rs_ext_completion_type_is_file(val: c_int) -> c_int {
    match ExtendedCompletionType::from_c_int(val) {
        Some(t) => c_int::from(t.is_file_type()),
        Option::None => 0,
    }
}

/// Check if extended completion type is user-defined.
#[no_mangle]
pub extern "C" fn rs_ext_completion_type_is_user_defined(val: c_int) -> c_int {
    match ExtendedCompletionType::from_c_int(val) {
        Some(t) => c_int::from(t.is_user_defined()),
        Option::None => 0,
    }
}

/// Check completion flag.
#[no_mangle]
pub extern "C" fn rs_completion_has_flag(flags: c_int, flag: c_int) -> c_int {
    c_int::from(has_flag(flags, flag))
}

/// Set completion flag.
#[no_mangle]
pub extern "C" fn rs_completion_set_flag(flags: c_int, flag: c_int) -> c_int {
    set_flag(flags, flag)
}

/// Clear completion flag.
#[no_mangle]
pub extern "C" fn rs_completion_clear_flag(flags: c_int, flag: c_int) -> c_int {
    clear_flag(flags, flag)
}

/// Check if character is a valid command character.
#[no_mangle]
#[allow(clippy::manual_range_contains)]
pub extern "C" fn rs_is_cmd_char(c: c_int) -> c_int {
    if c < 0 || c > 127 {
        return 0;
    }
    c_int::from(is_cmd_char(c as u8))
}

/// Count command characters from a string.
///
/// # Safety
/// `s` must be a valid null-terminated C string.
#[no_mangle]
pub unsafe extern "C" fn rs_cmd_char_count(s: *const c_char) -> c_int {
    if s.is_null() {
        return 0;
    }
    let mut count = 0;
    let mut ptr = s;
    while *ptr != 0 && is_cmd_char(*ptr as u8) {
        count += 1;
        ptr = ptr.add(1);
    }
    count
}

/// Check if command matches prefix.
///
/// # Safety
/// `cmd` and `prefix` must be valid null-terminated C strings.
#[no_mangle]
pub unsafe extern "C" fn rs_cmd_matches_prefix(cmd: *const c_char, prefix: *const c_char) -> c_int {
    if cmd.is_null() {
        return 0;
    }
    if prefix.is_null() {
        return 1; // Empty prefix matches everything
    }

    let mut cmd_ptr = cmd;
    let mut prefix_ptr = prefix;

    while *prefix_ptr != 0 {
        if *cmd_ptr == 0 {
            return 0; // Command shorter than prefix
        }
        let c = (*cmd_ptr as u8).to_ascii_lowercase();
        let p = (*prefix_ptr as u8).to_ascii_lowercase();
        if c != p {
            return 0;
        }
        cmd_ptr = cmd_ptr.add(1);
        prefix_ptr = prefix_ptr.add(1);
    }
    1
}

// =============================================================================
// Completion type constants
// =============================================================================

/// EXPAND_NOTHING
pub const EXPAND_NOTHING: c_int = 0;
/// EXPAND_COMMANDS
pub const EXPAND_COMMANDS: c_int = 1;
/// EXPAND_FILES
pub const EXPAND_FILES: c_int = 2;
/// EXPAND_DIRECTORIES
pub const EXPAND_DIRECTORIES: c_int = 3;
/// EXPAND_BUFFERS
pub const EXPAND_BUFFERS: c_int = 4;
/// EXPAND_HELP
pub const EXPAND_HELP: c_int = 5;
/// EXPAND_SETTINGS
pub const EXPAND_SETTINGS: c_int = 6;
/// EXPAND_TAGS
pub const EXPAND_TAGS: c_int = 7;
/// EXPAND_FUNCTIONS
pub const EXPAND_FUNCTIONS: c_int = 8;
/// EXPAND_USER_DEFINED
pub const EXPAND_USER_DEFINED: c_int = 9;
/// EXPAND_MAPPINGS
pub const EXPAND_MAPPINGS: c_int = 10;

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_completion_type_conversion() {
        assert_eq!(ExtendedCompletionType::None.to_c_int(), 0);
        assert_eq!(ExtendedCompletionType::Command.to_c_int(), 1);
        assert_eq!(ExtendedCompletionType::File.to_c_int(), 2);

        assert_eq!(
            ExtendedCompletionType::from_c_int(0),
            Some(ExtendedCompletionType::None)
        );
        assert_eq!(
            ExtendedCompletionType::from_c_int(1),
            Some(ExtendedCompletionType::Command)
        );
        assert_eq!(ExtendedCompletionType::from_c_int(99), Option::None);
    }

    #[test]
    fn test_completion_type_is_file() {
        assert!(ExtendedCompletionType::File.is_file_type());
        assert!(ExtendedCompletionType::Directory.is_file_type());
        assert!(!ExtendedCompletionType::Buffer.is_file_type());
        assert!(!ExtendedCompletionType::Command.is_file_type());
    }

    #[test]
    fn test_completion_type_is_user_defined() {
        assert!(ExtendedCompletionType::User.is_user_defined());
        assert!(ExtendedCompletionType::Custom.is_user_defined());
        assert!(ExtendedCompletionType::CustomList.is_user_defined());
        assert!(!ExtendedCompletionType::File.is_user_defined());
        assert!(!ExtendedCompletionType::Command.is_user_defined());
    }

    #[test]
    fn test_completion_context() {
        let ctx = CompletionContext::new();
        assert!(ctx.is_command_completion());
        assert!(!ctx.is_argument_completion());
        assert_eq!(ctx.arg_len(), 0);

        let mut ctx = CompletionContext::new();
        ctx.in_command = false;
        ctx.arg_start = 5;
        ctx.cur_pos = 10;
        assert!(!ctx.is_command_completion());
        assert!(ctx.is_argument_completion());
        assert_eq!(ctx.arg_len(), 5);
    }

    #[test]
    fn test_completion_flags() {
        let flags = 0;
        assert!(!has_flag(flags, comp_flags::DIRECTORY));

        let flags = set_flag(flags, comp_flags::DIRECTORY);
        assert!(has_flag(flags, comp_flags::DIRECTORY));

        let flags = set_flag(flags, comp_flags::ESCAPE);
        assert!(has_flag(flags, comp_flags::DIRECTORY));
        assert!(has_flag(flags, comp_flags::ESCAPE));

        let flags = clear_flag(flags, comp_flags::DIRECTORY);
        assert!(!has_flag(flags, comp_flags::DIRECTORY));
        assert!(has_flag(flags, comp_flags::ESCAPE));
    }

    #[test]
    fn test_cmd_matches_prefix() {
        assert!(cmd_matches_prefix(b"edit", b"ed"));
        assert!(cmd_matches_prefix(b"Edit", b"ed"));
        assert!(cmd_matches_prefix(b"edit", b"ED"));
        assert!(cmd_matches_prefix(b"edit", b""));
        assert!(cmd_matches_prefix(b"edit", b"edit"));
        assert!(!cmd_matches_prefix(b"edit", b"edita"));
        assert!(!cmd_matches_prefix(b"edit", b"ex"));
    }

    #[test]
    fn test_is_cmd_char() {
        assert!(is_cmd_char(b'a'));
        assert!(is_cmd_char(b'z'));
        assert!(is_cmd_char(b'A'));
        assert!(is_cmd_char(b'Z'));
        assert!(!is_cmd_char(b'0'));
        assert!(!is_cmd_char(b' '));
        assert!(!is_cmd_char(b'_'));
    }

    #[test]
    fn test_cmd_char_count() {
        assert_eq!(cmd_char_count(b"edit file"), 4);
        assert_eq!(cmd_char_count(b"e"), 1);
        assert_eq!(cmd_char_count(b"123"), 0);
        assert_eq!(cmd_char_count(b""), 0);
        assert_eq!(cmd_char_count(b"quit!"), 4);
    }

    #[test]
    fn test_ffi_completion_context() {
        let ctx = rs_completion_context_new();
        assert_eq!(unsafe { rs_completion_context_is_command(&ctx) }, 1);
        assert_eq!(unsafe { rs_completion_context_is_argument(&ctx) }, 0);

        let mut ctx = CompletionContext::new();
        ctx.in_command = false;
        assert_eq!(unsafe { rs_completion_context_is_command(&ctx) }, 0);
        assert_eq!(unsafe { rs_completion_context_is_argument(&ctx) }, 1);
    }

    #[test]
    fn test_ffi_completion_type() {
        assert_eq!(rs_ext_completion_type_from_int(0), 0);
        assert_eq!(rs_ext_completion_type_from_int(2), 2);
        assert_eq!(rs_ext_completion_type_from_int(99), -1);

        assert_eq!(rs_ext_completion_type_is_file(2), 1); // File
        assert_eq!(rs_ext_completion_type_is_file(3), 1); // Directory
        assert_eq!(rs_ext_completion_type_is_file(4), 0); // Buffer

        assert_eq!(rs_ext_completion_type_is_user_defined(9), 1); // User
        assert_eq!(rs_ext_completion_type_is_user_defined(1), 0); // Command
    }
}
