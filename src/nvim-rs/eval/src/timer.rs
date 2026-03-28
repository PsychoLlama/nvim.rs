//! Timer functions migrated from eval_shim.c (Phase 1, eval_shim pass 8).
//!
//! Implements `find_timer_by_nr`, `add_timer_info`, `add_timer_info_all`,
//! `timer_due_cb`, `timer_start`, `timer_stop`, `timer_close_cb`,
//! `timer_stop_all`, and `timer_teardown`.

#![allow(clippy::cast_sign_loss)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_possible_wrap)]

use std::ffi::{c_char, c_int, c_void};

use super::typval::{tv_init as tv_init_typval, TypvalT as TypvalTRepr};

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
// TimerFields: bulk scalar field accessor (Phase 13)
// =============================================================================

/// Mirror of C `NvimTimerFields` -- scalar fields of `timer_T` only.
///
/// Layout must stay in sync with the C typedef in eval_shim.c.
/// Validated by `_Static_assert` in eval_shim.c.
#[repr(C)]
#[derive(Default, Clone, Copy)]
struct TimerFields {
    timer_id: c_int,
    repeat_count: c_int,
    refcount: c_int,
    emsg_count: c_int,
    timeout: i64,
    stopped: bool,
    paused: bool,
}

// =============================================================================
// C Extern Declarations
// =============================================================================

extern "C" {
    // -- Bulk timer field accessors (Phase 13) --
    fn nvim_timer_read_fields(timer: TimerHandle, out: *mut TimerFields);
    fn nvim_timer_write_fields(timer: TimerHandle, fields: *const TimerFields);

    // -- Retained individual accessors --
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
    fn nvim_timers_foreach(
        cb: unsafe extern "C" fn(TimerHandle, *mut c_void),
        userdata: *mut c_void,
    );

    // -- typval operations --
    fn nvim_tv_set_number(tv: TvHandle, num: i64);
    fn tv_clear(tv: TvHandle);

    // -- Dict/List operations for add_timer_info --
    #[link_name = "tv_list_alloc_ret"]
    fn nvim_tv_list_alloc_ret(rettv: TvHandle, count_hint: isize) -> ListHandle;
    #[link_name = "tv_list_append_dict"]
    fn nvim_tv_list_append_dict(list: ListHandle, dict: DictHandle);
    fn nvim_tv_dict_add_nr(dict: DictHandle, key: *const c_char, key_len: usize, nr: i64);
    #[link_name = "tv_dict_alloc"]
    fn nvim_tv_dict_alloc() -> DictHandle;
    #[link_name = "tv_dict_item_alloc"]
    fn nvim_tv_dict_item_alloc_key(key: *const c_char) -> DictItemHandle;
    #[link_name = "tv_dict_add"]
    fn nvim_tv_dict_add_item(dict: DictHandle, di: DictItemHandle) -> c_int;
    fn nvim_di_get_tv(di: DictItemHandle) -> TvHandle;
    #[link_name = "xfree"]
    fn nvim_tv_dict_item_free(di: DictItemHandle);

    // -- Callback operations --
    fn nvim_callback_free(cb: CallbackHandle);
    fn nvim_callback_put(cb: CallbackHandle, tv: TvHandle);
    fn callback_call(
        callback: CallbackHandle,
        argcount: c_int,
        argvars: TvHandle,
        rettv: TvHandle,
    ) -> bool;

    // -- Error state accessors --
    static mut did_emsg: c_int;
    static mut called_emsg: c_int;
    static mut did_throw: bool;
    fn nvim_get_pressedreturn() -> c_int;
    fn nvim_set_pressedreturn(val: c_int);
    #[link_name = "discard_current_exception"]
    fn nvim_discard_current_exception();
}

// =============================================================================
// Constants
// =============================================================================

/// Size of typval_T in bytes (16 bytes, verified by C static assert).
const TYPVAL_SIZE: usize = 16;

/// FAIL return value from C.
const FAIL: c_int = 0;

// =============================================================================
// Helper: read timer fields
// =============================================================================

/// Read all scalar fields from a timer into a `TimerFields`.
///
/// # Safety
/// `timer` must be a valid timer_T pointer.
#[inline]
unsafe fn read_fields(timer: TimerHandle) -> TimerFields {
    let mut f = TimerFields::default();
    nvim_timer_read_fields(timer, &raw mut f);
    f
}

/// Write all scalar fields from `TimerFields` back to a timer.
///
/// # Safety
/// `timer` must be a valid timer_T pointer.
#[inline]
unsafe fn write_fields(timer: TimerHandle, f: &TimerFields) {
    nvim_timer_write_fields(timer, std::ptr::from_ref::<TimerFields>(f));
}

// =============================================================================
// Implementation
// =============================================================================

/// Look up a timer by its numeric ID.
///
/// # Safety
/// Safe to call from C.
#[must_use]
#[export_name = "find_timer_by_nr"]
pub unsafe extern "C" fn rs_find_timer_by_nr(xx: i64) -> TimerHandle {
    nvim_timers_get(xx)
}

/// Add information about a single timer to the return list.
///
/// # Safety
/// `rettv` must be a valid typval_T pointer with v_list set.
/// `timer` must be a valid timer_T pointer.
#[export_name = "add_timer_info"]
pub unsafe extern "C" fn rs_add_timer_info(rettv: TvHandle, timer: TimerHandle) {
    let list = (*rettv.cast::<TypvalTRepr>().cast_const()).vval.v_list;

    let dict = nvim_tv_dict_alloc();
    nvim_tv_list_append_dict(list, dict);

    let f = read_fields(timer);
    let id = i64::from(f.timer_id);
    let timeout = f.timeout;
    let paused = i64::from(f.paused);
    let repeat_val = if f.repeat_count < 0 {
        -1i64
    } else {
        i64::from(f.repeat_count)
    };

    nvim_tv_dict_add_nr(dict, c"id".as_ptr(), 2, id);
    nvim_tv_dict_add_nr(dict, c"time".as_ptr(), 4, timeout);
    nvim_tv_dict_add_nr(dict, c"paused".as_ptr(), 6, paused);
    nvim_tv_dict_add_nr(dict, c"repeat".as_ptr(), 6, repeat_val);

    let di = nvim_tv_dict_item_alloc_key(c"callback".as_ptr());
    if nvim_tv_dict_add_item(dict, di) == FAIL {
        nvim_tv_dict_item_free(di);
        return;
    }

    let di_tv = nvim_di_get_tv(di);
    let cb_ptr = nvim_timer_get_callback_ptr(timer);
    nvim_callback_put(cb_ptr, di_tv);
}

/// Add information about all timers to the return typval (which becomes a list).
///
/// # Safety
/// `rettv` must be a valid typval_T pointer.
#[export_name = "add_timer_info_all"]
pub unsafe extern "C" fn rs_add_timer_info_all(rettv: TvHandle) {
    unsafe extern "C" fn foreach_cb(timer: TimerHandle, userdata: *mut c_void) {
        let f = read_fields(timer);
        if !f.stopped || f.refcount > 1 {
            rs_add_timer_info(userdata as TvHandle, timer);
        }
    }

    nvim_tv_list_alloc_ret(rettv, nvim_timers_size() as isize);
    nvim_timers_foreach(foreach_cb, rettv);
}

/// Decrement the timer's refcount and free it if it reaches 0.
///
/// # Safety
/// `timer` must be a valid timer_T pointer.
unsafe fn timer_decref(timer: TimerHandle) {
    let mut f = read_fields(timer);
    f.refcount -= 1;
    write_fields(timer, &f);
    if f.refcount == 0 {
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
    let f = read_fields(timer);
    nvim_timers_del(i64::from(f.timer_id));
    timer_decref(timer);
}

/// Stop a timer: mark it stopped, stop the time watcher, and schedule close.
///
/// # Safety
/// `timer` must be a valid timer_T pointer.
#[export_name = "timer_stop"]
pub unsafe extern "C" fn rs_timer_stop(timer: TimerHandle) {
    let f = read_fields(timer);
    if f.stopped {
        // avoid double free
        return;
    }
    let mut f2 = f;
    f2.stopped = true;
    write_fields(timer, &f2);
    nvim_timer_tw_stop(timer);
    nvim_timer_tw_close(timer);
}

/// Timer due callback -- invoked by libuv when the timer fires.
///
/// This is the main timer callback. It calls the user's Vimscript callback.
///
/// # Safety
/// Called from libuv. `_tw` is the TimeWatcher, `data` is the timer_T pointer.
#[export_name = "timer_due_cb"]
pub unsafe extern "C" fn rs_timer_due_cb(_tw: TimeWatcherHandle, data: *mut c_void) {
    let timer = data as TimerHandle;

    let save_did_emsg = did_emsg;
    let called_emsg_before = called_emsg;
    let save_ex_pressedreturn = nvim_get_pressedreturn();

    let f = read_fields(timer);
    if f.stopped || f.paused {
        return;
    }

    // Increment refcount to keep timer alive during callback
    let mut f = f;
    f.refcount += 1;

    // if repeat was negative, repeat forever; otherwise count down
    if f.repeat_count >= 0 {
        f.repeat_count -= 1;
        if f.repeat_count == 0 {
            write_fields(timer, &f);
            rs_timer_stop(timer);
            // Re-read after stop (stopped flag changed)
            f = read_fields(timer);
        }
    }

    // Write back updated refcount (and potentially repeat_count)
    write_fields(timer, &f);

    // Build argv[2] on the stack. typval_T is 16 bytes.
    let mut argv = [0u8; TYPVAL_SIZE * 2];
    let argv0: TvHandle = argv.as_mut_ptr().cast();
    let argv1: TvHandle = argv.as_mut_ptr().add(TYPVAL_SIZE).cast();

    // Initialize both typvals
    tv_init_typval(argv0);
    tv_init_typval(argv1);

    // Set argv[0] = timer->timer_id as VAR_NUMBER
    let timer_id = i64::from(f.timer_id);
    nvim_tv_set_number(argv0, timer_id);

    // rettv
    let mut rettv = [0u8; TYPVAL_SIZE];
    let rettv_ptr: TvHandle = rettv.as_mut_ptr().cast();
    tv_init_typval(rettv_ptr);

    let cb_ptr = nvim_timer_get_callback_ptr(timer);
    callback_call(cb_ptr, 1, argv0, rettv_ptr);

    // Handle error message
    let called_emsg_now = called_emsg;
    let did_emsg_now = did_emsg;
    if called_emsg_now > called_emsg_before && did_emsg_now != 0 {
        let mut f2 = read_fields(timer);
        f2.emsg_count += 1;
        write_fields(timer, &f2);
        if did_throw {
            nvim_discard_current_exception();
        }
    }
    did_emsg = save_did_emsg;
    nvim_set_pressedreturn(save_ex_pressedreturn);

    let f3 = read_fields(timer);
    if f3.emsg_count >= 3 {
        rs_timer_stop(timer);
    }

    tv_clear(rettv_ptr);

    // timeout==0: requeue for next event loop tick
    let f4 = read_fields(timer);
    if !f4.stopped && f4.timeout == 0 {
        nvim_timer_tw_start(timer, 0, 0);
    }

    timer_decref(timer);
}

/// Start a new timer and return its ID.
///
/// # Safety
/// `callback` must be a valid Callback pointer.
#[export_name = "timer_start"]
pub unsafe extern "C" fn rs_timer_start(
    timeout: i64,
    repeat_count: c_int,
    callback: CallbackHandle,
) -> u64 {
    let timer = nvim_timer_alloc();

    let f = TimerFields {
        timer_id: 0, // will be set below
        repeat_count,
        refcount: 1,
        emsg_count: 0,
        timeout,
        stopped: false,
        paused: false,
    };
    // Write initial scalar fields (timer_id set below after getting the id)
    write_fields(timer, &f);

    nvim_timer_set_callback(timer, callback);

    let id = nvim_timers_next_id() as c_int;
    let mut f2 = f;
    f2.timer_id = id;
    write_fields(timer, &f2);

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
#[export_name = "timer_stop_all"]
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
#[export_name = "timer_teardown"]
pub unsafe extern "C" fn rs_timer_teardown() {
    rs_timer_stop_all();
}
