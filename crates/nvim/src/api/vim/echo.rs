//! `nvim_echo()`: printing a chunked, highlighted message.
//!
//! One function, because a message is an array of (text, highlight) chunks
//! that has to be rendered as a unit under the caller's `history`, `err`
//! and `verbose` options, with the message state saved and restored around
//! it.

#![deny(unsafe_op_in_unsafe_fn)]

use super::*;
use crate::api::private::helpers::{ERROR_INIT, Reported};
use crate::guard::Suppress;

pub unsafe fn nvim_echo(
    chunks: Array,
    history: Boolean,
    opts: *mut KeyDict_echo_opts,
) -> Result<Object, Error> {
    let mut error = ERROR_INIT;
    let err = &raw mut error;
    let mut kind: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut is_progress: bool = false;
    let mut needs_clear: bool = false;
    let mut msg_data: MessageData = MessageData {
        source: String_0::NULL,
        percent: 0,
        title: String_0::NULL,
        status: String_0::NULL,
        data: Dict {
            size: 0,
            capacity: 0,
            items: ::core::ptr::null_mut::<KeyValuePair>(),
        },
    };
    let mut save_nwr: bool = false;
    let mut save_lines_left: ::core::ffi::c_int = 0;
    let mut save_msg_didany: bool = false;
    let mut id: Object = object {
        type_0: kObjectTypeInteger,
        data: object_data {
            integer: -1 as Integer,
        },
    };
    let mut hl_msg: HlMessage = unsafe { parse_hl_msg(chunks, (*opts).err, err) };
    if unsafe { (*err).type_0 } as ::core::ffi::c_int == kErrorTypeNone as ::core::ffi::c_int {
        kind = unsafe { (*opts).kind }.data();
        if unsafe { (*opts).verbose } {
            unsafe { verbose_enter() };
        } else if kind.is_null() {
            kind = (if unsafe { (*opts).err } as ::core::ffi::c_int != 0 {
                c"echoerr".as_ptr()
            } else if history as ::core::ffi::c_int != 0 {
                c"echomsg".as_ptr()
            } else {
                c"echo".as_ptr()
            }) as *mut ::core::ffi::c_char;
        }
        is_progress = unsafe { strequal(kind, c"progress".as_ptr()) };
        needs_clear = !history;
        if !(is_progress as ::core::ffi::c_int != 0
            || unsafe { (*opts).status }.len() == 0 as size_t
                && unsafe { (*opts).title }.len() == 0 as size_t
                && unsafe { (*opts).percent } == 0 as Integer
                && unsafe { (*opts).data.size } == 0 as size_t
                && unsafe { (*opts).source }.len() == 0 as size_t)
        {
            unsafe {
                api_set_error(
                    err,
                    kErrorTypeValidation,
                    c"Conflict: title/source/status/percent/data not allowed with kind='%s'"
                        .as_ptr(),
                    kind,
                )
            };
        } else if !(!is_progress
            || unsafe { strequal((*opts).status.data(), c"success".as_ptr()) }
                as ::core::ffi::c_int
                != 0
            || unsafe { strequal((*opts).status.data(), c"failed".as_ptr()) } as ::core::ffi::c_int
                != 0
            || unsafe { strequal((*opts).status.data(), c"running".as_ptr()) }
                as ::core::ffi::c_int
                != 0
            || unsafe { strequal((*opts).status.data(), c"cancel".as_ptr()) } as ::core::ffi::c_int
                != 0)
        {
            unsafe {
                api_err_exp(
                    err,
                    c"status".as_ptr(),
                    c"success|failed|running|cancel".as_ptr(),
                    (*opts).status.data(),
                )
            };
        } else if !(!is_progress
            || unsafe { (*opts).percent } >= 0 as Integer
                && unsafe { (*opts).percent } <= 100 as Integer)
        {
            unsafe {
                api_err_invalid(
                    err,
                    c"percent".as_ptr(),
                    c"out of range".as_ptr(),
                    0 as int64_t,
                    false,
                )
            };
        } else if !(!is_progress || unsafe { (*opts).source }.len() != 0 as size_t) {
            unsafe { api_err_required(err, c"opts.source".as_ptr()) };
        } else if !(unsafe { (*opts).id.type_0 } as ::core::ffi::c_uint
            != kObjectTypeInteger as ::core::ffi::c_int as ::core::ffi::c_uint
            || msg_id_exists(unsafe { (*opts).id.data.integer } as int64_t) as ::core::ffi::c_int
                != 0)
        {
            unsafe {
                api_set_error(
                    err,
                    kErrorTypeValidation,
                    c"Invalid 'id': %ld".as_ptr(),
                    (*opts).id.data.integer,
                )
            };
        } else {
            msg_data = msg_data {
                source: unsafe { (*opts).source },
                percent: unsafe { (*opts).percent },
                title: unsafe { (*opts).title },
                status: unsafe { (*opts).status },
                data: unsafe { (*opts).data },
            };
            save_nwr = need_wait_return.get();
            save_lines_left = lines_left.get();
            save_msg_didany = msg_didany.get();
            let no_prompt = unsafe { (*opts)._truncate }.then(Suppress::wait_return);
            if unsafe { (*opts)._truncate } {
                lines_left.set(0 as ::core::ffi::c_int);
                msg_didany.set(true);
                msg_no_more.set(true);
            }
            id = unsafe {
                msg_multihl(
                    (*opts).id,
                    hl_msg.clone(),
                    kind,
                    history,
                    (*opts).err,
                    &raw mut msg_data,
                    &raw mut needs_clear,
                )
            };
            if unsafe { (*opts)._truncate } {
                msg_no_more.set(false);
                msg_didany.set(save_msg_didany);
                lines_left.set(save_lines_left);
                drop(no_prompt);
                need_wait_return.set(save_nwr);
            }
            if unsafe { (*opts).verbose } {
                unsafe { verbose_leave() };
                unsafe { verbose_stop() };
            }
            if is_progress {
                unsafe { do_autocmd_progress(id, hl_msg.clone(), &raw mut msg_data) };
            }
            if !needs_clear {
                return id.reported(error);
            }
        }
    }
    unsafe { hl_msg_free(hl_msg) };
    id.reported(error)
}
