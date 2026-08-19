//! The `buffer_`/`window_`/`tabpage_`/`vim_` variable accessors.
//!
//! Eight shims of one shape: the old spellings returned the *previous* value
//! where the modern `nvim_*_set_var`/`nvim_*_del_var` return nothing, so each
//! reads the variable before writing it.

#![deny(unsafe_op_in_unsafe_fn)]

use super::*;
use crate::api::private::helpers::{ERROR_INIT, NIL, Reported};

pub unsafe fn buffer_set_var(
    buffer: Buffer,
    name: String_0,
    value: Object,
    arena: *mut Arena,
) -> Result<Object, Error> {
    let mut error = ERROR_INIT;
    let err = &raw mut error;
    unsafe {
        let mut buf: *mut buf_T = find_buffer_by_handle(buffer, err);
        if buf.is_null() {
            return NIL.reported(error);
        }
        return dict_set_var((*buf).b_vars, name, value, false, true, arena, err).reported(error);
    }
}

pub unsafe fn buffer_del_var(
    buffer: Buffer,
    name: String_0,
    arena: *mut Arena,
) -> Result<Object, Error> {
    let mut error = ERROR_INIT;
    let err = &raw mut error;
    unsafe {
        let mut buf: *mut buf_T = find_buffer_by_handle(buffer, err);
        if buf.is_null() {
            return NIL.reported(error);
        }
        return dict_set_var((*buf).b_vars, name, NIL, true, true, arena, err).reported(error);
    }
}

pub unsafe fn window_set_var(
    window: Window,
    name: String_0,
    value: Object,
    arena: *mut Arena,
) -> Result<Object, Error> {
    let mut error = ERROR_INIT;
    let err = &raw mut error;
    unsafe {
        let mut win: *mut win_T = find_window_by_handle(window, err);
        if win.is_null() {
            return NIL.reported(error);
        }
        return dict_set_var((*win).w_vars, name, value, false, true, arena, err).reported(error);
    }
}

pub unsafe fn window_del_var(
    window: Window,
    name: String_0,
    arena: *mut Arena,
) -> Result<Object, Error> {
    let mut error = ERROR_INIT;
    let err = &raw mut error;
    unsafe {
        let mut win: *mut win_T = find_window_by_handle(window, err);
        if win.is_null() {
            return NIL.reported(error);
        }
        return dict_set_var((*win).w_vars, name, NIL, true, true, arena, err).reported(error);
    }
}

pub unsafe fn tabpage_set_var(
    tabpage: Tabpage,
    name: String_0,
    value: Object,
    arena: *mut Arena,
) -> Result<Object, Error> {
    let mut error = ERROR_INIT;
    let err = &raw mut error;
    unsafe {
        let mut tab: *mut tabpage_T = find_tab_by_handle(tabpage, err);
        if tab.is_null() {
            return NIL.reported(error);
        }
        return dict_set_var((*tab).tp_vars, name, value, false, true, arena, err).reported(error);
    }
}

pub unsafe fn tabpage_del_var(
    tabpage: Tabpage,
    name: String_0,
    arena: *mut Arena,
) -> Result<Object, Error> {
    let mut error = ERROR_INIT;
    let err = &raw mut error;
    unsafe {
        let mut tab: *mut tabpage_T = find_tab_by_handle(tabpage, err);
        if tab.is_null() {
            return NIL.reported(error);
        }
        return dict_set_var((*tab).tp_vars, name, NIL, true, true, arena, err).reported(error);
    }
}

pub unsafe fn vim_set_var(
    name: String_0,
    value: Object,
    arena: *mut Arena,
) -> Result<Object, Error> {
    let mut error = ERROR_INIT;
    let err = &raw mut error;
    unsafe {
        return dict_set_var(get_globvar_dict(), name, value, false, true, arena, err)
            .reported(error);
    }
}

pub unsafe fn vim_del_var(name: String_0, arena: *mut Arena) -> Result<Object, Error> {
    let mut error = ERROR_INIT;
    let err = &raw mut error;
    unsafe {
        return dict_set_var(get_globvar_dict(), name, NIL, true, true, arena, err).reported(error);
    }
}
