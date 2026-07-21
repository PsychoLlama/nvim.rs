use crate::src::nvim::global_cell::GlobalCell;
extern "C" {
    fn __assert_fail(
        __assertion: *const ::core::ffi::c_char,
        __file: *const ::core::ffi::c_char,
        __line: ::core::ffi::c_uint,
        __function: *const ::core::ffi::c_char,
    ) -> !;
    fn xmalloc(size: size_t) -> *mut ::core::ffi::c_void;
    fn xfree(ptr: *mut ::core::ffi::c_void);
}
pub type size_t = usize;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct queue {
    pub next: *mut queue,
    pub prev: *mut queue,
}
pub type QUEUE = queue;
pub type argv_callback = Option<unsafe extern "C" fn(*mut *mut ::core::ffi::c_void) -> ()>;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct Event {
    pub handler: argv_callback,
    pub argv: [*mut ::core::ffi::c_void; 10],
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct multiqueue {
    pub parent: *mut MultiQueue,
    pub headtail: QUEUE,
    pub on_put: PutCallback,
    pub data: *mut ::core::ffi::c_void,
    pub size: size_t,
}
pub type PutCallback =
    Option<unsafe extern "C" fn(*mut MultiQueue, *mut ::core::ffi::c_void) -> ()>;
pub type MultiQueue = multiqueue;
pub type MultiQueueItem = multiqueue_item;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct multiqueue_item {
    pub data: C2Rust_Unnamed,
    pub link: bool,
    pub node: QUEUE,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2Rust_Unnamed {
    pub queue: *mut MultiQueue,
    pub item: C2Rust_Unnamed_0,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2Rust_Unnamed_0 {
    pub event: Event,
    pub parent_item: *mut MultiQueueItem,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct MulticastEvent {
    pub event: Event,
    pub fired: bool,
    pub refcount: ::core::ffi::c_int,
}
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
#[inline(always)]
unsafe extern "C" fn QUEUE_EMPTY(q: *const QUEUE) -> ::core::ffi::c_int {
    return (q == (*q).next as *const QUEUE) as ::core::ffi::c_int;
}
#[inline(always)]
unsafe extern "C" fn QUEUE_INIT(q: *mut QUEUE) {
    (*q).next = q as *mut queue;
    (*q).prev = q as *mut queue;
}
#[inline(always)]
unsafe extern "C" fn QUEUE_INSERT_TAIL(h: *mut QUEUE, q: *mut QUEUE) {
    (*q).next = h as *mut queue;
    (*q).prev = (*h).prev;
    (*(*q).prev).next = q as *mut queue;
    (*h).prev = q as *mut queue;
}
#[inline(always)]
unsafe extern "C" fn QUEUE_REMOVE(q: *mut QUEUE) {
    (*(*q).prev).next = (*q).next;
    (*(*q).next).prev = (*q).prev;
}
static NILEVENT: GlobalCell<Event> = GlobalCell::new(Event {
    handler: None,
    argv: [
        NULL,
        ::core::ptr::null_mut::<::core::ffi::c_void>(),
        ::core::ptr::null_mut::<::core::ffi::c_void>(),
        ::core::ptr::null_mut::<::core::ffi::c_void>(),
        ::core::ptr::null_mut::<::core::ffi::c_void>(),
        ::core::ptr::null_mut::<::core::ffi::c_void>(),
        ::core::ptr::null_mut::<::core::ffi::c_void>(),
        ::core::ptr::null_mut::<::core::ffi::c_void>(),
        ::core::ptr::null_mut::<::core::ffi::c_void>(),
        ::core::ptr::null_mut::<::core::ffi::c_void>(),
    ],
});
#[no_mangle]
pub unsafe extern "C" fn multiqueue_new(
    mut on_put: PutCallback,
    mut data: *mut ::core::ffi::c_void,
) -> *mut MultiQueue {
    return _multiqueue_new(::core::ptr::null_mut::<MultiQueue>(), on_put, data);
}
#[no_mangle]
pub unsafe extern "C" fn multiqueue_new_child(mut parent: *mut MultiQueue) -> *mut MultiQueue {
    '_c2rust_label: {
        if (*parent).parent.is_null() {
        } else {
            __assert_fail(
                b"!parent->parent\0".as_ptr() as *const ::core::ffi::c_char,
                b"src/nvim/event/multiqueue.rs\0".as_ptr() as *const ::core::ffi::c_char,
                95 as ::core::ffi::c_uint,
                b"MultiQueue *multiqueue_new_child(MultiQueue *)\0".as_ptr()
                    as *const ::core::ffi::c_char,
            );
        }
    };
    (*parent).size = (*parent).size.wrapping_add(1);
    return _multiqueue_new(parent, None, NULL);
}
unsafe extern "C" fn _multiqueue_new(
    mut parent: *mut MultiQueue,
    mut on_put: PutCallback,
    mut data: *mut ::core::ffi::c_void,
) -> *mut MultiQueue {
    let mut rv: *mut MultiQueue = xmalloc(::core::mem::size_of::<MultiQueue>()) as *mut MultiQueue;
    QUEUE_INIT(&raw mut (*rv).headtail);
    (*rv).size = 0 as size_t;
    (*rv).parent = parent;
    (*rv).on_put = on_put;
    (*rv).data = data;
    return rv;
}
#[no_mangle]
pub unsafe extern "C" fn multiqueue_free(mut self_0: *mut MultiQueue) {
    '_c2rust_label: {
        if !self_0.is_null() {
        } else {
            __assert_fail(
                b"self\0".as_ptr() as *const ::core::ffi::c_char,
                b"src/nvim/event/multiqueue.rs\0".as_ptr() as *const ::core::ffi::c_char,
                113 as ::core::ffi::c_uint,
                b"void multiqueue_free(MultiQueue *)\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
    };
    let mut q: *mut QUEUE = ::core::ptr::null_mut::<QUEUE>();
    q = (*self_0).headtail.next as *mut QUEUE;
    while q != &raw mut (*self_0).headtail {
        let mut next: *mut QUEUE = (*q).next as *mut QUEUE;
        let mut item: *mut MultiQueueItem = multiqueue_node_data(q);
        if !(*self_0).parent.is_null() {
            QUEUE_REMOVE(&raw mut (*(*item).data.item.parent_item).node);
            xfree((*item).data.item.parent_item as *mut ::core::ffi::c_void);
        }
        QUEUE_REMOVE(q);
        xfree(item as *mut ::core::ffi::c_void);
        q = next;
    }
    xfree(self_0 as *mut ::core::ffi::c_void);
}
#[no_mangle]
pub unsafe extern "C" fn multiqueue_get(mut self_0: *mut MultiQueue) -> Event {
    return if multiqueue_empty(self_0) as ::core::ffi::c_int != 0 {
        NILEVENT.get()
    } else {
        multiqueue_remove(self_0)
    };
}
#[no_mangle]
pub unsafe extern "C" fn multiqueue_put_event(mut self_0: *mut MultiQueue, mut event: Event) {
    '_c2rust_label: {
        if !self_0.is_null() {
        } else {
            __assert_fail(
                b"self\0".as_ptr() as *const ::core::ffi::c_char,
                b"src/nvim/event/multiqueue.rs\0".as_ptr() as *const ::core::ffi::c_char,
                136 as ::core::ffi::c_uint,
                b"void multiqueue_put_event(MultiQueue *, Event)\0".as_ptr()
                    as *const ::core::ffi::c_char,
            );
        }
    };
    multiqueue_push(self_0, event);
    if !(*self_0).parent.is_null() && (*(*self_0).parent).on_put.is_some() {
        (*(*self_0).parent)
            .on_put
            .expect("non-null function pointer")((*self_0).parent, (*(*self_0).parent).data);
    }
}
#[no_mangle]
pub unsafe extern "C" fn multiqueue_move_events(
    mut dest: *mut MultiQueue,
    mut src: *mut MultiQueue,
) {
    while !multiqueue_empty(src) {
        let mut event: Event = multiqueue_get(src);
        multiqueue_put_event(dest, event);
    }
}
#[no_mangle]
pub unsafe extern "C" fn multiqueue_process_events(mut self_0: *mut MultiQueue) {
    '_c2rust_label: {
        if !self_0.is_null() {
        } else {
            __assert_fail(
                b"self\0".as_ptr() as *const ::core::ffi::c_char,
                b"src/nvim/event/multiqueue.rs\0".as_ptr() as *const ::core::ffi::c_char,
                155 as ::core::ffi::c_uint,
                b"void multiqueue_process_events(MultiQueue *)\0".as_ptr()
                    as *const ::core::ffi::c_char,
            );
        }
    };
    while !multiqueue_empty(self_0) {
        let mut event: Event = multiqueue_remove(self_0);
        if event.handler.is_some() {
            event.handler.expect("non-null function pointer")(
                &raw mut event.argv as *mut *mut ::core::ffi::c_void,
            );
        }
    }
}
#[no_mangle]
pub unsafe extern "C" fn multiqueue_purge_events(mut self_0: *mut MultiQueue) {
    '_c2rust_label: {
        if !self_0.is_null() {
        } else {
            __assert_fail(
                b"self\0".as_ptr() as *const ::core::ffi::c_char,
                b"src/nvim/event/multiqueue.rs\0".as_ptr() as *const ::core::ffi::c_char,
                167 as ::core::ffi::c_uint,
                b"void multiqueue_purge_events(MultiQueue *)\0".as_ptr()
                    as *const ::core::ffi::c_char,
            );
        }
    };
    while !multiqueue_empty(self_0) {
        multiqueue_remove(self_0);
    }
}
#[no_mangle]
pub unsafe extern "C" fn multiqueue_empty(mut self_0: *mut MultiQueue) -> bool {
    '_c2rust_label: {
        if !self_0.is_null() {
        } else {
            __assert_fail(
                b"self\0".as_ptr() as *const ::core::ffi::c_char,
                b"src/nvim/event/multiqueue.rs\0".as_ptr() as *const ::core::ffi::c_char,
                175 as ::core::ffi::c_uint,
                b"_Bool multiqueue_empty(MultiQueue *)\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
    };
    return QUEUE_EMPTY(&raw mut (*self_0).headtail) != 0;
}
#[no_mangle]
pub unsafe extern "C" fn multiqueue_replace_parent(
    mut self_0: *mut MultiQueue,
    mut new_parent: *mut MultiQueue,
) {
    '_c2rust_label: {
        if multiqueue_empty(self_0) {
        } else {
            __assert_fail(
                b"multiqueue_empty(self)\0".as_ptr() as *const ::core::ffi::c_char,
                b"src/nvim/event/multiqueue.rs\0".as_ptr() as *const ::core::ffi::c_char,
                181 as ::core::ffi::c_uint,
                b"void multiqueue_replace_parent(MultiQueue *, MultiQueue *)\0".as_ptr()
                    as *const ::core::ffi::c_char,
            );
        }
    };
    (*self_0).parent = new_parent;
}
#[no_mangle]
pub unsafe extern "C" fn multiqueue_size(mut self_0: *mut MultiQueue) -> size_t {
    return (*self_0).size;
}
unsafe extern "C" fn multiqueueitem_get_event(
    mut item: *mut MultiQueueItem,
    mut remove: bool,
) -> Event {
    '_c2rust_label: {
        if !item.is_null() {
        } else {
            __assert_fail(
                b"item != NULL\0".as_ptr() as *const ::core::ffi::c_char,
                b"src/nvim/event/multiqueue.rs\0".as_ptr() as *const ::core::ffi::c_char,
                196 as ::core::ffi::c_uint,
                b"Event multiqueueitem_get_event(MultiQueueItem *, _Bool)\0".as_ptr()
                    as *const ::core::ffi::c_char,
            );
        }
    };
    let mut ev: Event = Event {
        handler: None,
        argv: [::core::ptr::null_mut::<::core::ffi::c_void>(); 10],
    };
    if (*item).link {
        let mut linked: *mut MultiQueue = (*item).data.queue;
        '_c2rust_label_0: {
            if !multiqueue_empty(linked) {
            } else {
                __assert_fail(
                    b"!multiqueue_empty(linked)\0".as_ptr() as *const ::core::ffi::c_char,
                    b"src/nvim/event/multiqueue.rs\0".as_ptr() as *const ::core::ffi::c_char,
                    201 as ::core::ffi::c_uint,
                    b"Event multiqueueitem_get_event(MultiQueueItem *, _Bool)\0".as_ptr()
                        as *const ::core::ffi::c_char,
                );
            }
        };
        let mut child: *mut MultiQueueItem =
            multiqueue_node_data((*linked).headtail.next as *mut QUEUE);
        ev = (*child).data.item.event;
        if remove {
            QUEUE_REMOVE(&raw mut (*child).node);
            xfree(child as *mut ::core::ffi::c_void);
        }
    } else {
        if remove as ::core::ffi::c_int != 0 && !(*item).data.item.parent_item.is_null() {
            QUEUE_REMOVE(&raw mut (*(*item).data.item.parent_item).node);
            xfree((*item).data.item.parent_item as *mut ::core::ffi::c_void);
            (*item).data.item.parent_item = ::core::ptr::null_mut::<MultiQueueItem>();
        }
        ev = (*item).data.item.event;
    }
    return ev;
}
unsafe extern "C" fn multiqueue_remove(mut self_0: *mut MultiQueue) -> Event {
    '_c2rust_label: {
        if !multiqueue_empty(self_0) {
        } else {
            __assert_fail(
                b"!multiqueue_empty(self)\0".as_ptr() as *const ::core::ffi::c_char,
                b"src/nvim/event/multiqueue.rs\0".as_ptr() as *const ::core::ffi::c_char,
                224 as ::core::ffi::c_uint,
                b"Event multiqueue_remove(MultiQueue *)\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
    };
    let mut h: *mut QUEUE = (*self_0).headtail.next as *mut QUEUE;
    QUEUE_REMOVE(h);
    let mut item: *mut MultiQueueItem = multiqueue_node_data(h);
    '_c2rust_label_0: {
        if !(*item).link || (*self_0).parent.is_null() {
        } else {
            __assert_fail(
                b"!item->link || !self->parent\0".as_ptr() as *const ::core::ffi::c_char,
                b"src/nvim/event/multiqueue.rs\0".as_ptr() as *const ::core::ffi::c_char,
                228 as ::core::ffi::c_uint,
                b"Event multiqueue_remove(MultiQueue *)\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
    };
    let mut ev: Event = multiqueueitem_get_event(item, true_0 != 0);
    (*self_0).size = (*self_0).size.wrapping_sub(1);
    xfree(item as *mut ::core::ffi::c_void);
    return ev;
}
unsafe extern "C" fn multiqueue_push(mut self_0: *mut MultiQueue, mut event: Event) {
    let mut item: *mut MultiQueueItem =
        xmalloc(::core::mem::size_of::<MultiQueueItem>()) as *mut MultiQueueItem;
    (*item).link = false_0 != 0;
    (*item).data.item.event = event;
    (*item).data.item.parent_item = ::core::ptr::null_mut::<MultiQueueItem>();
    QUEUE_INSERT_TAIL(&raw mut (*self_0).headtail, &raw mut (*item).node);
    if !(*self_0).parent.is_null() {
        (*item).data.item.parent_item =
            xmalloc(::core::mem::size_of::<MultiQueueItem>()) as *mut MultiQueueItem;
        (*(*item).data.item.parent_item).link = true_0 != 0;
        (*(*item).data.item.parent_item).data.queue = self_0;
        QUEUE_INSERT_TAIL(
            &raw mut (*(*self_0).parent).headtail,
            &raw mut (*(*item).data.item.parent_item).node,
        );
    }
    (*self_0).size = (*self_0).size.wrapping_add(1);
}
unsafe extern "C" fn multiqueue_node_data(mut q: *mut QUEUE) -> *mut MultiQueueItem {
    return (q as *mut ::core::ffi::c_char).offset(-(104 as ::core::ffi::c_ulong as isize))
        as *mut MultiQueueItem;
}
#[no_mangle]
pub unsafe extern "C" fn event_create_oneshot(mut ev: Event, mut num: ::core::ffi::c_int) -> Event {
    let mut data: *mut MulticastEvent =
        xmalloc(::core::mem::size_of::<MulticastEvent>()) as *mut MulticastEvent;
    (*data).event = ev;
    (*data).fired = false_0 != 0;
    (*data).refcount = num;
    return Event {
        handler: Some(
            multiqueue_oneshot_event as unsafe extern "C" fn(*mut *mut ::core::ffi::c_void) -> (),
        ),
        argv: [
            data as *mut ::core::ffi::c_void,
            ::core::ptr::null_mut::<::core::ffi::c_void>(),
            ::core::ptr::null_mut::<::core::ffi::c_void>(),
            ::core::ptr::null_mut::<::core::ffi::c_void>(),
            ::core::ptr::null_mut::<::core::ffi::c_void>(),
            ::core::ptr::null_mut::<::core::ffi::c_void>(),
            ::core::ptr::null_mut::<::core::ffi::c_void>(),
            ::core::ptr::null_mut::<::core::ffi::c_void>(),
            ::core::ptr::null_mut::<::core::ffi::c_void>(),
            ::core::ptr::null_mut::<::core::ffi::c_void>(),
        ],
    };
}
unsafe extern "C" fn multiqueue_oneshot_event(mut argv: *mut *mut ::core::ffi::c_void) {
    let mut data: *mut MulticastEvent =
        *argv.offset(0 as ::core::ffi::c_int as isize) as *mut MulticastEvent;
    if !(*data).fired {
        (*data).fired = true_0 != 0;
        if (*data).event.handler.is_some() {
            (*data).event.handler.expect("non-null function pointer")(
                &raw mut (*data).event.argv as *mut *mut ::core::ffi::c_void,
            );
        }
    }
    (*data).refcount -= 1;
    if (*data).refcount == 0 as ::core::ffi::c_int {
        xfree(data as *mut ::core::ffi::c_void);
    }
}
pub const true_0: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const false_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
