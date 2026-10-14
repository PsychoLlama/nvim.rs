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

#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use crate::cstr;
use core::ffi::c_int;
use std::ffi::CStr;

use super::*;
use crate::os::env::expand_env_into;
use crate::os::fs::{file_id, link_info};
use crate::types::{MAXPATHL, NUL};

/// Compare two file names, by identity where the file system can say and by
/// name where it cannot.
///
/// Answers [`kEqualFiles`], [`kDifferentFiles`], [`kOneFileMissing`],
/// [`kBothFilesMissing`], or — when neither file exists, `checkname` is set
/// and their full paths agree — [`kEqualFileNames`]. With `expandenv`, `s1`
/// has its environment variables expanded first.
pub fn path_full_compare(s1: &CStr, s2: &CStr, checkname: bool, expandenv: bool) -> FileComparison {
    let expanded1 = if expandenv {
        // Upstream's buffer is `MAXPATHL` bytes and so is its bound.
        expand_env_into(s1, MAXPATHL as usize + 1)
    } else {
        let s1 = s1.to_bytes();
        s1[..s1.len().min(MAXPATHL as usize - 1)].to_vec()
    };
    let expanded1 = cstr::owned(&expanded1);

    let id1 = file_id(&expanded1);
    let id2 = file_id(s2);
    match (id1, id2) {
        (None, None) => {
            // With no id from the file system the names are all there is.
            if checkname {
                let full = |name: &CStr| {
                    vim_full_name(name, MAXPATHL as usize, false).unwrap_or_else(|name| name)
                };
                if path_fnamecmp(&full(&expanded1), &full(s2)) == 0 {
                    return kEqualFileNames;
                }
            }
            kBothFilesMissing
        }
        (Some(id1), Some(id2)) if id1 == id2 => kEqualFiles,
        (Some(_), Some(_)) => kDifferentFiles,
        _ => kOneFileMissing,
    }
}

/// Rewrite the last component of `name` with the spelling the file system
/// holds, when the two differ only in case.
///
/// Nothing happens unless a directory entry matches case-insensitively, is
/// the same byte length — a longer name would not fit where this one is —
/// and turns out to be the same file.
pub fn path_fix_case(name: &mut [u8]) {
    let Some(file_info) = cstr::with_terminated(name, link_info) else {
        return;
    };

    // Open the directory the file is in.
    let mut dir = Directory::default();
    let head = match name.iter().rposition(|&b| b == b'/') {
        None => {
            if !os_scandir(&mut dir, c".") {
                return;
            }
            0
        }
        Some(at) => {
            if !cstr::with_terminated(&name[..at], |dir_name| os_scandir(&mut dir, dir_name)) {
                return;
            }
            at + 1
        }
    };

    while let Some(entry) = os_scandir_next(&mut dir) {
        let entry = entry.to_bytes();
        // Only names that differ in case and are the same byte length.
        // TODO(upstream): accept a different length name.
        if !name[head..].eq_ignore_ascii_case(entry) {
            continue;
        }
        // Then check it really is this file, and not a second one whose
        // name happens to fold to the same thing.
        let mut newname = name[..head].to_vec();
        newname.extend_from_slice(entry);
        newname.truncate(MAXPATHL as usize);
        let same = cstr::with_terminated(&newname, link_info)
            .is_some_and(|info| info.same_file_as(&file_info));
        if same {
            let entry = entry.to_vec();
            name[head..].copy_from_slice(&entry);
            break;
        }
    }
    os_closedir(&mut dir);
}

/// Are `f1` and `f2` in the same directory? `f1` may be a short name; `f2`
/// must be a full path.
pub fn same_directory(f1: &CStr, f2: &CStr) -> bool {
    let full = vim_full_name(f1, MAXPATHL as usize, false).unwrap_or_else(|name| name);
    let head = tail_with_sep_index(&full);
    let head2 = tail_with_sep_index(f2.to_bytes());
    head == head2 && pathcmp(&full, f2.to_bytes(), Some(head)) == 0
}

/// Which of the two names [`pathcmp`] was still reading when the other ran
/// out.
#[derive(PartialEq, Eq)]
enum Side {
    Left,
    Right,
}

/// Compare the file names `p` and `q`, over at most `maxlen` bytes of each.
/// Each ends at its end or its first NUL.
///
/// Answers zero when they name the same thing, which includes one of them
/// having a trailing separator the other lacks. Otherwise the sign says
/// which sorts first, and a path separator sorts before anything else — so
/// `"foo/bar"` comes before `"foo-bar"`, and a name that is a prefix of the
/// other comes first. `'fileignorecase'` folds case.
pub fn pathcmp(p: &[u8], q: &[u8], maxlen: Option<usize>) -> c_int {
    let ignorecase = p_fic();
    let fold = |c: c_int| if ignorecase { mb_toupper(c) } else { c };
    let limit = maxlen.unwrap_or(usize::MAX);
    let (left, right) = (prefix_of(p, limit), prefix_of(q, limit));

    // The name that still had bytes left when the other ran out: which one
    // it is, and how far into it the leftovers start. `None` means neither
    // ran out -- the comparison reached `maxlen` with both still going.
    let mut leftover: Option<(Side, &[u8], usize)> = None;
    let mut at = 0;
    let mut j = 0;
    while at < limit && j < limit {
        // The end of a slice is the name's end. The other name may still
        // match, if all it has left is a trailing separator.
        if at >= left.len() {
            if j >= right.len() {
                return 0;
            }
            leftover = Some((Side::Right, right, j));
            break;
        }
        if j >= right.len() {
            leftover = Some((Side::Left, left, at));
            break;
        }

        let c1 = char_at(&left[at..]);
        let c2 = char_at(&right[j..]);
        if fold(c1) != fold(c2) {
            if vim_ispathsep(c1) {
                return -1;
            }
            if vim_ispathsep(c2) {
                return 1;
            }
            return fold(c1) - fold(c2);
        }
        at += cluster_len(&left[at..]);
        j += cluster_len(&right[j..]);
    }

    let Some((side, name, at)) = leftover else {
        return 0;
    };

    // The longer name matches if all it has left is a trailing
    // separator — but "//" and ":/" are not that.
    let rest = &name[at..];
    let after = char_at(&rest[cluster_len(rest)..]);
    if after == NUL && at > 0 && !after_pathsep(name, at) && char_at(rest) == c_int::from(b'/') {
        return 0;
    }
    if side == Side::Right { -1 } else { 1 }
}
