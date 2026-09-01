//! Running Lua from Vimscript, and calling a `LuaRef` back.
//!
//! [`nlua_typval_eval`]/[`nlua_typval_call`] are `luaeval()` and `v:lua`;
//! [`nlua_exec`] runs a chunk; [`nlua_call_ref_ctx`] is the callback path
//! every api-registered Lua function is invoked through, and
//! `nlua_call_pop_retval` is the shared conversion of whatever it left on
//! the stack, governed by the `LuaRetMode` the caller asked for.

#![deny(unsafe_op_in_unsafe_fn)]

use crate::cstr;
use core::ffi::{CStr, c_char, c_int, c_void};
use core::ptr;

use super::{
    FCERR_NONE, FCERR_OTHER, get_global_lstate, kRetLuaref, kRetMulti, kRetNilBool, kRetObject,
    nlua_error, nlua_fast_cfpcall, nlua_pcall, nlua_pushref, nlua_ref_global,
};
use crate::api::private::helpers::arena_array;
use crate::ex_cmds::check_secure;
use crate::garray::ga_concat_strings;
use crate::lua::converter::{
    kNluaPushSpecial, nlua_pop_object, nlua_pop_typval, nlua_push_object, nlua_push_typval,
};
use crate::lua::ffi::{
    LUA_MULTRET, LUA_TNIL, lua_gettop, lua_pop, lua_pushinteger, lua_pushnil, lua_pushstring,
    lua_toboolean, lua_tolstring, lua_type, luaL_loadbuffer,
};
use crate::memory::{xfree, xmalloc};
use crate::message_fmt::c_str_len;
use crate::os::cshim::gettext;
use crate::types::{
    Arena, Array, Error, ErrorType, IOSIZE, LuaRef, LuaRetMode, Object, String_0, VAR_NUMBER,
    VAR_UNKNOWN, expand_T, garray_T, kErrorTypeException, kErrorTypeValidation, lua_Integer,
    lua_State, size_t, typval_T, varnumber_T,
};

/// `luaeval("expr")` becomes this chunk with the expression appended and a
/// `)` closed after it, so the expression is evaluated with `_A` bound to
/// the second argument.
const EVALHEADER: &CStr = c"local _A=select(1,...) return (";
/// `v:lua.f(…)` becomes this, the name, and [`CALLSUFFIX`].
const CALLHEADER: &CStr = c"return ";
const CALLSUFFIX: &CStr = c"(...)";

/// Where the assembled chunk goes: `scratch`, the caller's own `IOSIZE`
/// buffer, when it fits; a fresh allocation otherwise. Upstream shares
/// `IObuff` for it, which anything the chunk runs may overwrite.
///
/// # Safety
/// `scratch` must be `IOSIZE` writable bytes and `len` the length the
/// caller is about to write.
unsafe fn chunk_buffer(scratch: *mut c_char, len: size_t) -> *mut c_char {
    if len < IOSIZE as size_t {
        return scratch;
    }
    // SAFETY: a plain allocation.
    unsafe { xmalloc(len).cast::<c_char>() }
}

/// Free [`chunk_buffer`]'s answer, unless it was the caller's `scratch`.
///
/// # Safety
/// `buf` must be [`chunk_buffer`]'s answer for `scratch`.
unsafe fn free_chunk_buffer(scratch: *const c_char, buf: *mut c_char) {
    if !ptr::eq(buf.cast_const(), scratch) {
        // SAFETY: the caller's contract.
        unsafe { xfree(buf.cast::<c_void>()) };
    }
}

/// `luaeval(str, arg)`.
///
/// # Safety
/// `str` must be a live api string and `ret_tv` writable.
pub unsafe fn nlua_typval_eval(str: String_0, arg: *mut typval_T, ret_tv: *mut typval_T) {
    let mut chunk = [0 as c_char; IOSIZE as usize];
    let scratch = chunk.as_mut_ptr();
    unsafe {
        let head = EVALHEADER.count_bytes();
        let lcmd_len = head + str.len() + 1;
        let lcmd = chunk_buffer(scratch, lcmd_len);
        lcmd.cast::<u8>()
            .copy_from_nonoverlapping(EVALHEADER.as_ptr().cast(), head);
        lcmd.add(head)
            .cast::<u8>()
            .copy_from_nonoverlapping(str.data().cast(), str.len());
        *lcmd.add(lcmd_len - 1) = b')' as c_char;
        nlua_typval_exec(lcmd, lcmd_len, c"luaeval()".as_ptr(), arg, 1, true, ret_tv);
        free_chunk_buffer(scratch, lcmd);
    }
}

/// `v:lua.name(...)`.
///
/// # Safety
/// `str`/`len` must name a Lua expression and `ret_tv` be writable.
pub unsafe fn nlua_typval_call(
    str: *const c_char,
    len: size_t,
    args: *mut typval_T,
    argcount: c_int,
    ret_tv: *mut typval_T,
) {
    let mut chunk = [0 as c_char; IOSIZE as usize];
    let scratch = chunk.as_mut_ptr();
    unsafe {
        let head = CALLHEADER.count_bytes();
        let tail = CALLSUFFIX.count_bytes();
        let lcmd_len = head + len + tail;
        let lcmd = chunk_buffer(scratch, lcmd_len);
        lcmd.cast::<u8>()
            .copy_from_nonoverlapping(CALLHEADER.as_ptr().cast(), head);
        lcmd.add(head)
            .cast::<u8>()
            .copy_from_nonoverlapping(str.cast(), len);
        (lcmd.add(head + len))
            .cast::<u8>()
            .copy_from_nonoverlapping(CALLSUFFIX.as_ptr().cast(), tail);
        nlua_typval_exec(
            lcmd,
            lcmd_len,
            c"v:lua".as_ptr(),
            args,
            argcount,
            false,
            ret_tv,
        );
        free_chunk_buffer(scratch, lcmd);
    }
}

/// The `customlist,v:lua.…` completion callback.
///
/// # Safety
/// `xp` must carry a live `xp_luaref`, and `ret_tv` be writable.
pub unsafe fn nlua_call_user_expand_func(xp: *mut expand_T, ret_tv: *mut typval_T) {
    unsafe {
        let lstate = get_global_lstate();
        nlua_pushref(lstate, (*xp).xp_luaref);
        lua_pushstring(lstate, (*xp).xp_pattern);
        lua_pushstring(lstate, (*xp).xp_line);
        lua_pushinteger(lstate, (*xp).xp_col as lua_Integer);
        if nlua_pcall(lstate, 3, 1) != 0 {
            nlua_error(lstate, gettext(c"E5108: Lua function: %.*s").as_ptr());
            return;
        }
        nlua_pop_typval(lstate, ret_tv);
    }
}

/// Load and run one chunk with `argcount` typvals as its arguments.
///
/// `special` decides how a Vimscript value with no Lua image is pushed; a
/// `VAR_UNKNOWN` argument is `nil`, which is how a missing `luaeval()`
/// argument reaches the chunk.
///
/// # Safety
/// `lcmd`/`lcmd_len` must name a readable chunk, and `ret_tv` be writable or
/// null.
pub(crate) unsafe fn nlua_typval_exec(
    lcmd: *const c_char,
    lcmd_len: size_t,
    name: *const c_char,
    args: *mut typval_T,
    argcount: c_int,
    special: bool,
    ret_tv: *mut typval_T,
) {
    unsafe {
        if check_secure() {
            if !ret_tv.is_null() {
                (*ret_tv).v_type = VAR_NUMBER;
                (*ret_tv).vval.v_number = 0 as varnumber_T;
            }
            return;
        }
        let lstate = get_global_lstate();
        if luaL_loadbuffer(lstate, lcmd, lcmd_len, name) != 0 {
            nlua_error(lstate, gettext(c"E5107: Lua: %.*s").as_ptr());
            return;
        }
        push_typval_args(lstate, args, argcount, special);
        if nlua_pcall(lstate, argcount, if ret_tv.is_null() { 0 } else { 1 }) != 0 {
            nlua_error(lstate, gettext(c"E5108: Lua: %.*s").as_ptr());
            return;
        }
        if !ret_tv.is_null() {
            nlua_pop_typval(lstate, ret_tv);
        }
    }
}

/// Push `argcount` typvals, with `VAR_UNKNOWN` standing for `nil`.
///
/// # Safety
/// `args` must point at `argcount` live typvals.
unsafe fn push_typval_args(
    lstate: *mut lua_State,
    args: *mut typval_T,
    argcount: c_int,
    special: bool,
) {
    unsafe {
        let flags = if special {
            kNluaPushSpecial as c_int
        } else {
            0
        };
        for i in 0..argcount {
            let arg = args.offset(i as isize);
            if (*arg).v_type == VAR_UNKNOWN {
                lua_pushnil(lstate);
            } else {
                nlua_push_typval(lstate, arg, flags);
            }
        }
    }
}

/// Run a `:lua` heredoc: the garray's lines joined by newlines.
///
/// # Safety
/// `ga` must be a live garray of strings.
pub unsafe fn nlua_exec_ga(ga: *mut garray_T, name: *mut c_char) {
    unsafe {
        let code = ga_concat_strings(ga, c"\n".as_ptr());
        let len = cstr::bytes_at(code).len();
        nlua_typval_exec(
            code,
            len,
            name,
            ptr::null_mut::<typval_T>(),
            0,
            false,
            ptr::null_mut::<typval_T>(),
        );
        xfree(code.cast::<c_void>());
    }
}

/// Call a Lua function stored as a Vimscript Funcref.
///
/// # Safety
/// `lua_cb` must be a live reference, `argvars` `argcount` live typvals, and
/// `rettv` writable.
pub unsafe fn typval_exec_lua_callable(
    lua_cb: LuaRef,
    argcount: c_int,
    argvars: *mut typval_T,
    rettv: *mut typval_T,
) -> c_int {
    unsafe {
        let lstate = get_global_lstate();
        nlua_pushref(lstate, lua_cb);
        push_typval_args(lstate, argvars, argcount, false);
        if nlua_pcall(lstate, argcount, 1) != 0 {
            nlua_error(lstate, gettext(c"Lua callback: %.*s").as_ptr());
            return FCERR_OTHER as c_int;
        }
        nlua_pop_typval(lstate, rettv);
        FCERR_NONE as c_int
    }
}

/// Run a chunk with api values as its arguments and its answer as one.
///
/// # Safety
/// `str` must be a live api string and `err` the caller's error slot.
pub unsafe fn nlua_exec(
    str: String_0,
    chunkname: *const c_char,
    args: Array,
    mode: LuaRetMode,
    arena: *mut Arena,
    err: &mut Error,
) -> Object {
    unsafe {
        let lstate = get_global_lstate();
        let top = lua_gettop(lstate);
        let name = if !chunkname.is_null() && *chunkname != 0 {
            chunkname
        } else {
            c"<nvim>".as_ptr()
        };
        if luaL_loadbuffer(lstate, str.data(), str.len(), name) != 0 {
            set_lua_error(err, kErrorTypeValidation, lstate);
            return Object::Nil;
        }
        for i in 0..args.size {
            nlua_push_object(lstate, args.items.add(i), 0);
        }
        if nlua_pcall(lstate, args.size as c_int, 1) != 0 {
            set_lua_error(err, kErrorTypeException, lstate);
            return Object::Nil;
        }
        nlua_call_pop_retval(lstate, mode, arena, top, Some(err))
    }
}

/// Report the Lua error on top of the stack through `err`.
///
/// # Safety
/// the error value be on top of the stack.
unsafe fn set_lua_error(err: &mut Error, type_0: ErrorType, lstate: *mut lua_State) {
    unsafe {
        let mut len: size_t = 0;
        let errstr = lua_tolstring(lstate, -1, &raw mut len);
        let text = c_str_len(errstr, len);
        *err = Error::new(type_0, format_args!("Lua: {text}"));
    }
}

/// [`nlua_call_ref_ctx`] outside a fast context.
///
/// # Safety
/// As [`nlua_call_ref_ctx`].
pub unsafe fn nlua_call_ref(
    ref_0: LuaRef,
    name: *const c_char,
    args: Array,
    mode: LuaRetMode,
    arena: *mut Arena,
    err: &mut Error,
) -> Object {
    unsafe { nlua_call_ref_ctx(false, ref_0, name, args, mode, arena, Some(err)) }
}

/// [`nlua_call_ref`] for a caller with nowhere to report to: a failing
/// callback shows its error rather than answering with one.
///
/// # Safety
/// As [`nlua_call_ref`].
pub unsafe fn nlua_call_ref_quiet(
    ref_0: LuaRef,
    name: *const c_char,
    args: Array,
    mode: LuaRetMode,
    arena: *mut Arena,
) -> Object {
    unsafe { nlua_call_ref_ctx(false, ref_0, name, args, mode, arena, None) }
}

/// How many results `mode` wants off the call.
fn mode_ret(mode: LuaRetMode) -> c_int {
    if mode == kRetMulti { LUA_MULTRET } else { 1 }
}

/// Call the function `ref_0` refers to.
///
/// `name`, when given, is pushed as the *first* argument — that is how one
/// registered callback serves several event names.  `fast` runs the call
/// through the luv path instead, which is the only one allowed inside a fast
/// callback and which reports rather than returns a failure.
///
/// # Safety
/// `ref_0` must be a live reference.
pub unsafe fn nlua_call_ref_ctx(
    fast: bool,
    ref_0: LuaRef,
    name: *const c_char,
    args: Array,
    mode: LuaRetMode,
    arena: *mut Arena,
    err: Option<&mut Error>,
) -> Object {
    unsafe {
        let lstate = get_global_lstate();
        let top = lua_gettop(lstate);
        nlua_pushref(lstate, ref_0);
        let mut nargs = args.size as c_int;
        if !name.is_null() {
            lua_pushstring(lstate, name);
            nargs += 1;
        }
        for i in 0..args.size {
            nlua_push_object(lstate, args.items.add(i), 0);
        }

        let mut err = err;
        if fast {
            if nlua_fast_cfpcall(lstate, nargs, mode_ret(mode), -1) < 0 {
                if let Some(err) = err.as_deref_mut() {
                    *err = Error::exception(c"fast context failure");
                }
                return Object::Nil;
            }
        } else if nlua_pcall(lstate, nargs, mode_ret(mode)) != 0 {
            match err.as_deref_mut() {
                // Nobody to report to: show it instead.
                None => nlua_error(lstate, gettext(c"Lua callback: %.*s").as_ptr()),
                Some(err) => set_lua_error(err, kErrorTypeException, lstate),
            }
            return Object::Nil;
        }
        nlua_call_pop_retval(lstate, mode, arena, top, err)
    }
}

/// Convert whatever the call left on the stack, and pop it.
///
/// A `nil` answer is nil whatever the mode asked for — except `kRetMulti`,
/// where it is one element of the list.
///
/// # Safety
/// `lstate` must hold the call's results down to `pretop`.
unsafe fn nlua_call_pop_retval(
    lstate: *mut lua_State,
    mode: LuaRetMode,
    arena: *mut Arena,
    pretop: c_int,
    err: Option<&mut Error>,
) -> Object {
    unsafe {
        if mode != kRetMulti && lua_type(lstate, -1) == LUA_TNIL {
            lua_pop(lstate, 1);
            return Object::Nil;
        }
        let mut dummy = Error::none();
        let perr: &mut Error = err.unwrap_or(&mut dummy);
        match mode {
            kRetNilBool => {
                let bool_value = lua_toboolean(lstate, -1) != 0;
                lua_pop(lstate, 1);
                Object::boolean(bool_value)
            }
            kRetLuaref => {
                let ref_0 = nlua_ref_global(lstate, -1);
                lua_pop(lstate, 1);
                Object::luaref(ref_0)
            }
            kRetObject => nlua_pop_object(lstate, false, arena, perr),
            kRetMulti => {
                // The results come off the stack top-down, so they are stored
                // back-to-front.
                let nres = lua_gettop(lstate) - pretop;
                let mut res: Array = arena_array(arena, nres as size_t);
                for i in 0..nres {
                    *res.items.offset((nres - i - 1) as isize) =
                        nlua_pop_object(lstate, false, arena, perr);
                    if (*perr).is_set() {
                        return Object::Nil;
                    }
                }
                res.size = nres as size_t;
                Object::array(res)
            }
            _ => unreachable!(),
        }
    }
}
