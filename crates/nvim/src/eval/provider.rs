//! Calling out of the evaluator: provider script hosts, the job callbacks
//! they are driven by, and prompt-buffer callbacks.

#![forbid(unsafe_code)]
// The globals here keep upstream's spelling; upper-casing them is a per-module rewrite.
#![allow(non_upper_case_globals)]

use crate::eval::typval::TV_INITIAL_VALUE;
use crate::guard::Depth;
use crate::memory::ThinCString;
use crate::message_fmt::{msg_bytes, msg_cstr};
use crate::semsg;
use core::ffi::{CStr, c_char, c_int};
use core::ptr::null_mut;
use std::ffi::CString;

use crate::autocmd::state::{autocmd_bufnr, autocmd_fname, autocmd_fname_full, autocmd_match};
use crate::buffer::buf_is_prompt;
use crate::change::appended_lines_mark;
use crate::channel::{
    callback_reader_free, channel_job_running, channel_stream_type, find_channel,
};
use crate::eval::typval::{
    DictRef, ListRef, callback_free, dict_get_callback, dict_get_number, tv_list_alloc,
};
use crate::eval::userfunc::{CallStackAside, CallWith, call_func_with, current_fc_id, func_exists};
use crate::eval::vars::eval_variable;
use crate::eval::vars::{clear_local, emsg_static};
use crate::eval::{callback_call, kChannelStreamProc};
use crate::ex_cmds::check_secure;
use crate::getchar::state::got_int;
use crate::global_cell::GlobalCell;
use crate::lua::executor::nlua_is_deferred_safe;
use crate::memline::{Lines, ml_append_bytes};
use crate::message::{e_invarg, e_invchan, e_invchanjob};
use crate::option::vars::p_lpl;
use crate::runtime::script_autoload_named;
use crate::runtime::sourcing_name_copy;
use crate::runtime::state::{ETYPE_TOP, current_sctx};
use crate::types::{
    Callback, CallbackReader, CallerScope, Channel, EStack, EstackInfo, FAIL, ScriptCtx, TypVal,
    VAR_NUMBER, VAR_STRING, VarNumber, ptrdiff_t, uint64_t,
};
use crate::undo::u_clearallandblockfree;
use crate::winlayer::{Buf, Win};

pub(crate) static provider_caller_scope: GlobalCell<CallerScope> = GlobalCell::new(CallerScope {
    script_ctx: ScriptCtx::NONE,
    es_entry: EStack {
        es_lnum: 0,
        es_name: ::core::ptr::null_mut::<c_char>(),
        es_type: ETYPE_TOP,
        es_info: EstackInfo::None,
    },
    autocmd_fname: None,
    autocmd_match: None,
    autocmd_fname_full: false,
    autocmd_bufnr: 0,
    funccalp: None,
});
pub(crate) static provider_call_nesting: GlobalCell<c_int> = GlobalCell::new(0 as c_int);

/// A freshly declared typval.
const UNSET_TV: TypVal = TV_INITIAL_VALUE;

/// The top of the execution stack, which is where a provider records who
/// called it.
///
/// The innermost execution-stack frame.
fn top_estack() -> EStack {
    crate::runtime::innermost_frame()
}

/// Read the three job callbacks and the two "buffered" flags out of the
/// options dictionary, taking a reference to it for the readers. Answers
/// false -- having released whatever it did read -- when any of them is
/// unusable.
///
/// `v:_null_dict` is an empty options dictionary: it names no callback, is
/// not buffered into, and has no reference to take.
pub fn common_job_callbacks(
    options: Option<&DictRef>,
    on_stdout: &mut CallbackReader,
    on_stderr: &mut CallbackReader,
    on_exit: &mut Callback,
) -> bool {
    let ok = dict_get_callback(options, b"on_stdout", &mut on_stdout.cb)
        && dict_get_callback(options, b"on_stderr", &mut on_stderr.cb)
        && dict_get_callback(options, b"on_exit", on_exit);
    if !ok {
        // Whatever was read into the three slots before one of them failed
        // is released here.
        callback_reader_free(on_stdout);
        callback_reader_free(on_stderr);
        callback_free(on_exit);
        return false;
    }

    let dict = options.map(|options| &**options);
    on_stdout.buffered = dict_get_number(dict, b"stdout_buffered") != 0;
    on_stderr.buffered = dict_get_number(dict, b"stderr_buffered") != 0;
    // Buffered output with no callback is collected into the options
    // dictionary itself, which is why it becomes the reader's `self`.
    let own = options.map_or(null_mut(), DictRef::as_ptr);
    if on_stdout.buffered && !on_stdout.cb.is_set() {
        on_stdout.self_0 = own;
    }
    if on_stderr.buffered && !on_stderr.cb.is_set() {
        on_stderr.self_0 = own;
    }
    // The reference the readers now share, given up to them: nothing here
    // releases it.
    core::mem::forget(options.cloned());
    true
}

/// The channel a job id names, or null.
pub fn find_job(id: uint64_t, show_error: bool) -> *mut Channel {
    if channel_job_running(id) {
        return find_channel(id);
    }
    if show_error {
        // A channel that exists but is not a job gets its own message.
        let wrong_kind = channel_stream_type(id).is_some_and(|kind| kind != kChannelStreamProc);
        emsg_static(if wrong_kind { e_invchanjob } else { e_invchan });
    }
    null_mut()
}

/// `py3eval()` and its relatives: hand one expression to a script host.
pub fn script_host_eval(name: &CStr, args: &[TypVal], result: &mut TypVal) {
    if check_secure() {
        return;
    }
    let arg = &args[0];
    if arg.v_type() != VAR_STRING {
        emsg_static(e_invarg);
        return;
    }
    let list = tv_list_alloc(1 as ptrdiff_t);
    list.edit().push(TypVal::string(arg.string_ref().cloned()));
    *result = eval_call_provider(name, c"eval", Some(list), false);
}

/// Call `provider#<name>#Call(method, arguments)`.
///
/// The caller's scope — script context, execution-stack entry, the
/// autocommand variables and the function-call frame — is stashed in
/// `provider_caller_scope` first, because the provider runs Vimscript that
/// may ask about any of it.
///
/// The argument list is handed over: the array holds it for the length of
/// the call and releases it afterwards.
pub fn eval_call_provider(
    provider: &CStr,
    method: &CStr,
    arguments: Option<ListRef>,
    discard: bool,
) -> TypVal {
    if !eval_has_provider(provider, false) {
        let provider = msg_cstr(provider);
        semsg!("E319: No \"{provider}\" provider found. Run \":checkhealth vim.provider\"");
        return TypVal::Number(0);
    }

    // Upstream renders the name into 256 bytes; no known provider comes
    // near that.
    let func = provider_name(provider.to_bytes(), b"Call");

    let scope = CallerScope {
        script_ctx: current_sctx.get(),
        es_entry: top_estack(),
        autocmd_fname: autocmd_fname.with(Clone::clone),
        autocmd_match: autocmd_match.with(Clone::clone),
        autocmd_fname_full: autocmd_fname_full.get(),
        autocmd_bufnr: autocmd_bufnr.get(),
        funccalp: current_fc_id(),
    };
    let saved_provider_caller_scope =
        provider_caller_scope.with_mut(|current| core::mem::replace(current, scope));
    let call_stack_aside = CallStackAside::new();
    let nesting = Depth::of(&provider_call_nesting);

    // The argument array holds the two values, so the caller's reference is
    // given back when the array drops -- which is why the method name is
    // duplicated rather than borrowed.
    let argvars = [
        TypVal::string(Some(ThinCString::from_cstr(method))),
        TypVal::list(arguments),
    ];
    let mut rettv = UNSET_TV;

    let _ = call_func_with(&func, None, &mut rettv, &argvars, CallWith::at_cursor(true));
    drop(argvars);
    drop(call_stack_aside);
    provider_caller_scope.set(saved_provider_caller_scope);
    drop(nesting);
    debug_assert!(provider_call_nesting.get() >= 0);

    if discard {
        clear_local(&mut rettv);
    }
    rettv
}

/// `provider#<name>#<what>`.
fn provider_name(name: &[u8], what: &[u8]) -> CString {
    let text = [b"provider#".as_slice(), name, b"#", what].concat();
    CString::new(text).unwrap_or_default()
}

/// Is this provider both known and usable? Loads its autoload script if it
/// has not been loaded yet.
pub fn eval_has_provider(feat: &CStr, throw_if_fast: bool) -> bool {
    const KNOWN: [&[u8]; 7] = [
        b"clipboard",
        b"python3",
        b"python3_compiled",
        b"python3_dynamic",
        b"perl",
        b"ruby",
        b"node",
    ];
    let feat = feat.to_bytes();
    if !KNOWN.contains(&feat) {
        return false;
    }
    if throw_if_fast && !nlua_is_deferred_safe() {
        let what = "Vimscript function";
        semsg!("E5560: {what} must not be called in a fast event context");
        return false;
    }

    // The variable and function names use the part before the first
    // `_`: "python3_dynamic" asks about "python3".
    let name = feat.split(|&b| b == b'_').next().unwrap_or(feat);
    let loaded = [b"g:loaded_".as_slice(), name, b"_provider"].concat();
    let call = provider_name(name, b"Call");
    let mut tv = UNSET_TV;

    if eval_variable(&loaded, Some(&mut tv), false, true).is_err() {
        // Not loaded yet: sourcing any function in the provider's
        // autoload namespace is what pulls the script in.
        script_autoload_named(provider_name(name, b"bogus").as_bytes(), false);
        if eval_variable(&loaded, Some(&mut tv), false, true).is_err() {
            if func_exists(call.as_bytes()) && p_lpl() {
                let nm = msg_bytes(name);
                semsg!("provider: {nm}: missing required variable g:loaded_{nm}_provider");
            }
            return false;
        }
    }

    // 2 is the "working" value; 1 means the provider declined.
    let mut ok = tv.v_type() == VAR_NUMBER && tv.number_or_zero() == 2 as VarNumber;
    if ok && !func_exists(call.as_bytes()) {
        let (nm, call) = (msg_bytes(name), msg_cstr(&call));
        semsg!("provider: {nm}: g:loaded_{nm}_provider=2 but {call} is not defined");
        ok = false;
    }
    ok
}

/// `"<script>:<line>"` for the innermost execution-stack entry, or `"?"`
/// when nothing is executing.
pub fn eval_source_name_line() -> CString {
    let Some(name) = sourcing_name_copy() else {
        return c"?".to_owned();
    };
    let mut text = name;
    text.extend_from_slice(format!(":{}", top_estack().es_lnum).as_bytes());
    CString::new(text).unwrap_or_default()
}

/// Everything the user typed into a prompt buffer since the prompt, as one
/// newline-joined string.
pub fn prompt_get_input(buffer: Option<Buf>) -> Option<ThinCString> {
    let buffer = buffer?;
    if !buf_is_prompt(Some(buffer)) {
        return None;
    }
    let lnum_start = buffer.b_prompt_start.mark.lnum;
    let lnum_last = buffer.line_count();
    let col = buffer.b_prompt_start.mark.col;

    let mut lines = Lines::in_buffer(buffer);
    let first = lines.line(lnum_start);
    // The prompt itself is skipped, unless the line is shorter than the
    // recorded column.
    let skip = usize::try_from(col).ok().filter(|&col| col <= first.len());
    let mut text = first[skip.unwrap_or(0)..].to_vec();
    for lnum in (lnum_start + 1)..=lnum_last {
        text.push(b'\n');
        text.extend_from_slice(lines.line(lnum));
    }
    Some(ThinCString::from_vec(text))
}

/// The user pressed Enter in a prompt buffer: open the next line and hand
/// what was typed to the buffer's callback.
pub fn prompt_invoke_callback() {
    let lnum = Buf::current().line_count();
    let Some(user_input) = prompt_get_input(Buf::current_or_none()) else {
        return;
    };

    let _ = ml_append_bytes(Buf::current(), lnum, b"");
    appended_lines_mark(lnum, 1);
    Win::current().w_cursor.lnum = lnum + 1;
    Win::current().w_cursor.col = 0;
    Buf::current().b_prompt_start.mark.lnum = lnum + 1;

    if !Buf::current().b_prompt_callback.is_set() {
        drop(user_input);
    } else {
        let mut rettv = UNSET_TV;
        // The array takes the input over and frees it.
        let argv = [TypVal::string(Some(user_input))];
        // A copy: the callback may replace itself, or wipe the buffer.
        let callback = Buf::current().b_prompt_callback.duplicate();
        callback_call(&callback, &argv, &mut rettv);
        drop(argv);
        clear_local(&mut rettv);
    }

    u_clearallandblockfree(Buf::current());
    Buf::current().b_prompt_start.mark.lnum = Buf::current().line_count();
    Buf::current().b_prompt_append_new_line = true;
}

/// CTRL-C in a prompt buffer. Answers whether the buffer had an interrupt
/// callback at all.
pub fn invoke_prompt_interrupt() -> bool {
    if !Buf::current().b_prompt_interrupt.is_set() {
        return false;
    }
    let mut rettv = UNSET_TV;
    // The interrupt is consumed here; the callback decides what to do
    // about it.
    got_int.set(false);
    // A copy: the callback may replace itself, or wipe the buffer.
    let callback = Buf::current().b_prompt_interrupt.duplicate();
    let ret = callback_call(&callback, &[], &mut rettv);
    clear_local(&mut rettv);
    ret as c_int != FAIL
}
