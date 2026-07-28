//! `timer_start()` and what fires when one is due.

#[allow(unused_imports)]
use super::*;

#[inline]
pub(crate) unsafe extern "C" fn map_get_uint64_t_ptr_t(
    mut map: *mut Map_uint64_t_ptr_t,
    mut key: uint64_t,
) -> ptr_t {
    let mut k: uint32_t = mh_get_uint64_t(&raw mut (*map).set, key);
    return if k == MH_TOMBSTONE as uint32_t {
        value_init_ptr_t.get()
    } else {
        *(*map).values.offset(k as isize)
    };
}

#[inline]
pub(crate) unsafe extern "C" fn map_put_uint64_t_ptr_t(
    mut map: *mut Map_uint64_t_ptr_t,
    mut key: uint64_t,
    mut value: ptr_t,
) {
    let mut val: *mut ptr_t = map_put_ref_uint64_t_ptr_t(
        map,
        key,
        ::core::ptr::null_mut::<*mut uint64_t>(),
        ::core::ptr::null_mut::<bool>(),
    );
    *val = value;
}

pub unsafe extern "C" fn find_timer_by_nr(mut xx: varnumber_T) -> *mut timer_T {
    return map_get_uint64_t_ptr_t(timers.ptr(), xx as uint64_t) as *mut timer_T;
}

pub unsafe extern "C" fn add_timer_info(mut rettv: *mut typval_T, mut timer: *mut timer_T) {
    let mut list: *mut list_T = (*rettv).vval.v_list;
    let mut dict: *mut dict_T = tv_dict_alloc();
    tv_list_append_dict(list, dict);
    tv_dict_add_nr(
        dict,
        b"id\0".as_ptr() as *const c_char,
        ::core::mem::size_of::<[c_char; 3]>().wrapping_sub(1 as size_t),
        (*timer).timer_id as varnumber_T,
    );
    tv_dict_add_nr(
        dict,
        b"time\0".as_ptr() as *const c_char,
        ::core::mem::size_of::<[c_char; 5]>().wrapping_sub(1 as size_t),
        (*timer).timeout as varnumber_T,
    );
    tv_dict_add_nr(
        dict,
        b"paused\0".as_ptr() as *const c_char,
        ::core::mem::size_of::<[c_char; 7]>().wrapping_sub(1 as size_t),
        (*timer).paused as varnumber_T,
    );
    tv_dict_add_nr(
        dict,
        b"repeat\0".as_ptr() as *const c_char,
        ::core::mem::size_of::<[c_char; 7]>().wrapping_sub(1 as size_t),
        (if (*timer).repeat_count < 0 as c_int {
            -1 as c_int
        } else {
            (*timer).repeat_count
        }) as varnumber_T,
    );
    let mut di: *mut dictitem_T = tv_dict_item_alloc(b"callback\0".as_ptr() as *const c_char);
    if tv_dict_add(dict, di) == FAIL {
        xfree(di as *mut c_void);
        return;
    }
    callback_put(&raw mut (*timer).callback, &raw mut (*di).di_tv);
}

pub unsafe extern "C" fn add_timer_info_all(mut rettv: *mut typval_T) {
    tv_list_alloc_ret(rettv, (*timers.ptr()).set.h.size as ptrdiff_t);
    let mut timer: *mut timer_T = ::core::ptr::null_mut::<timer_T>();
    let mut __i: uint32_t = 0;
    __i = 0 as uint32_t;
    while __i < (*timers.ptr()).set.h.n_keys {
        timer = *(*timers.ptr()).values.offset(__i as isize) as *mut timer_T;
        if !(*timer).stopped || (*timer).refcount > 1 as c_int {
            add_timer_info(rettv, timer);
        }
        __i = __i.wrapping_add(1);
    }
}

pub unsafe extern "C" fn timer_due_cb(mut _tw: *mut TimeWatcher, mut data: *mut c_void) {
    let mut timer: *mut timer_T = data as *mut timer_T;
    let mut save_did_emsg: c_int = did_emsg.get();
    let called_emsg_before: c_int = called_emsg.get();
    let save_ex_pressedreturn: bool = get_pressedreturn();
    if (*timer).stopped as c_int != 0 || (*timer).paused as c_int != 0 {
        return;
    }
    (*timer).refcount += 1;
    if (*timer).repeat_count >= 0 as c_int && {
        (*timer).repeat_count -= 1;
        (*timer).repeat_count == 0 as c_int
    } {
        timer_stop(timer);
    }
    let mut argv: [typval_T; 2] = [
        typval_T {
            v_type: VAR_UNKNOWN,
            v_lock: VAR_UNLOCKED,
            vval: typval_vval_union { v_number: 0 },
        },
        typval_T {
            v_type: VAR_UNKNOWN,
            v_lock: VAR_UNLOCKED,
            vval: typval_vval_union { v_number: 0 },
        },
    ];
    argv[0 as c_int as usize].v_type = VAR_NUMBER;
    argv[0 as c_int as usize].vval.v_number = (*timer).timer_id as varnumber_T;
    let mut rettv: typval_T = typval_T {
        v_type: VAR_UNKNOWN,
        v_lock: VAR_UNLOCKED,
        vval: typval_vval_union { v_number: 0 },
    };
    callback_call(
        &raw mut (*timer).callback,
        1 as c_int,
        &raw mut argv as *mut typval_T,
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
    if (*timer).emsg_count >= 3 as c_int {
        timer_stop(timer);
    }
    tv_clear(&raw mut rettv);
    if !(*timer).stopped && (*timer).timeout == 0 as int64_t {
        time_watcher_start(
            &raw mut (*timer).tw,
            Some(timer_due_cb as unsafe extern "C" fn(*mut TimeWatcher, *mut c_void) -> ()),
            0 as uint64_t,
            0 as uint64_t,
        );
    }
    timer_decref(timer);
}

pub unsafe extern "C" fn timer_start(
    timeout: int64_t,
    repeat_count: c_int,
    callback: *const Callback,
) -> uint64_t {
    let mut timer: *mut timer_T = xmalloc(::core::mem::size_of::<timer_T>()) as *mut timer_T;
    (*timer).refcount = 1 as c_int;
    (*timer).stopped = false_0 != 0;
    (*timer).paused = false_0 != 0;
    (*timer).emsg_count = 0 as c_int;
    (*timer).repeat_count = repeat_count;
    (*timer).timeout = timeout;
    let c2rust_fresh17 = last_timer_id.get();
    last_timer_id.set((*last_timer_id.ptr()).wrapping_add(1));
    (*timer).timer_id = c2rust_fresh17 as c_int;
    (*timer).callback = *callback;
    time_watcher_init(main_loop.ptr(), &raw mut (*timer).tw, timer as *mut c_void);
    (*timer).tw.events = multiqueue_new_child((*main_loop.ptr()).events);
    (*timer).tw.blockable = true_0 != 0;
    time_watcher_start(
        &raw mut (*timer).tw,
        Some(timer_due_cb as unsafe extern "C" fn(*mut TimeWatcher, *mut c_void) -> ()),
        timeout as uint64_t,
        timeout as uint64_t,
    );
    map_put_uint64_t_ptr_t(timers.ptr(), (*timer).timer_id as uint64_t, timer as ptr_t);
    return (*timer).timer_id as uint64_t;
}

pub unsafe extern "C" fn timer_stop(mut timer: *mut timer_T) {
    if (*timer).stopped {
        return;
    }
    (*timer).stopped = true_0 != 0;
    time_watcher_stop(&raw mut (*timer).tw);
    time_watcher_close(
        &raw mut (*timer).tw,
        Some(timer_close_cb as unsafe extern "C" fn(*mut TimeWatcher, *mut c_void) -> ()),
    );
}

pub(crate) unsafe extern "C" fn timer_close_cb(mut _tw: *mut TimeWatcher, mut data: *mut c_void) {
    let mut timer: *mut timer_T = data as *mut timer_T;
    multiqueue_free((*timer).tw.events);
    callback_free(&raw mut (*timer).callback);
    map_del_uint64_t_ptr_t(
        timers.ptr(),
        (*timer).timer_id as uint64_t,
        ::core::ptr::null_mut::<uint64_t>(),
    );
    timer_decref(timer);
}

pub(crate) unsafe extern "C" fn timer_decref(mut timer: *mut timer_T) {
    (*timer).refcount -= 1;
    if (*timer).refcount == 0 as c_int {
        xfree(timer as *mut c_void);
    }
}

pub unsafe extern "C" fn timer_stop_all() {
    let mut timer: *mut timer_T = ::core::ptr::null_mut::<timer_T>();
    let mut __i: uint32_t = 0;
    __i = 0 as uint32_t;
    while __i < (*timers.ptr()).set.h.n_keys {
        timer = *(*timers.ptr()).values.offset(__i as isize) as *mut timer_T;
        timer_stop(timer);
        __i = __i.wrapping_add(1);
    }
}

pub unsafe extern "C" fn timer_teardown() {
    timer_stop_all();
}
