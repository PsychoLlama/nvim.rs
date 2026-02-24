//! Timer functions migrated from eval_shim.c (Phase 1, eval_shim pass 8).
//!
//! Implements `find_timer_by_nr`, `add_timer_info`, `add_timer_info_all`,
//! `timer_due_cb`, `timer_start`, `timer_stop`, `timer_close_cb`,
//! `timer_stop_all`, and `timer_teardown`.

#![allow(clippy::cast_sign_loss)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_possible_wrap)]

use std::ffi::{c_char, c_int, c_void};

// =============================================================================
// Opaque handle types
// =============================================================================

/// Opaque pointer to timer_T.
type TimerHandle = *mut c_void;
/// Opaque pointer to typval_T.
type TvHandle = *mut c_void;
/// Opaque pointer to Callback.
type CallbackHandle = *mut c_void;
/// Opaque pointer to dict_T.
type DictHandle = *mut c_void;
/// Opaque pointer to dictitem_T.
type DictItemHandle = *mut c_void;
/// Opaque pointer to list_T.
type ListHandle = *mut c_void;
/// Opaque pointer to TimeWatcher.
type TimeWatcherHandle = *mut c_void;

// =============================================================================
// C Extern Declarations
// =============================================================================

extern "C" {
    // -- timer_T field accessors --
    fn nvim_timer_get_id(timer: TimerHandle) -> c_int;
    fn nvim_timer_set_id(timer: TimerHandle, id: c_int);
    fn nvim_timer_get_repeat_count(timer: TimerHandle) -> c_int;
    fn nvim_timer_set_repeat_count(timer: TimerHandle, count: c_int);
    fn nvim_timer_get_refcount(timer: TimerHandle) -> c_int;
    fn nvim_timer_set_refcount(timer: TimerHandle, refcount: c_int);
    fn nvim_timer_get_emsg_count(timer: TimerHandle) -> c_int;
    fn nvim_timer_set_emsg_count(timer: TimerHandle, count: c_int);
    fn nvim_timer_get_timeout(timer: TimerHandle) -> i64;
    fn nvim_timer_get_stopped(timer: TimerHandle) -> c_int;
    fn nvim_timer_set_stopped(timer: TimerHandle, stopped: c_int);
    fn nvim_timer_get_paused(timer: TimerHandle) -> c_int;
    fn nvim_timer_get_callback_ptr(timer: TimerHandle) -> CallbackHandle;
    fn nvim_timer_set_callback(timer: TimerHandle, cb: CallbackHandle);
    fn nvim_timer_alloc() -> TimerHandle;
    fn nvim_timer_free(timer: TimerHandle);

    // -- TimeWatcher compound operations --
    fn nvim_timer_tw_init(timer: TimerHandle);
    fn nvim_timer_tw_start(timer: TimerHandle, timeout: u64, repeat: u64);
    fn nvim_timer_tw_stop(timer: TimerHandle);
    fn nvim_timer_tw_close(timer: TimerHandle);
    fn nvim_timer_tw_set_events_child(timer: TimerHandle);
    fn nvim_timer_tw_set_blockable(timer: TimerHandle, blockable: c_int);
    fn nvim_timer_tw_free_events(timer: TimerHandle);

    // -- Timer map operations --
    fn nvim_timers_get(id: i64) -> TimerHandle;
    fn nvim_timers_put(timer: TimerHandle);
    fn nvim_timers_del(id: i64);
    fn nvim_timers_size() -> usize;
    fn nvim_timers_next_id() -> u64;
    fn nvim_timers_foreach(cb: unsafe extern "C" fn(TimerHandle, *mut c_void), userdata: *mut c_void);

    // -- typval operations --
    fn nvim_tv_set_number(tv: TvHandle, num: i64);
    fn nvim_tv_init(tv: TvHandle);
    fn tv_clear(tv: TvHandle);

    // -- Dict/List operations for add_timer_info --
    fn nvim_tv_list_alloc_ret(rettv: TvHandle, count_hint: isize) -> ListHandle;
    fn nvim_tv_list_append_dict(list: ListHandle, dict: DictHandle);
    fn nvim_tv_get_v_list(tv: TvHandle) -> ListHandle;
    fn nvim_tv_dict_add_nr(dict: DictHandle, key: *const c_char, key_len: usize, nr: i64);
    fn nvim_tv_dict_alloc() -> DictHandle;
    fn nvim_tv_dict_item_alloc_key(key: *const c_char) -> DictItemHandle;
    fn nvim_tv_dict_add_item(dict: DictHandle, di: DictItemHandle) -> c_int;
    fn nvim_di_get_tv_ptr(di: DictItemHandle) -> TvHandle;
    fn nvim_di_free(di: DictItemHandle);

    // -- Callback operations --
    fn nvim_callback_free(cb: CallbackHandle);
    fn nvim_callback_put(cb: CallbackHandle, tv: TvHandle);
    fn callback_call(callback: CallbackHandle, argcount: c_int, argvars: TvHandle, rettv: TvHandle) -> bool;

    // -- Error state accessors --
    fn nvim_get_did_emsg() -> c_int;
    fn nvim_set_did_emsg(val: c_int);
    fn nvim_get_called_emsg() -> c_int;
    fn nvim_get_did_throw() -> c_int;
    fn nvim_get_pressedreturn() -> c_int;
    fn nvim_set_pressedreturn(val: c_int);
    fn nvim_discard_current_exception();

    // -- timer_stop (calls rs_timer_stop via C) --
    // We use rs_timer_stop directly in recursion (timer_due_cb calls timer_stop).
    // To avoid forward reference issues, we call through C wrapper.
}

// =============================================================================
// Constants
// =============================================================================

/// Size of typval_T in bytes (16 bytes, verified by C static assert).
const TYPVAL_SIZE: usize = 16;

/// FAIL return value from C.
const FAIL: c_int = 0;

// =============================================================================
// Implementation
// =============================================================================

/// Look up a timer by its numeric ID.
///
/// # Safety
/// Safe to call from C.
#[no_mangle]
pub unsafe extern "C" fn rs_find_timer_by_nr(xx: i64) -> TimerHandle {
    nvim_timers_get(xx)
}

/// Add information about a single timer to the return list.
///
/// # Safety
/// `rettv` must be a valid typval_T pointer with v_list set.
/// `timer` must be a valid timer_T pointer.
#[no_mangle]
pub unsafe extern "C" fn rs_add_timer_info(rettv: TvHandle, timer: TimerHandle) {
    let list = nvim_tv_get_v_list(rettv);

    let dict = nvim_tv_dict_alloc();
    nvim_tv_list_append_dict(list, dict);

    let id = nvim_timer_get_id(timer) as i64;
    let timeout = nvim_timer_get_timeout(timer);
    let paused = nvim_timer_get_paused(timer) as i64;
    let repeat_count = nvim_timer_get_repeat_count(timer);
    let repeat_val = if repeat_count < 0 { -1i64 } else { repeat_count as i64 };

    nvim_tv_dict_add_nr(dict, c"id".as_ptr(), 2, id);
    nvim_tv_dict_add_nr(dict, c"time".as_ptr(), 4, timeout);
    nvim_tv_dict_add_nr(dict, c"paused".as_ptr(), 6, paused);
    nvim_tv_dict_add_nr(dict, c"repeat".as_ptr(), 6, repeat_val);

    let di = nvim_tv_dict_item_alloc_key(c"callback".as_ptr());
    if nvim_tv_dict_add_item(dict, di) == FAIL {
        nvim_di_free(di);
        return;
    }

    let di_tv = nvim_di_get_tv_ptr(di);
    let cb_ptr = nvim_timer_get_callback_ptr(timer);
    nvim_callback_put(cb_ptr, di_tv);
}

/// Add information about all timers to the return typval (which becomes a list).
///
/// # Safety
/// `rettv` must be a valid typval_T pointer.
#[no_mangle]
pub unsafe extern "C" fn rs_add_timer_info_all(rettv: TvHandle) {
    nvim_tv_list_alloc_ret(rettv, nvim_timers_size() as isize);

    unsafe extern "C" fn foreach_cb(timer: TimerHandle, userdata: *mut c_void) {
        let stopped = nvim_timer_get_stopped(timer) != 0;
        let refcount = nvim_timer_get_refcount(timer);
        if !stopped || refcount > 1 {
            rs_add_timer_info(userdata as TvHandle, timer);
        }
    }

    nvim_timers_foreach(foreach_cb, rettv);
}

/// Decrement the timer's refcount and free it if it reaches 0.
///
/// # Safety
/// `timer` must be a valid timer_T pointer.
unsafe fn timer_decref(timer: TimerHandle) {
    let refcount = nvim_timer_get_refcount(timer) - 1;
    nvim_timer_set_refcount(timer, refcount);
    if refcount == 0 {
        nvim_timer_free(timer);
    }
}

/// Timer close callback -- invoked by libuv after time_watcher_close completes.
///
/// Frees the timer's event queue, callback, and removes it from the map.
///
/// # Safety
/// This is called from libuv. `tw` and `data` are valid pointers.
#[no_mangle]
pub unsafe extern "C" fn rs_timer_close_cb(_tw: TimeWatcherHandle, data: *mut c_void) {
    let timer = data as TimerHandle;
    nvim_timer_tw_free_events(timer);
    let cb_ptr = nvim_timer_get_callback_ptr(timer);
    nvim_callback_free(cb_ptr);
    let id = nvim_timer_get_id(timer) as i64;
    nvim_timers_del(id);
    timer_decref(timer);
}

/// Stop a timer: mark it stopped, stop the time watcher, and schedule close.
///
/// # Safety
/// `timer` must be a valid timer_T pointer.
#[no_mangle]
pub unsafe extern "C" fn rs_timer_stop(timer: TimerHandle) {
    if nvim_timer_get_stopped(timer) != 0 {
        // avoid double free
        return;
    }
    nvim_timer_set_stopped(timer, 1);
    nvim_timer_tw_stop(timer);
    nvim_timer_tw_close(timer);
}

/// Timer due callback -- invoked by libuv when the timer fires.
///
/// This is the main timer callback. It calls the user's Vimscript callback.
///
/// # Safety
/// Called from libuv. `_tw` is the TimeWatcher, `data` is the timer_T pointer.
#[no_mangle]
pub unsafe extern "C" fn rs_timer_due_cb(_tw: TimeWatcherHandle, data: *mut c_void) {
    let timer = data as TimerHandle;

    let save_did_emsg = nvim_get_did_emsg();
    let called_emsg_before = nvim_get_called_emsg();
    let save_ex_pressedreturn = nvim_get_pressedreturn();

    if nvim_timer_get_stopped(timer) != 0 || nvim_timer_get_paused(timer) != 0 {
        return;
    }

    // Increment refcount to keep timer alive during callback
    let refcount = nvim_timer_get_refcount(timer);
    nvim_timer_set_refcount(timer, refcount + 1);

    // if repeat was negative, repeat forever; otherwise count down
    let repeat_count = nvim_timer_get_repeat_count(timer);
    if repeat_count >= 0 {
        let new_repeat = repeat_count - 1;
        nvim_timer_set_repeat_count(timer, new_repeat);
        if new_repeat == 0 {
            rs_timer_stop(timer);
        }
    }

    // Build argv[2] on the stack. typval_T is 16 bytes.
    let mut argv = [0u8; TYPVAL_SIZE * 2];
    let argv0: TvHandle = argv.as_mut_ptr().cast();
    let argv1: TvHandle = argv.as_mut_ptr().add(TYPVAL_SIZE).cast();

    // Initialize both typvals
    nvim_tv_init(argv0);
    nvim_tv_init(argv1);

    // Set argv[0] = timer->timer_id as VAR_NUMBER
    let timer_id = nvim_timer_get_id(timer) as i64;
    nvim_tv_set_number(argv0, timer_id);

    // rettv
    let mut rettv = [0u8; TYPVAL_SIZE];
    let rettv_ptr: TvHandle = rettv.as_mut_ptr().cast();
    nvim_tv_init(rettv_ptr);

    let cb_ptr = nvim_timer_get_callback_ptr(timer);
    callback_call(cb_ptr, 1, argv0, rettv_ptr);

    // Handle error message
    let called_emsg_now = nvim_get_called_emsg();
    let did_emsg_now = nvim_get_did_emsg();
    if called_emsg_now > called_emsg_before && did_emsg_now != 0 {
        let emsg_count = nvim_timer_get_emsg_count(timer);
        nvim_timer_set_emsg_count(timer, emsg_count + 1);
        if nvim_get_did_throw() != 0 {
            nvim_discard_current_exception();
        }
    }
    nvim_set_did_emsg(save_did_emsg);
    nvim_set_pressedreturn(save_ex_pressedreturn);

    if nvim_timer_get_emsg_count(timer) >= 3 {
        rs_timer_stop(timer);
    }

    tv_clear(rettv_ptr);

    // timeout==0: requeue for next event loop tick
    if nvim_timer_get_stopped(timer) == 0 && nvim_timer_get_timeout(timer) == 0 {
        nvim_timer_tw_start(timer, 0, 0);
    }

    timer_decref(timer);
}

/// Start a new timer and return its ID.
///
/// # Safety
/// `callback` must be a valid Callback pointer.
#[no_mangle]
pub unsafe extern "C" fn rs_timer_start(
    timeout: i64,
    repeat_count: c_int,
    callback: CallbackHandle,
) -> u64 {
    let timer = nvim_timer_alloc();
    nvim_timer_set_refcount(timer, 1);
    nvim_timer_set_stopped(timer, 0);
    nvim_timer_set_emsg_count(timer, 0);
    nvim_timer_set_repeat_count(timer, repeat_count);
    nvim_timer_set_callback(timer, callback);

    let id = nvim_timers_next_id() as c_int;
    nvim_timer_set_id(timer, id);

    nvim_timer_tw_init(timer);
    nvim_timer_tw_set_events_child(timer);
    // if main loop is blocked, don't queue up multiple events
    nvim_timer_tw_set_blockable(timer, 1);
    nvim_timer_tw_start(timer, timeout as u64, timeout as u64);

    nvim_timers_put(timer);
    id as u64
}

/// Stop all active timers.
///
/// # Safety
/// Safe to call from C.
#[no_mangle]
pub unsafe extern "C" fn rs_timer_stop_all() {
    unsafe extern "C" fn foreach_cb(timer: TimerHandle, _userdata: *mut c_void) {
        rs_timer_stop(timer);
    }
    nvim_timers_foreach(foreach_cb, std::ptr::null_mut());
}

/// Teardown all timers (calls timer_stop_all).
///
/// # Safety
/// Safe to call from C.
#[no_mangle]
pub unsafe extern "C" fn rs_timer_teardown() {
    rs_timer_stop_all();
}
