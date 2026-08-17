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
//! retire once the loop and channel code above them is rewritten.

use crate::types::multiqueue_list::{Item, ItemList};
use crate::types::{Event, MultiQueue, PutCallback, multiqueue, size_t};
use core::ffi::c_void;
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
    refcount: ::core::ffi::c_int,
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn multiqueue_new(on_put: PutCallback, data: *mut c_void) -> *mut MultiQueue {
    new_queue(ptr::null_mut(), on_put, data)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn multiqueue_new_child(parent: *mut MultiQueue) -> *mut MultiQueue {
    assert!(
        (*parent).parent.is_null(),
        "queues nest only one level deep"
    );
    // Upstream counts a new child in the parent's size. See the field's note.
    (*parent).size = (*parent).size.wrapping_add(1);
    new_queue(parent, None, ptr::null_mut())
}

fn new_queue(parent: *mut MultiQueue, on_put: PutCallback, data: *mut c_void) -> *mut MultiQueue {
    Box::into_raw(Box::new(multiqueue {
        parent,
        on_put,
        data,
        size: 0,
        items: Box::into_raw(Box::new(ItemList::new())),
    }))
}

/// The items `queue` owns. Every queue is created with a list and keeps it
/// until `multiqueue_free` takes both away together.
unsafe fn items<'a>(queue: *mut MultiQueue) -> &'a mut ItemList {
    &mut *(*queue).items
}

/// Release `queue` and unlink whatever it still holds from its parent.
///
/// Freeing a *parent* leaves its children pointing at released memory, as it
/// did upstream; nothing does that while children are alive.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn multiqueue_free(queue: *mut MultiQueue) {
    debug_assert!(!queue.is_null());
    // Both boxes stay alive until the end of the scope: the parent is read
    // out of `owned` while its items are being unlinked.
    let owned = Box::from_raw(queue);
    let mut own_items = Box::from_raw(owned.items);
    while let Some(item) = own_items.pop_front() {
        if let Item::Event {
            parent_slot: Some(slot),
            ..
        } = item
        {
            items(owned.parent).remove(slot);
        }
    }
}

/// The next event, or a handler-less event when there is none.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn multiqueue_get(queue: *mut MultiQueue) -> Event {
    if multiqueue_empty(queue) {
        NIL_EVENT
    } else {
        take_event(queue)
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn multiqueue_put_event(queue: *mut MultiQueue, event: Event) {
    debug_assert!(!queue.is_null());
    let parent = (*queue).parent;
    let parent_slot = if parent.is_null() {
        None
    } else {
        Some(items(parent).push_back(Item::Link { child: queue }))
    };
    items(queue).push_back(Item::Event { event, parent_slot });
    (*queue).size = (*queue).size.wrapping_add(1);
    // Only a child's put notifies, and only through its parent's callback —
    // a parentless queue's own `on_put` is never reached from here.
    if !parent.is_null()
        && let Some(on_put) = (*parent).on_put
    {
        on_put(parent, (*parent).data);
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn multiqueue_size(queue: *mut MultiQueue) -> size_t {
    (*queue).size
}

/// Take the head event, unlinking it from the other side as well.
unsafe fn take_event(queue: *mut MultiQueue) -> Event {
    let item = items(queue)
        .pop_front()
        .expect("caller checked the queue is not empty");
    let event = match item {
        // Reached from a parent: the link stands for the child's head event.
        // Upstream leaves the child's own counter alone here, so this does
        // too.
        Item::Link { child } => match items(child).pop_front() {
            Some(Item::Event { event, .. }) => event,
            _ => panic!("a link names an event of a non-empty child"),
        },
        // Reached from the queue the event was pushed to.
        Item::Event { event, parent_slot } => {
            if let Some(slot) = parent_slot {
                items((*queue).parent).remove(slot);
            }
            event
        }
    };
    (*queue).size = (*queue).size.wrapping_sub(1);
    event
}

pub unsafe fn multiqueue_empty(queue: *mut MultiQueue) -> bool {
    debug_assert!(!queue.is_null());
    items(queue).is_empty()
}

pub unsafe fn multiqueue_move_events(dest: *mut MultiQueue, src: *mut MultiQueue) {
    while !multiqueue_empty(src) {
        let event = multiqueue_get(src);
        multiqueue_put_event(dest, event);
    }
}

pub unsafe fn multiqueue_process_events(queue: *mut MultiQueue) {
    debug_assert!(!queue.is_null());
    while !multiqueue_empty(queue) {
        let mut event = take_event(queue);
        if let Some(handler) = event.handler {
            handler(&raw mut event.argv as *mut *mut c_void);
        }
    }
}

pub unsafe fn multiqueue_purge_events(queue: *mut MultiQueue) {
    debug_assert!(!queue.is_null());
    while !multiqueue_empty(queue) {
        take_event(queue);
    }
}

pub unsafe fn multiqueue_replace_parent(queue: *mut MultiQueue, new_parent: *mut MultiQueue) {
    debug_assert!(multiqueue_empty(queue));
    (*queue).parent = new_parent;
}

/// An event that fires the first time it is reached and is released once
/// `num` queues have reached it.
pub fn event_create_oneshot(event: Event, num: ::core::ffi::c_int) -> Event {
    let data = Box::into_raw(Box::new(MulticastEvent {
        event,
        fired: false,
        refcount: num,
    }));
    let mut argv = [ptr::null_mut::<c_void>(); 10];
    argv[0] = data.cast();
    Event {
        handler: Some(multiqueue_oneshot_event),
        argv,
    }
}

unsafe extern "C" fn multiqueue_oneshot_event(argv: *mut *mut c_void) {
    let data = (*argv.offset(0)).cast::<MulticastEvent>();
    if !(*data).fired {
        (*data).fired = true;
        if let Some(handler) = (*data).event.handler {
            handler(&raw mut (*data).event.argv as *mut *mut c_void);
        }
    }
    (*data).refcount -= 1;
    if (*data).refcount == 0 {
        drop(Box::from_raw(data));
    }
}
