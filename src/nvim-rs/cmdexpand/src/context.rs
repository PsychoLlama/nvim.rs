//! Expansion context types and constants for command-line completion.

// =============================================================================
// Constants - Expansion context types (mirrors cmdexpand_defs.h)
// =============================================================================

/// Expansion context type constants
#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExpandContext {
    Unsuccessful = -2,
    Ok = -1,
    Nothing = 0,
    Commands = 1,
    Files = 2,
    Directories = 3,
    Settings = 4,
    BoolSettings = 5,
    Tags = 6,
    OldSetting = 7,
    Help = 8,
    Buffers = 9,
    Events = 10,
    Menus = 11,
    Syntax = 12,
    Highlight = 13,
    Augroup = 14,
    UserVars = 15,
    Mappings = 16,
    TagsListfiles = 17,
    Functions = 18,
    UserFunc = 19,
    Expression = 20,
    Menunames = 21,
    UserCommands = 22,
    UserCmdFlags = 23,
    UserNargs = 24,
    UserComplete = 25,
    EnvVars = 26,
    Language = 27,
    Colors = 28,
    Compiler = 29,
    UserDefined = 30,
    UserList = 31,
    UserLua = 32,
    Shellcmd = 33,
    Sign = 34,
    Profile = 35,
    Filetype = 36,
    FilesInPath = 37,
    Ownsyntax = 38,
    Locales = 39,
    History = 40,
    User = 41,
    Syntime = 42,
    UserAddrType = 43,
    Packadd = 44,
    Messages = 45,
    Mapclear = 46,
    Arglist = 47,
    DiffBuffers = 48,
    Breakpoint = 49,
    Scriptnames = 50,
    Runtime = 51,
    StringSetting = 52,
    SettingSubtract = 53,
    Argopt = 54,
    Keymap = 55,
    DirsInCdpath = 56,
    Shellcmdline = 57,
    Findfunc = 58,
    Filetypecmd = 59,
    PatternInBuf = 60,
    Retab = 61,
    Checkhealth = 62,
    Lua = 63,
}

impl ExpandContext {
    /// Convert from raw i32 value
    #[must_use]
    pub const fn from_raw(value: i32) -> Option<Self> {
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

    /// Convert to raw i32 value
    #[must_use]
    pub const fn to_raw(self) -> i32 {
        self as i32
    }
}

// =============================================================================
// Constants - Backslash handling flags
// =============================================================================

/// Backslash handling flags for expansion
pub mod backslash {
    /// Nothing special for backslashes
    pub const XP_BS_NONE: i32 = 0;
    /// Uses one backslash before a space
    pub const XP_BS_ONE: i32 = 0x1;
    /// Uses three backslashes before a space
    pub const XP_BS_THREE: i32 = 0x2;
    /// Commas need to be escaped with a backslash
    pub const XP_BS_COMMA: i32 = 0x4;
}

// =============================================================================
// Constants - Wild mode types
// =============================================================================

/// Wild expansion mode constants
pub mod wild_mode {
    pub const WILD_FREE: i32 = 1;
    pub const WILD_EXPAND_FREE: i32 = 2;
    pub const WILD_EXPAND_KEEP: i32 = 3;
    pub const WILD_NEXT: i32 = 4;
    pub const WILD_PREV: i32 = 5;
    pub const WILD_ALL: i32 = 6;
    pub const WILD_LONGEST: i32 = 7;
    pub const WILD_ALL_KEEP: i32 = 8;
    pub const WILD_CANCEL: i32 = 9;
    pub const WILD_APPLY: i32 = 10;
    pub const WILD_PAGEUP: i32 = 11;
    pub const WILD_PAGEDOWN: i32 = 12;
    pub const WILD_PUM_WANT: i32 = 13;
}

// =============================================================================
// Constants - Wild option flags
// =============================================================================

/// Wild expansion option flags
pub mod wild_options {
    pub const WILD_LIST_NOTFOUND: i32 = 0x01;
    pub const WILD_HOME_REPLACE: i32 = 0x02;
    pub const WILD_USE_NL: i32 = 0x04;
    pub const WILD_NO_BEEP: i32 = 0x08;
    pub const WILD_ADD_SLASH: i32 = 0x10;
    pub const WILD_KEEP_ALL: i32 = 0x20;
    pub const WILD_SILENT: i32 = 0x40;
    pub const WILD_ESCAPE: i32 = 0x80;
    pub const WILD_ICASE: i32 = 0x100;
    pub const WILD_ALLLINKS: i32 = 0x200;
    pub const WILD_IGNORE_COMPLETESLASH: i32 = 0x400;
    pub const WILD_NOERROR: i32 = 0x800;
    pub const WILD_BUFLASTUSED: i32 = 0x1000;
    pub const BUF_DIFF_FILTER: i32 = 0x2000;
    pub const WILD_NOSELECT: i32 = 0x4000;
    pub const WILD_MAY_EXPAND_PATTERN: i32 = 0x8000;
    pub const WILD_FUNC_TRIGGER: i32 = 0x1_0000;
}

// =============================================================================
// Constants - EW flags (from path.h, for expand_wildcards)
// =============================================================================

/// Flags for `expand_wildcards()`
pub mod ew_flags {
    pub const EW_DIR: i32 = 0x01;
    pub const EW_FILE: i32 = 0x02;
    pub const EW_NOTFOUND: i32 = 0x04;
    pub const EW_ADDSLASH: i32 = 0x08;
    pub const EW_KEEPALL: i32 = 0x10;
    pub const EW_SILENT: i32 = 0x20;
    pub const EW_EXEC: i32 = 0x40;
    pub const EW_PATH: i32 = 0x80;
    pub const EW_ICASE: i32 = 0x100;
    pub const EW_NOERROR: i32 = 0x200;
    pub const EW_NOTWILD: i32 = 0x400;
    pub const EW_KEEPDOLLAR: i32 = 0x800;
    pub const EW_ALLLINKS: i32 = 0x1000;
    pub const EW_SHELLCMD: i32 = 0x2000;
    pub const EW_DODOT: i32 = 0x4000;
    pub const EW_EMPTYOK: i32 = 0x8000;
    pub const EW_NOTENV: i32 = 0x1_0000;
    pub const EW_CDPATH: i32 = 0x2_0000;
    pub const EW_NOBREAK: i32 = 0x4_0000;
}

// =============================================================================
// Constants - VSE flags (from ex_getln.h)
// =============================================================================

/// Flags for `vim_strsave_fnameescape()`
pub mod vse_flags {
    pub const VSE_NONE: i32 = 0;
    pub const VSE_SHELL: i32 = 1;
    pub const VSE_BUFFER: i32 = 2;
}

// =============================================================================
// Constants - Wildoptions flags (from option_vars.h)
// =============================================================================

/// Flag for 'wildoptions' fuzzy matching
pub const K_OPT_WOP_FLAG_FUZZY: u32 = 0x01;
/// Flag for 'wildoptions' pum
pub const K_OPT_WOP_FLAG_PUM: u32 = 0x04;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_expand_context_conversion() {
        assert_eq!(
            ExpandContext::from_raw(ExpandContext::Commands.to_raw()),
            Some(ExpandContext::Commands)
        );
        assert_eq!(
            ExpandContext::from_raw(ExpandContext::Files.to_raw()),
            Some(ExpandContext::Files)
        );
        assert_eq!(
            ExpandContext::from_raw(ExpandContext::Nothing.to_raw()),
            Some(ExpandContext::Nothing)
        );
        assert_eq!(ExpandContext::from_raw(999), None);
        assert_eq!(ExpandContext::from_raw(-10), None);
    }

    #[test]
    fn test_expand_context_values() {
        assert_eq!(ExpandContext::Unsuccessful.to_raw(), -2);
        assert_eq!(ExpandContext::Ok.to_raw(), -1);
        assert_eq!(ExpandContext::Nothing.to_raw(), 0);
        assert_eq!(ExpandContext::Commands.to_raw(), 1);
        assert_eq!(ExpandContext::Files.to_raw(), 2);
    }

    #[test]
    fn test_backslash_constants() {
        use backslash::*;
        assert_eq!(XP_BS_NONE, 0);
        assert_eq!(XP_BS_ONE, 0x1);
        assert_eq!(XP_BS_THREE, 0x2);
        assert_eq!(XP_BS_COMMA, 0x4);
    }

    #[test]
    fn test_wild_mode_constants() {
        use wild_mode::*;
        assert_eq!(WILD_FREE, 1);
        assert_eq!(WILD_EXPAND_FREE, 2);
        assert_eq!(WILD_NEXT, 4);
        assert_eq!(WILD_PREV, 5);
        assert_eq!(WILD_ALL, 6);
        assert_eq!(WILD_LONGEST, 7);
    }

    #[test]
    fn test_wild_options_constants() {
        use wild_options::*;
        assert_eq!(WILD_LIST_NOTFOUND, 0x01);
        assert_eq!(WILD_HOME_REPLACE, 0x02);
        assert_eq!(WILD_SILENT, 0x40);
        assert_eq!(WILD_ESCAPE, 0x80);
    }

    #[test]
    fn test_ew_flags() {
        use ew_flags::*;
        assert_eq!(EW_DIR, 0x01);
        assert_eq!(EW_FILE, 0x02);
        assert_eq!(EW_NOTFOUND, 0x04);
        assert_eq!(EW_ADDSLASH, 0x08);
        assert_eq!(EW_KEEPALL, 0x10);
        assert_eq!(EW_SILENT, 0x20);
        assert_eq!(EW_NOERROR, 0x200);
        assert_eq!(EW_ALLLINKS, 0x1000);
    }
}
