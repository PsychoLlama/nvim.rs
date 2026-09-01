//! The `buffer_`/`window_`/`tabpage_`/`vim_` variable accessors.
//!
//! Eight shims of one shape: the old spellings returned the *previous* value
//! where the modern `nvim_*_set_var`/`nvim_*_del_var` return nothing, so each
//! reads the variable before writing it.

#![deny(unsafe_op_in_unsafe_fn)]

use super::*;
use crate::api::private::helpers::{
    Reported, buffer_by_handle, tabpage_by_handle, window_by_handle,
};

pub unsafe fn buffer_set_var(
    buffer: Buffer,
    name: String_0,
    value: Object,
    arena: *mut Arena,
) -> Result<Object, Error> {
    let mut error = Error::none();
    let Some(buf) = buffer_by_handle(buffer, &mut error) else {
        return Object::Nil.reported(error);
    };
    let vars = buf.b_vars;
    // SAFETY: `vars` is that buffer's own dictionary, `arena` the caller's
    // and `error` this frame's slot.
    unsafe { dict_set_var(vars, name, value, false, true, arena, &mut error) }.reported(error)
}

pub unsafe fn buffer_del_var(
    buffer: Buffer,
    name: String_0,
    arena: *mut Arena,
) -> Result<Object, Error> {
    let mut error = Error::none();
    let Some(buf) = buffer_by_handle(buffer, &mut error) else {
        return Object::Nil.reported(error);
    };
    let vars = buf.b_vars;
    // SAFETY: as `buffer_set_var`.
    unsafe { dict_set_var(vars, name, Object::Nil, true, true, arena, &mut error) }.reported(error)
}

pub unsafe fn window_set_var(
    window: Window,
    name: String_0,
    value: Object,
    arena: *mut Arena,
) -> Result<Object, Error> {
    let mut error = Error::none();
    let Some(win) = window_by_handle(window, &mut error) else {
        return Object::Nil.reported(error);
    };
    let vars = win.w_vars;
    // SAFETY: as `buffer_set_var`, for that window's dictionary.
    unsafe { dict_set_var(vars, name, value, false, true, arena, &mut error) }.reported(error)
}

pub unsafe fn window_del_var(
    window: Window,
    name: String_0,
    arena: *mut Arena,
) -> Result<Object, Error> {
    let mut error = Error::none();
    let Some(win) = window_by_handle(window, &mut error) else {
        return Object::Nil.reported(error);
    };
    let vars = win.w_vars;
    // SAFETY: as `buffer_set_var`.
    unsafe { dict_set_var(vars, name, Object::Nil, true, true, arena, &mut error) }.reported(error)
}

pub unsafe fn tabpage_set_var(
    tabpage: Tabpage,
    name: String_0,
    value: Object,
    arena: *mut Arena,
) -> Result<Object, Error> {
    let mut error = Error::none();
    let Some(tab) = tabpage_by_handle(tabpage, &mut error) else {
        return Object::Nil.reported(error);
    };
    let vars = tab.tp_vars;
    // SAFETY: as `buffer_set_var`, for that tab page's dictionary.
    unsafe { dict_set_var(vars, name, value, false, true, arena, &mut error) }.reported(error)
}

pub unsafe fn tabpage_del_var(
    tabpage: Tabpage,
    name: String_0,
    arena: *mut Arena,
) -> Result<Object, Error> {
    let mut error = Error::none();
    let Some(tab) = tabpage_by_handle(tabpage, &mut error) else {
        return Object::Nil.reported(error);
    };
    let vars = tab.tp_vars;
    // SAFETY: as `buffer_set_var`.
    unsafe { dict_set_var(vars, name, Object::Nil, true, true, arena, &mut error) }.reported(error)
}

pub unsafe fn vim_set_var(
    name: String_0,
    value: Object,
    arena: *mut Arena,
) -> Result<Object, Error> {
    let mut error = Error::none();
    let vars = get_globvar_dict();
    // SAFETY: as `buffer_set_var`, for the global dictionary.
    unsafe { dict_set_var(vars, name, value, false, true, arena, &mut error) }.reported(error)
}

pub unsafe fn vim_del_var(name: String_0, arena: *mut Arena) -> Result<Object, Error> {
    let mut error = Error::none();
    let vars = get_globvar_dict();
    // SAFETY: as `vim_set_var`.
    unsafe { dict_set_var(vars, name, Object::Nil, true, true, arena, &mut error) }.reported(error)
}
