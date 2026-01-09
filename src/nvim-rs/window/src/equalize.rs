//! Window equalization functions.
//!
//! This module provides Rust implementations of window equalization functions
//! from `src/nvim/window.c`.
//!
//! These functions distribute window sizes evenly and compute frame positions.

// This module is a placeholder for future equalization-related migrations.
// Functions to migrate:
// - win_equal(): Equalize window sizes
// - win_equal_rec(): Recursive equalization helper
// - win_comp_pos(): Compute window positions
// - frame_comp_pos(): Compute frame positions
// - frame_setheight(): Set frame height with constraints
// - frame_setwidth(): Set frame width with constraints
//
// These functions are good candidates for migration as they primarily
// involve frame tree traversal and arithmetic operations.
//
// See frame.rs for related frame operations that have been migrated
// (frame_fixed_height, frame_fixed_width, frame_check_height, frame_check_width).
