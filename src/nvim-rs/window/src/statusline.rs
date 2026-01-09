//! Status line and window bar functions.
//!
//! This module provides Rust implementations of status line and window bar
//! functions from `src/nvim/window.c`.
//!
//! These functions manage the display of status lines, window bars, and tab panels.

// This module is a placeholder for future statusline-related migrations.
// Functions to migrate:
// - stl_connected(): Check if status lines are visually connected
// - frame_add_statusline(): Add status line to frame height calculations
// - frame_add_winbar(): Add window bar to frame height calculations
// - frame_has_statusline(): Check if frame should show status line
// - winbar_height(): Get window bar height
// - last_stl_height(): Get last status line height
// - global_stl_height(): Get global status line height
// - tabpanel_width(): Get tab panel width
// - tabpanel_side(): Get which side tab panel is on
//
// These functions are relatively simple and good candidates for migration
// once the appropriate option accessor functions are available.
