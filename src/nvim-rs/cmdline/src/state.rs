//! Command line state management
//!
//! This module provides the core state structure for command-line editing,
//! including the command buffer, cursor position, and prompt configuration.

#![allow(unsafe_code)]
#![allow(clippy::doc_markdown)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::missing_const_for_fn)]

use std::ffi::{c_char, c_int, c_uint};

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

/// Check if the current command line is an Ex command.
///
/// # Safety
///
/// Calls external C function to access static variable.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_cmdline_state_is_ex_cmd() -> c_int {
    c_int::from(nvim_get_ccline_cmdfirstc() == c_int::from(prompt_type::EX_CMD))
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

/// Check if command line level is at maximum.
///
/// # Safety
///
/// Calls external C function to access static variable.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_cmdline_at_max_level() -> c_int {
    c_int::from(nvim_get_ccline_level() >= MAX_CMDLINE_LEVEL)
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

/// Get the current command line length.
///
/// # Safety
///
/// Calls external C function to access static variable.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_cmdline_get_cmdlen() -> c_int {
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
