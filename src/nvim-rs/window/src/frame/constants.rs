//! Frame direction constants, layout flags, and split constraints.
//!
//! This module provides Rust FFI exports for frame-related constants used
//! throughout Neovim's window management system. These constants define:
//! - Frame layout types (leaf, row, column)
//! - Window split flags (WSP_* flags for win_split)
//! - Screen dimension constraints (MIN_COLUMNS, MIN_LINES)
//! - Snapshot indices for layout restoration
//! - Window layout types for startup (-o, -O, -p flags)

use std::ffi::c_int;

// =============================================================================
// Frame Layout Type Constants
// =============================================================================
// These are already defined in lib.rs but we re-export FFI accessors here

/// Frame is a leaf (contains a window).
pub const FR_LEAF: i8 = 0;

/// Frame contains a row of windows (horizontal arrangement).
pub const FR_ROW: i8 = 1;

/// Frame contains a column of windows (vertical arrangement).
pub const FR_COL: i8 = 2;

// =============================================================================
// Window Split Flags (WSP_*)
// =============================================================================
// Arguments for win_split() from window.h

/// Require enough room for the split.
pub const WSP_ROOM: c_int = 0x01;

/// Split/equalize vertically.
pub const WSP_VERT: c_int = 0x02;

/// Equalize horizontally.
pub const WSP_HOR: c_int = 0x04;

/// Window at top-left of shell.
pub const WSP_TOP: c_int = 0x08;

/// Window at bottom-right of shell.
pub const WSP_BOT: c_int = 0x10;

/// Creating the help window.
pub const WSP_HELP: c_int = 0x20;

/// Put new window below/right.
pub const WSP_BELOW: c_int = 0x40;

/// Put new window above/left.
pub const WSP_ABOVE: c_int = 0x80;

/// Don't copy location list.
pub const WSP_NEWLOC: c_int = 0x100;

/// Don't enter the new window.
pub const WSP_NOENTER: c_int = 0x200;

// =============================================================================
// Screen Dimension Constraints
// =============================================================================

/// Minimal columns for screen.
pub const MIN_COLUMNS: c_int = 12;

/// Minimal lines for screen.
pub const MIN_LINES: c_int = 2;

/// Height of a status line under a window.
pub const STATUS_HEIGHT: c_int = 1;

/// Lowest number used for window ID. Cannot have this many windows per tab.
pub const LOWEST_WIN_ID: c_int = 1000;

// =============================================================================
// Window Layout Snapshot Indices
// =============================================================================

/// Index for help window layout snapshot.
pub const SNAP_HELP_IDX: c_int = 0;

/// Index for autocmd window layout snapshot.
pub const SNAP_AUCMD_IDX: c_int = 1;

/// Number of snapshot slots per tabpage.
pub const SNAP_COUNT: c_int = 2;

// =============================================================================
// Window Layout Types (for startup: -o, -O, -p flags)
// =============================================================================

/// Horizontal window layout (-o flag).
pub const WIN_LAYOUT_HOR: c_int = 1;

/// Vertical window layout (-O flag).
pub const WIN_LAYOUT_VER: c_int = 2;

/// Tab pages layout (-p flag).
pub const WIN_LAYOUT_TABS: c_int = 3;

// =============================================================================
// FFI Exports - Frame Layout Constants
// =============================================================================

/// FFI export: Get the FR_LEAF constant.
#[unsafe(no_mangle)]
pub extern "C" fn rs_frame_const_fr_leaf() -> c_int {
    c_int::from(FR_LEAF)
}

/// FFI export: Get the FR_ROW constant.
#[unsafe(no_mangle)]
pub extern "C" fn rs_frame_const_fr_row() -> c_int {
    c_int::from(FR_ROW)
}

/// FFI export: Get the FR_COL constant.
#[unsafe(no_mangle)]
pub extern "C" fn rs_frame_const_fr_col() -> c_int {
    c_int::from(FR_COL)
}

// =============================================================================
// FFI Exports - Window Split Flags
// =============================================================================

/// FFI export: Get WSP_ROOM flag.
#[unsafe(no_mangle)]
pub extern "C" fn rs_wsp_room() -> c_int {
    WSP_ROOM
}

/// FFI export: Get WSP_VERT flag.
#[unsafe(no_mangle)]
pub extern "C" fn rs_wsp_vert() -> c_int {
    WSP_VERT
}

/// FFI export: Get WSP_HOR flag.
#[unsafe(no_mangle)]
pub extern "C" fn rs_wsp_hor() -> c_int {
    WSP_HOR
}

/// FFI export: Get WSP_TOP flag.
#[unsafe(no_mangle)]
pub extern "C" fn rs_wsp_top() -> c_int {
    WSP_TOP
}

/// FFI export: Get WSP_BOT flag.
#[unsafe(no_mangle)]
pub extern "C" fn rs_wsp_bot() -> c_int {
    WSP_BOT
}

/// FFI export: Get WSP_HELP flag.
#[unsafe(no_mangle)]
pub extern "C" fn rs_wsp_help() -> c_int {
    WSP_HELP
}

/// FFI export: Get WSP_BELOW flag.
#[unsafe(no_mangle)]
pub extern "C" fn rs_wsp_below() -> c_int {
    WSP_BELOW
}

/// FFI export: Get WSP_ABOVE flag.
#[unsafe(no_mangle)]
pub extern "C" fn rs_wsp_above() -> c_int {
    WSP_ABOVE
}

/// FFI export: Get WSP_NEWLOC flag.
#[unsafe(no_mangle)]
pub extern "C" fn rs_wsp_newloc() -> c_int {
    WSP_NEWLOC
}

/// FFI export: Get WSP_NOENTER flag.
#[unsafe(no_mangle)]
pub extern "C" fn rs_wsp_noenter() -> c_int {
    WSP_NOENTER
}

// =============================================================================
// FFI Exports - Screen Dimension Constraints
// =============================================================================

/// FFI export: Get MIN_COLUMNS constant.
#[unsafe(no_mangle)]
pub extern "C" fn rs_min_columns() -> c_int {
    MIN_COLUMNS
}

/// FFI export: Get MIN_LINES constant.
#[unsafe(no_mangle)]
pub extern "C" fn rs_min_lines() -> c_int {
    MIN_LINES
}

/// FFI export: Get LOWEST_WIN_ID constant.
#[unsafe(no_mangle)]
pub extern "C" fn rs_lowest_win_id() -> c_int {
    LOWEST_WIN_ID
}

// =============================================================================
// FFI Exports - Snapshot Indices
// =============================================================================

/// FFI export: Get SNAP_HELP_IDX constant.
#[unsafe(no_mangle)]
pub extern "C" fn rs_snap_help_idx() -> c_int {
    SNAP_HELP_IDX
}

/// FFI export: Get SNAP_AUCMD_IDX constant.
#[unsafe(no_mangle)]
pub extern "C" fn rs_snap_aucmd_idx() -> c_int {
    SNAP_AUCMD_IDX
}

/// FFI export: Get SNAP_COUNT constant.
#[unsafe(no_mangle)]
pub extern "C" fn rs_snap_count() -> c_int {
    SNAP_COUNT
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_frame_layout_constants() {
        assert_eq!(FR_LEAF, 0);
        assert_eq!(FR_ROW, 1);
        assert_eq!(FR_COL, 2);
    }

    #[test]
    fn test_wsp_flags() {
        // Verify flags are distinct powers of 2 (can be combined)
        assert_eq!(WSP_ROOM, 0x01);
        assert_eq!(WSP_VERT, 0x02);
        assert_eq!(WSP_HOR, 0x04);
        assert_eq!(WSP_TOP, 0x08);
        assert_eq!(WSP_BOT, 0x10);
        assert_eq!(WSP_HELP, 0x20);
        assert_eq!(WSP_BELOW, 0x40);
        assert_eq!(WSP_ABOVE, 0x80);
        assert_eq!(WSP_NEWLOC, 0x100);
        assert_eq!(WSP_NOENTER, 0x200);

        // Flags can be combined
        let combined = WSP_VERT | WSP_BELOW | WSP_NOENTER;
        assert_eq!(combined, 0x242);
    }

    #[test]
    fn test_screen_constraints() {
        assert_eq!(MIN_COLUMNS, 12);
        assert_eq!(MIN_LINES, 2);
        assert_eq!(STATUS_HEIGHT, 1);
        assert_eq!(LOWEST_WIN_ID, 1000);
    }

    #[test]
    fn test_snapshot_indices() {
        assert_eq!(SNAP_HELP_IDX, 0);
        assert_eq!(SNAP_AUCMD_IDX, 1);
        assert_eq!(SNAP_COUNT, 2);
    }

    #[test]
    fn test_win_layout_types() {
        assert_eq!(WIN_LAYOUT_HOR, 1);
        assert_eq!(WIN_LAYOUT_VER, 2);
        assert_eq!(WIN_LAYOUT_TABS, 3);
    }
}
