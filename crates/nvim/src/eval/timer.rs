//! `timer_start()` and what fires when one is due.
//!
//! A `timer_T` is reference counted rather than owned by the map, because
//! it has to survive its own callback: `timer_due_cb` takes a reference for
//! the duration of the call, and `timer_stop` may run inside that call and
//! hand the map's reference to `timer_close_cb`.
//!
//! The registry itself is a [`SlotTable`]: every walk below goes over a
//! snapshot, because a timer callback runs Vimscript and Vimscript can start
//! and stop timers. Nothing holds a borrow of the table across one.

#![deny(unsafe_op_in_unsafe_fn)]

use core::ffi::{c_int, c_void};
use core::mem::{offset_of, size_of};
use core::ptr::null_mut;

use crate::eval::typval::{
    callback_free, callback_put, tv_dict_add, tv_dict_add_nr, tv_dict_alloc, tv_dict_item_alloc,
    tv_list_alloc_ret, tv_list_append_dict,
};
use crate::eval::vars::clear_local;
use crate::eval::{Tm, Tv, callback_call, last_timer_id, timers};
use crate::event::multiqueue::{multiqueue_free, multiqueue_new_child};
use crate::event::time::{
    time_watcher_close, time_watcher_init, time_watcher_start, time_watcher_stop,
};
use crate::ex_docmd::{get_pressedreturn, set_pressedreturn};
use crate::ex_eval::discard_current_exception;
use crate::main::{called_emsg, did_emsg, did_throw, main_loop};
use crate::memory::{xfree, xmalloc};
use crate::registry::SlotTable;
use crate::types::{
    Callback, Refcount, TimeWatcher, VAR_NUMBER, VAR_UNKNOWN, VarLock, dict_T, dictitem_T, int64_t,
    ptrdiff_t, size_t, timer_T, typval_T, typval_vval_union, uint64_t, varnumber_T,
};

/// How many consecutive errors a timer's callback may raise before the
/// timer is stopped for good.
const MAX_ERRORS: c_int = 3;

/// A freshly declared typval.
const UNSET_TV: typval_T = typval_T {
    v_type: VAR_UNKNOWN,
    v_lock: VarLock::Unlocked,
    vval: typval_vval_union { v_number: 0 },
};

/// Every live timer, in registration order.
///
/// A snapshot rather than a borrow: each caller below runs the editor
/// between one timer and the next, and that can register or drop timers.
fn timer_snapshot() -> Vec<*mut timer_T> {
    timers.with(SlotTable::snapshot_values)
}

/// The timer with this id, or null.
pub fn find_timer_by_nr(id: varnumber_T) -> *mut timer_T {
    timers
        .with(|map| map.get(&(id as uint64_t)))
        .unwrap_or(null_mut())
}

/// Append one timer's description to the List in `rettv`.
///
/// # Safety
/// `rettv` must hold a List; `timer` must be valid.
pub unsafe fn add_timer_info(rettv: *mut typval_T, timer: *mut timer_T) {
    // SAFETY: the caller's promise -- both pointees outlive the call.
    let (rettv, timer) = unsafe { (Tv::new(rettv), Tm::new(timer)) };
    // SAFETY: `tv_dict_alloc` never answers NULL.
    let dict: *mut dict_T = unsafe { tv_dict_alloc() };
    // SAFETY: the caller's promise that `rettv` holds a List, so `v_list` is
    // the union's live arm; the append takes over the dictionary's
    // reference.
    unsafe { tv_list_append_dict(rettv.list_or_null(), dict) };

    for (key, value) in [
        (c"id", timer.timer_id as varnumber_T),
        (c"time", timer.timeout as varnumber_T),
        (c"paused", timer.paused as varnumber_T),
        // A negative repeat count means "for ever", reported as -1.
        (c"repeat", timer.repeat_count.max(-1) as varnumber_T),
    ] {
        let len = key.count_bytes() as size_t;
        // SAFETY: `dict` is the dictionary just appended, and `key` is a
        // NUL-terminated literal `len` bytes long.
        let _ = unsafe { tv_dict_add_nr(dict, key.as_ptr(), len, value) };
    }

    // SAFETY: `tv_dict_item_alloc` never answers NULL.
    let di: *mut dictitem_T = unsafe { tv_dict_item_alloc(c"callback".as_ptr()) };
    // SAFETY: `di` is the item just allocated, and it is freed again here
    // when the dictionary refuses it.
    if unsafe { tv_dict_add(dict, di) }.is_err() {
        // SAFETY: nothing took the item over.
        unsafe { xfree(di as *mut c_void) };
        return;
    }
    let cb: *mut Callback = timer.field_ptr(offset_of!(timer_T, callback));
    // SAFETY: `cb` is the timer's own callback and `di` the item just added.
    unsafe { callback_put(cb, &raw mut (*di).di_tv) };
}

/// Fill `rettv` with a List describing every live timer.
///
/// # Safety
/// `rettv` must be valid.
pub unsafe fn add_timer_info_all(rettv: *mut typval_T) {
    let live = timer_snapshot();
    // SAFETY: the caller's promise about `rettv`.
    unsafe { tv_list_alloc_ret(rettv, live.len() as ptrdiff_t) };
    for timer in live {
        // SAFETY: the snapshot holds registered timers, and nothing has run
        // between taking it and reading it.
        let timer = unsafe { Tm::new(timer) };
        // A stopped timer is still listed while something else holds a
        // reference to it — its own callback, for instance.
        if !timer.stopped || timer.refcount.is_shared() {
            // SAFETY: as above; `rettv` now holds the List.
            unsafe { add_timer_info(rettv, timer.raw()) };
        }
    }
}

/// The timer is due: run its callback.
///
/// # Safety
/// Called by the event loop with `data` a live `timer_T`.
pub unsafe fn timer_due_cb(_tw: *mut TimeWatcher, data: *mut c_void) {
    // SAFETY: the event loop's promise -- `data` is the live timer this
    // watcher was armed with.
    let mut timer = unsafe { Tm::new(data as *mut timer_T) };
    let save_did_emsg = did_emsg.get();
    let called_emsg_before = called_emsg.get();
    let save_ex_pressedreturn = get_pressedreturn();
    if timer.stopped || timer.paused {
        return;
    }

    // The reference is what keeps the timer alive if its own callback
    // stops it.
    timer.refcount.retain();
    if timer.repeat_count >= 0 && {
        timer.repeat_count -= 1;
        timer.repeat_count == 0
    } {
        // SAFETY: the reference taken above keeps the timer live.
        unsafe { timer_stop(timer.raw()) };
    }

    let mut argv = [UNSET_TV; 2];
    argv[0].v_type = VAR_NUMBER;
    argv[0].vval.v_number = timer.timer_id as varnumber_T;
    let mut rettv = UNSET_TV;
    let cb: *mut Callback = timer.field_ptr(offset_of!(timer_T, callback));
    // SAFETY: `cb` is the timer's own callback, kept live by the reference
    // above; `argv` and `rettv` are this frame's.
    unsafe { callback_call(cb, 1, argv.as_mut_ptr(), &raw mut rettv) };

    if called_emsg.get() > called_emsg_before && did_emsg.get() != 0 {
        timer.emsg_count += 1;
        if did_throw.get() {
            unsafe { discard_current_exception() };
        }
    }
    did_emsg.set(save_did_emsg);
    set_pressedreturn(save_ex_pressedreturn);
    if timer.emsg_count >= MAX_ERRORS {
        // SAFETY: as above.
        unsafe { timer_stop(timer.raw()) };
    }
    // SAFETY: `rettv` is this frame's, filled in by the callback.
    clear_local(&mut rettv);

    // A zero timeout does not repeat by itself; it is re-armed here so
    // that it yields to the event loop between runs.
    if !timer.stopped && timer.timeout == 0 {
        let tw: *mut TimeWatcher = timer.field_ptr(offset_of!(timer_T, tw));
        // SAFETY: `tw` is the timer's own watcher.
        unsafe { time_watcher_start(tw, Some(timer_due_cb), 0, 0) };
    }
    // SAFETY: this hands back the reference taken above.
    unsafe { timer_decref(timer.raw()) };
}

/// Start a timer, answering its id.
///
/// # Safety
/// `callback` must be valid; its ownership moves into the timer.
pub unsafe fn timer_start(
    timeout: int64_t,
    repeat_count: c_int,
    callback: *const Callback,
) -> uint64_t {
    // SAFETY: `xmalloc` never answers NULL and the block is one `timer_T`;
    // every field is written below before anything reads one.
    let mut timer = unsafe { Tm::new(xmalloc(size_of::<timer_T>()) as *mut timer_T) };
    timer.refcount = Refcount::ONE;
    timer.stopped = false;
    timer.paused = false;
    timer.emsg_count = 0;
    timer.repeat_count = repeat_count;
    timer.timeout = timeout;
    timer.timer_id = last_timer_id.get() as c_int;
    last_timer_id.set(last_timer_id.get().wrapping_add(1));
    // SAFETY: the caller's promise about `callback`.
    timer.callback = unsafe { (*callback).clone() };

    let tw: *mut TimeWatcher = timer.field_ptr(offset_of!(timer_T, tw));
    // SAFETY: the loop lives from startup to exit, `tw` is the timer's own
    // watcher, and the timer is the data it is armed with.
    unsafe { time_watcher_init(main_loop.ptr(), tw, timer.raw() as *mut c_void) };
    // The loop now holds `tw`, so the two writes below go through it rather
    // than through `DerefMut`, which would borrow the whole `timer_T` and
    // pop the address the loop is holding — `winlayer::live`'s note.
    // SAFETY: as above -- the loop's queue is live and `tw` is the timer's.
    unsafe { (*tw).events = multiqueue_new_child((*main_loop.ptr()).events) };
    // SAFETY: as above.
    unsafe { (*tw).blockable = true };
    let repeat = timeout as uint64_t;
    // SAFETY: `tw` is the timer's own watcher, initialised above.
    unsafe { time_watcher_start(tw, Some(timer_due_cb), repeat, repeat) };

    let id = timer.timer_id as uint64_t;
    timers.with_mut(|map| map.insert(id, timer.raw()));
    id
}

/// Stop a timer. The map's reference is released asynchronously, once the
/// event loop has closed the watcher.
///
/// # Safety
/// `timer` must be valid.
pub unsafe fn timer_stop(timer: *mut timer_T) {
    // SAFETY: the caller's promise -- a live timer.
    let mut timer = unsafe { Tm::new(timer) };
    if timer.stopped {
        return;
    }
    timer.stopped = true;
    let tw: *mut TimeWatcher = timer.field_ptr(offset_of!(timer_T, tw));
    // SAFETY: `tw` is the timer's own watcher.
    unsafe { time_watcher_stop(tw) };
    // SAFETY: as above; `timer_close_cb` takes over the map's reference
    // once the loop has closed the watcher.
    unsafe { time_watcher_close(tw, Some(timer_close_cb)) };
}

/// The watcher is closed: drop the map's reference.
///
/// # Safety
/// Called by the event loop with `data` the `timer_T` being closed.
pub(crate) unsafe fn timer_close_cb(_tw: *mut TimeWatcher, data: *mut c_void) {
    // SAFETY: the event loop's promise -- `data` is the timer being closed.
    let timer = unsafe { Tm::new(data as *mut timer_T) };
    // SAFETY: the watcher's queue is the timer's own child queue.
    unsafe { multiqueue_free(timer.tw.events) };
    let cb: *mut Callback = timer.field_ptr(offset_of!(timer_T, callback));
    // SAFETY: `cb` is the timer's own callback.
    unsafe { callback_free(cb) };
    let id = timer.timer_id as uint64_t;
    let _ = timers.with_mut(|map| map.remove(&id));
    // SAFETY: this hands back the map's reference.
    unsafe { timer_decref(timer.raw()) };
}

/// Drop one reference to a timer, freeing it at zero.
///
/// # Safety
/// `timer` must be valid and hold a reference this call takes over.
pub(crate) unsafe fn timer_decref(timer: *mut timer_T) {
    // SAFETY: the caller's promise -- a live timer.
    let mut timer = unsafe { Tm::new(timer) };
    if timer.refcount.release() == 0 {
        // SAFETY: the last reference has gone, so nothing can reach it.
        unsafe { xfree(timer.raw() as *mut c_void) };
    }
}

/// Stop every timer.
///
/// # Safety
/// Called from the main thread, with every registered timer live.
pub unsafe fn timer_stop_all() {
    for timer in timer_snapshot() {
        // SAFETY: the caller's promise; the snapshot is taken while the
        // table is untouched, and `timer_stop` only queues the removal.
        unsafe { timer_stop(timer) };
    }
}

/// Shut the timers down at exit.
///
/// # Safety
/// As [`timer_stop_all`].
pub unsafe fn timer_teardown() {
    unsafe { timer_stop_all() }
}
