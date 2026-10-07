//! The Vimscript builtins that name, find and touch files.
//!
//! Carved by what the builtin does with a path:
//!
//! | child | what |
//! | --- | --- |
//! | [`name`] | `fnamemodify()` and the `:h`/`:t`/`:r`/`:e`/`:p`/`:s?` modifiers |
//! | [`path`] | `resolve()`, `simplify()`, `pathshorten()`, `glob2regpat()`, `isabsolutepath()` |
//! | [`find`] | `glob()`, `globpath()`, `finddir()`, `findfile()`, `readdir()` |
//! | [`read`] | `readfile()` and `readblob()` |
//! | [`write`] | `writefile()` |
//! | [`dir`] | `chdir()`, `getcwd()`, `haslocaldir()`, `mkdir()`, `delete()`, `rename()`, `filecopy()`, `tempname()` |
//!
//! What stays here is the flag constants the children share, the one static
//! message, the safe layer they are all written against, and the predicates
//! that only *ask* the filesystem a question and rewrite nothing:
//! `executable()`, `exepath()`, `filereadable()`, `filewritable()`,
//! `getfperm()`, `getfsize()`, `getftime()`, `getftype()`, `isdirectory()`,
//! and the two `browse()` stubs.
//!
//! # The safe layer
//!
//! A builtin is handed its arguments as a slice; what the fs family adds on
//! top is the handful of coercions its builtins do to the arguments -- a
//! path as a [`CStr`], an optional flag as a Number -- and the byte
//! arithmetic over a path the children share.
//!
//! Original: `src/nvim/eval/fs.c`, Vim/Neovim, Vim license.

#![forbid(unsafe_code)]
// The globals here keep upstream's spelling; upper-casing them is a per-module rewrite.
#![allow(non_upper_case_globals)]

use crate::eval::typval::{
    NumBuf, tv_check_for_nonempty_string_arg, tv_check_for_string_arg, tv_get_number_chk,
};
use crate::mbyte::head_off;
use crate::memory::ThinCString;
use crate::message::emsg;
use crate::os::cshim::gettext;
use crate::os::fileio::FileOpenFlags;
use crate::os::fs::{
    can_execute, executable_path, file_info, link_info, os_file_is_readable, os_file_is_writable,
    os_isdir_of,
};
use crate::path::{tail_index, vim_ispathsep};
use crate::types::{Direction, EvalFuncData, FileInfo, TypVal, VAR_STRING, VarNumber, uint64_t};
use core::ffi::{CStr, c_int};

// The carve of the transpiled module; see each child's docs.
mod dir;
mod find;
mod name;
mod path;
mod read;
mod write;

pub use self::dir::*;
pub use self::find::*;
pub use self::name::*;
pub use self::path::*;
pub use self::read::*;
pub use self::write::*;

pub const kDirectionNotSet: Direction = 0;
pub const VALID_PATH: ::core::ffi::c_uint = 1;
pub const VALID_HEAD: ::core::ffi::c_uint = 2;
pub const FINDFILE_DIR: ::core::ffi::c_uint = 1;
pub const FINDFILE_FILE: ::core::ffi::c_uint = 0;
pub const kFileCreate: FileOpenFlags = 2;
pub const kFileMkDir: FileOpenFlags = 256;
pub const kFileTruncate: FileOpenFlags = 32;
pub const kFileAppend: FileOpenFlags = 64;
pub const kFileCreateOnly: FileOpenFlags = 16;
pub const kFileNoSymlink: FileOpenFlags = 8;
pub const kFileWriteOnly: FileOpenFlags = 4;
pub const kFileReadOnly: FileOpenFlags = 1;
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const SEEK_SET: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const SEEK_END: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
static e_error_while_writing_str: &::core::ffi::CStr = c"E80: Error while writing: %s";

// ---------------------------------------------------------------------
// The safe layer
// ---------------------------------------------------------------------

/// Argument `i` as a NUL-terminated path, coercing what can be coerced.
///
/// A Number argument has no string of its own, so the caller lends `buf` for
/// it to be spelled into and the answer borrows one or the other.
pub(crate) fn str_arg<'a>(args: &'a [TypVal], i: usize, buf: &'a mut NumBuf) -> &'a CStr {
    buf.string(&args[i])
}

/// Argument `i` as a NUL-terminated path, or None -- having reported the
/// error -- for a type that has no string form. As [`str_arg`], the caller
/// lends the scratch a Number is spelled into.
pub(crate) fn str_arg_chk<'a>(
    args: &'a [TypVal],
    i: usize,
    buf: &'a mut NumBuf,
) -> Option<&'a CStr> {
    buf.string_chk(&args[i])
}

/// Argument `i` as a Number, setting `error` -- and reporting one -- for a
/// type that has no number form.
pub(crate) fn nr_arg(args: &[TypVal], i: usize, error: &mut bool) -> VarNumber {
    tv_get_number_chk(&args[i]).unwrap_or_else(|_| {
        *error = true;
        0
    })
}

/// Report `msg`, translated.
pub(crate) fn err(msg: &'static CStr) {
    emsg(gettext(msg));
}

/// Byte `i` of `b`, reading its terminator -- and anything past it -- as the
/// NUL the C reads there.
///
/// Every `p[1]`/`p[2]`/`p[3]` in this family is guarded by the byte before
/// it being something other than the terminator, so a read that lands past
/// the end is one the C would answer 0 for too.
pub(crate) fn at(b: &[u8], i: usize) -> u8 {
    b.get(i).copied().unwrap_or(0)
}

/// Whether byte `i` of `b` is a path separator.
pub(crate) fn is_sep(b: &[u8], i: usize) -> bool {
    vim_ispathsep(at(b, i) as c_int)
}

/// Whether byte `i` of `b` follows a path separator -- one that is not the
/// trailing byte of a multibyte character. Upstream's `after_pathsep`.
pub(crate) fn after_sep(b: &[u8], i: usize) -> bool {
    i > 0 && is_sep(b, i - 1) && head_off(b, i - 1) == 0
}

/// Where the separators before the last component of `b` start, never
/// before the head of the path: upstream's `path_tail_with_sep`, as an
/// index.
pub(crate) fn tail_with_sep(b: &[u8]) -> usize {
    let past_head = b.iter().position(|&c| c != b'/').unwrap_or(b.len());
    let mut tail = tail_index(b);
    while tail > past_head && after_sep(b, tail) {
        tail -= 1;
    }
    tail
}

/// Where the component after the one at `at` starts in `b`: past its
/// separator, or at the end when there is none. Upstream's
/// `path_next_component`, as an index.
pub(crate) fn next_component(b: &[u8], at: usize) -> usize {
    b[at..]
        .iter()
        .position(|&c| c == b'/')
        .map_or(b.len(), |sep| at + sep + 1)
}

/// `s` from byte `from` on, which is still NUL-terminated.
pub(crate) fn from(s: &CStr, from: usize) -> &CStr {
    CStr::from_bytes_with_nul(&s.to_bytes_with_nul()[from..]).expect("one NUL, at the end")
}

// ---------------------------------------------------------------------
// The predicates
// ---------------------------------------------------------------------
//
// Everything below only *asks* the filesystem a question.  Each one is the
// builtin's own arithmetic over one of the small wrappers here, so the whole
// group's unchecked surface is the wrappers.

/// Whether argument `i` is a String, having reported if not.
fn is_string_arg(args: &[TypVal], i: usize) -> bool {
    tv_check_for_string_arg(args, i).is_ok()
}

/// Whether argument `i` is a non-empty String, having reported if not.
fn is_nonempty_string_arg(args: &[TypVal], i: usize) -> bool {
    tv_check_for_nonempty_string_arg(args, i).is_ok()
}

/// Whether `p` names something executable, looking in `$PATH` as well as
/// directly, so that a directory name answers too.
fn can_exe(p: &CStr) -> bool {
    can_execute(p, true)
}

/// Where `p`'s executable was found, or `None` when it is not one.
fn exe_path(p: &CStr) -> Option<ThinCString> {
    executable_path(p, true).map(ThinCString::from)
}

fn is_dir(p: &CStr) -> bool {
    os_isdir_of(p)
}

/// The permission bits of `p` -- the whole `st_mode`, as upstream's
/// `os_getperm` answers it -- or `None` when it has none.
fn getperm(p: &CStr) -> Option<uint64_t> {
    file_info(p).map(|info| info.stat.st_mode)
}

/// The `stat` of what `p` names, following symlinks.
fn stat(p: &CStr) -> Option<FileInfo> {
    file_info(p)
}

/// As [`stat`], but of the symlink itself rather than what it points at.
fn lstat(p: &CStr) -> Option<FileInfo> {
    link_info(p)
}

/// The size the `stat` reports.
fn size(info: &FileInfo) -> uint64_t {
    info.stat.st_size
}

/// `executable({expr})`: whether the name can be run.
pub fn f_executable(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    let mut numbuf = NumBuf::new();
    if !is_string_arg(args, 0) {
        return;
    }
    result.write_number(can_exe(str_arg(args, 0, &mut numbuf)) as VarNumber);
}

/// `exepath({expr})`: the full path of the executable, or the empty string.
pub fn f_exepath(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    let mut numbuf = NumBuf::new();
    if !is_nonempty_string_arg(args, 0) {
        return;
    }
    result.write_string(exe_path(str_arg(args, 0, &mut numbuf)));
}

/// `filereadable({file})`: whether the file exists and can be read.
pub fn f_filereadable(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    let mut numbuf = NumBuf::new();
    let p = str_arg(args, 0, &mut numbuf);
    let readable = !p.to_bytes().is_empty() && !is_dir(p) && os_file_is_readable(p);
    result.write_number(readable as VarNumber);
}

/// `filewritable({file})`: 0 for not writable, 1 for a writable file, 2 for
/// a directory that can be written into.
pub fn f_filewritable(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    let mut numbuf = NumBuf::new();
    result.write_number(os_file_is_writable(str_arg(args, 0, &mut numbuf)) as VarNumber);
}

/// `getfperm({fname})`: the permissions as `rwxrwxrwx`, or the empty string
/// when the file has none to report.
pub fn f_getfperm(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    let mut numbuf = NumBuf::new();
    let perm = getperm(str_arg(args, 0, &mut numbuf)).map(|file_perm| {
        let mut spelled = *b"---------";
        for (i, c) in spelled.iter_mut().enumerate() {
            if file_perm & (1 << (8 - i)) != 0 {
                *c = b"rwx"[i % 3];
            }
        }
        ThinCString::from_bytes(&spelled)
    });
    result.write_string(perm);
}

/// `getfsize({fname})`: the size in bytes, 0 for a directory, -1 when the
/// file cannot be measured and -2 when it does not fit in a Number.
pub fn f_getfsize(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    let mut numbuf = NumBuf::new();
    let fname = str_arg(args, 0, &mut numbuf);
    result.write_number(match stat(fname) {
        None => -1 as VarNumber,
        Some(info) => {
            let filesize = size(&info);
            let answer = filesize as VarNumber;
            if is_dir(fname) {
                0 as VarNumber
            } else if answer as uint64_t == filesize {
                answer
            } else {
                // Too big for a Number.
                -2 as VarNumber
            }
        }
    });
}

/// `getftime({fname})`: the modification time, or -1.
pub fn f_getftime(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    let mut numbuf = NumBuf::new();
    let mtime = stat(str_arg(args, 0, &mut numbuf)).map(|info| info.stat.st_mtim.tv_sec);
    result.write_number(mtime.map_or(-1 as VarNumber, |t| t as VarNumber));
}

/// `getftype({fname})`: what kind of thing the name refers to -- of the
/// symlink itself, not of what it points at.
pub fn f_getftype(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    let mut numbuf = NumBuf::new();
    result.write_empty(VAR_STRING);
    let named = lstat(str_arg(args, 0, &mut numbuf)).map(|info| {
        // The `S_IS*` family, spelled out.
        match info.stat.st_mode & __S_IFMT as uint64_t {
            0o100000 => c"file",
            0o40000 => c"dir",
            0o120000 => c"link",
            0o60000 => c"bdev",
            0o20000 => c"cdev",
            0o10000 => c"fifo",
            0o140000 => c"socket",
            _ => c"other",
        }
    });
    result.write_string(named.map(ThinCString::from_cstr));
}

/// `isdirectory({directory})`: whether the name is a directory.
pub fn f_isdirectory(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    let mut numbuf = NumBuf::new();
    result.write_number(is_dir(str_arg(args, 0, &mut numbuf)) as VarNumber);
}

/// `browse({save}, {title}, {initdir}, {default})`: a stub -- there is no
/// file dialog to open.
pub fn f_browse(_args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    result.write_string(None);
}

/// `browsedir({title}, {initdir})`: the same stub.
pub fn f_browsedir(args: &[TypVal], result: &mut TypVal, fptr: EvalFuncData) {
    f_browse(args, result, fptr);
}

pub const __S_IFMT: ::core::ffi::c_int = 0o170000 as ::core::ffi::c_int;
