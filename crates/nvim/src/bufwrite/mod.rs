//! Writing a buffer's lines to a file: `:write`, `:update`, `:saveas`, the
//! input of a filter command and the temp files `:diffupdate` hands to an
//! external program.
//!
//! [`Buf::write_file`] is the one entry point. Its children carve it up: the
//! autocommands that bracket it ([`autocmds`]), the write's phases
//! ([`job`]), the pre-flight, the backup and the file it writes
//! ([`backup`]), the conversion of the text ([`convert`]) and the loop over
//! the lines and the report it ends with ([`lines`]).

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
use crate::fileio::{EOL_DOS, EOL_MAC, EOL_UNIX, FileOffset};
use crate::mbyte::canonize;
use crate::memline::MlFlags;
use crate::narrow::len_as_int;
use crate::os::fs::NODE_WRITABLE;
use crate::semsg;
use crate::tr_plural;
use crate::types::ColNr;
use core::ffi::{CStr, c_int, c_uint};
use std::ffi::CString;

use crate::autocmd::defs::AcoSave;
use crate::autocmd::{apply_autocmds_exarg, aucmd_prepbuf, aucmd_restbuf};
use crate::buffer::{BufFlags, buf_get_changedtick, buf_is_nofilename, buf_set_file_id};
use crate::change::unchanged;
use crate::drawscreen::state::NEED_MAKETITLE;
use crate::drawscreen::status_redraw_all;
use crate::eval::vars::eval_charconvert;
use crate::event::uv::{uv_gid_t, uv_uid_t};
use crate::ex_cmds::check_secure;
use crate::ex_docmd::state::EX_NO_REPRINT;
use crate::ex_eval::{aborting, should_abort_err};
use crate::fileio::{
    FioFlags, Report, buf_store_file_info, get_fio_flags, match_file_list, modname,
    msg_add_fileformat, msg_add_lines, need_conversion, rename_file, set_rw_fname, temp_name,
    time_differs,
};
use crate::getchar::state::GOT_INT;
use crate::highlight_group::HLF_E;
use crate::input::ask_yesno;
use crate::memline::{ml_preserve, ml_timestamp};
use crate::memory::XString;
use crate::message::IOSIZE;
use crate::message::state::{MSG_SCROLL, MSG_SILENT, NO_WAIT_RETURN};
use crate::message::{E_EMPTY_BUFFER, E_FSYNC, E_INTERR, E_LONGNAME};
use crate::message::{emsg, msg_progress, msg_str_hl, set_keep_msg};
use crate::message_fmt::{emsg_text, msg_cstr};
use crate::option::defs::{CpoFlag, ShmFlag};
use crate::option::vars::{P_BSK, P_CCV, P_PM, p_bk, p_bsk, p_fs, p_pm, p_wb};
use crate::option::{cpo_has, get_bkc_flags, shortmess};
use crate::options::{
    OPT_BKC_FLAG_AUTO, OPT_BKC_FLAG_BREAKHARDLINK, OPT_BKC_FLAG_BREAKSYMLINK, OPT_BKC_FLAG_YES,
};
use crate::os::cshim::gettext;
use crate::os::fs::meta::VimAcl;
use crate::os::fs::{FileInfo, os_strerror};
use crate::os::fs::{
    os_chown, os_copy, os_file_is_writable, os_file_settime, os_free_acl, os_get_acl, os_nodetype,
    os_remove, os_set_acl, os_setperm,
};
use crate::os::input::os_breakcheck;
use crate::os::libc::uint64_t;
use crate::path::MAXPATHL;
use crate::path::{after_pathsep, path_fnamecmp, tail_index};
use crate::sha256::Sha256;
use crate::startup::EXITING;
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
mod job;
use self::job::WriteJob;
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
            Self::FsyncFailed(code) => with_errno(E_FSYNC, code),
            Self::CloseFailed(code) => with_errno(c"E512: Close failed: %s", code),
            Self::Interrupted => plain(E_INTERR),
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

pub const UV_FS_COPYFILE_FICLONE: ::core::ffi::c_int = 0x2 as ::core::ffi::c_int;
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
        let mut buf = self;
        let mut end = end;
        let msg_save = MSG_SCROLL.get();
        let mut prev_got_int = GOT_INT.get();
        let whole = start == LineNr::ONE && end == buf.b_ml.ml_line_count;
        let bkc = get_bkc_flags(buf);

        let Some(fname) = fname.filter(|name| !name.as_cstr().is_empty()) else {
            return Err(Failed); // safety check
        };
        buf.check_can_write(fname.as_cstr())?;

        // After writing, changedtick changes; don't display the line.
        EX_NO_REPRINT.set(true);

        // With no file name yet, take the one being written to. BufFlags::NOTEDITED
        // records that, in case the write fails. Not for a filter command,
        // not when appending, and only when 'cpoptions' contains "F".
        if buf.name.full().is_none()
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
        }
        // Unix: use the short file name whenever possible. It avoids
        // problems with networks and with directories that get renamed.
        let ffname = fname; // remember the full fname
        let sfname = sfname.unwrap_or_else(|| ffname.clone());
        let fname = sfname.clone();

        // Writing over the file the buffer came from?
        let overwriting = buf
            .name
            .full()
            .is_some_and(|full| path_fnamecmp(ffname.as_cstr().to_bytes(), full.to_bytes()) == 0);
        NO_WAIT_RETURN.set(NO_WAIT_RETURN.get() + 1); // don't wait for return yet

        let orig = OpMarks {
            start: buf.b_op_start,
            end: buf.b_op_end,
        };
        // Set '[ and '] to the lines to be written.
        buf.b_op_start.lnum = start;
        buf.b_op_start.col = ColNr::ZERO;
        buf.b_op_end.lnum = end;
        buf.b_op_end.col = ColNr::ZERO;

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
            buf.b_op_start = orig.start;
            buf.b_op_end = orig.end;
        }
        // Overwrite the previous file message, or don't.
        MSG_SCROLL.set(if shortmess(ShmFlag::OVER) && !EXITING.get() {
            0
        } else {
            1
        });
        if !req.filtering {
            buf.file_message(Some(fname), c""); // show that we are busy
        }
        MSG_SCROLL.set(0); // always overwrite the file message now

        let mut job = WriteJob::new(buf, start, end, mode);
        let err = job.write(
            fname,
            sfname,
            ffname,
            excmd.as_deref(),
            bkc,
            &mut prev_got_int,
        );

        // Done saving; accept changed-buffer warnings again.
        buf.b_saving = false;
        let original_lost = job.end == LineNr::ZERO;
        let undo_hash = job.release();

        let mut retval = Ok(());
        if let Some(err) = &err {
            buf.report_write_failure(fname, err, original_lost);
            retval = Err(Failed);
        }
        MSG_SCROLL.set(msg_save);

        // Writing the whole file with 'undofile' set writes the undo file too.
        if retval.is_ok()
            && let Some(hash) = undo_hash
        {
            u_write_undo(None, false, buf, &hash.finish());
        }

        if !should_abort_err(retval) {
            buf.write_post_autocmds(fname, excmd, mode);
            if aborting() {
                retval = Err(Failed); // autocmds may abort script processing
            }
        }

        GOT_INT.set(GOT_INT.get() | prev_got_int);
        retval
    }

    /// Refuse a write that cannot work at all: no memline yet, the sandbox,
    /// or a name too long to handle. Said here.
    fn check_can_write(self, fname: &CStr) -> Result<(), Failed> {
        if self.b_ml.ml_mfp.is_none() {
            // Can happen during startup, from a stray "w" in the vimrc.
            emsg(gettext(E_EMPTY_BUFFER));
            return Err(Failed);
        }
        if check_secure() {
            return Err(Failed); // writing is disallowed in secure mode
        }
        if fname.to_bytes().len() >= MAXPATHL as usize {
            emsg(gettext(E_LONGNAME)); // avoid a crash for a long name
            return Err(Failed);
        }
        Ok(())
    }

    /// Say why the write to `fname` failed, and warn when the original may
    /// be gone too.
    fn report_write_failure(mut self, fname: &CStr, err: &WriteError, original_lost: bool) {
        // -100 to save some space for a further error message.
        let quoted = Report::quoted_fname(self, Some(fname), IOSIZE as usize - 100);
        err.report(quoted.as_cstr());
        if !original_lost {
            return;
        }
        let hl_id = HLF_E;
        let warning = translate(c"\nWARNING: Original file may be lost or damaged\n");
        msg_str_hl(warning, hl_id, true);
        let advice = translate(c"don't quit the editor until the file is successfully written!");
        msg_str_hl(advice, hl_id, true);
        // Update the timestamp to avoid an "overwrite changed file" prompt
        // when writing again.
        if let Some(info) = crate::os::fs::file_info(fname) {
            buf_store_file_info(self, &info);
            self.b_mtime_read = self.b_mtime;
            self.b_mtime_read_ns = self.b_mtime_ns;
        }
    }
}
