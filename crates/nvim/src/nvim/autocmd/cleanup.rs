//! Deleting autocommands, and the deferred cleanup that makes it safe.
//!
//! An `AutoCmd` is never freed while a firing walk might be holding a
//! pointer to it: `aucmd_del` marks it and sets `au_need_clean`, and
//! `au_cleanup` does the actual removal once `active_apc_list` is empty.
//! The `aupat_*` half is the buffer-local pattern format -- `<buffer=N>` --
//! which is a pattern in the same table but matched by buffer number, and
//! `aubuflocal_remove` is what a wiped-out buffer's patterns go through.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;

pub(crate) unsafe extern "C" fn aucmd_del(mut ac: *mut AutoCmd) {
    unsafe {
        if !(*ac).pat.is_null() && {
            (*(*ac).pat).refcount = (*(*ac).pat).refcount.wrapping_sub(1);
            (*(*ac).pat).refcount == 0 as size_t
        } {
            let mut ptr_: *mut *mut ::core::ffi::c_void =
                &raw mut (*(*ac).pat).pat as *mut *mut ::core::ffi::c_void;
            xfree(*ptr_);
            *ptr_ = NULL_0;
            let _ = *ptr_;
            vim_regfree((*(*ac).pat).reg_prog);
            xfree((*ac).pat as *mut ::core::ffi::c_void);
        }
        (*ac).pat = ::core::ptr::null_mut::<AutoPat>();
        if !(*ac).handler_cmd.is_null() {
            let mut ptr__0: *mut *mut ::core::ffi::c_void =
                &raw mut (*ac).handler_cmd as *mut *mut ::core::ffi::c_void;
            xfree(*ptr__0);
            *ptr__0 = NULL_0;
            let _ = *ptr__0;
        } else {
            callback_free(&raw mut (*ac).handler_fn);
        }
        let mut ptr__1: *mut *mut ::core::ffi::c_void =
            &raw mut (*ac).desc as *mut *mut ::core::ffi::c_void;
        xfree(*ptr__1);
        *ptr__1 = NULL_0;
        let _ = *ptr__1;
        au_need_clean.set(true_0 != 0);
    }
}

pub unsafe extern "C" fn aucmd_del_for_event_and_group(
    mut event: event_T,
    mut group: ::core::ffi::c_int,
) {
    unsafe {
        let acs: *mut AutoCmdVec =
            (autocmds.ptr() as *mut AutoCmdVec).offset(event as ::core::ffi::c_int as isize);
        let mut i: size_t = 0 as size_t;
        while i < (*acs).size {
            let ac: *mut AutoCmd = (*acs).items.offset(i as isize);
            if !(*ac).pat.is_null() && (*(*ac).pat).group == group {
                aucmd_del(ac);
            }
            i = i.wrapping_add(1);
        }
        au_cleanup();
    }
}

pub(crate) unsafe extern "C" fn au_cleanup() {
    unsafe {
        if autocmd_busy.get() as ::core::ffi::c_int != 0 || !au_need_clean.get() {
            return;
        }
        let mut event: event_T = EVENT_BUFADD;
        while (event as ::core::ffi::c_int) < NUM_EVENTS as ::core::ffi::c_int {
            let acs: *mut AutoCmdVec =
                (autocmds.ptr() as *mut AutoCmdVec).offset(event as ::core::ffi::c_int as isize);
            let mut nsize: size_t = 0 as size_t;
            let mut i: size_t = 0 as size_t;
            while i < (*acs).size {
                let ac: *mut AutoCmd = (*acs).items.offset(i as isize);
                if nsize != i {
                    *(*acs).items.offset(nsize as isize) = *ac;
                }
                if !(*ac).pat.is_null() {
                    nsize = nsize.wrapping_add(1);
                }
                i = i.wrapping_add(1);
            }
            if nsize == 0 as size_t {
                xfree((*acs).items as *mut ::core::ffi::c_void);
                (*acs).capacity = 0 as size_t;
                (*acs).size = (*acs).capacity;
                (*acs).items = ::core::ptr::null_mut::<AutoCmd>();
            } else {
                (*acs).size = nsize;
            }
            event = (event as ::core::ffi::c_int + 1 as ::core::ffi::c_int) as event_T;
        }
        au_need_clean.set(false_0 != 0);
    }
}

pub unsafe extern "C" fn au_get_autocmds_for_event(mut event: event_T) -> *mut AutoCmdVec {
    unsafe {
        return (autocmds.ptr() as *mut AutoCmdVec).offset(event as ::core::ffi::c_int as isize);
    }
}

pub unsafe extern "C" fn aubuflocal_remove(mut buf: *mut buf_T) {
    unsafe {
        let mut apc: *mut AutoPatCmd = active_apc_list.get();
        while !apc.is_null() {
            if (*buf).handle == (*apc).arg_bufnr {
                (*apc).arg_bufnr = 0 as ::core::ffi::c_int;
            }
            apc = (*apc).next;
        }
        let mut event: event_T = EVENT_BUFADD;
        while (event as ::core::ffi::c_int) < NUM_EVENTS as ::core::ffi::c_int {
            let acs: *mut AutoCmdVec =
                (autocmds.ptr() as *mut AutoCmdVec).offset(event as ::core::ffi::c_int as isize);
            let mut i: size_t = 0 as size_t;
            while i < (*acs).size {
                let ac: *mut AutoCmd = (*acs).items.offset(i as isize);
                if !((*ac).pat.is_null() || (*(*ac).pat).buflocal_nr != (*buf).handle) {
                    aucmd_del(ac);
                    if p_verbose.get() >= 6 as OptInt {
                        verbose_enter();
                        smsg(
                            0 as ::core::ffi::c_int,
                            gettext(b"auto-removing autocommand: %s <buffer=%d>\0".as_ptr()
                                as *const ::core::ffi::c_char),
                            event_nr2name(event),
                            (*buf).handle,
                        );
                        verbose_leave();
                    }
                }
                i = i.wrapping_add(1);
            }
            event = (event as ::core::ffi::c_int + 1 as ::core::ffi::c_int) as event_T;
        }
        au_cleanup();
    }
}

pub unsafe extern "C" fn aupat_is_buflocal(
    mut pat: *const ::core::ffi::c_char,
    mut patlen: ::core::ffi::c_int,
) -> bool {
    unsafe {
        return patlen >= 8 as ::core::ffi::c_int
            && strncmp(
                pat,
                b"<buffer\0".as_ptr() as *const ::core::ffi::c_char,
                7 as size_t,
            ) == 0 as ::core::ffi::c_int
            && *pat.offset((patlen - 1 as ::core::ffi::c_int) as isize) as ::core::ffi::c_int
                == '>' as ::core::ffi::c_int;
    }
}

pub unsafe extern "C" fn aupat_get_buflocal_nr(
    mut pat: *const ::core::ffi::c_char,
    mut patlen: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    unsafe {
        '_c2rust_label: {
            if aupat_is_buflocal(pat, patlen) {
            } else {
                __assert_fail(
                    b"aupat_is_buflocal(pat, patlen)\0".as_ptr() as *const ::core::ffi::c_char,
                    b"src/nvim/autocmd.rs\0".as_ptr() as *const ::core::ffi::c_char,
                    2514 as ::core::ffi::c_uint,
                    b"int aupat_get_buflocal_nr(const char *, int)\0".as_ptr()
                        as *const ::core::ffi::c_char,
                );
            }
        };
        if patlen == 8 as ::core::ffi::c_int {
            return (*curbuf.get()).handle as ::core::ffi::c_int;
        }
        if patlen > 9 as ::core::ffi::c_int
            && *pat.offset(7 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                == '=' as ::core::ffi::c_int
        {
            if patlen == 13 as ::core::ffi::c_int
                && strncasecmp(
                    pat as *mut ::core::ffi::c_char,
                    b"<buffer=abuf>\0".as_ptr() as *const ::core::ffi::c_char
                        as *mut ::core::ffi::c_char,
                    13 as ::core::ffi::c_int as size_t,
                ) == 0 as ::core::ffi::c_int
            {
                return autocmd_bufnr.get();
            }
            if skipdigits(pat.offset(8 as ::core::ffi::c_int as isize))
                == pat
                    .offset(patlen as isize)
                    .offset(-(1 as ::core::ffi::c_int as isize))
                    as *mut ::core::ffi::c_char
            {
                return atoi(pat.offset(8 as ::core::ffi::c_int as isize));
            }
        }
        return 0 as ::core::ffi::c_int;
    }
}

pub unsafe extern "C" fn aupat_normalize_buflocal_pat(
    mut dest: *mut ::core::ffi::c_char,
    mut pat: *const ::core::ffi::c_char,
    mut patlen: ::core::ffi::c_int,
    mut buflocal_nr: ::core::ffi::c_int,
) {
    unsafe {
        '_c2rust_label: {
            if aupat_is_buflocal(pat, patlen) {
            } else {
                __assert_fail(
                    b"aupat_is_buflocal(pat, patlen)\0".as_ptr() as *const ::core::ffi::c_char,
                    b"src/nvim/autocmd.rs\0".as_ptr() as *const ::core::ffi::c_char,
                    2539 as ::core::ffi::c_uint,
                    b"void aupat_normalize_buflocal_pat(char *, const char *, int, int)\0".as_ptr()
                        as *const ::core::ffi::c_char,
                );
            }
        };
        if buflocal_nr == 0 as ::core::ffi::c_int {
            buflocal_nr = (*curbuf.get()).handle as ::core::ffi::c_int;
        }
        snprintf(
            dest,
            BUFLOCAL_PAT_LEN as ::core::ffi::c_int as size_t,
            b"<buffer=%d>\0".as_ptr() as *const ::core::ffi::c_char,
            buflocal_nr,
        );
    }
}
