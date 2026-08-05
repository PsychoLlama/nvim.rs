//! `'completefunc'`, `'omnifunc'`, `'thesaurusfunc'` and the `'complete'` `F` flag.
//!
//! The `did_set_*` halves are the option callbacks that compile a funcname
//! into a `Callback`; [`expand_by_function`] is the call itself, which runs
//! the function twice (`findstart` then the matches) exactly as upstream
//! does.  The `cpt_sources_*` half tracks the per-`'complete'`-entry state
//! those functions need.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;

pub(crate) unsafe extern "C" fn get_cpt_sources_count() -> ::core::ffi::c_int {
    unsafe {
        let mut dummy: [::core::ffi::c_char; 512] = [0; 512];
        let mut count: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        let mut p: *mut ::core::ffi::c_char = (*curbuf.get()).b_p_cpt;
        while *p as ::core::ffi::c_int != NUL {
            while *p as ::core::ffi::c_int == ',' as ::core::ffi::c_int
                || *p as ::core::ffi::c_int == ' ' as ::core::ffi::c_int
            {
                p = p.offset(1);
            }
            if *p as ::core::ffi::c_int != NUL {
                copy_option_part(
                    &raw mut p,
                    &raw mut dummy as *mut ::core::ffi::c_char,
                    LSIZE as size_t,
                    b",\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
                );
                count += 1;
            }
        }
        return count;
    }
}

pub(crate) unsafe extern "C" fn copy_global_to_buflocal_cb(
    mut globcb: *mut Callback,
    mut bufcb: *mut Callback,
) {
    unsafe {
        callback_free(bufcb);
        if (*globcb).type_0 as ::core::ffi::c_uint
            != kCallbackNone as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            callback_copy(bufcb, globcb);
        }
    }
}

pub unsafe extern "C" fn did_set_completefunc(
    mut args: *mut optset_T,
) -> *const ::core::ffi::c_char {
    unsafe {
        let mut buf: *mut buf_T = (*args).os_buf as *mut buf_T;
        let mut retval: ::core::ffi::c_int = 0;
        if (*args).os_flags & OPT_LOCAL != 0 {
            retval =
                option_set_callback_func((*args).os_newval.string.data, &raw mut (*buf).b_cfu_cb);
        } else {
            retval = option_set_callback_func((*args).os_newval.string.data, cfu_cb.ptr());
            if retval == OK && (*args).os_flags & OPT_GLOBAL == 0 {
                set_buflocal_cfu_callback(buf);
            }
        }
        return if retval == FAIL {
            &raw const e_invarg as *const ::core::ffi::c_char
        } else {
            ::core::ptr::null::<::core::ffi::c_char>()
        };
    }
}

pub unsafe extern "C" fn set_buflocal_cfu_callback(mut buf: *mut buf_T) {
    unsafe {
        copy_global_to_buflocal_cb(cfu_cb.ptr(), &raw mut (*buf).b_cfu_cb);
    }
}

pub unsafe extern "C" fn did_set_omnifunc(mut args: *mut optset_T) -> *const ::core::ffi::c_char {
    unsafe {
        let mut buf: *mut buf_T = (*args).os_buf as *mut buf_T;
        let mut retval: ::core::ffi::c_int = 0;
        if (*args).os_flags & OPT_LOCAL != 0 {
            retval =
                option_set_callback_func((*args).os_newval.string.data, &raw mut (*buf).b_ofu_cb);
        } else {
            retval = option_set_callback_func((*args).os_newval.string.data, ofu_cb.ptr());
            if retval == OK && (*args).os_flags & OPT_GLOBAL == 0 {
                set_buflocal_ofu_callback(buf);
            }
        }
        return if retval == FAIL {
            &raw const e_invarg as *const ::core::ffi::c_char
        } else {
            ::core::ptr::null::<::core::ffi::c_char>()
        };
    }
}

pub unsafe extern "C" fn set_buflocal_ofu_callback(mut buf: *mut buf_T) {
    unsafe {
        copy_global_to_buflocal_cb(ofu_cb.ptr(), &raw mut (*buf).b_ofu_cb);
    }
}

pub unsafe extern "C" fn clear_cpt_callbacks(
    mut callbacks: *mut *mut Callback,
    mut count: ::core::ffi::c_int,
) {
    unsafe {
        if callbacks.is_null() || (*callbacks).is_null() {
            return;
        }
        let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while i < count {
            callback_free((*callbacks).offset(i as isize));
            i += 1;
        }
        let mut ptr_: *mut *mut ::core::ffi::c_void = callbacks as *mut *mut ::core::ffi::c_void;
        xfree(*ptr_);
        *ptr_ = NULL;
        let _ = *ptr_;
    }
}

pub(crate) unsafe extern "C" fn copy_cpt_callbacks(
    mut dest: *mut *mut Callback,
    mut dest_cnt: *mut ::core::ffi::c_int,
    mut src: *mut Callback,
    mut cnt: ::core::ffi::c_int,
) {
    unsafe {
        if cnt == 0 as ::core::ffi::c_int {
            return;
        }
        clear_cpt_callbacks(dest, *dest_cnt);
        *dest = xcalloc(cnt as size_t, ::core::mem::size_of::<Callback>()) as *mut Callback;
        *dest_cnt = cnt;
        let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while i < cnt {
            if (*src.offset(i as isize)).type_0 as ::core::ffi::c_uint
                != kCallbackNone as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                callback_copy((*dest).offset(i as isize), src.offset(i as isize));
            }
            i += 1;
        }
    }
}

pub unsafe extern "C" fn set_buflocal_cpt_callbacks(mut buf: *mut buf_T) {
    unsafe {
        if buf.is_null() || cpt_cb_count.get() == 0 as ::core::ffi::c_int {
            return;
        }
        copy_cpt_callbacks(
            &raw mut (*buf).b_p_cpt_cb,
            &raw mut (*buf).b_p_cpt_count,
            cpt_cb.get(),
            cpt_cb_count.get(),
        );
    }
}

pub unsafe extern "C" fn set_cpt_callbacks(mut args: *mut optset_T) -> ::core::ffi::c_int {
    unsafe {
        let mut local: bool = (*args).os_flags & OPT_LOCAL != 0 as ::core::ffi::c_int;
        if (*curbuf.ptr()).is_null() {
            return FAIL;
        }
        clear_cpt_callbacks(
            &raw mut (*curbuf.get()).b_p_cpt_cb,
            (*curbuf.get()).b_p_cpt_count,
        );
        (*curbuf.get()).b_p_cpt_count = 0 as ::core::ffi::c_int;
        let mut count: ::core::ffi::c_int = get_cpt_sources_count();
        if count == 0 as ::core::ffi::c_int {
            return OK;
        }
        (*curbuf.get()).b_p_cpt_cb =
            xcalloc(count as size_t, ::core::mem::size_of::<Callback>()) as *mut Callback;
        (*curbuf.get()).b_p_cpt_count = count;
        let mut buf: [::core::ffi::c_char; 512] = [0; 512];
        let mut idx: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        let mut p: *mut ::core::ffi::c_char = (*curbuf.get()).b_p_cpt;
        while *p as ::core::ffi::c_int != NUL {
            while *p as ::core::ffi::c_int == ',' as ::core::ffi::c_int
                || *p as ::core::ffi::c_int == ' ' as ::core::ffi::c_int
            {
                p = p.offset(1);
            }
            if *p as ::core::ffi::c_int != NUL {
                let mut slen: size_t = copy_option_part(
                    &raw mut p,
                    &raw mut buf as *mut ::core::ffi::c_char,
                    LSIZE as size_t,
                    b",\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
                );
                if slen > 0 as size_t
                    && buf[0 as ::core::ffi::c_int as usize] as ::core::ffi::c_int
                        == 'F' as ::core::ffi::c_int
                    && buf[1 as ::core::ffi::c_int as usize] as ::core::ffi::c_int != NUL
                {
                    let mut caret: *mut ::core::ffi::c_char = vim_strchr(
                        &raw mut buf as *mut ::core::ffi::c_char,
                        '^' as ::core::ffi::c_int,
                    );
                    if !caret.is_null() {
                        *caret = NUL as ::core::ffi::c_char;
                    }
                    if option_set_callback_func(
                        (&raw mut buf as *mut ::core::ffi::c_char)
                            .offset(1 as ::core::ffi::c_int as isize),
                        (*curbuf.get()).b_p_cpt_cb.offset(idx as isize),
                    ) != OK
                    {
                        (*(*curbuf.get()).b_p_cpt_cb.offset(idx as isize)).type_0 = kCallbackNone;
                    }
                }
                idx += 1;
            }
        }
        if !local {
            copy_cpt_callbacks(
                cpt_cb.ptr(),
                cpt_cb_count.ptr(),
                (*curbuf.get()).b_p_cpt_cb,
                (*curbuf.get()).b_p_cpt_count,
            );
        }
        return OK;
    }
}

pub unsafe extern "C" fn did_set_thesaurusfunc(
    mut args: *mut optset_T,
) -> *const ::core::ffi::c_char {
    unsafe {
        let mut buf: *mut buf_T = (*args).os_buf as *mut buf_T;
        let mut retval: ::core::ffi::c_int = 0;
        if (*args).os_flags & OPT_LOCAL != 0 {
            retval = option_set_callback_func((*buf).b_p_tsrfu, &raw mut (*buf).b_tsrfu_cb);
        } else {
            retval = option_set_callback_func(p_tsrfu.get(), tsrfu_cb.ptr());
            if (*args).os_flags & OPT_GLOBAL == 0 {
                callback_free(&raw mut (*buf).b_tsrfu_cb);
            }
        }
        return if retval == FAIL {
            &raw const e_invarg as *const ::core::ffi::c_char
        } else {
            ::core::ptr::null::<::core::ffi::c_char>()
        };
    }
}

pub unsafe extern "C" fn set_ref_in_cpt_callbacks(
    mut callbacks: *mut Callback,
    mut count: ::core::ffi::c_int,
    mut copyID: ::core::ffi::c_int,
) -> bool {
    unsafe {
        let mut abort: bool = false_0 != 0;
        if callbacks.is_null() {
            return false_0 != 0;
        }
        let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while i < count {
            abort = abort as ::core::ffi::c_int != 0
                || set_ref_in_callback(
                    callbacks.offset(i as isize),
                    copyID,
                    ::core::ptr::null_mut::<*mut ht_stack_T>(),
                    ::core::ptr::null_mut::<*mut list_stack_T>(),
                ) as ::core::ffi::c_int
                    != 0;
            i += 1;
        }
        return abort;
    }
}

pub unsafe extern "C" fn set_ref_in_insexpand_funcs(mut copyID: ::core::ffi::c_int) -> bool {
    unsafe {
        let mut abort: bool = set_ref_in_callback(
            cfu_cb.ptr(),
            copyID,
            ::core::ptr::null_mut::<*mut ht_stack_T>(),
            ::core::ptr::null_mut::<*mut list_stack_T>(),
        );
        abort = abort as ::core::ffi::c_int != 0
            || set_ref_in_callback(
                ofu_cb.ptr(),
                copyID,
                ::core::ptr::null_mut::<*mut ht_stack_T>(),
                ::core::ptr::null_mut::<*mut list_stack_T>(),
            ) as ::core::ffi::c_int
                != 0;
        abort = abort as ::core::ffi::c_int != 0
            || set_ref_in_callback(
                tsrfu_cb.ptr(),
                copyID,
                ::core::ptr::null_mut::<*mut ht_stack_T>(),
                ::core::ptr::null_mut::<*mut list_stack_T>(),
            ) as ::core::ffi::c_int
                != 0;
        abort = abort as ::core::ffi::c_int != 0
            || set_ref_in_cpt_callbacks(cpt_cb.get(), cpt_cb_count.get(), copyID)
                as ::core::ffi::c_int
                != 0;
        return abort;
    }
}

pub(crate) unsafe extern "C" fn get_complete_funcname(
    mut type_0: ::core::ffi::c_int,
) -> *mut ::core::ffi::c_char {
    unsafe {
        match type_0 {
            12 => return (*curbuf.get()).b_p_cfu,
            13 => return (*curbuf.get()).b_p_ofu,
            266 => {
                return if *(*curbuf.get()).b_p_tsrfu as ::core::ffi::c_int == NUL {
                    p_tsrfu.get()
                } else {
                    (*curbuf.get()).b_p_tsrfu
                };
            }
            _ => {
                return b"\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
            }
        };
    }
}

pub(crate) unsafe extern "C" fn get_insert_callback(
    mut type_0: ::core::ffi::c_int,
) -> *mut Callback {
    unsafe {
        if type_0 == CTRL_X_FUNCTION {
            return &raw mut (*curbuf.get()).b_cfu_cb;
        }
        if type_0 == CTRL_X_OMNI {
            return &raw mut (*curbuf.get()).b_ofu_cb;
        }
        return if *(*curbuf.get()).b_p_tsrfu as ::core::ffi::c_int != NUL {
            &raw mut (*curbuf.get()).b_tsrfu_cb
        } else {
            tsrfu_cb.ptr()
        };
    }
}

pub(crate) unsafe extern "C" fn expand_by_function(
    mut type_0: ::core::ffi::c_int,
    mut base: *mut ::core::ffi::c_char,
    mut cb: *mut Callback,
) {
    unsafe {
        let mut matchlist: *mut list_T = ::core::ptr::null_mut::<list_T>();
        let mut matchdict: *mut dict_T = ::core::ptr::null_mut::<dict_T>();
        let mut rettv: typval_T = typval_T {
            v_type: VAR_UNKNOWN,
            v_lock: VAR_UNLOCKED,
            vval: typval_vval_union { v_number: 0 },
        };
        let save_State: ::core::ffi::c_int = State.get();
        '_c2rust_label: {
            if !(*curbuf.ptr()).is_null() {
            } else {
                __assert_fail(
                    b"curbuf != NULL\0".as_ptr() as *const ::core::ffi::c_char,
                    b"src/nvim/insexpand.rs\0".as_ptr() as *const ::core::ffi::c_char,
                    3189 as ::core::ffi::c_uint,
                    b"void expand_by_function(int, char *, Callback *)\0".as_ptr()
                        as *const ::core::ffi::c_char,
                );
            }
        };
        let is_cpt_function: bool = !cb.is_null();
        if !is_cpt_function {
            let mut funcname: *mut ::core::ffi::c_char = get_complete_funcname(type_0);
            if *funcname as ::core::ffi::c_int == NUL {
                return;
            }
            cb = get_insert_callback(type_0);
        }
        let mut args: [typval_T; 3] = [typval_T {
            v_type: VAR_UNKNOWN,
            v_lock: VAR_UNLOCKED,
            vval: typval_vval_union { v_number: 0 },
        }; 3];
        args[0 as ::core::ffi::c_int as usize].v_type = VAR_NUMBER;
        args[1 as ::core::ffi::c_int as usize].v_type = VAR_STRING;
        args[2 as ::core::ffi::c_int as usize].v_type = VAR_UNKNOWN;
        args[0 as ::core::ffi::c_int as usize].vval.v_number = 0 as varnumber_T;
        args[1 as ::core::ffi::c_int as usize].vval.v_string = (if !base.is_null() {
            base as *const ::core::ffi::c_char
        } else {
            b"\0".as_ptr() as *const ::core::ffi::c_char
        })
            as *mut ::core::ffi::c_char;
        let mut pos: pos_T = (*curwin.get()).w_cursor;
        (*textlock.ptr()) += 1;
        if callback_call(
            cb,
            2 as ::core::ffi::c_int,
            &raw mut args as *mut typval_T,
            &raw mut rettv,
        ) {
            match rettv.v_type as ::core::ffi::c_uint {
                4 => {
                    matchlist = rettv.vval.v_list;
                }
                5 => {
                    matchdict = rettv.vval.v_dict;
                }
                8 | _ => {
                    tv_clear(&raw mut rettv);
                }
            }
        }
        (*textlock.ptr()) -= 1;
        (*curwin.get()).w_cursor = pos;
        check_cursor(curwin.get());
        validate_cursor(curwin.get());
        if !equalpos((*curwin.get()).w_cursor, pos) {
            emsg(gettext(E_COMPLDEL.as_ptr()));
        } else if !matchlist.is_null() {
            ins_compl_add_list(matchlist);
        } else if !matchdict.is_null() {
            ins_compl_add_dict(matchdict);
        }
        State.set(save_State);
        if !matchdict.is_null() {
            tv_dict_unref(matchdict);
        }
        if !matchlist.is_null() {
            tv_list_unref(matchlist);
        }
    }
}

#[inline]
pub(crate) unsafe extern "C" fn get_user_highlight_attr(
    mut hlname: *const ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    unsafe {
        if !hlname.is_null() && *hlname as ::core::ffi::c_int != NUL {
            return syn_name2attr(hlname);
        }
        return -1 as ::core::ffi::c_int;
    }
}

pub(crate) unsafe extern "C" fn get_callback_if_cpt_func(
    mut p: *mut ::core::ffi::c_char,
    mut idx: ::core::ffi::c_int,
) -> *mut Callback {
    unsafe {
        if *p as ::core::ffi::c_int == 'o' as ::core::ffi::c_int {
            return &raw mut (*curbuf.get()).b_ofu_cb;
        }
        if *p as ::core::ffi::c_int == 'F' as ::core::ffi::c_int {
            p = p.offset(1);
            if *p as ::core::ffi::c_int != ',' as ::core::ffi::c_int
                && *p as ::core::ffi::c_int != NUL
            {
                return if (*(*curbuf.get()).b_p_cpt_cb.offset(idx as isize)).type_0
                    as ::core::ffi::c_uint
                    != kCallbackNone as ::core::ffi::c_int as ::core::ffi::c_uint
                {
                    (*curbuf.get()).b_p_cpt_cb.offset(idx as isize)
                } else {
                    ::core::ptr::null_mut::<Callback>()
                };
            } else {
                return &raw mut (*curbuf.get()).b_cfu_cb;
            }
        }
        return ::core::ptr::null_mut::<Callback>();
    }
}

pub(crate) unsafe extern "C" fn prepare_cpt_compl_funcs() {
    unsafe {
        let mut cpt: *mut ::core::ffi::c_char = xstrdup((*curbuf.get()).b_p_cpt);
        strip_caret_numbers_in_place(cpt);
        let mut idx: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        let mut p: *mut ::core::ffi::c_char = cpt;
        while *p != 0 {
            while *p as ::core::ffi::c_int == ',' as ::core::ffi::c_int
                || *p as ::core::ffi::c_int == ' ' as ::core::ffi::c_int
            {
                p = p.offset(1);
            }
            if *p as ::core::ffi::c_int == NUL {
                break;
            }
            let mut cb: *mut Callback = get_callback_if_cpt_func(p, idx);
            if !cb.is_null() {
                let mut startcol: ::core::ffi::c_int = 0;
                if get_userdefined_compl_info((*curwin.get()).w_cursor.col, cb, &raw mut startcol)
                    == FAIL
                {
                    if startcol == -3 as ::core::ffi::c_int {
                        (*(*cpt_sources_array.ptr()).offset(idx as isize)).cs_refresh_always =
                            false_0 != 0;
                    } else {
                        startcol = -2 as ::core::ffi::c_int;
                    }
                } else if startcol < 0 as ::core::ffi::c_int
                    || startcol > (*curwin.get()).w_cursor.col
                {
                    startcol = (*curwin.get()).w_cursor.col as ::core::ffi::c_int;
                }
                (*(*cpt_sources_array.ptr()).offset(idx as isize)).cs_startcol = startcol;
            } else {
                (*(*cpt_sources_array.ptr()).offset(idx as isize)).cs_startcol =
                    -3 as ::core::ffi::c_int;
            }
            copy_option_part(
                &raw mut p,
                IObuff.ptr() as *mut ::core::ffi::c_char,
                IOSIZE as size_t,
                b",\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            );
            idx += 1;
        }
        xfree(cpt as *mut ::core::ffi::c_void);
    }
}

pub(crate) unsafe extern "C" fn advance_cpt_sources_index_safe() -> ::core::ffi::c_int {
    unsafe {
        if cpt_sources_index.get() >= 0 as ::core::ffi::c_int
            && cpt_sources_index.get() < cpt_sources_count.get() - 1 as ::core::ffi::c_int
        {
            (*cpt_sources_index.ptr()) += 1;
            return OK;
        }
        semsg(
            gettext(&raw const e_list_index_out_of_range_nr as *const ::core::ffi::c_char),
            cpt_sources_index.get(),
        );
        return FAIL;
    }
}

pub(crate) unsafe extern "C" fn cpt_sources_clear() {
    unsafe {
        let mut ptr_: *mut *mut ::core::ffi::c_void =
            cpt_sources_array.ptr() as *mut *mut ::core::ffi::c_void;
        xfree(*ptr_);
        *ptr_ = NULL;
        let _ = *ptr_;
        cpt_sources_index.set(-1 as ::core::ffi::c_int);
        cpt_sources_count.set(0 as ::core::ffi::c_int);
    }
}

pub(crate) unsafe extern "C" fn setup_cpt_sources() {
    unsafe {
        cpt_sources_clear();
        let mut count: ::core::ffi::c_int = get_cpt_sources_count();
        if count == 0 as ::core::ffi::c_int {
            return;
        }
        cpt_sources_array.set(
            xcalloc(count as size_t, ::core::mem::size_of::<cpt_source_T>()) as *mut cpt_source_T,
        );
        let mut buf: [::core::ffi::c_char; 512] = [0; 512];
        let mut idx: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        let mut p: *mut ::core::ffi::c_char = (*curbuf.get()).b_p_cpt;
        while *p != 0 {
            while *p as ::core::ffi::c_int == ',' as ::core::ffi::c_int
                || *p as ::core::ffi::c_int == ' ' as ::core::ffi::c_int
            {
                p = p.offset(1);
            }
            if *p != 0 {
                (*(*cpt_sources_array.ptr()).offset(idx as isize)).cs_flag = *p;
                memset(
                    &raw mut buf as *mut ::core::ffi::c_char as *mut ::core::ffi::c_void,
                    0 as ::core::ffi::c_int,
                    LSIZE as size_t,
                );
                let mut slen: size_t = copy_option_part(
                    &raw mut p,
                    &raw mut buf as *mut ::core::ffi::c_char,
                    LSIZE as size_t,
                    b",\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
                );
                if slen > 0 as size_t {
                    let mut caret: *mut ::core::ffi::c_char = vim_strchr(
                        &raw mut buf as *mut ::core::ffi::c_char,
                        '^' as ::core::ffi::c_int,
                    );
                    if !caret.is_null() {
                        (*(*cpt_sources_array.ptr()).offset(idx as isize)).cs_max_matches =
                            atoi(caret.offset(1 as ::core::ffi::c_int as isize));
                    }
                }
                idx += 1;
            }
        }
        cpt_sources_count.set(count);
    }
}

pub(crate) unsafe extern "C" fn is_cpt_func_refresh_always() -> bool {
    unsafe {
        let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while i < cpt_sources_count.get() {
            if (*(*cpt_sources_array.ptr()).offset(i as isize)).cs_refresh_always {
                return true_0 != 0;
            }
            i += 1;
        }
        return false_0 != 0;
    }
}

pub(crate) unsafe extern "C" fn get_cpt_func_completion_matches(mut cb: *mut Callback) {
    unsafe {
        let mut cpt_src: *mut cpt_source_T =
            (*cpt_sources_array.ptr()).offset(cpt_sources_index.get() as isize);
        let mut startcol: ::core::ffi::c_int = (*cpt_src).cs_startcol;
        if startcol == -2 as ::core::ffi::c_int || startcol == -3 as ::core::ffi::c_int {
            return;
        }
        set_compl_globals(startcol, (*curwin.get()).w_cursor.col, true_0 != 0);
        if !(*cpt_src).cs_refresh_always {
            ins_compl_insert_bytes(ins_compl_leader(), -1 as ::core::ffi::c_int);
        }
        expand_by_function(0 as ::core::ffi::c_int, (*cpt_compl_pattern.ptr()).data, cb);
        if !(*cpt_src).cs_refresh_always {
            ins_compl_delete(false_0 != 0);
        }
        (*cpt_src).cs_refresh_always = compl_opt_refresh_always.get();
        compl_opt_refresh_always.set(false_0 != 0);
    }
}

pub(crate) unsafe extern "C" fn cpt_compl_refresh() {
    unsafe {
        ins_compl_make_linear();
        let mut cpt: *mut ::core::ffi::c_char = xstrdup((*curbuf.get()).b_p_cpt);
        strip_caret_numbers_in_place(cpt);
        cpt_sources_index.set(0 as ::core::ffi::c_int);
        let mut p: *mut ::core::ffi::c_char = cpt;
        while *p != 0 {
            while *p as ::core::ffi::c_int == ',' as ::core::ffi::c_int
                || *p as ::core::ffi::c_int == ' ' as ::core::ffi::c_int
            {
                p = p.offset(1);
            }
            if *p as ::core::ffi::c_int == NUL {
                break;
            }
            if (*(*cpt_sources_array.ptr()).offset(cpt_sources_index.get() as isize))
                .cs_refresh_always
            {
                let mut cb: *mut Callback = get_callback_if_cpt_func(p, cpt_sources_index.get());
                if !cb.is_null() {
                    remove_old_matches();
                    let mut startcol: ::core::ffi::c_int = 0;
                    let mut ret: ::core::ffi::c_int = get_userdefined_compl_info(
                        (*curwin.get()).w_cursor.col,
                        cb,
                        &raw mut startcol,
                    );
                    if ret == FAIL {
                        if startcol == -3 as ::core::ffi::c_int {
                            (*(*cpt_sources_array.ptr())
                                .offset(cpt_sources_index.get() as isize))
                            .cs_refresh_always = false_0 != 0;
                        } else {
                            startcol = -2 as ::core::ffi::c_int;
                        }
                    } else if startcol < 0 as ::core::ffi::c_int
                        || startcol > (*curwin.get()).w_cursor.col
                    {
                        startcol = (*curwin.get()).w_cursor.col as ::core::ffi::c_int;
                    }
                    (*(*cpt_sources_array.ptr()).offset(cpt_sources_index.get() as isize))
                        .cs_startcol = startcol;
                    if ret == OK {
                        compl_source_start_timer(cpt_sources_index.get());
                        get_cpt_func_completion_matches(cb);
                    }
                }
            }
            copy_option_part(
                &raw mut p,
                IObuff.ptr() as *mut ::core::ffi::c_char,
                IOSIZE as size_t,
                b",\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            );
            if may_advance_cpt_index(p) {
                advance_cpt_sources_index_safe();
            }
        }
        cpt_sources_index.set(-1 as ::core::ffi::c_int);
        xfree(cpt as *mut ::core::ffi::c_void);
        compl_matches.set(ins_compl_make_cyclic());
    }
}
