use crate::api::private::dispatch::KeyDict_xdl_diff_get_field;
use crate::api::private::helpers::{api_clear_error, api_free_string, api_set_error};
use crate::linematch::{block_from_lnum, linematch_nbuffers};
use crate::lua::converter::nlua_pop_keydict;
use crate::lua::executor::{api_free_luaref, nlua_pushref};
use crate::lua::ffi::{
    lua_concat, lua_createtable, lua_error, lua_gettop, lua_isnumber, lua_objlen, lua_pcall,
    lua_pushinteger, lua_pushstring, lua_pushvalue, lua_rawseti, lua_settop, lua_tolstring,
    lua_tonumber, lua_type, luaL_argerror, luaL_buffinit, luaL_error, luaL_prepbuffer,
    luaL_pushresult, luaL_where,
};
use crate::memory::strequal;
use crate::types::{
    Arena, Error, KeyDict_xdl_diff, KeySetLink, Object, OptionalKeys, String_0, int64_t,
    kErrorTypeException, kErrorTypeNone, kErrorTypeValidation, kObjectTypeBoolean,
    kObjectTypeInteger, kObjectTypeNil, linenr_T, lua_Integer, lua_State, luaL_Buffer, mmbuffer_t,
    mmfile_t, object_data as C2Rust_Unnamed, size_t, xdemitcb_t, xdemitconf_t,
    xdl_emit_hunk_consume_func_t, xpparam_t,
};
use crate::xdiff::ffi::xdl_diff;
use ::libc::{memcpy, memset};
pub const kNluaXdiffModeLocations: NluaXdiffMode = 2;
pub type NluaXdiffMode = ::core::ffi::c_uint;
pub const kNluaXdiffModeOnHunkCB: NluaXdiffMode = 1;
pub const kNluaXdiffModeUnified: NluaXdiffMode = 0;
#[derive(Copy, Clone)]
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
unsafe fn lua_pushhunk(
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
unsafe fn get_linematch_results(
    mut lstate: *mut lua_State,
    mut ma: *mut mmfile_t,
    mut mb: *mut mmfile_t,
    mut start_a: ::core::ffi::c_int,
    mut count_a: ::core::ffi::c_int,
    mut start_b: ::core::ffi::c_int,
    mut count_b: ::core::ffi::c_int,
    mut iwhite: bool,
) {
    // The two blocks as bytes; `xdl_diff` only hands over non-empty ones.
    let bytes_a: &[u8] = ::core::slice::from_raw_parts((*ma).ptr as *const u8, (*ma).size as usize);
    let bytes_b: &[u8] = ::core::slice::from_raw_parts((*mb).ptr as *const u8, (*mb).size as usize);
    let block_a = block_from_lnum(bytes_a, start_a as linenr_T + 1).unwrap_or_default();
    let block_b = block_from_lnum(bytes_b, start_b as linenr_T + 1).unwrap_or_default();
    let decisions = linematch_nbuffers(&[block_a, block_b], &[count_a, count_b], iwhite);
    let mut lnuma: ::core::ffi::c_int = start_a;
    let mut lnumb: ::core::ffi::c_int = start_b;
    let mut hunkstarta: ::core::ffi::c_int = lnuma;
    let mut hunkstartb: ::core::ffi::c_int = lnumb;
    let mut hunkcounta: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut hunkcountb: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    for (i, &decision) in decisions.iter().enumerate() {
        if i != 0 && decisions[i - 1] != decision {
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
        if decision & COMPARED_BUFFER0 != 0 {
            lnuma += 1;
            hunkcounta += 1;
        }
        if decision & COMPARED_BUFFER1 != 0 {
            lnumb += 1;
            hunkcountb += 1;
        }
    }
    lua_pushhunk(
        lstate,
        hunkstarta as ::core::ffi::c_long,
        hunkcounta as ::core::ffi::c_long,
        hunkstartb as ::core::ffi::c_long,
        hunkcountb as ::core::ffi::c_long,
    );
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
            c"on_hunk: %s".as_ptr(),
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
unsafe fn get_string_arg(mut lstate: *mut lua_State, mut idx: ::core::ffi::c_int) -> mmfile_t {
    if lua_type(lstate, idx) != LUA_TSTRING {
        luaL_argerror(lstate, idx, c"expected string".as_ptr());
    }
    let mut mf: mmfile_t = mmfile_t {
        ptr: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        size: 0,
    };
    let mut size: size_t = 0;
    mf.ptr = lua_tolstring(lstate, idx, &raw mut size) as *mut ::core::ffi::c_char;
    if size > INT_MAX as size_t {
        luaL_argerror(lstate, idx, c"string too long".as_ptr());
    }
    mf.size = size as ::core::ffi::c_int;
    return mf;
}
unsafe fn process_xdl_diff_opts(
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
            if !strequal(c"unified".as_ptr(), opts.result_type.data) {
                if strequal(c"indices".as_ptr(), opts.result_type.data) {
                    had_result_type_indices = true_0 != 0;
                } else {
                    api_set_error(
                        err,
                        kErrorTypeValidation,
                        c"not a valid result_type".as_ptr(),
                    );
                    break '_exit_1;
                }
            }
        }
        if opts.is_set__xdl_diff_ as ::core::ffi::c_ulonglong
            & (1 as ::core::ffi::c_ulonglong) << KEYSET_OPTIDX_xdl_diff__algorithm
            != 0 as ::core::ffi::c_ulonglong
        {
            if !strequal(c"myers".as_ptr(), opts.algorithm.data) {
                if strequal(c"minimal".as_ptr(), opts.algorithm.data) {
                    (*params).flags |= XDF_NEED_MINIMAL as ::core::ffi::c_ulong;
                } else if strequal(c"patience".as_ptr(), opts.algorithm.data) {
                    (*params).flags |= XDF_PATIENCE_DIFF as ::core::ffi::c_ulong;
                } else if strequal(c"histogram".as_ptr(), opts.algorithm.data) {
                    (*params).flags |= XDF_HISTOGRAM_DIFF as ::core::ffi::c_ulong;
                } else {
                    api_set_error(err, kErrorTypeValidation, c"not a valid algorithm".as_ptr());
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
                    c"linematch must be a boolean or integer".as_ptr(),
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
                    c"on_hunk is not a function".as_ptr(),
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
pub unsafe extern "C-unwind" fn nlua_xdl_diff(mut lstate: *mut lua_State) -> ::core::ffi::c_int {
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
        return luaL_error(lstate, c"Expected at least 2 arguments".as_ptr());
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
                return luaL_argerror(lstate, 3 as ::core::ffi::c_int, c"expected table".as_ptr());
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
                    c"diff operation failed".as_ptr(),
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
