//! The passes over the file: read it block by block into the buffer, and
//! start again when a guess turns out wrong.
//!
//! Three guesses can fail part-way through a file. The encoding: the
//! conversion fails, so the next entry of `'fileencodings'` is tried (or
//! `'charconvert'` after iconv). A byte-order mark: it names the encoding,
//! which is then set up for it. The line ending: a CR shows up where the
//! guess said Unix, so the file is read again keeping the format just
//! settled on. [`Reader`] holds what survives from one pass to the next;
//! each pass is [`Reader::start_pass`] and then [`Reader::read_blocks`].

#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use super::*;

/// Where a step of a pass leaves the read.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Next {
    /// Carry on with this pass.
    Go,
    /// Start another pass.
    Retry,
    /// The read is over, finished or failed.
    Done,
}

/// A read in progress: the file, the window its bytes go through, the
/// guesses made so far and what to do differently on the next pass.
#[derive(Debug)]
pub(crate) struct Reader<'a> {
    pub how: &'a How,
    /// The lines go after this one.
    from: LineNr,
    lines_to_skip: i32,
    lines_to_read: i32,
    /// The name 'charconvert' converts from.
    fname: Option<&'a CStr>,
    /// The file, or stdin; `None` when reading from the buffer.
    pub input: Option<File>,
    pub guess: FormatGuess,
    /// The last line appended.
    pub lnum: LineNr,
    pub w: ReadBuf,
    pub conv: Conv,
    /// Bytes read, after conversion.
    pub filesize: FileOffset,
    /// The undo file is read too, by a hash of the text.
    pub read_undo_file: bool,
    pub sha_ctx: Sha256,
    /// How many lines were split for being too long.
    pub split: c_int,
    pub error: bool,
    /// The file format that had errors.
    pub ff_error: c_int,
    pub fileformat: c_int,
    skip_count: i32,
    pub read_count: i32,
    /// The first line with an illegal byte.
    pub illegal_byte: LineNr,
    /// The line count before the read; after the autocommands, which may
    /// change it.
    pub linecnt: i32,
    /// The encoding being tried.
    pub fenc: XString,
    /// The rest of 'fileencodings' still to try.
    fencs: FencList,
    /// The conversion is needed: 'fileencoding' is not the buffer's.
    pub converted: bool,
    /// "++enc=" asked for a conversion that was not possible.
    pub notconverted: bool,
    /// The file 'charconvert' wrote, to be deleted afterwards.
    pub tmpname: Option<XString>,
    /// Don't retry when a character doesn't fit in the destination
    /// encoding.
    keep_dest_enc: bool,
    /// Re-use the bytes already read, after a BOM.
    skip_read: bool,
    /// Rewind the file and read it again.
    file_rewind: bool,
    /// Keep the file format just settled on.
    keep_fileformat: bool,
    /// Move on to the next "fenc".
    advance_fenc: bool,
    /// iconv() failed: 'charconvert' is next.
    did_iconv: bool,
    /// Where the next line and character come from, for `READ_BUFFER`.
    read_buf_lnum: LineNr,
    read_buf_col: usize,
}

impl<'a> Reader<'a> {
    /// Ready to read after line `from`, with the line count, the "++bad="
    /// argument and the first encoding to try decided.
    pub(crate) fn new(
        how: &'a How,
        from: LineNr,
        (lines_to_skip, lines_to_read): (i32, i32),
        fname: Option<&'a CStr>,
        input: Option<File>,
        guess: FormatGuess,
        excmd: Option<&ExArg>,
    ) -> Self {
        let mut conv = Conv::new(BAD_REPLACE);
        // The "++bad=" argument.
        if let Some(command) = excmd
            && command.bad_char != 0
        {
            conv.bad_char = command.bad_char;
            if how.set_options {
                Buf::current().b_bad_char = command.bad_char;
            }
        } else {
            Buf::current().b_bad_char = 0;
        }
        let (fenc, fencs, keep_dest_enc) = first_encoding(excmd);
        Reader {
            how,
            from,
            lines_to_skip,
            lines_to_read,
            fname,
            input,
            guess,
            lnum: from,
            w: ReadBuf::new(),
            conv,
            filesize: 0,
            read_undo_file: false,
            sha_ctx: Sha256::new(),
            split: 0,
            error: false,
            ff_error: EOL_UNKNOWN,
            fileformat: 0,
            skip_count: 0,
            read_count: 0,
            illegal_byte: LineNr::ZERO,
            // Set now, before the "retry" caused by a wrong guess at the
            // fileformat, and after the autocommands, which may change it.
            linecnt: Buf::current().b_ml.ml_line_count.get(),
            fenc,
            fencs,
            converted: false,
            notconverted: false,
            tmpname: None,
            keep_dest_enc,
            skip_read: false,
            file_rewind: false,
            keep_fileformat: false,
            advance_fenc: false,
            did_iconv: false,
            read_buf_lnum: LineNr::ONE,
            read_buf_col: 0,
        }
    }

    /// Read the file, as many passes as the guesses need.
    pub(crate) fn read_passes(&mut self, excmd: Option<&ExArg>) {
        loop {
            match self.start_pass(excmd) {
                Next::Go => {}
                Next::Retry => continue,
                Next::Done => return,
            }
            if self.read_blocks() != Next::Retry {
                return;
            }
        }
    }

    /// Set up a pass: rewind when asked to, reset the file format, move on
    /// to the next encoding, and set up its conversion.
    fn start_pass(&mut self, excmd: Option<&ExArg>) -> Next {
        if self.file_rewind && !self.rewind_for_retry() {
            // Can't rewind the file, give up.
            self.error = true;
            return Next::Done;
        }

        // The "fileformat" is reset the first time round, and whenever we
        // are retrying with another "fenc".
        if self.keep_fileformat {
            self.keep_fileformat = false;
        } else {
            self.reset_fileformat(excmd);
        }

        // An aborted iconv() conversion: close the descriptor.
        self.conv.close_iconv();

        if self.advance_fenc {
            self.next_encoding(excmd);
        }

        match self.set_up_conversion() {
            Next::Go => {}
            next => return next,
        }

        // Rewinding the file and trying another "fenc" is possible unless
        // there is no other one to try, or we are reading stdin or a fifo,
        // or the encoding is fixed.
        let how = self.how;
        self.conv.can_retry =
            !self.fenc.is_empty() && !how.stdin && !self.keep_dest_enc && !how.fifo;
        self.conv.linecnt = self.linecnt;

        if !self.skip_read {
            self.reset_counts();
        }
        Next::Go
    }

    /// Go back to the start of the input and delete the lines read so far.
    /// Whether the input could be rewound.
    fn rewind_for_retry(&mut self) -> bool {
        if self.how.buffer {
            self.read_buf_lnum = LineNr::ONE;
            self.read_buf_col = 0;
        } else if self.how.stdin || !rewind(self.input.as_ref()) {
            return false;
        }
        // Delete the lines read so far.
        while self.lnum > self.from {
            let _ = ml_delete(self.lnum);
            self.lnum -= 1;
        }
        self.file_rewind = false;
        if self.how.set_options {
            Buf::current().b_p_bomb = c_int::from(false);
            Buf::current().b_start_bomb = c_int::from(false);
        }
        self.conv.conv_error = LineNr::ZERO;
        true
    }

    /// The file format to start from: forced by "++ff=", Unix for
    /// 'binary', the buffer's with an empty 'fileformats', otherwise
    /// detected from the file.
    fn reset_fileformat(&mut self, excmd: Option<&ExArg>) {
        self.fileformat = if let Some(command) = excmd
            && command.force_ff.is_some()
        {
            self.guess.try_unix = 0;
            self.guess.try_dos = false;
            self.guess.try_mac = 0;
            Buf::current().fileformat_forced(Some(command))
        } else if Buf::current().b_p_bin != 0 {
            EOL_UNIX // binary: use Unix format
        } else if p_ffs(CStr::is_empty) {
            get_fileformat(Buf::current()) // from the buffer
        } else {
            EOL_UNKNOWN // detect from the file
        };
    }

    /// Try the next entry in 'fileencodings', or no conversion at all when
    /// the one "++enc=" gave wasn't possible.
    fn next_encoding(&mut self, excmd: Option<&ExArg>) {
        self.advance_fenc = false;
        if excmd.is_some_and(|command| command.force_enc.is_some()) {
            // The conversion given with "++enc=" wasn't possible; read
            // without conversion.
            self.notconverted = true;
            self.conv.conv_error = LineNr::ZERO;
            self.fenc = XString::new();
        } else {
            self.fenc = self.fencs.next();
        }
        if let Some(name) = self.tmpname.take() {
            os_remove(name.as_cstr()); // delete the converted file
        }
    }

    /// Work out how "fenc" is converted: here, by iconv, or by
    /// 'charconvert' into a temp file. `Retry` when it can't be.
    fn set_up_conversion(&mut self) -> Next {
        let how = self.how;
        // Conversion is needed when the file's encoding differs from
        // 'encoding', or 'encoding' is UTF-16, UCS-2 or UCS-4.
        self.conv.flags = FioFlags::NONE;
        self.converted = need_conversion(&self.fenc);
        if !self.converted {
            return Next::Go;
        }
        self.conv.flags = if self.fenc.as_cstr() == ENC_UCSBOM {
            // "ucs-bom" means the first bytes of the file decide.
            FioFlags::UCSBOM
        } else {
            // Work out whether UCS-2/4 or Latin1 to UTF-8 can be done here
            // rather than by iconv, which appears not to handle Unicode to
            // Latin1 correctly. Doing it now saves parsing the name on every
            // read.
            get_fio_flags(&self.fenc)
        };

        // Try iconv() if we can't convert it ourselves.
        if self.conv.flags.is_empty() && !self.did_iconv {
            self.conv.open_iconv(self.fenc.as_cstr());
        }

        // Use the 'charconvert' expression when conversion is needed and
        // neither of the above can do it.
        if self.conv.flags.is_empty()
            && !how.stdin
            && !how.buffer
            && P_CCV.first_byte() != 0
            && !how.fifo
            && !self.conv.has_iconv()
        {
            self.did_iconv = false;
            // Skip the conversion when it is already done (a retry for a
            // wrong "fileformat").
            if self.tmpname.is_none() {
                let name = self.fname.unwrap_or_default();
                self.tmpname = readfile_charconvert(name, self.fenc.as_cstr(), &mut self.input);
                if self.tmpname.is_none() {
                    // Conversion failed. Try another one.
                    self.advance_fenc = true;
                    if self.input.is_none() {
                        // Re-opening the original file failed!
                        emsg(gettext(c"E202: Conversion made file unreadable!"));
                        self.error = true;
                        return Next::Done;
                    }
                    return Next::Retry;
                }
            }
        } else if self.conv.flags.is_empty() && !self.conv.has_iconv() {
            // Conversion is wanted but we can't do it; try the next entry in
            // 'fileencodings'.
            self.advance_fenc = true;
            return Next::Retry;
        }
        Next::Go
    }

    /// Start counting from the top of the file again.
    fn reset_counts(&mut self) {
        let how = self.how;
        self.w.linerest = 0;
        self.filesize = 0;
        self.skip_count = self.lines_to_skip;
        self.read_count = self.lines_to_read;
        self.conv.restlen = 0;
        self.read_undo_file = how.newfile
            && !how.keep_undo
            && !Buf::current().name.full().is_none()
            && Buf::current().b_p_udf != 0
            && !how.filtering
            && !how.fifo
            && !how.stdin
            && !how.buffer;
        if self.read_undo_file {
            self.sha_ctx = Sha256::new();
        }
    }

    /// The block loop: read, look for a BOM, convert, split into lines.
    /// `Retry` when a guess turned out wrong part-way.
    fn read_blocks(&mut self) -> Next {
        while !self.error && !GOT_INT.get() {
            match self.fill_block() {
                Next::Go => {}
                next => return next,
            }
            self.skip_read = false;

            if self.check_bom() == Next::Retry {
                return Next::Retry;
            }

            // Include the bytes that were not converted last time.
            let w = &mut self.w;
            w.ptr -= self.conv.restlen;
            w.size += self.conv.restlen.cast_signed();
            self.conv.restlen = 0;
            // A read error or end of file.
            if w.size <= 0 {
                break;
            }

            if self.convert_block() == Next::Retry {
                return Next::Retry;
            }

            // Count the characters, after conversion.
            self.filesize += self.w.size as FileOffset;

            // When reading the first part of a file, guess the EOL type.
            if self.fileformat == EOL_UNKNOWN {
                self.fileformat = self.guess.guess(self.w.window());
                // May set 'fileformat' when editing a new file.
                if self.how.set_options {
                    set_fileformat(self.fileformat, OptionSetFlags::LOCAL);
                }
            }

            match self.split_block() {
                Split::Done => {}
                Split::Stop => self.error = true,
                Split::RetryUnix => {
                    self.file_rewind = true;
                    self.keep_fileformat = true;
                    return Next::Retry;
                }
            }
            self.w.linerest = self.w.ptr - self.w.line_start;
            os_breakcheck();
        }
        Next::Done
    }

    /// Get the next block of bytes into the window: from the file, or from
    /// the buffer for `READ_BUFFER`, or the bytes already there after a
    /// BOM. A line too long for `ColNr` is split instead.
    fn fill_block(&mut self) -> Next {
        let w = &mut self.w;
        if !self.skip_read {
            // Use a buffer of at least 64K. Add linerest to double the size
            // if the line gets very long, to avoid a lot of copying. But
            // don't read more than 1 Mbyte at a time, so that we can be
            // interrupted.
            w.size = (0x10000 + w.linerest.cast_signed()).min(0x100000);
        }

        // Split lines that are too long for ColNr. After this check we read
        // up to "size" more bytes, and even then the line length must stay
        // below MAXCOL - 1 (we add 1 for the NUL when casting to ColNr). If
        // it fires we insert a newline right away, so linerest does not
        // grow.
        if w.size < 0 || w.linerest.cast_signed() >= c_int::MAX as isize - w.size {
            self.split += 1;
            w.bytes[w.ptr] = b'\n'; // split the line by inserting a NL
            w.size = 1;
            return Next::Go;
        }
        if self.skip_read {
            return Next::Go;
        }
        if !w.make_room() {
            self.error = true;
            return Next::Done;
        }
        self.leave_room_to_convert();

        if self.conv.restlen > 0 {
            // Put the unconverted bytes from last time first.
            let w = &mut self.w;
            self.conv.restore(&mut w.bytes[w.ptr..]);
            w.ptr += self.conv.restlen;
            w.size -= self.conv.restlen.cast_signed();
        }

        let w = &mut self.w;
        w.size = if self.how.buffer {
            // Read bytes from curbuf. Used for converting text read from
            // stdin.
            read_from_buffer(
                w,
                self.from,
                &mut self.read_buf_lnum,
                &mut self.read_buf_col,
            )
        } else {
            // Read bytes from the file.
            let window = w.ptr..w.ptr + w.len();
            self.input
                .as_mut()
                .map_or(-1, |file| read_retrying(file, &mut w.bytes[window]))
        };

        if w.size < 0 {
            self.error = true; // read error
        } else if w.size == 0 && self.conv.restlen > 0 {
            return self.end_mid_character();
        }
        Next::Go
    }

    /// We may need room to translate into. For iconv() we don't really know
    /// how much, so use a factor. latin1 to utf-8: 1 byte becomes up to 2
    /// bytes. utf-16 to utf-8: 2 bytes become up to 3, 4 become up to 4, and
    /// the size must be a multiple of 2. ucs-2 to utf-8: 2 bytes become up
    /// to 3, size must be a multiple of 2. ucs-4 to utf-8: 4 bytes become up
    /// to 6, size must be a multiple of 4.
    fn leave_room_to_convert(&mut self) {
        let (w, conv) = (&mut self.w, &self.conv);
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
    }

    /// End of file, but some trailing bytes could not be converted. A
    /// truncated file? Retry with another conversion when one was done and
    /// that is possible; otherwise note the error and replace or drop the
    /// bytes.
    fn end_mid_character(&mut self) -> Next {
        let conv = &mut self.conv;
        let w = &mut self.w;
        if conv.active() {
            // We did a conversion, so report an error.
            if conv.can_retry {
                let had_iconv = conv.has_iconv();
                self.retry_conversion(had_iconv);
                return Next::Retry;
            }
            if conv.conv_error == LineNr::ZERO {
                conv.conv_error = Buf::current().b_ml.ml_line_count - self.linecnt + 1;
            }
        } else if self.illegal_byte == LineNr::ZERO {
            // Remember the first line with an illegal byte.
            self.illegal_byte = Buf::current().b_ml.ml_line_count - self.linecnt + 1;
        }
        if conv.bad_char == BAD_DROP {
            w.bytes[w.ptr - conv.restlen] = 0;
            conv.restlen = 0;
        } else {
            // Replace the trailing bytes with the replacement character if
            // we were converting; if we weren't, leave it to the UTF-8
            // check, which works slightly differently.
            if conv.bad_char != BAD_KEEP && conv.active() {
                while conv.restlen > 0 {
                    w.ptr -= 1;
                    w.bytes[w.ptr] = conv.bad_byte();
                    conv.restlen -= 1;
                }
            }
            conv.flags = FioFlags::NONE; // don't convert this
            conv.close_iconv();
        }
        Next::Go
    }

    /// At the start of the file: check for a BOM, and remove it. Also for
    /// the other Unicode encodings, but not after converting with
    /// 'charconvert' and not when a BOM was already found. `Retry` (without
    /// reading new bytes or rewinding) when "ucs-bom" asked the BOM to pick
    /// the encoding.
    fn check_bom(&mut self) -> Next {
        let w = &mut self.w;
        let conv = &self.conv;
        if self.filesize != 0
            || !(conv.flags == FioFlags::UCSBOM
                || (Buf::current().b_p_bomb == 0
                    && self.tmpname.is_none()
                    && matches!(self.fenc.first(), Some(b'u') | None)))
        {
            return Next::Go;
        }
        // No BOM detection in a short file or in binary mode.
        let found = if w.size < 2 || Buf::current().b_p_bin != 0 {
            None
        } else {
            let allowed = if conv.flags == FioFlags::UCSBOM {
                FioFlags::ALL
            } else {
                get_fio_flags(&self.fenc)
            };
            check_for_bom(w.window(), allowed)
        };
        if let Some((_, blen)) = found {
            // Remove the BOM from the text.
            self.filesize += FileOffset::try_from(blen).expect("a BOM is a few bytes");
            w.size -= blen.cast_signed();
            let rest = w.ptr + blen..w.ptr + blen + w.len();
            w.bytes.copy_within(rest, w.ptr);
            if self.how.set_options {
                Buf::current().b_p_bomb = c_int::from(true);
                Buf::current().b_start_bomb = c_int::from(true);
            }
        }

        if conv.flags != FioFlags::UCSBOM {
            return Next::Go;
        }
        match found {
            None => self.advance_fenc = true, // retry with the next encoding
            // A BOM was found: set "fenc" and start over.
            Some((name, _)) => self.fenc = XString::from_cstr(name),
        }
        self.skip_read = true;
        Next::Retry
    }

    /// Convert the block to UTF-8 (by iconv, or here), or check that it is
    /// valid UTF-8 already. `Retry` with another conversion when that fails.
    fn convert_block(&mut self) -> Next {
        let conv = &mut self.conv;
        if conv.has_iconv() && !conv.with_iconv(&mut self.w) {
            conv.close_iconv();
            let had_iconv = conv.has_iconv();
            self.retry_conversion(had_iconv);
            return Next::Retry;
        }
        let converts = !conv.flags.is_empty();
        let failed = if converts {
            !conv.units_to_utf8(&mut self.w)
        } else {
            Buf::current().b_p_bin == 0
                && !conv.check_utf8(&mut self.w, self.filesize, &mut self.illegal_byte)
        };
        if failed {
            let had_iconv = conv.has_iconv();
            conv.close_iconv();
            self.retry_conversion(had_iconv);
            return Next::Retry;
        }
        Next::Go
    }

    /// Set up for reading the file again with another conversion: by
    /// 'charconvert' when iconv (`had_iconv`) failed, otherwise the next
    /// entry of 'fileencodings'.
    fn retry_conversion(&mut self, had_iconv: bool) {
        rewind_retry(
            &mut self.did_iconv,
            &mut self.advance_fenc,
            &mut self.file_rewind,
            had_iconv,
        );
    }

    /// Hand the block's lines to the buffer.
    fn split_block(&mut self) -> Split {
        let how = self.how;
        let mut sink = LineSplit {
            lnum: self.lnum,
            skip_count: self.skip_count,
            read_count: self.read_count,
            sha: &mut self.sha_ctx,
            read_undo_file: self.read_undo_file,
            newfile: how.newfile,
            fileformat: self.fileformat,
            ff_error: self.ff_error,
            try_unix: self.guess.try_unix != 0,
            stdin: how.stdin,
            from_buffer: how.buffer,
            input: self.input.as_ref(),
            set_options: how.set_options,
        };
        let split_as = split_lines(&mut self.w, &mut sink);
        (self.lnum, self.skip_count, self.read_count) =
            (sink.lnum, sink.skip_count, sink.read_count);
        (self.fileformat, self.ff_error) = (sink.fileformat, sink.ff_error);
        split_as
    }

    /// After the last pass: drop a DOS file's trailing CTRL-Z and complete
    /// a last line that has no end-of-line. Answers that line's number, or
    /// zero when the last line was complete.
    pub(crate) fn finish_last_line(&mut self) -> LineNr {
        // Not an error: the maximum number of lines was reached.
        if self.error && self.read_count == 0 {
            self.error = false;
        }
        let set_options = self.how.set_options;
        let w = &mut self.w;

        // In Dos format ignore a trailing CTRL-Z, unless 'binary' is set.
        // In the old days the file length was a sector count and the CTRL-Z
        // was the marker for where the file really ended. Assuming we write
        // to a file system that keeps the length properly, the CTRL-Z
        // should be dropped; 'endoffile' lets the user decide what to write
        // later. In Unix format the CTRL-Z is just a character.
        if w.linerest != 0
            && Buf::current().b_p_bin == 0
            && self.fileformat == EOL_DOS
            && w.bytes[w.ptr - 1] == 0x1a
        {
            w.ptr -= 1;
            w.linerest -= 1;
            if set_options {
                Buf::current().b_p_eof = c_int::from(true);
            }
        }

        // If we hit end of file in the middle of a line, note that and
        // complete the line ourselves.
        if self.error || GOT_INT.get() || w.linerest == 0 {
            return LineNr::ZERO;
        }
        // Remember it for when writing.
        if set_options {
            Buf::current().b_p_eol = c_int::from(false);
        }
        w.bytes[w.ptr] = 0;
        let line = &w.bytes[w.line_start..=w.ptr];
        if ml_append_terminated(self.lnum, line, self.how.newfile).is_err() {
            self.error = true;
            return LineNr::ZERO;
        }
        if self.read_undo_file {
            self.sha_ctx.update(line);
        }
        self.lnum += 1;
        self.lnum
    }
}

/// The 'fileencoding' to use, or to start with, and the rest of the list to
/// try after it; and whether the encoding is fixed ("++enc=").
fn first_encoding(excmd: Option<&ExArg>) -> (XString, FencList, bool) {
    if let Some(command) = excmd
        && let Some(force_enc) = command.force_enc
    {
        // `force_enc` is an offset into the command's own line.
        let name = command.line.rest_of(command.line.cmd + force_enc);
        (canonize(name), FencList::none(), true)
    } else if Buf::current().b_p_bin != 0 {
        (XString::new(), FencList::none(), false) // binary: don't convert
    } else if Buf::current().b_help {
        // Help files are either utf-8 or latin1. Try utf-8 first; if that
        // fails it must be latin1. This is needed when the first line has
        // non-ASCII characters, which happens only in *.??x files.
        (
            XString::from_cstr(c"utf-8"),
            FencList::new(b"latin1"),
            false,
        )
    } else if p_fencs(CStr::is_empty) {
        // Use the buffer's encoding.
        (Buf::current().b_p_fenc.get(), FencList::none(), false)
    } else {
        // Try the items in 'fileencodings'; a copy, because the walk spans
        // every pass.
        let mut fencs = FencList::new(&P_FENCS.get());
        let fenc = fencs.next();
        (fenc, fencs, false)
    }
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
            Ok(n) => return n.cast_signed(),
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
            *to = if b == b'\n' { 0 } else { b };
        }
        tlen += n;
        if partial {
            *col += n;
            return tlen.cast_signed();
        }
        // Append the whole line and a newline.
        w.bytes[w.ptr + tlen] = b'\n';
        tlen += 1;
        *col = 0;
        *lnum += 1;
        if *lnum > last {
            // When the last line had no end-of-line, don't add one now.
            if Buf::current().b_p_eol == 0 {
                tlen -= 1;
            }
            return tlen.cast_signed();
        }
    }
}
