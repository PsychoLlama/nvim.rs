//! Running Vimscript: `nvim_exec2()` and `nvim_command()`.
//!
//! [`exec_impl`] is the shared body -- it sources the string as an anonymous
//! script with `do_source_str`, optionally capturing the messages it prints so
//! `opts.output` can hand them back -- and the two entry points differ only in
//! whether they take that option.

#![forbid(unsafe_code)]
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

pub fn nvim_exec2(
    channel_id: uint64_t,
    src: String_0,
    opts: &mut KeyDict_exec_opts,
) -> Result<ApiDict, Error> {
    let output: String_0 = (exec_impl(channel_id, src, opts))?;
    if !opts.output.unwrap_or(false) {
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
pub fn exec_impl(
    channel_id: uint64_t,
    src: String_0,
    opts: &mut KeyDict_exec_opts,
) -> Result<String_0, Error> {
    // Read once: `opts` is the dispatcher's own copy of the keyword
    // arguments, which nothing the sourced script can do reaches.
    let capture = opts.output.unwrap_or(false);
    let save_redir_off = redir_off.get();
    let save_msg_col = msg_col.get();
    let outer_capture = capture.then(capture_start);
    let thrown = api_try(|| {
        let silenced = capture.then(Suppress::messages_saved);
        if capture {
            redir_off.set(false);
            msg_col.set(0);
        }
        let sctx = api_set_sctx(channel_id);
        do_source_str(src.as_cstr(), c"nvim_exec2()");
        drop(silenced);
        let captured = outer_capture.map(capture_finish);
        if capture {
            redir_off.set(save_redir_off);
            msg_col.set(save_msg_col);
        }
        drop(sctx);
        captured
    });

    // An exception outranks whatever was captured.
    let captured = thrown?;
    // The capture always starts with the newline that separated the first
    // message from whatever was on screen; drop it. A one-byte capture is
    // that newline alone, i.e. nothing was printed.
    if let Some(captured) = captured.filter(|captured| captured.len() > 1) {
        // Messages open with a newline the caller did not ask for.
        let skip = usize::from(captured[0] == b'\n');
        return Ok(String_0::from_bytes(&captured[skip..]));
    }
    Ok(String_0::NULL)
}

pub fn nvim_command(cmd: String_0) -> Result<(), Error> {
    api_try(|| {
        let _ = do_cmdline_cmd(cmd.as_cstr());
    })?;
    Ok(())
}
