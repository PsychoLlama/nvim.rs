// Canonical type definitions extracted by tools/unify (phase 5a).
// One definition per logical type; every module re-exports from here.
use super::*;

pub type CheckItem = Option<
    unsafe extern "C" fn(*mut ::core::ffi::c_void, *const ::core::ffi::c_char) -> varnumber_T,
>;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct FileDescriptor {
    pub fd: ::core::ffi::c_int,
    pub buffer: *mut ::core::ffi::c_char,
    pub read_pos: *mut ::core::ffi::c_char,
    pub write_pos: *mut ::core::ffi::c_char,
    pub wr: bool,
    pub eof: bool,
    pub non_blocking: bool,
    pub bytes_read: uint64_t,
}
