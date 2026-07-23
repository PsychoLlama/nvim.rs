use crate::src::mpack::conv::{
    mpack_pack_array, mpack_pack_bin, mpack_pack_boolean, mpack_pack_chunk, mpack_pack_ext,
    mpack_pack_map, mpack_pack_nil, mpack_pack_number, mpack_pack_str, mpack_unpack_boolean,
    mpack_unpack_number,
};
use crate::src::mpack::object::{mpack_parse, mpack_parser_copy, mpack_parser_init, mpack_unparse};
use crate::src::mpack::rpc::{
    mpack_rpc_notify, mpack_rpc_receive, mpack_rpc_reply, mpack_rpc_request,
    mpack_rpc_session_copy, mpack_rpc_session_init,
};
use crate::src::nvim::global_cell::SharedCell;
use crate::src::nvim::lua::ffi::{
    luaL_argerror, luaL_buffinit, luaL_checklstring, luaL_checknumber, luaL_checkudata, luaL_error,
    luaL_newmetatable, luaL_prepbuffer, luaL_pushresult, luaL_ref, luaL_register, luaL_unref,
    lua_call, lua_createtable, lua_getfield, lua_getmetatable, lua_gettable, lua_gettop,
    lua_insert, lua_isnumber, lua_isstring, lua_isuserdata, lua_newuserdata, lua_next, lua_objlen,
    lua_pushboolean, lua_pushcclosure, lua_pushfstring, lua_pushinteger, lua_pushlstring,
    lua_pushnil, lua_pushnumber, lua_pushstring, lua_pushvalue, lua_rawequal, lua_rawgeti,
    lua_remove, lua_replace, lua_setfield, lua_setmetatable, lua_settable, lua_settop,
    lua_toboolean, lua_tolstring, lua_tonumber, lua_topointer, lua_type,
};
use crate::src::nvim::os::libc::{__assert_fail, free, malloc, memcpy, snprintf};
pub use crate::src::nvim::types::{
    luaL_Buffer, luaL_Reg, lua_CFunction, lua_Integer, lua_Number, lua_State, mpack_data_t,
    mpack_node_s, mpack_node_t, mpack_one_parser_t, mpack_parser_t, mpack_rpc_header_s,
    mpack_rpc_header_t, mpack_rpc_message_s, mpack_rpc_message_t, mpack_rpc_one_session_t,
    mpack_rpc_session_t, mpack_rpc_slot_s, mpack_sintmax_t, mpack_tokbuf_s, mpack_tokbuf_t,
    mpack_token_s, mpack_token_s_data as C2Rust_Unnamed, mpack_token_t, mpack_token_type_t,
    mpack_uint32_t, mpack_uintmax_t, mpack_value_s, mpack_value_t, mpack_walk_cb, ptrdiff_t,
    size_t,
};
#[derive(Copy, Clone)]
#[repr(C)]
pub struct Packer {
    pub L: *mut lua_State,
    pub parser: *mut mpack_parser_t,
    pub reg: ::core::ffi::c_int,
    pub ext: ::core::ffi::c_int,
    pub root: ::core::ffi::c_int,
    pub packing: ::core::ffi::c_int,
    pub mtdict: ::core::ffi::c_int,
    pub is_bin: ::core::ffi::c_int,
    pub is_bin_fn: ::core::ffi::c_int,
}
pub const MPACK_TOKEN_EXT: mpack_token_type_t = 11;
pub const MPACK_TOKEN_STR: mpack_token_type_t = 10;
pub const MPACK_TOKEN_BIN: mpack_token_type_t = 9;
pub const MPACK_TOKEN_MAP: mpack_token_type_t = 8;
pub const MPACK_TOKEN_ARRAY: mpack_token_type_t = 7;
pub const MPACK_TOKEN_CHUNK: mpack_token_type_t = 6;
pub const MPACK_TOKEN_FLOAT: mpack_token_type_t = 5;
pub const MPACK_TOKEN_SINT: mpack_token_type_t = 4;
pub const MPACK_TOKEN_UINT: mpack_token_type_t = 3;
pub const MPACK_TOKEN_BOOLEAN: mpack_token_type_t = 2;
pub const MPACK_TOKEN_NIL: mpack_token_type_t = 1;
pub const MPACK_EOF: C2Rust_Unnamed_1 = 1;
pub const MPACK_NOMEM: C2Rust_Unnamed_2 = 3;
pub const MPACK_OK: C2Rust_Unnamed_1 = 0;
pub const MPACK_ERROR: C2Rust_Unnamed_1 = 2;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct Unpacker {
    pub L: *mut lua_State,
    pub parser: *mut mpack_parser_t,
    pub reg: ::core::ffi::c_int,
    pub ext: ::core::ffi::c_int,
    pub unpacking: ::core::ffi::c_int,
    pub mtdict: ::core::ffi::c_int,
    pub string_buffer: *mut ::core::ffi::c_char,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct Session {
    pub L: *mut lua_State,
    pub reg: ::core::ffi::c_int,
    pub session: *mut mpack_rpc_session_t,
    pub unpacked: C2Rust_Unnamed_0,
    pub unpacker: ::core::ffi::c_int,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2Rust_Unnamed_0 {
    pub type_0: ::core::ffi::c_int,
    pub msg: mpack_rpc_message_t,
    pub method_or_error: ::core::ffi::c_int,
    pub args_or_result: ::core::ffi::c_int,
}
pub const MPACK_RPC_NOTIFICATION: C2Rust_Unnamed_3 = 6;
pub const MPACK_RPC_RESPONSE: C2Rust_Unnamed_3 = 5;
pub const MPACK_RPC_REQUEST: C2Rust_Unnamed_3 = 4;
pub type C2Rust_Unnamed_1 = ::core::ffi::c_uint;
pub type C2Rust_Unnamed_2 = ::core::ffi::c_int;
pub const MPACK_EXCEPTION: C2Rust_Unnamed_2 = -1;
pub type C2Rust_Unnamed_3 = ::core::ffi::c_uint;
pub const MPACK_RPC_ERROR: C2Rust_Unnamed_3 = 7;
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const LUAL_BUFFERSIZE: ::core::ffi::c_int = if BUFSIZ > 16384 as ::core::ffi::c_int {
    8192 as ::core::ffi::c_int
} else {
    BUFSIZ
};
pub const BUFSIZ: ::core::ffi::c_int = 8192 as ::core::ffi::c_int;
pub const LUA_REGISTRYINDEX: ::core::ffi::c_int = -10000 as ::core::ffi::c_int;
pub const LUA_TNIL: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const LUA_TBOOLEAN: ::core::ffi::c_int = 1;
pub const LUA_TNUMBER: ::core::ffi::c_int = 3 as ::core::ffi::c_int;
pub const LUA_TSTRING: ::core::ffi::c_int = 4;
pub const LUA_TTABLE: ::core::ffi::c_int = 5 as ::core::ffi::c_int;
pub const LUA_TFUNCTION: ::core::ffi::c_int = 6 as ::core::ffi::c_int;
pub const LUA_TUSERDATA: ::core::ffi::c_int = 7;
pub const false_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const LUA_NOREF: ::core::ffi::c_int = -2 as ::core::ffi::c_int;
pub const LUA_REFNIL: ::core::ffi::c_int = -1 as ::core::ffi::c_int;
pub const UNPACKER_META_NAME: [::core::ffi::c_char; 15] =
    unsafe { ::core::mem::transmute::<[u8; 15], [::core::ffi::c_char; 15]>(*b"mpack.Unpacker\0") };
pub const UNPACK_FN_NAME: [::core::ffi::c_char; 7] =
    unsafe { ::core::mem::transmute::<[u8; 7], [::core::ffi::c_char; 7]>(*b"decode\0") };
pub const PACKER_META_NAME: [::core::ffi::c_char; 13] =
    unsafe { ::core::mem::transmute::<[u8; 13], [::core::ffi::c_char; 13]>(*b"mpack.Packer\0") };
pub const PACK_FN_NAME: [::core::ffi::c_char; 7] =
    unsafe { ::core::mem::transmute::<[u8; 7], [::core::ffi::c_char; 7]>(*b"encode\0") };
pub const SESSION_META_NAME: [::core::ffi::c_char; 14] =
    unsafe { ::core::mem::transmute::<[u8; 14], [::core::ffi::c_char; 14]>(*b"mpack.Session\0") };
pub const NIL_NAME: [::core::ffi::c_char; 10] =
    unsafe { ::core::mem::transmute::<[u8; 10], [::core::ffi::c_char; 10]>(*b"mpack.NIL\0") };
pub const EMPTY_DICT_NAME: [::core::ffi::c_char; 17] = unsafe {
    ::core::mem::transmute::<[u8; 17], [::core::ffi::c_char; 17]>(*b"mpack.empty_dict\0")
};
unsafe extern "C-unwind" fn lmpack_ref(
    mut L: *mut lua_State,
    mut reg: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let mut rv: ::core::ffi::c_int = 0;
    lua_rawgeti(L, LUA_REGISTRYINDEX, reg);
    lua_pushvalue(L, -2 as ::core::ffi::c_int);
    rv = luaL_ref(L, -2 as ::core::ffi::c_int);
    lua_settop(L, -2 as ::core::ffi::c_int - 1 as ::core::ffi::c_int);
    return rv;
}
unsafe extern "C-unwind" fn lmpack_unref(
    mut L: *mut lua_State,
    mut reg: ::core::ffi::c_int,
    mut ref_0: ::core::ffi::c_int,
) {
    lua_rawgeti(L, LUA_REGISTRYINDEX, reg);
    luaL_unref(L, -1 as ::core::ffi::c_int, ref_0);
    lua_settop(L, -1 as ::core::ffi::c_int - 1 as ::core::ffi::c_int);
}
unsafe extern "C-unwind" fn lmpack_geti(
    mut L: *mut lua_State,
    mut reg: ::core::ffi::c_int,
    mut ref_0: ::core::ffi::c_int,
) {
    lua_rawgeti(L, LUA_REGISTRYINDEX, reg);
    lua_rawgeti(L, -1 as ::core::ffi::c_int, ref_0);
    lua_replace(L, -2 as ::core::ffi::c_int);
}
unsafe extern "C-unwind" fn lmpack_shallow_copy(mut L: *mut lua_State) {
    lua_createtable(L, 0 as ::core::ffi::c_int, 0 as ::core::ffi::c_int);
    lua_pushnil(L);
    while lua_next(L, -3 as ::core::ffi::c_int) != 0 {
        lua_pushvalue(L, -2 as ::core::ffi::c_int);
        lua_insert(L, -2 as ::core::ffi::c_int);
        lua_settable(L, -4 as ::core::ffi::c_int);
    }
    lua_remove(L, -2 as ::core::ffi::c_int);
}
unsafe extern "C-unwind" fn lmpack_grow_parser(
    mut parser: *mut mpack_parser_t,
) -> *mut mpack_parser_t {
    let mut old: *mut mpack_parser_t = parser;
    let mut new_capacity: mpack_uint32_t = (*old).capacity.wrapping_mul(2 as mpack_uint32_t);
    parser = malloc(
        ::core::mem::size_of::<mpack_node_t>()
            .wrapping_mul(new_capacity as size_t)
            .wrapping_add(::core::mem::size_of::<mpack_one_parser_t>()),
    ) as *mut mpack_parser_t;
    if !parser.is_null() {
        mpack_parser_init(parser, new_capacity);
        mpack_parser_copy(parser, old);
        free(old as *mut ::core::ffi::c_void);
    }
    return parser;
}
unsafe extern "C-unwind" fn lmpack_grow_session(
    mut session: *mut mpack_rpc_session_t,
) -> *mut mpack_rpc_session_t {
    let mut old: *mut mpack_rpc_session_t = session;
    let mut new_capacity: mpack_uint32_t = (*old).capacity.wrapping_mul(2 as mpack_uint32_t);
    session = malloc(
        ::core::mem::size_of::<mpack_rpc_slot_s>()
            .wrapping_mul(new_capacity.wrapping_sub(1 as mpack_uint32_t) as size_t)
            .wrapping_add(::core::mem::size_of::<mpack_rpc_one_session_t>()),
    ) as *mut mpack_rpc_session_t;
    if !session.is_null() {
        mpack_rpc_session_init(session, new_capacity);
        mpack_rpc_session_copy(session, old);
        free(old as *mut ::core::ffi::c_void);
    }
    return session;
}
unsafe extern "C-unwind" fn lmpack_check_unpacker(
    mut L: *mut lua_State,
    mut index: ::core::ffi::c_int,
) -> *mut Unpacker {
    return luaL_checkudata(L, index, UNPACKER_META_NAME.as_ptr()) as *mut Unpacker;
}
unsafe extern "C-unwind" fn lmpack_check_packer(
    mut L: *mut lua_State,
    mut index: ::core::ffi::c_int,
) -> *mut Packer {
    return luaL_checkudata(L, index, PACKER_META_NAME.as_ptr()) as *mut Packer;
}
unsafe extern "C-unwind" fn lmpack_check_session(
    mut L: *mut lua_State,
    mut index: ::core::ffi::c_int,
) -> *mut Session {
    return luaL_checkudata(L, index, SESSION_META_NAME.as_ptr()) as *mut Session;
}
unsafe extern "C-unwind" fn lmpack_isnil(
    mut L: *mut lua_State,
    mut index: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let mut rv: ::core::ffi::c_int = 0;
    if lua_isuserdata(L, index) == 0 {
        return 0 as ::core::ffi::c_int;
    }
    lua_getfield(L, LUA_REGISTRYINDEX, NIL_NAME.as_ptr());
    rv = lua_rawequal(L, -1 as ::core::ffi::c_int, -2 as ::core::ffi::c_int);
    lua_settop(L, -1 as ::core::ffi::c_int - 1 as ::core::ffi::c_int);
    return rv;
}
unsafe extern "C-unwind" fn lmpack_isunpacker(
    mut L: *mut lua_State,
    mut index: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let mut rv: ::core::ffi::c_int = 0;
    if lua_isuserdata(L, index) == 0 || lua_getmetatable(L, index) == 0 {
        return 0 as ::core::ffi::c_int;
    }
    lua_getfield(
        L,
        LUA_REGISTRYINDEX,
        b"mpack.Unpacker\0".as_ptr() as *const ::core::ffi::c_char,
    );
    rv = lua_rawequal(L, -1 as ::core::ffi::c_int, -2 as ::core::ffi::c_int);
    lua_settop(L, -2 as ::core::ffi::c_int - 1 as ::core::ffi::c_int);
    return rv;
}
unsafe extern "C-unwind" fn lmpack_pushnil(mut L: *mut lua_State) {
    lua_getfield(L, LUA_REGISTRYINDEX, NIL_NAME.as_ptr());
}
unsafe extern "C-unwind" fn lmpack_objlen(
    mut L: *mut lua_State,
    mut is_array: *mut ::core::ffi::c_int,
) -> mpack_uint32_t {
    let mut len: size_t = 0;
    let mut max: size_t = 0;
    let mut isarr: ::core::ffi::c_int = 0;
    let mut n: lua_Number = 0.;
    let mut top: ::core::ffi::c_int = lua_gettop(L);
    '_c2rust_label: {
        if top != 0 {
        } else {
            __assert_fail(
                b"top\0".as_ptr() as *const ::core::ffi::c_char,
                b"src/mpack/lmpack.rs\0".as_ptr() as *const ::core::ffi::c_char,
                210 as ::core::ffi::c_uint,
                b"mpack_uint32_t lmpack_objlen(lua_State *, int *)\0".as_ptr()
                    as *const ::core::ffi::c_char,
            );
        }
    };
    if lua_type(L, -1 as ::core::ffi::c_int) != LUA_TTABLE {
        len = lua_objlen(L, -1 as ::core::ffi::c_int);
    } else {
        len = 0 as size_t;
        max = 0 as size_t;
        isarr = 1 as ::core::ffi::c_int;
        lua_pushnil(L);
        while lua_next(L, -2 as ::core::ffi::c_int) != 0 {
            lua_settop(L, -1 as ::core::ffi::c_int - 1 as ::core::ffi::c_int);
            isarr = (isarr != 0
                && lua_type(L, -1 as ::core::ffi::c_int) == LUA_TNUMBER
                && {
                    n = lua_tonumber(L, -1 as ::core::ffi::c_int);
                    n > 0 as ::core::ffi::c_int as lua_Number
                }
                && n as size_t as lua_Number == n) as ::core::ffi::c_int;
            max = if isarr != 0 && n as size_t > max {
                n as size_t
            } else {
                max
            };
            len = len.wrapping_add(1);
        }
        if len > 0 as size_t {
            *is_array = (isarr != 0 && max == len) as ::core::ffi::c_int;
        }
    }
    if -1 as ::core::ffi::c_int as size_t > -1 as ::core::ffi::c_int as mpack_uint32_t as size_t
        && len > -1 as ::core::ffi::c_int as mpack_uint32_t as size_t
    {
        len = -1 as ::core::ffi::c_int as mpack_uint32_t as size_t;
    }
    '_c2rust_label_0: {
        if top == lua_gettop(L) {
        } else {
            __assert_fail(
                b"top == lua_gettop(L)\0".as_ptr() as *const ::core::ffi::c_char,
                b"src/mpack/lmpack.rs\0".as_ptr() as *const ::core::ffi::c_char,
                249 as ::core::ffi::c_uint,
                b"mpack_uint32_t lmpack_objlen(lua_State *, int *)\0".as_ptr()
                    as *const ::core::ffi::c_char,
            );
        }
    };
    return len as mpack_uint32_t;
}
unsafe extern "C-unwind" fn lmpack_unpacker_new(mut L: *mut lua_State) -> ::core::ffi::c_int {
    let mut rv: *mut Unpacker = ::core::ptr::null_mut::<Unpacker>();
    if lua_gettop(L) > 1 as ::core::ffi::c_int {
        return luaL_error(
            L,
            b"expecting at most 1 table argument\0".as_ptr() as *const ::core::ffi::c_char,
        );
    }
    rv = lua_newuserdata(L, ::core::mem::size_of::<Unpacker>()) as *mut Unpacker;
    (*rv).parser = malloc(::core::mem::size_of::<mpack_parser_t>()) as *mut mpack_parser_t;
    if (*rv).parser.is_null() {
        return luaL_error(
            L,
            b"Failed to allocate memory\0".as_ptr() as *const ::core::ffi::c_char,
        );
    }
    mpack_parser_init((*rv).parser, 0 as mpack_uint32_t);
    (*(*rv).parser).data.p = rv as *mut ::core::ffi::c_void;
    (*rv).string_buffer = ::core::ptr::null_mut::<::core::ffi::c_char>();
    (*rv).L = L;
    (*rv).unpacking = 0 as ::core::ffi::c_int;
    lua_getfield(
        L,
        LUA_REGISTRYINDEX,
        b"mpack.Unpacker\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_setmetatable(L, -2 as ::core::ffi::c_int);
    lua_createtable(L, 0 as ::core::ffi::c_int, 0 as ::core::ffi::c_int);
    (*rv).reg = luaL_ref(L, LUA_REGISTRYINDEX);
    (*rv).ext = LUA_NOREF;
    lua_getfield(L, LUA_REGISTRYINDEX, EMPTY_DICT_NAME.as_ptr());
    (*rv).mtdict = lmpack_ref(L, (*rv).reg);
    if lua_type(L, 1 as ::core::ffi::c_int) == LUA_TTABLE {
        lua_getfield(
            L,
            1 as ::core::ffi::c_int,
            b"ext\0".as_ptr() as *const ::core::ffi::c_char,
        );
        if !(lua_type(L, -1 as ::core::ffi::c_int) == LUA_TNIL) {
            if !(lua_type(L, -1 as ::core::ffi::c_int) == LUA_TTABLE) {
                return luaL_error(
                    L,
                    b"\"ext\" option must be a table\0".as_ptr() as *const ::core::ffi::c_char,
                );
            }
            lmpack_shallow_copy(L);
        }
        (*rv).ext = lmpack_ref(L, (*rv).reg);
    }
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C-unwind" fn lmpack_unpacker_delete(mut L: *mut lua_State) -> ::core::ffi::c_int {
    let mut unpacker: *mut Unpacker = lmpack_check_unpacker(L, 1 as ::core::ffi::c_int);
    if (*unpacker).ext != LUA_NOREF {
        lmpack_unref(L, (*unpacker).reg, (*unpacker).ext);
    }
    luaL_unref(L, LUA_REGISTRYINDEX, (*unpacker).reg);
    free((*unpacker).parser as *mut ::core::ffi::c_void);
    return 0 as ::core::ffi::c_int;
}
unsafe extern "C-unwind" fn lmpack_parse_enter(
    mut parser: *mut mpack_parser_t,
    mut node: *mut mpack_node_t,
) {
    let mut unpacker: *mut Unpacker = (*parser).data.p as *mut Unpacker;
    let mut L: *mut lua_State = (*unpacker).L;
    match (*node).tok.type_0 as ::core::ffi::c_uint {
        1 => {
            lmpack_pushnil(L);
        }
        2 => {
            lua_pushboolean(L, mpack_unpack_boolean((*node).tok) as ::core::ffi::c_int);
        }
        3 | 4 | 5 => {
            lua_pushnumber(L, mpack_unpack_number((*node).tok));
        }
        6 => {
            '_c2rust_label: {
                if !(*unpacker).string_buffer.is_null() {
                } else {
                    __assert_fail(
                        b"unpacker->string_buffer\0".as_ptr() as *const ::core::ffi::c_char,
                        b"src/mpack/lmpack.rs\0".as_ptr() as *const ::core::ffi::c_char,
                        321 as ::core::ffi::c_uint,
                        b"void lmpack_parse_enter(mpack_parser_t *, mpack_node_t *)\0".as_ptr()
                            as *const ::core::ffi::c_char,
                    );
                }
            };
            memcpy(
                (*unpacker).string_buffer.offset(
                    (*(if (*node.offset(-(1 as ::core::ffi::c_int as isize))).pos
                        == -1 as ::core::ffi::c_int as size_t
                    {
                        ::core::ptr::null_mut::<mpack_node_t>()
                    } else {
                        node.offset(-(1 as ::core::ffi::c_int as isize))
                    }))
                    .pos as isize,
                ) as *mut ::core::ffi::c_void,
                (*node).tok.data.chunk_ptr as *const ::core::ffi::c_void,
                (*node).tok.length as size_t,
            );
        }
        9 | 10 | 11 => {
            (*unpacker).string_buffer =
                malloc((*node).tok.length as size_t) as *mut ::core::ffi::c_char;
            if (*unpacker).string_buffer.is_null() {
                luaL_error(
                    L,
                    b"Failed to allocate memory\0".as_ptr() as *const ::core::ffi::c_char,
                );
            }
        }
        7 | 8 => {
            lua_createtable(L, 0 as ::core::ffi::c_int, 0 as ::core::ffi::c_int);
            (*node).data[0 as ::core::ffi::c_int as usize].i =
                lmpack_ref(L, (*unpacker).reg) as mpack_sintmax_t;
        }
        _ => {}
    };
}
unsafe extern "C-unwind" fn lmpack_parse_exit(
    mut parser: *mut mpack_parser_t,
    mut node: *mut mpack_node_t,
) {
    let mut unpacker: *mut Unpacker = (*parser).data.p as *mut Unpacker;
    let mut L: *mut lua_State = (*unpacker).L;
    let mut parent: *mut mpack_node_t = if (*node.offset(-(1 as ::core::ffi::c_int as isize))).pos
        == -1 as ::core::ffi::c_int as size_t
    {
        ::core::ptr::null_mut::<mpack_node_t>()
    } else {
        node.offset(-(1 as ::core::ffi::c_int as isize))
    };
    match (*node).tok.type_0 as ::core::ffi::c_uint {
        9 | 10 | 11 => {
            lua_pushlstring(L, (*unpacker).string_buffer, (*node).tok.length as size_t);
            free((*unpacker).string_buffer as *mut ::core::ffi::c_void);
            (*unpacker).string_buffer = ::core::ptr::null_mut::<::core::ffi::c_char>();
            if (*node).tok.type_0 as ::core::ffi::c_uint
                == MPACK_TOKEN_EXT as ::core::ffi::c_int as ::core::ffi::c_uint
                && (*unpacker).ext != LUA_NOREF
            {
                lmpack_geti(L, (*unpacker).reg, (*unpacker).ext);
                lua_rawgeti(L, -1 as ::core::ffi::c_int, (*node).tok.data.ext_type);
                if lua_type(L, -1 as ::core::ffi::c_int) == LUA_TFUNCTION {
                    lua_pushinteger(L, (*node).tok.data.ext_type as lua_Integer);
                    lua_pushvalue(L, -4 as ::core::ffi::c_int);
                    lua_call(L, 2 as ::core::ffi::c_int, 1 as ::core::ffi::c_int);
                    lua_replace(L, -3 as ::core::ffi::c_int);
                } else {
                    lua_settop(L, -1 as ::core::ffi::c_int - 1 as ::core::ffi::c_int);
                }
                lua_settop(L, -1 as ::core::ffi::c_int - 1 as ::core::ffi::c_int);
            }
        }
        7 | 8 => {
            lmpack_geti(
                L,
                (*unpacker).reg,
                (*node).data[0 as ::core::ffi::c_int as usize].i as ::core::ffi::c_int,
            );
            lmpack_unref(
                L,
                (*unpacker).reg,
                (*node).data[0 as ::core::ffi::c_int as usize].i as ::core::ffi::c_int,
            );
            if (*node).key_visited == 0 as ::core::ffi::c_int
                && (*node).tok.type_0 as ::core::ffi::c_uint
                    == MPACK_TOKEN_MAP as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                lmpack_geti(L, (*unpacker).reg, (*unpacker).mtdict);
                lua_setmetatable(L, -2 as ::core::ffi::c_int);
            }
        }
        _ => {}
    }
    if !parent.is_null()
        && ((*parent).tok.type_0 as ::core::ffi::c_uint)
            < MPACK_TOKEN_BIN as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        lmpack_geti(
            L,
            (*unpacker).reg,
            (*parent).data[0 as ::core::ffi::c_int as usize].i as ::core::ffi::c_int,
        );
        if (*parent).tok.type_0 as ::core::ffi::c_uint
            == MPACK_TOKEN_ARRAY as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            lua_pushnumber(L, (*parent).pos as lua_Number);
            lua_pushvalue(L, -3 as ::core::ffi::c_int);
            lua_settable(L, -3 as ::core::ffi::c_int);
        } else {
            '_c2rust_label: {
                if (*parent).tok.type_0 as ::core::ffi::c_uint
                    == MPACK_TOKEN_MAP as ::core::ffi::c_int as ::core::ffi::c_uint
                {
                } else {
                    __assert_fail(
                        b"parent->tok.type == MPACK_TOKEN_MAP\0".as_ptr()
                            as *const ::core::ffi::c_char,
                        b"src/mpack/lmpack.rs\0".as_ptr() as *const ::core::ffi::c_char,
                        410 as ::core::ffi::c_uint,
                        b"void lmpack_parse_exit(mpack_parser_t *, mpack_node_t *)\0".as_ptr()
                            as *const ::core::ffi::c_char,
                    );
                }
            };
            if (*parent).key_visited != 0 {
                lua_pushvalue(L, -2 as ::core::ffi::c_int);
                (*parent).data[1 as ::core::ffi::c_int as usize].i =
                    lmpack_ref(L, (*unpacker).reg) as mpack_sintmax_t;
            } else {
                lmpack_geti(
                    L,
                    (*unpacker).reg,
                    (*parent).data[1 as ::core::ffi::c_int as usize].i as ::core::ffi::c_int,
                );
                lmpack_unref(
                    L,
                    (*unpacker).reg,
                    (*parent).data[1 as ::core::ffi::c_int as usize].i as ::core::ffi::c_int,
                );
                lua_pushvalue(L, -3 as ::core::ffi::c_int);
                lua_settable(L, -3 as ::core::ffi::c_int);
            }
        }
        lua_settop(L, -2 as ::core::ffi::c_int - 1 as ::core::ffi::c_int);
    }
}
unsafe extern "C-unwind" fn lmpack_unpacker_unpack_str(
    mut L: *mut lua_State,
    mut unpacker: *mut Unpacker,
    mut str: *mut *const ::core::ffi::c_char,
    mut len: *mut size_t,
) -> ::core::ffi::c_int {
    let mut rv: ::core::ffi::c_int = 0;
    if (*unpacker).unpacking != 0 {
        return luaL_error(
            L,
            b"Unpacker instance already working. Use another Unpacker or mpack.decode() if you need to decode from the ext handler\0"
                .as_ptr() as *const ::core::ffi::c_char,
        );
    }
    loop {
        (*unpacker).unpacking = 1 as ::core::ffi::c_int;
        rv = mpack_parse(
            (*unpacker).parser,
            str,
            len,
            Some(
                lmpack_parse_enter
                    as unsafe extern "C-unwind" fn(*mut mpack_parser_t, *mut mpack_node_t) -> (),
            ),
            Some(
                lmpack_parse_exit
                    as unsafe extern "C-unwind" fn(*mut mpack_parser_t, *mut mpack_node_t) -> (),
            ),
        );
        (*unpacker).unpacking = 0 as ::core::ffi::c_int;
        if rv == MPACK_NOMEM as ::core::ffi::c_int {
            (*unpacker).parser = lmpack_grow_parser((*unpacker).parser);
            if (*unpacker).parser.is_null() {
                return luaL_error(
                    L,
                    b"failed to grow Unpacker capacity\0".as_ptr() as *const ::core::ffi::c_char,
                );
            }
        }
        if rv != MPACK_NOMEM as ::core::ffi::c_int {
            break;
        }
    }
    if rv == MPACK_ERROR as ::core::ffi::c_int {
        return luaL_error(
            L,
            b"invalid msgpack string\0".as_ptr() as *const ::core::ffi::c_char,
        );
    }
    return rv;
}
unsafe extern "C-unwind" fn lmpack_unpacker_unpack(mut L: *mut lua_State) -> ::core::ffi::c_int {
    let mut result: ::core::ffi::c_int = 0;
    let mut argc: ::core::ffi::c_int = 0;
    let mut startpos: lua_Number = 0.;
    let mut len: size_t = 0;
    let mut offset: size_t = 0;
    let mut str: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut str_init: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut unpacker: *mut Unpacker = ::core::ptr::null_mut::<Unpacker>();
    argc = lua_gettop(L);
    if argc > 3 as ::core::ffi::c_int || argc < 2 as ::core::ffi::c_int {
        return luaL_error(
            L,
            b"expecting between 2 and 3 arguments\0".as_ptr() as *const ::core::ffi::c_char,
        );
    }
    unpacker = lmpack_check_unpacker(L, 1 as ::core::ffi::c_int);
    (*unpacker).L = L;
    str = luaL_checklstring(L, 2 as ::core::ffi::c_int, &raw mut len);
    str_init = str;
    startpos = if lua_gettop(L) == 3 as ::core::ffi::c_int {
        luaL_checknumber(L, 3 as ::core::ffi::c_int)
    } else {
        1 as ::core::ffi::c_int as lua_Number
    };
    (startpos > 0 as ::core::ffi::c_int as lua_Number
        || luaL_argerror(
            L,
            3 as ::core::ffi::c_int,
            b"start position must be greater than zero\0".as_ptr() as *const ::core::ffi::c_char,
        ) != 0) as ::core::ffi::c_int;
    (startpos as size_t as lua_Number == startpos
        || luaL_argerror(
            L,
            3 as ::core::ffi::c_int,
            b"start position must be an integer\0".as_ptr() as *const ::core::ffi::c_char,
        ) != 0) as ::core::ffi::c_int;
    (startpos as size_t <= len
        || luaL_argerror(
            L,
            3 as ::core::ffi::c_int,
            b"start position must be less than or equal to the input string length\0".as_ptr()
                as *const ::core::ffi::c_char,
        ) != 0) as ::core::ffi::c_int;
    offset = (startpos as size_t).wrapping_sub(1 as size_t);
    str = str.offset(offset as isize);
    len = len.wrapping_sub(offset);
    result = lmpack_unpacker_unpack_str(L, unpacker, &raw mut str, &raw mut len);
    if result == MPACK_EOF as ::core::ffi::c_int {
        lua_pushnil(L);
    }
    lua_pushinteger(L, str.offset_from(str_init) + 1 as lua_Integer);
    '_c2rust_label: {
        if lua_gettop(L) == argc + 2 as ::core::ffi::c_int {
        } else {
            __assert_fail(
                b"lua_gettop(L) == argc + 2\0".as_ptr() as *const ::core::ffi::c_char,
                b"src/mpack/lmpack.rs\0".as_ptr() as *const ::core::ffi::c_char,
                493 as ::core::ffi::c_uint,
                b"int lmpack_unpacker_unpack(lua_State *)\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
    };
    return 2 as ::core::ffi::c_int;
}
unsafe extern "C-unwind" fn lmpack_packer_new(mut L: *mut lua_State) -> ::core::ffi::c_int {
    let mut rv: *mut Packer = ::core::ptr::null_mut::<Packer>();
    if lua_gettop(L) > 1 as ::core::ffi::c_int {
        return luaL_error(
            L,
            b"expecting at most 1 table argument\0".as_ptr() as *const ::core::ffi::c_char,
        );
    }
    rv = lua_newuserdata(L, ::core::mem::size_of::<Packer>()) as *mut Packer;
    (*rv).parser = malloc(::core::mem::size_of::<mpack_parser_t>()) as *mut mpack_parser_t;
    if (*rv).parser.is_null() {
        return luaL_error(
            L,
            b"failed to allocate parser memory\0".as_ptr() as *const ::core::ffi::c_char,
        );
    }
    mpack_parser_init((*rv).parser, 0 as mpack_uint32_t);
    (*(*rv).parser).data.p = rv as *mut ::core::ffi::c_void;
    (*rv).L = L;
    (*rv).packing = 0 as ::core::ffi::c_int;
    (*rv).is_bin = 0 as ::core::ffi::c_int;
    (*rv).is_bin_fn = LUA_NOREF;
    lua_getfield(
        L,
        LUA_REGISTRYINDEX,
        b"mpack.Packer\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_setmetatable(L, -2 as ::core::ffi::c_int);
    lua_createtable(L, 0 as ::core::ffi::c_int, 0 as ::core::ffi::c_int);
    (*rv).reg = luaL_ref(L, LUA_REGISTRYINDEX);
    (*rv).ext = LUA_NOREF;
    lua_getfield(L, LUA_REGISTRYINDEX, EMPTY_DICT_NAME.as_ptr());
    (*rv).mtdict = lmpack_ref(L, (*rv).reg);
    if lua_type(L, 1 as ::core::ffi::c_int) == LUA_TTABLE {
        lua_getfield(
            L,
            1 as ::core::ffi::c_int,
            b"ext\0".as_ptr() as *const ::core::ffi::c_char,
        );
        if !(lua_type(L, -1 as ::core::ffi::c_int) == LUA_TNIL) {
            if !(lua_type(L, -1 as ::core::ffi::c_int) == LUA_TTABLE) {
                return luaL_error(
                    L,
                    b"\"ext\" option must be a table\0".as_ptr() as *const ::core::ffi::c_char,
                );
            }
            lmpack_shallow_copy(L);
        }
        (*rv).ext = lmpack_ref(L, (*rv).reg);
        lua_getfield(
            L,
            1 as ::core::ffi::c_int,
            b"is_bin\0".as_ptr() as *const ::core::ffi::c_char,
        );
        if !(lua_type(L, -1 as ::core::ffi::c_int) == LUA_TNIL) {
            if !(lua_type(L, -1 as ::core::ffi::c_int) == LUA_TBOOLEAN)
                && !(lua_type(L, -1 as ::core::ffi::c_int) == LUA_TFUNCTION)
            {
                return luaL_error(
                    L,
                    b"\"is_bin\" option must be a boolean or function\0".as_ptr()
                        as *const ::core::ffi::c_char,
                );
            }
            (*rv).is_bin = lua_toboolean(L, -1 as ::core::ffi::c_int);
            if lua_type(L, -1 as ::core::ffi::c_int) == LUA_TFUNCTION {
                (*rv).is_bin_fn = lmpack_ref(L, (*rv).reg);
            } else {
                lua_settop(L, -1 as ::core::ffi::c_int - 1 as ::core::ffi::c_int);
            }
        } else {
            lua_settop(L, -1 as ::core::ffi::c_int - 1 as ::core::ffi::c_int);
        }
    }
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C-unwind" fn lmpack_packer_delete(mut L: *mut lua_State) -> ::core::ffi::c_int {
    let mut packer: *mut Packer = lmpack_check_packer(L, 1 as ::core::ffi::c_int);
    if (*packer).ext != LUA_NOREF {
        lmpack_unref(L, (*packer).reg, (*packer).ext);
    }
    luaL_unref(L, LUA_REGISTRYINDEX, (*packer).reg);
    free((*packer).parser as *mut ::core::ffi::c_void);
    return 0 as ::core::ffi::c_int;
}
unsafe extern "C-unwind" fn lmpack_unparse_enter(
    mut parser: *mut mpack_parser_t,
    mut node: *mut mpack_node_t,
) {
    let mut type_0: ::core::ffi::c_int = 0;
    let mut packer: *mut Packer = (*parser).data.p as *mut Packer;
    let mut L: *mut lua_State = (*packer).L;
    let mut parent: *mut mpack_node_t = if (*node.offset(-(1 as ::core::ffi::c_int as isize))).pos
        == -1 as ::core::ffi::c_int as size_t
    {
        ::core::ptr::null_mut::<mpack_node_t>()
    } else {
        node.offset(-(1 as ::core::ffi::c_int as isize))
    };
    if !parent.is_null() {
        lmpack_geti(
            L,
            (*packer).reg,
            (*parent).data[0 as ::core::ffi::c_int as usize].i as ::core::ffi::c_int,
        );
        if (*parent).tok.type_0 as ::core::ffi::c_uint
            > MPACK_TOKEN_MAP as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            let mut str: *const ::core::ffi::c_char = lua_tolstring(
                L,
                -1 as ::core::ffi::c_int,
                ::core::ptr::null_mut::<size_t>(),
            );
            (*node).tok = mpack_pack_chunk(str, (*parent).tok.length);
            lua_settop(L, -1 as ::core::ffi::c_int - 1 as ::core::ffi::c_int);
            return;
        }
        if (*parent).tok.type_0 as ::core::ffi::c_uint
            == MPACK_TOKEN_ARRAY as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            lua_pushnumber(L, (*parent).pos.wrapping_add(1 as size_t) as lua_Number);
            lua_gettable(L, -2 as ::core::ffi::c_int);
        } else if (*parent).tok.type_0 as ::core::ffi::c_uint
            == MPACK_TOKEN_MAP as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            let mut result: ::core::ffi::c_int = 0;
            lmpack_geti(
                L,
                (*packer).reg,
                (*parent).data[1 as ::core::ffi::c_int as usize].i as ::core::ffi::c_int,
            );
            result = lua_next(L, -2 as ::core::ffi::c_int);
            '_c2rust_label: {
                if result != 0 {
                } else {
                    __assert_fail(
                        b"result\0".as_ptr() as *const ::core::ffi::c_char,
                        b"src/mpack/lmpack.rs\0".as_ptr() as *const ::core::ffi::c_char,
                        594 as ::core::ffi::c_uint,
                        b"void lmpack_unparse_enter(mpack_parser_t *, mpack_node_t *)\0".as_ptr()
                            as *const ::core::ffi::c_char,
                    );
                }
            };
            if (*parent).key_visited != 0 {
                lmpack_unref(
                    L,
                    (*packer).reg,
                    (*parent).data[1 as ::core::ffi::c_int as usize].i as ::core::ffi::c_int,
                );
                lua_pushvalue(L, -2 as ::core::ffi::c_int);
                (*parent).data[1 as ::core::ffi::c_int as usize].i =
                    lmpack_ref(L, (*packer).reg) as mpack_sintmax_t;
                lua_replace(L, -2 as ::core::ffi::c_int);
            } else {
                lua_settop(L, -1 as ::core::ffi::c_int - 1 as ::core::ffi::c_int);
            }
        }
        lua_remove(L, -2 as ::core::ffi::c_int);
    } else {
        lmpack_geti(L, (*packer).reg, (*packer).root);
    }
    type_0 = lua_type(L, -1 as ::core::ffi::c_int);
    '_end: {
        match type_0 {
            LUA_TBOOLEAN => {
                (*node).tok = mpack_pack_boolean(
                    lua_toboolean(L, -1 as ::core::ffi::c_int) as ::core::ffi::c_uint
                );
                break '_end;
            }
            LUA_TNUMBER => {
                (*node).tok = mpack_pack_number(lua_tonumber(L, -1 as ::core::ffi::c_int));
                break '_end;
            }
            LUA_TSTRING => {
                let mut is_bin: ::core::ffi::c_int = (*packer).is_bin;
                if is_bin != 0 && (*packer).is_bin_fn != LUA_NOREF {
                    lmpack_geti(L, (*packer).reg, (*packer).is_bin_fn);
                    lua_pushvalue(L, -2 as ::core::ffi::c_int);
                    lua_call(L, 1 as ::core::ffi::c_int, 1 as ::core::ffi::c_int);
                    is_bin = lua_toboolean(L, -1 as ::core::ffi::c_int);
                    lua_settop(L, -1 as ::core::ffi::c_int - 1 as ::core::ffi::c_int);
                }
                if is_bin != 0 {
                    (*node).tok = mpack_pack_bin(lmpack_objlen(
                        L,
                        ::core::ptr::null_mut::<::core::ffi::c_int>(),
                    ));
                } else {
                    (*node).tok = mpack_pack_str(lmpack_objlen(
                        L,
                        ::core::ptr::null_mut::<::core::ffi::c_int>(),
                    ));
                }
                break '_end;
            }
            LUA_TTABLE => {
                let mut len: mpack_uint32_t = 0;
                let mut n: *mut mpack_node_t = ::core::ptr::null_mut::<mpack_node_t>();
                let mut has_meta: ::core::ffi::c_int =
                    lua_getmetatable(L, -1 as ::core::ffi::c_int);
                let mut has_mtdict: ::core::ffi::c_int = false_0;
                if has_meta != 0 && (*packer).mtdict != LUA_NOREF {
                    lmpack_geti(L, (*packer).reg, (*packer).mtdict);
                    has_mtdict =
                        lua_rawequal(L, -1 as ::core::ffi::c_int, -2 as ::core::ffi::c_int);
                    lua_settop(L, -1 as ::core::ffi::c_int - 1 as ::core::ffi::c_int);
                }
                if (*packer).ext != LUA_NOREF && has_meta != 0 && has_mtdict == 0 {
                    lmpack_geti(L, (*packer).reg, (*packer).ext);
                    lua_pushvalue(L, -2 as ::core::ffi::c_int);
                    lua_gettable(L, -2 as ::core::ffi::c_int);
                    if lua_type(L, -1 as ::core::ffi::c_int) == LUA_TFUNCTION {
                        let mut ext: lua_Number = -1 as ::core::ffi::c_int as lua_Number;
                        lua_pushvalue(L, -4 as ::core::ffi::c_int);
                        lua_call(L, 1 as ::core::ffi::c_int, 2 as ::core::ffi::c_int);
                        if lua_isnumber(L, -2 as ::core::ffi::c_int) == 0
                            || {
                                ext = lua_tonumber(L, -2 as ::core::ffi::c_int);
                                ext < 0 as ::core::ffi::c_int as lua_Number
                            }
                            || ext > 127 as ::core::ffi::c_int as lua_Number
                            || ext as ::core::ffi::c_int as lua_Number != ext
                        {
                            luaL_error(
                                L,
                                b"the first result from ext packer must be an integer between 0 and 127\0"
                                    .as_ptr() as *const ::core::ffi::c_char,
                            );
                        }
                        if lua_isstring(L, -1 as ::core::ffi::c_int) == 0 {
                            luaL_error(
                                L,
                                b"the second result from ext packer must be a string\0".as_ptr()
                                    as *const ::core::ffi::c_char,
                            );
                        }
                        (*node).tok = mpack_pack_ext(
                            ext as ::core::ffi::c_int,
                            lmpack_objlen(L, ::core::ptr::null_mut::<::core::ffi::c_int>()),
                        );
                        lua_replace(L, -5 as ::core::ffi::c_int);
                        lua_settop(L, -3 as ::core::ffi::c_int - 1 as ::core::ffi::c_int);
                        break '_end;
                    } else {
                        lua_settop(L, -1 as ::core::ffi::c_int - 1 as ::core::ffi::c_int);
                    }
                }
                if has_meta != 0 {
                    lua_settop(L, -1 as ::core::ffi::c_int - 1 as ::core::ffi::c_int);
                }
                n = node;
                loop {
                    n = if (*n.offset(-(1 as ::core::ffi::c_int as isize))).pos
                        == -1 as ::core::ffi::c_int as size_t
                    {
                        ::core::ptr::null_mut::<mpack_node_t>()
                    } else {
                        n.offset(-(1 as ::core::ffi::c_int as isize))
                    };
                    if n.is_null() {
                        break;
                    }
                    lmpack_geti(
                        L,
                        (*packer).reg,
                        (*n).data[0 as ::core::ffi::c_int as usize].i as ::core::ffi::c_int,
                    );
                    if lua_rawequal(L, -1 as ::core::ffi::c_int, -2 as ::core::ffi::c_int) != 0 {
                        (*node).tok = mpack_pack_nil();
                        lua_settop(L, -2 as ::core::ffi::c_int - 1 as ::core::ffi::c_int);
                        lmpack_pushnil(L);
                        break '_end;
                    } else {
                        lua_settop(L, -1 as ::core::ffi::c_int - 1 as ::core::ffi::c_int);
                    }
                }
                let mut is_array: ::core::ffi::c_int = (has_mtdict == 0) as ::core::ffi::c_int;
                len = lmpack_objlen(L, &raw mut is_array);
                if is_array != 0 {
                    (*node).tok = mpack_pack_array(len);
                } else {
                    (*node).tok = mpack_pack_map(len);
                    (*node).data[1 as ::core::ffi::c_int as usize].i =
                        LUA_REFNIL as mpack_sintmax_t;
                }
                break '_end;
            }
            LUA_TUSERDATA => {
                if lmpack_isnil(L, -1 as ::core::ffi::c_int) != 0 {
                    (*node).tok = mpack_pack_nil();
                    break '_end;
                }
            }
            _ => {}
        }
        let mut errmsg: [::core::ffi::c_char; 50] = [0; 50];
        snprintf(
            &raw mut errmsg as *mut ::core::ffi::c_char,
            50 as size_t,
            b"can't serialize object of type %d\0".as_ptr() as *const ::core::ffi::c_char,
            type_0,
        );
        luaL_error(L, &raw mut errmsg as *mut ::core::ffi::c_char);
    }
    (*node).data[0 as ::core::ffi::c_int as usize].i =
        lmpack_ref(L, (*packer).reg) as mpack_sintmax_t;
}
unsafe extern "C-unwind" fn lmpack_unparse_exit(
    mut parser: *mut mpack_parser_t,
    mut node: *mut mpack_node_t,
) {
    let mut packer: *mut Packer = (*parser).data.p as *mut Packer;
    let mut L: *mut lua_State = (*packer).L;
    if (*node).tok.type_0 as ::core::ffi::c_uint
        != MPACK_TOKEN_CHUNK as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        lmpack_unref(
            L,
            (*packer).reg,
            (*node).data[0 as ::core::ffi::c_int as usize].i as ::core::ffi::c_int,
        );
        if (*node).tok.type_0 as ::core::ffi::c_uint
            == MPACK_TOKEN_MAP as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            lmpack_unref(
                L,
                (*packer).reg,
                (*node).data[1 as ::core::ffi::c_int as usize].i as ::core::ffi::c_int,
            );
        }
    }
}
unsafe extern "C-unwind" fn lmpack_packer_pack(mut L: *mut lua_State) -> ::core::ffi::c_int {
    let mut b: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut bl: size_t = 0;
    let mut result: ::core::ffi::c_int = 0;
    let mut argc: ::core::ffi::c_int = 0;
    let mut packer: *mut Packer = ::core::ptr::null_mut::<Packer>();
    let mut buffer: luaL_Buffer = luaL_Buffer {
        p: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        lvl: 0,
        L: ::core::ptr::null_mut::<lua_State>(),
        buffer: [0; 8192],
    };
    argc = lua_gettop(L);
    if argc != 2 as ::core::ffi::c_int {
        return luaL_error(
            L,
            b"expecting exactly 2 arguments\0".as_ptr() as *const ::core::ffi::c_char,
        );
    }
    packer = lmpack_check_packer(L, 1 as ::core::ffi::c_int);
    (*packer).L = L;
    (*packer).root = lmpack_ref(L, (*packer).reg);
    luaL_buffinit(L, &raw mut buffer);
    b = luaL_prepbuffer(&raw mut buffer);
    bl = LUAL_BUFFERSIZE as size_t;
    if (*packer).packing != 0 {
        return luaL_error(
            L,
            b"Packer instance already working. Use another Packer or mpack.encode() if you need to encode from the ext handler\0"
                .as_ptr() as *const ::core::ffi::c_char,
        );
    }
    loop {
        let mut bl_init: size_t = bl;
        (*packer).packing = 1 as ::core::ffi::c_int;
        result = mpack_unparse(
            (*packer).parser,
            &raw mut b,
            &raw mut bl,
            Some(
                lmpack_unparse_enter
                    as unsafe extern "C-unwind" fn(*mut mpack_parser_t, *mut mpack_node_t) -> (),
            ),
            Some(
                lmpack_unparse_exit
                    as unsafe extern "C-unwind" fn(*mut mpack_parser_t, *mut mpack_node_t) -> (),
            ),
        );
        (*packer).packing = 0 as ::core::ffi::c_int;
        if result == MPACK_NOMEM as ::core::ffi::c_int {
            (*packer).parser = lmpack_grow_parser((*packer).parser);
            if (*packer).parser.is_null() {
                return luaL_error(
                    L,
                    b"Failed to grow Packer capacity\0".as_ptr() as *const ::core::ffi::c_char,
                );
            }
        }
        buffer.p = buffer.p.offset(bl_init.wrapping_sub(bl) as isize);
        if bl == 0 {
            b = luaL_prepbuffer(&raw mut buffer);
            bl = LUAL_BUFFERSIZE as size_t;
        }
        if !(result == MPACK_EOF as ::core::ffi::c_int
            || result == MPACK_NOMEM as ::core::ffi::c_int)
        {
            break;
        }
    }
    lmpack_unref(L, (*packer).reg, (*packer).root);
    luaL_pushresult(&raw mut buffer);
    '_c2rust_label: {
        if lua_gettop(L) == argc {
        } else {
            __assert_fail(
                b"lua_gettop(L) == argc\0".as_ptr() as *const ::core::ffi::c_char,
                b"src/mpack/lmpack.rs\0".as_ptr() as *const ::core::ffi::c_char,
                816 as ::core::ffi::c_uint,
                b"int lmpack_packer_pack(lua_State *)\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
    };
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C-unwind" fn lmpack_session_new(mut L: *mut lua_State) -> ::core::ffi::c_int {
    let mut rv: *mut Session =
        lua_newuserdata(L, ::core::mem::size_of::<Session>()) as *mut Session;
    (*rv).session =
        malloc(::core::mem::size_of::<mpack_rpc_session_t>()) as *mut mpack_rpc_session_t;
    if (*rv).session.is_null() {
        return luaL_error(
            L,
            b"Failed to allocate memory\0".as_ptr() as *const ::core::ffi::c_char,
        );
    }
    mpack_rpc_session_init((*rv).session, 0 as mpack_uint32_t);
    (*rv).L = L;
    lua_getfield(
        L,
        LUA_REGISTRYINDEX,
        b"mpack.Session\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_setmetatable(L, -2 as ::core::ffi::c_int);
    lua_createtable(L, 0 as ::core::ffi::c_int, 0 as ::core::ffi::c_int);
    (*rv).reg = luaL_ref(L, LUA_REGISTRYINDEX);
    (*rv).unpacker = LUA_REFNIL;
    (*rv).unpacked.args_or_result = LUA_NOREF;
    (*rv).unpacked.method_or_error = LUA_NOREF;
    (*rv).unpacked.type_0 = MPACK_EOF as ::core::ffi::c_int;
    if lua_type(L, 1 as ::core::ffi::c_int) == LUA_TTABLE {
        lua_getfield(
            L,
            1 as ::core::ffi::c_int,
            b"unpack\0".as_ptr() as *const ::core::ffi::c_char,
        );
        if lmpack_isunpacker(L, -1 as ::core::ffi::c_int) == 0 {
            return luaL_error(
                L,
                b"\"unpack\" option must be a mpack.Unpacker instance\0".as_ptr()
                    as *const ::core::ffi::c_char,
            );
        }
        (*rv).unpacker = lmpack_ref(L, (*rv).reg);
    }
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C-unwind" fn lmpack_session_delete(mut L: *mut lua_State) -> ::core::ffi::c_int {
    let mut session: *mut Session = lmpack_check_session(L, 1 as ::core::ffi::c_int);
    lmpack_unref(L, (*session).reg, (*session).unpacker);
    luaL_unref(L, LUA_REGISTRYINDEX, (*session).reg);
    free((*session).session as *mut ::core::ffi::c_void);
    return 0 as ::core::ffi::c_int;
}
unsafe extern "C-unwind" fn lmpack_session_receive(mut L: *mut lua_State) -> ::core::ffi::c_int {
    let mut argc: ::core::ffi::c_int = 0;
    let mut done: ::core::ffi::c_int = 0;
    let mut rcount: ::core::ffi::c_int = 3 as ::core::ffi::c_int;
    let mut startpos: lua_Number = 0.;
    let mut len: size_t = 0;
    let mut str: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut str_init: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut session: *mut Session = ::core::ptr::null_mut::<Session>();
    let mut unpacker: *mut Unpacker = ::core::ptr::null_mut::<Unpacker>();
    argc = lua_gettop(L);
    if argc > 3 as ::core::ffi::c_int || argc < 2 as ::core::ffi::c_int {
        return luaL_error(
            L,
            b"expecting between 2 and 3 arguments\0".as_ptr() as *const ::core::ffi::c_char,
        );
    }
    session = lmpack_check_session(L, 1 as ::core::ffi::c_int);
    str = luaL_checklstring(L, 2 as ::core::ffi::c_int, &raw mut len);
    str_init = str;
    startpos = if lua_gettop(L) == 3 as ::core::ffi::c_int {
        luaL_checknumber(L, 3 as ::core::ffi::c_int)
    } else {
        1 as ::core::ffi::c_int as lua_Number
    };
    (startpos > 0 as ::core::ffi::c_int as lua_Number
        || luaL_argerror(
            L,
            3 as ::core::ffi::c_int,
            b"start position must be greater than zero\0".as_ptr() as *const ::core::ffi::c_char,
        ) != 0) as ::core::ffi::c_int;
    (startpos as size_t as lua_Number == startpos
        || luaL_argerror(
            L,
            3 as ::core::ffi::c_int,
            b"start position must be an integer\0".as_ptr() as *const ::core::ffi::c_char,
        ) != 0) as ::core::ffi::c_int;
    (startpos as size_t <= len
        || luaL_argerror(
            L,
            3 as ::core::ffi::c_int,
            b"start position must be less than or equal to the input string length\0".as_ptr()
                as *const ::core::ffi::c_char,
        ) != 0) as ::core::ffi::c_int;
    let mut offset: size_t = (startpos as size_t).wrapping_sub(1 as size_t);
    str = str.offset(offset as isize);
    len = len.wrapping_sub(offset);
    if (*session).unpacker != LUA_REFNIL {
        lmpack_geti(L, (*session).reg, (*session).unpacker);
        unpacker = lmpack_check_unpacker(L, -1 as ::core::ffi::c_int);
        (*unpacker).L = L;
        rcount += 2 as ::core::ffi::c_int;
        lua_settop(L, -1 as ::core::ffi::c_int - 1 as ::core::ffi::c_int);
    }
    loop {
        let mut result: ::core::ffi::c_int = 0;
        if (*session).unpacked.type_0 == MPACK_EOF as ::core::ffi::c_int {
            (*session).unpacked.type_0 = mpack_rpc_receive(
                (*session).session,
                &raw mut str,
                &raw mut len,
                &raw mut (*session).unpacked.msg,
            );
            if unpacker.is_null() || (*session).unpacked.type_0 == MPACK_EOF as ::core::ffi::c_int {
                break;
            }
        }
        result = lmpack_unpacker_unpack_str(L, unpacker, &raw mut str, &raw mut len);
        if result == MPACK_EOF as ::core::ffi::c_int {
            break;
        }
        if (*session).unpacked.method_or_error == LUA_NOREF {
            (*session).unpacked.method_or_error = lmpack_ref(L, (*session).reg);
        } else {
            (*session).unpacked.args_or_result = lmpack_ref(L, (*session).reg);
            break;
        }
    }
    done = ((*session).unpacked.type_0 != MPACK_EOF as ::core::ffi::c_int
        && ((*session).unpacked.args_or_result != LUA_NOREF || unpacker.is_null()))
        as ::core::ffi::c_int;
    if done == 0 {
        lua_pushnil(L);
        lua_pushnil(L);
        if !unpacker.is_null() {
            lua_pushnil(L);
            lua_pushnil(L);
        }
    } else {
        match (*session).unpacked.type_0 {
            4 => {
                lua_pushstring(L, b"request\0".as_ptr() as *const ::core::ffi::c_char);
                lua_pushnumber(L, (*session).unpacked.msg.id as lua_Number);
            }
            5 => {
                lua_pushstring(L, b"response\0".as_ptr() as *const ::core::ffi::c_char);
                lmpack_geti(
                    L,
                    (*session).reg,
                    (*session).unpacked.msg.data.i as ::core::ffi::c_int,
                );
            }
            6 => {
                lua_pushstring(L, b"notification\0".as_ptr() as *const ::core::ffi::c_char);
                lua_pushnil(L);
            }
            _ => {
                return luaL_error(
                    L,
                    b"invalid msgpack-rpc string\0".as_ptr() as *const ::core::ffi::c_char,
                );
            }
        }
        (*session).unpacked.type_0 = MPACK_EOF as ::core::ffi::c_int;
        if !unpacker.is_null() {
            lmpack_geti(L, (*session).reg, (*session).unpacked.method_or_error);
            lmpack_geti(L, (*session).reg, (*session).unpacked.args_or_result);
            lmpack_unref(L, (*session).reg, (*session).unpacked.method_or_error);
            lmpack_unref(L, (*session).reg, (*session).unpacked.args_or_result);
            (*session).unpacked.method_or_error = LUA_NOREF;
            (*session).unpacked.args_or_result = LUA_NOREF;
        }
    }
    lua_pushinteger(L, str.offset_from(str_init) + 1 as lua_Integer);
    return rcount;
}
unsafe extern "C-unwind" fn lmpack_session_request(mut L: *mut lua_State) -> ::core::ffi::c_int {
    let mut result: ::core::ffi::c_int = 0;
    let mut buf: [::core::ffi::c_char; 16] = [0; 16];
    let mut b: *mut ::core::ffi::c_char = &raw mut buf as *mut ::core::ffi::c_char;
    let mut bl: size_t = ::core::mem::size_of::<[::core::ffi::c_char; 16]>();
    let mut session: *mut Session = ::core::ptr::null_mut::<Session>();
    let mut data: mpack_data_t = mpack_data_t {
        p: ::core::ptr::null_mut::<::core::ffi::c_void>(),
    };
    if lua_gettop(L) > 2 as ::core::ffi::c_int || lua_gettop(L) < 1 as ::core::ffi::c_int {
        return luaL_error(
            L,
            b"expecting 1 or 2 arguments\0".as_ptr() as *const ::core::ffi::c_char,
        );
    }
    session = lmpack_check_session(L, 1 as ::core::ffi::c_int);
    data.i = (if lua_type(L, 2 as ::core::ffi::c_int) <= 0 as ::core::ffi::c_int {
        LUA_NOREF
    } else {
        lmpack_ref(L, (*session).reg)
    }) as mpack_sintmax_t;
    loop {
        result = mpack_rpc_request((*session).session, &raw mut b, &raw mut bl, data);
        if result == MPACK_NOMEM as ::core::ffi::c_int {
            (*session).session = lmpack_grow_session((*session).session);
            if (*session).session.is_null() {
                return luaL_error(
                    L,
                    b"Failed to grow Session capacity\0".as_ptr() as *const ::core::ffi::c_char,
                );
            }
        }
        if result != MPACK_NOMEM as ::core::ffi::c_int {
            break;
        }
    }
    '_c2rust_label: {
        if result == MPACK_OK as ::core::ffi::c_int {
        } else {
            __assert_fail(
                b"result == MPACK_OK\0".as_ptr() as *const ::core::ffi::c_char,
                b"src/mpack/lmpack.rs\0".as_ptr() as *const ::core::ffi::c_char,
                991 as ::core::ffi::c_uint,
                b"int lmpack_session_request(lua_State *)\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
    };
    lua_pushlstring(
        L,
        &raw mut buf as *mut ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 16]>().wrapping_sub(bl),
    );
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C-unwind" fn lmpack_session_reply(mut L: *mut lua_State) -> ::core::ffi::c_int {
    let mut result: ::core::ffi::c_int = 0;
    let mut buf: [::core::ffi::c_char; 16] = [0; 16];
    let mut b: *mut ::core::ffi::c_char = &raw mut buf as *mut ::core::ffi::c_char;
    let mut bl: size_t = ::core::mem::size_of::<[::core::ffi::c_char; 16]>();
    let mut session: *mut Session = ::core::ptr::null_mut::<Session>();
    let mut id: lua_Number = 0.;
    if lua_gettop(L) != 2 as ::core::ffi::c_int {
        return luaL_error(
            L,
            b"expecting exactly 2 arguments\0".as_ptr() as *const ::core::ffi::c_char,
        );
    }
    session = lmpack_check_session(L, 1 as ::core::ffi::c_int);
    id = lua_tonumber(L, 2 as ::core::ffi::c_int);
    (id as size_t as lua_Number == id
        && id >= 0 as ::core::ffi::c_int as lua_Number
        && id <= 0xffffffff as ::core::ffi::c_uint as lua_Number
        || luaL_argerror(
            L,
            2 as ::core::ffi::c_int,
            b"invalid request id\0".as_ptr() as *const ::core::ffi::c_char,
        ) != 0) as ::core::ffi::c_int;
    result = mpack_rpc_reply(
        (*session).session,
        &raw mut b,
        &raw mut bl,
        id as mpack_uint32_t,
    );
    '_c2rust_label: {
        if result == MPACK_OK as ::core::ffi::c_int {
        } else {
            __assert_fail(
                b"result == MPACK_OK\0".as_ptr() as *const ::core::ffi::c_char,
                b"src/mpack/lmpack.rs\0".as_ptr() as *const ::core::ffi::c_char,
                1012 as ::core::ffi::c_uint,
                b"int lmpack_session_reply(lua_State *)\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
    };
    lua_pushlstring(
        L,
        &raw mut buf as *mut ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 16]>().wrapping_sub(bl),
    );
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C-unwind" fn lmpack_session_notify(mut L: *mut lua_State) -> ::core::ffi::c_int {
    let mut result: ::core::ffi::c_int = 0;
    let mut buf: [::core::ffi::c_char; 16] = [0; 16];
    let mut b: *mut ::core::ffi::c_char = &raw mut buf as *mut ::core::ffi::c_char;
    let mut bl: size_t = ::core::mem::size_of::<[::core::ffi::c_char; 16]>();
    let mut session: *mut Session = ::core::ptr::null_mut::<Session>();
    if lua_gettop(L) != 1 as ::core::ffi::c_int {
        return luaL_error(
            L,
            b"expecting exactly 1 argument\0".as_ptr() as *const ::core::ffi::c_char,
        );
    }
    session = lmpack_check_session(L, 1 as ::core::ffi::c_int);
    result = mpack_rpc_notify((*session).session, &raw mut b, &raw mut bl);
    '_c2rust_label: {
        if result == MPACK_OK as ::core::ffi::c_int {
        } else {
            __assert_fail(
                b"result == MPACK_OK\0".as_ptr() as *const ::core::ffi::c_char,
                b"src/mpack/lmpack.rs\0".as_ptr() as *const ::core::ffi::c_char,
                1030 as ::core::ffi::c_uint,
                b"int lmpack_session_notify(lua_State *)\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
    };
    lua_pushlstring(
        L,
        &raw mut buf as *mut ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 16]>().wrapping_sub(bl),
    );
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C-unwind" fn lmpack_nil_tostring(mut L: *mut lua_State) -> ::core::ffi::c_int {
    lua_pushfstring(
        L,
        NIL_NAME.as_ptr(),
        lua_topointer(L, 1 as ::core::ffi::c_int),
    );
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C-unwind" fn lmpack_unpack(mut L: *mut lua_State) -> ::core::ffi::c_int {
    let mut result: ::core::ffi::c_int = 0;
    let mut len: size_t = 0;
    let mut str: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut unpacker: Unpacker = Unpacker {
        L: ::core::ptr::null_mut::<lua_State>(),
        parser: ::core::ptr::null_mut::<mpack_parser_t>(),
        reg: 0,
        ext: 0,
        unpacking: 0,
        mtdict: 0,
        string_buffer: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut parser: mpack_parser_t = mpack_parser_t {
        data: mpack_data_t {
            p: ::core::ptr::null_mut::<::core::ffi::c_void>(),
        },
        size: 0,
        capacity: 0,
        status: 0,
        exiting: 0,
        tokbuf: mpack_tokbuf_t {
            pending: [0; 9],
            pending_tok: mpack_token_t {
                type_0: 0 as mpack_token_type_t,
                length: 0,
                data: C2Rust_Unnamed {
                    value: mpack_value_t { lo: 0, hi: 0 },
                },
            },
            ppos: 0,
            plen: 0,
            passthrough: 0,
        },
        items: [mpack_node_t {
            tok: mpack_token_t {
                type_0: 0 as mpack_token_type_t,
                length: 0,
                data: C2Rust_Unnamed {
                    value: mpack_value_t { lo: 0, hi: 0 },
                },
            },
            pos: 0,
            key_visited: 0,
            data: [mpack_data_t {
                p: ::core::ptr::null_mut::<::core::ffi::c_void>(),
            }; 2],
        }; 33],
    };
    if lua_gettop(L) != 1 as ::core::ffi::c_int {
        return luaL_error(
            L,
            b"expecting exactly 1 argument\0".as_ptr() as *const ::core::ffi::c_char,
        );
    }
    str = luaL_checklstring(L, 1 as ::core::ffi::c_int, &raw mut len);
    lua_createtable(L, 0 as ::core::ffi::c_int, 0 as ::core::ffi::c_int);
    unpacker.reg = luaL_ref(L, LUA_REGISTRYINDEX);
    unpacker.ext = LUA_NOREF;
    unpacker.parser = &raw mut parser;
    mpack_parser_init(unpacker.parser, 0 as mpack_uint32_t);
    (*unpacker.parser).data.p = &raw mut unpacker as *mut ::core::ffi::c_void;
    unpacker.string_buffer = ::core::ptr::null_mut::<::core::ffi::c_char>();
    unpacker.L = L;
    lua_getfield(L, LUA_REGISTRYINDEX, EMPTY_DICT_NAME.as_ptr());
    unpacker.mtdict = lmpack_ref(L, unpacker.reg);
    result = mpack_parse(
        &raw mut parser,
        &raw mut str,
        &raw mut len,
        Some(
            lmpack_parse_enter
                as unsafe extern "C-unwind" fn(*mut mpack_parser_t, *mut mpack_node_t) -> (),
        ),
        Some(
            lmpack_parse_exit
                as unsafe extern "C-unwind" fn(*mut mpack_parser_t, *mut mpack_node_t) -> (),
        ),
    );
    luaL_unref(L, LUA_REGISTRYINDEX, unpacker.reg);
    if result == MPACK_NOMEM as ::core::ffi::c_int {
        return luaL_error(
            L,
            b"object was too deep to unpack\0".as_ptr() as *const ::core::ffi::c_char,
        );
    } else if result == MPACK_EOF as ::core::ffi::c_int {
        return luaL_error(
            L,
            b"incomplete msgpack string\0".as_ptr() as *const ::core::ffi::c_char,
        );
    } else if result == MPACK_ERROR as ::core::ffi::c_int {
        return luaL_error(
            L,
            b"invalid msgpack string\0".as_ptr() as *const ::core::ffi::c_char,
        );
    } else if result == MPACK_OK as ::core::ffi::c_int && len != 0 {
        return luaL_error(
            L,
            b"trailing data in msgpack string\0".as_ptr() as *const ::core::ffi::c_char,
        );
    }
    '_c2rust_label: {
        if result == MPACK_OK as ::core::ffi::c_int {
        } else {
            __assert_fail(
                b"result == MPACK_OK\0".as_ptr() as *const ::core::ffi::c_char,
                b"src/mpack/lmpack.rs\0".as_ptr() as *const ::core::ffi::c_char,
                1082 as ::core::ffi::c_uint,
                b"int lmpack_unpack(lua_State *)\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
    };
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C-unwind" fn lmpack_pack(mut L: *mut lua_State) -> ::core::ffi::c_int {
    let mut b: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut bl: size_t = 0;
    let mut result: ::core::ffi::c_int = 0;
    let mut packer: Packer = Packer {
        L: ::core::ptr::null_mut::<lua_State>(),
        parser: ::core::ptr::null_mut::<mpack_parser_t>(),
        reg: 0,
        ext: 0,
        root: 0,
        packing: 0,
        mtdict: 0,
        is_bin: 0,
        is_bin_fn: 0,
    };
    let mut parser: mpack_parser_t = mpack_parser_t {
        data: mpack_data_t {
            p: ::core::ptr::null_mut::<::core::ffi::c_void>(),
        },
        size: 0,
        capacity: 0,
        status: 0,
        exiting: 0,
        tokbuf: mpack_tokbuf_t {
            pending: [0; 9],
            pending_tok: mpack_token_t {
                type_0: 0 as mpack_token_type_t,
                length: 0,
                data: C2Rust_Unnamed {
                    value: mpack_value_t { lo: 0, hi: 0 },
                },
            },
            ppos: 0,
            plen: 0,
            passthrough: 0,
        },
        items: [mpack_node_t {
            tok: mpack_token_t {
                type_0: 0 as mpack_token_type_t,
                length: 0,
                data: C2Rust_Unnamed {
                    value: mpack_value_t { lo: 0, hi: 0 },
                },
            },
            pos: 0,
            key_visited: 0,
            data: [mpack_data_t {
                p: ::core::ptr::null_mut::<::core::ffi::c_void>(),
            }; 2],
        }; 33],
    };
    let mut buffer: luaL_Buffer = luaL_Buffer {
        p: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        lvl: 0,
        L: ::core::ptr::null_mut::<lua_State>(),
        buffer: [0; 8192],
    };
    if lua_gettop(L) != 1 as ::core::ffi::c_int {
        return luaL_error(
            L,
            b"expecting exactly 1 argument\0".as_ptr() as *const ::core::ffi::c_char,
        );
    }
    lua_createtable(L, 0 as ::core::ffi::c_int, 0 as ::core::ffi::c_int);
    packer.reg = luaL_ref(L, LUA_REGISTRYINDEX);
    packer.ext = LUA_NOREF;
    packer.parser = &raw mut parser;
    mpack_parser_init(packer.parser, 0 as mpack_uint32_t);
    (*packer.parser).data.p = &raw mut packer as *mut ::core::ffi::c_void;
    packer.is_bin = 0 as ::core::ffi::c_int;
    packer.L = L;
    packer.root = lmpack_ref(L, packer.reg);
    lua_getfield(L, LUA_REGISTRYINDEX, EMPTY_DICT_NAME.as_ptr());
    packer.mtdict = lmpack_ref(L, packer.reg);
    luaL_buffinit(L, &raw mut buffer);
    b = luaL_prepbuffer(&raw mut buffer);
    bl = LUAL_BUFFERSIZE as size_t;
    loop {
        let mut bl_init: size_t = bl;
        result = mpack_unparse(
            packer.parser,
            &raw mut b,
            &raw mut bl,
            Some(
                lmpack_unparse_enter
                    as unsafe extern "C-unwind" fn(*mut mpack_parser_t, *mut mpack_node_t) -> (),
            ),
            Some(
                lmpack_unparse_exit
                    as unsafe extern "C-unwind" fn(*mut mpack_parser_t, *mut mpack_node_t) -> (),
            ),
        );
        if result == MPACK_NOMEM as ::core::ffi::c_int {
            lmpack_unref(L, packer.reg, packer.root);
            luaL_unref(L, LUA_REGISTRYINDEX, packer.reg);
            return luaL_error(
                L,
                b"object was too deep to pack\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        buffer.p = buffer.p.offset(bl_init.wrapping_sub(bl) as isize);
        if bl == 0 {
            b = luaL_prepbuffer(&raw mut buffer);
            bl = LUAL_BUFFERSIZE as size_t;
        }
        if result != MPACK_EOF as ::core::ffi::c_int {
            break;
        }
    }
    lmpack_unref(L, packer.reg, packer.root);
    luaL_unref(L, LUA_REGISTRYINDEX, packer.reg);
    luaL_pushresult(&raw mut buffer);
    return 1 as ::core::ffi::c_int;
}
static unpacker_methods: SharedCell<[luaL_Reg; 3]> = SharedCell::new([
    luaL_Reg {
        name: b"__call\0".as_ptr() as *const ::core::ffi::c_char,
        func: Some(
            lmpack_unpacker_unpack
                as unsafe extern "C-unwind" fn(*mut lua_State) -> ::core::ffi::c_int,
        ),
    },
    luaL_Reg {
        name: b"__gc\0".as_ptr() as *const ::core::ffi::c_char,
        func: Some(
            lmpack_unpacker_delete
                as unsafe extern "C-unwind" fn(*mut lua_State) -> ::core::ffi::c_int,
        ),
    },
    luaL_Reg {
        name: ::core::ptr::null::<::core::ffi::c_char>(),
        func: None,
    },
]);
static packer_methods: SharedCell<[luaL_Reg; 3]> = SharedCell::new([
    luaL_Reg {
        name: b"__call\0".as_ptr() as *const ::core::ffi::c_char,
        func: Some(
            lmpack_packer_pack as unsafe extern "C-unwind" fn(*mut lua_State) -> ::core::ffi::c_int,
        ),
    },
    luaL_Reg {
        name: b"__gc\0".as_ptr() as *const ::core::ffi::c_char,
        func: Some(
            lmpack_packer_delete
                as unsafe extern "C-unwind" fn(*mut lua_State) -> ::core::ffi::c_int,
        ),
    },
    luaL_Reg {
        name: ::core::ptr::null::<::core::ffi::c_char>(),
        func: None,
    },
]);
static session_methods: SharedCell<[luaL_Reg; 6]> = SharedCell::new([
    luaL_Reg {
        name: b"receive\0".as_ptr() as *const ::core::ffi::c_char,
        func: Some(
            lmpack_session_receive
                as unsafe extern "C-unwind" fn(*mut lua_State) -> ::core::ffi::c_int,
        ),
    },
    luaL_Reg {
        name: b"request\0".as_ptr() as *const ::core::ffi::c_char,
        func: Some(
            lmpack_session_request
                as unsafe extern "C-unwind" fn(*mut lua_State) -> ::core::ffi::c_int,
        ),
    },
    luaL_Reg {
        name: b"reply\0".as_ptr() as *const ::core::ffi::c_char,
        func: Some(
            lmpack_session_reply
                as unsafe extern "C-unwind" fn(*mut lua_State) -> ::core::ffi::c_int,
        ),
    },
    luaL_Reg {
        name: b"notify\0".as_ptr() as *const ::core::ffi::c_char,
        func: Some(
            lmpack_session_notify
                as unsafe extern "C-unwind" fn(*mut lua_State) -> ::core::ffi::c_int,
        ),
    },
    luaL_Reg {
        name: b"__gc\0".as_ptr() as *const ::core::ffi::c_char,
        func: Some(
            lmpack_session_delete
                as unsafe extern "C-unwind" fn(*mut lua_State) -> ::core::ffi::c_int,
        ),
    },
    luaL_Reg {
        name: ::core::ptr::null::<::core::ffi::c_char>(),
        func: None,
    },
]);
static mpack_functions: SharedCell<[luaL_Reg; 6]> = SharedCell::new([
    luaL_Reg {
        name: b"Unpacker\0".as_ptr() as *const ::core::ffi::c_char,
        func: Some(
            lmpack_unpacker_new
                as unsafe extern "C-unwind" fn(*mut lua_State) -> ::core::ffi::c_int,
        ),
    },
    luaL_Reg {
        name: b"Packer\0".as_ptr() as *const ::core::ffi::c_char,
        func: Some(
            lmpack_packer_new as unsafe extern "C-unwind" fn(*mut lua_State) -> ::core::ffi::c_int,
        ),
    },
    luaL_Reg {
        name: b"Session\0".as_ptr() as *const ::core::ffi::c_char,
        func: Some(
            lmpack_session_new as unsafe extern "C-unwind" fn(*mut lua_State) -> ::core::ffi::c_int,
        ),
    },
    luaL_Reg {
        name: UNPACK_FN_NAME.as_ptr(),
        func: Some(
            lmpack_unpack as unsafe extern "C-unwind" fn(*mut lua_State) -> ::core::ffi::c_int,
        ),
    },
    luaL_Reg {
        name: PACK_FN_NAME.as_ptr(),
        func: Some(
            lmpack_pack as unsafe extern "C-unwind" fn(*mut lua_State) -> ::core::ffi::c_int,
        ),
    },
    luaL_Reg {
        name: ::core::ptr::null::<::core::ffi::c_char>(),
        func: None,
    },
]);
pub unsafe extern "C-unwind" fn luaopen_mpack(mut L: *mut lua_State) -> ::core::ffi::c_int {
    luaL_newmetatable(L, UNPACKER_META_NAME.as_ptr());
    lua_pushvalue(L, -1 as ::core::ffi::c_int);
    lua_setfield(
        L,
        -2 as ::core::ffi::c_int,
        b"__index\0".as_ptr() as *const ::core::ffi::c_char,
    );
    luaL_register(
        L,
        ::core::ptr::null::<::core::ffi::c_char>(),
        (unpacker_methods.ptr() as *const _) as *const luaL_Reg,
    );
    lua_settop(L, -1 as ::core::ffi::c_int - 1 as ::core::ffi::c_int);
    luaL_newmetatable(L, PACKER_META_NAME.as_ptr());
    lua_pushvalue(L, -1 as ::core::ffi::c_int);
    lua_setfield(
        L,
        -2 as ::core::ffi::c_int,
        b"__index\0".as_ptr() as *const ::core::ffi::c_char,
    );
    luaL_register(
        L,
        ::core::ptr::null::<::core::ffi::c_char>(),
        (packer_methods.ptr() as *const _) as *const luaL_Reg,
    );
    lua_settop(L, -1 as ::core::ffi::c_int - 1 as ::core::ffi::c_int);
    luaL_newmetatable(L, SESSION_META_NAME.as_ptr());
    lua_pushvalue(L, -1 as ::core::ffi::c_int);
    lua_setfield(
        L,
        -2 as ::core::ffi::c_int,
        b"__index\0".as_ptr() as *const ::core::ffi::c_char,
    );
    luaL_register(
        L,
        ::core::ptr::null::<::core::ffi::c_char>(),
        (session_methods.ptr() as *const _) as *const luaL_Reg,
    );
    lua_settop(L, -1 as ::core::ffi::c_int - 1 as ::core::ffi::c_int);
    lua_getfield(L, LUA_REGISTRYINDEX, NIL_NAME.as_ptr());
    if lua_type(L, -1 as ::core::ffi::c_int) == LUA_TNIL {
        lua_newuserdata(L, ::core::mem::size_of::<*mut ::core::ffi::c_void>());
        lua_createtable(L, 0 as ::core::ffi::c_int, 1 as ::core::ffi::c_int);
        lua_pushstring(L, b"__tostring\0".as_ptr() as *const ::core::ffi::c_char);
        lua_pushcclosure(
            L,
            Some(
                lmpack_nil_tostring
                    as unsafe extern "C-unwind" fn(*mut lua_State) -> ::core::ffi::c_int,
            ),
            0 as ::core::ffi::c_int,
        );
        lua_settable(L, -3 as ::core::ffi::c_int);
        lua_setmetatable(L, -2 as ::core::ffi::c_int);
        lua_setfield(L, LUA_REGISTRYINDEX, NIL_NAME.as_ptr());
    }
    lua_settop(L, -1 as ::core::ffi::c_int - 1 as ::core::ffi::c_int);
    lua_createtable(L, 0 as ::core::ffi::c_int, 0 as ::core::ffi::c_int);
    luaL_register(
        L,
        ::core::ptr::null::<::core::ffi::c_char>(),
        (mpack_functions.ptr() as *const _) as *const luaL_Reg,
    );
    lua_getfield(L, LUA_REGISTRYINDEX, NIL_NAME.as_ptr());
    lua_setfield(
        L,
        -2 as ::core::ffi::c_int,
        b"NIL\0".as_ptr() as *const ::core::ffi::c_char,
    );
    return 1 as ::core::ffi::c_int;
}
