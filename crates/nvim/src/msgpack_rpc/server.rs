#![deny(unsafe_op_in_unsafe_fn)]
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

use core::ffi::{CStr, c_char, c_int, c_void};

use crate::channel::channel_from_connection;
use crate::eval::vars::{get_vim_var_str, set_vim_var_string};
use crate::event::libuv::{uv_freeaddrinfo, uv_strerror};
use crate::event::socket::address::is_bare_server_name;
use crate::event::socket::{socket_watcher_close, socket_watcher_init, socket_watcher_start};
use crate::global_cell::GlobalCell;
use crate::log::{LOGLVL_ERR, LOGLVL_WRN, logmsg};
use crate::main::{IObuff, NameBuff, main_loop};
use crate::memory::{strequal, xcalloc, xfree, xmalloc, xstrdup};
use crate::os::cshim::snprintf;
use crate::os::env::{os_env_exists, os_get_pid, os_getenv, os_unsetenv};
use crate::os::stdpaths::{get_appname, stdpaths_get_xdg_var};
use crate::path::fix_fname;
use crate::types::{IOSIZE, SocketWatcher, Vv, size_t, uint32_t};

use crate::event::socket::address::SOCKET_ADDR_LEN;

/// Values these belong to other modules; nested so they stay out of the flat
/// namespace the unit-test header generator collects constants into.
mod known {
    use core::ffi::{c_int, c_uint};

    /// `XDGVarType` of the runtime directory generated addresses live in.
    pub(super) const XDG_RUNTIME_DIR: c_int = 4;
    /// libuv's handle type for a TCP socket.
    pub(super) const UV_TCP: c_uint = 12;
    /// How many connections the kernel may queue behind a listening socket.
    pub(super) const MAX_CONNECTIONS: c_int = 32;
}

use known::*;

const ENV_LISTEN: &CStr = c"NVIM_LISTEN_ADDRESS";

/// Every socket this instance is listening on. The first one is what
/// `v:servername` reports.
static WATCHERS: GlobalCell<Vec<*mut SocketWatcher>> = GlobalCell::new(Vec::new());

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
/// Returns whether that succeeded. An address the user did not ask for is
/// generated, and failing to listen on *that* is not an error — the editor
/// runs fine without a server.
///
/// # Safety
/// `listen_addr` is either null or a NUL-terminated string.
pub unsafe fn server_init(listen_addr: *const c_char) -> bool {
    let mut listen_addr = listen_addr;
    let mut must_free = false;
    // Which of the three sources the address came from, in the order they
    // are consulted.
    let mut source = ListenSource::Argument;

    // SAFETY: the caller's address, and the environment, which is this
    // process's own.
    let empty = listen_addr.is_null() || unsafe { *listen_addr == 0 };
    if empty {
        // SAFETY: `ENV_LISTEN` is a static string.
        if unsafe { os_env_exists(ENV_LISTEN.as_ptr(), true) } {
            source = ListenSource::Environment;
            // SAFETY: as above.
            listen_addr = unsafe { os_getenv(ENV_LISTEN.as_ptr()) };
        } else {
            source = ListenSource::Generated;
            // SAFETY: a null name means "use the editor's own".
            listen_addr = unsafe { server_address_new(core::ptr::null()) };
        }
        must_free = true;
    }

    // SAFETY: `listen_addr` is null or NUL-terminated either way.
    let rv = unsafe { server_start(listen_addr) };

    // SAFETY: a static name, and a message with no arguments.
    unsafe {
        if os_env_exists(c"__NVIM_TEST_LOG".as_ptr(), false) {
            logmsg!(LOGLVL_ERR, c"server_init", 58, c"test log message");
        }
    }

    let mut ok = true;
    if rv != 0 && source != ListenSource::Generated {
        let fmt = if source == ListenSource::Argument {
            c"Failed to --listen: %s: \"%s\""
        } else {
            c"Failed $NVIM_LISTEN_ADDRESS: %s: \"%s\""
        };
        // SAFETY: `uv_strerror` answers a static string for any code, and
        // `IObuff` is `IOSIZE` writable bytes; both verbs take a string.
        unsafe {
            let reason = if rv < 0 {
                uv_strerror(rv)
            } else if rv == 1 {
                c"empty address".as_ptr()
            } else {
                c"?".as_ptr()
            };
            snprintf(
                IObuff.ptr().cast::<c_char>(),
                IOSIZE as usize,
                fmt.as_ptr(),
                reason,
                listen_addr,
            );
        }
        ok = false;
    }

    // The variable exists to tell *this* process where to listen; child
    // processes must not inherit it.
    // SAFETY: a static name, and an address this function owns when
    // `must_free` says so.
    unsafe {
        if os_env_exists(ENV_LISTEN.as_ptr(), false) {
            os_unsetenv(ENV_LISTEN.as_ptr());
        }
        if must_free {
            xfree(listen_addr.cast_mut().cast::<c_void>());
        }
    }
    ok
}

/// A watcher's address, as the NUL-terminated string it stores.
///
/// # Safety
/// `watcher` points at a live `SocketWatcher`.
unsafe fn watcher_addr(watcher: *mut SocketWatcher) -> *mut c_char {
    // SAFETY: the caller's watcher; `addr` is a fixed-size array inside it.
    unsafe { (&raw mut (*watcher).addr).cast::<c_char>() }
}

/// Points `v:servername` at whatever is now the first listening socket, or
/// clears it when there is none.
fn set_vservername() {
    let default_server = WATCHERS.with(|watchers| match watchers.first() {
        // SAFETY: every watcher in the list is live until `server_stop` or
        // `server_teardown` removes it.
        Some(&watcher) => unsafe { watcher_addr(watcher) },
        None => core::ptr::null_mut(),
    });
    // SAFETY: the address is either null or the watcher's own string.
    unsafe { set_vim_var_string(Vv::Servername, default_server, -1) };
}

pub fn server_teardown() {
    let listening = WATCHERS.with_mut(core::mem::take);
    for watcher in listening {
        // SAFETY: the list owned each watcher, and `free_server` releases it
        // once libuv is done with it.
        unsafe { socket_watcher_close(watcher, Some(free_server)) };
    }
}

/// Generates an address in the runtime directory, unique to this process.
///
/// `name` names the socket; without one the editor's own name is used, which
/// `get_appname` has just written into `NameBuff`.
///
/// # Safety
/// `name` is either null or a NUL-terminated string.
pub unsafe fn server_address_new(name: *const c_char) -> *mut c_char {
    static SEQUENCE: GlobalCell<uint32_t> = GlobalCell::new(0);

    let mut address = [0 as c_char; SOCKET_ADDR_LEN];
    let sequence = SEQUENCE.get();
    SEQUENCE.set(sequence.wrapping_add(1));

    // SAFETY: `stdpaths_get_xdg_var` answers an owned string, `get_appname`
    // fills `NameBuff` with one, the caller's `name` is the other candidate,
    // and `address` is `SOCKET_ADDR_LEN` writable bytes.
    let written = unsafe {
        let dir = stdpaths_get_xdg_var(XDG_RUNTIME_DIR);
        get_appname(true);
        let base = if name.is_null() {
            NameBuff.ptr().cast::<c_char>().cast_const()
        } else {
            name
        };
        let written = snprintf(
            address.as_mut_ptr(),
            SOCKET_ADDR_LEN,
            c"%s/%s.%lu.%u".as_ptr(),
            dir,
            base,
            os_get_pid(),
            sequence,
        );
        xfree(dir.cast::<c_void>());
        written
    };
    // `snprintf` answers how many bytes it *wanted*, or a negative on an
    // output error; upstream's bare `(size_t)` cast made the latter a huge
    // number, which takes this branch, and so does this.
    let truncated = usize::try_from(written)
        .ok()
        .is_none_or(|w| w >= SOCKET_ADDR_LEN);
    if truncated {
        // SAFETY: `address` is NUL-terminated and the verb takes a string.
        unsafe {
            logmsg!(
                LOGLVL_ERR,
                c"server_address_new",
                133,
                c"truncated server address: %.40s...",
                address.as_mut_ptr(),
            );
        }
    }
    // SAFETY: `address` is NUL-terminated.
    unsafe { xstrdup(address.as_ptr()) }
}

/// Whether one of this instance's sockets is the file at `address`.
///
/// Compared after resolving both sides, so a relative path or a symlink to a
/// socket this process opened still counts.
///
/// # Safety
/// `address` is a NUL-terminated string.
pub unsafe fn server_owns_pipe_address(address: *const c_char) -> bool {
    // SAFETY: the caller's address; `fix_fname` answers an owned string.
    let path = unsafe { fix_fname(address) };
    let owned = WATCHERS.with(|watchers| {
        watchers.iter().any(|&watcher| {
            // SAFETY: a live watcher, and two owned strings.
            unsafe {
                let addr = fix_fname(watcher_addr(watcher));
                let same = strequal(path, addr);
                xfree(addr.cast::<c_void>());
                same
            }
        })
    });
    // SAFETY: the string `fix_fname` handed over.
    unsafe { xfree(path.cast::<c_void>()) };
    owned
}

/// Starts listening on `addr`.
///
/// Returns 0 on success, 1 for an empty address, 2 when this instance is
/// already listening there, and a negative libuv error otherwise.
///
/// # Safety
/// `addr` is either null or a NUL-terminated string.
pub unsafe fn server_start(addr: *const c_char) -> c_int {
    // SAFETY: the caller's address.
    if addr.is_null() || unsafe { *addr == 0 } {
        // SAFETY: a message with no arguments.
        unsafe {
            logmsg!(LOGLVL_WRN, c"server_start", 169, c"Empty or NULL address");
        }
        return 1;
    }

    // A bare name is not an address: it names a socket to create in the
    // runtime directory.
    // SAFETY: the caller's address, which is NUL-terminated and non-empty.
    let generated = unsafe {
        if is_bare_server_name(CStr::from_ptr(addr).to_bytes()) {
            server_address_new(addr)
        } else {
            core::ptr::null_mut()
        }
    };
    // SAFETY: `xmalloc` hands back `size_of::<SocketWatcher>()` writable
    // bytes, which `socket_watcher_init` fills in.
    let (watcher, result) = unsafe {
        let watcher = xmalloc(size_of::<SocketWatcher>()).cast::<SocketWatcher>();
        let result = socket_watcher_init(
            main_loop.ptr(),
            watcher,
            if generated.is_null() { addr } else { generated },
        );
        xfree(generated.cast::<c_void>());
        (watcher, result)
    };
    if result < 0 {
        // SAFETY: nothing took the allocation.
        unsafe { xfree(watcher.cast::<c_void>()) };
        return result;
    }

    // `socket_watcher_init` resolves the address, so duplicates are only
    // detectable now — after a generated name has become a path and a TCP
    // endpoint has picked up its port.
    let already_listening = WATCHERS.with(|watchers| {
        watchers.iter().any(|&other| {
            // SAFETY: both watchers are live and hold NUL-terminated
            // addresses.
            unsafe { strequal(watcher_addr(watcher), watcher_addr(other)) }
        })
    });
    if already_listening {
        // SAFETY: the watcher is live and its address is NUL-terminated; a
        // TCP watcher owns the addrinfo `socket_watcher_init` resolved.
        unsafe {
            logmsg!(
                LOGLVL_ERR,
                c"server_start",
                186,
                c"Already listening on %s",
                watcher_addr(watcher),
            );
            if (*(*watcher).stream).type_0 == UV_TCP {
                uv_freeaddrinfo((*watcher).uv.tcp.addrinfo);
            }
            socket_watcher_close(watcher, Some(free_server));
        }
        return 2;
    }

    // SAFETY: the watcher is live and `connection_cb` is handed it back.
    let result = unsafe { socket_watcher_start(watcher, MAX_CONNECTIONS, Some(connection_cb)) };
    if result < 0 {
        // SAFETY: the watcher is still live, and `free_server` releases it
        // once libuv is done.
        unsafe {
            logmsg!(
                LOGLVL_WRN,
                c"server_start",
                197,
                c"Failed to start server: %s: %s",
                uv_strerror(result),
                watcher_addr(watcher),
            );
            socket_watcher_close(watcher, Some(free_server));
        }
        return result;
    }

    WATCHERS.with_mut(|watchers| watchers.push(watcher));
    // SAFETY: `v:servername` is a live vim variable holding a string.
    if unsafe { *get_vim_var_str(Vv::Servername) == 0 } {
        set_vservername();
    }
    0
}

/// Stops listening on `endpoint`, reporting whether it was one of ours.
///
/// `keep_vservername` is for the restart `:detach` performs, where the name
/// is about to be reused.
///
/// # Safety
/// `endpoint` is a NUL-terminated string.
pub unsafe fn server_stop(endpoint: *const c_char, keep_vservername: bool) -> bool {
    // Truncated the same way a watcher's own address is, so an over-long
    // endpoint still matches the socket it opened.
    let mut addr = [0 as c_char; SOCKET_ADDR_LEN];
    // SAFETY: the caller's endpoint, copied into a buffer that is one byte
    // longer than the copy so the terminator the zero-fill left survives.
    unsafe {
        let endpoint_len = CStr::from_ptr(endpoint)
            .to_bytes()
            .len()
            .min(SOCKET_ADDR_LEN - 1);
        addr.as_mut_ptr()
            .copy_from_nonoverlapping(endpoint, endpoint_len);
    }

    let found = WATCHERS.with_mut(|watchers| {
        let index = watchers.iter().position(|&watcher| {
            // SAFETY: a live watcher, and a NUL-terminated local buffer.
            unsafe { strequal(addr.as_ptr().cast_mut(), watcher_addr(watcher)) }
        })?;
        // Order beyond the first entry does not matter, and the first is only
        // reachable here when it is the one being removed.
        Some(watchers.swap_remove(index))
    });
    let Some(watcher) = found else {
        // SAFETY: `addr` is NUL-terminated and the verb takes a string.
        unsafe {
            logmsg!(
                LOGLVL_WRN,
                c"server_stop",
                236,
                c"Not listening on %s",
                addr.as_mut_ptr(),
            );
        }
        return false;
    };

    // SAFETY: the list owned the watcher and `free_server` releases it;
    // `v:servername` is a live string.
    let renamed = unsafe {
        socket_watcher_close(watcher, Some(free_server));
        !keep_vservername && strequal(addr.as_ptr().cast_mut(), get_vim_var_str(Vv::Servername))
    };
    if renamed {
        set_vservername();
    }
    true
}

/// Every address this instance is listening on, as an owned array of owned
/// strings for `serverlist()`.
///
/// # Safety
/// `size` points at a writable `size_t`.
pub unsafe fn server_address_list(size: *mut size_t) -> *mut *mut c_char {
    WATCHERS.with(|watchers| {
        // SAFETY: the caller's out-parameter.
        unsafe { *size = watchers.len() };
        if watchers.is_empty() {
            return core::ptr::null_mut();
        }
        // SAFETY: `xcalloc` hands back one writable slot per watcher, and
        // every watcher's address is NUL-terminated.
        unsafe {
            let addrs = xcalloc(watchers.len(), size_of::<*const c_char>()).cast::<*mut c_char>();
            for (i, &watcher) in watchers.iter().enumerate() {
                *addrs.add(i) = xstrdup(watcher_addr(watcher));
            }
            addrs
        }
    })
}

/// libuv accepted (or failed to accept) a connection on one of our sockets.
///
/// # Safety
/// `watcher` is the live watcher the connection arrived on.
unsafe fn connection_cb(watcher: *mut SocketWatcher, result: c_int, _data: *mut c_void) {
    if result != 0 {
        // SAFETY: `uv_strerror` answers a static string for any code.
        unsafe {
            logmsg!(
                LOGLVL_ERR,
                c"connection_cb",
                276,
                c"Failed to accept connection: %s",
                uv_strerror(result),
            );
        }
        return;
    }
    // SAFETY: the caller's watcher, which has a pending connection.
    unsafe { channel_from_connection(watcher) };
}

/// Releases a watcher once libuv has finished closing it.
///
/// # Safety
/// `watcher` was allocated by [`server_start`] and libuv is done with it.
unsafe fn free_server(watcher: *mut SocketWatcher, _data: *mut c_void) {
    // SAFETY: the caller's guarantee that nothing else holds it.
    unsafe { xfree(watcher.cast::<c_void>()) };
}
