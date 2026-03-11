//! Command-line argument parsing
//!
//! This module provides Rust implementations for Neovim's
//! command-line argument parsing (argc/argv processing).

use std::ffi::{c_char, c_int};

// =============================================================================
// Argument Types
// =============================================================================

/// Command-line argument types.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArgType {
    /// Regular file argument
    File = 0,
    /// Command to execute (-c)
    Command = 1,
    /// Expression to evaluate (--cmd)
    PreCommand = 2,
    /// Lua script (-l)
    LuaScript = 3,
    /// Lua argument (after --)
    LuaArg = 4,
    /// Tag to jump to (-t)
    Tag = 5,
    /// Error pattern to jump to (-q)
    ErrorFile = 6,
    /// Window position (-geometry on X11)
    Geometry = 7,
}

impl ArgType {
    /// Create from C int.
    #[must_use]
    pub const fn from_c_int(val: c_int) -> Self {
        match val {
            1 => Self::Command,
            2 => Self::PreCommand,
            3 => Self::LuaScript,
            4 => Self::LuaArg,
            5 => Self::Tag,
            6 => Self::ErrorFile,
            7 => Self::Geometry,
            _ => Self::File,
        }
    }

    /// Convert to C int.
    #[must_use]
    pub const fn to_c_int(self) -> c_int {
        self as c_int
    }
}

// =============================================================================
// Argument Flags
// =============================================================================

/// Flags from parsed arguments.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct ArgFlags {
    /// Start in binary mode (-b)
    pub binary: bool,
    /// Start in diff mode (-d)
    pub diff: bool,
    /// Start in Ex mode (-e, -E)
    pub ex_mode: bool,
    /// Improved Ex mode (-E)
    pub ex_improved: bool,
    /// Read-only mode (-R)
    pub readonly: bool,
    /// Restricted mode (-Z)
    pub restricted: bool,
    /// View mode (implied -R)
    pub view: bool,
    /// Recovery mode (-r)
    pub recover: bool,
    /// No swap file (-n)
    pub no_swap: bool,
    /// Verbose mode (-V)
    pub verbose: bool,
    /// Silent mode (-es, -Es)
    pub silent: bool,
    /// Open in tabs (-p)
    pub tabs: bool,
    /// Open in windows (-o, -O)
    pub windows: bool,
    /// Horizontal windows (-o)
    pub horizontal: bool,
    /// Vertical windows (-O)
    pub vertical: bool,
}

impl ArgFlags {
    /// Create new arg flags.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            binary: false,
            diff: false,
            ex_mode: false,
            ex_improved: false,
            readonly: false,
            restricted: false,
            view: false,
            recover: false,
            no_swap: false,
            verbose: false,
            silent: false,
            tabs: false,
            windows: false,
            horizontal: false,
            vertical: false,
        }
    }

    /// Check if any window layout is requested.
    #[must_use]
    pub const fn has_layout(&self) -> bool {
        self.tabs || self.windows
    }

    /// Check if starting in any Ex mode.
    #[must_use]
    pub const fn is_ex(&self) -> bool {
        self.ex_mode || self.ex_improved
    }
}

// =============================================================================
// Argument Parsing State
// =============================================================================

/// State during argument parsing.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct ArgParseState {
    /// Current argument index
    pub index: c_int,
    /// Total argument count
    pub argc: c_int,
    /// Number of files found
    pub file_count: c_int,
    /// Number of commands found
    pub cmd_count: c_int,
    /// Encountered "--"
    pub double_dash: bool,
    /// Encountered error
    pub had_error: bool,
    /// Should exit after parsing
    pub should_exit: bool,
    /// Exit code if should_exit
    pub exit_code: c_int,
}

impl ArgParseState {
    /// Create new parse state.
    #[must_use]
    pub const fn new(argc: c_int) -> Self {
        Self {
            index: 1, // Skip argv[0]
            argc,
            file_count: 0,
            cmd_count: 0,
            double_dash: false,
            had_error: false,
            should_exit: false,
            exit_code: 0,
        }
    }

    /// Check if more arguments available.
    #[must_use]
    pub const fn has_more(&self) -> bool {
        self.index < self.argc
    }

    /// Advance to next argument.
    pub fn advance(&mut self) {
        self.index += 1;
    }

    /// Mark parsing as done with exit.
    pub fn exit_with(&mut self, code: c_int) {
        self.should_exit = true;
        self.exit_code = code;
    }
}

// =============================================================================
// Option Recognition
// =============================================================================

/// Recognized short options.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShortOpt {
    /// Unknown option
    Unknown = 0,
    /// -b binary mode
    Binary = 1,
    /// -c command
    Command = 2,
    /// -d diff mode
    Diff = 3,
    /// -e Ex mode
    Ex = 4,
    /// -h help (show usage)
    Help = 5,
    /// -l Lisp mode (legacy)
    Lisp = 6,
    /// -m modifications disabled
    NoModify = 7,
    /// -n no swap file
    NoSwap = 8,
    /// -o horizontal windows
    Horizontal = 9,
    /// -p tabs
    Tabs = 10,
    /// -q quickfix errorfile
    Quickfix = 11,
    /// -r recovery mode
    Recover = 12,
    /// -s script mode
    Script = 13,
    /// -t tag
    Tag = 14,
    /// -u use vimrc
    UseVimrc = 15,
    /// -v version (deprecated)
    Version = 16,
    /// -w script output
    ScriptOut = 17,
    /// -A Arabic mode (legacy)
    Arabic = 18,
    /// -D debug mode
    Debug = 19,
    /// -E improved Ex mode
    ExImproved = 20,
    /// -M no modifications at all
    NoModifyAny = 21,
    /// -O vertical windows
    Vertical = 22,
    /// -R readonly mode
    Readonly = 23,
    /// -S source file
    Source = 24,
    /// -V verbose mode
    Verbose = 25,
    /// -W script output (overwrite)
    ScriptOutOverwrite = 26,
    /// -Z restricted mode
    Restricted = 27,
}

impl ShortOpt {
    /// Parse a short option character.
    #[must_use]
    pub const fn from_char(c: u8) -> Self {
        match c {
            b'b' => Self::Binary,
            b'c' => Self::Command,
            b'd' => Self::Diff,
            b'e' => Self::Ex,
            b'h' => Self::Help,
            b'l' => Self::Lisp,
            b'm' => Self::NoModify,
            b'n' => Self::NoSwap,
            b'o' => Self::Horizontal,
            b'p' => Self::Tabs,
            b'q' => Self::Quickfix,
            b'r' => Self::Recover,
            b's' => Self::Script,
            b't' => Self::Tag,
            b'u' => Self::UseVimrc,
            b'v' => Self::Version,
            b'w' => Self::ScriptOut,
            b'A' => Self::Arabic,
            b'D' => Self::Debug,
            b'E' => Self::ExImproved,
            b'M' => Self::NoModifyAny,
            b'O' => Self::Vertical,
            b'R' => Self::Readonly,
            b'S' => Self::Source,
            b'V' => Self::Verbose,
            b'W' => Self::ScriptOutOverwrite,
            b'Z' => Self::Restricted,
            _ => Self::Unknown,
        }
    }

    /// Check if option requires an argument.
    #[must_use]
    pub const fn requires_arg(self) -> bool {
        matches!(
            self,
            Self::Command
                | Self::Quickfix
                | Self::Tag
                | Self::UseVimrc
                | Self::ScriptOut
                | Self::Source
                | Self::Verbose
                | Self::ScriptOutOverwrite
        )
    }
}

// =============================================================================
// Long Option Recognition
// =============================================================================

/// Recognized long options.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LongOpt {
    /// Unknown option
    Unknown = 0,
    /// --help
    Help = 1,
    /// --version
    Version = 2,
    /// --noplugin
    NoPlugin = 3,
    /// --clean
    Clean = 4,
    /// --headless
    Headless = 5,
    /// --embed
    Embed = 6,
    /// --listen
    Listen = 7,
    /// --server
    Server = 8,
    /// --remote
    Remote = 9,
    /// --remote-silent
    RemoteSilent = 10,
    /// --remote-wait
    RemoteWait = 11,
    /// --remote-wait-silent
    RemoteWaitSilent = 12,
    /// --remote-send
    RemoteSend = 13,
    /// --remote-expr
    RemoteExpr = 14,
    /// --startuptime
    StartupTime = 15,
    /// --cmd
    Cmd = 16,
    /// --api-info
    ApiInfo = 17,
    /// --luamod-dev
    LuamodDev = 18,
}

/// Check if a string matches a long option name.
///
/// # Safety
/// `arg` must be valid.
pub unsafe fn rs_match_long_opt(arg: *const c_char) -> c_int {
    if arg.is_null() {
        return 0;
    }

    // Skip leading dashes
    let mut p = arg;
    if *p as u8 == b'-' {
        p = p.add(1);
    }
    if *p as u8 == b'-' {
        p = p.add(1);
    }

    // Match against known options
    let opt = match_long_opt_str(p);
    opt as c_int
}

/// Match a long option string (after --).
///
/// # Safety
/// `p` must be valid.
unsafe fn match_long_opt_str(p: *const c_char) -> LongOpt {
    // Helper to compare
    fn str_eq(p: *const c_char, s: &[u8]) -> bool {
        let mut i = 0;
        unsafe {
            while i < s.len() {
                if *p.add(i) as u8 != s[i] {
                    return false;
                }
                i += 1;
            }
            // Check for end or '='
            let next = *p.add(i) as u8;
            next == 0 || next == b'='
        }
    }

    if str_eq(p, b"help") {
        LongOpt::Help
    } else if str_eq(p, b"version") {
        LongOpt::Version
    } else if str_eq(p, b"noplugin") {
        LongOpt::NoPlugin
    } else if str_eq(p, b"clean") {
        LongOpt::Clean
    } else if str_eq(p, b"headless") {
        LongOpt::Headless
    } else if str_eq(p, b"embed") {
        LongOpt::Embed
    } else if str_eq(p, b"listen") {
        LongOpt::Listen
    } else if str_eq(p, b"server") {
        LongOpt::Server
    } else if str_eq(p, b"remote") {
        LongOpt::Remote
    } else if str_eq(p, b"remote-silent") {
        LongOpt::RemoteSilent
    } else if str_eq(p, b"remote-wait") {
        LongOpt::RemoteWait
    } else if str_eq(p, b"remote-wait-silent") {
        LongOpt::RemoteWaitSilent
    } else if str_eq(p, b"remote-send") {
        LongOpt::RemoteSend
    } else if str_eq(p, b"remote-expr") {
        LongOpt::RemoteExpr
    } else if str_eq(p, b"startuptime") {
        LongOpt::StartupTime
    } else if str_eq(p, b"cmd") {
        LongOpt::Cmd
    } else if str_eq(p, b"api-info") {
        LongOpt::ApiInfo
    } else if str_eq(p, b"luamod-dev") {
        LongOpt::LuamodDev
    } else {
        LongOpt::Unknown
    }
}

// =============================================================================
// Argument Classification
// =============================================================================

/// Classify an argument string.
///
/// # Safety
/// `arg` must be valid.
pub unsafe fn rs_classify_arg(arg: *const c_char) -> c_int {
    if arg.is_null() {
        return 0;
    }

    let first = *arg as u8;

    if first == b'-' {
        let second = *arg.add(1) as u8;
        if second == b'-' {
            // Long option or --
            let third = *arg.add(2) as u8;
            if third == 0 {
                2 // Just "--"
            } else {
                3 // Long option
            }
        } else if second == 0 {
            0 // Just "-" (stdin)
        } else {
            1 // Short option
        }
    } else if first == b'+' {
        4 // +command
    } else {
        0 // File
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_arg_type() {
        assert_eq!(ArgType::from_c_int(0), ArgType::File);
        assert_eq!(ArgType::from_c_int(1), ArgType::Command);
    }

    #[test]
    fn test_arg_flags() {
        let mut flags = ArgFlags::new();
        assert!(!flags.has_layout());

        flags.tabs = true;
        assert!(flags.has_layout());

        flags.ex_mode = true;
        assert!(flags.is_ex());
    }

    #[test]
    fn test_arg_parse_state() {
        let mut state = ArgParseState::new(5);
        assert_eq!(state.index, 1);
        assert!(state.has_more());

        state.advance();
        assert_eq!(state.index, 2);

        state.exit_with(1);
        assert!(state.should_exit);
        assert_eq!(state.exit_code, 1);
    }

    #[test]
    fn test_short_opt() {
        assert_eq!(ShortOpt::from_char(b'c'), ShortOpt::Command);
        assert_eq!(ShortOpt::from_char(b'h'), ShortOpt::Help);
        assert_eq!(ShortOpt::from_char(b'x'), ShortOpt::Unknown);

        assert!(ShortOpt::Command.requires_arg());
        assert!(!ShortOpt::Help.requires_arg());
    }

    #[test]
    fn test_classify_arg() {
        unsafe {
            assert_eq!(rs_classify_arg(c"file.txt".as_ptr()), 0);
            assert_eq!(rs_classify_arg(c"-c".as_ptr()), 1);
            assert_eq!(rs_classify_arg(c"--help".as_ptr()), 3);
            assert_eq!(rs_classify_arg(c"--".as_ptr()), 2);
            assert_eq!(rs_classify_arg(c"+10".as_ptr()), 4);
        }
    }
}
