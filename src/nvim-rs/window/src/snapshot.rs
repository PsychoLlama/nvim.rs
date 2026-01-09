//! Window layout snapshot functions.
//!
//! This module provides Rust implementations of window layout snapshot
//! functions from `src/nvim/window.c`.
//!
//! Snapshots capture the window layout state for later restoration,
//! used primarily for help window handling.

// This module is a placeholder for future snapshot-related migrations.
// Functions to migrate:
// - make_snapshot(): Create a snapshot of current layout
// - make_snapshot_rec(): Recursive snapshot creation
// - restore_snapshot(): Restore a previously saved layout
// - restore_snapshot_rec(): Recursive snapshot restoration
// - clear_snapshot(): Clear a saved snapshot
// - clear_snapshot_rec(): Recursive snapshot clearing
// - check_snapshot_rec(): Validate snapshot matches current layout
// - snapshot_idx(): Get snapshot index
// - match_snapshot(): Check if layout matches snapshot
//
// These functions involve frame tree allocation/deallocation and
// matching algorithms. The recursive helpers could be migrated once
// proper frame memory management is available from Rust.
