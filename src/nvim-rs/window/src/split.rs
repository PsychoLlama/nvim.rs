//! Window splitting functions.
//!
//! This module provides Rust implementations of window splitting functions
//! from `src/nvim/window.c`.
//!
//! Note: The main splitting functions (`win_split`, `win_split_ins`) remain in C
//! due to their complexity (~500 lines for `win_split_ins`) and dependencies on:
//! - Frame tree manipulation
//! - Window allocation
//! - Autocmd handling
//! - Size calculation
//! - Focus management
//!
//! These functions would benefit from refactoring into smaller pieces before
//! migration to Rust.

// This module is a placeholder for future split-related migrations.
// The complex win_split_ins() function has too many responsibilities:
// - Frame creation and linking
// - Window allocation
// - Snapshot management
// - Size computation and distribution
// - Autocmd triggering (WinNew, WinEnter)
// - Focus switching
// - Frame flattening
//
// A potential migration strategy would be:
// 1. Extract frame manipulation into reusable helpers
// 2. Extract size computation into dedicated functions
// 3. Keep autocmd-related code in C
// 4. Migrate pure frame tree logic to Rust

// The win_init() and win_init_some() functions copy window settings between
// windows and have extensive field access that would require many accessor
// functions.

// See frame.rs for frame tree operations that have been successfully migrated.
