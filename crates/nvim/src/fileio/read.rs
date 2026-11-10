//! Reading a file into a buffer.
//!
//! [`readfile`] allocates as big a block as it can get, fills it with one
//! `read()`, converts it if the file's encoding is not UTF-8, and hands the
//! lines to `ml_append`. Everything hard about it comes from three things:
//! the encoding is a guess, so the whole file may have to be read again with
//! the next guess from `'fileencodings'`; the line ending is a guess too, so
//! the same can happen for `'fileformat'`; and a character can be split across
//! two reads, so each conversion keeps the leftover bytes.

#![deny(unsafe_op_in_unsafe_fn)]
#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use crate::fileio::Loaded;
use crate::mbyte::canonize;
use crate::memline::MlFlags;
use crate::option::vars::{P_CCV, P_FENCS};
use crate::winlayer::{Buf, Win};
use core::ffi::CStr;
use core::ffi::c_int;

use super::*;
use crate::edit::BeginlineOpts;
use crate::ex_docmd::cmdmod_has;
use crate::memfile::MfDirty;
use crate::option::cpo_has;
use crate::option::defs::{CpoFlag, OptionSetFlags};
use crate::types::{CmdModFlags, Failed};
use std::io::Read;

mod pass;
mod tail;

use self::pass::Reader;
use self::tail::{report_and_place, run_read_autocmds};
use crate::optionstr::OptString;
/// What the read is being asked to do, decoded from `readfile`'s `flags`.
#[derive(Clone, Debug)]
pub(crate) struct How {
    /// Starting to edit a new buffer.
    pub newfile: bool,
    /// Reading filter output.
    pub filtering: bool,
    /// Read from stdin instead of a file.
    pub stdin: bool,
    /// Read from `CURBUF` instead of a file, to convert what stdin gave us.
    pub buffer: bool,
    /// Read from a fifo or socket instead of a file.
    pub fifo: bool,
    /// Read into a dummy buffer, to see whether the file changed.
    pub dummy: bool,
    /// Don't read a file at all, only trigger `BufReadCmd`.
    pub nofile: bool,
    /// Don't clear the undo info or read it from a file.
    pub keep_undo: bool,
    /// The read may set the buffer's options.
    pub set_options: bool,
}

/// Read all of `fname` into the current buffer as a new file, for a
/// buffer that exists only to be searched (`READ_NEW | READ_DUMMY`).
pub(crate) fn read_dummy_file(fname: &CStr) -> Result<Loaded, Failed> {
    let flags = ReadFlags::NEW | ReadFlags::DUMMY;
    readfile_named(
        Some(fname),
        None,
        LineNr::ZERO,
        0,
        MAXLNUM.get(),
        None,
        flags,
        false,
    )
}

/// Read the lines of `fname` into the current buffer, after line `from`.
///
/// `fname` may be `None` only with [`ReadFlags::STDIN`]; `sfname` is the short
/// name to use for it, when there is one. When not recovering,
/// `lines_to_skip` is 0 and `lines_to_read` is `MAXLNUM`.
///
/// The names are the caller's, and autocommands run while they are held:
/// a caller reading a buffer's own name hands in a copy of it.
///
/// Answers [`Loaded::Skipped`] for a directory or a `BufReadCmd` that did
/// the reading itself, which are not failures.
#[allow(clippy::too_many_arguments)]
pub(crate) fn readfile_named(
    fname: Option<&CStr>,
    sfname: Option<&CStr>,
    from: LineNr,
    lines_to_skip: i32,
    lines_to_read: i32,
    mut excmd: Option<&mut ExArg>,
    flags: ReadFlags,
    silent: bool,
) -> Result<Loaded, Failed> {
    let how = How {
        newfile: flags.has(ReadFlags::NEW),
        filtering: flags.has(ReadFlags::FILTER),
        stdin: flags.has(ReadFlags::STDIN),
        buffer: flags.has(ReadFlags::BUFFER),
        fifo: flags.has(ReadFlags::FIFO),
        dummy: flags.has(ReadFlags::DUMMY),
        nofile: flags.has(ReadFlags::NOFILE),
        keep_undo: flags.has(ReadFlags::KEEP_UNDO),
        set_options: flags.has(ReadFlags::NEW | ReadFlags::BUFFER)
            || excmd.as_deref().is_some_and(|command| command.read_edit),
    };
    let msg_save = MSG_SCROLL.get();

    // Reset before triggering any autocommands.
    Buf::current().b_au_did_filetype = false;
    // In case it was set by the previous read.
    Buf::current().b_no_eol_lnum = LineNr::ZERO;

    // If there is no file name yet, use the one for the read file, and set
    // BufFlags::NOTEDITED to reflect that. Not for a read from a filter,
    // and only when 'cpoptions' contains the 'f' flag.
    if Buf::current().name.full().is_none()
        && !how.filtering
        && fname.is_some()
        && cpo_has(CpoFlag::FNAMER)
        && !how.dummy
        && set_rw_fname(fname, sfname).is_err()
    {
        return allow_swap_sync(Err(Failed));
    }

    let opened = open_source(
        fname,
        sfname,
        from,
        excmd.as_deref_mut(),
        &how,
        silent,
        msg_save,
    );
    let Opened {
        fname,
        sfname,
        input,
        perm,
        guess,
    } = match opened {
        Ok(opened) => opened,
        // Said here, where upstream's `readfile` said it.
        Err(early) => {
            return allow_swap_sync(early.map_err(|err| {
                err.report();
                Failed
            }));
        }
    };

    // Autocommands may have added lines, so check whether the buffer is
    // empty now.
    let wasempty = Buf::current().b_ml.ml_flags.has(MlFlags::EMPTY);

    if !RECOVERYMODE.get() && !how.filtering && !how.dummy && !silent && !how.stdin && !how.buffer {
        Buf::current().file_message(sfname, c"");
    }
    MSG_SCROLL.set(c_int::from(false)); // overwrite the file message

    let counts = (lines_to_skip, lines_to_read);
    let mut reader = Reader::new(&how, from, counts, fname, input, guess, excmd.as_deref());
    reader.read_passes(excmd.as_deref());
    let read_no_eol_lnum = reader.finish_last_line();
    let read = close_read(reader);

    // In recovery mode everything but autocommands is skipped.
    if !RECOVERYMODE.get() {
        let linecnt = settle_lines(&how, from, wasempty, read.linecnt, read.filesize);
        if GOT_INT.get() {
            if !how.dummy {
                Buf::current().file_message(sfname, gettext(E_INTERR));
                if how.newfile {
                    Buf::current().b_p_ro = c_int::from(true); // must use "w!" now
                }
            }
            MSG_SCROLL.set(msg_save);
            check_marks_read();
            return allow_swap_sync(Ok(Loaded::Read)); // an interrupt isn't really an error
        }
        let out = Outcome {
            perm,
            read_no_eol_lnum,
            linecnt,
            ..read.outcome
        };
        report_and_place(sfname, &how, silent, &out, read.bad_char, from);
    }
    MSG_SCROLL.set(msg_save);

    // Get the marks before running autocommands, so they can use them.
    check_marks_read();

    // Remember whether the last line read had no end-of-line, even when
    // 'binary' is off, to support turning 'fixeol' off or writing the same
    // text again with 'binary' on. The latter is needed for
    // ":autocmd FileReadPost *.gz set bin|'[,']!gunzip" to work.
    Buf::current().b_no_eol_lnum = read_no_eol_lnum;

    // When reloading a buffer put the cursor on the first line that differs.
    if how.keep_undo {
        u_find_first_changed();
    }

    // When opening a new file, locate the undo info and read it.
    if let Some(sha_ctx) = read.undo_hash {
        u_read_undo(None, &sha_ctx.finish(), fname);
    }

    if !how.stdin
        && !how.fifo
        && (!how.buffer || sfname.is_some())
        && !run_read_autocmds(sfname, excmd, &how, how.set_options)
    {
        // Autocommands may abort script processing. Note that this skips
        // the swap-file sync below, as upstream does.
        return Err(Failed);
    }

    allow_swap_sync(if RECOVERYMODE.get() && read.outcome.error {
        Err(Failed)
    } else {
        Ok(Loaded::Read)
    })
}

/// What a finished read leaves for the report and the autocommands.
#[derive(Debug)]
struct ReadDone {
    /// The report's facts; `perm`, `read_no_eol_lnum` and `linecnt` are
    /// filled in later.
    outcome: Outcome,
    /// The line count before the read.
    linecnt: i32,
    filesize: FileOffset,
    bad_char: c_int,
    /// The hash of the text read, when the undo file is to be read.
    undo_hash: Option<Sha256>,
}

/// Settle what the read decided — the file format, and 'fileencoding' when
/// the read may set options — and close the input. Stdin is replaced by
/// stderr, which makes shell commands work.
fn close_read(reader: Reader<'_>) -> ReadDone {
    let how = reader.how;
    if how.set_options {
        // Remember the current file format.
        save_file_ff(Buf::current());
        // When editing a new file set 'fileencoding' for this buffer. Also
        // for ":read ++edit file".
        set_option_direct(
            OPT_FILEENCODING,
            OptVal::from_cstr(reader.fenc.as_cstr()),
            OptionSetFlags::LOCAL,
            0 as ScriptId,
        );
    }
    let Reader {
        input,
        w,
        mut conv,
        tmpname,
        ..
    } = reader;
    conv.close_iconv();

    if how.stdin {
        // Closed; stderr takes its place, which makes shell commands work.
        drop(input);
        if STDIN_FD.get() < 0 {
            VIM_IGNORED.set(reopen_stdin_on_stderr());
        }
    } else if how.buffer {
        mark_stdin_cloexec();
    } else {
        drop(input); // errors are ignored
    }
    drop(w);

    if let Some(name) = tmpname {
        os_remove(name.as_cstr()); // delete the converted file
    }
    NO_WAIT_RETURN.set(NO_WAIT_RETURN.get() - 1); // may wait for return now

    ReadDone {
        outcome: Outcome {
            perm: 0,
            read_no_eol_lnum: LineNr::ZERO,
            ff_error: reader.ff_error,
            split: reader.split,
            notconverted: reader.notconverted,
            converted: reader.converted,
            conv_error: conv.conv_error,
            illegal_byte: reader.illegal_byte,
            error: reader.error,
            fileformat: reader.fileformat,
            linecnt: 0,
            filesize: reader.filesize,
        },
        linecnt: reader.linecnt,
        filesize: reader.filesize,
        bad_char: conv.bad_char,
        undo_hash: reader.read_undo_file.then_some(reader.sha_ctx),
    }
}

/// The buffer after the lines are in: the empty buffer's last line gone,
/// the change counters reset, and the screen, diff and folds told. Answers
/// how many lines were added to the `linecnt` there were before.
fn settle_lines(
    how: &How,
    from: LineNr,
    wasempty: bool,
    mut linecnt: i32,
    filesize: FileOffset,
) -> i32 {
    // The last line, which came from the empty buffer, has to go.
    if how.newfile && wasempty && !Buf::current().b_ml.ml_flags.has(MlFlags::EMPTY) {
        let _ = ml_delete(Buf::current().b_ml.ml_line_count);
        linecnt -= 1;
    }
    Buf::current().deleted_bytes = 0;
    Buf::current().deleted_bytes2 = 0;
    Buf::current().deleted_codepoints = 0;
    Buf::current().deleted_codeunits = 0;
    linecnt = (Buf::current().b_ml.ml_line_count - linecnt).get();
    if filesize == 0 {
        linecnt = 0;
    }
    if how.newfile || how.buffer {
        redraw_curbuf_later(UPD_NOT_VALID as c_int);
        // The diff info needs updating now that the text is in.
        diff_invalidate(Buf::current());
        // All folds in the window are invalid now. Mark them for update
        // before triggering autocommands.
        fold_update_all(Win::current());
    } else if linecnt != 0 {
        // At least one line was appended.
        appended_lines_mark(from, linecnt);
    }
    linecnt
}

/// The read is over: it is OK to sync the swap file now. Passes `answer`
/// through.
fn allow_swap_sync(answer: Result<Loaded, Failed>) -> Result<Loaded, Failed> {
    if Buf::current().memfile_dirty() == Some(MfDirty::YesNoSync) {
        Buf::current().set_memfile_dirty(MfDirty::Yes);
    }
    answer
}
