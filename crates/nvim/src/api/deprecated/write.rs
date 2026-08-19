//! Writing messages, and the retired subscription API.
//!
//! `write_msg` is the line-buffered writer behind `nvim_out_write` and the two
//! `nvim_err_write` spellings: it accumulates until a newline, so a client may
//! send a message in pieces.  `nvim_notify` is the message-with-a-level shim.
//! `nvim_subscribe`/`nvim_unsubscribe` are empty: the broadcast events they
//! filtered no longer exist.

#![deny(unsafe_op_in_unsafe_fn)]

use super::*;
use crate::api::private::helpers::{ERROR_INIT, NIL, Reported, array_add};
use crate::guard::Suppress;
use crate::kvec::Kvec;
use crate::types::NUL;

pub unsafe fn nvim_subscribe(_channel_id: uint64_t, _event: String_0) {}

pub unsafe fn nvim_unsubscribe(_channel_id: uint64_t, _event: String_0) {}

unsafe fn write_msg(mut message: String_0, mut to_err: bool, mut writeln: bool) {
    unsafe {
        static out_line_buf: GlobalCell<StringBuilder> = GlobalCell::new(StringBuilder {
            size: 0 as size_t,
            capacity: 0 as size_t,
            items: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        });
        static err_line_buf: GlobalCell<StringBuilder> = GlobalCell::new(StringBuilder {
            size: 0 as size_t,
            capacity: 0 as size_t,
            items: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        });
        let mut line_buf: *mut StringBuilder = if to_err as ::core::ffi::c_int != 0 {
            err_line_buf.ptr()
        } else {
            out_line_buf.ptr()
        };
        let no_prompt = Suppress::wait_return();
        let mut i: uint32_t = 0 as uint32_t;
        while (i as size_t) < message.len() {
            if got_int.get() {
                break;
            }
            if (*line_buf).capacity == 0 as size_t {
                (*line_buf).capacity = LINE_BUFFER_MIN_SIZE as ::core::ffi::c_int as size_t;
                (*line_buf).items = xrealloc(
                    (*line_buf).items as *mut ::core::ffi::c_void,
                    ::core::mem::size_of::<::core::ffi::c_char>()
                        .wrapping_mul((*line_buf).capacity),
                ) as *mut ::core::ffi::c_char;
            }
            if *message.data().offset(i as isize) as ::core::ffi::c_int == NL {
                // `kv_push`, whose growth step c2rust expanded inline.
                Kvec::new(
                    &mut (*line_buf).size,
                    &mut (*line_buf).capacity,
                    &mut (*line_buf).items,
                )
                .push('\0' as ::core::ffi::c_char);
                if to_err {
                    emsg((*line_buf).items);
                } else {
                    msg((*line_buf).items, 0 as ::core::ffi::c_int);
                }
                if msg_silent.get() == 0 as ::core::ffi::c_int {
                    msg_didout.set(true);
                }
                (*line_buf).size = (*line_buf).size.wrapping_sub((*line_buf).size);
                (*line_buf).capacity = LINE_BUFFER_MIN_SIZE as ::core::ffi::c_int as size_t;
                (*line_buf).items = xrealloc(
                    (*line_buf).items as *mut ::core::ffi::c_void,
                    ::core::mem::size_of::<::core::ffi::c_char>()
                        .wrapping_mul((*line_buf).capacity),
                ) as *mut ::core::ffi::c_char;
            } else if *message.data().offset(i as isize) as ::core::ffi::c_int == NUL {
                // `kv_push`, whose growth step c2rust expanded inline.
                Kvec::new(
                    &mut (*line_buf).size,
                    &mut (*line_buf).capacity,
                    &mut (*line_buf).items,
                )
                .push('\n' as ::core::ffi::c_char);
            } else {
                // `kv_push`, whose growth step c2rust expanded inline.
                Kvec::new(
                    &mut (*line_buf).size,
                    &mut (*line_buf).capacity,
                    &mut (*line_buf).items,
                )
                .push(*message.data().offset(i as isize));
            }
            i = i.wrapping_add(1);
        }
        if writeln {
            if (*line_buf).capacity == 0 as size_t {
                (*line_buf).capacity = LINE_BUFFER_MIN_SIZE as ::core::ffi::c_int as size_t;
                (*line_buf).items = xrealloc(
                    (*line_buf).items as *mut ::core::ffi::c_void,
                    ::core::mem::size_of::<::core::ffi::c_char>()
                        .wrapping_mul((*line_buf).capacity),
                ) as *mut ::core::ffi::c_char;
            }
            if '\n' as ::core::ffi::c_int == NL {
                // `kv_push`, whose growth step c2rust expanded inline.
                Kvec::new(
                    &mut (*line_buf).size,
                    &mut (*line_buf).capacity,
                    &mut (*line_buf).items,
                )
                .push('\0' as ::core::ffi::c_char);
                if to_err {
                    emsg((*line_buf).items);
                } else {
                    msg((*line_buf).items, 0 as ::core::ffi::c_int);
                }
                if msg_silent.get() == 0 as ::core::ffi::c_int {
                    msg_didout.set(true);
                }
                (*line_buf).size = (*line_buf).size.wrapping_sub((*line_buf).size);
                (*line_buf).capacity = LINE_BUFFER_MIN_SIZE as ::core::ffi::c_int as size_t;
                (*line_buf).items = xrealloc(
                    (*line_buf).items as *mut ::core::ffi::c_void,
                    ::core::mem::size_of::<::core::ffi::c_char>()
                        .wrapping_mul((*line_buf).capacity),
                ) as *mut ::core::ffi::c_char;
            } else {
                // `kv_push`, whose growth step c2rust expanded inline. The C
                // spelled the byte `NL`, which the macro tested against `NUL`
                // before pushing; c2rust kept both arms of a comparison of two
                // constants that can never be equal.
                Kvec::new(
                    &mut (*line_buf).size,
                    &mut (*line_buf).capacity,
                    &mut (*line_buf).items,
                )
                .push('\n' as ::core::ffi::c_char);
            }
        }
        drop(no_prompt);
        msg_end();
    }
}

pub unsafe fn nvim_out_write(str: String_0) {
    unsafe {
        write_msg(str, false, false);
    }
}

pub unsafe fn nvim_err_write(str: String_0) {
    unsafe {
        write_msg(str, true, false);
    }
}

pub unsafe fn nvim_err_writeln(str: String_0) {
    unsafe {
        write_msg(str, true, true);
    }
}

pub unsafe fn nvim_notify(
    msg_0: String_0,
    log_level: Integer,
    opts: Dict,
    arena: *mut Arena,
) -> Result<Object, Error> {
    let mut error = ERROR_INIT;
    let err = &raw mut error;
    unsafe {
        let mut args: Array = Array {
            size: 0 as size_t,
            capacity: 0 as size_t,
            items: ::core::ptr::null_mut::<Object>(),
        };
        let mut args__items: [Object; 3] = [NIL; 3];
        args.capacity = 3 as size_t;
        args.items = &raw mut args__items as *mut Object;
        array_add(&mut args, Object::string(msg_0));
        array_add(&mut args, Object::integer(log_level));
        array_add(&mut args, Object::dict(opts));
        nlua_exec(
            String_0::from_raw_parts(
                c"return vim.notify(...)".as_ptr() as *mut ::core::ffi::c_char,
                ::core::mem::size_of::<[::core::ffi::c_char; 23]>().wrapping_sub(1 as size_t),
            ),
            ::core::ptr::null::<::core::ffi::c_char>(),
            args,
            kRetObject,
            arena,
            err,
        )
        .reported(error)
    }
}
