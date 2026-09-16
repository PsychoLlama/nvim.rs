//! Writing messages, and the retired subscription API.
//!
//! `write_msg` is the line-buffered writer behind `nvim_out_write` and the two
//! `nvim_err_write` spellings: it accumulates until a newline, so a client may
//! send a message in pieces.  `nvim_notify` is the message-with-a-level shim.
//! `nvim_subscribe`/`nvim_unsubscribe` are empty: the broadcast events they
//! filtered no longer exist.

#![deny(unsafe_op_in_unsafe_fn)]
#![allow(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]
// The globals here keep upstream's spelling; upper-casing them is a per-module rewrite.
#![allow(non_upper_case_globals)]

use super::*;
use crate::guard::Suppress;
use crate::kvec::Kvec;
use crate::message::{emsg_ptr, msg_ptr};
use crate::types::NUL;
use crate::types::builders::ArrayBuf;

pub fn nvim_subscribe(_channel_id: uint64_t, _event: String_0) {}

pub fn nvim_unsubscribe(_channel_id: uint64_t, _event: String_0) {}

fn write_msg(message: String_0, to_err: bool, writeln: bool) {
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
    let line_buf: *mut StringBuilder = if to_err {
        err_line_buf.ptr()
    } else {
        out_line_buf.ptr()
    };

    // The buffer is reached through the pointer at each use rather than
    // borrowed once for the body: `msg`/`emsg` below can re-enter this
    // function, and a `&mut` held across them would promise otherwise.

    // C's `kv_push`, whose growth step c2rust expanded at every use.
    let push = |byte: ::core::ffi::c_char| {
        // SAFETY: `line_buf` names one of the two statics above, whose
        // vector is null with a zero capacity or one heap block.
        unsafe {
            Kvec::new(
                &mut (*line_buf).size,
                &mut (*line_buf).capacity,
                &mut (*line_buf).items,
            )
            .push(byte);
        }
    };
    // Give the buffer its minimum size, which is also what it shrinks back
    // to after every line. One byte per element, so the byte count is the
    // capacity.
    let reserve = || {
        // SAFETY: as `push`.
        unsafe {
            (*line_buf).capacity = LINE_BUFFER_MIN_SIZE as size_t;
            (*line_buf).items = xrealloc((*line_buf).items.cast(), (*line_buf).capacity).cast();
        }
    };
    // Publish the line the buffer holds and empty it.
    let flush = || {
        push(0);
        // SAFETY: as `push`; the buffer is NUL-terminated by the push above.
        unsafe {
            if to_err {
                emsg_ptr((*line_buf).items);
            } else {
                msg_ptr((*line_buf).items, 0 as ::core::ffi::c_int);
            }
        }
        if msg_silent.get() == 0 as ::core::ffi::c_int {
            msg_didout.set(true);
        }
        // SAFETY: as `push`.
        unsafe { (*line_buf).size = 0 as size_t };
        reserve();
    };

    let no_prompt = Suppress::wait_return();
    let mut i: size_t = 0;
    while i < message.len() {
        if got_int.get() {
            break;
        }
        // SAFETY: as `push`.
        if unsafe { (*line_buf).capacity } == 0 as size_t {
            reserve();
        }
        // SAFETY: `i` is below the length, so the byte is in the message.
        let byte = unsafe { *message.data().add(i) };
        if ::core::ffi::c_int::from(byte) == NL {
            flush();
        } else if ::core::ffi::c_int::from(byte) == NUL {
            // A NUL in the text stands for a newline, as it does everywhere
            // a buffer line is passed as a C string.
            push('\n' as ::core::ffi::c_char);
        } else {
            push(byte);
        }
        i += 1;
    }
    if writeln {
        // SAFETY: as `push`.
        if unsafe { (*line_buf).capacity } == 0 as size_t {
            reserve();
        }
        // The C spelled the byte `NL`, which `kv_push`'s macro tested
        // against `NUL` before pushing; c2rust kept both arms of a
        // comparison of two constants that can never be equal, and this is
        // the arm that runs.
        flush();
    }
    drop(no_prompt);
    msg_end();
}

pub fn nvim_out_write(str: String_0) {
    write_msg(str, false, false);
}

pub fn nvim_err_write(str: String_0) {
    write_msg(str, true, false);
}

pub fn nvim_err_writeln(str: String_0) {
    write_msg(str, true, true);
}

/// # Safety
///
/// `msg_0` must be a well-formed API string: `size` readable bytes with a NUL
/// at `data[size]`. `opts` must be a well-formed API dictionary, its `size`
/// entries initialized. `arena` must point at a live arena, which the memory
/// this answers with is taken from and must outlive.
pub unsafe fn nvim_notify(
    msg_0: String_0,
    log_level: Integer,
    opts: ApiDict,
    arena: *mut Arena,
) -> Result<Object, Error> {
    let mut args = ArrayBuf::<3>::new();
    args.push(Object::string(msg_0));
    args.push(Object::integer(log_level));
    args.push(Object::dict(opts));
    let code = String_0::from_cstr(c"return vim.notify(...)");
    let (args, no_name) = (args.array(), ::core::ptr::null());
    // SAFETY: `code` borrows a static, `args` borrows this frame's buffer
    // for the length of the call, and `arena`/`error` are the caller's and
    // this frame's slot.
    unsafe { nlua_exec(&code, no_name, args, kRetObject, arena) }
}
