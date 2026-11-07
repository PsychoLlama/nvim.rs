//! The editor's event loop: a libuv loop plus the multi-level event queue.
//!
//! The loop drains two queues of [`crate::event_queue`]'s: [`QueueId::MAIN`],
//! the deferred queue the editor drains from its main state machine, and
//! [`QueueId::FAST`], its child, drained as soon as `uv_run` returns. Other
//! threads post jobs to an inbox behind a mutex, which the async handle
//! moves onto the fast queue.
//!
//! Every subsystem in `event/` reaches the loop through a raw pointer libuv
//! also holds — in the loop's own `data` field, and in each handle's. That is
//! what [`EventLoop`] is for: the pointer is wrapped once, paying the `unsafe`
//! at construction, and the field accesses below are ordinary Rust. Nothing
//! holds a borrow across `uv_run`, which re-enters this module.

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

use crate::os::signal::SIGCHLD;
use crate::os::uv_error::UV_EBUSY;
use core::ffi::{c_int, c_void};
use core::ops::{Deref, DerefMut};
use core::ptr;

use crate::event::libuv::{
    uv_async_init, uv_async_send, uv_close, uv_is_closing, uv_loop_close, uv_loop_init,
    uv_print_all_handles, uv_run, uv_signal_init, uv_signal_start, uv_signal_stop, uv_stop,
    uv_timer_init, uv_timer_start, uv_timer_stop, uv_walk,
};
use crate::event::objects::Loop;
use crate::event::objects::{Event, Proc};
use crate::event::uv::{uv_async_t, uv_handle_t, uv_loop_t, uv_run_mode, uv_signal_t, uv_timer_t};
use crate::event_queue::{self, QueueId};
use crate::log::{LOGLVL_ERR, log_file_path, logmsg, with_log_lock};
use crate::os::cshim::stderr;
use crate::os::time::os_hrtime;
use ::libc::{abort, fclose, fopen};
use std::collections::VecDeque;
use std::sync::Mutex;
use std::sync::atomic::{AtomicPtr, Ordering};

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
    /// [`main_loop_close`] (the only loop that is ever closed), that outlives
    /// every use of this handle.
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
    event_queue::create_loop_queues(wake_main_loop);
    // SAFETY: every handle below is a field of the loop just initialised,
    // and each is registered with that same loop.
    unsafe {
        uv_async_init(uv_loop.uv(), uv_loop.wakeup(), Some(async_cb));
        uv_signal_init(uv_loop.uv(), uv_loop.children_watcher());
        uv_timer_init(uv_loop.uv(), uv_loop.kill_timer());
        uv_timer_init(uv_loop.uv(), uv_loop.poll_timer());
        uv_timer_init(uv_loop.uv(), uv_loop.exit_delay_timer());
    }
    // The poll timer's only job is to flip this flag; it outlives every
    // `loop_uv_run` and is released by the handle's close callback.
    uv_loop.poll_timer.data = Box::into_raw(Box::new(false)).cast();
    // Other threads wake the loop through this handle from now on.
    WAKEUP.store(uv_loop.wakeup(), Ordering::Release);
}

/// The editor's own loop, as the functions here take it. It is a static, so
/// it lives for the process and never moves.
pub(crate) fn main_loop_raw() -> *mut Loop {
    crate::startup::main_loop.ptr()
}

/// Run the next event on the editor's main queue, if it holds one, and
/// answer whether the queue is empty afterwards: one step of the main
/// loop's `K_EVENT` drain, which checks for input between steps.
pub(crate) fn run_next_main_event() -> bool {
    if let Some(event) = event_queue::take(QueueId::MAIN) {
        event.run();
    }
    event_queue::is_empty(QueueId::MAIN)
}

/// How many jobs other threads have posted to the editor's loop and it has
/// not moved onto its fast queue yet.
pub(crate) fn main_loop_size() -> usize {
    inbox().len()
}

/// Drop everything posted to the editor's loop but not yet processed: the
/// other threads' jobs and the fast queue.
pub(crate) fn main_loop_purge() {
    inbox().clear();
    event_queue::purge(QueueId::FAST);
}

/// [`loop_init`] of the editor's own loop. Once, at startup.
pub(crate) fn main_loop_init() {
    // SAFETY: the static loop, which does not move and is initialised here
    // once, before anything else uses it.
    unsafe { loop_init(main_loop_raw()) };
}

/// Whether [`main_loop_init`] has run: the loop's queues exist.
pub(crate) fn main_loop_is_up() -> bool {
    !WAKEUP.load(Ordering::Acquire).is_null()
}

/// Whether the editor's loop is running a callback of its own right now —
/// code reached from there must not re-enter it.
pub(crate) fn main_loop_is_recursing() -> bool {
    // SAFETY: the static loop; reading one field of it.
    unsafe { (*main_loop_raw()).recursive != 0 }
}

/// Whether the editor's loop is being taken down: nothing new may be
/// scheduled onto it.
pub(crate) fn main_loop_is_closing() -> bool {
    // SAFETY: the static loop; reading one field of it.
    unsafe { (*main_loop_raw()).closing }
}

/// [`loop_poll_events`] on the editor's own loop: run libuv once (for up to
/// `ms` milliseconds), then drain the fast queue. Whether `ms` ran out.
pub(crate) fn poll_main_loop(ms: i64) -> bool {
    // SAFETY: the static loop, up from startup to exit.
    unsafe { loop_poll_events(main_loop_raw(), ms) }
}

/// Run what is already queued on the editor's loop, then poll it once
/// without waiting -- the drain before the loop is taken down.
pub(crate) fn main_loop_drain() {
    event_queue::process(QueueId::MAIN);
    // SAFETY: the static loop, up (the caller checked [`main_loop_is_up`]).
    unsafe { loop_poll_events(main_loop_raw(), 0) };
}

/// Close every handle of the editor's loop and release its queues. `wait`
/// gives libuv up to two seconds to finish; returns false if it did not.
///
/// Once, on the way out: nothing may use the loop afterwards.
pub(crate) fn main_loop_close(wait: bool) -> bool {
    // SAFETY: the static loop, up and no longer in use.
    let mut uv_loop = unsafe { EventLoop::new(main_loop_raw()) };
    uv_loop.closing = true;
    WAKEUP.store(ptr::null_mut(), Ordering::Release);
    // SAFETY: the loop's own handles, each still open.
    unsafe {
        uv_close(uv_loop.children_watcher().cast(), None);
        uv_close(uv_loop.kill_timer().cast(), None);
        uv_close(uv_loop.poll_timer().cast(), Some(timer_close_cb));
        uv_close(uv_loop.exit_delay_timer().cast(), None);
        uv_close(uv_loop.wakeup().cast(), None);
    }

    let rv = drain_until_closed(uv_loop, wait);

    inbox().clear();
    event_queue::free_loop_queues();
    // SAFETY: the children list this loop boxed. Nothing reaches it after
    // the loop is closed.
    drop(unsafe { Box::from_raw(uv_loop.children) });
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

/// Arm the editor loop's `SIGCHLD` watcher with `on_child`: what reaps the
/// children libuv did not spawn itself (the pty ones). Starting it again
/// replaces the callback.
pub(crate) fn start_children_watcher(on_child: unsafe extern "C" fn(*mut uv_signal_t, c_int)) {
    // SAFETY: the static loop's own watcher, initialised by `loop_init`.
    unsafe {
        uv_signal_start(
            &raw mut (*main_loop_raw()).children_watcher,
            Some(on_child),
            SIGCHLD,
        )
    };
}

/// Disarm [`start_children_watcher`], when the loop shuts down.
pub(crate) fn stop_children_watcher() {
    // SAFETY: the static loop's own watcher.
    unsafe { uv_signal_stop(&raw mut (*main_loop_raw()).children_watcher) };
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
    event_queue::process(QueueId::FAST);
    expired
}

/// One pass of upstream's `LOOP_PROCESS_EVENTS`: drain `queue` if it has
/// anything, otherwise let the loop poll for `timeout` milliseconds.
///
/// # Safety
/// `uv_loop` is live, and so is everything its events reach.
pub unsafe fn process_events(uv_loop: *mut Loop, queue: Option<QueueId>, timeout: i64) {
    match queue {
        Some(queue) if !event_queue::is_empty(queue) => event_queue::process(queue),
        _ => {
            // SAFETY: the caller's loop.
            unsafe { loop_poll_events(uv_loop, timeout) };
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
    queue: Option<QueueId>,
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

/// What a wait on the editor's own loop drains between polls.
pub(crate) enum Drain {
    /// The main queue: everything the editor has deferred.
    Events,
    /// Only the fast queue, which the poll itself drains.
    FastOnly,
    /// Nothing but what the poll itself runs.
    Nothing,
}

/// [`process_events_until`] on the editor's own loop: the safe spelling for
/// a frame that waits on its own condition, `done`, which may read anything
/// the events it lets run have changed.
pub(crate) fn wait_on_main_loop(drain: Drain, ms: i64, done: impl FnMut() -> bool) {
    let uv_loop = main_loop_raw();
    let queue = drain.queue();
    // SAFETY: the editor's loop and its queues, live from startup to exit,
    // or a queue the caller holds for the length of the wait.
    unsafe { process_events_until(uv_loop, queue, ms, done) };
}

impl Drain {
    /// The queue this drains, which lives at least as long as `self`.
    pub(crate) fn queue(&self) -> Option<QueueId> {
        match self {
            Drain::Events => Some(QueueId::MAIN),
            Drain::FastOnly => Some(QueueId::FAST),
            Drain::Nothing => None,
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
// Events
// ---------------------------------------------------------------------------

/// A C-shaped event: a handler and the one argument word it is run with --
/// upstream's `Event`, which the stream, process, timer and signal watchers
/// here still post with their own address as the argument.
pub struct RawEvent {
    handler: unsafe extern "C" fn(*mut *mut c_void),
    arg: *mut c_void,
}

impl RawEvent {
    /// The event that runs `handler` with an argv whose one word is `arg`.
    ///
    /// # Safety
    /// `handler` is safe to call, once, with that argv, whenever the queue
    /// the event is put on reaches it: whatever `arg` names outlives the
    /// event, or is released by the handler.
    pub unsafe fn event(
        handler: unsafe extern "C" fn(*mut *mut c_void),
        arg: *mut c_void,
    ) -> Event {
        Event::Raw(RawEvent { handler, arg })
    }
}

impl Event {
    /// Run the event, once.
    pub fn run(self) {
        match self {
            Event::Raw(raw) => {
                let mut argv = [raw.arg];
                // SAFETY: `RawEvent::event`'s contract; the argv is this
                // frame's own.
                unsafe { (raw.handler)(argv.as_mut_ptr()) };
            }
            Event::Job(job) => job(),
            Event::Ids(handler, a, b) => handler(a, b),
        }
    }
}

/// What another thread posts: work it hands over whole.
type ThreadJob = Box<dyn FnOnce() + Send>;

/// Which of the editor's queues a [`ThreadJob`] is for.
enum Lane {
    Fast,
    Deferred,
}

/// The jobs other threads have posted and the loop has not moved onto its
/// fast queue yet. The one piece of the loop other threads touch.
static INBOX: Mutex<VecDeque<(Lane, ThreadJob)>> = Mutex::new(VecDeque::new());

/// The handle another thread wakes the editor's loop with: null before
/// [`loop_init`] and after [`main_loop_close`].
static WAKEUP: AtomicPtr<uv_async_t> = AtomicPtr::new(ptr::null_mut());

/// The inbox, locked. A panic while it was held cannot leave it half-made:
/// every critical section is one push, one drain or one clear.
fn inbox() -> std::sync::MutexGuard<'static, VecDeque<(Lane, ThreadJob)>> {
    INBOX
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}

/// Post `job` from any thread, and wake the loop to take it.
fn post(lane: Lane, job: ThreadJob) {
    inbox().push_back((lane, job));
    let wakeup = WAKEUP.load(Ordering::Acquire);
    if !wakeup.is_null() {
        // SAFETY: the loop's async handle, open from `loop_init` until
        // `main_loop_close` clears the pointer; `uv_async_send` is the one
        // libuv call that may be made from any thread.
        unsafe { uv_async_send(wakeup) };
    }
}

/// Run `job` once, soon, from the editor loop's fast queue: what a thread
/// that is not the main one -- or a callback that must not run the editor
/// inline -- posts.
pub(crate) fn schedule_fast_job(job: ThreadJob) {
    post(Lane::Fast, job);
}

/// Run `job` once, later, from the main loop's deferred queue.
///
/// For a caller that cannot do the work where it is -- an error raised
/// where it cannot be shown, say, or another thread. A job the loop never
/// runs is dropped unrun with the process.
pub(crate) fn schedule_deferred_job(job: ThreadJob) {
    post(Lane::Deferred, job);
}

/// Break out of `uv_run` when an event lands on a child of the main queue
/// while the loop is running, so the caller gets a chance to process it.
pub(crate) fn wake_main_loop() {
    // SAFETY: the static loop, which lives for the process; reading one
    // field of it.
    let recursive = unsafe { (*main_loop_raw()).recursive };
    if recursive != 0 {
        // SAFETY: the loop is live and this thread is inside its `uv_run`.
        unsafe { uv_stop(&raw mut (*main_loop_raw()).uv) };
    }
}

/// Another thread posted to the inbox: move the lot to the fast queue. A
/// deferred job travels through the fast queue to get to the main one --
/// only the main thread may touch that -- as upstream's did.
///
/// # Safety
/// libuv's async callback, on the handle [`loop_init`] registered.
unsafe extern "C" fn async_cb(_handle: *mut uv_async_t) {
    let posted: Vec<_> = inbox().drain(..).collect();
    for (lane, job) in posted {
        let event = match lane {
            Lane::Fast => Event::Job(job),
            Lane::Deferred => Event::Job(Box::new(move || {
                event_queue::put(QueueId::MAIN, Event::Job(job));
            })),
        };
        event_queue::put(QueueId::FAST, event);
    }
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
