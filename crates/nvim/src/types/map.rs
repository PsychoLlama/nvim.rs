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

pub type MHPutStatus = ::core::ffi::c_uint;
/// The bucket table every khash-derived map and set is built on.
///
/// Not `Copy`: `hash` is the table's own allocation, freed by `mh_clear` /
/// `map_destroy`. Two copies of one of these is two owners of that table and
/// of whichever `keys`/`values` arrays the set or map around it holds.
#[derive(Clone)]
#[repr(C)]
pub struct MapHash {
    pub n_buckets: uint32_t,
    pub size: uint32_t,
    pub n_occupied: uint32_t,
    pub upper_bound: uint32_t,
    pub n_keys: uint32_t,
    pub keys_capacity: uint32_t,
    pub hash: *mut uint32_t,
}
impl MapHash {
    /// A table with nothing in it and nothing allocated. Every map and set
    /// starts here; the first insert allocates.
    pub const EMPTY: Self = Self {
        n_buckets: 0,
        size: 0,
        n_occupied: 0,
        upper_bound: 0,
        n_keys: 0,
        keys_capacity: 0,
        hash: ::core::ptr::null_mut(),
    };
}

impl Set_cstr_t {
    /// An empty set of C strings. See [`MapHash::EMPTY`].
    pub const EMPTY: Self = Self {
        h: MapHash::EMPTY,
        keys: ::core::ptr::null_mut(),
    };
}

pub type cstr_t = *const ::core::ffi::c_char;
pub type ptr_t = *mut ::core::ffi::c_void;
