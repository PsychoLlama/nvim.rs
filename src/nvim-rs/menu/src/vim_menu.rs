//! `#[repr(C)]` mirror of `vimmenu_T` from `menu_defs.h`.
//!
//! This struct must match the C `struct VimMenu` layout exactly.
//! Layout is verified by `_Static_assert` checks in `menu.c`.

use std::ffi::{c_char, c_int};

/// Number of menu modes (MENU_MODES constant from menu_defs.h).
pub const MENU_MODES: usize = 8;

/// Rust mirror of `vimmenu_T` / `struct VimMenu`.
///
/// # Safety
/// This struct must match the C `struct VimMenu` layout exactly.
/// The layout is validated at compile time via `_Static_assert` in `menu.c`.
#[repr(C)]
pub struct VimMenu {
    /// Which modes this menu is visible for.
    pub modes: c_int,
    /// For which modes the menu is enabled.
    pub enabled: c_int,
    /// Name of menu, possibly translated.
    pub name: *mut c_char,
    /// Displayed Name ("name" without '&').
    pub dname: *mut c_char,
    /// "name" untranslated, NULL when was not translated.
    pub en_name: *mut c_char,
    /// NULL when "dname" untranslated.
    pub en_dname: *mut c_char,
    /// Mnemonic key (after '&').
    pub mnemonic: c_int,
    /// Accelerator text (after TAB).
    pub actext: *mut c_char,
    /// Menu order priority.
    pub priority: c_int,
    /// Mapped string for each mode.
    pub strings: [*mut c_char; MENU_MODES],
    /// A REMAP_VALUES flag for each mode.
    pub noremap: [c_int; MENU_MODES],
    /// A silent flag for each mode.
    pub silent: [bool; MENU_MODES],
    /// Children of sub-menu.
    pub children: *mut VimMenu,
    /// Parent of menu.
    pub parent: *mut VimMenu,
    /// Next item in menu.
    pub next: *mut VimMenu,
}
