//! `nvim_echo()`: printing a chunked, highlighted message.
//!
//! One function, because a message is an array of (text, highlight) chunks
//! that has to be rendered as a unit under the caller's `history`, `err`
//! and `verbose` options, with the message state saved and restored around
//! it.

#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use super::*;
use crate::api::private::helpers::Reported;
use crate::api::private::validate::{err_expected, err_out_of_range, err_required};
use crate::api_error;
use crate::guard::Suppress;
use crate::message_fmt::msg_cstr_opt;
use crate::types::HlMessage;
use core::ffi::CStr;

/// The `status` key's four spellings, which only a `kind='progress'` message
/// may carry.
const PROGRESS_STATUS: [&CStr; 4] = [c"success", c"failed", c"running", c"cancel"];

/// Print `chunks` as one message, `history` to record it in `:messages`.
pub fn nvim_echo(
    chunks: Array,
    history: Boolean,
    opts: &mut KeyDict_echo_opts,
) -> Result<Object, Error> {
    let mut error = Error::none();
    // The keys, with every one the caller left out reading as its default:
    // no text, no percentage, no dictionary.
    let no_string = String_0::NULL;
    let (err, verbose, truncate) = (
        opts.err.unwrap_or(false),
        opts.verbose.unwrap_or(false),
        opts._truncate.unwrap_or(false),
    );
    let (title, status, source) = (
        opts.title.as_ref().unwrap_or(&no_string),
        opts.status.as_ref().unwrap_or(&no_string),
        opts.source.as_ref().unwrap_or(&no_string),
    );
    let percent = opts.percent.unwrap_or(0);
    let (no_dict, nil) = (ApiDict::EMPTY, Object::Nil);
    let data = opts.data.as_ref().unwrap_or(&no_dict);
    let given_id = opts.id.as_ref().unwrap_or(&nil);
    let mut id = Object::integer(-1);
    let mut hl_msg = HlMessage::new();
    parse_hl_msg(&mut hl_msg, &chunks, err)?;

    let given_kind = opts.kind.as_ref().unwrap_or(&no_string);
    if verbose {
        verbose_enter();
    }
    // `:verbose` keeps whatever kind it was given, the empty one included:
    // `verbose_enter` has already set its own, and the defaults below would
    // overwrite it.
    let kind: Option<&CStr> = match () {
        _ if !given_kind.is_null() => Some(given_kind.as_cstr()),
        _ if verbose => None,
        _ if err => Some(c"echoerr"),
        _ if history => Some(c"echomsg"),
        _ => Some(c"echo"),
    };
    let is_progress = kind == Some(c"progress");
    let mut needs_clear = !history;

    // The progress keys belong to `kind='progress'` and to nothing else, and
    // each of them has its own range.
    let has_progress_keys = !status.is_empty()
        || !title.is_empty()
        || percent != 0
        || !data.is_empty()
        || !source.is_empty();
    let echo_id = given_id.as_integer();
    let rejected = {
        if !is_progress && has_progress_keys {
            let kind = msg_cstr_opt(kind);
            error = api_error!(
                kErrorTypeValidation,
                "Conflict: title/source/status/percent/data not allowed with kind='{kind}'"
            );
            true
        } else if is_progress && !status_named(status.clone()) {
            let names = c"success|failed|running|cancel";
            let got = (!status.is_null()).then(|| status.as_cstr());
            error = err_expected(c"status", names, got);
            true
        } else if is_progress && !(0..=100).contains(&percent) {
            error = err_out_of_range(c"percent");
            true
        } else if is_progress && source.is_empty() {
            error = err_required(c"opts.source");
            true
        } else if let Some(id) = echo_id.filter(|&id| !msg_id_exists(id)) {
            error = api_error!(kErrorTypeValidation, "Invalid 'id': {id}");
            true
        } else {
            false
        }
    };

    if !rejected {
        let msg_data = MessageData {
            source: source.clone(),
            percent,
            title: title.clone(),
            status: status.clone(),
            data: data.clone(),
        };
        let save_nwr = need_wait_return.get();
        let save_lines_left = lines_left.get();
        let save_msg_didany = msg_didany.get();
        let no_prompt = truncate.then(Suppress::wait_return);
        if truncate {
            lines_left.set(0 as ::core::ffi::c_int);
            msg_didany.set(true);
            msg_no_more.set(true);
        }
        id = msg_multihl(
            given_id.clone(),
            &hl_msg,
            kind,
            history,
            err,
            Some(&msg_data),
            &mut needs_clear,
        );
        if truncate {
            msg_no_more.set(false);
            msg_didany.set(save_msg_didany);
            lines_left.set(save_lines_left);
            drop(no_prompt);
            need_wait_return.set(save_nwr);
        }
        if verbose {
            verbose_leave();
            verbose_stop();
        }
        if is_progress {
            do_autocmd_progress(id.clone(), &hl_msg, Some(&msg_data));
        }
    }
    id.reported(error)
}

/// Whether `status` is one of the four a progress message may carry. The
/// empty string is not one of them.
fn status_named(status: String_0) -> bool {
    PROGRESS_STATUS.iter().any(|name| status.as_cstr() == *name)
}
