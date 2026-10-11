//! `:restart`, `:detach` and `:connect` — the commands that hand the
//! session to another process or take it back.
#![forbid(unsafe_code)]

use crate::channel::channel_close_or_report;
use crate::channel::channel_job_stop_wait;
use crate::channel::open::channel_job_start_argv;
use crate::memory::ThinCString;
use crate::message::emsg;
use crate::message_fmt::msg_cstr;
use crate::msgpack_rpc::channel::rpc_call_with;
use crate::os::shell::Argv;
use core::ffi::CStr;

use crate::semsg;

use crate::api::ui::{remote_ui_connect, remote_ui_disconnect};
use crate::api::vim::nvim__chan_set_detach;
use crate::api::vimscript::nvim_command;
use crate::channel::find_channel;

use crate::eval::typval::{NumBuf, list_items, list_len};
use crate::eval::vars::{get_vim_var_list_handle, set_vim_var_string, vim_var_bytes};

use crate::ex_docmd::{cmdmod_has, kChannelPartAll};
use crate::log::{LOGLVL_INF, logmsg};
use crate::message::e_invchan;
use crate::startup::{exiting, getout};
use crate::ui::state::current_ui;

use crate::msgpack_rpc::server::{server_start, server_stop};

use crate::strings::{concat, has_bytes};
use crate::types::{
    ApiDict, Array, Callback, CallbackReader, CmdModFlags, Error, ExArg, KeyValuePair, Object,
    String_0, VarNumber, Vv, key_value_pair,
};
use crate::ui::{ui_active, ui_call_restart, ui_flush};

/// An `Object` holding a copy of a string.
fn obj_str(s: &CStr) -> Object {
    Object::string(String_0::from_cstr(s))
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
fn entry(key: &'static CStr, value: Object) -> KeyValuePair {
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
    let mut err = Error::none();
    let no_ui = ui_active() == 0;
    // A copy: the variable is held across the job start below.
    let exepath = ThinCString::from_vec(vim_var_bytes(Vv::Progpath));
    let argv_list = get_vim_var_list_handle(Vv::Argv);

    let mut words: Vec<ThinCString> =
        Vec::with_capacity(list_len(argv_list.as_deref()) as usize + 2);
    // The address after `--listen`: in use by *this* server, so it has to be
    // released before the new one can take it.
    let mut listen_arg: Option<ThinCString> = None;

    let items = list_items(argv_list.as_deref());
    let mut at = 0;
    while at < items.len() {
        let arg = ThinCString::from_cstr(numbuf.string(&items[at].li_tv));
        let first = words.is_empty();
        // `-- [files…]` is dropped: it is almost never wanted, and
        // `:mksession` is the way to carry a session over.
        if !first && arg.as_cstr() == c"--" {
            break;
        }
        // `-s <scriptfile>` is dropped, script file and all.
        if !first && arg.as_cstr() == c"-s" {
            if at + 1 >= items.len() {
                break;
            }
            at += 2;
            continue;
        }
        if !first
            && arg.as_cstr() == c"--listen"
            && let Some(next_li) = items.get(at + 1)
            && let text = numbuf.string(&next_li.li_tv)
            && (has_bytes(text, b":") || has_bytes(text, b"/") || has_bytes(text, b"\\"))
        {
            listen_arg = Some(ThinCString::from_cstr(text));
        }
        // `--embed`, `--headless` and `-` are replaced by exactly one
        // `--embed` (plus `--headless` when there is no UI), inserted
        // right after argv[0].
        let dropped = [c"--embed", c"--headless", c"-"].contains(&arg.as_cstr());
        if first || !dropped {
            words.push(arg);
            if first {
                words.push(ThinCString::from_cstr(c"--embed"));
                // Without `--headless`, an embedded server waits for a
                // UI to attach.
                if no_ui {
                    words.push(ThinCString::from_cstr(c"--headless"));
                }
            }
        }
        at += 1;
    }
    let server_stopped = listen_arg
        .as_ref()
        .is_some_and(|addr| server_stop(addr.as_cstr(), true));

    let mut on_err = blank_reader();
    // The stderr fd is inherited, so forwarding still works after this
    // server exits.
    on_err.fwd_err = true;
    let mut exit_status: VarNumber = 0;
    // `v:argv` names at least the program; an empty one starts nothing, as
    // a vector with no program cannot.
    let channel = (!words.is_empty())
        .then(|| {
            channel_job_start_argv(
                Argv::from_words(words),
                Some(exepath.as_cstr()),
                blank_reader(),
                on_err,
                blank_callback(),
                true,
                true,
                true, // detach: the new server outlives this one
                &mut exit_status,
            )
        })
        .flatten();

    if let Some(id) = channel {
        'fail_2: {
            // Stop the new server exiting when this channel closes.
            let detach_items = [obj_bool(true)];
            if let Err(e) =
                rpc_call_with(id, c"nvim__chan_set_detach", array_of(detach_items), drop)
            {
                err = e;
                break 'fail_2;
            }

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
                let autocmd_items = [obj_str(c"UIEnter"), Object::dict(dict_of(opt_items))];
                if let Err(e) =
                    rpc_call_with(id, c"nvim_create_autocmd", array_of(autocmd_items), drop)
                {
                    err = e;
                    break 'fail_2;
                }
            }

            // Where the UIs are to reconnect.
            let name_items = [obj_str(c"servername")];
            let servername = match rpc_call_with(id, c"nvim_get_vvar", array_of(name_items), |r| {
                r.into_string()
            }) {
                Ok(name) => name.filter(|s| !s.is_empty()),
                Err(e) => {
                    err = e;
                    break 'fail_2;
                }
            };
            let Some(servername) = servername else {
                emsg(c"restart failed: could not get listen address from new server");
                break 'fail_2;
            };

            ui_call_restart(servername);
            ui_flush();

            set_vim_var_string(Vv::Exitreason, Some(b"restart"));

            let quit_cmd = excmd.do_ecmd_cmd.cstr(&excmd.line).unwrap_or(c"qall");
            let quit_cmd = if cmdmod_has(CmdModFlags::CONFIRM) {
                String_0::from_bytes(&concat(b"confirm ", quit_cmd.to_bytes()))
            } else {
                String_0::from_cstr(quit_cmd)
            };
            if let Err(e) = nvim_command(quit_cmd) {
                err = e;
            }

            if err.is_set() {
                emsg(err.message_or_empty());
                err.clear();
            } else if !exiting.get() {
                emsg(c"restart failed: +cmd did not quit the server");
            }
        }

        // Reached both on success — where `exiting` is set and this is
        // the last thing that runs — and on every failure.
        set_vim_var_string(Vv::Exitreason, None);
        if err.is_set() {
            emsg(err.message_or_empty());
            err.clear();
        }

        // Close the new server's stderr before killing it, or its dying
        // words land on this UI.
        let chanclose_items = [obj_str(c"chanclose(v:stderr)")];
        drop(rpc_call_with(
            id,
            c"nvim_eval",
            array_of(chanclose_items),
            drop,
        ));

        if channel_job_stop_wait(id, -1).is_some_and(|status| status < 0) {
            emsg(c"killing new nvim server failed");
        }
    } else {
        emsg(c"cannot create a channel job");
    }

    // The address was released for a server that is not going to use it.
    if server_stopped
        && let Some(addr) = &listen_arg
        && server_start(Some(addr.as_cstr())) != 0
    {
        let listen_arg = msg_cstr(addr.as_cstr());
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
    let id = current_ui.get();
    if find_channel(id).is_null() {
        emsg(e_invchan);
        return;
    }

    // Tell the UI's channel not to take the server down with it. A
    // failure here is not worth reporting, but its message is still ours
    // to free.
    if let Err(mut detach_err) = nvim__chan_set_detach(id, true) {
        detach_err.clear();
    }

    if let Err(e) = remote_ui_disconnect(id, true) {
        emsg(e.message_or_empty());
        return;
    }

    if !channel_close_or_report(id, kChannelPartAll) {
        return;
    }
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
