use crate::src::nvim::api::private::helpers::{
    api_clear_error, api_set_error, arena_array, dict_get_value, dict_set_var,
    find_buffer_by_handle, find_tab_by_handle, find_window_by_handle, try_enter, try_leave,
};
use crate::src::nvim::api::vim::nvim_get_current_win;

use crate::src::nvim::main::{
    autocmd_no_enter, autocmd_no_leave, cmdwin_buf, cmdwin_type, curtab, curwin, e_cmdwin, firstwin,
};
use crate::src::nvim::os::libc::abort;
use crate::src::nvim::types::api::{kErrorTypeException, kErrorTypeNone};
use crate::src::nvim::types::{
    Arena, Array, BoolVarValue, Boolean, Buffer, Error, Integer, KeyDict_tabpage_config, Object,
    SpecialVarValue, String_0, Tabpage, TryState, Window, buf_T, except_T, kObjectTypeNil,
    kObjectTypeWindow, msglist_T, object, object_data as C2Rust_Unnamed, size_t, tabpage_T, win_T,
};
use crate::src::nvim::window::{
    tabpage_index, tabpage_win_valid, valid_tabpage, win_goto, win_new_tabpage, win_set_buf,
};
pub const kSpecialVarNull: SpecialVarValue = 0;
pub const kBoolVarTrue: BoolVarValue = 1;
pub const kBoolVarFalse: BoolVarValue = 0;
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const KV_INITIAL_VALUE: Array = Array {
    size: 0 as size_t,
    capacity: 0 as size_t,
    items: ::core::ptr::null_mut::<Object>(),
};
pub const KEYSET_OPTIDX_tabpage_config__after: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const ARRAY_DICT_INIT: Array = KV_INITIAL_VALUE;
pub unsafe extern "C" fn nvim_tabpage_list_wins(
    mut tabpage: Tabpage,
    mut arena: *mut Arena,
    mut err: *mut Error,
) -> Array {
    let mut rv: Array = ARRAY_DICT_INIT;
    let mut tab: *mut tabpage_T = find_tab_by_handle(tabpage, err);
    if tab.is_null() || !valid_tabpage(tab) {
        return rv;
    }
    let mut n: size_t = 0 as size_t;
    let mut wp: *mut win_T = if tab == curtab.get() {
        firstwin.get()
    } else {
        (*tab).tp_firstwin
    };
    while !wp.is_null() {
        n = n.wrapping_add(1);
        wp = (*wp).w_next;
    }
    rv = arena_array(arena, n);
    let mut wp_0: *mut win_T = if tab == curtab.get() {
        firstwin.get()
    } else {
        (*tab).tp_firstwin
    };
    while !wp_0.is_null() {
        let c2rust_fresh0 = rv.size;
        rv.size = rv.size.wrapping_add(1);
        *rv.items.offset(c2rust_fresh0 as isize) = object {
            type_0: kObjectTypeWindow,
            data: C2Rust_Unnamed {
                integer: (*wp_0).handle as Integer,
            },
        };
        wp_0 = (*wp_0).w_next;
    }
    return rv;
}
pub unsafe extern "C" fn nvim_tabpage_get_var(
    mut tabpage: Tabpage,
    mut name: String_0,
    mut arena: *mut Arena,
    mut err: *mut Error,
) -> Object {
    let mut tab: *mut tabpage_T = find_tab_by_handle(tabpage, err);
    if tab.is_null() {
        return object {
            type_0: kObjectTypeNil,
            data: C2Rust_Unnamed { boolean: false },
        };
    }
    return dict_get_value((*tab).tp_vars, name, arena, err);
}
pub unsafe extern "C" fn nvim_tabpage_set_var(
    mut tabpage: Tabpage,
    mut name: String_0,
    mut value: Object,
    mut err: *mut Error,
) {
    let mut tab: *mut tabpage_T = find_tab_by_handle(tabpage, err);
    if tab.is_null() {
        return;
    }
    dict_set_var(
        (*tab).tp_vars,
        name,
        value,
        false_0 != 0,
        false_0 != 0,
        ::core::ptr::null_mut::<Arena>(),
        err,
    );
}
pub unsafe extern "C" fn nvim_tabpage_del_var(
    mut tabpage: Tabpage,
    mut name: String_0,
    mut err: *mut Error,
) {
    let mut tab: *mut tabpage_T = find_tab_by_handle(tabpage, err);
    if tab.is_null() {
        return;
    }
    dict_set_var(
        (*tab).tp_vars,
        name,
        object {
            type_0: kObjectTypeNil,
            data: C2Rust_Unnamed { boolean: false },
        },
        true_0 != 0,
        false_0 != 0,
        ::core::ptr::null_mut::<Arena>(),
        err,
    );
}
pub unsafe extern "C" fn nvim_tabpage_get_win(mut tabpage: Tabpage, mut err: *mut Error) -> Window {
    let mut tab: *mut tabpage_T = find_tab_by_handle(tabpage, err);
    if tab.is_null() || !valid_tabpage(tab) {
        return 0 as Window;
    }
    if tab == curtab.get() {
        return nvim_get_current_win();
    }
    let mut wp: *mut win_T = if tab == curtab.get() {
        firstwin.get()
    } else {
        (*tab).tp_firstwin
    };
    while !wp.is_null() {
        if wp == (*tab).tp_curwin {
            return (*wp).handle as Window;
        }
        wp = (*wp).w_next;
    }
    abort();
}
pub unsafe extern "C" fn nvim_tabpage_set_win(
    mut tabpage: Tabpage,
    mut win: Window,
    mut err: *mut Error,
) {
    let mut tp: *mut tabpage_T = find_tab_by_handle(tabpage, err);
    if tp.is_null() {
        return;
    }
    let mut wp: *mut win_T = find_window_by_handle(win, err);
    if wp.is_null() {
        return;
    }
    if !tabpage_win_valid(tp, wp) {
        api_set_error(
            err,
            kErrorTypeException,
            b"Window does not belong to tabpage %d\0".as_ptr() as *const ::core::ffi::c_char,
            (*tp).handle,
        );
        return;
    }
    if tp == curtab.get() {
        let mut tstate: TryState = TryState {
            current_exception: ::core::ptr::null_mut::<except_T>(),
            private_msg_list: ::core::ptr::null_mut::<msglist_T>(),
            msg_list: ::core::ptr::null::<*const msglist_T>(),
            got_int: 0,
            did_throw: false,
            need_rethrow: 0,
            did_emsg: 0,
        };
        try_enter(&raw mut tstate);
        win_goto(wp);
        try_leave(&raw mut tstate, err);
    } else if (*tp).tp_curwin != wp {
        (*tp).tp_prevwin = (*tp).tp_curwin;
        (*tp).tp_curwin = wp;
    }
}
pub unsafe extern "C" fn nvim_tabpage_get_number(
    mut tabpage: Tabpage,
    mut err: *mut Error,
) -> Integer {
    let mut tab: *mut tabpage_T = find_tab_by_handle(tabpage, err);
    if tab.is_null() {
        return 0 as Integer;
    }
    return tabpage_index(tab) as Integer;
}
pub unsafe extern "C" fn nvim_tabpage_is_valid(mut tabpage: Tabpage) -> Boolean {
    let mut stub: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut ret: Boolean = !find_tab_by_handle(tabpage, &raw mut stub).is_null();
    api_clear_error(&raw mut stub);
    return ret;
}
pub unsafe extern "C" fn nvim_open_tabpage(
    mut buf: Buffer,
    mut enter: Boolean,
    mut config: *mut KeyDict_tabpage_config,
    mut err: *mut Error,
) -> Tabpage {
    let mut b: *mut buf_T = find_buffer_by_handle(buf, err);
    if b.is_null() {
        return 0 as Tabpage;
    }
    if cmdwin_type.get() != 0 as ::core::ffi::c_int && enter as ::core::ffi::c_int != 0
        || b == cmdwin_buf.get()
    {
        api_set_error(
            err,
            kErrorTypeException,
            b"%s\0".as_ptr() as *const ::core::ffi::c_char,
            &raw const e_cmdwin as *const ::core::ffi::c_char,
        );
        return 0 as Tabpage;
    }
    let mut after: ::core::ffi::c_int = -1 as ::core::ffi::c_int;
    if (*config).is_set__tabpage_config_ as ::core::ffi::c_ulonglong
        & (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_tabpage_config__after
        != 0 as ::core::ffi::c_ulonglong
    {
        after = (*config).after as ::core::ffi::c_int;
    }
    let mut tp: *mut tabpage_T = ::core::ptr::null_mut::<tabpage_T>();
    let mut wp: *mut win_T = ::core::ptr::null_mut::<win_T>();
    let mut tstate: TryState = TryState {
        current_exception: ::core::ptr::null_mut::<except_T>(),
        private_msg_list: ::core::ptr::null_mut::<msglist_T>(),
        msg_list: ::core::ptr::null::<*const msglist_T>(),
        got_int: 0,
        did_throw: false,
        need_rethrow: 0,
        did_emsg: 0,
    };
    try_enter(&raw mut tstate);
    tp = win_new_tabpage(
        after + 1 as ::core::ffi::c_int,
        ::core::ptr::null_mut::<::core::ffi::c_char>(),
        enter as bool,
        &raw mut wp,
    );
    try_leave(&raw mut tstate, err);
    if tp.is_null() {
        if !((*err).type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int) {
            api_set_error(
                err,
                kErrorTypeException,
                b"Failed to create new tabpage\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        return 0 as Tabpage;
    }
    if !valid_tabpage(tp) {
        api_clear_error(err);
        api_set_error(
            err,
            kErrorTypeException,
            b"Tabpage was closed immediately\0".as_ptr() as *const ::core::ffi::c_char,
        );
        return 0 as Tabpage;
    }
    if tabpage_win_valid(tp, wp) as ::core::ffi::c_int != 0 && (*wp).w_buffer != b {
        let au_no_enter_leave: bool = curwin.get() != wp;
        if au_no_enter_leave {
            (*autocmd_no_enter.ptr()) += 1;
            (*autocmd_no_leave.ptr()) += 1;
        }
        win_set_buf(wp, b, err);
        if au_no_enter_leave {
            (*autocmd_no_enter.ptr()) -= 1;
            (*autocmd_no_leave.ptr()) -= 1;
        }
        if !valid_tabpage(tp) {
            api_clear_error(err);
            api_set_error(
                err,
                kErrorTypeException,
                b"Tabpage was closed immediately\0".as_ptr() as *const ::core::ffi::c_char,
            );
            return 0 as Tabpage;
        }
    }
    return (*tp).handle as Tabpage;
}
pub const true_0: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const false_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
