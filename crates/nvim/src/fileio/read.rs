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
use crate::keycodes::Ctrl_Z;
use crate::memfile::MfDirty;
use crate::option::cpo_has;
use crate::pos::MAXCOL;
use crate::types::{CmdModFlags, CpoFlag, Failed, OptionSetFlags};
use std::io::Read;

mod tail;

use self::tail::{report_and_place, run_read_autocmds};
use crate::optionstr::OptString;
/// What the read is being asked to do, decoded from `readfile`'s `flags`.
#[derive(Clone, Copy)]
pub(crate) struct How {
    /// Starting to edit a new buffer.
    pub newfile: bool,
    /// Reading filter output.
    pub filtering: bool,
    /// Read from stdin instead of a file.
    pub stdin: bool,
    /// Read from `curbuf` instead of a file, to convert what stdin gave us.
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
    readfile_named(Some(fname), None, 0, 0, MAXLNUM, None, flags, false)
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
    lines_to_skip: LineNr,
    lines_to_read: LineNr,
    mut excmd: Option<&mut ExArg>,
    flags: ReadFlags,
    silent: bool,
) -> Result<Loaded, Failed> {
    let mut retval = Err(Failed);
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
    let set_options = how.set_options;

    // Where the next line and character to read from curbuf are, for
    // READ_BUFFER.
    let mut read_buf_lnum: LineNr = 1;
    let mut read_buf_col: usize = 0;

    let mut lnum = from;
    let mut w = ReadBuf::new();
    let mut filesize: FileOffset = 0;
    let mut skip_read = false;
    let mut sha_ctx = Sha256::new();
    let mut read_undo_file = false;
    let mut split = 0; // number of split lines
    let mut error = false;
    let mut ff_error = EOL_UNKNOWN; // file format with errors
    let mut fileformat = 0;
    let mut keep_fileformat = false;
    let mut skip_count: LineNr = 0;
    let mut read_count: LineNr = 0;
    let msg_save = msg_scroll.get();
    // Non-zero line number when the last line read had no end-of-line.
    let mut read_no_eol_lnum: LineNr = 0;
    let mut file_rewind = false;
    let mut illegal_byte: LineNr = 0; // line nr with an illegal byte
    // Don't retry when a character doesn't fit in the destination
    // encoding.
    let mut keep_dest_enc = false;
    // The file 'charconvert' wrote, to be deleted afterwards.
    let mut tmpname: Option<XString> = None;
    let mut fenc: XString;
    let mut fencs = FencList::none();
    let mut advance_fenc = false;
    let mut did_iconv = false; // iconv() failed, try 'charconvert' next
    let mut converted = false;
    let mut notconverted = false;
    let mut conv = Conv::new(BAD_REPLACE);
    let mut linecnt: LineNr;
    let wasempty;

    // Reset before triggering any autocommands.
    Buf::current().b_au_did_filetype = false;
    // In case it was set by the previous read.
    Buf::current().b_no_eol_lnum = 0;

    'theend: {
        // If there is no file name yet, use the one for the read file, and
        // set BufFlags::NOTEDITED to reflect that. Not for a read from a filter,
        // and only when 'cpoptions' contains the 'f' flag.
        if Buf::current().name.full().is_none()
            && !how.filtering
            && fname.is_some()
            && cpo_has(CpoFlag::FNAMER)
            && !how.dummy
            && set_rw_fname(fname, sfname).is_err()
        {
            break 'theend;
        }

        let Opened {
            fname,
            sfname,
            mut input,
            perm,
            mut guess,
        } = match open_source(
            fname,
            sfname,
            from,
            excmd.as_deref_mut(),
            how,
            silent,
            msg_save,
        ) {
            Ok(opened) => opened,
            Err(early) => {
                retval = early;
                break 'theend;
            }
        };

        // Autocommands may have added lines, so check whether the buffer
        // is empty now.
        wasempty = Buf::current().b_ml.ml_flags.has(MlFlags::EMPTY);

        if !recoverymode.get()
            && !how.filtering
            && !how.dummy
            && !silent
            && !how.stdin
            && !how.buffer
        {
            filemess(Buf::current(), sfname, c"");
        }

        msg_scroll.set(false as c_int); // overwrite the file message

        // Set linecnt now, before the "retry" caused by a wrong guess at
        // the fileformat, and after the autocommands, which may change it.
        linecnt = Buf::current().b_ml.ml_line_count;

        // The "++bad=" argument.
        if let Some(command) = excmd.as_deref()
            && command.bad_char != 0
        {
            conv.bad_char = command.bad_char;
            if set_options {
                Buf::current().b_bad_char = command.bad_char;
            }
        } else {
            Buf::current().b_bad_char = 0;
        }

        // Decide which 'fileencoding' to use, or to start with.
        if let Some(command) = excmd.as_deref()
            && command.force_enc != 0
        {
            // `force_enc` is an offset into the command's own line.
            let name = command
                .line
                .rest_of(command.line.cmd + command.force_enc as usize);
            fenc = canonize(name);
            keep_dest_enc = true;
        } else if Buf::current().b_p_bin != 0 {
            fenc = XString::new(); // binary: don't convert
        } else if Buf::current().b_help {
            // Help files are either utf-8 or latin1. Try utf-8 first; if
            // that fails it must be latin1. This is needed when the first
            // line has non-ASCII characters, which happens only in *.??x
            // files.
            fencs = FencList::new(b"latin1");
            fenc = XString::from_cstr(c"utf-8");
        } else if p_fencs(CStr::is_empty) {
            fenc = Buf::current().b_p_fenc.get(); // use the buffer's encoding
        } else {
            // Try the items in 'fileencodings'; a copy, because the walk
            // spans the whole retry loop below.
            fencs = FencList::new(&P_FENCS.get());
            fenc = fencs.next();
        }

        // The retry loop. Reasons to go round again:
        // - the encoding conversion failed: try the next "fenc";
        // - a BOM was detected and "fenc" has to be set up for it;
        // - the "fileformat" guess was wrong: try another.
        //
        // "file_rewind" rewinds the file and reads it again, "advance_fenc"
        // moves on to the next "fenc", "skip_read" re-uses the bytes
        // already read (after a BOM), "did_iconv" means iconv() failed and
        // 'charconvert' is next, and "keep_fileformat" keeps the format we
        // just settled on. A "tmpname" is a file 'charconvert' made, which
        // has to be deleted afterwards.
        'retry: loop {
            if file_rewind {
                if how.buffer {
                    read_buf_lnum = 1;
                    read_buf_col = 0;
                } else if how.stdin || !rewind(input.as_ref()) {
                    // Can't rewind the file, give up.
                    error = true;
                    break 'retry;
                }
                // Delete the lines read so far.
                while lnum > from {
                    let _ = ml_delete(lnum);
                    lnum -= 1;
                }
                file_rewind = false;
                if set_options {
                    Buf::current().b_p_bomb = false as c_int;
                    Buf::current().b_start_bomb = false as c_int;
                }
                conv.conv_error = 0;
            }

            // The "fileformat" is reset the first time round, and whenever
            // we are retrying with another "fenc".
            if keep_fileformat {
                keep_fileformat = false;
            } else {
                if let Some(command) = excmd.as_deref()
                    && command.force_ff != 0
                {
                    fileformat = get_fileformat_force(Buf::current(), Some(command));
                    guess.try_unix = 0;
                    guess.try_dos = false;
                    guess.try_mac = 0;
                } else if Buf::current().b_p_bin != 0 {
                    fileformat = EOL_UNIX; // binary: use Unix format
                } else if p_ffs(CStr::is_empty) {
                    fileformat = get_fileformat(Buf::current()); // from the buffer
                } else {
                    fileformat = EOL_UNKNOWN; // detect from the file
                }
            }

            // An aborted iconv() conversion: close the descriptor.
            conv.close_iconv();

            if advance_fenc {
                // Try the next entry in 'fileencodings'.
                advance_fenc = false;

                if excmd
                    .as_deref()
                    .is_some_and(|command| command.force_enc != 0)
                {
                    // The conversion given with "++enc=" wasn't possible;
                    // read without conversion.
                    notconverted = true;
                    conv.conv_error = 0;
                    fenc = XString::new();
                } else {
                    fenc = fencs.next();
                }
                if let Some(name) = tmpname.take() {
                    os_remove(name.as_cstr()); // delete the converted file
                }
            }

            // Conversion is needed when the file's encoding differs from
            // 'encoding', or 'encoding' is UTF-16, UCS-2 or UCS-4.
            conv.flags = FioFlags::NONE;
            converted = need_conversion(&fenc);
            if converted {
                conv.flags = if fenc.as_cstr() == ENC_UCSBOM {
                    // "ucs-bom" means the first bytes of the file decide.
                    FioFlags::UCSBOM
                } else {
                    // Work out whether UCS-2/4 or Latin1 to UTF-8 can be
                    // done here rather than by iconv, which appears not to
                    // handle Unicode to Latin1 correctly. Doing it now
                    // saves parsing the name on every read.
                    get_fio_flags(&fenc)
                };

                // Try iconv() if we can't convert it ourselves.
                if conv.flags.is_empty() && !did_iconv {
                    conv.open_iconv(fenc.as_cstr());
                }

                // Use the 'charconvert' expression when conversion is
                // needed and neither of the above can do it.
                if conv.flags.is_empty()
                    && !how.stdin
                    && !how.buffer
                    && P_CCV.first_byte() != 0
                    && !how.fifo
                    && !conv.has_iconv()
                {
                    did_iconv = false;
                    // Skip the conversion when it is already done (a retry
                    // for a wrong "fileformat").
                    if tmpname.is_none() {
                        let name = fname.unwrap_or_default();
                        tmpname = readfile_charconvert(name, fenc.as_cstr(), &mut input);
                        if tmpname.is_none() {
                            // Conversion failed. Try another one.
                            advance_fenc = true;
                            if input.is_none() {
                                // Re-opening the original file failed!
                                emsg(gettext(c"E202: Conversion made file unreadable!"));
                                error = true;
                                break 'retry;
                            }
                            continue 'retry;
                        }
                    }
                } else if conv.flags.is_empty() && !conv.has_iconv() {
                    // Conversion is wanted but we can't do it; try the next
                    // entry in 'fileencodings'.
                    advance_fenc = true;
                    continue 'retry;
                }
            }

            // Rewinding the file and trying another "fenc" is possible
            // unless there is no other one to try, or we are reading stdin
            // or a fifo, or the encoding is fixed.
            conv.can_retry = !fenc.is_empty() && !how.stdin && !keep_dest_enc && !how.fifo;
            conv.linecnt = linecnt;

            if !skip_read {
                w.linerest = 0;
                filesize = 0;
                skip_count = lines_to_skip;
                read_count = lines_to_read;
                conv.restlen = 0;
                read_undo_file = how.newfile
                    && !how.keep_undo
                    && !Buf::current().name.full().is_none()
                    && Buf::current().b_p_udf != 0
                    && !how.filtering
                    && !how.fifo
                    && !how.stdin
                    && !how.buffer;
                if read_undo_file {
                    sha_ctx = Sha256::new();
                }
            }

            while !error && !got_int.get() {
                if !skip_read {
                    // Use a buffer of at least 64K. Add linerest to double
                    // the size if the line gets very long, to avoid a lot
                    // of copying. But don't read more than 1 Mbyte at a
                    // time, so that we can be interrupted.
                    w.size = (0x10000 + w.linerest as isize).min(0x100000);
                }

                // Split lines that are too long for ColNr. After this check
                // we read up to "size" more bytes, and even then the line
                // length must stay below MAXCOL - 1 (we add 1 for the NUL
                // when casting to ColNr). If it fires we insert a newline
                // right away, so linerest does not grow.
                if w.size < 0 || w.linerest as isize >= MAXCOL as isize - w.size {
                    split += 1;
                    w.bytes[w.ptr] = NL as u8; // split the line by inserting a NL
                    w.size = 1;
                } else if !skip_read {
                    if !w.make_room() {
                        error = true;
                        break;
                    }

                    // We may need room to translate into. For iconv() we
                    // don't really know how much, so use a factor.
                    // latin1 to utf-8: 1 byte becomes up to 2 bytes.
                    // utf-16 to utf-8: 2 bytes become up to 3, 4 become up
                    // to 4, and the size must be a multiple of 2.
                    // ucs-2 to utf-8: 2 bytes become up to 3, size must be
                    // a multiple of 2.
                    // ucs-4 to utf-8: 4 bytes become up to 6, size must be
                    // a multiple of 4.
                    w.real_size = w.len();
                    if conv.has_iconv() {
                        w.size /= ICONV_MULT;
                    } else if conv.flags.has(FioFlags::LATIN1) {
                        w.size /= 2;
                    } else if conv.flags.has(FioFlags::UCS2 | FioFlags::UTF16) {
                        w.size = (w.size * 2 / 3) & !1;
                    } else if conv.flags.has(FioFlags::UCS4) {
                        w.size = (w.size * 2 / 3) & !3;
                    } else if conv.flags == FioFlags::UCSBOM {
                        w.size /= ICONV_MULT; // worst case
                    }

                    if conv.restlen > 0 {
                        // Put the unconverted bytes from last time first.
                        conv.restore(&mut w.bytes[w.ptr..]);
                        w.ptr += conv.restlen;
                        w.size -= conv.restlen as isize;
                    }

                    w.size = if how.buffer {
                        // Read bytes from curbuf. Used for converting text
                        // read from stdin.
                        read_from_buffer(&mut w, from, &mut read_buf_lnum, &mut read_buf_col)
                    } else {
                        // Read bytes from the file.
                        let window = w.ptr..w.ptr + w.len();
                        input
                            .as_mut()
                            .map_or(-1, |file| read_retrying(file, &mut w.bytes[window]))
                    };

                    if w.size <= 0 {
                        if w.size < 0 {
                            error = true; // read error
                        } else if conv.restlen > 0 {
                            // End of file, but some trailing bytes could
                            // not be converted. A truncated file?
                            if conv.active() {
                                // We did a conversion, so report an error.
                                if conv.can_retry {
                                    rewind_retry(
                                        &mut did_iconv,
                                        &mut advance_fenc,
                                        &mut file_rewind,
                                        conv.has_iconv(),
                                    );
                                    continue 'retry;
                                }
                                if conv.conv_error == 0 {
                                    conv.conv_error =
                                        Buf::current().b_ml.ml_line_count - linecnt + 1;
                                }
                            } else if illegal_byte == 0 {
                                // Remember the first line with an illegal
                                // byte.
                                illegal_byte = Buf::current().b_ml.ml_line_count - linecnt + 1;
                            }
                            if conv.bad_char == BAD_DROP {
                                w.bytes[w.ptr - conv.restlen] = 0;
                                conv.restlen = 0;
                            } else {
                                // Replace the trailing bytes with the
                                // replacement character if we were
                                // converting; if we weren't, leave it to
                                // the UTF-8 check, which works slightly
                                // differently.
                                if conv.bad_char != BAD_KEEP && conv.active() {
                                    while conv.restlen > 0 {
                                        w.ptr -= 1;
                                        w.bytes[w.ptr] = conv.bad_char as u8;
                                        conv.restlen -= 1;
                                    }
                                }
                                conv.flags = FioFlags::NONE; // don't convert this
                                conv.close_iconv();
                            }
                        }
                    }
                }

                skip_read = false;

                // At the start of the file: check for a BOM. Also for the
                // other Unicode encodings, but not after converting with
                // 'charconvert' and not when a BOM was already found.
                if filesize == 0
                    && (conv.flags == FioFlags::UCSBOM
                        || (Buf::current().b_p_bomb == 0
                            && tmpname.is_none()
                            && matches!(fenc.first(), Some(b'u') | None)))
                {
                    // No BOM detection in a short file or in binary mode.
                    let found = if w.size < 2 || Buf::current().b_p_bin != 0 {
                        None
                    } else {
                        let allowed = if conv.flags == FioFlags::UCSBOM {
                            FioFlags::ALL
                        } else {
                            get_fio_flags(&fenc)
                        };
                        check_for_bom(w.window(), allowed)
                    };
                    if let Some((_, blen)) = found {
                        // Remove the BOM from the text.
                        filesize += blen as FileOffset;
                        w.size -= blen as isize;
                        let rest = w.ptr + blen..w.ptr + blen + w.len();
                        w.bytes.copy_within(rest, w.ptr);
                        if set_options {
                            Buf::current().b_p_bomb = true as c_int;
                            Buf::current().b_start_bomb = true as c_int;
                        }
                    }

                    if conv.flags == FioFlags::UCSBOM {
                        match found {
                            None => advance_fenc = true, // retry with the next encoding
                            // A BOM was found: set "fenc" and start over.
                            Some((name, _)) => fenc = XString::from_cstr(name),
                        }
                        // Retry without reading new bytes or rewinding.
                        skip_read = true;
                        continue 'retry;
                    }
                }

                // Include the bytes that were not converted last time.
                w.ptr -= conv.restlen;
                w.size += conv.restlen as isize;
                conv.restlen = 0;
                // A read error or end of file.
                if w.size <= 0 {
                    break;
                }

                if conv.has_iconv() && !conv.with_iconv(&mut w) {
                    conv.close_iconv();
                    rewind_retry(
                        &mut did_iconv,
                        &mut advance_fenc,
                        &mut file_rewind,
                        conv.has_iconv(),
                    );
                    continue 'retry;
                }

                if !conv.flags.is_empty() {
                    if !conv.units_to_utf8(&mut w) {
                        let had_iconv = conv.has_iconv();
                        conv.close_iconv();
                        rewind_retry(
                            &mut did_iconv,
                            &mut advance_fenc,
                            &mut file_rewind,
                            had_iconv,
                        );
                        continue 'retry;
                    }
                } else if Buf::current().b_p_bin == 0
                    && !conv.check_utf8(&mut w, filesize, &mut illegal_byte)
                {
                    let had_iconv = conv.has_iconv();
                    conv.close_iconv();
                    rewind_retry(
                        &mut did_iconv,
                        &mut advance_fenc,
                        &mut file_rewind,
                        had_iconv,
                    );
                    continue 'retry;
                }

                // Count the characters, after conversion.
                filesize += w.size as FileOffset;

                // When reading the first part of a file, guess the EOL
                // type.
                if fileformat == EOL_UNKNOWN {
                    fileformat = guess.guess(w.window());
                    // May set 'fileformat' when editing a new file.
                    if set_options {
                        set_fileformat(fileformat, OptionSetFlags::LOCAL);
                    }
                }

                let mut sink = LineSplit {
                    lnum,
                    skip_count,
                    read_count,
                    sha: &mut sha_ctx,
                    read_undo_file,
                    newfile: how.newfile,
                    fileformat,
                    ff_error,
                    try_unix: guess.try_unix != 0,
                    stdin: how.stdin,
                    from_buffer: how.buffer,
                    input: input.as_ref(),
                    set_options,
                };
                let split_as = split_lines(&mut w, &mut sink);
                (lnum, skip_count, read_count) = (sink.lnum, sink.skip_count, sink.read_count);
                (fileformat, ff_error) = (sink.fileformat, sink.ff_error);
                match split_as {
                    Split::Done => {}
                    Split::Stop => error = true,
                    Split::RetryUnix => {
                        file_rewind = true;
                        keep_fileformat = true;
                        continue 'retry;
                    }
                }
                w.linerest = w.ptr - w.line_start;
                os_breakcheck();
            }
            break 'retry;
        }

        // Not an error: the maximum number of lines was reached.
        if error && read_count == 0 {
            error = false;
        }

        // In Dos format ignore a trailing CTRL-Z, unless 'binary' is set.
        // In the old days the file length was a sector count and the
        // CTRL-Z was the marker for where the file really ended. Assuming
        // we write to a file system that keeps the length properly, the
        // CTRL-Z should be dropped; 'endoffile' lets the user decide what
        // to write later. In Unix format the CTRL-Z is just a character.
        if w.linerest != 0
            && Buf::current().b_p_bin == 0
            && fileformat == EOL_DOS
            && w.bytes[w.ptr - 1] == Ctrl_Z as u8
        {
            w.ptr -= 1;
            w.linerest -= 1;
            if set_options {
                Buf::current().b_p_eof = true as c_int;
            }
        }

        // If we hit end of file in the middle of a line, note that and
        // complete the line ourselves.
        if !error && !got_int.get() && w.linerest != 0 {
            // Remember it for when writing.
            if set_options {
                Buf::current().b_p_eol = false as c_int;
            }
            w.bytes[w.ptr] = 0;
            let line = &w.bytes[w.line_start..=w.ptr];
            if ml_append_terminated(lnum, line, how.newfile).is_err() {
                error = true;
            } else {
                if read_undo_file {
                    sha_ctx.update(line);
                }
                lnum += 1;
                read_no_eol_lnum = lnum;
            }
        }

        if set_options {
            // Remember the current file format.
            save_file_ff(Buf::current());
            // When editing a new file set 'fileencoding' for this buffer.
            // Also for ":read ++edit file".
            set_option_direct(
                kOptFileencoding,
                OptVal::from_cstr(fenc.as_cstr()),
                OptionSetFlags::LOCAL,
                0 as ScriptId,
            );
        }
        conv.close_iconv();

        if how.stdin {
            // Closed; stderr takes its place, which makes shell commands
            // work.
            drop(input);
            if stdin_fd.get() < 0 {
                vim_ignored.set(reopen_stdin_on_stderr());
            }
        } else if how.buffer {
            mark_stdin_cloexec();
        } else {
            drop(input); // errors are ignored
        }
        drop(w);

        if let Some(name) = tmpname.take() {
            os_remove(name.as_cstr()); // delete the converted file
        }
        no_wait_return.set(no_wait_return.get() - 1); // may wait for return now

        // In recovery mode everything but autocommands is skipped.
        if !recoverymode.get() {
            // The last line, which came from the empty buffer, has to go.
            if how.newfile && wasempty && !Buf::current().b_ml.ml_flags.has(MlFlags::EMPTY) {
                let _ = ml_delete(Buf::current().b_ml.ml_line_count);
                linecnt -= 1;
            }
            Buf::current().deleted_bytes = 0;
            Buf::current().deleted_bytes2 = 0;
            Buf::current().deleted_codepoints = 0;
            Buf::current().deleted_codeunits = 0;
            linecnt = Buf::current().b_ml.ml_line_count - linecnt;
            if filesize == 0 {
                linecnt = 0;
            }
            if how.newfile || how.buffer {
                redraw_curbuf_later(UPD_NOT_VALID as c_int);
                // The diff info needs updating now that the text is in.
                diff_invalidate(Buf::current());
                // All folds in the window are invalid now. Mark them for
                // update before triggering autocommands.
                fold_update_all(Win::current());
            } else if linecnt != 0 {
                // At least one line was appended.
                appended_lines_mark(from, linecnt);
            }

            if got_int.get() {
                if !how.dummy {
                    filemess(Buf::current(), sfname, gettext(e_interr));
                    if how.newfile {
                        Buf::current().b_p_ro = true as c_int; // must use "w!" now
                    }
                }
                msg_scroll.set(msg_save);
                check_marks_read();
                retval = Ok(Loaded::Read); // an interrupt isn't really an error
                break 'theend;
            }

            let out = Outcome {
                perm,
                read_no_eol_lnum,
                ff_error,
                split,
                notconverted,
                converted,
                conv_error: conv.conv_error,
                illegal_byte,
                error,
                fileformat,
                linecnt,
                filesize,
            };
            report_and_place(sfname, how, silent, &out, conv.bad_char, from);
        }
        msg_scroll.set(msg_save);

        // Get the marks before running autocommands, so they can use them.
        check_marks_read();

        // Remember whether the last line read had no end-of-line, even
        // when 'binary' is off, to support turning 'fixeol' off or writing
        // the same text again with 'binary' on. The latter is needed for
        // ":autocmd FileReadPost *.gz set bin|'[,']!gunzip" to work.
        Buf::current().b_no_eol_lnum = read_no_eol_lnum;

        // When reloading a buffer put the cursor on the first line that
        // differs.
        if how.keep_undo {
            u_find_first_changed();
        }

        // When opening a new file, locate the undo info and read it.
        if read_undo_file {
            let hash = sha_ctx.finish();
            u_read_undo(None, &hash, fname);
        }

        if !how.stdin
            && !how.fifo
            && (!how.buffer || sfname.is_some())
            && !run_read_autocmds(sfname, excmd, how, set_options)
        {
            // Autocommands may abort script processing. Note that this
            // skips the swap-file sync below, as upstream does.
            return Err(Failed);
        }

        if !(recoverymode.get() && error) {
            retval = Ok(Loaded::Read);
        }
    }

    if Buf::current().memfile_dirty() == Some(MfDirty::YesNoSync) {
        // It is OK to sync the swap file now.
        Buf::current().set_memfile_dirty(MfDirty::Yes);
    }
    retval
}

/// Rewind the file being read to its start: whether that worked.
fn rewind(input: Option<&File>) -> bool {
    input.is_some_and(|mut file| file.seek(SeekFrom::Start(0)).is_ok_and(|at| at == 0))
}

/// One `read()` into `into`, retried when a signal interrupts it (which
/// `SIGWINCH` makes routine): the count, 0 at the end of the file, or -1.
fn read_retrying(file: &mut File, into: &mut [u8]) -> isize {
    loop {
        match file.read(into) {
            Ok(n) => return n as isize,
            Err(err) if err.kind() == ErrorKind::Interrupted => {}
            Err(_) => return -1,
        }
    }
}

/// Fill the read window from the current buffer's first `last` lines, at
/// most `w.size` bytes, carrying on where the previous fill stopped. Used to
/// convert what stdin gave us. Answers how many bytes it put there.
fn read_from_buffer(w: &mut ReadBuf, last: LineNr, lnum: &mut LineNr, col: &mut usize) -> isize {
    if *lnum > last {
        return 0;
    }
    let room = w.len();
    let mut tlen = 0;
    let mut lines = Buf::current().lines();
    loop {
        let line = &lines.line(*lnum)[*col..];
        // Filled up to "size"? Then only a part of the line fits.
        let partial = tlen + line.len() + 1 > room;
        let n = if partial { room - tlen } else { line.len() };
        // Change NL to NUL to reverse the effect done below.
        for (to, &b) in w.bytes[w.ptr + tlen..w.ptr + tlen + n]
            .iter_mut()
            .zip(&line[..n])
        {
            *to = if b == NL as u8 { 0 } else { b };
        }
        tlen += n;
        if partial {
            *col += n;
            return tlen as isize;
        }
        // Append the whole line and a newline.
        w.bytes[w.ptr + tlen] = NL as u8;
        tlen += 1;
        *col = 0;
        *lnum += 1;
        if *lnum > last {
            // When the last line had no end-of-line, don't add one now.
            if Buf::current().b_p_eol == 0 {
                tlen -= 1;
            }
            return tlen as isize;
        }
    }
}
