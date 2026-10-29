#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]
// LuaJIT's `lua_CFunction`, `lua_Integer` and `lua_Number`.
#![allow(non_camel_case_types)]
// And `lua_Debug`'s `L`, which is LuaJIT's field name.
#![allow(non_snake_case)]

// Canonical type definitions, hoisted out of the per-module copies c2rust
// emitted. One definition per logical type; every module re-exports here.
use super::*;

// Opaque C type: layout unknown here, only ever used behind a pointer.
#[repr(C)]
pub struct lua_State {
    _data: [u8; 0],
    _marker: ::core::marker::PhantomData<(*mut u8, ::core::marker::PhantomPinned)>,
}

pub type lua_CFunction = Option<unsafe extern "C-unwind" fn(*mut lua_State) -> ::core::ffi::c_int>;
pub type lua_Integer = ptrdiff_t;
pub type lua_Number = ::core::ffi::c_double;
