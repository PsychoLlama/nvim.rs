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

pub unsafe extern "C" fn f_executable(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    unsafe {
        if tv_check_for_string_arg(argvars, 0 as ::core::ffi::c_int) == FAIL {
            return;
        }
        (*rettv).vval.v_number = os_can_exe(
            tv_get_string(argvars.offset(0 as ::core::ffi::c_int as isize)),
            ::core::ptr::null_mut::<*mut ::core::ffi::c_char>(),
            true_0 != 0,
        ) as varnumber_T;
    }
}
pub unsafe extern "C" fn f_exepath(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    unsafe {
        if tv_check_for_nonempty_string_arg(argvars, 0 as ::core::ffi::c_int) == FAIL {
            return;
        }
        let mut path: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
        os_can_exe(
            tv_get_string(argvars.offset(0 as ::core::ffi::c_int as isize)),
            &raw mut path,
            true_0 != 0,
        );
        (*rettv).v_type = VAR_STRING;
        (*rettv).vval.v_string = path;
    }
}
pub unsafe extern "C" fn f_filereadable(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    unsafe {
        let p: *const ::core::ffi::c_char =
            tv_get_string(argvars.offset(0 as ::core::ffi::c_int as isize));
        (*rettv).vval.v_number = (*p as ::core::ffi::c_int != 0
            && !os_isdir(p)
            && os_file_is_readable(p) as ::core::ffi::c_int != 0)
            as ::core::ffi::c_int as varnumber_T;
    }
}
pub unsafe extern "C" fn f_filewritable(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    unsafe {
        let mut filename: *const ::core::ffi::c_char =
            tv_get_string(argvars.offset(0 as ::core::ffi::c_int as isize));
        (*rettv).vval.v_number = os_file_is_writable(filename) as varnumber_T;
    }
}
pub unsafe extern "C" fn f_getfperm(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    unsafe {
        let mut perm: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
        let mut flags: [::core::ffi::c_char; 4] =
            ::core::mem::transmute::<[u8; 4], [::core::ffi::c_char; 4]>(*b"rwx\0");
        let mut filename: *const ::core::ffi::c_char =
            tv_get_string(argvars.offset(0 as ::core::ffi::c_int as isize));
        let mut file_perm: int32_t = os_getperm(filename);
        if file_perm >= 0 as int32_t {
            perm = xstrdup(c"---------".as_ptr());
            let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
            while i < 9 as ::core::ffi::c_int {
                if file_perm & ((1 as int32_t) << (8 as ::core::ffi::c_int - i)) != 0 {
                    *perm.offset(i as isize) = flags[(i % 3 as ::core::ffi::c_int) as usize];
                }
                i += 1;
            }
        }
        (*rettv).v_type = VAR_STRING;
        (*rettv).vval.v_string = perm;
    }
}
pub unsafe extern "C" fn f_getfsize(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    unsafe {
        let mut fname: *const ::core::ffi::c_char =
            tv_get_string(argvars.offset(0 as ::core::ffi::c_int as isize));
        (*rettv).v_type = VAR_NUMBER;
        let mut file_info: FileInfo = no_fileinfo();
        if os_fileinfo(fname, &raw mut file_info) {
            let mut filesize: uint64_t = os_fileinfo_size(&raw mut file_info);
            if os_isdir(fname) {
                (*rettv).vval.v_number = 0 as varnumber_T;
            } else {
                (*rettv).vval.v_number = filesize as varnumber_T;
                if (*rettv).vval.v_number as uint64_t != filesize {
                    (*rettv).vval.v_number = -2 as varnumber_T;
                }
            }
        } else {
            (*rettv).vval.v_number = -1 as varnumber_T;
        };
    }
}
pub unsafe extern "C" fn f_getftime(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    unsafe {
        let mut fname: *const ::core::ffi::c_char =
            tv_get_string(argvars.offset(0 as ::core::ffi::c_int as isize));
        let mut file_info: FileInfo = no_fileinfo();
        if os_fileinfo(fname, &raw mut file_info) {
            (*rettv).vval.v_number = file_info.stat.st_mtim.tv_sec as varnumber_T;
        } else {
            (*rettv).vval.v_number = -1 as varnumber_T;
        };
    }
}
pub unsafe extern "C" fn f_getftype(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    unsafe {
        let mut type_0: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
        let mut t: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
        let mut fname: *const ::core::ffi::c_char =
            tv_get_string(argvars.offset(0 as ::core::ffi::c_int as isize));
        (*rettv).v_type = VAR_STRING;
        let mut file_info: FileInfo = no_fileinfo();
        if os_fileinfo_link(fname, &raw mut file_info) {
            let mut mode: uint64_t = file_info.stat.st_mode;
            if mode & __S_IFMT as uint64_t == 0o100000 as uint64_t {
                t = c"file".as_ptr() as *mut ::core::ffi::c_char;
            } else if mode & __S_IFMT as uint64_t == 0o40000 as uint64_t {
                t = c"dir".as_ptr() as *mut ::core::ffi::c_char;
            } else if mode & __S_IFMT as uint64_t == 0o120000 as uint64_t {
                t = c"link".as_ptr() as *mut ::core::ffi::c_char;
            } else if mode & __S_IFMT as uint64_t == 0o60000 as uint64_t {
                t = c"bdev".as_ptr() as *mut ::core::ffi::c_char;
            } else if mode & __S_IFMT as uint64_t == 0o20000 as uint64_t {
                t = c"cdev".as_ptr() as *mut ::core::ffi::c_char;
            } else if mode & __S_IFMT as uint64_t == 0o10000 as uint64_t {
                t = c"fifo".as_ptr() as *mut ::core::ffi::c_char;
            } else if mode & __S_IFMT as uint64_t == 0o140000 as uint64_t {
                t = c"socket".as_ptr() as *mut ::core::ffi::c_char;
            } else {
                t = c"other".as_ptr() as *mut ::core::ffi::c_char;
            }
            type_0 = xstrdup(t);
        }
        (*rettv).vval.v_string = type_0;
    }
}
pub unsafe extern "C" fn f_isdirectory(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    unsafe {
        (*rettv).vval.v_number = os_isdir(tv_get_string(
            argvars.offset(0 as ::core::ffi::c_int as isize),
        )) as varnumber_T;
    }
}
pub unsafe extern "C" fn f_browse(
    mut _argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    unsafe {
        (*rettv).vval.v_string = ::core::ptr::null_mut::<::core::ffi::c_char>();
        (*rettv).v_type = VAR_STRING;
    }
}
pub unsafe extern "C" fn f_browsedir(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut fptr: EvalFuncData,
) {
    unsafe {
        f_browse(argvars, rettv, fptr);
    }
}
pub const true_0: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const false_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const __S_IFMT: ::core::ffi::c_int = 0o170000 as ::core::ffi::c_int;
pub const READBIN: [::core::ffi::c_char; 3] =
    unsafe { ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(*b"rb\0") };
