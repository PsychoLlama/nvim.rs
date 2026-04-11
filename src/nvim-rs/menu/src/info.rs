//! Menu info functions: `menuitem_getinfo` and `f_menu_info`.
//!
//! Migrated from `src/nvim/menu.c`.

use std::ffi::{c_char, c_int, c_void};

use crate::handle::VimMenuHandle;
use crate::menu_modes::MENU_MODES;

/// REMAP_NONE flag.
const REMAP_NONE: c_int = -1;
/// REMAP_SCRIPT flag.
const REMAP_SCRIPT: c_int = -2;
/// kListLenMayKnow sentinel.
const K_LIST_LEN_MAY_KNOW: isize = -3;
/// NUMBUFLEN: buffer size for number/char conversion.
const NUMBUFLEN: usize = 65;

extern "C" {
    // Dict/list operations
    fn tv_dict_add_str(
        dict: *mut c_void,
        key: *const c_char,
        key_len: usize,
        val: *const c_char,
    ) -> c_int;
    fn tv_dict_add_nr(dict: *mut c_void, key: *const c_char, key_len: usize, nr: i64) -> c_int;
    fn tv_dict_add_bool(dict: *mut c_void, key: *const c_char, key_len: usize, val: c_int)
        -> c_int;
    fn tv_dict_add_list(
        dict: *mut c_void,
        key: *const c_char,
        key_len: usize,
        list: *mut c_void,
    ) -> c_int;
    fn tv_dict_add_allocated_str(
        dict: *mut c_void,
        key: *const c_char,
        key_len: usize,
        val: *mut c_char,
    ) -> c_int;
    fn tv_dict_alloc_ret(rettv: *mut c_void);
    fn tv_list_alloc(count: isize) -> *mut c_void;
    fn tv_list_append_string(list: *mut c_void, s: *const c_char, len: isize);

    // String helpers
    fn str2special_save(s: *const c_char, replace_spaces: bool, replace_lt: bool) -> *mut c_char;
    fn utf_char2bytes(c: c_int, buf: *mut c_char) -> c_int;
    fn xstrdup(s: *const c_char) -> *mut c_char;
    fn xfree(p: *mut c_void);

    // Menu helpers
    fn menu_is_hidden(name: *mut c_char) -> bool;
    fn get_menu_mode_str(modes: c_int) -> *const c_char;

    // Typval helpers
    fn tv_get_string_chk(tv: *const c_void) -> *const c_char;

    // Menu name helpers (already in Rust, exported by name)
    fn menu_name_skip(name: *mut c_char) -> *mut c_char;
    fn menu_name_equal(name: *const c_char, menu: VimMenuHandle) -> bool;

    // Root menu global
    static mut root_menu: *mut crate::vim_menu::VimMenu;
}

/// Macro helpers
macro_rules! dict_add_str {
    ($dict:expr, $key:expr, $val:expr) => {
        unsafe { tv_dict_add_str($dict, $key.as_ptr().cast::<c_char>(), $key.len() - 1, $val) }
    };
}

macro_rules! dict_add_nr {
    ($dict:expr, $key:expr, $nr:expr) => {
        unsafe { tv_dict_add_nr($dict, $key.as_ptr().cast::<c_char>(), $key.len() - 1, $nr) }
    };
}

macro_rules! dict_add_bool {
    ($dict:expr, $key:expr, $val:expr) => {
        unsafe { tv_dict_add_bool($dict, $key.as_ptr().cast::<c_char>(), $key.len() - 1, $val) }
    };
}

macro_rules! dict_add_list {
    ($dict:expr, $key:expr, $list:expr) => {
        unsafe { tv_dict_add_list($dict, $key.as_ptr().cast::<c_char>(), $key.len() - 1, $list) }
    };
}

macro_rules! dict_add_allocated_str {
    ($dict:expr, $key:expr, $val:expr) => {
        unsafe {
            tv_dict_add_allocated_str($dict, $key.as_ptr().cast::<c_char>(), $key.len() - 1, $val)
        }
    };
}

/// Get information about a menu item.
///
/// Matches the C `menuitem_getinfo` function exactly.
///
/// # Safety
/// All pointers must be valid.
unsafe fn menuitem_getinfo(
    menu_name: *const c_char,
    menu: VimMenuHandle,
    modes: c_int,
    dict: *mut c_void,
) {
    if unsafe { *menu_name } == 0 {
        // Return all the top-level menus.
        let l = unsafe { tv_list_alloc(K_LIST_LEN_MAY_KNOW) };
        dict_add_list!(dict, b"submenus\0", l);
        // Get all children, skip hidden PopUp[nvoci].
        let mut topmenu = menu;
        while !topmenu.is_null() {
            if !unsafe { menu_is_hidden(topmenu.dname().cast_mut()) } {
                unsafe { tv_list_append_string(l, topmenu.dname(), -1) };
            }
            topmenu = topmenu.next();
        }
        return;
    }

    dict_add_str!(dict, b"name\0", menu.name());
    dict_add_str!(dict, b"display\0", menu.dname());
    if !menu.actext().is_null() {
        dict_add_str!(dict, b"accel\0", menu.actext());
    }
    dict_add_nr!(dict, b"priority\0", i64::from(menu.priority()));
    dict_add_str!(dict, b"modes\0", get_menu_mode_str(menu.modes()));

    // shortcut: utf_char2bytes then NUL
    let mut buf = [0u8; NUMBUFLEN];
    let n = unsafe { utf_char2bytes(menu.mnemonic(), buf.as_mut_ptr().cast::<c_char>()) };
    buf[n as usize] = 0;
    dict_add_str!(dict, b"shortcut\0", buf.as_ptr().cast::<c_char>());

    if menu.children().is_null() {
        // Leaf menu: find the first mode with a mapping.
        let mut bit = 0;
        while bit < MENU_MODES && ((1 << bit) & modes) == 0 {
            bit += 1;
        }

        if bit < MENU_MODES {
            let strings_ptr = unsafe { (*menu.as_ptr()).strings[bit as usize] };
            let noremap_val = unsafe { (*menu.as_ptr()).noremap[bit as usize] };
            let silent_val = unsafe { (*menu.as_ptr()).silent[bit as usize] };
            let enabled_val = menu.enabled();

            if !strings_ptr.is_null() {
                let rhs_str = if unsafe { *strings_ptr } == 0 {
                    unsafe { xstrdup(c"<Nop>".as_ptr()) }
                } else {
                    unsafe { str2special_save(strings_ptr.cast_const(), false, false) }
                };
                dict_add_allocated_str!(dict, b"rhs\0", rhs_str);
            }
            dict_add_bool!(dict, b"noremenu\0", c_int::from(noremap_val == REMAP_NONE));
            dict_add_bool!(dict, b"script\0", c_int::from(noremap_val == REMAP_SCRIPT));
            dict_add_bool!(dict, b"silent\0", c_int::from(silent_val));
            dict_add_bool!(
                dict,
                b"enabled\0",
                c_int::from((enabled_val & (1 << bit)) != 0)
            );
        }
    } else {
        // Has submenus: add the display names of each child.
        let l = unsafe { tv_list_alloc(K_LIST_LEN_MAY_KNOW) };
        dict_add_list!(dict, b"submenus\0", l);
        let mut child = menu.children();
        while !child.is_null() {
            unsafe { tv_list_append_string(l, child.dname(), -1) };
            child = child.next();
        }
    }
}

/// "menu_info()" VimL function.
///
/// Returns information about a menu (including all child menus).
/// This is the Rust replacement for C `f_menu_info()`.
///
/// # Safety
/// `argvars` must be a valid `typval_T[2]`, `rettv` must be a valid `typval_T*`.
#[export_name = "f_menu_info"]
pub unsafe extern "C" fn rs_f_menu_info(
    argvars: *const c_void,
    rettv: *mut c_void,
    _fptr: *mut c_void,
) {
    unsafe { tv_dict_alloc_ret(rettv) };

    // Get the return dict pointer by reading rettv->vval.v_dict.
    // tv_dict_alloc_ret sets rettv->v_type = VAR_DICT and rettv->vval.v_dict.
    // We need the dict pointer: it's at offset 8 in a 16-byte typval_T.
    let retdict: *mut c_void = unsafe {
        let tv_bytes = rettv as *const u8;
        let dict_ptr_ptr = tv_bytes.add(8) as *const *mut c_void;
        *dict_ptr_ptr
    };

    // Get menu_name from argvars[0].
    let menu_name = unsafe { tv_get_string_chk(argvars) };
    if menu_name.is_null() {
        return;
    }

    // Get 'which' from argvars[1] (16-byte offset in typval_T array).
    const TYPVAL_SZ: usize = 16;
    let arg1 = unsafe { (argvars as *const u8).add(TYPVAL_SZ).cast::<c_void>() };
    // Check v_type == VAR_UNKNOWN (0)
    let arg1_type: c_int = unsafe { *(arg1 as *const c_int) };
    let which: *const c_char = if arg1_type != 0 {
        // VAR_UNKNOWN = 0
        let w = unsafe { tv_get_string_chk(arg1) };
        if w.is_null() {
            return;
        }
        w
    } else {
        c"".as_ptr()
    };

    let first_char: u8 = if which.is_null() {
        0
    } else {
        (unsafe { *which }) as u8
    };
    let cmd_result = unsafe { crate::commands::rs_get_menu_cmd_modes(which, first_char == b'!') };
    let modes = cmd_result.modes;

    // Locate the specified menu or menu item.
    let mut menu = VimMenuHandle::from_ptr(unsafe { root_menu });

    let saved_name = unsafe { xstrdup(menu_name) };
    if unsafe { *saved_name } != 0 {
        let mut name = saved_name;
        loop {
            if unsafe { *name } == 0 {
                break;
            }
            // Find in the menu hierarchy.
            let p = unsafe { menu_name_skip(name) };
            // Search for matching menu at this level.
            while !menu.is_null() {
                if unsafe { menu_name_equal(name.cast_const(), menu) } {
                    break;
                }
                menu = menu.next();
            }
            if menu.is_null() || unsafe { *p } == 0 {
                break;
            }
            menu = menu.children();
            name = p;
        }
    }
    unsafe { xfree(saved_name.cast::<c_void>()) };

    if menu.is_null() {
        return;
    }

    if (menu.modes() & modes) != 0 {
        unsafe { menuitem_getinfo(menu_name, menu, modes, retdict) };
    }
}
