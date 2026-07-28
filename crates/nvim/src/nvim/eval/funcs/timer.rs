//! Time: the `timer_*()` family, `wait()` and the `reltime()` clock.
#![deny(unsafe_op_in_unsafe_fn)]

use super::args::frame;
use super::{
    FAIL, VAR_FLOAT, VAR_LIST, VAR_NUMBER, VAR_STRING, VAR_UNKNOWN, VAR_UNLOCKED, kCallbackNone,
    kListLenUnknown,
};
use crate::semsg;
use crate::src::nvim::eval::typval::{
    tv_check_for_nonnull_dict_arg, tv_check_for_number_arg, tv_check_for_opt_number_arg,
};
use crate::src::nvim::eval::typval::{
    tv_dict_find, tv_get_number, tv_get_number_chk, tv_list_alloc_ret, tv_list_append_number,
    tv_list_find_nr, tv_list_len,
};
use crate::src::nvim::eval_1::{
    add_timer_info, add_timer_info_all, callback_from_typval, eval_expr_typval, find_timer_by_nr,
    timer_due_cb, timer_start, timer_stop, timer_stop_all,
};
use crate::src::nvim::event::r#loop::process_events_until;
use crate::src::nvim::event::time::{
    time_watcher_close, time_watcher_init, time_watcher_start, time_watcher_stop,
};
use crate::src::nvim::ex_cmds::check_secure;
use crate::src::nvim::getchar::vgetc;
use crate::src::nvim::main::{called_emsg, got_int, main_loop};
use crate::src::nvim::memory::{xfree, xmalloc, xstrdup};
use crate::src::nvim::os::libc::time;
use crate::src::nvim::profile::{
    profile_end, profile_msg, profile_signed, profile_start, profile_sub,
};
use crate::src::nvim::types::{
    Callback, Callback_data, EvalFuncData, MultiQueue, TimeWatcher, float_T, int32_t, proftime_T,
    time_t, typval_T, typval_vval_union, varnumber_T,
};
use crate::src::nvim::ui::ui_flush;
use core::ffi::{c_char, c_int, c_void};
use core::ptr;

/// A cleared typval, the shape the evaluator's out-parameters start in.
const EMPTY_TV: typval_T = typval_T {
    v_type: VAR_UNKNOWN,
    v_lock: VAR_UNLOCKED,
    vval: typval_vval_union { v_number: 0 },
};

/// `wait()`'s idle timer keeps the event loop turning; it is only closed
/// here when the loop itself is shutting down, since `f_wait` cannot run to
/// its own cleanup in that case.
///
/// # Safety
/// A libuv callback: `tw` is the watcher this module allocated.
unsafe extern "C" fn dummy_timer_due_cb(tw: *mut TimeWatcher, _data: *mut c_void) {
    // SAFETY: the caller's obligation; `main_loop` is live for the process.
    unsafe {
        if (*main_loop.ptr()).closing {
            time_watcher_stop(tw);
            time_watcher_close(tw, Some(dummy_timer_close_cb));
        }
    }
}

/// Free the watcher `f_wait` allocated, once libuv is done with it.
///
/// # Safety
/// A libuv callback: `tw` is the watcher this module allocated.
unsafe extern "C" fn dummy_timer_close_cb(tw: *mut TimeWatcher, _data: *mut c_void) {
    // SAFETY: the caller's obligation; nothing else holds the watcher by
    // the time libuv reports it closed.
    unsafe { xfree(tw as *mut c_void) }
}

/// `wait({timeout}, {condition} [, {interval}])` — pump the event loop until
/// `condition` evaluates true. 0 when it did, -1 on timeout, -2 on CTRL-C,
/// -3 when evaluating `condition` failed.
pub unsafe extern "C" fn f_wait(argvars: *mut typval_T, rettv: *mut typval_T, _fptr: EvalFuncData) {
    let (args, rettv) = frame!(argvars, rettv);
    rettv.v_type = VAR_NUMBER;
    rettv.vval.v_number = -1;
    // SAFETY: the watcher is owned here and handed to libuv's close
    // callback; every typval below is either from the frame or a local.
    unsafe {
        if args.ty(0) != VAR_NUMBER {
            semsg!("E475: Invalid value for argument 1");
            return;
        }
        // The interval must be absent or a positive Number. The C spells
        // this as one `A && B || C && D`; it is the same test.
        let interval = match args.ty(2) {
            VAR_UNKNOWN => 200,
            VAR_NUMBER if args.get(2).vval.v_number > 0 => args.get(2).vval.v_number as c_int,
            _ => {
                semsg!("E475: Invalid value for argument 3");
                return;
            }
        };
        let timeout = args.get(0).vval.v_number as c_int;
        let expr = *args.get(1);

        let tw = xmalloc(core::mem::size_of::<TimeWatcher>()) as *mut TimeWatcher;
        time_watcher_init(main_loop.ptr(), tw, ptr::null_mut());
        (*tw).events = ptr::null_mut::<MultiQueue>();
        time_watcher_start(
            tw,
            Some(dummy_timer_due_cb),
            interval as u64,
            interval as u64,
        );

        let mut argv = EMPTY_TV;
        let mut exprval = EMPTY_TV;
        let mut error = false;
        let called_emsg_before = called_emsg.get();
        ui_flush();
        process_events_until(
            main_loop.ptr(),
            (*main_loop.ptr()).events,
            timeout as i64,
            || {
                eval_expr_typval(&raw const expr, false, &raw mut argv, 0, &raw mut exprval) != 1
                    || tv_get_number_chk(&raw mut exprval, &raw mut error) != 0
                    || called_emsg.get() > called_emsg_before
                    || error
                    || got_int.get()
            },
        );
        if called_emsg.get() > called_emsg_before || error {
            rettv.vval.v_number = -3;
        } else if got_int.get() {
            got_int.set(false);
            vgetc();
            rettv.vval.v_number = -2;
        } else if tv_get_number_chk(&raw mut exprval, &raw mut error) != 0 {
            rettv.vval.v_number = 0;
        }
        time_watcher_stop(tw);
        time_watcher_close(tw, Some(dummy_timer_close_cb));
    }
}

/// `localtime()` — seconds since the epoch.
pub unsafe extern "C" fn f_localtime(
    _argvars: *mut typval_T,
    rettv: *mut typval_T,
    _fptr: EvalFuncData,
) {
    let (_args, rettv) = frame!(_argvars, rettv);
    // SAFETY: `time(NULL)` writes nothing.
    rettv.vval.v_number = unsafe { time(ptr::null_mut::<time_t>()) } as varnumber_T;
}

/// A `proftime_T` split into the pair of 32-bit halves `reltime()` reports.
///
/// The C reads the profile time through a union of the timestamp with a
/// `struct { int32_t low, high; }`, so the halves are the timestamp's own
/// bytes in memory order. `to_ne_bytes` reproduces exactly that, on any
/// endianness, without the transmute.
fn proftime_halves(tm: proftime_T) -> (int32_t, int32_t) {
    let bytes = tm.to_ne_bytes();
    let low = int32_t::from_ne_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);
    let high = int32_t::from_ne_bytes([bytes[4], bytes[5], bytes[6], bytes[7]]);
    (high, low)
}

/// The inverse of [`proftime_halves`].
fn proftime_from_halves(high: int32_t, low: int32_t) -> proftime_T {
    let (lo, hi) = (low.to_ne_bytes(), high.to_ne_bytes());
    proftime_T::from_ne_bytes([lo[0], lo[1], lo[2], lo[3], hi[0], hi[1], hi[2], hi[3]])
}

/// Read a `[high, low]` List back into a profile timestamp. `None` when the
/// argument is not a two-element List of Numbers.
///
/// # Safety
/// `arg` is a live typval from the call frame.
unsafe fn list2proftime(arg: *const typval_T) -> Option<proftime_T> {
    // SAFETY: the caller's obligation; the list is only read.
    unsafe {
        let arg = &*arg;
        if arg.v_type != VAR_LIST || tv_list_len(arg.vval.v_list) != 2 {
            return None;
        }
        let mut error = false;
        let n1 = tv_list_find_nr(arg.vval.v_list, 0, &raw mut error);
        let n2 = tv_list_find_nr(arg.vval.v_list, 1, &raw mut error);
        if error {
            return None;
        }
        Some(proftime_from_halves(n1 as int32_t, n2 as int32_t))
    }
}

/// `reltime([{start} [, {end}]])` — a timestamp, an elapsed time, or the
/// difference between two timestamps, as a `[high, low]` List.
pub unsafe extern "C" fn f_reltime(
    argvars: *mut typval_T,
    rettv: *mut typval_T,
    _fptr: EvalFuncData,
) {
    let (args, rettv) = frame!(argvars, rettv);
    // SAFETY: the list entry points take the frame's return value, which is
    // cleared and owned by the caller.
    unsafe {
        let res = if !args.has(0) {
            profile_start()
        } else if !args.has(1) {
            let Some(start) = list2proftime(args.ptr(0)) else {
                return;
            };
            profile_end(start)
        } else {
            // Short-circuit as the C `||` does: a bad first argument means
            // the second is never read, so its own coercion errors do not
            // fire.
            let Some(start) = list2proftime(args.ptr(0)) else {
                return;
            };
            let Some(end) = list2proftime(args.ptr(1)) else {
                return;
            };
            profile_sub(end, start)
        };
        let (high, low) = proftime_halves(res);
        tv_list_alloc_ret(rettv, 2);
        tv_list_append_number(rettv.vval.v_list, high as varnumber_T);
        tv_list_append_number(rettv.vval.v_list, low as varnumber_T);
    }
}

/// `reltimestr({time})` — the elapsed time as seconds with six decimals.
pub unsafe extern "C" fn f_reltimestr(
    argvars: *mut typval_T,
    rettv: *mut typval_T,
    _fptr: EvalFuncData,
) {
    let (args, rettv) = frame!(argvars, rettv);
    rettv.v_type = VAR_STRING;
    rettv.vval.v_string = ptr::null_mut();
    // SAFETY: `profile_msg` returns a pointer to its own static buffer,
    // which `xstrdup` copies before anything else can reuse it.
    unsafe {
        if let Some(tm) = list2proftime(args.ptr(0)) {
            rettv.vval.v_string = xstrdup(profile_msg(tm));
        }
    }
}

/// `reltimefloat({time})` — the elapsed time in seconds.
pub unsafe extern "C" fn f_reltimefloat(
    argvars: *mut typval_T,
    rettv: *mut typval_T,
    _fptr: EvalFuncData,
) {
    let (args, rettv) = frame!(argvars, rettv);
    rettv.v_type = VAR_FLOAT;
    rettv.vval.v_float = 0.0;
    // SAFETY: reads the argument through the frame.
    if let Some(tm) = unsafe { list2proftime(args.ptr(0)) } {
        rettv.vval.v_float = (profile_signed(tm) as f64 / 1_000_000_000.0) as float_T;
    }
}

/// `timer_info([{id}])` — one timer's state, or every live timer's.
pub unsafe extern "C" fn f_timer_info(
    argvars: *mut typval_T,
    rettv: *mut typval_T,
    _fptr: EvalFuncData,
) {
    let (args, rettv) = frame!(argvars, rettv);
    // SAFETY: the timer list is main-thread state; the return value is the
    // caller's cleared typval.
    unsafe {
        tv_list_alloc_ret(rettv, kListLenUnknown as c_int as isize);
        if tv_check_for_opt_number_arg(args.ptr(0), 0) == FAIL {
            return;
        }
        if !args.has(0) {
            add_timer_info_all(rettv);
            return;
        }
        let timer = find_timer_by_nr(tv_get_number(args.ptr(0)));
        // A stopped timer is still reported while a callback holds a
        // reference to it.
        if !timer.is_null() && (!(*timer).stopped || (*timer).refcount > 1) {
            add_timer_info(rettv, timer);
        }
    }
}

/// `timer_pause({id}, {pause})` — stop or restart a timer's clock without
/// forgetting it.
pub unsafe extern "C" fn f_timer_pause(
    argvars: *mut typval_T,
    _unused: *mut typval_T,
    _fptr: EvalFuncData,
) {
    let (args, _rettv) = frame!(argvars, _unused);
    // SAFETY: the timer comes from the main-thread timer table and its
    // watcher is embedded in it.
    unsafe {
        if args.ty(0) != VAR_NUMBER {
            semsg!("E39: Number expected");
            return;
        }
        // Read before the timer is looked up, as the C does: the coercion
        // of the second argument can report its own error.
        let paused = tv_get_number(args.ptr(1)) != 0;
        let timer = find_timer_by_nr(tv_get_number(args.ptr(0)));
        if timer.is_null() {
            return;
        }
        if !(*timer).paused && paused {
            time_watcher_stop(&raw mut (*timer).tw);
        } else if (*timer).paused && !paused {
            time_watcher_start(
                &raw mut (*timer).tw,
                Some(timer_due_cb),
                (*timer).timeout as u64,
                (*timer).timeout as u64,
            );
        }
        (*timer).paused = paused;
    }
}

/// `timer_start({time}, {callback} [, {options}])` — the new timer's id, or
/// -1 when it could not be started.
pub unsafe extern "C" fn f_timer_start(
    argvars: *mut typval_T,
    rettv: *mut typval_T,
    _fptr: EvalFuncData,
) {
    let (args, rettv) = frame!(argvars, rettv);
    rettv.vval.v_number = -1;
    // SAFETY: the options dict and the callback typval are the frame's;
    // `timer_start` takes the callback over.
    unsafe {
        if check_secure() {
            return;
        }
        let mut repeat: c_int = 1;
        if args.has(2) {
            if tv_check_for_nonnull_dict_arg(args.ptr(0), 2) == FAIL {
                return;
            }
            let di = tv_dict_find(args.get(2).vval.v_dict, c"repeat".as_ptr(), 6);
            if !di.is_null() {
                repeat = tv_get_number(&raw mut (*di).di_tv) as c_int;
                // A repeat of 0 means "once", the same as the default.
                if repeat == 0 {
                    repeat = 1;
                }
            }
        }
        let mut callback = Callback {
            data: Callback_data {
                funcref: ptr::null_mut::<c_char>(),
            },
            type_0: kCallbackNone,
        };
        if !callback_from_typval(&raw mut callback, args.ptr(1)) {
            return;
        }
        rettv.vval.v_number =
            timer_start(tv_get_number(args.ptr(0)), repeat, &raw mut callback) as varnumber_T;
    }
}

/// `timer_stop({id})`.
pub unsafe extern "C" fn f_timer_stop(
    argvars: *mut typval_T,
    _rettv: *mut typval_T,
    _fptr: EvalFuncData,
) {
    let (args, _rettv) = frame!(argvars, _rettv);
    // SAFETY: the timer comes from the main-thread timer table.
    unsafe {
        if tv_check_for_number_arg(args.ptr(0), 0) == FAIL {
            return;
        }
        let timer = find_timer_by_nr(tv_get_number(args.ptr(0)));
        if !timer.is_null() {
            timer_stop(timer);
        }
    }
}

/// `timer_stopall()`.
pub unsafe extern "C" fn f_timer_stopall(
    _argvars: *mut typval_T,
    _unused: *mut typval_T,
    _fptr: EvalFuncData,
) {
    // SAFETY: walks the main-thread timer table.
    unsafe { timer_stop_all() }
}
