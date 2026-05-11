//! Lua typval execution and exec paths
//!
//! Phase C migration from executor.c.
//! Migrates: nlua_typval_exec, nlua_typval_eval, nlua_typval_call,
//!           nlua_call_user_expand_func, nlua_exec_ga, typval_exec_lua_callable,
//!           nlua_exec, nlua_call_pop_retval, nlua_funcref_str.

use std::ffi::{c_char, c_int};
use std::ptr;

use nvim_api::{Arena, Array, Error, NvimString, Object, ObjectData};

use crate::state::LuaState;

// LuaRetMode values (match C enum in executor.h)
const K_RET_OBJECT: c_int = 0;
const K_RET_NIL_BOOL: c_int = 1;
const K_RET_LUAREF: c_int = 2;
const K_RET_MULTI: c_int = 3;

// kObjectTypeLuaRef
const K_OBJECT_TYPE_LUAREF: c_int = 7;

// Error kind: none
const K_ERROR_TYPE_NONE: c_int = -1;

// FCERR_NONE and FCERR_OTHER (from eval/typval_defs.h)
const FCERR_NONE: c_int = 0;
const FCERR_OTHER: c_int = 10;

// =============================================================================
// C FFI declarations
// =============================================================================

extern "C" {
    // Global Lua state
    fn get_global_lstate() -> *mut LuaState;

    // Lua C API
    fn lua_gettop(lstate: *mut LuaState) -> c_int;
    fn lua_settop(lstate: *mut LuaState, idx: c_int);
    fn lua_type(lstate: *mut LuaState, idx: c_int) -> c_int;
    fn lua_toboolean(lstate: *mut LuaState, idx: c_int) -> c_int;
    fn lua_tolstring(lstate: *mut LuaState, idx: c_int, len: *mut usize) -> *const c_char;
    fn lua_checkstack(lstate: *mut LuaState, extra: c_int) -> c_int;
    fn luaL_loadbuffer(
        lstate: *mut LuaState,
        buf: *const c_char,
        sz: usize,
        name: *const c_char,
    ) -> c_int;

    fn nlua_pop_typval(lstate: *mut LuaState, ret_tv: *mut std::ffi::c_void) -> bool;
    fn nlua_pcall(lstate: *mut LuaState, nargs: c_int, nresults: c_int) -> c_int;

    // Phase B function
    fn nlua_error(lstate: *mut LuaState, msg: *const c_char);

    // rs_check_secure (Rust, nvim-eval crate)
    fn rs_check_secure() -> c_int;

    // Rust executor helpers
    fn rs_lua_eval_cmd_size(str_size: usize) -> usize;
    fn rs_lua_call_cmd_size(str_size: usize) -> usize;
    fn rs_lcmd_fits_iosize(lcmd_len: usize, iosize: usize) -> bool;
    fn rs_lua_mode_ret(mode: c_int) -> c_int;

    // Phase C C accessors (added to executor.c)
    fn nvim_get_iobuff() -> *mut c_char;
    fn nvim_lua_get_iosize() -> usize;
    fn nvim_lua_api_set_error_validation(err: *mut Error, len: c_int, str_: *const c_char);
    fn nvim_lua_api_set_error_exception(err: *mut Error, len: c_int, str_: *const c_char);
    fn nvim_lua_push_all_typvals(
        lstate: *mut LuaState,
        args: *mut std::ffi::c_void,
        argcount: c_int,
        special: bool,
    );
    fn nvim_lua_funcref_info(
        lstate: *mut LuaState,
        src_out: *mut *mut c_char,
        line_out: *mut c_int,
    ) -> bool;
    fn nvim_callback_arena_printf_luaref(arena: *mut Arena, ref_: c_int) -> *mut c_char;
    fn nvim_callback_arena_printf_luaref_src(
        arena: *mut Arena,
        ref_: c_int,
        src: *const c_char,
        line: c_int,
    ) -> *mut c_char;
    fn nvim_lua_expand_push_args(lstate: *mut LuaState, xp: *const std::ffi::c_void);
    fn nvim_lua_init_typval_zero(ret_tv: *mut std::ffi::c_void);

    // ga_concat_strings
    fn ga_concat_strings(ga: *const std::ffi::c_void, sep: *const c_char) -> *mut c_char;

    // Memory management
    fn xmalloc(size: usize) -> *mut std::ffi::c_void;
    fn xfree(ptr: *mut std::ffi::c_void);

    // rs_arena_array (nvim_api crate)
    fn rs_arena_array(arena: *mut Arena, max_size: usize) -> Array;
}

/// lua_pop: lua_settop(L, -(n)-1)
#[inline]
unsafe fn lua_pop(lstate: *mut LuaState, n: c_int) {
    lua_settop(lstate, -n - 1);
}

/// Push an Object onto the Lua stack.
///
/// Delegates to the to_lua module's push_object wrapper.
/// `nvim_api::Object` and `to_lua::Object` are both `#[repr(C)]` with the
/// same field layout; the pointer cast is a no-op at the ABI level.
#[inline]
unsafe fn push_object(lstate: *mut LuaState, obj: *mut Object, flags: c_int) {
    crate::to_lua::rs_nlua_push_object(lstate, obj.cast::<crate::to_lua::Object>(), flags);
}

/// Pop an Object from the Lua stack.
///
/// Delegates to the from_lua module's pop_object wrapper.
/// All pointer casts are no-ops: the pointed-to types are `#[repr(C)]`
/// structs with identical field layouts across crate boundaries.
#[allow(clippy::transmute_ptr_to_ptr)]
#[inline]
unsafe fn pop_object(
    lstate: *mut LuaState,
    keep_ref: bool,
    arena: *mut Arena,
    err: *mut Error,
) -> Object {
    // Safety: to_lua::Object and nvim_api::Object have the same #[repr(C)] layout.
    std::mem::transmute::<crate::to_lua::Object, Object>(crate::from_lua::rs_nlua_pop_object(
        lstate,
        keep_ref,
        arena.cast::<crate::from_lua::Arena>(),
        err.cast::<crate::from_lua::Error>(),
    ))
}

/// lua_isnil: lua_type(L, n) == LUA_TNIL (0)
#[inline]
unsafe fn lua_isnil(lstate: *mut LuaState, n: c_int) -> bool {
    lua_type(lstate, n) == 0
}

/// lua_isfunction: lua_type(L, n) == LUA_TFUNCTION (6)
#[inline]
unsafe fn lua_isfunction(lstate: *mut LuaState, n: c_int) -> bool {
    lua_type(lstate, n) == 6
}

/// Check if error is set (ERROR_SET macro equivalent)
#[inline]
const unsafe fn error_is_set(err: *const Error) -> bool {
    !err.is_null() && (*err).err_type != K_ERROR_TYPE_NONE
}

// =============================================================================
// Rust implementations (Phase C)
// =============================================================================

/// Core Lua string execution helper.
///
/// Checks secure mode, loads `lcmd` into Lua, pushes typval args, pcalls,
/// and optionally pops result into `ret_tv`.
///
/// # Safety
///
/// All pointers must be valid. `args` is `typval_T *`, `ret_tv` is `typval_T *`.
#[unsafe(export_name = "nlua_typval_exec")]
pub unsafe extern "C" fn rs_nlua_typval_exec(
    lcmd: *const c_char,
    lcmd_len: usize,
    name: *const c_char,
    args: *mut std::ffi::c_void,
    argcount: c_int,
    special: bool,
    ret_tv: *mut std::ffi::c_void,
) {
    if rs_check_secure() != 0 {
        if !ret_tv.is_null() {
            nvim_lua_init_typval_zero(ret_tv);
        }
        return;
    }

    let lstate = get_global_lstate();
    if luaL_loadbuffer(lstate, lcmd, lcmd_len, name) != 0 {
        nlua_error(lstate, c"E5107: Lua: %.*s".as_ptr());
        return;
    }

    if argcount > 0 && !args.is_null() {
        nvim_lua_push_all_typvals(lstate, args, argcount, special);
    }

    let nresults = c_int::from(!ret_tv.is_null());
    if nlua_pcall(lstate, argcount, nresults) != 0 {
        nlua_error(lstate, c"E5108: Lua: %.*s".as_ptr());
        return;
    }

    if !ret_tv.is_null() {
        nlua_pop_typval(lstate, ret_tv);
    }
}

/// Evaluate a Lua string for `luaeval()`.
///
/// # Safety
///
/// `str` is a valid `String` (NvimString). `arg` and `ret_tv` are `typval_T *`.
#[unsafe(export_name = "nlua_typval_eval")]
pub unsafe extern "C" fn rs_nlua_typval_eval(
    str: NvimString,
    arg: *mut std::ffi::c_void,
    ret_tv: *mut std::ffi::c_void,
) {
    // "local _A=select(1,...) return ("
    const EVALHEADER: &[u8] = b"local _A=select(1,...) return (";
    const HEADER_LEN: usize = 31;

    let lcmd_len = rs_lua_eval_cmd_size(str.size);
    let iosize = nvim_lua_get_iosize();

    let lcmd = if rs_lcmd_fits_iosize(lcmd_len, iosize) {
        nvim_get_iobuff()
    } else {
        xmalloc(lcmd_len).cast::<c_char>()
    };

    std::ptr::copy_nonoverlapping(EVALHEADER.as_ptr(), lcmd.cast::<u8>(), HEADER_LEN);
    std::ptr::copy_nonoverlapping(
        str.data.cast::<u8>(),
        lcmd.add(HEADER_LEN).cast::<u8>(),
        str.size,
    );
    *lcmd.add(lcmd_len - 1) = b')' as c_char;

    rs_nlua_typval_exec(lcmd, lcmd_len, c"luaeval()".as_ptr(), arg, 1, true, ret_tv);

    if lcmd != nvim_get_iobuff() {
        xfree(lcmd.cast::<std::ffi::c_void>());
    }
}

/// Call a Lua function for `v:lua`.
///
/// # Safety
///
/// `str_` is a valid C string of length `len`. `args` and `ret_tv` are `typval_T *`.
#[unsafe(export_name = "nlua_typval_call")]
pub unsafe extern "C" fn rs_nlua_typval_call(
    str_: *const c_char,
    len: usize,
    args: *mut std::ffi::c_void,
    argcount: c_int,
    ret_tv: *mut std::ffi::c_void,
) {
    // "return " + str + "(...)"
    const CALLHEADER: &[u8] = b"return ";
    const CALLSUFFIX: &[u8] = b"(...)";
    const HEADER_LEN: usize = 7;
    const SUFFIX_LEN: usize = 5;

    let lcmd_len = rs_lua_call_cmd_size(len);
    let iosize = nvim_lua_get_iosize();

    let lcmd = if rs_lcmd_fits_iosize(lcmd_len, iosize) {
        nvim_get_iobuff()
    } else {
        xmalloc(lcmd_len).cast::<c_char>()
    };

    std::ptr::copy_nonoverlapping(CALLHEADER.as_ptr(), lcmd.cast::<u8>(), HEADER_LEN);
    std::ptr::copy_nonoverlapping(str_.cast::<u8>(), lcmd.add(HEADER_LEN).cast::<u8>(), len);
    std::ptr::copy_nonoverlapping(
        CALLSUFFIX.as_ptr(),
        lcmd.add(HEADER_LEN + len).cast::<u8>(),
        SUFFIX_LEN,
    );

    rs_nlua_typval_exec(
        lcmd,
        lcmd_len,
        c"v:lua".as_ptr(),
        args,
        argcount,
        false,
        ret_tv,
    );

    if lcmd != nvim_get_iobuff() {
        xfree(lcmd.cast::<std::ffi::c_void>());
    }
}

/// Call a Lua function for user expand completion.
///
/// # Safety
///
/// `xp` must be a valid `expand_T *` and `ret_tv` a valid `typval_T *`.
#[unsafe(export_name = "nlua_call_user_expand_func")]
pub unsafe extern "C" fn rs_nlua_call_user_expand_func(
    xp: *const std::ffi::c_void,
    ret_tv: *mut std::ffi::c_void,
) {
    let lstate = get_global_lstate();

    // Push ref and 3 args via C shim (avoids binding expand_T layout)
    nvim_lua_expand_push_args(lstate, xp);

    if nlua_pcall(lstate, 3, 1) != 0 {
        nlua_error(lstate, c"E5108: Lua function: %.*s".as_ptr());
        return;
    }

    nlua_pop_typval(lstate, ret_tv);
}

/// Execute Lua code from a garray, concatenated with newlines.
///
/// # Safety
///
/// `ga` must be a valid `garray_T *` and `name` a valid C string.
#[unsafe(export_name = "nlua_exec_ga")]
pub unsafe extern "C" fn rs_nlua_exec_ga(ga: *mut std::ffi::c_void, name: *mut c_char) {
    let code = ga_concat_strings(ga, c"\n".as_ptr());
    let len = std::ffi::CStr::from_ptr(code).to_bytes().len();
    rs_nlua_typval_exec(code, len, name, ptr::null_mut(), 0, false, ptr::null_mut());
    xfree(code.cast::<std::ffi::c_void>());
}

/// Call a Lua callable (`LuaRef`) given typval arguments.
///
/// # Safety
///
/// `argvars` is `typval_T *` and `rettv` is `typval_T *`.
#[unsafe(export_name = "typval_exec_lua_callable")]
pub unsafe extern "C" fn rs_typval_exec_lua_callable(
    lua_cb: c_int,
    argcount: c_int,
    argvars: *mut std::ffi::c_void,
    rettv: *mut std::ffi::c_void,
) -> c_int {
    let lstate = get_global_lstate();
    crate::refs::rs_nlua_pushref(lstate, lua_cb);

    if argcount > 0 && !argvars.is_null() {
        nvim_lua_push_all_typvals(lstate, argvars, argcount, false);
    }

    if nlua_pcall(lstate, argcount, 1) != 0 {
        nlua_error(lstate, c"Lua callback: %.*s".as_ptr());
        return FCERR_OTHER;
    }

    nlua_pop_typval(lstate, rettv);
    FCERR_NONE
}

/// Execute Lua string for `nvim_exec_lua()`.
///
/// # Safety
///
/// All pointer args must be valid.
#[unsafe(export_name = "nlua_exec")]
pub unsafe extern "C" fn rs_nlua_exec(
    str: NvimString,
    chunkname: *const c_char,
    args: Array,
    mode: c_int,
    arena: *mut Arena,
    err: *mut Error,
) -> Object {
    let lstate = get_global_lstate();
    let top = lua_gettop(lstate);

    let name = if !chunkname.is_null() && !std::ffi::CStr::from_ptr(chunkname).to_bytes().is_empty()
    {
        chunkname
    } else {
        c"<nvim>".as_ptr()
    };

    if luaL_loadbuffer(lstate, str.data, str.size, name) != 0 {
        let mut len: usize = 0;
        let errstr = lua_tolstring(lstate, -1, &raw mut len);
        #[allow(clippy::cast_possible_truncation)]
        nvim_lua_api_set_error_validation(err, len as c_int, errstr);
        return Object::nil();
    }

    for i in 0..args.size {
        // args.items is *mut Object; add(i) gives *mut Object
        push_object(lstate, args.items.add(i), 0);
    }

    #[allow(clippy::cast_possible_truncation)]
    let nargs = args.size as c_int;
    let nresults = rs_lua_mode_ret(mode);
    if nlua_pcall(lstate, nargs, nresults) != 0 {
        let mut len: usize = 0;
        let errstr = lua_tolstring(lstate, -1, &raw mut len);
        #[allow(clippy::cast_possible_truncation)]
        nvim_lua_api_set_error_exception(err, len as c_int, errstr);
        return Object::nil();
    }

    rs_nlua_call_pop_retval(lstate, mode, arena, top, err)
}

/// Pop return values from the Lua stack and convert to an Object.
///
/// # Safety
///
/// `lstate` must be valid.
#[unsafe(export_name = "nlua_call_pop_retval")]
pub unsafe extern "C" fn rs_nlua_call_pop_retval(
    lstate: *mut LuaState,
    mode: c_int,
    arena: *mut Arena,
    pretop: c_int,
    err: *mut Error,
) -> Object {
    if mode != K_RET_MULTI && lua_isnil(lstate, -1) {
        lua_pop(lstate, 1);
        return Object::nil();
    }

    let mut dummy = Error {
        err_type: K_ERROR_TYPE_NONE,
        msg: ptr::null_mut(),
    };
    let perr = if err.is_null() { &raw mut dummy } else { err };

    match mode {
        K_RET_NIL_BOOL => {
            let b = lua_toboolean(lstate, -1) != 0;
            lua_pop(lstate, 1);
            Object::boolean(b)
        }
        K_RET_LUAREF => {
            let ref_ = crate::refs::rs_nlua_ref_global(lstate, -1);
            lua_pop(lstate, 1);
            Object {
                obj_type: K_OBJECT_TYPE_LUAREF,
                data: ObjectData {
                    luaref: i64::from(ref_),
                },
            }
        }
        K_RET_OBJECT => pop_object(lstate, false, arena, perr),
        K_RET_MULTI => {
            let nres = lua_gettop(lstate) - pretop;
            let nres_usize = usize::try_from(nres).unwrap_or(0);
            let mut res = rs_arena_array(arena, nres_usize);
            for i in 0..nres {
                let obj = pop_object(lstate, false, arena, perr);
                // Fill in reverse order (as in original C)
                let idx = usize::try_from(nres - i - 1).unwrap_or(0);
                *res.items.add(idx) = obj;
                if error_is_set(perr) {
                    return Object::nil();
                }
            }
            res.size = nres_usize;
            Object::array(res)
        }
        _ => Object::nil(),
    }
}

/// Get a string representation of a Lua function reference.
///
/// # Safety
///
/// `arena` must be NULL or a valid Arena pointer.
#[must_use]
#[unsafe(export_name = "nlua_funcref_str")]
pub unsafe extern "C" fn rs_nlua_funcref_str(ref_: c_int, arena: *mut Arena) -> *mut c_char {
    let lstate = get_global_lstate();

    if lua_checkstack(lstate, 1) != 0 {
        crate::refs::rs_nlua_pushref(lstate, ref_);
        if lua_isfunction(lstate, -1) {
            // lua_getinfo(">S") pops the function from stack
            let mut src: *mut c_char = ptr::null_mut();
            let mut line: c_int = 0;
            if nvim_lua_funcref_info(lstate, &raw mut src, &raw mut line) {
                let result = nvim_callback_arena_printf_luaref_src(arena, ref_, src, line);
                xfree(src.cast::<std::ffi::c_void>());
                return result;
            }
            // On failure, funcref_info already consumed the stack value (">S" always pops)
        } else {
            lua_pop(lstate, 1);
        }
    }

    nvim_callback_arena_printf_luaref(arena, ref_)
}
