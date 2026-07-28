//! The garbage collector: marking every value reachable from a root,
//! then freeing the lists and dicts nothing marked.
//!
//! `copyID` is the mark. Anything that can hold a reference has a
//! `set_ref_in_*` that stamps it and recurses.

#[allow(unused_imports)]
use super::*;

pub unsafe extern "C" fn get_copyID() -> c_int {
    static current_copyID: GlobalCell<c_int> = GlobalCell::new(0 as c_int);
    (*current_copyID.ptr()) += COPYID_INC;
    return current_copyID.get();
}

pub unsafe extern "C" fn garbage_collect(mut testing: bool) -> bool {
    let mut abort_0: bool = false_0 != 0;
    if !testing {
        want_garbage_collect.set(false_0 != 0);
        may_garbage_collect.set(false_0 != 0);
        garbage_collect_at_exit.set(false_0 != 0);
    }
    if (*exestack.ptr()).ga_maxlen - (*exestack.ptr()).ga_len > 500 as c_int {
        let mut n: c_int = (*exestack.ptr()).ga_len / 2 as c_int;
        if n < (*exestack.ptr()).ga_growsize {
            n = (*exestack.ptr()).ga_growsize;
        }
        if (*exestack.ptr()).ga_len + n < (*exestack.ptr()).ga_maxlen {
            let mut new_len: size_t = ((*exestack.ptr()).ga_itemsize as size_t)
                .wrapping_mul(((*exestack.ptr()).ga_len + n) as size_t);
            let mut pp: *mut c_char = xrealloc((*exestack.ptr()).ga_data, new_len) as *mut c_char;
            (*exestack.ptr()).ga_maxlen = (*exestack.ptr()).ga_len + n;
            (*exestack.ptr()).ga_data = pp as *mut c_void;
        }
    }
    let copyID: c_int = get_copyID();
    abort_0 = abort_0 as c_int != 0 || set_ref_in_previous_funccal(copyID) as c_int != 0;
    abort_0 = abort_0 as c_int != 0 || garbage_collect_scriptvars(copyID) as c_int != 0;
    let mut buf: *mut buf_T = firstbuf.get();
    while !buf.is_null() {
        abort_0 = abort_0 as c_int != 0
            || set_ref_in_item(
                &raw mut (*buf).b_bufvar.di_tv,
                copyID,
                ::core::ptr::null_mut::<*mut ht_stack_T>(),
                ::core::ptr::null_mut::<*mut list_stack_T>(),
            ) as c_int
                != 0;
        abort_0 = abort_0 as c_int != 0
            || set_ref_in_callback(
                &raw mut (*buf).b_prompt_callback,
                copyID,
                ::core::ptr::null_mut::<*mut ht_stack_T>(),
                ::core::ptr::null_mut::<*mut list_stack_T>(),
            ) as c_int
                != 0;
        abort_0 = abort_0 as c_int != 0
            || set_ref_in_callback(
                &raw mut (*buf).b_prompt_interrupt,
                copyID,
                ::core::ptr::null_mut::<*mut ht_stack_T>(),
                ::core::ptr::null_mut::<*mut list_stack_T>(),
            ) as c_int
                != 0;
        abort_0 = abort_0 as c_int != 0
            || set_ref_in_callback(
                &raw mut (*buf).b_cfu_cb,
                copyID,
                ::core::ptr::null_mut::<*mut ht_stack_T>(),
                ::core::ptr::null_mut::<*mut list_stack_T>(),
            ) as c_int
                != 0;
        abort_0 = abort_0 as c_int != 0
            || set_ref_in_callback(
                &raw mut (*buf).b_ofu_cb,
                copyID,
                ::core::ptr::null_mut::<*mut ht_stack_T>(),
                ::core::ptr::null_mut::<*mut list_stack_T>(),
            ) as c_int
                != 0;
        abort_0 = abort_0 as c_int != 0
            || set_ref_in_callback(
                &raw mut (*buf).b_tsrfu_cb,
                copyID,
                ::core::ptr::null_mut::<*mut ht_stack_T>(),
                ::core::ptr::null_mut::<*mut list_stack_T>(),
            ) as c_int
                != 0;
        abort_0 = abort_0 as c_int != 0
            || set_ref_in_callback(
                &raw mut (*buf).b_tfu_cb,
                copyID,
                ::core::ptr::null_mut::<*mut ht_stack_T>(),
                ::core::ptr::null_mut::<*mut list_stack_T>(),
            ) as c_int
                != 0;
        abort_0 = abort_0 as c_int != 0
            || set_ref_in_callback(
                &raw mut (*buf).b_ffu_cb,
                copyID,
                ::core::ptr::null_mut::<*mut ht_stack_T>(),
                ::core::ptr::null_mut::<*mut list_stack_T>(),
            ) as c_int
                != 0;
        if !abort_0 && !(*buf).b_p_cpt_cb.is_null() {
            abort_0 = abort_0 as c_int != 0
                || set_ref_in_cpt_callbacks((*buf).b_p_cpt_cb, (*buf).b_p_cpt_count, copyID)
                    as c_int
                    != 0;
        }
        buf = (*buf).b_next;
    }
    abort_0 = abort_0 as c_int != 0 || set_ref_in_insexpand_funcs(copyID) as c_int != 0;
    abort_0 = abort_0 as c_int != 0 || set_ref_in_opfunc(copyID) as c_int != 0;
    abort_0 = abort_0 as c_int != 0 || set_ref_in_tagfunc(copyID) as c_int != 0;
    abort_0 = abort_0 as c_int != 0 || set_ref_in_findfunc(copyID) as c_int != 0;
    let mut tp: *mut tabpage_T = first_tabpage.get() as *mut tabpage_T;
    while !tp.is_null() {
        let mut wp: *mut win_T = if tp == curtab.get() {
            firstwin.get()
        } else {
            (*tp).tp_firstwin
        };
        while !wp.is_null() {
            abort_0 = abort_0 as c_int != 0
                || set_ref_in_item(
                    &raw mut (*wp).w_winvar.di_tv,
                    copyID,
                    ::core::ptr::null_mut::<*mut ht_stack_T>(),
                    ::core::ptr::null_mut::<*mut list_stack_T>(),
                ) as c_int
                    != 0;
            wp = (*wp).w_next;
        }
        tp = (*tp).tp_next as *mut tabpage_T;
    }
    let mut i: c_int = 0 as c_int;
    while i < (*aucmd_win_vec.ptr()).size as c_int {
        if !(*(*aucmd_win_vec.ptr()).items.offset(i as isize))
            .auc_win
            .is_null()
        {
            abort_0 = abort_0 as c_int != 0
                || set_ref_in_item(
                    &raw mut (*(*(*aucmd_win_vec.ptr()).items.offset(i as isize)).auc_win)
                        .w_winvar
                        .di_tv,
                    copyID,
                    ::core::ptr::null_mut::<*mut ht_stack_T>(),
                    ::core::ptr::null_mut::<*mut list_stack_T>(),
                ) as c_int
                    != 0;
        }
        i += 1;
    }
    let mut reg_iter: *const c_void = ::core::ptr::null::<c_void>();
    loop {
        let mut reg: yankreg_T = yankreg_T {
            y_array: ::core::ptr::null_mut::<String_0>(),
            y_size: 0,
            y_type: kMTCharWise,
            y_width: 0,
            timestamp: 0,
            additional_data: ::core::ptr::null_mut::<AdditionalData>(),
        };
        let mut name: c_char = NUL as c_char;
        let mut is_unnamed: bool = false_0 != 0;
        reg_iter = op_global_reg_iter(reg_iter, &raw mut name, &raw mut reg, &raw mut is_unnamed);
        if reg_iter.is_null() {
            break;
        }
    }
    let mut mark_iter: *const c_void = ::core::ptr::null::<c_void>();
    loop {
        let mut fm: xfmark_T = xfmark_T {
            fmark: fmark_T {
                mark: pos_T {
                    lnum: 0,
                    col: 0,
                    coladd: 0,
                },
                fnum: 0,
                timestamp: 0,
                view: fmarkv_T {
                    topline_offset: 0,
                    skipcol: 0,
                },
                additional_data: ::core::ptr::null_mut::<AdditionalData>(),
            },
            fname: ::core::ptr::null_mut::<c_char>(),
        };
        let mut name_0: c_char = NUL as c_char;
        mark_iter = mark_global_iter(mark_iter, &raw mut name_0, &raw mut fm);
        if mark_iter.is_null() {
            break;
        }
    }
    let mut tp_0: *mut tabpage_T = first_tabpage.get() as *mut tabpage_T;
    while !tp_0.is_null() {
        abort_0 = abort_0 as c_int != 0
            || set_ref_in_item(
                &raw mut (*tp_0).tp_winvar.di_tv,
                copyID,
                ::core::ptr::null_mut::<*mut ht_stack_T>(),
                ::core::ptr::null_mut::<*mut list_stack_T>(),
            ) as c_int
                != 0;
        tp_0 = (*tp_0).tp_next as *mut tabpage_T;
    }
    abort_0 = abort_0 as c_int != 0 || garbage_collect_globvars(copyID) != 0;
    abort_0 = abort_0 as c_int != 0 || set_ref_in_call_stack(copyID) as c_int != 0;
    abort_0 = abort_0 as c_int != 0 || set_ref_in_functions(copyID) as c_int != 0;
    let mut data: *mut Channel = ::core::ptr::null_mut::<Channel>();
    let mut __i: uint32_t = 0;
    __i = 0 as uint32_t;
    while __i < (*channels.ptr()).set.h.n_keys {
        data = *(*channels.ptr()).values.offset(__i as isize) as *mut Channel;
        set_ref_in_callback_reader(
            &raw mut (*data).on_data,
            copyID,
            ::core::ptr::null_mut::<*mut ht_stack_T>(),
            ::core::ptr::null_mut::<*mut list_stack_T>(),
        );
        set_ref_in_callback_reader(
            &raw mut (*data).on_stderr,
            copyID,
            ::core::ptr::null_mut::<*mut ht_stack_T>(),
            ::core::ptr::null_mut::<*mut list_stack_T>(),
        );
        set_ref_in_callback(
            &raw mut (*data).on_exit,
            copyID,
            ::core::ptr::null_mut::<*mut ht_stack_T>(),
            ::core::ptr::null_mut::<*mut list_stack_T>(),
        );
        __i = __i.wrapping_add(1);
    }
    let mut timer: *mut timer_T = ::core::ptr::null_mut::<timer_T>();
    let mut __i_0: uint32_t = 0;
    __i_0 = 0 as uint32_t;
    while __i_0 < (*timers.ptr()).set.h.n_keys {
        timer = *(*timers.ptr()).values.offset(__i_0 as isize) as *mut timer_T;
        set_ref_in_callback(
            &raw mut (*timer).callback,
            copyID,
            ::core::ptr::null_mut::<*mut ht_stack_T>(),
            ::core::ptr::null_mut::<*mut list_stack_T>(),
        );
        __i_0 = __i_0.wrapping_add(1);
    }
    abort_0 = abort_0 as c_int != 0 || set_ref_in_func_args(copyID) as c_int != 0;
    abort_0 = abort_0 as c_int != 0 || garbage_collect_vimvars(copyID) as c_int != 0;
    abort_0 = abort_0 as c_int != 0 || set_ref_in_quickfix(copyID) as c_int != 0;
    let mut did_free: bool = false_0 != 0;
    if !abort_0 {
        did_free = free_unref_items(copyID) != 0;
        did_free =
            free_unref_funccal(copyID, testing as c_int) as c_int != 0 || did_free as c_int != 0;
    } else if p_verbose.get() > 0 as OptInt {
        verb_msg(gettext(
            b"Not enough memory to set references, garbage collection aborted!\0".as_ptr()
                as *const c_char,
        ));
    }
    return did_free;
}

pub(crate) unsafe extern "C" fn free_unref_items(mut copyID: c_int) -> c_int {
    let mut did_free: bool = false_0 != 0;
    tv_in_free_unref_items.set(true_0 != 0);
    let mut dd: *mut dict_T = gc_first_dict.get();
    while !dd.is_null() {
        if (*dd).dv_copyID & COPYID_MASK != copyID & COPYID_MASK {
            tv_dict_free_contents(dd);
            did_free = true_0 != 0;
        }
        dd = (*dd).dv_used_next;
    }
    let mut ll: *mut list_T = gc_first_list.get();
    while !ll.is_null() {
        if tv_list_copyid(ll) & COPYID_MASK != copyID & COPYID_MASK && !tv_list_has_watchers(ll) {
            tv_list_free_contents(ll);
            did_free = true_0 != 0;
        }
        ll = (*ll).lv_used_next;
    }
    let mut dd_next: *mut dict_T = ::core::ptr::null_mut::<dict_T>();
    let mut dd_0: *mut dict_T = gc_first_dict.get();
    while !dd_0.is_null() {
        dd_next = (*dd_0).dv_used_next;
        if (*dd_0).dv_copyID & COPYID_MASK != copyID & COPYID_MASK {
            tv_dict_free_dict(dd_0);
        }
        dd_0 = dd_next;
    }
    let mut ll_next: *mut list_T = ::core::ptr::null_mut::<list_T>();
    let mut ll_0: *mut list_T = gc_first_list.get();
    while !ll_0.is_null() {
        ll_next = (*ll_0).lv_used_next;
        if (*ll_0).lv_copyID & COPYID_MASK != copyID & COPYID_MASK && !tv_list_has_watchers(ll_0) {
            tv_list_free_list(ll_0);
        }
        ll_0 = ll_next;
    }
    tv_in_free_unref_items.set(false_0 != 0);
    return did_free as c_int;
}

pub unsafe extern "C" fn set_ref_in_ht(
    mut ht: *mut hashtab_T,
    mut copyID: c_int,
    mut list_stack: *mut *mut list_stack_T,
) -> bool {
    let mut abort_0: bool = false_0 != 0;
    let mut ht_stack: *mut ht_stack_T = ::core::ptr::null_mut::<ht_stack_T>();
    let mut cur_ht: *mut hashtab_T = ht;
    loop {
        if !abort_0 {
            let hiht_: *mut hashtab_T = cur_ht;
            let mut hitodo_: size_t = (*hiht_).ht_used;
            let mut hi: *mut hashitem_T = (*hiht_).ht_array;
            while hitodo_ != 0 {
                if !((*hi).hi_key.is_null()
                    || (*hi).hi_key == &raw const hash_removed as *mut c_char)
                {
                    hitodo_ = hitodo_.wrapping_sub(1);
                    abort_0 = abort_0 as c_int != 0
                        || set_ref_in_item(
                            &raw mut (*((*hi).hi_key.offset(-(17 as c_ulong as isize))
                                as *mut dictitem_T))
                                .di_tv,
                            copyID,
                            &raw mut ht_stack,
                            list_stack,
                        ) as c_int
                            != 0;
                }
                hi = hi.offset(1);
            }
        }
        if ht_stack.is_null() {
            break;
        }
        cur_ht = (*ht_stack).ht;
        let mut tempitem: *mut ht_stack_T = ht_stack;
        ht_stack = (*ht_stack).prev as *mut ht_stack_T;
        xfree(tempitem as *mut c_void);
    }
    return abort_0;
}

pub unsafe extern "C" fn set_ref_in_list_items(
    mut l: *mut list_T,
    mut copyID: c_int,
    mut ht_stack: *mut *mut ht_stack_T,
) -> bool {
    let mut abort_0: bool = false_0 != 0;
    let mut list_stack: *mut list_stack_T = ::core::ptr::null_mut::<list_stack_T>();
    let mut cur_l: *mut list_T = l;
    loop {
        let l_: *mut list_T = cur_l;
        if !l_.is_null() {
            let mut li: *mut listitem_T = (*l_).lv_first;
            while !li.is_null() {
                if abort_0 {
                    break;
                }
                abort_0 =
                    set_ref_in_item(&raw mut (*li).li_tv, copyID, ht_stack, &raw mut list_stack);
                li = (*li).li_next;
            }
        }
        if list_stack.is_null() {
            break;
        }
        cur_l = (*list_stack).list;
        let mut tempitem: *mut list_stack_T = list_stack;
        list_stack = (*list_stack).prev as *mut list_stack_T;
        xfree(tempitem as *mut c_void);
    }
    return abort_0;
}

pub(crate) unsafe extern "C" fn set_ref_in_item_dict(
    mut dd: *mut dict_T,
    mut copyID: c_int,
    mut ht_stack: *mut *mut ht_stack_T,
    mut list_stack: *mut *mut list_stack_T,
) -> bool {
    if dd.is_null() || (*dd).dv_copyID == copyID {
        return false_0 != 0;
    }
    (*dd).dv_copyID = copyID;
    if ht_stack.is_null() {
        return set_ref_in_ht(&raw mut (*dd).dv_hashtab, copyID, list_stack);
    }
    let newitem: *mut ht_stack_T = xmalloc(::core::mem::size_of::<ht_stack_T>()) as *mut ht_stack_T;
    (*newitem).ht = &raw mut (*dd).dv_hashtab;
    (*newitem).prev = *ht_stack as *mut ht_stack_S;
    *ht_stack = newitem;
    let mut w: *mut QUEUE = ::core::ptr::null_mut::<QUEUE>();
    let mut watcher: *mut DictWatcher = ::core::ptr::null_mut::<DictWatcher>();
    w = (*dd).watchers.next as *mut QUEUE;
    while w != &raw mut (*dd).watchers {
        let mut next: *mut QUEUE = (*w).next as *mut QUEUE;
        watcher = tv_dict_watcher_node_data(w);
        set_ref_in_callback(&raw mut (*watcher).callback, copyID, ht_stack, list_stack);
        w = next;
    }
    return false_0 != 0;
}

pub(crate) unsafe extern "C" fn set_ref_in_item_list(
    mut ll: *mut list_T,
    mut copyID: c_int,
    mut ht_stack: *mut *mut ht_stack_T,
    mut list_stack: *mut *mut list_stack_T,
) -> bool {
    if ll.is_null() || (*ll).lv_copyID == copyID {
        return false_0 != 0;
    }
    (*ll).lv_copyID = copyID;
    if list_stack.is_null() {
        return set_ref_in_list_items(ll, copyID, ht_stack);
    }
    let newitem: *mut list_stack_T =
        xmalloc(::core::mem::size_of::<list_stack_T>()) as *mut list_stack_T;
    (*newitem).list = ll;
    (*newitem).prev = *list_stack as *mut list_stack_S;
    *list_stack = newitem;
    return false_0 != 0;
}

pub(crate) unsafe extern "C" fn set_ref_in_item_partial(
    mut pt: *mut partial_T,
    mut copyID: c_int,
    mut ht_stack: *mut *mut ht_stack_T,
    mut list_stack: *mut *mut list_stack_T,
) -> bool {
    if pt.is_null() || (*pt).pt_copyID == copyID {
        return false_0 != 0;
    }
    (*pt).pt_copyID = copyID;
    let mut abort_0: bool = set_ref_in_func((*pt).pt_name, (*pt).pt_func, copyID);
    if !(*pt).pt_dict.is_null() {
        let mut dtv: typval_T = typval_T {
            v_type: VAR_UNKNOWN,
            v_lock: VAR_UNLOCKED,
            vval: typval_vval_union { v_number: 0 },
        };
        dtv.v_type = VAR_DICT;
        dtv.vval.v_dict = (*pt).pt_dict;
        abort_0 = abort_0 as c_int != 0
            || set_ref_in_item(&raw mut dtv, copyID, ht_stack, list_stack) as c_int != 0;
    }
    let mut i: c_int = 0 as c_int;
    while i < (*pt).pt_argc {
        abort_0 = abort_0 as c_int != 0
            || set_ref_in_item(
                (*pt).pt_argv.offset(i as isize),
                copyID,
                ht_stack,
                list_stack,
            ) as c_int
                != 0;
        i += 1;
    }
    return abort_0;
}

pub unsafe extern "C" fn set_ref_in_item(
    mut tv: *mut typval_T,
    mut copyID: c_int,
    mut ht_stack: *mut *mut ht_stack_T,
    mut list_stack: *mut *mut list_stack_T,
) -> bool {
    let mut abort_0: bool = false_0 != 0;
    match (*tv).v_type as c_uint {
        5 => return set_ref_in_item_dict((*tv).vval.v_dict, copyID, ht_stack, list_stack),
        4 => return set_ref_in_item_list((*tv).vval.v_list, copyID, ht_stack, list_stack),
        3 => {
            abort_0 = set_ref_in_func(
                (*tv).vval.v_string,
                ::core::ptr::null_mut::<ufunc_T>(),
                copyID,
            );
        }
        9 => {
            return set_ref_in_item_partial((*tv).vval.v_partial, copyID, ht_stack, list_stack);
        }
        0 | 7 | 8 | 6 | 1 | 2 | 10 | _ => {}
    }
    return abort_0;
}

pub unsafe extern "C" fn var_item_copy(
    conv: *const vimconv_T,
    from: *mut typval_T,
    to: *mut typval_T,
    deep: bool,
    copyID: c_int,
) -> c_int {
    static recurse: GlobalCell<c_int> = GlobalCell::new(0 as c_int);
    let mut ret: c_int = OK;
    if recurse.get() >= DICT_MAXNEST {
        emsg(gettext(
            (e_variable_nested_too_deep_for_making_copy.ptr() as *const _) as *const c_char,
        ));
        return FAIL;
    }
    (*recurse.ptr()) += 1;
    match (*from).v_type as c_uint {
        1 | 6 | 3 | 9 | 7 | 8 => {
            tv_copy(from, to);
        }
        2 => {
            if conv.is_null()
                || (*conv).vc_type == CONV_NONE as c_int
                || (*from).vval.v_string.is_null()
            {
                tv_copy(from, to);
            } else {
                (*to).v_type = VAR_STRING;
                (*to).v_lock = VAR_UNLOCKED;
                (*to).vval.v_string = string_convert(
                    conv as *mut vimconv_T,
                    (*from).vval.v_string,
                    ::core::ptr::null_mut::<size_t>(),
                );
                if (*to).vval.v_string.is_null() {
                    (*to).vval.v_string = xstrdup((*from).vval.v_string);
                }
            }
        }
        4 => {
            (*to).v_type = VAR_LIST;
            (*to).v_lock = VAR_UNLOCKED;
            if (*from).vval.v_list.is_null() {
                (*to).vval.v_list = ::core::ptr::null_mut::<list_T>();
            } else if copyID != 0 as c_int && tv_list_copyid((*from).vval.v_list) == copyID {
                (*to).vval.v_list = tv_list_latest_copy((*from).vval.v_list);
                tv_list_ref((*to).vval.v_list);
            } else {
                (*to).vval.v_list = tv_list_copy(conv, (*from).vval.v_list, deep, copyID);
            }
            if (*to).vval.v_list.is_null() && !(*from).vval.v_list.is_null() {
                ret = FAIL;
            }
        }
        10 => {
            tv_blob_copy((*from).vval.v_blob, to);
        }
        5 => {
            (*to).v_type = VAR_DICT;
            (*to).v_lock = VAR_UNLOCKED;
            if (*from).vval.v_dict.is_null() {
                (*to).vval.v_dict = ::core::ptr::null_mut::<dict_T>();
            } else if copyID != 0 as c_int && (*(*from).vval.v_dict).dv_copyID == copyID {
                (*to).vval.v_dict = (*(*from).vval.v_dict).dv_copydict;
                (*(*to).vval.v_dict).dv_refcount += 1;
            } else {
                (*to).vval.v_dict = tv_dict_copy(conv, (*from).vval.v_dict, deep, copyID);
            }
            if (*to).vval.v_dict.is_null() && !(*from).vval.v_dict.is_null() {
                ret = FAIL;
            }
        }
        0 => {
            internal_error(b"var_item_copy(UNKNOWN)\0".as_ptr() as *const c_char);
            ret = FAIL;
        }
        _ => {}
    }
    (*recurse.ptr()) -= 1;
    return ret;
}

#[inline]
pub(crate) unsafe extern "C" fn tv_list_latest_copy(l: *const list_T) -> *mut list_T {
    return (*l).lv_copylist;
}

#[inline]
pub(crate) unsafe extern "C" fn tv_list_has_watchers(l: *const list_T) -> bool {
    return !l.is_null() && !(*l).lv_watch.is_null();
}
