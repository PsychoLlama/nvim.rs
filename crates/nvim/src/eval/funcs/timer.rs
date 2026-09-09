//! Time: the `timer_*()` family, `wait()` and the `reltime()` clock.
#![deny(unsafe_op_in_unsafe_fn)]
#![allow(unsafe_code)]

use super::wrappers::{arg_number, list_alloc_ret};
use crate::eval::typval::TV_INITIAL_VALUE;
use crate::eval::typval::{
    tv_check_for_nonnull_dict_arg, tv_check_for_number_arg, tv_check_for_opt_number_arg,
    tv_dict_find, tv_get_number, tv_get_number_chk, tv_list_append_number, tv_list_find_nr,
    tv_list_len,
};
use crate::eval::{
    add_timer_info, add_timer_info_all, callback_from_typval, eval_expr_typval, find_timer_by_nr,
    timer_due_cb, timer_start, timer_stop, timer_stop_all,
};
use crate::event::r#loop::process_events_until;
use crate::event::time::{
    time_watcher_close, time_watcher_init, time_watcher_start, time_watcher_stop,
};
use crate::ex_cmds::check_secure;
use crate::getchar::state::got_int;
use crate::getchar::vgetc;
use crate::memory::{xfree, xmalloc, xstrdup};
use crate::message::state::called_emsg;
use crate::profile::{profile_end, profile_msg, profile_signed, profile_start, profile_sub};
use crate::semsg;
use crate::startup::main_loop;
use crate::types::{
    Callback, EvalFuncData, Float, MultiQueue, ProfTime, TimeWatcher, TypVal, VAR_LIST, VAR_NUMBER,
    VarNumber, int32_t, kListLenUnknown, time_t,
};
use crate::ui::ui_flush;
use ::libc::time;
use core::ffi::{c_int, c_void};
use core::ptr;

/// A cleared typval, the shape the evaluator's out-parameters start in.
const EMPTY_TV: TypVal = TV_INITIAL_VALUE;

/// `wait()`'s idle timer keeps the event loop turning; it is only closed
/// here when the loop itself is shutting down, since `f_wait` cannot run to
/// its own cleanup in that case.
///
/// # Safety
/// A libuv callback: `tw` is the watcher this module allocated.
unsafe fn dummy_timer_due_cb(tw: *mut TimeWatcher, _data: *mut c_void) {
    // SAFETY: the caller's obligation; `main_loop` is live for the process.
    if unsafe { (*main_loop.ptr()).closing } {
        unsafe { time_watcher_stop(tw) };
        unsafe { time_watcher_close(tw, Some(dummy_timer_close_cb)) };
    }
}

/// Free the watcher `f_wait` allocated, once libuv is done with it.
///
/// # Safety
/// A libuv callback: `tw` is the watcher this module allocated.
unsafe fn dummy_timer_close_cb(tw: *mut TimeWatcher, _data: *mut c_void) {
    // SAFETY: the caller's obligation; nothing else holds the watcher by
    // the time libuv reports it closed.
    unsafe { xfree(tw as *mut c_void) }
}

/// `wait({timeout}, {condition} [, {interval}])` — pump the event loop until
/// `condition` evaluates true. 0 when it did, -1 on timeout, -2 on CTRL-C,
/// -3 when evaluating `condition` failed.
pub fn f_wait(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    result.write_number(-1);
    // SAFETY throughout: the watcher is owned here and handed to libuv's close
    // callback; every typval below is either from the frame or a local.
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

    let tw = unsafe { xmalloc(size_of::<TimeWatcher>()) } as *mut TimeWatcher;
    unsafe { time_watcher_init(main_loop.ptr(), tw, ptr::null_mut()) };
    unsafe { (*tw).events = ptr::null_mut::<MultiQueue>() };
    let every = interval as u64;
    let due = dummy_timer_due_cb;
    unsafe { time_watcher_start(tw, Some(due), every, every) };

    let mut argv = EMPTY_TV;
    let mut exprval = EMPTY_TV;
    let mut error = false;
    let called_emsg_before = called_emsg.get();
    unsafe { ui_flush() };
    let loop_ = main_loop.ptr();
    let events = unsafe { (*loop_).events };
    // SAFETY throughout: `expr`, `argv` and `exprval` are this frame's locals, which
    // outlive the wait, and the main loop is running.
    let done = || {
        let out = &raw mut exprval;
        let got = unsafe { eval_expr_typval(expr, false, &raw mut argv, 0, out) };
        got.is_err()
            || unsafe { tv_get_number_chk(out, &raw mut error) } != 0
            || called_emsg.get() > called_emsg_before
            || error
            || got_int.get()
    };
    unsafe { process_events_until(loop_, events, timeout as i64, done) };
    if called_emsg.get() > called_emsg_before || error {
        result.write_number(-3);
    } else if got_int.get() {
        got_int.set(false);
        vgetc();
        result.write_number(-2);
    } else if unsafe { tv_get_number_chk(&raw mut exprval, &raw mut error) } != 0 {
        result.write_number(0);
    }
    unsafe { time_watcher_stop(tw) };
    unsafe { time_watcher_close(tw, Some(dummy_timer_close_cb)) };
}

/// `localtime()` — seconds since the epoch.
pub fn f_localtime(_args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    // SAFETY: `time(NULL)` writes nothing.
    result.write_number(unsafe { time(ptr::null_mut::<time_t>()) } as VarNumber);
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
///
/// # Safety
/// `arg` is a live typval from the call frame.
unsafe fn list2proftime(arg: *const TypVal) -> Option<ProfTime> {
    // SAFETY: the caller's obligation; the list is only read.
    let arg = unsafe { &*arg };
    if arg.v_type() != VAR_LIST || unsafe { tv_list_len(arg.list_or_null()) } != 2 {
        return None;
    }
    let mut error = false;
    let n1 = unsafe { tv_list_find_nr(arg.list_or_null(), 0, &raw mut error) };
    let n2 = unsafe { tv_list_find_nr(arg.list_or_null(), 1, &raw mut error) };
    if error {
        return None;
    }
    Some(proftime_from_halves(n1 as int32_t, n2 as int32_t))
}

/// `reltime([{start} [, {end}]])` — a timestamp, an elapsed time, or the
/// difference between two timestamps, as a `[high, low]` List.
pub fn f_reltime(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    // SAFETY throughout: the list entry points take the frame's return value, which is
    // cleared and owned by the caller.
    let res = if args.is_empty() {
        profile_start()
    } else if args.len() <= 1 {
        let Some(start) = (unsafe { list2proftime(&args[0]) }) else {
            return;
        };
        profile_end(start)
    } else {
        // Short-circuit as the C `||` does: a bad first argument means
        // the second is never read, so its own coercion errors do not
        // fire.
        let Some(start) = (unsafe { list2proftime(&args[0]) }) else {
            return;
        };
        let Some(end) = (unsafe { list2proftime(&args[1]) }) else {
            return;
        };
        profile_sub(end, start)
    };
    let (high, low) = proftime_halves(res);
    list_alloc_ret(result, 2);
    unsafe { tv_list_append_number(result.list_or_null(), high as VarNumber) };
    unsafe { tv_list_append_number(result.list_or_null(), low as VarNumber) };
}

/// `reltimestr({time})` — the elapsed time as seconds with six decimals.
pub fn f_reltimestr(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    result.write_string(ptr::null_mut());
    // SAFETY: `profile_msg` returns a pointer to its own static buffer,
    // which `xstrdup` copies before anything else can reuse it.
    if let Some(tm) = unsafe { list2proftime(&args[0]) } {
        result.write_string(unsafe { xstrdup(profile_msg(tm).as_ptr()) });
    }
}

/// `reltimefloat({time})` — the elapsed time in seconds.
pub fn f_reltimefloat(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    result.write_float(0.0);
    // SAFETY: reads the argument through the frame.
    if let Some(tm) = unsafe { list2proftime(&args[0]) } {
        result.write_float((profile_signed(tm) as f64 / 1_000_000_000.0) as Float);
    }
}

/// `timer_info([{id}])` — one timer's state, or every live timer's.
pub fn f_timer_info(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    // SAFETY throughout: the timer list is main-thread state; the return value is the
    // caller's cleared typval.
    list_alloc_ret(result, kListLenUnknown as c_int as isize);
    if tv_check_for_opt_number_arg(args, 0).is_err() {
        return;
    }
    if args.is_empty() {
        unsafe { add_timer_info_all(result) };
        return;
    }
    let timer = find_timer_by_nr(arg_number(&args[0]));
    // A stopped timer is still reported while a callback holds a
    // reference to it.
    if !timer.is_null()
        && (!unsafe { (*timer).stopped } || unsafe { (*timer).refcount }.is_shared())
    {
        unsafe { add_timer_info(result, timer) };
    }
}

/// `timer_pause({id}, {pause})` — stop or restart a timer's clock without
/// forgetting it.
pub fn f_timer_pause(args: &[TypVal], _unused: &mut TypVal, _fptr: EvalFuncData) {
    let _rettv = _unused;
    // SAFETY throughout: the timer comes from the main-thread timer table and its
    // watcher is embedded in it.
    if args[0].v_type() != VAR_NUMBER {
        semsg!("E39: Number expected");
        return;
    }
    // Read before the timer is looked up, as the C does: the coercion
    // of the second argument can report its own error.
    let paused = arg_number(&args[1]) != 0;
    let timer = find_timer_by_nr(arg_number(&args[0]));
    if timer.is_null() {
        return;
    }
    if !unsafe { (*timer).paused } && paused {
        unsafe { time_watcher_stop(&raw mut (*timer).tw) };
    } else if unsafe { (*timer).paused } && !paused {
        let tw = unsafe { &raw mut (*timer).tw };
        let every = unsafe { (*timer).timeout } as u64;
        unsafe { time_watcher_start(tw, Some(timer_due_cb), every, every) };
    }
    unsafe { (*timer).paused = paused };
}

/// `timer_start({time}, {callback} [, {options}])` — the new timer's id, or
/// -1 when it could not be started.
pub fn f_timer_start(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    result.write_number(-1);
    // SAFETY throughout: the options dict and the callback typval are the frame's;
    // `timer_start` takes the callback over.
    if check_secure() {
        return;
    }
    let mut repeat: c_int = 1;
    if args.len() > 2 {
        if tv_check_for_nonnull_dict_arg(args, 2).is_err() {
            return;
        }
        let di = unsafe { tv_dict_find(args[2].dict_or_null(), c"repeat".as_ptr(), 6) };
        if !di.is_null() {
            repeat = unsafe { tv_get_number(&raw mut (*di).di_tv) } as c_int;
            // A repeat of 0 means "once", the same as the default.
            if repeat == 0 {
                repeat = 1;
            }
        }
    }
    let mut callback = Callback::None;
    if !unsafe { callback_from_typval(&raw mut callback, &args[1]) } {
        return;
    }
    result.write_number(
        unsafe { timer_start(arg_number(&args[0]), repeat, &raw mut callback) } as VarNumber,
    );
}

/// `timer_stop({id})`.
pub fn f_timer_stop(args: &[TypVal], _result: &mut TypVal, _fptr: EvalFuncData) {
    let _rettv = _result;
    // SAFETY throughout: the timer comes from the main-thread timer table.
    if tv_check_for_number_arg(args, 0).is_err() {
        return;
    }
    let timer = find_timer_by_nr(arg_number(&args[0]));
    if !timer.is_null() {
        unsafe { timer_stop(timer) };
    }
}

/// `timer_stopall()`.
pub fn f_timer_stopall(_args: &[TypVal], _unused: &mut TypVal, _fptr: EvalFuncData) {
    // SAFETY: walks the main-thread timer table.
    unsafe { timer_stop_all() }
}
