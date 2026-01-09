//! Window deallocation functions.
//!
//! This module provides Rust implementations of window and frame deallocation
//! functions from `src/nvim/window.c`.
//!
//! Note: The main deallocation functions (`win_free`, `win_free_mem`, `win_free_all`)
//! remain in C due to their complex dependencies on memory management, autocmds,
//! buffer iteration, variable cleanup, and other subsystems.

// This module is a placeholder for future deallocation-related migrations.
// The complex win_free() and related functions have too many dependencies
// on C subsystems to be easily migrated:
// - Memory deallocation (xfree)
// - Window handles map (pmap_del)
// - Variable dictionaries (vars_clear, unref_var_dict)
// - Fold management (clearFolding)
// - Argument lists (alist_unlink)
// - Buffer iteration (FOR_ALL_BUFFERS)
// - Autocmd blocking (block_autocmds, unblock_autocmds)
// - Jumplist management (free_jumplist)
// - Quickfix (qf_free_all)
// - Grid management (win_free_grid)

// The snapshot functions (clear_snapshot, clear_snapshot_rec) could potentially
// be migrated but require access to tabpage tp_snapshot fields and frame memory
// deallocation which is still managed by the C side.

// See alloc.rs for similar notes about win_alloc().
