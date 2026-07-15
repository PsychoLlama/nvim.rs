extern "C" {
    pub type lua_State;
    fn __assert_fail(
        __assertion: *const ::core::ffi::c_char,
        __file: *const ::core::ffi::c_char,
        __line: ::core::ffi::c_uint,
        __function: *const ::core::ffi::c_char,
    ) -> !;
    fn lua_gettop(L: *mut lua_State) -> ::core::ffi::c_int;
    fn lua_type(L: *mut lua_State, idx: ::core::ffi::c_int) -> ::core::ffi::c_int;
    fn lua_tolstring(
        L: *mut lua_State,
        idx: ::core::ffi::c_int,
        len: *mut size_t,
    ) -> *const ::core::ffi::c_char;
    fn lua_pushlstring(L: *mut lua_State, s: *const ::core::ffi::c_char, l: size_t);
    fn lua_pushstring(L: *mut lua_State, s: *const ::core::ffi::c_char);
    fn lua_createtable(
        L: *mut lua_State,
        narr: ::core::ffi::c_int,
        nrec: ::core::ffi::c_int,
    );
    fn luaL_register(
        L: *mut lua_State,
        libname: *const ::core::ffi::c_char,
        l: *const luaL_Reg,
    );
    fn luaL_argerror(
        L: *mut lua_State,
        numarg: ::core::ffi::c_int,
        extramsg: *const ::core::ffi::c_char,
    ) -> ::core::ffi::c_int;
    fn luaL_error(
        L: *mut lua_State,
        fmt: *const ::core::ffi::c_char,
        ...
    ) -> ::core::ffi::c_int;
    fn base64_encode(
        src: *const ::core::ffi::c_char,
        src_len: size_t,
    ) -> *mut ::core::ffi::c_char;
    fn base64_decode(
        src: *const ::core::ffi::c_char,
        src_len: size_t,
        out_lenp: *mut size_t,
    ) -> *mut ::core::ffi::c_char;
    fn xfree(ptr: *mut ::core::ffi::c_void);
}
pub type size_t = usize;
pub type lua_CFunction = Option<
    unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int,
>;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct luaL_Reg {
    pub name: *const ::core::ffi::c_char,
    pub func: lua_CFunction,
}
pub const __ASSERT_FUNCTION: [::core::ffi::c_char; 36] = unsafe {
    ::core::mem::transmute::<
        [u8; 36],
        [::core::ffi::c_char; 36],
    >(*b"int nlua_base64_encode(lua_State *)\0")
};
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<
    ::core::ffi::c_void,
>();
pub const LUA_TSTRING: ::core::ffi::c_int = 4 as ::core::ffi::c_int;
unsafe extern "C" fn nlua_base64_encode(mut L: *mut lua_State) -> ::core::ffi::c_int {
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
    let mut src: *const ::core::ffi::c_char = lua_tolstring(
        L,
        1 as ::core::ffi::c_int,
        &raw mut src_len,
    );
    let mut ret: *const ::core::ffi::c_char = base64_encode(src, src_len);
    '_c2rust_label: {
        if !ret.is_null() {} else {
            __assert_fail(
                b"ret != NULL\0".as_ptr() as *const ::core::ffi::c_char,
                b"/home/overlord/projects/neovim/neovim/src/nvim/lua/base64.c\0".as_ptr()
                    as *const ::core::ffi::c_char,
                26 as ::core::ffi::c_uint,
                __ASSERT_FUNCTION.as_ptr(),
            );
        }
    };
    lua_pushstring(L, ret);
    xfree(ret as *mut ::core::ffi::c_void);
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn nlua_base64_decode(mut L: *mut lua_State) -> ::core::ffi::c_int {
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
    let mut src: *const ::core::ffi::c_char = lua_tolstring(
        L,
        1 as ::core::ffi::c_int,
        &raw mut src_len,
    );
    let mut out_len: size_t = 0 as size_t;
    let mut ret: *const ::core::ffi::c_char = base64_decode(
        src,
        src_len,
        &raw mut out_len,
    );
    if ret.is_null() {
        return luaL_error(L, b"Invalid input\0".as_ptr() as *const ::core::ffi::c_char);
    }
    lua_pushlstring(L, ret, out_len);
    xfree(ret as *mut ::core::ffi::c_void);
    return 1 as ::core::ffi::c_int;
}
static mut base64_functions: [luaL_Reg; 3] = [
    luaL_Reg {
        name: b"encode\0".as_ptr() as *const ::core::ffi::c_char,
        func: Some(
            nlua_base64_encode
                as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int,
        ),
    },
    luaL_Reg {
        name: b"decode\0".as_ptr() as *const ::core::ffi::c_char,
        func: Some(
            nlua_base64_decode
                as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int,
        ),
    },
    luaL_Reg {
        name: ::core::ptr::null::<::core::ffi::c_char>(),
        func: None,
    },
];
#[no_mangle]
pub unsafe extern "C" fn luaopen_base64(mut L: *mut lua_State) -> ::core::ffi::c_int {
    lua_createtable(L, 0 as ::core::ffi::c_int, 0 as ::core::ffi::c_int);
    luaL_register(
        L,
        ::core::ptr::null::<::core::ffi::c_char>(),
        &raw const base64_functions as *const luaL_Reg,
    );
    return 1 as ::core::ffi::c_int;
}
