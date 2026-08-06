//! `print()`, `require()` and `vim.debug()`.
//!
//! Neovim replaces Lua's `print` so its output goes through the editor's
//! message path rather than stdout, and defers it when called from a fast
//! callback (`nlua_print_event`).  `nlua_require` is the wrapper that keeps
//! `package.loaded` and the runtime path in step, and `nlua_debug` is the
//! `vim.debug()` read-eval-print loop.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;

unsafe extern "C" fn nlua_print_event(mut argv: *mut *mut ::core::ffi::c_void) {
    unsafe {
        let mut msg: HlMessage = HlMessage {
            size: 0 as size_t,
            capacity: 0 as size_t,
            items: ::core::ptr::null_mut::<HlMessageChunk>(),
        };
        let mut chunk: HlMessageChunk = HlMessageChunk {
            text: String_0 {
                data: *argv.offset(0 as ::core::ffi::c_int as isize) as *mut ::core::ffi::c_char,
                size: ((*argv.offset(1 as ::core::ffi::c_int as isize)).expose_provenance()
                    as intptr_t as size_t)
                    .wrapping_sub(1 as size_t),
            },
            hl_id: 0 as ::core::ffi::c_int,
        };
        if msg.size == msg.capacity {
            msg.capacity = if msg.capacity != 0 {
                msg.capacity << 1 as ::core::ffi::c_int
            } else {
                8 as size_t
            };
            msg.items = xrealloc(
                msg.items as *mut ::core::ffi::c_void,
                ::core::mem::size_of::<HlMessageChunk>().wrapping_mul(msg.capacity),
            ) as *mut HlMessageChunk;
        } else {
        };
        let c2rust_fresh0 = msg.size;
        msg.size = msg.size.wrapping_add(1);
        *msg.items.offset(c2rust_fresh0 as isize) = chunk;
        let mut needs_clear: bool = false_0 != 0;
        msg_multihl(
            object {
                type_0: kObjectTypeNil,
                data: C2Rust_Unnamed_11 { boolean: false },
            },
            msg,
            b"lua_print\0".as_ptr() as *const ::core::ffi::c_char,
            true_0 != 0,
            false_0 != 0,
            ::core::ptr::null_mut::<MessageData>(),
            &raw mut needs_clear,
        );
    }
}

pub(crate) unsafe extern "C-unwind" fn nlua_print(lstate: *mut lua_State) -> ::core::ffi::c_int {
    unsafe {
        let mut is_thread: bool = false;
        let nargs: ::core::ffi::c_int = lua_gettop(lstate);
        lua_getfield(
            lstate,
            LUA_GLOBALSINDEX,
            b"tostring\0".as_ptr() as *const ::core::ffi::c_char,
        );
        let mut errmsg: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
        let mut errmsg_len: size_t = 0 as size_t;
        let mut msg_ga: garray_T = garray_T {
            ga_len: 0,
            ga_maxlen: 0,
            ga_itemsize: 0,
            ga_growsize: 0,
            ga_data: ::core::ptr::null_mut::<::core::ffi::c_void>(),
        };
        ga_init(
            &raw mut msg_ga,
            1 as ::core::ffi::c_int,
            80 as ::core::ffi::c_int,
        );
        let mut curargidx: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
        '_nlua_print_error: {
            while curargidx <= nargs {
                lua_pushvalue(lstate, -1 as ::core::ffi::c_int);
                lua_pushvalue(lstate, curargidx);
                if lua_pcall(
                    lstate,
                    1 as ::core::ffi::c_int,
                    1 as ::core::ffi::c_int,
                    0 as ::core::ffi::c_int,
                ) != 0
                {
                    errmsg = lua_tolstring(lstate, -1 as ::core::ffi::c_int, &raw mut errmsg_len);
                    break '_nlua_print_error;
                } else {
                    let mut len: size_t = 0;
                    let s: *const ::core::ffi::c_char =
                        lua_tolstring(lstate, -1 as ::core::ffi::c_int, &raw mut len);
                    if s.is_null() {
                        errmsg =
                            b"<Unknown error: lua_tolstring returned NULL for tostring result>\0"
                                .as_ptr() as *const ::core::ffi::c_char;
                        errmsg_len = ::core::mem::size_of::<[::core::ffi::c_char; 65]>()
                            .wrapping_sub(1 as usize)
                            as size_t;
                        break '_nlua_print_error;
                    } else {
                        ga_concat_len(&raw mut msg_ga, s, len);
                        if curargidx < nargs {
                            ga_append(&raw mut msg_ga, ' ' as uint8_t);
                        }
                        lua_settop(lstate, -1 as ::core::ffi::c_int - 1 as ::core::ffi::c_int);
                        curargidx += 1;
                    }
                }
            }
            ga_append(&raw mut msg_ga, NUL as uint8_t);
            lua_getfield(
                lstate,
                LUA_REGISTRYINDEX,
                b"nvim.thread\0".as_ptr() as *const ::core::ffi::c_char,
            );
            is_thread = lua_toboolean(lstate, -1 as ::core::ffi::c_int) != 0;
            lua_settop(lstate, -1 as ::core::ffi::c_int - 1 as ::core::ffi::c_int);
            if is_thread {
                loop_schedule_deferred(
                    main_loop.ptr(),
                    Event {
                        handler: Some(
                            nlua_print_event
                                as unsafe extern "C" fn(*mut *mut ::core::ffi::c_void) -> (),
                        ),
                        argv: [
                            msg_ga.ga_data,
                            ::core::ptr::with_exposed_provenance_mut::<::core::ffi::c_void>(
                                msg_ga.ga_len as intptr_t as usize,
                            ),
                            ::core::ptr::null_mut::<::core::ffi::c_void>(),
                            ::core::ptr::null_mut::<::core::ffi::c_void>(),
                            ::core::ptr::null_mut::<::core::ffi::c_void>(),
                            ::core::ptr::null_mut::<::core::ffi::c_void>(),
                            ::core::ptr::null_mut::<::core::ffi::c_void>(),
                            ::core::ptr::null_mut::<::core::ffi::c_void>(),
                            ::core::ptr::null_mut::<::core::ffi::c_void>(),
                            ::core::ptr::null_mut::<::core::ffi::c_void>(),
                        ],
                    },
                );
            } else if in_fast_callback.get() != 0 {
                multiqueue_put_event(
                    (*main_loop.ptr()).events,
                    Event {
                        handler: Some(
                            nlua_print_event
                                as unsafe extern "C" fn(*mut *mut ::core::ffi::c_void) -> (),
                        ),
                        argv: [
                            msg_ga.ga_data,
                            ::core::ptr::with_exposed_provenance_mut::<::core::ffi::c_void>(
                                msg_ga.ga_len as intptr_t as usize,
                            ),
                            ::core::ptr::null_mut::<::core::ffi::c_void>(),
                            ::core::ptr::null_mut::<::core::ffi::c_void>(),
                            ::core::ptr::null_mut::<::core::ffi::c_void>(),
                            ::core::ptr::null_mut::<::core::ffi::c_void>(),
                            ::core::ptr::null_mut::<::core::ffi::c_void>(),
                            ::core::ptr::null_mut::<::core::ffi::c_void>(),
                            ::core::ptr::null_mut::<::core::ffi::c_void>(),
                            ::core::ptr::null_mut::<::core::ffi::c_void>(),
                        ],
                    },
                );
            } else {
                let mut c2rust_lvalue: [*mut ::core::ffi::c_void; 2] = [
                    msg_ga.ga_data,
                    ::core::ptr::with_exposed_provenance_mut::<::core::ffi::c_void>(
                        msg_ga.ga_len as intptr_t as usize,
                    ),
                ];
                nlua_print_event(&raw mut c2rust_lvalue as *mut *mut ::core::ffi::c_void);
            }
            return 0 as ::core::ffi::c_int;
        }
        ga_clear(&raw mut msg_ga);
        let mut buff: *mut ::core::ffi::c_char =
            xmalloc(IOSIZE as size_t) as *mut ::core::ffi::c_char;
        let mut fmt: *const ::core::ffi::c_char =
            gettext(b"E5114: Converting print argument #%i: %.*s\0".as_ptr()
                as *const ::core::ffi::c_char);
        let mut len_0: size_t = vim_snprintf(
            buff,
            IOSIZE as size_t,
            fmt,
            curargidx,
            errmsg_len as ::core::ffi::c_int,
            errmsg,
        ) as size_t;
        lua_pushlstring(lstate, buff, len_0);
        xfree(buff as *mut ::core::ffi::c_void);
        return lua_error(lstate);
    }
}

pub(crate) unsafe extern "C-unwind" fn nlua_require(lstate: *mut lua_State) -> ::core::ffi::c_int {
    unsafe {
        let mut name: *const ::core::ffi::c_char = luaL_checklstring(
            lstate,
            1 as ::core::ffi::c_int,
            ::core::ptr::null_mut::<size_t>(),
        );
        lua_settop(lstate, 1 as ::core::ffi::c_int);
        lua_getfield(
            lstate,
            LUA_REGISTRYINDEX,
            b"_LOADED\0".as_ptr() as *const ::core::ffi::c_char,
        );
        lua_getfield(lstate, 2 as ::core::ffi::c_int, name);
        if lua_toboolean(lstate, -1 as ::core::ffi::c_int) != 0 {
            return 1 as ::core::ffi::c_int;
        }
        lua_settop(lstate, -2 as ::core::ffi::c_int - 1 as ::core::ffi::c_int);
        nlua_pushref(lstate, require_ref.get());
        lua_insert(lstate, 1 as ::core::ffi::c_int);
        if (*time_fd.ptr()).is_null() {
            lua_getfield(
                lstate,
                LUA_GLOBALSINDEX,
                b"require\0".as_ptr() as *const ::core::ffi::c_char,
            );
            if lua_iscfunction(lstate, -1 as ::core::ffi::c_int) != 0
                && lua_tocfunction(lstate, -1 as ::core::ffi::c_int).is_some_and(|f| {
                    ::core::ptr::fn_addr_eq(
                        f,
                        nlua_require
                            as unsafe extern "C-unwind" fn(*mut lua_State) -> ::core::ffi::c_int,
                    )
                })
            {
                lua_pushvalue(lstate, 1 as ::core::ffi::c_int);
                lua_setfield(
                    lstate,
                    LUA_GLOBALSINDEX,
                    b"require\0".as_ptr() as *const ::core::ffi::c_char,
                );
            }
            lua_settop(lstate, -1 as ::core::ffi::c_int - 1 as ::core::ffi::c_int);
            lua_call(lstate, 1 as ::core::ffi::c_int, 1 as ::core::ffi::c_int);
            return 1 as ::core::ffi::c_int;
        }
        let mut rel_time: proftime_T = 0;
        let mut start_time: proftime_T = 0;
        (rel_time, start_time) = time_push();
        let mut status: ::core::ffi::c_int = lua_pcall(
            lstate,
            1 as ::core::ffi::c_int,
            1 as ::core::ffi::c_int,
            0 as ::core::ffi::c_int,
        );
        if status == 0 as ::core::ffi::c_int {
            vim_snprintf(
                IObuff.ptr() as *mut ::core::ffi::c_char,
                IOSIZE as size_t,
                b"require('%s')\0".as_ptr() as *const ::core::ffi::c_char,
                name,
            );
            time_msg(
                IObuff.ptr() as *mut ::core::ffi::c_char,
                &raw mut start_time,
            );
        }
        time_pop(rel_time);
        return if status == 0 as ::core::ffi::c_int {
            1 as ::core::ffi::c_int
        } else {
            lua_error(lstate)
        };
    }
}

pub(crate) unsafe extern "C-unwind" fn nlua_debug(
    mut lstate: *mut lua_State,
) -> ::core::ffi::c_int {
    unsafe {
        let input_args: [typval_T; 2] = [
            typval_T {
                v_type: VAR_STRING,
                v_lock: VAR_FIXED,
                vval: typval_vval_union {
                    v_string: b"lua_debug> \0".as_ptr() as *const ::core::ffi::c_char
                        as *mut ::core::ffi::c_char,
                },
            },
            typval_T {
                v_type: VAR_UNKNOWN,
                v_lock: VAR_UNLOCKED,
                vval: typval_vval_union { v_number: 0 },
            },
        ];
        loop {
            lua_settop(lstate, 0 as ::core::ffi::c_int);
            let mut input: typval_T = typval_T {
                v_type: VAR_UNKNOWN,
                v_lock: VAR_UNLOCKED,
                vval: typval_vval_union { v_number: 0 },
            };
            get_user_input(
                &raw const input_args as *const typval_T,
                &raw mut input,
                false_0 != 0,
                false_0 != 0,
            );
            if ui_has(kUICmdline) {
                snprintf(
                    IObuff.ptr() as *mut ::core::ffi::c_char,
                    IOSIZE as size_t,
                    b"lua_debug> %s\0".as_ptr() as *const ::core::ffi::c_char,
                    input.vval.v_string,
                );
                ui_ext_cmdline_block_append(0 as size_t, IObuff.ptr() as *mut ::core::ffi::c_char);
            } else {
                msg_putchar('\n' as ::core::ffi::c_int);
            }
            if input.v_type as ::core::ffi::c_uint
                != VAR_STRING as ::core::ffi::c_int as ::core::ffi::c_uint
                || input.vval.v_string.is_null()
                || *input.vval.v_string as ::core::ffi::c_int == NUL
                || strcmp(
                    input.vval.v_string,
                    b"cont\0".as_ptr() as *const ::core::ffi::c_char,
                ) == 0 as ::core::ffi::c_int
            {
                tv_clear(&raw mut input);
                if ui_has(kUICmdline) {
                    ui_ext_cmdline_block_leave();
                }
                return 0 as ::core::ffi::c_int;
            }
            if luaL_loadbuffer(
                lstate,
                input.vval.v_string,
                strlen(input.vval.v_string),
                b"=(debug command)\0".as_ptr() as *const ::core::ffi::c_char,
            ) != 0
            {
                nlua_error(
                    lstate,
                    gettext(b"E5115: Loading Lua debug string: %.*s\0".as_ptr()
                        as *const ::core::ffi::c_char),
                );
            } else if nlua_pcall(lstate, 0 as ::core::ffi::c_int, 0 as ::core::ffi::c_int) != 0 {
                nlua_error(
                    lstate,
                    gettext(b"E5116: Calling Lua debug string: %.*s\0".as_ptr()
                        as *const ::core::ffi::c_char),
                );
            }
            tv_clear(&raw mut input);
        }
    }
}
