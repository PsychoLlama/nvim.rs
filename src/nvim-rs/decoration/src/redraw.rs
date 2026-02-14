//! Decoration redraw functions for line rendering
//!
//! This module contains Rust implementations for decoration redraw operations
//! during line rendering, migrated from `src/nvim/decoration.c`.

use std::ffi::{c_int, c_void};

use crate::decor::DECOR_ID_INVALID;
use crate::range::DecorVtHandle;
use crate::{
    DecorKind, DecorRangeHandle, DecorStateHandle, VirtTextPos, WinHandle, DRAW_COL_JUST_ADDED,
    DRAW_COL_UNSET, KVT_IS_LINES, KVT_LINES_ABOVE,
};

/// Opaque handle to buf_T.
#[repr(transparent)]
#[derive(Debug, Clone, Copy)]
pub struct BufHandle(*mut c_void);

// =============================================================================
// External C Functions
// =============================================================================

extern "C" {
    // DecorState accessors
    fn nvim_decor_state_get_row(state: DecorStateHandle) -> c_int;
    fn nvim_decor_state_set_eol_col(state: DecorStateHandle, val: c_int);
    fn nvim_decor_state_get_current_end(state: DecorStateHandle) -> c_int;
    fn nvim_decor_state_get_future_begin(state: DecorStateHandle) -> c_int;
    fn nvim_decor_state_get_ranges_count(state: DecorStateHandle) -> c_int;
    fn nvim_decor_state_get_range_by_idx(state: DecorStateHandle, idx: c_int) -> DecorRangeHandle;
    fn nvim_decor_state_get_range(state: DecorStateHandle, idx: c_int) -> DecorRangeHandle;

    // Phase 1 accessors
    fn nvim_decor_state_win_has_buffer(state: DecorStateHandle, buf: BufHandle) -> c_int;
    fn nvim_decor_state_set_itr_valid(state: DecorStateHandle, val: c_int);
    fn nvim_decor_state_set_win(state: DecorStateHandle, win: WinHandle);
    fn nvim_decor_state_destroy_slots(state: DecorStateHandle);
    fn nvim_decor_state_destroy_ranges_i(state: DecorStateHandle);

    // DecorRange accessors
    fn nvim_decor_range_get_start_row(range: DecorRangeHandle) -> c_int;
    fn nvim_decor_range_get_kind(range: DecorRangeHandle) -> c_int;
    fn nvim_decor_range_get_draw_col(range: DecorRangeHandle) -> c_int;
    fn nvim_decor_range_has_virt_pos(range: DecorRangeHandle) -> bool;
    fn nvim_decor_range_get_virt_pos_kind(range: DecorRangeHandle) -> c_int;

    // DecorRange virt_text accessors
    fn nvim_decor_range_get_virt_text(range: DecorRangeHandle) -> crate::DecorVirtTextHandle;
    fn nvim_decor_virt_text_get_width(vt: crate::DecorVirtTextHandle) -> c_int;

    // Phase 3 accessors
    fn nvim_decor_state_set_row(state: DecorStateHandle, val: c_int);
    fn nvim_decor_state_set_col_until(state: DecorStateHandle, val: c_int);
    fn nvim_decor_state_set_current_end(state: DecorStateHandle, val: c_int);
    fn nvim_decor_state_set_future_begin(state: DecorStateHandle, val: c_int);
    fn nvim_decor_state_set_new_range_ordering(state: DecorStateHandle, val: c_int);
    fn nvim_decor_state_set_free_slot_i(state: DecorStateHandle, val: c_int);
    fn nvim_decor_state_set_slots_size(state: DecorStateHandle, val: c_int);
    fn nvim_decor_state_set_ranges_i_size(state: DecorStateHandle, val: c_int);
    fn nvim_decor_state_ranges_i_memmove(
        state: DecorStateHandle,
        dst_idx: c_int,
        src_idx: c_int,
        count: c_int,
    );
    fn nvim_decor_state_free_owned_ranges(state: DecorStateHandle, beg: c_int, end: c_int);
    fn nvim_buf_get_marktree_n_keys(buf: BufHandle) -> c_int;
    fn nvim_decor_win_get_buffer(wp: WinHandle) -> BufHandle;
    fn nvim_decor_state_itr_current_row(state: DecorStateHandle) -> c_int;
    fn nvim_decor_state_itr_get(state: DecorStateHandle, buf: BufHandle, row: c_int, col: c_int);
    fn nvim_decor_state_get_itr_valid(state: DecorStateHandle) -> c_int;

    // C functions for decor operations
    fn decor_redraw_start(wp: WinHandle, top_row: c_int, state: DecorStateHandle) -> bool;

    // Phase 5: Redraw Dispatch accessors
    fn nvim_redraw_buf_line_later(buf: BufHandle, lnum: c_int, redraw: bool);
    fn nvim_changed_lines_invalidate_buf(
        buf: BufHandle,
        lnum1: c_int,
        col1: c_int,
        lnum2: c_int,
        xtra: c_int,
    );
    fn nvim_redraw_buf_range_later(buf: BufHandle, first: c_int, last: c_int);
    fn nvim_decor_buf_get_line_count(buf: BufHandle) -> c_int;
    fn nvim_decor_vt_ptr_get_pos(vt: DecorVtHandle) -> c_int;
    fn nvim_decor_vt_ptr_get_flags(vt: DecorVtHandle) -> u8;
    fn nvim_decor_vt_ptr_get_next(vt: DecorVtHandle) -> DecorVtHandle;
    fn nvim_decor_redraw_sh_by_idx(buf: BufHandle, row1: c_int, row2: c_int, idx: u32);
    fn nvim_decor_redraw_sh_inline(
        buf: BufHandle,
        row1: c_int,
        row2: c_int,
        hl_flags: u16,
        hl_priority: u16,
        hl_hl_id: c_int,
        hl_conceal_char: u32,
    );
    fn nvim_decor_items_get_next(idx: u32) -> u32;
    fn nvim_buf_put_decor_sh_by_idx(buf: BufHandle, idx: u32, row1: c_int, row2: c_int);
    fn nvim_buf_remove_decor_sh_by_idx(buf: BufHandle, row1: c_int, row2: c_int, idx: u32);
}

// =============================================================================
// Phase 1: Simple State Helpers
// =============================================================================

/// Invalidate the decor_state marktree iterator if the given buffer
/// is the one currently being drawn.
///
/// Rust implementation of `decor_state_invalidate()`.
#[no_mangle]
pub extern "C" fn rs_decor_state_invalidate(state: DecorStateHandle, buf: BufHandle) {
    if state.is_null() {
        return;
    }
    if unsafe { nvim_decor_state_win_has_buffer(state, buf) } != 0 {
        unsafe { nvim_decor_state_set_itr_valid(state, 0) };
    }
}

/// Mark the end of decoration redraw by clearing the window pointer.
///
/// Rust implementation of `decor_redraw_end()`.
#[no_mangle]
pub extern "C" fn rs_decor_redraw_end(state: DecorStateHandle) {
    if state.is_null() {
        return;
    }
    unsafe { nvim_decor_state_set_win(state, WinHandle(std::ptr::null_mut())) };
}

/// Free the memory used by DecorState's kvecs.
///
/// Rust implementation of `decor_state_free()`.
#[no_mangle]
pub extern "C" fn rs_decor_state_free(state: DecorStateHandle) {
    if state.is_null() {
        return;
    }
    unsafe {
        nvim_decor_state_destroy_slots(state);
        nvim_decor_state_destroy_ranges_i(state);
    }
}

// =============================================================================
// Redraw Line State
// =============================================================================

/// State for redrawing a decoration line.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct RedrawLineState {
    /// Current row being processed
    pub row: c_int,
    /// Current column until decorations are valid
    pub col_until: c_int,
    /// EOL column for this line
    pub eol_col: c_int,
    /// Whether any virtual text was encountered
    pub has_virt_text: bool,
    /// Total width of EOL virtual text
    pub eol_virt_width: c_int,
    /// Total width of right-aligned EOL virtual text
    pub eol_right_width: c_int,
}

impl RedrawLineState {
    /// Create new state for a row.
    #[must_use]
    pub const fn new(row: c_int) -> Self {
        Self {
            row,
            col_until: -1,
            eol_col: -1,
            has_virt_text: false,
            eol_virt_width: 0,
            eol_right_width: 0,
        }
    }

    /// Reset for a new row.
    pub fn reset(&mut self, row: c_int) {
        self.row = row;
        self.col_until = -1;
        self.eol_col = -1;
        self.has_virt_text = false;
        self.eol_virt_width = 0;
        self.eol_right_width = 0;
    }
}

/// FFI: Create redraw line state.
#[no_mangle]
pub extern "C" fn rs_redraw_line_state_new(row: c_int) -> RedrawLineState {
    RedrawLineState::new(row)
}

/// FFI: Reset redraw line state.
///
/// # Safety
/// `state` must be valid or null.
#[no_mangle]
pub unsafe extern "C" fn rs_redraw_line_state_reset(state: *mut RedrawLineState, row: c_int) {
    if !state.is_null() {
        (*state).reset(row);
    }
}

// =============================================================================
// Redraw Line Functions
// =============================================================================

// =============================================================================
// Phase 3: DecorState Reset and Line Setup
// =============================================================================

/// Reset decoration redraw state for a new window redraw pass.
///
/// Frees owned ranges, resets all state fields, and returns whether the
/// buffer's marktree has any keys.
///
/// Rust implementation of `decor_redraw_reset()`.
#[no_mangle]
pub extern "C" fn rs_decor_redraw_reset(wp: WinHandle, state: DecorStateHandle) -> bool {
    if state.is_null() || wp.is_null() {
        return false;
    }

    unsafe {
        nvim_decor_state_set_row(state, -1);
        nvim_decor_state_set_win(state, wp);

        let current_end = nvim_decor_state_get_current_end(state);
        let future_begin = nvim_decor_state_get_future_begin(state);
        let ranges_count = nvim_decor_state_get_ranges_count(state);

        // Free owned ranges in [0, current_end) and [future_begin, count)
        nvim_decor_state_free_owned_ranges(state, 0, current_end);
        nvim_decor_state_free_owned_ranges(state, future_begin, ranges_count);

        // Reset kvec sizes and state fields
        nvim_decor_state_set_slots_size(state, 0);
        nvim_decor_state_set_ranges_i_size(state, 0);
        nvim_decor_state_set_free_slot_i(state, -1);
        nvim_decor_state_set_current_end(state, 0);
        nvim_decor_state_set_future_begin(state, 0);
        nvim_decor_state_set_new_range_ordering(state, 0);

        let buf = nvim_decor_win_get_buffer(wp);
        nvim_buf_get_marktree_n_keys(buf) != 0
    }
}

/// Compact the ranges_i array by closing the gap between current_end and future_begin.
///
/// Moves future ranges to start right after current ranges to prevent
/// indefinite forward growth.
///
/// Rust implementation of `decor_state_pack()`.
#[no_mangle]
pub extern "C" fn rs_decor_state_pack(state: DecorStateHandle) {
    if state.is_null() {
        return;
    }

    unsafe {
        let mut count = nvim_decor_state_get_ranges_count(state);
        let cur_end = nvim_decor_state_get_current_end(state);
        let mut fut_beg = nvim_decor_state_get_future_begin(state);

        // Move future ranges to start right after current ranges.
        // Otherwise future ranges will grow forward indefinitely.
        if fut_beg == count {
            count = cur_end;
            fut_beg = cur_end;
        } else if fut_beg != cur_end {
            let move_count = count - fut_beg;
            nvim_decor_state_ranges_i_memmove(state, cur_end, fut_beg, move_count);
            count = cur_end + move_count;
            fut_beg = cur_end;
        }

        nvim_decor_state_set_ranges_i_size(state, count);
        nvim_decor_state_set_future_begin(state, fut_beg);
    }
}

/// Per-line decoration redraw setup.
///
/// Packs the ranges_i array, optionally starts the marktree scan,
/// and sets row/col_until/eol_col.
///
/// Rust implementation of `decor_redraw_line()`.
#[no_mangle]
pub extern "C" fn rs_decor_redraw_line(wp: WinHandle, row: c_int, state: DecorStateHandle) {
    if state.is_null() || wp.is_null() {
        return;
    }

    unsafe {
        rs_decor_state_pack(state);

        let cur_row = nvim_decor_state_get_row(state);
        if cur_row == -1 {
            decor_redraw_start(wp, row, state);
        } else if nvim_decor_state_get_itr_valid(state) == 0 {
            let buf = nvim_decor_win_get_buffer(wp);
            nvim_decor_state_itr_get(state, buf, row, 0);
            nvim_decor_state_set_itr_valid(state, 1);
        }

        nvim_decor_state_set_row(state, row);
        nvim_decor_state_set_col_until(state, -1);
        nvim_decor_state_set_eol_col(state, -1);
    }
}

/// Check if there are (likely) more decorations on the current line.
///
/// Checks active ranges, future ranges, and the marktree iterator position.
///
/// Rust implementation of `decor_has_more_decorations()`.
#[no_mangle]
pub extern "C" fn rs_decor_has_more_decorations(state: DecorStateHandle, row: c_int) -> bool {
    if state.is_null() {
        return false;
    }

    unsafe {
        let current_end = nvim_decor_state_get_current_end(state);
        let future_begin = nvim_decor_state_get_future_begin(state);
        let ranges_count = nvim_decor_state_get_ranges_count(state);

        if current_end != 0 || future_begin != ranges_count {
            return true;
        }

        // Check marktree iterator position
        let itr_row = nvim_decor_state_itr_current_row(state);
        itr_row >= 0 && itr_row <= row
    }
}

// =============================================================================
// Phase 5: Redraw Dispatch and Buffer Operations
// =============================================================================

/// Trigger redraw for a decoration.
///
/// For ext decorations: walks vt linked list (triggering line redraw for each),
/// then walks sh linked list (triggering range redraw).
/// For inline decorations: converts to sh and triggers range redraw.
///
/// Rust implementation of `decor_redraw()`.
#[no_mangle]
pub unsafe extern "C" fn rs_decor_redraw(
    buf: BufHandle,
    row1: c_int,
    row2: c_int,
    col1: c_int,
    ext: bool,
    vt: DecorVtHandle,
    sh_idx: u32,
    hl_flags: u16,
    hl_priority: u16,
    hl_hl_id: c_int,
    hl_conceal_char: u32,
) {
    if ext {
        // Walk virtual text linked list
        let mut cur_vt = vt;
        while !cur_vt.is_null() {
            let flags = nvim_decor_vt_ptr_get_flags(cur_vt);
            let is_lines = flags & KVT_IS_LINES != 0;
            let is_lines_above = flags & KVT_LINES_ABOVE != 0;
            let below = is_lines && !is_lines_above;
            let vt_lnum = row1 + 1 + c_int::from(below);
            nvim_redraw_buf_line_later(buf, vt_lnum, true);

            let pos = nvim_decor_vt_ptr_get_pos(cur_vt);
            if is_lines || pos == VirtTextPos::Inline as c_int {
                let vt_col = if is_lines { 0 } else { col1 };
                nvim_changed_lines_invalidate_buf(buf, vt_lnum, vt_col, vt_lnum + 1, 0);
            }
            cur_vt = nvim_decor_vt_ptr_get_next(cur_vt);
        }

        // Walk sign/highlight linked list
        let mut idx = sh_idx;
        while idx != DECOR_ID_INVALID {
            let next = nvim_decor_items_get_next(idx);
            nvim_decor_redraw_sh_by_idx(buf, row1, row2, idx);
            idx = next;
        }
    } else {
        // Inline highlight
        nvim_decor_redraw_sh_inline(
            buf,
            row1,
            row2,
            hl_flags,
            hl_priority,
            hl_hl_id,
            hl_conceal_char,
        );
    }
}

/// Add decoration tracking to a buffer.
///
/// For ext decorations with signs, walks the sh linked list and calls
/// buf_put_decor_sh for each.
///
/// Rust implementation of `buf_put_decor()`.
#[no_mangle]
pub unsafe extern "C" fn rs_buf_put_decor(
    buf: BufHandle,
    ext: bool,
    _vt: DecorVtHandle,
    sh_idx: u32,
    row: c_int,
    row2: c_int,
) {
    if !ext {
        return;
    }
    let line_count = nvim_decor_buf_get_line_count(buf);
    if row >= line_count {
        return;
    }
    let clamped_row2 = if row2 < line_count - 1 {
        row2
    } else {
        line_count - 1
    };

    let mut idx = sh_idx;
    while idx != DECOR_ID_INVALID {
        let next = nvim_decor_items_get_next(idx);
        nvim_buf_put_decor_sh_by_idx(buf, idx, row, clamped_row2);
        idx = next;
    }
}

/// Remove decoration from a buffer.
///
/// Triggers redraw, then for ext decorations walks sh linked list to remove
/// sign tracking. Optionally frees the decoration.
///
/// Rust implementation of `buf_decor_remove()`.
#[no_mangle]
pub unsafe extern "C" fn rs_buf_decor_remove(
    buf: BufHandle,
    row1: c_int,
    row2: c_int,
    col1: c_int,
    ext: bool,
    vt: DecorVtHandle,
    sh_idx: u32,
    hl_flags: u16,
    hl_priority: u16,
    hl_hl_id: c_int,
    hl_conceal_char: u32,
    do_free: bool,
) {
    // First trigger redraw
    rs_decor_redraw(
        buf,
        row1,
        row2,
        col1,
        ext,
        vt,
        sh_idx,
        hl_flags,
        hl_priority,
        hl_hl_id,
        hl_conceal_char,
    );

    // Remove sign tracking
    if ext {
        let line_count = nvim_decor_buf_get_line_count(buf);
        if row1 < line_count {
            let clamped_row2 = if row2 < line_count - 1 {
                row2
            } else {
                line_count - 1
            };
            let mut idx = sh_idx;
            while idx != DECOR_ID_INVALID {
                let next = nvim_decor_items_get_next(idx);
                nvim_buf_remove_decor_sh_by_idx(buf, row1, clamped_row2, idx);
                idx = next;
            }
        }
    }

    // Optionally free
    if do_free && ext {
        // Call rs_decor_free which is already implemented
        rs_decor_free(1, vt, sh_idx);
    }
}

extern "C" {
    fn rs_decor_free(ext: c_int, vt: DecorVtHandle, sh_idx: u32);
}

/// Calculate EOL virtual text widths for the current row.
///
/// This iterates through active decoration ranges to sum up:
/// - Total EOL virtual text width
/// - Right-aligned EOL virtual text width
///
/// # Safety
/// `total_width` and `right_width` must be valid or null.
#[no_mangle]
pub unsafe extern "C" fn rs_calc_eol_virt_widths(
    state: DecorStateHandle,
    row: c_int,
    total_width: *mut c_int,
    right_width: *mut c_int,
) {
    if state.is_null() {
        return;
    }

    let current_end = unsafe { nvim_decor_state_get_current_end(state) };
    let mut total: c_int = 0;
    let mut right: c_int = 0;

    for i in 0..current_end {
        let range = unsafe { nvim_decor_state_get_range(state, i) };
        if range.is_null() {
            continue;
        }

        let start_row = unsafe { nvim_decor_range_get_start_row(range) };
        if start_row != row {
            continue;
        }

        if !unsafe { nvim_decor_range_has_virt_pos(range) } {
            continue;
        }

        let draw_col = unsafe { nvim_decor_range_get_draw_col(range) };
        if draw_col != DRAW_COL_UNSET {
            continue;
        }

        let pos_kind = unsafe { nvim_decor_range_get_virt_pos_kind(range) };
        let pos = VirtTextPos::from_c_int(pos_kind);

        let kind_raw = unsafe { nvim_decor_range_get_kind(range) };
        let kind = DecorKind::from_c_int(kind_raw);

        if kind != Some(DecorKind::VirtText) {
            continue;
        }

        let vt = unsafe { nvim_decor_range_get_virt_text(range) };
        if vt.is_null() {
            continue;
        }

        let width = unsafe { nvim_decor_virt_text_get_width(vt) };

        match pos {
            Some(VirtTextPos::EndOfLine) => {
                total += width + 1; // +1 for spacing
            }
            Some(VirtTextPos::EndOfLineRightAlign) => {
                total += width + 1;
                right += width + 1;
            }
            _ => {}
        }
    }

    // Remove trailing spacing
    if total > 0 {
        total -= 1;
    }
    if right > 0 {
        right -= 1;
    }

    if !total_width.is_null() {
        *total_width = total;
    }
    if !right_width.is_null() {
        *right_width = right;
    }
}

/// Check if any decoration range on the current row has virtual text position.
#[no_mangle]
pub extern "C" fn rs_decor_row_has_virt_pos(state: DecorStateHandle, row: c_int) -> c_int {
    if state.is_null() {
        return 0;
    }

    let current_end = unsafe { nvim_decor_state_get_current_end(state) };

    for i in 0..current_end {
        let range = unsafe { nvim_decor_state_get_range(state, i) };
        if range.is_null() {
            continue;
        }

        let start_row = unsafe { nvim_decor_range_get_start_row(range) };
        if start_row == row && unsafe { nvim_decor_range_has_virt_pos(range) } {
            return 1;
        }
    }

    0
}

/// Count virtual text items at a specific position type on the current row.
#[no_mangle]
pub extern "C" fn rs_count_virt_at_pos(
    state: DecorStateHandle,
    row: c_int,
    pos_type: c_int,
) -> c_int {
    if state.is_null() {
        return 0;
    }

    let target_pos = VirtTextPos::from_c_int(pos_type);
    let current_end = unsafe { nvim_decor_state_get_current_end(state) };
    let mut count: c_int = 0;

    for i in 0..current_end {
        let range = unsafe { nvim_decor_state_get_range(state, i) };
        if range.is_null() {
            continue;
        }

        let start_row = unsafe { nvim_decor_range_get_start_row(range) };
        if start_row != row {
            continue;
        }

        let kind_raw = unsafe { nvim_decor_range_get_kind(range) };
        if DecorKind::from_c_int(kind_raw) != Some(DecorKind::VirtText) {
            continue;
        }

        let pos_kind = unsafe { nvim_decor_range_get_virt_pos_kind(range) };
        if VirtTextPos::from_c_int(pos_kind) == target_pos {
            count += 1;
        }
    }

    count
}

/// Get the next pending virtual text item on the current row.
///
/// Returns the range index, or -1 if none found.
#[no_mangle]
pub extern "C" fn rs_next_pending_virt_text(
    state: DecorStateHandle,
    row: c_int,
    pos_type: c_int,
) -> c_int {
    if state.is_null() {
        return -1;
    }

    let target_pos = VirtTextPos::from_c_int(pos_type);
    let current_end = unsafe { nvim_decor_state_get_current_end(state) };

    for i in 0..current_end {
        let range = unsafe { nvim_decor_state_get_range(state, i) };
        if range.is_null() {
            continue;
        }

        let start_row = unsafe { nvim_decor_range_get_start_row(range) };
        if start_row != row {
            continue;
        }

        let draw_col = unsafe { nvim_decor_range_get_draw_col(range) };
        if draw_col != DRAW_COL_UNSET && draw_col != DRAW_COL_JUST_ADDED {
            continue;
        }

        let kind_raw = unsafe { nvim_decor_range_get_kind(range) };
        if DecorKind::from_c_int(kind_raw) != Some(DecorKind::VirtText) {
            continue;
        }

        let pos_kind = unsafe { nvim_decor_range_get_virt_pos_kind(range) };
        if VirtTextPos::from_c_int(pos_kind) == target_pos {
            return i;
        }
    }

    -1
}

// =============================================================================
// Range Iteration Helpers
// =============================================================================

/// Iterator state for walking decoration ranges.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct RangeIterator {
    /// Current index in ranges_i array
    pub index: c_int,
    /// End index (current_end for active, ranges_count for all)
    pub end_index: c_int,
    /// Filter by row (-1 = no filter)
    pub filter_row: c_int,
    /// Filter by kind (-1 = no filter)
    pub filter_kind: c_int,
}

impl RangeIterator {
    /// Create iterator for active ranges.
    #[must_use]
    pub const fn active(current_end: c_int) -> Self {
        Self {
            index: 0,
            end_index: current_end,
            filter_row: -1,
            filter_kind: -1,
        }
    }

    /// Create iterator for ranges on a specific row.
    #[must_use]
    pub const fn for_row(current_end: c_int, row: c_int) -> Self {
        Self {
            index: 0,
            end_index: current_end,
            filter_row: row,
            filter_kind: -1,
        }
    }

    /// Create iterator for ranges of a specific kind.
    #[must_use]
    pub const fn for_kind(current_end: c_int, kind: DecorKind) -> Self {
        Self {
            index: 0,
            end_index: current_end,
            filter_row: -1,
            filter_kind: kind as c_int,
        }
    }

    /// Check if iterator has more items.
    #[must_use]
    pub const fn has_next(&self) -> bool {
        self.index < self.end_index
    }

    /// Advance to next item.
    pub fn advance(&mut self) {
        self.index += 1;
    }
}

/// FFI: Create active range iterator.
#[no_mangle]
pub extern "C" fn rs_range_iterator_active(state: DecorStateHandle) -> RangeIterator {
    if state.is_null() {
        return RangeIterator::default();
    }
    let current_end = unsafe { nvim_decor_state_get_current_end(state) };
    RangeIterator::active(current_end)
}

/// FFI: Create row-filtered iterator.
#[no_mangle]
pub extern "C" fn rs_range_iterator_for_row(state: DecorStateHandle, row: c_int) -> RangeIterator {
    if state.is_null() {
        return RangeIterator::default();
    }
    let current_end = unsafe { nvim_decor_state_get_current_end(state) };
    RangeIterator::for_row(current_end, row)
}

/// FFI: Check if iterator has next.
#[no_mangle]
pub extern "C" fn rs_range_iterator_has_next(iter: RangeIterator) -> c_int {
    c_int::from(iter.has_next())
}

/// FFI: Advance iterator.
///
/// # Safety
/// `iter` must be valid or null.
#[no_mangle]
pub unsafe extern "C" fn rs_range_iterator_advance(iter: *mut RangeIterator) {
    if !iter.is_null() {
        (*iter).advance();
    }
}

/// FFI: Get current range from iterator.
#[no_mangle]
pub extern "C" fn rs_range_iterator_current(
    state: DecorStateHandle,
    iter: RangeIterator,
) -> DecorRangeHandle {
    if state.is_null() || !iter.has_next() {
        return DecorRangeHandle(std::ptr::null_mut());
    }
    unsafe { nvim_decor_state_get_range(state, iter.index) }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_redraw_line_state() {
        let state = RedrawLineState::new(10);
        assert_eq!(state.row, 10);
        assert_eq!(state.col_until, -1);
        assert_eq!(state.eol_col, -1);
        assert!(!state.has_virt_text);
    }

    #[test]
    fn test_redraw_line_state_reset() {
        let mut state = RedrawLineState::new(10);
        state.has_virt_text = true;
        state.eol_virt_width = 50;

        state.reset(20);
        assert_eq!(state.row, 20);
        assert!(!state.has_virt_text);
        assert_eq!(state.eol_virt_width, 0);
    }

    #[test]
    fn test_range_iterator() {
        let iter = RangeIterator::active(5);
        assert_eq!(iter.index, 0);
        assert_eq!(iter.end_index, 5);
        assert!(iter.has_next());
        assert_eq!(iter.filter_row, -1);
    }

    #[test]
    fn test_range_iterator_for_row() {
        let iter = RangeIterator::for_row(10, 42);
        assert_eq!(iter.filter_row, 42);
        assert!(iter.has_next());
    }

    #[test]
    fn test_range_iterator_advance() {
        let mut iter = RangeIterator::active(2);
        assert!(iter.has_next());
        iter.advance();
        assert!(iter.has_next());
        iter.advance();
        assert!(!iter.has_next());
    }

    #[test]
    fn test_range_iterator_empty() {
        let iter = RangeIterator::active(0);
        assert!(!iter.has_next());
    }
}
