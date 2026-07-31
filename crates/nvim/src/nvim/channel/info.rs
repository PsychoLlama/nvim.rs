//! What the editor tells the user about its channels.
//!
//! `nvim_get_chan_info()` and `nvim_list_chans()` read the dicts built here,
//! and the same dict is what the `ChanOpen`/`ChanInfo` autocommands publish in
//! `v:event`.

use core::ffi::{CStr, c_char, c_int, c_void};
use core::{mem, ptr};

use crate::src::nvim::api::private::converter::object_to_vim;
use crate::src::nvim::api::private::helpers::{
    arena_array, arena_dict, arena_string, cstr_as_string,
};
use crate::src::nvim::autocmd::{EVENT_CHANINFO, EVENT_CHANOPEN, apply_autocmds, has_event};
use crate::src::nvim::eval::encode::encode_tv2json;
use crate::src::nvim::eval::typval::{tv_dict_add_dict, tv_dict_set_keys_readonly};
use crate::src::nvim::eval::{eval_fmt_source_name_line, get_v_event, restore_v_event};
use crate::src::nvim::event::r#loop::one_arg_event;
use crate::src::nvim::event::multiqueue::multiqueue_put_event;
use crate::src::nvim::event::proc::proc_is_stopped;
use crate::src::nvim::log::logmsg;
use crate::src::nvim::main::{IObuff, channels, curbuf, main_loop};
use crate::src::nvim::memory::{ARENA_EMPTY, arena_alloc, arena_finish, arena_mem_free, xfree};
use crate::src::nvim::os::libc::qsort;
use crate::src::nvim::os::pty_proc_unix::pty_proc_tty_name;
use crate::src::nvim::terminal::terminal_buf;
use crate::src::nvim::types::{
    Arena, Array, Channel, Dict, Integer, Object, event_T, hashtab_T, int64_t, kObjectTypeArray,
    kObjectTypeBoolean, kObjectTypeBuffer, kObjectTypeDict, kObjectTypeInteger, kObjectTypeString,
    key_value_pair, object_data, save_v_event_T, size_t, typval_T, typval_vval_union, uint64_t,
};

use super::known::*;
use super::{channel_decref, channel_incref, channel_proc, channel_pty, empty_dict, find_channel};

/// Announces a newly opened channel: `ChanOpen`, plus a log line naming
/// whatever opened it.
pub unsafe fn channel_create_event(chan: *mut Channel, ext_source: *const c_char) {
    let source = if ext_source.is_null() {
        eval_fmt_source_name_line(IObuff.ptr() as *mut c_char, IOSIZE);
        IObuff.ptr() as *const c_char
    } else {
        ext_source
    };
    assert!((*chan).id <= i64::MAX as uint64_t);

    let mut arena: Arena = ARENA_EMPTY;
    let info = channel_info((*chan).id, &raw mut arena);
    let mut tv = unknown_tv();
    object_to_vim(
        Object {
            type_0: kObjectTypeDict,
            data: object_data { dict: info },
        },
        &raw mut tv,
        ptr::null_mut(),
    );
    assert!(tv.v_type as c_int == VAR_DICT);
    let str = encode_tv2json(&raw mut tv, ptr::null_mut());
    logmsg(
        LOGLVL_INF,
        ptr::null(),
        c"channel_create_event".as_ptr(),
        258,
        true,
        c"new channel %lu (%s) : %s".as_ptr(),
        (*chan).id,
        source,
        str,
    );
    xfree(str as *mut c_void);
    arena_mem_free(arena_finish(&raw mut arena));
    channel_info_changed(chan, true);
}

/// Queues the `ChanOpen`/`ChanInfo` autocommand, if anything is listening.
///
/// It is queued rather than fired here because the caller is often inside the
/// channel's own setup, where running arbitrary Vimscript would be reentrant.
pub unsafe fn channel_info_changed(chan: *mut Channel, new_chan: bool) {
    let event = if new_chan {
        EVENT_CHANOPEN
    } else {
        EVENT_CHANINFO
    } as event_T;
    if !has_event(event) {
        return;
    }
    channel_incref(chan);
    let mut ev = one_arg_event(Some(set_info_event), chan as *mut c_void);
    ev.argv[1] = ptr::with_exposed_provenance_mut::<c_void>(event as usize);
    multiqueue_put_event((*main_loop.ptr()).events, ev);
}

unsafe extern "C" fn set_info_event(argv: *mut *mut c_void) {
    let chan = *argv as *mut Channel;
    let event = (*argv.add(1)).expose_provenance() as event_T;

    let mut save_v_event = save_v_event_T {
        sve_did_save: false,
        sve_hashtab: mem::zeroed::<hashtab_T>(),
    };
    let dict = get_v_event(&raw mut save_v_event);
    let mut arena: Arena = ARENA_EMPTY;
    let info = channel_info((*chan).id, &raw mut arena);
    let mut retval = unknown_tv();
    object_to_vim(
        Object {
            type_0: kObjectTypeDict,
            data: object_data { dict: info },
        },
        &raw mut retval,
        ptr::null_mut(),
    );
    assert!(retval.v_type as c_int == VAR_DICT);
    tv_dict_add_dict(dict, c"info".as_ptr(), 4, retval.vval.v_dict);
    tv_dict_set_keys_readonly(dict);
    apply_autocmds(event, ptr::null_mut(), ptr::null_mut(), true, curbuf.get());
    restore_v_event(dict, &raw mut save_v_event);
    arena_mem_free(arena_finish(&raw mut arena));
    channel_decref(chan);
}

/// A fresh `typval_T` of no type, which is what every consumer here starts
/// from before something writes into it.
pub(super) fn unknown_tv() -> typval_T {
    typval_T {
        v_type: VAR_UNKNOWN as _,
        v_lock: VAR_UNLOCKED as _,
        vval: typval_vval_union { v_number: 0 },
    }
}

/// Whether `id` names a job whose process is still running.
pub unsafe fn channel_job_running(id: uint64_t) -> bool {
    let chan = find_channel(id);
    !chan.is_null()
        && (*chan).streamtype == kChannelStreamProc
        && !proc_is_stopped(&*channel_proc(chan))
}

/// What `nvim_get_chan_info()` reports. An unknown id answers with an empty
/// dict rather than an error.
pub unsafe fn channel_info(id: uint64_t, arena: *mut Arena) -> Dict {
    let chan = find_channel(id);
    if chan.is_null() {
        return empty_dict();
    }
    // id, stream, mode, and up to six more from the branches below.
    let mut info = arena_dict(arena, 9);
    let mut push = |key: &CStr, value: Object| {
        *info.items.add(info.size) = key_value_pair {
            key: cstr_as_string(key.as_ptr()),
            value,
        };
        info.size += 1;
    };
    push(
        c"id",
        Object {
            type_0: kObjectTypeInteger,
            data: object_data {
                integer: (*chan).id as Integer,
            },
        },
    );

    let stream_desc = match (*chan).streamtype {
        kChannelStreamProc => {
            let proc = channel_proc(chan);
            if (*proc).type_0 as c_int == kProcTypePty {
                let name = pty_proc_tty_name(channel_pty(chan));
                push(
                    c"pty",
                    Object {
                        type_0: kObjectTypeString,
                        data: object_data {
                            string: arena_string(arena, cstr_as_string(name)),
                        },
                    },
                );
            }
            push(
                c"argv",
                Object {
                    type_0: kObjectTypeArray,
                    data: object_data {
                        array: argv_array((*proc).argv, arena),
                    },
                },
            );
            c"job"
        }
        kChannelStreamStdio => c"stdio",
        kChannelStreamStderr => c"stderr",
        kChannelStreamInternal => {
            push(
                c"internal",
                Object {
                    type_0: kObjectTypeBoolean,
                    data: object_data { boolean: true },
                },
            );
            // An internal channel reports itself as a socket, because that is
            // what it stands in for.
            c"socket"
        }
        _ => c"socket",
    };
    push(
        c"stream",
        Object {
            type_0: kObjectTypeString,
            data: object_data {
                string: cstr_as_string(stream_desc.as_ptr()),
            },
        },
    );

    let mode_desc = if (*chan).is_rpc {
        push(
            c"client",
            Object {
                type_0: kObjectTypeDict,
                data: object_data {
                    dict: (*chan).rpc.info,
                },
            },
        );
        c"rpc"
    } else if (*chan).term.is_null() {
        c"bytes"
    } else {
        let buf = Object {
            type_0: kObjectTypeBuffer,
            data: object_data {
                integer: terminal_buf((*chan).term) as Integer,
            },
        };
        // `buf` is the documented key; `buffer` is kept for older plugins.
        push(c"buf", buf);
        push(c"buffer", buf);
        push(
            c"exitcode",
            Object {
                type_0: kObjectTypeInteger,
                data: object_data {
                    integer: (*chan).exit_status as Integer,
                },
            },
        );
        c"terminal"
    };
    push(
        c"mode",
        Object {
            type_0: kObjectTypeString,
            data: object_data {
                string: cstr_as_string(mode_desc.as_ptr()),
            },
        },
    );
    info
}

/// The child's command line, as an array of strings borrowed from it.
unsafe fn argv_array(args: *mut *mut c_char, arena: *mut Arena) -> Array {
    if args.is_null() {
        return Array {
            size: 0,
            capacity: 0,
            items: ptr::null_mut(),
        };
    }
    let mut n = 0;
    while !(*args.add(n)).is_null() {
        n += 1;
    }
    let mut argv = arena_array(arena, n);
    for i in 0..n {
        *argv.items.add(i) = Object {
            type_0: kObjectTypeString,
            data: object_data {
                string: cstr_as_string(*args.add(i)),
            },
        };
    }
    argv.size = n;
    argv
}

/// Every channel's info, ordered by id.
pub unsafe fn channel_all_info(arena: *mut Arena) -> Array {
    let map = channels.ptr();
    let n = (*map).set.h.n_keys as usize;
    let ids = arena_alloc(arena, mem::size_of::<int64_t>().wrapping_mul(n), true) as *mut int64_t;
    for i in 0..n {
        *ids.add(i) = *(*map).set.keys.add(i) as int64_t;
    }
    // The map iterates in hash order; the API contract is ascending id.
    qsort(
        ids as *mut c_void,
        n,
        mem::size_of::<int64_t>(),
        Some(int64_t_cmp),
    );
    let mut ret = arena_array(arena, n);
    for i in 0..n {
        *ret.items.add(i) = Object {
            type_0: kObjectTypeDict,
            data: object_data {
                dict: channel_info(*ids.add(i) as uint64_t, arena),
            },
        };
    }
    ret.size = n;
    ret
}

unsafe extern "C" fn int64_t_cmp(pa: *const c_void, pb: *const c_void) -> c_int {
    let a = *(pa as *const int64_t);
    let b = *(pb as *const int64_t);
    a.cmp(&b) as c_int
}

/// The capacity of `IObuff`.
const IOSIZE: size_t = 1025;
