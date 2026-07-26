//! Signal watchers: a libuv signal handle whose deliveries become events.
//!
//! Like the timer watchers, the owner's callback runs from a queue rather
//! than from libuv's stack. The close notification is the exception — it is
//! delivered directly, see [`close_cb`].

use core::ffi::{c_int, c_void};

use crate::src::nvim::event::libuv::{uv_close, uv_signal_init, uv_signal_start, uv_signal_stop};
use crate::src::nvim::event::r#loop::one_arg_event;
use crate::src::nvim::event::multiqueue::multiqueue_put_event;
use crate::src::nvim::types::{
    Loop, SignalWatcher, signal_cb, signal_close_cb, uv_handle_t, uv_signal_t,
};

/// Attach `watcher` to `uv_loop`, delivering its events on the fast queue.
pub unsafe fn signal_watcher_init(
    uv_loop: *mut Loop,
    watcher: *mut SignalWatcher,
    data: *mut c_void,
) {
    uv_signal_init(&raw mut (*uv_loop).uv, &raw mut (*watcher).uv);
    (*watcher).uv.data = watcher.cast();
    (*watcher).data = data;
    (*watcher).cb = None;
    (*watcher).events = (*uv_loop).fast_events;
}

/// Listen for `signum`, reporting each delivery to `cb`.
pub unsafe fn signal_watcher_start(watcher: *mut SignalWatcher, cb: signal_cb, signum: c_int) {
    (*watcher).cb = cb;
    uv_signal_start(&raw mut (*watcher).uv, Some(signal_watcher_cb), signum);
}

pub unsafe fn signal_watcher_stop(watcher: *mut SignalWatcher) {
    uv_signal_stop(&raw mut (*watcher).uv);
}

/// Close the watcher's handle; `cb` is told once libuv is done with it.
pub unsafe fn signal_watcher_close(watcher: *mut SignalWatcher, cb: signal_close_cb) {
    (*watcher).close_cb = cb;
    uv_close((&raw mut (*watcher).uv).cast(), Some(close_cb));
}

/// libuv: the signal arrived. The number is read back off the handle rather
/// than taken from the argument, so a watcher restarted on a different signal
/// between delivery and processing reports the one it is watching now.
unsafe extern "C" fn signal_watcher_cb(handle: *mut uv_signal_t, _signum: c_int) {
    let watcher: *mut SignalWatcher = (*handle).data.cast();
    if (*watcher).events.is_null() {
        let mut argv = [watcher.cast::<c_void>()];
        signal_event(argv.as_mut_ptr());
    } else {
        multiqueue_put_event(
            (*watcher).events,
            one_arg_event(Some(signal_event), watcher.cast()),
        );
    }
}

unsafe extern "C" fn signal_event(argv: *mut *mut c_void) {
    let watcher: *mut SignalWatcher = (*argv).cast();
    let notify = (*watcher).cb.expect("a started watcher has a callback");
    notify(watcher, (*watcher).uv.signum, (*watcher).data);
}

/// libuv: the handle is closed.
///
/// Unlike the timer watchers, this notification is *not* queued: upstream
/// calls the owner's callback straight from libuv's stack. Preserved —
/// `os/signal.rs` and `tui/tui.rs` both act on the watcher from here, and a
/// queued close would land after the loop has already been torn down.
unsafe extern "C" fn close_cb(handle: *mut uv_handle_t) {
    let watcher: *mut SignalWatcher = (*handle).data.cast();
    if let Some(notify) = (*watcher).close_cb {
        notify(watcher, (*watcher).data);
    }
}
