//! Window allocation functions.
//!
//! This module provides Rust implementations of window and frame allocation
//! functions from `src/nvim/window.c`.
//!
//! Note: The main allocation functions (`win_alloc`, `win_free`) remain in C
//! due to their complex dependencies on memory management, autocmds, and
//! other subsystems. This module provides helper functions and simpler
//! allocation utilities.

// This module is a placeholder for future allocation-related migrations.
// The complex win_alloc() and win_free() functions have too many dependencies
// on C subsystems to be easily migrated:
// - Memory allocation (xcalloc, xfree)
// - Window handles map (pmap_put, pmap_del)
// - Grid allocation
// - Variable dictionaries
// - Autocmd blocking
// - Fold initialization
// - Argument lists
// - Buffer lists

// The frame list operations (frame_append, frame_insert, frame_remove) are
// already migrated and exported from lib.rs.
