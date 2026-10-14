//! The search context's stack and its "already been here" lists.
//!
//! The walk keeps a stack of directories still to look at and, beside it, a
//! list of the files and directories it has already reported, so that links
//! and self-referencing directories cannot make it loop.
//! [`VisitedList::add`] is the test: it compares by file id rather than by
//! name, and treats two entries as the same only when their wildcard tails
//! agree as well ([`wildcards_equal`], which ignores the counter byte behind a
//! `**`).

#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use super::*;
use crate::mbyte::{c_cluster_len, char_at};
use crate::memory::XString;
use crate::os::fs::file_id;
use crate::path::ExpandFlags;
use core::ffi::c_int;

/// The names one directory's wildcard expansion produced.
pub(crate) struct FileList {
    names: Vec<XString>,
}

impl FileList {
    /// Expand `patterns` into the names they match. Upstream ignores a
    /// failure, which leaves the list empty.
    pub(crate) fn expand(patterns: &[&CStr], flags: ExpandFlags) -> Self {
        FileList {
            names: expand_wildcards(patterns, flags).unwrap_or_default(),
        }
    }

    /// The one-entry list a URL expands to — there is no file system to ask.
    pub(crate) fn of_one(name: &[u8]) -> Self {
        FileList {
            names: vec![XString::from_bytes(name)],
        }
    }

    pub(crate) fn len(&self) -> usize {
        self.names.len()
    }

    /// Name `at`, which must be less than [`len`](Self::len).
    pub(crate) fn get(&self, at: usize) -> &XString {
        &self.names[at]
    }
}

/// One directory the walk still has to look at.
///
/// A frame is pushed back onto the stack when a match is answered from it,
/// so that the next call carries on with the same directory's remaining
/// entries — that is what `files_cur` and `stage` remember.
pub(crate) struct StackFrame {
    /// The part of the search path with no wildcards in it.
    pub(crate) fix_path: Name,
    /// The part that still has wildcards. A leading `**` carries a counter
    /// byte, which the walk decrements as it descends.
    pub(crate) wc_path: Name,
    /// The names in `fix_path` that the first wildcard of `wc_path` matched.
    /// `None` until this directory has been expanded.
    pub(crate) files: Option<FileList>,
    /// How far through `files` this frame has been worked.
    pub(crate) files_cur: usize,
    /// 0: this directory is being worked on for the first time.
    /// 1: it was partly searched in an earlier step.
    pub(crate) stage: u8,
    /// How deep in the directory tree we are, counting down from the level
    /// given to [`vim_findfile_init`].
    pub(crate) level: c_int,
    /// Whether `**` has already been expanded to an empty string here.
    pub(crate) star_star_empty: bool,
}

impl StackFrame {
    pub(crate) fn new(fix_path: Name, wc_path: Name, level: c_int, star_star_empty: bool) -> Self {
        StackFrame {
            fix_path,
            wc_path,
            files: None,
            files_cur: 0,
            stage: 0,
            level,
            star_star_empty,
        }
    }

    /// The names this frame expanded to.
    ///
    /// # Panics
    /// The frame must have been expanded, which every caller has just made
    /// sure of.
    pub(crate) fn files(&self) -> &FileList {
        self.files.as_ref().expect("the frame was expanded above")
    }
}

/// One file or directory the search has already reported.
struct Visited {
    /// The wildcard tail it was reached through.
    wc_path: Name,
    /// Its identity, or `None` for a URL, which is compared by name.
    file_id: Option<FileID>,
    /// The URL; empty when `file_id` says who this is.
    fname: Name,
}

/// Everywhere one search for one file name has already been.
#[derive(Default)]
pub(crate) struct VisitedList {
    /// The file name this list is about.
    filename: Name,
    entries: Vec<Visited>,
}

impl VisitedList {
    /// Record that the search reached `fname` through `wc_path`, answering
    /// whether that is news.
    ///
    /// A file it cannot identify counts as already seen, which is how
    /// upstream keeps an unreadable name out of the results.
    pub(crate) fn add(&mut self, fname: &[u8], wc_path: &[u8]) -> bool {
        // An owned copy first: the file system wants a terminated name.
        let fname = Name::from_bytes(fname);
        // For a URL we only compare the name, otherwise the
        // device/inode.
        let url = path_with_url(fname.as_cstr()) != 0;
        let id = if url {
            None
        } else {
            let Some(id) = file_id(fname.as_cstr()) else {
                return false;
            };
            Some(id)
        };

        let known = self.entries.iter().any(|seen| {
            let same = match (&seen.file_id, &id) {
                (Some(seen_id), Some(id)) => seen_id == id,
                (None, None) => path_fnamecmp(seen.fname.bytes(), fname.bytes()) == 0,
                _ => false,
            };
            same && wildcards_equal(seen.wc_path.bytes(), wc_path)
        });
        if known {
            return false;
        }

        // New file/dir. Add it to the list of visited files/dirs.
        self.entries.push(Visited {
            wc_path: Name::from_bytes(wc_path),
            file_id: id,
            fname: if url { fname } else { Name::default() },
        });
        true
    }
}

/// The visited lists one search context holds, and which of them is in use.
///
/// Several are needed for a `'tags'` setting like
/// `"./**/tags,./**/TAGS,**/tags"`: the first and third searches are for the
/// same file, so the third can reuse the first's list, but the second has to
/// start from an empty one.
#[derive(Default)]
pub(crate) struct VisitedLists {
    lists: Vec<VisitedList>,
    at: usize,
}

impl VisitedLists {
    /// Use the list for `filename`, creating it if this is the first search
    /// for that name.
    pub(crate) fn select(&mut self, filename: &[u8]) {
        let found = self
            .lists
            .iter()
            .position(|list| path_fnamecmp(filename, list.filename.bytes()) == 0);
        self.at = found.unwrap_or_else(|| {
            self.lists.push(VisitedList {
                filename: Name::from_bytes(filename),
                entries: Vec::new(),
            });
            self.lists.len() - 1
        });
    }

    /// Free the list of lists of visited files and directories.
    pub(crate) fn clear(&mut self) {
        self.lists.clear();
        // Upstream leaves the context's "current list" pointer dangling
        // here and dereferences it on the next search. No caller asks for
        // this — both pass `free_visited` false — and an index cannot
        // dangle.
        self.at = 0;
    }

    /// The list the search is using.
    pub(crate) fn current(&mut self) -> &mut VisitedList {
        if self.lists.is_empty() {
            self.lists.push(VisitedList::default());
        }
        &mut self.lists[self.at]
    }
}

/// Are two wildcard paths equal?
///
/// They are equal if they compare character by character to the same length,
/// except that the counters behind a `**` may differ — `**\20` is equal to
/// `**\24`. The two characters remembered are `s1`'s, which is how one
/// mismatch is let through once `s1` has shown `**`.
///
/// Upstream's `ff_wc_equal` walks two C strings; a NUL inside either slice
/// ends it the same way.
pub(crate) fn wildcards_equal(s1: &[u8], s2: &[u8]) -> bool {
    let ignorecase = p_fic();
    let fold = |c: c_int| if ignorecase { mb_tolower(c) } else { c };

    let (mut i, mut j) = (0usize, 0usize);
    let mut prev1 = 0;
    let mut prev2 = 0;
    while cstr::byte_at(s1, i) != 0 && cstr::byte_at(s2, j) != 0 {
        let c1 = char_at(&s1[i..]);
        let c2 = char_at(&s2[j..]);
        if fold(c1) != fold(c2) && (prev1 != c_int::from(b'*') || prev2 != c_int::from(b'*')) {
            return false;
        }
        prev2 = prev1;
        prev1 = c1;
        i += c_cluster_len(&s1[i..]);
        j += c_cluster_len(&s2[j..]);
    }
    cstr::byte_at(s1, i) == cstr::byte_at(s2, j)
}

/// Is `path`, for its first `path_len` bytes, at or below one of the stop
/// directories?
///
/// A parent matches: `/home` stops a search whose start directory is
/// `/home/rks`. The separator test is what keeps `/home/r` from matching
/// `/home/rks`.
pub(crate) fn ff_path_in_stoplist(path: &Name, path_len: usize, stopdirs: &[Name]) -> bool {
    // Eat up trailing path separators, except the first.
    let mut path_len = path_len;
    while path_len > 1 && vim_ispathsep(c_int::from(path.at(path_len - 1))) {
        path_len -= 1;
    }
    // If no path consider it as match.
    if path_len == 0 {
        return true;
    }

    stopdirs.iter().any(|stop| {
        path_fnamencmp(stop.bytes(), path.bytes(), path_len) == 0
            && (stop.len() <= path_len || vim_ispathsep(c_int::from(stop.at(path_len))))
    })
}
