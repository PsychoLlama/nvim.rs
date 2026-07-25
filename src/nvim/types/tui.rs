#![forbid(unsafe_code)]

// Canonical type definitions extracted by tools/unify (phase 5a).
// One definition per logical type; every module re-exports from here.

// Opaque C type: layout unknown here, only ever used behind a pointer.
#[repr(C)]
pub struct TUIData {
    _data: [u8; 0],
    _marker: ::core::marker::PhantomData<(*mut u8, ::core::marker::PhantomPinned)>,
}

pub type TermMode = ::core::ffi::c_uint;
pub type TermModeState = ::core::ffi::c_uint;
