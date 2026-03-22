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
    // CommandLineState field accessors (for Phase 4)
    fn nvim_cls_get_c(s: *mut c_void) -> c_int;
    fn nvim_cls_set_c(s: *mut c_void, val: c_int);
    fn nvim_cls_get_firstc(s: *mut c_void) -> c_int;
    fn nvim_cls_get_gotesc(s: *mut c_void) -> c_int;
    fn nvim_cls_set_gotesc(s: *mut c_void, val: c_int);
    fn nvim_cls_set_do_abbr(s: *mut c_void, val: c_int);
    fn nvim_cls_get_did_wild_list(s: *mut c_void) -> c_int;
    fn nvim_cls_set_did_wild_list(s: *mut c_void, val: c_int);
    fn nvim_cls_get_wim_index(s: *mut c_void) -> c_int;
    fn nvim_cls_set_wim_index(s: *mut c_void, val: c_int);
    fn nvim_cls_get_xpc(s: *mut c_void) -> *mut c_void;
    fn nvim_cls_get_xpc_numfiles(s: *mut c_void) -> c_int;
    fn nvim_cls_set_xpc_context(s: *mut c_void, val: c_int);
    fn nvim_cls_set_some_key_typed(s: *mut c_void, val: c_int);
    fn nvim_cls_get_break_ctrl_c(s: *mut c_void) -> c_int;
    fn nvim_cls_set_event_cmdlineleavepre_triggered(s: *mut c_void, val: c_int);
    fn nvim_cls_set_did_hist_navigate(s: *mut c_void, val: c_int);
    fn nvim_cls_get_did_hist_navigate(s: *mut c_void) -> c_int;
    fn nvim_cls_xfree_lookfor(s: *mut c_void);
    fn nvim_cls_get_lookfor(s: *mut c_void) -> *mut c_char;
    fn nvim_cls_set_xpc_pre_incsearch_from_is_state(s: *mut c_void);
    fn nvim_cls_maybe_reset_incsearch_state(s: *mut c_void);
    fn nvim_cls_may_do_incsearch(s: *mut c_void);
    fn nvim_cls_get_is_state_did_incsearch(s: *mut c_void) -> c_int;
    fn nvim_cls_trigger_cmdlineleavepre(s: *mut c_void);

    // Wrappers for wildmenu functions
    fn nvim_wildmenu_translate_key(s: *mut c_void) -> c_int;
    fn nvim_wildmenu_process_key(s: *mut c_void) -> c_int;
    fn nvim_command_line_end_wildmenu(s: *mut c_void, key_is_wc: bool);

    // Wrappers for command_line_not/changed
    fn nvim_command_line_not_changed(s: *mut c_void) -> c_int;
    fn nvim_command_line_changed(s: *mut c_void) -> c_int;

    // Wrappers for nextwild/showmatches
    fn nvim_nextwild(xp: *mut c_void, wild_type: c_int, options: c_int, escape: bool) -> c_int;
    fn nvim_showmatches(
        xp: *mut c_void,
        display_wildmenu: bool,
        display_list: bool,
        noselect: bool,
    ) -> c_int;

    // Global accessors
    fn nvim_get_wim_flags(idx: c_int) -> u8;
    fn nvim_get_p_wc() -> c_int;
    fn nvim_get_p_wcm() -> c_int;
    fn nvim_get_p_wmnu() -> c_int;
    fn nvim_get_key_typed_cmdline() -> c_int;
    fn nvim_get_cmdmsg_rl() -> c_int;
    fn nvim_get_key_stuffed() -> c_int;
    fn nvim_get_cmd_silent() -> c_int;
    fn nvim_get_got_int() -> c_int;
    fn nvim_set_got_int(val: c_int);
    fn nvim_get_global_busy() -> bool;
    fn nvim_get_ex_normal_busy() -> c_int;
    fn nvim_get_exmode_active() -> bool;
    fn nvim_get_cedit_key() -> c_int;
    fn nvim_open_cmdwin() -> c_int;
    fn nvim_get_pum_want_active() -> c_int;
    fn nvim_edit_get_pum_want_finish() -> c_int;
    fn nvim_set_pum_want_active(val: c_int);
    fn nvim_get_wild_menu_showing() -> c_int;
    fn nvim_get_cmdline_was_last_drawn() -> c_int;
    fn nvim_syn_get_display_tick() -> c_int;
    fn nvim_get_got_int_val() -> c_int;
    fn nvim_get_emsg_silent() -> c_int;
    fn nvim_set_emsg_silent(val: c_int);

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
    fn nvim_state_handle_k_event();
    fn nvim_edit_map_execute_lua();
    fn nvim_cmdline_do_cmdline_nowait();
    fn msg_cursor_goto(row: c_int, col: c_int);
    fn ui_flush();
    fn ui_has(what: c_int) -> c_int;
    fn nvim_get_msg_row() -> c_int;
    fn vim_strchr(haystack: *const c_char, needle: c_int) -> *mut c_char;
    fn nvim_get_p_cpo() -> *const c_char;
    fn cmdline_pum_active() -> c_int;
    // For command_line_check
    fn nvim_cls_set_prev_cmdpos(s: *mut c_void, val: c_int);
    fn nvim_cls_xfree_prev_cmdbuff(s: *mut c_void);
    fn nvim_cls_dup_cmdbuff_to_prev(s: *mut c_void);
    fn nvim_cls_set_skip_pum_redraw(s: *mut c_void, val: c_int);
    fn nvim_set_redir_off(val: c_int);
    fn nvim_set_quit_more(val: bool);
    fn nvim_set_did_emsg(val: c_int);
    fn nvim_get_typebuf_len() -> c_int;
    fn stuff_empty() -> c_int;
    fn may_trigger_safestate(pending: bool);
    fn cursorcmd();
    fn ui_cursor_shape();
}

// CPO_ESC is 'x' in C's cpoptions
const CPO_ESC_CHAR: c_int = b'x' as c_int;

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
    let mut options: c_int = WILD_NO_BEEP;
    let c = nvim_cls_get_c(s);
    let firstc = nvim_cls_get_firstc(s);
    let escape = firstc != b'@' as c_int;
    let redraw_if_menu_empty = c == K_WILD;
    let p_wmnu = nvim_get_p_wmnu() != 0;
    let wim_noselect = p_wmnu && (nvim_get_wim_flags(0) & WIM_FLAG_NOSELECT) != 0;

    let wim_index = nvim_cls_get_wim_index(s);
    if (nvim_get_wim_flags(wim_index) & WIM_FLAG_LASTUSED) != 0 {
        options |= WILD_BUFLASTUSED;
    }

    let xpc_numfiles = nvim_cls_get_xpc_numfiles(s);
    let xp = nvim_cls_get_xpc(s);

    let res: c_int;
    if xpc_numfiles > 0 {
        // typed p_wc at least twice
        // If "list" is present, list matches unless already listed
        if xpc_numfiles > 1
            && nvim_cls_get_did_wild_list(s) == 0
            && (nvim_get_wim_flags(wim_index) & WIM_FLAG_LIST) != 0
        {
            nvim_showmatches(xp, false, true, wim_noselect);
            crate::screen::redrawcmd_rs();
            nvim_cls_set_did_wild_list(s, 1);
        }
        if (nvim_get_wim_flags(wim_index) & WIM_FLAG_LONGEST) != 0 {
            res = nvim_nextwild(xp, WILD_LONGEST, options, escape);
        } else if (nvim_get_wim_flags(wim_index) & WIM_FLAG_FULL) != 0 {
            res = nvim_nextwild(xp, WILD_NEXT, options, escape);
        } else {
            res = OK; // don't insert 'wildchar' now
        }
    } else {
        // typed p_wc first time
        let wim_longest = (nvim_get_wim_flags(0) & WIM_FLAG_LONGEST) != 0;
        let wim_list = (nvim_get_wim_flags(0) & WIM_FLAG_LIST) != 0;
        let wim_full = (nvim_get_wim_flags(0) & WIM_FLAG_FULL) != 0;

        nvim_cls_set_wim_index(s, 0);
        let p_wc = nvim_get_p_wc();
        let p_wcm = nvim_get_p_wcm();
        if c == p_wc || c == p_wcm || c == K_WILD || c == CTRL_Z {
            options |= WILD_MAY_EXPAND_PATTERN;
            if c == K_WILD {
                options |= WILD_FUNC_TRIGGER;
            }
            nvim_cls_set_xpc_pre_incsearch_from_is_state(s);
        }
        let cmdpos_before = nvim_get_ccline_cmdpos();

        // if 'wildmode' first contains "longest", get longest common part
        if wim_longest {
            res = nvim_nextwild(xp, WILD_LONGEST, options, escape);
        } else {
            let mut opts = options;
            if wim_noselect || wim_list {
                opts |= WILD_NOSELECT;
            }
            res = nvim_nextwild(xp, WILD_EXPAND_KEEP, opts, escape);
        }

        // Remove popup menu if no completion items are available
        if redraw_if_menu_empty && nvim_cls_get_xpc_numfiles(s) <= 0 {
            pum_check_clear();
        }

        // if interrupted while completing, behave like it failed
        if nvim_get_got_int_val() != 0 {
            vpeekc(); // remove <C-C> from input stream
            nvim_set_got_int(0); // don't abandon the command line
            ExpandOne(xp, std::ptr::null_mut(), std::ptr::null_mut(), 0, WILD_FREE);
            nvim_cls_set_xpc_context(s, EXPAND_NOTHING);
            return CMDLINE_CHANGED;
        }

        // Display matches
        let xpc_numfiles2 = nvim_cls_get_xpc_numfiles(s);
        let threshold = i32::from(!wim_noselect);
        if res == OK && xpc_numfiles2 > threshold {
            if wim_longest {
                let found_longest_prefix = nvim_get_ccline_cmdpos() != cmdpos_before;
                let wim_list_check = wim_list || (p_wmnu && wim_full);
                if wim_list_check {
                    nvim_showmatches(xp, p_wmnu, wim_list, true);
                } else if !found_longest_prefix {
                    let wim_list_next = (nvim_get_wim_flags(1) & WIM_FLAG_LIST) != 0;
                    let wim_full_next = (nvim_get_wim_flags(1) & WIM_FLAG_FULL) != 0;
                    let wim_noselect_next = (nvim_get_wim_flags(1) & WIM_FLAG_NOSELECT) != 0;
                    if wim_list_next || (p_wmnu && (wim_full_next || wim_noselect_next)) {
                        if wim_full_next && !wim_noselect_next {
                            nvim_nextwild(xp, WILD_NEXT, options, escape);
                        } else {
                            nvim_showmatches(xp, p_wmnu, wim_list_next, wim_noselect_next);
                        }
                        if wim_list_next {
                            nvim_cls_set_did_wild_list(s, 1);
                        }
                    }
                }
            } else {
                let wim_list2 = wim_list || (p_wmnu && (wim_full || wim_noselect));
                if wim_list2 {
                    nvim_showmatches(xp, p_wmnu, wim_list, wim_noselect);
                } else {
                    vim_beep(K_OPT_BO_FLAG_WILDMODE);
                }
            }
            crate::screen::redrawcmd_rs();
            if wim_list {
                nvim_cls_set_did_wild_list(s, 1);
            }
        } else if nvim_cls_get_xpc_numfiles(s) == -1 {
            nvim_cls_set_xpc_context(s, EXPAND_NOTHING);
        }
    }

    let wim_index_new = nvim_cls_get_wim_index(s);
    if wim_index_new < 3 {
        nvim_cls_set_wim_index(s, wim_index_new + 1);
    }

    if c == ESC {
        nvim_cls_set_gotesc(s, 1);
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
    let s = state;
    if key == K_IGNORE || key == K_NOP {
        return -1; // get another key
    }

    let display_tick_saved = nvim_syn_get_display_tick();
    nvim_cls_set_c(s, key);

    // Skip wildmenu during history navigation via Up/Down keys
    if nvim_cls_get_c(s) == K_WILD && nvim_cls_get_did_hist_navigate(s) != 0 {
        nvim_cls_set_did_hist_navigate(s, 0);
        return 1;
    }

    let c = nvim_cls_get_c(s);
    if c == K_EVENT || c == K_COMMAND || c == K_LUA {
        if c == K_EVENT {
            nvim_state_handle_k_event();
        } else if c == K_COMMAND {
            nvim_cmdline_do_cmdline_nowait();
        } else {
            nvim_edit_map_execute_lua();
        }
        // If the window changed incremental search state is not valid.
        nvim_cls_maybe_reset_incsearch_state(s);
        // Re-apply 'incsearch' highlighting in case it was cleared.
        if nvim_syn_get_display_tick() > display_tick_saved
            && nvim_cls_get_is_state_did_incsearch(s) != 0
        {
            nvim_cls_may_do_incsearch(s);
        }

        // nvim_select_popupmenu_item() can be called from K_EVENT/K_COMMAND/K_LUA
        if nvim_get_pum_want_active() != 0 {
            if cmdline_pum_active() != 0 {
                let firstc = nvim_cls_get_firstc(s);
                let xp = nvim_cls_get_xpc(s);
                nvim_nextwild(xp, WILD_PUM_WANT, 0, firstc != b'@' as c_int);
                if nvim_edit_get_pum_want_finish() != 0 {
                    nvim_nextwild(xp, WILD_APPLY, WILD_NO_BEEP, firstc != b'@' as c_int);
                    nvim_command_line_end_wildmenu(s, false);
                }
            }
            nvim_set_pum_want_active(0);
        }

        if nvim_get_cmdline_was_last_drawn() == 0 {
            crate::screen::rs_redrawcmdline();
        }
        return 1;
    }

    if nvim_get_key_typed_cmdline() != 0 {
        nvim_cls_set_some_key_typed(s, 1);

        if nvim_get_cmdmsg_rl() != 0 && nvim_get_key_stuffed() == 0 {
            // Invert horizontal movements and operations. Only when typed by user
            // directly, not when the result of a mapping.
            let c_curr = nvim_cls_get_c(s);
            nvim_cls_set_c(s, crate::keys::invert_rtl_key(c_curr));
        }
    }

    // Ignore got_int when CTRL-C was typed here.
    // Don't ignore it in :global, we really need to break then.
    // Don't ignore it for the input() function.
    {
        let c_curr = nvim_cls_get_c(s);
        let firstc = nvim_cls_get_firstc(s);
        let break_ctrl_c = nvim_cls_get_break_ctrl_c(s) != 0;
        let exmode = nvim_get_exmode_active();
        if c_curr == CTRL_C
            && firstc != b'@' as c_int
            && (!break_ctrl_c || exmode)
            && !nvim_get_global_busy()
        {
            nvim_set_got_int(0);
        }
    }

    // free old command line when finished moving around in the history list
    {
        let c_curr = nvim_cls_get_c(s);
        let xpc_numfiles = nvim_cls_get_xpc_numfiles(s);
        if !nvim_cls_get_lookfor(s).is_null()
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
            nvim_cls_xfree_lookfor(s);
        }
    }

    // When there are matching completions to select <S-Tab> works like CTRL-P
    {
        let c_curr = nvim_cls_get_c(s);
        let p_wc = nvim_get_p_wc();
        if crate::keys::rs_is_stab_to_ctrl_p(c_curr, p_wc) != 0 && nvim_cls_get_xpc_numfiles(s) > 0
        {
            nvim_cls_set_c(s, CTRL_P);
        }
    }

    if nvim_get_p_wmnu() != 0 {
        let c_new = nvim_wildmenu_translate_key(s);
        nvim_cls_set_c(s, c_new);
    }

    let c_curr = nvim_cls_get_c(s);
    let p_wc = nvim_get_p_wc();
    let p_wcm = nvim_get_p_wcm();
    let key_is_wc = (c_curr == p_wc && nvim_get_key_typed_cmdline() != 0) || c_curr == p_wcm;

    let mut wild_type = 0_i32;
    if (cmdline_pum_active() != 0
        || nvim_get_wild_menu_showing() != 0
        || nvim_cls_get_did_wild_list(s) != 0)
        && !key_is_wc
    {
        let c_check = nvim_cls_get_c(s);
        if c_check == CTRL_E || c_check == CTRL_Y {
            wild_type = if c_check == CTRL_E {
                WILD_CANCEL
            } else {
                WILD_APPLY
            };
            let xp = nvim_cls_get_xpc(s);
            let firstc = nvim_cls_get_firstc(s);
            nvim_nextwild(xp, wild_type, WILD_NO_BEEP, firstc != b'@' as c_int);
        }
    }

    // Trigger CmdlineLeavePre autocommand
    {
        let c_check = nvim_cls_get_c(s);
        if (nvim_get_key_typed_cmdline() != 0
            && (c_check == b'\n' as c_int
                || c_check == b'\r' as c_int
                || c_check == K_KENTER
                || c_check == ESC))
            || c_check == CTRL_C
        {
            nvim_cls_trigger_cmdlineleavepre(s);
            nvim_cls_set_event_cmdlineleavepre_triggered(s, 1);
            if (c_check == ESC || c_check == CTRL_C) && (nvim_get_wim_flags(0) & WIM_FLAG_LIST) != 0
            {
                set_no_hlsearch(1);
            }
        }
    }

    // The wildmenu is cleared if the pressed key is not used for navigating
    let c_check = nvim_cls_get_c(s);
    let end_wildmenu = !key_is_wc && crate::keys::rs_should_end_wildmenu(c_check, p_wc, p_wcm) != 0;
    let end_wildmenu = end_wildmenu
        && (cmdline_pum_active() == 0 || crate::keys::rs_should_end_wildmenu_pum(c_check) != 0);

    // free expanded names when finished walking through matches
    if end_wildmenu {
        nvim_command_line_end_wildmenu(s, key_is_wc);
    }

    if nvim_get_p_wmnu() != 0 {
        let c_new = nvim_wildmenu_process_key(s);
        nvim_cls_set_c(s, c_new);
    }

    // CTRL-\ handling
    {
        let c_check = nvim_cls_get_c(s);
        if c_check == CTRL_BSL {
            let mut c_val = nvim_cls_get_c(s);
            let mut gotesc_val = (nvim_cls_get_gotesc(s) != 0) as bool;
            match crate::keys::rs_command_line_handle_ctrl_bsl(&raw mut c_val, &raw mut gotesc_val)
            {
                x if x == CMDLINE_CHANGED => {
                    nvim_cls_set_c(s, c_val);
                    nvim_cls_set_gotesc(s, gotesc_val as c_int);
                    return nvim_command_line_changed(s);
                }
                x if x == CMDLINE_NOT_CHANGED => {
                    nvim_cls_set_c(s, c_val);
                    nvim_cls_set_gotesc(s, gotesc_val as c_int);
                    return nvim_command_line_not_changed(s);
                }
                0 => {
                    nvim_cls_set_c(s, c_val);
                    nvim_cls_set_gotesc(s, gotesc_val as c_int);
                    return 0; // back to cmd mode
                }
                _ => {
                    // backslash key not processed
                    nvim_cls_set_c(s, CTRL_BSL);
                }
            }
        }
    }

    // Handle cedit_key or K_CMDWIN
    {
        let c_check = nvim_cls_get_c(s);
        let cedit_key = nvim_get_cedit_key();
        if c_check == cedit_key || c_check == K_CMDWIN {
            if (c_check == K_CMDWIN || nvim_get_ex_normal_busy() == 0) && nvim_get_got_int() == 0 {
                let c_new = nvim_open_cmdwin();
                nvim_cls_set_c(s, c_new);
                nvim_cls_set_some_key_typed(s, 1);
            }
        } else {
            let c_check = nvim_cls_get_c(s);
            let c_new = do_digraph(c_check);
            nvim_cls_set_c(s, c_new);
        }
    }

    // Handle Enter/ESC
    {
        let c_check = nvim_cls_get_c(s);
        if c_check == b'\n' as c_int
            || c_check == b'\r' as c_int
            || c_check == K_KENTER
            || (c_check == ESC
                && (nvim_get_key_typed_cmdline() == 0
                    || !vim_strchr(nvim_get_p_cpo(), CPO_ESC_CHAR).is_null()))
        {
            let exmode = nvim_get_exmode_active();
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
                    nvim_cls_set_c(s, b'\n' as c_int);
                }
                // fall through: don't return, just continue with the char
            } else {
                nvim_cls_set_gotesc(s, 0);
                if crate::edit::rs_ccheck_abbr(nvim_cls_get_c(s) + 0x100) != 0 {
                    return nvim_command_line_changed(s);
                }
                if nvim_get_cmd_silent() == 0 {
                    if ui_has(K_UI_CMDLINE) == 0 {
                        msg_cursor_goto(nvim_get_msg_row(), 0);
                    }
                    ui_flush();
                }
                return 0;
            }
        }
    }

    // Completion for 'wildchar', 'wildcharm', and wildtrigger()
    {
        let c_check = nvim_cls_get_c(s);
        let p_wc = nvim_get_p_wc();
        let p_wcm = nvim_get_p_wcm();
        if (c_check == p_wc && nvim_cls_get_gotesc(s) == 0 && nvim_get_key_typed_cmdline() != 0)
            || c_check == p_wcm
            || c_check == K_WILD
            || c_check == CTRL_Z
        {
            if c_check == K_WILD {
                nvim_set_emsg_silent(nvim_get_emsg_silent() + 1); // silence the bell
            }
            let res = rs_command_line_wildchar_complete(s);
            if c_check == K_WILD {
                nvim_set_emsg_silent(nvim_get_emsg_silent() - 1);
            }
            if res == CMDLINE_CHANGED {
                return nvim_command_line_changed(s);
            }
            if c_check == K_WILD {
                return nvim_command_line_not_changed(s);
            }
        }
    }

    nvim_cls_set_gotesc(s, 0);

    // <S-Tab> goes to last match, in a clumsy way
    {
        let c_check = nvim_cls_get_c(s);
        if c_check == K_S_TAB && nvim_get_key_typed_cmdline() != 0 {
            let xp = nvim_cls_get_xpc(s);
            let firstc = nvim_cls_get_firstc(s);
            if nvim_nextwild(xp, WILD_EXPAND_KEEP, 0, firstc != b'@' as c_int) == OK {
                let numfiles = nvim_cls_get_xpc_numfiles(s);
                let wim_idx = nvim_cls_get_wim_index(s);
                if numfiles > 1
                    && ((nvim_cls_get_did_wild_list(s) == 0
                        && (nvim_get_wim_flags(wim_idx) & WIM_FLAG_LIST) != 0)
                        || nvim_get_p_wmnu() != 0)
                {
                    nvim_showmatches(
                        xp,
                        nvim_get_p_wmnu() != 0,
                        (nvim_get_wim_flags(wim_idx) & WIM_FLAG_LIST) != 0,
                        (nvim_get_wim_flags(0) & WIM_FLAG_NOSELECT) != 0,
                    );
                }
                nvim_nextwild(xp, WILD_PREV, 0, firstc != b'@' as c_int);
                nvim_nextwild(xp, WILD_PREV, 0, firstc != b'@' as c_int);
                return nvim_command_line_changed(s);
            }
        }
    }

    // NUL is stored as NL
    if nvim_cls_get_c(s) == NUL_INT || nvim_cls_get_c(s) == K_ZERO {
        nvim_cls_set_c(s, NL);
    }

    nvim_cls_set_do_abbr(s, 1); // default: check for abbreviation

    // If already used to cancel/accept wildmenu, don't process the key further.
    if wild_type == WILD_CANCEL || wild_type == WILD_APPLY {
        nvim_cls_maybe_reset_incsearch_state(s);
        if nvim_get_key_typed_cmdline() != 0 || vpeekc() == NUL_INT {
            nvim_cls_may_do_incsearch(s);
        }
        return nvim_command_line_not_changed(s);
    }

    // Dispatch to command_line_handle_key (now Rust-exported)
    crate::keys::rs_command_line_handle_key(s)
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
    let s = state;

    nvim_cls_set_prev_cmdpos(s, nvim_get_ccline_cmdpos());
    nvim_cls_xfree_prev_cmdbuff(s);

    nvim_set_redir_off(1); // Don't redirect the typed command.
    nvim_set_quit_more(false); // reset after CTRL-D which had a more-prompt

    nvim_set_did_emsg(0); // There can't really be a reason why an error
                          // that occurs while typing a command should
                          // cause the command not to be executed.

    if stuff_empty() != 0 && nvim_get_typebuf_len() == 0 {
        // There is no pending input from sources other than user input, so
        // Vim is going to wait for the user to type a key.  Consider the
        // command line typed even if next key will trigger a mapping.
        nvim_cls_set_some_key_typed(s, 1);
    }

    // Trigger SafeState if nothing is pending.
    may_trigger_safestate(nvim_cls_get_xpc_numfiles(s) <= 0);

    nvim_cls_dup_cmdbuff_to_prev(s);

    // Defer screen update to avoid pum flicker during wildtrigger()
    if nvim_cls_get_c(s) == K_WILD && nvim_cls_get_firstc(s) != b'@' as c_int {
        nvim_cls_set_skip_pum_redraw(s, 1);
    }

    cursorcmd(); // set the cursor on the right spot
    ui_cursor_shape();
    1
}
