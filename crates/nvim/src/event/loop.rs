//! The editor's event loop: a libuv loop plus the multi-level event queue.
//!
//! `Loop` owns three queues. `events` is the deferred queue the editor drains
//! from its main state machine; `fast_events` is its child, drained as soon as
//! `uv_run` returns; `thread_events` is the mutex-guarded inbox other threads
//! post to, moved into `fast_events` by the async handle.
//!
//! Every subsystem in `event/` reaches the loop through a raw pointer libuv
//! also holds — in the loop's own `data` field, and in each handle's. That is
//! what [`EventLoop`] is for: the pointer is wrapped once, paying the `unsafe`
//! at construction, and the field accesses below are ordinary Rust. Nothing
//! holds a borrow across `uv_run`, which re-enters this module.

#![deny(unsafe_op_in_unsafe_fn)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use crate::os::uv_error::UV_EBUSY;
use core::ffi::c_void;
use core::ops::{Deref, DerefMut};
use core::ptr;

use crate::event::libuv::{
    uv_async_init, uv_async_send, uv_close, uv_is_closing, uv_loop_close, uv_loop_init,
    uv_mutex_destroy, uv_mutex_init, uv_mutex_lock, uv_mutex_unlock, uv_print_all_handles, uv_run,
    uv_signal_init, uv_stop, uv_timer_init, uv_timer_start, uv_timer_stop, uv_walk,
};
use crate::event::multiqueue::{
    multiqueue_empty, multiqueue_free, multiqueue_move_events, multiqueue_new,
    multiqueue_new_child, multiqueue_process_events, multiqueue_purge_events, multiqueue_put_event,
    multiqueue_size,
};
use crate::log::{LOGLVL_ERR, log_file_path, logmsg, with_log_lock};
use crate::os::cshim::stderr;
use crate::os::time::os_hrtime;
use crate::types::{
    Event, Loop, MultiQueue, Proc, argv_callback, uv_async_t, uv_handle_t, uv_loop_t, uv_mutex_t,
    uv_run_mode, uv_signal_t, uv_timer_t,
};
use ::libc::{abort, fclose, fopen};

const UV_RUN_DEFAULT: uv_run_mode = 0;
const UV_RUN_ONCE: uv_run_mode = 1;
const UV_RUN_NOWAIT: uv_run_mode = 2;

/// The loop, reached through the raw pointer its subsystems hold.
///
/// A loop is created once and never moves — libuv keeps its address in the
/// `data` field of every handle registered with it — so wrapping that pointer
/// is the only unsafe step, and the accessors below derive their handle
/// pointers from the wrapped pointer rather than from a borrow of it.
#[derive(Copy, Clone)]
pub struct EventLoop(*mut Loop);

impl EventLoop {
    /// # Safety
    /// `uv_loop` is a live loop, through [`loop_init`] and not yet through
    /// [`loop_close`], that outlives every use of this handle.
    pub unsafe fn new(uv_loop: *mut Loop) -> Self {
        debug_assert!(!uv_loop.is_null());
        EventLoop(uv_loop)
    }

    /// The pointer back, for the callers that still pass one around.
    pub fn as_ptr(self) -> *mut Loop {
        self.0
    }

    /// The libuv loop every handle in `event/` is registered with.
    pub fn uv(self) -> *mut uv_loop_t {
        // SAFETY: a field of the live loop, derived from the wrapped pointer.
        unsafe { &raw mut (*self.0).uv }
    }

    /// The mutex guarding `thread_events`. See [`with_mutex`].
    fn mutex(self) -> *mut uv_mutex_t {
        // SAFETY: a field of the live loop.
        unsafe { &raw mut (*self.0).mutex }
    }

    /// The handle other threads wake the loop with.
    fn wakeup(self) -> *mut uv_async_t {
        // SAFETY: a field of the live loop.
        unsafe { &raw mut (*self.0).async_0 }
    }

    /// The timer that bounds one [`loop_poll_events`].
    fn poll_timer(self) -> *mut uv_timer_t {
        // SAFETY: a field of the live loop.
        unsafe { &raw mut (*self.0).poll_timer }
    }

    /// The `SIGCHLD` watcher the job-control layer arms.
    fn children_watcher(self) -> *mut uv_signal_t {
        // SAFETY: a field of the live loop.
        unsafe { &raw mut (*self.0).children_watcher }
    }

    /// The one timer that kills every child that outstayed `proc_stop`.
    fn kill_timer(self) -> *mut uv_timer_t {
        // SAFETY: a field of the live loop.
        unsafe { &raw mut (*self.0).children_kill_timer }
    }

    /// The timer that bounds how long exiting waits on the children.
    fn exit_delay_timer(self) -> *mut uv_timer_t {
        // SAFETY: a field of the live loop.
        unsafe { &raw mut (*self.0).exit_delay_timer }
    }

    /// Whether the last [`loop_poll_events`] ran out its whole timeout.
    ///
    /// The flag is a `Box<bool>` in the poll timer's `data`: it outlives every
    /// poll and is released by the timer's close callback. It stays a raw
    /// pointer because `timer_cb` writes to it from inside `uv_run`.
    fn timeout_expired(self) -> *mut bool {
        // SAFETY: `loop_init` boxed the flag and only `timer_close_cb`
        // releases it.
        unsafe { (*self.0).poll_timer.data.cast() }
    }
}

impl Deref for EventLoop {
    type Target = Loop;

    fn deref(&self) -> &Loop {
        // SAFETY: the promise made at construction.
        unsafe { &*self.0 }
    }
}

impl DerefMut for EventLoop {
    fn deref_mut(&mut self) -> &mut Loop {
        // SAFETY: the promise made at construction.
        unsafe { &mut *self.0 }
    }
}

/// Run `body` with the loop's mutex held.
///
/// `thread_events` is the only state other threads touch, so it is the only
/// state this guards. A panic inside `body` leaves the mutex locked, exactly
/// as the C's early `return` would have.
fn with_mutex<T>(uv_loop: EventLoop, body: impl FnOnce() -> T) -> T {
    // SAFETY: the mutex is live from `loop_init` until `loop_close`.
    unsafe { uv_mutex_lock(uv_loop.mutex()) };
    let rv = body();
    // SAFETY: as above, and this thread holds the lock.
    unsafe { uv_mutex_unlock(uv_loop.mutex()) };
    rv
}

// ---------------------------------------------------------------------------
// Lifetime
// ---------------------------------------------------------------------------

/// Bring `uv_loop` up: the libuv loop, the three queues, the mutex and every
/// handle the job-control and polling machinery needs.
///
/// # Safety
/// `uv_loop` points at storage for a loop that does not move afterwards, and
/// has not already been initialised.
pub unsafe fn loop_init(uv_loop: *mut Loop) {
    // SAFETY: the caller's storage, filled in below before anything reads it.
    let mut uv_loop = unsafe { EventLoop::new(uv_loop) };
    // SAFETY: a fresh loop.
    unsafe { uv_loop_init(uv_loop.uv()) };
    uv_loop.recursive = 0;
    uv_loop.closing = false;
    uv_loop.uv.data = uv_loop.as_ptr().cast();
    uv_loop.children = Box::into_raw(Box::new(Vec::<*mut Proc>::new()));
    // SAFETY: the queues are this loop's own, and `loop_on_put` is called
    // with the loop it is installed on.
    unsafe {
        uv_loop.events = multiqueue_new(Some(loop_on_put), uv_loop.as_ptr().cast());
        uv_loop.fast_events = multiqueue_new_child(uv_loop.events);
        uv_loop.thread_events = multiqueue_new(None, ptr::null_mut());
    }
    // SAFETY: the mutex and every handle below are fields of the loop just
    // initialised, and each is registered with that same loop.
    unsafe {
        uv_mutex_init(uv_loop.mutex());
        uv_async_init(uv_loop.uv(), uv_loop.wakeup(), Some(async_cb));
        uv_signal_init(uv_loop.uv(), uv_loop.children_watcher());
        uv_timer_init(uv_loop.uv(), uv_loop.kill_timer());
        uv_timer_init(uv_loop.uv(), uv_loop.poll_timer());
        uv_timer_init(uv_loop.uv(), uv_loop.exit_delay_timer());
    }
    // The poll timer's only job is to flip this flag; it outlives every
    // `loop_uv_run` and is released by the handle's close callback.
    uv_loop.poll_timer.data = Box::into_raw(Box::new(false)).cast();
}

/// Close every handle and release the queues. `wait` gives libuv up to two
/// seconds to finish; returns false if it did not.
///
/// # Safety
/// `uv_loop` is live and nothing is still using it.
pub unsafe fn loop_close(uv_loop: *mut Loop, wait: bool) -> bool {
    // SAFETY: the caller's loop.
    let mut uv_loop = unsafe { EventLoop::new(uv_loop) };
    uv_loop.closing = true;
    // SAFETY: the loop's own mutex and handles, each still open.
    unsafe {
        uv_mutex_destroy(uv_loop.mutex());
        uv_close(uv_loop.children_watcher().cast(), None);
        uv_close(uv_loop.kill_timer().cast(), None);
        uv_close(uv_loop.poll_timer().cast(), Some(timer_close_cb));
        uv_close(uv_loop.exit_delay_timer().cast(), None);
        uv_close(uv_loop.wakeup().cast(), None);
    }

    let rv = drain_until_closed(uv_loop, wait);

    // SAFETY: the three queues this loop made, and the children list it
    // boxed. Nothing reaches any of them after the loop is closed.
    unsafe {
        multiqueue_free(uv_loop.fast_events);
        multiqueue_free(uv_loop.thread_events);
        multiqueue_free(uv_loop.events);
        drop(Box::from_raw(uv_loop.children));
    }
    uv_loop.children = ptr::null_mut();
    rv
}

/// Run libuv until it lets go of the loop, stopping and walking its handles
/// if a first pass does not. Returns false if it was still busy after two
/// seconds; `wait` false gives it a single pass.
fn drain_until_closed(uv_loop: EventLoop, wait: bool) -> bool {
    let start = if wait { os_hrtime() } else { 0 };
    let mut didstop = false;
    loop {
        let mode = if didstop {
            UV_RUN_DEFAULT
        } else {
            UV_RUN_NOWAIT
        };
        // SAFETY: the loop is live until `uv_loop_close` accepts it, and this
        // is the pass that lets its handles finish closing.
        let busy = unsafe {
            uv_run(uv_loop.uv(), mode);
            uv_loop_close(uv_loop.uv()) == UV_EBUSY
        };
        if !busy || !wait {
            return true;
        }
        if os_hrtime().wrapping_sub(start).wrapping_div(1_000_000_000) >= 2 {
            logmsg!(LOGLVL_ERR, c"loop_close", 172, "uv_loop_close() hang?");
            // SAFETY: the loop is still readable, and `log_uv_handles` takes
            // the log's lock around the write.
            unsafe { log_uv_handles(uv_loop.uv()) };
            return false;
        }
        if !didstop {
            // SAFETY: as above; `loop_walk_cb` closes what it is handed.
            unsafe {
                uv_stop(uv_loop.uv());
                uv_walk(uv_loop.uv(), Some(loop_walk_cb), ptr::null_mut());
            }
            didstop = true;
        }
    }
}

/// The list of job-control children.
///
/// Handed out as a pointer, not a borrow: closing a child's handles re-enters
/// this list, so callers take a momentary borrow around each access.
///
/// # Safety
/// `uv_loop` is live.
pub unsafe fn loop_children(uv_loop: *mut Loop) -> *mut Vec<*mut Proc> {
    // SAFETY: the caller's loop.
    unsafe { EventLoop::new(uv_loop) }.children
}

/// Close every handle libuv still knows about, so `uv_loop_close` can succeed.
///
/// # Safety
/// libuv's walk callback: `handle` is one of the loop's own.
unsafe extern "C" fn loop_walk_cb(handle: *mut uv_handle_t, _arg: *mut c_void) {
    // SAFETY: libuv hands back a handle of the loop being walked.
    unsafe {
        if uv_is_closing(handle) == 0 {
            uv_close(handle, None);
        }
    }
}

// ---------------------------------------------------------------------------
// Polling
// ---------------------------------------------------------------------------

/// Run libuv once, then drain the fast queue. Returns true if `ms` elapsed
/// before anything else woke the loop.
///
/// Recursion is a bug rather than a supported mode: nvim aborts on it.
///
/// # Safety
/// `uv_loop` is live, and so is everything its ready handles reach.
pub unsafe fn loop_poll_events(uv_loop: *mut Loop, ms: i64) -> bool {
    // SAFETY: the caller's loop.
    let mut uv_loop = unsafe { EventLoop::new(uv_loop) };
    if uv_loop.recursive > 0 {
        // SAFETY: `abort` does not return.
        unsafe { abort() };
    }
    uv_loop.recursive += 1;

    let timeout_expired = uv_loop.timeout_expired();
    // SAFETY: the flag `loop_init` boxed, written here and by `timer_cb`.
    unsafe { *timeout_expired = false };
    let mut mode = UV_RUN_ONCE;
    if ms > 0 {
        // SAFETY: the loop's own timer; `ms` is positive, so the cast is
        // exact.
        unsafe {
            uv_timer_start(
                uv_loop.poll_timer(),
                Some(timer_cb),
                ms.cast_unsigned(),
                ms.cast_unsigned(),
            );
        }
    } else if ms == 0 {
        mode = UV_RUN_NOWAIT;
    }
    // SAFETY: the caller's promise about what the ready handles reach.
    unsafe { uv_run(uv_loop.uv(), mode) };
    if ms > 0 {
        // SAFETY: the timer started above.
        unsafe { uv_timer_stop(uv_loop.poll_timer()) };
    }
    uv_loop.recursive -= 1;

    // Read before the queue is drained: a handler may poll again.
    // SAFETY: as above.
    let expired = unsafe { *timeout_expired };
    // SAFETY: the loop's own queue.
    unsafe { multiqueue_process_events(uv_loop.fast_events) };
    expired
}

/// One pass of upstream's `LOOP_PROCESS_EVENTS`: drain `queue` if it has
/// anything, otherwise let the loop poll for `timeout` milliseconds.
///
/// # Safety
/// `uv_loop` is live, `queue` is live or null, and so is everything their
/// events reach.
pub unsafe fn process_events(uv_loop: *mut Loop, queue: *mut MultiQueue, timeout: i64) {
    // SAFETY: the caller's queue and loop.
    unsafe {
        if !queue.is_null() && !multiqueue_empty(queue) {
            multiqueue_process_events(queue);
        } else {
            loop_poll_events(uv_loop, timeout);
        }
    }
}

/// Upstream's `LOOP_PROCESS_EVENTS_UNTIL`: run passes until `done` holds or
/// the millisecond budget runs out.
///
/// `done` is re-evaluated between passes and may observe state that the pass
/// itself changed, which is the whole point — a child's refcount reaching one,
/// or input arriving, is what ends most of these waits.
///
/// # Safety
/// As [`process_events`].
pub unsafe fn process_events_until(
    uv_loop: *mut Loop,
    queue: *mut MultiQueue,
    ms: i64,
    mut done: impl FnMut() -> bool,
) {
    let mut budget = Budget::new(ms, os_hrtime);
    while !done() {
        // SAFETY: the caller's promise.
        unsafe { process_events(uv_loop, queue, budget.remaining()) };
        // SAFETY: as above.
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
        self.remaining -= now
            .wrapping_sub(self.before)
            .wrapping_div(1_000_000)
            .cast_signed();
        self.before = now;
        self.remaining > 0
    }
}

/// The poll timer fired, so `loop_poll_events` waited out its whole timeout.
///
/// # Safety
/// libuv's timer callback: `handle` is the poll timer, whose `data` is the
/// flag [`loop_init`] boxed.
unsafe extern "C" fn timer_cb(handle: *mut uv_timer_t) {
    // SAFETY: libuv hands back the timer `loop_poll_events` started, whose
    // `data` is the boxed flag `loop_init` left there.
    unsafe { *(*handle).data.cast::<bool>() = true };
}

/// The poll timer is gone; release the flag it wrote to.
///
/// # Safety
/// libuv's close callback, registered on the poll timer alone, and called
/// once.
unsafe extern "C" fn timer_close_cb(handle: *mut uv_handle_t) {
    // SAFETY: as above, and this is the timer's last callback.
    drop(unsafe { Box::from_raw((*handle).data.cast::<bool>()) });
}

// ---------------------------------------------------------------------------
// Posting events
// ---------------------------------------------------------------------------

/// An [`Event`] carrying a single argument, which is all most handlers take.
pub fn one_arg_event(handler: argv_callback, arg: *mut c_void) -> Event {
    Event::new(handler, [arg])
}

/// Post `event` to the fast queue from any thread.
///
/// # Safety
/// `uv_loop` is live, and `event`'s handler is safe to call with its argv
/// once the loop reaches it.
pub unsafe fn loop_schedule_fast(uv_loop: *mut Loop, event: Event) {
    // SAFETY: the caller's loop.
    let uv_loop = unsafe { EventLoop::new(uv_loop) };
    with_mutex(uv_loop, || {
        // SAFETY: the inbox is this loop's own, and the async handle is what
        // tells the loop's thread to drain it.
        unsafe {
            multiqueue_put_event(uv_loop.thread_events, event);
            uv_async_send(uv_loop.wakeup());
        }
    });
}

/// Post `event` to the deferred queue from any thread.
///
/// The event has to travel through the fast queue to get there — only the
/// main thread may touch `loop->events` — so it is boxed and unwrapped by
/// [`loop_deferred_event`] once it lands.
///
/// # Safety
/// As [`loop_schedule_fast`].
pub unsafe fn loop_schedule_deferred(uv_loop: *mut Loop, event: Event) {
    let boxed = Box::into_raw(Box::new(event)).cast::<c_void>();
    let wrapper = Event::new(Some(loop_deferred_event), [uv_loop.cast(), boxed]);
    // SAFETY: the caller's loop; `loop_deferred_event` is called with the two
    // arguments packed just above.
    unsafe { loop_schedule_fast(uv_loop, wrapper) };
}

/// The fast queue reached the wrapper [`loop_schedule_deferred`] made:
/// unpack the caller's event onto the deferred queue.
///
/// # Safety
/// `argv` is that wrapper's — the loop, then the boxed event.
unsafe extern "C" fn loop_deferred_event(argv: *mut *mut c_void) {
    // SAFETY: the two arguments `loop_schedule_deferred` packed — the loop,
    // and the caller's event, boxed there and taken back here.
    let uv_loop = unsafe { EventLoop::new((*argv).cast()) };
    // SAFETY: as above.
    let event = unsafe { Box::from_raw((*argv.add(1)).cast::<Event>()) };
    // SAFETY: the loop's own deferred queue.
    unsafe { multiqueue_put_event(uv_loop.events, *event) };
}

/// Break out of `uv_run` when an event lands while the loop is running, so
/// the caller gets a chance to process it.
///
/// # Safety
/// `data` is the loop this callback was installed for.
pub unsafe extern "C" fn loop_on_put(_queue: *mut MultiQueue, data: *mut c_void) {
    // SAFETY: the caller's promise.
    let uv_loop = unsafe { EventLoop::new(data.cast()) };
    if uv_loop.recursive != 0 {
        // SAFETY: the loop is live and this thread is inside its `uv_run`.
        unsafe { uv_stop(uv_loop.uv()) };
    }
}

/// Another thread posted to `thread_events`; move the lot to `fast_events`.
///
/// # Safety
/// libuv's async callback: `handle` is the one [`loop_init`] registered, on
/// the loop whose address it left in the libuv loop's `data`.
unsafe extern "C" fn async_cb(handle: *mut uv_async_t) {
    // SAFETY: libuv hands back the handle `loop_init` registered, and
    // `loop_init` put the loop's own address in the libuv loop's `data`.
    let uv_loop = unsafe { EventLoop::new((*(*handle).loop_0).data.cast()) };
    with_mutex(uv_loop, || {
        // SAFETY: both queues are this loop's own.
        unsafe { multiqueue_move_events(uv_loop.fast_events, uv_loop.thread_events) };
    });
}

// ---------------------------------------------------------------------------
// Queue inspection
// ---------------------------------------------------------------------------

/// Drop everything queued but not yet processed.
///
/// # Safety
/// `uv_loop` is live.
pub unsafe fn loop_purge(uv_loop: *mut Loop) {
    // SAFETY: the caller's loop.
    let uv_loop = unsafe { EventLoop::new(uv_loop) };
    with_mutex(uv_loop, || {
        // SAFETY: both queues are this loop's own.
        unsafe {
            multiqueue_purge_events(uv_loop.thread_events);
            multiqueue_purge_events(uv_loop.fast_events);
        }
    });
}

/// How many events other threads have posted and the loop has not moved yet.
///
/// # Safety
/// `uv_loop` is live.
pub unsafe fn loop_size(uv_loop: *mut Loop) -> usize {
    // SAFETY: the caller's loop.
    let uv_loop = unsafe { EventLoop::new(uv_loop) };
    // SAFETY: the inbox is this loop's own.
    with_mutex(uv_loop, || unsafe {
        multiqueue_size(uv_loop.thread_events)
    })
}

/// Dump libuv's handle table to the log — the `:checkhealth`-adjacent view
/// of what the event loop is still holding.
///
/// This is the one caller that needs the log as a `FILE *`, because
/// `uv_print_all_handles` takes one; `log.rs` writes bytes and hands out only
/// the path. It sat there for that reason and now sits here, on the side of
/// the boundary that already makes libuv calls.
///
/// # Safety
/// `uv_loop` is a live `uv_loop_t *`.
pub(crate) unsafe fn log_uv_handles(uv_loop: *mut uv_loop_t) {
    with_log_lock(|| {
        let path = log_file_path();
        // SAFETY: a NUL-terminated path held alive by `path`; the handle is
        // closed below and never escapes.
        let opened = path
            .as_ref()
            .map(|path| unsafe { fopen(path.as_ptr(), c"a".as_ptr()) })
            .filter(|file| !file.is_null());
        // SAFETY: `stderr` is open for the life of the process.
        let out = opened.unwrap_or(unsafe { stderr });
        // SAFETY: the caller's loop, and a handle open for writing.
        unsafe { uv_print_all_handles(uv_loop, out) };
        if let Some(file) = opened {
            // SAFETY: the handle this call opened, closed once.
            unsafe { fclose(file) };
        }
    });
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
