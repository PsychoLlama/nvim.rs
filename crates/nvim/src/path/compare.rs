//! Deciding whether two names mean the same file.
//!
//! [`path_full_compare`] is the one with an answer for every case: same file,
//! different files, one missing, both missing — resolving both names and
//! consulting the file system when it has to. [`pathcmp`] is the text-only
//! comparison that sorts and matches names, treating a path separator as
//! less than any other byte so `"foo/bar"` sorts before `"foo-bar"`.
//! [`path_fix_case`] replaces a name's last component with the spelling the
//! file system actually uses, which is what makes completion look right on a
//! case-insensitive volume.

#![deny(unsafe_op_in_unsafe_fn)]

use core::ffi::{c_char, c_int};
use std::ffi::CStr;

use super::*;
use crate::types::{MAXPATHL, NUL};

/// Compare two file names, by identity where the file system can say and by
/// name where it cannot.
///
/// Answers [`kEqualFiles`], [`kDifferentFiles`], [`kOneFileMissing`],
/// [`kBothFilesMissing`], or — when neither file exists, `checkname` is set
/// and their full paths agree — [`kEqualFileNames`].
///
/// # Safety
/// Both names must be NUL-terminated strings. With `expandenv`, `s1` also
/// has its environment variables expanded.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn path_full_compare(
    s1: *mut c_char,
    s2: *mut c_char,
    checkname: bool,
    expandenv: bool,
) -> FileComparison {
    unsafe {
        let mut expanded1 = [0 as c_char; MAXPATHL as usize];
        if expandenv {
            expand_env(s1, expanded1.as_mut_ptr(), MAXPATHL);
        } else {
            xstrlcpy(expanded1.as_mut_ptr(), s1, MAXPATHL as size_t);
        }

        let mut file_id_1 = FileID::default();
        let mut file_id_2 = FileID::default();
        let id_ok_1 = os_fileid(expanded1.as_mut_ptr(), &raw mut file_id_1);
        let id_ok_2 = os_fileid(s2, &raw mut file_id_2);

        if !id_ok_1 && !id_ok_2 {
            // With no id from the file system the names are all there is.
            if checkname {
                let mut full1 = [0 as c_char; MAXPATHL as usize];
                let mut full2 = [0 as c_char; MAXPATHL as usize];
                vim_FullName(
                    expanded1.as_mut_ptr(),
                    full1.as_mut_ptr(),
                    MAXPATHL as size_t,
                    false,
                );
                vim_FullName(s2, full2.as_mut_ptr(), MAXPATHL as size_t, false);
                if path_fnamecmp(full1.as_mut_ptr(), full2.as_mut_ptr()) == 0 {
                    return kEqualFileNames;
                }
            }
            return kBothFilesMissing;
        }
        if !id_ok_1 || !id_ok_2 {
            return kOneFileMissing;
        }
        if os_fileid_equal(&raw const file_id_1, &raw const file_id_2) {
            kEqualFiles
        } else {
            kDifferentFiles
        }
    }
}

/// Rewrite the last component of `name` with the spelling the file system
/// holds, when the two differ only in case.
///
/// Nothing happens unless a directory entry matches case-insensitively, is
/// the same byte length — a longer name would not fit where this one is —
/// and turns out to be the same file.
///
/// # Safety
/// `name` must be a writable NUL-terminated string: its directory part is
/// terminated in place while the directory is opened, as upstream does.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn path_fix_case(name: *mut c_char) {
    unsafe {
        let mut file_info = FileInfo::default();
        if !os_fileinfo_link(name, &raw mut file_info) {
            return;
        }

        // Open the directory the file is in.
        let mut dir = Directory::default();
        let at = CStr::from_ptr(name)
            .to_bytes()
            .iter()
            .rposition(|&b| b == b'/');
        let tail = match at {
            None => {
                if !os_scandir(&raw mut dir, c".".as_ptr()) {
                    return;
                }
                name
            }
            Some(at) => {
                let slash = name.add(at);
                *slash = 0;
                let ok = os_scandir(&raw mut dir, name);
                *slash = b'/' as c_char;
                if !ok {
                    return;
                }
                slash.add(1)
            }
        };

        let head = tail.offset_from(name) as usize;
        let taillen = CStr::from_ptr(tail).to_bytes().len();
        loop {
            let entry = os_scandir_next(&raw mut dir);
            if entry.is_null() {
                break;
            }
            // Only names that differ in case and are the same byte length.
            // TODO(upstream): accept a different length name.
            if strcasecmp(tail, entry) != 0 || taillen != CStr::from_ptr(entry).to_bytes().len() {
                continue;
            }
            // Then check it really is this file, and not a second one whose
            // name happens to fold to the same thing.
            let mut newname = [0 as c_char; MAXPATHL as usize + 1];
            xstrlcpy(newname.as_mut_ptr(), name, MAXPATHL as size_t + 1);
            xstrlcpy(
                newname.as_mut_ptr().add(head),
                entry,
                (MAXPATHL as usize - head + 1) as size_t,
            );
            let mut file_info_new = FileInfo::default();
            if os_fileinfo_link(newname.as_mut_ptr(), &raw mut file_info_new)
                && os_fileinfo_id_equal(&raw const file_info, &raw const file_info_new)
            {
                strcpy(tail, entry);
                break;
            }
        }
        os_closedir(&raw mut dir);
    }
}

/// Are `f1` and `f2` in the same directory? `f1` may be a short name; `f2`
/// must be a full path.
///
/// # Safety
/// Both must be NUL-terminated strings, or NULL for "no".
pub unsafe fn same_directory(f1: *mut c_char, f2: *mut c_char) -> bool {
    unsafe {
        if f1.is_null() || f2.is_null() {
            return false;
        }
        let mut ffname = [0 as c_char; MAXPATHL as usize];
        let full = ffname.as_mut_ptr();
        vim_FullName(f1, full, MAXPATHL as size_t, false);
        let head = path_tail_with_sep(full).offset_from(full);
        head == path_tail_with_sep(f2).offset_from(f2) && pathcmp(full, f2, head as c_int) == 0
    }
}

/// Compare the file names `p` and `q`, over at most `maxlen` bytes of each
/// when `maxlen` is not negative.
///
/// Answers zero when they name the same thing, which includes one of them
/// having a trailing separator the other lacks. Otherwise the sign says
/// which sorts first, and a path separator sorts before anything else — so
/// `"foo/bar"` comes before `"foo-bar"`, and a name that is a prefix of the
/// other comes first. `'fileignorecase'` folds case.
///
/// # Safety
/// Both must be NUL-terminated strings.
pub unsafe fn pathcmp(p: *const c_char, q: *const c_char, maxlen: c_int) -> c_int {
    unsafe {
        let ignorecase = p_fic.get() != 0;
        let fold = |c: c_int| if ignorecase { mb_toupper(c) } else { c };
        let limit = if maxlen < 0 {
            usize::MAX
        } else {
            maxlen as usize
        };

        // Where one name ran out, and how far into it that was. Staying NULL
        // means the comparison ran into `maxlen` with both still going.
        let mut short: *const c_char = core::ptr::null();
        let mut at = 0;
        let mut j = 0;
        while at < limit && j < limit {
            let c1 = utf_ptr2char(p.add(at));
            let c2 = utf_ptr2char(q.add(j));

            // End of one name: the other may just have a trailing separator.
            if c1 == NUL {
                if c2 == NUL {
                    return 0;
                }
                short = q;
                at = j;
                break;
            }
            if c2 == NUL {
                short = p;
                break;
            }

            if fold(c1) != fold(c2) {
                if vim_ispathsep(c1) {
                    return -1;
                }
                if vim_ispathsep(c2) {
                    return 1;
                }
                return fold(c1) - fold(c2);
            }
            at += utfc_ptr2len(p.add(at)) as usize;
            j += utfc_ptr2len(q.add(j)) as usize;
        }
        if short.is_null() {
            return 0;
        }

        // The longer name matches if all it has left is a trailing
        // separator — but "//" and ":/" are not that.
        let rest = short.add(at);
        let c1 = utf_ptr2char(rest);
        let c2 = utf_ptr2char(rest.add(utfc_ptr2len(rest) as usize));
        if c2 == NUL && at > 0 && after_pathsep(short, rest) == 0 && c1 == c_int::from(b'/') {
            return 0;
        }
        if short == q { -1 } else { 1 }
    }
}
