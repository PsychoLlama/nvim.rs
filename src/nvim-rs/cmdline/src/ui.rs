//! Command line UI integration
//!
//! This module provides types and utilities for command-line UI events,
//! which are sent to external UI clients via the msgpack-rpc protocol.

#![allow(clippy::doc_markdown)]
#![allow(clippy::missing_const_for_fn)]

use std::ffi::c_int;

// =============================================================================
// UI Event Types
// =============================================================================

/// Type of command line UI event.
#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CmdlineUiEvent {
    /// Show command line (cmdline_show)
    Show = 0,
    /// Update cursor position (cmdline_pos)
    Pos = 1,
    /// Show special character (cmdline_special_char)
    SpecialChar = 2,
    /// Hide command line (cmdline_hide)
    Hide = 3,
    /// Show block of command lines (cmdline_block_show)
    BlockShow = 4,
    /// Append to command line block (cmdline_block_append)
    BlockAppend = 5,
    /// Hide command line block (cmdline_block_hide)
    BlockHide = 6,
}

// =============================================================================
// UI State
// =============================================================================

/// State for command line UI.
#[derive(Debug, Clone, Copy, Default)]
#[allow(clippy::struct_excessive_bools)]
pub struct CmdlineUiState {
    /// Whether UI events are pending flush.
    pub dirty: bool,
    /// Whether command line is currently shown.
    pub shown: bool,
    /// Current cursor position sent to UI.
    pub sent_pos: i32,
    /// Current level sent to UI.
    pub sent_level: i32,
    /// Whether special char is currently displayed.
    pub special_char_shown: bool,
    /// Whether block is currently shown.
    pub block_shown: bool,
}

impl CmdlineUiState {
    /// Create a new UI state.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            dirty: false,
            shown: false,
            sent_pos: 0,
            sent_level: 0,
            special_char_shown: false,
            block_shown: false,
        }
    }

    /// Mark state as dirty (needs flush).
    pub fn mark_dirty(&mut self) {
        self.dirty = true;
    }

    /// Clear dirty flag.
    pub fn clear_dirty(&mut self) {
        self.dirty = false;
    }

    /// Update after show event.
    pub fn on_show(&mut self, level: i32) {
        self.shown = true;
        self.sent_level = level;
        self.dirty = false;
    }

    /// Update after hide event.
    pub fn on_hide(&mut self) {
        self.shown = false;
        self.special_char_shown = false;
    }

    /// Update after pos event.
    pub fn on_pos(&mut self, pos: i32) {
        self.sent_pos = pos;
    }

    /// Update after special char event.
    pub fn on_special_char(&mut self, shown: bool) {
        self.special_char_shown = shown;
    }

    /// Update after block show.
    pub fn on_block_show(&mut self) {
        self.block_shown = true;
    }

    /// Update after block hide.
    pub fn on_block_hide(&mut self) {
        self.block_shown = false;
    }

    /// Reset state for new command line.
    pub fn reset(&mut self) {
        *self = Self::new();
    }
}

// =============================================================================
// Content Attributes
// =============================================================================

/// Attribute for command line content.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ContentAttr {
    /// Highlight group ID.
    pub hl_id: i32,
    /// Start byte position.
    pub start: i32,
    /// End byte position (exclusive).
    pub end: i32,
}

impl ContentAttr {
    /// Create a new content attribute.
    #[must_use]
    pub const fn new(hl_id: i32, start: i32, end: i32) -> Self {
        Self { hl_id, start, end }
    }
}

// =============================================================================
// Special Character Types
// =============================================================================

/// Type of special character shown in command line.
#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpecialCharType {
    /// No special character
    None = 0,
    /// Digraph entry (Ctrl-K)
    Digraph = 1,
    /// Literal character (Ctrl-V)
    Literal = 2,
    /// Register (Ctrl-R)
    Register = 3,
}

impl SpecialCharType {
    /// Get description for the special char type.
    #[must_use]
    pub const fn description(self) -> &'static str {
        match self {
            Self::None => "",
            Self::Digraph => "digraph",
            Self::Literal => "literal",
            Self::Register => "register",
        }
    }
}

// =============================================================================
// Redraw State
// =============================================================================

/// Redraw state for command line.
#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum RedrawState {
    /// No redraw needed.
    #[default]
    None = 0,
    /// Only cursor position changed.
    Pos = 1,
    /// Full redraw needed.
    All = 2,
}

impl RedrawState {
    /// Check if any redraw is needed.
    #[must_use]
    pub const fn needs_redraw(self) -> bool {
        !matches!(self, Self::None)
    }

    /// Check if full redraw is needed.
    #[must_use]
    pub const fn needs_full_redraw(self) -> bool {
        matches!(self, Self::All)
    }

    /// Merge two redraw states (takes more severe).
    #[must_use]
    pub const fn merge(self, other: Self) -> Self {
        match (self, other) {
            (Self::All, _) | (_, Self::All) => Self::All,
            (Self::Pos, _) | (_, Self::Pos) => Self::Pos,
            _ => Self::None,
        }
    }
}

// =============================================================================
// Prompt Character
// =============================================================================

/// Prompt character for command line.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Prompt {
    /// First character of command line (determines type).
    pub firstc: u8,
    /// Additional prompt string (if any).
    pub prompt_id: i32,
    /// Indent level for prompt.
    pub indent: i32,
}

impl Prompt {
    /// Create a new prompt.
    #[must_use]
    pub const fn new(firstc: u8, prompt_id: i32, indent: i32) -> Self {
        Self {
            firstc,
            prompt_id,
            indent,
        }
    }

    /// Get prompt character as string.
    #[must_use]
    pub fn firstc_str(&self) -> &'static str {
        match self.firstc {
            b':' => ":",
            b'/' => "/",
            b'?' => "?",
            b'=' => "=",
            b'>' => ">",
            b'@' => "@",
            _ => "",
        }
    }

    /// Check if this is a search prompt.
    #[must_use]
    pub const fn is_search(&self) -> bool {
        self.firstc == b'/' || self.firstc == b'?'
    }

    /// Check if this is an Ex command prompt.
    #[must_use]
    pub const fn is_ex_cmd(&self) -> bool {
        self.firstc == b':'
    }
}

// =============================================================================
// Color Cache Validation
// =============================================================================

/// Maximum number of callback errors before disabling highlighting.
pub const MAX_CALLBACK_ERRORS: i32 = 3;

/// Check if color cache is still valid.
///
/// The cache is valid if:
/// 1. The prompt ID matches
/// 2. The cached command buffer is not null
/// 3. The command buffer content matches
#[must_use]
pub const fn color_cache_valid(
    cache_prompt_id: u32,
    current_prompt_id: u32,
    cache_cmdbuff_is_null: bool,
) -> bool {
    if cache_prompt_id != current_prompt_id {
        return false;
    }
    if cache_cmdbuff_is_null {
        return false;
    }
    // Actual string comparison must be done by caller
    true
}

/// Check if coloring should be skipped due to too many errors.
#[must_use]
pub const fn should_skip_coloring(
    current_prompt_id: u32,
    prev_prompt_id: u32,
    prev_errors: i32,
) -> bool {
    current_prompt_id == prev_prompt_id && prev_errors >= MAX_CALLBACK_ERRORS
}

/// Check if callback errors should be reset (new prompt).
#[must_use]
pub const fn should_reset_callback_errors(current_prompt_id: u32, prev_prompt_id: u32) -> bool {
    current_prompt_id != prev_prompt_id
}

// =============================================================================
// Draw Range Calculations
// =============================================================================

/// Calculate the number of bytes to draw.
///
/// Ensures we don't draw past the end of the command buffer.
#[must_use]
pub const fn calculate_draw_len(start: i32, requested_len: i32, cmdlen: i32) -> i32 {
    let remaining = cmdlen - start;
    if requested_len > remaining {
        remaining
    } else {
        requested_len
    }
}

/// Check if drawing should proceed.
///
/// Drawing should not proceed if:
/// - Command buffer is null
/// - Start position is past end
/// - Length is zero or negative
#[must_use]
pub const fn should_draw(cmdbuff_is_null: bool, start: i32, len: i32, cmdlen: i32) -> bool {
    if cmdbuff_is_null {
        return false;
    }
    if start >= cmdlen {
        return false;
    }
    if len <= 0 {
        return false;
    }
    true
}

// =============================================================================
// FFI Exports
// =============================================================================

/// Check if color cache is valid (FFI).
///
/// Note: Actual string comparison must be done by caller.
#[no_mangle]
pub extern "C" fn rs_color_cache_valid(
    cache_prompt_id: u32,
    current_prompt_id: u32,
    cache_cmdbuff_is_null: c_int,
) -> c_int {
    c_int::from(color_cache_valid(
        cache_prompt_id,
        current_prompt_id,
        cache_cmdbuff_is_null != 0,
    ))
}

/// Check if coloring should be skipped (FFI).
#[no_mangle]
pub extern "C" fn rs_should_skip_coloring(
    current_prompt_id: u32,
    prev_prompt_id: u32,
    prev_errors: c_int,
) -> c_int {
    c_int::from(should_skip_coloring(
        current_prompt_id,
        prev_prompt_id,
        prev_errors,
    ))
}

/// Check if callback errors should be reset (FFI).
#[no_mangle]
pub extern "C" fn rs_should_reset_callback_errors(
    current_prompt_id: u32,
    prev_prompt_id: u32,
) -> c_int {
    c_int::from(should_reset_callback_errors(
        current_prompt_id,
        prev_prompt_id,
    ))
}

/// Calculate draw length (FFI).
#[no_mangle]
pub extern "C" fn rs_calculate_draw_len(
    start: c_int,
    requested_len: c_int,
    cmdlen: c_int,
) -> c_int {
    calculate_draw_len(start, requested_len, cmdlen)
}

/// Check if drawing should proceed (FFI).
#[no_mangle]
pub extern "C" fn rs_should_draw(
    cmdbuff_is_null: c_int,
    start: c_int,
    len: c_int,
    cmdlen: c_int,
) -> c_int {
    c_int::from(should_draw(cmdbuff_is_null != 0, start, len, cmdlen))
}

/// Check if redraw state needs any redraw (FFI).
#[no_mangle]
pub extern "C" fn rs_redraw_needs_any(state: c_int) -> c_int {
    let rs = match state {
        1 => RedrawState::Pos,
        2 => RedrawState::All,
        _ => RedrawState::None,
    };
    c_int::from(rs.needs_redraw())
}

/// Check if redraw state needs full redraw (FFI).
#[no_mangle]
pub extern "C" fn rs_redraw_needs_full(state: c_int) -> c_int {
    let rs = match state {
        1 => RedrawState::Pos,
        2 => RedrawState::All,
        _ => RedrawState::None,
    };
    c_int::from(rs.needs_full_redraw())
}

/// Merge two redraw states (FFI).
#[no_mangle]
pub extern "C" fn rs_redraw_merge(a: c_int, b: c_int) -> c_int {
    let ra = match a {
        1 => RedrawState::Pos,
        2 => RedrawState::All,
        _ => RedrawState::None,
    };
    let rb = match b {
        1 => RedrawState::Pos,
        2 => RedrawState::All,
        _ => RedrawState::None,
    };
    ra.merge(rb) as c_int
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cmdline_ui_state() {
        let mut state = CmdlineUiState::new();
        assert!(!state.dirty);
        assert!(!state.shown);

        state.mark_dirty();
        assert!(state.dirty);

        state.on_show(1);
        assert!(state.shown);
        assert_eq!(state.sent_level, 1);
        assert!(!state.dirty);

        state.on_pos(5);
        assert_eq!(state.sent_pos, 5);

        state.on_hide();
        assert!(!state.shown);
    }

    #[test]
    fn test_content_attr() {
        let attr = ContentAttr::new(10, 0, 5);
        assert_eq!(attr.hl_id, 10);
        assert_eq!(attr.start, 0);
        assert_eq!(attr.end, 5);
    }

    #[test]
    fn test_special_char_type() {
        assert_eq!(SpecialCharType::None.description(), "");
        assert_eq!(SpecialCharType::Digraph.description(), "digraph");
        assert_eq!(SpecialCharType::Literal.description(), "literal");
        assert_eq!(SpecialCharType::Register.description(), "register");
    }

    #[test]
    fn test_redraw_state() {
        assert!(!RedrawState::None.needs_redraw());
        assert!(RedrawState::Pos.needs_redraw());
        assert!(RedrawState::All.needs_redraw());

        assert!(!RedrawState::None.needs_full_redraw());
        assert!(!RedrawState::Pos.needs_full_redraw());
        assert!(RedrawState::All.needs_full_redraw());

        assert_eq!(
            RedrawState::None.merge(RedrawState::None),
            RedrawState::None
        );
        assert_eq!(RedrawState::None.merge(RedrawState::Pos), RedrawState::Pos);
        assert_eq!(RedrawState::Pos.merge(RedrawState::All), RedrawState::All);
        assert_eq!(RedrawState::All.merge(RedrawState::None), RedrawState::All);
    }

    #[test]
    fn test_prompt() {
        let prompt = Prompt::new(b':', 0, 0);
        assert!(prompt.is_ex_cmd());
        assert!(!prompt.is_search());
        assert_eq!(prompt.firstc_str(), ":");

        let search = Prompt::new(b'/', 0, 0);
        assert!(search.is_search());
        assert!(!search.is_ex_cmd());
        assert_eq!(search.firstc_str(), "/");
    }

    #[test]
    fn test_color_cache_valid() {
        // Same prompt ID, non-null cache
        assert!(color_cache_valid(1, 1, false));

        // Different prompt ID
        assert!(!color_cache_valid(1, 2, false));

        // Null cache buffer
        assert!(!color_cache_valid(1, 1, true));
    }

    #[test]
    fn test_should_skip_coloring() {
        // Same prompt, too many errors
        assert!(should_skip_coloring(1, 1, MAX_CALLBACK_ERRORS));
        assert!(should_skip_coloring(1, 1, MAX_CALLBACK_ERRORS + 1));

        // Same prompt, not enough errors
        assert!(!should_skip_coloring(1, 1, MAX_CALLBACK_ERRORS - 1));

        // Different prompt, many errors (should NOT skip - new prompt)
        assert!(!should_skip_coloring(2, 1, MAX_CALLBACK_ERRORS));
    }

    #[test]
    fn test_should_reset_callback_errors() {
        assert!(should_reset_callback_errors(2, 1));
        assert!(!should_reset_callback_errors(1, 1));
    }

    #[test]
    fn test_calculate_draw_len() {
        // Normal case
        assert_eq!(calculate_draw_len(0, 10, 20), 10);

        // Requested more than available
        assert_eq!(calculate_draw_len(15, 10, 20), 5);

        // Exactly at end
        assert_eq!(calculate_draw_len(10, 10, 20), 10);
    }

    #[test]
    fn test_should_draw() {
        // Normal case
        assert!(should_draw(false, 0, 10, 20));

        // Null buffer
        assert!(!should_draw(true, 0, 10, 20));

        // Start past end
        assert!(!should_draw(false, 25, 10, 20));

        // Zero length
        assert!(!should_draw(false, 0, 0, 20));

        // Negative length
        assert!(!should_draw(false, 0, -1, 20));
    }
}

// =============================================================================
// Phase 2: cmdline_screen_cleared and cmdline_ui_flush in Rust
// =============================================================================

#[allow(unsafe_code)]
mod phase2 {
    use std::ffi::{c_int, c_void};

    // kUICmdline enum value from ui_defs.h
    const K_UI_CMDLINE: c_int = 24;

    unsafe extern "C" {
        fn ui_has(what: c_int) -> c_int;
        fn nvim_get_cmdline_block_size() -> usize;
        fn nvim_ui_call_cmdline_block_show_all();
        fn nvim_get_ccline_prev_ptr() -> *mut c_void;
        fn nvim_ccline_ptr_get_level(p: *mut c_void) -> c_int;
        fn nvim_ccline_ptr_set_redraw_all(p: *mut c_void);
        fn nvim_ccline_ptr_get_prev(p: *mut c_void) -> *mut c_void;
        fn nvim_get_cmdwin_level() -> c_int;
        fn nvim_get_ccline_level() -> c_int;
    }

    /// Rust replacement for `cmdline_screen_cleared` in ex_getln.c.
    ///
    /// Extra redrawing needed for redraw! and on ui_attach.
    ///
    /// # Safety
    ///
    /// Must only be called when Neovim's cmdline state is valid.
    #[no_mangle]
    pub unsafe extern "C" fn cmdline_screen_cleared() {
        if ui_has(K_UI_CMDLINE) == 0 {
            return;
        }
        if nvim_get_cmdline_block_size() > 0 {
            nvim_ui_call_cmdline_block_show_all();
        }
        let mut prev_level = nvim_get_ccline_level() - 1;
        let mut line = nvim_get_ccline_prev_ptr();
        while prev_level > 0 && !line.is_null() {
            if nvim_ccline_ptr_get_level(line) == prev_level {
                // don't redraw a cmdline already shown in the cmdline window
                if prev_level != nvim_get_cmdwin_level() {
                    nvim_ccline_ptr_set_redraw_all(line);
                }
                prev_level -= 1;
            }
            line = nvim_ccline_ptr_get_prev(line);
        }
        crate::screen::redrawcmd_rs();
    }
}
