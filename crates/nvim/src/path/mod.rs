//! File names: taking them apart, putting them together, and expanding them.
//!
//! The parent holds the two ends of the job. [`simplify_filename`] is the
//! canonicaliser — it removes `.`, `..` and duplicate separators from a name
//! in place, asking the file system before it strips anything that a symlink
//! could make a lie. [`vim_full_name`] and its neighbours are the other end:
//! making a name absolute, which is what everything that compares or stores
//! a name wants first.
//!
//! The children carry the rest: [`names`] the pure text of a name,
//! [`compare`] the comparisons, [`unique`] the shortenings, [`glob`] the
//! file-system walk of one pattern, and [`expand`] the list-level expansion
//! a command line asks for.

#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]
// The globals here keep upstream's spelling; upper-casing them is a per-module rewrite.
#![allow(non_upper_case_globals)]

use crate::cstr;
use crate::memory::XString;
use core::ffi::c_int;
use std::ffi::CStr;

use crate::cmdexpand::globpath;
use crate::eval::eval_to_string;
use crate::ex_docmd::eval_vars;
use crate::fileio::match_file_list;
use crate::getchar::state::got_int;
use crate::global_cell::GlobalCell;
use crate::mbyte::{char_at, cluster_len, head_off, mb_isalpha, mb_toupper};
use crate::option::vars::{p_fic, p_wig};
use crate::os::env::getenv_owned;
use crate::os::fs::{
    can_execute, dir_exists, dirname_within, file_info, link_info, os_closedir,
    os_file_is_readable, os_scandir, os_scandir_next, realpath,
};
use crate::os::input::os_breakcheck;
use crate::os::shell::{get_cmd_output, os_expand_wildcards};
use crate::regexp::vim_regexec;
use crate::types::{Directory, Failed, FileComparison, MAXPATHL, PATHSEPSTR};

// The carve of the transpiled module; see each child's docs.
mod names;
pub use self::names::*;
mod compare;
pub use self::compare::*;
mod unique;
pub use self::unique::*;
mod glob;
pub use self::glob::*;
mod expand;
pub use self::expand::*;

crate::flag_set! {
    /// What a wildcard expansion should look for, and what it should do with
    /// what it finds — upstream's `EW_*` family, the `flags` argument
    /// [`expand_wildcards`] and everything under it thread.
    pub struct ExpandFlags;

    /// Include directories in the matches.
    const DIR = 1;
    /// Include plain files in the matches.
    const FILE = 2;
    /// Answer a pattern that matched nothing as itself, rather than failing.
    const NOTFOUND = 4;
    /// Append a path separator to every directory answered.
    const ADDSLASH = 8;
    /// Keep the matches `'wildignore'` and `'suffixes'` would drop.
    const KEEPALL = 16;
    /// Do not report a failure to the user.
    const SILENT = 32;
    /// Only executables, and only together with [`Self::FILE`].
    const EXEC = 64;
    /// Search `'path'` as well as the pattern's own directory.
    const PATH = 128;
    /// Match without regard to case, whatever `'fileignorecase'` says.
    const ICASE = 256;
    /// Do not report a pattern that could not be expanded at all.
    const NOERROR = 512;
    /// The pattern is a literal name, not a wildcard.
    const NOTWILD = 1024;
    /// Leave `$var` alone when escaping the pattern for the shell.
    const KEEPDOLLAR = 2048;
    /// Answer a dangling symbolic link as a match.
    const ALLLINKS = 4096;
    /// Look the pattern up in `$PATH`, as `:!` would.
    const SHELLCMD = 8192;
    /// Answer `.` and `..` as matches.
    const DODOT = 16384;
    /// Answer OK for a pattern that expanded to nothing at all.
    const EMPTYOK = 32768;
    /// Do not expand environment variables in the pattern.
    const NOTENV = 65536;
    /// Search `'cdpath'` rather than `'path'`.
    const CDPATH = 131072;
    /// Do not let CTRL-C out of the `**` walk.
    const NOBREAK = 262144;
}
pub const kEqualFileNames: FileComparison = 7;
pub const kOneFileMissing: FileComparison = 6;
pub const kBothFilesMissing: FileComparison = 4;
pub const kDifferentFiles: FileComparison = 2;
pub const kEqualFiles: FileComparison = 1;
pub const URL_BACKSLASH: c_int = 2;
pub const URL_SLASH: c_int = 1;
pub const PATHSEP: c_int = '/' as c_int;
pub const MAXSUFLEN: c_int = 30;
pub const ENV_SEPCHAR: u8 = b':';

/// The full path of `fname`, in a string the caller owns — or a copy of
/// `fname` itself when it cannot be made absolute.
pub fn full_name_save(fname: &CStr, force: bool) -> XString {
    match vim_full_name(fname, MAXPATHL as usize, force) {
        Ok(full) => XString::from_vec(full),
        Err(_) => XString::from_cstr(fname),
    }
}

/// [`full_name_save`] for a name that may already be absolute, in which case
/// it is only copied.
pub fn save_abs_path(name: &CStr) -> XString {
    if path_is_absolute(name) {
        XString::from_cstr(name)
    } else {
        full_name_save(name, true)
    }
}

/// A file name being reduced to its simplest form, in place.
struct Simplify<'a> {
    /// The name, its NUL included.
    name: &'a mut [u8],
    /// Where the name ends: the index of its NUL.
    end: usize,
    /// Where the path proper starts, past any leading separators.
    start: usize,
    /// Is the name relative — has it no leading separator?
    relative: bool,
    /// How many components have been passed that a `".."` could strip.
    components: c_int,
    /// Has a component turned up that could not be stripped? Later ones are
    /// then left alone too, rather than replacing a name that is wrong with
    /// one that only looks right.
    stripping_disabled: bool,
}

impl Simplify<'_> {
    /// Cut `name[at..upto]` out.
    fn remove(&mut self, at: usize, upto: usize) {
        self.name.copy_within(upto..self.end + 1, at);
        self.end -= upto - at;
    }

    /// Where the run of separators starting at `at` ends.
    ///
    /// # Panics
    /// If `at` is past the name's NUL.
    fn past_separators(&self, mut at: usize) -> usize {
        while vim_ispathsep(c_int::from(self.name[at])) {
            // A separator with a composing character after it is one
            // character, and upstream steps over both.
            at += cluster_len(&self.name[at..self.end]);
        }
        at
    }

    /// Handle the `".."` at `p`, answering where to carry on from.
    fn strip_parent(&mut self, mut p: usize) -> usize {
        // Past the ".." and any separators after it.
        let mut tail = self.past_separators(p + 2);

        if self.components > 0 {
            if self.can_strip(&mut p, tail) {
                // Strip the component before it. If that would leave
                // nothing and there is no trailing separator, leave a
                // single "." instead.
                if p == self.start && self.relative && self.name[tail - 1] == b'.' {
                    self.name[p] = b'.';
                    self.name[p + 1] = 0;
                    // Upstream does not move the end of the name here,
                    // so a name that simplifies to "." answers the
                    // length it had before. Preserved: the walk stops at
                    // the NUL just written, so nothing else reads it.
                    p += 1;
                } else {
                    // A component is left before it, which can lose its
                    // trailing separator as well.
                    if p > self.start && self.name[tail - 1] == b'.' {
                        p -= 1;
                    }
                    self.remove(p, tail);
                }
                self.components -= 1;
            } else {
                // Skip the ".." and start counting again: nothing before
                // it may be stripped either.
                p = tail;
                self.components = 0;
            }
        } else if p == self.start && !self.relative {
            // A leading "/.." names the root, which is already there.
            self.remove(p, tail);
        } else {
            if p == self.start + 2 && self.name[p - 2] == b'.' {
                // A leading "./" before a ".." says nothing.
                self.remove(p - 2, p);
                tail -= 2;
            }
            p = tail;
        }
        p
    }

    /// May the component before the `".."` at `p` be stripped by it? Moves
    /// `p` back to the start of that component either way. A component and
    /// a separator precede the `".."`, and `tail` indexes past it.
    fn can_strip(&mut self, p: &mut usize, tail: usize) -> bool {
        if self.stripping_disabled {
            return false;
        }

        // A component that does not exist is stripped without further
        // thought — and a symlink to a name that does not exist counts
        // as not existing.
        let file_info = self.terminated_at(*p - 1, link_info);

        // Back to the start of the component being stripped.
        *p -= 1;
        while *p > self.start && !after_pathsep(self.name, *p) {
            // MB_PTR_BACK: to the head of the character before this one.
            *p -= head_off(self.name, *p - 1) + 1;
        }
        let Some(_) = file_info else {
            return true;
        };

        // The component does exist. Stripping it may still change what
        // the name means, so ask the file system about the unstripped
        // name. That can fail when the component is not a searchable
        // directory — a regular file, say — since the trailing "/.."
        // cannot be applied then; the name is wrong, and later
        // components are left alone too.
        let Some(file_info) = self.terminated_at(tail, file_info_of) else {
            self.stripping_disabled = true;
            return false;
        };

        // That test passes for a symlink to a searchable directory too,
        // and then the directory's parent must be the same file as the
        // stripped name — which does exist, being the component's own
        // parent.
        let new_file_info = if *p == self.start && self.relative {
            file_info_of(c".")
        } else {
            self.terminated_at(*p, file_info_of)
        };
        new_file_info.unwrap_or_default().same_file_as(&file_info)
    }

    /// Ask `question` about the name cut short at `at`, which is put back
    /// afterwards.
    fn terminated_at<T>(&mut self, at: usize, question: impl FnOnce(&CStr) -> T) -> T {
        let saved = core::mem::replace(&mut self.name[at], 0);
        let answer = question(cstr::in_bytes(self.name));
        self.name[at] = saved;
        answer
    }
}

/// [`file_info`], as [`Simplify::terminated_at`] asks it.
fn file_info_of(name: &CStr) -> Option<crate::types::FileInfo> {
    file_info(name)
}

/// Reduce a file name to its simplest form, in place: no `"."` components,
/// no duplicate separators, and no `".."` that the file system agrees may
/// come off. The result is never longer than what it started as.
///
/// `name` is the string with its NUL, and the answer is upstream's length --
/// where the NUL is now, except for a name that simplifies to `"."`, which
/// answers the length it had before. Read the result up to its first NUL;
/// nothing after that NUL is meaningful any more.
pub fn simplify_filename(name: &mut [u8]) -> usize {
    let len = name
        .iter()
        .position(|&b| b == 0)
        .expect("a NUL-terminated name");
    let mut s = Simplify {
        name: &mut name[..=len],
        end: len,
        start: 0,
        relative: true,
        components: 0,
        stripping_disabled: false,
    };

    let mut p = 0;
    if vim_ispathsep(c_int::from(s.name[0])) {
        s.relative = false;
        while vim_ispathsep(c_int::from(s.name[p])) {
            p += 1;
        }
    }
    // Where the path starts, after "/" or "///".
    s.start = p;
    // Posix says that "//path" is unchanged but "///path" is "/path".
    if s.start > 2 {
        s.remove(1, p);
        p = 1;
        s.start = 1;
    }

    loop {
        // `p` is now at the character after a single separator, or at
        // the start of the path.
        let at = |i: usize| c_int::from(cstr::byte_at(s.name, i));
        if vim_ispathsep(at(p)) {
            // A duplicate separator.
            s.remove(p, p + 1);
        } else if s.name[p] == b'.' && (vim_ispathsep(at(p + 1)) || s.name[p + 1] == 0) {
            if p == s.start && s.relative {
                // Keep a single "." or a leading "./".
                p += 1 + usize::from(s.name[p + 1] != 0);
            } else {
                // Strip "./" or ".///". At the end of the name, with no
                // trailing separator, strip the "/." after the start —
                // or the "." alone at the start of an absolute name.
                let mut tail = p + 1;
                if s.name[p + 1] != 0 {
                    tail = s.past_separators(tail);
                } else if p > s.start {
                    p -= 1;
                }
                s.remove(p, tail);
            }
        } else if s.name[p] == b'.'
            && s.name[p + 1] == b'.'
            && (vim_ispathsep(at(p + 2)) || s.name[p + 2] == 0)
        {
            p = s.strip_parent(p);
        } else {
            // A simple path component: on past the separator after it.
            s.components += 1;
            p += next_component_index(cstr::until_nul(&s.name[p..]));
        }
        if s.name[p] == 0 {
            break;
        }
    }
    s.end
}

/// `bytes` cut to fit a `size`-byte buffer with its NUL — what `xstrlcpy`
/// leaves there.
fn truncated(bytes: &[u8], size: usize) -> &[u8] {
    &bytes[..bytes.len().min(size.saturating_sub(1))]
}

/// Append `bytes` to `buf`, as far as a `size`-byte buffer with its NUL has
/// room — what `xstrlcat` leaves there.
fn push_truncated(buf: &mut Vec<u8>, bytes: &[u8], size: usize) {
    let room = size.saturating_sub(1).saturating_sub(buf.len());
    buf.extend_from_slice(&bytes[..bytes.len().min(room)]);
}

/// The full path of `fname`, made to fit a buffer of `len` bytes.
///
/// `Ok` holds the absolute path — or `fname` unchanged when it is a URL.
/// `Err` holds what upstream's buffer is left with when there is no full
/// path: `fname` itself, cut to fit. `force` asks for the expansion even
/// when `fname` is already absolute.
pub fn vim_full_name(fname: &CStr, len: usize, force: bool) -> Result<Vec<u8>, Vec<u8>> {
    let name = fname.to_bytes();
    if name.len() > len.saturating_sub(1) {
        return Err(truncated(name, len).to_vec());
    }
    if path_with_url(fname) != 0 {
        return Ok(name.to_vec());
    }
    // Something failed: use the file name.
    path_to_absolute(name, len, force).map_err(|Failed| name.to_vec())
}

/// The full resolved path of `fname`, in a string the caller owns.
///
/// A name that looks absolute may still hold a `"dir/../subdir"`, a symlink
/// or a doubled separator; this resolves all of those.
pub fn fix_fname(fname: &CStr) -> XString {
    full_name_save(fname, true)
}

/// The absolute name of the directory `directory` — relative to the current
/// one — made to fit a buffer of `len` bytes.
pub fn path_full_dir_name(directory: &CStr, len: usize) -> Result<Vec<u8>, Failed> {
    if directory.is_empty() {
        return dirname_within(len).map_err(|_| Failed);
    }
    if let Some(real) = realpath(directory) {
        return Ok(truncated(&real, len).to_vec());
    }
    // The path does not exist (yet). An absolute one fails, and the
    // caller uses it as it is.
    if path_is_absolute(directory) {
        return Err(Failed);
    }
    // A relative one is taken from the current directory.
    let old_dir = dirname_within(MAXPATHL as usize).map_err(|_| Failed)?;
    let mut buffer = truncated(&old_dir, len).to_vec();
    append_path(&mut buffer, directory.to_bytes(), len)?;
    Ok(buffer)
}

/// Append `to_append` to `path`, with a separator between them, answering
/// `Err` when a buffer of `max_len` bytes is not enough for the result and
/// its NUL.
pub fn append_path(path: &mut Vec<u8>, to_append: &[u8], max_len: usize) -> Result<(), Failed> {
    // Do not append an empty string, or a dot.
    if to_append.is_empty() || to_append == b"." {
        return Ok(());
    }

    // Join them with a separator, when there is not one there already.
    if path
        .last()
        .is_some_and(|&last| !vim_ispathsep_nocolon(c_int::from(last)))
    {
        // The separator and the NUL at the end.
        if path.len() + PATHSEPSTR.count_bytes() + 1 > max_len {
            return Err(Failed);
        }
        path.extend_from_slice(PATHSEPSTR.to_bytes());
    }

    // The name and the NUL at the end.
    if path.len() + to_append.len() + 1 > max_len {
        return Err(Failed);
    }
    path.extend_from_slice(to_append);
    Ok(())
}

/// The full path of `fname`, made to fit a buffer of `len` bytes. What
/// [`vim_full_name`] and [`fix_fname`] are built on: it resolves the
/// directory part and appends the name to it.
///
/// `force` asks for the expansion even when `fname` is already absolute.
/// `fname` is no longer than `len - 1`.
fn path_to_absolute(fname: &[u8], len: usize, force: bool) -> Result<Vec<u8>, Failed> {
    let mut buf = Vec::new();
    // Where the part appended to the directory starts.
    let mut end_of_path = 0;

    // Expand it if forced, or if it is not an absolute path.
    if force || !matches!(fname.first(), Some(b'/' | b'~')) {
        let mut sep = fname.iter().rposition(|&b| b == b'/');
        if sep.is_none() && fname == b".." {
            // A ".." with no separator in it names a directory too.
            sep = Some(2);
        }
        // What the name is relative to: everything up to and including its
        // last separator.
        let mut relative_directory: &[u8] = b"";
        if let Some(mut at) = sep {
            let at_sep = |at: usize| vim_ispathsep(c_int::from(cstr::byte_at(fname, at)));
            if at_sep(at) && fname[at + 1..] == *b".." {
                // For "/path/dir/.." include the "/..".
                at += 3;
            }
            relative_directory = &fname[..(at + 1).min(fname.len())];
            end_of_path = if at_sep(at) { at + 1 } else { at };
        }
        buf = path_full_dir_name(&cstr::owned(relative_directory), len)?;
    }
    append_path(&mut buf, &fname[end_of_path.min(fname.len())..], len)?;
    Ok(buf)
}

/// Guess the full path Nvim was started from, given the `argv0` it was
/// invoked by, cut to fit a buffer of `bufsize` bytes.
///
/// An absolute name is taken as it is, one holding a separator is taken from
/// the current directory, and a bare name is looked for along `$PATH`.
pub fn path_guess_exepath(argv0: &CStr, bufsize: usize) -> Vec<u8> {
    let name = argv0.to_bytes();
    let path = getenv_owned(c"PATH");
    let Some(path) = path.filter(|_| !path_is_absolute(argv0)) else {
        return truncated(name, bufsize).to_vec();
    };
    if name.first() == Some(&b'.') || name.contains(&b'/') {
        // Relative to the current directory.
        let mut buf = dirname_within(MAXPATHL as usize)
            .map(|dir| truncated(&dir, bufsize).to_vec())
            .unwrap_or_default();
        push_truncated(&mut buf, PATHSEPSTR.to_bytes(), bufsize);
        push_truncated(&mut buf, name, bufsize);
        return buf;
    }
    // Search $PATH for a plausible location.
    for dir in path.split(|&b| b == ENV_SEPCHAR) {
        if dir.is_empty() {
            break;
        }
        if dir.len() < MAXPATHL as usize {
            let mut candidate = dir.to_vec();
            push_truncated(&mut candidate, PATHSEPSTR.to_bytes(), MAXPATHL as usize);
            push_truncated(&mut candidate, name, MAXPATHL as usize);
            if can_execute(&cstr::owned(&candidate), false) {
                return truncated(&candidate, bufsize).to_vec();
            }
        }
    }
    // Not found in $PATH; fall back on argv0.
    truncated(name, bufsize).to_vec()
}
