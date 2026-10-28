//! Timer watchers: a libuv timer whose expiry is delivered as an event.
//!
//! The callback does not run from libuv's stack. It is queued on the
//! watcher's own [`MultiQueue`] and runs when that queue is drained, so a
//! timer cannot re-enter the editor from inside `uv_run`. A watcher with no
//! queue is driven synchronously and its callback runs on the spot.
//!
//! # Ownership
//!
//! A watcher is an [`OwnedTimer`]: this module boxes it, so its address --
//! which libuv keeps in the timer handle's `data` -- never moves, and the
//! box is released by the handle's close callback, the last thing libuv does
//! with it. The owner names itself to its callbacks by an *id* it picks (a
//! timer's number, a channel's id), never by an address: a callback finds
//! its record again through that id, and stops cleanly when the record has
//! gone. Dropping an [`OwnedTimer`] closes it.
//!
//! [`ScopedTimer`] is the one shape that needs no callback at all: a timer
//! that only exists to wake the loop up while a frame waits on it
//! (`wait()`, `vim.wait()`).

#![deny(unsafe_op_in_unsafe_fn)]
// Unsafe perimeter: the `event/` row in docs/perimeter.md.
#![allow(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use core::ffi::c_void;
use core::ptr;

use crate::event::libuv::{uv_close, uv_timer_init, uv_timer_start, uv_timer_stop};
use crate::event::r#loop::{main_loop_raw, one_arg_event};
use crate::event::multiqueue::{ChildQueue, multiqueue_empty, multiqueue_put_event};
use crate::types::{MultiQueue, uv_handle_t, uv_timer_t};

/// Where a watcher's expiries are delivered.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub(crate) enum TimerQueue {
    /// Run on the spot, from libuv's own callback: a watcher that only wakes
    /// the loop up, with nothing in its callback that could re-enter it.
    Inline,
    /// A child queue of the main loop's, which the watcher owns and frees
    /// with itself.
    OwnChild,
}

/// What an expiry does.
#[derive(Copy, Clone)]
enum OnDue {
    /// Nothing armed yet.
    Unset,
    /// The owner's callback, told the owner's id.
    Owner(fn(u64)),
    /// Wake the loop, and close the timer if the loop is shutting down
    /// under the frame that owns it ([`ScopedTimer`]).
    Wake,
}

/// The boxed watcher. `uv` comes first so the handle's address is the
/// watcher's.
#[repr(C)]
struct Watcher {
    uv: uv_timer_t,
    /// The owner's id, handed back to each callback.
    owner: u64,
    on_due: OnDue,
    on_closed: Option<fn(u64)>,
    /// The queue expiries go on, or null to run them inline.
    events: *mut MultiQueue,
    /// The queue `events` names, when the watcher owns it.
    queue: Option<ChildQueue>,
    /// Skip an expiry while the previous one is still queued.
    blockable: bool,
    /// The handle is closing; nothing may restart or close it again.
    closing: bool,
    /// Release the box once the close has been delivered. Cleared only for
    /// a [`ScopedTimer`] that closed itself while its frame still holds it.
    free_on_close: bool,
}

/// A watcher, reached through the box's pointer -- which libuv also keeps
/// in the handle's `data` -- wrapped once, so the field accesses below are
/// ordinary Rust. Nothing here may move a watcher.
#[derive(Copy, Clone)]
struct Handle(*mut Watcher);

impl Handle {
    /// # Safety
    /// `watcher` is a live box from [`OwnedTimer::new`], until its close
    /// event frees it, which outlives every use of this handle.
    unsafe fn new(watcher: *mut Watcher) -> Self {
        debug_assert!(!watcher.is_null());
        Handle(watcher)
    }

    /// The libuv timer, derived from the wrapped pointer rather than from a
    /// borrow of it.
    fn uv(self) -> *mut uv_timer_t {
        // SAFETY: a field of the live box.
        unsafe { &raw mut (*self.0).uv }
    }

    /// The handle libuv is given back in its callbacks.
    fn from_uv(uv: *mut uv_timer_t) -> *mut Watcher {
        // `uv` is the watcher's first field, so the two share an address.
        uv.cast()
    }
}

impl core::ops::Deref for Handle {
    type Target = Watcher;

    fn deref(&self) -> &Watcher {
        // SAFETY: the promise made at construction.
        unsafe { &*self.0 }
    }
}

impl core::ops::DerefMut for Handle {
    fn deref_mut(&mut self) -> &mut Watcher {
        // SAFETY: the promise made at construction.
        unsafe { &mut *self.0 }
    }
}

/// A libuv timer this module owns at a fixed address. See the module docs.
pub(crate) struct OwnedTimer(Handle);

impl OwnedTimer {
    /// A stopped timer on the main loop, delivering to `queue`, whose
    /// callbacks will be told `owner`.
    ///
    /// The main loop is up from `event_init` until exit, which is when every
    /// owner of one of these exists.
    pub(crate) fn new(queue: TimerQueue, blockable: bool, owner: u64) -> Self {
        let own = (queue == TimerQueue::OwnChild).then(ChildQueue::of_main_loop);
        let events = own.as_ref().map_or(ptr::null_mut(), ChildQueue::as_ptr);
        debug_assert!(
            !blockable || !events.is_null(),
            "a blockable timer has a queue"
        );
        let watcher = Box::into_raw(Box::new(Watcher {
            uv: uv_timer_t::default(),
            owner,
            on_due: OnDue::Unset,
            on_closed: None,
            events,
            queue: own,
            blockable,
            closing: false,
            free_on_close: true,
        }));
        // SAFETY: the box just made, at the address it keeps until the close
        // event frees it.
        let mut handle = unsafe { Handle::new(watcher) };
        // SAFETY: the box's own timer, on the editor's loop, live from
        // startup until exit.
        unsafe { uv_timer_init(&raw mut (*main_loop_raw()).uv, handle.uv()) };
        handle.uv.data = watcher.cast();
        OwnedTimer(handle)
    }

    /// Fire `on_due` after `timeout` milliseconds, then every `repeat` (0
    /// for once).
    pub(crate) fn start(&mut self, on_due: fn(u64), timeout: u64, repeat: u64) {
        self.arm(OnDue::Owner(on_due), timeout, repeat);
    }

    fn arm(&mut self, on_due: OnDue, timeout: u64, repeat: u64) {
        // A watcher closed under its owner (a `ScopedTimer` at loop
        // shutdown) is not armed again.
        if self.0.closing {
            return;
        }
        self.0.on_due = on_due;
        // SAFETY: the box's own timer, open.
        unsafe { uv_timer_start(self.0.uv(), Some(time_watcher_cb), timeout, repeat) };
    }

    /// Disarm the timer. A tick already queued is still delivered.
    pub(crate) fn stop(&mut self) {
        // SAFETY: the box's own timer handle; stopping a closing or stopped
        // timer is a no-op to libuv.
        unsafe { uv_timer_stop(self.0.uv()) };
    }

    /// The queue expiries are delivered on, or null for an inline timer.
    pub(crate) fn events(&self) -> *mut MultiQueue {
        self.0.events
    }

    /// Close the timer; `on_closed` is told once libuv is done with it,
    /// after any expiry it had already queued.
    pub(crate) fn close(self, on_closed: Option<fn(u64)>) {
        let handle = self.0;
        core::mem::forget(self);
        close_watcher(handle, on_closed);
    }
}

impl Drop for OwnedTimer {
    fn drop(&mut self) {
        close_watcher(self.0, None);
    }
}

/// Stop and close `watcher`, unless it is closing already. The box passes
/// to [`close_cb`], which frees it.
fn close_watcher(mut watcher: Handle, on_closed: Option<fn(u64)>) {
    if watcher.closing {
        // A `ScopedTimer` that closed itself at loop shutdown: the close is
        // under way and will not free the box, which goes with the process.
        return;
    }
    watcher.closing = true;
    watcher.on_closed = on_closed;
    // SAFETY: the box's own timer, open until now; its close callback is
    // the last thing libuv does with it.
    unsafe {
        uv_timer_stop(watcher.uv());
        uv_close(watcher.uv().cast(), Some(close_cb));
    }
}

/// A timer that only keeps the loop turning every `interval` milliseconds
/// while the frame that made it waits: `wait()`'s and `vim.wait()`'s.
///
/// If the loop shuts down under the waiting frame -- the frame never gets
/// back to drop this -- the tick closes the timer itself, so that the loop
/// can finish closing.
pub(crate) struct ScopedTimer {
    /// Held for its drop, which closes it.
    _timer: OwnedTimer,
}

impl ScopedTimer {
    /// Wake the loop every `interval` milliseconds until dropped.
    pub(crate) fn wake_every(interval: u64) -> Self {
        let mut timer = OwnedTimer::new(TimerQueue::Inline, false, 0);
        timer.arm(OnDue::Wake, interval, interval);
        ScopedTimer { _timer: timer }
    }
}

/// Deliver `handler` for `watcher`: queued when it has a queue, run on the
/// spot when it does not.
fn deliver(watcher: Handle, handler: unsafe extern "C" fn(*mut *mut c_void)) {
    let arg = watcher.0.cast::<c_void>();
    if watcher.events.is_null() {
        let mut argv = [arg];
        // SAFETY: the handler takes the watcher it is given, live until its
        // close event; the argv is this frame's.
        unsafe { handler(argv.as_mut_ptr()) };
    } else {
        // SAFETY: the watcher's queue, which lives as long as the watcher;
        // the box is freed only by the close event, queued behind this one.
        unsafe { multiqueue_put_event(watcher.events, one_arg_event(Some(handler), arg)) };
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
    // SAFETY: the box lives until the handle's close callback.
    let watcher = unsafe { Handle::new(Handle::from_uv(handle)) };
    // SAFETY: a blockable watcher always has a queue, its own.
    if watcher.blockable && !unsafe { multiqueue_empty(watcher.events) } {
        return;
    }
    deliver(watcher, time_event);
}

/// # Safety
/// `argv` slot 0 is the watcher, as [`deliver`] packed it.
unsafe extern "C" fn time_event(argv: *mut *mut c_void) {
    // SAFETY: the caller's promise; the box outlives every event queued
    // before its close event.
    let watcher = unsafe { Handle::new((*argv).cast()) };
    match watcher.on_due {
        OnDue::Owner(notify) => notify(watcher.owner),
        OnDue::Wake => {
            // SAFETY: the editor's loop, live until it is closed.
            if unsafe { (*main_loop_raw()).closing } {
                // The box stays with the frame that owns it, which is never
                // going to drop it.
                let mut watcher = watcher;
                watcher.free_on_close = false;
                close_watcher(watcher, None);
            }
        }
        OnDue::Unset => {}
    }
}

/// libuv: the handle is closed. Reported the same way as an expiry, so the
/// owner cannot see the close before events the timer already queued.
///
/// # Safety
/// libuv's close callback: `handle` is a watcher's own timer, and this is its
/// last callback.
unsafe extern "C" fn close_cb(handle: *mut uv_handle_t) {
    // SAFETY: the box lives until the close event this delivers.
    deliver(
        unsafe { Handle::new(Handle::from_uv(handle.cast())) },
        close_event,
    );
}

/// # Safety
/// `argv` slot 0 is the watcher, as [`deliver`] packed it, after the close.
unsafe extern "C" fn close_event(argv: *mut *mut c_void) {
    // SAFETY: the caller's promise; nothing but this event refers to the box
    // any more.
    let watcher = unsafe { Handle::new((*argv).cast()) };
    if let Some(notify) = watcher.on_closed {
        notify(watcher.owner);
    }
    if watcher.free_on_close {
        // SAFETY: the box `OwnedTimer::new` made, released once, here. Its
        // own queue, if any, goes with it.
        drop(unsafe { Box::from_raw(watcher.0) });
    }
}
