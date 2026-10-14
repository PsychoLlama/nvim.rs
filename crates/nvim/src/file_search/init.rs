//! Setting up a downward-and-upward file search.
//!
//! [`FindContext::start`] takes one entry of `'path'`, `'tags'` or
//! `'cdpath'` apart into the fixed leading part, the wildcard tail, and the
//! directory the search starts from, then pushes the first directory onto
//! the context's stack for [`FindContext::next_match`] to walk. The `**`
//! wildcard's depth limiter is parsed here — `**3` is stored as `**`
//! followed by a byte holding 3 — and so is the `;` that asks for the
//! upward search ([`split_stop_directories`]).

#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use super::*;
use crate::charset::skip;
use crate::cstr;
use crate::option::cpo_has;
use crate::semsg;
use crate::types::{CpoFlag, MAXPATHL};
use core::ffi::c_int;

/// `full_name_save`'s answer, owned.
fn full_name_of(name: &CStr, force: bool) -> Name {
    Name::from_bytes(&full_name_save(name, force))
}

/// Does `name` need a path separator adding before something else goes
/// after it?
fn needs_separator(name: &Name) -> bool {
    !after_pathsep(name.bytes(), name.len())
}

/// Where the search should start when the given path is relative.
///
/// A leading `"./"` means the current *file*'s directory, unless this is a
/// `'tags'` search and `'cpoptions'` holds `"d"`. Anything else relative
/// starts from the current directory. An absolute path has its own starting
/// directory worked out later, from its fixed part.
///
/// Answers where the path proper begins, which is past the `"./"`.
fn starting_dir(
    ctx: &mut FindContext,
    path: &CStr,
    rel_fname: Option<&CStr>,
    tagfile: bool,
) -> Result<usize, ()> {
    let bytes = path.to_bytes();
    let second = cstr::byte_at(bytes, 1);
    let dot_slash =
        bytes.first() == Some(&b'.') && (vim_ispathsep(c_int::from(second)) || second == 0);
    if dot_slash
        && (!tagfile || !cpo_has(CpoFlag::DOTTAG))
        && let Some(rel_fname) = rel_fname
    {
        let len = tail_index(rel_fname.to_bytes());
        let dir = Name::from_bytes(&rel_fname.to_bytes()[..len]);
        ctx.start_dir = Some(
            if !vim_is_abs_name(rel_fname) && len + 1 < MAXPATHL as usize {
                // Make the start dir an absolute path name.
                full_name_of(dir.as_cstr(), false)
            } else {
                dir
            },
        );
        // Step over the "." and the separator after it, if any.
        return Ok(if second != 0 { 2 } else { 1 });
    }

    if bytes.is_empty() || !vim_is_abs_name(path) {
        let Ok(curdir) = dirname_within(MAXPATHL as usize) else {
            return Err(());
        };
        ctx.start_dir = Some(Name::from_bytes(&curdir));
    }
    Ok(0)
}

/// Split `stopdirs` into the directories the upward search stops at.
///
/// An empty entry means "ascend to the top of the directory tree", which is
/// why `";"` and `""` both give an unlimited upward search.
fn stop_directories(stopdirs: &[u8]) -> Vec<Name> {
    // Terminated, for the name tests below, which read to the end of the
    // string rather than of the entry.
    let stopdirs = Name::from_bytes(stopdirs);
    let bytes = stopdirs.bytes();
    let mut entry = bytes.iter().take_while(|&&b| b == b';').count();

    let mut dirs = Vec::new();
    loop {
        let next = bytes[entry..]
            .iter()
            .position(|&b| b == b';')
            .map(|at| entry + at);
        let len = next.unwrap_or(bytes.len()) - entry;
        let rest = cstr::suffix(stopdirs.as_cstr(), entry);

        dirs.push(
            if !rest.is_empty() && !vim_is_abs_name(rest) && len + 1 < MAXPATHL as usize {
                // Upstream copies the entry into a scratch buffer and then
                // resolves `entry` instead, which is not NUL-terminated at
                // the ';': a relative stop directory with another after it
                // resolves the whole rest of the string. It also passes the
                // entry's length where `force` goes. Preserved — this is
                // what the option means today.
                full_name_of(rest, len != 0)
            } else {
                Name::from_bytes(&bytes[entry..entry + len])
            },
        );

        let Some(next) = next else {
            return dirs;
        };
        entry = next + 1;
    }
}

/// Copy the wildcard tail, encoding each `**`'s descent limit.
///
/// The octet after a `**` is used as a binary counter, so `**3` becomes
/// `**` and a byte holding 3, and `**76` becomes `**` and an `L`. `**0`
/// removes the `**` altogether, and no number at all means
/// [`FF_MAX_STAR_STAR_EXPAND`]. Because of this technique the path looks
/// awful if you print it as a string.
fn wildcard_tail(wc_part: &[u8]) -> Result<Name, ()> {
    let mut tail = Vec::<u8>::new();
    let mut at = 0;
    while at < wc_part.len() {
        if tail.len() + 5 >= MAXPATHL as usize {
            emsg(gettext(c"E854: Path too long for completion"));
            break;
        }
        if !wc_part[at..].starts_with(b"**") {
            tail.push(wc_part[at]);
            at += 1;
            continue;
        }

        tail.extend_from_slice(b"**");
        at += 2;
        // Read as `strtol` reads it: blanks and a sign before the digits,
        // nothing taken when there are none.
        let (limit, used, _) = skip::decimal(&wc_part[at..]);
        match u8::try_from(limit) {
            Ok(limit @ 1..=254) if used != 0 => tail.push(limit),
            // A restrict of 0: remove the '**' already added.
            Ok(0) if used != 0 => tail.truncate(tail.len() - 2),
            _ => tail.push(FF_MAX_STAR_STAR_EXPAND),
        }

        at += used;
        if at < wc_part.len() && !vim_ispathsep(c_int::from(wc_part[at])) {
            let arg0 = "/";
            semsg!(
                "E343: Invalid path: '**[number]' must be at the end of the path or be followed by '{arg0}'."
            );
            return Err(());
        }
    }
    Ok(Name::from_bytes(&tail))
}

/// The directory the first stack frame should search, and the wildcards it
/// should search with.
///
/// The fixed part of the path may name a directory, in which case it all
/// belongs to the starting directory; otherwise its last component is a name
/// pattern and moves to the front of the wildcards.
fn first_frame(ctx: &mut FindContext) -> Result<Name, ()> {
    let start_dir = ctx.start_dir.as_ref().expect("set above");
    // Create an absolute path.
    if start_dir.len() + ctx.fix_path.len() + 3 >= MAXPATHL as usize {
        emsg(gettext(c"E854: Path too long for completion"));
        return Err(());
    }

    let mut dir = start_dir.bytes().to_vec();
    if needs_separator(start_dir) {
        dir.push(b'/');
    }

    let mut whole = dir.clone();
    whole.extend_from_slice(ctx.fix_path.bytes());
    if dir_exists(Name::from_bytes(&whole).as_cstr()) {
        if !ctx.fix_path.is_empty() {
            dir.extend_from_slice(ctx.fix_path.bytes());
            if needs_separator(&ctx.fix_path) {
                dir.push(b'/');
            }
        }
        return Ok(Name::from_bytes(&dir));
    }

    // The fixed part's last component is a name, not a directory.
    let tail = tail_index(ctx.fix_path.bytes());
    let mut kept = ctx.fix_path.len();
    if tail > 0 {
        // Do not add '..' to the path and start upwards searching.
        kept = tail - 1;
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

impl FindContext {
    /// Create or reinitialise a search context for [`next_match`].
    ///
    /// Find the file `filename` in the directory `path`. `path` may contain
    /// wildcards; if so only search `level` directories deep. `level` is the
    /// absolute maximum and is not related to the restriction given to the
    /// `**` wildcard: with a level of 100 and a `**200`, the search still
    /// stops after 100 levels.
    ///
    /// `filename` cannot contain wildcards. It is used as-is, with no
    /// backslashes to escape special characters.
    ///
    /// If `stopdirs` is given and nothing is found downward, the search is
    /// restarted on the next higher directory level, repeatedly, until the
    /// starting directory of a search is contained in `stopdirs`. `stopdirs`
    /// has the format `";*<dirname>*\(;<dirname>\)*;\=$"`.
    ///
    /// If `path` is relative, the search starts in Vim's current directory,
    /// or in the directory of `rel_fname` when it starts with `"./"`. If
    /// `path` is absolute, the search starts at the part of it before the
    /// first wildcard. Upward search is only done on the starting dir.
    ///
    /// A context answered by a previous call can be passed as `reuse`: its
    /// lists of files and directories already visited are kept, one per name
    /// searched for, so that a directory searched for this name before is
    /// not searched again — useful when looking for a file in several
    /// places. It is dropped if the reinitialisation fails, which answers
    /// `None`.
    ///
    /// A few errors are silently ignored; [`next_match`] has limited
    /// functionality then.
    ///
    /// `tagfile` says the names are those of tags files.
    ///
    /// [`next_match`]: FindContext::next_match
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn start(
        reuse: Option<FindContext>,
        path: &[u8],
        filename: &[u8],
        stopdirs: Option<&[u8]>,
        level: c_int,
        find_what: c_int,
        tagfile: bool,
        rel_fname: Option<&CStr>,
    ) -> Option<FindContext> {
        let mut ctx = reuse.unwrap_or_default();
        ctx.find_what = find_what;
        ctx.tagfile = tagfile;
        // Clear the search context, but NOT the visited lists.
        ctx.reset();

        // Reuse the old visited lists: get the list for the given filename,
        // creating one if none exists yet.
        ctx.visited.select(filename);
        ctx.dir_visited.select(filename);

        // Store information on the starting dir now if the path is
        // relative. If it is absolute we do that below, from the fixed part.
        let path = Name::from_bytes(path);
        let start = starting_dir(&mut ctx, path.as_cstr(), rel_fname, tagfile).ok()?;
        let path = &path.bytes()[start..];

        // If stopdirs are given, split them into an array. If a stop
        // directory is not recognized there is no upward search at all;
        // see ff_path_in_stoplist() for the details.
        ctx.stopdirs = stopdirs.map(stop_directories);

        ctx.level = level;

        // Split into the fixed part and the wildcard part.
        match path.iter().position(|&b| b == b'*') {
            None => ctx.fix_path = Name::from_bytes(path),
            Some(wc_part) => {
                ctx.fix_path = Name::from_bytes(&path[..wc_part]);
                ctx.wc_path = Some(wildcard_tail(&path[wc_part..]).ok()?);
            }
        }

        if ctx.start_dir.is_none() {
            // Store the fixed part as the starting dir. This is needed when
            // the given path is fully qualified.
            ctx.start_dir = Some(ctx.fix_path.clone());
            ctx.fix_path.clear();
        }

        let fix_path = first_frame(&mut ctx).ok()?;
        let wc_path = ctx.wc_path.clone().unwrap_or_default();
        ctx.stack
            .push(StackFrame::new(fix_path, wc_path, level, false));
        ctx.file_to_search = Name::from_bytes(filename);
        Some(ctx)
    }

    /// Forget the files and directories already visited, for every name.
    pub(crate) fn forget_visited(&mut self) {
        self.visited.clear();
        self.dir_visited.clear();
    }
}

/// Split a `'path'` entry at its first unescaped `;`: the entry, with every
/// `"\;"` before that collapsed to a plain `;`, and the stop directories
/// after it — `None` when there is no `;`.
pub(crate) fn split_stop_directories(entry: &[u8]) -> (Vec<u8>, Option<&[u8]>) {
    let mut dir = Vec::with_capacity(entry.len());
    let mut read = 0;
    while read < entry.len() && entry[read] != b';' {
        if entry[read] == b'\\' && entry.get(read + 1) == Some(&b';') {
            dir.push(b';');
            read += 2;
        } else {
            dir.push(entry[read]);
            read += 1;
        }
    }
    let stopdirs = (read < entry.len()).then(|| &entry[read + 1..]);
    (dir, stopdirs)
}
