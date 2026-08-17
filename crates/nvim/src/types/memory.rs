#![forbid(unsafe_code)]

// Canonical type definitions, hoisted out of the per-module copies c2rust
// emitted. One definition per logical type; every module re-exports here.

#[derive(Copy, Clone)]
pub struct consumed_blk {
    pub prev: *mut consumed_blk,
}
