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
#![allow(unsafe_code)]

use super::Owned;
use super::{
    GETFILE_ERROR, GETFILE_NOT_WRITTEN, GETFILE_OPEN_OTHER, GETFILE_SAME_FILE, NODE_OTHER,
    VIM_QUESTION, VIM_YES, buf_autocmd, do_bang, do_ecmd,
};
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
use crate::cstr;
use crate::cursor::check_cursor_lnum;
use crate::drawscreen::state::redraw_tabline;
use crate::edit::{BeginlineOpts, beginline};
use crate::ex_cmds::EcmdFlags;
use crate::ex_cmds2::{autowrite, buf_write_all, check_fname, dialog_changed};
use crate::ex_docmd::{before_quit_all, cmdmod_has, dialog_msg, not_exiting};
use crate::ex_eval::aborting;
use crate::ex_getln::{curbuf_locked, text_locked};
use crate::guard::Suppress;
use crate::mark::setpcmark;
use crate::memline::makeswapname;
use crate::memory::xfree;
use crate::message::state::emsg_silent;
use crate::message::{e_argreq, e_bufloaded, e_exists, e_invarg, e_readonly};
use crate::message::{emsg, vim_dialog_yesno};
use crate::message_fmt::c_str;
use crate::option::vars::{P_DIR, p_confirm, p_dir, p_wa, p_write};
use crate::option::{copy_option_part, cpo_has, shortmess};
use crate::optionstr::LocalOptStr;
use crate::os::cshim::{gettext, gettext_ptr};
use crate::os::fs::{os_file_is_writable, os_file_mkdir, os_isdir, os_nodetype, os_path_exists};
use crate::path::fix_fname;
use crate::semsg;
use crate::startup::{exiting, getout};
use crate::types::AutoEvent;
use crate::types::CmdIdx;
use crate::types::{
    CmdModFlags, CpoFlag, ExArg, Failed, LineNr, MAXPATHL, NUL, OptionSetFlags, ShmFlag, int32_t,
    int64_t,
};
use crate::undo::{buf_is_changed, curbuf_is_changed};
use crate::window::check_can_set_curbuf_forceit;
use crate::winlayer::Win;
use crate::winlayer::{Buf, first_buffer};
use core::ffi::CStr;
use core::ffi::{c_char, c_int};
use core::ptr;

/// The buffer `dialog_msg` formats a prompt into.
const DIALOG_MSG_SIZE: usize = 1000;

/// Is a `:confirm` dialog wanted here -- either from 'confirm' or from the
/// command's own modifier?
fn confirming() -> bool {
    p_confirm() || cmdmod_has(CmdModFlags::CONFIRM)
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
    let buf = Buf::current_raw();
    buf_autocmd(AutoEvent::BufFilePre, Buf::current());
    // buffer changed, don't change name now
    if buf != Buf::current_raw() {
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
    let (fname, sfname, xfname) = (
        Buf::current().b_ffname,
        Buf::current().b_sfname,
        Buf::current().b_fname,
    );
    Buf::current().b_ffname = ptr::null_mut();
    Buf::current().b_sfname = ptr::null_mut();
    // SAFETY: caller's contract; the names are handed back on failure.
    if unsafe { setfname(Buf::current(), new_fname, ptr::null_mut(), true) }.is_err() {
        Buf::current().b_ffname = fname;
        Buf::current().b_sfname = sfname;
        return Err(Failed);
    }
    Buf::current().b_flags |= BufFlags::NOTEDITED;
    if !xfname.is_null() && unsafe { *xfname } as c_int != NUL {
        let alt = unsafe { buflist_new(fname, xfname, Win::current().w_cursor.lnum, 0) };
        if let Some(alt) = alt.filter(|_| !cmdmod_has(CmdModFlags::KEEPALT)) {
            Win::current().w_alt_fnum = alt.handle as c_int;
        }
    }
    unsafe { xfree(fname.cast()) };
    unsafe { xfree(sfname.cast()) };
    buf_autocmd(AutoEvent::BufFilePost, Buf::current());
    // Change directories when the 'acd' option is set.
    do_autochdir();
    Ok(())
}

/// `:file[!] [fname]`.
pub fn ex_file(excmd: &mut ExArg) {
    // SAFETY: `args.arg` is the command's NUL-terminated argument.
    let no_arg = unsafe { *excmd.arg } as c_int == NUL;

    // ":0file" removes the file name.  Check for illegal uses ":3file",
    // "0file name", etc.
    if excmd.addr_count > 0 && (!no_arg || excmd.line2 > 0 || excmd.addr_count > 1) {
        emsg(gettext(e_invarg));
        return;
    }

    if !no_arg || excmd.addr_count == 1 {
        // SAFETY: as above.
        if unsafe { rename_buffer(excmd.arg) }.is_err() {
            return;
        }
        redraw_tabline.set(true);
    }

    // print file name if no argument or 'F' is not in 'shortmess'
    if no_arg || !shortmess(ShmFlag::FILEINFO) {
        fileinfo(0, 0, excmd.forceit);
    }
}

/// `:update` -- write only when there is something to write.
pub fn ex_update(excmd: &mut ExArg) {
    // SAFETY: `curbuf` is live.
    if curbuf_is_changed()
        || (!buf_is_nofilename(current_buf())
            && !Buf::current().b_ffname.is_null()
            && !unsafe { os_path_exists(Buf::current().b_ffname) })
    {
        let _ = do_write(excmd);
    }
}

/// `:write` and `:saveas`.
pub fn ex_write(excmd: &mut ExArg) {
    if excmd.cmdidx == CmdIdx::saveas {
        // :saveas does not take a range, uses all lines.
        excmd.line1 = 1;
        excmd.line2 = Buf::current().b_ml.ml_line_count;
    }

    if excmd.usefilter {
        // input lines to shell command
        do_bang(1, excmd, false, true, false);
    } else {
        let _ = do_write(excmd);
    }
}

/// Refuse a device or a socket: only a regular file, or something that can be
/// written like one, may be a write target.
///
/// # Safety
/// `fname` must be a live NUL-terminated name. The one caller reaches this
/// only past `check_fname`, which rejects a buffer with no file name.
unsafe fn check_writable(fname: *const c_char) -> Result<(), Failed> {
    // SAFETY: caller's contract; one `%s` for one string.
    if unsafe { os_nodetype(cstr::at(fname)) } == NODE_OTHER {
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
unsafe fn handle_mkdir_p_arg(args: &ExArg, fname: *mut c_char) -> Result<(), Failed> {
    // SAFETY: caller's contract.
    if args.mkdir_p && unsafe { os_file_mkdir(fname, 0o755 as int32_t) } < 0 {
        return Err(Failed);
    }
    Ok(())
}

/// Write the current buffer to the file `args.arg` names, or to its own file
/// when that argument is empty.  `args.append` appends instead of replacing.
///
/// Answers `Err` for failure.
pub fn do_write(args: &mut ExArg) -> Result<(), Failed> {
    // check 'write' option
    if not_writing() {
        return Err(Failed);
    }

    let mut fname = ptr::null_mut(); // init to shut up gcc
    let mut ffname = args.arg;
    // When out-of-memory, keep the unexpanded file name, because we MUST be
    // able to write the file in this situation.
    let free_fname;

    // SAFETY: `ffname` is the command's NUL-terminated argument.
    let other = if unsafe { *ffname } as c_int == NUL {
        if args.cmdidx == CmdIdx::saveas {
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
            if cpo_has(CpoFlag::ALTWRITE) || args.cmdidx == CmdIdx::saveas {
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
        if cannot_write_curbuf(args) {
            return Err(Failed);
        }
        (ffname, fname) = (Buf::current().b_ffname, Buf::current().b_fname);
        if !confirm_partial_write(args) {
            return Err(Failed);
        }
    }

    // SAFETY: the names are live.
    unsafe { check_overwrite(args, Buf::current(), fname, ffname, other) }?;

    if args.cmdidx == CmdIdx::saveas
        && let Some(alt_buf) = alt_buf
    {
        match saveas_exchange_names(alt_buf) {
            Some(sfname) => fname = sfname,
            None => return Err(Failed),
        }
    }

    // SAFETY: `fname` is live.
    unsafe { handle_mkdir_p_arg(args, fname) }?;

    let name_was_missing = Buf::current().b_ffname.is_null();
    let request = WriteRequest {
        append: args.append,
        forceit: args.forceit,
        reset_changed: true,
        filtering: false,
    };
    let (line1, line2) = (args.line1, args.line2);
    // SAFETY: the names and the range are the ones checked above, and the
    // command block is the one borrowed here.
    let retval = unsafe {
        buf_write(
            Buf::current(),
            ffname,
            fname,
            line1,
            line2,
            Some(args),
            request,
        )
    };

    // After ":saveas fname" reset 'readonly'.
    if args.cmdidx == CmdIdx::saveas && retval.is_ok() {
        Buf::current().b_p_ro = 0;
        redraw_tabline.set(true);
    }
    // Change directories when the 'acd' option is set and the file name
    // got changed or set.
    if args.cmdidx == CmdIdx::saveas || name_was_missing {
        do_autochdir();
    }
    retval
}

/// The reasons `:write` may not write the current buffer to its own file:
/// readonly mode, no file name, an unwritable target, or a "nofile"/"nowrite"
/// buffer that cannot be written implicitly.
fn cannot_write_curbuf(args: &mut ExArg) -> bool {
    // SAFETY: `curbuf` is the live current buffer. The whole chain is one
    // region so the short-circuiting is untouched -- a block cannot lead a
    // `||` chain in tail position anyway.
    unsafe {
        buf_dontwrite_msg(current_buf())
            || check_fname().is_err()
            || check_writable(Buf::current().b_ffname).is_err()
            || check_readonly(&mut args.forceit, Buf::current())
    }
}

/// Writing less than the whole buffer needs a `!`, or the user's blessing.
fn confirm_partial_write(args: &mut ExArg) -> bool {
    if (args.line1 == 1 && args.line2 == Buf::current().b_ml.ml_line_count)
        || args.forceit
        || args.append
        || p_wa()
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
    args.forceit = true;
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
    let was_curbuf = Buf::current_raw();
    buf_autocmd(AutoEvent::BufFilePre, Buf::current());
    buf_autocmd(AutoEvent::BufFilePre, alt_buf);
    // buffer changed, don't change name now
    if Buf::current_raw() != was_curbuf || aborting() {
        return None;
    }

    // Exchange the file names for the current and the alternate buffer.
    // SAFETY: both buffers are live, so every field address below is.
    unsafe {
        ptr::swap(&raw mut alt_buf.b_fname, &raw mut Buf::current().b_fname);
        ptr::swap(&raw mut alt_buf.b_ffname, &raw mut Buf::current().b_ffname);
        ptr::swap(&raw mut alt_buf.b_sfname, &raw mut Buf::current().b_sfname);
    };
    buf_name_changed(Buf::current());
    buf_autocmd(AutoEvent::BufFilePost, Buf::current());
    buf_autocmd(AutoEvent::BufFilePost, alt_buf);
    if alt_buf.b_p_bl == 0 {
        alt_buf.b_p_bl = 1;
        buf_autocmd(AutoEvent::BufAdd, alt_buf);
    }
    // buffer changed, don't write the file
    if Buf::current_raw() != was_curbuf || aborting() {
        return None;
    }

    // If 'filetype' was empty try detecting it now.
    if Buf::current().b_p_ft.first_byte() as c_int == NUL {
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
    Some(Buf::current().b_sfname)
}

/// Check if it is allowed to overwrite a file.  If `b_flags` has `BufFlags::NOTEDITED`,
/// `BufFlags::NEW` or `BufFlags::READERR`, check for overwriting the current file.
///
/// May set `args.forceit` if a dialog says it is fine to overwrite.  `fname` is
/// the file name to be used (which can differ from `buffer`'s), `ffname` its full
/// path version, and `other` says the write goes under another name.
///
/// Answers `Err` when the write must not go ahead.
///
/// # Safety
/// The two names must be live.
pub unsafe fn check_overwrite(
    args: &mut ExArg,
    buffer: Buf,
    fname: *mut c_char,
    ffname: *mut c_char,
    other: bool,
) -> Result<(), Failed> {
    // Write to another file or b_flags set or not writing the whole file.
    // SAFETY: a live buffer.
    let contested = other
        || (!buf_is_nofilename(Some(buffer))
            && (buffer.b_flags.has(BufFlags::NOTEDITED)
                || buffer.b_flags.has(BufFlags::NEW) && !cpo_has(CpoFlag::OVERNEW)
                || buffer.b_flags.has(BufFlags::READERR)));
    // SAFETY: `ffname` is a live file name.
    if !contested || p_wa() || !unsafe { os_path_exists(ffname) } {
        return Ok(());
    }

    if !args.forceit && !args.append {
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
        args.forceit = true;
    }

    if !other || emsg_silent.get() != 0 {
        return Ok(());
    }

    // A swap file of the target's own would be silently orphaned by the
    // write, so it is worth a question of its own.
    let mut dir = swap_dir();
    // SAFETY: the names are live and `dir` is this call's own buffer.
    let swapname = Owned(unsafe {
        makeswapname(
            fname,
            ffname,
            Buf::current_or_none(),
            dir.as_mut_ptr().cast(),
        )
    });
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
    args.forceit = true;
    Ok(())
}

/// The first entry of 'directory', or `"."` when the option is empty -- where
/// `makeswapname` should look for the target's swap file.
///
/// NUL-terminated, and `MAXPATHL` long in the second case because that is the
/// room `copy_option_part` is told it has.
fn swap_dir() -> Vec<u8> {
    // SAFETY: 'directory' is a live option string.
    if p_dir(CStr::is_empty) {
        return b".\0".to_vec();
    }
    let mut first = vec![0u8; MAXPATHL as usize];
    // A copy: the cursor below walks past the end of a projection's borrow.
    let dir = P_DIR.get();
    let mut p = dir.as_ptr().cast_mut();
    // SAFETY: the buffer really is `MAXPATHL` bytes, and `p` walks the live
    // option string.
    unsafe {
        copy_option_part(
            &raw mut p,
            first.as_mut_ptr().cast(),
            MAXPATHL as usize,
            c",".as_ptr().cast_mut(),
        )
    };
    first
}

/// `:wnext`, `:wNext` and `:wprevious` -- write, then step through the
/// argument list.
pub fn ex_wnext(excmd: &mut ExArg) {
    let step = excmd.line2 as c_int;
    // SAFETY: the command name is at least two bytes long.
    let forwards = unsafe { *excmd.cmd.add(1) } as c_int == 'n' as c_int;
    let i = if forwards {
        Win::current().w_arg_idx + step
    } else {
        Win::current().w_arg_idx - step
    };
    excmd.line1 = 1;
    excmd.line2 = Buf::current().b_ml.ml_line_count;
    // SAFETY: main thread; the command block is the one borrowed here.
    if do_write(excmd).is_ok() {
        do_argfile(excmd, i);
    }
}

/// `:wall`, `:wqall` and `:xall`: write all changed files (and exit).
pub fn do_wqall(excmd: &mut ExArg) {
    let mut error = 0;
    let save_forceit = excmd.forceit;
    let save_exiting = exiting.get();

    if excmd.cmdidx == CmdIdx::xall || excmd.cmdidx == CmdIdx::wqall {
        // SAFETY: the command block is the one borrowed here.
        if before_quit_all(excmd).is_err() {
            return;
        }
        exiting.set(true);
    }

    // Not `winlayer::buffers()`: an autocommand fired while writing can delete
    // the buffer under the walk, which is what `WriteAll::Restart` is for --
    // the head has to be re-read, and no iterator re-reads it.
    let mut cur = first_buffer();
    while let Some(buf) = cur {
        match write_one_buffer(excmd, buf, save_forceit, &mut error) {
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
            getout(0);
        }
        not_exiting(save_exiting);
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
fn write_one_buffer(
    args: &mut ExArg,
    buffer: Buf,
    save_forceit: bool,
    error: &mut c_int,
) -> WriteAll {
    if exiting.get()
        && !args.forceit
        && !buffer.terminal.is_null()
        && channel_job_running(buffer.b_p_channel as u64)
    {
        no_write_message_buf(buffer);
        *error += 1;
    } else if !buf_is_changed(buffer) || buf_is_dontwrite(Some(buffer)) {
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
    if buffer.b_ffname.is_null() {
        semsg!("E141: No file name for buffer {}", buffer.handle as int64_t);
        *error += 1;
    } else if check_readonly(&mut args.forceit, buffer)
        || unsafe { check_overwrite(args, buffer, buffer.b_fname, buffer.b_ffname, false) }.is_err()
    {
        *error += 1;
    } else {
        let bufref = BufRef::of(buffer);
        if unsafe { handle_mkdir_p_arg(args, buffer.b_fname) }.is_err()
            || buf_write_all(buffer, args.forceit).is_err()
        {
            *error += 1;
        }
        // An autocommand may have deleted the buffer.
        deleted = !bufref.valid();
    }
    // check_overwrite() may set it
    args.forceit = save_forceit;
    if deleted {
        WriteAll::Restart
    } else {
        WriteAll::Next
    }
}

/// Check the 'write' option.
///
/// Returns true and gives a message when writing is disabled.
fn not_writing() -> bool {
    if p_write() {
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
/// `forceit` is set when the dialog overrules a read-only file.
fn check_readonly(forceit: &mut bool, buffer: Buf) -> bool {
    // Handle a file being readonly when the 'readonly' option is set or when
    // the file exists and permissions are read-only.
    let file = buffer.b_ffname;
    // SAFETY: the buffer's own file name.
    let readonly = !*forceit
        && (buffer.b_p_ro != 0
            || unsafe { os_path_exists(file) && os_file_is_writable(cstr::at(file)) == 0 });
    if !readonly {
        return false;
    }

    let (is_ro, name) = (buffer.b_p_ro != 0, buffer.b_fname);
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
    *forceit = true;
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
    lnum: LineNr,
    forceit: bool,
) -> c_int {
    if !check_can_set_curbuf_forceit(forceit as c_int) {
        return GETFILE_ERROR;
    }
    if text_locked() || curbuf_locked() {
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
        unsafe { fname_expand(&raw mut ffname, &raw mut sfname) };
        other = unsafe { otherfile(ffname) };
        free_me = Owned(ffname);
    } else {
        // SAFETY: `curbuf` is live.
        other = fnum != Buf::current().handle;
    }

    // Don't wait for the autowrite message. Released at two exits.
    let mut no_prompt = other.then(Suppress::wait_return);
    if other
        && !forceit
        && Buf::current().b_nwindows == 1
        && !buf_hide(Buf::current())
        && curbuf_is_changed()
        && autowrite(Buf::current(), forceit).is_err()
    {
        if p_confirm() && p_write() {
            dialog_changed(Buf::current(), false);
        }
        if curbuf_is_changed() {
            drop(no_prompt.take());
            // File has been changed.
            no_write_message();
            return GETFILE_NOT_WRITTEN;
        }
    }
    drop(no_prompt.take());
    if setpm {
        setpcmark();
    }

    if !other {
        // SAFETY: `curwin` is the live current window.
        if lnum != 0 {
            Win::current().w_cursor.lnum = lnum;
        }
        check_cursor_lnum(Win::current());
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
            None,
            lnum,
            EcmdFlags::HIDE.when(buf_hide(Buf::current())) | EcmdFlags::FORCEIT.when(forceit),
            Some(Win::current().id()),
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
