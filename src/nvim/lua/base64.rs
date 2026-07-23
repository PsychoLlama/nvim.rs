use crate::src::nvim::global_cell::SharedCell;
use crate::src::nvim::lua::ffi::{
    luaL_argerror, luaL_error, luaL_register, lua_createtable, lua_gettop, lua_pushlstring,
    lua_tolstring, lua_type,
};
pub use crate::src::nvim::types::{luaL_Reg, lua_CFunction, lua_State, size_t};
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const LUA_TSTRING: ::core::ffi::c_int = 4 as ::core::ffi::c_int;
unsafe extern "C-unwind" fn nlua_base64_encode(mut L: *mut lua_State) -> ::core::ffi::c_int {
    if lua_gettop(L) < 1 as ::core::ffi::c_int {
        return luaL_error(
            L,
            b"Expected 1 argument\0".as_ptr() as *const ::core::ffi::c_char,
        );
    }
    if lua_type(L, 1 as ::core::ffi::c_int) != LUA_TSTRING {
        luaL_argerror(
            L,
            1 as ::core::ffi::c_int,
            b"expected string\0".as_ptr() as *const ::core::ffi::c_char,
        );
    }
    let mut src_len: size_t = 0 as size_t;
    let mut src: *const ::core::ffi::c_char =
        lua_tolstring(L, 1 as ::core::ffi::c_int, &raw mut src_len);
    let ret =
        crate::src::nvim::base64::encode(::core::slice::from_raw_parts(src as *const u8, src_len));
    lua_pushlstring(L, ret.as_ptr() as *const ::core::ffi::c_char, ret.len());
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C-unwind" fn nlua_base64_decode(mut L: *mut lua_State) -> ::core::ffi::c_int {
    if lua_gettop(L) < 1 as ::core::ffi::c_int {
        return luaL_error(
            L,
            b"Expected 1 argument\0".as_ptr() as *const ::core::ffi::c_char,
        );
    }
    if lua_type(L, 1 as ::core::ffi::c_int) != LUA_TSTRING {
        luaL_argerror(
            L,
            1 as ::core::ffi::c_int,
            b"expected string\0".as_ptr() as *const ::core::ffi::c_char,
        );
    }
    let mut src_len: size_t = 0 as size_t;
    let mut src: *const ::core::ffi::c_char =
        lua_tolstring(L, 1 as ::core::ffi::c_int, &raw mut src_len);
    let Some(ret) =
        crate::src::nvim::base64::decode(::core::slice::from_raw_parts(src as *const u8, src_len))
    else {
        return luaL_error(L, b"Invalid input\0".as_ptr() as *const ::core::ffi::c_char);
    };
    lua_pushlstring(L, ret.as_ptr() as *const ::core::ffi::c_char, ret.len());
    return 1 as ::core::ffi::c_int;
}
static base64_functions: SharedCell<[luaL_Reg; 3]> = SharedCell::new([
    luaL_Reg {
        name: b"encode\0".as_ptr() as *const ::core::ffi::c_char,
        func: Some(
            nlua_base64_encode as unsafe extern "C-unwind" fn(*mut lua_State) -> ::core::ffi::c_int,
        ),
    },
    luaL_Reg {
        name: b"decode\0".as_ptr() as *const ::core::ffi::c_char,
        func: Some(
            nlua_base64_decode as unsafe extern "C-unwind" fn(*mut lua_State) -> ::core::ffi::c_int,
        ),
    },
    luaL_Reg {
        name: ::core::ptr::null::<::core::ffi::c_char>(),
        func: None,
    },
]);
pub unsafe extern "C-unwind" fn luaopen_base64(mut L: *mut lua_State) -> ::core::ffi::c_int {
    lua_createtable(L, 0 as ::core::ffi::c_int, 0 as ::core::ffi::c_int);
    luaL_register(
        L,
        ::core::ptr::null::<::core::ffi::c_char>(),
        (base64_functions.ptr() as *const _) as *const luaL_Reg,
    );
    return 1 as ::core::ffi::c_int;
}
