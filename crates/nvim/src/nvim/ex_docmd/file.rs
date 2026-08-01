//! Commands that name a file or a buffer: reading, editing, finding,
//! recovering, and the buffer list.
#![deny(unsafe_op_in_unsafe_fn)]

use core::ffi::{c_char, c_int, c_void};
use core::ptr;

use crate::src::nvim::arglist::check_arg_idx;
use crate::src::nvim::buffer::{
    bt_prompt, buf_hide, goto_buffer, maketitle, otherfile, setaltfname, setfname,
};
use crate::src::nvim::change::deleted_lines_mark;
use crate::src::nvim::drawscreen::{
    UPD_NOT_VALID, UPD_VALID, redraw_all_later, redraw_curbuf_later,
};
use crate::src::nvim::ex_cmds::{do_bang, do_ecmd};
use crate::src::nvim::ex_cmds2::{check_changed, check_fname};
use crate::src::nvim::ex_docmd::cmdline::do_cmdline_cmd;
use crate::src::nvim::ex_docmd::path::findfunc_find_file;
use crate::src::nvim::ex_docmd::source::ex_errmsg;
use crate::src::nvim::ex_docmd::{
    ACTION_SHOW, ACTION_SHOW_ALL, CCGD_AW, CCGD_EXCMD, CCGD_FORCEIT, CCGD_MULTWIN, CHECK_PATH,
    CMD_badd, CMD_balt, CMD_edit, CMD_enew, CMD_new, CMD_rshada, CMD_rviminfo, CMD_split,
    CMD_sview, CMD_tabedit, CMD_tabnew, CMD_view, CMD_visual, CMD_vnew, CMD_vsplit, CMOD_KEEPALT,
    CPO_ALTREAD, DOBUF_CURRENT, DOBUF_FIRST, DOBUF_LAST, DOBUF_MOD, ECMD_ADDBUF, ECMD_ALTBUF,
    ECMD_FORCEIT, ECMD_HIDE, ECMD_OLDBUF, ECMD_ONE, FAIL, FNAME_MESS, ML_EMPTY, NUL, OK,
    ex_pressedreturn, kDirectionNotSet,
};
use crate::src::nvim::ex_eval::{aborting, enter_cleanup, leave_cleanup};
use crate::src::nvim::ex_getln::{text_or_buf_locked, ui_ext_cmdline_block_leave};
use crate::src::nvim::file_search::{find_file_in_path, vim_findfile_cleanup};
use crate::src::nvim::fileio::readfile;
use crate::src::nvim::getchar::stuffReadbuff;
use crate::src::nvim::main::{
    RedrawingDisabled, cmdmod, curbuf, curwin, e_notopen, e_trailing_arg, ex_no_reprint,
    exmode_active, global_busy, msg_scroll, need_wait_return, no_wait_return, p_awa, p_cpo,
    p_shada, pending_exmode_active, readonlymode, recoverymode,
};
use crate::src::nvim::mark::setpcmark;
use crate::src::nvim::memfile::mf_fname;
use crate::src::nvim::memline::{ml_delete, ml_get, ml_preserve, ml_recover};
use crate::src::nvim::memory::xfree;
use crate::src::nvim::message::{emsg, msg, semsg};
use crate::src::nvim::normal::normal_enter;
use crate::src::nvim::option::get_findfunc;
use crate::src::nvim::os::libc::{gettext, strlen};
use crate::src::nvim::path::path_fnamecmp;
use crate::src::nvim::pos::MAXLNUM;
use crate::src::nvim::search::{BACKWARD, FORWARD, find_pattern_in_path};
use crate::src::nvim::shada::{shada_read_everything, shada_write_file};
use crate::src::nvim::strings::vim_strchr;
use crate::src::nvim::types::ui::kUICmdline;
use crate::src::nvim::types::{cleanup_T, exarg_T, linenr_T, size_t, uint8_t, win_T};
use crate::src::nvim::ui::ui_has;
use crate::src::nvim::undo::{
    curbufIsChanged, u_compute_hash, u_read_undo, u_save, u_savedel, u_write_undo,
};
use crate::src::nvim::window::{check_can_set_curbuf_forceit, win_close, win_valid};
use crate::src::nvim::winfloat::win_float_remove;

/// Would editing `fnum`/`ffname` mean leaving the current buffer?
///
/// A buffer whose file could not be stat'ed is compared by its *short*
/// name, because the full name may have been resolved against a directory
/// that no longer exists.
pub(crate) unsafe fn is_other_file(fnum: c_int, ffname: *mut c_char) -> bool {
    unsafe {
        if fnum != 0 {
            return fnum != (*curbuf.get()).handle;
        }
        if ffname.is_null() {
            return true;
        }
        // An empty name means "this buffer", not "no buffer".
        if *ffname as c_int == NUL {
            return false;
        }
        if !(*curbuf.get()).file_id_valid
            && !(*curbuf.get()).b_sfname.is_null()
            && *(*curbuf.get()).b_sfname as c_int != NUL
        {
            return path_fnamecmp(ffname, (*curbuf.get()).b_sfname) != 0;
        }
        otherfile(ffname)
    }
}

/// `:buffer`.
pub(crate) unsafe fn ex_buffer(eap: *mut exarg_T) {
    unsafe {
        do_exbuffer(eap);
    }
}

/// `:buffer`, shared with `:pbuffer`.
pub(crate) unsafe fn do_exbuffer(eap: *mut exarg_T) {
    unsafe {
        // The buffer was already resolved from the argument by
        // `execute_cmd0`'s `EX_BUFNAME` handling, so anything left is junk.
        if *(*eap).arg != 0 {
            (*eap).errmsg = ex_errmsg(&raw const e_trailing_arg as *const c_char, (*eap).arg);
            return;
        }
        if (*eap).addr_count == 0 {
            goto_buffer(eap, DOBUF_CURRENT as c_int, FORWARD as c_int, 0);
        } else {
            goto_buffer(
                eap,
                DOBUF_FIRST as c_int,
                FORWARD as c_int,
                (*eap).line2 as c_int,
            );
        }
        run_ecmd_cmd(eap);
    }
}

/// Run the `+cmd` argument, once the buffer it applies to is current.
unsafe fn run_ecmd_cmd(eap: *mut exarg_T) {
    unsafe {
        if !(*eap).do_ecmd_cmd.is_null() {
            do_cmdline_cmd((*eap).do_ecmd_cmd);
        }
    }
}

/// `:bmodified`.
pub(crate) unsafe fn ex_bmodified(eap: *mut exarg_T) {
    unsafe {
        goto_buffer(
            eap,
            DOBUF_MOD as c_int,
            FORWARD as c_int,
            (*eap).line2 as c_int,
        );
        run_ecmd_cmd(eap);
    }
}

/// `:bnext`.
pub(crate) unsafe fn ex_bnext(eap: *mut exarg_T) {
    unsafe {
        goto_buffer(
            eap,
            DOBUF_CURRENT as c_int,
            FORWARD as c_int,
            (*eap).line2 as c_int,
        );
        run_ecmd_cmd(eap);
    }
}

/// `:bprevious` and `:bNext`.
pub(crate) unsafe fn ex_bprevious(eap: *mut exarg_T) {
    unsafe {
        goto_buffer(
            eap,
            DOBUF_CURRENT as c_int,
            BACKWARD as c_int,
            (*eap).line2 as c_int,
        );
        run_ecmd_cmd(eap);
    }
}

/// `:brewind` and `:bfirst`.
pub(crate) unsafe fn ex_brewind(eap: *mut exarg_T) {
    unsafe {
        goto_buffer(eap, DOBUF_FIRST as c_int, FORWARD as c_int, 0);
        run_ecmd_cmd(eap);
    }
}

/// `:blast`.
pub(crate) unsafe fn ex_blast(eap: *mut exarg_T) {
    unsafe {
        goto_buffer(eap, DOBUF_LAST as c_int, BACKWARD as c_int, 0);
        run_ecmd_cmd(eap);
    }
}

/// `:preserve` — flush the swap file to disk now.
pub(crate) unsafe fn ex_preserve(_eap: *mut exarg_T) {
    unsafe {
        ml_preserve(curbuf.get(), true, true);
    }
}

/// `:recover` — read the buffer back out of a swap file.
pub(crate) unsafe fn ex_recover(eap: *mut exarg_T) {
    unsafe {
        // The flag changes what the swap-file machinery does with what it
        // finds, and is read from several modules.
        recoverymode.set(true);
        let unsaved = check_changed(
            curbuf.get(),
            (if p_awa.get() != 0 {
                CCGD_AW as c_int
            } else {
                0
            }) | CCGD_MULTWIN as c_int
                | (if (*eap).forceit != 0 {
                    CCGD_FORCEIT as c_int
                } else {
                    0
                })
                | CCGD_EXCMD as c_int,
        );
        if !unsaved
            && (*(*eap).arg as c_int == NUL
                || setfname(curbuf.get(), (*eap).arg, ptr::null_mut(), true) == OK)
        {
            ml_recover(true);
        }
        recoverymode.set(false);
    }
}

/// `:find` — edit the first file of that name on 'path', or the `count`'th.
pub(crate) unsafe fn ex_find(eap: *mut exarg_T) {
    unsafe {
        if !check_can_set_curbuf_forceit((*eap).forceit) {
            return;
        }
        let count = if (*eap).addr_count > 0 {
            (*eap).line2 as c_int
        } else {
            1
        };
        let fname = if *get_findfunc() as c_int != NUL {
            findfunc_find_file((*eap).arg, strlen((*eap).arg), count)
        } else {
            find_nth_on_path((*eap).arg, (*eap).addr_count, (*eap).line2)
        };
        if fname.is_null() {
            return;
        }
        (*eap).arg = fname;
        do_exedit(eap, ptr::null_mut());
        xfree(fname as *mut c_void);
    }
}

/// The `count`'th match for `pat` on 'path'.
///
/// The search context is what makes the second and later matches cheap:
/// each `find_file_in_path(NULL, …)` resumes the walk the first one
/// started.
unsafe fn find_nth_on_path(pat: *mut c_char, addr_count: c_int, count: linenr_T) -> *mut c_char {
    unsafe {
        let mut file_to_find: *mut c_char = ptr::null_mut();
        let mut search_ctx: *mut c_char = ptr::null_mut();
        let mut fname = find_file_in_path(
            pat,
            strlen(pat),
            FNAME_MESS as c_int,
            1,
            (*curbuf.get()).b_ffname,
            &raw mut file_to_find,
            &raw mut search_ctx,
        );
        if addr_count > 0 {
            let mut n = count;
            while !fname.is_null() && {
                n -= 1;
                n > 0
            } {
                xfree(fname as *mut c_void);
                fname = find_file_in_path(
                    ptr::null_mut(),
                    0 as size_t,
                    FNAME_MESS as c_int,
                    0,
                    (*curbuf.get()).b_ffname,
                    &raw mut file_to_find,
                    &raw mut search_ctx,
                );
            }
        }
        xfree(file_to_find as *mut c_void);
        vim_findfile_cleanup(search_ctx as *mut c_void);
        fname
    }
}

/// `:edit`, `:enew`, `:view`, `:badd`, `:balt`.
pub(crate) unsafe fn ex_edit(eap: *mut exarg_T) {
    unsafe {
        let ffname = if (*eap).cmdidx as c_int == CMD_enew as c_int {
            ptr::null_mut()
        } else {
            (*eap).arg
        };
        // `:badd` and `:balt` only add to the buffer list; they never leave
        // the current buffer, so they are not asked about it.
        if (*eap).cmdidx as c_int != CMD_badd as c_int
            && (*eap).cmdidx as c_int != CMD_balt as c_int
            && is_other_file(0, ffname)
            && !check_can_set_curbuf_forceit((*eap).forceit)
        {
            return;
        }
        if bt_prompt(curbuf.get())
            && (*eap).cmdidx as c_int == CMD_edit as c_int
            && *(*eap).arg as c_int == NUL
        {
            emsg(c"cannot :edit a prompt buffer".as_ptr());
            return;
        }
        do_exedit(eap, ptr::null_mut());
    }
}

/// The shared body of every command that opens a file into a window.
///
/// `old_curwin` is the window a *split* came from, and is null for a plain
/// `:edit`. It is what tells the failure path that there is a new window
/// to close again, and what makes the alternate file be set on the window
/// left behind.
pub unsafe fn do_exedit(eap: *mut exarg_T, old_curwin: *mut win_T) {
    unsafe {
        let ea = &mut *eap;
        // `:visual` and `:view` with no argument leave Ex mode.
        if exmode_active.get()
            && (ea.cmdidx as c_int == CMD_visual as c_int
                || ea.cmdidx as c_int == CMD_view as c_int)
        {
            exmode_active.set(false);
            ex_pressedreturn.set(false);
            if ui_has(kUICmdline) {
                ui_ext_cmdline_block_leave();
            }
            if *ea.arg as c_int == NUL {
                // Inside `:global`, normal mode is entered for the rest of
                // the line and Ex mode resumes afterwards.
                if global_busy.get() != 0 {
                    if !ea.nextcmd.is_null() {
                        stuffReadbuff(ea.nextcmd);
                        ea.nextcmd = ptr::null_mut();
                    }
                    let save_rd = RedrawingDisabled.get();
                    RedrawingDisabled.set(0);
                    let save_nwr = no_wait_return.get();
                    no_wait_return.set(0);
                    need_wait_return.set(false);
                    let save_ms = msg_scroll.get();
                    msg_scroll.set(0);
                    redraw_all_later(UPD_NOT_VALID);
                    pending_exmode_active.set(true);
                    normal_enter(false, true);
                    pending_exmode_active.set(false);
                    RedrawingDisabled.set(save_rd);
                    no_wait_return.set(save_nwr);
                    msg_scroll.set(save_ms);
                }
                return;
            }
        }

        let idx = ea.cmdidx as c_int;
        if (idx == CMD_new as c_int
            || idx == CMD_tabnew as c_int
            || idx == CMD_tabedit as c_int
            || idx == CMD_vnew as c_int)
            && *ea.arg as c_int == NUL
        {
            // A new, empty buffer.
            setpcmark();
            do_ecmd(
                0,
                ptr::null_mut(),
                ptr::null_mut(),
                eap,
                ECMD_ONE as linenr_T,
                ECMD_HIDE as c_int
                    + if ea.forceit != 0 {
                        ECMD_FORCEIT as c_int
                    } else {
                        0
                    },
                if old_curwin.is_null() {
                    curwin.get()
                } else {
                    ptr::null_mut()
                },
            );
        } else if idx != CMD_split as c_int && idx != CMD_vsplit as c_int || *ea.arg as c_int != NUL
        {
            if *ea.arg as c_int != NUL && text_or_buf_locked() {
                return;
            }
            let saved_readonly = readonlymode.get();
            if idx == CMD_view as c_int || idx == CMD_sview as c_int {
                readonlymode.set(true);
            } else if idx == CMD_enew as c_int {
                readonlymode.set(false);
            }
            if idx != CMD_balt as c_int && idx != CMD_badd as c_int {
                setpcmark();
            }

            let opened = do_ecmd(
                0,
                if idx == CMD_enew as c_int {
                    ptr::null_mut()
                } else {
                    ea.arg
                },
                ptr::null_mut(),
                eap,
                ea.do_ecmd_lnum,
                (if buf_hide(curbuf.get()) {
                    ECMD_HIDE as c_int
                } else {
                    0
                }) + (if ea.forceit != 0 {
                    ECMD_FORCEIT as c_int
                } else {
                    0
                }) + (if old_curwin.is_null() {
                    0
                } else {
                    ECMD_OLDBUF as c_int
                }) + (if idx == CMD_badd as c_int {
                    ECMD_ADDBUF as c_int
                } else {
                    0
                }) + (if idx == CMD_balt as c_int {
                    ECMD_ALTBUF as c_int
                } else {
                    0
                }),
                if old_curwin.is_null() {
                    curwin.get()
                } else {
                    ptr::null_mut()
                },
            );

            if opened == FAIL {
                // The split has already happened; close it again. The
                // cleanup pair keeps an exception from the failed edit from
                // being lost while the window is closed.
                if !old_curwin.is_null() {
                    let need_hide = curbufIsChanged() && (*curbuf.get()).b_nwindows <= 1;
                    if !need_hide || buf_hide(curbuf.get()) {
                        let mut cs: cleanup_T = core::mem::zeroed();
                        enter_cleanup(&raw mut cs);
                        win_close(curwin.get(), !need_hide && !buf_hide(curbuf.get()), false);
                        leave_cleanup(&raw mut cs);
                    }
                }
            } else if readonlymode.get() && (*curbuf.get()).b_nwindows == 1 {
                (*curbuf.get()).b_p_ro = 1;
            }
            readonlymode.set(saved_readonly);
        } else {
            // A `:split` with no file name: the window is already there.
            run_ecmd_cmd(eap);
            let was_invalid = (*curwin.get()).w_arg_idx_invalid;
            check_arg_idx(curwin.get());
            if was_invalid != (*curwin.get()).w_arg_idx_invalid {
                maketitle();
            }
        }

        if !old_curwin.is_null()
            && *ea.arg as c_int != NUL
            && curwin.get() != old_curwin
            && win_valid(old_curwin)
            && (*old_curwin).w_buffer != curbuf.get()
            && (*cmdmod.ptr()).cmod_flags & CMOD_KEEPALT as c_int == 0
        {
            (*old_curwin).w_alt_fnum = (*curbuf.get()).handle as c_int;
        }
        ex_no_reprint.set(true);
    }
}

/// `:swapname`.
pub(crate) unsafe fn ex_swapname(_eap: *mut exarg_T) {
    unsafe {
        let mfp = (*curbuf.get()).b_ml.ml_mfp;
        if mfp.is_null() || mf_fname(mfp).is_null() {
            msg(gettext(c"No swap file".as_ptr()), 0);
        } else {
            msg(mf_fname(mfp), 0);
        }
    }
}

/// `:read` — insert a file, or the output of a command.
pub(crate) unsafe fn ex_read(eap: *mut exarg_T) {
    unsafe {
        let was_empty = (*curbuf.get()).b_ml.ml_flags & ML_EMPTY;
        if (*eap).usefilter != 0 {
            do_bang(1, eap, false, false, true);
            return;
        }
        if u_save((*eap).line2, (*eap).line2 + 1) == FAIL {
            return;
        }

        let read = if *(*eap).arg as c_int == NUL {
            if check_fname() == FAIL {
                return;
            }
            readfile(
                (*curbuf.get()).b_ffname,
                (*curbuf.get()).b_fname,
                (*eap).line2,
                0,
                MAXLNUM as linenr_T,
                eap,
                0,
                false,
            )
        } else {
            // 'cpoptions' `a` makes `:read file` set the alternate file.
            if !vim_strchr(p_cpo.get(), CPO_ALTREAD).is_null() {
                setaltfname((*eap).arg, (*eap).arg, 1);
            }
            readfile(
                (*eap).arg,
                ptr::null_mut(),
                (*eap).line2,
                0,
                MAXLNUM as linenr_T,
                eap,
                0,
                false,
            )
        };

        if read != OK {
            if !aborting() {
                semsg(gettext(&raw const e_notopen as *const c_char), (*eap).arg);
            }
            return;
        }
        // Reading into an empty buffer in Ex mode leaves the empty line the
        // buffer started with; drop it.
        if was_empty != 0 && exmode_active.get() {
            let lnum = if (*eap).line2 == 0 {
                (*curbuf.get()).b_ml.ml_line_count
            } else {
                1
            };
            if *ml_get(lnum) as c_int == NUL && u_savedel(lnum, 1) == OK {
                ml_delete(lnum);
                if (*curwin.get()).w_cursor.lnum > 1 && (*curwin.get()).w_cursor.lnum >= lnum {
                    (*curwin.get()).w_cursor.lnum -= 1;
                }
                deleted_lines_mark(lnum, 1);
            }
        }
        redraw_curbuf_later(UPD_VALID);
    }
}

/// `:!cmd`.
pub(crate) unsafe fn ex_bang(eap: *mut exarg_T) {
    unsafe {
        do_bang((*eap).addr_count, eap, (*eap).forceit != 0, true, true);
    }
}

/// `:wundo` — write the undo tree to a file, tagged with a hash of the
/// buffer text so that reading it back into a different buffer is refused.
pub(crate) unsafe fn ex_wundo(eap: *mut exarg_T) {
    unsafe {
        let mut hash: [uint8_t; 32] = [0; 32];
        u_compute_hash(curbuf.get(), &raw mut hash as *mut uint8_t);
        u_write_undo(
            (*eap).arg,
            (*eap).forceit != 0,
            curbuf.get(),
            &raw mut hash as *mut uint8_t,
        );
    }
}

/// `:rundo`.
pub(crate) unsafe fn ex_rundo(eap: *mut exarg_T) {
    unsafe {
        let mut hash: [uint8_t; 32] = [0; 32];
        u_compute_hash(curbuf.get(), &raw mut hash as *mut uint8_t);
        u_read_undo((*eap).arg, &raw mut hash as *mut uint8_t, ptr::null());
    }
}

/// `:checkpath` — every file 'path' reaches from the includes of this one.
pub(crate) unsafe fn ex_checkpath(eap: *mut exarg_T) {
    unsafe {
        find_pattern_in_path(
            ptr::null_mut(),
            kDirectionNotSet,
            0 as size_t,
            false,
            false,
            CHECK_PATH as c_int,
            1,
            if (*eap).forceit != 0 {
                ACTION_SHOW_ALL as c_int
            } else {
                ACTION_SHOW as c_int
            },
            1,
            MAXLNUM as linenr_T,
            (*eap).forceit != 0,
            false,
        );
    }
}

/// `:rshada`, `:wshada` and their `viminfo` spellings.
pub(crate) unsafe fn ex_shada(eap: *mut exarg_T) {
    unsafe {
        // An empty 'shada' would mean "save nothing", which is not what an
        // explicit command means.
        let save_shada = p_shada.get();
        if *p_shada.get() as c_int == NUL {
            p_shada.set(c"'100".as_ptr() as *mut c_char);
        }
        if (*eap).cmdidx as c_int == CMD_rviminfo as c_int
            || (*eap).cmdidx as c_int == CMD_rshada as c_int
        {
            shada_read_everything((*eap).arg, (*eap).forceit != 0, false);
        } else {
            shada_write_file((*eap).arg, (*eap).forceit != 0);
        }
        p_shada.set(save_shada);
    }
}

/// `:fclose` — close a floating window by its handle.
pub(crate) unsafe fn ex_fclose(eap: *mut exarg_T) {
    unsafe {
        win_float_remove((*eap).forceit != 0, (*eap).line1 as c_int);
    }
}
