//! The `emsg` family: errors, warnings and where they came from.
//!
//! [`emsg_multiline`] is the funnel — it consults `'debug'`, the `:try`
//! stack and `v:errmsg` before anything is displayed — and
//! [`get_emsg_source`] is what prefixes the message with the script and line
//! that raised it.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;

pub unsafe extern "C" fn reset_last_sourcing() {
    unsafe {
        let mut ptr_: *mut *mut ::core::ffi::c_void =
            last_sourcing_name.ptr() as *mut *mut ::core::ffi::c_void;
        xfree(*ptr_);
        *ptr_ = NULL;
        let _ = *ptr_;
        last_sourcing_lnum.set(0 as ::core::ffi::c_int);
    }
}

pub(crate) unsafe extern "C" fn other_sourcing_name() -> bool {
    unsafe {
        if !(*exestack.ptr()).ga_data.is_null()
            && (*exestack.ptr()).ga_len > 0 as ::core::ffi::c_int
            && !(*((*exestack.ptr()).ga_data as *mut estack_T)
                .offset(((*exestack.ptr()).ga_len - 1 as ::core::ffi::c_int) as isize))
            .es_name
            .is_null()
        {
            if !(*last_sourcing_name.ptr()).is_null() {
                return strcmp(
                    (*((*exestack.ptr()).ga_data as *mut estack_T)
                        .offset(((*exestack.ptr()).ga_len - 1 as ::core::ffi::c_int) as isize))
                    .es_name,
                    last_sourcing_name.get(),
                ) != 0 as ::core::ffi::c_int;
            }
            return true_0 != 0;
        }
        return false_0 != 0;
    }
}

pub(crate) unsafe extern "C" fn get_emsg_source() -> *mut ::core::ffi::c_char {
    unsafe {
        if !(*exestack.ptr()).ga_data.is_null()
            && (*exestack.ptr()).ga_len > 0 as ::core::ffi::c_int
            && !(*((*exestack.ptr()).ga_data as *mut estack_T)
                .offset(((*exestack.ptr()).ga_len - 1 as ::core::ffi::c_int) as isize))
            .es_name
            .is_null()
            && other_sourcing_name() as ::core::ffi::c_int != 0
        {
            let mut sname: *mut ::core::ffi::c_char = estack_sfile(ESTACK_NONE);
            let mut tofree: *mut ::core::ffi::c_char = sname;
            if sname.is_null() {
                sname = (*((*exestack.ptr()).ga_data as *mut estack_T)
                    .offset(((*exestack.ptr()).ga_len - 1 as ::core::ffi::c_int) as isize))
                .es_name;
            }
            let p: *const ::core::ffi::c_char =
                gettext(b"Error in %s:\0".as_ptr() as *const ::core::ffi::c_char);
            let buf_len: size_t = strlen(sname)
                .wrapping_add(strlen(p))
                .wrapping_add(1 as size_t);
            let buf: *mut ::core::ffi::c_char = xmalloc(buf_len) as *mut ::core::ffi::c_char;
            snprintf(buf, buf_len, p, sname);
            xfree(tofree as *mut ::core::ffi::c_void);
            return buf;
        }
        return ::core::ptr::null_mut::<::core::ffi::c_char>();
    }
}

pub(crate) unsafe extern "C" fn get_emsg_lnum() -> *mut ::core::ffi::c_char {
    unsafe {
        if !(*((*exestack.ptr()).ga_data as *mut estack_T)
            .offset(((*exestack.ptr()).ga_len - 1 as ::core::ffi::c_int) as isize))
        .es_name
        .is_null()
            && (other_sourcing_name() as ::core::ffi::c_int != 0
                || (*((*exestack.ptr()).ga_data as *mut estack_T)
                    .offset(((*exestack.ptr()).ga_len - 1 as ::core::ffi::c_int) as isize))
                .es_lnum
                    != last_sourcing_lnum.get() as linenr_T)
            && (*((*exestack.ptr()).ga_data as *mut estack_T)
                .offset(((*exestack.ptr()).ga_len - 1 as ::core::ffi::c_int) as isize))
            .es_lnum
                != 0 as linenr_T
        {
            let p: *const ::core::ffi::c_char =
                gettext(b"line %4d:\0".as_ptr() as *const ::core::ffi::c_char);
            let buf_len: size_t = (20 as size_t).wrapping_add(strlen(p));
            let buf: *mut ::core::ffi::c_char = xmalloc(buf_len) as *mut ::core::ffi::c_char;
            snprintf(
                buf,
                buf_len,
                p,
                (*((*exestack.ptr()).ga_data as *mut estack_T)
                    .offset(((*exestack.ptr()).ga_len - 1 as ::core::ffi::c_int) as isize))
                .es_lnum,
            );
            return buf;
        }
        return ::core::ptr::null_mut::<::core::ffi::c_char>();
    }
}

pub unsafe extern "C" fn msg_source(mut hl_id: ::core::ffi::c_int) {
    unsafe {
        static recursive: GlobalCell<bool> = GlobalCell::new(false_0 != 0);
        if recursive.get() {
            return;
        }
        recursive.set(true_0 != 0);
        (*no_wait_return.ptr()) += 1;
        let mut p: *mut ::core::ffi::c_char = get_emsg_source();
        if !p.is_null() {
            msg_scroll.set(true_0);
            msg(p, hl_id);
            xfree(p as *mut ::core::ffi::c_void);
        }
        p = get_emsg_lnum();
        if !p.is_null() {
            msg(p, HLF_N);
            xfree(p as *mut ::core::ffi::c_void);
            last_sourcing_lnum.set(
                (*((*exestack.ptr()).ga_data as *mut estack_T)
                    .offset(((*exestack.ptr()).ga_len - 1 as ::core::ffi::c_int) as isize))
                .es_lnum as ::core::ffi::c_int,
            );
        }
        if (*((*exestack.ptr()).ga_data as *mut estack_T)
            .offset(((*exestack.ptr()).ga_len - 1 as ::core::ffi::c_int) as isize))
        .es_name
        .is_null()
            || other_sourcing_name() as ::core::ffi::c_int != 0
        {
            let mut ptr_: *mut *mut ::core::ffi::c_void =
                last_sourcing_name.ptr() as *mut *mut ::core::ffi::c_void;
            xfree(*ptr_);
            *ptr_ = NULL;
            let _ = *ptr_;
            if !(*((*exestack.ptr()).ga_data as *mut estack_T)
                .offset(((*exestack.ptr()).ga_len - 1 as ::core::ffi::c_int) as isize))
            .es_name
            .is_null()
            {
                last_sourcing_name.set(xstrdup(
                    (*((*exestack.ptr()).ga_data as *mut estack_T)
                        .offset(((*exestack.ptr()).ga_len - 1 as ::core::ffi::c_int) as isize))
                    .es_name,
                ));
                if redirecting() == 0 {
                    msg_putchar_hl('\n' as ::core::ffi::c_int, hl_id);
                }
            }
        }
        (*no_wait_return.ptr()) -= 1;
        recursive.set(false_0 != 0);
    }
}

pub(crate) unsafe extern "C" fn emsg_not_now() -> ::core::ffi::c_int {
    unsafe {
        if emsg_off.get() > 0 as ::core::ffi::c_int
            && vim_strchr(p_debug.get(), 'm' as ::core::ffi::c_int).is_null()
            && vim_strchr(p_debug.get(), 't' as ::core::ffi::c_int).is_null()
            || emsg_skip.get() > 0 as ::core::ffi::c_int
        {
            return true_0;
        }
        return false_0;
    }
}

pub unsafe extern "C" fn emsg_multiline(
    mut s: *const ::core::ffi::c_char,
    mut kind: *const ::core::ffi::c_char,
    mut hl_id: ::core::ffi::c_int,
    mut multiline: bool,
) -> bool {
    unsafe {
        let mut ignore: bool = false_0 != 0;
        if emsg_not_now() != 0 {
            return true_0 != 0;
        }
        (*called_emsg.ptr()) += 1;
        let mut severe: bool = emsg_severe.get();
        emsg_severe.set(false_0 != 0);
        if emsg_off.get() == 0 || !vim_strchr(p_debug.get(), 't' as ::core::ffi::c_int).is_null() {
            if cause_errthrow(
                s,
                multiline,
                is_multihl.get() > 1 as ::core::ffi::c_int,
                severe,
                &raw mut ignore,
            ) {
                if !ignore {
                    (*did_emsg.ptr()) += 1;
                }
                return true_0 != 0;
            }
            if in_assert_fails.get() as ::core::ffi::c_int != 0
                && (*emsg_assert_fails_msg.ptr()).is_null()
            {
                emsg_assert_fails_msg.set(xstrdup(s));
                emsg_assert_fails_lnum.set(
                    (*((*exestack.ptr()).ga_data as *mut estack_T)
                        .offset(((*exestack.ptr()).ga_len - 1 as ::core::ffi::c_int) as isize))
                    .es_lnum as ::core::ffi::c_long,
                );
                xfree(emsg_assert_fails_context.get() as *mut ::core::ffi::c_void);
                emsg_assert_fails_context.set(xstrdup(
                    if (*((*exestack.ptr()).ga_data as *mut estack_T)
                        .offset(((*exestack.ptr()).ga_len - 1 as ::core::ffi::c_int) as isize))
                    .es_name
                    .is_null()
                    {
                        b"\0".as_ptr() as *const ::core::ffi::c_char
                    } else {
                        (*((*exestack.ptr()).ga_data as *mut estack_T)
                            .offset(((*exestack.ptr()).ga_len - 1 as ::core::ffi::c_int) as isize))
                        .es_name as *const ::core::ffi::c_char
                    },
                ));
            }
            set_vim_var_string(VV_ERRMSG, s, -1 as ptrdiff_t);
            if emsg_silent.get() != 0 as ::core::ffi::c_int {
                if !emsg_noredir.get() {
                    msg_start();
                    let mut p: *mut ::core::ffi::c_char = get_emsg_source();
                    if !p.is_null() {
                        let p_len: size_t = strlen(p);
                        *p.offset(p_len as isize) = '\n' as ::core::ffi::c_char;
                        redir_write(p, p_len as ptrdiff_t + 1 as ptrdiff_t);
                        xfree(p as *mut ::core::ffi::c_void);
                    }
                    p = get_emsg_lnum();
                    if !p.is_null() {
                        let p_len_0: size_t = strlen(p);
                        *p.offset(p_len_0 as isize) = '\n' as ::core::ffi::c_char;
                        redir_write(p, p_len_0 as ptrdiff_t + 1 as ptrdiff_t);
                        xfree(p as *mut ::core::ffi::c_void);
                    }
                    redir_write(s, strlen(s) as ptrdiff_t);
                }
                if !(*((*exestack.ptr()).ga_data as *mut estack_T)
                    .offset(((*exestack.ptr()).ga_len - 1 as ::core::ffi::c_int) as isize))
                .es_name
                .is_null()
                    && (*((*exestack.ptr()).ga_data as *mut estack_T)
                        .offset(((*exestack.ptr()).ga_len - 1 as ::core::ffi::c_int) as isize))
                    .es_lnum
                        != 0 as linenr_T
                {
                    logmsg(
                        LOGLVL_DBG,
                        ::core::ptr::null::<::core::ffi::c_char>(),
                        b"emsg_multiline\0".as_ptr() as *const ::core::ffi::c_char,
                        845 as ::core::ffi::c_int,
                        true_0 != 0,
                        b"(:silent) %s (%s (line %d))\0".as_ptr() as *const ::core::ffi::c_char,
                        s,
                        (*((*exestack.ptr()).ga_data as *mut estack_T)
                            .offset(((*exestack.ptr()).ga_len - 1 as ::core::ffi::c_int) as isize))
                        .es_name,
                        (*((*exestack.ptr()).ga_data as *mut estack_T)
                            .offset(((*exestack.ptr()).ga_len - 1 as ::core::ffi::c_int) as isize))
                        .es_lnum,
                    );
                } else {
                    logmsg(
                        LOGLVL_DBG,
                        ::core::ptr::null::<::core::ffi::c_char>(),
                        b"emsg_multiline\0".as_ptr() as *const ::core::ffi::c_char,
                        847 as ::core::ffi::c_int,
                        true_0 != 0,
                        b"(:silent) %s\0".as_ptr() as *const ::core::ffi::c_char,
                        s,
                    );
                }
                return true_0 != 0;
            }
            if !(*((*exestack.ptr()).ga_data as *mut estack_T)
                .offset(((*exestack.ptr()).ga_len - 1 as ::core::ffi::c_int) as isize))
            .es_name
            .is_null()
                && (*((*exestack.ptr()).ga_data as *mut estack_T)
                    .offset(((*exestack.ptr()).ga_len - 1 as ::core::ffi::c_int) as isize))
                .es_lnum
                    != 0 as linenr_T
            {
                logmsg(
                    LOGLVL_INF,
                    ::core::ptr::null::<::core::ffi::c_char>(),
                    b"emsg_multiline\0".as_ptr() as *const ::core::ffi::c_char,
                    855 as ::core::ffi::c_int,
                    true_0 != 0,
                    b"%s (%s (line %d))\0".as_ptr() as *const ::core::ffi::c_char,
                    s,
                    (*((*exestack.ptr()).ga_data as *mut estack_T)
                        .offset(((*exestack.ptr()).ga_len - 1 as ::core::ffi::c_int) as isize))
                    .es_name,
                    (*((*exestack.ptr()).ga_data as *mut estack_T)
                        .offset(((*exestack.ptr()).ga_len - 1 as ::core::ffi::c_int) as isize))
                    .es_lnum,
                );
            } else {
                logmsg(
                    LOGLVL_INF,
                    ::core::ptr::null::<::core::ffi::c_char>(),
                    b"emsg_multiline\0".as_ptr() as *const ::core::ffi::c_char,
                    857 as ::core::ffi::c_int,
                    true_0 != 0,
                    b"%s\0".as_ptr() as *const ::core::ffi::c_char,
                    s,
                );
            }
            ex_exitval.set(1 as ::core::ffi::c_int);
            msg_silent.set(0 as ::core::ffi::c_int);
            cmd_silent.set(false_0 != 0);
            if global_busy.get() != 0 {
                (*global_busy.ptr()) += 1;
            }
            if p_eb.get() != 0 {
                beep_flush();
            } else {
                flush_buffers(FLUSH_MINIMAL);
            }
            (*did_emsg.ptr()) += 1;
        }
        emsg_on_display.set(true_0 != 0);
        if msg_scrolled.get() != 0 as ::core::ffi::c_int {
            need_wait_return.set(true_0 != 0);
        }
        msg_ext_set_kind(kind);
        msg_scroll.set(true_0);
        let mut save_msg_skip_flush: bool = msg_ext_skip_flush.get();
        msg_ext_skip_flush.set(true_0 != 0);
        msg_source(hl_id);
        msg_nowait.set(false_0 != 0);
        let mut rv: ::core::ffi::c_int =
            msg_keep(s, hl_id, false_0 != 0, multiline) as ::core::ffi::c_int;
        msg_ext_skip_flush.set(save_msg_skip_flush);
        return rv != 0;
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn emsg(mut s: *const ::core::ffi::c_char) -> bool {
    unsafe {
        return emsg_multiline(
            s,
            b"emsg\0".as_ptr() as *const ::core::ffi::c_char,
            HLF_E,
            false_0 != 0,
        );
    }
}

pub unsafe extern "C" fn emsg_invreg(mut name: ::core::ffi::c_int) {
    unsafe {
        semsg(
            gettext(b"E354: Invalid register name: '%s'\0".as_ptr() as *const ::core::ffi::c_char),
            transchar_buf(::core::ptr::null::<buf_T>(), name),
        );
    }
}

pub unsafe extern "C" fn semsg(fmt: *const ::core::ffi::c_char, mut c2rust_args: ...) -> bool {
    unsafe {
        let mut ret: bool = false;
        let mut ap: ::core::ffi::VaList;
        ap = c2rust_args.clone();
        ret = semsgv(fmt, ap);
        return ret;
    }
}

pub unsafe extern "C" fn semsg_multiline(
    mut kind: *const ::core::ffi::c_char,
    fmt: *const ::core::ffi::c_char,
    mut c2rust_args: ...
) -> bool {
    unsafe {
        let mut ret: bool = false;
        let mut ap: ::core::ffi::VaList;
        static errbuf: GlobalCell<[::core::ffi::c_char; 8192]> = GlobalCell::new([0; 8192]);
        if emsg_not_now() != 0 {
            return true_0 != 0;
        }
        ap = c2rust_args.clone();
        vim_vsnprintf(
            errbuf.ptr() as *mut ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 8192]>(),
            fmt,
            ap,
        );
        ret = emsg_multiline(
            errbuf.ptr() as *mut ::core::ffi::c_char,
            kind,
            HLF_E,
            true_0 != 0,
        );
        return ret;
    }
}

pub(crate) unsafe extern "C" fn semsgv(
    mut fmt: *const ::core::ffi::c_char,
    mut ap: ::core::ffi::VaList,
) -> bool {
    unsafe {
        static errbuf: GlobalCell<[::core::ffi::c_char; 1025]> = GlobalCell::new([0; 1025]);
        if emsg_not_now() != 0 {
            return true_0 != 0;
        }
        vim_vsnprintf(
            errbuf.ptr() as *mut ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 1025]>(),
            fmt,
            ap,
        );
        return emsg(errbuf.ptr() as *mut ::core::ffi::c_char);
    }
}

pub unsafe extern "C" fn iemsg(mut s: *const ::core::ffi::c_char) {
    unsafe {
        if emsg_not_now() != 0 {
            return;
        }
        emsg(s);
    }
}

pub unsafe extern "C" fn siemsg(mut s: *const ::core::ffi::c_char, mut c2rust_args: ...) {
    unsafe {
        if emsg_not_now() != 0 {
            return;
        }
        let mut ap: ::core::ffi::VaList;
        ap = c2rust_args.clone();
        semsgv(s, ap);
    }
}

pub unsafe extern "C" fn internal_error(mut where_0: *const ::core::ffi::c_char) {
    unsafe {
        siemsg(
            gettext(&raw const e_intern2 as *const ::core::ffi::c_char),
            where_0,
        );
    }
}

pub(crate) unsafe extern "C" fn msg_semsg_event(mut argv: *mut *mut ::core::ffi::c_void) {
    unsafe {
        let mut s: *mut ::core::ffi::c_char =
            *argv.offset(0 as ::core::ffi::c_int as isize) as *mut ::core::ffi::c_char;
        emsg(s);
        xfree(s as *mut ::core::ffi::c_void);
    }
}

pub unsafe extern "C" fn msg_schedule_semsg(fmt: *const ::core::ffi::c_char, mut c2rust_args: ...) {
    unsafe {
        let mut ap: ::core::ffi::VaList;
        ap = c2rust_args.clone();
        vim_vsnprintf(
            IObuff.ptr() as *mut ::core::ffi::c_char,
            IOSIZE as size_t,
            fmt,
            ap,
        );
        let mut s: *mut ::core::ffi::c_char = xstrdup(IObuff.ptr() as *mut ::core::ffi::c_char);
        loop_schedule_deferred(
            main_loop.ptr(),
            Event {
                handler: Some(
                    msg_semsg_event as unsafe extern "C" fn(*mut *mut ::core::ffi::c_void) -> (),
                ),
                argv: [
                    s as *mut ::core::ffi::c_void,
                    ::core::ptr::null_mut::<::core::ffi::c_void>(),
                    ::core::ptr::null_mut::<::core::ffi::c_void>(),
                    ::core::ptr::null_mut::<::core::ffi::c_void>(),
                    ::core::ptr::null_mut::<::core::ffi::c_void>(),
                    ::core::ptr::null_mut::<::core::ffi::c_void>(),
                    ::core::ptr::null_mut::<::core::ffi::c_void>(),
                    ::core::ptr::null_mut::<::core::ffi::c_void>(),
                    ::core::ptr::null_mut::<::core::ffi::c_void>(),
                    ::core::ptr::null_mut::<::core::ffi::c_void>(),
                ],
            },
        );
    }
}

pub(crate) unsafe extern "C" fn msg_semsg_multiline_event(mut argv: *mut *mut ::core::ffi::c_void) {
    unsafe {
        let mut s: *mut ::core::ffi::c_char =
            *argv.offset(0 as ::core::ffi::c_int as isize) as *mut ::core::ffi::c_char;
        emsg_multiline(
            s,
            b"emsg\0".as_ptr() as *const ::core::ffi::c_char,
            HLF_E,
            true_0 != 0,
        );
        xfree(s as *mut ::core::ffi::c_void);
    }
}

pub unsafe extern "C" fn msg_schedule_semsg_multiline(
    fmt: *const ::core::ffi::c_char,
    mut c2rust_args: ...
) {
    unsafe {
        let mut ap: ::core::ffi::VaList;
        ap = c2rust_args.clone();
        vim_vsnprintf(
            IObuff.ptr() as *mut ::core::ffi::c_char,
            IOSIZE as size_t,
            fmt,
            ap,
        );
        let mut s: *mut ::core::ffi::c_char = xstrdup(IObuff.ptr() as *mut ::core::ffi::c_char);
        loop_schedule_deferred(
            main_loop.ptr(),
            Event {
                handler: Some(
                    msg_semsg_multiline_event
                        as unsafe extern "C" fn(*mut *mut ::core::ffi::c_void) -> (),
                ),
                argv: [
                    s as *mut ::core::ffi::c_void,
                    ::core::ptr::null_mut::<::core::ffi::c_void>(),
                    ::core::ptr::null_mut::<::core::ffi::c_void>(),
                    ::core::ptr::null_mut::<::core::ffi::c_void>(),
                    ::core::ptr::null_mut::<::core::ffi::c_void>(),
                    ::core::ptr::null_mut::<::core::ffi::c_void>(),
                    ::core::ptr::null_mut::<::core::ffi::c_void>(),
                    ::core::ptr::null_mut::<::core::ffi::c_void>(),
                    ::core::ptr::null_mut::<::core::ffi::c_void>(),
                    ::core::ptr::null_mut::<::core::ffi::c_void>(),
                ],
            },
        );
    }
}

pub unsafe extern "C" fn give_warning(
    mut message: *const ::core::ffi::c_char,
    mut hl: bool,
    mut hist: bool,
) {
    unsafe {
        if msg_silent.get() != 0 as ::core::ffi::c_int {
            return;
        }
        let mut save_msg_hist_off: bool = msg_hist_off.get();
        msg_hist_off.set(!hist);
        (*no_wait_return.ptr()) += 1;
        set_vim_var_string(VV_WARNINGMSG, message, -1 as ptrdiff_t);
        let mut ptr_: *mut *mut ::core::ffi::c_void =
            keep_msg.ptr() as *mut *mut ::core::ffi::c_void;
        xfree(*ptr_);
        *ptr_ = NULL;
        let _ = *ptr_;
        if hl {
            keep_msg_hl_id.set(HLF_W);
        } else {
            keep_msg_hl_id.set(0 as ::core::ffi::c_int);
        }
        if (*msg_ext_kind.ptr()).is_null() {
            msg_ext_set_kind(b"wmsg\0".as_ptr() as *const ::core::ffi::c_char);
        }
        if msg(message, keep_msg_hl_id.get()) as ::core::ffi::c_int != 0
            && msg_scrolled.get() == 0 as ::core::ffi::c_int
        {
            set_keep_msg(message, keep_msg_hl_id.get());
        }
        msg_didout.set(false_0 != 0);
        msg_nowait.set(true_0 != 0);
        msg_col.set(0 as ::core::ffi::c_int);
        (*no_wait_return.ptr()) -= 1;
        msg_hist_off.set(save_msg_hist_off);
    }
}

pub unsafe extern "C" fn swmsg(
    mut hl: bool,
    fmt: *const ::core::ffi::c_char,
    mut c2rust_args: ...
) {
    unsafe {
        let mut args: ::core::ffi::VaList;
        args = c2rust_args.clone();
        vim_vsnprintf(
            IObuff.ptr() as *mut ::core::ffi::c_char,
            IOSIZE as size_t,
            fmt,
            args,
        );
        give_warning(IObuff.ptr() as *mut ::core::ffi::c_char, hl, true_0 != 0);
    }
}
