//! Feeding the buffer's lines into the byte writer, and reporting what came
//! out.
//!
//! [`Buf::write_lines`] is the loop that runs once per character of the file, so
//! it is the only part of a write that scales with the file's size. All it
//! does is translate line endings and hand bytes to the [`ByteWriter`];
//! conversion, buffering and the actual `write()` happen there.
//!
//! [`report_written`] builds the `"name" 12L, 345B written` message out of
//! what the write turned out to be.

#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use crate::memline::Lines;
use crate::memory::XString;
use crate::narrow::len_as_int;
use core::ffi::{CStr, c_int};

use super::*;
use crate::message::IOSIZE;
use crate::option::defs::ShmFlag;

/// What a pass over the lines produced.
#[derive(Default)]
pub(crate) struct Written {
    /// One past the last line written, so `lnum - start` is the count.
    pub lnum: LineNr,
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
    pub conv_error_lnum: LineNr,
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
///
/// The count is upstream's `int`, which a write past 2 GiB overflows; it
/// wraps, as C's did, and only the "written" message reads it.
fn flush_full(writer: &mut ByteWriter, nchars: &mut c_int) -> bool {
    let full = writer.capacity();
    if !writer.flush() {
        return false;
    }
    *nchars = nchars.wrapping_add(len_as_int(full - writer.staged()));
    true
}

impl Buf {
    /// Write lines `start` through `end` of `buffer`.
    ///
    /// `sha` hashes the text as it goes, for the undo file. When `writer.fd` is
    /// -1 nothing reaches a file: that pass only exists to find out whether the
    /// conversion works.
    pub(crate) fn write_lines(
        self,
        range: (LineNr, LineNr),
        writer: &mut ByteWriter,
        fileformat: c_int,
        write_bin: bool,
        mut sha: Option<&mut Sha256>,
    ) -> Written {
        let (start, end) = range;
        // Zero means "the write failed"; it is also how a caller says the
        // write was doomed before it began.
        let mut end = end;
        let mut nchars = 0;
        let mut no_eol = false;
        writer.start_lnum = start;

        let mac = fileformat == EOL_MAC;
        let mut lines = Lines::in_buffer(self);
        let mut lnum = start;
        while lnum <= end {
            let text = lines.line(lnum);
            if let Some(sha) = sha.as_deref_mut() {
                // The terminating NUL goes in as the line separator.
                sha.update(text);
                sha.update(b"\0");
            }
            // Stage the line a run at a time, flushing whenever the staging
            // buffer fills. This is the loop that runs over every byte of the
            // file. Keep it fast!
            let mut rest = text;
            while !rest.is_empty() {
                let taken = writer.push_text(rest, mac);
                rest = &rest[taken..];
                if writer.is_full() {
                    if !flush_full(writer, &mut nchars) {
                        end = LineNr::ZERO; // write error: break the loop
                        break;
                    }
                    writer.start_lnum = lnum;
                }
            }

            // Write failed, or the last line has no end-of-line: stop here.
            if end == LineNr::ZERO
                || (lnum == end
                    && (write_bin || self.b_p_fixeol == 0)
                    && ((write_bin && lnum == self.b_no_eol_lnum)
                        || (lnum == self.b_ml.ml_line_count && self.b_p_eol == 0)))
            {
                lnum += 1; // written the line, count it
                no_eol = true;
                break;
            }

            // EOL_MAC and EOL_DOS write a CR; EOL_DOS follows it with an NL.
            let full = if fileformat == EOL_UNIX {
                writer.push(b'\n')
            } else {
                let full = writer.push(b'\r');
                if fileformat != EOL_DOS {
                    full
                } else {
                    if full && !flush_full(writer, &mut nchars) {
                        end = LineNr::ZERO; // write error: break the loop
                        break;
                    }
                    writer.push(b'\n')
                }
            };
            if full {
                if !flush_full(writer, &mut nchars) {
                    end = LineNr::ZERO; // write error: break the loop
                    break;
                }
                os_breakcheck();
                if got_int.get() {
                    end = LineNr::ZERO; // interrupted: break the loop
                    break;
                }
            }
            lnum += 1;
        }

        if writer.staged() > 0 && end > LineNr::ZERO {
            let remaining = writer.staged();
            if !writer.flush() {
                end = LineNr::ZERO; // write error
            }
            nchars = nchars.wrapping_add(len_as_int(remaining - writer.staged()));
        }
        // Did everything convert and get written?
        if end != LineNr::ZERO && writer.staged() > 0 {
            writer.conv_error = true;
            writer.conv_error_lnum = end;
            end = LineNr::ZERO;
        }
        if self.b_p_fixeol == 0 && self.b_p_eof != 0 {
            // Write the trailing CTRL-Z that 'endoffile' asks for.
            writer.write_raw(b"\x1a");
        }

        Written {
            lnum,
            nchars,
            no_eol,
            failed: end == LineNr::ZERO,
        }
    }
}

/// Build and show the message a successful write ends with.
pub(crate) fn report_written(
    buffer: Buf,
    fname: &CStr,
    written: &Written,
    notes: &WriteNotes,
    append: bool,
) {
    // The report. Upstream assembles it in `IObuff`, which `msg_progress`
    // and `set_keep_msg` write again.
    let (lnum, nchars) = (written.lnum, FileOffset::from(written.nchars));
    let mut report = Report::quoted_fname(buffer, Some(fname), IOSIZE as usize);
    let note = |report: &mut Report, text: &'static CStr| report.push(translate(text).to_bytes());

    let mut insert_space = notes.conv_error || notes.notconverted || notes.converted;
    if notes.conv_error {
        note(&mut report, c" CONVERSION ERROR");
        if notes.conv_error_lnum != LineNr::ZERO {
            let fmt = translate(c" in line %ld;");
            report.push_fmt(fmt, &[i64::from(notes.conv_error_lnum).into()]);
        }
    } else if notes.notconverted {
        note(&mut report, c"[NOT converted]");
    } else if notes.converted {
        note(&mut report, c"[converted]");
    }

    if notes.device {
        note(&mut report, c"[Device]");
        insert_space = true;
    } else if notes.newfile {
        note(&mut report, c"[New]");
        insert_space = true;
    }
    if written.no_eol {
        note(&mut report, c"[noeol]");
        insert_space = true;
    }
    // May add [unix/dos/mac].
    if msg_add_fileformat(&mut report, notes.fileformat) {
        insert_space = true;
    }
    msg_add_lines(&mut report, insert_space, lnum, nchars);

    if !shortmess(ShmFlag::WRITE) {
        let short = shortmess(ShmFlag::WRI);
        note(
            &mut report,
            match (append, short) {
                (true, true) => c" [a]",
                (true, false) => c" appended",
                (false, true) => c" [w]",
                (false, false) => c" written",
            },
        );
    }
    let text = report.as_cstr();
    let shown = XString::from_bytes(&msg_progress(text, c"bufwrite", c"success", 0, true, true));
    set_keep_msg(Some(shown.as_cstr()), 0);
}
