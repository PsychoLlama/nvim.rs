#![forbid(unsafe_code)]

// Canonical type definitions, hoisted out of the per-module copies c2rust
// emitted. One definition per logical type; every module re-exports here.
use super::*;

#[derive(Copy, Clone)]
#[repr(C)]
pub struct ListReaderState {
    pub list: *const list_T,
    pub li: *const listitem_T,
    pub offset: size_t,
    pub li_length: size_t,
}
