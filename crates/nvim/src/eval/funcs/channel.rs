//! Channels and servers: `chansend()`, `rpcrequest()`, `serverstart()` and
//! the rest of the RPC and socket surface.
#![deny(unsafe_op_in_unsafe_fn)]
#![allow(unsafe_code)]

use super::wrappers::{arg_string, list_alloc_ret};
use super::{
    ARENA_EMPTY, kChannelPartAll, kChannelPartRpc, kChannelPartStderr, kChannelPartStdin,
    kChannelPartStdout, kRetObject,
};
use crate::api::private::helpers::cstr_to_string;
use crate::autocmd::state::{autocmd_bufnr, autocmd_fname, autocmd_fname_full, autocmd_match};
use crate::channel::{
    channel_close, channel_connect, channel_from_stdio, channel_send, find_channel,
};
use crate::cstr;
use crate::eval::provider::{provider_call_nesting, provider_caller_scope};
use crate::eval::save_tv_as_string;
use crate::eval::typval::{
    NumBuf, tv_blob_len, tv_dict_get_bool, tv_dict_get_callback, tv_dict_get_number,
    tv_list_append_allocated_string, tv_list_append_string,
};
use crate::eval::userfunc::{restore_funccal, save_funccal, set_current_funccal};
use crate::event::libuv::uv_strerror;
use crate::ex_cmds::check_secure;
use crate::log::{LOGLVL_ERR, logmsg};
use crate::lua::executor::nlua_exec;
use crate::memory::{arena_finish, arena_mem_free, xfree, xmemdup, xstrdup};
use crate::message::e_invarg;
use crate::message::on_print_cb;
use crate::message::{emsg, emsg_ptr};
use crate::message_fmt::{c_str, msg_cstr};
use crate::msgpack_rpc::channel::{get_client_info, rpc_send_call, rpc_send_event};
use crate::msgpack_rpc::server::{
    server_address_list, server_address_new, server_start, server_stop,
};
use crate::os::cshim::gettext;
use crate::runtime::exestack;
use crate::runtime::state::current_sctx;
use crate::semsg;
use crate::semsg_multiline;
use crate::types::{
    Arena, ArenaMem, Array, Blob, CallbackReader, ChannelPart, Error, EvalFuncData, FuncCall,
    FuncCallEntry, Object, ScriptCtx, String_0, TypVal, VAR_BLOB, VAR_DICT, VAR_NUMBER, VAR_STRING,
    VarNumber, uint64_t,
};
use core::ffi::{CStr, c_char, c_int, c_void};
use core::ptr;

/// A cleared `CallbackReader`, which the option parsers fill in.
const NO_READER: CallbackReader = CallbackReader::none();

/// A cleared `Object`.
/// The `{stream}` names `chanclose()` accepts.
const CHANNEL_PARTS: [(&CStr, ChannelPart); 4] = [
    (c"stdin", kChannelPartStdin),
    (c"stdout", kChannelPartStdout),
    (c"stderr", kChannelPartStderr),
    (c"rpc", kChannelPartRpc),
];

/// The trailing arguments of `rpcnotify()`/`rpcrequest()` as an API `Array`
/// the caller owns.
fn trailing_args(args: &[TypVal], first: usize) -> Array {
    args[first.min(args.len())..]
        .iter()
        .map(Object::from)
        .collect()
}

/// `chanclose({id} [, {stream}])`
pub fn f_chanclose(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    let mut numbuf = NumBuf::new();
    result.write_number(0);
    // SAFETY throughout: the frame is live; `error` is a borrowed static message.
    if check_secure() {
        return;
    }
    if args[0].v_type() != VAR_NUMBER
        || (!args.get(1).is_some_and(|arg| arg.v_type() == VAR_STRING) && args.len() > 1)
    {
        emsg(gettext(e_invarg));
        return;
    }

    let mut part = kChannelPartAll;
    if args.get(1).is_some_and(|arg| arg.v_type() == VAR_STRING) {
        let stream = arg_string(&mut numbuf, &args[1]);
        let found = CHANNEL_PARTS
            .iter()
            .find(|(name, _)| unsafe { cstr::eq(stream, name.as_ptr()) });
        match found {
            Some(&(_, p)) => part = p,
            None => {
                // SAFETY: a message argument the caller holds as a NUL-terminated string.
                let stream = unsafe { c_str(stream) };
                semsg!("Invalid channel stream \"{stream}\"");
                return;
            }
        }
    }

    let mut error = ptr::null::<c_char>();
    result.write_number(unsafe {
        channel_close(args[0].number_or_zero() as uint64_t, part, &raw mut error)
    } as VarNumber);
    if result.number_or_zero() == 0 {
        unsafe { emsg_ptr(error) };
    }
}

/// `chansend({id}, {data})`
pub fn f_chansend(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    result.write_number(0);
    // SAFETY throughout: the frame is live; `input` is an allocation `channel_send`
    // adopts.
    if check_secure() {
        return;
    }
    if args[0].v_type() != VAR_NUMBER || args.len() <= 1 {
        emsg(gettext(e_invarg));
        return;
    }

    let mut input_len = 0isize;
    let input = if args[1].v_type() == VAR_BLOB {
        // A Blob goes over byte for byte; an empty one sends nothing
        // and is reported as a failure below.
        let b: *const Blob = args[1].blob_or_null();
        input_len = unsafe { tv_blob_len(b) } as isize;
        if input_len > 0 {
            unsafe { xmemdup((*b).bv_ga.ga_data, input_len as usize) as *mut c_char }
        } else {
            ptr::null_mut()
        }
    } else {
        // `false` for both: a List joins with NL, not CR-NL, and the
        // trailing NL is the caller's business.
        unsafe { save_tv_as_string(&args[1], &raw mut input_len, false, false) }
    };
    if input.is_null() {
        return;
    }

    let mut error = ptr::null::<c_char>();
    let id = args[0].number_or_zero() as uint64_t;
    let len = input_len as usize;
    let err = &raw mut error;
    let sent = unsafe { channel_send(id, input, len, true, err) };
    result.write_number(sent as VarNumber);
    if !error.is_null() {
        unsafe { emsg_ptr(error) };
    }
}

/// `rpcnotify({channel}, {event} [, {args}...])`
pub fn f_rpcnotify(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    let mut numbuf = NumBuf::new();
    result.write_number(0);
    // SAFETY throughout: the frame is live; `items` outlives the `Array` that borrows
    // it and the arena owns what the conversion allocates.
    if check_secure() {
        return;
    }
    // Channel 0 is the broadcast channel, so zero is allowed here where
    // `rpcrequest()` insists on a real one.
    if args[0].v_type() != VAR_NUMBER || args[0].number_or_zero() < 0 {
        let what = c"Channel id must be a positive integer".as_ptr();
        // SAFETY: a message argument the caller holds as a NUL-terminated string.
        let what = unsafe { c_str(what) };
        semsg!("E475: Invalid argument: {what}");
        return;
    }
    if args[1].v_type() != VAR_STRING {
        let what = c"Event type must be a string".as_ptr();
        // SAFETY: a message argument the caller holds as a NUL-terminated string.
        let what = unsafe { c_str(what) };
        semsg!("E475: Invalid argument: {what}");
        return;
    }

    let event_args = trailing_args(args, 2);
    let id = args[0].number_or_zero() as uint64_t;
    let event = arg_string(&mut numbuf, &args[1]);
    let ok = unsafe { rpc_send_event(id, event, event_args) };
    if !ok {
        let what = c"Channel doesn't exist".as_ptr();
        // SAFETY: a message argument the caller holds as a NUL-terminated string.
        let what = unsafe { c_str(what) };
        semsg!("E475: Invalid argument: {what}");
        return;
    }
    result.write_number(1);
}

/// The caller's context, restored around a provider's nested
/// `rpcrequest()`.
///
/// A provider call reaches back into the script that started it: the
/// request has to run with *that* script's context, autocommand state and
/// function call stack, not with whatever the provider left behind.
struct ProviderScope {
    sctx: ScriptCtx,
    autocmd_fname: *mut c_char,
    autocmd_match: *mut c_char,
    autocmd_fname_full: bool,
    autocmd_bufnr: c_int,
    funccal: FuncCallEntry,
}

impl ProviderScope {
    fn enter() -> Self {
        // SAFETY throughout: the caller's obligation.
        let mut saved = ProviderScope {
            sctx: current_sctx.get(),
            autocmd_fname: autocmd_fname.get(),
            autocmd_match: autocmd_match.get(),
            autocmd_fname_full: autocmd_fname_full.get(),
            autocmd_bufnr: autocmd_bufnr.get(),
            funccal: FuncCallEntry {
                top_funccal: ptr::null_mut(),
                next: ptr::null_mut(),
            },
        };
        unsafe { save_funccal(&raw mut saved.funccal) };

        // The scope is *read*, field by field, and nothing here writes it
        // back -- `exestack` is a different cell and `set_current_funccal`
        // touches neither -- so a shared borrow reaches every field and the
        // cell's address is not needed.
        provider_caller_scope.with(|scope| {
            current_sctx.set(scope.script_ctx);
            // Push the caller's execution-stack entry so that any message
            // names the caller's script, not the provider's.
            exestack.with_mut(|stack| stack.push(scope.es_entry));
            autocmd_fname.set(scope.autocmd_fname);
            autocmd_match.set(scope.autocmd_match);
            autocmd_fname_full.set(scope.autocmd_fname_full);
            autocmd_bufnr.set(scope.autocmd_bufnr);
            unsafe { set_current_funccal(scope.funccalp.cast::<FuncCall>()) };
        });
        saved
    }

    /// # Safety
    /// `self` came from [`enter`](Self::enter) and nothing else has touched
    /// the execution stack since.
    unsafe fn leave(self) {
        // SAFETY throughout: the caller's obligation.
        current_sctx.set(self.sctx);
        exestack.with_mut(|stack| {
            stack.pop();
        });
        autocmd_fname.set(self.autocmd_fname);
        autocmd_match.set(self.autocmd_match);
        autocmd_fname_full.set(self.autocmd_fname_full);
        autocmd_bufnr.set(self.autocmd_bufnr);
        unsafe { restore_funccal() };
    }
}

/// `rpcrequest({channel}, {method} [, {args}...])`
pub fn f_rpcrequest(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    let mut numbuf = NumBuf::new();
    result.write_number(0);
    // Read before `check_secure`, because that is when it still describes
    // this call rather than anything the request goes on to do.
    let nesting = provider_call_nesting.get();

    // SAFETY throughout: the frame is live; `items` outlives the `Array` that borrows
    // it, and both arenas own what they allocated until freed below.
    if check_secure() {
        return;
    }
    if args[0].v_type() != VAR_NUMBER || args[0].number_or_zero() <= 0 {
        let what = c"Channel id must be a positive integer".as_ptr();
        // SAFETY: a message argument the caller holds as a NUL-terminated string.
        let what = unsafe { c_str(what) };
        semsg!("E475: Invalid argument: {what}");
        return;
    }
    if args[1].v_type() != VAR_STRING {
        let what = c"Method name must be a string".as_ptr();
        // SAFETY: a message argument the caller holds as a NUL-terminated string.
        let what = unsafe { c_str(what) };
        semsg!("E475: Invalid argument: {what}");
        return;
    }

    let call_args = trailing_args(args, 2);

    let scope = (nesting != 0).then(|| ProviderScope::enter());

    let chan_id = args[0].number_or_zero() as uint64_t;
    let method = arg_string(&mut numbuf, &args[1]);
    let mut res_mem: ArenaMem = ptr::null_mut();
    let called = unsafe { rpc_send_call(chan_id, method, call_args, &raw mut res_mem) };

    if let Some(scope) = scope {
        unsafe { scope.leave() };
    }

    if let Err(err) = &called {
        // Name the peer when it told us what it is called.
        let chan = find_channel(chan_id);
        let name = if chan.is_null() {
            ptr::null()
        } else {
            unsafe { get_client_info(chan, c"name".as_ptr()) }
        };
        if name.is_null() {
            let why = err.message_or_empty();
            // SAFETY: the method name is NUL-terminated.
            let method = unsafe { c_str(method) };
            let msg = msg_cstr(why);
            semsg_multiline!(
                c"rpc_error",
                "Invoking '{method}' on channel {chan_id}:\n{msg}"
            );
        } else {
            // SAFETY: as above, plus the client name the channel answered.
            let why = err.message_or_empty();
            // SAFETY: both names are NUL-terminated.
            let (method, name) = unsafe { (c_str(method), c_str(name)) };
            let msg = msg_cstr(why);
            semsg_multiline!(
                c"rpc_error",
                "Invoking '{method}' on channel {chan_id} ({name}):\n{msg}"
            );
        }
    } else if let Ok(object) = called {
        // The response is this frame's, so its Lua references move into the
        // value rather than being copied.
        *result = TypVal::from(object);
    }
    unsafe { arena_mem_free(res_mem) };
}

/// `serverlist([{opts}])` — this instance's listen addresses, plus the
/// peers Lua knows about when asked for them.
pub fn f_serverlist(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    // SAFETY throughout: the frame is live; `addrs` is an allocation this body owns,
    // and the strings in it are handed to the List one at a time.
    let mut n = 0usize;
    let addrs = unsafe { server_address_list(&raw mut n) };
    let mut arena: Arena = ARENA_EMPTY;
    // The same addresses twice: once handed to the List, once copied
    // into the Array the Lua helper is passed.
    let mut addrs_arr = Array::with_capacity(n);
    let list = list_alloc_ret(result, n as isize);
    for i in 0..n {
        unsafe { tv_list_append_allocated_string(list, *addrs.add(i)) };
        let addr = unsafe { *addrs.add(i) };
        addrs_arr.push(Object::string(unsafe { cstr_to_string(addr) }));
    }

    if args.first().is_some_and(|arg| arg.v_type() == VAR_DICT)
        && unsafe { tv_dict_get_bool(args[0].dict_or_null(), c"peer".as_ptr(), 0) } != 0
    {
        let lua_args = Array::from(vec![Object::array(addrs_arr)]);

        let mut err = Error::none();
        const PEERS: &str = "return require('vim._core.server').serverlist(...)";
        let code = String_0::from(PEERS);
        let mem = &raw mut arena;
        let rv = match unsafe { nlua_exec(&code, ptr::null(), lua_args, kRetObject, mem) } {
            Ok(value) => value,
            Err(e) => {
                err = e;
                Object::Nil
            }
        };
        if err.is_set() {
            // A missing or broken helper is logged, not reported: the
            // local addresses above are still a useful answer.
            // SAFETY: the error's own message, NUL-terminated.
            let why = unsafe { c_str(err.message_or_empty().as_ptr()) };
            logmsg!(
                LOGLVL_ERR,
                c"f_serverlist",
                6338,
                "vim._core.serverlist failed: {why}"
            );
        } else {
            let peers = rv
                .as_array()
                .expect("`vim._core.server.serverlist()` answers with a list");
            for item in peers {
                let addr = item
                    .as_string()
                    .expect("`serverlist()` answers with a list of strings");
                // SAFETY: the address is the object's own, NUL-terminated.
                unsafe { tv_list_append_string(list, addr.data(), -1) };
            }
        }
    }

    unsafe { xfree(addrs as *mut c_void) };
    unsafe { arena_mem_free(arena_finish(&raw mut arena)) };
}

/// `serverstart([{address}])`
pub fn f_serverstart(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    let mut numbuf = NumBuf::new();
    result.write_string(ptr::null_mut());
    // SAFETY throughout: the frame is live; `address` and `addrs` are allocations this
    // body owns, bar the one entry handed to `result`.
    if check_secure() {
        return;
    }
    let address = if args.is_empty() {
        unsafe { server_address_new(ptr::null()) }
    } else if !args.first().is_some_and(|arg| arg.v_type() == VAR_STRING) {
        emsg(gettext(e_invarg));
        return;
    } else {
        unsafe { xstrdup(arg_string(&mut numbuf, &args[0])) }
    };

    let status = unsafe { server_start(address) };
    unsafe { xfree(address as *mut c_void) };
    if status != 0 {
        let why = if status > 0 {
            c"Unknown system error".as_ptr()
        } else {
            // SAFETY: `uv_strerror` answers a `'static` message for any code.
            unsafe { uv_strerror(status) }
        };
        // SAFETY: a message argument the caller holds as a NUL-terminated string.
        let why = unsafe { c_str(why) };
        semsg!("Failed to start server: {why}");
        return;
    }

    // The address just started is the last one in the list; the rest
    // are other people's and are released here.
    let mut n = 0usize;
    let addrs = unsafe { server_address_list(&raw mut n) };
    result.write_string(unsafe { *addrs.add(n - 1) });
    for i in 0..n - 1 {
        unsafe { xfree(*addrs.add(i) as *mut c_void) };
    }
    unsafe { xfree(addrs as *mut c_void) };
}

/// `serverstop({address})`
pub fn f_serverstop(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    // SAFETY throughout: the frame is live.
    if check_secure() {
        return;
    }
    if args[0].v_type() != VAR_STRING {
        emsg(gettext(e_invarg));
        return;
    }
    // Note the order: the return value is only cleared *after* the type
    // check, so a non-String argument answers 0 by way of the caller's
    // already-cleared return value rather than by this assignment.
    result.write_number(0);
    // v:_null_string stops nothing.
    if !args[0].string_or_null().is_null() {
        result.write_number(unsafe { server_stop(args[0].string_or_null(), false) } as VarNumber);
    }
}

/// `sockconnect({mode}, {address} [, {opts}])`
pub fn f_sockconnect(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    let mut numbuf = NumBuf::new();
    let mut numbuf2 = NumBuf::new();
    // SAFETY throughout: the frame is live; `on_data` is moved into `channel_connect`,
    // which adopts its callback.
    if args[0].v_type() != VAR_STRING || args[1].v_type() != VAR_STRING {
        emsg(gettext(e_invarg));
        return;
    }
    if !args.get(2).is_some_and(|arg| arg.v_type() == VAR_DICT) && args.len() > 2 {
        let arg0 = "expected dictionary";
        semsg!("E475: Invalid argument: {arg0}");
        return;
    }

    let mode = arg_string(&mut numbuf, &args[0]);
    let address = arg_string(&mut numbuf2, &args[1]);
    let tcp = if unsafe { cstr::eq_bytes(mode, b"tcp") } {
        true
    } else if unsafe { cstr::eq_bytes(mode, b"pipe") } {
        false
    } else {
        let arg0 = "invalid mode";
        semsg!("E475: Invalid argument: {arg0}");
        return;
    };

    let mut rpc = false;
    let mut on_data = NO_READER;
    if args.get(2).is_some_and(|arg| arg.v_type() == VAR_DICT) {
        let opts = args[2].dict_or_null();
        rpc = unsafe { tv_dict_get_number(opts, c"rpc".as_ptr()) } != 0;
        if !unsafe { tv_dict_get_callback(opts, c"on_data".as_ptr(), 7, &raw mut on_data.cb) } {
            return;
        }
        on_data.buffered = unsafe { tv_dict_get_number(opts, c"data_buffered".as_ptr()) } != 0;
        // Buffered with no callback means "collect it on the Dict", so
        // the Dict has to be reachable from the reader.
        if on_data.buffered && !on_data.cb.is_set() {
            on_data.self_0 = opts;
        }
    }

    let mut error = ptr::null::<c_char>();
    let id = unsafe { channel_connect(tcp, address, rpc, on_data, 50, &raw mut error) };
    if !error.is_null() {
        // SAFETY: a message argument the caller holds as a NUL-terminated string.
        let error = unsafe { c_str(error) };
        semsg!("connection failed: {error}");
    }
    result.write_number(id as VarNumber);
}

/// `stdioopen({opts})` — turn this process's own stdin/stdout into a
/// channel.
pub fn f_stdioopen(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    // SAFETY throughout: the frame is live; `on_stdin` is moved into
    // `channel_from_stdio`, which adopts its callback.
    if args[0].v_type() != VAR_DICT {
        emsg(gettext(e_invarg));
        return;
    }
    let opts = args[0].dict_or_null();
    let mut on_stdin = NO_READER;
    let rpc = unsafe { tv_dict_get_number(opts, c"rpc".as_ptr()) } != 0;
    if !unsafe { tv_dict_get_callback(opts, c"on_stdin".as_ptr(), 8, &raw mut on_stdin.cb) } {
        return;
    }
    // `on_print` is a global: there is only one stdio channel.
    if !unsafe { tv_dict_get_callback(opts, c"on_print".as_ptr(), 8, on_print_cb()) } {
        return;
    }
    on_stdin.buffered = unsafe { tv_dict_get_number(opts, c"stdin_buffered".as_ptr()) } != 0;
    if on_stdin.buffered && !on_stdin.cb.is_set() {
        on_stdin.self_0 = opts;
    }

    let mut error = ptr::null::<c_char>();
    let id = unsafe { channel_from_stdio(rpc, on_stdin, &raw mut error) };
    if id == 0 {
        // SAFETY: a message argument the caller holds as a NUL-terminated string.
        let error = unsafe { c_str(error) };
        semsg!("E905: Couldn't open stdio channel: {error}");
    }
    result.write_number(id as VarNumber);
}
