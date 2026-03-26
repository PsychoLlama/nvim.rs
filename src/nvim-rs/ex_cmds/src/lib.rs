//! Ex command implementations for Neovim
//!
//! This crate provides Rust implementations of Ex commands from `ex_cmds.c`.
//! Ex commands are the colon commands like `:write`, `:read`, `:substitute`,
//! `:sort`, `:global`, and many more.
//!
//! ## Modules
//!
//! - [`range`] - Line range types and utilities
//! - [`read`] - `:read` command implementation
//! - [`write`] - `:write`, `:update`, `:saveas` command implementations
//! - [`substitute`] - `:substitute` (`:s`) command implementation
//! - [`global`] - `:global` (`:g`) and `:vglobal` (`:v`) command implementations
//! - [`sort`] - `:sort` command implementation
//! - [`lines`] - Line manipulation commands (`:copy`, `:move`, `:delete`, `:yank`, `:put`, `:join`)
//! - [`display`] - Display commands (`:print`, `:number`, `:list`, `:=`)
//! - [`shell`] - Shell integration (`:!`, `:shell`, filter commands)
//! - [`format`] - Text formatting (`:retab`, `:left`, `:center`, `:right`)
//! - [`mark`] - Mark-related commands (`:marks`, `:delmarks`, `:jumps`, `:changes`)
//! - [`buffer`] - Buffer commands (`:buffer`, `:bdelete`, `:bunload`, `:bwipeout`)
//! - [`window`] - Window commands (`:split`, `:vsplit`, `:close`, `:only`)
//!
//! ## Opaque Handles
//!
//! This crate uses opaque handle types to safely pass C struct pointers
//! between Rust and C code without needing to know their internal layout.

#![allow(unsafe_code)]

use std::ffi::{c_char, c_int, c_uint};

pub mod buffer;
pub mod display;
pub mod edit;
pub mod format;
pub mod global;
pub mod lines;
pub mod mark;
pub mod range;
pub mod read;
pub mod shell;
pub mod sort;
pub mod substitute;
pub mod window;
pub mod write;

// =============================================================================
// Opaque Handles
// =============================================================================

// CMD_* enum constants (stable values from ex_cmds.lua)
pub(crate) const CMD_APPEND: c_int = 0;
pub(crate) const CMD_CHANGE: c_int = 43;

/// Opaque handle to a C `exarg_T` struct.
///
/// Represents Ex command arguments passed from the command parser.
/// Contains information like line range, command name, arguments, flags, etc.
#[repr(C)]
pub struct ExArgHandle {
    _opaque: [u8; 0],
}

/// Opaque handle to a C `buf_T` struct.
///
/// Represents a Neovim buffer.
#[repr(C)]
pub struct BufHandle {
    _opaque: [u8; 0],
}

/// Opaque handle to a C `win_T` struct.
///
/// Represents a Neovim window.
#[repr(C)]
pub struct WinHandle {
    _opaque: [u8; 0],
}

/// Opaque handle to a C `regmatch_T` struct.
///
/// Represents a compiled regex match state.
#[repr(C)]
pub struct RegmatchHandle {
    _opaque: [u8; 0],
}

/// Opaque handle to a C `regmmatch_T` struct.
///
/// Represents a multi-line compiled regex match state used by `do_sub`.
#[repr(C)]
pub struct RegmmatchHandle {
    _opaque: [u8; 0],
}

// =============================================================================
// Command Address Type
// =============================================================================

/// Type of address/range used by an Ex command.
///
/// Corresponds to C `cmd_addr_T` enum.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[repr(i32)]
pub enum AddressType {
    /// Buffer line numbers (default)
    #[default]
    Lines = 0,
    /// Window number
    Windows = 1,
    /// Argument number
    Arguments = 2,
    /// Buffer number of loaded buffer
    LoadedBuffers = 3,
    /// Buffer number
    Buffers = 4,
    /// Tab page number
    Tabs = 5,
    /// Tab page that only relative
    TabsRelative = 6,
    /// Quickfix list valid entry number
    QuickfixValid = 7,
    /// Quickfix list entry number
    Quickfix = 8,
    /// Positive count or zero, defaults to 1
    Unsigned = 9,
    /// Something else, use line number for '$', '%', etc.
    Other = 10,
    /// No range used
    None = 11,
}

impl AddressType {
    /// Convert from C integer value.
    #[inline]
    #[must_use]
    pub fn from_c(value: c_int) -> Self {
        match value {
            0 => AddressType::Lines,
            1 => AddressType::Windows,
            2 => AddressType::Arguments,
            3 => AddressType::LoadedBuffers,
            4 => AddressType::Buffers,
            5 => AddressType::Tabs,
            6 => AddressType::TabsRelative,
            7 => AddressType::QuickfixValid,
            8 => AddressType::Quickfix,
            9 => AddressType::Unsigned,
            10 => AddressType::Other,
            11 => AddressType::None,
            _ => AddressType::None,
        }
    }

    /// Convert to C integer value.
    #[inline]
    #[must_use]
    pub fn to_c(self) -> c_int {
        self as c_int
    }
}

// =============================================================================
// Command Flags
// =============================================================================

bitflags::bitflags! {
    /// Flags for Ex command definitions.
    ///
    /// These flags indicate what arguments and features a command supports.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct CommandFlags: u32 {
        /// Allow line specifications
        const RANGE = 0x001;
        /// Allow a ! after the command name
        const BANG = 0x002;
        /// Allow extra args after command name
        const EXTRA = 0x004;
        /// Expand wildcards in extra part
        const XFILE = 0x008;
        /// No spaces allowed in the extra part
        const NOSPC = 0x010;
        /// Default file range is 1,$
        const DFLALL = 0x020;
        /// Extend range to include whole fold
        const WHOLEFOLD = 0x040;
        /// Argument required
        const NEEDARG = 0x080;
        /// Check for trailing vertical bar
        const TRLBAR = 0x100;
        /// Allow "x for register designation
        const REGSTR = 0x200;
        /// Allow count in argument, after command
        const COUNT = 0x400;
        /// No trailing comment allowed
        const NOTRLCOM = 0x800;
        /// Zero line number allowed
        const ZEROR = 0x1000;
        /// Do not remove CTRL-V from argument
        const CTRLV = 0x2000;
        /// Allow "+command" argument
        const CMDARG = 0x4000;
        /// Accepts buffer name
        const BUFNAME = 0x8000;
        /// Accepts unlisted buffer too
        const BUFUNL = 0x10000;
        /// Allow "++opt=val" argument
        const ARGOPT = 0x20000;
        /// Allowed in the sandbox
        const SBOXOK = 0x40000;
        /// Allowed in cmdline window
        const CMDWIN = 0x80000;
        /// Forbidden in non-'modifiable' buffer
        const MODIFY = 0x100000;
        /// Allow flags after count in argument
        const FLAGS = 0x200000;
        /// Command can be executed when textlock is set
        const LOCK_OK = 0x1000000;
        /// Keep sctx of where command was invoked
        const KEEPSCRIPT = 0x4000000;
        /// Allow incremental command preview
        const PREVIEW = 0x8000000;
    }
}

impl CommandFlags {
    /// Create from C uint value.
    #[inline]
    #[must_use]
    pub fn from_c(value: c_uint) -> Self {
        CommandFlags::from_bits_truncate(value)
    }

    /// Convert to C uint value.
    #[inline]
    #[must_use]
    pub fn to_c(self) -> c_uint {
        self.bits()
    }
}

bitflags::bitflags! {
    /// Flags for the `:substitute` command.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
    pub struct SubstituteFlags: u32 {
        /// Do multiple substitutions per line (g flag)
        const GLOBAL = 0x01;
        /// Ask for confirmation (c flag)
        const CONFIRM = 0x02;
        /// Count only, don't substitute (n flag)
        const COUNT = 0x04;
        /// If false, ignore errors (e flag)
        const ERROR = 0x08;
        /// Print last line with subs (p flag)
        const PRINT = 0x10;
        /// List last line with subs (l flag)
        const LIST = 0x20;
        /// List last line with line number (# flag)
        const NUMBER = 0x40;
        /// Ignore case (i flag)
        const IGNORE_CASE = 0x80;
        /// Match case (I flag)
        const MATCH_CASE = 0x100;
    }
}

// =============================================================================
// Extra Flags (for :print, :list, etc.)
// =============================================================================

bitflags::bitflags! {
    /// Extra flags used in exarg.flags field.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
    pub struct ExtraFlags: c_int {
        /// 'l': list
        const LIST = 0x01;
        /// '#': number
        const NR = 0x02;
        /// 'p': print
        const PRINT = 0x04;
    }
}

// =============================================================================
// Case Matching Style
// =============================================================================

/// Case matching style for `:substitute`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[repr(i32)]
pub enum SubIgnoreType {
    /// Honor the user's 'ignorecase'/'smartcase' options
    #[default]
    HonorOptions = 0,
    /// Ignore case of the search
    IgnoreCase = 1,
    /// Match case of the search
    MatchCase = 2,
}

impl SubIgnoreType {
    /// Convert from C integer value.
    #[inline]
    #[must_use]
    pub fn from_c(value: c_int) -> Self {
        match value {
            0 => SubIgnoreType::HonorOptions,
            1 => SubIgnoreType::IgnoreCase,
            2 => SubIgnoreType::MatchCase,
            _ => SubIgnoreType::HonorOptions,
        }
    }

    /// Convert to C integer value.
    #[inline]
    #[must_use]
    pub fn to_c(self) -> c_int {
        self as c_int
    }
}

// =============================================================================
// Binary Mode Forcing
// =============================================================================

/// Force binary/text mode for file operations.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[repr(i32)]
pub enum ForceBin {
    /// No forcing (use default)
    #[default]
    None = 0,
    /// Force binary mode (++bin)
    Binary = 1,
    /// Force text mode (++nobin)
    NoBinary = 2,
}

impl ForceBin {
    /// Convert from C integer value.
    #[inline]
    #[must_use]
    pub fn from_c(value: c_int) -> Self {
        match value {
            1 => ForceBin::Binary,
            2 => ForceBin::NoBinary,
            _ => ForceBin::None,
        }
    }

    /// Convert to C integer value.
    #[inline]
    #[must_use]
    pub fn to_c(self) -> c_int {
        self as c_int
    }
}

// =============================================================================
// Bad Character Handling
// =============================================================================

/// How to handle bad (unconvertible) characters during encoding conversion.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum BadCharBehavior {
    /// Replace bad characters with '?' (default)
    #[default]
    Replace,
    /// Keep bad characters as-is
    Keep,
    /// Drop (erase) bad characters
    Drop,
    /// Replace with a specific character
    ReplaceWith(u8),
}

impl BadCharBehavior {
    /// Default replacement character.
    pub const DEFAULT_REPLACEMENT: u8 = b'?';

    /// Convert from C integer value.
    ///
    /// In C: `BAD_REPLACE` = '?', `BAD_KEEP` = -1, `BAD_DROP` = -2
    #[inline]
    #[must_use]
    pub fn from_c(value: c_int) -> Self {
        match value {
            -1 => BadCharBehavior::Keep,
            -2 => BadCharBehavior::Drop,
            c if c == c_int::from(b'?') => BadCharBehavior::Replace,
            c if c > 0 && c <= 255 => BadCharBehavior::ReplaceWith(c as u8),
            _ => BadCharBehavior::Replace,
        }
    }

    /// Convert to C integer value.
    #[inline]
    #[must_use]
    pub fn to_c(self) -> c_int {
        match self {
            BadCharBehavior::Replace => c_int::from(b'?'),
            BadCharBehavior::Keep => -1,
            BadCharBehavior::Drop => -2,
            BadCharBehavior::ReplaceWith(c) => c_int::from(c),
        }
    }
}

// =============================================================================
// C Accessor Function Declarations
// =============================================================================

extern "C" {
    // ExArg accessors - these access fields of exarg_T from C
    /// Get command index from exarg_T
    pub fn nvim_exarg_get_cmdidx(eap: *mut ExArgHandle) -> c_int;
    /// Get the argument string from exarg_T
    pub fn nvim_exarg_get_arg(eap: *const ExArgHandle) -> *const c_char;
    /// Get line1 (first line number) from exarg_T
    pub fn nvim_exarg_get_line1(eap: *const ExArgHandle) -> c_int;
    /// Get line2 (second line number) from exarg_T
    pub fn nvim_exarg_get_line2(eap: *const ExArgHandle) -> c_int;
    /// Get addr_count (number of addresses given) from exarg_T
    pub fn nvim_exarg_get_addr_count(eap: *const ExArgHandle) -> c_int;
    /// Get forceit (! present) from exarg_T
    pub fn nvim_exarg_get_forceit(eap: *const ExArgHandle) -> c_int;
    /// Get flags (EXFLAG_*) from exarg_T
    pub fn nvim_exarg_get_flags(eap: *const ExArgHandle) -> c_int;

    // Buffer accessors
    /// Get the current buffer
    pub fn nvim_get_curbuf() -> *mut BufHandle;
    /// Get line count from buffer
    pub fn nvim_buf_get_line_count(buf: *const BufHandle) -> c_int;
    /// Check if buffer is modified
    pub fn nvim_buf_is_modified(buf: *const BufHandle) -> c_int;
    /// Check if buffer is readonly
    pub fn nvim_buf_is_readonly(buf: *const BufHandle) -> c_int;

    // Window accessors
    /// Get the current window
    pub fn nvim_get_curwin() -> *mut WinHandle;
    /// Get cursor line number from window (1-based)
    pub fn nvim_win_get_cursor_lnum(win: *const WinHandle) -> c_int;

    // Cursor/window option accessors
    /// Get curwin->w_p_rl (right-to-left flag)
    pub fn nvim_curwin_get_w_p_rl() -> c_int;
    /// Get curbuf->b_p_tw (textwidth)
    pub fn nvim_curbuf_get_b_p_tw() -> c_int;
    /// Get curbuf->b_p_wm (wrapmargin)
    pub fn nvim_curbuf_get_b_p_wm() -> c_int;
    /// Get curwin->w_view_width
    pub fn nvim_curwin_get_view_width() -> c_int;
    /// Get curwin->w_cursor.lnum
    pub fn nvim_curwin_get_cursor_lnum() -> c_int;
    /// Set curwin->w_cursor.lnum
    pub fn nvim_curwin_set_cursor_lnum(lnum: c_int);

    // Buffer/undo operations
    /// Save undo information for lines [top+1, bot-1]
    pub fn u_save(top: c_int, bot: c_int) -> c_int;
    /// Set line indentation
    pub fn set_indent(size: c_int, flags: c_int) -> c_int;
    /// Get current line indent
    pub fn get_indent() -> c_int;
    /// Get pointer to current cursor line
    pub fn get_cursor_line_ptr() -> *mut c_char;
    /// Notify that lines have changed
    pub fn changed_lines(
        buf: *mut BufHandle,
        lnum: c_int,
        col: c_int,
        lnume: c_int,
        xtra: c_int,
        do_buf_event: c_int,
    );
    /// Move cursor to beginning of line
    pub fn beginline(flags: c_int);
    /// Skip whitespace in string
    pub fn skipwhite(p: *const c_char) -> *const c_char;
    /// Get visual line length from column 0
    pub fn linetabsize_col(startvcol: c_int, s: *mut c_char) -> c_int;
    /// Find character in string
    pub fn vim_strchr(string: *const c_char, c: c_int) -> *mut c_char;

    // Phase 1 direct declarations (inlined from thin wrappers)
    /// check_nextcmd: returns next command pointer or NULL
    pub fn check_nextcmd(p: *const c_char) -> *mut c_char;
    /// skiptohex: skip to next hex digit
    pub fn skiptohex(p: *mut c_char) -> *mut c_char;
    /// skiptobin: skip to next binary digit
    pub fn skiptobin(p: *mut c_char) -> *mut c_char;
    /// skiptodigit: skip to next decimal digit
    pub fn skiptodigit(p: *mut c_char) -> *mut c_char;
    /// last_search_pat: return last search pattern
    pub fn last_search_pat() -> *const c_char;
    /// messaging: returns true if in messaging mode
    pub fn messaging() -> c_int;
    /// aborting: returns true if aborting
    pub fn aborting() -> c_int;
    /// line_breakcheck: check for line break
    pub fn line_breakcheck();
    /// ml_firstmarked: return first marked line
    pub fn ml_firstmarked() -> c_int;
    /// ml_setmarked: mark a line
    pub fn ml_setmarked(lnum: c_int);
    /// ml_clearmarked: clear all marks
    pub fn ml_clearmarked();
    /// changed_line_abv_curs: notify of line change above cursor
    pub fn changed_line_abv_curs();
    /// text_locked: returns true if text is locked
    pub fn text_locked() -> c_int;
    /// autowrite_all: write all changed buffers
    pub fn autowrite_all();
    /// do_autochdir: change to buffer's directory if autochdir is set
    pub fn do_autochdir();
    /// no_write_message: display "no write" error message
    pub fn no_write_message();
    /// wait_return: wait for user to press return
    pub fn wait_return(restart_edit: c_int);

    // Phase 2 direct declarations (inlined from thin wrappers)
    /// buflist_findnr: find buffer by number
    pub fn buflist_findnr(nr: c_int) -> *mut BufHandle;
    /// buf_ensure_loaded: ensure buffer is loaded
    pub fn buf_ensure_loaded(buf: *mut BufHandle);
    /// bufIsChanged: check if buffer is changed
    pub fn bufIsChanged(buf: *mut BufHandle) -> c_int;
    /// rs_bt_dontwrite: check if buffer type doesn't write (Rust impl)
    pub fn rs_bt_dontwrite(buf: *mut BufHandle) -> bool;
    /// rs_bt_nofilename: check if buffer type has no filename (Rust impl)
    pub fn rs_bt_nofilename(buf: *mut BufHandle) -> bool;
    /// os_path_exists: check if path exists
    pub fn os_path_exists(fname: *const c_char) -> c_int;
    /// os_isdir: check if path is a directory
    pub fn os_isdir(fname: *const c_char) -> c_int;
    /// os_nodetype: get node type of path
    pub fn os_nodetype(fname: *const c_char) -> c_int;
    /// os_file_mkdir: create directory (mode 0755)
    pub fn os_file_mkdir(fname: *const c_char, mode: c_int) -> c_int;
    /// fix_fname: expand and fix a filename
    pub fn fix_fname(fname: *const c_char) -> *mut c_char;
    /// otherfile: check if fname is different from current file
    pub fn otherfile(fname: *const c_char) -> c_int;
    /// check_fname: check if current buffer has a filename
    pub fn check_fname() -> c_int;
    /// do_argfile: edit a file from the argument list
    pub fn do_argfile(eap: *mut ExArgHandle, i: c_int);
    /// vim_tempname: create a temporary file name
    pub fn vim_tempname() -> *mut c_char;
    /// del_lines: delete lines from buffer
    pub fn del_lines(count: c_int, undo: c_int) -> c_int;
    /// write_lnum_adjust: adjust line numbers after write
    pub fn write_lnum_adjust(offset: c_int);
    /// no_write_message_nobang: display "no write" error without ! override
    pub fn no_write_message_nobang(buf: *mut BufHandle);
    /// getout: exit Vim (diverges)
    pub fn getout(exitval: c_int) -> !;
    /// before_quit_all: pre-quit checks
    pub fn before_quit_all(eap: *mut ExArgHandle) -> c_int;
    /// fileinfo: display file information
    pub fn fileinfo(shortmess: c_int, dont_truncate: c_int, forceit: c_int);
    /// expand_env_save: expand environment variables, return allocated string
    pub fn expand_env_save(str_: *const c_char) -> *mut c_char;
    /// rs_buflist_altfpos: update alternate file position for window (Rust impl)
    pub fn rs_buflist_altfpos(win: *mut WinHandle);
    /// setaltfname: set alternate filename
    pub fn setaltfname(
        ffname: *mut c_char,
        sfname: *mut c_char,
        lnum: c_int,
    ) -> *mut crate::BufHandle;
    /// rs_delbuf_msg: display buffer delete message (Rust)
    pub fn rs_delbuf_msg(name: *mut c_char);
    /// do_cmdline: execute a cmdline
    pub fn do_cmdline(
        cmdline: *mut c_char,
        fgetline: *mut std::ffi::c_void,
        cookie: *mut std::ffi::c_void,
        flags: c_int,
    ) -> c_int;
    /// set_bufref: set a bufref to point to a buffer
    pub fn set_bufref(ref_: *mut std::ffi::c_void, buf: *mut BufHandle);
    /// bufref_valid: check if bufref is still valid
    pub fn bufref_valid(ref_: *mut std::ffi::c_void) -> bool;
    /// close_buffer: close buffer, decrement nwindows
    pub fn close_buffer(
        win: *mut WinHandle,
        buf: *mut BufHandle,
        action: c_int,
        abort_if_last: bool,
        ignore_abort: bool,
    ) -> bool;
    /// open_buffer: open a buffer in the current window
    pub fn open_buffer(read_stdin: bool, eap: *mut ExArgHandle, flags: c_int) -> c_int;
    /// should_abort: check if we should abort after an operation
    pub fn should_abort(retval: c_int) -> c_int;
    /// check_changed: check if buffer was changed and show message
    pub fn check_changed(buf: *mut BufHandle, flags: c_int) -> bool;
    /// u_savecommon: save common undo information
    pub fn u_savecommon(
        buf: *mut BufHandle,
        top: c_int,
        bot: c_int,
        newbot: c_int,
        reload: bool,
    ) -> c_int;
    /// check_cursor_col: check cursor column for window
    pub fn check_cursor_col(win: *mut WinHandle);
    /// set_file_options: set file options based on exarg
    pub fn set_file_options(set_options: bool, eap: *mut ExArgHandle);
    /// set_forced_fenc: set forced fileencoding from exarg
    pub fn set_forced_fenc(eap: *mut ExArgHandle);

    // ex_z accessors
    /// Check if there is only one window (ONE_WINDOW macro)
    pub fn nvim_is_one_window() -> c_int;
    /// Get curwin->w_p_scr (scroll option, OptInt = i64)
    pub fn nvim_curwin_get_p_scr() -> i64;
    /// Get curwin->w_view_height
    pub fn nvim_curwin_get_view_height() -> c_int;
    /// Set ex_no_reprint flag
    pub fn nvim_set_ex_no_reprint(val: c_int);
    /// Get curbuf->b_ml.ml_line_count
    pub fn nvim_curbuf_get_b_ml_ml_line_count() -> c_int;
    /// Direct C global: p_window
    pub static mut p_window: i64;
    /// Set curwin->w_cursor.col
    pub fn nvim_curwin_set_cursor_col(col: c_int);
    /// Put a character to the message area
    pub fn msg_putchar(c: c_int);

    // print_line accessors
    /// Get curwin->w_p_nu
    pub fn nvim_curwin_get_w_p_nu() -> c_int;
    /// Get number_width(curwin)
    pub fn nvim_number_width_curwin() -> c_int;
    /// silent_mode global (bool in C)
    pub static mut silent_mode: bool;
    /// info_message global (bool in C)
    pub static mut info_message: bool;
    // Message state globals (direct C access)
    /// Direct C global: msg_scroll (int in C)
    pub static mut msg_scroll: c_int;
    /// Direct C global: msg_scrolled (int in C)
    pub static mut msg_scrolled: c_int;
    /// Direct C global: msg_row (int in C)
    pub static mut msg_row: c_int;
    /// Direct C global: msg_col (int in C)
    pub static mut msg_col: c_int;
    /// Direct C global: msg_silent (int in C)
    pub static mut msg_silent: c_int;
    /// Direct C global: msg_didout (bool in C)
    pub static mut msg_didout: bool;
    /// Direct C global: emsg_silent (int in C)
    pub static mut emsg_silent: c_int;
    // Misc state globals
    /// Direct C global: bangredo (bool in C)
    pub static mut bangredo: bool;
    /// Direct C global: quit_more (bool in C)
    pub static mut quit_more: bool;
    /// Direct C global: autocmd_busy (bool in C)
    pub static mut autocmd_busy: bool;
    /// Direct C global: did_check_timestamps (bool in C)
    pub static mut did_check_timestamps: bool;
    /// Direct C global: need_check_timestamps (bool in C)
    pub static mut need_check_timestamps: bool;
    /// Direct C global: exiting (bool in C)
    pub static mut exiting: bool;
    /// Direct C global: KeyTyped (bool in C)
    pub static mut KeyTyped: bool;
    /// Direct C global: g_do_tagpreview (int in C)
    pub static mut g_do_tagpreview: c_int;
    /// Direct C global: redraw_tabline (bool in C)
    pub static mut redraw_tabline: bool;
    /// Direct C global: p_srr (shellredir option, char* in C)
    pub static mut p_srr: *mut c_char;
    /// Direct C global: p_shq (shellquote option, char* in C)
    pub static mut p_shq: *mut c_char;
    /// msg_prt_line: print one line (direct C call)
    pub fn msg_prt_line(s: *const c_char, list: bool);
    /// message_filtered: check if message is filtered (direct C call)
    pub fn message_filtered(msg: *const c_char) -> bool;
    /// msg_ext_set_kind: set kind for extended message (direct C call)
    pub fn msg_ext_set_kind(kind: *const c_char);
    /// msg_puts_hl: print string with highlight (direct C call)
    pub fn msg_puts_hl(s: *const c_char, hl_id: c_int, right: bool);
    /// msg_outtrans: display string with translation
    pub fn msg_outtrans(str_: *const c_char, hl_id: c_int, hist: bool) -> c_int;
    /// Display error message, returns true
    pub fn emsg(s: *const c_char) -> c_int;
    /// call_shell: execute a shell command
    pub fn call_shell(cmd: *mut c_char, opts: c_int, extra_shell_arg: *mut c_char) -> c_int;
    /// ui_cursor_goto: move cursor to position
    pub fn ui_cursor_goto(new_row: c_int, new_col: c_int);
    /// win_enter: enter a window
    pub fn win_enter(wp: *mut WinHandle, undo_sync: bool);
    /// win_split: split current window
    pub fn win_split(size: c_int, flags: c_int) -> c_int;
    /// vim_strsave_escaped: escape characters in string
    pub fn vim_strsave_escaped(string: *const c_char, esc_chars: *const c_char) -> *mut c_char;
    /// AppendToRedobuff: append string to redo buffer
    pub fn AppendToRedobuff(s: *const c_char);
    /// AppendToRedobuffLit: append literal string to redo buffer
    pub fn AppendToRedobuffLit(str_: *const c_char, len: c_int);
    /// buflist_new: create or find a buffer in the buffer list
    pub fn buflist_new(
        ffname_arg: *mut c_char,
        sfname_arg: *mut c_char,
        lnum: c_int,
        flags: c_int,
    ) -> *mut BufHandle;

    // ex_copy accessors and functions
    /// Check if CMOD_LOCKMARKS is set in cmdmod
    pub fn nvim_cmdmod_has_lockmarks() -> c_int;
    /// Set curbuf->b_op_start
    pub fn nvim_curbuf_set_op_start(lnum: c_int, col: c_int);
    /// Set curbuf->b_op_end
    pub fn nvim_curbuf_set_op_end(lnum: c_int, col: c_int);
    pub static mut VIsual_active: bool;
    /// Call check_pos(curbuf, &VIsual)
    pub fn nvim_check_pos_visual();
    /// Get a line from the buffer
    pub fn ml_get(lnum: c_int) -> *const c_char;
    /// Get the length of a line from the buffer
    pub fn ml_get_len(lnum: c_int) -> c_int;
    /// Append a line after lnum
    pub fn ml_append(lnum: c_int, line: *const c_char, len: c_int, newfile: c_int) -> c_int;
    /// ml_get_buf: get a line from a specific buffer
    pub fn ml_get_buf(buf: *mut BufHandle, lnum: c_int) -> *mut c_char;
    /// ml_get_buf_len: get length of a line from a specific buffer
    pub fn ml_get_buf_len(buf: *mut BufHandle, lnum: c_int) -> c_int;
    /// ml_append_buf: append a line to a specific buffer
    pub fn ml_append_buf(
        buf: *mut BufHandle,
        lnum: c_int,
        line: *mut c_char,
        len: c_int,
        newfile: bool,
    ) -> c_int;
    /// ml_delete_flags: delete a line with flags
    pub fn ml_delete_flags(lnum: c_int, flags: c_int) -> c_int;
    /// Copy a string with known length
    pub fn xstrnsave(string: *const c_char, len: usize) -> *mut c_char;
    /// Free memory
    pub fn xfree(ptr: *mut std::ffi::c_void);
    /// Mark lines as appended
    pub fn appended_lines_mark(lnum: c_int, count: c_int);
    /// Display line count message
    pub fn msgmore(n: c_int);

    // do_ascii accessors and functions
    /// Get cursor position pointer
    pub fn get_cursor_pos_ptr() -> *const c_char;
    /// Get utfc_ptr2len (length of multibyte char sequence)
    pub fn utfc_ptr2len(p: *const c_char) -> c_int;
    /// Get character at pointer
    pub fn utf_ptr2char(p: *const c_char) -> c_int;
    /// Get byte length of character at pointer
    pub fn utf_ptr2len(p: *const c_char) -> c_int;
    /// Encode a character into bytes, return length
    pub fn utf_char2bytes(c: c_int, buf: *mut c_char) -> c_int;
    /// Check if character is a composing character (first in sequence)
    pub fn utf_iscomposing_first(c: c_int) -> c_int;
    /// Check if character is printable
    pub fn vim_isprintc(c: c_int) -> c_int;
    /// Get printable representation of character
    pub fn transchar(c: c_int) -> *const c_char;
    /// Get non-printable character representation into buffer (curbuf)
    pub fn nvim_transchar_nonprint_curbuf(buf: *mut c_char, c: c_int);
    /// Get digraph string for a character (NULL if none)
    pub fn get_digraph_for_char(val_arg: c_int) -> *const c_char;
    /// Get file format of a buffer (0=unix, 1=dos, 2=mac)
    pub fn rs_get_fileformat(buf: *mut BufHandle) -> c_int;
    /// Display a message
    pub fn msg(s: *const c_char, hl_id: c_int) -> c_int;
    /// Start message output
    pub fn msg_sb_eol();
    /// Start a message
    pub fn msg_start();
    /// Display a multiline message from C string
    pub fn nvim_msg_multiline_cstr(
        s: *const c_char,
        hl_id: c_int,
        check_int: c_int,
        hist: c_int,
        need_clear: *mut c_int,
    );
    /// Clear to end of screen
    pub fn msg_clr_eos();
    /// End message output
    pub fn msg_end() -> c_int;

    // --- Sort/uniq FFI functions ---
    // Regex (opaque regmatch_T via void*)
    /// Compile a regex pattern. Returns opaque handle or null.
    pub fn nvim_excmds_regcomp(pat: *const c_char, magic_val: c_int) -> *mut RegmatchHandle;
    /// vim_regexec: execute regex match. col=0 for start of line.
    pub fn vim_regexec(rmp: *mut RegmatchHandle, line: *const c_char, col: c_int) -> bool;
    /// Free regex handle.
    pub fn nvim_excmds_regfree(rm: *mut RegmatchHandle);
    /// Get startp[0] from regex match.
    pub fn nvim_excmds_regmatch_startp0(rm: *const RegmatchHandle) -> *const c_char;
    /// Get endp[0] from regex match.
    pub fn nvim_excmds_regmatch_endp0(rm: *const RegmatchHandle) -> *const c_char;
    /// Set rm_ic (ignore case) on regex handle.
    pub fn nvim_excmds_regmatch_set_ic(rm: *mut RegmatchHandle, ic: c_int);

    // Search/skip (direct declarations -- wrappers deleted in Phase 1)
    /// skip_regexp_err wrapper.
    pub fn nvim_excmds_skip_regexp_err(p: *const c_char, delim: c_int) -> *mut c_char;

    // Number parsing
    /// Parse a number string with given flags, store result in *result.
    pub fn nvim_excmds_str2nr(s: *const c_char, what: c_int, result: *mut i64);

    // Interrupt
    /// Direct C global: got_int (user interrupt flag)
    pub static mut got_int: bool;
    /// fast_breakcheck() - check for user interrupt.
    pub fn fast_breakcheck();

    // Error messages
    /// semsg(_(e_invarg2), p) - "Invalid argument: %s".
    pub fn nvim_excmds_semsg_invarg2(p: *const c_char);
    /// emsg(_(e_invarg)) - "Invalid argument".
    pub fn nvim_excmds_emsg_invarg();
    /// emsg(_(e_noprevre)) - "No previous regular expression".
    pub fn nvim_excmds_emsg_noprevre();
    /// emsg(_(e_interr)) - "Interrupted".
    pub fn nvim_excmds_emsg_interr();

    // Global options (direct static access)
    /// Direct C global: p_ic (ignorecase option)
    pub static mut p_ic: c_int;
    /// Direct C global: p_report (report option, int64_t/OptInt)
    pub static mut p_report: i64;
    /// Direct C global: p_warn (warn option)
    pub static mut p_warn: c_int;
    /// Direct C global: p_wa (writeany option)
    pub static mut p_wa: c_int;
    /// Direct C global: p_write (write option)
    pub static mut p_write: c_int;
    /// Direct C global: p_stmp (shelltemp option)
    pub static mut p_stmp: c_int;
    /// Direct C global: p_confirm (confirm option)
    pub static mut p_confirm: c_int;
    /// Direct C global: p_awa (autowriteall option)
    pub static mut p_awa: c_int;
    /// Direct C global: msg_scrolled_ign (bool in C)
    pub static mut msg_scrolled_ign: bool;
    /// Direct C global: firstbuf (pointer to first buffer)
    pub static mut firstbuf: *mut BufHandle;
    /// Direct C global: msg_listdo_overwrite (int in C)
    pub static mut msg_listdo_overwrite: c_int;
    /// Direct C global: keep_help_flag (bool in C)
    pub static mut keep_help_flag: bool;
    /// Direct C global: p_ur (undoreload option, OptInt = i64)
    pub static mut p_ur: i64;
    /// C global in ex_cmds_shim.c: global_need_beginline (int in C)
    pub static mut global_need_beginline: c_int;
    /// C global in ex_cmds_shim.c: append_indent (autoindent for first appended line)
    pub static mut append_indent: c_int;

    // Exarg mutation
    /// Set eap->nextcmd.
    pub fn nvim_exarg_set_nextcmd(eap: *mut ExArgHandle, p: *const c_char);
    /// Check if eap->nextcmd is NULL.
    pub fn nvim_exarg_is_nextcmd_null(eap: *mut ExArgHandle) -> c_int;

    // Mark/extmark
    /// mark_adjust wrapper.
    pub fn nvim_excmds_mark_adjust(
        line1: c_int,
        line2: c_int,
        amount: c_int,
        amount_after: c_int,
        etype: c_int,
    );
    /// extmark_splice wrapper (operates on curbuf).
    pub fn nvim_excmds_extmark_splice(
        start_row: c_int,
        start_col: c_int,
        old_row: c_int,
        old_col: c_int,
        old_byte: i64,
        new_row: c_int,
        new_col: c_int,
        new_byte: i64,
        etype: c_int,
    );

    // --- do_move FFI functions ---
    /// mark_adjust_nofold wrapper.
    pub fn nvim_excmds_mark_adjust_nofold(
        line1: c_int,
        line2: c_int,
        amount: c_int,
        amount_after: c_int,
        etype: c_int,
    );
    /// ml_find_line_or_offset wrapper (on curbuf).
    pub fn nvim_excmds_ml_find_line_or_offset(lnum: c_int) -> i64;
    /// ml_delete_flags wrapper.
    pub fn nvim_excmds_ml_delete_flags(lnum: c_int, flags: c_int) -> c_int;
    /// extmark_move_region wrapper (on curbuf).
    pub fn nvim_excmds_extmark_move_region(
        start_row: c_int,
        start_col: c_int,
        start_byte: i64,
        extent_row: c_int,
        extent_col: c_int,
        extent_byte: i64,
        new_row: c_int,
        new_col: c_int,
        new_byte: i64,
        etype: c_int,
    );
    /// buf_updates_send_changes wrapper (on curbuf).
    pub fn nvim_excmds_buf_updates_send_changes(lnum: c_int, added: i64, deleted: i64);
    /// FOR_ALL_TAB_WINDOWS fold move range wrapper.
    pub fn nvim_excmds_fold_move_range_all_wins(line1: c_int, line2: c_int, dest: c_int);
    /// Direct C global: disable_fold_update (fold update disable counter)
    pub static mut disable_fold_update: c_int;
    /// Direct C global: no_wait_return (don't wait for return)
    pub static mut no_wait_return: c_int;
    /// Direct C global: global_busy (global command busy flag)
    pub static mut global_busy: c_int;
    /// Direct C global: sub_nsubs (number of substitutions made)
    pub static mut sub_nsubs: c_int;
    /// Direct C global: sub_nlines (number of lines substituted)
    pub static mut sub_nlines: c_int;
    /// Display "N line(s) moved" message.
    pub fn nvim_excmds_smsg_lines_moved(num_lines: i64);
    /// Display E134 error message.
    pub fn nvim_excmds_emsg_e134();

    // ex_change accessors and functions
    /// Get curbuf->b_p_ai (autoindent)
    pub fn nvim_curbuf_get_b_p_ai() -> c_int;
    /// Set eap->line2
    pub fn nvim_exarg_set_line2(eap: *mut ExArgHandle, line2: c_int);
    /// Call check_cursor_lnum(curwin)
    pub fn nvim_check_cursor_lnum_call();
    /// Get curbuf->b_ml.ml_flags
    pub fn nvim_curbuf_get_ml_flags() -> c_int;
    /// Get indent of a line
    pub fn get_indent_lnum(lnum: c_int) -> c_int;
    /// Delete a line from the buffer
    pub fn ml_delete(lnum: c_int) -> c_int;
    /// Mark lines as deleted
    pub fn deleted_lines_mark(lnum: c_int, count: c_int);

    // --- ex_append FFI functions ---
    /// Toggle curbuf->b_p_ai (autoindent)
    pub fn nvim_excmds_toggle_b_p_ai();
    /// Get curbuf->b_p_iminsert
    pub fn nvim_excmds_get_b_p_iminsert() -> c_int;
    /// Check if eap->ea_getline is NULL
    pub fn nvim_excmds_ea_getline_is_null(eap: *mut ExArgHandle) -> c_int;
    /// Get eap->cstack->cs_looplevel
    pub fn nvim_excmds_get_cstack_looplevel(eap: *mut ExArgHandle) -> c_int;
    /// Call eap->ea_getline(c, eap->cookie, indent, true)
    pub fn nvim_excmds_call_getline(eap: *mut ExArgHandle, c: c_int, indent: c_int) -> *mut c_char;
    /// Get eap->nextcmd pointer
    pub fn nvim_excmds_get_nextcmd(eap: *mut ExArgHandle) -> *mut c_char;
    /// Get mutable eap->arg
    pub fn nvim_excmds_get_arg_mut(eap: *mut ExArgHandle) -> *mut c_char;
    /// Mark lines as appended (without mark adjustment)
    pub fn appended_lines(lnum: c_int, count: c_int);
    pub static mut State: c_int;
    pub static mut need_wait_return: bool;
    /// Call ui_cursor_shape()
    pub fn nvim_ui_cursor_shape_wrapper();
    /// Duplicate a string with length (allocates len+1 bytes)
    pub fn xmemdupz(data: *const c_char, len: usize) -> *mut c_char;
    /// Duplicate a string
    pub fn xstrdup(s: *const c_char) -> *mut c_char;

    // --- sub_joining_lines + sub_grow_buf FFI accessors ---
    /// Get eap->skip flag.
    pub fn nvim_exarg_get_skip(eap: *const ExArgHandle) -> c_int;
    /// Set eap->flags.
    pub fn nvim_exarg_set_flags(eap: *mut ExArgHandle, flags: c_int);
    /// do_join: join count lines (direct C call)
    pub fn do_join(
        count: usize,
        insert_space: bool,
        save_undo: bool,
        use_formatoptions: bool,
        setmark: bool,
    ) -> c_int;
    /// Get sub_nsubs global.
    pub fn nvim_excmds_get_sub_nsubs() -> c_int;
    /// Set sub_nsubs global.
    pub fn nvim_excmds_set_sub_nsubs(val: c_int);
    /// Get sub_nlines global.
    pub fn nvim_excmds_get_sub_nlines() -> c_int;
    /// Set sub_nlines global.
    pub fn nvim_excmds_set_sub_nlines(val: c_int);
    /// Format and display the substitution count message (NGETTEXT in C).
    pub fn nvim_excmds_format_sub_msg(count_only: c_int) -> c_int;
    /// Call nvim_docmd_ex_may_print_impl(eap) -- implemented in Rust (ex_docmd crate).
    pub fn nvim_docmd_ex_may_print_impl(eap: *mut ExArgHandle);
    /// save_re_pat: save regexp pattern (direct C call)
    pub fn save_re_pat(idx: c_int, pat: *mut c_char, patlen: usize, magic: c_int);
    /// add_to_history: add entry to history (direct C call)
    pub fn add_to_history(
        histype: c_int,
        new_entry: *const c_char,
        new_entrylen: usize,
        in_map: bool,
        sep: c_int,
    );
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_address_type_roundtrip() {
        for i in 0..=11 {
            let addr = AddressType::from_c(i);
            assert_eq!(addr.to_c(), i);
        }
    }

    #[test]
    fn test_command_flags() {
        let flags = CommandFlags::RANGE | CommandFlags::BANG | CommandFlags::EXTRA;
        assert!(flags.contains(CommandFlags::RANGE));
        assert!(flags.contains(CommandFlags::BANG));
        assert!(!flags.contains(CommandFlags::NEEDARG));

        // Test roundtrip
        let c_val = flags.to_c();
        let back = CommandFlags::from_c(c_val);
        assert_eq!(flags, back);
    }

    #[test]
    fn test_bad_char_behavior() {
        assert_eq!(
            BadCharBehavior::from_c(b'?' as c_int),
            BadCharBehavior::Replace
        );
        assert_eq!(BadCharBehavior::from_c(-1), BadCharBehavior::Keep);
        assert_eq!(BadCharBehavior::from_c(-2), BadCharBehavior::Drop);
        assert_eq!(
            BadCharBehavior::from_c(b'X' as c_int),
            BadCharBehavior::ReplaceWith(b'X')
        );

        assert_eq!(BadCharBehavior::Replace.to_c(), b'?' as c_int);
        assert_eq!(BadCharBehavior::Keep.to_c(), -1);
        assert_eq!(BadCharBehavior::Drop.to_c(), -2);
    }

    #[test]
    fn test_sub_ignore_type() {
        assert_eq!(SubIgnoreType::from_c(0), SubIgnoreType::HonorOptions);
        assert_eq!(SubIgnoreType::from_c(1), SubIgnoreType::IgnoreCase);
        assert_eq!(SubIgnoreType::from_c(2), SubIgnoreType::MatchCase);
        assert_eq!(SubIgnoreType::from_c(99), SubIgnoreType::HonorOptions);
    }

    #[test]
    fn test_force_bin() {
        assert_eq!(ForceBin::from_c(0), ForceBin::None);
        assert_eq!(ForceBin::from_c(1), ForceBin::Binary);
        assert_eq!(ForceBin::from_c(2), ForceBin::NoBinary);
    }
}
