//! Feeding the editor keys, and the mappings they may hit.
//!
//! `nvim_feedkeys` and `nvim_input` are the two ends of the input path: one
//! goes through the typeahead buffer with the caller's mode flags, the
//! other straight into it as if typed.  `nvim_input_mouse` synthesises a
//! mouse event from (button, action, grid, row, col).  The keymap
//! accessors sit here because they answer for the same table `nvim_input`
//! is resolved against.

#![deny(unsafe_op_in_unsafe_fn)]

use super::*;
use crate::api::private::helpers::{ERROR_INIT, Reported};
use crate::keycodes::{
    KE_LEFTDRAG, KE_LEFTMOUSE, KE_LEFTRELEASE, KE_MIDDLEMOUSE, KE_MOUSEDOWN, KE_MOUSELEFT,
    KE_MOUSEMOVE, KE_MOUSERIGHT, KE_MOUSEUP, KE_RIGHTMOUSE, KE_X1MOUSE, KE_X2MOUSE,
};

pub unsafe fn nvim_feedkeys(keys: String_0, mode: String_0, escape_ks: Boolean) {
    unsafe {
        let mut remap: bool = true;
        let mut insert: bool = false;
        let mut typed: bool = false;
        let mut execute: bool = false;
        let mut dangerous: bool = false;
        let mut lowlevel: bool = false;
        let mut i: size_t = 0 as size_t;
        while i < mode.len() {
            match *mode.data().add(i) as ::core::ffi::c_int {
                110 => {
                    remap = false;
                }
                109 => {
                    remap = true;
                }
                116 => {
                    typed = true;
                }
                105 => {
                    insert = true;
                }
                120 => {
                    execute = true;
                }
                33 => {
                    dangerous = true;
                }
                76 => {
                    lowlevel = true;
                }
                _ => {}
            }
            i = i.wrapping_add(1);
        }
        if keys.len() == 0 as size_t && !execute {
            return;
        }
        let mut keys_esc: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
        if escape_ks {
            keys_esc = vim_strsave_escape_ks(keys.data());
        } else {
            keys_esc = keys.data();
        }
        if lowlevel {
            input_enqueue_raw(keys_esc, strlen(keys_esc));
        } else {
            ins_typebuf(
                keys_esc,
                if remap as ::core::ffi::c_int != 0 {
                    REMAP_YES as ::core::ffi::c_int
                } else {
                    REMAP_NONE as ::core::ffi::c_int
                },
                if insert as ::core::ffi::c_int != 0 {
                    0 as ::core::ffi::c_int
                } else {
                    (*typebuf.ptr()).tb_len
                },
                !typed,
                false,
            );
            if vgetc_busy.get() != 0 {
                typebuf_was_filled.set(true);
            }
        }
        if escape_ks {
            xfree(keys_esc as *mut ::core::ffi::c_void);
        }
        if execute {
            let mut save_msg_scroll: ::core::ffi::c_int = msg_scroll.get();
            msg_scroll.set(0);
            if !dangerous {
                (*ex_normal_busy.ptr()) += 1;
            }
            exec_normal(true, lowlevel);
            if !dangerous {
                (*ex_normal_busy.ptr()) -= 1;
            }
            (*msg_scroll.ptr()) |= save_msg_scroll;
        }
    }
}

pub unsafe fn nvim_input(channel_id: uint64_t, keys: String_0) -> Integer {
    unsafe {
        may_trigger_vim_suspend_resume(false);
        return input_enqueue(channel_id, keys) as Integer;
    }
}

pub unsafe fn nvim_input_mouse(
    button: String_0,
    action: String_0,
    modifier: String_0,
    grid: Integer,
    row: Integer,
    col: Integer,
) -> Result<(), Error> {
    let mut error = ERROR_INIT;
    let err = &raw mut error;
    unsafe {
        let mut code: ::core::ffi::c_int = 0;
        let mut modmask: ::core::ffi::c_int = 0;
        may_trigger_vim_suspend_resume(false);
        '_error: {
            if !(button.data().is_null() || action.data().is_null()) {
                code = 0 as ::core::ffi::c_int;
                if strequal(button.data(), c"left".as_ptr()) {
                    code = KE_LEFTMOUSE as ::core::ffi::c_int;
                } else if strequal(button.data(), c"middle".as_ptr()) {
                    code = KE_MIDDLEMOUSE as ::core::ffi::c_int;
                } else if strequal(button.data(), c"right".as_ptr()) {
                    code = KE_RIGHTMOUSE as ::core::ffi::c_int;
                } else if strequal(button.data(), c"wheel".as_ptr()) {
                    code = KE_MOUSEDOWN as ::core::ffi::c_int;
                } else if strequal(button.data(), c"x1".as_ptr()) {
                    code = KE_X1MOUSE as ::core::ffi::c_int;
                } else if strequal(button.data(), c"x2".as_ptr()) {
                    code = KE_X2MOUSE as ::core::ffi::c_int;
                } else if strequal(button.data(), c"move".as_ptr()) {
                    code = KE_MOUSEMOVE as ::core::ffi::c_int;
                } else {
                    break '_error;
                }
                if code == KE_MOUSEDOWN as ::core::ffi::c_int {
                    if strequal(action.data(), c"down".as_ptr()) {
                        code = KE_MOUSEUP as ::core::ffi::c_int;
                    } else if !strequal(action.data(), c"up".as_ptr()) {
                        if strequal(action.data(), c"left".as_ptr()) {
                            code = KE_MOUSERIGHT as ::core::ffi::c_int;
                        } else if strequal(action.data(), c"right".as_ptr()) {
                            code = KE_MOUSELEFT as ::core::ffi::c_int;
                        } else {
                            break '_error;
                        }
                    }
                } else if code != KE_MOUSEMOVE as ::core::ffi::c_int {
                    if !strequal(action.data(), c"press".as_ptr()) {
                        if strequal(action.data(), c"drag".as_ptr()) {
                            code += KE_LEFTDRAG as ::core::ffi::c_int
                                - KE_LEFTMOUSE as ::core::ffi::c_int;
                        } else if strequal(action.data(), c"release".as_ptr()) {
                            code += KE_LEFTRELEASE as ::core::ffi::c_int
                                - KE_LEFTMOUSE as ::core::ffi::c_int;
                        } else {
                            break '_error;
                        }
                    }
                }
                modmask = 0 as ::core::ffi::c_int;
                let mut i: size_t = 0 as size_t;
                while i < modifier.len() {
                    let mut byte: ::core::ffi::c_char = *modifier.data().add(i);
                    if byte as ::core::ffi::c_int != '-' as ::core::ffi::c_int {
                        let mut mod_0: ::core::ffi::c_int =
                            name_to_mod_mask(byte as ::core::ffi::c_int);
                        if !(mod_0 != 0 as ::core::ffi::c_int) {
                            api_set_error(
                                err,
                                kErrorTypeValidation,
                                c"Invalid modifier: %c".as_ptr(),
                                byte as ::core::ffi::c_int,
                            );
                            return ().reported(error);
                        }
                        modmask |= mod_0;
                    }
                    i = i.wrapping_add(1);
                }
                input_enqueue_mouse(
                    code,
                    modmask as uint8_t,
                    grid as ::core::ffi::c_int,
                    row as ::core::ffi::c_int,
                    col as ::core::ffi::c_int,
                );
                return ().reported(error);
            }
        }
        api_set_error(
            err,
            kErrorTypeValidation,
            c"invalid button or action".as_ptr(),
        );
    }
    ().reported(error)
}

pub unsafe fn nvim_replace_termcodes(
    str: String_0,
    from_part: Boolean,
    do_lt: Boolean,
    special: Boolean,
) -> String_0 {
    unsafe {
        if str.len() == 0 as size_t {
            return String_0::from_raw_parts(
                ::core::ptr::null_mut::<::core::ffi::c_char>(),
                0 as size_t,
            );
        }
        let mut flags: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        if from_part {
            flags |= REPTERM_FROM_PART as ::core::ffi::c_int;
        }
        if do_lt {
            flags |= REPTERM_DO_LT as ::core::ffi::c_int;
        }
        if !special {
            flags |= REPTERM_NO_SPECIAL as ::core::ffi::c_int;
        }
        let mut ptr: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
        replace_termcodes(
            str.data(),
            str.len(),
            &raw mut ptr,
            0 as scid_T,
            flags,
            ::core::ptr::null_mut::<bool>(),
            p_cpo.get(),
        );
        return cstr_as_string(ptr);
    }
}

pub unsafe fn nvim_get_keymap(mode: String_0, arena: *mut Arena) -> Array {
    unsafe {
        return keymap_array(mode, ::core::ptr::null_mut::<buf_T>(), arena);
    }
}

pub unsafe fn nvim_set_keymap(
    channel_id: uint64_t,
    mode: String_0,
    lhs: String_0,
    rhs: String_0,
    opts: *mut KeyDict_keymap,
) -> Result<(), Error> {
    let mut error = ERROR_INIT;
    let err = &raw mut error;
    unsafe {
        modify_keymap(channel_id, -1 as Buffer, false, mode, lhs, rhs, opts, err);
    }
    ().reported(error)
}

pub unsafe fn nvim_del_keymap(
    channel_id: uint64_t,
    mode: String_0,
    lhs: String_0,
) -> Result<(), Error> {
    unsafe { nvim_buf_del_keymap(channel_id, -1 as Buffer, mode, lhs) }
}

pub unsafe fn nvim_select_popupmenu_item(
    item: Integer,
    mut insert: Boolean,
    finish: Boolean,
    _opts: *mut KeyDict_empty,
) {
    if finish {
        insert = true;
    }
    pum_ext_select_item(item as ::core::ffi::c_int, insert, finish);
}
