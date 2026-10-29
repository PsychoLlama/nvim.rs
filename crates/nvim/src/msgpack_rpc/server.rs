#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

//! The listening side of msgpack-rpc: the sockets `--listen`,
//! `serverstart()` and `$NVIM_LISTEN_ADDRESS` open, and the channels that
//! come out of them.

use core::ffi::{CStr, c_int};

use crate::channel::channel_from_connection;
use crate::eval::vars::{set_vim_var_string, with_vim_var_str};
use crate::event::socket::Listener;
use crate::event::socket::address::is_bare_server_name;
use crate::global_cell::GlobalCell;
use crate::log::{LOGLVL_ERR, LOGLVL_WRN, logmsg};
use crate::memory::XString;
use crate::message_fmt::{msg_bytes, msg_cstr};
use crate::os::env::{ENV_TEST_LOG, env_exists, env_unset, getenv_owned, os_get_pid};
use crate::os::fs::os_strerror;
use crate::os::stdpaths::{get_appname, xdg_var};
use crate::path::fix_fname;
use crate::types::{IOSIZE, SocketWatcher, Vv, uint32_t};
use std::ffi::CString;

use crate::event::socket::address::SOCKET_ADDR_LEN;

/// Values these belong to other modules; nested so they stay out of the flat
/// namespace the unit-test header generator collects constants into.
mod known {
    use core::ffi::c_int;

    /// `XDGVarType` of the runtime directory generated addresses live in.
    pub(super) const XDG_RUNTIME_DIR: c_int = 4;
    /// How many connections the kernel may queue behind a listening socket.
    pub(super) const MAX_CONNECTIONS: c_int = 32;
}

use known::*;

const ENV_LISTEN: &CStr = c"NVIM_LISTEN_ADDRESS";

/// Every socket this instance is listening on. The first one is what
/// `v:servername` reports.
static WATCHERS: GlobalCell<Vec<Listener>> = GlobalCell::new(Vec::new());

/// Where the start-up listening address came from.
///
/// Upstream keeps this in a `TriState`, but the three values name three
/// sources, not an unknown boolean: only the first two are worth a message
/// when listening fails, and each gets its own wording.
#[derive(Copy, Clone, PartialEq, Eq)]
enum ListenSource {
    /// `--listen <address>`.
    Argument,
    /// `$NVIM_LISTEN_ADDRESS`.
    Environment,
    /// Nobody asked; the address was made up.
    Generated,
}

/// Opens the start-up listening socket, if there is to be one.
///
/// Answers why not when that failed. An address the user did not ask for is
/// generated, and failing to listen on *that* is not an error — the editor
/// runs fine without a server. Upstream leaves the reason in the shared
/// `IObuff` instead.
pub fn server_init(listen_addr: Option<&CStr>) -> Result<(), CString> {
    // Which of the three sources the address came from, in the order they
    // are consulted, and the address itself when this call made it.
    let given = listen_addr.filter(|addr| !addr.is_empty());
    let (source, made) = match given {
        Some(_) => (ListenSource::Argument, None),
        None if env_exists(ENV_LISTEN, true) => {
            (ListenSource::Environment, getenv_owned(ENV_LISTEN))
        }
        None => (ListenSource::Generated, Some(server_address_generated())),
    };
    let listen_addr = given.or(made.as_ref().map(XString::as_cstr));

    let rv = server_start(listen_addr);

    if env_exists(ENV_TEST_LOG, false) {
        logmsg!(LOGLVL_ERR, c"server_init", 58, "test log message");
    }

    let mut result = Ok(());
    if rv != 0 && source != ListenSource::Generated {
        let what: &[u8] = if source == ListenSource::Argument {
            b"Failed to --listen"
        } else {
            b"Failed $NVIM_LISTEN_ADDRESS"
        };
        let text = if rv < 0 {
            os_strerror(rv)
        } else if rv == 1 {
            c"empty address"
        } else {
            c"?"
        };
        let addr = listen_addr.map_or(&b"(null)"[..], CStr::to_bytes);
        let mut reason = [what, b": ", text.to_bytes(), b": \"", addr, b"\""].concat();
        // Upstream formats into `IObuff`, which cuts the reason short.
        reason.truncate(IOSIZE as usize - 1);
        result = Err(CString::new(reason).expect("an address and a message hold no NUL"));
    }

    // The variable exists to tell *this* process where to listen; child
    // processes must not inherit it.
    if env_exists(ENV_LISTEN, false) {
        env_unset(ENV_LISTEN);
    }
    result
}

/// Points `v:servername` at whatever is now the first listening socket, or
/// clears it when there is none.
fn set_vservername() {
    let default_server =
        WATCHERS.with(|watchers| watchers.first().map(|first| first.address().to_owned()));
    set_vim_var_string(
        Vv::Servername,
        default_server.as_deref().map(CStr::to_bytes),
    );
}

pub fn server_teardown() {
    let listening = WATCHERS.with_mut(core::mem::take);
    for watcher in listening {
        watcher.close();
    }
}

/// [`server_address_new`] under the editor's own name.
pub fn server_address_generated() -> XString {
    server_address_new(None)
}

/// Generates an address in the runtime directory, unique to this process.
///
/// `name` names the socket; without one the editor's own name is used. The
/// answer is cut to what a socket address holds, as upstream's
/// `snprintf` into a `SOCKET_ADDR_LEN` buffer cut it.
pub fn server_address_new(name: Option<&CStr>) -> XString {
    static SEQUENCE: GlobalCell<uint32_t> = GlobalCell::new(0);

    let sequence = SEQUENCE.get();
    SEQUENCE.set(sequence.wrapping_add(1));

    let appname = get_appname(true);
    let dir = xdg_var(XDG_RUNTIME_DIR);
    let base = name.unwrap_or(&appname);
    let pid = os_get_pid();
    let mut address = Vec::with_capacity(SOCKET_ADDR_LEN);
    address.extend_from_slice(dir.as_ref().map_or(&b"(null)"[..], |dir| dir.to_bytes()));
    address.push(b'/');
    address.extend_from_slice(base.to_bytes());
    address.extend_from_slice(format!(".{pid}.{sequence}").as_bytes());

    if address.len() >= SOCKET_ADDR_LEN {
        address.truncate(SOCKET_ADDR_LEN - 1);
        let head = msg_bytes(&address[..address.len().min(40)]);
        logmsg!(
            LOGLVL_ERR,
            c"server_address_new",
            133,
            "truncated server address: {head}..."
        );
    }
    XString::from_vec(address)
}

/// Whether one of this instance's sockets is the file at `address`.
///
/// Compared after resolving both sides, so a relative path or a symlink to a
/// socket this process opened still counts.
pub fn server_owns_pipe_address(address: &CStr) -> bool {
    let path = fix_fname(address);
    WATCHERS.with(|watchers| {
        watchers
            .iter()
            .any(|watcher| fix_fname(watcher.address()) == path)
    })
}

/// Starts listening on `addr`.
///
/// Returns 0 on success, 1 for an empty address, 2 when this instance is
/// already listening there, and a negative libuv error otherwise.
pub fn server_start(addr: Option<&CStr>) -> c_int {
    let Some(addr) = addr.filter(|addr| !addr.is_empty()) else {
        logmsg!(LOGLVL_WRN, c"server_start", 169, "Empty or NULL address");
        return 1;
    };

    // A bare name is not an address: it names a socket to create in the
    // runtime directory.
    let generated = is_bare_server_name(addr.to_bytes()).then(|| server_address_new(Some(addr)));
    let endpoint = generated.as_ref().map_or(addr, XString::as_cstr);
    let mut watcher = match Listener::new(endpoint) {
        Ok(watcher) => watcher,
        Err(result) => return result,
    };

    // `Listener::new` resolves the address, so duplicates are only
    // detectable now — after a generated name has become a path and a TCP
    // endpoint has picked up its port.
    let already_listening = WATCHERS.with(|watchers| {
        watchers
            .iter()
            .any(|other| other.address() == watcher.address())
    });
    if already_listening {
        logmsg!(
            LOGLVL_ERR,
            c"server_start",
            186,
            "Already listening on {}",
            msg_cstr(watcher.address())
        );
        watcher.discard();
        return 2;
    }

    let result = watcher.start(MAX_CONNECTIONS, connection_cb);
    if result < 0 {
        logmsg!(
            LOGLVL_WRN,
            c"server_start",
            197,
            "Failed to start server: {}: {}",
            msg_cstr(os_strerror(result)),
            msg_cstr(watcher.address())
        );
        watcher.close();
        return result;
    }

    WATCHERS.with_mut(|watchers| watchers.push(watcher));
    if with_vim_var_str(Vv::Servername, CStr::is_empty) {
        set_vservername();
    }
    0
}

/// Stops listening on `endpoint`, reporting whether it was one of ours.
///
/// `keep_vservername` is for the restart `:detach` performs, where the name
/// is about to be reused.
pub fn server_stop(endpoint: &CStr, keep_vservername: bool) -> bool {
    // Truncated the same way a watcher's own address is, so an over-long
    // endpoint still matches the socket it opened.
    let addr = endpoint.to_bytes();
    let addr = &addr[..addr.len().min(SOCKET_ADDR_LEN - 1)];

    let found = WATCHERS.with_mut(|watchers| {
        let index = watchers
            .iter()
            .position(|watcher| watcher.address().to_bytes() == addr)?;
        // Order beyond the first entry does not matter, and the first is only
        // reachable here when it is the one being removed.
        Some(watchers.swap_remove(index))
    });
    let Some(watcher) = found else {
        logmsg!(
            LOGLVL_WRN,
            c"server_stop",
            236,
            "Not listening on {}",
            msg_bytes(addr)
        );
        return false;
    };

    watcher.close();
    let renamed =
        !keep_vservername && with_vim_var_str(Vv::Servername, |name| name.to_bytes() == addr);
    if renamed {
        set_vservername();
    }
    true
}

/// Every address this instance is listening on, for `serverlist()`.
pub fn server_addresses() -> Vec<CString> {
    WATCHERS.with(|watchers| {
        watchers
            .iter()
            .map(|watcher| watcher.address().to_owned())
            .collect()
    })
}

/// libuv accepted (or failed to accept) a connection on one of our sockets.
fn connection_cb(watcher: &mut SocketWatcher, result: c_int) {
    if result != 0 {
        logmsg!(
            LOGLVL_ERR,
            c"connection_cb",
            276,
            "Failed to accept connection: {}",
            msg_cstr(os_strerror(result))
        );
        return;
    }
    channel_from_connection(watcher);
}
