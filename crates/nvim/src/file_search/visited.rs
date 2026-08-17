//! The search context's stack and its "already been here" lists.
//!
//! The walk keeps a stack of directories still to look at and, beside it, a
//! list of the files and directories it has already reported, so that links
//! and self-referencing directories cannot make it loop.
//! [`VisitedList::add`] is the test: it compares by file id rather than by
//! name, and treats two entries as the same only when their wildcard tails
//! agree as well ([`ff_wc_equal`], which ignores the counter byte behind a
//! `**`).

#![deny(unsafe_op_in_unsafe_fn)]

use super::*;
use crate::memory::xmalloc;
use crate::path::ExpandFlags;
use core::ffi::{c_char, c_int};
use core::{ptr, slice};

/// The names one directory's wildcard expansion produced.
///
/// Owns the array `expand_wildcards` handed back; `FreeWild` is the only way
/// to give it back, so the list frees itself when its stack frame is
/// dropped.
pub(crate) struct FileList {
    names: *mut *mut c_char,
    len: c_int,
}

impl FileList {
    /// Expand `num_pat` patterns into the names they match.
    ///
    /// # Safety
    /// `pat` must hold `num_pat` NUL-terminated strings.
    pub(crate) unsafe fn expand(num_pat: c_int, pat: *mut *mut c_char, flags: ExpandFlags) -> Self {
        let mut list = FileList {
            names: ptr::null_mut(),
            len: 0,
        };
        // Upstream ignores the answer: on failure the two out-parameters are
        // left as they are, which is the empty list.
        // SAFETY: the caller's promise.
        unsafe { expand_wildcards(num_pat, pat, &raw mut list.len, &raw mut list.names, flags) };
        list
    }

    /// The one-entry list a URL expands to — there is no file system to ask.
    ///
    /// # Safety
    /// `name` must hold `namelen` readable bytes.
    pub(crate) unsafe fn of_one(name: *const c_char, namelen: usize) -> Self {
        // SAFETY: the caller's promise. `FreeWild` frees the entry and then
        // the array, so both are taken from the allocator it gives them to.
        unsafe {
            let names = xmalloc(size_of::<*mut c_char>()).cast::<*mut c_char>();
            *names = xmemdupz(name.cast(), namelen).cast();
            FileList { names, len: 1 }
        }
    }

    pub(crate) fn len(&self) -> usize {
        self.len.max(0) as usize
    }

    /// # Safety
    /// `at` must be less than [`len`](Self::len).
    pub(crate) unsafe fn get(&self, at: usize) -> *mut c_char {
        // SAFETY: the caller's promise.
        unsafe { *self.names.add(at) }
    }
}

impl Drop for FileList {
    fn drop(&mut self) {
        // SAFETY: the array and its entries are ours, and nothing else holds
        // them.
        unsafe { FreeWild(self.len, self.names) };
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
    ///
    /// # Safety
    /// There is nothing to promise; both names carry their own length.
    pub(crate) unsafe fn add(&mut self, fname: &[u8], wc_path: &[u8]) -> bool {
        unsafe {
            // Owned copies first: the comparisons below walk to a
            // terminator, which a borrowed slice does not have.
            let fname = Name::from_bytes(fname);
            let wc_path = Name::from_bytes(wc_path);
            // For a URL we only compare the name, otherwise the
            // device/inode.
            let url = path_with_url(fname.as_ptr()) != 0;
            let file_id = if url {
                None
            } else {
                let mut file_id = FileID::default();
                if !os_fileid(fname.as_ptr(), &raw mut file_id) {
                    return false;
                }
                Some(file_id)
            };

            let known = self.entries.iter().any(|seen| {
                let same = match (&seen.file_id, &file_id) {
                    (Some(seen_id), Some(id)) => os_fileid_equal(seen_id, id),
                    (None, None) => path_fnamecmp(seen.fname.as_ptr(), fname.as_ptr()) == 0,
                    _ => false,
                };
                same && ff_wc_equal(seen.wc_path.as_ptr(), wc_path.as_ptr())
            });
            if known {
                return false;
            }

            // New file/dir. Add it to the list of visited files/dirs.
            self.entries.push(Visited {
                wc_path,
                file_id,
                fname: if url { fname } else { Name::default() },
            });
            true
        }
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
    ///
    /// # Safety
    /// `filename` must hold `filenamelen` readable bytes.
    pub(crate) unsafe fn select(&mut self, filename: *const c_char, filenamelen: usize) {
        unsafe {
            let filename =
                Name::from_bytes(slice::from_raw_parts(filename.cast::<u8>(), filenamelen));
            let found = self
                .lists
                .iter()
                .position(|list| path_fnamecmp(filename.as_ptr(), list.filename.as_ptr()) == 0);
            self.at = found.unwrap_or_else(|| {
                self.lists.push(VisitedList {
                    filename,
                    entries: Vec::new(),
                });
                self.lists.len() - 1
            });
        }
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
/// # Safety
/// Both must be NUL-terminated strings.
pub(crate) unsafe fn ff_wc_equal(s1: *const c_char, s2: *const c_char) -> bool {
    unsafe {
        if s1 == s2 {
            return true;
        }
        let ignorecase = p_fic.get() != 0;
        let fold = |c: c_int| if ignorecase { mb_tolower(c) } else { c };

        let (mut i, mut j) = (0usize, 0usize);
        let mut prev1 = 0;
        let mut prev2 = 0;
        while *s1.add(i) != 0 && *s2.add(j) != 0 {
            let c1 = utf_ptr2char(s1.add(i));
            let c2 = utf_ptr2char(s2.add(j));
            if fold(c1) != fold(c2) && (prev1 != '*' as c_int || prev2 != '*' as c_int) {
                return false;
            }
            prev2 = prev1;
            prev1 = c1;
            i += utfc_ptr2len(s1.add(i)) as usize;
            j += utfc_ptr2len(s2.add(j)) as usize;
        }
        *s1.add(i) == *s2.add(j)
    }
}

/// Is `path`, for its first `path_len` bytes, at or below one of the stop
/// directories?
///
/// A parent matches: `/home` stops a search whose start directory is
/// `/home/rks`. The separator test is what keeps `/home/r` from matching
/// `/home/rks`.
///
/// # Safety
/// There must be a current buffer, for `'fileignorecase'`.
pub(crate) unsafe fn ff_path_in_stoplist(path: &Name, path_len: usize, stopdirs: &[Name]) -> bool {
    unsafe {
        // Eat up trailing path separators, except the first.
        let mut path_len = path_len;
        while path_len > 1 && vim_ispathsep(path.at(path_len - 1) as c_int) {
            path_len -= 1;
        }
        // If no path consider it as match.
        if path_len == 0 {
            return true;
        }

        stopdirs.iter().any(|stop| {
            path_fnamencmp(stop.as_ptr(), path.as_ptr(), path_len) == 0
                && (stop.len() <= path_len || vim_ispathsep(stop.at(path_len) as c_int))
        })
    }
}
