//! `:restart`, `:detach` and `:connect` — the commands that hand the
//! session to another process or take it back.
#![deny(unsafe_op_in_unsafe_fn)]

use crate::semsg_c;
use core::ffi::{c_char, c_int, c_void};
use core::ptr;

use crate::api::private::helpers::{api_clear_error, cstr_as_string};
use crate::api::ui::{remote_ui_connect, remote_ui_disconnect};
use crate::api::vim::nvim__chan_set_detach;
use crate::api::vimscript::nvim_command;
use crate::channel::{channel_close, channel_job_start, channel_proc, find_channel};
use crate::eval::typval::{NumBuf, kCallbackNone, tv_list_len};
use crate::eval::vars::{get_vim_var_list, get_vim_var_str, set_vim_var_string};
use crate::event::proc::{proc_stop, proc_wait};
use crate::ex_docmd::{GA_EMPTY_INIT_VALUE, cmdmod_has, kChannelPartAll};
use crate::log::{LOGLVL_INF, logmsg_c};
use crate::main::{current_ui, e_invchan, exiting, getout};
use crate::memory::{arena_mem_free, strequal, xcalloc, xfree, xmemdupz, xstrdup};
use crate::message::emsg;
use crate::msgpack_rpc::channel::rpc_send_call;
use crate::msgpack_rpc::server::{server_start, server_stop};
use crate::os::cshim::strstr;
use crate::strings::concat_str;
use crate::types::channel::kChannelStdinPipe;
use crate::types::{
    ArenaMem, Array, Callback, CallbackReader, CmdModFlags, Dict, Error, KeyValuePair, NUL, Object,
    Vv, exarg_T, kErrorTypeNone, kObjectTypeBoolean, kObjectTypeDict, kObjectTypeString,
    key_value_pair, listitem_T, object_data, ptrdiff_t, size_t, uint16_t, varnumber_T,
};
use crate::ui::{ui_active, ui_call_restart, ui_flush};

/// An `Object` holding a NUL-terminated string, without copying it.
fn obj_str(s: *const c_char) -> Object {
    Object {
        type_0: kObjectTypeString,
        data: object_data {
            // SAFETY: `cstr_as_string` measures the string; it does not
            // outlive the caller's storage, which every call site keeps
            // alive across the RPC call.
            string: unsafe { cstr_as_string(s) },
        },
    }
}

/// An `Object` holding a boolean.
fn obj_bool(b: bool) -> Object {
    Object {
        type_0: kObjectTypeBoolean,
        data: object_data { boolean: b },
    }
}

/// A borrowed `Array` over `items`, full.
fn array_of(items: &mut [Object]) -> Array {
    Array {
        size: items.len() as size_t,
        capacity: items.len() as size_t,
        items: items.as_mut_ptr(),
    }
}

/// A borrowed `Dict` over `items`, full.
fn dict_of(items: &mut [KeyValuePair]) -> Dict {
    Dict {
        size: items.len() as size_t,
        capacity: items.len() as size_t,
        items: items.as_mut_ptr(),
    }
}

/// A `key = value` entry for a borrowed `Dict`.
fn entry(key: &'static core::ffi::CStr, value: Object) -> KeyValuePair {
    key_value_pair {
        // SAFETY: the key is a `'static` C string literal.
        key: unsafe { cstr_as_string(key.as_ptr()) },
        value,
    }
}

/// `:restart` — start a second Nvim, hand every UI over to it, and quit.
///
/// The new server is started as an embedded RPC job so that this one can
/// talk to it: it has to be told not to exit when the channel closes, told
/// what to run once a UI arrives, and asked for the address to send the
/// UIs to. Only then does this server try to quit — and if it *cannot*
/// (an unsaved buffer, a `+cmd` that did not quit), the new server is
/// killed again and nothing has changed.
pub(crate) unsafe fn ex_restart(eap: *mut exarg_T) {
    let mut numbuf = NumBuf::new();
    let mut numbuf2 = NumBuf::new();
    unsafe {
        let mut err = Error {
            type_0: kErrorTypeNone,
            msg: ptr::null_mut(),
        };
        let no_ui = ui_active() == 0;
        let exepath = get_vim_var_str(Vv::Progpath);
        let argv_list = get_vim_var_list(Vv::Argv);
        let argc = tv_list_len(argv_list);

        // Three more than `v:argv`: `--embed`, `--headless`, and the null
        // terminator.
        let argv = xcalloc(argc as size_t + 3, size_of::<*mut c_char>()) as *mut *mut c_char;
        let mut i: size_t = 0;
        let mut listen_arg: *const c_char = ptr::null();

        let mut li: *const listitem_T = (*argv_list).lv_first;
        while !li.is_null() {
            let arg = numbuf.string(&raw const (*li).li_tv);
            // `-- [files…]` is dropped: it is almost never wanted, and
            // `:mksession` is the way to carry a session over.
            if i > 0 && strequal(arg, c"--".as_ptr()) {
                break;
            }
            // `-s <scriptfile>` is dropped, script file and all.
            if i > 0 && strequal(arg, c"-s".as_ptr()) {
                li = (*li).li_next;
                if li.is_null() {
                    break;
                }
                li = (*li).li_next;
                continue;
            }
            // The address after `--listen` is in use by *this* server, so
            // it has to be released before the new one can take it.
            if i > 0 && strequal(arg, c"--listen".as_ptr()) {
                let next_li = (*li).li_next;
                if !next_li.is_null() {
                    let addr = numbuf2.string(&raw const (*next_li).li_tv);
                    if !strstr(addr, c":".as_ptr()).is_null()
                        || !strstr(addr, c"/".as_ptr()).is_null()
                        || !strstr(addr, c"\\".as_ptr()).is_null()
                    {
                        listen_arg = addr;
                    }
                }
            }
            // `--embed`, `--headless` and `-` are replaced by exactly one
            // `--embed` (plus `--headless` when there is no UI), inserted
            // right after argv[0].
            if i == 0
                || !strequal(arg, c"--embed".as_ptr())
                    && !strequal(arg, c"--headless".as_ptr())
                    && !strequal(arg, c"-".as_ptr())
            {
                *argv.add(i as usize) = xstrdup(arg);
                i += 1;
                if i == 1 {
                    *argv.add(i as usize) = xstrdup(c"--embed".as_ptr());
                    i += 1;
                    // Without `--headless`, an embedded server waits for a
                    // UI to attach.
                    if no_ui {
                        *argv.add(i as usize) = xstrdup(c"--headless".as_ptr());
                        i += 1;
                    }
                }
            }
            li = (*li).li_next;
        }

        let server_stopped = !listen_arg.is_null() && server_stop(listen_arg, true);

        let mut on_err = blank_reader();
        // The stderr fd is inherited, so forwarding still works after this
        // server exits.
        on_err.fwd_err = true;
        let mut exit_status: varnumber_T = 0;
        let channel = channel_job_start(
            argv,
            exepath,
            blank_reader(),
            on_err,
            blank_callback(),
            false,
            true,
            true,
            true, // detach: the new server outlives this one
            kChannelStdinPipe,
            ptr::null(),
            0 as uint16_t,
            0 as uint16_t,
            ptr::null_mut(),
            &raw mut exit_status,
        );

        'fail_1: {
            if channel.is_null() {
                emsg(c"cannot create a channel job".as_ptr());
                break 'fail_1;
            }
            let id = (*channel).id;
            let mut result_mem: ArenaMem = ptr::null_mut();

            'fail_2: {
                // Stop the new server exiting when this channel closes.
                let mut detach_items = [obj_bool(true)];
                rpc_send_call(
                    id,
                    c"nvim__chan_set_detach".as_ptr(),
                    array_of(&mut detach_items),
                    &raw mut result_mem,
                    &raw mut err,
                );
                if err.type_0 as c_int != kErrorTypeNone as c_int {
                    break 'fail_2;
                }
                arena_mem_free(result_mem);
                result_mem = ptr::null_mut();

                // `:restart {cmd}` runs {cmd} over there, once a UI has
                // arrived.
                if *(*eap).arg as c_int != NUL {
                    let mut opt_items = [
                        entry(c"once", obj_bool(true)),
                        entry(c"nested", obj_bool(true)),
                        entry(c"command", obj_str((*eap).arg)),
                    ];
                    let mut autocmd_items = [
                        obj_str(c"UIEnter".as_ptr()),
                        Object {
                            type_0: kObjectTypeDict,
                            data: object_data {
                                dict: dict_of(&mut opt_items),
                            },
                        },
                    ];
                    rpc_send_call(
                        id,
                        c"nvim_create_autocmd".as_ptr(),
                        array_of(&mut autocmd_items),
                        &raw mut result_mem,
                        &raw mut err,
                    );
                    if err.type_0 as c_int != kErrorTypeNone as c_int {
                        break 'fail_2;
                    }
                    arena_mem_free(result_mem);
                    result_mem = ptr::null_mut();
                }

                // Where the UIs are to reconnect.
                let mut name_items = [obj_str(c"servername".as_ptr())];
                let result = rpc_send_call(
                    id,
                    c"nvim_get_vvar".as_ptr(),
                    array_of(&mut name_items),
                    &raw mut result_mem,
                    &raw mut err,
                );
                if err.type_0 as c_int != kErrorTypeNone as c_int {
                    break 'fail_2;
                }
                if result.type_0 as c_int != kObjectTypeString as c_int
                    || result.data.string.is_empty()
                {
                    emsg(c"restart failed: could not get listen address from new server".as_ptr());
                    break 'fail_2;
                }
                // Copied out before the arena it lives in is freed.
                let listen_addr = xmemdupz(
                    result.data.string.data() as *const c_void,
                    result.data.string.len(),
                ) as *mut c_char;
                arena_mem_free(result_mem);
                result_mem = ptr::null_mut();

                ui_call_restart(cstr_as_string(listen_addr));
                ui_flush();
                xfree(listen_addr as *mut c_void);

                set_vim_var_string(Vv::Exitreason, c"restart".as_ptr(), 7 as ptrdiff_t);

                let mut quit_cmd = if (*eap).do_ecmd_cmd.is_null() {
                    c"qall".as_ptr() as *mut c_char
                } else {
                    (*eap).do_ecmd_cmd
                };
                let mut quit_cmd_copy: *mut c_char = ptr::null_mut();
                if cmdmod_has(CmdModFlags::CONFIRM) {
                    quit_cmd_copy = concat_str(c"confirm ".as_ptr(), quit_cmd);
                    quit_cmd = quit_cmd_copy;
                }
                if let Err(e) = nvim_command(cstr_as_string(quit_cmd)) {
                    err = e;
                }
                xfree(quit_cmd_copy as *mut c_void);

                if err.type_0 as c_int != kErrorTypeNone as c_int {
                    emsg(err.msg);
                    api_clear_error(&raw mut err);
                } else if !exiting.get() {
                    emsg(c"restart failed: +cmd did not quit the server".as_ptr());
                }
            }

            // Reached both on success — where `exiting` is set and this is
            // the last thing that runs — and on every failure.
            set_vim_var_string(Vv::Exitreason, ptr::null(), -1 as ptrdiff_t);
            if err.type_0 as c_int != kErrorTypeNone as c_int {
                emsg(err.msg);
                api_clear_error(&raw mut err);
            }
            arena_mem_free(result_mem);
            result_mem = ptr::null_mut();

            // Close the new server's stderr before killing it, or its dying
            // words land on this UI.
            let mut chanclose_items = [obj_str(c"chanclose(v:stderr)".as_ptr())];
            rpc_send_call(
                id,
                c"nvim_eval".as_ptr(),
                array_of(&mut chanclose_items),
                &raw mut result_mem,
                &raw mut err,
            );
            api_clear_error(&raw mut err);
            arena_mem_free(result_mem);

            proc_stop(channel_proc(channel));
            if proc_wait(channel_proc(channel), -1, ptr::null_mut()) < 0 {
                emsg(c"killing new nvim server failed".as_ptr());
            }
        }

        // The address was released for a server that is not going to use it.
        if server_stopped && server_start(listen_arg) != 0 {
            semsg_c!(c"couldn't resume listening on %s".as_ptr(), listen_arg);
        }
    }
}

/// A `CallbackReader` that reads nothing.
fn blank_reader() -> CallbackReader {
    CallbackReader {
        cb: blank_callback(),
        self_0: ptr::null_mut(),
        buffer: GA_EMPTY_INIT_VALUE,
        eof: false,
        buffered: false,
        fwd_err: false,
        type_0: ptr::null(),
    }
}

/// A `Callback` that calls nothing.
fn blank_callback() -> Callback {
    Callback {
        data: crate::types::Callback_data {
            funcref: ptr::null_mut(),
        },
        type_0: kCallbackNone,
    }
}

/// `:detach` — let the UI go, and keep running headless.
///
/// Called with a null `eap` by `:connect`, which has already attached
/// somewhere else.
pub(crate) unsafe fn ex_detach(eap: *mut exarg_T) {
    unsafe {
        if !eap.is_null() && (*eap).forceit != 0 {
            emsg(c"bang (!) not supported yet".as_ptr());
            return;
        }
        if current_ui.get() == 0 {
            emsg(c"UI not attached".as_ptr());
            return;
        }
        let chan = find_channel(current_ui.get());
        if chan.is_null() {
            emsg(&raw const e_invchan as *const c_char);
            return;
        }

        // Tell the UI's channel not to take the server down with it. A
        // failure here is not worth reporting, but its message is still ours
        // to free.
        if let Err(mut detach_err) = nvim__chan_set_detach((*chan).id, true) {
            api_clear_error(&raw mut detach_err);
        }

        let mut err2 = Error {
            type_0: kErrorTypeNone,
            msg: ptr::null_mut(),
        };
        remote_ui_disconnect((*chan).id, &raw mut err2, true);
        if err2.type_0 as c_int != kErrorTypeNone as c_int {
            emsg(err2.msg);
            api_clear_error(&raw mut err2);
            return;
        }

        let mut close_err: *const c_char = ptr::null();
        if !channel_close((*chan).id, kChannelPartAll, &raw mut close_err) && !close_err.is_null() {
            emsg(close_err);
            return;
        }
        logmsg_c!(
            LOGLVL_INF,
            ptr::null(),
            c"ex_detach".as_ptr(),
            6019,
            true,
            c"detach current_ui=%ld".as_ptr(),
            (*chan).id,
        );
    }
}

/// `:connect` — attach this session's UI to another server, then detach
/// from here.
///
/// `:connect!` also *exits* when this was the only UI, so that the session
/// really moves rather than being left running.
pub(crate) unsafe fn ex_connect(eap: *mut exarg_T) {
    unsafe {
        let stop_server = (*eap).forceit != 0 && ui_active() == 1;
        let mut err = Error {
            type_0: kErrorTypeNone,
            msg: ptr::null_mut(),
        };
        remote_ui_connect(current_ui.get(), (*eap).arg, &raw mut err);
        if err.type_0 as c_int != kErrorTypeNone as c_int {
            emsg(err.msg);
            api_clear_error(&raw mut err);
            return;
        }
        ex_detach(ptr::null_mut());
        if stop_server {
            exiting.set(true);
            getout(0);
        }
    }
}
