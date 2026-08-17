//! The editor's event loop: a libuv loop plus the multi-level event queue.
//!
//! `Loop` owns three queues. `events` is the deferred queue the editor drains
//! from its main state machine; `fast_events` is its child, drained as soon as
//! `uv_run` returns; `thread_events` is the mutex-guarded inbox other threads
//! post to, moved into `fast_events` by the async handle.

use crate::os::uv_error::UV_EBUSY;
use core::ffi::c_void;
use core::ptr;

use crate::event::libuv::{
    uv_async_init, uv_async_send, uv_close, uv_is_closing, uv_loop_close, uv_loop_init,
    uv_mutex_destroy, uv_mutex_init, uv_mutex_lock, uv_mutex_unlock, uv_run, uv_signal_init,
    uv_stop, uv_timer_init, uv_timer_start, uv_timer_stop,
};
use crate::event::multiqueue::{
    multiqueue_empty, multiqueue_free, multiqueue_move_events, multiqueue_new,
    multiqueue_new_child, multiqueue_process_events, multiqueue_purge_events, multiqueue_put_event,
    multiqueue_size,
};
use crate::log::{LOGLVL_ERR, log_uv_handles, logmsg_c};
use crate::os::libc::abort;
use crate::os::time::os_hrtime;
use crate::types::{
    Event, Loop, MultiQueue, Proc, argv_callback, uv_async_t, uv_handle_t, uv_run_mode, uv_timer_t,
};

const UV_RUN_DEFAULT: uv_run_mode = 0;
const UV_RUN_ONCE: uv_run_mode = 1;
const UV_RUN_NOWAIT: uv_run_mode = 2;

unsafe extern "C" {
    fn uv_walk(uv_loop: *mut crate::types::uv_loop_t, walk_cb: uv_walk_cb, arg: *mut c_void);
}

type uv_walk_cb = Option<unsafe extern "C" fn(*mut uv_handle_t, *mut c_void)>;

// ---------------------------------------------------------------------------
// Lifetime
// ---------------------------------------------------------------------------

/// Bring `uv_loop` up: the libuv loop, the three queues, the mutex and every
/// handle the job-control and polling machinery needs.
pub unsafe fn loop_init(uv_loop: *mut Loop) {
    uv_loop_init(&raw mut (*uv_loop).uv);
    (*uv_loop).recursive = 0;
    (*uv_loop).closing = false;
    (*uv_loop).uv.data = uv_loop.cast();
    (*uv_loop).children = Box::into_raw(Box::new(Vec::<*mut Proc>::new()));
    (*uv_loop).events = multiqueue_new(Some(loop_on_put), uv_loop.cast());
    (*uv_loop).fast_events = multiqueue_new_child((*uv_loop).events);
    (*uv_loop).thread_events = multiqueue_new(None, ptr::null_mut());
    uv_mutex_init(&raw mut (*uv_loop).mutex);
    uv_async_init(
        &raw mut (*uv_loop).uv,
        &raw mut (*uv_loop).async_0,
        Some(async_cb),
    );
    uv_signal_init(&raw mut (*uv_loop).uv, &raw mut (*uv_loop).children_watcher);
    uv_timer_init(
        &raw mut (*uv_loop).uv,
        &raw mut (*uv_loop).children_kill_timer,
    );
    uv_timer_init(&raw mut (*uv_loop).uv, &raw mut (*uv_loop).poll_timer);
    uv_timer_init(&raw mut (*uv_loop).uv, &raw mut (*uv_loop).exit_delay_timer);
    // The poll timer's only job is to flip this flag; it outlives every
    // `loop_uv_run` and is released by the handle's close callback.
    (*uv_loop).poll_timer.data = Box::into_raw(Box::new(false)).cast();
}

/// Close every handle and release the queues. `wait` gives libuv up to two
/// seconds to finish; returns false if it did not.
pub unsafe fn loop_close(uv_loop: *mut Loop, wait: bool) -> bool {
    let mut rv = true;
    (*uv_loop).closing = true;
    uv_mutex_destroy(&raw mut (*uv_loop).mutex);
    uv_close((&raw mut (*uv_loop).children_watcher).cast(), None);
    uv_close((&raw mut (*uv_loop).children_kill_timer).cast(), None);
    uv_close(
        (&raw mut (*uv_loop).poll_timer).cast(),
        Some(timer_close_cb),
    );
    uv_close((&raw mut (*uv_loop).exit_delay_timer).cast(), None);
    uv_close((&raw mut (*uv_loop).async_0).cast(), None);

    let start = if wait { os_hrtime() } else { 0 };
    let mut didstop = false;
    loop {
        uv_run(
            &raw mut (*uv_loop).uv,
            if didstop {
                UV_RUN_DEFAULT
            } else {
                UV_RUN_NOWAIT
            },
        );
        if uv_loop_close(&raw mut (*uv_loop).uv) != UV_EBUSY || !wait {
            break;
        }
        if os_hrtime().wrapping_sub(start).wrapping_div(1_000_000_000) >= 2 {
            rv = false;
            logmsg_c!(
                LOGLVL_ERR,
                ptr::null(),
                c"loop_close".as_ptr(),
                172,
                true,
                c"uv_loop_close() hang?".as_ptr(),
            );
            log_uv_handles((&raw mut (*uv_loop).uv).cast());
            break;
        }
        if !didstop {
            uv_stop(&raw mut (*uv_loop).uv);
            uv_walk(&raw mut (*uv_loop).uv, Some(loop_walk_cb), ptr::null_mut());
            didstop = true;
        }
    }

    multiqueue_free((*uv_loop).fast_events);
    multiqueue_free((*uv_loop).thread_events);
    multiqueue_free((*uv_loop).events);
    drop(Box::from_raw(loop_children(uv_loop)));
    (*uv_loop).children = ptr::null_mut();
    rv
}

/// The list of job-control children.
///
/// Handed out as a pointer, not a borrow: closing a child's handles re-enters
/// this list, so callers take a momentary borrow around each access.
pub unsafe fn loop_children(uv_loop: *mut Loop) -> *mut Vec<*mut Proc> {
    (*uv_loop).children
}

/// Close every handle libuv still knows about, so `uv_loop_close` can succeed.
unsafe extern "C" fn loop_walk_cb(handle: *mut uv_handle_t, _arg: *mut c_void) {
    if uv_is_closing(handle) == 0 {
        uv_close(handle, None);
    }
}

// ---------------------------------------------------------------------------
// Polling
// ---------------------------------------------------------------------------

/// Run libuv once, then drain the fast queue. Returns true if `ms` elapsed
/// before anything else woke the loop.
///
/// Recursion is a bug rather than a supported mode: nvim aborts on it.
pub unsafe fn loop_poll_events(uv_loop: *mut Loop, ms: i64) -> bool {
    if (*uv_loop).recursive > 0 {
        abort();
    }
    (*uv_loop).recursive += 1;

    let timeout_expired: *mut bool = (*uv_loop).poll_timer.data.cast();
    *timeout_expired = false;
    let mut mode = UV_RUN_ONCE;
    if ms > 0 {
        uv_timer_start(
            &raw mut (*uv_loop).poll_timer,
            Some(timer_cb),
            ms as u64,
            ms as u64,
        );
    } else if ms == 0 {
        mode = UV_RUN_NOWAIT;
    }
    uv_run(&raw mut (*uv_loop).uv, mode);
    if ms > 0 {
        uv_timer_stop(&raw mut (*uv_loop).poll_timer);
    }
    (*uv_loop).recursive -= 1;

    let expired = *timeout_expired;
    multiqueue_process_events((*uv_loop).fast_events);
    expired
}

/// One pass of upstream's `LOOP_PROCESS_EVENTS`: drain `queue` if it has
/// anything, otherwise let the loop poll for `timeout` milliseconds.
pub unsafe fn process_events(uv_loop: *mut Loop, queue: *mut MultiQueue, timeout: i64) {
    if !queue.is_null() && !multiqueue_empty(queue) {
        multiqueue_process_events(queue);
    } else {
        loop_poll_events(uv_loop, timeout);
    }
}

/// Upstream's `LOOP_PROCESS_EVENTS_UNTIL`: run passes until `done` holds or
/// the millisecond budget runs out.
///
/// `done` is re-evaluated between passes and may observe state that the pass
/// itself changed, which is the whole point — a child's refcount reaching one,
/// or input arriving, is what ends most of these waits.
pub unsafe fn process_events_until(
    uv_loop: *mut Loop,
    queue: *mut MultiQueue,
    ms: i64,
    mut done: impl FnMut() -> bool,
) {
    let mut budget = Budget::new(ms, os_hrtime);
    while !done() {
        process_events(uv_loop, queue, budget.remaining());
        if !budget.charge(os_hrtime) {
            break;
        }
    }
}

/// The millisecond budget of a wait loop.
///
/// A budget of zero means "one pass, then stop"; a negative one is unlimited.
/// The clock is only read when the budget is finite, which is why `now` is
/// taken as a closure rather than a value.
pub struct Budget {
    remaining: i64,
    /// The reading `remaining` was last charged against.
    before: u64,
}

impl Budget {
    pub fn new(ms: i64, now: impl FnOnce() -> u64) -> Self {
        Budget {
            remaining: ms,
            before: if ms > 0 { now() } else { 0 },
        }
    }

    pub fn remaining(&self) -> i64 {
        self.remaining
    }

    /// Charge the time elapsed since the last call. Returns false once the
    /// budget is spent and the loop should stop.
    pub fn charge(&mut self, now: impl FnOnce() -> u64) -> bool {
        if self.remaining == 0 {
            return false;
        }
        if self.remaining < 0 {
            return true;
        }
        let now = now();
        self.remaining -= now.wrapping_sub(self.before).wrapping_div(1_000_000) as i64;
        self.before = now;
        self.remaining > 0
    }
}

/// The poll timer fired, so `loop_poll_events` waited out its whole timeout.
unsafe extern "C" fn timer_cb(handle: *mut uv_timer_t) {
    *(*handle).data.cast::<bool>() = true;
}

/// The poll timer is gone; release the flag it wrote to.
unsafe extern "C" fn timer_close_cb(handle: *mut uv_handle_t) {
    drop(Box::from_raw((*handle).data.cast::<bool>()));
}

// ---------------------------------------------------------------------------
// Posting events
// ---------------------------------------------------------------------------

/// An [`Event`] carrying a single argument, which is all most handlers take.
pub fn one_arg_event(handler: argv_callback, arg: *mut c_void) -> Event {
    let mut argv = [ptr::null_mut::<c_void>(); 10];
    argv[0] = arg;
    Event { handler, argv }
}

/// Post `event` to the fast queue from any thread.
pub unsafe fn loop_schedule_fast(uv_loop: *mut Loop, event: Event) {
    uv_mutex_lock(&raw mut (*uv_loop).mutex);
    multiqueue_put_event((*uv_loop).thread_events, event);
    uv_async_send(&raw mut (*uv_loop).async_0);
    uv_mutex_unlock(&raw mut (*uv_loop).mutex);
}

/// Post `event` to the deferred queue from any thread.
///
/// The event has to travel through the fast queue to get there — only the
/// main thread may touch `loop->events` — so it is boxed and unwrapped by
/// [`loop_deferred_event`] once it lands.
pub unsafe fn loop_schedule_deferred(uv_loop: *mut Loop, event: Event) {
    let mut argv = [ptr::null_mut::<c_void>(); 10];
    argv[0] = uv_loop.cast();
    argv[1] = Box::into_raw(Box::new(event)).cast();
    loop_schedule_fast(
        uv_loop,
        Event {
            handler: Some(loop_deferred_event),
            argv,
        },
    );
}

unsafe extern "C" fn loop_deferred_event(argv: *mut *mut c_void) {
    let uv_loop: *mut Loop = (*argv.add(0)).cast();
    let event = Box::from_raw((*argv.add(1)).cast::<Event>());
    multiqueue_put_event((*uv_loop).events, *event);
}

/// Break out of `uv_run` when an event lands while the loop is running, so
/// the caller gets a chance to process it.
pub unsafe extern "C" fn loop_on_put(_queue: *mut MultiQueue, data: *mut c_void) {
    let uv_loop: *mut Loop = data.cast();
    if (*uv_loop).recursive != 0 {
        uv_stop(&raw mut (*uv_loop).uv);
    }
}

/// Another thread posted to `thread_events`; move the lot to `fast_events`.
unsafe extern "C" fn async_cb(handle: *mut uv_async_t) {
    let uv_loop: *mut Loop = (*(*handle).loop_0).data.cast();
    uv_mutex_lock(&raw mut (*uv_loop).mutex);
    multiqueue_move_events((*uv_loop).fast_events, (*uv_loop).thread_events);
    uv_mutex_unlock(&raw mut (*uv_loop).mutex);
}

// ---------------------------------------------------------------------------
// Queue inspection
// ---------------------------------------------------------------------------

/// Drop everything queued but not yet processed.
pub unsafe fn loop_purge(uv_loop: *mut Loop) {
    uv_mutex_lock(&raw mut (*uv_loop).mutex);
    multiqueue_purge_events((*uv_loop).thread_events);
    multiqueue_purge_events((*uv_loop).fast_events);
    uv_mutex_unlock(&raw mut (*uv_loop).mutex);
}

/// How many events other threads have posted and the loop has not moved yet.
pub unsafe fn loop_size(uv_loop: *mut Loop) -> usize {
    uv_mutex_lock(&raw mut (*uv_loop).mutex);
    let rv = multiqueue_size((*uv_loop).thread_events);
    uv_mutex_unlock(&raw mut (*uv_loop).mutex);
    rv
}

#[cfg(test)]
mod tests {
    use super::Budget;

    #[test]
    fn an_unlimited_budget_never_expires() {
        let mut budget = Budget::new(-1, || panic!("the clock is not read"));
        assert_eq!(budget.remaining(), -1);
        assert!(budget.charge(|| panic!("the clock is not read")));
        assert!(budget.charge(|| panic!("the clock is not read")));
    }

    #[test]
    fn a_zero_budget_stops_after_one_pass() {
        let mut budget = Budget::new(0, || panic!("the clock is not read"));
        assert_eq!(budget.remaining(), 0);
        assert!(!budget.charge(|| panic!("the clock is not read")));
    }

    #[test]
    fn a_finite_budget_is_charged_in_whole_milliseconds() {
        let mut budget = Budget::new(10, || 1_000_000_000);
        // 2.9ms rounds down to 2.
        assert!(budget.charge(|| 1_002_900_000));
        assert_eq!(budget.remaining(), 8);
        // Charging is against the previous reading, not the start.
        assert!(budget.charge(|| 1_005_900_000));
        assert_eq!(budget.remaining(), 5);
    }

    #[test]
    fn a_finite_budget_stops_when_it_reaches_zero() {
        let mut budget = Budget::new(5, || 0);
        assert!(!budget.charge(|| 5_000_000));
        assert_eq!(budget.remaining(), 0);
    }

    #[test]
    fn an_overspent_budget_stops_rather_than_wrapping() {
        let mut budget = Budget::new(5, || 0);
        assert!(!budget.charge(|| 50_000_000));
        assert_eq!(budget.remaining(), -45);
    }
}
