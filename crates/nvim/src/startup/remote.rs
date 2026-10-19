//! `--server` and the `--remote` family: connect to another Nvim and hand it
//! the rest of the command line.
//!
//! Everything past the `--remote*` argument is forwarded verbatim to
//! `vim._cs_remote()` in the server, which answers with a dict saying what
//! this process should do next -- print something, open some tab pages, or
//! exit.

#![forbid(unsafe_code)]

use core::ffi::{CStr, c_int};

use crate::channel::channel_connect;
use crate::cstr;
use crate::event::socket::socket_address_is_tcp;
use crate::lua::executor::nlua_exec_cstr;
use crate::lua::state::kRetObject;
use crate::os::cshim::{write_c_stderr, write_c_stdout};
use crate::os::env::{ENV_NVIM, env_get_bounded};
use crate::startup::exit::os_exit;
use crate::startup::{MainParams, WIN_TABS, ui_client_channel_id};
use crate::types::{ApiDict, Array, CallbackReader, Integer, Object, String_0, size_t, uint64_t};

/// How long to wait for the server to answer the connection, in ms.
const CONNECT_TIMEOUT_MS: c_int = 500;

/// The Lua entry point that does the actual work, in the *server*.
const CS_REMOTE: &CStr = c"return vim._cs_remote(...)";

/// Open a channel to `server_addr`, or say why not.
pub(crate) fn server_connect(server_addr: Option<&CStr>) -> Result<uint64_t, &'static CStr> {
    let server_addr = server_addr.ok_or(c"no address specified")?;
    // Nothing reads from this channel; the reply comes back through the
    // request itself.
    let on_data = CallbackReader::none();
    let is_tcp = socket_address_is_tcp(server_addr);
    channel_connect(is_tcp, server_addr, true, on_data, CONNECT_TIMEOUT_MS)
}

/// Write `parts` to stderr as one message, and exit with `status`.
fn fail(parts: &[&[u8]], status: c_int) -> ! {
    write_c_stderr(&parts.concat());
    os_exit(status)
}

/// Complain that `vim._cs_remote` answered with the wrong shape, and exit 2.
fn bad_reply_type(key: &CStr) -> ! {
    let what = b"vim._cs_remote returned an unexpected type for '";
    fail(&[what, key.to_bytes(), b"'\n"], 2)
}

/// Read one key of the reply dict, checking its type first.
fn field(dict: &ApiDict, index: size_t) -> (&CStr, &Object) {
    let pair = &dict[index];
    (pair.key.as_c_str(), &pair.value)
}

/// An API string's text, up to its first NUL -- what `printf("%s")` wrote.
fn text_of(text: &String_0) -> &[u8] {
    cstr::until_nul(text.as_bytes())
}

/// Hand the rest of the command line to the server named by `--server`.
///
/// `remote_args` is the index of the `--remote*` argument itself, so
/// `argv[remote_args..]` is what gets forwarded.
///
/// `ui_only` says this is the pass that runs before the built-in UI starts,
/// and only `--remote-ui` is allowed to do anything in it -- everything else
/// waits until the UI is up so its output has somewhere to go.
///
/// Returns only when the process should carry on starting up; the server's
/// answer may instead exit.
pub(crate) fn remote_request(parm: &mut MainParams, remote_args: usize, ui_only: bool) {
    let is_ui = parm.argv[remote_args].as_c_str() == c"--remote-ui";
    if ui_only && !is_ui {
        return;
    }

    let server_addr = parm.server_addr.as_deref();
    let connected = server_connect(server_addr);

    if is_ui {
        match connected {
            Err(why) => fail(&[b"Remote ui failed to start: ", why.to_bytes(), b"\n"], 1),
            Ok(chan) => {
                let addr = server_addr.expect("a connection had an address").to_bytes();
                if addr == env_get_bounded(ENV_NVIM) {
                    // $NVIM in a `:terminal` child names its own parent, and
                    // a UI attached to that is a loop.
                    let why = b"Cannot attach UI of :terminal child to its parent. ";
                    fail(&[why, b"(Unset $NVIM to skip this check)\n"], 1);
                }
                ui_client_channel_id.set(chan);
                return;
            }
        }
    }

    // The forwarded arguments, as an API Array of strings.
    let forwarded = &parm.argv[remote_args..];
    let mut args = Array::with_capacity(forwarded.len());
    for word in forwarded {
        args.push(Object::string(String_0::from_cstr(word)));
    }

    // `vim._cs_remote(channel, address, connect_error, args)`. A missing
    // address or error is a null string, as upstream's NULL `char *` was.
    let string =
        |text: Option<&CStr>| Object::string(text.map_or(String_0::NULL, String_0::from_cstr));
    let (chan, connect_error) = match connected {
        Ok(chan) => (chan, None),
        Err(why) => (0, Some(why)),
    };
    let call_args = Array::from(vec![
        Object::Integer(Integer::from(chan as c_int)),
        string(server_addr),
        string(connect_error),
        Object::array(args),
    ]);

    let reply = match nlua_exec_cstr(CS_REMOTE, call_args, kRetObject) {
        Ok(value) => value,
        Err(err) => fail(&[err.message_or_empty().to_bytes(), b"\n"], 2),
    };
    let Some(dict) = reply.as_dict() else {
        fail(&[b"vim._cs_remote returned unexpected value\n"], 2);
    };

    // `should_exit` and `tabbed` are three-state so that "the server did
    // not say" is distinguishable from "the server said no".
    let mut should_exit: Option<bool> = None;
    let mut tabbed: Option<bool> = None;
    for i in 0..dict.len() {
        let (key, value) = field(dict, i);
        match key.to_bytes() {
            b"errmsg" => {
                let Some(text) = value.as_string() else {
                    bad_reply_type(c"errmsg");
                };
                fail(&[text_of(text), b"\n"], 2);
            }
            b"result" => {
                let Some(text) = value.as_string() else {
                    bad_reply_type(c"result");
                };
                write_c_stdout(text_of(text));
            }
            b"tabbed" => {
                let Some(flag) = value.as_boolean() else {
                    bad_reply_type(c"tabbed");
                };
                tabbed = Some(flag);
            }
            b"should_exit" => {
                let Some(flag) = value.as_boolean() else {
                    bad_reply_type(c"should_exit");
                };
                should_exit = Some(flag);
            }
            _ => {}
        }
    }

    if should_exit.is_none() || tabbed.is_none() {
        let msg = b"vim._cs_remote didn't return a value for should_exit or tabbed, bailing\n";
        fail(&[msg], 2);
    }

    drop(reply);

    if should_exit == Some(true) {
        os_exit(0);
    }
    if tabbed == Some(true) {
        // One tab page per file the server was asked to open, less the
        // `--remote*` argument itself.
        let files = parm.argv.len() - remote_args - 1;
        parm.window_count = c_int::try_from(files).expect("an argv count");
        parm.window_layout = WIN_TABS as c_int;
    }
}
