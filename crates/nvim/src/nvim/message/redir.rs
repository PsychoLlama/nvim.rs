//! Where a message goes besides the screen.
//!
//! `:redir` (to a variable, a register or a file) and `'verbosefile'` both
//! tee the message stream; [`redir_write`] is the tee, and the `verbose_*`
//! pair brackets the sections of code that write to it.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;

pub unsafe extern "C" fn verb_msg(mut s: *const ::core::ffi::c_char) -> ::core::ffi::c_int {
    unsafe {
        verbose_enter();
        let mut n: ::core::ffi::c_int =
            msg_keep(s, 0 as ::core::ffi::c_int, false_0 != 0, false_0 != 0) as ::core::ffi::c_int;
        verbose_leave();
        return n;
    }
}

pub(crate) unsafe extern "C" fn redir_write(str: *const ::core::ffi::c_char, maxlen: ptrdiff_t) {
    unsafe {
        let mut s: *const ::core::ffi::c_char = str;
        if maxlen == 0 as ptrdiff_t {
            return;
        }
        if redir_off.get() {
            return;
        }
        if *p_vfile.get() as ::core::ffi::c_int != NUL && (*verbose_fd.ptr()).is_null() {
            verbose_open();
        }
        if redirecting() != 0 {
            if *s as ::core::ffi::c_int != '\n' as ::core::ffi::c_int
                && *s as ::core::ffi::c_int != '\r' as ::core::ffi::c_int
            {
                while redir_col.get() < msg_col.get() {
                    if !(*capture_ga.ptr()).is_null() {
                        ga_concat_len(
                            capture_ga.get(),
                            b" \0".as_ptr() as *const ::core::ffi::c_char,
                            1 as size_t,
                        );
                    }
                    if redir_reg.get() != 0 {
                        write_reg_contents(
                            redir_reg.get(),
                            b" \0".as_ptr() as *const ::core::ffi::c_char,
                            1 as ssize_t,
                            true_0,
                        );
                    } else if redir_vname.get() {
                        var_redir_str(
                            b" \0".as_ptr() as *const ::core::ffi::c_char,
                            -1 as ::core::ffi::c_int,
                        );
                    } else if !(*redir_fd.ptr()).is_null() {
                        fputs(
                            b" \0".as_ptr() as *const ::core::ffi::c_char,
                            redir_fd.get(),
                        );
                    }
                    if !(*verbose_fd.ptr()).is_null() {
                        fputs(
                            b" \0".as_ptr() as *const ::core::ffi::c_char,
                            verbose_fd.get(),
                        );
                    }
                    (*redir_col.ptr()) += 1;
                }
            }
            let mut len: size_t = if maxlen == -1 as ptrdiff_t {
                strlen(s)
            } else {
                maxlen as size_t
            };
            if !(*capture_ga.ptr()).is_null() {
                ga_concat_len(capture_ga.get(), str, len);
            }
            if redir_reg.get() != 0 {
                write_reg_contents(redir_reg.get(), s, len as ssize_t, true_0);
            }
            if redir_vname.get() {
                var_redir_str(s, maxlen as ::core::ffi::c_int);
            }
            while *s as ::core::ffi::c_int != NUL
                && (maxlen < 0 as ptrdiff_t
                    || (s.offset_from(str) as ::core::ffi::c_int as ptrdiff_t) < maxlen)
            {
                if redir_reg.get() == 0 && !redir_vname.get() && (*capture_ga.ptr()).is_null() {
                    if !(*redir_fd.ptr()).is_null() {
                        putc(*s as ::core::ffi::c_int, redir_fd.get());
                    }
                }
                if !(*verbose_fd.ptr()).is_null() {
                    putc(*s as ::core::ffi::c_int, verbose_fd.get());
                }
                if *s as ::core::ffi::c_int == '\r' as ::core::ffi::c_int
                    || *s as ::core::ffi::c_int == '\n' as ::core::ffi::c_int
                {
                    redir_col.set(0 as ::core::ffi::c_int);
                } else if *s as ::core::ffi::c_int == '\t' as ::core::ffi::c_int {
                    (*redir_col.ptr()) +=
                        8 as ::core::ffi::c_int - redir_col.get() % 8 as ::core::ffi::c_int;
                } else {
                    (*redir_col.ptr()) += 1;
                }
                s = s.offset(1);
            }
            if msg_silent.get() != 0 as ::core::ffi::c_int {
                msg_col.set(redir_col.get());
            }
        }
    }
}

pub unsafe extern "C" fn redirecting() -> ::core::ffi::c_int {
    unsafe {
        return (!(*redir_fd.ptr()).is_null()
            || *p_vfile.get() as ::core::ffi::c_int != NUL
            || redir_reg.get() != 0
            || redir_vname.get() as ::core::ffi::c_int != 0
            || !(*capture_ga.ptr()).is_null()) as ::core::ffi::c_int;
    }
}

pub unsafe extern "C" fn verbose_enter() {
    unsafe {
        if *p_vfile.get() as ::core::ffi::c_int != NUL {
            (*msg_silent.ptr()) += 1;
        }
        if !msg_ext_skip_verbose.get() {
            if msg_ext_kind.get() != verbose_kind.get() {
                pre_verbose_kind.set(msg_ext_kind.get());
            }
            msg_ext_set_kind(b"verbose\0".as_ptr() as *const ::core::ffi::c_char);
        }
        msg_ext_skip_verbose.set(false_0 != 0);
    }
}

pub unsafe extern "C" fn verbose_leave() {
    unsafe {
        if *p_vfile.get() as ::core::ffi::c_int != NUL {
            (*msg_silent.ptr()) -= 1;
            if msg_silent.get() < 0 as ::core::ffi::c_int {
                msg_silent.set(0 as ::core::ffi::c_int);
            }
        }
        if !(*pre_verbose_kind.ptr()).is_null() {
            msg_ext_set_kind(pre_verbose_kind.get());
            pre_verbose_kind.set(::core::ptr::null::<::core::ffi::c_char>());
        }
    }
}

pub unsafe extern "C" fn verbose_enter_scroll() {
    unsafe {
        verbose_enter();
        if *p_vfile.get() as ::core::ffi::c_int == NUL {
            msg_scroll.set(true_0);
        }
    }
}

pub unsafe extern "C" fn verbose_leave_scroll() {
    unsafe {
        verbose_leave();
        if *p_vfile.get() as ::core::ffi::c_int == NUL {
            cmdline_row.set(msg_row.get());
        }
    }
}

pub unsafe extern "C" fn verbose_stop() {
    unsafe {
        if !(*verbose_fd.ptr()).is_null() {
            fclose(verbose_fd.get());
            verbose_fd.set(::core::ptr::null_mut::<FILE>());
        }
        verbose_did_open.set(false_0 != 0);
    }
}

pub unsafe extern "C" fn verbose_open() -> ::core::ffi::c_int {
    unsafe {
        if (*verbose_fd.ptr()).is_null() && !verbose_did_open.get() {
            verbose_did_open.set(true_0 != 0);
            verbose_fd.set(os_fopen(
                p_vfile.get(),
                b"a\0".as_ptr() as *const ::core::ffi::c_char,
            ));
            if (*verbose_fd.ptr()).is_null() {
                semsg(
                    gettext(&raw const e_notopen as *const ::core::ffi::c_char),
                    p_vfile.get(),
                );
                return FAIL;
            }
        }
        return OK;
    }
}
