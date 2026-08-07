//! Running one diff, internal or external.
//!
//! `diff_file` picks between the three: `'diffexpr'`, the external `diff(1)` and
//! the built-in `xdl_diff`.  `check_external_diff` is the probe that decides
//! whether the host's `diff` is usable at all (and caches the answer in
//! `diff_a_works`); `diff_file_internal` is the `xdl_diff` call, with `xdiff_out`
//! as its hunk callback.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;

pub(crate) unsafe extern "C" fn check_external_diff(
    mut diffio: *mut diffio_T,
) -> ::core::ffi::c_int {
    unsafe {
        let mut io_error: bool = false_0 != 0;
        let mut ok: TriState = kFalse;
        loop {
            ok = kFalse;
            let mut fd: *mut FILE = os_fopen(
                (*diffio).dio_orig.din_fname,
                b"w\0".as_ptr() as *const ::core::ffi::c_char,
            );
            if fd.is_null() {
                io_error = true_0 != 0;
            } else {
                if fwrite(
                    b"line1\n\0".as_ptr() as *const ::core::ffi::c_char
                        as *const ::core::ffi::c_void,
                    6 as size_t,
                    1 as size_t,
                    fd,
                ) != 1 as ::core::ffi::c_ulong
                {
                    io_error = true_0 != 0;
                }
                fclose(fd);
                fd = os_fopen(
                    (*diffio).dio_new.din_fname,
                    b"w\0".as_ptr() as *const ::core::ffi::c_char,
                );
                if fd.is_null() {
                    io_error = true_0 != 0;
                } else {
                    if fwrite(
                        b"line2\n\0".as_ptr() as *const ::core::ffi::c_char
                            as *const ::core::ffi::c_void,
                        6 as size_t,
                        1 as size_t,
                        fd,
                    ) != 1 as ::core::ffi::c_ulong
                    {
                        io_error = true_0 != 0;
                    }
                    fclose(fd);
                    fd = if diff_file(diffio) == OK {
                        os_fopen(
                            (*diffio).dio_diff.dout_fname,
                            b"r\0".as_ptr() as *const ::core::ffi::c_char,
                        )
                    } else {
                        ::core::ptr::null_mut::<FILE>()
                    };
                    if fd.is_null() {
                        io_error = true_0 != 0;
                    } else {
                        let mut linebuf: [::core::ffi::c_char; 50] = [0; 50];
                        while !vim_fgets(&raw mut linebuf as *mut ::core::ffi::c_char, LBUFLEN, fd)
                        {
                            if strncmp(
                                &raw mut linebuf as *mut ::core::ffi::c_char,
                                b"1c1\0".as_ptr() as *const ::core::ffi::c_char,
                                3 as size_t,
                            ) == 0 as ::core::ffi::c_int
                                || strncmp(
                                    &raw mut linebuf as *mut ::core::ffi::c_char,
                                    b"@@ -1 +1 @@\0".as_ptr() as *const ::core::ffi::c_char,
                                    11 as size_t,
                                ) == 0 as ::core::ffi::c_int
                            {
                                ok = kTrue;
                            }
                        }
                        fclose(fd);
                    }
                    os_remove((*diffio).dio_diff.dout_fname);
                    os_remove((*diffio).dio_new.din_fname);
                }
                os_remove((*diffio).dio_orig.din_fname);
            }
            if *p_dex.get() as ::core::ffi::c_int != NUL {
                break;
            }
            if diff_a_works.get() as ::core::ffi::c_int != kNone as ::core::ffi::c_int {
                break;
            }
            diff_a_works.set(ok);
            if ok as u64 != 0 {
                break;
            }
        }
        if ok as u64 == 0 {
            if io_error {
                emsg(gettext(b"E810: Cannot read or write temp files\0".as_ptr()
                    as *const ::core::ffi::c_char));
            }
            emsg(gettext(
                b"E97: Cannot create diffs\0".as_ptr() as *const ::core::ffi::c_char
            ));
            diff_a_works.set(kNone);
            return FAIL;
        }
        return OK;
    }
}

pub(crate) unsafe extern "C" fn diff_file_internal(
    mut diffio: *mut diffio_T,
) -> ::core::ffi::c_int {
    unsafe {
        let mut param: xpparam_t = xpparam_t {
            flags: 0,
            anchors: ::core::ptr::null_mut::<*mut ::core::ffi::c_char>(),
            anchors_nr: 0,
        };
        let mut emit_cfg: xdemitconf_t = xdemitconf_t {
            ctxlen: 0,
            interhunkctxlen: 0,
            flags: 0,
            find_func: None,
            find_func_priv: ::core::ptr::null_mut::<::core::ffi::c_void>(),
            hunk_func: None,
        };
        let mut emit_cb: xdemitcb_t = xdemitcb_t {
            priv_0: ::core::ptr::null_mut::<::core::ffi::c_void>(),
            out_hunk: None,
            out_line: None,
        };
        memset(
            &raw mut param as *mut ::core::ffi::c_void,
            0 as ::core::ffi::c_int,
            ::core::mem::size_of::<xpparam_t>(),
        );
        memset(
            &raw mut emit_cfg as *mut ::core::ffi::c_void,
            0 as ::core::ffi::c_int,
            ::core::mem::size_of::<xdemitconf_t>(),
        );
        memset(
            &raw mut emit_cb as *mut ::core::ffi::c_void,
            0 as ::core::ffi::c_int,
            ::core::mem::size_of::<xdemitcb_t>(),
        );
        param.flags = diff_algorithm.get() as ::core::ffi::c_ulong;
        if diff_flags.get() & DIFF_IWHITE != 0 {
            param.flags |= XDF_IGNORE_WHITESPACE_CHANGE as ::core::ffi::c_ulong;
        }
        if diff_flags.get() & DIFF_IWHITEALL != 0 {
            param.flags |= XDF_IGNORE_WHITESPACE as ::core::ffi::c_ulong;
        }
        if diff_flags.get() & DIFF_IWHITEEOL != 0 {
            param.flags |= XDF_IGNORE_WHITESPACE_AT_EOL as ::core::ffi::c_ulong;
        }
        if diff_flags.get() & DIFF_IBLANK != 0 {
            param.flags |= XDF_IGNORE_BLANK_LINES as ::core::ffi::c_ulong;
        }
        emit_cfg.ctxlen = 0 as ::core::ffi::c_long;
        emit_cb.priv_0 = &raw mut (*diffio).dio_diff as *mut ::core::ffi::c_void;
        emit_cfg.hunk_func = Some(
            xdiff_out
                as unsafe extern "C" fn(
                    ::core::ffi::c_int,
                    ::core::ffi::c_int,
                    ::core::ffi::c_int,
                    ::core::ffi::c_int,
                    *mut ::core::ffi::c_void,
                ) -> ::core::ffi::c_int,
        ) as xdl_emit_hunk_consume_func_t;
        if (*diffio).dio_orig.din_mmfile.size as ::core::ffi::c_long > MAX_XDIFF_SIZE
            || (*diffio).dio_new.din_mmfile.size as ::core::ffi::c_long > MAX_XDIFF_SIZE
        {
            emsg(gettext(
                &raw const e_problem_creating_internal_diff as *const ::core::ffi::c_char,
            ));
            return FAIL;
        }
        if xdl_diff(
            &raw mut (*diffio).dio_orig.din_mmfile,
            &raw mut (*diffio).dio_new.din_mmfile,
            &raw mut param,
            &raw mut emit_cfg,
            &raw mut emit_cb,
        ) < 0 as ::core::ffi::c_int
        {
            emsg(gettext(
                &raw const e_problem_creating_internal_diff as *const ::core::ffi::c_char,
            ));
            return FAIL;
        }
        return OK;
    }
}

pub(crate) unsafe extern "C" fn diff_file(mut dio: *mut diffio_T) -> ::core::ffi::c_int {
    unsafe {
        let mut tmp_orig: *mut ::core::ffi::c_char = (*dio).dio_orig.din_fname;
        let mut tmp_new: *mut ::core::ffi::c_char = (*dio).dio_new.din_fname;
        let mut tmp_diff: *mut ::core::ffi::c_char = (*dio).dio_diff.dout_fname;
        if *p_dex.get() as ::core::ffi::c_int != NUL {
            eval_diff(tmp_orig, tmp_new, tmp_diff);
            return OK;
        }
        if (*dio).dio_internal != 0 {
            return diff_file_internal(dio);
        }
        let len: size_t = strlen(tmp_orig)
            .wrapping_add(strlen(tmp_new))
            .wrapping_add(strlen(tmp_diff))
            .wrapping_add(strlen(p_srr.get()))
            .wrapping_add(27 as size_t);
        let cmd: *mut ::core::ffi::c_char = xmalloc(len) as *mut ::core::ffi::c_char;
        if os_env_exists(
            b"DIFF_OPTIONS\0".as_ptr() as *const ::core::ffi::c_char,
            true_0 != 0,
        ) {
            os_unsetenv(b"DIFF_OPTIONS\0".as_ptr() as *const ::core::ffi::c_char);
        }
        vim_snprintf(
            cmd,
            len,
            b"diff %s%s%s%s%s%s%s%s %s\0".as_ptr() as *const ::core::ffi::c_char,
            if diff_a_works.get() as ::core::ffi::c_int == kFalse as ::core::ffi::c_int {
                b"\0".as_ptr() as *const ::core::ffi::c_char
            } else {
                b"-a \0".as_ptr() as *const ::core::ffi::c_char
            },
            b"\0".as_ptr() as *const ::core::ffi::c_char,
            if diff_flags.get() & DIFF_IWHITE != 0 {
                b"-b \0".as_ptr() as *const ::core::ffi::c_char
            } else {
                b"\0".as_ptr() as *const ::core::ffi::c_char
            },
            if diff_flags.get() & DIFF_IWHITEALL != 0 {
                b"-w \0".as_ptr() as *const ::core::ffi::c_char
            } else {
                b"\0".as_ptr() as *const ::core::ffi::c_char
            },
            if diff_flags.get() & DIFF_IWHITEEOL != 0 {
                b"-Z \0".as_ptr() as *const ::core::ffi::c_char
            } else {
                b"\0".as_ptr() as *const ::core::ffi::c_char
            },
            if diff_flags.get() & DIFF_IBLANK != 0 {
                b"-B \0".as_ptr() as *const ::core::ffi::c_char
            } else {
                b"\0".as_ptr() as *const ::core::ffi::c_char
            },
            if diff_flags.get() & DIFF_ICASE != 0 {
                b"-i \0".as_ptr() as *const ::core::ffi::c_char
            } else {
                b"\0".as_ptr() as *const ::core::ffi::c_char
            },
            tmp_orig,
            tmp_new,
        );
        append_redir(cmd, len, p_srr.get(), tmp_diff);
        block_autocmds();
        call_shell(
            cmd,
            kShellOptFilter as ::core::ffi::c_int
                | kShellOptSilent as ::core::ffi::c_int
                | kShellOptDoOut as ::core::ffi::c_int,
            ::core::ptr::null_mut::<::core::ffi::c_char>(),
        );
        unblock_autocmds();
        xfree(cmd as *mut ::core::ffi::c_void);
        return OK;
    }
}

unsafe extern "C" fn xdiff_out(
    mut start_a: ::core::ffi::c_int,
    mut count_a: ::core::ffi::c_int,
    mut start_b: ::core::ffi::c_int,
    mut count_b: ::core::ffi::c_int,
    mut priv_0: *mut ::core::ffi::c_void,
) -> ::core::ffi::c_int {
    unsafe {
        let mut dout: *mut diffout_T = priv_0 as *mut diffout_T;
        ga_grow(&raw mut (*dout).dout_ga, 1 as ::core::ffi::c_int);
        *((*dout).dout_ga.ga_data as *mut diffhunk_T).offset((*dout).dout_ga.ga_len as isize) =
            diffhunk_T {
                lnum_orig: start_a as linenr_T + 1 as linenr_T,
                count_orig: count_a,
                lnum_new: start_b as linenr_T + 1 as linenr_T,
                count_new: count_b,
            };
        (*dout).dout_ga.ga_len += 1;
        return 0 as ::core::ffi::c_int;
    }
}
