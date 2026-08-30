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

#![deny(unsafe_op_in_unsafe_fn)]

use core::ffi::{CStr, c_char, c_int};
use core::ptr;

use crate::eval::encode::encode_tv2echo;
use crate::eval::typval::{
    NumBuf, tv_check_for_float_or_nr_arg, tv_check_for_opt_string_arg, tv_equal, tv_get_float,
    tv_get_number_chk, tv_get_string_buf_chk,
};
use crate::eval::vars::{get_vim_var_nr, get_vim_var_str, get_vim_var_tv};
use crate::eval::{garbage_collect, pattern_match};
use crate::ex_docmd::do_cmdline_cmd;
use crate::garray::{ga_concat, ga_concat_len};
use crate::main::{
    called_vim_beep, e_cant_read_file_str, emsg_on_display, emsg_silent, suppress_errthrow,
};
use crate::memory::{xfree, xstrlcpy};
use crate::message::emsg;
use crate::os::cshim::{gettext, strstr};
use crate::os::fs::os_fopen;
use crate::strings::{vim_snprintf, vim_snprintf_safelen};
use crate::types::{
    BoolVarValue, EvalFuncData, FILE, IOSIZE, READBIN, VAR_BOOL, VAR_FLOAT, VAR_NUMBER,
    VAR_UNKNOWN, VarType, Vv, estack_arg_T, float_T, garray_T, int64_t, kBoolVarFalse,
    kBoolVarTrue, ptrdiff_t, size_t, typval_T, varnumber_T,
};
use ::libc::{fclose, fgetc, strcmp};

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
const ESTACK_NONE: estack_arg_T = 0;
/// `NUMBUFLEN`: the scratch buffer the `tv_get_string_buf_chk` family wants,
/// and what a formatted number goes into.
const NUMBUFLEN: usize = 65;

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

pub(crate) use fails::f_assert_fails;
use report::{fill_assert_error, ga_concat_lit, prepare_assert_error, report_assert_error};

// ---------------------------------------------------------------------------
// Argument and buffer helpers
// ---------------------------------------------------------------------------

/// `argvars[i]`.
///
/// # Safety
/// `argvars` has at least `i + 1` slots, which every builtin's declared
/// maximum arity guarantees (missing ones are `VAR_UNKNOWN`).
unsafe fn arg(argvars: *mut typval_T, i: usize) -> *mut typval_T {
    // SAFETY: the caller's argument vector.
    unsafe { argvars.add(i) }
}

/// The type of `argvars[i]`.
///
/// # Safety
/// As [`arg`].
unsafe fn arg_type(argvars: *mut typval_T, i: usize) -> VarType {
    // SAFETY: the caller's argument vector.
    unsafe { (*arg(argvars, i)).v_type }
}

/// Whether `argvars[i]` was supplied at all.
///
/// # Safety
/// As [`arg`].
unsafe fn arg_given(argvars: *mut typval_T, i: usize) -> bool {
    // SAFETY: the caller's argument vector.
    unsafe { arg_type(argvars, i) != VAR_UNKNOWN }
}

// ---------------------------------------------------------------------------
// The checks
// ---------------------------------------------------------------------------

/// `assert_equal()` and `assert_notequal()`.
///
/// # Safety
/// `argvars` has three slots.
unsafe fn assert_equal_common(argvars: *mut typval_T, atype: AssertType) -> c_int {
    // SAFETY: the caller's arguments.
    if unsafe { tv_equal(arg(argvars, 0), arg(argvars, 1), false) } == (atype == AssertType::Equal)
    {
        return 0;
    }
    let mut ga = unsafe { prepare_assert_error() };
    unsafe {
        fill_assert_error(
            &raw mut ga,
            arg(argvars, 2),
            ptr::null(),
            arg(argvars, 0),
            arg(argvars, 1),
            atype,
        )
    };
    unsafe { report_assert_error(&raw mut ga) };
    1
}

/// `assert_match()` and `assert_notmatch()`.
///
/// # Safety
/// `argvars` has three slots.
unsafe fn assert_match_common(argvars: *mut typval_T, atype: AssertType) -> c_int {
    let mut buf1 = [0 as c_char; NUMBUFLEN];
    let mut buf2 = [0 as c_char; NUMBUFLEN];
    // SAFETY: the caller's arguments, and two scratch buffers of the size the
    // `_buf_chk` contract asks for.
    let pat = unsafe { tv_get_string_buf_chk(arg(argvars, 0), buf1.as_mut_ptr()) };
    let text = unsafe { tv_get_string_buf_chk(arg(argvars, 1), buf2.as_mut_ptr()) };
    if pat.is_null()
        || text.is_null()
        || unsafe { pattern_match(pat, text, false) } == (atype == AssertType::Match)
    {
        return 0;
    }
    let mut ga = unsafe { prepare_assert_error() };
    unsafe {
        fill_assert_error(
            &raw mut ga,
            arg(argvars, 2),
            ptr::null(),
            arg(argvars, 0),
            arg(argvars, 1),
            atype,
        )
    };
    unsafe { report_assert_error(&raw mut ga) };
    1
}

/// `assert_true()` and `assert_false()`.
///
/// A number is truthy when non-zero; a `v:true`/`v:false` must match exactly.
/// Anything else fails both.
///
/// # Safety
/// `argvars` has two slots.
unsafe fn assert_bool(argvars: *mut typval_T, is_true: bool) -> c_int {
    let mut error = false;
    // SAFETY: the caller's arguments.
    let actual = unsafe { &*arg(argvars, 0) };
    let number_ok = actual.v_type == VAR_NUMBER
        && (unsafe { tv_get_number_chk(arg(argvars, 0), &raw mut error) } == 0) != is_true
        && !error;
    let want = (if is_true { kBoolVarTrue } else { kBoolVarFalse }) as BoolVarValue;
    let bool_ok = actual.v_type == VAR_BOOL && unsafe { actual.vval.v_bool } == want;
    if number_ok || bool_ok {
        return 0;
    }
    let mut ga = unsafe { prepare_assert_error() };
    unsafe {
        fill_assert_error(
            &raw mut ga,
            arg(argvars, 1),
            (if is_true { c"True" } else { c"False" }).as_ptr(),
            ptr::null_mut(),
            arg(argvars, 0),
            AssertType::Other,
        )
    };
    unsafe { report_assert_error(&raw mut ga) };
    1
}

/// Name the command a failed `assert_beeps()`/`assert_fails()` ran.
///
/// With both optional arguments present the caller's own third argument names
/// it instead, which is how a test labels a command that is unreadable.
///
/// # Safety
/// `gap` is open and `argvars` has three slots.
unsafe fn assert_append_cmd_or_arg(gap: *mut garray_T, argvars: *mut typval_T, cmd: *const c_char) {
    // SAFETY: the caller's garray and arguments.
    if unsafe { arg_given(argvars, 1) } && unsafe { arg_given(argvars, 2) } {
        let tofree = unsafe { encode_tv2echo(arg(argvars, 2), ptr::null_mut()) };
        unsafe { ga_concat(gap, tofree) };
        unsafe { xfree(tofree.cast()) };
    } else {
        unsafe { ga_concat(gap, cmd) };
    }
}

/// `assert_beeps()` (`no_beep` false) and `assert_nobeep()` (true).
///
/// # Safety
/// `argvars` has one slot.
unsafe fn assert_beeps(argvars: *mut typval_T, no_beep: bool) -> c_int {
    let mut numbuf = NumBuf::new();
    // SAFETY: the caller's arguments; `do_cmdline_cmd` runs user code, which
    // is the whole point, and the flags around it are restored below.
    let cmd = unsafe { numbuf.string_chk(arg(argvars, 0)) };
    called_vim_beep.set(false);
    suppress_errthrow.set(true);
    emsg_silent.set(0);
    let _ = unsafe { do_cmdline_cmd(cmd) };

    let mut ret = 0;
    if called_vim_beep.get() == no_beep {
        let mut ga = unsafe { prepare_assert_error() };
        unsafe {
            ga_concat_lit(
                &raw mut ga,
                if no_beep {
                    c"command did beep: "
                } else {
                    c"command did not beep: "
                },
            )
        };
        unsafe { ga_concat(&raw mut ga, cmd) };
        unsafe { report_assert_error(&raw mut ga) };
        ret = 1;
    }

    suppress_errthrow.set(false);
    emsg_on_display.set(false);
    ret
}

/// The first difference between two files, as `assert_equalfile()` words it,
/// plus the tail of the line it was on.
struct FileDiff {
    /// The verdict, e.g. `difference at byte 3, line 1`. Empty means equal.
    verdict: [c_char; IOSIZE as usize],
    verdict_len: size_t,
    /// The last bytes read from each file, up to the difference.
    line1: [c_char; 200],
    line2: [c_char; 200],
    lineidx: ptrdiff_t,
}

/// Compare the two files byte by byte.
///
/// Upstream formats the verdict into the shared `IObuff`; it is a local here,
/// because it is read back *after* `prepare_assert_error()` and
/// `encode_tv2echo()` have run, either of which may format a message of its
/// own through the same buffer.
///
/// # Safety
/// Both names are C strings.
unsafe fn compare_files(fname1: *const c_char, fname2: *const c_char) -> FileDiff {
    let mut diff = FileDiff {
        verdict: [0; IOSIZE as usize],
        verdict_len: 0,
        line1: [0; 200],
        line2: [0; 200],
        lineidx: 0,
    };
    const EOF: c_int = -1;

    // SAFETY: the caller's names; every stream opened here is closed here, and
    // `lineidx` is kept below `line1.len() - 1` by the shift below.
    let cant_read = e_cant_read_file_str.as_ptr();
    let fd1: *mut FILE = unsafe { os_fopen(fname1, READBIN.as_ptr()) };
    if fd1.is_null() {
        diff.verdict_len = unsafe {
            vim_snprintf_safelen(
                diff.verdict.as_mut_ptr(),
                IOSIZE as usize,
                cant_read,
                fname1,
            )
        };
        return diff;
    }
    let fd2: *mut FILE = unsafe { os_fopen(fname2, READBIN.as_ptr()) };
    if fd2.is_null() {
        unsafe { fclose(fd1) };
        diff.verdict_len = unsafe {
            vim_snprintf_safelen(
                diff.verdict.as_mut_ptr(),
                IOSIZE as usize,
                cant_read,
                fname2,
            )
        };
        return diff;
    }

    let mut linecount: int64_t = 1;
    let mut count: int64_t = 0;
    loop {
        let c1 = unsafe { fgetc(fd1) };
        let c2 = unsafe { fgetc(fd2) };
        if c1 == EOF {
            if c2 != EOF {
                diff.verdict_len = unsafe {
                    xstrlcpy(
                        diff.verdict.as_mut_ptr(),
                        c"first file is shorter".as_ptr(),
                        IOSIZE as usize,
                    )
                };
            }
            break;
        }
        if c2 == EOF {
            diff.verdict_len = unsafe {
                xstrlcpy(
                    diff.verdict.as_mut_ptr(),
                    c"second file is shorter".as_ptr(),
                    IOSIZE as usize,
                )
            };
            break;
        }
        diff.line1[diff.lineidx as usize] = c1 as c_char;
        diff.line2[diff.lineidx as usize] = c2 as c_char;
        diff.lineidx += 1;
        if c1 != c2 {
            diff.verdict_len = unsafe {
                vim_snprintf_safelen(
                    diff.verdict.as_mut_ptr(),
                    IOSIZE as usize,
                    c"difference at byte %ld, line %ld".as_ptr(),
                    count,
                    linecount,
                )
            };
            break;
        }
        if c1 == b'\n' as c_int {
            linecount += 1;
            diff.lineidx = 0;
        } else if diff.lineidx + 2 == diff.line1.len() as ptrdiff_t {
            // Keep only the last 98 bytes of an over-long line.
            let tail = 100..diff.lineidx as usize;
            diff.line1.copy_within(tail.clone(), 0);
            diff.line2.copy_within(tail, 0);
            diff.lineidx -= 100;
        }
        count += 1;
    }
    unsafe { fclose(fd1) };
    unsafe { fclose(fd2) };
    diff
}

/// `assert_equalfile()`.
///
/// # Safety
/// `argvars` has three slots.
unsafe fn assert_equalfile(argvars: *mut typval_T) -> c_int {
    let mut buf1 = [0 as c_char; NUMBUFLEN];
    let mut buf2 = [0 as c_char; NUMBUFLEN];
    // SAFETY: the caller's arguments and two scratch buffers of the size the
    // `_buf_chk` contract asks for.
    let fname1 = unsafe { tv_get_string_buf_chk(arg(argvars, 0), buf1.as_mut_ptr()) };
    let fname2 = unsafe { tv_get_string_buf_chk(arg(argvars, 1), buf2.as_mut_ptr()) };
    if fname1.is_null() || fname2.is_null() {
        return 0;
    }

    let mut diff = unsafe { compare_files(fname1, fname2) };
    if diff.verdict_len == 0 {
        return 0;
    }

    let mut ga = unsafe { prepare_assert_error() };
    let gap = &raw mut ga;
    if unsafe { arg_given(argvars, 2) } {
        let tofree = unsafe { encode_tv2echo(arg(argvars, 2), ptr::null_mut()) };
        unsafe { ga_concat(gap, tofree) };
        unsafe { xfree(tofree.cast()) };
        unsafe { ga_concat_lit(gap, c": ") };
    }
    unsafe { ga_concat_len(gap, diff.verdict.as_ptr(), diff.verdict_len) };
    if diff.lineidx > 0 {
        let idx = diff.lineidx as usize;
        diff.line1[idx] = 0;
        diff.line2[idx] = 0;
        unsafe { ga_concat_lit(gap, c" after \"") };
        unsafe { ga_concat_len(gap, diff.line1.as_ptr(), idx) };
        if unsafe { strcmp(diff.line1.as_ptr(), diff.line2.as_ptr()) } != 0 {
            unsafe { ga_concat_lit(gap, c"\" vs \"") };
            unsafe { ga_concat_len(gap, diff.line2.as_ptr(), idx) };
        }
        unsafe { ga_concat_lit(gap, c"\"") };
    }
    unsafe { report_assert_error(gap) };
    1
}

/// `assert_inrange()`. Floats and integers are compared and printed
/// differently, so the two halves are separate.
///
/// # Safety
/// `argvars` has four slots.
unsafe fn assert_inrange(argvars: *mut typval_T) -> c_int {
    let mut expected = [0 as c_char; 200];
    // SAFETY: the caller's arguments, and a scratch buffer `vim_snprintf`
    // never writes past.
    if (0..3).any(|i| unsafe { arg_type(argvars, i) } == VAR_FLOAT) {
        let lower = unsafe { tv_get_float(arg(argvars, 0)) };
        let upper = unsafe { tv_get_float(arg(argvars, 1)) };
        let actual: float_T = unsafe { tv_get_float(arg(argvars, 2)) };
        // Written as upstream does, so a NaN — which compares false both
        // ways — is in range rather than out of it.
        if !(actual < lower || actual > upper) {
            return 0;
        }
        unsafe {
            vim_snprintf(
                expected.as_mut_ptr(),
                expected.len(),
                c"range %g - %g,".as_ptr(),
                lower,
                upper,
            )
        };
    } else {
        let mut error = false;
        let lower = unsafe { tv_get_number_chk(arg(argvars, 0), &raw mut error) };
        let upper = unsafe { tv_get_number_chk(arg(argvars, 1), &raw mut error) };
        let actual: varnumber_T = unsafe { tv_get_number_chk(arg(argvars, 2), &raw mut error) };
        if error || !(actual < lower || actual > upper) {
            return 0;
        }
        unsafe {
            vim_snprintf(
                expected.as_mut_ptr(),
                expected.len(),
                c"range %ld - %ld,".as_ptr(),
                lower,
                upper,
            )
        };
    }

    let mut ga = unsafe { prepare_assert_error() };
    unsafe {
        fill_assert_error(
            &raw mut ga,
            arg(argvars, 3),
            expected.as_ptr(),
            ptr::null_mut(),
            arg(argvars, 2),
            AssertType::Other,
        )
    };
    unsafe { report_assert_error(&raw mut ga) };
    1
}

// ---------------------------------------------------------------------------
// The builtins
// ---------------------------------------------------------------------------

/// `assert_beeps(cmd)`.
pub(crate) unsafe fn f_assert_beeps(
    argvars: *mut typval_T,
    rettv: *mut typval_T,
    _fptr: EvalFuncData,
) {
    // SAFETY: the evaluator's argument vector and return slot.
    unsafe { (*rettv).vval.v_number = assert_beeps(argvars, false) as varnumber_T };
}

/// `assert_nobeep(cmd)`.
pub(crate) unsafe fn f_assert_nobeep(
    argvars: *mut typval_T,
    rettv: *mut typval_T,
    _fptr: EvalFuncData,
) {
    // SAFETY: the evaluator's argument vector and return slot.
    unsafe { (*rettv).vval.v_number = assert_beeps(argvars, true) as varnumber_T };
}

/// `assert_equal(expected, actual[, msg])`.
pub(crate) unsafe fn f_assert_equal(
    argvars: *mut typval_T,
    rettv: *mut typval_T,
    _fptr: EvalFuncData,
) {
    // SAFETY: the evaluator's argument vector and return slot.
    unsafe {
        (*rettv).vval.v_number = assert_equal_common(argvars, AssertType::Equal) as varnumber_T
    };
}

/// `assert_notequal(expected, actual[, msg])`.
pub(crate) unsafe fn f_assert_notequal(
    argvars: *mut typval_T,
    rettv: *mut typval_T,
    _fptr: EvalFuncData,
) {
    // SAFETY: the evaluator's argument vector and return slot.
    unsafe {
        (*rettv).vval.v_number = assert_equal_common(argvars, AssertType::NotEqual) as varnumber_T
    };
}

/// `assert_equalfile(fname-one, fname-two[, msg])`.
pub(crate) unsafe fn f_assert_equalfile(
    argvars: *mut typval_T,
    rettv: *mut typval_T,
    _fptr: EvalFuncData,
) {
    // SAFETY: the evaluator's argument vector and return slot.
    unsafe { (*rettv).vval.v_number = assert_equalfile(argvars) as varnumber_T };
}

/// `assert_exception(string[, msg])`.
pub(crate) unsafe fn f_assert_exception(
    argvars: *mut typval_T,
    rettv: *mut typval_T,
    _fptr: EvalFuncData,
) {
    let mut numbuf = NumBuf::new();
    // SAFETY: the evaluator's argument vector and return slot.
    let error = unsafe { numbuf.string_chk(arg(argvars, 0)) };
    if unsafe { *get_vim_var_str(Vv::Exception) } == 0 {
        let mut ga = unsafe { prepare_assert_error() };
        unsafe { ga_concat_lit(&raw mut ga, c"v:exception is not set") };
        unsafe { report_assert_error(&raw mut ga) };
        unsafe { (*rettv).vval.v_number = 1 };
    } else if !error.is_null() && unsafe { strstr(get_vim_var_str(Vv::Exception), error) }.is_null()
    {
        let mut ga = unsafe { prepare_assert_error() };
        unsafe {
            fill_assert_error(
                &raw mut ga,
                arg(argvars, 1),
                ptr::null(),
                arg(argvars, 0),
                get_vim_var_tv(Vv::Exception),
                AssertType::Other,
            )
        };
        unsafe { report_assert_error(&raw mut ga) };
        unsafe { (*rettv).vval.v_number = 1 };
    }
}

/// `assert_false(actual[, msg])`.
pub(crate) unsafe fn f_assert_false(
    argvars: *mut typval_T,
    rettv: *mut typval_T,
    _fptr: EvalFuncData,
) {
    // SAFETY: the evaluator's argument vector and return slot.
    unsafe { (*rettv).vval.v_number = assert_bool(argvars, false) as varnumber_T };
}

/// `assert_true(actual[, msg])`.
pub(crate) unsafe fn f_assert_true(
    argvars: *mut typval_T,
    rettv: *mut typval_T,
    _fptr: EvalFuncData,
) {
    // SAFETY: the evaluator's argument vector and return slot.
    unsafe { (*rettv).vval.v_number = assert_bool(argvars, true) as varnumber_T };
}

/// `assert_inrange(lower, upper, actual[, msg])`.
pub(crate) unsafe fn f_assert_inrange(
    argvars: *mut typval_T,
    rettv: *mut typval_T,
    _fptr: EvalFuncData,
) {
    // SAFETY: the evaluator's argument vector and return slot.
    if unsafe { tv_check_for_float_or_nr_arg(argvars, 0) }.is_err()
        || unsafe { tv_check_for_float_or_nr_arg(argvars, 1) }.is_err()
        || unsafe { tv_check_for_float_or_nr_arg(argvars, 2) }.is_err()
        || unsafe { tv_check_for_opt_string_arg(argvars, 3) }.is_err()
    {
        return;
    }
    unsafe { (*rettv).vval.v_number = assert_inrange(argvars) as varnumber_T };
}

/// `assert_match(pattern, actual[, msg])`.
pub(crate) unsafe fn f_assert_match(
    argvars: *mut typval_T,
    rettv: *mut typval_T,
    _fptr: EvalFuncData,
) {
    // SAFETY: the evaluator's argument vector and return slot.
    unsafe {
        (*rettv).vval.v_number = assert_match_common(argvars, AssertType::Match) as varnumber_T
    };
}

/// `assert_notmatch(pattern, actual[, msg])`.
pub(crate) unsafe fn f_assert_notmatch(
    argvars: *mut typval_T,
    rettv: *mut typval_T,
    _fptr: EvalFuncData,
) {
    // SAFETY: the evaluator's argument vector and return slot.
    unsafe {
        (*rettv).vval.v_number = assert_match_common(argvars, AssertType::NotMatch) as varnumber_T
    };
}

/// `assert_report(msg)`: an unconditional failure.
pub(crate) unsafe fn f_assert_report(
    argvars: *mut typval_T,
    rettv: *mut typval_T,
    _fptr: EvalFuncData,
) {
    let mut numbuf = NumBuf::new();
    // SAFETY: the evaluator's argument vector and return slot.
    let mut ga = unsafe { prepare_assert_error() };
    unsafe { ga_concat(&raw mut ga, numbuf.string(arg(argvars, 0))) };
    unsafe { report_assert_error(&raw mut ga) };
    unsafe { (*rettv).vval.v_number = 1 };
}

/// `test_garbagecollect_now()`: collect immediately rather than at the next
/// safe point.
///
/// This is dangerous — any list or dict held only by internal C state is freed
/// while still in use — so it is refused unless `v:testing` says the caller
/// meant it.
pub(crate) unsafe fn f_test_garbagecollect_now(
    _argvars: *mut typval_T,
    _rettv: *mut typval_T,
    _fptr: EvalFuncData,
) {
    // SAFETY: called from the evaluator on the main thread.
    if unsafe { get_vim_var_nr(Vv::Testing) } == 0 {
        emsg(gettext(E_TEST_GARBAGECOLLECT_NOW));
    } else {
        unsafe { garbage_collect(true) };
    }
}

/// `test_write_list_log(fname)`: a no-op.
///
/// Upstream keeps the builtin so scripts that call it still parse, but the
/// list-allocation log it wrote is only compiled in under a debug define that
/// no shipped build sets. The argument is still read, so a bad one is still
/// reported.
pub(crate) unsafe fn f_test_write_list_log(
    argvars: *mut typval_T,
    _rettv: *mut typval_T,
    _fptr: EvalFuncData,
) {
    let mut numbuf = NumBuf::new();
    // SAFETY: the evaluator's argument vector.
    unsafe { numbuf.string_chk(arg(argvars, 0)) };
}
