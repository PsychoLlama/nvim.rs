//! The multi-level queue's ordering semantics, mirroring
//! `test/unit/multiqueue_spec.lua`. That spec pushes an event whose first
//! `argv` slot is a label and reads the label back, which is exactly what the
//! helpers here do — but from Rust, so the whole thing runs under Miri.

use std::ffi::{CStr, CString, c_void};
use std::sync::atomic::{AtomicUsize, Ordering};

use c2rust_neovim::src::nvim::event::multiqueue::{
    Event, MultiQueue, multiqueue_empty, multiqueue_free, multiqueue_get, multiqueue_new,
    multiqueue_new_child, multiqueue_process_events, multiqueue_purge_events, multiqueue_put_event,
    multiqueue_size,
};

/// An event carrying `label` and no handler, so `get` hands the label back.
fn labelled(label: &CStr) -> Event {
    let mut argv = [std::ptr::null_mut::<c_void>(); 10];
    argv[0] = label.as_ptr() as *mut c_void;
    Event {
        handler: None,
        argv,
    }
}

/// The labels a fixture keeps alive for the duration of a test.
struct Labels(Vec<CString>);

impl Labels {
    fn new() -> Self {
        Labels(Vec::new())
    }

    fn put(&mut self, queue: *mut MultiQueue, label: &str) {
        self.0.push(CString::new(label).unwrap());
        let event = labelled(self.0.last().unwrap());
        unsafe { multiqueue_put_event(queue, event) };
    }
}

/// The label of the next event on `queue`.
fn get(queue: *mut MultiQueue) -> String {
    let event = unsafe { multiqueue_get(queue) };
    assert!(!event.argv[0].is_null(), "queue was empty");
    unsafe { CStr::from_ptr(event.argv[0].cast()) }
        .to_str()
        .unwrap()
        .to_owned()
}

/// The parent/three-children fixture the Lua spec builds in `before_each`.
struct Fixture {
    labels: Labels,
    parent: *mut MultiQueue,
    child1: *mut MultiQueue,
    child2: *mut MultiQueue,
    child3: *mut MultiQueue,
}

impl Fixture {
    fn new() -> Self {
        let mut labels = Labels::new();
        let parent = unsafe { multiqueue_new(None, std::ptr::null_mut()) };
        let child1 = unsafe { multiqueue_new_child(parent) };
        let child2 = unsafe { multiqueue_new_child(parent) };
        let child3 = unsafe { multiqueue_new_child(parent) };
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
            labels,
            parent,
            child1,
            child2,
            child3,
        }
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        for queue in [self.child1, self.child2, self.child3, self.parent] {
            if !queue.is_null() {
                unsafe { multiqueue_free(queue) };
            }
        }
    }
}

#[test]
fn counts_added_events() {
    let f = Fixture::new();
    unsafe {
        assert_eq!(multiqueue_size(f.child1), 3);
        assert_eq!(multiqueue_size(f.child2), 4);
        assert_eq!(multiqueue_size(f.child3), 2);
    }
}

#[test]
fn counts_removed_events() {
    let mut f = Fixture::new();
    unsafe {
        multiqueue_get(f.child1);
        assert_eq!(multiqueue_size(f.child1), 2);
        multiqueue_get(f.child1);
        assert_eq!(multiqueue_size(f.child1), 1);
        multiqueue_get(f.child1);
        assert_eq!(multiqueue_size(f.child1), 0);
    }
    f.labels.put(f.child1, "c2ixx");
    unsafe {
        assert_eq!(multiqueue_size(f.child1), 1);
        multiqueue_get(f.child1);
        assert_eq!(multiqueue_size(f.child1), 0);
        // Reading an empty queue yields the handler-less event and leaves the
        // count alone.
        multiqueue_get(f.child1);
        assert_eq!(multiqueue_size(f.child1), 0);
    }
}

#[test]
fn removing_from_the_parent_removes_from_the_child() {
    let f = Fixture::new();
    for expected in ["c1i1", "c1i2", "c2i1", "c1i3", "c2i2", "c2i3", "c2i4"] {
        assert_eq!(get(f.parent), expected);
    }
}

#[test]
fn removing_from_a_child_removes_from_the_parent() {
    let f = Fixture::new();
    assert_eq!(get(f.child2), "c2i1");
    assert_eq!(get(f.child2), "c2i2");
    assert_eq!(get(f.child1), "c1i1");
    for expected in ["c1i2", "c1i3", "c2i3", "c2i4"] {
        assert_eq!(get(f.parent), expected);
    }
}

#[test]
fn removing_from_a_child_at_the_head_of_the_parent() {
    let f = Fixture::new();
    assert_eq!(get(f.child1), "c1i1");
    assert_eq!(get(f.child1), "c1i2");
    assert_eq!(get(f.parent), "c2i1");
}

#[test]
fn putting_to_a_child_after_draining_the_parent() {
    let mut f = Fixture::new();
    for expected in [
        "c1i1", "c1i2", "c2i1", "c1i3", "c2i2", "c2i3", "c2i4", "c3i1",
    ] {
        assert_eq!(get(f.parent), expected);
    }
    f.labels.put(f.child1, "c1i11");
    f.labels.put(f.child1, "c1i22");
    assert_eq!(get(f.parent), "c3i2");
    assert_eq!(get(f.parent), "c1i11");
    assert_eq!(get(f.parent), "c1i22");
}

#[test]
fn putting_to_a_child_after_draining_the_children() {
    let mut f = Fixture::new();
    assert_eq!(get(f.child1), "c1i1");
    assert_eq!(get(f.child1), "c1i2");
    assert_eq!(get(f.child2), "c2i1");
    assert_eq!(get(f.child1), "c1i3");
    assert_eq!(get(f.child2), "c2i2");
    assert_eq!(get(f.child2), "c2i3");
    assert_eq!(get(f.child2), "c2i4");
    assert_eq!(get(f.child3), "c3i1");
    assert_eq!(get(f.parent), "c3i2");
    f.labels.put(f.child1, "c1i11");
    f.labels.put(f.child2, "c2i11");
    f.labels.put(f.child1, "c1i12");
    assert_eq!(get(f.child2), "c2i11");
    assert_eq!(get(f.parent), "c1i11");
    assert_eq!(get(f.parent), "c1i12");
}

#[test]
fn putting_after_draining_a_child_at_the_tail_of_the_parent() {
    let mut f = Fixture::new();
    assert_eq!(get(f.child3), "c3i1");
    assert_eq!(get(f.child3), "c3i2");
    f.labels.put(f.child1, "c1i11");
    f.labels.put(f.child2, "c2i11");
    for expected in [
        "c1i1", "c1i2", "c2i1", "c1i3", "c2i2", "c2i3", "c2i4", "c1i11", "c2i11",
    ] {
        assert_eq!(get(f.parent), expected);
    }
}

#[test]
fn freeing_a_child_removes_its_events_from_the_parent() {
    let mut f = Fixture::new();
    unsafe { multiqueue_free(f.child2) };
    f.child2 = std::ptr::null_mut();
    for expected in ["c1i1", "c1i2", "c1i3"] {
        assert_eq!(get(f.parent), expected);
    }
    assert_eq!(get(f.child3), "c3i1");
    assert_eq!(get(f.child3), "c3i2");
}

#[test]
fn a_parentless_queue_is_plain_fifo() {
    let mut labels = Labels::new();
    let queue = unsafe { multiqueue_new(None, std::ptr::null_mut()) };
    assert!(unsafe { multiqueue_empty(queue) });
    for label in ["a", "b", "c"] {
        labels.put(queue, label);
    }
    assert!(!unsafe { multiqueue_empty(queue) });
    assert_eq!(unsafe { multiqueue_size(queue) }, 3);
    for expected in ["a", "b", "c"] {
        assert_eq!(get(queue), expected);
    }
    assert!(unsafe { multiqueue_empty(queue) });
    unsafe { multiqueue_free(queue) };
}

#[test]
fn purging_drops_everything_reachable_from_the_parent() {
    let f = Fixture::new();
    unsafe { multiqueue_purge_events(f.parent) };
    assert!(unsafe { multiqueue_empty(f.parent) });
    for child in [f.child1, f.child2, f.child3] {
        assert!(unsafe { multiqueue_empty(child) });
    }
}

/// Counts how many times [`count_handler`] ran.
static HANDLER_CALLS: AtomicUsize = AtomicUsize::new(0);

unsafe extern "C" fn count_handler(_argv: *mut *mut c_void) {
    HANDLER_CALLS.fetch_add(1, Ordering::Relaxed);
}

#[test]
fn processing_runs_every_handler_once() {
    let queue = unsafe { multiqueue_new(None, std::ptr::null_mut()) };
    let child = unsafe { multiqueue_new_child(queue) };
    for _ in 0..4 {
        let event = Event {
            handler: Some(count_handler),
            argv: [std::ptr::null_mut(); 10],
        };
        unsafe { multiqueue_put_event(child, event) };
    }
    HANDLER_CALLS.store(0, Ordering::Relaxed);
    unsafe {
        multiqueue_process_events(queue);
        assert_eq!(HANDLER_CALLS.load(Ordering::Relaxed), 4);
        assert!(multiqueue_empty(queue));
        assert!(multiqueue_empty(child));
        multiqueue_free(child);
        multiqueue_free(queue);
    }
}
