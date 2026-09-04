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
    GETFILE_ERROR, GETFILE_NOT_WRITTEN, GETFILE_OPEN_OTHER, GETFILE_SAME_FILE, NODE_OTHER,
    VIM_QUESTION, VIM_YES, buf_autocmd, do_bang, do_ecmd,
};
use super::{cur_buf, cur_win};
use crate::arglist::do_argfile;
use crate::autocmd::{augroup_exists, do_doautocmd};
use crate::buffer::{
    BufFlags, BufRef, buf_dontwrite_msg, buf_hide, buf_is_dontwrite, buf_is_nofilename,
    buf_name_changed, buflist_findname, buflist_new, current_buf, do_autochdir, do_modelines,
    fileinfo, fname_expand, no_write_message, no_write_message_buf, otherfile, setaltfname,
    setfname,
};
use crate::bufwrite::{WriteRequest, buf_write};
use crate::channel::channel_job_running;
use crate::cursor::check_cursor_lnum;
use crate::edit::{BeginlineOpts, beginline};
use crate::ex_cmds::EcmdFlags;
use crate::ex_cmds2::{autowrite, buf_write_all, check_fname, dialog_changed};
use crate::ex_docmd::{before_quit_all, cmdmod_has, dialog_msg, not_exiting};
use crate::ex_eval::aborting;
use crate::ex_getln::{curbuf_locked, text_locked};
use crate::guard::Suppress;
use crate::main::{
    curbuf, curwin, e_argreq, e_bufloaded, e_exists, e_invarg, e_readonly, emsg_silent, exiting,
    getout, p_confirm, p_dir, p_wa, p_write, redraw_tabline,
};
use crate::mark::setpcmark;
use crate::memline::makeswapname;
use crate::memory::{xfree, xmalloc};
use crate::message::{emsg, vim_dialog_yesno};
use crate::message_fmt::c_str;
use crate::option::{copy_option_part, cpo_has, shortmess};
use crate::os::cshim::{gettext, gettext_ptr};
use crate::os::fs::{os_file_is_writable, os_file_mkdir, os_isdir, os_nodetype, os_path_exists};
use crate::path::fix_fname;
use crate::semsg;
use crate::types::AutoEvent;
use crate::types::CmdIdx;
use crate::types::{
    CmdModFlags, CpoFlag, Failed, MAXPATHL, NUL, OptionSetFlags, ShmFlag, exarg_T, int32_t,
    int64_t, linenr_T,
};
use crate::undo::{buf_is_changed, curbuf_is_changed};
use crate::window::check_can_set_curbuf_forceit;
use crate::winlayer::{Buf, first_buffer};
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
    p_confirm.get() != 0 || cmdmod_has(CmdModFlags::CONFIRM)
}

/// Put `name` into the one-`%s` message `fmt` and ask the user to confirm it.
///
/// # Safety
/// `fmt` must be a format taking exactly one string, and `name` must be live.
unsafe fn dialog_yesno_about(fmt: *mut c_char, name: *mut c_char) -> bool {
    let mut buff: [c_char; DIALOG_MSG_SIZE] = [0; DIALOG_MSG_SIZE];
    // SAFETY: caller's contract; `buff` is the `DIALOG_MSG_SIZE` upstream
    // sizes its own prompt buffers to.
    unsafe { dialog_msg(buff.as_mut_ptr(), fmt, name) };
    unsafe {
        vim_dialog_yesno(VIM_QUESTION as c_int, ptr::null_mut(), buff.as_mut_ptr(), 2)
            == VIM_YES as c_int
    }
}

/// Give the current buffer the name `new_fname`, moving the old name into a
/// new unlisted buffer so that it becomes the alternate file.
///
/// # Safety
/// `new_fname` must be a live file name.
pub unsafe fn rename_buffer(new_fname: *mut c_char) -> Result<(), Failed> {
    let buf = curbuf.get();
    buf_autocmd(AutoEvent::BufFilePre, cur_buf());
    // buffer changed, don't change name now
    if buf != curbuf.get() {
        return Err(Failed);
    }
    if aborting() {
        // autocmds may abort script processing
        return Err(Failed);
    }

    // The name of the current buffer will be changed.
    // A new (unlisted) buffer entry needs to be made to hold the old file
    // name, which will become the alternate file name.  But don't set the
    // alternate file name if the buffer didn't have a name.
    let (fname, sfname, xfname) = (cur_buf().b_ffname, cur_buf().b_sfname, cur_buf().b_fname);
    cur_buf().b_ffname = ptr::null_mut();
    cur_buf().b_sfname = ptr::null_mut();
    // SAFETY: caller's contract; the names are handed back on failure.
    if unsafe { setfname(cur_buf(), new_fname, ptr::null_mut(), true) }.is_err() {
        cur_buf().b_ffname = fname;
        cur_buf().b_sfname = sfname;
        return Err(Failed);
    }
    cur_buf().b_flags |= BufFlags::NOTEDITED;
    if !xfname.is_null() && unsafe { *xfname } as c_int != NUL {
        let alt = unsafe { buflist_new(fname, xfname, cur_win().w_cursor.lnum, 0) };
        if !alt.is_null() && !cmdmod_has(CmdModFlags::KEEPALT) {
            cur_win().w_alt_fnum = unsafe { (*alt).handle } as c_int;
        }
    }
    unsafe { xfree(fname.cast()) };
    unsafe { xfree(sfname.cast()) };
    buf_autocmd(AutoEvent::BufFilePost, cur_buf());
    // Change directories when the 'acd' option is set.
    do_autochdir();
    Ok(())
}

/// `:file[!] [fname]`.
///
/// # Safety
/// `eap` must be the live Ex-command argument.
pub unsafe fn ex_file(eap: *mut exarg_T) {
    // SAFETY: caller's contract.
    let eap = unsafe { &mut *eap };
    // SAFETY: `eap.arg` is the command's NUL-terminated argument.
    let no_arg = unsafe { *eap.arg } as c_int == NUL;

    // ":0file" removes the file name.  Check for illegal uses ":3file",
    // "0file name", etc.
    if eap.addr_count > 0 && (!no_arg || eap.line2 > 0 || eap.addr_count > 1) {
        emsg(gettext(e_invarg));
        return;
    }

    if !no_arg || eap.addr_count == 1 {
        // SAFETY: as above.
        if unsafe { rename_buffer(eap.arg) }.is_err() {
            return;
        }
        redraw_tabline.set(true);
    }

    // print file name if no argument or 'F' is not in 'shortmess'
    if no_arg || !shortmess(ShmFlag::FILEINFO) {
        // SAFETY: main thread, message state.
        unsafe { fileinfo(0, 0, eap.forceit != 0) };
    }
}

/// `:update` -- write only when there is something to write.
///
/// # Safety
/// `eap` must be the live Ex-command argument.
pub unsafe fn ex_update(eap: *mut exarg_T) {
    // SAFETY: caller's contract.
    let eap = unsafe { &mut *eap };
    // SAFETY: `curbuf` is live.
    if curbuf_is_changed()
        || (!buf_is_nofilename(current_buf())
            && !cur_buf().b_ffname.is_null()
            && !unsafe { os_path_exists(cur_buf().b_ffname) })
    {
        let _ = unsafe { do_write(eap) };
    }
}

/// `:write` and `:saveas`.
///
/// # Safety
/// `eap` must be the live Ex-command argument.
pub unsafe fn ex_write(eap: *mut exarg_T) {
    // SAFETY: caller's contract.
    let eap = unsafe { &mut *eap };
    if eap.cmdidx == CmdIdx::saveas {
        // :saveas does not take a range, uses all lines.
        eap.line1 = 1;
        eap.line2 = cur_buf().b_ml.ml_line_count;
    }

    if eap.usefilter != 0 {
        // input lines to shell command
        // SAFETY: the command block is the one just borrowed.
        unsafe { do_bang(1, &raw mut *eap, false, true, false) };
    } else {
        let _ = unsafe { do_write(eap) };
    }
}

/// Refuse a device or a socket: only a regular file, or something that can be
/// written like one, may be a write target.
///
/// # Safety
/// `fname` must be live, or NULL.
unsafe fn check_writable(fname: *const c_char) -> Result<(), Failed> {
    // SAFETY: caller's contract; one `%s` for one string.
    if unsafe { os_nodetype(fname) } == NODE_OTHER {
        // SAFETY: a message argument the caller holds as a NUL-terminated string.
        let fname = unsafe { c_str(fname) };
        semsg!("E503: \"{fname}\" is not a file or writable device");
        return Err(Failed);
    }
    Ok(())
}

/// `:write ++p` -- create the missing leading directories.
///
/// # Safety
/// `fname` must be live.
unsafe fn handle_mkdir_p_arg(eap: &exarg_T, fname: *mut c_char) -> Result<(), Failed> {
    // SAFETY: caller's contract.
    if eap.mkdir_p != 0 && unsafe { os_file_mkdir(fname, 0o755 as int32_t) } < 0 {
        return Err(Failed);
    }
    Ok(())
}

/// Write the current buffer to the file `eap->arg` names, or to its own file
/// when that argument is empty.  `eap->append` appends instead of replacing.
///
/// Answers `Err` for failure.
///
/// # Safety
/// `eap` must be the live Ex-command argument.
pub unsafe fn do_write(eap: &mut exarg_T) -> Result<(), Failed> {
    // check 'write' option
    if unsafe { not_writing() } {
        return Err(Failed);
    }

    let mut fname = ptr::null_mut(); // init to shut up gcc
    let mut ffname = eap.arg;
    // When out-of-memory, keep the unexpanded file name, because we MUST be
    // able to write the file in this situation.
    let mut free_fname = Owned(ptr::null_mut());

    // SAFETY: `ffname` is the command's NUL-terminated argument.
    let other = if unsafe { *ffname } as c_int == NUL {
        if eap.cmdidx == CmdIdx::saveas {
            emsg(gettext(e_argreq));
            return Err(Failed);
        }
        false
    } else {
        fname = ffname;
        // SAFETY: as above.
        free_fname = Owned(unsafe { fix_fname(ffname) });
        if !free_fname.0.is_null() {
            ffname = free_fname.0;
        }
        // SAFETY: as above.
        unsafe { otherfile(ffname) }
    };

    // If we have a new file, put its name in the list of alternate file names.
    let mut alt_buf = None;
    if other {
        // SAFETY: the names are live, 'cpoptions' is a live option string, and
        // both lookups hand back a live buffer or NULL.
        alt_buf = unsafe {
            if cpo_has(CpoFlag::ALTWRITE) || eap.cmdidx == CmdIdx::saveas {
                setaltfname(ffname, fname, 1)
            } else {
                buflist_findname(ffname)
            }
        };
        // Overwriting a file that is loaded in another buffer is not a good
        // idea.
        if let Some(alt_buf) = alt_buf
            && !alt_buf.b_ml.ml_mfp.is_null()
        {
            emsg(gettext(e_bufloaded));
            return Err(Failed);
        }
    }

    if !other {
        // SAFETY: `curbuf` is the current buffer.
        if unsafe { cannot_write_curbuf(eap) } {
            return Err(Failed);
        }
        (ffname, fname) = (cur_buf().b_ffname, cur_buf().b_fname);
        // SAFETY: main thread, message state.
        if !unsafe { confirm_partial_write(eap) } {
            return Err(Failed);
        }
    }

    // SAFETY: the names are live.
    unsafe { check_overwrite(eap, cur_buf(), fname, ffname, other) }?;

    if eap.cmdidx == CmdIdx::saveas
        && let Some(alt_buf) = alt_buf
    {
        match saveas_exchange_names(alt_buf) {
            Some(sfname) => fname = sfname,
            None => return Err(Failed),
        }
    }

    // SAFETY: `fname` is live.
    unsafe { handle_mkdir_p_arg(eap, fname) }?;

    let name_was_missing = cur_buf().b_ffname.is_null();
    let request = WriteRequest {
        append: eap.append != 0,
        forceit: eap.forceit != 0,
        reset_changed: true,
        filtering: false,
    };
    let (line1, line2) = (eap.line1, eap.line2);
    // SAFETY: the names and the range are the ones checked above, and the
    // command block is the one borrowed here.
    let retval = unsafe {
        buf_write(
            curbuf.get(),
            ffname,
            fname,
            line1,
            line2,
            &raw mut *eap,
            request,
        )
    };

    // After ":saveas fname" reset 'readonly'.
    if eap.cmdidx == CmdIdx::saveas && retval.is_ok() {
        cur_buf().b_p_ro = 0;
        redraw_tabline.set(true);
    }
    // Change directories when the 'acd' option is set and the file name
    // got changed or set.
    if eap.cmdidx == CmdIdx::saveas || name_was_missing {
        do_autochdir();
    }
    retval
}

/// The reasons `:write` may not write the current buffer to its own file:
/// readonly mode, no file name, an unwritable target, or a "nofile"/"nowrite"
/// buffer that cannot be written implicitly.
///
/// # Safety
/// Main thread, message state; `eap.forceit` may be set by the dialog.
unsafe fn cannot_write_curbuf(eap: &mut exarg_T) -> bool {
    let forceit = &raw mut eap.forceit;
    // SAFETY: `curbuf` is the live current buffer, and `forceit` is the
    // borrowed command's own field. The whole chain is one region so the
    // short-circuiting is untouched -- a block cannot lead a `||` chain in
    // tail position anyway.
    unsafe {
        buf_dontwrite_msg(current_buf())
            || check_fname().is_err()
            || check_writable(cur_buf().b_ffname).is_err()
            || check_readonly(forceit, cur_buf())
    }
}

/// Writing less than the whole buffer needs a `!`, or the user's blessing.
///
/// # Safety
/// Main thread, message state; `eap.forceit` may be set by the dialog.
unsafe fn confirm_partial_write(eap: &mut exarg_T) -> bool {
    if (eap.line1 == 1 && eap.line2 == cur_buf().b_ml.ml_line_count)
        || eap.forceit != 0
        || eap.append != 0
        || p_wa.get() != 0
    {
        return true;
    }
    if !confirming() {
        emsg(gettext(c"E140: Use ! to write partial buffer"));
        return false;
    }
    if unsafe {
        vim_dialog_yesno(
            VIM_QUESTION as c_int,
            ptr::null_mut(),
            gettext(c"Write partial file?").as_ptr().cast_mut(),
            2,
        )
    } != VIM_YES as c_int
    {
        return false;
    }
    eap.forceit = 1;
    true
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
/// Safe: [`Buf`] is the live buffer this needs; `alt_buf` is expected to be a
/// buffer other than the current one, which is a matter of sense rather than
/// of soundness -- swapping a buffer's names with its own is a no-op.
fn saveas_exchange_names(mut alt_buf: Buf) -> Option<*mut c_char> {
    let was_curbuf = curbuf.get();
    buf_autocmd(AutoEvent::BufFilePre, cur_buf());
    buf_autocmd(AutoEvent::BufFilePre, alt_buf);
    // buffer changed, don't change name now
    if curbuf.get() != was_curbuf || aborting() {
        return None;
    }

    // Exchange the file names for the current and the alternate buffer.
    // SAFETY: both buffers are live, so every field address below is.
    unsafe {
        ptr::swap(&raw mut alt_buf.b_fname, &raw mut cur_buf().b_fname);
        ptr::swap(&raw mut alt_buf.b_ffname, &raw mut cur_buf().b_ffname);
        ptr::swap(&raw mut alt_buf.b_sfname, &raw mut cur_buf().b_sfname);
    };
    // SAFETY: `curbuf` is live.
    unsafe { buf_name_changed(cur_buf()) };
    buf_autocmd(AutoEvent::BufFilePost, cur_buf());
    buf_autocmd(AutoEvent::BufFilePost, alt_buf);
    if alt_buf.b_p_bl == 0 {
        alt_buf.b_p_bl = 1;
        buf_autocmd(AutoEvent::BufAdd, alt_buf);
    }
    // buffer changed, don't write the file
    if curbuf.get() != was_curbuf || aborting() {
        return None;
    }

    // SAFETY: `curbuf` is live.
    // If 'filetype' was empty try detecting it now.
    if unsafe { *cur_buf().b_p_ft } as c_int == NUL {
        if unsafe { augroup_exists(c"filetypedetect".as_ptr()) } {
            let _ = unsafe {
                do_doautocmd(
                    c"filetypedetect BufRead".as_ptr().cast_mut(),
                    true,
                    ptr::null_mut(),
                )
            };
        }
        do_modelines(OptionSetFlags::NONE);
    }
    // Autocommands may have changed buffer names, esp. when 'autochdir'
    // is set.
    Some(cur_buf().b_sfname)
}

/// Check if it is allowed to overwrite a file.  If `b_flags` has `BufFlags::NOTEDITED`,
/// `BufFlags::NEW` or `BufFlags::READERR`, check for overwriting the current file.
///
/// May set `eap->forceit` if a dialog says it is fine to overwrite.  `fname` is
/// the file name to be used (which can differ from `buf`'s), `ffname` its full
/// path version, and `other` says the write goes under another name.
///
/// Answers `Err` when the write must not go ahead.
///
/// # Safety
/// The two names must be live.
pub unsafe fn check_overwrite(
    eap: &mut exarg_T,
    buf: Buf,
    fname: *mut c_char,
    ffname: *mut c_char,
    other: bool,
) -> Result<(), Failed> {
    // Write to another file or b_flags set or not writing the whole file.
    // SAFETY: a live buffer.
    let contested = other
        || (!buf_is_nofilename(Some(buf))
            && (buf.b_flags.has(BufFlags::NOTEDITED)
                || buf.b_flags.has(BufFlags::NEW) && !cpo_has(CpoFlag::OVERNEW)
                || buf.b_flags.has(BufFlags::READERR)));
    // SAFETY: `ffname` is a live file name.
    if !contested || p_wa.get() != 0 || !unsafe { os_path_exists(ffname) } {
        return Ok(());
    }

    if eap.forceit == 0 && eap.append == 0 {
        // SAFETY: as above; one `%s` for one string.
        if unsafe { os_isdir(ffname) } {
            // SAFETY: a message argument the caller holds as a NUL-terminated string.
            let ffname = unsafe { c_str(ffname) };
            semsg!("E17: \"{ffname}\" is a directory");
            return Err(Failed);
        }
        if !confirming() {
            emsg(gettext(e_exists));
            return Err(Failed);
        }
        // SAFETY: one `%s` for `fname`.
        if !unsafe {
            dialog_yesno_about(
                gettext(c"Overwrite existing file \"%s\"?")
                    .as_ptr()
                    .cast_mut(),
                fname,
            )
        } {
            return Err(Failed);
        }
        eap.forceit = 1;
    }

    if !other || emsg_silent.get() != 0 {
        return Ok(());
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
        return Ok(());
    }
    if !confirming() {
        // SAFETY: one `%s` for one string.
        let arg0 = unsafe { c_str(swapname.0) };
        semsg!("E768: Swap file exists: {arg0} (:silent! overrides)");
        return Err(Failed);
    }
    // SAFETY: one `%s` for `swapname`.
    if !unsafe {
        dialog_yesno_about(
            gettext(c"Swap file \"%s\" exists, overwrite anyway?")
                .as_ptr()
                .cast_mut(),
            swapname.0,
        )
    } {
        return Err(Failed);
    }
    eap.forceit = 1;
    Ok(())
}

/// The first entry of 'directory', or `"."` when the option is empty -- where
/// `makeswapname` should look for the target's swap file.
///
/// # Safety
/// Main thread; the result is the caller's to `xfree`.
unsafe fn swap_dir() -> *mut c_char {
    // SAFETY: 'directory' is a live option string, and both allocations are
    // large enough for what is copied into them.
    if unsafe { *p_dir.get() } as c_int == NUL {
        let dir = unsafe { xmalloc(5) } as *mut c_char;
        unsafe { strcpy(dir, c".".as_ptr()) };
        dir
    } else {
        let dir = unsafe { xmalloc(MAXPATHL as usize) } as *mut c_char;
        let mut p = p_dir.get();
        unsafe { copy_option_part(&raw mut p, dir, MAXPATHL as usize, c",".as_ptr().cast_mut()) };
        dir
    }
}

/// `:wnext`, `:wNext` and `:wprevious` -- write, then step through the
/// argument list.
///
/// # Safety
/// `eap` must be the live Ex-command argument.
pub unsafe fn ex_wnext(eap: *mut exarg_T) {
    // SAFETY: caller's contract.
    let eap = unsafe { &mut *eap };
    let step = eap.line2 as c_int;
    // SAFETY: the command name is at least two bytes long.
    let forwards = unsafe { *eap.cmd.add(1) } as c_int == 'n' as c_int;
    let i = if forwards {
        cur_win().w_arg_idx + step
    } else {
        cur_win().w_arg_idx - step
    };
    eap.line1 = 1;
    eap.line2 = cur_buf().b_ml.ml_line_count;
    // SAFETY: main thread; the command block is the one borrowed here.
    if unsafe { do_write(eap) }.is_ok() {
        unsafe { do_argfile(&raw mut *eap, i) };
    }
}

/// `:wall`, `:wqall` and `:xall`: write all changed files (and exit).
///
/// # Safety
/// `eap` must be the live Ex-command argument.
pub unsafe fn do_wqall(eap: *mut exarg_T) {
    // SAFETY: caller's contract.
    let eap = unsafe { &mut *eap };
    let mut error = 0;
    let save_forceit = eap.forceit;
    let save_exiting = exiting.get();

    if eap.cmdidx == CmdIdx::xall || eap.cmdidx == CmdIdx::wqall {
        // SAFETY: the command block is the one borrowed here.
        if unsafe { before_quit_all(&raw mut *eap) }.is_err() {
            return;
        }
        exiting.set(true);
    }

    // Not `winlayer::buffers()`: an autocommand fired while writing can delete
    // the buffer under the walk, which is what `WriteAll::Restart` is for --
    // the head has to be re-read, and no iterator re-reads it.
    let mut cur = first_buffer();
    while let Some(buf) = cur {
        // SAFETY: `buf` is a live buffer of the editor's own list.
        match unsafe { write_one_buffer(eap, buf, save_forceit, &mut error) } {
            WriteAll::Stop => break,
            // The buffer was deleted under us.  Upstream restarts from
            // `firstbuf` and then takes the step below, so the first buffer
            // is not looked at a second time.
            WriteAll::Restart => cur = first_buffer(),
            WriteAll::Next => {}
        }
        cur = cur.and_then(Buf::next);
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
/// Main thread; `buf` must be a live buffer.
unsafe fn write_one_buffer(
    eap: &mut exarg_T,
    buf: Buf,
    save_forceit: c_int,
    error: &mut c_int,
) -> WriteAll {
    // SAFETY: caller's contract, and a live buffer.
    // TODO(zeertzjq): channel_job_running always returns false for
    // nvim_open_term() terminals.  Use terminal_running() instead?
    if exiting.get()
        && eap.forceit == 0
        && !buf.terminal.is_null()
        && unsafe { channel_job_running(buf.b_p_channel as u64) }
    {
        no_write_message_buf(buf);
        *error += 1;
    } else if !buf_is_changed(buf) || buf_is_dontwrite(Some(buf)) {
        return WriteAll::Next;
    }

    // Check if there is a reason the buffer cannot be written:
    // 1. if the 'write' option is set
    // 2. if there is no file name (even after browsing)
    // 3. if the 'readonly' is set (even after a dialog)
    // 4. if overwriting is allowed (even after a dialog)
    if unsafe { not_writing() } {
        *error += 1;
        return WriteAll::Stop;
    }
    let mut deleted = false;
    if buf.b_ffname.is_null() {
        semsg!("E141: No file name for buffer {}", buf.handle as int64_t);
        *error += 1;
    } else if unsafe { check_readonly(&raw mut eap.forceit, buf) }
        || unsafe { check_overwrite(eap, buf, buf.b_fname, buf.b_ffname, false) }.is_err()
    {
        *error += 1;
    } else {
        let bufref = BufRef::of(buf);
        if unsafe { handle_mkdir_p_arg(eap, buf.b_fname) }.is_err()
            || unsafe { buf_write_all(buf.raw(), eap.forceit != 0) }.is_err()
        {
            *error += 1;
        }
        // An autocommand may have deleted the buffer.
        deleted = !bufref.valid();
    }
    // check_overwrite() may set it
    eap.forceit = save_forceit;
    if deleted {
        WriteAll::Restart
    } else {
        WriteAll::Next
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
    emsg(gettext(
        c"E142: File not written: Writing is disabled by 'write' option",
    ));
    true
}

/// Check if a buffer is read-only -- either the 'readonly' option is set, or
/// the file's own permissions say so.  Asks for overruling in a dialog.
///
/// Returns true and gives an error message when the buffer is read-only.
///
/// # Safety
/// `forceit` must be live; `*forceit` may be set by the dialog.
unsafe fn check_readonly(forceit: *mut c_int, buf: Buf) -> bool {
    // Handle a file being readonly when the 'readonly' option is set or when
    // the file exists and permissions are read-only.
    // SAFETY: caller's contract, and the buffer's own file name.
    let readonly = unsafe {
        *forceit == 0
            && (buf.b_p_ro != 0
                || os_path_exists(buf.b_ffname) && os_file_is_writable(buf.b_ffname) == 0)
    };
    if !readonly {
        return false;
    }

    let (is_ro, name) = (buf.b_p_ro != 0, buf.b_fname);
    if !confirming() || name.is_null() {
        // SAFETY: live message strings; one `%s` for one string.
        if is_ro {
            emsg(gettext(e_readonly));
        } else {
            // SAFETY: a message argument the caller holds as a NUL-terminated string.
            let name = unsafe { c_str(name) };
            semsg!("E505: \"{name}\" is read-only (add ! to override)");
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
    if !unsafe { dialog_yesno_about(gettext_ptr(prompt).as_ptr().cast_mut(), name) } {
        return true;
    }
    // Set forceit, to force the writing of a readonly file.
    // SAFETY: caller's contract.
    unsafe { *forceit = 1 };
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
        unsafe { fname_expand(cur_buf(), &raw mut ffname, &raw mut sfname) };
        other = unsafe { otherfile(ffname) };
        free_me = Owned(ffname);
    } else {
        // SAFETY: `curbuf` is live.
        other = fnum != cur_buf().handle;
    }

    // Don't wait for the autowrite message. Released at two exits.
    let mut no_prompt = other.then(Suppress::wait_return);
    // SAFETY: `curbuf` is the live current buffer.
    if other
        && !forceit
        && cur_buf().b_nwindows == 1
        && !unsafe { buf_hide(curbuf.get()) }
        && curbuf_is_changed()
        && unsafe { autowrite(curbuf.get(), forceit) }.is_err()
    {
        if p_confirm.get() != 0 && p_write.get() != 0 {
            // SAFETY: as above.
            unsafe { dialog_changed(curbuf.get(), false) };
        }
        // SAFETY: as above.
        if curbuf_is_changed() {
            drop(no_prompt.take());
            // File has been changed.
            no_write_message();
            return GETFILE_NOT_WRITTEN;
        }
    }
    drop(no_prompt.take());
    if setpm {
        // SAFETY: main thread.
        setpcmark();
    }

    if !other {
        // SAFETY: `curwin` is the live current window.
        if lnum != 0 {
            cur_win().w_cursor.lnum = lnum;
        }
        check_cursor_lnum(cur_win());
        beginline(BeginlineOpts::SOL | BeginlineOpts::FIX);
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
            EcmdFlags::HIDE.when(buf_hide(curbuf.get())) | EcmdFlags::FORCEIT.when(forceit),
            curwin.get(),
        )
    }
    .is_ok();
    drop(free_me);
    if opened {
        // opened another file
        GETFILE_OPEN_OTHER
    } else {
        // error encountered
        GETFILE_ERROR
    }
}
