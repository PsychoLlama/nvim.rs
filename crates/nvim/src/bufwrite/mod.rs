#![deny(unsafe_op_in_unsafe_fn)]

use crate::cstr;
use crate::ex_docmd::cmdmod_has;
use crate::memline::MlFlags;
use crate::semsg;
use crate::tr_plural;
use core::ffi::CStr;
use std::borrow::Cow;
use std::ffi::CString;

use crate::autocmd::{
    EVENT_BUFWRITECMD, EVENT_BUFWRITEPOST, EVENT_BUFWRITEPRE, EVENT_FILEAPPENDCMD,
    EVENT_FILEAPPENDPOST, EVENT_FILEAPPENDPRE, EVENT_FILEWRITECMD, EVENT_FILEWRITEPOST,
    EVENT_FILEWRITEPRE, EVENT_FILTERWRITEPOST, EVENT_FILTERWRITEPRE, apply_autocmds_exarg,
    aucmd_prepbuf, aucmd_restbuf,
};
use crate::buffer::{BufFlags, buf_get_changedtick, buf_is_nofilename, buf_set_file_id};
use crate::change::unchanged;
use crate::drawscreen::status_redraw_all;
use crate::eval::vars::eval_charconvert;
use crate::event::libuv::uv_strerror;
use crate::ex_cmds::check_secure;
use crate::ex_eval::{aborting, should_abort_err};
use crate::fileio::{
    add_quoted_fname, buf_store_file_info, filemess, get_fio_flags, match_file_list, modname,
    msg_add_fileformat, msg_add_lines, need_conversion, set_rw_fname, time_differs, vim_rename,
    vim_tempname, write_eintr,
};
use crate::highlight_group::HLF_E;
use crate::input::ask_yesno;
use crate::main::{
    curbuf, e_empty_buffer, e_fsync, e_interr, e_longname, ex_no_reprint, exiting, got_int,
    msg_scroll, msg_silent, need_maketitle, no_wait_return, p_bdir, p_bex, p_bk, p_bsk, p_ccv,
    p_fs, p_pm, p_wb,
};
use crate::mbyte::{enc_canonize, my_iconv_open, utf_ptr2char, utf_ptr2len_len};
use crate::memline::{get_file_in_dir, make_percent_swname, ml_get_buf, ml_preserve, ml_timestamp};
use crate::memory::{verbose_try_malloc, xfree, xmemcpyz, xstrlcat};
use crate::message::{emsg, emsg_ptr, msg, msg_progress, msg_puts_hl, set_keep_msg};
use crate::message_fmt::{c_str, emsg_text};
use crate::option::{copy_option_part, cpo_has, get_bkc_flags, get_fileformat_force, shortmess};
use crate::options::{
    kOptBkcFlagAuto, kOptBkcFlagBreakhardlink, kOptBkcFlagBreaksymlink, kOptBkcFlagYes,
};
use crate::os::cshim::{gettext, gettext_ptr, snprintf};
use crate::os::fs::{
    os_chown, os_close, os_copy, os_copy_xattr, os_fchown, os_file_is_writable, os_file_settime,
    os_fileinfo, os_fileinfo_hardlinks, os_fileinfo_id_equal, os_fileinfo_link, os_free_acl,
    os_fsync, os_get_acl, os_getperm, os_isdir, os_mkdir_recurse, os_nodetype, os_open,
    os_path_exists, os_remove, os_set_acl, os_setperm,
};
use crate::os::input::os_breakcheck;
use crate::path::{after_pathsep, path_fnamecmp, path_tail};
use crate::sha256::Sha256;
use crate::strings::{vim_snprintf, vim_snprintf_add};
use crate::types::{
    CmdModFlags, CpoFlag, FAIL, Failed, FileInfo, IOSIZE, MAXPATHL, ShmFlag, aco_save_T, buf_T,
    exarg_T, iconv_t, int64_t, linenr_T, off_T, pos_T, size_t, uint64_t, uv_gid_t, uv_uid_t,
    vim_acl_T,
};
use crate::ui::ui_flush;
use crate::undo::{curbuf_is_changed, u_unchanged, u_update_save_nr, u_write_undo};
use crate::winlayer::Buf;
use ::libc::{__errno_location, close, getgid, getuid, iconv, iconv_close};

// The carve of the transpiled module; see each child's docs.
mod convert;
pub(crate) use self::convert::*;
mod backup;
pub(crate) use self::backup::*;
mod autocmds;
pub(crate) use self::autocmds::*;
mod lines;
pub(crate) use self::lines::*;
/// A write error, held until the cleanup path can report it.
///
/// `buf_write` has a single exit that emits the message, because the file
/// name has to be quoted first and because every failure shares the
/// "original file may be lost" handling that follows.
pub(crate) struct WriteError {
    /// The `E502`-style code, printed before the quoted file name.
    num: Option<&'static CStr>,
    /// The message, already translated. Owned only when it had to be
    /// formatted; otherwise borrowed from the translation catalog.
    msg: Cow<'static, CStr>,
    /// A libuv error code whose text is appended, or 0 for none. Without a
    /// `num` it is instead the argument of the message's own `%s`.
    arg: ::core::ffi::c_int,
}

/// The translated form of a literal message.
pub(crate) fn translate(msg: &'static CStr) -> &'static CStr {
    // SAFETY: gettext returns either its own argument or a pointer into the
    // loaded message catalog. Both outlive the process.
    unsafe { CStr::from_ptr(gettext(msg).as_ptr()) }
}

impl WriteError {
    /// A numbered error: `E502: "name" is a directory`.
    pub(crate) fn numbered(num: &'static CStr, msg: &'static CStr) -> Self {
        WriteError {
            num: Some(num),
            msg: translate(msg).into(),
            arg: 0,
        }
    }

    /// A message that carries its own error number, if any.
    pub(crate) fn plain(msg: &'static CStr) -> Self {
        WriteError {
            num: None,
            msg: translate(msg).into(),
            arg: 0,
        }
    }

    /// A message whose single `%s` takes the text of libuv error `arg`.
    pub(crate) fn errno(msg: &'static CStr, arg: ::core::ffi::c_int) -> Self {
        WriteError {
            num: None,
            msg: translate(msg).into(),
            arg,
        }
    }

    /// A message that had to be formatted at the point of failure.
    pub(crate) fn formatted(msg: CString) -> Self {
        WriteError {
            num: None,
            msg: msg.into(),
            arg: 0,
        }
    }

    /// A message from one of the shared `e_*` globals rather than from a
    /// literal here, with `arg` as for [`errno`](Self::errno).
    ///
    /// # Safety
    ///
    /// `msg` must point at a NUL-terminated string that outlives the error.
    pub(crate) unsafe fn shared(msg: *const ::core::ffi::c_char, arg: ::core::ffi::c_int) -> Self {
        WriteError {
            num: None,
            msg: unsafe { CStr::from_ptr(gettext_ptr(msg).as_ptr()) }.into(),
            arg,
        }
    }

    /// Report the error, against the already-quoted file name in `fname`.
    pub(crate) unsafe fn emit(&self, fname: &[::core::ffi::c_char; IOSIZE as usize]) {
        let msg = self.msg.as_ptr();
        let iobuff = fname.as_ptr().cast_mut();
        match (self.num, self.arg) {
            (Some(num), 0) => {
                // SAFETY: a message argument the caller holds as a NUL-terminated string, one apiece.
                let (num, iobuff, msg) =
                    unsafe { (c_str(num.as_ptr()), c_str(iobuff), c_str(msg)) };
                semsg!("{num}: {iobuff}{msg}");
            }
            (Some(num), arg) => {
                let why = unsafe { uv_strerror(arg) };
                // SAFETY: a message argument the caller holds as a NUL-terminated string, one apiece.
                let (num, iobuff, msg, why) =
                    unsafe { (c_str(num.as_ptr()), c_str(iobuff), c_str(msg), c_str(why)) };
                semsg!("{num}: {iobuff}{msg}: {why}");
            }
            // The message is deliberately its own format string here.
            (None, arg) if arg != 0 => {
                // SAFETY: `msg` is this error's own format, and `uv_strerror`
                // answers a NUL-terminated string for any code.
                let (template, why) = unsafe { (gettext_ptr(msg), c_str(uv_strerror(arg))) };
                emsg_text(tr_plural!(template, why));
            }
            (None, _) => {
                unsafe { emsg_ptr(msg) };
            }
        }
    }
}

/// "conversion failed", naming the line it failed on when one is known.
///
/// Upstream allocates 300 bytes for this; a stack buffer plus an owned copy
/// of what actually landed in it does the same job.
pub(crate) unsafe fn conversion_failed(lnum: linenr_T) -> WriteError {
    if lnum == 0 {
        return WriteError::plain(
            c"E513: Write error, conversion failed (make 'fenc' empty to override)",
        );
    }
    let mut msg = [0 as ::core::ffi::c_char; 300];
    let (into, size) = (msg.as_mut_ptr(), msg.len() as size_t);
    let fmt = translate(
        c"E513: Write error, conversion failed in line %d (make 'fenc' empty to override)",
    )
    .as_ptr();
    unsafe { vim_snprintf(into, size, fmt, lnum) };
    WriteError::formatted(unsafe { CStr::from_ptr(msg.as_ptr()) }.to_owned())
}

#[derive(Copy, Clone)]
pub struct bw_info {
    pub bw_fd: ::core::ffi::c_int,
    pub bw_buf: *mut ::core::ffi::c_char,
    pub bw_len: ::core::ffi::c_int,
    pub bw_flags: ::core::ffi::c_int,
    pub bw_first: ::core::ffi::c_int,
    pub bw_conv_buf: *mut ::core::ffi::c_char,
    pub bw_conv_buflen: size_t,
    pub bw_conv_error: ::core::ffi::c_int,
    pub bw_conv_error_lnum: linenr_T,
    pub bw_start_lnum: linenr_T,
    pub bw_iconv_fd: iconv_t,
}
pub const WRITEBUFSIZE: ::core::ffi::c_uint = 8192;
pub const FIO_LATIN1: ::core::ffi::c_int = 1;
pub const FIO_ENDIAN_L: ::core::ffi::c_int = 128;
pub const FIO_UTF16: ::core::ffi::c_int = 16;
pub const FIO_UCS2: ::core::ffi::c_int = 4;
pub const FIO_UCS4: ::core::ffi::c_int = 8;
pub const FIO_NOCONVERT: ::core::ffi::c_int = 8192;
pub const FIO_UTF8: ::core::ffi::c_int = 2;
pub const ICONV_MULT: ::core::ffi::c_uint = 8;
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const O_WRONLY: ::core::ffi::c_int = 0o1 as ::core::ffi::c_int;
pub const O_CREAT: ::core::ffi::c_int = 0o100 as ::core::ffi::c_int;
pub const O_EXCL: ::core::ffi::c_int = 0o200 as ::core::ffi::c_int;
pub const O_TRUNC: ::core::ffi::c_int = 0o1000 as ::core::ffi::c_int;
pub const O_APPEND: ::core::ffi::c_int = 0o2000 as ::core::ffi::c_int;
pub const __O_NOFOLLOW: ::core::ffi::c_int = 0o400000 as ::core::ffi::c_int;
pub const O_NOFOLLOW: ::core::ffi::c_int = __O_NOFOLLOW;
pub const UV_FS_COPYFILE_FICLONE: ::core::ffi::c_int = 0x2 as ::core::ffi::c_int;
pub const NL: ::core::ffi::c_int = '\n' as ::core::ffi::c_int;
pub const CAR: ::core::ffi::c_int = '\r' as ::core::ffi::c_int;
pub const NODE_WRITABLE: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
/// `'cpoptions'` "W": refuse to overwrite a read-only file even with `!`.
pub(crate) const E_READONLY_CPO: &CStr = c"is read-only (cannot override: \"W\" in 'cpoptions')";
pub const SMALLBUFSIZE: usize = 256;

/// What the caller is asking a write to be.
#[derive(Copy, Clone)]
pub struct WriteRequest {
    /// Append to the file instead of replacing it.
    pub append: bool,
    /// `!` was given: a failure to make a backup is not worth stopping for,
    /// and a read-only file may be written over.
    pub forceit: bool,
    /// Reset 'modified' when the write succeeds.
    pub reset_changed: bool,
    /// The write is the input side of a filter command (`:%!cmd`).
    pub filtering: bool,
}

impl WriteRequest {
    /// Writing a filter command's input, or a temporary file `:diffpatch`
    /// and friends hand to an external program: none of the buffer's own
    /// bookkeeping applies.
    pub const fn filter() -> Self {
        WriteRequest {
            append: false,
            forceit: false,
            reset_changed: false,
            filtering: true,
        }
    }
}

/// Write lines `start` through `end` of `buf` to `fname`.
///
/// Nvim does its own buffering because `fwrite()` is so slow. In case of an
/// error everything possible is done to restore the original file — but with
/// `req.forceit` we risk losing it, because that is what `!` asks for.
///
/// When `req.reset_changed` is set and the whole buffer is being written,
/// `b_changed` is reset.
///
/// This function must NOT use `NameBuff`: `autowrite()` calls it.
///
/// `eap` may be null; it carries a forced `'ff'`/`'fenc'`.
pub unsafe fn buf_write(
    buf: *mut buf_T,
    fname: *mut ::core::ffi::c_char,
    sfname: *mut ::core::ffi::c_char,
    start: linenr_T,
    end: linenr_T,
    eap: *mut exarg_T,
    req: WriteRequest,
) -> Result<(), Failed> {
    // SAFETY: the caller's promise, taken once for the whole body.
    let mut b = unsafe { Buf::new(buf) };
    // The quoted file name the failure path reports against.
    let mut quoted = [0 as ::core::ffi::c_char; IOSIZE as usize];
    let (mut buf, mut start, mut end) = (buf, start, end);
    let mut retval = Ok(());
    let msg_save = msg_scroll.get();
    let mut prev_got_int = got_int.get();
    let whole = start == 1 && end == b.b_ml.ml_line_count;
    let mut write_undo_file = false;
    let mut sha_ctx = Sha256::new();
    let bkc = get_bkc_flags(b);

    if fname.is_null() || unsafe { *fname } == 0 {
        return Err(Failed); // safety check
    }
    if b.b_ml.ml_mfp.is_null() {
        // Can happen during startup, from a stray "w" in the vimrc.
        emsg(gettext(e_empty_buffer));
        return Err(Failed);
    }
    if check_secure() {
        return Err(Failed); // writing is disallowed in secure mode
    }
    if unsafe { cstr::bytes_at(fname) }.len() >= MAXPATHL as size_t {
        emsg(gettext(e_longname)); // avoid a crash for a long name
        return Err(Failed);
    }

    // After writing, changedtick changes; don't display the line.
    ex_no_reprint.set(true);

    let (mut fname, mut sfname) = (fname, sfname);
    // With no file name yet, take the one being written to. BufFlags::NOTEDITED
    // records that, in case the write fails. Not for a filter command,
    // not when appending, and only when 'cpoptions' contains "F".
    if b.b_ffname.is_null()
        && req.reset_changed
        && whole
        && buf == curbuf.get()
        && !buf_is_nofilename(unsafe { Buf::from_raw(buf) })
        && !req.filtering
        && (!req.append || cpo_has(CpoFlag::FNAMEAPP))
        && cpo_has(CpoFlag::FNAMEW)
    {
        unsafe { set_rw_fname(fname, sfname) }?;
        buf = curbuf.get(); // just in case autocmds made "buf" invalid
        // SAFETY: `curbuf` is live; keep the handle in step with the pointer.
        b = unsafe { Buf::new(buf) };
    }
    if sfname.is_null() {
        sfname = fname;
    }
    // Unix: use the short file name whenever possible. It avoids
    // problems with networks and with directories that get renamed.
    let ffname = fname; // remember the full fname
    fname = sfname;

    // Writing over the file the buffer came from?
    let overwriting = !b.b_ffname.is_null() && unsafe { path_fnamecmp(ffname, b.b_ffname) } == 0;
    no_wait_return.set(no_wait_return.get() + 1); // don't wait for return yet

    let orig = OpMarks {
        start: b.b_op_start,
        end: b.b_op_end,
    };
    // Set '[ and '] to the lines to be written.
    b.b_op_start.lnum = start;
    b.b_op_start.col = 0;
    b.b_op_end.lnum = end;
    b.b_op_end.col = 0;

    let mode = WriteMode {
        req,
        whole,
        overwriting,
    };
    let mut names = WriteNames {
        fname,
        sfname,
        ffname,
    };
    let pre = unsafe { buf_write_do_autocmds(buf, &mut names, start, &mut end, eap, mode, orig) };
    // The autocommands may have renamed the buffer out from under them.
    let WriteNames {
        fname,
        sfname,
        ffname,
    } = names;
    if let PreWrite::Finished(res) = pre {
        return res;
    }

    if cmdmod_has(CmdModFlags::LOCKMARKS) {
        // Restore the original '[ and '] positions.
        b.b_op_start = orig.start;
        b.b_op_end = orig.end;
    }
    // Overwrite the previous file message, or don't.
    msg_scroll.set(if shortmess(ShmFlag::OVER) && !exiting.get() {
        0
    } else {
        1
    });
    if !req.filtering {
        unsafe { filemess(b, fname, c"".as_ptr().cast_mut()) }; // show that we are busy
    }
    msg_scroll.set(0); // always overwrite the file message now

    // The staging buffer. When the big one cannot be had, a small one on
    // the stack, so that writing still works when out of memory.
    let big = unsafe { verbose_try_malloc(WRITEBUFSIZE as usize) };
    let mut smallbuf = [0 as ::core::ffi::c_char; SMALLBUFSIZE];
    let staging: &mut [::core::ffi::c_char] = if big.is_null() {
        &mut smallbuf
    } else {
        unsafe { core::slice::from_raw_parts_mut(big.cast(), WRITEBUFSIZE as usize) }
    };
    let mut writer = ByteWriter::new(staging);

    let mut err: Option<WriteError> = None;
    let mut backup = Backup {
        copy: false,
        path: core::ptr::null_mut(),
    };
    let mut fenc_tofree: *mut ::core::ffi::c_char = core::ptr::null_mut();
    let mut file_info_old = FileInfo::default();
    // ACL copied from the original file to the backup or the new file.
    let mut acl: vim_acl_T = NULL;
    let mut target = TargetFile {
        perm: -1,
        device: false,
        newfile: false,
        readonly: false,
        made_writable: false,
    };
    let mut dobackup = false;
    let mut wfname: *mut ::core::ffi::c_char = core::ptr::null_mut();

    'cleanup: {
        'failed: {
            match unsafe {
                get_fileinfo(buf, fname, overwriting, req.forceit, &raw mut file_info_old)
            } {
                Ok(found) => target = found,
                // Err(None): the user declined; nothing to report.
                Err(reason) => {
                    err = reason;
                    break 'failed;
                }
            }
            // For systems that support ACL: take the original's.
            if !target.newfile {
                acl = os_get_acl(fname);
            }

            // 'backupskip' names files that get no backup.
            dobackup = p_wb.get() != 0 || p_bk.get() != 0 || unsafe { *p_pm.get() } != 0;
            if dobackup
                && unsafe { *p_bsk.get() } != 0
                && unsafe { match_file_list(p_bsk.get(), sfname, ffname) }
            {
                dobackup = false;
            }

            // Save got_int and reset it: an earlier interruption must not
            // cancel this write, only CTRL-C during it.
            prev_got_int = got_int.get();
            got_int.set(false);
            // Mark the buffer as being saved, to suppress changed-buffer
            // warnings.
            b.b_saving = true;

            // Back up when the file exists and 'writebackup', 'backup' or
            // 'patchmode' asks for it; appending only backs up for
            // 'patchmode'. With 'writebackup' and 'backup' both off there
            // is no backup at all, which helps on almost-full disks.
            if !(req.append && unsafe { *p_pm.get() } == 0)
                && !req.filtering
                && target.perm >= 0
                && dobackup
            {
                let old = &raw mut file_info_old;
                let (append, forceit) = (req.append, req.forceit);
                let made = unsafe {
                    buf_write_make_backup(fname, old, &target, acl, bkc, append, forceit)
                };
                match made {
                    Ok(made) => backup = made,
                    Err(e) => {
                        err = Some(e);
                        retval = Err(Failed);
                        break 'failed;
                    }
                }
            }

            // With ":w!" on a read-only file of our own, make it writable.
            if req.forceit
                && target.perm >= 0
                && target.perm & 0o200 == 0
                && file_info_old.stat.st_uid == unsafe { getuid() } as uint64_t
                && !cpo_has(CpoFlag::FWRITE)
            {
                target.perm |= 0o200;
                unsafe { os_setperm(fname, target.perm) };
                target.made_writable = true;
            }
            // With ":w!" over the current file, 'readonly' makes no
            // sense; reset it unless 'cpoptions' contains "Z".
            if req.forceit && overwriting && !cpo_has(CpoFlag::KEEPRO) {
                b.b_p_ro = 0;
                need_maketitle.set(true); // set the window title later
                unsafe { status_redraw_all() }; // redraw status lines later
            }

            end = end.min(b.b_ml.ml_line_count);
            if b.b_ml.ml_flags.has(MlFlags::EMPTY) {
                start = end + 1;
            }

            'restore_backup: {
                // Overwriting the original risks crashing in the middle
                // of the write, so preserve the buffer now: that makes
                // every block number positive, so recovery will not need
                // the original file. Not when there is a backup and we
                // are exiting anyway.
                if req.reset_changed
                    && !target.newfile
                    && overwriting
                    && !(exiting.get() && !backup.path.is_null())
                {
                    let fsync = if b.b_p_fs >= 0 { b.b_p_fs } else { p_fs.get() };
                    unsafe { ml_preserve(buf, false, fsync != 0) };
                    if got_int.get() {
                        err = Some(unsafe { WriteError::shared(e_interr.as_ptr(), 0) });
                        break 'restore_backup;
                    }
                }

                // Write the file directly, unless a conversion sends it
                // through a temp file first.
                wfname = fname;

                // A forced 'fileencoding' from a "++opt=val" argument.
                let fenc = if !eap.is_null() && unsafe { (*eap).force_enc } != 0 {
                    fenc_tofree =
                        unsafe { enc_canonize((*eap).cmd.offset((*eap).force_enc as isize)) };
                    fenc_tofree
                } else {
                    b.b_p_fenc
                };
                let converted = unsafe { need_conversion(fenc) };

                // UTF-8 to UCS-2/UCS-4/UTF-16/Latin1 (and back) is a
                // conversion the ByteWriter does itself, given the flags
                // and a buffer to translate into.
                let mut wb_flags = 0;
                if converted {
                    wb_flags = unsafe { get_fio_flags(fenc) };
                    if wb_flags & (FIO_UCS2 | FIO_UCS4 | FIO_UTF16 | FIO_UTF8) != 0 {
                        let mult = if wb_flags & (FIO_UCS2 | FIO_UTF16 | FIO_UTF8) != 0 {
                            2
                        } else {
                            4 // FIO_UCS4
                        };
                        if !unsafe { writer.reserve_conv_buf(mult) } {
                            end = 0;
                        }
                    }
                }
                if converted && wb_flags == 0 {
                    // Not one of ours: iconv, or failing that a
                    // 'charconvert' pass over a temp file afterwards.
                    if unsafe { writer.open_iconv(fenc) } {
                        if !unsafe { writer.reserve_conv_buf(ICONV_MULT as usize) } {
                            end = 0;
                        }
                    } else if unsafe { *p_ccv.get() } != 0 {
                        wfname = unsafe { vim_tempname() };
                        if wfname.is_null() {
                            // Can't write without a temp file!
                            err =
                                Some(WriteError::plain(c"E214: Can't find temp file for writing"));
                            break 'restore_backup;
                        }
                    }
                }
                let mut notconverted = false;
                if converted && wb_flags == 0 && !writer.has_iconv() && wfname == fname {
                    if !req.forceit {
                        err = Some(WriteError::plain(
                            c"E213: Cannot convert (add ! to write without conversion)",
                        ));
                        break 'restore_backup;
                    }
                    notconverted = true;
                }

                // When converting, first pretend to write and check for
                // conversion errors, then go round again and write for
                // real. With no conversion this writes for real straight
                // away.
                let mut checking_conversion = true;
                let mut fd = -1;
                let mut fileformat = 0;
                let mut written = Written::default();
                loop {
                    // No need to check when there is no conversion, or
                    // when a backup exists that a conversion failure can
                    // be restored from.
                    if !converted || dobackup {
                        checking_conversion = false;
                    }
                    if checking_conversion {
                        fd = -1; // make sure nothing is written
                    } else {
                        let old = &raw mut file_info_old;
                        let opened = unsafe {
                            open_write_file(wfname, fname, &mut target, old, req, &mut err)
                        };
                        match opened {
                            Some(opened) => fd = opened,
                            None => break 'restore_backup,
                        }
                    }
                    writer.fd = fd;
                    err = None;

                    // use "++bin", "++nobin" or 'binary'
                    let write_bin = if !eap.is_null() && unsafe { (*eap).force_bin } != 0 {
                        unsafe { (*eap).force_bin == FORCE_BIN }
                    } else {
                        b.b_p_bin != 0
                    };

                    let mut bom_chars = 0;
                    // Skip the BOM when appending to a file that already
                    // existed: it only means anything at the start.
                    if b.b_p_bomb != 0
                        && !write_bin
                        && (!req.append || target.perm < 0)
                        && unsafe { writer.stage_bom(fenc) } > 0
                    {
                        writer.flags = FIO_NOCONVERT | wb_flags; // don't convert
                        if !unsafe { writer.flush() } {
                            end = 0;
                        } else {
                            // Upstream reads the staged length back
                            // *after* the flush, where it is zero:
                            // the BOM does not count towards the
                            // character total.
                            bom_chars += writer.staged() as ::core::ffi::c_int;
                        }
                    }

                    write_undo_file = b.b_p_udf != 0
                        && overwriting
                        && !req.append
                        && !req.filtering
                        && req.reset_changed
                        && !checking_conversion;
                    if write_undo_file {
                        sha_ctx = Sha256::new(); // hash the text as it goes
                    }

                    writer.clear();
                    writer.flags = wb_flags;
                    fileformat = unsafe { get_fileformat_force(b, eap) };
                    let hash = write_undo_file.then_some(&mut sha_ctx);
                    let lines = (start, end);
                    written = unsafe {
                        write_lines(buf, lines, &mut writer, fileformat, write_bin, hash)
                    };
                    written.nchars += bom_chars;
                    if written.failed {
                        end = 0;
                    }

                    // Stop when the writing is done or an error happened.
                    if !checking_conversion || end == 0 {
                        break;
                    }
                    // Nothing went wrong, so writing should be fine: go
                    // round again and do it for real.
                    checking_conversion = false;
                }

                // If we started writing, finish writing — also when an
                // error was encountered.
                if !checking_conversion {
                    let old = &raw mut file_info_old;
                    let done = unsafe { finish_write(buf, fd, wfname, &target, &backup, acl, old) };
                    if let Some(e) = done {
                        err = Some(e);
                        end = 0;
                    }
                    if wfname != fname {
                        // The file went to a temp file; 'charconvert'
                        // turns that into the output file.
                        if end != 0
                            && unsafe { eval_charconvert(c"utf-8".as_ptr(), fenc, wfname, fname) }
                                == FAIL
                        {
                            writer.conv_error = true;
                            end = 0;
                        }
                        unsafe { os_remove(wfname) };
                        unsafe { xfree(wfname.cast()) };
                    }
                }

                if end == 0 {
                    if err.is_none() {
                        err = Some(if writer.conv_error {
                            unsafe { conversion_failed(writer.conv_error_lnum) }
                        } else if got_int.get() {
                            unsafe { WriteError::shared(e_interr.as_ptr(), 0) }
                        } else {
                            WriteError::plain(c"E514: Write error (file system full?)")
                        });
                    }
                    if unsafe { recover_from_backup(&backup, fname) } {
                        end = 1; // the original is back; no extra warning
                    }
                    break 'failed;
                }

                written.lnum -= start; // the number of lines written
                no_wait_return.set(no_wait_return.get() - 1); // may wait now

                if !req.filtering {
                    let notes = WriteNotes {
                        conv_error: writer.conv_error,
                        conv_error_lnum: writer.conv_error_lnum,
                        notconverted,
                        converted,
                        device: target.device,
                        newfile: target.newfile,
                        fileformat,
                    };
                    unsafe { report_written(buf, fname, &written, &notes, req.append) };
                }

                // Everything went out correctly: reset 'modified'. Unless
                // this was not the original file and 'cpoptions' has no
                // "+".
                if req.reset_changed
                    && whole
                    && !req.append
                    && !writer.conv_error
                    && (overwriting || cpo_has(CpoFlag::PLUS))
                {
                    unchanged(b, true, false);
                    let changedtick = buf_get_changedtick(b);
                    if b.b_last_changedtick + 1 == changedtick {
                        // b:changedtick may have been incremented in
                        // unchanged(), but that must not fire TextChanged.
                        b.b_last_changedtick = changedtick;
                    }
                    u_unchanged(b);
                    u_update_save_nr(b);
                }

                // Written to the current file: update the swap file's
                // timestamp (which also sets b_mtime) and reset the
                // BufFlags::WRITE_MASK flags.
                if overwriting {
                    unsafe { ml_timestamp(buf) };
                    if req.append {
                        b.b_flags.clear(BufFlags::NEW);
                    } else {
                        b.b_flags.clear(BufFlags::WRITE_MASK);
                    }
                }

                if unsafe { *p_pm.get() } != 0 && dobackup {
                    unsafe { apply_patchmode(fname, &mut backup, target.perm, &file_info_old) };
                }

                // Remove the backup unless 'backup' is set.
                if p_bk.get() == 0
                    && !backup.path.is_null()
                    && !writer.conv_error
                    && unsafe { os_remove(backup.path) } != 0
                {
                    emsg(translate(c"E207: Can't delete backup file"));
                }
                break 'cleanup;
            }

            // Reached by giving up on the write: throw the backup away,
            // and put the original back if it was moved out of the way.
            if !unsafe { restore_backup(&backup, fname, wfname, target.newfile) } {
                end = 0; // the original no longer exists either
            }
            if wfname != fname {
                unsafe { xfree(wfname.cast()) };
            }
        }
        no_wait_return.set(no_wait_return.get() - 1); // may wait for return now
    }

    // Done saving; accept changed-buffer warnings again.
    b.b_saving = false;

    unsafe { xfree(backup.path.cast()) };
    unsafe { xfree(fenc_tofree.cast()) };
    drop(writer); // frees the conversion buffer and closes iconv
    if !big.is_null() {
        unsafe { xfree(big) };
    }
    os_free_acl(acl);

    if let Some(err) = &err {
        // -100 to save some space for a further error message.
        unsafe { add_quoted_fname(quoted.as_mut_ptr(), (IOSIZE - 100) as size_t, b, fname) };
        unsafe { err.emit(&quoted) };
        retval = Err(Failed);
        if end == 0 {
            let hl_id = HLF_E;
            let warning = translate(c"\nWARNING: Original file may be lost or damaged\n").as_ptr();
            unsafe { msg_puts_hl(warning, hl_id, true) };
            let advice =
                translate(c"don't quit the editor until the file is successfully written!")
                    .as_ptr();
            unsafe { msg_puts_hl(advice, hl_id, true) };
            // Update the timestamp to avoid an "overwrite changed file"
            // prompt when writing again.
            if unsafe { os_fileinfo(fname, &raw mut file_info_old) } {
                unsafe { buf_store_file_info(b, &raw mut file_info_old) };
                b.b_mtime_read = b.b_mtime;
                b.b_mtime_read_ns = b.b_mtime_ns;
            }
        }
    }
    msg_scroll.set(msg_save);

    // Writing the whole file with 'undofile' set writes the undo file too.
    if retval.is_ok() && write_undo_file {
        let mut hash = sha_ctx.finish();
        unsafe { u_write_undo(core::ptr::null(), false, b, hash.as_mut_ptr()) };
    }

    if !should_abort_err(retval) {
        unsafe { buf_write_do_post_autocmds(buf, fname, eap, mode) };
        if aborting() {
            retval = Err(Failed); // autocmds may abort script processing
        }
    }

    got_int.set(got_int.get() | prev_got_int);
    retval
}

pub const FORCE_BIN: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const EOL_UNIX: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const EOL_DOS: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const EOL_MAC: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
pub const __S_IFMT: ::core::ffi::c_int = 0o170000 as ::core::ffi::c_int;
