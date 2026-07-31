//! Noticing that a file changed underneath us.
//!
//! `check_timestamps` sweeps every buffer whenever Nvim regains focus or
//! returns to the main loop; `buf_check_timestamp` is the per-buffer test that
//! compares the file's mtime, size and mode against what was recorded when it
//! was read, asks the user (or `FileChangedShell`) what to do about it, and
//! `buf_reload` carries out a reload — re-reading into a scratch buffer and
//! moving the lines across so that marks, folds and undo survive.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;

pub unsafe extern "C" fn time_differs(
    mut file_info: *const FileInfo,
    mut mtime: int64_t,
    mut mtime_ns: int64_t,
) -> bool {
    unsafe {
        return (*file_info).stat.st_mtim.tv_nsec as int64_t != mtime_ns
            || (*file_info).stat.st_mtim.tv_sec as int64_t - mtime > 1 as int64_t
            || mtime - (*file_info).stat.st_mtim.tv_sec as int64_t > 1 as int64_t;
    }
}

pub unsafe extern "C" fn check_timestamps(mut focus: ::core::ffi::c_int) -> ::core::ffi::c_int {
    unsafe {
        if no_check_timestamps.get() > 0 as ::core::ffi::c_int {
            return false_0;
        }
        if focus != 0 && did_check_timestamps.get() as ::core::ffi::c_int != 0 {
            need_check_timestamps.set(true_0 != 0);
            return false_0;
        }
        let mut didit: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        if !stuff_empty()
            || global_busy.get() != 0
            || typebuf_typed() == 0
            || autocmd_busy.get() as ::core::ffi::c_int != 0
            || (*curbuf.get()).b_ro_locked > 0 as ::core::ffi::c_int
            || allbuf_lock.get() > 0 as ::core::ffi::c_int
        {
            need_check_timestamps.set(true_0 != 0);
        } else {
            (*no_wait_return.ptr()) += 1;
            did_check_timestamps.set(true_0 != 0);
            already_warned.set(false_0 != 0);
            let mut buf: *mut buf_T = firstbuf.get();
            while !buf.is_null() {
                if (*buf).b_nwindows > 0 as ::core::ffi::c_int {
                    let mut bufref: bufref_T = bufref_T {
                        br_buf: ::core::ptr::null_mut::<buf_T>(),
                        br_fnum: 0,
                        br_buf_free_count: 0,
                    };
                    set_bufref(&raw mut bufref, buf);
                    let n: ::core::ffi::c_int = buf_check_timestamp(buf);
                    didit = if didit > n { didit } else { n };
                    if n > 0 as ::core::ffi::c_int && !bufref_valid(&raw mut bufref) {
                        buf = firstbuf.get();
                    }
                }
                buf = (*buf).b_next;
            }
            (*no_wait_return.ptr()) -= 1;
            need_check_timestamps.set(false_0 != 0);
            if need_wait_return.get() as ::core::ffi::c_int != 0 && didit == 2 as ::core::ffi::c_int
            {
                msg_puts(b"\n\0".as_ptr() as *const ::core::ffi::c_char);
                ui_flush();
            }
        }
        return didit;
    }
}

pub(crate) unsafe extern "C" fn move_lines(
    mut frombuf: *mut buf_T,
    mut tobuf: *mut buf_T,
) -> ::core::ffi::c_int {
    unsafe {
        let mut tbuf: *mut buf_T = curbuf.get();
        let mut retval: ::core::ffi::c_int = OK;
        curbuf.set(tobuf);
        let mut lnum: linenr_T = 1 as linenr_T;
        while lnum <= (*frombuf).b_ml.ml_line_count {
            let mut p: *mut ::core::ffi::c_char = xmemdupz(
                ml_get_buf(frombuf, lnum) as *const ::core::ffi::c_void,
                ml_get_buf_len(frombuf, lnum) as size_t,
            ) as *mut ::core::ffi::c_char;
            if ml_append(lnum - 1 as linenr_T, p, 0 as colnr_T, false_0 != 0) == FAIL {
                xfree(p as *mut ::core::ffi::c_void);
                retval = FAIL;
                break;
            } else {
                xfree(p as *mut ::core::ffi::c_void);
                lnum += 1;
            }
        }
        if retval != FAIL {
            curbuf.set(frombuf);
            let mut lnum_0: linenr_T = (*curbuf.get()).b_ml.ml_line_count;
            while lnum_0 > 0 as linenr_T {
                if ml_delete(lnum_0) == FAIL {
                    retval = FAIL;
                    break;
                } else {
                    lnum_0 -= 1;
                }
            }
        }
        curbuf.set(tbuf);
        return retval;
    }
}

pub unsafe extern "C" fn buf_check_timestamp(mut buf: *mut buf_T) -> ::core::ffi::c_int {
    unsafe {
        let mut retval: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        let mut mesg: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
        let mut mesg2: *mut ::core::ffi::c_char =
            b"\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        let mut helpmesg: bool = false_0 != 0;
        let mut reload: C2Rust_Unnamed_31 = RELOAD_NONE;
        let mut can_reload: bool = false_0 != 0;
        let mut orig_size: uint64_t = (*buf).b_orig_size;
        let mut orig_mode: ::core::ffi::c_int = (*buf).b_orig_mode;
        static busy: GlobalCell<bool> = GlobalCell::new(false_0 != 0);
        let mut bufref: bufref_T = bufref_T {
            br_buf: ::core::ptr::null_mut::<buf_T>(),
            br_fnum: 0,
            br_buf_free_count: 0,
        };
        set_bufref(&raw mut bufref, buf);
        if !(*buf).terminal.is_null()
            || (*buf).b_ffname.is_null()
            || (*buf).b_ml.ml_mfp.is_null()
            || !bt_normal(buf)
            || (*buf).b_saving as ::core::ffi::c_int != 0
            || busy.get() as ::core::ffi::c_int != 0
        {
            return 0 as ::core::ffi::c_int;
        }
        let mut file_info: FileInfo = FileInfo {
            stat: uv_stat_t {
                st_dev: 0,
                st_mode: 0,
                st_nlink: 0,
                st_uid: 0,
                st_gid: 0,
                st_rdev: 0,
                st_ino: 0,
                st_size: 0,
                st_blksize: 0,
                st_blocks: 0,
                st_flags: 0,
                st_gen: 0,
                st_atim: uv_timespec_t {
                    tv_sec: 0,
                    tv_nsec: 0,
                },
                st_mtim: uv_timespec_t {
                    tv_sec: 0,
                    tv_nsec: 0,
                },
                st_ctim: uv_timespec_t {
                    tv_sec: 0,
                    tv_nsec: 0,
                },
                st_birthtim: uv_timespec_t {
                    tv_sec: 0,
                    tv_nsec: 0,
                },
            },
        };
        let mut file_info_ok: bool = false;
        if (*buf).b_flags & BF_NOTEDITED == 0 && (*buf).b_mtime != 0 as int64_t && {
            file_info_ok = os_fileinfo((*buf).b_ffname, &raw mut file_info);
            !file_info_ok
                || time_differs(&raw mut file_info, (*buf).b_mtime, (*buf).b_mtime_ns)
                    as ::core::ffi::c_int
                    != 0
                || file_info.stat.st_mode as ::core::ffi::c_int != (*buf).b_orig_mode
        } {
            let prev_b_mtime: int64_t = (*buf).b_mtime;
            retval = 1 as ::core::ffi::c_int;
            if !file_info_ok {
                (*buf).b_mtime = -1 as int64_t;
                (*buf).b_orig_size = 0 as uint64_t;
                (*buf).b_orig_mode = 0 as ::core::ffi::c_int;
            } else {
                buf_store_file_info(buf, &raw mut file_info);
            }
            if !os_isdir((*buf).b_fname) {
                if (if (*buf).b_p_ar >= 0 as ::core::ffi::c_int {
                    (*buf).b_p_ar
                } else {
                    p_ar.get()
                }) != 0
                    && !bufIsChanged(buf)
                    && file_info_ok as ::core::ffi::c_int != 0
                {
                    reload = RELOAD_NORMAL;
                } else {
                    let mut reason: *mut ::core::ffi::c_char =
                        ::core::ptr::null_mut::<::core::ffi::c_char>();
                    let mut reasonlen: size_t = 0;
                    if !file_info_ok {
                        reason = b"deleted\0".as_ptr() as *const ::core::ffi::c_char
                            as *mut ::core::ffi::c_char;
                        reasonlen = ::core::mem::size_of::<[::core::ffi::c_char; 8]>()
                            .wrapping_sub(1 as usize) as size_t;
                    } else if bufIsChanged(buf) {
                        reason = b"conflict\0".as_ptr() as *const ::core::ffi::c_char
                            as *mut ::core::ffi::c_char;
                        reasonlen = ::core::mem::size_of::<[::core::ffi::c_char; 9]>()
                            .wrapping_sub(1 as usize) as size_t;
                    } else if orig_size != (*buf).b_orig_size
                        || buf_contents_changed(buf) as ::core::ffi::c_int != 0
                    {
                        reason = b"changed\0".as_ptr() as *const ::core::ffi::c_char
                            as *mut ::core::ffi::c_char;
                        reasonlen = ::core::mem::size_of::<[::core::ffi::c_char; 8]>()
                            .wrapping_sub(1 as usize) as size_t;
                    } else if orig_mode != (*buf).b_orig_mode {
                        reason = b"mode\0".as_ptr() as *const ::core::ffi::c_char
                            as *mut ::core::ffi::c_char;
                        reasonlen = ::core::mem::size_of::<[::core::ffi::c_char; 5]>()
                            .wrapping_sub(1 as usize) as size_t;
                    } else {
                        reason = b"time\0".as_ptr() as *const ::core::ffi::c_char
                            as *mut ::core::ffi::c_char;
                        reasonlen = ::core::mem::size_of::<[::core::ffi::c_char; 5]>()
                            .wrapping_sub(1 as usize) as size_t;
                    }
                    busy.set(true_0 != 0);
                    set_vim_var_string(
                        VV_FCS_REASON,
                        reason,
                        reasonlen as ::core::ffi::c_int as ptrdiff_t,
                    );
                    set_vim_var_string(
                        VV_FCS_CHOICE,
                        b"\0".as_ptr() as *const ::core::ffi::c_char,
                        0 as ptrdiff_t,
                    );
                    (*allbuf_lock.ptr()) += 1;
                    let mut n: bool = apply_autocmds(
                        EVENT_FILECHANGEDSHELL,
                        (*buf).b_fname,
                        (*buf).b_fname,
                        false_0 != 0,
                        buf,
                    );
                    (*allbuf_lock.ptr()) -= 1;
                    busy.set(false_0 != 0);
                    if n {
                        if !bufref_valid(&raw mut bufref) {
                            emsg(gettext(
                                b"E246: FileChangedShell autocommand deleted buffer\0".as_ptr()
                                    as *const ::core::ffi::c_char,
                            ));
                        }
                        let mut s: *mut ::core::ffi::c_char = get_vim_var_str(VV_FCS_CHOICE);
                        if strcmp(s, b"reload\0".as_ptr() as *const ::core::ffi::c_char)
                            == 0 as ::core::ffi::c_int
                            && *reason as ::core::ffi::c_int != 'd' as ::core::ffi::c_int
                        {
                            reload = RELOAD_NORMAL;
                        } else if strcmp(s, b"edit\0".as_ptr() as *const ::core::ffi::c_char)
                            == 0 as ::core::ffi::c_int
                        {
                            reload = RELOAD_DETECT;
                        } else if strcmp(s, b"ask\0".as_ptr() as *const ::core::ffi::c_char)
                            == 0 as ::core::ffi::c_int
                        {
                            n = false_0 != 0;
                        } else {
                            return 2 as ::core::ffi::c_int;
                        }
                    }
                    if !n {
                        if *reason as ::core::ffi::c_int == 'd' as ::core::ffi::c_int {
                            if prev_b_mtime != -1 as int64_t {
                                mesg = gettext(b"E211: File \"%s\" no longer available\0".as_ptr()
                                    as *const ::core::ffi::c_char);
                            }
                        } else {
                            helpmesg = true_0 != 0;
                            can_reload = true_0 != 0;
                            if *reason.offset(2 as ::core::ffi::c_int as isize)
                                as ::core::ffi::c_int
                                == 'n' as ::core::ffi::c_int
                            {
                                mesg = gettext(
                                b"W12: Warning: File \"%s\" has changed and the buffer was changed in Vim as well\0"
                                    .as_ptr() as *const ::core::ffi::c_char,
                            );
                                mesg2 = gettext(b"See \":help W12\" for more info.\0".as_ptr()
                                    as *const ::core::ffi::c_char);
                            } else if *reason.offset(1 as ::core::ffi::c_int as isize)
                                as ::core::ffi::c_int
                                == 'h' as ::core::ffi::c_int
                            {
                                mesg = gettext(
                                    b"W11: Warning: File \"%s\" has changed since editing started\0"
                                        .as_ptr()
                                        as *const ::core::ffi::c_char,
                                );
                                mesg2 = gettext(b"See \":help W11\" for more info.\0".as_ptr()
                                    as *const ::core::ffi::c_char);
                            } else if *reason as ::core::ffi::c_int == 'm' as ::core::ffi::c_int {
                                mesg = gettext(
                                b"W16: Warning: Mode of file \"%s\" has changed since editing started\0"
                                    .as_ptr() as *const ::core::ffi::c_char,
                            );
                                mesg2 = gettext(b"See \":help W16\" for more info.\0".as_ptr()
                                    as *const ::core::ffi::c_char);
                            } else {
                                (*buf).b_mtime_read = (*buf).b_mtime;
                                (*buf).b_mtime_read_ns = (*buf).b_mtime_ns;
                            }
                        }
                    }
                }
            }
        } else if (*buf).b_flags & BF_NEW != 0
            && (*buf).b_flags & BF_NEW_W == 0
            && os_path_exists((*buf).b_ffname) as ::core::ffi::c_int != 0
        {
            retval = 1 as ::core::ffi::c_int;
            mesg = gettext(
                b"W13: Warning: File \"%s\" has been created after editing started\0".as_ptr()
                    as *const ::core::ffi::c_char,
            );
            (*buf).b_flags |= BF_NEW_W;
            can_reload = true_0 != 0;
        }
        if !mesg.is_null() {
            let mut path: *mut ::core::ffi::c_char = home_replace_save(buf, (*buf).b_fname);
            if !helpmesg {
                mesg2 = b"\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
            }
            let tbufsize: size_t = strlen(path)
                .wrapping_add(strlen(mesg))
                .wrapping_add(strlen(mesg2))
                .wrapping_add(3 as size_t);
            let tbuf: *mut ::core::ffi::c_char = xmalloc(tbufsize) as *mut ::core::ffi::c_char;
            let mut tbuflen: ::core::ffi::c_int = snprintf(tbuf, tbufsize, mesg, path);
            set_vim_var_string(VV_WARNINGMSG, tbuf, tbuflen as ptrdiff_t);
            if can_reload {
                if *mesg2 as ::core::ffi::c_int != NUL {
                    snprintf(
                        tbuf.offset(tbuflen as isize),
                        tbufsize.wrapping_sub(tbuflen as size_t),
                        b"\n%s\0".as_ptr() as *const ::core::ffi::c_char,
                        mesg2,
                    );
                }
                match do_dialog(
                    VIM_WARNING as ::core::ffi::c_int,
                    gettext(b"Warning\0".as_ptr() as *const ::core::ffi::c_char),
                    tbuf,
                    gettext(b"&OK\n&Load File\nLoad File &and Options\0".as_ptr()
                        as *const ::core::ffi::c_char),
                    1 as ::core::ffi::c_int,
                    ::core::ptr::null::<::core::ffi::c_char>(),
                    true_0,
                ) {
                    2 => {
                        reload = RELOAD_NORMAL;
                    }
                    3 => {
                        reload = RELOAD_DETECT;
                    }
                    _ => {}
                }
            } else if State.get() > MODE_NORMAL_BUSY
                || State.get() & MODE_CMDLINE != 0
                || already_warned.get() as ::core::ffi::c_int != 0
            {
                if *mesg2 as ::core::ffi::c_int != NUL {
                    snprintf(
                        tbuf.offset(tbuflen as isize),
                        tbufsize.wrapping_sub(tbuflen as size_t),
                        b"; %s\0".as_ptr() as *const ::core::ffi::c_char,
                        mesg2,
                    );
                }
                emsg(tbuf);
                retval = 2 as ::core::ffi::c_int;
            } else {
                if !autocmd_busy.get() {
                    msg_start();
                    msg_puts_hl(tbuf, HLF_E as ::core::ffi::c_int, true_0 != 0);
                    if *mesg2 as ::core::ffi::c_int != NUL {
                        msg_puts_hl(mesg2, HLF_W as ::core::ffi::c_int, true_0 != 0);
                    }
                    msg_clr_eos();
                    msg_end();
                    if emsg_silent.get() == 0 as ::core::ffi::c_int
                        && !in_assert_fails.get()
                        && !ui_has(kUIMessages)
                    {
                        msg_delay(1004 as uint64_t, true_0 != 0);
                        redraw_cmdline.set(false_0 != 0);
                    }
                }
                already_warned.set(true_0 != 0);
            }
            xfree(tbuf as *mut ::core::ffi::c_void);
            xfree(path as *mut ::core::ffi::c_void);
        }
        if reload as ::core::ffi::c_uint != RELOAD_NONE as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            buf_reload(
                buf,
                orig_mode,
                reload as ::core::ffi::c_uint
                    == RELOAD_DETECT as ::core::ffi::c_int as ::core::ffi::c_uint,
            );
            if bufref_valid(&raw mut bufref) as ::core::ffi::c_int != 0
                && (*buf).b_p_udf != 0
                && !(*buf).b_ffname.is_null()
            {
                let mut hash: [uint8_t; 32] = [0; 32];
                u_compute_hash(buf, &raw mut hash as *mut uint8_t);
                u_write_undo(
                    ::core::ptr::null::<::core::ffi::c_char>(),
                    false_0 != 0,
                    buf,
                    &raw mut hash as *mut uint8_t,
                );
            }
        }
        if bufref_valid(&raw mut bufref) as ::core::ffi::c_int != 0
            && retval != 0 as ::core::ffi::c_int
        {
            apply_autocmds(
                EVENT_FILECHANGEDSHELLPOST,
                (*buf).b_fname,
                (*buf).b_fname,
                false_0 != 0,
                buf,
            );
        }
        return retval;
    }
}

pub unsafe extern "C" fn buf_reload(
    mut buf: *mut buf_T,
    mut orig_mode: ::core::ffi::c_int,
    mut reload_options: bool,
) {
    unsafe {
        let mut ea: exarg_T = exarg_T {
            arg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            args: ::core::ptr::null_mut::<*mut ::core::ffi::c_char>(),
            arglens: ::core::ptr::null_mut::<size_t>(),
            argc: 0,
            nextcmd: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            cmd: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            cmdlinep: ::core::ptr::null_mut::<*mut ::core::ffi::c_char>(),
            cmdline_tofree: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            cmdidx: CMD_append,
            argt: 0,
            skip: 0,
            forceit: 0,
            addr_count: 0,
            line1: 0,
            line2: 0,
            addr_type: ADDR_LINES,
            flags: 0,
            do_ecmd_cmd: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            do_ecmd_lnum: 0,
            append: 0,
            usefilter: 0,
            amount: 0,
            regname: 0,
            force_bin: 0,
            read_edit: 0,
            mkdir_p: 0,
            force_ff: 0,
            force_enc: 0,
            bad_char: 0,
            useridx: 0,
            errmsg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            ea_getline: None,
            cookie: ::core::ptr::null_mut::<::core::ffi::c_void>(),
            cstack: ::core::ptr::null_mut::<cstack_T>(),
        };
        let mut old_ro: ::core::ffi::c_int = (*buf).b_p_ro;
        let mut savebuf: *mut buf_T = ::core::ptr::null_mut::<buf_T>();
        let mut bufref: bufref_T = bufref_T {
            br_buf: ::core::ptr::null_mut::<buf_T>(),
            br_fnum: 0,
            br_buf_free_count: 0,
        };
        let mut saved: ::core::ffi::c_int = OK;
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
        let mut flags: ::core::ffi::c_int = READ_NEW as ::core::ffi::c_int;
        aucmd_prepbuf(&raw mut aco, buf);
        if reload_options {
            memset(
                &raw mut ea as *mut ::core::ffi::c_void,
                0 as ::core::ffi::c_int,
                ::core::mem::size_of::<exarg_T>(),
            );
        } else {
            prep_exarg(&raw mut ea, buf);
        }
        let mut old_cursor: pos_T = (*curwin.get()).w_cursor;
        let mut old_topline: linenr_T = (*curwin.get()).w_topline;
        if p_ur.get() < 0 as OptInt || (*curbuf.get()).b_ml.ml_line_count as OptInt <= p_ur.get() {
            u_sync(false_0 != 0);
            saved = u_savecommon(
                curbuf.get(),
                0 as linenr_T,
                (*curbuf.get()).b_ml.ml_line_count + 1 as linenr_T,
                0 as linenr_T,
                true_0 != 0,
            );
            flags |= READ_KEEP_UNDO as ::core::ffi::c_int;
        }
        if buf_is_empty(curbuf.get()) as ::core::ffi::c_int != 0 || saved == FAIL {
            savebuf = ::core::ptr::null_mut::<buf_T>();
        } else {
            savebuf = buflist_new(
                ::core::ptr::null_mut::<::core::ffi::c_char>(),
                ::core::ptr::null_mut::<::core::ffi::c_char>(),
                1 as linenr_T,
                BLN_DUMMY as ::core::ffi::c_int,
            );
            set_bufref(&raw mut bufref, savebuf);
            if !savebuf.is_null() && buf == curbuf.get() {
                curbuf.set(savebuf);
                (*curwin.get()).w_buffer = savebuf;
                saved = ml_open(curbuf.get());
                curbuf.set(buf);
                (*curwin.get()).w_buffer = buf;
            }
            if savebuf.is_null()
                || saved == FAIL
                || buf != curbuf.get()
                || move_lines(buf, savebuf) == FAIL
            {
                semsg(
                    gettext(b"E462: Could not prepare for reloading \"%s\"\0".as_ptr()
                        as *const ::core::ffi::c_char),
                    (*buf).b_fname,
                );
                saved = FAIL;
            }
        }
        if saved == OK {
            (*curbuf.get()).b_flags |= BF_CHECK_RO;
            (*curbuf.get()).b_keep_filetype = true_0 != 0;
            if readfile(
                (*buf).b_ffname,
                (*buf).b_fname,
                0 as linenr_T,
                0 as linenr_T,
                MAXLNUM as ::core::ffi::c_int as linenr_T,
                &raw mut ea,
                flags,
                shortmess(SHM_FILEINFO as ::core::ffi::c_int),
            ) != OK
            {
                if !aborting() {
                    semsg(
                        gettext(b"E321: Could not reload \"%s\"\0".as_ptr()
                            as *const ::core::ffi::c_char),
                        (*buf).b_fname,
                    );
                }
                if !savebuf.is_null()
                    && bufref_valid(&raw mut bufref) as ::core::ffi::c_int != 0
                    && buf == curbuf.get()
                {
                    while !buf_is_empty(curbuf.get()) {
                        if ml_delete((*buf).b_ml.ml_line_count) == FAIL {
                            break;
                        }
                    }
                    move_lines(savebuf, buf);
                }
            } else if buf == curbuf.get() {
                unchanged(buf, true_0 != 0, true_0 != 0);
                if flags & READ_KEEP_UNDO as ::core::ffi::c_int == 0 as ::core::ffi::c_int {
                    u_clearallandblockfree(buf);
                } else {
                    u_unchanged(curbuf.get());
                }
                buf_updates_unload(curbuf.get(), true_0 != 0);
                (*curbuf.get()).b_mod_set = true_0 != 0;
            }
        }
        xfree(ea.cmd as *mut ::core::ffi::c_void);
        if !savebuf.is_null() && bufref_valid(&raw mut bufref) as ::core::ffi::c_int != 0 {
            wipe_buffer(savebuf, false_0 != 0);
        }
        diff_invalidate(curbuf.get());
        (*curwin.get()).w_topline = if old_topline < (*curbuf.get()).b_ml.ml_line_count {
            old_topline
        } else {
            (*curbuf.get()).b_ml.ml_line_count
        };
        (*curwin.get()).w_cursor = old_cursor;
        check_cursor(curwin.get());
        update_topline(curwin.get());
        (*curbuf.get()).b_keep_filetype = false_0 != 0;
        let mut tp: *mut tabpage_T = first_tabpage.get() as *mut tabpage_T;
        while !tp.is_null() {
            let mut wp: *mut win_T = if tp == curtab.get() {
                firstwin.get()
            } else {
                (*tp).tp_firstwin
            };
            while !wp.is_null() {
                if (*wp).w_buffer == (*curwin.get()).w_buffer && !foldmethodIsManual(wp) {
                    foldUpdateAll(wp);
                }
                wp = (*wp).w_next;
            }
            tp = (*tp).tp_next as *mut tabpage_T;
        }
        if orig_mode == (*curbuf.get()).b_orig_mode {
            (*curbuf.get()).b_p_ro |= old_ro;
        }
        do_modelines(0 as ::core::ffi::c_int);
        aucmd_restbuf(&raw mut aco);
    }
}

pub unsafe extern "C" fn buf_store_file_info(mut buf: *mut buf_T, mut file_info: *mut FileInfo) {
    unsafe {
        (*buf).b_mtime = (*file_info).stat.st_mtim.tv_sec as int64_t;
        (*buf).b_mtime_ns = (*file_info).stat.st_mtim.tv_nsec as int64_t;
        (*buf).b_orig_size = os_fileinfo_size(file_info);
        (*buf).b_orig_mode = (*file_info).stat.st_mode as ::core::ffi::c_int;
    }
}

pub unsafe extern "C" fn write_lnum_adjust(mut offset: linenr_T) {
    unsafe {
        if (*curbuf.get()).b_no_eol_lnum != 0 as linenr_T {
            (*curbuf.get()).b_no_eol_lnum += offset;
        }
    }
}
