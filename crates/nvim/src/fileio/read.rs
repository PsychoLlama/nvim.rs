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

use core::ffi::{c_char, c_int};

use super::*;
use crate::edit::BeginlineOpts;
use crate::ex_docmd::cmdmod_has;
use crate::keycodes::Ctrl_Z;
use crate::memfile::MfDirty;
use crate::option::cpo_has;
use crate::pos::MAXCOL;
use crate::types::{CmdModFlags, CpoFlag, FAIL, OK, OptionSetFlags};
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

/// Read the lines of `fname` into the current buffer, after line `from`.
///
/// The caller must check that `fname` is not NULL unless `READ_STDIN` is
/// used. `eap` may be NULL. When not recovering, `lines_to_skip` is 0 and
/// `lines_to_read` is `MAXLNUM`.
///
/// @return  FAIL for failure, NOTDONE for a directory (also a failure), or OK
pub unsafe fn readfile(
    fname: *mut c_char,
    sfname: *mut c_char,
    from: linenr_T,
    lines_to_skip: linenr_T,
    lines_to_read: linenr_T,
    eap: *mut exarg_T,
    flags: c_int,
    silent: bool,
) -> c_int {
    unsafe {
        let mut fname = fname;
        let mut sfname = sfname;
        let mut retval = FAIL;
        let how = How {
            newfile: flags & READ_NEW as c_int != 0,
            filtering: flags & READ_FILTER as c_int != 0,
            stdin: flags & READ_STDIN as c_int != 0,
            buffer: flags & READ_BUFFER as c_int != 0,
            fifo: flags & READ_FIFO as c_int != 0,
            dummy: flags & READ_DUMMY as c_int != 0,
            nofile: flags & READ_NOFILE as c_int != 0,
            keep_undo: flags & READ_KEEP_UNDO as c_int != 0,
            set_options: flags & (READ_NEW | READ_BUFFER) as c_int != 0
                || (!eap.is_null() && (*eap).read_edit != 0),
        };
        let set_options = how.set_options;

        // Where the next line and character to read from curbuf are, for
        // READ_BUFFER.
        let mut read_buf_lnum: linenr_T = 1;
        let mut read_buf_col: colnr_T = 0;

        let mut lnum = from;
        let mut w = Window {
            buffer: core::ptr::null_mut(),
            ptr: core::ptr::null_mut(),
            line_start: core::ptr::null_mut(),
            size: 0,
            real_size: 0,
            linerest: 0,
        };
        let mut filesize: off_T = 0;
        let mut skip_read = false;
        let mut sha_ctx = Sha256::new();
        let mut read_undo_file = false;
        let mut split = 0; // number of split lines
        let mut error = false;
        let mut ff_error = EOL_UNKNOWN; // file format with errors
        let mut fileformat = 0;
        let mut keep_fileformat = false;
        let mut skip_count: linenr_T = 0;
        let mut read_count: linenr_T = 0;
        let msg_save = msg_scroll.get();
        // Non-zero line number when the last line read had no end-of-line.
        let mut read_no_eol_lnum: linenr_T = 0;
        let mut file_rewind = false;
        let mut illegal_byte: linenr_T = 0; // line nr with an illegal byte
        // Don't retry when a character doesn't fit in the destination
        // encoding.
        let mut keep_dest_enc = false;
        let mut tmpname: *mut c_char = core::ptr::null_mut();
        let mut fenc: *mut c_char = core::ptr::null_mut();
        let mut fenc_alloced = false;
        let mut fenc_next: *mut c_char = core::ptr::null_mut();
        let mut advance_fenc = false;
        let mut did_iconv = false; // iconv() failed, try 'charconvert' next
        let mut converted = false;
        let mut notconverted = false;
        let mut conv = Conv::new(BAD_REPLACE);
        let mut linecnt: linenr_T = 0;
        let mut wasempty = 0;

        // Reset before triggering any autocommands.
        (*curbuf.get()).b_au_did_filetype = false;
        // In case it was set by the previous read.
        (*curbuf.get()).b_no_eol_lnum = 0;

        'theend: {
            // If there is no file name yet, use the one for the read file, and
            // set BufFlags::NOTEDITED to reflect that. Not for a read from a filter,
            // and only when 'cpoptions' contains the 'f' flag.
            if (*curbuf.get()).b_ffname.is_null()
                && !how.filtering
                && !fname.is_null()
                && cpo_has(CpoFlag::FNAMER)
                && !how.dummy
                && set_rw_fname(fname, sfname) == FAIL
            {
                break 'theend;
            }

            let Opened {
                fname,
                sfname,
                mut fd,
                perm,
                mut guess,
            } = match open_source(fname, sfname, from, eap, how, silent, msg_save) {
                Ok(opened) => opened,
                Err(early) => {
                    retval = early;
                    break 'theend;
                }
            };

            // Autocommands may have added lines, so check whether the buffer
            // is empty now.
            wasempty = (*curbuf.get()).b_ml.ml_flags & ML_EMPTY;

            if !recoverymode.get()
                && !how.filtering
                && !how.dummy
                && !silent
                && !how.stdin
                && !how.buffer
            {
                filemess(curbuf.get(), sfname, c"".as_ptr().cast_mut());
            }

            msg_scroll.set(false as c_int); // overwrite the file message

            // Set linecnt now, before the "retry" caused by a wrong guess at
            // the fileformat, and after the autocommands, which may change it.
            linecnt = (*curbuf.get()).b_ml.ml_line_count;

            // The "++bad=" argument.
            if !eap.is_null() && (*eap).bad_char != 0 {
                conv.bad_char = (*eap).bad_char;
                if set_options {
                    (*curbuf.get()).b_bad_char = (*eap).bad_char;
                }
            } else {
                (*curbuf.get()).b_bad_char = 0;
            }

            // Decide which 'fileencoding' to use, or to start with.
            if !eap.is_null() && (*eap).force_enc != 0 {
                fenc = enc_canonize((*eap).cmd.offset((*eap).force_enc as isize));
                fenc_alloced = true;
                keep_dest_enc = true;
            } else if (*curbuf.get()).b_p_bin != 0 {
                fenc = c"".as_ptr().cast_mut(); // binary: don't convert
                fenc_alloced = false;
            } else if (*curbuf.get()).b_help {
                // Help files are either utf-8 or latin1. Try utf-8 first; if
                // that fails it must be latin1. This is needed when the first
                // line has non-ASCII characters, which happens only in *.??x
                // files.
                fenc_next = c"latin1".as_ptr().cast_mut();
                fenc = c"utf-8".as_ptr().cast_mut();
                fenc_alloced = false;
            } else if *p_fencs.get() == 0 {
                fenc = (*curbuf.get()).b_p_fenc; // use the buffer's encoding
                fenc_alloced = false;
            } else {
                fenc_next = p_fencs.get(); // try the items in 'fileencodings'
                fenc = next_fenc(&mut fenc_next, &mut fenc_alloced);
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
            // just settled on. A non-null "tmpname" means 'charconvert' made a
            // file that has to be deleted afterwards.
            'retry: loop {
                if file_rewind {
                    if how.buffer {
                        read_buf_lnum = 1;
                        read_buf_col = 0;
                    } else if how.stdin || lseek(fd, 0, SEEK_SET) != 0 {
                        // Can't rewind the file, give up.
                        error = true;
                        break 'retry;
                    }
                    // Delete the lines read so far.
                    while lnum > from {
                        ml_delete(lnum);
                        lnum -= 1;
                    }
                    file_rewind = false;
                    if set_options {
                        (*curbuf.get()).b_p_bomb = false as c_int;
                        (*curbuf.get()).b_start_bomb = false as c_int;
                    }
                    conv.conv_error = 0;
                }

                // The "fileformat" is reset the first time round, and whenever
                // we are retrying with another "fenc".
                if keep_fileformat {
                    keep_fileformat = false;
                } else {
                    if !eap.is_null() && (*eap).force_ff != 0 {
                        fileformat = get_fileformat_force(curbuf.get(), eap);
                        guess.try_unix = 0;
                        guess.try_dos = false;
                        guess.try_mac = 0;
                    } else if (*curbuf.get()).b_p_bin != 0 {
                        fileformat = EOL_UNIX; // binary: use Unix format
                    } else if *p_ffs.get() == 0 {
                        fileformat = get_fileformat(curbuf.get()); // from the buffer
                    } else {
                        fileformat = EOL_UNKNOWN; // detect from the file
                    }
                }

                // An aborted iconv() conversion: close the descriptor.
                conv.close_iconv();

                if advance_fenc {
                    // Try the next entry in 'fileencodings'.
                    advance_fenc = false;

                    if !eap.is_null() && (*eap).force_enc != 0 {
                        // The conversion given with "++enc=" wasn't possible;
                        // read without conversion.
                        notconverted = true;
                        conv.conv_error = 0;
                        if fenc_alloced {
                            xfree(fenc.cast());
                        }
                        fenc = c"".as_ptr().cast_mut();
                        fenc_alloced = false;
                    } else {
                        if fenc_alloced {
                            xfree(fenc.cast());
                        }
                        if fenc_next.is_null() {
                            fenc = c"".as_ptr().cast_mut();
                            fenc_alloced = false;
                        } else {
                            fenc = next_fenc(&mut fenc_next, &mut fenc_alloced);
                        }
                    }
                    if !tmpname.is_null() {
                        os_remove(tmpname); // delete the converted file
                        xfree(tmpname.cast());
                        tmpname = core::ptr::null_mut();
                    }
                }

                // Conversion is needed when the file's encoding differs from
                // 'encoding', or 'encoding' is UTF-16, UCS-2 or UCS-4.
                conv.flags = 0;
                converted = need_conversion(fenc);
                if converted {
                    if strcmp(fenc, ENC_UCSBOM.as_ptr()) == 0 {
                        // "ucs-bom" means the first bytes of the file decide.
                        conv.flags = FIO_UCSBOM;
                    } else {
                        // Work out whether UCS-2/4 or Latin1 to UTF-8 can be
                        // done here rather than by iconv, which appears not to
                        // handle Unicode to Latin1 correctly. Doing it now
                        // saves parsing the name on every read.
                        conv.flags = get_fio_flags(fenc);
                    }

                    // Try iconv() if we can't convert it ourselves.
                    if conv.flags == 0 && !did_iconv {
                        conv.open_iconv(fenc);
                    }

                    // Use the 'charconvert' expression when conversion is
                    // needed and neither of the above can do it.
                    if conv.flags == 0
                        && !how.stdin
                        && !how.buffer
                        && *p_ccv.get() != 0
                        && !how.fifo
                        && !conv.has_iconv()
                    {
                        did_iconv = false;
                        // Skip the conversion when it is already done (a retry
                        // for a wrong "fileformat").
                        if tmpname.is_null() {
                            tmpname = readfile_charconvert(fname, fenc, &mut fd);
                            if tmpname.is_null() {
                                // Conversion failed. Try another one.
                                advance_fenc = true;
                                if fd < 0 {
                                    // Re-opening the original file failed!
                                    emsg(gettext(
                                        c"E202: Conversion made file unreadable!".as_ptr(),
                                    ));
                                    error = true;
                                    break 'retry;
                                }
                                continue 'retry;
                            }
                        }
                    } else if conv.flags == 0 && !conv.has_iconv() {
                        // Conversion is wanted but we can't do it; try the next
                        // entry in 'fileencodings'.
                        advance_fenc = true;
                        continue 'retry;
                    }
                }

                // Rewinding the file and trying another "fenc" is possible
                // unless there is no other one to try, or we are reading stdin
                // or a fifo, or the encoding is fixed.
                conv.can_retry = *fenc != 0 && !how.stdin && !keep_dest_enc && !how.fifo;
                conv.linecnt = linecnt;

                if !skip_read {
                    w.linerest = 0;
                    filesize = 0;
                    skip_count = lines_to_skip;
                    read_count = lines_to_read;
                    conv.restlen = 0;
                    read_undo_file = how.newfile
                        && !how.keep_undo
                        && !(*curbuf.get()).b_ffname.is_null()
                        && (*curbuf.get()).b_p_udf != 0
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
                        w.size = (0x10000 + w.linerest).min(0x100000);
                    }

                    // Protect against the argument of lalloc() going negative,
                    // and split lines that are too long for colnr_T. After this
                    // check we read up to "size" more bytes, and even then the
                    // line length must stay below MAXCOL - 1 (we add 1 for the
                    // NUL when casting to colnr_T). If it fires we insert a
                    // newline right away, so linerest does not grow.
                    if w.size < 0
                        || w.size + w.linerest + 1 < 0
                        || w.linerest >= MAXCOL as ptrdiff_t - w.size
                    {
                        split += 1;
                        *w.ptr = NL as c_char; // split the line by inserting a NL
                        w.size = 1;
                    } else if !skip_read {
                        let mut new_buffer: *mut c_char = core::ptr::null_mut();
                        while w.size >= 10 {
                            new_buffer =
                                verbose_try_malloc(w.size as usize + w.linerest as usize + 1)
                                    .cast();
                            if !new_buffer.is_null() {
                                break;
                            }
                            w.size /= 2;
                        }
                        if new_buffer.is_null() {
                            error = true;
                            break;
                        }
                        if w.linerest != 0 {
                            // Copy the characters from the previous buffer.
                            core::ptr::copy(
                                w.ptr.offset(-w.linerest),
                                new_buffer,
                                w.linerest as usize,
                            );
                        }
                        xfree(w.buffer.cast());
                        w.buffer = new_buffer;
                        w.ptr = w.buffer.offset(w.linerest);
                        w.line_start = w.buffer;

                        // We may need room to translate into. For iconv() we
                        // don't really know how much, so use a factor.
                        // latin1 to utf-8: 1 byte becomes up to 2 bytes.
                        // utf-16 to utf-8: 2 bytes become up to 3, 4 become up
                        // to 4, and the size must be a multiple of 2.
                        // ucs-2 to utf-8: 2 bytes become up to 3, size must be
                        // a multiple of 2.
                        // ucs-4 to utf-8: 4 bytes become up to 6, size must be
                        // a multiple of 4.
                        w.real_size = w.size as c_int;
                        if conv.has_iconv() {
                            w.size /= ICONV_MULT as ptrdiff_t;
                        } else if conv.flags & FIO_LATIN1 != 0 {
                            w.size /= 2;
                        } else if conv.flags & (FIO_UCS2 | FIO_UTF16) != 0 {
                            w.size = (w.size * 2 / 3) & !1;
                        } else if conv.flags & FIO_UCS4 != 0 {
                            w.size = (w.size * 2 / 3) & !3;
                        } else if conv.flags == FIO_UCSBOM {
                            w.size /= ICONV_MULT as ptrdiff_t; // worst case
                        }

                        if conv.restlen > 0 {
                            // Put the unconverted bytes from last time first.
                            conv.restore(w.ptr);
                            w.ptr = w.ptr.offset(conv.restlen as isize);
                            w.size -= conv.restlen as ptrdiff_t;
                        }

                        if how.buffer {
                            // Read bytes from curbuf. Used for converting text
                            // read from stdin.
                            if read_buf_lnum > from {
                                w.size = 0;
                            } else {
                                let mut tlen: ptrdiff_t = 0;
                                loop {
                                    let line = ml_get(read_buf_lnum);
                                    let p = line.add(read_buf_col as usize).cast::<u8>();
                                    let whole = ml_get_len(read_buf_lnum) - read_buf_col;
                                    // Filled up to "size"? Then only a part of
                                    // the line fits.
                                    let partial = tlen + whole as ptrdiff_t + 1 > w.size;
                                    let n = if partial {
                                        (w.size - tlen) as c_int
                                    } else {
                                        whole
                                    };
                                    // Change NL to NUL to reverse the effect
                                    // done below.
                                    for ni in 0..n as usize {
                                        let b = *p.add(ni);
                                        *w.ptr.offset(tlen) =
                                            if b == NL as u8 { 0 } else { b as c_char };
                                        tlen += 1;
                                    }
                                    if partial {
                                        read_buf_col += n;
                                        break;
                                    }
                                    // Append the whole line and a newline.
                                    *w.ptr.offset(tlen) = NL as c_char;
                                    tlen += 1;
                                    read_buf_col = 0;
                                    read_buf_lnum += 1;
                                    if read_buf_lnum > from {
                                        // When the last line had no
                                        // end-of-line, don't add one now.
                                        if (*curbuf.get()).b_p_eol == 0 {
                                            tlen -= 1;
                                        }
                                        w.size = tlen;
                                        break;
                                    }
                                }
                            }
                        } else {
                            // Read bytes from the file.
                            w.size = read_eintr(fd, w.ptr.cast(), w.size as size_t);
                        }

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
                                            (*curbuf.get()).b_ml.ml_line_count - linecnt + 1;
                                    }
                                } else if illegal_byte == 0 {
                                    // Remember the first line with an illegal
                                    // byte.
                                    illegal_byte = (*curbuf.get()).b_ml.ml_line_count - linecnt + 1;
                                }
                                if conv.bad_char == BAD_DROP {
                                    *w.ptr.offset(-(conv.restlen as isize)) = 0;
                                    conv.restlen = 0;
                                } else {
                                    // Replace the trailing bytes with the
                                    // replacement character if we were
                                    // converting; if we weren't, leave it to
                                    // the UTF-8 check, which works slightly
                                    // differently.
                                    if conv.bad_char != BAD_KEEP && conv.active() {
                                        while conv.restlen > 0 {
                                            w.ptr = w.ptr.offset(-1);
                                            *w.ptr = conv.bad_char as c_char;
                                            conv.restlen -= 1;
                                        }
                                    }
                                    conv.flags = 0; // don't convert this
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
                        && (conv.flags == FIO_UCSBOM
                            || ((*curbuf.get()).b_p_bomb == 0
                                && tmpname.is_null()
                                && (*fenc == b'u' as c_char || *fenc == 0)))
                    {
                        // No BOM detection in a short file or in binary mode.
                        let found = if w.size < 2 || (*curbuf.get()).b_p_bin != 0 {
                            None
                        } else {
                            check_for_bom(
                                core::slice::from_raw_parts(w.ptr.cast::<u8>(), w.size as usize),
                                if conv.flags == FIO_UCSBOM {
                                    FIO_ALL
                                } else {
                                    get_fio_flags(fenc)
                                },
                            )
                        };
                        if let Some((_, blen)) = found {
                            // Remove the BOM from the text.
                            filesize += blen as off_T;
                            w.size -= blen as ptrdiff_t;
                            core::ptr::copy(w.ptr.add(blen), w.ptr, w.size as usize);
                            if set_options {
                                (*curbuf.get()).b_p_bomb = true as c_int;
                                (*curbuf.get()).b_start_bomb = true as c_int;
                            }
                        }

                        if conv.flags == FIO_UCSBOM {
                            match found {
                                None => advance_fenc = true, // retry with the next encoding
                                Some((name, _)) => {
                                    // A BOM was found: set "fenc" and start over.
                                    if fenc_alloced {
                                        xfree(fenc.cast());
                                    }
                                    fenc = name.as_ptr().cast_mut();
                                    fenc_alloced = false;
                                }
                            }
                            // Retry without reading new bytes or rewinding.
                            skip_read = true;
                            continue 'retry;
                        }
                    }

                    // Include the bytes that were not converted last time.
                    w.ptr = w.ptr.offset(-(conv.restlen as isize));
                    w.size += conv.restlen as ptrdiff_t;
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

                    if conv.flags != 0 {
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
                    } else if (*curbuf.get()).b_p_bin == 0
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
                    filesize += w.size as off_T;

                    // When reading the first part of a file, guess the EOL
                    // type.
                    if fileformat == EOL_UNKNOWN {
                        fileformat = guess.guess(w.ptr, w.size);
                        // May set 'fileformat' when editing a new file.
                        if set_options {
                            set_fileformat(fileformat, OptionSetFlags::LOCAL);
                        }
                    }

                    match split_lines(
                        &mut w,
                        &mut Lines {
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
                            fd,
                            set_options,
                        },
                        &mut lnum,
                        &mut skip_count,
                        &mut read_count,
                        &mut fileformat,
                        &mut ff_error,
                    ) {
                        Split::Done => {}
                        Split::Stop => error = true,
                        Split::RetryUnix => {
                            file_rewind = true;
                            keep_fileformat = true;
                            continue 'retry;
                        }
                    }
                    w.linerest = w.ptr.offset_from(w.line_start);
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
                && (*curbuf.get()).b_p_bin == 0
                && fileformat == EOL_DOS
                && *w.ptr.offset(-1) == Ctrl_Z as c_char
            {
                w.ptr = w.ptr.offset(-1);
                w.linerest -= 1;
                if set_options {
                    (*curbuf.get()).b_p_eof = true as c_int;
                }
            }

            // If we hit end of file in the middle of a line, note that and
            // complete the line ourselves.
            if !error && !got_int.get() && w.linerest != 0 {
                // Remember it for when writing.
                if set_options {
                    (*curbuf.get()).b_p_eol = false as c_int;
                }
                *w.ptr = 0;
                let len = (w.ptr.offset_from(w.line_start) + 1) as colnr_T;
                if ml_append(lnum, w.line_start, len, how.newfile) == FAIL {
                    error = true;
                } else {
                    if read_undo_file {
                        sha_ctx.update(core::slice::from_raw_parts(
                            w.line_start.cast::<u8>(),
                            len as usize,
                        ));
                    }
                    lnum += 1;
                    read_no_eol_lnum = lnum;
                }
            }

            if set_options {
                // Remember the current file format.
                save_file_ff(curbuf.get());
                // When editing a new file set 'fileencoding' for this buffer.
                // Also for ":read ++edit file".
                set_option_direct(
                    kOptFileencoding,
                    OptVal {
                        type_0: kOptValTypeString,
                        data: OptValData {
                            string: cstr_as_string(fenc),
                        },
                    },
                    OptionSetFlags::LOCAL,
                    0 as scid_T,
                );
            }
            if fenc_alloced {
                xfree(fenc.cast());
            }
            conv.close_iconv();

            if !how.buffer && !how.stdin {
                close(fd); // errors are ignored
            } else {
                os_set_cloexec(fd);
            }
            xfree(w.buffer.cast());

            if how.stdin {
                close(fd);
                if stdin_fd.get() < 0 {
                    // Use stderr for stdin, which makes shell commands work.
                    vim_ignored.set(dup(2));
                }
            }

            if !tmpname.is_null() {
                os_remove(tmpname); // delete the converted file
                xfree(tmpname.cast());
            }
            (*no_wait_return.ptr()) -= 1; // may wait for return now

            // In recovery mode everything but autocommands is skipped.
            if !recoverymode.get() {
                // The last line, which came from the empty buffer, has to go.
                if how.newfile && wasempty != 0 && (*curbuf.get()).b_ml.ml_flags & ML_EMPTY == 0 {
                    ml_delete((*curbuf.get()).b_ml.ml_line_count);
                    linecnt -= 1;
                }
                (*curbuf.get()).deleted_bytes = 0;
                (*curbuf.get()).deleted_bytes2 = 0;
                (*curbuf.get()).deleted_codepoints = 0;
                (*curbuf.get()).deleted_codeunits = 0;
                linecnt = (*curbuf.get()).b_ml.ml_line_count - linecnt;
                if filesize == 0 {
                    linecnt = 0;
                }
                if how.newfile || how.buffer {
                    redraw_curbuf_later(UPD_NOT_VALID as c_int);
                    // The diff info needs updating now that the text is in.
                    diff_invalidate(curbuf.get());
                    // All folds in the window are invalid now. Mark them for
                    // update before triggering autocommands.
                    fold_update_all(curwin.get());
                } else if linecnt != 0 {
                    // At least one line was appended.
                    appended_lines_mark(from, linecnt);
                }

                if got_int.get() {
                    if !how.dummy {
                        filemess(curbuf.get(), sfname, gettext(e_interr.as_ptr()));
                        if how.newfile {
                            (*curbuf.get()).b_p_ro = true as c_int; // must use "w!" now
                        }
                    }
                    msg_scroll.set(msg_save);
                    check_marks_read();
                    retval = OK; // an interrupt isn't really an error
                    break 'theend;
                }

                if !how.filtering && !how.dummy && !silent {
                    report_read(
                        sfname,
                        how,
                        &Outcome {
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
                        },
                    );
                }

                // With errors, writing the file requires ":w!".
                if how.newfile
                    && (error
                        || conv.conv_error != 0
                        || (illegal_byte > 0 && conv.bad_char != BAD_KEEP))
                {
                    (*curbuf.get()).b_p_ro = true as c_int;
                }

                u_clearline(curbuf.get()); // "U" cannot be used after adding lines

                // In Ex mode the cursor goes on the last new line, otherwise
                // on the first one.
                (*curwin.get()).w_cursor.lnum = if exmode_active.get() {
                    from + linecnt
                } else {
                    from + 1
                };
                check_cursor_lnum(curwin.get());
                beginline(BeginlineOpts::WHITE | BeginlineOpts::FIX); // on the first non-blank

                if !cmdmod_has(CmdModFlags::LOCKMARKS) {
                    // Set the '[ and '] marks to the newly read lines.
                    (*curbuf.get()).b_op_start.lnum = from + 1;
                    (*curbuf.get()).b_op_start.col = 0;
                    (*curbuf.get()).b_op_end.lnum = from + linecnt;
                    (*curbuf.get()).b_op_end.col = 0;
                }
            }
            msg_scroll.set(msg_save);

            // Get the marks before running autocommands, so they can use them.
            check_marks_read();

            // Remember whether the last line read had no end-of-line, even
            // when 'binary' is off, to support turning 'fixeol' off or writing
            // the same text again with 'binary' on. The latter is needed for
            // ":autocmd FileReadPost *.gz set bin|'[,']!gunzip" to work.
            (*curbuf.get()).b_no_eol_lnum = read_no_eol_lnum;

            // When reloading a buffer put the cursor on the first line that
            // differs.
            if how.keep_undo {
                u_find_first_changed();
            }

            // When opening a new file, locate the undo info and read it.
            if read_undo_file {
                let mut hash = sha_ctx.finish();
                u_read_undo(core::ptr::null_mut(), hash.as_mut_ptr(), fname);
            }

            if !how.stdin && !how.fifo && (!how.buffer || !sfname.is_null()) {
                let m = msg_scroll.get();
                let n = msg_scrolled.get();

                // Save the fileformat now, or the buffer would be considered
                // modified because the format or encoding was auto-detected.
                if set_options {
                    save_file_ff(curbuf.get());
                }

                // The output from the autocommands should neither overwrite
                // anything nor be overwritten: set msg_scroll, and restore it
                // if no output was done.
                msg_scroll.set(true as c_int);
                if how.filtering {
                    apply_autocmds_exarg(
                        EVENT_FILTERREADPOST,
                        core::ptr::null_mut(),
                        sfname,
                        false,
                        curbuf.get(),
                        eap,
                    );
                } else if how.newfile || (how.buffer && !sfname.is_null()) {
                    apply_autocmds_exarg(
                        EVENT_BUFREADPOST,
                        core::ptr::null_mut(),
                        sfname,
                        false,
                        curbuf.get(),
                        eap,
                    );
                    if !(*curbuf.get()).b_au_did_filetype && *(*curbuf.get()).b_p_ft != 0 {
                        // EVENT_FILETYPE was not triggered but the buffer
                        // already has a filetype; trigger it with that one.
                        apply_autocmds(
                            EVENT_FILETYPE,
                            (*curbuf.get()).b_p_ft,
                            (*curbuf.get()).b_fname,
                            true,
                            curbuf.get(),
                        );
                    }
                } else {
                    apply_autocmds_exarg(
                        EVENT_FILEREADPOST,
                        sfname,
                        sfname,
                        false,
                        core::ptr::null_mut(),
                        eap,
                    );
                }
                if msg_scrolled.get() == n {
                    msg_scroll.set(m);
                }
                if aborting() {
                    // Autocommands may abort script processing. Note that this
                    // skips the swap-file sync below, as upstream does.
                    return FAIL;
                }
            }

            if !(recoverymode.get() && error) {
                retval = OK;
            }
        }

        let mfp = (*curbuf.get()).b_ml.ml_mfp;
        if !mfp.is_null() && (*mfp).mf_dirty == MfDirty::YesNoSync {
            // It is OK to sync the swap file now.
            (*mfp).mf_dirty = MfDirty::Yes;
        }
        retval
    }
}
