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
//! # How the strings are held
//!
//! `resolve()` juggles three strings across a loop with an early exit --
//! the name resolved so far, the part of the argument still to be appended,
//! and the link just read -- which upstream frees by hand before each
//! `return`. Here each is a `Vec<u8>` without its terminator, every offset
//! is a byte index into one of them, and the C's reads one past a component
//! are the terminator, which [`at`] answers as 0.
//!
//! Original: `src/nvim/eval/fs.c`, Vim/Neovim, Vim license.

#![forbid(unsafe_code)]

use super::{after_sep, at, err, is_sep, next_component, str_arg, str_arg_chk, tail_with_sep};
use crate::eval::typval::NumBuf;
use crate::eval::typval::tv_get_number;
use crate::fileio::file_pat_to_regpat;
use crate::memory::ThinCString;
use crate::path::{path_is_absolute, shorten_dir_name, simplify_name, tail_index};
use crate::types::{EvalFuncData, MAXPATHL, TypVal, VAR_STRING, VarNumber};
use core::ffi::{CStr, c_int};
use std::ffi::OsStr;
use std::os::unix::ffi::{OsStrExt, OsStringExt};

// ---------------------------------------------------------------------
// The byte arithmetic over a path
// ---------------------------------------------------------------------

/// The value of the symlink `p` names, as `readlink` fills a [`MAXPATHL`]
/// buffer with it; `None` when `p` is not a link or the value is empty.
fn read_link(p: &[u8]) -> Option<Vec<u8>> {
    let mut link = std::fs::read_link(OsStr::from_bytes(p))
        .ok()?
        .into_os_string()
        .into_vec();
    link.truncate(MAXPATHL as usize);
    (!link.is_empty()).then_some(link)
}

/// Append a path separator to the link value `p`, unless it ends in one
/// already or a `MAXPATHL` buffer has no room for it -- upstream's
/// `add_pathsep` over the `readlink` buffer.
fn add_pathsep(p: &mut Vec<u8>) {
    let len = p.len();
    if len == 0 || after_sep(p, len) || len + 2 > MAXPATHL as usize {
        return;
    }
    p.push(b'/');
}

/// Whether `p` starts at the root: [`path_is_absolute`], which reads only
/// the first byte, of bytes without their terminator.
fn is_absolute(p: &[u8]) -> bool {
    let probe = [p.first().copied().unwrap_or(0), 0];
    path_is_absolute(CStr::from_bytes_until_nul(&probe).unwrap_or(c""))
}

/// `name` with `.`, `..` and duplicate separators collapsed.
fn simplify(name: &[u8]) -> ThinCString {
    let mut text = Vec::with_capacity(name.len() + 1);
    text.extend_from_slice(name);
    text.push(0);
    let len = simplify_name(&mut text);
    text.truncate(len);
    ThinCString::from_vec(text)
}

// ---------------------------------------------------------------------
// The builtins
// ---------------------------------------------------------------------

/// `glob2regpat({pattern})`: the wildcard pattern as a regular expression.
pub fn f_glob2regpat(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    let mut numbuf = NumBuf::new();
    let pat = str_arg_chk(args, 0, &mut numbuf);
    result.write_string(pat.and_then(file_pat_to_regpat).map(ThinCString::from));
}

/// `isabsolutepath({path})`: whether the path starts at the root.
pub fn f_isabsolutepath(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    let mut numbuf = NumBuf::new();
    result.write_number(path_is_absolute(str_arg(args, 0, &mut numbuf)) as VarNumber);
}

/// `pathshorten({path} [, {len}])`: every component but the last one cut
/// down to its first `{len}` characters.
///
/// The length is coerced first, as upstream does, so a bad second argument
/// reports before a bad first one does.
pub fn f_pathshorten(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    let mut numbuf = NumBuf::new();
    let trim_len = if args.len() > 1 {
        (tv_get_number(&args[1]) as c_int).max(1)
    } else {
        1
    };
    result.write_empty(VAR_STRING);
    let Some(p) = str_arg_chk(args, 0, &mut numbuf) else {
        result.write_string(None);
        return;
    };
    let mut shortened = p.to_bytes_with_nul().to_vec();
    shorten_dir_name(&mut shortened, trim_len);
    let end = shortened
        .iter()
        .position(|&b| b == 0)
        .unwrap_or(shortened.len());
    shortened.truncate(end);
    result.write_string(Some(ThinCString::from_vec(shortened)));
}

/// `simplify({path})`: `.`, `..` and duplicate separators collapsed, without
/// asking the filesystem anything.
pub fn f_simplify(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    let mut numbuf = NumBuf::new();
    result.write_string(Some(simplify(str_arg(args, 0, &mut numbuf).to_bytes())));
}

/// `resolve({path})`: the symlink chain followed to its end.
pub fn f_resolve(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    let mut numbuf = NumBuf::new();
    result.write_string(None);
    if let Some(resolved) = resolve(str_arg(args, 0, &mut numbuf)) {
        result.write_string(Some(simplify(&resolved)));
    }
}

/// Follow the symlink chain from `fname`, or None having reported E655 when
/// it does not end within a hundred links.
fn resolve(fname: &CStr) -> Option<Vec<u8>> {
    let mut is_relative_to_current = false;
    let mut has_trailing_pathsep = false;
    let mut limit = 100;

    let mut p = fname.to_bytes().to_vec();
    if at(&p, 0) == b'.' && (is_sep(&p, 1) || (at(&p, 1) == b'.' && is_sep(&p, 2))) {
        is_relative_to_current = true;
    }

    let len = p.len();
    if len > 1 && after_sep(&p, len) {
        has_trailing_pathsep = true;
        // The trailing separator breaks `readlink`.
        p.truncate(len - 1);
    }

    // Separate the first component, keeping the remainder -- which starts at
    // the separator before it -- for the walk below to put back.
    let mut remain = None;
    let split = next_component(&p, 0);
    if at(&p, split) != 0 {
        remain = Some(p[split - 1..].to_vec());
        p.truncate(split - 1);
    }

    loop {
        while let Some(mut link) = read_link(&p) {
            if limit == 0 {
                err(c"E655: Too many symbolic links (cycle?)");
                return None;
            }
            limit -= 1;

            // The answer keeps the trailing separator the argument had.
            if remain.is_none() && has_trailing_pathsep {
                add_pathsep(&mut link);
            }

            // Separate the first component of the link's value and hang what
            // is left of it in front of what was already left over.
            let head = usize::from(is_sep(&link, 0));
            let split = next_component(&link, head);
            if at(&link, split) != 0 {
                let rest = &link[split - 1..];
                remain = Some(match remain.take() {
                    Some(old) => [rest, &old].concat(),
                    None => rest.to_vec(),
                });
                link.truncate(split - 1);
            }

            let mut t = tail_index(&p);
            if t > 0 && at(&p, t) == 0 {
                // Ignore a trailing path separator.
                p.truncate(t - 1);
                t = tail_index(&p);
            }
            if t > 0 && !is_absolute(&link) {
                // The link is relative to the directory of the name it was
                // reached through: resolve it in that same directory.
                p.truncate(t);
                p.extend_from_slice(&link);
            } else {
                p = link;
            }
        }

        // Append the first component of what is left over.
        let Some(rest) = remain.take() else { break };
        let split = next_component(&rest, 1);
        let more = at(&rest, split) != 0;
        p.extend_from_slice(&rest[..split - usize::from(more)]);
        if more {
            remain = Some(rest[split - 1..].to_vec());
        }
    }

    // A relative answer is explicitly relative to the current directory if
    // and only if the argument was.
    if !is_sep(&p, 0) {
        let b = &p;
        let dot_component = at(b, 0) == b'.'
            && (at(b, 1) == 0
                || is_sep(b, 1)
                || (at(b, 1) == b'.' && (at(b, 2) == 0 || is_sep(b, 2))));
        if is_relative_to_current && at(b, 0) != 0 && !dot_component {
            p = [&b"./"[..], &p].concat();
        } else if !is_relative_to_current {
            // Strip a leading "./" -- one of them, though upstream's loop
            // counts however many there are.
            if at(b, 0) == b'.' && is_sep(b, 1) {
                p.drain(..2);
            }
        }
    }

    // And carries no trailing separator unless the argument did -- but "/"
    // and "//" are kept whole, which `tail_with_sep` never cuts into.
    if !has_trailing_pathsep && after_sep(&p, p.len()) {
        p.truncate(tail_with_sep(&p));
    }
    Some(p)
}
