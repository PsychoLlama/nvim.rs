//! Ex command utilities for Neovim
//!
//! Provides utility functions for Ex command parsing and processing.

#![allow(unsafe_code)]
#![allow(clippy::doc_markdown)]
#![allow(clippy::missing_safety_doc)]
#![allow(clippy::missing_const_for_fn)]
#![allow(clippy::must_use_candidate)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::cast_possible_wrap)]
#![allow(clippy::cast_possible_truncation)]

pub mod address;
pub mod args;
pub mod cmd_impl;
pub mod commands;
pub mod completion;
pub mod dispatch;
pub mod errors;
pub mod execute;
pub mod findfunc;
pub mod impl_bodies;
pub mod lookup;
pub mod modifiers;
pub mod range;
pub mod source;
pub mod table;

use std::ffi::{c_char, c_int, c_void};
use std::ptr;

/// Opaque handle to a C `exarg_T` struct.
///
/// Rust code never dereferences this directly — all field access goes through
/// C accessor functions (`nvim_eap_get_*`/`nvim_eap_set_*`).
pub type ExArgHandle = *mut c_void;

/// Opaque handle to a C `cmdmod_T` struct.
///
/// Rust code never dereferences this directly — all field access goes through
/// C accessor functions (`nvim_cmod_*`).
pub type CmdModHandle = *mut c_void;

pub use address::*;
pub use args::*;
pub use cmd_impl::*;
pub use commands::*;
pub use completion::*;
pub use dispatch::*;
pub use errors::*;
pub use execute::*;
pub use findfunc::*;
pub use impl_bodies::*;
pub use lookup::*;
pub use modifiers::*;
pub use range::*;
pub use source::*;
pub use table::*;

// =============================================================================
// Vimgrep flags
// =============================================================================

/// Vimgrep flag: Match globally (all matches on a line, not just first)
pub const VGR_GLOBAL: c_int = 1;
/// Vimgrep flag: Don't jump to first match
pub const VGR_NOJUMP: c_int = 2;
/// Vimgrep flag: Use fuzzy matching
pub const VGR_FUZZY: c_int = 4;

// FFI declarations for C helper functions
extern "C" {
    fn cmdname_first_char(cmdidx: c_int) -> c_int;
    fn nvim_get_ex_pressedreturn() -> c_int;
    fn nvim_get_expr_map_lock() -> c_int;
    fn nvim_curbuf_is_dummy() -> c_int;
    static cmdwin_type: c_int;
    fn nvim_get_textlock() -> c_int;
    fn nvim_get_e_cmdwin() -> *const c_char;
    fn nvim_get_e_textlock() -> *const c_char;

    // Character classification from charset crate
    #[link_name = "vim_isIDc"]
    fn rs_vim_isIDc(c: c_int) -> bool;
    #[link_name = "skiptowhite"]
    fn rs_skiptowhite(p: *const c_char) -> *const c_char;

    // Regex skipping from Rust regexp crate
    fn rs_skip_regexp(startp: *mut c_char, delim: c_int, magic: c_int) -> *mut c_char;
}

/// Check if character ends an Ex command.
///
/// Returns true if the character is one of:
/// - NUL (0) - end of string
/// - '|' - command separator
/// - '"' - start of comment
/// - '\n' - newline
///
/// These characters terminate command parsing in Ex command lines.
#[inline]
pub fn ends_excmd(c: i32) -> bool {
    c == 0 || c == b'|' as i32 || c == b'"' as i32 || c == b'\n' as i32
}

/// FFI export for `ends_excmd`.
///
/// Returns 1 if the character ends an Ex command, 0 otherwise.
#[export_name = "ends_excmd"]
pub extern "C" fn rs_ends_excmd(c: c_int) -> c_int {
    c_int::from(ends_excmd(c))
}

/// Find the next command in a string.
///
/// Scans past the first '|' or '\n' character, returning the position after it.
/// Returns `None` if no command separator is found (i.e., reaches NUL).
///
/// This is used for parsing Ex command lines that can contain multiple
/// commands separated by '|' or '\n'.
#[inline]
pub fn find_nextcmd(s: &[u8]) -> Option<usize> {
    for (i, &c) in s.iter().enumerate() {
        if c == b'|' || c == b'\n' {
            return Some(i + 1);
        }
        if c == 0 {
            return None;
        }
    }
    None
}

/// FFI export for `find_nextcmd`.
///
/// Returns a pointer to the character after the first '|' or '\n',
/// or NULL if no separator is found before NUL.
///
/// # Safety
///
/// `p` must be a valid null-terminated C string.
#[export_name = "find_nextcmd"]
pub unsafe extern "C" fn rs_find_nextcmd(p: *const c_char) -> *mut c_char {
    if p.is_null() {
        return ptr::null_mut();
    }

    let mut ptr = p;
    loop {
        let c = unsafe { *ptr } as u8;
        if c == b'|' || c == b'\n' {
            return unsafe { ptr.add(1) as *mut c_char };
        }
        if c == 0 {
            return ptr::null_mut();
        }
        ptr = unsafe { ptr.add(1) };
    }
}

/// Check if pointer is at a command separator, skipping whitespace.
///
/// Skips over whitespace (' ' and '\t'), then checks if the next character
/// is '|' or '\n'. If so, returns the position after the separator.
/// Returns `None` if not at a separator.
#[inline]
pub fn check_nextcmd(s: &[u8]) -> Option<usize> {
    let mut i = 0;
    // Skip whitespace
    while i < s.len() && (s[i] == b' ' || s[i] == b'\t') {
        i += 1;
    }
    // Check for separator
    if i < s.len() && (s[i] == b'|' || s[i] == b'\n') {
        Some(i + 1)
    } else {
        None
    }
}

/// FFI export for `check_nextcmd`.
///
/// Skips whitespace, then returns a pointer to after the '|' or '\n',
/// or NULL if not at a separator.
///
/// # Safety
///
/// `p` must be a valid C string pointer.
#[export_name = "check_nextcmd"]
pub unsafe extern "C" fn rs_check_nextcmd(p: *const c_char) -> *mut c_char {
    if p.is_null() {
        return ptr::null_mut();
    }

    let mut ptr = p;
    // Skip whitespace
    loop {
        let c = unsafe { *ptr } as u8;
        if c != b' ' && c != b'\t' {
            break;
        }
        ptr = unsafe { ptr.add(1) };
    }

    let c = unsafe { *ptr } as u8;
    if c == b'|' || c == b'\n' {
        unsafe { ptr.add(1) as *mut c_char }
    } else {
        ptr::null_mut()
    }
}

/// Check if command index is for a location list command.
///
/// Returns true if the command at the given index starts with 'l',
/// indicating it's a location list command rather than a quickfix command.
/// Returns false if the index is out of bounds.
#[inline]
pub fn is_loclist_cmd(cmdidx: i32) -> bool {
    let cmd_size = crate::commands::CMD_SIZE;
    if cmdidx < 0 || cmdidx >= cmd_size {
        return false;
    }
    // Call C helper to get first char of command name
    let first_char = unsafe { cmdname_first_char(cmdidx) };
    first_char == b'l' as c_int
}

/// FFI export for `is_loclist_cmd`.
///
/// Returns 1 if the command is a location list command, 0 otherwise.
#[export_name = "is_loclist_cmd"]
pub extern "C" fn rs_is_loclist_cmd(cmdidx: c_int) -> c_int {
    c_int::from(is_loclist_cmd(cmdidx))
}

/// Get the current value of ex_pressedreturn.
///
/// Returns true if the user pressed Enter on an empty command line.
///
/// # Safety
///
/// Calls external C function to access static variable.
#[export_name = "get_pressedreturn"]
pub unsafe extern "C" fn rs_get_pressedreturn() -> c_int {
    nvim_get_ex_pressedreturn()
}

/// Check if expression mapping is locked.
///
/// Returns true if `expr_map_lock > 0` and current buffer is not a dummy buffer.
/// This prevents use of ex_normal() and text changes while running an expr mapping.
///
/// # Safety
///
/// Calls external C functions to access global variables.
#[export_name = "expr_map_locked"]
pub unsafe extern "C" fn rs_expr_map_locked() -> c_int {
    let lock = nvim_get_expr_map_lock();
    let is_dummy = nvim_curbuf_is_dummy();
    c_int::from(lock > 0 && is_dummy == 0)
}

/// Check if text is locked.
///
/// Returns true when the text must not be changed and we can't switch to
/// another window or buffer. True when editing the command line, etc.
///
/// This returns true if:
/// - cmdwin_type != 0 (in command-line window)
/// - expr_map_locked() is true (running expression mapping)
/// - textlock != 0 (text editing is locked)
///
/// # Safety
///
/// Calls external C functions to access global variables.
#[no_mangle]
pub unsafe extern "C" fn rs_text_locked() -> c_int {
    if cmdwin_type != 0 {
        return 1;
    }
    if rs_expr_map_locked() != 0 {
        return 1;
    }
    let textlock = nvim_get_textlock();
    c_int::from(textlock != 0)
}

/// Direct C replacement for text_locked().
///
/// # Safety
///
/// Calls external C functions to access global variables.
#[must_use]
#[export_name = "text_locked"]
pub unsafe extern "C" fn text_locked_rs() -> bool {
    if cmdwin_type != 0 {
        return true;
    }
    if rs_expr_map_locked() != 0 {
        return true;
    }
    let textlock = nvim_get_textlock();
    textlock != 0
}

/// Get the appropriate error message for text being locked.
///
/// Returns a pointer to either e_cmdwin or e_textlock based on
/// whether we're in a command-line window or not.
///
/// # Safety
///
/// Returns a pointer to a static C string. Caller must not free it.
#[no_mangle]
pub unsafe extern "C" fn rs_get_text_locked_msg() -> *const c_char {
    if cmdwin_type != 0 {
        nvim_get_e_cmdwin()
    } else {
        nvim_get_e_textlock()
    }
}

/// Direct C replacement for get_text_locked_msg().
///
/// # Safety
///
/// Returns a pointer to a static C string. Caller must not free it.
#[must_use]
#[export_name = "get_text_locked_msg"]
pub unsafe extern "C" fn get_text_locked_msg_rs() -> *const c_char {
    if cmdwin_type != 0 {
        nvim_get_e_cmdwin()
    } else {
        nvim_get_e_textlock()
    }
}

// =============================================================================
// Skip functions for vimgrep patterns
// =============================================================================

/// Skip over a vimgrep pattern.
///
/// Handles both forms:
/// - `:vimgrep pattern fname` - pattern is an identifier
/// - `:vimgrep /pattern/[g][j][f] fname` - pattern is delimited
///
/// # Arguments
///
/// * `p` - Pointer to the start of the pattern
/// * `s` - If not NULL, points to the start of the pattern string (will be NUL-terminated)
/// * `flags` - If not NULL, receives the flags: VGR_GLOBAL, VGR_NOJUMP, VGR_FUZZY
///
/// # Returns
///
/// A pointer to the character just past the pattern plus flags, or NULL on error.
///
/// # Safety
///
/// `p` must be a valid pointer to a null-terminated C string.
/// `s` and `flags` must be valid for writes if non-NULL.
#[no_mangle]
pub unsafe extern "C" fn rs_skip_vimgrep_pat(
    p: *mut c_char,
    s: *mut *mut c_char,
    flags: *mut c_int,
) -> *mut c_char {
    if p.is_null() {
        return ptr::null_mut();
    }

    let first_char = *p as u8;

    // Check if the first character is an identifier character
    if rs_vim_isIDc(first_char as c_int) {
        // ":vimgrep pattern fname"
        if !s.is_null() {
            *s = p;
        }
        let end = rs_skiptowhite(p as *const c_char) as *mut c_char;
        if !s.is_null() && *end != 0 {
            *end = 0;
            return end.add(1);
        }
        return end;
    }

    // ":vimgrep /pattern/[g][j][f] fname"
    if !s.is_null() {
        *s = p.add(1);
    }

    let delim = first_char as c_int;
    let mut ptr = rs_skip_regexp(p.add(1), delim, 1);

    // Check if we found the closing delimiter
    if *ptr as u8 != first_char {
        return ptr::null_mut();
    }

    // Truncate the pattern (NUL-terminate it)
    if !s.is_null() {
        *ptr = 0;
    }
    ptr = ptr.add(1);

    // Find the flags
    loop {
        let c = *ptr as u8;
        match c {
            b'g' => {
                if !flags.is_null() {
                    *flags |= VGR_GLOBAL;
                }
            }
            b'j' => {
                if !flags.is_null() {
                    *flags |= VGR_NOJUMP;
                }
            }
            b'f' => {
                if !flags.is_null() {
                    *flags |= VGR_FUZZY;
                }
            }
            _ => break,
        }
        ptr = ptr.add(1);
    }

    ptr
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ends_excmd() {
        // Command terminators
        assert!(ends_excmd(0)); // NUL
        assert!(ends_excmd(b'|' as i32)); // pipe separator
        assert!(ends_excmd(b'"' as i32)); // comment start
        assert!(ends_excmd(b'\n' as i32)); // newline

        // Non-terminators
        assert!(!ends_excmd(b'a' as i32));
        assert!(!ends_excmd(b' ' as i32));
        assert!(!ends_excmd(b':' as i32));
        assert!(!ends_excmd(b'!' as i32));
        assert!(!ends_excmd(b'#' as i32));
        assert!(!ends_excmd(b'\t' as i32));
        assert!(!ends_excmd(b'\r' as i32));
    }

    #[test]
    fn test_ffi_ends_excmd() {
        assert_eq!(rs_ends_excmd(0), 1);
        assert_eq!(rs_ends_excmd(b'|' as c_int), 1);
        assert_eq!(rs_ends_excmd(b'"' as c_int), 1);
        assert_eq!(rs_ends_excmd(b'\n' as c_int), 1);
        assert_eq!(rs_ends_excmd(b'a' as c_int), 0);
    }

    #[test]
    fn test_find_nextcmd() {
        // Find pipe separator
        assert_eq!(find_nextcmd(b"cmd1|cmd2\0"), Some(5));
        assert_eq!(find_nextcmd(b"|cmd\0"), Some(1));

        // Find newline separator
        assert_eq!(find_nextcmd(b"cmd1\ncmd2\0"), Some(5));
        assert_eq!(find_nextcmd(b"\ncmd\0"), Some(1));

        // No separator - NUL first
        assert_eq!(find_nextcmd(b"cmd\0"), None);
        assert_eq!(find_nextcmd(b"\0"), None);

        // Empty slice returns None
        assert_eq!(find_nextcmd(b""), None);
    }

    #[test]
    fn test_ffi_find_nextcmd() {
        use std::ffi::CString;

        // Find pipe separator
        let s = CString::new("cmd1|cmd2").unwrap();
        let result = unsafe { rs_find_nextcmd(s.as_ptr()) };
        assert!(!result.is_null());
        let result_str = unsafe { std::ffi::CStr::from_ptr(result) };
        assert_eq!(result_str.to_bytes(), b"cmd2");

        // Find newline separator
        let s = CString::new("cmd1\ncmd2").unwrap();
        let result = unsafe { rs_find_nextcmd(s.as_ptr()) };
        assert!(!result.is_null());
        let result_str = unsafe { std::ffi::CStr::from_ptr(result) };
        assert_eq!(result_str.to_bytes(), b"cmd2");

        // No separator
        let s = CString::new("cmd").unwrap();
        let result = unsafe { rs_find_nextcmd(s.as_ptr()) };
        assert!(result.is_null());

        // NULL input
        let result = unsafe { rs_find_nextcmd(ptr::null()) };
        assert!(result.is_null());
    }

    #[test]
    fn test_check_nextcmd() {
        // Separator after whitespace
        assert_eq!(check_nextcmd(b"  |cmd\0"), Some(3));
        assert_eq!(check_nextcmd(b"\t\t\ncmd\0"), Some(3));
        assert_eq!(check_nextcmd(b" \t |rest\0"), Some(4));

        // Direct separator (no whitespace)
        assert_eq!(check_nextcmd(b"|cmd\0"), Some(1));
        assert_eq!(check_nextcmd(b"\ncmd\0"), Some(1));

        // Not a separator
        assert_eq!(check_nextcmd(b"cmd\0"), None);
        assert_eq!(check_nextcmd(b"  cmd\0"), None);
        assert_eq!(check_nextcmd(b"\0"), None);
    }

    #[test]
    fn test_ffi_check_nextcmd() {
        use std::ffi::CString;

        // Separator after whitespace
        let s = CString::new("  |cmd").unwrap();
        let result = unsafe { rs_check_nextcmd(s.as_ptr()) };
        assert!(!result.is_null());
        let result_str = unsafe { std::ffi::CStr::from_ptr(result) };
        assert_eq!(result_str.to_bytes(), b"cmd");

        // Not a separator
        let s = CString::new("  cmd").unwrap();
        let result = unsafe { rs_check_nextcmd(s.as_ptr()) };
        assert!(result.is_null());

        // NULL input
        let result = unsafe { rs_check_nextcmd(ptr::null()) };
        assert!(result.is_null());
    }

    #[test]
    fn test_ends_excmd_all_terminators() {
        // Test all four terminators explicitly
        let terminators = [0, b'|' as i32, b'"' as i32, b'\n' as i32];
        for term in terminators {
            assert!(
                ends_excmd(term),
                "Character {} should be a terminator",
                term
            );
        }
    }

    #[test]
    fn test_find_nextcmd_first_char() {
        // Separator as first character
        assert_eq!(find_nextcmd(b"|rest\0"), Some(1));
        assert_eq!(find_nextcmd(b"\nrest\0"), Some(1));
    }

    #[test]
    fn test_check_nextcmd_only_whitespace() {
        // Only whitespace followed by non-separator
        assert_eq!(check_nextcmd(b"   \0"), None);
        // Only whitespace followed by NUL (empty)
        assert_eq!(check_nextcmd(b"\t\t\t\0"), None);
    }

    #[test]
    fn test_check_nextcmd_mixed_whitespace() {
        // Mix of spaces and tabs before separator
        assert_eq!(check_nextcmd(b" \t \t|x\0"), Some(5));
        assert_eq!(check_nextcmd(b"\t \t \nx\0"), Some(5));
    }
}

// =============================================================================
// Phase 81: Ex Command Execution Helpers
// =============================================================================

/// Command execution flags
pub mod exec_flags {
    use std::os::raw::c_int;

    /// Execute command normally
    pub const NORMAL: c_int = 0;
    /// Silent execution (no messages)
    pub const SILENT: c_int = 0x01;
    /// Verify before executing
    pub const VERIFY: c_int = 0x02;
    /// Preview mode (no side effects)
    pub const PREVIEW: c_int = 0x04;
    /// Source file mode
    pub const SOURCE: c_int = 0x08;
    /// Don't add to history
    pub const NO_HISTORY: c_int = 0x10;
    /// Execute in sandbox
    pub const SANDBOX: c_int = 0x20;
    /// Continue after error
    pub const CONTINUE: c_int = 0x40;
    /// Verbose mode
    pub const VERBOSE: c_int = 0x80;
}

/// Command modifier flags
pub mod cmdmod_flags {
    use std::os::raw::c_int;

    /// :silent modifier
    pub const SILENT: c_int = 0x001;
    /// :silent! modifier (ignore errors)
    pub const SILENT_ERR: c_int = 0x002;
    /// :hide modifier
    pub const HIDE: c_int = 0x004;
    /// :keepalt modifier
    pub const KEEPALT: c_int = 0x008;
    /// :keepmarks modifier
    pub const KEEPMARKS: c_int = 0x010;
    /// :keepjumps modifier
    pub const KEEPJUMPS: c_int = 0x020;
    /// :lockmarks modifier
    pub const LOCKMARKS: c_int = 0x040;
    /// :noswapfile modifier
    pub const NOSWAPFILE: c_int = 0x080;
    /// :unsilent modifier
    pub const UNSILENT: c_int = 0x100;
    /// :noautocmd modifier
    pub const NOAUTOCMD: c_int = 0x200;
    /// :browse modifier
    pub const BROWSE: c_int = 0x400;
    /// :confirm modifier
    pub const CONFIRM: c_int = 0x800;
}

/// Check if execution flags include a specific flag.
#[no_mangle]
pub const extern "C" fn rs_exec_has_flag(flags: c_int, flag: c_int) -> bool {
    (flags & flag) != 0
}

/// Set an execution flag.
#[no_mangle]
pub const extern "C" fn rs_exec_set_flag(flags: c_int, flag: c_int) -> c_int {
    flags | flag
}

/// Clear an execution flag.
#[no_mangle]
pub const extern "C" fn rs_exec_clear_flag(flags: c_int, flag: c_int) -> c_int {
    flags & !flag
}

/// Check if command modifier flags include a specific modifier.
#[no_mangle]
pub const extern "C" fn rs_cmdmod_has_flag(flags: c_int, flag: c_int) -> bool {
    (flags & flag) != 0
}

/// Set a command modifier flag.
#[no_mangle]
pub const extern "C" fn rs_cmdmod_set_flag(flags: c_int, flag: c_int) -> c_int {
    flags | flag
}

/// Clear a command modifier flag.
#[no_mangle]
pub const extern "C" fn rs_cmdmod_clear_flag(flags: c_int, flag: c_int) -> c_int {
    flags & !flag
}

/// Check if execution is in silent mode.
#[no_mangle]
pub const extern "C" fn rs_exec_is_silent(flags: c_int) -> bool {
    (flags & (cmdmod_flags::SILENT | cmdmod_flags::SILENT_ERR)) != 0
}

/// Check if errors should be suppressed.
#[no_mangle]
pub const extern "C" fn rs_exec_suppress_errors(flags: c_int) -> bool {
    (flags & cmdmod_flags::SILENT_ERR) != 0
}

// =============================================================================
// Range Specification Helpers
// =============================================================================

/// Address type for Ex commands
pub mod addr_type {
    use std::os::raw::c_int;

    /// Lines in current buffer
    pub const LINES: c_int = 0;
    /// Windows in current tab
    pub const WINDOWS: c_int = 1;
    /// Arguments in argument list
    pub const ARGUMENTS: c_int = 2;
    /// Loaded buffers
    pub const LOADED_BUFS: c_int = 3;
    /// All buffers
    pub const BUFFERS: c_int = 4;
    /// Tabs
    pub const TABS: c_int = 5;
    /// Tabs with new tab
    pub const TABS_REL: c_int = 6;
    /// Quickfix entries
    pub const QUICKFIX: c_int = 7;
    /// No range
    pub const NONE: c_int = 8;
    /// Other (custom)
    pub const OTHER: c_int = 9;

    /// Number of address types
    pub const COUNT: c_int = 10;
}

/// Check if an address type refers to lines.
#[no_mangle]
pub const extern "C" fn rs_addr_is_lines(addr: c_int) -> bool {
    addr == addr_type::LINES
}

/// Check if an address type refers to windows.
#[no_mangle]
pub const extern "C" fn rs_addr_is_windows(addr: c_int) -> bool {
    addr == addr_type::WINDOWS
}

/// Check if an address type refers to buffers.
#[no_mangle]
pub const extern "C" fn rs_addr_is_buffers(addr: c_int) -> bool {
    matches!(addr, x if x == addr_type::LOADED_BUFS || x == addr_type::BUFFERS)
}

/// Check if an address type refers to tabs.
#[no_mangle]
pub const extern "C" fn rs_addr_is_tabs(addr: c_int) -> bool {
    matches!(addr, x if x == addr_type::TABS || x == addr_type::TABS_REL)
}

/// Check if range is empty (no range specified).
#[no_mangle]
pub const extern "C" fn rs_range_is_empty(line1: c_int, line2: c_int) -> bool {
    line1 == 0 && line2 == 0
}

/// Check if range is a single line.
#[no_mangle]
pub const extern "C" fn rs_range_is_single(line1: c_int, line2: c_int) -> bool {
    line1 > 0 && line1 == line2
}

/// Normalize range (ensure line1 <= line2).
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct ExRange {
    pub line1: c_int,
    pub line2: c_int,
}

/// Normalize range so line1 <= line2.
#[no_mangle]
pub const extern "C" fn rs_range_normalize(line1: c_int, line2: c_int) -> ExRange {
    if line1 <= line2 {
        ExRange { line1, line2 }
    } else {
        ExRange {
            line1: line2,
            line2: line1,
        }
    }
}

/// Calculate range count (number of lines).
#[no_mangle]
pub const extern "C" fn rs_range_count(line1: c_int, line2: c_int) -> c_int {
    if line2 >= line1 && line1 > 0 {
        line2 - line1 + 1
    } else {
        0
    }
}

/// Clamp a line number to a valid range.
#[no_mangle]
pub const extern "C" fn rs_range_clamp_line(line: c_int, max_line: c_int) -> c_int {
    if line < 1 {
        1
    } else if line > max_line {
        max_line
    } else {
        line
    }
}

// =============================================================================
// Command Context Helpers
// =============================================================================

/// Command context type
pub mod cmd_context {
    use std::os::raw::c_int;

    /// Normal command execution
    pub const NORMAL: c_int = 0;
    /// Inside a function
    pub const FUNCTION: c_int = 1;
    /// Inside a script
    pub const SCRIPT: c_int = 2;
    /// Inside a command-line window
    pub const CMDWIN: c_int = 3;
    /// Inside an autocommand
    pub const AUTOCMD: c_int = 4;
    /// Inside a mapping
    pub const MAPPING: c_int = 5;
    /// Inside an expression
    pub const EXPR: c_int = 6;
}

/// Check if command context allows certain operations.
#[no_mangle]
pub const extern "C" fn rs_context_allows_modify(context: c_int) -> bool {
    // Most contexts allow modification
    context != cmd_context::EXPR
}

/// Check if command context is interactive.
#[no_mangle]
pub const extern "C" fn rs_context_is_interactive(context: c_int) -> bool {
    context == cmd_context::NORMAL || context == cmd_context::CMDWIN
}

// =============================================================================
// Command Recursion Helpers
// =============================================================================

/// Maximum command recursion depth
pub const MAX_CMD_DEPTH: c_int = 200;

/// Check if command depth is safe.
#[no_mangle]
pub const extern "C" fn rs_cmd_depth_ok(depth: c_int) -> bool {
    depth >= 0 && depth < MAX_CMD_DEPTH
}

/// Calculate recursion level from depth.
#[no_mangle]
pub const extern "C" fn rs_cmd_depth_to_level(depth: c_int) -> c_int {
    if depth < 0 {
        0
    } else {
        depth + 1
    }
}

// =============================================================================
// Special Argument Detection
// =============================================================================

/// Check if a character could start a special argument.
///
/// Special arguments include:
/// - '%' - current file
/// - '#' - alternate file
/// - '<' - visual range
#[no_mangle]
pub const extern "C" fn rs_is_special_arg_char(c: c_int) -> bool {
    matches!(c as u8, b'%' | b'#' | b'<')
}

/// Check if a character starts a register specification.
#[no_mangle]
pub const extern "C" fn rs_is_register_char(c: c_int) -> bool {
    let c = c as u8;
    c.is_ascii_alphabetic() || matches!(c, b'"' | b'*' | b'+' | b'-' | b'/' | b':' | b'_')
}

/// Check if a character is a valid count digit.
#[no_mangle]
pub const extern "C" fn rs_is_count_char(c: c_int) -> bool {
    (c as u8).is_ascii_digit()
}

/// Parse a count from the beginning of a string.
///
/// Returns the count value (0 if no count found).
#[no_mangle]
pub const unsafe extern "C" fn rs_ex_parse_count(s: *const u8, consumed: *mut c_int) -> c_int {
    if s.is_null() {
        if !consumed.is_null() {
            *consumed = 0;
        }
        return 0;
    }

    let mut count: c_int = 0;
    let mut i: c_int = 0;

    loop {
        let c = *s.add(i as usize);
        if c == 0 || !c.is_ascii_digit() {
            break;
        }
        count = count.saturating_mul(10).saturating_add((c - b'0') as c_int);
        i += 1;
    }

    if !consumed.is_null() {
        *consumed = i;
    }
    count
}

// =============================================================================
// Phase 81 Tests
// =============================================================================

#[cfg(test)]
#[allow(clippy::manual_c_str_literals)]
mod phase81_tests {
    use super::*;

    #[test]
    fn test_exec_flags() {
        let flags = 0;
        let flags = rs_exec_set_flag(flags, exec_flags::SILENT);
        assert!(rs_exec_has_flag(flags, exec_flags::SILENT));
        assert!(!rs_exec_has_flag(flags, exec_flags::PREVIEW));

        let flags = rs_exec_set_flag(flags, exec_flags::PREVIEW);
        assert!(rs_exec_has_flag(flags, exec_flags::SILENT));
        assert!(rs_exec_has_flag(flags, exec_flags::PREVIEW));

        let flags = rs_exec_clear_flag(flags, exec_flags::SILENT);
        assert!(!rs_exec_has_flag(flags, exec_flags::SILENT));
        assert!(rs_exec_has_flag(flags, exec_flags::PREVIEW));
    }

    #[test]
    fn test_cmdmod_flags() {
        let flags = 0;
        let flags = rs_cmdmod_set_flag(flags, cmdmod_flags::SILENT);
        assert!(rs_cmdmod_has_flag(flags, cmdmod_flags::SILENT));

        let flags = rs_cmdmod_set_flag(flags, cmdmod_flags::SILENT_ERR);
        assert!(rs_exec_is_silent(flags));
        assert!(rs_exec_suppress_errors(flags));
    }

    #[test]
    fn test_addr_types() {
        assert!(rs_addr_is_lines(addr_type::LINES));
        assert!(!rs_addr_is_lines(addr_type::WINDOWS));

        assert!(rs_addr_is_windows(addr_type::WINDOWS));
        assert!(!rs_addr_is_windows(addr_type::LINES));

        assert!(rs_addr_is_buffers(addr_type::LOADED_BUFS));
        assert!(rs_addr_is_buffers(addr_type::BUFFERS));
        assert!(!rs_addr_is_buffers(addr_type::LINES));

        assert!(rs_addr_is_tabs(addr_type::TABS));
        assert!(rs_addr_is_tabs(addr_type::TABS_REL));
        assert!(!rs_addr_is_tabs(addr_type::LINES));
    }

    #[test]
    fn test_range_helpers() {
        assert!(rs_range_is_empty(0, 0));
        assert!(!rs_range_is_empty(1, 0));
        assert!(!rs_range_is_empty(0, 1));
        assert!(!rs_range_is_empty(1, 1));

        assert!(rs_range_is_single(1, 1));
        assert!(rs_range_is_single(5, 5));
        assert!(!rs_range_is_single(1, 2));
        assert!(!rs_range_is_single(0, 0));

        let r = rs_range_normalize(5, 3);
        assert_eq!(r.line1, 3);
        assert_eq!(r.line2, 5);

        let r = rs_range_normalize(3, 5);
        assert_eq!(r.line1, 3);
        assert_eq!(r.line2, 5);

        assert_eq!(rs_range_count(1, 5), 5);
        assert_eq!(rs_range_count(5, 5), 1);
        assert_eq!(rs_range_count(5, 3), 0); // Invalid
        assert_eq!(rs_range_count(0, 5), 0); // Invalid

        assert_eq!(rs_range_clamp_line(0, 100), 1);
        assert_eq!(rs_range_clamp_line(-5, 100), 1);
        assert_eq!(rs_range_clamp_line(50, 100), 50);
        assert_eq!(rs_range_clamp_line(150, 100), 100);
    }

    #[test]
    fn test_context_helpers() {
        assert!(rs_context_allows_modify(cmd_context::NORMAL));
        assert!(rs_context_allows_modify(cmd_context::FUNCTION));
        assert!(!rs_context_allows_modify(cmd_context::EXPR));

        assert!(rs_context_is_interactive(cmd_context::NORMAL));
        assert!(rs_context_is_interactive(cmd_context::CMDWIN));
        assert!(!rs_context_is_interactive(cmd_context::SCRIPT));
    }

    #[test]
    fn test_cmd_depth() {
        assert!(rs_cmd_depth_ok(0));
        assert!(rs_cmd_depth_ok(199));
        assert!(!rs_cmd_depth_ok(200));
        assert!(!rs_cmd_depth_ok(-1));

        assert_eq!(rs_cmd_depth_to_level(0), 1);
        assert_eq!(rs_cmd_depth_to_level(5), 6);
        assert_eq!(rs_cmd_depth_to_level(-1), 0);
    }

    #[test]
    fn test_special_chars() {
        assert!(rs_is_special_arg_char(b'%' as c_int));
        assert!(rs_is_special_arg_char(b'#' as c_int));
        assert!(rs_is_special_arg_char(b'<' as c_int));
        assert!(!rs_is_special_arg_char(b'x' as c_int));

        assert!(rs_is_register_char(b'a' as c_int));
        assert!(rs_is_register_char(b'Z' as c_int));
        assert!(rs_is_register_char(b'"' as c_int));
        assert!(rs_is_register_char(b'*' as c_int));
        assert!(rs_is_register_char(b'+' as c_int));
        assert!(!rs_is_register_char(b'1' as c_int));

        assert!(rs_is_count_char(b'0' as c_int));
        assert!(rs_is_count_char(b'9' as c_int));
        assert!(!rs_is_count_char(b'a' as c_int));
    }

    #[test]
    fn test_ex_parse_count() {
        unsafe {
            let mut consumed: c_int = 0;

            assert_eq!(rs_ex_parse_count(b"123abc\0".as_ptr(), &mut consumed), 123);
            assert_eq!(consumed, 3);

            assert_eq!(rs_ex_parse_count(b"0\0".as_ptr(), &mut consumed), 0);
            assert_eq!(consumed, 1);

            assert_eq!(rs_ex_parse_count(b"abc\0".as_ptr(), &mut consumed), 0);
            assert_eq!(consumed, 0);

            assert_eq!(rs_ex_parse_count(std::ptr::null(), &mut consumed), 0);
            assert_eq!(consumed, 0);
        }
    }
}
