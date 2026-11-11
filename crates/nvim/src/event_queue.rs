//! The editor's multi-level event queue.
//!
//! There is one parent queue and any number of children. An event pushed onto
//! a child is visible from both: from the child in the order that child saw
//! it, and from the parent interleaved with the other children's events in
//! the order the parent saw them. Taking an event from either side removes it
//! from both. Nesting stops at one level — a child may not have children of
//! its own.
//!
//! Every queue lives in one table, [`QUEUES`], and is named by a
//! [`QueueId`]: an index into the table and the generation of the slot it
//! names, so an id that outlived its queue is refused rather than reaching
//! whatever took the slot over. Each queue owns an [`ItemList`]. An event
//! pushed onto a child appends an `Item::Event` to the child and an
//! `Item::Link` to the parent; the event remembers the handle of its link so
//! that taking it from the child can unlink it from the parent in constant
//! time, and the link stands for whichever event is at the head of that
//! child when the parent reaches it. Both lists are FIFO, so the parent's
//! earliest link to a child always names that child's earliest remaining
//! event.
//!
//! The table is borrowed for one step at a time and never across a call out:
//! a handler, or a parent's `on_put`, runs with nothing borrowed, because
//! both can put to a queue again.
//!
//! What an [`Event`] *is* belongs to `event/`: running one may call a
//! C-shaped handler, which is `event/loop.rs`'s [`Event::run`]. The editor's
//! loop owns two queues for its whole life, [`QueueId::MAIN`] (the deferred
//! queue the main state machine drains) and [`QueueId::FAST`] (its child,
//! drained after every poll), made first by [`create_loop_queues`].

#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

pub mod list;

use core::cell::Cell;
use core::num::NonZeroU64;
use std::rc::Rc;

use self::list::{Item, ItemList};
use crate::event::objects::Event;
use crate::global_cell::GlobalCell;

/// A queue in [`QUEUES`]: its slot's index and the generation of the queue
/// that holds it. One word, so an `Option<QueueId>` is too, and an all-zero
/// one is `None`.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub struct QueueId(NonZeroU64);

impl QueueId {
    /// The editor loop's deferred queue.
    pub const MAIN: QueueId = QueueId::new(0, 0);
    /// The editor loop's fast queue, a child of [`QueueId::MAIN`].
    pub const FAST: QueueId = QueueId::new(1, 0);

    const fn new(index: u32, generation: u32) -> Self {
        let bits = (generation as u64) << 32 | (index as u64 + 1);
        match NonZeroU64::new(bits) {
            Some(bits) => QueueId(bits),
            None => panic!("an id is never zero"),
        }
    }

    fn index(self) -> usize {
        let low = self.0.get() & u64::from(u32::MAX);
        usize::try_from(low - 1).expect("a queue index fits a usize")
    }

    fn generation(self) -> u32 {
        u32::try_from(self.0.get() >> 32).expect("the high half is 32 bits")
    }
}

/// One queue's state.
#[derive(Debug)]
struct Queue {
    parent: Option<QueueId>,
    /// Run, with nothing borrowed, after an event is put on one of this
    /// queue's children.
    on_put: Option<fn()>,
    /// Events held, plus -- for a parent -- one per child, as upstream
    /// counted it.
    size: usize,
    items: ItemList,
}

#[derive(Debug)]
struct Slot {
    generation: u32,
    queue: Option<Queue>,
}

/// Every queue, by [`QueueId`].
#[derive(Debug)]
pub struct Queues {
    slots: Vec<Slot>,
    /// Slots holding no queue, for the next one made.
    vacant: Vec<u32>,
}

impl Queues {
    const fn new() -> Self {
        Queues {
            slots: Vec::new(),
            vacant: Vec::new(),
        }
    }

    #[inline]
    fn get(&mut self, id: QueueId) -> &mut Queue {
        let slot = &mut self.slots[id.index()];
        assert!(
            slot.generation == id.generation(),
            "a queue used after it was freed"
        );
        slot.queue
            .as_mut()
            .expect("a queue used after it was freed")
    }

    /// The two slots [`QueueId::MAIN`] and [`QueueId::FAST`] name, held
    /// back from every other queue whichever is made first.
    fn reserve_loop_slots(&mut self) {
        while self.slots.len() < 2 {
            self.slots.push(Slot {
                generation: 0,
                queue: None,
            });
        }
    }

    fn insert(&mut self, queue: Queue) -> QueueId {
        self.reserve_loop_slots();
        match self.vacant.pop() {
            Some(index) => {
                let slot = &mut self.slots[index as usize];
                slot.queue = Some(queue);
                QueueId::new(index, slot.generation)
            }
            None => {
                let index = u32::try_from(self.slots.len()).expect("event queue overflow");
                self.slots.push(Slot {
                    generation: 0,
                    queue: Some(queue),
                });
                QueueId::new(index, 0)
            }
        }
    }

    fn remove(&mut self, id: QueueId) -> Queue {
        let index = id.index();
        let slot = &mut self.slots[index];
        assert!(slot.generation == id.generation(), "a queue freed twice");
        let queue = slot.queue.take().expect("a queue freed twice");
        slot.generation = slot.generation.wrapping_add(1);
        self.vacant
            .push(u32::try_from(index).expect("a queue index fits 32 bits"));
        queue
    }

    /// Append `event` to `id`, linking it into the parent: the parent's
    /// `on_put`, for the caller to run once the table is released.
    fn put(&mut self, id: QueueId, event: Event) -> Option<fn()> {
        let parent = self.get(id).parent;
        let (parent_slot, on_put) = match parent {
            Some(parent) => {
                let parent = self.get(parent);
                (
                    Some(parent.items.push_back(Item::Link { child: id })),
                    parent.on_put,
                )
            }
            None => (None, None),
        };
        let queue = self.get(id);
        queue.items.push_back(Item::Event { event, parent_slot });
        queue.size = queue.size.wrapping_add(1);
        // Only a child's put notifies, and only through its parent's
        // callback -- a parentless queue's own `on_put` is never reached
        // from here.
        on_put
    }

    /// Take the head event, unlinking it from the other side as well.
    fn take(&mut self, id: QueueId) -> Option<Event> {
        let queue = self.get(id);
        let item = queue.items.pop_front()?;
        queue.size = queue.size.wrapping_sub(1);
        let parent = queue.parent;
        Some(match item {
            // Reached from a parent: the link stands for the child's head
            // event. Upstream leaves the child's own counter alone here, so
            // this does too.
            Item::Link { child } => match self.get(child).items.pop_front() {
                Some(Item::Event { event, .. }) => event,
                _ => panic!("a link names an event of a non-empty child"),
            },
            // Reached from the queue the event was pushed to.
            Item::Event { event, parent_slot } => {
                if let Some(slot) = parent_slot {
                    let parent = parent.expect("an event with a parent slot has a parent");
                    self.get(parent).items.remove(slot);
                }
                event
            }
        })
    }
}

/// Every queue the editor has made.
static QUEUES: GlobalCell<Queues> = GlobalCell::new(Queues::new());

/// A queue with no parent; `on_put` runs after every put onto one of its
/// children.
pub fn new_parent(on_put: Option<fn()>) -> QueueId {
    QUEUES.with_mut(|queues| {
        queues.insert(Queue {
            parent: None,
            on_put,
            size: 0,
            items: ItemList::new(),
        })
    })
}

/// A child of `parent`. Nesting stops here: a child may not have children.
pub fn new_child(parent: QueueId) -> QueueId {
    QUEUES.with_mut(|queues| {
        let parent_queue = queues.get(parent);
        assert!(
            parent_queue.parent.is_none(),
            "queues nest only one level deep"
        );
        // Upstream counts a new child in the parent's size.
        parent_queue.size = parent_queue.size.wrapping_add(1);
        queues.insert(Queue {
            parent: Some(parent),
            on_put: None,
            size: 0,
            items: ItemList::new(),
        })
    })
}

/// Release `id` and unlink whatever it still holds from its parent. Its
/// events are dropped unrun, after the table is released: a job's captures
/// may free a queue of their own.
///
/// Freeing a *parent* leaves its children naming a freed queue, as it did
/// upstream; nothing does that while children are alive.
pub fn free(id: QueueId) {
    let dropped = QUEUES.with_mut(|queues| {
        let mut queue = queues.remove(id);
        let mut dropped = Vec::new();
        while let Some(item) = queue.items.pop_front() {
            if let Item::Event { event, parent_slot } = item {
                if let Some(slot) = parent_slot {
                    let parent = queue
                        .parent
                        .expect("an event with a parent slot has a parent");
                    queues.get(parent).items.remove(slot);
                }
                dropped.push(event);
            }
        }
        dropped
    });
    drop(dropped);
}

/// The editor loop's two queues, [`QueueId::MAIN`] and [`QueueId::FAST`],
/// in the two slots held back for them: made once, at startup. `wake` is
/// run after every put onto a child of the main queue.
pub(crate) fn create_loop_queues(wake: fn()) {
    QUEUES.with_mut(|queues| {
        queues.reserve_loop_slots();
        let [main, fast, ..] = &mut queues.slots[..] else {
            unreachable!("the two loop slots are reserved")
        };
        assert!(
            main.queue.is_none() && fast.queue.is_none(),
            "the loop's queues are made once"
        );
        main.queue = Some(Queue {
            parent: None,
            on_put: Some(wake),
            // Upstream counts the child in its parent's size.
            size: 1,
            items: ItemList::new(),
        });
        fast.queue = Some(Queue {
            parent: Some(QueueId::MAIN),
            on_put: None,
            size: 0,
            items: ItemList::new(),
        });
    });
}

/// Release the editor loop's two queues, on the way out. Their slots stay
/// held back.
pub(crate) fn free_loop_queues() {
    for id in [QueueId::FAST, QueueId::MAIN] {
        let dropped = QUEUES.with_mut(|queues| queues.slots[id.index()].queue.take());
        drop(dropped);
    }
}

/// Append `event` to `id`, and to its parent if it has one.
pub fn put(id: QueueId, event: Event) {
    let on_put = QUEUES.with_mut(|queues| queues.put(id, event));
    if let Some(on_put) = on_put {
        on_put();
    }
}

/// The next event on `id`, taken off it (and its parent or child), or
/// `None` when it is empty.
pub fn take(id: QueueId) -> Option<Event> {
    QUEUES.with_mut(|queues| queues.take(id))
}

/// Whether `id` holds no event.
pub fn is_empty(id: QueueId) -> bool {
    QUEUES.with_mut(|queues| queues.get(id).items.is_empty())
}

/// How many events `id` holds -- plus, for a parent, one per child.
pub fn size(id: QueueId) -> usize {
    QUEUES.with_mut(|queues| queues.get(id).size)
}

/// Move every event on `src` to `dest`, in order.
pub fn move_events(dest: QueueId, src: QueueId) {
    while let Some(event) = take(src) {
        put(dest, event);
    }
}

/// Run every queued handler, including any the handlers queue themselves.
pub fn process(id: QueueId) {
    while let Some(event) = take(id) {
        event.run();
    }
}

/// Drop every queued event without running it.
pub fn purge(id: QueueId) {
    while let Some(event) = take(id) {
        drop(event);
    }
}

/// Re-home an empty child under `new_parent`.
pub fn replace_parent(id: QueueId, new_parent: QueueId) {
    QUEUES.with_mut(|queues| {
        let queue = queues.get(id);
        debug_assert!(queue.items.is_empty());
        queue.parent = Some(new_parent);
    });
}

// ---------------------------------------------------------------------------
// Owners
// ---------------------------------------------------------------------------

/// A child of the editor loop's main queue, freed when dropped: what a
/// channel, a timer or a cell outside `event/` holds (`RESIZE_EVENTS`,
/// `DEFERRED_EVENTS`, `CH_BEFORE_BLOCKING_EVENTS`).
#[derive(Debug)]
pub(crate) struct ChildQueue(QueueId);

impl ChildQueue {
    /// A new child of the main loop's queue. The loop's queues are made at
    /// startup, before anything that asks for a child runs.
    pub(crate) fn of_main_loop() -> Self {
        Self(new_child(QueueId::MAIN))
    }

    /// The queue, for a callee that takes one. It lives as long as `self`.
    pub(crate) fn id(&self) -> QueueId {
        self.0
    }
}

impl Drop for ChildQueue {
    fn drop(&mut self) {
        free(self.0);
    }
}

/// A parent queue a frame owns, freed when dropped. Its children must have
/// been handed back to another parent first.
///
/// [`ParkingQueue::on_main_loop`] is the one `jobwait()` parks the jobs'
/// queues on, so that only their events run while it blocks;
/// [`ParkingQueue::detached`] wakes nobody, for a terminal's pending
/// events.
#[derive(Debug)]
pub(crate) struct ParkingQueue(QueueId);

impl ParkingQueue {
    /// A new parent queue whose children's puts wake the editor's loop.
    pub(crate) fn on_main_loop() -> Self {
        Self(new_parent(Some(crate::event::r#loop::wake_main_loop)))
    }

    /// A new parent queue nobody is woken for.
    pub(crate) fn detached() -> Self {
        Self(new_parent(None))
    }

    /// The queue, for a callee that takes one. It lives as long as `self`.
    pub(crate) fn id(&self) -> QueueId {
        self.0
    }
}

impl Drop for ParkingQueue {
    fn drop(&mut self) {
        free(self.0);
    }
}

/// The queue `cell` holds, or `None` before it is made.
pub(crate) fn queue_in(cell: &GlobalCell<Option<ChildQueue>>) -> Option<QueueId> {
    cell.with(|queue| queue.as_ref().map(ChildQueue::id))
}

// ---------------------------------------------------------------------------
// Jobs
// ---------------------------------------------------------------------------

/// Queue `job` on the queue `cell` holds, to run once when that queue is
/// processed. A job the queue never runs is dropped unrun.
///
/// # Panics
/// If the cell holds no queue yet.
pub(crate) fn put_job_in(cell: &GlobalCell<Option<ChildQueue>>, job: impl FnOnce() + 'static) {
    put_event_in(cell, Event::Job(Box::new(job)));
}

/// [`put_job_in`]'s queue step, kept apart from the generic front so the
/// cell is read in one place.
#[inline(never)]
fn put_event_in(cell: &GlobalCell<Option<ChildQueue>>, event: Event) {
    let queue = queue_in(cell).expect("a job is queued after its queue is made");
    put(queue, event);
}

/// [`put_job_in`] the main loop's own queue.
pub(crate) fn put_job_on_main_loop(job: impl FnOnce() + 'static) {
    put(QueueId::MAIN, Event::Job(Box::new(job)));
}

/// [`put_job_in`] the main loop's *fast* queue, which the loop drains while
/// it polls.
pub(crate) fn put_job_on_fast_events(job: impl FnOnce() + 'static) {
    put(QueueId::FAST, Event::Job(Box::new(job)));
}

/// Queue `job` on `queue`, to run once when that queue is processed: the
/// spelling [`put_job_in`] has for a queue the caller holds (a channel's).
pub(crate) fn put_job(queue: &ChildQueue, job: impl FnOnce() + 'static) {
    put(queue.id(), Event::Job(Box::new(job)));
}

/// Queue `job` on `first` and on the queue `second` holds; it runs once, on
/// whichever of the two is processed first, and the other's copy finds
/// nothing to run.
///
/// # Panics
/// If `second` holds no queue yet.
pub(crate) fn put_job_on_either(
    first: &ChildQueue,
    second: &GlobalCell<Option<ChildQueue>>,
    job: impl FnOnce() + 'static,
) {
    let other = queue_in(second).expect("a job is queued after its queue is made");
    let shared: Rc<SharedJob> = Rc::new(Cell::new(Some(Box::new(job))));
    let first_copy = Rc::clone(&shared);
    put(
        first.id(),
        Event::Job(Box::new(move || run_shared(&first_copy))),
    );
    put(other, Event::Job(Box::new(move || run_shared(&shared))));
}

/// A job two queues hold, taken by whichever runs it first.
type SharedJob = Cell<Option<Box<dyn FnOnce()>>>;

/// Run the job [`put_job_on_either`] shared, if the other queue has not.
fn run_shared(shared: &SharedJob) {
    if let Some(job) = shared.take() {
        job();
    }
}

/// Queue `handler(a, b)` on the main loop's *fast* queue, which the loop
/// drains while it polls: [`put_id_event`] for work that must not wait for
/// the editor to be idle.
pub(crate) fn put_id_event_on_fast_events(handler: fn(u64, u64), a: u64, b: u64) {
    put(QueueId::FAST, Event::Ids(handler, a, b));
}

/// Queue `handler(a, b)` on `queue`, to run once when that queue is
/// processed: the allocation-free spelling of a job whose state is two ids
/// (a channel's id and what to do with it). The handler finds its record
/// again by id, so an event that outlives the record runs harmlessly.
pub(crate) fn put_id_event(queue: &ChildQueue, handler: fn(u64, u64), a: u64, b: u64) {
    put(queue.id(), Event::Ids(handler, a, b));
}

/// [`put_id_event`] on the main loop's own queue.
pub(crate) fn put_id_event_on_main_loop(handler: fn(u64, u64), a: u64, b: u64) {
    put(QueueId::MAIN, Event::Ids(handler, a, b));
}

/// [`process`] over the queue `cell` holds, if it holds one: the editor's
/// own child queues, which live as long as their cell.
pub(crate) fn process_events_in(cell: &GlobalCell<Option<ChildQueue>>) {
    if let Some(queue) = queue_in(cell) {
        process(queue);
    }
}
