//! Timer watchers: a libuv timer whose expiry is delivered as an event.
//!
//! The callback does not run from libuv's stack. It is queued on the
//! watcher's own [`MultiQueue`] (the loop's fast queue by default) and runs
//! when that queue is drained, so a timer cannot re-enter the editor from
//! inside `uv_run`. A watcher with no queue is driven synchronously and its
//! callback runs on the spot.

use core::ffi::c_void;

use crate::src::nvim::event::libuv::{uv_close, uv_timer_init, uv_timer_start, uv_timer_stop};
use crate::src::nvim::event::r#loop::one_arg_event;
use crate::src::nvim::event::multiqueue::{multiqueue_empty, multiqueue_put_event};
use crate::src::nvim::types::{Loop, TimeWatcher, time_cb, uv_handle_t, uv_timer_t};

/// Attach `watcher` to `uv_loop`, delivering its events on the fast queue.
pub unsafe fn time_watcher_init(uv_loop: *mut Loop, watcher: *mut TimeWatcher, data: *mut c_void) {
    uv_timer_init(&raw mut (*uv_loop).uv, &raw mut (*watcher).uv);
    (*watcher).uv.data = watcher.cast();
    (*watcher).data = data;
    (*watcher).events = (*uv_loop).fast_events;
    (*watcher).blockable = false;
}

/// Fire `cb` after `timeout` milliseconds, then every `repeat` (0 for once).
pub unsafe fn time_watcher_start(
    watcher: *mut TimeWatcher,
    cb: time_cb,
    timeout: u64,
    repeat: u64,
) {
    (*watcher).cb = cb;
    uv_timer_start(
        &raw mut (*watcher).uv,
        Some(time_watcher_cb),
        timeout,
        repeat,
    );
}

pub unsafe fn time_watcher_stop(watcher: *mut TimeWatcher) {
    uv_timer_stop(&raw mut (*watcher).uv);
}

/// Close the timer's handle; `cb` is told once libuv is done with it.
pub unsafe fn time_watcher_close(watcher: *mut TimeWatcher, cb: time_cb) {
    (*watcher).close_cb = cb;
    uv_close((&raw mut (*watcher).uv).cast(), Some(close_cb));
}

/// libuv: the timer expired.
///
/// A *blockable* timer skips a tick whose event would only queue up behind
/// one the editor has not processed yet — that is how the terminal refresh
/// timer keeps a flood of output from outrunning the redraw.
unsafe extern "C" fn time_watcher_cb(handle: *mut uv_timer_t) {
    let watcher: *mut TimeWatcher = (*handle).data.cast();
    if (*watcher).blockable && !multiqueue_empty((*watcher).events) {
        return;
    }
    if (*watcher).events.is_null() {
        let mut argv = [watcher.cast::<c_void>()];
        time_event(argv.as_mut_ptr());
    } else {
        multiqueue_put_event(
            (*watcher).events,
            one_arg_event(Some(time_event), watcher.cast()),
        );
    }
}

unsafe extern "C" fn time_event(argv: *mut *mut c_void) {
    let watcher: *mut TimeWatcher = (*argv).cast();
    let notify = (*watcher).cb.expect("a started timer has a callback");
    notify(watcher, (*watcher).data);
}

/// libuv: the handle is closed. Reported the same way as an expiry, so the
/// owner cannot see the close before events the timer already queued.
unsafe extern "C" fn close_cb(handle: *mut uv_handle_t) {
    let watcher: *mut TimeWatcher = (*handle).data.cast();
    if (*watcher).close_cb.is_none() {
        return;
    }
    if (*watcher).events.is_null() {
        let mut argv = [watcher.cast::<c_void>()];
        close_event(argv.as_mut_ptr());
    } else {
        multiqueue_put_event(
            (*watcher).events,
            one_arg_event(Some(close_event), watcher.cast()),
        );
    }
}

unsafe extern "C" fn close_event(argv: *mut *mut c_void) {
    let watcher: *mut TimeWatcher = (*argv).cast();
    let notify = (*watcher).close_cb.expect("checked by the caller");
    notify(watcher, (*watcher).data);
}
