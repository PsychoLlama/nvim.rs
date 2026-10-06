//! `:redir => var` -- capturing messages into a variable.
//!
//! [`var_redir_start`] resolves the target once and seeds it,
//! [`var_redir_str`] appends every message to a growable buffer, and
//! [`var_redir_stop`] stores the result.  [`assert_error`] is the same trick
//! for `v:errors`.

#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use crate::cstr::byte_at;
use crate::eval::typval::tv_list_alloc;
use crate::eval::{FNE_CHECK_START, eval_isnamec1, get_lval, set_var_lval};
use crate::global_cell::GlobalCell;
use crate::memory::XString;
use crate::message::e_invarg;
use crate::message::state::{called_emsg, did_emsg};
use crate::message_fmt::msg_bytes;
use crate::semsg;
use crate::types::{Failed, TypVal, Vv};
use core::ffi::c_int;

use super::{clear_local, emsg_static, get_vim_var_list_handle, set_vim_var_list};

/// Append `message` to `v:errors`, which the `assert_*` builtins report
/// through.
pub fn assert_error(message: &[u8]) {
    let mut errors = match get_vim_var_list_handle(Vv::Errors) {
        Some(errors) => errors,
        None => {
            // Something replaced it; make sure `v:errors` is a List again.
            let errors = tv_list_alloc(1);
            set_vim_var_list(Vv::Errors, Some(errors.clone()));
            errors
        }
    };
    // A message that was never appended to used to be a null `ga_data`,
    // which appends `v:null`'s string; an empty one is an empty string.
    let text = if message.is_empty() {
        core::ptr::null_mut()
    } else {
        XString::from_bytes(message).into_raw()
    };
    errors.push(TypVal::string_raw(text));
}

/// The name of the variable a running `:redir =>` captures into, `None`
/// while none runs. The name, not the resolved lvalue, is what is kept: a
/// Dict or List entry may move before the end, so [`var_redir_stop`] parses
/// it again.
static REDIR_TARGET: GlobalCell<Option<Vec<u8>>> = GlobalCell::new(None);
/// The text collected so far.
static REDIR_TEXT: GlobalCell<Vec<u8>> = GlobalCell::new(Vec::new());

/// An owned String value holding `text`.
fn string_value(text: &[u8]) -> TypVal {
    TypVal::string_from(text)
}

/// Start capturing messages into the variable `name`, appending to it rather
/// than replacing it when `append`.
pub(crate) fn var_redir_start(name: &[u8], append: bool) -> Result<(), Failed> {
    // Catch a bad name early.
    if !eval_isnamec1(c_int::from(byte_at(name, 0))) {
        emsg_static(e_invarg);
        return Err(Failed);
    }

    // The name is parsed again in `var_redir_stop`, so it is kept for as
    // long as the redirection runs.
    REDIR_TARGET.set(Some(name.to_vec()));
    // The output is collected here until redirection ends.
    REDIR_TEXT.with_mut(|text| {
        text.clear();
        text.reserve(500);
    });

    // Parse the name, which may be a Dict or List entry. Resolving it may
    // run user code (an index expression), which may start or stop a
    // redirection, so it is the caller's text that is read and not the
    // copy kept above.
    let (mut lval, end) = get_lval(name, None, false, false, 0, FNE_CHECK_START);
    let trailing = end.map(|end| byte_at(name, end));
    if trailing.is_none_or(|c| c != 0) || !lval.has_name() {
        match end {
            Some(end) if byte_at(name, end) != 0 => {
                let rest = msg_bytes(&name[end..]);
                semsg!("E488: Trailing characters: {rest}");
            }
            _ => {
                let name = String::from_utf8_lossy(name);
                semsg!("E475: Invalid argument: {name}");
            }
        }
        abandon_redir();
        return Err(Failed);
    }

    // Check the variable can be written, by setting it to -- or
    // appending to it -- an empty string.
    let called_emsg_before = called_emsg.get();
    did_emsg.set(0);
    let mut tv = string_value(b"");
    let op = if append { b'.' } else { b'=' };
    set_var_lval(&mut lval, &mut tv, true, false, Some(op));
    clear_local(&mut tv);
    if called_emsg.get() > called_emsg_before {
        abandon_redir();
        return Err(Failed);
    }
    Ok(())
}

/// End a redirection whose start failed: store nothing.
fn abandon_redir() {
    REDIR_TARGET.set(None);
    REDIR_TEXT.take();
}

/// Append `value` to what `:redir =>` is capturing.
///
/// The store is postponed to [`var_redir_stop`] on purpose: what is being
/// appended may *be* the string being written to, and changing it would then
/// use freed memory --
///
/// ```text
///     :redir => foo
///     :let foo
///     :redir END
/// ```
pub(crate) fn var_redir_str(value: &[u8]) {
    if REDIR_TARGET.with(Option::is_none) {
        return;
    }
    REDIR_TEXT.with_mut(|text| text.extend_from_slice(value));
}

/// Stop capturing and store what was collected.
pub fn var_redir_stop() {
    let Some(target) = REDIR_TARGET.with(Clone::clone) else {
        return;
    };
    // Collecting is over: take the buffer, so that a message emitted from
    // inside `set_var_lval` appends to a fresh one. The redirection stays on
    // until the store is done, as upstream's does, and what it captures
    // meanwhile is dropped below.
    let text = REDIR_TEXT.take();
    // The collected text up to its first NUL, as the C string it was.
    let text = &text[..text.iter().position(|&b| b == 0).unwrap_or(text.len())];
    let mut tv = string_value(text);
    // Resolve the name again: inside a Dict or List it may have moved since.
    let (mut lval, end) = get_lval(&target, None, false, false, 0, FNE_CHECK_START);
    if end.is_some() && lval.has_name() {
        set_var_lval(&mut lval, &mut tv, false, false, Some(b'.'));
    }
    clear_local(&mut tv);
    abandon_redir();
}
