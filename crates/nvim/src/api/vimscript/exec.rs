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
    // SAFETY: `result` was sized for exactly this pair, and the key is a
    // fresh copy the caller takes over with it.
    unsafe {
        let key = cstr_to_string(c"output".as_ptr());
        result.insert(key, Object::string(output));
    }
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
    let save_capture_ga = capture_ga.get();
    let save_msg_col = msg_col.get();
    // SAFETY: a `GArray` is two counts, an item size and a pointer, so
    // all-zero is a valid value; `ga_init` fills it in before it is used.
    let mut capture_local: GArray = unsafe { ::core::mem::zeroed() };
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
    // SAFETY: `tstate` is what the `try_enter` above filled in.
    let thrown = unsafe { try_leave(&raw mut tstate) };

    let caught = thrown.is_err();
    // The capture always starts with the newline that separated the first
    // message from whatever was on screen; drop it. A one-byte capture is
    // that newline alone, i.e. nothing was printed.
    if !caught && capture && capture_local.ga_len > 1 {
        let captured =
            usize::try_from(capture_local.ga_len).expect("a garray length is never negative");
        // SAFETY: the capture holds `ga_len` bytes of message text.
        let mut s: String_0 = String_0::from_bytes(unsafe {
            core::slice::from_raw_parts(capture_local.ga_data.cast::<u8>(), captured)
        });
        // Messages open with a newline the caller did not ask for.
        if s.as_bytes().first() == Some(&b'\n') {
            // SAFETY: the string is its own, and the shift leaves `len - 1`
            // bytes with the terminator `truncate` writes after them.
            unsafe {
                s.data()
                    .cast::<u8>()
                    .copy_from(s.data().add(1).cast(), s.len() - 1);
                s.truncate(s.len() - 1);
            }
        }
        return Ok(s);
    }
    if capture {
        // SAFETY: `capture_local` is this frame's, and nothing points at it
        // any more.
        unsafe { ga_clear(&raw mut capture_local) };
    }
    thrown.map(|()| String_0::NULL)
}

/// # Safety
///
/// `cmd` must be a well-formed API string: `size` readable bytes with a NUL
/// at `data[size]`.
pub unsafe fn nvim_command(cmd: String_0) -> Result<(), Error> {
    api_try(|| {
        // SAFETY: `cmd` is the caller's NUL-terminated command line.
        let _ = unsafe { do_cmdline_cmd(cmd.data()) };
    })?;
    Ok(())
}
