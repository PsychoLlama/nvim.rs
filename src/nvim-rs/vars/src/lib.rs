//! VimL variable storage and scope management for Neovim
//!
//! This crate provides Rust implementations of variable-related functions
//! from `src/nvim/eval/vars.c`. It implements the v: scope variables,
//! variable flavour detection, and scope management utilities.
//!
//! ## Phase 28.2: Variable Storage System
//!
//! This module provides:
//! - `VimVarIndex` enum for all v: variables
//! - `VimVarFlags` for variable properties (read-only, compat, sandbox)
//! - Scope type definitions (global, local, script, etc.)
//! - Variable flavour detection for ShaDa persistence
//! - FFI exports for variable access
//!
//! ## Architecture
//!
//! VimL has multiple variable scopes:
//! - `g:` - Global variables
//! - `v:` - Vim predefined variables
//! - `b:` - Buffer-local variables
//! - `w:` - Window-local variables
//! - `t:` - Tab-local variables
//! - `l:` - Function-local variables
//! - `s:` - Script-local variables
//! - `a:` - Function argument variables

#![allow(unsafe_code)] // FFI requires unsafe
#![allow(clippy::doc_markdown)]
#![allow(clippy::missing_safety_doc)]
#![allow(clippy::missing_const_for_fn)]
#![allow(clippy::cast_possible_wrap)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::must_use_candidate)]
#![allow(clippy::too_many_lines)]

use std::ffi::{c_char, c_int};

// Submodules
pub mod checks;
pub mod eval_helpers;
pub mod lookup;
pub mod option_conv;
pub mod vimvar_accessors;

// Re-export typval types for convenience
pub use nvim_typval::{VarLockStatus, VarType};

// Re-export lookup types and functions
pub use lookup::{
    rs_find_var_ht, rs_get_var_value, rs_parse_scope_prefix, rs_skip_scope_prefix, DictHandle,
    DictitemHandle, HashtabHandle, ScopePrefix,
};

// =============================================================================
// VimVar Index enum (matching C's VimVarIndex in eval_defs.h)
// =============================================================================

/// Index values for v: variables.
///
/// These match the C enum `VimVarIndex` in `eval_defs.h`.
#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum VimVarIndex {
    Count = 0,
    Count1 = 1,
    Prevcount = 2,
    Errmsg = 3,
    Warningmsg = 4,
    Statusmsg = 5,
    ShellError = 6,
    ThisSession = 7,
    Version = 8,
    Lnum = 9,
    Termrequest = 10,
    Termresponse = 11,
    Fname = 12,
    Lang = 13,
    LcTime = 14,
    Ctype = 15,
    CcFrom = 16,
    CcTo = 17,
    FnameIn = 18,
    FnameOut = 19,
    FnameNew = 20,
    FnameDiff = 21,
    Cmdarg = 22,
    Foldstart = 23,
    Foldend = 24,
    Folddashes = 25,
    Foldlevel = 26,
    Progname = 27,
    SendServer = 28,
    Dying = 29,
    Exception = 30,
    Throwpoint = 31,
    Reg = 32,
    Cmdbang = 33,
    Insertmode = 34,
    Val = 35,
    Key = 36,
    Profiling = 37,
    FcsReason = 38,
    FcsChoice = 39,
    BevalBufnr = 40,
    BevalWinnr = 41,
    BevalWinid = 42,
    BevalLnum = 43,
    BevalCol = 44,
    BevalText = 45,
    Scrollstart = 46,
    Swapname = 47,
    Swapchoice = 48,
    Swapcommand = 49,
    Char = 50,
    MouseWin = 51,
    MouseWinid = 52,
    MouseLnum = 53,
    MouseCol = 54,
    Op = 55,
    Searchforward = 56,
    Hlsearch = 57,
    Oldfiles = 58,
    Windowid = 59,
    Progpath = 60,
    CompletedItem = 61,
    OptionNew = 62,
    OptionOld = 63,
    OptionOldlocal = 64,
    OptionOldglobal = 65,
    OptionCommand = 66,
    OptionType = 67,
    Errors = 68,
    False = 69,
    True = 70,
    Null = 71,
    Numbermax = 72,
    Numbermin = 73,
    Numbersize = 74,
    VimDidEnter = 75,
    Testing = 76,
    TypeNumber = 77,
    TypeString = 78,
    TypeFunc = 79,
    TypeList = 80,
    TypeDict = 81,
    TypeFloat = 82,
    TypeBool = 83,
    TypeBlob = 84,
    Event = 85,
    Versionlong = 86,
    Echospace = 87,
    Argv = 88,
    Collate = 89,
    Exiting = 90,
    Maxcol = 91,
    Stacktrace = 92,
    VimDidInit = 93,
    // Neovim-specific
    Stderr = 94,
    MsgpackTypes = 95,
    NullString = 96,
    NullList = 97,
    NullDict = 98,
    NullBlob = 99,
    Lua = 100,
    Relnum = 101,
    Virtnum = 102,
}

impl VimVarIndex {
    /// Get the number of defined v: variables.
    pub const COUNT: usize = 103;

    /// Convert from C integer.
    #[inline]
    pub const fn from_c_int(v: c_int) -> Option<Self> {
        if v >= 0 && (v as usize) < Self::COUNT {
            // Safe because we checked bounds and repr(i32)
            Some(unsafe { std::mem::transmute::<i32, Self>(v) })
        } else {
            None
        }
    }

    /// Convert to C integer.
    #[inline]
    pub const fn as_c_int(self) -> c_int {
        self as c_int
    }

    /// Get the variable name (without "v:" prefix).
    #[inline]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Count => "count",
            Self::Count1 => "count1",
            Self::Prevcount => "prevcount",
            Self::Errmsg => "errmsg",
            Self::Warningmsg => "warningmsg",
            Self::Statusmsg => "statusmsg",
            Self::ShellError => "shell_error",
            Self::ThisSession => "this_session",
            Self::Version => "version",
            Self::Lnum => "lnum",
            Self::Termrequest => "termrequest",
            Self::Termresponse => "termresponse",
            Self::Fname => "fname",
            Self::Lang => "lang",
            Self::LcTime => "lc_time",
            Self::Ctype => "ctype",
            Self::CcFrom => "charconvert_from",
            Self::CcTo => "charconvert_to",
            Self::FnameIn => "fname_in",
            Self::FnameOut => "fname_out",
            Self::FnameNew => "fname_new",
            Self::FnameDiff => "fname_diff",
            Self::Cmdarg => "cmdarg",
            Self::Foldstart => "foldstart",
            Self::Foldend => "foldend",
            Self::Folddashes => "folddashes",
            Self::Foldlevel => "foldlevel",
            Self::Progname => "progname",
            Self::SendServer => "servername",
            Self::Dying => "dying",
            Self::Exception => "exception",
            Self::Throwpoint => "throwpoint",
            Self::Reg => "register",
            Self::Cmdbang => "cmdbang",
            Self::Insertmode => "insertmode",
            Self::Val => "val",
            Self::Key => "key",
            Self::Profiling => "profiling",
            Self::FcsReason => "fcs_reason",
            Self::FcsChoice => "fcs_choice",
            Self::BevalBufnr => "beval_bufnr",
            Self::BevalWinnr => "beval_winnr",
            Self::BevalWinid => "beval_winid",
            Self::BevalLnum => "beval_lnum",
            Self::BevalCol => "beval_col",
            Self::BevalText => "beval_text",
            Self::Scrollstart => "scrollstart",
            Self::Swapname => "swapname",
            Self::Swapchoice => "swapchoice",
            Self::Swapcommand => "swapcommand",
            Self::Char => "char",
            Self::MouseWin => "mouse_win",
            Self::MouseWinid => "mouse_winid",
            Self::MouseLnum => "mouse_lnum",
            Self::MouseCol => "mouse_col",
            Self::Op => "operator",
            Self::Searchforward => "searchforward",
            Self::Hlsearch => "hlsearch",
            Self::Oldfiles => "oldfiles",
            Self::Windowid => "windowid",
            Self::Progpath => "progpath",
            Self::CompletedItem => "completed_item",
            Self::OptionNew => "option_new",
            Self::OptionOld => "option_old",
            Self::OptionOldlocal => "option_oldlocal",
            Self::OptionOldglobal => "option_oldglobal",
            Self::OptionCommand => "option_command",
            Self::OptionType => "option_type",
            Self::Errors => "errors",
            Self::False => "false",
            Self::True => "true",
            Self::Null => "null",
            Self::Numbermax => "numbermax",
            Self::Numbermin => "numbermin",
            Self::Numbersize => "numbersize",
            Self::VimDidEnter => "vim_did_enter",
            Self::Testing => "testing",
            Self::TypeNumber => "t_number",
            Self::TypeString => "t_string",
            Self::TypeFunc => "t_func",
            Self::TypeList => "t_list",
            Self::TypeDict => "t_dict",
            Self::TypeFloat => "t_float",
            Self::TypeBool => "t_bool",
            Self::TypeBlob => "t_blob",
            Self::Event => "event",
            Self::Versionlong => "versionlong",
            Self::Echospace => "echospace",
            Self::Argv => "argv",
            Self::Collate => "collate",
            Self::Exiting => "exiting",
            Self::Maxcol => "maxcol",
            Self::Stacktrace => "stacktrace",
            Self::VimDidInit => "vim_did_init",
            Self::Stderr => "stderr",
            Self::MsgpackTypes => "msgpack_types",
            Self::NullString => "_null_string",
            Self::NullList => "_null_list",
            Self::NullDict => "_null_dict",
            Self::NullBlob => "_null_blob",
            Self::Lua => "lua",
            Self::Relnum => "relnum",
            Self::Virtnum => "virtnum",
        }
    }
}

// =============================================================================
// VimVar Flags (matching C's VV_* flags)
// =============================================================================

bitflags::bitflags! {
    /// Flags for v: variable properties.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct VimVarFlags: u8 {
        /// Compatible - also available without "v:" prefix
        const COMPAT = 0b0000_0001;
        /// Read-only
        const RO = 0b0000_0010;
        /// Read-only in sandbox
        const RO_SBX = 0b0000_0100;
    }
}

impl VimVarFlags {
    /// Check if the variable is read-only.
    #[inline]
    pub const fn is_read_only(self) -> bool {
        self.bits() & Self::RO.bits() != 0
    }

    /// Check if the variable is read-only in sandbox mode.
    #[inline]
    pub const fn is_read_only_sandbox(self) -> bool {
        self.bits() & Self::RO_SBX.bits() != 0
    }

    /// Check if the variable is compatible (accessible without v: prefix).
    #[inline]
    pub const fn is_compat(self) -> bool {
        self.bits() & Self::COMPAT.bits() != 0
    }
}

// =============================================================================
// Scope Types (matching C's ScopeType in typval_defs.h)
// =============================================================================

/// Variable scope type.
///
/// Matches C's `ScopeType` enum in `typval_defs.h`.
#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ScopeType {
    /// Not a scope dictionary.
    NoScope = 0,
    /// Scope dictionary which requires prefix (a:, v:, etc.).
    Scope = 1,
    /// Scope dictionary which may be accessed without prefix (l:, g:).
    DefScope = 2,
}

impl ScopeType {
    /// Convert from C integer.
    #[inline]
    pub const fn from_c_int(v: c_int) -> Option<Self> {
        match v {
            0 => Some(Self::NoScope),
            1 => Some(Self::Scope),
            2 => Some(Self::DefScope),
            _ => None,
        }
    }

    /// Check if this is a scope dictionary.
    #[inline]
    pub const fn is_scope(self) -> bool {
        !matches!(self, Self::NoScope)
    }

    /// Check if this scope can be accessed without prefix.
    #[inline]
    pub const fn can_access_without_prefix(self) -> bool {
        matches!(self, Self::DefScope)
    }
}

// =============================================================================
// Variable Flavour (for ShaDa persistence)
// =============================================================================

/// Variable flavour for ShaDa persistence.
///
/// Matches C's `var_flavour_T` enum.
#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum VarFlavour {
    /// Variable doesn't start with uppercase
    Default = 1,
    /// Variable starts with uppercase, has some lowercase
    Session = 2,
    /// Variable is all uppercase
    Shada = 4,
}

impl VarFlavour {
    /// Determine the flavour of a variable name.
    ///
    /// - All uppercase (e.g., "FOO") -> Shada
    /// - Starts with uppercase but has lowercase (e.g., "Foo") -> Session
    /// - Starts with lowercase -> Default
    #[inline]
    pub fn from_name(name: &str) -> Self {
        let bytes = name.as_bytes();
        if bytes.is_empty() {
            return Self::Default;
        }

        let first = bytes[0];
        if !first.is_ascii_uppercase() {
            return Self::Default;
        }

        // Check if any lowercase letter exists
        for &c in &bytes[1..] {
            if c.is_ascii_lowercase() {
                return Self::Session;
            }
        }

        Self::Shada
    }
}

/// Determine the flavour of a variable name for ShaDa persistence.
///
/// # Safety
/// `varname` must be a valid null-terminated C string.
#[no_mangle]
pub unsafe extern "C" fn rs_vars_var_flavour(varname: *const c_char) -> c_int {
    if varname.is_null() {
        return VarFlavour::Default as c_int;
    }

    let mut p = varname;

    // Check first character - must be uppercase to be Session or Shada
    let first = *p as u8;
    if !first.is_ascii_uppercase() {
        return VarFlavour::Default as c_int;
    }

    // Move to next character and check all remaining
    p = p.add(1);
    loop {
        let c = *p as u8;
        if c == 0 {
            break;
        }
        // If any lowercase letter found, it's Session flavour
        if c.is_ascii_lowercase() {
            return VarFlavour::Session as c_int;
        }
        p = p.add(1);
    }

    // All uppercase -> Shada flavour
    VarFlavour::Shada as c_int
}

// =============================================================================
// Variable name validation
// =============================================================================

/// Check if a character can be used in a variable or function name.
/// Does not include '{' or '}' for magic braces.
///
/// Valid characters: alphanumeric, underscore, colon, or autoload char (#).
#[inline]
pub const fn is_name_char(c: u8) -> bool {
    c.is_ascii_alphanumeric() || c == b'_' || c == b':' || c == b'#'
}

/// Check if a character can be the first character of a variable or function name.
///
/// Valid first characters: alphabetic or underscore.
#[inline]
pub const fn is_name_first_char(c: u8) -> bool {
    c.is_ascii_alphabetic() || c == b'_'
}

/// Check if a character can be used in a dictionary key.
///
/// Valid characters: alphanumeric or underscore.
#[inline]
pub const fn is_dict_key_char(c: u8) -> bool {
    c.is_ascii_alphanumeric() || c == b'_'
}

/// FFI: Check if a character can be used in a variable name.
#[no_mangle]
pub extern "C" fn rs_vars_is_name_char(c: c_int) -> bool {
    let Ok(c) = u8::try_from(c) else {
        return false;
    };
    is_name_char(c)
}

/// FFI: Check if a character can be the first character of a variable name.
#[no_mangle]
pub extern "C" fn rs_vars_is_name_first_char(c: c_int) -> bool {
    let Ok(c) = u8::try_from(c) else {
        return false;
    };
    is_name_first_char(c)
}

/// FFI: Check if a character can be used in a dictionary key.
#[no_mangle]
pub extern "C" fn rs_vars_is_dict_key_char(c: c_int) -> bool {
    let Ok(c) = u8::try_from(c) else {
        return false;
    };
    is_dict_key_char(c)
}

// =============================================================================
// Dict Item Flags (matching C's DictItemFlags)
// =============================================================================

bitflags::bitflags! {
    /// Flags for dictionary items (dictitem_T.di_flags).
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct DictItemFlags: u8 {
        /// Read-only value
        const RO = 1;
        /// Read-only in sandbox
        const RO_SBX = 2;
        /// Fixed value: cannot be :unlet or remove()d
        const FIX = 4;
        /// Locked value
        const LOCK = 8;
        /// Separately allocated
        const ALLOC = 16;
    }
}

impl DictItemFlags {
    /// Check if the item is read-only.
    #[inline]
    pub const fn is_read_only(self) -> bool {
        self.bits() & Self::RO.bits() != 0
    }

    /// Check if the item is read-only in sandbox.
    #[inline]
    pub const fn is_read_only_sandbox(self) -> bool {
        self.bits() & Self::RO_SBX.bits() != 0
    }

    /// Check if the item is fixed (cannot be removed).
    #[inline]
    pub const fn is_fixed(self) -> bool {
        self.bits() & Self::FIX.bits() != 0
    }

    /// Check if the item is locked.
    #[inline]
    pub const fn is_locked(self) -> bool {
        self.bits() & Self::LOCK.bits() != 0
    }

    /// Check if the item was separately allocated.
    #[inline]
    pub const fn is_allocated(self) -> bool {
        self.bits() & Self::ALLOC.bits() != 0
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vimvar_index() {
        assert_eq!(VimVarIndex::Count as i32, 0);
        assert_eq!(VimVarIndex::Version as i32, 8);
        assert_eq!(VimVarIndex::Virtnum as i32, 102);

        assert_eq!(VimVarIndex::from_c_int(0), Some(VimVarIndex::Count));
        assert_eq!(VimVarIndex::from_c_int(102), Some(VimVarIndex::Virtnum));
        assert_eq!(VimVarIndex::from_c_int(-1), None);
        assert_eq!(VimVarIndex::from_c_int(200), None);
    }

    #[test]
    fn test_vimvar_names() {
        assert_eq!(VimVarIndex::Count.name(), "count");
        assert_eq!(VimVarIndex::Version.name(), "version");
        assert_eq!(VimVarIndex::CcFrom.name(), "charconvert_from");
        assert_eq!(VimVarIndex::SendServer.name(), "servername");
        assert_eq!(VimVarIndex::Reg.name(), "register");
        assert_eq!(VimVarIndex::Op.name(), "operator");
    }

    #[test]
    fn test_vimvar_flags() {
        let ro = VimVarFlags::RO;
        assert!(ro.is_read_only());
        assert!(!ro.is_read_only_sandbox());
        assert!(!ro.is_compat());

        let compat_ro = VimVarFlags::COMPAT | VimVarFlags::RO;
        assert!(compat_ro.is_read_only());
        assert!(compat_ro.is_compat());

        let sandbox = VimVarFlags::RO_SBX;
        assert!(!sandbox.is_read_only());
        assert!(sandbox.is_read_only_sandbox());
    }

    #[test]
    fn test_scope_type() {
        assert_eq!(ScopeType::from_c_int(0), Some(ScopeType::NoScope));
        assert_eq!(ScopeType::from_c_int(1), Some(ScopeType::Scope));
        assert_eq!(ScopeType::from_c_int(2), Some(ScopeType::DefScope));
        assert_eq!(ScopeType::from_c_int(99), None);

        assert!(!ScopeType::NoScope.is_scope());
        assert!(ScopeType::Scope.is_scope());
        assert!(ScopeType::DefScope.is_scope());

        assert!(!ScopeType::NoScope.can_access_without_prefix());
        assert!(!ScopeType::Scope.can_access_without_prefix());
        assert!(ScopeType::DefScope.can_access_without_prefix());
    }

    #[test]
    fn test_var_flavour() {
        // All uppercase -> Shada
        assert_eq!(VarFlavour::from_name("FOO"), VarFlavour::Shada);
        assert_eq!(VarFlavour::from_name("X"), VarFlavour::Shada);
        assert_eq!(VarFlavour::from_name("FOO123"), VarFlavour::Shada);
        assert_eq!(VarFlavour::from_name("FOO_BAR"), VarFlavour::Shada);

        // Mixed case -> Session
        assert_eq!(VarFlavour::from_name("Foo"), VarFlavour::Session);
        assert_eq!(VarFlavour::from_name("FooBar"), VarFlavour::Session);
        assert_eq!(VarFlavour::from_name("FOo"), VarFlavour::Session);

        // Starts lowercase -> Default
        assert_eq!(VarFlavour::from_name("foo"), VarFlavour::Default);
        assert_eq!(VarFlavour::from_name("fooBar"), VarFlavour::Default);
        assert_eq!(VarFlavour::from_name("_foo"), VarFlavour::Default);
        assert_eq!(VarFlavour::from_name("123foo"), VarFlavour::Default);
        assert_eq!(VarFlavour::from_name(""), VarFlavour::Default);
    }

    #[test]
    fn test_name_char_validation() {
        // Valid name characters
        assert!(is_name_char(b'a'));
        assert!(is_name_char(b'Z'));
        assert!(is_name_char(b'0'));
        assert!(is_name_char(b'_'));
        assert!(is_name_char(b':'));
        assert!(is_name_char(b'#'));

        // Invalid name characters
        assert!(!is_name_char(b'{'));
        assert!(!is_name_char(b'}'));
        assert!(!is_name_char(b' '));
        assert!(!is_name_char(b'.'));
    }

    #[test]
    fn test_name_first_char_validation() {
        // Valid first characters
        assert!(is_name_first_char(b'a'));
        assert!(is_name_first_char(b'Z'));
        assert!(is_name_first_char(b'_'));

        // Invalid first characters
        assert!(!is_name_first_char(b'0'));
        assert!(!is_name_first_char(b':'));
        assert!(!is_name_first_char(b'#'));
    }

    #[test]
    fn test_dict_key_char_validation() {
        // Valid dict key characters
        assert!(is_dict_key_char(b'a'));
        assert!(is_dict_key_char(b'Z'));
        assert!(is_dict_key_char(b'0'));
        assert!(is_dict_key_char(b'_'));

        // Invalid dict key characters
        assert!(!is_dict_key_char(b':'));
        assert!(!is_dict_key_char(b'#'));
        assert!(!is_dict_key_char(b' '));
    }

    #[test]
    fn test_dict_item_flags() {
        let ro = DictItemFlags::RO;
        assert!(ro.is_read_only());
        assert!(!ro.is_fixed());

        let fix = DictItemFlags::FIX;
        assert!(fix.is_fixed());
        assert!(!fix.is_read_only());

        let combined = DictItemFlags::RO | DictItemFlags::FIX | DictItemFlags::LOCK;
        assert!(combined.is_read_only());
        assert!(combined.is_fixed());
        assert!(combined.is_locked());
        assert!(!combined.is_allocated());
    }
}
