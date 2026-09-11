//! What the editor tells the user about its channels.
//!
//! `nvim_get_chan_info()` and `nvim_list_chans()` read the dicts built here,
//! and the same dict is what the `ChanOpen`/`ChanInfo` autocommands publish in
//! `v:event`.

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

use crate::types::AutoEvent;
use crate::types::String_0;
use crate::winlayer::Buf;
use core::ffi::{CStr, c_char, c_void};
use core::ptr;
use std::ffi::CString;

use crate::api::private::helpers::cstr_to_string;
use crate::autocmd::{apply_autocmds, has_event};
use crate::channel::channels;
use crate::eval::encode::encode_tv2json;
use crate::eval::typval::{DictRef, tv_dict_add_dict, tv_dict_set_keys_readonly};
use crate::eval::{eval_fmt_source_name_line, get_v_event, restore_v_event};
use crate::event::r#loop::one_arg_event;
use crate::event::multiqueue::multiqueue_put_event;
use crate::event::proc::proc_is_stopped;
use crate::log::{LOGLVL_INF, logmsg};
use crate::memory::xfree;
use crate::message_fmt::c_str;
use crate::os::pty_proc_unix::pty_proc_tty_name;
use crate::registry::SlotTable;
use crate::terminal::terminal_buf;
use crate::types::{
    ApiDict, Array, Channel, IOSIZE, Integer, Object, SaveVEvent, TypVal, VAR_DICT, uint64_t,
};

use super::known::*;
use super::{
    channel_decref, channel_incref, channel_proc, channel_pty, empty_dict, find_channel,
    main_loop_events,
};

/// A string object holding a copy of a `'static` C literal.
fn literal_obj(text: &'static CStr) -> Object {
    Object::string(String_0::from_cstr(text))
}

/// A fresh `TypVal` of no type, which is what every consumer here starts
/// from before something writes into it.
/// `chan`'s info dict, as the `TypVal` the Vimscript layer wants.
///
/// # Safety
/// `id` may name any channel; `arena` owns the dict's storage.
unsafe fn info_tv(id: uint64_t) -> TypVal {
    // SAFETY: `channel_info` answers a dict, which converts without ever
    // failing.
    let info = unsafe { channel_info(id) };
    let tv = TypVal::from(Object::dict(info));
    debug_assert!(tv.v_type() == VAR_DICT);
    tv
}

// ---------------------------------------------------------------------------
// Announcements
// ---------------------------------------------------------------------------

/// Announces a newly opened channel: `ChanOpen`, plus a log line naming
/// whatever opened it.
///
/// # Safety
/// `chan` is a live channel; `ext_source` is null or a C string.
pub unsafe fn channel_create_event(chan: *mut Channel, ext_source: *const c_char) {
    // The script location is copied out of `IObuff` rather than pointed at:
    // building the info dict below reenters the evaluator, which formats its
    // own messages through the same scratch buffer.
    let script_source = if ext_source.is_null() {
        Some(source_name_line())
    } else {
        None
    };
    let source = script_source
        .as_ref()
        .map_or(ext_source, |name| name.as_ptr());

    // SAFETY: the caller's live channel.
    debug_assert!(unsafe { (*chan).id } <= i64::MAX as uint64_t);
    let tv = unsafe { info_tv((*chan).id) };
    let str = unsafe { encode_tv2json(&tv, ptr::null_mut()) };
    // SAFETY: the caller's live channel, and two NUL-terminated strings --
    // the caller's `source` and the JSON just rendered.
    let (id, source, info) = unsafe { ((*chan).id, c_str(source), c_str(str)) };
    logmsg!(
        LOGLVL_INF,
        c"channel_create_event",
        258,
        "new channel {id} ({source}) : {info}"
    );
    unsafe { xfree(str.cast()) };
    unsafe { channel_info_changed(chan, true) };
}

/// `"script:line"` for whatever is executing, as an owned copy.
fn source_name_line() -> CString {
    let mut buf = [0 as c_char; IOSIZE as usize];
    // SAFETY: `eval_fmt_source_name_line` only `snprintf`s into the buffer
    // it is handed, so the result is NUL-terminated within `IOSIZE`.
    unsafe { eval_fmt_source_name_line(buf.as_mut_ptr(), IOSIZE as usize) };
    unsafe { CStr::from_ptr(buf.as_ptr()) }.to_owned()
}

/// Queues the `ChanOpen`/`ChanInfo` autocommand, if anything is listening.
///
/// It is queued rather than fired here because the caller is often inside the
/// channel's own setup, where running arbitrary Vimscript would be reentrant.
///
/// # Safety
/// `chan` is a live channel.
pub unsafe fn channel_info_changed(chan: *mut Channel, new_chan: bool) {
    let event = if new_chan {
        AutoEvent::ChanOpen
    } else {
        AutoEvent::ChanInfo
    } as AutoEvent;
    // SAFETY: the caller's live channel. The event carries the reference
    // taken here and `set_info_event` drops it.
    if !has_event(event) {
        return;
    }
    unsafe { channel_incref(chan) };
    let mut ev = one_arg_event(Some(set_info_event), chan.cast());
    ev.argv[1] = ptr::with_exposed_provenance_mut::<c_void>(event.index());
    unsafe { multiqueue_put_event(main_loop_events(), ev) };
}

/// # Safety
///
/// `argv` must point at a writable `*mut c_void` slot the caller owns for the
/// call.
unsafe extern "C" fn set_info_event(argv: *mut *mut c_void) {
    // SAFETY: the event carries the channel and the event id
    // `channel_info_changed` queued it with, plus one reference to drop.
    let chan = unsafe { *argv }.cast::<Channel>();
    let event = AutoEvent::at_row(unsafe { *argv.add(1) }.expose_provenance())
        .expect("`channel_info_changed` queued a real event");

    let mut save_v_event = SaveVEvent::default();
    let dict = unsafe { get_v_event(&raw mut save_v_event) };
    let retval = unsafe { info_tv((*chan).id) };
    // SAFETY: the answer's own dictionary; `v:event` takes a reference.
    let info = unsafe { DictRef::retained(retval.dict_or_null()) };
    let _ = unsafe { tv_dict_add_dict(dict, c"info".as_ptr(), 4, info) };
    unsafe { tv_dict_set_keys_readonly(dict) };
    let __hoisted_0 = Buf::current_or_none();
    unsafe { apply_autocmds(event, ptr::null_mut(), ptr::null_mut(), true, __hoisted_0) };
    unsafe { restore_v_event(dict, &raw mut save_v_event) };
    unsafe { channel_decref(chan) };
}

// ---------------------------------------------------------------------------
// The info dict
// ---------------------------------------------------------------------------

/// Whether `id` names a job whose process is still running.
///
/// # Safety
/// Called from the main thread with the registry live.
pub unsafe fn channel_job_running(id: uint64_t) -> bool {
    // SAFETY: the caller's promise; the channel is only read.
    let chan = find_channel(id);
    !chan.is_null()
        && unsafe { (*chan).streamtype } == kChannelStreamProc
        && !proc_is_stopped(unsafe { &*channel_proc(chan) })
}

/// What `nvim_get_chan_info()` reports. An unknown id answers with an empty
/// dict rather than an error.
///
/// # Safety
/// Called from the main thread.
pub unsafe fn channel_info(id: uint64_t) -> ApiDict {
    let chan = find_channel(id);
    if chan.is_null() {
        return empty_dict();
    }

    // id, stream, mode, and up to six more from the branches below.
    let mut info = ApiDict::with_capacity(9);
    let mut push = |key: &CStr, value: Object| info.insert(String_0::from_cstr(key), value);

    // SAFETY: `chan` is a live channel; the transport reads below are guarded
    // by its `streamtype`.
    push(c"id", Object::Integer(unsafe { (*chan).id }.cast_signed()));

    let stream_desc = match unsafe { (*chan).streamtype } {
        kChannelStreamProc => {
            let proc = unsafe { channel_proc(chan) };
            if unsafe { (*proc).type_0 }.cast_signed() == kProcTypePty {
                let name = unsafe { cstr_to_string(pty_proc_tty_name(channel_pty(chan))) };
                push(c"pty", Object::string(name));
            }
            push(c"argv", Object::array(unsafe { argv_array((*proc).argv) }));
            c"job"
        }
        kChannelStreamStdio => c"stdio",
        kChannelStreamStderr => c"stderr",
        kChannelStreamInternal => {
            push(c"internal", Object::Boolean(true));
            // An internal channel reports itself as a socket, because that
            // is what it stands in for.
            c"socket"
        }
        _ => c"socket",
    };
    push(c"stream", literal_obj(stream_desc));

    let mode_desc = if unsafe { (*chan).is_rpc } {
        push(c"client", Object::dict(unsafe { (*chan).rpc.info.clone() }));
        c"rpc"
    } else if unsafe { (*chan).term }.is_null() {
        c"bytes"
    } else {
        let handle = Integer::from(unsafe { terminal_buf((*chan).term) });
        // `buf` is the documented key; `buffer` is kept for older plugins.
        push(c"buf", Object::Buffer(handle));
        push(c"buffer", Object::Buffer(handle));
        push(
            c"exitcode",
            Object::Integer(Integer::from(unsafe { (*chan).exit_status })),
        );
        c"terminal"
    };
    push(c"mode", literal_obj(mode_desc));
    info
}

/// The child's command line, as an array of strings copied out of it.
///
/// # Safety
/// `args` is null or a NULL-terminated argument vector.
unsafe fn argv_array(args: *mut *mut c_char) -> Array {
    if args.is_null() {
        return Array::EMPTY;
    }
    // SAFETY: the caller's NULL-terminated vector.
    let mut n = 0;
    while !unsafe { *args.add(n) }.is_null() {
        n += 1;
    }
    let mut argv = Array::with_capacity(n);
    for i in 0..n {
        // SAFETY: `i` is below the count walked above.
        argv.push(Object::string(unsafe { cstr_to_string(*args.add(i)) }));
    }
    argv
}

/// Every channel's info, ordered by id.
///
/// # Safety
/// Called from the main thread.
pub unsafe fn channel_all_info() -> Array {
    // The registry iterates in registration order; the API contract is
    // ascending id.
    let mut ids = channels.with(SlotTable::snapshot_keys);
    ids.sort_unstable();
    let mut ret = Array::with_capacity(ids.len());
    for id in ids {
        // SAFETY: the id came out of the registry.
        ret.push(Object::dict(unsafe { channel_info(id) }));
    }
    ret
}
