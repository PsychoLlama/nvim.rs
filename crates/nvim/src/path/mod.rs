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

#![deny(unsafe_op_in_unsafe_fn)]

use crate::cstr;
use core::ffi::{c_char, c_int};
use std::ffi::CStr;

use crate::charset::{backslash_halve, backslash_halve_save, rem_backslash, skipwhite};
use crate::cmdexpand::globpath;
use crate::eval::eval_to_string;
use crate::ex_docmd::eval_vars;
use crate::fileio::{file_pat_to_reg_pat, match_file_list};
use crate::garray::{
    ga_clear_strings, ga_concat_strings, ga_grow, ga_init, ga_remove_duplicate_strings,
};
use crate::global_cell::GlobalCell;
use crate::main::{curbuf, emsg_silent, got_int, p_cdpath, p_fic, p_path, p_su, p_wig};
use crate::mbyte::{
    mb_isalpha, mb_strcmp_ic, mb_strnicmp, mb_toupper, utf_head_off, utf_ptr2char, utfc_ptr2len,
};
use crate::memory::{xfree, xmalloc, xmemcpyz, xmemdupz, xrealloc, xstrdup, xstrlcat, xstrlcpy};
use crate::option::copy_option_part;
use crate::os::cshim::strchr;
use crate::os::env::{expand_env, expand_env_save_opt, os_getenv, vim_env_iter};
use crate::os::fs::{
    os_can_exe, os_closedir, os_dirname, os_file_is_readable, os_fileid, os_fileid_equal,
    os_fileinfo, os_fileinfo_id_equal, os_fileinfo_link, os_isdir, os_path_exists, os_realpath,
    os_scandir, os_scandir_next,
};
use crate::os::input::os_breakcheck;
use crate::os::shell::{get_cmd_output, os_expand_wildcards};
use crate::regexp::{vim_regcomp, vim_regexec, vim_regfree};
use crate::strings::{concat_str, vim_snprintf, vim_strchr};
use crate::types::{
    Directory, Failed, FileComparison, FileID, FileInfo, MAXPATHL, PATHSEPSTR, file_comparison,
    garray_T, regmatch_T, size_t,
};
use ::libc::{qsort, strcasecmp, strcpy};

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
pub const kEqualFileNames: file_comparison = 7;
pub const kOneFileMissing: file_comparison = 6;
pub const kBothFilesMissing: file_comparison = 4;
pub const kDifferentFiles: file_comparison = 2;
pub const kEqualFiles: file_comparison = 1;
pub const URL_BACKSLASH: ::core::ffi::c_uint = 2;
pub const URL_SLASH: ::core::ffi::c_uint = 1;
pub const PATHSEP: c_int = '/' as c_int;
pub const MAXSUFLEN: c_int = 30;
pub const ENV_SEPCHAR: c_int = ':' as c_int;

/// The full path of `fname`, in a string the caller owns.
///
/// Answers a copy of `fname` itself when it cannot be made absolute, and
/// NULL only for a NULL `fname`.
///
/// # Safety
/// `fname` must be a NUL-terminated string, or NULL.
pub unsafe fn full_name_save(fname: *const c_char, force: bool) -> *mut c_char {
    if fname.is_null() {
        return core::ptr::null_mut();
    }
    let buf: *mut c_char = unsafe { xmalloc(MAXPATHL as size_t) }.cast();
    if unsafe { vim_full_name(fname, buf, MAXPATHL as size_t, force) }.is_err() {
        unsafe { xfree(buf.cast()) };
        return unsafe { xstrdup(fname) };
    }
    buf
}

/// [`full_name_save`] for a name that may already be absolute, in which case
/// it is only copied.
///
/// # Safety
/// `name` must be a NUL-terminated string.
pub unsafe fn save_abs_path(name: *const c_char) -> *mut c_char {
    if unsafe { path_is_absolute(name) } {
        unsafe { xstrdup(name) }
    } else {
        unsafe { full_name_save(name, true) }
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
    /// # Safety
    /// `at` must index into the name.
    unsafe fn past_separators(&self, mut at: usize) -> usize {
        while vim_ispathsep(self.name[at] as c_int) {
            // A separator with a composing character after it is one
            // character, and upstream steps over both.
            at += unsafe { utfc_ptr2len(self.name.as_ptr().add(at).cast()) } as usize;
        }
        at
    }

    /// Handle the `".."` at `p`, answering where to carry on from.
    ///
    /// # Safety
    /// `p` must index the `".."`.
    unsafe fn strip_parent(&mut self, mut p: usize) -> usize {
        // Past the ".." and any separators after it.
        let mut tail = unsafe { self.past_separators(p + 2) };

        if self.components > 0 {
            if unsafe { self.can_strip(&mut p, tail) } {
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

    /// May the component before `p` be stripped by a `".."`? Moves `p` back
    /// to the start of that component either way.
    ///
    /// # Safety
    /// `p` must index a `".."` that a component and a separator precede, and
    /// `tail` must index past it.
    unsafe fn can_strip(&mut self, p: &mut usize, tail: usize) -> bool {
        if self.stripping_disabled {
            return false;
        }
        let filename: *mut c_char = self.name.as_mut_ptr().cast();
        let mut file_info = FileInfo::default();

        // A component that does not exist is stripped without further
        // thought — and a symlink to a name that does not exist counts
        // as not existing.
        let exists = self.terminated_at(*p - 1, |name| unsafe {
            os_fileinfo_link(name, &raw mut file_info)
        });

        // Back to the start of the component being stripped.
        *p -= 1;
        while *p > self.start && unsafe { after_pathsep(filename, filename.add(*p)) } == 0 {
            // MB_PTR_BACK: to the head of the character before this one.
            *p -= unsafe { utf_head_off(filename, filename.add(*p - 1)) } as usize + 1;
        }
        if !exists {
            return true;
        }

        // The component does exist. Stripping it may still change what
        // the name means, so ask the file system about the unstripped
        // name. That can fail when the component is not a searchable
        // directory — a regular file, say — since the trailing "/.."
        // cannot be applied then; the name is wrong, and later
        // components are left alone too.
        if !self.terminated_at(tail, |name| unsafe {
            os_fileinfo(name, &raw mut file_info)
        }) {
            self.stripping_disabled = true;
            return false;
        }

        // That test passes for a symlink to a searchable directory too,
        // and then the directory's parent must be the same file as the
        // stripped name — which does exist, being the component's own
        // parent.
        let mut new_file_info = FileInfo::default();
        if *p == self.start && self.relative {
            unsafe { os_fileinfo(c".".as_ptr(), &raw mut new_file_info) };
        } else {
            self.terminated_at(*p, |name| unsafe {
                os_fileinfo(name, &raw mut new_file_info)
            });
        }
        unsafe { os_fileinfo_id_equal(&raw mut file_info, &raw mut new_file_info) }
    }

    /// Ask `question` about the name cut short at `at`, which is put back
    /// afterwards.
    ///
    /// # Safety
    /// `at` must index into the name.
    fn terminated_at(&mut self, at: usize, question: impl FnOnce(*const c_char) -> bool) -> bool {
        let saved = core::mem::replace(&mut self.name[at], 0);
        let answer = question(self.name.as_ptr().cast());
        self.name[at] = saved;
        answer
    }
}

/// Reduce `filename` to its simplest form, in place: no `"."` components, no
/// duplicate separators, and no `".."` that the file system agrees may come
/// off. The result is never longer than what it started as.
///
/// Answers how long it is now.
///
/// # Safety
/// `filename` must be a writable NUL-terminated string.
pub unsafe fn simplify_filename(filename: *mut c_char) -> size_t {
    let len = unsafe { CStr::from_ptr(filename) }.to_bytes().len();
    let mut s = Simplify {
        name: unsafe { core::slice::from_raw_parts_mut(filename.cast::<u8>(), len + 1) },
        end: len,
        start: 0,
        relative: true,
        components: 0,
        stripping_disabled: false,
    };

    let mut p = 0;
    if vim_ispathsep(s.name[0] as c_int) {
        s.relative = false;
        while vim_ispathsep(s.name[p] as c_int) {
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
        if vim_ispathsep(s.name[p] as c_int) {
            // A duplicate separator.
            s.remove(p, p + 1);
        } else if s.name[p] == b'.' && (vim_ispathsep(s.name[p + 1] as c_int) || s.name[p + 1] == 0)
        {
            if p == s.start && s.relative {
                // Keep a single "." or a leading "./".
                p += 1 + usize::from(s.name[p + 1] != 0);
            } else {
                // Strip "./" or ".///". At the end of the name, with no
                // trailing separator, strip the "/." after the start —
                // or the "." alone at the start of an absolute name.
                let mut tail = p + 1;
                if s.name[p + 1] != 0 {
                    tail = unsafe { s.past_separators(tail) };
                } else if p > s.start {
                    p -= 1;
                }
                s.remove(p, tail);
            }
        } else if s.name[p] == b'.'
            && s.name[p + 1] == b'.'
            && (vim_ispathsep(s.name[p + 2] as c_int) || s.name[p + 2] == 0)
        {
            p = unsafe { s.strip_parent(p) };
        } else {
            // A simple path component: on past the separator after it.
            // Everything here works through `s.name`; `filename` itself
            // is not touched again, so the borrow stays whole.
            s.components += 1;
            let base = s.name.as_ptr().cast::<c_char>();
            // SAFETY: `p` is within `s.name`, and `path_next_component`
            // answers a pointer into the same name.
            p = unsafe { path_next_component(base.add(p)).offset_from(base) } as usize;
        }
        if s.name[p] == 0 {
            break;
        }
    }
    s.end as size_t
}

/// Put the full path of `fname` in `buf`, which holds `len` bytes.
///
/// `buf` gets `fname` truncated when it does not fit, `fname` unchanged when
/// it is a URL or cannot be made absolute, and the absolute path otherwise.
/// `force` asks for the expansion even when `fname` is already absolute.
///
/// Answers `Err` when `buf` holds anything but a full path.
///
/// # Safety
/// `buf` must be writable for `len` bytes; `fname` must be a NUL-terminated
/// string, or NULL.
pub unsafe fn vim_full_name(
    fname: *const c_char,
    buf: *mut c_char,
    len: size_t,
    force: bool,
) -> Result<(), Failed> {
    unsafe { *buf = 0 };
    if fname.is_null() {
        return Err(Failed);
    }
    if unsafe { cstr::bytes_at(fname) }.len() > len.wrapping_sub(1) {
        unsafe { xstrlcpy(buf, fname, len) }; // truncate
        return Err(Failed);
    }
    if unsafe { path_with_url(fname) } != 0 {
        unsafe { xstrlcpy(buf, fname, len) };
        return Ok(());
    }
    let rv = unsafe { path_to_absolute(fname, buf, len, force) };
    if rv.is_err() {
        unsafe { xstrlcpy(buf, fname, len) }; // something failed; use the file name
    }
    rv
}

/// The full resolved path of `fname`, in a string the caller owns.
///
/// A name that looks absolute may still hold a `"dir/../subdir"`, a symlink
/// or a doubled separator; this resolves all of those.
///
/// # Safety
/// `fname` must be a NUL-terminated string, or NULL.
pub unsafe fn fix_fname(fname: *const c_char) -> *mut c_char {
    unsafe { full_name_save(fname, true) }
}

/// Put the absolute name of the directory `directory` — relative to the
/// current one — in `buffer`, which holds `len` bytes.
///
/// # Safety
/// `directory` must be a NUL-terminated string and `buffer` writable for
/// `len` bytes.
pub unsafe fn path_full_dir_name(
    directory: *mut c_char,
    buffer: *mut c_char,
    len: size_t,
) -> Result<(), Failed> {
    if unsafe { *directory } == 0 {
        return unsafe { os_dirname(buffer, len) };
    }
    if !unsafe { os_realpath(directory, buffer, len) }.is_null() {
        return Ok(());
    }
    // The path does not exist (yet). An absolute one fails, and the
    // caller uses it as it is.
    if unsafe { path_is_absolute(directory) } {
        return Err(Failed);
    }
    // A relative one is taken from the current directory.
    let mut old_dir = [0 as c_char; MAXPATHL as usize];
    if unsafe { os_dirname(old_dir.as_mut_ptr(), MAXPATHL as size_t) }.is_err() {
        return Err(Failed);
    }
    unsafe { xstrlcpy(buffer, old_dir.as_ptr(), len) };
    unsafe { append_path(buffer, directory, len) }
}

/// Append `to_append` to `path`, with a separator between them, answering
/// `Err` when `max_len` bytes are not enough for the result.
///
/// # Safety
/// `path` must be a NUL-terminated string writable for `max_len` bytes, and
/// `to_append` a NUL-terminated string.
pub unsafe fn append_path(
    path: *mut c_char,
    to_append: *const c_char,
    max_len: size_t,
) -> Result<(), Failed> {
    let mut current_length = unsafe { CStr::from_ptr(path) }.to_bytes().len();
    let to_append_length = unsafe { CStr::from_ptr(to_append) }.to_bytes().len();
    // The separator, without its NUL.
    let sep_len = PATHSEPSTR.count_bytes();

    // Do not append an empty string, or a dot.
    if to_append_length == 0 || unsafe { cstr::bytes_at(to_append) == b"." } {
        return Ok(());
    }

    // Join them with a separator, when there is not one there already.
    if current_length > 0
        && !vim_ispathsep_nocolon(unsafe { *path.add(current_length - 1) } as c_int)
    {
        // The separator and the NUL at the end.
        if current_length + sep_len + 1 > max_len {
            return Err(Failed);
        }
        unsafe {
            xstrlcpy(
                path.add(current_length),
                PATHSEPSTR.as_ptr(),
                (max_len - current_length) as size_t,
            )
        };
        current_length += sep_len;
    }

    // The name and the NUL at the end.
    if current_length + to_append_length + 1 > max_len {
        return Err(Failed);
    }
    unsafe {
        xstrlcpy(
            path.add(current_length),
            to_append,
            (max_len - current_length) as size_t,
        )
    };
    Ok(())
}

/// Put the full path of `fname` in `buf`, which holds `len` bytes. What
/// [`vim_full_name`] and [`fix_fname`] are built on: it resolves the
/// directory part and appends the name to it.
///
/// `force` asks for the expansion even when `fname` is already absolute.
///
/// # Safety
/// `fname` must be a NUL-terminated string no longer than `len - 1`, and
/// `buf` writable for `len` bytes.
unsafe fn path_to_absolute(
    fname: *const c_char,
    buf: *mut c_char,
    len: size_t,
    force: bool,
) -> Result<(), Failed> {
    unsafe { *buf = 0 };
    let name = unsafe { CStr::from_ptr(fname) }.to_bytes();
    // What the name is relative to: everything up to and including its
    // last separator. One byte longer than upstream's, which writes its
    // NUL one past the end for a name of exactly `len - 1` bytes ending
    // in "/..".
    let mut relative_directory = vec![0 as c_char; len + 1];
    let mut end_of_path = fname;

    // Expand it if forced, or if it is not an absolute path.
    if force || !unsafe { path_is_absolute(fname) } {
        let mut sep = name.iter().rposition(|&b| b == b'/');
        if sep.is_none() && name == b".." {
            // A ".." with no separator in it names a directory too.
            sep = Some(2);
        }
        if let Some(mut at) = sep {
            if vim_ispathsep(unsafe { *fname.add(at) } as c_int) && name[at + 1..] == *b".." {
                // For "/path/dir/.." include the "/..".
                at += 3;
            }
            unsafe {
                core::ptr::copy_nonoverlapping(fname, relative_directory.as_mut_ptr(), at + 1)
            };
            relative_directory[at + 1] = 0;
            end_of_path = if vim_ispathsep(unsafe { *fname.add(at) } as c_int) {
                unsafe { fname.add(at + 1) }
            } else {
                unsafe { fname.add(at) }
            };
        }

        if unsafe { path_full_dir_name(relative_directory.as_mut_ptr(), buf, len) }.is_err() {
            return Err(Failed);
        }
    }
    unsafe { append_path(buf, end_of_path, len) }
}

/// Guess the full path Nvim was started from, given the `argv0` it was
/// invoked by, and put it in `buf`.
///
/// An absolute name is taken as it is, one holding a separator is taken from
/// the current directory, and a bare name is looked for along `$PATH`.
///
/// # Safety
/// `argv0` must be a NUL-terminated string and `buf` writable for `bufsize`
/// bytes.
pub unsafe fn path_guess_exepath(argv0: *const c_char, buf: *mut c_char, bufsize: size_t) {
    let mut candidate = [0 as c_char; MAXPATHL as usize];
    let path = unsafe { os_getenv(c"PATH".as_ptr()) };
    if path.is_null() || unsafe { path_is_absolute(argv0) } {
        unsafe { xstrlcpy(buf, argv0, bufsize) };
    } else if unsafe { *argv0 } == b'.' as c_char || !unsafe { strchr(argv0, PATHSEP) }.is_null() {
        // Relative to the current directory.
        if unsafe { os_dirname(buf, MAXPATHL as size_t) }.is_err() {
            unsafe { *buf = 0 };
        }
        unsafe { xstrlcat(buf, PATHSEPSTR.as_ptr(), bufsize) };
        unsafe { xstrlcat(buf, argv0, bufsize) };
    } else {
        // Search $PATH for a plausible location.
        let name = candidate.as_mut_ptr();
        let size = candidate.len();
        let mut iter: *const core::ffi::c_void = core::ptr::null();
        loop {
            let mut dir: *const c_char = core::ptr::null();
            let mut dir_len: size_t = 0;
            iter = unsafe {
                vim_env_iter(
                    ENV_SEPCHAR as c_char,
                    path,
                    iter,
                    &raw mut dir,
                    &raw mut dir_len,
                )
            };
            if dir.is_null() || dir_len == 0 {
                break;
            }
            if (dir_len as usize) < size {
                unsafe { xmemcpyz(name.cast(), dir.cast(), dir_len) };
                unsafe { xstrlcat(name, PATHSEPSTR.as_ptr(), size) };
                unsafe { xstrlcat(name, argv0, size) };
                if unsafe { os_can_exe(name, core::ptr::null_mut(), false) } {
                    unsafe { xstrlcpy(buf, name, bufsize) };
                    return;
                }
            }
            if iter.is_null() {
                break;
            }
        }
        // Not found in $PATH; fall back on argv0.
        unsafe { xstrlcpy(buf, argv0, bufsize) };
    }
    unsafe { xfree(path.cast()) };
}
