// Canonical type definitions extracted by tools/unify (phase 5a).
// One definition per logical type; every module re-exports from here.
use super::*;

pub type chanode_t = s_chanode;
pub type chastore_t = s_chastore;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct s_chanode {
    pub next: *mut s_chanode,
    pub icurr: ::core::ffi::c_long,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct s_chastore {
    pub head: *mut chanode_t,
    pub tail: *mut chanode_t,
    pub isize: ::core::ffi::c_long,
    pub nsize: ::core::ffi::c_long,
    pub ancur: *mut chanode_t,
    pub sncur: *mut chanode_t,
    pub scurr: ::core::ffi::c_long,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct s_xdfenv {
    pub xdf1: xdfile_t,
    pub xdf2: xdfile_t,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct s_xdfile {
    pub rcha: chastore_t,
    pub nrec: ::core::ffi::c_long,
    pub hbits: ::core::ffi::c_uint,
    pub rhash: *mut *mut xrecord_t,
    pub dstart: ::core::ffi::c_long,
    pub dend: ::core::ffi::c_long,
    pub recs: *mut *mut xrecord_t,
    pub rchg: *mut ::core::ffi::c_char,
    pub rindex: *mut ::core::ffi::c_long,
    pub nreff: ::core::ffi::c_long,
    pub ha: *mut ::core::ffi::c_ulong,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct s_xrecord {
    pub next: *mut s_xrecord,
    pub ptr: *const ::core::ffi::c_char,
    pub size: ::core::ffi::c_long,
    pub ha: ::core::ffi::c_ulong,
}
pub type xdfenv_t = s_xdfenv;
pub type xdfile_t = s_xdfile;
pub type xrecord_t = s_xrecord;
