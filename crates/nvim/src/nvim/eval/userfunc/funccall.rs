//! The funccall_T stack, the function table, and the GC roots.
//!
//! Three families that all read the same two globals.  `create_funccal` /
//! `cleanup_function_call` / `funccal_unref` own the funccall's lifetime --
//! including the case where a closure outlives the call that made it and
//! the funccall has to be kept alive with it.  `func_ref`/`func_unref` and
//! the `func_clear*` group own the `ufunc_T`'s.  The `set_ref_in_*` group
//! is what the garbage collector calls to mark everything reachable from a
//! call in progress, and `find_var_in_scoped_ht` is how a closure body
//! reaches the `l:` of the call it captured.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;

pub unsafe extern "C" fn func_init() {
    unsafe {
        hash_init(func_hashtab.ptr());
    }
}

pub unsafe extern "C" fn func_tbl_get() -> *mut hashtab_T {
    return func_hashtab.ptr();
}

unsafe extern "C" fn free_funccal(mut fc: *mut funccall_T) {
    unsafe {
        let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while i < (*fc).fc_ufuncs.ga_len {
            let mut fp: *mut ufunc_T =
                *((*fc).fc_ufuncs.ga_data as *mut *mut ufunc_T).offset(i as isize);
            if !fp.is_null() && (*fp).uf_scoped == fc {
                (*fp).uf_scoped = ::core::ptr::null_mut::<funccall_T>();
            }
            i += 1;
        }
        ga_clear(&raw mut (*fc).fc_ufuncs);
        func_ptr_unref((*fc).fc_func);
        xfree(fc as *mut ::core::ffi::c_void);
    }
}

unsafe extern "C" fn free_funccal_contents(mut fc: *mut funccall_T) {
    unsafe {
        vars_clear(&raw mut (*fc).fc_l_vars.dv_hashtab);
        vars_clear(&raw mut (*fc).fc_l_avars.dv_hashtab);
        let l_: *mut list_T = &raw mut (*fc).fc_l_varlist;
        if !l_.is_null() {
            let mut li: *mut listitem_T = (*l_).lv_first;
            while !li.is_null() {
                tv_clear(&raw mut (*li).li_tv);
                li = (*li).li_next;
            }
        }
        free_funccal(fc);
    }
}

pub(crate) unsafe extern "C" fn cleanup_function_call(mut fc: *mut funccall_T) {
    unsafe {
        let mut may_free_fc: bool = (*fc).fc_refcount <= 0 as ::core::ffi::c_int;
        let mut free_fc: bool = true_0 != 0;
        current_funccal.set((*fc).fc_caller);
        if may_free_fc as ::core::ffi::c_int != 0
            && (*fc).fc_l_vars.dv_refcount == DO_NOT_FREE_CNT as ::core::ffi::c_int
        {
            vars_clear(&raw mut (*fc).fc_l_vars.dv_hashtab);
        } else {
            free_fc = false_0 != 0;
        }
        if may_free_fc as ::core::ffi::c_int != 0
            && (*fc).fc_l_avars.dv_refcount == DO_NOT_FREE_CNT as ::core::ffi::c_int
        {
            vars_clear_ext(&raw mut (*fc).fc_l_avars.dv_hashtab, false_0 != 0);
        } else {
            free_fc = false_0 != 0;
            let dihi_ht_: *mut hashtab_T = &raw mut (*fc).fc_l_avars.dv_hashtab;
            let mut dihi_todo_: size_t = (*dihi_ht_).ht_used;
            let mut dihi_: *mut hashitem_T = (*dihi_ht_).ht_array;
            while dihi_todo_ != 0 {
                if !((*dihi_).hi_key.is_null()
                    || (*dihi_).hi_key == &raw const hash_removed as *mut ::core::ffi::c_char)
                {
                    dihi_todo_ = dihi_todo_.wrapping_sub(1);
                    let di: *mut dictitem_T = (*dihi_)
                        .hi_key
                        .offset(-(17 as ::core::ffi::c_ulong as isize))
                        as *mut dictitem_T;
                    tv_copy(&raw mut (*di).di_tv, &raw mut (*di).di_tv);
                }
                dihi_ = dihi_.offset(1);
            }
        }
        if may_free_fc as ::core::ffi::c_int != 0
            && (*fc).fc_l_varlist.lv_refcount == DO_NOT_FREE_CNT as ::core::ffi::c_int
        {
            (*fc).fc_l_varlist.lv_first = ::core::ptr::null_mut::<listitem_T>();
        } else {
            free_fc = false_0 != 0;
            let l_: *mut list_T = &raw mut (*fc).fc_l_varlist;
            if !l_.is_null() {
                let mut li: *mut listitem_T = (*l_).lv_first;
                while !li.is_null() {
                    tv_copy(&raw mut (*li).li_tv, &raw mut (*li).li_tv);
                    li = (*li).li_next;
                }
            }
        }
        if free_fc {
            free_funccal(fc);
        } else {
            static made_copy: GlobalCell<::core::ffi::c_int> =
                GlobalCell::new(0 as ::core::ffi::c_int);
            (*fc).fc_caller = previous_funccal.get();
            previous_funccal.set(fc);
            if want_garbage_collect.get() {
                made_copy.set(0 as ::core::ffi::c_int);
            } else {
                (*made_copy.ptr()) += 1;
                if made_copy.get()
                    >= ((4096 as ::core::ffi::c_int * 1024 as ::core::ffi::c_int) as usize)
                        .wrapping_div(::core::mem::size_of::<funccall_T>())
                        as ::core::ffi::c_int
                {
                    made_copy.set(0 as ::core::ffi::c_int);
                    want_garbage_collect.set(true_0 != 0);
                }
            }
        };
    }
}

pub(crate) unsafe extern "C" fn funccal_unref(
    mut fc: *mut funccall_T,
    mut fp: *mut ufunc_T,
    mut force: bool,
) {
    unsafe {
        if fc.is_null() {
            return;
        }
        (*fc).fc_refcount -= 1;
        if if force as ::core::ffi::c_int != 0 {
            ((*fc).fc_refcount <= 0 as ::core::ffi::c_int) as ::core::ffi::c_int
        } else {
            !fc_referenced(fc) as ::core::ffi::c_int
        } != 0
        {
            let mut pfc: *mut *mut funccall_T = previous_funccal.ptr();
            while !(*pfc).is_null() {
                if fc == *pfc {
                    *pfc = (*fc).fc_caller;
                    free_funccal_contents(fc);
                    return;
                }
                pfc = &raw mut (**pfc).fc_caller;
            }
        }
        let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while i < (*fc).fc_ufuncs.ga_len {
            if *((*fc).fc_ufuncs.ga_data as *mut *mut ufunc_T).offset(i as isize) == fp {
                *((*fc).fc_ufuncs.ga_data as *mut *mut ufunc_T).offset(i as isize) =
                    ::core::ptr::null_mut::<ufunc_T>();
            }
            i += 1;
        }
    }
}

pub(crate) unsafe extern "C" fn func_remove(mut fp: *mut ufunc_T) -> bool {
    unsafe {
        let mut hi: *mut hashitem_T = hash_find(
            func_hashtab.ptr(),
            &raw mut (*fp).uf_name as *mut ::core::ffi::c_char,
        );
        if (*hi).hi_key.is_null()
            || (*hi).hi_key == &raw const hash_removed as *mut ::core::ffi::c_char
        {
            return false_0 != 0;
        }
        hash_remove(func_hashtab.ptr(), hi);
        return true_0 != 0;
    }
}

pub(crate) unsafe extern "C" fn func_clear_items(mut fp: *mut ufunc_T) {
    unsafe {
        ga_clear_strings(&raw mut (*fp).uf_args);
        ga_clear_strings(&raw mut (*fp).uf_def_args);
        ga_clear_strings(&raw mut (*fp).uf_lines);
        if (*fp).uf_flags & FC_LUAREF != 0 {
            api_free_luaref((*fp).uf_luaref);
            (*fp).uf_luaref = LUA_NOREF as LuaRef;
        }
        let mut ptr_: *mut *mut ::core::ffi::c_void =
            &raw mut (*fp).uf_tml_count as *mut *mut ::core::ffi::c_void;
        xfree(*ptr_);
        *ptr_ = NULL;
        let _ = *ptr_;
        let mut ptr__0: *mut *mut ::core::ffi::c_void =
            &raw mut (*fp).uf_tml_total as *mut *mut ::core::ffi::c_void;
        xfree(*ptr__0);
        *ptr__0 = NULL;
        let _ = *ptr__0;
        let mut ptr__1: *mut *mut ::core::ffi::c_void =
            &raw mut (*fp).uf_tml_self as *mut *mut ::core::ffi::c_void;
        xfree(*ptr__1);
        *ptr__1 = NULL;
        let _ = *ptr__1;
    }
}

unsafe extern "C" fn func_clear(mut fp: *mut ufunc_T, mut force: bool) {
    unsafe {
        if (*fp).uf_cleared {
            return;
        }
        (*fp).uf_cleared = true_0 != 0;
        func_clear_items(fp);
        funccal_unref((*fp).uf_scoped, fp, force);
    }
}

unsafe extern "C" fn func_free(mut fp: *mut ufunc_T) {
    unsafe {
        if (*fp).uf_flags & (FC_DELETED | FC_REMOVED) == 0 as ::core::ffi::c_int {
            func_remove(fp);
        }
        let mut ptr_: *mut *mut ::core::ffi::c_void =
            &raw mut (*fp).uf_name_exp as *mut *mut ::core::ffi::c_void;
        xfree(*ptr_);
        *ptr_ = NULL;
        let _ = *ptr_;
        xfree(fp as *mut ::core::ffi::c_void);
    }
}

pub(crate) unsafe extern "C" fn func_clear_free(mut fp: *mut ufunc_T, mut force: bool) {
    unsafe {
        func_clear(fp, force);
        func_free(fp);
    }
}

pub unsafe extern "C" fn create_funccal(
    mut fp: *mut ufunc_T,
    mut rettv: *mut typval_T,
) -> *mut funccall_T {
    unsafe {
        let mut fc: *mut funccall_T =
            xcalloc(1 as size_t, ::core::mem::size_of::<funccall_T>()) as *mut funccall_T;
        (*fc).fc_caller = current_funccal.get();
        current_funccal.set(fc);
        (*fc).fc_func = fp;
        func_ptr_ref(fp);
        (*fc).fc_rettv = rettv;
        return fc;
    }
}

pub(crate) static funccal_stack: GlobalCell<*mut funccal_entry_T> =
    GlobalCell::new(::core::ptr::null_mut::<funccal_entry_T>());

pub unsafe extern "C" fn save_funccal(mut entry: *mut funccal_entry_T) {
    unsafe {
        (*entry).top_funccal = current_funccal.get() as *mut ::core::ffi::c_void;
        (*entry).next = funccal_stack.get();
        funccal_stack.set(entry);
        current_funccal.set(::core::ptr::null_mut::<funccall_T>());
    }
}

pub unsafe extern "C" fn restore_funccal() {
    unsafe {
        if (*funccal_stack.ptr()).is_null() {
            iemsg(b"INTERNAL: restore_funccal()\0".as_ptr() as *const ::core::ffi::c_char);
        } else {
            current_funccal.set((*funccal_stack.get()).top_funccal as *mut funccall_T);
            funccal_stack.set((*funccal_stack.get()).next);
        };
    }
}

pub unsafe extern "C" fn get_current_funccal() -> *mut funccall_T {
    return current_funccal.get();
}

pub unsafe extern "C" fn set_current_funccal(mut fc: *mut funccall_T) {
    current_funccal.set(fc);
}

pub unsafe extern "C" fn func_unref(mut name: *mut ::core::ffi::c_char) {
    unsafe {
        if name.is_null() || !func_name_refcount(name) {
            return;
        }
        let mut fp: *mut ufunc_T = find_func(name);
        if fp.is_null()
            && *(*__ctype_b_loc()).offset(*name as uint8_t as ::core::ffi::c_int as isize)
                as ::core::ffi::c_int
                & _ISdigit as ::core::ffi::c_int as ::core::ffi::c_ushort as ::core::ffi::c_int
                != 0
        {
            internal_error(b"func_unref()\0".as_ptr() as *const ::core::ffi::c_char);
            abort();
        }
        func_ptr_unref(fp);
    }
}

pub unsafe extern "C" fn func_ptr_unref(mut fp: *mut ufunc_T) {
    unsafe {
        if !fp.is_null() && {
            (*fp).uf_refcount -= 1;
            (*fp).uf_refcount <= 0 as ::core::ffi::c_int
        } {
            if (*fp).uf_calls == 0 as ::core::ffi::c_int {
                func_clear_free(fp, false_0 != 0);
            }
        }
    }
}

pub unsafe extern "C" fn func_ref(mut name: *mut ::core::ffi::c_char) {
    unsafe {
        if name.is_null() || !func_name_refcount(name) {
            return;
        }
        let mut fp: *mut ufunc_T = find_func(name);
        if !fp.is_null() {
            (*fp).uf_refcount += 1;
        } else if *(*__ctype_b_loc()).offset(*name as uint8_t as ::core::ffi::c_int as isize)
            as ::core::ffi::c_int
            & _ISdigit as ::core::ffi::c_int as ::core::ffi::c_ushort as ::core::ffi::c_int
            != 0
        {
            internal_error(b"func_ref()\0".as_ptr() as *const ::core::ffi::c_char);
        }
    }
}

pub unsafe extern "C" fn func_ptr_ref(mut fp: *mut ufunc_T) {
    unsafe {
        if !fp.is_null() {
            (*fp).uf_refcount += 1;
        }
    }
}

#[inline(always)]
unsafe extern "C" fn fc_referenced(fc: *const funccall_T) -> bool {
    unsafe {
        return (*fc).fc_l_varlist.lv_refcount != DO_NOT_FREE_CNT as ::core::ffi::c_int
            || (*fc).fc_l_vars.dv_refcount != DO_NOT_FREE_CNT as ::core::ffi::c_int
            || (*fc).fc_l_avars.dv_refcount != DO_NOT_FREE_CNT as ::core::ffi::c_int
            || (*fc).fc_refcount > 0 as ::core::ffi::c_int;
    }
}

unsafe extern "C" fn can_free_funccal(
    mut fc: *mut funccall_T,
    mut copyID: ::core::ffi::c_int,
) -> bool {
    unsafe {
        return (*fc).fc_l_varlist.lv_copyID != copyID
            && (*fc).fc_l_vars.dv_copyID != copyID
            && (*fc).fc_l_avars.dv_copyID != copyID
            && (*fc).fc_copyID != copyID;
    }
}

pub unsafe extern "C" fn free_unref_funccal(
    mut copyID: ::core::ffi::c_int,
    mut testing: ::core::ffi::c_int,
) -> bool {
    unsafe {
        let mut did_free: bool = false_0 != 0;
        let mut did_free_funccal: bool = false_0 != 0;
        let mut pfc: *mut *mut funccall_T = previous_funccal.ptr();
        while !(*pfc).is_null() {
            if can_free_funccal(*pfc, copyID) {
                let mut fc: *mut funccall_T = *pfc;
                *pfc = (*fc).fc_caller;
                free_funccal_contents(fc);
                did_free = true_0 != 0;
                did_free_funccal = true_0 != 0;
            } else {
                pfc = &raw mut (**pfc).fc_caller;
            }
        }
        if did_free_funccal {
            garbage_collect(testing != 0);
        }
        return did_free;
    }
}

pub unsafe extern "C" fn get_funccal() -> *mut funccall_T {
    unsafe {
        let mut funccal: *mut funccall_T = current_funccal.get();
        if debug_backtrace_level.get() > 0 as ::core::ffi::c_int {
            let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
            while i < debug_backtrace_level.get() {
                let mut temp_funccal: *mut funccall_T = (*funccal).fc_caller;
                if !temp_funccal.is_null() {
                    funccal = temp_funccal;
                } else {
                    debug_backtrace_level.set(i);
                }
                i += 1;
            }
        }
        return funccal;
    }
}

pub unsafe extern "C" fn get_funccal_local_dict() -> *mut dict_T {
    unsafe {
        if (*current_funccal.ptr()).is_null()
            || (*current_funccal.get()).fc_l_vars.dv_refcount == 0 as ::core::ffi::c_int
        {
            return ::core::ptr::null_mut::<dict_T>();
        }
        return &raw mut (*(get_funccal as unsafe extern "C" fn() -> *mut funccall_T)()).fc_l_vars;
    }
}

pub unsafe extern "C" fn get_funccal_local_ht() -> *mut hashtab_T {
    unsafe {
        let mut d: *mut dict_T = get_funccal_local_dict();
        return if !d.is_null() {
            &raw mut (*d).dv_hashtab
        } else {
            ::core::ptr::null_mut::<hashtab_T>()
        };
    }
}

pub unsafe extern "C" fn get_funccal_local_var() -> *mut dictitem_T {
    unsafe {
        if (*current_funccal.ptr()).is_null()
            || (*current_funccal.get()).fc_l_vars.dv_refcount == 0 as ::core::ffi::c_int
        {
            return ::core::ptr::null_mut::<dictitem_T>();
        }
        return &raw mut (*(get_funccal as unsafe extern "C" fn() -> *mut funccall_T)())
            .fc_l_vars_var as *mut dictitem_T;
    }
}

pub unsafe extern "C" fn get_funccal_args_dict() -> *mut dict_T {
    unsafe {
        if (*current_funccal.ptr()).is_null()
            || (*current_funccal.get()).fc_l_vars.dv_refcount == 0 as ::core::ffi::c_int
        {
            return ::core::ptr::null_mut::<dict_T>();
        }
        return &raw mut (*(get_funccal as unsafe extern "C" fn() -> *mut funccall_T)()).fc_l_avars;
    }
}

pub unsafe extern "C" fn get_funccal_args_ht() -> *mut hashtab_T {
    unsafe {
        let mut d: *mut dict_T = get_funccal_args_dict();
        return if !d.is_null() {
            &raw mut (*d).dv_hashtab
        } else {
            ::core::ptr::null_mut::<hashtab_T>()
        };
    }
}

pub unsafe extern "C" fn get_funccal_args_var() -> *mut dictitem_T {
    unsafe {
        if (*current_funccal.ptr()).is_null()
            || (*current_funccal.get()).fc_l_vars.dv_refcount == 0 as ::core::ffi::c_int
        {
            return ::core::ptr::null_mut::<dictitem_T>();
        }
        return &raw mut (*(get_funccal as unsafe extern "C" fn() -> *mut funccall_T)())
            .fc_l_avars_var as *mut dictitem_T;
    }
}

pub unsafe extern "C" fn list_func_vars(mut first: *mut ::core::ffi::c_int) {
    unsafe {
        if !(*current_funccal.ptr()).is_null()
            && (*current_funccal.get()).fc_l_vars.dv_refcount > 0 as ::core::ffi::c_int
        {
            list_hashtable_vars(
                &raw mut (*current_funccal.get()).fc_l_vars.dv_hashtab,
                b"l:\0".as_ptr() as *const ::core::ffi::c_char,
                false,
                first,
            );
        }
    }
}

pub unsafe extern "C" fn get_current_funccal_dict(mut ht: *mut hashtab_T) -> *mut dict_T {
    unsafe {
        if !(*current_funccal.ptr()).is_null()
            && ht == &raw mut (*current_funccal.get()).fc_l_vars.dv_hashtab
        {
            return &raw mut (*current_funccal.get()).fc_l_vars;
        }
        return ::core::ptr::null_mut::<dict_T>();
    }
}

pub unsafe extern "C" fn find_hi_in_scoped_ht(
    mut name: *const ::core::ffi::c_char,
    mut pht: *mut *mut hashtab_T,
) -> *mut hashitem_T {
    unsafe {
        if (*current_funccal.ptr()).is_null()
            || (*(*current_funccal.get()).fc_func).uf_scoped.is_null()
        {
            return ::core::ptr::null_mut::<hashitem_T>();
        }
        let mut old_current_funccal: *mut funccall_T = current_funccal.get();
        let mut hi: *mut hashitem_T = ::core::ptr::null_mut::<hashitem_T>();
        let namelen: size_t = strlen(name);
        let mut varname: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
        current_funccal.set((*(*current_funccal.get()).fc_func).uf_scoped);
        while !(*current_funccal.ptr()).is_null() {
            let mut ht: *mut hashtab_T = find_var_ht(name, namelen, &raw mut varname);
            if !ht.is_null() && *varname as ::core::ffi::c_int != NUL {
                hi = hash_find_len(
                    ht,
                    varname,
                    namelen.wrapping_sub(varname.offset_from(name) as size_t),
                );
                if !((*hi).hi_key.is_null()
                    || (*hi).hi_key == &raw const hash_removed as *mut ::core::ffi::c_char)
                {
                    *pht = ht;
                    break;
                }
            }
            if current_funccal.get() == (*(*current_funccal.get()).fc_func).uf_scoped {
                break;
            }
            current_funccal.set((*(*current_funccal.get()).fc_func).uf_scoped);
        }
        current_funccal.set(old_current_funccal);
        return hi;
    }
}

pub unsafe extern "C" fn find_var_in_scoped_ht(
    mut name: *const ::core::ffi::c_char,
    namelen: size_t,
    mut no_autoload: ::core::ffi::c_int,
) -> *mut dictitem_T {
    unsafe {
        if (*current_funccal.ptr()).is_null()
            || (*(*current_funccal.get()).fc_func).uf_scoped.is_null()
        {
            return ::core::ptr::null_mut::<dictitem_T>();
        }
        let mut v: *mut dictitem_T = ::core::ptr::null_mut::<dictitem_T>();
        let mut old_current_funccal: *mut funccall_T = current_funccal.get();
        let mut varname: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
        current_funccal.set((*(*current_funccal.get()).fc_func).uf_scoped);
        while !(*current_funccal.ptr()).is_null() {
            let mut ht: *mut hashtab_T = find_var_ht(name, namelen, &raw mut varname);
            if !ht.is_null() && *varname as ::core::ffi::c_int != NUL {
                v = find_var_in_ht(
                    ht,
                    *name as ::core::ffi::c_int,
                    varname,
                    namelen.wrapping_sub(varname.offset_from(name) as size_t),
                    no_autoload != 0,
                );
                if !v.is_null() {
                    break;
                }
            }
            if current_funccal.get() == (*(*current_funccal.get()).fc_func).uf_scoped {
                break;
            }
            current_funccal.set((*(*current_funccal.get()).fc_func).uf_scoped);
        }
        current_funccal.set(old_current_funccal);
        return v;
    }
}

pub unsafe extern "C" fn set_ref_in_previous_funccal(mut copyID: ::core::ffi::c_int) -> bool {
    unsafe {
        let mut fc: *mut funccall_T = previous_funccal.get();
        while !fc.is_null() {
            (*fc).fc_copyID = copyID + 1 as ::core::ffi::c_int;
            if set_ref_in_ht(
                &raw mut (*fc).fc_l_vars.dv_hashtab,
                copyID + 1 as ::core::ffi::c_int,
                ::core::ptr::null_mut::<*mut list_stack_T>(),
            ) as ::core::ffi::c_int
                != 0
                || set_ref_in_ht(
                    &raw mut (*fc).fc_l_avars.dv_hashtab,
                    copyID + 1 as ::core::ffi::c_int,
                    ::core::ptr::null_mut::<*mut list_stack_T>(),
                ) as ::core::ffi::c_int
                    != 0
                || set_ref_in_list_items(
                    &raw mut (*fc).fc_l_varlist,
                    copyID + 1 as ::core::ffi::c_int,
                    ::core::ptr::null_mut::<*mut ht_stack_T>(),
                ) as ::core::ffi::c_int
                    != 0
            {
                return true_0 != 0;
            }
            fc = (*fc).fc_caller;
        }
        return false_0 != 0;
    }
}

unsafe extern "C" fn set_ref_in_funccal(
    mut fc: *mut funccall_T,
    mut copyID: ::core::ffi::c_int,
) -> bool {
    unsafe {
        if (*fc).fc_copyID != copyID {
            (*fc).fc_copyID = copyID;
            if set_ref_in_ht(
                &raw mut (*fc).fc_l_vars.dv_hashtab,
                copyID,
                ::core::ptr::null_mut::<*mut list_stack_T>(),
            ) as ::core::ffi::c_int
                != 0
                || set_ref_in_ht(
                    &raw mut (*fc).fc_l_avars.dv_hashtab,
                    copyID,
                    ::core::ptr::null_mut::<*mut list_stack_T>(),
                ) as ::core::ffi::c_int
                    != 0
                || set_ref_in_list_items(
                    &raw mut (*fc).fc_l_varlist,
                    copyID,
                    ::core::ptr::null_mut::<*mut ht_stack_T>(),
                ) as ::core::ffi::c_int
                    != 0
                || set_ref_in_func(
                    ::core::ptr::null_mut::<::core::ffi::c_char>(),
                    (*fc).fc_func,
                    copyID,
                ) as ::core::ffi::c_int
                    != 0
            {
                return true_0 != 0;
            }
        }
        return false_0 != 0;
    }
}

pub unsafe extern "C" fn set_ref_in_call_stack(mut copyID: ::core::ffi::c_int) -> bool {
    unsafe {
        let mut fc: *mut funccall_T = current_funccal.get();
        while !fc.is_null() {
            if set_ref_in_funccal(fc, copyID) {
                return true_0 != 0;
            }
            fc = (*fc).fc_caller;
        }
        let mut entry: *mut funccal_entry_T = funccal_stack.get();
        while !entry.is_null() {
            let mut fc_0: *mut funccall_T = (*entry).top_funccal as *mut funccall_T;
            while !fc_0.is_null() {
                if set_ref_in_funccal(fc_0, copyID) {
                    return true_0 != 0;
                }
                fc_0 = (*fc_0).fc_caller;
            }
            entry = (*entry).next;
        }
        return false_0 != 0;
    }
}

pub unsafe extern "C" fn set_ref_in_functions(mut copyID: ::core::ffi::c_int) -> bool {
    unsafe {
        let mut todo: ::core::ffi::c_int = (*func_hashtab.ptr()).ht_used as ::core::ffi::c_int;
        let mut hi: *mut hashitem_T = (*func_hashtab.ptr()).ht_array;
        while todo > 0 as ::core::ffi::c_int && !got_int.get() {
            if !((*hi).hi_key.is_null()
                || (*hi).hi_key == &raw const hash_removed as *mut ::core::ffi::c_char)
            {
                todo -= 1;
                let mut fp: *mut ufunc_T =
                    (*hi).hi_key.offset(-(240 as ::core::ffi::c_ulong as isize)) as *mut ufunc_T;
                if !func_name_refcount(&raw mut (*fp).uf_name as *mut ::core::ffi::c_char)
                    && set_ref_in_func(::core::ptr::null_mut::<::core::ffi::c_char>(), fp, copyID)
                        as ::core::ffi::c_int
                        != 0
                {
                    return true_0 != 0;
                }
            }
            hi = hi.offset(1);
        }
        return false_0 != 0;
    }
}

pub unsafe extern "C" fn set_ref_in_func_args(mut copyID: ::core::ffi::c_int) -> bool {
    unsafe {
        let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while i < (*funcargs.ptr()).ga_len {
            if set_ref_in_item(
                *((*funcargs.ptr()).ga_data as *mut *mut typval_T).offset(i as isize),
                copyID,
                ::core::ptr::null_mut::<*mut ht_stack_T>(),
                ::core::ptr::null_mut::<*mut list_stack_T>(),
            ) {
                return true_0 != 0;
            }
            i += 1;
        }
        return false_0 != 0;
    }
}

pub unsafe extern "C" fn set_ref_in_func(
    mut name: *mut ::core::ffi::c_char,
    mut fp_in: *mut ufunc_T,
    mut copyID: ::core::ffi::c_int,
) -> bool {
    unsafe {
        let mut fp: *mut ufunc_T = fp_in;
        let mut error: ::core::ffi::c_int = FCERR_NONE as ::core::ffi::c_int;
        let mut fname_buf: [::core::ffi::c_char; 41] = [0; 41];
        let mut tofree: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
        let mut abort_0: bool = false_0 != 0;
        if name.is_null() && fp_in.is_null() {
            return false_0 != 0;
        }
        if fp_in.is_null() {
            let mut fname: *mut ::core::ffi::c_char = fname_trans_sid(
                name,
                &raw mut fname_buf as *mut ::core::ffi::c_char,
                &raw mut tofree,
                &raw mut error,
            );
            fp = find_func(fname);
        }
        if !fp.is_null() {
            let mut fc: *mut funccall_T = (*fp).uf_scoped;
            while !fc.is_null() {
                abort_0 = abort_0 as ::core::ffi::c_int != 0
                    || set_ref_in_funccal(fc, copyID) as ::core::ffi::c_int != 0;
                fc = (*(*fc).fc_func).uf_scoped;
            }
        }
        xfree(tofree as *mut ::core::ffi::c_void);
        return abort_0;
    }
}
