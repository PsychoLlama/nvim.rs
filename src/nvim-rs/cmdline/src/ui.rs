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
// Phase 2: block UI, per-level UI update, special_char, hide
// =============================================================================

#[allow(unsafe_code)]
mod phase2 {
    use std::ffi::{c_char, c_int, c_void};
    use std::mem;
    use std::ptr;

    use nvim_api::{Array, NvimString, Object, ObjectData};

    // kUICmdline enum value from ui_defs.h
    const K_UI_CMDLINE: c_int = 24;
    // kObjectTypeInteger = 1, kObjectTypeString = 3, kObjectTypeArray = 5
    const K_OBJECT_TYPE_INTEGER: c_int = 1;
    const K_OBJECT_TYPE_STRING: c_int = 3;
    const K_OBJECT_TYPE_ARRAY: c_int = 5;
    // CmdRedraw enum values
    const K_CMD_REDRAW_POS: c_int = 1;
    const K_CMD_REDRAW_ALL: c_int = 2;

    /// Rust-owned storage for the cmdline block (replaces C static `cmdline_block`).
    ///
    /// Single-threaded; all access is from the main UI thread.
    static mut CMDLINE_BLOCK: Array = Array {
        size: 0,
        capacity: 0,
        items: ptr::null_mut(),
    };

    unsafe extern "C" {
        fn ui_has(what: c_int) -> c_int;
        fn ui_call_cmdline_block_show(lines: Array);
        fn ui_call_cmdline_block_append(lines: Array);
        fn ui_call_cmdline_block_hide();
        fn ui_call_cmdline_show(
            content: Array,
            pos: i64,
            firstc: NvimString,
            prompt: NvimString,
            indent: i64,
            level: i64,
            hl_id: i64,
        );
        fn ui_call_cmdline_pos(pos: i64, level: i64);
        fn ui_call_cmdline_special_char(c: NvimString, shift: bool, level: i64);
        fn ui_call_cmdline_hide(level: i64, abort: bool);
        fn api_free_array(arr: Array);
        fn xmallocz(size: usize) -> *mut c_void;
        fn xrealloc(ptr: *mut c_void, size: usize) -> *mut c_void;
        fn cstr_as_string(s: *const c_char) -> NvimString;
        fn syn_id2attr(hl_id: c_int) -> c_int;
        fn utfc_ptr2len(p: *const c_char) -> c_int;

        // ccline accessors (opaque pointer)
        fn nvim_get_ccline_prev_ptr() -> *mut c_void;
        fn nvim_ccline_ptr_get_level(p: *mut c_void) -> c_int;
        fn nvim_ccline_ptr_set_redraw_all(p: *mut c_void);
        fn nvim_ccline_ptr_get_prev(p: *mut c_void) -> *mut c_void;
        fn nvim_get_cmdwin_level() -> c_int;
        fn nvim_get_ccline_level() -> c_int;
        fn nvim_get_ccline_self_ptr() -> *mut c_void;
        fn nvim_get_cmdline_was_last_drawn() -> c_int;
        fn nvim_set_cmdline_was_last_drawn(val: c_int);
        fn nvim_ccline_ptr_get_redraw_state(p: *mut c_void) -> c_int;
        fn nvim_ccline_ptr_set_redraw_none(p: *mut c_void);
        fn nvim_ccline_ptr_get_cmdpos(p: *mut c_void) -> c_int;
        fn nvim_ccline_ptr_get_cmdbuff(p: *mut c_void) -> *mut c_char;
        fn nvim_ccline_ptr_get_cmdfirstc(p: *mut c_void) -> c_int;
        fn nvim_ccline_ptr_get_cmdprompt(p: *mut c_void) -> *mut c_char;
        fn nvim_ccline_ptr_get_cmdindent(p: *mut c_void) -> c_int;
        fn nvim_ccline_ptr_get_hl_id(p: *mut c_void) -> c_int;
        fn nvim_ccline_ptr_get_special_char(p: *mut c_void) -> c_int;
        fn nvim_ccline_ptr_get_special_shift(p: *mut c_void) -> c_int;
        fn nvim_ccline_ptr_get_colors_size(p: *mut c_void) -> usize;
        fn nvim_ccline_ptr_get_color_chunk(
            p: *mut c_void,
            idx: usize,
            start_out: *mut c_int,
            end_out: *mut c_int,
            hl_id_out: *mut c_int,
        );

        // For nvim_cmdline_ui_hide
        fn nvim_set_ccline_redraw_state(state: c_int);

        // cmdline_star global
        fn nvim_get_cmdline_star() -> c_int;
        // ccline.cmdbuff accessed via nvim_get_ccline_cmdbuff
        fn nvim_get_ccline_cmdbuff() -> *mut c_char;
    }

    // =========================================================================
    // Array helpers
    // =========================================================================

    /// Push an item onto a heap-allocated Array (kv_push equivalent).
    unsafe fn array_push(arr: &mut Array, item: Object) {
        if arr.size == arr.capacity {
            let new_cap = if arr.capacity == 0 {
                4
            } else {
                arr.capacity * 2
            };
            arr.items = xrealloc(
                arr.items.cast::<c_void>(),
                new_cap * mem::size_of::<Object>(),
            )
            .cast::<Object>();
            arr.capacity = new_cap;
        }
        *arr.items.add(arr.size) = item;
        arr.size += 1;
    }

    /// Create an empty heap-allocated Array.
    const fn empty_array() -> Array {
        Array {
            size: 0,
            capacity: 0,
            items: ptr::null_mut(),
        }
    }

    /// Build an Object wrapping an integer.
    fn integer_obj(n: i64) -> Object {
        Object {
            obj_type: K_OBJECT_TYPE_INTEGER,
            data: ObjectData { integer: n },
        }
    }

    /// Build an Object wrapping a string.
    fn string_obj(s: NvimString) -> Object {
        Object {
            obj_type: K_OBJECT_TYPE_STRING,
            data: ObjectData { string: s },
        }
    }

    /// Build an Object wrapping an Array.
    fn array_obj(a: Array) -> Object {
        Object {
            obj_type: K_OBJECT_TYPE_ARRAY,
            data: ObjectData { array: a },
        }
    }

    /// Inline equivalent of C `cbuf_as_string_rs(buf, size)` — borrows, no copy.
    fn cbuf_as_string_rs(buf: *mut c_char, size: usize) -> NvimString {
        NvimString { data: buf, size }
    }

    // =========================================================================
    // Block UI
    // =========================================================================

    /// Rust replacement for C `ui_ext_cmdline_block_append`.
    ///
    /// Appends `line` (with `indent` leading spaces) to the cmdline block Array
    /// and notifies the UI via `ui_call_cmdline_block_append` or
    /// `ui_call_cmdline_block_show`.
    ///
    /// # Safety
    ///
    /// Must only be called on the main UI thread.
    #[unsafe(export_name = "ui_ext_cmdline_block_append")]
    pub unsafe extern "C" fn cmdline_block_append(indent: usize, line: *const c_char) {
        // Build the indented string into a heap buffer.
        let line_len = libc::strlen(line);
        let total = indent + line_len;
        let buf = xmallocz(total).cast::<c_char>();
        libc::memset(buf.cast::<c_void>(), b' ' as c_int, indent);
        libc::memcpy(
            buf.add(indent).cast::<c_void>(),
            line.cast::<c_void>(),
            line_len,
        );

        // Build [0, text, 0] item Array.
        let mut item = empty_array();
        array_push(&mut item, integer_obj(0));
        array_push(&mut item, string_obj(cbuf_as_string_rs(buf, total)));
        array_push(&mut item, integer_obj(0));

        // Build [[item]] content Array.
        let mut content = empty_array();
        array_push(&mut content, array_obj(item));

        // Append to the block.
        let block = &raw mut CMDLINE_BLOCK;
        if (*block).size > 0 {
            // Already showing — append and notify.
            array_push(&mut *block, array_obj(content));
            // The last entry in the block is a content Array; send it to the UI.
            let last_idx = (*block).size - 1;
            let last_item = &(*(*block).items.add(last_idx)).data.array;
            ui_call_cmdline_block_append(*last_item);
        } else {
            array_push(&mut *block, array_obj(content));
            ui_call_cmdline_block_show(*block);
        }
    }

    /// Rust replacement for C `ui_ext_cmdline_block_leave`.
    ///
    /// Frees the cmdline block Array and hides the UI block.
    ///
    /// # Safety
    ///
    /// Must only be called on the main UI thread.
    #[unsafe(export_name = "ui_ext_cmdline_block_leave")]
    pub unsafe extern "C" fn cmdline_block_leave() {
        let block = &raw mut CMDLINE_BLOCK;
        api_free_array(*block);
        *block = empty_array();
        ui_call_cmdline_block_hide();
    }

    // =========================================================================
    // Per-level UI update
    // =========================================================================

    /// Build and emit the cmdline UI event for a single cmdline level.
    ///
    /// Replaces C `nvim_cmdline_ui_update_for_level`.
    /// `show=true` → `ui_call_cmdline_show` (full redraw).
    /// `show=false` → `ui_call_cmdline_pos` (position update only).
    ///
    /// # Safety
    ///
    /// `line` must be a valid `CmdlineInfo *` pointer.
    unsafe fn update_for_level(line: *mut c_void, show: bool) {
        if !show {
            ui_call_cmdline_pos(
                i64::from(nvim_ccline_ptr_get_cmdpos(line)),
                i64::from(nvim_ccline_ptr_get_level(line)),
            );
            return;
        }

        let cmdpos = nvim_ccline_ptr_get_cmdpos(line);
        let level = nvim_ccline_ptr_get_level(line);
        let cmdfirstc = nvim_ccline_ptr_get_cmdfirstc(line) as u8;
        let cmdprompt = nvim_ccline_ptr_get_cmdprompt(line);
        let cmdindent = nvim_ccline_ptr_get_cmdindent(line);
        let hl_id = nvim_ccline_ptr_get_hl_id(line);
        let special_char_raw = nvim_ccline_ptr_get_special_char(line);
        let special_shift = nvim_ccline_ptr_get_special_shift(line) != 0;

        let mut content = empty_array();

        if nvim_get_cmdline_star() != 0 {
            // Password mode: replace all chars with '*'.
            let cmdbuff = nvim_get_ccline_cmdbuff();
            let mut len: usize = 0;
            if !cmdbuff.is_null() {
                let mut q = cmdbuff;
                while *q != 0 {
                    len += 1;
                    let adv = utfc_ptr2len(q);
                    q = q.add(adv as usize);
                }
            }
            let buf = xmallocz(len).cast::<c_char>();
            libc::memset(buf.cast::<c_void>(), b'*' as c_int, len);

            let mut item = empty_array();
            array_push(&mut item, integer_obj(0));
            array_push(&mut item, string_obj(cbuf_as_string_rs(buf, len)));
            array_push(&mut item, integer_obj(0));
            array_push(&mut content, array_obj(item));
        } else {
            let colors_size = nvim_ccline_ptr_get_colors_size(line);
            let cmdbuff = nvim_ccline_ptr_get_cmdbuff(line);
            if colors_size > 0 {
                for i in 0..colors_size {
                    let mut start: c_int = 0;
                    let mut end: c_int = 0;
                    let mut chunk_hl: c_int = 0;
                    nvim_ccline_ptr_get_color_chunk(
                        line,
                        i,
                        &raw mut start,
                        &raw mut end,
                        &raw mut chunk_hl,
                    );
                    let attr = if chunk_hl == 0 {
                        0_i64
                    } else {
                        i64::from(syn_id2attr(chunk_hl))
                    };
                    let text_ptr = cmdbuff.add(start as usize);
                    let text_len = (end - start) as usize;

                    let mut item = empty_array();
                    array_push(&mut item, integer_obj(attr));
                    array_push(&mut item, string_obj(cbuf_as_string_rs(text_ptr, text_len)));
                    array_push(&mut item, integer_obj(i64::from(chunk_hl)));
                    array_push(&mut content, array_obj(item));
                }
            } else {
                let mut item = empty_array();
                array_push(&mut item, integer_obj(0));
                array_push(&mut item, string_obj(cstr_as_string(cmdbuff)));
                array_push(&mut item, integer_obj(0));
                array_push(&mut content, array_obj(item));
            }
        }

        // Build firstc string (single char or empty).
        let mut firstc_buf: [c_char; 2] = [cmdfirstc as c_char, 0];
        let firstc_str = cstr_as_string(firstc_buf.as_mut_ptr());

        ui_call_cmdline_show(
            content,
            i64::from(cmdpos),
            firstc_str,
            cstr_as_string(cmdprompt),
            i64::from(cmdindent),
            i64::from(level),
            i64::from(hl_id),
        );

        if special_char_raw != 0 {
            let mut sc_buf: [c_char; 2] = [special_char_raw as c_char, 0];
            ui_call_cmdline_special_char(
                cstr_as_string(sc_buf.as_mut_ptr()),
                special_shift,
                i64::from(level),
            );
        }
    }

    // =========================================================================
    // Special char emission
    // =========================================================================

    /// Rust replacement for C `nvim_ui_cmdline_special_char`.
    ///
    /// # Safety
    ///
    /// Must only be called on the main UI thread.
    #[unsafe(no_mangle)]
    pub unsafe extern "C" fn nvim_ui_cmdline_special_char(c: c_int, shift: bool, level: c_int) {
        let mut buf: [c_char; 2] = [c as c_char, 0];
        ui_call_cmdline_special_char(cstr_as_string(buf.as_mut_ptr()), shift, i64::from(level));
    }

    // =========================================================================
    // Cmdline hide
    // =========================================================================

    /// Rust replacement for C `nvim_cmdline_ui_hide`.
    ///
    /// # Safety
    ///
    /// Must only be called on the main UI thread.
    #[unsafe(no_mangle)]
    pub unsafe extern "C" fn nvim_cmdline_ui_hide(gotesc: c_int) {
        if ui_has(K_UI_CMDLINE) != 0 {
            nvim_set_cmdline_was_last_drawn(0);
            nvim_set_ccline_redraw_state(0); // kCmdRedrawNone
            ui_call_cmdline_hide(i64::from(nvim_get_ccline_level()), gotesc != 0);
        }
    }

    // =========================================================================
    // Screen cleared / UI flush
    // =========================================================================

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
        if CMDLINE_BLOCK.size > 0 {
            ui_call_cmdline_block_show(CMDLINE_BLOCK);
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

    /// Rust replacement for `cmdline_ui_flush` in ex_getln.c.
    ///
    /// Called by ui_flush to flush pending cmdline UI events to external UIs.
    ///
    /// # Safety
    ///
    /// Must only be called when Neovim's cmdline state is valid.
    #[unsafe(no_mangle)]
    pub unsafe extern "C" fn rs_cmdline_ui_flush() {
        if ui_has(K_UI_CMDLINE) == 0 {
            return;
        }
        let mut level = nvim_get_ccline_level();
        let mut line = nvim_get_ccline_self_ptr();
        while level > 0 && !line.is_null() {
            if nvim_ccline_ptr_get_level(line) == level {
                let redraw_state = nvim_ccline_ptr_get_redraw_state(line);
                nvim_ccline_ptr_set_redraw_none(line);
                if redraw_state == K_CMD_REDRAW_ALL {
                    nvim_set_cmdline_was_last_drawn(1);
                    update_for_level(line, true);
                } else if redraw_state == K_CMD_REDRAW_POS && nvim_get_cmdline_was_last_drawn() != 0
                {
                    update_for_level(line, false);
                }
                level -= 1;
            }
            line = nvim_ccline_ptr_get_prev(line);
        }
    }
}
