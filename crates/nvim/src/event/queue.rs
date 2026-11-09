#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]
// libuv's intrusive `queue` node, named for the `QUEUE` macro that walks
// it.
#![allow(non_camel_case_types)]

//! libuv's intrusive queue node.

pub type QUEUE = queue;
/// libuv's intrusive doubly-linked list node.
///
/// **Self-referential.** An empty queue's `next` and `prev` both point at the
/// node itself, and a linked node's neighbours point back at *its* address, so
/// a node is only valid where it was initialised. Not `Copy`.
#[derive(Clone, Debug)]
pub struct queue {
    pub next: *mut queue,
    pub prev: *mut queue,
}
