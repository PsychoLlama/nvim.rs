//! Delivering a channel's output to Vimscript.
//!
//! Bytes arrive on the event loop and are accumulated per reader; the
//! `on_stdout`/`on_stderr`/`on_exit` callbacks run later, from the channel's
//! own queue, so that a callback which writes back to its channel cannot
//! recurse into the read path.

use crate::semsg_c;
use core::ffi::{c_char, c_int, c_void};
use core::{mem, ptr};

use crate::src::nvim::eval::callback_call;
use crate::src::nvim::eval::encode::encode_list_write;
use crate::src::nvim::eval::typval::{
    callback_free, tv_clear, tv_dict_add_list, tv_dict_find, tv_list_alloc, tv_list_append_string,
    tv_list_unref,
};
use crate::src::nvim::eval::typval::{kCallbackNone, tv_list_ref};
use crate::src::nvim::event::r#loop::one_arg_event;
use crate::src::nvim::event::multiqueue::multiqueue_put_event;
use crate::src::nvim::garray::{ga_clear, ga_concat_len, ga_init};
use crate::src::nvim::main::e_streamkey;
use crate::src::nvim::os::libc::{gettext, strlen};
use crate::src::nvim::terminal::terminal_receive;
use crate::src::nvim::types::{
    CallbackReader, Channel, RStream, VAR_LIST, VAR_NUMBER, VAR_STRING, VAR_UNKNOWN, VAR_UNLOCKED,
    kListLenMayKnow, list_T, size_t, typval_T, typval_vval_union, varnumber_T,
};

use super::{channel_decref, channel_incref};

/// Starts buffering a reader's output under `type_0`, which names it in the
/// callback and in the `self` dict.
pub unsafe fn callback_reader_start(reader: *mut CallbackReader, type_0: *const c_char) {
    ga_init(
        &raw mut (*reader).buffer,
        mem::size_of::<*mut c_char>() as c_int,
        32,
    );
    (*reader).type_0 = type_0;
}

pub unsafe fn callback_reader_free(reader: *mut CallbackReader) {
    callback_free(&raw mut (*reader).cb);
    ga_clear(&raw mut (*reader).buffer);
}

/// Whether a reader has anywhere to deliver to.
pub(super) unsafe fn callback_reader_set(reader: CallbackReader) -> bool {
    reader.cb.type_0 != kCallbackNone || !reader.self_0.is_null()
}

pub unsafe extern "C" fn on_channel_data(
    stream: *mut RStream,
    buf: *const c_char,
    count: size_t,
    data: *mut c_void,
    eof: bool,
) -> size_t {
    let chan = data as *mut Channel;
    on_channel_output(stream, chan, buf, count, eof, &raw mut (*chan).on_data)
}

pub unsafe extern "C" fn on_job_stderr(
    stream: *mut RStream,
    buf: *const c_char,
    count: size_t,
    data: *mut c_void,
    eof: bool,
) -> size_t {
    let chan = data as *mut Channel;
    on_channel_output(stream, chan, buf, count, eof, &raw mut (*chan).on_stderr)
}

/// Accepts everything the stream offers: a terminal gets it immediately, a
/// reader accumulates it for the callback that runs on the channel's queue.
unsafe fn on_channel_output(
    _stream: *mut RStream,
    chan: *mut Channel,
    buf: *const c_char,
    count: size_t,
    eof: bool,
    reader: *mut CallbackReader,
) -> size_t {
    if !(*chan).term.is_null() {
        terminal_receive((*chan).term, buf, count);
    }
    if eof {
        (*reader).eof = true;
    }
    if callback_reader_set(*reader) {
        ga_concat_len(&raw mut (*reader).buffer, buf, count);
        schedule_channel_event(chan);
    }
    count
}

/// Asks for the channel's callbacks to run.
///
/// At most one such event is outstanding, and a request made while they are
/// already running is deferred to the end of that run instead — otherwise a
/// callback that writes to its own channel would recurse.
pub(super) unsafe fn schedule_channel_event(chan: *mut Channel) {
    if (*chan).callback_scheduled {
        return;
    }
    if !(*chan).callback_busy {
        multiqueue_put_event(
            (*chan).events,
            one_arg_event(Some(on_channel_event), chan as *mut c_void),
        );
        channel_incref(chan);
    }
    (*chan).callback_scheduled = true;
}

unsafe extern "C" fn on_channel_event(args: *mut *mut c_void) {
    let chan = *args as *mut Channel;
    (*chan).callback_busy = true;
    (*chan).callback_scheduled = false;

    // Latched before the reader callbacks run: one of them may start another
    // job on this channel and reset it.
    let exit_status = (*chan).exit_status;
    channel_reader_callbacks(chan, &raw mut (*chan).on_data);
    channel_reader_callbacks(chan, &raw mut (*chan).on_stderr);
    if exit_status > -1 {
        channel_callback_call(chan, ptr::null_mut());
        (*chan).exit_status = -1;
    }

    (*chan).callback_busy = false;
    if (*chan).callback_scheduled {
        multiqueue_put_event(
            (*chan).events,
            one_arg_event(Some(on_channel_event), chan as *mut c_void),
        );
        channel_incref(chan);
    }
    channel_decref(chan);
}

/// Delivers whatever a reader has accumulated.
///
/// A buffered reader delivers once, at EOF — into the `self` dict if it has
/// one, otherwise through the callback. An unbuffered reader delivers as it
/// goes, and again with an empty list at EOF.
pub unsafe fn channel_reader_callbacks(chan: *mut Channel, reader: *mut CallbackReader) {
    if (*reader).buffered {
        if !(*reader).eof {
            return;
        }
        if (*reader).self_0.is_null() {
            channel_callback_call(chan, reader);
        } else if tv_dict_find((*reader).self_0, (*reader).type_0, -1).is_null() {
            let data = buffer_to_tv_list(
                (*reader).buffer.ga_data as *const c_char,
                (*reader).buffer.ga_len as size_t,
            );
            tv_dict_add_list(
                (*reader).self_0,
                (*reader).type_0,
                strlen((*reader).type_0),
                data,
            );
        } else {
            semsg_c!(
                gettext(e_streamkey.ptr() as *const c_char),
                (*reader).type_0,
                (*chan).id,
            );
        }
        (*reader).eof = false;
    } else {
        let is_eof = (*reader).eof;
        if (*reader).buffer.ga_len > 0 {
            channel_callback_call(chan, reader);
        }
        if is_eof {
            channel_callback_call(chan, reader);
            (*reader).eof = false;
        }
    }
}

/// Calls `on_stdout`/`on_stderr` for `reader`, or `on_exit` when it is null.
unsafe fn channel_callback_call(chan: *mut Channel, reader: *mut CallbackReader) {
    let mut argv: [typval_T; 4] = [typval_T {
        v_type: VAR_UNKNOWN as _,
        v_lock: VAR_UNLOCKED as _,
        vval: typval_vval_union { v_number: 0 },
    }; 4];
    argv[0].v_type = VAR_NUMBER as _;
    argv[0].vval.v_number = (*chan).id as varnumber_T;

    let cb = if reader.is_null() {
        argv[1].v_type = VAR_NUMBER as _;
        argv[1].vval.v_number = (*chan).exit_status as varnumber_T;
        argv[2].vval.v_string = c"exit".as_ptr() as *mut c_char;
        &raw mut (*chan).on_exit
    } else {
        argv[1].v_type = VAR_LIST as _;
        argv[1].vval.v_list = buffer_to_tv_list(
            (*reader).buffer.ga_data as *const c_char,
            (*reader).buffer.ga_len as size_t,
        );
        tv_list_ref(argv[1].vval.v_list);
        ga_clear(&raw mut (*reader).buffer);
        argv[2].vval.v_string = (*reader).type_0 as *mut c_char;
        &raw mut (*reader).cb
    };
    argv[2].v_type = VAR_STRING as _;

    let mut rettv: typval_T = typval_T {
        v_type: VAR_UNKNOWN as _,
        v_lock: VAR_UNLOCKED as _,
        vval: typval_vval_union { v_number: 0 },
    };
    callback_call(cb, 3, argv.as_mut_ptr(), &raw mut rettv);
    tv_clear(&raw mut rettv);
    if !reader.is_null() {
        tv_list_unref(argv[1].vval.v_list);
    }
}

/// Splits accumulated bytes into the list of lines a callback is handed.
///
/// The list always starts with one empty string, so a chunk that does not end
/// in a newline leaves a partial last line the next chunk continues.
unsafe fn buffer_to_tv_list(buf: *const c_char, len: size_t) -> *mut list_T {
    let l = tv_list_alloc(kListLenMayKnow as isize);
    tv_list_append_string(l, c"".as_ptr(), 0);
    if len > 0 {
        encode_list_write(l as *mut c_void, buf, len);
    }
    l
}
