//! Canonicalising a path -- `resolve()`, `simplify()`, `pathshorten()`,
//! `glob2regpat()` and `isabsolutepath()`.
//!
//! These are the pure-ish string transforms over a path: `f_resolve` is the
//! only one that reads the filesystem, following a symlink chain (with its own
//! loop guard) until it reaches something that is not a link; `f_simplify`
//! collapses `.`/`..`/duplicate separators without looking at the disk;
//! `f_pathshorten` reduces every leading component to its first character;
//! `f_glob2regpat` translates a wildcard pattern into the regex the search
//! engine wants.
//!
//! # How the pointers are held
//!
//! `resolve()` juggles three heap strings across a loop with an early exit --
//! the name resolved so far, the part of the argument still to be appended,
//! and the `readlink` scratch -- which upstream frees by hand before each
//! `return`.  [`Owned`] owns one of them and frees it on the way out, so the
//! whole body is ordinary control flow; every offset below is a byte index
//! into one of those strings rather than a pointer into it, and the C's reads
//! one past a component are the terminator, which [`at`] answers as 0.
//!
//! Original: `src/nvim/eval/fs.c`, Vim/Neovim, Vim license.

#![deny(unsafe_op_in_unsafe_fn)]

use super::{MAXPATHL, Owned, at, err, frame, from, is_sep, ret_string, str_arg, str_arg_chk};
use crate::eval::typval::tv_get_number;
use crate::fileio::file_pat_to_reg_pat;
use crate::memory::{xrealloc, xstrlcat};
use crate::os::cshim::memmove;
use crate::path::{
    add_pathsep, after_pathsep, path_is_absolute, path_next_component, path_tail,
    path_tail_with_sep, shorten_dir_len, simplify_filename,
};
use crate::types::{EvalFuncData, VAR_STRING, size_t, typval_T, varnumber_T};
use ::libc::{memcpy, readlink};
use core::ffi::{CStr, c_char, c_int};
use core::ptr;

// ---------------------------------------------------------------------
// A heap string, and the byte arithmetic over one
// ---------------------------------------------------------------------

/// The three shapes of allocation `resolve()` does to the name it is
/// building, on top of the family's [`Owned`].
impl Owned {
    /// Drop the first `n` bytes, sliding the rest -- and the terminator --
    /// down to the front.  Upstream's `STRMOVE`.
    fn drop_front(&self, n: usize) {
        let rest = self.len() - n + 1;
        // SAFETY: source and destination are both inside the allocation and
        // the two overlap, which is what `memmove` is for.
        unsafe { memmove(self.0.cast(), self.0.add(n).cast(), rest as size_t) };
    }

    /// Replace the last component with `name`, growing the allocation.
    ///
    /// The head that is kept plus `name` and its terminator is what the new
    /// size covers; upstream asks for the whole of both, which is never less.
    fn replace_tail(&mut self, name: &CStr) {
        let (len, name_len) = (self.len(), name.to_bytes().len());
        // SAFETY: the pointer came from the same allocator, and the new size
        // covers the head that is kept plus `name` and its terminator.
        self.0 = unsafe { xrealloc(self.0.cast(), (len + name_len + 1) as size_t).cast() };
        // SAFETY: `path_tail` answers a pointer inside the block just grown,
        // at most `len` bytes in, so `name` and its terminator fit after it.
        unsafe { memcpy(path_tail(self.0).cast(), name.as_ptr().cast(), name_len + 1) };
    }

    /// `self` with the first `n` bytes of `tail` appended.
    fn with_suffix(&self, tail: &CStr, n: usize) -> Owned {
        let len = self.len();
        let out = Owned::zeroed(len + n);
        // SAFETY: `out` holds `len + n` bytes and a terminator; the copy is
        // `self` and its NUL, and `xstrlcat` then writes at most `n` bytes
        // and a NUL of its own after them.
        unsafe {
            memcpy(out.0.cast(), self.0.cast(), len + 1);
            xstrlcat(out.0.add(len), tail.as_ptr(), (n + 1) as size_t);
        }
        out
    }
}

/// The [`MAXPATHL`] scratch `readlink` writes a link's value into.
struct LinkBuf(Owned);

impl LinkBuf {
    fn new() -> Self {
        Self(Owned::zeroed(MAXPATHL as usize))
    }

    /// Read the link `p` names into the scratch; false when `p` is not one.
    fn read(&self, p: &CStr) -> bool {
        // SAFETY: `p` is NUL-terminated and the scratch holds `MAXPATHL`
        // writable bytes plus the terminator slot the NUL below goes into.
        let len = unsafe { readlink(p.as_ptr(), self.0.0, MAXPATHL as size_t) };
        if len <= 0 {
            return false;
        }
        self.0.set(len as usize, 0);
        true
    }

    /// Append a path separator, so that a resolved directory keeps the one
    /// the argument had.
    fn add_pathsep(&self) {
        // SAFETY: the scratch holds `MAXPATHL` bytes and `readlink` filled at
        // most `MAXPATHL` of them, so there is room for one more.
        unsafe { add_pathsep(self.0.0) };
    }

    fn cstr<'a>(&self) -> &'a CStr {
        self.0.cstr()
    }
}

/// Where the component after the one at `at` starts: past its separator, or
/// at the terminator when there is no separator left.
fn next_component(s: &CStr, at: usize) -> usize {
    // SAFETY: `at` indexes `s`, so the argument is inside the same
    // NUL-terminated string and so is the answer.
    unsafe { path_next_component(s.as_ptr().add(at)).offset_from(s.as_ptr()) as usize }
}

/// Where the last component of `s` starts.
fn tail(s: &CStr) -> usize {
    // SAFETY: `s` is NUL-terminated, and the answer is inside it.
    unsafe { path_tail(s.as_ptr()).offset_from(s.as_ptr()) as usize }
}

/// Where the separators before the last component start.
fn tail_with_sep(s: &CStr) -> usize {
    // SAFETY: as [`tail`]; the cast is `path_tail_with_sep`'s `char *`
    // parameter, which it only reads.
    unsafe { path_tail_with_sep(s.as_ptr().cast_mut()).offset_from(s.as_ptr()) as usize }
}

/// Whether byte `at` of `s` follows a path separator -- and one that is not
/// the whole of a root, so that `"/"` and `"//"` answer false.
fn after_sep(s: &CStr, at: usize) -> bool {
    // SAFETY: `at` indexes `s` or is its terminator, both inside it.
    unsafe { after_pathsep(s.as_ptr(), s.as_ptr().add(at)) != 0 }
}

fn is_absolute(s: &CStr) -> bool {
    // SAFETY: `s` is NUL-terminated.
    unsafe { path_is_absolute(s.as_ptr()) }
}

/// Collapse `.`, `..` and duplicate separators, in place.
fn simplify(s: *mut c_char) {
    // SAFETY: `s` is a NUL-terminated string this module owns; the result is
    // never longer than the input, so it stays inside the allocation.
    unsafe { simplify_filename(s) };
}

// ---------------------------------------------------------------------
// The builtins
// ---------------------------------------------------------------------

/// `glob2regpat({pattern})`: the wildcard pattern as a regular expression.
///
/// # Safety
/// `argvars` is the evaluator's own argument vector, arity 1, and `rettv` a
/// cleared result.
pub unsafe fn f_glob2regpat(argvars: *mut typval_T, rettv: *mut typval_T, _fptr: EvalFuncData) {
    let (args, rettv) = frame!(argvars, rettv);
    let pat = str_arg_chk(args, 0);
    ret_string(
        rettv,
        pat.map_or(ptr::null_mut(), |pat| {
            // SAFETY: `pat` is NUL-terminated, which is what a NULL end
            // pointer promises; a NULL `allow_dirs` asks for none reported.
            unsafe {
                file_pat_to_reg_pat(pat.as_ptr(), ptr::null(), ptr::null_mut(), false as c_int)
            }
        }),
    );
}

/// `isabsolutepath({path})`: whether the path starts at the root.
///
/// # Safety
/// As [`f_glob2regpat`].
pub unsafe fn f_isabsolutepath(argvars: *mut typval_T, rettv: *mut typval_T, _fptr: EvalFuncData) {
    let (args, rettv) = frame!(argvars, rettv);
    rettv.vval.v_number = is_absolute(str_arg(args, 0)) as varnumber_T;
}

/// `pathshorten({path} [, {len}])`: every component but the last one cut
/// down to its first `{len}` characters.
///
/// The length is coerced first, as upstream does, so a bad second argument
/// reports before a bad first one does.
///
/// # Safety
/// As [`f_glob2regpat`], arity 1..2.
pub unsafe fn f_pathshorten(argvars: *mut typval_T, rettv: *mut typval_T, _fptr: EvalFuncData) {
    let (args, rettv) = frame!(argvars, rettv);
    let trim_len = if args.has(1) {
        // SAFETY: a live typval; `tv_get_number` reports its own error and
        // reads as 0 for a type that has no number form.
        (unsafe { tv_get_number(args.ptr(1)) } as c_int).max(1)
    } else {
        1
    };
    rettv.v_type = VAR_STRING;
    let Some(p) = str_arg_chk(args, 0) else {
        rettv.vval.v_string = ptr::null_mut();
        return;
    };
    let shortened = Owned::dup(p);
    // SAFETY: a NUL-terminated string this module owns; shortening only ever
    // moves bytes down, so the result stays inside the allocation.
    unsafe { shorten_dir_len(shortened.0, trim_len) };
    rettv.vval.v_string = shortened.into_raw();
}

/// `simplify({path})`: `.`, `..` and duplicate separators collapsed, without
/// asking the filesystem anything.
///
/// # Safety
/// As [`f_glob2regpat`].
pub unsafe fn f_simplify(argvars: *mut typval_T, rettv: *mut typval_T, _fptr: EvalFuncData) {
    let (args, rettv) = frame!(argvars, rettv);
    let simplified = Owned::dup(str_arg(args, 0)).into_raw();
    simplify(simplified);
    ret_string(rettv, simplified);
}

/// `resolve({path})`: the symlink chain followed to its end.
///
/// # Safety
/// As [`f_glob2regpat`].
pub unsafe fn f_resolve(argvars: *mut typval_T, rettv: *mut typval_T, _fptr: EvalFuncData) {
    let (args, rettv) = frame!(argvars, rettv);
    ret_string(rettv, ptr::null_mut());
    if let Some(resolved) = resolve(str_arg(args, 0)) {
        let raw = resolved.into_raw();
        simplify(raw);
        rettv.vval.v_string = raw;
    }
}

/// Follow the symlink chain from `fname`, or None having reported E655 when
/// it does not end within a hundred links.
fn resolve(fname: &CStr) -> Option<Owned> {
    let mut is_relative_to_current = false;
    let mut has_trailing_pathsep = false;
    let mut limit = 100;

    let mut p = Owned::dup(fname);
    let b = p.bytes();
    if at(b, 0) == b'.' && (is_sep(b, 1) || (at(b, 1) == b'.' && is_sep(b, 2))) {
        is_relative_to_current = true;
    }

    let len = p.len();
    if len > 1 && after_sep(p.cstr(), len) {
        has_trailing_pathsep = true;
        // The trailing separator breaks `readlink`.
        p.set(len - 1, 0);
    }

    // Separate the first component, keeping the remainder -- which starts at
    // the separator before it -- for the walk below to put back.
    let mut remain = None;
    let split = next_component(p.cstr(), 0);
    if at(p.bytes(), split) != 0 {
        remain = Some(Owned::dup(from(p.cstr(), split - 1)));
        p.set(split - 1, 0);
    }

    let buf = LinkBuf::new();
    loop {
        while buf.read(p.cstr()) {
            if limit == 0 {
                err(c"E655: Too many symbolic links (cycle?)");
                return None;
            }
            limit -= 1;

            // The answer keeps the trailing separator the argument had.
            if remain.is_none() && has_trailing_pathsep {
                buf.add_pathsep();
            }

            // Separate the first component of the link's value and hang what
            // is left of it in front of what was already left over.
            let link = buf.cstr();
            let head = usize::from(is_sep(link.to_bytes(), 0));
            let split = next_component(link, head);
            if at(link.to_bytes(), split) != 0 {
                let rest = from(link, split - 1);
                remain = Some(match remain.take() {
                    Some(old) => Owned::cat(rest, old.cstr()),
                    None => Owned::dup(rest),
                });
                buf.0.set(split - 1, 0);
            }

            let mut t = tail(p.cstr());
            if t > 0 && at(p.bytes(), t) == 0 {
                // Ignore a trailing path separator.
                p.set(t - 1, 0);
                t = tail(p.cstr());
            }
            if t > 0 && !is_absolute(buf.cstr()) {
                // The link is relative to the directory of the name it was
                // reached through: resolve it in that same directory.
                p.replace_tail(buf.cstr());
            } else {
                p = Owned::dup(buf.cstr());
            }
        }

        // Append the first component of what is left over.
        let Some(rest) = remain.take() else { break };
        let split = next_component(rest.cstr(), 1);
        let more = at(rest.bytes(), split) != 0;
        p = p.with_suffix(rest.cstr(), split - usize::from(more));
        if more {
            rest.drop_front(split - 1);
            remain = Some(rest);
        }
    }

    // A relative answer is explicitly relative to the current directory if
    // and only if the argument was.
    if !is_sep(p.bytes(), 0) {
        let b = p.bytes();
        let dot_component = at(b, 0) == b'.'
            && (at(b, 1) == 0
                || is_sep(b, 1)
                || (at(b, 1) == b'.' && (at(b, 2) == 0 || is_sep(b, 2))));
        if is_relative_to_current && at(b, 0) != 0 && !dot_component {
            p = Owned::cat(c"./", p.cstr());
        } else if !is_relative_to_current {
            // Strip a leading "./" -- one of them, though upstream's loop
            // counts however many there are.
            if at(b, 0) == b'.' && is_sep(b, 1) {
                p.drop_front(2);
            }
        }
    }

    // And carries no trailing separator unless the argument did -- but "/"
    // and "//" are kept whole, which is what `after_sep` answers false for.
    if !has_trailing_pathsep && after_sep(p.cstr(), p.len()) {
        p.set(tail_with_sep(p.cstr()), 0);
    }
    Some(p)
}
