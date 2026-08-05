//! Channels and servers: `chansend()`, `rpcrequest()`, `serverstart()` and
//! the rest of the RPC and socket surface.
#![deny(unsafe_op_in_unsafe_fn)]

use super::args::{Args, frame};
use super::{
    ARENA_EMPTY, ARRAY_DICT_INIT, C2Rust_Unnamed_16, C2Rust_Unnamed_22, GA_EMPTY_INIT_VALUE,
    MAX_FUNC_ARGS, false_0, kChannelPartAll, kChannelPartRpc, kChannelPartStderr,
    kChannelPartStdin, kChannelPartStdout, kRetObject,
};
use crate::src::nvim::api::private::converter::{object_to_vim, vim_to_object};
use crate::src::nvim::api::private::helpers::{api_clear_error, arena_array, cstr_as_string};
use crate::src::nvim::channel::{
    channel_close, channel_connect, channel_from_stdio, channel_send, find_channel,
};
use crate::src::nvim::eval::save_tv_as_string;
use crate::src::nvim::eval::typval::{
    kCallbackNone, tv_blob_len, tv_dict_get_bool, tv_dict_get_callback, tv_dict_get_number,
    tv_get_string, tv_list_alloc_ret, tv_list_append_allocated_string, tv_list_append_string,
};
use crate::src::nvim::eval::userfunc::{restore_funccal, save_funccal, set_current_funccal};
use crate::src::nvim::event::libuv::uv_strerror;
use crate::src::nvim::ex_cmds::check_secure;
use crate::src::nvim::garray::ga_grow;
use crate::src::nvim::log::{LOGLVL_ERR, logmsg};
use crate::src::nvim::lua::executor::nlua_exec;
use crate::src::nvim::main::{
    autocmd_bufnr, autocmd_fname, autocmd_fname_full, autocmd_match, current_sctx, e_invarg,
    e_invarg2, e_stdiochan2, on_print, provider_call_nesting, provider_caller_scope,
};
use crate::src::nvim::memory::{arena_finish, arena_mem_free, xfree, xmemdup, xstrdup};
use crate::src::nvim::message::{emsg, semsg, semsg_multiline};
use crate::src::nvim::msgpack_rpc::channel::{get_client_info, rpc_send_call, rpc_send_event};
use crate::src::nvim::msgpack_rpc::server::{
    server_address_list, server_address_new, server_start, server_stop,
};
use crate::src::nvim::os::libc::{gettext, strcmp};
use crate::src::nvim::runtime::exestack;
use crate::src::nvim::types::api::kErrorTypeNone;
use crate::src::nvim::types::{
    Arena, ArenaMem, Array, Callback, CallbackReader, ChannelPart, Error, EvalFuncData, Object,
    String_0, VAR_BLOB, VAR_DICT, VAR_NUMBER, VAR_STRING, blob_T, dict_T, estack_T,
    funccal_entry_T, funccall_T, kObjectTypeArray, kObjectTypeNil, kObjectTypeString, object,
    sctx_T, typval_T, uint64_t, varnumber_T,
};
use core::ffi::{CStr, c_char, c_int, c_void};
use core::ptr;

/// A cleared `CallbackReader`, which the option parsers fill in.
const NO_READER: CallbackReader = CallbackReader {
    cb: Callback {
        data: C2Rust_Unnamed_22 {
            funcref: ptr::null_mut::<c_char>(),
        },
        type_0: kCallbackNone,
    },
    self_0: ptr::null_mut::<dict_T>(),
    buffer: GA_EMPTY_INIT_VALUE,
    eof: false,
    buffered: false,
    fwd_err: false,
    type_0: ptr::null::<c_char>(),
};

/// A cleared `Object`.
const NIL: Object = Object {
    type_0: kObjectTypeNil,
    data: C2Rust_Unnamed_16 { boolean: false },
};

/// The `{stream}` names `chanclose()` accepts.
const CHANNEL_PARTS: [(&CStr, ChannelPart); 4] = [
    (c"stdin", kChannelPartStdin),
    (c"stdout", kChannelPartStdout),
    (c"stderr", kChannelPartStderr),
    (c"rpc", kChannelPartRpc),
];

/// The trailing arguments of `rpcnotify()`/`rpcrequest()`, converted to an
/// API `Array` backed by the caller's storage.
///
/// # Safety
/// `args` is a live call frame, `items` is at least `MAX_FUNC_ARGS` long
/// and outlives the returned `Array`, and `arena` is a live arena that owns
/// what the conversion allocates.
unsafe fn trailing_args(
    args: Args,
    first: usize,
    items: &mut [Object; MAX_FUNC_ARGS as usize],
    arena: *mut Arena,
) -> Array {
    let mut out = ARRAY_DICT_INIT;
    out.capacity = MAX_FUNC_ARGS as usize;
    out.items = items.as_mut_ptr();
    // SAFETY: the caller's obligation; the loop stops at the terminator,
    // which the dispatcher writes at or before `MAX_ARGS`.
    unsafe {
        let mut i = first;
        while args.has(i) {
            *out.items.add(out.size) = vim_to_object(args.ptr(i), arena, true);
            out.size += 1;
            i += 1;
        }
    }
    out
}

/// `chanclose({id} [, {stream}])`
pub unsafe extern "C" fn f_chanclose(
    argvars: *mut typval_T,
    rettv: *mut typval_T,
    _fptr: EvalFuncData,
) {
    let (args, rettv) = frame!(argvars, rettv);
    rettv.v_type = VAR_NUMBER;
    rettv.vval.v_number = 0;
    // SAFETY: the frame is live; `error` is a borrowed static message.
    unsafe {
        if check_secure() {
            return;
        }
        if args.ty(0) != VAR_NUMBER || (args.ty(1) != VAR_STRING && args.has(1)) {
            emsg(gettext(e_invarg.ptr() as *const c_char));
            return;
        }

        let mut part = kChannelPartAll;
        if args.ty(1) == VAR_STRING {
            let stream = tv_get_string(args.ptr(1));
            let found = CHANNEL_PARTS
                .iter()
                .find(|(name, _)| strcmp(stream, name.as_ptr()) == 0);
            match found {
                Some(&(_, p)) => part = p,
                None => {
                    semsg(gettext(c"Invalid channel stream \"%s\"".as_ptr()), stream);
                    return;
                }
            }
        }

        let mut error = ptr::null::<c_char>();
        rettv.vval.v_number =
            channel_close(args.get(0).vval.v_number as uint64_t, part, &raw mut error)
                as varnumber_T;
        if rettv.vval.v_number == 0 {
            emsg(error);
        }
    }
}

/// `chansend({id}, {data})`
pub unsafe extern "C" fn f_chansend(
    argvars: *mut typval_T,
    rettv: *mut typval_T,
    _fptr: EvalFuncData,
) {
    let (args, rettv) = frame!(argvars, rettv);
    rettv.v_type = VAR_NUMBER;
    rettv.vval.v_number = 0;
    // SAFETY: the frame is live; `input` is an allocation `channel_send`
    // adopts.
    unsafe {
        if check_secure() {
            return;
        }
        if args.ty(0) != VAR_NUMBER || !args.has(1) {
            emsg(gettext(e_invarg.ptr() as *const c_char));
            return;
        }

        let mut input_len = 0isize;
        let input = if args.ty(1) == VAR_BLOB {
            // A Blob goes over byte for byte; an empty one sends nothing
            // and is reported as a failure below.
            let b: *const blob_T = args.get(1).vval.v_blob;
            input_len = tv_blob_len(b) as isize;
            if input_len > 0 {
                xmemdup((*b).bv_ga.ga_data, input_len as usize) as *mut c_char
            } else {
                ptr::null_mut()
            }
        } else {
            // `false` for both: a List joins with NL, not CR-NL, and the
            // trailing NL is the caller's business.
            save_tv_as_string(args.ptr(1), &raw mut input_len, false, false)
        };
        if input.is_null() {
            return;
        }

        let mut error = ptr::null::<c_char>();
        rettv.vval.v_number = channel_send(
            args.get(0).vval.v_number as uint64_t,
            input,
            input_len as usize,
            true,
            &raw mut error,
        ) as varnumber_T;
        if !error.is_null() {
            emsg(error);
        }
    }
}

/// `rpcnotify({channel}, {event} [, {args}...])`
pub unsafe extern "C" fn f_rpcnotify(
    argvars: *mut typval_T,
    rettv: *mut typval_T,
    _fptr: EvalFuncData,
) {
    let (args, rettv) = frame!(argvars, rettv);
    rettv.v_type = VAR_NUMBER;
    rettv.vval.v_number = 0;
    // SAFETY: the frame is live; `items` outlives the `Array` that borrows
    // it and the arena owns what the conversion allocates.
    unsafe {
        if check_secure() {
            return;
        }
        // Channel 0 is the broadcast channel, so zero is allowed here where
        // `rpcrequest()` insists on a real one.
        if args.ty(0) != VAR_NUMBER || args.get(0).vval.v_number < 0 {
            semsg(
                gettext(e_invarg2.ptr() as *const c_char),
                c"Channel id must be a positive integer".as_ptr(),
            );
            return;
        }
        if args.ty(1) != VAR_STRING {
            semsg(
                gettext(e_invarg2.ptr() as *const c_char),
                c"Event type must be a string".as_ptr(),
            );
            return;
        }

        let mut items = [NIL; MAX_FUNC_ARGS as usize];
        let mut arena: Arena = ARENA_EMPTY;
        let event_args = trailing_args(args, 2, &mut items, &raw mut arena);
        let ok = rpc_send_event(
            args.get(0).vval.v_number as uint64_t,
            tv_get_string(args.ptr(1)),
            event_args,
        );
        arena_mem_free(arena_finish(&raw mut arena));
        if !ok {
            semsg(
                gettext(e_invarg2.ptr() as *const c_char),
                c"Channel doesn't exist".as_ptr(),
            );
            return;
        }
        rettv.vval.v_number = 1;
    }
}

/// The caller's context, restored around a provider's nested
/// `rpcrequest()`.
///
/// A provider call reaches back into the script that started it: the
/// request has to run with *that* script's context, autocommand state and
/// function call stack, not with whatever the provider left behind.
struct ProviderScope {
    sctx: sctx_T,
    autocmd_fname: *mut c_char,
    autocmd_match: *mut c_char,
    autocmd_fname_full: bool,
    autocmd_bufnr: c_int,
    funccal: funccal_entry_T,
}

impl ProviderScope {
    /// # Safety
    /// Called only when `provider_call_nesting` is non-zero, so that
    /// `provider_caller_scope` holds a live scope.
    unsafe fn enter() -> Self {
        // SAFETY: the caller's obligation.
        unsafe {
            let mut saved = ProviderScope {
                sctx: current_sctx.get(),
                autocmd_fname: autocmd_fname.get(),
                autocmd_match: autocmd_match.get(),
                autocmd_fname_full: autocmd_fname_full.get(),
                autocmd_bufnr: autocmd_bufnr.get(),
                funccal: funccal_entry_T {
                    top_funccal: ptr::null_mut(),
                    next: ptr::null_mut(),
                },
            };
            save_funccal(&raw mut saved.funccal);

            let scope = provider_caller_scope.ptr();
            current_sctx.set((*scope).script_ctx);
            // Push the caller's execution-stack entry so that any message
            // names the caller's script, not the provider's.
            ga_grow(exestack.ptr(), 1);
            let stack = exestack.ptr();
            *((*stack).ga_data as *mut estack_T).add((*stack).ga_len as usize) = (*scope).es_entry;
            (*stack).ga_len += 1;
            autocmd_fname.set((*scope).autocmd_fname);
            autocmd_match.set((*scope).autocmd_match);
            autocmd_fname_full.set((*scope).autocmd_fname_full);
            autocmd_bufnr.set((*scope).autocmd_bufnr);
            set_current_funccal((*scope).funccalp as *mut funccall_T);
            saved
        }
    }

    /// # Safety
    /// `self` came from [`enter`](Self::enter) and nothing else has touched
    /// the execution stack since.
    unsafe fn leave(self) {
        // SAFETY: the caller's obligation.
        unsafe {
            current_sctx.set(self.sctx);
            (*exestack.ptr()).ga_len -= 1;
            autocmd_fname.set(self.autocmd_fname);
            autocmd_match.set(self.autocmd_match);
            autocmd_fname_full.set(self.autocmd_fname_full);
            autocmd_bufnr.set(self.autocmd_bufnr);
            restore_funccal();
        }
    }
}

/// `rpcrequest({channel}, {method} [, {args}...])`
pub unsafe extern "C" fn f_rpcrequest(
    argvars: *mut typval_T,
    rettv: *mut typval_T,
    _fptr: EvalFuncData,
) {
    let (args, rettv) = frame!(argvars, rettv);
    rettv.v_type = VAR_NUMBER;
    rettv.vval.v_number = 0;
    // Read before `check_secure`, because that is when it still describes
    // this call rather than anything the request goes on to do.
    let nesting = provider_call_nesting.get();

    // SAFETY: the frame is live; `items` outlives the `Array` that borrows
    // it, and both arenas own what they allocated until freed below.
    unsafe {
        if check_secure() {
            return;
        }
        if args.ty(0) != VAR_NUMBER || args.get(0).vval.v_number <= 0 {
            semsg(
                gettext(e_invarg2.ptr() as *const c_char),
                c"Channel id must be a positive integer".as_ptr(),
            );
            return;
        }
        if args.ty(1) != VAR_STRING {
            semsg(
                gettext(e_invarg2.ptr() as *const c_char),
                c"Method name must be a string".as_ptr(),
            );
            return;
        }

        let mut items = [NIL; MAX_FUNC_ARGS as usize];
        let mut arena: Arena = ARENA_EMPTY;
        let call_args = trailing_args(args, 2, &mut items, &raw mut arena);

        let scope = (nesting != 0).then(|| ProviderScope::enter());

        let mut err = Error {
            type_0: kErrorTypeNone,
            msg: ptr::null_mut(),
        };
        let chan_id = args.get(0).vval.v_number as uint64_t;
        let method = tv_get_string(args.ptr(1));
        let mut res_mem: ArenaMem = ptr::null_mut();
        let result = rpc_send_call(chan_id, method, call_args, &raw mut res_mem, &raw mut err);
        arena_mem_free(arena_finish(&raw mut arena));

        if let Some(scope) = scope {
            scope.leave();
        }

        if err.type_0 != kErrorTypeNone {
            // Name the peer when it told us what it is called.
            let chan = find_channel(chan_id);
            let name = if chan.is_null() {
                ptr::null()
            } else {
                get_client_info(chan, c"name".as_ptr())
            };
            if name.is_null() {
                semsg_multiline(
                    c"rpc_error".as_ptr(),
                    c"Invoking '%s' on channel %lu:\n%s".as_ptr(),
                    method,
                    chan_id,
                    err.msg,
                );
            } else {
                semsg_multiline(
                    c"rpc_error".as_ptr(),
                    c"Invoking '%s' on channel %lu (%s):\n%s".as_ptr(),
                    method,
                    chan_id,
                    name,
                    err.msg,
                );
            }
        } else {
            object_to_vim(result, rettv, &raw mut err);
        }
        arena_mem_free(res_mem);
        api_clear_error(&raw mut err);
    }
}

/// `serverlist([{opts}])` — this instance's listen addresses, plus the
/// peers Lua knows about when asked for them.
pub unsafe extern "C" fn f_serverlist(
    argvars: *mut typval_T,
    rettv: *mut typval_T,
    _fptr: EvalFuncData,
) {
    let (args, rettv) = frame!(argvars, rettv);
    // SAFETY: the frame is live; `addrs` is an allocation this body owns,
    // and the strings in it are handed to the List one at a time.
    unsafe {
        let mut n = 0usize;
        let addrs = server_address_list(&raw mut n);
        let mut arena: Arena = ARENA_EMPTY;
        // The same addresses twice: once handed to the List, once copied
        // into the Array the Lua helper is passed.
        let mut addrs_arr = arena_array(&raw mut arena, n);
        let list = tv_list_alloc_ret(rettv, n as isize);
        for i in 0..n {
            tv_list_append_allocated_string(list, *addrs.add(i));
            *addrs_arr.items.add(addrs_arr.size) = object {
                type_0: kObjectTypeString,
                data: C2Rust_Unnamed_16 {
                    string: cstr_as_string(*addrs.add(i)),
                },
            };
            addrs_arr.size += 1;
        }

        if args.ty(0) == VAR_DICT
            && tv_dict_get_bool(args.get(0).vval.v_dict, c"peer".as_ptr(), false_0) != 0
        {
            let mut items = [NIL; 1];
            let mut lua_args = ARRAY_DICT_INIT;
            lua_args.capacity = 1;
            lua_args.items = items.as_mut_ptr();
            *lua_args.items = object {
                type_0: kObjectTypeArray,
                data: C2Rust_Unnamed_16 { array: addrs_arr },
            };
            lua_args.size = 1;

            let mut err = Error {
                type_0: kErrorTypeNone,
                msg: ptr::null_mut(),
            };
            const PEERS: &str = "return require('vim._core.server').serverlist(...)";
            let rv = nlua_exec(
                String_0 {
                    data: PEERS.as_ptr() as *mut c_char,
                    size: PEERS.len(),
                },
                ptr::null(),
                lua_args,
                kRetObject,
                &raw mut arena,
                &raw mut err,
            );
            if err.type_0 != kErrorTypeNone {
                // A missing or broken helper is logged, not reported: the
                // local addresses above are still a useful answer.
                logmsg(
                    LOGLVL_ERR,
                    ptr::null(),
                    c"f_serverlist".as_ptr(),
                    6338,
                    true,
                    c"vim._core.serverlist failed: %s".as_ptr(),
                    err.msg,
                );
            } else {
                for i in 0..rv.data.array.size {
                    tv_list_append_string(list, (*rv.data.array.items.add(i)).data.string.data, -1);
                }
            }
        }

        xfree(addrs as *mut c_void);
        arena_mem_free(arena_finish(&raw mut arena));
    }
}

/// `serverstart([{address}])`
pub unsafe extern "C" fn f_serverstart(
    argvars: *mut typval_T,
    rettv: *mut typval_T,
    _fptr: EvalFuncData,
) {
    let (args, rettv) = frame!(argvars, rettv);
    rettv.v_type = VAR_STRING;
    rettv.vval.v_string = ptr::null_mut();
    // SAFETY: the frame is live; `address` and `addrs` are allocations this
    // body owns, bar the one entry handed to `rettv`.
    unsafe {
        if check_secure() {
            return;
        }
        let address = if !args.has(0) {
            server_address_new(ptr::null())
        } else if args.ty(0) != VAR_STRING {
            emsg(gettext(e_invarg.ptr() as *const c_char));
            return;
        } else {
            xstrdup(tv_get_string(args.ptr(0)))
        };

        let result = server_start(address);
        xfree(address as *mut c_void);
        if result != 0 {
            semsg(
                c"Failed to start server: %s".as_ptr(),
                if result > 0 {
                    c"Unknown system error".as_ptr()
                } else {
                    uv_strerror(result)
                },
            );
            return;
        }

        // The address just started is the last one in the list; the rest
        // are other people's and are released here.
        let mut n = 0usize;
        let addrs = server_address_list(&raw mut n);
        rettv.vval.v_string = *addrs.add(n - 1);
        for i in 0..n - 1 {
            xfree(*addrs.add(i) as *mut c_void);
        }
        xfree(addrs as *mut c_void);
    }
}

/// `serverstop({address})`
pub unsafe extern "C" fn f_serverstop(
    argvars: *mut typval_T,
    rettv: *mut typval_T,
    _fptr: EvalFuncData,
) {
    let (args, rettv) = frame!(argvars, rettv);
    // SAFETY: the frame is live.
    unsafe {
        if check_secure() {
            return;
        }
        if args.ty(0) != VAR_STRING {
            emsg(gettext(e_invarg.ptr() as *const c_char));
            return;
        }
        // Note the order: the return value is only cleared *after* the type
        // check, so a non-String argument answers 0 by way of the caller's
        // already-cleared return value rather than by this assignment.
        rettv.v_type = VAR_NUMBER;
        rettv.vval.v_number = 0;
        // v:_null_string stops nothing.
        if !args.get(0).vval.v_string.is_null() {
            rettv.vval.v_number = server_stop(args.get(0).vval.v_string, false) as varnumber_T;
        }
    }
}

/// `sockconnect({mode}, {address} [, {opts}])`
pub unsafe extern "C" fn f_sockconnect(
    argvars: *mut typval_T,
    rettv: *mut typval_T,
    _fptr: EvalFuncData,
) {
    let (args, rettv) = frame!(argvars, rettv);
    // SAFETY: the frame is live; `on_data` is moved into `channel_connect`,
    // which adopts its callback.
    unsafe {
        if args.ty(0) != VAR_STRING || args.ty(1) != VAR_STRING {
            emsg(gettext(e_invarg.ptr() as *const c_char));
            return;
        }
        if args.ty(2) != VAR_DICT && args.has(2) {
            semsg(
                gettext(e_invarg2.ptr() as *const c_char),
                c"expected dictionary".as_ptr(),
            );
            return;
        }

        let mode = tv_get_string(args.ptr(0));
        let address = tv_get_string(args.ptr(1));
        let tcp = if strcmp(mode, c"tcp".as_ptr()) == 0 {
            true
        } else if strcmp(mode, c"pipe".as_ptr()) == 0 {
            false
        } else {
            semsg(
                gettext(e_invarg2.ptr() as *const c_char),
                c"invalid mode".as_ptr(),
            );
            return;
        };

        let mut rpc = false;
        let mut on_data = NO_READER;
        if args.ty(2) == VAR_DICT {
            let opts = args.get(2).vval.v_dict;
            rpc = tv_dict_get_number(opts, c"rpc".as_ptr()) != 0;
            if !tv_dict_get_callback(opts, c"on_data".as_ptr(), 7, &raw mut on_data.cb) {
                return;
            }
            on_data.buffered = tv_dict_get_number(opts, c"data_buffered".as_ptr()) != 0;
            // Buffered with no callback means "collect it on the Dict", so
            // the Dict has to be reachable from the reader.
            if on_data.buffered && on_data.cb.type_0 == kCallbackNone {
                on_data.self_0 = opts;
            }
        }

        let mut error = ptr::null::<c_char>();
        let id = channel_connect(tcp, address, rpc, on_data, 50, &raw mut error);
        if !error.is_null() {
            semsg(gettext(c"connection failed: %s".as_ptr()), error);
        }
        rettv.vval.v_number = id as varnumber_T;
        rettv.v_type = VAR_NUMBER;
    }
}

/// `stdioopen({opts})` — turn this process's own stdin/stdout into a
/// channel.
pub unsafe extern "C" fn f_stdioopen(
    argvars: *mut typval_T,
    rettv: *mut typval_T,
    _fptr: EvalFuncData,
) {
    let (args, rettv) = frame!(argvars, rettv);
    // SAFETY: the frame is live; `on_stdin` is moved into
    // `channel_from_stdio`, which adopts its callback.
    unsafe {
        if args.ty(0) != VAR_DICT {
            emsg(gettext(e_invarg.ptr() as *const c_char));
            return;
        }
        let opts = args.get(0).vval.v_dict;
        let mut on_stdin = NO_READER;
        let rpc = tv_dict_get_number(opts, c"rpc".as_ptr()) != 0;
        if !tv_dict_get_callback(opts, c"on_stdin".as_ptr(), 8, &raw mut on_stdin.cb) {
            return;
        }
        // `on_print` is a global: there is only one stdio channel.
        if !tv_dict_get_callback(opts, c"on_print".as_ptr(), 8, on_print.ptr()) {
            return;
        }
        on_stdin.buffered = tv_dict_get_number(opts, c"stdin_buffered".as_ptr()) != 0;
        if on_stdin.buffered && on_stdin.cb.type_0 == kCallbackNone {
            on_stdin.self_0 = opts;
        }

        let mut error = ptr::null::<c_char>();
        let id = channel_from_stdio(rpc, on_stdin, &raw mut error);
        if id == 0 {
            semsg(e_stdiochan2.ptr() as *const c_char, error);
        }
        rettv.vval.v_number = id as varnumber_T;
        rettv.v_type = VAR_NUMBER;
    }
}
