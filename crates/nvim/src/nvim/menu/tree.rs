//! The menu tree itself -- creating, walking, listing and freeing it.
//!
//! A menu is a linked list of `vimmenu_T` siblings, each with a `children`
//! list of its own.  This is everything that treats that tree as a data
//! structure: [`find_menu`] resolves a path to a node,
//! [`menu_get_recursive`]/[`menu_get`] dump it as the nested Dict
//! `menu_get()` returns, [`show_menus`] and [`show_menus_recursive`] print the
//! `:menu` listing, [`remove_menu`] unlinks and [`free_menu`] releases a
//! subtree, and [`menu_enable_recurse`] flips the `enabled` flag.
//!
//! Original: `src/nvim/menu.c`, Vim/Neovim, Vim license.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;
use crate::semsg_c;
use crate::src::nvim::eval::typval::tv_dict_len;
use crate::src::nvim::eval::typval::{
    tv_dict_add_allocated_str, tv_dict_add_dict, tv_dict_add_list, tv_dict_add_nr, tv_dict_add_str,
    tv_dict_alloc, tv_list_alloc, tv_list_append_dict,
};
use crate::src::nvim::highlight_group::{HLF_8, HLF_D};
use crate::src::nvim::main::{e_menu_only_exists_in_another_mode, got_int, root_menu};
use crate::src::nvim::mbyte::utf_char2bytes;
use crate::src::nvim::memory::{xfree, xstrdup};
use crate::src::nvim::message::{
    emsg, msg_outnum, msg_outtrans, msg_outtrans_special, msg_putchar, msg_puts, msg_puts_hl,
    msg_puts_title, str2special_save,
};
use crate::src::nvim::os::libc::gettext;
use crate::src::nvim::types::{
    dict_T, kListLenMayKnow, list_T, ptrdiff_t, size_t, varnumber_T, vimmenu_T,
};

pub(crate) unsafe extern "C" fn menu_enable_recurse(
    mut menu: *mut vimmenu_T,
    mut name: *mut ::core::ffi::c_char,
    mut modes: ::core::ffi::c_int,
    mut enable: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    unsafe {
        if menu.is_null() {
            return OK;
        }
        let mut p: *mut ::core::ffi::c_char = menu_name_skip(name);
        while !menu.is_null() {
            if *name as ::core::ffi::c_int == NUL
                || *name as ::core::ffi::c_int == '*' as ::core::ffi::c_int
                || menu_name_equal(name, menu) as ::core::ffi::c_int != 0
            {
                if *p as ::core::ffi::c_int != NUL {
                    if (*menu).children.is_null() {
                        emsg(gettext(e_notsubmenu.as_ptr()));
                        return FAIL;
                    }
                    if menu_enable_recurse((*menu).children, p, modes, enable) == FAIL {
                        return FAIL;
                    }
                } else if enable != 0 {
                    (*menu).enabled |= modes;
                } else {
                    (*menu).enabled &= !modes;
                }
                if *name as ::core::ffi::c_int != NUL
                    && *name as ::core::ffi::c_int != '*' as ::core::ffi::c_int
                {
                    break;
                }
            }
            menu = (*menu).next;
        }
        if *name as ::core::ffi::c_int != NUL
            && *name as ::core::ffi::c_int != '*' as ::core::ffi::c_int
            && menu.is_null()
        {
            semsg_c!(gettext(e_nomenu.as_ptr()), name,);
            return FAIL;
        }
        return OK;
    }
}

pub(crate) unsafe extern "C" fn remove_menu(
    mut menup: *mut *mut vimmenu_T,
    mut name: *mut ::core::ffi::c_char,
    mut modes: ::core::ffi::c_int,
    mut silent: bool,
) -> ::core::ffi::c_int {
    unsafe {
        let mut menu: *mut vimmenu_T = ::core::ptr::null_mut::<vimmenu_T>();
        if (*menup).is_null() {
            return OK;
        }
        let mut p: *mut ::core::ffi::c_char = menu_name_skip(name);
        loop {
            menu = *menup;
            if menu.is_null() {
                break;
            }
            if *name as ::core::ffi::c_int == NUL
                || menu_name_equal(name, menu) as ::core::ffi::c_int != 0
            {
                if *p as ::core::ffi::c_int != NUL && (*menu).children.is_null() {
                    if !silent {
                        emsg(gettext(e_notsubmenu.as_ptr()));
                    }
                    return FAIL;
                }
                if (*menu).modes & modes != 0 as ::core::ffi::c_int {
                    if remove_menu(&raw mut (*menu).children, p, modes, silent) == FAIL {
                        return FAIL;
                    }
                } else if *name as ::core::ffi::c_int != NUL {
                    if !silent {
                        emsg(gettext(
                            &raw const e_menu_only_exists_in_another_mode
                                as *const ::core::ffi::c_char,
                        ));
                    }
                    return FAIL;
                }
                if *name as ::core::ffi::c_int != NUL {
                    break;
                }
                (*menu).modes &= !modes;
                if modes & MENU_TIP_MODE as ::core::ffi::c_int != 0 {
                    free_menu_string(menu, MENU_INDEX_TIP as ::core::ffi::c_int);
                }
                if (*menu).modes & MENU_ALL_MODES as ::core::ffi::c_int == 0 as ::core::ffi::c_int {
                    free_menu(menup);
                } else {
                    menup = &raw mut (*menu).next;
                }
            } else {
                menup = &raw mut (*menu).next;
            }
        }
        if *name as ::core::ffi::c_int != NUL {
            if menu.is_null() {
                if !silent {
                    semsg_c!(gettext(e_nomenu.as_ptr()), name,);
                }
                return FAIL;
            }
            (*menu).modes &= !modes;
            let mut child: *mut vimmenu_T = (*menu).children;
            while !child.is_null() {
                (*menu).modes |= (*child).modes;
                child = (*child).next;
            }
            if modes & MENU_TIP_MODE as ::core::ffi::c_int != 0 {
                free_menu_string(menu, MENU_INDEX_TIP as ::core::ffi::c_int);
            }
            if (*menu).modes & MENU_ALL_MODES as ::core::ffi::c_int == 0 as ::core::ffi::c_int {
                *menup = menu;
                free_menu(menup);
            }
        }
        return OK;
    }
}

pub(crate) unsafe extern "C" fn free_menu(mut menup: *mut *mut vimmenu_T) {
    unsafe {
        let mut menu: *mut vimmenu_T = *menup;
        *menup = (*menu).next;
        xfree((*menu).name as *mut ::core::ffi::c_void);
        xfree((*menu).dname as *mut ::core::ffi::c_void);
        xfree((*menu).en_name as *mut ::core::ffi::c_void);
        xfree((*menu).en_dname as *mut ::core::ffi::c_void);
        xfree((*menu).actext as *mut ::core::ffi::c_void);
        let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while i < MENU_MODES as ::core::ffi::c_int {
            free_menu_string(menu, i);
            i += 1;
        }
        xfree(menu as *mut ::core::ffi::c_void);
    }
}

pub(crate) unsafe extern "C" fn free_menu_string(
    mut menu: *mut vimmenu_T,
    mut idx: ::core::ffi::c_int,
) {
    unsafe {
        let mut count: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while i < MENU_MODES as ::core::ffi::c_int {
            if (*menu).strings[i as usize] == (*menu).strings[idx as usize] {
                count += 1;
            }
            i += 1;
        }
        if count == 1 as ::core::ffi::c_int {
            xfree((*menu).strings[idx as usize] as *mut ::core::ffi::c_void);
        }
        (*menu).strings[idx as usize] = ::core::ptr::null_mut::<::core::ffi::c_char>();
    }
}

unsafe extern "C" fn menu_get_recursive(
    mut menu: *const vimmenu_T,
    mut modes: ::core::ffi::c_int,
) -> *mut dict_T {
    unsafe {
        if menu.is_null() || (*menu).modes & modes == 0 as ::core::ffi::c_int {
            return ::core::ptr::null_mut::<dict_T>();
        }
        let mut dict: *mut dict_T = tv_dict_alloc();
        tv_dict_add_str(
            dict,
            c"name".as_ptr(),
            ::core::mem::size_of::<[::core::ffi::c_char; 5]>().wrapping_sub(1 as size_t),
            (*menu).dname,
        );
        tv_dict_add_nr(
            dict,
            c"priority".as_ptr(),
            ::core::mem::size_of::<[::core::ffi::c_char; 9]>().wrapping_sub(1 as size_t),
            (*menu).priority as varnumber_T,
        );
        tv_dict_add_nr(
            dict,
            c"hidden".as_ptr(),
            ::core::mem::size_of::<[::core::ffi::c_char; 7]>().wrapping_sub(1 as size_t),
            menu_is_hidden((*menu).dname) as varnumber_T,
        );
        if (*menu).mnemonic != 0 {
            let mut buf: [::core::ffi::c_char; 7] = [0 as ::core::ffi::c_char, 0, 0, 0, 0, 0, 0];
            utf_char2bytes((*menu).mnemonic, &raw mut buf as *mut ::core::ffi::c_char);
            tv_dict_add_str(
                dict,
                c"shortcut".as_ptr(),
                ::core::mem::size_of::<[::core::ffi::c_char; 9]>().wrapping_sub(1 as size_t),
                &raw mut buf as *mut ::core::ffi::c_char,
            );
        }
        if !(*menu).actext.is_null() {
            tv_dict_add_str(
                dict,
                c"actext".as_ptr(),
                ::core::mem::size_of::<[::core::ffi::c_char; 7]>().wrapping_sub(1 as size_t),
                (*menu).actext,
            );
        }
        if (*menu).modes & MENU_TIP_MODE as ::core::ffi::c_int != 0
            && !(*menu).strings[MENU_INDEX_TIP as ::core::ffi::c_int as usize].is_null()
        {
            tv_dict_add_str(
                dict,
                c"tooltip".as_ptr(),
                ::core::mem::size_of::<[::core::ffi::c_char; 8]>().wrapping_sub(1 as size_t),
                (*menu).strings[MENU_INDEX_TIP as ::core::ffi::c_int as usize],
            );
        }
        if (*menu).children.is_null() {
            let mut commands: *mut dict_T = tv_dict_alloc();
            tv_dict_add_dict(
                dict,
                c"mappings".as_ptr(),
                ::core::mem::size_of::<[::core::ffi::c_char; 9]>().wrapping_sub(1 as size_t),
                commands,
            );
            let mut bit: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
            while bit < MENU_MODES as ::core::ffi::c_int {
                if (*menu).modes & modes & (1 as ::core::ffi::c_int) << bit
                    != 0 as ::core::ffi::c_int
                {
                    let mut impl_0: *mut dict_T = tv_dict_alloc();
                    tv_dict_add_allocated_str(
                        impl_0,
                        c"rhs".as_ptr(),
                        ::core::mem::size_of::<[::core::ffi::c_char; 4]>()
                            .wrapping_sub(1 as size_t),
                        str2special_save((*menu).strings[bit as usize], false_0 != 0, false_0 != 0),
                    );
                    tv_dict_add_nr(
                        impl_0,
                        c"silent".as_ptr(),
                        ::core::mem::size_of::<[::core::ffi::c_char; 7]>()
                            .wrapping_sub(1 as size_t),
                        (*menu).silent[bit as usize] as varnumber_T,
                    );
                    tv_dict_add_nr(
                        impl_0,
                        c"enabled".as_ptr(),
                        ::core::mem::size_of::<[::core::ffi::c_char; 8]>()
                            .wrapping_sub(1 as size_t),
                        (if (*menu).enabled & (1 as ::core::ffi::c_int) << bit != 0 {
                            1 as ::core::ffi::c_int
                        } else {
                            0 as ::core::ffi::c_int
                        }) as varnumber_T,
                    );
                    tv_dict_add_nr(
                        impl_0,
                        c"noremap".as_ptr(),
                        ::core::mem::size_of::<[::core::ffi::c_char; 8]>()
                            .wrapping_sub(1 as size_t),
                        (if (*menu).noremap[bit as usize] & REMAP_NONE as ::core::ffi::c_int != 0 {
                            1 as ::core::ffi::c_int
                        } else {
                            0 as ::core::ffi::c_int
                        }) as varnumber_T,
                    );
                    tv_dict_add_nr(
                        impl_0,
                        c"sid".as_ptr(),
                        ::core::mem::size_of::<[::core::ffi::c_char; 4]>()
                            .wrapping_sub(1 as size_t),
                        (if (*menu).noremap[bit as usize] & REMAP_SCRIPT as ::core::ffi::c_int != 0
                        {
                            1 as ::core::ffi::c_int
                        } else {
                            0 as ::core::ffi::c_int
                        }) as varnumber_T,
                    );
                    tv_dict_add_dict(
                        commands,
                        (*menu_mode_chars.ptr())[bit as usize],
                        1 as size_t,
                        impl_0,
                    );
                }
                bit += 1;
            }
        } else {
            let children_list: *mut list_T =
                tv_list_alloc(kListLenMayKnow as ::core::ffi::c_int as ptrdiff_t);
            menu = (*menu).children;
            while !menu.is_null() {
                let mut d: *mut dict_T = menu_get_recursive(menu, modes);
                if tv_dict_len(d) > 0 as ::core::ffi::c_long {
                    tv_list_append_dict(children_list, d);
                }
                menu = (*menu).next;
            }
            tv_dict_add_list(
                dict,
                c"submenus".as_ptr(),
                ::core::mem::size_of::<[::core::ffi::c_char; 9]>().wrapping_sub(1 as size_t),
                children_list,
            );
        }
        return dict;
    }
}

pub unsafe extern "C" fn menu_get(
    path_name: *mut ::core::ffi::c_char,
    mut modes: ::core::ffi::c_int,
    mut list: *mut list_T,
) -> bool {
    unsafe {
        let mut menu: *mut vimmenu_T = *get_root_menu(path_name);
        if *path_name as ::core::ffi::c_int != NUL {
            menu = find_menu(menu, path_name, modes);
            if menu.is_null() {
                return false_0 != 0;
            }
        }
        while !menu.is_null() {
            let mut d: *mut dict_T = menu_get_recursive(menu, modes);
            if !d.is_null() && tv_dict_len(d) > 0 as ::core::ffi::c_long {
                tv_list_append_dict(list, d);
            }
            if *path_name as ::core::ffi::c_int != NUL {
                break;
            }
            menu = (*menu).next;
        }
        return true_0 != 0;
    }
}

unsafe extern "C" fn find_menu(
    mut menu: *mut vimmenu_T,
    mut path_name: *const ::core::ffi::c_char,
    mut modes: ::core::ffi::c_int,
) -> *mut vimmenu_T {
    unsafe {
        debug_assert!(*path_name != 0, "*path_name");
        let saved_name: *mut ::core::ffi::c_char = xstrdup(path_name);
        let mut name: *mut ::core::ffi::c_char = saved_name;
        '_theend: while *name != 0 {
            let mut p: *mut ::core::ffi::c_char = menu_name_skip(name);
            while !menu.is_null() {
                if menu_name_equal(name, menu) {
                    if *p as ::core::ffi::c_int != NUL && (*menu).children.is_null() {
                        emsg(gettext(e_notsubmenu.as_ptr()));
                        menu = ::core::ptr::null_mut::<vimmenu_T>();
                        break '_theend;
                    } else if (*menu).modes & modes == 0 as ::core::ffi::c_int {
                        emsg(gettext(
                            &raw const e_menu_only_exists_in_another_mode
                                as *const ::core::ffi::c_char,
                        ));
                        menu = ::core::ptr::null_mut::<vimmenu_T>();
                        break '_theend;
                    } else if *p as ::core::ffi::c_int == NUL {
                        break '_theend;
                    } else {
                        break;
                    }
                } else {
                    menu = (*menu).next;
                }
            }
            if menu.is_null() {
                semsg_c!(gettext(e_nomenu.as_ptr()), name,);
                break;
            } else {
                name = p;
                debug_assert!(*name != 0, "*name");
                menu = (*menu).children;
            }
        }
        xfree(saved_name as *mut ::core::ffi::c_void);
        return menu;
    }
}

pub(crate) unsafe extern "C" fn show_menus(
    path_name: *mut ::core::ffi::c_char,
    mut modes: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    unsafe {
        let mut menu: *mut vimmenu_T = ::core::ptr::null_mut::<vimmenu_T>();
        if *path_name as ::core::ffi::c_int != NUL {
            menu = find_menu(*get_root_menu(path_name), path_name, modes);
            if menu.is_null() {
                return FAIL;
            }
        }
        (*menus_locked.ptr()) += 1;
        msg_puts_title(gettext(c"\n--- Menus ---".as_ptr()));
        show_menus_recursive(menu, modes, 0 as ::core::ffi::c_int);
        (*menus_locked.ptr()) -= 1;
        return OK;
    }
}

unsafe extern "C" fn show_menus_recursive(
    mut menu: *mut vimmenu_T,
    mut modes: ::core::ffi::c_int,
    mut depth: ::core::ffi::c_int,
) {
    unsafe {
        if !menu.is_null() && (*menu).modes & modes == 0 as ::core::ffi::c_int {
            return;
        }
        if !menu.is_null() {
            msg_putchar('\n' as ::core::ffi::c_int);
            if got_int.get() {
                return;
            }
            let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
            while i < depth {
                msg_puts(c"  ".as_ptr());
                i += 1;
            }
            if (*menu).priority != 0 {
                msg_outnum((*menu).priority);
                msg_puts(c" ".as_ptr());
            }
            msg_outtrans((*menu).name, HLF_D, false_0 != 0);
        }
        if !menu.is_null() && (*menu).children.is_null() {
            let mut bit: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
            while bit < MENU_MODES as ::core::ffi::c_int {
                if (*menu).modes & modes & (1 as ::core::ffi::c_int) << bit
                    != 0 as ::core::ffi::c_int
                {
                    msg_putchar('\n' as ::core::ffi::c_int);
                    if got_int.get() {
                        return;
                    }
                    let mut i_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                    while i_0 < depth + 2 as ::core::ffi::c_int {
                        msg_puts(c"  ".as_ptr());
                        i_0 += 1;
                    }
                    msg_puts((*menu_mode_chars.ptr())[bit as usize]);
                    if (*menu).noremap[bit as usize] == REMAP_NONE as ::core::ffi::c_int {
                        msg_putchar('*' as ::core::ffi::c_int);
                    } else if (*menu).noremap[bit as usize] == REMAP_SCRIPT as ::core::ffi::c_int {
                        msg_putchar('&' as ::core::ffi::c_int);
                    } else {
                        msg_putchar(' ' as ::core::ffi::c_int);
                    }
                    if (*menu).silent[bit as usize] {
                        msg_putchar('s' as ::core::ffi::c_int);
                    } else {
                        msg_putchar(' ' as ::core::ffi::c_int);
                    }
                    if (*menu).modes & (*menu).enabled & (1 as ::core::ffi::c_int) << bit
                        == 0 as ::core::ffi::c_int
                    {
                        msg_putchar('-' as ::core::ffi::c_int);
                    } else {
                        msg_putchar(' ' as ::core::ffi::c_int);
                    }
                    msg_puts(c" ".as_ptr());
                    if *(*menu).strings[bit as usize] as ::core::ffi::c_int == NUL {
                        msg_puts_hl(c"<Nop>".as_ptr(), HLF_8, false_0 != 0);
                    } else {
                        msg_outtrans_special(
                            (*menu).strings[bit as usize],
                            false_0 != 0,
                            0 as ::core::ffi::c_int,
                        );
                    }
                }
                bit += 1;
            }
        } else {
            if menu.is_null() {
                menu = root_menu.get();
                depth -= 1;
            } else {
                menu = (*menu).children;
            }
            while !menu.is_null() && !got_int.get() {
                if !menu_is_hidden((*menu).dname) {
                    show_menus_recursive(menu, modes, depth + 1 as ::core::ffi::c_int);
                }
                menu = (*menu).next;
            }
        };
    }
}
