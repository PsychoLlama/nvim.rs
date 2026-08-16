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
//! A builtin's call frame is the tree's own [`Args`] and [`frame`], shared
//! with every other `f_*` family; what the fs family adds on top is the
//! handful of coercions its builtins do to the arguments -- a path as a
//! [`CStr`], an optional flag as a Number -- and the two shapes of answer
//! they give back, an owned string ([`ret_string`]) or a List of them
//! ([`RetList`]).  Each carries exactly one `unsafe` line, so the builtins
//! above them are ordinary safe Rust.
//!
//! Original: `src/nvim/eval/fs.c`, Vim/Neovim, Vim license.

#![deny(unsafe_op_in_unsafe_fn)]

// The builtin call frame, shared with every other `f_*` family; named here
// so the six children reach it as `super::{Args, frame}`.
pub(crate) use crate::src::nvim::eval::funcs::args::{Args, frame};
use crate::src::nvim::eval::typval::{
    tv_check_for_nonempty_string_arg, tv_check_for_string_arg, tv_get_number_chk, tv_get_string,
    tv_get_string_buf_chk, tv_get_string_chk, tv_list_alloc_ret, tv_list_append_string,
};
use crate::src::nvim::main::c_bytes;
use crate::src::nvim::memory::{xfree, xmallocz, xmemdupz, xstrdup};
use crate::src::nvim::message::emsg;
use crate::src::nvim::os::fs::{
    os_can_exe, os_file_is_readable, os_file_is_writable, os_fileinfo, os_fileinfo_link,
    os_fileinfo_size, os_getperm, os_isdir,
};
use crate::src::nvim::os::libc::gettext;
use crate::src::nvim::path::vim_ispathsep;
use crate::src::nvim::strings::concat_str;
use crate::src::nvim::types::{
    Direction, EvalFuncData, FileInfo, VAR_NUMBER, VAR_STRING, int32_t, list_T, ptrdiff_t, size_t,
    ssize_t, typval_T, uint64_t, uv_stat_t, uv_timespec_t, varnumber_T, xp_prefix_T,
};
use core::ffi::{CStr, c_char, c_int, c_void};
use core::mem::ManuallyDrop;
use core::ptr;

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
pub const XP_PREFIX_NONE: xp_prefix_T = 0;
pub type C2Rust_Unnamed_14 = ::core::ffi::c_int;
pub const EXPAND_FILES: C2Rust_Unnamed_14 = 2;
pub type C2Rust_Unnamed_15 = ::core::ffi::c_uint;
pub const WILD_ALL_KEEP: C2Rust_Unnamed_15 = 8;
pub const WILD_ALL: C2Rust_Unnamed_15 = 6;
pub type C2Rust_Unnamed_16 = ::core::ffi::c_uint;
pub const WILD_IGNORE_COMPLETESLASH: C2Rust_Unnamed_16 = 1024;
pub const WILD_ALLLINKS: C2Rust_Unnamed_16 = 512;
pub const WILD_ICASE: C2Rust_Unnamed_16 = 256;
pub const WILD_SILENT: C2Rust_Unnamed_16 = 64;
pub const WILD_KEEP_ALL: C2Rust_Unnamed_16 = 32;
pub const WILD_USE_NL: C2Rust_Unnamed_16 = 4;
pub const VALID_PATH: C2Rust_Unnamed_17 = 1;
pub const VALID_HEAD: C2Rust_Unnamed_17 = 2;
pub const FINDFILE_DIR: C2Rust_Unnamed_18 = 1;
pub const FINDFILE_FILE: C2Rust_Unnamed_18 = 0;
pub const kFileCreate: C2Rust_Unnamed_19 = 2;
pub const kFileMkDir: C2Rust_Unnamed_19 = 256;
pub const kFileTruncate: C2Rust_Unnamed_19 = 32;
pub const kFileAppend: C2Rust_Unnamed_19 = 64;
pub type C2Rust_Unnamed_17 = ::core::ffi::c_uint;
pub type C2Rust_Unnamed_18 = ::core::ffi::c_uint;
pub type C2Rust_Unnamed_19 = ::core::ffi::c_uint;
pub const kFileCreateOnly: C2Rust_Unnamed_19 = 16;
pub const kFileNoSymlink: C2Rust_Unnamed_19 = 8;
pub const kFileWriteOnly: C2Rust_Unnamed_19 = 4;
pub const kFileReadOnly: C2Rust_Unnamed_19 = 1;
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const DEFAULT_MAXPATHL: ::core::ffi::c_int = 4096 as ::core::ffi::c_int;
pub const MAXPATHL: ::core::ffi::c_int = DEFAULT_MAXPATHL;
pub const NUL: ::core::ffi::c_int = '\0' as ::core::ffi::c_int;
pub const OK: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const FAIL: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const SEEK_SET: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const SEEK_END: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
static e_error_while_writing_str: [::core::ffi::c_char; 29] =
    c_bytes(b"E80: Error while writing: %s\0");

// ---------------------------------------------------------------------
// The safe layer
// ---------------------------------------------------------------------

/// `NUMBUFLEN`: the scratch a Number argument's decimal form is spelled
/// into, so that two coerced arguments can be held at once.
pub(crate) const NUMBUFLEN: usize = 65;

/// A fresh [`NUMBUFLEN`] scratch.
pub(crate) fn numbuf() -> [c_char; NUMBUFLEN] {
    [0; NUMBUFLEN]
}

/// Argument `i` as a NUL-terminated path, coercing what can be coerced.
pub(crate) fn str_arg<'a>(args: Args<'_>, i: usize) -> &'a CStr {
    // SAFETY: a live typval, and `tv_get_string` answers a NUL-terminated
    // string, never NULL.
    unsafe { CStr::from_ptr(tv_get_string(args.ptr(i))) }
}

/// Argument `i` as a NUL-terminated path, or None -- having reported the
/// error -- for a type that has no string form.
pub(crate) fn str_arg_chk<'a>(args: Args<'_>, i: usize) -> Option<&'a CStr> {
    // SAFETY: a live typval, and `tv_get_string_chk` answers a
    // NUL-terminated string, or NULL.
    unsafe {
        tv_get_string_chk(args.ptr(i))
            .as_ref()
            .map(|p| CStr::from_ptr(p))
    }
}

/// As [`str_arg_chk`], but a Number argument is spelled into `buf` rather
/// than into the shared static one -- what a builtin needs when it holds two
/// coerced arguments at once.
pub(crate) fn str_arg_buf<'a>(
    args: Args<'_>,
    i: usize,
    buf: &'a mut [c_char; NUMBUFLEN],
) -> Option<&'a CStr> {
    // SAFETY: a live typval and a scratch of the length the callee is
    // promised; the answer is NUL-terminated, or NULL.
    unsafe {
        tv_get_string_buf_chk(args.ptr(i), buf.as_mut_ptr())
            .as_ref()
            .map(|p| CStr::from_ptr(p))
    }
}

/// Argument `i` as a Number, setting `error` -- and reporting one -- for a
/// type that has no number form.
pub(crate) fn nr_arg(args: Args<'_>, i: usize, error: &mut bool) -> varnumber_T {
    // SAFETY: a live typval; the callee reports through `error` rather than
    // by returning a failure.
    unsafe { tv_get_number_chk(args.ptr(i), error) }
}

/// Answer the owned string `s`, or `v:_null_string` when it is NULL.
pub(crate) fn ret_string(rettv: &mut typval_T, s: *mut c_char) {
    rettv.v_type = VAR_STRING;
    // A union *write* needs no `unsafe`; the tag above is what names the arm.
    rettv.vval.v_string = s;
}

/// Report `msg`, translated.
pub(crate) fn err(msg: &CStr) {
    // SAFETY: `msg` is NUL-terminated, which is all `gettext` and `emsg` ask.
    unsafe { emsg(gettext(msg.as_ptr())) };
}

/// The List a builtin is answering.
///
/// `glob()`, `globpath()`, `findfile()` and `finddir()` decide between a
/// String and a List answer from their flags and then fill whichever they
/// chose, so the list has to be reachable from `rettv` as well as from the
/// call that made it.
#[derive(Clone, Copy)]
pub(crate) struct RetList(*mut list_T);

impl RetList {
    /// Make `rettv` a fresh List with room for `len` items, or
    /// `kListLenUnknown` when the count is not known yet.
    pub(crate) fn alloc(rettv: &mut typval_T, len: ptrdiff_t) -> Self {
        // SAFETY: `rettv` is the builtin's own cleared result slot.
        Self(unsafe { tv_list_alloc_ret(rettv, len) })
    }

    /// The List `rettv` already holds.
    pub(crate) fn of(rettv: &typval_T) -> Self {
        // SAFETY: only reached under a `VAR_LIST` tag, which is what makes
        // `v_list` the live arm.
        Self(unsafe { rettv.vval.v_list })
    }

    /// Append a copy of the NUL-terminated `s`.
    pub(crate) fn push(self, s: *const c_char) {
        // SAFETY: a live list and a NUL-terminated string, which is what a
        // length of -1 promises.
        unsafe { tv_list_append_string(self.0, s, -1 as ssize_t) };
    }
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

/// `s` from byte `from` on, which is still NUL-terminated.
pub(crate) fn from(s: &CStr, from: usize) -> &CStr {
    CStr::from_bytes_with_nul(&s.to_bytes_with_nul()[from..]).expect("one NUL, at the end")
}

/// A NUL-terminated string in nvim's heap, freed when it goes out of scope.
///
/// What upstream frees by hand before each `return`, and the reason the
/// bodies below are ordinary control flow rather than a chain of gotos.
/// [`Owned::into_raw`] is how a string that becomes a builtin's answer, or a
/// caller's buffer, leaves without being freed.
///
/// The accessors rebuild their view from the raw pointer rather than
/// borrowing `self`, because the same string is read and then written within
/// one step of `resolve()`'s loop; nothing here holds a view across a write.
pub(crate) struct Owned(pub(crate) *mut c_char);

impl Drop for Owned {
    fn drop(&mut self) {
        // SAFETY: every constructor allocates through nvim's allocator, and
        // `into_raw` is the only way to leave without freeing.
        unsafe { xfree(self.0.cast::<c_void>()) };
    }
}

impl Owned {
    /// A fresh copy of `s`.
    pub(crate) fn dup(s: &CStr) -> Self {
        // SAFETY: `s` is NUL-terminated, which is all `xstrdup` reads.
        Self(unsafe { xstrdup(s.as_ptr()) })
    }

    /// A fresh NUL-terminated copy of `len` bytes at `p`, which need not be
    /// NUL-terminated themselves -- the name a modifier has shortened is
    /// exactly that case.
    ///
    /// # Safety
    /// `p` has `len` readable bytes.
    pub(crate) unsafe fn dupz(p: *const c_char, len: usize) -> Self {
        // SAFETY: the caller's contract.
        Self(unsafe { xmemdupz(p.cast::<c_void>(), len).cast::<c_char>() })
    }

    /// `len` zeroed bytes plus a terminator slot after them.
    pub(crate) fn zeroed(len: usize) -> Self {
        // SAFETY: `xmallocz` allocates `len + 1` and zeroes the last byte.
        Self(unsafe { xmallocz(len as size_t).cast::<c_char>() })
    }

    /// `a` followed by `b`.
    pub(crate) fn cat(a: &CStr, b: &CStr) -> Self {
        // SAFETY: both are NUL-terminated, which is all `concat_str` reads.
        Self(unsafe { concat_str(a.as_ptr(), b.as_ptr()) })
    }

    /// The string, for reading.
    pub(crate) fn cstr<'a>(&self) -> &'a CStr {
        // SAFETY: the allocation holds a NUL-terminated string throughout --
        // every write either lands inside it or writes a terminator.
        unsafe { CStr::from_ptr(self.0) }
    }

    pub(crate) fn bytes<'a>(&self) -> &'a [u8] {
        self.cstr().to_bytes()
    }

    pub(crate) fn len(&self) -> usize {
        self.bytes().len()
    }

    /// Write `b` at `i`, which may be the terminator's own index.
    pub(crate) fn set(&self, i: usize, b: u8) {
        debug_assert!(i <= self.len());
        // SAFETY: `i` is inside the string or is its terminator, both of
        // which are inside the allocation.
        unsafe { *self.0.add(i) = b as c_char };
    }

    /// Give the string up to the caller, who frees it.
    pub(crate) fn into_raw(self) -> *mut c_char {
        ManuallyDrop::new(self).0
    }
}

/// A `uv_stat_t` with every field zero: what the `os_fileinfo` family fills
/// in, and what the three predicates below declare before asking.
fn no_fileinfo() -> FileInfo {
    const NO_TIME: uv_timespec_t = uv_timespec_t {
        tv_sec: 0,
        tv_nsec: 0,
    };
    FileInfo {
        stat: uv_stat_t {
            st_dev: 0,
            st_mode: 0,
            st_nlink: 0,
            st_uid: 0,
            st_gid: 0,
            st_rdev: 0,
            st_ino: 0,
            st_size: 0,
            st_blksize: 0,
            st_blocks: 0,
            st_flags: 0,
            st_gen: 0,
            st_atim: NO_TIME,
            st_mtim: NO_TIME,
            st_ctim: NO_TIME,
            st_birthtim: NO_TIME,
        },
    }
}

// ---------------------------------------------------------------------
// The predicates
// ---------------------------------------------------------------------
//
// Everything below only *asks* the filesystem a question.  Each one is the
// builtin's own arithmetic over one of the small wrappers here, so the whole
// group's unchecked surface is the wrappers.

/// Whether argument `i` is a String, having reported if not.
fn is_string_arg(args: Args<'_>, i: usize) -> bool {
    // SAFETY: the argument vector's own base, and `i` an index into it.
    unsafe { tv_check_for_string_arg(args.ptr(0), i as c_int) != FAIL }
}

/// Whether argument `i` is a non-empty String, having reported if not.
fn is_nonempty_string_arg(args: Args<'_>, i: usize) -> bool {
    // SAFETY: as [`is_string_arg`].
    unsafe { tv_check_for_nonempty_string_arg(args.ptr(0), i as c_int) != FAIL }
}

/// Whether `p` names something executable, looking in `$PATH` as well as
/// directly, so that a directory name answers too.
fn can_exe(p: &CStr) -> bool {
    // SAFETY: `p` is NUL-terminated; a null out-parameter asks for no path.
    unsafe { os_can_exe(p.as_ptr(), ptr::null_mut(), true) }
}

/// Where `p`'s executable was found, or NULL when it is not one.
fn exe_path(p: &CStr) -> *mut c_char {
    let mut path = ptr::null_mut();
    // SAFETY: `p` is NUL-terminated and `path` is this frame's own; the
    // answer is a string in nvim's heap, or NULL.
    unsafe { os_can_exe(p.as_ptr(), &raw mut path, true) };
    path
}

fn is_dir(p: &CStr) -> bool {
    // SAFETY: `p` is NUL-terminated.
    unsafe { os_isdir(p.as_ptr()) }
}

fn is_readable(p: &CStr) -> bool {
    // SAFETY: `p` is NUL-terminated.
    unsafe { os_file_is_readable(p.as_ptr()) }
}

/// 0 for not writable, 1 for a writable file, 2 for a directory that can be
/// written into.
fn writability(p: &CStr) -> c_int {
    // SAFETY: `p` is NUL-terminated.
    unsafe { os_file_is_writable(p.as_ptr()) }
}

/// The permission bits of `p`, or a negative number when it has none.
fn getperm(p: &CStr) -> int32_t {
    // SAFETY: `p` is NUL-terminated.
    unsafe { os_getperm(p.as_ptr()) }
}

/// The `stat` of what `p` names, following symlinks.
fn stat(p: &CStr) -> Option<FileInfo> {
    let mut info = no_fileinfo();
    // SAFETY: `p` is NUL-terminated and `info` is this frame's own.
    let taken = unsafe { os_fileinfo(p.as_ptr(), &raw mut info) };
    taken.then_some(info)
}

/// As [`stat`], but of the symlink itself rather than what it points at.
fn lstat(p: &CStr) -> Option<FileInfo> {
    let mut info = no_fileinfo();
    // SAFETY: as [`stat`].
    let taken = unsafe { os_fileinfo_link(p.as_ptr(), &raw mut info) };
    taken.then_some(info)
}

/// The size the `stat` reports, which is not always `st_size`.
fn size(info: &FileInfo) -> uint64_t {
    // SAFETY: a `FileInfo` the caller owns.
    unsafe { os_fileinfo_size(info) }
}

/// `executable({expr})`: whether the name can be run.
///
/// # Safety
/// `argvars` is the evaluator's own argument vector, arity 1, and `rettv` a
/// cleared result.
pub unsafe extern "C" fn f_executable(
    argvars: *mut typval_T,
    rettv: *mut typval_T,
    _fptr: EvalFuncData,
) {
    let (args, rettv) = frame!(argvars, rettv);
    if !is_string_arg(args, 0) {
        return;
    }
    rettv.vval.v_number = can_exe(str_arg(args, 0)) as varnumber_T;
}

/// `exepath({expr})`: the full path of the executable, or the empty string.
///
/// # Safety
/// As [`f_executable`].
pub unsafe extern "C" fn f_exepath(
    argvars: *mut typval_T,
    rettv: *mut typval_T,
    _fptr: EvalFuncData,
) {
    let (args, rettv) = frame!(argvars, rettv);
    if !is_nonempty_string_arg(args, 0) {
        return;
    }
    ret_string(rettv, exe_path(str_arg(args, 0)));
}

/// `filereadable({file})`: whether the file exists and can be read.
///
/// # Safety
/// As [`f_executable`].
pub unsafe extern "C" fn f_filereadable(
    argvars: *mut typval_T,
    rettv: *mut typval_T,
    _fptr: EvalFuncData,
) {
    let (args, rettv) = frame!(argvars, rettv);
    let p = str_arg(args, 0);
    let readable = !p.to_bytes().is_empty() && !is_dir(p) && is_readable(p);
    rettv.vval.v_number = readable as varnumber_T;
}

/// `filewritable({file})`: 0 for not writable, 1 for a writable file, 2 for
/// a directory that can be written into.
///
/// # Safety
/// As [`f_executable`].
pub unsafe extern "C" fn f_filewritable(
    argvars: *mut typval_T,
    rettv: *mut typval_T,
    _fptr: EvalFuncData,
) {
    let (args, rettv) = frame!(argvars, rettv);
    rettv.vval.v_number = writability(str_arg(args, 0)) as varnumber_T;
}

/// `getfperm({fname})`: the permissions as `rwxrwxrwx`, or the empty string
/// when the file has none to report.
///
/// # Safety
/// As [`f_executable`].
pub unsafe extern "C" fn f_getfperm(
    argvars: *mut typval_T,
    rettv: *mut typval_T,
    _fptr: EvalFuncData,
) {
    let (args, rettv) = frame!(argvars, rettv);
    let file_perm = getperm(str_arg(args, 0));
    let mut perm = ptr::null_mut();
    if file_perm >= 0 {
        let spelled = Owned::dup(c"---------");
        for i in 0..9 {
            if file_perm & (1 << (8 - i)) != 0 {
                spelled.set(i as usize, b"rwx"[i as usize % 3]);
            }
        }
        perm = spelled.into_raw();
    }
    ret_string(rettv, perm);
}

/// `getfsize({fname})`: the size in bytes, 0 for a directory, -1 when the
/// file cannot be measured and -2 when it does not fit in a Number.
///
/// # Safety
/// As [`f_executable`].
pub unsafe extern "C" fn f_getfsize(
    argvars: *mut typval_T,
    rettv: *mut typval_T,
    _fptr: EvalFuncData,
) {
    let (args, rettv) = frame!(argvars, rettv);
    let fname = str_arg(args, 0);
    rettv.v_type = VAR_NUMBER;
    rettv.vval.v_number = match stat(fname) {
        None => -1 as varnumber_T,
        Some(info) => {
            let filesize = size(&info);
            let answer = filesize as varnumber_T;
            if is_dir(fname) {
                0 as varnumber_T
            } else if answer as uint64_t == filesize {
                answer
            } else {
                // Too big for a Number.
                -2 as varnumber_T
            }
        }
    };
}

/// `getftime({fname})`: the modification time, or -1.
///
/// # Safety
/// As [`f_executable`].
pub unsafe extern "C" fn f_getftime(
    argvars: *mut typval_T,
    rettv: *mut typval_T,
    _fptr: EvalFuncData,
) {
    let (args, rettv) = frame!(argvars, rettv);
    let mtime = stat(str_arg(args, 0)).map(|info| info.stat.st_mtim.tv_sec);
    rettv.vval.v_number = mtime.map_or(-1 as varnumber_T, |t| t as varnumber_T);
}

/// `getftype({fname})`: what kind of thing the name refers to -- of the
/// symlink itself, not of what it points at.
///
/// # Safety
/// As [`f_executable`].
pub unsafe extern "C" fn f_getftype(
    argvars: *mut typval_T,
    rettv: *mut typval_T,
    _fptr: EvalFuncData,
) {
    let (args, rettv) = frame!(argvars, rettv);
    rettv.v_type = VAR_STRING;
    let named = lstat(str_arg(args, 0)).map(|info| {
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
    let answer = named.map_or(ptr::null_mut(), |t| Owned::dup(t).into_raw());
    rettv.vval.v_string = answer;
}

/// `isdirectory({directory})`: whether the name is a directory.
///
/// # Safety
/// As [`f_executable`].
pub unsafe extern "C" fn f_isdirectory(
    argvars: *mut typval_T,
    rettv: *mut typval_T,
    _fptr: EvalFuncData,
) {
    let (args, rettv) = frame!(argvars, rettv);
    rettv.vval.v_number = is_dir(str_arg(args, 0)) as varnumber_T;
}

/// `browse({save}, {title}, {initdir}, {default})`: a stub -- there is no
/// file dialog to open.
///
/// # Safety
/// As [`f_executable`], arity 4.
pub unsafe extern "C" fn f_browse(
    argvars: *mut typval_T,
    rettv: *mut typval_T,
    _fptr: EvalFuncData,
) {
    let (_, rettv) = frame!(argvars, rettv);
    ret_string(rettv, ptr::null_mut());
}

/// `browsedir({title}, {initdir})`: the same stub.
///
/// # Safety
/// As [`f_browse`], arity 2.
pub unsafe extern "C" fn f_browsedir(
    argvars: *mut typval_T,
    rettv: *mut typval_T,
    fptr: EvalFuncData,
) {
    // SAFETY: forwarded unchanged to a function with the same contract.
    unsafe { f_browse(argvars, rettv, fptr) };
}

pub const true_0: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const false_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const __S_IFMT: ::core::ffi::c_int = 0o170000 as ::core::ffi::c_int;
pub const READBIN: &::core::ffi::CStr = c"rb";
