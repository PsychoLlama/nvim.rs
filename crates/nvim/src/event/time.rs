//! Timer watchers: a libuv timer whose expiry is delivered as an event.
//!
//! The callback does not run from libuv's stack. It is queued on the
//! watcher's own [`MultiQueue`] (the loop's fast queue by default) and runs
//! when that queue is drained, so a timer cannot re-enter the editor from
//! inside `uv_run`. A watcher with no queue is driven synchronously and its
//! callback runs on the spot.
//!
//! libuv keeps the watcher's address in its timer's `data` field, which is
//! how the callbacks find their way back. [`Timer`] wraps that pointer once,
//! so the bodies below are ordinary Rust; nothing here may move a watcher.

#![deny(unsafe_op_in_unsafe_fn)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use core::ffi::c_void;
use core::ops::{Deref, DerefMut};

use crate::event::libuv::{uv_close, uv_timer_init, uv_timer_start, uv_timer_stop};
use crate::event::r#loop::{EventLoop, one_arg_event};
use crate::event::multiqueue::{multiqueue_empty, multiqueue_put_event};
use crate::types::{Loop, MultiQueue, TimeWatcher, time_cb, uv_handle_t, uv_timer_t};

/// A timer watcher, reached through the raw pointer libuv keeps in its
/// timer's `data` field.
#[derive(Copy, Clone)]
struct Timer(*mut TimeWatcher);

impl Timer {
    /// # Safety
    /// `watcher` is live, does not move while it is armed, and outlives every
    /// use of this handle.
    unsafe fn new(watcher: *mut TimeWatcher) -> Self {
        debug_assert!(!watcher.is_null());
        Timer(watcher)
    }

    /// The pointer back, for the owner's callback, which still takes one.
    fn as_ptr(self) -> *mut TimeWatcher {
        self.0
    }

    /// The libuv timer this watcher owns.
    fn uv(self) -> *mut uv_timer_t {
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

impl Deref for Timer {
    type Target = TimeWatcher;

    fn deref(&self) -> &TimeWatcher {
        // SAFETY: the promise made at construction.
        unsafe { &*self.0 }
    }
}

impl DerefMut for Timer {
    fn deref_mut(&mut self) -> &mut TimeWatcher {
        // SAFETY: the promise made at construction.
        unsafe { &mut *self.0 }
    }
}

/// Attach `watcher` to `uv_loop`, delivering its events on the fast queue.
///
/// # Safety
/// `uv_loop` is live, and `watcher` points at storage for a watcher that does
/// not move afterwards.
pub unsafe fn time_watcher_init(uv_loop: *mut Loop, watcher: *mut TimeWatcher, data: *mut c_void) {
    // SAFETY: the caller's loop and storage.
    let (uv_loop, mut watcher) = unsafe { (EventLoop::new(uv_loop), Timer::new(watcher)) };
    // SAFETY: the watcher's own timer, on the caller's loop.
    unsafe { uv_timer_init(uv_loop.uv(), watcher.uv()) };
    watcher.uv.data = watcher.as_ptr().cast();
    watcher.data = data;
    watcher.events = uv_loop.fast_events;
    watcher.blockable = false;
}

/// Fire `cb` after `timeout` milliseconds, then every `repeat` (0 for once).
///
/// # Safety
/// `watcher` has been through [`time_watcher_init`], and `cb` is safe to call
/// with the watcher's `data`.
pub unsafe fn time_watcher_start(
    watcher: *mut TimeWatcher,
    cb: time_cb,
    timeout: u64,
    repeat: u64,
) {
    // SAFETY: the caller's watcher.
    let mut watcher = unsafe { Timer::new(watcher) };
    watcher.cb = cb;
    // SAFETY: the watcher's own timer, registered with a loop.
    unsafe { uv_timer_start(watcher.uv(), Some(time_watcher_cb), timeout, repeat) };
}

/// # Safety
/// `watcher` has been through [`time_watcher_init`].
pub unsafe fn time_watcher_stop(watcher: *mut TimeWatcher) {
    // SAFETY: the caller's watcher, and its own timer.
    unsafe { uv_timer_stop(Timer::new(watcher).uv()) };
}

/// Close the timer's handle; `cb` is told once libuv is done with it.
///
/// # Safety
/// `watcher` has been through [`time_watcher_init`] and stays put until `cb`
/// runs.
pub unsafe fn time_watcher_close(watcher: *mut TimeWatcher, cb: time_cb) {
    // SAFETY: the caller's watcher.
    let mut watcher = unsafe { Timer::new(watcher) };
    watcher.close_cb = cb;
    // SAFETY: the watcher's own timer, still open.
    unsafe { uv_close(watcher.uv().cast(), Some(close_cb)) };
}

/// Deliver `handler` for `watcher`: queued when it has a queue, run on the
/// spot when it does not.
fn deliver(watcher: Timer, handler: unsafe extern "C" fn(*mut *mut c_void)) {
    let arg = watcher.as_ptr().cast::<c_void>();
    match watcher.events() {
        // SAFETY: the watcher outlives the event, which carries nothing but
        // the watcher itself.
        Some(events) => unsafe { multiqueue_put_event(events, one_arg_event(Some(handler), arg)) },
        None => {
            let mut argv = [arg];
            // SAFETY: as above; the argv is this frame's.
            unsafe { handler(argv.as_mut_ptr()) };
        }
    }
}

/// libuv: the timer expired.
///
/// A *blockable* timer skips a tick whose event would only queue up behind
/// one the editor has not processed yet — that is how the terminal refresh
/// timer keeps a flood of output from outrunning the redraw.
///
/// # Safety
/// libuv's timer callback: `handle` is a watcher's own timer.
unsafe extern "C" fn time_watcher_cb(handle: *mut uv_timer_t) {
    // SAFETY: `time_watcher_init` put the watcher's address in `data`.
    let watcher = unsafe { Timer::new((*handle).data.cast()) };
    // SAFETY: a blockable watcher always has a queue to look at.
    if watcher.blockable && !unsafe { multiqueue_empty(watcher.events) } {
        return;
    }
    deliver(watcher, time_event);
}

/// # Safety
/// `argv` slot 0 is the watcher, as [`deliver`] packed it.
unsafe extern "C" fn time_event(argv: *mut *mut c_void) {
    // SAFETY: the caller's promise.
    let watcher = unsafe { Timer::new((*argv).cast()) };
    let (notify, data) = (
        watcher.cb.expect("a started timer has a callback"),
        watcher.data,
    );
    // SAFETY: the callback `time_watcher_start` installed, with the data
    // `time_watcher_init` was given for it.
    unsafe { notify(watcher.as_ptr(), data) };
}

/// libuv: the handle is closed. Reported the same way as an expiry, so the
/// owner cannot see the close before events the timer already queued.
///
/// # Safety
/// libuv's close callback: `handle` is a watcher's own timer, and this is its
/// last callback.
unsafe extern "C" fn close_cb(handle: *mut uv_handle_t) {
    // SAFETY: `time_watcher_init` put the watcher's address in `data`.
    let watcher = unsafe { Timer::new((*handle).data.cast()) };
    if watcher.close_cb.is_none() {
        return;
    }
    deliver(watcher, close_event);
}

/// # Safety
/// `argv` slot 0 is the watcher, as [`deliver`] packed it.
unsafe extern "C" fn close_event(argv: *mut *mut c_void) {
    // SAFETY: the caller's promise.
    let watcher = unsafe { Timer::new((*argv).cast()) };
    let (notify, data) = (
        watcher.close_cb.expect("checked by the caller"),
        watcher.data,
    );
    // SAFETY: the callback `time_watcher_close` installed, with the data
    // `time_watcher_init` was given for it.
    unsafe { notify(watcher.as_ptr(), data) };
}
