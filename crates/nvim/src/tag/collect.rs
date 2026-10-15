//! Recording a match.
//!
//! [`FindTags::add_match`] files a parsed line under one of the sixteen
//! `MT_*` priorities — whether the match was exact, whether it came from
//! this file, whether it is static — in the shape the caller of
//! [`find_tags`](super::find_tags) will read it back in, and drops it if
//! an equal one is already there.

#![forbid(unsafe_code)]

use super::*;
use core::ffi::{CStr, c_int};

/// The room upstream gave a help match's score: ten bytes, less the
/// terminator.
const SCORE_ROOM: usize = 10;

impl FindTags {
    /// Convert the line just read from the tags file's own encoding.
    ///
    /// Converting the pattern the other way does not work, because the
    /// characters would not be recognised.
    pub(crate) fn convert_line(&mut self) {
        let Some(mut converted) = self.vimconv.convert(self.line()) else {
            return;
        };
        // What a C reader of the converted string sees, terminated.
        converted.truncate(crate::cstr::until_nul(&converted).len());
        self.line_len = converted.len();
        converted.push(0);
        if converted.len() > self.lbuf.len() {
            // Upstream sizes the buffer to exactly the converted line,
            // which leaves its last-but-one byte non-NUL — so the
            // caller reads that as "the line did not fit" and reads it
            // again into a buffer twice as big. Preserved.
            self.lbuf = converted;
        } else {
            self.lbuf[..converted.len()].copy_from_slice(&converted);
        }
    }

    /// File the match the line holds under the priority it deserves.
    #[inline]
    pub(crate) fn add_match(
        &mut self,
        tagp: &TagParts,
        margs: &MatchArgs,
        buf_ffname: Option<&CStr>,
    ) {
        let name_only = self.flags & TAG_NAMES as c_int != 0;
        let line = self.line();
        let is_current = test_for_current(tagp.fname.of(line), self.tag_fname.bytes(), buf_ffname);
        let is_static = test_for_static(line, tagp);
        let bucket = self.bucket(is_static, is_current, margs);

        let found = if self.help_only {
            Some(self.help_match(tagp, margs, bucket))
        } else if name_only && self.get_searchpat {
            let key = search_pattern_match(line, tagp);
            self.get_searchpat = false;
            key.map(|key| Found::new(key, bucket))
        } else if name_only {
            let key = tagp.tagname.of(line).to_vec();
            // If wanted, read the line again to get the long form too.
            if State.get() & MODE_INSERT != 0 {
                self.get_searchpat = p_sft();
            }
            Some(Found::new(key, bucket))
        } else {
            Some(self.whole_match(bucket))
        };

        if let Some(found) = found {
            self.record(bucket, found);
        }
    }

    /// A help match: `{name}@{lang}`, and the heuristic score that orders
    /// the matches later — which takes no part in finding duplicates.
    fn help_match(&self, tagp: &TagParts, margs: &MatchArgs, bucket: usize) -> Found {
        let name = tagp.tagname.of(self.line());
        let mut key = Vec::with_capacity(name.len() + 3);
        key.extend_from_slice(name);
        key.push(b'@');
        key.extend_from_slice(&self.help_lang);

        let offset = if margs.match_re { margs.matchoff } else { 0 };
        let score = help_heuristic(name, offset, !margs.match_no_ic) + self.help_pri;
        let mut score = format!("{score:06}").into_bytes();
        // What `snprintf` had room for.
        score.truncate(SCORE_ROOM);

        let mut found = Found::new(key, bucket);
        found.score = score;
        found
    }

    /// The whole line, for a caller that wants to jump to the tag:
    /// `<tags file>NUL<line>`.
    fn whole_match(&self, bucket: usize) -> Found {
        let line = self.line();
        let fname = self.tag_fname.bytes();
        let mut key = Vec::with_capacity(fname.len() + 1 + line.len() + 1);
        key.extend_from_slice(fname);
        key.push(0);
        key.extend_from_slice(line);
        Found::new(key, bucket)
    }
}

/// `'showfulltag'`: the search pattern of the line, without the `/^`
/// that opens it and the `$/` that may close it.
///
/// Answers `None` when there is no pattern to take, which is how a line
/// addressed by number reads.
fn search_pattern_match(line: &[u8], tagp: &TagParts) -> Option<Vec<u8>> {
    let command = &line[tagp.command..];
    let mut end = 0;
    if command.first() == Some(&b'/') {
        end = command
            .iter()
            .position(|b| matches!(b, b'\r' | b'\n' | b'$'))
            .unwrap_or(command.len());
    }
    (end > 2).then(|| command[2..end].to_vec())
}
