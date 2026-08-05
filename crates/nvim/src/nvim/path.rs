//! File names: taking them apart, putting them together, and expanding them.
//!
//! The parent holds the two ends of the job. [`simplify_filename`] is the
//! canonicaliser — it removes `.`, `..` and duplicate separators from a name
//! in place, asking the file system before it strips anything that a symlink
//! could make a lie. [`vim_FullName`] and its neighbours are the other end:
//! making a name absolute, which is what everything that compares or stores
//! a name wants first.
//!
//! The children carry the rest: [`names`] the pure text of a name,
//! [`compare`] the comparisons, [`unique`] the shortenings, [`glob`] the
//! file-system walk of one pattern, and [`expand`] the list-level expansion
//! a command line asks for.

#![deny(unsafe_op_in_unsafe_fn)]

use core::ffi::{c_char, c_int};
use std::ffi::CStr;

use crate::src::nvim::charset::{backslash_halve, backslash_halve_save, rem_backslash, skipwhite};
use crate::src::nvim::cmdexpand::globpath;
use crate::src::nvim::eval::eval_to_string;
use crate::src::nvim::ex_docmd::eval_vars;
use crate::src::nvim::fileio::{file_pat_to_reg_pat, match_file_list};
use crate::src::nvim::garray::{
    ga_clear_strings, ga_concat_strings, ga_grow, ga_init, ga_remove_duplicate_strings,
};
use crate::src::nvim::global_cell::GlobalCell;
use crate::src::nvim::main::{
    NameBuff, curbuf, emsg_off, emsg_silent, got_int, p_cdpath, p_fic, p_path, p_su, p_wig,
};
use crate::src::nvim::mbyte::{
    mb_isalpha, mb_strcmp_ic, mb_strnicmp, mb_toupper, utf_head_off, utf_ptr2char, utfc_ptr2len,
};
use crate::src::nvim::memory::{
    xfree, xmalloc, xmemcpyz, xmemdupz, xrealloc, xstrdup, xstrlcat, xstrlcpy,
};
use crate::src::nvim::option::copy_option_part;
use crate::src::nvim::os::env::{expand_env, expand_env_save_opt, os_getenv, vim_env_iter};
use crate::src::nvim::os::fs::{
    os_can_exe, os_closedir, os_dirname, os_file_is_readable, os_fileid, os_fileid_equal,
    os_fileinfo, os_fileinfo_id_equal, os_fileinfo_link, os_isdir, os_path_exists, os_realpath,
    os_scandir, os_scandir_next,
};
use crate::src::nvim::os::input::os_breakcheck;
use crate::src::nvim::os::libc::{qsort, strcasecmp, strchr, strcmp, strcpy, strlen, strncmp};
use crate::src::nvim::os::shell::{get_cmd_output, os_expand_wildcards};
use crate::src::nvim::regexp::{vim_regcomp, vim_regexec, vim_regfree};
use crate::src::nvim::strings::{concat_str, vim_snprintf, vim_strchr};
use crate::src::nvim::types::{
    Directory, FileComparison, FileID, FileInfo, file_comparison, garray_T, regmatch_T, size_t,
};

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

pub type C2Rust_Unnamed_18 = ::core::ffi::c_uint;
pub const WILD_ICASE: C2Rust_Unnamed_18 = 256;
pub const WILD_ADD_SLASH: C2Rust_Unnamed_18 = 16;
pub type C2Rust_Unnamed_19 = ::core::ffi::c_uint;
pub const kShellOptSilent: C2Rust_Unnamed_19 = 8;
pub type C2Rust_Unnamed_20 = ::core::ffi::c_uint;
pub const EW_NOBREAK: C2Rust_Unnamed_20 = 262144;
pub const EW_CDPATH: C2Rust_Unnamed_20 = 131072;
pub const EW_NOTENV: C2Rust_Unnamed_20 = 65536;
pub const EW_EMPTYOK: C2Rust_Unnamed_20 = 32768;
pub const EW_DODOT: C2Rust_Unnamed_20 = 16384;
pub const EW_SHELLCMD: C2Rust_Unnamed_20 = 8192;
pub const EW_ALLLINKS: C2Rust_Unnamed_20 = 4096;
pub const EW_KEEPDOLLAR: C2Rust_Unnamed_20 = 2048;
pub const EW_NOTWILD: C2Rust_Unnamed_20 = 1024;
pub const EW_NOERROR: C2Rust_Unnamed_20 = 512;
pub const EW_ICASE: C2Rust_Unnamed_20 = 256;
pub const EW_PATH: C2Rust_Unnamed_20 = 128;
pub const EW_EXEC: C2Rust_Unnamed_20 = 64;
pub const EW_SILENT: C2Rust_Unnamed_20 = 32;
pub const EW_KEEPALL: C2Rust_Unnamed_20 = 16;
pub const EW_ADDSLASH: C2Rust_Unnamed_20 = 8;
pub const EW_NOTFOUND: C2Rust_Unnamed_20 = 4;
pub const EW_FILE: C2Rust_Unnamed_20 = 2;
pub const EW_DIR: C2Rust_Unnamed_20 = 1;
pub const kEqualFileNames: file_comparison = 7;
pub const kOneFileMissing: file_comparison = 6;
pub const kBothFilesMissing: file_comparison = 4;
pub const kDifferentFiles: file_comparison = 2;
pub const kEqualFiles: file_comparison = 1;
pub const URL_BACKSLASH: C2Rust_Unnamed_21 = 2;
pub const URL_SLASH: C2Rust_Unnamed_21 = 1;
pub type C2Rust_Unnamed_21 = ::core::ffi::c_uint;
pub const MAXPATHL: c_int = 4096;
pub const NUL: c_int = 0;
pub const PATHSEP: c_int = '/' as c_int;
pub const PATHSEPSTR: [c_char; 2] = [b'/' as c_char, 0];
pub const OK: c_int = 1;
pub const FAIL: c_int = 0;
pub const MAXSUFLEN: c_int = 30;
pub const ENV_SEPCHAR: c_int = ':' as c_int;

/// The full path of `fname`, in a string the caller owns.
///
/// Answers a copy of `fname` itself when it cannot be made absolute, and
/// NULL only for a NULL `fname`.
///
/// # Safety
/// `fname` must be a NUL-terminated string, or NULL.
pub unsafe fn FullName_save(fname: *const c_char, force: bool) -> *mut c_char {
    unsafe {
        if fname.is_null() {
            return core::ptr::null_mut();
        }
        let buf: *mut c_char = xmalloc(MAXPATHL as size_t).cast();
        if vim_FullName(fname, buf, MAXPATHL as size_t, force) == FAIL {
            xfree(buf.cast());
            return xstrdup(fname);
        }
        buf
    }
}

/// [`FullName_save`] for a name that may already be absolute, in which case
/// it is only copied.
///
/// # Safety
/// `name` must be a NUL-terminated string.
pub unsafe fn save_abs_path(name: *const c_char) -> *mut c_char {
    unsafe {
        if path_is_absolute(name) {
            xstrdup(name)
        } else {
            FullName_save(name, true)
        }
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
        unsafe {
            while vim_ispathsep(self.name[at] as c_int) {
                // A separator with a composing character after it is one
                // character, and upstream steps over both.
                at += utfc_ptr2len(self.name.as_ptr().add(at).cast()) as usize;
            }
            at
        }
    }

    /// Handle the `".."` at `p`, answering where to carry on from.
    ///
    /// # Safety
    /// `p` must index the `".."`.
    unsafe fn strip_parent(&mut self, mut p: usize) -> usize {
        unsafe {
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
    }

    /// May the component before `p` be stripped by a `".."`? Moves `p` back
    /// to the start of that component either way.
    ///
    /// # Safety
    /// `p` must index a `".."` that a component and a separator precede, and
    /// `tail` must index past it.
    unsafe fn can_strip(&mut self, p: &mut usize, tail: usize) -> bool {
        unsafe {
            if self.stripping_disabled {
                return false;
            }
            let filename: *mut c_char = self.name.as_mut_ptr().cast();
            let mut file_info = FileInfo::default();

            // A component that does not exist is stripped without further
            // thought — and a symlink to a name that does not exist counts
            // as not existing.
            let exists =
                self.terminated_at(*p - 1, |name| os_fileinfo_link(name, &raw mut file_info));

            // Back to the start of the component being stripped.
            *p -= 1;
            while *p > self.start && after_pathsep(filename, filename.add(*p)) == 0 {
                // MB_PTR_BACK: to the head of the character before this one.
                *p -= utf_head_off(filename, filename.add(*p - 1)) as usize + 1;
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
            if !self.terminated_at(tail, |name| os_fileinfo(name, &raw mut file_info)) {
                self.stripping_disabled = true;
                return false;
            }

            // That test passes for a symlink to a searchable directory too,
            // and then the directory's parent must be the same file as the
            // stripped name — which does exist, being the component's own
            // parent.
            let mut new_file_info = FileInfo::default();
            if *p == self.start && self.relative {
                os_fileinfo(c".".as_ptr(), &raw mut new_file_info);
            } else {
                self.terminated_at(*p, |name| os_fileinfo(name, &raw mut new_file_info));
            }
            os_fileinfo_id_equal(&raw mut file_info, &raw mut new_file_info)
        }
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
    unsafe {
        let len = CStr::from_ptr(filename).to_bytes().len();
        let mut s = Simplify {
            name: core::slice::from_raw_parts_mut(filename.cast::<u8>(), len + 1),
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
            } else if s.name[p] == b'.'
                && (vim_ispathsep(s.name[p + 1] as c_int) || s.name[p + 1] == 0)
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
                        tail = s.past_separators(tail);
                    } else if p > s.start {
                        p -= 1;
                    }
                    s.remove(p, tail);
                }
            } else if s.name[p] == b'.'
                && s.name[p + 1] == b'.'
                && (vim_ispathsep(s.name[p + 2] as c_int) || s.name[p + 2] == 0)
            {
                p = s.strip_parent(p);
            } else {
                // A simple path component: on past the separator after it.
                // Everything here works through `s.name`; `filename` itself
                // is not touched again, so the borrow stays whole.
                s.components += 1;
                let base = s.name.as_ptr().cast::<c_char>();
                p = path_next_component(base.add(p)).offset_from(base) as usize;
            }
            if s.name[p] == 0 {
                break;
            }
        }
        s.end as size_t
    }
}

/// Put the full path of `fname` in `buf`, which holds `len` bytes.
///
/// `buf` gets `fname` truncated when it does not fit, `fname` unchanged when
/// it is a URL or cannot be made absolute, and the absolute path otherwise.
/// `force` asks for the expansion even when `fname` is already absolute.
///
/// Answers FAIL when `buf` holds anything but a full path.
///
/// # Safety
/// `buf` must be writable for `len` bytes; `fname` must be a NUL-terminated
/// string, or NULL.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn vim_FullName(
    fname: *const c_char,
    buf: *mut c_char,
    len: size_t,
    force: bool,
) -> c_int {
    unsafe {
        *buf = 0;
        if fname.is_null() {
            return FAIL;
        }
        if strlen(fname) > len.wrapping_sub(1) {
            xstrlcpy(buf, fname, len); // truncate
            return FAIL;
        }
        if path_with_url(fname) != 0 {
            xstrlcpy(buf, fname, len);
            return OK;
        }
        let rv = path_to_absolute(fname, buf, len, force);
        if rv == FAIL {
            xstrlcpy(buf, fname, len); // something failed; use the file name
        }
        rv
    }
}

/// The full resolved path of `fname`, in a string the caller owns.
///
/// A name that looks absolute may still hold a `"dir/../subdir"`, a symlink
/// or a doubled separator; this resolves all of those.
///
/// # Safety
/// `fname` must be a NUL-terminated string, or NULL.
pub unsafe fn fix_fname(fname: *const c_char) -> *mut c_char {
    unsafe { FullName_save(fname, true) }
}

/// Put the absolute name of the directory `directory` — relative to the
/// current one — in `buffer`, which holds `len` bytes.
///
/// # Safety
/// `directory` must be a NUL-terminated string and `buffer` writable for
/// `len` bytes.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn path_full_dir_name(
    directory: *mut c_char,
    buffer: *mut c_char,
    len: size_t,
) -> c_int {
    unsafe {
        if *directory == 0 {
            return os_dirname(buffer, len);
        }
        if !os_realpath(directory, buffer, len).is_null() {
            return OK;
        }
        // The path does not exist (yet). An absolute one fails, and the
        // caller uses it as it is.
        if path_is_absolute(directory) {
            return FAIL;
        }
        // A relative one is taken from the current directory.
        let mut old_dir = [0 as c_char; MAXPATHL as usize];
        if os_dirname(old_dir.as_mut_ptr(), MAXPATHL as size_t) == FAIL {
            return FAIL;
        }
        xstrlcpy(buffer, old_dir.as_ptr(), len);
        append_path(buffer, directory, len)
    }
}

/// Append `to_append` to `path`, with a separator between them, answering
/// FAIL when `max_len` bytes are not enough for the result.
///
/// # Safety
/// `path` must be a NUL-terminated string writable for `max_len` bytes, and
/// `to_append` a NUL-terminated string.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn append_path(
    path: *mut c_char,
    to_append: *const c_char,
    max_len: size_t,
) -> c_int {
    unsafe {
        let mut current_length = CStr::from_ptr(path).to_bytes().len();
        let to_append_length = CStr::from_ptr(to_append).to_bytes().len();
        let max_len = max_len as usize;
        // The separator, without its NUL.
        let sep_len = PATHSEPSTR.len() - 1;

        // Do not append an empty string, or a dot.
        if to_append_length == 0 || strcmp(to_append, c".".as_ptr()) == 0 {
            return OK;
        }

        // Join them with a separator, when there is not one there already.
        if current_length > 0 && !vim_ispathsep_nocolon(*path.add(current_length - 1) as c_int) {
            // The separator and the NUL at the end.
            if current_length + sep_len + 1 > max_len {
                return FAIL;
            }
            xstrlcpy(
                path.add(current_length),
                PATHSEPSTR.as_ptr(),
                (max_len - current_length) as size_t,
            );
            current_length += sep_len;
        }

        // The name and the NUL at the end.
        if current_length + to_append_length + 1 > max_len {
            return FAIL;
        }
        xstrlcpy(
            path.add(current_length),
            to_append,
            (max_len - current_length) as size_t,
        );
        OK
    }
}

/// Put the full path of `fname` in `buf`, which holds `len` bytes. What
/// [`vim_FullName`] and [`fix_fname`] are built on: it resolves the
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
) -> c_int {
    unsafe {
        *buf = 0;
        let name = CStr::from_ptr(fname).to_bytes();
        // What the name is relative to: everything up to and including its
        // last separator. One byte longer than upstream's, which writes its
        // NUL one past the end for a name of exactly `len - 1` bytes ending
        // in "/..".
        let mut relative_directory = vec![0 as c_char; len as usize + 1];
        let mut end_of_path = fname;

        // Expand it if forced, or if it is not an absolute path.
        if force || !path_is_absolute(fname) {
            let mut sep = name.iter().rposition(|&b| b == b'/');
            if sep.is_none() && name == b".." {
                // A ".." with no separator in it names a directory too.
                sep = Some(2);
            }
            if let Some(mut at) = sep {
                if vim_ispathsep(*fname.add(at) as c_int) && name[at + 1..] == *b".." {
                    // For "/path/dir/.." include the "/..".
                    at += 3;
                }
                core::ptr::copy_nonoverlapping(fname, relative_directory.as_mut_ptr(), at + 1);
                relative_directory[at + 1] = 0;
                end_of_path = if vim_ispathsep(*fname.add(at) as c_int) {
                    fname.add(at + 1)
                } else {
                    fname.add(at)
                };
            }

            if path_full_dir_name(relative_directory.as_mut_ptr(), buf, len) == FAIL {
                return FAIL;
            }
        }
        append_path(buf, end_of_path, len)
    }
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
#[unsafe(no_mangle)]
pub unsafe extern "C" fn path_guess_exepath(
    argv0: *const c_char,
    buf: *mut c_char,
    bufsize: size_t,
) {
    unsafe {
        let path = os_getenv(c"PATH".as_ptr());
        if path.is_null() || path_is_absolute(argv0) {
            xstrlcpy(buf, argv0, bufsize);
        } else if *argv0 == b'.' as c_char || !strchr(argv0, PATHSEP).is_null() {
            // Relative to the current directory.
            if os_dirname(buf, MAXPATHL as size_t) != OK {
                *buf = 0;
            }
            xstrlcat(buf, PATHSEPSTR.as_ptr(), bufsize);
            xstrlcat(buf, argv0, bufsize);
        } else {
            // Search $PATH for a plausible location.
            let name = NameBuff.ptr().cast::<c_char>();
            let size = size_of::<[c_char; MAXPATHL as usize]>();
            let mut iter: *const core::ffi::c_void = core::ptr::null();
            loop {
                let mut dir: *const c_char = core::ptr::null();
                let mut dir_len: size_t = 0;
                iter = vim_env_iter(
                    ENV_SEPCHAR as c_char,
                    path,
                    iter,
                    &raw mut dir,
                    &raw mut dir_len,
                );
                if dir.is_null() || dir_len == 0 {
                    break;
                }
                if dir_len as usize + 1 <= size {
                    xmemcpyz(name.cast(), dir.cast(), dir_len);
                    xstrlcat(name, PATHSEPSTR.as_ptr(), size);
                    xstrlcat(name, argv0, size);
                    if os_can_exe(name, core::ptr::null_mut(), false) {
                        xstrlcpy(buf, name, bufsize);
                        return;
                    }
                }
                if iter.is_null() {
                    break;
                }
            }
            // Not found in $PATH; fall back on argv0.
            xstrlcpy(buf, argv0, bufsize);
        }
        xfree(path.cast());
    }
}
