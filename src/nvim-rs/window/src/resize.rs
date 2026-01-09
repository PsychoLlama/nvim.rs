//! Window resizing and scrolling functions.
//!
//! This module provides Rust implementations of window size management
//! functions from `src/nvim/window.c`.
//!
//! These functions handle window dimensions, scroll positions, and size changes.

// This module is a placeholder for future resize-related migrations.
// Functions to migrate:
// - win_new_width(): Set new window width
// - win_new_height(): Set new window height
// - win_new_height_fraction(): Set height with scroll fraction preserved
// - scroll_to_fraction(): Scroll to saved fraction position
// - win_set_inner_size(): Set internal window dimensions
// - win_resize_vert(): Resize window vertically
// - win_resize_hor(): Resize window horizontally
// - win_comp_scroll(): Compute scroll-related values
// - command_height(): Handle command line height changes
// - last_status(): Handle status line display
// - set_fraction(): Save cursor line fraction
//
// Many of these functions are good candidates for migration as they
// involve arithmetic and state updates without complex C dependencies.
