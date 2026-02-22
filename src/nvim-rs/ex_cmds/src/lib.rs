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
    /// Wrapper for linetabsize_str (inline in C)
    pub fn nvim_linetabsize_str(s: *mut c_char) -> c_int;
    /// Find character in string
    pub fn vim_strchr(string: *const c_char, c: c_int) -> *const c_char;

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
    /// Get Rows (screen height)
    pub fn nvim_get_Rows() -> c_int;
    /// Get Columns (screen width)
    pub fn nvim_get_Columns() -> c_int;
    /// Get p_window option (OptInt = i64)
    pub fn nvim_get_p_window() -> i64;
    /// Set p_window option (OptInt = i64)
    pub fn nvim_set_p_window(val: i64);
    /// Set curwin->w_cursor.col
    pub fn nvim_curwin_set_cursor_col(col: c_int);
    /// Put a character to the message area
    pub fn msg_putchar(c: c_int);

    // print_line accessors
    /// Get curwin->w_p_nu
    pub fn nvim_curwin_get_w_p_nu() -> c_int;
    /// Get number_width(curwin)
    pub fn nvim_number_width_curwin() -> c_int;
    /// Get silent_mode
    pub fn nvim_get_silent_mode() -> c_int;
    /// Set silent_mode
    pub fn nvim_set_silent_mode(val: c_int);
    /// Get info_message
    pub fn nvim_get_info_message() -> c_int;
    /// Set info_message
    pub fn nvim_set_info_message(val: c_int);
    /// msg_prt_line wrapper
    pub fn nvim_msg_prt_line(s: *const c_char, list: c_int);
    /// message_filtered wrapper
    pub fn nvim_message_filtered(msg: *const c_char) -> c_int;
    /// msg_ext_set_kind wrapper
    pub fn nvim_msg_ext_set_kind_excmd(kind: *const c_char);
    /// msg_puts_hl wrapper
    pub fn nvim_msg_puts_hl_excmd(s: *const c_char, hl_id: c_int);
    /// Display error message, returns true
    pub fn emsg(s: *const c_char) -> c_int;

    // ex_copy accessors and functions
    /// Check if CMOD_LOCKMARKS is set in cmdmod
    pub fn nvim_cmdmod_has_lockmarks() -> c_int;
    /// Set curbuf->b_op_start
    pub fn nvim_curbuf_set_op_start(lnum: c_int, col: c_int);
    /// Set curbuf->b_op_end
    pub fn nvim_curbuf_set_op_end(lnum: c_int, col: c_int);
    /// Check if VIsual_active is set
    pub fn nvim_get_visual_active() -> c_int;
    /// Call check_pos(curbuf, &VIsual)
    pub fn nvim_check_pos_visual();
    /// Get a line from the buffer
    pub fn ml_get(lnum: c_int) -> *const c_char;
    /// Get the length of a line from the buffer
    pub fn ml_get_len(lnum: c_int) -> c_int;
    /// Append a line after lnum
    pub fn ml_append(lnum: c_int, line: *const c_char, len: c_int, newfile: c_int) -> c_int;
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

    // ex_change accessors and functions
    /// Get curbuf->b_p_ai (autoindent)
    pub fn nvim_curbuf_get_b_p_ai() -> c_int;
    /// Set append_indent
    pub fn nvim_set_append_indent(val: c_int);
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
    /// Execute :append command
    pub fn ex_append(eap: *mut ExArgHandle);
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
