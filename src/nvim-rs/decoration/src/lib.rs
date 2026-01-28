//! Decoration and virtual text system for Neovim
//!
//! This crate provides Rust implementations for decoration handling
//! from `src/nvim/decoration.c`, focusing on virtual text positioning.

#![allow(unsafe_code)]
#![allow(clippy::doc_markdown)]
#![allow(clippy::similar_names)]
#![allow(clippy::too_many_arguments)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::cast_possible_wrap)]
#![allow(clippy::cast_precision_loss)]
#![allow(clippy::missing_safety_doc)]
#![allow(clippy::must_use_candidate)]
#![allow(clippy::missing_const_for_fn)]
#![allow(clippy::match_same_arms)]
#![allow(dead_code)]

pub mod cache;
pub mod decor;
pub mod invoke;
pub mod provider;

use std::ffi::{c_char, c_int, c_void};

/// schar_T is stored as a u32.
pub type ScharT = u32;

/// Opaque handle to DecorState.
#[repr(transparent)]
#[derive(Debug, Clone, Copy)]
pub struct DecorStateHandle(*mut c_void);

impl DecorStateHandle {
    /// Check if the handle is null.
    #[allow(clippy::missing_const_for_fn)]
    pub fn is_null(self) -> bool {
        self.0.is_null()
    }
}

/// Opaque handle to DecorRange.
#[repr(transparent)]
#[derive(Debug, Clone, Copy)]
pub struct DecorRangeHandle(*mut c_void);

impl DecorRangeHandle {
    /// Check if the handle is null.
    #[allow(clippy::missing_const_for_fn)]
    pub fn is_null(self) -> bool {
        self.0.is_null()
    }
}

/// Opaque handle to DecorVirtText.
#[repr(transparent)]
#[derive(Debug, Clone, Copy)]
pub struct DecorVirtTextHandle(*mut c_void);

impl DecorVirtTextHandle {
    /// Check if the handle is null.
    #[allow(clippy::missing_const_for_fn)]
    pub fn is_null(self) -> bool {
        self.0.is_null()
    }
}

/// Opaque handle to VirtText (kvec_t(VirtTextChunk)).
#[repr(transparent)]
#[derive(Debug, Clone, Copy)]
pub struct VirtTextHandle(*mut c_void);

impl VirtTextHandle {
    /// Check if the handle is null.
    #[allow(clippy::missing_const_for_fn)]
    pub fn is_null(self) -> bool {
        self.0.is_null()
    }
}

/// Opaque handle to window (win_T).
#[repr(transparent)]
#[derive(Debug, Clone, Copy)]
pub struct WinHandle(*mut c_void);

impl WinHandle {
    /// Check if the handle is null.
    #[allow(clippy::missing_const_for_fn)]
    pub fn is_null(self) -> bool {
        self.0.is_null()
    }
}

/// Virtual text position types (matches VirtTextPos enum in C).
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VirtTextPos {
    EndOfLine = 0,
    EndOfLineRightAlign = 1,
    Inline = 2,
    Overlay = 3,
    RightAlign = 4,
    WinCol = 5,
}

impl VirtTextPos {
    /// Convert from C int to VirtTextPos.
    pub const fn from_c_int(val: c_int) -> Option<Self> {
        match val {
            0 => Some(Self::EndOfLine),
            1 => Some(Self::EndOfLineRightAlign),
            2 => Some(Self::Inline),
            3 => Some(Self::Overlay),
            4 => Some(Self::RightAlign),
            5 => Some(Self::WinCol),
            _ => None,
        }
    }
}

/// Highlight mode types (matches HlMode enum in C).
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HlMode {
    Unknown = 0,
    Replace = 1,
    Combine = 2,
    Blend = 3,
}

impl HlMode {
    /// Convert from C int to HlMode.
    pub const fn from_c_int(val: c_int) -> Option<Self> {
        match val {
            0 => Some(Self::Unknown),
            1 => Some(Self::Replace),
            2 => Some(Self::Combine),
            3 => Some(Self::Blend),
            _ => None,
        }
    }
}

/// Decoration range kind.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DecorKind {
    Highlight = 0,
    Sign = 1,
    VirtText = 2,
    VirtLines = 3,
    UIWatched = 4,
}

impl DecorKind {
    /// Convert from C int to DecorKind.
    pub const fn from_c_int(val: c_int) -> Option<Self> {
        match val {
            0 => Some(Self::Highlight),
            1 => Some(Self::Sign),
            2 => Some(Self::VirtText),
            3 => Some(Self::VirtLines),
            4 => Some(Self::UIWatched),
            _ => None,
        }
    }
}

/// Constants for special draw_col values.
pub const DRAW_COL_UNSET: c_int = -1;
pub const DRAW_COL_PENDING: c_int = -3;
pub const DRAW_COL_JUST_ADDED: c_int = -10;
pub const DRAW_COL_DISABLED: c_int = c_int::MIN;

/// VirtText flags.
pub const KVT_IS_LINES: u8 = 1;
pub const KVT_HIDE: u8 = 2;
pub const KVT_LINES_ABOVE: u8 = 4;
pub const KVT_REPEAT_LINEBREAK: u8 = 8;

// ============================================================================
// C accessor functions for DecorState
// ============================================================================

extern "C" {
    // DecorState accessors
    fn nvim_get_decor_state() -> DecorStateHandle;
    fn nvim_decor_state_get_row(state: DecorStateHandle) -> c_int;
    fn nvim_decor_state_get_eol_col(state: DecorStateHandle) -> c_int;
    fn nvim_decor_state_set_eol_col(state: DecorStateHandle, val: c_int);
    fn nvim_decor_state_get_current_end(state: DecorStateHandle) -> c_int;
    fn nvim_decor_state_get_current(state: DecorStateHandle) -> c_int;
    fn nvim_decor_state_get_col_until(state: DecorStateHandle) -> c_int;
    fn nvim_decor_state_get_conceal(state: DecorStateHandle) -> c_int;
    fn nvim_decor_state_get_conceal_char(state: DecorStateHandle) -> ScharT;
    fn nvim_decor_state_get_conceal_attr(state: DecorStateHandle) -> c_int;
    fn nvim_decor_state_get_spell(state: DecorStateHandle) -> c_int;
    fn nvim_decor_state_get_win(state: DecorStateHandle) -> WinHandle;
    fn nvim_decor_state_get_top_row(state: DecorStateHandle) -> c_int;
    fn nvim_decor_state_get_range(state: DecorStateHandle, idx: c_int) -> DecorRangeHandle;

    // DecorRange accessors
    fn nvim_decor_range_get_start_row(range: DecorRangeHandle) -> c_int;
    fn nvim_decor_range_get_start_col(range: DecorRangeHandle) -> c_int;
    fn nvim_decor_range_get_end_row(range: DecorRangeHandle) -> c_int;
    fn nvim_decor_range_get_end_col(range: DecorRangeHandle) -> c_int;
    fn nvim_decor_range_get_draw_col(range: DecorRangeHandle) -> c_int;
    fn nvim_decor_range_set_draw_col(range: DecorRangeHandle, val: c_int);
    fn nvim_decor_range_get_kind(range: DecorRangeHandle) -> c_int;
    fn nvim_decor_range_get_attr_id(range: DecorRangeHandle) -> c_int;
    fn nvim_decor_range_get_priority(range: DecorRangeHandle) -> u32;
    fn nvim_decor_range_has_virt_pos(range: DecorRangeHandle) -> bool;
    fn nvim_decor_range_get_virt_pos_kind(range: DecorRangeHandle) -> c_int;
    fn nvim_decor_range_get_virt_text(range: DecorRangeHandle) -> DecorVirtTextHandle;

    // DecorVirtText accessors
    fn nvim_decor_virt_text_get_hl_mode(vt: DecorVirtTextHandle) -> c_int;
    fn nvim_decor_virt_text_get_pos(vt: DecorVirtTextHandle) -> c_int;
    fn nvim_decor_virt_text_get_width(vt: DecorVirtTextHandle) -> c_int;
    fn nvim_decor_virt_text_get_col(vt: DecorVirtTextHandle) -> c_int;
    fn nvim_decor_virt_text_get_flags(vt: DecorVirtTextHandle) -> c_int;
    fn nvim_decor_virt_text_get_chunk_count(vt: DecorVirtTextHandle) -> usize;
    fn nvim_decor_virt_text_get_chunk_text(vt: DecorVirtTextHandle, idx: usize) -> *const c_char;
    fn nvim_decor_virt_text_get_chunk_hl_id(vt: DecorVirtTextHandle, idx: usize) -> c_int;
    fn nvim_decor_virt_text_get_virt_text(vt: DecorVirtTextHandle) -> VirtTextHandle;

    // VirtText iteration
    fn nvim_next_virt_text_chunk(
        vt: VirtTextHandle,
        pos: *mut usize,
        attr: *mut c_int,
    ) -> *const c_char;

    // UIWatched accessors
    fn nvim_decor_range_get_ui_ns_id(range: DecorRangeHandle) -> u64;
    fn nvim_decor_range_get_ui_mark_id(range: DecorRangeHandle) -> u32;

    // win_extmark_arr push
    fn nvim_win_extmark_push(ns_id: u64, mark_id: u64, win_row: c_int, win_col: c_int);

    // High-level iteration helpers for draw_virt_text
    fn nvim_decor_state_get_active_range(state: DecorStateHandle, i: c_int) -> DecorRangeHandle;
    fn nvim_decor_state_get_eol_right_width(state: DecorStateHandle, from_idx: c_int) -> c_int;
}

// ============================================================================
// DecorState wrapper functions
// ============================================================================

/// Get the global decor_state.
pub fn get_decor_state() -> DecorStateHandle {
    unsafe { nvim_get_decor_state() }
}

/// Get the current row from decor_state.
pub fn decor_state_row(state: DecorStateHandle) -> c_int {
    unsafe { nvim_decor_state_get_row(state) }
}

/// Get the EOL column from decor_state.
pub fn decor_state_eol_col(state: DecorStateHandle) -> c_int {
    unsafe { nvim_decor_state_get_eol_col(state) }
}

/// Set the EOL column in decor_state.
pub fn decor_state_set_eol_col(state: DecorStateHandle, val: c_int) {
    unsafe { nvim_decor_state_set_eol_col(state, val) }
}

/// Get the current_end from decor_state (number of active ranges).
pub fn decor_state_current_end(state: DecorStateHandle) -> c_int {
    unsafe { nvim_decor_state_get_current_end(state) }
}

/// Get the current attr from decor_state.
pub fn decor_state_current(state: DecorStateHandle) -> c_int {
    unsafe { nvim_decor_state_get_current(state) }
}

/// Get the col_until from decor_state.
pub fn decor_state_col_until(state: DecorStateHandle) -> c_int {
    unsafe { nvim_decor_state_get_col_until(state) }
}

/// Get the conceal from decor_state.
pub fn decor_state_conceal(state: DecorStateHandle) -> c_int {
    unsafe { nvim_decor_state_get_conceal(state) }
}

/// Get the conceal_char from decor_state.
pub fn decor_state_conceal_char(state: DecorStateHandle) -> ScharT {
    unsafe { nvim_decor_state_get_conceal_char(state) }
}

/// Get the conceal_attr from decor_state.
pub fn decor_state_conceal_attr(state: DecorStateHandle) -> c_int {
    unsafe { nvim_decor_state_get_conceal_attr(state) }
}

/// Get the spell from decor_state.
pub fn decor_state_spell(state: DecorStateHandle) -> c_int {
    unsafe { nvim_decor_state_get_spell(state) }
}

/// Get the window from decor_state.
pub fn decor_state_win(state: DecorStateHandle) -> WinHandle {
    unsafe { nvim_decor_state_get_win(state) }
}

/// Get the top_row from decor_state.
pub fn decor_state_top_row(state: DecorStateHandle) -> c_int {
    unsafe { nvim_decor_state_get_top_row(state) }
}

/// Get a DecorRange by index.
pub fn decor_state_get_range(state: DecorStateHandle, idx: c_int) -> DecorRangeHandle {
    unsafe { nvim_decor_state_get_range(state, idx) }
}

// ============================================================================
// DecorRange wrapper functions
// ============================================================================

/// Get the start_row from a DecorRange.
pub fn decor_range_start_row(range: DecorRangeHandle) -> c_int {
    unsafe { nvim_decor_range_get_start_row(range) }
}

/// Get the start_col from a DecorRange.
pub fn decor_range_start_col(range: DecorRangeHandle) -> c_int {
    unsafe { nvim_decor_range_get_start_col(range) }
}

/// Get the end_row from a DecorRange.
pub fn decor_range_end_row(range: DecorRangeHandle) -> c_int {
    unsafe { nvim_decor_range_get_end_row(range) }
}

/// Get the end_col from a DecorRange.
pub fn decor_range_end_col(range: DecorRangeHandle) -> c_int {
    unsafe { nvim_decor_range_get_end_col(range) }
}

/// Get the draw_col from a DecorRange.
pub fn decor_range_draw_col(range: DecorRangeHandle) -> c_int {
    unsafe { nvim_decor_range_get_draw_col(range) }
}

/// Set the draw_col in a DecorRange.
pub fn decor_range_set_draw_col(range: DecorRangeHandle, val: c_int) {
    unsafe { nvim_decor_range_set_draw_col(range, val) }
}

/// Get the kind from a DecorRange.
pub fn decor_range_kind(range: DecorRangeHandle) -> Option<DecorKind> {
    DecorKind::from_c_int(unsafe { nvim_decor_range_get_kind(range) })
}

/// Get the attr_id from a DecorRange.
pub fn decor_range_attr_id(range: DecorRangeHandle) -> c_int {
    unsafe { nvim_decor_range_get_attr_id(range) }
}

/// Get the priority from a DecorRange.
pub fn decor_range_priority(range: DecorRangeHandle) -> u32 {
    unsafe { nvim_decor_range_get_priority(range) }
}

/// Check if a DecorRange has virtual text position.
pub fn decor_range_has_virt_pos(range: DecorRangeHandle) -> bool {
    unsafe { nvim_decor_range_has_virt_pos(range) }
}

/// Get the virtual text position kind from a DecorRange.
pub fn decor_range_virt_pos_kind(range: DecorRangeHandle) -> Option<VirtTextPos> {
    VirtTextPos::from_c_int(unsafe { nvim_decor_range_get_virt_pos_kind(range) })
}

/// Get the DecorVirtText from a DecorRange.
pub fn decor_range_virt_text(range: DecorRangeHandle) -> DecorVirtTextHandle {
    unsafe { nvim_decor_range_get_virt_text(range) }
}

// ============================================================================
// DecorVirtText wrapper functions
// ============================================================================

/// Get the hl_mode from a DecorVirtText.
pub fn virt_text_hl_mode(vt: DecorVirtTextHandle) -> Option<HlMode> {
    HlMode::from_c_int(unsafe { nvim_decor_virt_text_get_hl_mode(vt) })
}

/// Get the pos from a DecorVirtText.
pub fn virt_text_pos(vt: DecorVirtTextHandle) -> Option<VirtTextPos> {
    VirtTextPos::from_c_int(unsafe { nvim_decor_virt_text_get_pos(vt) })
}

/// Get the width from a DecorVirtText.
pub fn virt_text_width(vt: DecorVirtTextHandle) -> c_int {
    unsafe { nvim_decor_virt_text_get_width(vt) }
}

/// Get the col from a DecorVirtText.
pub fn virt_text_col(vt: DecorVirtTextHandle) -> c_int {
    unsafe { nvim_decor_virt_text_get_col(vt) }
}

/// Get the flags from a DecorVirtText.
pub fn virt_text_flags(vt: DecorVirtTextHandle) -> c_int {
    unsafe { nvim_decor_virt_text_get_flags(vt) }
}

/// Get the number of chunks in a VirtText.
pub fn virt_text_chunk_count(vt: DecorVirtTextHandle) -> usize {
    unsafe { nvim_decor_virt_text_get_chunk_count(vt) }
}

/// Get a chunk text from a VirtText by index.
/// Returns None if index is out of bounds or text is null.
pub fn virt_text_chunk_text(vt: DecorVirtTextHandle, idx: usize) -> Option<*const c_char> {
    let ptr = unsafe { nvim_decor_virt_text_get_chunk_text(vt, idx) };
    if ptr.is_null() {
        None
    } else {
        Some(ptr)
    }
}

/// Get a chunk hl_id from a VirtText by index.
pub fn virt_text_chunk_hl_id(vt: DecorVirtTextHandle, idx: usize) -> c_int {
    unsafe { nvim_decor_virt_text_get_chunk_hl_id(vt, idx) }
}

/// Get the VirtText handle from a DecorVirtText.
pub fn virt_text_get_virt_text(vt: DecorVirtTextHandle) -> VirtTextHandle {
    unsafe { nvim_decor_virt_text_get_virt_text(vt) }
}

// ============================================================================
// VirtText iteration wrapper functions
// ============================================================================

/// Iterator for VirtText chunks.
/// Returns the next text chunk or None if no more chunks.
/// Updates pos and attr through the C function.
///
/// # Safety
/// pos and attr must be valid pointers.
pub unsafe fn next_virt_text_chunk(
    vt: VirtTextHandle,
    pos: *mut usize,
    attr: *mut c_int,
) -> Option<*const c_char> {
    let ptr = nvim_next_virt_text_chunk(vt, pos, attr);
    if ptr.is_null() {
        None
    } else {
        Some(ptr)
    }
}

// ============================================================================
// UIWatched accessor wrapper functions
// ============================================================================

/// Get the ns_id from a UIWatched DecorRange.
pub fn decor_range_ui_ns_id(range: DecorRangeHandle) -> u64 {
    unsafe { nvim_decor_range_get_ui_ns_id(range) }
}

/// Get the mark_id from a UIWatched DecorRange.
pub fn decor_range_ui_mark_id(range: DecorRangeHandle) -> u32 {
    unsafe { nvim_decor_range_get_ui_mark_id(range) }
}

// ============================================================================
// win_extmark_arr wrapper functions
// ============================================================================

/// Push a WinExtmark to the global win_extmark_arr.
pub fn win_extmark_push(ns_id: u64, mark_id: u64, win_row: c_int, win_col: c_int) {
    unsafe { nvim_win_extmark_push(ns_id, mark_id, win_row, win_col) }
}

// ============================================================================
// High-level iteration wrappers for draw_virt_text
// ============================================================================

/// Get an active DecorRange by iteration index.
pub fn decor_state_get_active_range(state: DecorStateHandle, i: c_int) -> DecorRangeHandle {
    unsafe { nvim_decor_state_get_active_range(state, i) }
}

/// Get the total width of EOL right-aligned virtual text.
pub fn decor_state_get_eol_right_width(state: DecorStateHandle, from_idx: c_int) -> c_int {
    unsafe { nvim_decor_state_get_eol_right_width(state, from_idx) }
}

// ============================================================================
// FFI exports
// ============================================================================

/// Check if a DecorRange has a virtual position (virtual text or ui_watched).
/// This is the Rust implementation of decor_virt_pos().
#[no_mangle]
pub extern "C" fn rs_decor_virt_pos(range: DecorRangeHandle) -> c_int {
    if range.is_null() {
        return 0;
    }
    let kind = decor_range_kind(range);
    c_int::from(kind == Some(DecorKind::VirtText) || kind == Some(DecorKind::UIWatched))
}

/// Get the virtual text position kind from a DecorRange.
/// This is the Rust implementation of decor_virt_pos_kind().
/// Returns VirtTextPos enum value.
#[no_mangle]
pub extern "C" fn rs_decor_virt_pos_kind(range: DecorRangeHandle) -> c_int {
    if range.is_null() {
        return VirtTextPos::EndOfLine as c_int;
    }
    let kind = decor_range_kind(range);
    if kind == Some(DecorKind::VirtText) || kind == Some(DecorKind::UIWatched) {
        unsafe { nvim_decor_range_get_virt_pos_kind(range) }
    } else {
        VirtTextPos::EndOfLine as c_int
    }
}

// ============================================================================
// Phase 3: Conceal and Decoration Attribute Helpers
// ============================================================================

/// Check if concealment is active in the current decoration state.
///
/// Returns true if decor_state.conceal > 0.
#[no_mangle]
pub extern "C" fn rs_conceal_check(state: DecorStateHandle) -> c_int {
    if state.is_null() {
        return 0;
    }
    c_int::from(decor_state_conceal(state) > 0)
}

/// Check if concealment should show a replacement character.
///
/// Returns true if conceal level allows showing a character (level 1 or 2).
#[no_mangle]
pub extern "C" fn rs_conceal_shows_char(state: DecorStateHandle) -> c_int {
    if state.is_null() {
        return 0;
    }
    let conceal = decor_state_conceal(state);
    c_int::from(conceal == 1 || conceal == 2)
}

/// Check if concealment is full (completely hides text).
///
/// Returns true if conceal level is 3 (full concealment).
#[no_mangle]
pub extern "C" fn rs_conceal_is_full(state: DecorStateHandle) -> c_int {
    if state.is_null() {
        return 0;
    }
    c_int::from(decor_state_conceal(state) >= 3)
}

/// Get the conceal character if one is set.
///
/// Returns the conceal_char from decor_state, or 0 if no character is set.
#[no_mangle]
pub extern "C" fn rs_get_conceal_char(state: DecorStateHandle) -> ScharT {
    if state.is_null() {
        return 0;
    }
    decor_state_conceal_char(state)
}

/// Get the conceal attribute if one is set.
///
/// Returns the conceal_attr from decor_state, or 0 if no attribute is set.
#[no_mangle]
pub extern "C" fn rs_get_conceal_attr(state: DecorStateHandle) -> c_int {
    if state.is_null() {
        return 0;
    }
    decor_state_conceal_attr(state)
}

/// Check if decoration has a custom conceal character.
///
/// Returns true if decor_state has a non-zero conceal_char.
#[no_mangle]
pub extern "C" fn rs_has_conceal_char(state: DecorStateHandle) -> c_int {
    if state.is_null() {
        return 0;
    }
    c_int::from(decor_state_conceal_char(state) != 0)
}

/// Check if decoration has a custom conceal attribute.
///
/// Returns true if decor_state has a non-zero conceal_attr.
#[no_mangle]
pub extern "C" fn rs_has_conceal_attr(state: DecorStateHandle) -> c_int {
    if state.is_null() {
        return 0;
    }
    c_int::from(decor_state_conceal_attr(state) != 0)
}

/// Get the spell state from decoration.
///
/// Returns the spell tristate: -1 = inherit, 0 = spell off, 1 = spell on.
#[no_mangle]
pub extern "C" fn rs_get_decor_spell(state: DecorStateHandle) -> c_int {
    if state.is_null() {
        return -1; // Inherit
    }
    decor_state_spell(state)
}

/// Check if decoration forces spell checking on.
#[no_mangle]
pub extern "C" fn rs_decor_spell_on(state: DecorStateHandle) -> c_int {
    if state.is_null() {
        return 0;
    }
    c_int::from(decor_state_spell(state) == 1)
}

/// Check if decoration forces spell checking off.
#[no_mangle]
pub extern "C" fn rs_decor_spell_off(state: DecorStateHandle) -> c_int {
    if state.is_null() {
        return 0;
    }
    c_int::from(decor_state_spell(state) == 0)
}

/// Get decoration attributes for a specific column.
///
/// Returns the col_until value indicating how far the current decoration extends.
#[no_mangle]
pub extern "C" fn rs_get_decor_col_until(state: DecorStateHandle) -> c_int {
    if state.is_null() {
        return 0;
    }
    decor_state_col_until(state)
}

/// Check if we're past the decoration column extent.
///
/// Returns true if `col >= col_until`, meaning decoration needs refresh.
#[no_mangle]
pub extern "C" fn rs_decor_needs_refresh(state: DecorStateHandle, col: c_int) -> c_int {
    if state.is_null() {
        return 1;
    }
    c_int::from(col >= decor_state_col_until(state))
}

/// Get the current decoration row.
#[no_mangle]
pub extern "C" fn rs_get_decor_row(state: DecorStateHandle) -> c_int {
    if state.is_null() {
        return -1;
    }
    decor_state_row(state)
}

/// Check if decoration state is on a specific row.
#[no_mangle]
pub extern "C" fn rs_decor_on_row(state: DecorStateHandle, row: c_int) -> c_int {
    if state.is_null() {
        return 0;
    }
    c_int::from(decor_state_row(state) == row)
}

/// Get the number of active decoration ranges.
#[no_mangle]
pub extern "C" fn rs_get_active_decor_count(state: DecorStateHandle) -> c_int {
    if state.is_null() {
        return 0;
    }
    decor_state_current_end(state)
}

/// Check if there are any active decorations.
#[no_mangle]
pub extern "C" fn rs_has_active_decor(state: DecorStateHandle) -> c_int {
    if state.is_null() {
        return 0;
    }
    c_int::from(decor_state_current_end(state) > 0)
}

/// Get the current decoration attribute ID.
#[no_mangle]
pub extern "C" fn rs_get_decor_attr(state: DecorStateHandle) -> c_int {
    if state.is_null() {
        return 0;
    }
    decor_state_current(state)
}

/// Check if decoration range is for the current row.
#[no_mangle]
pub extern "C" fn rs_decor_range_on_row(range: DecorRangeHandle, row: c_int) -> c_int {
    if range.is_null() {
        return 0;
    }
    c_int::from(decor_range_start_row(range) == row)
}

/// Check if decoration range starts at or before a column.
#[no_mangle]
pub extern "C" fn rs_decor_range_starts_by(range: DecorRangeHandle, col: c_int) -> c_int {
    if range.is_null() {
        return 0;
    }
    c_int::from(decor_range_start_col(range) <= col)
}

/// Check if decoration range ends after a column.
#[no_mangle]
pub extern "C" fn rs_decor_range_ends_after(range: DecorRangeHandle, col: c_int) -> c_int {
    if range.is_null() {
        return 0;
    }
    c_int::from(decor_range_end_col(range) > col)
}

/// Check if a column is within a decoration range (on same row).
#[no_mangle]
pub extern "C" fn rs_decor_range_contains_col(range: DecorRangeHandle, col: c_int) -> c_int {
    if range.is_null() {
        return 0;
    }
    let start = decor_range_start_col(range);
    let end = decor_range_end_col(range);
    c_int::from(col >= start && col < end)
}

/// Get the decoration range priority.
#[no_mangle]
pub extern "C" fn rs_get_decor_priority(range: DecorRangeHandle) -> u32 {
    if range.is_null() {
        return 0;
    }
    decor_range_priority(range)
}

/// Get the decoration range attribute ID.
#[no_mangle]
pub extern "C" fn rs_get_decor_range_attr(range: DecorRangeHandle) -> c_int {
    if range.is_null() {
        return 0;
    }
    decor_range_attr_id(range)
}

/// Check if decoration range is a highlight type.
#[no_mangle]
pub extern "C" fn rs_decor_range_is_highlight(range: DecorRangeHandle) -> c_int {
    if range.is_null() {
        return 0;
    }
    c_int::from(decor_range_kind(range) == Some(DecorKind::Highlight))
}

/// Check if decoration range is a sign type.
#[no_mangle]
pub extern "C" fn rs_decor_range_is_sign(range: DecorRangeHandle) -> c_int {
    if range.is_null() {
        return 0;
    }
    c_int::from(decor_range_kind(range) == Some(DecorKind::Sign))
}

/// Check if decoration range is a virtual text type.
#[no_mangle]
pub extern "C" fn rs_decor_range_is_virt_text(range: DecorRangeHandle) -> c_int {
    if range.is_null() {
        return 0;
    }
    c_int::from(decor_range_kind(range) == Some(DecorKind::VirtText))
}

/// Check if decoration range is a virtual lines type.
#[no_mangle]
pub extern "C" fn rs_decor_range_is_virt_lines(range: DecorRangeHandle) -> c_int {
    if range.is_null() {
        return 0;
    }
    c_int::from(decor_range_kind(range) == Some(DecorKind::VirtLines))
}

/// Check if decoration range is UI watched.
#[no_mangle]
pub extern "C" fn rs_decor_range_is_ui_watched(range: DecorRangeHandle) -> c_int {
    if range.is_null() {
        return 0;
    }
    c_int::from(decor_range_kind(range) == Some(DecorKind::UIWatched))
}

/// Get virtual text width from a decoration range.
///
/// Returns 0 if the range is not a virtual text type.
#[no_mangle]
pub extern "C" fn rs_get_virt_text_width(range: DecorRangeHandle) -> c_int {
    if range.is_null() {
        return 0;
    }
    let vt = decor_range_virt_text(range);
    if vt.is_null() {
        return 0;
    }
    virt_text_width(vt)
}

/// Get virtual text highlight mode from a decoration range.
///
/// Returns HlMode::Unknown (0) if not a virtual text type.
#[no_mangle]
pub extern "C" fn rs_get_virt_hl_mode(range: DecorRangeHandle) -> c_int {
    if range.is_null() {
        return HlMode::Unknown as c_int;
    }
    let vt = decor_range_virt_text(range);
    if vt.is_null() {
        return HlMode::Unknown as c_int;
    }
    virt_text_hl_mode(vt).map_or(HlMode::Unknown as c_int, |m| m as c_int)
}

/// Get virtual text position from a decoration range.
///
/// Returns VirtTextPos::EndOfLine (0) if not a virtual text type.
#[no_mangle]
pub extern "C" fn rs_get_virt_pos(range: DecorRangeHandle) -> c_int {
    if range.is_null() {
        return VirtTextPos::EndOfLine as c_int;
    }
    let vt = decor_range_virt_text(range);
    if vt.is_null() {
        return VirtTextPos::EndOfLine as c_int;
    }
    virt_text_pos(vt).map_or(VirtTextPos::EndOfLine as c_int, |p| p as c_int)
}

/// Check if virtual text is inline.
#[no_mangle]
pub extern "C" fn rs_virt_is_inline(range: DecorRangeHandle) -> c_int {
    if range.is_null() {
        return 0;
    }
    let vt = decor_range_virt_text(range);
    if vt.is_null() {
        return 0;
    }
    c_int::from(virt_text_pos(vt) == Some(VirtTextPos::Inline))
}

/// Check if virtual text is overlay.
#[no_mangle]
pub extern "C" fn rs_virt_is_overlay(range: DecorRangeHandle) -> c_int {
    if range.is_null() {
        return 0;
    }
    let vt = decor_range_virt_text(range);
    if vt.is_null() {
        return 0;
    }
    c_int::from(virt_text_pos(vt) == Some(VirtTextPos::Overlay))
}

/// Check if virtual text is right-aligned.
#[no_mangle]
pub extern "C" fn rs_virt_is_right_align(range: DecorRangeHandle) -> c_int {
    if range.is_null() {
        return 0;
    }
    let vt = decor_range_virt_text(range);
    if vt.is_null() {
        return 0;
    }
    c_int::from(virt_text_pos(vt) == Some(VirtTextPos::RightAlign))
}

/// Check if virtual text is at end of line.
#[no_mangle]
pub extern "C" fn rs_virt_is_eol(range: DecorRangeHandle) -> c_int {
    if range.is_null() {
        return 0;
    }
    let vt = decor_range_virt_text(range);
    if vt.is_null() {
        return 0;
    }
    let pos = virt_text_pos(vt);
    c_int::from(
        pos == Some(VirtTextPos::EndOfLine) || pos == Some(VirtTextPos::EndOfLineRightAlign),
    )
}

/// Get virtual text flags from a decoration range.
#[no_mangle]
pub extern "C" fn rs_get_virt_flags(range: DecorRangeHandle) -> c_int {
    if range.is_null() {
        return 0;
    }
    let vt = decor_range_virt_text(range);
    if vt.is_null() {
        return 0;
    }
    virt_text_flags(vt)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_virt_text_pos_values() {
        assert_eq!(VirtTextPos::EndOfLine as c_int, 0);
        assert_eq!(VirtTextPos::EndOfLineRightAlign as c_int, 1);
        assert_eq!(VirtTextPos::Inline as c_int, 2);
        assert_eq!(VirtTextPos::Overlay as c_int, 3);
        assert_eq!(VirtTextPos::RightAlign as c_int, 4);
        assert_eq!(VirtTextPos::WinCol as c_int, 5);
    }

    #[test]
    fn test_decor_state_handle_size() {
        assert_eq!(
            std::mem::size_of::<DecorStateHandle>(),
            std::mem::size_of::<*mut c_void>()
        );
    }

    #[test]
    fn test_virt_text_pos_from_c_int() {
        assert_eq!(VirtTextPos::from_c_int(0), Some(VirtTextPos::EndOfLine));
        assert_eq!(
            VirtTextPos::from_c_int(1),
            Some(VirtTextPos::EndOfLineRightAlign)
        );
        assert_eq!(VirtTextPos::from_c_int(2), Some(VirtTextPos::Inline));
        assert_eq!(VirtTextPos::from_c_int(3), Some(VirtTextPos::Overlay));
        assert_eq!(VirtTextPos::from_c_int(4), Some(VirtTextPos::RightAlign));
        assert_eq!(VirtTextPos::from_c_int(5), Some(VirtTextPos::WinCol));
        assert_eq!(VirtTextPos::from_c_int(100), None);
        assert_eq!(VirtTextPos::from_c_int(-1), None);
    }

    #[test]
    fn test_hl_mode_from_c_int() {
        assert_eq!(HlMode::from_c_int(0), Some(HlMode::Unknown));
        assert_eq!(HlMode::from_c_int(1), Some(HlMode::Replace));
        assert_eq!(HlMode::from_c_int(2), Some(HlMode::Combine));
        assert_eq!(HlMode::from_c_int(3), Some(HlMode::Blend));
        assert_eq!(HlMode::from_c_int(100), None);
        assert_eq!(HlMode::from_c_int(-1), None);
    }

    #[test]
    fn test_decor_kind_from_c_int() {
        assert_eq!(DecorKind::from_c_int(0), Some(DecorKind::Highlight));
        assert_eq!(DecorKind::from_c_int(1), Some(DecorKind::Sign));
        assert_eq!(DecorKind::from_c_int(2), Some(DecorKind::VirtText));
        assert_eq!(DecorKind::from_c_int(3), Some(DecorKind::VirtLines));
        assert_eq!(DecorKind::from_c_int(4), Some(DecorKind::UIWatched));
        assert_eq!(DecorKind::from_c_int(100), None);
        assert_eq!(DecorKind::from_c_int(-1), None);
    }

    #[test]
    fn test_draw_col_constants() {
        // Verify DRAW_COL constants have expected values
        assert_eq!(DRAW_COL_UNSET, -1);
        assert_eq!(DRAW_COL_PENDING, -3);
        assert_eq!(DRAW_COL_JUST_ADDED, -10);
        assert_eq!(DRAW_COL_DISABLED, c_int::MIN);
    }

    #[test]
    fn test_kvt_flag_constants() {
        // Verify KVT flag constants have expected bit values
        assert_eq!(KVT_IS_LINES, 1);
        assert_eq!(KVT_HIDE, 2);
        assert_eq!(KVT_LINES_ABOVE, 4);
        assert_eq!(KVT_REPEAT_LINEBREAK, 8);
        // Flags should be distinct powers of 2
        assert_eq!(KVT_IS_LINES & KVT_HIDE, 0);
        assert_eq!(KVT_HIDE & KVT_LINES_ABOVE, 0);
        assert_eq!(KVT_LINES_ABOVE & KVT_REPEAT_LINEBREAK, 0);
    }

    #[test]
    fn test_handle_null_checks() {
        // Test that null handle detection works
        let null_decor = DecorStateHandle(std::ptr::null_mut());
        let null_range = DecorRangeHandle(std::ptr::null_mut());
        let null_virt = DecorVirtTextHandle(std::ptr::null_mut());
        let null_vt = VirtTextHandle(std::ptr::null_mut());
        let null_win = WinHandle(std::ptr::null_mut());

        assert!(null_decor.is_null());
        assert!(null_range.is_null());
        assert!(null_virt.is_null());
        assert!(null_vt.is_null());
        assert!(null_win.is_null());
    }

    #[test]
    fn test_handle_sizes() {
        // All handles should be pointer-sized
        let ptr_size = std::mem::size_of::<*mut c_void>();
        assert_eq!(std::mem::size_of::<DecorStateHandle>(), ptr_size);
        assert_eq!(std::mem::size_of::<DecorRangeHandle>(), ptr_size);
        assert_eq!(std::mem::size_of::<DecorVirtTextHandle>(), ptr_size);
        assert_eq!(std::mem::size_of::<VirtTextHandle>(), ptr_size);
        assert_eq!(std::mem::size_of::<WinHandle>(), ptr_size);
    }

    #[test]
    fn test_kvt_flags_combinable() {
        // KVT flags should be combinable
        let combined = KVT_IS_LINES | KVT_HIDE | KVT_LINES_ABOVE | KVT_REPEAT_LINEBREAK;
        assert_eq!(combined, 0b1111);
        // Individual flags should be extractable
        assert_ne!(combined & KVT_IS_LINES, 0);
        assert_ne!(combined & KVT_HIDE, 0);
        assert_ne!(combined & KVT_LINES_ABOVE, 0);
        assert_ne!(combined & KVT_REPEAT_LINEBREAK, 0);
    }

    #[test]
    fn test_draw_col_ordering() {
        // DRAW_COL constants should have relative ordering for comparisons
        let disabled = DRAW_COL_DISABLED;
        let just_added = DRAW_COL_JUST_ADDED;
        let pending = DRAW_COL_PENDING;
        let unset = DRAW_COL_UNSET;
        assert!(disabled < just_added);
        assert!(just_added < pending);
        assert!(pending < unset);
        assert!(unset < 0);
    }
}
