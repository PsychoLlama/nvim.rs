//! Turning a Lua error into an editor error.
//!
//! [`nlua_get_error`] reads whatever is on the stack -- a string, or a table
//! with a `__tostring` -- and [`nlua_error`] prints it through the
//! multiline message path. [`nlua_pcall`] is the protected call every entry
//! point in this module funnels through, [`fast_pcall`] the one a libuv
//! callback on the main loop runs through, and [`report_luv_error`] reports
//! an error raised there, or on a luv thread, where there is no caller to
//! return to.

#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use core::ffi::{CStr, c_int};
use std::io::Write;

use super::{LUA_ERRMEM, LUVF_CALLBACK_NOEXIT, in_fast_callback, in_script};
use crate::event::multiqueue::put_job_on_main_loop;
use crate::guard::Depth;
use crate::lua::ffi::LUA_MULTRET;
use crate::lua::stack::{At, Kind, LuaFailed, Stack};
use crate::message::e_outofmem;
use crate::message_fmt::{msg_bytes_opt, report_emsg_multiline};
use crate::os::cshim::gettext;
use crate::startup::preserve_exit;
use crate::tr_c;

/// The message kind every Lua error is reported under.
const LUA_ERROR_KIND: &CStr = c"lua_error";
/// The metamethod a Lua error object may render itself through.
const TOSTRING: &CStr = c"__tostring";

/// The bytes a C string's copy of `bytes` would keep: up to the first NUL.
pub(crate) fn until_nul(bytes: &[u8]) -> &[u8] {
    let end = bytes.iter().position(|&b| b == 0).unwrap_or(bytes.len());
    &bytes[..end]
}

/// The error on top of the stack as text, leaving it there (rendered through
/// its `__tostring` first when it has one). `None` for an error that is no
/// string and renders as none.
pub(crate) fn nlua_get_error(stack: &mut Stack) -> Option<&[u8]> {
    if stack.get_metafield(At::TOP, TOSTRING) {
        if stack.kind(At::TOP) == Kind::Function && stack.call_meta(At::from_top(2), TOSTRING) {
            stack.replace(At::from_top(3));
        }
        stack.pop(1);
    }
    stack.to_bytes(At::TOP)
}

/// Report the error on top of the stack, and pop it. `template` is the
/// message, with one `%.*s` for the error's own text.
///
/// A `-l` script has no editor to show a message in, so its errors go to
/// stderr.
pub fn nlua_error(stack: &mut Stack, template: &'static CStr) {
    let text = nlua_get_error(stack);
    if in_script.get() {
        // `fprintf(stderr, template, len, text)`: the bytes as they are,
        // `%.*s` stopping at a NUL, and a null string printing as nothing
        // at the zero precision it comes with.
        let template = gettext(template).to_bytes();
        let at = template
            .windows(4)
            .position(|w| w == b"%.*s")
            .unwrap_or(template.len());
        let rest = template.get(at + 4..).unwrap_or_default();
        let text = text.map(until_nul).unwrap_or_default();
        let line = [&template[..at], text, rest, b"\n"].concat();
        let _ = std::io::stderr().write_all(&line);
    } else {
        let len = text.map_or(0, <[u8]>::len);
        let len = c_int::try_from(len).unwrap_or(c_int::MAX);
        report_emsg_multiline(LUA_ERROR_KIND, || tr_c!(template, len, msg_bytes_opt(text)));
    }
    stack.pop(1);
}

/// `lua_pcall` with `debug.traceback` installed as the message handler, so a
/// failure carries the Lua stack that produced it.
///
/// The handler is inserted below the function and its arguments, and removed
/// again whichever way the call went — so the stack effect is exactly
/// `lua_pcall`'s. Expects a function and `nargs` arguments on top.
pub fn nlua_pcall(stack: &mut Stack, nargs: c_int, mut nresults: c_int) -> Result<(), LuaFailed> {
    stack.get_global(c"debug");
    stack.get_field(At::TOP, c"traceback");
    stack.remove(At::from_top(2));
    stack.insert(At::from_top(2 + nargs));
    let pre_top = stack.top().index();
    let status = stack.pcall(nargs, nresults, Some(At::from_top(2 + nargs)));
    if status.is_err() {
        stack.remove(At::from_top(2));
    } else {
        if nresults == LUA_MULTRET {
            nresults = stack.top().index() - (pre_top - nargs - 1);
        }
        stack.remove(At::from_top(1 + nresults));
    }
    status
}

/// Where an error [`report_luv_error`] reports was raised.
#[derive(Clone, Copy)]
pub(crate) enum LuvError {
    /// A libuv callback on the main loop.
    Callback,
    /// A luv thread's own code.
    Thread,
    /// A callback on a luv thread.
    ThreadCallback,
}

/// Report an error raised where no caller can see it: a luv callback, a luv
/// thread, or a callback on one. `error` is its text (`None` for one that had
/// none), already cut at its first NUL.
///
/// Scheduled onto the main loop by whoever raised it.
pub(crate) fn report_luv_error(kind: LuvError, error: Option<&[u8]>) {
    let template = match kind {
        LuvError::Callback => c"Lua callback:\n%s",
        LuvError::Thread => c"Luv thread:\n%s",
        LuvError::ThreadCallback => c"Luv callback, thread:\n%s",
    };
    report_emsg_multiline(LUA_ERROR_KIND, || tr_c!(template, msg_bytes_opt(error)));
}

/// Run a libuv callback on the main loop: [`nlua_pcall`] with
/// `in_fast_callback` raised for its duration. Expects the callback and
/// `nargs` arguments on top; answers how many results it left, or a failure's
/// negated status.
///
/// A failure is *reported* rather than propagated — there is no caller to
/// return an error to — so its text is copied and queued onto the main loop.
pub(crate) fn fast_pcall(
    stack: &mut Stack,
    nargs: c_int,
    mut nresult: c_int,
    flags: c_int,
) -> c_int {
    let _fast = Depth::of(&in_fast_callback);
    let top = stack.top().index();
    if let Err(LuaFailed(status)) = nlua_pcall(stack, nargs, nresult) {
        if status == LUA_ERRMEM && flags & LUVF_CALLBACK_NOEXIT == 0 {
            preserve_exit(Some(e_outofmem));
        }
        let error = nlua_get_error(stack).map(|text| until_nul(text).to_vec());
        put_job_on_main_loop(Box::new(move || {
            report_luv_error(LuvError::Callback, error.as_deref());
        }));
        stack.pop(1);
        return -status;
    }
    if nresult == LUA_MULTRET {
        nresult = stack.top().index() - top + nargs + 1;
    }
    nresult
}
