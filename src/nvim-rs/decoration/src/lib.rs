//! Decoration and virtual text system for Neovim
//!
//! This crate provides Rust implementations for decoration handling
//! from `src/nvim/decoration.c`, focusing on virtual text positioning.

#![allow(unsafe_code)]
#![allow(clippy::doc_markdown)]
#![allow(clippy::similar_names)]
#![allow(clippy::too_many_arguments)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::missing_safety_doc)]
#![allow(clippy::must_use_candidate)]
#![allow(clippy::match_same_arms)]
#![allow(dead_code)]

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

// ============================================================================
// FFI exports
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_virt_text_pos_values() {
        assert_eq!(VirtTextPos::EndOfLine as c_int, 0);
        assert_eq!(VirtTextPos::EndOfLineRightAlign as c_int, 1);
        assert_eq!(VirtTextPos::Inline as c_int, 2);
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
        assert_eq!(VirtTextPos::from_c_int(100), None);
    }

    #[test]
    fn test_hl_mode_from_c_int() {
        assert_eq!(HlMode::from_c_int(0), Some(HlMode::Unknown));
        assert_eq!(HlMode::from_c_int(1), Some(HlMode::Replace));
        assert_eq!(HlMode::from_c_int(2), Some(HlMode::Combine));
        assert_eq!(HlMode::from_c_int(3), Some(HlMode::Blend));
        assert_eq!(HlMode::from_c_int(100), None);
    }

    #[test]
    fn test_decor_kind_from_c_int() {
        assert_eq!(DecorKind::from_c_int(0), Some(DecorKind::Highlight));
        assert_eq!(DecorKind::from_c_int(1), Some(DecorKind::Sign));
        assert_eq!(DecorKind::from_c_int(2), Some(DecorKind::VirtText));
        assert_eq!(DecorKind::from_c_int(100), None);
    }
}
