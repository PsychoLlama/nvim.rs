//! Running Vimscript: `nvim_exec2()` and `nvim_command()`.
//!
//! [`exec_impl`] is the shared body -- it sources the string as an anonymous
//! script with `do_source_str`, optionally capturing the messages it prints so
//! `opts.output` can hand them back -- and the two entry points differ only in
//! whether they take that option.

#![deny(unsafe_op_in_unsafe_fn)]

use super::*;
use crate::api::private::helpers::{ERROR_INIT, Reported, api_try, dict_put_str};
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
    // SAFETY: `src`/`opts` are the caller's and `error` this frame's slot.
    let output: String_0 = unsafe { exec_impl(channel_id, src, opts, &mut error) };
    // SAFETY: `opts` is the caller's keydict, live for the call.
    let wanted = unsafe { (*opts).output };
    if error.is_set() || !wanted {
        return Dict::EMPTY.reported(error);
    }
    // Heap-allocated rather than arena-allocated: the caller frees this
    // dictionary key by key, so the key is a copy too.
    let mut result: Dict = arena_dict(ptr::null_mut(), 1);
    // SAFETY: `result` was sized for exactly this pair, and the key is a
    // fresh copy the caller takes over with it.
    unsafe {
        let key = cstr_to_string(c"output".as_ptr());
        dict_put_str(&mut result, key, Object::string(output));
    }
    result.reported(error)
}

/// Source `src` as an anonymous script, answering whatever it printed when
/// `opts.output` asked for it (and the empty string otherwise, or on error).
pub unsafe fn exec_impl(
    channel_id: uint64_t,
    src: String_0,
    opts: *mut KeyDict_exec_opts,
    err: &mut Error,
) -> String_0 {
    // Read once: `opts` is the dispatcher's own copy of the keyword
    // arguments, which nothing the sourced script can do reaches.
    // SAFETY: `opts` is the caller's keydict, live for the call.
    let capture = unsafe { (*opts).output };
    let save_redir_off = redir_off.get();
    let save_capture_ga = capture_ga.get();
    let save_msg_col = msg_col.get();
    // SAFETY: a `garray_T` is two counts, an item size and a pointer, so
    // all-zero is a valid value; `ga_init` fills it in before it is used.
    let mut capture_local: garray_T = unsafe { ::core::mem::zeroed() };
    if capture {
        // SAFETY: `capture_local` is this frame's, and outlives the source
        // below -- the global is put back before this returns.
        unsafe { ga_init(&raw mut capture_local, 1, 80) };
        capture_ga.set(&raw mut capture_local);
    }
    let mut tstate: TryState = TRY_STATE_INIT;
    // SAFETY: `tstate` is this frame's, live until the `try_leave` below.
    unsafe { try_enter(&raw mut tstate) };
    let silenced = capture.then(Suppress::messages_saved);
    if capture {
        redir_off.set(false);
        msg_col.set(0);
    }
    let sctx = api_set_sctx(channel_id);
    let name = c"nvim_exec2()".as_ptr() as *mut c_char;
    // SAFETY: `src` names its own bytes and `name` is a static C string.
    unsafe { do_source_str(src.data(), name) };
    drop(silenced);
    if capture {
        capture_ga.set(save_capture_ga);
        redir_off.set(save_redir_off);
        msg_col.set(save_msg_col);
    }
    drop(sctx);
    // SAFETY: `tstate` is what the `try_enter` above filled in, and `err`
    // is the caller's slot.
    unsafe { try_leave(&raw mut tstate, err) };

    let caught = err.kind() != kErrorTypeNone;
    // The capture always starts with the newline that separated the first
    // message from whatever was on screen; drop it. A one-byte capture is
    // that newline alone, i.e. nothing was printed.
    if !caught && capture && capture_local.ga_len > 1 {
        let mut s: String_0 = String_0::from_raw_parts(
            capture_local.ga_data.cast::<c_char>(),
            capture_local.ga_len as size_t,
        );
        // SAFETY: the capture holds `ga_len` bytes, at least two of them.
        unsafe {
            if *s.data() == '\n' as c_char {
                memmove(s.data().cast(), s.data().add(1).cast(), s.len() - 1);
                *s.data().add(s.len() - 1) = NUL as c_char;
                s.set_len(s.len() - 1);
            }
        }
        return s;
    }
    if capture {
        // SAFETY: `capture_local` is this frame's, and nothing points at it
        // any more.
        unsafe { ga_clear(&raw mut capture_local) };
    }
    String_0::NULL
}

pub unsafe fn nvim_command(cmd: String_0) -> Result<(), Error> {
    let mut error = ERROR_INIT;
    api_try(&mut error, |_| {
        // SAFETY: `cmd` is the caller's NUL-terminated command line.
        unsafe { do_cmdline_cmd(cmd.data()) };
    });
    ().reported(error)
}
