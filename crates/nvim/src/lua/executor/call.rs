//! `vim.call()` and the rpc entry points.
//!
//! [`nlua_call`] invokes a *Vimscript* function from Lua: it converts up to
//! `MAX_FUNC_ARGS` Lua values to `typval_T`s, calls through `call_func`, and
//! converts the result back.  `nlua_rpc` is `vim.rpcrequest()` and
//! `vim.rpcnotify()`, which differ only in whether they wait.

#![deny(unsafe_op_in_unsafe_fn)]

use core::ffi::{c_char, c_int, c_void};
use core::ptr;

use super::{
    FUNCEXE_INIT, LUA_INTERNAL_CALL, MAX_FUNC_ARGS, nlua_is_deferred_safe, viml_func_is_fast,
};
use crate::api::private::helpers::{
    api_clear_error, api_set_error, api_set_sctx, arena_array, try_enter, try_leave,
};
use crate::eval::typval::{TV_INITIAL_VALUE, tv_clear};
use crate::eval::userfunc::call_func;
use crate::ex_getln::{ERROR_INIT, TRY_STATE_INIT};
use crate::lua::converter::{nlua_pop_object, nlua_pop_typval, nlua_push_object, nlua_push_typval};
use crate::lua::ffi::{
    lua_error, lua_gettop, lua_pushstring, lua_pushvalue, luaL_checkinteger, luaL_checklstring,
    luaL_error,
};
use crate::main::{
    current_sctx, curwin, did_emsg, did_throw, e_fast_api_disabled, force_abort, suppress_errthrow,
};
use crate::memory::{ARENA_EMPTY, arena_finish, arena_mem_free, xrealloc};
use crate::msgpack_rpc::channel::{rpc_send_call, rpc_send_event};
use crate::strings::vim_snprintf;
use crate::types::{
    Arena, ArenaMem, Array, Object, consumed_blk, kErrorTypeException, kErrorTypeNone,
    kErrorTypeValidation, lua_State, size_t, uint64_t,
};
use ::libc::strlen;

/// How much of a rejected function's name the "not allowed in a fast event"
/// message quotes.
const NAME_LIMIT: size_t = 100;
/// `Vimscript function "%s"` less the `%s`, plus its terminator.
const QUOTED_FMT_LEN: size_t = size_of::<[c_char; 22]>();

/// `vim.call(name, ...)`: call a Vimscript function.
///
/// Every argument is converted into a stack array of typvals and every one
/// converted is cleared again, whichever way the call went — which is what
/// the running `i` counts.
///
/// # Safety
/// `lstate` must be a live Lua state holding this function's arguments.
pub unsafe extern "C-unwind" fn nlua_call(lstate: *mut lua_State) -> c_int {
    let mut refused = [0 as c_char; NAME_LIMIT + QUOTED_FMT_LEN + 1];
    unsafe {
        let mut err = ERROR_INIT;
        let mut name_len: size_t = 0;
        let name = luaL_checklstring(lstate, 1, &raw mut name_len);

        if !nlua_is_deferred_safe() && !viml_func_is_fast(name) {
            let length = strlen(name).min(NAME_LIMIT) + QUOTED_FMT_LEN;
            vim_snprintf(
                refused.as_mut_ptr(),
                length,
                c"Vimscript function \"%s\"".as_ptr(),
                name,
            );
            let fmt = &raw const e_fast_api_disabled as *const _;
            return luaL_error(lstate, fmt, refused.as_ptr());
        }

        let nargs = lua_gettop(lstate) - 1;
        if nargs > MAX_FUNC_ARGS as c_int {
            return luaL_error(lstate, c"Function called with too many arguments".as_ptr());
        }

        let mut vim_args = [TV_INITIAL_VALUE; MAX_FUNC_ARGS as usize + 1];
        let mut i: c_int = 0;
        'free_vim_args: {
            while i < nargs {
                lua_pushvalue(lstate, i + 2);
                if !nlua_pop_typval(lstate, vim_args.as_mut_ptr().offset(i as isize)) {
                    api_set_error(
                        &raw mut err,
                        kErrorTypeException,
                        c"error converting argument %d".as_ptr(),
                        i + 1,
                    );
                    break 'free_vim_args;
                }
                i += 1;
            }

            // Start the call from a clean exception state: a Lua caller has
            // no `:try` around it and must not inherit one.
            force_abort.set(false);
            suppress_errthrow.set(false);
            did_throw.set(false);
            did_emsg.set(0);

            let mut rettv = TV_INITIAL_VALUE;
            let mut funcexe = FUNCEXE_INIT;
            funcexe.fe_firstline = (*curwin.get()).w_cursor.lnum;
            funcexe.fe_lastline = (*curwin.get()).w_cursor.lnum;
            funcexe.fe_evaluate = true;

            let save_current_sctx = api_set_sctx(LUA_INTERNAL_CALL);
            let mut tstate = TRY_STATE_INIT;
            try_enter(&raw mut tstate);
            call_func(
                name,
                name_len as c_int,
                &raw mut rettv,
                nargs,
                vim_args.as_mut_ptr(),
                &raw mut funcexe,
            );
            try_leave(&raw mut tstate, &raw mut err);
            current_sctx.set(save_current_sctx);

            if err.type_0 == kErrorTypeNone {
                nlua_push_typval(lstate, &raw mut rettv, 0);
            }
            tv_clear(&raw mut rettv);
        }

        while i > 0 {
            i -= 1;
            tv_clear(vim_args.as_mut_ptr().offset(i as isize));
        }

        if err.type_0 != kErrorTypeNone {
            lua_pushstring(lstate, err.msg);
            api_clear_error(&raw mut err);
            return lua_error(lstate);
        }
        1
    }
}

/// `vim.rpcrequest(channel, method, ...)`.
///
/// # Safety
/// `lstate` must be a live Lua state holding this function's arguments.
pub(crate) unsafe extern "C-unwind" fn nlua_rpcrequest(lstate: *mut lua_State) -> c_int {
    unsafe {
        if !nlua_is_deferred_safe() {
            return luaL_error(
                lstate,
                &raw const e_fast_api_disabled as *const _,
                c"rpcrequest".as_ptr(),
            );
        }
        nlua_rpc(lstate, true)
    }
}

/// `vim.rpcnotify(channel, method, ...)`.
///
/// # Safety
/// As [`nlua_rpcrequest`].
pub(crate) unsafe extern "C-unwind" fn nlua_rpcnotify(lstate: *mut lua_State) -> c_int {
    unsafe { nlua_rpc(lstate, false) }
}

/// The shared body of the two rpc entry points. `request` waits for the
/// answer and pushes it; a notification pushes nothing.
///
/// # Safety
/// As [`nlua_rpcrequest`].
unsafe fn nlua_rpc(lstate: *mut lua_State, request: bool) -> c_int {
    unsafe {
        let mut name_len: size_t = 0;
        let chan_id = luaL_checkinteger(lstate, 1) as uint64_t;
        let name = luaL_checklstring(lstate, 2, &raw mut name_len);
        let nargs = lua_gettop(lstate) - 2;

        let mut err = ERROR_INIT;
        let mut arena: Arena = ARENA_EMPTY;
        let mut args: Array = arena_array(&raw mut arena, nargs as size_t);

        'check_err: {
            for i in 0..nargs {
                lua_pushvalue(lstate, i + 3);
                // The arena sized the array for exactly `nargs`, but a
                // conversion may push more; grow as upstream's `kv_push`
                // would.
                if args.size == args.capacity {
                    args.capacity = if args.capacity != 0 {
                        args.capacity << 1
                    } else {
                        8
                    };
                    args.items = xrealloc(
                        args.items.cast::<c_void>(),
                        size_of::<Object>().wrapping_mul(args.capacity),
                    )
                    .cast::<Object>();
                }
                *args.items.add(args.size) =
                    nlua_pop_object(lstate, false, &raw mut arena, &raw mut err);
                args.size = args.size.wrapping_add(1);
                if err.type_0 != kErrorTypeNone {
                    break 'check_err;
                }
            }

            if request {
                let mut res_mem: ArenaMem = ptr::null_mut::<consumed_blk>();
                let mut result = rpc_send_call(chan_id, name, args, &raw mut res_mem, &raw mut err);
                if err.type_0 == kErrorTypeNone {
                    nlua_push_object(lstate, &raw mut result, 0);
                    arena_mem_free(res_mem);
                }
            } else if !rpc_send_event(chan_id, name, args) {
                api_set_error(
                    &raw mut err,
                    kErrorTypeValidation,
                    c"Invalid channel: %lu".as_ptr(),
                    chan_id,
                );
            }
        }
        arena_mem_free(arena_finish(&raw mut arena));

        if err.type_0 != kErrorTypeNone {
            lua_pushstring(lstate, err.msg);
            api_clear_error(&raw mut err);
            return lua_error(lstate);
        }
        if request { 1 } else { 0 }
    }
}
