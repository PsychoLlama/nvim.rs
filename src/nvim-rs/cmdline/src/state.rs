//! Command line state management
//!
//! This module provides the core state structure for command-line editing,
//! including the command buffer, cursor position, and prompt configuration.

#![allow(unsafe_code)]
#![allow(clippy::doc_markdown)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::missing_const_for_fn)]

use std::ffi::{c_char, c_int, c_uint, c_void};

// =============================================================================
// Constants
// =============================================================================

/// Initial command buffer size
pub const INITIAL_CMDBUFF_SIZE: usize = 256;

/// Minimum buffer growth
pub const MIN_CMDBUFF_GROWTH: usize = 256;

/// Maximum command line recursion depth
pub const MAX_CMDLINE_LEVEL: i32 = 50;

/// NUL character constant
pub const NUL: u8 = 0;

/// Prompt type characters
pub mod prompt_type {
    /// Ex command prompt ':'
    pub const EX_CMD: u8 = b':';
    /// Forward search prompt '/'
    pub const FORWARD_SEARCH: u8 = b'/';
    /// Backward search prompt '?'
    pub const BACKWARD_SEARCH: u8 = b'?';
    /// Expression evaluation prompt '='
    pub const EXPRESSION: u8 = b'=';
    /// Debug command prompt '>'
    pub const DEBUG: u8 = b'>';
    /// Input function prompt '@'
    pub const INPUT_FN: u8 = b'@';
    /// No prompt
    pub const NONE: u8 = 0;
}

// =============================================================================
// Redraw State
// =============================================================================

/// Keeps track how much state must be sent to external UI.
#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CmdRedraw {
    /// No redraw needed
    #[default]
    None = 0,
    /// Only position changed
    Pos = 1,
    /// Full redraw needed
    All = 2,
}

impl CmdRedraw {
    /// Convert from raw C integer.
    #[must_use]
    pub const fn from_raw(value: c_int) -> Option<Self> {
        match value {
            0 => Some(Self::None),
            1 => Some(Self::Pos),
            2 => Some(Self::All),
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
// Command Line State
// =============================================================================

/// Command line state structure.
///
/// This mirrors the C `CmdlineInfo` structure and provides the core state
/// for command-line editing. The actual buffer is managed by C code;
/// this structure provides Rust-side access and manipulation functions.
#[derive(Debug)]
pub struct CmdlineState {
    /// Current cursor position in the command buffer (byte offset)
    pub cmdpos: usize,
    /// Length of the command string (in bytes)
    pub cmdlen: usize,
    /// Cursor column on screen
    pub cmdspos: usize,
    /// First character of the command line (':', '/', '?', '=', '>' or NUL)
    pub cmdfirstc: u8,
    /// Number of spaces before cmdline (indent)
    pub cmdindent: usize,
    /// Typing mode: true for overstrike, false for insert
    pub overstrike: bool,
    /// Prompt ID for this command line
    pub prompt_id: u32,
    /// Current cmdline level (for recursive calls)
    pub level: i32,
    /// Redraw state for external UI
    pub redraw_state: CmdRedraw,
    /// Whether this is an input() function call
    pub input_fn: bool,
    /// Return after one key press (for button prompt)
    pub one_key: bool,
}

impl Default for CmdlineState {
    fn default() -> Self {
        Self::new()
    }
}

impl CmdlineState {
    /// Create a new command line state with default values.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            cmdpos: 0,
            cmdlen: 0,
            cmdspos: 0,
            cmdfirstc: NUL,
            cmdindent: 0,
            overstrike: false,
            prompt_id: 0,
            level: 0,
            redraw_state: CmdRedraw::None,
            input_fn: false,
            one_key: false,
        }
    }

    /// Initialize state for a new command line.
    pub fn init(&mut self, firstc: u8, indent: usize) {
        self.overstrike = false;
        self.cmdfirstc = if firstc == b'@' { NUL } else { firstc };
        self.cmdindent = if firstc > 0 { indent } else { 0 };
        self.cmdpos = 0;
        self.cmdlen = 0;
        self.cmdspos = 0;
        self.redraw_state = CmdRedraw::None;
    }

    /// Check if cursor is at the end of the command line.
    #[must_use]
    pub const fn at_end(&self) -> bool {
        self.cmdpos >= self.cmdlen
    }

    /// Check if the command line is empty.
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.cmdlen == 0
    }

    /// Check if this is a search command.
    #[must_use]
    pub const fn is_search(&self) -> bool {
        self.cmdfirstc == prompt_type::FORWARD_SEARCH
            || self.cmdfirstc == prompt_type::BACKWARD_SEARCH
    }

    /// Check if this is an Ex command.
    #[must_use]
    pub const fn is_ex_cmd(&self) -> bool {
        self.cmdfirstc == prompt_type::EX_CMD
    }

    /// Check if this is an expression evaluation.
    #[must_use]
    pub const fn is_expression(&self) -> bool {
        self.cmdfirstc == prompt_type::EXPRESSION
    }

    /// Check if this is a debug command.
    #[must_use]
    pub const fn is_debug(&self) -> bool {
        self.cmdfirstc == prompt_type::DEBUG
    }

    /// Get the prompt character for the command line type.
    #[must_use]
    pub const fn prompt_char(&self) -> Option<char> {
        match self.cmdfirstc {
            prompt_type::EX_CMD => Some(':'),
            prompt_type::FORWARD_SEARCH => Some('/'),
            prompt_type::BACKWARD_SEARCH => Some('?'),
            prompt_type::EXPRESSION => Some('='),
            prompt_type::DEBUG => Some('>'),
            _ => None,
        }
    }
}

// =============================================================================
// C Function Declarations
// =============================================================================

#[allow(dead_code)]
extern "C" {
    static mut got_int: bool;
    static mut did_emsg: c_int;
    // Current ccline accessors
    fn nvim_get_ccline_cmdpos() -> c_int;
    fn nvim_get_ccline_cmdlen() -> c_int;
    fn nvim_get_ccline_cmdspos() -> c_int;
    fn nvim_get_ccline_cmdfirstc() -> c_int;
    fn nvim_get_ccline_cmdindent() -> c_int;
    fn nvim_get_ccline_overstrike() -> c_int;
    fn nvim_get_ccline_prompt_id() -> c_uint;
    fn nvim_get_ccline_level() -> c_int;
    fn nvim_get_ccline_input_fn() -> c_int;
    fn nvim_get_ccline_redraw_state() -> c_int;
    fn nvim_get_ccline_one_key() -> c_int;
    fn nvim_get_ccline_special_char() -> c_int;
    fn nvim_get_ccline_special_shift() -> c_int;
    fn nvim_get_ccline_hl_id() -> c_int;
    fn nvim_get_ccline_xp_context() -> c_int;
    fn nvim_get_ccline_cmdprompt() -> *mut c_char;

    fn nvim_set_ccline_cmdpos(pos: c_int);
    fn nvim_set_ccline_cmdlen(len: c_int);
    fn nvim_set_ccline_cmdspos(spos: c_int);
    fn nvim_set_ccline_overstrike(overstrike: c_int);
    fn nvim_set_ccline_redraw_state(state: c_int);
    fn nvim_set_ccline_one_key(one_key: c_int);
    fn nvim_set_ccline_special_char(c: c_int);
    fn nvim_set_ccline_special_shift(shift: c_int);
    fn nvim_set_ccline_hl_id(hl_id: c_int);
    fn nvim_set_ccline_xp_context(context: c_int);
    fn nvim_set_ccline_cmdprompt(prompt: *mut c_char);
    fn nvim_set_ccline_cmdindent(indent: c_int);
    fn nvim_set_ccline_cmdfirstc(firstc: c_int);

    // Buffer access
    fn nvim_get_ccline_cmdbuff() -> *mut c_char;
    fn nvim_get_ccline_cmdbufflen() -> c_int;
}

// =============================================================================
// FFI Functions for State Access
// =============================================================================

/// Get the current command line state from C.
///
/// # Safety
///
/// Calls C functions to access global state.
#[must_use]
pub unsafe fn get_current_state() -> CmdlineState {
    CmdlineState {
        cmdpos: nvim_get_ccline_cmdpos() as usize,
        cmdlen: nvim_get_ccline_cmdlen() as usize,
        cmdspos: nvim_get_ccline_cmdspos() as usize,
        #[allow(clippy::cast_sign_loss)]
        cmdfirstc: nvim_get_ccline_cmdfirstc() as u8,
        cmdindent: nvim_get_ccline_cmdindent() as usize,
        overstrike: nvim_get_ccline_overstrike() != 0,
        prompt_id: nvim_get_ccline_prompt_id(),
        level: nvim_get_ccline_level(),
        redraw_state: CmdRedraw::from_raw(nvim_get_ccline_redraw_state()).unwrap_or_default(),
        input_fn: nvim_get_ccline_input_fn() != 0,
        one_key: nvim_get_ccline_one_key() != 0,
    }
}

/// Update C state from Rust state.
///
/// # Safety
///
/// Calls C functions to modify global state.
pub unsafe fn set_current_state(state: &CmdlineState) {
    #[allow(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]
    {
        nvim_set_ccline_cmdpos(state.cmdpos as c_int);
        nvim_set_ccline_cmdlen(state.cmdlen as c_int);
        nvim_set_ccline_cmdspos(state.cmdspos as c_int);
        nvim_set_ccline_overstrike(c_int::from(state.overstrike));
        nvim_set_ccline_redraw_state(state.redraw_state.to_raw());
        nvim_set_ccline_one_key(c_int::from(state.one_key));
        nvim_set_ccline_cmdindent(state.cmdindent as c_int);
        nvim_set_ccline_cmdfirstc(c_int::from(state.cmdfirstc));
    }
}

/// Get a slice of the current command buffer.
///
/// # Safety
///
/// The returned slice is valid only as long as the command buffer is not
/// reallocated or freed.
#[must_use]
pub unsafe fn get_cmdbuff_slice() -> Option<&'static [u8]> {
    let ptr = nvim_get_ccline_cmdbuff();
    if ptr.is_null() {
        return None;
    }
    let len = nvim_get_ccline_cmdlen();
    if len < 0 {
        return None;
    }
    Some(std::slice::from_raw_parts(ptr.cast::<u8>(), len as usize))
}

/// Get a mutable slice of the current command buffer.
///
/// # Safety
///
/// The returned slice is valid only as long as the command buffer is not
/// reallocated or freed. Caller must ensure no other references exist.
#[must_use]
pub unsafe fn get_cmdbuff_slice_mut() -> Option<&'static mut [u8]> {
    let ptr = nvim_get_ccline_cmdbuff();
    if ptr.is_null() {
        return None;
    }
    let bufflen = nvim_get_ccline_cmdbufflen();
    if bufflen < 0 {
        return None;
    }
    Some(std::slice::from_raw_parts_mut(
        ptr.cast::<u8>(),
        bufflen as usize,
    ))
}

// =============================================================================
// FFI Exports
// =============================================================================

/// Check if command line is in overstrike mode.
///
/// # Safety
///
/// Calls external C function to access static variable.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_cmdline_state_overstrike() -> c_int {
    c_int::from(nvim_get_ccline_overstrike() != 0)
}

/// Check if cursor is at the end of the command line.
///
/// # Safety
///
/// Calls external C functions to access static variables.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_cmdline_state_at_end() -> c_int {
    let pos = nvim_get_ccline_cmdpos();
    let len = nvim_get_ccline_cmdlen();
    c_int::from(pos >= len)
}

/// Check if the command line is empty.
///
/// # Safety
///
/// Calls external C function to access static variable.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_cmdline_state_is_empty() -> c_int {
    c_int::from(nvim_get_ccline_cmdlen() == 0)
}

/// Direct C replacement for cmdline_is_empty().
///
/// # Safety
///
/// Calls external C function to access static variable.
#[must_use]
#[export_name = "cmdline_is_empty"]
pub unsafe extern "C" fn cmdline_is_empty_rs() -> bool {
    nvim_get_ccline_cmdlen() == 0
}

/// Check if the current command line is a search command.
///
/// # Safety
///
/// Calls external C function to access static variable.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_cmdline_state_is_search() -> c_int {
    let firstc = nvim_get_ccline_cmdfirstc();
    c_int::from(
        firstc == c_int::from(prompt_type::FORWARD_SEARCH)
            || firstc == c_int::from(prompt_type::BACKWARD_SEARCH),
    )
}

/// Direct C replacement for cmdline_is_search().
///
/// # Safety
///
/// Calls external C function to access static variable.
#[must_use]
#[export_name = "cmdline_is_search"]
pub unsafe extern "C" fn cmdline_is_search_rs() -> bool {
    let firstc = nvim_get_ccline_cmdfirstc();
    firstc == c_int::from(prompt_type::FORWARD_SEARCH)
        || firstc == c_int::from(prompt_type::BACKWARD_SEARCH)
}

/// Check if the current command line is an Ex command.
///
/// # Safety
///
/// Calls external C function to access static variable.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_cmdline_state_is_ex_cmd() -> c_int {
    c_int::from(nvim_get_ccline_cmdfirstc() == c_int::from(prompt_type::EX_CMD))
}

/// Direct C replacement for cmdline_is_ex_cmd().
///
/// # Safety
///
/// Calls external C function to access static variable.
#[must_use]
#[export_name = "cmdline_is_ex_cmd"]
pub unsafe extern "C" fn cmdline_is_ex_cmd_rs() -> bool {
    nvim_get_ccline_cmdfirstc() == c_int::from(prompt_type::EX_CMD)
}

/// Get the current command line level.
///
/// # Safety
///
/// Calls external C function to access static variable.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_cmdline_get_level() -> c_int {
    nvim_get_ccline_level()
}

/// Direct C replacement for cmdline_level().
///
/// # Safety
///
/// Calls external C function to access static variable.
#[must_use]
#[export_name = "cmdline_level"]
pub unsafe extern "C" fn cmdline_level_rs() -> c_int {
    nvim_get_ccline_level()
}

/// Check if command line level is at maximum.
///
/// # Safety
///
/// Calls external C function to access static variable.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_cmdline_at_max_level() -> c_int {
    c_int::from(nvim_get_ccline_level() >= MAX_CMDLINE_LEVEL)
}

/// Direct C replacement for cmdline_at_max_level().
///
/// # Safety
///
/// Calls external C function to access static variable.
#[must_use]
#[export_name = "cmdline_at_max_level"]
pub unsafe extern "C" fn cmdline_at_max_level_rs() -> bool {
    nvim_get_ccline_level() >= MAX_CMDLINE_LEVEL
}

/// Get the current cursor position in the command buffer.
///
/// # Safety
///
/// Calls external C function to access static variable.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_cmdline_get_cmdpos() -> c_int {
    nvim_get_ccline_cmdpos()
}

/// Direct C replacement for cmdline_get_pos().
///
/// # Safety
///
/// Calls external C function to access static variable.
#[must_use]
#[export_name = "cmdline_get_pos"]
pub unsafe extern "C" fn cmdline_get_pos_rs() -> c_int {
    nvim_get_ccline_cmdpos()
}

/// Get the current command line length.
///
/// # Safety
///
/// Calls external C function to access static variable.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_cmdline_get_cmdlen() -> c_int {
    nvim_get_ccline_cmdlen()
}

/// Direct C replacement for cmdline_get_len().
///
/// # Safety
///
/// Calls external C function to access static variable.
#[must_use]
#[export_name = "cmdline_get_len"]
pub unsafe extern "C" fn cmdline_get_len_rs() -> c_int {
    nvim_get_ccline_cmdlen()
}

/// Set the cursor position in the command buffer.
///
/// # Safety
///
/// Calls external C function to modify static variable.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_cmdline_set_cmdpos(pos: c_int) {
    nvim_set_ccline_cmdpos(pos);
}

/// Get the current redraw state.
///
/// # Safety
///
/// Calls external C function to access static variable.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_cmdline_get_redraw_state() -> c_int {
    nvim_get_ccline_redraw_state()
}

/// Set the redraw state.
///
/// # Safety
///
/// Calls external C function to modify static variable.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_cmdline_set_redraw_state(state: c_int) {
    nvim_set_ccline_redraw_state(state);
}

/// Get the one_key flag.
///
/// # Safety
///
/// Calls external C function to access static variable.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_cmdline_get_one_key() -> c_int {
    nvim_get_ccline_one_key()
}

/// Set the one_key flag.
///
/// # Safety
///
/// Calls external C function to modify static variable.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_cmdline_set_one_key(one_key: c_int) {
    nvim_set_ccline_one_key(one_key);
}

/// Get the highlight ID for the command line prompt.
///
/// # Safety
///
/// Calls external C function to access static variable.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_cmdline_get_hl_id() -> c_int {
    nvim_get_ccline_hl_id()
}

/// Set the highlight ID for the command line prompt.
///
/// # Safety
///
/// Calls external C function to modify static variable.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_cmdline_set_hl_id(hl_id: c_int) {
    nvim_set_ccline_hl_id(hl_id);
}

/// Get the expansion context type.
///
/// # Safety
///
/// Calls external C function to access static variable.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_cmdline_get_xp_context() -> c_int {
    nvim_get_ccline_xp_context()
}

/// Set the expansion context type.
///
/// # Safety
///
/// Calls external C function to modify static variable.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_cmdline_set_xp_context(context: c_int) {
    nvim_set_ccline_xp_context(context);
}

/// Get the special character for redraws.
///
/// # Safety
///
/// Calls external C function to access static variable.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_cmdline_get_special_char() -> c_int {
    nvim_get_ccline_special_char()
}

/// Set the special character for redraws.
///
/// # Safety
///
/// Calls external C function to modify static variable.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_cmdline_set_special_char(c: c_int) {
    nvim_set_ccline_special_char(c);
}

/// Get the special shift flag.
///
/// # Safety
///
/// Calls external C function to access static variable.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_cmdline_get_special_shift() -> c_int {
    nvim_get_ccline_special_shift()
}

/// Set the special shift flag.
///
/// # Safety
///
/// Calls external C function to modify static variable.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_cmdline_set_special_shift(shift: c_int) {
    nvim_set_ccline_special_shift(shift);
}

/// Get the command line indent.
///
/// # Safety
///
/// Calls external C function to access static variable.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_cmdline_get_cmdindent() -> c_int {
    nvim_get_ccline_cmdindent()
}

/// Set the command line indent.
///
/// # Safety
///
/// Calls external C function to modify static variable.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_cmdline_set_cmdindent(indent: c_int) {
    nvim_set_ccline_cmdindent(indent);
}

/// Get the first character of the command line.
///
/// # Safety
///
/// Calls external C function to access static variable.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_cmdline_get_cmdfirstc() -> c_int {
    nvim_get_ccline_cmdfirstc()
}

/// Set the first character of the command line.
///
/// # Safety
///
/// Calls external C function to modify static variable.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_cmdline_set_cmdfirstc(firstc: c_int) {
    nvim_set_ccline_cmdfirstc(firstc);
}

// =============================================================================
// Buffer Position Utilities
// =============================================================================

/// Calculate the byte offset of a character position in UTF-8 text.
///
/// This is useful for moving by characters rather than bytes.
#[must_use]
pub fn char_offset_to_byte(text: &[u8], char_offset: usize) -> usize {
    let mut byte_pos = 0;
    let mut char_count = 0;

    while byte_pos < text.len() && char_count < char_offset {
        let c = text[byte_pos];
        // Count bytes in this UTF-8 character
        let char_len = if c < 0x80 {
            1
        } else if c < 0xE0 {
            2
        } else if c < 0xF0 {
            3
        } else {
            4
        };
        byte_pos += char_len.min(text.len() - byte_pos);
        char_count += 1;
    }

    byte_pos
}

/// Calculate the character offset of a byte position in UTF-8 text.
#[must_use]
pub fn byte_offset_to_char(text: &[u8], byte_offset: usize) -> usize {
    let mut byte_pos = 0;
    let mut char_count = 0;

    while byte_pos < byte_offset && byte_pos < text.len() {
        let c = text[byte_pos];
        let char_len = if c < 0x80 {
            1
        } else if c < 0xE0 {
            2
        } else if c < 0xF0 {
            3
        } else {
            4
        };
        byte_pos += char_len.min(text.len() - byte_pos);
        char_count += 1;
    }

    char_count
}

/// Get the byte length of the UTF-8 character at the given position.
#[must_use]
pub fn utf8_char_len(text: &[u8], pos: usize) -> usize {
    if pos >= text.len() {
        return 0;
    }
    let c = text[pos];
    if c < 0x80 {
        1
    } else if c < 0xE0 {
        2
    } else if c < 0xF0 {
        3
    } else {
        4
    }
}

/// Move to the previous character position in UTF-8 text.
///
/// Returns the new byte position.
#[must_use]
pub fn prev_char_pos(text: &[u8], pos: usize) -> usize {
    if pos == 0 || text.is_empty() {
        return 0;
    }

    let mut new_pos = pos - 1;
    // Skip continuation bytes (10xxxxxx)
    while new_pos > 0 && (text[new_pos] & 0xC0) == 0x80 {
        new_pos -= 1;
    }
    new_pos
}

/// Move to the next character position in UTF-8 text.
///
/// Returns the new byte position.
#[must_use]
pub fn next_char_pos(text: &[u8], pos: usize) -> usize {
    if pos >= text.len() {
        return text.len();
    }
    pos + utf8_char_len(text, pos)
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cmdline_state_new() {
        let state = CmdlineState::new();
        assert_eq!(state.cmdpos, 0);
        assert_eq!(state.cmdlen, 0);
        assert!(!state.overstrike);
        assert_eq!(state.cmdfirstc, NUL);
    }

    #[test]
    fn test_cmdline_state_init() {
        let mut state = CmdlineState::new();
        state.init(b':', 4);
        assert_eq!(state.cmdfirstc, b':');
        assert_eq!(state.cmdindent, 4);
        assert!(!state.overstrike);

        // '@' becomes NUL for cmdfirstc, but indent is still set because '@' > 0
        state.init(b'@', 2);
        assert_eq!(state.cmdfirstc, NUL);
        assert_eq!(state.cmdindent, 2);

        // When firstc is 0, cmdindent should be 0 regardless of indent
        state.init(0, 5);
        assert_eq!(state.cmdfirstc, 0);
        assert_eq!(state.cmdindent, 0);
    }

    #[test]
    fn test_cmdline_state_at_end() {
        let mut state = CmdlineState::new();
        state.cmdlen = 10;
        state.cmdpos = 5;
        assert!(!state.at_end());

        state.cmdpos = 10;
        assert!(state.at_end());

        state.cmdpos = 15;
        assert!(state.at_end());
    }

    #[test]
    fn test_cmdline_state_is_search() {
        let mut state = CmdlineState::new();

        state.cmdfirstc = b'/';
        assert!(state.is_search());

        state.cmdfirstc = b'?';
        assert!(state.is_search());

        state.cmdfirstc = b':';
        assert!(!state.is_search());
    }

    #[test]
    fn test_cmdline_state_prompt_char() {
        let mut state = CmdlineState::new();

        state.cmdfirstc = b':';
        assert_eq!(state.prompt_char(), Some(':'));

        state.cmdfirstc = b'/';
        assert_eq!(state.prompt_char(), Some('/'));

        state.cmdfirstc = NUL;
        assert_eq!(state.prompt_char(), None);
    }

    #[test]
    fn test_cmd_redraw() {
        assert_eq!(CmdRedraw::from_raw(0), Some(CmdRedraw::None));
        assert_eq!(CmdRedraw::from_raw(1), Some(CmdRedraw::Pos));
        assert_eq!(CmdRedraw::from_raw(2), Some(CmdRedraw::All));
        assert_eq!(CmdRedraw::from_raw(3), None);

        assert_eq!(CmdRedraw::None.to_raw(), 0);
        assert_eq!(CmdRedraw::Pos.to_raw(), 1);
        assert_eq!(CmdRedraw::All.to_raw(), 2);
    }

    #[test]
    fn test_char_offset_to_byte() {
        // ASCII
        assert_eq!(char_offset_to_byte(b"hello", 0), 0);
        assert_eq!(char_offset_to_byte(b"hello", 2), 2);
        assert_eq!(char_offset_to_byte(b"hello", 5), 5);

        // UTF-8: "héllo" (é is 2 bytes)
        let text = "héllo".as_bytes();
        assert_eq!(char_offset_to_byte(text, 0), 0);
        assert_eq!(char_offset_to_byte(text, 1), 1); // 'h'
        assert_eq!(char_offset_to_byte(text, 2), 3); // after 'é' (2 bytes)
        assert_eq!(char_offset_to_byte(text, 3), 4); // 'l'
    }

    #[test]
    fn test_byte_offset_to_char() {
        // ASCII
        assert_eq!(byte_offset_to_char(b"hello", 0), 0);
        assert_eq!(byte_offset_to_char(b"hello", 2), 2);

        // UTF-8: "héllo"
        let text = "héllo".as_bytes();
        assert_eq!(byte_offset_to_char(text, 0), 0);
        assert_eq!(byte_offset_to_char(text, 1), 1); // 'h'
        assert_eq!(byte_offset_to_char(text, 3), 2); // after 'é'
        assert_eq!(byte_offset_to_char(text, 4), 3); // 'l'
    }

    #[test]
    fn test_utf8_char_len() {
        // ASCII
        assert_eq!(utf8_char_len(b"hello", 0), 1);

        // 2-byte UTF-8 (é)
        let text = "é".as_bytes();
        assert_eq!(utf8_char_len(text, 0), 2);

        // 3-byte UTF-8 (€)
        let text = "€".as_bytes();
        assert_eq!(utf8_char_len(text, 0), 3);

        // 4-byte UTF-8 (emoji)
        let text = "😀".as_bytes();
        assert_eq!(utf8_char_len(text, 0), 4);

        // Empty
        assert_eq!(utf8_char_len(b"", 0), 0);
    }

    #[test]
    fn test_prev_char_pos() {
        // ASCII
        assert_eq!(prev_char_pos(b"hello", 3), 2);
        assert_eq!(prev_char_pos(b"hello", 1), 0);
        assert_eq!(prev_char_pos(b"hello", 0), 0);

        // UTF-8: "héllo"
        let text = "héllo".as_bytes();
        assert_eq!(prev_char_pos(text, 3), 1); // Before 'é' -> after 'h'
        assert_eq!(prev_char_pos(text, 1), 0); // Before 'é' first byte
    }

    #[test]
    fn test_next_char_pos() {
        // ASCII
        assert_eq!(next_char_pos(b"hello", 0), 1);
        assert_eq!(next_char_pos(b"hello", 4), 5);
        assert_eq!(next_char_pos(b"hello", 5), 5);

        // UTF-8: "héllo"
        let text = "héllo".as_bytes();
        assert_eq!(next_char_pos(text, 0), 1); // 'h' -> 'é'
        assert_eq!(next_char_pos(text, 1), 3); // 'é' (2 bytes) -> 'l'
    }
}

// =============================================================================
// Phase 4: command_line_wildchar_complete and command_line_execute
// =============================================================================

// WimFlag constants (from auto/option_vars.generated.h)
const WIM_FLAG_FULL: u8 = 0x01;
const WIM_FLAG_LONGEST: u8 = 0x02;
const WIM_FLAG_LIST: u8 = 0x04;
const WIM_FLAG_LASTUSED: u8 = 0x08;
const WIM_FLAG_NOSELECT: u8 = 0x10;

// Cmdline result constants (matching C enum)
const CMDLINE_NOT_CHANGED: c_int = 1;
const CMDLINE_CHANGED: c_int = 2;

// Key constants (re-exported from keys module for use in this module)
use crate::keys::{
    K_CMDWIN, K_COMMAND, K_DOWN, K_EVENT, K_IGNORE, K_KENTER, K_KPAGEDOWN, K_KPAGEUP, K_LEFT,
    K_LUA, K_NOP, K_PAGEDOWN, K_PAGEUP, K_RIGHT, K_S_DOWN, K_S_TAB, K_S_UP, K_UP, K_WILD, K_ZERO,
};

const ESC: c_int = 0x1b;
const CTRL_C: c_int = 0x03;
const CTRL_P: c_int = 0x10;
const CTRL_N: c_int = 0x0e;
const CTRL_E: c_int = 0x05;
const CTRL_Y: c_int = 0x19;
const CTRL_Z: c_int = 0x1a;
const CTRL_BSL: c_int = 0x1c;
const NUL_INT: c_int = 0;
const NL: c_int = 10;

// Wild mode constants
const WILD_FREE: c_int = crate::expand::wild_mode::WILD_FREE;
const WILD_LONGEST: c_int = crate::expand::wild_mode::WILD_LONGEST;
const WILD_EXPAND_KEEP: c_int = crate::expand::wild_mode::WILD_EXPAND_KEEP;
const WILD_NEXT: c_int = crate::expand::wild_mode::WILD_NEXT;
const WILD_PREV: c_int = crate::expand::wild_mode::WILD_PREV;
const WILD_PUM_WANT: c_int = crate::expand::wild_mode::WILD_PUM_WANT;
const WILD_APPLY: c_int = crate::expand::wild_mode::WILD_APPLY;
const WILD_CANCEL: c_int = crate::expand::wild_mode::WILD_CANCEL;

// Wild option flags
const WILD_NO_BEEP: c_int = crate::expand::wild_flags::WILD_NO_BEEP;
const WILD_BUFLASTUSED: c_int = crate::expand::wild_flags::WILD_BUFLASTUSED;
const WILD_NOSELECT: c_int = crate::expand::wild_flags::WILD_NOSELECT;
const WILD_MAY_EXPAND_PATTERN: c_int = crate::expand::wild_flags::WILD_MAY_EXPAND_PATTERN;
const WILD_FUNC_TRIGGER: c_int = crate::expand::wild_flags::WILD_FUNC_TRIGGER;

// EXPAND_NOTHING constant
const EXPAND_NOTHING: c_int = 0;

// OK/FAIL constants
const OK: c_int = 1;

// kOptBoFlagWildmode constant (from option_vars.generated.h)
const K_OPT_BO_FLAG_WILDMODE: c_int = 0x1;

// kUICmdline (24)
const K_UI_CMDLINE: c_int = 24;

unsafe extern "C" {
    // Autocmd trigger (now implemented in Rust as rs_trigger_cmd_autocmd)
    fn xfree(ptr: *mut c_char);
    fn xstrdup(s: *const c_char) -> *mut c_char;

    // v:char setter
    fn set_vim_var_char(c: c_int);

    // curwin handle for incsearch state reset
    fn nvim_get_curwin_handle() -> c_int;

    // Direct wildmenu functions (from cmdexpand crate)
    fn wildmenu_translate_key(
        cclp: *mut c_void,
        key: c_int,
        xp: *mut c_void,
        did_wild_list: bool,
    ) -> c_int;
    fn wildmenu_process_key(cclp: *mut c_void, key: c_int, xp: *mut c_void) -> c_int;
    // nvim_command_line_end_wildmenu defined in Rust below

    // nvim_command_line_not_changed and nvim_command_line_changed are defined in Rust below

    // Direct C expansion functions (replaces nvim_nextwild/nvim_showmatches wrappers)
    fn nextwild(xp: *mut c_void, wild_type: c_int, options: c_int, escape: bool) -> c_int;
    fn showmatches(
        xp: *mut c_void,
        display_wildmenu: bool,
        display_list: bool,
        noselect: bool,
    ) -> c_int;

    // Global accessors
    fn nvim_get_wim_flags(idx: c_int) -> u8;
    static p_wc: i64;
    static p_wcm: i64;
    static p_wmnu: c_int;
    static mut KeyTyped: bool;
    static mut cmdmsg_rl: bool;
    static KeyStuffed: c_int;
    static cmd_silent: bool;
    fn nvim_get_global_busy() -> bool;
    static ex_normal_busy: c_int;
    static mut exmode_active: bool;
    fn nvim_get_cedit_key() -> c_int;
    fn nvim_open_cmdwin() -> c_int;
    fn nvim_get_pum_want_active() -> c_int;
    fn nvim_get_pum_want_finish() -> c_int;
    fn nvim_set_pum_want_active(val: c_int);
    static mut wild_menu_showing: c_int;
    fn nvim_get_cmdline_was_last_drawn() -> c_int;
    fn nvim_syn_get_display_tick() -> c_int;
    static mut emsg_silent: c_int;

    // C functions
    fn do_digraph(c: c_int) -> c_int;
    fn pum_check_clear();
    fn vpeekc() -> c_int;
    fn ExpandOne(
        xp: *mut c_void,
        str_: *mut c_char,
        orig: *mut c_char,
        options: c_int,
        mode: c_int,
    ) -> *mut c_char;
    fn vim_beep(flag: c_int);
    fn set_no_hlsearch(flag: c_int);
    fn state_handle_k_event();
    #[link_name = "map_execute_lua"]
    fn map_execute_lua_direct(may_repeat: bool, discard: bool) -> bool;
    fn do_cmdline(
        cmdp: *const c_char,
        getline: Option<unsafe extern "C" fn(c_int, *mut c_void, c_int, bool) -> *mut c_char>,
        cookie: *mut c_void,
        flags: c_int,
    ) -> c_int;
    fn getcmdkeycmd(c: c_int, cookie: *mut c_void, indent: c_int, do_concat: bool) -> *mut c_char;
    fn msg_cursor_goto(row: c_int, col: c_int);
    fn ui_flush();
    fn ui_has(what: c_int) -> c_int;
    static mut msg_row: c_int;
    fn vim_strchr(haystack: *const c_char, needle: c_int) -> *mut c_char;
    fn nvim_get_p_cpo() -> *const c_char;
    fn cmdline_pum_active() -> c_int;

    static mut redir_off: bool;
    static mut quit_more: bool;
    fn nvim_get_typebuf_len() -> c_int;
    fn stuff_empty() -> bool;
    fn may_trigger_safestate(pending: bool);
    fn cursorcmd();
    fn ui_cursor_shape();
}

// CPO_ESC is 'x' in C's cpoptions
const CPO_ESC_CHAR: c_int = b'x' as c_int;

// Autocmd event constants (from auevents_enum.generated.h)
const EVENT_CMDLINELEAVEPRE: c_int = 28;

/// Rust replacement for `command_line_wildchar_complete(CommandLineState *s)`.
///
/// Handles wildchar completion in command-line mode.
///
/// # Safety
///
/// `s` must be a valid non-null pointer to a `CommandLineState`.
#[allow(clippy::too_many_lines)]
#[allow(clippy::cognitive_complexity)]
#[allow(clippy::many_single_char_names)]
#[allow(clippy::similar_names)]
#[unsafe(export_name = "command_line_wildchar_complete")]
pub unsafe extern "C" fn rs_command_line_wildchar_complete(s: *mut c_void) -> c_int {
    let s = s.cast::<crate::command_line_state::CommandLineState>();
    let mut options: c_int = WILD_NO_BEEP;
    let c = (*s).c;
    let firstc = (*s).firstc;
    let escape = firstc != b'@' as c_int;
    let redraw_if_menu_empty = c == K_WILD;
    let p_wmnu_set = p_wmnu != 0;
    let wim_noselect = p_wmnu_set && (nvim_get_wim_flags(0) & WIM_FLAG_NOSELECT) != 0;

    let wim_index = (*s).wim_index;
    if (nvim_get_wim_flags(wim_index) & WIM_FLAG_LASTUSED) != 0 {
        options |= WILD_BUFLASTUSED;
    }

    let xpc_numfiles = (*s).xpc.xp_numfiles;
    let xp = std::ptr::addr_of_mut!((*s).xpc).cast::<c_void>();

    let res: c_int;
    if xpc_numfiles > 0 {
        // typed p_wc at least twice
        // If "list" is present, list matches unless already listed
        if xpc_numfiles > 1
            && !(*s).did_wild_list
            && (nvim_get_wim_flags(wim_index) & WIM_FLAG_LIST) != 0
        {
            showmatches(xp, false, true, wim_noselect);
            crate::screen::redrawcmd_rs();
            (*s).did_wild_list = true;
        }
        if (nvim_get_wim_flags(wim_index) & WIM_FLAG_LONGEST) != 0 {
            res = nextwild(xp, WILD_LONGEST, options, escape);
        } else if (nvim_get_wim_flags(wim_index) & WIM_FLAG_FULL) != 0 {
            res = nextwild(xp, WILD_NEXT, options, escape);
        } else {
            res = OK; // don't insert 'wildchar' now
        }
    } else {
        // typed p_wc first time
        let wim_longest = (nvim_get_wim_flags(0) & WIM_FLAG_LONGEST) != 0;
        let wim_list = (nvim_get_wim_flags(0) & WIM_FLAG_LIST) != 0;
        let wim_full = (nvim_get_wim_flags(0) & WIM_FLAG_FULL) != 0;

        (*s).wim_index = 0;
        let wc = p_wc as c_int;
        let wcm = p_wcm as c_int;
        if c == wc || c == wcm || c == K_WILD || c == CTRL_Z {
            options |= WILD_MAY_EXPAND_PATTERN;
            if c == K_WILD {
                options |= WILD_FUNC_TRIGGER;
            }
            // Inlined nvim_cls_set_xpc_pre_incsearch_from_is_state
            {
                let is_state_ptr = std::ptr::addr_of_mut!((*s).is_state);
                let xp_ptr = std::ptr::addr_of_mut!((*s).xpc);
                (*xp_ptr).xp_pre_incsearch_pos.lnum = (*is_state_ptr).search_start.lnum;
                (*xp_ptr).xp_pre_incsearch_pos.col = (*is_state_ptr).search_start.col;
                (*xp_ptr).xp_pre_incsearch_pos.coladd = (*is_state_ptr).search_start.coladd;
            }
        }
        let cmdpos_before = nvim_get_ccline_cmdpos();

        // if 'wildmode' first contains "longest", get longest common part
        if wim_longest {
            res = nextwild(xp, WILD_LONGEST, options, escape);
        } else {
            let mut opts = options;
            if wim_noselect || wim_list {
                opts |= WILD_NOSELECT;
            }
            res = nextwild(xp, WILD_EXPAND_KEEP, opts, escape);
        }

        // Remove popup menu if no completion items are available
        if redraw_if_menu_empty && (*s).xpc.xp_numfiles <= 0 {
            pum_check_clear();
        }

        // if interrupted while completing, behave like it failed
        if unsafe { got_int } {
            vpeekc(); // remove <C-C> from input stream
            unsafe {
                got_int = false;
            } // don't abandon the command line
            ExpandOne(xp, std::ptr::null_mut(), std::ptr::null_mut(), 0, WILD_FREE);
            (*s).xpc.xp_context = EXPAND_NOTHING;
            return CMDLINE_CHANGED;
        }

        // Display matches
        let xpc_numfiles2 = (*s).xpc.xp_numfiles;
        let threshold = i32::from(!wim_noselect);
        if res == OK && xpc_numfiles2 > threshold {
            if wim_longest {
                let found_longest_prefix = nvim_get_ccline_cmdpos() != cmdpos_before;
                let wim_list_check = wim_list || (p_wmnu_set && wim_full);
                if wim_list_check {
                    showmatches(xp, p_wmnu_set, wim_list, true);
                } else if !found_longest_prefix {
                    let wim_list_next = (nvim_get_wim_flags(1) & WIM_FLAG_LIST) != 0;
                    let wim_full_next = (nvim_get_wim_flags(1) & WIM_FLAG_FULL) != 0;
                    let wim_noselect_next = (nvim_get_wim_flags(1) & WIM_FLAG_NOSELECT) != 0;
                    if wim_list_next || (p_wmnu_set && (wim_full_next || wim_noselect_next)) {
                        if wim_full_next && !wim_noselect_next {
                            nextwild(xp, WILD_NEXT, options, escape);
                        } else {
                            showmatches(xp, p_wmnu_set, wim_list_next, wim_noselect_next);
                        }
                        if wim_list_next {
                            (*s).did_wild_list = true;
                        }
                    }
                }
            } else {
                let wim_list2 = wim_list || (p_wmnu_set && (wim_full || wim_noselect));
                if wim_list2 {
                    showmatches(xp, p_wmnu_set, wim_list, wim_noselect);
                } else {
                    vim_beep(K_OPT_BO_FLAG_WILDMODE);
                }
            }
            crate::screen::redrawcmd_rs();
            if wim_list {
                (*s).did_wild_list = true;
            }
        } else if (*s).xpc.xp_numfiles == -1 {
            (*s).xpc.xp_context = EXPAND_NOTHING;
        }
    }

    let wim_index_new = (*s).wim_index;
    if wim_index_new < 3 {
        (*s).wim_index = wim_index_new + 1;
    }

    if c == ESC {
        (*s).gotesc = true;
    }

    if res == OK {
        CMDLINE_CHANGED
    } else {
        CMDLINE_NOT_CHANGED
    }
}

/// Rust replacement for `command_line_execute(VimState *state, int key)`.
///
/// Handles a key in command-line mode (VimState execute callback).
///
/// # Safety
///
/// `state` must be a valid non-null pointer to a `CommandLineState`.
#[allow(clippy::too_many_lines)]
#[allow(clippy::cognitive_complexity)]
#[allow(clippy::many_single_char_names)]
#[allow(clippy::similar_names)]
#[unsafe(export_name = "command_line_execute")]
pub unsafe extern "C" fn rs_command_line_execute(state: *mut c_void, key: c_int) -> c_int {
    let s = state.cast::<crate::command_line_state::CommandLineState>();
    if key == K_IGNORE || key == K_NOP {
        return -1; // get another key
    }

    let display_tick_saved = nvim_syn_get_display_tick();
    (*s).c = key;

    // Skip wildmenu during history navigation via Up/Down keys
    if (*s).c == K_WILD && (*s).did_hist_navigate {
        (*s).did_hist_navigate = false;
        return 1;
    }

    let c = (*s).c;
    if c == K_EVENT || c == K_COMMAND || c == K_LUA {
        if c == K_EVENT {
            state_handle_k_event();
        } else if c == K_COMMAND {
            const DOCMD_NOWAIT: c_int = 0x02;
            do_cmdline(
                std::ptr::null(),
                Some(getcmdkeycmd),
                std::ptr::null_mut(),
                DOCMD_NOWAIT,
            );
        } else {
            map_execute_lua_direct(false, false);
        }
        // If the window changed incremental search state is not valid.
        // Inlined nvim_cls_maybe_reset_incsearch_state
        {
            let is_state_ptr = std::ptr::addr_of_mut!((*s).is_state);
            if (*is_state_ptr).winid != nvim_get_curwin_handle() {
                crate::search::rs_init_incsearch_state(is_state_ptr);
            }
        }
        // Re-apply 'incsearch' highlighting in case it was cleared.
        if nvim_syn_get_display_tick() > display_tick_saved && (*s).is_state.did_incsearch {
            // Inlined nvim_cls_may_do_incsearch
            {
                let firstc = (*s).firstc;
                let count = (*s).count;
                let is_state_ptr = std::ptr::addr_of_mut!((*s).is_state);
                crate::search::rs_may_do_incsearch_highlighting(firstc, count, is_state_ptr);
            }
        }

        // nvim_select_popupmenu_item() can be called from K_EVENT/K_COMMAND/K_LUA
        if nvim_get_pum_want_active() != 0 {
            if cmdline_pum_active() != 0 {
                let firstc = (*s).firstc;
                let xp = std::ptr::addr_of_mut!((*s).xpc).cast::<c_void>();
                nextwild(xp, WILD_PUM_WANT, 0, firstc != b'@' as c_int);
                if nvim_get_pum_want_finish() != 0 {
                    nextwild(xp, WILD_APPLY, WILD_NO_BEEP, firstc != b'@' as c_int);
                    nvim_command_line_end_wildmenu(s.cast::<c_void>(), false);
                }
            }
            nvim_set_pum_want_active(0);
        }

        if nvim_get_cmdline_was_last_drawn() == 0 {
            crate::screen::rs_redrawcmdline();
        }
        return 1;
    }

    if KeyTyped {
        (*s).some_key_typed = true;

        if cmdmsg_rl && KeyStuffed == 0 {
            // Invert horizontal movements and operations. Only when typed by user
            // directly, not when the result of a mapping.
            let c_curr = (*s).c;
            (*s).c = crate::keys::invert_rtl_key(c_curr);
        }
    }

    // Ignore got_int when CTRL-C was typed here.
    // Don't ignore it in :global, we really need to break then.
    // Don't ignore it for the input() function.
    {
        let c_curr = (*s).c;
        let firstc = (*s).firstc;
        let break_ctrl_c = (*s).break_ctrl_c;
        let exmode = exmode_active;
        if c_curr == CTRL_C
            && firstc != b'@' as c_int
            && (!break_ctrl_c || exmode)
            && !nvim_get_global_busy()
        {
            unsafe {
                got_int = false;
            }
        }
    }

    // free old command line when finished moving around in the history list
    {
        let c_curr = (*s).c;
        let xpc_numfiles = (*s).xpc.xp_numfiles;
        if !(*s).lookfor.is_null()
            && c_curr != K_S_DOWN
            && c_curr != K_S_UP
            && c_curr != K_DOWN
            && c_curr != K_UP
            && c_curr != K_PAGEDOWN
            && c_curr != K_PAGEUP
            && c_curr != K_KPAGEDOWN
            && c_curr != K_KPAGEUP
            && c_curr != K_LEFT
            && c_curr != K_RIGHT
            && (xpc_numfiles > 0 || (c_curr != CTRL_P && c_curr != CTRL_N))
        {
            xfree((*s).lookfor);
            (*s).lookfor = std::ptr::null_mut();
            (*s).lookforlen = 0;
        }
    }

    // When there are matching completions to select <S-Tab> works like CTRL-P
    {
        let c_curr = (*s).c;
        let wc = p_wc as c_int;
        if crate::keys::rs_is_stab_to_ctrl_p(c_curr, wc) != 0 && (*s).xpc.xp_numfiles > 0 {
            (*s).c = CTRL_P;
        }
    }

    if p_wmnu != 0 {
        let xp = std::ptr::addr_of_mut!((*s).xpc).cast::<c_void>();
        let c_new = wildmenu_translate_key(std::ptr::null_mut(), (*s).c, xp, (*s).did_wild_list);
        (*s).c = c_new;
    }

    let c_curr = (*s).c;
    let wc = p_wc as c_int;
    let wcm = p_wcm as c_int;
    let key_is_wc = (c_curr == wc && KeyTyped) || c_curr == wcm;

    let mut wild_type = 0_i32;
    if (cmdline_pum_active() != 0 || wild_menu_showing != 0 || (*s).did_wild_list) && !key_is_wc {
        let c_check = (*s).c;
        if c_check == CTRL_E || c_check == CTRL_Y {
            wild_type = if c_check == CTRL_E {
                WILD_CANCEL
            } else {
                WILD_APPLY
            };
            let xp = std::ptr::addr_of_mut!((*s).xpc).cast::<c_void>();
            let firstc = (*s).firstc;
            nextwild(xp, wild_type, WILD_NO_BEEP, firstc != b'@' as c_int);
        }
    }

    // Trigger CmdlineLeavePre autocommand
    {
        let c_check = (*s).c;
        if (KeyTyped
            && (c_check == b'\n' as c_int
                || c_check == b'\r' as c_int
                || c_check == K_KENTER
                || c_check == ESC))
            || c_check == CTRL_C
        {
            // Inlined nvim_cls_trigger_cmdlineleavepre
            {
                let c_val = (*s).c;
                set_vim_var_char(c_val);
                let cmdline_type = crate::entry::rs_entry_cmdline_type((*s).firstc);
                rs_trigger_cmd_autocmd(cmdline_type, EVENT_CMDLINELEAVEPRE);
            }
            (*s).event_cmdlineleavepre_triggered = true;
            if (c_check == ESC || c_check == CTRL_C) && (nvim_get_wim_flags(0) & WIM_FLAG_LIST) != 0
            {
                set_no_hlsearch(1);
            }
        }
    }

    // The wildmenu is cleared if the pressed key is not used for navigating
    let c_check = (*s).c;
    let end_wildmenu = !key_is_wc
        && crate::keys::rs_should_end_wildmenu(c_check, p_wc as c_int, p_wcm as c_int) != 0;
    let end_wildmenu = end_wildmenu
        && (cmdline_pum_active() == 0 || crate::keys::rs_should_end_wildmenu_pum(c_check) != 0);

    // free expanded names when finished walking through matches
    if end_wildmenu {
        nvim_command_line_end_wildmenu(s.cast::<c_void>(), key_is_wc);
    }

    if p_wmnu != 0 {
        let xp = std::ptr::addr_of_mut!((*s).xpc).cast::<c_void>();
        let c_new = wildmenu_process_key(std::ptr::null_mut(), (*s).c, xp);
        (*s).c = c_new;
    }

    // CTRL-\ handling
    {
        let c_check = (*s).c;
        if c_check == CTRL_BSL {
            let mut c_val = (*s).c;
            let mut gotesc_val = (*s).gotesc;
            match crate::keys::rs_command_line_handle_ctrl_bsl(&raw mut c_val, &raw mut gotesc_val)
            {
                x if x == CMDLINE_CHANGED => {
                    (*s).c = c_val;
                    (*s).gotesc = gotesc_val;
                    return nvim_command_line_changed(s.cast::<c_void>());
                }
                x if x == CMDLINE_NOT_CHANGED => {
                    (*s).c = c_val;
                    (*s).gotesc = gotesc_val;
                    return nvim_command_line_not_changed(s.cast::<c_void>());
                }
                0 => {
                    (*s).c = c_val;
                    (*s).gotesc = gotesc_val;
                    return 0; // back to cmd mode
                }
                _ => {
                    // backslash key not processed
                    (*s).c = CTRL_BSL;
                }
            }
        }
    }

    // Handle cedit_key or K_CMDWIN
    {
        let c_check = (*s).c;
        let cedit_key = nvim_get_cedit_key();
        if c_check == cedit_key || c_check == K_CMDWIN {
            if (c_check == K_CMDWIN || ex_normal_busy == 0) && !unsafe { got_int } {
                let c_new = nvim_open_cmdwin();
                (*s).c = c_new;
                (*s).some_key_typed = true;
            }
        } else {
            let c_check = (*s).c;
            let c_new = do_digraph(c_check);
            (*s).c = c_new;
        }
    }

    // Handle Enter/ESC
    {
        let c_check = (*s).c;
        if c_check == b'\n' as c_int
            || c_check == b'\r' as c_int
            || c_check == K_KENTER
            || (c_check == ESC
                && (!KeyTyped || !vim_strchr(nvim_get_p_cpo(), CPO_ESC_CHAR).is_null()))
        {
            let exmode = exmode_active;
            let cmdpos = nvim_get_ccline_cmdpos();
            let cmdlen = nvim_get_ccline_cmdlen();
            // In Ex mode a backslash escapes a newline
            if exmode
                && c_check != ESC
                && cmdpos == cmdlen
                && cmdpos > 0
                && unsafe { *nvim_get_ccline_cmdbuff().add(cmdpos as usize - 1) } == b'\\' as c_char
            {
                if c_check == K_KENTER {
                    (*s).c = b'\n' as c_int;
                }
                // fall through: don't return, just continue with the char
            } else {
                (*s).gotesc = false;
                if crate::edit::rs_ccheck_abbr((*s).c + 0x100) != 0 {
                    return nvim_command_line_changed(s.cast::<c_void>());
                }
                if !cmd_silent {
                    if ui_has(K_UI_CMDLINE) == 0 {
                        msg_cursor_goto(msg_row, 0);
                    }
                    ui_flush();
                }
                return 0;
            }
        }
    }

    // Completion for 'wildchar', 'wildcharm', and wildtrigger()
    {
        let c_check = (*s).c;
        let wc = p_wc as c_int;
        let wcm = p_wcm as c_int;
        if (c_check == wc && !(*s).gotesc && KeyTyped)
            || c_check == wcm
            || c_check == K_WILD
            || c_check == CTRL_Z
        {
            if c_check == K_WILD {
                emsg_silent += 1; // silence the bell
            }
            let res = rs_command_line_wildchar_complete(s.cast::<c_void>());
            if c_check == K_WILD {
                emsg_silent -= 1;
            }
            if res == CMDLINE_CHANGED {
                return nvim_command_line_changed(s.cast::<c_void>());
            }
            if c_check == K_WILD {
                return nvim_command_line_not_changed(s.cast::<c_void>());
            }
        }
    }

    (*s).gotesc = false;

    // <S-Tab> goes to last match, in a clumsy way
    {
        let c_check = (*s).c;
        if c_check == K_S_TAB && KeyTyped {
            let xp = std::ptr::addr_of_mut!((*s).xpc).cast::<c_void>();
            let firstc = (*s).firstc;
            if nextwild(xp, WILD_EXPAND_KEEP, 0, firstc != b'@' as c_int) == OK {
                let numfiles = (*s).xpc.xp_numfiles;
                let wim_idx = (*s).wim_index;
                if numfiles > 1
                    && ((!(*s).did_wild_list && (nvim_get_wim_flags(wim_idx) & WIM_FLAG_LIST) != 0)
                        || p_wmnu != 0)
                {
                    showmatches(
                        xp,
                        p_wmnu != 0,
                        (nvim_get_wim_flags(wim_idx) & WIM_FLAG_LIST) != 0,
                        (nvim_get_wim_flags(0) & WIM_FLAG_NOSELECT) != 0,
                    );
                }
                nextwild(xp, WILD_PREV, 0, firstc != b'@' as c_int);
                nextwild(xp, WILD_PREV, 0, firstc != b'@' as c_int);
                return nvim_command_line_changed(s.cast::<c_void>());
            }
        }
    }

    // NUL is stored as NL
    if (*s).c == NUL_INT || (*s).c == K_ZERO {
        (*s).c = NL;
    }

    (*s).do_abbr = true; // default: check for abbreviation

    // If already used to cancel/accept wildmenu, don't process the key further.
    if wild_type == WILD_CANCEL || wild_type == WILD_APPLY {
        // Inlined nvim_cls_maybe_reset_incsearch_state
        {
            let is_state_ptr = std::ptr::addr_of_mut!((*s).is_state);
            if (*is_state_ptr).winid != nvim_get_curwin_handle() {
                crate::search::rs_init_incsearch_state(is_state_ptr);
            }
        }
        if KeyTyped || vpeekc() == NUL_INT {
            // Inlined nvim_cls_may_do_incsearch
            {
                let firstc = (*s).firstc;
                let count = (*s).count;
                let is_state_ptr = std::ptr::addr_of_mut!((*s).is_state);
                crate::search::rs_may_do_incsearch_highlighting(firstc, count, is_state_ptr);
            }
        }
        return nvim_command_line_not_changed(s.cast::<c_void>());
    }

    // Dispatch to command_line_handle_key (now Rust-exported)
    crate::keys::rs_command_line_handle_key(s.cast::<c_void>())
}

/// Rust replacement for `command_line_check(VimState *state)`.
///
/// VimState check callback: called by the state machine before each key read.
/// Sets up state for the next key, triggers incsearch highlights, and moves cursor.
///
/// # Safety
///
/// `state` must be a valid non-null pointer to a `CommandLineState`.
#[unsafe(export_name = "command_line_check")]
pub unsafe extern "C" fn rs_command_line_check(state: *mut c_void) -> c_int {
    let s = state.cast::<crate::command_line_state::CommandLineState>();

    (*s).prev_cmdpos = nvim_get_ccline_cmdpos();
    xfree((*s).prev_cmdbuff);
    (*s).prev_cmdbuff = std::ptr::null_mut();

    redir_off = (1) != 0; // Don't redirect the typed command.
    quit_more = false; // reset after CTRL-D which had a more-prompt

    did_emsg = 0; // There can't really be a reason why an error
                  // that occurs while typing a command should
                  // cause the command not to be executed.

    if stuff_empty() && nvim_get_typebuf_len() == 0 {
        // typebuf_len accessor kept
        // There is no pending input from sources other than user input, so
        // Vim is going to wait for the user to type a key.  Consider the
        // command line typed even if next key will trigger a mapping.
        (*s).some_key_typed = true;
    }

    // Trigger SafeState if nothing is pending.
    may_trigger_safestate((*s).xpc.xp_numfiles <= 0);

    // Inline nvim_cls_dup_cmdbuff_to_prev: copy ccline.cmdbuff to s->prev_cmdbuff
    {
        let cmdbuff = nvim_get_ccline_cmdbuff();
        if !cmdbuff.is_null() {
            xfree((*s).prev_cmdbuff);
            (*s).prev_cmdbuff = xstrdup(cmdbuff);
        }
    }

    // Defer screen update to avoid pum flicker during wildtrigger()
    if (*s).c == K_WILD && (*s).firstc != b'@' as c_int {
        (*s).skip_pum_redraw = true;
    }

    cursorcmd(); // set the cursor on the right spot
    ui_cursor_shape();
    1
}

// =============================================================================
// Rust replacements for C command-line handler functions
// =============================================================================

// Additional FFI needed for the command-line handler functions below.
unsafe extern "C" {
    // Autocmd event IDs
    // EVENT_CURSORMOVEDC = 44 (from auevents_enum.generated.h)

    // cmdline pum and vim char helpers
    fn cmdline_pum_remove(skip_redraw: bool);
    fn rs_ascii_iswhite(c: c_int) -> c_int;
    fn vim_isprintc(c: c_int) -> bool;

    // For browse_history
    fn xstrnsave(s: *const c_char, len: usize) -> *mut c_char;

    // globals for command_line_changed
    static mut cmdpreview: bool;
    fn nvim_get_current_sctx_sid() -> c_int;
    fn nvim_excmds_get_p_icm_first() -> c_int;
    static mut cmdline_star: c_int;
    fn vpeekc_any() -> c_int;
    fn cmdpreview_may_show() -> c_int;
    fn update_screen();
    static p_arshape: c_int;
    static p_tbidi: c_int;

    // for toggle_langmap
    fn buf_valid(buf: *mut c_void) -> bool;
    static mut State: c_int;
    fn map_to_exists_mode(keys: *const c_char, mode: c_int, abbr: bool) -> c_int;
    fn set_iminsert_global(buf: *mut c_void);
    fn set_imsearch_global(buf: *mut c_void);
    fn nvim_get_curbuf() -> *mut c_void;
    fn nvim_get_curbuf_b_p_iminsert_ptr() -> *mut i64;

    // for left_right_mouse
    static mouse_row: c_int;
    static mouse_col: c_int;
    static Columns: c_int;
    fn correct_screencol(idx: c_int, cells: c_int, col: *mut c_int);
    fn utfc_ptr2len(p: *const c_char) -> c_int;
    fn rs_cmdline_charsize(idx: c_int) -> c_int;
    fn rs_cmd_startcol() -> c_int;

    // for toggle_langmap and not_changed
    fn status_redraw_curbuf();

    // for end_wildmenu
    fn wildmenu_cleanup(cclp: *mut c_void);

    // for left_right_mouse (already declared elsewhere but needed here)
    static cmdline_row: c_int;

    // for command_line_changed: string comparison
    fn strcmp(s1: *const c_char, s2: *const c_char) -> c_int;

    // for command_line_changed: incsearch highlighting check
    fn nvim_get_key_typed() -> c_int;
}

// Autocmd event constant
const EVENT_CURSORMOVEDC: c_int = 44;
const MODE_LANGMAP_LOCAL: c_int = 0x8000;
const B_IMODE_LMAP: i64 = 2;
const B_IMODE_NONE: i64 = 0;

// Local copy of termcap2key for key constant definitions (matches keys.rs)
// TERMCAP2KEY(a, b) = -((a) + ((int)(b) << 8))
const fn termcap2key_local(a: c_int, b: c_int) -> c_int {
    -(a + (b << 8))
}

// Key constants used in end_wildmenu / left_right_mouse
const KS_EXTRA_LOCAL: c_int = 253;
const KE_KDEL_LOCAL: c_int = 49;
const K_BS_LOCAL: c_int = termcap2key_local(b'k' as c_int, b'b' as c_int);
const CTRL_H_LOCAL: c_int = 8;
const K_DEL_LOCAL: c_int = termcap2key_local(b'k' as c_int, b'D' as c_int);
const K_KDEL_LOCAL: c_int = termcap2key_local(KS_EXTRA_LOCAL, KE_KDEL_LOCAL);
const CTRL_W_LOCAL: c_int = 23;
const CTRL_U_LOCAL: c_int = 21;
const K_UP_LOCAL: c_int = termcap2key_local(b'k' as c_int, b'u' as c_int);
const K_DOWN_LOCAL: c_int = termcap2key_local(b'k' as c_int, b'd' as c_int);

/// Safe wrapper for C strcmp.
unsafe fn strcmp_c(s1: *const c_char, s2: *const c_char) -> c_int {
    strcmp(s1, s2)
}

/// Rust replacement for `nvim_command_line_not_changed(void *s)` in ex_getln.c.
///
/// Post-key no-change handler: triggers CursorMoveD if position changed,
/// then falls through to changed handler if incsearch was postponed.
///
/// # Safety
///
/// `s` must be a valid non-null pointer to a `CommandLineState`.
#[no_mangle]
pub unsafe extern "C" fn nvim_command_line_not_changed(s: *mut c_void) -> c_int {
    let ss = s.cast::<crate::command_line_state::CommandLineState>();
    let cmdpos = nvim_get_ccline_cmdpos();
    if cmdpos != (*ss).prev_cmdpos {
        rs_trigger_cmd_autocmd((*ss).cmdline_type, EVENT_CURSORMOVEDC);
        // ccline.redraw_state = max(ccline.redraw_state, kCmdRedrawPos=1)
        let rs = nvim_get_ccline_redraw_state();
        if rs < 1 {
            nvim_set_ccline_redraw_state(1);
        }
    }
    (*ss).prev_cmdpos = cmdpos;
    if !(*ss).is_state.incsearch_postponed {
        return 1;
    }
    nvim_command_line_changed(s)
}

/// Rust replacement for `nvim_command_line_changed(void *vs)` in ex_getln.c.
///
/// Post-key change handler: handles inccommand preview, incsearch, and
/// triggers CmdlineChanged/CursorMoveD autocmds.
///
/// # Safety
///
/// `s` must be a valid non-null pointer to a `CommandLineState`.
#[no_mangle]
#[allow(clippy::too_many_lines)]
pub unsafe extern "C" fn nvim_command_line_changed(s: *mut c_void) -> c_int {
    let ss = s.cast::<crate::command_line_state::CommandLineState>();
    let prev_cmdpreview = cmdpreview;
    if (*ss).firstc == b':' as c_int
        && nvim_get_current_sctx_sid() == 0
        && nvim_excmds_get_p_icm_first() != 0
        && !exmode_active
        && cmdline_star == 0
        && vpeekc_any() == 0
        && cmdpreview_may_show() != 0
    {
        // 'inccommand' preview has been shown.
    } else {
        cmdpreview = false;
        if prev_cmdpreview {
            update_screen();
        }
        if (*ss).xpc.xp_context == EXPAND_NOTHING && (nvim_get_key_typed() != 0 || vpeekc() == 0) {
            crate::search::rs_may_do_incsearch_highlighting(
                (*ss).firstc,
                (*ss).count,
                std::ptr::addr_of_mut!((*ss).is_state),
            );
        }
    }

    let cmdpos = nvim_get_ccline_cmdpos();
    let prev_cmdbuff = (*ss).prev_cmdbuff;
    let cmdbuff = nvim_get_ccline_cmdbuff();
    let cmdpos_changed = cmdpos != (*ss).prev_cmdpos;
    let cmdbuff_changed =
        !prev_cmdbuff.is_null() && !cmdbuff.is_null() && strcmp_c(prev_cmdbuff, cmdbuff) != 0;
    if cmdpos_changed || cmdbuff_changed {
        let effective_firstc = if (*ss).firstc > 0 {
            (*ss).firstc
        } else {
            b'-' as c_int
        };
        do_autocmd_cmdlinechanged_inner(effective_firstc);
    }

    // trigger CursorMoveD if position changed
    if cmdpos_changed {
        rs_trigger_cmd_autocmd((*ss).cmdline_type, EVENT_CURSORMOVEDC);
        let rs = nvim_get_ccline_redraw_state();
        if rs < 1 {
            nvim_set_ccline_redraw_state(1);
        }
    }

    if p_arshape != 0 && p_tbidi == 0 && ui_has(K_UI_CMDLINE) == 0 && vpeekc() == 0 {
        crate::screen::redrawcmd_rs();
    }

    1
}

/// Rust replacement for `nvim_command_line_toggle_langmap(void *vs)` in ex_getln.c.
///
/// Toggles the langmap (CTRL-^ key) in command-line mode.
///
/// # Safety
///
/// `s` must be a valid non-null pointer to a `CommandLineState`.
#[no_mangle]
pub unsafe extern "C" fn nvim_command_line_toggle_langmap(s: *mut c_void) {
    let ss = s.cast::<crate::command_line_state::CommandLineState>();
    let b_im_ptr = if buf_valid((*ss).b_im_ptr_buf) {
        (*ss).b_im_ptr
    } else {
        std::ptr::null_mut()
    };
    if map_to_exists_mode(c"".as_ptr(), MODE_LANGMAP_LOCAL, false) != 0 {
        State ^= MODE_LANGMAP_LOCAL;
        if !b_im_ptr.is_null() {
            if State & MODE_LANGMAP_LOCAL != 0 {
                *b_im_ptr = B_IMODE_LMAP;
            } else {
                *b_im_ptr = B_IMODE_NONE;
            }
        }
    }
    if !b_im_ptr.is_null() {
        let curbuf_b_p_iminsert_ptr = nvim_get_curbuf_b_p_iminsert_ptr();
        if b_im_ptr == curbuf_b_p_iminsert_ptr {
            set_iminsert_global(nvim_get_curbuf());
        } else {
            set_imsearch_global(nvim_get_curbuf());
        }
    }
    ui_cursor_shape();
    status_redraw_curbuf();
}

/// Rust replacement for `nvim_command_line_left_right_mouse(void *vs)` in ex_getln.c.
///
/// Handles Left/Right mouse clicks in command-line mode.
///
/// # Safety
///
/// `s` must be a valid non-null pointer to a `CommandLineState`.
#[no_mangle]
pub unsafe extern "C" fn nvim_command_line_left_right_mouse(s: *mut c_void) {
    use crate::keys::{K_LEFTRELEASE, K_RIGHTRELEASE};
    let ss = s.cast::<crate::command_line_state::CommandLineState>();
    (*ss).ignore_drag_release = (*ss).c == K_LEFTRELEASE || (*ss).c == K_RIGHTRELEASE;
    let startcol = rs_cmd_startcol();
    nvim_set_ccline_cmdspos(startcol);
    let cmdlen = nvim_get_ccline_cmdlen();
    let cmdbuff = nvim_get_ccline_cmdbuff();
    let mut screen_col = startcol;
    let mut buf_idx = 0i32;
    while buf_idx < cmdlen {
        let cells = rs_cmdline_charsize(buf_idx);
        if mouse_row <= cmdline_row + screen_col / Columns
            && mouse_col < screen_col % Columns + cells
        {
            break;
        }
        correct_screencol(buf_idx, cells, &raw mut screen_col);
        buf_idx += utfc_ptr2len(cmdbuff.add(buf_idx as usize)) - 1;
        screen_col += cells;
        buf_idx += 1;
    }
    nvim_set_ccline_cmdpos(buf_idx);
    nvim_set_ccline_cmdspos(screen_col);
}

/// Rust replacement for `nvim_command_line_browse_history(void *vs)` in ex_getln.c.
///
/// Browse command-line history (called from Rust via opaque handle).
///
/// # Safety
///
/// `s` must be a valid non-null pointer to a `CommandLineState`.
#[no_mangle]
pub unsafe extern "C" fn nvim_command_line_browse_history(s: *mut c_void) -> c_int {
    let ss = s.cast::<crate::command_line_state::CommandLineState>();
    // Save current command string so it can be restored later.
    if (*ss).lookfor.is_null() {
        let cmdpos = nvim_get_ccline_cmdpos();
        let cmdlen = nvim_get_ccline_cmdlen();
        let cmdbuff = nvim_get_ccline_cmdbuff();
        (*ss).lookfor = xstrnsave(cmdbuff, cmdlen as usize);
        *(*ss).lookfor.add(cmdpos as usize) = 0; // NUL terminate at cmdpos
        (*ss).lookforlen = cmdpos;
    }
    // Pack state for Rust history browsing
    let mut rs_state = crate::history::HistoryBrowseState {
        c: (*ss).c,
        firstc: (*ss).firstc,
        hiscnt: (*ss).hiscnt,
        save_hiscnt: (*ss).save_hiscnt,
        histype: (*ss).histype,
        lookfor: (*ss).lookfor,
        lookforlen: (*ss).lookforlen,
    };
    // Call Rust implementation
    let result = crate::history::rs_command_line_browse_history(&raw mut rs_state);
    // Update state from Rust
    (*ss).hiscnt = rs_state.hiscnt;
    (*ss).save_hiscnt = rs_state.save_hiscnt;
    // Clear xp_context on history change
    if result == CMDLINE_CHANGED {
        (*ss).xpc.xp_context = EXPAND_NOTHING;
    }
    result
}

/// Rust replacement for `nvim_command_line_end_wildmenu(void *vs, bool key_is_wc)` in ex_getln.c.
///
/// Cleans up wildmenu when a non-wildchar key is pressed.
///
/// # Safety
///
/// `s` must be a valid non-null pointer to a `CommandLineState`.
#[no_mangle]
pub unsafe extern "C" fn nvim_command_line_end_wildmenu(s: *mut c_void, key_is_wc: bool) {
    let ss = s.cast::<crate::command_line_state::CommandLineState>();
    if cmdline_pum_active() != 0 {
        let c = (*ss).c;
        (*ss).skip_pum_redraw = (*ss).skip_pum_redraw
            && !key_is_wc
            && rs_ascii_iswhite(c) == 0
            && (vim_isprintc(c)
                || c == K_BS_LOCAL
                || c == CTRL_H_LOCAL
                || c == K_DEL_LOCAL
                || c == K_KDEL_LOCAL
                || c == CTRL_W_LOCAL
                || c == CTRL_U_LOCAL);
        cmdline_pum_remove((*ss).skip_pum_redraw);
    }
    if (*ss).xpc.xp_numfiles != -1 {
        let xp = std::ptr::addr_of_mut!((*ss).xpc).cast::<c_void>();
        ExpandOne(xp, std::ptr::null_mut(), std::ptr::null_mut(), 0, WILD_FREE);
    }
    (*ss).did_wild_list = false;
    if p_wmnu == 0 || ((*ss).c != K_UP_LOCAL && (*ss).c != K_DOWN_LOCAL) {
        (*ss).xpc.xp_context = EXPAND_NOTHING;
    }
    (*ss).wim_index = 0;
    // wildmenu_cleanup takes CmdlineInfo* but ignores the parameter in its Rust impl
    wildmenu_cleanup(std::ptr::null_mut());
}

// =============================================================================
// Phase 4: do_autocmd_cmdlinechanged migrated to Rust
// Phase 3 (ex_getln migration): nvim_fire_cmdlinechanged_autocmd migrated here
// =============================================================================

// Event IDs (from autocmd_events.h)
const EVENT_CMDLINECHANGED: c_int = 23;

// Size of C `save_v_event_T` in bytes (bool + hashtab_T = 304; validated in scroll.rs)
const SAVE_V_EVENT_SIZE: usize = 304;

// HLF_E = 6 (ErrorMsg highlight group)
const HLF_E: c_int = 6;

unsafe extern "C" {
    fn nvim_has_event(event: c_int) -> c_int;
    fn apply_autocmds(
        event: c_int,
        pat: *const c_char,
        fname_io: *const c_char,
        force: bool,
        buf: *mut c_void,
    ) -> bool;
    static mut curbuf: *mut c_void;
    fn nvim_get_v_event_opaque(buf: *mut u8) -> *mut c_void;
    fn tv_dict_add_str(
        dict: *mut c_void,
        key: *const c_char,
        key_len: usize,
        val: *const c_char,
    ) -> c_int;
    fn tv_dict_add_nr(dict: *mut c_void, key: *const c_char, key_len: usize, nr: i64) -> c_int;
    fn tv_dict_set_keys_readonly(dict: *mut c_void);
    fn nvim_cmdline_try_autocmd_restore(
        event: c_int,
        firstcbuf: *const c_char,
        dict: *mut c_void,
        save_buf: *mut u8,
        err_msg_out: *mut *mut c_char,
    ) -> c_int;
    fn msg_putchar(c: c_int);
    fn msg_puts_hl(s: *const c_char, hl_id: c_int, hist: bool);
    fn redrawcmd();
    static mut msg_scroll: c_int;
}

/// Rust replacement for `nvim_trigger_cmd_autocmd` in ex_getln.c.
///
/// Formats `typechar` as a 2-byte C string and calls `apply_autocmds`.
///
/// # Safety
///
/// Calls C autocmd functions with global editor state.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_trigger_cmd_autocmd(typechar: c_int, evt: c_int) {
    let typestr: [c_char; 2] = [typechar as c_char, 0];
    apply_autocmds(evt, typestr.as_ptr(), typestr.as_ptr(), false, curbuf);
}

/// Rust replacement for `nvim_fire_cmdlinechanged_autocmd` in ex_getln.c.
///
/// Sets up v:event dict and fires CmdlineChanged via TRY_WRAP shim.
/// Error handling: print message with HLF_E + redrawcmd.
///
/// # Safety
///
/// Calls C autocmd/dict functions.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn nvim_fire_cmdlinechanged_autocmd(firstc: c_int) {
    let mut save_buf = std::mem::MaybeUninit::<[u8; SAVE_V_EVENT_SIZE]>::zeroed();
    let dict = nvim_get_v_event_opaque(save_buf.as_mut_ptr().cast());

    let firstcbuf: [c_char; 2] = [firstc as c_char, 0];

    tv_dict_add_str(dict, c"cmdtype".as_ptr(), 7, firstcbuf.as_ptr());
    tv_dict_add_nr(
        dict,
        c"cmdlevel".as_ptr(),
        8,
        i64::from(nvim_get_ccline_level()),
    );
    tv_dict_set_keys_readonly(dict);

    let mut err_msg: *mut c_char = std::ptr::null_mut();
    // nvim_cmdline_try_autocmd_restore runs TRY_WRAP { apply_autocmds; restore_v_event; }
    // On error it xstrdup()s err.msg into err_msg; we must xfree it after use.
    if nvim_cmdline_try_autocmd_restore(
        EVENT_CMDLINECHANGED,
        firstcbuf.as_ptr(),
        dict,
        save_buf.as_mut_ptr().cast(),
        &raw mut err_msg,
    ) != 0
    {
        msg_putchar(b'\n' as c_int);
        msg_scroll = 1;
        msg_puts_hl(err_msg, HLF_E, true);
        libc::free(err_msg.cast::<libc::c_void>());
        redrawcmd();
    }
}

/// Inner logic for `do_autocmd_cmdlinechanged`.
///
/// # Safety
///
/// Calls C autocmd/dict functions.
unsafe fn do_autocmd_cmdlinechanged_inner(firstc: c_int) {
    if nvim_has_event(EVENT_CMDLINECHANGED) != 0 {
        nvim_fire_cmdlinechanged_autocmd(firstc);
    }
}

/// Rust replacement for `do_autocmd_cmdlinechanged` in ex_getln.c.
///
/// # Safety
///
/// Calls C autocmd/dict functions.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_do_autocmd_cmdlinechanged(firstc: c_int) {
    do_autocmd_cmdlinechanged_inner(firstc);
}
