//! `nvim_paste()` and `nvim_put()`: bulk text insertion.
//!
//! `nvim_paste` is the streaming one -- it takes a chunk and a phase, so a
//! paste can arrive in pieces and be undone as a unit -- and it defers to
//! the `vim.paste()` Lua handler.  `nvim_put` is the register-style
//! insertion instead, taking a whole array of lines and a motion type.

#![deny(unsafe_op_in_unsafe_fn)]

use super::*;
use crate::api::private::helpers::array_add;
use crate::types::{NUL, PUT_CURSEND};

pub unsafe extern "C" fn nvim_paste(
    mut channel_id: uint64_t,
    mut data: String_0,
    mut crlf: Boolean,
    mut phase: Integer,
    mut arena: *mut Arena,
    mut err: *mut Error,
) -> Boolean {
    unsafe {
        let mut lines: Array = Array {
            size: 0,
            capacity: 0,
            items: ::core::ptr::null_mut::<Object>(),
        };
        let mut args: Array = Array {
            size: 0,
            capacity: 0,
            items: ::core::ptr::null_mut::<Object>(),
        };
        let mut args__items: [Object; 2] = [Object {
            type_0: kObjectTypeNil,
            data: C2Rust_Unnamed { boolean: false },
        }; 2];
        let mut rv: Object = Object {
            type_0: kObjectTypeNil,
            data: C2Rust_Unnamed { boolean: false },
        };
        static cancelled: GlobalCell<bool> = GlobalCell::new(false);
        if !(phase >= -1 as Integer && phase <= 3 as Integer) {
            api_err_invalid(
                err,
                c"phase".as_ptr(),
                ::core::ptr::null::<::core::ffi::c_char>(),
                phase as int64_t,
                false,
            );
            return false;
        }
        's_151: {
            if phase == -1 as Integer || phase == 1 as Integer {
                cancelled.set(false);
                if !(*curbuf.get()).terminal.is_null() {
                    terminal_set_streamed_paste((*curbuf.get()).terminal, true);
                }
            } else if cancelled.get() {
                break 's_151;
            }
            lines = string_to_array(data, crlf, arena);
            args = Array {
                size: 0 as size_t,
                capacity: 0 as size_t,
                items: ::core::ptr::null_mut::<Object>(),
            };
            args__items = [Object {
                type_0: kObjectTypeNil,
                data: C2Rust_Unnamed { boolean: false },
            }; 2];
            args.capacity = 2 as size_t;
            args.items = &raw mut args__items as *mut Object;
            array_add(&mut args, Object::array(lines));
            array_add(&mut args, Object::integer(phase));
            rv = nlua_exec(
                String_0 {
                    data: c"return vim.paste(...)".as_ptr() as *mut ::core::ffi::c_char,
                    size: ::core::mem::size_of::<[::core::ffi::c_char; 22]>()
                        .wrapping_sub(1 as size_t),
                },
                ::core::ptr::null::<::core::ffi::c_char>(),
                args,
                kRetNilBool,
                arena,
                err,
            );
            if (*err).type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int
                || rv.type_0 as ::core::ffi::c_uint
                    == kObjectTypeBoolean as ::core::ffi::c_int as ::core::ffi::c_uint
                    && !rv.data.boolean
            {
                cancelled.set(true);
            }
            if (phase == -1 as Integer
                || phase == 3 as Integer
                || cancelled.get() as ::core::ffi::c_int != 0)
                && !(*curbuf.get()).terminal.is_null()
            {
                terminal_set_streamed_paste((*curbuf.get()).terminal, false);
            }
            if !cancelled.get() && (phase == -1 as Integer || phase == 1 as Integer) {
                paste_store(channel_id, kFalse, NULL_STRING, crlf);
            }
            if !cancelled.get() {
                paste_store(channel_id, kNone, data, crlf);
            }
            if phase == 3 as Integer
                || phase
                    == (if cancelled.get() as ::core::ffi::c_int != 0 {
                        2 as ::core::ffi::c_int
                    } else {
                        -1 as ::core::ffi::c_int
                    }) as Integer
            {
                paste_store(channel_id, kTrue, NULL_STRING, crlf);
            }
        }
        let mut retval: bool = !cancelled.get();
        if phase == -1 as Integer || phase == 3 as Integer {
            cancelled.set(false);
        }
        return retval as Boolean;
    }
}

pub unsafe extern "C" fn nvim_put(
    mut lines: Array,
    mut type_0: String_0,
    mut after: Boolean,
    mut follow: Boolean,
    mut arena: *mut Arena,
    mut err: *mut Error,
) {
    unsafe {
        let mut reg: [yankreg_T; 1] = [yankreg_T {
            y_array: ::core::ptr::null_mut::<String_0>(),
            y_size: 0,
            y_type: kMTCharWise,
            y_width: 0,
            timestamp: 0,
            additional_data: ::core::ptr::null_mut::<AdditionalData>(),
        }];
        if !prepare_yankreg_from_object(&raw mut reg as *mut yankreg_T, type_0, lines.size) {
            api_err_invalid(err, c"type".as_ptr(), type_0.data, 0 as int64_t, true);
            return;
        }
        if lines.size == 0 as size_t {
            return;
        }
        (*(&raw mut reg as *mut yankreg_T)).y_array = arena_alloc(
            arena,
            lines.size.wrapping_mul(::core::mem::size_of::<String_0>()),
            true,
        ) as *mut String_0;
        (*(&raw mut reg as *mut yankreg_T)).y_size = lines.size;
        let mut i: size_t = 0 as size_t;
        while i < lines.size {
            if kObjectTypeString as ::core::ffi::c_int as ::core::ffi::c_uint
                != (*lines.items.add(i)).type_0 as ::core::ffi::c_uint
            {
                api_err_exp(
                    err,
                    c"line".as_ptr(),
                    api_typename(kObjectTypeString),
                    api_typename((*lines.items.add(i)).type_0),
                );
                return;
            }
            let mut line: String_0 = (*lines.items.add(i)).data.string;
            *(*(&raw mut reg as *mut yankreg_T)).y_array.add(i) = copy_string(line, arena);
            memchrsub(
                (*(*(&raw mut reg as *mut yankreg_T)).y_array.add(i)).data
                    as *mut ::core::ffi::c_void,
                NUL as ::core::ffi::c_char,
                NL as ::core::ffi::c_char,
                line.size,
            );
            i = i.wrapping_add(1);
        }
        finish_yankreg_from_object(&raw mut reg as *mut yankreg_T, false);
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
        let mut VIsual_was_active: bool = VIsual_active.get();
        (*msg_silent.ptr()) += 1;
        do_put(
            0 as ::core::ffi::c_int,
            &raw mut reg as *mut yankreg_T,
            if after as ::core::ffi::c_int != 0 {
                FORWARD as ::core::ffi::c_int
            } else {
                BACKWARD as ::core::ffi::c_int
            },
            1 as ::core::ffi::c_int,
            if follow as ::core::ffi::c_int != 0 {
                PUT_CURSEND as ::core::ffi::c_int
            } else {
                0 as ::core::ffi::c_int
            },
        );
        (*msg_silent.ptr()) -= 1;
        VIsual_active.set(VIsual_was_active);
        try_leave(&raw mut tstate, err);
    }
}
