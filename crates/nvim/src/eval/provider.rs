//! Calling out of the evaluator: provider script hosts, the job callbacks
//! they are driven by, and prompt-buffer callbacks.

#![deny(unsafe_op_in_unsafe_fn)]

use crate::message_fmt::c_str;
use crate::semsg;
use core::ffi::{CStr, c_char, c_int, c_void};
use core::mem::{offset_of, size_of};
use core::ptr::null_mut;

use crate::buffer::buf_is_prompt;
use crate::change::appended_lines_mark;
use crate::channel::{callback_reader_free, channel_proc, find_channel};
use crate::eval::typval::{
    callback_free, kCallbackNone, tv_clear, tv_dict_get_callback, tv_dict_get_number,
    tv_list_alloc, tv_list_append_string, tv_list_ref, tv_list_unref,
};
use crate::eval::userfunc::{
    call_func, find_func, get_current_funccal, restore_funccal, save_funccal,
};
use crate::eval::vars::eval_variable;
use crate::eval::vars::{clear_local, emsg_static};
use crate::eval::window::{cur_buf, cur_win};
use crate::eval::{FUNCEXE_INIT, Tv, callback_call, kChannelStreamProc};
use crate::event::proc::proc_is_stopped;
use crate::ex_cmds::check_secure;
use crate::lua::executor::nlua_is_deferred_safe;
use crate::main::{
    autocmd_bufnr, autocmd_fname, autocmd_fname_full, autocmd_match, current_sctx, e_invarg,
    e_invchan, e_invchanjob, got_int, p_lpl, provider_call_nesting, provider_caller_scope,
};
use crate::memline::{ml_append, ml_get_buf};
use crate::memory::{strchrsub, strequal, xfree, xstrdup};
use crate::os::cshim::snprintf;
use crate::runtime::script_autoload;
use crate::strings::concat_str;
use crate::types::{
    Callback, CallbackReader, Channel, FAIL, NUL, VAR_LIST, VAR_NUMBER, VAR_STRING, VAR_UNKNOWN,
    VarLock, buf_T, caller_scope, colnr_T, dict_T, estack_T, funccal_entry_T, funcexe_T, list_T,
    ptrdiff_t, size_t, ssize_t, typval_T, typval_vval_union, uint64_t, varnumber_T,
};
use crate::undo::u_clearallandblockfree;
use crate::winlayer::{Buf, Live};
use ::libc::strlen;

/// A freshly declared typval.
const UNSET_TV: typval_T = typval_T {
    v_type: VAR_UNKNOWN,
    v_lock: VarLock::Unlocked,
    vval: typval_vval_union { v_number: 0 },
};

/// The scratch a provider function name is rendered into.
const NAMEBUF: usize = 256;

/// A job's output reader, whose caller has promised it outlives the value.
type Reader = Live<CallbackReader>;

/// The top of the execution stack, which is where a provider records who
/// called it.
///
/// The innermost execution-stack frame.
fn top_estack() -> estack_T {
    crate::runtime::innermost_frame()
}

/// Read the three job callbacks and the two "buffered" flags out of the
/// options Dict, taking a reference to it. Answers false — having released
/// whatever it did read — when any of them is unusable.
///
/// # Safety
/// All four pointers must be valid.
pub unsafe fn common_job_callbacks(
    vopts: *mut dict_T,
    on_stdout: *mut CallbackReader,
    on_stderr: *mut CallbackReader,
    on_exit: *mut Callback,
) -> bool {
    // SAFETY: the caller's promise -- both readers outlive the call.
    let (mut out, mut err) = unsafe { (Reader::new(on_stdout), Reader::new(on_stderr)) };
    let out_cb: *mut Callback = out.field_ptr(offset_of!(CallbackReader, cb));
    let err_cb: *mut Callback = err.field_ptr(offset_of!(CallbackReader, cb));
    // SAFETY: the caller's promise -- a live Dict and three callback slots,
    // two of which are the readers' own.
    let ok = unsafe { job_callback(vopts, c"on_stdout", out_cb) }
        && unsafe { job_callback(vopts, c"on_stderr", err_cb) }
        && unsafe { job_callback(vopts, c"on_exit", on_exit) };
    if !ok {
        // SAFETY: as above; whatever was read into the three slots before
        // one of them failed is released here.
        unsafe { callback_reader_free(on_stdout) };
        // SAFETY: as above.
        unsafe { callback_reader_free(on_stderr) };
        // SAFETY: as above.
        unsafe { callback_free(on_exit) };
        return false;
    }

    // SAFETY: the caller's promise -- `vopts` is a live Dict.
    out.buffered = unsafe { tv_dict_get_number(vopts, c"stdout_buffered".as_ptr()) } != 0;
    // SAFETY: as above.
    err.buffered = unsafe { tv_dict_get_number(vopts, c"stderr_buffered".as_ptr()) } != 0;
    // Buffered output with no callback is collected into the options
    // Dict itself, which is why it becomes the reader's `self`.
    if out.buffered && out.cb.type_0 == kCallbackNone {
        out.self_0 = vopts;
    }
    if err.buffered && err.cb.type_0 == kCallbackNone {
        err.self_0 = vopts;
    }
    // SAFETY: as above; this is the reference the readers now share.
    unsafe { (*vopts).dv_refcount.retain() };
    true
}

/// One `on_*` callback out of the options Dict, by name.
///
/// # Safety
/// `vopts` must be a live Dict and `into` a valid callback slot.
unsafe fn job_callback(vopts: *mut dict_T, key: &CStr, into: *mut Callback) -> bool {
    let len = key.count_bytes() as ptrdiff_t;
    // SAFETY: the caller's promise; `key` is a NUL-terminated literal of
    // `len` bytes.
    unsafe { tv_dict_get_callback(vopts, key.as_ptr(), len, into) }
}

/// The channel a job id names, or null.
///
/// # Safety
/// Called with the channel table initialised.
pub unsafe fn find_job(id: uint64_t, show_error: bool) -> *mut Channel {
    let data = find_channel(id);
    // SAFETY: a non-null channel is live, and a proc channel has a proc.
    let running = !data.is_null()
        && unsafe { (*data).streamtype } == kChannelStreamProc
        && !unsafe { proc_is_stopped(&*channel_proc(data)) };
    if running {
        return data;
    }
    if show_error {
        // A channel that exists but is not a job gets its own message.
        // SAFETY: a non-null channel is live.
        let wrong_kind = !data.is_null() && unsafe { (*data).streamtype } != kChannelStreamProc;
        if wrong_kind {
            // SAFETY: a shared NUL-terminated message.
            emsg_static(e_invchanjob);
        } else {
            // SAFETY: as above.
            emsg_static(e_invchan);
        }
    }
    null_mut()
}

/// `py3eval()` and its relatives: hand one expression to a script host.
///
/// # Safety
/// `name` must be NUL-terminated; `argvars` and `rettv` valid.
pub unsafe fn script_host_eval(name: *mut c_char, argvars: *mut typval_T, rettv: *mut typval_T) {
    if check_secure() {
        return;
    }
    // SAFETY: the caller's promise -- both typvals outlive the call.
    let (arg, mut ret) = unsafe { (Tv::new(argvars), Tv::new(rettv)) };
    if arg.v_type != VAR_STRING {
        // SAFETY: `e_invarg` is a shared NUL-terminated message.
        emsg_static(e_invarg);
        return;
    }
    // SAFETY: the List is fresh and this frame's.
    let args: *mut list_T = unsafe { tv_list_alloc(1 as ptrdiff_t) };
    // SAFETY: `VAR_STRING` says `v_string` is the union's live member, and
    // -1 asks the callee to measure it.
    unsafe { tv_list_append_string(args, arg.vval.v_string, -1 as ssize_t) };
    let method = c"eval".as_ptr() as *mut c_char;
    // SAFETY: `name` and `method` are NUL-terminated and `args` is live.
    *ret = unsafe { eval_call_provider(name, method, args, false) };
}

/// Call `provider#<name>#Call(method, arguments)`.
///
/// The caller's scope — script context, execution-stack entry, the
/// autocommand variables and the function-call frame — is stashed in
/// `provider_caller_scope` first, because the provider runs Vimscript that
/// may ask about any of it.
///
/// # Safety
/// `provider` and `method` must be NUL-terminated; `arguments` valid.
pub unsafe fn eval_call_provider(
    provider: *mut c_char,
    method: *mut c_char,
    arguments: *mut list_T,
    discard: bool,
) -> typval_T {
    // SAFETY: the caller's promise -- `provider` is NUL-terminated.
    if !unsafe { eval_has_provider(provider, false) } {
        // SAFETY: the format takes one NUL-terminated string.
        let provider = unsafe { c_str(provider) };
        semsg!("E319: No \"{provider}\" provider found. Run \":checkhealth vim.provider\"");
        return typval_T {
            v_type: VAR_NUMBER,
            v_lock: VarLock::Unlocked,
            vval: typval_vval_union { v_number: 0 },
        };
    }

    let mut func: [c_char; NAMEBUF] = [0; NAMEBUF];
    let size = size_of::<[c_char; NAMEBUF]>();
    let fmt = c"provider#%s#Call".as_ptr();
    // SAFETY: `func` is this frame's and `size` is its length; the format
    // takes the one NUL-terminated string `provider`.
    let name_len = unsafe { snprintf(func.as_mut_ptr(), size, fmt, provider) };

    let saved_provider_caller_scope = provider_caller_scope.get();
    // SAFETY: the frame pointer is only stashed, never read through here.
    let funccalp = unsafe { get_current_funccal() } as *mut c_void;
    provider_caller_scope.set(caller_scope {
        script_ctx: current_sctx.get(),
        es_entry: top_estack(),
        autocmd_fname: autocmd_fname.get(),
        autocmd_match: autocmd_match.get(),
        autocmd_fname_full: autocmd_fname_full.get(),
        autocmd_bufnr: autocmd_bufnr.get(),
        funccalp,
    });
    let mut funccal_entry = funccal_entry_T {
        top_funccal: null_mut(),
        next: null_mut(),
    };
    // SAFETY: `funccal_entry` is this frame's and outlives the save.
    unsafe { save_funccal(&raw mut funccal_entry) };
    provider_call_nesting.set(provider_call_nesting.get() + 1);

    let mut argvars: [typval_T; 3] = [
        typval_T {
            v_type: VAR_STRING,
            v_lock: VarLock::Unlocked,
            vval: typval_vval_union { v_string: method },
        },
        typval_T {
            v_type: VAR_LIST,
            v_lock: VarLock::Unlocked,
            vval: typval_vval_union { v_list: arguments },
        },
        UNSET_TV,
    ];
    let mut rettv = UNSET_TV;
    // The argument array borrows the List, so the reference is taken
    // for the duration of the call and given back after it.
    // SAFETY: the caller's promise -- `arguments` is a live List.
    unsafe { tv_list_ref(arguments) };

    let mut funcexe: funcexe_T = FUNCEXE_INIT;
    funcexe.fe_firstline = cur_win().w_cursor.lnum;
    funcexe.fe_lastline = cur_win().w_cursor.lnum;
    funcexe.fe_evaluate = true;
    let (name, args) = (func.as_mut_ptr(), argvars.as_mut_ptr());
    // SAFETY: `name` is the NUL-terminated name rendered above, `args` the
    // two argument typvals, and `rettv` and `funcexe` are this frame's.
    let _ = unsafe { call_func(name, name_len, &raw mut rettv, 2, args, &raw mut funcexe) };

    // SAFETY: this gives back the reference taken above.
    unsafe { tv_list_unref(arguments) };
    // SAFETY: this undoes the save above.
    unsafe { restore_funccal() };
    provider_caller_scope.set(saved_provider_caller_scope);
    provider_call_nesting.set(provider_call_nesting.get() - 1);
    debug_assert!(provider_call_nesting.get() >= 0);

    if discard {
        // SAFETY: `rettv` is this frame's.
        clear_local(&mut rettv);
    }
    rettv
}

/// `g:loaded_<name>_provider`, into `buf`.
///
/// # Safety
/// Both buffers must be valid, and `buf` must hold `NAMEBUF` bytes.
unsafe fn loaded_var(buf: *mut c_char, name: *mut c_char) -> c_int {
    let fmt = c"g:loaded_%s_provider".as_ptr();
    // SAFETY: the caller's promise about both buffers.
    unsafe { snprintf(buf, size_of::<[c_char; NAMEBUF]>(), fmt, name) }
}

/// `provider#<name>#<what>`, into `buf`.
///
/// # Safety
/// As [`loaded_var`].
unsafe fn provider_fn(buf: *mut c_char, name: *mut c_char, what: &CStr) -> c_int {
    // SAFETY: the caller's promise about both buffers; `what` is a
    // NUL-terminated literal.
    unsafe { snprintf(buf, size_of::<[c_char; NAMEBUF]>(), what.as_ptr(), name) }
}

/// Is this provider both known and usable? Loads its autoload script if it
/// has not been loaded yet.
///
/// # Safety
/// `feat` must be NUL-terminated.
pub unsafe fn eval_has_provider(feat: *const c_char, throw_if_fast: bool) -> bool {
    const KNOWN: [&CStr; 7] = [
        c"clipboard",
        c"python3",
        c"python3_compiled",
        c"python3_dynamic",
        c"perl",
        c"ruby",
        c"node",
    ];
    // SAFETY: the caller's promise -- `feat` is NUL-terminated, as is every
    // name it is compared with.
    if !KNOWN.iter().any(|k| unsafe { strequal(feat, k.as_ptr()) }) {
        return false;
    }
    // SAFETY: the check only reads the Lua scheduler's state.
    if throw_if_fast && !unsafe { nlua_is_deferred_safe() } {
        let what = c"Vimscript function".as_ptr();
        // SAFETY: the format takes one NUL-terminated string.
        let what = unsafe { c_str(what) };
        semsg!("E5560: {what} must not be called in a fast event context");
        return false;
    }

    // The variable and function names use the part before the first
    // `_`: "python3_dynamic" asks about "python3".
    let mut name: [c_char; 32] = [0; 32];
    let size = size_of::<[c_char; 32]>();
    // SAFETY: `name` is this frame's and `size` its length; the format
    // takes the one NUL-terminated string `feat`.
    unsafe { snprintf(name.as_mut_ptr(), size, c"%s".as_ptr(), feat) };
    // SAFETY: `name` now holds a NUL-terminated copy of `feat`.
    unsafe { strchrsub(name.as_mut_ptr(), b'_' as c_char, NUL as c_char) };

    let mut buf: [c_char; NAMEBUF] = [0; NAMEBUF];
    let mut tv = UNSET_TV;
    let (nm, bp) = (name.as_mut_ptr(), buf.as_mut_ptr());

    // SAFETY (every call below): `bp` names this frame's `NAMEBUF` bytes,
    // `nm` the NUL-terminated provider name, and `tv` is this frame's.
    let mut len = unsafe { loaded_var(bp, nm) };
    if unsafe { eval_variable(bp, len, &raw mut tv, null_mut(), false, true) }.is_err() {
        // Not loaded yet: sourcing any function in the provider's
        // autoload namespace is what pulls the script in.
        len = unsafe { provider_fn(bp, nm, c"provider#%s#bogus") };
        unsafe { script_autoload(bp, len as size_t, false) };

        len = unsafe { loaded_var(bp, nm) };
        if unsafe { eval_variable(bp, len, &raw mut tv, null_mut(), false, true) }.is_err() {
            unsafe { provider_fn(bp, nm, c"provider#%s#Call") };
            // SAFETY: `bp` holds the NUL-terminated function name.
            let defined = !unsafe { find_func(bp) }.is_null();
            if defined && p_lpl.get() != 0 {
                // SAFETY: the format takes two NUL-terminated strings.
                let (nm2, nm) = unsafe { (c_str(nm), c_str(nm)) };
                semsg!("provider: {nm2}: missing required variable g:loaded_{nm}_provider");
            }
            return false;
        }
    }

    // 2 is the "working" value; 1 means the provider declined.
    // SAFETY: `VAR_NUMBER` says `v_number` is the union's live member.
    let mut ok = tv.v_type == VAR_NUMBER && unsafe { tv.vval.v_number } == 2 as varnumber_T;
    if ok {
        // SAFETY: as above.
        unsafe { provider_fn(bp, nm, c"provider#%s#Call") };
        // SAFETY: `bp` holds the NUL-terminated function name just built.
        if unsafe { find_func(bp) }.is_null() {
            // SAFETY: the format takes three NUL-terminated strings.
            let (nm2, nm, bp) = unsafe { (c_str(nm), c_str(nm), c_str(bp)) };
            semsg!("provider: {nm2}: g:loaded_{nm}_provider=2 but {bp} is not defined");
            ok = false;
        }
    }
    ok
}

/// `"<script>:<line>"` for the innermost execution-stack entry.
///
/// # Safety
/// `buf` must hold `bufsize` writable bytes.
pub unsafe fn eval_fmt_source_name_line(buf: *mut c_char, bufsize: size_t) {
    let top = top_estack();
    if top.es_name.is_null() {
        // SAFETY: the caller's promise about `buf` and `bufsize`.
        unsafe { snprintf(buf, bufsize, c"?".as_ptr()) };
    } else {
        // SAFETY: as above; the entry's name is NUL-terminated.
        unsafe { snprintf(buf, bufsize, c"%s:%d".as_ptr(), top.es_name, top.es_lnum) };
    }
}

/// Everything the user typed into a prompt buffer since the prompt, as one
/// newline-joined string.
///
/// # Safety
/// `buf` must be valid.
pub unsafe fn prompt_get_input(buf: *mut buf_T) -> *mut c_char {
    // SAFETY: the caller's promise -- a live buffer.
    let Some(buf) = (unsafe { Buf::from_raw(buf) }) else {
        return null_mut();
    };
    if !buf_is_prompt(Some(buf)) {
        return null_mut();
    }
    let lnum_start = buf.b_prompt_start.mark.lnum;
    let lnum_last = buf.line_count();

    // SAFETY: the prompt's line is a line of the buffer.
    let mut text = unsafe { ml_get_buf(buf.raw(), lnum_start) };
    // The prompt itself is skipped, unless the line is shorter than
    // the recorded column.
    let col = buf.b_prompt_start.mark.col;
    // SAFETY: a buffer line is NUL-terminated.
    if unsafe { strlen(text) } as c_int >= col {
        // SAFETY: `col` is inside the line, measured just above.
        text = unsafe { text.offset(col as isize) };
    }
    // SAFETY: `text` is NUL-terminated.
    let mut full_text = unsafe { xstrdup(text) };
    for i in (lnum_start + 1)..=lnum_last {
        // SAFETY: `full_text` is owned and NUL-terminated, `i` is a line of
        // the buffer, and each join frees what it consumed.
        let half_text = unsafe { concat_str(full_text, c"\n".as_ptr()) };
        // SAFETY: the join copied what it needed.
        unsafe { xfree(full_text as *mut c_void) };
        // SAFETY: as above.
        full_text = unsafe { concat_str(half_text, ml_get_buf(buf.raw(), i)) };
        // SAFETY: as above.
        unsafe { xfree(half_text as *mut c_void) };
    }
    full_text
}

/// The user pressed Enter in a prompt buffer: open the next line and hand
/// what was typed to the buffer's callback.
///
/// # Safety
/// Called from the prompt-buffer key handling, with a prompt buffer
/// current.
pub unsafe fn prompt_invoke_callback() {
    let lnum = cur_buf().line_count();
    // SAFETY: the current buffer is live.
    let user_input = unsafe { prompt_get_input(cur_buf().raw()) };
    if user_input.is_null() {
        return;
    }

    // SAFETY: `lnum` is the buffer's last line, and the literal is
    // NUL-terminated.
    let _ = unsafe { ml_append(lnum, c"".as_ptr() as *mut c_char, 0 as colnr_T, false) };
    // SAFETY: the line was just appended.
    unsafe { appended_lines_mark(lnum, 1) };
    cur_win().w_cursor.lnum = lnum + 1;
    cur_win().w_cursor.col = 0;
    cur_buf().b_prompt_start.mark.lnum = lnum + 1;

    if cur_buf().b_prompt_callback.type_0 == kCallbackNone {
        // SAFETY: nothing took the input over.
        unsafe { xfree(user_input as *mut c_void) };
    } else {
        let mut rettv = UNSET_TV;
        let mut argv = [UNSET_TV; 2];
        argv[0].v_type = VAR_STRING;
        argv[0].vval.v_string = user_input;
        argv[1].v_type = VAR_UNKNOWN;
        // SAFETY: the callback is the current buffer's own, and the
        // argument array and result are this frame's.
        let cb = unsafe { &raw mut (*cur_buf().raw()).b_prompt_callback };
        // SAFETY: as above.
        unsafe { callback_call(cb, 1, argv.as_mut_ptr(), &raw mut rettv) };
        // SAFETY: the argument array and the result are this frame's.
        unsafe { tv_clear(argv.as_mut_ptr()) };
        // SAFETY: as above.
        clear_local(&mut rettv);
    }

    // SAFETY: the current buffer is live.
    unsafe { u_clearallandblockfree(Buf::current()) };
    cur_buf().b_prompt_start.mark.lnum = cur_buf().line_count();
    cur_buf().b_prompt_append_new_line = true;
}

/// CTRL-C in a prompt buffer. Answers whether the buffer had an interrupt
/// callback at all.
///
/// # Safety
/// As `prompt_invoke_callback`.
pub unsafe fn invoke_prompt_interrupt() -> bool {
    if cur_buf().b_prompt_interrupt.type_0 == kCallbackNone {
        return false;
    }
    let mut rettv = UNSET_TV;
    let mut argv = [UNSET_TV; 1];
    argv[0].v_type = VAR_UNKNOWN;
    // The interrupt is consumed here; the callback decides what to do
    // about it.
    got_int.set(false);
    // SAFETY: the callback is the current buffer's own, and the argument
    // array and result are this frame's.
    let cb = unsafe { &raw mut (*cur_buf().raw()).b_prompt_interrupt };
    // SAFETY: as above.
    let ret = unsafe { callback_call(cb, 0, argv.as_mut_ptr(), &raw mut rettv) };
    // SAFETY: `rettv` is this frame's.
    clear_local(&mut rettv);
    ret as c_int != FAIL
}
