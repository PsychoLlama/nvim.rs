//! Menu query functions: `menu_get_recursive` and `menu_get`.
//!
//! Migrated from `src/nvim/menu.c`.

use std::ffi::{c_char, c_int, c_void};

use crate::handle::VimMenuHandle;
use crate::menu_modes::{MENU_INDEX_TIP, MENU_MODES, MENU_TIP_MODE};

/// REMAP_NONE flag.
const REMAP_NONE: c_int = -1;
/// REMAP_SCRIPT flag.
const REMAP_SCRIPT: c_int = -2;
/// kListLenMayKnow sentinel.
const K_LIST_LEN_MAY_KNOW: isize = -3;

/// The character for each menu mode.
/// Must stay in sync with `VimMenu::strings` index ordering.
static MENU_MODE_CHARS: [&[u8]; 8] = [
    b"n\0", b"v\0", b"s\0", b"o\0", b"i\0", b"c\0", b"tl\0", b"t\0",
];

extern "C" {
    // Dict/list operations
    fn tv_dict_alloc() -> *mut c_void;
    fn tv_dict_add_str(
        dict: *mut c_void,
        key: *const c_char,
        key_len: usize,
        val: *const c_char,
    ) -> c_int;
    fn tv_dict_add_nr(dict: *mut c_void, key: *const c_char, key_len: usize, nr: i64) -> c_int;
    fn tv_dict_add_dict(
        dict: *mut c_void,
        key: *const c_char,
        key_len: usize,
        child: *mut c_void,
    ) -> c_int;
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
    #[link_name = "rs_tv_dict_len"]
    fn tv_dict_len(dict: *mut c_void) -> i64;
    fn tv_list_alloc(count: isize) -> *mut c_void;
    fn tv_list_append_dict(list: *mut c_void, dict: *mut c_void);

    // String helpers
    fn str2special_save(s: *const c_char, replace_spaces: bool, replace_lt: bool) -> *mut c_char;
    fn utf_char2bytes(c: c_int, buf: *mut c_char) -> c_int;

    // Menu hidden check
    fn menu_is_hidden(name: *mut c_char) -> bool;

    // Root menu global
    static mut root_menu: *mut crate::vim_menu::VimMenu;

    // find_menu (already in Rust, exported by name)
    fn find_menu(menu: VimMenuHandle, name: *mut c_char, modes: c_int) -> VimMenuHandle;
}

/// Macro-equivalent: add a string to a dict with key literal.
/// Returns early returning NULL on error (we ignore the return value here).
macro_rules! dict_add_str {
    ($dict:expr, $key:expr, $val:expr) => {
        unsafe {
            tv_dict_add_str(
                $dict,
                $key.as_ptr().cast::<c_char>(),
                $key.len() - 1, // exclude NUL terminator
                $val,
            )
        }
    };
}

macro_rules! dict_add_nr {
    ($dict:expr, $key:expr, $nr:expr) => {
        unsafe { tv_dict_add_nr($dict, $key.as_ptr().cast::<c_char>(), $key.len() - 1, $nr) }
    };
}

macro_rules! dict_add_dict {
    ($dict:expr, $key:expr, $child:expr) => {
        unsafe {
            tv_dict_add_dict(
                $dict,
                $key.as_ptr().cast::<c_char>(),
                $key.len() - 1,
                $child,
            )
        }
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

/// Recursively build a dict for a menu item.
///
/// Returns NULL if the menu is not visible in `modes`.
/// Matches the C `menu_get_recursive` exactly.
///
/// # Safety
/// `menu` must be a valid VimMenuHandle or null.
unsafe fn menu_get_recursive(menu: VimMenuHandle, modes: c_int) -> *mut c_void {
    if menu.is_null() || (menu.modes() & modes) == 0 {
        return std::ptr::null_mut();
    }

    let dict = unsafe { tv_dict_alloc() };

    dict_add_str!(dict, b"name\0", menu.dname());
    dict_add_nr!(dict, b"priority\0", i64::from(menu.priority()));

    let hidden = unsafe { menu_is_hidden(menu.dname().cast_mut()) };
    dict_add_nr!(dict, b"hidden\0", i64::from(hidden));

    if menu.mnemonic() != 0 {
        let mut buf = [0u8; 8]; // MB_MAXCHAR + 1 = 7 + 1
        unsafe { utf_char2bytes(menu.mnemonic(), buf.as_mut_ptr().cast::<c_char>()) };
        dict_add_str!(dict, b"shortcut\0", buf.as_ptr().cast::<c_char>());
    }

    if !menu.actext().is_null() {
        dict_add_str!(dict, b"actext\0", menu.actext());
    }

    if (menu.modes() & MENU_TIP_MODE) != 0 {
        let tip_str = unsafe { (*menu.as_ptr()).strings[MENU_INDEX_TIP as usize] };
        if !tip_str.is_null() {
            dict_add_str!(dict, b"tooltip\0", tip_str);
        }
    }

    let children = menu.children();
    if children.is_null() {
        // leaf menu
        let commands = unsafe { tv_dict_alloc() };
        dict_add_dict!(dict, b"mappings\0", commands);

        for bit in 0..MENU_MODES {
            if (menu.modes() & modes & (1 << bit)) != 0 {
                let impl_dict = unsafe { tv_dict_alloc() };
                let strings_ptr = unsafe { (*menu.as_ptr()).strings[bit as usize] };
                let noremap_val = unsafe { (*menu.as_ptr()).noremap[bit as usize] };
                let silent_val = unsafe { (*menu.as_ptr()).silent[bit as usize] };
                let enabled_val = menu.enabled();

                dict_add_allocated_str!(
                    impl_dict,
                    b"rhs\0",
                    str2special_save(strings_ptr.cast::<c_char>().cast_const(), false, false)
                );
                dict_add_nr!(impl_dict, b"silent\0", i64::from(silent_val));
                dict_add_nr!(
                    impl_dict,
                    b"enabled\0",
                    if (enabled_val & (1 << bit)) != 0 {
                        1
                    } else {
                        0
                    }
                );
                dict_add_nr!(
                    impl_dict,
                    b"noremap\0",
                    if (noremap_val & REMAP_NONE) != 0 {
                        1
                    } else {
                        0
                    }
                );
                dict_add_nr!(
                    impl_dict,
                    b"sid\0",
                    if (noremap_val & REMAP_SCRIPT) != 0 {
                        1
                    } else {
                        0
                    }
                );

                // key is menu_mode_chars[bit], length 1 or 2
                let mode_char = MENU_MODE_CHARS[bit as usize];
                let key_len = mode_char.len() - 1; // exclude NUL
                unsafe {
                    tv_dict_add_dict(
                        commands,
                        mode_char.as_ptr().cast::<c_char>(),
                        key_len,
                        impl_dict,
                    )
                };
            }
        }
    } else {
        // visit recursively all children
        let children_list = unsafe { tv_list_alloc(K_LIST_LEN_MAY_KNOW) };
        let mut child = children;
        while !child.is_null() {
            let d = unsafe { menu_get_recursive(child, modes) };
            if !d.is_null() && unsafe { tv_dict_len(d) } > 0 {
                unsafe { tv_list_append_dict(children_list, d) };
            }
            child = child.next();
        }
        dict_add_list!(dict, b"submenus\0", children_list);
    }

    dict
}

/// Export menus matching `path_name`.
///
/// This is the Rust replacement for C `menu_get()`.
///
/// # Safety
/// `path_name` must be a valid NUL-terminated string (possibly empty).
/// `list` must be a valid `list_T*`.
#[export_name = "menu_get"]
pub unsafe extern "C" fn rs_menu_get(
    path_name: *mut c_char,
    modes: c_int,
    list: *mut c_void,
) -> bool {
    // Start from root_menu.
    let mut menu = VimMenuHandle::from_ptr(unsafe { root_menu });

    if unsafe { *path_name } != 0 {
        // Non-empty path: locate the menu node.
        menu = unsafe { find_menu(menu, path_name, modes) };
        if menu.is_null() {
            return false;
        }
    }

    let path_empty = unsafe { *path_name } == 0;

    loop {
        if menu.is_null() {
            break;
        }
        let d = unsafe { menu_get_recursive(menu, modes) };
        if !d.is_null() && unsafe { tv_dict_len(d) } > 0 {
            unsafe { tv_list_append_dict(list, d) };
        }
        if !path_empty {
            // Only the first node is relevant for a named path.
            break;
        }
        menu = menu.next();
    }

    true
}
