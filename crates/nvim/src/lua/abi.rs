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

//! LuaJIT's C API types: the interpreter state every binding is handed, and
//! the scalars it passes across.

use crate::os::libc::ptrdiff_t;

// Opaque C type: layout unknown here, only ever used behind a pointer.
#[repr(C)]
#[derive(Debug)]
pub struct lua_State {
    _data: [u8; 0],
    _marker: ::core::marker::PhantomData<(*mut u8, ::core::marker::PhantomPinned)>,
}

pub type lua_CFunction = Option<unsafe extern "C-unwind" fn(*mut lua_State) -> ::core::ffi::c_int>;
pub type lua_Integer = ptrdiff_t;
pub type lua_Number = ::core::ffi::c_double;
