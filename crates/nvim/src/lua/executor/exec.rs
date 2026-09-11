//! Running Lua from Vimscript, and calling a `LuaRef` back.
//!
//! [`nlua_typval_eval`]/[`nlua_typval_call`] are `luaeval()` and `v:lua`;
//! [`nlua_exec`] runs a chunk; [`nlua_call_ref_ctx`] is the callback path
//! every api-registered Lua function is invoked through, and
//! `nlua_call_pop_retval` is the shared conversion of whatever it left on
//! the stack, governed by the `LuaRetMode` the caller asked for.

#![deny(unsafe_op_in_unsafe_fn)]
// Unsafe perimeter: the `lua/` row in docs/perimeter.md.
#![allow(unsafe_code)]
// The globals here keep upstream's spelling; upper-casing them is a per-module rewrite.
#![allow(non_upper_case_globals)]

use core::ffi::{CStr, c_char, c_int, c_void};
use core::ptr;
use std::ffi::CString;

use super::{
    FCERR_NONE, FCERR_OTHER, get_global_lstate, kRetLuaref, kRetMulti, kRetNilBool, kRetObject,
    nlua_error, nlua_fast_cfpcall, nlua_pcall, nlua_pushref, nlua_ref_global,
};
use crate::ex_cmds::check_secure;
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
    Arena, Array, Error, ErrorType, Expand, IOSIZE, LuaRef, LuaRetMode, Object, String_0, TypVal,
    VAR_UNKNOWN, VarNumber, kErrorTypeException, kErrorTypeValidation, lua_Integer, lua_State,
    size_t,
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

/// The chunk name `luaeval()` errors carry.
const EVALNAME: &CStr = c"luaeval()";

/// `luaeval(str, arg)`.
///
/// # Safety
/// `str` must be a live api string and `ret_tv` writable.
pub unsafe fn nlua_typval_eval(str: String_0, arg: &TypVal, ret_tv: &mut TypVal) {
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
        let arg = ::core::slice::from_ref(arg);
        nlua_typval_exec(lcmd, lcmd_len, EVALNAME.as_ptr(), arg, true, Some(ret_tv));
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
    args: &[TypVal],
    ret_tv: &mut TypVal,
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
        nlua_typval_exec(lcmd, lcmd_len, c"v:lua".as_ptr(), args, false, Some(ret_tv));
        free_chunk_buffer(scratch, lcmd);
    }
}

/// The `customlist,v:lua.…` completion callback.
///
/// # Safety
/// `xp` must carry a live `xp_luaref`, and `ret_tv` be writable.
pub unsafe fn nlua_call_user_expand_func(xp: *mut Expand, ret_tv: &mut TypVal) {
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

/// Load and run one chunk with `args` as its arguments.
///
/// `special` decides how a Vimscript value with no Lua image is pushed; a
/// `VAR_UNKNOWN` argument is `nil`, which is how a missing `luaeval()`
/// argument reaches the chunk.
///
/// # Safety
/// `lcmd`/`lcmd_len` must name a readable chunk.
pub(crate) unsafe fn nlua_typval_exec(
    lcmd: *const c_char,
    lcmd_len: size_t,
    name: *const c_char,
    args: &[TypVal],
    special: bool,
    mut ret_tv: Option<&mut TypVal>,
) {
    unsafe {
        if check_secure() {
            if let Some(ret_tv) = ret_tv {
                ret_tv.write_number(0 as VarNumber);
            }
            return;
        }
        let lstate = get_global_lstate();
        if luaL_loadbuffer(lstate, lcmd, lcmd_len, name) != 0 {
            nlua_error(lstate, gettext(c"E5107: Lua: %.*s").as_ptr());
            return;
        }
        push_typval_args(lstate, args, special);
        let argcount = args.len() as c_int;
        if nlua_pcall(lstate, argcount, c_int::from(ret_tv.is_some())) != 0 {
            nlua_error(lstate, gettext(c"E5108: Lua: %.*s").as_ptr());
            return;
        }
        if let Some(ret_tv) = ret_tv.take() {
            nlua_pop_typval(lstate, ret_tv);
        }
    }
}

/// Push the arguments, with `VAR_UNKNOWN` standing for `nil`.
///
/// # Safety
/// `lstate` must be a live Lua state.
unsafe fn push_typval_args(lstate: *mut lua_State, args: &[TypVal], special: bool) {
    unsafe {
        let flags = if special {
            kNluaPushSpecial as c_int
        } else {
            0
        };
        for arg in args {
            if arg.v_type() == VAR_UNKNOWN {
                lua_pushnil(lstate);
            } else {
                nlua_push_typval(lstate, arg, flags);
            }
        }
    }
}

/// Run a `:lua` heredoc: `lines` joined by newlines.
///
/// # Safety
/// `name` is a NUL-terminated chunk name.
pub unsafe fn nlua_exec_lines(lines: &[CString], name: *mut c_char) {
    let mut code = Vec::new();
    for (i, line) in lines.iter().enumerate() {
        if i > 0 {
            code.push(b'\n');
        }
        code.extend_from_slice(line.to_bytes());
    }
    let len = code.len();
    code.push(0);
    // SAFETY: the caller's chunk name; `code` outlives the call, and the
    // executor is handed the length as well as the terminator.
    unsafe {
        nlua_typval_exec(
            code.as_mut_ptr().cast::<c_char>(),
            len,
            name,
            &[],
            false,
            None,
        );
    };
}

/// Call a Lua function stored as a Vimscript Funcref.
///
/// # Safety
/// `lua_cb` must be a live reference.
pub unsafe fn typval_exec_lua_callable(
    lua_cb: LuaRef,
    argvars: &[TypVal],
    rettv: &mut TypVal,
) -> c_int {
    unsafe {
        let lstate = get_global_lstate();
        nlua_pushref(lstate, lua_cb);
        push_typval_args(lstate, argvars, false);
        if nlua_pcall(lstate, argvars.len() as c_int, 1) != 0 {
            nlua_error(lstate, gettext(c"Lua callback: %.*s").as_ptr());
            return FCERR_OTHER as c_int;
        }
        nlua_pop_typval(lstate, rettv);
        FCERR_NONE as c_int
    }
}

/// Run a chunk with api values as its arguments and its answer as one.
///
/// The arguments are this call's: they are pushed and then released.
///
/// # Safety
/// `chunkname` must be null or a NUL-terminated string.
pub unsafe fn nlua_exec(
    str: &String_0,
    chunkname: *const c_char,
    mut args: Array,
    mode: LuaRetMode,
    arena: *mut Arena,
) -> Result<Object, Error> {
    unsafe {
        let lstate = get_global_lstate();
        let top = lua_gettop(lstate);
        let name = if !chunkname.is_null() && *chunkname != 0 {
            chunkname
        } else {
            c"<nvim>".as_ptr()
        };
        if luaL_loadbuffer(lstate, str.data(), str.len(), name) != 0 {
            return Err(lua_error_of(kErrorTypeValidation, lstate));
        }
        let argc = args.len() as c_int;
        for item in args.iter_mut() {
            nlua_push_object(lstate, item, 0);
        }
        if nlua_pcall(lstate, argc, 1) != 0 {
            return Err(lua_error_of(kErrorTypeException, lstate));
        }
        nlua_call_pop_retval(lstate, mode, arena, top)
    }
}

/// The Lua error on top of the stack.
///
/// # Safety
/// the error value be on top of the stack.
unsafe fn lua_error_of(type_0: ErrorType, lstate: *mut lua_State) -> Error {
    unsafe {
        let mut len: size_t = 0;
        let errstr = lua_tolstring(lstate, -1, &raw mut len);
        let text = c_str_len(errstr, len);
        Error::new(type_0, format_args!("Lua: {text}"))
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
) -> Result<Object, Error> {
    unsafe { nlua_call_ref_ctx(false, ref_0, name, args, mode, arena, true) }
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
    // SAFETY: the caller's.
    unsafe { nlua_call_ref_ctx(false, ref_0, name, args, mode, arena, false) }
        .unwrap_or(Object::Nil)
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
    mut args: Array,
    mode: LuaRetMode,
    arena: *mut Arena,
    reports: bool,
) -> Result<Object, Error> {
    unsafe {
        let lstate = get_global_lstate();
        let top = lua_gettop(lstate);
        nlua_pushref(lstate, ref_0);
        let mut nargs = args.len() as c_int;
        if !name.is_null() {
            lua_pushstring(lstate, name);
            nargs += 1;
        }
        for item in args.iter_mut() {
            nlua_push_object(lstate, item, 0);
        }

        if fast {
            if nlua_fast_cfpcall(lstate, nargs, mode_ret(mode), -1) < 0 {
                if !reports {
                    return Ok(Object::Nil);
                }
                return Err(Error::exception(c"fast context failure"));
            }
        } else if nlua_pcall(lstate, nargs, mode_ret(mode)) != 0 {
            if !reports {
                // Nobody to report to: show it instead.
                nlua_error(lstate, gettext(c"Lua callback: %.*s").as_ptr());
                return Ok(Object::Nil);
            }
            return Err(lua_error_of(kErrorTypeException, lstate));
        }
        nlua_call_pop_retval(lstate, mode, arena, top)
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
) -> Result<Object, Error> {
    unsafe {
        if mode != kRetMulti && lua_type(lstate, -1) == LUA_TNIL {
            lua_pop(lstate, 1);
            return Ok(Object::Nil);
        }
        match mode {
            kRetNilBool => {
                let bool_value = lua_toboolean(lstate, -1) != 0;
                lua_pop(lstate, 1);
                Ok(Object::boolean(bool_value))
            }
            kRetLuaref => {
                let ref_0 = nlua_ref_global(lstate, -1);
                lua_pop(lstate, 1);
                Ok(Object::luaref(ref_0))
            }
            kRetObject => nlua_pop_object(lstate, false, arena),
            kRetMulti => {
                // The results come off the stack top-down, so they are stored
                // back-to-front.
                let nres = lua_gettop(lstate) - pretop;
                // The values come off the stack topmost first, so they are
                // collected and then turned back into call order.
                let mut res: Vec<Object> = Vec::with_capacity(nres as size_t);
                for _ in 0..nres {
                    res.push(nlua_pop_object(lstate, false, arena)?);
                }
                res.reverse();
                Ok(Object::array(Array::from(res)))
            }
            _ => unreachable!(),
        }
    }
}
