//! The autocommands that bracket a write.
//!
//! `buf_write_do_autocmds` fires the `*WritePre`/`*WriteCmd` family before
//! anything is written, and has to cope with what they may have done: deleted
//! the buffer, renamed it, changed its line count, or written the file
//! themselves. `buf_write_do_post_autocmds` fires the matching `*WritePost`
//! family afterwards.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;

pub(crate) unsafe extern "C" fn buf_write_do_autocmds(
    mut buf: *mut buf_T,
    mut fnamep: *mut *mut ::core::ffi::c_char,
    mut sfnamep: *mut *mut ::core::ffi::c_char,
    mut ffnamep: *mut *mut ::core::ffi::c_char,
    mut start: linenr_T,
    mut endp: *mut linenr_T,
    mut eap: *mut exarg_T,
    mut append: bool,
    mut filtering: bool,
    mut reset_changed: bool,
    mut overwriting: bool,
    mut whole: bool,
    orig_start: pos_T,
    orig_end: pos_T,
) -> ::core::ffi::c_int {
    unsafe {
        let mut old_line_count: linenr_T = (*buf).b_ml.ml_line_count;
        let mut msg_save: ::core::ffi::c_int = msg_scroll.get();
        let mut aco: aco_save_T = aco_save_T {
            use_aucmd_win_idx: 0,
            save_curwin_handle: 0,
            new_curwin_handle: 0,
            save_prevwin_handle: 0,
            new_curbuf: bufref_T {
                br_buf: ::core::ptr::null_mut::<buf_T>(),
                br_fnum: 0,
                br_buf_free_count: 0,
            },
            tp_localdir: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            globaldir: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            save_VIsual_active: false,
            save_prompt_insert: 0,
        };
        let mut did_cmd: bool = false_0 != 0;
        let mut nofile_err: bool = false_0 != 0;
        let mut empty_memline: bool = (*buf).b_ml.ml_mfp.is_null();
        let mut bufref: bufref_T = bufref_T {
            br_buf: ::core::ptr::null_mut::<buf_T>(),
            br_fnum: 0,
            br_buf_free_count: 0,
        };
        let mut sfname: *mut ::core::ffi::c_char = *sfnamep;
        let mut buf_ffname: bool = *ffnamep == (*buf).b_ffname;
        let mut buf_sfname: bool = sfname == (*buf).b_sfname;
        let mut buf_fname_f: bool = *fnamep == (*buf).b_ffname;
        let mut buf_fname_s: bool = *fnamep == (*buf).b_sfname;
        aucmd_prepbuf(&raw mut aco, buf);
        set_bufref(&raw mut bufref, buf);
        if append {
            did_cmd = apply_autocmds_exarg(
                EVENT_FILEAPPENDCMD,
                sfname,
                sfname,
                false_0 != 0,
                curbuf.get(),
                eap,
            );
            if !did_cmd {
                if overwriting as ::core::ffi::c_int != 0
                    && bt_nofilename(curbuf.get()) as ::core::ffi::c_int != 0
                {
                    nofile_err = true_0 != 0;
                } else {
                    apply_autocmds_exarg(
                        EVENT_FILEAPPENDPRE,
                        sfname,
                        sfname,
                        false_0 != 0,
                        curbuf.get(),
                        eap,
                    );
                }
            }
        } else if filtering {
            apply_autocmds_exarg(
                EVENT_FILTERWRITEPRE,
                ::core::ptr::null_mut::<::core::ffi::c_char>(),
                sfname,
                false_0 != 0,
                curbuf.get(),
                eap,
            );
        } else if reset_changed as ::core::ffi::c_int != 0 && whole as ::core::ffi::c_int != 0 {
            let mut was_changed: bool = curbufIsChanged();
            did_cmd = apply_autocmds_exarg(
                EVENT_BUFWRITECMD,
                sfname,
                sfname,
                false_0 != 0,
                curbuf.get(),
                eap,
            );
            if did_cmd {
                if was_changed as ::core::ffi::c_int != 0 && !curbufIsChanged() {
                    u_unchanged(curbuf.get());
                    u_update_save_nr(curbuf.get());
                }
            } else if overwriting as ::core::ffi::c_int != 0
                && bt_nofilename(curbuf.get()) as ::core::ffi::c_int != 0
            {
                nofile_err = true_0 != 0;
            } else {
                apply_autocmds_exarg(
                    EVENT_BUFWRITEPRE,
                    sfname,
                    sfname,
                    false_0 != 0,
                    curbuf.get(),
                    eap,
                );
            }
        } else {
            did_cmd = apply_autocmds_exarg(
                EVENT_FILEWRITECMD,
                sfname,
                sfname,
                false_0 != 0,
                curbuf.get(),
                eap,
            );
            if !did_cmd {
                if overwriting as ::core::ffi::c_int != 0
                    && bt_nofilename(curbuf.get()) as ::core::ffi::c_int != 0
                {
                    nofile_err = true_0 != 0;
                } else {
                    apply_autocmds_exarg(
                        EVENT_FILEWRITEPRE,
                        sfname,
                        sfname,
                        false_0 != 0,
                        curbuf.get(),
                        eap,
                    );
                }
            }
        }
        aucmd_restbuf(&raw mut aco);
        if !bufref_valid(&raw mut bufref) {
            buf = ::core::ptr::null_mut::<buf_T>();
        }
        if buf.is_null()
            || (*buf).b_ml.ml_mfp.is_null() && !empty_memline
            || did_cmd as ::core::ffi::c_int != 0
            || nofile_err as ::core::ffi::c_int != 0
            || aborting() as ::core::ffi::c_int != 0
        {
            if !buf.is_null()
                && (*cmdmod.ptr()).cmod_flags & CMOD_LOCKMARKS as ::core::ffi::c_int != 0
            {
                (*buf).b_op_start = orig_start;
                (*buf).b_op_end = orig_end;
            }
            (*no_wait_return.ptr()) -= 1;
            msg_scroll.set(msg_save);
            if nofile_err {
                semsg(
                    gettext(
                        (e_no_matching_autocommands_for_buftype_str_buffer.ptr() as *const _)
                            as *const ::core::ffi::c_char,
                    ),
                    (*curbuf.get()).b_p_bt,
                );
            }
            if nofile_err as ::core::ffi::c_int != 0 || aborting() as ::core::ffi::c_int != 0 {
                return FAIL;
            }
            if did_cmd {
                if buf.is_null() {
                    return OK;
                }
                if overwriting {
                    ml_timestamp(buf);
                    if append {
                        (*buf).b_flags &= !BF_NEW;
                    } else {
                        (*buf).b_flags &= !BF_WRITE_MASK;
                    }
                }
                if reset_changed as ::core::ffi::c_int != 0
                    && (*buf).b_changed != 0
                    && !append
                    && (overwriting as ::core::ffi::c_int != 0
                        || !vim_strchr(p_cpo.get(), CPO_PLUS).is_null())
                {
                    return FAIL;
                }
                return OK;
            }
            if !aborting() {
                emsg(gettext(
                    b"E203: Autocommands deleted or unloaded buffer to be written\0".as_ptr()
                        as *const ::core::ffi::c_char,
                ));
            }
            return FAIL;
        }
        if (*buf).b_ml.ml_line_count != old_line_count {
            if whole {
                *endp = (*buf).b_ml.ml_line_count;
            } else if (*buf).b_ml.ml_line_count > old_line_count {
                *endp += (*buf).b_ml.ml_line_count - old_line_count;
            } else {
                *endp -= old_line_count - (*buf).b_ml.ml_line_count;
                if *endp < start {
                    (*no_wait_return.ptr()) -= 1;
                    msg_scroll.set(msg_save);
                    emsg(gettext(
                        b"E204: Autocommand changed number of lines in unexpected way\0".as_ptr()
                            as *const ::core::ffi::c_char,
                    ));
                    return FAIL;
                }
            }
        }
        if buf_ffname {
            *ffnamep = (*buf).b_ffname;
        }
        if buf_sfname {
            *sfnamep = (*buf).b_sfname;
        }
        if buf_fname_f {
            *fnamep = (*buf).b_ffname;
        }
        if buf_fname_s {
            *fnamep = (*buf).b_sfname;
        }
        return NOTDONE;
    }
}

pub(crate) unsafe extern "C" fn buf_write_do_post_autocmds(
    mut buf: *mut buf_T,
    mut fname: *mut ::core::ffi::c_char,
    mut eap: *mut exarg_T,
    mut append: bool,
    mut filtering: bool,
    mut reset_changed: bool,
    mut whole: bool,
) {
    unsafe {
        let mut aco: aco_save_T = aco_save_T {
            use_aucmd_win_idx: 0,
            save_curwin_handle: 0,
            new_curwin_handle: 0,
            save_prevwin_handle: 0,
            new_curbuf: bufref_T {
                br_buf: ::core::ptr::null_mut::<buf_T>(),
                br_fnum: 0,
                br_buf_free_count: 0,
            },
            tp_localdir: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            globaldir: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            save_VIsual_active: false,
            save_prompt_insert: 0,
        };
        (*curbuf.get()).b_no_eol_lnum = 0 as ::core::ffi::c_int as linenr_T;
        aucmd_prepbuf(&raw mut aco, buf);
        if append {
            apply_autocmds_exarg(
                EVENT_FILEAPPENDPOST,
                fname,
                fname,
                false_0 != 0,
                curbuf.get(),
                eap,
            );
        } else if filtering {
            apply_autocmds_exarg(
                EVENT_FILTERWRITEPOST,
                ::core::ptr::null_mut::<::core::ffi::c_char>(),
                fname,
                false_0 != 0,
                curbuf.get(),
                eap,
            );
        } else if reset_changed as ::core::ffi::c_int != 0 && whole as ::core::ffi::c_int != 0 {
            apply_autocmds_exarg(
                EVENT_BUFWRITEPOST,
                fname,
                fname,
                false_0 != 0,
                curbuf.get(),
                eap,
            );
        } else {
            apply_autocmds_exarg(
                EVENT_FILEWRITEPOST,
                fname,
                fname,
                false_0 != 0,
                curbuf.get(),
                eap,
            );
        }
        aucmd_restbuf(&raw mut aco);
    }
}
