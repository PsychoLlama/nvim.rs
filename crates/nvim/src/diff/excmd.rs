//! The `:diff*` commands that turn diff mode on and off.
//!
//! `:diffthis`, `:diffsplit`, `:diffoff` and `:diffpatch`, plus
//! [`diff_win_options`], which is the option set every window entering diff
//! mode takes (and, through the saved `w_p_*_save` fields, gives back on the
//! way out).
//!
//! The save/restore pair is not symmetric, and both halves say why at their
//! site: the save happens **once**, guarded by `wo_diff_saved`, so a second
//! `:diffthis` cannot record the diff-mode values as the ones to go back to;
//! and the restore only applies where the window still holds the value diff
//! mode gave it, so an option the user changed in between survives
//! `:diffoff`.

#![deny(unsafe_op_in_unsafe_fn)]

use super::*;
use crate::types::{FAIL, MAXPATHL, NUL, OK, OptionSetFlags};
use core::ffi::{c_char, c_int};

/// `:diffpatch {file}`: apply a patch to a copy of the current buffer and
/// open the result beside it.
///
/// The patch runs in the temp directory, because `patch(1)` writes its
/// `.orig` and `.rej` files next to its output and the user's directory is
/// not the place for them; the original directory is restored afterwards.
/// `'patchexpr'` replaces the shell-out entirely.
pub unsafe fn ex_diffpatch(eap: *mut exarg_T) {
    unsafe {
        let mut buflen: size_t = 0;
        let mut dirbuf: [c_char; 4096] = [0; 4096];
        let mut file_info: FileInfo = FileInfo::default();
        let mut info_ok: bool = false;
        let mut filesize: uint64_t = 0;
        let mut buf: *mut c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
        let mut old_curwin: *mut win_T = curwin.get();
        let mut newname: *mut c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
        let mut esc_name: *mut c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
        let mut fullname: *mut c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
        let mut tmp_orig: *mut c_char = vim_tempname();
        let mut tmp_new: *mut c_char = vim_tempname();
        if !(tmp_orig.is_null() || tmp_new.is_null()) {
            if buf_write(
                curbuf.get(),
                tmp_orig,
                ::core::ptr::null_mut::<::core::ffi::c_char>(),
                1 as linenr_T,
                (*curbuf.get()).b_ml.ml_line_count,
                ::core::ptr::null_mut(),
                WriteRequest::filter(),
            ) != FAIL
            {
                fullname = FullName_save((*eap).arg, false);
                esc_name = vim_strsave_shellescape(
                    if !fullname.is_null() {
                        fullname
                    } else {
                        (*eap).arg
                    },
                    true,
                    true,
                );
                buflen = strlen(tmp_orig)
                    .wrapping_add(strlen(esc_name))
                    .wrapping_add(strlen(tmp_new))
                    .wrapping_add(16 as size_t);
                buf = xmalloc(buflen) as *mut c_char;
                dirbuf = [0; 4096];
                if os_dirname(&raw mut dirbuf as *mut c_char, MAXPATHL as size_t) != OK
                    || os_chdir(&raw mut dirbuf as *mut c_char) != 0
                {
                    dirbuf[0] = NUL as c_char;
                } else {
                    let mut tempdir: *mut c_char = vim_gettempdir();
                    if tempdir.is_null() {
                        tempdir = c"/tmp".as_ptr() as *mut c_char;
                    }
                    os_chdir(tempdir);
                    shorten_fnames(true_0);
                }
                if *p_pex.get() as c_int != NUL {
                    eval_patch(
                        tmp_orig,
                        if !fullname.is_null() {
                            fullname
                        } else {
                            (*eap).arg
                        },
                        tmp_new,
                    );
                } else {
                    vim_snprintf(
                        buf,
                        buflen,
                        c"patch -o %s %s < %s".as_ptr(),
                        tmp_new,
                        tmp_orig,
                        esc_name,
                    );
                    block_autocmds();
                    call_shell(
                        buf,
                        ShellOpts::FILTER,
                        ::core::ptr::null_mut::<::core::ffi::c_char>(),
                    );
                    unblock_autocmds();
                }
                if dirbuf[0] as c_int != NUL {
                    if os_chdir(&raw mut dirbuf as *mut c_char) != 0 {
                        emsg(gettext(&raw const e_prev_dir as *const c_char));
                    }
                    shorten_fnames(true_0);
                }
                strcpy(buf, tmp_new);
                strcat(buf, c".orig".as_ptr());
                os_remove(buf);
                strcpy(buf, tmp_new);
                strcat(buf, c".rej".as_ptr());
                os_remove(buf);
                file_info = FileInfo::default();
                info_ok = os_fileinfo(tmp_new, &raw mut file_info);
                filesize = os_fileinfo_size(&raw mut file_info);
                if !info_ok || filesize == 0 as uint64_t {
                    emsg(gettext(c"E816: Cannot read patch output".as_ptr()));
                } else {
                    if !(*curbuf.get()).b_fname.is_null() {
                        newname = xstrnsave(
                            (*curbuf.get()).b_fname,
                            strlen((*curbuf.get()).b_fname).wrapping_add(4 as size_t),
                        );
                        strcat(newname, c".new".as_ptr());
                    }
                    (*cmdmod.ptr()).cmod_tab = 0;
                    if win_split(
                        0,
                        if diff_flags.get() & DIFF_VERTICAL != 0 {
                            WSP_VERT as c_int
                        } else {
                            0
                        },
                    ) != FAIL
                    {
                        (*eap).cmdidx = CMD_split;
                        (*eap).arg = tmp_new;
                        do_exedit(eap, old_curwin);
                        if curwin.get() != old_curwin && win_valid(old_curwin) {
                            diff_win_options(curwin.get(), true);
                            diff_win_options(old_curwin, true);
                            if !newname.is_null() {
                                (*eap).arg = newname;
                                ex_file(eap);
                                if augroup_exists(c"filetypedetect".as_ptr()) {
                                    do_cmdline_cmd(c":doau filetypedetect BufRead".as_ptr());
                                }
                            }
                        }
                    }
                }
            }
        }
        if !tmp_orig.is_null() {
            os_remove(tmp_orig);
        }
        xfree(tmp_orig as *mut ::core::ffi::c_void);
        if !tmp_new.is_null() {
            os_remove(tmp_new);
        }
        xfree(tmp_new as *mut ::core::ffi::c_void);
        xfree(newname as *mut ::core::ffi::c_void);
        xfree(buf as *mut ::core::ffi::c_void);
        xfree(fullname as *mut ::core::ffi::c_void);
        xfree(esc_name as *mut ::core::ffi::c_void);
    }
}

/// `:diffsplit {file}`: open `file` in a new window and diff it against the
/// current buffer.
pub unsafe fn ex_diffsplit(eap: *mut exarg_T) {
    unsafe {
        let mut old_curwin: *mut win_T = curwin.get();
        let mut old_curbuf: bufref_T = bufref_T::default();
        set_bufref(&raw mut old_curbuf, curbuf.get());
        validate_cursor(curwin.get());
        set_fraction(curwin.get());
        (*cmdmod.ptr()).cmod_tab = 0;
        if win_split(
            0,
            if diff_flags.get() & DIFF_VERTICAL != 0 {
                WSP_VERT as c_int
            } else {
                0
            },
        ) == FAIL
        {
            return;
        }
        (*eap).cmdidx = CMD_split;
        (*curwin.get()).w_onebuf_opt.wo_diff = true_0;
        do_exedit(eap, old_curwin);
        if curwin.get() == old_curwin {
            return;
        }
        diff_win_options(curwin.get(), true);
        if win_valid(old_curwin) {
            diff_win_options(old_curwin, true);
            if bufref_valid(&raw mut old_curbuf) {
                (*curwin.get()).w_cursor.lnum =
                    diff_get_corresponding_line(old_curbuf.br_buf, (*old_curwin).w_cursor.lnum);
            }
        }
        scroll_to_fraction(curwin.get(), (*curwin.get()).w_height);
    }
}

/// `:diffthis`: put the current window in diff mode.
pub unsafe fn ex_diffthis(_eap: *mut exarg_T) {
    unsafe {
        diff_win_options(curwin.get(), true);
    }
}

/// Set `'diff'` in `wp` without letting the option's side effects run.
///
/// `curwin` is moved to `wp` for the call because the option code reads it,
/// and `diff_buf_adjust` is suppressed so that the caller stays in charge of
/// the buffer registry.
unsafe fn set_diff_option(wp: *mut win_T, value: bool) {
    unsafe {
        let mut old_curwin: *mut win_T = curwin.get();
        curwin.set(wp);
        curbuf.set((*curwin.get()).w_buffer);
        (*curbuf.get()).b_ro_locked += 1;
        set_option_value_give_err(
            kOptDiff,
            OptVal {
                type_0: kOptValTypeBoolean,
                data: OptValData {
                    boolean: value as TriState,
                },
            },
            OptionSetFlags::LOCAL,
        );
        (*curbuf.get()).b_ro_locked -= 1;
        curwin.set(old_curwin);
        curbuf.set((*curwin.get()).w_buffer);
    }
}

/// Put `wp` into diff mode: the option set, and optionally its buffer.
///
/// Every option this changes is saved into the matching `w_p_*_save` field
/// first, but **only on the first call** -- `wo_diff_saved` is what stops a
/// second `:diffthis` from saving the diff-mode values as the ones to
/// restore.
pub unsafe fn diff_win_options(wp: *mut win_T, addbuf: bool) {
    unsafe {
        let mut old_curwin: *mut win_T = curwin.get();
        curwin.set(wp);
        newFoldLevel();
        curwin.set(old_curwin);
        if (*wp).w_onebuf_opt.wo_diff == 0 {
            (*wp).w_onebuf_opt.wo_scb_save = (*wp).w_onebuf_opt.wo_scb;
        }
        (*wp).w_onebuf_opt.wo_scb = true_0;
        if (*wp).w_onebuf_opt.wo_diff == 0 {
            (*wp).w_onebuf_opt.wo_crb_save = (*wp).w_onebuf_opt.wo_crb;
        }
        (*wp).w_onebuf_opt.wo_crb = true_0;
        if diff_flags.get() & DIFF_FOLLOWWRAP == 0 {
            if (*wp).w_onebuf_opt.wo_diff == 0 {
                (*wp).w_onebuf_opt.wo_wrap_save = (*wp).w_onebuf_opt.wo_wrap;
            }
            (*wp).w_onebuf_opt.wo_wrap = false_0;
            (*wp).w_skipcol = 0 as colnr_T;
        }
        if (*wp).w_onebuf_opt.wo_diff == 0 {
            if (*wp).w_onebuf_opt.wo_diff_saved != 0 {
                free_string_option((*wp).w_onebuf_opt.wo_fdm_save);
            }
            (*wp).w_onebuf_opt.wo_fdm_save = xstrdup((*wp).w_onebuf_opt.wo_fdm);
        }
        set_option_direct_for(
            kOptFoldmethod,
            OptVal {
                type_0: kOptValTypeString,
                data: OptValData {
                    string: String_0 {
                        data: c"diff".as_ptr() as *mut c_char,
                        size: ::core::mem::size_of::<[c_char; 5]>().wrapping_sub(1 as size_t),
                    },
                },
            },
            OptionSetFlags::LOCAL,
            0 as scid_T,
            kOptScopeWin,
            wp as *mut ::core::ffi::c_void,
        );
        if (*wp).w_onebuf_opt.wo_diff == 0 {
            (*wp).w_onebuf_opt.wo_fen_save = (*wp).w_onebuf_opt.wo_fen;
            (*wp).w_onebuf_opt.wo_fdl_save = (*wp).w_onebuf_opt.wo_fdl;
            if (*wp).w_onebuf_opt.wo_diff_saved != 0 {
                free_string_option((*wp).w_onebuf_opt.wo_fdc_save);
            }
            (*wp).w_onebuf_opt.wo_fdc_save = xstrdup((*wp).w_onebuf_opt.wo_fdc);
        }
        free_string_option((*wp).w_onebuf_opt.wo_fdc);
        (*wp).w_onebuf_opt.wo_fdc = xstrdup(c"2".as_ptr());
        // A single digit, because the option's buffer is one byte plus the
        // NUL. C's `assert()` is `debug_assert!`: it vanishes under NDEBUG.
        debug_assert!((0..=9).contains(&diff_foldcolumn.get()));
        snprintf(
            (*wp).w_onebuf_opt.wo_fdc,
            strlen((*wp).w_onebuf_opt.wo_fdc).wrapping_add(1 as size_t),
            c"%d".as_ptr(),
            diff_foldcolumn.get(),
        );
        (*wp).w_onebuf_opt.wo_fen = true_0;
        (*wp).w_onebuf_opt.wo_fdl = 0 as OptInt;
        foldUpdateAll(wp);
        changed_window_setting(wp);
        if vim_strchr(p_sbo.get(), 'h' as c_int).is_null() {
            do_cmdline_cmd(c"set sbo+=hor".as_ptr());
        }
        (*wp).w_onebuf_opt.wo_diff_saved = true_0;
        set_diff_option(wp, true);
        if addbuf {
            diff_buf_add((*wp).w_buffer);
        }
        redraw_later(wp, UPD_NOT_VALID);
    }
}

/// `:diffoff[!]`: leave diff mode in this window, or with `!` in every
/// window of the tabpage.
///
/// Each option goes back to its `w_p_*_save` value, but only where the
/// window still holds the value diff mode gave it: a value the user changed
/// in the meantime is left alone.
pub unsafe fn ex_diffoff(eap: *mut exarg_T) {
    unsafe {
        let mut diffwin: bool = false;
        let mut wp: *mut win_T = if curtab.get() == curtab.get() {
            firstwin.get()
        } else {
            (*curtab.get()).tp_firstwin
        };
        while !wp.is_null() {
            if if (*eap).forceit != 0 {
                (*wp).w_onebuf_opt.wo_diff
            } else {
                (wp == curwin.get()) as c_int
            } != 0
            {
                set_diff_option(wp, false);
                if (*wp).w_onebuf_opt.wo_diff_saved != 0 {
                    if (*wp).w_onebuf_opt.wo_scb != 0 {
                        (*wp).w_onebuf_opt.wo_scb = (*wp).w_onebuf_opt.wo_scb_save;
                    }
                    if (*wp).w_onebuf_opt.wo_crb != 0 {
                        (*wp).w_onebuf_opt.wo_crb = (*wp).w_onebuf_opt.wo_crb_save;
                    }
                    if diff_flags.get() & DIFF_FOLLOWWRAP == 0 {
                        if (*wp).w_onebuf_opt.wo_wrap == 0 && (*wp).w_onebuf_opt.wo_wrap_save != 0 {
                            (*wp).w_onebuf_opt.wo_wrap = true_0;
                            (*wp).w_leftcol = 0 as colnr_T;
                        }
                    }
                    free_string_option((*wp).w_onebuf_opt.wo_fdm);
                    (*wp).w_onebuf_opt.wo_fdm =
                        xstrdup(if *(*wp).w_onebuf_opt.wo_fdm_save as c_int != 0 {
                            (*wp).w_onebuf_opt.wo_fdm_save as *const c_char
                        } else {
                            c"manual".as_ptr()
                        });
                    free_string_option((*wp).w_onebuf_opt.wo_fdc);
                    (*wp).w_onebuf_opt.wo_fdc =
                        xstrdup(if *(*wp).w_onebuf_opt.wo_fdc_save as c_int != 0 {
                            (*wp).w_onebuf_opt.wo_fdc_save as *const c_char
                        } else {
                            c"0".as_ptr()
                        });
                    if (*wp).w_onebuf_opt.wo_fdl == 0 as OptInt {
                        (*wp).w_onebuf_opt.wo_fdl = (*wp).w_onebuf_opt.wo_fdl_save;
                    }
                    if (*wp).w_onebuf_opt.wo_fen != 0 {
                        (*wp).w_onebuf_opt.wo_fen = if foldmethodIsManual(wp) as c_int != 0 {
                            false_0
                        } else {
                            (*wp).w_onebuf_opt.wo_fen_save
                        };
                    }
                    foldUpdateAll(wp);
                }
                (*wp).w_topfill = 0;
                changed_window_setting(wp);
                diff_buf_adjust(wp);
            }
            diffwin = diffwin as c_int | (*wp).w_onebuf_opt.wo_diff != 0;
            wp = (*wp).w_next;
        }
        if (*eap).forceit != 0 {
            diff_buf_clear();
        }
        if !diffwin {
            diff_need_update.set(false);
            (*curtab.get()).tp_diff_invalid = false_0;
            (*curtab.get()).tp_diff_update = false_0;
            diff_clear(curtab.get());
        }
        if !diffwin && !vim_strchr(p_sbo.get(), 'h' as c_int).is_null() {
            do_cmdline_cmd(c"set sbo-=hor".as_ptr());
        }
    }
}
