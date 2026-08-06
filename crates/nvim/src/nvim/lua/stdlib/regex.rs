//! `vim.regex()`: a Vimscript regexp as a Lua userdatum.
//!
//! `nlua_regex` compiles the pattern and pushes a userdatum whose metatable
//! is [`regex_meta`]; `regex_match_str` and `regex_match_line` are its two
//! `match_*` methods, both reaching the same `regex_match` core, and
//! `regex_gc` frees the compiled program.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;

unsafe extern "C-unwind" fn regex_match(
    mut lstate: *mut lua_State,
    mut prog: *mut *mut regprog_T,
    mut str: *mut ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    unsafe {
        let mut rm: regmatch_T = regmatch_T {
            regprog: ::core::ptr::null_mut::<regprog_T>(),
            startp: [::core::ptr::null_mut::<::core::ffi::c_char>(); 10],
            endp: [::core::ptr::null_mut::<::core::ffi::c_char>(); 10],
            rm_matchcol: 0,
            rm_ic: false,
        };
        rm.regprog = *prog;
        rm.rm_ic = false_0 != 0;
        let mut match_0: bool = vim_regexec(&raw mut rm, str, 0 as colnr_T);
        *prog = rm.regprog;
        if match_0 {
            lua_pushinteger(
                lstate,
                rm.startp[0 as ::core::ffi::c_int as usize].offset_from(str),
            );
            lua_pushinteger(
                lstate,
                rm.endp[0 as ::core::ffi::c_int as usize].offset_from(str),
            );
            return 2 as ::core::ffi::c_int;
        }
        return 0 as ::core::ffi::c_int;
    }
}

unsafe extern "C-unwind" fn regex_match_str(mut lstate: *mut lua_State) -> ::core::ffi::c_int {
    unsafe {
        let mut prog: *mut *mut regprog_T = regex_check(lstate);
        let mut str: *const ::core::ffi::c_char = luaL_checklstring(
            lstate,
            2 as ::core::ffi::c_int,
            ::core::ptr::null_mut::<size_t>(),
        );
        let mut nret: ::core::ffi::c_int =
            regex_match(lstate, prog, str as *mut ::core::ffi::c_char);
        if (*prog).is_null() {
            return luaL_error(
                lstate,
                b"regex: internal error\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        return nret;
    }
}

unsafe extern "C-unwind" fn regex_match_line(mut lstate: *mut lua_State) -> ::core::ffi::c_int {
    unsafe {
        let mut prog: *mut *mut regprog_T = regex_check(lstate);
        let mut narg: ::core::ffi::c_int = lua_gettop(lstate);
        if narg < 3 as ::core::ffi::c_int {
            return luaL_error(
                lstate,
                b"not enough args\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        let mut bufnr: handle_T = luaL_checkinteger(lstate, 2 as ::core::ffi::c_int) as handle_T;
        let mut rownr: linenr_T = luaL_checkinteger(lstate, 3 as ::core::ffi::c_int) as linenr_T;
        let mut start: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        let mut end: ::core::ffi::c_int = -1 as ::core::ffi::c_int;
        if narg >= 4 as ::core::ffi::c_int {
            start = luaL_checkinteger(lstate, 4 as ::core::ffi::c_int) as ::core::ffi::c_int;
        }
        if narg >= 5 as ::core::ffi::c_int {
            end = luaL_checkinteger(lstate, 5 as ::core::ffi::c_int) as ::core::ffi::c_int;
            if end < 0 as ::core::ffi::c_int {
                return luaL_error(
                    lstate,
                    b"invalid end\0".as_ptr() as *const ::core::ffi::c_char,
                );
            }
        }
        let mut buf: *mut buf_T = (if bufnr != 0 {
            map_get_int_ptr_t(buffer_handles.ptr(), bufnr as ::core::ffi::c_int)
        } else {
            curbuf.get() as *mut ::core::ffi::c_void
        }) as *mut buf_T;
        if buf.is_null() || (*buf).b_ml.ml_mfp.is_null() {
            return luaL_error(
                lstate,
                b"invalid buffer\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        if rownr >= (*buf).b_ml.ml_line_count {
            return luaL_error(
                lstate,
                b"invalid row\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        let mut line: *mut ::core::ffi::c_char = ml_get_buf(buf, rownr + 1 as linenr_T);
        let mut len: colnr_T = ml_get_buf_len(buf, rownr + 1 as linenr_T);
        if start < 0 as ::core::ffi::c_int || start > len {
            return luaL_error(
                lstate,
                b"invalid start\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        let mut save: ::core::ffi::c_char = NUL as ::core::ffi::c_char;
        if end >= 0 as ::core::ffi::c_int {
            if end > len || end < start {
                return luaL_error(
                    lstate,
                    b"invalid end\0".as_ptr() as *const ::core::ffi::c_char,
                );
            }
            save = *line.offset(end as isize);
            *line.offset(end as isize) = NUL as ::core::ffi::c_char;
        }
        let mut nret: ::core::ffi::c_int = regex_match(lstate, prog, line.offset(start as isize));
        if end >= 0 as ::core::ffi::c_int {
            *line.offset(end as isize) = save;
        }
        if (*prog).is_null() {
            return luaL_error(
                lstate,
                b"regex: internal error\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        return nret;
    }
}

unsafe extern "C-unwind" fn regex_check(mut L: *mut lua_State) -> *mut *mut regprog_T {
    unsafe {
        return luaL_checkudata(
            L,
            1 as ::core::ffi::c_int,
            b"nvim_regex\0".as_ptr() as *const ::core::ffi::c_char,
        ) as *mut *mut regprog_T;
    }
}

unsafe extern "C-unwind" fn regex_gc(mut lstate: *mut lua_State) -> ::core::ffi::c_int {
    unsafe {
        let mut prog: *mut *mut regprog_T = regex_check(lstate);
        vim_regfree(*prog);
        return 0 as ::core::ffi::c_int;
    }
}

unsafe extern "C-unwind" fn regex_tostring(mut lstate: *mut lua_State) -> ::core::ffi::c_int {
    unsafe {
        lua_pushstring(lstate, b"<regex>\0".as_ptr() as *const ::core::ffi::c_char);
        return 1 as ::core::ffi::c_int;
    }
}

pub(crate) static regex_meta: GlobalCell<[luaL_Reg; 5]> = GlobalCell::new([
    luaL_Reg {
        name: b"__gc\0".as_ptr() as *const ::core::ffi::c_char,
        func: Some(regex_gc as unsafe extern "C-unwind" fn(*mut lua_State) -> ::core::ffi::c_int),
    },
    luaL_Reg {
        name: b"__tostring\0".as_ptr() as *const ::core::ffi::c_char,
        func: Some(
            regex_tostring as unsafe extern "C-unwind" fn(*mut lua_State) -> ::core::ffi::c_int,
        ),
    },
    luaL_Reg {
        name: b"match_str\0".as_ptr() as *const ::core::ffi::c_char,
        func: Some(
            regex_match_str as unsafe extern "C-unwind" fn(*mut lua_State) -> ::core::ffi::c_int,
        ),
    },
    luaL_Reg {
        name: b"match_line\0".as_ptr() as *const ::core::ffi::c_char,
        func: Some(
            regex_match_line as unsafe extern "C-unwind" fn(*mut lua_State) -> ::core::ffi::c_int,
        ),
    },
    luaL_Reg {
        name: ::core::ptr::null::<::core::ffi::c_char>(),
        func: None,
    },
]);

pub unsafe extern "C-unwind" fn nlua_regex(mut lstate: *mut lua_State) -> ::core::ffi::c_int {
    unsafe {
        let mut err: Error = Error {
            type_0: kErrorTypeNone,
            msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        };
        let mut text: *const ::core::ffi::c_char = luaL_checklstring(
            lstate,
            1 as ::core::ffi::c_int,
            ::core::ptr::null_mut::<size_t>(),
        );
        let mut prog: *mut regprog_T = ::core::ptr::null_mut::<regprog_T>();
        let mut tstate: TryState = TryState {
            current_exception: ::core::ptr::null_mut::<except_T>(),
            private_msg_list: ::core::ptr::null_mut::<msglist_T>(),
            msg_list: ::core::ptr::null::<*const msglist_T>(),
            got_int: 0,
            did_throw: false,
            need_rethrow: 0,
            did_emsg: 0,
        };
        try_enter(&raw mut tstate);
        prog = vim_regcomp(
            text,
            8 as ::core::ffi::c_int | 1 as ::core::ffi::c_int | 4 as ::core::ffi::c_int,
        );
        try_leave(&raw mut tstate, &raw mut err);
        if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
            nlua_push_errstr(
                lstate,
                b"couldn't parse regex: %s\0".as_ptr() as *const ::core::ffi::c_char,
                err.msg,
            );
            api_clear_error(&raw mut err);
            return lua_error(lstate);
        } else if prog.is_null() {
            nlua_push_errstr(
                lstate,
                b"couldn't parse regex\0".as_ptr() as *const ::core::ffi::c_char,
            );
            return lua_error(lstate);
        }
        let mut p: *mut *mut regprog_T =
            lua_newuserdata(lstate, ::core::mem::size_of::<*mut regprog_T>())
                as *mut *mut regprog_T;
        *p = prog;
        lua_getfield(
            lstate,
            LUA_REGISTRYINDEX,
            b"nvim_regex\0".as_ptr() as *const ::core::ffi::c_char,
        );
        lua_setmetatable(lstate, -2 as ::core::ffi::c_int);
        return 1 as ::core::ffi::c_int;
    }
}
