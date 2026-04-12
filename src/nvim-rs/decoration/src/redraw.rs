//! Decoration redraw functions for line rendering
//!
//! This module contains Rust implementations for decoration redraw operations
//! during line rendering, migrated from `src/nvim/decoration.c`.

use std::ffi::{c_char, c_int, c_void};

use crate::decor::{range_end_before, DECOR_ID_INVALID, KSH_CONCEAL, KSH_SPELL_OFF, KSH_SPELL_ON};
use crate::range::DecorVtHandle;
use crate::types::{
    DecorRange, DecorRangeSlot, DecorSignHighlight, DecorVirtText, KVec, MTKey, MTPair, VirtLines,
    VirtText,
};
use crate::{
    DecorKind, DecorRangeHandle, DecorStateHandle, ScharT, VirtTextPos, WinHandle,
    DRAW_COL_JUST_ADDED, DRAW_COL_UNSET, KVT_IS_LINES, KVT_LINES_ABOVE,
};

/// Opaque handle to buf_T.
#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BufHandle(pub *mut c_void);

// =============================================================================
// External C Functions
// =============================================================================

extern "C" {
    // Complex C functions that remain in C (marktree, kvec operations)
    fn nvim_decor_state_win_has_buffer(state: DecorStateHandle, buf: BufHandle) -> c_int;
    fn nvim_decor_state_destroy_slots(state: DecorStateHandle);
    fn nvim_decor_state_destroy_ranges_i(state: DecorStateHandle);
    fn nvim_buf_get_marktree_n_keys(buf: BufHandle) -> c_int;
    fn nvim_decor_win_get_buffer(wp: WinHandle) -> BufHandle;
    fn nvim_decor_state_itr_current_row(state: DecorStateHandle) -> c_int;
    fn nvim_decor_state_itr_get(state: DecorStateHandle, buf: BufHandle, row: c_int, col: c_int);

    // Marktree operations (Rust functions called via C linkage from this Rust crate)
    #[link_name = "rs_marktree_itr_get_overlap"]
    fn rs_marktree_itr_get_overlap(b: *mut c_void, row: i32, col: i32, itr: *mut c_void) -> bool;
    #[link_name = "rs_marktree_itr_step_overlap"]
    fn rs_marktree_itr_step_overlap(b: *mut c_void, itr: *mut c_void, pair: *mut MTPair) -> bool;
    #[link_name = "rs_marktree_itr_get"]
    fn rs_marktree_itr_get(b: *mut c_void, row: i32, col: i32, itr: *mut c_void) -> bool;
    #[link_name = "rs_marktree_itr_next"]
    fn rs_marktree_itr_next(b: *mut c_void, itr: *mut c_void) -> bool;
    #[link_name = "rs_marktree_itr_current"]
    fn rs_marktree_itr_current(itr: *mut c_void) -> MTKey;
    #[link_name = "rs_marktree_itr_get_filter"]
    fn rs_marktree_itr_get_filter(
        b: *mut c_void,
        row: i32,
        col: i32,
        stop_row: i32,
        stop_col: i32,
        meta_filter: *const u32,
        itr: *mut c_void,
    ) -> bool;
    #[link_name = "rs_marktree_itr_step_out_filter"]
    fn rs_marktree_itr_step_out_filter(
        b: *mut c_void,
        itr: *mut c_void,
        meta_filter: *const u32,
    ) -> bool;
    #[link_name = "rs_marktree_itr_next_filter"]
    fn rs_marktree_itr_next_filter(
        b: *mut c_void,
        itr: *mut c_void,
        stop_row: i32,
        stop_col: i32,
        meta_filter: *const u32,
    ) -> bool;
    #[link_name = "rs_marktree_get_altpos"]
    fn rs_marktree_get_altpos(b: *mut c_void, mark: MTKey, itr: *mut c_void)
        -> crate::types::MTPos;

    // C accessor wrappers for buf_T fields and inline functions (Phase 1)
    fn nvim_buf_get_marktree(buf: BufHandle) -> *mut c_void;
    fn nvim_buf_meta_total_conceal_lines(buf: BufHandle) -> c_int;
    fn nvim_decor_providers_invoke_conceal_line(wp: WinHandle, row: c_int) -> c_int;
    fn nvim_conceal_cursor_line(wp: WinHandle) -> c_int;
    fn nvim_ns_in_win(ns: u32, wp: WinHandle) -> c_int;
    fn nvim_mt_conceal_lines(mark: MTKey) -> c_int;

    // Window accessors (from nvim-window Rust crate)
    fn nvim_win_get_p_cole(wp: WinHandle) -> i64; // OptInt = i64
    fn nvim_win_get_cursor_lnum(wp: WinHandle) -> c_int; // LineNr = i32
    fn nvim_get_curwin() -> WinHandle;
    // Fold accessor (from nvim-fold Rust crate)
    #[link_name = "rs_hasAnyFolding"]
    fn rs_has_any_folding(wp: WinHandle) -> bool;

    // Memory management (shared with decor.rs)
    fn nvim_xfree_ptr(ptr: *mut c_void);
    fn nvim_clear_virttext(vt: *mut c_void);

    // Phase 5: Redraw Dispatch
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
    fn nvim_decor_items_get_next(idx: u32) -> u32;
    fn nvim_decor_items_get_ptr(idx: u32) -> *mut c_void;
    // Phase 4: bufhl helpers
    fn nvim_extmark_set_hl(
        buf: BufHandle,
        ns_id: c_int,
        row: c_int,
        col: c_int,
        end_row: c_int,
        end_col: c_int,
        hl_id: c_int,
    );

    // Phase 3: Sign buffer operation C accessors
    fn nvim_changed_window_setting(wp: WinHandle);
    fn nvim_may_force_numberwidth_recompute(buf: BufHandle, unplace: bool);
    fn nvim_buf_meta_total(buf: BufHandle, key: c_int) -> c_int;
    fn nvim_buf_signcols_get_count0(buf: BufHandle) -> c_int;
    fn nvim_buf_signcols_set_count0(buf: BufHandle, val: c_int);
    fn nvim_buf_signcols_get_count_at(buf: BufHandle, idx: c_int) -> c_int;
    fn nvim_buf_signcols_set_count_at(buf: BufHandle, idx: c_int, val: c_int);
    fn nvim_buf_signcols_get_max(buf: BufHandle) -> c_int;
    fn nvim_buf_signcols_set_max(buf: BufHandle, val: c_int);
    fn nvim_buf_signcols_get_autom(buf: BufHandle) -> bool;
    fn nvim_get_sign_add_id() -> c_int;
    fn nvim_incr_sign_add_id() -> c_int;
    fn nvim_curtab_first_win() -> WinHandle;
    // nvim_buf_signcols_count_range is no longer needed (buf_signcols_count_range is now Rust)
    fn nvim_win_get_next_in_tab(wp: WinHandle) -> WinHandle;
    fn nvim_win_get_buffer(wp: WinHandle) -> BufHandle;
    // Sign and window accessors for decor_redraw_signs
    fn buf_has_signs(buf: BufHandle) -> bool;
    fn nvim_win_get_minscwidth(wp: WinHandle) -> c_int;
    fn nvim_win_get_scwidth(wp: WinHandle) -> c_int;
    // decor_find_sign(DecorInline) - exported as "decor_find_sign" from decor.rs
    #[link_name = "decor_find_sign"]
    fn decor_find_sign(decor: crate::types::DecorInline) -> *mut crate::types::DecorSignHighlight;
    // rs_sign_item_cmp from nvim-sign crate
    fn rs_sign_item_cmp(
        priority1: c_int,
        id1: u32,
        add_id1: u32,
        priority2: c_int,
        id2: u32,
        add_id2: u32,
    ) -> c_int;
    // Fold accessor (used by decor_virt_lines)
    fn nvim_hasFolding(wp: WinHandle, lnum: c_int, firstp: *mut c_int, lastp: *mut c_int) -> c_int;
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
        unsafe { (*state).itr_valid = false };
    }
}

/// Mark the end of decoration redraw by clearing the window pointer.
///
/// Rust implementation of `decor_redraw_end()`.
#[export_name = "decor_redraw_end"]
pub extern "C" fn rs_decor_redraw_end(state: DecorStateHandle) {
    if state.is_null() {
        return;
    }
    unsafe { (*state).win = std::ptr::null_mut() };
}

/// Free the memory used by DecorState's kvecs.
///
/// Rust implementation of `decor_state_free()`.
#[export_name = "decor_state_free"]
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
#[export_name = "decor_redraw_reset"]
pub extern "C" fn rs_decor_redraw_reset(wp: WinHandle, state: DecorStateHandle) -> bool {
    if state.is_null() || wp.is_null() {
        return false;
    }

    unsafe {
        let s = &mut *state;
        s.row = -1;
        s.win = wp.0;

        let current_end = s.current_end;
        let future_begin = s.future_begin;
        let ranges_count = s.ranges_i.size as c_int;

        // Free owned ranges in [0, current_end) and [future_begin, count)
        free_owned_ranges(state, 0, current_end);
        free_owned_ranges(state, future_begin, ranges_count);

        // Reset kvec sizes and state fields
        (*state).slots.size = 0;
        (*state).ranges_i.size = 0;
        (*state).free_slot_i = -1;
        (*state).current_end = 0;
        (*state).future_begin = 0;
        (*state).new_range_ordering = 0;

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
        let mut count = (*state).ranges_i.size as c_int;
        let cur_end = (*state).current_end;
        let mut fut_beg = (*state).future_begin;

        // Move future ranges to start right after current ranges.
        // Otherwise future ranges will grow forward indefinitely.
        if fut_beg == count {
            count = cur_end;
            fut_beg = cur_end;
        } else if fut_beg != cur_end {
            let move_count = count - fut_beg;
            // Move future ranges to start right after current ranges (memmove semantics)
            let items = (*state).ranges_i.items;
            std::ptr::copy(
                items.add(fut_beg as usize),
                items.add(cur_end as usize),
                move_count as usize,
            );
            count = cur_end + move_count;
            fut_beg = cur_end;
        }

        (*state).ranges_i.size = count as usize;
        (*state).future_begin = fut_beg;
    }
}

// =============================================================================
// MTKey flag constants (from marktree.h)
// =============================================================================

const MT_FLAG_END: u16 = 1 << 1;
const MT_FLAG_INVALID: u16 = 1 << 6;
const MT_FLAG_DECOR_EXT: u16 = 1 << 7;
const MT_FLAG_DECOR_HL: u16 = 1 << 8;
const MT_FLAG_DECOR_SIGNTEXT: u16 = 1 << 9;
const MT_FLAG_DECOR_SIGNHL: u16 = 1 << 10;
const MT_FLAG_DECOR_VIRT_LINES: u16 = 1 << 11;
const MT_FLAG_DECOR_VIRT_TEXT_INLINE: u16 = 1 << 12;
const MT_FLAG_DECOR_MASK: u16 = MT_FLAG_DECOR_EXT
    | MT_FLAG_DECOR_HL
    | MT_FLAG_DECOR_SIGNTEXT
    | MT_FLAG_DECOR_SIGNHL
    | MT_FLAG_DECOR_VIRT_LINES
    | MT_FLAG_DECOR_VIRT_TEXT_INLINE;

// kMTMeta enum indices (from marktree_defs.h) - as usize for filter array indexing
const K_MT_META_LINES: usize = 1; // kMTMetaLines
const K_MT_META_SIGN_HL_IDX: usize = 2; // kMTMetaSignHL
const K_MT_META_SIGN_TEXT_IDX: usize = 3; // kMTMetaSignText
const K_MT_META_CONCEAL_LINES: usize = 4; // kMTMetaConcealLines
const K_MT_META_COUNT: usize = 5; // kMTMetaCount

// kMTMeta enum indices as c_int (for nvim_buf_meta_total)
const K_MT_META_LINES_INT: c_int = 1; // kMTMetaLines
const K_MT_META_SIGN_TEXT: c_int = 3; // kMTMetaSignText

/// kMTFilterSelect value: selects entries that match the filter.
const K_MT_FILTER_SELECT: u32 = u32::MAX;

// Sign column constants
const SCL_NUM: c_int = -2; // signcolumn='number'
const SIGN_SHOW_MAX: usize = 9;
const SIGN_WIDTH: usize = 2; // SIGN_WIDTH - number of schar_T per sign text

// =============================================================================
// Phase 1: decor_redraw_start - Migrated from C
// =============================================================================

/// Initialize DecorState for a window redraw.
///
/// Scans the marktree for marks that overlap the top_row position and adds
/// them to the decoration ranges. Returns true (a hint that decorations
/// may be available in the region; always true for simplicity).
///
/// Rust implementation of `decor_redraw_start()`.
unsafe fn decor_redraw_start_impl(wp: WinHandle, top_row: c_int, state: DecorStateHandle) -> bool {
    let buf = nvim_decor_win_get_buffer(wp);
    let s = &mut *state;
    s.top_row = top_row;
    s.itr_valid = true;

    let tree = nvim_buf_get_marktree(buf);
    let itr_ptr = std::ptr::addr_of_mut!(s.itr).cast::<c_void>();

    if !rs_marktree_itr_get_overlap(tree, top_row, 0, itr_ptr) {
        return false;
    }

    let mut pair = MTPair::default();
    while rs_marktree_itr_step_overlap(tree, itr_ptr, std::ptr::addr_of_mut!(pair)) {
        let m = pair.start;
        if m.flags & MT_FLAG_INVALID != 0 || m.flags & MT_FLAG_DECOR_MASK == 0 {
            continue;
        }
        let ext = m.flags & MT_FLAG_DECOR_EXT != 0;
        let (vt, sh_idx, hl_flags, hl_priority, hl_hl_id, conceal_char) = if ext {
            let d = m.decor_data.ext;
            (d.vt.cast::<c_void>(), d.sh_idx, 0u16, 0u16, 0i32, 0u32)
        } else {
            let hl = m.decor_data.hl;
            (
                std::ptr::null_mut::<c_void>(),
                DECOR_ID_INVALID,
                hl.flags,
                hl.priority,
                hl.hl_id,
                hl.conceal_char,
            )
        };
        crate::range::rs_decor_range_add_from_inline(
            state,
            pair.start.pos.row,
            pair.start.pos.col,
            pair.end_pos.row,
            pair.end_pos.col,
            ext,
            DecorVtHandle(vt),
            sh_idx,
            hl_flags,
            hl_priority,
            hl_hl_id,
            conceal_char,
            false,
            m.ns,
            m.id,
        );
    }

    true
}

// =============================================================================
// Phase 1: decor_conceal_line - Migrated from C
// =============================================================================

/// Conceal filter: selects only kMTMetaConcealLines meta entries.
/// Matches `static const uint32_t conceal_filter[kMTMetaCount]` from C.
/// Index 4 = kMTMetaConcealLines = K_MT_FILTER_SELECT, all others = 0.
static CONCEAL_FILTER: [u32; K_MT_META_COUNT] = [0, 0, 0, 0, K_MT_FILTER_SELECT];

/// Check if a line is concealed by decorations.
///
/// Scans the marktree for conceal_line marks on "row" and invokes any
/// _on_conceal_line decoration provider callbacks if necessary.
///
/// Rust implementation of `decor_conceal_line()`.
#[unsafe(export_name = "decor_conceal_line")]
pub unsafe extern "C" fn rs_decor_conceal_line(
    wp: WinHandle,
    row: c_int,
    check_cursor: bool,
) -> bool {
    let p_cole = nvim_win_get_p_cole(wp);
    if row < 0 || p_cole < 2 {
        return false;
    }
    if !check_cursor {
        let cursor_lnum = nvim_win_get_cursor_lnum(wp);
        let conceal_cur = nvim_conceal_cursor_line(wp) != 0;
        if wp.0 == curwin_handle().0 && row + 1 == cursor_lnum && !conceal_cur {
            return false;
        }
    }

    let buf = nvim_decor_win_get_buffer(wp);
    let tree = nvim_buf_get_marktree(buf);

    // No need to scan the marktree if there are no conceal_line marks.
    if nvim_buf_meta_total_conceal_lines(buf) == 0 {
        return nvim_decor_providers_invoke_conceal_line(wp, row) != 0;
    }

    // Scan the marktree for any conceal_line marks on this row (overlap scan).
    let mut itr_buf = crate::types::MarkTreeIter::new();
    let itr_ptr = std::ptr::addr_of_mut!(itr_buf).cast::<c_void>();
    let mut pair = MTPair::default();
    rs_marktree_itr_get_overlap(tree, row, 0, itr_ptr);
    while rs_marktree_itr_step_overlap(tree, itr_ptr, std::ptr::addr_of_mut!(pair)) {
        if nvim_mt_conceal_lines(pair.start) != 0 && nvim_ns_in_win(pair.start.ns, wp) != 0 {
            return true;
        }
    }

    // Advance iterator out of overlap zone using conceal filter.
    rs_marktree_itr_step_out_filter(tree, itr_ptr, CONCEAL_FILTER.as_ptr());

    loop {
        let mark = rs_marktree_itr_current(itr_ptr);
        // itr->x != NULL check: the iterator is done if pos.row == -1 (sentinel)
        if mark.pos.row < 0 {
            break;
        }
        if mark.pos.row > row {
            break;
        }
        if nvim_mt_conceal_lines(mark) != 0 && nvim_ns_in_win(pair.start.ns, wp) != 0 {
            return true;
        }
        if !rs_marktree_itr_next_filter(tree, itr_ptr, row + 1, 0, CONCEAL_FILTER.as_ptr()) {
            break;
        }
    }

    nvim_decor_providers_invoke_conceal_line(wp, row) != 0
}

/// Wrapper for decor_conceal_line returning int (for backward C compatibility).
#[no_mangle]
pub unsafe extern "C" fn nvim_decor_conceal_line(
    wp: WinHandle,
    row: c_int,
    check_cursor: c_int,
) -> c_int {
    c_int::from(rs_decor_conceal_line(wp, row, check_cursor != 0))
}

/// Return whether a window may have folded or concealed lines.
#[unsafe(export_name = "win_lines_concealed")]
pub unsafe extern "C" fn rs_win_lines_concealed(wp: WinHandle) -> bool {
    rs_has_any_folding(wp) || nvim_win_get_p_cole(wp) >= 2
}

// nvim_win_lines_concealed is provided by drawscreen_shim.c (calls Rust win_lines_concealed)

/// Get the curwin handle for use in conceal_line check.
unsafe fn curwin_handle() -> WinHandle {
    nvim_get_curwin()
}

/// Per-line decoration redraw setup.
///
/// Packs the ranges_i array, optionally starts the marktree scan,
/// and sets row/col_until/eol_col.
///
/// Rust implementation of `decor_redraw_line()`.
#[export_name = "decor_redraw_line"]
pub extern "C" fn rs_decor_redraw_line(wp: WinHandle, row: c_int, state: DecorStateHandle) {
    if state.is_null() || wp.is_null() {
        return;
    }

    unsafe {
        rs_decor_state_pack(state);

        let cur_row = (*state).row;
        if cur_row == -1 {
            decor_redraw_start_impl(wp, row, state);
        } else if !(*state).itr_valid {
            let buf = nvim_decor_win_get_buffer(wp);
            nvim_decor_state_itr_get(state, buf, row, 0);
            (*state).itr_valid = true;
        }

        (*state).row = row;
        (*state).col_until = -1;
        (*state).eol_col = -1;
    }
}

/// Check if there are (likely) more decorations on the current line.
///
/// Checks active ranges, future ranges, and the marktree iterator position.
///
/// Rust implementation of `decor_has_more_decorations()`.
#[export_name = "decor_has_more_decorations"]
pub extern "C" fn rs_decor_has_more_decorations(state: DecorStateHandle, row: c_int) -> bool {
    if state.is_null() {
        return false;
    }

    unsafe {
        let s = &*state;
        let current_end = s.current_end;
        let future_begin = s.future_begin;
        let ranges_count = s.ranges_i.size as c_int;

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
            let vt_typed = cur_vt.0.cast::<DecorVirtText>();
            let flags = (*vt_typed).flags;
            let is_lines = flags & KVT_IS_LINES != 0;
            let is_lines_above = flags & KVT_LINES_ABOVE != 0;
            let below = is_lines && !is_lines_above;
            let vt_lnum = row1 + 1 + c_int::from(below);
            nvim_redraw_buf_line_later(buf, vt_lnum, true);

            let pos = (*vt_typed).pos;
            if is_lines || pos == VirtTextPos::Inline as c_int {
                let vt_col = if is_lines { 0 } else { col1 };
                nvim_changed_lines_invalidate_buf(buf, vt_lnum, vt_col, vt_lnum + 1, 0);
            }
            cur_vt = DecorVtHandle((*vt_typed).next.cast::<c_void>());
        }

        // Walk sign/highlight linked list
        let mut idx = sh_idx;
        while idx != DECOR_ID_INVALID {
            let next = nvim_decor_items_get_next(idx);
            let sh = std::ptr::read(nvim_decor_items_get_ptr(idx).cast::<DecorSignHighlight>());
            rs_decor_redraw_sh(buf, row1, row2, sh);
            idx = next;
        }
    } else {
        // Inline highlight: build types::DecorSignHighlight directly and redraw.
        let sh = DecorSignHighlight {
            flags: hl_flags,
            priority: hl_priority,
            hl_id: hl_hl_id,
            text: [hl_conceal_char, 0],
            sign_name: std::ptr::null_mut(),
            sign_add_id: 0,
            number_hl_id: 0,
            line_hl_id: 0,
            cursorline_hl_id: 0,
            next: crate::decor::DECOR_ID_INVALID,
            _pad_next: 0,
            url: std::ptr::null(),
        };
        rs_decor_redraw_sh(buf, row1, row2, sh);
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
        rs_buf_put_decor_sh(
            buf,
            nvim_decor_items_get_ptr(idx).cast::<DecorSignHighlight>(),
            row,
            clamped_row2,
        );
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
                rs_buf_remove_decor_sh(
                    buf,
                    row1,
                    clamped_row2,
                    nvim_decor_items_get_ptr(idx).cast::<DecorSignHighlight>(),
                );
                idx = next;
            }
        }
    }

    // Optionally free
    if do_free && ext {
        // rs_decor_free takes *mut DecorVirtText; cast from opaque handle
        // DecorVtHandle is repr(transparent) over *mut c_void; transmute to typed ptr
        let vt_ptr: *mut DecorVirtText = std::mem::transmute(vt);
        crate::decor::rs_decor_free(1, vt_ptr, sh_idx);
    }
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

    let current_end = unsafe { (*state).current_end };
    let mut total: c_int = 0;
    let mut right: c_int = 0;

    for i in 0..current_end {
        let range = unsafe { crate::decor_state_get_range(state, i) };
        if range.is_null() {
            continue;
        }

        let start_row = unsafe { (*range).start_row };
        if start_row != row {
            continue;
        }

        if !unsafe { crate::decor_range_has_virt_pos(range) } {
            continue;
        }

        let draw_col = unsafe { (*range).draw_col };
        if draw_col != DRAW_COL_UNSET {
            continue;
        }

        let pos = unsafe { crate::decor_range_virt_pos_kind(range) };
        let kind = unsafe { crate::decor_range_kind(range) };

        if kind != Some(DecorKind::VirtText) {
            continue;
        }

        let vt = unsafe { crate::decor_range_virt_text(range) };
        if vt.is_null() {
            continue;
        }

        let width = unsafe { (*vt).width };

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

    let current_end = unsafe { (*state).current_end };

    for i in 0..current_end {
        let range = unsafe { crate::decor_state_get_range(state, i) };
        if range.is_null() {
            continue;
        }

        let start_row = unsafe { (*range).start_row };
        if start_row == row && unsafe { crate::decor_range_has_virt_pos(range) } {
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
    let current_end = unsafe { (*state).current_end };
    let mut count: c_int = 0;

    for i in 0..current_end {
        let range = unsafe { crate::decor_state_get_range(state, i) };
        if range.is_null() {
            continue;
        }

        let start_row = unsafe { (*range).start_row };
        if start_row != row {
            continue;
        }

        if unsafe { crate::decor_range_kind(range) } != Some(DecorKind::VirtText) {
            continue;
        }

        let pos_kind = unsafe { crate::decor_range_virt_pos_kind(range) };
        if pos_kind == target_pos {
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
    let current_end = unsafe { (*state).current_end };

    for i in 0..current_end {
        let range = unsafe { crate::decor_state_get_range(state, i) };
        if range.is_null() {
            continue;
        }

        let start_row = unsafe { (*range).start_row };
        if start_row != row {
            continue;
        }

        let draw_col = unsafe { (*range).draw_col };
        if draw_col != DRAW_COL_UNSET && draw_col != DRAW_COL_JUST_ADDED {
            continue;
        }

        if unsafe { crate::decor_range_kind(range) } != Some(DecorKind::VirtText) {
            continue;
        }

        let pos_kind = unsafe { crate::decor_range_virt_pos_kind(range) };
        if pos_kind == target_pos {
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
    let current_end = unsafe { (*state).current_end };
    RangeIterator::active(current_end)
}

/// FFI: Create row-filtered iterator.
#[no_mangle]
pub extern "C" fn rs_range_iterator_for_row(state: DecorStateHandle, row: c_int) -> RangeIterator {
    if state.is_null() {
        return RangeIterator::default();
    }
    let current_end = unsafe { (*state).current_end };
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
        return std::ptr::null_mut();
    }
    unsafe { crate::decor_state_get_range(state, iter.index) }
}

// =============================================================================
// Phase 6: Core Column Rendering
// =============================================================================

/// Flat view of a DecorRange, returned by batch accessor.
/// All fields needed by the attribute computation loop in one FFI call.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct DecorRangeFlatView {
    pub start_row: c_int,
    pub start_col: c_int,
    pub end_row: c_int,
    pub end_col: c_int,
    pub attr_id: c_int,
    pub draw_col: c_int,
    pub ordering: c_int,
    pub priority_internal: u32,
    pub kind: u8,
    pub owned: bool,
    pub sh_flags: u16,
    pub sh_text0: ScharT,
    pub sh_url: *const c_char,
    pub has_virt_pos: bool,
    pub slot_index: c_int,
}

extern "C" {
    #[link_name = "hl_combine_attr"]
    fn hl_combine_attr(char_attr: c_int, prim_attr: c_int) -> c_int;

    #[link_name = "hl_add_url"]
    fn hl_add_url(attr: c_int, url: *const c_char) -> c_int;

    #[link_name = "syn_id2attr"]
    fn syn_id2attr(hl_id: c_int) -> c_int;
}

/// TriState values matching C enum: kNone = -1, kFalse = 0, kTrue = 1
const KNONE: c_int = -1;
const KTRUE: c_int = 1;
const KFALSE: c_int = 0;

// =============================================================================
// Phase 2: Compound helper implementations in Rust
// =============================================================================

/// Get the slot pointer for a given slot index.
///
/// # Safety
/// `state` must be valid and `slot_index` must be in bounds.
unsafe fn get_slot_range(slots: *mut KVec<DecorRangeSlot>, slot_index: c_int) -> *mut DecorRange {
    (*slots)
        .get_unchecked(slot_index as usize)
        .cast::<DecorRange>()
}

/// Retire a range: free owned data and return slot to freelist.
///
/// # Safety
/// `state` must be a valid non-null pointer.
unsafe fn retire_range(state: DecorStateHandle, slot_index: c_int) {
    let s = &mut *state;
    let r_ptr = get_slot_range(std::ptr::addr_of_mut!(s.slots), slot_index);
    let r = &*r_ptr; // create reference to avoid implicit autoref of union fields

    if r.owned {
        let kind = r.kind;
        if kind == DecorKind::VirtText as u8 || kind == DecorKind::VirtLines as u8 {
            let vt = r.data.vt;
            if !vt.is_null() {
                // Clear the virt_text data then free the allocation
                let vt_data_ptr = std::ptr::addr_of_mut!((*vt).data.virt_text).cast::<c_void>();
                nvim_clear_virttext(vt_data_ptr);
                nvim_xfree_ptr(vt.cast::<c_void>());
            }
        } else if kind == DecorKind::Highlight as u8 {
            let url = r.data.sh.url;
            if !url.is_null() {
                nvim_xfree_ptr(url.cast_mut().cast::<c_void>());
            }
        }
    }

    // Return slot to freelist
    let slot = &mut *r_ptr.cast::<DecorRangeSlot>();
    slot.next_free_i = s.free_slot_i;
    s.free_slot_i = slot_index;
}

/// Free owned ranges in the given index range [beg, end).
///
/// # Safety
/// `state` must be a valid non-null pointer.
unsafe fn free_owned_ranges(state: DecorStateHandle, beg: c_int, end: c_int) {
    let s = &*state;
    for i in beg..end {
        let slot_index = *s.ranges_i.get_unchecked(i as usize);
        let slot = s
            .slots
            .get_unchecked(slot_index as usize)
            .cast::<DecorRange>();
        let r = &*slot;
        if r.owned && r.kind == DecorKind::VirtText as u8 {
            let vt = r.data.vt;
            if !vt.is_null() {
                let vt_data_ptr = std::ptr::addr_of!((*vt).data.virt_text)
                    .cast_mut()
                    .cast::<c_void>();
                nvim_clear_virttext(vt_data_ptr);
                nvim_xfree_ptr(vt.cast::<c_void>());
            }
        }
    }
}

/// Get a flat view of the i-th active range (by ranges_i index).
///
/// # Safety
/// `state` must be valid and `i` must be in `[0, current_end)`.
unsafe fn get_range_flat(state: DecorStateHandle, i: c_int) -> DecorRangeFlatView {
    let s = &*state;
    let slot_index = *s.ranges_i.get_unchecked(i as usize);
    let slot = s
        .slots
        .get_unchecked(slot_index as usize)
        .cast::<DecorRange>();
    let r = &*slot;

    let is_hl = r.kind == DecorKind::Highlight as u8;
    let (sh_flags, sh_text0, sh_url) = if is_hl {
        // Explicit reference to union field to avoid implicit autoref lint
        let sh = &r.data.sh;
        (sh.flags, sh.text[0], sh.url)
    } else {
        (0u16, 0u32, std::ptr::null())
    };

    let has_virt_pos = r.kind == DecorKind::VirtText as u8 || r.kind == DecorKind::UIWatched as u8;

    DecorRangeFlatView {
        start_row: r.start_row,
        start_col: r.start_col,
        end_row: r.end_row,
        end_col: r.end_col,
        attr_id: r.attr_id,
        draw_col: r.draw_col,
        ordering: r.ordering,
        priority_internal: r.priority_internal,
        kind: r.kind,
        owned: r.owned,
        sh_flags,
        sh_text0,
        sh_url,
        has_virt_pos,
        slot_index,
    }
}

/// Write back an index into the ranges_i array.
///
/// # Safety
/// `state` must be valid and `pos` must be in bounds.
unsafe fn set_ranges_i_at(state: DecorStateHandle, pos: c_int, value: c_int) {
    *(*state).ranges_i.get_unchecked(pos as usize) = value;
}

/// Initialize draw_col for a range at the given slot_index.
///
/// # Safety
/// `state` must be valid and `slot_index` must be in bounds.
unsafe fn col_init_draw_col(
    state: DecorStateHandle,
    slot_index: c_int,
    win_col: c_int,
    hidden: bool,
) {
    let s = &mut *state;
    let r = get_slot_range(std::ptr::addr_of_mut!(s.slots), slot_index);
    decor_init_draw_col_export(win_col, hidden, r);
}

/// Update DecorState output fields after attribute computation.
///
/// # Safety
/// `state` must be a valid non-null pointer.
unsafe fn col_update_state(
    state: DecorStateHandle,
    col_until: c_int,
    cur_end: c_int,
    fut_beg: c_int,
    count: c_int,
    attr: c_int,
    conceal: c_int,
    conceal_char: ScharT,
    conceal_attr: c_int,
    spell: c_int,
) {
    let s = &mut *state;
    s.ranges_i.size = count as usize;
    s.future_begin = fut_beg;
    s.current_end = cur_end;
    s.col_until = col_until;
    s.current = attr;
    s.conceal = conceal;
    s.conceal_char = conceal_char;
    s.conceal_attr = conceal_attr;
    s.spell = spell;
}

/// Promote future ranges that start at or before (row, col) into the active set.
///
/// This is Part 2 of the former `nvim_decor_col_advance` C function.
/// Returns (col_until, cur_end, fut_beg, count).
///
/// # Safety
/// `state` must be a valid non-null pointer.
unsafe fn promote_future_ranges(
    state: DecorStateHandle,
    col: c_int,
    col_until_in: c_int,
) -> (c_int, c_int, c_int, c_int) {
    let s = &mut *state;
    let row = s.row;
    let indices = s.ranges_i.items;
    let slots = s.slots.items;

    let count = s.ranges_i.size as c_int;
    let mut cur_end = s.current_end;
    let mut fut_beg = s.future_begin;
    let mut col_until = col_until_in;

    // Promote future ranges before the cursor to active.
    while fut_beg < count {
        let index = *indices.add(fut_beg as usize);
        let r = &(*slots.add(index as usize)).range;
        if r.start_row > row || (r.start_row == row && r.start_col > col) {
            break;
        }
        let ordering = r.ordering;
        let priority = r.priority_internal;

        // Binary search for insertion position by (priority, ordering) descending
        let mut begin = 0;
        let mut end = cur_end;
        while begin < end {
            let mid = begin + ((end - begin) >> 1);
            let mi = *indices.add(mid as usize);
            let mr = &(*slots.add(mi as usize)).range;
            if mr.priority_internal < priority
                || (mr.priority_internal == priority && mr.ordering < ordering)
            {
                begin = mid + 1;
            } else {
                end = mid;
            }
        }

        // Shift right and insert
        let item = indices.add(begin as usize);
        std::ptr::copy(item, item.add(1), (cur_end - begin) as usize);
        item.write(index);
        cur_end += 1;
        fut_beg += 1;
    }

    // Update col_until from the next future range
    if fut_beg < count {
        let r = &(*slots.add(*indices.add(fut_beg as usize) as usize)).range;
        if r.start_row == row {
            col_until = col_until.min(r.start_col - 1);
        }
    }

    (col_until, cur_end, fut_beg, count)
}

/// Advance the marktree iterator for column decorations.
///
/// Rust implementation of C's `nvim_decor_col_iter_marks()`.
///
/// Walks the marktree iterator from the current position, adding inline decorations
/// to the decor state. Returns the `col_until` value (next mark column - 1, or MAXCOL).
///
/// # Safety
/// `wp` and `state` must be valid.
unsafe fn decor_col_iter_marks_impl(wp: WinHandle, col: c_int, state: DecorStateHandle) -> c_int {
    let buf = nvim_decor_win_get_buffer(wp);
    let tree = nvim_buf_get_marktree(buf);
    let row = (*state).row;
    let mut col_until = i32::MAX; // MAXCOL

    let itr_ptr = std::ptr::addr_of_mut!((*state).itr).cast::<c_void>();

    loop {
        let mark = rs_marktree_itr_current(itr_ptr);
        if mark.pos.row < 0 || mark.pos.row > row {
            break;
        }
        if mark.pos.row == row && mark.pos.col > col {
            col_until = mark.pos.col - 1;
            break;
        }

        // mt_invalid || mt_end || !mt_decor_any || !ns_in_win: skip
        if mark.flags & MT_FLAG_INVALID != 0
            || mark.flags & MT_FLAG_END != 0
            || mark.flags & MT_FLAG_DECOR_MASK == 0
            || nvim_ns_in_win(mark.ns, wp) == 0
        {
            rs_marktree_itr_next(tree, itr_ptr);
            continue;
        }

        let endpos = rs_marktree_get_altpos(tree, mark, std::ptr::null_mut());
        // mt_decor(mark): construct DecorInline from mark flags and decor_data
        let ext = mark.flags & MT_FLAG_DECOR_EXT != 0;
        let (vt_handle, sh_idx, hl_flags, hl_priority, hl_hl_id, conceal_char) = if ext {
            let d = mark.decor_data.ext;
            (
                crate::range::DecorVtHandle(d.vt.cast::<c_void>()),
                d.sh_idx,
                0u16,
                0u16,
                0i32,
                0u32,
            )
        } else {
            let hl = mark.decor_data.hl;
            (
                crate::range::DecorVtHandle(std::ptr::null_mut()),
                crate::decor::DECOR_ID_INVALID,
                hl.flags,
                hl.priority,
                hl.hl_id,
                hl.conceal_char,
            )
        };

        crate::range::rs_decor_range_add_from_inline(
            state,
            mark.pos.row,
            mark.pos.col,
            endpos.row,
            endpos.col,
            ext,
            vt_handle,
            sh_idx,
            hl_flags,
            hl_priority,
            hl_hl_id,
            conceal_char,
            false,
            mark.ns,
            mark.id,
        );

        rs_marktree_itr_next(tree, itr_ptr);
    }

    col_until
}

/// Core column rendering implementation.
///
/// This is the hot-path function called for each column during line rendering.
/// It:
/// 1. Advances the marktree iterator (Rust)
/// 2. Promotes future ranges to active (Rust)
/// 3. Computes highlight, conceal, and spell attributes for active ranges
/// 4. Retires expired ranges
/// 5. Updates DecorState output fields
///
/// Rust implementation of `decor_redraw_col_impl()`.
#[export_name = "decor_redraw_col_impl"]
pub unsafe extern "C" fn rs_decor_redraw_col_impl(
    wp: WinHandle,
    col: c_int,
    win_col: c_int,
    hidden: bool,
    state: DecorStateHandle,
) -> c_int {
    // Part 1: Advance marktree iterator, get initial col_until
    let col_until_from_itr = decor_col_iter_marks_impl(wp, col, state);

    // Part 2: Promote future ranges to active (Rust)
    let (mut col_until, cur_end, fut_beg, count) =
        promote_future_ranges(state, col, col_until_from_itr);

    let row = (*state).row;

    // Part 3: Attribute computation loop
    let mut new_cur_end: c_int = 0;
    let mut attr: c_int = 0;
    let mut conceal: c_int = 0;
    let mut conceal_char: ScharT = 0;
    let mut conceal_attr: c_int = 0;
    let mut spell: c_int = KNONE;

    for i in 0..cur_end {
        let v = get_range_flat(state, i);

        let keep;
        if range_end_before(v.end_row, v.end_col, row, col) {
            keep = v.start_row >= row && v.has_virt_pos;
        } else {
            keep = true;

            if v.end_row == row && v.end_col > col {
                col_until = col_until.min(v.end_col - 1);
            }

            if v.attr_id > 0 {
                attr = hl_combine_attr(attr, v.attr_id);
            }

            if v.kind == DecorKind::Highlight as u8 && (v.sh_flags & KSH_CONCEAL != 0) {
                conceal = 1;
                if v.start_row == row && v.start_col == col {
                    conceal = 2;
                    conceal_char = v.sh_text0;
                    col_until = col_until.min(v.start_col);
                    conceal_attr = v.attr_id;
                }
            }

            if v.kind == DecorKind::Highlight as u8 {
                if v.sh_flags & KSH_SPELL_ON != 0 {
                    spell = KTRUE;
                } else if v.sh_flags & KSH_SPELL_OFF != 0 {
                    spell = KFALSE;
                }
                if !v.sh_url.is_null() {
                    attr = hl_add_url(attr, v.sh_url);
                }
            }
        }

        if v.start_row == row
            && v.start_col <= col
            && v.has_virt_pos
            && v.draw_col == DRAW_COL_JUST_ADDED
        {
            col_init_draw_col(state, v.slot_index, win_col, hidden);
        }

        if keep {
            set_ranges_i_at(state, new_cur_end, v.slot_index);
            new_cur_end += 1;
        } else {
            retire_range(state, v.slot_index);
        }
    }

    let final_cur_end = new_cur_end;
    let mut final_fut_beg = fut_beg;
    let mut final_count = count;

    if final_fut_beg == final_count {
        final_fut_beg = final_cur_end;
        final_count = final_cur_end;
    }

    // Part 3: Update state directly
    col_update_state(
        state,
        col_until,
        final_cur_end,
        final_fut_beg,
        final_count,
        attr,
        conceal,
        conceal_char,
        conceal_attr,
        spell,
    );

    attr
}

// =============================================================================
// Phase 1: DecorInline-accepting wrappers
// =============================================================================

use crate::types::DecorInline;

/// Direct export of decor_redraw(buf_T *buf, int row1, int row2, int col1, DecorInline decor).
/// Replaces C wrapper that unpacked DecorInline and called rs_decor_redraw.
#[export_name = "decor_redraw"]
pub unsafe extern "C" fn decor_redraw_export(
    buf: BufHandle,
    row1: c_int,
    row2: c_int,
    col1: c_int,
    decor: DecorInline,
) {
    let ext = decor.ext;
    // Safety: we branch on ext before reading the union variant
    let (vt, sh_idx, hl_flags, hl_priority, hl_hl_id, hl_conceal_char) = if ext {
        let e = unsafe { &*decor.data.ext };
        (
            DecorVtHandle(e.vt.cast::<c_void>()),
            e.sh_idx,
            0u16,
            0u16,
            0i32,
            0u32,
        )
    } else {
        let h = unsafe { &*decor.data.hl };
        (
            DecorVtHandle(std::ptr::null_mut()),
            0u32,
            h.flags,
            h.priority,
            h.hl_id,
            h.conceal_char,
        )
    };
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
}

/// Direct export of buf_put_decor(buf_T *buf, DecorInline decor, int row, int row2).
/// Replaces C wrapper that unpacked DecorInline and called rs_buf_put_decor.
#[export_name = "buf_put_decor"]
pub unsafe extern "C" fn buf_put_decor_export(
    buf: BufHandle,
    decor: DecorInline,
    row: c_int,
    row2: c_int,
) {
    let ext = decor.ext;
    // Safety: we branch on ext before reading the union variant
    let (vt, sh_idx) = if ext {
        let e = unsafe { &*decor.data.ext };
        (DecorVtHandle(e.vt.cast::<c_void>()), e.sh_idx)
    } else {
        (DecorVtHandle(std::ptr::null_mut()), 0u32)
    };
    rs_buf_put_decor(buf, ext, vt, sh_idx, row, row2);
}

/// Direct export of buf_decor_remove(buf_T *buf, int row1, int row2, int col1, DecorInline decor, bool free).
/// Replaces C wrapper that unpacked DecorInline and called rs_buf_decor_remove.
#[export_name = "buf_decor_remove"]
pub unsafe extern "C" fn buf_decor_remove_export(
    buf: BufHandle,
    row1: c_int,
    row2: c_int,
    col1: c_int,
    decor: DecorInline,
    do_free: bool,
) {
    let ext = decor.ext;
    // Safety: we branch on ext before reading the union variant
    let (vt, sh_idx, hl_flags, hl_priority, hl_hl_id, hl_conceal_char) = if ext {
        let e = unsafe { &*decor.data.ext };
        (
            DecorVtHandle(e.vt.cast::<c_void>()),
            e.sh_idx,
            0u16,
            0u16,
            0i32,
            0u32,
        )
    } else {
        let h = unsafe { &*decor.data.hl };
        (
            DecorVtHandle(std::ptr::null_mut()),
            0u32,
            h.flags,
            h.priority,
            h.hl_id,
            h.conceal_char,
        )
    };
    rs_buf_decor_remove(
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
        do_free,
    );
}

// =============================================================================
// Phase 3: Self-contained function migrations
// =============================================================================

/// Initialize the draw_col of a newly-added virtual text item.
///
/// Rust replacement for C `decor_init_draw_col`.
///
/// # Safety
/// `item` must be a valid non-null pointer to a `DecorRange`.
#[export_name = "decor_init_draw_col"]
pub unsafe extern "C" fn decor_init_draw_col_export(
    win_col: c_int,
    hidden: bool,
    item: *mut DecorRange,
) {
    (*item).draw_col = crate::decor::rs_decor_init_draw_col(win_col, c_int::from(hidden), item);
}

/// Recheck draw_col for all active ranges that need it.
///
/// Rust replacement for C `decor_recheck_draw_col`.
///
/// # Safety
/// `state` must be a valid non-null `DecorState` pointer.
#[export_name = "decor_recheck_draw_col"]
pub unsafe extern "C" fn decor_recheck_draw_col_export(
    win_col: c_int,
    hidden: bool,
    state: DecorStateHandle,
) {
    let s = &*state;
    let end = s.current_end;
    let ranges_i_items = s.ranges_i.items;
    let slots_items = s.slots.items;
    for i in 0..end {
        let slot_idx = *ranges_i_items.add(i as usize);
        let r = slots_items.add(slot_idx as usize).cast::<DecorRange>();
        if crate::decor::should_recheck_draw_col((*r).draw_col) {
            decor_init_draw_col_export(win_col, hidden, r);
        }
    }
}

/// Handle EOL decorations: advance to end of line and collect eol attributes.
///
/// Returns true if any active range has a virtual position on the current row.
///
/// Rust replacement for C `decor_redraw_eol`.
///
/// # Safety
/// `wp`, `state`, `eol_attr` must be valid non-null pointers.
#[export_name = "decor_redraw_eol"]
pub unsafe extern "C" fn decor_redraw_eol_export(
    wp: WinHandle,
    state: DecorStateHandle,
    eol_attr: *mut c_int,
    eol_col: c_int,
) -> bool {
    // MAXCOL = 0x7fffffff = i32::MAX
    rs_decor_redraw_col_impl(wp, i32::MAX, i32::MAX, false, state);
    (*state).eol_col = eol_col;

    let count = (*state).current_end;
    let ranges_i_items = (*state).ranges_i.items;
    let slots_items = (*state).slots.items;

    let mut has_virt_pos = false;
    for i in 0..count {
        let slot_idx = *ranges_i_items.add(i as usize);
        let r = &*slots_items.add(slot_idx as usize).cast::<DecorRange>();
        if r.start_row == (*state).row {
            has_virt_pos |= crate::decor_range_has_virt_pos(std::ptr::from_ref(r).cast_mut());
        }
        if crate::decor::decor_kind_is_highlight(
            crate::DecorKind::from_c_int(c_int::from(r.kind))
                .unwrap_or(crate::DecorKind::Highlight),
        ) && crate::decor::sh_is_hl_eol(r.data.sh.flags)
        {
            *eol_attr = hl_combine_attr(*eol_attr, r.attr_id);
        }
    }
    has_virt_pos
}

/// Get the next chunk of a virtual text item.
///
/// Rust replacement for C `next_virt_text_chunk`.
///
/// # Safety
/// `pos` and `attr` must be valid non-null pointers.
#[export_name = "next_virt_text_chunk"]
pub unsafe extern "C" fn next_virt_text_chunk_export(
    vt: VirtText,
    pos: *mut usize,
    attr: *mut c_int,
) -> *mut c_char {
    let mut text: *mut c_char = std::ptr::null_mut();
    while text.is_null() && *pos < vt.size {
        let chunk = &*vt.items.add(*pos).cast_const();
        *pos += 1;
        text = chunk.text;
        let hl_id = chunk.hl_id;
        if hl_id >= 0 {
            *attr = (*attr).max(0);
            if hl_id > 0 {
                *attr = hl_combine_attr(*attr, syn_id2attr(hl_id));
            }
        }
    }
    text
}

/// Get the total EOL right-aligned virtual text width from `from_idx` onwards.
///
/// Rust replacement for C `nvim_decor_state_get_eol_right_width`.
///
/// # Safety
/// `state_ptr` must be a valid non-null `DecorState` pointer.
#[no_mangle]
pub unsafe extern "C" fn nvim_decor_state_get_eol_right_width(
    state_ptr: DecorStateHandle,
    from_idx: c_int,
) -> c_int {
    let state = &*state_ptr;
    let count = state.ranges_i.size as c_int;
    let ranges_i_items = state.ranges_i.items;
    let slots_items = state.slots.items;

    let mut total_width: c_int = 0;
    let mut j = from_idx;
    while j < state.current_end && j < count {
        let slot_idx = *ranges_i_items.add(j as usize);
        let r = &*slots_items.add(slot_idx as usize).cast::<DecorRange>();

        if r.start_row != state.row
            || !crate::decor_range_has_virt_pos(std::ptr::from_ref(r).cast_mut())
            || r.draw_col != -1
        {
            j += 1;
            continue;
        }

        if crate::decor_range_virt_pos_kind(std::ptr::from_ref(r).cast_mut())
            == Some(crate::VirtTextPos::EndOfLineRightAlign)
            && r.kind == crate::DecorKind::VirtText as u8
        {
            let vt = r.data.vt;
            if !vt.is_null() {
                total_width += (*vt).width as c_int + 1;
            }
        }
        j += 1;
    }

    if total_width > 0 {
        total_width -= 1;
    }
    total_width
}

// =============================================================================
// Phase 3: Sign buffer operations migrated from C
// =============================================================================

/// kFalse from tristate (TriState is 0=kFalse, 1=kTrue, -1=kNone).
const K_FALSE: c_int = 0;

/// Trigger redraw for sign/hl/spell/conceal decoration changes.
///
/// Rust implementation of `decor_redraw_sh()`.
///
/// # Safety
/// `buf` must be a valid buf_T pointer.
#[export_name = "decor_redraw_sh"]
pub unsafe extern "C" fn rs_decor_redraw_sh(
    buf: BufHandle,
    row1: c_int,
    row2: c_int,
    sh: DecorSignHighlight,
) {
    use crate::decor::{KSH_CONCEAL, KSH_SPELL_OFF, KSH_SPELL_ON};
    use crate::decor::{KSH_CONCEAL_LINES, KSH_IS_SIGN, KSH_UI_WATCHED};

    let flags = sh.flags;

    if row2 >= row1
        && (sh.hl_id != 0
            || !sh.url.is_null()
            || (flags & KSH_IS_SIGN != 0)
            || (flags & KSH_SPELL_ON != 0)
            || (flags & KSH_SPELL_OFF != 0)
            || (flags & KSH_CONCEAL != 0))
    {
        nvim_redraw_buf_range_later(buf, row1 + 1, row2 + 1);
    }

    if flags & KSH_CONCEAL_LINES != 0 {
        let mut wp = nvim_curtab_first_win();
        while !wp.is_null() {
            if nvim_win_get_buffer(wp) == buf {
                nvim_changed_window_setting(wp);
            }
            wp = nvim_win_get_next_in_tab(wp);
        }
    }

    if flags & KSH_UI_WATCHED != 0 {
        nvim_redraw_buf_line_later(buf, row1 + 1, false);
    }
}

/// Place sign decoration in buffer.
///
/// Rust implementation of `buf_put_decor_sh()`.
///
/// # Safety
/// `buf` and `sh` must be valid pointers.
#[export_name = "buf_put_decor_sh"]
pub unsafe extern "C" fn rs_buf_put_decor_sh(
    buf: BufHandle,
    sh: *mut DecorSignHighlight,
    row1: c_int,
    row2: c_int,
) {
    use crate::decor::KSH_IS_SIGN;

    if sh.is_null() {
        return;
    }
    let flags = (*sh).flags;
    if flags & KSH_IS_SIGN != 0 {
        (*sh).sign_add_id = nvim_incr_sign_add_id();
        if (*sh).text[0] != 0 {
            buf_signcols_count_range_impl(buf, row1, row2, 1, K_FALSE);
            nvim_may_force_numberwidth_recompute(buf, false);
        }
    }
}

/// Remove sign decoration from buffer.
///
/// Rust implementation of `buf_remove_decor_sh()`.
///
/// # Safety
/// `buf` and `sh` must be valid pointers.
#[export_name = "buf_remove_decor_sh"]
pub unsafe extern "C" fn rs_buf_remove_decor_sh(
    buf: BufHandle,
    row1: c_int,
    row2: c_int,
    sh: *mut DecorSignHighlight,
) {
    use crate::decor::KSH_IS_SIGN;

    if sh.is_null() {
        return;
    }
    let flags = (*sh).flags;
    if flags & KSH_IS_SIGN != 0 && (*sh).text[0] != 0 {
        if nvim_buf_meta_total(buf, K_MT_META_SIGN_TEXT) != 0 {
            buf_signcols_count_range_impl(buf, row1, row2, -1, K_FALSE);
        } else {
            nvim_may_force_numberwidth_recompute(buf, true);
            nvim_buf_signcols_set_count0(buf, 0);
            nvim_buf_signcols_set_max(buf, 0);
        }
    }
}

// =============================================================================
// Phase 2: decor_virt_lines and decor_find_virttext - Migrated from C
// =============================================================================

/// Lines filter: selects only kMTMetaLines meta entries.
/// Matches `static const uint32_t lines_filter[kMTMetaCount]` from C.
/// Index 1 = kMTMetaLines = K_MT_FILTER_SELECT, all others = 0.
static LINES_FILTER: [u32; K_MT_META_COUNT] = [0, K_MT_FILTER_SELECT, 0, 0, 0];

/// Splice src VirtLines into dest (appends all src entries to dest).
/// Equivalent to C `kv_splice(*dest, src)`.
///
/// # Safety
/// `dest` must be valid. `src` contents are bitwise-copied.
///
/// # Panics
/// Panics if `libc::realloc` returns null (out of memory).
unsafe fn virt_lines_splice(dest: &mut VirtLines, src: &VirtLines) {
    let n = src.size;
    if n == 0 {
        return;
    }
    // Grow dest to fit n more items.
    let new_size = dest.size + n;
    if new_size > dest.capacity {
        let new_cap = new_size.next_power_of_two().max(8);
        let bytes = new_cap * std::mem::size_of::<crate::types::VirtLine>();
        let ptr = libc::realloc(dest.items.cast::<libc::c_void>(), bytes);
        assert!(!ptr.is_null(), "virt_lines_splice: realloc failed");
        dest.items = ptr.cast();
        dest.capacity = new_cap;
    }
    // Copy src items into dest.
    libc::memcpy(
        dest.items.add(dest.size).cast::<libc::c_void>(),
        src.items.cast::<libc::c_void>(),
        n * std::mem::size_of::<crate::types::VirtLine>(),
    );
    dest.size = new_size;
}

/// Find the first non-virt_lines virtual text decoration on a row.
///
/// Rust implementation of `decor_find_virttext()`.
///
/// # Safety
/// `buf` must be valid. Returned pointer is into the marktree, not owned.
#[unsafe(export_name = "decor_find_virttext")]
pub unsafe extern "C" fn rs_decor_find_virttext(
    buf: BufHandle,
    row: c_int,
    ns_id: u64,
) -> *mut DecorVirtText {
    let tree = nvim_buf_get_marktree(buf);
    let mut itr_buf = crate::types::MarkTreeIter::new();
    let itr_ptr = std::ptr::addr_of_mut!(itr_buf).cast::<c_void>();
    rs_marktree_itr_get(tree, row, 0, itr_ptr);

    loop {
        let mark = rs_marktree_itr_current(itr_ptr);
        // itr->x != NULL equivalent: pos.row < 0 is sentinel
        if mark.pos.row < 0 || mark.pos.row > row {
            break;
        }
        if mark.flags & MT_FLAG_INVALID == 0 {
            // mt_decor_virt(mark): returns vt if MT_FLAG_DECOR_EXT set
            let vt_ptr: *mut DecorVirtText = if mark.flags & MT_FLAG_DECOR_EXT != 0 {
                mark.decor_data.ext.vt
            } else {
                std::ptr::null_mut()
            };
            // Skip all virt_lines decorations to find the first virt_text
            let mut vt = vt_ptr;
            while !vt.is_null() && ((*vt).flags & KVT_IS_LINES != 0) {
                vt = (*vt).next;
            }
            if (ns_id == 0 || ns_id == u64::from(mark.ns)) && !vt.is_null() {
                return vt;
            }
        }
        rs_marktree_itr_next(tree, itr_ptr);
    }
    std::ptr::null_mut()
}

/// Count virtual lines in a range (and optionally collect them).
///
/// Rust implementation of `decor_virt_lines()`.
///
/// @param apply_folds  Only count virtual lines that are not in folds.
///
/// # Safety
/// `wp` must be valid. `num_below` and `lines` may be null.
///
/// # Panics
/// Panics if memory reallocation fails when appending to `lines`.
#[unsafe(export_name = "decor_virt_lines")]
pub unsafe extern "C" fn rs_decor_virt_lines(
    wp: WinHandle,
    start_row: c_int,
    end_row: c_int,
    num_below: *mut c_int,
    lines: *mut VirtLines,
    apply_folds: bool,
) -> c_int {
    let buf = nvim_decor_win_get_buffer(wp);
    if nvim_buf_meta_total(buf, K_MT_META_LINES_INT) == 0 {
        // Only pay for what you use: no virt_lines active.
        return 0;
    }

    let tree = nvim_buf_get_marktree(buf);
    let mut itr_buf = crate::types::MarkTreeIter::new();
    let itr_ptr = std::ptr::addr_of_mut!(itr_buf).cast::<c_void>();

    if !rs_marktree_itr_get_filter(
        tree,
        (start_row - 1).max(0),
        0,
        end_row,
        0,
        LINES_FILTER.as_ptr(),
        itr_ptr,
    ) {
        return 0;
    }

    assert!(start_row >= 0);

    let mut virt_lines_total: c_int = 0;

    loop {
        let mark = rs_marktree_itr_current(itr_ptr);
        // mt_decor_virt: returns vt if MT_FLAG_DECOR_EXT set
        let vt_ptr: *mut DecorVirtText = if mark.flags & MT_FLAG_DECOR_EXT != 0 {
            mark.decor_data.ext.vt
        } else {
            std::ptr::null_mut()
        };
        if mark.flags & MT_FLAG_INVALID == 0 && nvim_ns_in_win(mark.ns, wp) != 0 {
            let mut vt = vt_ptr;
            while !vt.is_null() {
                let flags = (*vt).flags;
                if flags & KVT_IS_LINES != 0 {
                    let above = flags & KVT_LINES_ABOVE != 0;
                    let mrow = mark.pos.row;
                    let draw_row = mrow + i32::from(!above);
                    if draw_row >= start_row && draw_row < end_row {
                        let skip = apply_folds
                            && (nvim_hasFolding(
                                wp,
                                mrow + 1,
                                std::ptr::null_mut(),
                                std::ptr::null_mut(),
                            ) != 0
                                || rs_decor_conceal_line(wp, mrow, false));
                        if !skip {
                            let vt_lines = &(*vt).data.virt_lines;
                            let n = vt_lines.size as c_int;
                            virt_lines_total += n;
                            if !lines.is_null() {
                                virt_lines_splice(&mut *lines, vt_lines);
                            }
                            if !num_below.is_null() && !above {
                                *num_below += n;
                            }
                        }
                    }
                }
                vt = (*vt).next;
            }
        }

        if !rs_marktree_itr_next_filter(tree, itr_ptr, end_row, 0, LINES_FILTER.as_ptr()) {
            break;
        }
    }

    virt_lines_total
}

// =============================================================================
// Phase 3: decor_redraw_signs and buf_signcols_count_range - Migrated from C
// =============================================================================

/// Sign filter: selects kMTMetaSignText and kMTMetaSignHL meta entries.
/// Matches `static const uint32_t sign_filter[kMTMetaCount]` from C.
/// Index 2 = kMTMetaSignHL, index 3 = kMTMetaSignText = K_MT_FILTER_SELECT.
static SIGN_FILTER: [u32; K_MT_META_COUNT] = [0, 0, K_MT_FILTER_SELECT, K_MT_FILTER_SELECT, 0];

/// Signtext filter: selects only kMTMetaSignText meta entries.
/// Matches `static const uint32_t signtext_filter[kMTMetaCount]` from C.
static SIGNTEXT_FILTER: [u32; K_MT_META_COUNT] = [0, 0, 0, K_MT_FILTER_SELECT, 0];

/// Local mirror of C's SignItem: pointer to DecorSignHighlight + mark id.
/// Layout: { *mut DecorSignHighlight sh @ 0, u32 id @ 8 }. Size = 16 bytes.
#[repr(C)]
pub(crate) struct SignItem {
    sh: *mut DecorSignHighlight,
    id: u32,
}

/// Local mirror of C's SignTextAttrs.
///
/// Layout: `schar_T text[SIGN_WIDTH]` = `u32[2]` at offset 0, `int hl_id` = `i32` at offset 8.
#[repr(C)]
pub struct SignTextAttrsLocal {
    pub text: [ScharT; SIGN_WIDTH],
    pub hl_id: c_int,
}

/// Return the signs and highest priority sign attributes on a row.
///
/// Rust implementation of `decor_redraw_signs()`.
///
/// # Safety
/// All pointers must be valid. `sattrs` may be null, `line_id`/`cul_id`/`num_id` may be null.
#[allow(clippy::too_many_lines)]
#[unsafe(export_name = "decor_redraw_signs")]
pub unsafe extern "C" fn rs_decor_redraw_signs(
    wp: WinHandle,
    buf: BufHandle,
    row: c_int,
    sattrs: *mut SignTextAttrsLocal,
    line_id: *mut c_int,
    cul_id: *mut c_int,
    num_id: *mut c_int,
) {
    if !buf_has_signs(buf) {
        return;
    }

    let tree = nvim_buf_get_marktree(buf);
    let mut itr_buf = crate::types::MarkTreeIter::new();
    let itr_ptr = std::ptr::addr_of_mut!(itr_buf).cast::<c_void>();
    let mut signs: Vec<SignItem> = Vec::new();
    let mut num_text: c_int = 0;

    // Collect signs that overlap this row (start before row).
    rs_marktree_itr_get_overlap(tree, row, 0, itr_ptr);
    let mut pair = MTPair::default();
    while rs_marktree_itr_step_overlap(tree, itr_ptr, std::ptr::addr_of_mut!(pair)) {
        let start = pair.start;
        if start.flags & MT_FLAG_INVALID == 0
            && (start.flags & (MT_FLAG_DECOR_SIGNTEXT | MT_FLAG_DECOR_SIGNHL)) != 0
            && nvim_ns_in_win(start.ns, wp) != 0
        {
            let decor = crate::types::DecorInline {
                ext: true,
                _pad: [0u8; 7],
                data: crate::types::DecorInlineData {
                    ext: std::mem::ManuallyDrop::new(crate::types::DecorExt {
                        sh_idx: start.decor_data.ext.sh_idx,
                        _pad: 0,
                        vt: start.decor_data.ext.vt,
                    }),
                },
            };
            let sh = decor_find_sign(decor);
            if !sh.is_null() {
                num_text += c_int::from((*sh).text[0] != 0);
                signs.push(SignItem { sh, id: start.id });
            }
        }
    }

    // Step out of overlap zone and continue with sign filter.
    rs_marktree_itr_step_out_filter(tree, itr_ptr, SIGN_FILTER.as_ptr());

    // Collect signs that start on this row.
    loop {
        let mark = rs_marktree_itr_current(itr_ptr);
        if mark.pos.row < 0 {
            break;
        }
        if mark.pos.row != row {
            break;
        }
        if mark.flags & MT_FLAG_INVALID == 0
            && mark.flags & MT_FLAG_END == 0
            && (mark.flags & (MT_FLAG_DECOR_SIGNTEXT | MT_FLAG_DECOR_SIGNHL)) != 0
            && nvim_ns_in_win(mark.ns, wp) != 0
        {
            let ext = mark.flags & MT_FLAG_DECOR_EXT != 0;
            let decor = crate::types::DecorInline {
                ext,
                _pad: [0u8; 7],
                data: if ext {
                    crate::types::DecorInlineData {
                        ext: std::mem::ManuallyDrop::new(crate::types::DecorExt {
                            sh_idx: mark.decor_data.ext.sh_idx,
                            _pad: 0,
                            vt: mark.decor_data.ext.vt,
                        }),
                    }
                } else {
                    crate::types::DecorInlineData {
                        hl: std::mem::ManuallyDrop::new(*mark.decor_data.hl),
                    }
                },
            };
            let sh = decor_find_sign(decor);
            if !sh.is_null() {
                num_text += c_int::from((*sh).text[0] != 0);
                signs.push(SignItem { sh, id: mark.id });
            }
        }
        if !rs_marktree_itr_next_filter(tree, itr_ptr, row + 1, 0, SIGN_FILTER.as_ptr()) {
            break;
        }
    }

    if signs.is_empty() {
        return;
    }

    // Sort signs by priority/id (same order as C qsort with sign_item_cmp).
    signs.sort_by(|a, b| {
        let sh1 = &*a.sh;
        let sh2 = &*b.sh;
        rs_sign_item_cmp(
            c_int::from(sh1.priority),
            a.id,
            sh1.sign_add_id as u32,
            c_int::from(sh2.priority),
            b.id,
            sh2.sign_add_id as u32,
        )
        .cmp(&0)
    });

    let width = if nvim_win_get_minscwidth(wp) == SCL_NUM {
        1
    } else {
        nvim_win_get_scwidth(wp)
    };
    let len = width.min(num_text) as usize;
    let mut idx: usize = 0;

    for item in &signs {
        let sh = &*item.sh;
        if !sattrs.is_null() && idx < len && sh.text[0] != 0 {
            let out = &mut *sattrs.add(idx);
            out.text[0] = sh.text[0];
            out.text[1] = sh.text[1];
            out.hl_id = sh.hl_id;
            idx += 1;
        }
        if !num_id.is_null() && *num_id <= 0 {
            *num_id = sh.number_hl_id;
        }
        if !line_id.is_null() && *line_id <= 0 {
            *line_id = sh.line_hl_id;
        }
        if !cul_id.is_null() && *cul_id <= 0 {
            *cul_id = sh.cursorline_hl_id;
        }
    }
}

/// Internal implementation of buf_signcols_count_range, shared by the export and
/// by the existing Rust callers (rs_buf_put_decor_sh, rs_buf_remove_decor_sh).
///
/// # Safety
/// `buf` must be valid.
unsafe fn buf_signcols_count_range_impl(
    buf: BufHandle,
    row1: c_int,
    row2: c_int,
    add: c_int,
    clear: c_int,
) {
    if !nvim_buf_signcols_get_autom(buf)
        || row2 < row1
        || nvim_buf_meta_total(buf, K_MT_META_SIGN_TEXT) == 0
    {
        return;
    }

    let n = (row2 + 1 - row1) as usize;
    let mut count: Vec<c_int> = vec![0; n];

    let tree = nvim_buf_get_marktree(buf);
    let mut itr_buf = crate::types::MarkTreeIter::new();
    let itr_ptr = std::ptr::addr_of_mut!(itr_buf).cast::<c_void>();

    // Count signs that start before row1 but overlap the range.
    rs_marktree_itr_get_overlap(tree, row1, 0, itr_ptr);
    let mut pair = MTPair::default();
    while rs_marktree_itr_step_overlap(tree, itr_ptr, std::ptr::addr_of_mut!(pair)) {
        let start = pair.start;
        if (start.flags & MT_FLAG_DECOR_SIGNTEXT) != 0 && (start.flags & MT_FLAG_INVALID) == 0 {
            let end_row = pair.end_pos.row;
            let stop = row2.min(end_row);
            let lo = 0usize;
            let hi = (stop - row1) as usize;
            for c in &mut count[lo..=hi] {
                *c += 1;
            }
        }
    }

    rs_marktree_itr_step_out_filter(tree, itr_ptr, SIGNTEXT_FILTER.as_ptr());

    // Count signs that start within the range.
    loop {
        let mark = rs_marktree_itr_current(itr_ptr);
        if mark.pos.row < 0 || mark.pos.row > row2 {
            break;
        }
        if (mark.flags & MT_FLAG_DECOR_SIGNTEXT) != 0
            && (mark.flags & MT_FLAG_INVALID) == 0
            && (mark.flags & MT_FLAG_END) == 0
        {
            let end_pos = rs_marktree_get_altpos(tree, mark, std::ptr::null_mut());
            let stop = row2.min(end_pos.row);
            let lo = (mark.pos.row - row1) as usize;
            let hi = (stop - row1) as usize;
            for c in &mut count[lo..=hi] {
                *c += 1;
            }
        }
        if !rs_marktree_itr_next_filter(tree, itr_ptr, row2 + 1, 0, SIGNTEXT_FILTER.as_ptr()) {
            break;
        }
    }

    // Update b_signcols.count[] based on the counts.
    for c in count {
        let prevwidth = (SIGN_SHOW_MAX as c_int).min(c - add);
        if clear != KNONE && prevwidth > 0 {
            let idx = prevwidth - 1;
            let cur = nvim_buf_signcols_get_count_at(buf, idx);
            nvim_buf_signcols_set_count_at(buf, idx, cur - 1);
            // In non-RELDEBUG builds: assert(buf->b_signcols.count[prevwidth-1] >= 0)
            // We skip the assertion here as it's a debug-only check in C.
        }
        let width = (SIGN_SHOW_MAX as c_int).min(c);
        if clear != KTRUE && width > 0 {
            let idx = width - 1;
            let cur = nvim_buf_signcols_get_count_at(buf, idx);
            nvim_buf_signcols_set_count_at(buf, idx, cur + 1);
            if width > nvim_buf_signcols_get_max(buf) {
                nvim_buf_signcols_set_max(buf, width);
            }
        }
    }
}

/// Count the number of signs in a range after adding/removing a sign.
///
/// Rust implementation of `buf_signcols_count_range()`.
///
/// @param add    1, -1 or 0 for an added, deleted or initialized range.
/// @param clear  0 (kFalse), 1 (kTrue) or -1 (kNone) for added/deleted, cleared, or initialized range.
///
/// # Safety
/// `buf` must be valid.
#[unsafe(export_name = "buf_signcols_count_range")]
pub unsafe extern "C" fn rs_buf_signcols_count_range(
    buf: BufHandle,
    row1: c_int,
    row2: c_int,
    add: c_int,
    clear: c_int,
) {
    buf_signcols_count_range_impl(buf, row1, row2, add, clear);
}

// =============================================================================
// Phase 4: bufhl_add_hl_pos_offset migrated from C
// =============================================================================

/// Add highlighting to a buffer, bounded by two cursor positions, with an offset.
///
/// Rust implementation of `bufhl_add_hl_pos_offset()`.
///
/// # Safety
/// `buf` must be a valid buf_T pointer.
#[no_mangle]
pub unsafe extern "C" fn rs_bufhl_add_hl_pos_offset(
    buf: BufHandle,
    src_id: c_int,
    hl_id: c_int,
    pos_start_lnum: c_int,
    pos_start_col: c_int,
    pos_end_lnum: c_int,
    pos_end_col: c_int,
    offset: c_int,
) {
    for lnum in pos_start_lnum..=pos_end_lnum {
        let (hl_start, hl_end, end_off) = if pos_start_lnum < lnum && lnum < pos_end_lnum {
            (c_int::max(offset - 1, 0), 0, 1)
        } else if lnum == pos_start_lnum && lnum < pos_end_lnum {
            (pos_start_col + offset, 0, 1)
        } else if pos_start_lnum < lnum && lnum == pos_end_lnum {
            (c_int::max(offset - 1, 0), pos_end_col + offset, 0)
        } else {
            // pos_start_lnum == lnum == pos_end_lnum
            (pos_start_col + offset, pos_end_col + offset, 0)
        };
        nvim_extmark_set_hl(
            buf,
            src_id,
            lnum - 1,
            hl_start,
            lnum - 1 + end_off,
            hl_end,
            hl_id,
        );
    }
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
