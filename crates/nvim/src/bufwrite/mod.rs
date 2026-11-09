//! Writing a buffer's lines to a file: `:write`, `:update`, `:saveas`, the
//! input of a filter command and the temp files `:diffupdate` hands to an
//! external program.
//!
//! [`Buf::write_file`] is the one entry point. Its children carve it up: the
//! autocommands that bracket it ([`autocmds`]), the pre-flight, the backup
//! and the file it writes ([`backup`]), the conversion of the text
//! ([`convert`]) and the loop over the lines and the report it ends with
//! ([`lines`]).

#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use crate::ascii::NL;
use crate::ex_docmd::cmdmod_has;
use crate::mbyte::canonize;
use crate::memline::MlFlags;
use crate::narrow::len_as_int;
use crate::semsg;
use crate::tr_plural;
use crate::types::ColNr;
use core::ffi::{CStr, c_int};
use std::ffi::CString;

use crate::autocmd::defs::AcoSave;
use crate::autocmd::{apply_autocmds_exarg, aucmd_prepbuf, aucmd_restbuf};
use crate::buffer::{BufFlags, buf_get_changedtick, buf_is_nofilename, buf_set_file_id};
use crate::change::unchanged;
use crate::drawscreen::state::need_maketitle;
use crate::drawscreen::status_redraw_all;
use crate::eval::vars::eval_charconvert;
use crate::event::uv::{uv_gid_t, uv_uid_t};
use crate::ex_cmds::check_secure;
use crate::ex_docmd::state::ex_no_reprint;
use crate::ex_eval::{aborting, should_abort_err};
use crate::fileio::FileOffset;
use crate::fileio::{
    FioFlags, Report, buf_store_file_info, get_fio_flags, match_file_list, modname,
    msg_add_fileformat, msg_add_lines, need_conversion, rename_file, set_rw_fname, temp_name,
    time_differs,
};
use crate::getchar::state::got_int;
use crate::highlight_group::HLF_E;
use crate::input::ask_yesno;
use crate::memline::{ml_preserve, ml_timestamp};
use crate::memory::XString;
use crate::message::IOSIZE;
use crate::message::state::{msg_scroll, msg_silent, no_wait_return};
use crate::message::{e_empty_buffer, e_fsync, e_interr, e_longname};
use crate::message::{emsg, msg_progress, msg_str_hl, set_keep_msg};
use crate::message_fmt::{emsg_text, msg_cstr};
use crate::option::defs::{CpoFlag, ShmFlag};
use crate::option::vars::{P_BSK, P_CCV, P_PM, p_bk, p_bsk, p_fs, p_pm, p_wb};
use crate::option::{cpo_has, get_bkc_flags, shortmess};
use crate::options::{
    kOptBkcFlagAuto, kOptBkcFlagBreakhardlink, kOptBkcFlagBreaksymlink, kOptBkcFlagYes,
};
use crate::os::cshim::gettext;
use crate::os::fs::FileInfo;
use crate::os::fs::meta::VimAcl;
use crate::os::fs::os_strerror;
use crate::os::fs::{
    os_chown, os_copy, os_file_is_writable, os_file_settime, os_free_acl, os_get_acl, os_nodetype,
    os_remove, os_set_acl, os_setperm,
};
use crate::os::input::os_breakcheck;
use crate::os::libc::uint64_t;
use crate::path::MAXPATHL;
use crate::path::{after_pathsep, path_fnamecmp, tail_index};
use crate::sha256::Sha256;
use crate::startup::exiting;
use crate::strings::{FmtArg, format_into};
use crate::types::{CmdModFlags, ExArg, Failed, LineNr, Pos};
use crate::ui::ui_flush;
use crate::undo::{curbuf_is_changed, u_unchanged, u_update_save_nr, u_write_undo};
use crate::winlayer::Buf;

// The carve of the transpiled module; see each child's docs.
mod convert;
pub(crate) use self::convert::*;
mod backup;
pub(crate) use self::backup::*;
mod autocmds;
pub(crate) use self::autocmds::*;
mod lines;
pub(crate) use self::lines::*;
/// Why a write failed, held until the cleanup path can report it.
///
/// `buf_write` has a single exit that reports it, because the file name has
/// to be quoted first and because every failure shares the "original file
/// may be lost" handling that follows. The messages are upstream's; each is
/// translated through the same catalogue entry it always was.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum WriteError {
    /// `E502`: the name is a directory.
    IsDirectory,
    /// `E503`: neither a file nor a writable device.
    NotWritable,
    /// `E504`: read-only, and 'cpoptions' has "W".
    ReadOnlyCpo,
    /// `E505`: read-only.
    ReadOnly,
    /// `E509`: no backup could be made by copying.
    CannotCreateBackup,
    /// `E510`: no backup could be made by renaming.
    CannotMakeBackup,
    /// `E166`: the file is a link that was not followed.
    LinkedFile,
    /// `E212`: the file would not open, with libuv's code.
    CannotOpen(c_int),
    /// `E667`: the sync failed, with libuv's code.
    FsyncFailed(c_int),
    /// `E512`: the close failed, with libuv's code.
    CloseFailed(c_int),
    /// Interrupted while writing.
    Interrupted,
    /// `E214`: no temp file for a conversion.
    NoTempFile,
    /// `E213`: the conversion is not possible.
    CannotConvert,
    /// `E513`: the conversion failed, on this line when one is known.
    ConversionFailed(LineNr),
    /// `E514`: the write itself failed.
    WriteFailed,
}

const _: () = assert!(size_of::<WriteError>() <= 16);

/// The translated form of a literal message.
pub(crate) fn translate(msg: &'static CStr) -> &'static CStr {
    gettext(msg)
}

impl WriteError {
    /// Report the error, against the already-quoted file name in `fname`.
    pub(crate) fn report(&self, fname: &CStr) {
        let iobuff = msg_cstr(fname);
        // A numbered error: `E502: "name" is a directory`.
        let numbered = |num: &'static CStr, msg: &'static CStr| {
            let (num, msg) = (msg_cstr(num), msg_cstr(translate(msg)));
            semsg!("{num}: {iobuff}{msg}");
        };
        // A message whose single `%s` takes the text of libuv error `arg`;
        // the message is deliberately its own format string here.
        let with_errno = |msg: &'static CStr, arg: c_int| {
            let why = msg_cstr(os_strerror(arg));
            emsg_text(tr_plural!(translate(msg), why));
        };
        let plain = |msg: &'static CStr| {
            emsg(translate(msg));
        };
        match *self {
            Self::IsDirectory => numbered(c"E502", c"is a directory"),
            Self::NotWritable => numbered(c"E503", c"is not a file or writable device"),
            Self::ReadOnlyCpo => numbered(c"E504", E_READONLY_CPO),
            Self::ReadOnly => numbered(c"E505", c"is read-only (add ! to override)"),
            Self::CannotCreateBackup => {
                plain(c"E509: Cannot create backup file (add ! to override)");
            }
            Self::CannotMakeBackup => plain(c"E510: Can't make backup file (add ! to override)"),
            Self::LinkedFile => plain(c"E166: Can't open linked file for writing"),
            Self::CannotOpen(code) => with_errno(c"E212: Can't open file for writing: %s", code),
            Self::FsyncFailed(code) => with_errno(e_fsync, code),
            Self::CloseFailed(code) => with_errno(c"E512: Close failed: %s", code),
            Self::Interrupted => plain(e_interr),
            Self::NoTempFile => plain(c"E214: Can't find temp file for writing"),
            Self::CannotConvert => {
                plain(c"E213: Cannot convert (add ! to write without conversion)");
            }
            Self::ConversionFailed(lnum) if lnum == LineNr::ZERO => {
                plain(c"E513: Write error, conversion failed (make 'fenc' empty to override)");
            }
            Self::ConversionFailed(lnum) => {
                // Upstream formats into 300 bytes.
                let fmt = translate(
                    c"E513: Write error, conversion failed in line %d (make 'fenc' empty to override)",
                );
                let mut msg = vec![0; 300];
                let len = format_into(&mut msg, fmt.to_bytes(), &[FmtArg::from(lnum)]).min(299);
                msg.truncate(len);
                emsg(&CString::new(msg).unwrap_or_default());
            }
            Self::WriteFailed => plain(c"E514: Write error (file system full?)"),
        }
    }
}

pub const O_NOFOLLOW: ::core::ffi::c_int = ::libc::O_NOFOLLOW;
pub const UV_FS_COPYFILE_FICLONE: ::core::ffi::c_int = 0x2 as ::core::ffi::c_int;
pub const NODE_WRITABLE: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
/// `'cpoptions'` "W": refuse to overwrite a read-only file even with `!`.
pub(crate) const E_READONLY_CPO: &CStr = c"is read-only (cannot override: \"W\" in 'cpoptions')";

/// What the caller is asking a write to be.
#[derive(Copy, Clone, Debug)]
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

/// A name [`Buf::write_file`] writes under.
///
/// The buffer's own names are *named*, not borrowed: the write's
/// autocommands may rename the buffer, and the write follows such a rename
/// for a name that was the buffer's own (see [`WriteNameCopy`]).
#[derive(Clone, Copy, Debug)]
pub(crate) enum WriteName<'a> {
    /// The buffer's full name, `b_ffname`.
    Full,
    /// The buffer's short name, `b_sfname`.
    Short,
    /// The name the buffer shows, `b_fname`: its short name or its full one.
    Shown,
    /// A name the caller holds for the call, which nothing the write runs
    /// can free.
    Other(&'a CStr),
}

impl WriteName<'_> {
    /// A copy of the name, and the address that tells the buffer's own
    /// names from a caller's (zero for a caller's, which no name of the
    /// buffer's has). `None` for a buffer name that is not set.
    fn resolve(self, buffer: Buf) -> Option<WriteNameCopy> {
        let own =
            |name: Option<&CStr>| name.map(|name| WriteNameCopy::new(name, name.as_ptr().addr()));
        match self {
            WriteName::Full => own(buffer.name.full()),
            WriteName::Short => own(buffer.name.short()),
            WriteName::Shown => own(buffer.name.shown()),
            WriteName::Other(name) => Some(WriteNameCopy::new(name, 0)),
        }
    }
}

/// Where the text goes: straight into the file, or into a temp file that
/// `'charconvert'` turns into the file afterwards.
#[derive(Debug)]
enum WriteTarget {
    Direct,
    Temp(XString),
}

impl Buf {
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
    /// `sfname` is the short name to report, `None` for the same as `fname`;
    /// `excmd` carries a forced `'ff'`/`'fenc'`.
    pub(crate) fn write_file(
        self,
        fname: WriteName<'_>,
        sfname: Option<WriteName<'_>>,
        start: LineNr,
        end: LineNr,
        mut excmd: Option<&mut ExArg>,
        req: WriteRequest,
    ) -> Result<(), Failed> {
        let fname = fname.resolve(self);
        let sfname = sfname.and_then(|name| name.resolve(self));
        let mut b = self;
        let (mut buf, mut start, mut end) = (self, start, end);
        let mut retval = Ok(());
        let msg_save = msg_scroll.get();
        let mut prev_got_int = got_int.get();
        let whole = start == LineNr::ONE && end == b.b_ml.ml_line_count;
        let mut write_undo_file = false;
        let mut sha_ctx = Sha256::new();
        let bkc = get_bkc_flags(b);

        let Some(fname) = fname.filter(|name| !name.as_cstr().is_empty()) else {
            return Err(Failed); // safety check
        };
        if b.b_ml.ml_mfp.is_none() {
            // Can happen during startup, from a stray "w" in the vimrc.
            emsg(gettext(e_empty_buffer));
            return Err(Failed);
        }
        if check_secure() {
            return Err(Failed); // writing is disallowed in secure mode
        }
        if fname.as_cstr().to_bytes().len() >= MAXPATHL as usize {
            emsg(gettext(e_longname)); // avoid a crash for a long name
            return Err(Failed);
        }

        // After writing, changedtick changes; don't display the line.
        ex_no_reprint.set(true);

        // With no file name yet, take the one being written to. BufFlags::NOTEDITED
        // records that, in case the write fails. Not for a filter command,
        // not when appending, and only when 'cpoptions' contains "F".
        if b.name.full().is_none()
            && req.reset_changed
            && whole
            && buf == Buf::current()
            && !buf_is_nofilename(Some(buf))
            && !req.filtering
            && (!req.append || cpo_has(CpoFlag::FNAMEAPP))
            && cpo_has(CpoFlag::FNAMEW)
        {
            set_rw_fname(
                Some(fname.as_cstr()),
                sfname.as_ref().map(WriteNameCopy::as_cstr),
            )?;
            buf = Buf::current(); // just in case autocmds made "buf" invalid
            b = buf;
        }
        // Unix: use the short file name whenever possible. It avoids
        // problems with networks and with directories that get renamed.
        let ffname = fname; // remember the full fname
        let sfname = sfname.unwrap_or_else(|| ffname.clone());
        let fname = sfname.clone();

        // Writing over the file the buffer came from?
        let overwriting = b
            .name
            .full()
            .is_some_and(|full| path_fnamecmp(ffname.as_cstr().to_bytes(), full.to_bytes()) == 0);
        no_wait_return.set(no_wait_return.get() + 1); // don't wait for return yet

        let orig = OpMarks {
            start: b.b_op_start,
            end: b.b_op_end,
        };
        // Set '[ and '] to the lines to be written.
        b.b_op_start.lnum = start;
        b.b_op_start.col = ColNr::ZERO;
        b.b_op_end.lnum = end;
        b.b_op_end.col = ColNr::ZERO;

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
        let pre = buf.write_autocmds(
            &mut names,
            start,
            &mut end,
            excmd.as_deref_mut(),
            mode,
            orig,
        );
        if let PreWrite::Finished(res) = pre {
            return res;
        }
        // The autocommands may have renamed the buffer out from under them.
        let (fname, sfname, ffname) = (
            names.fname.as_cstr(),
            names.sfname.as_cstr(),
            names.ffname.as_cstr(),
        );

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
            b.file_message(Some(fname), c""); // show that we are busy
        }
        msg_scroll.set(0); // always overwrite the file message now

        let mut writer = ByteWriter::new();
        let mut err: Option<WriteError> = None;
        let mut backup = Backup {
            copy: false,
            path: None,
        };
        let mut file_info_old = FileInfo::default();
        // ACL copied from the original file to the backup or the new file.
        let mut acl: VimAcl = core::ptr::null_mut();
        let mut target;
        let mut dobackup;
        let mut wfname = WriteTarget::Direct;

        'cleanup: {
            'failed: {
                match get_fileinfo(buf, fname, overwriting, req.forceit, &mut file_info_old) {
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
                dobackup = p_wb() || p_bk() || P_PM.first_byte() != 0;
                if dobackup
                    && P_BSK.first_byte() != 0
                    && p_bsk(|value| match_file_list(value, sfname, ffname))
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
                if !(req.append && p_pm(CStr::is_empty))
                    && !req.filtering
                    && target.perm >= 0
                    && dobackup
                {
                    let (append, forceit) = (req.append, req.forceit);
                    let made = buf_write_make_backup(
                        fname,
                        &file_info_old,
                        &target,
                        acl,
                        bkc,
                        append,
                        forceit,
                    );
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
                    && file_info_old.stat.st_uid == uint64_t::from(crate::os::users::os_getuid())
                    && !cpo_has(CpoFlag::FWRITE)
                {
                    target.perm |= 0o200;
                    let _ = os_setperm(fname, target.perm);
                    target.made_writable = true;
                }
                // With ":w!" over the current file, 'readonly' makes no
                // sense; reset it unless 'cpoptions' contains "Z".
                if req.forceit && overwriting && !cpo_has(CpoFlag::KEEPRO) {
                    b.b_p_ro = 0;
                    need_maketitle.set(true); // set the window title later
                    status_redraw_all(); // redraw status lines later
                }

                end = end.min(b.b_ml.ml_line_count);
                if b.b_ml.ml_flags.has(MlFlags::EMPTY) {
                    start = end + 1;
                }

                // A copy of the encoding to write in: a forced 'fileencoding'
                // from a "++opt=val" argument, or the buffer's own.
                let forced_enc = excmd
                    .as_deref()
                    .and_then(|command| Some(command.line.cmd + command.force_enc?));
                let fenc = if let Some(at) = forced_enc {
                    // `force_enc` is an offset into the command's line.
                    let line = &excmd.as_ref().expect("the command is there").line;
                    canonize(line.rest_of(at))
                } else {
                    XString::from_bytes(b.b_p_fenc.as_deref().unwrap_or_default())
                };
                let fenc = fenc.as_cstr();

                'restore_backup: {
                    // Overwriting the original risks crashing in the middle
                    // of the write, so preserve the buffer now: that makes
                    // every block number positive, so recovery will not need
                    // the original file. Not when there is a backup and we
                    // are exiting anyway.
                    if req.reset_changed
                        && !target.newfile
                        && overwriting
                        && !(exiting.get() && backup.path.is_some())
                    {
                        let fsync = if b.b_p_fs >= 0 { b.b_p_fs != 0 } else { p_fs() };
                        ml_preserve(buf, false, fsync);
                        if got_int.get() {
                            err = Some(WriteError::Interrupted);
                            break 'restore_backup;
                        }
                    }

                    let converted = need_conversion(fenc.to_bytes());

                    // UTF-8 to UCS-2/UCS-4/UTF-16/Latin1 (and back) is a
                    // conversion the ByteWriter does itself, given the flags
                    // and a buffer to translate into.
                    let mut wb_flags = FioFlags::NONE;
                    if converted {
                        wb_flags = get_fio_flags(fenc.to_bytes());
                        let wide =
                            FioFlags::UCS2 | FioFlags::UCS4 | FioFlags::UTF16 | FioFlags::UTF8;
                        if wb_flags.has(wide) {
                            let mult = if wb_flags.has(wide.without(FioFlags::UCS4)) {
                                2
                            } else {
                                4 // FioFlags::UCS4
                            };
                            if !writer.reserve_conv_buf(mult) {
                                end = LineNr::ZERO;
                            }
                        }
                    }
                    if converted && wb_flags.is_empty() {
                        // Not one of ours: iconv, or failing that a
                        // 'charconvert' pass over a temp file afterwards.
                        if writer.open_iconv(fenc) {
                            if !writer.reserve_iconv_buf() {
                                end = LineNr::ZERO;
                            }
                        } else if P_CCV.first_byte() != 0 {
                            let Some(temp) = temp_name() else {
                                // Can't write without a temp file!
                                err = Some(WriteError::NoTempFile);
                                break 'restore_backup;
                            };
                            wfname = WriteTarget::Temp(temp);
                        }
                    }
                    let mut notconverted = false;
                    if converted
                        && wb_flags.is_empty()
                        && !writer.has_iconv()
                        && matches!(wfname, WriteTarget::Direct)
                    {
                        if !req.forceit {
                            err = Some(WriteError::CannotConvert);
                            break 'restore_backup;
                        }
                        notconverted = true;
                    }
                    let wpath = match &wfname {
                        WriteTarget::Direct => fname,
                        WriteTarget::Temp(temp) => temp.as_cstr(),
                    };

                    // When converting, first pretend to write and check for
                    // conversion errors, then go round again and write for
                    // real. With no conversion this writes for real straight
                    // away.
                    let mut checking_conversion = true;
                    let mut fileformat;
                    let mut written;
                    loop {
                        // No need to check when there is no conversion, or
                        // when a backup exists that a conversion failure can
                        // be restored from.
                        if !converted || dobackup {
                            checking_conversion = false;
                        }
                        writer.file = if checking_conversion {
                            None // make sure nothing is written
                        } else {
                            match open_write_file(
                                wpath,
                                fname,
                                &mut target,
                                &file_info_old,
                                req,
                                &mut err,
                            ) {
                                Some(file) => Some(file),
                                None => break 'restore_backup,
                            }
                        };
                        err = None;

                        // use "++bin", "++nobin" or 'binary'
                        let write_bin = excmd
                            .as_deref()
                            .and_then(|command| command.force_bin)
                            .unwrap_or(b.b_p_bin != 0);

                        let mut bom_chars = 0;
                        // Skip the BOM when appending to a file that already
                        // existed: it only means anything at the start.
                        if b.b_p_bomb != 0
                            && !write_bin
                            && (!req.append || target.perm < 0)
                            && writer.stage_bom(fenc.to_bytes()) > 0
                        {
                            writer.flags = FioFlags::NOCONVERT | wb_flags; // don't convert
                            if !writer.flush() {
                                end = LineNr::ZERO;
                            } else {
                                // Upstream reads the staged length back
                                // *after* the flush, where it is zero:
                                // the BOM does not count towards the
                                // character total.
                                bom_chars += len_as_int(writer.staged());
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
                        fileformat = b.fileformat_forced(excmd.as_deref());
                        let hash = write_undo_file.then_some(&mut sha_ctx);
                        let lines = (start, end);
                        written = buf.write_lines(lines, &mut writer, fileformat, write_bin, hash);
                        written.nchars += bom_chars;
                        if written.failed {
                            end = LineNr::ZERO;
                        }

                        // Stop when the writing is done or an error happened.
                        if !checking_conversion || end == LineNr::ZERO {
                            break;
                        }
                        // Nothing went wrong, so writing should be fine: go
                        // round again and do it for real.
                        checking_conversion = false;
                    }

                    // If we started writing, finish writing — also when an
                    // error was encountered.
                    if let Some(file) = writer.file.take() {
                        let done =
                            finish_write(buf, file, wpath, &target, &backup, acl, &file_info_old);
                        if let Some(e) = done {
                            err = Some(e);
                            end = LineNr::ZERO;
                        }
                        if let WriteTarget::Temp(temp) = &wfname {
                            // The file went to a temp file; 'charconvert'
                            // turns that into the output file.
                            let from = temp.as_cstr();
                            if end != LineNr::ZERO
                                && eval_charconvert(c"utf-8", fenc, from, fname).is_err()
                            {
                                writer.conv_error = true;
                                end = LineNr::ZERO;
                            }
                            os_remove(from);
                        }
                    }

                    if end == LineNr::ZERO {
                        if err.is_none() {
                            err = Some(if writer.conv_error {
                                WriteError::ConversionFailed(writer.conv_error_lnum)
                            } else if got_int.get() {
                                WriteError::Interrupted
                            } else {
                                WriteError::WriteFailed
                            });
                        }
                        if recover_from_backup(&backup, fname).is_ok() {
                            end = LineNr::ONE; // the original is back; no extra warning
                        }
                        break 'failed;
                    }

                    written.lnum -= start.get(); // the number of lines written
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
                        report_written(buf, fname, &written, &notes, req.append);
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
                        ml_timestamp(buf);
                        if req.append {
                            b.b_flags.clear(BufFlags::NEW);
                        } else {
                            b.b_flags.clear(BufFlags::WRITE_MASK);
                        }
                    }

                    if P_PM.first_byte() != 0 && dobackup {
                        apply_patchmode(fname, &mut backup, target.perm, &file_info_old);
                    }

                    // Remove the backup unless 'backup' is set.
                    if !p_bk()
                        && let Some(path) = &backup.path
                        && !writer.conv_error
                        && os_remove(path.as_cstr()) != 0
                    {
                        emsg(translate(c"E207: Can't delete backup file"));
                    }
                    break 'cleanup;
                }

                // Reached by giving up on the write: throw the backup away,
                // and put the original back if it was moved out of the way.
                let direct = matches!(wfname, WriteTarget::Direct);
                if restore_backup(&backup, fname, direct, target.newfile).is_err() {
                    end = LineNr::ZERO; // the original no longer exists either
                }
            }
            no_wait_return.set(no_wait_return.get() - 1); // may wait for return now
        }

        // Done saving; accept changed-buffer warnings again.
        b.b_saving = false;

        drop(writer); // frees the conversion buffer and closes iconv
        os_free_acl(acl);

        if let Some(err) = &err {
            // -100 to save some space for a further error message.
            let quoted = Report::quoted_fname(b, Some(fname), IOSIZE as usize - 100);
            err.report(quoted.as_cstr());
            retval = Err(Failed);
            if end == LineNr::ZERO {
                let hl_id = HLF_E;
                let warning = translate(c"\nWARNING: Original file may be lost or damaged\n");
                msg_str_hl(warning, hl_id, true);
                let advice =
                    translate(c"don't quit the editor until the file is successfully written!");
                msg_str_hl(advice, hl_id, true);
                // Update the timestamp to avoid an "overwrite changed file"
                // prompt when writing again.
                if let Some(info) = crate::os::fs::file_info(fname) {
                    file_info_old = info;
                    buf_store_file_info(b, &file_info_old);
                    b.b_mtime_read = b.b_mtime;
                    b.b_mtime_read_ns = b.b_mtime_ns;
                }
            }
        }
        msg_scroll.set(msg_save);

        // Writing the whole file with 'undofile' set writes the undo file too.
        if retval.is_ok() && write_undo_file {
            let hash = sha_ctx.finish();
            u_write_undo(None, false, b, &hash);
        }

        if !should_abort_err(retval) {
            buf.write_post_autocmds(fname, excmd, mode);
            if aborting() {
                retval = Err(Failed); // autocmds may abort script processing
            }
        }

        got_int.set(got_int.get() | prev_got_int);
        retval
    }
}

pub const EOL_UNIX: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const EOL_DOS: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const EOL_MAC: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
pub const __S_IFMT: ::core::ffi::c_int = 0o170000 as ::core::ffi::c_int;
