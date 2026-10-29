//! The editor's multi-level event queue.
//!
//! There is one parent queue and any number of children. An event pushed onto
//! a child is visible from both: from the child in the order that child saw
//! it, and from the parent interleaved with the other children's events in
//! the order the parent saw them. Taking an event from either side removes it
//! from both. Nesting stops at one level — a child may not have children of
//! its own.
//!
//! Each queue owns an [`ItemList`]. An event pushed onto a
//! child appends an `Item::Event` to the child and an `Item::Link` to the
//! parent; the event remembers the handle of its link so that taking it from
//! the child can unlink it from the parent in constant time, and the link
//! stands for whichever event is at the head of that child when the parent
//! reaches it. Both lists are FIFO, so the parent's earliest link to a child
//! always names that child's earliest remaining event.
//!
//! The entry points keep their C shapes (`*mut MultiQueue`, `Event` by
//! value): roughly forty still-transpiled modules call them, and every one of
//! those call sites is already inside an `unsafe fn`. They are the surface to
//! retire once the loop and channel code above them is rewritten. Inside,
//! each of those pointers is wrapped in a [`Queue`] once, so the bodies below
//! are ordinary Rust.

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

use crate::global_cell::GlobalCell;
use crate::startup::main_loop;
use crate::types::multiqueue_list::{Item, ItemList};
use crate::types::{Event, MultiQueue, PutCallback, Refcount, size_t};
use core::ffi::c_void;
use core::ops::{Deref, DerefMut};
use core::ptr;

/// What `multiqueue_get` yields for an empty queue.
const NIL_EVENT: Event = Event {
    handler: None,
    argv: [ptr::null_mut(); 10],
};

/// An event that runs at most once however many queues it was put on, and is
/// released once all `refcount` of them have reached it.
struct MulticastEvent {
    event: Event,
    fired: bool,
    refcount: Refcount,
}

/// A queue, reached through the raw pointer its callers hand around.
///
/// A queue is always addressed by pointer and never moves, so wrapping that
/// pointer once pays the `unsafe` at construction and leaves every field
/// access below as ordinary Rust. Nothing here holds a borrow across a call
/// that can re-enter the queue — draining one runs handlers that push onto it.
#[derive(Copy, Clone)]
struct Queue(*mut MultiQueue);

impl Queue {
    /// # Safety
    /// `queue` is a live queue that outlives every use of this handle.
    unsafe fn new(queue: *mut MultiQueue) -> Self {
        debug_assert!(!queue.is_null());
        Queue(queue)
    }

    /// The pointer back, for the callers that still pass one around.
    fn as_ptr(self) -> *mut MultiQueue {
        self.0
    }

    /// The items this queue owns. Every queue is created with a list and
    /// keeps it until [`multiqueue_free`] takes both away together.
    fn list<'a>(self) -> &'a mut ItemList {
        // SAFETY: the list is a `Box` of its own, and its pointer is read out
        // of the wrapped pointer rather than out of a borrow of the queue.
        unsafe { &mut *(*self.0).items }
    }

    /// The queue this one is a child of, if it has one.
    fn parent(self) -> Option<Queue> {
        // A parent outlives its children, so its pointer inherits this
        // handle's promise.
        (!self.parent.is_null()).then(|| Queue(self.parent))
    }
}

impl Deref for Queue {
    type Target = MultiQueue;

    fn deref(&self) -> &MultiQueue {
        // SAFETY: the promise made at construction.
        unsafe { &*self.0 }
    }
}

impl DerefMut for Queue {
    fn deref_mut(&mut self) -> &mut MultiQueue {
        // SAFETY: the promise made at construction.
        unsafe { &mut *self.0 }
    }
}

/// A queue with no parent, notifying `on_put` — with `data` — on every put
/// onto one of its children.
///
/// # Safety
/// `on_put`, if given, is safe to call with `data` for as long as the queue
/// lives.
pub unsafe fn multiqueue_new(on_put: PutCallback, data: *mut c_void) -> *mut MultiQueue {
    new_queue(ptr::null_mut(), on_put, data)
}

/// A child of `parent`. Nesting stops here: a child may not have children.
///
/// # Safety
/// `parent` is live and outlives the child.
pub unsafe fn multiqueue_new_child(parent: *mut MultiQueue) -> *mut MultiQueue {
    // SAFETY: the caller's live queue.
    let mut parent = unsafe { Queue::new(parent) };
    assert!(parent.parent().is_none(), "queues nest only one level deep");
    // Upstream counts a new child in the parent's size. See the field's note.
    parent.size = parent.size.wrapping_add(1);
    new_queue(parent.as_ptr(), None, ptr::null_mut())
}

/// A child of the main loop's event queue.
///
/// Safe for the reason the editor's other event-queue owners are: the main
/// loop and its `events` queue are set up before anything that asks for a
/// child runs, and live until exit.
pub(crate) fn main_loop_child_queue() -> *mut MultiQueue {
    // SAFETY: the main loop's queue, live from startup to exit.
    unsafe { multiqueue_new_child((*main_loop.ptr()).events) }
}

/// A child of the main loop's queue that the editor keeps for its whole run
/// (`resize_events`, `deferred_events`, `ch_before_blocking_events`): the
/// owner a cell outside `event/` holds instead of the queue's address.
/// Dropping it frees the queue.
pub(crate) struct ChildQueue(ptr::NonNull<MultiQueue>);

impl ChildQueue {
    /// A new child of the main loop's queue; see [`main_loop_child_queue`].
    pub(crate) fn of_main_loop() -> Self {
        Self(ptr::NonNull::new(main_loop_child_queue()).expect("a new queue is never null"))
    }

    /// The queue, for a callee that takes one. It lives as long as `self`.
    pub(crate) fn as_ptr(&self) -> *mut MultiQueue {
        self.0.as_ptr()
    }
}

impl Drop for ChildQueue {
    fn drop(&mut self) {
        // SAFETY: the queue this value made and owns, freed once.
        unsafe { multiqueue_free(self.0.as_ptr()) };
    }
}

/// A parent queue of a waiting frame's own, that the loop is woken for when
/// a child of it is put to: `jobwait()` parks the jobs' queues here so that
/// only their events run while it blocks. Dropping it frees the queue; the
/// children must have been handed back to another parent first.
pub(crate) struct ParkingQueue(ptr::NonNull<MultiQueue>);

impl ParkingQueue {
    /// A new parent queue whose puts wake the main loop.
    pub(crate) fn on_main_loop() -> Self {
        let main = crate::event::r#loop::main_loop_raw();
        // SAFETY: `loop_on_put` is safe to call with the main loop, which
        // outlives every queue made here.
        let queue = unsafe { multiqueue_new(Some(crate::event::r#loop::loop_on_put), main.cast()) };
        Self(ptr::NonNull::new(queue).expect("a new queue is never null"))
    }

    /// The queue, for a callee that takes one. It lives as long as `self`.
    pub(crate) fn as_ptr(&self) -> *mut MultiQueue {
        self.0.as_ptr()
    }
}

impl Drop for ParkingQueue {
    fn drop(&mut self) {
        // SAFETY: the queue this value made and owns, freed once.
        unsafe { multiqueue_free(self.0.as_ptr()) };
    }
}

/// The queue `cell` holds, or null before it is made.
pub(crate) fn queue_in(cell: &GlobalCell<Option<ChildQueue>>) -> *mut MultiQueue {
    cell.with(|queue| queue.as_ref().map_or(ptr::null_mut(), ChildQueue::as_ptr))
}

fn new_queue(parent: *mut MultiQueue, on_put: PutCallback, data: *mut c_void) -> *mut MultiQueue {
    Box::into_raw(Box::new(MultiQueue {
        parent,
        on_put,
        data,
        size: 0,
        items: Box::into_raw(Box::new(ItemList::new())),
    }))
}

/// Release `queue` and unlink whatever it still holds from its parent.
///
/// Freeing a *parent* leaves its children pointing at released memory, as it
/// did upstream; nothing does that while children are alive.
///
/// # Safety
/// `queue` is live, was made by [`multiqueue_new`] or
/// [`multiqueue_new_child`], and is not used again.
pub unsafe fn multiqueue_free(queue: *mut MultiQueue) {
    debug_assert!(!queue.is_null());
    // SAFETY: the caller hands the queue over; `new_queue` boxed it and its
    // list, and this is the only place either is taken back.
    let owned = unsafe { Box::from_raw(queue) };
    // SAFETY: the list box, still reachable through the queue being dropped.
    let mut own_items = unsafe { Box::from_raw(owned.items) };
    // A slot in the parent's list is itself the proof that the parent is
    // live, so the handle is only built when there is one to unlink from.
    // SAFETY: a parent outlives its children.
    let parent = (!owned.parent.is_null()).then(|| unsafe { Queue::new(owned.parent) });
    while let Some(item) = own_items.pop_front() {
        if let Item::Event {
            parent_slot: Some(slot),
            ..
        } = item
        {
            parent
                .expect("an event with a parent slot has a parent")
                .list()
                .remove(slot);
        }
    }
}

/// The next event, or a handler-less event when there is none.
///
/// # Safety
/// `queue` is live.
pub unsafe fn multiqueue_get(queue: *mut MultiQueue) -> Event {
    // SAFETY: the caller's live queue.
    let queue = unsafe { Queue::new(queue) };
    take_event(queue).unwrap_or(NIL_EVENT)
}

/// Append `event` to `queue`, and to its parent if it has one.
///
/// # Safety
/// `queue` is live, and `event`'s handler is safe to call with its argv once
/// the queue reaches it.
pub unsafe fn multiqueue_put_event(queue: *mut MultiQueue, event: Event) {
    debug_assert!(!queue.is_null());
    // SAFETY: the caller's live queue.
    put_event(unsafe { Queue::new(queue) }, event);
}

/// Queue `job` on the queue `cell` holds, to run once when that queue is
/// processed: the safe spelling of an [`Event`] whose one argument is the
/// work to do. The job owns what it needs; one the queue never runs is
/// dropped unrun with the process, as an unrun event's arguments were.
///
/// # Panics
/// If the cell holds no queue yet.
pub(crate) fn put_job_in(cell: &GlobalCell<Option<ChildQueue>>, job: Box<dyn FnOnce()>) {
    let queue = queue_in(cell);
    assert!(!queue.is_null(), "a job is queued after its queue is made");
    // SAFETY: the cell's own live queue; `run_job` takes back the job packed
    // here, once.
    unsafe { multiqueue_put_event(queue, job_event(job)) };
}

/// [`put_job_in`] the main loop's own queue.
pub(crate) fn put_job_on_main_loop(job: Box<dyn FnOnce()>) {
    // SAFETY: the main loop's queue, live from startup to exit; `run_job`
    // takes back the job packed here, once.
    unsafe {
        multiqueue_put_event(
            (*crate::event::r#loop::main_loop_raw()).events,
            job_event(job),
        )
    };
}

/// [`put_job_in`] the main loop's *fast* queue, which the loop drains while
/// it polls.
pub(crate) fn put_job_on_fast_events(job: Box<dyn FnOnce()>) {
    // SAFETY: the main loop's fast queue, live from startup to exit;
    // `run_job` takes back the job packed here, once.
    unsafe {
        multiqueue_put_event(
            (*crate::event::r#loop::main_loop_raw()).fast_events,
            job_event(job),
        )
    };
}

/// Queue `job` on `queue`, to run once when that queue is processed: the
/// spelling [`put_job_in`] has for a queue the caller holds (a channel's).
pub(crate) fn put_job(queue: &ChildQueue, job: Box<dyn FnOnce()>) {
    // SAFETY: the caller's live queue; `run_job` takes back the job packed
    // here, once.
    unsafe { multiqueue_put_event(queue.as_ptr(), job_event(job)) };
}

/// Queue `job` on `first` and on the queue `second` holds; it runs once, on
/// whichever of the two is processed first.
///
/// # Panics
/// If `second` holds no queue yet.
pub(crate) fn put_job_on_either(
    first: &ChildQueue,
    second: &GlobalCell<Option<ChildQueue>>,
    job: Box<dyn FnOnce()>,
) {
    let other = queue_in(second);
    assert!(!other.is_null(), "a job is queued after its queue is made");
    let event = event_create_oneshot(job_event(job), 2);
    // SAFETY: both live queues; the one-shot wrapper runs the job once and
    // is released once both queues have reached it.
    unsafe {
        multiqueue_put_event(first.as_ptr(), event.clone());
        multiqueue_put_event(other, event);
    }
}

/// Queue `handler(a, b)` on the main loop's *fast* queue, which the loop
/// drains while it polls: [`put_id_event`] for work that must not wait for
/// the editor to be idle.
pub(crate) fn put_id_event_on_fast_events(handler: fn(u64, u64), a: u64, b: u64) {
    // SAFETY: the main loop's fast queue, live from startup to exit.
    unsafe {
        multiqueue_put_event(
            (*crate::event::r#loop::main_loop_raw()).fast_events,
            id_event(handler, a, b),
        )
    };
}

/// Queue `handler(a, b)` on `queue`, to run once when that queue is
/// processed: the allocation-free spelling of a job whose state is two ids
/// (a channel's id and what to do with it). The handler finds its record
/// again by id, so an event that outlives the record runs harmlessly.
pub(crate) fn put_id_event(queue: &ChildQueue, handler: fn(u64, u64), a: u64, b: u64) {
    // SAFETY: the child's own live queue; `run_id_event` takes back what
    // `id_event` packed.
    unsafe { multiqueue_put_event(queue.as_ptr(), id_event(handler, a, b)) };
}

/// [`put_id_event`] on the main loop's own queue.
pub(crate) fn put_id_event_on_main_loop(handler: fn(u64, u64), a: u64, b: u64) {
    // SAFETY: the main loop's queue, live from startup to exit.
    unsafe {
        multiqueue_put_event(
            (*crate::event::r#loop::main_loop_raw()).events,
            id_event(handler, a, b),
        )
    };
}

/// The event that runs `handler(a, b)`: the handler's address and the two
/// ids, each in a word of its own.
fn id_event(handler: fn(u64, u64), a: u64, b: u64) -> Event {
    let word = |n: u64| ptr::without_provenance_mut(usize::try_from(n).expect("an id fits a word"));
    Event::new(
        Some(run_id_event),
        [handler as *mut c_void, word(a), word(b)],
    )
}

/// A queue reached an [`id_event`]: run its handler.
///
/// # Safety
/// `argv` holds the handler and the two ids [`id_event`] packed.
unsafe extern "C" fn run_id_event(argv: *mut *mut c_void) {
    // SAFETY: the caller's promise: slot 0 is a `fn(u64, u64)` cast to a
    // word, slots 1 and 2 its two ids.
    let (handler, a, b) = unsafe {
        let handler: fn(u64, u64) = core::mem::transmute::<*mut c_void, fn(u64, u64)>(*argv);
        (
            handler,
            (*argv.add(1)).addr() as u64,
            (*argv.add(2)).addr() as u64,
        )
    };
    handler(a, b);
}

/// The event that runs `job`.
fn job_event(job: Box<dyn FnOnce()>) -> Event {
    let job = Box::into_raw(Box::new(job)).cast::<c_void>();
    Event::new(Some(run_job), [job])
}

/// A queue reached a [`put_job_in`] event: run it.
///
/// # Safety
/// `argv[0]` is the boxed job [`job_event`] packed, taken back once here.
unsafe extern "C" fn run_job(argv: *mut *mut c_void) {
    // SAFETY: the caller's promise.
    let job = unsafe { Box::from_raw((*argv).cast::<Box<dyn FnOnce()>>()) };
    job();
}

/// Append `event`, linking it into the parent and notifying that parent's
/// `on_put` if there is one.
fn put_event(mut queue: Queue, event: Event) {
    let parent = queue.parent();
    let parent_slot = parent.map(|parent| {
        parent.list().push_back(Item::Link {
            child: queue.as_ptr(),
        })
    });
    queue.list().push_back(Item::Event { event, parent_slot });
    queue.size = queue.size.wrapping_add(1);
    // Only a child's put notifies, and only through its parent's callback —
    // a parentless queue's own `on_put` is never reached from here.
    if let Some(parent) = parent
        && let Some(on_put) = parent.on_put
    {
        // SAFETY: the callback and the data were installed together by
        // `multiqueue_new`, and the queue is the one it was installed on.
        unsafe { on_put(parent.as_ptr(), parent.data) };
    }
}

/// How many events `queue` holds — plus, for a parent, one per child.
///
/// # Safety
/// `queue` is live.
pub unsafe fn multiqueue_size(queue: *mut MultiQueue) -> size_t {
    // SAFETY: the caller's live queue.
    unsafe { Queue::new(queue) }.size
}

/// Take the head event, unlinking it from the other side as well. `None` when
/// the queue is empty.
fn take_event(mut queue: Queue) -> Option<Event> {
    let event = match queue.list().pop_front()? {
        // Reached from a parent: the link stands for the child's head event.
        // Upstream leaves the child's own counter alone here, so this does
        // too.
        Item::Link { child } => {
            // SAFETY: a link is unlinked when its child is freed, so a link
            // that is still here names a live child.
            let child = unsafe { Queue::new(child) };
            match child.list().pop_front() {
                Some(Item::Event { event, .. }) => event,
                _ => panic!("a link names an event of a non-empty child"),
            }
        }
        // Reached from the queue the event was pushed to.
        Item::Event { event, parent_slot } => {
            if let Some(slot) = parent_slot {
                queue
                    .parent()
                    .expect("an event with a parent slot has a parent")
                    .list()
                    .remove(slot);
            }
            event
        }
    };
    queue.size = queue.size.wrapping_sub(1);
    Some(event)
}

/// # Safety
/// `queue` is live.
pub unsafe fn multiqueue_empty(queue: *mut MultiQueue) -> bool {
    debug_assert!(!queue.is_null());
    // SAFETY: the caller's live queue.
    unsafe { Queue::new(queue) }.list().is_empty()
}

/// Move every event on `src` to `dest`, in order.
///
/// # Safety
/// Both queues are live.
pub unsafe fn multiqueue_move_events(dest: *mut MultiQueue, src: *mut MultiQueue) {
    // SAFETY: the caller's live queues.
    let (dest, src) = unsafe { (Queue::new(dest), Queue::new(src)) };
    while let Some(event) = take_event(src) {
        put_event(dest, event);
    }
}

/// [`multiqueue_process_events`] over the queue `cell` holds, if it holds
/// one: the editor's own child queues, which live as long as their cell.
pub(crate) fn process_events_in(cell: &GlobalCell<Option<ChildQueue>>) {
    let queue = queue_in(cell);
    if !queue.is_null() {
        // SAFETY: the queue the cell owns; its handlers are the editor's.
        unsafe { multiqueue_process_events(queue) };
    }
}

/// Run every queued handler, including any the handlers queue themselves.
///
/// # Safety
/// `queue` is live, and so is everything its events' handlers reach.
pub unsafe fn multiqueue_process_events(queue: *mut MultiQueue) {
    debug_assert!(!queue.is_null());
    // SAFETY: the caller's live queue.
    let queue = unsafe { Queue::new(queue) };
    while let Some(mut event) = take_event(queue) {
        if let Some(handler) = event.handler {
            // SAFETY: an event carries the argv its handler was queued with;
            // the array is this frame's copy, so the handler cannot reach it
            // again by re-entering the queue.
            unsafe { handler(event.argv.as_mut_ptr()) };
        }
    }
}

/// Drop every queued event without running it.
///
/// # Safety
/// `queue` is live.
pub unsafe fn multiqueue_purge_events(queue: *mut MultiQueue) {
    debug_assert!(!queue.is_null());
    // SAFETY: the caller's live queue.
    let queue = unsafe { Queue::new(queue) };
    while take_event(queue).is_some() {}
}

/// Re-home an empty child under `new_parent`.
///
/// # Safety
/// Both queues are live.
pub unsafe fn multiqueue_replace_parent(queue: *mut MultiQueue, new_parent: *mut MultiQueue) {
    // SAFETY: the caller's live queue.
    let mut queue = unsafe { Queue::new(queue) };
    debug_assert!(queue.list().is_empty());
    queue.parent = new_parent;
}

/// An event that fires the first time it is reached and is released once
/// `num` queues have reached it.
pub fn event_create_oneshot(event: Event, num: ::core::ffi::c_int) -> Event {
    let data = Box::into_raw(Box::new(MulticastEvent {
        event,
        fired: false,
        refcount: Refcount::new(num),
    }));
    Event::new(Some(multiqueue_oneshot_event), [data.cast::<c_void>()])
}

/// The handler [`event_create_oneshot`] wraps the caller's event in.
///
/// # Safety
/// Slot 0 of `argv` is the `MulticastEvent` that function boxed, and this
/// call is one of the `refcount` it was created with.
unsafe extern "C" fn multiqueue_oneshot_event(argv: *mut *mut c_void) {
    // SAFETY: `event_create_oneshot` boxed this and left it in slot 0, and
    // the box outlives every queue that still holds one of its references.
    let data = unsafe { (*argv).cast::<MulticastEvent>() };
    // The flag is set before the wrapped handler runs, and no borrow is held
    // across it: draining one queue can re-enter another and reach this same
    // event.
    // SAFETY: as above.
    let first = !unsafe { core::mem::replace(&mut (*data).fired, true) };
    // SAFETY: as above; the argv is the wrapped event's own.
    if first && let Some(handler) = unsafe { (*data).event.handler } {
        unsafe { handler((&raw mut (*data).event.argv).cast()) };
    }
    // SAFETY: this queue's reference, and it is this call that gives it up.
    let last = unsafe { (*data).refcount.release() == 0 };
    if last {
        // SAFETY: no queue holds a reference any more.
        drop(unsafe { Box::from_raw(data) });
    }
}
