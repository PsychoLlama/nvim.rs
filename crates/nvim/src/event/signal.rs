//! Signal watchers: a libuv signal handle whose deliveries become events.
//!
//! Like the timer watchers, the owner's callback runs from a queue rather
//! than from libuv's stack. The close notification is the exception — it is
//! delivered directly, see [`close_cb`].
//!
//! libuv keeps the watcher's address in its handle's `data` field, which is
//! how the callbacks find their way back. [`Signal`] wraps that pointer once,
//! so the bodies below are ordinary Rust; nothing here may move a watcher.

#![deny(unsafe_op_in_unsafe_fn)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use core::ffi::{c_int, c_void};
use core::ops::{Deref, DerefMut};

use crate::event::libuv::{uv_close, uv_signal_init, uv_signal_start, uv_signal_stop};
use crate::event::r#loop::{EventLoop, one_arg_event};
use crate::event::multiqueue::multiqueue_put_event;
use crate::types::{
    Loop, MultiQueue, SignalWatcher, signal_cb, signal_close_cb, uv_handle_t, uv_signal_t,
};

/// A signal watcher, reached through the raw pointer libuv keeps in its
/// handle's `data` field.
#[derive(Copy, Clone)]
struct Signal(*mut SignalWatcher);

impl Signal {
    /// # Safety
    /// `watcher` is live, does not move while it is started, and outlives
    /// every use of this handle.
    unsafe fn new(watcher: *mut SignalWatcher) -> Self {
        debug_assert!(!watcher.is_null());
        Signal(watcher)
    }

    /// The pointer back, for the owner's callback, which still takes one.
    fn as_ptr(self) -> *mut SignalWatcher {
        self.0
    }

    /// The libuv handle this watcher owns.
    fn uv(self) -> *mut uv_signal_t {
        // SAFETY: a field of the live watcher, derived from the wrapped
        // pointer rather than from a borrow of it.
        unsafe { &raw mut (*self.0).uv }
    }

    /// The queue this watcher's events go on, or `None` when it is driven
    /// synchronously.
    fn events(self) -> Option<*mut MultiQueue> {
        let events = self.events;
        (!events.is_null()).then_some(events)
    }
}

impl Deref for Signal {
    type Target = SignalWatcher;

    fn deref(&self) -> &SignalWatcher {
        // SAFETY: the promise made at construction.
        unsafe { &*self.0 }
    }
}

impl DerefMut for Signal {
    fn deref_mut(&mut self) -> &mut SignalWatcher {
        // SAFETY: the promise made at construction.
        unsafe { &mut *self.0 }
    }
}

/// Attach `watcher` to `uv_loop`, delivering its events on the fast queue.
///
/// # Safety
/// `uv_loop` is live, and `watcher` points at storage for a watcher that does
/// not move afterwards.
pub unsafe fn signal_watcher_init(
    uv_loop: *mut Loop,
    watcher: *mut SignalWatcher,
    data: *mut c_void,
) {
    // SAFETY: the caller's loop and storage.
    let (uv_loop, mut watcher) = unsafe { (EventLoop::new(uv_loop), Signal::new(watcher)) };
    // SAFETY: the watcher's own handle, on the caller's loop.
    unsafe { uv_signal_init(uv_loop.uv(), watcher.uv()) };
    watcher.uv.data = watcher.as_ptr().cast();
    watcher.data = data;
    watcher.cb = None;
    watcher.events = uv_loop.fast_events;
}

/// Listen for `signum`, reporting each delivery to `cb`.
///
/// # Safety
/// `watcher` has been through [`signal_watcher_init`], and `cb` is safe to
/// call with the watcher's `data`.
pub unsafe fn signal_watcher_start(watcher: *mut SignalWatcher, cb: signal_cb, signum: c_int) {
    // SAFETY: the caller's watcher.
    let mut watcher = unsafe { Signal::new(watcher) };
    watcher.cb = cb;
    // SAFETY: the watcher's own handle, registered with a loop.
    unsafe { uv_signal_start(watcher.uv(), Some(signal_watcher_cb), signum) };
}

/// # Safety
/// `watcher` has been through [`signal_watcher_init`].
pub unsafe fn signal_watcher_stop(watcher: *mut SignalWatcher) {
    // SAFETY: the caller's watcher, and its own handle.
    unsafe { uv_signal_stop(Signal::new(watcher).uv()) };
}

/// Close the watcher's handle; `cb` is told once libuv is done with it.
///
/// # Safety
/// `watcher` has been through [`signal_watcher_init`] and stays put until
/// `cb` runs.
pub unsafe fn signal_watcher_close(watcher: *mut SignalWatcher, cb: signal_close_cb) {
    // SAFETY: the caller's watcher.
    let mut watcher = unsafe { Signal::new(watcher) };
    watcher.close_cb = cb;
    // SAFETY: the watcher's own handle, still open.
    unsafe { uv_close(watcher.uv().cast(), Some(close_cb)) };
}

/// libuv: the signal arrived. The number is read back off the handle rather
/// than taken from the argument, so a watcher restarted on a different signal
/// between delivery and processing reports the one it is watching now.
///
/// # Safety
/// libuv's signal callback: `handle` is a watcher's own.
unsafe extern "C" fn signal_watcher_cb(handle: *mut uv_signal_t, _signum: c_int) {
    // SAFETY: `signal_watcher_init` put the watcher's address in `data`.
    let watcher = unsafe { Signal::new((*handle).data.cast()) };
    let arg = watcher.as_ptr().cast::<c_void>();
    match watcher.events() {
        // SAFETY: the watcher outlives the event, which carries nothing but
        // the watcher itself.
        Some(events) => unsafe {
            multiqueue_put_event(events, one_arg_event(Some(signal_event), arg));
        },
        None => {
            let mut argv = [arg];
            // SAFETY: as above; the argv is this frame's.
            unsafe { signal_event(argv.as_mut_ptr()) };
        }
    }
}

/// # Safety
/// `argv` slot 0 is the watcher, as [`signal_watcher_cb`] packed it.
unsafe extern "C" fn signal_event(argv: *mut *mut c_void) {
    // SAFETY: the caller's promise.
    let watcher = unsafe { Signal::new((*argv).cast()) };
    let notify = watcher.cb.expect("a started watcher has a callback");
    let (signum, data) = (watcher.uv.signum, watcher.data);
    // SAFETY: the callback `signal_watcher_start` installed, with the data
    // `signal_watcher_init` was given for it.
    unsafe { notify(watcher.as_ptr(), signum, data) };
}

/// libuv: the handle is closed.
///
/// Unlike the timer watchers, this notification is *not* queued: upstream
/// calls the owner's callback straight from libuv's stack. Preserved —
/// `os/signal.rs` and `tui/tui.rs` both act on the watcher from here, and a
/// queued close would land after the loop has already been torn down.
///
/// # Safety
/// libuv's close callback: `handle` is a watcher's own, and this is its last
/// callback.
unsafe extern "C" fn close_cb(handle: *mut uv_handle_t) {
    // SAFETY: `signal_watcher_init` put the watcher's address in `data`.
    let watcher = unsafe { Signal::new((*handle).data.cast()) };
    if let Some(notify) = watcher.close_cb {
        let data = watcher.data;
        // SAFETY: the callback `signal_watcher_close` installed, with the
        // data `signal_watcher_init` was given for it.
        unsafe { notify(watcher.as_ptr(), data) };
    }
}
