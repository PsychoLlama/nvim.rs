//! `:lua`, `:luado`, `:luafile` and sourcing a file.
//!
//! `ex_luado` is the one with a shape of its own: it compiles the body once
//! into a function of `(line, linenr)` and runs it over the range, replacing
//! or deleting each line by what the function returns.  `nlua_exec_file`
//! is what `:luafile` and the runtime loader both reach.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;

pub unsafe fn ex_lua(eap: *mut exarg_T) {
    unsafe {
        if *(*eap).arg as ::core::ffi::c_int == NUL {
            if (*eap).addr_count > 0 as ::core::ffi::c_int {
                cmd_source_buffer(eap, true_0 != 0);
            } else {
                emsg(gettext(&raw const e_argreq as *const ::core::ffi::c_char));
            }
            return;
        }
        let mut len: size_t = 0;
        let mut code: *mut ::core::ffi::c_char = script_get(eap, &raw mut len);
        if (*eap).skip != 0 || code.is_null() {
            xfree(code as *mut ::core::ffi::c_void);
            return;
        }
        if (*eap).cmdidx as ::core::ffi::c_int == CMD_equal as ::core::ffi::c_int
            || *code.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                == '=' as ::core::ffi::c_int
        {
            let mut off: size_t =
                (if (*eap).cmdidx as ::core::ffi::c_int == CMD_equal as ::core::ffi::c_int {
                    0 as ::core::ffi::c_int
                } else {
                    1 as ::core::ffi::c_int
                }) as size_t;
            len = (len as ::core::ffi::c_ulong).wrapping_add(
                ::core::mem::size_of::<[::core::ffi::c_char; 19]>()
                    .wrapping_sub(1 as usize)
                    .wrapping_sub(off as usize) as ::core::ffi::c_ulong,
            ) as size_t;
            let mut code_buf: *mut ::core::ffi::c_char = xmallocz(len) as *mut ::core::ffi::c_char;
            vim_snprintf(
                code_buf,
                len.wrapping_add(1 as size_t),
                b"vim._print(true, %s)\0".as_ptr() as *const ::core::ffi::c_char,
                code.offset(off as isize),
            );
            xfree(code as *mut ::core::ffi::c_void);
            code = code_buf;
        }
        nlua_typval_exec(
            code,
            len,
            b":lua\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::ptr::null_mut::<typval_T>(),
            0 as ::core::ffi::c_int,
            false_0 != 0,
            ::core::ptr::null_mut::<typval_T>(),
        );
        xfree(code as *mut ::core::ffi::c_void);
    }
}

pub unsafe fn ex_luado(eap: *mut exarg_T) {
    unsafe {
        if u_save((*eap).line1 - 1 as linenr_T, (*eap).line2 + 1 as linenr_T) == FAIL {
            emsg(gettext(
                b"cannot save undo information\0".as_ptr() as *const ::core::ffi::c_char
            ));
            return;
        }
        let cmd: *const ::core::ffi::c_char = (*eap).arg;
        let cmd_len: size_t = strlen(cmd);
        let lstate: *mut lua_State = global_lstate.get();
        let lcmd_len: size_t = cmd_len
            .wrapping_add(
                ::core::mem::size_of::<[::core::ffi::c_char; 31]>().wrapping_sub(1 as size_t),
            )
            .wrapping_add(
                ::core::mem::size_of::<[::core::ffi::c_char; 5]>().wrapping_sub(1 as size_t),
            );
        let mut lcmd: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
        if lcmd_len < IOSIZE as size_t {
            lcmd = IObuff.ptr() as *mut ::core::ffi::c_char;
        } else {
            lcmd = xmalloc(lcmd_len.wrapping_add(1 as size_t)) as *mut ::core::ffi::c_char;
        }
        memcpy(
            lcmd as *mut ::core::ffi::c_void,
            DOSTART.as_ptr() as *const ::core::ffi::c_void,
            ::core::mem::size_of::<[::core::ffi::c_char; 31]>().wrapping_sub(1 as size_t),
        );
        memcpy(
            lcmd.offset(::core::mem::size_of::<[::core::ffi::c_char; 31]>() as isize)
                .offset(-(1 as ::core::ffi::c_int as isize))
                as *mut ::core::ffi::c_void,
            cmd as *const ::core::ffi::c_void,
            cmd_len,
        );
        memcpy(
            lcmd.offset(::core::mem::size_of::<[::core::ffi::c_char; 31]>() as isize)
                .offset(-(1 as ::core::ffi::c_int as isize))
                .offset(cmd_len as isize) as *mut ::core::ffi::c_void,
            DOEND.as_ptr() as *const ::core::ffi::c_void,
            ::core::mem::size_of::<[::core::ffi::c_char; 5]>().wrapping_sub(1 as size_t),
        );
        if luaL_loadbuffer(
            lstate,
            lcmd,
            lcmd_len,
            b":luado\0".as_ptr() as *const ::core::ffi::c_char,
        ) != 0
        {
            nlua_error(
                lstate,
                gettext(b"E5109: Lua: %.*s\0".as_ptr() as *const ::core::ffi::c_char),
            );
            if lcmd_len >= IOSIZE as size_t {
                xfree(lcmd as *mut ::core::ffi::c_void);
            }
            return;
        }
        if lcmd_len >= IOSIZE as size_t {
            xfree(lcmd as *mut ::core::ffi::c_void);
        }
        if nlua_pcall(lstate, 0 as ::core::ffi::c_int, 1 as ::core::ffi::c_int) != 0 {
            nlua_error(
                lstate,
                gettext(b"E5110: Lua: %.*s\0".as_ptr() as *const ::core::ffi::c_char),
            );
            return;
        }
        let was_curbuf: *mut buf_T = curbuf.get();
        let mut l: linenr_T = (*eap).line1;
        while l <= (*eap).line2 {
            if l > (*curbuf.get()).b_ml.ml_line_count {
                break;
            }
            lua_pushvalue(lstate, -1 as ::core::ffi::c_int);
            let old_line: *const ::core::ffi::c_char = ml_get_buf(curbuf.get(), l);
            let old_line_len: colnr_T = ml_get_buf_len(curbuf.get(), l);
            lua_pushstring(lstate, old_line);
            lua_pushnumber(lstate, l as lua_Number);
            if nlua_pcall(lstate, 2 as ::core::ffi::c_int, 1 as ::core::ffi::c_int) != 0 {
                nlua_error(
                    lstate,
                    gettext(b"E5111: Lua: %.*s\0".as_ptr() as *const ::core::ffi::c_char),
                );
                break;
            } else {
                if curbuf.get() != was_curbuf || l > (*curbuf.get()).b_ml.ml_line_count {
                    break;
                }
                if lua_isstring(lstate, -1 as ::core::ffi::c_int) != 0 {
                    let mut new_line_len: size_t = 0;
                    let new_line: *const ::core::ffi::c_char =
                        lua_tolstring(lstate, -1 as ::core::ffi::c_int, &raw mut new_line_len);
                    let new_line_transformed: *mut ::core::ffi::c_char =
                        xmemdupz(new_line as *const ::core::ffi::c_void, new_line_len)
                            as *mut ::core::ffi::c_char;
                    let mut i: size_t = 0 as size_t;
                    while i < new_line_len {
                        if *new_line_transformed.offset(i as isize) as ::core::ffi::c_int == NUL {
                            *new_line_transformed.offset(i as isize) = '\n' as ::core::ffi::c_char;
                        }
                        i = i.wrapping_add(1);
                    }
                    ml_replace(l, new_line_transformed, false_0 != 0);
                    inserted_bytes(
                        l,
                        0 as colnr_T,
                        old_line_len as ::core::ffi::c_int,
                        new_line_len as ::core::ffi::c_int,
                    );
                }
                lua_settop(lstate, -1 as ::core::ffi::c_int - 1 as ::core::ffi::c_int);
                l += 1;
            }
        }
        lua_settop(lstate, -1 as ::core::ffi::c_int - 1 as ::core::ffi::c_int);
        check_cursor(curwin.get());
        redraw_curbuf_later(UPD_NOT_VALID);
    }
}

pub const DOSTART: [::core::ffi::c_char; 31] = unsafe {
    ::core::mem::transmute::<[u8; 31], [::core::ffi::c_char; 31]>(
        *b"return function(line, linenr) \0",
    )
};

pub const DOEND: [::core::ffi::c_char; 5] =
    unsafe { ::core::mem::transmute::<[u8; 5], [::core::ffi::c_char; 5]>(*b" end\0") };

pub unsafe fn ex_luafile(eap: *mut exarg_T) {
    unsafe {
        nlua_exec_file((*eap).arg);
    }
}

pub unsafe extern "C-unwind" fn nlua_exec_file(mut path: *const ::core::ffi::c_char) -> bool {
    unsafe {
        let lstate: *mut lua_State = global_lstate.get();
        if !strequal(path, b"-\0".as_ptr() as *const ::core::ffi::c_char) {
            lua_getfield(
                lstate,
                LUA_GLOBALSINDEX,
                b"loadfile\0".as_ptr() as *const ::core::ffi::c_char,
            );
            lua_pushstring(lstate, path);
        } else {
            let mut stdin_dup: FileDescriptor = FileDescriptor {
                fd: 0,
                buffer: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                read_pos: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                write_pos: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                wr: false,
                eof: false,
                non_blocking: false,
                bytes_read: 0,
            };
            let mut error: ::core::ffi::c_int = file_open_stdin(&raw mut stdin_dup);
            if error != 0 {
                return false_0 != 0;
            }
            let mut sb: StringBuilder = StringBuilder {
                size: 0 as size_t,
                capacity: 0 as size_t,
                items: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            };
            sb.capacity = 64 as size_t;
            sb.items = xrealloc(
                sb.items as *mut ::core::ffi::c_void,
                ::core::mem::size_of::<::core::ffi::c_char>().wrapping_mul(sb.capacity),
            ) as *mut ::core::ffi::c_char;
            loop {
                if got_int.get() {
                    file_close(&raw mut stdin_dup, false_0 != 0);
                    xfree(sb.items as *mut ::core::ffi::c_void);
                    sb.capacity = 0 as size_t;
                    sb.size = sb.capacity;
                    sb.items = ::core::ptr::null_mut::<::core::ffi::c_char>();
                    return false_0 != 0;
                }
                let mut read_size: ptrdiff_t = file_read(
                    &raw mut stdin_dup,
                    IObuff.ptr() as *mut ::core::ffi::c_char,
                    64 as size_t,
                );
                if read_size < 0 as ptrdiff_t {
                    file_close(&raw mut stdin_dup, false_0 != 0);
                    xfree(sb.items as *mut ::core::ffi::c_void);
                    sb.capacity = 0 as size_t;
                    sb.size = sb.capacity;
                    sb.items = ::core::ptr::null_mut::<::core::ffi::c_char>();
                    return false_0 != 0;
                }
                if read_size > 0 as ptrdiff_t {
                    if read_size as size_t > 0 as size_t {
                        if sb.capacity < sb.size.wrapping_add(read_size as size_t) {
                            sb.capacity = sb.size.wrapping_add(read_size as size_t);
                            sb.capacity = sb.capacity.wrapping_sub(1);
                            sb.capacity |= sb.capacity >> 1 as ::core::ffi::c_int;
                            sb.capacity |= sb.capacity >> 2 as ::core::ffi::c_int;
                            sb.capacity |= sb.capacity >> 4 as ::core::ffi::c_int;
                            sb.capacity |= sb.capacity >> 8 as ::core::ffi::c_int;
                            sb.capacity |= sb.capacity >> 16 as ::core::ffi::c_int;
                            sb.capacity = sb.capacity.wrapping_add(1);
                            sb.items = xrealloc(
                                sb.items as *mut ::core::ffi::c_void,
                                ::core::mem::size_of::<::core::ffi::c_char>()
                                    .wrapping_mul(sb.capacity),
                            ) as *mut ::core::ffi::c_char;
                        }
                        '_c2rust_label: {
                            if !sb.items.is_null() {
                            } else {
                                __assert_fail(
                                    b"(sb).items\0".as_ptr() as *const ::core::ffi::c_char,
                                    b"src/nvim/lua/executor.rs\0".as_ptr()
                                        as *const ::core::ffi::c_char,
                                    1910 as ::core::ffi::c_uint,
                                    b"_Bool nlua_exec_file(const char *)\0".as_ptr()
                                        as *const ::core::ffi::c_char,
                                );
                            }
                        };
                        memcpy(
                            sb.items.offset(sb.size as isize) as *mut ::core::ffi::c_void,
                            IObuff.ptr() as *mut ::core::ffi::c_char as *const ::core::ffi::c_void,
                            ::core::mem::size_of::<::core::ffi::c_char>()
                                .wrapping_mul(read_size as size_t),
                        );
                        sb.size = sb.size.wrapping_add(read_size as size_t);
                    }
                }
                if read_size < 64 as ptrdiff_t {
                    break;
                }
            }
            if sb.size == sb.capacity {
                sb.capacity = if sb.capacity != 0 {
                    sb.capacity << 1 as ::core::ffi::c_int
                } else {
                    8 as size_t
                };
                sb.items = xrealloc(
                    sb.items as *mut ::core::ffi::c_void,
                    ::core::mem::size_of::<::core::ffi::c_char>().wrapping_mul(sb.capacity),
                ) as *mut ::core::ffi::c_char;
            } else {
            };
            let c2rust_fresh2 = sb.size;
            sb.size = sb.size.wrapping_add(1);
            *sb.items.offset(c2rust_fresh2 as isize) = '\0' as ::core::ffi::c_char;
            file_close(&raw mut stdin_dup, false_0 != 0);
            lua_getfield(
                lstate,
                LUA_GLOBALSINDEX,
                b"loadstring\0".as_ptr() as *const ::core::ffi::c_char,
            );
            lua_pushstring(lstate, sb.items);
            xfree(sb.items as *mut ::core::ffi::c_void);
            sb.capacity = 0 as size_t;
            sb.size = sb.capacity;
            sb.items = ::core::ptr::null_mut::<::core::ffi::c_char>();
        }
        if nlua_pcall(lstate, 1 as ::core::ffi::c_int, 2 as ::core::ffi::c_int) != 0 {
            nlua_error(
                lstate,
                gettext(b"E5111: Lua: %.*s\0".as_ptr() as *const ::core::ffi::c_char),
            );
            return false_0 != 0;
        }
        if lua_type(lstate, -2 as ::core::ffi::c_int) == LUA_TNIL {
            nlua_error(
                lstate,
                gettext(b"E5112: Lua chunk: %.*s\0".as_ptr() as *const ::core::ffi::c_char),
            );
            '_c2rust_label_0: {
                if lua_type(lstate, -1 as ::core::ffi::c_int) == 0 as ::core::ffi::c_int {
                } else {
                    __assert_fail(
                        b"lua_isnil(lstate, -1)\0".as_ptr() as *const ::core::ffi::c_char,
                        b"src/nvim/lua/executor.rs\0".as_ptr() as *const ::core::ffi::c_char,
                        1936 as ::core::ffi::c_uint,
                        b"_Bool nlua_exec_file(const char *)\0".as_ptr()
                            as *const ::core::ffi::c_char,
                    );
                }
            };
            lua_settop(lstate, -1 as ::core::ffi::c_int - 1 as ::core::ffi::c_int);
            return false_0 != 0;
        }
        '_c2rust_label_1: {
            if lua_type(lstate, -1 as ::core::ffi::c_int) == 0 as ::core::ffi::c_int {
            } else {
                __assert_fail(
                    b"lua_isnil(lstate, -1)\0".as_ptr() as *const ::core::ffi::c_char,
                    b"src/nvim/lua/executor.rs\0".as_ptr() as *const ::core::ffi::c_char,
                    1942 as ::core::ffi::c_uint,
                    b"_Bool nlua_exec_file(const char *)\0".as_ptr() as *const ::core::ffi::c_char,
                );
            }
        };
        lua_settop(lstate, -1 as ::core::ffi::c_int - 1 as ::core::ffi::c_int);
        if nlua_pcall(lstate, 0 as ::core::ffi::c_int, 0 as ::core::ffi::c_int) != 0 {
            nlua_error(
                lstate,
                gettext(b"E5113: Lua chunk: %.*s\0".as_ptr() as *const ::core::ffi::c_char),
            );
            return false_0 != 0;
        }
        return true_0 != 0;
    }
}
