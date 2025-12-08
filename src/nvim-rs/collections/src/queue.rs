//! Queue utilities for Neovim
//!
//! This module provides C-compatible implementations of queue operations
//! compatible with nvim's lib/queue_defs.h circular linked list.

use std::ffi::c_int;

/// QUEUE structure matching Neovim's lib/queue_defs.h
///
/// A circular doubly-linked list node.
#[repr(C)]
pub struct Queue {
    /// Pointer to the next node in the queue
    pub next: *mut Queue,
    /// Pointer to the previous node in the queue
    pub prev: *mut Queue,
}

/// Check if the queue is empty.
///
/// A queue is empty when it points to itself (circular reference to self).
///
/// # Safety
///
/// `q` must be a valid, non-null pointer to a Queue struct.
#[no_mangle]
pub unsafe extern "C" fn rs_queue_empty(q: *const Queue) -> c_int {
    if q.is_null() {
        return 1; // Treat null as empty
    }
    c_int::from(q == (*q).next)
}

/// Initialize a queue to empty state (self-referential).
///
/// # Safety
///
/// `q` must be a valid, non-null pointer to a Queue struct.
#[no_mangle]
pub unsafe extern "C" fn rs_queue_init(q: *mut Queue) {
    (*q).next = q;
    (*q).prev = q;
}

/// Add all nodes from queue `n` to the end of queue `h`.
///
/// After this operation, `n` becomes empty (but still valid).
///
/// # Safety
///
/// `h` and `n` must be valid, non-null pointers to Queue structs.
#[no_mangle]
pub unsafe extern "C" fn rs_queue_add(h: *mut Queue, n: *mut Queue) {
    (*(*h).prev).next = (*n).next;
    (*(*n).next).prev = (*h).prev;
    (*h).prev = (*n).prev;
    (*(*h).prev).next = h;
}

/// Insert node `q` at the head (front) of queue `h`.
///
/// # Safety
///
/// `h` and `q` must be valid, non-null pointers to Queue structs.
#[no_mangle]
pub unsafe extern "C" fn rs_queue_insert_head(h: *mut Queue, q: *mut Queue) {
    (*q).next = (*h).next;
    (*q).prev = h;
    (*(*q).next).prev = q;
    (*h).next = q;
}

/// Insert node `q` at the tail (end) of queue `h`.
///
/// # Safety
///
/// `h` and `q` must be valid, non-null pointers to Queue structs.
#[no_mangle]
pub unsafe extern "C" fn rs_queue_insert_tail(h: *mut Queue, q: *mut Queue) {
    (*q).next = h;
    (*q).prev = (*h).prev;
    (*(*q).prev).next = q;
    (*h).prev = q;
}

/// Remove node `q` from its current queue.
///
/// After removal, `q` is unlinked but its next/prev pointers are unchanged.
///
/// # Safety
///
/// `q` must be a valid, non-null pointer to a Queue struct that is linked in a queue.
#[no_mangle]
pub unsafe extern "C" fn rs_queue_remove(q: *mut Queue) {
    (*(*q).prev).next = (*q).next;
    (*(*q).next).prev = (*q).prev;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_queue_empty() {
        // Create an empty queue (points to itself)
        let mut empty_queue = Queue {
            next: std::ptr::null_mut(),
            prev: std::ptr::null_mut(),
        };
        // Initialize: empty queue points to itself
        empty_queue.next = &mut empty_queue;
        empty_queue.prev = &mut empty_queue;

        unsafe {
            assert_ne!(rs_queue_empty(&empty_queue), 0);
        }

        // Create a non-empty queue (has another node)
        let mut head = Queue {
            next: std::ptr::null_mut(),
            prev: std::ptr::null_mut(),
        };
        let mut node = Queue {
            next: std::ptr::null_mut(),
            prev: std::ptr::null_mut(),
        };

        // Link: head -> node -> head (circular)
        head.next = &mut node;
        head.prev = &mut node;
        node.next = &mut head;
        node.prev = &mut head;

        unsafe {
            assert_eq!(rs_queue_empty(&head), 0);
        }
    }

    #[test]
    fn test_queue_empty_null() {
        // Null queue should be treated as empty
        unsafe {
            assert_ne!(rs_queue_empty(std::ptr::null()), 0);
        }
    }

    #[test]
    fn test_queue_init() {
        let mut queue = Queue {
            next: std::ptr::null_mut(),
            prev: std::ptr::null_mut(),
        };
        let queue_ptr = &mut queue as *mut Queue;

        unsafe {
            rs_queue_init(queue_ptr);
            // After init, queue should point to itself
            assert_eq!((*queue_ptr).next, queue_ptr);
            assert_eq!((*queue_ptr).prev, queue_ptr);
            // Should be empty
            assert_ne!(rs_queue_empty(queue_ptr), 0);
        }
    }

    #[test]
    fn test_queue_insert_head() {
        let mut head = Queue {
            next: std::ptr::null_mut(),
            prev: std::ptr::null_mut(),
        };
        let mut node = Queue {
            next: std::ptr::null_mut(),
            prev: std::ptr::null_mut(),
        };
        let head_ptr = &mut head as *mut Queue;
        let node_ptr = &mut node as *mut Queue;

        unsafe {
            rs_queue_init(head_ptr);
            rs_queue_insert_head(head_ptr, node_ptr);

            // head -> node -> head
            assert_eq!((*head_ptr).next, node_ptr);
            assert_eq!((*head_ptr).prev, node_ptr);
            assert_eq!((*node_ptr).next, head_ptr);
            assert_eq!((*node_ptr).prev, head_ptr);
            // No longer empty
            assert_eq!(rs_queue_empty(head_ptr), 0);
        }
    }

    #[test]
    fn test_queue_insert_tail() {
        let mut head = Queue {
            next: std::ptr::null_mut(),
            prev: std::ptr::null_mut(),
        };
        let mut node1 = Queue {
            next: std::ptr::null_mut(),
            prev: std::ptr::null_mut(),
        };
        let mut node2 = Queue {
            next: std::ptr::null_mut(),
            prev: std::ptr::null_mut(),
        };
        let head_ptr = &mut head as *mut Queue;
        let node1_ptr = &mut node1 as *mut Queue;
        let node2_ptr = &mut node2 as *mut Queue;

        unsafe {
            rs_queue_init(head_ptr);
            rs_queue_insert_tail(head_ptr, node1_ptr);
            rs_queue_insert_tail(head_ptr, node2_ptr);

            // head -> node1 -> node2 -> head
            assert_eq!((*head_ptr).next, node1_ptr);
            assert_eq!((*node1_ptr).next, node2_ptr);
            assert_eq!((*node2_ptr).next, head_ptr);
            assert_eq!((*head_ptr).prev, node2_ptr);
        }
    }

    #[test]
    fn test_queue_remove() {
        let mut head = Queue {
            next: std::ptr::null_mut(),
            prev: std::ptr::null_mut(),
        };
        let mut node = Queue {
            next: std::ptr::null_mut(),
            prev: std::ptr::null_mut(),
        };
        let head_ptr = &mut head as *mut Queue;
        let node_ptr = &mut node as *mut Queue;

        unsafe {
            rs_queue_init(head_ptr);
            rs_queue_insert_head(head_ptr, node_ptr);
            assert_eq!(rs_queue_empty(head_ptr), 0);

            rs_queue_remove(node_ptr);
            // After removing the only node, head should be empty
            assert_ne!(rs_queue_empty(head_ptr), 0);
        }
    }
}
