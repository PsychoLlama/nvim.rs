//! Rewriting a file name as text -- `fnamemodify()` and the `:h` `:t` `:r`
//! `:e` `:p` `:~` `:.` `:s?` `:gs?` modifier language it shares with `%:h` and
//! friends on the command line.
//!
//! [`modify_fname`] is the whole modifier alphabet: it expands to a full path,
//! strips head, tail, root and extension, makes the name relative to the home
//! directory or the current one, and runs `:s` substitutions over the result,
//! any of which may replace the caller's buffer.  Nothing here touches the
//! filesystem except `:p`, which has to resolve the name to say whether it is
//! a directory.
//!
//! # What the stages thread through
//!
//! Upstream carries six parameters -- three of them out -- and a `goto
//! repeat` between them.  [`Mods`] is the modifier text and the cursor into
//! it; [`Fname`] is the name, the buffer it lives in and its length, which is
//! *not* its NUL-terminated length: `:h` shortens the name by moving an
//! offset, leaving the tail in the buffer where the next `:e` still reads it.
//! Every stage is a function over those two, and the `goto` is the loop in
//! [`modify_fname`].
//!
//! The `:h`/`:t`/`:e`/`:r` group does no allocation (bar the `"."` an emptied
//! `:h` falls back to), so [`trim_stages`] works in byte offsets from the
//! name it started at and writes the pair back once.  Those offsets are
//! *signed*: `:e:e` moves the name past the tail, which is exactly the test
//! `is_second_e` makes.
//!
//! Original: `src/nvim/eval/fs.c`, Vim/Neovim, Vim license.

#![deny(unsafe_op_in_unsafe_fn)]

use super::{
    Owned, VALID_HEAD, VALID_PATH, at, frame, from, is_sep, numbuf, ret_string, str_arg_buf,
    str_arg_chk,
};
use crate::eval::do_string_sub;
use crate::mbyte::{utf_head_off, utfc_ptr2len};
use crate::memory::xfree;
use crate::os::env::{expand_env_save, home_replace};
use crate::os::fs::{os_dirname, os_isdir};
use crate::path::{
    FullName_save, add_pathsep, after_pathsep, get_past_head, path_fnamencmp, path_tail,
    vim_isAbsName,
};
use crate::strings::{vim_strchr, vim_strsave_shellescape, xstrnsave};
use crate::types::{EvalFuncData, MAXPATHL, buf_T, size_t, typval_T};
use core::ffi::{CStr, c_char, c_int, c_void};
use core::mem::replace;
use core::ptr;

/// A `MAXPATHL` scratch, which is what `os_dirname` and `home_replace` write
/// into.
type PathBuf = [c_char; MAXPATHL as usize];

// ---------------------------------------------------------------------
// The two things every stage is written against
// ---------------------------------------------------------------------

/// The modifier text and how much of it has been read.
#[derive(Clone, Copy)]
struct Mods {
    src: *const c_char,
    usedlen: *mut size_t,
}

impl Mods {
    /// # Safety
    /// `src` is NUL-terminated and `usedlen` a live cursor into it.
    unsafe fn new(src: *const c_char, usedlen: *mut size_t) -> Self {
        Self { src, usedlen }
    }

    fn text<'a>(self) -> &'a CStr {
        // SAFETY: the constructor's obligation.
        unsafe { CStr::from_ptr(self.src) }
    }

    fn used(self) -> usize {
        // SAFETY: the constructor's obligation.
        unsafe { *self.usedlen }
    }

    fn set_used(self, n: usize) {
        // SAFETY: the constructor's obligation.
        unsafe { *self.usedlen = n as size_t };
    }

    fn advance(self, n: usize) {
        self.set_used(self.used() + n);
    }

    /// Byte `i` past the cursor, or 0 past the end.
    fn at(self, i: usize) -> u8 {
        at(self.text().to_bytes(), self.used() + i)
    }

    /// Whether the next modifier is `:c`.
    fn is(self, c: u8) -> bool {
        self.at(0) == b':' && self.at(1) == c
    }
}

/// The name being modified: the pointer to it, the buffer this call
/// allocated for it (which the *caller* frees), and its length.
///
/// The length is the answer's, not the buffer's: `:h` leaves the tail in
/// place and just shortens it, which is what makes `:h:e` work.
#[derive(Clone, Copy)]
struct Fname {
    fnamep: *mut *mut c_char,
    bufp: *mut *mut c_char,
    fnamelen: *mut size_t,
}

impl Fname {
    /// # Safety
    /// The three are the caller's own out-parameters, all live, with `*bufp`
    /// either NULL or an owned string.
    unsafe fn new(fnamep: *mut *mut c_char, bufp: *mut *mut c_char, fnamelen: *mut size_t) -> Self {
        Self {
            fnamep,
            bufp,
            fnamelen,
        }
    }

    fn name(self) -> *mut c_char {
        // SAFETY: the constructor's obligation.
        unsafe { *self.fnamep }
    }

    fn set_name(self, p: *mut c_char) {
        // SAFETY: the constructor's obligation.
        unsafe { *self.fnamep = p };
    }

    fn cstr<'a>(self) -> &'a CStr {
        cstr_at(self.name())
    }

    fn len(self) -> usize {
        // SAFETY: the constructor's obligation.
        unsafe { *self.fnamelen }
    }

    fn set_len(self, n: usize) {
        // SAFETY: the constructor's obligation.
        unsafe { *self.fnamelen = n as size_t };
    }

    /// Byte `i` of the name; `i` may be the one past its length, which is
    /// what `:S` reads and puts back.
    fn byte(self, i: usize) -> u8 {
        // SAFETY: the name has at least `*fnamelen` bytes and a terminator.
        unsafe { *self.name().add(i) as u8 }
    }

    fn set_byte(self, i: usize, b: u8) {
        // SAFETY: as [`Fname::byte`].
        unsafe { *self.name().add(i) = b as c_char };
    }

    /// Adopt `p` as both the name and the buffer, freeing the buffer this
    /// call had allocated before it.
    fn adopt(self, p: *mut c_char) {
        self.set_name(p);
        self.adopt_buf(p);
    }

    /// Adopt `p` as the buffer only -- `:.` leaves the name pointing *into*
    /// it rather than at its start.
    fn adopt_buf(self, p: *mut c_char) {
        // SAFETY: the constructor's obligation.  `p` is always a fresh
        // allocation, so it is never the buffer being freed.
        let old = unsafe { replace(&mut *self.bufp, p) };
        // SAFETY: `*bufp` was NULL or a string this call had allocated.
        unsafe { xfree(old.cast::<c_void>()) };
    }
}

// ---------------------------------------------------------------------
// One-line views of what the stages call
// ---------------------------------------------------------------------

fn cstr_at<'a>(p: *const c_char) -> &'a CStr {
    // SAFETY: every pointer this module holds is into a NUL-terminated name.
    unsafe { CStr::from_ptr(p) }
}

fn offset(p: *mut c_char, n: usize) -> *mut c_char {
    // SAFETY: `n` is an offset within the NUL-terminated string at `p`.
    unsafe { p.add(n) }
}

/// The length of the character `s` starts with, combining characters
/// included.  Never 0 here: the callers stop at the terminator.
fn char_len(s: &CStr) -> usize {
    // SAFETY: `s` is NUL-terminated, which is where the scan stops.
    unsafe { utfc_ptr2len(s.as_ptr()) as usize }
}

/// How far back the start of the character ending at byte `i` of `s` is.
fn head_off(s: &CStr, i: usize) -> usize {
    // SAFETY: `i` indexes `s`, so both pointers are inside it.
    unsafe { utf_head_off(s.as_ptr(), s.as_ptr().add(i)) as usize }
}

/// Where the last component of `s` starts.
fn tail_off(s: &CStr) -> usize {
    // SAFETY: `s` is NUL-terminated, and the answer is inside it.
    unsafe { path_tail(s.as_ptr()).offset_from(s.as_ptr()) as usize }
}

/// Where the part of `s` that may be stripped begins -- past a leading `/`
/// or a drive letter, neither of which `:h` removes.
fn past_head_off(s: &CStr) -> usize {
    // SAFETY: as [`tail_off`].
    unsafe { get_past_head(s.as_ptr()).offset_from(s.as_ptr()) as usize }
}

/// Whether byte `i` of `s` follows a path separator that is not the whole of
/// a root, looking back no further than byte `from`.
fn after_sep(s: &CStr, from: usize, i: usize) -> bool {
    // SAFETY: both index `s` or are its terminator.
    unsafe { after_pathsep(s.as_ptr().add(from), s.as_ptr().add(i)) != 0 }
}

/// Whether `a` and `b` agree over `n` bytes, by the rules 'fileignorecase'
/// sets for file names.
fn same_prefix(a: &CStr, b: &CStr, n: usize) -> bool {
    // SAFETY: both are NUL-terminated and `n` is within each.
    unsafe { path_fnamencmp(a.as_ptr(), b.as_ptr(), n as size_t) == 0 }
}

/// `$VAR` and a leading `~` expanded; NULL when the expansion failed.
fn expand_env(p: *mut c_char) -> *mut c_char {
    // SAFETY: `p` is a NUL-terminated name.
    unsafe { expand_env_save(p) }
}

/// The name as a full path; NULL when the current directory is unreadable.
/// `force` asks for it even when the name already looks absolute, which is
/// how an embedded `/.` or `/..` is removed.
fn full_name(p: *mut c_char, force: bool) -> *mut c_char {
    // SAFETY: `p` is a NUL-terminated name.
    unsafe { FullName_save(p, force) }
}

fn is_abs_name(s: &CStr) -> bool {
    // SAFETY: `s` is NUL-terminated.
    unsafe { vim_isAbsName(s.as_ptr()) }
}

fn is_dir(s: &CStr) -> bool {
    // SAFETY: `s` is NUL-terminated.
    unsafe { os_isdir(s.as_ptr()) }
}

/// A copy of `s` with `extra` spare bytes after it, for the separator `:p`
/// appends to a directory.
fn grown_copy(s: &CStr, extra: usize) -> *mut c_char {
    // SAFETY: `s` is NUL-terminated; `xstrnsave` allocates the length asked
    // for plus a terminator and zero-fills what it does not copy.
    unsafe { xstrnsave(s.as_ptr(), (s.to_bytes().len() + extra) as size_t) }
}

/// Append a path separator if there is not one already.
fn append_sep(p: *mut c_char) {
    // SAFETY: only called on a buffer [`grown_copy`] left room in.
    unsafe { add_pathsep(p) };
}

/// The current directory, into `buf`.
fn get_dirname(buf: &mut PathBuf) {
    // SAFETY: `buf` is exactly the `MAXPATHL` writable bytes claimed.
    unsafe { os_dirname(buf.as_mut_ptr(), MAXPATHL as size_t) };
}

/// `src` with the home directory replaced by `~`, into `buf`.
fn home_rel(src: &CStr, buf: &mut PathBuf) {
    let (from, into, room) = (src.as_ptr(), buf.as_mut_ptr(), MAXPATHL as size_t);
    // SAFETY: `src` is NUL-terminated and `buf` is `MAXPATHL` writable bytes;
    // a NULL buffer means no 'path'-relative shortening.
    unsafe { home_replace(ptr::null::<buf_T>(), from, into, room, true) };
}

/// What a scratch holds, which both fillers above NUL-terminate.
fn scratch(buf: &PathBuf) -> &CStr {
    // SAFETY: both fillers write a terminator within `MAXPATHL`.
    unsafe { CStr::from_ptr(buf.as_ptr()) }
}

/// Where the character `c` next appears in `s`.  A character, not a byte:
/// a separator above 0x7f is matched as the multibyte sequence it encodes.
fn find_char(s: &CStr, c: u8) -> Option<usize> {
    // SAFETY: `s` is NUL-terminated, which is where the search stops.
    let found = unsafe { vim_strchr(s.as_ptr(), c as c_int) };
    // SAFETY: a hit is inside `s`, and no offset is taken from a miss.
    (!found.is_null()).then(|| unsafe { found.offset_from(s.as_ptr()) as usize })
}

/// `text` with `pat` replaced by `sub`, and the length of the answer.
fn string_sub(
    text: &Owned,
    len: usize,
    pat: &Owned,
    sub: &Owned,
    global: bool,
) -> (*mut c_char, usize) {
    let flags = if global { c"g" } else { c"" };
    let mut out_len: size_t = 0;
    let (n, expr, fl, ret) = (
        len as size_t,
        ptr::null_mut::<typval_T>(),
        flags.as_ptr(),
        &raw mut out_len,
    );
    // SAFETY: `text` has `len` readable bytes -- it was copied from exactly
    // that many -- and the rest are NUL-terminated strings; a NULL `expr` is
    // what asks for a plain replacement rather than a `\=` one.
    let out = unsafe { do_string_sub(text.0, n, pat.0, sub.0, expr, fl, ret) };
    (out, out_len as usize)
}

/// The name, single-quoted for the shell.
fn shellescape(s: &CStr) -> *mut c_char {
    // SAFETY: `s` is NUL-terminated; neither flag asks for cmdline-special
    // or newline escaping.
    unsafe { vim_strsave_shellescape(s.as_ptr(), false, false) }
}

// ---------------------------------------------------------------------
// The stages
// ---------------------------------------------------------------------

/// `:p` -- the full path.  None when the expansion failed, which is
/// [`modify_fname`]'s `-1`.
fn full_path_stage(f: Fname, tilde_file: bool) -> Option<()> {
    // Expand a leading "~", unless the name is literally "~" and the caller
    // says that is a file rather than $HOME.
    let b = f.cstr().to_bytes();
    if at(b, 0) == b'~' && !(tilde_file && at(b, 1) == 0) {
        f.adopt(expand_env(f.name()));
        if f.name().is_null() {
            return None;
        }
    }

    // A "/." or "/.." anywhere forces the expansion, which is what removes
    // it; `FullName_save` is slow, so it is skipped when nothing needs it.
    let b = f.cstr().to_bytes();
    let mut i = 0;
    while at(b, i) != 0 {
        if is_sep(b, i)
            && at(b, i + 1) == b'.'
            && (at(b, i + 2) == 0
                || is_sep(b, i + 2)
                || (at(b, i + 2) == b'.' && (at(b, i + 3) == 0 || is_sep(b, i + 3))))
        {
            break;
        }
        i += char_len(from(f.cstr(), i));
    }
    let has_dot = at(b, i) != 0;
    if has_dot || !is_abs_name(f.cstr()) {
        f.adopt(full_name(f.name(), has_dot));
        if f.name().is_null() {
            return None;
        }
    }

    // A directory answers with a trailing separator.
    if is_dir(f.cstr()) {
        f.adopt(grown_copy(f.cstr(), 2));
        append_sep(f.name());
    }
    Some(())
}

/// `:.` -- relative to the current directory; `:~` -- relative to the home
/// one; `:8` -- the short name, which this platform has none of.
fn home_stages(mods: Mods, f: Fname, has_fullname: &mut bool, has_homerelative: &mut bool) {
    let mut dirname: PathBuf = [0; MAXPATHL as usize];
    while mods.at(0) == b':' && matches!(mods.at(1), b'.' | b'~' | b'8') {
        let which = mods.at(1);
        mods.advance(2);
        if which == b'8' {
            continue;
        }

        // The full path first, so that the comparison below has something to
        // compare; `expand_env_save` is what removes a leading "~".
        let mut owned = None;
        let p = if !*has_fullname && !*has_homerelative {
            let made = if f.byte(0) == b'~' {
                expand_env(f.name())
            } else {
                full_name(f.name(), false)
            };
            owned = Some(Owned(made));
            made
        } else {
            f.name()
        };
        *has_fullname = false;
        if p.is_null() {
            continue;
        }

        if which == b'.' {
            get_dirname(&mut dirname);
            if *has_homerelative {
                let saved = Owned::dup(scratch(&dirname));
                home_rel(saved.cstr(), &mut dirname);
            }
            let namelen = scratch(&dirname).to_bytes().len();
            // Not `shorten_fname`: that removes the prefix even when the path
            // does not have one.
            if same_prefix(cstr_at(p), scratch(&dirname), namelen) {
                let rest = from(cstr_at(p), namelen).to_bytes();
                if is_sep(rest, 0) {
                    let mut skip = 0;
                    while at(rest, skip) != 0 && is_sep(rest, skip) {
                        skip += 1;
                    }
                    f.set_name(offset(p, namelen + skip));
                    if let Some(pbuf) = owned.take() {
                        f.adopt_buf(pbuf.into_raw());
                    }
                }
            }
        } else {
            home_rel(cstr_at(p), &mut dirname);
            // Only replace it when it did start with the home directory.
            if scratch(&dirname).to_bytes().first() == Some(&b'~') {
                f.adopt(Owned::dup(scratch(&dirname)).into_raw());
                *has_homerelative = true;
            }
        }
    }
}

/// `:h` `:8` `:t` `:e` `:r` -- head, tail, root and extension, all of them
/// offsets into the name the group starts with.
fn trim_stages(mods: Mods, f: Fname, valid: &mut c_int) {
    let mut base = f.name();
    let mut tail = tail_off(f.cstr());
    let mut start = 0usize;
    let mut len = f.cstr().to_bytes().len();

    // ":h" -- drop "/name", repeatable.  Never the leading "/" or "c:\".
    while mods.is(b'h') {
        *valid |= VALID_HEAD as c_int;
        mods.advance(2);
        let s = cstr_at(base);
        let head = past_head_off(s);
        while tail > head && after_sep(s, head, tail) {
            tail -= head_off(s, tail - 1) + 1;
        }
        len = tail - start;
        if len == 0 {
            // The result is empty: make it "." so that `:cd %:h` works.
            let dot = Owned::dup(c".").into_raw();
            f.adopt(dot);
            base = dot;
            (tail, start, len) = (0, 0, 1);
        } else {
            while tail > head && !after_sep(s, head, tail) {
                tail -= head_off(s, tail - 1) + 1;
            }
        }
    }

    // ":8" -- the short name, which is a no-op away from MS-Windows.
    if mods.is(b'8') {
        mods.advance(2);
    }

    // ":t" -- just the basename.
    if mods.is(b't') {
        mods.advance(2);
        len -= tail - start;
        start = tail;
    }

    // ":e" -- the extension; ":r" -- the root.  Both repeatable, and a
    // second ":e" looks for the dot *before* what the first one left.
    while mods.is(b'e') || mods.is(b'r') {
        let want_ext = mods.at(1) == b'e';
        let b = cstr_at(base).to_bytes();
        let second_e = start > tail;
        let mut s = if want_ext && second_e {
            start as isize - 2
        } else {
            start as isize + len as isize - 1
        };
        while s > tail as isize && at(b, s as usize) != b'.' {
            s -= 1;
        }
        if want_ext {
            if s > tail as isize {
                // Stopped at a dot, so anchor just past it.  The name may
                // move *backwards*, and the length follows it.
                let anchor = s + 1;
                len = (len as isize + start as isize - anchor) as usize;
                start = anchor as usize;
            } else if start <= tail {
                len = 0;
            }
        } else if s > tail.max(start) as isize {
            // ":r" must stop at both the tail and the name, or
            // "path/to/this.file.ext:e:e:r:r" and ":r:r:r" take too many
            // roots.
            len = (s - start as isize) as usize;
        }
        mods.advance(2);
    }

    f.set_name(offset(base, start));
    f.set_len(len);
}

/// `:s?pat?sub?` and `:gs?pat?sub?`.  True when a substitution happened, in
/// which case every modifier is offered the result again.
fn subst_stage(mods: Mods, f: Fname) -> bool {
    let global = mods.at(0) == b':' && mods.at(1) == b'g' && mods.at(2) == b's';
    if !(mods.is(b's') || global) {
        return false;
    }
    let text = mods.text();
    let b = text.to_bytes();
    let mut i = mods.used() + 2 + usize::from(global);
    let sep = at(b, i);
    i += 1;
    if sep == 0 {
        return false;
    }

    // The pattern, then the replacement, each up to the next separator.
    let Some(pat_len) = find_char(from(text, i), sep) else {
        return false;
    };
    // SAFETY: `pat_len` bytes from `i` are inside `text`.
    let pat = unsafe { Owned::dupz(text.as_ptr().add(i), pat_len) };
    let j = i + pat_len + 1;
    let Some(sub_len) = find_char(from(text, j), sep) else {
        return false;
    };
    // SAFETY: as above, from `j`.
    let sub = unsafe { Owned::dupz(text.as_ptr().add(j), sub_len) };
    // SAFETY: the name has `*fnamelen` readable bytes by [`Fname`]'s contract.
    let subject = unsafe { Owned::dupz(f.name(), f.len()) };

    mods.set_used(j + sub_len + 1);
    let (out, out_len) = string_sub(&subject, f.len(), &pat, &sub, global);
    f.adopt(out);
    f.set_len(out_len);
    true
}

/// `:S` -- the name quoted for the shell.
fn shell_stage(mods: Mods, f: Fname) {
    if !mods.is(b'S') {
        return;
    }
    // The escaper wants a NUL-terminated string, so the byte after the name
    // is borrowed for the call and put back afterwards.
    let len = f.len();
    let cut = f.byte(len);
    if cut != 0 {
        f.set_byte(len, 0);
    }
    let escaped = shellescape(f.cstr());
    if cut != 0 {
        f.set_byte(len, cut);
    }
    f.adopt(escaped);
    f.set_len(cstr_at(escaped).to_bytes().len());
    mods.advance(2);
}

// ---------------------------------------------------------------------
// The entry points
// ---------------------------------------------------------------------

/// Apply the modifiers at `src[*usedlen]` to the name in `*fnamep`.
///
/// Answers which of `VALID_PATH`/`VALID_HEAD` were reached -- `eval_vars`
/// needs both before it will accept an empty `%` -- or -1 when an expansion
/// failed, in which case `*fnamep` is NULL.
///
/// # Safety
/// `src` is NUL-terminated and `*usedlen` a cursor within it; `*fnamep` is a
/// NUL-terminated name with at least `*fnamelen` bytes; `*bufp` is NULL or an
/// owned string, and the caller frees whatever it holds afterwards.
pub unsafe fn modify_fname(
    src: *mut c_char,
    tilde_file: bool,
    usedlen: *mut size_t,
    fnamep: *mut *mut c_char,
    bufp: *mut *mut c_char,
    fnamelen: *mut size_t,
) -> c_int {
    // SAFETY: the caller's contract.
    let mods = unsafe { Mods::new(src, usedlen) };
    // SAFETY: the caller's contract.
    let f = unsafe { Fname::new(fnamep, bufp, fnamelen) };

    let mut valid = 0;
    let mut has_fullname = false;
    let mut has_homerelative = false;
    loop {
        if mods.is(b'p') {
            has_fullname = true;
            valid |= VALID_PATH as c_int;
            mods.advance(2);
            if full_path_stage(f, tilde_file).is_none() {
                return -1;
            }
        }
        home_stages(mods, f, &mut has_fullname, &mut has_homerelative);
        trim_stages(mods, f, &mut valid);
        // A ":s" that did something offers the result to every modifier
        // again -- upstream's `goto repeat`.
        if !subst_stage(mods, f) {
            break;
        }
    }
    shell_stage(mods, f);
    valid
}

/// `fnamemodify({fname}, {mods})`.
///
/// # Safety
/// `argvars` is the evaluator's own argument vector, arity 2, and `rettv` a
/// cleared result.
pub unsafe fn f_fnamemodify(argvars: *mut typval_T, rettv: *mut typval_T, _fptr: EvalFuncData) {
    let (args, rettv) = frame!(argvars, rettv);
    let mut buf = numbuf();
    let (fname, mods) = (str_arg_chk(args, 0), str_arg_buf(args, 1, &mut buf));
    let (Some(fname), Some(mods)) = (fname, mods) else {
        ret_string(rettv, ptr::null_mut());
        return;
    };

    // `modify_fname` may replace the name with something it allocated, which
    // it hands back through `owned` for this call to free.
    let mut name = fname.as_ptr().cast_mut();
    let mut len = fname.to_bytes().len();
    let mut owned = Owned(ptr::null_mut());
    if !mods.to_bytes().is_empty() {
        let mut used: size_t = 0;
        let (m, u) = (mods.as_ptr().cast_mut(), &raw mut used);
        let (n, b, l) = (&raw mut name, &raw mut owned.0, &raw mut len);
        // SAFETY: `mods` and `name` are NUL-terminated, `len` is the name's
        // own length, and the three out-parameters are this call's locals.
        unsafe { modify_fname(m, false, u, n, b, l) };
    }
    ret_string(
        rettv,
        if name.is_null() {
            ptr::null_mut()
        } else {
            // SAFETY: `name` has `len` readable bytes.
            unsafe { Owned::dupz(name, len) }.into_raw()
        },
    );
}
