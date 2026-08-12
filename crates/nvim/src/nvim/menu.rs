//! Menus: `:menu` and the tree it builds.
//!
//! Carved by the stage:
//!
//! | child | what |
//! | --- | --- |
//! | [`define`] | `:menu` and `add_menu_path()` |
//! | [`tree`] | the `vimmenu_T` tree -- find, list, dump, remove, free |
//! | [`complete`] | command-line completion of a menu path |
//! | [`name`] | names as text: path components, mode letters, accelerators |
//! | [`exec`] | `:emenu`, `:popup` and running a right-hand side |
//! | [`info`] | `:menutranslate` and `menu_info()` |
//!
//! What stays here is the mode alphabet the six share (`MENU_*_MODE`,
//! `MENU_INDEX_*`, `menu_mode_chars`), the root-menu lookup, the
//! `menus_locked` guard, the `menu_is_*` predicates that classify a node, and
//! `get_menu_mode()`, which answers "which mode is the editor in?" for the
//! executing side.
//!
//! Original: `src/nvim/menu.c`, Vim/Neovim, Vim license.

#![deny(unsafe_op_in_unsafe_fn)]

use crate::src::nvim::autocmd::{EVENT_MENUPOPUP, apply_autocmds};
use crate::src::nvim::global_cell::GlobalCell;
use crate::src::nvim::main::{
    State, VIsual_active, VIsual_select, c_bytes, curbuf, e_cannot_change_menus_while_listing,
    finish_op, root_menu,
};
use crate::src::nvim::message::emsg;
use crate::src::nvim::os::libc::{gettext, strlen, strncmp};
use crate::src::nvim::popupmenu::pum_show_popupmenu;
use crate::src::nvim::state::{
    MODE_ASKMORE, MODE_CMDLINE, MODE_HITRETURN, MODE_INSERT, MODE_LANGMAP, MODE_NORMAL,
    MODE_TERMINAL,
};
use crate::src::nvim::types::{
    AlignTextPos, RemapValues, WinSplit, WinStyle, garray_T, size_t, vimmenu_T,
};

// The carve of the transpiled module; see each child's docs.
mod complete;
mod define;
mod exec;
mod info;
mod name;
mod tree;

pub use self::complete::*;
pub use self::define::*;
pub use self::exec::*;
pub use self::info::*;
pub use self::name::*;
pub use self::tree::*;

pub const kAlignLeft: AlignTextPos = 0;
pub const kWinStyleUnused: WinStyle = 0;
pub const kWinSplitLeft: WinSplit = 0;
pub type C2Rust_Unnamed_14 = ::core::ffi::c_int;
pub const EXPAND_MENUNAMES: C2Rust_Unnamed_14 = 21;
pub const EXPAND_MENUS: C2Rust_Unnamed_14 = 11;
pub const EXPAND_NOTHING: C2Rust_Unnamed_14 = 0;
pub const EXPAND_UNSUCCESSFUL: C2Rust_Unnamed_14 = -2;
pub const REMAP_SCRIPT: RemapValues = -2;
pub const REMAP_NONE: RemapValues = -1;
pub const REMAP_YES: RemapValues = 0;
pub type C2Rust_Unnamed_16 = ::core::ffi::c_int;
pub const MENU_MODES: C2Rust_Unnamed_16 = 8;
pub const MENU_INDEX_TIP: C2Rust_Unnamed_16 = 7;
pub const MENU_INDEX_TERMINAL: C2Rust_Unnamed_16 = 6;
pub const MENU_INDEX_CMDLINE: C2Rust_Unnamed_16 = 5;
pub const MENU_INDEX_INSERT: C2Rust_Unnamed_16 = 4;
pub const MENU_INDEX_OP_PENDING: C2Rust_Unnamed_16 = 3;
pub const MENU_INDEX_SELECT: C2Rust_Unnamed_16 = 2;
pub const MENU_INDEX_VISUAL: C2Rust_Unnamed_16 = 1;
pub const MENU_INDEX_NORMAL: C2Rust_Unnamed_16 = 0;
pub const MENU_INDEX_INVALID: C2Rust_Unnamed_16 = -1;
pub type C2Rust_Unnamed_17 = ::core::ffi::c_uint;
pub const MENU_ALL_MODES: C2Rust_Unnamed_17 = 127;
pub const MENU_TIP_MODE: C2Rust_Unnamed_17 = 128;
pub const MENU_TERMINAL_MODE: C2Rust_Unnamed_17 = 64;
pub const MENU_CMDLINE_MODE: C2Rust_Unnamed_17 = 32;
pub const MENU_INSERT_MODE: C2Rust_Unnamed_17 = 16;
pub const MENU_OP_PENDING_MODE: C2Rust_Unnamed_17 = 8;
pub const MENU_SELECT_MODE: C2Rust_Unnamed_17 = 4;
pub const MENU_VISUAL_MODE: C2Rust_Unnamed_17 = 2;
pub const MENU_NORMAL_MODE: C2Rust_Unnamed_17 = 1;
pub type C2Rust_Unnamed_19 = ::core::ffi::c_uint;
pub const REPTERM_DO_LT: C2Rust_Unnamed_19 = 2;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct menutrans_T {
    pub from: *mut ::core::ffi::c_char,
    pub from_noamp: *mut ::core::ffi::c_char,
    pub to: *mut ::core::ffi::c_char,
}
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const NUL: ::core::ffi::c_int = '\0' as ::core::ffi::c_int;
pub const TAB: ::core::ffi::c_int = '\t' as ::core::ffi::c_int;
pub const GA_EMPTY_INIT_VALUE: garray_T = garray_T {
    ga_len: 0 as ::core::ffi::c_int,
    ga_maxlen: 0 as ::core::ffi::c_int,
    ga_itemsize: 0 as ::core::ffi::c_int,
    ga_growsize: 1 as ::core::ffi::c_int,
    ga_data: NULL,
};
pub const OK: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const FAIL: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const MNU_HIDDEN_CHAR: ::core::ffi::c_int = ']' as ::core::ffi::c_int;
pub const MENUDEPTH: ::core::ffi::c_int = 10 as ::core::ffi::c_int;
static menus_locked: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0 as ::core::ffi::c_int);
static menu_mode_chars: GlobalCell<[*mut ::core::ffi::c_char; 8]> = GlobalCell::new([
    c"n".as_ptr() as *mut ::core::ffi::c_char,
    c"v".as_ptr() as *mut ::core::ffi::c_char,
    c"s".as_ptr() as *mut ::core::ffi::c_char,
    c"o".as_ptr() as *mut ::core::ffi::c_char,
    c"i".as_ptr() as *mut ::core::ffi::c_char,
    c"c".as_ptr() as *mut ::core::ffi::c_char,
    c"tl".as_ptr() as *mut ::core::ffi::c_char,
    c"t".as_ptr() as *mut ::core::ffi::c_char,
]);
static e_notsubmenu: [::core::ffi::c_char; 45] =
    c_bytes(b"E327: Part of menu-item path is not sub-menu\0");
static e_nomenu: [::core::ffi::c_char; 19] = c_bytes(b"E329: No menu \"%s\"\0");
unsafe extern "C" fn menu_is_winbar(name: *const ::core::ffi::c_char) -> bool {
    unsafe {
        return strncmp(name, c"WinBar".as_ptr(), 6 as size_t) == 0 as ::core::ffi::c_int;
    }
}
unsafe extern "C" fn get_root_menu(_name: *const ::core::ffi::c_char) -> *mut *mut vimmenu_T {
    return root_menu.ptr();
}
unsafe extern "C" fn is_menus_locked() -> ::core::ffi::c_int {
    unsafe {
        if menus_locked.get() > 0 as ::core::ffi::c_int {
            emsg(gettext(
                &raw const e_cannot_change_menus_while_listing as *const ::core::ffi::c_char,
            ));
            return true_0;
        }
        return false_0;
    }
}
pub unsafe extern "C" fn menu_is_menubar(name: *const ::core::ffi::c_char) -> bool {
    unsafe {
        return !menu_is_popup(name)
            && !menu_is_toolbar(name)
            && !menu_is_winbar(name)
            && *name as ::core::ffi::c_int != MNU_HIDDEN_CHAR;
    }
}
pub unsafe extern "C" fn menu_is_popup(name: *const ::core::ffi::c_char) -> bool {
    unsafe {
        return strncmp(name, c"PopUp".as_ptr(), 5 as size_t) == 0 as ::core::ffi::c_int;
    }
}
pub unsafe extern "C" fn menu_is_toolbar(name: *const ::core::ffi::c_char) -> bool {
    unsafe {
        return strncmp(name, c"ToolBar".as_ptr(), 7 as size_t) == 0 as ::core::ffi::c_int;
    }
}
pub unsafe extern "C" fn menu_is_separator(mut name: *mut ::core::ffi::c_char) -> bool {
    unsafe {
        return *name.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
            == '-' as ::core::ffi::c_int
            && *name.add(strlen(name).wrapping_sub(1 as size_t)) as ::core::ffi::c_int
                == '-' as ::core::ffi::c_int;
    }
}
unsafe extern "C" fn menu_is_hidden(mut name: *mut ::core::ffi::c_char) -> bool {
    unsafe {
        return *name.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
            == MNU_HIDDEN_CHAR
            || menu_is_popup(name) as ::core::ffi::c_int != 0
                && *name.offset(5 as ::core::ffi::c_int as isize) as ::core::ffi::c_int != NUL;
    }
}
unsafe extern "C" fn get_menu_mode() -> ::core::ffi::c_int {
    if State.get() & MODE_TERMINAL != 0 {
        return MENU_INDEX_TERMINAL as ::core::ffi::c_int;
    }
    if VIsual_active.get() {
        if VIsual_select.get() {
            return MENU_INDEX_SELECT as ::core::ffi::c_int;
        }
        return MENU_INDEX_VISUAL as ::core::ffi::c_int;
    }
    if State.get() & MODE_INSERT != 0 {
        return MENU_INDEX_INSERT as ::core::ffi::c_int;
    }
    if State.get() & MODE_CMDLINE != 0
        || State.get() == MODE_ASKMORE
        || State.get() == MODE_HITRETURN
    {
        return MENU_INDEX_CMDLINE as ::core::ffi::c_int;
    }
    if finish_op.get() {
        return MENU_INDEX_OP_PENDING as ::core::ffi::c_int;
    }
    if State.get() & MODE_NORMAL != 0 {
        return MENU_INDEX_NORMAL as ::core::ffi::c_int;
    }
    if State.get() & MODE_LANGMAP != 0 {
        return MENU_INDEX_INSERT as ::core::ffi::c_int;
    }
    return MENU_INDEX_INVALID as ::core::ffi::c_int;
}
pub unsafe extern "C" fn get_menu_mode_flag() -> ::core::ffi::c_int {
    unsafe {
        let mut mode: ::core::ffi::c_int = get_menu_mode();
        if mode == MENU_INDEX_INVALID as ::core::ffi::c_int {
            return 0 as ::core::ffi::c_int;
        }
        return (1 as ::core::ffi::c_int) << mode;
    }
}
pub unsafe extern "C" fn show_popupmenu() {
    unsafe {
        let mut menu_mode: ::core::ffi::c_int = get_menu_mode();
        if menu_mode == MENU_INDEX_INVALID as ::core::ffi::c_int {
            return;
        }
        let mut mode: *mut ::core::ffi::c_char = (*menu_mode_chars.ptr())[menu_mode as usize];
        let mut mode_len: size_t = strlen(mode);
        apply_autocmds(
            EVENT_MENUPOPUP,
            mode,
            ::core::ptr::null_mut::<::core::ffi::c_char>(),
            false_0 != 0,
            curbuf.get(),
        );
        let mut menu: *mut vimmenu_T = ::core::ptr::null_mut::<vimmenu_T>();
        menu = root_menu.get();
        while !menu.is_null() {
            if strncmp(c"PopUp".as_ptr(), (*menu).name, 5 as size_t) == 0 as ::core::ffi::c_int
                && strncmp(
                    (*menu).name.offset(5 as ::core::ffi::c_int as isize),
                    mode,
                    mode_len,
                ) == 0 as ::core::ffi::c_int
            {
                break;
            }
            menu = (*menu).next;
        }
        if menu.is_null() || (*menu).children.is_null() {
            return;
        }
        pum_show_popupmenu(menu);
    }
}
pub const true_0: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const false_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
