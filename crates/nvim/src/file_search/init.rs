//! Setting up a downward-and-upward file search.
//!
//! [`vim_findfile_init`] takes one entry of `'path'`, `'tags'` or
//! `'cdpath'` apart into the fixed leading part, the wildcard tail, and the
//! directory the search starts from, then pushes the first directory onto
//! the context's stack for [`vim_findfile`](super::vim_findfile) to walk.
//! The `**` wildcard's depth limiter is parsed here — `**3` is stored as
//! `**` followed by a byte holding 3 — and so is the `;` that asks for the
//! upward search ([`vim_findfile_stopdir`]).

#![deny(unsafe_op_in_unsafe_fn)]

use super::*;
use crate::option::cpo_has;
use crate::semsg_c;
use crate::types::{CpoFlag, FAIL, MAXPATHL};
use ::libc::strtol;
use core::ffi::{c_char, c_int, c_void};
use core::{ptr, slice};

/// A name held for `len` bytes at `p`.
///
/// # Safety
/// `p` must hold `len` readable bytes.
unsafe fn name_of(p: *const c_char, len: usize) -> Name {
    // SAFETY: the caller's promise.
    Name::from_bytes(unsafe { slice::from_raw_parts(p.cast::<u8>(), len) })
}

/// `FullName_save`'s answer, owned.
///
/// # Safety
/// `p` must be a NUL-terminated string.
unsafe fn full_name_of(p: *const c_char, force: bool) -> Name {
    unsafe {
        let full = FullName_save(p, force);
        let name = Name::from_ptr(full);
        xfree(full.cast());
        name
    }
}

/// Does `name` need a path separator adding before something else goes
/// after it?
///
/// # Safety
/// There is nothing to promise; `name` carries its own length.
unsafe fn needs_separator(name: &Name) -> bool {
    // SAFETY: a `Name` holds `len()` bytes and a terminator.
    unsafe { after_pathsep(name.as_ptr(), name.as_ptr().add(name.len())) == 0 }
}

/// Where the search should start when the given path is relative.
///
/// A leading `"./"` means the current *file*'s directory, unless this is a
/// `'tags'` search and `'cpoptions'` holds `"d"`. Anything else relative
/// starts from the current directory. An absolute path has its own starting
/// directory worked out later, from its fixed part.
///
/// Answers where the path proper begins, which is past the `"./"`.
///
/// # Safety
/// `path` and `rel_fname` must be NUL-terminated strings, `rel_fname` may be
/// NULL.
unsafe fn starting_dir(
    ctx: &mut FindContext,
    path: *mut c_char,
    rel_fname: *const c_char,
    tagfile: bool,
) -> Result<*mut c_char, ()> {
    unsafe {
        let dot_slash =
            *path == b'.' as c_char && (vim_ispathsep(*path.add(1) as c_int) || *path.add(1) == 0);
        if dot_slash && (!tagfile || !cpo_has(CpoFlag::DOTTAG)) && !rel_fname.is_null() {
            let len = path_tail(rel_fname.cast_mut()).offset_from(rel_fname) as usize;
            ctx.start_dir = Some(
                if !vim_is_abs_name(rel_fname) && len + 1 < MAXPATHL as usize {
                    // Make the start dir an absolute path name.
                    full_name_of(name_of(rel_fname, len).as_ptr(), false)
                } else {
                    name_of(rel_fname, len)
                },
            );
            // Step over the "." and the separator after it, if any.
            let path = path.add(1);
            return Ok(if *path != 0 { path.add(1) } else { path });
        }

        if *path == 0 || !vim_is_abs_name(path) {
            let mut curdir = [0 as c_char; MAXPATHL as usize];
            if os_dirname(curdir.as_mut_ptr(), MAXPATHL as usize) == FAIL {
                return Err(());
            }
            ctx.start_dir = Some(Name::from_ptr(curdir.as_ptr()));
        }
        Ok(path)
    }
}

/// Split `stopdirs` into the directories the upward search stops at.
///
/// An empty entry means "ascend to the top of the directory tree", which is
/// why `";"` and `""` both give an unlimited upward search.
///
/// # Safety
/// `stopdirs` must be a NUL-terminated string.
unsafe fn stop_directories(stopdirs: *mut c_char) -> Vec<Name> {
    unsafe {
        let mut walker = stopdirs;
        while *walker == b';' as c_char {
            walker = walker.add(1);
        }

        let mut dirs = Vec::new();
        loop {
            let entry = walker;
            let next = vim_strchr(walker, c_int::from(b';'));
            let len = if next.is_null() {
                strlen(entry)
            } else {
                next.offset_from(entry) as usize
            };

            dirs.push(
                if *entry != 0 && !vim_is_abs_name(entry) && len + 1 < MAXPATHL as usize {
                    // Upstream copies the entry into a scratch buffer and then
                    // resolves `entry` instead, which is not NUL-terminated at
                    // the ';': a relative stop directory with another after it
                    // resolves the whole rest of the string. It also passes the
                    // entry's length where `force` goes. Preserved — this is
                    // what the option means today.
                    full_name_of(entry, len != 0)
                } else {
                    name_of(entry, len)
                },
            );

            let Some(next) = (!next.is_null()).then(|| next.add(1)) else {
                return dirs;
            };
            walker = next;
        }
    }
}

/// Copy the wildcard tail, encoding each `**`'s descent limit.
///
/// The octet after a `**` is used as a binary counter, so `**3` becomes
/// `**` and a byte holding 3, and `**76` becomes `**` and an `L`. `**0`
/// removes the `**` altogether, and no number at all means
/// [`FF_MAX_STAR_STAR_EXPAND`]. Because of this technique the path looks
/// awful if you print it as a string.
///
/// # Safety
/// `wc_part` must be a NUL-terminated string.
unsafe fn wildcard_tail(wc_part: *mut c_char) -> Result<Name, ()> {
    unsafe {
        let mut tail = Vec::<u8>::new();
        let mut at = wc_part;
        while *at != 0 {
            if tail.len() + 5 >= MAXPATHL as usize {
                emsg(gettext(c"E854: Path too long for completion".as_ptr()));
                break;
            }
            if !(*at == b'*' as c_char && *at.add(1) == b'*' as c_char) {
                tail.push(*at as u8);
                at = at.add(1);
                continue;
            }

            tail.extend_from_slice(b"**");
            at = at.add(2);
            let mut errpt: *mut c_char = ptr::null_mut();
            let limit = strtol(at, &raw mut errpt, 10);
            if errpt != at && limit > 0 && limit < 255 {
                tail.push(limit as u8);
            } else if errpt != at && limit == 0 {
                // A restrict of 0: remove the '**' already added.
                tail.truncate(tail.len() - 2);
            } else {
                tail.push(FF_MAX_STAR_STAR_EXPAND);
            }

            at = errpt;
            if *at != 0 && !vim_ispathsep(*at as c_int) {
                semsg_c!(
                    gettext(
                        c"E343: Invalid path: '**[number]' must be at the end of the path or be followed by '%s'."
                            .as_ptr(),
                    ),
                    c"/".as_ptr(),
                );
                return Err(());
            }
        }
        Ok(Name::from_bytes(&tail))
    }
}

/// The directory the first stack frame should search, and the wildcards it
/// should search with.
///
/// The fixed part of the path may name a directory, in which case it all
/// belongs to the starting directory; otherwise its last component is a name
/// pattern and moves to the front of the wildcards.
///
/// # Safety
/// There must be a current buffer.
unsafe fn first_frame(ctx: &mut FindContext) -> Result<Name, ()> {
    unsafe {
        let start_dir = ctx.start_dir.as_ref().expect("set above");
        // Create an absolute path.
        if start_dir.len() + ctx.fix_path.len() + 3 >= MAXPATHL as usize {
            emsg(gettext(c"E854: Path too long for completion".as_ptr()));
            return Err(());
        }

        let mut dir = start_dir.bytes().to_vec();
        if needs_separator(start_dir) {
            dir.push(b'/');
        }

        let mut whole = dir.clone();
        whole.extend_from_slice(ctx.fix_path.bytes());
        if os_isdir(Name::from_bytes(&whole).as_ptr()) {
            if !ctx.fix_path.is_empty() {
                dir.extend_from_slice(ctx.fix_path.bytes());
                if needs_separator(&ctx.fix_path) {
                    dir.push(b'/');
                }
            }
            return Ok(Name::from_bytes(&dir));
        }

        // The fixed part's last component is a name, not a directory.
        let tail = path_tail(ctx.fix_path.as_ptr().cast_mut());
        let mut kept = ctx.fix_path.len();
        if tail.cast_const() > ctx.fix_path.as_ptr() {
            // Do not add '..' to the path and start upwards searching.
            kept = tail.offset_from(ctx.fix_path.as_ptr()) as usize - 1;
            if kept >= 2
                && ctx.fix_path.bytes().starts_with(b"..")
                && (kept == 2 || ctx.fix_path.at(2) == b'/')
            {
                return Err(());
            }
            dir.extend_from_slice(&ctx.fix_path.bytes()[..kept]);
            // The separator test is upstream's, and it asks about the whole
            // fixed part rather than the piece just written.
            if needs_separator(&ctx.fix_path) {
                dir.push(b'/');
            }
        }

        if let Some(wc_path) = &ctx.wc_path {
            let mut moved = ctx.fix_path.bytes()[kept..].to_vec();
            moved.extend_from_slice(wc_path.bytes());
            ctx.wc_path = Some(Name::from_bytes(&moved));
        }
        Ok(Name::from_bytes(&dir))
    }
}

/// Create or reinitialise a search context for [`vim_findfile`].
///
/// Don't forget to clean up by calling [`vim_findfile_cleanup`] when you are
/// done with the search context.
///
/// Find the file `filename` in the directory `path`. `path` may contain
/// wildcards; if so only search `level` directories deep. `level` is the
/// absolute maximum and is not related to the restriction given to the `**`
/// wildcard: with a level of 100 and a `**200`, the search still stops after
/// 100 levels.
///
/// `filename` cannot contain wildcards. It is used as-is, with no
/// backslashes to escape special characters.
///
/// If `stopdirs` is not NULL and nothing is found downward, the search is
/// restarted on the next higher directory level, repeatedly, until the
/// starting directory of a search is contained in `stopdirs`. `stopdirs` has
/// the format `";*<dirname>*\(;<dirname>\)*;\=$"`.
///
/// If `path` is relative, the search starts in Vim's current directory, or
/// in the current file's directory when it starts with `"./"`. If `path` is
/// absolute, the search starts at the part of it before the first wildcard.
/// Upward search is only done on the starting dir.
///
/// If `free_visited` is true the list of already visited files and
/// directories is cleared. Set it false if you just want to search from
/// another directory but want to be sure no directory from a previous search
/// is searched again — useful when looking for a file in several places.
///
/// A search context returned by a previous call can be passed in
/// `search_ctx_arg`; it is reused and reinitialised, and its list of
/// already visited directories is only deleted when `free_visited` is true.
/// Be aware that `search_ctx_arg` is freed if the reinitialisation fails. If
/// you don't have a search context from a previous call it must be NULL.
///
/// This function silently ignores a few errors; [`vim_findfile`] will have
/// limited functionality then.
///
/// @param tagfile  expanding names of tags files
/// @param rel_fname  file name to use for "."
///
/// @return  the newly allocated search context, or NULL if an error occurred.
#[allow(clippy::too_many_arguments)]
pub unsafe fn vim_findfile_init(
    path: *mut c_char,
    filename: *mut c_char,
    filenamelen: size_t,
    stopdirs: *mut c_char,
    level: c_int,
    free_visited: bool,
    find_what: c_int,
    search_ctx_arg: *mut c_void,
    tagfile: bool,
    rel_fname: *mut c_char,
) -> *mut c_void {
    unsafe {
        // If a search context is given by the caller, reuse it, else make a
        // new one.
        let mut ctx = if search_ctx_arg.is_null() {
            Box::new(FindContext::default())
        } else {
            Box::from_raw(search_ctx_arg.cast::<FindContext>())
        };
        ctx.find_what = find_what;
        ctx.tagfile = tagfile;
        // Clear the search context, but NOT the visited lists.
        ctx.reset();

        if free_visited {
            ctx.visited.clear();
            ctx.dir_visited.clear();
        } else {
            // Reuse the old visited lists: get the list for the given
            // filename, creating one if none exists yet.
            ctx.visited.select(filename, filenamelen);
            ctx.dir_visited.select(filename, filenamelen);
        }

        // Store information on the starting dir now if the path is relative.
        // If it is absolute we do that below, from the fixed part.
        let Ok(path) = starting_dir(&mut ctx, path, rel_fname, tagfile) else {
            return ptr::null_mut();
        };

        // If stopdirs are given, split them into an array. If a stop
        // directory is not recognized there is no upward search at all;
        // see ff_path_in_stoplist() for the details.
        if !stopdirs.is_null() {
            ctx.stopdirs = Some(stop_directories(stopdirs));
        }

        ctx.level = level;

        // Split into the fixed part and the wildcard part.
        let wc_part = vim_strchr(path, c_int::from(b'*'));
        if wc_part.is_null() {
            ctx.fix_path = Name::from_ptr(path);
        } else {
            ctx.fix_path = name_of(path, wc_part.offset_from(path) as usize);
            let Ok(tail) = wildcard_tail(wc_part) else {
                return ptr::null_mut();
            };
            ctx.wc_path = Some(tail);
        }

        if ctx.start_dir.is_none() {
            // Store the fixed part as the starting dir. This is needed when
            // the given path is fully qualified.
            ctx.start_dir = Some(ctx.fix_path.clone());
            ctx.fix_path.clear();
        }

        let Ok(fix_path) = first_frame(&mut ctx) else {
            return ptr::null_mut();
        };
        let wc_path = ctx.wc_path.clone().unwrap_or_default();
        ctx.stack
            .push(StackFrame::new(fix_path, wc_path, level, false));
        ctx.file_to_search = name_of(filename, filenamelen);
        Box::into_raw(ctx).cast()
    }
}

/// Split a `'path'` entry at its first unescaped `;`, answering the stop
/// directories after it — or NULL when there are none.
///
/// The entry itself is left in `buf`, NUL-terminated at the `;` and with
/// every `"\;"` in it collapsed to a plain `;`.
///
/// # Safety
/// `buf` must be a writable NUL-terminated string.
pub unsafe fn vim_findfile_stopdir(buf: *mut c_char) -> *mut c_char {
    unsafe {
        let at = |i: usize| *buf.add(i);
        // Nothing before the first escape needs moving.
        let mut read = 0;
        while at(read) != 0
            && at(read) != b';' as c_char
            && !(at(read) == b'\\' as c_char && at(read + 1) == b';' as c_char)
        {
            read += 1;
        }

        // From here every "\;" loses its backslash, so the entry shortens
        // as it is copied over itself.
        let mut write = read;
        while at(read) != 0 && at(read) != b';' as c_char {
            if at(read) == b'\\' as c_char && at(read + 1) == b';' as c_char {
                *buf.add(write) = b';' as c_char;
                read += 2;
            } else {
                *buf.add(write) = at(read);
                read += 1;
            }
            write += 1;
        }

        let ends_at_semicolon = at(read) == b';' as c_char;
        if write < read {
            *buf.add(write) = 0;
        }
        if ends_at_semicolon {
            *buf.add(read) = 0;
            return buf.add(read + 1);
        }
        ptr::null_mut()
    }
}

/// Free the lists of visited files and directories. Can handle a NULL
/// pointer.
///
/// # Safety
/// `ctx` must be a search context, or NULL.
pub(crate) unsafe fn vim_findfile_free_visited(ctx: *mut c_void) {
    if ctx.is_null() {
        return;
    }
    // SAFETY: the caller's context came from `vim_findfile_init`.
    let ctx = unsafe { &mut *ctx.cast::<FindContext>() };
    ctx.visited.clear();
    ctx.dir_visited.clear();
}

/// Clean up the given search context. Can handle a NULL pointer.
pub unsafe fn vim_findfile_cleanup(ctx: *mut c_void) {
    if ctx.is_null() {
        return;
    }
    // SAFETY: the caller's context came from `vim_findfile_init`.
    drop(unsafe { Box::from_raw(ctx.cast::<FindContext>()) });
}
