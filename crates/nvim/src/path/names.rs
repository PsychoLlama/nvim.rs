//! Taking a file name apart, and putting one together.
//!
//! Everything here is about the *text* of a name: which byte the last
//! component starts at ([`tail_index`] and its neighbours), whether a byte is
//! a path separator, how to join a directory to a name ([`concat_fnames`]),
//! and whether a name is really a URL ([`path_with_url`]) or an absolute
//! path. Nothing here touches the file system — except
//! [`dir_of_file_exists`], which is here because it is a question about a
//! name's directory part.
//!
//! Upstream walks these names a character at a time. Scanning bytes finds
//! the same separators: `utf_ptr2len` only steps over a lead byte when every
//! byte after it is a continuation byte, and none of `/`, `:`, `.` or a
//! space is one.

#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use crate::cstr;
use crate::mbyte::{strcmp_ic_in, strnicmp_in};
use crate::memory::XString;
use core::ffi::c_int;
use std::ffi::CStr;

use super::*;
use crate::types::MAXPATHL;

/// Where the last component of `name` starts — upstream's `path_tail`, as an
/// index. Leading separators are the head of the path rather than part of a
/// component, so `"/foo"` has its tail at `foo`.
pub(crate) fn tail_index(name: &[u8]) -> usize {
    let head = past_head_index(name);
    name[head..]
        .iter()
        .rposition(|&b| b == b'/')
        .map_or(head, |at| head + at + 1)
}

/// The tail of `fname`: everything past its last path separator, so
/// `"dir/file.txt"` gives `"file.txt"` and `"dir/"` gives `""`.
pub fn path_tail(fname: &CStr) -> &CStr {
    cstr::suffix(fname, tail_index(fname.to_bytes()))
}

/// Where the tail of `name` starts *with* its leading path separators, so
/// `"dir///file.txt"` gives the index of `"///file.txt"`. Never past the head
/// of the path: `"/file"` keeps its one separator.
pub fn tail_with_sep_index(name: &[u8]) -> usize {
    let past_head = past_head_index(name);
    let mut tail = tail_index(name);
    while tail > past_head && after_pathsep(name, tail) {
        tail -= 1;
    }
    tail
}

/// The program name in a `"path/to/exe [args]"` invocation: everything from
/// the tail of the program's path on, its arguments included.
pub fn invocation_path_tail(invocation: &CStr) -> &CStr {
    let bytes = invocation.to_bytes();
    let past_head = past_head_index(bytes);
    let rest = &bytes[past_head..];
    // The arguments are not part of the name.
    let exe = rest.split(|&b| b == b' ').next().unwrap_or(rest);
    let at = exe.iter().rposition(|&b| b == b'/').map_or(0, |at| at + 1);
    cstr::suffix(invocation, past_head + at)
}

/// Where what is left of `name` after its first path component starts: the
/// byte past the first separator, or the end when there is none.
pub fn next_component_index(name: &[u8]) -> usize {
    name.iter()
        .position(|&b| b == b'/')
        .map_or(name.len(), |at| at + 1)
}

/// How long the head of a path is here: 1, where Windows would say 3.
pub fn path_head_length() -> usize {
    1
}

/// Does `path` begin with the head of a path — `/` here, `D:` on Windows?
pub fn is_path_head(path: &[u8]) -> bool {
    path.first().is_some_and(|&b| vim_ispathsep(c_int::from(b)))
}

/// Where `path` goes past its head: after its leading separators, which on
/// Windows would also mean after `c:`. Zero when it has no head.
pub(crate) fn past_head_index(path: &[u8]) -> usize {
    path.iter().position(|&b| b != b'/').unwrap_or(path.len())
}

/// Is `c` a path separator? On Windows this includes the colon, which is why
/// Unix does not: a colon is an ordinary character in a file name here.
pub fn vim_ispathsep(c: c_int) -> bool {
    c == c_int::from(b'/')
}

/// [`vim_ispathsep`], but never the colon.
pub fn vim_ispathsep_nocolon(c: c_int) -> bool {
    vim_ispathsep(c)
}

/// Is `c` what separates the entries of a `'path'`-like list?
pub fn vim_ispathlistsep(c: c_int) -> bool {
    c == c_int::from(b':')
}

/// Does the directory `fname` names exist? A name with no directory part
/// counts as existing.
pub fn dir_of_file_exists(fname: &[u8]) -> bool {
    let tail = tail_with_sep_index(fname);
    if tail == 0 {
        return true;
    }
    cstr::with_terminated(&fname[..tail], dir_exists)
}

/// Compare two file names, honouring `'fileignorecase'`. Each ends at its
/// end or its first NUL.
///
/// Not exact — it knows nothing about maximum name lengths or `"../dir"`,
/// and the file system may fold case by some other rule.
pub fn path_fnamecmp(fname1: &[u8], fname2: &[u8]) -> c_int {
    strcmp_ic_in(p_fic(), cstr::until_nul(fname1), cstr::until_nul(fname2))
}

/// [`path_fnamecmp`] over at most `len` bytes of each.
pub fn path_fnamencmp(fname1: &[u8], fname2: &[u8], len: usize) -> c_int {
    let (a, b) = (prefix_of(fname1, len), prefix_of(fname2, len));
    if p_fic() {
        strnicmp_in(a, b)
    } else {
        a.cmp(b) as c_int
    }
}

/// At most `len` bytes of `name`, which ends at its first NUL.
pub(crate) fn prefix_of(name: &[u8], len: usize) -> &[u8] {
    let name = cstr::until_nul(name);
    &name[..len.min(name.len())]
}

/// Join `fname1` and `fname2`, with a path separator between them if `sep`
/// is set and one is needed.
pub fn concat_fnames(fname1: &[u8], fname2: &[u8], sep: bool) -> XString {
    let mut joined = XString::with_capacity(fname1.len() + fname2.len() + 1);
    joined.push_bytes(fname1);
    if sep && !fname1.is_empty() && !after_pathsep(fname1, fname1.len()) {
        joined.push_byte(PATHSEP_BYTE);
    }
    joined.push_bytes(fname2);
    joined
}

/// The path separator, as a byte.
pub(crate) const PATHSEP_BYTE: u8 = b'/';

/// Add a path separator to `name` unless it ends in one already.
///
/// Answers false, having done nothing, only when there is no room for the
/// separator within [`MAXPATHL`].
pub fn add_pathsep(name: &mut Vec<u8>) -> bool {
    if name.is_empty() || after_pathsep(name, name.len()) {
        return true;
    }
    // The separator and the NUL after it.
    if name.len() + 2 > MAXPATHL as usize {
        return false;
    }
    name.push(PATHSEP_BYTE);
    true
}

/// [`add_pathsep`] for a name in a buffer of its own, NUL-terminated: the
/// separator goes in when the buffer — and [`MAXPATHL`] — has room for it
/// and its NUL.
pub(crate) fn add_pathsep_in(buf: &mut [u8]) -> bool {
    let len = cstr::until_nul(buf).len();
    if len == 0 || after_pathsep(buf, len) {
        return true;
    }
    if len + 2 > MAXPATHL as usize || len + 2 > buf.len() {
        return false;
    }
    buf[len] = PATHSEP_BYTE;
    buf[len + 1] = 0;
    true
}

/// Does `p` start with a Windows drive letter (`"C:/"`)?
///
/// See <https://url.spec.whatwg.org/#start-with-a-windows-drive-letter>.
pub fn path_has_drive_letter(p: &[u8]) -> bool {
    p.len() >= 2
        && p[0].is_ascii_alphabetic()
        && (p[1] == b':' || p[1] == b'|')
        && (p.len() == 2 || matches!(p[2], b'/' | b'\\' | b'?' | b'#'))
}

/// Is the `":/"` that separates a URL's scheme from its path at `p`?
///
/// Answers [`URL_SLASH`] for `":/"` and [`URL_BACKSLASH`] for `":\\"`, which
/// MS Internet Explorer accepts, and zero otherwise.
pub fn path_is_url(p: &CStr) -> c_int {
    let p = p.to_bytes();
    if p.starts_with(b":/") {
        URL_SLASH
    } else if p.starts_with(br":\\") {
        URL_BACKSLASH
    } else {
        0
    }
}

/// Does `fname` start with `"name:/"` or `"name:\\"`?
///
/// Answers what [`path_is_url`] does for the separator it found, or zero.
pub fn path_with_url(fname: &CStr) -> c_int {
    let bytes = fname.to_bytes();
    // A scheme starts with a letter — and a Windows drive letter, which
    // also does, is not a scheme.
    if !bytes.first().is_some_and(u8::is_ascii_alphabetic) || path_has_drive_letter(bytes) {
        return 0;
    }
    // The rest of the scheme is what RFC 3986 allows, and may not end in
    // a `+`, `-` or `.`.
    let end = 1 + bytes[1..]
        .iter()
        .position(|b| !(b.is_ascii_alphanumeric() || matches!(b, b'+' | b'-' | b'.')))
        .unwrap_or(bytes.len() - 1);
    if matches!(bytes[end - 1], b'+' | b'-' | b'.') {
        return 0;
    }
    path_is_url(cstr::suffix(fname, end))
}

/// Does `path` end in `extension`, ignoring the dot and honouring
/// `'fileignorecase'`?
pub fn path_with_extension(path: &CStr, extension: &CStr) -> bool {
    let bytes = path.to_bytes();
    let Some(dot) = bytes.iter().rposition(|&b| b == b'.') else {
        return false;
    };
    strcmp_ic_in(p_fic(), &bytes[dot + 1..], extension.to_bytes()) == 0
}

/// Is `name` a full (absolute) path name, or a URL?
pub fn vim_is_abs_name(name: &CStr) -> bool {
    path_with_url(name) != 0 || path_is_absolute(name)
}

/// Is `at` just past a path separator in `name`?
///
/// Upstream also asks that the separator start a character, so that a
/// separator byte inside a multibyte character is not taken for one — but a
/// `/` is ASCII, which never continues a character, so the byte alone
/// answers.
pub fn after_pathsep(name: &[u8], at: usize) -> bool {
    at > 0 && vim_ispathsep(c_int::from(name[at - 1]))
}

/// Is `fname` an absolute path? `~` counts: it names the home directory.
pub fn path_is_absolute(fname: &CStr) -> bool {
    matches!(fname.to_bytes().first(), Some(b'/' | b'~'))
}
