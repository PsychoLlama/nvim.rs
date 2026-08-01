//! Recording a match.
//!
//! [`FindTags::add_match`] files a parsed line under one of the sixteen
//! `MT_*` priorities — whether the match was exact, whether it came from
//! this file, whether it is static — in the shape the caller of
//! [`find_tags`](super::find_tags) will read it back in, and drops it if
//! an equal one is already there.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;
use core::ffi::{CStr, c_char, c_int};
use core::ptr;

/// A help match's language and the NUL after it: `{name}@{lang}NUL`.
const ML_EXTRA: usize = 3;

impl FindTags {
    /// Convert the line just read from the tags file's own encoding.
    ///
    /// Converting the pattern the other way does not work, because the
    /// characters would not be recognised.
    pub(crate) fn convert_line(&mut self) {
        // SAFETY: the buffer is NUL-terminated, and what `string_convert`
        // answers is an allocated NUL-terminated string that is freed
        // here.
        unsafe {
            let line = self.lbuf.as_mut_ptr();
            let conv = string_convert(&raw const self.vimconv, line, ptr::null_mut());
            if conv.is_null() {
                return;
            }
            let converted = CStr::from_ptr(conv).to_bytes_with_nul();
            if converted.len() > self.lbuf.len() {
                // Upstream sizes the buffer to exactly the converted line,
                // which leaves its last-but-one byte non-NUL — so the
                // caller reads that as "the line did not fit" and reads it
                // again into a buffer twice as big. Preserved.
                self.lbuf = converted.iter().map(|&b| b as c_char).collect();
            } else {
                for (at, &byte) in converted.iter().enumerate() {
                    self.lbuf[at] = byte as c_char;
                }
            }
            xfree(conv.cast());
        }
    }

    /// File the match the line holds under the priority it deserves.
    pub(crate) fn add_match(
        &mut self,
        tagp: &tagptrs_T,
        margs: &MatchArgs,
        buf_ffname: *mut c_char,
    ) {
        let name_only = self.flags & TAG_NAMES as c_int != 0;
        // SAFETY: `tagp`'s pointers bracket the fields of the line in the
        // buffer, and `tag_fname` is NUL-terminated. The NUL written over
        // `tagname_end` is put back before returning.
        unsafe {
            let is_current = test_for_current(
                tagp.fname,
                tagp.fname_end,
                self.tag_fname.as_ptr().cast_mut(),
                buf_ffname,
            ) != 0;
            let is_static = test_for_static((tagp as *const tagptrs_T).cast_mut());
            let bucket = self.bucket(is_static, is_current, margs);

            let mfp = if self.help_only {
                Some(self.help_match(tagp, margs))
            } else if name_only && self.get_searchpat {
                self.get_searchpat = false;
                self.search_pattern_match(tagp)
            } else if name_only {
                // If wanted, read the line again to get the long form too.
                if State.get() & MODE_INSERT != 0 {
                    self.get_searchpat = p_sft.get() != 0;
                }
                Some(name_match(tagp))
            } else {
                Some(self.whole_match(bucket))
            };

            if let Some(mfp) = mfp {
                self.record(bucket, mfp);
            }
        }
    }

    /// A help match: `{name}@{lang}NUL{heuristic}NUL`.
    ///
    /// The heuristic is what orders the matches later; it is deliberately
    /// past the NUL, so that it takes no part in finding duplicates.
    ///
    /// # Safety
    /// `tagp`'s name must lie in the line buffer, ending at a TAB.
    unsafe fn help_match(&self, tagp: &tagptrs_T, margs: &MatchArgs) -> Match {
        // SAFETY: the caller's promise; the TAB is put back below.
        unsafe {
            *tagp.tagname_end = 0;
            let name = CStr::from_ptr(tagp.tagname).to_bytes();
            let len = name.len();

            // One byte for the '@', ten for the number and its NUL, and
            // `ML_EXTRA` for the language and the NUL after it.
            let mut mfp = vec![0u8; 1 + len + 10 + ML_EXTRA + 1];
            mfp[..len].copy_from_slice(name);
            mfp[len] = b'@';
            mfp[len + 1..len + 3].copy_from_slice(&self.help_lang);

            let score = help_heuristic(
                tagp.tagname,
                if margs.match_re { margs.matchoff } else { 0 },
                !margs.match_no_ic,
            ) + self.help_pri;
            let at = len + 1 + ML_EXTRA;
            let printed = format!("{score:06}");
            // What `snprintf` would have had room for: everything left in
            // the buffer, less its terminator.
            let room = mfp.len() - at - 1;
            let printed = &printed.as_bytes()[..printed.len().min(room)];
            mfp[at..at + printed.len()].copy_from_slice(printed);

            *tagp.tagname_end = TAB as c_char;
            Match(mfp)
        }
    }

    /// `'showfulltag'`: the search pattern of the line, without the `/^`
    /// that opens it and the `$/` that may close it.
    ///
    /// Answers `None` when there is no pattern to take, which is how a
    /// line addressed by number reads.
    ///
    /// # Safety
    /// `tagp.command` must point into the line buffer.
    unsafe fn search_pattern_match(&self, tagp: &tagptrs_T) -> Option<Match> {
        // SAFETY: the caller's promise; the walk stops at the line's NUL.
        unsafe {
            let mut end = tagp.command;
            if *end == b'/' as c_char {
                while !matches!(*end as u8, 0 | b'\r' | b'\n' | b'$') {
                    end = end.add(1);
                }
            }
            if tagp.command.add(2) >= end {
                return None;
            }
            let len = end.offset_from(tagp.command) as usize - 2;
            let mut mfp = vec![0u8; len + 2];
            ptr::copy_nonoverlapping(tagp.command.add(2).cast::<u8>(), mfp.as_mut_ptr(), len);
            Some(Match(mfp))
        }
    }

    /// The whole line, for a caller that wants to jump to the tag:
    /// `<bucket><tags file><sep><line>NUL`.
    ///
    /// The fields are separated by 0x02 rather than NUL because the key
    /// duplicates are found by ends at the first NUL; the caller puts them
    /// back. The bucket is stored one higher so that it is never a NUL.
    ///
    /// # Safety
    /// The line buffer must be NUL-terminated.
    unsafe fn whole_match(&self, bucket: usize) -> Match {
        // SAFETY: the caller's promise.
        let line = unsafe { CStr::from_ptr(self.lbuf.as_ptr()) }.to_bytes();
        let fname = self.tag_fname.bytes();

        let mut mfp = vec![0u8; fname.len() + line.len() + 5];
        mfp[0] = bucket as u8 + 1;
        mfp[1..=fname.len()].copy_from_slice(fname);
        mfp[fname.len() + 1] = TAG_SEP as u8;
        let at = fname.len() + 2;
        mfp[at..at + line.len()].copy_from_slice(line);
        Match(mfp)
    }
}

/// Just the tag's name, for a caller that only wants the names.
///
/// # Safety
/// `tagp`'s name must lie in the line buffer.
unsafe fn name_match(tagp: &tagptrs_T) -> Match {
    // SAFETY: the caller's promise.
    unsafe {
        let len = tagp.tagname_end.offset_from(tagp.tagname) as usize;
        let mut mfp = vec![0u8; len + 2];
        ptr::copy_nonoverlapping(tagp.tagname.cast::<u8>(), mfp.as_mut_ptr(), len);
        Match(mfp)
    }
}
