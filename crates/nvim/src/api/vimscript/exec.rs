//! Running Vimscript: `nvim_exec2()` and `nvim_command()`.
//!
//! [`exec_impl`] is the shared body -- it sources the string as an anonymous
//! script with `do_source_str`, optionally capturing the messages it prints so
//! `opts.output` can hand them back -- and the two entry points differ only in
//! whether they take that option.

#![deny(unsafe_op_in_unsafe_fn)]

use super::*;
use crate::api::private::helpers::{ERROR_INIT, Reported, dict_put_str};
use crate::guard::Suppress;
use crate::types::NUL;
use core::ffi::c_char;
use core::ptr;

pub unsafe fn nvim_exec2(
    channel_id: uint64_t,
    src: String_0,
    opts: *mut KeyDict_exec_opts,
) -> Result<Dict, Error> {
    let mut error = ERROR_INIT;
    let err = &raw mut error;
    unsafe {
        let output: String_0 = exec_impl(channel_id, src, opts, err);
        if (*err).type_0 != kErrorTypeNone || !(*opts).output {
            return Dict::EMPTY.reported(error);
        }
        // Heap-allocated rather than arena-allocated: the caller frees this
        // dictionary key by key, so the key is a copy too.
        let mut result: Dict = arena_dict(ptr::null_mut(), 1);
        dict_put_str(
            &mut result,
            cstr_to_string(c"output".as_ptr()),
            Object::string(output),
        );
        result.reported(error)
    }
}

/// Source `src` as an anonymous script, answering whatever it printed when
/// `opts.output` asked for it (and the empty string otherwise, or on error).
pub unsafe fn exec_impl(
    channel_id: uint64_t,
    src: String_0,
    opts: *mut KeyDict_exec_opts,
    err: *mut Error,
) -> String_0 {
    unsafe {
        // Read once: `opts` is the dispatcher's own copy of the keyword
        // arguments, which nothing the sourced script can do reaches.
        let capture = (*opts).output;
        let save_redir_off = redir_off.get();
        let save_capture_ga = capture_ga.get();
        let save_msg_col = msg_col.get();
        let mut capture_local: garray_T = ::core::mem::zeroed();
        if capture {
            ga_init(&raw mut capture_local, 1, 80);
            capture_ga.set(&raw mut capture_local);
        }
        let mut tstate: TryState = TRY_STATE_INIT;
        try_enter(&raw mut tstate);
        let silenced = capture.then(Suppress::messages_saved);
        if capture {
            redir_off.set(false);
            msg_col.set(0);
        }
        let save_current_sctx: sctx_T = api_set_sctx(channel_id);
        do_source_str(src.data(), c"nvim_exec2()".as_ptr() as *mut c_char);
        drop(silenced);
        if capture {
            capture_ga.set(save_capture_ga);
            redir_off.set(save_redir_off);
            msg_col.set(save_msg_col);
        }
        current_sctx.set(save_current_sctx);
        try_leave(&raw mut tstate, err);

        // The capture always starts with the newline that separated the first
        // message from whatever was on screen; drop it. A one-byte capture is
        // that newline alone, i.e. nothing was printed.
        if (*err).type_0 == kErrorTypeNone && capture && capture_local.ga_len > 1 {
            let mut s: String_0 = String_0::from_raw_parts(
                capture_local.ga_data.cast::<c_char>(),
                capture_local.ga_len as size_t,
            );
            if *s.data() == '\n' as c_char {
                memmove(s.data().cast(), s.data().add(1).cast(), s.len() - 1);
                *s.data().add(s.len() - 1) = NUL as c_char;
                s.set_len(s.len() - 1);
            }
            return s;
        }
        if capture {
            ga_clear(&raw mut capture_local);
        }
        String_0::NULL
    }
}

pub unsafe fn nvim_command(cmd: String_0) -> Result<(), Error> {
    let mut error = ERROR_INIT;
    let err = &raw mut error;
    unsafe {
        let mut tstate: TryState = TRY_STATE_INIT;
        try_enter(&raw mut tstate);
        do_cmdline_cmd(cmd.data());
        try_leave(&raw mut tstate, err);
    }
    ().reported(error)
}
