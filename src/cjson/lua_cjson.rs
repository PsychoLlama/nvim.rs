extern "C" {
    pub type lua_State;
    fn __assert_fail(
        __assertion: *const ::core::ffi::c_char,
        __file: *const ::core::ffi::c_char,
        __line: ::core::ffi::c_uint,
        __function: *const ::core::ffi::c_char,
    ) -> !;
    fn memcpy(
        __dest: *mut ::core::ffi::c_void,
        __src: *const ::core::ffi::c_void,
        __n: size_t,
    ) -> *mut ::core::ffi::c_void;
    fn memset(
        __s: *mut ::core::ffi::c_void,
        __c: ::core::ffi::c_int,
        __n: size_t,
    ) -> *mut ::core::ffi::c_void;
    fn memcmp(
        __s1: *const ::core::ffi::c_void,
        __s2: *const ::core::ffi::c_void,
        __n: size_t,
    ) -> ::core::ffi::c_int;
    fn strncmp(
        __s1: *const ::core::ffi::c_char,
        __s2: *const ::core::ffi::c_char,
        __n: size_t,
    ) -> ::core::ffi::c_int;
    fn strncasecmp(
        __s1: *const ::core::ffi::c_char,
        __s2: *const ::core::ffi::c_char,
        __n: size_t,
    ) -> ::core::ffi::c_int;
    fn floor(__x: ::core::ffi::c_double) -> ::core::ffi::c_double;
    fn lua_gettop(L: *mut lua_State) -> ::core::ffi::c_int;
    fn lua_settop(L: *mut lua_State, idx: ::core::ffi::c_int);
    fn lua_pushvalue(L: *mut lua_State, idx: ::core::ffi::c_int);
    fn lua_insert(L: *mut lua_State, idx: ::core::ffi::c_int);
    fn lua_checkstack(L: *mut lua_State, sz: ::core::ffi::c_int) -> ::core::ffi::c_int;
    fn lua_type(L: *mut lua_State, idx: ::core::ffi::c_int) -> ::core::ffi::c_int;
    fn lua_typename(L: *mut lua_State, tp: ::core::ffi::c_int) -> *const ::core::ffi::c_char;
    fn lua_rawequal(
        L: *mut lua_State,
        idx1: ::core::ffi::c_int,
        idx2: ::core::ffi::c_int,
    ) -> ::core::ffi::c_int;
    fn lua_tonumber(L: *mut lua_State, idx: ::core::ffi::c_int) -> lua_Number;
    fn lua_tointeger(L: *mut lua_State, idx: ::core::ffi::c_int) -> lua_Integer;
    fn lua_toboolean(L: *mut lua_State, idx: ::core::ffi::c_int) -> ::core::ffi::c_int;
    fn lua_tolstring(
        L: *mut lua_State,
        idx: ::core::ffi::c_int,
        len: *mut size_t,
    ) -> *const ::core::ffi::c_char;
    fn lua_objlen(L: *mut lua_State, idx: ::core::ffi::c_int) -> size_t;
    fn lua_touserdata(L: *mut lua_State, idx: ::core::ffi::c_int) -> *mut ::core::ffi::c_void;
    fn lua_pushnil(L: *mut lua_State);
    fn lua_pushnumber(L: *mut lua_State, n: lua_Number);
    fn lua_pushinteger(L: *mut lua_State, n: lua_Integer);
    fn lua_pushlstring(L: *mut lua_State, s: *const ::core::ffi::c_char, l: size_t);
    fn lua_pushstring(L: *mut lua_State, s: *const ::core::ffi::c_char);
    fn lua_pushcclosure(L: *mut lua_State, fn_0: lua_CFunction, n: ::core::ffi::c_int);
    fn lua_pushboolean(L: *mut lua_State, b: ::core::ffi::c_int);
    fn lua_pushlightuserdata(L: *mut lua_State, p: *mut ::core::ffi::c_void);
    fn lua_gettable(L: *mut lua_State, idx: ::core::ffi::c_int);
    fn lua_getfield(L: *mut lua_State, idx: ::core::ffi::c_int, k: *const ::core::ffi::c_char);
    fn lua_rawget(L: *mut lua_State, idx: ::core::ffi::c_int);
    fn lua_rawgeti(L: *mut lua_State, idx: ::core::ffi::c_int, n: ::core::ffi::c_int);
    fn lua_createtable(L: *mut lua_State, narr: ::core::ffi::c_int, nrec: ::core::ffi::c_int);
    fn lua_newuserdata(L: *mut lua_State, sz: size_t) -> *mut ::core::ffi::c_void;
    fn lua_getmetatable(L: *mut lua_State, objindex: ::core::ffi::c_int) -> ::core::ffi::c_int;
    fn lua_setfield(L: *mut lua_State, idx: ::core::ffi::c_int, k: *const ::core::ffi::c_char);
    fn lua_rawset(L: *mut lua_State, idx: ::core::ffi::c_int);
    fn lua_rawseti(L: *mut lua_State, idx: ::core::ffi::c_int, n: ::core::ffi::c_int);
    fn lua_setmetatable(L: *mut lua_State, objindex: ::core::ffi::c_int) -> ::core::ffi::c_int;
    fn lua_call(L: *mut lua_State, nargs: ::core::ffi::c_int, nresults: ::core::ffi::c_int);
    fn lua_pcall(
        L: *mut lua_State,
        nargs: ::core::ffi::c_int,
        nresults: ::core::ffi::c_int,
        errfunc: ::core::ffi::c_int,
    ) -> ::core::ffi::c_int;
    fn lua_next(L: *mut lua_State, idx: ::core::ffi::c_int) -> ::core::ffi::c_int;
    fn luaL_getmetafield(
        L: *mut lua_State,
        obj: ::core::ffi::c_int,
        e: *const ::core::ffi::c_char,
    ) -> ::core::ffi::c_int;
    fn luaL_argerror(
        L: *mut lua_State,
        numarg: ::core::ffi::c_int,
        extramsg: *const ::core::ffi::c_char,
    ) -> ::core::ffi::c_int;
    fn luaL_checklstring(
        L: *mut lua_State,
        numArg: ::core::ffi::c_int,
        l: *mut size_t,
    ) -> *const ::core::ffi::c_char;
    fn luaL_checkstack(L: *mut lua_State, sz: ::core::ffi::c_int, msg: *const ::core::ffi::c_char);
    fn luaL_checktype(L: *mut lua_State, narg: ::core::ffi::c_int, t: ::core::ffi::c_int);
    fn luaL_error(L: *mut lua_State, fmt: *const ::core::ffi::c_char, ...) -> ::core::ffi::c_int;
    fn strtoll(
        __nptr: *const ::core::ffi::c_char,
        __endptr: *mut *mut ::core::ffi::c_char,
        __base: ::core::ffi::c_int,
    ) -> ::core::ffi::c_longlong;
    fn malloc(__size: size_t) -> *mut ::core::ffi::c_void;
    fn realloc(__ptr: *mut ::core::ffi::c_void, __size: size_t) -> *mut ::core::ffi::c_void;
    fn free(__ptr: *mut ::core::ffi::c_void);
    fn abort() -> !;
    fn qsort(
        __base: *mut ::core::ffi::c_void,
        __nmemb: size_t,
        __size: size_t,
        __compar: __compar_fn_t,
    );
    fn nlua_get_nil_ref(lstate: *mut lua_State) -> LuaRef;
    fn nlua_get_empty_dict_ref(lstate: *mut lua_State) -> LuaRef;
    fn nlua_pushref(lstate: *mut lua_State, ref_0: LuaRef);
    fn strbuf_new(len: size_t) -> *mut strbuf_t;
    fn strbuf_init(s: *mut strbuf_t, len: size_t);
    fn strbuf_free(s: *mut strbuf_t);
    fn strbuf_resize(s: *mut strbuf_t, len: size_t);
    fn strbuf_append_string(s: *mut strbuf_t, str: *const ::core::ffi::c_char);
    fn fpconv_init();
    fn fpconv_g_fmt(
        _: *mut ::core::ffi::c_char,
        _: ::core::ffi::c_double,
        _: ::core::ffi::c_int,
    ) -> ::core::ffi::c_int;
    fn fpconv_strtod(
        _: *const ::core::ffi::c_char,
        _: *mut *mut ::core::ffi::c_char,
    ) -> ::core::ffi::c_double;
}
pub type uintptr_t = usize;
pub type size_t = usize;
pub type ptrdiff_t = isize;
pub type lua_CFunction = Option<unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int>;
pub type lua_Number = ::core::ffi::c_double;
pub type lua_Integer = ptrdiff_t;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct luaL_Reg {
    pub name: *const ::core::ffi::c_char,
    pub func: lua_CFunction,
}
pub type __compar_fn_t = Option<
    unsafe extern "C" fn(
        *const ::core::ffi::c_void,
        *const ::core::ffi::c_void,
    ) -> ::core::ffi::c_int,
>;
pub type LuaRef = ::core::ffi::c_int;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct strbuf_t {
    pub buf: *mut ::core::ffi::c_char,
    pub size: size_t,
    pub length: size_t,
    pub dynamic: ::core::ffi::c_int,
    pub reallocs: ::core::ffi::c_int,
    pub debug: ::core::ffi::c_int,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct json_parse_t {
    pub data: *const ::core::ffi::c_char,
    pub ptr: *const ::core::ffi::c_char,
    pub tmp: *mut strbuf_t,
    pub cfg: *mut json_config_t,
    pub options: *mut json_options_t,
    pub current_depth: ::core::ffi::c_int,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct json_options_t {
    pub luanil_object: bool,
    pub luanil_array: bool,
    pub skip_comments: bool,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct json_config_t {
    pub ch2token: [json_token_type_t; 256],
    pub escape2char: [::core::ffi::c_char; 256],
    pub encode_buf: strbuf_t,
    pub encode_keybuf: keybuf_t,
    pub encode_sparse_convert: ::core::ffi::c_int,
    pub encode_sparse_ratio: ::core::ffi::c_int,
    pub encode_sparse_safe: ::core::ffi::c_int,
    pub encode_max_depth: ::core::ffi::c_int,
    pub encode_invalid_numbers: ::core::ffi::c_int,
    pub encode_number_precision: ::core::ffi::c_int,
    pub encode_keep_buffer: ::core::ffi::c_int,
    pub encode_empty_table_as_object: ::core::ffi::c_int,
    pub encode_escape_forward_slash: ::core::ffi::c_int,
    pub encode_indent: *const ::core::ffi::c_char,
    pub encode_sort_keys: ::core::ffi::c_int,
    pub decode_invalid_numbers: ::core::ffi::c_int,
    pub decode_max_depth: ::core::ffi::c_int,
    pub decode_array_with_array_mt: ::core::ffi::c_int,
    pub decode_skip_comments: ::core::ffi::c_int,
    pub encode_skip_unsupported_value_types: ::core::ffi::c_int,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct keybuf_t {
    pub buf: strbuf_t,
    pub keys: *mut key_entry_t,
    pub size: size_t,
    pub capacity: size_t,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct key_entry_t {
    pub buf: *mut strbuf_t,
    pub offset: size_t,
    pub length: size_t,
    pub raw_typ: ::core::ffi::c_int,
    pub raw: C2Rust_Unnamed,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2Rust_Unnamed {
    pub number: lua_Number,
    pub string: *const ::core::ffi::c_char,
}
pub type json_token_type_t = ::core::ffi::c_uint;
pub const T_UNKNOWN: json_token_type_t = 14;
pub const T_ERROR: json_token_type_t = 13;
pub const T_WHITESPACE: json_token_type_t = 12;
pub const T_END: json_token_type_t = 11;
pub const T_COMMA: json_token_type_t = 10;
pub const T_COLON: json_token_type_t = 9;
pub const T_NULL: json_token_type_t = 8;
pub const T_BOOLEAN: json_token_type_t = 7;
pub const T_INTEGER: json_token_type_t = 6;
pub const T_NUMBER: json_token_type_t = 5;
pub const T_STRING: json_token_type_t = 4;
pub const T_ARR_END: json_token_type_t = 3;
pub const T_ARR_BEGIN: json_token_type_t = 2;
pub const T_OBJ_END: json_token_type_t = 1;
pub const T_OBJ_BEGIN: json_token_type_t = 0;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct json_token_t {
    pub type_0: json_token_type_t,
    pub index: size_t,
    pub value: C2Rust_Unnamed_0,
    pub string_len: size_t,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub union C2Rust_Unnamed_0 {
    pub string: *const ::core::ffi::c_char,
    pub number: ::core::ffi::c_double,
    pub integer: lua_Integer,
    pub boolean: ::core::ffi::c_int,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct json_encode_options_t {
    pub char2escape: [*mut *const ::core::ffi::c_char; 256],
    pub indent: *const ::core::ffi::c_char,
    pub sort_keys: ::core::ffi::c_int,
    pub keybuf: keybuf_t,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct json_encode_t {
    pub cfg: *mut json_config_t,
    pub options: *mut json_encode_options_t,
    pub json: *mut strbuf_t,
}
pub const __ASSERT_FUNCTION: [::core::ffi::c_char; 60] = unsafe {
    ::core::mem::transmute::<[u8; 60], [::core::ffi::c_char; 60]>(
        *b"void json_next_string_token(json_parse_t *, json_token_t *)\0",
    )
};
pub const PTRDIFF_MIN: ::core::ffi::c_long =
    -9223372036854775807 as ::core::ffi::c_long - 1 as ::core::ffi::c_long;
pub const PTRDIFF_MAX: ::core::ffi::c_long = 9223372036854775807 as ::core::ffi::c_long;
pub const SIZE_MAX: ::core::ffi::c_ulong = 18446744073709551615 as ::core::ffi::c_ulong;
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const LUA_REGISTRYINDEX: ::core::ffi::c_int = -10000 as ::core::ffi::c_int;
pub const LUA_GLOBALSINDEX: ::core::ffi::c_int = -10002 as ::core::ffi::c_int;
pub const LUA_ERRRUN: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
pub const LUA_TNIL: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const LUA_TBOOLEAN: ::core::ffi::c_int = 1;
pub const LUA_TLIGHTUSERDATA: ::core::ffi::c_int = 2;
pub const LUA_TNUMBER: ::core::ffi::c_int = 3 as ::core::ffi::c_int;
pub const LUA_TSTRING: ::core::ffi::c_int = 4 as ::core::ffi::c_int;
pub const LUA_TTABLE: ::core::ffi::c_int = 5 as ::core::ffi::c_int;
pub const LUA_TUSERDATA: ::core::ffi::c_int = 7;
pub const DEFAULT_SPARSE_CONVERT: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const DEFAULT_SPARSE_RATIO: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
pub const DEFAULT_SPARSE_SAFE: ::core::ffi::c_int = 10 as ::core::ffi::c_int;
pub const DEFAULT_ENCODE_MAX_DEPTH: ::core::ffi::c_int = 1000 as ::core::ffi::c_int;
pub const DEFAULT_DECODE_MAX_DEPTH: ::core::ffi::c_int = 1000 as ::core::ffi::c_int;
pub const DEFAULT_ENCODE_INVALID_NUMBERS: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const DEFAULT_DECODE_INVALID_NUMBERS: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const DEFAULT_ENCODE_KEEP_BUFFER: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const DEFAULT_ENCODE_NUMBER_PRECISION: ::core::ffi::c_int = 16 as ::core::ffi::c_int;
pub const DEFAULT_ENCODE_EMPTY_TABLE_AS_OBJECT: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const DEFAULT_DECODE_ARRAY_WITH_ARRAY_MT: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const DEFAULT_DECODE_SKIP_COMMENTS: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const DEFAULT_ENCODE_ESCAPE_FORWARD_SLASH: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const DEFAULT_ENCODE_SKIP_UNSUPPORTED_VALUE_TYPES: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const DEFAULT_ENCODE_INDENT: *mut ::core::ffi::c_void = NULL;
pub const DEFAULT_ENCODE_SORT_KEYS: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
static mut json_empty_array: *const *const ::core::ffi::c_char =
    ::core::ptr::null::<*const ::core::ffi::c_char>();
static mut json_array: *const *const ::core::ffi::c_char =
    ::core::ptr::null::<*const ::core::ffi::c_char>();
static mut json_token_type_name: [*const ::core::ffi::c_char; 16] = [
    b"T_OBJ_BEGIN\0".as_ptr() as *const ::core::ffi::c_char,
    b"T_OBJ_END\0".as_ptr() as *const ::core::ffi::c_char,
    b"T_ARR_BEGIN\0".as_ptr() as *const ::core::ffi::c_char,
    b"T_ARR_END\0".as_ptr() as *const ::core::ffi::c_char,
    b"T_STRING\0".as_ptr() as *const ::core::ffi::c_char,
    b"T_NUMBER\0".as_ptr() as *const ::core::ffi::c_char,
    b"T_INTEGER\0".as_ptr() as *const ::core::ffi::c_char,
    b"T_BOOLEAN\0".as_ptr() as *const ::core::ffi::c_char,
    b"T_NULL\0".as_ptr() as *const ::core::ffi::c_char,
    b"T_COLON\0".as_ptr() as *const ::core::ffi::c_char,
    b"T_COMMA\0".as_ptr() as *const ::core::ffi::c_char,
    b"T_END\0".as_ptr() as *const ::core::ffi::c_char,
    b"T_WHITESPACE\0".as_ptr() as *const ::core::ffi::c_char,
    b"T_ERROR\0".as_ptr() as *const ::core::ffi::c_char,
    b"T_UNKNOWN\0".as_ptr() as *const ::core::ffi::c_char,
    ::core::ptr::null::<::core::ffi::c_char>(),
];
pub const KEYBUF_DEFAULT_CAPACITY: ::core::ffi::c_int = 32 as ::core::ffi::c_int;
static mut char2escape: [*const ::core::ffi::c_char; 256] = [
    b"\\u0000\0".as_ptr() as *const ::core::ffi::c_char,
    b"\\u0001\0".as_ptr() as *const ::core::ffi::c_char,
    b"\\u0002\0".as_ptr() as *const ::core::ffi::c_char,
    b"\\u0003\0".as_ptr() as *const ::core::ffi::c_char,
    b"\\u0004\0".as_ptr() as *const ::core::ffi::c_char,
    b"\\u0005\0".as_ptr() as *const ::core::ffi::c_char,
    b"\\u0006\0".as_ptr() as *const ::core::ffi::c_char,
    b"\\u0007\0".as_ptr() as *const ::core::ffi::c_char,
    b"\\b\0".as_ptr() as *const ::core::ffi::c_char,
    b"\\t\0".as_ptr() as *const ::core::ffi::c_char,
    b"\\n\0".as_ptr() as *const ::core::ffi::c_char,
    b"\\u000b\0".as_ptr() as *const ::core::ffi::c_char,
    b"\\f\0".as_ptr() as *const ::core::ffi::c_char,
    b"\\r\0".as_ptr() as *const ::core::ffi::c_char,
    b"\\u000e\0".as_ptr() as *const ::core::ffi::c_char,
    b"\\u000f\0".as_ptr() as *const ::core::ffi::c_char,
    b"\\u0010\0".as_ptr() as *const ::core::ffi::c_char,
    b"\\u0011\0".as_ptr() as *const ::core::ffi::c_char,
    b"\\u0012\0".as_ptr() as *const ::core::ffi::c_char,
    b"\\u0013\0".as_ptr() as *const ::core::ffi::c_char,
    b"\\u0014\0".as_ptr() as *const ::core::ffi::c_char,
    b"\\u0015\0".as_ptr() as *const ::core::ffi::c_char,
    b"\\u0016\0".as_ptr() as *const ::core::ffi::c_char,
    b"\\u0017\0".as_ptr() as *const ::core::ffi::c_char,
    b"\\u0018\0".as_ptr() as *const ::core::ffi::c_char,
    b"\\u0019\0".as_ptr() as *const ::core::ffi::c_char,
    b"\\u001a\0".as_ptr() as *const ::core::ffi::c_char,
    b"\\u001b\0".as_ptr() as *const ::core::ffi::c_char,
    b"\\u001c\0".as_ptr() as *const ::core::ffi::c_char,
    b"\\u001d\0".as_ptr() as *const ::core::ffi::c_char,
    b"\\u001e\0".as_ptr() as *const ::core::ffi::c_char,
    b"\\u001f\0".as_ptr() as *const ::core::ffi::c_char,
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    b"\\\"\0".as_ptr() as *const ::core::ffi::c_char,
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    b"\\\\\0".as_ptr() as *const ::core::ffi::c_char,
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    b"\\u007f\0".as_ptr() as *const ::core::ffi::c_char,
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
    ::core::ptr::null::<::core::ffi::c_char>(),
];
unsafe extern "C" fn json_fetch_config(mut l: *mut lua_State) -> *mut json_config_t {
    let mut cfg: *mut json_config_t = ::core::ptr::null_mut::<json_config_t>();
    cfg = lua_touserdata(l, LUA_GLOBALSINDEX - 1 as ::core::ffi::c_int) as *mut json_config_t;
    if cfg.is_null() {
        luaL_error(
            l,
            b"BUG: Unable to fetch CJSON configuration\0".as_ptr() as *const ::core::ffi::c_char,
        );
    }
    return cfg;
}
unsafe extern "C" fn json_destroy_config(mut l: *mut lua_State) -> ::core::ffi::c_int {
    let mut cfg: *mut json_config_t = ::core::ptr::null_mut::<json_config_t>();
    cfg = lua_touserdata(l, 1 as ::core::ffi::c_int) as *mut json_config_t;
    if !cfg.is_null() {
        strbuf_free(&raw mut (*cfg).encode_buf);
    }
    cfg = ::core::ptr::null_mut::<json_config_t>();
    return 0 as ::core::ffi::c_int;
}
unsafe extern "C" fn json_create_config(mut l: *mut lua_State) {
    let mut cfg: *mut json_config_t = ::core::ptr::null_mut::<json_config_t>();
    let mut i: ::core::ffi::c_int = 0;
    cfg = lua_newuserdata(l, ::core::mem::size_of::<json_config_t>()) as *mut json_config_t;
    if cfg.is_null() {
        abort();
    }
    memset(
        cfg as *mut ::core::ffi::c_void,
        0 as ::core::ffi::c_int,
        ::core::mem::size_of::<json_config_t>(),
    );
    lua_createtable(l, 0 as ::core::ffi::c_int, 0 as ::core::ffi::c_int);
    lua_pushcclosure(
        l,
        Some(json_destroy_config as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int),
        0 as ::core::ffi::c_int,
    );
    lua_setfield(
        l,
        -2 as ::core::ffi::c_int,
        b"__gc\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_setmetatable(l, -2 as ::core::ffi::c_int);
    (*cfg).encode_sparse_convert = DEFAULT_SPARSE_CONVERT;
    (*cfg).encode_sparse_ratio = DEFAULT_SPARSE_RATIO;
    (*cfg).encode_sparse_safe = DEFAULT_SPARSE_SAFE;
    (*cfg).encode_max_depth = DEFAULT_ENCODE_MAX_DEPTH;
    (*cfg).decode_max_depth = DEFAULT_DECODE_MAX_DEPTH;
    (*cfg).encode_invalid_numbers = DEFAULT_ENCODE_INVALID_NUMBERS;
    (*cfg).decode_invalid_numbers = DEFAULT_DECODE_INVALID_NUMBERS;
    (*cfg).encode_keep_buffer = DEFAULT_ENCODE_KEEP_BUFFER;
    (*cfg).encode_number_precision = DEFAULT_ENCODE_NUMBER_PRECISION;
    (*cfg).encode_empty_table_as_object = DEFAULT_ENCODE_EMPTY_TABLE_AS_OBJECT;
    (*cfg).decode_array_with_array_mt = DEFAULT_DECODE_ARRAY_WITH_ARRAY_MT;
    (*cfg).decode_skip_comments = DEFAULT_DECODE_SKIP_COMMENTS;
    (*cfg).encode_escape_forward_slash = DEFAULT_ENCODE_ESCAPE_FORWARD_SLASH;
    (*cfg).encode_skip_unsupported_value_types = DEFAULT_ENCODE_SKIP_UNSUPPORTED_VALUE_TYPES;
    (*cfg).encode_indent = ::core::ptr::null::<::core::ffi::c_char>();
    (*cfg).encode_sort_keys = DEFAULT_ENCODE_SORT_KEYS;
    strbuf_init(&raw mut (*cfg).encode_buf, 0 as size_t);
    i = 0 as ::core::ffi::c_int;
    while i < 256 as ::core::ffi::c_int {
        (*cfg).ch2token[i as usize] = T_ERROR;
        i += 1;
    }
    (*cfg).ch2token['{' as ::core::ffi::c_int as usize] = T_OBJ_BEGIN;
    (*cfg).ch2token['}' as ::core::ffi::c_int as usize] = T_OBJ_END;
    (*cfg).ch2token['[' as ::core::ffi::c_int as usize] = T_ARR_BEGIN;
    (*cfg).ch2token[']' as ::core::ffi::c_int as usize] = T_ARR_END;
    (*cfg).ch2token[',' as ::core::ffi::c_int as usize] = T_COMMA;
    (*cfg).ch2token[':' as ::core::ffi::c_int as usize] = T_COLON;
    (*cfg).ch2token['\0' as ::core::ffi::c_int as usize] = T_END;
    (*cfg).ch2token[' ' as ::core::ffi::c_int as usize] = T_WHITESPACE;
    (*cfg).ch2token['\t' as ::core::ffi::c_int as usize] = T_WHITESPACE;
    (*cfg).ch2token['\n' as ::core::ffi::c_int as usize] = T_WHITESPACE;
    (*cfg).ch2token['\r' as ::core::ffi::c_int as usize] = T_WHITESPACE;
    (*cfg).ch2token['f' as ::core::ffi::c_int as usize] = T_UNKNOWN;
    (*cfg).ch2token['i' as ::core::ffi::c_int as usize] = T_UNKNOWN;
    (*cfg).ch2token['I' as ::core::ffi::c_int as usize] = T_UNKNOWN;
    (*cfg).ch2token['n' as ::core::ffi::c_int as usize] = T_UNKNOWN;
    (*cfg).ch2token['N' as ::core::ffi::c_int as usize] = T_UNKNOWN;
    (*cfg).ch2token['t' as ::core::ffi::c_int as usize] = T_UNKNOWN;
    (*cfg).ch2token['"' as ::core::ffi::c_int as usize] = T_UNKNOWN;
    (*cfg).ch2token['+' as ::core::ffi::c_int as usize] = T_UNKNOWN;
    (*cfg).ch2token['-' as ::core::ffi::c_int as usize] = T_UNKNOWN;
    i = 0 as ::core::ffi::c_int;
    while i < 10 as ::core::ffi::c_int {
        (*cfg).ch2token[('0' as ::core::ffi::c_int + i) as usize] = T_UNKNOWN;
        i += 1;
    }
    i = 0 as ::core::ffi::c_int;
    while i < 256 as ::core::ffi::c_int {
        (*cfg).escape2char[i as usize] = 0 as ::core::ffi::c_char;
        i += 1;
    }
    (*cfg).escape2char['"' as ::core::ffi::c_int as usize] = '"' as ::core::ffi::c_char;
    (*cfg).escape2char['\\' as ::core::ffi::c_int as usize] = '\\' as ::core::ffi::c_char;
    (*cfg).escape2char['/' as ::core::ffi::c_int as usize] = '/' as ::core::ffi::c_char;
    (*cfg).escape2char['b' as ::core::ffi::c_int as usize] = '\u{8}' as ::core::ffi::c_char;
    (*cfg).escape2char['t' as ::core::ffi::c_int as usize] = '\t' as ::core::ffi::c_char;
    (*cfg).escape2char['n' as ::core::ffi::c_int as usize] = '\n' as ::core::ffi::c_char;
    (*cfg).escape2char['f' as ::core::ffi::c_int as usize] = '\u{c}' as ::core::ffi::c_char;
    (*cfg).escape2char['r' as ::core::ffi::c_int as usize] = '\r' as ::core::ffi::c_char;
    (*cfg).escape2char['u' as ::core::ffi::c_int as usize] = 'u' as ::core::ffi::c_char;
}
unsafe extern "C" fn json_encode_exception(
    mut l: *mut lua_State,
    mut ctx: *mut json_encode_t,
    mut lindex: ::core::ffi::c_int,
    mut reason: *const ::core::ffi::c_char,
) {
    if (*(*ctx).cfg).encode_keep_buffer == 0 {
        strbuf_free((*ctx).json);
    }
    if (*(*ctx).options).sort_keys != 0 {
        strbuf_free(&raw mut (*(*ctx).options).keybuf.buf);
        free((*(*ctx).options).keybuf.keys as *mut ::core::ffi::c_void);
    }
    luaL_error(
        l,
        b"Cannot serialise %s: %s\0".as_ptr() as *const ::core::ffi::c_char,
        lua_typename(l, lua_type(l, lindex)),
        reason,
    );
}
unsafe extern "C" fn json_append_string_contents(
    mut l: *mut lua_State,
    mut ctx: *mut json_encode_t,
    mut lindex: ::core::ffi::c_int,
    mut use_keybuf: ::core::ffi::c_int,
) {
    let mut escstr: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut str: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut len: size_t = 0;
    let mut i: size_t = 0;
    let mut json: *mut strbuf_t = (*ctx).json;
    if use_keybuf != 0 {
        json = &raw mut (*(*ctx).options).keybuf.buf;
    }
    str = lua_tolstring(l, lindex, &raw mut len);
    if len >= (SIZE_MAX as size_t).wrapping_div(6 as size_t) {
        abort();
    }
    strbuf_ensure_empty_length(json, len.wrapping_mul(6 as size_t));
    i = 0 as size_t;
    while i < len {
        escstr = *(*(&raw mut (*(*ctx).options).char2escape
            as *mut *mut *const ::core::ffi::c_char))
            .offset(*str.offset(i as isize) as ::core::ffi::c_uchar as isize);
        if !escstr.is_null() {
            strbuf_append_string(json, escstr);
        } else {
            strbuf_append_char_unsafe(json, *str.offset(i as isize));
        }
        i = i.wrapping_add(1);
    }
}
unsafe extern "C" fn json_append_string(
    mut l: *mut lua_State,
    mut ctx: *mut json_encode_t,
    mut lindex: ::core::ffi::c_int,
) {
    strbuf_append_char((*ctx).json, '"' as ::core::ffi::c_char);
    json_append_string_contents(l, ctx, lindex, false_0);
    strbuf_append_char((*ctx).json, '"' as ::core::ffi::c_char);
}
unsafe extern "C" fn lua_array_length(
    mut l: *mut lua_State,
    mut ctx: *mut json_encode_t,
) -> ::core::ffi::c_int {
    let mut k: ::core::ffi::c_double = 0.;
    let mut max: ::core::ffi::c_int = 0;
    let mut items: ::core::ffi::c_int = 0;
    let mut cfg: *mut json_config_t = (*ctx).cfg;
    max = 0 as ::core::ffi::c_int;
    items = 0 as ::core::ffi::c_int;
    lua_pushnil(l);
    while lua_next(l, -2 as ::core::ffi::c_int) != 0 as ::core::ffi::c_int {
        if lua_type(l, -2 as ::core::ffi::c_int) == LUA_TNUMBER && {
            k = lua_tonumber(l, -2 as ::core::ffi::c_int) as ::core::ffi::c_double;
            k != 0.
        } {
            if floor(k) == k && k >= 1 as ::core::ffi::c_int as ::core::ffi::c_double {
                if k > max as ::core::ffi::c_double {
                    max = k as ::core::ffi::c_int;
                }
                items += 1;
                lua_settop(l, -1 as ::core::ffi::c_int - 1 as ::core::ffi::c_int);
                continue;
            }
        }
        lua_settop(l, -2 as ::core::ffi::c_int - 1 as ::core::ffi::c_int);
        return -1 as ::core::ffi::c_int;
    }
    if (*cfg).encode_sparse_ratio > 0 as ::core::ffi::c_int
        && max > items * (*cfg).encode_sparse_ratio
        && max > (*cfg).encode_sparse_safe
    {
        if (*cfg).encode_sparse_convert == 0 {
            json_encode_exception(
                l,
                ctx,
                -1 as ::core::ffi::c_int,
                b"excessively sparse array\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        return -1 as ::core::ffi::c_int;
    }
    return max;
}
unsafe extern "C" fn json_check_encode_depth(
    mut l: *mut lua_State,
    mut cfg: *mut json_config_t,
    mut current_depth: ::core::ffi::c_int,
    mut json: *mut strbuf_t,
) {
    if current_depth <= (*cfg).encode_max_depth && lua_checkstack(l, 3 as ::core::ffi::c_int) != 0 {
        return;
    }
    if (*cfg).encode_keep_buffer == 0 {
        strbuf_free(json);
    }
    luaL_error(
        l,
        b"Cannot serialise, excessive nesting (%d)\0".as_ptr() as *const ::core::ffi::c_char,
        current_depth,
    );
}
unsafe extern "C" fn json_append_newline_and_indent(
    mut json: *mut strbuf_t,
    mut ctx: *mut json_encode_t,
    mut depth: ::core::ffi::c_int,
) {
    strbuf_append_char(json, '\n' as ::core::ffi::c_char);
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i < depth {
        strbuf_append_string(json, (*(*ctx).options).indent);
        i += 1;
    }
}
unsafe extern "C" fn json_append_array(
    mut l: *mut lua_State,
    mut ctx: *mut json_encode_t,
    mut current_depth: ::core::ffi::c_int,
    mut array_length: ::core::ffi::c_int,
    mut raw: ::core::ffi::c_int,
) {
    let mut comma: ::core::ffi::c_int = 0;
    let mut i: ::core::ffi::c_int = 0;
    let mut json_pos: ::core::ffi::c_int = 0;
    let mut err: ::core::ffi::c_int = 0;
    let mut has_items: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut json: *mut strbuf_t = (*ctx).json;
    strbuf_append_char(json, '[' as ::core::ffi::c_char);
    comma = 0 as ::core::ffi::c_int;
    i = 1 as ::core::ffi::c_int;
    while i <= array_length {
        has_items = 1 as ::core::ffi::c_int;
        json_pos = strbuf_length(json) as ::core::ffi::c_int;
        let c2rust_fresh1 = comma;
        comma = comma + 1;
        if c2rust_fresh1 > 0 as ::core::ffi::c_int {
            strbuf_append_char(json, ',' as ::core::ffi::c_char);
        }
        if !(*(*ctx).options).indent.is_null() {
            json_append_newline_and_indent(json, ctx, current_depth);
        }
        if raw != 0 {
            lua_rawgeti(l, -1 as ::core::ffi::c_int, i);
        } else {
            lua_pushinteger(l, i as lua_Integer);
            lua_gettable(l, -2 as ::core::ffi::c_int);
        }
        err = json_append_data(l, ctx, current_depth);
        if err != 0 {
            strbuf_set_length(json, json_pos);
            if comma == 1 as ::core::ffi::c_int {
                comma = 0 as ::core::ffi::c_int;
            }
        }
        lua_settop(l, -1 as ::core::ffi::c_int - 1 as ::core::ffi::c_int);
        i += 1;
    }
    if has_items != 0 && !(*(*ctx).options).indent.is_null() {
        json_append_newline_and_indent(json, ctx, current_depth - 1 as ::core::ffi::c_int);
    }
    strbuf_append_char(json, ']' as ::core::ffi::c_char);
}
unsafe extern "C" fn json_append_number(
    mut l: *mut lua_State,
    mut ctx: *mut json_encode_t,
    mut lindex: ::core::ffi::c_int,
    mut use_keybuf: ::core::ffi::c_int,
) {
    let mut len: ::core::ffi::c_int = 0;
    let mut num: ::core::ffi::c_double = lua_tonumber(l, lindex);
    let mut cfg: *mut json_config_t = (*ctx).cfg;
    let mut json: *mut strbuf_t = (*ctx).json;
    if use_keybuf != 0 {
        json = &raw mut (*(*ctx).options).keybuf.buf;
    }
    if (*cfg).encode_invalid_numbers == 0 as ::core::ffi::c_int {
        if if num.is_infinite() {
            if num.is_sign_positive() {
                1
            } else {
                -1
            }
        } else {
            0
        } != 0
            || num.is_nan() as i32 != 0
        {
            json_encode_exception(
                l,
                ctx,
                lindex,
                b"must not be NaN or Infinity\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
    } else if (*cfg).encode_invalid_numbers == 1 as ::core::ffi::c_int {
        if num.is_nan() as i32 != 0 {
            strbuf_append_mem(
                json,
                b"NaN\0".as_ptr() as *const ::core::ffi::c_char,
                3 as size_t,
            );
            return;
        }
        if if num.is_infinite() {
            if num.is_sign_positive() {
                1
            } else {
                -1
            }
        } else {
            0
        } != 0
        {
            if num < 0 as ::core::ffi::c_int as ::core::ffi::c_double {
                strbuf_append_mem(
                    json,
                    b"-Infinity\0".as_ptr() as *const ::core::ffi::c_char,
                    9 as size_t,
                );
            } else {
                strbuf_append_mem(
                    json,
                    b"Infinity\0".as_ptr() as *const ::core::ffi::c_char,
                    8 as size_t,
                );
            }
            return;
        }
    } else if if num.is_infinite() {
        if num.is_sign_positive() {
            1
        } else {
            -1
        }
    } else {
        0
    } != 0
        || num.is_nan() as i32 != 0
    {
        strbuf_append_mem(
            json,
            b"null\0".as_ptr() as *const ::core::ffi::c_char,
            4 as size_t,
        );
        return;
    }
    strbuf_ensure_empty_length(json, FPCONV_G_FMT_BUFSIZE as size_t);
    len = fpconv_g_fmt(strbuf_empty_ptr(json), num, (*cfg).encode_number_precision);
    strbuf_extend_length(json, len as size_t);
}
unsafe extern "C" fn cmp_key_entries(
    mut a: *const ::core::ffi::c_void,
    mut b: *const ::core::ffi::c_void,
) -> ::core::ffi::c_int {
    let mut ka: *const key_entry_t = a as *const key_entry_t;
    let mut kb: *const key_entry_t = b as *const key_entry_t;
    let mut res: ::core::ffi::c_int = memcmp(
        (*(*ka).buf).buf.offset((*ka).offset as isize) as *const ::core::ffi::c_void,
        (*(*kb).buf).buf.offset((*kb).offset as isize) as *const ::core::ffi::c_void,
        if (*ka).length < (*kb).length {
            (*ka).length
        } else {
            (*kb).length
        },
    );
    if res == 0 as ::core::ffi::c_int {
        return (*ka).length.wrapping_sub((*kb).length) as ::core::ffi::c_int;
    }
    return res;
}
unsafe extern "C" fn json_append_object(
    mut l: *mut lua_State,
    mut ctx: *mut json_encode_t,
    mut current_depth: ::core::ffi::c_int,
) {
    let mut comma: ::core::ffi::c_int = 0;
    let mut keytype: ::core::ffi::c_int = 0;
    let mut json_pos: ::core::ffi::c_int = 0;
    let mut err: ::core::ffi::c_int = 0;
    let mut has_items: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut json: *mut strbuf_t = (*ctx).json;
    strbuf_append_char(json, '{' as ::core::ffi::c_char);
    comma = 0 as ::core::ffi::c_int;
    lua_pushnil(l);
    if (*(*ctx).options).sort_keys != 0 {
        let mut keybuf: *mut keybuf_t = &raw mut (*(*ctx).options).keybuf;
        let mut init_keybuf_size: size_t = (*keybuf).size;
        let mut init_keybuf_length: size_t = strbuf_length(&raw mut (*keybuf).buf);
        while lua_next(l, -2 as ::core::ffi::c_int) != 0 as ::core::ffi::c_int {
            has_items = 1 as ::core::ffi::c_int;
            if (*keybuf).size == (*keybuf).capacity {
                (*keybuf).capacity = (*keybuf).capacity.wrapping_mul(2 as size_t);
                let mut tmp: *mut key_entry_t = realloc(
                    (*keybuf).keys as *mut ::core::ffi::c_void,
                    (*keybuf)
                        .capacity
                        .wrapping_mul(::core::mem::size_of::<key_entry_t>()),
                ) as *mut key_entry_t;
                if tmp.is_null() {
                    json_encode_exception(
                        l,
                        ctx,
                        -1 as ::core::ffi::c_int,
                        b"out of memory\0".as_ptr() as *const ::core::ffi::c_char,
                    );
                }
                (*keybuf).keys = tmp;
            }
            keytype = lua_type(l, -2 as ::core::ffi::c_int);
            let mut key_entry: key_entry_t = key_entry_t {
                buf: &raw mut (*keybuf).buf,
                offset: strbuf_length(&raw mut (*keybuf).buf),
                length: 0,
                raw_typ: keytype,
                raw: C2Rust_Unnamed { number: 0. },
            };
            if keytype == LUA_TSTRING {
                json_append_string_contents(l, ctx, -2 as ::core::ffi::c_int, true_0);
                key_entry.raw.string = lua_tolstring(
                    l,
                    -2 as ::core::ffi::c_int,
                    ::core::ptr::null_mut::<size_t>(),
                );
            } else if keytype == LUA_TNUMBER {
                json_append_number(l, ctx, -2 as ::core::ffi::c_int, true_0);
                key_entry.raw.number = lua_tointeger(l, -2 as ::core::ffi::c_int) as lua_Number;
            } else {
                json_encode_exception(
                    l,
                    ctx,
                    -2 as ::core::ffi::c_int,
                    b"table key must be number or string\0".as_ptr() as *const ::core::ffi::c_char,
                );
            }
            key_entry.length = strbuf_length(&raw mut (*keybuf).buf).wrapping_sub(key_entry.offset);
            let c2rust_fresh3 = (*keybuf).size;
            (*keybuf).size = (*keybuf).size.wrapping_add(1);
            *(*keybuf).keys.offset(c2rust_fresh3 as isize) = key_entry;
            lua_settop(l, -1 as ::core::ffi::c_int - 1 as ::core::ffi::c_int);
        }
        let mut keys_count: size_t = (*keybuf).size.wrapping_sub(init_keybuf_size);
        qsort(
            (*keybuf).keys.offset(init_keybuf_size as isize) as *mut ::core::ffi::c_void,
            keys_count,
            ::core::mem::size_of::<key_entry_t>(),
            Some(
                cmp_key_entries
                    as unsafe extern "C" fn(
                        *const ::core::ffi::c_void,
                        *const ::core::ffi::c_void,
                    ) -> ::core::ffi::c_int,
            ),
        );
        let mut i: size_t = init_keybuf_size;
        while i < init_keybuf_size.wrapping_add(keys_count) {
            let mut current_key: *mut key_entry_t = (*keybuf).keys.offset(i as isize);
            json_pos = strbuf_length(json) as ::core::ffi::c_int;
            let c2rust_fresh4 = comma;
            comma = comma + 1;
            if c2rust_fresh4 > 0 as ::core::ffi::c_int {
                strbuf_append_char(json, ',' as ::core::ffi::c_char);
            }
            if !(*(*ctx).options).indent.is_null() {
                json_append_newline_and_indent(json, ctx, current_depth);
            }
            strbuf_ensure_empty_length(json, (*current_key).length.wrapping_add(3 as size_t));
            strbuf_append_char_unsafe(json, '"' as ::core::ffi::c_char);
            strbuf_append_mem_unsafe(
                json,
                (*keybuf).buf.buf.offset((*current_key).offset as isize),
                (*current_key).length,
            );
            strbuf_append_mem_unsafe(
                json,
                b"\":\0".as_ptr() as *const ::core::ffi::c_char,
                2 as size_t,
            );
            if !(*(*ctx).options).indent.is_null() {
                strbuf_append_char(json, ' ' as ::core::ffi::c_char);
            }
            if (*current_key).raw_typ == LUA_TSTRING {
                lua_pushstring(l, (*current_key).raw.string);
            } else {
                lua_pushnumber(l, (*current_key).raw.number);
            }
            lua_gettable(l, -2 as ::core::ffi::c_int);
            err = json_append_data(l, ctx, current_depth);
            if err != 0 {
                strbuf_set_length(json, json_pos);
                if comma == 1 as ::core::ffi::c_int {
                    comma = 0 as ::core::ffi::c_int;
                }
            }
            lua_settop(l, -1 as ::core::ffi::c_int - 1 as ::core::ffi::c_int);
            i = i.wrapping_add(1);
        }
        strbuf_set_length(
            &raw mut (*keybuf).buf,
            init_keybuf_length as ::core::ffi::c_int,
        );
        (*keybuf).size = init_keybuf_size;
    } else {
        while lua_next(l, -2 as ::core::ffi::c_int) != 0 as ::core::ffi::c_int {
            has_items = 1 as ::core::ffi::c_int;
            json_pos = strbuf_length(json) as ::core::ffi::c_int;
            let c2rust_fresh5 = comma;
            comma = comma + 1;
            if c2rust_fresh5 > 0 as ::core::ffi::c_int {
                strbuf_append_char(json, ',' as ::core::ffi::c_char);
            }
            if !(*(*ctx).options).indent.is_null() {
                json_append_newline_and_indent(json, ctx, current_depth);
            }
            keytype = lua_type(l, -2 as ::core::ffi::c_int);
            if keytype == LUA_TNUMBER {
                strbuf_append_char(json, '"' as ::core::ffi::c_char);
                json_append_number(l, ctx, -2 as ::core::ffi::c_int, false_0);
                strbuf_append_mem(
                    json,
                    b"\":\0".as_ptr() as *const ::core::ffi::c_char,
                    2 as size_t,
                );
            } else if keytype == LUA_TSTRING {
                json_append_string(l, ctx, -2 as ::core::ffi::c_int);
                strbuf_append_char(json, ':' as ::core::ffi::c_char);
            } else {
                json_encode_exception(
                    l,
                    ctx,
                    -2 as ::core::ffi::c_int,
                    b"table key must be a number or string\0".as_ptr()
                        as *const ::core::ffi::c_char,
                );
            }
            if !(*(*ctx).options).indent.is_null() {
                strbuf_append_char(json, ' ' as ::core::ffi::c_char);
            }
            err = json_append_data(l, ctx, current_depth);
            if err != 0 {
                strbuf_set_length(json, json_pos);
                if comma == 1 as ::core::ffi::c_int {
                    comma = 0 as ::core::ffi::c_int;
                }
            }
            lua_settop(l, -1 as ::core::ffi::c_int - 1 as ::core::ffi::c_int);
        }
    }
    if has_items != 0 && !(*(*ctx).options).indent.is_null() {
        json_append_newline_and_indent(json, ctx, current_depth - 1 as ::core::ffi::c_int);
    }
    strbuf_append_char(json, '}' as ::core::ffi::c_char);
}
unsafe extern "C" fn json_append_data(
    mut l: *mut lua_State,
    mut ctx: *mut json_encode_t,
    mut current_depth: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let mut len: ::core::ffi::c_int = 0;
    let mut as_array: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut as_empty_dict: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut has_metatable: ::core::ffi::c_int = 0;
    let mut raw: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
    let mut cfg: *mut json_config_t = (*ctx).cfg;
    let mut json: *mut strbuf_t = (*ctx).json;
    let mut is_nil: bool = false;
    's_309: {
        match lua_type(l, -1 as ::core::ffi::c_int) {
            LUA_TSTRING => {
                json_append_string(l, ctx, -1 as ::core::ffi::c_int);
                break 's_309;
            }
            LUA_TNUMBER => {
                json_append_number(l, ctx, -1 as ::core::ffi::c_int, false_0);
                break 's_309;
            }
            LUA_TBOOLEAN => {
                if lua_toboolean(l, -1 as ::core::ffi::c_int) != 0 {
                    strbuf_append_mem(
                        json,
                        b"true\0".as_ptr() as *const ::core::ffi::c_char,
                        4 as size_t,
                    );
                } else {
                    strbuf_append_mem(
                        json,
                        b"false\0".as_ptr() as *const ::core::ffi::c_char,
                        5 as size_t,
                    );
                }
                break 's_309;
            }
            LUA_TTABLE => {
                current_depth += 1;
                json_check_encode_depth(l, cfg, current_depth, json);
                has_metatable = lua_getmetatable(l, -1 as ::core::ffi::c_int);
                if has_metatable != 0 {
                    nlua_pushref(l, nlua_get_empty_dict_ref(l));
                    if lua_rawequal(l, -2 as ::core::ffi::c_int, -1 as ::core::ffi::c_int) != 0 {
                        as_empty_dict = true_0;
                    } else {
                        lua_settop(l, -1 as ::core::ffi::c_int - 1 as ::core::ffi::c_int);
                        lua_pushlightuserdata(
                            l,
                            ::core::ptr::from_exposed_addr_mut::<::core::ffi::c_void>(
                                ((&raw mut json_array).expose_addr() as uintptr_t
                                    & ((1 as uintptr_t) << 47 as ::core::ffi::c_int)
                                        .wrapping_sub(1 as ::core::ffi::c_int as uintptr_t))
                                    as usize,
                            ),
                        );
                        lua_rawget(l, LUA_REGISTRYINDEX);
                        as_array =
                            lua_rawequal(l, -1 as ::core::ffi::c_int, -2 as ::core::ffi::c_int);
                    }
                    if as_array != 0 {
                        raw = 1 as ::core::ffi::c_int;
                        lua_settop(l, -2 as ::core::ffi::c_int - 1 as ::core::ffi::c_int);
                        len = lua_objlen(l, -1 as ::core::ffi::c_int) as ::core::ffi::c_int;
                    } else {
                        raw = 0 as ::core::ffi::c_int;
                        lua_settop(l, -2 as ::core::ffi::c_int - 1 as ::core::ffi::c_int);
                        if luaL_getmetafield(
                            l,
                            -1 as ::core::ffi::c_int,
                            b"__len\0".as_ptr() as *const ::core::ffi::c_char,
                        ) != 0
                        {
                            lua_pushvalue(l, -2 as ::core::ffi::c_int);
                            lua_call(l, 1 as ::core::ffi::c_int, 1 as ::core::ffi::c_int);
                            len = lua_tonumber(l, -1 as ::core::ffi::c_int) as ::core::ffi::c_int;
                            lua_settop(l, -1 as ::core::ffi::c_int - 1 as ::core::ffi::c_int);
                            as_array = 1 as ::core::ffi::c_int;
                        }
                    }
                }
                if as_array != 0 {
                    len = lua_objlen(l, -1 as ::core::ffi::c_int) as ::core::ffi::c_int;
                    json_append_array(l, ctx, current_depth, len, raw);
                    break 's_309;
                } else {
                    len = lua_array_length(l, ctx);
                    if len > 0 as ::core::ffi::c_int
                        || len == 0 as ::core::ffi::c_int
                            && (*cfg).encode_empty_table_as_object == 0
                            && as_empty_dict == 0
                    {
                        json_append_array(l, ctx, current_depth, len, raw);
                        break 's_309;
                    } else {
                        if has_metatable != 0 {
                            lua_getmetatable(l, -1 as ::core::ffi::c_int);
                            lua_pushlightuserdata(
                                l,
                                ::core::ptr::from_exposed_addr_mut::<::core::ffi::c_void>(
                                    ((&raw mut json_empty_array).expose_addr() as uintptr_t
                                        & ((1 as uintptr_t) << 47 as ::core::ffi::c_int)
                                            .wrapping_sub(1 as ::core::ffi::c_int as uintptr_t))
                                        as usize,
                                ),
                            );
                            lua_rawget(l, LUA_REGISTRYINDEX);
                            as_array =
                                lua_rawequal(l, -1 as ::core::ffi::c_int, -2 as ::core::ffi::c_int);
                            lua_settop(l, -2 as ::core::ffi::c_int - 1 as ::core::ffi::c_int);
                            if as_array != 0 {
                                len = lua_objlen(l, -1 as ::core::ffi::c_int) as ::core::ffi::c_int;
                                raw = 1 as ::core::ffi::c_int;
                                json_append_array(l, ctx, current_depth, len, raw);
                                break 's_309;
                            }
                        }
                        json_append_object(l, ctx, current_depth);
                        break 's_309;
                    }
                }
            }
            LUA_TNIL => {
                strbuf_append_mem(
                    json,
                    b"null\0".as_ptr() as *const ::core::ffi::c_char,
                    4 as size_t,
                );
                break 's_309;
            }
            LUA_TLIGHTUSERDATA => {
                if lua_touserdata(l, -1 as ::core::ffi::c_int)
                    == &raw mut json_array as *mut ::core::ffi::c_void
                {
                    json_append_array(
                        l,
                        ctx,
                        current_depth,
                        0 as ::core::ffi::c_int,
                        1 as ::core::ffi::c_int,
                    );
                }
                break 's_309;
            }
            LUA_TUSERDATA => {
                nlua_pushref(l, nlua_get_nil_ref(l));
                is_nil = lua_rawequal(l, -2 as ::core::ffi::c_int, -1 as ::core::ffi::c_int) != 0;
                lua_settop(l, -1 as ::core::ffi::c_int - 1 as ::core::ffi::c_int);
                if is_nil {
                    strbuf_append_mem(
                        json,
                        b"null\0".as_ptr() as *const ::core::ffi::c_char,
                        4 as size_t,
                    );
                    break 's_309;
                }
            }
            _ => {}
        }
        if (*cfg).encode_skip_unsupported_value_types != 0 {
            return 1 as ::core::ffi::c_int;
        } else {
            json_encode_exception(
                l,
                ctx,
                -1 as ::core::ffi::c_int,
                b"type not supported\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
    }
    return 0 as ::core::ffi::c_int;
}
unsafe extern "C" fn json_encode(mut l: *mut lua_State) -> ::core::ffi::c_int {
    let mut cfg: *mut json_config_t = json_fetch_config(l);
    let mut options: json_encode_options_t = json_encode_options_t {
        char2escape: [
            &raw mut char2escape as *mut *const ::core::ffi::c_char,
            ::core::ptr::null_mut::<*const ::core::ffi::c_char>(),
            ::core::ptr::null_mut::<*const ::core::ffi::c_char>(),
            ::core::ptr::null_mut::<*const ::core::ffi::c_char>(),
            ::core::ptr::null_mut::<*const ::core::ffi::c_char>(),
            ::core::ptr::null_mut::<*const ::core::ffi::c_char>(),
            ::core::ptr::null_mut::<*const ::core::ffi::c_char>(),
            ::core::ptr::null_mut::<*const ::core::ffi::c_char>(),
            ::core::ptr::null_mut::<*const ::core::ffi::c_char>(),
            ::core::ptr::null_mut::<*const ::core::ffi::c_char>(),
            ::core::ptr::null_mut::<*const ::core::ffi::c_char>(),
            ::core::ptr::null_mut::<*const ::core::ffi::c_char>(),
            ::core::ptr::null_mut::<*const ::core::ffi::c_char>(),
            ::core::ptr::null_mut::<*const ::core::ffi::c_char>(),
            ::core::ptr::null_mut::<*const ::core::ffi::c_char>(),
            ::core::ptr::null_mut::<*const ::core::ffi::c_char>(),
            ::core::ptr::null_mut::<*const ::core::ffi::c_char>(),
            ::core::ptr::null_mut::<*const ::core::ffi::c_char>(),
            ::core::ptr::null_mut::<*const ::core::ffi::c_char>(),
            ::core::ptr::null_mut::<*const ::core::ffi::c_char>(),
            ::core::ptr::null_mut::<*const ::core::ffi::c_char>(),
            ::core::ptr::null_mut::<*const ::core::ffi::c_char>(),
            ::core::ptr::null_mut::<*const ::core::ffi::c_char>(),
            ::core::ptr::null_mut::<*const ::core::ffi::c_char>(),
            ::core::ptr::null_mut::<*const ::core::ffi::c_char>(),
            ::core::ptr::null_mut::<*const ::core::ffi::c_char>(),
            ::core::ptr::null_mut::<*const ::core::ffi::c_char>(),
            ::core::ptr::null_mut::<*const ::core::ffi::c_char>(),
            ::core::ptr::null_mut::<*const ::core::ffi::c_char>(),
            ::core::ptr::null_mut::<*const ::core::ffi::c_char>(),
            ::core::ptr::null_mut::<*const ::core::ffi::c_char>(),
            ::core::ptr::null_mut::<*const ::core::ffi::c_char>(),
            ::core::ptr::null_mut::<*const ::core::ffi::c_char>(),
            ::core::ptr::null_mut::<*const ::core::ffi::c_char>(),
            ::core::ptr::null_mut::<*const ::core::ffi::c_char>(),
            ::core::ptr::null_mut::<*const ::core::ffi::c_char>(),
            ::core::ptr::null_mut::<*const ::core::ffi::c_char>(),
            ::core::ptr::null_mut::<*const ::core::ffi::c_char>(),
            ::core::ptr::null_mut::<*const ::core::ffi::c_char>(),
            ::core::ptr::null_mut::<*const ::core::ffi::c_char>(),
            ::core::ptr::null_mut::<*const ::core::ffi::c_char>(),
            ::core::ptr::null_mut::<*const ::core::ffi::c_char>(),
            ::core::ptr::null_mut::<*const ::core::ffi::c_char>(),
            ::core::ptr::null_mut::<*const ::core::ffi::c_char>(),
            ::core::ptr::null_mut::<*const ::core::ffi::c_char>(),
            ::core::ptr::null_mut::<*const ::core::ffi::c_char>(),
            ::core::ptr::null_mut::<*const ::core::ffi::c_char>(),
            ::core::ptr::null_mut::<*const ::core::ffi::c_char>(),
            ::core::ptr::null_mut::<*const ::core::ffi::c_char>(),
            ::core::ptr::null_mut::<*const ::core::ffi::c_char>(),
            ::core::ptr::null_mut::<*const ::core::ffi::c_char>(),
            ::core::ptr::null_mut::<*const ::core::ffi::c_char>(),
            ::core::ptr::null_mut::<*const ::core::ffi::c_char>(),
            ::core::ptr::null_mut::<*const ::core::ffi::c_char>(),
            ::core::ptr::null_mut::<*const ::core::ffi::c_char>(),
            ::core::ptr::null_mut::<*const ::core::ffi::c_char>(),
            ::core::ptr::null_mut::<*const ::core::ffi::c_char>(),
            ::core::ptr::null_mut::<*const ::core::ffi::c_char>(),
            ::core::ptr::null_mut::<*const ::core::ffi::c_char>(),
            ::core::ptr::null_mut::<*const ::core::ffi::c_char>(),
            ::core::ptr::null_mut::<*const ::core::ffi::c_char>(),
            ::core::ptr::null_mut::<*const ::core::ffi::c_char>(),
            ::core::ptr::null_mut::<*const ::core::ffi::c_char>(),
            ::core::ptr::null_mut::<*const ::core::ffi::c_char>(),
            ::core::ptr::null_mut::<*const ::core::ffi::c_char>(),
            ::core::ptr::null_mut::<*const ::core::ffi::c_char>(),
            ::core::ptr::null_mut::<*const ::core::ffi::c_char>(),
            ::core::ptr::null_mut::<*const ::core::ffi::c_char>(),
            ::core::ptr::null_mut::<*const ::core::ffi::c_char>(),
            ::core::ptr::null_mut::<*const ::core::ffi::c_char>(),
            ::core::ptr::null_mut::<*const ::core::ffi::c_char>(),
            ::core::ptr::null_mut::<*const ::core::ffi::c_char>(),
            ::core::ptr::null_mut::<*const ::core::ffi::c_char>(),
            ::core::ptr::null_mut::<*const ::core::ffi::c_char>(),
            ::core::ptr::null_mut::<*const ::core::ffi::c_char>(),
            ::core::ptr::null_mut::<*const ::core::ffi::c_char>(),
            ::core::ptr::null_mut::<*const ::core::ffi::c_char>(),
            ::core::ptr::null_mut::<*const ::core::ffi::c_char>(),
            ::core::ptr::null_mut::<*const ::core::ffi::c_char>(),
            ::core::ptr::null_mut::<*const ::core::ffi::c_char>(),
            ::core::ptr::null_mut::<*const ::core::ffi::c_char>(),
            ::core::ptr::null_mut::<*const ::core::ffi::c_char>(),
            ::core::ptr::null_mut::<*const ::core::ffi::c_char>(),
            ::core::ptr::null_mut::<*const ::core::ffi::c_char>(),
            ::core::ptr::null_mut::<*const ::core::ffi::c_char>(),
            ::core::ptr::null_mut::<*const ::core::ffi::c_char>(),
            ::core::ptr::null_mut::<*const ::core::ffi::c_char>(),
            ::core::ptr::null_mut::<*const ::core::ffi::c_char>(),
            ::core::ptr::null_mut::<*const ::core::ffi::c_char>(),
            ::core::ptr::null_mut::<*const ::core::ffi::c_char>(),
            ::core::ptr::null_mut::<*const ::core::ffi::c_char>(),
            ::core::ptr::null_mut::<*const ::core::ffi::c_char>(),
            ::core::ptr::null_mut::<*const ::core::ffi::c_char>(),
            ::core::ptr::null_mut::<*const ::core::ffi::c_char>(),
            ::core::ptr::null_mut::<*const ::core::ffi::c_char>(),
            ::core::ptr::null_mut::<*const ::core::ffi::c_char>(),
            ::core::ptr::null_mut::<*const ::core::ffi::c_char>(),
            ::core::ptr::null_mut::<*const ::core::ffi::c_char>(),
            ::core::ptr::null_mut::<*const ::core::ffi::c_char>(),
            ::core::ptr::null_mut::<*const ::core::ffi::c_char>(),
            ::core::ptr::null_mut::<*const ::core::ffi::c_char>(),
            ::core::ptr::null_mut::<*const ::core::ffi::c_char>(),
            ::core::ptr::null_mut::<*const ::core::ffi::c_char>(),
            ::core::ptr::null_mut::<*const ::core::ffi::c_char>(),
            ::core::ptr::null_mut::<*const ::core::ffi::c_char>(),
            ::core::ptr::null_mut::<*const ::core::ffi::c_char>(),
            ::core::ptr::null_mut::<*const ::core::ffi::c_char>(),
            ::core::ptr::null_mut::<*const ::core::ffi::c_char>(),
            ::core::ptr::null_mut::<*const ::core::ffi::c_char>(),
            ::core::ptr::null_mut::<*const ::core::ffi::c_char>(),
            ::core::ptr::null_mut::<*const ::core::ffi::c_char>(),
            ::core::ptr::null_mut::<*const ::core::ffi::c_char>(),
            ::core::ptr::null_mut::<*const ::core::ffi::c_char>(),
            ::core::ptr::null_mut::<*const ::core::ffi::c_char>(),
            ::core::ptr::null_mut::<*const ::core::ffi::c_char>(),
            ::core::ptr::null_mut::<*const ::core::ffi::c_char>(),
            ::core::ptr::null_mut::<*const ::core::ffi::c_char>(),
            ::core::ptr::null_mut::<*const ::core::ffi::c_char>(),
            ::core::ptr::null_mut::<*const ::core::ffi::c_char>(),
            ::core::ptr::null_mut::<*const ::core::ffi::c_char>(),
            ::core::ptr::null_mut::<*const ::core::ffi::c_char>(),
            ::core::ptr::null_mut::<*const ::core::ffi::c_char>(),
            ::core::ptr::null_mut::<*const ::core::ffi::c_char>(),
            ::core::ptr::null_mut::<*const ::core::ffi::c_char>(),
            ::core::ptr::null_mut::<*const ::core::ffi::c_char>(),
            ::core::ptr::null_mut::<*const ::core::ffi::c_char>(),
            ::core::ptr::null_mut::<*const ::core::ffi::c_char>(),
            ::core::ptr::null_mut::<*const ::core::ffi::c_char>(),
            ::core::ptr::null_mut::<*const ::core::ffi::c_char>(),
            ::core::ptr::null_mut::<*const ::core::ffi::c_char>(),
            ::core::ptr::null_mut::<*const ::core::ffi::c_char>(),
            ::core::ptr::null_mut::<*const ::core::ffi::c_char>(),
            ::core::ptr::null_mut::<*const ::core::ffi::c_char>(),
            ::core::ptr::null_mut::<*const ::core::ffi::c_char>(),
            ::core::ptr::null_mut::<*const ::core::ffi::c_char>(),
            ::core::ptr::null_mut::<*const ::core::ffi::c_char>(),
            ::core::ptr::null_mut::<*const ::core::ffi::c_char>(),
            ::core::ptr::null_mut::<*const ::core::ffi::c_char>(),
            ::core::ptr::null_mut::<*const ::core::ffi::c_char>(),
            ::core::ptr::null_mut::<*const ::core::ffi::c_char>(),
            ::core::ptr::null_mut::<*const ::core::ffi::c_char>(),
            ::core::ptr::null_mut::<*const ::core::ffi::c_char>(),
            ::core::ptr::null_mut::<*const ::core::ffi::c_char>(),
            ::core::ptr::null_mut::<*const ::core::ffi::c_char>(),
            ::core::ptr::null_mut::<*const ::core::ffi::c_char>(),
            ::core::ptr::null_mut::<*const ::core::ffi::c_char>(),
            ::core::ptr::null_mut::<*const ::core::ffi::c_char>(),
            ::core::ptr::null_mut::<*const ::core::ffi::c_char>(),
            ::core::ptr::null_mut::<*const ::core::ffi::c_char>(),
            ::core::ptr::null_mut::<*const ::core::ffi::c_char>(),
            ::core::ptr::null_mut::<*const ::core::ffi::c_char>(),
            ::core::ptr::null_mut::<*const ::core::ffi::c_char>(),
            ::core::ptr::null_mut::<*const ::core::ffi::c_char>(),
            ::core::ptr::null_mut::<*const ::core::ffi::c_char>(),
            ::core::ptr::null_mut::<*const ::core::ffi::c_char>(),
            ::core::ptr::null_mut::<*const ::core::ffi::c_char>(),
            ::core::ptr::null_mut::<*const ::core::ffi::c_char>(),
            ::core::ptr::null_mut::<*const ::core::ffi::c_char>(),
            ::core::ptr::null_mut::<*const ::core::ffi::c_char>(),
            ::core::ptr::null_mut::<*const ::core::ffi::c_char>(),
            ::core::ptr::null_mut::<*const ::core::ffi::c_char>(),
            ::core::ptr::null_mut::<*const ::core::ffi::c_char>(),
            ::core::ptr::null_mut::<*const ::core::ffi::c_char>(),
            ::core::ptr::null_mut::<*const ::core::ffi::c_char>(),
            ::core::ptr::null_mut::<*const ::core::ffi::c_char>(),
            ::core::ptr::null_mut::<*const ::core::ffi::c_char>(),
            ::core::ptr::null_mut::<*const ::core::ffi::c_char>(),
            ::core::ptr::null_mut::<*const ::core::ffi::c_char>(),
            ::core::ptr::null_mut::<*const ::core::ffi::c_char>(),
            ::core::ptr::null_mut::<*const ::core::ffi::c_char>(),
            ::core::ptr::null_mut::<*const ::core::ffi::c_char>(),
            ::core::ptr::null_mut::<*const ::core::ffi::c_char>(),
            ::core::ptr::null_mut::<*const ::core::ffi::c_char>(),
            ::core::ptr::null_mut::<*const ::core::ffi::c_char>(),
            ::core::ptr::null_mut::<*const ::core::ffi::c_char>(),
            ::core::ptr::null_mut::<*const ::core::ffi::c_char>(),
            ::core::ptr::null_mut::<*const ::core::ffi::c_char>(),
            ::core::ptr::null_mut::<*const ::core::ffi::c_char>(),
            ::core::ptr::null_mut::<*const ::core::ffi::c_char>(),
            ::core::ptr::null_mut::<*const ::core::ffi::c_char>(),
            ::core::ptr::null_mut::<*const ::core::ffi::c_char>(),
            ::core::ptr::null_mut::<*const ::core::ffi::c_char>(),
            ::core::ptr::null_mut::<*const ::core::ffi::c_char>(),
            ::core::ptr::null_mut::<*const ::core::ffi::c_char>(),
            ::core::ptr::null_mut::<*const ::core::ffi::c_char>(),
            ::core::ptr::null_mut::<*const ::core::ffi::c_char>(),
            ::core::ptr::null_mut::<*const ::core::ffi::c_char>(),
            ::core::ptr::null_mut::<*const ::core::ffi::c_char>(),
            ::core::ptr::null_mut::<*const ::core::ffi::c_char>(),
            ::core::ptr::null_mut::<*const ::core::ffi::c_char>(),
            ::core::ptr::null_mut::<*const ::core::ffi::c_char>(),
            ::core::ptr::null_mut::<*const ::core::ffi::c_char>(),
            ::core::ptr::null_mut::<*const ::core::ffi::c_char>(),
            ::core::ptr::null_mut::<*const ::core::ffi::c_char>(),
            ::core::ptr::null_mut::<*const ::core::ffi::c_char>(),
            ::core::ptr::null_mut::<*const ::core::ffi::c_char>(),
            ::core::ptr::null_mut::<*const ::core::ffi::c_char>(),
            ::core::ptr::null_mut::<*const ::core::ffi::c_char>(),
            ::core::ptr::null_mut::<*const ::core::ffi::c_char>(),
            ::core::ptr::null_mut::<*const ::core::ffi::c_char>(),
            ::core::ptr::null_mut::<*const ::core::ffi::c_char>(),
            ::core::ptr::null_mut::<*const ::core::ffi::c_char>(),
            ::core::ptr::null_mut::<*const ::core::ffi::c_char>(),
            ::core::ptr::null_mut::<*const ::core::ffi::c_char>(),
            ::core::ptr::null_mut::<*const ::core::ffi::c_char>(),
            ::core::ptr::null_mut::<*const ::core::ffi::c_char>(),
            ::core::ptr::null_mut::<*const ::core::ffi::c_char>(),
            ::core::ptr::null_mut::<*const ::core::ffi::c_char>(),
            ::core::ptr::null_mut::<*const ::core::ffi::c_char>(),
            ::core::ptr::null_mut::<*const ::core::ffi::c_char>(),
            ::core::ptr::null_mut::<*const ::core::ffi::c_char>(),
            ::core::ptr::null_mut::<*const ::core::ffi::c_char>(),
            ::core::ptr::null_mut::<*const ::core::ffi::c_char>(),
            ::core::ptr::null_mut::<*const ::core::ffi::c_char>(),
            ::core::ptr::null_mut::<*const ::core::ffi::c_char>(),
            ::core::ptr::null_mut::<*const ::core::ffi::c_char>(),
            ::core::ptr::null_mut::<*const ::core::ffi::c_char>(),
            ::core::ptr::null_mut::<*const ::core::ffi::c_char>(),
            ::core::ptr::null_mut::<*const ::core::ffi::c_char>(),
            ::core::ptr::null_mut::<*const ::core::ffi::c_char>(),
            ::core::ptr::null_mut::<*const ::core::ffi::c_char>(),
            ::core::ptr::null_mut::<*const ::core::ffi::c_char>(),
            ::core::ptr::null_mut::<*const ::core::ffi::c_char>(),
            ::core::ptr::null_mut::<*const ::core::ffi::c_char>(),
            ::core::ptr::null_mut::<*const ::core::ffi::c_char>(),
            ::core::ptr::null_mut::<*const ::core::ffi::c_char>(),
            ::core::ptr::null_mut::<*const ::core::ffi::c_char>(),
            ::core::ptr::null_mut::<*const ::core::ffi::c_char>(),
            ::core::ptr::null_mut::<*const ::core::ffi::c_char>(),
            ::core::ptr::null_mut::<*const ::core::ffi::c_char>(),
            ::core::ptr::null_mut::<*const ::core::ffi::c_char>(),
            ::core::ptr::null_mut::<*const ::core::ffi::c_char>(),
            ::core::ptr::null_mut::<*const ::core::ffi::c_char>(),
            ::core::ptr::null_mut::<*const ::core::ffi::c_char>(),
            ::core::ptr::null_mut::<*const ::core::ffi::c_char>(),
            ::core::ptr::null_mut::<*const ::core::ffi::c_char>(),
            ::core::ptr::null_mut::<*const ::core::ffi::c_char>(),
            ::core::ptr::null_mut::<*const ::core::ffi::c_char>(),
            ::core::ptr::null_mut::<*const ::core::ffi::c_char>(),
            ::core::ptr::null_mut::<*const ::core::ffi::c_char>(),
            ::core::ptr::null_mut::<*const ::core::ffi::c_char>(),
            ::core::ptr::null_mut::<*const ::core::ffi::c_char>(),
            ::core::ptr::null_mut::<*const ::core::ffi::c_char>(),
            ::core::ptr::null_mut::<*const ::core::ffi::c_char>(),
            ::core::ptr::null_mut::<*const ::core::ffi::c_char>(),
            ::core::ptr::null_mut::<*const ::core::ffi::c_char>(),
            ::core::ptr::null_mut::<*const ::core::ffi::c_char>(),
            ::core::ptr::null_mut::<*const ::core::ffi::c_char>(),
            ::core::ptr::null_mut::<*const ::core::ffi::c_char>(),
            ::core::ptr::null_mut::<*const ::core::ffi::c_char>(),
            ::core::ptr::null_mut::<*const ::core::ffi::c_char>(),
            ::core::ptr::null_mut::<*const ::core::ffi::c_char>(),
            ::core::ptr::null_mut::<*const ::core::ffi::c_char>(),
            ::core::ptr::null_mut::<*const ::core::ffi::c_char>(),
            ::core::ptr::null_mut::<*const ::core::ffi::c_char>(),
            ::core::ptr::null_mut::<*const ::core::ffi::c_char>(),
        ],
        indent: ::core::ptr::null::<::core::ffi::c_char>(),
        sort_keys: DEFAULT_ENCODE_SORT_KEYS,
        keybuf: keybuf_t {
            buf: strbuf_t {
                buf: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                size: 0,
                length: 0,
                dynamic: 0,
                reallocs: 0,
                debug: 0,
            },
            keys: ::core::ptr::null_mut::<key_entry_t>(),
            size: 0,
            capacity: 0,
        },
    };
    let mut ctx: json_encode_t = json_encode_t {
        cfg: cfg,
        options: &raw mut options,
        json: ::core::ptr::null_mut::<strbuf_t>(),
    };
    let mut local_encode_buf: strbuf_t = strbuf_t {
        buf: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        size: 0,
        length: 0,
        dynamic: 0,
        reallocs: 0,
        debug: 0,
    };
    let mut encode_buf: *mut strbuf_t = ::core::ptr::null_mut::<strbuf_t>();
    let mut json: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut len: size_t = 0;
    let mut customChar2escape: [*const ::core::ffi::c_char; 256] =
        [::core::ptr::null::<::core::ffi::c_char>(); 256];
    match lua_gettop(l) {
        1 => {}
        2 => {
            luaL_checktype(l, 2 as ::core::ffi::c_int, LUA_TTABLE);
            lua_getfield(
                l,
                2 as ::core::ffi::c_int,
                b"escape_slash\0".as_ptr() as *const ::core::ffi::c_char,
            );
            if !(lua_type(l, -1 as ::core::ffi::c_int) == LUA_TNIL) {
                luaL_checktype(l, -1 as ::core::ffi::c_int, LUA_TBOOLEAN);
                let mut escape_slash: ::core::ffi::c_int =
                    lua_toboolean(l, -1 as ::core::ffi::c_int);
                if escape_slash != 0 {
                    memcpy(
                        &raw mut customChar2escape as *mut *const ::core::ffi::c_char
                            as *mut ::core::ffi::c_void,
                        &raw mut char2escape as *mut *const ::core::ffi::c_char
                            as *const ::core::ffi::c_void,
                        ::core::mem::size_of::<[*const ::core::ffi::c_char; 256]>(),
                    );
                    customChar2escape['/' as ::core::ffi::c_int as usize] =
                        b"\\/\0".as_ptr() as *const ::core::ffi::c_char;
                    *(&raw mut (*ctx.options).char2escape
                        as *mut *mut *const ::core::ffi::c_char) =
                        &raw mut customChar2escape as *mut *const ::core::ffi::c_char;
                }
            }
            lua_settop(l, -1 as ::core::ffi::c_int - 1 as ::core::ffi::c_int);
            lua_getfield(
                l,
                2 as ::core::ffi::c_int,
                b"indent\0".as_ptr() as *const ::core::ffi::c_char,
            );
            if !(lua_type(l, -1 as ::core::ffi::c_int) == LUA_TNIL) {
                options.indent = luaL_checklstring(
                    l,
                    -1 as ::core::ffi::c_int,
                    ::core::ptr::null_mut::<size_t>(),
                );
                if *options.indent.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                    == '\0' as ::core::ffi::c_int
                {
                    options.indent = ::core::ptr::null::<::core::ffi::c_char>();
                }
            }
            lua_settop(l, -1 as ::core::ffi::c_int - 1 as ::core::ffi::c_int);
            lua_getfield(
                l,
                2 as ::core::ffi::c_int,
                b"sort_keys\0".as_ptr() as *const ::core::ffi::c_char,
            );
            if !(lua_type(l, -1 as ::core::ffi::c_int) == LUA_TNIL) {
                luaL_checktype(l, -1 as ::core::ffi::c_int, LUA_TBOOLEAN);
                let mut sort_keys: ::core::ffi::c_int = lua_toboolean(l, -1 as ::core::ffi::c_int);
                if sort_keys != 0 {
                    options.sort_keys = sort_keys;
                    strbuf_init(&raw mut options.keybuf.buf, 0 as size_t);
                    options.keybuf.size = 0 as size_t;
                    options.keybuf.capacity = KEYBUF_DEFAULT_CAPACITY as size_t;
                    options.keybuf.keys = malloc(
                        options
                            .keybuf
                            .capacity
                            .wrapping_mul(::core::mem::size_of::<key_entry_t>()),
                    ) as *mut key_entry_t;
                    if options.keybuf.keys.is_null() {
                        return luaL_error(
                            l,
                            b"out of memory\0".as_ptr() as *const ::core::ffi::c_char,
                        );
                    }
                }
            }
            lua_settop(l, -2 as ::core::ffi::c_int - 1 as ::core::ffi::c_int);
        }
        _ => {
            return luaL_error(
                l,
                b"expected 1 or 2 arguments\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
    }
    if (*cfg).encode_keep_buffer == 0 {
        encode_buf = &raw mut local_encode_buf;
        strbuf_init(encode_buf, 0 as size_t);
    } else {
        encode_buf = &raw mut (*cfg).encode_buf;
        strbuf_reset(encode_buf);
    }
    ctx.json = encode_buf;
    json_append_data(l, &raw mut ctx, 0 as ::core::ffi::c_int);
    json = strbuf_string(encode_buf, &raw mut len);
    lua_pushlstring(l, json, len);
    if (*cfg).encode_keep_buffer == 0 {
        strbuf_free(encode_buf);
    }
    if options.sort_keys != 0 {
        strbuf_free(&raw mut options.keybuf.buf);
        free(options.keybuf.keys as *mut ::core::ffi::c_void);
    }
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn hexdigit2int(mut hex: ::core::ffi::c_char) -> ::core::ffi::c_int {
    if '0' as ::core::ffi::c_int <= hex as ::core::ffi::c_int
        && hex as ::core::ffi::c_int <= '9' as ::core::ffi::c_int
    {
        return hex as ::core::ffi::c_int - '0' as ::core::ffi::c_int;
    }
    hex = (hex as ::core::ffi::c_int | 0x20 as ::core::ffi::c_int) as ::core::ffi::c_char;
    if 'a' as ::core::ffi::c_int <= hex as ::core::ffi::c_int
        && hex as ::core::ffi::c_int <= 'f' as ::core::ffi::c_int
    {
        return 10 as ::core::ffi::c_int + hex as ::core::ffi::c_int - 'a' as ::core::ffi::c_int;
    }
    return -1 as ::core::ffi::c_int;
}
unsafe extern "C" fn decode_hex4(mut hex: *const ::core::ffi::c_char) -> ::core::ffi::c_int {
    let mut digit: [::core::ffi::c_int; 4] = [0; 4];
    let mut i: ::core::ffi::c_int = 0;
    i = 0 as ::core::ffi::c_int;
    while i < 4 as ::core::ffi::c_int {
        digit[i as usize] = hexdigit2int(*hex.offset(i as isize));
        if digit[i as usize] < 0 as ::core::ffi::c_int {
            return -1 as ::core::ffi::c_int;
        }
        i += 1;
    }
    return (digit[0 as ::core::ffi::c_int as usize] << 12 as ::core::ffi::c_int)
        + (digit[1 as ::core::ffi::c_int as usize] << 8 as ::core::ffi::c_int)
        + (digit[2 as ::core::ffi::c_int as usize] << 4 as ::core::ffi::c_int)
        + digit[3 as ::core::ffi::c_int as usize];
}
unsafe extern "C" fn codepoint_to_utf8(
    mut utf8: *mut ::core::ffi::c_char,
    mut codepoint: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    if codepoint <= 0x7f as ::core::ffi::c_int {
        *utf8.offset(0 as ::core::ffi::c_int as isize) = codepoint as ::core::ffi::c_char;
        return 1 as ::core::ffi::c_int;
    }
    if codepoint <= 0x7ff as ::core::ffi::c_int {
        *utf8.offset(0 as ::core::ffi::c_int as isize) = (codepoint >> 6 as ::core::ffi::c_int
            | 0xc0 as ::core::ffi::c_int)
            as ::core::ffi::c_char;
        *utf8.offset(1 as ::core::ffi::c_int as isize) = (codepoint & 0x3f as ::core::ffi::c_int
            | 0x80 as ::core::ffi::c_int)
            as ::core::ffi::c_char;
        return 2 as ::core::ffi::c_int;
    }
    if codepoint <= 0xffff as ::core::ffi::c_int {
        *utf8.offset(0 as ::core::ffi::c_int as isize) = (codepoint >> 12 as ::core::ffi::c_int
            | 0xe0 as ::core::ffi::c_int)
            as ::core::ffi::c_char;
        *utf8.offset(1 as ::core::ffi::c_int as isize) =
            (codepoint >> 6 as ::core::ffi::c_int & 0x3f as ::core::ffi::c_int
                | 0x80 as ::core::ffi::c_int) as ::core::ffi::c_char;
        *utf8.offset(2 as ::core::ffi::c_int as isize) = (codepoint & 0x3f as ::core::ffi::c_int
            | 0x80 as ::core::ffi::c_int)
            as ::core::ffi::c_char;
        return 3 as ::core::ffi::c_int;
    }
    if codepoint <= 0x1fffff as ::core::ffi::c_int {
        *utf8.offset(0 as ::core::ffi::c_int as isize) = (codepoint >> 18 as ::core::ffi::c_int
            | 0xf0 as ::core::ffi::c_int)
            as ::core::ffi::c_char;
        *utf8.offset(1 as ::core::ffi::c_int as isize) =
            (codepoint >> 12 as ::core::ffi::c_int & 0x3f as ::core::ffi::c_int
                | 0x80 as ::core::ffi::c_int) as ::core::ffi::c_char;
        *utf8.offset(2 as ::core::ffi::c_int as isize) =
            (codepoint >> 6 as ::core::ffi::c_int & 0x3f as ::core::ffi::c_int
                | 0x80 as ::core::ffi::c_int) as ::core::ffi::c_char;
        *utf8.offset(3 as ::core::ffi::c_int as isize) = (codepoint & 0x3f as ::core::ffi::c_int
            | 0x80 as ::core::ffi::c_int)
            as ::core::ffi::c_char;
        return 4 as ::core::ffi::c_int;
    }
    return 0 as ::core::ffi::c_int;
}
unsafe extern "C" fn json_append_unicode_escape(mut json: *mut json_parse_t) -> ::core::ffi::c_int {
    let mut utf8: [::core::ffi::c_char; 4] = [0; 4];
    let mut codepoint: ::core::ffi::c_int = 0;
    let mut surrogate_low: ::core::ffi::c_int = 0;
    let mut len: ::core::ffi::c_int = 0;
    let mut escape_len: ::core::ffi::c_int = 6 as ::core::ffi::c_int;
    codepoint = decode_hex4((*json).ptr.offset(2 as ::core::ffi::c_int as isize));
    if codepoint < 0 as ::core::ffi::c_int {
        return -1 as ::core::ffi::c_int;
    }
    if codepoint & 0xf800 as ::core::ffi::c_int == 0xd800 as ::core::ffi::c_int {
        if codepoint & 0x400 as ::core::ffi::c_int != 0 {
            return -1 as ::core::ffi::c_int;
        }
        if *(*json).ptr.offset(escape_len as isize) as ::core::ffi::c_int
            != '\\' as ::core::ffi::c_int
            || *(*json)
                .ptr
                .offset(escape_len as isize)
                .offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                != 'u' as ::core::ffi::c_int
        {
            return -1 as ::core::ffi::c_int;
        }
        surrogate_low = decode_hex4(
            (*json)
                .ptr
                .offset(2 as ::core::ffi::c_int as isize)
                .offset(escape_len as isize),
        );
        if surrogate_low < 0 as ::core::ffi::c_int {
            return -1 as ::core::ffi::c_int;
        }
        if surrogate_low & 0xfc00 as ::core::ffi::c_int != 0xdc00 as ::core::ffi::c_int {
            return -1 as ::core::ffi::c_int;
        }
        codepoint = (codepoint & 0x3ff as ::core::ffi::c_int) << 10 as ::core::ffi::c_int;
        surrogate_low &= 0x3ff as ::core::ffi::c_int;
        codepoint = (codepoint | surrogate_low) + 0x10000 as ::core::ffi::c_int;
        escape_len = 12 as ::core::ffi::c_int;
    }
    len = codepoint_to_utf8(&raw mut utf8 as *mut ::core::ffi::c_char, codepoint);
    if len == 0 {
        return -1 as ::core::ffi::c_int;
    }
    strbuf_append_mem_unsafe(
        (*json).tmp,
        &raw mut utf8 as *mut ::core::ffi::c_char,
        len as size_t,
    );
    (*json).ptr = (*json).ptr.offset(escape_len as isize);
    return 0 as ::core::ffi::c_int;
}
unsafe extern "C" fn json_set_token_error(
    mut token: *mut json_token_t,
    mut json: *mut json_parse_t,
    mut errtype: *const ::core::ffi::c_char,
) {
    (*token).type_0 = T_ERROR;
    (*token).index = (*json).ptr.offset_from((*json).data) as size_t;
    (*token).value.string = errtype;
}
unsafe extern "C" fn json_next_string_token(
    mut json: *mut json_parse_t,
    mut token: *mut json_token_t,
) {
    let mut escape2char: *mut ::core::ffi::c_char =
        &raw mut (*(*json).cfg).escape2char as *mut ::core::ffi::c_char;
    let mut ch: ::core::ffi::c_char = 0;
    '_c2rust_label: {
        if *(*json).ptr as ::core::ffi::c_int == '"' as ::core::ffi::c_int {
        } else {
            __assert_fail(
                b"*json->ptr == '\"'\0".as_ptr() as *const ::core::ffi::c_char,
                b"/home/overlord/projects/neovim/neovim/src/cjson/lua_cjson.c\0".as_ptr()
                    as *const ::core::ffi::c_char,
                1445 as ::core::ffi::c_uint,
                __ASSERT_FUNCTION.as_ptr(),
            );
        }
    };
    (*json).ptr = (*json).ptr.offset(1);
    strbuf_reset((*json).tmp);
    loop {
        ch = *(*json).ptr;
        if ch as ::core::ffi::c_int == '"' as ::core::ffi::c_int {
            break;
        }
        if ch == 0 {
            json_set_token_error(
                token,
                json,
                b"unexpected end of string\0".as_ptr() as *const ::core::ffi::c_char,
            );
            return;
        }
        if ch as ::core::ffi::c_int == '\\' as ::core::ffi::c_int {
            ch = *(*json).ptr.offset(1 as ::core::ffi::c_int as isize);
            ch = *escape2char.offset(ch as ::core::ffi::c_uchar as isize);
            if ch as ::core::ffi::c_int == 'u' as ::core::ffi::c_int {
                if json_append_unicode_escape(json) == 0 as ::core::ffi::c_int {
                    continue;
                }
                json_set_token_error(
                    token,
                    json,
                    b"invalid unicode escape code\0".as_ptr() as *const ::core::ffi::c_char,
                );
                return;
            } else {
                if ch == 0 {
                    json_set_token_error(
                        token,
                        json,
                        b"invalid escape code\0".as_ptr() as *const ::core::ffi::c_char,
                    );
                    return;
                }
                (*json).ptr = (*json).ptr.offset(1);
            }
        }
        strbuf_append_char_unsafe((*json).tmp, ch);
        (*json).ptr = (*json).ptr.offset(1);
    }
    (*json).ptr = (*json).ptr.offset(1);
    strbuf_ensure_null((*json).tmp);
    (*token).type_0 = T_STRING;
    (*token).value.string = strbuf_string((*json).tmp, &raw mut (*token).string_len);
}
unsafe extern "C" fn json_is_invalid_number(mut json: *mut json_parse_t) -> ::core::ffi::c_int {
    let mut p: *const ::core::ffi::c_char = (*json).ptr;
    if *p as ::core::ffi::c_int == '+' as ::core::ffi::c_int {
        return 1 as ::core::ffi::c_int;
    }
    if *p as ::core::ffi::c_int == '-' as ::core::ffi::c_int {
        p = p.offset(1);
    }
    if *p as ::core::ffi::c_int == '0' as ::core::ffi::c_int {
        let mut ch2: ::core::ffi::c_char = *p.offset(1 as ::core::ffi::c_int as isize);
        if ch2 as ::core::ffi::c_int | 0x20 as ::core::ffi::c_int == 'x' as ::core::ffi::c_int
            || '0' as ::core::ffi::c_int <= ch2 as ::core::ffi::c_int
                && ch2 as ::core::ffi::c_int <= '9' as ::core::ffi::c_int
        {
            return 1 as ::core::ffi::c_int;
        }
        return 0 as ::core::ffi::c_int;
    } else if *p as ::core::ffi::c_int <= '9' as ::core::ffi::c_int {
        return 0 as ::core::ffi::c_int;
    }
    if strncasecmp(
        p,
        b"inf\0".as_ptr() as *const ::core::ffi::c_char,
        3 as size_t,
    ) == 0
    {
        return 1 as ::core::ffi::c_int;
    }
    if strncasecmp(
        p,
        b"nan\0".as_ptr() as *const ::core::ffi::c_char,
        3 as size_t,
    ) == 0
    {
        return 1 as ::core::ffi::c_int;
    }
    return 0 as ::core::ffi::c_int;
}
unsafe extern "C" fn json_next_number_token(
    mut json: *mut json_parse_t,
    mut token: *mut json_token_t,
) {
    let mut endptr: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut tmpval: ::core::ffi::c_longlong =
        strtoll((*json).ptr, &raw mut endptr, 10 as ::core::ffi::c_int);
    if (*json).ptr == endptr as *const ::core::ffi::c_char
        || *endptr as ::core::ffi::c_int == '.' as ::core::ffi::c_int
        || *endptr as ::core::ffi::c_int == 'e' as ::core::ffi::c_int
        || *endptr as ::core::ffi::c_int == 'E' as ::core::ffi::c_int
        || *endptr as ::core::ffi::c_int == 'x' as ::core::ffi::c_int
    {
        (*token).type_0 = T_NUMBER;
        (*token).value.number = fpconv_strtod((*json).ptr, &raw mut endptr);
        if (*json).ptr == endptr as *const ::core::ffi::c_char {
            json_set_token_error(
                token,
                json,
                b"invalid number\0".as_ptr() as *const ::core::ffi::c_char,
            );
            return;
        }
    } else if tmpval > PTRDIFF_MAX as ::core::ffi::c_longlong
        || tmpval < PTRDIFF_MIN as ::core::ffi::c_longlong
    {
        (*token).type_0 = T_NUMBER;
        (*token).value.number = tmpval as ::core::ffi::c_double;
    } else {
        (*token).type_0 = T_INTEGER;
        (*token).value.integer = tmpval as lua_Integer;
    }
    (*json).ptr = endptr;
}
unsafe extern "C" fn json_next_token(mut json: *mut json_parse_t, mut token: *mut json_token_t) {
    let mut ch2token: *const json_token_type_t =
        &raw mut (*(*json).cfg).ch2token as *mut json_token_type_t;
    let mut ch: ::core::ffi::c_int = 0;
    loop {
        loop {
            ch = *(*json).ptr as ::core::ffi::c_uchar as ::core::ffi::c_int;
            (*token).type_0 = *ch2token.offset(ch as isize);
            if (*token).type_0 as ::core::ffi::c_uint
                != T_WHITESPACE as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                break;
            }
            (*json).ptr = (*json).ptr.offset(1);
        }
        if !(*(*json).options).skip_comments {
            break;
        }
        if *(*json).ptr.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_uchar
            as ::core::ffi::c_int
            != '/' as ::core::ffi::c_int
            || *(*json).ptr.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_uchar
                as ::core::ffi::c_int
                != '/' as ::core::ffi::c_int
                && *(*json).ptr.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_uchar
                    as ::core::ffi::c_int
                    != '*' as ::core::ffi::c_int
        {
            break;
        }
        if *(*json).ptr.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
            == '/' as ::core::ffi::c_int
        {
            (*json).ptr = (*json).ptr.offset(2 as ::core::ffi::c_int as isize);
            while *(*json).ptr as ::core::ffi::c_int != '\0' as ::core::ffi::c_int
                && *(*json).ptr as ::core::ffi::c_int != '\n' as ::core::ffi::c_int
            {
                (*json).ptr = (*json).ptr.offset(1);
            }
        } else {
            (*json).ptr = (*json).ptr.offset(2 as ::core::ffi::c_int as isize);
            loop {
                if *(*json).ptr as ::core::ffi::c_int == '\0' as ::core::ffi::c_int {
                    json_set_token_error(
                        token,
                        json,
                        b"unclosed multi-line comment\0".as_ptr() as *const ::core::ffi::c_char,
                    );
                    return;
                }
                if *(*json).ptr.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                    == '*' as ::core::ffi::c_int
                    && *(*json).ptr.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                        == '/' as ::core::ffi::c_int
                {
                    (*json).ptr = (*json).ptr.offset(2 as ::core::ffi::c_int as isize);
                    break;
                } else {
                    (*json).ptr = (*json).ptr.offset(1);
                }
            }
        }
    }
    (*token).index = (*json).ptr.offset_from((*json).data) as size_t;
    if (*token).type_0 as ::core::ffi::c_uint
        == T_ERROR as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        json_set_token_error(
            token,
            json,
            b"invalid token\0".as_ptr() as *const ::core::ffi::c_char,
        );
        return;
    }
    if (*token).type_0 as ::core::ffi::c_uint == T_END as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        return;
    }
    if (*token).type_0 as ::core::ffi::c_uint
        != T_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        (*json).ptr = (*json).ptr.offset(1);
        return;
    }
    if ch == '"' as ::core::ffi::c_int {
        json_next_string_token(json, token);
        return;
    } else if ch == '-' as ::core::ffi::c_int
        || '0' as ::core::ffi::c_int <= ch && ch <= '9' as ::core::ffi::c_int
    {
        if (*(*json).cfg).decode_invalid_numbers == 0 && json_is_invalid_number(json) != 0 {
            json_set_token_error(
                token,
                json,
                b"invalid number\0".as_ptr() as *const ::core::ffi::c_char,
            );
            return;
        }
        json_next_number_token(json, token);
        return;
    } else if strncmp(
        (*json).ptr,
        b"true\0".as_ptr() as *const ::core::ffi::c_char,
        4 as size_t,
    ) == 0
    {
        (*token).type_0 = T_BOOLEAN;
        (*token).value.boolean = 1 as ::core::ffi::c_int;
        (*json).ptr = (*json).ptr.offset(4 as ::core::ffi::c_int as isize);
        return;
    } else if strncmp(
        (*json).ptr,
        b"false\0".as_ptr() as *const ::core::ffi::c_char,
        5 as size_t,
    ) == 0
    {
        (*token).type_0 = T_BOOLEAN;
        (*token).value.boolean = 0 as ::core::ffi::c_int;
        (*json).ptr = (*json).ptr.offset(5 as ::core::ffi::c_int as isize);
        return;
    } else if strncmp(
        (*json).ptr,
        b"null\0".as_ptr() as *const ::core::ffi::c_char,
        4 as size_t,
    ) == 0
    {
        (*token).type_0 = T_NULL;
        (*json).ptr = (*json).ptr.offset(4 as ::core::ffi::c_int as isize);
        return;
    } else if (*(*json).cfg).decode_invalid_numbers != 0 && json_is_invalid_number(json) != 0 {
        json_next_number_token(json, token);
        return;
    }
    json_set_token_error(
        token,
        json,
        b"invalid token\0".as_ptr() as *const ::core::ffi::c_char,
    );
}
unsafe extern "C" fn json_throw_parse_error(
    mut l: *mut lua_State,
    mut json: *mut json_parse_t,
    mut exp: *const ::core::ffi::c_char,
    mut token: *mut json_token_t,
) {
    let mut found: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    strbuf_free((*json).tmp);
    if (*token).type_0 as ::core::ffi::c_uint
        == T_ERROR as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        found = (*token).value.string;
    } else {
        found = json_token_type_name[(*token).type_0 as usize];
    }
    luaL_error(
        l,
        b"Expected %s but found %s at character %d\0".as_ptr() as *const ::core::ffi::c_char,
        exp,
        found,
        (*token).index.wrapping_add(1 as size_t),
    );
}
#[inline]
unsafe extern "C" fn json_decode_ascend(mut json: *mut json_parse_t) {
    (*json).current_depth -= 1;
}
unsafe extern "C" fn json_decode_descend(
    mut l: *mut lua_State,
    mut json: *mut json_parse_t,
    mut slots: ::core::ffi::c_int,
) {
    (*json).current_depth += 1;
    if (*json).current_depth <= (*(*json).cfg).decode_max_depth && lua_checkstack(l, slots) != 0 {
        return;
    }
    strbuf_free((*json).tmp);
    luaL_error(
        l,
        b"Found too many nested data structures (%d) at character %d\0".as_ptr()
            as *const ::core::ffi::c_char,
        (*json).current_depth,
        (*json).ptr.offset_from((*json).data),
    );
}
unsafe extern "C" fn json_parse_object_context(mut l: *mut lua_State, mut json: *mut json_parse_t) {
    let mut token: json_token_t = json_token_t {
        type_0: T_OBJ_BEGIN,
        index: 0,
        value: C2Rust_Unnamed_0 {
            string: ::core::ptr::null::<::core::ffi::c_char>(),
        },
        string_len: 0,
    };
    json_decode_descend(l, json, 3 as ::core::ffi::c_int);
    lua_createtable(l, 0 as ::core::ffi::c_int, 0 as ::core::ffi::c_int);
    json_next_token(json, &raw mut token);
    if token.type_0 as ::core::ffi::c_uint == T_OBJ_END as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        nlua_pushref(l, nlua_get_empty_dict_ref(l));
        lua_setmetatable(l, -2 as ::core::ffi::c_int);
        json_decode_ascend(json);
        return;
    }
    loop {
        if token.type_0 as ::core::ffi::c_uint
            != T_STRING as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            json_throw_parse_error(
                l,
                json,
                b"object key string\0".as_ptr() as *const ::core::ffi::c_char,
                &raw mut token,
            );
        }
        lua_pushlstring(l, token.value.string, token.string_len);
        json_next_token(json, &raw mut token);
        if token.type_0 as ::core::ffi::c_uint
            != T_COLON as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            json_throw_parse_error(
                l,
                json,
                b"colon\0".as_ptr() as *const ::core::ffi::c_char,
                &raw mut token,
            );
        }
        json_next_token(json, &raw mut token);
        json_process_value(l, json, &raw mut token, (*(*json).options).luanil_object);
        lua_rawset(l, -3 as ::core::ffi::c_int);
        json_next_token(json, &raw mut token);
        if token.type_0 as ::core::ffi::c_uint
            == T_OBJ_END as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            json_decode_ascend(json);
            return;
        }
        if token.type_0 as ::core::ffi::c_uint
            != T_COMMA as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            json_throw_parse_error(
                l,
                json,
                b"comma or object end\0".as_ptr() as *const ::core::ffi::c_char,
                &raw mut token,
            );
        }
        json_next_token(json, &raw mut token);
    }
}
unsafe extern "C" fn json_parse_array_context(mut l: *mut lua_State, mut json: *mut json_parse_t) {
    let mut token: json_token_t = json_token_t {
        type_0: T_OBJ_BEGIN,
        index: 0,
        value: C2Rust_Unnamed_0 {
            string: ::core::ptr::null::<::core::ffi::c_char>(),
        },
        string_len: 0,
    };
    let mut i: ::core::ffi::c_int = 0;
    json_decode_descend(l, json, 2 as ::core::ffi::c_int);
    lua_createtable(l, 0 as ::core::ffi::c_int, 0 as ::core::ffi::c_int);
    if (*(*json).cfg).decode_array_with_array_mt != 0 {
        lua_pushlightuserdata(
            l,
            ::core::ptr::from_exposed_addr_mut::<::core::ffi::c_void>(
                ((&raw mut json_array).expose_addr() as uintptr_t
                    & ((1 as uintptr_t) << 47 as ::core::ffi::c_int)
                        .wrapping_sub(1 as ::core::ffi::c_int as uintptr_t))
                    as usize,
            ),
        );
        lua_rawget(l, LUA_REGISTRYINDEX);
        lua_setmetatable(l, -2 as ::core::ffi::c_int);
    }
    json_next_token(json, &raw mut token);
    if token.type_0 as ::core::ffi::c_uint == T_ARR_END as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        json_decode_ascend(json);
        return;
    }
    i = 1 as ::core::ffi::c_int;
    loop {
        json_process_value(l, json, &raw mut token, (*(*json).options).luanil_array);
        lua_rawseti(l, -2 as ::core::ffi::c_int, i);
        json_next_token(json, &raw mut token);
        if token.type_0 as ::core::ffi::c_uint
            == T_ARR_END as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            json_decode_ascend(json);
            return;
        }
        if token.type_0 as ::core::ffi::c_uint
            != T_COMMA as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            json_throw_parse_error(
                l,
                json,
                b"comma or array end\0".as_ptr() as *const ::core::ffi::c_char,
                &raw mut token,
            );
        }
        json_next_token(json, &raw mut token);
        i += 1;
    }
}
unsafe extern "C" fn json_process_value(
    mut l: *mut lua_State,
    mut json: *mut json_parse_t,
    mut token: *mut json_token_t,
    mut use_luanil: bool,
) {
    match (*token).type_0 as ::core::ffi::c_uint {
        4 => {
            lua_pushlstring(l, (*token).value.string, (*token).string_len);
        }
        5 => {
            lua_pushnumber(l, (*token).value.number as lua_Number);
        }
        6 => {
            lua_pushinteger(l, (*token).value.integer);
        }
        7 => {
            lua_pushboolean(l, (*token).value.boolean);
        }
        0 => {
            json_parse_object_context(l, json);
        }
        2 => {
            json_parse_array_context(l, json);
        }
        8 => {
            if use_luanil {
                lua_pushnil(l);
            } else {
                nlua_pushref(l, nlua_get_nil_ref(l));
            }
        }
        _ => {
            json_throw_parse_error(
                l,
                json,
                b"value\0".as_ptr() as *const ::core::ffi::c_char,
                token,
            );
        }
    };
}
unsafe extern "C" fn json_decode(mut l: *mut lua_State) -> ::core::ffi::c_int {
    let mut json: json_parse_t = json_parse_t {
        data: ::core::ptr::null::<::core::ffi::c_char>(),
        ptr: ::core::ptr::null::<::core::ffi::c_char>(),
        tmp: ::core::ptr::null_mut::<strbuf_t>(),
        cfg: ::core::ptr::null_mut::<json_config_t>(),
        options: ::core::ptr::null_mut::<json_options_t>(),
        current_depth: 0,
    };
    let mut token: json_token_t = json_token_t {
        type_0: T_OBJ_BEGIN,
        index: 0,
        value: C2Rust_Unnamed_0 {
            string: ::core::ptr::null::<::core::ffi::c_char>(),
        },
        string_len: 0,
    };
    let mut options: json_options_t = json_options_t {
        luanil_object: false_0 != 0,
        luanil_array: false_0 != 0,
        skip_comments: false_0 != 0,
    };
    let mut json_len: size_t = 0;
    match lua_gettop(l) {
        1 => {}
        2 => {
            luaL_checktype(l, 2 as ::core::ffi::c_int, LUA_TTABLE);
            lua_getfield(
                l,
                2 as ::core::ffi::c_int,
                b"skip_comments\0".as_ptr() as *const ::core::ffi::c_char,
            );
            options.skip_comments = lua_toboolean(l, -1 as ::core::ffi::c_int) != 0;
            lua_settop(l, -1 as ::core::ffi::c_int - 1 as ::core::ffi::c_int);
            lua_getfield(
                l,
                2 as ::core::ffi::c_int,
                b"luanil\0".as_ptr() as *const ::core::ffi::c_char,
            );
            if lua_type(l, -1 as ::core::ffi::c_int) == LUA_TNIL {
                lua_settop(l, -1 as ::core::ffi::c_int - 1 as ::core::ffi::c_int);
            } else {
                luaL_checktype(l, -1 as ::core::ffi::c_int, LUA_TTABLE);
                lua_getfield(
                    l,
                    -1 as ::core::ffi::c_int,
                    b"object\0".as_ptr() as *const ::core::ffi::c_char,
                );
                options.luanil_object = lua_toboolean(l, -1 as ::core::ffi::c_int) != 0;
                lua_settop(l, -1 as ::core::ffi::c_int - 1 as ::core::ffi::c_int);
                lua_getfield(
                    l,
                    -1 as ::core::ffi::c_int,
                    b"array\0".as_ptr() as *const ::core::ffi::c_char,
                );
                options.luanil_array = lua_toboolean(l, -1 as ::core::ffi::c_int) != 0;
                lua_settop(l, -2 as ::core::ffi::c_int - 1 as ::core::ffi::c_int);
            }
        }
        _ => {
            return luaL_error(
                l,
                b"expected 1 or 2 arguments\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
    }
    json.cfg = json_fetch_config(l);
    json.data = luaL_checklstring(l, 1 as ::core::ffi::c_int, &raw mut json_len);
    json.options = &raw mut options;
    json.current_depth = 0 as ::core::ffi::c_int;
    json.ptr = json.data;
    if json_len >= 2 as size_t
        && (*json.data.offset(0 as ::core::ffi::c_int as isize) == 0
            || *json.data.offset(1 as ::core::ffi::c_int as isize) == 0)
    {
        luaL_error(
            l,
            b"JSON parser does not support UTF-16 or UTF-32\0".as_ptr()
                as *const ::core::ffi::c_char,
        );
    }
    json.tmp = strbuf_new(json_len);
    json_next_token(&raw mut json, &raw mut token);
    json_process_value(
        l,
        &raw mut json,
        &raw mut token,
        (*json.options).luanil_object,
    );
    json_next_token(&raw mut json, &raw mut token);
    if token.type_0 as ::core::ffi::c_uint != T_END as ::core::ffi::c_int as ::core::ffi::c_uint {
        json_throw_parse_error(
            l,
            &raw mut json,
            b"the end\0".as_ptr() as *const ::core::ffi::c_char,
            &raw mut token,
        );
    }
    strbuf_free(json.tmp);
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn compat_luaL_setfuncs(
    mut l: *mut lua_State,
    mut reg: *const luaL_Reg,
    mut nup: ::core::ffi::c_int,
) {
    let mut i: ::core::ffi::c_int = 0;
    luaL_checkstack(
        l,
        nup,
        b"too many upvalues\0".as_ptr() as *const ::core::ffi::c_char,
    );
    while !(*reg).name.is_null() {
        i = 0 as ::core::ffi::c_int;
        while i < nup {
            lua_pushvalue(l, -nup);
            i += 1;
        }
        lua_pushcclosure(l, (*reg).func, nup);
        lua_setfield(l, -(nup + 2 as ::core::ffi::c_int), (*reg).name);
        reg = reg.offset(1);
    }
    lua_settop(l, -nup - 1 as ::core::ffi::c_int);
}
unsafe extern "C" fn json_protect_conversion(mut l: *mut lua_State) -> ::core::ffi::c_int {
    let mut err: ::core::ffi::c_int = 0;
    (lua_gettop(l) == 1 as ::core::ffi::c_int
        || luaL_argerror(
            l,
            1 as ::core::ffi::c_int,
            b"expected 1 argument\0".as_ptr() as *const ::core::ffi::c_char,
        ) != 0) as ::core::ffi::c_int;
    lua_pushvalue(l, LUA_GLOBALSINDEX - 1 as ::core::ffi::c_int);
    lua_insert(l, 1 as ::core::ffi::c_int);
    err = lua_pcall(
        l,
        1 as ::core::ffi::c_int,
        1 as ::core::ffi::c_int,
        0 as ::core::ffi::c_int,
    );
    if err == 0 {
        return 1 as ::core::ffi::c_int;
    }
    if err == LUA_ERRRUN {
        lua_pushnil(l);
        lua_insert(l, -2 as ::core::ffi::c_int);
        return 2 as ::core::ffi::c_int;
    }
    return luaL_error(
        l,
        b"Memory allocation error in CJSON protected call\0".as_ptr() as *const ::core::ffi::c_char,
    );
}
#[no_mangle]
pub unsafe extern "C" fn lua_cjson_new(mut l: *mut lua_State) -> ::core::ffi::c_int {
    let mut reg: [luaL_Reg; 4] = [
        luaL_Reg {
            name: b"encode\0".as_ptr() as *const ::core::ffi::c_char,
            func: Some(json_encode as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int),
        },
        luaL_Reg {
            name: b"decode\0".as_ptr() as *const ::core::ffi::c_char,
            func: Some(json_decode as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int),
        },
        luaL_Reg {
            name: b"new\0".as_ptr() as *const ::core::ffi::c_char,
            func: Some(lua_cjson_new as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int),
        },
        luaL_Reg {
            name: ::core::ptr::null::<::core::ffi::c_char>(),
            func: None,
        },
    ];
    lua_getfield(
        l,
        LUA_REGISTRYINDEX,
        b"nvim.thread\0".as_ptr() as *const ::core::ffi::c_char,
    );
    let mut is_thread: bool = lua_toboolean(l, -1 as ::core::ffi::c_int) != 0;
    lua_settop(l, -1 as ::core::ffi::c_int - 1 as ::core::ffi::c_int);
    if !is_thread {
        fpconv_init();
    }
    lua_pushlightuserdata(
        l,
        ::core::ptr::from_exposed_addr_mut::<::core::ffi::c_void>(
            ((&raw mut json_empty_array).expose_addr() as uintptr_t
                & ((1 as uintptr_t) << 47 as ::core::ffi::c_int)
                    .wrapping_sub(1 as ::core::ffi::c_int as uintptr_t)) as usize,
        ),
    );
    lua_rawget(l, LUA_REGISTRYINDEX);
    if lua_type(l, -1 as ::core::ffi::c_int) == LUA_TNIL {
        lua_settop(l, -1 as ::core::ffi::c_int - 1 as ::core::ffi::c_int);
        lua_pushlightuserdata(
            l,
            ::core::ptr::from_exposed_addr_mut::<::core::ffi::c_void>(
                ((&raw mut json_empty_array).expose_addr() as uintptr_t
                    & ((1 as uintptr_t) << 47 as ::core::ffi::c_int)
                        .wrapping_sub(1 as ::core::ffi::c_int as uintptr_t))
                    as usize,
            ),
        );
        lua_createtable(l, 0 as ::core::ffi::c_int, 0 as ::core::ffi::c_int);
        lua_rawset(l, LUA_REGISTRYINDEX);
        lua_pushlightuserdata(
            l,
            ::core::ptr::from_exposed_addr_mut::<::core::ffi::c_void>(
                ((&raw mut json_array).expose_addr() as uintptr_t
                    & ((1 as uintptr_t) << 47 as ::core::ffi::c_int)
                        .wrapping_sub(1 as ::core::ffi::c_int as uintptr_t))
                    as usize,
            ),
        );
        lua_createtable(l, 0 as ::core::ffi::c_int, 0 as ::core::ffi::c_int);
        lua_rawset(l, LUA_REGISTRYINDEX);
    }
    lua_createtable(l, 0 as ::core::ffi::c_int, 0 as ::core::ffi::c_int);
    json_create_config(l);
    compat_luaL_setfuncs(l, &raw mut reg as *mut luaL_Reg, 1 as ::core::ffi::c_int);
    lua_pushlstring(
        l,
        b"cjson\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 6]>()
            .wrapping_div(::core::mem::size_of::<::core::ffi::c_char>())
            .wrapping_sub(1 as size_t),
    );
    lua_setfield(
        l,
        -2 as ::core::ffi::c_int,
        b"_NAME\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushlstring(
        l,
        b"2.1.0.11\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 9]>()
            .wrapping_div(::core::mem::size_of::<::core::ffi::c_char>())
            .wrapping_sub(1 as size_t),
    );
    lua_setfield(
        l,
        -2 as ::core::ffi::c_int,
        b"_VERSION\0".as_ptr() as *const ::core::ffi::c_char,
    );
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C" fn lua_cjson_safe_new(mut l: *mut lua_State) -> ::core::ffi::c_int {
    let mut func: [*const ::core::ffi::c_char; 3] = [
        b"decode\0".as_ptr() as *const ::core::ffi::c_char,
        b"encode\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::ptr::null::<::core::ffi::c_char>(),
    ];
    let mut i: ::core::ffi::c_int = 0;
    lua_cjson_new(l);
    lua_pushcclosure(
        l,
        Some(lua_cjson_safe_new as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int),
        0 as ::core::ffi::c_int,
    );
    lua_setfield(
        l,
        -2 as ::core::ffi::c_int,
        b"new\0".as_ptr() as *const ::core::ffi::c_char,
    );
    i = 0 as ::core::ffi::c_int;
    while !func[i as usize].is_null() {
        lua_getfield(l, -1 as ::core::ffi::c_int, func[i as usize]);
        lua_pushcclosure(
            l,
            Some(
                json_protect_conversion
                    as unsafe extern "C" fn(*mut lua_State) -> ::core::ffi::c_int,
            ),
            1 as ::core::ffi::c_int,
        );
        lua_setfield(l, -2 as ::core::ffi::c_int, func[i as usize]);
        i += 1;
    }
    return 1 as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn luaopen_cjson(mut l: *mut lua_State) -> ::core::ffi::c_int {
    lua_cjson_new(l);
    return 1 as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn luaopen_cjson_safe(mut l: *mut lua_State) -> ::core::ffi::c_int {
    lua_cjson_safe_new(l);
    return 1 as ::core::ffi::c_int;
}
#[inline]
unsafe extern "C" fn strbuf_reset(mut s: *mut strbuf_t) {
    (*s).length = 0 as size_t;
}
#[inline]
unsafe extern "C" fn strbuf_empty_length(mut s: *mut strbuf_t) -> size_t {
    return (*s)
        .size
        .wrapping_sub((*s).length)
        .wrapping_sub(1 as size_t);
}
#[inline]
unsafe extern "C" fn strbuf_ensure_empty_length(mut s: *mut strbuf_t, mut len: size_t) {
    if len > strbuf_empty_length(s) {
        strbuf_resize(s, (*s).length.wrapping_add(len));
    }
}
#[inline]
unsafe extern "C" fn strbuf_empty_ptr(mut s: *mut strbuf_t) -> *mut ::core::ffi::c_char {
    return (*s).buf.offset((*s).length as isize);
}
#[inline]
unsafe extern "C" fn strbuf_set_length(mut s: *mut strbuf_t, mut len: ::core::ffi::c_int) {
    (*s).length = len as size_t;
}
#[inline]
unsafe extern "C" fn strbuf_extend_length(mut s: *mut strbuf_t, mut len: size_t) {
    (*s).length = (*s).length.wrapping_add(len);
}
#[inline]
unsafe extern "C" fn strbuf_length(mut s: *mut strbuf_t) -> size_t {
    return (*s).length;
}
#[inline]
unsafe extern "C" fn strbuf_append_char(mut s: *mut strbuf_t, c: ::core::ffi::c_char) {
    strbuf_ensure_empty_length(s, 1 as size_t);
    let c2rust_fresh2 = (*s).length;
    (*s).length = (*s).length.wrapping_add(1);
    *(*s).buf.offset(c2rust_fresh2 as isize) = c;
}
#[inline]
unsafe extern "C" fn strbuf_append_char_unsafe(mut s: *mut strbuf_t, c: ::core::ffi::c_char) {
    let c2rust_fresh0 = (*s).length;
    (*s).length = (*s).length.wrapping_add(1);
    *(*s).buf.offset(c2rust_fresh0 as isize) = c;
}
#[inline]
unsafe extern "C" fn strbuf_append_mem(
    mut s: *mut strbuf_t,
    mut c: *const ::core::ffi::c_char,
    mut len: size_t,
) {
    strbuf_ensure_empty_length(s, len);
    memcpy(
        (*s).buf.offset((*s).length as isize) as *mut ::core::ffi::c_void,
        c as *const ::core::ffi::c_void,
        len,
    );
    (*s).length = (*s).length.wrapping_add(len);
}
#[inline]
unsafe extern "C" fn strbuf_append_mem_unsafe(
    mut s: *mut strbuf_t,
    mut c: *const ::core::ffi::c_char,
    mut len: size_t,
) {
    memcpy(
        (*s).buf.offset((*s).length as isize) as *mut ::core::ffi::c_void,
        c as *const ::core::ffi::c_void,
        len,
    );
    (*s).length = (*s).length.wrapping_add(len);
}
#[inline]
unsafe extern "C" fn strbuf_ensure_null(mut s: *mut strbuf_t) {
    *(*s).buf.offset((*s).length as isize) = 0 as ::core::ffi::c_char;
}
#[inline]
unsafe extern "C" fn strbuf_string(
    mut s: *mut strbuf_t,
    mut len: *mut size_t,
) -> *mut ::core::ffi::c_char {
    if !len.is_null() {
        *len = (*s).length;
    }
    return (*s).buf;
}
pub const FPCONV_G_FMT_BUFSIZE: ::core::ffi::c_int = 32 as ::core::ffi::c_int;
pub const true_0: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const false_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
