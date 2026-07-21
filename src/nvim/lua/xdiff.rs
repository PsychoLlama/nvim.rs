pub use crate::src::nvim::types::{
    find_func_t, int32_t, int64_t, key_value_pair, linenr_T, luaL_Buffer, lua_Integer, lua_Number,
    lua_State, mmbuffer_t, mmfile_t, object, object_data as C2Rust_Unnamed, ptrdiff_t, s_mmbuffer,
    s_mmfile, s_xdemitcb, s_xdemitconf, s_xpparam, size_t, uint64_t, xdemitcb_t, xdemitconf_t,
    xdl_emit_hunk_consume_func_t, xpparam_t, Arena, Array, Boolean, Dict, Error, ErrorType,
    FieldHashfn, Float, Integer, KeySetLink, KeyValuePair, LuaRef, Object, ObjectType,
    OptionalKeys, String_0,
};
extern "C" {
    fn lua_gettop(L: *mut lua_State) -> ::core::ffi::c_int;
    fn lua_settop(L: *mut lua_State, idx: ::core::ffi::c_int);
    fn lua_pushvalue(L: *mut lua_State, idx: ::core::ffi::c_int);
    fn lua_isnumber(L: *mut lua_State, idx: ::core::ffi::c_int) -> ::core::ffi::c_int;
    fn lua_type(L: *mut lua_State, idx: ::core::ffi::c_int) -> ::core::ffi::c_int;
    fn lua_tonumber(L: *mut lua_State, idx: ::core::ffi::c_int) -> lua_Number;
    fn lua_tolstring(
        L: *mut lua_State,
        idx: ::core::ffi::c_int,
        len: *mut size_t,
    ) -> *const ::core::ffi::c_char;
    fn lua_objlen(L: *mut lua_State, idx: ::core::ffi::c_int) -> size_t;
    fn lua_pushinteger(L: *mut lua_State, n: lua_Integer);
    fn lua_pushstring(L: *mut lua_State, s: *const ::core::ffi::c_char);
    fn lua_createtable(L: *mut lua_State, narr: ::core::ffi::c_int, nrec: ::core::ffi::c_int);
    fn lua_rawseti(L: *mut lua_State, idx: ::core::ffi::c_int, n: ::core::ffi::c_int);
    fn lua_pcall(
        L: *mut lua_State,
        nargs: ::core::ffi::c_int,
        nresults: ::core::ffi::c_int,
        errfunc: ::core::ffi::c_int,
    ) -> ::core::ffi::c_int;
    fn lua_error(L: *mut lua_State) -> ::core::ffi::c_int;
    fn lua_concat(L: *mut lua_State, n: ::core::ffi::c_int);
    fn luaL_argerror(
        L: *mut lua_State,
        numarg: ::core::ffi::c_int,
        extramsg: *const ::core::ffi::c_char,
    ) -> ::core::ffi::c_int;
    fn luaL_where(L: *mut lua_State, lvl: ::core::ffi::c_int);
    fn luaL_error(L: *mut lua_State, fmt: *const ::core::ffi::c_char, ...) -> ::core::ffi::c_int;
    fn luaL_buffinit(L: *mut lua_State, B: *mut luaL_Buffer);
    fn luaL_prepbuffer(B: *mut luaL_Buffer) -> *mut ::core::ffi::c_char;
    fn luaL_pushresult(B: *mut luaL_Buffer);
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
    fn xfree(ptr: *mut ::core::ffi::c_void);
    fn strequal(a: *const ::core::ffi::c_char, b: *const ::core::ffi::c_char) -> bool;
    fn KeyDict_xdl_diff_get_field(str: *const ::core::ffi::c_char, len: size_t) -> *mut KeySetLink;
    fn api_free_string(value: String_0);
    fn api_clear_error(value: *mut Error);
    fn api_set_error(err: *mut Error, errType: ErrorType, format: *const ::core::ffi::c_char, ...);
    fn xdl_diff(
        mf1: *mut mmfile_t,
        mf2: *mut mmfile_t,
        xpp: *const xpparam_t,
        xecfg: *const xdemitconf_t,
        ecb: *mut xdemitcb_t,
    ) -> ::core::ffi::c_int;
    fn fastforward_buf_to_lnum(s: mmfile_t, lnum: linenr_T) -> mmfile_t;
    fn linematch_nbuffers(
        diff_blk: *mut *const mmfile_t,
        diff_len: *const ::core::ffi::c_int,
        ndiffs: size_t,
        decisions: *mut *mut ::core::ffi::c_int,
        iwhite: bool,
    ) -> size_t;
    fn nlua_pop_keydict(
        L: *mut lua_State,
        retval: *mut ::core::ffi::c_void,
        hashy: FieldHashfn,
        err_opt: *mut *mut ::core::ffi::c_char,
        arena: *mut Arena,
        err: *mut Error,
    );
    fn api_free_luaref(ref_0: LuaRef);
    fn nlua_pushref(lstate: *mut lua_State, ref_0: LuaRef);
}
pub const kErrorTypeValidation: ErrorType = 1;
pub const kErrorTypeException: ErrorType = 0;
pub const kErrorTypeNone: ErrorType = -1;
pub const kObjectTypeTabpage: ObjectType = 10;
pub const kObjectTypeWindow: ObjectType = 9;
pub const kObjectTypeBuffer: ObjectType = 8;
pub const kObjectTypeLuaRef: ObjectType = 7;
pub const kObjectTypeDict: ObjectType = 6;
pub const kObjectTypeArray: ObjectType = 5;
pub const kObjectTypeString: ObjectType = 4;
pub const kObjectTypeFloat: ObjectType = 3;
pub const kObjectTypeInteger: ObjectType = 2;
pub const kObjectTypeBoolean: ObjectType = 1;
pub const kObjectTypeNil: ObjectType = 0;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct KeyDict_xdl_diff {
    pub is_set__xdl_diff_: OptionalKeys,
    pub on_hunk: LuaRef,
    pub result_type: String_0,
    pub algorithm: String_0,
    pub ctxlen: Integer,
    pub interhunkctxlen: Integer,
    pub linematch: Object,
    pub ignore_whitespace: Boolean,
    pub ignore_whitespace_change: Boolean,
    pub ignore_whitespace_change_at_eol: Boolean,
    pub ignore_cr_at_eol: Boolean,
    pub ignore_blank_lines: Boolean,
    pub indent_heuristic: Boolean,
}
pub const kNluaXdiffModeLocations: NluaXdiffMode = 2;
pub type NluaXdiffMode = ::core::ffi::c_uint;
pub const kNluaXdiffModeOnHunkCB: NluaXdiffMode = 1;
pub const kNluaXdiffModeUnified: NluaXdiffMode = 0;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct hunkpriv_t {
    pub lstate: *mut lua_State,
    pub err: *mut Error,
    pub ma: *mut mmfile_t,
    pub mb: *mut mmfile_t,
    pub linematch: int64_t,
    pub iwhite: bool,
}
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const BUFSIZ: ::core::ffi::c_int = 8192 as ::core::ffi::c_int;
pub const LUA_TSTRING: ::core::ffi::c_int = 4 as ::core::ffi::c_int;
pub const LUA_TTABLE: ::core::ffi::c_int = 5 as ::core::ffi::c_int;
pub const LUA_TFUNCTION: ::core::ffi::c_int = 6 as ::core::ffi::c_int;
pub const INT64_MAX: ::core::ffi::c_long = 9223372036854775807 as ::core::ffi::c_long;
pub const KEYSET_OPTIDX_xdl_diff__ctxlen: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX_xdl_diff__on_hunk: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX_xdl_diff__algorithm: ::core::ffi::c_int = 3 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX_xdl_diff__linematch: ::core::ffi::c_int = 4 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX_xdl_diff__result_type: ::core::ffi::c_int = 5 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX_xdl_diff__interhunkctxlen: ::core::ffi::c_int = 6 as ::core::ffi::c_int;
pub const KEYDICT_INIT: KeyDict_xdl_diff = KeyDict_xdl_diff {
    is_set__xdl_diff_: 0 as OptionalKeys,
    on_hunk: 0,
    result_type: String_0 {
        data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        size: 0,
    },
    algorithm: String_0 {
        data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        size: 0,
    },
    ctxlen: 0,
    interhunkctxlen: 0,
    linematch: Object {
        type_0: kObjectTypeNil,
        data: C2Rust_Unnamed { boolean: false },
    },
    ignore_whitespace: false,
    ignore_whitespace_change: false,
    ignore_whitespace_change_at_eol: false,
    ignore_cr_at_eol: false,
    ignore_blank_lines: false,
    indent_heuristic: false,
};
pub const XDF_NEED_MINIMAL: ::core::ffi::c_int =
    (1 as ::core::ffi::c_int) << 0 as ::core::ffi::c_int;
pub const XDF_IGNORE_WHITESPACE: ::core::ffi::c_int =
    (1 as ::core::ffi::c_int) << 1 as ::core::ffi::c_int;
pub const XDF_IGNORE_WHITESPACE_CHANGE: ::core::ffi::c_int =
    (1 as ::core::ffi::c_int) << 2 as ::core::ffi::c_int;
pub const XDF_IGNORE_WHITESPACE_AT_EOL: ::core::ffi::c_int =
    (1 as ::core::ffi::c_int) << 3 as ::core::ffi::c_int;
pub const XDF_IGNORE_CR_AT_EOL: ::core::ffi::c_int =
    (1 as ::core::ffi::c_int) << 4 as ::core::ffi::c_int;
pub const XDF_IGNORE_BLANK_LINES: ::core::ffi::c_int =
    (1 as ::core::ffi::c_int) << 7 as ::core::ffi::c_int;
pub const XDF_PATIENCE_DIFF: ::core::ffi::c_int =
    (1 as ::core::ffi::c_int) << 14 as ::core::ffi::c_int;
pub const XDF_HISTOGRAM_DIFF: ::core::ffi::c_int =
    (1 as ::core::ffi::c_int) << 15 as ::core::ffi::c_int;
pub const XDF_INDENT_HEURISTIC: ::core::ffi::c_int =
    (1 as ::core::ffi::c_int) << 23 as ::core::ffi::c_int;
pub const COMPARED_BUFFER0: ::core::ffi::c_int =
    (1 as ::core::ffi::c_int) << 0 as ::core::ffi::c_int;
pub const COMPARED_BUFFER1: ::core::ffi::c_int =
    (1 as ::core::ffi::c_int) << 1 as ::core::ffi::c_int;
unsafe extern "C" fn lua_pushhunk(
    mut lstate: *mut lua_State,
    mut start_a: ::core::ffi::c_long,
    mut count_a: ::core::ffi::c_long,
    mut start_b: ::core::ffi::c_long,
    mut count_b: ::core::ffi::c_long,
) {
    if count_a > 0 as ::core::ffi::c_long {
        start_a += 1 as ::core::ffi::c_long;
    }
    if count_b > 0 as ::core::ffi::c_long {
        start_b += 1 as ::core::ffi::c_long;
    }
    lua_createtable(lstate, 0 as ::core::ffi::c_int, 0 as ::core::ffi::c_int);
    lua_pushinteger(lstate, start_a as lua_Integer);
    lua_rawseti(lstate, -2 as ::core::ffi::c_int, 1 as ::core::ffi::c_int);
    lua_pushinteger(lstate, count_a as lua_Integer);
    lua_rawseti(lstate, -2 as ::core::ffi::c_int, 2 as ::core::ffi::c_int);
    lua_pushinteger(lstate, start_b as lua_Integer);
    lua_rawseti(lstate, -2 as ::core::ffi::c_int, 3 as ::core::ffi::c_int);
    lua_pushinteger(lstate, count_b as lua_Integer);
    lua_rawseti(lstate, -2 as ::core::ffi::c_int, 4 as ::core::ffi::c_int);
    lua_rawseti(
        lstate,
        -2 as ::core::ffi::c_int,
        lua_objlen(lstate, -2 as ::core::ffi::c_int) as ::core::ffi::c_int
            + 1 as ::core::ffi::c_int,
    );
}
unsafe extern "C" fn get_linematch_results(
    mut lstate: *mut lua_State,
    mut ma: *mut mmfile_t,
    mut mb: *mut mmfile_t,
    mut start_a: ::core::ffi::c_int,
    mut count_a: ::core::ffi::c_int,
    mut start_b: ::core::ffi::c_int,
    mut count_b: ::core::ffi::c_int,
    mut iwhite: bool,
) {
    let mut ma0: mmfile_t = fastforward_buf_to_lnum(*ma, start_a as linenr_T + 1 as linenr_T);
    let mut mb0: mmfile_t = fastforward_buf_to_lnum(*mb, start_b as linenr_T + 1 as linenr_T);
    let mut diff_begin: [*const mmfile_t; 2] = [
        &raw mut ma0 as *const mmfile_t,
        &raw mut mb0 as *const mmfile_t,
    ];
    let mut diff_length: [::core::ffi::c_int; 2] = [count_a, count_b];
    let mut decisions: *mut ::core::ffi::c_int = ::core::ptr::null_mut::<::core::ffi::c_int>();
    let mut decisions_length: size_t = linematch_nbuffers(
        &raw mut diff_begin as *mut *const mmfile_t,
        &raw mut diff_length as *mut ::core::ffi::c_int,
        2 as size_t,
        &raw mut decisions,
        iwhite,
    );
    let mut lnuma: ::core::ffi::c_int = start_a;
    let mut lnumb: ::core::ffi::c_int = start_b;
    let mut hunkstarta: ::core::ffi::c_int = lnuma;
    let mut hunkstartb: ::core::ffi::c_int = lnumb;
    let mut hunkcounta: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut hunkcountb: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut i: size_t = 0 as size_t;
    while i < decisions_length {
        if i != 0
            && *decisions.offset(i.wrapping_sub(1 as size_t) as isize)
                != *decisions.offset(i as isize)
        {
            lua_pushhunk(
                lstate,
                hunkstarta as ::core::ffi::c_long,
                hunkcounta as ::core::ffi::c_long,
                hunkstartb as ::core::ffi::c_long,
                hunkcountb as ::core::ffi::c_long,
            );
            hunkstarta = lnuma;
            hunkstartb = lnumb;
            hunkcounta = 0 as ::core::ffi::c_int;
            hunkcountb = 0 as ::core::ffi::c_int;
        }
        if *decisions.offset(i as isize) & COMPARED_BUFFER0 != 0 {
            lnuma += 1;
            hunkcounta += 1;
        }
        if *decisions.offset(i as isize) & COMPARED_BUFFER1 != 0 {
            lnumb += 1;
            hunkcountb += 1;
        }
        i = i.wrapping_add(1);
    }
    lua_pushhunk(
        lstate,
        hunkstarta as ::core::ffi::c_long,
        hunkcounta as ::core::ffi::c_long,
        hunkstartb as ::core::ffi::c_long,
        hunkcountb as ::core::ffi::c_long,
    );
    xfree(decisions as *mut ::core::ffi::c_void);
}
unsafe extern "C" fn write_string(
    mut priv_0: *mut ::core::ffi::c_void,
    mut mb: *mut mmbuffer_t,
    mut nbuf: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let mut buf: *mut luaL_Buffer = priv_0 as *mut luaL_Buffer;
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i < nbuf {
        let size: ::core::ffi::c_int = (*mb.offset(i as isize)).size;
        let mut total: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while total < size {
            let tocopy: ::core::ffi::c_int = if size - total
                < (if 8192 as ::core::ffi::c_int > 16384 as ::core::ffi::c_int {
                    8192 as ::core::ffi::c_int
                } else {
                    8192 as ::core::ffi::c_int
                }) {
                size - total
            } else if 8192 as ::core::ffi::c_int > 16384 as ::core::ffi::c_int {
                8192 as ::core::ffi::c_int
            } else {
                8192 as ::core::ffi::c_int
            };
            let mut p: *mut ::core::ffi::c_char = luaL_prepbuffer(buf);
            if p.is_null() {
                return -1 as ::core::ffi::c_int;
            }
            memcpy(
                p as *mut ::core::ffi::c_void,
                (*mb.offset(i as isize)).ptr.offset(total as isize) as *const ::core::ffi::c_void,
                tocopy as ::core::ffi::c_uint as size_t,
            );
            (*buf).p = (*buf).p.offset(tocopy as ::core::ffi::c_uint as isize);
            total += LUAL_BUFFERSIZE;
        }
        i += 1;
    }
    return 0 as ::core::ffi::c_int;
}
unsafe extern "C" fn hunk_locations_cb(
    mut start_a: ::core::ffi::c_int,
    mut count_a: ::core::ffi::c_int,
    mut start_b: ::core::ffi::c_int,
    mut count_b: ::core::ffi::c_int,
    mut cb_data: *mut ::core::ffi::c_void,
) -> ::core::ffi::c_int {
    let mut priv_0: *mut hunkpriv_t = cb_data as *mut hunkpriv_t;
    let mut lstate: *mut lua_State = (*priv_0).lstate;
    if (*priv_0).linematch > 0 as int64_t && (count_a + count_b) as int64_t <= (*priv_0).linematch {
        get_linematch_results(
            lstate,
            (*priv_0).ma,
            (*priv_0).mb,
            start_a,
            count_a,
            start_b,
            count_b,
            (*priv_0).iwhite,
        );
    } else {
        lua_pushhunk(
            lstate,
            start_a as ::core::ffi::c_long,
            count_a as ::core::ffi::c_long,
            start_b as ::core::ffi::c_long,
            count_b as ::core::ffi::c_long,
        );
    }
    return 0 as ::core::ffi::c_int;
}
unsafe extern "C" fn call_on_hunk_cb(
    mut start_a: ::core::ffi::c_int,
    mut count_a: ::core::ffi::c_int,
    mut start_b: ::core::ffi::c_int,
    mut count_b: ::core::ffi::c_int,
    mut cb_data: *mut ::core::ffi::c_void,
) -> ::core::ffi::c_int {
    if count_a > 0 as ::core::ffi::c_int {
        start_a += 1 as ::core::ffi::c_int;
    }
    if count_b > 0 as ::core::ffi::c_int {
        start_b += 1 as ::core::ffi::c_int;
    }
    let mut priv_0: *mut hunkpriv_t = cb_data as *mut hunkpriv_t;
    let mut lstate: *mut lua_State = (*priv_0).lstate;
    let mut err: *mut Error = (*priv_0).err;
    let fidx: ::core::ffi::c_int = lua_gettop(lstate);
    lua_pushvalue(lstate, fidx);
    lua_pushinteger(lstate, start_a as lua_Integer);
    lua_pushinteger(lstate, count_a as lua_Integer);
    lua_pushinteger(lstate, start_b as lua_Integer);
    lua_pushinteger(lstate, count_b as lua_Integer);
    if lua_pcall(
        lstate,
        4 as ::core::ffi::c_int,
        1 as ::core::ffi::c_int,
        0 as ::core::ffi::c_int,
    ) != 0 as ::core::ffi::c_int
    {
        api_set_error(
            err,
            kErrorTypeException,
            b"on_hunk: %s\0".as_ptr() as *const ::core::ffi::c_char,
            lua_tolstring(
                lstate,
                -1 as ::core::ffi::c_int,
                ::core::ptr::null_mut::<size_t>(),
            ),
        );
        return -1 as ::core::ffi::c_int;
    }
    let mut r: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    if lua_isnumber(lstate, -1 as ::core::ffi::c_int) != 0 {
        r = lua_tonumber(lstate, -1 as ::core::ffi::c_int) as ::core::ffi::c_int;
    }
    lua_settop(lstate, -1 as ::core::ffi::c_int - 1 as ::core::ffi::c_int);
    lua_settop(lstate, fidx);
    return r;
}
unsafe extern "C" fn get_string_arg(
    mut lstate: *mut lua_State,
    mut idx: ::core::ffi::c_int,
) -> mmfile_t {
    if lua_type(lstate, idx) != LUA_TSTRING {
        luaL_argerror(
            lstate,
            idx,
            b"expected string\0".as_ptr() as *const ::core::ffi::c_char,
        );
    }
    let mut mf: mmfile_t = mmfile_t {
        ptr: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        size: 0,
    };
    let mut size: size_t = 0;
    mf.ptr = lua_tolstring(lstate, idx, &raw mut size) as *mut ::core::ffi::c_char;
    if size > INT_MAX as size_t {
        luaL_argerror(
            lstate,
            idx,
            b"string too long\0".as_ptr() as *const ::core::ffi::c_char,
        );
    }
    mf.size = size as ::core::ffi::c_int;
    return mf;
}
unsafe extern "C" fn process_xdl_diff_opts(
    mut lstate: *mut lua_State,
    mut cfg: *mut xdemitconf_t,
    mut params: *mut xpparam_t,
    mut linematch: *mut int64_t,
    mut err: *mut Error,
) -> NluaXdiffMode {
    let mut opts: KeyDict_xdl_diff = KEYDICT_INIT;
    let mut err_param: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    nlua_pop_keydict(
        lstate,
        &raw mut opts as *mut ::core::ffi::c_void,
        Some(
            KeyDict_xdl_diff_get_field
                as unsafe extern "C" fn(*const ::core::ffi::c_char, size_t) -> *mut KeySetLink,
        ),
        &raw mut err_param,
        ::core::ptr::null_mut::<Arena>(),
        err,
    );
    let mut mode: NluaXdiffMode = kNluaXdiffModeUnified;
    let mut had_result_type_indices: bool = false_0 != 0;
    '_exit_1: {
        if opts.is_set__xdl_diff_ as ::core::ffi::c_ulonglong
            & (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_xdl_diff__result_type
            != 0 as ::core::ffi::c_ulonglong
        {
            if !strequal(
                b"unified\0".as_ptr() as *const ::core::ffi::c_char,
                opts.result_type.data,
            ) {
                if strequal(
                    b"indices\0".as_ptr() as *const ::core::ffi::c_char,
                    opts.result_type.data,
                ) {
                    had_result_type_indices = true_0 != 0;
                } else {
                    api_set_error(
                        err,
                        kErrorTypeValidation,
                        b"not a valid result_type\0".as_ptr() as *const ::core::ffi::c_char,
                    );
                    break '_exit_1;
                }
            }
        }
        if opts.is_set__xdl_diff_ as ::core::ffi::c_ulonglong
            & (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_xdl_diff__algorithm
            != 0 as ::core::ffi::c_ulonglong
        {
            if !strequal(
                b"myers\0".as_ptr() as *const ::core::ffi::c_char,
                opts.algorithm.data,
            ) {
                if strequal(
                    b"minimal\0".as_ptr() as *const ::core::ffi::c_char,
                    opts.algorithm.data,
                ) {
                    (*params).flags |= XDF_NEED_MINIMAL as ::core::ffi::c_ulong;
                } else if strequal(
                    b"patience\0".as_ptr() as *const ::core::ffi::c_char,
                    opts.algorithm.data,
                ) {
                    (*params).flags |= XDF_PATIENCE_DIFF as ::core::ffi::c_ulong;
                } else if strequal(
                    b"histogram\0".as_ptr() as *const ::core::ffi::c_char,
                    opts.algorithm.data,
                ) {
                    (*params).flags |= XDF_HISTOGRAM_DIFF as ::core::ffi::c_ulong;
                } else {
                    api_set_error(
                        err,
                        kErrorTypeValidation,
                        b"not a valid algorithm\0".as_ptr() as *const ::core::ffi::c_char,
                    );
                    break '_exit_1;
                }
            }
        }
        if opts.is_set__xdl_diff_ as ::core::ffi::c_ulonglong
            & (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_xdl_diff__ctxlen
            != 0 as ::core::ffi::c_ulonglong
        {
            (*cfg).ctxlen = opts.ctxlen as ::core::ffi::c_long;
        }
        if opts.is_set__xdl_diff_ as ::core::ffi::c_ulonglong
            & (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_xdl_diff__interhunkctxlen
            != 0 as ::core::ffi::c_ulonglong
        {
            (*cfg).interhunkctxlen = opts.interhunkctxlen as ::core::ffi::c_long;
        }
        if opts.is_set__xdl_diff_ as ::core::ffi::c_ulonglong
            & (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_xdl_diff__linematch
            != 0 as ::core::ffi::c_ulonglong
        {
            if opts.linematch.type_0 as ::core::ffi::c_uint
                == kObjectTypeBoolean as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                *linematch = (if opts.linematch.data.boolean as ::core::ffi::c_int != 0 {
                    INT64_MAX
                } else {
                    0 as ::core::ffi::c_long
                }) as int64_t;
            } else if opts.linematch.type_0 as ::core::ffi::c_uint
                == kObjectTypeInteger as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                *linematch = opts.linematch.data.integer as int64_t;
            } else {
                api_set_error(
                    err,
                    kErrorTypeValidation,
                    b"linematch must be a boolean or integer\0".as_ptr()
                        as *const ::core::ffi::c_char,
                );
                break '_exit_1;
            }
        }
        (*params).flags |= (if opts.ignore_whitespace as ::core::ffi::c_int != 0 {
            XDF_IGNORE_WHITESPACE
        } else {
            0 as ::core::ffi::c_int
        }) as ::core::ffi::c_ulong;
        (*params).flags |= (if opts.ignore_whitespace_change as ::core::ffi::c_int != 0 {
            XDF_IGNORE_WHITESPACE_CHANGE
        } else {
            0 as ::core::ffi::c_int
        }) as ::core::ffi::c_ulong;
        (*params).flags |= (if opts.ignore_whitespace_change_at_eol as ::core::ffi::c_int != 0 {
            XDF_IGNORE_WHITESPACE_AT_EOL
        } else {
            0 as ::core::ffi::c_int
        }) as ::core::ffi::c_ulong;
        (*params).flags |= (if opts.ignore_cr_at_eol as ::core::ffi::c_int != 0 {
            XDF_IGNORE_CR_AT_EOL
        } else {
            0 as ::core::ffi::c_int
        }) as ::core::ffi::c_ulong;
        (*params).flags |= (if opts.ignore_blank_lines as ::core::ffi::c_int != 0 {
            XDF_IGNORE_BLANK_LINES
        } else {
            0 as ::core::ffi::c_int
        }) as ::core::ffi::c_ulong;
        (*params).flags |= (if opts.indent_heuristic as ::core::ffi::c_int != 0 {
            XDF_INDENT_HEURISTIC
        } else {
            0 as ::core::ffi::c_int
        }) as ::core::ffi::c_ulong;
        if opts.is_set__xdl_diff_ as ::core::ffi::c_ulonglong
            & (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_xdl_diff__on_hunk
            != 0 as ::core::ffi::c_ulonglong
        {
            mode = kNluaXdiffModeOnHunkCB;
            nlua_pushref(lstate, opts.on_hunk);
            if lua_type(lstate, -1 as ::core::ffi::c_int) != LUA_TFUNCTION {
                api_set_error(
                    err,
                    kErrorTypeValidation,
                    b"on_hunk is not a function\0".as_ptr() as *const ::core::ffi::c_char,
                );
            }
        } else if had_result_type_indices {
            mode = kNluaXdiffModeLocations;
        }
    }
    api_free_string(opts.result_type);
    api_free_string(opts.algorithm);
    api_free_luaref(opts.on_hunk);
    return mode;
}
#[no_mangle]
pub unsafe extern "C" fn nlua_xdl_diff(mut lstate: *mut lua_State) -> ::core::ffi::c_int {
    let mut buf: luaL_Buffer = luaL_Buffer {
        p: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        lvl: 0,
        L: ::core::ptr::null_mut::<lua_State>(),
        buffer: [0; 8192],
    };
    let mut priv_0: hunkpriv_t = hunkpriv_t {
        lstate: ::core::ptr::null_mut::<lua_State>(),
        err: ::core::ptr::null_mut::<Error>(),
        ma: ::core::ptr::null_mut::<mmfile_t>(),
        mb: ::core::ptr::null_mut::<mmfile_t>(),
        linematch: 0,
        iwhite: false,
    };
    if lua_gettop(lstate) < 2 as ::core::ffi::c_int {
        return luaL_error(
            lstate,
            b"Expected at least 2 arguments\0".as_ptr() as *const ::core::ffi::c_char,
        );
    }
    let mut ma: mmfile_t = get_string_arg(lstate, 1 as ::core::ffi::c_int);
    let mut mb: mmfile_t = get_string_arg(lstate, 2 as ::core::ffi::c_int);
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut cfg: xdemitconf_t = xdemitconf_t {
        ctxlen: 0,
        interhunkctxlen: 0,
        flags: 0,
        find_func: None,
        find_func_priv: ::core::ptr::null_mut::<::core::ffi::c_void>(),
        hunk_func: None,
    };
    let mut params: xpparam_t = xpparam_t {
        flags: 0,
        anchors: ::core::ptr::null_mut::<*mut ::core::ffi::c_char>(),
        anchors_nr: 0,
    };
    let mut ecb: xdemitcb_t = xdemitcb_t {
        priv_0: ::core::ptr::null_mut::<::core::ffi::c_void>(),
        out_hunk: None,
        out_line: None,
    };
    let mut linematch: int64_t = 0 as int64_t;
    memset(
        &raw mut cfg as *mut ::core::ffi::c_void,
        0 as ::core::ffi::c_int,
        ::core::mem::size_of::<xdemitconf_t>(),
    );
    memset(
        &raw mut params as *mut ::core::ffi::c_void,
        0 as ::core::ffi::c_int,
        ::core::mem::size_of::<xpparam_t>(),
    );
    memset(
        &raw mut ecb as *mut ::core::ffi::c_void,
        0 as ::core::ffi::c_int,
        ::core::mem::size_of::<xdemitcb_t>(),
    );
    let mut mode: NluaXdiffMode = kNluaXdiffModeUnified;
    '_exit_0: {
        if lua_gettop(lstate) == 3 as ::core::ffi::c_int {
            if lua_type(lstate, 3 as ::core::ffi::c_int) != LUA_TTABLE {
                return luaL_argerror(
                    lstate,
                    3 as ::core::ffi::c_int,
                    b"expected table\0".as_ptr() as *const ::core::ffi::c_char,
                );
            }
            mode = process_xdl_diff_opts(
                lstate,
                &raw mut cfg,
                &raw mut params,
                &raw mut linematch,
                &raw mut err,
            );
            if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
                break '_exit_0;
            }
        }
        buf = luaL_Buffer {
            p: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            lvl: 0,
            L: ::core::ptr::null_mut::<lua_State>(),
            buffer: [0; 8192],
        };
        priv_0 = hunkpriv_t {
            lstate: ::core::ptr::null_mut::<lua_State>(),
            err: ::core::ptr::null_mut::<Error>(),
            ma: ::core::ptr::null_mut::<mmfile_t>(),
            mb: ::core::ptr::null_mut::<mmfile_t>(),
            linematch: 0,
            iwhite: false,
        };
        match mode as ::core::ffi::c_uint {
            0 => {
                luaL_buffinit(lstate, &raw mut buf);
                ecb.priv_0 = &raw mut buf as *mut ::core::ffi::c_void;
                ecb.out_line = Some(
                    write_string
                        as unsafe extern "C" fn(
                            *mut ::core::ffi::c_void,
                            *mut mmbuffer_t,
                            ::core::ffi::c_int,
                        ) -> ::core::ffi::c_int,
                )
                    as Option<
                        unsafe extern "C" fn(
                            *mut ::core::ffi::c_void,
                            *mut mmbuffer_t,
                            ::core::ffi::c_int,
                        ) -> ::core::ffi::c_int,
                    >;
            }
            1 => {
                cfg.hunk_func = Some(
                    call_on_hunk_cb
                        as unsafe extern "C" fn(
                            ::core::ffi::c_int,
                            ::core::ffi::c_int,
                            ::core::ffi::c_int,
                            ::core::ffi::c_int,
                            *mut ::core::ffi::c_void,
                        ) -> ::core::ffi::c_int,
                ) as xdl_emit_hunk_consume_func_t;
                priv_0 = hunkpriv_t {
                    lstate: lstate,
                    err: &raw mut err,
                    ma: ::core::ptr::null_mut::<mmfile_t>(),
                    mb: ::core::ptr::null_mut::<mmfile_t>(),
                    linematch: 0,
                    iwhite: false,
                };
                ecb.priv_0 = &raw mut priv_0 as *mut ::core::ffi::c_void;
            }
            2 => {
                cfg.hunk_func = Some(
                    hunk_locations_cb
                        as unsafe extern "C" fn(
                            ::core::ffi::c_int,
                            ::core::ffi::c_int,
                            ::core::ffi::c_int,
                            ::core::ffi::c_int,
                            *mut ::core::ffi::c_void,
                        ) -> ::core::ffi::c_int,
                ) as xdl_emit_hunk_consume_func_t;
                priv_0 = hunkpriv_t {
                    lstate: lstate,
                    err: ::core::ptr::null_mut::<Error>(),
                    ma: &raw mut ma,
                    mb: &raw mut mb,
                    linematch: linematch,
                    iwhite: params.flags & XDF_IGNORE_WHITESPACE as ::core::ffi::c_ulong
                        > 0 as ::core::ffi::c_ulong,
                };
                ecb.priv_0 = &raw mut priv_0 as *mut ::core::ffi::c_void;
                lua_createtable(lstate, 0 as ::core::ffi::c_int, 0 as ::core::ffi::c_int);
            }
            _ => {}
        }
        if xdl_diff(
            &raw mut ma,
            &raw mut mb,
            &raw mut params,
            &raw mut cfg,
            &raw mut ecb,
        ) == -1 as ::core::ffi::c_int
        {
            if !(err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int) {
                api_set_error(
                    &raw mut err,
                    kErrorTypeException,
                    b"diff operation failed\0".as_ptr() as *const ::core::ffi::c_char,
                );
            }
        }
    }
    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        luaL_where(lstate, 1 as ::core::ffi::c_int);
        lua_pushstring(lstate, err.msg);
        api_clear_error(&raw mut err);
        lua_concat(lstate, 2 as ::core::ffi::c_int);
        return lua_error(lstate);
    } else if mode as ::core::ffi::c_uint
        == kNluaXdiffModeUnified as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        luaL_pushresult(&raw mut buf);
        return 1 as ::core::ffi::c_int;
    } else if mode as ::core::ffi::c_uint
        == kNluaXdiffModeLocations as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        return 1 as ::core::ffi::c_int;
    }
    return 0 as ::core::ffi::c_int;
}
pub const INT_MAX: ::core::ffi::c_int = __INT_MAX__;
pub const LUAL_BUFFERSIZE: ::core::ffi::c_int = if BUFSIZ > 16384 as ::core::ffi::c_int {
    8192 as ::core::ffi::c_int
} else {
    BUFSIZ
};
pub const true_0: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const false_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const __INT_MAX__: ::core::ffi::c_int = 2147483647 as ::core::ffi::c_int;
