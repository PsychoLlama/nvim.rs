//! Turning one file name into another.
//!
//! [`modname`] builds the "same name with a different extension" that backups,
//! swap files and `:make` want, honouring the `BASENAMELEN` limit on how long
//! a basename may get. [`vim_rename`] and [`vim_copyfile`] move a file,
//! falling back from `rename` to a copy when the two paths turn out to be on
//! different filesystems. [`file_pat_to_reg_pat`] compiles a shell-style file
//! pattern into a regexp, which [`match_file_pat`]/[`match_file_list`] then
//! run against a name — that is how `'wildignore'`, `'backupskip'` and
//! autocommand patterns are matched.

#![deny(unsafe_op_in_unsafe_fn)]

use core::ffi::{c_char, c_int, c_void};
use std::ffi::CStr;

use crate::src::nvim::path::tail_index;

#[allow(unused_imports)]
use super::*;

/// `S_IFLNK`: the file type bits of a symbolic link.
const S_IFLNK: u64 = 0o120000;

/// Shorten `buf`'s displayed file name to be relative to `dirname`.
pub unsafe fn shorten_buf_fname(buf: *mut buf_T, dirname: *mut c_char, force: c_int) {
    unsafe {
        if (*buf).b_fname.is_null()
            || bt_nofilename(buf)
            || path_with_url((*buf).b_fname) != 0
            || !(force != 0 || (*buf).b_sfname.is_null() || path_is_absolute((*buf).b_sfname))
        {
            return;
        }
        if (*buf).b_sfname != (*buf).b_ffname {
            xfree((*buf).b_sfname.cast());
            (*buf).b_sfname = core::ptr::null_mut();
        }
        let p = path_shorten_fname((*buf).b_ffname, dirname);
        if p.is_null() {
            (*buf).b_fname = (*buf).b_ffname;
        } else {
            (*buf).b_sfname = xstrdup(p);
            (*buf).b_fname = (*buf).b_sfname;
        }
    }
}

/// Shorten file names for all buffers.
pub unsafe extern "C" fn shorten_fnames(force: c_int) {
    unsafe {
        let mut dirname = [0 as c_char; MAXPATHL as usize];
        os_dirname(dirname.as_mut_ptr(), MAXPATHL as size_t);
        let mut buf = firstbuf.get();
        while !buf.is_null() {
            shorten_buf_fname(buf, dirname.as_mut_ptr(), force);
            // Always make the swap file name a full path; a "nofile" buffer
            // may also have a swap file.
            mf_fullname((*buf).b_ml.ml_mfp);
            buf = (*buf).b_next;
        }
        status_redraw_all();
        redraw_tabline.set(true);
    }
}

/// Get a new file name ended by the given extension.
///
/// @param fname        The original file name. If NULL or empty, the current
///                     directory name is used instead.
/// @param ext          The extension to add. 4 characters max if it starts
///                     with a dot, 3 otherwise.
/// @param prepend_dot  Prefix the basename with a dot. Does nothing if it
///                     already starts with one, or if `fname` was empty.
///
/// @return [allocated] The new name, guaranteed to end with `ext`, to have a
///                     basename of at most `BASENAMELEN` characters, and to
///                     differ from `fname` — basename characters are replaced
///                     with `_` if that is what it takes, and if the whole
///                     truncated basename was already underscores, the first
///                     becomes a `v`. NULL only when `fname` was empty and
///                     the current directory could not be read.
pub unsafe extern "C" fn modname(
    fname: *const c_char,
    ext: *const c_char,
    prepend_dot: bool,
) -> *mut c_char {
    unsafe {
        let ext = CStr::from_ptr(ext).to_bytes();
        let mut prepend_dot = prepend_dot;
        // Room for `os_dirname`'s MAXPATHL plus the separator below.
        let mut cwd = [0 as c_char; MAXPATHL as usize + 2];

        let name: &[u8] = if fname.is_null() || *fname == 0 {
            // With no file name we need the name of the current directory —
            // in full, in case `:cd` is used.
            if os_dirname(cwd.as_mut_ptr(), MAXPATHL as size_t) == FAIL
                || CStr::from_ptr(cwd.as_ptr()).is_empty()
            {
                return core::ptr::null_mut();
            }
            add_pathsep(cwd.as_mut_ptr());
            prepend_dot = false; // nothing to prepend a dot to
            CStr::from_ptr(cwd.as_ptr()).to_bytes()
        } else {
            CStr::from_ptr(fname).to_bytes()
        };

        // Everything after the last path separator is the basename, and it
        // may keep at most BASENAMELEN characters. Upstream's backwards walk
        // stops before the first byte, so a name that *is* a separator there
        // ("/foo") keeps it as part of the basename.
        let start = name[1..]
            .iter()
            .rposition(|&b| b == b'/')
            .map_or(0, |at| at + 2);
        let ptrlen = (name.len() - start).min(BASENAMELEN as usize);

        let mut out = Vec::with_capacity(start + ptrlen + ext.len() + 2);
        out.extend_from_slice(&name[..start + ptrlen]);
        // The extension starts here, and this is where the search for a
        // character to replace below starts from.
        let ext_at = out.len();
        out.extend_from_slice(ext);

        if prepend_dot {
            let e = tail_index(&out);
            if out.get(e) != Some(&b'.') {
                out.insert(e, b'.');
            }
        }

        // Check that, after appending the extension, the file name really is
        // different.
        if !fname.is_null() && CStr::from_ptr(fname).to_bytes() == out {
            // Look backwards through the basename for a character that can be
            // replaced by '_'.
            let mut at = ext_at;
            let mut replaced = false;
            while at > start {
                at -= 1;
                if out[at] != b'_' {
                    out[at] = b'_';
                    replaced = true;
                    break;
                }
            }
            if !replaced {
                // fname was "________.<ext>", how tricky!
                match out.get_mut(start) {
                    Some(slot) => *slot = b'v',
                    None => out.push(b'v'),
                }
            }
        }

        xmemdupz(out.as_ptr().cast(), out.len()).cast()
    }
}

/// Rename `from` to `to` via a third name in the same directory.
///
/// Needed when the two names refer to the same file but are spelled
/// differently, which a plain rename would treat as a no-op.
unsafe fn rename_with_tmp(from: *const c_char, to: *const c_char) -> c_int {
    unsafe {
        let from_len = CStr::from_ptr(from).to_bytes().len();
        if from_len >= MAXPATHL as usize - 5 {
            return -1;
        }

        let mut tempname = [0 as c_char; MAXPATHL as usize + 1];
        core::ptr::copy_nonoverlapping(from, tempname.as_mut_ptr(), from_len + 1);
        // Everything up to the tail stays put; only the last component is
        // replaced with a number.
        let tail = tail_index(CStr::from_ptr(tempname.as_ptr()).to_bytes());

        for n in 123..99999 {
            let digits = n.to_string();
            let end = tail + digits.len();
            tempname[tail..end].copy_from_slice(core::slice::from_raw_parts(
                digits.as_ptr().cast::<c_char>(),
                digits.len(),
            ));
            tempname[end] = 0;

            if os_path_exists(tempname.as_ptr()) {
                continue;
            }
            if os_rename(from, tempname.as_ptr()) != OK {
                // If it fails for one temp name it will most likely fail for
                // any temp name, so give up.
                return -1;
            }
            if os_rename(tempname.as_ptr(), to) == OK {
                return 0;
            }
            // Strange, the second step failed. Try moving the file back and
            // report the failure.
            os_rename(tempname.as_ptr(), from);
            return -1;
        }
        -1
    }
}

/// Rename `from` to `to`, copying the file across if a rename cannot do it.
///
/// `os_rename` only works when both names are on the same file system.
///
/// @return  -1 for failure, 0 for success
pub unsafe extern "C" fn vim_rename(from: *const c_char, to: *const c_char) -> c_int {
    unsafe {
        let mut use_tmp_file = false;

        // When the names are identical there is nothing to do. When they refer
        // to the same file but the spelling differs we have to go through a
        // temp file.
        if path_fnamecmp(from, to) == 0 {
            if p_fic.get() != 0 && strcmp(path_tail(from), path_tail(to)) != 0 {
                use_tmp_file = true;
            } else {
                return 0;
            }
        }

        // Fail if the "from" file doesn't exist. Avoids that "to" is deleted.
        let mut from_info = FileInfo::default();
        if !os_fileinfo(from, &raw mut from_info) {
            return -1;
        }

        // It's possible for the source and destination to be the same file.
        // This happens when "from" and "to" differ in case and are on a FAT32
        // filesystem. In that case go through a temp file name.
        let mut to_info = FileInfo::default();
        if os_fileinfo(to, &raw mut to_info)
            && os_fileinfo_id_equal(&raw mut from_info, &raw mut to_info)
        {
            use_tmp_file = true;
        }

        if use_tmp_file {
            return rename_with_tmp(from, to);
        }

        // Delete the "to" file. This is required on some systems to make the
        // rename work, and on others it makes sure we don't end up with two
        // files when the rename fails.
        os_remove(to);

        // First try a normal rename, and return if it works.
        if os_rename(from, to) == OK {
            return 0;
        }

        // The rename failed, try copying the file.
        if vim_copyfile(from, to) != OK {
            return -1;
        }
        if os_fileinfo(from, &raw mut from_info) {
            os_remove(from);
        }
        0
    }
}

/// Copy `from` to `to`, with the same permissions and ACL.
///
/// A symbolic link is copied as a link, not as its target.
///
/// @return  FAIL for failure, OK for success
pub unsafe extern "C" fn vim_copyfile(from: *const c_char, to: *const c_char) -> c_int {
    unsafe {
        let mut from_info = FileInfo::default();
        if os_fileinfo_link(from, &raw mut from_info)
            && from_info.stat.st_mode & __S_IFMT as u64 == S_IFLNK
        {
            let mut linkbuf = [0 as c_char; MAXPATHL as usize + 1];
            let len = readlink(from, linkbuf.as_mut_ptr(), MAXPATHL as size_t);
            if len <= 0 {
                return FAIL;
            }
            linkbuf[len as usize] = 0;
            return if symlink(linkbuf.as_ptr(), to) == 0 {
                OK
            } else {
                FAIL
            };
        }

        // For systems that support ACL: get the ACL from the original file.
        let acl = os_get_acl(from);
        if os_copy(from, to, UV_FS_COPYFILE_EXCL) != 0 {
            os_free_acl(acl);
            return FAIL;
        }
        os_set_acl(to, acl);
        os_free_acl(acl);
        OK
    }
}

/// Try matching a file name with `pattern`, or with the pre-compiled `prog`
/// when that avoids recompiling the same pattern over and over.
///
/// Used for autocommands and `'wildignore'`.
///
/// @param pattern    pattern to match with, when `prog` is NULL
/// @param prog       pre-compiled regprog, or NULL
/// @param fname      full path of the file name
/// @param sfname     short file name, or NULL
/// @param tail       tail of the path
/// @param allow_dirs the pattern may match a directory
pub unsafe extern "C" fn match_file_pat(
    pattern: *mut c_char,
    prog: *mut *mut regprog_T,
    fname: *mut c_char,
    sfname: *mut c_char,
    tail: *mut c_char,
    allow_dirs: c_int,
) -> bool {
    unsafe {
        let mut regmatch = regmatch_T {
            rm_ic: p_fic.get() != 0, // ignore case if 'fileignorecase' is set
            regprog: if prog.is_null() {
                vim_regcomp(pattern, RE_MAGIC)
            } else {
                *prog
            },
            ..Default::default()
        };

        // Try for a match with the pattern with:
        // 1. the full file name, when the pattern has a '/'.
        // 2. the short file name, when the pattern has a '/'.
        // 3. the tail of the file name, when the pattern has no '/'.
        let result = !regmatch.regprog.is_null()
            && if allow_dirs != 0 {
                vim_regexec(&raw mut regmatch, fname, 0)
                    || (!sfname.is_null() && vim_regexec(&raw mut regmatch, sfname, 0))
            } else {
                vim_regexec(&raw mut regmatch, tail, 0)
            };

        if prog.is_null() {
            vim_regfree(regmatch.regprog);
        } else {
            *prog = regmatch.regprog;
        }
        result
    }
}

/// Check whether a file matches any pattern in `list`.
///
/// @param list    comma-separated list of patterns, like `'wildignore'`
/// @param sfname  short file name
/// @param ffname  full file name
pub unsafe extern "C" fn match_file_list(
    list: *mut c_char,
    sfname: *mut c_char,
    ffname: *mut c_char,
) -> bool {
    unsafe {
        let tail = path_tail(sfname);
        let mut p = list;
        while *p != 0 {
            let mut buf = [0 as c_char; MAXPATHL as usize];
            copy_option_part(
                &raw mut p,
                buf.as_mut_ptr(),
                buf.len(),
                c",".as_ptr().cast_mut(),
            );
            let mut allow_dirs: c_char = 0;
            let regpat = file_pat_to_reg_pat(
                buf.as_ptr(),
                core::ptr::null(),
                &raw mut allow_dirs,
                false as c_int,
            );
            if regpat.is_null() {
                break;
            }
            let matched = match_file_pat(
                regpat,
                core::ptr::null_mut(),
                ffname,
                sfname,
                tail,
                allow_dirs as c_int,
            );
            xfree(regpat.cast());
            if matched {
                return true;
            }
        }
        false
    }
}

/// Convert `pat`, which has shell-style wildcards in it, into a regular
/// expression. Backslashes before special characters, like `\*` and `\ `, are
/// handled -- webb.
///
/// # Safety
///
/// `pat` must be NUL-terminated at or after `pat_end`: upstream reads up to
/// two bytes past `pat_end`, each guarded by the one before it not being NUL.
///
/// @param pat_end     first char after the pattern, or NULL for its end
/// @param allow_dirs  set when a directory path separator has to be matched
/// @param no_bslash   don't use a backward slash as a path separator (only
///                    makes a difference on Windows, so never here)
///
/// @return            [allocated] the regexp, or NULL when the braces in the
///                    pattern do not balance.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn file_pat_to_reg_pat(
    pat: *const c_char,
    pat_end: *const c_char,
    allow_dirs: *mut c_char,
    _no_bslash: c_int,
) -> *mut c_char {
    unsafe {
        if !allow_dirs.is_null() {
            *allow_dirs = false as c_char;
        }
        let note_dir = |c: u8| {
            if c == b'/' && !allow_dirs.is_null() {
                *allow_dirs = true as c_char;
            }
        };

        let end = if pat_end.is_null() {
            CStr::from_ptr(pat).to_bytes().len()
        } else {
            pat_end.offset_from(pat) as usize
        };
        if end == 0 {
            return xstrdup(c"^$".as_ptr());
        }
        let at = |i: usize| *pat.add(i) as u8;

        let mut reg = Vec::<u8>::with_capacity(end * 2 + 3);

        // A pattern that starts with stars matches anywhere, so it needs
        // neither the leading '^' nor all of the stars.
        let mut start = 0;
        if at(0) == b'*' {
            while at(start) == b'*' && start < end - 1 {
                start += 1;
            }
        } else {
            reg.push(b'^');
        }

        // Likewise, trailing stars make the '$' pointless.
        let mut last = end - 1;
        let mut add_dollar = true;
        if last >= start && at(last) == b'*' {
            while last > start && at(last) == b'*' {
                last -= 1;
            }
            add_dollar = false;
        }

        let mut nested = 0;
        let mut p = start;
        while p <= last && at(p) != 0 && nested >= 0 {
            match at(p) {
                b'*' => {
                    reg.extend_from_slice(b".*");
                    while at(p + 1) == b'*' {
                        // "**" matches like "*".
                        p += 1;
                    }
                }
                c @ (b'.' | b'~') => {
                    reg.push(b'\\');
                    reg.push(c);
                }
                b'?' => reg.push(b'.'),
                b'\\' => {
                    if at(p + 1) == 0 {
                        break;
                    }
                    p += 1;
                    // Undo the escaping ExpandEscape() added:
                    //   foo\?bar -> foo?bar
                    //   foo\%bar -> foo%bar
                    //   foo\,bar -> foo,bar
                    //   foo\ bar -> foo bar
                    // Don't unescape '\', '*' and the others that are also
                    // special in a regexp. An escaped '{' must be unescaped
                    // since we use magic, not verymagic: "\\\{n,m\}" is how
                    // you get "\{n,m}".
                    let c = at(p);
                    if c == b'?' {
                        reg.push(b'?');
                    } else if c == b','
                        || c == b'%'
                        || c == b'#'
                        || ascii_isspace(c as c_int)
                        || c == b'{'
                        || c == b'}'
                    {
                        reg.push(c);
                    } else if c == b'\\' && at(p + 1) == b'\\' && at(p + 2) == b'{' {
                        reg.extend_from_slice(b"\\{");
                        p += 2;
                    } else {
                        note_dir(c);
                        reg.push(b'\\');
                        reg.push(c);
                    }
                }
                b'{' => {
                    reg.extend_from_slice(b"\\(");
                    nested += 1;
                }
                b'}' => {
                    reg.extend_from_slice(b"\\)");
                    nested -= 1;
                }
                b',' => {
                    if nested != 0 {
                        reg.extend_from_slice(b"\\|");
                    } else {
                        reg.push(b',');
                    }
                }
                c => {
                    note_dir(c);
                    reg.push(c);
                }
            }
            p += 1;
        }
        if add_dollar {
            reg.push(b'$');
        }

        if nested != 0 {
            emsg(gettext(if nested < 0 {
                c"E219: Missing {.".as_ptr()
            } else {
                c"E220: Missing }.".as_ptr()
            }));
            return core::ptr::null_mut();
        }
        xmemdupz(reg.as_ptr().cast::<c_void>(), reg.len()).cast()
    }
}
