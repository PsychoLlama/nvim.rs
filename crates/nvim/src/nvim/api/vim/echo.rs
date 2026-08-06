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
        let mut hl_msg: HlMessage = parse_hl_msg(chunks, (*opts).err as bool, err);
        if (*err).type_0 as ::core::ffi::c_int == kErrorTypeNone as ::core::ffi::c_int {
            kind = (*opts).kind.data;
            if (*opts).verbose {
                verbose_enter();
            } else if kind.is_null() {
                kind = (if (*opts).err as ::core::ffi::c_int != 0 {
                    b"echoerr\0".as_ptr() as *const ::core::ffi::c_char
                } else if history as ::core::ffi::c_int != 0 {
                    b"echomsg\0".as_ptr() as *const ::core::ffi::c_char
                } else {
                    b"echo\0".as_ptr() as *const ::core::ffi::c_char
                }) as *mut ::core::ffi::c_char;
            }
            is_progress = strequal(kind, b"progress\0".as_ptr() as *const ::core::ffi::c_char);
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
                    b"Conflict: title/source/status/percent/data not allowed with kind='%s'\0"
                        .as_ptr() as *const ::core::ffi::c_char,
                    kind,
                );
            } else if !(!is_progress
                || strequal(
                    (*opts).status.data,
                    b"success\0".as_ptr() as *const ::core::ffi::c_char,
                ) as ::core::ffi::c_int
                    != 0
                || strequal(
                    (*opts).status.data,
                    b"failed\0".as_ptr() as *const ::core::ffi::c_char,
                ) as ::core::ffi::c_int
                    != 0
                || strequal(
                    (*opts).status.data,
                    b"running\0".as_ptr() as *const ::core::ffi::c_char,
                ) as ::core::ffi::c_int
                    != 0
                || strequal(
                    (*opts).status.data,
                    b"cancel\0".as_ptr() as *const ::core::ffi::c_char,
                ) as ::core::ffi::c_int
                    != 0)
            {
                api_err_exp(
                    err,
                    b"status\0".as_ptr() as *const ::core::ffi::c_char,
                    b"success|failed|running|cancel\0".as_ptr() as *const ::core::ffi::c_char,
                    (*opts).status.data,
                );
            } else if !(!is_progress
                || (*opts).percent >= 0 as Integer && (*opts).percent <= 100 as Integer)
            {
                api_err_invalid(
                    err,
                    b"percent\0".as_ptr() as *const ::core::ffi::c_char,
                    b"out of range\0".as_ptr() as *const ::core::ffi::c_char,
                    0 as int64_t,
                    false_0 != 0,
                );
            } else if !(!is_progress || (*opts).source.size != 0 as size_t) {
                api_err_required(err, b"opts.source\0".as_ptr() as *const ::core::ffi::c_char);
            } else if !((*opts).id.type_0 as ::core::ffi::c_uint
                != kObjectTypeInteger as ::core::ffi::c_int as ::core::ffi::c_uint
                || msg_id_exists((*opts).id.data.integer as int64_t) as ::core::ffi::c_int != 0)
            {
                api_set_error(
                    err,
                    kErrorTypeValidation,
                    b"Invalid 'id': %ld\0".as_ptr() as *const ::core::ffi::c_char,
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
                    history as bool,
                    (*opts).err as bool,
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
