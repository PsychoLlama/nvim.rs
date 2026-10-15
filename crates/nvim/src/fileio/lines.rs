//! Turning the bytes just read into buffer lines.
//!
//! This runs once for every character read, so it is written to be fast: the
//! Unix and Dos cases find the next newline (and any NUL before it) a word at
//! a time, and only the Mac case, where the line ending is a carriage return
//! that also has to be swapped with any newlines in the text, walks byte by
//! byte.

#![forbid(unsafe_code)]

use std::fs::File;
use std::io::{Seek, SeekFrom};

use crate::memline::ml_append_terminated;
use crate::memory::memchr_in;

use super::*;
use crate::types::OptionSetFlags;

/// What a pass of line splitting decided.
pub(crate) enum Split {
    /// The bytes became lines.
    Done,
    /// `ml_append` failed, or the caller's line limit was reached. Either way
    /// the read stops.
    Stop,
    /// Reading in Dos format but no CR-LF was found, and `'fileformats'`
    /// allows Unix: start the whole file again in Unix format.
    RetryUnix,
}

/// What line splitting needs to know, and what it changes.
pub(crate) struct LineSplit<'a> {
    /// The line to append after.
    pub lnum: LineNr,
    /// How many more lines to throw away before keeping any (`:recover`).
    pub skip_count: LineNr,
    /// How many more lines to keep.
    pub read_count: LineNr,
    /// The hash of the text, for the undo file.
    pub sha: &'a mut Sha256,
    pub read_undo_file: bool,
    pub newfile: bool,
    pub fileformat: c_int,
    /// Set to `EOL_DOS` once a missing CR has been complained about.
    pub ff_error: c_int,
    /// Does `'fileformats'` allow Unix line endings?
    pub try_unix: bool,
    pub stdin: bool,
    pub from_buffer: bool,
    /// The file being read, which a retry in Unix format rewinds.
    pub input: Option<&'a File>,
    pub set_options: bool,
}

impl LineSplit<'_> {
    /// Append one line, its terminating NUL included in `line`.
    fn append(&mut self, line: &[u8]) -> bool {
        if ml_append_terminated(self.lnum, line, self.newfile).is_err() {
            return false;
        }
        if self.read_undo_file {
            self.sha.update(line);
        }
        self.lnum += 1;
        true
    }

    /// Can the file be read again from the start?
    fn rewinds(&self) -> bool {
        self.from_buffer
            || self
                .input
                .is_some_and(|mut file| file.seek(SeekFrom::Start(0)).is_ok_and(|at| at == 0))
    }
}

/// Where the newline that ends the line at `from` is, before `end`. Every NUL
/// on the way becomes a newline -- which is how the buffer keeps a NUL --
/// and so does every one up to `end` when there is no newline at all.
///
/// Two `memchr`s, as upstream's: the newline first, then the NULs before it,
/// which there almost never are.
fn next_newline(bytes: &mut [u8], from: usize, end: usize) -> Option<usize> {
    let nl = memchr_in(&bytes[from..end], NL as u8).map(|at| from + at);
    let mut at = from;
    let stop = nl.unwrap_or(end);
    while let Some(nul) = memchr_in(&bytes[at..stop], 0) {
        bytes[at + nul] = NL as u8;
        at += nul + 1;
    }
    nl
}

/// Append the lines in `w` to the buffer.
pub(crate) fn split_lines(w: &mut ReadBuf, st: &mut LineSplit) -> Split {
    if st.fileformat == EOL_MAC {
        split_mac(w, st)
    } else {
        split_nl(w, st)
    }
}

/// Mac format: a carriage return ends the line, and a newline in the text is
/// kept as a carriage return.
fn split_mac(w: &mut ReadBuf, st: &mut LineSplit) -> Split {
    let end = w.ptr + w.len();
    while w.ptr < end {
        let c = w.bytes[w.ptr];
        // Catch the most common case first.
        if c != 0 && c != CAR as u8 && c != NL as u8 {
            w.ptr += 1;
            continue;
        }
        if c == 0 {
            w.bytes[w.ptr] = NL as u8; // NULs are replaced by newlines!
        } else if c == NL as u8 {
            w.bytes[w.ptr] = CAR as u8; // NLs are replaced by CRs!
        } else {
            if st.skip_count == 0 {
                w.bytes[w.ptr] = 0; // end of line
                if !st.append(&w.bytes[w.line_start..=w.ptr]) {
                    return Split::Stop;
                }
                st.read_count -= 1;
                if st.read_count == 0 {
                    w.line_start = w.ptr; // nothing left to write
                    return Split::Stop;
                }
            } else {
                st.skip_count -= 1;
            }
            w.line_start = w.ptr + 1;
        }
        w.ptr += 1;
    }
    w.size = -1;
    Split::Done
}

/// Unix and Dos format: a newline ends the line, after a carriage return in
/// Dos format.
fn split_nl(w: &mut ReadBuf, st: &mut LineSplit) -> Split {
    let end = w.ptr + w.len();
    // Locals for the walk, written back on the way out: this runs for every
    // line of the file.
    let (mut ptr, mut line_start) = (w.ptr, w.line_start);
    let bytes = &mut w.bytes[..end];
    let dos = st.fileformat == EOL_DOS;
    let outcome = loop {
        if ptr >= end {
            break Split::Done;
        }
        let Some(nl) = next_newline(bytes, ptr, end) else {
            ptr = end;
            break Split::Done;
        };

        // Process the newline.
        ptr = nl;
        if st.skip_count == 0 {
            bytes[nl] = 0; // end of line
            let mut line_end = nl;
            if dos {
                if nl > line_start && bytes[nl - 1] == CAR as u8 {
                    // Remove the CR before the NL.
                    bytes[nl - 1] = 0;
                    line_end -= 1;
                } else if st.ff_error != EOL_DOS {
                    // Reading in Dos format but no CR-LF found. When
                    // 'fileformats' includes "unix", delete all the lines
                    // read so far and start all over again; otherwise give
                    // an error later.
                    if st.try_unix && !st.stdin && st.rewinds() {
                        st.fileformat = EOL_UNIX;
                        if st.set_options {
                            set_fileformat(EOL_UNIX, OptionSetFlags::LOCAL);
                        }
                        break Split::RetryUnix;
                    }
                    st.ff_error = EOL_DOS;
                }
            }
            if !st.append(&bytes[line_start..=line_end]) {
                break Split::Stop;
            }
            st.read_count -= 1;
            if st.read_count == 0 {
                line_start = nl; // nothing left to write
                break Split::Stop;
            }
        } else {
            st.skip_count -= 1;
        }
        line_start = nl + 1;
        ptr = nl + 1;
    };
    (w.ptr, w.line_start) = (ptr, line_start);
    if matches!(outcome, Split::Done) {
        w.size = -1;
    }
    outcome
}
