//! Running Vimscript: `nvim_exec2()` and `nvim_command()`.
//!
//! [`exec_impl`] is the shared body -- it sources the string as an anonymous
//! script with `do_source_str`, optionally capturing the messages it prints so
//! `opts.output` can hand them back -- and the two entry points differ only in
//! whether they take that option.

#![deny(unsafe_op_in_unsafe_fn)]
#![allow(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use super::*;
use crate::api::private::helpers::api_try;
use crate::cstr;
use crate::guard::Suppress;
use core::ffi::c_char;

/// # Safety
///
/// `src` must be a well-formed API string: `size` readable bytes with a NUL
/// at `data[size]`. `opts` must point at the `KeyDict_exec_opts` the
/// dispatcher filled in, live for the call.
pub unsafe fn nvim_exec2(
    channel_id: uint64_t,
    src: String_0,
    opts: *mut KeyDict_exec_opts,
) -> Result<ApiDict, Error> {
    // SAFETY: `src`/`opts` are the caller's.
    let output: String_0 = unsafe { exec_impl(channel_id, src, opts) }?;
    // SAFETY: `opts` is the caller's keydict, live for the call.
    if !unsafe { (*opts).output }.unwrap_or(false) {
        return Ok(ApiDict::EMPTY);
    }
    // Heap-allocated rather than arena-allocated: the caller frees this
    // dictionary key by key, so the key is a copy too.
    let mut result: ApiDict = ApiDict::with_capacity(1);
    result.insert(c"output", Object::string(output));
    Ok(result)
}

/// Source `src` as an anonymous script, answering whatever it printed when
/// `opts.output` asked for it (and the empty string otherwise, or on error).
///
/// # Safety
///
/// `src` must be a well-formed API string: `size` readable bytes with a NUL
/// at `data[size]`. `opts` must point at the `KeyDict_exec_opts` the
/// dispatcher filled in, live for the call.
pub unsafe fn exec_impl(
    channel_id: uint64_t,
    src: String_0,
    opts: *mut KeyDict_exec_opts,
) -> Result<String_0, Error> {
    // Read once: `opts` is the dispatcher's own copy of the keyword
    // arguments, which nothing the sourced script can do reaches.
    // SAFETY: `opts` is the caller's keydict, live for the call.
    let capture = unsafe { (*opts).output }.unwrap_or(false);
    let save_redir_off = redir_off.get();
    let save_msg_col = msg_col.get();
    let outer_capture = capture.then(capture_start);
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
    let captured = outer_capture.map(capture_finish);
    if capture {
        redir_off.set(save_redir_off);
        msg_col.set(save_msg_col);
    }
    drop(sctx);
    // SAFETY: `tstate` is what the `try_enter` above filled in.
    let thrown = unsafe { try_leave(&raw mut tstate) };

    let caught = thrown.is_err();
    // The capture always starts with the newline that separated the first
    // message from whatever was on screen; drop it. A one-byte capture is
    // that newline alone, i.e. nothing was printed.
    if let Some(captured) = captured.filter(|captured| captured.len() > 1)
        && !caught
    {
        // Messages open with a newline the caller did not ask for.
        let skip = usize::from(captured[0] == b'\n');
        return Ok(String_0::from_bytes(&captured[skip..]));
    }
    thrown.map(|()| String_0::NULL)
}

pub fn nvim_command(cmd: String_0) -> Result<(), Error> {
    api_try(|| {
        // SAFETY: `cmd` is the caller's NUL-terminated command line.
        let _ = do_cmdline_cmd(unsafe { cstr::at(cmd.data()) });
    })?;
    Ok(())
}
