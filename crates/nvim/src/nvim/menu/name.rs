//! Menu names as text -- parsing a path, matching one, and the mode
//! letters.
//!
//! [`menu_name_skip`] steps over one `\ `-escaped path component;
//! [`menu_name_equal`]/[`menu_namecmp`] compare a component against a node,
//! ignoring the `&` mnemonic marker.  [`get_menu_cmd_modes`] maps the command
//! name (`nmenu`, `vnoremenu`, `amenu!`, ...) onto the mode bitmask and the
//! `:noremap` flag, [`get_menu_mode_str`] and [`popup_mode_name`] go the other
//! way, and [`menu_text`] splits a name into the displayed text and the
//! `<Tab>`-separated accelerator, dropping the mnemonic `&`.
//!
//! Original: `src/nvim/menu.c`, Vim/Neovim, Vim license.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;
use crate::src::nvim::keycodes::Ctrl_V;
use crate::src::nvim::mbyte::utfc_ptr2len;
use crate::src::nvim::memory::{xmemdupz, xstrdup};
use crate::src::nvim::os::libc::{memmove, strlen};
use crate::src::nvim::strings::{vim_strchr, xstrnsave};
use crate::src::nvim::types::{size_t, uint8_t, vimmenu_T};

pub(crate) unsafe extern "C" fn menu_name_skip(
    name: *mut ::core::ffi::c_char,
) -> *mut ::core::ffi::c_char {
    unsafe {
        let mut p: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
        p = name;
        while *p as ::core::ffi::c_int != 0 && *p as ::core::ffi::c_int != '.' as ::core::ffi::c_int
        {
            if *p as ::core::ffi::c_int == '\\' as ::core::ffi::c_int
                || *p as ::core::ffi::c_int == Ctrl_V
            {
                memmove(
                    p as *mut ::core::ffi::c_void,
                    p.offset(1 as ::core::ffi::c_int as isize) as *const ::core::ffi::c_void,
                    strlen(p.offset(1 as ::core::ffi::c_int as isize)).wrapping_add(1 as size_t),
                );
                if *p as ::core::ffi::c_int == NUL {
                    break;
                }
            }
            p = p.offset(utfc_ptr2len(p) as isize);
        }
        if *p != 0 {
            let c2rust_fresh2 = p;
            p = p.offset(1);
            *c2rust_fresh2 = NUL as ::core::ffi::c_char;
        }
        return p;
    }
}

pub(crate) unsafe extern "C" fn menu_name_equal(
    name: *const ::core::ffi::c_char,
    menu: *const vimmenu_T,
) -> bool {
    unsafe {
        if !(*menu).en_name.is_null()
            && (menu_namecmp(name, (*menu).en_name) as ::core::ffi::c_int != 0
                || menu_namecmp(name, (*menu).en_dname) as ::core::ffi::c_int != 0)
        {
            return true_0 != 0;
        }
        return menu_namecmp(name, (*menu).name) as ::core::ffi::c_int != 0
            || menu_namecmp(name, (*menu).dname) as ::core::ffi::c_int != 0;
    }
}

unsafe extern "C" fn menu_namecmp(
    name: *const ::core::ffi::c_char,
    mname: *const ::core::ffi::c_char,
) -> bool {
    unsafe {
        let mut i: ::core::ffi::c_int = 0;
        i = 0 as ::core::ffi::c_int;
        while *name.offset(i as isize) as ::core::ffi::c_int != NUL
            && *name.offset(i as isize) as ::core::ffi::c_int != TAB
        {
            if *name.offset(i as isize) as ::core::ffi::c_int
                != *mname.offset(i as isize) as ::core::ffi::c_int
            {
                break;
            }
            i += 1;
        }
        return (*name.offset(i as isize) as ::core::ffi::c_int == NUL
            || *name.offset(i as isize) as ::core::ffi::c_int == TAB)
            && (*mname.offset(i as isize) as ::core::ffi::c_int == NUL
                || *mname.offset(i as isize) as ::core::ffi::c_int == TAB);
    }
}

pub unsafe extern "C" fn get_menu_cmd_modes(
    mut cmd: *const ::core::ffi::c_char,
    mut forceit: bool,
    mut noremap: *mut ::core::ffi::c_int,
    mut unmenu: *mut bool,
) -> ::core::ffi::c_int {
    unsafe {
        let mut modes: ::core::ffi::c_int = 0;
        's_121: {
            let c2rust_fresh3 = cmd;
            cmd = cmd.offset(1);
            match *c2rust_fresh3 as ::core::ffi::c_int {
                118 => {
                    modes = MENU_VISUAL_MODE as ::core::ffi::c_int
                        | MENU_SELECT_MODE as ::core::ffi::c_int;
                    break 's_121;
                }
                120 => {
                    modes = MENU_VISUAL_MODE as ::core::ffi::c_int;
                    break 's_121;
                }
                115 => {
                    modes = MENU_SELECT_MODE as ::core::ffi::c_int;
                    break 's_121;
                }
                111 => {
                    modes = MENU_OP_PENDING_MODE as ::core::ffi::c_int;
                    break 's_121;
                }
                105 => {
                    modes = MENU_INSERT_MODE as ::core::ffi::c_int;
                    break 's_121;
                }
                116 => {
                    if *cmd as ::core::ffi::c_int == 'l' as ::core::ffi::c_int {
                        modes = MENU_TERMINAL_MODE as ::core::ffi::c_int;
                        cmd = cmd.offset(1);
                        break 's_121;
                    } else {
                        modes = MENU_TIP_MODE as ::core::ffi::c_int;
                        break 's_121;
                    }
                }
                99 => {
                    modes = MENU_CMDLINE_MODE as ::core::ffi::c_int;
                    break 's_121;
                }
                97 => {
                    modes = MENU_INSERT_MODE as ::core::ffi::c_int
                        | MENU_CMDLINE_MODE as ::core::ffi::c_int
                        | MENU_NORMAL_MODE as ::core::ffi::c_int
                        | MENU_VISUAL_MODE as ::core::ffi::c_int
                        | MENU_SELECT_MODE as ::core::ffi::c_int
                        | MENU_OP_PENDING_MODE as ::core::ffi::c_int;
                    break 's_121;
                }
                110 => {
                    if *cmd as ::core::ffi::c_int != 'o' as ::core::ffi::c_int {
                        modes = MENU_NORMAL_MODE as ::core::ffi::c_int;
                        break 's_121;
                    }
                }
                _ => {}
            }
            cmd = cmd.offset(-1);
            if forceit {
                modes = MENU_INSERT_MODE as ::core::ffi::c_int
                    | MENU_CMDLINE_MODE as ::core::ffi::c_int;
            } else {
                modes = MENU_NORMAL_MODE as ::core::ffi::c_int
                    | MENU_VISUAL_MODE as ::core::ffi::c_int
                    | MENU_SELECT_MODE as ::core::ffi::c_int
                    | MENU_OP_PENDING_MODE as ::core::ffi::c_int;
            }
        }
        if !noremap.is_null() {
            *noremap = if *cmd as ::core::ffi::c_int == 'n' as ::core::ffi::c_int {
                REMAP_NONE as ::core::ffi::c_int
            } else {
                REMAP_YES as ::core::ffi::c_int
            };
        }
        if !unmenu.is_null() {
            *unmenu = *cmd as ::core::ffi::c_int == 'u' as ::core::ffi::c_int;
        }
        return modes;
    }
}

pub(crate) unsafe extern "C" fn get_menu_mode_str(
    mut modes: ::core::ffi::c_int,
) -> *mut ::core::ffi::c_char {
    if modes
        & (MENU_INSERT_MODE as ::core::ffi::c_int
            | MENU_CMDLINE_MODE as ::core::ffi::c_int
            | MENU_NORMAL_MODE as ::core::ffi::c_int
            | MENU_VISUAL_MODE as ::core::ffi::c_int
            | MENU_SELECT_MODE as ::core::ffi::c_int
            | MENU_OP_PENDING_MODE as ::core::ffi::c_int)
        == MENU_INSERT_MODE as ::core::ffi::c_int
            | MENU_CMDLINE_MODE as ::core::ffi::c_int
            | MENU_NORMAL_MODE as ::core::ffi::c_int
            | MENU_VISUAL_MODE as ::core::ffi::c_int
            | MENU_SELECT_MODE as ::core::ffi::c_int
            | MENU_OP_PENDING_MODE as ::core::ffi::c_int
    {
        return c"a".as_ptr() as *mut ::core::ffi::c_char;
    }
    if modes
        & (MENU_NORMAL_MODE as ::core::ffi::c_int
            | MENU_VISUAL_MODE as ::core::ffi::c_int
            | MENU_SELECT_MODE as ::core::ffi::c_int
            | MENU_OP_PENDING_MODE as ::core::ffi::c_int)
        == MENU_NORMAL_MODE as ::core::ffi::c_int
            | MENU_VISUAL_MODE as ::core::ffi::c_int
            | MENU_SELECT_MODE as ::core::ffi::c_int
            | MENU_OP_PENDING_MODE as ::core::ffi::c_int
    {
        return c" ".as_ptr() as *mut ::core::ffi::c_char;
    }
    if modes & (MENU_INSERT_MODE as ::core::ffi::c_int | MENU_CMDLINE_MODE as ::core::ffi::c_int)
        == MENU_INSERT_MODE as ::core::ffi::c_int | MENU_CMDLINE_MODE as ::core::ffi::c_int
    {
        return c"!".as_ptr() as *mut ::core::ffi::c_char;
    }
    if modes & (MENU_VISUAL_MODE as ::core::ffi::c_int | MENU_SELECT_MODE as ::core::ffi::c_int)
        == MENU_VISUAL_MODE as ::core::ffi::c_int | MENU_SELECT_MODE as ::core::ffi::c_int
    {
        return c"v".as_ptr() as *mut ::core::ffi::c_char;
    }
    if modes & MENU_VISUAL_MODE as ::core::ffi::c_int != 0 {
        return c"x".as_ptr() as *mut ::core::ffi::c_char;
    }
    if modes & MENU_SELECT_MODE as ::core::ffi::c_int != 0 {
        return c"s".as_ptr() as *mut ::core::ffi::c_char;
    }
    if modes & MENU_OP_PENDING_MODE as ::core::ffi::c_int != 0 {
        return c"o".as_ptr() as *mut ::core::ffi::c_char;
    }
    if modes & MENU_INSERT_MODE as ::core::ffi::c_int != 0 {
        return c"i".as_ptr() as *mut ::core::ffi::c_char;
    }
    if modes & MENU_TERMINAL_MODE as ::core::ffi::c_int != 0 {
        return c"tl".as_ptr() as *mut ::core::ffi::c_char;
    }
    if modes & MENU_CMDLINE_MODE as ::core::ffi::c_int != 0 {
        return c"c".as_ptr() as *mut ::core::ffi::c_char;
    }
    if modes & MENU_NORMAL_MODE as ::core::ffi::c_int != 0 {
        return c"n".as_ptr() as *mut ::core::ffi::c_char;
    }
    if modes & MENU_TIP_MODE as ::core::ffi::c_int != 0 {
        return c"t".as_ptr() as *mut ::core::ffi::c_char;
    }
    return c"".as_ptr() as *mut ::core::ffi::c_char;
}

pub(crate) unsafe extern "C" fn popup_mode_name(
    mut name: *mut ::core::ffi::c_char,
    mut idx: ::core::ffi::c_int,
) -> *mut ::core::ffi::c_char {
    unsafe {
        let mut len: size_t = strlen(name);
        debug_assert!(len >= 4 as size_t, "len >= 4");
        let mut mode_chars: *mut ::core::ffi::c_char = (*menu_mode_chars.ptr())[idx as usize];
        let mut mode_chars_len: size_t = strlen(mode_chars);
        let mut p: *mut ::core::ffi::c_char = xstrnsave(name, len.wrapping_add(mode_chars_len));
        memmove(
            p.offset(5 as ::core::ffi::c_int as isize)
                .add(mode_chars_len) as *mut ::core::ffi::c_void,
            p.offset(5 as ::core::ffi::c_int as isize) as *const ::core::ffi::c_void,
            len.wrapping_sub(4 as size_t),
        );
        let mut i: size_t = 0 as size_t;
        while i < mode_chars_len {
            *p.add((5 as size_t).wrapping_add(i)) = *(*menu_mode_chars.ptr())[idx as usize].add(i);
            i = i.wrapping_add(1);
        }
        return p;
    }
}

pub(crate) unsafe extern "C" fn menu_text(
    mut str: *const ::core::ffi::c_char,
    mut mnemonic: *mut ::core::ffi::c_int,
    mut actext: *mut *mut ::core::ffi::c_char,
) -> *mut ::core::ffi::c_char {
    unsafe {
        let mut text: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
        let mut p: *mut ::core::ffi::c_char = vim_strchr(str, TAB);
        if !p.is_null() {
            if !actext.is_null() {
                *actext = xstrdup(p.offset(1 as ::core::ffi::c_int as isize));
            }
            debug_assert!(p >= str as *mut ::core::ffi::c_char, "p >= str");
            text = xmemdupz(
                str as *const ::core::ffi::c_void,
                p.offset_from(str) as size_t,
            ) as *mut ::core::ffi::c_char;
        } else {
            text = xstrdup(str);
        }
        p = text;
        while !p.is_null() {
            p = vim_strchr(p, '&' as ::core::ffi::c_int);
            if p.is_null() {
                continue;
            }
            if *p.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == NUL {
                break;
            }
            if !mnemonic.is_null()
                && *p.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                    != '&' as ::core::ffi::c_int
            {
                *mnemonic =
                    *p.offset(1 as ::core::ffi::c_int as isize) as uint8_t as ::core::ffi::c_int;
            }
            memmove(
                p as *mut ::core::ffi::c_void,
                p.offset(1 as ::core::ffi::c_int as isize) as *const ::core::ffi::c_void,
                strlen(p.offset(1 as ::core::ffi::c_int as isize)).wrapping_add(1 as size_t),
            );
            p = p.offset(1 as ::core::ffi::c_int as isize);
        }
        return text;
    }
}
