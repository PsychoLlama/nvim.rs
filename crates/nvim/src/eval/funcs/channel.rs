//! Channels and servers: `chansend()`, `rpcrequest()`, `serverstart()` and
//! the rest of the RPC and socket surface.
#![forbid(unsafe_code)]

use super::{
    kChannelPartAll, kChannelPartRpc, kChannelPartStderr, kChannelPartStdin, kChannelPartStdout,
    kRetObject,
};
use crate::autocmd::state::{autocmd_bufnr, autocmd_fname, autocmd_fname_full, autocmd_match};
use crate::channel::{channel_close, channel_connect, channel_from_stdio, channel_send};
use crate::eval::provider::{provider_call_nesting, provider_caller_scope};
use crate::eval::save_tv_as_string;
use crate::eval::typval::{
    NumBuf, blob_bytes, dict_get_bool, dict_get_callback, dict_get_number, tv_list_alloc,
};
use crate::eval::userfunc::{CallStackAside, set_current_fc};
use crate::ex_cmds::check_secure;
use crate::log::{LOGLVL_ERR, logmsg};
use crate::lua::executor::nlua_exec_cstr;
use crate::memory::{ThinCString, XString};
use crate::message::e_invarg;
use crate::message::emsg;
use crate::message::with_on_print;
use crate::message_fmt::msg_cstr;
use crate::msgpack_rpc::channel::{client_info, rpc_call_with, rpc_send_event};
use crate::msgpack_rpc::server::{
    server_address_generated, server_addresses, server_start, server_stop,
};
use crate::os::cshim::gettext;
use crate::os::fs::os_strerror;
use crate::runtime::EXESTACK;
use crate::runtime::state::current_sctx;
use crate::semsg;
use crate::semsg_multiline;
use crate::types::{
    Array, CallbackReader, ChannelPart, EvalFuncData, Object, ScriptCtx, String_0, TypVal,
    VAR_BLOB, VAR_DICT, VAR_NUMBER, VAR_STRING, VarNumber, uint64_t,
};
use core::ffi::{CStr, c_int};

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
        let stream = numbuf.string(&args[1]);
        let found = CHANNEL_PARTS.iter().find(|&&(name, _)| name == stream);
        match found {
            Some(&(_, p)) => part = p,
            None => {
                let stream = msg_cstr(stream);
                semsg!("Invalid channel stream \"{stream}\"");
                return;
            }
        }
    }

    match channel_close(args[0].number_or_zero() as uint64_t, part) {
        Ok(()) => result.write_number(1),
        Err(why) => {
            emsg(why);
        }
    }
}

/// `chansend({id}, {data})`
pub fn f_chansend(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    result.write_number(0);
    if check_secure() {
        return;
    }
    if args[0].v_type() != VAR_NUMBER || args.len() <= 1 {
        emsg(gettext(e_invarg));
        return;
    }

    let input: Vec<u8> = if args[1].v_type() == VAR_BLOB {
        // A Blob goes over byte for byte; an empty one sends nothing
        // and is reported as a failure below.
        blob_bytes(args[1].blob_ref()).to_vec()
    } else {
        // `false` for both: a List joins with NL, not CR-NL, and the
        // trailing NL is the caller's business.
        match save_tv_as_string(&args[1], false, false) {
            Ok(Some(text)) => text,
            Ok(None) | Err(_) => return,
        }
    };
    if args[1].v_type() == VAR_BLOB && input.is_empty() {
        return;
    }

    let id = args[0].number_or_zero() as uint64_t;
    match channel_send(id, &input) {
        Ok(sent) => result.write_number(sent as VarNumber),
        Err(why) => {
            emsg(why);
        }
    }
}

/// `rpcnotify({channel}, {event} [, {args}...])`
pub fn f_rpcnotify(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    let mut numbuf = NumBuf::new();
    result.write_number(0);
    if check_secure() {
        return;
    }
    // Channel 0 is the broadcast channel, so zero is allowed here where
    // `rpcrequest()` insists on a real one.
    if args[0].v_type() != VAR_NUMBER || args[0].number_or_zero() < 0 {
        semsg!("E475: Invalid argument: Channel id must be a positive integer");
        return;
    }
    if args[1].v_type() != VAR_STRING {
        semsg!("E475: Invalid argument: Event type must be a string");
        return;
    }

    let event_args = trailing_args(args, 2);
    let id = args[0].number_or_zero() as uint64_t;
    let event = numbuf.string(&args[1]);
    let ok = rpc_send_event(id, event, event_args);
    if !ok {
        semsg!("E475: Invalid argument: Channel doesn't exist");
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
    autocmd_fname: Option<XString>,
    autocmd_match: Option<XString>,
    autocmd_fname_full: bool,
    autocmd_bufnr: c_int,
    /// Held for its drop, which puts the call stack back.
    _funccal: CallStackAside,
}

impl ProviderScope {
    fn enter() -> Self {
        let saved = ProviderScope {
            sctx: current_sctx.get(),
            autocmd_fname: autocmd_fname.take(),
            autocmd_match: autocmd_match.take(),
            autocmd_fname_full: autocmd_fname_full.get(),
            autocmd_bufnr: autocmd_bufnr.get(),
            _funccal: CallStackAside::new(),
        };

        // The scope is *read*, field by field, and nothing here writes it
        // back -- `EXESTACK` is a different cell and `set_current_funccal`
        // touches neither -- so a shared borrow reaches every field and the
        // cell's address is not needed.
        provider_caller_scope.with(|scope| {
            current_sctx.set(scope.script_ctx);
            // Push the caller's execution-stack entry so that any message
            // names the caller's script, not the provider's.
            let entry = scope.es_entry.clone();
            EXESTACK.with_mut(|stack| stack.push(entry));
            autocmd_fname.set(scope.autocmd_fname.clone());
            autocmd_match.set(scope.autocmd_match.clone());
            autocmd_fname_full.set(scope.autocmd_fname_full);
            autocmd_bufnr.set(scope.autocmd_bufnr);
            set_current_fc(scope.funccalp);
        });
        saved
    }

    /// Put back what [`enter`](Self::enter) saved, popping the execution
    /// stack entry it pushed.
    fn leave(self) {
        current_sctx.set(self.sctx);
        EXESTACK.with_mut(|stack| {
            stack.pop();
        });
        autocmd_fname.set(self.autocmd_fname);
        autocmd_match.set(self.autocmd_match);
        autocmd_fname_full.set(self.autocmd_fname_full);
        autocmd_bufnr.set(self.autocmd_bufnr);
        // Dropping `self._funccal` last puts the call stack back.
    }
}

/// `rpcrequest({channel}, {method} [, {args}...])`
pub fn f_rpcrequest(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    let mut numbuf = NumBuf::new();
    result.write_number(0);
    // Read before `check_secure`, because that is when it still describes
    // this call rather than anything the request goes on to do.
    let nesting = provider_call_nesting.get();

    if check_secure() {
        return;
    }
    if args[0].v_type() != VAR_NUMBER || args[0].number_or_zero() <= 0 {
        semsg!("E475: Invalid argument: Channel id must be a positive integer");
        return;
    }
    if args[1].v_type() != VAR_STRING {
        semsg!("E475: Invalid argument: Method name must be a string");
        return;
    }

    let call_args = trailing_args(args, 2);

    let scope = (nesting != 0).then(ProviderScope::enter);

    let chan_id = args[0].number_or_zero() as uint64_t;
    let method = numbuf.string(&args[1]);
    // The response is this frame's, so its Lua references move into the
    // value rather than being copied.
    let called = rpc_call_with(chan_id, method, call_args, TypVal::from);

    if let Some(scope) = scope {
        scope.leave();
    }

    match called {
        Ok(value) => *result = value,
        Err(err) => {
            let (method, msg) = (msg_cstr(method), msg_cstr(err.message_or_empty()));
            // Name the peer when it told us what it is called.
            match client_info(chan_id, c"name") {
                None => {
                    semsg_multiline!(
                        c"rpc_error",
                        "Invoking '{method}' on channel {chan_id}:\n{msg}"
                    );
                }
                Some(name) => {
                    let name = msg_cstr(&name);
                    semsg_multiline!(
                        c"rpc_error",
                        "Invoking '{method}' on channel {chan_id} ({name}):\n{msg}"
                    );
                }
            }
        }
    }
}

/// `serverlist([{opts}])` — this instance's listen addresses, plus the
/// peers Lua knows about when asked for them.
pub fn f_serverlist(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    let addrs = server_addresses();
    // The same addresses twice: once in the List, once copied into the
    // Array the Lua helper is passed.
    let list = tv_list_alloc(addrs.len() as isize);
    result.write_list(Some(list.clone()));
    let mut addrs_arr = Array::with_capacity(addrs.len());
    for addr in &addrs {
        list.edit().push_str(Some(addr));
        addrs_arr.push(Object::string(String_0::from_cstr(addr)));
    }

    if args.first().is_some_and(|arg| arg.v_type() == VAR_DICT)
        && dict_get_bool(args[0].dict_ref(), b"peer", 0) != 0
    {
        let lua_args = Array::from(vec![Object::array(addrs_arr)]);
        const PEERS: &CStr = c"return require('vim._core.server').serverlist(...)";
        match nlua_exec_cstr(PEERS, lua_args, kRetObject) {
            Err(err) => {
                // A missing or broken helper is logged, not reported: the
                // local addresses above are still a useful answer.
                let why = msg_cstr(err.message_or_empty());
                logmsg!(
                    LOGLVL_ERR,
                    c"f_serverlist",
                    6338,
                    "vim._core.serverlist failed: {why}"
                );
            }
            Ok(rv) => {
                let peers = rv
                    .as_array()
                    .expect("`vim._core.server.serverlist()` answers with a list");
                for item in peers {
                    let addr = item
                        .as_string()
                        .expect("`serverlist()` answers with a list of strings");
                    let addr = (!addr.is_null()).then(|| addr.as_cstr());
                    list.edit().push_str(addr);
                }
            }
        }
    }
}

/// `serverstart([{address}])`
pub fn f_serverstart(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    let mut numbuf = NumBuf::new();
    result.write_string(None);
    if check_secure() {
        return;
    }
    let address = if args.is_empty() {
        server_address_generated()
    } else if !args.first().is_some_and(|arg| arg.v_type() == VAR_STRING) {
        emsg(gettext(e_invarg));
        return;
    } else {
        XString::from_cstr(numbuf.string(&args[0]))
    };

    let status = server_start(Some(address.as_cstr()));
    drop(address);
    if status != 0 {
        let why = if status > 0 {
            c"Unknown system error"
        } else {
            os_strerror(status)
        };
        let why = msg_cstr(why);
        semsg!("Failed to start server: {why}");
        return;
    }

    // The address just started is the last one in the list.
    let started = server_addresses().pop();
    result.write_string(started.map(|addr| ThinCString::from_cstr(&addr)));
}

/// `serverstop({address})`
pub fn f_serverstop(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
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
    if let Some(address) = args[0].string_ref() {
        result.write_number(server_stop(address.as_cstr(), false) as VarNumber);
    }
}

/// `sockconnect({mode}, {address} [, {opts}])`
pub fn f_sockconnect(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    let mut numbuf = NumBuf::new();
    let mut numbuf2 = NumBuf::new();
    if args[0].v_type() != VAR_STRING || args[1].v_type() != VAR_STRING {
        emsg(gettext(e_invarg));
        return;
    }
    if !args.get(2).is_some_and(|arg| arg.v_type() == VAR_DICT) && args.len() > 2 {
        let arg0 = "expected dictionary";
        semsg!("E475: Invalid argument: {arg0}");
        return;
    }

    let mode = numbuf.bytes(&args[0]);
    let address = numbuf2.string(&args[1]);
    let tcp = if mode == b"tcp" {
        true
    } else if mode == b"pipe" {
        false
    } else {
        let arg0 = "invalid mode";
        semsg!("E475: Invalid argument: {arg0}");
        return;
    };

    let mut rpc = false;
    let mut on_data = NO_READER;
    if args.get(2).is_some_and(|arg| arg.v_type() == VAR_DICT) {
        let opts = args[2].dict_ref();
        rpc = dict_get_number(opts, b"rpc") != 0;
        if !dict_get_callback(args[2].dict_shared(), b"on_data", &mut on_data.cb) {
            return;
        }
        on_data.buffered = dict_get_number(args[2].dict_ref(), b"data_buffered") != 0;
        // Buffered with no callback means "collect it on the Dict", so
        // the Dict has to be reachable from the reader.
        if on_data.buffered && !on_data.cb.is_set() {
            on_data.self_dict = args[2].dict_shared().cloned();
        }
    }

    let id = channel_connect(tcp, address, rpc, on_data, 50).unwrap_or_else(|error| {
        let error = msg_cstr(error);
        semsg!("connection failed: {error}");
        0
    });
    result.write_number(id as VarNumber);
}

/// `stdioopen({opts})` — turn this process's own stdin/stdout into a
/// channel.
pub fn f_stdioopen(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    if args[0].v_type() != VAR_DICT {
        emsg(gettext(e_invarg));
        return;
    }
    let mut on_stdin = NO_READER;
    let rpc = dict_get_number(args[0].dict_ref(), b"rpc") != 0;
    if !dict_get_callback(args[0].dict_shared(), b"on_stdin", &mut on_stdin.cb) {
        return;
    }
    // `on_print` is a global: there is only one stdio channel.
    let options = args[0].dict_shared();
    if !with_on_print(|cb| dict_get_callback(options, b"on_print", cb)) {
        return;
    }
    on_stdin.buffered = dict_get_number(args[0].dict_ref(), b"stdin_buffered") != 0;
    if on_stdin.buffered && !on_stdin.cb.is_set() {
        on_stdin.self_dict = args[0].dict_shared().cloned();
    }

    let id = channel_from_stdio(rpc, on_stdin).unwrap_or_else(|error| {
        let error = msg_cstr(error);
        semsg!("E905: Couldn't open stdio channel: {error}");
        0
    });
    result.write_number(id as VarNumber);
}
