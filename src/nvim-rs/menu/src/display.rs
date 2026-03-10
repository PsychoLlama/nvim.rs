//! Menu display functions.
//!
//! This module provides functions for displaying menu information,
//! including the `:menu` command output.

use std::ffi::{c_char, c_int};

use crate::handle::VimMenuHandle;
use crate::hidden::rs_menu_is_hidden;
use crate::menu_modes::MENU_MODES;
use crate::vim_menu::VimMenu;

/// Highlight group IDs (from highlight_defs.h).
const HLF_D: c_int = 5; // directories in CTRL-D listing
const HLF_8: c_int = 1; // Meta & special keys

/// Remap values for display.
const REMAP_NONE: c_int = -1;
const REMAP_SCRIPT: c_int = -2;

extern "C" {
    fn nvim_menu_get_got_int() -> c_int;
    static mut root_menu: *mut VimMenu;

    // Message output functions (already exist in C)
    fn msg_putchar(c: c_int);
    fn msg_puts(s: *const c_char);
    fn msg_outnum(n: c_int);
    fn msg_outtrans(s: *const c_char, hl_id: c_int, hist: bool) -> c_int;
    fn msg_outtrans_special(s: *const c_char, from: bool, maxlen: c_int) -> c_int;
    fn msg_puts_title(s: *const c_char);
    fn msg_puts_hl(s: *const c_char, hl_id: c_int, hist: bool);

    fn nvim_gettext(s: *const c_char) -> *const c_char;
    fn rs_find_menu(menu: VimMenuHandle, name: *mut c_char, modes: c_int) -> VimMenuHandle;
}

/// The menu_mode_chars static strings for display.
const MENU_MODE_CHARS: [*const c_char; 8] = [
    c"n".as_ptr(),
    c"v".as_ptr(),
    c"s".as_ptr(),
    c"o".as_ptr(),
    c"i".as_ptr(),
    c"c".as_ptr(),
    c"tl".as_ptr(),
    c"t".as_ptr(),
];

/// OK return value.
const OK: c_int = 1;
/// FAIL return value.
const FAIL: c_int = 0;

/// Show the mapping associated with a menu item or hierarchy in a sub-menu.
///
/// This is the Rust implementation of C `show_menus()`.
///
/// # Safety
/// `path_name` must be a valid pointer to a mutable NUL-terminated C string.
#[no_mangle]
pub unsafe extern "C" fn rs_show_menus(path_name: *mut c_char, modes: c_int) -> c_int {
    let mut menu = VimMenuHandle::null();

    if !path_name.is_null() && unsafe { *path_name } != 0 {
        // First, find the (sub)menu with the given name
        let root = VimMenuHandle::from_ptr(unsafe { root_menu });
        menu = unsafe { rs_find_menu(root, path_name, modes) };
        if menu.is_null() {
            return FAIL;
        }
    }

    // Now we have found the matching menu, and we list the mappings.
    unsafe {
        msg_puts_title(nvim_gettext(c"\n--- Menus ---".as_ptr()));
    }
    unsafe { rs_show_menus_recursive(menu, modes, 0) };

    OK
}

/// Recursively show the mappings associated with the menus under the given one.
///
/// This is the Rust implementation of C `show_menus_recursive()`.
///
/// # Safety
/// `menu` must be a valid menu handle or null.
#[no_mangle]
pub unsafe extern "C" fn rs_show_menus_recursive(menu: VimMenuHandle, modes: c_int, depth: c_int) {
    if !menu.is_null() && (menu.modes() & modes) == 0 {
        return;
    }

    if !menu.is_null() {
        unsafe { msg_putchar(b'\n' as c_int) };
        if unsafe { nvim_menu_get_got_int() } != 0 {
            return;
        }
        for _ in 0..depth {
            unsafe { msg_puts(c"  ".as_ptr()) };
        }
        if menu.priority() != 0 {
            unsafe { msg_outnum(menu.priority()) };
            unsafe { msg_puts(c" ".as_ptr()) };
        }
        // Same highlighting as for directories!?
        unsafe { msg_outtrans(menu.name(), HLF_D, false) };
    }

    if !menu.is_null() && menu.children().is_null() {
        // Leaf menu: show mappings for each mode
        let m = unsafe { &*menu.as_ptr() };
        for bit in 0..MENU_MODES {
            if (menu.modes() & modes & (1 << bit)) != 0 {
                unsafe { msg_putchar(b'\n' as c_int) };
                if unsafe { nvim_menu_get_got_int() } != 0 {
                    return;
                }
                for _ in 0..(depth + 2) {
                    unsafe { msg_puts(c"  ".as_ptr()) };
                }
                unsafe { msg_puts(MENU_MODE_CHARS[bit as usize]) };

                let noremap = m.noremap[bit as usize];
                if noremap == REMAP_NONE {
                    unsafe { msg_putchar(b'*' as c_int) };
                } else if noremap == REMAP_SCRIPT {
                    unsafe { msg_putchar(b'&' as c_int) };
                } else {
                    unsafe { msg_putchar(b' ' as c_int) };
                }

                if m.silent[bit as usize] {
                    unsafe { msg_putchar(b's' as c_int) };
                } else {
                    unsafe { msg_putchar(b' ' as c_int) };
                }

                if (menu.modes() & menu.enabled() & (1 << bit)) == 0 {
                    unsafe { msg_putchar(b'-' as c_int) };
                } else {
                    unsafe { msg_putchar(b' ' as c_int) };
                }
                unsafe { msg_puts(c" ".as_ptr()) };

                let str_ptr = m.strings[bit as usize];
                if !str_ptr.is_null() && unsafe { *str_ptr } == 0 {
                    unsafe { msg_puts_hl(c"<Nop>".as_ptr(), HLF_8, false) };
                } else if !str_ptr.is_null() {
                    unsafe { msg_outtrans_special(str_ptr, false, 0) };
                }
            }
        }
    } else {
        // Non-leaf or null: recurse into children
        let start_menu;
        let actual_depth;
        if menu.is_null() {
            start_menu = VimMenuHandle::from_ptr(unsafe { root_menu });
            actual_depth = depth - 1;
        } else {
            start_menu = menu.children();
            actual_depth = depth;
        }

        // recursively show all children. Skip PopUp[nvoci].
        let mut child = start_menu;
        while !child.is_null() && unsafe { nvim_menu_get_got_int() } == 0 {
            if !unsafe { rs_menu_is_hidden(child.dname()) } {
                unsafe { rs_show_menus_recursive(child, modes, actual_depth + 1) };
            }
            child = child.next();
        }
    }
}
