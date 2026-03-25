//! Command-line expansion system for Neovim
//!
//! This module provides the Rust implementation of cmdexpand.c functionality,
//! including completion context management, pattern matching for command-line
//! completion, and wildmenu support.

#![allow(unsafe_code)]
#![allow(clippy::doc_markdown)]

use std::ffi::{c_char, c_int, c_uint};
use std::ptr;

// =============================================================================
// Expansion Context Types (from cmdexpand_defs.h)
// =============================================================================

/// Type of completion context for command-line expansion.
///
/// These values match the C enum in `cmdexpand_defs.h`.
#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExpandContext {
    /// Expansion failed
    Unsuccessful = -2,
    /// Expansion succeeded
    Ok = -1,
    /// Nothing to expand
    Nothing = 0,
    /// Expand command names
    Commands = 1,
    /// Expand file names
    Files = 2,
    /// Expand directory names
    Directories = 3,
    /// Expand option settings
    Settings = 4,
    /// Expand boolean settings
    BoolSettings = 5,
    /// Expand tag names
    Tags = 6,
    /// Expand old setting value
    OldSetting = 7,
    /// Expand help tags
    Help = 8,
    /// Expand buffer names
    Buffers = 9,
    /// Expand event names
    Events = 10,
    /// Expand menu names
    Menus = 11,
    /// Expand syntax group names
    Syntax = 12,
    /// Expand highlight group names
    Highlight = 13,
    /// Expand autocmd group names
    Augroup = 14,
    /// Expand user-defined variables
    UserVars = 15,
    /// Expand key mappings
    Mappings = 16,
    /// Expand tags and list files
    TagsListfiles = 17,
    /// Expand function names
    Functions = 18,
    /// Expand user function names
    UserFunc = 19,
    /// Expand expression
    Expression = 20,
    /// Expand menu names (deeper)
    Menunames = 21,
    /// Expand user-defined command names
    UserCommands = 22,
    /// Expand user command flags
    UserCmdFlags = 23,
    /// Expand user command nargs
    UserNargs = 24,
    /// Expand user command complete types
    UserComplete = 25,
    /// Expand environment variables
    EnvVars = 26,
    /// Expand language names
    Language = 27,
    /// Expand colorscheme names
    Colors = 28,
    /// Expand compiler names
    Compiler = 29,
    /// Expand user-defined completion
    UserDefined = 30,
    /// Expand user list completion
    UserList = 31,
    /// Expand user Lua completion
    UserLua = 32,
    /// Expand shell commands
    Shellcmd = 33,
    /// Expand sign commands
    Sign = 34,
    /// Expand profile commands
    Profile = 35,
    /// Expand filetype names
    Filetype = 36,
    /// Expand files in path
    FilesInPath = 37,
    /// Expand own syntax files
    Ownsyntax = 38,
    /// Expand locale names
    Locales = 39,
    /// Expand history types
    History = 40,
    /// Expand user names
    User = 41,
    /// Expand syntime arguments
    Syntime = 42,
    /// Expand user address types
    UserAddrType = 43,
    /// Expand packadd names
    Packadd = 44,
    /// Expand message types
    Messages = 45,
    /// Expand mapclear arguments
    Mapclear = 46,
    /// Expand arglist entries
    Arglist = 47,
    /// Expand diff buffers
    DiffBuffers = 48,
    /// Expand breakpoint locations
    Breakpoint = 49,
    /// Expand script names
    Scriptnames = 50,
    /// Expand runtime paths
    Runtime = 51,
    /// Expand string setting values
    StringSetting = 52,
    /// Expand setting subtract values
    SettingSubtract = 53,
    /// Expand argument options
    Argopt = 54,
    /// Expand keymap names
    Keymap = 55,
    /// Expand directories in cdpath
    DirsInCdpath = 56,
    /// Expand shell command line
    Shellcmdline = 57,
    /// Expand findfunc results
    Findfunc = 58,
    /// Expand filetype command arguments
    Filetypecmd = 59,
    /// Expand pattern in buffer
    PatternInBuf = 60,
    /// Expand retab arguments
    Retab = 61,
    /// Expand checkhealth arguments
    Checkhealth = 62,
    /// Expand Lua expression
    Lua = 63,
}

impl ExpandContext {
    /// Convert from raw C integer to ExpandContext.
    ///
    /// Returns `None` if the value is out of range.
    #[must_use]
    pub const fn from_raw(value: c_int) -> Option<Self> {
        match value {
            -2 => Some(Self::Unsuccessful),
            -1 => Some(Self::Ok),
            0 => Some(Self::Nothing),
            1 => Some(Self::Commands),
            2 => Some(Self::Files),
            3 => Some(Self::Directories),
            4 => Some(Self::Settings),
            5 => Some(Self::BoolSettings),
            6 => Some(Self::Tags),
            7 => Some(Self::OldSetting),
            8 => Some(Self::Help),
            9 => Some(Self::Buffers),
            10 => Some(Self::Events),
            11 => Some(Self::Menus),
            12 => Some(Self::Syntax),
            13 => Some(Self::Highlight),
            14 => Some(Self::Augroup),
            15 => Some(Self::UserVars),
            16 => Some(Self::Mappings),
            17 => Some(Self::TagsListfiles),
            18 => Some(Self::Functions),
            19 => Some(Self::UserFunc),
            20 => Some(Self::Expression),
            21 => Some(Self::Menunames),
            22 => Some(Self::UserCommands),
            23 => Some(Self::UserCmdFlags),
            24 => Some(Self::UserNargs),
            25 => Some(Self::UserComplete),
            26 => Some(Self::EnvVars),
            27 => Some(Self::Language),
            28 => Some(Self::Colors),
            29 => Some(Self::Compiler),
            30 => Some(Self::UserDefined),
            31 => Some(Self::UserList),
            32 => Some(Self::UserLua),
            33 => Some(Self::Shellcmd),
            34 => Some(Self::Sign),
            35 => Some(Self::Profile),
            36 => Some(Self::Filetype),
            37 => Some(Self::FilesInPath),
            38 => Some(Self::Ownsyntax),
            39 => Some(Self::Locales),
            40 => Some(Self::History),
            41 => Some(Self::User),
            42 => Some(Self::Syntime),
            43 => Some(Self::UserAddrType),
            44 => Some(Self::Packadd),
            45 => Some(Self::Messages),
            46 => Some(Self::Mapclear),
            47 => Some(Self::Arglist),
            48 => Some(Self::DiffBuffers),
            49 => Some(Self::Breakpoint),
            50 => Some(Self::Scriptnames),
            51 => Some(Self::Runtime),
            52 => Some(Self::StringSetting),
            53 => Some(Self::SettingSubtract),
            54 => Some(Self::Argopt),
            55 => Some(Self::Keymap),
            56 => Some(Self::DirsInCdpath),
            57 => Some(Self::Shellcmdline),
            58 => Some(Self::Findfunc),
            59 => Some(Self::Filetypecmd),
            60 => Some(Self::PatternInBuf),
            61 => Some(Self::Retab),
            62 => Some(Self::Checkhealth),
            63 => Some(Self::Lua),
            _ => None,
        }
    }

    /// Convert to raw C integer.
    #[must_use]
    pub const fn to_raw(self) -> c_int {
        self as c_int
    }

    /// Check if this context supports fuzzy completion.
    ///
    /// Some contexts like files, help, and user-defined completion
    /// do not support fuzzy matching.
    #[must_use]
    pub const fn supports_fuzzy(&self) -> bool {
        !matches!(
            self,
            Self::BoolSettings
                | Self::Colors
                | Self::Compiler
                | Self::Directories
                | Self::DirsInCdpath
                | Self::Files
                | Self::FilesInPath
                | Self::Filetype
                | Self::Filetypecmd
                | Self::Findfunc
                | Self::Help
                | Self::Keymap
                | Self::Lua
                | Self::OldSetting
                | Self::StringSetting
                | Self::SettingSubtract
                | Self::Ownsyntax
                | Self::Packadd
                | Self::Runtime
                | Self::Shellcmd
                | Self::Shellcmdline
                | Self::Tags
                | Self::TagsListfiles
                | Self::UserList
                | Self::UserLua
        )
    }

    /// Check if this context is for file-like expansion.
    #[must_use]
    pub const fn is_file_expansion(&self) -> bool {
        matches!(
            self,
            Self::Files
                | Self::FilesInPath
                | Self::Shellcmd
                | Self::Buffers
                | Self::Directories
                | Self::DirsInCdpath
        )
    }

    /// Check if this context uses internal pattern matching (not file wildcards).
    #[must_use]
    pub const fn uses_internal_matching(&self) -> bool {
        !matches!(
            self,
            Self::Files
                | Self::FilesInPath
                | Self::Shellcmd
                | Self::Directories
                | Self::DirsInCdpath
        )
    }
}

// =============================================================================
// XP Backslash Flags (from cmdexpand_defs.h)
// =============================================================================

/// Backslash handling modes for expansion patterns.
#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum XpBackslash {
    /// Nothing special for backslashes
    None = 0,
    /// Uses one backslash before a space
    One = 0x1,
    /// Uses three backslashes before a space
    Three = 0x2,
    /// Commas need to be escaped with a backslash
    Comma = 0x4,
}

impl XpBackslash {
    /// Check if flag is set in a bitmask.
    #[must_use]
    #[allow(clippy::cast_sign_loss)]
    pub const fn is_set(self, flags: c_int) -> bool {
        (flags as u32 & self as u32) != 0
    }
}

/// XP_BS_NONE constant
pub const XP_BS_NONE: c_int = 0;
/// XP_BS_ONE constant
pub const XP_BS_ONE: c_int = 0x1;
/// XP_BS_THREE constant
pub const XP_BS_THREE: c_int = 0x2;
/// XP_BS_COMMA constant
pub const XP_BS_COMMA: c_int = 0x4;

// =============================================================================
// XP Prefix Types (from cmdexpand_defs.h)
// =============================================================================

/// Prefix type for boolean option completion.
#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum XpPrefix {
    /// Prefix not used
    #[default]
    None = 0,
    /// "no" prefix for bool option
    No = 1,
    /// "inv" prefix for bool option
    Inv = 2,
}

impl XpPrefix {
    /// Convert from raw C integer.
    #[must_use]
    pub const fn from_raw(value: c_int) -> Option<Self> {
        match value {
            0 => Some(Self::None),
            1 => Some(Self::No),
            2 => Some(Self::Inv),
            _ => None,
        }
    }

    /// Convert to raw C integer.
    #[must_use]
    pub const fn to_raw(self) -> c_int {
        self as c_int
    }
}

// =============================================================================
// Wild Mode Constants (from cmdexpand.h)
// =============================================================================

/// Wild expansion mode values for `ExpandOne()` and `nextwild()`.
pub mod wild_mode {
    use std::os::raw::c_int;

    /// Free expansion memory
    pub const WILD_FREE: c_int = 1;
    /// Expand and free
    pub const WILD_EXPAND_FREE: c_int = 2;
    /// Expand and keep results
    pub const WILD_EXPAND_KEEP: c_int = 3;
    /// Get next match
    pub const WILD_NEXT: c_int = 4;
    /// Get previous match
    pub const WILD_PREV: c_int = 5;
    /// Return all matches
    pub const WILD_ALL: c_int = 6;
    /// Return longest common match
    pub const WILD_LONGEST: c_int = 7;
    /// Return all matches, keep order
    pub const WILD_ALL_KEEP: c_int = 8;
    /// Cancel expansion
    pub const WILD_CANCEL: c_int = 9;
    /// Apply current match
    pub const WILD_APPLY: c_int = 10;
    /// Page up in popup menu
    pub const WILD_PAGEUP: c_int = 11;
    /// Page down in popup menu
    pub const WILD_PAGEDOWN: c_int = 12;
    /// Select specific item in popup menu
    pub const WILD_PUM_WANT: c_int = 13;
}

/// Wild option flags for `ExpandOne()` and `nextwild()`.
pub mod wild_flags {
    use std::os::raw::c_int;

    /// List if pattern not found
    pub const WILD_LIST_NOTFOUND: c_int = 0x01;
    /// Replace home dir with ~
    pub const WILD_HOME_REPLACE: c_int = 0x02;
    /// Use NL as separator
    pub const WILD_USE_NL: c_int = 0x04;
    /// Don't beep on no match
    pub const WILD_NO_BEEP: c_int = 0x08;
    /// Add slash to directories
    pub const WILD_ADD_SLASH: c_int = 0x10;
    /// Keep all matches
    pub const WILD_KEEP_ALL: c_int = 0x20;
    /// Don't show messages
    pub const WILD_SILENT: c_int = 0x40;
    /// Escape special chars in results
    pub const WILD_ESCAPE: c_int = 0x80;
    /// Ignore case
    pub const WILD_ICASE: c_int = 0x100;
    /// Include all symlinks
    pub const WILD_ALLLINKS: c_int = 0x200;
    /// Ignore completeslash option
    pub const WILD_IGNORE_COMPLETESLASH: c_int = 0x400;
    /// Don't show errors
    pub const WILD_NOERROR: c_int = 0x800;
    /// Sort buffers by last used
    pub const WILD_BUFLASTUSED: c_int = 0x1000;
    /// Filter for diff buffers
    pub const BUF_DIFF_FILTER: c_int = 0x2000;
    /// Don't select first match
    pub const WILD_NOSELECT: c_int = 0x4000;
    /// May expand pattern
    pub const WILD_MAY_EXPAND_PATTERN: c_int = 0x8000;
    /// Called from wildtrigger()
    pub const WILD_FUNC_TRIGGER: c_int = 0x10000;
}

// =============================================================================
// Opaque Handle for expand_T
// =============================================================================

/// Opaque handle to C `expand_T` structure.
///
/// This provides a safe Rust interface to the C expansion context without
/// exposing the internal structure.
#[repr(transparent)]
#[derive(Debug, Clone, Copy)]
pub struct ExpandHandle(ptr::NonNull<()>);

impl ExpandHandle {
    /// Create a new handle from a raw pointer.
    ///
    /// # Safety
    ///
    /// The pointer must be a valid `expand_T*` from C code.
    #[must_use]
    pub unsafe fn from_raw(ptr: *mut ()) -> Option<Self> {
        ptr::NonNull::new(ptr).map(Self)
    }

    /// Get the raw pointer.
    #[must_use]
    pub const fn as_ptr(self) -> *mut () {
        self.0.as_ptr()
    }
}

// =============================================================================
// C Accessor Function Declarations
// =============================================================================

#[allow(dead_code)]
extern "C" {
    // Accessors for expand_T fields (to be added in cmdexpand.c)
    fn nvim_expand_get_context(xp: *const ()) -> c_int;
    fn nvim_expand_get_pattern(xp: *const ()) -> *const c_char;
    fn nvim_expand_get_pattern_len(xp: *const ()) -> usize;
    fn nvim_expand_get_backslash(xp: *const ()) -> c_int;
    fn nvim_expand_get_numfiles(xp: *const ()) -> c_int;
    fn nvim_expand_get_selected(xp: *const ()) -> c_int;
    fn nvim_expand_get_shell(xp: *const ()) -> c_int;

    // Setters
    fn nvim_expand_set_context(xp: *mut (), context: c_int);
    fn nvim_expand_set_backslash(xp: *mut (), backslash: c_int);
    fn nvim_expand_set_selected(xp: *mut (), selected: c_int);

    // Global state accessors
    fn nvim_get_wop_flags() -> c_uint;
}

// =============================================================================
// Safe Rust Wrappers for expand_T Access
// =============================================================================

/// Get the expansion context type from an expand_T handle.
///
/// # Safety
///
/// The handle must be valid.
#[must_use]
pub unsafe fn expand_get_context(xp: ExpandHandle) -> Option<ExpandContext> {
    let raw = nvim_expand_get_context(xp.as_ptr());
    ExpandContext::from_raw(raw)
}

/// Get the expansion pattern from an expand_T handle.
///
/// # Safety
///
/// The handle must be valid and the pattern must remain valid for the
/// lifetime of the returned slice.
#[must_use]
pub unsafe fn expand_get_pattern(xp: ExpandHandle) -> Option<&'static str> {
    let ptr = nvim_expand_get_pattern(xp.as_ptr());
    if ptr.is_null() {
        return None;
    }
    let len = nvim_expand_get_pattern_len(xp.as_ptr());
    let bytes = std::slice::from_raw_parts(ptr.cast::<u8>(), len);
    std::str::from_utf8(bytes).ok()
}

/// Get the backslash flags from an expand_T handle.
///
/// # Safety
///
/// The handle must be valid.
#[must_use]
pub unsafe fn expand_get_backslash(xp: ExpandHandle) -> c_int {
    nvim_expand_get_backslash(xp.as_ptr())
}

/// Get the number of files from an expand_T handle.
///
/// # Safety
///
/// The handle must be valid.
#[must_use]
pub unsafe fn expand_get_numfiles(xp: ExpandHandle) -> c_int {
    nvim_expand_get_numfiles(xp.as_ptr())
}

/// Get the selected index from an expand_T handle.
///
/// # Safety
///
/// The handle must be valid.
#[must_use]
pub unsafe fn expand_get_selected(xp: ExpandHandle) -> c_int {
    nvim_expand_get_selected(xp.as_ptr())
}

/// Set the expansion context type.
///
/// # Safety
///
/// The handle must be valid.
pub unsafe fn expand_set_context(xp: ExpandHandle, context: ExpandContext) {
    nvim_expand_set_context(xp.as_ptr(), context.to_raw());
}

/// Set the backslash flags.
///
/// # Safety
///
/// The handle must be valid.
pub unsafe fn expand_set_backslash(xp: ExpandHandle, backslash: c_int) {
    nvim_expand_set_backslash(xp.as_ptr(), backslash);
}

/// Set the selected index.
///
/// # Safety
///
/// The handle must be valid.
pub unsafe fn expand_set_selected(xp: ExpandHandle, selected: c_int) {
    nvim_expand_set_selected(xp.as_ptr(), selected);
}

// =============================================================================
// FFI Functions
// =============================================================================

/// Check if fuzzy completion is supported for the given expansion context.
///
/// # Safety
///
/// `xp` must be a valid pointer to an `expand_T` structure.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_expand_fuzzy_supported(xp: *const ()) -> c_int {
    if xp.is_null() {
        return 0;
    }

    let context_raw = nvim_expand_get_context(xp);
    let Some(context) = ExpandContext::from_raw(context_raw) else {
        return 0;
    };

    // Check if fuzzy is enabled in wildoptions
    let wop_flags = nvim_get_wop_flags();
    let fuzzy_enabled = (wop_flags & 0x01) != 0; // kOptWopFlagFuzzy

    c_int::from(context.supports_fuzzy() && fuzzy_enabled)
}

/// Direct C replacement for cmdline_expand_fuzzy_supported().
///
/// # Safety
///
/// `xp` must be a valid pointer to an `expand_T` structure.
#[must_use]
#[export_name = "cmdline_expand_fuzzy_supported"]
pub unsafe extern "C" fn cmdline_expand_fuzzy_supported_rs(xp: *const ()) -> bool {
    if xp.is_null() {
        return false;
    }
    let context_raw = nvim_expand_get_context(xp);
    let Some(context) = ExpandContext::from_raw(context_raw) else {
        return false;
    };
    let wop_flags = nvim_get_wop_flags();
    let fuzzy_enabled = (wop_flags & 0x01) != 0;
    context.supports_fuzzy() && fuzzy_enabled
}

/// Get the expansion context type as a raw integer.
///
/// # Safety
///
/// `xp` must be a valid pointer to an `expand_T` structure.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_expand_get_context(xp: *const ()) -> c_int {
    if xp.is_null() {
        return ExpandContext::Nothing.to_raw();
    }
    nvim_expand_get_context(xp)
}

/// Check if the context is for file-like expansion.
///
/// # Safety
///
/// `xp` must be a valid pointer to an `expand_T` structure.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_expand_is_file_context(xp: *const ()) -> c_int {
    if xp.is_null() {
        return 0;
    }

    let context_raw = nvim_expand_get_context(xp);
    let Some(context) = ExpandContext::from_raw(context_raw) else {
        return 0;
    };

    c_int::from(context.is_file_expansion())
}

/// Direct C replacement for cmdline_expand_is_file_context().
///
/// # Safety
///
/// `xp` must be a valid pointer to an `expand_T` structure.
#[must_use]
#[export_name = "cmdline_expand_is_file_context"]
pub unsafe extern "C" fn cmdline_expand_is_file_context_rs(xp: *const ()) -> bool {
    if xp.is_null() {
        return false;
    }
    let context_raw = nvim_expand_get_context(xp);
    let Some(context) = ExpandContext::from_raw(context_raw) else {
        return false;
    };
    context.is_file_expansion()
}

/// Check if the context uses internal pattern matching.
///
/// # Safety
///
/// `xp` must be a valid pointer to an `expand_T` structure.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_expand_uses_internal_matching(xp: *const ()) -> c_int {
    if xp.is_null() {
        return 1; // Default to internal matching
    }

    let context_raw = nvim_expand_get_context(xp);
    let Some(context) = ExpandContext::from_raw(context_raw) else {
        return 1;
    };

    c_int::from(context.uses_internal_matching())
}

/// Direct C replacement for cmdline_expand_uses_internal().
///
/// # Safety
///
/// `xp` must be a valid pointer to an `expand_T` structure.
#[must_use]
#[export_name = "cmdline_expand_uses_internal"]
pub unsafe extern "C" fn cmdline_expand_uses_internal_rs(xp: *const ()) -> bool {
    if xp.is_null() {
        return true; // Default to internal matching
    }
    let context_raw = nvim_expand_get_context(xp);
    let Some(context) = ExpandContext::from_raw(context_raw) else {
        return true;
    };
    context.uses_internal_matching()
}

// =============================================================================
// Expand Wildcards Flags (from path.h)
// =============================================================================

/// Flags for `expand_wildcards()`.
pub mod ew_flags {
    use std::os::raw::c_int;

    /// Include directory names
    pub const EW_DIR: c_int = 0x01;
    /// Include file names
    pub const EW_FILE: c_int = 0x02;
    /// Include not found names
    pub const EW_NOTFOUND: c_int = 0x04;
    /// Append slash to directory name
    pub const EW_ADDSLASH: c_int = 0x08;
    /// Keep all matches
    pub const EW_KEEPALL: c_int = 0x10;
    /// Don't print "1 returned" from shell
    pub const EW_SILENT: c_int = 0x20;
    /// Executable files
    pub const EW_EXEC: c_int = 0x40;
    /// Search in 'path' too
    pub const EW_PATH: c_int = 0x80;
    /// Ignore case
    pub const EW_ICASE: c_int = 0x100;
    /// No error for bad regexp
    pub const EW_NOERROR: c_int = 0x200;
    /// Add match with literal name if exists
    pub const EW_NOTWILD: c_int = 0x400;
    /// Do not escape $, $var is expanded
    pub const EW_KEEPDOLLAR: c_int = 0x800;
    /// Also links not pointing to existing file
    pub const EW_ALLLINKS: c_int = 0x1000;
    /// Called from expand_shellcmd()
    pub const EW_SHELLCMD: c_int = 0x2000;
    /// Also files starting with a dot
    pub const EW_DODOT: c_int = 0x4000;
    /// No matches is not an error
    pub const EW_EMPTYOK: c_int = 0x8000;
    /// Do not expand environment variables
    pub const EW_NOTENV: c_int = 0x10000;
    /// Search in 'cdpath' too
    pub const EW_CDPATH: c_int = 0x20000;
    /// Do not invoke breakcheck
    pub const EW_NOBREAK: c_int = 0x40000;
}

// =============================================================================
// Pure Utility Functions
// =============================================================================

/// Sort function comparator for completion matches.
///
/// `<SNR>` functions (script-local functions starting with '<') should be
/// sorted to the end.
///
/// Returns:
/// - -1 if s1 should come before s2
/// - 1 if s1 should come after s2
/// - 0 or strcmp result otherwise
#[inline]
#[must_use]
pub fn sort_func_compare(s1: &str, s2: &str) -> std::cmp::Ordering {
    let p1_starts_with_lt = s1.starts_with('<');
    let p2_starts_with_lt = s2.starts_with('<');

    match (p1_starts_with_lt, p2_starts_with_lt) {
        (false, true) => std::cmp::Ordering::Less,
        (true, false) => std::cmp::Ordering::Greater,
        _ => s1.cmp(s2),
    }
}

/// FFI version of sort_func_compare for qsort.
///
/// # Safety
///
/// `s1` and `s2` must be valid pointers to `char*` pointers (i.e., `char**`).
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_sort_func_compare(
    s1: *const *const c_char,
    s2: *const *const c_char,
) -> c_int {
    if s1.is_null() || s2.is_null() {
        return 0;
    }

    let p1 = *s1;
    let p2 = *s2;

    if p1.is_null() || p2.is_null() {
        return 0;
    }

    // Check first character for '<' (SNR function prefix)
    let c1 = *p1;
    let c2 = *p2;
    #[allow(clippy::cast_possible_wrap)]
    let lt = b'<' as c_char;

    if c1 != lt && c2 == lt {
        return -1;
    }
    if c1 == lt && c2 != lt {
        return 1;
    }

    // Fall back to strcmp
    libc::strcmp(p1, p2)
}

/// Map wild expand options to flags for `expand_wildcards()`.
///
/// Converts `WILD_*` option flags to `EW_*` flags used by the filesystem
/// expansion functions.
#[must_use]
#[allow(clippy::wildcard_imports)]
pub const fn map_wildopts_to_ewflags(options: c_int) -> c_int {
    use ew_flags::*;
    use wild_flags::*;

    let mut flags = EW_DIR; // Always include directories

    if (options & WILD_LIST_NOTFOUND) != 0 {
        flags |= EW_NOTFOUND;
    }
    if (options & WILD_ADD_SLASH) != 0 {
        flags |= EW_ADDSLASH;
    }
    if (options & WILD_KEEP_ALL) != 0 {
        flags |= EW_KEEPALL;
    }
    if (options & WILD_SILENT) != 0 {
        flags |= EW_SILENT;
    }
    if (options & WILD_NOERROR) != 0 {
        flags |= EW_NOERROR;
    }
    if (options & WILD_ALLLINKS) != 0 {
        flags |= EW_ALLLINKS;
    }

    flags
}

/// FFI version of map_wildopts_to_ewflags.
#[unsafe(no_mangle)]
pub const extern "C" fn rs_map_wildopts_to_ewflags(options: c_int) -> c_int {
    map_wildopts_to_ewflags(options)
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_expand_context_from_raw() {
        assert_eq!(
            ExpandContext::from_raw(-2),
            Some(ExpandContext::Unsuccessful)
        );
        assert_eq!(ExpandContext::from_raw(-1), Some(ExpandContext::Ok));
        assert_eq!(ExpandContext::from_raw(0), Some(ExpandContext::Nothing));
        assert_eq!(ExpandContext::from_raw(1), Some(ExpandContext::Commands));
        assert_eq!(ExpandContext::from_raw(2), Some(ExpandContext::Files));
        assert_eq!(ExpandContext::from_raw(63), Some(ExpandContext::Lua));
        assert_eq!(ExpandContext::from_raw(64), None);
        assert_eq!(ExpandContext::from_raw(100), None);
    }

    #[test]
    fn test_expand_context_to_raw() {
        assert_eq!(ExpandContext::Unsuccessful.to_raw(), -2);
        assert_eq!(ExpandContext::Ok.to_raw(), -1);
        assert_eq!(ExpandContext::Nothing.to_raw(), 0);
        assert_eq!(ExpandContext::Commands.to_raw(), 1);
        assert_eq!(ExpandContext::Lua.to_raw(), 63);
    }

    #[test]
    fn test_expand_context_roundtrip() {
        for i in -2..=63 {
            if let Some(ctx) = ExpandContext::from_raw(i) {
                assert_eq!(ctx.to_raw(), i);
            }
        }
    }

    #[test]
    fn test_supports_fuzzy() {
        // These should NOT support fuzzy
        assert!(!ExpandContext::Files.supports_fuzzy());
        assert!(!ExpandContext::Directories.supports_fuzzy());
        assert!(!ExpandContext::Help.supports_fuzzy());
        assert!(!ExpandContext::Colors.supports_fuzzy());
        assert!(!ExpandContext::Tags.supports_fuzzy());
        assert!(!ExpandContext::UserLua.supports_fuzzy());

        // These SHOULD support fuzzy
        assert!(ExpandContext::Commands.supports_fuzzy());
        assert!(ExpandContext::Buffers.supports_fuzzy());
        assert!(ExpandContext::Functions.supports_fuzzy());
        assert!(ExpandContext::Mappings.supports_fuzzy());
        assert!(ExpandContext::UserCommands.supports_fuzzy());
    }

    #[test]
    fn test_is_file_expansion() {
        assert!(ExpandContext::Files.is_file_expansion());
        assert!(ExpandContext::FilesInPath.is_file_expansion());
        assert!(ExpandContext::Directories.is_file_expansion());
        assert!(ExpandContext::Shellcmd.is_file_expansion());
        assert!(ExpandContext::Buffers.is_file_expansion());
        assert!(ExpandContext::DirsInCdpath.is_file_expansion());

        assert!(!ExpandContext::Commands.is_file_expansion());
        assert!(!ExpandContext::Functions.is_file_expansion());
        assert!(!ExpandContext::Help.is_file_expansion());
    }

    #[test]
    fn test_uses_internal_matching() {
        // File-like contexts use external matching
        assert!(!ExpandContext::Files.uses_internal_matching());
        assert!(!ExpandContext::Directories.uses_internal_matching());
        assert!(!ExpandContext::Shellcmd.uses_internal_matching());

        // Other contexts use internal matching
        assert!(ExpandContext::Commands.uses_internal_matching());
        assert!(ExpandContext::Buffers.uses_internal_matching());
        assert!(ExpandContext::Help.uses_internal_matching());
    }

    #[test]
    fn test_xp_backslash_is_set() {
        assert!(!XpBackslash::One.is_set(0));
        assert!(XpBackslash::One.is_set(0x1));
        assert!(XpBackslash::One.is_set(0x3));
        assert!(!XpBackslash::One.is_set(0x2));

        assert!(XpBackslash::Three.is_set(0x2));
        assert!(XpBackslash::Three.is_set(0x3));
        assert!(!XpBackslash::Three.is_set(0x1));

        assert!(XpBackslash::Comma.is_set(0x4));
        assert!(XpBackslash::Comma.is_set(0x5));
        assert!(!XpBackslash::Comma.is_set(0x3));
    }

    #[test]
    fn test_xp_prefix() {
        assert_eq!(XpPrefix::from_raw(0), Some(XpPrefix::None));
        assert_eq!(XpPrefix::from_raw(1), Some(XpPrefix::No));
        assert_eq!(XpPrefix::from_raw(2), Some(XpPrefix::Inv));
        assert_eq!(XpPrefix::from_raw(3), None);

        assert_eq!(XpPrefix::None.to_raw(), 0);
        assert_eq!(XpPrefix::No.to_raw(), 1);
        assert_eq!(XpPrefix::Inv.to_raw(), 2);
    }

    #[test]
    fn test_wild_mode_constants() {
        use wild_mode::*;
        assert_eq!(WILD_FREE, 1);
        assert_eq!(WILD_EXPAND_FREE, 2);
        assert_eq!(WILD_EXPAND_KEEP, 3);
        assert_eq!(WILD_NEXT, 4);
        assert_eq!(WILD_PREV, 5);
        assert_eq!(WILD_ALL, 6);
        assert_eq!(WILD_LONGEST, 7);
        assert_eq!(WILD_ALL_KEEP, 8);
        assert_eq!(WILD_CANCEL, 9);
        assert_eq!(WILD_APPLY, 10);
        assert_eq!(WILD_PAGEUP, 11);
        assert_eq!(WILD_PAGEDOWN, 12);
        assert_eq!(WILD_PUM_WANT, 13);
    }

    #[test]
    fn test_wild_flags_constants() {
        use wild_flags::*;
        assert_eq!(WILD_LIST_NOTFOUND, 0x01);
        assert_eq!(WILD_HOME_REPLACE, 0x02);
        assert_eq!(WILD_USE_NL, 0x04);
        assert_eq!(WILD_NO_BEEP, 0x08);
        assert_eq!(WILD_ADD_SLASH, 0x10);
        assert_eq!(WILD_KEEP_ALL, 0x20);
        assert_eq!(WILD_SILENT, 0x40);
        assert_eq!(WILD_ESCAPE, 0x80);
        assert_eq!(WILD_ICASE, 0x100);
        assert_eq!(WILD_ALLLINKS, 0x200);
        assert_eq!(WILD_IGNORE_COMPLETESLASH, 0x400);
        assert_eq!(WILD_NOERROR, 0x800);
        assert_eq!(WILD_BUFLASTUSED, 0x1000);
        assert_eq!(BUF_DIFF_FILTER, 0x2000);
        assert_eq!(WILD_NOSELECT, 0x4000);
        assert_eq!(WILD_MAY_EXPAND_PATTERN, 0x8000);
        assert_eq!(WILD_FUNC_TRIGGER, 0x10000);
    }

    #[test]
    fn test_xp_bs_constants() {
        assert_eq!(XP_BS_NONE, 0);
        assert_eq!(XP_BS_ONE, 0x1);
        assert_eq!(XP_BS_THREE, 0x2);
        assert_eq!(XP_BS_COMMA, 0x4);
    }

    #[test]
    fn test_ew_flags_constants() {
        use ew_flags::*;
        assert_eq!(EW_DIR, 0x01);
        assert_eq!(EW_FILE, 0x02);
        assert_eq!(EW_NOTFOUND, 0x04);
        assert_eq!(EW_ADDSLASH, 0x08);
        assert_eq!(EW_KEEPALL, 0x10);
        assert_eq!(EW_SILENT, 0x20);
        assert_eq!(EW_EXEC, 0x40);
        assert_eq!(EW_PATH, 0x80);
        assert_eq!(EW_ICASE, 0x100);
        assert_eq!(EW_NOERROR, 0x200);
        assert_eq!(EW_NOTWILD, 0x400);
        assert_eq!(EW_KEEPDOLLAR, 0x800);
        assert_eq!(EW_ALLLINKS, 0x1000);
        assert_eq!(EW_SHELLCMD, 0x2000);
        assert_eq!(EW_DODOT, 0x4000);
        assert_eq!(EW_EMPTYOK, 0x8000);
        assert_eq!(EW_NOTENV, 0x10000);
        assert_eq!(EW_CDPATH, 0x20000);
        assert_eq!(EW_NOBREAK, 0x40000);
    }

    #[test]
    fn test_sort_func_compare() {
        use std::cmp::Ordering;

        // Regular strings sort normally
        assert_eq!(sort_func_compare("abc", "def"), Ordering::Less);
        assert_eq!(sort_func_compare("def", "abc"), Ordering::Greater);
        assert_eq!(sort_func_compare("abc", "abc"), Ordering::Equal);

        // <SNR> functions (starting with '<') sort to the end
        assert_eq!(sort_func_compare("foo", "<SNR>bar"), Ordering::Less);
        assert_eq!(sort_func_compare("<SNR>foo", "bar"), Ordering::Greater);
        assert_eq!(sort_func_compare("<SNR>foo", "<SNR>bar"), Ordering::Greater); // SNRfoo > SNRbar

        // Two <SNR> functions sort alphabetically
        assert_eq!(
            sort_func_compare("<SNR>123_abc", "<SNR>456_def"),
            Ordering::Less
        );
    }

    #[test]
    fn test_map_wildopts_to_ewflags_empty() {
        // No options should give just EW_DIR
        let flags = map_wildopts_to_ewflags(0);
        assert_eq!(flags, ew_flags::EW_DIR);
    }

    #[test]
    fn test_map_wildopts_to_ewflags_single() {
        use ew_flags::*;
        use wild_flags::*;

        // Test each flag mapping individually
        assert_eq!(
            map_wildopts_to_ewflags(WILD_LIST_NOTFOUND),
            EW_DIR | EW_NOTFOUND
        );
        assert_eq!(
            map_wildopts_to_ewflags(WILD_ADD_SLASH),
            EW_DIR | EW_ADDSLASH
        );
        assert_eq!(map_wildopts_to_ewflags(WILD_KEEP_ALL), EW_DIR | EW_KEEPALL);
        assert_eq!(map_wildopts_to_ewflags(WILD_SILENT), EW_DIR | EW_SILENT);
        assert_eq!(map_wildopts_to_ewflags(WILD_NOERROR), EW_DIR | EW_NOERROR);
        assert_eq!(map_wildopts_to_ewflags(WILD_ALLLINKS), EW_DIR | EW_ALLLINKS);
    }

    #[test]
    fn test_map_wildopts_to_ewflags_combined() {
        use ew_flags::*;
        use wild_flags::*;

        // Test multiple flags combined
        let options = WILD_LIST_NOTFOUND | WILD_ADD_SLASH | WILD_SILENT;
        let expected = EW_DIR | EW_NOTFOUND | EW_ADDSLASH | EW_SILENT;
        assert_eq!(map_wildopts_to_ewflags(options), expected);

        // All flags
        let all_options = WILD_LIST_NOTFOUND
            | WILD_ADD_SLASH
            | WILD_KEEP_ALL
            | WILD_SILENT
            | WILD_NOERROR
            | WILD_ALLLINKS;
        let all_expected =
            EW_DIR | EW_NOTFOUND | EW_ADDSLASH | EW_KEEPALL | EW_SILENT | EW_NOERROR | EW_ALLLINKS;
        assert_eq!(map_wildopts_to_ewflags(all_options), all_expected);
    }

    #[test]
    fn test_map_wildopts_ignores_unmapped_flags() {
        use wild_flags::*;

        // Flags that don't map to EW_* should be ignored
        let options = WILD_HOME_REPLACE | WILD_USE_NL | WILD_ESCAPE | WILD_ICASE;
        let flags = map_wildopts_to_ewflags(options);
        // Should only have EW_DIR
        assert_eq!(flags, ew_flags::EW_DIR);
    }
}
