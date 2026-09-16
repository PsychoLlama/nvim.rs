//! The `buffer_`/`window_`/`tabpage_`/`vim_` variable accessors.
//!
//! Eight shims of one shape: the old spellings returned the *previous* value
//! where the modern `nvim_*_set_var`/`nvim_*_del_var` return nothing, so each
//! reads the variable before writing it.

#![deny(unsafe_op_in_unsafe_fn)]
#![allow(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use super::*;
use crate::api::private::helpers::{
    find_buffer_by_handle, find_tab_by_handle, find_window_by_handle,
};

pub fn buffer_set_var(
    buffer: BufferHandle,
    name: String_0,
    value: Object,
) -> Result<Object, Error> {
    let Some(buf) = find_buffer_by_handle(buffer)? else {
        return Ok(Object::Nil);
    };
    let vars = buf.b_vars;
    // SAFETY: `vars` is that buffer's own dictionary, `arena` the caller's
    // and `error` this frame's slot.
    unsafe { dict_set_var(vars, &name, value, false, true) }
}

pub fn buffer_del_var(buffer: BufferHandle, name: String_0) -> Result<Object, Error> {
    let Some(buf) = find_buffer_by_handle(buffer)? else {
        return Ok(Object::Nil);
    };
    let vars = buf.b_vars;
    // SAFETY: as `buffer_set_var`.
    unsafe { dict_set_var(vars, &name, Object::Nil, true, true) }
}

pub fn window_set_var(
    window: WindowHandle,
    name: String_0,
    value: Object,
) -> Result<Object, Error> {
    let Some(win) = find_window_by_handle(window)? else {
        return Ok(Object::Nil);
    };
    let vars = win.w_vars;
    // SAFETY: as `buffer_set_var`, for that window's dictionary.
    unsafe { dict_set_var(vars, &name, value, false, true) }
}

pub fn window_del_var(window: WindowHandle, name: String_0) -> Result<Object, Error> {
    let Some(win) = find_window_by_handle(window)? else {
        return Ok(Object::Nil);
    };
    let vars = win.w_vars;
    // SAFETY: as `buffer_set_var`.
    unsafe { dict_set_var(vars, &name, Object::Nil, true, true) }
}

pub fn tabpage_set_var(
    tabpage: TabpageHandle,
    name: String_0,
    value: Object,
) -> Result<Object, Error> {
    let Some(tab) = find_tab_by_handle(tabpage)? else {
        return Ok(Object::Nil);
    };
    let vars = tab.tp_vars;
    // SAFETY: as `buffer_set_var`, for that tab page's dictionary.
    unsafe { dict_set_var(vars, &name, value, false, true) }
}

pub fn tabpage_del_var(tabpage: TabpageHandle, name: String_0) -> Result<Object, Error> {
    let Some(tab) = find_tab_by_handle(tabpage)? else {
        return Ok(Object::Nil);
    };
    let vars = tab.tp_vars;
    // SAFETY: as `buffer_set_var`.
    unsafe { dict_set_var(vars, &name, Object::Nil, true, true) }
}

pub fn vim_set_var(name: String_0, value: Object) -> Result<Object, Error> {
    let vars = get_globvar_dict();
    // SAFETY: as `buffer_set_var`, for the global dictionary.
    unsafe { dict_set_var(vars, &name, value, false, true) }
}

pub fn vim_del_var(name: String_0) -> Result<Object, Error> {
    let vars = get_globvar_dict();
    // SAFETY: as `vim_set_var`.
    unsafe { dict_set_var(vars, &name, Object::Nil, true, true) }
}
