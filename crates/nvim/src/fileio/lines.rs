//! Turning the bytes just read into buffer lines.
//!
//! This runs once for every character read, so it is written to be fast: the
//! Unix and Dos cases scan for the newline with `memchr`, and only the Mac
//! case, where the line ending is a carriage return that also has to be
//! swapped with any newlines in the text, walks byte by byte.

#![deny(unsafe_op_in_unsafe_fn)]
#![allow(unsafe_code)]

use core::ffi::{c_char, c_int};

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
pub(crate) struct Lines<'a> {
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
    pub fd: c_int,
    pub set_options: bool,
}

/// Append the lines in `w` to the buffer.
pub(crate) fn split_lines(
    w: &mut Window,
    st: &mut Lines,
    lnum: &mut LineNr,
    skip_count: &mut LineNr,
    read_count: &mut LineNr,
    fileformat: &mut c_int,
    ff_error: &mut c_int,
) -> Split {
    let result = split(w, st);
    *lnum = st.lnum;
    *skip_count = st.skip_count;
    *read_count = st.read_count;
    *fileformat = st.fileformat;
    *ff_error = st.ff_error;
    result
}

fn split(w: &mut Window, st: &mut Lines) -> Split {
    // The loops below run once for every character read, so keep them
    // fast.
    let mut appended = |line_start: *mut c_char, len: ColNr| -> bool {
        if unsafe { ml_append(st.lnum, line_start, len, st.newfile) }.is_err() {
            return false;
        }
        if st.read_undo_file {
            let (at, n) = (line_start.cast::<u8>(), len as usize);
            st.sha.update(unsafe { core::slice::from_raw_parts(at, n) });
        }
        st.lnum += 1;
        true
    };

    if st.fileformat == EOL_MAC {
        w.ptr = unsafe { w.ptr.offset(-1) };
        loop {
            w.ptr = unsafe { w.ptr.add(1) };
            w.size -= 1;
            if w.size < 0 {
                break;
            }
            // Catch the most common case first.
            let c = unsafe { *w.ptr };
            if c != 0 && c != CAR as c_char && c != NL as c_char {
                continue;
            }
            if c == 0 {
                unsafe { *w.ptr = NL as c_char }; // NULs are replaced by newlines!
            } else if c == NL as c_char {
                unsafe { *w.ptr = CAR as c_char }; // NLs are replaced by CRs!
            } else {
                if st.skip_count == 0 {
                    unsafe { *w.ptr = 0 }; // end of line
                    let len = (unsafe { w.ptr.offset_from(w.line_start) } + 1) as ColNr;
                    if !appended(w.line_start, len) {
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
                w.line_start = unsafe { w.ptr.add(1) };
            }
        }
    } else {
        let end = unsafe { w.ptr.offset(w.size) };
        while w.ptr < end {
            // memchr is SIMD-optimised, unlike scanning each
            // byte here.
            let nl = unsafe { memchr(w.ptr.cast(), NL, end.offset_from(w.ptr) as size_t) }
                .cast::<c_char>();
            if nl.is_null() {
                // No more newlines. Replace any NUL bytes in
                // what is left with NL.
                loop {
                    let nul = unsafe { memchr(w.ptr.cast(), 0, end.offset_from(w.ptr) as size_t) }
                        .cast::<c_char>();
                    if nul.is_null() {
                        break;
                    }
                    unsafe { *nul = NL as c_char };
                    w.ptr = unsafe { nul.add(1) };
                }
                w.ptr = end;
                break;
            }

            // Replace NUL bytes with NL before the newline.
            let mut scan = w.ptr;
            loop {
                let nul = unsafe { memchr(scan.cast(), 0, nl.offset_from(scan) as size_t) }
                    .cast::<c_char>();
                if nul.is_null() {
                    break;
                }
                unsafe { *nul = NL as c_char };
                scan = unsafe { nul.add(1) };
            }

            // Process the newline.
            w.ptr = nl;
            if st.skip_count == 0 {
                unsafe { *w.ptr = 0 }; // end of line
                let mut len = (unsafe { w.ptr.offset_from(w.line_start) } + 1) as ColNr;
                if st.fileformat == EOL_DOS {
                    if w.ptr > w.line_start && unsafe { *w.ptr.offset(-1) } == CAR as c_char {
                        // Remove the CR before the NL.
                        unsafe { *w.ptr.offset(-1) = 0 };
                        len -= 1;
                    } else if st.ff_error != EOL_DOS {
                        // Reading in Dos format but no CR-LF
                        // found. When 'fileformats' includes
                        // "unix", delete all the lines read so
                        // far and start all over again;
                        // otherwise give an error later.
                        if st.try_unix
                            && !st.stdin
                            && (st.from_buffer || unsafe { lseek(st.fd, 0, SEEK_SET) } == 0)
                        {
                            st.fileformat = EOL_UNIX;
                            if st.set_options {
                                set_fileformat(EOL_UNIX, OptionSetFlags::LOCAL);
                            }
                            return Split::RetryUnix;
                        }
                        st.ff_error = EOL_DOS;
                    }
                }
                if !appended(w.line_start, len) {
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
            w.line_start = unsafe { w.ptr.add(1) };
            w.ptr = unsafe { w.ptr.add(1) };
        }
        w.size = -1;
    }
    Split::Done
}
