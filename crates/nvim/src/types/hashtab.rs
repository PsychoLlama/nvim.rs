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

pub type hash_T = size_t;
/// One slot of a [`hashtab_T`].
///
/// `Copy`: `hi_key` points into the `dictitem_T` (or equivalent) that the
/// table indexes, which the table does not own.
#[derive(Copy, Clone)]
pub struct hashitem_T {
    pub hi_hash: hash_T,
    pub hi_key: *mut ::core::ffi::c_char,
}
/// Vim's open-addressed hash table.
///
/// **Self-referential.** While the table fits its inline array, `ht_array`
/// points at this struct's own `ht_smallarray`, so a table is only valid at
/// the address it was initialised at. Not `Copy` for that reason, and even a
/// clone is only sound when the value ends up back at the address it came
/// from -- which is exactly what `get_v_event`/`restore_v_event` do, and they
/// say so.
#[derive(Clone)]
pub struct hashtab_T {
    pub ht_mask: hash_T,
    pub ht_used: size_t,
    pub ht_filled: size_t,
    pub ht_changed: ::core::ffi::c_int,
    pub ht_locked: ::core::ffi::c_int,
    pub ht_array: *mut hashitem_T,
    pub ht_smallarray: [hashitem_T; 16],
}

impl Default for hashitem_T {
    fn default() -> Self {
        Self {
            hi_hash: 0,
            hi_key: ::core::ptr::null_mut(),
        }
    }
}

impl Default for hashtab_T {
    fn default() -> Self {
        Self {
            ht_mask: 0,
            ht_used: 0,
            ht_filled: 0,
            ht_changed: 0,
            ht_locked: 0,
            ht_array: ::core::ptr::null_mut(),
            ht_smallarray: [hashitem_T::default(); 16],
        }
    }
}
