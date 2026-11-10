//! The write itself, phase by phase: look at the file, back it up, write
//! the text (converted, through a temp file for `'charconvert'`), then the
//! bookkeeping a successful write does. [`Buf::write_file`] drives it.

#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use super::*;

/// How a write that got as far as the backup gave up.
#[derive(Debug)]
enum Abandon {
    /// Before the text went out: throw the backup away, putting the
    /// original back if it was moved out of the way.
    RestoreBackup(Option<WriteError>),
    /// Writing the text failed; the original has been recovered from the
    /// backup when that was possible.
    Failed(WriteError),
}

/// The conversion a write does, decided before the first byte goes out.
#[derive(Debug)]
struct WriteConversion {
    /// The file's encoding differs from the buffer's.
    converted: bool,
    /// The conversions [`ByteWriter`] does itself; empty for iconv or
    /// `'charconvert'`.
    flags: FioFlags,
    /// Written without conversion, because `!` said to go ahead anyway.
    notconverted: bool,
}

/// What a write in progress carries from one phase to the next.
#[derive(Debug)]
pub(super) struct WriteJob {
    buf: Buf,
    start: LineNr,
    /// The last line to write — zero once the write has failed in a way
    /// that may have cost the original file.
    pub(super) end: LineNr,
    mode: WriteMode,
    writer: ByteWriter,
    backup: Backup,
    /// The file being written over, as it was before the write.
    file_info_old: FileInfo,
    /// ACL copied from the original file to the backup or the new file.
    acl: VimAcl,
    /// 'writebackup', 'backup' or 'patchmode' asks for a backup.
    dobackup: bool,
    /// The undo file is written too, from a hash of the text written.
    write_undo_file: bool,
    sha_ctx: Sha256,
}

/// The encoding to write in: a forced 'fileencoding' from a "++opt=val"
/// argument, or the buffer's own.
fn write_encoding(buffer: Buf, excmd: Option<&ExArg>) -> XString {
    match excmd.and_then(|command| Some((command, command.line.cmd + command.force_enc?))) {
        // `force_enc` is an offset into the command's line.
        Some((command, at)) => canonize(command.line.rest_of(at)),
        None => XString::from_bytes(buffer.b_p_fenc.as_deref().unwrap_or_default()),
    }
}

impl WriteJob {
    pub(super) fn new(buffer: Buf, start: LineNr, end: LineNr, mode: WriteMode) -> Self {
        WriteJob {
            buf: buffer,
            start,
            end,
            mode,
            writer: ByteWriter::new(),
            backup: Backup {
                copy: false,
                path: None,
            },
            file_info_old: FileInfo::default(),
            acl: core::ptr::null_mut(),
            dobackup: false,
            write_undo_file: false,
            sha_ctx: Sha256::new(),
        }
    }

    /// Free the conversion state and the ACL; the hash for the undo file,
    /// when there is one to write.
    pub(super) fn release(self) -> Option<Sha256> {
        drop(self.writer); // frees the conversion buffer and closes iconv
        os_free_acl(self.acl);
        self.write_undo_file.then_some(self.sha_ctx)
    }

    /// Write the file, with its backup: why it failed, if it did. `None`
    /// also when the user declined to overwrite a changed file.
    pub(super) fn write(
        &mut self,
        fname: &CStr,
        sfname: &CStr,
        ffname: &CStr,
        excmd: Option<&ExArg>,
        bkc: c_uint,
        prev_got_int: &mut bool,
    ) -> Option<WriteError> {
        let failed = self.try_write(fname, sfname, ffname, excmd, bkc, prev_got_int);
        let err = failed.err()?;
        NO_WAIT_RETURN.set(NO_WAIT_RETURN.get() - 1); // may wait for return now
        err
    }

    /// [`Self::write`]'s phases: look at the file, back it up, write the
    /// text, then the bookkeeping a successful write does.
    fn try_write(
        &mut self,
        fname: &CStr,
        sfname: &CStr,
        ffname: &CStr,
        excmd: Option<&ExArg>,
        bkc: c_uint,
        prev_got_int: &mut bool,
    ) -> Result<(), Option<WriteError>> {
        let (overwriting, forceit) = (self.mode.overwriting, self.mode.req.forceit);
        let mut target = get_fileinfo(
            self.buf,
            fname,
            overwriting,
            forceit,
            &mut self.file_info_old,
        )?;
        self.prepare(&mut target, fname, sfname, ffname, bkc, prev_got_int)?;

        let fenc = write_encoding(self.buf, excmd);
        let fenc = fenc.as_cstr();
        let mut wfname = WriteTarget::Direct;
        match self.write_text(&mut target, fname, fenc, excmd, &mut wfname) {
            Ok(done) => {
                self.after_write(&target, fname, done);
                Ok(())
            }
            Err(Abandon::RestoreBackup(err)) => {
                let direct = matches!(wfname, WriteTarget::Direct);
                if restore_backup(&self.backup, fname, direct, target.newfile).is_err() {
                    self.end = LineNr::ZERO; // the original no longer exists either
                }
                Err(err)
            }
            Err(Abandon::Failed(err)) => Err(Some(err)),
        }
    }

    /// Everything between looking at the file and writing it: the ACL, the
    /// backup, making a read-only file of our own writable under `!`, and
    /// the range clamped to the buffer as it is now.
    fn prepare(
        &mut self,
        target: &mut TargetFile,
        fname: &CStr,
        sfname: &CStr,
        ffname: &CStr,
        bkc: c_uint,
        prev_got_int: &mut bool,
    ) -> Result<(), WriteError> {
        let req = self.mode.req;
        let mut buf = self.buf;
        // For systems that support ACL: take the original's.
        if !target.newfile {
            self.acl = os_get_acl(fname);
        }

        // 'backupskip' names files that get no backup.
        self.dobackup = p_wb() || p_bk() || P_PM.first_byte() != 0;
        if self.dobackup
            && P_BSK.first_byte() != 0
            && p_bsk(|value| match_file_list(value, sfname, ffname))
        {
            self.dobackup = false;
        }

        // Save got_int and reset it: an earlier interruption must not cancel
        // this write, only CTRL-C during it.
        *prev_got_int = GOT_INT.get();
        GOT_INT.set(false);
        // Mark the buffer as being saved, to suppress changed-buffer
        // warnings.
        buf.b_saving = true;

        // Back up when the file exists and 'writebackup', 'backup' or
        // 'patchmode' asks for it; appending only backs up for 'patchmode'.
        // With 'writebackup' and 'backup' both off there is no backup at
        // all, which helps on almost-full disks.
        if !(req.append && p_pm(CStr::is_empty))
            && !req.filtering
            && target.perm >= 0
            && self.dobackup
        {
            self.backup = buf_write_make_backup(
                fname,
                &self.file_info_old,
                target,
                self.acl,
                bkc,
                req.append,
                req.forceit,
            )?;
        }

        // With ":w!" on a read-only file of our own, make it writable.
        if req.forceit
            && target.perm >= 0
            && target.perm & 0o200 == 0
            && self.file_info_old.stat.st_uid == uint64_t::from(crate::os::users::os_getuid())
            && !cpo_has(CpoFlag::FWRITE)
        {
            target.perm |= 0o200;
            let _ = os_setperm(fname, target.perm);
            target.made_writable = true;
        }
        // With ":w!" over the current file, 'readonly' makes no sense; reset
        // it unless 'cpoptions' contains "Z".
        if req.forceit && self.mode.overwriting && !cpo_has(CpoFlag::KEEPRO) {
            buf.b_p_ro = 0;
            NEED_MAKETITLE.set(true); // set the window title later
            status_redraw_all(); // redraw status lines later
        }

        self.end = self.end.min(buf.b_ml.ml_line_count);
        if buf.b_ml.ml_flags.has(MlFlags::EMPTY) {
            self.start = self.end + 1;
        }
        Ok(())
    }

    /// Write the text to `fname`, converted to `fenc`, through a temp file
    /// in `wfname` when `'charconvert'` does the converting. Answers what the
    /// report needs to say.
    fn write_text(
        &mut self,
        target: &mut TargetFile,
        fname: &CStr,
        fenc: &CStr,
        excmd: Option<&ExArg>,
        wfname: &mut WriteTarget,
    ) -> Result<(Written, WriteNotes), Abandon> {
        let req = self.mode.req;
        // Overwriting the original risks crashing in the middle of the
        // write, so preserve the buffer now: that makes every block number
        // positive, so recovery will not need the original file. Not when
        // there is a backup and we are exiting anyway.
        if req.reset_changed
            && !target.newfile
            && self.mode.overwriting
            && !(EXITING.get() && self.backup.path.is_some())
        {
            let fsync = if self.buf.b_p_fs >= 0 {
                self.buf.b_p_fs != 0
            } else {
                p_fs()
            };
            ml_preserve(self.buf, false, fsync);
            if GOT_INT.get() {
                return Err(Abandon::RestoreBackup(Some(WriteError::Interrupted)));
            }
        }

        let conversion = self
            .set_up_conversion(fenc, wfname)
            .map_err(|err| Abandon::RestoreBackup(Some(err)))?;
        let wpath = match &*wfname {
            WriteTarget::Direct => fname,
            WriteTarget::Temp(temp) => temp.as_cstr(),
        };

        let (mut written, fileformat) = self
            .write_passes(target, wpath, fname, &conversion, fenc, excmd)
            .map_err(Abandon::RestoreBackup)?;

        // If we started writing, finish writing — also when an error was
        // encountered.
        let mut err = None;
        if let Some(file) = self.writer.file.take() {
            let done = finish_write(
                self.buf,
                file,
                wpath,
                target,
                &self.backup,
                self.acl,
                &self.file_info_old,
            );
            if let Some(e) = done {
                err = Some(e);
                self.end = LineNr::ZERO;
            }
            if let WriteTarget::Temp(temp) = &*wfname {
                // The file went to a temp file; 'charconvert' turns that into
                // the output file.
                let from = temp.as_cstr();
                if self.end != LineNr::ZERO
                    && eval_charconvert(c"utf-8", fenc, from, fname).is_err()
                {
                    self.writer.conv_error = true;
                    self.end = LineNr::ZERO;
                }
                os_remove(from);
            }
        }

        if self.end == LineNr::ZERO {
            let err = err.unwrap_or(if self.writer.conv_error {
                WriteError::ConversionFailed(self.writer.conv_error_lnum)
            } else if GOT_INT.get() {
                WriteError::Interrupted
            } else {
                WriteError::WriteFailed
            });
            if recover_from_backup(&self.backup, fname).is_ok() {
                self.end = LineNr::ONE; // the original is back; no extra warning
            }
            return Err(Abandon::Failed(err));
        }

        written.lnum -= self.start.get(); // the number of lines written
        let notes = WriteNotes {
            conv_error: self.writer.conv_error,
            conv_error_lnum: self.writer.conv_error_lnum,
            notconverted: conversion.notconverted,
            converted: conversion.converted,
            device: target.device,
            newfile: target.newfile,
            fileformat,
        };
        Ok((written, notes))
    }

    /// Decide how the text gets to `fenc`: the [`ByteWriter`]'s own
    /// conversions, iconv, or `'charconvert'` over a temp file it puts in
    /// `wfname`.
    fn set_up_conversion(
        &mut self,
        fenc: &CStr,
        wfname: &mut WriteTarget,
    ) -> Result<WriteConversion, WriteError> {
        let converted = need_conversion(fenc.to_bytes());
        let writer = &mut self.writer;

        // UTF-8 to UCS-2/UCS-4/UTF-16/Latin1 (and back) is a conversion the
        // ByteWriter does itself, given the flags and a buffer to translate
        // into.
        let mut flags = FioFlags::NONE;
        if converted {
            flags = get_fio_flags(fenc.to_bytes());
            let wide = FioFlags::UCS2 | FioFlags::UCS4 | FioFlags::UTF16 | FioFlags::UTF8;
            if flags.has(wide) {
                let mult = if flags.has(wide.without(FioFlags::UCS4)) {
                    2
                } else {
                    4 // FioFlags::UCS4
                };
                if !writer.reserve_conv_buf(mult) {
                    self.end = LineNr::ZERO;
                }
            }
        }
        if converted && flags.is_empty() {
            // Not one of ours: iconv, or failing that a 'charconvert' pass
            // over a temp file afterwards.
            if writer.open_iconv(fenc) {
                if !writer.reserve_iconv_buf() {
                    self.end = LineNr::ZERO;
                }
            } else if P_CCV.first_byte() != 0 {
                // Can't write without a temp file!
                *wfname = WriteTarget::Temp(temp_name().ok_or(WriteError::NoTempFile)?);
            }
        }
        let mut notconverted = false;
        if converted
            && flags.is_empty()
            && !self.writer.has_iconv()
            && matches!(wfname, WriteTarget::Direct)
        {
            if !self.mode.req.forceit {
                return Err(WriteError::CannotConvert);
            }
            notconverted = true;
        }
        Ok(WriteConversion {
            converted,
            flags,
            notconverted,
        })
    }

    /// The write itself. When converting, first pretend to write and check
    /// for conversion errors, then go round again and write for real; with
    /// no conversion this writes for real straight away. `Err` is the file
    /// refusing to open, with why when that is known.
    fn write_passes(
        &mut self,
        target: &mut TargetFile,
        wpath: &CStr,
        fname: &CStr,
        conversion: &WriteConversion,
        fenc: &CStr,
        excmd: Option<&ExArg>,
    ) -> Result<(Written, c_int), Option<WriteError>> {
        let req = self.mode.req;
        let buf = self.buf;
        let mut checking_conversion = true;
        loop {
            // No need to check when there is no conversion, or when a backup
            // exists that a conversion failure can be restored from.
            if !conversion.converted || self.dobackup {
                checking_conversion = false;
            }
            self.writer.file = if checking_conversion {
                None // make sure nothing is written
            } else {
                let mut err = None;
                let file =
                    open_write_file(wpath, fname, target, &self.file_info_old, req, &mut err);
                Some(file.ok_or(err)?)
            };

            // use "++bin", "++nobin" or 'binary'
            let write_bin = excmd
                .and_then(|command| command.force_bin)
                .unwrap_or(buf.b_p_bin != 0);

            let mut bom_chars = 0;
            // Skip the BOM when appending to a file that already existed: it
            // only means anything at the start.
            if buf.b_p_bomb != 0
                && !write_bin
                && (!req.append || target.perm < 0)
                && self.writer.stage_bom(fenc.to_bytes()) > 0
            {
                self.writer.flags = FioFlags::NOCONVERT | conversion.flags; // don't convert
                if !self.writer.flush() {
                    self.end = LineNr::ZERO;
                } else {
                    // Upstream reads the staged length back *after* the
                    // flush, where it is zero: the BOM does not count
                    // towards the character total.
                    bom_chars += len_as_int(self.writer.staged());
                }
            }

            self.write_undo_file = buf.b_p_udf != 0
                && self.mode.overwriting
                && !req.append
                && !req.filtering
                && req.reset_changed
                && !checking_conversion;
            if self.write_undo_file {
                self.sha_ctx = Sha256::new(); // hash the text as it goes
            }

            self.writer.clear();
            self.writer.flags = conversion.flags;
            let fileformat = buf.fileformat_forced(excmd);
            let hash = self.write_undo_file.then_some(&mut self.sha_ctx);
            let lines = (self.start, self.end);
            let mut written = buf.write_lines(lines, &mut self.writer, fileformat, write_bin, hash);
            written.nchars += bom_chars;
            if written.failed {
                self.end = LineNr::ZERO;
            }

            // Stop when the writing is done or an error happened.
            if !checking_conversion || self.end == LineNr::ZERO {
                return Ok((written, fileformat));
            }
            // Nothing went wrong, so writing should be fine: go round again
            // and do it for real.
            checking_conversion = false;
        }
    }

    /// The bookkeeping after the text went out correctly: the report,
    /// 'modified', the swap file's timestamp, 'patchmode' and the backup.
    fn after_write(
        &mut self,
        target: &TargetFile,
        fname: &CStr,
        (written, notes): (Written, WriteNotes),
    ) {
        let (mut buf, req) = (self.buf, self.mode.req);
        NO_WAIT_RETURN.set(NO_WAIT_RETURN.get() - 1); // may wait now

        if !req.filtering {
            report_written(buf, fname, &written, &notes, req.append);
        }

        // Everything went out correctly: reset 'modified'. Unless this was
        // not the original file and 'cpoptions' has no "+".
        if req.reset_changed
            && self.mode.whole
            && !req.append
            && !self.writer.conv_error
            && (self.mode.overwriting || cpo_has(CpoFlag::PLUS))
        {
            unchanged(buf, true, false);
            let changedtick = buf_get_changedtick(buf);
            if buf.b_last_changedtick + 1 == changedtick {
                // b:changedtick may have been incremented in unchanged(), but
                // that must not fire TextChanged.
                buf.b_last_changedtick = changedtick;
            }
            u_unchanged(buf);
            u_update_save_nr(buf);
        }

        // Written to the current file: update the swap file's timestamp
        // (which also sets b_mtime) and reset the BufFlags::WRITE_MASK flags.
        if self.mode.overwriting {
            ml_timestamp(buf);
            if req.append {
                buf.b_flags.clear(BufFlags::NEW);
            } else {
                buf.b_flags.clear(BufFlags::WRITE_MASK);
            }
        }

        if P_PM.first_byte() != 0 && self.dobackup {
            apply_patchmode(fname, &mut self.backup, target.perm, &self.file_info_old);
        }

        // Remove the backup unless 'backup' is set.
        if !p_bk()
            && let Some(path) = &self.backup.path
            && !self.writer.conv_error
            && os_remove(path.as_cstr()) != 0
        {
            emsg(translate(c"E207: Can't delete backup file"));
        }
    }
}
