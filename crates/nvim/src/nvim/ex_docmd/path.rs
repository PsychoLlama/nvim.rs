//! The working directory, and 'findfunc' — the callback `:find` and
//! `gf` resolve a name through.

#[allow(unused_imports)]
use super::*;

pub(crate) unsafe extern "C" fn get_findfunc_callback() -> *mut Callback {
    return if *(*curbuf.get()).b_p_ffu as c_int != NUL {
        &raw mut (*curbuf.get()).b_ffu_cb
    } else {
        ffu_cb.ptr()
    };
}

pub(crate) unsafe extern "C" fn call_findfunc(
    mut pat: *mut c_char,
    mut cmdcomplete: BoolVarValue,
) -> *mut list_T {
    let saved_sctx: sctx_T = current_sctx.get();
    let mut args: [typval_T; 3] = [typval_T {
        v_type: VAR_UNKNOWN,
        v_lock: VAR_UNLOCKED,
        vval: typval_vval_union { v_number: 0 },
    }; 3];
    args[0 as c_int as usize].v_type = VAR_STRING;
    args[0 as c_int as usize].vval.v_string = pat;
    args[1 as c_int as usize].v_type = VAR_BOOL;
    args[1 as c_int as usize].vval.v_bool = cmdcomplete;
    args[2 as c_int as usize].v_type = VAR_UNKNOWN;
    (*textlock.ptr()) += 1;
    let mut ctx: *mut sctx_T = get_option_sctx(kOptFindfunc);
    if !ctx.is_null() {
        current_sctx.set(*ctx);
    }
    let mut cb: *mut Callback = get_findfunc_callback();
    let mut rettv: typval_T = typval_T {
        v_type: VAR_UNKNOWN,
        v_lock: VAR_UNLOCKED,
        vval: typval_vval_union { v_number: 0 },
    };
    let mut retval: c_int = callback_call(
        cb,
        2 as c_int,
        &raw mut args as *mut typval_T,
        &raw mut rettv,
    ) as c_int;
    current_sctx.set(saved_sctx);
    (*textlock.ptr()) -= 1;
    let mut retlist: *mut list_T = ::core::ptr::null_mut::<list_T>();
    if retval == OK {
        if rettv.v_type as c_uint == VAR_LIST as c_int as c_uint {
            retlist = tv_list_copy(
                ::core::ptr::null::<vimconv_T>(),
                rettv.vval.v_list,
                false_0 != 0,
                get_copyID(),
            );
        } else {
            emsg(gettext(
                &raw const e_invalid_return_type_from_findfunc as *const c_char,
            ));
        }
        tv_clear(&raw mut rettv);
    }
    return retlist;
}

pub unsafe extern "C" fn expand_findfunc(
    mut pat: *mut c_char,
    mut files: *mut *mut *mut c_char,
    mut numMatches: *mut c_int,
) -> c_int {
    *numMatches = 0 as c_int;
    *files = ::core::ptr::null_mut::<*mut c_char>();
    let mut l: *mut list_T = call_findfunc(pat, kBoolVarTrue);
    if l.is_null() {
        return FAIL;
    }
    let mut len: c_int = tv_list_len(l);
    if len == 0 as c_int {
        tv_list_free(l);
        return FAIL;
    }
    *files = xmalloc(::core::mem::size_of::<*mut c_char>().wrapping_mul(len as size_t))
        as *mut *mut c_char;
    let mut idx: c_int = 0 as c_int;
    let l_: *const list_T = l;
    if !l_.is_null() {
        let mut li: *const listitem_T = (*l_).lv_first;
        while !li.is_null() {
            if (*li).li_tv.v_type as c_uint == VAR_STRING as c_int as c_uint {
                *(*files).offset(idx as isize) = xstrdup((*li).li_tv.vval.v_string);
                idx += 1;
            }
            li = (*li).li_next;
        }
    }
    *numMatches = idx;
    tv_list_free(l);
    return OK;
}

pub(crate) unsafe extern "C" fn findfunc_find_file(
    mut findarg: *mut c_char,
    mut findarg_len: size_t,
    mut count: c_int,
) -> *mut c_char {
    let mut ret_fname: *mut c_char = ::core::ptr::null_mut::<c_char>();
    let cc: c_char = *findarg.offset(findarg_len as isize);
    *findarg.offset(findarg_len as isize) = NUL as c_char;
    let mut fname_list: *mut list_T = call_findfunc(findarg, kBoolVarFalse);
    let mut fname_count: c_int = tv_list_len(fname_list);
    if fname_count == 0 as c_int {
        semsg(
            gettext(&raw const e_cant_find_file_str_in_path as *const c_char),
            findarg,
        );
    } else if count > fname_count {
        semsg(
            gettext(&raw const e_no_more_file_str_found_in_path as *const c_char),
            findarg,
        );
    } else {
        let mut li: *mut listitem_T = tv_list_find(fname_list, count - 1 as c_int);
        if !li.is_null() && (*li).li_tv.v_type as c_uint == VAR_STRING as c_int as c_uint {
            ret_fname = xstrdup((*li).li_tv.vval.v_string);
        }
    }
    if !fname_list.is_null() {
        tv_list_free(fname_list);
    }
    *findarg.offset(findarg_len as isize) = cc;
    return ret_fname;
}

pub unsafe extern "C" fn did_set_findfunc(mut args: *mut optset_T) -> *const c_char {
    let mut buf: *mut buf_T = (*args).os_buf as *mut buf_T;
    let mut retval: c_int = 0;
    if (*args).os_flags & OPT_LOCAL as c_int != 0 {
        retval = option_set_callback_func((*buf).b_p_ffu, &raw mut (*buf).b_ffu_cb);
    } else {
        retval = option_set_callback_func(p_ffu.get(), ffu_cb.ptr());
        if (*args).os_flags & OPT_GLOBAL as c_int == 0 {
            callback_free(&raw mut (*buf).b_ffu_cb);
        }
    }
    if retval == FAIL {
        return &raw const e_invarg as *const c_char;
    }
    let mut varp: *mut *mut c_char = (*args).os_varp as *mut *mut c_char;
    let mut name: *mut c_char = get_scriptlocal_funcname(*varp);
    if !name.is_null() {
        free_string_option(*varp);
        *varp = name;
    }
    return ::core::ptr::null::<c_char>();
}

pub unsafe extern "C" fn set_ref_in_findfunc(mut copyID: c_int) -> bool {
    let mut abort_0: bool = false_0 != 0;
    abort_0 = set_ref_in_callback(
        ffu_cb.ptr(),
        copyID,
        ::core::ptr::null_mut::<*mut ht_stack_T>(),
        ::core::ptr::null_mut::<*mut list_stack_T>(),
    );
    return abort_0;
}

pub(crate) unsafe extern "C" fn get_prevdir(mut scope: CdScope) -> *mut c_char {
    match scope as c_int {
        1 => return (*curtab.get()).tp_prevdir,
        0 => return (*curwin.get()).w_prevdir,
        _ => return prev_dir.get(),
    };
}

pub(crate) unsafe extern "C" fn post_chdir(mut scope: CdScope, mut trigger_dirchanged: bool) {
    let mut ptr_: *mut *mut c_void = &raw mut (*curwin.get()).w_localdir as *mut *mut c_void;
    xfree(*ptr_);
    *ptr_ = NULL_1;
    let _ = *ptr_;
    if scope as c_int >= kCdScopeTabpage as c_int {
        let mut ptr__0: *mut *mut c_void = &raw mut (*curtab.get()).tp_localdir as *mut *mut c_void;
        xfree(*ptr__0);
        *ptr__0 = NULL_1;
        let _ = *ptr__0;
    }
    if (scope as c_int) < kCdScopeGlobal as c_int {
        let mut pdir: *mut c_char = get_prevdir(scope);
        if (*globaldir.ptr()).is_null() && !pdir.is_null() {
            globaldir.set(xstrdup(pdir));
        }
    }
    let mut cwd: [c_char; 4096] = [0; 4096];
    if os_dirname(&raw mut cwd as *mut c_char, MAXPATHL as size_t) != OK {
        return;
    }
    match scope as c_int {
        2 => {
            let mut ptr__1: *mut *mut c_void = globaldir.ptr() as *mut *mut c_void;
            xfree(*ptr__1);
            *ptr__1 = NULL_1;
            let _ = *ptr__1;
        }
        1 => {
            (*curtab.get()).tp_localdir = xstrdup(&raw mut cwd as *mut c_char);
        }
        0 => {
            (*curwin.get()).w_localdir = xstrdup(&raw mut cwd as *mut c_char);
        }
        -1 => {
            abort();
        }
        _ => {}
    }
    last_chdir_reason.set(::core::ptr::null_mut::<c_char>());
    shorten_fnames(vim_strchr(p_cpo.get(), CPO_NOSYMLINKS).is_null() as c_int);
    if trigger_dirchanged {
        do_autocmd_dirchanged(
            &raw mut cwd as *mut c_char,
            scope,
            kCdCauseManual,
            false_0 != 0,
        );
    }
}

pub unsafe extern "C" fn changedir_func(mut new_dir: *mut c_char, mut scope: CdScope) -> bool {
    if new_dir.is_null() || allbuf_locked() as c_int != 0 {
        return false_0 != 0;
    }
    let mut pdir: *mut c_char = ::core::ptr::null_mut::<c_char>();
    if strcmp(new_dir, b"-\0".as_ptr() as *const c_char) == 0 as c_int {
        pdir = get_prevdir(scope);
        if pdir.is_null() {
            emsg(gettext(
                b"E186: No previous directory\0".as_ptr() as *const c_char
            ));
            return false_0 != 0;
        }
        new_dir = pdir;
    }
    if os_dirname(NameBuff.ptr() as *mut c_char, MAXPATHL as size_t) == OK {
        pdir = xstrdup(NameBuff.ptr() as *mut c_char);
    } else {
        pdir = ::core::ptr::null_mut::<c_char>();
    }
    if *new_dir as c_int == NUL && p_cdh.get() != 0 {
        expand_env(
            b"$HOME\0".as_ptr() as *const c_char as *mut c_char,
            NameBuff.ptr() as *mut c_char,
            MAXPATHL,
        );
        new_dir = NameBuff.ptr() as *mut c_char;
    }
    let mut dir_differs: bool = pdir.is_null() || pathcmp(pdir, new_dir, -1 as c_int) != 0 as c_int;
    if dir_differs {
        do_autocmd_dirchanged(new_dir, scope, kCdCauseManual, true_0 != 0);
        if vim_chdir(new_dir) != 0 as c_int {
            emsg(gettext(&raw const e_failed as *const c_char));
            xfree(pdir as *mut c_void);
            return false_0 != 0;
        }
    }
    let mut pp: *mut *mut c_char = ::core::ptr::null_mut::<*mut c_char>();
    match scope as c_int {
        1 => {
            pp = &raw mut (*curtab.get()).tp_prevdir;
        }
        0 => {
            pp = &raw mut (*curwin.get()).w_prevdir;
        }
        _ => {
            pp = prev_dir.ptr();
        }
    }
    xfree(*pp as *mut c_void);
    *pp = pdir;
    post_chdir(scope, dir_differs);
    return true_0 != 0;
}

pub unsafe extern "C" fn ex_cd(mut eap: *mut exarg_T) {
    let mut new_dir: *mut c_char = (*eap).arg;
    if *new_dir as c_int == NUL && p_cdh.get() == 0 {
        ex_pwd(::core::ptr::null_mut::<exarg_T>());
        return;
    }
    let mut scope: CdScope = kCdScopeGlobal;
    match (*eap).cmdidx as c_int {
        448 | 449 => {
            scope = kCdScopeTabpage;
        }
        225 | 226 => {
            scope = kCdScopeWindow;
        }
        _ => {}
    }
    if changedir_func(new_dir, scope) {
        if KeyTyped.get() as c_int != 0 || p_verbose.get() >= 5 as OptInt {
            ex_pwd(eap);
        }
    }
}

pub(crate) unsafe extern "C" fn ex_pwd(mut _eap: *mut exarg_T) {
    if os_dirname(NameBuff.ptr() as *mut c_char, MAXPATHL as size_t) == OK {
        if p_verbose.get() > 0 as OptInt {
            let mut context: *mut c_char = b"global\0".as_ptr() as *const c_char as *mut c_char;
            if !(*last_chdir_reason.ptr()).is_null() {
                context = last_chdir_reason.get();
            } else if !(*curwin.get()).w_localdir.is_null() {
                context = b"window\0".as_ptr() as *const c_char as *mut c_char;
            } else if !(*curtab.get()).tp_localdir.is_null() {
                context = b"tabpage\0".as_ptr() as *const c_char as *mut c_char;
            }
            smsg(
                0 as c_int,
                b"[%s] %s\0".as_ptr() as *const c_char,
                context,
                NameBuff.ptr() as *mut c_char,
            );
        } else {
            msg(NameBuff.ptr() as *mut c_char, 0 as c_int);
        }
    } else {
        emsg(gettext(b"E187: Unknown\0".as_ptr() as *const c_char));
    };
}
