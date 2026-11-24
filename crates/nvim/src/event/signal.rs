//! Signal watchers: a libuv signal handle whose deliveries become events.
//!
//! [`SignalWatcher`] is a watcher as its owner holds it (`os/signal.rs`'s
//! table, the TUI's `SIGWINCH`): in place, driven through `&self`, opened by
//! the one unsafe call. Like the timer watchers, the owner's callback runs
//! from the fast queue rather than from libuv's stack; the queued event
//! carries the signal and the callback, not the watcher, so it can outlive
//! the watcher.

#![deny(unsafe_op_in_unsafe_fn)]
// Unsafe perimeter: the `event/` row in docs/perimeter.md.
#![allow(unsafe_code)]

use core::cell::UnsafeCell;
use core::ffi::c_int;

use crate::event::libuv::{uv_close, uv_signal_init, uv_signal_start, uv_signal_stop};
use crate::event::r#loop::main_uv_loop;
use crate::event::objects::Event;
use crate::event::uv::{uv_handle_t, uv_signal_t};
use crate::event_queue::{self, QueueId};

/// What a watcher's owner is told: the signal that arrived.
pub(crate) type SignalCallback = fn(c_int);

/// Where a watcher is in its life with libuv.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum State {
    /// libuv has never seen the handle, or is done with it (the close
    /// callback has run).
    Unregistered,
    /// Registered with the loop: started or stopped, but open.
    Open,
    /// `uv_close` was called; libuv still holds the address until the close
    /// callback.
    Closing,
}

/// The watcher's words: libuv's handle, first, and what this module keeps
/// beside it.
#[repr(C)]
struct Watcher {
    uv: uv_signal_t,
    cb: Option<SignalCallback>,
    state: State,
}

/// A signal watcher as its owner holds it: in place, at the address libuv
/// is given when it is opened.
///
/// Every operation takes `&self`. libuv writes the handle behind any
/// reference to it, and the `UnsafeCell` is what lets a shared one coexist
/// with that; no `&mut` to a registered watcher is ever formed. Opening is
/// the one unsafe step, because from then on the owner promises the watcher
/// stays put until its close callback has run. The other operations are
/// gated on the watcher being open: starting one that is not is a bug and
/// panics, stopping or closing one does nothing.
#[repr(transparent)]
pub struct SignalWatcher(UnsafeCell<Watcher>);

impl core::fmt::Debug for SignalWatcher {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("SignalWatcher")
            .field("state", &self.state())
            .finish_non_exhaustive()
    }
}

impl SignalWatcher {
    /// A watcher libuv has not been given.
    pub(crate) const fn unopened() -> Self {
        // SAFETY: a libuv handle is plain C data -- pointers, integers and
        // `Option`s of function pointers -- for all of which all-zero is the
        // state `uv_signal_init` starts from.
        let uv: uv_signal_t = unsafe { core::mem::MaybeUninit::zeroed().assume_init() };
        Self(UnsafeCell::new(Watcher {
            uv,
            cb: None,
            state: State::Unregistered,
        }))
    }

    /// The handle libuv is given.
    fn uv(&self) -> *mut uv_signal_t {
        // SAFETY: a field of the live watcher, by place: no reference over
        // memory libuv writes.
        unsafe { &raw mut (*self.0.get()).uv }
    }

    fn state(&self) -> State {
        // SAFETY: a field libuv never writes, read by value; nothing holds a
        // reference to it (every write below is by place, too).
        unsafe { (*self.0.get()).state }
    }

    fn set_state(&self, state: State) {
        // SAFETY: as [`SignalWatcher::state`].
        unsafe { (*self.0.get()).state = state };
    }

    /// Register the watcher with the editor's loop, stopped.
    ///
    /// # Panics
    /// When the watcher is open, or still closing.
    ///
    /// # Safety
    /// The watcher stays at this address, alive, until its close callback
    /// has run (after [`SignalWatcher::close`] and the loop turn that
    /// finishes it).
    pub(crate) unsafe fn open(&self) {
        assert!(
            self.state() == State::Unregistered,
            "a signal watcher is opened only when libuv is done with it"
        );
        // SAFETY: the editor's loop, live for the process, and the
        // watcher's own handle, which the caller keeps in place.
        unsafe {
            uv_signal_init(main_uv_loop(), self.uv());
            (*self.uv()).data = self.0.get().cast();
        }
        self.set_state(State::Open);
    }

    /// Listen for `signum`, telling `cb` of each delivery.
    ///
    /// # Panics
    /// When the watcher is not open.
    pub(crate) fn start(&self, cb: SignalCallback, signum: c_int) {
        assert!(
            self.state() == State::Open,
            "a signal watcher is opened before it is started"
        );
        // SAFETY: a field libuv never writes, by place.
        unsafe { (*self.0.get()).cb = Some(cb) };
        // SAFETY: the watcher's own handle, open.
        unsafe { uv_signal_start(self.uv(), Some(signal_watcher_cb), signum) };
    }

    /// Stop listening; does nothing unless the watcher is open.
    pub(crate) fn stop(&self) {
        if self.state() == State::Open {
            // SAFETY: the watcher's own handle, open.
            unsafe { uv_signal_stop(self.uv()) };
        }
    }

    /// Close the handle; does nothing unless the watcher is open. libuv is
    /// done with it after the loop turn that runs its close callback.
    pub(crate) fn close(&self) {
        if self.state() == State::Open {
            self.set_state(State::Closing);
            // SAFETY: the watcher's own handle, open until this call.
            unsafe { uv_close(self.uv().cast(), Some(close_cb)) };
        }
    }
}

/// The watcher libuv's callbacks are handed, from the handle's `data`.
///
/// # Safety
/// `handle` is the handle of an opened watcher, which is still in place.
unsafe fn watcher_of<'a>(handle: *mut uv_signal_t) -> &'a SignalWatcher {
    // SAFETY: `open` put the watcher's address in `data`, and the watcher
    // is `repr(transparent)` over the cell at that address.
    unsafe { &*(*handle).data.cast::<SignalWatcher>() }
}

/// libuv: the signal arrived. Queued with the callback and the number the
/// handle watches now.
///
/// # Safety
/// libuv's signal callback: `handle` is an open watcher's own.
unsafe extern "C" fn signal_watcher_cb(handle: *mut uv_signal_t, _signum: c_int) {
    // SAFETY: the caller's promise.
    let watcher = unsafe { watcher_of(handle) };
    // SAFETY: fields read by value, by place: libuv is not writing them
    // while it runs this callback.
    let (cb, signum) = unsafe { ((*watcher.0.get()).cb, (*watcher.uv()).signum) };
    let notify = cb.expect("a started watcher has a callback");
    event_queue::put(QueueId::FAST, Event::Job(Box::new(move || notify(signum))));
}

/// libuv: the handle is closed, and libuv is done with the watcher.
///
/// # Safety
/// libuv's close callback: `handle` is a closing watcher's own, and this is
/// its last callback.
unsafe extern "C" fn close_cb(handle: *mut uv_handle_t) {
    // SAFETY: the caller's promise; a signal handle's address is its
    // `uv_signal_t`'s.
    let watcher = unsafe { watcher_of(handle.cast()) };
    watcher.set_state(State::Unregistered);
}
