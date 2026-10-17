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

#![forbid(unsafe_code)]

use crate::ascii::ascii_isspace;
use crate::cstr;
use crate::memory::XString;
use core::ffi::c_int;
use std::ffi::CStr;
use std::os::unix::ffi::OsStrExt;

use crate::option::split_option_part;
use crate::os::fs::{dirname_within, file_info, link_info, path_exists};
use crate::path::tail_index;

use super::*;
use crate::regexp::{OwnedProg, RE_MAGIC};
use crate::types::{FAIL, MAXPATHL, OK};
use crate::winlayer::buffers;

/// `S_IFLNK`: the file type bits of a symbolic link.
const S_IFLNK: u64 = 0o120000;

/// Shorten `buffer`'s displayed file name to be relative to `dirname`.
pub fn shorten_buf_fname(mut buffer: Buf, dirname: &[u8], force: bool) {
    if buffer.name.is_unnamed()
        || buf_is_nofilename(Some(buffer))
        || buffer
            .name
            .shown()
            .is_some_and(|shown| path_with_url(shown) != 0)
        || !(force || buffer.name.short().is_none_or(path_is_absolute))
    {
        return;
    }
    buffer.name.drop_own_short();
    let short = buffer
        .name
        .full()
        .and_then(|full| path_shorten_fname(full, dirname));
    match short.map(XString::from_cstr) {
        // The one place in the tree that shows the *full* name.
        None => buffer.name.show_full(),
        Some(short) => buffer.name.set_short(Some(short)),
    }
}

/// Shorten file names for all buffers.
pub fn shorten_fnames(force: bool) {
    // Upstream ignores a failure here too, and works on what the buffer was
    // left holding.
    let dirname = dirname_within(MAXPATHL as usize).unwrap_or_else(|left| left);
    for buf in buffers() {
        shorten_buf_fname(buf, &dirname, force);
        // Always make the swap file name a full path; a "nofile" buffer
        // may also have a swap file.
        buf.make_swap_name_full();
    }
    status_redraw_all();
    redraw_tabline.set(true);
}

/// Get a new file name ended by the given extension.
///
/// @param fname        The original file name. If `None` or empty, the
///                     current directory name is used instead.
/// @param ext          The extension to add. 4 characters max if it starts
///                     with a dot, 3 otherwise.
/// @param prepend_dot  Prefix the basename with a dot. Does nothing if it
///                     already starts with one, or if `fname` was empty.
///
/// @return The new name, guaranteed to end with `ext`, to have a basename of
///         at most `BASENAMELEN` characters, and to differ from `fname` --
///         basename characters are replaced with `_` if that is what it
///         takes, and if the whole truncated basename was already
///         underscores, the first becomes a `v`. `None` only when `fname`
///         was empty and the current directory could not be read.
pub fn modname(fname: Option<&CStr>, ext: &[u8], prepend_dot: bool) -> Option<XString> {
    let fname = fname.map(CStr::to_bytes).filter(|name| !name.is_empty());
    let mut prepend_dot = prepend_dot;
    let cwd;
    let name: &[u8] = match fname {
        Some(name) => name,
        None => {
            // With no file name we need the name of the current directory --
            // in full, in case `:cd` is used.
            let mut dir = dirname_within(MAXPATHL as usize)
                .ok()
                .filter(|dir| !dir.is_empty())?;
            add_pathsep(&mut dir);
            prepend_dot = false; // nothing to prepend a dot to
            cwd = dir;
            &cwd
        }
    };

    // Everything after the last path separator is the basename, and it may
    // keep at most BASENAMELEN characters. Upstream's backwards walk stops
    // before the first byte, so a name that *is* a separator there ("/foo")
    // keeps it as part of the basename.
    let start = name[1..]
        .iter()
        .rposition(|&b| b == b'/')
        .map_or(0, |at| at + 2);
    let ptrlen = (name.len() - start).min(BASENAMELEN);

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
    if fname == Some(&out[..]) {
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

    Some(XString::from_vec(out))
}

/// How long a basename [`modname`] keeps.
const BASENAMELEN: usize = 255 - 5;

/// Rename `from` to `to` via a third name in the same directory: whether it
/// worked.
///
/// Needed when the two names refer to the same file but are spelled
/// differently, which a plain rename would treat as a no-op.
fn rename_with_tmp(from: &CStr, to: &CStr) -> bool {
    let from_bytes = from.to_bytes();
    if from_bytes.len() >= MAXPATHL as usize - 5 {
        return false;
    }

    // Everything up to the tail stays put; only the last component is
    // replaced with a number.
    let tail = tail_index(from_bytes);
    let mut tempname = from_bytes[..tail].to_vec();

    for n in 123..99999 {
        tempname.truncate(tail);
        tempname.extend_from_slice(n.to_string().as_bytes());
        let temp = cstr::owned(&tempname);

        if path_exists(&temp) {
            continue;
        }
        if os_rename(from, &temp) != OK {
            // If it fails for one temp name it will most likely fail for
            // any temp name, so give up.
            return false;
        }
        if os_rename(&temp, to) == OK {
            return true;
        }
        // Strange, the second step failed. Try moving the file back and
        // report the failure.
        os_rename(&temp, from);
        return false;
    }
    false
}

/// Rename `from` to `to`, copying the file across if a rename cannot do it:
/// whether it worked.
///
/// `os_rename` only works when both names are on the same file system.
pub(crate) fn rename_file(from: &CStr, to: &CStr) -> bool {
    let mut use_tmp_file = false;

    // When the names are identical there is nothing to do. When they refer
    // to the same file but the spelling differs we have to go through a
    // temp file.
    if path_fnamecmp(from.to_bytes(), to.to_bytes()) == 0 {
        if p_fic() && path_tail(from) != path_tail(to) {
            use_tmp_file = true;
        } else {
            return true;
        }
    }

    // Fail if the "from" file doesn't exist. Avoids that "to" is deleted.
    let Some(from_info) = file_info(from) else {
        return false;
    };

    // It's possible for the source and destination to be the same file.
    // This happens when "from" and "to" differ in case and are on a FAT32
    // filesystem. In that case go through a temp file name.
    if file_info(to).is_some_and(|to_info| from_info.same_file_as(&to_info)) {
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
        return true;
    }

    // The rename failed, try copying the file.
    if vim_copyfile(from, to) != OK {
        return false;
    }
    if file_info(from).is_some() {
        os_remove(from);
    }
    true
}

/// Copy `from` to `to`, with the same permissions and ACL.
///
/// A symbolic link is copied as a link, not as its target.
///
/// @return  FAIL for failure, OK for success
pub fn vim_copyfile(from: &CStr, to: &CStr) -> c_int {
    let path = |name: &CStr| std::path::PathBuf::from(std::ffi::OsStr::from_bytes(name.to_bytes()));
    if link_info(from).is_some_and(|info| info.stat.st_mode & S_IFMT as u64 == S_IFLNK) {
        // Upstream reads the link into a `MAXPATHL` buffer: a longer target
        // fails.
        let target = std::fs::read_link(path(from))
            .ok()
            .filter(|target| !target.as_os_str().is_empty())
            .filter(|target| target.as_os_str().len() <= MAXPATHL as usize);
        let Some(target) = target else {
            return FAIL;
        };
        return if std::os::unix::fs::symlink(target, path(to)).is_ok() {
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

/// `uv_fs_copyfile`'s flag: fail if the destination exists.
const UV_FS_COPYFILE_EXCL: c_int = 0x1;

/// Try matching a file name with the pre-compiled `prog`, which the matcher
/// may swap for a recompiled one (and so is the caller's to keep).
///
/// Used for autocommands. Ignores case when `'fileignorecase'` is set.
///
/// @param fname      full path of the file name
/// @param sfname     short file name, if there is one
/// @param tail       tail of the path
/// @param allow_dirs the pattern may match a directory
pub(crate) fn match_file_pat(
    prog: &mut Option<OwnedProg>,
    fname: &CStr,
    sfname: Option<&CStr>,
    tail: &CStr,
    allow_dirs: bool,
) -> bool {
    let mut regmatch = RegMatch {
        rm_ic: p_fic(),
        regprog: prog.take(),
        ..Default::default()
    };
    let result = regmatch.regprog.is_some()
        && if allow_dirs {
            // Short-circuits: the second match must not run when the full
            // name already matched, as it overwrites `regmatch`.
            vim_regexec(&mut regmatch, fname, 0)
                || sfname.is_some_and(|sfname| vim_regexec(&mut regmatch, sfname, 0))
        } else {
            vim_regexec(&mut regmatch, tail, 0)
        };
    *prog = regmatch.regprog.take();
    result
}

/// Check whether a file matches any pattern in `list`.
///
/// @param list    comma-separated list of patterns, like `'wildignore'`
/// @param sfname  short file name
/// @param ffname  full file name
pub fn match_file_list(list: &CStr, sfname: &CStr, ffname: &CStr) -> bool {
    let tail = path_tail(sfname);
    let ic = p_fic();
    let mut rest = list.to_bytes();
    while !rest.is_empty() {
        let (part, after) = split_option_part(rest, MAXPATHL as usize, b",");
        rest = after;
        let Some((regpat, allow_dirs)) = file_pat_to_reg_pat(&part, part.len()) else {
            break;
        };
        let Some(mut prog) = OwnedProg::compile(regpat.as_cstr(), RE_MAGIC) else {
            continue;
        };
        let matched = if allow_dirs {
            prog.exec(ffname, 0, ic).is_some() || prog.exec(sfname, 0, ic).is_some()
        } else {
            prog.exec(tail, 0, ic).is_some()
        };
        if matched {
            return true;
        }
    }
    false
}

/// [`file_pat_to_reg_pat`] of a whole pattern the caller holds: the regexp,
/// or `None` when the braces in the pattern do not balance.
pub(crate) fn file_pat_to_regpat(pat: &CStr) -> Option<XString> {
    let pat = pat.to_bytes();
    file_pat_to_reg_pat(pat, pat.len()).map(|(regpat, _)| regpat)
}

/// Convert the first `end` bytes of `pat`, which has shell-style wildcards
/// in it, into a regular expression. Backslashes before special characters,
/// like `\*` and `\ `, are handled -- webb.
///
/// `pat` is the pattern and whatever follows it in the caller's string, up
/// to that string's end: upstream reads up to two bytes past `end`, each
/// guarded by the one before it not being NUL, and so may this.
///
/// @return  the regexp, and whether a directory path separator has to be
///          matched; `None` when the braces in the pattern do not balance.
pub fn file_pat_to_reg_pat(pat: &[u8], end: usize) -> Option<(XString, bool)> {
    let mut allow_dirs = false;
    let mut note_dir = |c: u8| {
        if c == b'/' {
            allow_dirs = true;
        }
    };

    if end == 0 {
        return Some((XString::from_bytes(b"^$"), false));
    }
    let at = |i: usize| cstr::byte_at(pat, i);

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
        let unbalanced = if nested < 0 {
            c"E219: Missing {."
        } else {
            c"E220: Missing }."
        };
        emsg(gettext(unbalanced));
        return None;
    }
    Some((XString::from_vec(reg), allow_dirs))
}
