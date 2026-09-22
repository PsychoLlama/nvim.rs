//! `:restart`, `:detach` and `:connect` — the commands that hand the
//! session to another process or take it back.
#![deny(unsafe_op_in_unsafe_fn)]
#![allow(unsafe_code)]

use crate::cstr;
use crate::message::emsg;
use crate::message_fmt::c_str;
use crate::types::{Channel, Proc};

use crate::semsg;
use core::ffi::{c_char, c_void};
use core::ptr;

use crate::api::ui::{remote_ui_connect, remote_ui_disconnect};
use crate::api::vim::nvim__chan_set_detach;
use crate::api::vimscript::nvim_command;
use crate::channel::{channel_close, channel_job_start, find_channel};

use crate::eval::typval::{NumBuf, list_items, list_len};
use crate::eval::vars::{get_vim_var_list, get_vim_var_str};

use crate::event::proc::{proc_stop, proc_wait};
use crate::ex_docmd::xfree;
use crate::ex_docmd::{cmdmod_has, kChannelPartAll};
use crate::log::{LOGLVL_INF, logmsg};
use crate::memory::xcalloc;
use crate::message::e_invchan;
use crate::startup::{exiting, getout};
use crate::ui::state::current_ui;

use crate::msgpack_rpc::server::{server_start, server_stop};

use crate::strings::{concat_str, has_bytes};
use crate::types::channel::kChannelStdinPipe;
use crate::types::{
    ApiDict, ArenaMem, Array, Callback, CallbackReader, CmdModFlags, Error, ExArg, KeyValuePair,
    Object, String_0, VarNumber, Vv, key_value_pair, ptrdiff_t, size_t, uint16_t, uint64_t,
};
use crate::ui::{ui_active, ui_call_restart, ui_flush};

/// An `Object` holding a copy of a NUL-terminated string.
fn obj_str(s: *const c_char) -> Object {
    Object::string(cstr_to_string(s))
}

/// An `Object` holding a boolean.
fn obj_bool(b: bool) -> Object {
    Object::Boolean(b)
}

/// An `Array` of `items`.
fn array_of<const N: usize>(items: [Object; N]) -> Array {
    Array::from(Vec::from(items))
}

/// An `ApiDict` of `items`.
fn dict_of<const N: usize>(items: [KeyValuePair; N]) -> ApiDict {
    ApiDict::from(Vec::from(items))
}

/// A `key = value` entry for a borrowed `ApiDict`.
fn entry(key: &'static core::ffi::CStr, value: Object) -> KeyValuePair {
    key_value_pair {
        key: key.into(),
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
pub(crate) fn ex_restart(excmd: &mut ExArg) {
    let mut numbuf = NumBuf::new();
    let mut numbuf2 = NumBuf::new();
    let mut err = Error::none();
    let no_ui = ui_active() == 0;
    let exepath = get_vim_var_str(Vv::Progpath);
    let argv_list = get_vim_var_list(Vv::Argv);
    let argc = list_len(unsafe { argv_list.as_ref() });

    // Three more than `v:argv`: `--embed`, `--headless`, and the null
    // terminator.
    let argv = unsafe { xcalloc(argc as size_t + 3, size_of::<*mut c_char>()) } as *mut *mut c_char;
    let mut i: size_t = 0;
    let mut listen_arg: *const c_char = ptr::null();

    // SAFETY: `v:argv` is a live list of strings.
    let items = list_items(unsafe { argv_list.as_ref() });
    let mut at = 0;
    while at < items.len() {
        let arg = numbuf.string_ptr(&items[at].li_tv);
        // `-- [files…]` is dropped: it is almost never wanted, and
        // `:mksession` is the way to carry a session over.
        if i > 0 && strequal(arg, c"--".as_ptr()) {
            break;
        }
        // `-s <scriptfile>` is dropped, script file and all.
        if i > 0 && strequal(arg, c"-s".as_ptr()) {
            if at + 1 >= items.len() {
                break;
            }
            at += 2;
            continue;
        }
        // The address after `--listen` is in use by *this* server, so
        // it has to be released before the new one can take it.
        if i > 0 && strequal(arg, c"--listen".as_ptr()) {
            // SAFETY: the list entry is live and `string` answers a
            // NUL-terminated buffer that outlives the loop.
            if let Some(next_li) = items.get(at + 1)
                && let addr = numbuf2.string_ptr(&next_li.li_tv)
                && let text = unsafe { cstr::at(addr) }
                && (has_bytes(text, b":") || has_bytes(text, b"/") || has_bytes(text, b"\\"))
            {
                listen_arg = addr;
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
            unsafe { *argv.add(i as usize) = xstrdup(arg) };
            i += 1;
            if i == 1 {
                unsafe { *argv.add(i as usize) = xstrdup(c"--embed".as_ptr()) };
                i += 1;
                // Without `--headless`, an embedded server waits for a
                // UI to attach.
                if no_ui {
                    unsafe { *argv.add(i as usize) = xstrdup(c"--headless".as_ptr()) };
                    i += 1;
                }
            }
        }
        at += 1;
    }

    let server_stopped = !listen_arg.is_null() && unsafe { server_stop(listen_arg, true) };

    let mut on_err = blank_reader();
    // The stderr fd is inherited, so forwarding still works after this
    // server exits.
    on_err.fwd_err = true;
    let mut exit_status: VarNumber = 0;
    let channel = unsafe {
        channel_job_start(
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
        )
    };

    'fail_1: {
        if channel.is_null() {
            emsg(c"cannot create a channel job");
            break 'fail_1;
        }
        let id = unsafe { (*channel).id };
        let mut result_mem: ArenaMem = ptr::null_mut();

        'fail_2: {
            // Stop the new server exiting when this channel closes.
            let detach_items = [obj_bool(true)];
            if let Err(e) = rpc_send_call(
                id,
                c"nvim__chan_set_detach".as_ptr(),
                array_of(detach_items),
                &raw mut result_mem,
            ) {
                err = e;
                break 'fail_2;
            }
            arena_mem_free(result_mem);
            result_mem = ptr::null_mut();

            // `:restart {cmd}` runs {cmd} over there, once a UI has
            // arrived.
            if excmd.line.byte_at(excmd.line.arg) != 0 {
                let opt_items = [
                    entry(c"once", obj_bool(true)),
                    entry(c"nested", obj_bool(true)),
                    entry(
                        c"command",
                        Object::string(String_0::from_bytes(excmd.line.arg())),
                    ),
                ];
                let autocmd_items = [
                    obj_str(c"UIEnter".as_ptr()),
                    Object::dict(dict_of(opt_items)),
                ];
                if let Err(e) = rpc_send_call(
                    id,
                    c"nvim_create_autocmd".as_ptr(),
                    array_of(autocmd_items),
                    &raw mut result_mem,
                ) {
                    err = e;
                    break 'fail_2;
                }
                arena_mem_free(result_mem);
                result_mem = ptr::null_mut();
            }

            // Where the UIs are to reconnect.
            let name_items = [obj_str(c"servername".as_ptr())];
            let result = match rpc_send_call(
                id,
                c"nvim_get_vvar".as_ptr(),
                array_of(name_items),
                &raw mut result_mem,
            ) {
                Ok(result) => result,
                Err(e) => {
                    err = e;
                    break 'fail_2;
                }
            };
            let servername = result.into_string().filter(|s| !s.is_empty());
            let Some(servername) = servername else {
                emsg(c"restart failed: could not get listen address from new server");
                break 'fail_2;
            };
            arena_mem_free(result_mem);
            result_mem = ptr::null_mut();

            ui_call_restart(servername);
            ui_flush();

            set_vim_var_string(Vv::Exitreason, c"restart".as_ptr(), 7 as ptrdiff_t);

            let mut quit_cmd = if excmd.do_ecmd_cmd.is_none() {
                c"qall".as_ptr().cast_mut()
            } else {
                excmd.do_ecmd_cmd.ptr(&excmd.line).cast_mut()
            };
            let mut quit_cmd_copy: *mut c_char = ptr::null_mut();
            if cmdmod_has(CmdModFlags::CONFIRM) {
                quit_cmd_copy = unsafe { concat_str(c"confirm ".as_ptr(), quit_cmd) };
                quit_cmd = quit_cmd_copy;
            }
            if let Err(e) = nvim_command(cstr_to_string(quit_cmd)) {
                err = e;
            }
            xfree(quit_cmd_copy as *mut c_void);

            if err.is_set() {
                emsg(err.message_or_empty());
                err.clear();
            } else if !exiting.get() {
                emsg(c"restart failed: +cmd did not quit the server");
            }
        }

        // Reached both on success — where `exiting` is set and this is
        // the last thing that runs — and on every failure.
        set_vim_var_string(Vv::Exitreason, ptr::null(), -1 as ptrdiff_t);
        if err.is_set() {
            emsg(err.message_or_empty());
            err.clear();
        }
        arena_mem_free(result_mem);
        result_mem = ptr::null_mut();

        // Close the new server's stderr before killing it, or its dying
        // words land on this UI.
        let chanclose_items = [obj_str(c"chanclose(v:stderr)".as_ptr())];
        drop(rpc_send_call(
            id,
            c"nvim_eval".as_ptr(),
            array_of(chanclose_items),
            &raw mut result_mem,
        ));
        arena_mem_free(result_mem);

        unsafe { proc_stop(channel_proc(channel)) };
        if unsafe { proc_wait(channel_proc(channel), -1, ptr::null_mut()) } < 0 {
            emsg(c"killing new nvim server failed");
        }
    }

    // The address was released for a server that is not going to use it.
    if server_stopped && unsafe { server_start(listen_arg) } != 0 {
        // SAFETY: a message argument the caller holds as a NUL-terminated string.
        let listen_arg = unsafe { c_str(listen_arg) };
        semsg!("couldn't resume listening on {listen_arg}");
    }
}

/// A `CallbackReader` that reads nothing.
fn blank_reader() -> CallbackReader {
    CallbackReader {
        cb: blank_callback(),
        ..CallbackReader::none()
    }
}

/// A `Callback` that calls nothing.
fn blank_callback() -> Callback {
    Callback::None
}

/// `:detach` — let the UI go, and keep running headless.
pub(crate) fn ex_detach(excmd: &mut ExArg) {
    if excmd.forceit {
        emsg(c"bang (!) not supported yet");
        return;
    }
    detach_ui();
}

/// Let the current UI go without taking the server down with it.
///
/// Split from the handler because `:connect` detaches once it has attached
/// somewhere else, and has no bang of its own to answer for.
fn detach_ui() {
    if current_ui.get() == 0 {
        emsg(c"UI not attached");
        return;
    }
    let chan = find_channel(current_ui.get());
    if chan.is_null() {
        emsg(e_invchan);
        return;
    }

    // Tell the UI's channel not to take the server down with it. A
    // failure here is not worth reporting, but its message is still ours
    // to free.
    if let Err(mut detach_err) = unsafe { nvim__chan_set_detach((*chan).id, true) } {
        detach_err.clear();
    }

    if let Err(e) = unsafe { remote_ui_disconnect((*chan).id, true) } {
        emsg(e.message_or_empty());
        return;
    }

    let mut close_err: *const c_char = ptr::null();
    if !unsafe { channel_close((*chan).id, kChannelPartAll, &raw mut close_err) }
        && !close_err.is_null()
    {
        // SAFETY: `channel_close` left a NUL-terminated reason behind.
        emsg(unsafe { cstr::at(close_err) });
        return;
    }
    // SAFETY: the channel this command just closed is still live.
    let id = unsafe { (*chan).id };
    logmsg!(LOGLVL_INF, c"ex_detach", 6019, "detach current_ui={id}");
}

/// `:connect` — attach this session's UI to another server, then detach
/// from here.
///
/// `:connect!` also *exits* when this was the only UI, so that the session
/// really moves rather than being left running.
pub(crate) fn ex_connect(excmd: &mut ExArg) {
    let stop_server = excmd.forceit && ui_active() == 1;
    let addr = excmd.line.cstr_from(excmd.line.arg);
    if let Err(e) = remote_ui_connect(current_ui.get(), addr) {
        emsg(e.message_or_empty());
        return;
    }
    detach_ui();
    if stop_server {
        exiting.set(true);
        getout(0);
    }
}

/// `arena_mem_free()` as checked code.
fn arena_mem_free(mem: ArenaMem) {
    // SAFETY: reads the editor's own state, which exists from startup to exit.
    unsafe { crate::memory::arena_mem_free(mem) }
}

/// `channel_proc()` as checked code.
fn channel_proc(chan: *mut Channel) -> *mut Proc {
    // SAFETY: the pointers are the command line's own, and live for the call.
    unsafe { crate::channel::channel_proc(chan) }
}

/// `cstr_to_string()` as checked code.
fn cstr_to_string(str: *const c_char) -> String_0 {
    // SAFETY: the pointers are the command line's own, and live for the call.
    unsafe { crate::api::private::helpers::cstr_to_string(str) }
}

/// `rpc_send_call()` as checked code.
fn rpc_send_call(
    id: uint64_t,
    method_name: *const c_char,
    args: Array,
    result_mem: *mut ArenaMem,
) -> Result<Object, Error> {
    // SAFETY: the pointers are the command line's own, and live for the call.
    unsafe { crate::msgpack_rpc::channel::rpc_send_call(id, method_name, args, result_mem) }
}

/// `set_vim_var_string()` as checked code.
fn set_vim_var_string(idx: Vv, val: *const c_char, len: ptrdiff_t) {
    // SAFETY: the pointers are the command line's own, and live for the call.
    unsafe { crate::eval::vars::set_vim_var_string(idx, val, len) }
}

/// `strequal()` as checked code.
fn strequal(a: *const c_char, b: *const c_char) -> bool {
    // SAFETY: two NUL-terminated strings, or null.
    unsafe { crate::memory::strequal(a, b) }
}

/// `xstrdup()` as checked code.
fn xstrdup(str: *const c_char) -> *mut c_char {
    // SAFETY: a NUL-terminated string.
    unsafe { crate::memory::xstrdup(str) }
}
