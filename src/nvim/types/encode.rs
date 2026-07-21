// Canonical type definitions extracted by tools/unify (phase 5a).
// One definition per logical type; every module re-exports from here.
use super::*;

#[derive(Copy, Clone)]
#[repr(C)]
pub struct ListReaderState {
    pub list: *const list_T,
    pub li: *const listitem_T,
    pub offset: size_t,
    pub li_length: size_t,
}
