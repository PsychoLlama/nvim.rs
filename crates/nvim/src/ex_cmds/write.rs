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

#![forbid(unsafe_code)]

use super::{
    GETFILE_ERROR, GETFILE_NOT_WRITTEN, GETFILE_OPEN_OTHER, GETFILE_SAME_FILE, NODE_OTHER,
    VIM_QUESTION, VIM_YES, buf_autocmd, do_bang,
};
use crate::arglist::do_argfile;
use crate::autocmd::{augroup_exists_of, do_doautocmd};
use crate::buffer::{
    BufFlags, BufRef, buf_dontwrite_msg, buf_hide, buf_is_dontwrite, buf_is_nofilename,
    buf_name_changed, buflist_findname_of, current_buf, do_autochdir, do_modelines, fileinfo,
    no_write_message, no_write_message_buf, otherfile, setaltfname, setfname,
};
use crate::bufwrite::{WriteName, WriteRequest, buf_write};
use crate::channel::channel_job_running;
use crate::cursor::check_cursor_lnum;
use crate::drawscreen::state::redraw_tabline;
use crate::edit::{BeginlineOpts, beginline};
use crate::ex_cmds::EcmdFlags;
use crate::ex_cmds::ecmd::do_ecmd;
use crate::ex_cmds2::{autowrite, buf_write_all, check_fname, dialog_changed};
use crate::ex_docmd::{before_quit_all, cmdmod_has, dialog_msg, not_exiting};
use crate::ex_eval::aborting;
use crate::ex_getln::{curbuf_locked, text_locked};
use crate::guard::Suppress;
use crate::mark::setpcmark;
use crate::memline::swap_name_for;
use crate::memory::XString;
use crate::message::state::emsg_silent;
use crate::message::{e_argreq, e_bufloaded, e_exists, e_invarg, e_readonly};
use crate::message::{emsg, vim_dialog_yesno};
use crate::message_fmt::msg_cstr;
use crate::option::vars::{P_DIR, p_confirm, p_dir, p_wa, p_write};
use crate::option::{cpo_has, option_part, shortmess};
use crate::optionstr::OptString;
use crate::os::cshim::gettext;
use crate::os::fs::{file_mkdir, os_file_is_writable, os_isdir_of, os_nodetype, path_exists};
use crate::path::fixed_fname;
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
use core::ffi::c_int;
use std::ffi::CString;

/// Is a `:confirm` dialog wanted here -- either from 'confirm' or from the
/// command's own modifier?
fn confirming() -> bool {
    p_confirm() || cmdmod_has(CmdModFlags::CONFIRM)
}

/// Put `name` into the one-`%s` message `fmt` and ask the user to confirm it.
fn dialog_yesno_about(fmt: &CStr, name: Option<&CStr>) -> bool {
    let question = dialog_msg(fmt, name);
    vim_dialog_yesno(VIM_QUESTION as c_int, None, question.as_cstr(), 2) == VIM_YES as c_int
}

/// Give the current buffer the name `new_fname`, moving the old name into a
/// new unlisted buffer so that it becomes the alternate file.
pub fn rename_buffer(new_fname: &CStr) -> Result<(), Failed> {
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
    // The old name is taken *out* of the buffer rather than copied:
    // `setfname` would release what it found, and the entry made below
    // wants the name the buffer had. `saved` owns it until this returns.
    let saved = core::mem::take(&mut Buf::current().name);
    if setfname(Buf::current(), Some(new_fname), None, true).is_err() {
        Buf::current().name = saved;
        return Err(Failed);
    }
    Buf::current().b_flags |= BufFlags::NOTEDITED;
    if saved
        .shown()
        .is_some_and(|shown| !shown.to_bytes().is_empty())
    {
        // `saved` keeps both names alive across the call, which copies
        // whatever it keeps.
        let _ = setaltfname(saved.full(), saved.shown(), Win::current().w_cursor.lnum);
    }
    drop(saved);
    buf_autocmd(AutoEvent::BufFilePost, Buf::current());
    // Change directories when the 'acd' option is set.
    do_autochdir();
    Ok(())
}

/// `:file[!] [fname]`.
pub fn ex_file(excmd: &mut ExArg) {
    let no_arg = c_int::from(excmd.line.byte_at(excmd.line.arg)) == NUL;

    // ":0file" removes the file name.  Check for illegal uses ":3file",
    // "0file name", etc.
    if excmd.addr_count > 0 && (!no_arg || excmd.line2 > 0 || excmd.addr_count > 1) {
        emsg(gettext(e_invarg));
        return;
    }

    if !no_arg || excmd.addr_count == 1 {
        let name = CString::new(excmd.line.arg()).expect("an argument stops at its NUL");
        if rename_buffer(&name).is_err() {
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
    if curbuf_is_changed()
        || (!buf_is_nofilename(current_buf())
            && Buf::current()
                .name
                .full()
                .is_some_and(|name| !path_exists(name)))
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
fn check_writable(fname: &CStr) -> Result<(), Failed> {
    if os_nodetype(fname) == NODE_OTHER {
        let fname = msg_cstr(fname);
        semsg!("E503: \"{fname}\" is not a file or writable device");
        return Err(Failed);
    }
    Ok(())
}

/// `:write ++p` -- create the missing leading directories.
fn handle_mkdir_p_arg(args: &ExArg, fname: Option<&CStr>) -> Result<(), Failed> {
    if args.mkdir_p && !file_mkdir(fname.unwrap_or_default(), 0o755 as int32_t) {
        return Err(Failed);
    }
    Ok(())
}

/// One of the current buffer's names, copied: a callee that can run user
/// code (a dialog, an autocommand) may rename the buffer under a borrow.
fn own_name(name: Option<&CStr>) -> Option<CString> {
    name.map(CStr::to_owned)
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

    // The name typed, and its full form.  When out-of-memory, keep the
    // unexpanded file name, because we MUST be able to write the file in
    // this situation.
    let typed: Option<CString> = if args.line.arg().is_empty() {
        if args.cmdidx == CmdIdx::saveas {
            emsg(gettext(e_argreq));
            return Err(Failed);
        }
        None
    } else {
        Some(CString::new(args.line.arg()).expect("an argument stops at its NUL"))
    };
    let full: Option<XString> = typed
        .as_deref()
        .map(|typed| fixed_fname(typed).unwrap_or_else(|| XString::from_cstr(typed)));
    let other = full.as_ref().is_some_and(|full| otherfile(full.as_cstr()));

    // If we have a new file, put its name in the list of alternate file names.
    let mut alt_buf = None;
    if other {
        let (full, typed) = (full.as_ref().map(XString::as_cstr), typed.as_deref());
        alt_buf = if cpo_has(CpoFlag::ALTWRITE) || args.cmdidx == CmdIdx::saveas {
            setaltfname(full, typed, 1)
        } else {
            full.and_then(buflist_findname_of)
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
        if !confirm_partial_write(args) {
            return Err(Failed);
        }
    }

    // The two names to check and write under: the ones typed, or the
    // buffer's own -- named rather than borrowed for the write, which may
    // rename the buffer, and copied for the checks, whose dialogs can too.
    let (ffname, fname) = if other {
        (
            full.as_ref().map(|full| full.as_cstr().to_owned()),
            typed.clone(),
        )
    } else {
        let name = &Buf::current().name;
        (own_name(name.full()), own_name(name.shown()))
    };
    check_overwrite(
        args,
        Buf::current(),
        fname.as_deref(),
        ffname.as_deref(),
        other,
    )?;

    let mut short = if other {
        typed.as_deref().map(WriteName::Other)
    } else {
        Some(WriteName::Shown)
    };
    let mut mkdir_name = fname;
    if args.cmdidx == CmdIdx::saveas
        && let Some(alt_buf) = alt_buf
    {
        if !saveas_exchange_names(alt_buf) {
            return Err(Failed);
        }
        // The current buffer now goes by the name typed.
        short = Some(WriteName::Short);
        mkdir_name = own_name(Buf::current().name.short());
    }

    handle_mkdir_p_arg(args, mkdir_name.as_deref())?;

    let name_was_missing = Buf::current().name.full().is_none();
    let request = WriteRequest {
        append: args.append,
        forceit: args.forceit,
        reset_changed: true,
        filtering: false,
    };
    let (line1, line2) = (args.line1, args.line2);
    let target = if other {
        full.as_ref()
            .map_or(WriteName::Full, |full| WriteName::Other(full.as_cstr()))
    } else {
        WriteName::Full
    };
    let retval = buf_write(
        Buf::current(),
        target,
        short,
        line1,
        line2,
        Some(args),
        request,
    );

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
    buf_dontwrite_msg(current_buf())
        || check_fname().is_err()
        || check_writable(
            own_name(Buf::current().name.full())
                .as_deref()
                .unwrap_or_default(),
        )
        .is_err()
        || check_readonly(&mut args.forceit, Buf::current())
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
    if vim_dialog_yesno(
        VIM_QUESTION as c_int,
        None,
        gettext(c"Write partial file?"),
        2,
    ) != VIM_YES as c_int
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
/// Answers false when an autocommand changed the current buffer or aborted
/// the script.
///
/// Safe: [`Buf`] is the live buffer this needs; `alt_buf` is expected to be a
/// buffer other than the current one, which is a matter of sense rather than
/// of soundness -- swapping a buffer's names with its own is a no-op.
fn saveas_exchange_names(mut alt_buf: Buf) -> bool {
    let was_curbuf = Buf::current_raw();
    buf_autocmd(AutoEvent::BufFilePre, Buf::current());
    buf_autocmd(AutoEvent::BufFilePre, alt_buf);
    // buffer changed, don't change name now
    if Buf::current_raw() != was_curbuf || aborting() {
        return false;
    }

    // Exchange the file names for the current and the alternate buffer --
    // one move now that a buffer's three names are one value. The guard is
    // the doc comment above: `alt_buf` is expected to be another buffer,
    // and swapping a place with itself is not a swap.
    if alt_buf.raw() != Buf::current_raw() {
        core::mem::swap(&mut alt_buf.name, &mut Buf::current().name);
    }
    buf_name_changed(Buf::current());
    buf_autocmd(AutoEvent::BufFilePost, Buf::current());
    buf_autocmd(AutoEvent::BufFilePost, alt_buf);
    if alt_buf.b_p_bl == 0 {
        alt_buf.b_p_bl = 1;
        buf_autocmd(AutoEvent::BufAdd, alt_buf);
    }
    // buffer changed, don't write the file
    if Buf::current_raw() != was_curbuf || aborting() {
        return false;
    }

    // If 'filetype' was empty try detecting it now.
    if Buf::current().b_p_ft.first_byte() as c_int == NUL {
        if augroup_exists_of(c"filetypedetect") {
            let _ = do_doautocmd(c"filetypedetect BufRead", true, None);
        }
        do_modelines(OptionSetFlags::NONE);
    }
    // Autocommands may have changed buffer names, esp. when 'autochdir'
    // is set: the caller reads the short name after this.
    true
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
pub fn check_overwrite(
    args: &mut ExArg,
    buffer: Buf,
    fname: Option<&CStr>,
    ffname: Option<&CStr>,
    other: bool,
) -> Result<(), Failed> {
    // Write to another file or b_flags set or not writing the whole file.
    let contested = other
        || (!buf_is_nofilename(Some(buffer))
            && (buffer.b_flags.has(BufFlags::NOTEDITED)
                || buffer.b_flags.has(BufFlags::NEW) && !cpo_has(CpoFlag::OVERNEW)
                || buffer.b_flags.has(BufFlags::READERR)));
    let ffname = ffname.unwrap_or_default();
    if !contested || p_wa() || !path_exists(ffname) {
        return Ok(());
    }

    if !args.forceit && !args.append {
        if os_isdir_of(ffname) {
            let ffname = msg_cstr(ffname);
            semsg!("E17: \"{ffname}\" is a directory");
            return Err(Failed);
        }
        if !confirming() {
            emsg(gettext(e_exists));
            return Err(Failed);
        }
        if !dialog_yesno_about(gettext(c"Overwrite existing file \"%s\"?"), fname) {
            return Err(Failed);
        }
        args.forceit = true;
    }

    if !other || emsg_silent.get() != 0 {
        return Ok(());
    }

    // A swap file of the target's own would be silently orphaned by the
    // write, so it is worth a question of its own.
    let dir = swap_dir();
    let Some(swapname) = swap_name_for(fname.unwrap_or_default(), ffname, &dir) else {
        return Ok(());
    };
    if !path_exists(swapname.as_cstr()) {
        return Ok(());
    }
    if !confirming() {
        let arg0 = msg_cstr(swapname.as_cstr());
        semsg!("E768: Swap file exists: {arg0} (:silent! overrides)");
        return Err(Failed);
    }
    let name = Some(swapname.as_cstr());
    if !dialog_yesno_about(gettext(c"Swap file \"%s\" exists, overwrite anyway?"), name) {
        return Err(Failed);
    }
    args.forceit = true;
    Ok(())
}

/// [`check_overwrite`] of `buffer` under its own names, copied for the
/// dialogs, which can rename it.
pub(crate) fn check_overwrite_own(args: &mut ExArg, buffer: Buf) -> Result<(), Failed> {
    let (shown, full) = (own_name(buffer.name.shown()), own_name(buffer.name.full()));
    check_overwrite(args, buffer, shown.as_deref(), full.as_deref(), false)
}

/// The first entry of 'directory', or `"."` when the option is empty -- where
/// `makeswapname` should look for the target's swap file, cut to the
/// `MAXPATHL` room `copy_option_part` is given.
fn swap_dir() -> CString {
    if p_dir(CStr::is_empty) {
        return c".".to_owned();
    }
    let dir = P_DIR.get();
    let (first, _) = option_part(dir.as_cstr(), MAXPATHL as usize, c",");
    CString::new(first).expect("an option part stops at its NUL")
}

/// `:wnext`, `:wNext` and `:wprevious` -- write, then step through the
/// argument list.
pub fn ex_wnext(excmd: &mut ExArg) {
    let step = excmd.line2 as c_int;
    let forwards = c_int::from(excmd.line.byte_at(excmd.line.cmd + 1)) == 'n' as c_int;
    let i = if forwards {
        Win::current().w_arg_idx + step
    } else {
        Win::current().w_arg_idx - step
    };
    excmd.line1 = 1;
    excmd.line2 = Buf::current().b_ml.ml_line_count;
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
    if buffer.name.full().is_none() {
        semsg!("E141: No file name for buffer {}", buffer.handle as int64_t);
        *error += 1;
    } else if check_readonly(&mut args.forceit, buffer)
        || check_overwrite_own(args, buffer).is_err()
    {
        *error += 1;
    } else {
        let bufref = BufRef::of(buffer);
        if handle_mkdir_p_arg(args, own_name(buffer.name.shown()).as_deref()).is_err()
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
    let file = own_name(buffer.name.full());
    let file = file.as_deref().unwrap_or_default();
    let readonly =
        !*forceit && (buffer.b_p_ro != 0 || path_exists(file) && os_file_is_writable(file) == 0);
    if !readonly {
        return false;
    }

    let (is_ro, name) = (buffer.b_p_ro != 0, own_name(buffer.name.shown()));
    if !confirming() || name.is_none() {
        if is_ro {
            emsg(gettext(e_readonly));
        } else {
            let name = msg_cstr(name.as_deref().unwrap_or_default());
            semsg!("E505: \"{name}\" is read-only (add ! to override)");
        }
        return true;
    }

    let prompt = if is_ro {
        c"'readonly' option is set for \"%s\".\nDo you wish to write anyway?"
    } else {
        c"File permissions of \"%s\" are read-only.\nIt may still be possible to write it.\nDo you wish to try?"
    };
    if !dialog_yesno_about(gettext(prompt), name.as_deref()) {
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
pub fn getfile(
    fnum: c_int,
    ffname_arg: Option<&CStr>,
    sfname_arg: Option<&CStr>,
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

    // `fname_expand`: the full name made from the one given, and the short
    // name -- the one given, unless there was another.
    let mut full: Option<XString> = None;
    let mut sfname = sfname_arg;
    let other = if fnum == 0 {
        // make ffname full path, set sfname
        if let Some(name) = ffname_arg {
            sfname = sfname.or(Some(name));
            full = fixed_fname(name);
        }
        otherfile(full.as_ref().map_or(c"", XString::as_cstr))
    } else {
        fnum != Buf::current().handle
    };

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
        if lnum != 0 {
            Win::current().w_cursor.lnum = lnum;
        }
        check_cursor_lnum(Win::current());
        beginline(BeginlineOpts::SOL | BeginlineOpts::FIX);
        // it's in the same file
        return GETFILE_SAME_FILE;
    }

    let ffname = if fnum == 0 {
        full.as_ref().map(XString::as_cstr)
    } else {
        ffname_arg
    };
    let opened = do_ecmd(
        fnum,
        ffname,
        sfname,
        None,
        lnum,
        EcmdFlags::HIDE.when(buf_hide(Buf::current())) | EcmdFlags::FORCEIT.when(forceit),
        Some(Win::current().id()),
    )
    .is_ok();
    if opened {
        // opened another file
        GETFILE_OPEN_OTHER
    } else {
        // error encountered
        GETFILE_ERROR
    }
}
