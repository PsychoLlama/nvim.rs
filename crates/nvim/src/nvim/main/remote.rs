//! `--server` and the `--remote` family: connect to another Nvim and hand it
//! the rest of the command line.
//!
//! Everything past the `--remote*` argument is forwarded verbatim to
//! `vim._cs_remote()` in the server, which answers with a dict saying what
//! this process should do next -- print something, open some tab pages, or
//! exit.

#![deny(unsafe_op_in_unsafe_fn)]

use core::ffi::{CStr, c_char, c_int, c_void};
use core::mem::size_of;
use core::ptr;

use crate::src::nvim::api::private::helpers::{api_free_object, cstr_as_string};
use crate::src::nvim::channel::channel_connect;
use crate::src::nvim::eval::typval::kCallbackNone;
use crate::src::nvim::event::socket::socket_address_is_tcp;
use crate::src::nvim::lua::executor::nlua_exec;
use crate::src::nvim::main::exit::os_exit;
use crate::src::nvim::main::{
    ARRAY_DICT_INIT, GA_EMPTY_INIT_VALUE, WIN_TABS, kRetObject, mparm_T, ui_client_channel_id,
};
use crate::src::nvim::memory::{strequal, xfree, xrealloc};
use crate::src::nvim::os::env::os_getenv_noalloc;
use crate::src::nvim::os::libc::{fprintf, printf, stderr};
use crate::src::nvim::types::{
    Arena, Array, Callback, Callback_data, CallbackReader, Dict, Error, Integer, Object, String_0,
    TriState, dict_T, kErrorTypeNone, kFalse, kNone, kObjectTypeArray, kObjectTypeBoolean,
    kObjectTypeDict, kObjectTypeInteger, kObjectTypeString, kTrue, object_data, size_t, uint64_t,
};

/// How long to wait for the server to answer the connection, in ms.
const CONNECT_TIMEOUT_MS: c_int = 500;

/// The Lua entry point that does the actual work, in the *server*.
const CS_REMOTE: &CStr = c"return vim._cs_remote(...)";

/// Open a channel to `server_addr`.
///
/// Answers 0 and fills `errmsg` on failure; the message is owned by the
/// channel layer and outlives the call.
pub(crate) unsafe fn server_connect(
    server_addr: *mut c_char,
    errmsg: *mut *const c_char,
) -> uint64_t {
    // SAFETY: `errmsg` is a live out-parameter; `server_addr`, when non-null,
    // is a NUL-terminated address.
    unsafe {
        if server_addr.is_null() {
            *errmsg = c"no address specified".as_ptr();
            return 0;
        }

        // Nothing reads from this channel; the reply comes back through the
        // request itself.
        let on_data = CallbackReader {
            cb: Callback {
                data: Callback_data {
                    funcref: ptr::null_mut(),
                },
                type_0: kCallbackNone,
            },
            self_0: ptr::null_mut::<dict_T>(),
            buffer: GA_EMPTY_INIT_VALUE,
            eof: false,
            buffered: false,
            fwd_err: false,
            type_0: ptr::null(),
        };

        let is_tcp = socket_address_is_tcp(CStr::from_ptr(server_addr));
        let mut error: *const c_char = ptr::null();
        let chan = channel_connect(
            is_tcp,
            server_addr,
            true,
            on_data,
            CONNECT_TIMEOUT_MS,
            &raw mut error,
        );
        if !error.is_null() {
            *errmsg = error;
            return 0;
        }
        chan
    }
}

/// Complain that `vim._cs_remote` answered with the wrong shape, and exit 2.
unsafe fn bad_reply_type(key: &CStr) -> ! {
    // SAFETY: writes one message to stderr and does not return.
    unsafe {
        fprintf(
            stderr,
            c"vim._cs_remote returned an unexpected type for '%s'\n".as_ptr(),
            key.as_ptr(),
        );
        os_exit(2)
    }
}

/// Read one key of the reply dict, checking its type first.
unsafe fn field<'a>(dict: &'a Dict, index: size_t) -> (&'a CStr, &'a Object) {
    // SAFETY: `index` is below `dict.size`, so the pair is in the items array
    // and its key is a NUL-terminated string.
    unsafe {
        let pair = &*dict.items.add(index);
        (CStr::from_ptr(pair.key.data), &pair.value)
    }
}

/// Hand the rest of the command line to the server named by `--server`.
///
/// `remote_args` is the index of the `--remote*` argument itself, so
/// `argv[remote_args..argc]` is what gets forwarded.
///
/// `ui_only` says this is the pass that runs before the built-in UI starts,
/// and only `--remote-ui` is allowed to do anything in it -- everything else
/// waits until the UI is up so its output has somewhere to go.
///
/// Returns only when the process should carry on starting up; the server's
/// answer may instead exit.
pub(crate) unsafe fn remote_request(
    params: *mut mparm_T,
    remote_args: c_int,
    server_addr: *mut c_char,
    argc: c_int,
    argv: *mut *mut c_char,
    ui_only: bool,
) {
    // SAFETY: `argv[0..argc]` are the process arguments and `params` is the
    // caller's live parameter block.
    unsafe {
        let is_ui = strequal(*argv.offset(remote_args as isize), c"--remote-ui".as_ptr());
        if ui_only && !is_ui {
            return;
        }

        let mut connect_error: *const c_char = ptr::null();
        let chan = server_connect(server_addr, &raw mut connect_error);

        if is_ui {
            if chan == 0 {
                fprintf(
                    stderr,
                    c"Remote ui failed to start: %s\n".as_ptr(),
                    connect_error,
                );
                os_exit(1);
            } else if strequal(server_addr, os_getenv_noalloc(c"NVIM".as_ptr())) {
                // $NVIM in a `:terminal` child names its own parent, and a UI
                // attached to that is a loop.
                fprintf(
                    stderr,
                    c"%s".as_ptr(),
                    c"Cannot attach UI of :terminal child to its parent. ".as_ptr(),
                );
                fprintf(
                    stderr,
                    c"%s\n".as_ptr(),
                    c"(Unset $NVIM to skip this check)".as_ptr(),
                );
                os_exit(1);
            }
            ui_client_channel_id.set(chan);
            return;
        }

        // The forwarded arguments, as an API Array of strings.
        let mut args: Array = ARRAY_DICT_INIT;
        args.capacity = (argc - remote_args) as size_t;
        args.items = xrealloc(
            args.items as *mut c_void,
            size_of::<Object>().wrapping_mul(args.capacity),
        ) as *mut Object;
        for i in remote_args..argc {
            *args.items.add(args.size) = Object {
                type_0: kObjectTypeString,
                data: object_data {
                    string: cstr_as_string(*argv.offset(i as isize)),
                },
            };
            args.size += 1;
        }

        // `vim._cs_remote(channel, address, connect_error, args)`.
        let mut call_args: [Object; 4] = [
            Object {
                type_0: kObjectTypeInteger,
                data: object_data {
                    integer: chan as c_int as Integer,
                },
            },
            Object {
                type_0: kObjectTypeString,
                data: object_data {
                    string: cstr_as_string(server_addr),
                },
            },
            Object {
                type_0: kObjectTypeString,
                data: object_data {
                    string: cstr_as_string(connect_error),
                },
            },
            Object {
                type_0: kObjectTypeArray,
                data: object_data { array: args },
            },
        ];
        let a = Array {
            size: 4,
            capacity: 4,
            items: call_args.as_mut_ptr(),
        };

        let mut err = Error {
            type_0: kErrorTypeNone,
            msg: ptr::null_mut(),
        };
        let script = String_0 {
            data: CS_REMOTE.as_ptr() as *mut c_char,
            size: CS_REMOTE.count_bytes(),
        };
        let reply = nlua_exec(
            script,
            ptr::null(),
            a,
            kRetObject,
            ptr::null_mut::<Arena>(),
            &raw mut err,
        );

        xfree(args.items as *mut c_void);
        args.size = 0;
        args.capacity = 0;
        args.items = ptr::null_mut();

        if err.type_0 != kErrorTypeNone {
            fprintf(stderr, c"%s\n".as_ptr(), err.msg);
            os_exit(2);
        }
        if reply.type_0 != kObjectTypeDict {
            fprintf(
                stderr,
                c"vim._cs_remote returned unexpected value\n".as_ptr(),
            );
            os_exit(2);
        }
        let dict = reply.data.dict;

        // `should_exit` and `tabbed` are three-state so that "the server did
        // not say" is distinguishable from "the server said no".
        let mut should_exit: TriState = kNone;
        let mut tabbed: TriState = kNone;
        for i in 0..dict.size {
            let (key, value) = field(&dict, i);
            match key.to_bytes() {
                b"errmsg" => {
                    if value.type_0 != kObjectTypeString {
                        bad_reply_type(c"errmsg");
                    }
                    fprintf(stderr, c"%s\n".as_ptr(), value.data.string.data);
                    os_exit(2);
                }
                b"result" => {
                    if value.type_0 != kObjectTypeString {
                        bad_reply_type(c"result");
                    }
                    printf(c"%s".as_ptr(), value.data.string.data);
                }
                b"tabbed" => {
                    if value.type_0 != kObjectTypeBoolean {
                        bad_reply_type(c"tabbed");
                    }
                    tabbed = if value.data.boolean { kTrue } else { kFalse };
                }
                b"should_exit" => {
                    if value.type_0 != kObjectTypeBoolean {
                        bad_reply_type(c"should_exit");
                    }
                    should_exit = if value.data.boolean { kTrue } else { kFalse };
                }
                _ => {}
            }
        }

        if should_exit == kNone || tabbed == kNone {
            fprintf(
                stderr,
                c"vim._cs_remote didn't return a value for should_exit or tabbed, bailing\n"
                    .as_ptr(),
            );
            os_exit(2);
        }

        api_free_object(reply);

        if should_exit == kTrue {
            os_exit(0);
        }
        if tabbed == kTrue {
            // One tab page per file the server was asked to open, less the
            // `--remote*` argument itself.
            (*params).window_count = argc - remote_args - 1;
            (*params).window_layout = WIN_TABS as c_int;
        }
    }
}
