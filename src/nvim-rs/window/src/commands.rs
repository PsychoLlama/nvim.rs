//! Window command handler functions.
//!
//! This module provides Rust helper functions for window command operations
//! from `src/nvim/window.c`, supporting do_window() and Ex commands.

// Window dimensions may need truncation when converting between types.
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_possible_wrap)]
#![allow(clippy::missing_const_for_fn)]

use std::ffi::c_int;

use crate::WinHandle;

// =============================================================================
// Window Command Constants
// =============================================================================

/// Control character constants for CTRL-W commands.
/// These match the Ctrl_* macros in vim.h.
const CTRL_S: c_int = 19; // ^S - horizontal split
const CTRL_V: c_int = 22; // ^V - vertical split
const CTRL_N: c_int = 14; // ^N - new window
const CTRL_Q: c_int = 17; // ^Q - quit window
const CTRL_C: c_int = 3; // ^C - close window
const CTRL_O: c_int = 15; // ^O - close others
const CTRL_W: c_int = 23; // ^W - next window
const CTRL_P: c_int = 16; // ^P - previous window
const CTRL_H: c_int = 8; // ^H - left
const CTRL_J: c_int = 10; // ^J - down
const CTRL_K: c_int = 11; // ^K - up
const CTRL_L: c_int = 12; // ^L - right
const CTRL_T: c_int = 20; // ^T - new tab
const CTRL_X: c_int = 24; // ^X - exchange
const CTRL_R: c_int = 18; // ^R - rotate

/// WSP flags for split operations.
const WSP_VERT: c_int = 0x01;
#[allow(dead_code)]
const WSP_TOP: c_int = 0x02;
#[allow(dead_code)]
const WSP_BOT: c_int = 0x04;
#[allow(dead_code)]
const WSP_HELP: c_int = 0x08;

// =============================================================================
// External C Functions
// =============================================================================

extern "C" {
    /// Get curwin.
    fn nvim_get_curwin() -> WinHandle;

    /// Get firstwin.
    fn nvim_get_firstwin() -> WinHandle;

    /// Get w_next from window.
    fn nvim_win_get_next(wp: WinHandle) -> WinHandle;

    /// Get w_floating from window.
    fn nvim_win_get_floating(wp: WinHandle) -> c_int;

    /// Get cmdwin_type global.
    fn nvim_get_cmdwin_type() -> c_int;
}

// =============================================================================
// Command Classification
// =============================================================================

/// Window command category for dispatch.
#[repr(C)]
pub enum WinCmdCategory {
    /// Split commands (s, v, n, ^)
    Split = 0,
    /// Close commands (c, q, o)
    Close = 1,
    /// Navigation commands (h, j, k, l, w, W, p, t, b)
    Navigate = 2,
    /// Resize commands (+, -, <, >, =, |, _)
    Resize = 3,
    /// Move commands (r, R, x, H, J, K, L, T)
    Move = 4,
    /// Tab commands (g<Tab>)
    Tab = 5,
    /// Preview/quickfix (P, z)
    Preview = 6,
    /// Unknown/other
    Other = 7,
}

/// Classify a CTRL-W command character into its category.
fn classify_win_cmd_impl(nchar: c_int) -> WinCmdCategory {
    match nchar {
        // Split commands: 'S', 's', ^S, 'V', 'v', ^V, 'N', 'n', ^N, '^'
        0x53 | 0x73 | CTRL_S | 0x56 | 0x76 | CTRL_V | 0x4E | 0x6E | CTRL_N | 0x5E => {
            WinCmdCategory::Split
        }

        // Close commands: 'c', ^C, 'q', ^Q, 'o', ^O
        0x63 | CTRL_C | 0x71 | CTRL_Q | 0x6F | CTRL_O => WinCmdCategory::Close,

        // Navigation commands: 'h', ^H, 'j', ^J, 'k', ^K, 'l', ^L, 'w', ^W, 'W', 'p', ^P, 't', 'b'
        0x68 | CTRL_H | 0x6A | CTRL_J | 0x6B | CTRL_K | 0x6C | CTRL_L | 0x77 | CTRL_W | 0x57
        | 0x70 | CTRL_P | 0x74 | 0x62 => WinCmdCategory::Navigate,

        // Resize commands: '+', '-', '<', '>', '=', '|', '_'
        0x2B | 0x2D | 0x3C | 0x3E | 0x3D | 0x7C | 0x5F => WinCmdCategory::Resize,

        // Move commands: 'r', ^R, 'R', 'x', ^X, 'H', 'J', 'K', 'L', 'T', ^T
        0x72 | CTRL_R | 0x52 | 0x78 | CTRL_X | 0x48 | 0x4A | 0x4B | 0x4C | 0x54 | CTRL_T => {
            WinCmdCategory::Move
        }

        // Tab commands: 'g' (followed by Tab)
        0x67 => WinCmdCategory::Tab,

        // Preview commands: 'P', 'z'
        0x50 | 0x7A => WinCmdCategory::Preview,

        _ => WinCmdCategory::Other,
    }
}

// =============================================================================
// Command Validation
// =============================================================================

/// Check if command is blocked in command-line window.
fn cmd_blocked_in_cmdwin_impl(nchar: c_int) -> bool {
    unsafe {
        if nvim_get_cmdwin_type() == 0 {
            return false;
        }

        // Most commands are blocked in cmdwin except navigation
        matches!(
            classify_win_cmd_impl(nchar),
            WinCmdCategory::Split
                | WinCmdCategory::Close
                | WinCmdCategory::Move
                | WinCmdCategory::Tab
        )
    }
}

/// Check if window can be target of command (not floating if required).
fn cmd_requires_non_floating_impl(nchar: c_int) -> bool {
    // Commands that require non-floating window
    matches!(
        classify_win_cmd_impl(nchar),
        WinCmdCategory::Split | WinCmdCategory::Move | WinCmdCategory::Resize
    )
}

/// Check if current window is valid for command.
fn curwin_valid_for_cmd_impl(nchar: c_int) -> bool {
    unsafe {
        let curwin = nvim_get_curwin();
        if curwin.is_null() {
            return false;
        }

        // Some commands need non-floating window
        if cmd_requires_non_floating_impl(nchar) && nvim_win_get_floating(curwin) != 0 {
            return false;
        }

        true
    }
}

// =============================================================================
// Command Parameter Helpers
// =============================================================================

/// Calculate effective Prenum (count) for command.
/// Returns Prenum if > 0, otherwise 1.
fn effective_prenum_impl(prenum: c_int) -> c_int {
    if prenum > 0 {
        prenum
    } else {
        1
    }
}

/// Get split flags from command character.
fn get_split_flags_impl(nchar: c_int) -> c_int {
    match nchar {
        // Vertical split
        0x56 | 0x76 | CTRL_V => WSP_VERT, // 'V', 'v', ^V
        // Other splits ('N', 'n', ^N, etc.) depend on context or use default
        _ => 0,
    }
}

/// Check if command needs to check for only one window.
fn cmd_needs_multiple_windows_impl(nchar: c_int) -> bool {
    matches!(
        classify_win_cmd_impl(nchar),
        WinCmdCategory::Navigate | WinCmdCategory::Move
    )
}

// =============================================================================
// Navigation Direction Helpers
// =============================================================================

/// Navigation direction for window commands.
#[repr(C)]
pub enum NavDirection {
    Left = 0,
    Down = 1,
    Up = 2,
    Right = 3,
    Next = 4,
    Prev = 5,
    First = 6,
    Last = 7,
}

/// Get navigation direction from command character.
fn get_nav_direction_impl(nchar: c_int) -> NavDirection {
    match nchar {
        0x68 | CTRL_H => NavDirection::Left,        // 'h', ^H
        0x6A | CTRL_J => NavDirection::Down,        // 'j', ^J
        0x6B | CTRL_K => NavDirection::Up,          // 'k', ^K
        0x6C | CTRL_L => NavDirection::Right,       // 'l', ^L
        0x77 | CTRL_W => NavDirection::Next,        // 'w', ^W
        0x57 | 0x70 | CTRL_P => NavDirection::Prev, // 'W', 'p', ^P
        0x74 => NavDirection::First,                // 't'
        0x62 => NavDirection::Last,                 // 'b'
        _ => NavDirection::Next,
    }
}

// =============================================================================
// Move Direction Helpers
// =============================================================================

/// Move direction for window move commands.
#[repr(C)]
pub enum MoveDirection {
    Left = 0,
    Down = 1,
    Up = 2,
    Right = 3,
    Rotate = 4,
    RotateBack = 5,
    Exchange = 6,
    ToTab = 7,
}

/// Get move direction from command character.
fn get_move_direction_impl(nchar: c_int) -> MoveDirection {
    match nchar {
        0x48 => MoveDirection::Left,              // 'H'
        0x4A => MoveDirection::Down,              // 'J'
        0x4B => MoveDirection::Up,                // 'K'
        0x4C => MoveDirection::Right,             // 'L'
        0x72 | CTRL_R => MoveDirection::Rotate,   // 'r', ^R
        0x52 => MoveDirection::RotateBack,        // 'R'
        0x78 | CTRL_X => MoveDirection::Exchange, // 'x', ^X
        0x54 | CTRL_T => MoveDirection::ToTab,    // 'T', ^T
        _ => MoveDirection::Exchange,
    }
}

// =============================================================================
// Resize Direction Helpers
// =============================================================================

/// Resize operation for window resize commands.
#[repr(C)]
#[derive(Clone, Copy)]
pub enum ResizeOp {
    IncHeight = 0,
    DecHeight = 1,
    IncWidth = 2,
    DecWidth = 3,
    SetHeight = 4,
    SetWidth = 5,
    Equalize = 6,
}

/// Get resize operation from command character.
fn get_resize_op_impl(nchar: c_int) -> ResizeOp {
    match nchar {
        0x2B => ResizeOp::IncHeight, // '+'
        0x2D => ResizeOp::DecHeight, // '-'
        0x3E => ResizeOp::IncWidth,  // '>'
        0x3C => ResizeOp::DecWidth,  // '<'
        0x5F => ResizeOp::SetHeight, // '_'
        0x7C => ResizeOp::SetWidth,  // '|'
        // '=' and all other chars map to Equalize
        _ => ResizeOp::Equalize,
    }
}

/// Check if resize op affects height (vs width).
fn resize_is_height_impl(op: ResizeOp) -> bool {
    matches!(
        op,
        ResizeOp::IncHeight | ResizeOp::DecHeight | ResizeOp::SetHeight
    )
}

// =============================================================================
// Window Count Helpers
// =============================================================================

/// Count non-floating windows.
fn count_non_floating_windows_impl() -> c_int {
    unsafe {
        let mut count = 0;
        let mut wp = nvim_get_firstwin();
        while !wp.is_null() {
            if nvim_win_get_floating(wp) == 0 {
                count += 1;
            }
            wp = nvim_win_get_next(wp);
        }
        count
    }
}

/// Check if only one non-floating window exists.
fn only_one_window_impl() -> bool {
    count_non_floating_windows_impl() <= 1
}

// =============================================================================
// FFI Exports
// =============================================================================

/// FFI: Classify window command.
#[unsafe(no_mangle)]
pub extern "C" fn rs_cmd_classify(nchar: c_int) -> c_int {
    classify_win_cmd_impl(nchar) as c_int
}

/// FFI: Check if command blocked in cmdwin.
#[unsafe(no_mangle)]
pub extern "C" fn rs_cmd_blocked_cmdwin(nchar: c_int) -> c_int {
    c_int::from(cmd_blocked_in_cmdwin_impl(nchar))
}

/// FFI: Check if command requires non-floating window.
#[unsafe(no_mangle)]
pub extern "C" fn rs_cmd_requires_non_floating(nchar: c_int) -> c_int {
    c_int::from(cmd_requires_non_floating_impl(nchar))
}

/// FFI: Check if curwin valid for command.
#[unsafe(no_mangle)]
pub extern "C" fn rs_cmd_curwin_valid(nchar: c_int) -> c_int {
    c_int::from(curwin_valid_for_cmd_impl(nchar))
}

/// FFI: Get effective prenum.
#[unsafe(no_mangle)]
pub extern "C" fn rs_cmd_effective_prenum(prenum: c_int) -> c_int {
    effective_prenum_impl(prenum)
}

/// FFI: Get split flags for command.
#[unsafe(no_mangle)]
pub extern "C" fn rs_cmd_split_flags(nchar: c_int) -> c_int {
    get_split_flags_impl(nchar)
}

/// FFI: Check if command needs multiple windows.
#[unsafe(no_mangle)]
pub extern "C" fn rs_cmd_needs_multi_win(nchar: c_int) -> c_int {
    c_int::from(cmd_needs_multiple_windows_impl(nchar))
}

/// FFI: Get navigation direction.
#[unsafe(no_mangle)]
pub extern "C" fn rs_cmd_nav_direction(nchar: c_int) -> c_int {
    get_nav_direction_impl(nchar) as c_int
}

/// FFI: Get move direction.
#[unsafe(no_mangle)]
pub extern "C" fn rs_cmd_move_direction(nchar: c_int) -> c_int {
    get_move_direction_impl(nchar) as c_int
}

/// FFI: Get resize operation.
#[unsafe(no_mangle)]
pub extern "C" fn rs_cmd_resize_op(nchar: c_int) -> c_int {
    get_resize_op_impl(nchar) as c_int
}

/// FFI: Check if resize op affects height.
#[unsafe(no_mangle)]
pub extern "C" fn rs_cmd_resize_is_height(op: c_int) -> c_int {
    // Safe conversion since op values are 0-6
    let resize_op = match op {
        0 => ResizeOp::IncHeight,
        1 => ResizeOp::DecHeight,
        2 => ResizeOp::IncWidth,
        3 => ResizeOp::DecWidth,
        4 => ResizeOp::SetHeight,
        5 => ResizeOp::SetWidth,
        _ => ResizeOp::Equalize,
    };
    c_int::from(resize_is_height_impl(resize_op))
}

/// FFI: Count non-floating windows.
#[unsafe(no_mangle)]
pub extern "C" fn rs_cmd_count_windows() -> c_int {
    count_non_floating_windows_impl()
}

/// FFI: Check if only one window.
#[unsafe(no_mangle)]
pub extern "C" fn rs_cmd_only_one_window() -> c_int {
    c_int::from(only_one_window_impl())
}

// =============================================================================
// Exchange/Rotate Helpers
// =============================================================================

extern "C" {
    /// Get w_frame from window.
    fn nvim_win_get_frame(wp: WinHandle) -> *mut crate::Frame;
}

/// Find the target frame for window exchange operation.
///
/// Returns the frame to exchange with based on Prenum:
/// - Prenum > 0: find frame at position Prenum in parent's children
/// - Prenum == 0 and next exists: use next frame
/// - Otherwise: use prev frame
fn find_exchange_target_frame_impl(prenum: c_int) -> *mut crate::Frame {
    unsafe {
        let curwin = nvim_get_curwin();
        if curwin.is_null() {
            return std::ptr::null_mut();
        }

        let cur_frame = nvim_win_get_frame(curwin);
        if cur_frame.is_null() {
            return std::ptr::null_mut();
        }

        let parent = (*cur_frame).fr_parent;
        if parent.is_null() {
            return std::ptr::null_mut();
        }

        if prenum > 0 {
            // Find frame at position prenum
            let mut frp = (*parent).fr_child;
            let mut n = prenum;
            while !frp.is_null() && n > 1 {
                frp = (*frp).fr_next;
                n -= 1;
            }
            frp
        } else if !(*cur_frame).fr_next.is_null() {
            // Swap with next
            (*cur_frame).fr_next
        } else {
            // Swap with prev
            (*cur_frame).fr_prev
        }
    }
}

/// Check if exchange target frame is valid.
///
/// Target must be a leaf frame (not containing sub-frames) and not curwin.
fn is_exchange_target_valid_impl(frp: *const crate::Frame) -> bool {
    if frp.is_null() {
        return false;
    }

    unsafe {
        // Must be a leaf (have a window, not sub-frames)
        let target_win = (*frp).fr_win;
        if target_win.is_null() {
            return false;
        }

        // Can't exchange with self
        let curwin = nvim_get_curwin();
        target_win != curwin
    }
}

/// Check if rotation is possible in current frame's parent.
///
/// All frames in the parent row/col must be leaves (single windows).
fn can_rotate_frames_impl() -> bool {
    unsafe {
        let curwin = nvim_get_curwin();
        if curwin.is_null() {
            return false;
        }

        let cur_frame = nvim_win_get_frame(curwin);
        if cur_frame.is_null() {
            return false;
        }

        let parent = (*cur_frame).fr_parent;
        if parent.is_null() {
            return false;
        }

        // Check all sibling frames
        let mut frp = (*parent).fr_child;
        while !frp.is_null() {
            if (*frp).fr_win.is_null() {
                // Found a frame without a direct window (has sub-frames)
                return false;
            }
            frp = (*frp).fr_next;
        }
        true
    }
}

/// Count frames in current parent for rotation.
fn count_rotate_frames_impl() -> c_int {
    unsafe {
        let curwin = nvim_get_curwin();
        if curwin.is_null() {
            return 0;
        }

        let cur_frame = nvim_win_get_frame(curwin);
        if cur_frame.is_null() {
            return 0;
        }

        let parent = (*cur_frame).fr_parent;
        if parent.is_null() {
            return 1; // Only curwin's frame
        }

        let mut count = 0;
        let mut frp = (*parent).fr_child;
        while !frp.is_null() {
            count += 1;
            frp = (*frp).fr_next;
        }
        count
    }
}

/// FFI: Find exchange target frame.
#[unsafe(no_mangle)]
pub extern "C" fn rs_cmd_exchange_target(prenum: c_int) -> *mut crate::Frame {
    find_exchange_target_frame_impl(prenum)
}

/// FFI: Check if exchange target is valid.
#[unsafe(no_mangle)]
pub extern "C" fn rs_cmd_exchange_valid(frp: *const crate::Frame) -> c_int {
    c_int::from(is_exchange_target_valid_impl(frp))
}

/// FFI: Check if rotation is possible.
#[unsafe(no_mangle)]
pub extern "C" fn rs_cmd_can_rotate() -> c_int {
    c_int::from(can_rotate_frames_impl())
}

/// FFI: Count frames for rotation.
#[unsafe(no_mangle)]
pub extern "C" fn rs_cmd_rotate_frame_count() -> c_int {
    count_rotate_frames_impl()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_classify_split_commands() {
        // 's' = 0x73
        assert!(matches!(classify_win_cmd_impl(0x73), WinCmdCategory::Split));
        // 'v' = 0x76
        assert!(matches!(classify_win_cmd_impl(0x76), WinCmdCategory::Split));
    }

    #[test]
    fn test_classify_close_commands() {
        // 'c' = 0x63
        assert!(matches!(classify_win_cmd_impl(0x63), WinCmdCategory::Close));
        // 'q' = 0x71
        assert!(matches!(classify_win_cmd_impl(0x71), WinCmdCategory::Close));
    }

    #[test]
    fn test_classify_navigate_commands() {
        // 'h' = 0x68
        assert!(matches!(
            classify_win_cmd_impl(0x68),
            WinCmdCategory::Navigate
        ));
        // 'j' = 0x6A
        assert!(matches!(
            classify_win_cmd_impl(0x6A),
            WinCmdCategory::Navigate
        ));
    }

    #[test]
    fn test_classify_resize_commands() {
        // '+' = 0x2B
        assert!(matches!(
            classify_win_cmd_impl(0x2B),
            WinCmdCategory::Resize
        ));
        // '=' = 0x3D
        assert!(matches!(
            classify_win_cmd_impl(0x3D),
            WinCmdCategory::Resize
        ));
    }

    #[test]
    fn test_effective_prenum() {
        assert_eq!(effective_prenum_impl(0), 1);
        assert_eq!(effective_prenum_impl(5), 5);
        assert_eq!(effective_prenum_impl(-1), 1);
    }

    #[test]
    fn test_nav_direction() {
        assert!(matches!(get_nav_direction_impl(0x68), NavDirection::Left));
        assert!(matches!(get_nav_direction_impl(0x6C), NavDirection::Right));
    }

    #[test]
    fn test_resize_is_height() {
        assert!(resize_is_height_impl(ResizeOp::IncHeight));
        assert!(resize_is_height_impl(ResizeOp::DecHeight));
        assert!(!resize_is_height_impl(ResizeOp::IncWidth));
        assert!(!resize_is_height_impl(ResizeOp::DecWidth));
    }
}
