#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

// Canonical type definitions, hoisted out of the per-module copies c2rust
// emitted. One definition per logical type; every module re-exports here.
use super::*;
use crate::memory::XString;

crate::flag_set! {
    /// How the completion machinery must escape a backslash in what it
    /// answers -- upstream's `XP_BS_*`, the bits [`Expand::backslash`]
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
/// [`Expand::context`] carries.
///
/// c2rust left this a bare `c_int` and re-emitted the sixty-five constants
/// into twenty-eight modules, so nothing related a value to the field and
/// every dispatch over it needed a catch-all arm. It is also the family with
/// the most name collisions in the tree: `ExpandContext::Nothing` existed eighteen
/// times over.
///
/// `#[repr(i32)]`, because [`Expand`] is `repr(C)`; `EXPAND_OK` is *not* a
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

/// One completion candidate, as a generator answers it: a name out of a
/// table the program carries, or a copy of one out of the editor's state.
///
/// An empty one is skipped but still counts, which is how a generator keeps
/// its numbering across an entry it does not want offered.
pub type Candidate = ::std::borrow::Cow<'static, ::core::ffi::CStr>;

/// A completion generator: the `idx`th candidate for `expand`, or `None` once
/// `idx` is past the last one. Called with `idx` 0 first and rising by one,
/// which several generators rely on to restart a walk they keep a cursor
/// for.
///
/// Upstream answers a `char *` borrowed from wherever the name lives --
/// often the context's own scratch buffer, so the caller had to copy it
/// before asking again. The answer owns or borrows for `'static` instead.
pub type CompleteListItemGetter = fn(&Expand, usize) -> Option<Candidate>;
/// One completion: what is being completed, the text it was worked out
/// from, and the matches it found -- upstream's `expand_T`.
///
/// Upstream points `xp_pattern` and `xp_line` into whatever string the
/// caller handed over -- most often the command line itself, which then has
/// to fix the pointer up every time it reallocates. Here the context owns a
/// copy of that text, and the pattern is an offset into it: the command
/// line's own offsets are the same numbers, so the one caller that edits
/// the command line in place keeps using them there.
pub struct Expand {
    /// What is being completed: upstream's `xp_context`.
    pub context: ExpandContext,
    /// A copy of the text the context was worked out from, upstream's
    /// `xp_line`: the command line, or the string `getcompletion()` was
    /// handed. While the context is being worked out it stops at the cursor.
    pub line: XString,
    /// Where the cursor is in [`line`](Self::line): upstream's `xp_col`.
    pub col: ::core::ffi::c_int,
    /// Where the text to complete starts in [`line`](Self::line):
    /// upstream's `xp_pattern`, as an offset.
    pub pattern: usize,
    /// How long the text to complete is: upstream's `xp_pattern_len`.
    pub pattern_len: usize,
    /// For a boolean option name, the `no`/`inv` it was typed with.
    pub prefix: XpPrefix,
    /// The function a `custom,`/`customlist,` completion calls: upstream's
    /// `xp_arg`.
    pub arg: Option<XString>,
    /// The Lua function a Lua user command completes with.
    pub luaref: LuaRef,
    /// Where the user command that completes was defined, for the call.
    pub script_ctx: ScriptCtx,
    /// How a backslash in a match has to be escaped.
    pub backslash: BackslashEscape,
    /// Whether the matches go to a shell, which escapes more.
    pub shell: bool,
    /// The matches, kept between `<Tab>` presses: upstream's `xp_files`
    /// and `xp_numfiles`, whose -1 -- "nothing expanded yet" -- is `None`.
    pub matches: Option<Vec<XString>>,
    /// Whether the last expansion found anything, which upstream's
    /// `xp_files != NULL` answers -- and keeps answering after the matches
    /// are freed, because freeing them does not clear the pointer.
    /// `cmdcomplete_info()` tells "nothing expanded" from "nothing kept" by
    /// it.
    pub found_any: bool,
    /// The selected match, -1 for the original text.
    pub selected: ::core::ffi::c_int,
    /// The text the matches replaced, to go back to: upstream's `xp_orig`.
    pub orig: Option<XString>,
    /// Which way a pattern-in-buffer completion searches.
    pub search_dir: Direction,
    /// Where `'incsearch'` started, for a pattern-in-buffer completion.
    pub pre_incsearch_pos: Pos,
}

impl Expand {
    /// A context with nothing to complete and nothing expanded: upstream's
    /// `ExpandInit` over a zeroed `expand_T`.
    pub fn new() -> Self {
        Expand {
            context: ExpandContext::Nothing,
            line: XString::new(),
            col: 0,
            pattern: 0,
            pattern_len: 0,
            prefix: XpPrefix::None,
            arg: None,
            luaref: 0,
            script_ctx: ScriptCtx::NONE,
            backslash: BackslashEscape::NONE,
            shell: false,
            matches: None,
            found_any: false,
            selected: 0,
            orig: None,
            search_dir: 0,
            pre_incsearch_pos: Pos::default(),
        }
    }

    /// Start over on the same text: `ExpandInit` for a context that is about
    /// to be worked out again from [`line`](Self::line).
    pub fn reset_keeping_line(&mut self) {
        let line = ::core::mem::take(&mut self.line);
        *self = Expand::new();
        self.line = line;
    }

    /// How many matches there are: upstream's `xp_numfiles`, -1 before
    /// anything was expanded.
    pub fn match_count(&self) -> ::core::ffi::c_int {
        self.matches.as_ref().map_or(-1, |matches| {
            ::core::ffi::c_int::try_from(matches.len()).expect("a match count fits a c_int")
        })
    }

    /// The matches, empty before anything was expanded.
    pub fn matches(&self) -> &[XString] {
        self.matches.as_deref().unwrap_or_default()
    }

    /// The text from the pattern to the end of [`line`](Self::line), or to
    /// a NUL before that: what upstream reads at `xp_pattern` as a C string.
    pub fn pattern_text(&self) -> &[u8] {
        let tail = self.line.get(self.pattern..).unwrap_or_default();
        let end = tail.iter().position(|&b| b == 0).unwrap_or(tail.len());
        &tail[..end]
    }

    /// The pattern as far as [`pattern_len`](Self::pattern_len) says.
    pub fn pattern_span(&self) -> &[u8] {
        let tail = self.line.get(self.pattern..).unwrap_or_default();
        &tail[..self.pattern_len.min(tail.len())]
    }

    /// Whether the text to complete starts with `prefix`.
    pub fn pattern_starts_with(&self, prefix: &[u8]) -> bool {
        self.pattern_text().starts_with(prefix)
    }

    /// Whether there is no text to complete, so everything matches.
    pub fn pattern_is_empty(&self) -> bool {
        self.pattern_text().is_empty()
    }

    /// The whole of [`line`](Self::line) as a C string reads it.
    pub fn line_cstr(&self) -> &::core::ffi::CStr {
        self.line.as_cstr()
    }

    /// The address of position `at` in the line, for a parser that walks it
    /// as a C string: the line is NUL-terminated, and `at` is checked to be
    /// in it, so a walk from here that stops at the NUL stays in bounds.
    ///
    /// Valid until [`line`](Self::line) is replaced or grown, which nothing
    /// does while the context is being worked out.
    ///
    /// # Panics
    ///
    /// When `at` is past the line's terminator.
    pub fn line_ptr_at(&mut self, at: usize) -> *mut ::core::ffi::c_char {
        assert!(
            at <= self.line.len(),
            "not a position in the completion's line"
        );
        self.line.as_mut_ptr().wrapping_add(at)
    }

    /// The offset of `at` in [`line`](Self::line), for a parser that walked
    /// it through [`line_ptr_at`](Self::line_ptr_at).
    ///
    /// # Panics
    ///
    /// When `at` is not inside the line or at its terminator: an address
    /// from anywhere else is a bug, not an offset.
    pub fn offset_of(&self, at: *const ::core::ffi::c_char) -> usize {
        let offset = at.addr().wrapping_sub(self.line.as_ptr().addr());
        assert!(
            offset <= self.line.len(),
            "not a position in the completion's line"
        );
        offset
    }

    /// Put the pattern at `at`, a position a parser reached in the line.
    pub fn set_pattern_at(&mut self, at: *const ::core::ffi::c_char) {
        self.pattern = self.offset_of(at);
    }
}

impl Default for Expand {
    fn default() -> Self {
        Expand::new()
    }
}

/// For a boolean option's name, how it was typed: `:set no…`, `:set inv…`,
/// or neither. Upstream's `XP_PREFIX_*`.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum XpPrefix {
    /// Typed bare.
    None,
    /// Typed `no…`.
    No,
    /// Typed `inv…`.
    Inv,
}
