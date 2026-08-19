use crate::api::private::helpers::{
    ERROR_INIT, NIL, Reported, api_clear_error, api_set_error, arena_array, array_add,
    dict_get_value, dict_set_var, find_buffer_by_handle, find_tab_by_handle, find_window_by_handle,
    has_key, try_enter, try_leave,
};
use crate::api::vim::nvim_get_current_win;

use crate::main::{
    autocmd_no_enter, autocmd_no_leave, cmdwin_buf, cmdwin_type, curtab, curwin, e_cmdwin, firstwin,
};
use crate::types::{
    Arena, Array, Boolean, Buffer, Error, Integer, KeyDict_tabpage_config, Object, String_0,
    Tabpage, TryState, Window, buf_T, except_T, kErrorTypeException, kErrorTypeNone,
    kObjectTypeWindow, msglist_T, object, object_data as C2Rust_Unnamed, size_t, tabpage_T, win_T,
};
use crate::window::{
    tabpage_index, tabpage_win_valid, valid_tabpage, win_goto, win_new_tabpage, win_set_buf,
};
use ::libc::abort;
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const KV_INITIAL_VALUE: Array = Array {
    size: 0 as size_t,
    capacity: 0 as size_t,
    items: ::core::ptr::null_mut::<Object>(),
};
pub const KEYSET_OPTIDX_tabpage_config__after: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const ARRAY_DICT_INIT: Array = KV_INITIAL_VALUE;
pub unsafe fn nvim_tabpage_list_wins(
    mut tabpage: Tabpage,
    mut arena: *mut Arena,
) -> Result<Array, Error> {
    let mut err = ERROR_INIT;
    let mut rv: Array = ARRAY_DICT_INIT;
    let mut tab: *mut tabpage_T = find_tab_by_handle(tabpage, &raw mut err);
    if tab.is_null() || !valid_tabpage(tab) {
        return rv.reported(err);
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
        array_add(
            &mut rv,
            object {
                type_0: kObjectTypeWindow,
                data: C2Rust_Unnamed {
                    integer: (*wp_0).handle as Integer,
                },
            },
        );
        wp_0 = (*wp_0).w_next;
    }
    Ok(rv)
}
pub unsafe fn nvim_tabpage_get_var(
    tabpage: Tabpage,
    name: String_0,
    arena: *mut Arena,
) -> Result<Object, Error> {
    let mut err = ERROR_INIT;
    // SAFETY: `err` is this frame's own, and the lookup dereferences nothing
    // else.
    let tab: *mut tabpage_T = unsafe { find_tab_by_handle(tabpage, &raw mut err) };
    if tab.is_null() {
        return NIL.reported(err);
    }
    // SAFETY: `tab` is a live tabpage, so `tp_vars` is its own dictionary;
    // `name` and `arena` are the caller's, per this function's contract.
    let value = unsafe { dict_get_value((*tab).tp_vars, name, arena, &raw mut err) };
    value.reported(err)
}
pub unsafe fn nvim_tabpage_set_var(
    tabpage: Tabpage,
    name: String_0,
    value: Object,
) -> Result<(), Error> {
    let mut err = ERROR_INIT;
    // SAFETY: as `nvim_tabpage_get_var`.
    let tab: *mut tabpage_T = unsafe { find_tab_by_handle(tabpage, &raw mut err) };
    if tab.is_null() {
        return ().reported(err);
    }
    // SAFETY: as above; `value` is the caller's and the store takes it over.
    unsafe {
        dict_set_var(
            (*tab).tp_vars,
            name,
            value,
            false,
            false,
            ::core::ptr::null_mut::<Arena>(),
            &raw mut err,
        )
    };
    ().reported(err)
}
pub unsafe fn nvim_tabpage_del_var(mut tabpage: Tabpage, mut name: String_0) -> Result<(), Error> {
    let mut err = ERROR_INIT;
    let mut tab: *mut tabpage_T = find_tab_by_handle(tabpage, &raw mut err);
    if tab.is_null() {
        return ().reported(err);
    }
    dict_set_var(
        (*tab).tp_vars,
        name,
        NIL,
        true,
        false,
        ::core::ptr::null_mut::<Arena>(),
        &raw mut err,
    );
    ().reported(err)
}
pub fn nvim_tabpage_get_win(tabpage: Tabpage) -> Result<Window, Error> {
    let mut err = ERROR_INIT;
    // SAFETY: `err` is this frame's own, and the lookup dereferences nothing
    // else.
    let tab: *mut tabpage_T = unsafe { find_tab_by_handle(tabpage, &raw mut err) };
    if tab.is_null() || !valid_tabpage(tab) {
        return (0 as Window).reported(err);
    }
    if tab == curtab.get() {
        // SAFETY: the current window is whatever `curwin` names.
        return Ok(unsafe { nvim_get_current_win() });
    }
    // SAFETY: `tab` is a live tabpage, so its window list is its own and every
    // link in it is live for as long as the tabpage is.
    unsafe {
        let mut wp: *mut win_T = (*tab).tp_firstwin;
        while !wp.is_null() {
            if wp == (*tab).tp_curwin {
                return Ok((*wp).handle as Window);
            }
            wp = (*wp).w_next;
        }
        abort();
    }
}
pub unsafe fn nvim_tabpage_set_win(tabpage: Tabpage, win: Window) -> Result<(), Error> {
    let mut err = ERROR_INIT;
    let mut tp: *mut tabpage_T = find_tab_by_handle(tabpage, &raw mut err);
    if tp.is_null() {
        return ().reported(err);
    }
    let mut wp: *mut win_T = find_window_by_handle(win, &raw mut err);
    if wp.is_null() {
        return ().reported(err);
    }
    if !tabpage_win_valid(tp, wp) {
        api_set_error(
            &raw mut err,
            kErrorTypeException,
            c"Window does not belong to tabpage %d".as_ptr(),
            (*tp).handle,
        );
        return Err(err);
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
        try_leave(&raw mut tstate, &raw mut err);
    } else if (*tp).tp_curwin != wp {
        (*tp).tp_prevwin = (*tp).tp_curwin;
        (*tp).tp_curwin = wp;
    }
    ().reported(err)
}
pub fn nvim_tabpage_get_number(tabpage: Tabpage) -> Result<Integer, Error> {
    let mut err = ERROR_INIT;
    // SAFETY: `err` is this frame's own, and the lookup dereferences nothing
    // else.
    let tab: *mut tabpage_T = unsafe { find_tab_by_handle(tabpage, &raw mut err) };
    if tab.is_null() {
        return (0 as Integer).reported(err);
    }
    Ok(tabpage_index(tab) as Integer)
}
pub fn nvim_tabpage_is_valid(tabpage: Tabpage) -> Boolean {
    let mut stub: Error = ERROR_INIT;
    // SAFETY: `stub` is this frame's own; the lookup dereferences nothing
    // else, and the message it may leave behind is freed right after.
    unsafe {
        let ret: Boolean = !find_tab_by_handle(tabpage, &raw mut stub).is_null();
        api_clear_error(&raw mut stub);
        ret
    }
}
pub unsafe fn nvim_open_tabpage(
    buf: Buffer,
    enter: Boolean,
    config: *mut KeyDict_tabpage_config,
) -> Result<Tabpage, Error> {
    let mut err = ERROR_INIT;
    let mut b: *mut buf_T = find_buffer_by_handle(buf, &raw mut err);
    if b.is_null() {
        return (0 as Tabpage).reported(err);
    }
    if cmdwin_type.get() != 0 as ::core::ffi::c_int && enter as ::core::ffi::c_int != 0
        || b == cmdwin_buf.get()
    {
        api_set_error(
            &raw mut err,
            kErrorTypeException,
            c"%s".as_ptr(),
            &raw const e_cmdwin as *const ::core::ffi::c_char,
        );
        return Err(err);
    }
    let mut after: ::core::ffi::c_int = -1 as ::core::ffi::c_int;
    if has_key(
        (*config).is_set__tabpage_config_,
        KEYSET_OPTIDX_tabpage_config__after,
    ) {
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
        enter,
        &raw mut wp,
    );
    try_leave(&raw mut tstate, &raw mut err);
    if tp.is_null() {
        if err.type_0 == kErrorTypeNone {
            api_set_error(
                &raw mut err,
                kErrorTypeException,
                c"Failed to create new tabpage".as_ptr(),
            );
        }
        return Err(err);
    }
    if !valid_tabpage(tp) {
        api_clear_error(&raw mut err);
        api_set_error(
            &raw mut err,
            kErrorTypeException,
            c"Tabpage was closed immediately".as_ptr(),
        );
        return Err(err);
    }
    if tabpage_win_valid(tp, wp) as ::core::ffi::c_int != 0 && (*wp).w_buffer != b {
        let au_no_enter_leave: bool = curwin.get() != wp;
        if au_no_enter_leave {
            (*autocmd_no_enter.ptr()) += 1;
            (*autocmd_no_leave.ptr()) += 1;
        }
        win_set_buf(wp, b, &raw mut err);
        if au_no_enter_leave {
            (*autocmd_no_enter.ptr()) -= 1;
            (*autocmd_no_leave.ptr()) -= 1;
        }
        if !valid_tabpage(tp) {
            api_clear_error(&raw mut err);
            api_set_error(
                &raw mut err,
                kErrorTypeException,
                c"Tabpage was closed immediately".as_ptr(),
            );
            return Err(err);
        }
    }
    ((*tp).handle as Tabpage).reported(err)
}
