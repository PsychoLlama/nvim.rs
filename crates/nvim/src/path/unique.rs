//! Shortening a list of names to the shortest ones that still differ.
//!
//! Command-line completion of file names shows the tails rather than whole
//! paths, and [`uniquefy_paths`] is what decides how much tail each entry
//! needs: the shortest suffix that no other entry shares, extended by whole
//! components until it is unique. [`path_shorten_fname`] and
//! [`shorten_dir_name`] are the simpler shortenings — relative to a
//! directory, and one letter per component — that `'shortmess'` and the
//! status line use.

#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use crate::cstr;
use crate::memory::XString;
use core::ffi::c_int;

use std::ffi::CStr;

use super::*;
use crate::fileio::file_pat_to_regpat;
use crate::mbyte::{cluster_len, head_off};
use crate::os::fs::{current_dir, dirname_text};
use crate::regexp::{OwnedMatch, RE_MAGIC, RE_STRING};

/// Shorten the directory part of a name in place, keeping `trim_len`
/// characters of each component: `"~/foo/../.bar/fname"` with a `trim_len`
/// of 1 becomes `"~/f/../.b/fname"`. The last component is left alone, and a
/// leading `~` or `.` does not count towards the length.
///
/// `bytes` is the string with its NUL, and the shortened name ends at the
/// first NUL after. `trim_len` must be at least 1.
pub fn shorten_dir_name(bytes: &mut [u8], trim_len: c_int) {
    let len = bytes
        .iter()
        .position(|&b| b == 0)
        .expect("a NUL-terminated name");
    let tail_at = tail_index(&bytes[..len]);

    // Where the next kept byte goes. Never past `read`, so the copy is
    // always backwards over ground already read.
    let mut write = 0;
    let mut read = 0;
    let mut skip = false;
    let mut chunk_len = 0;
    loop {
        if read >= tail_at {
            // The tail is copied whole.
            bytes[write] = bytes[read];
            write += 1;
            if bytes[read] == 0 {
                break;
            }
        } else if vim_ispathsep(c_int::from(bytes[read])) {
            // A separator starts a new component.
            bytes[write] = bytes[read];
            write += 1;
            skip = false;
            chunk_len = 0;
        } else if !skip {
            bytes[write] = bytes[read];
            write += 1;
            if bytes[read] != b'~' && bytes[read] != b'.' {
                // Only word characters count towards the length.
                chunk_len += 1;
                skip = chunk_len >= trim_len;
            }
            // A character is kept whole, however many bytes it takes.
            for _ in 1..cluster_len(&bytes[read..len]) {
                read += 1;
                bytes[write] = bytes[read];
                write += 1;
            }
        }
        read += 1;
    }
}

/// [`shorten_dir_name`] with one character per directory component.
pub fn shorten_dir(bytes: &mut [u8]) {
    shorten_dir_name(bytes, 1);
}

/// Move `sep` back to the previous path separator in `path`, answering
/// false when it ends up at the start of `path` instead.
fn find_previous_pathsep(path: &[u8], sep: &mut usize) -> bool {
    // Step off the separator this started on.
    if *sep > 0 && vim_ispathsep(c_int::from(path[*sep])) {
        *sep -= 1;
    }
    while *sep > 0 {
        if vim_ispathsep(c_int::from(path[*sep])) {
            return true;
        }
        // MB_PTR_BACK: to the head of the character before this one.
        *sep -= head_off(path, *sep - 1) + 1;
    }
    false
}

/// Is `maybe_unique` the tail of no entry of `names` but the `i`th?
///
/// A tail only counts when it starts at a component boundary, so `"bar"` is
/// still unique against `"foobar"`.
fn is_unique(maybe_unique: &[u8], names: &[XString], i: usize) -> bool {
    names.iter().enumerate().all(|(j, other)| {
        if j == i || other.len() < maybe_unique.len() {
            // Not compared with itself; and it's different when it's
            // shorter.
            return true;
        }
        let rival_at = other.len() - maybe_unique.len();
        let matched = path_fnamecmp(maybe_unique, &other[rival_at..]) == 0
            && (rival_at == 0 || vim_ispathsep(c_int::from(other[rival_at - 1])));
        !matched
    })
}

/// Where `fname` stops being a directory that `paths` — the expanded
/// `'path'` — already names, so that everything before it can come off.
///
/// `None` when no entry shares even a byte with it.
fn get_path_cutoff(fname: &[u8], paths: &[XString]) -> Option<usize> {
    // The longest prefix any `'path'` entry shares with the name.
    let mut maxlen = 0;
    let mut cutoff = None;
    for part in paths {
        let shared = fname
            .iter()
            .zip(part.iter())
            .take_while(|(a, b)| a == b)
            .count();
        if shared > maxlen {
            maxlen = shared;
            cutoff = Some(shared);
        }
    }
    // On to the file or directory name itself.
    cutoff.map(|mut at| {
        while vim_ispathsep(c_int::from(cstr::byte_at(fname, at))) {
            at += cluster_len(&fname[at..]);
        }
        at
    })
}

/// Reduce every name in `names` to the shortest tail of it that still tells
/// it apart from the others, and that `pattern` still matches.
///
/// A name under the current directory is answered relative to it — as
/// `"file"` when that is unique, and as `"./file"` when it is not.
/// `path_option` is the `'path'` the names were found through, whose
/// directories are what a name may be shortened *to*.
pub(crate) fn uniquefy_paths(names: &mut Vec<XString>, pattern: &CStr, path_option: &[u8]) {
    remove_duplicate_names(names);

    // The pattern has to match anywhere in a path, so it gets a leading
    // "*". FIXME(upstream): is this valid for all possible patterns?
    let mut file_pattern = vec![b'*'];
    file_pattern.extend_from_slice(pattern.to_bytes());
    let Some(pat) = file_pat_to_regpat(&cstr::owned(&file_pattern)) else {
        return;
    };
    // Always ignoring case.
    let Some(mut regmatch) = OwnedMatch::compile(pat.as_cstr(), RE_MAGIC + RE_STRING, true) else {
        return;
    };
    drop(pat);

    // What a failing `os_dirname` leaves behind -- its error message -- is
    // what upstream goes on with.
    let curdir = dirname_text();
    let curdir: &[u8] = &curdir;
    let path_dirs = expand_path_option(curdir, path_option);

    // A "**/" pattern can reach files that their own name alone cannot,
    // so those only lose the prefix they share with the `'path'`.
    let pattern = pattern.to_bytes();
    let starstar =
        pattern.starts_with(b"**") && vim_ispathsep_nocolon(c_int::from(cstr::byte_at(pattern, 2)));

    // The names that turn out to be under the current directory, kept
    // unshortened for the second pass.
    let mut in_curdir: Vec<Option<XString>> = vec![None; names.len()];
    let mut sort_again = false;

    for i in 0..names.len() {
        if got_int.get() {
            break;
        }
        let path = &names[i];
        let len = path.len();
        let dir_end = gettail_dir(path);
        if path_fnamencmp(curdir, path, dir_end) == 0 && cstr::byte_at(curdir, dir_end) == 0 {
            in_curdir[i] = Some(path.clone());
        }

        // Shorten the name while keeping it unique.
        let cutoff = get_path_cutoff(path, &path_dirs);
        let mut keep_from = None;
        if let Some(cutoff) = cutoff.filter(|&cutoff| {
            starstar
                && vim_regexec(&mut regmatch, path.cstr_at(cutoff), 0)
                && is_unique(&path[cutoff..], names, i)
        }) {
            keep_from = Some(cutoff);
        } else {
            // Every file here can be reached without its path, so take
            // the shortest unique tail — walking back one separator at a
            // time from the end.
            let mut sep = len.saturating_sub(1);
            while find_previous_pathsep(path, &mut sep) {
                let tail = sep + 1;
                if vim_regexec(&mut regmatch, path.cstr_at(tail), 0)
                    && is_unique(&path[tail..], names, i)
                    && cutoff.is_some_and(|cutoff| tail >= cutoff)
                {
                    keep_from = Some(tail);
                    break;
                }
            }
        }
        if let Some(at) = keep_from {
            sort_again = true;
            names[i] = XString::from_bytes(&names[i][at..]);
        }

        let path = &names[i];
        if path_is_absolute(path.as_cstr()) {
            // Last resort: relative to the current directory, when the
            // file is under it and the result is actually shorter.
            //
            //     Before               curdir      After
            //     /foo/bar/file.txt    /foo/bar    ./file.txt
            //     /file.txt            /           /file.txt
            if let Some(short) = shorten_index(path, curdir).filter(|&short| short > 1) {
                let mut relative = XString::from_bytes(b"./");
                relative.push_bytes(&path[short..]);
                names[i] = relative;
            }
        }
        os_breakcheck();
    }

    // The names in the current directory can lose it entirely — unless
    // that leaves something another entry also ends with, in which case
    // they keep a leading "./".
    for (i, slot) in in_curdir.iter().enumerate() {
        if got_int.get() {
            break;
        }
        let Some(path) = slot else {
            continue;
        };
        let short = &path[shorten_index(path, curdir).unwrap_or(0)..];
        if is_unique(short, names, i) {
            names[i] = XString::from_bytes(short);
            continue;
        }
        let mut relative = XString::from_bytes(b"./");
        relative.push_bytes(short);
        names[i] = relative;
        sort_again = true;
        os_breakcheck();
    }

    drop(regmatch);
    if sort_again {
        remove_duplicate_names(names);
    }
}

/// Sort `names` bytewise and drop the ones [`path_fnamecmp`] calls equal to
/// the one before them -- upstream's `ga_remove_duplicate_strings`.
pub(crate) fn remove_duplicate_names(names: &mut Vec<XString>) {
    names.sort_unstable_by(|a, b| a.as_cstr().cmp(b.as_cstr()));
    let mut i = names.len();
    while i > 1 {
        i -= 1;
        if path_fnamecmp(&names[i - 1], &names[i]) == 0 {
            names.remove(i);
        }
    }
}

/// Where the last directory component of `fname` starts: `"a/b/c/"` answers
/// at `"c/"`, and `"a/b/c"` at `"b/c"` — the trailing name is not a
/// directory unless a separator says it is.
pub fn gettail_dir(fname: &[u8]) -> usize {
    let mut dir_end = 0;
    let mut next_dir_end = 0;
    let mut look_for_sep = true;
    for (at, &b) in fname.iter().enumerate() {
        if b == b'/' {
            // The first separator of a run ends the component before it.
            if look_for_sep {
                next_dir_end = at;
                look_for_sep = false;
            }
        } else {
            // A component after a separator: what came before it was a
            // directory after all.
            if !look_for_sep {
                dir_end = next_dir_end;
            }
            look_for_sep = true;
        }
    }
    dir_end
}

/// [`path_shorten_fname`] against the current directory, answering
/// `full_path` itself when it cannot be shortened.
pub fn path_try_shorten_fname(full_path: &CStr) -> &CStr {
    let Some(dirname) = current_dir() else {
        return full_path;
    };
    match path_shorten_fname(full_path, &dirname) {
        Some(short) if !short.is_empty() => short,
        _ => full_path,
    }
}

/// The part of `full_path` that names it relative to `dir_name`, or `None`
/// when it is not under that directory at all.
pub fn path_shorten_fname<'a>(full_path: &'a CStr, dir_name: &[u8]) -> Option<&'a CStr> {
    shorten_index(full_path.to_bytes(), dir_name).map(|at| cstr::suffix(full_path, at))
}

/// [`path_shorten_fname`] as an index into `full_path`.
pub(crate) fn shorten_index(full_path: &[u8], dir_name: &[u8]) -> Option<usize> {
    let dir_name = cstr::until_nul(dir_name);
    let len = dir_name.len();

    // Names that do not start alike cannot be made relative at all.
    if path_fnamencmp(dir_name, full_path, len) != 0 {
        return None;
    }
    // Everything is under the head of a path.
    if len == path_head_length() && is_path_head(dir_name) {
        return Some(len);
    }

    // Without a separator here, `full_path`'s last directory is merely
    // longer than `dir_name`'s — they are different directories.
    let mut p = len;
    if !vim_ispathsep(c_int::from(cstr::byte_at(full_path, p))) {
        return None;
    }
    loop {
        p += 1;
        if !vim_ispathsep_nocolon(c_int::from(cstr::byte_at(full_path, p))) {
            break;
        }
    }
    Some(p)
}
