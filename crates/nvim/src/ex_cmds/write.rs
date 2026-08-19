//! Putting the buffer on disk -- `:write`, `:update`, `:wall`, `:wq` and the
//! checks that guard them.
//!
//! [`do_write`] is the entry point every `:w` form funnels into; the risk it
//! manages is not the writing (that is `bufwrite.rs`) but *which file* and
//! *whether we may*: [`check_overwrite`] refuses an existing other file without
//! `!`, `check_readonly` handles 'readonly' and a read-only file mode, and
//! `check_writable`/`not_writing` cover 'write' and `:noautocmd`.  [`ex_file`]
//! is `:file`, which renames the buffer, and [`getfile`] is the shared "switch
//! to this file, writing or abandoning the current one first" helper that
//! `:tag` and friends call.
//!
//! Original: `src/nvim/ex_cmds.c`, Vim/Neovim, Vim license.

#![deny(unsafe_op_in_unsafe_fn)]

use super::{
    BF_NEW, BF_NOTEDITED, BF_READERR, BL_FIX, BL_SOL, CPO_ALTWRITE, CPO_OVERNEW, ECMD_FORCEIT,
    ECMD_HIDE, FAIL, GETFILE_ERROR, GETFILE_NOT_WRITTEN, GETFILE_OPEN_OTHER, GETFILE_SAME_FILE,
    MAXPATHL, NODE_OTHER, SHM_FILEINFO, VIM_QUESTION, VIM_YES, buf_autocmd, do_bang, do_ecmd,
    false_0, true_0,
};
use crate::arglist::do_argfile;
use crate::autocmd::{
    EVENT_BUFADD, EVENT_BUFFILEPOST, EVENT_BUFFILEPRE, augroup_exists, do_doautocmd,
};
use crate::buffer::{
    bt_dontwrite, bt_dontwrite_msg, bt_nofilename, buf_hide, buf_name_changed, buflist_findname,
    buflist_new, bufref_valid, do_autochdir, do_modelines, fileinfo, fname_expand,
    no_write_message, no_write_message_buf, otherfile, set_bufref, setaltfname, setfname,
};
use crate::bufwrite::{WriteRequest, buf_write};
use crate::channel::channel_job_running;
use crate::cursor::check_cursor_lnum;
use crate::edit::beginline;
use crate::ex_cmds2::{autowrite, buf_write_all, check_fname, dialog_changed};
use crate::ex_docmd::{before_quit_all, dialog_msg, not_exiting};
use crate::ex_eval::aborting;
use crate::ex_getln::{curbuf_locked, text_locked};
use crate::main::{
    cmdmod, curbuf, curwin, e_argreq, e_bufloaded, e_exists, e_invarg, e_isadir2, e_readonly,
    emsg_silent, exiting, firstbuf, getout, no_wait_return, p_confirm, p_cpo, p_dir, p_wa, p_write,
    redraw_tabline,
};
use crate::mark::setpcmark;
use crate::memline::makeswapname;
use crate::memory::{xfree, xmalloc};
use crate::message::{emsg, vim_dialog_yesno};
use crate::option::{copy_option_part, shortmess};
use crate::os::cshim::gettext;
use crate::os::fs::{os_file_is_writable, os_file_mkdir, os_isdir, os_nodetype, os_path_exists};
use crate::path::fix_fname;
use crate::semsg_c;
use crate::strings::vim_strchr;
use crate::types::{
    CMD_saveas, CMD_wqall, CMD_xall, CMOD_CONFIRM, CMOD_KEEPALT, NUL, OK, buf_T, bufref_T, exarg_T,
    int32_t, int64_t, linenr_T,
};
use crate::undo::{bufIsChanged, curbufIsChanged};
use crate::window::check_can_set_curbuf_forceit;
use ::libc::strcpy;
use core::ffi::{c_char, c_int};
use core::ptr;

/// The buffer `dialog_msg` formats a prompt into.
const DIALOG_MSG_SIZE: usize = 1000;

/// An `xmalloc`ed C string this module owns, freed when it goes out of scope.
/// A NULL is allowed and frees nothing, so it stands in for the `char *x =
/// NULL; ... xfree(x)` shape upstream uses around a conditional allocation.
struct Owned(*mut c_char);

impl Drop for Owned {
    fn drop(&mut self) {
        // SAFETY: our own allocation, or NULL.
        unsafe { xfree(self.0.cast()) };
    }
}

/// Is a `:confirm` dialog wanted here -- either from 'confirm' or from the
/// command's own modifier?
fn confirming() -> bool {
    p_confirm.get() != 0 || cmdmod.with(|mods| mods.cmod_flags) & CMOD_CONFIRM as c_int != 0
}

/// Put `name` into the one-`%s` message `fmt` and ask the user to confirm it.
///
/// # Safety
/// `fmt` must be a format taking exactly one string, and `name` must be live.
unsafe fn dialog_yesno_about(fmt: *mut c_char, name: *mut c_char) -> bool {
    let mut buff: [c_char; DIALOG_MSG_SIZE] = [0; DIALOG_MSG_SIZE];
    // SAFETY: caller's contract; `buff` is the `DIALOG_MSG_SIZE` upstream
    // sizes its own prompt buffers to.
    unsafe {
        dialog_msg(buff.as_mut_ptr(), fmt, name);
        vim_dialog_yesno(VIM_QUESTION as c_int, ptr::null_mut(), buff.as_mut_ptr(), 2)
            == VIM_YES as c_int
    }
}

/// Give the current buffer the name `new_fname`, moving the old name into a
/// new unlisted buffer so that it becomes the alternate file.
///
/// # Safety
/// `new_fname` must be a live file name.
pub unsafe fn rename_buffer(new_fname: *mut c_char) -> c_int {
    let buf = curbuf.get();
    // SAFETY: the autocommand runs with the current buffer.
    unsafe {
        buf_autocmd(EVENT_BUFFILEPRE, curbuf.get());
    }
    // buffer changed, don't change name now
    if buf != curbuf.get() {
        return FAIL;
    }
    if aborting() {
        // autocmds may abort script processing
        return FAIL;
    }

    // The name of the current buffer will be changed.
    // A new (unlisted) buffer entry needs to be made to hold the old file
    // name, which will become the alternate file name.  But don't set the
    // alternate file name if the buffer didn't have a name.
    // SAFETY: `curbuf` is live and owns the three names.
    let (fname, sfname, xfname) = unsafe {
        let names = (
            (*curbuf.get()).b_ffname,
            (*curbuf.get()).b_sfname,
            (*curbuf.get()).b_fname,
        );
        (*curbuf.get()).b_ffname = ptr::null_mut();
        (*curbuf.get()).b_sfname = ptr::null_mut();
        names
    };
    // SAFETY: caller's contract; the names are handed back on failure.
    unsafe {
        if setfname(curbuf.get(), new_fname, ptr::null_mut(), true) == FAIL {
            (*curbuf.get()).b_ffname = fname;
            (*curbuf.get()).b_sfname = sfname;
            return FAIL;
        }
        (*curbuf.get()).b_flags |= BF_NOTEDITED;
        if !xfname.is_null() && *xfname as c_int != NUL {
            let alt = buflist_new(fname, xfname, (*curwin.get()).w_cursor.lnum, 0);
            if !alt.is_null() && cmdmod.with(|mods| mods.cmod_flags) & CMOD_KEEPALT as c_int == 0 {
                (*curwin.get()).w_alt_fnum = (*alt).handle as c_int;
            }
        }
        xfree(fname.cast());
        xfree(sfname.cast());
        buf_autocmd(EVENT_BUFFILEPOST, curbuf.get());
        // Change directories when the 'acd' option is set.
        do_autochdir();
    }
    OK
}

/// `:file[!] [fname]`.
///
/// # Safety
/// `eap` must be the live Ex-command argument.
pub unsafe fn ex_file(eap: *mut exarg_T) {
    // SAFETY: caller's contract.
    unsafe {
        // ":0file" removes the file name.  Check for illegal uses ":3file",
        // "0file name", etc.
        if (*eap).addr_count > 0
            && (*(*eap).arg as c_int != NUL || (*eap).line2 > 0 || (*eap).addr_count > 1)
        {
            emsg(gettext(&raw const e_invarg as *const c_char));
            return;
        }

        if *(*eap).arg as c_int != NUL || (*eap).addr_count == 1 {
            if rename_buffer((*eap).arg) == FAIL {
                return;
            }
            redraw_tabline.set(true);
        }

        // print file name if no argument or 'F' is not in 'shortmess'
        if *(*eap).arg as c_int == NUL || !shortmess(SHM_FILEINFO as c_int) {
            fileinfo(false_0, false_0, (*eap).forceit != 0);
        }
    }
}

/// `:update` -- write only when there is something to write.
///
/// # Safety
/// `eap` must be the live Ex-command argument.
pub unsafe fn ex_update(eap: *mut exarg_T) {
    // SAFETY: caller's contract; `curbuf` is live.
    unsafe {
        if curbufIsChanged()
            || (!bt_nofilename(curbuf.get())
                && !(*curbuf.get()).b_ffname.is_null()
                && !os_path_exists((*curbuf.get()).b_ffname))
        {
            do_write(eap);
        }
    }
}

/// `:write` and `:saveas`.
///
/// # Safety
/// `eap` must be the live Ex-command argument.
pub unsafe fn ex_write(eap: *mut exarg_T) {
    // SAFETY: caller's contract.
    unsafe {
        if (*eap).cmdidx == CMD_saveas {
            // :saveas does not take a range, uses all lines.
            (*eap).line1 = 1;
            (*eap).line2 = (*curbuf.get()).b_ml.ml_line_count;
        }

        if (*eap).usefilter != 0 {
            // input lines to shell command
            do_bang(1, eap, false, true, false);
        } else {
            do_write(eap);
        }
    }
}

/// Refuse a device or a socket: only a regular file, or something that can be
/// written like one, may be a write target.
///
/// # Safety
/// `fname` must be live, or NULL.
unsafe fn check_writable(fname: *const c_char) -> c_int {
    // SAFETY: caller's contract; one `%s` for one string.
    unsafe {
        if os_nodetype(fname) == NODE_OTHER {
            semsg_c!(
                gettext(c"E503: \"%s\" is not a file or writable device".as_ptr()),
                fname,
            );
            return FAIL;
        }
    }
    OK
}

/// `:write ++p` -- create the missing leading directories.
///
/// # Safety
/// `eap` and `fname` must be live.
unsafe fn handle_mkdir_p_arg(eap: *mut exarg_T, fname: *mut c_char) -> c_int {
    // SAFETY: caller's contract.
    if unsafe { (*eap).mkdir_p } != 0 && unsafe { os_file_mkdir(fname, 0o755 as int32_t) } < 0 {
        return FAIL;
    }
    OK
}

/// Write the current buffer to the file `eap->arg` names, or to its own file
/// when that argument is empty.  `eap->append` appends instead of replacing.
///
/// Returns `FAIL` for failure, `OK` otherwise.
///
/// # Safety
/// `eap` must be the live Ex-command argument.
pub unsafe fn do_write(eap: *mut exarg_T) -> c_int {
    // check 'write' option
    if unsafe { not_writing() } {
        return FAIL;
    }

    let mut fname = ptr::null_mut(); // init to shut up gcc
    // SAFETY: caller's contract.
    let mut ffname = unsafe { (*eap).arg };
    // When out-of-memory, keep the unexpanded file name, because we MUST be
    // able to write the file in this situation.
    let mut free_fname = Owned(ptr::null_mut());
    let other;
    // SAFETY: `ffname` is the command's NUL-terminated argument.
    if unsafe { *ffname } as c_int == NUL {
        if unsafe { (*eap).cmdidx } == CMD_saveas {
            // SAFETY: a live message string.
            unsafe { emsg(gettext(&raw const e_argreq as *const c_char)) };
            return FAIL;
        }
        other = false;
    } else {
        fname = ffname;
        // SAFETY: as above.
        free_fname = Owned(unsafe { fix_fname(ffname) });
        if !free_fname.0.is_null() {
            ffname = free_fname.0;
        }
        // SAFETY: as above.
        other = unsafe { otherfile(ffname) };
    }

    // If we have a new file, put its name in the list of alternate file names.
    let mut alt_buf = ptr::null_mut();
    if other {
        // SAFETY: the names are live and 'cpoptions' is a live option string.
        alt_buf = unsafe {
            if !vim_strchr(p_cpo.get(), CPO_ALTWRITE).is_null() || (*eap).cmdidx == CMD_saveas {
                setaltfname(ffname, fname, 1)
            } else {
                buflist_findname(ffname)
            }
        };
        // Overwriting a file that is loaded in another buffer is not a good
        // idea.
        // SAFETY: `alt_buf` is a live buffer when non-NULL.
        if !alt_buf.is_null() && !unsafe { (*alt_buf).b_ml.ml_mfp }.is_null() {
            // SAFETY: a live message string.
            unsafe { emsg(gettext(&raw const e_bufloaded as *const c_char)) };
            return FAIL;
        }
    }

    if !other {
        // SAFETY: `eap` is live and `curbuf` is the current buffer.
        if unsafe { cannot_write_curbuf(eap) } {
            return FAIL;
        }
        // SAFETY: `curbuf` is live.
        (ffname, fname) = unsafe { ((*curbuf.get()).b_ffname, (*curbuf.get()).b_fname) };
        // SAFETY: `eap` is live.
        if !unsafe { confirm_partial_write(eap) } {
            return FAIL;
        }
    }

    // SAFETY: the names are live and `eap` is the caller's.
    if unsafe { check_overwrite(eap, curbuf.get(), fname, ffname, other) } != OK {
        return FAIL;
    }

    if unsafe { (*eap).cmdidx } == CMD_saveas && !alt_buf.is_null() {
        // SAFETY: `alt_buf` is a live, unloaded buffer.
        match unsafe { saveas_exchange_names(alt_buf) } {
            Some(sfname) => fname = sfname,
            None => return FAIL,
        }
    }

    // SAFETY: `eap` and `fname` are live.
    if unsafe { handle_mkdir_p_arg(eap, fname) } == FAIL {
        return FAIL;
    }

    // SAFETY: `curbuf` is live.
    let name_was_missing = unsafe { (*curbuf.get()).b_ffname }.is_null();
    // SAFETY: the names and the range are the ones checked above.
    let retval = unsafe {
        buf_write(
            curbuf.get(),
            ffname,
            fname,
            (*eap).line1,
            (*eap).line2,
            eap,
            WriteRequest {
                append: (*eap).append != 0,
                forceit: (*eap).forceit != 0,
                reset_changed: true,
                filtering: false,
            },
        )
    };

    // After ":saveas fname" reset 'readonly'.
    // SAFETY: `eap` and `curbuf` are live.
    unsafe {
        if (*eap).cmdidx == CMD_saveas && retval == OK {
            (*curbuf.get()).b_p_ro = false_0;
            redraw_tabline.set(true);
        }
        // Change directories when the 'acd' option is set and the file name
        // got changed or set.
        if (*eap).cmdidx == CMD_saveas || name_was_missing {
            do_autochdir();
        }
    }
    retval
}

/// The reasons `:write` may not write the current buffer to its own file:
/// readonly mode, no file name, an unwritable target, or a "nofile"/"nowrite"
/// buffer that cannot be written implicitly.
///
/// # Safety
/// `eap` must be live; `eap->forceit` may be set by the dialog.
unsafe fn cannot_write_curbuf(eap: *mut exarg_T) -> bool {
    // SAFETY: caller's contract; `curbuf` is the live current buffer.
    unsafe {
        bt_dontwrite_msg(curbuf.get())
            || check_fname() == FAIL
            || check_writable((*curbuf.get()).b_ffname) == FAIL
            || check_readonly(&raw mut (*eap).forceit, curbuf.get())
    }
}

/// Writing less than the whole buffer needs a `!`, or the user's blessing.
///
/// # Safety
/// `eap` must be live; `eap->forceit` may be set by the dialog.
unsafe fn confirm_partial_write(eap: *mut exarg_T) -> bool {
    // SAFETY: caller's contract; `curbuf` is live.
    unsafe {
        if ((*eap).line1 == 1 && (*eap).line2 == (*curbuf.get()).b_ml.ml_line_count)
            || (*eap).forceit != 0
            || (*eap).append != 0
            || p_wa.get() != 0
        {
            return true;
        }
        if !confirming() {
            emsg(gettext(c"E140: Use ! to write partial buffer".as_ptr()));
            return false;
        }
        if vim_dialog_yesno(
            VIM_QUESTION as c_int,
            ptr::null_mut(),
            gettext(c"Write partial file?".as_ptr()),
            2,
        ) != VIM_YES as c_int
        {
            return false;
        }
        (*eap).forceit = true_0;
        true
    }
}

/// `:saveas` swaps the current buffer's names with the alternate buffer's, so
/// that it looks like the buffer is now being edited under the new name.
///
/// This has to happen before `buf_write`, because with no file name and 'cpo'
/// containing 'F' that call would set one.
///
/// Returns the short name to write under, or `None` when an autocommand
/// changed the current buffer or aborted the script.
///
/// # Safety
/// `alt_buf` must be a live buffer other than the current one.
unsafe fn saveas_exchange_names(alt_buf: *mut buf_T) -> Option<*mut c_char> {
    let was_curbuf = curbuf.get();
    // SAFETY: both buffers are live.
    unsafe {
        buf_autocmd(EVENT_BUFFILEPRE, curbuf.get());
        buf_autocmd(EVENT_BUFFILEPRE, alt_buf);
    }
    // buffer changed, don't change name now
    if curbuf.get() != was_curbuf || aborting() {
        return None;
    }

    // Exchange the file names for the current and the alternate buffer.
    // SAFETY: caller's contract, and the two buffers are distinct.
    unsafe {
        ptr::swap(
            &raw mut (*alt_buf).b_fname,
            &raw mut (*curbuf.get()).b_fname,
        );
        ptr::swap(
            &raw mut (*alt_buf).b_ffname,
            &raw mut (*curbuf.get()).b_ffname,
        );
        ptr::swap(
            &raw mut (*alt_buf).b_sfname,
            &raw mut (*curbuf.get()).b_sfname,
        );
        buf_name_changed(curbuf.get());
        buf_autocmd(EVENT_BUFFILEPOST, curbuf.get());
        buf_autocmd(EVENT_BUFFILEPOST, alt_buf);
        if (*alt_buf).b_p_bl == 0 {
            (*alt_buf).b_p_bl = true_0;
            buf_autocmd(EVENT_BUFADD, alt_buf);
        }
    }
    // buffer changed, don't write the file
    if curbuf.get() != was_curbuf || aborting() {
        return None;
    }

    // SAFETY: `curbuf` is live.
    unsafe {
        // If 'filetype' was empty try detecting it now.
        if *(*curbuf.get()).b_p_ft as c_int == NUL {
            if augroup_exists(c"filetypedetect".as_ptr()) {
                do_doautocmd(
                    c"filetypedetect BufRead".as_ptr().cast_mut(),
                    true,
                    ptr::null_mut(),
                );
            }
            do_modelines(0);
        }
        // Autocommands may have changed buffer names, esp. when 'autochdir'
        // is set.
        Some((*curbuf.get()).b_sfname)
    }
}

/// Check if it is allowed to overwrite a file.  If `b_flags` has `BF_NOTEDITED`,
/// `BF_NEW` or `BF_READERR`, check for overwriting the current file.
///
/// May set `eap->forceit` if a dialog says it's OK to overwrite.  `fname` is
/// the file name to be used (which can differ from `buf`'s), `ffname` its full
/// path version, and `other` says the write goes under another name.
///
/// Returns `OK` if it's OK, `FAIL` if it is not.
///
/// # Safety
/// `eap`, `buf` and the two names must be live.
pub unsafe fn check_overwrite(
    eap: *mut exarg_T,
    buf: *mut buf_T,
    fname: *mut c_char,
    ffname: *mut c_char,
    other: bool,
) -> c_int {
    // Write to another file or b_flags set or not writing the whole file.
    // SAFETY: caller's contract.
    let contested = other
        || unsafe {
            !bt_nofilename(buf)
                && ((*buf).b_flags & BF_NOTEDITED != 0
                    || (*buf).b_flags & BF_NEW != 0
                        && vim_strchr(p_cpo.get(), CPO_OVERNEW).is_null()
                    || (*buf).b_flags & BF_READERR != 0)
        };
    // SAFETY: `ffname` is a live file name.
    if !contested || p_wa.get() != 0 || !unsafe { os_path_exists(ffname) } {
        return OK;
    }

    // SAFETY: caller's contract.
    if unsafe { (*eap).forceit } == 0 && unsafe { (*eap).append } == 0 {
        // SAFETY: as above; one `%s` for one string.
        if unsafe { os_isdir(ffname) } {
            unsafe {
                semsg_c!(gettext(&raw const e_isadir2 as *const c_char), ffname,);
            }
            return FAIL;
        }
        if !confirming() {
            // SAFETY: a live message string.
            unsafe { emsg(gettext(&raw const e_exists as *const c_char)) };
            return FAIL;
        }
        // SAFETY: one `%s` for `fname`.
        if !unsafe {
            dialog_yesno_about(gettext(c"Overwrite existing file \"%s\"?".as_ptr()), fname)
        } {
            return FAIL;
        }
        // SAFETY: caller's contract.
        unsafe { (*eap).forceit = true_0 };
    }

    if !other || emsg_silent.get() != 0 {
        return OK;
    }

    // A swap file of the target's own would be silently orphaned by the
    // write, so it is worth a question of its own.
    // SAFETY: the names are live and `dir` is our own allocation.
    let swapname = unsafe {
        let dir = swap_dir();
        let swapname = makeswapname(fname, ffname, curbuf.get(), dir);
        xfree(dir.cast());
        Owned(swapname)
    };
    // SAFETY: `swapname` is a live file name.
    if !unsafe { os_path_exists(swapname.0) } {
        return OK;
    }
    if !confirming() {
        // SAFETY: one `%s` for one string.
        unsafe {
            semsg_c!(
                gettext(c"E768: Swap file exists: %s (:silent! overrides)".as_ptr()),
                swapname.0,
            );
        }
        return FAIL;
    }
    // SAFETY: one `%s` for `swapname`.
    if !unsafe {
        dialog_yesno_about(
            gettext(c"Swap file \"%s\" exists, overwrite anyway?".as_ptr()),
            swapname.0,
        )
    } {
        return FAIL;
    }
    // SAFETY: caller's contract.
    unsafe { (*eap).forceit = true_0 };
    OK
}

/// The first entry of 'directory', or `"."` when the option is empty -- where
/// `makeswapname` should look for the target's swap file.
///
/// # Safety
/// Main thread; the result is the caller's to `xfree`.
unsafe fn swap_dir() -> *mut c_char {
    // SAFETY: 'directory' is a live option string, and both allocations are
    // large enough for what is copied into them.
    unsafe {
        if *p_dir.get() as c_int == NUL {
            let dir = xmalloc(5) as *mut c_char;
            strcpy(dir, c".".as_ptr());
            dir
        } else {
            let dir = xmalloc(MAXPATHL as usize) as *mut c_char;
            let mut p = p_dir.get();
            copy_option_part(&raw mut p, dir, MAXPATHL as usize, c",".as_ptr().cast_mut());
            dir
        }
    }
}

/// `:wnext`, `:wNext` and `:wprevious` -- write, then step through the
/// argument list.
///
/// # Safety
/// `eap` must be the live Ex-command argument.
pub unsafe fn ex_wnext(eap: *mut exarg_T) {
    // SAFETY: caller's contract; `curwin`/`curbuf` are live.
    unsafe {
        let step = (*eap).line2 as c_int;
        let i = if *(*eap).cmd.add(1) as c_int == 'n' as c_int {
            (*curwin.get()).w_arg_idx + step
        } else {
            (*curwin.get()).w_arg_idx - step
        };
        (*eap).line1 = 1;
        (*eap).line2 = (*curbuf.get()).b_ml.ml_line_count;
        if do_write(eap) != FAIL {
            do_argfile(eap, i);
        }
    }
}

/// `:wall`, `:wqall` and `:xall`: write all changed files (and exit).
///
/// # Safety
/// `eap` must be the live Ex-command argument.
pub unsafe fn do_wqall(eap: *mut exarg_T) {
    let mut error = 0;
    // SAFETY: caller's contract.
    let save_forceit = unsafe { (*eap).forceit };
    let save_exiting = exiting.get();

    // SAFETY: as above.
    if unsafe { (*eap).cmdidx } == CMD_xall || unsafe { (*eap).cmdidx } == CMD_wqall {
        if unsafe { before_quit_all(eap) } == FAIL {
            return;
        }
        exiting.set(true);
    }

    let mut buf: *mut buf_T = firstbuf.get();
    while !buf.is_null() {
        // SAFETY: `buf` is a live buffer of the editor's own list.
        match unsafe { write_one_buffer(eap, buf, save_forceit, &mut error) } {
            WriteAll::Stop => break,
            // The buffer was deleted under us.  Upstream restarts from
            // `firstbuf` and then takes the step below, so the first buffer
            // is not looked at a second time.
            WriteAll::Restart => buf = firstbuf.get(),
            WriteAll::Next => {}
        }
        // SAFETY: as above.
        buf = unsafe { (*buf).b_next };
    }

    if exiting.get() {
        if error == 0 {
            // exit Vim
            // SAFETY: main thread; this does not return.
            unsafe { getout(0) };
        }
        // SAFETY: main thread.
        unsafe { not_exiting(save_exiting) };
    }
}

/// What `:wall`'s walk should do after one buffer.
enum WriteAll {
    /// Step to the next buffer.
    Next,
    /// An autocommand deleted this buffer; resume from the buffer list's head.
    Restart,
    /// Writing is disabled; abandon the walk.
    Stop,
}

/// One step of `:wall`'s walk, counting every buffer it could not write into
/// `error`.
///
/// # Safety
/// `eap` and `buf` must be live.
unsafe fn write_one_buffer(
    eap: *mut exarg_T,
    buf: *mut buf_T,
    save_forceit: c_int,
    error: &mut c_int,
) -> WriteAll {
    // SAFETY: caller's contract.
    unsafe {
        // TODO(zeertzjq): channel_job_running always returns false for
        // nvim_open_term() terminals.  Use terminal_running() instead?
        if exiting.get()
            && (*eap).forceit == 0
            && !(*buf).terminal.is_null()
            && channel_job_running((*buf).b_p_channel as u64)
        {
            no_write_message_buf(buf);
            *error += 1;
        } else if !bufIsChanged(buf) || bt_dontwrite(buf) {
            return WriteAll::Next;
        }

        // Check if there is a reason the buffer cannot be written:
        // 1. if the 'write' option is set
        // 2. if there is no file name (even after browsing)
        // 3. if the 'readonly' is set (even after a dialog)
        // 4. if overwriting is allowed (even after a dialog)
        if not_writing() {
            *error += 1;
            return WriteAll::Stop;
        }
        let mut deleted = false;
        if (*buf).b_ffname.is_null() {
            semsg_c!(
                gettext(c"E141: No file name for buffer %ld".as_ptr()),
                (*buf).handle as int64_t,
            );
            *error += 1;
        } else if check_readonly(&raw mut (*eap).forceit, buf)
            || check_overwrite(eap, buf, (*buf).b_fname, (*buf).b_ffname, false) == FAIL
        {
            *error += 1;
        } else {
            let mut bufref = bufref_T::default();
            set_bufref(&raw mut bufref, buf);
            if handle_mkdir_p_arg(eap, (*buf).b_fname) == FAIL
                || buf_write_all(buf, (*eap).forceit != 0) == FAIL
            {
                *error += 1;
            }
            // An autocommand may have deleted the buffer.
            deleted = !bufref_valid(&raw mut bufref);
        }
        // check_overwrite() may set it
        (*eap).forceit = save_forceit;
        if deleted {
            WriteAll::Restart
        } else {
            WriteAll::Next
        }
    }
}

/// Check the 'write' option.
///
/// Returns true and gives a message when writing is disabled.
///
/// # Safety
/// Main thread, message state.
unsafe fn not_writing() -> bool {
    if p_write.get() != 0 {
        return false;
    }
    // SAFETY: a literal.
    unsafe {
        emsg(gettext(
            c"E142: File not written: Writing is disabled by 'write' option".as_ptr(),
        ));
    }
    true
}

/// Check if a buffer is read-only -- either the 'readonly' option is set, or
/// the file's own permissions say so.  Asks for overruling in a dialog.
///
/// Returns true and gives an error message when the buffer is read-only.
///
/// # Safety
/// `forceit` and `buf` must be live; `*forceit` may be set by the dialog.
unsafe fn check_readonly(forceit: *mut c_int, buf: *mut buf_T) -> bool {
    // Handle a file being readonly when the 'readonly' option is set or when
    // the file exists and permissions are read-only.
    // SAFETY: caller's contract.
    let readonly = unsafe {
        *forceit == 0
            && ((*buf).b_p_ro != 0
                || os_path_exists((*buf).b_ffname) && os_file_is_writable((*buf).b_ffname) == 0)
    };
    if !readonly {
        return false;
    }

    // SAFETY: caller's contract.
    let (is_ro, name) = unsafe { ((*buf).b_p_ro != 0, (*buf).b_fname) };
    if !confirming() || name.is_null() {
        // SAFETY: live message strings; one `%s` for one string.
        unsafe {
            if is_ro {
                emsg(gettext(&raw const e_readonly as *const c_char));
            } else {
                semsg_c!(
                    gettext(c"E505: \"%s\" is read-only (add ! to override)".as_ptr()),
                    name,
                );
            }
        }
        return true;
    }

    let prompt = if is_ro {
        c"'readonly' option is set for \"%s\".\nDo you wish to write anyway?".as_ptr()
    } else {
        c"File permissions of \"%s\" are read-only.\nIt may still be possible to write it.\nDo you wish to try?"
            .as_ptr()
    };
    // SAFETY: one `%s` for `name`.
    if !unsafe { dialog_yesno_about(gettext(prompt), name) } {
        return true;
    }
    // Set forceit, to force the writing of a readonly file.
    // SAFETY: caller's contract.
    unsafe { *forceit = true_0 };
    false
}

/// Try to abandon the current file and edit a new or existing one.  `fnum` is
/// the number of the file, or zero to use `ffname_arg`/`sfname_arg`; `lnum` is
/// the line the cursor should land on, if non-zero.
///
/// Returns `GETFILE_ERROR` for a "normal" error, `GETFILE_NOT_WRITTEN` for a
/// "not written" error, `GETFILE_SAME_FILE` for success and
/// `GETFILE_OPEN_OTHER` for successfully opening another file.
///
/// # Safety
/// The two names must be live, or NULL.
pub unsafe fn getfile(
    fnum: c_int,
    ffname_arg: *mut c_char,
    sfname_arg: *mut c_char,
    setpm: bool,
    lnum: linenr_T,
    forceit: bool,
) -> c_int {
    if !check_can_set_curbuf_forceit(forceit as c_int) {
        return GETFILE_ERROR;
    }
    // SAFETY: main thread.
    if unsafe { text_locked() } || unsafe { curbuf_locked() } {
        return GETFILE_ERROR;
    }

    let mut ffname = ffname_arg;
    let mut sfname = sfname_arg;
    // has been allocated, freed when it goes out of scope
    let mut free_me = Owned(ptr::null_mut());
    let other;
    if fnum == 0 {
        // make ffname full path, set sfname
        // SAFETY: caller's contract; `curbuf` is live.
        unsafe {
            fname_expand(curbuf.get(), &raw mut ffname, &raw mut sfname);
            other = otherfile(ffname);
        }
        free_me = Owned(ffname);
    } else {
        // SAFETY: `curbuf` is live.
        other = fnum != unsafe { (*curbuf.get()).handle };
    }

    if other {
        // don't wait for autowrite message
        no_wait_return.set(no_wait_return.get() + 1);
    }
    // SAFETY: `curbuf` is the live current buffer.
    if other
        && !forceit
        && unsafe { (*curbuf.get()).b_nwindows } == 1
        && !unsafe { buf_hide(curbuf.get()) }
        && unsafe { curbufIsChanged() }
        && unsafe { autowrite(curbuf.get(), forceit) } == FAIL
    {
        if p_confirm.get() != 0 && p_write.get() != 0 {
            // SAFETY: as above.
            unsafe { dialog_changed(curbuf.get(), false) };
        }
        // SAFETY: as above.
        if unsafe { curbufIsChanged() } {
            no_wait_return.set(no_wait_return.get() - 1);
            // File has been changed.
            no_write_message();
            return GETFILE_NOT_WRITTEN;
        }
    }
    if other {
        no_wait_return.set(no_wait_return.get() - 1);
    }
    if setpm {
        // SAFETY: main thread.
        unsafe { setpcmark() };
    }

    if !other {
        // SAFETY: `curwin` is the live current window.
        unsafe {
            if lnum != 0 {
                (*curwin.get()).w_cursor.lnum = lnum;
            }
            check_cursor_lnum(curwin.get());
            beginline(BL_SOL as c_int | BL_FIX as c_int);
        }
        // it's in the same file
        return GETFILE_SAME_FILE;
    }

    // SAFETY: the names are live for the duration of the call; `free_me` owns
    // whatever `fname_expand` allocated until this function returns.
    let opened = unsafe {
        do_ecmd(
            fnum,
            ffname,
            sfname,
            ptr::null_mut(),
            lnum,
            (if buf_hide(curbuf.get()) {
                ECMD_HIDE as c_int
            } else {
                0
            }) + (if forceit { ECMD_FORCEIT as c_int } else { 0 }),
            curwin.get(),
        )
    } == OK;
    drop(free_me);
    if opened {
        // opened another file
        GETFILE_OPEN_OTHER
    } else {
        // error encountered
        GETFILE_ERROR
    }
}
