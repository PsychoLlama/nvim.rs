//! `assert_fails()`: run a command and check that it failed, optionally with
//! a particular message, from a particular line, in a particular context.
//!
//! It is the one `assert_*()` that runs user code, so it is also the one that
//! has to put the message state back: the command it ran was *expected* to
//! fail, and everything that failure left behind — the error flags, the
//! pending `hit-enter`, `v:errmsg` — is [`finish_assert_fails`]'s to undo.

#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use crate::strings::has_bytes;
use core::ffi::{CStr, c_int};

use crate::eval::pattern_match;
use crate::eval::typval::{
    NumBuf, tv_check_for_opt_number_arg, tv_check_for_opt_string_arg,
    tv_check_for_opt_string_or_list_arg, tv_check_for_string_or_number_arg,
};
use crate::eval::vars::{set_vim_var_string, vim_var_bytes};
use crate::ex_docmd::do_cmdline_cmd;
use crate::ex_eval::state::{suppress_errthrow, trylevel};
use crate::getchar::state::got_int;
use crate::guard::{MsgBump, Suppress};
use crate::memory::{ThinCString, XString};
use crate::message::state::{
    called_emsg, did_emsg, emsg_assert_fails_context, emsg_assert_fails_lnum,
    emsg_assert_fails_msg, emsg_on_display, in_assert_fails, lines_left, msg_col, need_wait_return,
};
use crate::message::{emsg, msg_reset_scroll};
use crate::os::cshim::gettext;
use crate::types::{
    EvalFuncData, TypVal, VAR_LIST, VAR_NUMBER, VAR_STRING, VAR_UNKNOWN, VarNumber, Vv,
};
use crate::ui::state::Rows;

use super::report::{
    Expected, fill_assert_error, prepare_assert_error, push_lit, report_assert_error,
};
use super::{
    AssertType, E_ASSERT_FAILS_FIFTH_ARGUMENT, E_ASSERT_FAILS_FOURTH_ARGUMENT,
    E_ASSERT_FAILS_SECOND_ARG, assert_append_cmd_or_arg,
};

/// What checking `assert_fails()`'s expectations against the reported error
/// produced.
enum FailsCheck {
    /// Everything the caller asked for matched.
    Matched,
    /// It did not; report this.
    Mismatch(FailsMismatch),
    /// Stop without reporting: an argument could not be read as a string, and
    /// the `tv_get_string_buf_chk` that found that already said so.
    Abandon,
    /// Stop, and report this argument error after the cleanup.
    BadArg(&'static CStr),
}

/// The mismatch a failed `assert_fails()` describes.
struct FailsMismatch {
    /// The pattern the caller gave, quoted in the message, copied out of
    /// the scratch buffer it may have been rendered into (a Number item),
    /// which does not outlive the check. `None` when the expectation is
    /// printed from the argument itself instead.
    expected_str: Option<ThinCString>,
    /// Which `argvars` slot the unmet expectation came from: 1, 3 or 4.
    index: usize,
    /// The error text that actually arrived, for `index == 1`.
    actual: Option<ThinCString>,
}

/// Whether `assert_fails()`'s arguments have the shapes it documents.
///
/// The later ones are only checked when the earlier optional ones are present,
/// exactly as upstream: `assert_fails(cmd, err, msg, lnum, context)`.
fn assert_fails_args_ok(args: &[TypVal]) -> bool {
    if tv_check_for_string_or_number_arg(args, 0).is_err()
        || tv_check_for_opt_string_or_list_arg(args, 1).is_err()
    {
        return false;
    }
    if args.len() <= 2 {
        return true;
    }
    if tv_check_for_opt_number_arg(args, 3).is_err() {
        return false;
    }
    args.len() <= 3 || tv_check_for_opt_string_arg(args, 4).is_ok()
}

/// Match the error the command reported against the caller's second argument.
///
/// A string must be a substring of it; a one- or two-element list holds
/// patterns, the second of which is matched against `v:errmsg` rather than the
/// raw message.
fn check_reported_error(args: &[TypVal], reported: &CStr) -> FailsCheck {
    let mut buf = NumBuf::new();
    let mismatch = |expected: Option<&CStr>, actual: &CStr| {
        FailsCheck::Mismatch(FailsMismatch {
            expected_str: expected.map(ThinCString::from_cstr),
            index: 1,
            actual: Some(ThinCString::from_cstr(actual)),
        })
    };

    match args.get(1).map_or(VAR_UNKNOWN, TypVal::v_type) {
        VAR_STRING => {
            let expected = buf.string_chk(&args[1]);
            if expected.is_some_and(|expected| has_bytes(reported, expected.to_bytes())) {
                return FailsCheck::Matched;
            }
            mismatch(None, reported)
        }
        VAR_LIST => {
            // The patterns are copied out first: matching one can raise an
            // error, and that runs no user code, but the list is the
            // caller's and nothing here needs it borrowed.
            let patterns: Vec<TypVal> = match args[1].list_ref() {
                Some(list) if (1..=2).contains(&list.len()) => {
                    list.items().iter().map(|item| item.li_tv.clone()).collect()
                }
                _ => return FailsCheck::BadArg(E_ASSERT_FAILS_SECOND_ARG),
            };
            let Some(expected) = buf.string_chk(&patterns[0]) else {
                return FailsCheck::Abandon;
            };
            if !pattern_match(expected, reported, false) {
                return mismatch(Some(expected), reported);
            }
            let Some(second) = patterns.get(1) else {
                return FailsCheck::Matched;
            };
            // Take a copy: an error inside pattern_match() may free it.
            let errmsg = ThinCString::from_vec(vim_var_bytes(Vv::Errmsg));
            let mut buf = NumBuf::new();
            let Some(expected) = buf.string_chk(second) else {
                return FailsCheck::Abandon;
            };
            if pattern_match(expected, &errmsg, false) {
                return FailsCheck::Matched;
            }
            mismatch(Some(expected), &errmsg)
        }
        _ => FailsCheck::BadArg(E_ASSERT_FAILS_SECOND_ARG),
    }
}

/// Match the line number and context the error was reported from against the
/// caller's fourth and fifth arguments.
///
/// A negative line number means "do not check", which is how a test asks only
/// about the context.
fn check_error_position(args: &[TypVal], context: &CStr) -> FailsCheck {
    if args.len() <= 3 {
        return FailsCheck::Matched;
    }
    if !args.get(3).is_some_and(|arg| arg.v_type() == VAR_NUMBER) {
        return FailsCheck::BadArg(E_ASSERT_FAILS_FOURTH_ARGUMENT);
    }
    let want_lnum = args[3].number_or_zero();
    if want_lnum >= 0 && want_lnum != VarNumber::from(emsg_assert_fails_lnum.get()) {
        return FailsCheck::Mismatch(FailsMismatch {
            expected_str: None,
            index: 3,
            actual: None,
        });
    }
    if args.len() <= 4 {
        return FailsCheck::Matched;
    }
    if !args.get(4).is_some_and(|arg| arg.v_type() == VAR_STRING) {
        return FailsCheck::BadArg(E_ASSERT_FAILS_FIFTH_ARGUMENT);
    }
    let want_context = args[4].string_cstr();
    if want_context.is_none_or(|want| pattern_match(want, context, false)) {
        return FailsCheck::Matched;
    }
    FailsCheck::Mismatch(FailsMismatch {
        expected_str: None,
        index: 4,
        actual: None,
    })
}

/// Append a failed `assert_fails()`'s report to `v:errors`.
fn report_fails_mismatch(args: &[TypVal], cmd: &[u8], context: &CStr, mismatch: &FailsMismatch) {
    let actual_tv = match mismatch.index {
        3 => TypVal::Number(VarNumber::from(emsg_assert_fails_lnum.get())),
        4 => TypVal::string_from(context.to_bytes()),
        _ => TypVal::string(mismatch.actual.clone()),
    };
    let expected = match &mismatch.expected_str {
        Some(text) => Expected::Text(text.as_bytes()),
        None => Expected::Value(&args[mismatch.index]),
    };
    let mut message = prepare_assert_error();
    fill_assert_error(
        &mut message,
        args.get(2),
        expected,
        &actual_tv,
        AssertType::Fails,
    );
    push_lit(&mut message, c": ");
    assert_append_cmd_or_arg(&mut message, args, cmd);
    report_assert_error(&message);
}

/// Put the message and screen state back the way `assert_fails()` found it.
///
/// The command it ran was expected to fail, so everything that failure left
/// behind — the error flags, the pending `hit-enter`, `v:errmsg` — is this
/// function's to undo.
fn finish_assert_fails(save_trylevel: c_int, no_prompt: MsgBump) {
    trylevel.set(save_trylevel);
    suppress_errthrow.set(false);
    in_assert_fails.set(false);
    did_emsg.set(0);
    got_int.set(false);
    msg_col.set(0);
    drop(no_prompt);
    need_wait_return.set(false);
    emsg_on_display.set(false);
    msg_reset_scroll();
    lines_left.set(Rows.get());
    emsg_assert_fails_msg.set(None);
    set_vim_var_string(Vv::Errmsg, None);
}

/// `assert_fails(cmd [, error [, msg [, lnum [, context]]]])`.
pub(crate) fn f_assert_fails(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    let mut numbuf = NumBuf::new();
    if !assert_fails_args_ok(args) {
        return;
    }

    let save_trylevel = trylevel.get();
    let called_emsg_before = called_emsg.get();
    let mut wrong_arg_msg: Option<&'static CStr> = None;

    // trylevel must be zero for a ":throw" command to be considered failed.
    trylevel.set(0);
    suppress_errthrow.set(true);
    in_assert_fails.set(true);
    // Threaded into `finish_assert_fails`, which is where the C released
    // it — before the wrong-argument message below, which *does* want the
    // hit-enter prompt.
    let no_prompt = Suppress::wait_return();

    // An argument with no string form has reported itself and runs as the
    // empty command. The text is the argument's own, which the call holds
    // for its whole length, so the command it runs cannot free it.
    let cmd = numbuf.string_chk(&args[0]).unwrap_or(c"");
    let _ = do_cmdline_cmd(cmd);

    // Reset here for any errors reported below.
    trylevel.set(save_trylevel);
    suppress_errthrow.set(false);

    if called_emsg.get() == called_emsg_before {
        let mut message = prepare_assert_error();
        push_lit(&mut message, c"command did not fail: ");
        assert_append_cmd_or_arg(&mut message, args, cmd.to_bytes());
        report_assert_error(&message);
        result.write_number(1);
    } else if args.len() > 1 {
        // Copies: matching a pattern can raise an error of its own, and
        // the reports below borrow them.
        let reported = emsg_assert_fails_msg.with(Clone::clone);
        let reported = reported.as_ref().map_or(c"[unknown]", XString::as_cstr);
        let context = emsg_assert_fails_context.with(Clone::clone);
        let context = context.as_ref().map_or(c"", XString::as_cstr);
        let mut check = check_reported_error(args, reported);
        if matches!(check, FailsCheck::Matched) {
            check = check_error_position(args, context);
        }
        match check {
            FailsCheck::Matched | FailsCheck::Abandon => {}
            FailsCheck::BadArg(msg) => wrong_arg_msg = Some(msg),
            FailsCheck::Mismatch(mismatch) => {
                report_fails_mismatch(args, cmd.to_bytes(), context, &mismatch);
                result.write_number(1);
            }
        }
    }

    finish_assert_fails(save_trylevel, no_prompt);
    if let Some(msg) = wrong_arg_msg {
        emsg(gettext(msg));
    }
}
