//! Window API helper functions
//!
//! This module provides Rust implementations of window manipulation utilities.

use std::ffi::c_int;

/// Window split direction
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WinSplit {
    Left = 0,
    Right = 1,
    Above = 2,
    Below = 3,
}

/// Float anchor position
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FloatAnchor {
    NW = 0,
    NE = 1,
    SW = 2,
    SE = 3,
}

/// Float relative type
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FloatRelative {
    Editor = 0,
    Window = 1,
    Cursor = 2,
    Mouse = 3,
    Tabline = 4,
    Laststatus = 5,
}

/// WSP (window split) flags
pub const WSP_VERT: c_int = 0x01;
pub const WSP_HOR: c_int = 0x02;
pub const WSP_TOP: c_int = 0x04;
pub const WSP_BOT: c_int = 0x08;
pub const WSP_ABOVE: c_int = 0x10;
pub const WSP_BELOW: c_int = 0x20;

/// kFloatAnchorEast flag
pub const FLOAT_ANCHOR_EAST: u8 = 1;
/// kFloatAnchorSouth flag
pub const FLOAT_ANCHOR_SOUTH: u8 = 2;

/// MAXCOL constant
pub const MAXCOL: c_int = i32::MAX;

// Note: rs_win_valid is already defined in window crate

/// Check if row position is valid for cursor
#[no_mangle]
pub extern "C" fn rs_win_cursor_row_valid(row: i64, line_count: i64) -> bool {
    row > 0 && row <= line_count
}

/// Check if column position is in valid range
#[no_mangle]
pub extern "C" fn rs_win_cursor_col_valid(col: i64) -> bool {
    col >= 0 && col <= MAXCOL as i64
}

/// Check if position array has correct format (size 2, integers)
#[no_mangle]
pub extern "C" fn rs_pos_array_valid(size: usize, type0: c_int, type1: c_int) -> bool {
    // kObjectTypeInteger = 1
    const OBJECT_TYPE_INTEGER: c_int = 1;
    size == 2 && type0 == OBJECT_TYPE_INTEGER && type1 == OBJECT_TYPE_INTEGER
}

/// Check if namespace id is valid (-1 or higher allowed)
#[no_mangle]
pub extern "C" fn rs_ns_id_valid(ns_id: i64) -> bool {
    ns_id >= -1
}

/// Calculate window split flags
#[no_mangle]
pub extern "C" fn rs_win_split_flags(split: c_int, toplevel: bool) -> c_int {
    let mut flags = 0;

    // Above or Below = horizontal split
    if split == WinSplit::Above as c_int || split == WinSplit::Below as c_int {
        flags |= WSP_HOR;
    } else {
        flags |= WSP_VERT;
    }

    // Above or Left = top/above position
    if split == WinSplit::Above as c_int || split == WinSplit::Left as c_int {
        flags |= if toplevel { WSP_TOP } else { WSP_ABOVE };
    } else {
        flags |= if toplevel { WSP_BOT } else { WSP_BELOW };
    }

    flags
}

/// Check if split flags indicate vertical split
#[no_mangle]
pub extern "C" fn rs_is_vert_split(flags: c_int) -> bool {
    (flags & WSP_VERT) != 0
}

/// Get split size based on flags (width for vert, height for horiz)
#[no_mangle]
pub extern "C" fn rs_split_size(flags: c_int, width: c_int, height: c_int) -> c_int {
    if (flags & WSP_VERT) != 0 {
        width
    } else {
        height
    }
}

/// Parse float anchor from string comparison results
/// Returns the anchor flags (EAST=1, SOUTH=2)
#[no_mangle]
pub extern "C" fn rs_parse_anchor_nw() -> u8 {
    0 // NW is default
}

#[no_mangle]
pub extern "C" fn rs_parse_anchor_ne() -> u8 {
    FLOAT_ANCHOR_EAST
}

#[no_mangle]
pub extern "C" fn rs_parse_anchor_sw() -> u8 {
    FLOAT_ANCHOR_SOUTH
}

#[no_mangle]
pub extern "C" fn rs_parse_anchor_se() -> u8 {
    FLOAT_ANCHOR_SOUTH | FLOAT_ANCHOR_EAST
}

/// Check if anchor has south flag
#[no_mangle]
pub extern "C" fn rs_anchor_has_south(anchor: u8) -> bool {
    (anchor & FLOAT_ANCHOR_SOUTH) != 0
}

/// Check if anchor has east flag
#[no_mangle]
pub extern "C" fn rs_anchor_has_east(anchor: u8) -> bool {
    (anchor & FLOAT_ANCHOR_EAST) != 0
}

/// Get default row for bufpos based on anchor
#[no_mangle]
pub extern "C" fn rs_bufpos_default_row(anchor: u8) -> f64 {
    if (anchor & FLOAT_ANCHOR_SOUTH) != 0 {
        0.0
    } else {
        1.0
    }
}

/// Check if float relative is window
#[no_mangle]
pub extern "C" fn rs_relative_is_window(relative: c_int) -> bool {
    relative == FloatRelative::Window as c_int
}

/// Check if we have vertical or split config (non-float)
#[no_mangle]
pub extern "C" fn rs_is_split_config(has_vertical: bool, has_split: bool) -> bool {
    has_vertical || has_split
}

/// Calculate split direction for vertical config
#[no_mangle]
pub extern "C" fn rs_vert_split_dir(p_spr: bool) -> c_int {
    if p_spr {
        WinSplit::Right as c_int
    } else {
        WinSplit::Left as c_int
    }
}

/// Calculate split direction for horizontal config
#[no_mangle]
pub extern "C" fn rs_horiz_split_dir(p_sb: bool) -> c_int {
    if p_sb {
        WinSplit::Below as c_int
    } else {
        WinSplit::Above as c_int
    }
}

/// Check if size needs adjustment after split
#[no_mangle]
pub extern "C" fn rs_need_size_adjust(requested: c_int, actual: c_int) -> bool {
    requested > 0 && actual != requested
}

/// Check if window is cmdwin or old curwin
#[no_mangle]
pub extern "C" fn rs_is_cmdwin_related(
    win_handle: c_int,
    cmdwin_win_handle: c_int,
    cmdwin_old_curwin_handle: c_int,
) -> bool {
    win_handle == cmdwin_win_handle || win_handle == cmdwin_old_curwin_handle
}

/// Check if buffer is cmdwin buffer
#[no_mangle]
pub extern "C" fn rs_is_cmdwin_buf(buf_handle: c_int, cmdwin_buf_handle: c_int) -> bool {
    buf_handle == cmdwin_buf_handle
}

/// Check if cmdline offset is set (< INT_MAX)
#[no_mangle]
pub extern "C" fn rs_has_cmdline_offset(offset: c_int) -> bool {
    offset < c_int::MAX
}

/// Get INT_MAX for cmdline offset reset
#[no_mangle]
pub extern "C" fn rs_cmdline_offset_unset() -> c_int {
    c_int::MAX
}

/// Check if border array size is valid (1, 2, 4, or 8)
#[no_mangle]
pub extern "C" fn rs_border_size_valid(size: usize) -> bool {
    size > 0 && size <= 8 && (size & (size - 1)) == 0
}

/// Double border array size (for expansion)
#[no_mangle]
pub extern "C" fn rs_border_double_size(size: usize) -> usize {
    size << 1
}

/// Check if border corner needs to be specified
/// (non-empty adjacent edges with empty corner)
#[no_mangle]
pub extern "C" fn rs_border_corner_missing(
    corner_empty: bool,
    edge1_present: bool,
    edge2_present: bool,
) -> bool {
    corner_empty && edge1_present && edge2_present
}

/// Check if zindex is valid (positive)
#[no_mangle]
pub extern "C" fn rs_zindex_valid(zindex: i64) -> bool {
    zindex > 0
}

/// Check if width/height is valid (positive)
#[no_mangle]
pub extern "C" fn rs_dimension_valid(dim: i64) -> bool {
    dim > 0
}

/// Check if style is minimal
#[no_mangle]
pub extern "C" fn rs_style_is_minimal(style: c_int) -> bool {
    // kWinStyleMinimal = 1
    style == 1
}

/// Check if style is unused (empty string)
#[no_mangle]
pub extern "C" fn rs_style_is_unused(style: c_int) -> bool {
    // kWinStyleUnused = 0
    style == 0
}

/// WinSplit direction from frame layout
#[no_mangle]
pub extern "C" fn rs_split_dir_from_layout(layout_is_col: bool, has_next: bool) -> c_int {
    if layout_is_col {
        if has_next {
            WinSplit::Above as c_int
        } else {
            WinSplit::Below as c_int
        }
    } else if has_next {
        WinSplit::Left as c_int
    } else {
        WinSplit::Right as c_int
    }
}

/// Check if this is a to_split scenario
#[no_mangle]
pub extern "C" fn rs_is_to_split(
    relative_empty: bool,
    external: bool,
    has_split_or_vert: bool,
    was_split: bool,
) -> bool {
    relative_empty && !external && (has_split_or_vert || was_split)
}

/// Check if we should keep existing split
#[no_mangle]
pub extern "C" fn rs_keep_existing_split(
    has_vertical: bool,
    has_split: bool,
    was_split: bool,
    has_win: bool,
    old_split: c_int,
    new_split: c_int,
) -> bool {
    (!has_vertical && !has_split) || (was_split && !has_win && old_split == new_split)
}

/// Get neighbor direction for split reconfiguration
#[no_mangle]
pub extern "C" fn rs_get_neighbor_direction(split: c_int) -> bool {
    // Returns true for Above/Left (take next), false for Below/Right (take prev)
    split == WinSplit::Above as c_int || split == WinSplit::Left as c_int
}

/// Calculate vertical split direction with old_split consideration
#[no_mangle]
pub extern "C" fn rs_vert_split_dir_reconfig(old_split: c_int, p_spr: bool) -> c_int {
    if old_split == WinSplit::Right as c_int || p_spr {
        WinSplit::Right as c_int
    } else {
        WinSplit::Left as c_int
    }
}

/// Calculate horizontal split direction with old_split consideration
#[no_mangle]
pub extern "C" fn rs_horiz_split_dir_reconfig(old_split: c_int, p_sb: bool) -> c_int {
    if old_split == WinSplit::Below as c_int || p_sb {
        WinSplit::Below as c_int
    } else {
        WinSplit::Above as c_int
    }
}

/// Check if we need to switch away from curwin moving tabpage
#[no_mangle]
pub extern "C" fn rs_curwin_moving_tp(
    win_is_curwin: bool,
    has_parent: bool,
    win_tp_is_parent_tp: bool,
) -> bool {
    win_is_curwin && has_parent && !win_tp_is_parent_tp
}

/// Count of items needed for text height return dict
#[no_mangle]
pub extern "C" fn rs_text_height_dict_size() -> usize {
    4 // all, fill, end_row, end_vcol
}

/// Check start <= end for line validation
#[no_mangle]
pub extern "C" fn rs_lnum_range_valid(start: i64, end: i64) -> bool {
    start <= end
}

/// Check vcol range is valid
#[no_mangle]
pub extern "C" fn rs_vcol_range_valid(start: i64, end: i64, is_same_row: bool) -> bool {
    !is_same_row || start <= end
}

/// Check vcol is in valid range [0, MAXCOL]
#[no_mangle]
pub extern "C" fn rs_vcol_in_range(vcol: i64) -> bool {
    vcol >= 0 && vcol <= MAXCOL as i64
}

/// Check max_height is valid (positive)
#[no_mangle]
pub extern "C" fn rs_max_height_valid(max_height: i64) -> bool {
    max_height > 0
}

/// Default max for text height (INT64_MAX)
#[no_mangle]
pub extern "C" fn rs_text_height_default_max() -> i64 {
    i64::MAX
}

/// Convert lnum to 0-indexed row for return
#[no_mangle]
pub extern "C" fn rs_lnum_to_row(lnum: i64) -> i64 {
    lnum - 1
}

/// Check if parent is floating (invalid for split)
#[no_mangle]
pub extern "C" fn rs_parent_is_floating(floating: bool) -> bool {
    floating
}

/// Check frame count for self-split scenarios
#[no_mangle]
pub extern "C" fn rs_frame_count_for_self_split(n_frames: c_int) -> c_int {
    if n_frames > 2 {
        2 // Can find neighbor
    } else if n_frames == 2 {
        1 // Rotate with neighbor
    } else {
        0 // Cannot split into itself
    }
}

/// Check if config win equals self
#[no_mangle]
pub extern "C" fn rs_config_win_equals_self(config_win: c_int, self_handle: c_int) -> bool {
    config_win > 0 && config_win == self_handle
}

/// Array count for cursor position (always 2)
#[no_mangle]
pub extern "C" fn rs_cursor_array_size() -> usize {
    2
}

/// Array count for position (always 2)
#[no_mangle]
pub extern "C" fn rs_position_array_size() -> usize {
    2
}

/// Check if border tuple has valid size
#[no_mangle]
pub extern "C" fn rs_border_tuple_valid(size: usize) -> bool {
    (1..=2).contains(&size)
}

/// Check if single cell (for border char validation)
#[no_mangle]
pub extern "C" fn rs_is_single_cell(cells: c_int) -> bool {
    cells <= 1
}

/// Check if we should use shadow colors for border
#[no_mangle]
pub extern "C" fn rs_border_needs_shadow() -> bool {
    true
}

// Note: rs_border_char_count is already defined in winfloat crate

/// Check if external requires multigrid
#[no_mangle]
pub extern "C" fn rs_external_needs_multigrid(external: bool, has_multigrid: bool) -> bool {
    external && !has_multigrid
}

/// Check if cmdwin type is active
#[no_mangle]
pub extern "C" fn rs_cmdwin_active(cmdwin_type: c_int) -> bool {
    cmdwin_type != 0
}

/// Check if entering is blocked by cmdwin
#[no_mangle]
pub extern "C" fn rs_cmdwin_blocks_enter(cmdwin_type: c_int, enter: bool) -> bool {
    cmdwin_type != 0 && enter
}

/// AlignTextPos enumeration
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AlignTextPos {
    Left = 0,
    Center = 1,
    Right = 2,
}

/// Get default alignment position (left)
#[no_mangle]
pub extern "C" fn rs_default_align() -> c_int {
    AlignTextPos::Left as c_int
}

/// Check if bordertext is present (non-empty)
#[no_mangle]
pub extern "C" fn rs_bordertext_present(size: usize) -> bool {
    size > 0
}

/// Check if it's a valid style value (empty or "minimal")
#[no_mangle]
pub extern "C" fn rs_style_valid_or_empty(is_empty: bool, is_minimal: bool) -> bool {
    is_empty || is_minimal
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_win_split_flags() {
        // Above should be horizontal
        let flags = rs_win_split_flags(WinSplit::Above as c_int, false);
        assert_eq!(flags & WSP_HOR, WSP_HOR);
        assert_eq!(flags & WSP_ABOVE, WSP_ABOVE);

        // Right should be vertical
        let flags = rs_win_split_flags(WinSplit::Right as c_int, false);
        assert_eq!(flags & WSP_VERT, WSP_VERT);
        assert_eq!(flags & WSP_BELOW, WSP_BELOW);

        // Left toplevel should have TOP
        let flags = rs_win_split_flags(WinSplit::Left as c_int, true);
        assert_eq!(flags & WSP_TOP, WSP_TOP);
    }

    #[test]
    fn test_cursor_validation() {
        assert!(rs_win_cursor_row_valid(1, 100));
        assert!(rs_win_cursor_row_valid(100, 100));
        assert!(!rs_win_cursor_row_valid(0, 100));
        assert!(!rs_win_cursor_row_valid(101, 100));
        assert!(!rs_win_cursor_row_valid(-1, 100));

        assert!(rs_win_cursor_col_valid(0));
        assert!(rs_win_cursor_col_valid(100));
        assert!(!rs_win_cursor_col_valid(-1));
    }

    #[test]
    fn test_border_size_valid() {
        assert!(rs_border_size_valid(1));
        assert!(rs_border_size_valid(2));
        assert!(rs_border_size_valid(4));
        assert!(rs_border_size_valid(8));
        assert!(!rs_border_size_valid(0));
        assert!(!rs_border_size_valid(3));
        assert!(!rs_border_size_valid(5));
        assert!(!rs_border_size_valid(9));
    }

    #[test]
    fn test_anchor_flags() {
        assert_eq!(rs_parse_anchor_nw(), 0);
        assert_eq!(rs_parse_anchor_ne(), FLOAT_ANCHOR_EAST);
        assert_eq!(rs_parse_anchor_sw(), FLOAT_ANCHOR_SOUTH);
        assert_eq!(rs_parse_anchor_se(), FLOAT_ANCHOR_SOUTH | FLOAT_ANCHOR_EAST);

        assert!(!rs_anchor_has_south(0));
        assert!(rs_anchor_has_south(FLOAT_ANCHOR_SOUTH));
        assert!(rs_anchor_has_south(FLOAT_ANCHOR_SOUTH | FLOAT_ANCHOR_EAST));
    }

    #[test]
    fn test_split_dir() {
        assert!(rs_vert_split_dir(true) == WinSplit::Right as c_int);
        assert!(rs_vert_split_dir(false) == WinSplit::Left as c_int);
        assert!(rs_horiz_split_dir(true) == WinSplit::Below as c_int);
        assert!(rs_horiz_split_dir(false) == WinSplit::Above as c_int);
    }

    #[test]
    fn test_dimension_validation() {
        assert!(rs_dimension_valid(1));
        assert!(rs_dimension_valid(100));
        assert!(!rs_dimension_valid(0));
        assert!(!rs_dimension_valid(-1));
    }

    #[test]
    fn test_lnum_range() {
        assert!(rs_lnum_range_valid(1, 10));
        assert!(rs_lnum_range_valid(5, 5));
        assert!(!rs_lnum_range_valid(10, 1));
    }
}
