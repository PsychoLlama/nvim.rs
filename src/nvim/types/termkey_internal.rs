// Canonical type definitions extracted by tools/unify (phase 5a).
// One definition per logical type; every module re-exports from here.
use super::*;

#[derive(Copy, Clone)]
#[repr(C)]
pub struct TermKeyDriver {
    pub name: *const ::core::ffi::c_char,
    pub new_driver:
        Option<unsafe extern "C" fn(*mut TermKey, *mut TerminfoEntry) -> *mut ::core::ffi::c_void>,
    pub free_driver: Option<unsafe extern "C" fn(*mut ::core::ffi::c_void) -> ()>,
    pub start_driver:
        Option<unsafe extern "C" fn(*mut TermKey, *mut ::core::ffi::c_void) -> ::core::ffi::c_int>,
    pub stop_driver:
        Option<unsafe extern "C" fn(*mut TermKey, *mut ::core::ffi::c_void) -> ::core::ffi::c_int>,
    pub peekkey: Option<
        unsafe extern "C" fn(
            *mut TermKey,
            *mut ::core::ffi::c_void,
            *mut TermKeyKey,
            ::core::ffi::c_int,
            *mut size_t,
        ) -> TermKeyResult,
    >,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct TermKeyDriverNode {
    pub driver: *mut TermKeyDriver,
    pub info: *mut ::core::ffi::c_void,
    pub next: *mut TermKeyDriverNode,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct keyinfo {
    pub type_0: TermKeyType,
    pub sym: TermKeySym,
    pub modifier_mask: ::core::ffi::c_int,
    pub modifier_set: ::core::ffi::c_int,
}
pub type ssize_t = isize;
