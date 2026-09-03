//! `assert_fails()`: run a command and check that it failed, optionally with
//! a particular message, from a particular line, in a particular context.
//!
//! It is the one `assert_*()` that runs user code, so it is also the one that
//! has to put the message state back: the command it ran was *expected* to
//! fail, and everything that failure left behind — the error flags, the
//! pending `hit-enter`, `v:errmsg` — is [`finish_assert_fails`]'s to undo.

#![deny(unsafe_op_in_unsafe_fn)]

use core::ffi::{CStr, c_char, c_int};
use core::ptr;

use crate::eval::pattern_match;
use crate::eval::typval::{
    NumBuf, tv_check_for_opt_number_arg, tv_check_for_opt_string_arg,
    tv_check_for_opt_string_or_list_arg, tv_check_for_string_or_number_arg, tv_get_string_buf_chk,
    tv_list_first, tv_list_last, tv_list_len,
};
use crate::eval::vars::{get_vim_var_str, set_vim_var_string};
use crate::ex_docmd::do_cmdline_cmd;
use crate::guard::{Bump, Suppress};
use crate::main::{
    Rows, called_emsg, did_emsg, emsg_assert_fails_context, emsg_assert_fails_lnum,
    emsg_assert_fails_msg, emsg_on_display, got_int, in_assert_fails, lines_left, msg_col,
    need_wait_return, suppress_errthrow, trylevel,
};
use crate::memory::{xfree, xstrdup};
use crate::message::{emsg, msg_reset_scroll};
use crate::os::cshim::{gettext, strstr};
use crate::types::{
    EvalFuncData, VAR_LIST, VAR_NUMBER, VAR_STRING, VarLock, Vv, list_T, typval_T,
    typval_vval_union, varnumber_T,
};

use super::report::{fill_assert_error, ga_concat_lit, prepare_assert_error, report_assert_error};
use super::{
    AssertType, E_ASSERT_FAILS_FIFTH_ARGUMENT, E_ASSERT_FAILS_FOURTH_ARGUMENT,
    E_ASSERT_FAILS_SECOND_ARG, NUMBUFLEN, arg, arg_given, arg_type, assert_append_cmd_or_arg,
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
    /// The pattern the caller gave, quoted in the message. Null when the
    /// expectation is printed from the argument itself instead.
    expected_str: *const c_char,
    /// Which `argvars` slot the unmet expectation came from: 1, 3 or 4.
    index: usize,
    /// The error text that actually arrived, for `index == 1`.
    actual: *mut c_char,
}

/// Whether `assert_fails()`'s arguments have the shapes it documents.
///
/// The later ones are only checked when the earlier optional ones are present,
/// exactly as upstream: `assert_fails(cmd, err, msg, lnum, context)`.
///
/// # Safety
/// `argvars` has five slots.
unsafe fn assert_fails_args_ok(argvars: *mut typval_T) -> bool {
    // SAFETY: the caller's arguments.
    if unsafe { tv_check_for_string_or_number_arg(argvars, 0) }.is_err()
        || unsafe { tv_check_for_opt_string_or_list_arg(argvars, 1) }.is_err()
    {
        return false;
    }
    if !unsafe { arg_given(argvars, 1) } || !unsafe { arg_given(argvars, 2) } {
        return true;
    }
    if unsafe { tv_check_for_opt_number_arg(argvars, 3) }.is_err() {
        return false;
    }
    !unsafe { arg_given(argvars, 3) } || unsafe { tv_check_for_opt_string_arg(argvars, 4) }.is_ok()
}

/// Match the error the command reported against the caller's second argument.
///
/// A string must be a substring of it; a one- or two-element list holds
/// patterns, the second of which is matched against `v:errmsg` rather than the
/// raw message.
///
/// # Safety
/// `argvars` has five slots; `tofree` receives an allocation the caller frees.
unsafe fn check_reported_error(argvars: *mut typval_T, tofree: &mut *mut c_char) -> FailsCheck {
    let mut buf = [0 as c_char; NUMBUFLEN];
    // SAFETY: the caller's arguments and out-parameter.
    let unknown = c"[unknown]".as_ptr().cast_mut();
    let reported = emsg_assert_fails_msg.get();
    let mut actual = if reported.is_null() {
        unknown
    } else {
        reported
    };

    match unsafe { arg_type(argvars, 1) } {
        VAR_STRING => {
            let expected = unsafe { tv_get_string_buf_chk(arg(argvars, 1), buf.as_mut_ptr()) };
            if !expected.is_null() && !unsafe { strstr(actual, expected) }.is_null() {
                return FailsCheck::Matched;
            }
            FailsCheck::Mismatch(FailsMismatch {
                expected_str: ptr::null(),
                index: 1,
                actual,
            })
        }
        VAR_LIST => {
            let list: *const list_T = unsafe { (*arg(argvars, 1)).vval.v_list };
            if list.is_null() || !(1..=2).contains(&unsafe { tv_list_len(list) }) {
                return FailsCheck::BadArg(E_ASSERT_FAILS_SECOND_ARG);
            }
            let mut tv: *const typval_T = unsafe { &raw mut (*tv_list_first(list)).li_tv };
            let mut expected = unsafe { tv_get_string_buf_chk(tv, buf.as_mut_ptr()) };
            if expected.is_null() {
                return FailsCheck::Abandon;
            }
            if !unsafe { pattern_match(expected, actual, false) } {
                return FailsCheck::Mismatch(FailsMismatch {
                    expected_str: expected,
                    index: 1,
                    actual,
                });
            }
            if unsafe { tv_list_len(list) } != 2 {
                return FailsCheck::Matched;
            }
            // Take a copy: an error inside pattern_match() may free it.
            actual = unsafe { xstrdup(get_vim_var_str(Vv::Errmsg)) };
            *tofree = actual;
            tv = unsafe { &raw mut (*tv_list_last(list)).li_tv };
            expected = unsafe { tv_get_string_buf_chk(tv, buf.as_mut_ptr()) };
            if expected.is_null() {
                return FailsCheck::Abandon;
            }
            if unsafe { pattern_match(expected, actual, false) } {
                return FailsCheck::Matched;
            }
            FailsCheck::Mismatch(FailsMismatch {
                expected_str: expected,
                index: 1,
                actual,
            })
        }
        _ => FailsCheck::BadArg(E_ASSERT_FAILS_SECOND_ARG),
    }
}

/// Match the line number and context the error was reported from against the
/// caller's fourth and fifth arguments.
///
/// A negative line number means "do not check", which is how a test asks only
/// about the context.
///
/// # Safety
/// `argvars` has five slots.
unsafe fn check_error_position(argvars: *mut typval_T) -> FailsCheck {
    // SAFETY: the caller's arguments.
    if !unsafe { arg_given(argvars, 2) } || !unsafe { arg_given(argvars, 3) } {
        return FailsCheck::Matched;
    }
    if unsafe { arg_type(argvars, 3) } != VAR_NUMBER {
        return FailsCheck::BadArg(E_ASSERT_FAILS_FOURTH_ARGUMENT);
    }
    let want_lnum = unsafe { (*arg(argvars, 3)).vval.v_number };
    if want_lnum >= 0 && want_lnum != emsg_assert_fails_lnum.get() as varnumber_T {
        return FailsCheck::Mismatch(FailsMismatch {
            expected_str: ptr::null(),
            index: 3,
            actual: ptr::null_mut(),
        });
    }
    if !unsafe { arg_given(argvars, 4) } {
        return FailsCheck::Matched;
    }
    if unsafe { arg_type(argvars, 4) } != VAR_STRING {
        return FailsCheck::BadArg(E_ASSERT_FAILS_FIFTH_ARGUMENT);
    }
    let want_context = unsafe { (*arg(argvars, 4)).vval.v_string };
    if want_context.is_null()
        || unsafe { pattern_match(want_context, emsg_assert_fails_context.get(), false) }
    {
        return FailsCheck::Matched;
    }
    FailsCheck::Mismatch(FailsMismatch {
        expected_str: ptr::null(),
        index: 4,
        actual: ptr::null_mut(),
    })
}

/// Append a failed `assert_fails()`'s report to `v:errors`.
///
/// # Safety
/// `argvars` has five slots and `cmd` is the command that was run.
unsafe fn report_fails_mismatch(
    argvars: *mut typval_T,
    cmd: *const c_char,
    mismatch: &FailsMismatch,
) {
    // SAFETY: the caller's arguments; `actual_tv` borrows and is never cleared.
    let mut actual_tv = match mismatch.index {
        3 => typval_T {
            v_type: VAR_NUMBER,
            v_lock: VarLock::Unlocked,
            vval: typval_vval_union {
                v_number: emsg_assert_fails_lnum.get() as varnumber_T,
            },
        },
        4 => typval_T {
            v_type: VAR_STRING,
            v_lock: VarLock::Unlocked,
            vval: typval_vval_union {
                v_string: emsg_assert_fails_context.get(),
            },
        },
        _ => typval_T {
            v_type: VAR_STRING,
            v_lock: VarLock::Unlocked,
            vval: typval_vval_union {
                v_string: mismatch.actual,
            },
        },
    };
    let mut ga = unsafe { prepare_assert_error() };
    let gap = &mut ga;
    unsafe {
        fill_assert_error(
            gap,
            arg(argvars, 2),
            mismatch.expected_str,
            arg(argvars, mismatch.index),
            &raw mut actual_tv,
            AssertType::Fails,
        )
    };
    ga_concat_lit(gap, c": ");
    unsafe { assert_append_cmd_or_arg(gap, argvars, cmd) };
    report_assert_error(gap);
}

/// Put the message and screen state back the way `assert_fails()` found it.
///
/// The command it ran was expected to fail, so everything that failure left
/// behind — the error flags, the pending `hit-enter`, `v:errmsg` — is this
/// function's to undo.
///
/// # Safety
/// Called once, at the end of `assert_fails()`.
unsafe fn finish_assert_fails(save_trylevel: c_int, tofree: *mut c_char, no_prompt: Bump) {
    trylevel.set(save_trylevel);
    suppress_errthrow.set(false);
    in_assert_fails.set(false);
    did_emsg.set(0);
    got_int.set(false);
    msg_col.set(0);
    drop(no_prompt);
    need_wait_return.set(false);
    emsg_on_display.set(false);
    // SAFETY: the two allocations belong to this call.
    unsafe { msg_reset_scroll() };
    lines_left.set(Rows.get());
    unsafe { xfree(emsg_assert_fails_msg.get().cast()) };
    emsg_assert_fails_msg.set(ptr::null_mut());
    unsafe { xfree(tofree.cast()) };
    unsafe { set_vim_var_string(Vv::Errmsg, ptr::null(), 0) };
}

/// `assert_fails(cmd [, error [, msg [, lnum [, context]]]])`.
pub(crate) unsafe fn f_assert_fails(
    argvars: *mut typval_T,
    rettv: *mut typval_T,
    _fptr: EvalFuncData,
) {
    let mut numbuf = NumBuf::new();
    // SAFETY: the evaluator's argument vector and return slot. `do_cmdline_cmd`
    // runs user code that is expected to fail; every flag disturbed for it is
    // restored by `finish_assert_fails`.
    if !unsafe { assert_fails_args_ok(argvars) } {
        return;
    }

    let save_trylevel = trylevel.get();
    let called_emsg_before = called_emsg.get();
    let mut tofree: *mut c_char = ptr::null_mut();
    let mut wrong_arg_msg: Option<&'static CStr> = None;

    // trylevel must be zero for a ":throw" command to be considered failed.
    trylevel.set(0);
    suppress_errthrow.set(true);
    in_assert_fails.set(true);
    // Threaded into `finish_assert_fails`, which is where the C released
    // it — before the wrong-argument message below, which *does* want the
    // hit-enter prompt.
    let no_prompt = Suppress::wait_return();

    let cmd = unsafe { numbuf.string_chk(arg(argvars, 0)) };
    let _ = unsafe { do_cmdline_cmd(cmd) };

    // Reset here for any errors reported below.
    trylevel.set(save_trylevel);
    suppress_errthrow.set(false);

    if called_emsg.get() == called_emsg_before {
        let mut ga = unsafe { prepare_assert_error() };
        ga_concat_lit(&mut ga, c"command did not fail: ");
        unsafe { assert_append_cmd_or_arg(&mut ga, argvars, cmd) };
        report_assert_error(&ga);
        unsafe { (*rettv).vval.v_number = 1 };
    } else if unsafe { arg_given(argvars, 1) } {
        let mut check = unsafe { check_reported_error(argvars, &mut tofree) };
        if matches!(check, FailsCheck::Matched) {
            check = unsafe { check_error_position(argvars) };
        }
        match check {
            FailsCheck::Matched | FailsCheck::Abandon => {}
            FailsCheck::BadArg(msg) => wrong_arg_msg = Some(msg),
            FailsCheck::Mismatch(mismatch) => {
                unsafe { report_fails_mismatch(argvars, cmd, &mismatch) };
                unsafe { (*rettv).vval.v_number = 1 };
            }
        }
    }

    unsafe { finish_assert_fails(save_trylevel, tofree, no_prompt) };
    if let Some(msg) = wrong_arg_msg {
        emsg(gettext(msg));
    }
}
