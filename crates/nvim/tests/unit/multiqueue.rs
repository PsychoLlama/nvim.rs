//! The multi-level queue's ordering semantics, mirroring
//! `test/unit/multiqueue_spec.lua`. That spec pushes an event whose first
//! `argv` slot is a label and reads the label back; here the label is the
//! first id of an id event, and the whole thing runs under Miri.

use std::sync::atomic::{AtomicUsize, Ordering};
#[cfg(miri)]
use std::sync::{Mutex, MutexGuard};

use neovim::event::objects::Event;
use neovim::event_queue::{
    QueueId, free, is_empty, move_events, new_child, new_parent, process, purge, put,
    replace_parent, size, take,
};

/// The queue table is the editor's, process-wide, and `cargo test` runs
/// cases in parallel: every case here holds this for its whole length.
#[cfg(not(miri))]
fn exclusive() -> impl Sized {
    crate::support::editor_lock()
}

/// Under Miri nothing brings the editor up, and only these cases reach the
/// queue table.
#[cfg(miri)]
fn exclusive() -> MutexGuard<'static, ()> {
    static QUEUES: Mutex<()> = Mutex::new(());
    QUEUES.lock().unwrap_or_else(|e| e.into_inner())
}

/// An id event that does nothing when run, labelled `label`.
fn labelled(label: u64) -> Event {
    fn ignore(_: u64, _: u64) {}
    Event::Ids(ignore, label, 0)
}

/// The labels a fixture has put, by the number its events carry.
struct Labels(Vec<&'static str>);

impl Labels {
    fn new() -> Self {
        Labels(Vec::new())
    }

    fn put(&mut self, queue: QueueId, label: &'static str) {
        self.0.push(label);
        put(queue, labelled(self.0.len() as u64 - 1));
    }

    /// The label of the next event on `queue`.
    fn get(&self, queue: QueueId) -> &'static str {
        match take(queue) {
            Some(Event::Ids(_, label, _)) => self.0[label as usize],
            Some(_) => panic!("not a labelled event"),
            None => panic!("queue was empty"),
        }
    }
}

/// The parent/three-children fixture the Lua spec builds in `before_each`.
struct Fixture {
    _exclusive: Box<dyn std::any::Any>,
    labels: Labels,
    parent: QueueId,
    child1: QueueId,
    child2: Option<QueueId>,
    child3: QueueId,
}

impl Fixture {
    fn new() -> Self {
        let exclusive: Box<dyn std::any::Any> = Box::new(exclusive());
        let mut labels = Labels::new();
        let parent = new_parent(None);
        let child1 = new_child(parent);
        let child2 = new_child(parent);
        let child3 = new_child(parent);
        for (queue, label) in [
            (child1, "c1i1"),
            (child1, "c1i2"),
            (child2, "c2i1"),
            (child1, "c1i3"),
            (child2, "c2i2"),
            (child2, "c2i3"),
            (child2, "c2i4"),
            (child3, "c3i1"),
            (child3, "c3i2"),
        ] {
            labels.put(queue, label);
        }
        Fixture {
            _exclusive: exclusive,
            labels,
            parent,
            child1,
            child2: Some(child2),
            child3,
        }
    }

    fn child2(&self) -> QueueId {
        self.child2.expect("child 2 is not freed")
    }

    fn get(&self, queue: QueueId) -> &'static str {
        self.labels.get(queue)
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        for queue in [Some(self.child1), self.child2, Some(self.child3)]
            .into_iter()
            .flatten()
        {
            free(queue);
        }
        free(self.parent);
    }
}

#[test]
fn counts_added_events() {
    let f = Fixture::new();
    assert_eq!(size(f.child1), 3);
    assert_eq!(size(f.child2()), 4);
    assert_eq!(size(f.child3), 2);
}

#[test]
fn counts_removed_events() {
    let mut f = Fixture::new();
    for remaining in [2, 1, 0] {
        take(f.child1);
        assert_eq!(size(f.child1), remaining);
    }
    f.labels.put(f.child1, "c2ixx");
    assert_eq!(size(f.child1), 1);
    take(f.child1);
    assert_eq!(size(f.child1), 0);
    // Reading an empty queue yields nothing and leaves the count alone.
    assert!(take(f.child1).is_none());
    assert_eq!(size(f.child1), 0);
}

#[test]
fn removing_from_the_parent_removes_from_the_child() {
    let f = Fixture::new();
    for expected in ["c1i1", "c1i2", "c2i1", "c1i3", "c2i2", "c2i3", "c2i4"] {
        assert_eq!(f.get(f.parent), expected);
    }
}

#[test]
fn removing_from_a_child_removes_from_the_parent() {
    let f = Fixture::new();
    assert_eq!(f.get(f.child2()), "c2i1");
    assert_eq!(f.get(f.child2()), "c2i2");
    assert_eq!(f.get(f.child1), "c1i1");
    for expected in ["c1i2", "c1i3", "c2i3", "c2i4"] {
        assert_eq!(f.get(f.parent), expected);
    }
}

#[test]
fn removing_from_a_child_at_the_head_of_the_parent() {
    let f = Fixture::new();
    assert_eq!(f.get(f.child1), "c1i1");
    assert_eq!(f.get(f.child1), "c1i2");
    assert_eq!(f.get(f.parent), "c2i1");
}

#[test]
fn putting_to_a_child_after_draining_the_parent() {
    let mut f = Fixture::new();
    for expected in [
        "c1i1", "c1i2", "c2i1", "c1i3", "c2i2", "c2i3", "c2i4", "c3i1",
    ] {
        assert_eq!(f.get(f.parent), expected);
    }
    f.labels.put(f.child1, "c1i11");
    f.labels.put(f.child1, "c1i22");
    assert_eq!(f.get(f.parent), "c3i2");
    assert_eq!(f.get(f.parent), "c1i11");
    assert_eq!(f.get(f.parent), "c1i22");
}

#[test]
fn putting_to_a_child_after_draining_the_children() {
    let mut f = Fixture::new();
    assert_eq!(f.get(f.child1), "c1i1");
    assert_eq!(f.get(f.child1), "c1i2");
    assert_eq!(f.get(f.child2()), "c2i1");
    assert_eq!(f.get(f.child1), "c1i3");
    assert_eq!(f.get(f.child2()), "c2i2");
    assert_eq!(f.get(f.child2()), "c2i3");
    assert_eq!(f.get(f.child2()), "c2i4");
    assert_eq!(f.get(f.child3), "c3i1");
    assert_eq!(f.get(f.parent), "c3i2");
    f.labels.put(f.child1, "c1i11");
    f.labels.put(f.child2(), "c2i11");
    f.labels.put(f.child1, "c1i12");
    assert_eq!(f.get(f.child2()), "c2i11");
    assert_eq!(f.get(f.parent), "c1i11");
    assert_eq!(f.get(f.parent), "c1i12");
}

#[test]
fn putting_after_draining_a_child_at_the_tail_of_the_parent() {
    let mut f = Fixture::new();
    assert_eq!(f.get(f.child3), "c3i1");
    assert_eq!(f.get(f.child3), "c3i2");
    f.labels.put(f.child1, "c1i11");
    f.labels.put(f.child2(), "c2i11");
    for expected in [
        "c1i1", "c1i2", "c2i1", "c1i3", "c2i2", "c2i3", "c2i4", "c1i11", "c2i11",
    ] {
        assert_eq!(f.get(f.parent), expected);
    }
}

#[test]
fn freeing_a_child_removes_its_events_from_the_parent() {
    let mut f = Fixture::new();
    free(f.child2());
    f.child2 = None;
    for expected in ["c1i1", "c1i2", "c1i3"] {
        assert_eq!(f.get(f.parent), expected);
    }
    assert_eq!(f.get(f.child3), "c3i1");
    assert_eq!(f.get(f.child3), "c3i2");
}

#[test]
#[should_panic(expected = "a queue used after it was freed")]
fn a_freed_queue_is_refused_even_once_its_slot_is_reused() {
    let _exclusive = exclusive();
    let first = new_parent(None);
    free(first);
    let second = new_parent(None);
    // The second queue may well sit in the first one's slot; the first id
    // must not reach it.
    let _ = (second, is_empty(first));
}

#[test]
fn a_parentless_queue_is_plain_fifo() {
    let _exclusive = exclusive();
    let mut labels = Labels::new();
    let queue = new_parent(None);
    assert!(is_empty(queue));
    for label in ["a", "b", "c"] {
        labels.put(queue, label);
    }
    assert!(!is_empty(queue));
    assert_eq!(size(queue), 3);
    for expected in ["a", "b", "c"] {
        assert_eq!(labels.get(queue), expected);
    }
    assert!(is_empty(queue));
    free(queue);
}

#[test]
fn purging_drops_everything_reachable_from_the_parent() {
    let f = Fixture::new();
    purge(f.parent);
    assert!(is_empty(f.parent));
    for child in [f.child1, f.child2(), f.child3] {
        assert!(is_empty(child));
    }
}

/// Counts how many times [`count`] ran.
static CALLS: AtomicUsize = AtomicUsize::new(0);

fn count(_: u64, _: u64) {
    CALLS.fetch_add(1, Ordering::Relaxed);
}

#[test]
fn processing_reaches_what_the_handlers_queue() {
    let _exclusive = exclusive();
    let queue = new_parent(None);
    CALLS.store(0, Ordering::Relaxed);
    /// Queues one more copy of itself, up to four runs in all: draining a
    /// queue has to reach what the draining itself put there.
    fn requeue(queue: QueueId) {
        if CALLS.fetch_add(1, Ordering::Relaxed) < 3 {
            put(queue, Event::Job(Box::new(move || requeue(queue))));
        }
    }
    put(queue, Event::Job(Box::new(move || requeue(queue))));
    process(queue);
    assert_eq!(CALLS.load(Ordering::Relaxed), 4);
    assert!(is_empty(queue));
    free(queue);
}

#[test]
fn moving_events_preserves_their_order() {
    let _exclusive = exclusive();
    let mut labels = Labels::new();
    let from = new_parent(None);
    let to = new_parent(None);
    for label in ["a", "b", "c"] {
        labels.put(from, label);
    }
    move_events(to, from);
    assert!(is_empty(from));
    assert_eq!(size(from), 0);
    assert_eq!(size(to), 3);
    for expected in ["a", "b", "c"] {
        assert_eq!(labels.get(to), expected);
    }
    free(from);
    free(to);
}

#[test]
fn an_empty_child_can_be_re_homed() {
    let _exclusive = exclusive();
    let mut labels = Labels::new();
    let first = new_parent(None);
    let second = new_parent(None);
    let child = new_child(first);
    replace_parent(child, second);
    labels.put(child, "after");
    // The event reaches the new parent and not the old one.
    assert!(is_empty(first));
    assert_eq!(labels.get(second), "after");
    free(child);
    free(first);
    free(second);
}

/// Counts the runs of the parent's `on_put` hook.
static WAKES: AtomicUsize = AtomicUsize::new(0);

fn wake() {
    WAKES.fetch_add(1, Ordering::Relaxed);
}

#[test]
fn a_put_to_a_child_wakes_the_parent_and_a_put_to_it_does_not() {
    let _exclusive = exclusive();
    let parent = new_parent(Some(wake));
    let child = new_child(parent);
    WAKES.store(0, Ordering::Relaxed);
    put(child, labelled(0));
    assert_eq!(WAKES.load(Ordering::Relaxed), 1);
    put(parent, labelled(1));
    assert_eq!(WAKES.load(Ordering::Relaxed), 1);
    purge(parent);
    free(child);
    free(parent);
}

#[test]
fn processing_runs_every_handler_once() {
    let _exclusive = exclusive();
    let queue = new_parent(None);
    let child = new_child(queue);
    for _ in 0..4 {
        put(child, Event::Ids(count, 0, 0));
    }
    CALLS.store(0, Ordering::Relaxed);
    process(queue);
    assert_eq!(CALLS.load(Ordering::Relaxed), 4);
    assert!(is_empty(queue));
    assert!(is_empty(child));
    free(child);
    free(queue);
}
