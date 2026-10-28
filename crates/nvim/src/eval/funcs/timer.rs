//! Time: the `timer_*()` family, `wait()` and the `reltime()` clock.
#![forbid(unsafe_code)]

use super::wrappers::arg_number;
use crate::eval::typval::TV_INITIAL_VALUE;
use crate::eval::typval::{
    dict_find, list_find_nr, list_len, tv_check_for_nonnull_dict_arg, tv_check_for_number_arg,
    tv_check_for_opt_number_arg, tv_get_number, tv_get_number_chk, tv_list_alloc,
    tv_list_alloc_ret,
};
use crate::eval::{
    add_timer_info, add_timer_info_all, callback_from_typval, eval_expr_typval, timer_pause,
    timer_start, timer_stop, timer_stop_all,
};
use crate::event::r#loop::{Drain, wait_on_main_loop};
use crate::event::time::ScopedTimer;
use crate::ex_cmds::check_secure;
use crate::getchar::state::got_int;
use crate::getchar::vgetc;
use crate::memory::ThinCString;
use crate::message::state::called_emsg;
use crate::os::time::os_time;
use crate::profile::{profile_end, profile_msg_str, profile_signed, profile_start, profile_sub};
use crate::semsg;
use crate::types::{
    Callback, EvalFuncData, Float, ProfTime, TypVal, VAR_LIST, VAR_NUMBER, VarNumber, int32_t,
    kListLenUnknown,
};
use crate::ui::ui_flush;
use core::ffi::c_int;

/// A cleared typval, the shape the evaluator's out-parameters start in.
const EMPTY_TV: TypVal = TV_INITIAL_VALUE;

/// `wait({timeout}, {condition} [, {interval}])` — pump the event loop until
/// `condition` evaluates true. 0 when it did, -1 on timeout, -2 on CTRL-C,
/// -3 when evaluating `condition` failed.
pub fn f_wait(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    result.write_number(-1);
    if args[0].v_type() != VAR_NUMBER {
        semsg!("E475: Invalid value for argument 1");
        return;
    }
    // The interval must be absent or a positive Number. The C spells
    // this as one `A && B || C && D`; it is the same test.
    let interval = match args.get(2) {
        None => 200,
        Some(tv) if tv.v_type() == VAR_NUMBER && tv.number_or_zero() > 0 => {
            tv.number_or_zero() as c_int
        }
        Some(_) => {
            semsg!("E475: Invalid value for argument 3");
            return;
        }
    };
    let timeout = args[0].number_or_zero() as c_int;
    let expr = &args[1];

    // Keeps the loop turning, so that `condition` is evaluated at least
    // every `interval` even when nothing else happens.
    let waker = ScopedTimer::wake_every(interval as u64);

    let mut exprval = EMPTY_TV;
    let mut error = false;
    let called_emsg_before = called_emsg.get();
    ui_flush();
    let done = || {
        let got = eval_expr_typval(expr, false, &[], &mut exprval);
        got.is_err()
            || tv_get_number_chk(&exprval).map_or_else(
                |_| {
                    error = true;
                    false
                },
                |n| n != 0,
            )
            || called_emsg.get() > called_emsg_before
            || error
            || got_int.get()
    };
    wait_on_main_loop(Drain::Events, timeout as i64, done);
    drop(waker);
    if called_emsg.get() > called_emsg_before || error {
        result.write_number(-3);
    } else if got_int.get() {
        got_int.set(false);
        vgetc();
        result.write_number(-2);
    } else if tv_get_number_chk(&exprval).is_ok_and(|n| n != 0) {
        result.write_number(0);
    }
}

/// `localtime()` — seconds since the epoch.
pub fn f_localtime(_args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    result.write_number(os_time() as VarNumber);
}

/// A `ProfTime` split into the pair of 32-bit halves `reltime()` reports.
///
/// The C reads the profile time through a union of the timestamp with a
/// `struct { int32_t low, high; }`, so the halves are the timestamp's own
/// bytes in memory order. `to_ne_bytes` reproduces exactly that, on any
/// endianness, without the transmute.
fn proftime_halves(tm: ProfTime) -> (int32_t, int32_t) {
    let bytes = tm.to_ne_bytes();
    let low = int32_t::from_ne_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);
    let high = int32_t::from_ne_bytes([bytes[4], bytes[5], bytes[6], bytes[7]]);
    (high, low)
}

/// The inverse of [`proftime_halves`].
fn proftime_from_halves(high: int32_t, low: int32_t) -> ProfTime {
    let (lo, hi) = (low.to_ne_bytes(), high.to_ne_bytes());
    ProfTime::from_ne_bytes([lo[0], lo[1], lo[2], lo[3], hi[0], hi[1], hi[2], hi[3]])
}

/// Read a `[high, low]` List back into a profile timestamp. `None` when the
/// argument is not a two-element List of Numbers.
fn list2proftime(arg: &TypVal) -> Option<ProfTime> {
    if arg.v_type() != VAR_LIST || list_len(arg.list_ref()) != 2 {
        return None;
    }
    let mut error = false;
    let n1 = list_find_nr(arg.list_ref(), 0, Some(&mut error));
    let n2 = list_find_nr(arg.list_ref(), 1, Some(&mut error));
    if error {
        return None;
    }
    Some(proftime_from_halves(n1 as int32_t, n2 as int32_t))
}

/// `reltime([{start} [, {end}]])` — a timestamp, an elapsed time, or the
/// difference between two timestamps, as a `[high, low]` List.
pub fn f_reltime(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    let res = if args.is_empty() {
        profile_start()
    } else if args.len() <= 1 {
        let Some(start) = list2proftime(&args[0]) else {
            return;
        };
        profile_end(start)
    } else {
        // Short-circuit as the C `||` does: a bad first argument means
        // the second is never read, so its own coercion errors do not
        // fire.
        let Some(start) = list2proftime(&args[0]) else {
            return;
        };
        let Some(end) = list2proftime(&args[1]) else {
            return;
        };
        profile_sub(end, start)
    };
    let (high, low) = proftime_halves(res);
    let list = tv_list_alloc_ret(result, 2);
    list.push_number(VarNumber::from(high));
    list.push_number(VarNumber::from(low));
}

/// `reltimestr({time})` — the elapsed time as seconds with six decimals.
pub fn f_reltimestr(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    result.write_string(None);
    if let Some(tm) = list2proftime(&args[0]) {
        // Cut to the 49 bytes `profile_msg`'s buffer holds.
        let text = profile_msg_str(tm);
        let text = &text.as_bytes()[..text.len().min(49)];
        result.write_string(Some(ThinCString::from_bytes(text)));
    }
}

/// `reltimefloat({time})` — the elapsed time in seconds.
pub fn f_reltimefloat(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    result.write_float(0.0);
    if let Some(tm) = list2proftime(&args[0]) {
        result.write_float((profile_signed(tm) as f64 / 1_000_000_000.0) as Float);
    }
}

/// `timer_info([{id}])` — one timer's state, or every live timer's.
pub fn f_timer_info(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    let list = tv_list_alloc(kListLenUnknown as isize);
    result.write_list(Some(list.clone()));
    if tv_check_for_opt_number_arg(args, 0).is_err() {
        return;
    }
    if args.is_empty() {
        add_timer_info_all(&list);
    } else {
        add_timer_info(&list, arg_number(&args[0]));
    }
}

/// `timer_pause({id}, {pause})` — stop or restart a timer's clock without
/// forgetting it.
pub fn f_timer_pause(args: &[TypVal], _unused: &mut TypVal, _fptr: EvalFuncData) {
    if args[0].v_type() != VAR_NUMBER {
        semsg!("E39: Number expected");
        return;
    }
    // Read before the timer is looked up, as the C does: the coercion
    // of the second argument can report its own error.
    let paused = arg_number(&args[1]) != 0;
    timer_pause(arg_number(&args[0]), paused);
}

/// `timer_start({time}, {callback} [, {options}])` — the new timer's id, or
/// -1 when it could not be started.
pub fn f_timer_start(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    result.write_number(-1);
    if check_secure() {
        return;
    }
    let mut repeat: c_int = 1;
    if args.len() > 2 {
        if tv_check_for_nonnull_dict_arg(args, 2).is_err() {
            return;
        }
        if let Some(di) = dict_find(args[2].dict_ref(), b"repeat") {
            repeat = tv_get_number(&di.di_tv) as c_int;
            // A repeat of 0 means "once", the same as the default.
            if repeat == 0 {
                repeat = 1;
            }
        }
    }
    let mut callback = Callback::None;
    if !callback_from_typval(&mut callback, &args[1]) {
        return;
    }
    result.write_number(timer_start(arg_number(&args[0]), repeat, callback) as VarNumber);
}

/// `timer_stop({id})`.
pub fn f_timer_stop(args: &[TypVal], _result: &mut TypVal, _fptr: EvalFuncData) {
    if tv_check_for_number_arg(args, 0).is_err() {
        return;
    }
    timer_stop(arg_number(&args[0]));
}

/// `timer_stopall()`.
pub fn f_timer_stopall(_args: &[TypVal], _unused: &mut TypVal, _fptr: EvalFuncData) {
    timer_stop_all()
}
