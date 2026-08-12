//! Defining a menu -- `:menu` and everything it parses.
//!
//! [`ex_menu`] is the whole command line: the `<silent>`/`<script>`/`<special>`
//! modifiers, the `80.5.10` priority, the mode prefix, the `\ `-escaped path
//! and the right-hand side, plus the `:unmenu` and `:menu`-as-a-listing forms.
//! [`add_menu_path`] then walks the path one component at a time, creating the
//! `vimmenu_T` nodes that do not exist yet, inserting each at the place its
//! priority asks for, and finally storing the rhs for every mode the command
//! named.
//!
//! Original: `src/nvim/menu.c`, Vim/Neovim, Vim license.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;
use crate::semsg_c;
use crate::src::nvim::ascii::{ascii_isdigit, ascii_iswhite};
use crate::src::nvim::charset::{getdigits_int, skipwhite};
use crate::src::nvim::keycodes::{Ctrl_BSL, Ctrl_C, Ctrl_G, Ctrl_O, replace_termcodes};
use crate::src::nvim::main::{e_invarg2, e_trailing_arg, p_cpo, sys_menu};
use crate::src::nvim::mbyte::utfc_ptr2len;
use crate::src::nvim::memory::{xcalloc, xfree, xmalloc, xstrdup};
use crate::src::nvim::message::emsg;
use crate::src::nvim::os::libc::{gettext, memmove, strcasecmp, strcmp, strcpy, strlen, strncmp};
use crate::src::nvim::types::{
    TriState, exarg_T, kFalse, kNone, kTrue, linenr_T, scid_T, size_t, vimmenu_T,
};
use crate::src::nvim::ui::ui_call_update_menu;

pub unsafe fn ex_menu(mut eap: *mut exarg_T) {
    unsafe {
        let mut map_to: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
        let mut noremap: ::core::ffi::c_int = 0;
        let mut silent: bool = false_0 != 0;
        let mut unmenu: bool = false;
        let mut map_buf: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
        let mut p: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
        let mut i: ::core::ffi::c_int = 0;
        let mut pri_tab: [::core::ffi::c_int; 11] = [0; 11];
        let mut enable: TriState = kNone;
        let mut menuarg: vimmenu_T = vimmenu_T {
            modes: 0,
            enabled: 0,
            name: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            dname: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            en_name: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            en_dname: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            mnemonic: 0,
            actext: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            priority: 0,
            strings: [::core::ptr::null_mut::<::core::ffi::c_char>(); 8],
            noremap: [0; 8],
            silent: [false; 8],
            children: ::core::ptr::null_mut::<vimmenu_T>(),
            parent: ::core::ptr::null_mut::<vimmenu_T>(),
            next: ::core::ptr::null_mut::<vimmenu_T>(),
        };
        let mut modes: ::core::ffi::c_int = get_menu_cmd_modes(
            (*eap).cmd,
            (*eap).forceit != 0,
            &raw mut noremap,
            &raw mut unmenu,
        );
        let mut arg: *mut ::core::ffi::c_char = (*eap).arg;
        loop {
            if strncmp(arg, c"<script>".as_ptr(), 8 as size_t) == 0 as ::core::ffi::c_int {
                noremap = REMAP_SCRIPT as ::core::ffi::c_int;
                arg = skipwhite(arg.offset(8 as ::core::ffi::c_int as isize));
            } else if strncmp(arg, c"<silent>".as_ptr(), 8 as size_t) == 0 as ::core::ffi::c_int {
                silent = true_0 != 0;
                arg = skipwhite(arg.offset(8 as ::core::ffi::c_int as isize));
            } else {
                if strncmp(arg, c"<special>".as_ptr(), 9 as size_t) != 0 as ::core::ffi::c_int {
                    break;
                }
                arg = skipwhite(arg.offset(9 as ::core::ffi::c_int as isize));
            }
        }
        if strncmp(arg, c"icon=".as_ptr(), 5 as size_t) == 0 as ::core::ffi::c_int {
            arg = arg.offset(5 as ::core::ffi::c_int as isize);
            while *arg as ::core::ffi::c_int != NUL
                && *arg as ::core::ffi::c_int != ' ' as ::core::ffi::c_int
            {
                if *arg as ::core::ffi::c_int == '\\' as ::core::ffi::c_int {
                    memmove(
                        arg as *mut ::core::ffi::c_void,
                        arg.offset(1 as ::core::ffi::c_int as isize) as *const ::core::ffi::c_void,
                        strlen(arg.offset(1 as ::core::ffi::c_int as isize))
                            .wrapping_add(1 as size_t),
                    );
                }
                arg = arg.offset(utfc_ptr2len(arg) as isize);
            }
            if *arg as ::core::ffi::c_int != NUL {
                let c2rust_fresh0 = arg;
                arg = arg.offset(1);
                *c2rust_fresh0 = NUL as ::core::ffi::c_char;
                arg = skipwhite(arg);
            }
        }
        p = arg;
        while *p != 0 {
            if !ascii_isdigit(*p as ::core::ffi::c_int)
                && *p as ::core::ffi::c_int != '.' as ::core::ffi::c_int
            {
                break;
            }
            p = p.offset(1);
        }
        if ascii_iswhite(*p as ::core::ffi::c_int) {
            i = 0 as ::core::ffi::c_int;
            while i < MENUDEPTH && !ascii_iswhite(*arg as ::core::ffi::c_int) {
                pri_tab[i as usize] =
                    getdigits_int(&raw mut arg, false_0 != 0, 0 as ::core::ffi::c_int);
                if pri_tab[i as usize] == 0 as ::core::ffi::c_int {
                    pri_tab[i as usize] = 500 as ::core::ffi::c_int;
                }
                if *arg as ::core::ffi::c_int == '.' as ::core::ffi::c_int {
                    arg = arg.offset(1);
                }
                i += 1;
            }
            arg = skipwhite(arg);
        } else if (*eap).addr_count != 0 && (*eap).line2 != 0 as linenr_T {
            pri_tab[0 as ::core::ffi::c_int as usize] = (*eap).line2 as ::core::ffi::c_int;
            i = 1 as ::core::ffi::c_int;
        } else {
            i = 0 as ::core::ffi::c_int;
        }
        while i < MENUDEPTH {
            let c2rust_fresh1 = i;
            i = i + 1;
            pri_tab[c2rust_fresh1 as usize] = 500 as ::core::ffi::c_int;
        }
        pri_tab[MENUDEPTH as usize] = -1 as ::core::ffi::c_int;
        if strncmp(arg, c"enable".as_ptr(), 6 as size_t) == 0 as ::core::ffi::c_int
            && ascii_iswhite(*arg.offset(6 as ::core::ffi::c_int as isize) as ::core::ffi::c_int)
                as ::core::ffi::c_int
                != 0
        {
            enable = kTrue;
            arg = skipwhite(arg.offset(6 as ::core::ffi::c_int as isize));
        } else if strncmp(arg, c"disable".as_ptr(), 7 as size_t) == 0 as ::core::ffi::c_int
            && ascii_iswhite(*arg.offset(7 as ::core::ffi::c_int as isize) as ::core::ffi::c_int)
                as ::core::ffi::c_int
                != 0
        {
            enable = kFalse;
            arg = skipwhite(arg.offset(7 as ::core::ffi::c_int as isize));
        }
        if *arg as ::core::ffi::c_int == NUL {
            show_menus(arg, modes);
            return;
        }
        let mut menu_path: *mut ::core::ffi::c_char = arg;
        's_573: {
            if *menu_path as ::core::ffi::c_int == '.' as ::core::ffi::c_int {
                semsg_c!(
                    gettext(&raw const e_invarg2 as *const ::core::ffi::c_char),
                    menu_path,
                );
            } else {
                map_to = menu_translate_tab_and_shift(arg);
                if *map_to as ::core::ffi::c_int == NUL
                    && !unmenu
                    && enable as ::core::ffi::c_int == kNone as ::core::ffi::c_int
                {
                    show_menus(menu_path, modes);
                } else if *map_to as ::core::ffi::c_int != NUL
                    && (unmenu as ::core::ffi::c_int != 0
                        || enable as ::core::ffi::c_int != kNone as ::core::ffi::c_int)
                {
                    semsg_c!(
                        gettext(&raw const e_trailing_arg as *const ::core::ffi::c_char),
                        map_to,
                    );
                } else {
                    let mut root_menu_ptr: *mut *mut vimmenu_T = get_root_menu(menu_path);
                    if enable as ::core::ffi::c_int != kNone as ::core::ffi::c_int {
                        if strcmp(menu_path, c"*".as_ptr()) == 0 as ::core::ffi::c_int {
                            menu_path = c"".as_ptr() as *mut ::core::ffi::c_char;
                        }
                        if menu_is_popup(menu_path) {
                            i = 0 as ::core::ffi::c_int;
                            while i < MENU_INDEX_TIP as ::core::ffi::c_int {
                                if modes & (1 as ::core::ffi::c_int) << i != 0 {
                                    p = popup_mode_name(menu_path, i);
                                    menu_enable_recurse(
                                        *root_menu_ptr,
                                        p,
                                        MENU_ALL_MODES as ::core::ffi::c_int,
                                        enable as ::core::ffi::c_int,
                                    );
                                    xfree(p as *mut ::core::ffi::c_void);
                                }
                                i += 1;
                            }
                        }
                        menu_enable_recurse(
                            *root_menu_ptr,
                            menu_path,
                            modes,
                            enable as ::core::ffi::c_int,
                        );
                    } else if unmenu {
                        if is_menus_locked() != 0 {
                            break 's_573;
                        } else {
                            if strcmp(menu_path, c"*".as_ptr()) == 0 as ::core::ffi::c_int {
                                menu_path = c"".as_ptr() as *mut ::core::ffi::c_char;
                            }
                            if menu_is_popup(menu_path) {
                                i = 0 as ::core::ffi::c_int;
                                while i < MENU_INDEX_TIP as ::core::ffi::c_int {
                                    if modes & (1 as ::core::ffi::c_int) << i != 0 {
                                        p = popup_mode_name(menu_path, i);
                                        remove_menu(
                                            root_menu_ptr,
                                            p,
                                            MENU_ALL_MODES as ::core::ffi::c_int,
                                            true_0 != 0,
                                        );
                                        xfree(p as *mut ::core::ffi::c_void);
                                    }
                                    i += 1;
                                }
                            }
                            remove_menu(root_menu_ptr, menu_path, modes, false_0 != 0);
                        }
                    } else if is_menus_locked() != 0 {
                        break 's_573;
                    } else {
                        if strcasecmp(map_to, c"<nop>".as_ptr() as *mut ::core::ffi::c_char)
                            == 0 as ::core::ffi::c_int
                        {
                            map_to = c"".as_ptr() as *mut ::core::ffi::c_char;
                            map_buf = ::core::ptr::null_mut::<::core::ffi::c_char>();
                        } else if modes & MENU_TIP_MODE as ::core::ffi::c_int != 0 {
                            map_buf = ::core::ptr::null_mut::<::core::ffi::c_char>();
                        } else {
                            map_buf = ::core::ptr::null_mut::<::core::ffi::c_char>();
                            map_to = replace_termcodes(
                                map_to,
                                strlen(map_to),
                                &raw mut map_buf,
                                0 as scid_T,
                                REPTERM_DO_LT as ::core::ffi::c_int,
                                ::core::ptr::null_mut::<bool>(),
                                p_cpo.get(),
                            );
                        }
                        menuarg.modes = modes;
                        menuarg.noremap[0 as ::core::ffi::c_int as usize] = noremap;
                        menuarg.silent[0 as ::core::ffi::c_int as usize] = silent;
                        add_menu_path(
                            menu_path,
                            &raw mut menuarg,
                            &raw mut pri_tab as *mut ::core::ffi::c_int,
                            map_to,
                        );
                        if menu_is_popup(menu_path) {
                            i = 0 as ::core::ffi::c_int;
                            while i < MENU_INDEX_TIP as ::core::ffi::c_int {
                                if modes & (1 as ::core::ffi::c_int) << i != 0 {
                                    p = popup_mode_name(menu_path, i);
                                    menuarg.modes = modes;
                                    add_menu_path(
                                        p,
                                        &raw mut menuarg,
                                        &raw mut pri_tab as *mut ::core::ffi::c_int,
                                        map_to,
                                    );
                                    xfree(p as *mut ::core::ffi::c_void);
                                }
                                i += 1;
                            }
                        }
                        xfree(map_buf as *mut ::core::ffi::c_void);
                    }
                    ui_call_update_menu();
                }
            }
        };
    }
}

unsafe extern "C" fn add_menu_path(
    menu_path: *const ::core::ffi::c_char,
    mut menuarg: *mut vimmenu_T,
    pri_tab: *const ::core::ffi::c_int,
    call_data: *const ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    unsafe {
        let mut amenu: ::core::ffi::c_int = 0;
        let mut modes: ::core::ffi::c_int = (*menuarg).modes;
        let mut menu: *mut vimmenu_T = ::core::ptr::null_mut::<vimmenu_T>();
        let mut lower_pri: *mut *mut vimmenu_T = ::core::ptr::null_mut::<*mut vimmenu_T>();
        let mut dname: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
        let mut pri_idx: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        let mut old_modes: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        let mut en_name: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
        let mut path_name: *mut ::core::ffi::c_char = xstrdup(menu_path);
        let mut root_menu_ptr: *mut *mut vimmenu_T = get_root_menu(menu_path);
        let mut menup: *mut *mut vimmenu_T = root_menu_ptr;
        let mut parent: *mut vimmenu_T = ::core::ptr::null_mut::<vimmenu_T>();
        let mut name: *mut ::core::ffi::c_char = path_name;
        '_erret: {
            while *name != 0 {
                let mut next_name: *mut ::core::ffi::c_char = menu_name_skip(name);
                let mut map_to: *mut ::core::ffi::c_char =
                    menutrans_lookup(name, strlen(name) as ::core::ffi::c_int);
                if !map_to.is_null() {
                    en_name = name;
                    name = map_to;
                } else {
                    en_name = ::core::ptr::null_mut::<::core::ffi::c_char>();
                }
                dname = menu_text(
                    name,
                    ::core::ptr::null_mut::<::core::ffi::c_int>(),
                    ::core::ptr::null_mut::<*mut ::core::ffi::c_char>(),
                );
                if *dname as ::core::ffi::c_int == NUL {
                    emsg(gettext(c"E792: Empty menu name".as_ptr()));
                    break '_erret;
                } else {
                    lower_pri = menup;
                    menu = *menup;
                    while !menu.is_null() {
                        if menu_name_equal(name, menu) as ::core::ffi::c_int != 0
                            || menu_name_equal(dname, menu) as ::core::ffi::c_int != 0
                        {
                            if *next_name as ::core::ffi::c_int == NUL
                                && !(*menu).children.is_null()
                            {
                                if !sys_menu.get() {
                                    emsg(gettext(
                                        c"E330: Menu path must not lead to a sub-menu".as_ptr(),
                                    ));
                                }
                                break '_erret;
                            } else {
                                if !(*next_name as ::core::ffi::c_int != NUL
                                    && (*menu).children.is_null())
                                {
                                    break;
                                }
                                if !sys_menu.get() {
                                    emsg(gettext(e_notsubmenu.as_ptr()));
                                }
                                break '_erret;
                            }
                        } else {
                            menup = &raw mut (*menu).next;
                            if !parent.is_null()
                                || menu_is_menubar((*menu).name) as ::core::ffi::c_int != 0
                            {
                                if (*menu).priority <= *pri_tab.offset(pri_idx as isize) {
                                    lower_pri = menup;
                                }
                            }
                            menu = (*menu).next;
                        }
                    }
                    if menu.is_null() {
                        if *next_name as ::core::ffi::c_int == NUL && parent.is_null() {
                            emsg(gettext(
                                c"E331: Must not add menu items directly to menu bar".as_ptr(),
                            ));
                            break '_erret;
                        } else if menu_is_separator(dname) as ::core::ffi::c_int != 0
                            && *next_name as ::core::ffi::c_int != NUL
                        {
                            emsg(gettext(
                                c"E332: Separator cannot be part of a menu path".as_ptr(),
                            ));
                            break '_erret;
                        } else {
                            menu = xcalloc(1 as size_t, ::core::mem::size_of::<vimmenu_T>())
                                as *mut vimmenu_T;
                            (*menu).modes = modes;
                            (*menu).enabled = MENU_ALL_MODES as ::core::ffi::c_int;
                            (*menu).name = xstrdup(name);
                            (*menu).dname =
                                menu_text(name, &raw mut (*menu).mnemonic, &raw mut (*menu).actext);
                            if !en_name.is_null() {
                                (*menu).en_name = xstrdup(en_name);
                                (*menu).en_dname = menu_text(
                                    en_name,
                                    ::core::ptr::null_mut::<::core::ffi::c_int>(),
                                    ::core::ptr::null_mut::<*mut ::core::ffi::c_char>(),
                                );
                            } else {
                                (*menu).en_name = ::core::ptr::null_mut::<::core::ffi::c_char>();
                                (*menu).en_dname = ::core::ptr::null_mut::<::core::ffi::c_char>();
                            }
                            (*menu).priority = *pri_tab.offset(pri_idx as isize);
                            (*menu).parent = parent;
                            (*menu).next = *lower_pri;
                            *lower_pri = menu;
                            old_modes = 0 as ::core::ffi::c_int;
                        }
                    } else {
                        old_modes = (*menu).modes;
                        (*menu).modes |= modes;
                        (*menu).enabled |= modes;
                    }
                    menup = &raw mut (*menu).children;
                    parent = menu;
                    name = next_name;
                    let mut ptr_: *mut *mut ::core::ffi::c_void =
                        &raw mut dname as *mut *mut ::core::ffi::c_void;
                    xfree(*ptr_);
                    *ptr_ = NULL;
                    let _ = *ptr_;
                    if *pri_tab.offset((pri_idx + 1 as ::core::ffi::c_int) as isize)
                        != -1 as ::core::ffi::c_int
                    {
                        pri_idx += 1;
                    }
                }
            }
            xfree(path_name as *mut ::core::ffi::c_void);
            amenu = (modes
                & (MENU_NORMAL_MODE as ::core::ffi::c_int | MENU_INSERT_MODE as ::core::ffi::c_int)
                == MENU_NORMAL_MODE as ::core::ffi::c_int | MENU_INSERT_MODE as ::core::ffi::c_int)
                as ::core::ffi::c_int;
            if sys_menu.get() {
                modes &= !old_modes;
            }
            if !menu.is_null() && modes != 0 {
                let mut p: *mut ::core::ffi::c_char = if call_data.is_null() {
                    ::core::ptr::null_mut::<::core::ffi::c_char>()
                } else {
                    xstrdup(call_data)
                };
                let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                while i < MENU_MODES as ::core::ffi::c_int {
                    if modes & (1 as ::core::ffi::c_int) << i != 0 {
                        free_menu_string(menu, i);
                        let mut c: ::core::ffi::c_char = 0 as ::core::ffi::c_char;
                        let mut d: ::core::ffi::c_char = 0 as ::core::ffi::c_char;
                        if amenu != 0
                            && !call_data.is_null()
                            && *call_data as ::core::ffi::c_int != NUL
                        {
                            match (1 as ::core::ffi::c_int) << i {
                                2 | 4 | 8 | 32 => {
                                    c = Ctrl_C as ::core::ffi::c_char;
                                }
                                16 => {
                                    c = Ctrl_BSL as ::core::ffi::c_char;
                                    d = Ctrl_O as ::core::ffi::c_char;
                                }
                                _ => {}
                            }
                        }
                        if c as ::core::ffi::c_int != 0 as ::core::ffi::c_int {
                            (*menu).strings[i as usize] =
                                xmalloc(strlen(call_data).wrapping_add(5 as size_t))
                                    as *mut ::core::ffi::c_char;
                            *(*menu).strings[i as usize].offset(0 as ::core::ffi::c_int as isize) =
                                c;
                            if d as ::core::ffi::c_int == 0 as ::core::ffi::c_int {
                                strcpy(
                                    (*menu).strings[i as usize]
                                        .offset(1 as ::core::ffi::c_int as isize),
                                    call_data as *mut ::core::ffi::c_char,
                                );
                            } else {
                                *(*menu).strings[i as usize]
                                    .offset(1 as ::core::ffi::c_int as isize) = d;
                                strcpy(
                                    (*menu).strings[i as usize]
                                        .offset(2 as ::core::ffi::c_int as isize),
                                    call_data as *mut ::core::ffi::c_char,
                                );
                            }
                            if c as ::core::ffi::c_int == Ctrl_C {
                                let mut len: ::core::ffi::c_int =
                                    strlen((*menu).strings[i as usize]) as ::core::ffi::c_int;
                                *(*menu).strings[i as usize].offset(len as isize) =
                                    Ctrl_BSL as ::core::ffi::c_char;
                                *(*menu).strings[i as usize]
                                    .offset((len + 1 as ::core::ffi::c_int) as isize) =
                                    Ctrl_G as ::core::ffi::c_char;
                                *(*menu).strings[i as usize]
                                    .offset((len + 2 as ::core::ffi::c_int) as isize) =
                                    NUL as ::core::ffi::c_char;
                            }
                        } else {
                            (*menu).strings[i as usize] = p;
                        }
                        (*menu).noremap[i as usize] =
                            (*menuarg).noremap[0 as ::core::ffi::c_int as usize];
                        (*menu).silent[i as usize] =
                            (*menuarg).silent[0 as ::core::ffi::c_int as usize];
                    }
                    i += 1;
                }
            }
            return OK;
        }
        xfree(path_name as *mut ::core::ffi::c_void);
        xfree(dname as *mut ::core::ffi::c_void);
        while !parent.is_null() && (*parent).children.is_null() {
            if (*parent).parent.is_null() {
                menup = root_menu_ptr;
            } else {
                menup = &raw mut (*(*parent).parent).children;
            }
            while !(*menup).is_null() && *menup != parent {
                menup = &raw mut (**menup).next;
            }
            if (*menup).is_null() {
                break;
            }
            parent = (*parent).parent;
            free_menu(menup);
        }
        return FAIL;
    }
}
