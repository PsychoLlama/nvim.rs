//! `timer_start()` and what fires when one is due.
//!
//! The timer table owns every `Timer` by its number. A timer's libuv watcher
//! is told that number, never the timer's address, so each callback finds
//! its timer again by id and stops cleanly if it has gone: a timer callback
//! runs Vimscript, which can stop the timer -- itself included -- and the
//! close that follows removes it from the table.
//!
//! Every walk below goes over a snapshot of the ids for the same reason:
//! nothing holds a borrow of the table across a callback.

#![forbid(unsafe_code)]

use core::ffi::c_int;

use crate::eval::typval::TV_INITIAL_VALUE;
use crate::eval::typval::{ListRef, callback_copy, callback_free, callback_put, tv_dict_alloc};
use crate::eval::vars::clear_local;
use crate::eval::{callback_call, last_timer_id, timers};
use crate::event::time::{OwnedTimer, TimerQueue};
use crate::ex_docmd::{get_pressedreturn, set_pressedreturn};
use crate::ex_eval::discard_current_exception;
use crate::ex_eval::state::did_throw;
use crate::message::state::{called_emsg, did_emsg};
use crate::registry::SlotTable;
use crate::types::{Callback, DictItem, Timer, TypVal, VarNumber, int64_t, uint64_t};

/// How many consecutive errors a timer's callback may raise before the
/// timer is stopped for good.
const MAX_ERRORS: c_int = 3;

/// A freshly declared typval.
const UNSET_TV: TypVal = TV_INITIAL_VALUE;

/// The table's key for a timer number. Timer numbers are handed to
/// Vimscript as Numbers; a negative one names no timer.
fn timer_key(id: VarNumber) -> uint64_t {
    id as uint64_t
}

/// Run `f` on the timer numbered `id`, if it is registered. The borrow ends
/// with `f`, which must not run user code.
pub(crate) fn with_timer<R>(id: VarNumber, f: impl FnOnce(&mut Timer) -> R) -> Option<R> {
    timers.with_mut(|map| map.get_mut(&timer_key(id)).map(f))
}

/// Whether `timer_info()` lists `timer`: a stopped timer still is while one
/// of its own callbacks is running.
fn listed(timer: &Timer) -> bool {
    !timer.stopped || timer.busy > 0
}

/// Append the timer numbered `id`'s description to `list`, if it is listed.
pub fn add_timer_info(list: &ListRef, id: VarNumber) {
    let Some((fields, callback)) = with_timer(id, |timer| {
        if !listed(timer) {
            return None;
        }
        let fields = [
            (b"id".as_slice(), VarNumber::from(timer.timer_id)),
            (b"time", timer.timeout),
            (b"paused", VarNumber::from(timer.paused)),
            // A negative repeat count means "for ever", reported as -1.
            (b"repeat", VarNumber::from(timer.repeat_count.max(-1))),
        ];
        let mut item = DictItem::boxed(b"callback");
        callback_put(&timer.callback, &mut item.di_tv);
        Some((fields, item))
    })
    .flatten() else {
        return;
    };

    let dict = tv_dict_alloc();
    for (key, value) in fields {
        let _ = dict.edit().add_number(key, value);
    }
    // A refused item -- which a fresh dictionary never refuses -- goes with
    // its value.
    let _ = dict.edit().add_item(callback);
    list.edit().push_dict(Some(dict));
}

/// Append a description of every listed timer to `list`.
pub fn add_timer_info_all(list: &ListRef) {
    for id in timers.with(SlotTable::snapshot_keys) {
        add_timer_info(list, id as VarNumber);
    }
}

/// The timer numbered `id` is due: run its callback.
fn timer_due_cb(id: uint64_t) {
    let number = id as VarNumber;
    let save_did_emsg = did_emsg.get();
    let called_emsg_before = called_emsg.get();
    let save_ex_pressedreturn = get_pressedreturn();

    // The callback is copied out: the call below runs Vimscript, which may
    // stop this timer and close it, and the copy is what keeps the function
    // alive until the call returns.
    let Some((callback, last)) = with_timer(number, |timer| {
        if timer.stopped || timer.paused {
            return None;
        }
        timer.busy += 1;
        let last = timer.repeat_count >= 0 && {
            timer.repeat_count -= 1;
            timer.repeat_count == 0
        };
        let mut callback = Callback::None;
        callback_copy(&mut callback, &timer.callback);
        Some((callback, last))
    })
    .flatten() else {
        return;
    };
    if last {
        timer_stop(number);
    }

    let argv = [TypVal::Number(number)];
    let mut rettv = UNSET_TV;
    callback_call(&callback, &argv, &mut rettv);
    let mut callback = callback;
    callback_free(&mut callback);

    let failed = called_emsg.get() > called_emsg_before && did_emsg.get() != 0;
    if failed && did_throw.get() {
        discard_current_exception();
    }
    did_emsg.set(save_did_emsg);
    set_pressedreturn(save_ex_pressedreturn);
    clear_local(&mut rettv);

    // The timer may have been closed and removed while its callback ran;
    // then there is nothing left to account to it.
    let too_many_errors = with_timer(number, |timer| {
        timer.busy -= 1;
        if failed {
            timer.emsg_count += 1;
        }
        timer.emsg_count >= MAX_ERRORS
    });
    if too_many_errors == Some(true) {
        timer_stop(number);
    }

    // A zero timeout does not repeat by itself; it is re-armed here so
    // that it yields to the event loop between runs.
    with_timer(number, |timer| {
        if !timer.stopped && timer.timeout == 0 {
            if let Some(watcher) = &mut timer.watcher {
                watcher.start(timer_due_cb, 0, 0);
            }
        }
    });
}

/// Start a timer running `callback`, which it takes over, answering its id.
pub fn timer_start(timeout: int64_t, repeat_count: c_int, callback: Callback) -> uint64_t {
    let id = last_timer_id.get();
    last_timer_id.set(id.wrapping_add(1));
    let mut watcher = OwnedTimer::new(TimerQueue::OwnChild, true, id);
    let repeat = timeout as uint64_t;
    watcher.start(timer_due_cb, repeat, repeat);
    let timer = Timer {
        watcher: Some(watcher),
        timer_id: id as c_int,
        repeat_count,
        busy: 0,
        emsg_count: 0,
        timeout,
        stopped: false,
        paused: false,
        callback,
    };
    timers.with_mut(|map| map.insert(id, timer));
    id
}

/// Pause the timer numbered `id`, or let it run again.
pub(crate) fn timer_pause(id: VarNumber, paused: bool) {
    with_timer(id, |timer| {
        if let Some(watcher) = &mut timer.watcher {
            if !timer.paused && paused {
                watcher.stop();
            } else if timer.paused && !paused {
                let every = timer.timeout as uint64_t;
                watcher.start(timer_due_cb, every, every);
            }
        }
        timer.paused = paused;
    });
}

/// Stop the timer numbered `id`. It leaves the table once the event loop
/// has closed its watcher.
pub fn timer_stop(id: VarNumber) {
    let watcher = with_timer(id, |timer| {
        if timer.stopped {
            return None;
        }
        timer.stopped = true;
        timer.watcher.take()
    })
    .flatten();
    if let Some(mut watcher) = watcher {
        watcher.stop();
        watcher.close(Some(timer_close_cb));
    }
}

/// The watcher of the timer numbered `id` is closed: the timer goes.
fn timer_close_cb(id: uint64_t) {
    if let Some(mut timer) = timers.with_mut(|map| map.remove(&id)) {
        callback_free(&mut timer.callback);
    }
}

/// Stop every timer.
pub fn timer_stop_all() {
    for id in timers.with(SlotTable::snapshot_keys) {
        timer_stop(id as VarNumber);
    }
}

/// Shut the timers down at exit.
pub fn timer_teardown() {
    timer_stop_all()
}
