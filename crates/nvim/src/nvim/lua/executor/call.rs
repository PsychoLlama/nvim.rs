//! `vim.call()` and the rpc entry points.
//!
//! `nlua_call` invokes a *Vimscript* function from Lua: it converts up to
//! `MAX_FUNC_ARGS` Lua values to `typval_T`s, calls through `call_func`, and
//! converts the result back.  `nlua_rpc` is `vim.rpcrequest()` and
//! `vim.rpcnotify()`, which differ only in whether they wait.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;

pub unsafe extern "C-unwind" fn nlua_call(mut lstate: *mut lua_State) -> ::core::ffi::c_int {
    unsafe {
        let mut rettv: typval_T = typval_T {
            v_type: VAR_UNKNOWN,
            v_lock: VAR_UNLOCKED,
            vval: typval_vval_union { v_number: 0 },
        };
        let mut funcexe: funcexe_T = funcexe_T {
            fe_argv_func: None,
            fe_firstline: 0,
            fe_lastline: 0,
            fe_doesrange: ::core::ptr::null_mut::<bool>(),
            fe_evaluate: false,
            fe_partial: ::core::ptr::null_mut::<partial_T>(),
            fe_selfdict: ::core::ptr::null_mut::<dict_T>(),
            fe_basetv: ::core::ptr::null_mut::<typval_T>(),
            fe_found_var: false,
        };
        let mut save_current_sctx: sctx_T = sctx_T {
            sc_sid: 0,
            sc_seq: 0,
            sc_lnum: 0,
            sc_chan: 0,
        };
        let mut err: Error = Error {
            type_0: kErrorTypeNone,
            msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        };
        let mut name_len: size_t = 0;
        let mut name: *const ::core::ffi::c_char =
            luaL_checklstring(lstate, 1 as ::core::ffi::c_int, &raw mut name_len);
        if !nlua_is_deferred_safe() && !viml_func_is_fast(name) {
            let mut length: size_t = (if strlen(name) < 100 as size_t {
                strlen(name)
            } else {
                100 as size_t
            })
            .wrapping_add(::core::mem::size_of::<[::core::ffi::c_char; 22]>());
            vim_snprintf(
                IObuff.ptr() as *mut ::core::ffi::c_char,
                length,
                b"Vimscript function \"%s\"\0".as_ptr() as *const ::core::ffi::c_char,
                name,
            );
            let mut ret: ::core::ffi::c_int = luaL_error(
                lstate,
                &raw const e_fast_api_disabled as *const ::core::ffi::c_char,
                IObuff.ptr() as *mut ::core::ffi::c_char,
            );
            return ret;
        }
        let mut nargs: ::core::ffi::c_int = lua_gettop(lstate) - 1 as ::core::ffi::c_int;
        if nargs > MAX_FUNC_ARGS as ::core::ffi::c_int {
            return luaL_error(
                lstate,
                b"Function called with too many arguments\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        let mut vim_args: [typval_T; 21] = [typval_T {
            v_type: VAR_UNKNOWN,
            v_lock: VAR_UNLOCKED,
            vval: typval_vval_union { v_number: 0 },
        }; 21];
        let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        '_free_vim_args: {
            while i < nargs {
                lua_pushvalue(lstate, i + 2 as ::core::ffi::c_int);
                if !nlua_pop_typval(
                    lstate,
                    (&raw mut vim_args as *mut typval_T).offset(i as isize),
                ) {
                    api_set_error(
                        &raw mut err,
                        kErrorTypeException,
                        b"error converting argument %d\0".as_ptr() as *const ::core::ffi::c_char,
                        i + 1 as ::core::ffi::c_int,
                    );
                    break '_free_vim_args;
                } else {
                    i += 1;
                }
            }
            force_abort.set(false_0 != 0);
            suppress_errthrow.set(false_0 != 0);
            did_throw.set(false_0 != 0);
            did_emsg.set(false_0);
            rettv = typval_T {
                v_type: VAR_UNKNOWN,
                v_lock: VAR_UNLOCKED,
                vval: typval_vval_union { v_number: 0 },
            };
            funcexe = FUNCEXE_INIT;
            funcexe.fe_firstline = (*curwin.get()).w_cursor.lnum;
            funcexe.fe_lastline = (*curwin.get()).w_cursor.lnum;
            funcexe.fe_evaluate = true_0 != 0;
            save_current_sctx = api_set_sctx(LUA_INTERNAL_CALL);
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
            call_func(
                name,
                name_len as ::core::ffi::c_int,
                &raw mut rettv,
                nargs,
                &raw mut vim_args as *mut typval_T,
                &raw mut funcexe,
            );
            try_leave(&raw mut tstate, &raw mut err);
            current_sctx.set(save_current_sctx);
            if !(err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int) {
                nlua_push_typval(lstate, &raw mut rettv, 0 as ::core::ffi::c_int);
            }
            tv_clear(&raw mut rettv);
        }
        while i > 0 as ::core::ffi::c_int {
            i -= 1;
            tv_clear((&raw mut vim_args as *mut typval_T).offset(i as isize));
        }
        if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
            lua_pushstring(lstate, err.msg);
            api_clear_error(&raw mut err);
            return lua_error(lstate);
        }
        return 1 as ::core::ffi::c_int;
    }
}

pub(crate) unsafe extern "C-unwind" fn nlua_rpcrequest(
    mut lstate: *mut lua_State,
) -> ::core::ffi::c_int {
    unsafe {
        if !nlua_is_deferred_safe() {
            return luaL_error(
                lstate,
                &raw const e_fast_api_disabled as *const ::core::ffi::c_char,
                b"rpcrequest\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        return nlua_rpc(lstate, true_0 != 0);
    }
}

pub(crate) unsafe extern "C-unwind" fn nlua_rpcnotify(
    mut lstate: *mut lua_State,
) -> ::core::ffi::c_int {
    unsafe {
        return nlua_rpc(lstate, false_0 != 0);
    }
}

unsafe extern "C-unwind" fn nlua_rpc(
    mut lstate: *mut lua_State,
    mut request: bool,
) -> ::core::ffi::c_int {
    unsafe {
        let mut name_len: size_t = 0;
        let mut chan_id: uint64_t = luaL_checkinteger(lstate, 1 as ::core::ffi::c_int) as uint64_t;
        let mut name: *const ::core::ffi::c_char =
            luaL_checklstring(lstate, 2 as ::core::ffi::c_int, &raw mut name_len);
        let mut nargs: ::core::ffi::c_int = lua_gettop(lstate) - 2 as ::core::ffi::c_int;
        let mut err: Error = Error {
            type_0: kErrorTypeNone,
            msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        };
        let mut arena: Arena = ARENA_EMPTY;
        let mut args: Array = arena_array(&raw mut arena, nargs as size_t);
        let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        '_check_err: {
            while i < nargs {
                lua_pushvalue(lstate, i + 3 as ::core::ffi::c_int);
                if args.size == args.capacity {
                    args.capacity = if args.capacity != 0 {
                        args.capacity << 1 as ::core::ffi::c_int
                    } else {
                        8 as size_t
                    };
                    args.items = xrealloc(
                        args.items as *mut ::core::ffi::c_void,
                        ::core::mem::size_of::<Object>().wrapping_mul(args.capacity),
                    ) as *mut Object;
                } else {
                };
                let c2rust_fresh1 = args.size;
                args.size = args.size.wrapping_add(1);
                *args.items.offset(c2rust_fresh1 as isize) =
                    nlua_pop_Object(lstate, false, &raw mut arena, &raw mut err);
                if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
                    break '_check_err;
                }
                i += 1;
            }
            if request {
                let mut res_mem: ArenaMem = ::core::ptr::null_mut::<consumed_blk>();
                let mut result: Object =
                    rpc_send_call(chan_id, name, args, &raw mut res_mem, &raw mut err);
                if !(err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int) {
                    nlua_push_Object(lstate, &raw mut result, 0 as ::core::ffi::c_int);
                    arena_mem_free(res_mem);
                }
            } else if !rpc_send_event(chan_id, name, args) {
                api_set_error(
                    &raw mut err,
                    kErrorTypeValidation,
                    b"Invalid channel: %lu\0".as_ptr() as *const ::core::ffi::c_char,
                    chan_id,
                );
            }
        }
        arena_mem_free(arena_finish(&raw mut arena));
        if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
            lua_pushstring(lstate, err.msg);
            api_clear_error(&raw mut err);
            return lua_error(lstate);
        }
        return if request as ::core::ffi::c_int != 0 {
            1 as ::core::ffi::c_int
        } else {
            0 as ::core::ffi::c_int
        };
    }
}
