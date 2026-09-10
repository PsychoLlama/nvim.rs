#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

// Canonical type definitions, hoisted out of the per-module copies c2rust
// emitted. One definition per logical type; every module re-exports here.
use super::*;

/// How far a list-backed reader has got.
///
/// Three cursors into a list the reader does not own: which item, how far
/// into it, and how long it is.
pub struct ListReaderState {
    pub list: *const List,
    pub at: size_t,
    pub offset: size_t,
    pub li_length: size_t,
}
