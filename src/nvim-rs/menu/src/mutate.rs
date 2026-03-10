//! Menu tree mutation functions.
//!
//! This module provides functions for modifying the menu tree:
//! freeing menu strings, freeing menu nodes, enabling/disabling menus,
//! removing menus, and adding new menu paths.

use std::ffi::{c_char, c_int, c_void};

use crate::classify::{rs_menu_is_menubar, rs_menu_is_separator};
use crate::handle::VimMenuHandle;
use crate::menu_modes::{MENU_ALL_MODES, MENU_INDEX_TIP, MENU_MODES, MENU_TIP_MODE};
use crate::path::{rs_menu_name_equal, rs_menu_name_skip, rs_menu_text, MenuTextResult};
use crate::translate::rs_menutrans_lookup;
use crate::vim_menu::VimMenu;

// C return codes
const OK: c_int = 1;
const FAIL: c_int = 0;

// Control characters
const CTRL_C: u8 = 3;
const CTRL_BSL: u8 = 28;
const CTRL_O: u8 = 15;
const CTRL_G: u8 = 7;

// Error message strings
const E_NOTSUBMENU: *const c_char = c"E327: Part of menu-item path is not sub-menu".as_ptr();
const E_NOMENU: *const c_char = c"E329: No menu \"%s\"".as_ptr();
const E_MENU_ONLY_EXISTS_IN_ANOTHER_MODE: *const c_char =
    c"E328: Menu only exists in another mode".as_ptr();
const E330_MENU_PATH_MUST_NOT_LEAD_TO_SUBMENU: *const c_char =
    c"E330: Menu path must not lead to a sub-menu".as_ptr();
const E331_MUST_NOT_ADD_DIRECTLY: *const c_char =
    c"E331: Must not add menu items directly to menu bar".as_ptr();
const E332_SEPARATOR_NOT_IN_PATH: *const c_char =
    c"E332: Separator cannot be part of a menu path".as_ptr();
const E792_EMPTY_MENU_NAME: *const c_char = c"E792: Empty menu name".as_ptr();

// Mode flags used in add_menu_path for amenu handling
use crate::menu_modes::{
    MENU_CMDLINE_MODE, MENU_INSERT_MODE, MENU_NORMAL_MODE, MENU_OP_PENDING_MODE, MENU_SELECT_MODE,
    MENU_VISUAL_MODE,
};

extern "C" {
    fn emsg(s: *const c_char) -> bool;
    fn semsg(s: *const c_char, ...) -> bool;
    fn nvim_gettext(s: *const c_char) -> *const c_char;
    fn xfree(ptr: *mut c_void);
    fn xstrdup(s: *const c_char) -> *mut c_char;
    fn nvim_xmalloc(size: usize) -> *mut c_void;
    fn strlen(s: *const c_char) -> usize;
    fn xcalloc(count: usize, size: usize) -> *mut c_void;

    // Translation - call Rust directly rather than bouncing through C

    // Global state
    static sys_menu: bool;

    // Root menu global (still in C)
    static mut root_menu: *mut VimMenu;
}

/// Free the menu->strings[idx], checking if the string is shared.
///
/// If other mode strings point to the same allocation, only NULL out this slot.
/// If this is the last reference, actually xfree the string.
///
/// This is the Rust implementation of C `free_menu_string()`.
///
/// # Safety
/// The `menu` handle must be valid or null.
#[no_mangle]
pub unsafe extern "C" fn rs_free_menu_string(menu: VimMenuHandle, idx: c_int) {
    if menu.is_null() || !(0..MENU_MODES).contains(&idx) {
        return;
    }

    let m = unsafe { &mut *menu.as_ptr() };
    let target = m.strings[idx as usize];
    if target.is_null() {
        return;
    }

    // Count how many slots point to the same string
    let mut count = 0;
    for i in 0..MENU_MODES as usize {
        if m.strings[i] == target {
            count += 1;
        }
    }

    if count == 1 {
        unsafe { xfree(target as *mut c_void) };
    }
    m.strings[idx as usize] = std::ptr::null_mut();
}

/// Free the given menu structure and remove it from the linked list.
///
/// This is the Rust implementation of C `free_menu()`.
///
/// # Safety
/// `menup` must be a valid `vimmenu_T**` pointer.
#[no_mangle]
pub unsafe extern "C" fn rs_free_menu(menup: *mut *mut VimMenu) {
    let menu_ptr = unsafe { *menup };
    if menu_ptr.is_null() {
        return;
    }

    let menu = unsafe { &mut *menu_ptr };

    // Unlink from list: *menup = menu->next
    unsafe { *menup = menu.next };

    // Free all string fields
    unsafe {
        xfree(menu.name as *mut c_void);
        xfree(menu.dname as *mut c_void);
        xfree(menu.en_name as *mut c_void);
        xfree(menu.en_dname as *mut c_void);
        xfree(menu.actext as *mut c_void);
    }

    // Free per-mode strings
    let handle = VimMenuHandle::from_ptr(menu_ptr);
    for i in 0..MENU_MODES {
        unsafe { rs_free_menu_string(handle, i) };
    }

    // Free the struct itself
    unsafe { xfree(menu_ptr as *mut c_void) };
}

/// Recursively enable or disable menus by name.
///
/// This is the Rust implementation of C `menu_enable_recurse()`.
///
/// # Safety
/// All pointers must be valid. `name` is modified by `menu_name_skip`.
#[no_mangle]
pub unsafe extern "C" fn rs_menu_enable_recurse(
    menu: VimMenuHandle,
    name: *mut c_char,
    modes: c_int,
    enable: c_int,
) -> c_int {
    if menu.is_null() {
        return OK; // Got to bottom of hierarchy
    }

    // Get name of this element in the menu hierarchy
    let p = unsafe { rs_menu_name_skip(name) };

    let mut cur = menu;
    while !cur.is_null() {
        if unsafe { *name } == 0
            || unsafe { *name } as u8 == b'*'
            || unsafe { rs_menu_name_equal(name, cur) }
        {
            if unsafe { *p } != 0 {
                if cur.children().is_null() {
                    unsafe { emsg(nvim_gettext(E_NOTSUBMENU)) };
                    return FAIL;
                }
                if unsafe { rs_menu_enable_recurse(cur.children(), p, modes, enable) } == FAIL {
                    return FAIL;
                }
            } else if enable != 0 {
                unsafe { (*cur.as_ptr()).enabled |= modes };
            } else {
                unsafe { (*cur.as_ptr()).enabled &= !modes };
            }

            // When name is empty, we are doing all menu items for the given
            // modes, so keep looping, otherwise we are just doing the named
            // menu item (which has been found) so break here.
            if unsafe { *name } != 0 && unsafe { *name } as u8 != b'*' {
                break;
            }
        }
        cur = cur.next();
    }

    if unsafe { *name } != 0 && unsafe { *name } as u8 != b'*' && cur.is_null() {
        unsafe { semsg(nvim_gettext(E_NOMENU), name) };
        return FAIL;
    }

    OK
}

/// Remove the (sub)menu with the given name from the menu hierarchy.
///
/// Called recursively.
///
/// This is the Rust implementation of C `remove_menu()`.
///
/// # Safety
/// `menup` must be a valid `vimmenu_T**` pointer. `name` is modified by `menu_name_skip`.
#[no_mangle]
pub unsafe extern "C" fn rs_remove_menu(
    menup: *mut *mut VimMenu,
    name: *mut c_char,
    modes: c_int,
    silent: bool,
) -> c_int {
    if unsafe { (*menup).is_null() } {
        return OK; // Got to bottom of hierarchy
    }

    // Get name of this element in the menu hierarchy
    let p = unsafe { rs_menu_name_skip(name) };

    let mut cur_menup = menup;
    let mut menu_ptr;

    loop {
        menu_ptr = unsafe { *cur_menup };
        if menu_ptr.is_null() {
            break;
        }

        let menu = VimMenuHandle::from_ptr(menu_ptr);

        if unsafe { *name } == 0 || unsafe { rs_menu_name_equal(name, menu) } {
            if unsafe { *p } != 0 && menu.children().is_null() {
                if !silent {
                    unsafe { emsg(nvim_gettext(E_NOTSUBMENU)) };
                }
                return FAIL;
            }
            if (menu.modes() & modes) != 0 {
                let children_ptr = unsafe { &mut (*menu_ptr).children as *mut *mut VimMenu };
                if unsafe { rs_remove_menu(children_ptr, p, modes, silent) } == FAIL {
                    return FAIL;
                }
            } else if unsafe { *name } != 0 {
                if !silent {
                    unsafe { emsg(nvim_gettext(E_MENU_ONLY_EXISTS_IN_ANOTHER_MODE)) };
                }
                return FAIL;
            }

            if unsafe { *name } != 0 {
                break;
            }

            // Remove the menu item for the given mode[s]
            unsafe { (*menu_ptr).modes &= !modes };
            if (modes & MENU_TIP_MODE) != 0 {
                unsafe { rs_free_menu_string(menu, MENU_INDEX_TIP) };
            }
            if (menu.modes() & MENU_ALL_MODES) == 0 {
                unsafe { rs_free_menu(cur_menup) };
            } else {
                cur_menup = unsafe { &mut (*menu_ptr).next as *mut *mut VimMenu };
            }
        } else {
            cur_menup = unsafe { &mut (*menu_ptr).next as *mut *mut VimMenu };
        }
    }

    if unsafe { *name } != 0 {
        if menu_ptr.is_null() {
            if !silent {
                unsafe { semsg(nvim_gettext(E_NOMENU), name) };
            }
            return FAIL;
        }

        // Recalculate modes for menu based on the new updated children
        let menu = VimMenuHandle::from_ptr(menu_ptr);
        unsafe { (*menu_ptr).modes &= !modes };
        let mut child = menu.children();
        while !child.is_null() {
            unsafe { (*menu_ptr).modes |= child.modes() };
            child = child.next();
        }
        if (modes & MENU_TIP_MODE) != 0 {
            unsafe { rs_free_menu_string(menu, MENU_INDEX_TIP) };
        }
        if (menu.modes() & MENU_ALL_MODES) == 0 {
            // The menu item is no longer valid in ANY mode, so delete it
            unsafe { *cur_menup = menu_ptr };
            unsafe { rs_free_menu(cur_menup) };
        }
    }

    OK
}

/// Add the menu with the given name to the menu hierarchy.
///
/// This is the Rust implementation of C `add_menu_path()`.
///
/// # Safety
/// All pointers must be valid.
#[no_mangle]
pub unsafe extern "C" fn rs_add_menu_path(
    menu_path: *const c_char,
    menuarg: VimMenuHandle,
    pri_tab: *const c_int,
    call_data: *const c_char,
) -> c_int {
    let mut modes = menuarg.modes();
    let mut menu = VimMenuHandle::null();
    let mut lower_pri: *mut *mut VimMenu;
    let mut dname: *mut c_char;
    let mut pri_idx: usize = 0;
    let mut old_modes: c_int = 0;
    let mut en_name: *mut c_char;

    // Make a copy so we can stuff around with it, since it could be const
    let path_name = unsafe { xstrdup(menu_path) };
    let root_menu_ptr = &raw mut root_menu;
    let mut menup = root_menu_ptr;
    let mut parent = VimMenuHandle::null();
    let mut name = path_name;

    while unsafe { *name } != 0 {
        // Get name of this element in the menu hierarchy
        let next_name = unsafe { rs_menu_name_skip(name) };
        let map_to = unsafe { rs_menutrans_lookup(name, strlen(name) as c_int) };
        if !map_to.is_null() {
            en_name = name;
            name = map_to;
        } else {
            en_name = std::ptr::null_mut();
        }

        let text_result: MenuTextResult = unsafe { rs_menu_text(name) };
        dname = text_result.text;
        if dname.is_null() || unsafe { *dname } == 0 {
            // Only a mnemonic or accelerator is not valid.
            unsafe { emsg(nvim_gettext(E792_EMPTY_MENU_NAME)) };
            unsafe { xfree(dname as *mut c_void) };
            unsafe { xfree(text_result.actext as *mut c_void) };
            // goto erret
            return add_menu_path_erret(path_name, std::ptr::null_mut(), parent, root_menu_ptr);
        }
        // We don't use mnemonic/actext from this call - free actext
        unsafe { xfree(text_result.actext as *mut c_void) };

        // See if it's already there
        lower_pri = menup;
        menu = VimMenuHandle::from_ptr(unsafe { *menup });
        while !menu.is_null() {
            if unsafe { rs_menu_name_equal(name, menu) }
                || unsafe { rs_menu_name_equal(dname, menu) }
            {
                if unsafe { *next_name } == 0 && !menu.children().is_null() {
                    if !unsafe { sys_menu } {
                        unsafe {
                            emsg(nvim_gettext(E330_MENU_PATH_MUST_NOT_LEAD_TO_SUBMENU));
                        }
                    }
                    unsafe { xfree(dname as *mut c_void) };
                    return add_menu_path_erret(
                        path_name,
                        std::ptr::null_mut(),
                        parent,
                        root_menu_ptr,
                    );
                }
                if unsafe { *next_name } != 0 && menu.children().is_null() {
                    if !unsafe { sys_menu } {
                        unsafe { emsg(nvim_gettext(E_NOTSUBMENU)) };
                    }
                    unsafe { xfree(dname as *mut c_void) };
                    return add_menu_path_erret(
                        path_name,
                        std::ptr::null_mut(),
                        parent,
                        root_menu_ptr,
                    );
                }
                break;
            }
            menup = unsafe { &mut (*menu.as_ptr()).next as *mut *mut VimMenu };

            // Count menus, to find where this one needs to be inserted
            if (!parent.is_null() || unsafe { rs_menu_is_menubar(menu.name()) })
                && menu.priority() <= unsafe { *pri_tab.add(pri_idx) }
            {
                lower_pri = menup;
            }
            menu = menu.next();
        }

        if menu.is_null() {
            if unsafe { *next_name } == 0 && parent.is_null() {
                unsafe { emsg(nvim_gettext(E331_MUST_NOT_ADD_DIRECTLY)) };
                unsafe { xfree(dname as *mut c_void) };
                return add_menu_path_erret(path_name, std::ptr::null_mut(), parent, root_menu_ptr);
            }

            if unsafe { rs_menu_is_separator(dname) } && unsafe { *next_name } != 0 {
                unsafe { emsg(nvim_gettext(E332_SEPARATOR_NOT_IN_PATH)) };
                unsafe { xfree(dname as *mut c_void) };
                return add_menu_path_erret(path_name, std::ptr::null_mut(), parent, root_menu_ptr);
            }

            // Not already there, so let's add it
            let new_ptr = unsafe { xcalloc(1, std::mem::size_of::<VimMenu>()) } as *mut VimMenu;
            menu = unsafe { VimMenuHandle::from_ptr(new_ptr) };

            unsafe {
                (*new_ptr).modes = modes;
                (*new_ptr).enabled = MENU_ALL_MODES;
                (*new_ptr).name = xstrdup(name);
            }

            // separate mnemonic and accelerator text from actual menu name
            let mt: MenuTextResult = unsafe { rs_menu_text(name) };
            unsafe {
                (*new_ptr).dname = mt.text;
                (*new_ptr).mnemonic = mt.mnemonic;
                (*new_ptr).actext = mt.actext;
            }

            if !en_name.is_null() {
                unsafe {
                    (*new_ptr).en_name = xstrdup(en_name);
                }
                let en_mt: MenuTextResult = unsafe { rs_menu_text(en_name) };
                unsafe {
                    (*new_ptr).en_dname = en_mt.text;
                    xfree(en_mt.actext as *mut c_void);
                }
            } else {
                unsafe {
                    (*new_ptr).en_name = std::ptr::null_mut();
                    (*new_ptr).en_dname = std::ptr::null_mut();
                }
            }

            unsafe {
                (*new_ptr).priority = *pri_tab.add(pri_idx);
                (*new_ptr).parent = parent.as_ptr();
            }

            // Add after menu that has lower priority
            unsafe {
                (*new_ptr).next = *lower_pri;
                *lower_pri = new_ptr;
            }

            old_modes = 0;
        } else {
            old_modes = menu.modes();

            // If this menu option was previously only available in other
            // modes, then make sure it's available for this one now
            unsafe {
                (*menu.as_ptr()).modes |= modes;
                (*menu.as_ptr()).enabled |= modes;
            }
        }

        menup = unsafe { &mut (*menu.as_ptr()).children as *mut *mut VimMenu };
        parent = menu;
        name = next_name;
        unsafe { xfree(dname as *mut c_void) };
        if unsafe { *pri_tab.add(pri_idx + 1) } != -1 {
            pri_idx += 1;
        }
    }
    unsafe { xfree(path_name as *mut c_void) };

    // Only add system menu items which have not been defined yet.
    let amenu = ((modes & (MENU_NORMAL_MODE | MENU_INSERT_MODE))
        == (MENU_NORMAL_MODE | MENU_INSERT_MODE)) as c_int;
    if unsafe { sys_menu } {
        modes &= !old_modes;
    }

    if !menu.is_null() && modes != 0 {
        let p: *mut c_char = if call_data.is_null() {
            std::ptr::null_mut()
        } else {
            unsafe { xstrdup(call_data) }
        };

        // loop over all modes, may add more than one
        for i in 0..MENU_MODES {
            if (modes & (1 << i)) != 0 {
                // free any old menu
                unsafe { rs_free_menu_string(menu, i) };

                // For "amenu", may insert an extra character.
                // Don't do this for "<Nop>".
                let mut c: u8 = 0;
                let mut d: u8 = 0;
                if amenu != 0 && !call_data.is_null() && unsafe { *call_data } != 0 {
                    match 1 << i {
                        x if x == MENU_VISUAL_MODE
                            || x == MENU_SELECT_MODE
                            || x == MENU_OP_PENDING_MODE
                            || x == MENU_CMDLINE_MODE =>
                        {
                            c = CTRL_C;
                        }
                        x if x == MENU_INSERT_MODE => {
                            c = CTRL_BSL;
                            d = CTRL_O;
                        }
                        _ => {}
                    }
                }

                if c != 0 {
                    let call_data_len = unsafe { strlen(call_data) };
                    let s = unsafe { nvim_xmalloc(call_data_len + 5) } as *mut u8;
                    unsafe { *s = c };
                    if d == 0 {
                        unsafe {
                            std::ptr::copy_nonoverlapping(
                                call_data as *const u8,
                                s.add(1),
                                call_data_len + 1,
                            );
                        }
                    } else {
                        unsafe {
                            *s.add(1) = d;
                            std::ptr::copy_nonoverlapping(
                                call_data as *const u8,
                                s.add(2),
                                call_data_len + 1,
                            );
                        }
                    }
                    if c == CTRL_C {
                        let len = unsafe { strlen(s as *const c_char) };
                        unsafe {
                            *s.add(len) = CTRL_BSL;
                            *s.add(len + 1) = CTRL_G;
                            *s.add(len + 2) = 0;
                        }
                    }
                    unsafe { (*menu.as_ptr()).strings[i as usize] = s as *mut c_char };
                } else {
                    unsafe { (*menu.as_ptr()).strings[i as usize] = p };
                }
                // menuarg->noremap[0] and menuarg->silent[0] directly
                unsafe {
                    (*menu.as_ptr()).noremap[i as usize] = (*menuarg.as_ptr()).noremap[0];
                    (*menu.as_ptr()).silent[i as usize] = (*menuarg.as_ptr()).silent[0];
                }
            }
        }
    }
    OK
}

/// Helper for add_menu_path error cleanup.
///
/// Cleans up allocated path_name and dname, then removes empty submenu parents.
unsafe fn add_menu_path_erret(
    path_name: *mut c_char,
    dname: *mut c_char,
    mut parent: VimMenuHandle,
    root_menu_ptr: *mut *mut VimMenu,
) -> c_int {
    unsafe {
        xfree(path_name as *mut c_void);
        xfree(dname as *mut c_void);
    }

    // Delete any empty submenu we added before discovering the error
    while !parent.is_null() && parent.children().is_null() {
        let menup: *mut *mut VimMenu = if parent.parent().is_null() {
            root_menu_ptr
        } else {
            unsafe { &mut (*parent.parent().as_ptr()).children as *mut *mut VimMenu }
        };

        // Walk the list to find parent
        let mut cur = menup;
        loop {
            let m = unsafe { *cur };
            if m.is_null() || m == parent.as_ptr() {
                break;
            }
            cur = unsafe { &mut (*m).next as *mut *mut VimMenu };
        }
        if unsafe { (*cur).is_null() } {
            // safety check
            break;
        }
        parent = parent.parent();
        unsafe { rs_free_menu(cur) };
    }

    FAIL
}
