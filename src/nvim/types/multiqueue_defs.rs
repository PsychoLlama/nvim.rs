// Canonical type definitions extracted by tools/unify (phase 5a).
// One definition per logical type; every module re-exports from here.
use super::*;

#[derive(Copy, Clone)]
#[repr(C)]
pub struct multiqueue {
    pub parent: *mut MultiQueue,
    pub headtail: QUEUE,
    pub on_put: PutCallback,
    pub data: *mut ::core::ffi::c_void,
    pub size: size_t,
}
