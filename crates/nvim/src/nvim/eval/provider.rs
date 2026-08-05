//! Calling out of the evaluator: provider script hosts, the job callbacks
//! they are driven by, and prompt-buffer callbacks.

#![deny(unsafe_op_in_unsafe_fn)]

use core::ffi::{c_char, c_int, c_void};
use core::mem::size_of;
use core::ptr::null_mut;

use crate::src::nvim::buffer::bt_prompt;
use crate::src::nvim::change::appended_lines_mark;
use crate::src::nvim::channel::{callback_reader_free, channel_proc, find_channel};
use crate::src::nvim::eval::typval::{
    callback_free, kCallbackNone, tv_clear, tv_dict_get_callback, tv_dict_get_number,
    tv_list_alloc, tv_list_append_string, tv_list_ref, tv_list_unref,
};
use crate::src::nvim::eval::userfunc::{
    call_func, find_func, get_current_funccal, restore_funccal, save_funccal,
};
use crate::src::nvim::eval::vars::eval_variable;
use crate::src::nvim::eval::{FAIL, FUNCEXE_INIT, NUL, callback_call, kChannelStreamProc};
use crate::src::nvim::event::proc::proc_is_stopped;
use crate::src::nvim::ex_cmds::check_secure;
use crate::src::nvim::lua::executor::nlua_is_deferred_safe;
use crate::src::nvim::main::{
    autocmd_bufnr, autocmd_fname, autocmd_fname_full, autocmd_match, curbuf, current_sctx, curwin,
    e_fast_api_disabled, e_invarg, e_invchan, e_invchanjob, got_int, p_lpl, provider_call_nesting,
    provider_caller_scope,
};
use crate::src::nvim::memline::{ml_append, ml_get_buf};
use crate::src::nvim::memory::{strchrsub, strequal, xfree, xstrdup};
use crate::src::nvim::message::{emsg, semsg};
use crate::src::nvim::os::libc::{gettext, snprintf, strlen};
use crate::src::nvim::runtime::{exestack, script_autoload};
use crate::src::nvim::strings::concat_str;
use crate::src::nvim::types::{
    Callback, CallbackReader, Channel, VAR_LIST, VAR_NUMBER, VAR_STRING, VAR_UNKNOWN, VAR_UNLOCKED,
    buf_T, caller_scope, colnr_T, dict_T, estack_T, funccal_entry_T, funcexe_T, list_T, ptrdiff_t,
    size_t, ssize_t, typval_T, typval_vval_union, uint64_t, varnumber_T,
};
use crate::src::nvim::undo::u_clearallandblockfree;

/// A freshly declared typval.
const UNSET_TV: typval_T = typval_T {
    v_type: VAR_UNKNOWN,
    v_lock: VAR_UNLOCKED,
    vval: typval_vval_union { v_number: 0 },
};

/// The scratch a provider function name is rendered into.
const NAMEBUF: usize = 256;

/// The top of the execution stack, which is where a provider records who
/// called it.
///
/// # Safety
/// The stack always holds at least one entry.
unsafe fn top_estack() -> *mut estack_T {
    unsafe {
        let stack = exestack.ptr();
        ((*stack).ga_data as *mut estack_T).offset(((*stack).ga_len - 1) as isize)
    }
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
    unsafe {
        let ok = tv_dict_get_callback(
            vopts,
            c"on_stdout".as_ptr(),
            c"on_stdout".count_bytes() as ptrdiff_t,
            &raw mut (*on_stdout).cb,
        ) && tv_dict_get_callback(
            vopts,
            c"on_stderr".as_ptr(),
            c"on_stderr".count_bytes() as ptrdiff_t,
            &raw mut (*on_stderr).cb,
        ) && tv_dict_get_callback(
            vopts,
            c"on_exit".as_ptr(),
            c"on_exit".count_bytes() as ptrdiff_t,
            on_exit,
        );
        if !ok {
            callback_reader_free(on_stdout);
            callback_reader_free(on_stderr);
            callback_free(on_exit);
            return false;
        }

        (*on_stdout).buffered = tv_dict_get_number(vopts, c"stdout_buffered".as_ptr()) != 0;
        (*on_stderr).buffered = tv_dict_get_number(vopts, c"stderr_buffered".as_ptr()) != 0;
        // Buffered output with no callback is collected into the options
        // Dict itself, which is why it becomes the reader's `self`.
        if (*on_stdout).buffered && (*on_stdout).cb.type_0 == kCallbackNone {
            (*on_stdout).self_0 = vopts;
        }
        if (*on_stderr).buffered && (*on_stderr).cb.type_0 == kCallbackNone {
            (*on_stderr).self_0 = vopts;
        }
        (*vopts).dv_refcount += 1;
        true
    }
}

/// The channel a job id names, or null.
///
/// # Safety
/// Called with the channel table initialised.
pub unsafe fn find_job(id: uint64_t, show_error: bool) -> *mut Channel {
    unsafe {
        let data = find_channel(id);
        if !data.is_null()
            && (*data).streamtype == kChannelStreamProc
            && !proc_is_stopped(&*channel_proc(data))
        {
            return data;
        }
        if show_error {
            // A channel that exists but is not a job gets its own message.
            if !data.is_null() && (*data).streamtype != kChannelStreamProc {
                emsg(gettext(e_invchanjob.ptr().cast()));
            } else {
                emsg(gettext(e_invchan.ptr().cast()));
            }
        }
        null_mut()
    }
}

/// `py3eval()` and its relatives: hand one expression to a script host.
///
/// # Safety
/// `name` must be NUL-terminated; `argvars` and `rettv` valid.
pub unsafe fn script_host_eval(name: *mut c_char, argvars: *mut typval_T, rettv: *mut typval_T) {
    unsafe {
        if check_secure() {
            return;
        }
        if (*argvars).v_type != VAR_STRING {
            emsg(gettext(e_invarg.ptr().cast()));
            return;
        }
        let args: *mut list_T = tv_list_alloc(1 as ptrdiff_t);
        tv_list_append_string(args, (*argvars).vval.v_string, -1 as ssize_t);
        *rettv = eval_call_provider(name, c"eval".as_ptr() as *mut c_char, args, false);
    }
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
    unsafe {
        if !eval_has_provider(provider, false) {
            semsg(
                c"E319: No \"%s\" provider found. Run \":checkhealth vim.provider\"".as_ptr(),
                provider,
            );
            return typval_T {
                v_type: VAR_NUMBER,
                v_lock: VAR_UNLOCKED,
                vval: typval_vval_union { v_number: 0 },
            };
        }

        let mut func: [c_char; NAMEBUF] = [0; NAMEBUF];
        let name_len = snprintf(
            func.as_mut_ptr(),
            size_of::<[c_char; NAMEBUF]>(),
            c"provider#%s#Call".as_ptr(),
            provider,
        );

        let saved_provider_caller_scope = provider_caller_scope.get();
        provider_caller_scope.set(caller_scope {
            script_ctx: current_sctx.get(),
            es_entry: *top_estack(),
            autocmd_fname: autocmd_fname.get(),
            autocmd_match: autocmd_match.get(),
            autocmd_fname_full: autocmd_fname_full.get(),
            autocmd_bufnr: autocmd_bufnr.get(),
            funccalp: get_current_funccal() as *mut c_void,
        });
        let mut funccal_entry = funccal_entry_T {
            top_funccal: null_mut(),
            next: null_mut(),
        };
        save_funccal(&raw mut funccal_entry);
        *provider_call_nesting.ptr() += 1;

        let mut argvars: [typval_T; 3] = [
            typval_T {
                v_type: VAR_STRING,
                v_lock: VAR_UNLOCKED,
                vval: typval_vval_union { v_string: method },
            },
            typval_T {
                v_type: VAR_LIST,
                v_lock: VAR_UNLOCKED,
                vval: typval_vval_union { v_list: arguments },
            },
            UNSET_TV,
        ];
        let mut rettv = UNSET_TV;
        // The argument array borrows the List, so the reference is taken
        // for the duration of the call and given back after it.
        tv_list_ref(arguments);

        let mut funcexe: funcexe_T = FUNCEXE_INIT;
        funcexe.fe_firstline = (*curwin.get()).w_cursor.lnum;
        funcexe.fe_lastline = (*curwin.get()).w_cursor.lnum;
        funcexe.fe_evaluate = true;
        call_func(
            func.as_mut_ptr(),
            name_len,
            &raw mut rettv,
            2,
            argvars.as_mut_ptr(),
            &raw mut funcexe,
        );

        tv_list_unref(arguments);
        restore_funccal();
        provider_caller_scope.set(saved_provider_caller_scope);
        *provider_call_nesting.ptr() -= 1;
        debug_assert!(provider_call_nesting.get() >= 0);

        if discard {
            tv_clear(&raw mut rettv);
        }
        rettv
    }
}

/// Is this provider both known and usable? Loads its autoload script if it
/// has not been loaded yet.
///
/// # Safety
/// `feat` must be NUL-terminated.
pub unsafe fn eval_has_provider(feat: *const c_char, throw_if_fast: bool) -> bool {
    unsafe {
        const KNOWN: [&core::ffi::CStr; 7] = [
            c"clipboard",
            c"python3",
            c"python3_compiled",
            c"python3_dynamic",
            c"perl",
            c"ruby",
            c"node",
        ];
        if !KNOWN.iter().any(|k| strequal(feat, k.as_ptr())) {
            return false;
        }
        if throw_if_fast && !nlua_is_deferred_safe() {
            semsg(
                e_fast_api_disabled.ptr().cast(),
                c"Vimscript function".as_ptr(),
            );
            return false;
        }

        // The variable and function names use the part before the first
        // `_`: "python3_dynamic" asks about "python3".
        let mut name: [c_char; 32] = [0; 32];
        snprintf(
            name.as_mut_ptr(),
            size_of::<[c_char; 32]>(),
            c"%s".as_ptr(),
            feat,
        );
        strchrsub(name.as_mut_ptr(), b'_' as c_char, NUL as c_char);

        let mut buf: [c_char; NAMEBUF] = [0; NAMEBUF];
        let mut tv = UNSET_TV;

        /// `g:loaded_<name>_provider`, into `buf`.
        ///
        /// # Safety
        /// Both buffers must be valid.
        unsafe fn loaded_var(buf: *mut c_char, name: *mut c_char) -> c_int {
            unsafe {
                snprintf(
                    buf,
                    size_of::<[c_char; NAMEBUF]>(),
                    c"g:loaded_%s_provider".as_ptr(),
                    name,
                )
            }
        }

        let mut len = loaded_var(buf.as_mut_ptr(), name.as_mut_ptr());
        if eval_variable(buf.as_mut_ptr(), len, &raw mut tv, null_mut(), false, true) == FAIL {
            // Not loaded yet: sourcing any function in the provider's
            // autoload namespace is what pulls the script in.
            len = snprintf(
                buf.as_mut_ptr(),
                size_of::<[c_char; NAMEBUF]>(),
                c"provider#%s#bogus".as_ptr(),
                name.as_mut_ptr(),
            );
            script_autoload(buf.as_mut_ptr(), len as size_t, false);

            len = loaded_var(buf.as_mut_ptr(), name.as_mut_ptr());
            if eval_variable(buf.as_mut_ptr(), len, &raw mut tv, null_mut(), false, true) == FAIL {
                snprintf(
                    buf.as_mut_ptr(),
                    size_of::<[c_char; NAMEBUF]>(),
                    c"provider#%s#Call".as_ptr(),
                    name.as_mut_ptr(),
                );
                if !find_func(buf.as_mut_ptr()).is_null() && p_lpl.get() != 0 {
                    semsg(
                        c"provider: %s: missing required variable g:loaded_%s_provider".as_ptr(),
                        name.as_mut_ptr(),
                        name.as_mut_ptr(),
                    );
                }
                return false;
            }
        }

        // 2 is the "working" value; 1 means the provider declined.
        let mut ok = tv.v_type == VAR_NUMBER && tv.vval.v_number == 2 as varnumber_T;
        if ok {
            snprintf(
                buf.as_mut_ptr(),
                size_of::<[c_char; NAMEBUF]>(),
                c"provider#%s#Call".as_ptr(),
                name.as_mut_ptr(),
            );
            if find_func(buf.as_mut_ptr()).is_null() {
                semsg(
                    c"provider: %s: g:loaded_%s_provider=2 but %s is not defined".as_ptr(),
                    name.as_mut_ptr(),
                    name.as_mut_ptr(),
                    buf.as_mut_ptr(),
                );
                ok = false;
            }
        }
        ok
    }
}

/// `"<script>:<line>"` for the innermost execution-stack entry.
///
/// # Safety
/// `buf` must hold `bufsize` writable bytes.
pub unsafe fn eval_fmt_source_name_line(buf: *mut c_char, bufsize: size_t) {
    unsafe {
        let top = top_estack();
        if (*top).es_name.is_null() {
            snprintf(buf, bufsize, c"?".as_ptr());
        } else {
            snprintf(
                buf,
                bufsize,
                c"%s:%d".as_ptr(),
                (*top).es_name,
                (*top).es_lnum,
            );
        }
    }
}

/// Everything the user typed into a prompt buffer since the prompt, as one
/// newline-joined string.
///
/// # Safety
/// `buf` must be valid.
pub unsafe fn prompt_get_input(buf: *mut buf_T) -> *mut c_char {
    unsafe {
        if !bt_prompt(buf) {
            return null_mut();
        }
        let lnum_start = (*buf).b_prompt_start.mark.lnum;
        let lnum_last = (*buf).b_ml.ml_line_count;

        let mut text = ml_get_buf(buf, lnum_start);
        // The prompt itself is skipped, unless the line is shorter than
        // the recorded column.
        if strlen(text) as c_int >= (*buf).b_prompt_start.mark.col {
            text = text.offset((*buf).b_prompt_start.mark.col as isize);
        }
        let mut full_text = xstrdup(text);
        for i in (lnum_start + 1)..=lnum_last {
            let half_text = concat_str(full_text, c"\n".as_ptr());
            xfree(full_text as *mut c_void);
            full_text = concat_str(half_text, ml_get_buf(buf, i));
            xfree(half_text as *mut c_void);
        }
        full_text
    }
}

/// The user pressed Enter in a prompt buffer: open the next line and hand
/// what was typed to the buffer's callback.
///
/// # Safety
/// Called from the prompt-buffer key handling, with a prompt buffer
/// current.
pub unsafe fn prompt_invoke_callback() {
    unsafe {
        let lnum = (*curbuf.get()).b_ml.ml_line_count;
        let user_input = prompt_get_input(curbuf.get());
        if user_input.is_null() {
            return;
        }

        ml_append(lnum, c"".as_ptr() as *mut c_char, 0 as colnr_T, false);
        appended_lines_mark(lnum, 1);
        (*curwin.get()).w_cursor.lnum = lnum + 1;
        (*curwin.get()).w_cursor.col = 0;
        (*curbuf.get()).b_prompt_start.mark.lnum = lnum + 1;

        if (*curbuf.get()).b_prompt_callback.type_0 == kCallbackNone {
            xfree(user_input as *mut c_void);
        } else {
            let mut rettv = UNSET_TV;
            let mut argv = [UNSET_TV; 2];
            argv[0].v_type = VAR_STRING;
            argv[0].vval.v_string = user_input;
            argv[1].v_type = VAR_UNKNOWN;
            callback_call(
                &raw mut (*curbuf.get()).b_prompt_callback,
                1,
                argv.as_mut_ptr(),
                &raw mut rettv,
            );
            tv_clear(argv.as_mut_ptr());
            tv_clear(&raw mut rettv);
        }

        u_clearallandblockfree(curbuf.get());
        (*curbuf.get()).b_prompt_start.mark.lnum = (*curbuf.get()).b_ml.ml_line_count;
        (*curbuf.get()).b_prompt_append_new_line = true;
    }
}

/// CTRL-C in a prompt buffer. Answers whether the buffer had an interrupt
/// callback at all.
///
/// # Safety
/// As `prompt_invoke_callback`.
pub unsafe fn invoke_prompt_interrupt() -> bool {
    unsafe {
        if (*curbuf.get()).b_prompt_interrupt.type_0 == kCallbackNone {
            return false;
        }
        let mut rettv = UNSET_TV;
        let mut argv = [UNSET_TV; 1];
        argv[0].v_type = VAR_UNKNOWN;
        // The interrupt is consumed here; the callback decides what to do
        // about it.
        got_int.set(false);
        let ret = callback_call(
            &raw mut (*curbuf.get()).b_prompt_interrupt,
            0,
            argv.as_mut_ptr(),
            &raw mut rettv,
        );
        tv_clear(&raw mut rettv);
        ret as c_int != FAIL
    }
}
