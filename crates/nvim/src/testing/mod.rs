//! The `assert_*()` builtins, and the two `test_*()` ones: Vimscript's own
//! test harness, which the legacy Vim suite is written on top of.
//!
//! Each `assert_*()` answers 0 when the check holds and 1 when it does not,
//! and a failing one appends one line to `v:errors` describing what was
//! expected and what arrived. That line is this module's real output — its
//! wording, its escaping and its `- N equal items omitted` tail are matched
//! on by tests, so the phrasing here is load-bearing and none of it may drift.
//!
//! Every message is built the same way: [`prepare_assert_error`] opens a
//! buffer with the sourcing position, [`fill_assert_error`] (or a literal)
//! describes the failure, and [`report_assert_error`] publishes and releases
//! it.
//!
//! Ported from the C in `src/nvim/testing.c`.

#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use crate::memory::ThinCString;
use crate::strings::has_bytes;
use core::ffi::CStr;
use std::fs::File;
use std::io::{BufReader, Bytes, Read};
use std::os::unix::ffi::OsStrExt;

use crate::eval::encode::{encode_tv2echo, push_float_g};
use crate::eval::typval::{
    NumBuf, tv_check_for_float_or_nr_arg, tv_check_for_opt_string_arg, tv_equal, tv_get_float,
    tv_get_number_chk,
};
use crate::eval::vars::{testing_enabled, vim_var_string, with_vim_var};
use crate::eval::{garbage_collect, pattern_match};
use crate::ex_docmd::do_cmdline_cmd;
use crate::ex_eval::state::suppress_errthrow;
use crate::message::e_cant_read_file_str;
use crate::message::emsg;
use crate::message::state::{emsg_on_display, emsg_silent};
use crate::os::cshim::gettext;
use crate::types::{
    BoolVarValue, EStackArg, EvalFuncData, TypVal, VAR_FLOAT, VAR_NUMBER, VarNumber, Vv,
    kBoolVarFalse, kBoolVarTrue,
};
use crate::ui::state::called_vim_beep;

/// Which `assert_*()` is reporting. Decides the wording of the message.
#[derive(Clone, Copy, PartialEq, Eq)]
enum AssertType {
    Equal,
    NotEqual,
    Match,
    NotMatch,
    /// `assert_fails()`, whose expectation is quoted in the message.
    Fails,
    /// Everything else, which gets the plain `Expected … but got …` wording.
    Other,
}

/// `ESTACK_NONE`: `estack_sfile()` wants no `<sfile>`-style expansion.
const ESTACK_NONE: EStackArg = 0;
const E_ASSERT_FAILS_SECOND_ARG: &CStr =
    c"E856: \"assert_fails()\" second argument must be a string or a list with one or two strings";
const E_ASSERT_FAILS_FOURTH_ARGUMENT: &CStr =
    c"E1115: \"assert_fails()\" fourth argument must be a number";
const E_ASSERT_FAILS_FIFTH_ARGUMENT: &CStr =
    c"E1116: \"assert_fails()\" fifth argument must be a string";
const E_TEST_GARBAGECOLLECT_NOW: &CStr =
    c"E1142: Calling test_garbagecollect_now() while v:testing is not set";

mod fails;
mod report;
#[cfg(test)]
mod tests;

pub(crate) use fails::f_assert_fails;
use report::{Expected, fill_assert_error, prepare_assert_error, push_lit, report_assert_error};

// ---------------------------------------------------------------------------
// The checks
// ---------------------------------------------------------------------------

/// `assert_equal()` and `assert_notequal()`.
fn assert_equal_common(args: &[TypVal], atype: AssertType) -> bool {
    if tv_equal(&args[0], &args[1], false) == (atype == AssertType::Equal) {
        return true;
    }
    let mut message = prepare_assert_error();
    let expected = Expected::Value(&args[0]);
    fill_assert_error(&mut message, args.get(2), expected, &args[1], atype);
    report_assert_error(&message);
    false
}

/// `assert_match()` and `assert_notmatch()`.
fn assert_match_common(args: &[TypVal], atype: AssertType) -> bool {
    let mut buf1 = NumBuf::new();
    let mut buf2 = NumBuf::new();
    // Both arguments are read, so two bad ones report two errors.
    let pat = buf1.string_chk(&args[0]);
    let text = buf2.string_chk(&args[1]);
    let (Some(pat), Some(text)) = (pat, text) else {
        return true;
    };
    if pattern_match(pat, text, false) == (atype == AssertType::Match) {
        return true;
    }
    let mut message = prepare_assert_error();
    let expected = Expected::Value(&args[0]);
    fill_assert_error(&mut message, args.get(2), expected, &args[1], atype);
    report_assert_error(&message);
    false
}

/// `assert_true()` and `assert_false()`.
///
/// A number is truthy when non-zero; a `v:true`/`v:false` must match exactly.
/// Anything else fails both.
fn assert_bool(args: &[TypVal], is_true: bool) -> bool {
    let actual = &args[0];
    let number_ok = actual.v_type() == VAR_NUMBER
        && tv_get_number_chk(actual).is_ok_and(|n| (n == 0) != is_true);
    let want = (if is_true { kBoolVarTrue } else { kBoolVarFalse }) as BoolVarValue;
    let bool_ok = actual.as_bool() == Some(want);
    if number_ok || bool_ok {
        return true;
    }
    let mut message = prepare_assert_error();
    let expected: &[u8] = if is_true { b"True" } else { b"False" };
    fill_assert_error(
        &mut message,
        args.get(1),
        Expected::Text(expected),
        actual,
        AssertType::Other,
    );
    report_assert_error(&message);
    false
}

/// Name the command a failed `assert_beeps()`/`assert_fails()` ran.
///
/// With both optional arguments present the caller's own third argument names
/// it instead, which is how a test labels a command that is unreadable.
fn assert_append_cmd_or_arg(message: &mut Vec<u8>, args: &[TypVal], cmd: &[u8]) {
    match args.get(2) {
        Some(label) => message.extend_from_slice(encode_tv2echo(label).as_bytes()),
        None => message.extend_from_slice(cmd),
    }
}

/// `assert_beeps()` (`no_beep` false) and `assert_nobeep()` (true).
fn assert_beeps(args: &[TypVal], no_beep: bool) -> bool {
    let mut numbuf = NumBuf::new();
    // `do_cmdline_cmd` runs user code, which is the whole point, and the
    // flags around it are restored below. An argument with no string form
    // has reported itself and runs as the empty command. The text is the
    // argument's own, which the call holds for its whole length.
    let cmd = numbuf.string_chk(&args[0]).unwrap_or(c"");
    called_vim_beep.set(false);
    suppress_errthrow.set(true);
    emsg_silent.set(0);
    let _ = do_cmdline_cmd(cmd);

    let mut held = true;
    if called_vim_beep.get() == no_beep {
        let mut message = prepare_assert_error();
        push_lit(
            &mut message,
            if no_beep {
                c"command did beep: "
            } else {
                c"command did not beep: "
            },
        );
        message.extend_from_slice(cmd.to_bytes());
        report_assert_error(&message);
        held = false;
    }

    suppress_errthrow.set(false);
    emsg_on_display.set(false);
    held
}

/// The first difference between two files, as `assert_equalfile()` words it,
/// plus the tail of the line it was on.
struct FileDiff {
    /// The verdict, e.g. `difference at byte 3, line 1`. Empty means equal.
    verdict: Vec<u8>,
    /// The last bytes read from each file on the line of the difference, up
    /// to and including it.
    line1: Vec<u8>,
    line2: Vec<u8>,
}

/// The next byte of a stream, or `None` at its end — or on a read error,
/// which is where `fgetc` answered `EOF` too.
fn next_byte(bytes: &mut Bytes<BufReader<File>>) -> Option<u8> {
    bytes.next().and_then(Result::ok)
}

/// Compare the two files byte by byte.
fn compare_files(fname1: &CStr, fname2: &CStr) -> FileDiff {
    let mut diff = FileDiff {
        verdict: Vec::new(),
        line1: Vec::new(),
        line2: Vec::new(),
    };
    let open = |fname: &CStr| {
        File::open(std::ffi::OsStr::from_bytes(fname.to_bytes()))
            .map(|file| BufReader::new(file).bytes())
    };
    let cant_read = |fname: &CStr| {
        // The message names the file at its one `%s`.
        let format = gettext(e_cant_read_file_str).to_bytes();
        let mut verdict = Vec::with_capacity(format.len() + fname.to_bytes().len());
        match format.windows(2).position(|pair| pair == b"%s") {
            Some(at) => {
                verdict.extend_from_slice(&format[..at]);
                verdict.extend_from_slice(fname.to_bytes());
                verdict.extend_from_slice(&format[at + 2..]);
            }
            None => verdict.extend_from_slice(format),
        }
        verdict
    };
    let Ok(mut file1) = open(fname1) else {
        diff.verdict = cant_read(fname1);
        return diff;
    };
    let Ok(mut file2) = open(fname2) else {
        diff.verdict = cant_read(fname2);
        return diff;
    };

    let mut linecount: i64 = 1;
    let mut count: i64 = 0;
    loop {
        let c1 = next_byte(&mut file1);
        let c2 = next_byte(&mut file2);
        let (c1, c2) = match (c1, c2) {
            (None, None) => break,
            (None, Some(_)) => {
                diff.verdict = b"first file is shorter".to_vec();
                break;
            }
            (Some(_), None) => {
                diff.verdict = b"second file is shorter".to_vec();
                break;
            }
            (Some(c1), Some(c2)) => (c1, c2),
        };
        diff.line1.push(c1);
        diff.line2.push(c2);
        if c1 != c2 {
            diff.verdict = format!("difference at byte {count}, line {linecount}").into_bytes();
            break;
        }
        if c1 == b'\n' {
            linecount += 1;
            diff.line1.clear();
            diff.line2.clear();
        } else if diff.line1.len() == 198 {
            // Keep only the last 98 bytes of an over-long line.
            diff.line1.drain(..100);
            diff.line2.drain(..100);
        }
        count += 1;
    }
    diff
}

/// `assert_equalfile()`.
fn assert_equalfile(args: &[TypVal]) -> bool {
    let mut buf1 = NumBuf::new();
    let mut buf2 = NumBuf::new();
    // Both arguments are read, so two bad ones report two errors.
    let fname1 = buf1.string_chk(&args[0]);
    let fname2 = buf2.string_chk(&args[1]);
    let (Some(fname1), Some(fname2)) = (fname1, fname2) else {
        return true;
    };

    let diff = compare_files(fname1, fname2);
    if diff.verdict.is_empty() {
        return true;
    }

    let mut message = prepare_assert_error();
    if let Some(label) = args.get(2) {
        message.extend_from_slice(encode_tv2echo(label).as_bytes());
        push_lit(&mut message, c": ");
    }
    message.extend_from_slice(&diff.verdict);
    if !diff.line1.is_empty() {
        // The lines go in whole, but are compared as far as their first
        // NUL, as the C strings the comparison read them as.
        fn until_nul(line: &[u8]) -> &[u8] {
            line.split(|&byte| byte == 0).next().unwrap_or_default()
        }
        push_lit(&mut message, c" after \"");
        message.extend_from_slice(&diff.line1);
        if until_nul(&diff.line1) != until_nul(&diff.line2) {
            push_lit(&mut message, c"\" vs \"");
            message.extend_from_slice(&diff.line2);
        }
        push_lit(&mut message, c"\"");
    }
    report_assert_error(&message);
    false
}

/// `assert_inrange()`. Floats and integers are compared and printed
/// differently, so the two halves are separate.
fn assert_inrange(args: &[TypVal]) -> bool {
    let mut expected = Vec::new();
    if (0..3).any(|i| args.get(i).is_some_and(|arg| arg.v_type() == VAR_FLOAT)) {
        let lower = tv_get_float(&args[0]);
        let upper = tv_get_float(&args[1]);
        let actual = tv_get_float(&args[2]);
        // Written as upstream does, so a NaN — which compares false both
        // ways — is in range rather than out of it.
        if !(actual < lower || actual > upper) {
            return true;
        }
        expected.extend_from_slice(b"range ");
        push_float_g(&mut expected, lower);
        expected.extend_from_slice(b" - ");
        push_float_g(&mut expected, upper);
        expected.push(b',');
    } else {
        // All three are read, in this order, whatever the first answers:
        // each reports its own message.
        let bounds = (
            tv_get_number_chk(&args[0]),
            tv_get_number_chk(&args[1]),
            tv_get_number_chk(&args[2]),
        );
        let (Ok(lower), Ok(upper), Ok(actual)) = bounds else {
            return true;
        };
        if !(actual < lower || actual > upper) {
            return true;
        }
        expected.extend_from_slice(format!("range {lower} - {upper},").as_bytes());
    }

    let mut message = prepare_assert_error();
    fill_assert_error(
        &mut message,
        args.get(3),
        Expected::Text(&expected),
        &args[2],
        AssertType::Other,
    );
    report_assert_error(&message);
    false
}

/// What an `assert_*()` answers: 0 when the check held, 1 when it did not.
fn verdict(held: bool) -> VarNumber {
    VarNumber::from(!held)
}

// ---------------------------------------------------------------------------
// The builtins
// ---------------------------------------------------------------------------

/// `assert_beeps(cmd)`.
pub(crate) fn f_assert_beeps(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    result.write_number(verdict(assert_beeps(args, false)));
}

/// `assert_nobeep(cmd)`.
pub(crate) fn f_assert_nobeep(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    result.write_number(verdict(assert_beeps(args, true)));
}

/// `assert_equal(expected, actual[, msg])`.
pub(crate) fn f_assert_equal(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    result.write_number(verdict(assert_equal_common(args, AssertType::Equal)));
}

/// `assert_notequal(expected, actual[, msg])`.
pub(crate) fn f_assert_notequal(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    result.write_number(verdict(assert_equal_common(args, AssertType::NotEqual)));
}

/// `assert_equalfile(fname-one, fname-two[, msg])`.
pub(crate) fn f_assert_equalfile(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    result.write_number(verdict(assert_equalfile(args)));
}

/// `assert_exception(string[, msg])`.
pub(crate) fn f_assert_exception(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    let mut numbuf = NumBuf::new();
    let error = numbuf.string_chk(&args[0]);
    let thrown = vim_var_string(Vv::Exception).unwrap_or_else(ThinCString::empty);
    let thrown: &CStr = &thrown;
    if thrown.is_empty() {
        let mut message = prepare_assert_error();
        push_lit(&mut message, c"v:exception is not set");
        report_assert_error(&message);
        result.write_number(1);
    } else if error.is_some_and(|error| !has_bytes(thrown, error.to_bytes())) {
        let mut message = prepare_assert_error();
        // A copy: the report is not a leaf the variable may be lent to.
        let exception = with_vim_var(Vv::Exception, TypVal::clone);
        fill_assert_error(
            &mut message,
            args.get(1),
            Expected::Value(&args[0]),
            &exception,
            AssertType::Other,
        );
        report_assert_error(&message);
        result.write_number(1);
    }
}

/// `assert_false(actual[, msg])`.
pub(crate) fn f_assert_false(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    result.write_number(verdict(assert_bool(args, false)));
}

/// `assert_true(actual[, msg])`.
pub(crate) fn f_assert_true(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    result.write_number(verdict(assert_bool(args, true)));
}

/// `assert_inrange(lower, upper, actual[, msg])`.
pub(crate) fn f_assert_inrange(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    if tv_check_for_float_or_nr_arg(args, 0).is_err()
        || tv_check_for_float_or_nr_arg(args, 1).is_err()
        || tv_check_for_float_or_nr_arg(args, 2).is_err()
        || tv_check_for_opt_string_arg(args, 3).is_err()
    {
        return;
    }
    result.write_number(verdict(assert_inrange(args)));
}

/// `assert_match(pattern, actual[, msg])`.
pub(crate) fn f_assert_match(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    result.write_number(verdict(assert_match_common(args, AssertType::Match)));
}

/// `assert_notmatch(pattern, actual[, msg])`.
pub(crate) fn f_assert_notmatch(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    result.write_number(verdict(assert_match_common(args, AssertType::NotMatch)));
}

/// `assert_report(msg)`: an unconditional failure.
pub(crate) fn f_assert_report(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    let mut numbuf = NumBuf::new();
    let mut message = prepare_assert_error();
    message.extend_from_slice(numbuf.bytes(&args[0]));
    report_assert_error(&message);
    result.write_number(1);
}

/// `test_garbagecollect_now()`: collect immediately rather than at the next
/// safe point.
///
/// This is dangerous — any list or dict held only by internal C state is freed
/// while still in use — so it is refused unless `v:testing` says the caller
/// meant it.
pub(crate) fn f_test_garbagecollect_now(
    _args: &[TypVal],
    _result: &mut TypVal,
    _fptr: EvalFuncData,
) {
    if !testing_enabled() {
        emsg(gettext(E_TEST_GARBAGECOLLECT_NOW));
    } else {
        garbage_collect(true);
    }
}

/// `test_write_list_log(fname)`: a no-op.
///
/// Upstream keeps the builtin so scripts that call it still parse, but the
/// list-allocation log it wrote is only compiled in under a debug define that
/// no shipped build sets. The argument is still read, so a bad one is still
/// reported.
pub(crate) fn f_test_write_list_log(args: &[TypVal], _result: &mut TypVal, _fptr: EvalFuncData) {
    let mut numbuf = NumBuf::new();
    let _ = numbuf.string_chk(&args[0]);
}
