//! Listening on and connecting to sockets — TCP endpoints and local pipes.
//!
//! A [`SocketWatcher`] is a listening socket. Its address decides which kind:
//! anything that parses as `host:port` becomes a `uv_tcp_t`, everything else
//! a `uv_pipe_t` (a Unix domain socket). Incoming connections are reported as
//! events, like every other watcher here.
//!
//! [`socket_connect`] is the other direction, and is *synchronous*: it drives
//! the main loop itself until the connection succeeds, fails or times out.

pub mod address;

use core::ffi::{CStr, c_char, c_int, c_void};
use core::{mem, ptr};

use crate::src::nvim::charset::try_getdigits;
use crate::src::nvim::event::libuv::{
    uv_accept, uv_close, uv_freeaddrinfo, uv_listen, uv_pipe_bind, uv_pipe_connect, uv_pipe_init,
    uv_strerror, uv_tcp_bind, uv_tcp_connect, uv_tcp_getsockname, uv_tcp_init, uv_tcp_nodelay,
};
use crate::src::nvim::event::r#loop::{one_arg_event, process_events_until};
use crate::src::nvim::event::multiqueue::multiqueue_put_event;
use crate::src::nvim::event::socket::address::{SOCKET_ADDR_LEN, port_suffix, tcp_host_end};
use crate::src::nvim::event::stream::{stream_init, stream_may_close};
use crate::src::nvim::log::{LOGLVL_ERR, LOGLVL_INF, LOGLVL_WRN, logmsg};
use crate::src::nvim::main::main_loop;
use crate::src::nvim::memory::{xfree, xstrdup, xstrlcpy};
use crate::src::nvim::os::fs::{os_path_exists, os_remove};
use crate::src::nvim::os::libc::{gettext, ntohs};
use crate::src::nvim::path::path_tail;
use crate::src::nvim::types::{
    Loop, RStream, SocketWatcher, Stream, addrinfo, intmax_t, sa_family_t, socket_cb,
    socket_close_cb, uv__work, uv_connect_t, uv_handle_t, uv_handle_type, uv_loop_t, uv_pipe_t,
    uv_req_type, uv_stream_t, uv_tcp_t,
};

const UV_TCP: uv_handle_type = 12;

const UV_EINVAL: c_int = -22;
const UV_ENOENT: c_int = -2;
const UV_EACCES: c_int = -13;
const UV_EADDRINUSE: c_int = -98;

const AF_UNSPEC: c_int = 0;
const SOCK_STREAM: c_int = 1;
const AI_NUMERICSERV: c_int = 0x400;

unsafe extern "C" {
    fn uv_getaddrinfo(
        uv_loop: *mut uv_loop_t,
        req: *mut uv_getaddrinfo_t,
        getaddrinfo_cb: uv_getaddrinfo_cb,
        node: *const c_char,
        service: *const c_char,
        hints: *const addrinfo,
    ) -> c_int;
}

pub type uv_getaddrinfo_cb =
    Option<unsafe extern "C" fn(*mut uv_getaddrinfo_t, c_int, *mut addrinfo)>;
pub type uv_getaddrinfo_t = uv_getaddrinfo_s;

#[derive(Copy, Clone)]
#[repr(C)]
pub struct uv_getaddrinfo_s {
    pub data: *mut c_void,
    pub type_0: uv_req_type,
    pub reserved: [*mut c_void; 6],
    pub loop_0: *mut uv_loop_t,
    pub work_req: uv__work,
    pub cb: uv_getaddrinfo_cb,
    pub hints: *mut addrinfo,
    pub hostname: *mut c_char,
    pub service: *mut c_char,
    pub addrinfo: *mut addrinfo,
    pub retcode: c_int,
}

/// The buffer `uv_tcp_getsockname` fills in.
#[derive(Copy, Clone)]
#[repr(C)]
pub struct sockaddr_storage {
    pub ss_family: sa_family_t,
    pub __ss_padding: [c_char; 118],
    pub __ss_align: ::core::ffi::c_ulong,
}

/// Just enough of `sockaddr_in`/`sockaddr_in6` to read a bound port: both put
/// it, in network order, immediately after the address family. Nothing here
/// looks any further, so the family does not have to be inspected at all.
#[derive(Copy, Clone)]
#[repr(C)]
pub struct sockaddr_head {
    pub family: sa_family_t,
    pub port: u16,
}

/// Does `address` name a TCP endpoint rather than a local socket?
pub fn socket_address_is_tcp(address: &CStr) -> bool {
    tcp_host_end(address.to_bytes()).is_some()
}

// ---------------------------------------------------------------------------
// Listening
// ---------------------------------------------------------------------------

/// Prepare `watcher` to listen on `endpoint`, resolving it if it is TCP.
///
/// Returns a libuv error code; the host lookup is the only thing that can
/// fail here, binding and listening happen in [`socket_watcher_start`].
pub unsafe fn socket_watcher_init(
    uv_loop: *mut Loop,
    watcher: *mut SocketWatcher,
    endpoint: *const c_char,
) -> c_int {
    let addr = (&raw mut (*watcher).addr).cast::<c_char>();
    xstrlcpy(addr, endpoint, SOCKET_ADDR_LEN);

    match tcp_host_end(CStr::from_ptr(addr).to_bytes()) {
        Some(host_end) => {
            // Split the address in place; the port half is re-appended by
            // socket_watcher_start once the kernel has assigned one.
            *addr.add(host_end) = 0;
            let mut port = addr.add(host_end + 1);

            let mut iport = 0;
            let mut scan = port;
            let ok = try_getdigits(&raw mut scan, &raw mut iport);
            if !ok || iport < 0 || iport > u16::MAX as intmax_t {
                logmsg(
                    LOGLVL_ERR,
                    ptr::null(),
                    c"socket_watcher_init".as_ptr(),
                    62,
                    true,
                    c"Invalid port: %s".as_ptr(),
                    port,
                );
                return UV_EINVAL;
            }
            if *port == 0 {
                // getaddrinfo wants a null service rather than an empty one,
                // or it tries to look the name up and fails.
                port = ptr::null_mut();
            }

            let hints = addrinfo {
                ai_family: AF_UNSPEC,
                ai_socktype: SOCK_STREAM,
                ..mem::zeroed()
            };
            let mut request: uv_getaddrinfo_t = mem::zeroed();
            let retval = uv_getaddrinfo(
                &raw mut (*uv_loop).uv,
                &raw mut request,
                None,
                addr,
                port,
                &raw const hints,
            );
            if retval != 0 {
                logmsg(
                    LOGLVL_ERR,
                    ptr::null(),
                    c"socket_watcher_init".as_ptr(),
                    78,
                    true,
                    c"Host lookup failed: %s".as_ptr(),
                    endpoint,
                );
                return retval;
            }
            (*watcher).uv.tcp.addrinfo = request.addrinfo;
            uv_tcp_init(&raw mut (*uv_loop).uv, &raw mut (*watcher).uv.tcp.handle);
            uv_tcp_nodelay(&raw mut (*watcher).uv.tcp.handle, 1);
            (*watcher).stream = (&raw mut (*watcher).uv.tcp.handle).cast();
        }
        None => {
            uv_pipe_init(
                &raw mut (*uv_loop).uv,
                &raw mut (*watcher).uv.pipe.handle,
                0,
            );
            (*watcher).stream = (&raw mut (*watcher).uv.pipe.handle).cast();
        }
    }

    (*(*watcher).stream).data = watcher.cast();
    (*watcher).cb = None;
    (*watcher).close_cb = None;
    (*watcher).events = ptr::null_mut();
    (*watcher).data = ptr::null_mut();
    0
}

/// Bind and listen, reporting each connection to `cb`.
///
/// Returns a libuv error code. A TCP watcher tries every address the lookup
/// produced; a pipe watcher gets one retry, after clearing a socket file left
/// behind by an nvim that is no longer running.
pub unsafe fn socket_watcher_start(
    watcher: *mut SocketWatcher,
    backlog: c_int,
    cb: socket_cb,
) -> c_int {
    (*watcher).cb = cb;
    let addr = (&raw mut (*watcher).addr).cast::<c_char>();
    let mut result = UV_EINVAL;

    if (*(*watcher).stream).type_0 == UV_TCP {
        let mut ai = (*watcher).uv.tcp.addrinfo;
        while !ai.is_null() {
            result = uv_tcp_bind(&raw mut (*watcher).uv.tcp.handle, (*ai).ai_addr, 0);
            if result == 0 {
                result = uv_listen((*watcher).stream, backlog, Some(connection_cb));
                if result == 0 {
                    record_bound_port(watcher);
                    break;
                }
            }
            ai = (*ai).ai_next;
        }
        uv_freeaddrinfo((*watcher).uv.tcp.addrinfo);
    } else {
        result = uv_pipe_bind(&raw mut (*watcher).uv.pipe.handle, addr);
        if result == UV_EACCES || result == UV_EADDRINUSE {
            result = rebind_stale_socket(watcher, result);
        }
        if result == 0 {
            result = uv_listen((*watcher).stream, backlog, Some(connection_cb));
        }
    }

    debug_assert!(result <= 0, "libuv returns a negative error code or zero");
    if result == UV_EACCES {
        // libuv reports a missing parent directory as EACCES, for Windows
        // compatibility. ENOENT is the more useful answer.
        *path_tail(addr) = 0;
        if !os_path_exists(addr) {
            result = UV_ENOENT;
        }
    }
    result
}

/// Append the port the kernel assigned to the watcher's address.
///
/// An endpoint given without a port binds to a free one, and `v:servername`
/// is the address string, so it has to be told.
unsafe fn record_bound_port(watcher: *mut SocketWatcher) {
    let mut sas: sockaddr_storage = mem::zeroed();
    let mut len = size_of::<sockaddr_storage>() as c_int;
    uv_tcp_getsockname(
        &raw mut (*watcher).uv.tcp.handle,
        (&raw mut sas).cast(),
        &raw mut len,
    );
    let port = ntohs((*(&raw const sas).cast::<sockaddr_head>()).port);

    let addr = (&raw mut (*watcher).addr).cast::<c_char>();
    let used = CStr::from_ptr(addr).to_bytes().len();
    xstrlcpy(
        addr.add(used),
        port_suffix(port).as_ptr().cast(),
        SOCKET_ADDR_LEN - used,
    );
}

/// A pipe bind failed because the socket file is already there. If nothing is
/// listening on it, remove it and bind again; the handle has to be closed and
/// re-created first, because libuv will not reuse one that failed to bind.
///
/// Returns the new bind result, or `failure` unchanged if the socket is live
/// or could not be removed.
unsafe fn rebind_stale_socket(watcher: *mut SocketWatcher, failure: c_int) -> c_int {
    let addr = (&raw mut (*watcher).addr).cast::<c_char>();
    let uv_loop: *mut Loop = (*(*(*watcher).stream).loop_0).data.cast();

    if socket_alive(uv_loop, addr) {
        logmsg(
            LOGLVL_ERR,
            ptr::null(),
            c"socket_watcher_start".as_ptr(),
            203,
            true,
            c"Socket already in use by another Nvim instance: %s".as_ptr(),
            addr,
        );
        return failure;
    }

    logmsg(
        LOGLVL_INF,
        ptr::null(),
        c"socket_watcher_start".as_ptr(),
        180,
        true,
        c"Removing stale socket: %s".as_ptr(),
        addr,
    );
    let rm_result = os_remove(addr);
    if rm_result != 0 {
        logmsg(
            LOGLVL_WRN,
            ptr::null(),
            c"socket_watcher_start".as_ptr(),
            185,
            true,
            c"Failed to remove stale socket %s: %s".as_ptr(),
            addr,
            uv_strerror(rm_result),
        );
        return failure;
    }

    let handle_loop = (*watcher).uv.pipe.handle.loop_0;
    let mut closed = false;
    (*watcher).uv.pipe.handle.data = (&raw mut closed).cast();
    uv_close(
        (&raw mut (*watcher).uv.pipe.handle).cast(),
        Some(early_server_close_cb),
    );
    process_events_until(main_loop.ptr(), ptr::null_mut(), -1, || closed);

    uv_pipe_init(handle_loop, &raw mut (*watcher).uv.pipe.handle, 0);
    (*watcher).stream = (&raw mut (*watcher).uv.pipe.handle).cast();
    (*(*watcher).stream).data = watcher.cast();
    uv_pipe_bind(&raw mut (*watcher).uv.pipe.handle, addr)
}

/// Is anything listening on `addr`? Probed by connecting to it, with a 500ms
/// timeout so a dead socket is diagnosed quickly.
unsafe fn socket_alive(uv_loop: *mut Loop, addr: *const c_char) -> bool {
    let mut stream: RStream = mem::zeroed();
    let mut error: *const c_char = ptr::null();
    if !socket_connect(uv_loop, &raw mut stream, false, addr, 500, &raw mut error) {
        return false;
    }

    // Take the probe connection back down before answering: the stream is on
    // this stack frame.
    let mut closed = false;
    stream.s.internal_close_cb = Some(connect_close_cb);
    stream.s.internal_data = (&raw mut closed).cast();
    stream_may_close(&raw mut stream.s);
    process_events_until(main_loop.ptr(), ptr::null_mut(), -1, || closed);
    true
}

/// Hand an accepted connection to `stream`, which the caller owns.
pub unsafe fn socket_watcher_accept(watcher: *mut SocketWatcher, stream: *mut RStream) -> c_int {
    let client: *mut uv_stream_t = if (*(*watcher).stream).type_0 == UV_TCP {
        let client: *mut uv_tcp_t = &raw mut (*stream).s.uv.tcp;
        uv_tcp_init((*watcher).uv.tcp.handle.loop_0, client);
        uv_tcp_nodelay(client, 1);
        client.cast()
    } else {
        let client: *mut uv_pipe_t = &raw mut (*stream).s.uv.pipe;
        uv_pipe_init((*watcher).uv.pipe.handle.loop_0, client, 0);
        client.cast()
    };

    let result = uv_accept((*watcher).stream, client);
    if result != 0 {
        uv_close(client.cast(), None);
        return result;
    }
    stream_init(ptr::null_mut(), &raw mut (*stream).s, -1, client);
    0
}

/// Stop listening; `cb` is told once libuv is done with the handle.
pub unsafe fn socket_watcher_close(watcher: *mut SocketWatcher, cb: socket_close_cb) {
    (*watcher).close_cb = cb;
    uv_close((*watcher).stream.cast(), Some(close_cb));
}

// ---------------------------------------------------------------------------
// Connecting
// ---------------------------------------------------------------------------

/// Connect `stream` to `address`, driving the loop until it settles.
///
/// `timeout` is in milliseconds. On failure `*error` is set to a message the
/// caller may show; a TCP address that resolved to several candidates is
/// tried in turn before that happens.
pub unsafe fn socket_connect(
    uv_loop: *mut Loop,
    stream: *mut RStream,
    is_tcp: bool,
    address: *const c_char,
    timeout: c_int,
    error: *mut *const c_char,
) -> bool {
    let mut addr_req: uv_getaddrinfo_t = mem::zeroed();
    let mut addr: *mut c_char = ptr::null_mut();
    let mut success = false;

    'settled: {
        let mut candidate: *const addrinfo = ptr::null();
        if is_tcp {
            addr = xstrdup(address);
            let Some(host_end) = CStr::from_ptr(addr)
                .to_bytes()
                .iter()
                .rposition(|&b| b == b':')
            else {
                *error = gettext(c"tcp address must be host:port".as_ptr());
                break 'settled;
            };
            *addr.add(host_end) = 0;

            let hints = addrinfo {
                ai_flags: AI_NUMERICSERV,
                ai_family: AF_UNSPEC,
                ai_socktype: SOCK_STREAM,
                ..mem::zeroed()
            };
            if uv_getaddrinfo(
                &raw mut (*uv_loop).uv,
                &raw mut addr_req,
                None,
                addr,
                addr.add(host_end + 1),
                &raw const hints,
            ) != 0
            {
                *error = gettext(c"failed to lookup host or port".as_ptr());
                break 'settled;
            }
            candidate = addr_req.addrinfo;
        }

        let mut status: c_int = 1;
        let mut closed = false;
        let mut req: uv_connect_t = mem::zeroed();
        req.data = (&raw mut status).cast();

        loop {
            let uv_stream: *mut uv_stream_t = if is_tcp {
                let tcp: *mut uv_tcp_t = &raw mut (*stream).s.uv.tcp;
                uv_tcp_init(&raw mut (*uv_loop).uv, tcp);
                uv_tcp_nodelay(tcp, 1);
                uv_tcp_connect(&raw mut req, tcp, (*candidate).ai_addr, Some(connect_cb));
                tcp.cast()
            } else {
                let pipe: *mut uv_pipe_t = &raw mut (*stream).s.uv.pipe;
                uv_pipe_init(&raw mut (*uv_loop).uv, pipe, 0);
                uv_pipe_connect(&raw mut req, pipe, address, Some(connect_cb));
                pipe.cast()
            };
            stream_init(ptr::null_mut(), &raw mut (*stream).s, -1, uv_stream);
            (*stream).s.internal_close_cb = Some(connect_close_cb);
            (*stream).s.internal_data = (&raw mut closed).cast();
            closed = false;
            status = 1;

            process_events_until(main_loop.ptr(), ptr::null_mut(), timeout as i64, || {
                status != 1
            });
            if status == 0 {
                success = true;
                break 'settled;
            }

            stream_may_close(&raw mut (*stream).s);
            // Wait for the close callback before retrying or returning:
            // `stream` may be on the caller's stack.
            process_events_until(main_loop.ptr(), ptr::null_mut(), -1, || closed);

            if !is_tcp || (*candidate).ai_next.is_null() {
                *error = gettext(c"connection refused".as_ptr());
                break 'settled;
            }
            candidate = (*candidate).ai_next;
        }
    }

    (*stream).s.internal_close_cb = None;
    (*stream).s.internal_data = ptr::null_mut();
    xfree(addr.cast());
    uv_freeaddrinfo(addr_req.addrinfo);
    success
}

// ---------------------------------------------------------------------------
// Callbacks
// ---------------------------------------------------------------------------

/// libuv: a client is waiting. `status` rides in the event's second argument
/// as an integer, which is what upstream's `CREATE_EVENT` does with it.
unsafe extern "C" fn connection_cb(handle: *mut uv_stream_t, status: c_int) {
    let watcher: *mut SocketWatcher = (*handle).data.cast();
    let status = ptr::with_exposed_provenance_mut::<c_void>(status as usize);
    if (*watcher).events.is_null() {
        let mut argv = [watcher.cast::<c_void>(), status];
        connection_event(argv.as_mut_ptr());
    } else {
        let mut event = one_arg_event(Some(connection_event), watcher.cast());
        event.argv[1] = status;
        multiqueue_put_event((*watcher).events, event);
    }
}

unsafe extern "C" fn connection_event(argv: *mut *mut c_void) {
    let watcher: *mut SocketWatcher = (*argv).cast();
    let status = (*argv.add(1)).expose_provenance() as c_int;
    let notify = (*watcher).cb.expect("a started watcher has a callback");
    notify(watcher, status, (*watcher).data);
}

/// libuv: the listening handle is closed.
unsafe extern "C" fn close_cb(handle: *mut uv_handle_t) {
    let watcher: *mut SocketWatcher = (*handle).data.cast();
    if let Some(notify) = (*watcher).close_cb {
        notify(watcher, (*watcher).data);
    }
}

/// libuv: the connect request settled. A failure also takes the stream down,
/// so the caller only has to wait for the close.
unsafe extern "C" fn connect_cb(req: *mut uv_connect_t, status: c_int) {
    *(*req).data.cast::<c_int>() = status;
    if status != 0 {
        stream_may_close((*(*req).handle).data.cast::<Stream>());
    }
}

/// A stream opened by [`socket_connect`] finished closing.
unsafe extern "C" fn connect_close_cb(_stream: *mut Stream, data: *mut c_void) {
    *data.cast::<bool>() = true;
}

/// A listening handle being discarded before it ever listened finished
/// closing (see [`rebind_stale_socket`]).
unsafe extern "C" fn early_server_close_cb(handle: *mut uv_handle_t) {
    *(*handle).data.cast::<bool>() = true;
}
