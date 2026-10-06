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
//! Upstream carries six parameters -- three of them out, two of them a
//! pointer to a pointer -- and a `goto repeat` between them. [`Mods`] is the
//! modifier text and how much of it has been read; [`Fname`] is the string
//! the name lives in (the caller's, or one a stage made), where in it the
//! name starts, and its length, which is *not* its NUL-terminated length:
//! `:h` shortens the name by moving an offset, leaving the tail in the
//! string where the next `:e` still reads it. Every stage is a function over
//! those two, and the `goto` is the loop in [`modify_fname`].
//!
//! The `:h`/`:t`/`:e`/`:r` group does no allocation (bar the `"."` an emptied
//! `:h` falls back to), so [`trim_stages`] works in byte offsets from the
//! name it started at and writes the pair back once.  Those offsets are
//! *signed*: `:e:e` moves the name past the tail, which is exactly the test
//! `is_second_e` makes.
//!
//! Original: `src/nvim/eval/fs.c`, Vim/Neovim, Vim license.

#![forbid(unsafe_code)]

use super::{VALID_HEAD, VALID_PATH, at, is_sep, str_arg_chk};
use crate::eval::do_string_sub;
use crate::eval::typval::NumBuf;
use crate::mbyte::{cluster_len, head_off};
use crate::memory::XString;
use crate::memory::handoff::owned_cstr;
use crate::os::env::{expand_env_save_opt_of, home_replace_in};
use crate::os::fs::{dirname_text, os_isdir_of};
use crate::path::{full_name_of, path_fnamencmp, tail_index, vim_is_abs_name};
use crate::strings::find_char;
use crate::strings::shellescape_of;
use crate::types::{EvalFuncData, MAXPATHL, TypVal};
use core::ffi::{CStr, c_int};
use std::ffi::CString;

// ---------------------------------------------------------------------
// The two things every stage is written against
// ---------------------------------------------------------------------

/// The modifier text and how much of it has been read.
struct Mods<'a> {
    text: &'a CStr,
    used: &'a mut usize,
}

impl Mods<'_> {
    fn advance(&mut self, n: usize) {
        *self.used += n;
    }

    /// Byte `i` past what has been read, or 0 past the end.
    fn at(&self, i: usize) -> u8 {
        at(self.text.to_bytes(), *self.used + i)
    }

    /// Whether the next modifier is `:c`.
    fn is(&self, c: u8) -> bool {
        self.at(0) == b':' && self.at(1) == c
    }
}

/// Where the name being modified lives.
enum Storage<'a> {
    /// The caller's own string.
    Borrowed(&'a CStr),
    /// One a stage made, with its NUL. A `:s` can put a NUL inside it,
    /// which every reader but the final copy stops at.
    Owned(Vec<u8>),
}

/// The name being modified: the string it lives in, where in it the name
/// starts, and its length.
///
/// The length is the answer's, not the string's: `:h` leaves the tail in
/// place and just shortens it, which is what makes `:h:e` work.
struct Fname<'a> {
    storage: Storage<'a>,
    start: usize,
    len: usize,
}

impl Fname<'_> {
    /// The whole string the name lives in, its NUL included.
    fn with_nul(&self) -> &[u8] {
        match &self.storage {
            Storage::Borrowed(text) => text.to_bytes_with_nul(),
            Storage::Owned(text) => text,
        }
    }

    /// The name and everything after it, up to the NUL: what a callee that
    /// takes the C string reads.
    fn cstr(&self) -> &CStr {
        CStr::from_bytes_until_nul(&self.with_nul()[self.start..]).expect("a NUL-terminated name")
    }

    /// Byte `i` of the name; `i` may be past its length.
    fn byte(&self, i: usize) -> u8 {
        at(self.with_nul(), self.start + i)
    }

    /// The name's own `len` bytes.
    fn bytes(&self) -> &[u8] {
        let text = self.with_nul();
        let start = self.start.min(text.len());
        &text[start..(start + self.len).min(text.len())]
    }

    /// Adopt `text` -- with its NUL -- as the string, with the name at
    /// `start` in it.
    fn adopt_at(&mut self, text: Vec<u8>, start: usize) {
        self.storage = Storage::Owned(text);
        self.start = start;
    }

    /// Adopt `text` as both the string and the name.
    fn adopt(&mut self, text: XString) {
        self.adopt_at(text.into_vec(), 0);
    }
}

/// `text` up to its first NUL.
fn until_nul(text: &[u8]) -> &[u8] {
    &text[..text.iter().position(|&b| b == 0).unwrap_or(text.len())]
}

/// Where the part of `s` that may be stripped begins -- past a leading `/`,
/// which `:h` never removes.
fn past_head_off(s: &[u8]) -> usize {
    s.iter().position(|&b| b != b'/').unwrap_or(s.len())
}

/// Whether byte `i` of `s` follows a path separator that is not the
/// trailing byte of a multibyte character, looking back no further than
/// byte `from`.
fn after_sep(s: &[u8], from: usize, i: usize) -> bool {
    i > from && is_sep(s, i - 1) && head_off(&s[from..], i - 1 - from) == 0
}

// ---------------------------------------------------------------------
// The stages
// ---------------------------------------------------------------------

/// `:p` -- the full path.
fn full_path_stage(f: &mut Fname<'_>, tilde_file: bool) {
    // Expand a leading "~", unless the name is literally "~" and the caller
    // says that is a file rather than $HOME.
    let b = f.cstr().to_bytes();
    if at(b, 0) == b'~' && !(tilde_file && at(b, 1) == 0) {
        let expanded = expand_env_save_opt_of(f.cstr(), false);
        f.adopt(expanded);
    }

    // A "/." or "/.." anywhere forces the expansion, which is what removes
    // it; `full_name_save` is slow, so it is skipped when nothing needs it.
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
        i += cluster_len(&b[i..]);
    }
    let has_dot = at(b, i) != 0;
    if has_dot || !vim_is_abs_name(f.cstr()) {
        let full = full_name_of(f.cstr(), has_dot);
        f.adopt(full);
    }

    // A directory answers with a trailing separator, room permitting.
    if os_isdir_of(f.cstr()) {
        let mut dir = f.cstr().to_bytes().to_vec();
        let len = dir.len();
        if len != 0 && !after_sep(&dir, 0, len) && len + 2 <= MAXPATHL as usize {
            dir.push(b'/');
        }
        dir.push(0);
        f.adopt_at(dir, 0);
    }
}

/// `:.` -- relative to the current directory; `:~` -- relative to the home
/// one; `:8` -- the short name, which this platform has none of.
fn home_stages(
    mods: &mut Mods<'_>,
    f: &mut Fname<'_>,
    has_fullname: &mut bool,
    has_homerelative: &mut bool,
) {
    while mods.at(0) == b':' && matches!(mods.at(1), b'.' | b'~' | b'8') {
        let which = mods.at(1);
        mods.advance(2);
        if which == b'8' {
            continue;
        }

        // The full path first, so that the comparison below has something to
        // compare; `expand_env_save` is what removes a leading "~".
        let made = if !*has_fullname && !*has_homerelative {
            Some(if f.byte(0) == b'~' {
                expand_env_save_opt_of(f.cstr(), false)
            } else {
                full_name_of(f.cstr(), false)
            })
        } else {
            None
        };
        *has_fullname = false;
        let p = made.as_ref().map_or(f.cstr(), |made| made.as_cstr());

        if which == b'.' {
            let mut dirname = dirname_text();
            if *has_homerelative {
                dirname = home_replace_in(None, dirname.as_cstr(), MAXPATHL as usize, true);
            }
            let namelen = dirname.len();
            // Not `shorten_fname`: that removes the prefix even when the path
            // does not have one.
            if path_fnamencmp(p, dirname.as_cstr(), namelen) == 0 {
                let rest = p.to_bytes().get(namelen..).unwrap_or_default();
                if is_sep(rest, 0) {
                    let skip = (0..).take_while(|&i| is_sep(rest, i)).count();
                    match made {
                        Some(made) => f.adopt_at(made.into_vec(), namelen + skip),
                        None => f.start += namelen + skip,
                    }
                }
            }
        } else {
            let home = home_replace_in(None, p, MAXPATHL as usize, true);
            // Only replace it when it did start with the home directory.
            if home.first() == Some(&b'~') {
                f.adopt(home);
                *has_homerelative = true;
            }
        }
    }
}

/// `:h` `:8` `:t` `:e` `:r` -- head, tail, root and extension, all of them
/// offsets into the name the group starts with.
fn trim_stages(mods: &mut Mods<'_>, f: &mut Fname<'_>, valid: &mut c_int) {
    // `base` is where in the string the group started; every offset below
    // is from there.
    let mut base = f.start;
    let mut tail = tail_index(f.cstr().to_bytes());
    let mut start = 0usize;
    let mut len = f.cstr().to_bytes().len();

    // ":h" -- drop "/name", repeatable.  Never the leading "/".
    while mods.is(b'h') {
        *valid |= VALID_HEAD as c_int;
        mods.advance(2);
        let s = until_nul(&f.with_nul()[base..]);
        let head = past_head_off(s);
        while tail > head && after_sep(s, head, tail) {
            tail -= head_off(s, tail - 1) + 1;
        }
        len = tail - start;
        if len == 0 {
            // The result is empty: make it "." so that `:cd %:h` works.
            f.adopt_at(b".\0".to_vec(), 0);
            base = 0;
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
        let b = until_nul(&f.with_nul()[base..]);
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

    f.start = base + start;
    f.len = len;
}

/// `:s?pat?sub?` and `:gs?pat?sub?`.  True when a substitution happened, in
/// which case every modifier is offered the result again.
fn subst_stage(mods: &mut Mods<'_>, f: &mut Fname<'_>) -> bool {
    let global = mods.at(0) == b':' && mods.at(1) == b'g' && mods.at(2) == b's';
    if !(mods.is(b's') || global) {
        return false;
    }
    let b = mods.text.to_bytes();
    let mut i = *mods.used + 2 + usize::from(global);
    let sep = at(b, i);
    i += 1;
    if sep == 0 {
        return false;
    }

    // The pattern, then the replacement, each up to the next separator.
    let Some(pat_len) = find_char(&b[i..], c_int::from(sep)) else {
        return false;
    };
    let pat = owned_copy(&b[i..i + pat_len]);
    let j = i + pat_len + 1;
    let Some(sub_len) = find_char(&b[j..], c_int::from(sep)) else {
        return false;
    };
    let sub = owned_copy(&b[j..j + sub_len]);
    let subject = owned_copy(f.bytes());

    *mods.used = j + sub_len + 1;
    let flags = if global { c"g" } else { c"" };
    // No `expr`: a plain replacement rather than a `\=` one.
    let mut out = do_string_sub(&subject, &pat, Some(&sub), None, flags);
    let out_len = out.len();
    out.push(0);
    f.adopt_at(out, 0);
    f.len = out_len;
    true
}

/// `bytes` as a C string the way the C's `xstrnsave` copy read it: up to
/// its first NUL.
fn owned_copy(bytes: &[u8]) -> CString {
    let end = bytes.iter().position(|&b| b == 0).unwrap_or(bytes.len());
    CString::new(&bytes[..end]).expect("cut at its first NUL")
}

/// `:S` -- the name quoted for the shell.
fn shell_stage(mods: &mut Mods<'_>, f: &mut Fname<'_>) {
    if !mods.is(b'S') {
        return;
    }
    let escaped = shellescape_of(&owned_copy(f.bytes()), false, false);
    f.len = escaped.len();
    f.adopt(escaped);
    mods.advance(2);
}

// ---------------------------------------------------------------------
// The entry points
// ---------------------------------------------------------------------

/// Apply the modifiers at `mods[*used..]` to the name `fname[..len]`, whose
/// string the stages may read past `len` up to its NUL (`<cword>`'s line).
///
/// Answers which of `VALID_PATH`/`VALID_HEAD` were reached -- `eval_vars`
/// needs both before it will accept an empty `%` -- and the name: the
/// caller's own bytes when no stage replaced them.
pub(crate) fn modify_fname(
    mods: &CStr,
    used: &mut usize,
    tilde_file: bool,
    fname: &CStr,
    len: usize,
) -> (c_int, Vec<u8>) {
    let mut mods = Mods { text: mods, used };
    let mut f = Fname {
        storage: Storage::Borrowed(fname),
        start: 0,
        len,
    };

    let mut valid = 0;
    let mut has_fullname = false;
    let mut has_homerelative = false;
    loop {
        if mods.is(b'p') {
            has_fullname = true;
            valid |= VALID_PATH as c_int;
            mods.advance(2);
            full_path_stage(&mut f, tilde_file);
        }
        home_stages(&mut mods, &mut f, &mut has_fullname, &mut has_homerelative);
        trim_stages(&mut mods, &mut f, &mut valid);
        // A ":s" that did something offers the result to every modifier
        // again -- upstream's `goto repeat`.
        if !subst_stage(&mut mods, &mut f) {
            break;
        }
    }
    shell_stage(&mut mods, &mut f);
    (valid, f.bytes().to_vec())
}

/// `fnamemodify({fname}, {mods})`.
pub fn f_fnamemodify(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    let mut numbuf = NumBuf::new();
    let mut buf = NumBuf::new();
    let (fname, mods) = (
        str_arg_chk(args, 0, &mut numbuf),
        str_arg_chk(args, 1, &mut buf),
    );
    let (Some(fname), Some(mods)) = (fname, mods) else {
        result.write_string_raw(core::ptr::null_mut());
        return;
    };
    let name = if mods.is_empty() {
        fname.to_bytes().to_vec()
    } else {
        modify_fname(mods, &mut 0, false, fname, fname.to_bytes().len()).1
    };
    result.write_string_raw(owned_cstr(name));
}

#[cfg(test)]
mod tests {
    use super::modify_fname;
    use std::ffi::CString;

    /// `fnamemodify(name, mods)` for the modifiers that touch neither the
    /// file system nor an option, and how much of `mods` was read.
    fn modified(name: &str, mods: &str) -> (Vec<u8>, usize) {
        let (name, mods) = (CString::new(name).unwrap(), CString::new(mods).unwrap());
        let mut used = 0;
        let len = name.as_bytes().len();
        let (_, answer) = modify_fname(&mods, &mut used, false, &name, len);
        (answer, used)
    }

    /// Answers cut from the reference binary's `fnamemodify()`.
    #[test]
    fn head_tail_root_and_extension() {
        for (name, mods, want) in [
            ("/a/b/c.d.txt", ":t:r", "c.d"),
            ("/a/b/c.d.txt", ":h:h", "/a"),
            ("/a/b/c.d.txt", ":e:e", "d.txt"),
            ("/a/b/c.d.txt", ":r:r:r", "/a/b/c"),
            ("/a/b/c.d.txt", ":e:e:e", "d.txt"),
            ("/a/b/", ":h:t", "b"),
            ("a", ":h", "."),
            ("/", ":h", "/"),
            ("x/y.tar.gz", ":t:e:e:r", "tar"),
            ("a.b", ":8:t", "a.b"),
            ("ü/ö.c", ":h:t:e", ""),
        ] {
            let (answer, used) = modified(name, mods);
            assert_eq!(answer, want.as_bytes(), "{name} {mods}");
            assert_eq!(used, mods.len(), "{name} {mods}");
        }
    }

    /// The name's length, not its terminator, bounds the answer; a stage
    /// still reads past it to the NUL (`:e` after `:h`).
    #[test]
    fn a_shortened_name_keeps_its_tail() {
        let name = CString::new("/a/b.c/d").unwrap();
        let mods = CString::new(":h:e").unwrap();
        let mut used = 0;
        let (_, answer) = modify_fname(&mods, &mut used, false, &name, 8);
        assert_eq!(answer, b"c");
        // An unknown modifier stops the walk where it is.
        let mods = CString::new(":t:x").unwrap();
        let mut used = 0;
        let (_, answer) = modify_fname(&mods, &mut used, false, &name, 8);
        assert_eq!((answer.as_slice(), used), (&b"d"[..], 2));
    }
}
