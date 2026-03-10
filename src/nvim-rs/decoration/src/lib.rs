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
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::not_unsafe_ptr_arg_deref)]
#![allow(dead_code)]

pub mod cache;
pub mod decor;
pub mod invoke;
pub mod provider;
pub mod range;
pub mod redraw;
pub mod types;

use std::ffi::{c_char, c_int, c_void};

pub use types::{DecorRange, DecorSignHighlight, DecorState, DecorVirtText, KVec};

/// schar_T is stored as a u32.
pub type ScharT = u32;

// ============================================================================
// Type aliases replacing opaque handles
// ============================================================================

/// Typed pointer to DecorState (replaces DecorStateHandle).
pub type DecorStateHandle = *mut DecorState;

/// Typed pointer to DecorRange (replaces DecorRangeHandle).
pub type DecorRangeHandle = *mut DecorRange;

/// Typed pointer to DecorVirtText (replaces DecorVirtTextHandle).
pub type DecorVirtTextHandle = *mut DecorVirtText;

/// Opaque handle to VirtText (kvec_t(VirtTextChunk)) -- kept opaque
/// since we still use C for next_virt_text_chunk iteration.
#[repr(transparent)]
#[derive(Debug, Clone, Copy)]
pub struct VirtTextHandle(*mut c_void);

impl VirtTextHandle {
    pub fn is_null(self) -> bool {
        self.0.is_null()
    }
}

/// Opaque handle to window (win_T).
#[repr(transparent)]
#[derive(Debug, Clone, Copy)]
pub struct WinHandle(*mut c_void);

impl WinHandle {
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
// C functions we still need (non-accessor)
// ============================================================================

extern "C" {
    /// Get the global decor_state pointer.
    fn nvim_get_decor_state() -> DecorStateHandle;

    // VirtText iteration -- kept in C (complex kvec logic)
    fn nvim_next_virt_text_chunk(
        vt: VirtTextHandle,
        pos: *mut usize,
        attr: *mut c_int,
    ) -> *const c_char;

    // win_extmark_arr push
    fn nvim_win_extmark_push(ns_id: u64, mark_id: u64, win_row: c_int, win_col: c_int);
}

// ============================================================================
// Direct field access helpers for DecorState
// ============================================================================

/// Get the global decor_state pointer.
pub fn get_decor_state() -> DecorStateHandle {
    unsafe { nvim_get_decor_state() }
}

/// Get the current row from decor_state.
///
/// # Safety
/// `state` must be a valid non-null pointer.
pub unsafe fn decor_state_row(state: DecorStateHandle) -> c_int {
    (*state).row
}

/// Get the EOL column from decor_state.
///
/// # Safety
/// `state` must be a valid non-null pointer.
pub unsafe fn decor_state_eol_col(state: DecorStateHandle) -> c_int {
    (*state).eol_col
}

/// Set the EOL column in decor_state.
///
/// # Safety
/// `state` must be a valid non-null pointer.
pub unsafe fn decor_state_set_eol_col(state: DecorStateHandle, val: c_int) {
    (*state).eol_col = val;
}

/// Get the current_end from decor_state (number of active ranges).
///
/// # Safety
/// `state` must be a valid non-null pointer.
pub unsafe fn decor_state_current_end(state: DecorStateHandle) -> c_int {
    (*state).current_end
}

/// Get the current attr from decor_state.
///
/// # Safety
/// `state` must be a valid non-null pointer.
pub unsafe fn decor_state_current(state: DecorStateHandle) -> c_int {
    (*state).current
}

/// Get the col_until from decor_state.
///
/// # Safety
/// `state` must be a valid non-null pointer.
pub unsafe fn decor_state_col_until(state: DecorStateHandle) -> c_int {
    (*state).col_until
}

/// Get the conceal from decor_state.
///
/// # Safety
/// `state` must be a valid non-null pointer.
pub unsafe fn decor_state_conceal(state: DecorStateHandle) -> c_int {
    (*state).conceal
}

/// Get the conceal_char from decor_state.
///
/// # Safety
/// `state` must be a valid non-null pointer.
pub unsafe fn decor_state_conceal_char(state: DecorStateHandle) -> ScharT {
    (*state).conceal_char
}

/// Get the conceal_attr from decor_state.
///
/// # Safety
/// `state` must be a valid non-null pointer.
pub unsafe fn decor_state_conceal_attr(state: DecorStateHandle) -> c_int {
    (*state).conceal_attr
}

/// Get the spell from decor_state.
///
/// # Safety
/// `state` must be a valid non-null pointer.
pub unsafe fn decor_state_spell(state: DecorStateHandle) -> c_int {
    (*state).spell
}

/// Get the window from decor_state (as raw pointer).
///
/// # Safety
/// `state` must be a valid non-null pointer.
pub unsafe fn decor_state_win(state: DecorStateHandle) -> WinHandle {
    WinHandle((*state).win)
}

/// Get the top_row from decor_state.
///
/// # Safety
/// `state` must be a valid non-null pointer.
pub unsafe fn decor_state_top_row(state: DecorStateHandle) -> c_int {
    (*state).top_row
}

/// Get a DecorRange by active-range index (0..current_end).
///
/// Returns null if `idx` is out of bounds.
///
/// # Safety
/// `state` must be a valid non-null pointer.
pub unsafe fn decor_state_get_range(state: DecorStateHandle, idx: c_int) -> DecorRangeHandle {
    let state = &*state;
    if idx < 0 || idx >= state.current_end {
        return std::ptr::null_mut();
    }
    let slot_idx = *state.ranges_i.get_unchecked(idx as usize);
    // DecorRangeSlot is a union; the range field is at offset 0.
    // Cast the slot pointer directly to *mut DecorRange.
    state
        .slots
        .get_unchecked(slot_idx as usize)
        .cast::<types::DecorRange>()
}

/// Get a DecorRange by ranges_i array index (0..ranges_i.size).
///
/// Returns null if `idx` is out of bounds.
///
/// # Safety
/// `state` must be a valid non-null pointer.
pub unsafe fn decor_state_get_range_by_idx(
    state: DecorStateHandle,
    idx: c_int,
) -> DecorRangeHandle {
    let state = &*state;
    if idx < 0 || idx >= state.ranges_i.size as c_int {
        return std::ptr::null_mut();
    }
    let slot_idx = *state.ranges_i.get_unchecked(idx as usize);
    state
        .slots
        .get_unchecked(slot_idx as usize)
        .cast::<types::DecorRange>()
}

// ============================================================================
// Direct field access helpers for DecorRange
// ============================================================================

/// Get the start_row from a DecorRange.
///
/// # Safety
/// `range` must be a valid non-null pointer.
pub unsafe fn decor_range_start_row(range: DecorRangeHandle) -> c_int {
    (*range).start_row
}

/// Get the start_col from a DecorRange.
///
/// # Safety
/// `range` must be a valid non-null pointer.
pub unsafe fn decor_range_start_col(range: DecorRangeHandle) -> c_int {
    (*range).start_col
}

/// Get the end_row from a DecorRange.
///
/// # Safety
/// `range` must be a valid non-null pointer.
pub unsafe fn decor_range_end_row(range: DecorRangeHandle) -> c_int {
    (*range).end_row
}

/// Get the end_col from a DecorRange.
///
/// # Safety
/// `range` must be a valid non-null pointer.
pub unsafe fn decor_range_end_col(range: DecorRangeHandle) -> c_int {
    (*range).end_col
}

/// Get the draw_col from a DecorRange.
///
/// # Safety
/// `range` must be a valid non-null pointer.
pub unsafe fn decor_range_draw_col(range: DecorRangeHandle) -> c_int {
    (*range).draw_col
}

/// Set the draw_col in a DecorRange.
///
/// # Safety
/// `range` must be a valid non-null pointer.
pub unsafe fn decor_range_set_draw_col(range: DecorRangeHandle, val: c_int) {
    (*range).draw_col = val;
}

/// Get the kind from a DecorRange.
///
/// # Safety
/// `range` must be a valid non-null pointer.
pub unsafe fn decor_range_kind(range: DecorRangeHandle) -> Option<DecorKind> {
    DecorKind::from_c_int(c_int::from((*range).kind))
}

/// Get the attr_id from a DecorRange.
///
/// # Safety
/// `range` must be a valid non-null pointer.
pub unsafe fn decor_range_attr_id(range: DecorRangeHandle) -> c_int {
    (*range).attr_id
}

/// Get the priority from a DecorRange.
///
/// # Safety
/// `range` must be a valid non-null pointer.
pub unsafe fn decor_range_priority(range: DecorRangeHandle) -> u32 {
    (*range).priority_internal
}

/// Check if a DecorRange has virtual text position (VirtText or UIWatched kind).
///
/// # Safety
/// `range` must be a valid non-null pointer.
pub unsafe fn decor_range_has_virt_pos(range: DecorRangeHandle) -> bool {
    let kind = (*range).kind;
    kind == DecorKind::VirtText as u8 || kind == DecorKind::UIWatched as u8
}

/// Get the virtual text position kind from a DecorRange.
///
/// Returns VirtTextPos::EndOfLine if not a virtual text / UI watched range.
///
/// # Safety
/// `range` must be a valid non-null pointer.
pub unsafe fn decor_range_virt_pos_kind(range: DecorRangeHandle) -> Option<VirtTextPos> {
    let r = &*range;
    if r.kind == DecorKind::VirtText as u8 {
        let vt = r.data.vt;
        if !vt.is_null() {
            return VirtTextPos::from_c_int((*vt).pos);
        }
    } else if r.kind == DecorKind::UIWatched as u8 {
        return VirtTextPos::from_c_int(r.data.ui.pos);
    }
    Some(VirtTextPos::EndOfLine)
}

/// Get the DecorVirtText pointer from a DecorRange (for VirtText kind).
///
/// Returns null if range is not VirtText kind.
///
/// # Safety
/// `range` must be a valid non-null pointer.
pub unsafe fn decor_range_virt_text(range: DecorRangeHandle) -> DecorVirtTextHandle {
    let r = &*range;
    if r.kind == DecorKind::VirtText as u8 {
        r.data.vt
    } else {
        std::ptr::null_mut()
    }
}

/// Get the ns_id from a UIWatched DecorRange.
///
/// # Safety
/// `range` must be a valid non-null pointer.
pub unsafe fn decor_range_ui_ns_id(range: DecorRangeHandle) -> u64 {
    let r = &*range;
    if r.kind == DecorKind::UIWatched as u8 {
        u64::from(r.data.ui.ns_id)
    } else {
        0
    }
}

/// Get the mark_id from a UIWatched DecorRange.
///
/// # Safety
/// `range` must be a valid non-null pointer.
pub unsafe fn decor_range_ui_mark_id(range: DecorRangeHandle) -> u32 {
    let r = &*range;
    if r.kind == DecorKind::UIWatched as u8 {
        r.data.ui.mark_id
    } else {
        0
    }
}

// ============================================================================
// Direct field access helpers for DecorVirtText
// ============================================================================

/// Get the hl_mode from a DecorVirtText.
///
/// # Safety
/// `vt` must be a valid non-null pointer.
pub unsafe fn virt_text_hl_mode(vt: DecorVirtTextHandle) -> Option<HlMode> {
    HlMode::from_c_int(c_int::from((*vt).hl_mode))
}

/// Get the pos from a DecorVirtText.
///
/// # Safety
/// `vt` must be a valid non-null pointer.
pub unsafe fn virt_text_pos(vt: DecorVirtTextHandle) -> Option<VirtTextPos> {
    VirtTextPos::from_c_int((*vt).pos)
}

/// Get the width from a DecorVirtText.
///
/// # Safety
/// `vt` must be a valid non-null pointer.
pub unsafe fn virt_text_width(vt: DecorVirtTextHandle) -> c_int {
    (*vt).width
}

/// Get the col from a DecorVirtText.
///
/// # Safety
/// `vt` must be a valid non-null pointer.
pub unsafe fn virt_text_col(vt: DecorVirtTextHandle) -> c_int {
    (*vt).col
}

/// Get the flags from a DecorVirtText.
///
/// # Safety
/// `vt` must be a valid non-null pointer.
pub unsafe fn virt_text_flags(vt: DecorVirtTextHandle) -> c_int {
    c_int::from((*vt).flags)
}

/// Get the number of chunks in a VirtText inside DecorVirtText.
///
/// # Safety
/// `vt` must be a valid non-null pointer.
pub unsafe fn virt_text_chunk_count(vt: DecorVirtTextHandle) -> usize {
    std::ptr::addr_of!((*vt).data.virt_text).read().size
}

/// Get a chunk text from a VirtText inside DecorVirtText by index.
///
/// # Safety
/// `vt` must be a valid non-null pointer.
pub unsafe fn virt_text_chunk_text(vt: DecorVirtTextHandle, idx: usize) -> Option<*const c_char> {
    let vt_ref = &*vt;
    let kvec = &vt_ref.data.virt_text;
    if idx >= kvec.size {
        return None;
    }
    let chunk = kvec.get_unchecked(idx);
    let text = (*chunk).text.cast_const();
    if text.is_null() {
        None
    } else {
        Some(text)
    }
}

/// Get a chunk hl_id from a VirtText inside DecorVirtText by index.
///
/// # Safety
/// `vt` must be a valid non-null pointer.
pub unsafe fn virt_text_chunk_hl_id(vt: DecorVirtTextHandle, idx: usize) -> c_int {
    let vt_ref = &*vt;
    let kvec = &vt_ref.data.virt_text;
    if idx >= kvec.size {
        return 0;
    }
    let chunk = kvec.get_unchecked(idx);
    (*chunk).hl_id
}

/// Get a VirtTextHandle pointing to the virt_text data inside DecorVirtText.
/// (Still used for nvim_next_virt_text_chunk C calls.)
///
/// # Safety
/// `vt` must be a valid non-null pointer.
pub unsafe fn virt_text_get_virt_text(vt: DecorVirtTextHandle) -> VirtTextHandle {
    let vt_ref = &*vt;
    VirtTextHandle(
        std::ptr::addr_of!(vt_ref.data.virt_text)
            .cast_mut()
            .cast::<c_void>(),
    )
}

// ============================================================================
// VirtText iteration wrapper functions
// ============================================================================

/// Iterator for VirtText chunks via C helper.
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
// win_extmark_arr wrapper functions
// ============================================================================

/// Push a WinExtmark to the global win_extmark_arr.
pub fn win_extmark_push(ns_id: u64, mark_id: u64, win_row: c_int, win_col: c_int) {
    unsafe { nvim_win_extmark_push(ns_id, mark_id, win_row, win_col) }
}

// ============================================================================
// High-level iteration helpers for draw_virt_text
// (Implemented in Rust using direct struct access -- no C accessor needed)
// ============================================================================

/// Get an active DecorRange by ranges_i array index (unrestricted, not just current_end).
///
/// # Safety
/// `state` must be a valid non-null pointer.
pub unsafe fn decor_state_get_active_range(state: DecorStateHandle, i: c_int) -> DecorRangeHandle {
    let state_ref = &*state;
    if i < 0 || i >= state_ref.ranges_i.size as c_int {
        return std::ptr::null_mut();
    }
    let slot_idx = *state_ref.ranges_i.get_unchecked(i as usize);
    state_ref
        .slots
        .get_unchecked(slot_idx as usize)
        .cast::<types::DecorRange>()
}

/// Get the total width of EOL right-aligned virtual text from index i onwards.
///
/// # Safety
/// `state` must be a valid non-null pointer.
pub unsafe fn decor_state_get_eol_right_width(state: DecorStateHandle, from_idx: c_int) -> c_int {
    let state_ref = &*state;
    let mut total_width: c_int = 0;
    let count = state_ref.ranges_i.size as c_int;

    for j in from_idx..state_ref.current_end.min(count) {
        let slot_idx = *state_ref.ranges_i.get_unchecked(j as usize);
        let r_ptr = state_ref
            .slots
            .get_unchecked(slot_idx as usize)
            .cast::<types::DecorRange>();
        let r = &*r_ptr;

        if r.start_row != state_ref.row {
            continue;
        }
        if !decor_range_has_virt_pos(r_ptr) {
            continue;
        }
        if r.draw_col != DRAW_COL_UNSET {
            continue;
        }
        // Check if EOL right aligned
        let pos_kind = decor_range_virt_pos_kind(r_ptr);
        if pos_kind != Some(VirtTextPos::EndOfLineRightAlign) {
            continue;
        }
        if r.kind == DecorKind::VirtText as u8 {
            let vt = r.data.vt;
            if !vt.is_null() {
                total_width += (*vt).width + 1;
            }
        }
    }

    if total_width > 0 {
        total_width -= 1;
    }

    total_width
}

// ============================================================================
// FFI exports
// ============================================================================

/// Check if a DecorRange has a virtual position (virtual text or ui_watched).
/// Rust implementation of decor_virt_pos().
#[no_mangle]
pub unsafe extern "C" fn rs_decor_virt_pos(range: DecorRangeHandle) -> c_int {
    if range.is_null() {
        return 0;
    }
    c_int::from(decor_range_has_virt_pos(range))
}

/// Get the virtual text position kind from a DecorRange.
/// Rust implementation of decor_virt_pos_kind().
#[no_mangle]
pub unsafe extern "C" fn rs_decor_virt_pos_kind(range: DecorRangeHandle) -> c_int {
    if range.is_null() {
        return VirtTextPos::EndOfLine as c_int;
    }
    decor_range_virt_pos_kind(range).map_or(VirtTextPos::EndOfLine as c_int, |p| p as c_int)
}

// ============================================================================
// Phase 3: Conceal and Decoration Attribute Helpers
// ============================================================================

/// Check if concealment is active in the current decoration state.
#[no_mangle]
pub unsafe extern "C" fn rs_conceal_check(state: DecorStateHandle) -> c_int {
    if state.is_null() {
        return 0;
    }
    c_int::from(decor_state_conceal(state) > 0)
}

/// Check if concealment should show a replacement character.
#[no_mangle]
pub unsafe extern "C" fn rs_conceal_shows_char(state: DecorStateHandle) -> c_int {
    if state.is_null() {
        return 0;
    }
    let conceal = decor_state_conceal(state);
    c_int::from(conceal == 1 || conceal == 2)
}

/// Check if concealment is full (completely hides text).
#[no_mangle]
pub unsafe extern "C" fn rs_conceal_is_full(state: DecorStateHandle) -> c_int {
    if state.is_null() {
        return 0;
    }
    c_int::from(decor_state_conceal(state) >= 3)
}

/// Get the conceal character if one is set.
#[no_mangle]
pub unsafe extern "C" fn rs_get_conceal_char(state: DecorStateHandle) -> ScharT {
    if state.is_null() {
        return 0;
    }
    decor_state_conceal_char(state)
}

/// Get the conceal attribute if one is set.
#[no_mangle]
pub unsafe extern "C" fn rs_get_conceal_attr(state: DecorStateHandle) -> c_int {
    if state.is_null() {
        return 0;
    }
    decor_state_conceal_attr(state)
}

/// Check if decoration has a custom conceal character.
#[no_mangle]
pub unsafe extern "C" fn rs_has_conceal_char(state: DecorStateHandle) -> c_int {
    if state.is_null() {
        return 0;
    }
    c_int::from(decor_state_conceal_char(state) != 0)
}

/// Check if decoration has a custom conceal attribute.
#[no_mangle]
pub unsafe extern "C" fn rs_has_conceal_attr(state: DecorStateHandle) -> c_int {
    if state.is_null() {
        return 0;
    }
    c_int::from(decor_state_conceal_attr(state) != 0)
}

/// Get the spell state from decoration.
#[no_mangle]
pub unsafe extern "C" fn rs_get_decor_spell(state: DecorStateHandle) -> c_int {
    if state.is_null() {
        return -1;
    }
    decor_state_spell(state)
}

/// Check if decoration forces spell checking on.
#[no_mangle]
pub unsafe extern "C" fn rs_decor_spell_on(state: DecorStateHandle) -> c_int {
    if state.is_null() {
        return 0;
    }
    c_int::from(decor_state_spell(state) == 1)
}

/// Check if decoration forces spell checking off.
#[no_mangle]
pub unsafe extern "C" fn rs_decor_spell_off(state: DecorStateHandle) -> c_int {
    if state.is_null() {
        return 0;
    }
    c_int::from(decor_state_spell(state) == 0)
}

/// Get decoration attributes for a specific column.
#[no_mangle]
pub unsafe extern "C" fn rs_get_decor_col_until(state: DecorStateHandle) -> c_int {
    if state.is_null() {
        return 0;
    }
    decor_state_col_until(state)
}

/// Check if we're past the decoration column extent.
#[no_mangle]
pub unsafe extern "C" fn rs_decor_needs_refresh(state: DecorStateHandle, col: c_int) -> c_int {
    if state.is_null() {
        return 1;
    }
    c_int::from(col >= decor_state_col_until(state))
}

/// Get the current decoration row.
#[no_mangle]
pub unsafe extern "C" fn rs_get_decor_row(state: DecorStateHandle) -> c_int {
    if state.is_null() {
        return -1;
    }
    decor_state_row(state)
}

/// Check if decoration state is on a specific row.
#[no_mangle]
pub unsafe extern "C" fn rs_decor_on_row(state: DecorStateHandle, row: c_int) -> c_int {
    if state.is_null() {
        return 0;
    }
    c_int::from(decor_state_row(state) == row)
}

/// Get the number of active decoration ranges.
#[no_mangle]
pub unsafe extern "C" fn rs_get_active_decor_count(state: DecorStateHandle) -> c_int {
    if state.is_null() {
        return 0;
    }
    decor_state_current_end(state)
}

/// Check if there are any active decorations.
#[no_mangle]
pub unsafe extern "C" fn rs_has_active_decor(state: DecorStateHandle) -> c_int {
    if state.is_null() {
        return 0;
    }
    c_int::from(decor_state_current_end(state) > 0)
}

/// Get the current decoration attribute ID.
#[no_mangle]
pub unsafe extern "C" fn rs_get_decor_attr(state: DecorStateHandle) -> c_int {
    if state.is_null() {
        return 0;
    }
    decor_state_current(state)
}

/// Check if decoration range is for the current row.
#[no_mangle]
pub unsafe extern "C" fn rs_decor_range_on_row(range: DecorRangeHandle, row: c_int) -> c_int {
    if range.is_null() {
        return 0;
    }
    c_int::from(decor_range_start_row(range) == row)
}

/// Check if decoration range starts at or before a column.
#[no_mangle]
pub unsafe extern "C" fn rs_decor_range_starts_by(range: DecorRangeHandle, col: c_int) -> c_int {
    if range.is_null() {
        return 0;
    }
    c_int::from(decor_range_start_col(range) <= col)
}

/// Check if decoration range ends after a column.
#[no_mangle]
pub unsafe extern "C" fn rs_decor_range_ends_after(range: DecorRangeHandle, col: c_int) -> c_int {
    if range.is_null() {
        return 0;
    }
    c_int::from(decor_range_end_col(range) > col)
}

/// Check if a column is within a decoration range (on same row).
#[no_mangle]
pub unsafe extern "C" fn rs_decor_range_contains_col(range: DecorRangeHandle, col: c_int) -> c_int {
    if range.is_null() {
        return 0;
    }
    let start = decor_range_start_col(range);
    let end = decor_range_end_col(range);
    c_int::from(col >= start && col < end)
}

/// Get the decoration range priority.
#[no_mangle]
pub unsafe extern "C" fn rs_get_decor_priority(range: DecorRangeHandle) -> u32 {
    if range.is_null() {
        return 0;
    }
    decor_range_priority(range)
}

/// Get the decoration range attribute ID.
#[no_mangle]
pub unsafe extern "C" fn rs_get_decor_range_attr(range: DecorRangeHandle) -> c_int {
    if range.is_null() {
        return 0;
    }
    decor_range_attr_id(range)
}

/// Check if decoration range is a highlight type.
#[no_mangle]
pub unsafe extern "C" fn rs_decor_range_is_highlight(range: DecorRangeHandle) -> c_int {
    if range.is_null() {
        return 0;
    }
    c_int::from(decor_range_kind(range) == Some(DecorKind::Highlight))
}

/// Check if decoration range is a sign type.
#[no_mangle]
pub unsafe extern "C" fn rs_decor_range_is_sign(range: DecorRangeHandle) -> c_int {
    if range.is_null() {
        return 0;
    }
    c_int::from(decor_range_kind(range) == Some(DecorKind::Sign))
}

/// Check if decoration range is a virtual text type.
#[no_mangle]
pub unsafe extern "C" fn rs_decor_range_is_virt_text(range: DecorRangeHandle) -> c_int {
    if range.is_null() {
        return 0;
    }
    c_int::from(decor_range_kind(range) == Some(DecorKind::VirtText))
}

/// Check if decoration range is a virtual lines type.
#[no_mangle]
pub unsafe extern "C" fn rs_decor_range_is_virt_lines(range: DecorRangeHandle) -> c_int {
    if range.is_null() {
        return 0;
    }
    c_int::from(decor_range_kind(range) == Some(DecorKind::VirtLines))
}

/// Check if decoration range is UI watched.
#[no_mangle]
pub unsafe extern "C" fn rs_decor_range_is_ui_watched(range: DecorRangeHandle) -> c_int {
    if range.is_null() {
        return 0;
    }
    c_int::from(decor_range_kind(range) == Some(DecorKind::UIWatched))
}

/// Get virtual text width from a decoration range.
#[no_mangle]
pub unsafe extern "C" fn rs_get_virt_text_width(range: DecorRangeHandle) -> c_int {
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
#[no_mangle]
pub unsafe extern "C" fn rs_get_virt_hl_mode(range: DecorRangeHandle) -> c_int {
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
#[no_mangle]
pub unsafe extern "C" fn rs_get_virt_pos(range: DecorRangeHandle) -> c_int {
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
pub unsafe extern "C" fn rs_virt_is_inline(range: DecorRangeHandle) -> c_int {
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
pub unsafe extern "C" fn rs_virt_is_overlay(range: DecorRangeHandle) -> c_int {
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
pub unsafe extern "C" fn rs_virt_is_right_align(range: DecorRangeHandle) -> c_int {
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
pub unsafe extern "C" fn rs_virt_is_eol(range: DecorRangeHandle) -> c_int {
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
pub unsafe extern "C" fn rs_get_virt_flags(range: DecorRangeHandle) -> c_int {
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
        assert_eq!(DRAW_COL_UNSET, -1);
        assert_eq!(DRAW_COL_PENDING, -3);
        assert_eq!(DRAW_COL_JUST_ADDED, -10);
        assert_eq!(DRAW_COL_DISABLED, c_int::MIN);
    }

    #[test]
    fn test_kvt_flag_constants() {
        assert_eq!(KVT_IS_LINES, 1);
        assert_eq!(KVT_HIDE, 2);
        assert_eq!(KVT_LINES_ABOVE, 4);
        assert_eq!(KVT_REPEAT_LINEBREAK, 8);
        assert_eq!(KVT_IS_LINES & KVT_HIDE, 0);
        assert_eq!(KVT_HIDE & KVT_LINES_ABOVE, 0);
        assert_eq!(KVT_LINES_ABOVE & KVT_REPEAT_LINEBREAK, 0);
    }

    #[test]
    fn test_handle_null_checks() {
        let null_state: DecorStateHandle = std::ptr::null_mut();
        let null_range: DecorRangeHandle = std::ptr::null_mut();
        let null_virt: DecorVirtTextHandle = std::ptr::null_mut();
        let null_vt = VirtTextHandle(std::ptr::null_mut());
        let null_win = WinHandle(std::ptr::null_mut());

        assert!(null_state.is_null());
        assert!(null_range.is_null());
        assert!(null_virt.is_null());
        assert!(null_vt.is_null());
        assert!(null_win.is_null());
    }

    #[test]
    fn test_handle_sizes() {
        let ptr_size = std::mem::size_of::<*mut c_void>();
        assert_eq!(std::mem::size_of::<DecorStateHandle>(), ptr_size);
        assert_eq!(std::mem::size_of::<DecorRangeHandle>(), ptr_size);
        assert_eq!(std::mem::size_of::<DecorVirtTextHandle>(), ptr_size);
        assert_eq!(std::mem::size_of::<VirtTextHandle>(), ptr_size);
        assert_eq!(std::mem::size_of::<WinHandle>(), ptr_size);
    }

    #[test]
    fn test_kvt_flags_combinable() {
        let combined = KVT_IS_LINES | KVT_HIDE | KVT_LINES_ABOVE | KVT_REPEAT_LINEBREAK;
        assert_eq!(combined, 0b1111);
        assert_ne!(combined & KVT_IS_LINES, 0);
        assert_ne!(combined & KVT_HIDE, 0);
        assert_ne!(combined & KVT_LINES_ABOVE, 0);
        assert_ne!(combined & KVT_REPEAT_LINEBREAK, 0);
    }

    #[test]
    fn test_draw_col_ordering() {
        let disabled = DRAW_COL_DISABLED;
        let just_added = DRAW_COL_JUST_ADDED;
        let pending = DRAW_COL_PENDING;
        let unset = DRAW_COL_UNSET;
        // Disabled is most negative
        assert!(disabled < just_added);
        assert!(just_added < pending);
        assert!(pending < unset);
        assert!(unset < 0);
    }
}
