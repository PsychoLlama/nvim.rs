//! Core decoration functions migrated from decoration.c
//!
//! This module contains Rust implementations of decoration-related functions
//! from `src/nvim/decoration.c`.

use std::ffi::{c_int, c_void};

use crate::{
    DecorKind, DecorRangeHandle, DecorStateHandle, DecorVirtTextHandle, VirtTextPos,
    DRAW_COL_DISABLED, DRAW_COL_JUST_ADDED, DRAW_COL_PENDING, DRAW_COL_UNSET, KVT_HIDE,
    KVT_IS_LINES,
};

// =============================================================================
// Constants
// =============================================================================

/// Invalid decoration ID.
pub const DECOR_ID_INVALID: u32 = u32::MAX;

/// Base priority for decorations.
pub const DECOR_PRIORITY_BASE: u16 = 0x1000;

/// Sign width in display cells.
pub const SIGN_WIDTH: usize = 2;

// =============================================================================
// DecorSignHighlight Flags
// =============================================================================

/// Flags for DecorSignHighlight.
#[repr(u16)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DecorShFlag {
    /// Is a sign decoration.
    IsSign = 1,
    /// Highlight extends to end of line.
    HlEol = 2,
    /// UI watched decoration.
    UIWatched = 4,
    /// UI watched overlay decoration.
    UIWatchedOverlay = 8,
    /// Spelling on.
    SpellOn = 16,
    /// Spelling off.
    SpellOff = 32,
    /// Conceal decoration.
    Conceal = 64,
    /// Conceal lines decoration.
    ConcealLines = 128,
}

impl DecorShFlag {
    /// Get flag value as u16.
    #[must_use]
    pub const fn value(self) -> u16 {
        self as u16
    }
}

// C constants for flags
pub const KSH_IS_SIGN: u16 = 1;
pub const KSH_HL_EOL: u16 = 2;
pub const KSH_UI_WATCHED: u16 = 4;
pub const KSH_UI_WATCHED_OVERLAY: u16 = 8;
pub const KSH_SPELL_ON: u16 = 16;
pub const KSH_SPELL_OFF: u16 = 32;
pub const KSH_CONCEAL: u16 = 64;
pub const KSH_CONCEAL_LINES: u16 = 128;

// =============================================================================
// Extmark Type Flags
// =============================================================================

/// Extmark type flags.
#[repr(u16)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExtmarkType {
    /// No special type.
    None = 0,
    /// Has highlight.
    Highlight = 1,
    /// Has sign.
    Sign = 2,
    /// Has virtual text.
    VirtText = 4,
    /// Has virtual lines.
    VirtLines = 8,
}

impl ExtmarkType {
    /// Get type value as u16.
    #[must_use]
    pub const fn value(self) -> u16 {
        self as u16
    }
}

// C constants for extmark types
pub const KEXTMARK_NONE: u16 = 0;
pub const KEXTMARK_HIGHLIGHT: u16 = 1;
pub const KEXTMARK_SIGN: u16 = 2;
pub const KEXTMARK_VIRT_TEXT: u16 = 4;
pub const KEXTMARK_VIRT_LINES: u16 = 8;

// =============================================================================
// External C Functions
// =============================================================================

extern "C" {
    // DecorRange accessors
    fn nvim_decor_range_get_kind(range: DecorRangeHandle) -> c_int;
    fn nvim_decor_range_get_draw_col(range: DecorRangeHandle) -> c_int;
    fn nvim_decor_range_set_draw_col(range: DecorRangeHandle, val: c_int);
    fn nvim_decor_range_get_start_row(range: DecorRangeHandle) -> c_int;
    fn nvim_decor_range_get_start_col(range: DecorRangeHandle) -> c_int;
    fn nvim_decor_range_get_virt_text(range: DecorRangeHandle) -> DecorVirtTextHandle;
    fn nvim_decor_range_get_virt_pos_kind(range: DecorRangeHandle) -> c_int;

    // DecorVirtText accessors
    fn nvim_decor_virt_text_get_flags(vt: DecorVirtTextHandle) -> c_int;
    fn nvim_decor_virt_text_get_pos(vt: DecorVirtTextHandle) -> c_int;
    fn nvim_decor_virt_text_get_width(vt: DecorVirtTextHandle) -> c_int;

    // DecorState accessors
    fn nvim_decor_state_get_row(state: DecorStateHandle) -> c_int;
    fn nvim_decor_state_get_current_end(state: DecorStateHandle) -> c_int;
    fn nvim_decor_state_get_range(state: DecorStateHandle, idx: c_int) -> DecorRangeHandle;

    // Highlight functions
    fn syn_id2attr(hl_id: c_int) -> c_int;

    // Phase 2: Memory management accessors
    fn nvim_get_decor_freelist() -> u32;
    fn nvim_set_decor_freelist(val: u32);
    fn nvim_decor_items_size() -> u32;
    fn nvim_decor_items_get_next(idx: u32) -> u32;
    fn nvim_decor_items_set(idx: u32, item: DecorSignHighlight);
    fn nvim_decor_items_push(item: DecorSignHighlight) -> u32;
    fn nvim_xmalloc_decor_virt_text() -> *mut c_void;
    fn nvim_xfree_ptr(ptr: *mut c_void);
    fn nvim_get_to_free_virt() -> *mut c_void;
    fn nvim_set_to_free_virt(val: *mut c_void);
    fn nvim_get_to_free_sh() -> u32;
    fn nvim_set_to_free_sh(val: u32);
    fn nvim_decor_state_get_running_provider() -> c_int;
    fn nvim_decor_vt_get_next(vt: *mut c_void) -> *mut c_void;
    fn nvim_decor_vt_set_next(vt: *mut c_void, next: *mut c_void);
    fn nvim_decor_vt_get_flags(vt: *mut c_void) -> u8;
    fn nvim_clear_virttext(vt: *mut c_void);
    fn nvim_clear_virtlines(vt: *mut c_void);
    fn nvim_decor_vt_get_virt_text_data(vt: *mut c_void) -> *mut c_void;
    fn nvim_decor_vt_get_virt_lines_data(vt: *mut c_void) -> *mut c_void;
    fn nvim_decor_items_get_flags(idx: u32) -> u16;
    fn nvim_decor_items_set_flags(idx: u32, flags: u16);
    fn nvim_decor_items_set_next(idx: u32, next: u32);
    fn nvim_decor_items_clear_sign_name(idx: u32);
    fn nvim_decor_items_clear_url(idx: u32);
    fn nvim_decor_vt_copy_to(dst: *mut c_void, src: *const c_void, size: usize);
    fn nvim_decor_state_set_win(state: DecorStateHandle, win: *mut c_void);
}

// =============================================================================
// DecorSignHighlight Conversion
// =============================================================================

/// Data for inline highlight decoration.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct DecorHighlightInline {
    /// Flags.
    pub flags: u16,
    /// Priority.
    pub priority: u16,
    /// Highlight ID.
    pub hl_id: c_int,
    /// Conceal character.
    pub conceal_char: u32,
}

/// Data for sign/highlight decoration.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct DecorSignHighlight {
    /// Flags.
    pub flags: u16,
    /// Priority.
    pub priority: u16,
    /// Highlight ID.
    pub hl_id: c_int,
    /// Text (sign or conceal).
    pub text: [u32; SIGN_WIDTH],
    /// Sign name (null if not set).
    pub sign_name: *mut std::ffi::c_char,
    /// Sign add ID.
    pub sign_add_id: c_int,
    /// Number highlight ID.
    pub number_hl_id: c_int,
    /// Line highlight ID.
    pub line_hl_id: c_int,
    /// Cursor line highlight ID.
    pub cursorline_hl_id: c_int,
    /// Next decoration index.
    pub next: u32,
    /// URL for hyperlink.
    pub url: *const std::ffi::c_char,
}

impl Default for DecorSignHighlight {
    fn default() -> Self {
        Self {
            flags: 0,
            priority: DECOR_PRIORITY_BASE,
            hl_id: 0,
            text: [0, 0],
            sign_name: std::ptr::null_mut(),
            sign_add_id: 0,
            number_hl_id: 0,
            line_hl_id: 0,
            cursorline_hl_id: 0,
            next: DECOR_ID_INVALID,
            url: std::ptr::null(),
        }
    }
}

/// Convert DecorHighlightInline to DecorSignHighlight.
/// Rust implementation of decor_sh_from_inline().
#[must_use]
pub fn decor_sh_from_inline(item: DecorHighlightInline) -> DecorSignHighlight {
    debug_assert!(
        item.flags & KSH_IS_SIGN == 0,
        "sign decorations should not be inline"
    );

    DecorSignHighlight {
        flags: item.flags,
        priority: item.priority,
        text: [item.conceal_char, 0],
        hl_id: item.hl_id,
        number_hl_id: 0,
        line_hl_id: 0,
        cursorline_hl_id: 0,
        next: DECOR_ID_INVALID,
        sign_name: std::ptr::null_mut(),
        sign_add_id: 0,
        url: std::ptr::null(),
    }
}

/// FFI: Convert DecorHighlightInline to DecorSignHighlight.
#[no_mangle]
pub extern "C" fn rs_decor_sh_from_inline(
    flags: u16,
    priority: u16,
    hl_id: c_int,
    conceal_char: u32,
) -> DecorSignHighlight {
    decor_sh_from_inline(DecorHighlightInline {
        flags,
        priority,
        hl_id,
        conceal_char,
    })
}

// =============================================================================
// Decor Type Flags
// =============================================================================

/// Get type flags from decoration.
/// Rust implementation of decor_type_flags().
///
/// # Arguments
/// * `ext` - Whether this is an extended decoration
/// * `vt_flags` - Virtual text flags (kVTIsLines, etc.)
/// * `sh_flags` - Sign/highlight flags (kSHIsSign, etc.)
/// * `has_vt` - Whether there is virtual text
/// * `has_sh` - Whether there is sign/highlight
#[must_use]
pub const fn decor_type_flags(
    ext: bool,
    vt_flags: u8,
    sh_flags: u16,
    has_vt: bool,
    has_sh: bool,
) -> u16 {
    if ext {
        let mut type_flags: u16 = KEXTMARK_NONE;

        if has_vt {
            if vt_flags & KVT_IS_LINES != 0 {
                type_flags |= KEXTMARK_VIRT_LINES;
            } else {
                type_flags |= KEXTMARK_VIRT_TEXT;
            }
        }

        if has_sh {
            if sh_flags & KSH_IS_SIGN != 0 {
                type_flags |= KEXTMARK_SIGN;
            } else {
                type_flags |= KEXTMARK_HIGHLIGHT;
            }
        }

        type_flags
    } else {
        // Inline decoration
        if sh_flags & KSH_IS_SIGN != 0 {
            KEXTMARK_SIGN
        } else {
            KEXTMARK_HIGHLIGHT
        }
    }
}

/// FFI: Get type flags from decoration.
#[no_mangle]
pub extern "C" fn rs_decor_type_flags(
    ext: c_int,
    vt_flags: u8,
    sh_flags: u16,
    has_vt: c_int,
    has_sh: c_int,
) -> u16 {
    decor_type_flags(ext != 0, vt_flags, sh_flags, has_vt != 0, has_sh != 0)
}

// =============================================================================
// Draw Column Initialization
// =============================================================================

/// Initialize the draw_col of a newly-added virtual text item.
/// Rust implementation of decor_init_draw_col().
#[must_use]
pub fn decor_init_draw_col_value(
    win_col: c_int,
    hidden: bool,
    kind: DecorKind,
    pos: VirtTextPos,
    vt_flags: c_int,
) -> c_int {
    if win_col < 0 && pos != VirtTextPos::Inline {
        win_col
    } else if pos == VirtTextPos::Overlay {
        if kind == DecorKind::VirtText && (vt_flags & c_int::from(KVT_HIDE) != 0) && hidden {
            DRAW_COL_DISABLED
        } else {
            win_col
        }
    } else {
        DRAW_COL_UNSET
    }
}

/// FFI: Initialize draw_col value for virtual text.
#[no_mangle]
pub extern "C" fn rs_decor_init_draw_col_value(
    win_col: c_int,
    hidden: c_int,
    kind: c_int,
    pos: c_int,
    vt_flags: c_int,
) -> c_int {
    let kind = DecorKind::from_c_int(kind).unwrap_or(DecorKind::Highlight);
    let pos = VirtTextPos::from_c_int(pos).unwrap_or(VirtTextPos::EndOfLine);
    decor_init_draw_col_value(win_col, hidden != 0, kind, pos, vt_flags)
}

/// Initialize draw_col for a DecorRange directly.
/// Called via FFI with range pointer.
#[no_mangle]
pub extern "C" fn rs_decor_init_draw_col(
    win_col: c_int,
    hidden: c_int,
    range: DecorRangeHandle,
) -> c_int {
    if range.is_null() {
        return DRAW_COL_UNSET;
    }

    let kind_raw = unsafe { nvim_decor_range_get_kind(range) };
    let kind = DecorKind::from_c_int(kind_raw).unwrap_or(DecorKind::Highlight);

    let pos_raw = unsafe { nvim_decor_range_get_virt_pos_kind(range) };
    let pos = VirtTextPos::from_c_int(pos_raw).unwrap_or(VirtTextPos::EndOfLine);

    let vt_flags = if kind == DecorKind::VirtText {
        let vt = unsafe { nvim_decor_range_get_virt_text(range) };
        if vt.is_null() {
            0
        } else {
            unsafe { nvim_decor_virt_text_get_flags(vt) }
        }
    } else {
        0
    };

    decor_init_draw_col_value(win_col, hidden != 0, kind, pos, vt_flags)
}

// =============================================================================
// Recheck Draw Column
// =============================================================================

/// Check if a range needs draw_col rechecked.
/// Returns new draw_col value if it was DRAW_COL_PENDING, otherwise returns current.
#[must_use]
pub fn should_recheck_draw_col(draw_col: c_int) -> bool {
    draw_col == DRAW_COL_PENDING
}

/// FFI: Check if draw_col needs rechecking.
#[no_mangle]
pub extern "C" fn rs_should_recheck_draw_col(draw_col: c_int) -> c_int {
    c_int::from(should_recheck_draw_col(draw_col))
}

// =============================================================================
// Virtual Text Width Calculation
// =============================================================================

/// Calculate EOL right-aligned virtual text total width.
/// Used by draw_virt_text for right alignment.
#[no_mangle]
pub extern "C" fn rs_calc_eol_right_width(
    state: DecorStateHandle,
    from_idx: c_int,
    row: c_int,
) -> c_int {
    if state.is_null() {
        return 0;
    }

    let current_end = unsafe { nvim_decor_state_get_current_end(state) };
    let state_row = unsafe { nvim_decor_state_get_row(state) };

    let mut total_width: c_int = 0;

    for i in from_idx..current_end {
        let range = unsafe { nvim_decor_state_get_range(state, i) };
        if range.is_null() {
            continue;
        }

        let start_row = unsafe { nvim_decor_range_get_start_row(range) };
        if start_row != state_row && start_row != row {
            continue;
        }

        let draw_col = unsafe { nvim_decor_range_get_draw_col(range) };
        if draw_col != DRAW_COL_UNSET {
            continue;
        }

        let kind_raw = unsafe { nvim_decor_range_get_kind(range) };
        let kind = DecorKind::from_c_int(kind_raw);
        if kind != Some(DecorKind::VirtText) {
            continue;
        }

        let pos_raw = unsafe { nvim_decor_range_get_virt_pos_kind(range) };
        let pos = VirtTextPos::from_c_int(pos_raw);
        if pos != Some(VirtTextPos::EndOfLineRightAlign) {
            continue;
        }

        let vt = unsafe { nvim_decor_range_get_virt_text(range) };
        if !vt.is_null() {
            let width = unsafe { nvim_decor_virt_text_get_width(vt) };
            // Add 1 for spacing between entries
            total_width += width + 1;
        }
    }

    // Remove trailing space
    if total_width > 0 {
        total_width -= 1;
    }

    total_width
}

// =============================================================================
// Range Position Checks
// =============================================================================

/// Check if a range's end position is before a given position.
#[must_use]
pub const fn range_end_before(end_row: c_int, end_col: c_int, row: c_int, col: c_int) -> bool {
    end_row < row || (end_row == row && end_col <= col)
}

/// FFI: Check if range end is before position.
#[no_mangle]
pub extern "C" fn rs_range_end_before(
    end_row: c_int,
    end_col: c_int,
    row: c_int,
    col: c_int,
) -> c_int {
    c_int::from(range_end_before(end_row, end_col, row, col))
}

/// Check if a range's start position is after a given position.
#[must_use]
pub const fn range_start_after(start_row: c_int, start_col: c_int, row: c_int, col: c_int) -> bool {
    start_row > row || (start_row == row && start_col > col)
}

/// FFI: Check if range start is after position.
#[no_mangle]
pub extern "C" fn rs_range_start_after(
    start_row: c_int,
    start_col: c_int,
    row: c_int,
    col: c_int,
) -> c_int {
    c_int::from(range_start_after(start_row, start_col, row, col))
}

// =============================================================================
// Priority Comparison
// =============================================================================

/// Compare decoration priorities for ordering.
/// Returns true if (p1, o1) < (p2, o2) where p is priority and o is ordering.
#[must_use]
pub const fn priority_cmp(
    priority1: u32,
    ordering1: c_int,
    priority2: u32,
    ordering2: c_int,
) -> c_int {
    if priority1 < priority2 {
        -1
    } else if priority1 > priority2 {
        1
    } else if ordering1 < ordering2 {
        -1
    } else if ordering1 > ordering2 {
        1
    } else {
        0
    }
}

/// FFI: Compare decoration priorities.
#[no_mangle]
pub extern "C" fn rs_priority_cmp(
    priority1: u32,
    ordering1: c_int,
    priority2: u32,
    ordering2: c_int,
) -> c_int {
    priority_cmp(priority1, ordering1, priority2, ordering2)
}

// =============================================================================
// Draw Column State Machine
// =============================================================================

/// Check if draw_col indicates the item was just added.
#[must_use]
pub const fn draw_col_is_just_added(draw_col: c_int) -> bool {
    draw_col == DRAW_COL_JUST_ADDED
}

/// FFI: Check if draw_col is "just added" state.
#[no_mangle]
pub extern "C" fn rs_draw_col_is_just_added(draw_col: c_int) -> c_int {
    c_int::from(draw_col_is_just_added(draw_col))
}

/// Check if draw_col indicates the item is disabled.
#[must_use]
pub const fn draw_col_is_disabled(draw_col: c_int) -> bool {
    draw_col == DRAW_COL_DISABLED
}

/// FFI: Check if draw_col is disabled.
#[no_mangle]
pub extern "C" fn rs_draw_col_is_disabled(draw_col: c_int) -> c_int {
    c_int::from(draw_col_is_disabled(draw_col))
}

/// Check if draw_col indicates the item is pending.
#[must_use]
pub const fn draw_col_is_pending(draw_col: c_int) -> bool {
    draw_col == DRAW_COL_PENDING
}

/// FFI: Check if draw_col is pending.
#[no_mangle]
pub extern "C" fn rs_draw_col_is_pending(draw_col: c_int) -> c_int {
    c_int::from(draw_col_is_pending(draw_col))
}

/// Check if draw_col indicates the item is unset.
#[must_use]
pub const fn draw_col_is_unset(draw_col: c_int) -> bool {
    draw_col == DRAW_COL_UNSET
}

/// FFI: Check if draw_col is unset.
#[no_mangle]
pub extern "C" fn rs_draw_col_is_unset(draw_col: c_int) -> c_int {
    c_int::from(draw_col_is_unset(draw_col))
}

/// Check if draw_col has a valid column value (>= 0).
#[must_use]
pub const fn draw_col_is_valid(draw_col: c_int) -> bool {
    draw_col >= 0
}

/// FFI: Check if draw_col is valid.
#[no_mangle]
pub extern "C" fn rs_draw_col_is_valid(draw_col: c_int) -> c_int {
    c_int::from(draw_col_is_valid(draw_col))
}

// =============================================================================
// Virtual Text Position Classification
// =============================================================================

/// Check if position is an end-of-line variant.
#[must_use]
pub const fn virt_pos_is_eol(pos: VirtTextPos) -> bool {
    matches!(
        pos,
        VirtTextPos::EndOfLine | VirtTextPos::EndOfLineRightAlign
    )
}

/// FFI: Check if position is EOL variant.
#[no_mangle]
pub extern "C" fn rs_virt_pos_is_eol(pos: c_int) -> c_int {
    VirtTextPos::from_c_int(pos).map_or(0, |p| c_int::from(virt_pos_is_eol(p)))
}

/// Check if position needs screen column.
#[must_use]
pub const fn virt_pos_needs_col(pos: VirtTextPos) -> bool {
    matches!(pos, VirtTextPos::Inline | VirtTextPos::Overlay)
}

/// FFI: Check if position needs column.
#[no_mangle]
pub extern "C" fn rs_virt_pos_needs_col(pos: c_int) -> c_int {
    VirtTextPos::from_c_int(pos).map_or(0, |p| c_int::from(virt_pos_needs_col(p)))
}

/// Check if position can be drawn off-screen.
#[must_use]
pub const fn virt_pos_offscreen_ok(pos: VirtTextPos) -> bool {
    matches!(
        pos,
        VirtTextPos::EndOfLine
            | VirtTextPos::EndOfLineRightAlign
            | VirtTextPos::RightAlign
            | VirtTextPos::WinCol
    )
}

/// FFI: Check if position can be drawn off-screen.
#[no_mangle]
pub extern "C" fn rs_virt_pos_offscreen_ok(pos: c_int) -> c_int {
    VirtTextPos::from_c_int(pos).map_or(0, |p| c_int::from(virt_pos_offscreen_ok(p)))
}

// =============================================================================
// Highlight Mode Operations
// =============================================================================

use crate::HlMode;

/// Check if highlight mode replaces existing attributes.
#[must_use]
pub const fn hl_mode_replaces(mode: HlMode) -> bool {
    matches!(mode, HlMode::Replace)
}

/// FFI: Check if highlight mode replaces.
#[no_mangle]
pub extern "C" fn rs_hl_mode_replaces(mode: c_int) -> c_int {
    HlMode::from_c_int(mode).map_or(0, |m| c_int::from(hl_mode_replaces(m)))
}

/// Check if highlight mode combines with existing attributes.
#[must_use]
pub const fn hl_mode_combines(mode: HlMode) -> bool {
    matches!(mode, HlMode::Combine | HlMode::Unknown)
}

/// FFI: Check if highlight mode combines.
#[no_mangle]
pub extern "C" fn rs_hl_mode_combines(mode: c_int) -> c_int {
    HlMode::from_c_int(mode).map_or(1, |m| c_int::from(hl_mode_combines(m))) // Unknown defaults to combine
}

/// Check if highlight mode blends.
#[must_use]
pub const fn hl_mode_blends(mode: HlMode) -> bool {
    matches!(mode, HlMode::Blend)
}

/// FFI: Check if highlight mode blends.
#[no_mangle]
pub extern "C" fn rs_hl_mode_blends(mode: c_int) -> c_int {
    HlMode::from_c_int(mode).map_or(0, |m| c_int::from(hl_mode_blends(m)))
}

// =============================================================================
// Attribute Computation
// =============================================================================

/// Compute attribute for virtual text chunk.
/// Returns (attr, should_use_hl_id) tuple.
#[must_use]
pub fn compute_virt_text_attr(hl_id: c_int, base_attr: c_int, _mode: HlMode) -> (c_int, bool) {
    use std::cmp::Ordering;
    match hl_id.cmp(&0) {
        Ordering::Less => {
            // Negative hl_id means don't change attr
            (base_attr, false)
        }
        Ordering::Equal => {
            // Zero hl_id means use max of attr
            if base_attr < 0 {
                (0, false)
            } else {
                (base_attr, false)
            }
        }
        Ordering::Greater => {
            // Positive hl_id means apply this highlight
            (hl_id, true)
        }
    }
}

/// FFI: Compute virtual text attribute.
/// Returns the attribute value. Sets `use_hl_id` to 1 if hl_id should be used.
///
/// # Safety
/// `use_hl_id` must be valid or null.
#[no_mangle]
pub unsafe extern "C" fn rs_compute_virt_text_attr(
    hl_id: c_int,
    base_attr: c_int,
    mode: c_int,
    use_hl_id: *mut c_int,
) -> c_int {
    let hl_mode = HlMode::from_c_int(mode).unwrap_or(HlMode::Unknown);
    let (attr, should_use) = compute_virt_text_attr(hl_id, base_attr, hl_mode);
    if !use_hl_id.is_null() {
        *use_hl_id = c_int::from(should_use);
    }
    attr
}

// =============================================================================
// Decoration Kind Checks
// =============================================================================

/// Check if decoration kind is a virtual text variant.
#[must_use]
pub const fn decor_kind_is_virt(kind: DecorKind) -> bool {
    matches!(kind, DecorKind::VirtText | DecorKind::VirtLines)
}

/// FFI: Check if decoration kind is virtual text.
#[no_mangle]
pub extern "C" fn rs_decor_kind_is_virt(kind: c_int) -> c_int {
    DecorKind::from_c_int(kind).map_or(0, |k| c_int::from(decor_kind_is_virt(k)))
}

/// Check if decoration kind is sign.
#[must_use]
pub const fn decor_kind_is_sign(kind: DecorKind) -> bool {
    matches!(kind, DecorKind::Sign)
}

/// FFI: Check if decoration kind is sign.
#[no_mangle]
pub extern "C" fn rs_decor_kind_is_sign(kind: c_int) -> c_int {
    DecorKind::from_c_int(kind).map_or(0, |k| c_int::from(decor_kind_is_sign(k)))
}

/// Check if decoration kind is highlight.
#[must_use]
pub const fn decor_kind_is_highlight(kind: DecorKind) -> bool {
    matches!(kind, DecorKind::Highlight)
}

/// FFI: Check if decoration kind is highlight.
#[no_mangle]
pub extern "C" fn rs_decor_kind_is_highlight(kind: c_int) -> c_int {
    DecorKind::from_c_int(kind).map_or(0, |k| c_int::from(decor_kind_is_highlight(k)))
}

/// Check if decoration kind is UI watched.
#[must_use]
pub const fn decor_kind_is_ui_watched(kind: DecorKind) -> bool {
    matches!(kind, DecorKind::UIWatched)
}

/// FFI: Check if decoration kind is UI watched.
#[no_mangle]
pub extern "C" fn rs_decor_kind_is_ui_watched(kind: c_int) -> c_int {
    DecorKind::from_c_int(kind).map_or(0, |k| c_int::from(decor_kind_is_ui_watched(k)))
}

// =============================================================================
// Sign/Highlight Flag Checks
// =============================================================================

/// Check if flags indicate a sign.
#[must_use]
pub const fn sh_is_sign(flags: u16) -> bool {
    flags & KSH_IS_SIGN != 0
}

/// FFI: Check if flags indicate sign.
#[no_mangle]
pub extern "C" fn rs_sh_is_sign(flags: u16) -> c_int {
    c_int::from(sh_is_sign(flags))
}

/// Check if flags indicate highlight EOL.
#[must_use]
pub const fn sh_is_hl_eol(flags: u16) -> bool {
    flags & KSH_HL_EOL != 0
}

/// FFI: Check if flags indicate hl_eol.
#[no_mangle]
pub extern "C" fn rs_sh_is_hl_eol(flags: u16) -> c_int {
    c_int::from(sh_is_hl_eol(flags))
}

/// Check if flags indicate UI watched.
#[must_use]
pub const fn sh_is_ui_watched(flags: u16) -> bool {
    flags & KSH_UI_WATCHED != 0
}

/// FFI: Check if flags indicate UI watched.
#[no_mangle]
pub extern "C" fn rs_sh_is_ui_watched(flags: u16) -> c_int {
    c_int::from(sh_is_ui_watched(flags))
}

/// Check if flags indicate conceal.
#[must_use]
pub const fn sh_is_conceal(flags: u16) -> bool {
    flags & KSH_CONCEAL != 0
}

/// FFI: Check if flags indicate conceal.
#[no_mangle]
pub extern "C" fn rs_sh_is_conceal(flags: u16) -> c_int {
    c_int::from(sh_is_conceal(flags))
}

/// Check if flags indicate spell on.
#[must_use]
pub const fn sh_is_spell_on(flags: u16) -> bool {
    flags & KSH_SPELL_ON != 0
}

/// FFI: Check if flags indicate spell on.
#[no_mangle]
pub extern "C" fn rs_sh_is_spell_on(flags: u16) -> c_int {
    c_int::from(sh_is_spell_on(flags))
}

/// Check if flags indicate spell off.
#[must_use]
pub const fn sh_is_spell_off(flags: u16) -> bool {
    flags & KSH_SPELL_OFF != 0
}

/// FFI: Check if flags indicate spell off.
#[no_mangle]
pub extern "C" fn rs_sh_is_spell_off(flags: u16) -> c_int {
    c_int::from(sh_is_spell_off(flags))
}

/// Check if flags indicate conceal lines.
#[must_use]
pub const fn sh_is_conceal_lines(flags: u16) -> bool {
    flags & KSH_CONCEAL_LINES != 0
}

/// FFI: Check if flags indicate conceal lines.
#[no_mangle]
pub extern "C" fn rs_sh_is_conceal_lines(flags: u16) -> c_int {
    c_int::from(sh_is_conceal_lines(flags))
}

// =============================================================================
// Virtual Text Flag Checks
// =============================================================================

/// Check if virtual text flags indicate lines.
#[must_use]
pub const fn vt_is_lines(flags: u8) -> bool {
    flags & KVT_IS_LINES != 0
}

/// FFI: Check if VT flags indicate lines.
#[no_mangle]
pub extern "C" fn rs_vt_is_lines(flags: u8) -> c_int {
    c_int::from(vt_is_lines(flags))
}

/// Check if virtual text flags indicate hide.
#[must_use]
pub const fn vt_is_hide(flags: u8) -> bool {
    flags & KVT_HIDE != 0
}

/// FFI: Check if VT flags indicate hide.
#[no_mangle]
pub extern "C" fn rs_vt_is_hide(flags: u8) -> c_int {
    c_int::from(vt_is_hide(flags))
}

/// Check if virtual text flags indicate lines above.
#[must_use]
pub const fn vt_is_lines_above(flags: u8) -> bool {
    flags & crate::KVT_LINES_ABOVE != 0
}

/// FFI: Check if VT flags indicate lines above.
#[no_mangle]
pub extern "C" fn rs_vt_is_lines_above(flags: u8) -> c_int {
    c_int::from(vt_is_lines_above(flags))
}

/// Check if virtual text should repeat at linebreak.
#[must_use]
pub const fn vt_repeat_linebreak(flags: u8) -> bool {
    flags & crate::KVT_REPEAT_LINEBREAK != 0
}

/// FFI: Check if VT repeats at linebreak.
#[no_mangle]
pub extern "C" fn rs_vt_repeat_linebreak(flags: u8) -> c_int {
    c_int::from(vt_repeat_linebreak(flags))
}

// =============================================================================
// Phase 2: Memory Management
// =============================================================================

/// Allocate a DecorSignHighlight from the freelist or push to decor_items.
///
/// Rust implementation of `decor_put_sh()`.
#[no_mangle]
pub extern "C" fn rs_decor_put_sh(item: DecorSignHighlight) -> u32 {
    unsafe {
        let freelist = nvim_get_decor_freelist();
        if freelist == DECOR_ID_INVALID {
            nvim_decor_items_push(item)
        } else {
            let pos = freelist;
            let next = nvim_decor_items_get_next(freelist);
            nvim_set_decor_freelist(next);
            nvim_decor_items_set(pos, item);
            pos
        }
    }
}

/// Allocate a DecorVirtText on the heap, copy data, set next pointer.
///
/// Rust implementation of `decor_put_vt()`.
///
/// # Safety
/// `vt_data` must point to a valid `DecorVirtText` of `vt_size` bytes.
#[no_mangle]
pub unsafe extern "C" fn rs_decor_put_vt(
    vt_data: *const c_void,
    vt_size: usize,
    next: *mut c_void,
) -> *mut c_void {
    let alloc = nvim_xmalloc_decor_virt_text();
    nvim_decor_vt_copy_to(alloc, vt_data, vt_size);
    nvim_decor_vt_set_next(alloc, next);
    alloc
}

/// Clear a VirtText (free chunk texts + destroy kvec).
///
/// Rust implementation of `clear_virttext()`.
/// Delegates to the C helper which handles kvec internals.
///
/// # Safety
/// `text` must point to a valid `VirtText` or be null.
#[no_mangle]
pub unsafe extern "C" fn rs_clear_virttext(text: *mut c_void) {
    if text.is_null() {
        return;
    }
    nvim_clear_virttext(text);
}

/// Clear VirtLines (free all lines + destroy kvec).
///
/// Rust implementation of `clear_virtlines()`.
/// Delegates to the C helper which handles kvec internals.
///
/// # Safety
/// `lines` must point to a valid `VirtLines` or be null.
#[no_mangle]
pub unsafe extern "C" fn rs_clear_virtlines(lines: *mut c_void) {
    if lines.is_null() {
        return;
    }
    nvim_clear_virtlines(lines);
}

/// Free decoration inner data (virt text linked list + sign/highlight freelist).
///
/// Rust implementation of `decor_free_inner()`.
///
/// # Safety
/// `vt` must point to a valid `DecorVirtText` linked list or be null.
#[no_mangle]
pub unsafe extern "C" fn rs_decor_free_inner(mut vt: *mut c_void, first_idx: u32) {
    // Walk and free DecorVirtText linked list
    while !vt.is_null() {
        let flags = nvim_decor_vt_get_flags(vt);
        if flags & KVT_IS_LINES != 0 {
            let lines_data = nvim_decor_vt_get_virt_lines_data(vt);
            nvim_clear_virtlines(lines_data);
        } else {
            let text_data = nvim_decor_vt_get_virt_text_data(vt);
            nvim_clear_virttext(text_data);
        }
        let next = nvim_decor_vt_get_next(vt);
        nvim_xfree_ptr(vt);
        vt = next;
    }

    // Walk and free DecorSignHighlight linked list, returning to freelist
    let mut idx = first_idx;
    while idx != DECOR_ID_INVALID {
        let flags = nvim_decor_items_get_flags(idx);
        if flags & KSH_IS_SIGN != 0 {
            nvim_decor_items_clear_sign_name(idx);
        }
        nvim_decor_items_set_flags(idx, 0);
        // Check and clear url
        nvim_decor_items_clear_url(idx);

        let next = nvim_decor_items_get_next(idx);
        if next == DECOR_ID_INVALID {
            // Last in chain: link to freelist
            nvim_decor_items_set_next(idx, nvim_get_decor_freelist());
            nvim_set_decor_freelist(first_idx);
            break;
        }
        idx = next;
    }
}

/// Free decoration, possibly deferring if in a decoration provider callback.
///
/// Rust implementation of `decor_free()`.
///
/// # Safety
/// `vt_ptr` must point to a valid `DecorVirtText` linked list or be null.
#[no_mangle]
pub unsafe extern "C" fn rs_decor_free(ext: c_int, vt_ptr: *mut c_void, sh_idx: u32) {
    if ext == 0 {
        return;
    }

    if nvim_decor_state_get_running_provider() != 0 {
        // Defer deletion: append to to_free linked lists
        let mut vt = vt_ptr;
        let original_vt = vt_ptr;
        while !vt.is_null() {
            let next = nvim_decor_vt_get_next(vt);
            if next.is_null() {
                let to_free = nvim_get_to_free_virt();
                nvim_decor_vt_set_next(vt, to_free);
                nvim_set_to_free_virt(original_vt);
                break;
            }
            vt = next;
        }

        let mut idx = sh_idx;
        let original_idx = sh_idx;
        while idx != DECOR_ID_INVALID {
            let next = nvim_decor_items_get_next(idx);
            if next == DECOR_ID_INVALID {
                let to_free = nvim_get_to_free_sh();
                nvim_decor_items_set_next(idx, to_free);
                nvim_set_to_free_sh(original_idx);
                break;
            }
            idx = next;
        }
    } else {
        // Safe to delete right now
        rs_decor_free_inner(vt_ptr, sh_idx);
    }
}

/// Process deferred decoration deletions.
///
/// Rust implementation of `decor_check_to_be_deleted()`.
#[no_mangle]
pub extern "C" fn rs_decor_check_to_be_deleted() {
    // Safety: accessing globals through C accessors
    unsafe {
        let to_free_virt = nvim_get_to_free_virt();
        let to_free_sh = nvim_get_to_free_sh();
        rs_decor_free_inner(to_free_virt, to_free_sh);
    }
    unsafe { nvim_set_to_free_virt(std::ptr::null_mut()) };
    unsafe { nvim_set_to_free_sh(DECOR_ID_INVALID) };
    // Also clear the win pointer (original C code does this)
    let state = crate::get_decor_state();
    unsafe { nvim_decor_state_set_win(state, std::ptr::null_mut()) };
}

// =============================================================================
// Phase 7: Marktree Scanning Functions
// =============================================================================

/// Find the first sign decoration in a linked list.
///
/// Walks the decor_items linked list starting from `sh_idx`, returning
/// a pointer to the first DecorSignHighlight with the kSHIsSign flag.
///
/// Rust implementation of `decor_find_sign()`.
#[no_mangle]
pub unsafe extern "C" fn rs_decor_find_sign(ext: bool, sh_idx: u32) -> *mut c_void {
    if !ext {
        return std::ptr::null_mut();
    }
    let mut decor_id = sh_idx;
    loop {
        if decor_id == DECOR_ID_INVALID {
            return std::ptr::null_mut();
        }
        let flags = nvim_decor_items_get_flags(decor_id);
        if flags & KSH_IS_SIGN != 0 {
            // Return pointer to the item in decor_items
            return nvim_decor_items_get_ptr(decor_id);
        }
        decor_id = nvim_decor_items_get_next(decor_id);
    }
}

extern "C" {
    /// Get a raw pointer to decor_items[idx] (returns DecorSignHighlight*).
    fn nvim_decor_items_get_ptr(idx: u32) -> *mut c_void;
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decor_sh_from_inline() {
        let inline = DecorHighlightInline {
            flags: KSH_CONCEAL,
            priority: 100,
            hl_id: 5,
            conceal_char: 'x' as u32,
        };
        let sh = decor_sh_from_inline(inline);
        assert_eq!(sh.flags, KSH_CONCEAL);
        assert_eq!(sh.priority, 100);
        assert_eq!(sh.hl_id, 5);
        assert_eq!(sh.text[0], 'x' as u32);
        assert_eq!(sh.text[1], 0);
        assert_eq!(sh.next, DECOR_ID_INVALID);
    }

    #[test]
    fn test_decor_type_flags() {
        // Inline highlight
        let flags = decor_type_flags(false, 0, 0, false, true);
        assert_eq!(flags, KEXTMARK_HIGHLIGHT);

        // Inline sign
        let flags = decor_type_flags(false, 0, KSH_IS_SIGN, false, true);
        assert_eq!(flags, KEXTMARK_SIGN);

        // Extended with virt_text
        let flags = decor_type_flags(true, 0, 0, true, false);
        assert_eq!(flags, KEXTMARK_VIRT_TEXT);

        // Extended with virt_lines
        let flags = decor_type_flags(true, KVT_IS_LINES, 0, true, false);
        assert_eq!(flags, KEXTMARK_VIRT_LINES);

        // Extended with sign and virt_text
        let flags = decor_type_flags(true, 0, KSH_IS_SIGN, true, true);
        assert_eq!(flags, KEXTMARK_VIRT_TEXT | KEXTMARK_SIGN);
    }

    #[test]
    fn test_draw_col_states() {
        assert!(draw_col_is_just_added(DRAW_COL_JUST_ADDED));
        assert!(!draw_col_is_just_added(0));

        assert!(draw_col_is_disabled(DRAW_COL_DISABLED));
        assert!(!draw_col_is_disabled(-1));

        assert!(draw_col_is_pending(DRAW_COL_PENDING));
        assert!(!draw_col_is_pending(-1));

        assert!(draw_col_is_unset(DRAW_COL_UNSET));
        assert!(!draw_col_is_unset(0));

        assert!(draw_col_is_valid(0));
        assert!(draw_col_is_valid(100));
        assert!(!draw_col_is_valid(-1));
    }

    #[test]
    fn test_range_position_checks() {
        // End before: returns true when end is before or at position
        // This matches C: r->end_row < row || (r->end_row == row && r->end_col <= col)
        assert!(range_end_before(5, 10, 6, 0)); // row before
        assert!(range_end_before(5, 10, 5, 11)); // same row, col before
        assert!(range_end_before(5, 10, 5, 10)); // exact match (end_col <= col)
        assert!(!range_end_before(5, 10, 5, 9)); // same row, col after

        // Start after
        assert!(range_start_after(6, 0, 5, 10)); // row after
        assert!(range_start_after(5, 11, 5, 10)); // same row, col after
        assert!(!range_start_after(5, 10, 5, 10)); // exact match
        assert!(!range_start_after(5, 9, 5, 10)); // same row, col before
    }

    #[test]
    fn test_priority_cmp() {
        assert_eq!(priority_cmp(10, 0, 20, 0), -1);
        assert_eq!(priority_cmp(20, 0, 10, 0), 1);
        assert_eq!(priority_cmp(10, 0, 10, 1), -1);
        assert_eq!(priority_cmp(10, 1, 10, 0), 1);
        assert_eq!(priority_cmp(10, 0, 10, 0), 0);
    }

    #[test]
    fn test_virt_pos_classification() {
        assert!(virt_pos_is_eol(VirtTextPos::EndOfLine));
        assert!(virt_pos_is_eol(VirtTextPos::EndOfLineRightAlign));
        assert!(!virt_pos_is_eol(VirtTextPos::Inline));

        assert!(virt_pos_needs_col(VirtTextPos::Inline));
        assert!(virt_pos_needs_col(VirtTextPos::Overlay));
        assert!(!virt_pos_needs_col(VirtTextPos::EndOfLine));

        assert!(virt_pos_offscreen_ok(VirtTextPos::EndOfLine));
        assert!(virt_pos_offscreen_ok(VirtTextPos::RightAlign));
        assert!(!virt_pos_offscreen_ok(VirtTextPos::Inline));
    }

    #[test]
    fn test_hl_mode_checks() {
        assert!(hl_mode_replaces(HlMode::Replace));
        assert!(!hl_mode_replaces(HlMode::Combine));

        assert!(hl_mode_combines(HlMode::Combine));
        assert!(hl_mode_combines(HlMode::Unknown));
        assert!(!hl_mode_combines(HlMode::Replace));

        assert!(hl_mode_blends(HlMode::Blend));
        assert!(!hl_mode_blends(HlMode::Combine));
    }

    #[test]
    fn test_sh_flag_checks() {
        assert!(sh_is_sign(KSH_IS_SIGN));
        assert!(!sh_is_sign(0));

        assert!(sh_is_conceal(KSH_CONCEAL));
        assert!(!sh_is_conceal(0));

        assert!(sh_is_spell_on(KSH_SPELL_ON));
        assert!(sh_is_spell_off(KSH_SPELL_OFF));
        assert!(!sh_is_spell_on(KSH_SPELL_OFF));
    }

    #[test]
    fn test_vt_flag_checks() {
        assert!(vt_is_lines(KVT_IS_LINES));
        assert!(!vt_is_lines(0));

        assert!(vt_is_hide(KVT_HIDE));
        assert!(!vt_is_hide(0));
    }

    #[test]
    fn test_decor_kind_checks() {
        assert!(decor_kind_is_virt(DecorKind::VirtText));
        assert!(decor_kind_is_virt(DecorKind::VirtLines));
        assert!(!decor_kind_is_virt(DecorKind::Highlight));

        assert!(decor_kind_is_sign(DecorKind::Sign));
        assert!(!decor_kind_is_sign(DecorKind::Highlight));

        assert!(decor_kind_is_highlight(DecorKind::Highlight));
        assert!(!decor_kind_is_highlight(DecorKind::Sign));

        assert!(decor_kind_is_ui_watched(DecorKind::UIWatched));
        assert!(!decor_kind_is_ui_watched(DecorKind::VirtText));
    }
}
