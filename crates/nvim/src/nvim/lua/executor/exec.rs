//! Running Lua from Vimscript, and calling a `LuaRef` back.
//!
//! `nlua_typval_eval`/`nlua_typval_call` are `v:lua` and `luaeval()`;
//! `nlua_exec` runs a chunk; `nlua_call_ref_ctx` is the callback path every
//! api-registered Lua function is invoked through, and
//! `nlua_call_pop_retval` is the shared conversion of whatever it left on
//! the stack, governed by the `LuaRetMode` the caller asked for.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;

pub unsafe extern "C-unwind" fn nlua_typval_eval(
    str: String_0,
    arg: *mut typval_T,
    ret_tv: *mut typval_T,
) {
    unsafe {
        let lcmd_len: size_t = ::core::mem::size_of::<[::core::ffi::c_char; 32]>()
            .wrapping_sub(1 as size_t)
            .wrapping_add(str.size)
            .wrapping_add(1 as size_t);
        let mut lcmd: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
        if lcmd_len < IOSIZE as size_t {
            lcmd = IObuff.ptr() as *mut ::core::ffi::c_char;
        } else {
            lcmd = xmalloc(lcmd_len) as *mut ::core::ffi::c_char;
        }
        memcpy(
            lcmd as *mut ::core::ffi::c_void,
            EVALHEADER.as_ptr() as *const ::core::ffi::c_void,
            ::core::mem::size_of::<[::core::ffi::c_char; 32]>().wrapping_sub(1 as size_t),
        );
        memcpy(
            lcmd.offset(::core::mem::size_of::<[::core::ffi::c_char; 32]>() as isize)
                .offset(-(1 as ::core::ffi::c_int as isize))
                as *mut ::core::ffi::c_void,
            str.data as *const ::core::ffi::c_void,
            str.size,
        );
        *lcmd.offset(lcmd_len.wrapping_sub(1 as size_t) as isize) = ')' as ::core::ffi::c_char;
        nlua_typval_exec(
            lcmd,
            lcmd_len,
            b"luaeval()\0".as_ptr() as *const ::core::ffi::c_char,
            arg,
            1 as ::core::ffi::c_int,
            true_0 != 0,
            ret_tv,
        );
        if lcmd != IObuff.ptr() as *mut ::core::ffi::c_char {
            xfree(lcmd as *mut ::core::ffi::c_void);
        }
    }
}

pub const EVALHEADER: [::core::ffi::c_char; 32] = unsafe {
    ::core::mem::transmute::<[u8; 32], [::core::ffi::c_char; 32]>(
        *b"local _A=select(1,...) return (\0",
    )
};

pub unsafe extern "C-unwind" fn nlua_typval_call(
    mut str: *const ::core::ffi::c_char,
    mut len: size_t,
    args: *mut typval_T,
    mut argcount: ::core::ffi::c_int,
    mut ret_tv: *mut typval_T,
) {
    unsafe {
        let lcmd_len: size_t = ::core::mem::size_of::<[::core::ffi::c_char; 8]>()
            .wrapping_sub(1 as size_t)
            .wrapping_add(len)
            .wrapping_add(::core::mem::size_of::<[::core::ffi::c_char; 6]>())
            .wrapping_sub(1 as size_t);
        let mut lcmd: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
        if lcmd_len < IOSIZE as size_t {
            lcmd = IObuff.ptr() as *mut ::core::ffi::c_char;
        } else {
            lcmd = xmalloc(lcmd_len) as *mut ::core::ffi::c_char;
        }
        memcpy(
            lcmd as *mut ::core::ffi::c_void,
            CALLHEADER.as_ptr() as *const ::core::ffi::c_void,
            ::core::mem::size_of::<[::core::ffi::c_char; 8]>().wrapping_sub(1 as size_t),
        );
        memcpy(
            lcmd.offset(::core::mem::size_of::<[::core::ffi::c_char; 8]>() as isize)
                .offset(-(1 as ::core::ffi::c_int as isize))
                as *mut ::core::ffi::c_void,
            str as *const ::core::ffi::c_void,
            len,
        );
        memcpy(
            lcmd.offset(::core::mem::size_of::<[::core::ffi::c_char; 8]>() as isize)
                .offset(-(1 as ::core::ffi::c_int as isize))
                .offset(len as isize) as *mut ::core::ffi::c_void,
            CALLSUFFIX.as_ptr() as *const ::core::ffi::c_void,
            ::core::mem::size_of::<[::core::ffi::c_char; 6]>().wrapping_sub(1 as size_t),
        );
        nlua_typval_exec(
            lcmd,
            lcmd_len,
            b"v:lua\0".as_ptr() as *const ::core::ffi::c_char,
            args,
            argcount,
            false_0 != 0,
            ret_tv,
        );
        if lcmd != IObuff.ptr() as *mut ::core::ffi::c_char {
            xfree(lcmd as *mut ::core::ffi::c_void);
        }
    }
}

pub const CALLHEADER: [::core::ffi::c_char; 8] =
    unsafe { ::core::mem::transmute::<[u8; 8], [::core::ffi::c_char; 8]>(*b"return \0") };

pub const CALLSUFFIX: [::core::ffi::c_char; 6] =
    unsafe { ::core::mem::transmute::<[u8; 6], [::core::ffi::c_char; 6]>(*b"(...)\0") };

pub unsafe extern "C-unwind" fn nlua_call_user_expand_func(
    mut xp: *mut expand_T,
    mut ret_tv: *mut typval_T,
) {
    unsafe {
        let lstate: *mut lua_State = global_lstate.get();
        nlua_pushref(lstate, (*xp).xp_luaref);
        lua_pushstring(lstate, (*xp).xp_pattern);
        lua_pushstring(lstate, (*xp).xp_line);
        lua_pushinteger(lstate, (*xp).xp_col as lua_Integer);
        if nlua_pcall(lstate, 3 as ::core::ffi::c_int, 1 as ::core::ffi::c_int) != 0 {
            nlua_error(
                lstate,
                gettext(b"E5108: Lua function: %.*s\0".as_ptr() as *const ::core::ffi::c_char),
            );
            return;
        }
        nlua_pop_typval(lstate, ret_tv);
    }
}

pub(crate) unsafe extern "C-unwind" fn nlua_typval_exec(
    mut lcmd: *const ::core::ffi::c_char,
    mut lcmd_len: size_t,
    mut name: *const ::core::ffi::c_char,
    args: *mut typval_T,
    mut argcount: ::core::ffi::c_int,
    mut special: bool,
    mut ret_tv: *mut typval_T,
) {
    unsafe {
        if check_secure() {
            if !ret_tv.is_null() {
                (*ret_tv).v_type = VAR_NUMBER;
                (*ret_tv).vval.v_number = 0 as varnumber_T;
            }
            return;
        }
        let lstate: *mut lua_State = global_lstate.get();
        if luaL_loadbuffer(lstate, lcmd, lcmd_len, name) != 0 {
            nlua_error(
                lstate,
                gettext(b"E5107: Lua: %.*s\0".as_ptr() as *const ::core::ffi::c_char),
            );
            return;
        }
        let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while i < argcount {
            if (*args.offset(i as isize)).v_type as ::core::ffi::c_uint
                == VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                lua_pushnil(lstate);
            } else {
                nlua_push_typval(
                    lstate,
                    args.offset(i as isize),
                    if special as ::core::ffi::c_int != 0 {
                        kNluaPushSpecial as ::core::ffi::c_int
                    } else {
                        0 as ::core::ffi::c_int
                    },
                );
            }
            i += 1;
        }
        if nlua_pcall(
            lstate,
            argcount,
            if !ret_tv.is_null() {
                1 as ::core::ffi::c_int
            } else {
                0 as ::core::ffi::c_int
            },
        ) != 0
        {
            nlua_error(
                lstate,
                gettext(b"E5108: Lua: %.*s\0".as_ptr() as *const ::core::ffi::c_char),
            );
            return;
        }
        if !ret_tv.is_null() {
            nlua_pop_typval(lstate, ret_tv);
        }
    }
}

pub unsafe extern "C-unwind" fn nlua_exec_ga(
    mut ga: *mut garray_T,
    mut name: *mut ::core::ffi::c_char,
) {
    unsafe {
        let mut code: *mut ::core::ffi::c_char =
            ga_concat_strings(ga, b"\n\0".as_ptr() as *const ::core::ffi::c_char);
        let mut len: size_t = strlen(code);
        nlua_typval_exec(
            code,
            len,
            name,
            ::core::ptr::null_mut::<typval_T>(),
            0 as ::core::ffi::c_int,
            false_0 != 0,
            ::core::ptr::null_mut::<typval_T>(),
        );
        xfree(code as *mut ::core::ffi::c_void);
    }
}

pub unsafe extern "C-unwind" fn typval_exec_lua_callable(
    mut lua_cb: LuaRef,
    mut argcount: ::core::ffi::c_int,
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
) -> ::core::ffi::c_int {
    unsafe {
        let mut lstate: *mut lua_State = global_lstate.get();
        nlua_pushref(lstate, lua_cb);
        let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while i < argcount {
            if (*argvars.offset(i as isize)).v_type as ::core::ffi::c_uint
                == VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                lua_pushnil(lstate);
            } else {
                nlua_push_typval(
                    lstate,
                    argvars.offset(i as isize),
                    if false {
                        kNluaPushSpecial as ::core::ffi::c_int
                    } else {
                        0 as ::core::ffi::c_int
                    },
                );
            }
            i += 1;
        }
        if nlua_pcall(lstate, argcount, 1 as ::core::ffi::c_int) != 0 {
            nlua_error(
                lstate,
                gettext(b"Lua callback: %.*s\0".as_ptr() as *const ::core::ffi::c_char),
            );
            return FCERR_OTHER as ::core::ffi::c_int;
        }
        nlua_pop_typval(lstate, rettv);
        return FCERR_NONE as ::core::ffi::c_int;
    }
}

pub unsafe extern "C-unwind" fn nlua_exec(
    str: String_0,
    mut chunkname: *const ::core::ffi::c_char,
    args: Array,
    mut mode: LuaRetMode,
    mut arena: *mut Arena,
    mut err: *mut Error,
) -> Object {
    unsafe {
        let lstate: *mut lua_State = global_lstate.get();
        let mut top: ::core::ffi::c_int = lua_gettop(lstate);
        let mut name: *const ::core::ffi::c_char = if !chunkname.is_null()
            && *chunkname.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int != 0
        {
            chunkname
        } else {
            b"<nvim>\0".as_ptr() as *const ::core::ffi::c_char
        };
        if luaL_loadbuffer(lstate, str.data, str.size, name) != 0 {
            let mut len: size_t = 0;
            let mut errstr: *const ::core::ffi::c_char =
                lua_tolstring(lstate, -1 as ::core::ffi::c_int, &raw mut len);
            api_set_error(
                err,
                kErrorTypeValidation,
                b"Lua: %.*s\0".as_ptr() as *const ::core::ffi::c_char,
                len as ::core::ffi::c_int,
                errstr,
            );
            return object {
                type_0: kObjectTypeNil,
                data: C2Rust_Unnamed_11 { boolean: false },
            };
        }
        let mut i: size_t = 0 as size_t;
        while i < args.size {
            nlua_push_Object(
                lstate,
                args.items.offset(i as isize),
                0 as ::core::ffi::c_int,
            );
            i = i.wrapping_add(1);
        }
        if nlua_pcall(
            lstate,
            args.size as ::core::ffi::c_int,
            1 as ::core::ffi::c_int,
        ) != 0
        {
            let mut len_0: size_t = 0;
            let mut errstr_0: *const ::core::ffi::c_char =
                lua_tolstring(lstate, -1 as ::core::ffi::c_int, &raw mut len_0);
            api_set_error(
                err,
                kErrorTypeException,
                b"Lua: %.*s\0".as_ptr() as *const ::core::ffi::c_char,
                len_0 as ::core::ffi::c_int,
                errstr_0,
            );
            return object {
                type_0: kObjectTypeNil,
                data: C2Rust_Unnamed_11 { boolean: false },
            };
        }
        return nlua_call_pop_retval(lstate, mode, arena, top, err);
    }
}

pub unsafe extern "C-unwind" fn nlua_call_ref(
    mut ref_0: LuaRef,
    mut name: *const ::core::ffi::c_char,
    mut args: Array,
    mut mode: LuaRetMode,
    mut arena: *mut Arena,
    mut err: *mut Error,
) -> Object {
    unsafe {
        return nlua_call_ref_ctx(false_0 != 0, ref_0, name, args, mode, arena, err);
    }
}

unsafe extern "C-unwind" fn mode_ret(mut mode: LuaRetMode) -> ::core::ffi::c_int {
    return if mode as ::core::ffi::c_uint == kRetMulti as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        LUA_MULTRET
    } else {
        1 as ::core::ffi::c_int
    };
}

pub unsafe extern "C-unwind" fn nlua_call_ref_ctx(
    mut fast: bool,
    mut ref_0: LuaRef,
    mut name: *const ::core::ffi::c_char,
    mut args: Array,
    mut mode: LuaRetMode,
    mut arena: *mut Arena,
    mut err: *mut Error,
) -> Object {
    unsafe {
        let lstate: *mut lua_State = global_lstate.get();
        let mut top: ::core::ffi::c_int = lua_gettop(lstate);
        nlua_pushref(lstate, ref_0);
        let mut nargs: ::core::ffi::c_int = args.size as ::core::ffi::c_int;
        if !name.is_null() {
            lua_pushstring(lstate, name);
            nargs += 1;
        }
        let mut i: size_t = 0 as size_t;
        while i < args.size {
            nlua_push_Object(
                lstate,
                args.items.offset(i as isize),
                0 as ::core::ffi::c_int,
            );
            i = i.wrapping_add(1);
        }
        if fast {
            if nlua_fast_cfpcall(lstate, nargs, mode_ret(mode), -1 as ::core::ffi::c_int)
                < 0 as ::core::ffi::c_int
            {
                api_set_error(
                    err,
                    kErrorTypeException,
                    b"fast context failure\0".as_ptr() as *const ::core::ffi::c_char,
                );
                return object {
                    type_0: kObjectTypeNil,
                    data: C2Rust_Unnamed_11 { boolean: false },
                };
            }
        } else if nlua_pcall(lstate, nargs, mode_ret(mode)) != 0 {
            if !err.is_null() {
                let mut len: size_t = 0;
                let mut errstr: *const ::core::ffi::c_char =
                    lua_tolstring(lstate, -1 as ::core::ffi::c_int, &raw mut len);
                api_set_error(
                    err,
                    kErrorTypeException,
                    b"Lua: %.*s\0".as_ptr() as *const ::core::ffi::c_char,
                    len as ::core::ffi::c_int,
                    errstr,
                );
            } else {
                nlua_error(
                    lstate,
                    gettext(b"Lua callback: %.*s\0".as_ptr() as *const ::core::ffi::c_char),
                );
            }
            return object {
                type_0: kObjectTypeNil,
                data: C2Rust_Unnamed_11 { boolean: false },
            };
        }
        return nlua_call_pop_retval(lstate, mode, arena, top, err);
    }
}

unsafe extern "C-unwind" fn nlua_call_pop_retval(
    mut lstate: *mut lua_State,
    mut mode: LuaRetMode,
    mut arena: *mut Arena,
    mut pretop: ::core::ffi::c_int,
    mut err: *mut Error,
) -> Object {
    unsafe {
        if mode as ::core::ffi::c_uint != kRetMulti as ::core::ffi::c_int as ::core::ffi::c_uint
            && lua_type(lstate, -1 as ::core::ffi::c_int) == LUA_TNIL
        {
            lua_settop(lstate, -1 as ::core::ffi::c_int - 1 as ::core::ffi::c_int);
            return object {
                type_0: kObjectTypeNil,
                data: C2Rust_Unnamed_11 { boolean: false },
            };
        }
        let mut dummy: Error = Error {
            type_0: kErrorTypeNone,
            msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        };
        let mut perr: *mut Error = if !err.is_null() { err } else { &raw mut dummy };
        match mode as ::core::ffi::c_uint {
            1 => {
                let mut bool_value: bool = lua_toboolean(lstate, -1 as ::core::ffi::c_int) != 0;
                lua_settop(lstate, -1 as ::core::ffi::c_int - 1 as ::core::ffi::c_int);
                return object {
                    type_0: kObjectTypeBoolean,
                    data: C2Rust_Unnamed_11 {
                        boolean: bool_value,
                    },
                };
            }
            2 => {
                let mut ref_0: LuaRef = nlua_ref_global(lstate, -1 as ::core::ffi::c_int);
                lua_settop(lstate, -1 as ::core::ffi::c_int - 1 as ::core::ffi::c_int);
                return object {
                    type_0: kObjectTypeLuaRef,
                    data: C2Rust_Unnamed_11 { luaref: ref_0 },
                };
            }
            0 => return nlua_pop_Object(lstate, false_0 != 0, arena, perr),
            3 => {
                let mut nres: ::core::ffi::c_int = lua_gettop(lstate) - pretop;
                let mut res: Array = arena_array(arena, nres as size_t);
                let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                while i < nres {
                    *res.items
                        .offset((nres - i - 1 as ::core::ffi::c_int) as isize) =
                        nlua_pop_Object(lstate, false_0 != 0, arena, perr);
                    if (*perr).type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int
                    {
                        return object {
                            type_0: kObjectTypeNil,
                            data: C2Rust_Unnamed_11 { boolean: false },
                        };
                    }
                    i += 1;
                }
                res.size = nres as size_t;
                return object {
                    type_0: kObjectTypeArray,
                    data: C2Rust_Unnamed_11 { array: res },
                };
            }
            _ => {}
        }
        unreachable!();
    }
}
