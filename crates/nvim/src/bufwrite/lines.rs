//! Feeding the buffer's lines into the byte writer, and reporting what came
//! out.
//!
//! [`write_lines`] is the loop that runs once per character of the file, so
//! it is the only part of a write that scales with the file's size. All it
//! does is translate line endings and hand bytes to the [`ByteWriter`];
//! conversion, buffering and the actual `write()` happen there.
//!
//! [`report_written`] builds the `"name" 12L, 345B written` message out of
//! what the write turned out to be.

#![deny(unsafe_op_in_unsafe_fn)]

use core::ffi::{CStr, c_char, c_int};

use super::*;
use crate::types::{IOSIZE, NUL, ShmFlag};

/// What a pass over the lines produced.
#[derive(Default)]
pub(crate) struct Written {
    /// One past the last line written, so `lnum - start` is the count.
    pub lnum: linenr_T,
    /// File bytes written.
    pub nchars: c_int,
    /// The last line went out without a trailing end-of-line.
    pub no_eol: bool,
    /// The write failed part way; the file is not to be trusted.
    pub failed: bool,
}

/// The notes that go between the file name and the counts in the message.
pub(crate) struct WriteNotes {
    /// Some character could not be represented in `'fileencoding'`.
    pub conv_error: bool,
    /// The line the first such character was on, or zero.
    pub conv_error_lnum: linenr_T,
    /// Conversion was wanted but `!` said to write the bytes as they are.
    pub notconverted: bool,
    /// Conversion happened.
    pub converted: bool,
    /// The target is a device.
    pub device: bool,
    /// The target did not exist before.
    pub newfile: bool,
    /// `EOL_UNIX`/`EOL_DOS`/`EOL_MAC`.
    pub fileformat: c_int,
}

/// Flush a full staging buffer, adding what went out to `nchars`.
unsafe fn flush_full(writer: &mut ByteWriter, nchars: &mut c_int) -> bool {
    unsafe {
        let full = writer.capacity() as c_int;
        if !writer.flush() {
            return false;
        }
        *nchars += full - writer.staged() as c_int;
        true
    }
}

/// Write lines `start` through `end` of `buf`.
///
/// `sha` hashes the text as it goes, for the undo file. When `writer.fd` is
/// -1 nothing reaches a file: that pass only exists to find out whether the
/// conversion works.
pub(crate) unsafe fn write_lines(
    buf: *mut buf_T,
    range: (linenr_T, linenr_T),
    writer: &mut ByteWriter,
    fileformat: c_int,
    write_bin: bool,
    mut sha: Option<&mut Sha256>,
) -> Written {
    unsafe {
        let (start, end) = range;
        // Zero means "the write failed"; it is also how a caller says the
        // write was doomed before it began.
        let mut end = end;
        let mut nchars = 0;
        let mut no_eol = false;
        writer.start_lnum = start;

        let mut lnum = start;
        while lnum <= end {
            let mut ptr = ml_get_buf(buf, lnum);
            if let Some(sha) = sha.as_deref_mut() {
                // The terminating NUL goes in as the line separator.
                sha.update(core::slice::from_raw_parts(
                    ptr.cast::<u8>(),
                    strlen(ptr) + 1,
                ));
            }
            // The next loop runs once for each character written. Keep it
            // fast!
            loop {
                let c = *ptr;
                if c == NUL as c_char {
                    break;
                }
                ptr = ptr.add(1);
                let byte = if c == NL as c_char {
                    NUL as c_char // replace newlines with NULs
                } else if c == CAR as c_char && fileformat == EOL_MAC {
                    NL as c_char // Mac: replace CRs with NLs
                } else {
                    c
                };
                if writer.push(byte) {
                    if !flush_full(writer, &mut nchars) {
                        end = 0; // write error: break the loop
                        break;
                    }
                    writer.start_lnum = lnum;
                }
            }

            // Write failed, or the last line has no end-of-line: stop here.
            if end == 0
                || (lnum == end
                    && (write_bin || (*buf).b_p_fixeol == 0)
                    && ((write_bin && lnum == (*buf).b_no_eol_lnum)
                        || (lnum == (*buf).b_ml.ml_line_count && (*buf).b_p_eol == 0)))
            {
                lnum += 1; // written the line, count it
                no_eol = true;
                break;
            }

            // EOL_MAC and EOL_DOS write a CR; EOL_DOS follows it with an NL.
            let full = if fileformat == EOL_UNIX {
                writer.push(NL as c_char)
            } else {
                let full = writer.push(CAR as c_char);
                if fileformat != EOL_DOS {
                    full
                } else {
                    if full && !flush_full(writer, &mut nchars) {
                        end = 0; // write error: break the loop
                        break;
                    }
                    writer.push(NL as c_char)
                }
            };
            if full {
                if !flush_full(writer, &mut nchars) {
                    end = 0; // write error: break the loop
                    break;
                }
                os_breakcheck();
                if got_int.get() {
                    end = 0; // interrupted: break the loop
                    break;
                }
            }
            lnum += 1;
        }

        if writer.staged() > 0 && end > 0 {
            let remaining = writer.staged() as c_int;
            if !writer.flush() {
                end = 0; // write error
            }
            nchars += remaining - writer.staged() as c_int;
        }
        // Did everything convert and get written?
        if end != 0 && writer.staged() > 0 {
            writer.conv_error = true;
            writer.conv_error_lnum = end;
            end = 0;
        }
        if (*buf).b_p_fixeol == 0 && (*buf).b_p_eof != 0 {
            // Write the trailing CTRL-Z that 'endoffile' asks for.
            write_eintr(writer.fd, c"\x1a".as_ptr().cast_mut().cast(), 1);
        }

        Written {
            lnum,
            nchars,
            no_eol,
            failed: end == 0,
        }
    }
}

/// Build and show the message a successful write ends with.
pub(crate) unsafe fn report_written(
    buf: *mut buf_T,
    fname: *mut c_char,
    written: &Written,
    notes: &WriteNotes,
    append: bool,
) {
    // The report. Upstream assembles it in `IObuff`, which `msg_progress`
    // and `set_keep_msg` write again.
    let mut report = [0 as c_char; IOSIZE as usize];
    let (lnum, nchars) = (written.lnum, written.nchars as off_T);
    unsafe {
        let iobuff = report.as_mut_ptr();
        add_quoted_fname(iobuff, IOSIZE as size_t, buf, fname);
        let note = |text: &'static CStr| {
            xstrlcat(iobuff, translate(text).as_ptr(), IOSIZE as size_t);
        };

        let mut insert_space = notes.conv_error || notes.notconverted || notes.converted;
        if notes.conv_error {
            note(c" CONVERSION ERROR");
            if notes.conv_error_lnum != 0 {
                vim_snprintf_add(
                    iobuff,
                    IOSIZE as size_t,
                    translate(c" in line %ld;").as_ptr(),
                    notes.conv_error_lnum as int64_t,
                );
            }
        } else if notes.notconverted {
            note(c"[NOT converted]");
        } else if notes.converted {
            note(c"[converted]");
        }

        if notes.device {
            note(c"[Device]");
            insert_space = true;
        } else if notes.newfile {
            note(c"[New]");
            insert_space = true;
        }
        if written.no_eol {
            note(c"[noeol]");
            insert_space = true;
        }
        // May add [unix/dos/mac].
        if msg_add_fileformat(&mut report, notes.fileformat) {
            insert_space = true;
        }
        msg_add_lines(&mut report, insert_space as c_int, lnum, nchars);

        if !shortmess(ShmFlag::WRITE) {
            let short = shortmess(ShmFlag::WRI);
            note(match (append, short) {
                (true, true) => c" [a]",
                (true, false) => c" appended",
                (false, true) => c" [w]",
                (false, false) => c" written",
            });
        }
        set_keep_msg(
            msg_progress(
                iobuff,
                c"bufwrite".as_ptr().cast_mut(),
                c"success".as_ptr().cast_mut(),
                0,
                true,
                true,
            ),
            0,
        );
    }
}
