#![forbid(unsafe_code)]

// Canonical type definitions, hoisted out of the per-module copies c2rust
// emitted. One definition per logical type; every module re-exports here.

// Opaque C type: layout unknown here, only ever used behind a pointer.
#[repr(C)]
pub struct qf_info_S {
    _data: [u8; 0],
    _marker: ::core::marker::PhantomData<(*mut u8, ::core::marker::PhantomPinned)>,
}
