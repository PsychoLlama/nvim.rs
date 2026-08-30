//! What the editor tells the user about its channels.
//!
//! `nvim_get_chan_info()` and `nvim_list_chans()` read the dicts built here,
//! and the same dict is what the `ChanOpen`/`ChanInfo` autocommands publish in
//! `v:event`.

#![deny(unsafe_op_in_unsafe_fn)]

use core::ffi::{CStr, c_char, c_int, c_void};
use core::{mem, ptr};
use std::ffi::CString;

use crate::api::private::converter::object_to_vim;
use crate::api::private::helpers::{arena_array, arena_dict, arena_string, cstr_as_string};
use crate::autocmd::{EVENT_CHANINFO, EVENT_CHANOPEN, apply_autocmds, has_event};
use crate::eval::encode::encode_tv2json;
use crate::eval::typval::{tv_dict_add_dict, tv_dict_set_keys_readonly};
use crate::eval::{eval_fmt_source_name_line, get_v_event, restore_v_event};
use crate::event::r#loop::one_arg_event;
use crate::event::multiqueue::multiqueue_put_event;
use crate::event::proc::proc_is_stopped;
use crate::log::{LOGLVL_INF, logmsg};
use crate::main::{channels, curbuf};
use crate::memory::{ARENA_EMPTY, arena_finish, arena_mem_free, xfree};
use crate::message_fmt::c_str;
use crate::os::pty_proc_unix::pty_proc_tty_name;
use crate::registry::SlotTable;
use crate::terminal::terminal_buf;
use crate::types::{
    Arena, Array, Channel, Dict, IOSIZE, Integer, Object, String_0, VAR_DICT, VAR_UNKNOWN, VarLock,
    event_T, hashtab_T, kObjectTypeArray, kObjectTypeBoolean, kObjectTypeBuffer, kObjectTypeDict,
    kObjectTypeInteger, kObjectTypeString, key_value_pair, object_data, save_v_event_T, typval_T,
    typval_vval_union, uint64_t,
};

use super::known::*;
use super::{
    channel_decref, channel_incref, channel_proc, channel_pty, empty_dict, find_channel,
    main_loop_events,
};

// ---------------------------------------------------------------------------
// Object constructors
// ---------------------------------------------------------------------------
//
// `Object` is a tag plus a union, and writing a union member is safe; these
// exist so the dict below reads as a list of key/value pairs rather than as
// nine five-line literals.

fn integer_obj(value: Integer) -> Object {
    Object {
        type_0: kObjectTypeInteger,
        data: object_data { integer: value },
    }
}

fn boolean_obj(value: bool) -> Object {
    Object {
        type_0: kObjectTypeBoolean,
        data: object_data { boolean: value },
    }
}

fn buffer_obj(handle: Integer) -> Object {
    Object {
        type_0: kObjectTypeBuffer,
        data: object_data { integer: handle },
    }
}

fn dict_obj(dict: Dict) -> Object {
    Object {
        type_0: kObjectTypeDict,
        data: object_data { dict },
    }
}

fn array_obj(array: Array) -> Object {
    Object {
        type_0: kObjectTypeArray,
        data: object_data { array },
    }
}

fn string_obj(string: String_0) -> Object {
    Object {
        type_0: kObjectTypeString,
        data: object_data { string },
    }
}

/// A string object borrowing a `'static` C literal.
fn literal_obj(text: &'static CStr) -> Object {
    // SAFETY: a `CStr` is NUL-terminated and, being `'static`, outlives the
    // borrowed `String_0`.
    string_obj(unsafe { cstr_as_string(text.as_ptr()) })
}

/// A fresh `typval_T` of no type, which is what every consumer here starts
/// from before something writes into it.
pub(super) fn unknown_tv() -> typval_T {
    typval_T {
        v_type: VAR_UNKNOWN as _,
        v_lock: VarLock::Unlocked,
        vval: typval_vval_union { v_number: 0 },
    }
}

/// `chan`'s info dict, as the `typval_T` the Vimscript layer wants.
///
/// # Safety
/// `id` may name any channel; `arena` owns the dict's storage.
unsafe fn info_tv(id: uint64_t, arena: *mut Arena) -> typval_T {
    let mut tv = unknown_tv();
    // SAFETY: the caller's arena; `channel_info` answers a dict, which
    // `object_to_vim` converts without ever failing.
    let info = unsafe { channel_info(id, arena) };
    unsafe { object_to_vim(dict_obj(info), &raw mut tv) };
    debug_assert!(tv.v_type == VAR_DICT);
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

    // SAFETY: the caller's live channel; the arena owns everything the dict
    // points at until it is finished below.
    debug_assert!(unsafe { (*chan).id } <= i64::MAX as uint64_t);
    let mut arena: Arena = ARENA_EMPTY;
    let mut tv = unsafe { info_tv((*chan).id, &raw mut arena) };
    let str = unsafe { encode_tv2json(&raw mut tv, ptr::null_mut()) };
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
    unsafe { arena_mem_free(arena_finish(&raw mut arena)) };
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
        EVENT_CHANOPEN
    } else {
        EVENT_CHANINFO
    } as event_T;
    // SAFETY: the caller's live channel. The event carries the reference
    // taken here and `set_info_event` drops it.
    if !has_event(event) {
        return;
    }
    unsafe { channel_incref(chan) };
    let mut ev = one_arg_event(Some(set_info_event), chan.cast());
    ev.argv[1] = ptr::with_exposed_provenance_mut::<c_void>(event as usize);
    unsafe { multiqueue_put_event(main_loop_events(), ev) };
}

unsafe extern "C" fn set_info_event(argv: *mut *mut c_void) {
    // SAFETY: the event carries the channel and the event id
    // `channel_info_changed` queued it with, plus one reference to drop.
    let chan = unsafe { *argv }.cast::<Channel>();
    let event = unsafe { *argv.add(1) }.expose_provenance() as event_T;

    let mut save_v_event = save_v_event_T {
        sve_did_save: false,
        sve_hashtab: unsafe { mem::zeroed::<hashtab_T>() },
    };
    let dict = unsafe { get_v_event(&raw mut save_v_event) };
    let mut arena: Arena = ARENA_EMPTY;
    let retval = unsafe { info_tv((*chan).id, &raw mut arena) };
    let _ = unsafe { tv_dict_add_dict(dict, c"info".as_ptr(), 4, retval.vval.v_dict) };
    unsafe { tv_dict_set_keys_readonly(dict) };
    unsafe { apply_autocmds(event, ptr::null_mut(), ptr::null_mut(), true, curbuf.get()) };
    unsafe { restore_v_event(dict, &raw mut save_v_event) };
    unsafe { arena_mem_free(arena_finish(&raw mut arena)) };
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
/// Called from the main thread; `arena` owns the answer's storage.
pub unsafe fn channel_info(id: uint64_t, arena: *mut Arena) -> Dict {
    let chan = find_channel(id);
    if chan.is_null() {
        return empty_dict();
    }

    // id, stream, mode, and up to six more from the branches below.
    let mut info = arena_dict(arena, 9);
    let mut push = |key: &CStr, value: Object| {
        // SAFETY: the arena sized `items` for nine entries and no path below
        // pushes more than that.
        unsafe {
            *info.items.add(info.size) = key_value_pair {
                key: cstr_as_string(key.as_ptr()),
                value,
            }
        };
        info.size += 1;
    };

    // SAFETY: `chan` is a live channel; the transport reads below are guarded
    // by its `streamtype`.
    push(c"id", integer_obj(unsafe { (*chan).id } as Integer));

    let stream_desc = match unsafe { (*chan).streamtype } {
        kChannelStreamProc => {
            let proc = unsafe { channel_proc(chan) };
            if unsafe { (*proc).type_0 } as c_int == kProcTypePty {
                let name = unsafe { cstr_as_string(pty_proc_tty_name(channel_pty(chan))) };
                push(c"pty", string_obj(unsafe { arena_string(arena, name) }));
            }
            push(
                c"argv",
                array_obj(unsafe { argv_array((*proc).argv, arena) }),
            );
            c"job"
        }
        kChannelStreamStdio => c"stdio",
        kChannelStreamStderr => c"stderr",
        kChannelStreamInternal => {
            push(c"internal", boolean_obj(true));
            // An internal channel reports itself as a socket, because that
            // is what it stands in for.
            c"socket"
        }
        _ => c"socket",
    };
    push(c"stream", literal_obj(stream_desc));

    let mode_desc = if unsafe { (*chan).is_rpc } {
        push(c"client", dict_obj(unsafe { (*chan).rpc.info }));
        c"rpc"
    } else if unsafe { (*chan).term }.is_null() {
        c"bytes"
    } else {
        let buf = buffer_obj(unsafe { terminal_buf((*chan).term) } as Integer);
        // `buf` is the documented key; `buffer` is kept for older plugins.
        push(c"buf", buf);
        push(c"buffer", buf);
        push(
            c"exitcode",
            integer_obj(unsafe { (*chan).exit_status } as Integer),
        );
        c"terminal"
    };
    push(c"mode", literal_obj(mode_desc));
    info
}

/// The child's command line, as an array of strings borrowed from it.
///
/// # Safety
/// `args` is null or a NULL-terminated argument vector.
unsafe fn argv_array(args: *mut *mut c_char, arena: *mut Arena) -> Array {
    if args.is_null() {
        return Array {
            size: 0,
            capacity: 0,
            items: ptr::null_mut(),
        };
    }
    // SAFETY: the caller's NULL-terminated vector, and an arena array sized
    // for exactly the arguments counted out of it.
    let mut n = 0;
    while !unsafe { *args.add(n) }.is_null() {
        n += 1;
    }
    let mut argv = arena_array(arena, n);
    for i in 0..n {
        unsafe { *argv.items.add(i) = string_obj(cstr_as_string(*args.add(i))) };
    }
    argv.size = n;
    argv
}

/// Every channel's info, ordered by id.
///
/// # Safety
/// Called from the main thread; `arena` owns the answer's storage.
pub unsafe fn channel_all_info(arena: *mut Arena) -> Array {
    // The registry iterates in registration order; the API contract is
    // ascending id.
    let mut ids = channels.with(SlotTable::snapshot_keys);
    ids.sort_unstable();
    // SAFETY: the arena array is sized for exactly as many entries as there
    // are ids, and the arena is a bump allocator, so building the dicts
    // leaves the array where it is.
    let mut ret = arena_array(arena, ids.len());
    for (i, id) in ids.iter().enumerate() {
        unsafe { *ret.items.add(i) = dict_obj(channel_info(*id, arena)) };
    }
    ret.size = ids.len();
    ret
}
