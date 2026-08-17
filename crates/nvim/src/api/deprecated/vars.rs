//! The `buffer_`/`window_`/`tabpage_`/`vim_` variable accessors.
//!
//! Eight shims of one shape: the old spellings returned the *previous* value
//! where the modern `nvim_*_set_var`/`nvim_*_del_var` return nothing, so each
//! reads the variable before writing it.

#![deny(unsafe_op_in_unsafe_fn)]

use super::*;

pub unsafe extern "C" fn buffer_set_var(
    mut buffer: Buffer,
    mut name: String_0,
    mut value: Object,
    mut arena: *mut Arena,
    mut err: *mut Error,
) -> Object {
    unsafe {
        let mut buf: *mut buf_T = find_buffer_by_handle(buffer, err);
        if buf.is_null() {
            return object {
                type_0: kObjectTypeNil,
                data: C2Rust_Unnamed { boolean: false },
            };
        }
        return dict_set_var((*buf).b_vars, name, value, false, true, arena, err);
    }
}

pub unsafe extern "C" fn buffer_del_var(
    mut buffer: Buffer,
    mut name: String_0,
    mut arena: *mut Arena,
    mut err: *mut Error,
) -> Object {
    unsafe {
        let mut buf: *mut buf_T = find_buffer_by_handle(buffer, err);
        if buf.is_null() {
            return object {
                type_0: kObjectTypeNil,
                data: C2Rust_Unnamed { boolean: false },
            };
        }
        return dict_set_var(
            (*buf).b_vars,
            name,
            object {
                type_0: kObjectTypeNil,
                data: C2Rust_Unnamed { boolean: false },
            },
            true,
            true,
            arena,
            err,
        );
    }
}

pub unsafe extern "C" fn window_set_var(
    mut window: Window,
    mut name: String_0,
    mut value: Object,
    mut arena: *mut Arena,
    mut err: *mut Error,
) -> Object {
    unsafe {
        let mut win: *mut win_T = find_window_by_handle(window, err);
        if win.is_null() {
            return object {
                type_0: kObjectTypeNil,
                data: C2Rust_Unnamed { boolean: false },
            };
        }
        return dict_set_var((*win).w_vars, name, value, false, true, arena, err);
    }
}

pub unsafe extern "C" fn window_del_var(
    mut window: Window,
    mut name: String_0,
    mut arena: *mut Arena,
    mut err: *mut Error,
) -> Object {
    unsafe {
        let mut win: *mut win_T = find_window_by_handle(window, err);
        if win.is_null() {
            return object {
                type_0: kObjectTypeNil,
                data: C2Rust_Unnamed { boolean: false },
            };
        }
        return dict_set_var(
            (*win).w_vars,
            name,
            object {
                type_0: kObjectTypeNil,
                data: C2Rust_Unnamed { boolean: false },
            },
            true,
            true,
            arena,
            err,
        );
    }
}

pub unsafe extern "C" fn tabpage_set_var(
    mut tabpage: Tabpage,
    mut name: String_0,
    mut value: Object,
    mut arena: *mut Arena,
    mut err: *mut Error,
) -> Object {
    unsafe {
        let mut tab: *mut tabpage_T = find_tab_by_handle(tabpage, err);
        if tab.is_null() {
            return object {
                type_0: kObjectTypeNil,
                data: C2Rust_Unnamed { boolean: false },
            };
        }
        return dict_set_var((*tab).tp_vars, name, value, false, true, arena, err);
    }
}

pub unsafe extern "C" fn tabpage_del_var(
    mut tabpage: Tabpage,
    mut name: String_0,
    mut arena: *mut Arena,
    mut err: *mut Error,
) -> Object {
    unsafe {
        let mut tab: *mut tabpage_T = find_tab_by_handle(tabpage, err);
        if tab.is_null() {
            return object {
                type_0: kObjectTypeNil,
                data: C2Rust_Unnamed { boolean: false },
            };
        }
        return dict_set_var(
            (*tab).tp_vars,
            name,
            object {
                type_0: kObjectTypeNil,
                data: C2Rust_Unnamed { boolean: false },
            },
            true,
            true,
            arena,
            err,
        );
    }
}

pub unsafe extern "C" fn vim_set_var(
    mut name: String_0,
    mut value: Object,
    mut arena: *mut Arena,
    mut err: *mut Error,
) -> Object {
    unsafe {
        return dict_set_var(get_globvar_dict(), name, value, false, true, arena, err);
    }
}

pub unsafe extern "C" fn vim_del_var(
    mut name: String_0,
    mut arena: *mut Arena,
    mut err: *mut Error,
) -> Object {
    unsafe {
        return dict_set_var(
            get_globvar_dict(),
            name,
            object {
                type_0: kObjectTypeNil,
                data: C2Rust_Unnamed { boolean: false },
            },
            true,
            true,
            arena,
            err,
        );
    }
}
