//! The listening side of msgpack-rpc: the sockets `--listen`,
//! `serverstart()` and `$NVIM_LISTEN_ADDRESS` open, and the channels that
//! come out of them.

use core::ffi::{CStr, c_char, c_int, c_void};

use crate::src::nvim::channel::channel_from_connection;
use crate::src::nvim::eval::vars::{get_vim_var_str, set_vim_var_string};
use crate::src::nvim::event::libuv::{uv_freeaddrinfo, uv_strerror};
use crate::src::nvim::event::socket::address::is_bare_server_name;
use crate::src::nvim::event::socket::{
    socket_watcher_close, socket_watcher_init, socket_watcher_start,
};
use crate::src::nvim::global_cell::GlobalCell;
use crate::src::nvim::log::{LOGLVL_ERR, LOGLVL_WRN, logmsg};
use crate::src::nvim::main::{IObuff, NameBuff, main_loop};
use crate::src::nvim::memory::{strequal, xcalloc, xfree, xmalloc, xstrdup};
use crate::src::nvim::os::env::{os_env_exists, os_get_pid, os_getenv, os_unsetenv};
use crate::src::nvim::os::libc::snprintf;
use crate::src::nvim::os::stdpaths::{get_appname, stdpaths_get_xdg_var};
use crate::src::nvim::path::fix_fname;
use crate::src::nvim::types::{
    SocketWatcher, VV_SEND_SERVER, kFalse, kNone, kTrue, size_t, uint32_t,
};

use crate::src::nvim::event::socket::address::SOCKET_ADDR_LEN;

/// Values these belong to other modules; nested so they stay out of the flat
/// namespace the unit-test header generator collects constants into.
mod known {
    use core::ffi::{c_int, c_uint};

    /// `VimVarIndex` of `v:servername`.
    pub const VV_SEND_SERVER: c_uint = 28;
    /// `XDGVarType` of the runtime directory generated addresses live in.
    pub const XDG_RUNTIME_DIR: c_int = 4;
    /// libuv's handle type for a TCP socket.
    pub const UV_TCP: c_uint = 12;
    pub const LOGLVL_WRN: c_int = 3;
    pub const LOGLVL_ERR: c_int = 4;
    /// The capacity of `IObuff`.
    pub const IOSIZE: usize = 1025;
    /// How many connections the kernel may queue behind a listening socket.
    pub const MAX_CONNECTIONS: c_int = 32;
}

use known::*;

const ENV_LISTEN: &CStr = c"NVIM_LISTEN_ADDRESS";

/// Every socket this instance is listening on. The first one is what
/// `v:servername` reports.
static WATCHERS: GlobalCell<Vec<*mut SocketWatcher>> = GlobalCell::new(Vec::new());

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
    // Which of the three sources the address came from, in the order they are
    // consulted; `kNone` means nobody asked for one.
    let mut user_arg = kTrue;

    if listen_addr.is_null() || *listen_addr == 0 {
        if os_env_exists(ENV_LISTEN.as_ptr(), true) {
            user_arg = kFalse;
            listen_addr = os_getenv(ENV_LISTEN.as_ptr());
        } else {
            user_arg = kNone;
            listen_addr = server_address_new(core::ptr::null());
        }
        must_free = true;
    }

    let rv = server_start(listen_addr);

    if os_env_exists(c"__NVIM_TEST_LOG".as_ptr(), false) {
        logmsg(
            LOGLVL_ERR,
            core::ptr::null(),
            c"server_init".as_ptr(),
            58,
            true,
            c"test log message".as_ptr(),
        );
    }

    let mut ok = true;
    if rv != 0 && user_arg != kNone {
        let reason = if rv < 0 {
            uv_strerror(rv)
        } else if rv == 1 {
            c"empty address".as_ptr()
        } else {
            c"?".as_ptr()
        };
        snprintf(
            IObuff.ptr().cast::<c_char>(),
            IOSIZE,
            if user_arg == kTrue {
                c"Failed to --listen: %s: \"%s\"".as_ptr()
            } else {
                c"Failed $NVIM_LISTEN_ADDRESS: %s: \"%s\"".as_ptr()
            },
            reason,
            listen_addr,
        );
        ok = false;
    }

    // The variable exists to tell *this* process where to listen; child
    // processes must not inherit it.
    if os_env_exists(ENV_LISTEN.as_ptr(), false) {
        os_unsetenv(ENV_LISTEN.as_ptr());
    }
    if must_free {
        xfree(listen_addr as *mut c_void);
    }
    ok
}

/// A watcher's address, as the NUL-terminated string it stores.
unsafe fn watcher_addr(watcher: *mut SocketWatcher) -> *mut c_char {
    (&raw mut (*watcher).addr).cast::<c_char>()
}

/// Points `v:servername` at whatever is now the first listening socket, or
/// clears it when there is none.
fn set_vservername() {
    let default_server = WATCHERS.with(|watchers| match watchers.first() {
        Some(&watcher) => unsafe { watcher_addr(watcher) },
        None => core::ptr::null_mut(),
    });
    unsafe { set_vim_var_string(VV_SEND_SERVER, default_server, -1) };
}

pub fn server_teardown() {
    let listening = WATCHERS.with_mut(core::mem::take);
    for watcher in listening {
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
    let dir = stdpaths_get_xdg_var(XDG_RUNTIME_DIR);
    get_appname(true);
    let sequence = SEQUENCE.get();
    SEQUENCE.set(sequence.wrapping_add(1));

    let written = snprintf(
        address.as_mut_ptr(),
        SOCKET_ADDR_LEN,
        c"%s/%s.%lu.%u".as_ptr(),
        dir,
        if name.is_null() {
            NameBuff.ptr().cast::<c_char>().cast_const()
        } else {
            name
        },
        os_get_pid(),
        sequence,
    );
    xfree(dir.cast::<c_void>());
    if written as size_t >= SOCKET_ADDR_LEN {
        logmsg(
            LOGLVL_ERR,
            core::ptr::null(),
            c"server_address_new".as_ptr(),
            133,
            true,
            c"truncated server address: %.40s...".as_ptr(),
            address.as_mut_ptr(),
        );
    }
    xstrdup(address.as_ptr())
}

/// Whether one of this instance's sockets is the file at `address`.
///
/// Compared after resolving both sides, so a relative path or a symlink to a
/// socket this process opened still counts.
///
/// # Safety
/// `address` is a NUL-terminated string.
pub unsafe fn server_owns_pipe_address(address: *const c_char) -> bool {
    let path = fix_fname(address);
    let owned = WATCHERS.with(|watchers| {
        watchers.iter().any(|&watcher| {
            let addr = fix_fname(watcher_addr(watcher));
            let same = strequal(path, addr);
            xfree(addr.cast::<c_void>());
            same
        })
    });
    xfree(path.cast::<c_void>());
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
    if addr.is_null() || *addr == 0 {
        logmsg(
            LOGLVL_WRN,
            core::ptr::null(),
            c"server_start".as_ptr(),
            169,
            true,
            c"Empty or NULL address".as_ptr(),
        );
        return 1;
    }

    // A bare name is not an address: it names a socket to create in the
    // runtime directory.
    let generated = if is_bare_server_name(CStr::from_ptr(addr).to_bytes()) {
        server_address_new(addr)
    } else {
        core::ptr::null_mut()
    };
    let watcher: *mut SocketWatcher = xmalloc(size_of::<SocketWatcher>()).cast::<SocketWatcher>();
    let result = socket_watcher_init(
        main_loop.ptr(),
        watcher,
        if generated.is_null() { addr } else { generated },
    );
    xfree(generated.cast::<c_void>());
    if result < 0 {
        xfree(watcher.cast::<c_void>());
        return result;
    }

    // `socket_watcher_init` resolves the address, so duplicates are only
    // detectable now — after a generated name has become a path and a TCP
    // endpoint has picked up its port.
    let already_listening = WATCHERS.with(|watchers| {
        watchers
            .iter()
            .any(|&other| strequal(watcher_addr(watcher), watcher_addr(other)))
    });
    if already_listening {
        logmsg(
            LOGLVL_ERR,
            core::ptr::null(),
            c"server_start".as_ptr(),
            186,
            true,
            c"Already listening on %s".as_ptr(),
            watcher_addr(watcher),
        );
        if (*(*watcher).stream).type_0 == UV_TCP {
            uv_freeaddrinfo((*watcher).uv.tcp.addrinfo);
        }
        socket_watcher_close(watcher, Some(free_server));
        return 2;
    }

    let result = socket_watcher_start(watcher, MAX_CONNECTIONS, Some(connection_cb));
    if result < 0 {
        logmsg(
            LOGLVL_WRN,
            core::ptr::null(),
            c"server_start".as_ptr(),
            197,
            true,
            c"Failed to start server: %s: %s".as_ptr(),
            uv_strerror(result),
            watcher_addr(watcher),
        );
        socket_watcher_close(watcher, Some(free_server));
        return result;
    }

    WATCHERS.with_mut(|watchers| watchers.push(watcher));
    if *get_vim_var_str(VV_SEND_SERVER) == 0 {
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
    let endpoint_len = CStr::from_ptr(endpoint)
        .to_bytes()
        .len()
        .min(SOCKET_ADDR_LEN - 1);
    addr.as_mut_ptr()
        .copy_from_nonoverlapping(endpoint, endpoint_len);

    let found = WATCHERS.with_mut(|watchers| {
        let index = watchers
            .iter()
            .position(|&watcher| strequal(addr.as_ptr().cast_mut(), watcher_addr(watcher)))?;
        // Order beyond the first entry does not matter, and the first is only
        // reachable here when it is the one being removed.
        Some(watchers.swap_remove(index))
    });
    let Some(watcher) = found else {
        logmsg(
            LOGLVL_WRN,
            core::ptr::null(),
            c"server_stop".as_ptr(),
            236,
            true,
            c"Not listening on %s".as_ptr(),
            addr.as_mut_ptr(),
        );
        return false;
    };

    socket_watcher_close(watcher, Some(free_server));
    if !keep_vservername && strequal(addr.as_ptr().cast_mut(), get_vim_var_str(VV_SEND_SERVER)) {
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
        *size = watchers.len();
        if watchers.is_empty() {
            return core::ptr::null_mut();
        }
        let addrs = xcalloc(watchers.len(), size_of::<*const c_char>()).cast::<*mut c_char>();
        for (i, &watcher) in watchers.iter().enumerate() {
            *addrs.add(i) = xstrdup(watcher_addr(watcher));
        }
        addrs
    })
}

unsafe extern "C" fn connection_cb(watcher: *mut SocketWatcher, result: c_int, _data: *mut c_void) {
    if result != 0 {
        logmsg(
            LOGLVL_ERR,
            core::ptr::null(),
            c"connection_cb".as_ptr(),
            276,
            true,
            c"Failed to accept connection: %s".as_ptr(),
            uv_strerror(result),
        );
        return;
    }
    channel_from_connection(watcher);
}

unsafe extern "C" fn free_server(watcher: *mut SocketWatcher, _data: *mut c_void) {
    xfree(watcher.cast::<c_void>());
}
