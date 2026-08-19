//! Shortening a list of names to the shortest ones that still differ.
//!
//! Command-line completion of file names shows the tails rather than whole
//! paths, and [`uniquefy_paths`] is what decides how much tail each entry
//! needs: the shortest suffix that no other entry shares, extended by whole
//! components until it is unique. [`path_shorten_fname`] and
//! [`shorten_dir_len`] are the simpler shortenings — relative to a directory,
//! and one letter per component — that `'shortmess'` and the status line
//! use.

#![deny(unsafe_op_in_unsafe_fn)]

use core::ffi::{c_char, c_int};
use std::ffi::CStr;

use super::*;
use crate::regexp::{RE_MAGIC, RE_STRING};
use crate::types::{MAXPATHL, OK, PATHSEPSTR};

/// Shorten the directory part of `str` in place, keeping `trim_len`
/// characters of each component: `"~/foo/../.bar/fname"` with a `trim_len`
/// of 1 becomes `"~/f/../.b/fname"`. The last component is left alone, and a
/// leading `~` or `.` does not count towards the length.
///
/// # Safety
/// `str` must be a writable NUL-terminated string. `trim_len` must be at
/// least 1.
pub unsafe fn shorten_dir_len(str: *mut c_char, trim_len: c_int) {
    unsafe {
        let tail = path_tail(str);
        // Where the next kept byte goes. Never past `s`, so the copy is
        // always backwards over ground already read.
        let mut d = str;
        let mut s = str;
        let mut skip = false;
        let mut chunk_len = 0;
        loop {
            if s >= tail {
                // The tail is copied whole.
                *d = *s;
                d = d.add(1);
                if *s == 0 {
                    break;
                }
            } else if vim_ispathsep(*s as c_int) {
                // A separator starts a new component.
                *d = *s;
                d = d.add(1);
                skip = false;
                chunk_len = 0;
            } else if !skip {
                *d = *s;
                d = d.add(1);
                if *s != b'~' as c_char && *s != b'.' as c_char {
                    // Only word characters count towards the length.
                    chunk_len += 1;
                    skip = chunk_len >= trim_len;
                }
                // A character is kept whole, however many bytes it takes.
                for _ in 1..utfc_ptr2len(s) {
                    s = s.add(1);
                    *d = *s;
                    d = d.add(1);
                }
            }
            s = s.add(1);
        }
    }
}

/// [`shorten_dir_len`] with one character per directory component.
///
/// # Safety
/// `str` must be a writable NUL-terminated string.
pub unsafe fn shorten_dir(str: *mut c_char) {
    unsafe { shorten_dir_len(str, 1) }
}

/// Move `sep` back to the previous path separator in `path`, answering
/// false when it ends up at the start of `path` instead.
///
/// # Safety
/// `sep` must point into `path`, which must be a NUL-terminated string.
unsafe fn find_previous_pathsep(path: *mut c_char, sep: &mut *mut c_char) -> bool {
    unsafe {
        // Step off the separator this started on.
        if *sep > path && vim_ispathsep(**sep as c_int) {
            *sep = sep.sub(1);
        }
        while *sep > path {
            if vim_ispathsep(**sep as c_int) {
                return true;
            }
            // MB_PTR_BACK: to the head of the character before this one.
            *sep = sep.sub(utf_head_off(path, sep.sub(1)) as usize + 1);
        }
        false
    }
}

/// Is `maybe_unique` the tail of no entry of `gap` but the `i`th?
///
/// A tail only counts when it starts at a component boundary, so `"bar"` is
/// still unique against `"foobar"`.
///
/// # Safety
/// `gap` must hold `ga_len` NUL-terminated strings.
pub(crate) unsafe fn is_unique(maybe_unique: *mut c_char, gap: *mut garray_T, i: c_int) -> bool {
    unsafe {
        let candidate = CStr::from_ptr(maybe_unique).to_bytes();
        for j in 0..(*gap).ga_len {
            if j == i {
                continue; // don't compare it with itself
            }
            let other = *(*gap).ga_data.cast::<*mut c_char>().add(j as usize);
            let other_len = CStr::from_ptr(other).to_bytes().len();
            if other_len < candidate.len() {
                continue; // it's different when it's shorter
            }
            let rival = other.add(other_len - candidate.len());
            if path_fnamecmp(maybe_unique, rival) == 0
                && (rival == other || vim_ispathsep(*rival.sub(1) as c_int))
            {
                return false; // match
            }
        }
        true
    }
}

/// Where `fname` stops being a directory that `gap` — the expanded `'path'`
/// — already names, so that everything before it can come off.
///
/// NULL when `gap` is empty.
///
/// # Safety
/// `fname` must be a NUL-terminated string and `gap` must hold `ga_len`
/// NUL-terminated strings.
pub(crate) unsafe fn get_path_cutoff(fname: *mut c_char, gap: *mut garray_T) -> *mut c_char {
    unsafe {
        // The longest prefix any `'path'` entry shares with the name.
        let mut maxlen = 0;
        let mut cutoff = core::ptr::null_mut();
        for i in 0..(*gap).ga_len {
            let part = *(*gap).ga_data.cast::<*const c_char>().add(i as usize);
            let mut j = 0;
            while *fname.add(j) == *part.add(j) && *fname.add(j) != 0 && *part.add(j) != 0 {
                j += 1;
            }
            if j > maxlen {
                maxlen = j;
                cutoff = fname.add(j);
            }
        }
        // On to the file or directory name itself.
        if !cutoff.is_null() {
            while vim_ispathsep(*cutoff as c_int) {
                cutoff = cutoff.add(utfc_ptr2len(cutoff) as usize);
            }
        }
        cutoff
    }
}

/// Reduce every name in `gap` to the shortest tail of it that still tells it
/// apart from the others, and that `pattern` still matches.
///
/// A name under the current directory is answered relative to it — as
/// `"file"` when that is unique, and as `"./file"` when it is not.
/// `path_option` is the `'path'` the names were found through, whose
/// directories are what a name may be shortened *to*.
///
/// # Safety
/// `gap` must hold `ga_len` allocated NUL-terminated strings, and `pattern`
/// and `path_option` must be NUL-terminated strings.
pub(crate) unsafe fn uniquefy_paths(
    gap: *mut garray_T,
    pattern: *mut c_char,
    path_option: *mut c_char,
) {
    unsafe {
        ga_remove_duplicate_strings(gap);

        // The pattern has to match anywhere in a path, so it gets a leading
        // "*". FIXME(upstream): is this valid for all possible patterns?
        let mut file_pattern = vec![b'*'];
        file_pattern.extend_from_slice(CStr::from_ptr(pattern).to_bytes_with_nul());
        let pat = file_pat_to_reg_pat(
            file_pattern.as_ptr().cast(),
            core::ptr::null(),
            core::ptr::null_mut(),
            0,
        );
        if pat.is_null() {
            return;
        }
        let mut regmatch = regmatch_T {
            rm_ic: true, // always ignore case
            ..Default::default()
        };
        regmatch.regprog = vim_regcomp(pat, RE_MAGIC + RE_STRING);
        xfree(pat.cast());
        if regmatch.regprog.is_null() {
            return;
        }

        let mut curdir = vec![0 as c_char; MAXPATHL as usize];
        os_dirname(curdir.as_mut_ptr(), MAXPATHL as size_t);
        let mut path_ga = garray_T::default();
        ga_init(&raw mut path_ga, size_of::<*mut c_char>() as c_int, 1);
        expand_path_option(curdir.as_mut_ptr(), path_option, &raw mut path_ga);

        // A "**/" pattern can reach files that their own name alone cannot,
        // so those only lose the prefix they share with the `'path'`.
        let starstar = *pattern == b'*' as c_char
            && *pattern.add(1) == b'*' as c_char
            && vim_ispathsep_nocolon(*pattern.add(2) as c_int);

        let entries = (*gap).ga_len as usize;
        let fnames = (*gap).ga_data.cast::<*mut c_char>();
        // The names that turn out to be under the current directory, kept
        // unshortened for the second pass.
        let mut in_curdir: Vec<Option<Vec<u8>>> = vec![None; entries];
        let mut sort_again = false;

        for i in 0..entries {
            if got_int.get() {
                break;
            }
            let path = *fnames.add(i);
            let len = CStr::from_ptr(path).to_bytes().len();
            let dir_end = gettail_dir(path).offset_from(path) as usize;
            if path_fnamencmp(curdir.as_ptr(), path, dir_end as size_t) == 0 && curdir[dir_end] == 0
            {
                in_curdir[i] = Some(CStr::from_ptr(path).to_bytes_with_nul().to_vec());
            }

            // Shorten the name while keeping it unique.
            let cutoff = get_path_cutoff(path, &raw mut path_ga);
            if starstar
                && !cutoff.is_null()
                && vim_regexec(&raw mut regmatch, cutoff, 0)
                && is_unique(cutoff, gap, i as c_int)
            {
                sort_again = true;
                let keep = CStr::from_ptr(cutoff).to_bytes().len() + 1;
                core::ptr::copy(cutoff, path, keep);
            } else {
                // Every file here can be reached without its path, so take
                // the shortest unique tail — walking back one separator at a
                // time from the end.
                let mut sep = path.add(len.saturating_sub(1));
                while find_previous_pathsep(path, &mut sep) {
                    let tail = sep.add(1);
                    if vim_regexec(&raw mut regmatch, tail, 0)
                        && is_unique(tail, gap, i as c_int)
                        && !cutoff.is_null()
                        && tail >= cutoff
                    {
                        sort_again = true;
                        // The NUL comes with it.
                        core::ptr::copy(tail, path, path.add(len).offset_from(tail) as usize + 1);
                        break;
                    }
                }
            }

            if path_is_absolute(path) {
                // Last resort: relative to the current directory, when the
                // file is under it and the result is actually shorter.
                //
                //     Before               curdir      After
                //     /foo/bar/file.txt    /foo/bar    ./file.txt
                //     /file.txt            /           /file.txt
                let short_name = path_shorten_fname(path, curdir.as_mut_ptr());
                if !short_name.is_null() && short_name > path.add(1) {
                    vim_snprintf(
                        path,
                        MAXPATHL as size_t,
                        c".%s%s".as_ptr(),
                        PATHSEPSTR.as_ptr(),
                        short_name,
                    );
                }
            }
            os_breakcheck();
        }

        // The names in the current directory can lose it entirely — unless
        // that leaves something another entry also ends with, in which case
        // they keep a leading "./".
        for i in 0..entries {
            if got_int.get() {
                break;
            }
            let Some(path) = in_curdir[i].as_mut() else {
                continue;
            };
            let path = path.as_mut_ptr().cast::<c_char>();
            let mut short_name = path_shorten_fname(path, curdir.as_mut_ptr());
            if short_name.is_null() {
                short_name = path;
            }
            if is_unique(short_name, gap, i as c_int) {
                strcpy(*fnames.add(i), short_name);
                continue;
            }
            // The dot, the separator, the name, and the NUL.
            let size = 2 + CStr::from_ptr(short_name).to_bytes().len() + 1;
            let rel_path: *mut c_char = xmalloc(size).cast();
            vim_snprintf(
                rel_path,
                size,
                c".%s%s".as_ptr(),
                PATHSEPSTR.as_ptr(),
                short_name,
            );
            xfree((*fnames.add(i)).cast());
            *fnames.add(i) = rel_path;
            sort_again = true;
            os_breakcheck();
        }

        ga_clear_strings(&raw mut path_ga);
        vim_regfree(regmatch.regprog);
        if sort_again {
            ga_remove_duplicate_strings(gap);
        }
    }
}

/// Where the last directory component of `fname` starts: `"a/b/c/"` answers
/// at `"c/"`, and `"a/b/c"` at `"b/c"` — the trailing name is not a
/// directory unless a separator says it is.
///
/// # Safety
/// `fname` must be a NUL-terminated string.
pub unsafe fn gettail_dir(fname: *const c_char) -> *const c_char {
    unsafe {
        let mut dir_end = 0;
        let mut next_dir_end = 0;
        let mut look_for_sep = true;
        let name = CStr::from_ptr(fname).to_bytes();
        for (at, &b) in name.iter().enumerate() {
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
        fname.add(dir_end)
    }
}

/// [`path_shorten_fname`] against the current directory, answering
/// `full_path` itself when it cannot be shortened.
///
/// # Safety
/// `full_path` must be a NUL-terminated string.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn path_try_shorten_fname(full_path: *mut c_char) -> *mut c_char {
    unsafe {
        let mut dirname = vec![0 as c_char; MAXPATHL as usize];
        if os_dirname(dirname.as_mut_ptr(), MAXPATHL as size_t) != OK {
            return full_path;
        }
        let p = path_shorten_fname(full_path, dirname.as_mut_ptr());
        if p.is_null() || *p == 0 { full_path } else { p }
    }
}

/// The part of `full_path` that names it relative to `dir_name`, or NULL
/// when it is not under that directory at all.
///
/// # Safety
/// `dir_name` must be a NUL-terminated string, and `full_path` one or NULL.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn path_shorten_fname(
    full_path: *mut c_char,
    dir_name: *mut c_char,
) -> *mut c_char {
    unsafe {
        if full_path.is_null() {
            return core::ptr::null_mut();
        }
        debug_assert!(!dir_name.is_null(), "path: shortening against no directory");
        let len = CStr::from_ptr(dir_name).to_bytes().len();

        // Names that do not start alike cannot be made relative at all.
        if path_fnamencmp(dir_name, full_path, len as size_t) != 0 {
            return core::ptr::null_mut();
        }
        // Everything is under the head of a path.
        if len == path_head_length() as usize && is_path_head(dir_name) {
            return full_path.add(len);
        }

        // Without a separator here, `full_path`'s last directory is merely
        // longer than `dir_name`'s — they are different directories.
        let mut p = full_path.add(len);
        if !vim_ispathsep(*p as c_int) {
            return core::ptr::null_mut();
        }
        loop {
            p = p.add(1);
            if !vim_ispathsep_nocolon(*p as c_int) {
                break;
            }
        }
        p
    }
}
