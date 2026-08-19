//! Noticing that a file changed underneath us.
//!
//! [`check_timestamps`] sweeps every buffer whenever Nvim regains focus or
//! returns to the main loop; [`buf_check_timestamp`] is the per-buffer test
//! that compares the file's mtime, size and mode against what was recorded
//! when it was read, asks the user (or `FileChangedShell`) what to do about
//! it, and [`buf_reload`] carries out a reload — moving the old lines into a
//! scratch buffer first, so that they can be put back if the re-read fails.

#![deny(unsafe_op_in_unsafe_fn)]

use crate::buffer::BufFlags;
use crate::semsg_c;
use crate::undo::UNDO_HASH_SIZE;
use core::ffi::{c_char, c_int};
use std::ffi::CStr;

use super::*;
use crate::highlight_group::{HLF_E, HLF_W};
use crate::types::{FAIL, OK, ShmFlag, Vv};

/// Has a warning already been shown this sweep? Only one is worth reading.
static ALREADY_WARNED: GlobalCell<bool> = GlobalCell::new(false);

/// Set while `FileChangedShell` runs, so that `buf_check_timestamp` does not
/// re-enter itself from an autocommand.
static BUSY: GlobalCell<bool> = GlobalCell::new(false);

/// `gettext` on a literal, keeping the result a `&'static CStr`.
///
/// It returns either its argument or a string from the message catalogue,
/// both of which live as long as the process.
macro_rules! translate {
    ($msg:literal $(,)?) => {
        CStr::from_ptr(gettext($msg.as_ptr()))
    };
}

/// Why a buffer's file no longer matches what was read from it.
///
/// Upstream carries the name as a string and dispatches on its characters —
/// `reason[2] == 'n'` for "conflict", `reason[1] == 'h'` for "changed" — which
/// picks out exactly these five.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Reason {
    /// The file is gone.
    Deleted,
    /// It changed on disk, and the buffer was changed here too.
    Conflict,
    /// Its contents changed on disk.
    Changed,
    /// Only its mode changed.
    Mode,
    /// Only its timestamp changed.
    Time,
}

impl Reason {
    /// What `v:fcs_reason` is set to.
    fn name(self) -> &'static CStr {
        match self {
            Reason::Deleted => c"deleted",
            Reason::Conflict => c"conflict",
            Reason::Changed => c"changed",
            Reason::Mode => c"mode",
            Reason::Time => c"time",
        }
    }
}

/// Whether, and how, to re-read the file.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Reload {
    No,
    /// Re-read the text, keeping the buffer's options.
    Text,
    /// Re-read the file as if it were being edited afresh, so that
    /// `'fileformat'`, `'fileencoding'` and `'filetype'` are detected again.
    Detect,
}

/// What `FileChangedShell` decided.
enum Fcs {
    /// Nothing handled it, or something asked for the usual prompt.
    Ask,
    /// It said to reload, and said so in `v:fcs_choice`.
    Reload(Reload),
    /// It handled the event itself; upstream counts that as a message shown.
    Handled,
}

/// Has the file's modification time moved since it was read?
///
/// On a FAT filesystem, especially under Linux, there are only 5 bits to
/// store the seconds; the round-off happens when the inode is flushed, so
/// the time can change unexpectedly by one second.
pub fn time_differs(file_info: &FileInfo, mtime: i64, mtime_ns: i64) -> bool {
    file_info.stat.st_mtim.tv_nsec != mtime_ns
        || file_info.stat.st_mtim.tv_sec - mtime > 1
        || mtime - file_info.stat.st_mtim.tv_sec > 1
}

/// Check whether any non-hidden buffer has been changed.
///
/// The check is postponed if there are characters in the stuff buffer, a
/// global command is being executed, a mapping is being executed, or an
/// autocommand is busy.
///
/// @param focus  called for a GUI focus event
///
/// @return  true if a message was written, so the screen should be redrawn
///          and the cursor positioned.
pub unsafe fn check_timestamps(focus: c_int) -> c_int {
    unsafe {
        // Don't check timestamps while system() or another low-level function
        // may cause us to lose and gain focus.
        if no_check_timestamps.get() > 0 {
            return false as c_int;
        }

        // Avoid doing a check twice. The OK/Reload dialog can cause a focus
        // event, and we would keep on checking if the file were steadily
        // growing. Do check again after typing something.
        if focus != 0 && did_check_timestamps.get() {
            need_check_timestamps.set(true);
            return false as c_int;
        }

        if !stuff_empty()
            || global_busy.get() != 0
            || typebuf_typed() == 0
            || autocmd_busy.get()
            || (*curbuf.get()).b_ro_locked > 0
            || allbuf_lock.get() > 0
        {
            need_check_timestamps.set(true); // check later
            return 0;
        }

        let mut didit = 0;
        (*no_wait_return.ptr()) += 1;
        did_check_timestamps.set(true);
        ALREADY_WARNED.set(false);

        let mut buf = firstbuf.get();
        while !buf.is_null() {
            // Only check buffers in a window.
            if (*buf).b_nwindows > 0 {
                let mut bufref = bufref_T::default();
                set_bufref(&raw mut bufref, buf);
                let n = buf_check_timestamp(buf);
                didit = didit.max(n);
                if n > 0 && !bufref_valid(&raw mut bufref) {
                    // Autocommands have removed the buffer. Upstream's
                    // `buf = firstbuf; continue;` still runs the loop's own
                    // step, so this restarts at the *second* buffer.
                    buf = firstbuf.get();
                }
            }
            buf = (*buf).b_next;
        }

        (*no_wait_return.ptr()) -= 1;
        need_check_timestamps.set(false);
        if need_wait_return.get() && didit == 2 {
            // Make sure the message isn't overwritten.
            msg_puts(c"\n".as_ptr());
            ui_flush();
        }
        didit
    }
}

/// Move all the lines from buffer `frombuf` to buffer `tobuf`.
///
/// @return  OK or FAIL. On FAIL `tobuf` is incomplete and/or `frombuf` is not
///          empty.
unsafe fn move_lines(frombuf: *mut buf_T, tobuf: *mut buf_T) -> c_int {
    unsafe {
        let tbuf = curbuf.get();
        let mut retval = OK;

        // Copy the lines in "frombuf" to "tobuf".
        curbuf.set(tobuf);
        let mut lnum = 1;
        while lnum <= (*frombuf).b_ml.ml_line_count {
            let p = xmemdupz(
                ml_get_buf(frombuf, lnum).cast(),
                ml_get_buf_len(frombuf, lnum) as size_t,
            )
            .cast::<c_char>();
            let appended = ml_append(lnum - 1, p, 0, false);
            xfree(p.cast());
            if appended == FAIL {
                retval = FAIL;
                break;
            }
            lnum += 1;
        }

        // Delete all the lines in "frombuf".
        if retval != FAIL {
            curbuf.set(frombuf);
            let mut lnum = (*curbuf.get()).b_ml.ml_line_count;
            while lnum > 0 {
                if ml_delete(lnum) == FAIL {
                    // Oops! We could try putting back the saved lines, but
                    // that might fail again...
                    retval = FAIL;
                    break;
                }
                lnum -= 1;
            }
        }

        curbuf.set(tbuf);
        retval
    }
}

/// Give `FileChangedShell` the chance to handle the change itself.
///
/// Sets `v:fcs_reason` and clears `v:fcs_choice` first, and reads the latter
/// back afterwards.
unsafe fn file_changed_shell(buf: *mut buf_T, bufref: *mut bufref_T, reason: Reason) -> Fcs {
    unsafe {
        let name = reason.name();
        BUSY.set(true);
        set_vim_var_string(
            Vv::FcsReason,
            name.as_ptr(),
            name.count_bytes() as ptrdiff_t,
        );
        set_vim_var_string(Vv::FcsChoice, c"".as_ptr(), 0);
        (*allbuf_lock.ptr()) += 1;
        let handled = apply_autocmds(
            EVENT_FILECHANGEDSHELL,
            (*buf).b_fname,
            (*buf).b_fname,
            false,
            buf,
        );
        (*allbuf_lock.ptr()) -= 1;
        BUSY.set(false);

        if !handled {
            return Fcs::Ask;
        }
        if !bufref_valid(bufref) {
            emsg(gettext(
                c"E246: FileChangedShell autocommand deleted buffer".as_ptr(),
            ));
        }
        match CStr::from_ptr(get_vim_var_str(Vv::FcsChoice)).to_bytes() {
            b"reload" if reason != Reason::Deleted => Fcs::Reload(Reload::Text),
            b"edit" => Fcs::Reload(Reload::Detect),
            b"ask" => Fcs::Ask,
            // Note that "reload" on a deleted file lands here, not on `ask`.
            _ => Fcs::Handled,
        }
    }
}

/// Tell the user their file changed, and possibly offer to reload it.
///
/// `mesg` is a format string taking the file name; `mesg2` is the "see
/// `:help`" note the warnings carry, appended to it. Returns what the user
/// chose, and whether a message was displayed (which is `buf_check_timestamp`
/// returning 2).
unsafe fn warn_changed(
    buf: *mut buf_T,
    mesg: &CStr,
    mesg2: &CStr,
    can_reload: bool,
) -> (Reload, bool) {
    unsafe {
        let path = home_replace_save(buf, (*buf).b_fname);
        // +2 for either '\n' or "; " and +1 for NUL.
        let size = strlen(path) + mesg.count_bytes() + mesg2.count_bytes() + 3;
        let mut tbuf = vec![0 as c_char; size];
        let at = snprintf(tbuf.as_mut_ptr(), size, mesg.as_ptr(), path) as usize;
        xfree(path.cast());
        // Set v:warningmsg here, before the unimportant and output-specific
        // `mesg2` has been appended.
        set_vim_var_string(Vv::Warningmsg, tbuf.as_ptr(), at as ptrdiff_t);
        let mut append = |sep: &CStr| {
            if !mesg2.is_empty() {
                snprintf(
                    tbuf.as_mut_ptr().add(at),
                    size - at,
                    sep.as_ptr(),
                    mesg2.as_ptr(),
                );
            }
        };

        if can_reload {
            append(c"\n%s");
            return (
                match do_dialog(
                    VIM_WARNING as c_int,
                    gettext(c"Warning".as_ptr()),
                    tbuf.as_ptr(),
                    gettext(c"&OK\n&Load File\nLoad File &and Options".as_ptr()),
                    1,
                    core::ptr::null(),
                    true as c_int,
                ) {
                    2 => Reload::Text,
                    3 => Reload::Detect,
                    _ => Reload::No,
                },
                false,
            );
        }

        if State.get() > MODE_NORMAL_BUSY || State.get() & MODE_CMDLINE != 0 || ALREADY_WARNED.get()
        {
            append(c"; %s");
            emsg(tbuf.as_ptr());
            return (Reload::No, true);
        }

        if !autocmd_busy.get() {
            msg_start();
            msg_puts_hl(tbuf.as_ptr(), HLF_E, true);
            if !mesg2.is_empty() {
                msg_puts_hl(mesg2.as_ptr(), HLF_W, true);
            }
            msg_clr_eos();
            msg_end();
            if emsg_silent.get() == 0 && !in_assert_fails.get() && !ui_has(kUIMessages) {
                msg_delay(1004, true); // give the user some time to think about it
                redraw_cmdline.set(false); // don't redraw and erase the message
            }
        }
        ALREADY_WARNED.set(true);
        (Reload::No, false)
    }
}

/// Check whether buffer `buf` has been changed, or whether the file for a new
/// buffer unexpectedly appeared.
///
/// @return  1 if a changed buffer was found, 2 if a message has been
///          displayed, 0 otherwise.
pub unsafe fn buf_check_timestamp(buf: *mut buf_T) -> c_int {
    unsafe {
        let orig_size = (*buf).b_orig_size;
        let orig_mode = (*buf).b_orig_mode;

        let mut bufref = bufref_T::default();
        set_bufref(&raw mut bufref, buf);

        // If it's a terminal, there is no file name, the buffer is not
        // loaded, 'buftype' is set, we are in the middle of a save, or we are
        // being called recursively: ignore this buffer.
        if !(*buf).terminal.is_null()
            || (*buf).b_ffname.is_null()
            || (*buf).b_ml.ml_mfp.is_null()
            || !bt_normal(buf)
            || (*buf).b_saving
            || BUSY.get()
        {
            return 0;
        }

        let mut retval = 0;
        let mut reload = Reload::No;
        let mut can_reload = false;
        // The message to show, as a format string taking the file name, plus
        // the "see :help Wnn" note that only the warnings carry.
        let mut mesg: Option<&CStr> = None;
        let mut mesg2 = c"";

        let mut file_info = FileInfo::default();
        let mut file_info_ok = false;
        let differs = !(*buf).b_flags.has(BufFlags::NOTEDITED) && (*buf).b_mtime != 0 && {
            file_info_ok = os_fileinfo((*buf).b_ffname, &raw mut file_info);
            !file_info_ok
                || time_differs(&file_info, (*buf).b_mtime, (*buf).b_mtime_ns)
                || file_info.stat.st_mode as c_int != (*buf).b_orig_mode
        };

        if differs {
            let prev_b_mtime = (*buf).b_mtime;
            retval = 1;

            // Set b_mtime to stop further warnings, e.g. while executing a
            // FileChangedShell autocommand.
            if file_info_ok {
                buf_store_file_info(buf, &raw mut file_info);
            } else {
                // Check the file again later to see if it re-appears.
                (*buf).b_mtime = -1;
                (*buf).b_orig_size = 0;
                (*buf).b_orig_mode = 0;
            }

            // Don't do anything for a directory. It might contain the file
            // explorer.
            if os_isdir((*buf).b_fname) {
                // Nothing to do.
            } else if (if (*buf).b_p_ar >= 0 {
                (*buf).b_p_ar
            } else {
                p_ar.get()
            }) != 0
                && !bufIsChanged(buf)
                && file_info_ok
            {
                // If 'autoread' is set, the buffer has no changes and the file
                // still exists, reload the buffer. Use the buffer-local option
                // value if it was set, the global value otherwise.
                reload = Reload::Text;
            } else {
                let reason = if !file_info_ok {
                    Reason::Deleted
                } else if bufIsChanged(buf) {
                    Reason::Conflict
                } else if orig_size != (*buf).b_orig_size || buf_contents_changed(buf) {
                    Reason::Changed
                } else if orig_mode != (*buf).b_orig_mode {
                    Reason::Mode
                } else {
                    Reason::Time
                };

                // Only warn if no FileChangedShell autocommand handled it.
                match file_changed_shell(buf, &raw mut bufref, reason) {
                    Fcs::Handled => return 2,
                    Fcs::Reload(what) => reload = what,
                    Fcs::Ask => match reason {
                        Reason::Deleted => {
                            // Only give the message once.
                            if prev_b_mtime != -1 {
                                mesg = Some(translate!(c"E211: File \"%s\" no longer available"));
                            }
                        }
                        _ => {
                            can_reload = true;
                            // Check whether the file contents really changed,
                            // to avoid warning when only the timestamp was set
                            // (e.g. checked out of CVS). Always warn when the
                            // buffer was changed too.
                            match reason {
                                Reason::Conflict => {
                                    mesg = Some(translate!(c"W12: Warning: File \"%s\" has changed and the buffer was changed in Vim as well"));
                                    mesg2 = translate!(c"See \":help W12\" for more info.");
                                }
                                Reason::Changed => {
                                    mesg = Some(translate!(c"W11: Warning: File \"%s\" has changed since editing started"));
                                    mesg2 = translate!(c"See \":help W11\" for more info.");
                                }
                                Reason::Mode => {
                                    mesg = Some(translate!(c"W16: Warning: Mode of file \"%s\" has changed since editing started"));
                                    mesg2 = translate!(c"See \":help W16\" for more info.");
                                }
                                _ => {
                                    // Only the timestamp changed. Store it, to
                                    // avoid a warning in check_mtime() later.
                                    (*buf).b_mtime_read = (*buf).b_mtime;
                                    (*buf).b_mtime_read_ns = (*buf).b_mtime_ns;
                                }
                            }
                        }
                    },
                }
            }
        } else if (*buf).b_flags.has(BufFlags::NEW)
            && !(*buf).b_flags.has(BufFlags::NEW_W)
            && os_path_exists((*buf).b_ffname)
        {
            retval = 1;
            mesg = Some(translate!(
                c"W13: Warning: File \"%s\" has been created after editing started",
            ));
            (*buf).b_flags |= BufFlags::NEW_W;
            can_reload = true;
        }

        if let Some(mesg) = mesg {
            let (chose, displayed) = warn_changed(buf, mesg, mesg2, can_reload);
            if chose != Reload::No {
                reload = chose;
            }
            if displayed {
                retval = 2;
            }
        }

        if reload != Reload::No {
            buf_reload(buf, orig_mode, reload == Reload::Detect);
            if bufref_valid(&raw mut bufref) && (*buf).b_p_udf != 0 && !(*buf).b_ffname.is_null() {
                // Any existing undo file is unusable, write it now.
                let mut hash = [0u8; UNDO_HASH_SIZE as usize];
                u_compute_hash(buf, hash.as_mut_ptr());
                u_write_undo(core::ptr::null(), false, buf, hash.as_mut_ptr());
            }
        }

        // Trigger FileChangedShellPost when the file was changed in any way.
        if bufref_valid(&raw mut bufref) && retval != 0 {
            apply_autocmds(
                EVENT_FILECHANGEDSHELLPOST,
                (*buf).b_fname,
                (*buf).b_fname,
                false,
                buf,
            );
        }
        retval
    }
}

/// Reload a buffer that is already loaded, because the file changed outside
/// of Nvim.
///
/// @param orig_mode       `buf->b_orig_mode` from before the need for
///                        reloading was detected; it may have been reset by
///                        now.
/// @param reload_options  re-detect `'fileformat'`, `'fileencoding'` and
///                        `'filetype'`, rather than forcing the ones the
///                        buffer already has.
pub unsafe fn buf_reload(buf: *mut buf_T, orig_mode: c_int, reload_options: bool) {
    unsafe {
        let old_ro = (*buf).b_p_ro;
        let mut saved = OK;
        let mut flags = READ_NEW as c_int;

        // Set curwin/curbuf for "buf" and save some things.
        let mut aco = aco_save_T::default();
        aucmd_prepbuf(&raw mut aco, buf);

        // Unless reload_options is set we only want to read the text from the
        // file, not reset the syntax highlighting, clear marks, diff status
        // and so on. Force the fileformat and encoding to be the same.
        let mut ea = exarg_T::default();
        if !reload_options {
            prep_exarg(&raw mut ea, buf);
        }

        let old_cursor = (*curwin.get()).w_cursor;
        let old_topline = (*curwin.get()).w_topline;

        if p_ur.get() < 0 || (*curbuf.get()).b_ml.ml_line_count as OptInt <= p_ur.get() {
            // Save all the text, so that the reload can be undone. Sync first
            // so that this is a separate undo-able action.
            u_sync(false);
            saved = u_savecommon(
                curbuf.get(),
                0,
                (*curbuf.get()).b_ml.ml_line_count + 1,
                0,
                true,
            );
            flags |= READ_KEEP_UNDO as c_int;
        }

        // To behave like when a new file is edited (which matters for
        // BufReadPost autocommands) we first need to delete the current buffer
        // contents. But if reading the file fails we should keep the old
        // contents. Memory alone will not do, the file might be too big, so
        // move the buffer contents to a hidden buffer.
        let mut savebuf = core::ptr::null_mut::<buf_T>();
        let mut bufref = bufref_T::default();
        if !(buf_is_empty(curbuf.get()) || saved == FAIL) {
            // Allocate a buffer without putting it in the buffer list.
            savebuf = buflist_new(
                core::ptr::null_mut(),
                core::ptr::null_mut(),
                1,
                BLN_DUMMY as c_int,
            );
            set_bufref(&raw mut bufref, savebuf);
            if !savebuf.is_null() && buf == curbuf.get() {
                // Open the memline.
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
                semsg_c!(
                    gettext(c"E462: Could not prepare for reloading \"%s\"".as_ptr()),
                    (*buf).b_fname,
                );
                saved = FAIL;
            }
        }

        if saved == OK {
            (*curbuf.get()).b_flags |= BufFlags::CHECK_RO; // check for RO again
            (*curbuf.get()).b_keep_filetype = true; // don't detect 'filetype'
            if readfile(
                (*buf).b_ffname,
                (*buf).b_fname,
                0,
                0,
                MAXLNUM as linenr_T,
                &raw mut ea,
                flags,
                shortmess(ShmFlag::FILEINFO),
            ) != OK
            {
                if !aborting() {
                    semsg_c!(
                        gettext(c"E321: Could not reload \"%s\"".as_ptr()),
                        (*buf).b_fname,
                    );
                }
                if !savebuf.is_null() && bufref_valid(&raw mut bufref) && buf == curbuf.get() {
                    // Put the text back from the save buffer. First delete any
                    // lines that readfile() added.
                    while !buf_is_empty(curbuf.get()) {
                        if ml_delete((*buf).b_ml.ml_line_count) == FAIL {
                            break;
                        }
                    }
                    move_lines(savebuf, buf);
                }
            } else if buf == curbuf.get() {
                // "buf" is still valid. Mark the buffer as unmodified and free
                // the undo info.
                unchanged(buf, true, true);
                if flags & READ_KEEP_UNDO as c_int == 0 {
                    u_clearallandblockfree(buf);
                } else {
                    // Mark all undo states as changed.
                    u_unchanged(curbuf.get());
                }
                buf_updates_unload(curbuf.get(), true);
                (*curbuf.get()).b_mod_set = true;
            }
        }
        xfree(ea.cmd.cast());

        if !savebuf.is_null() && bufref_valid(&raw mut bufref) {
            wipe_buffer(savebuf, false);
        }

        // Invalidate diff info if necessary.
        diff_invalidate(curbuf.get());

        // Restore the topline and cursor position and check them; lines may
        // have been removed.
        (*curwin.get()).w_topline = old_topline.min((*curbuf.get()).b_ml.ml_line_count);
        (*curwin.get()).w_cursor = old_cursor;
        check_cursor(curwin.get());
        update_topline(curwin.get());
        (*curbuf.get()).b_keep_filetype = false;

        // Update folds unless they are defined manually.
        let mut tp = first_tabpage.get();
        while !tp.is_null() {
            let mut wp = if tp == curtab.get() {
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
            tp = (*tp).tp_next;
        }

        // If the mode didn't change and 'readonly' was set, keep the old
        // value; the user probably used the ":view" command. But don't reset
        // it, there might have been a read error.
        if orig_mode == (*curbuf.get()).b_orig_mode {
            (*curbuf.get()).b_p_ro |= old_ro;
        }

        // Modelines must override settings done by autocommands.
        do_modelines(OptionSetFlags::NONE);

        // Restore curwin/curbuf and a few other things. Careful: autocommands
        // may have made "buf" invalid!
        aucmd_restbuf(&raw mut aco);
    }
}

/// Record the file's size, mode and modification time on the buffer, so that
/// a later change to any of them can be noticed.
pub unsafe fn buf_store_file_info(buf: *mut buf_T, file_info: *mut FileInfo) {
    unsafe {
        (*buf).b_mtime = (*file_info).stat.st_mtim.tv_sec as int64_t;
        (*buf).b_mtime_ns = (*file_info).stat.st_mtim.tv_nsec as int64_t;
        (*buf).b_orig_size = os_fileinfo_size(file_info);
        (*buf).b_orig_mode = (*file_info).stat.st_mode as c_int;
    }
}

/// Adjust the line with a missing end-of-line, used for the next write.
///
/// Needed by `do_filter()`, where the input lines for the filter are deleted.
pub unsafe fn write_lnum_adjust(offset: linenr_T) {
    unsafe {
        if (*curbuf.get()).b_no_eol_lnum != 0 {
            // Only if there is a missing end-of-line.
            (*curbuf.get()).b_no_eol_lnum += offset;
        }
    }
}
