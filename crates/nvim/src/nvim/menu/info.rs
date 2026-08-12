//! Translating menu names, and describing one -- `:menutranslate` and
//! `menu_info()`.
//!
//! `menutrans_ga` is the translation table `:menutranslate` fills; a
//! translation is applied when a menu is *defined*, so [`menutrans_lookup`]
//! runs from `add_menu_path` and the English name is kept alongside the
//! translated one.  [`menu_translate_tab_and_shift`] rewrites `<Tab>` and
//! `<S-...>` in a name before it is parsed at all.
//! [`menuitem_getinfo`] and [`f_menu_info`] build the Dict `menu_info()`
//! answers with -- the rhs, the mode, the priority, the accelerator and the
//! `<silent>`/`<script>` flags.
//!
//! Original: `src/nvim/menu.c`, Vim/Neovim, Vim license.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;
use crate::src::nvim::ascii::ascii_iswhite;
use crate::src::nvim::charset::skipwhite;
use crate::src::nvim::eval::typval::{
    tv_dict_add_allocated_str, tv_dict_add_bool, tv_dict_add_list, tv_dict_add_nr, tv_dict_add_str,
    tv_dict_alloc_ret, tv_get_string_chk, tv_list_alloc, tv_list_append_string,
};
use crate::src::nvim::eval::vars::del_menutrans_vars;
use crate::src::nvim::ex_docmd::ends_excmd;
use crate::src::nvim::garray::{ga_append_via_ptr, ga_clear, ga_init};
use crate::src::nvim::global_cell::GlobalCell;
use crate::src::nvim::keycodes::Ctrl_V;
use crate::src::nvim::main::e_invarg;
use crate::src::nvim::mbyte::{utf_char2bytes, utfc_ptr2len};
use crate::src::nvim::memory::{xfree, xmemdupz, xstrdup};
use crate::src::nvim::message::{emsg, str2special_save};
use crate::src::nvim::os::libc::{gettext, memmove, strcasecmp, strlen, strncasecmp, strncmp};
use crate::src::nvim::types::{
    BoolVarValue, EvalFuncData, VAR_UNKNOWN, dict_T, exarg_T, garray_T, kListLenMayKnow, list_T,
    ptrdiff_t, size_t, ssize_t, typval_T, varnumber_T, vimmenu_T,
};

static menutrans_ga: GlobalCell<garray_T> = GlobalCell::new(GA_EMPTY_INIT_VALUE);

pub unsafe fn ex_menutranslate(mut eap: *mut exarg_T) {
    unsafe {
        let mut arg: *mut ::core::ffi::c_char = (*eap).arg;
        if (*menutrans_ga.ptr()).ga_itemsize == 0 as ::core::ffi::c_int {
            ga_init(
                menutrans_ga.ptr(),
                ::core::mem::size_of::<menutrans_T>() as ::core::ffi::c_int,
                5 as ::core::ffi::c_int,
            );
        }
        if strncmp(arg, c"clear".as_ptr(), 5 as size_t) == 0 as ::core::ffi::c_int
            && ends_excmd(
                *skipwhite(arg.offset(5 as ::core::ffi::c_int as isize)) as ::core::ffi::c_int
            ) != 0
        {
            let mut _gap: *mut garray_T = menutrans_ga.ptr();
            if !(*_gap).ga_data.is_null() {
                let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                while i < (*_gap).ga_len {
                    let mut _item: *mut menutrans_T =
                        ((*_gap).ga_data as *mut menutrans_T).offset(i as isize);
                    let mut _mt: *mut menutrans_T = _item;
                    xfree((*_mt).from as *mut ::core::ffi::c_void);
                    xfree((*_mt).from_noamp as *mut ::core::ffi::c_void);
                    xfree((*_mt).to as *mut ::core::ffi::c_void);
                    i += 1;
                }
            }
            ga_clear(_gap);
            del_menutrans_vars();
        } else {
            let mut from: *mut ::core::ffi::c_char = arg;
            arg = menu_skip_part(arg);
            let mut to: *mut ::core::ffi::c_char = skipwhite(arg);
            *arg = NUL as ::core::ffi::c_char;
            arg = menu_skip_part(to);
            if arg == to {
                emsg(gettext(&raw const e_invarg as *const ::core::ffi::c_char));
            } else {
                from = xstrdup(from);
                let mut from_noamp: *mut ::core::ffi::c_char = menu_text(
                    from,
                    ::core::ptr::null_mut::<::core::ffi::c_int>(),
                    ::core::ptr::null_mut::<*mut ::core::ffi::c_char>(),
                );
                debug_assert!(arg >= to, "arg >= to");
                to = xmemdupz(
                    to as *const ::core::ffi::c_void,
                    arg.offset_from(to) as size_t,
                ) as *mut ::core::ffi::c_char;
                menu_translate_tab_and_shift(from);
                menu_translate_tab_and_shift(to);
                menu_unescape_name(from);
                menu_unescape_name(to);
                let mut tp: *mut menutrans_T =
                    ga_append_via_ptr(menutrans_ga.ptr(), ::core::mem::size_of::<menutrans_T>())
                        as *mut menutrans_T;
                (*tp).from = from;
                (*tp).from_noamp = from_noamp;
                (*tp).to = to;
            }
        };
    }
}

unsafe extern "C" fn menu_skip_part(mut p: *mut ::core::ffi::c_char) -> *mut ::core::ffi::c_char {
    unsafe {
        while *p as ::core::ffi::c_int != NUL
            && *p as ::core::ffi::c_int != '.' as ::core::ffi::c_int
            && !ascii_iswhite(*p as ::core::ffi::c_int)
        {
            if (*p as ::core::ffi::c_int == '\\' as ::core::ffi::c_int
                || *p as ::core::ffi::c_int == Ctrl_V)
                && *p.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int != NUL
            {
                p = p.offset(1);
            }
            p = p.offset(1);
        }
        return p;
    }
}

pub(crate) unsafe extern "C" fn menutrans_lookup(
    mut name: *mut ::core::ffi::c_char,
    mut len: ::core::ffi::c_int,
) -> *mut ::core::ffi::c_char {
    unsafe {
        let mut tp: *mut menutrans_T = (*menutrans_ga.ptr()).ga_data as *mut menutrans_T;
        let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while i < (*menutrans_ga.ptr()).ga_len {
            if strncasecmp(name, (*tp.offset(i as isize)).from, len as size_t)
                == 0 as ::core::ffi::c_int
                && *(*tp.offset(i as isize)).from.offset(len as isize) as ::core::ffi::c_int == NUL
            {
                return (*tp.offset(i as isize)).to;
            }
            i += 1;
        }
        let mut c: ::core::ffi::c_char = *name.offset(len as isize);
        *name.offset(len as isize) = NUL as ::core::ffi::c_char;
        let mut dname: *mut ::core::ffi::c_char = menu_text(
            name,
            ::core::ptr::null_mut::<::core::ffi::c_int>(),
            ::core::ptr::null_mut::<*mut ::core::ffi::c_char>(),
        );
        *name.offset(len as isize) = c;
        let mut i_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while i_0 < (*menutrans_ga.ptr()).ga_len {
            if strcasecmp(dname, (*tp.offset(i_0 as isize)).from_noamp) == 0 as ::core::ffi::c_int {
                xfree(dname as *mut ::core::ffi::c_void);
                return (*tp.offset(i_0 as isize)).to;
            }
            i_0 += 1;
        }
        xfree(dname as *mut ::core::ffi::c_void);
        return ::core::ptr::null_mut::<::core::ffi::c_char>();
    }
}

unsafe extern "C" fn menu_unescape_name(mut name: *mut ::core::ffi::c_char) {
    unsafe {
        let mut p: *mut ::core::ffi::c_char = name;
        while *p as ::core::ffi::c_int != 0 && *p as ::core::ffi::c_int != '.' as ::core::ffi::c_int
        {
            if *p as ::core::ffi::c_int == '\\' as ::core::ffi::c_int {
                memmove(
                    p as *mut ::core::ffi::c_void,
                    p.offset(1 as ::core::ffi::c_int as isize) as *const ::core::ffi::c_void,
                    strlen(p.offset(1 as ::core::ffi::c_int as isize)).wrapping_add(1 as size_t),
                );
            }
            p = p.offset(utfc_ptr2len(p) as isize);
        }
    }
}

pub(crate) unsafe extern "C" fn menu_translate_tab_and_shift(
    mut arg_start: *mut ::core::ffi::c_char,
) -> *mut ::core::ffi::c_char {
    unsafe {
        let mut arg: *mut ::core::ffi::c_char = arg_start;
        while *arg as ::core::ffi::c_int != 0 && !ascii_iswhite(*arg as ::core::ffi::c_int) {
            if (*arg as ::core::ffi::c_int == '\\' as ::core::ffi::c_int
                || *arg as ::core::ffi::c_int == Ctrl_V)
                && *arg.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int != NUL
            {
                arg = arg.offset(1);
            } else if strncasecmp(
                arg,
                c"<TAB>".as_ptr() as *mut ::core::ffi::c_char,
                5 as ::core::ffi::c_int as size_t,
            ) == 0 as ::core::ffi::c_int
            {
                *arg = TAB as ::core::ffi::c_char;
                memmove(
                    arg.offset(1 as ::core::ffi::c_int as isize) as *mut ::core::ffi::c_void,
                    arg.offset(5 as ::core::ffi::c_int as isize) as *const ::core::ffi::c_void,
                    strlen(arg.offset(5 as ::core::ffi::c_int as isize)).wrapping_add(1 as size_t),
                );
            }
            arg = arg.offset(1);
        }
        if *arg as ::core::ffi::c_int != NUL {
            let c2rust_fresh4 = arg;
            arg = arg.offset(1);
            *c2rust_fresh4 = NUL as ::core::ffi::c_char;
        }
        arg = skipwhite(arg);
        return arg;
    }
}

unsafe extern "C" fn menuitem_getinfo(
    mut menu_name: *const ::core::ffi::c_char,
    mut menu: *const vimmenu_T,
    mut modes: ::core::ffi::c_int,
    mut dict: *mut dict_T,
) {
    unsafe {
        if *menu_name as ::core::ffi::c_int == NUL {
            let l: *mut list_T = tv_list_alloc(kListLenMayKnow as ::core::ffi::c_int as ptrdiff_t);
            tv_dict_add_list(
                dict,
                c"submenus".as_ptr(),
                ::core::mem::size_of::<[::core::ffi::c_char; 9]>().wrapping_sub(1 as size_t),
                l,
            );
            let mut topmenu: *const vimmenu_T = menu;
            while !topmenu.is_null() {
                if !menu_is_hidden((*topmenu).dname) {
                    tv_list_append_string(l, (*topmenu).dname, -1 as ssize_t);
                }
                topmenu = (*topmenu).next;
            }
            return;
        }
        tv_dict_add_str(
            dict,
            c"name".as_ptr(),
            ::core::mem::size_of::<[::core::ffi::c_char; 5]>().wrapping_sub(1 as size_t),
            (*menu).name,
        );
        tv_dict_add_str(
            dict,
            c"display".as_ptr(),
            ::core::mem::size_of::<[::core::ffi::c_char; 8]>().wrapping_sub(1 as size_t),
            (*menu).dname,
        );
        if !(*menu).actext.is_null() {
            tv_dict_add_str(
                dict,
                c"accel".as_ptr(),
                ::core::mem::size_of::<[::core::ffi::c_char; 6]>().wrapping_sub(1 as size_t),
                (*menu).actext,
            );
        }
        tv_dict_add_nr(
            dict,
            c"priority".as_ptr(),
            ::core::mem::size_of::<[::core::ffi::c_char; 9]>().wrapping_sub(1 as size_t),
            (*menu).priority as varnumber_T,
        );
        tv_dict_add_str(
            dict,
            c"modes".as_ptr(),
            ::core::mem::size_of::<[::core::ffi::c_char; 6]>().wrapping_sub(1 as size_t),
            get_menu_mode_str((*menu).modes),
        );
        let mut buf: [::core::ffi::c_char; 65] = [0; 65];
        buf[utf_char2bytes((*menu).mnemonic, &raw mut buf as *mut ::core::ffi::c_char) as usize] =
            NUL as ::core::ffi::c_char;
        tv_dict_add_str(
            dict,
            c"shortcut".as_ptr(),
            ::core::mem::size_of::<[::core::ffi::c_char; 9]>().wrapping_sub(1 as size_t),
            &raw mut buf as *mut ::core::ffi::c_char,
        );
        if (*menu).children.is_null() {
            let mut bit: ::core::ffi::c_int = 0;
            bit = 0 as ::core::ffi::c_int;
            while bit < MENU_MODES as ::core::ffi::c_int
                && (1 as ::core::ffi::c_int) << bit & modes == 0
            {
                bit += 1;
            }
            if bit < MENU_MODES as ::core::ffi::c_int {
                if !(*menu).strings[bit as usize].is_null() {
                    tv_dict_add_allocated_str(
                        dict,
                        c"rhs".as_ptr(),
                        ::core::mem::size_of::<[::core::ffi::c_char; 4]>()
                            .wrapping_sub(1 as size_t),
                        if *(*menu).strings[bit as usize] as ::core::ffi::c_int == NUL {
                            xstrdup(c"<Nop>".as_ptr())
                        } else {
                            str2special_save(
                                (*menu).strings[bit as usize],
                                false_0 != 0,
                                false_0 != 0,
                            )
                        },
                    );
                }
                tv_dict_add_bool(
                    dict,
                    c"noremenu".as_ptr(),
                    ::core::mem::size_of::<[::core::ffi::c_char; 9]>().wrapping_sub(1 as size_t),
                    ((*menu).noremap[bit as usize] == REMAP_NONE as ::core::ffi::c_int)
                        as ::core::ffi::c_int as BoolVarValue,
                );
                tv_dict_add_bool(
                    dict,
                    c"script".as_ptr(),
                    ::core::mem::size_of::<[::core::ffi::c_char; 7]>().wrapping_sub(1 as size_t),
                    ((*menu).noremap[bit as usize] == REMAP_SCRIPT as ::core::ffi::c_int)
                        as ::core::ffi::c_int as BoolVarValue,
                );
                tv_dict_add_bool(
                    dict,
                    c"silent".as_ptr(),
                    ::core::mem::size_of::<[::core::ffi::c_char; 7]>().wrapping_sub(1 as size_t),
                    (*menu).silent[bit as usize] as BoolVarValue,
                );
                tv_dict_add_bool(
                    dict,
                    c"enabled".as_ptr(),
                    ::core::mem::size_of::<[::core::ffi::c_char; 8]>().wrapping_sub(1 as size_t),
                    ((*menu).enabled & (1 as ::core::ffi::c_int) << bit != 0 as ::core::ffi::c_int)
                        as ::core::ffi::c_int as BoolVarValue,
                );
            }
        } else {
            let l_0: *mut list_T =
                tv_list_alloc(kListLenMayKnow as ::core::ffi::c_int as ptrdiff_t);
            tv_dict_add_list(
                dict,
                c"submenus".as_ptr(),
                ::core::mem::size_of::<[::core::ffi::c_char; 9]>().wrapping_sub(1 as size_t),
                l_0,
            );
            let mut child: *const vimmenu_T = (*menu).children;
            while !child.is_null() {
                tv_list_append_string(l_0, (*child).dname, -1 as ssize_t);
                child = (*child).next;
            }
        };
    }
}

pub unsafe extern "C" fn f_menu_info(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    unsafe {
        tv_dict_alloc_ret(rettv);
        let retdict: *mut dict_T = (*rettv).vval.v_dict;
        let menu_name: *const ::core::ffi::c_char =
            tv_get_string_chk(argvars.offset(0 as ::core::ffi::c_int as isize));
        if menu_name.is_null() {
            return;
        }
        let mut which: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
        if (*argvars.offset(1 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
            != VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            which = tv_get_string_chk(argvars.offset(1 as ::core::ffi::c_int as isize));
        } else {
            which = c"".as_ptr();
        }
        if which.is_null() {
            return;
        }
        let modes: ::core::ffi::c_int = get_menu_cmd_modes(
            which,
            *which as ::core::ffi::c_int == '!' as ::core::ffi::c_int,
            ::core::ptr::null_mut::<::core::ffi::c_int>(),
            ::core::ptr::null_mut::<bool>(),
        );
        let mut menu: *const vimmenu_T = *get_root_menu(menu_name);
        let saved_name: *mut ::core::ffi::c_char = xstrdup(menu_name);
        if *saved_name as ::core::ffi::c_int != NUL {
            let mut name: *mut ::core::ffi::c_char = saved_name;
            while *name != 0 {
                let mut p: *mut ::core::ffi::c_char = menu_name_skip(name);
                while !menu.is_null() {
                    if menu_name_equal(name, menu) {
                        break;
                    }
                    menu = (*menu).next;
                }
                if menu.is_null() || *p as ::core::ffi::c_int == NUL {
                    break;
                }
                menu = (*menu).children;
                name = p;
            }
        }
        xfree(saved_name as *mut ::core::ffi::c_void);
        if menu.is_null() {
            return;
        }
        if (*menu).modes & modes != 0 {
            menuitem_getinfo(menu_name, menu, modes, retdict);
        }
    }
}
