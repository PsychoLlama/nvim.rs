//! Listening on and connecting to sockets — TCP endpoints and local pipes.
//!
//! A [`SocketWatcher`] is a listening socket. Its address decides which kind:
//! anything that parses as `host:port` becomes a `uv_tcp_t`, everything else
//! a `uv_pipe_t` (a Unix domain socket). Incoming connections are reported as
//! events, like every other watcher here.
//!
//! [`socket_connect`] is the other direction, and is *synchronous*: it drives
//! the main loop itself until the connection succeeds, fails or times out.
//!
//! libuv holds the address of a watcher in its handle's `data` for as long as
//! the watcher listens, so a watcher is always reached through a raw pointer
//! that outlives any borrow of it. [`Watcher`] wraps that pointer once —
//! paying the `unsafe` at construction — and leaves the field accesses below
//! as ordinary Rust.

#![deny(unsafe_op_in_unsafe_fn)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

pub mod address;

use crate::message_fmt::c_str;
use crate::os::uv_error::{UV_EACCES, UV_EADDRINUSE, UV_EINVAL, UV_ENOENT};
use core::ffi::{CStr, c_char, c_int, c_void};
use core::ops::{Deref, DerefMut};
use core::{mem, ptr};

use crate::charset::try_getdigits;
use crate::event::libuv::{
    uv_accept, uv_close, uv_freeaddrinfo, uv_getaddrinfo, uv_listen, uv_pipe_bind, uv_pipe_connect,
    uv_pipe_init, uv_strerror, uv_tcp_bind, uv_tcp_connect, uv_tcp_getsockname, uv_tcp_init,
    uv_tcp_nodelay,
};
use crate::event::r#loop::{one_arg_event, process_events_until};
use crate::event::multiqueue::multiqueue_put_event;
use crate::event::rstream::Reader;
use crate::event::socket::address::{SOCKET_ADDR_LEN, port_suffix, tcp_host_end};
use crate::event::stream::{may_close, stream_init, stream_may_close};
use crate::event::{pack_int, unpack_int};
use crate::log::{LOGLVL_ERR, LOGLVL_INF, LOGLVL_WRN, logmsg};
use crate::main::main_loop;
use crate::memory::{xfree, xstrdup, xstrlcpy};
use crate::os::cshim::gettext;
use crate::os::fs::{os_path_exists, os_remove};
use crate::path::path_tail;
use crate::types::{
    Loop, RStream, SocketWatcher, Stream, addrinfo, intmax_t, sa_family_t, socket_cb,
    socket_close_cb, uv_connect_t, uv_getaddrinfo_t, uv_handle_t, uv_handle_type, uv_loop_t,
    uv_pipe_t, uv_stream_t, uv_tcp_t,
};
use ::libc::ntohs;

const UV_TCP: uv_handle_type = 12;

const AF_UNSPEC: c_int = 0;
const SOCK_STREAM: c_int = 1;
const AI_NUMERICSERV: c_int = 0x400;

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

/// A listening socket this module is working with, plus the promise that the
/// pointer behind it stays live for as long as the handle does.
#[derive(Copy, Clone)]
struct Watcher(*mut SocketWatcher);

impl Watcher {
    /// # Safety
    /// `watcher` is non-null and points at a live `SocketWatcher` for the
    /// whole life of the handle and of everything derived from it.
    unsafe fn new(watcher: *mut SocketWatcher) -> Self {
        debug_assert!(!watcher.is_null());
        Watcher(watcher)
    }

    /// The pointer back, for the C-shaped callees that still want one.
    fn as_ptr(self) -> *mut SocketWatcher {
        self.0
    }

    /// The address buffer, as the NUL-terminated string everything here
    /// treats it as.
    fn addr_ptr(self) -> *mut c_char {
        // SAFETY: a field of the live watcher.
        unsafe { (&raw mut (*self.0).addr).cast() }
    }

    /// The TCP arm of the handle union.
    fn tcp(self) -> *mut uv_tcp_t {
        // SAFETY: a field of the live watcher; the caller picks the arm.
        unsafe { &raw mut (*self.0).uv.tcp.handle }
    }

    /// The pipe arm of the handle union.
    fn pipe(self) -> *mut uv_pipe_t {
        // SAFETY: a field of the live watcher; the caller picks the arm.
        unsafe { &raw mut (*self.0).uv.pipe.handle }
    }

    /// What the host lookup produced, for a TCP watcher.
    fn addrinfo(self) -> *mut addrinfo {
        // SAFETY: a field of the live watcher, set by `socket_watcher_init`.
        unsafe { (*self.0).uv.tcp.addrinfo }
    }

    /// Is this a TCP endpoint rather than a local socket? The handle libuv
    /// built is what says so.
    fn is_tcp(self) -> bool {
        let stream = self.stream;
        // SAFETY: the handle `socket_watcher_init` initialised.
        unsafe { (*stream).type_0 == UV_TCP }
    }
}

impl Deref for Watcher {
    type Target = SocketWatcher;

    fn deref(&self) -> &SocketWatcher {
        // SAFETY: the promise made at construction.
        unsafe { &*self.0 }
    }
}

impl DerefMut for Watcher {
    fn deref_mut(&mut self) -> &mut SocketWatcher {
        // SAFETY: the promise made at construction.
        unsafe { &mut *self.0 }
    }
}

/// The libuv loop inside one of the editor's.
///
/// # Safety
/// `uv_loop` is a live `Loop`.
unsafe fn uv_of(uv_loop: *mut Loop) -> *mut uv_loop_t {
    // SAFETY: the caller's promise.
    unsafe { &raw mut (*uv_loop).uv }
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
///
/// # Safety
/// `watcher` is a live, uninitialised `SocketWatcher`, `uv_loop` is a live
/// loop, and `endpoint` is a NUL-terminated string.
pub unsafe fn socket_watcher_init(
    uv_loop: *mut Loop,
    watcher: *mut SocketWatcher,
    endpoint: *const c_char,
) -> c_int {
    // SAFETY: the caller's promise.
    let mut watcher = unsafe { Watcher::new(watcher) };
    let addr = watcher.addr_ptr();
    // SAFETY: the watcher's own buffer, and the caller's endpoint.
    unsafe { xstrlcpy(addr, endpoint, SOCKET_ADDR_LEN) };

    // SAFETY: `xstrlcpy` always terminates.
    let host_end = tcp_host_end(unsafe { CStr::from_ptr(addr) }.to_bytes());
    match host_end {
        Some(host_end) => {
            // Split the address in place; the port half is re-appended by
            // socket_watcher_start once the kernel has assigned one.
            // SAFETY: `host_end` indexes the string just measured.
            unsafe { *addr.add(host_end) = 0 };
            // SAFETY: the port half starts one past the separator.
            let mut port = unsafe { addr.add(host_end + 1) };

            let mut iport = 0;
            let mut scan = port;
            // SAFETY: `scan` walks the NUL-terminated port half.
            let ok = unsafe { try_getdigits(&raw mut scan, &raw mut iport) };
            if !ok || iport < 0 || iport > intmax_t::from(u16::MAX) {
                logmsg!(
                    LOGLVL_ERR,
                    c"socket_watcher_init",
                    62,
                    "Invalid port: {}",
                    unsafe { c_str(port) }
                );
                return UV_EINVAL;
            }
            // SAFETY: `port` points into the buffer.
            if unsafe { *port } == 0 {
                // getaddrinfo wants a null service rather than an empty one,
                // or it tries to look the name up and fails.
                port = ptr::null_mut();
            }

            // SAFETY: `addrinfo` is inhabited by the all-zero bit pattern.
            let zeroed: addrinfo = unsafe { mem::zeroed() };
            let hints = addrinfo {
                ai_family: AF_UNSPEC,
                ai_socktype: SOCK_STREAM,
                ..zeroed
            };
            // SAFETY: as above, for the request block.
            let mut request: uv_getaddrinfo_t = unsafe { mem::zeroed() };
            // SAFETY: the caller's loop, a request on this frame, and the two
            // halves of the watcher's own address buffer.
            let retval = unsafe {
                let uv = uv_of(uv_loop);
                uv_getaddrinfo(uv, &raw mut request, None, addr, port, &raw const hints)
            };
            if retval != 0 {
                logmsg!(
                    LOGLVL_ERR,
                    c"socket_watcher_init",
                    78,
                    "Host lookup failed: {}",
                    unsafe { c_str(endpoint) }
                );
                return retval;
            }
            // SAFETY: the watcher's own union, whose TCP arm this picks.
            unsafe { (*watcher.as_ptr()).uv.tcp.addrinfo = request.addrinfo };
            let handle = watcher.tcp();
            // SAFETY: the caller's loop and the handle just chosen.
            unsafe {
                uv_tcp_init(uv_of(uv_loop), handle);
                uv_tcp_nodelay(handle, 1);
            }
            watcher.stream = handle.cast();
        }
        None => {
            let handle = watcher.pipe();
            // SAFETY: the caller's loop and the handle just chosen.
            unsafe { uv_pipe_init(uv_of(uv_loop), handle, 0) };
            watcher.stream = handle.cast();
        }
    }

    let stream = watcher.stream;
    // SAFETY: the handle initialised just above.
    unsafe { (*stream).data = watcher.as_ptr().cast() };
    watcher.cb = None;
    watcher.close_cb = None;
    watcher.events = ptr::null_mut();
    watcher.data = ptr::null_mut();
    0
}

/// Bind and listen, reporting each connection to `cb`.
///
/// Returns a libuv error code. A TCP watcher tries every address the lookup
/// produced; a pipe watcher gets one retry, after clearing a socket file left
/// behind by an nvim that is no longer running.
///
/// # Safety
/// `watcher` has been through [`socket_watcher_init`] and has not been closed.
pub unsafe fn socket_watcher_start(
    watcher: *mut SocketWatcher,
    backlog: c_int,
    cb: socket_cb,
) -> c_int {
    // SAFETY: the caller's promise.
    let mut watcher = unsafe { Watcher::new(watcher) };
    watcher.cb = cb;
    let addr = watcher.addr_ptr();
    let stream = watcher.stream;
    let mut result = UV_EINVAL;

    if watcher.is_tcp() {
        let mut ai = watcher.addrinfo();
        while !ai.is_null() {
            // SAFETY: the watcher's own handle, and one lookup result.
            result = unsafe { uv_tcp_bind(watcher.tcp(), (*ai).ai_addr, 0) };
            if result == 0 {
                // SAFETY: the watcher's own listening handle.
                result = unsafe { uv_listen(stream, backlog, Some(connection_cb)) };
                if result == 0 {
                    record_bound_port(watcher);
                    break;
                }
            }
            // SAFETY: walking the lookup's own list.
            ai = unsafe { (*ai).ai_next };
        }
        // SAFETY: the list the lookup allocated for this watcher.
        unsafe { uv_freeaddrinfo(watcher.addrinfo()) };
    } else {
        // SAFETY: the watcher's own handle and address buffer.
        result = unsafe { uv_pipe_bind(watcher.pipe(), addr) };
        if result == UV_EACCES || result == UV_EADDRINUSE {
            result = rebind_stale_socket(watcher, result);
        }
        if result == 0 {
            // SAFETY: the watcher's own listening handle.
            result = unsafe { uv_listen(watcher.stream, backlog, Some(connection_cb)) };
        }
    }

    debug_assert!(result <= 0, "libuv returns a negative error code or zero");
    if result == UV_EACCES {
        // libuv reports a missing parent directory as EACCES, for Windows
        // compatibility. ENOENT is the more useful answer.
        // SAFETY: the watcher's own NUL-terminated address buffer.
        unsafe { *path_tail(addr) = 0 };
        // SAFETY: as above.
        if !unsafe { os_path_exists(addr) } {
            result = UV_ENOENT;
        }
    }
    result
}

/// Append the port the kernel assigned to the watcher's address.
///
/// An endpoint given without a port binds to a free one, and `v:servername`
/// is the address string, so it has to be told.
fn record_bound_port(watcher: Watcher) {
    // SAFETY: `sockaddr_storage` is inhabited by the all-zero bit pattern.
    let mut sas: sockaddr_storage = unsafe { mem::zeroed() };
    let size = size_of::<sockaddr_storage>();
    let mut len = c_int::try_from(size).expect("a socket address fits in an int");
    // SAFETY: the watcher's own handle, and a buffer on this frame.
    unsafe { uv_tcp_getsockname(watcher.tcp(), (&raw mut sas).cast(), &raw mut len) };
    // SAFETY: both address families put the port right after the family, and
    // both structs are `repr(C)`.
    let port = ntohs(unsafe { (*(&raw const sas).cast::<sockaddr_head>()).port });

    let addr = watcher.addr_ptr();
    // SAFETY: the watcher's own NUL-terminated address buffer.
    let used = unsafe { CStr::from_ptr(addr) }.to_bytes().len();
    let suffix = port_suffix(port);
    // SAFETY: appending within the buffer's own bound.
    unsafe {
        xstrlcpy(
            addr.add(used),
            suffix.as_ptr().cast(),
            SOCKET_ADDR_LEN - used,
        )
    };
}

/// A pipe bind failed because the socket file is already there. If nothing is
/// listening on it, remove it and bind again; the handle has to be closed and
/// re-created first, because libuv will not reuse one that failed to bind.
///
/// Returns the new bind result, or `failure` unchanged if the socket is live
/// or could not be removed.
fn rebind_stale_socket(mut watcher: Watcher, failure: c_int) -> c_int {
    let addr = watcher.addr_ptr();
    let stream = watcher.stream;
    // SAFETY: the watcher's handle, which libuv gave the editor's loop.
    let uv_loop: *mut Loop = unsafe { (*(*stream).loop_0).data }.cast();

    // SAFETY: the loop the watcher's handle belongs to, and its address.
    if unsafe { socket_alive(uv_loop, addr) } {
        logmsg!(
            LOGLVL_ERR,
            c"socket_watcher_start",
            203,
            "Socket already in use by another Nvim instance: {}",
            unsafe { c_str(addr) }
        );
        return failure;
    }

    logmsg!(
        LOGLVL_INF,
        c"socket_watcher_start",
        180,
        "Removing stale socket: {}",
        unsafe { c_str(addr) }
    );
    // SAFETY: the watcher's own address.
    let rm_result = unsafe { os_remove(addr) };
    if rm_result != 0 {
        // SAFETY: libuv's error strings are static.
        let why = unsafe { uv_strerror(rm_result) };
        logmsg!(
            LOGLVL_WRN,
            c"socket_watcher_start",
            185,
            "Failed to remove stale socket {}: {}",
            unsafe { c_str(addr) },
            unsafe { c_str(why) }
        );
        return failure;
    }

    let handle = watcher.pipe();
    // SAFETY: the watcher's own handle.
    let handle_loop = unsafe { (*handle).loop_0 };
    let mut closed = false;
    // SAFETY: as above; `closed` outlives the wait below.
    unsafe { (*handle).data = (&raw mut closed).cast() };
    // SAFETY: as above.
    unsafe { uv_close(handle.cast(), Some(early_server_close_cb)) };
    // SAFETY: the main loop is live for as long as the editor is.
    unsafe { process_events_until(main_loop.ptr(), ptr::null_mut(), -1, || closed) };

    // SAFETY: a fresh handle, on the loop the closed one used.
    unsafe { uv_pipe_init(handle_loop, handle, 0) };
    watcher.stream = handle.cast();
    // SAFETY: the handle just re-initialised.
    unsafe { (*handle).data = watcher.as_ptr().cast() };
    // SAFETY: as above, with the watcher's own address.
    unsafe { uv_pipe_bind(handle, addr) }
}

/// Is anything listening on `addr`? Probed by connecting to it, with a 500ms
/// timeout so a dead socket is diagnosed quickly.
///
/// # Safety
/// `uv_loop` is a live loop and `addr` is a NUL-terminated socket path.
unsafe fn socket_alive(uv_loop: *mut Loop, addr: *const c_char) -> bool {
    // SAFETY: `RStream` is inhabited by the all-zero bit pattern.
    let mut stream: RStream = unsafe { mem::zeroed() };
    let mut error: *const c_char = ptr::null();
    let probe = &raw mut stream;
    // SAFETY: the caller's loop and address, and a stream on this frame.
    let up = unsafe { socket_connect(uv_loop, probe, false, addr, 500, &raw mut error) };
    if !up {
        return false;
    }

    // Take the probe connection back down before answering: the stream is on
    // this stack frame.
    let mut closed = false;
    stream.s.internal_close_cb = Some(connect_close_cb);
    stream.s.internal_data = (&raw mut closed).cast();
    // SAFETY: the probe stream, still on this frame.
    unsafe { stream_may_close(&raw mut stream.s) };
    // SAFETY: the main loop is live for as long as the editor is.
    unsafe { process_events_until(main_loop.ptr(), ptr::null_mut(), -1, || closed) };
    true
}

/// Hand an accepted connection to `stream`, which the caller owns.
///
/// # Safety
/// `watcher` is listening, and `stream` is a live `RStream` the caller keeps
/// alive for as long as the connection lasts.
pub unsafe fn socket_watcher_accept(watcher: *mut SocketWatcher, stream: *mut RStream) -> c_int {
    // SAFETY: the caller's promise.
    let watcher = unsafe { Watcher::new(watcher) };
    // SAFETY: as above.
    let conn = unsafe { Reader::new(stream) }.conn();
    let client: *mut uv_stream_t = if watcher.is_tcp() {
        // SAFETY: the caller's stream, on the watcher handle's own loop.
        unsafe {
            let client = conn.tcp();
            uv_tcp_init((*watcher.tcp()).loop_0, client);
            uv_tcp_nodelay(client, 1);
            client.cast()
        }
    } else {
        // SAFETY: as above.
        unsafe {
            let client = conn.pipe();
            uv_pipe_init((*watcher.pipe()).loop_0, client, 0);
            client.cast()
        }
    };

    // SAFETY: the watcher's listening handle and the client just built.
    let result = unsafe { uv_accept(watcher.stream, client) };
    if result != 0 {
        // SAFETY: the client handle, which nothing else holds yet.
        unsafe { uv_close(client.cast(), None) };
        return result;
    }
    // SAFETY: the caller's stream, now holding the accepted connection.
    unsafe { stream_init(ptr::null_mut(), conn.as_ptr(), -1, client) };
    0
}

/// Stop listening; `cb` is told once libuv is done with the handle.
///
/// # Safety
/// `watcher` has been through [`socket_watcher_init`] and stays live until
/// `cb` runs.
pub unsafe fn socket_watcher_close(watcher: *mut SocketWatcher, cb: socket_close_cb) {
    // SAFETY: the caller's promise.
    let mut watcher = unsafe { Watcher::new(watcher) };
    watcher.close_cb = cb;
    let stream = watcher.stream;
    // SAFETY: the watcher's own listening handle.
    unsafe { uv_close(stream.cast(), Some(close_cb)) };
}

// ---------------------------------------------------------------------------
// Connecting
// ---------------------------------------------------------------------------

/// Connect `stream` to `address`, driving the loop until it settles.
///
/// `timeout` is in milliseconds. On failure `*error` is set to a message the
/// caller may show; a TCP address that resolved to several candidates is
/// tried in turn before that happens.
///
/// # Safety
/// `uv_loop` is live, `stream` is a live `RStream` the caller keeps alive
/// until this returns, `address` is NUL-terminated and `error` is writable.
pub unsafe fn socket_connect(
    uv_loop: *mut Loop,
    stream: *mut RStream,
    is_tcp: bool,
    address: *const c_char,
    timeout: c_int,
    error: *mut *const c_char,
) -> bool {
    // SAFETY: the caller's stream, live until this returns.
    let mut conn = unsafe { Reader::new(stream) }.conn();
    // SAFETY: `uv_getaddrinfo_t` is inhabited by the all-zero bit pattern.
    let mut addr_req: uv_getaddrinfo_t = unsafe { mem::zeroed() };
    let mut addr: *mut c_char = ptr::null_mut();
    let mut success = false;
    let fail = move |msg: &'static CStr| {
        // SAFETY: the caller's slot, and `gettext` answers a static string.
        unsafe { *error = gettext(msg).as_ptr() };
    };

    'settled: {
        let mut candidate: *const addrinfo = ptr::null();
        if is_tcp {
            // SAFETY: the caller's NUL-terminated address.
            addr = unsafe { xstrdup(address) };
            // SAFETY: `xstrdup` copies the terminator too.
            let bytes = unsafe { CStr::from_ptr(addr) }.to_bytes();
            let Some(host_end) = bytes.iter().rposition(|&b| b == b':') else {
                fail(c"tcp address must be host:port");
                break 'settled;
            };
            // SAFETY: `host_end` indexes the copy just measured.
            unsafe { *addr.add(host_end) = 0 };

            // SAFETY: `addrinfo` is inhabited by the all-zero bit pattern.
            let zeroed: addrinfo = unsafe { mem::zeroed() };
            let hints = addrinfo {
                ai_flags: AI_NUMERICSERV,
                ai_family: AF_UNSPEC,
                ai_socktype: SOCK_STREAM,
                ..zeroed
            };
            // SAFETY: the caller's loop, a request on this frame, and the
            // host and port halves of the copy above.
            let looked_up = unsafe {
                let (uv, port) = (uv_of(uv_loop), addr.add(host_end + 1));
                uv_getaddrinfo(uv, &raw mut addr_req, None, addr, port, &raw const hints)
            };
            if looked_up != 0 {
                fail(c"failed to lookup host or port");
                break 'settled;
            }
            candidate = addr_req.addrinfo;
        }

        let mut status: c_int = 1;
        let mut closed = false;
        // SAFETY: `uv_connect_t` is inhabited by the all-zero bit pattern.
        let mut req: uv_connect_t = unsafe { mem::zeroed() };
        req.data = (&raw mut status).cast();

        loop {
            let uv_stream: *mut uv_stream_t = if is_tcp {
                // SAFETY: the caller's stream and loop, and one candidate
                // from the lookup above.
                unsafe {
                    let tcp = conn.tcp();
                    uv_tcp_init(uv_of(uv_loop), tcp);
                    uv_tcp_nodelay(tcp, 1);
                    uv_tcp_connect(&raw mut req, tcp, (*candidate).ai_addr, Some(connect_cb));
                    tcp.cast()
                }
            } else {
                // SAFETY: the caller's stream, loop and address.
                unsafe {
                    let pipe = conn.pipe();
                    uv_pipe_init(uv_of(uv_loop), pipe, 0);
                    uv_pipe_connect(&raw mut req, pipe, address, Some(connect_cb));
                    pipe.cast()
                }
            };
            // SAFETY: the caller's stream, now holding the connect attempt.
            unsafe { stream_init(ptr::null_mut(), conn.as_ptr(), -1, uv_stream) };
            conn.internal_close_cb = Some(connect_close_cb);
            conn.internal_data = (&raw mut closed).cast();
            closed = false;
            status = 1;

            let ms = i64::from(timeout);
            // SAFETY: the main loop is live for as long as the editor is.
            unsafe { process_events_until(main_loop.ptr(), ptr::null_mut(), ms, || status != 1) };
            if status == 0 {
                success = true;
                break 'settled;
            }

            may_close(conn);
            // Wait for the close callback before retrying or returning:
            // `stream` may be on the caller's stack.
            // SAFETY: the main loop is live for as long as the editor is.
            unsafe { process_events_until(main_loop.ptr(), ptr::null_mut(), -1, || closed) };

            // The short circuit is load-bearing: a pipe connect never looked
            // an address up, so `candidate` is still null here.
            // SAFETY: for TCP, the candidate is one of the lookup's entries.
            if !is_tcp || unsafe { (*candidate).ai_next }.is_null() {
                fail(c"connection refused");
                break 'settled;
            }
            // SAFETY: as above.
            candidate = unsafe { (*candidate).ai_next };
        }
    }

    conn.internal_close_cb = None;
    conn.internal_data = ptr::null_mut();
    // SAFETY: the two allocations made above.
    unsafe {
        xfree(addr.cast());
        uv_freeaddrinfo(addr_req.addrinfo);
    }
    success
}

// ---------------------------------------------------------------------------
// Callbacks
// ---------------------------------------------------------------------------

/// libuv: a client is waiting. `status` rides in the event's second argument
/// as an integer, which is what upstream's `CREATE_EVENT` does with it.
///
/// # Safety
/// libuv's: `handle` is a listening handle whose `data` is its watcher.
unsafe extern "C" fn connection_cb(handle: *mut uv_stream_t, status: c_int) {
    // SAFETY: the caller's promise.
    let watcher = unsafe { Watcher::new((*handle).data.cast()) };
    let status = pack_int(status);
    if watcher.events.is_null() {
        let mut argv = [watcher.as_ptr().cast::<c_void>(), status];
        // SAFETY: exactly the argv `connection_event` reads.
        unsafe { connection_event(argv.as_mut_ptr()) };
    } else {
        let mut event = one_arg_event(Some(connection_event), watcher.as_ptr().cast());
        event.argv[1] = status;
        // SAFETY: the watcher's own queue.
        unsafe { multiqueue_put_event(watcher.events, event) };
    }
}

/// # Safety
/// Queued by [`connection_cb`]: `argv` is the watcher and its libuv status.
unsafe extern "C" fn connection_event(argv: *mut *mut c_void) {
    // SAFETY: the caller's promise about the argv.
    let watcher = unsafe { Watcher::new((*argv).cast()) };
    // SAFETY: as above.
    let status = unpack_int(unsafe { *argv.add(1) });
    let notify = watcher.cb.expect("a started watcher has a callback");
    // SAFETY: the watcher's own callback, given the watcher.
    unsafe { notify(watcher.as_ptr(), status, watcher.data) };
}

/// libuv: the listening handle is closed.
///
/// # Safety
/// libuv's: `handle` is a listening handle whose `data` is its watcher.
unsafe extern "C" fn close_cb(handle: *mut uv_handle_t) {
    // SAFETY: the caller's promise.
    let watcher = unsafe { Watcher::new((*handle).data.cast()) };
    if let Some(notify) = watcher.close_cb {
        // SAFETY: the watcher's own callback, given the watcher.
        unsafe { notify(watcher.as_ptr(), watcher.data) };
    }
}

/// libuv: the connect request settled. A failure also takes the stream down,
/// so the caller only has to wait for the close.
///
/// # Safety
/// libuv's: `req` is the request [`socket_connect`] made, whose `data` is its
/// status slot.
unsafe extern "C" fn connect_cb(req: *mut uv_connect_t, status: c_int) {
    // SAFETY: the caller's promise.
    unsafe { *(*req).data.cast::<c_int>() = status };
    if status != 0 {
        // SAFETY: the stream layer owns the connected handle's `data`.
        unsafe { stream_may_close((*(*req).handle).data.cast::<Stream>()) };
    }
}

/// A stream opened by [`socket_connect`] finished closing.
///
/// # Safety
/// `data` is the `bool` the opener is waiting on.
unsafe fn connect_close_cb(_stream: *mut Stream, data: *mut c_void) {
    // SAFETY: the caller's promise.
    unsafe { *data.cast::<bool>() = true };
}

/// A listening handle being discarded before it ever listened finished
/// closing (see [`rebind_stale_socket`]).
///
/// # Safety
/// libuv's: `handle`'s `data` is the `bool` the rebind is waiting on.
unsafe extern "C" fn early_server_close_cb(handle: *mut uv_handle_t) {
    // SAFETY: the caller's promise.
    unsafe { *(*handle).data.cast::<bool>() = true };
}
