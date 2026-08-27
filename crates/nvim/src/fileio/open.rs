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

use crate::buffer::BufFlags;
use crate::os::uv_error::{UV_EFBIG, UV_ENOENT};
use crate::winlayer::Buf;
use core::ffi::{c_char, c_int};

use crate::bufwrite::translate;
use crate::memfile::mf_fname;
use crate::types::event_T;
use core::ffi::CStr;

use super::*;
use crate::types::{FAIL, MAXPATHL, OK, ShmFlag};

/// `filemess` about the current buffer, with a translated note.
fn filemess_note(fname: *mut c_char, note: &'static CStr) {
    let text = translate(note).as_ptr().cast_mut();
    // SAFETY: the current buffer is live and the note outlives the process.
    unsafe { filemess(Buf::current(), fname, text) };
}

/// One of the `*ReadPre`/`*ReadCmd` autocommands, which all take the same
/// shape: the short name as the io name, and either the current buffer or the
/// name again as the subject.
///
/// `for_file` picks the `File*` form, which names a file rather than a buffer.
///
/// # Safety
/// `sfname` must be null or the name the read uses, and `eap` the caller's
/// command or null.
unsafe fn read_autocmd(
    event: event_T,
    sfname: *mut c_char,
    eap: *mut exarg_T,
    for_file: bool,
) -> bool {
    let (iofile, buf) = if for_file {
        (sfname, ptr::null_mut())
    } else {
        (ptr::null_mut(), curbuf.get())
    };
    // SAFETY: the current buffer is live and `eap` is the caller's command.
    unsafe { apply_autocmds_exarg(event, iofile, sfname, false, buf, eap) }
}

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
    let old_b_ffname = cur_buf().b_ffname;
    let old_b_fname = cur_buf().b_fname;
    let using_b_ffname = fname == old_b_ffname || sfname == old_b_ffname;
    let using_b_fname = fname == old_b_fname || sfname == old_b_fname;
    let buffer_changed = || {
        curbuf.get() != old_curbuf
            || (using_b_ffname && old_b_ffname != cur_buf().b_ffname)
            || (using_b_fname && old_b_fname != cur_buf().b_fname)
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
        orig_start = cur_buf().b_op_start;

        // Set the '[ mark to the line above where the lines go, line
        // 1 if zero.
        cur_buf().b_op_start.lnum = if from == 0 { 1 } else { from };
        cur_buf().b_op_start.col = 0;

        if how.newfile {
            if unsafe { read_autocmd(EVENT_BUFREADCMD, sfname, eap, false) } {
                retval = if aborting() { FAIL } else { OK };
                // The BufReadCmd code usually uses ":read" to get the
                // text and perhaps ":file" to change the buffer name,
                // but this should work like ":edit", so reset
                // BufFlags::NOTEDITED and let ":write" overwrite the file.
                if retval == OK {
                    cur_buf().b_flags.clear(BufFlags::NOTEDITED);
                }
                return Err(retval);
            }
        } else if unsafe { read_autocmd(EVENT_FILEREADCMD, sfname, eap, true) } {
            retval = if aborting() { FAIL } else { OK };
            return Err(retval);
        }

        cur_buf().b_op_start = orig_start;

        if how.nofile {
            // NOTDONE rather than FAIL, so that BufEnter can still be
            // triggered and other operations don't fail.
            retval = NOTDONE;
            return Err(retval);
        }
    }

    msg_scroll.set(
        (!((shortmess(ShmFlag::OVER) && msg_listdo_overwrite.get() == 0) || cur_buf().b_help)
            || p_verbose.get() != 0) as c_int,
    );

    if !fname.is_null() && unsafe { *fname } != 0 {
        let fnamelen = unsafe { strlen(fname) };
        // If the name is too long we might crash further on, quit
        // here.
        if fnamelen >= MAXPATHL as size_t {
            filemess_note(fname, c"Illegal file name");
            unsafe { msg_end() };
            msg_scroll.set(msg_save);
            return Err(retval);
        }
        // A name ending in a path separator can't be opened. Check it
        // here, because reading may actually work but then creating
        // the swap file would destroy it.
        if unsafe { after_pathsep(fname, fname.add(fnamelen)) } != 0 {
            if !silent {
                filemess_note(fname, msg_is_a_directory);
            }
            unsafe { msg_end() };
            msg_scroll.set(msg_save);
            retval = NOTDONE;
            return Err(retval);
        }
    }

    if !how.stdin && !fname.is_null() {
        perm = unsafe { os_getperm(fname) };
    }

    if !how.stdin && !how.buffer && !how.fifo {
        let kind = perm & __S_IFMT;
        if perm >= 0 && kind != 0o100000 && kind != 0o10000 && kind != 0o140000 {
            // On Unix it is possible to read a directory, so check for
            // one before os_open().
            if kind == 0o40000 {
                if !silent {
                    filemess_note(fname, msg_is_a_directory);
                }
                retval = NOTDONE;
            } else {
                filemess_note(fname, c"is not a file");
            }
            unsafe { msg_end() };
            msg_scroll.set(msg_save);
            return Err(retval);
        }
    }

    // Set the default or forced 'fileformat' and 'binary'.
    unsafe { set_file_options(set_options, eap) };

    // When opening a new file take the readonly flag from the file.
    // The default is r/w and can be set to r/o below; don't reset it
    // in readonly mode, and only touch b_p_ro when BufFlags::CHECK_RO is set.
    let check_readonly = how.newfile && cur_buf().b_flags.has(BufFlags::CHECK_RO);
    if check_readonly && !readonlymode.get() {
        cur_buf().b_p_ro = false as c_int;
    }

    if how.newfile && !how.stdin && !how.buffer && !how.fifo {
        // Remember the time of the file.
        if unsafe { os_fileinfo(fname, &raw mut file_info) } {
            unsafe { buf_store_file_info(Buf::current(), &raw mut file_info) };
            cur_buf().b_mtime_read = cur_buf().b_mtime;
            cur_buf().b_mtime_read_ns = cur_buf().b_mtime_ns;
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
            cur_buf().b_mtime = 0;
            cur_buf().b_mtime_ns = 0;
            cur_buf().b_mtime_read = 0;
            cur_buf().b_mtime_read_ns = 0;
            cur_buf().b_orig_size = 0;
            cur_buf().b_orig_mode = 0;
        }
        // Reset the "new file" flag; it is set again below when the
        // file doesn't exist.
        cur_buf().b_flags.clear(BufFlags::NEW | BufFlags::NEW_W);
    }

    // Check readonly.
    let mut file_readonly = false;
    if !how.buffer && !how.stdin {
        if !how.newfile
            || readonlymode.get()
            || perm & 0o222 == 0
            || unsafe { os_file_is_writable(fname) } == 0
        {
            file_readonly = true;
        }
        fd = unsafe { os_open(fname, O_RDONLY, 0) };
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
            cur_buf().b_flags |= BufFlags::NEW;

            // Create a swap file now, so that other Nvims are warned
            // that we are editing this file. Not for a "nofile" or
            // "nowrite" buffer type.
            if !unsafe { bt_dontwrite(curbuf.get()) } {
                unsafe { check_need_swap(how.newfile) };
                // The SwapExists autocommand may mess things up.
                if buffer_changed() {
                    unsafe { emsg(gettext(e_auchangedbuf.get())) };
                    return Err(retval);
                }
            }
            if !silent {
                let note = if unsafe { dir_of_file_exists(fname) } {
                    c"[New]"
                } else {
                    c"[New DIRECTORY]"
                };
                unsafe { filemess(Buf::current(), sfname, translate(note).as_ptr().cast_mut()) };
            }
            // Even though this is a new file, it might have been
            // edited before and deleted. Get the old marks.
            unsafe { check_marks_read() };
            // Set the forced 'fileencoding'.
            if !eap.is_null() {
                unsafe { set_forced_fenc(eap) };
            }
            unsafe {
                apply_autocmds_exarg(EVENT_BUFNEWFILE, sfname, sfname, false, curbuf.get(), eap)
            };
            // Remember the current fileformat.
            save_file_ff(unsafe { Buf::current() });

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
        unsafe { filemess(Buf::current(), sfname, translate(note).as_ptr().cast_mut()) };
        cur_buf().b_p_ro = true as c_int; // must use "w!" now
        return Err(retval);
    }

    // Only set the 'ro' flag for readonly files the first time they
    // are loaded. Help files always get readonly mode.
    if (check_readonly && file_readonly) || cur_buf().b_help {
        cur_buf().b_p_ro = true as c_int;
    }

    if set_options {
        // Don't change 'eol' when reading from a buffer: it was
        // already set correctly when stdin was read.
        if !how.buffer {
            cur_buf().b_p_eof = false as c_int;
            cur_buf().b_start_eof = false as c_int;
            cur_buf().b_p_eol = true as c_int;
            cur_buf().b_start_eol = true as c_int;
        }
        cur_buf().b_p_bomb = false as c_int;
        cur_buf().b_start_bomb = false as c_int;
    }

    // Create a swap file now, so that other Nvims are warned that we
    // are editing this file. Not for a "nofile" or "nowrite" buffer.
    if !unsafe { bt_dontwrite(curbuf.get()) } {
        unsafe { check_need_swap(how.newfile) };
        if !how.stdin && buffer_changed() {
            unsafe { emsg(gettext(e_auchangedbuf.get())) };
            if !how.buffer {
                unsafe { close(fd) };
            }
            return Err(retval);
        }
        // Set the swap file's protection bits now that it exists.
        let mfp = cur_buf().b_ml.ml_mfp;
        if swap_mode > 0 && !mfp.is_null() && !unsafe { mf_fname(mfp) }.is_null() {
            let swap_fname = unsafe { mf_fname(mfp) };
            // If the group-read bit is set but not the world-read bit,
            // the group must equal the group of the original file. If
            // we can't make that happen, reset the group-read bit;
            // that avoids making the swap file readable to more users
            // than the file itself when the user's primary group is
            // too permissive.
            if swap_mode & 0o44 == 0o40 {
                let mut swap_info = FileInfo::default();
                if unsafe { os_fileinfo(swap_fname, &raw mut swap_info) }
                    && file_info.stat.st_gid != swap_info.stat.st_gid
                    && {
                        let gid = file_info.stat.st_gid as uv_gid_t;
                        // SAFETY: the memfile's own descriptor.
                        unsafe { os_fchown((*mfp).mf_fd, -1i32 as uv_uid_t, gid) == -1 }
                    }
                {
                    swap_mode &= 0o600;
                }
            }
            unsafe { os_setperm(swap_fname, swap_mode) };
        }
    }

    // If "Quit" was selected at the ATTENTION dialog, don't load it.
    if swap_exists_action.get() == SEA_QUIT {
        if !how.buffer && !how.stdin {
            unsafe { close(fd) };
        }
        return Err(retval);
    }

    no_wait_return.set(no_wait_return.get() + 1); // don't wait for return yet

    // Set the '[ mark to the line above where the lines go, line 1 if
    // zero.
    orig_start = cur_buf().b_op_start;
    cur_buf().b_op_start.lnum = if from == 0 { 1 } else { from };
    cur_buf().b_op_start.col = 0;

    let mut guess = unsafe { FormatGuess::from_ffs() };

    if !how.buffer {
        let m = msg_scroll.get();
        let n = msg_scrolled.get();

        // The file must be closed again: the autocommands may want to
        // change it before it is read.
        if !how.stdin {
            unsafe { close(fd) }; // ignore errors
        }

        // The output from the autocommands should neither overwrite
        // anything nor be overwritten: set msg_scroll, and restore it
        // if no output was done.
        msg_scroll.set(true as c_int);
        if how.filtering {
            unsafe { read_autocmd(EVENT_FILTERREADPRE, sfname, eap, false) };
        } else if how.stdin {
            unsafe { read_autocmd(EVENT_STDINREADPRE, sfname, eap, false) };
        } else if how.newfile {
            unsafe { read_autocmd(EVENT_BUFREADPRE, sfname, eap, false) };
        } else {
            unsafe { read_autocmd(EVENT_FILEREADPRE, sfname, eap, true) };
        }

        // The autocommands may have changed 'fileformats'.
        guess = unsafe { FormatGuess::from_ffs() };
        cur_buf().b_op_start = orig_start;

        if msg_scrolled.get() == n {
            msg_scroll.set(m);
        }

        if aborting() {
            // Autocommands may abort script processing.
            no_wait_return.set(no_wait_return.get() - 1);
            msg_scroll.set(msg_save);
            cur_buf().b_p_ro = true as c_int; // must use "w!" now
            return Err(retval);
        }

        // Don't allow the autocommands to change the current buffer,
        // and don't allow them to change its name either (a `:cd`, for
        // instance) if that invalidates fname or sfname. Try to
        // re-open the file.
        if !how.stdin
            && (buffer_changed() || {
                fd = unsafe { os_open(fname, O_RDONLY, 0) };
                fd < 0
            })
        {
            no_wait_return.set(no_wait_return.get() - 1);
            msg_scroll.set(msg_save);
            let msg = if fd < 0 {
                c"E200: *ReadPre autocommands made the file unreadable"
            } else {
                c"E201: *ReadPre autocommands must not change current buffer"
            };
            // SAFETY: a static message string.
            unsafe { emsg(gettext(msg.as_ptr())) };
            cur_buf().b_p_ro = true as c_int; // must use "w!" now
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

/// The buffer the editor is working in.
fn cur_buf() -> Buf {
    // SAFETY: `curbuf` is set from startup to exit.
    unsafe { Buf::current() }
}
