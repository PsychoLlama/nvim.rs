//! Taking a file name apart, and putting one together.
//!
//! Everything here is about the *text* of a name: which byte the last
//! component starts at ([`path_tail`] and its neighbours), whether a byte is
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

#![deny(unsafe_op_in_unsafe_fn)]

use core::ffi::{c_char, c_int};
use std::ffi::CStr;

use super::*;
use crate::types::MAXPATHL;

/// Where the last component of `name` starts — what [`path_tail`] answers,
/// as an index. Leading separators are the head of the path rather than part
/// of a component, so `"/foo"` has its tail at `foo`.
pub(crate) fn tail_index(name: &[u8]) -> usize {
    let head = name.iter().position(|&b| b != b'/').unwrap_or(name.len());
    name[head..]
        .iter()
        .rposition(|&b| b == b'/')
        .map_or(head, |at| head + at + 1)
}

/// The tail of `fname`: everything past its last path separator, so
/// `"dir/file.txt"` gives `"file.txt"` and `"dir/"` gives `""`.
///
/// # Safety
/// `fname` must be a NUL-terminated string, or NULL for `""`.
pub unsafe fn path_tail(fname: *const c_char) -> *mut c_char {
    unsafe {
        if fname.is_null() {
            return c"".as_ptr().cast_mut();
        }
        let at = tail_index(CStr::from_ptr(fname).to_bytes());
        fname.add(at).cast_mut()
    }
}

/// The tail of `fname` *with* its leading path separators, so
/// `"dir///file.txt"` gives `"///file.txt"`. Never past the head of the
/// path: `"/file"` keeps its one separator.
///
/// # Safety
/// `fname` must be a NUL-terminated string.
pub unsafe fn path_tail_with_sep(fname: *mut c_char) -> *mut c_char {
    unsafe {
        let past_head = get_past_head(fname);
        let mut tail = path_tail(fname);
        while tail > past_head && after_pathsep(fname, tail) != 0 {
            tail = tail.sub(1);
        }
        tail
    }
}

/// The executable in a `"path/to/exe [args]"` invocation, and — through
/// `len`, if it is not NULL — how long its name is.
///
/// # Safety
/// `invocation` must be a NUL-terminated string and `len` writable or NULL.
pub unsafe fn invocation_path_tail(invocation: *const c_char, len: *mut size_t) -> *const c_char {
    unsafe {
        let past_head = get_past_head(invocation).cast_const();
        let bytes = CStr::from_ptr(past_head).to_bytes();
        // The arguments are not part of the name.
        let exe = bytes.split(|&b| b == b' ').next().unwrap_or(bytes);
        let at = exe.iter().rposition(|&b| b == b'/').map_or(0, |at| at + 1);
        if !len.is_null() {
            *len = (exe.len() - at) as size_t;
        }
        past_head.add(at)
    }
}

/// What is left of `fname` after its first path component: the byte past the
/// first separator, or the NUL when there is none.
///
/// # Safety
/// `fname` must be a NUL-terminated string.
pub unsafe fn path_next_component(fname: *const c_char) -> *const c_char {
    unsafe {
        let bytes = CStr::from_ptr(fname).to_bytes();
        let at = bytes
            .iter()
            .position(|&b| b == b'/')
            .map_or(bytes.len(), |at| at + 1);
        fname.add(at)
    }
}

/// How long the head of a path is here: 1, where Windows would say 3.
pub fn path_head_length() -> c_int {
    1
}

/// Does `path` begin with the head of a path — `/` here, `D:` on Windows?
///
/// # Safety
/// `path` must name at least one readable byte.
pub unsafe fn is_path_head(path: *const c_char) -> bool {
    unsafe { vim_ispathsep(*path as c_int) }
}

/// One byte past the head of `path`: after its leading separators, which on
/// Windows would also mean after `c:`. `path` itself when it has no head.
///
/// # Safety
/// `path` must be a NUL-terminated string.
pub unsafe fn get_past_head(path: *const c_char) -> *mut c_char {
    unsafe {
        let bytes = CStr::from_ptr(path).to_bytes();
        let head = bytes.iter().position(|&b| b != b'/').unwrap_or(bytes.len());
        path.add(head).cast_mut()
    }
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
///
/// # Safety
/// `fname` must be a writable NUL-terminated string: the directory part is
/// terminated in place for the question, as upstream does.
pub unsafe fn dir_of_file_exists(fname: *mut c_char) -> bool {
    unsafe {
        let tail = path_tail_with_sep(fname);
        if tail == fname {
            return true;
        }
        let saved = *tail;
        *tail = 0;
        let exists = os_isdir(fname);
        *tail = saved;
        exists
    }
}

/// Compare two file names, honouring `'fileignorecase'`.
///
/// Not exact — it knows nothing about maximum name lengths or `"../dir"`,
/// and the file system may fold case by some other rule.
///
/// # Safety
/// Both must be NUL-terminated strings.
pub unsafe fn path_fnamecmp(fname1: *const c_char, fname2: *const c_char) -> c_int {
    unsafe { mb_strcmp_ic(p_fic.get() != 0, fname1, fname2) }
}

/// [`path_fnamecmp`] over at most `len` bytes.
///
/// # Safety
/// Both must name at least `len` readable bytes, up to a NUL.
pub unsafe fn path_fnamencmp(fname1: *const c_char, fname2: *const c_char, len: size_t) -> c_int {
    unsafe {
        if p_fic.get() != 0 {
            mb_strnicmp(fname1, fname2, len)
        } else {
            strncmp(fname1, fname2, len)
        }
    }
}

/// Append `fname2` to the `len1` bytes already in `fname1`, with a path
/// separator between them if `sep` is set and there is not one there
/// already. Answers `fname1`.
///
/// # Safety
/// `fname1` must hold `len1` bytes and have room for `len2 + 2` more;
/// `fname2` must hold `len2` bytes and a NUL.
pub(crate) unsafe fn do_concat_fnames(
    fname1: *mut c_char,
    len1: size_t,
    fname2: *const c_char,
    len2: size_t,
    sep: bool,
) -> *mut c_char {
    unsafe {
        let mut at = len1;
        if sep && *fname1 != 0 && after_pathsep(fname1, fname1.add(at)) == 0 {
            *fname1.add(at) = PATHSEP as c_char;
            at += 1;
        }
        // The NUL comes across with the name.
        core::ptr::copy(fname2, fname1.add(at), len2 + 1);
        fname1
    }
}

/// Join `fname1` and `fname2`, with a path separator between them if `sep`
/// is set and one is needed.
///
/// # Safety
/// Both must be NUL-terminated strings. The result is the caller's to free.
pub unsafe fn concat_fnames(
    fname1: *const c_char,
    fname2: *const c_char,
    sep: bool,
) -> *mut c_char {
    unsafe {
        let len1 = CStr::from_ptr(fname1).to_bytes().len();
        let len2 = CStr::from_ptr(fname2).to_bytes().len();
        // Room for both names, the separator, and the NUL.
        let dest: *mut c_char = xmalloc(len1 + len2 + 3).cast();
        core::ptr::copy_nonoverlapping(fname1, dest, len1 + 1);
        do_concat_fnames(dest, len1 as size_t, fname2, len2 as size_t, sep)
    }
}

/// [`concat_fnames`], but growing `fname1` in place rather than allocating.
///
/// # Safety
/// `fname1` must be an allocated NUL-terminated string, and is consumed;
/// `fname2` must be a NUL-terminated string.
pub unsafe fn concat_fnames_realloc(
    fname1: *mut c_char,
    fname2: *const c_char,
    sep: bool,
) -> *mut c_char {
    unsafe {
        let len1 = CStr::from_ptr(fname1).to_bytes().len();
        let len2 = CStr::from_ptr(fname2).to_bytes().len();
        let dest: *mut c_char = xrealloc(fname1.cast(), len1 + len2 + 3).cast();
        do_concat_fnames(dest, len1 as size_t, fname2, len2 as size_t, sep)
    }
}

/// Add a path separator to `p` unless it ends in one already.
///
/// Answers false, having done nothing, only when there is no room for the
/// separator within [`MAXPATHL`].
///
/// # Safety
/// `p` must be a NUL-terminated string in a buffer of [`MAXPATHL`] bytes.
pub unsafe fn add_pathsep(p: *mut c_char) -> bool {
    unsafe {
        let len = CStr::from_ptr(p).to_bytes().len();
        if len == 0 || after_pathsep(p, p.add(len)) != 0 {
            return true;
        }
        // The separator and the NUL after it.
        if len + 2 > MAXPATHL as usize {
            return false;
        }
        *p.add(len) = PATHSEP as c_char;
        *p.add(len + 1) = 0;
        true
    }
}

/// Does `p` start with a Windows drive letter (`"C:/"`)?
///
/// See <https://url.spec.whatwg.org/#start-with-a-windows-drive-letter>.
///
/// # Safety
/// `p` must name `path_len` readable bytes.
pub unsafe fn path_has_drive_letter(p: *const c_char, path_len: size_t) -> bool {
    // SAFETY: the caller's promise. `p` is never NULL, which the slice needs
    // even at length zero.
    let p = unsafe { core::slice::from_raw_parts(p.cast::<u8>(), path_len) };
    p.len() >= 2
        && p[0].is_ascii_alphabetic()
        && (p[1] == b':' || p[1] == b'|')
        && (p.len() == 2 || matches!(p[2], b'/' | b'\\' | b'?' | b'#'))
}

/// Is the `":/"` that separates a URL's scheme from its path at `p`?
///
/// Answers [`URL_SLASH`] for `":/"` and [`URL_BACKSLASH`] for `":\\"`, which
/// MS Internet Explorer accepts, and zero otherwise.
///
/// # Safety
/// `p` must be a NUL-terminated string.
pub unsafe fn path_is_url(p: *const c_char) -> c_int {
    // SAFETY: the caller's promise.
    let p = unsafe { CStr::from_ptr(p) }.to_bytes();
    if p.starts_with(b":/") {
        URL_SLASH as c_int
    } else if p.starts_with(br":\\") {
        URL_BACKSLASH as c_int
    } else {
        0
    }
}

/// Does `fname` start with `"name:/"` or `"name:\\"`?
///
/// Answers what [`path_is_url`] does for the separator it found, or zero.
///
/// # Safety
/// `fname` must be a NUL-terminated string.
pub unsafe fn path_with_url(fname: *const c_char) -> c_int {
    unsafe {
        let bytes = CStr::from_ptr(fname).to_bytes();
        // A scheme starts with a letter — and a Windows drive letter, which
        // also does, is not a scheme.
        if !bytes.first().is_some_and(u8::is_ascii_alphabetic)
            || path_has_drive_letter(fname, bytes.len() as size_t)
        {
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
        path_is_url(fname.add(end))
    }
}

/// Does `path` end in `extension`, ignoring the dot and honouring
/// `'fileignorecase'`?
///
/// # Safety
/// Both must be NUL-terminated strings.
pub unsafe fn path_with_extension(path: *const c_char, extension: *const c_char) -> bool {
    unsafe {
        let bytes = CStr::from_ptr(path).to_bytes();
        let Some(dot) = bytes.iter().rposition(|&b| b == b'.') else {
            return false;
        };
        mb_strcmp_ic(p_fic.get() != 0, path.add(dot + 1), extension) == 0
    }
}

/// Is `name` a full (absolute) path name, or a URL?
///
/// # Safety
/// `name` must be a NUL-terminated string.
pub unsafe fn vim_is_abs_name(name: *const c_char) -> bool {
    unsafe { path_with_url(name) != 0 || path_is_absolute(name) }
}

/// Is `p` just past a path separator? `b` must be the start of the name, so
/// that a separator byte inside a multibyte character can be told apart from
/// a real one.
///
/// # Safety
/// `b` and `p` must point into the same string, with `b` no later than `p`.
pub unsafe fn after_pathsep(b: *const c_char, p: *const c_char) -> c_int {
    unsafe {
        (p > b && vim_ispathsep(*p.sub(1) as c_int) && utf_head_off(b, p.sub(1)) == 0) as c_int
    }
}

/// Is `fname` an absolute path? `~` counts: it names the home directory.
///
/// # Safety
/// `fname` must name at least one readable byte.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn path_is_absolute(fname: *const c_char) -> bool {
    unsafe { *fname == b'/' as c_char || *fname == b'~' as c_char }
}
