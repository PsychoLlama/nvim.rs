//! Menu system for Neovim
//!
//! This crate provides Rust implementations of menu-related functions from
//! `src/nvim/menu.c`. It uses an opaque handle pattern where `vimmenu_T*`
//! pointers are treated as opaque handles, with field access done through
//! C accessor functions.
//!
//! # Modules
//!
//! - [`classify`]: Menu name classification (popup, toolbar, winbar, etc.)
//! - [`handle`]: Opaque handle types for menu structures
//! - [`hidden`]: Hidden menu detection
//! - [`traverse`]: Tree traversal helpers

#![allow(unsafe_code)] // FFI requires unsafe
#![allow(clippy::missing_const_for_fn)] // extern "C" functions cannot be const

pub mod classify;
pub mod create;
pub mod delete;
pub mod handle;
pub mod hidden;
pub mod lookup;
pub mod path;
pub mod traverse;

// Re-exports for convenience
pub use classify::{
    rs_menu_is_menubar, rs_menu_is_popup, rs_menu_is_separator, rs_menu_is_toolbar,
    rs_menu_is_winbar,
};
pub use create::{
    rs_compute_new_menu_modes, rs_find_menu_insert_point, rs_menu_item_exists,
    rs_menu_path_needs_parents, rs_validate_menu_name,
};
pub use delete::{
    rs_menu_has_modes, rs_menu_is_empty, rs_menu_remaining_modes, rs_recalculate_parent_modes,
    rs_should_delete_menu, rs_should_free_tip,
};
pub use handle::VimMenuHandle;
pub use hidden::rs_menu_is_hidden;
pub use lookup::{
    rs_count_menu_siblings, rs_find_menu_by_name, rs_find_menu_sibling, rs_menu_exists,
    rs_menu_path_depth, MenuSearchResult,
};
pub use path::{rs_menu_name_equal, rs_menu_namelen, rs_menu_text, MenuTextResult};

/// Hidden menu character (']')
pub const MNU_HIDDEN_CHAR: u8 = b']';

/// Menu modes (matching C definitions in menu_defs.h)
pub mod menu_modes {
    use std::ffi::c_int;

    /// Menu index for normal mode.
    pub const MENU_INDEX_NORMAL: c_int = 0;
    /// Menu index for visual mode.
    pub const MENU_INDEX_VISUAL: c_int = 1;
    /// Menu index for select mode.
    pub const MENU_INDEX_SELECT: c_int = 2;
    /// Menu index for operator-pending mode.
    pub const MENU_INDEX_OP_PENDING: c_int = 3;
    /// Menu index for insert mode.
    pub const MENU_INDEX_INSERT: c_int = 4;
    /// Menu index for command-line mode.
    pub const MENU_INDEX_CMDLINE: c_int = 5;
    /// Menu index for terminal mode.
    pub const MENU_INDEX_TERMINAL: c_int = 6;
    /// Menu index for tooltip.
    pub const MENU_INDEX_TIP: c_int = 7;
    /// Number of menu modes.
    pub const MENU_MODES: c_int = 8;

    /// Normal mode flag.
    pub const MENU_NORMAL_MODE: c_int = 1 << MENU_INDEX_NORMAL;
    /// Visual mode flag.
    pub const MENU_VISUAL_MODE: c_int = 1 << MENU_INDEX_VISUAL;
    /// Select mode flag.
    pub const MENU_SELECT_MODE: c_int = 1 << MENU_INDEX_SELECT;
    /// Operator-pending mode flag.
    pub const MENU_OP_PENDING_MODE: c_int = 1 << MENU_INDEX_OP_PENDING;
    /// Insert mode flag.
    pub const MENU_INSERT_MODE: c_int = 1 << MENU_INDEX_INSERT;
    /// Command-line mode flag.
    pub const MENU_CMDLINE_MODE: c_int = 1 << MENU_INDEX_CMDLINE;
    /// Terminal mode flag.
    pub const MENU_TERMINAL_MODE: c_int = 1 << MENU_INDEX_TERMINAL;
    /// Tooltip mode flag.
    pub const MENU_TIP_MODE: c_int = 1 << MENU_INDEX_TIP;
    /// All modes except tooltip.
    pub const MENU_ALL_MODES: c_int = (1 << MENU_INDEX_TIP) - 1;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mnu_hidden_char_constant() {
        // Verify MNU_HIDDEN_CHAR matches C definition
        assert_eq!(MNU_HIDDEN_CHAR, b']');
    }

    #[test]
    fn test_menu_modes_consistency() {
        use menu_modes::*;

        // Verify mode flags are powers of 2
        assert_eq!(MENU_NORMAL_MODE, 1);
        assert_eq!(MENU_VISUAL_MODE, 2);
        assert_eq!(MENU_SELECT_MODE, 4);
        assert_eq!(MENU_OP_PENDING_MODE, 8);
        assert_eq!(MENU_INSERT_MODE, 16);
        assert_eq!(MENU_CMDLINE_MODE, 32);
        assert_eq!(MENU_TERMINAL_MODE, 64);
        assert_eq!(MENU_TIP_MODE, 128);

        // All modes should be sum of all mode flags
        assert_eq!(MENU_ALL_MODES, 127);
    }
}
