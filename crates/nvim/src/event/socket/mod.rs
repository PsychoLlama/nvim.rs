//! Listening on and connecting to sockets — TCP endpoints and local pipes.
//!
//! A [`Listener`] is a listening socket. Its address decides which kind
//! ([`Endpoint::parse`]): anything that parses as `host:port` becomes a
//! `uv_tcp_t`, everything else a `uv_pipe_t` (a Unix domain socket).
//! Incoming connections are reported to the callback it was started with.
//!
//! [`socket_connect`] is the other direction, and is *synchronous*: it drives
//! the main loop itself until the connection succeeds, fails or times out.
//!
//! libuv holds the address of a listener's handle, and the handle's `data`
//! points back at the listener, for as long as it listens, so a listener is
//! always reached through a raw pointer that outlives any borrow of it.
//! [`Watcher`] wraps that pointer once — paying the `unsafe` at construction
//! — and reaches each field by place, never through a reference to the whole
//! watcher, whose handle libuv writes behind any such borrow.

#![deny(unsafe_op_in_unsafe_fn)]
// Unsafe perimeter: the `event/` row in docs/perimeter.md.
#![allow(unsafe_code)]

pub mod address;

use crate::message_fmt::{msg_bytes, msg_cstr};
use crate::os::uv_error::{UV_EACCES, UV_EADDRINUSE, UV_EINVAL, UV_ENOENT};
use core::cell::Cell;
use core::ffi::{CStr, c_int, c_uint, c_void};
use core::{mem, ptr};

use crate::event::libuv::{
    uv_accept, uv_close, uv_freeaddrinfo, uv_getaddrinfo, uv_listen, uv_pipe_bind2,
    uv_pipe_connect, uv_pipe_init, uv_tcp_bind, uv_tcp_connect, uv_tcp_getsockname, uv_tcp_init,
    uv_tcp_nodelay,
};
use crate::event::r#loop::{Drain, main_uv_loop, wait_on_main_loop};
use crate::event::objects::Stream;
use crate::event::rstream::ReadStream;
use crate::event::socket::address::{Endpoint, InvalidPort, tcp_host_end};
use crate::event::stream::{may_close, stream_init, stream_may_close};
use crate::event::uv::{
    addrinfo, sockaddr, uv_connect_t, uv_getaddrinfo_t, uv_handle_t, uv_pipe_t, uv_stream_t,
    uv_tcp_t,
};
use crate::log::{LOGLVL_ERR, LOGLVL_INF, LOGLVL_WRN, logmsg};
use crate::os::cshim::gettext;
use crate::os::fs::{os_remove, os_strerror, path_exists};
use crate::os::libc::sa_family_t;
use crate::path::tail_index;
use ::libc::ntohs;
use std::ffi::CString;

/// `uv_pipe_bind2`: fail on a name too long for a socket path, rather than
/// truncating it.
const UV_PIPE_NO_TRUNCATE: c_uint = 1;

const AF_UNSPEC: c_int = 0;
const SOCK_STREAM: c_int = 1;
const AI_NUMERICSERV: c_int = 0x400;

/// Just enough of `sockaddr_in`/`sockaddr_in6` to read a bound port: both put
/// it, in network order, immediately after the address family. Nothing here
/// looks any further, so the family does not have to be inspected at all.
#[repr(C)]
#[derive(Debug)]
struct SockaddrHead {
    family: sa_family_t,
    port: u16,
}

/// A finished host lookup: the list `uv_getaddrinfo` allocated, freed when
/// this is dropped.
#[derive(Debug)]
struct Lookup(*mut addrinfo);

impl Lookup {
    /// Resolve `host` and `service` (a null one when `None`) on the editor's
    /// loop, synchronously; libuv's error otherwise.
    fn resolve(host: &CStr, service: Option<&CStr>, flags: c_int) -> Result<Self, c_int> {
        // SAFETY: `addrinfo` is inhabited by the all-zero bit pattern.
        let zeroed: addrinfo = unsafe { mem::zeroed() };
        let hints = addrinfo {
            ai_flags: flags,
            ai_family: AF_UNSPEC,
            ai_socktype: SOCK_STREAM,
            ..zeroed
        };
        // SAFETY: as above, for the request block.
        let mut request: uv_getaddrinfo_t = unsafe { mem::zeroed() };
        let service = service.map_or(ptr::null(), CStr::as_ptr);
        // SAFETY: the editor's loop, a request on this frame that libuv fills
        // before returning (no callback), and strings the caller holds.
        let result = unsafe {
            let (uv, node) = (main_uv_loop(), host.as_ptr());
            uv_getaddrinfo(uv, &raw mut request, None, node, service, &raw const hints)
        };
        if result == 0 {
            Ok(Lookup(request.addrinfo))
        } else {
            Err(result)
        }
    }

    /// Each address the lookup produced, in the resolver's order.
    fn addresses(&self) -> impl Iterator<Item = *const sockaddr> + '_ {
        let mut next = self.0.cast_const();
        core::iter::from_fn(move || {
            // SAFETY: null, or an entry of the list this lookup owns.
            let entry = unsafe { next.as_ref() }?;
            next = entry.ai_next;
            Some(entry.ai_addr.cast_const())
        })
    }
}

impl Drop for Lookup {
    fn drop(&mut self) {
        // SAFETY: the list `resolve` made, which nothing else frees.
        unsafe { uv_freeaddrinfo(self.0) };
    }
}

/// A listener's libuv handle; [`SocketWatcher::tcp`] says which arm.
#[repr(C)]
union ListenHandle {
    tcp: uv_tcp_t,
    pipe: uv_pipe_t,
}
crate::untagged_union_debug!(ListenHandle);

/// What a connection is reported to: the listener lent for the call, and
/// libuv's status for the connection.
type ConnectionCb = fn(&Incoming<'_>, c_int);

/// A listening socket, boxed by [`Listener::new`] and never moved: libuv
/// holds the address of `uv`, whose `data` points back here.
#[derive(Debug)]
struct SocketWatcher {
    /// The address it reports (`v:servername`, `serverlist()`): a local
    /// socket's path; a TCP endpoint's host, and from the bind on
    /// `host:port` with the port the kernel assigned.
    address: CString,
    uv: ListenHandle,
    /// Whether `uv` holds a TCP handle rather than a pipe.
    tcp: bool,
    /// A TCP endpoint's resolved addresses, until [`Listener::start`] binds
    /// one of them.
    lookup: Option<Lookup>,
    /// Told of each connection, from [`Listener::start`] on.
    cb: Option<ConnectionCb>,
}

/// A listening socket this module is working with, plus the promise that the
/// pointer behind it stays live for as long as the handle does.
#[derive(Copy, Clone, Debug)]
struct Watcher(*mut SocketWatcher);

impl Watcher {
    /// # Safety
    /// `watcher` is non-null and points at a live `SocketWatcher` for the
    /// whole life of the handle and of everything derived from it.
    unsafe fn new(watcher: *mut SocketWatcher) -> Self {
        debug_assert!(!watcher.is_null());
        Watcher(watcher)
    }

    /// The pointer back, for libuv's `data`.
    fn as_ptr(self) -> *mut SocketWatcher {
        self.0
    }

    /// The address it reports.
    ///
    /// # Safety
    /// The watcher stays alive, and its address unwritten, for `'a`.
    unsafe fn address<'a>(self) -> &'a CStr {
        // SAFETY: a field of the live watcher (the caller's promise), which
        // libuv never writes.
        unsafe { (*self.0).address.as_c_str() }
    }

    /// Is this a TCP endpoint rather than a local socket?
    fn is_tcp(self) -> bool {
        // SAFETY: a field of the live watcher, which libuv never writes.
        unsafe { (*self.0).tcp }
    }

    /// The connection callback, once started.
    fn cb(self) -> Option<ConnectionCb> {
        // SAFETY: as above.
        unsafe { (*self.0).cb }
    }

    /// The TCP arm of the handle union.
    fn tcp(self) -> *mut uv_tcp_t {
        // SAFETY: a field of the live watcher; the caller picks the arm.
        unsafe { (&raw mut (*self.0).uv).cast() }
    }

    /// The pipe arm of the handle union.
    fn pipe(self) -> *mut uv_pipe_t {
        // SAFETY: as above.
        unsafe { (&raw mut (*self.0).uv).cast() }
    }

    /// The handle, as the stream libuv listens on.
    fn stream(self) -> *mut uv_stream_t {
        if self.is_tcp() {
            self.tcp().cast()
        } else {
            self.pipe().cast()
        }
    }

    /// The resolved addresses, taken: they are bound or given up on once.
    fn take_lookup(self) -> Option<Lookup> {
        // SAFETY: a field of the live watcher, which libuv never writes.
        unsafe { (*self.0).lookup.take() }
    }

    /// Bind the local-socket arm to the address. A path too long for a
    /// socket fails with `UV_EINVAL` rather than being truncated, which would
    /// serve a different path than `v:servername` reports.
    ///
    /// The error is libuv's negative code.
    fn bind_pipe(self) -> Result<(), c_int> {
        // SAFETY: nothing writes the address while this borrow lives.
        let address = unsafe { self.address() };
        // SAFETY: the watcher's own handle, and its address, which libuv
        // only reads.
        let result = unsafe {
            let len = address.count_bytes();
            uv_pipe_bind2(self.pipe(), address.as_ptr(), len, UV_PIPE_NO_TRUNCATE)
        };
        if result == 0 { Ok(()) } else { Err(result) }
    }
}

crate::event::stream::field_writers!(Watcher {
    fn set_address => address: CString;
    fn set_cb => cb: Option<ConnectionCb>;
});

/// Does `address` name a TCP endpoint rather than a local socket?
pub fn socket_address_is_tcp(address: &CStr) -> bool {
    tcp_host_end(address.to_bytes()).is_some()
}

// ---------------------------------------------------------------------------
// Listening
// ---------------------------------------------------------------------------

/// A socket this process listens on: the watcher, boxed at the address
/// libuv's handle points back to, from [`Listener::new`] until
/// [`Listener::close`] hands it to libuv's close callback, which frees it.
#[derive(Debug)]
pub(crate) struct Listener(Watcher);

impl Listener {
    /// A watcher for `endpoint`, resolved if it is TCP but not yet bound;
    /// the libuv error otherwise.
    pub(crate) fn new(endpoint: &CStr) -> Result<Self, c_int> {
        let (address, lookup) = match Endpoint::parse(endpoint) {
            Ok(Endpoint::Pipe(path)) => (path, None),
            Ok(Endpoint::Tcp { host, port }) => match Lookup::resolve(&host, port.as_deref(), 0) {
                Ok(lookup) => (host, Some(lookup)),
                Err(result) => {
                    logmsg!(
                        LOGLVL_ERR,
                        c"socket_watcher_init",
                        78,
                        "Host lookup failed: {}",
                        msg_cstr(endpoint)
                    );
                    return Err(result);
                }
            },
            Err(InvalidPort(port)) => {
                logmsg!(
                    LOGLVL_ERR,
                    c"socket_watcher_init",
                    62,
                    "Invalid port: {}",
                    msg_bytes(&port)
                );
                return Err(UV_EINVAL);
            }
        };

        let tcp = lookup.is_some();
        let watcher = Box::new(SocketWatcher {
            address,
            // SAFETY: both handles are inhabited by the all-zero bit pattern,
            // and `uv_*_init` below fills the one chosen in place.
            uv: unsafe { mem::zeroed() },
            tcp,
            lookup,
            cb: None,
        });
        // SAFETY: the box, leaked here and freed by `close_cb` once libuv is
        // done with its handle.
        let watcher = unsafe { Watcher::new(Box::into_raw(watcher)) };
        let uv_loop = main_uv_loop();
        if tcp {
            // SAFETY: the editor's loop and the watcher's own handle, which
            // never moves.
            unsafe {
                uv_tcp_init(uv_loop, watcher.tcp());
                uv_tcp_nodelay(watcher.tcp(), 1);
            }
        } else {
            // SAFETY: as above.
            unsafe { uv_pipe_init(uv_loop, watcher.pipe(), 0) };
        }
        // SAFETY: the handle initialised just above.
        unsafe { (*watcher.stream()).data = watcher.as_ptr().cast() };
        Ok(Self(watcher))
    }

    /// The address it listens on: the endpoint as resolved, with the port
    /// the kernel picked once it is bound.
    pub(crate) fn address(&self) -> &CStr {
        // SAFETY: the watcher this value owns, live until `close` consumes
        // it; the address is written only by `start`, which takes `&mut
        // self`, so not while the answer borrows `self`.
        unsafe { self.0.address() }
    }

    /// Bind and listen, reporting each connection to `cb`; the libuv error
    /// otherwise.
    ///
    /// A TCP watcher tries every address the lookup produced; a pipe watcher
    /// gets one retry, after clearing a socket file left behind by an nvim
    /// that is no longer running.
    pub(crate) fn start(&mut self, backlog: c_int, cb: ConnectionCb) -> c_int {
        let watcher = self.0;
        watcher.set_cb(Some(cb));
        let result = if watcher.is_tcp() {
            listen_tcp(watcher, backlog)
        } else {
            listen_pipe(watcher, backlog)
        };
        debug_assert!(result <= 0, "libuv returns a negative error code or zero");
        result
    }

    /// Stop listening; the watcher, and a lookup it never bound, are freed
    /// once libuv is done with it.
    pub(crate) fn close(self) {
        // SAFETY: the watcher's own handle; the watcher stays allocated
        // until `close_cb` frees it.
        unsafe { uv_close(self.0.stream().cast(), Some(close_cb)) };
    }
}

/// Bind a TCP watcher to the first of its resolved addresses that takes,
/// and listen there.
fn listen_tcp(watcher: Watcher, backlog: c_int) -> c_int {
    // Spent whatever happens: freed when this returns.
    let lookup = watcher
        .take_lookup()
        .expect("a TCP listener is started once, holding its lookup");
    let mut result = UV_EINVAL;
    for address in lookup.addresses() {
        // SAFETY: the watcher's own handle, and an address the lookup holds.
        result = unsafe { uv_tcp_bind(watcher.tcp(), address, 0) };
        if result != 0 {
            continue;
        }
        // SAFETY: the watcher's own handle, bound just now.
        result = unsafe { uv_listen(watcher.stream(), backlog, Some(connection_cb)) };
        if result == 0 {
            record_bound_port(watcher);
            break;
        }
    }
    result
}

/// Bind a pipe watcher to its path and listen there.
fn listen_pipe(watcher: Watcher, backlog: c_int) -> c_int {
    let mut result = watcher.bind_pipe().err().unwrap_or(0);
    if result == UV_EACCES || result == UV_EADDRINUSE {
        result = rebind_stale_socket(watcher, result);
    }
    if result == 0 {
        // SAFETY: the watcher's own handle, bound just now.
        result = unsafe { uv_listen(watcher.stream(), backlog, Some(connection_cb)) };
    }
    if result == UV_EACCES {
        // libuv reports a missing parent directory as EACCES, for Windows
        // compatibility. ENOENT is the more useful answer.
        // SAFETY: nothing writes the address while this borrow lives.
        let address = unsafe { watcher.address() }.to_bytes();
        let parent = &address[..tail_index(address)];
        let parent = CString::new(parent).expect("a C string's prefix holds no NUL");
        if !path_exists(&parent) {
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
    let mut sas: ::libc::sockaddr_storage = unsafe { mem::zeroed() };
    let size = size_of::<::libc::sockaddr_storage>();
    let mut len = c_int::try_from(size).expect("a socket address fits in an int");
    // SAFETY: the watcher's own handle, and a buffer on this frame.
    unsafe { uv_tcp_getsockname(watcher.tcp(), (&raw mut sas).cast(), &raw mut len) };
    // SAFETY: both address families put the port right after the family, and
    // both structs are `repr(C)`.
    let port = ntohs(unsafe { (*(&raw const sas).cast::<SockaddrHead>()).port });

    // SAFETY: the copy is taken before the write below.
    let mut bound = unsafe { watcher.address() }.to_bytes().to_vec();
    bound.extend_from_slice(format!(":{port}").as_bytes());
    watcher.set_address(CString::new(bound).expect("an address and a port hold no NUL"));
}

/// A pipe bind failed because the socket file is already there. If nothing is
/// listening on it, remove it and bind again; the handle has to be closed and
/// re-created first, because libuv will not reuse one that failed to bind.
///
/// Returns the new bind result, or `failure` unchanged if the socket is live
/// or could not be removed.
fn rebind_stale_socket(watcher: Watcher, failure: c_int) -> c_int {
    // SAFETY: a copy, taken before the loop runs below.
    let address = unsafe { watcher.address() }.to_owned();
    if socket_alive(&address) {
        logmsg!(
            LOGLVL_ERR,
            c"socket_watcher_start",
            203,
            "Socket already in use by another Nvim instance: {}",
            msg_cstr(&address)
        );
        return failure;
    }

    logmsg!(
        LOGLVL_INF,
        c"socket_watcher_start",
        180,
        "Removing stale socket: {}",
        msg_cstr(&address)
    );
    let rm_result = os_remove(&address);
    if rm_result != 0 {
        logmsg!(
            LOGLVL_WRN,
            c"socket_watcher_start",
            185,
            "Failed to remove stale socket {}: {}",
            msg_cstr(&address),
            msg_cstr(os_strerror(rm_result))
        );
        return failure;
    }

    let handle = watcher.pipe();
    // A `Cell`: the close callback writes it through `data` while the
    // wait's closure reads it, and a plain `bool` borrowed by that closure
    // is one the optimiser may read once (the release build did, and
    // waited for ever).
    let closed = Cell::new(false);
    // SAFETY: the watcher's own handle; `closed` outlives the wait below.
    unsafe {
        (*handle).data = closed.as_ptr().cast();
        uv_close(handle.cast(), Some(early_server_close_cb));
    }
    wait_on_main_loop(Drain::Nothing, -1, || closed.get());

    // SAFETY: a fresh handle in the closed one's place, on the editor's loop.
    unsafe {
        uv_pipe_init(main_uv_loop(), handle, 0);
        (*handle).data = watcher.as_ptr().cast();
    }
    watcher.bind_pipe().err().unwrap_or(0)
}

/// Is anything listening on `address`? Probed by connecting to it, with a
/// 500ms timeout so a dead socket is diagnosed quickly.
fn socket_alive(address: &CStr) -> bool {
    let stream = ReadStream::unopened();
    // SAFETY: a stream on this frame, which is not moved and outlives the
    // probe: the frame waits for the close below before it returns.
    if unsafe { socket_connect(&stream, false, address, 500) }.is_err() {
        return false;
    }

    // Take the probe connection back down before answering: the stream is on
    // this stack frame.
    // A `Cell`, as in `rebind_stale_socket`.
    let closed = Cell::new(false);
    stream.set_internal_close_cb(Some(connect_close_cb), closed.as_ptr().cast());
    may_close(stream.conn());
    wait_on_main_loop(Drain::Nothing, -1, || closed.get());
    true
}

/// A client waiting on a listening socket, as its connection callback sees
/// it: the listener lent for that one call. It hands out no `&mut` to the
/// watcher, whose handles libuv holds by address; a callback can accept the
/// client and read the address, nothing else.
pub struct Incoming<'call> {
    watcher: Watcher,
    _call: core::marker::PhantomData<&'call SocketWatcher>,
}

impl Incoming<'_> {
    /// Hand the waiting client to `stream`, which the caller owns; libuv's
    /// error otherwise.
    ///
    /// # Safety
    /// `stream` is unopened, and the caller keeps it alive, unmoved, until
    /// its handle's close callback has run.
    pub(in crate::event) unsafe fn accept(&self, stream: &ReadStream) -> c_int {
        let watcher = self.watcher;
        let conn = stream.conn();
        let uv_loop = main_uv_loop();
        let client: *mut uv_stream_t = if watcher.is_tcp() {
            // SAFETY: the caller's stream, on the editor's loop.
            unsafe {
                let client = conn.tcp();
                uv_tcp_init(uv_loop, client);
                uv_tcp_nodelay(client, 1);
                client.cast()
            }
        } else {
            // SAFETY: as above.
            unsafe {
                let client = conn.pipe();
                uv_pipe_init(uv_loop, client, 0);
                client.cast()
            }
        };

        // SAFETY: the watcher's listening handle and the client just built.
        let result = unsafe { uv_accept(watcher.stream(), client) };
        if result != 0 {
            // SAFETY: the client handle, which nothing else holds yet.
            unsafe { uv_close(client.cast(), None) };
            return result;
        }
        // SAFETY: the caller's stream, now holding the accepted connection.
        unsafe { stream_init(ptr::null_mut(), conn.as_ptr(), -1, client) };
        0
    }

    /// The address the listener is bound to.
    pub(crate) fn address(&self) -> CString {
        // SAFETY: the listener, live for the callback this is lent to; the
        // copy is taken before the borrow ends.
        unsafe { self.watcher.address() }.to_owned()
    }
}

// ---------------------------------------------------------------------------
// Connecting
// ---------------------------------------------------------------------------

/// Connect `stream` to `address`, driving the editor's loop until it
/// settles.
///
/// `timeout` is in milliseconds. A failure answers a message the caller may
/// show; a TCP address that resolved to several candidates is tried in turn
/// before that happens.
///
/// # Safety
/// `stream` is unopened; on success the caller keeps it alive, unmoved,
/// until its handle's close callback has run (a failure has closed it
/// before this returns).
pub(crate) unsafe fn socket_connect(
    stream: &ReadStream,
    is_tcp: bool,
    address: &CStr,
    timeout: c_int,
) -> Result<(), &'static CStr> {
    let conn = stream.conn();
    let uv_loop = main_uv_loop();

    let lookup = if is_tcp {
        let bytes = address.to_bytes();
        let Some(host_end) = bytes.iter().rposition(|&b| b == b':') else {
            return Err(gettext(c"tcp address must be host:port"));
        };
        let host = CString::new(&bytes[..host_end]).expect("a C string holds no NUL");
        let port = CString::new(&bytes[host_end + 1..]).expect("a C string holds no NUL");
        let Ok(lookup) = Lookup::resolve(&host, Some(&port), AI_NUMERICSERV) else {
            return Err(gettext(c"failed to lookup host or port"));
        };
        Some(lookup)
    } else {
        None
    };
    // One attempt per resolved address; a pipe has the one, its path.
    let candidates: Vec<Option<*const sockaddr>> = match &lookup {
        Some(lookup) => lookup.addresses().map(Some).collect(),
        None => vec![None],
    };

    // `Cell`s, as in `rebind_stale_socket`: the callbacks write them while
    // the waits' closures read them.
    let status: Cell<c_int> = Cell::new(1);
    let closed = Cell::new(false);
    // SAFETY: `uv_connect_t` is inhabited by the all-zero bit pattern.
    let mut req: uv_connect_t = unsafe { mem::zeroed() };
    req.data = status.as_ptr().cast();

    let mut settled = Err(gettext(c"connection refused"));
    for candidate in candidates {
        let uv_stream: *mut uv_stream_t = match candidate {
            // SAFETY: the caller's stream, the editor's loop, and an address
            // the lookup holds.
            Some(target) => unsafe {
                let tcp = conn.tcp();
                uv_tcp_init(uv_loop, tcp);
                uv_tcp_nodelay(tcp, 1);
                uv_tcp_connect(&raw mut req, tcp, target, Some(connect_cb));
                tcp.cast()
            },
            // SAFETY: the caller's stream and address, and the editor's loop.
            None => unsafe {
                let pipe = conn.pipe();
                uv_pipe_init(uv_loop, pipe, 0);
                uv_pipe_connect(&raw mut req, pipe, address.as_ptr(), Some(connect_cb));
                pipe.cast()
            },
        };
        // SAFETY: the caller's stream, now holding the connect attempt.
        unsafe { stream_init(ptr::null_mut(), conn.as_ptr(), -1, uv_stream) };
        conn.set_internal_close_cb(Some(connect_close_cb));
        conn.set_internal_data(closed.as_ptr().cast());
        closed.set(false);
        status.set(1);

        wait_on_main_loop(Drain::Nothing, i64::from(timeout), || status.get() != 1);
        if status.get() == 0 {
            settled = Ok(());
            break;
        }

        may_close(conn);
        // Wait for the close callback before retrying or returning: `stream`
        // may be on the caller's stack.
        wait_on_main_loop(Drain::Nothing, -1, || closed.get());
    }

    conn.set_internal_close_cb(None);
    conn.set_internal_data(ptr::null_mut());
    settled
}

// ---------------------------------------------------------------------------
// Callbacks
// ---------------------------------------------------------------------------

/// libuv: a client is waiting. Reported to the watcher's callback straight
/// from libuv's stack, as upstream's watcher without a queue does.
///
/// # Safety
/// libuv's: `handle` is a listening handle whose `data` is its watcher.
unsafe extern "C" fn connection_cb(handle: *mut uv_stream_t, status: c_int) {
    // SAFETY: the caller's promise.
    let watcher = unsafe { Watcher::new((*handle).data.cast()) };
    let notify = watcher.cb().expect("a started watcher has a callback");
    let incoming = Incoming {
        watcher,
        _call: core::marker::PhantomData,
    };
    notify(&incoming, status);
}

/// libuv: a [`Listener`]'s handle is closed, and libuv is done with it:
/// free the watcher.
///
/// # Safety
/// libuv's: `handle` is a listening handle whose `data` is the box
/// [`Listener::new`] leaked, and nothing uses it again.
unsafe extern "C" fn close_cb(handle: *mut uv_handle_t) {
    // SAFETY: the caller's promise.
    drop(unsafe { Box::from_raw((*handle).data.cast::<SocketWatcher>()) });
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
