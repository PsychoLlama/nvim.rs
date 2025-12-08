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
}
