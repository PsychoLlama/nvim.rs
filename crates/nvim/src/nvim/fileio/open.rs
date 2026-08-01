//! Getting to the point where a file's bytes can be read.
//!
//! [`open_source`] is everything `readfile` does before the first `read()`:
//! letting `BufReadCmd`/`FileReadCmd` take over entirely, refusing names that
//! are too long or name a directory, working out whether the buffer is
//! read-only, recording the file's timestamp, opening it, making the swap
//! file, and running the `*ReadPre` autocommands — which may have changed
//! everything, so most of the work is checking that they did not.
//!
//! `prep_exarg`, `set_file_options` and `set_forced_fenc` are the `++opt`
//! side of the same question: what the caller has already decided about
//! `'fileformat'`, `'fileencoding'` and `'binary'`.

#![deny(unsafe_op_in_unsafe_fn)]

use core::ffi::{c_char, c_int};

use crate::src::nvim::bufwrite::translate;
use crate::src::nvim::memfile::mf_fname;

#[allow(unused_imports)]
use super::*;

/// The file, open and ready to read.
pub(crate) struct Opened {
    /// The names, after the short one was substituted for the long one.
    pub fname: *mut c_char,
    pub sfname: *mut c_char,
    pub fd: c_int,
    /// The file's mode bits, or a libuv error, or 0 when reading stdin.
    pub perm: c_int,
    /// Which line endings `'fileformats'` allows, read after the `*ReadPre`
    /// autocommands had their chance to change it.
    pub guess: FormatGuess,
}

/// Open what `readfile` was asked to read.
///
/// `Err` carries the value `readfile` should return: FAIL, or NOTDONE for a
/// directory, or OK when an autocommand did the reading itself.
#[allow(clippy::too_many_arguments)]
pub(crate) unsafe fn open_source(
    fname: *mut c_char,
    sfname: *mut c_char,
    from: linenr_T,
    eap: *mut exarg_T,
    how: How,
    silent: bool,
    msg_save: c_int,
) -> Result<Opened, c_int> {
    unsafe {
        let mut fname = fname;
        let mut sfname = sfname;
        let set_options = how.set_options;
        let mut retval = FAIL;
        let mut fd = if stdin_fd.get() >= 0 {
            stdin_fd.get()
        } else {
            0
        };
        let mut perm = 0;
        let mut swap_mode = -1; // protection bits for the swap file
        let mut file_info = FileInfo::default();
        let mut orig_start = pos_T {
            lnum: 0,
            col: 0,
            coladd: 0,
        };
        let msg_is_a_directory = c"is a directory";

        // Remember the initial values of curbuf, curbuf->b_ffname and
        // curbuf->b_fname, to detect nasty autocommands altering them.
        // Also check whether "fname" and "sfname" point at one of them.
        let old_curbuf = curbuf.get();
        let old_b_ffname = (*curbuf.get()).b_ffname;
        let old_b_fname = (*curbuf.get()).b_fname;
        let using_b_ffname = fname == old_b_ffname || sfname == old_b_ffname;
        let using_b_fname = fname == old_b_fname || sfname == old_b_fname;
        let buffer_changed = || {
            curbuf.get() != old_curbuf
                || (using_b_ffname && old_b_ffname != (*curbuf.get()).b_ffname)
                || (using_b_fname && old_b_fname != (*curbuf.get()).b_fname)
        };

        // After reading a file the cursor line changes, but we don't want
        // to display the line.
        ex_no_reprint.set(true);
        // Don't display the file info for another buffer now.
        need_fileinfo.set(false);

        // Use the short file name whenever possible: it avoids problems
        // with networks and when directory names change.
        if sfname.is_null() {
            sfname = fname;
        }
        fname = sfname;

        // The BufReadCmd and FileReadCmd events intercept the reading
        // process by running the associated commands instead.
        if !how.filtering && !how.stdin && !how.buffer {
            orig_start = (*curbuf.get()).b_op_start;

            // Set the '[ mark to the line above where the lines go, line
            // 1 if zero.
            (*curbuf.get()).b_op_start.lnum = if from == 0 { 1 } else { from };
            (*curbuf.get()).b_op_start.col = 0;

            if how.newfile {
                if apply_autocmds_exarg(
                    EVENT_BUFREADCMD,
                    core::ptr::null_mut(),
                    sfname,
                    false,
                    curbuf.get(),
                    eap,
                ) {
                    retval = if aborting() { FAIL } else { OK };
                    // The BufReadCmd code usually uses ":read" to get the
                    // text and perhaps ":file" to change the buffer name,
                    // but this should work like ":edit", so reset
                    // BF_NOTEDITED and let ":write" overwrite the file.
                    if retval == OK {
                        (*curbuf.get()).b_flags &= !BF_NOTEDITED;
                    }
                    return Err(retval);
                }
            } else if apply_autocmds_exarg(
                EVENT_FILEREADCMD,
                sfname,
                sfname,
                false,
                core::ptr::null_mut(),
                eap,
            ) {
                retval = if aborting() { FAIL } else { OK };
                return Err(retval);
            }

            (*curbuf.get()).b_op_start = orig_start;

            if how.nofile {
                // NOTDONE rather than FAIL, so that BufEnter can still be
                // triggered and other operations don't fail.
                retval = NOTDONE;
                return Err(retval);
            }
        }

        msg_scroll.set(
            (!((shortmess(SHM_OVER as c_int) && msg_listdo_overwrite.get() == 0)
                || (*curbuf.get()).b_help)
                || p_verbose.get() != 0) as c_int,
        );

        if !fname.is_null() && *fname != 0 {
            let fnamelen = strlen(fname);
            // If the name is too long we might crash further on, quit
            // here.
            if fnamelen >= MAXPATHL as size_t {
                filemess(
                    curbuf.get(),
                    fname,
                    translate(c"Illegal file name").as_ptr().cast_mut(),
                );
                msg_end();
                msg_scroll.set(msg_save);
                return Err(retval);
            }
            // A name ending in a path separator can't be opened. Check it
            // here, because reading may actually work but then creating
            // the swap file would destroy it.
            if after_pathsep(fname, fname.add(fnamelen)) != 0 {
                if !silent {
                    filemess(
                        curbuf.get(),
                        fname,
                        translate(msg_is_a_directory).as_ptr().cast_mut(),
                    );
                }
                msg_end();
                msg_scroll.set(msg_save);
                retval = NOTDONE;
                return Err(retval);
            }
        }

        if !how.stdin && !fname.is_null() {
            perm = os_getperm(fname);
        }

        if !how.stdin && !how.buffer && !how.fifo {
            let kind = perm & __S_IFMT;
            if perm >= 0 && kind != 0o100000 && kind != 0o10000 && kind != 0o140000 {
                // On Unix it is possible to read a directory, so check for
                // one before os_open().
                if kind == 0o40000 {
                    if !silent {
                        filemess(
                            curbuf.get(),
                            fname,
                            translate(msg_is_a_directory).as_ptr().cast_mut(),
                        );
                    }
                    retval = NOTDONE;
                } else {
                    filemess(
                        curbuf.get(),
                        fname,
                        translate(c"is not a file").as_ptr().cast_mut(),
                    );
                }
                msg_end();
                msg_scroll.set(msg_save);
                return Err(retval);
            }
        }

        // Set the default or forced 'fileformat' and 'binary'.
        set_file_options(set_options, eap);

        // When opening a new file take the readonly flag from the file.
        // The default is r/w and can be set to r/o below; don't reset it
        // in readonly mode, and only touch b_p_ro when BF_CHECK_RO is set.
        let check_readonly = how.newfile && (*curbuf.get()).b_flags & BF_CHECK_RO != 0;
        if check_readonly && !readonlymode.get() {
            (*curbuf.get()).b_p_ro = false as c_int;
        }

        if how.newfile && !how.stdin && !how.buffer && !how.fifo {
            // Remember the time of the file.
            if os_fileinfo(fname, &raw mut file_info) {
                buf_store_file_info(curbuf.get(), &raw mut file_info);
                (*curbuf.get()).b_mtime_read = (*curbuf.get()).b_mtime;
                (*curbuf.get()).b_mtime_read_ns = (*curbuf.get()).b_mtime_ns;
                // Use the protection bits of the original file for the
                // swap file, so that others can read the name of the
                // edited file from it, but only if they can read the file
                // itself. Remove the write and execute bits for group and
                // others (they must not write the swap file), and add read
                // and write for the user, or we might not be able to write
                // it ourselves. The bits are set below, after the swap
                // file is created.
                swap_mode = (file_info.stat.st_mode as c_int & 0o644) | 0o600;
            } else {
                (*curbuf.get()).b_mtime = 0;
                (*curbuf.get()).b_mtime_ns = 0;
                (*curbuf.get()).b_mtime_read = 0;
                (*curbuf.get()).b_mtime_read_ns = 0;
                (*curbuf.get()).b_orig_size = 0;
                (*curbuf.get()).b_orig_mode = 0;
            }
            // Reset the "new file" flag; it is set again below when the
            // file doesn't exist.
            (*curbuf.get()).b_flags &= !(BF_NEW | BF_NEW_W);
        }

        // Check readonly.
        let mut file_readonly = false;
        if !how.buffer && !how.stdin {
            if !how.newfile
                || readonlymode.get()
                || perm & 0o222 == 0
                || os_file_is_writable(fname) == 0
            {
                file_readonly = true;
            }
            fd = os_open(fname, O_RDONLY, 0);
        }

        if fd < 0 {
            // Cannot open at all.
            msg_scroll.set(msg_save);
            if !how.newfile {
                return Err(retval);
            }
            if perm == UV_ENOENT {
                // The file does not exist. Set the 'new-file' flag, so
                // that a ":w" complains if someone else created it since.
                (*curbuf.get()).b_flags |= BF_NEW;

                // Create a swap file now, so that other Nvims are warned
                // that we are editing this file. Not for a "nofile" or
                // "nowrite" buffer type.
                if !bt_dontwrite(curbuf.get()) {
                    check_need_swap(how.newfile);
                    // The SwapExists autocommand may mess things up.
                    if buffer_changed() {
                        emsg(gettext(e_auchangedbuf.get()));
                        return Err(retval);
                    }
                }
                if !silent {
                    let note = if dir_of_file_exists(fname) {
                        c"[New]"
                    } else {
                        c"[New DIRECTORY]"
                    };
                    filemess(curbuf.get(), sfname, translate(note).as_ptr().cast_mut());
                }
                // Even though this is a new file, it might have been
                // edited before and deleted. Get the old marks.
                check_marks_read();
                // Set the forced 'fileencoding'.
                if !eap.is_null() {
                    set_forced_fenc(eap);
                }
                apply_autocmds_exarg(EVENT_BUFNEWFILE, sfname, sfname, false, curbuf.get(), eap);
                // Remember the current fileformat.
                save_file_ff(curbuf.get());

                if !aborting() {
                    // Autocommands may abort script processing; a new file
                    // is not an error.
                    retval = OK;
                }
                return Err(retval);
            }

            // libuv only returns -errno, and on Windows open() does not
            // set EOVERFLOW.
            let note = if fd == UV_EFBIG || fd == -EOVERFLOW {
                c"[File too big]"
            } else {
                c"[Permission Denied]"
            };
            filemess(curbuf.get(), sfname, translate(note).as_ptr().cast_mut());
            (*curbuf.get()).b_p_ro = true as c_int; // must use "w!" now
            return Err(retval);
        }

        // Only set the 'ro' flag for readonly files the first time they
        // are loaded. Help files always get readonly mode.
        if (check_readonly && file_readonly) || (*curbuf.get()).b_help {
            (*curbuf.get()).b_p_ro = true as c_int;
        }

        if set_options {
            // Don't change 'eol' when reading from a buffer: it was
            // already set correctly when stdin was read.
            if !how.buffer {
                (*curbuf.get()).b_p_eof = false as c_int;
                (*curbuf.get()).b_start_eof = false as c_int;
                (*curbuf.get()).b_p_eol = true as c_int;
                (*curbuf.get()).b_start_eol = true as c_int;
            }
            (*curbuf.get()).b_p_bomb = false as c_int;
            (*curbuf.get()).b_start_bomb = false as c_int;
        }

        // Create a swap file now, so that other Nvims are warned that we
        // are editing this file. Not for a "nofile" or "nowrite" buffer.
        if !bt_dontwrite(curbuf.get()) {
            check_need_swap(how.newfile);
            if !how.stdin && buffer_changed() {
                emsg(gettext(e_auchangedbuf.get()));
                if !how.buffer {
                    close(fd);
                }
                return Err(retval);
            }
            // Set the swap file's protection bits now that it exists.
            let mfp = (*curbuf.get()).b_ml.ml_mfp;
            if swap_mode > 0 && !mfp.is_null() && !mf_fname(mfp).is_null() {
                let swap_fname = mf_fname(mfp);
                // If the group-read bit is set but not the world-read bit,
                // the group must equal the group of the original file. If
                // we can't make that happen, reset the group-read bit;
                // that avoids making the swap file readable to more users
                // than the file itself when the user's primary group is
                // too permissive.
                if swap_mode & 0o44 == 0o40 {
                    let mut swap_info = FileInfo::default();
                    if os_fileinfo(swap_fname, &raw mut swap_info)
                        && file_info.stat.st_gid != swap_info.stat.st_gid
                        && os_fchown(
                            (*mfp).mf_fd,
                            -1i32 as uv_uid_t,
                            file_info.stat.st_gid as uv_gid_t,
                        ) == -1
                    {
                        swap_mode &= 0o600;
                    }
                }
                os_setperm(swap_fname, swap_mode);
            }
        }

        // If "Quit" was selected at the ATTENTION dialog, don't load it.
        if swap_exists_action.get() == SEA_QUIT {
            if !how.buffer && !how.stdin {
                close(fd);
            }
            return Err(retval);
        }

        (*no_wait_return.ptr()) += 1; // don't wait for return yet

        // Set the '[ mark to the line above where the lines go, line 1 if
        // zero.
        orig_start = (*curbuf.get()).b_op_start;
        (*curbuf.get()).b_op_start.lnum = if from == 0 { 1 } else { from };
        (*curbuf.get()).b_op_start.col = 0;

        let mut guess = FormatGuess::from_ffs();

        if !how.buffer {
            let m = msg_scroll.get();
            let n = msg_scrolled.get();

            // The file must be closed again: the autocommands may want to
            // change it before it is read.
            if !how.stdin {
                close(fd); // ignore errors
            }

            // The output from the autocommands should neither overwrite
            // anything nor be overwritten: set msg_scroll, and restore it
            // if no output was done.
            msg_scroll.set(true as c_int);
            if how.filtering {
                apply_autocmds_exarg(
                    EVENT_FILTERREADPRE,
                    core::ptr::null_mut(),
                    sfname,
                    false,
                    curbuf.get(),
                    eap,
                );
            } else if how.stdin {
                apply_autocmds_exarg(
                    EVENT_STDINREADPRE,
                    core::ptr::null_mut(),
                    sfname,
                    false,
                    curbuf.get(),
                    eap,
                );
            } else if how.newfile {
                apply_autocmds_exarg(
                    EVENT_BUFREADPRE,
                    core::ptr::null_mut(),
                    sfname,
                    false,
                    curbuf.get(),
                    eap,
                );
            } else {
                apply_autocmds_exarg(
                    EVENT_FILEREADPRE,
                    sfname,
                    sfname,
                    false,
                    core::ptr::null_mut(),
                    eap,
                );
            }

            // The autocommands may have changed 'fileformats'.
            guess = FormatGuess::from_ffs();
            (*curbuf.get()).b_op_start = orig_start;

            if msg_scrolled.get() == n {
                msg_scroll.set(m);
            }

            if aborting() {
                // Autocommands may abort script processing.
                (*no_wait_return.ptr()) -= 1;
                msg_scroll.set(msg_save);
                (*curbuf.get()).b_p_ro = true as c_int; // must use "w!" now
                return Err(retval);
            }

            // Don't allow the autocommands to change the current buffer,
            // and don't allow them to change its name either (a `:cd`, for
            // instance) if that invalidates fname or sfname. Try to
            // re-open the file.
            if !how.stdin
                && (buffer_changed() || {
                    fd = os_open(fname, O_RDONLY, 0);
                    fd < 0
                })
            {
                (*no_wait_return.ptr()) -= 1;
                msg_scroll.set(msg_save);
                emsg(gettext(
                    if fd < 0 {
                        c"E200: *ReadPre autocommands made the file unreadable"
                    } else {
                        c"E201: *ReadPre autocommands must not change current buffer"
                    }
                    .as_ptr(),
                ));
                (*curbuf.get()).b_p_ro = true as c_int; // must use "w!" now
                return Err(retval);
            }
        }

        Ok(Opened {
            fname,
            sfname,
            fd,
            perm,
            guess,
        })
    }
}
