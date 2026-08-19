#![forbid(unsafe_code)]

// Canonical type definitions, hoisted out of the per-module copies c2rust
// emitted. One definition per logical type; every module re-exports here.
use super::*;

crate::flag_set! {
    /// How the completion machinery must escape a backslash in what it
    /// answers -- upstream's `XP_BS_*`, the bits [`expand_T::xp_backslash`]
    /// carries. `NONE` is upstream's `XP_BS_NONE`: the context takes its
    /// text literally and nothing is escaped.
    pub struct BackslashEscape;

    /// A space is escaped with one backslash.
    const ONE = 1;
    /// A space is escaped with three backslashes -- the `'*func'` options,
    /// where the value is read back through another layer.
    const THREE = 2;
    /// A comma is escaped as well as a space.
    const COMMA = 4;
}

/// What the command line wants completed -- upstream's `EXPAND_*`, the value
/// [`expand_T::xp_context`] carries.
///
/// c2rust left this a bare `c_int` and re-emitted the sixty-five constants
/// into twenty-eight modules, so nothing related a value to the field and
/// every dispatch over it needed a catch-all arm. It is also the family with
/// the most name collisions in the tree: `ExpandContext::Nothing` existed eighteen
/// times over.
///
/// `#[repr(i32)]`, because [`expand_T`] is `repr(C)`; `EXPAND_OK` is *not* a
/// member -- see [`crate::cmdexpand::Expanded`], which is what the two
/// functions that answered with it return now.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
#[repr(i32)]
pub enum ExpandContext {
    /// Something illegal stands before the cursor; expand nothing and beep.
    Unsuccessful = -2,
    /// Nothing to expand here — the caller may insert the trigger key literally.
    Nothing = 0,
    /// Ex command names.
    Commands = 1,
    /// File names.
    Files = 2,
    /// Directory names.
    Directories = 3,
    /// Option names.
    Settings = 4,
    /// Boolean option names, for `:set no…`/`:set inv…`.
    BoolSettings = 5,
    /// Tag names.
    Tags = 6,
    /// The option's current value, offered as the first match.
    OldSetting = 7,
    /// Help tags.
    Help = 8,
    /// Buffer names.
    Buffers = 9,
    /// Autocommand event names.
    Events = 10,
    /// Menu paths, as `:menu` takes them.
    Menus = 11,
    /// `:syntax` subcommands.
    Syntax = 12,
    /// Highlight group names.
    Highlight = 13,
    /// Autocommand group names.
    Augroup = 14,
    /// Vimscript variable names.
    UserVars = 15,
    /// `:map` subcommands and arguments.
    Mappings = 16,
    /// Tag names, listed with the files they come from.
    TagsListFiles = 17,
    /// Builtin function names.
    Functions = 18,
    /// User function names.
    UserFunc = 19,
    /// A Vimscript expression.
    Expression = 20,
    /// Menu names only.
    Menunames = 21,
    /// User command names.
    UserCommands = 22,
    /// `:command` attribute names.
    UserCmdFlags = 23,
    /// `:command -nargs=` values.
    UserNargs = 24,
    /// `:command -complete=` values.
    UserComplete = 25,
    /// Environment variable names.
    EnvVars = 26,
    /// `:language` arguments.
    Language = 27,
    /// Colour scheme names.
    Colors = 28,
    /// `:compiler` arguments.
    Compiler = 29,
    /// Whatever the user's `-complete=custom` function answers.
    UserDefined = 30,
    /// Whatever the user's `-complete=customlist` function answers.
    UserList = 31,
    /// Whatever the user's Lua completion function answers.
    UserLua = 32,
    /// Executables on `$PATH`.
    ShellCmd = 33,
    /// `:sign` subcommands and arguments.
    Sign = 34,
    /// `:profile` arguments.
    Profile = 35,
    /// File type names.
    Filetype = 36,
    /// File names, searched along `'path'`.
    FilesInPath = 37,
    /// `:ownsyntax` arguments.
    Ownsyntax = 38,
    /// Locale names.
    Locales = 39,
    /// `:history` arguments.
    History = 40,
    /// User names.
    User = 41,
    /// `:syntime` arguments.
    Syntime = 42,
    /// `:command -addr=` values.
    UserAddrType = 43,
    /// Optional package names.
    Packadd = 44,
    /// `:messages` arguments.
    Messages = 45,
    /// `:mapclear` arguments.
    Mapclear = 46,
    /// Argument-list entries.
    Arglist = 47,
    /// Buffers taking part in a diff.
    DiffBuffers = 48,
    /// `:breakadd`/`:breakdel` arguments.
    Breakpoint = 49,
    /// Sourced script names.
    Scriptnames = 50,
    /// Files under `'runtimepath'`.
    Runtime = 51,
    /// A string option's accepted values.
    StringSetting = 52,
    /// A string option's current values, for `:set opt-=`.
    SettingSubtract = 53,
    /// `++opt=` arguments.
    Argopt = 54,
    /// Keymap names.
    Keymap = 55,
    /// Directory names, searched along `'cdpath'`.
    DirsInCdpath = 56,
    /// A whole shell command line.
    ShellCmdLine = 57,
    /// Whatever `'findfunc'` answers.
    Findfunc = 58,
    /// `:filetype` arguments.
    FiletypeCmd = 59,
    /// Words from the buffer matching the pattern.
    PatternInBuf = 60,
    /// `:retab` arguments.
    Retab = 61,
    /// `:checkhealth` arguments.
    Checkhealth = 62,
    /// Lua identifiers.
    Lua = 63,
    /// Whatever an LSP client answers.
    Lsp = 64,
}

/// A number that is not one of [`ExpandContext`]'s values.
///
/// Only reachable from a table walk: the completion machinery indexes
/// `COMMAND_COMPLETE` *by* the context, so the holes in that table and the
/// slot past its end come back as numbers that name nothing.
#[derive(Clone, Copy, Debug)]
pub struct NotAContext;

impl TryFrom<::core::ffi::c_int> for ExpandContext {
    type Error = NotAContext;

    fn try_from(value: ::core::ffi::c_int) -> Result<Self, NotAContext> {
        Ok(match value {
            -2 => Self::Unsuccessful,
            0 => Self::Nothing,
            1 => Self::Commands,
            2 => Self::Files,
            3 => Self::Directories,
            4 => Self::Settings,
            5 => Self::BoolSettings,
            6 => Self::Tags,
            7 => Self::OldSetting,
            8 => Self::Help,
            9 => Self::Buffers,
            10 => Self::Events,
            11 => Self::Menus,
            12 => Self::Syntax,
            13 => Self::Highlight,
            14 => Self::Augroup,
            15 => Self::UserVars,
            16 => Self::Mappings,
            17 => Self::TagsListFiles,
            18 => Self::Functions,
            19 => Self::UserFunc,
            20 => Self::Expression,
            21 => Self::Menunames,
            22 => Self::UserCommands,
            23 => Self::UserCmdFlags,
            24 => Self::UserNargs,
            25 => Self::UserComplete,
            26 => Self::EnvVars,
            27 => Self::Language,
            28 => Self::Colors,
            29 => Self::Compiler,
            30 => Self::UserDefined,
            31 => Self::UserList,
            32 => Self::UserLua,
            33 => Self::ShellCmd,
            34 => Self::Sign,
            35 => Self::Profile,
            36 => Self::Filetype,
            37 => Self::FilesInPath,
            38 => Self::Ownsyntax,
            39 => Self::Locales,
            40 => Self::History,
            41 => Self::User,
            42 => Self::Syntime,
            43 => Self::UserAddrType,
            44 => Self::Packadd,
            45 => Self::Messages,
            46 => Self::Mapclear,
            47 => Self::Arglist,
            48 => Self::DiffBuffers,
            49 => Self::Breakpoint,
            50 => Self::Scriptnames,
            51 => Self::Runtime,
            52 => Self::StringSetting,
            53 => Self::SettingSubtract,
            54 => Self::Argopt,
            55 => Self::Keymap,
            56 => Self::DirsInCdpath,
            57 => Self::ShellCmdLine,
            58 => Self::Findfunc,
            59 => Self::FiletypeCmd,
            60 => Self::PatternInBuf,
            61 => Self::Retab,
            62 => Self::Checkhealth,
            63 => Self::Lua,
            64 => Self::Lsp,
            _ => return Err(NotAContext),
        })
    }
}

pub type CompleteListItemGetter =
    Option<unsafe fn(*mut expand_T, ::core::ffi::c_int) -> *mut ::core::ffi::c_char>;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct expand_T {
    pub xp_pattern: *mut ::core::ffi::c_char,
    pub xp_context: ExpandContext,
    pub xp_pattern_len: size_t,
    pub xp_prefix: xp_prefix_T,
    pub xp_arg: *mut ::core::ffi::c_char,
    pub xp_luaref: LuaRef,
    pub xp_script_ctx: sctx_T,
    pub xp_backslash: BackslashEscape,
    pub xp_shell: bool,
    pub xp_numfiles: ::core::ffi::c_int,
    pub xp_col: ::core::ffi::c_int,
    pub xp_selected: ::core::ffi::c_int,
    pub xp_orig: *mut ::core::ffi::c_char,
    pub xp_files: *mut *mut ::core::ffi::c_char,
    pub xp_line: *mut ::core::ffi::c_char,
    pub xp_buf: [::core::ffi::c_char; 256],
    pub xp_search_dir: Direction,
    pub xp_pre_incsearch_pos: pos_T,
}
pub type xp_prefix_T = ::core::ffi::c_uint;
