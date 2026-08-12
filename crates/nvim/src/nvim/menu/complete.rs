//! Command-line completion of a menu path -- `:emenu <Tab>` and friends.
//!
//! [`set_context_in_menu_cmd`] parses as much of a half-typed `:menu` command
//! as exists to decide what the next word could be -- a mode prefix, a menu
//! path, or nothing -- and leaves the node the path reached in `expand_menu`
//! for the generator.  [`get_menu_name`] and [`get_menu_names`] are then the
//! two generators the completion machinery calls repeatedly, the second
//! walking submenus and offering both the translated and the original name of
//! each.
//!
//! Original: `src/nvim/menu.c`, Vim/Neovim, Vim license.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;
use crate::src::nvim::ascii::{ascii_isdigit, ascii_iswhite};
use crate::src::nvim::global_cell::GlobalCell;
use crate::src::nvim::keycodes::Ctrl_V;
use crate::src::nvim::main::root_menu;
use crate::src::nvim::memory::{xfree, xmalloc, xstrlcpy};
use crate::src::nvim::os::libc::{strcat, strlen, strncmp};
use crate::src::nvim::types::{expand_T, size_t, vimmenu_T};

static expand_menu: GlobalCell<*mut vimmenu_T> =
    GlobalCell::new(::core::ptr::null_mut::<vimmenu_T>());

static expand_modes: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0 as ::core::ffi::c_int);

static expand_emenu: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0);

pub unsafe extern "C" fn set_context_in_menu_cmd(
    mut xp: *mut expand_T,
    mut cmd: *const ::core::ffi::c_char,
    mut arg: *mut ::core::ffi::c_char,
    mut forceit: bool,
) -> *mut ::core::ffi::c_char {
    unsafe {
        let mut after_dot: *mut ::core::ffi::c_char =
            ::core::ptr::null_mut::<::core::ffi::c_char>();
        let mut p: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
        let mut path_name: *mut ::core::ffi::c_char =
            ::core::ptr::null_mut::<::core::ffi::c_char>();
        let mut unmenu: bool = false;
        let mut menu: *mut vimmenu_T = ::core::ptr::null_mut::<vimmenu_T>();
        (*xp).xp_context = EXPAND_UNSUCCESSFUL as ::core::ffi::c_int;
        p = arg;
        while *p != 0 {
            if !ascii_isdigit(*p as ::core::ffi::c_int)
                && *p as ::core::ffi::c_int != '.' as ::core::ffi::c_int
            {
                break;
            }
            p = p.offset(1);
        }
        if !ascii_iswhite(*p as ::core::ffi::c_int) {
            if strncmp(arg, c"enable".as_ptr(), 6 as size_t) == 0 as ::core::ffi::c_int
                && (*arg.offset(6 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == NUL
                    || ascii_iswhite(
                        *arg.offset(6 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                    ) as ::core::ffi::c_int
                        != 0)
            {
                p = arg.offset(6 as ::core::ffi::c_int as isize);
            } else if strncmp(arg, c"disable".as_ptr(), 7 as size_t) == 0 as ::core::ffi::c_int
                && (*arg.offset(7 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == NUL
                    || ascii_iswhite(
                        *arg.offset(7 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                    ) as ::core::ffi::c_int
                        != 0)
            {
                p = arg.offset(7 as ::core::ffi::c_int as isize);
            } else {
                p = arg;
            }
        }
        while *p as ::core::ffi::c_int != NUL
            && ascii_iswhite(*p as ::core::ffi::c_int) as ::core::ffi::c_int != 0
        {
            p = p.offset(1);
        }
        after_dot = p;
        arg = after_dot;
        while *p as ::core::ffi::c_int != 0 && !ascii_iswhite(*p as ::core::ffi::c_int) {
            if (*p as ::core::ffi::c_int == '\\' as ::core::ffi::c_int
                || *p as ::core::ffi::c_int == Ctrl_V)
                && *p.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int != NUL
            {
                p = p.offset(1);
            } else if *p as ::core::ffi::c_int == '.' as ::core::ffi::c_int {
                after_dot = p.offset(1 as ::core::ffi::c_int as isize);
            }
            p = p.offset(1);
        }
        let mut expand_menus: ::core::ffi::c_int = !(*cmd as ::core::ffi::c_int
            == 't' as ::core::ffi::c_int
            && *cmd.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                == 'e' as ::core::ffi::c_int
            || *cmd as ::core::ffi::c_int == 'p' as ::core::ffi::c_int)
            as ::core::ffi::c_int;
        expand_emenu
            .set((*cmd as ::core::ffi::c_int == 'e' as ::core::ffi::c_int) as ::core::ffi::c_int);
        if expand_menus != 0 && ascii_iswhite(*p as ::core::ffi::c_int) as ::core::ffi::c_int != 0 {
            return ::core::ptr::null_mut::<::core::ffi::c_char>();
        }
        if *p as ::core::ffi::c_int == NUL {
            expand_modes.set(get_menu_cmd_modes(
                cmd,
                forceit,
                ::core::ptr::null_mut::<::core::ffi::c_int>(),
                &raw mut unmenu,
            ));
            if !unmenu {
                expand_modes.set(MENU_ALL_MODES as ::core::ffi::c_int);
            }
            menu = root_menu.get();
            if after_dot > arg {
                let mut path_len: size_t = after_dot.offset_from(arg) as size_t;
                path_name = xmalloc(path_len) as *mut ::core::ffi::c_char;
                xstrlcpy(path_name, arg, path_len);
            }
            let mut name: *mut ::core::ffi::c_char = path_name;
            while !name.is_null() && *name as ::core::ffi::c_int != 0 {
                p = menu_name_skip(name);
                while !menu.is_null() {
                    if menu_name_equal(name, menu) {
                        if *p as ::core::ffi::c_int != NUL && (*menu).children.is_null()
                            || (*menu).modes & expand_modes.get() == 0 as ::core::ffi::c_int
                        {
                            xfree(path_name as *mut ::core::ffi::c_void);
                            return ::core::ptr::null_mut::<::core::ffi::c_char>();
                        }
                        break;
                    } else {
                        menu = (*menu).next;
                    }
                }
                if menu.is_null() {
                    xfree(path_name as *mut ::core::ffi::c_void);
                    return ::core::ptr::null_mut::<::core::ffi::c_char>();
                }
                name = p;
                menu = (*menu).children;
            }
            xfree(path_name as *mut ::core::ffi::c_void);
            (*xp).xp_context = if expand_menus != 0 {
                EXPAND_MENUNAMES as ::core::ffi::c_int
            } else {
                EXPAND_MENUS as ::core::ffi::c_int
            };
            (*xp).xp_pattern = after_dot;
            expand_menu.set(menu);
        } else {
            (*xp).xp_context = EXPAND_NOTHING as ::core::ffi::c_int;
        }
        return ::core::ptr::null_mut::<::core::ffi::c_char>();
    }
}

pub unsafe extern "C" fn get_menu_name(
    mut _xp: *mut expand_T,
    mut idx: ::core::ffi::c_int,
) -> *mut ::core::ffi::c_char {
    unsafe {
        static menu: GlobalCell<*mut vimmenu_T> =
            GlobalCell::new(::core::ptr::null_mut::<vimmenu_T>());
        let mut str: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
        static should_advance: GlobalCell<bool> = GlobalCell::new(false_0 != 0);
        if idx == 0 as ::core::ffi::c_int {
            menu.set(expand_menu.get());
            should_advance.set(false_0 != 0);
        }
        while !(*menu.ptr()).is_null()
            && (menu_is_hidden((*menu.get()).dname) as ::core::ffi::c_int != 0
                || menu_is_separator((*menu.get()).dname) as ::core::ffi::c_int != 0
                || (*menu.get()).children.is_null())
        {
            menu.set((*menu.get()).next);
        }
        if (*menu.ptr()).is_null() {
            return ::core::ptr::null_mut::<::core::ffi::c_char>();
        }
        if (*menu.get()).modes & expand_modes.get() != 0 {
            if should_advance.get() {
                str = (*menu.get()).en_dname;
            } else {
                str = (*menu.get()).dname;
                if (*menu.get()).en_dname.is_null() {
                    should_advance.set(true_0 != 0);
                }
            }
        } else {
            str = c"".as_ptr() as *mut ::core::ffi::c_char;
        }
        if should_advance.get() {
            menu.set((*menu.get()).next);
        }
        should_advance.set(!should_advance.get());
        return str;
    }
}

pub unsafe extern "C" fn get_menu_names(
    mut _xp: *mut expand_T,
    mut idx: ::core::ffi::c_int,
) -> *mut ::core::ffi::c_char {
    unsafe {
        static menu: GlobalCell<*mut vimmenu_T> =
            GlobalCell::new(::core::ptr::null_mut::<vimmenu_T>());
        static tbuffer: GlobalCell<[::core::ffi::c_char; 256]> = GlobalCell::new([0; 256]);
        let mut str: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
        static should_advance: GlobalCell<bool> = GlobalCell::new(false_0 != 0);
        if idx == 0 as ::core::ffi::c_int {
            menu.set(expand_menu.get());
            should_advance.set(false_0 != 0);
        }
        while !(*menu.ptr()).is_null()
            && (menu_is_hidden((*menu.get()).dname) as ::core::ffi::c_int != 0
                || expand_emenu.get() != 0
                    && menu_is_separator((*menu.get()).dname) as ::core::ffi::c_int != 0
                || *(*menu.get())
                    .dname
                    .add(strlen((*menu.get()).dname).wrapping_sub(1 as size_t))
                    as ::core::ffi::c_int
                    == '.' as ::core::ffi::c_int)
        {
            menu.set((*menu.get()).next);
        }
        if (*menu.ptr()).is_null() {
            return ::core::ptr::null_mut::<::core::ffi::c_char>();
        }
        if (*menu.get()).modes & expand_modes.get() != 0 {
            if !(*menu.get()).children.is_null() {
                if should_advance.get() {
                    xstrlcpy(
                        tbuffer.ptr() as *mut ::core::ffi::c_char,
                        (*menu.get()).en_dname,
                        TBUFFER_LEN as size_t,
                    );
                } else {
                    xstrlcpy(
                        tbuffer.ptr() as *mut ::core::ffi::c_char,
                        (*menu.get()).dname,
                        TBUFFER_LEN as size_t,
                    );
                    if (*menu.get()).en_dname.is_null() {
                        should_advance.set(true_0 != 0);
                    }
                }
                strcat(tbuffer.ptr() as *mut ::core::ffi::c_char, c"\x01".as_ptr());
                str = tbuffer.ptr() as *mut ::core::ffi::c_char;
            } else if should_advance.get() {
                str = (*menu.get()).en_dname;
            } else {
                str = (*menu.get()).dname;
                if (*menu.get()).en_dname.is_null() {
                    should_advance.set(true_0 != 0);
                }
            }
        } else {
            str = c"".as_ptr() as *mut ::core::ffi::c_char;
        }
        if should_advance.get() {
            menu.set((*menu.get()).next);
        }
        should_advance.set(!should_advance.get());
        return str;
    }
}

pub const TBUFFER_LEN: ::core::ffi::c_int = 256 as ::core::ffi::c_int;
