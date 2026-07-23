// Canonical type definitions extracted by tools/unify (phase 5a).
// One definition per logical type; every module re-exports from here.
use super::*;

extern "C" {
    pub type lua_State;
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct luaL_Buffer {
    pub p: *mut ::core::ffi::c_char,
    pub lvl: ::core::ffi::c_int,
    pub L: *mut lua_State,
    pub buffer: [::core::ffi::c_char; 8192],
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct luaL_Reg {
    pub name: *const ::core::ffi::c_char,
    pub func: lua_CFunction,
}
pub type lua_CFunction = Option<unsafe extern "C-unwind" fn(*mut lua_State) -> ::core::ffi::c_int>;
pub type lua_Integer = ptrdiff_t;
pub type lua_Number = ::core::ffi::c_double;
