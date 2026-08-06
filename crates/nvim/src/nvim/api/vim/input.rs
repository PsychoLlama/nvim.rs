//! Feeding the editor keys, and the mappings they may hit.
//!
//! `nvim_feedkeys` and `nvim_input` are the two ends of the input path: one
//! goes through the typeahead buffer with the caller's mode flags, the
//! other straight into it as if typed.  `nvim_input_mouse` synthesises a
//! mouse event from (button, action, grid, row, col).  The keymap
//! accessors sit here because they answer for the same table `nvim_input`
//! is resolved against.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;

pub unsafe extern "C" fn nvim_feedkeys(
    mut keys: String_0,
    mut mode: String_0,
    mut escape_ks: Boolean,
) {
    unsafe {
        let mut remap: bool = true_0 != 0;
        let mut insert: bool = false_0 != 0;
        let mut typed: bool = false_0 != 0;
        let mut execute: bool = false_0 != 0;
        let mut dangerous: bool = false_0 != 0;
        let mut lowlevel: bool = false_0 != 0;
        let mut i: size_t = 0 as size_t;
        while i < mode.size {
            match *mode.data.offset(i as isize) as ::core::ffi::c_int {
                110 => {
                    remap = false_0 != 0;
                }
                109 => {
                    remap = true_0 != 0;
                }
                116 => {
                    typed = true_0 != 0;
                }
                105 => {
                    insert = true_0 != 0;
                }
                120 => {
                    execute = true_0 != 0;
                }
                33 => {
                    dangerous = true_0 != 0;
                }
                76 => {
                    lowlevel = true_0 != 0;
                }
                _ => {}
            }
            i = i.wrapping_add(1);
        }
        if keys.size == 0 as size_t && !execute {
            return;
        }
        let mut keys_esc: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
        if escape_ks {
            keys_esc = vim_strsave_escape_ks(keys.data);
        } else {
            keys_esc = keys.data;
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
                false_0 != 0,
            );
            if vgetc_busy.get() != 0 {
                typebuf_was_filled.set(true_0 != 0);
            }
        }
        if escape_ks {
            xfree(keys_esc as *mut ::core::ffi::c_void);
        }
        if execute {
            let mut save_msg_scroll: ::core::ffi::c_int = msg_scroll.get();
            msg_scroll.set(false_0);
            if !dangerous {
                (*ex_normal_busy.ptr()) += 1;
            }
            exec_normal(true_0 != 0, lowlevel);
            if !dangerous {
                (*ex_normal_busy.ptr()) -= 1;
            }
            (*msg_scroll.ptr()) |= save_msg_scroll;
        }
    }
}

pub unsafe extern "C" fn nvim_input(mut channel_id: uint64_t, mut keys: String_0) -> Integer {
    unsafe {
        may_trigger_vim_suspend_resume(false_0 != 0);
        return input_enqueue(channel_id, keys) as Integer;
    }
}

pub unsafe extern "C" fn nvim_input_mouse(
    mut button: String_0,
    mut action: String_0,
    mut modifier: String_0,
    mut grid: Integer,
    mut row: Integer,
    mut col: Integer,
    mut err: *mut Error,
) {
    unsafe {
        let mut code: ::core::ffi::c_int = 0;
        let mut modmask: ::core::ffi::c_int = 0;
        may_trigger_vim_suspend_resume(false_0 != 0);
        '_error: {
            if !(button.data.is_null() || action.data.is_null()) {
                code = 0 as ::core::ffi::c_int;
                if strequal(
                    button.data,
                    b"left\0".as_ptr() as *const ::core::ffi::c_char,
                ) {
                    code = KE_LEFTMOUSE as ::core::ffi::c_int;
                } else if strequal(
                    button.data,
                    b"middle\0".as_ptr() as *const ::core::ffi::c_char,
                ) {
                    code = KE_MIDDLEMOUSE as ::core::ffi::c_int;
                } else if strequal(
                    button.data,
                    b"right\0".as_ptr() as *const ::core::ffi::c_char,
                ) {
                    code = KE_RIGHTMOUSE as ::core::ffi::c_int;
                } else if strequal(
                    button.data,
                    b"wheel\0".as_ptr() as *const ::core::ffi::c_char,
                ) {
                    code = KE_MOUSEDOWN as ::core::ffi::c_int;
                } else if strequal(button.data, b"x1\0".as_ptr() as *const ::core::ffi::c_char) {
                    code = KE_X1MOUSE as ::core::ffi::c_int;
                } else if strequal(button.data, b"x2\0".as_ptr() as *const ::core::ffi::c_char) {
                    code = KE_X2MOUSE as ::core::ffi::c_int;
                } else if strequal(
                    button.data,
                    b"move\0".as_ptr() as *const ::core::ffi::c_char,
                ) {
                    code = KE_MOUSEMOVE as ::core::ffi::c_int;
                } else {
                    break '_error;
                }
                if code == KE_MOUSEDOWN as ::core::ffi::c_int {
                    if strequal(
                        action.data,
                        b"down\0".as_ptr() as *const ::core::ffi::c_char,
                    ) {
                        code = KE_MOUSEUP as ::core::ffi::c_int;
                    } else if !strequal(action.data, b"up\0".as_ptr() as *const ::core::ffi::c_char)
                    {
                        if strequal(
                            action.data,
                            b"left\0".as_ptr() as *const ::core::ffi::c_char,
                        ) {
                            code = KE_MOUSERIGHT as ::core::ffi::c_int;
                        } else if strequal(
                            action.data,
                            b"right\0".as_ptr() as *const ::core::ffi::c_char,
                        ) {
                            code = KE_MOUSELEFT as ::core::ffi::c_int;
                        } else {
                            break '_error;
                        }
                    }
                } else if code != KE_MOUSEMOVE as ::core::ffi::c_int {
                    if !strequal(
                        action.data,
                        b"press\0".as_ptr() as *const ::core::ffi::c_char,
                    ) {
                        if strequal(
                            action.data,
                            b"drag\0".as_ptr() as *const ::core::ffi::c_char,
                        ) {
                            code += KE_LEFTDRAG as ::core::ffi::c_int
                                - KE_LEFTMOUSE as ::core::ffi::c_int;
                        } else if strequal(
                            action.data,
                            b"release\0".as_ptr() as *const ::core::ffi::c_char,
                        ) {
                            code += KE_LEFTRELEASE as ::core::ffi::c_int
                                - KE_LEFTMOUSE as ::core::ffi::c_int;
                        } else {
                            break '_error;
                        }
                    }
                }
                modmask = 0 as ::core::ffi::c_int;
                let mut i: size_t = 0 as size_t;
                while i < modifier.size {
                    let mut byte: ::core::ffi::c_char = *modifier.data.offset(i as isize);
                    if byte as ::core::ffi::c_int != '-' as ::core::ffi::c_int {
                        let mut mod_0: ::core::ffi::c_int =
                            name_to_mod_mask(byte as ::core::ffi::c_int);
                        if !(mod_0 != 0 as ::core::ffi::c_int) {
                            api_set_error(
                                err,
                                kErrorTypeValidation,
                                b"Invalid modifier: %c\0".as_ptr() as *const ::core::ffi::c_char,
                                byte as ::core::ffi::c_int,
                            );
                            return;
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
                return;
            }
        }
        api_set_error(
            err,
            kErrorTypeValidation,
            b"invalid button or action\0".as_ptr() as *const ::core::ffi::c_char,
        );
    }
}

pub unsafe extern "C" fn nvim_replace_termcodes(
    mut str: String_0,
    mut from_part: Boolean,
    mut do_lt: Boolean,
    mut special: Boolean,
) -> String_0 {
    unsafe {
        if str.size == 0 as size_t {
            return String_0 {
                data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                size: 0 as size_t,
            };
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
            str.data,
            str.size,
            &raw mut ptr,
            0 as scid_T,
            flags,
            ::core::ptr::null_mut::<bool>(),
            p_cpo.get(),
        );
        return cstr_as_string(ptr);
    }
}

pub unsafe extern "C" fn nvim_get_keymap(mut mode: String_0, mut arena: *mut Arena) -> Array {
    unsafe {
        return keymap_array(mode, ::core::ptr::null_mut::<buf_T>(), arena);
    }
}

pub unsafe extern "C" fn nvim_set_keymap(
    mut channel_id: uint64_t,
    mut mode: String_0,
    mut lhs: String_0,
    mut rhs: String_0,
    mut opts: *mut KeyDict_keymap,
    mut err: *mut Error,
) {
    unsafe {
        modify_keymap(
            channel_id,
            -1 as Buffer,
            false_0 != 0,
            mode,
            lhs,
            rhs,
            opts,
            err,
        );
    }
}

pub unsafe extern "C" fn nvim_del_keymap(
    mut channel_id: uint64_t,
    mut mode: String_0,
    mut lhs: String_0,
    mut err: *mut Error,
) {
    unsafe {
        nvim_buf_del_keymap(channel_id, -1 as Buffer, mode, lhs, err);
    }
}

pub unsafe extern "C" fn nvim_select_popupmenu_item(
    mut item: Integer,
    mut insert: Boolean,
    mut finish: Boolean,
    mut _opts: *mut KeyDict_empty,
    mut _err: *mut Error,
) {
    if finish {
        insert = true_0 != 0;
    }
    pum_ext_select_item(item as ::core::ffi::c_int, insert as bool, finish as bool);
}
