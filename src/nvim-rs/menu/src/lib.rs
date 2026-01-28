//! Menu system for Neovim
//!
//! This crate provides Rust implementations of menu-related functions from
//! `src/nvim/menu.c`. It uses an opaque handle pattern where `vimmenu_T*`
//! pointers are treated as opaque handles, with field access done through
//! C accessor functions.
//!
//! # Architecture
//!
//! The menu system is a tree structure where each menu can have children
//! (submenus) and siblings (next menu at the same level). Menus support
//! multiple modes (Normal, Visual, Insert, etc.) with different actions
//! per mode.
//!
//! ## Opaque Handle Pattern
//!
//! All menu operations use [`VimMenuHandle`], which wraps a `*mut c_void`
//! pointer to the C `vimmenu_T` structure. Field access is done through
//! C accessor functions defined in `menu.c`:
//!
//! - `nvim_menu_get_modes()` - Get mode flags
//! - `nvim_menu_get_enabled()` - Get enabled flags
//! - `nvim_menu_get_name()` - Get menu name
//! - `nvim_menu_get_children()` - Get first child menu
//! - `nvim_menu_get_next()` - Get next sibling menu
//!
//! # Modules
//!
//! - [`classify`]: Menu name classification (popup, toolbar, winbar, etc.)
//! - [`commands`]: Ex command mode parsing (`:menu`, `:nmenu`, etc.)
//! - [`completion`]: Wildmenu completion utilities
//! - [`create`]: Menu creation helpers
//! - [`delete`]: Menu deletion helpers
//! - [`execute`]: Menu execution and mode utilities
//! - [`handle`]: Opaque handle types for menu structures
//! - [`hidden`]: Hidden menu detection
//! - [`lookup`]: Menu lookup and search
//! - [`path`]: Menu path parsing and name comparison
//! - [`popup`]: Popup menu utilities (right-click context menus)
//! - [`traverse`]: Tree traversal helpers
//!
//! # Menu Modes
//!
//! Menus can have different actions for different Vim modes. The mode
//! constants in [`menu_modes`] define both indices (0-7) and flags (bitmasks):
//!
//! | Mode      | Index | Flag |
//! |-----------|-------|------|
//! | Normal    | 0     | 1    |
//! | Visual    | 1     | 2    |
//! | Select    | 2     | 4    |
//! | Op-pending| 3     | 8    |
//! | Insert    | 4     | 16   |
//! | Cmdline   | 5     | 32   |
//! | Terminal  | 6     | 64   |
//! | Tip       | 7     | 128  |

#![allow(unsafe_code)] // FFI requires unsafe
#![allow(clippy::missing_const_for_fn)] // extern "C" functions cannot be const

pub mod classify;
pub mod commands;
pub mod completion;
pub mod create;
pub mod delete;
pub mod execute;
pub mod handle;
pub mod hidden;
pub mod lookup;
pub mod path;
pub mod popup;
pub mod traverse;

// Re-exports for convenience
pub use classify::{
    rs_menu_is_menubar, rs_menu_is_popup, rs_menu_is_separator, rs_menu_is_toolbar,
    rs_menu_is_winbar,
};
pub use commands::{rs_get_menu_cmd_modes, rs_get_menu_mode_str, MenuCmdResult};
pub use completion::{
    rs_count_completable_menus, rs_menu_completion_context, rs_menu_is_completable,
    rs_should_expand_all_modes, MenuCompletionContext,
};
pub use create::{
    rs_compute_new_menu_modes, rs_find_menu_insert_point, rs_menu_item_exists,
    rs_menu_path_needs_parents, rs_validate_menu_name,
};
pub use delete::{
    rs_menu_has_modes, rs_menu_is_empty, rs_menu_remaining_modes, rs_recalculate_parent_modes,
    rs_should_delete_menu, rs_should_free_tip,
};
pub use execute::{
    rs_get_menu_string, rs_menu_has_string_for_mode, rs_menu_is_enabled_for_mode,
    rs_menu_mode_name, rs_mode_flag_to_index, rs_mode_index_to_flag,
};
pub use handle::VimMenuHandle;
pub use hidden::rs_menu_is_hidden;
pub use lookup::{
    rs_count_menu_siblings, rs_find_menu_by_name, rs_find_menu_sibling, rs_menu_exists,
    rs_menu_path_depth, MenuSearchResult,
};
pub use path::{rs_menu_name_equal, rs_menu_namelen, rs_menu_text, MenuTextResult};
pub use popup::{
    rs_find_popup_menu, rs_get_menu_mode_chars, rs_get_menu_mode_chars_len,
    rs_is_popup_menu_for_mode, rs_popup_menu_should_show, rs_popup_mode_name_len, MENU_MODE_CHARS,
};

/// Hidden menu character (']')
pub const MNU_HIDDEN_CHAR: u8 = b']';

/// Menu modes (matching C definitions in menu_defs.h)
pub mod menu_modes {
    use std::ffi::c_int;

    /// Invalid menu index.
    pub const MENU_INDEX_INVALID: c_int = -1;
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
