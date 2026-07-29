//! `timer_start()` and what fires when one is due.
//!
//! A `timer_T` is reference counted rather than owned by the map, because
//! it has to survive its own callback: `timer_due_cb` takes a reference for
//! the duration of the call, and `timer_stop` may run inside that call and
//! hand the map's reference to `timer_close_cb`.

#![deny(unsafe_op_in_unsafe_fn)]

use core::ffi::{c_int, c_void};
use core::mem::size_of;
use core::ptr::null_mut;

use crate::src::nvim::eval::typval::{
    callback_free, callback_put, tv_clear, tv_dict_add, tv_dict_add_nr, tv_dict_alloc,
    tv_dict_item_alloc, tv_list_alloc_ret, tv_list_append_dict,
};
use crate::src::nvim::eval::{
    FAIL, MH_TOMBSTONE, VAR_NUMBER, VAR_UNKNOWN, VAR_UNLOCKED, callback_call, last_timer_id, timers,
};
use crate::src::nvim::event::multiqueue::{multiqueue_free, multiqueue_new_child};
use crate::src::nvim::event::time::{
    time_watcher_close, time_watcher_init, time_watcher_start, time_watcher_stop,
};
use crate::src::nvim::ex_docmd::{get_pressedreturn, set_pressedreturn};
use crate::src::nvim::ex_eval::discard_current_exception;
use crate::src::nvim::main::{called_emsg, did_emsg, did_throw, main_loop};
use crate::src::nvim::map::{map_del_uint64_t_ptr_t, map_put_ref_uint64_t_ptr_t, mh_get_uint64_t};
use crate::src::nvim::memory::{xfree, xmalloc};
use crate::src::nvim::types::{
    Callback, Map_uint64_t_ptr_t, TimeWatcher, dict_T, dictitem_T, int64_t, list_T, ptr_t,
    ptrdiff_t, size_t, timer_T, typval_T, typval_vval_union, uint32_t, uint64_t, varnumber_T,
};

/// How many consecutive errors a timer's callback may raise before the
/// timer is stopped for good.
const MAX_ERRORS: c_int = 3;

/// A freshly declared typval.
const UNSET_TV: typval_T = typval_T {
    v_type: VAR_UNKNOWN,
    v_lock: VAR_UNLOCKED,
    vval: typval_vval_union { v_number: 0 },
};

/// The timer map's lookup. A miss answers null: the "default value" global
/// every `map_get` expansion carries is a `static` nothing ever writes.
///
/// # Safety
/// `map` must be valid.
#[inline]
unsafe fn timers_get(map: *mut Map_uint64_t_ptr_t, key: uint64_t) -> ptr_t {
    unsafe {
        let k: uint32_t = mh_get_uint64_t(&raw mut (*map).set, key);
        if k == MH_TOMBSTONE {
            null_mut()
        } else {
            *(*map).values.offset(k as isize)
        }
    }
}

/// The timer map's insert.
///
/// # Safety
/// `map` must be valid.
#[inline]
unsafe fn timers_put(map: *mut Map_uint64_t_ptr_t, key: uint64_t, value: ptr_t) {
    unsafe {
        let slot = map_put_ref_uint64_t_ptr_t(map, key, null_mut(), null_mut());
        *slot = value;
    }
}

/// The timer with this id, or null.
///
/// # Safety
/// Called with the timer map initialised, which it always is.
pub unsafe fn find_timer_by_nr(id: varnumber_T) -> *mut timer_T {
    unsafe { timers_get(timers.ptr(), id as uint64_t) as *mut timer_T }
}

/// Append one timer's description to the List in `rettv`.
///
/// # Safety
/// `rettv` must hold a List; `timer` must be valid.
pub unsafe fn add_timer_info(rettv: *mut typval_T, timer: *mut timer_T) {
    unsafe {
        let list: *mut list_T = (*rettv).vval.v_list;
        let dict: *mut dict_T = tv_dict_alloc();
        tv_list_append_dict(list, dict);

        for (key, value) in [
            (c"id", (*timer).timer_id as varnumber_T),
            (c"time", (*timer).timeout as varnumber_T),
            (c"paused", (*timer).paused as varnumber_T),
            // A negative repeat count means "for ever", reported as -1.
            (c"repeat", (*timer).repeat_count.max(-1) as varnumber_T),
        ] {
            tv_dict_add_nr(dict, key.as_ptr(), key.count_bytes() as size_t, value);
        }

        let di: *mut dictitem_T = tv_dict_item_alloc(c"callback".as_ptr());
        if tv_dict_add(dict, di) == FAIL {
            xfree(di as *mut c_void);
            return;
        }
        callback_put(&raw mut (*timer).callback, &raw mut (*di).di_tv);
    }
}

/// Fill `rettv` with a List describing every live timer.
///
/// # Safety
/// `rettv` must be valid.
pub unsafe fn add_timer_info_all(rettv: *mut typval_T) {
    unsafe {
        let map = timers.ptr();
        tv_list_alloc_ret(rettv, (*map).set.h.size as ptrdiff_t);
        for i in 0..(*map).set.h.n_keys {
            let timer = *(*map).values.offset(i as isize) as *mut timer_T;
            // A stopped timer is still listed while something else holds a
            // reference to it — its own callback, for instance.
            if !(*timer).stopped || (*timer).refcount > 1 {
                add_timer_info(rettv, timer);
            }
        }
    }
}

/// The timer is due: run its callback.
///
/// # Safety
/// Called by the event loop with `data` a live `timer_T`.
pub unsafe extern "C" fn timer_due_cb(_tw: *mut TimeWatcher, data: *mut c_void) {
    unsafe {
        let timer = data as *mut timer_T;
        let save_did_emsg = did_emsg.get();
        let called_emsg_before = called_emsg.get();
        let save_ex_pressedreturn = get_pressedreturn();
        if (*timer).stopped || (*timer).paused {
            return;
        }

        // The reference is what keeps the timer alive if its own callback
        // stops it.
        (*timer).refcount += 1;
        if (*timer).repeat_count >= 0 && {
            (*timer).repeat_count -= 1;
            (*timer).repeat_count == 0
        } {
            timer_stop(timer);
        }

        let mut argv = [UNSET_TV; 2];
        argv[0].v_type = VAR_NUMBER;
        argv[0].vval.v_number = (*timer).timer_id as varnumber_T;
        let mut rettv = UNSET_TV;
        callback_call(
            &raw mut (*timer).callback,
            1,
            argv.as_mut_ptr(),
            &raw mut rettv,
        );

        if called_emsg.get() > called_emsg_before && did_emsg.get() != 0 {
            (*timer).emsg_count += 1;
            if did_throw.get() {
                discard_current_exception();
            }
        }
        did_emsg.set(save_did_emsg);
        set_pressedreturn(save_ex_pressedreturn);
        if (*timer).emsg_count >= MAX_ERRORS {
            timer_stop(timer);
        }
        tv_clear(&raw mut rettv);

        // A zero timeout does not repeat by itself; it is re-armed here so
        // that it yields to the event loop between runs.
        if !(*timer).stopped && (*timer).timeout == 0 {
            time_watcher_start(&raw mut (*timer).tw, Some(timer_due_cb), 0, 0);
        }
        timer_decref(timer);
    }
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
    unsafe {
        let timer = xmalloc(size_of::<timer_T>()) as *mut timer_T;
        (*timer).refcount = 1;
        (*timer).stopped = false;
        (*timer).paused = false;
        (*timer).emsg_count = 0;
        (*timer).repeat_count = repeat_count;
        (*timer).timeout = timeout;
        (*timer).timer_id = last_timer_id.get() as c_int;
        last_timer_id.set(last_timer_id.get().wrapping_add(1));
        (*timer).callback = *callback;

        time_watcher_init(main_loop.ptr(), &raw mut (*timer).tw, timer as *mut c_void);
        (*timer).tw.events = multiqueue_new_child((*main_loop.ptr()).events);
        (*timer).tw.blockable = true;
        time_watcher_start(
            &raw mut (*timer).tw,
            Some(timer_due_cb),
            timeout as uint64_t,
            timeout as uint64_t,
        );

        timers_put(timers.ptr(), (*timer).timer_id as uint64_t, timer as ptr_t);
        (*timer).timer_id as uint64_t
    }
}

/// Stop a timer. The map's reference is released asynchronously, once the
/// event loop has closed the watcher.
///
/// # Safety
/// `timer` must be valid.
pub unsafe fn timer_stop(timer: *mut timer_T) {
    unsafe {
        if (*timer).stopped {
            return;
        }
        (*timer).stopped = true;
        time_watcher_stop(&raw mut (*timer).tw);
        time_watcher_close(&raw mut (*timer).tw, Some(timer_close_cb));
    }
}

/// The watcher is closed: drop the map's reference.
///
/// # Safety
/// Called by the event loop with `data` the `timer_T` being closed.
pub(crate) unsafe extern "C" fn timer_close_cb(_tw: *mut TimeWatcher, data: *mut c_void) {
    unsafe {
        let timer = data as *mut timer_T;
        multiqueue_free((*timer).tw.events);
        callback_free(&raw mut (*timer).callback);
        map_del_uint64_t_ptr_t(timers.ptr(), (*timer).timer_id as uint64_t, null_mut());
        timer_decref(timer);
    }
}

/// Drop one reference to a timer, freeing it at zero.
///
/// # Safety
/// `timer` must be valid and hold a reference this call takes over.
pub(crate) unsafe fn timer_decref(timer: *mut timer_T) {
    unsafe {
        (*timer).refcount -= 1;
        if (*timer).refcount == 0 {
            xfree(timer as *mut c_void);
        }
    }
}

/// Stop every timer.
///
/// # Safety
/// Called with the timer map initialised.
pub unsafe fn timer_stop_all() {
    unsafe {
        let map = timers.ptr();
        for i in 0..(*map).set.h.n_keys {
            timer_stop(*(*map).values.offset(i as isize) as *mut timer_T);
        }
    }
}

/// Shut the timers down at exit.
///
/// # Safety
/// As `timer_stop_all`.
pub unsafe fn timer_teardown() {
    unsafe { timer_stop_all() }
}
