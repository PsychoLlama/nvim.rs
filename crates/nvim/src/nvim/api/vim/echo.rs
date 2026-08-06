//! `nvim_echo()`: printing a chunked, highlighted message.
//!
//! One function, because a message is an array of (text, highlight) chunks
//! that has to be rendered as a unit under the caller's `history`, `err`
//! and `verbose` options, with the message state saved and restored around
//! it.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;

pub unsafe extern "C" fn nvim_echo(
    mut chunks: Array,
    mut history: Boolean,
    mut opts: *mut KeyDict_echo_opts,
    mut err: *mut Error,
) -> Object {
    unsafe {
        let mut kind: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
        let mut is_progress: bool = false;
        let mut needs_clear: bool = false;
        let mut msg_data: MessageData = MessageData {
            source: String_0 {
                data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                size: 0,
            },
            percent: 0,
            title: String_0 {
                data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                size: 0,
            },
            status: String_0 {
                data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                size: 0,
            },
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
            data: C2Rust_Unnamed {
                integer: -1 as Integer,
            },
        };
        let mut hl_msg: HlMessage = parse_hl_msg(chunks, (*opts).err, err);
        if (*err).type_0 as ::core::ffi::c_int == kErrorTypeNone as ::core::ffi::c_int {
            kind = (*opts).kind.data;
            if (*opts).verbose {
                verbose_enter();
            } else if kind.is_null() {
                kind = (if (*opts).err as ::core::ffi::c_int != 0 {
                    c"echoerr".as_ptr()
                } else if history as ::core::ffi::c_int != 0 {
                    c"echomsg".as_ptr()
                } else {
                    c"echo".as_ptr()
                }) as *mut ::core::ffi::c_char;
            }
            is_progress = strequal(kind, c"progress".as_ptr());
            needs_clear = !history;
            if !(is_progress as ::core::ffi::c_int != 0
                || (*opts).status.size == 0 as size_t
                    && (*opts).title.size == 0 as size_t
                    && (*opts).percent == 0 as Integer
                    && (*opts).data.size == 0 as size_t
                    && (*opts).source.size == 0 as size_t)
            {
                api_set_error(
                    err,
                    kErrorTypeValidation,
                    c"Conflict: title/source/status/percent/data not allowed with kind='%s'"
                        .as_ptr(),
                    kind,
                );
            } else if !(!is_progress
                || strequal((*opts).status.data, c"success".as_ptr()) as ::core::ffi::c_int != 0
                || strequal((*opts).status.data, c"failed".as_ptr()) as ::core::ffi::c_int != 0
                || strequal((*opts).status.data, c"running".as_ptr()) as ::core::ffi::c_int != 0
                || strequal((*opts).status.data, c"cancel".as_ptr()) as ::core::ffi::c_int != 0)
            {
                api_err_exp(
                    err,
                    c"status".as_ptr(),
                    c"success|failed|running|cancel".as_ptr(),
                    (*opts).status.data,
                );
            } else if !(!is_progress
                || (*opts).percent >= 0 as Integer && (*opts).percent <= 100 as Integer)
            {
                api_err_invalid(
                    err,
                    c"percent".as_ptr(),
                    c"out of range".as_ptr(),
                    0 as int64_t,
                    false_0 != 0,
                );
            } else if !(!is_progress || (*opts).source.size != 0 as size_t) {
                api_err_required(err, c"opts.source".as_ptr());
            } else if !((*opts).id.type_0 as ::core::ffi::c_uint
                != kObjectTypeInteger as ::core::ffi::c_int as ::core::ffi::c_uint
                || msg_id_exists((*opts).id.data.integer as int64_t) as ::core::ffi::c_int != 0)
            {
                api_set_error(
                    err,
                    kErrorTypeValidation,
                    c"Invalid 'id': %ld".as_ptr(),
                    (*opts).id.data.integer,
                );
            } else {
                msg_data = msg_data {
                    source: (*opts).source,
                    percent: (*opts).percent,
                    title: (*opts).title,
                    status: (*opts).status,
                    data: (*opts).data,
                };
                save_nwr = need_wait_return.get();
                save_lines_left = lines_left.get();
                save_msg_didany = msg_didany.get();
                if (*opts)._truncate {
                    (*no_wait_return.ptr()) += 1;
                    lines_left.set(0 as ::core::ffi::c_int);
                    msg_didany.set(true_0 != 0);
                    msg_no_more.set(true_0 != 0);
                }
                id = msg_multihl(
                    (*opts).id,
                    hl_msg,
                    kind,
                    history,
                    (*opts).err,
                    &raw mut msg_data,
                    &raw mut needs_clear,
                );
                if (*opts)._truncate {
                    msg_no_more.set(false_0 != 0);
                    msg_didany.set(save_msg_didany);
                    lines_left.set(save_lines_left);
                    (*no_wait_return.ptr()) -= 1;
                    need_wait_return.set(save_nwr);
                }
                if (*opts).verbose {
                    verbose_leave();
                    verbose_stop();
                }
                if is_progress {
                    do_autocmd_progress(id, hl_msg, &raw mut msg_data);
                }
                if !needs_clear {
                    return id;
                }
            }
        }
        hl_msg_free(hl_msg);
        return id;
    }
}
