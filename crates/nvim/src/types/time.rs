#![forbid(unsafe_code)]

// Canonical type definitions, hoisted out of the per-module copies c2rust
// emitted. One definition per logical type; every module re-exports here.
use crate::os::libc::uint64_t;

pub type Timestamp = uint64_t;
