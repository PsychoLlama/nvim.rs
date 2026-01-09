//! Window closing functions.
//!
//! This module provides Rust implementations of window closing functions
//! from `src/nvim/window.c`.
//!
//! Note: The main closing functions (`win_close`, `win_close_othertab`) remain in C
//! due to their complexity (~275 lines for `win_close`) and dependencies on:
//! - Extensive autocmd handling (WinLeave, WinClosed, BufLeave, etc.)
//! - Buffer management
//! - Tabpage management
//! - Memory deallocation
//! - Layout restoration

// This module is a placeholder for future close-related migrations.
// The complex win_close() function handles many edge cases:
// - Autocmd re-entrancy (window may be freed during autocmd)
// - Buffer hiding/unloading
// - Last window handling
// - Tab page closing
// - Floating window special cases
// - Quickfix window handling
// - Help window snapshot restoration
// - Focus management (finding next window to focus)
//
// Related functions that remain in C:
// - close_windows(): Close all windows except specified
// - close_last_window_tabpage(): Handle last window in tabpage
// - last_window(): Check if only one window
// - one_window(): Check if truly single window (ignoring popups/floats)

// See free.rs for window memory deallocation notes.
// See focus.rs for window navigation that's been migrated.
