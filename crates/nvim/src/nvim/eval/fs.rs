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
//! message, and the predicates that only *ask* the filesystem a question and
//! rewrite nothing: `executable()`, `exepath()`, `filereadable()`,
//! `filewritable()`, `getfperm()`, `getfsize()`, `getftime()`, `getftype()`,
//! `isdirectory()`, and the two `browse()` stubs.
//!
//! Original: `src/nvim/eval/fs.c`, Vim/Neovim, Vim license.

#![deny(unsafe_op_in_unsafe_fn)]

use crate::src::nvim::eval::typval::{
    tv_check_for_nonempty_string_arg, tv_check_for_string_arg, tv_get_string,
};
use crate::src::nvim::main::c_bytes;
use crate::src::nvim::memory::xstrdup;
use crate::src::nvim::os::fs::{
    os_can_exe, os_file_is_readable, os_file_is_writable, os_fileinfo, os_fileinfo_link,
    os_fileinfo_size, os_getperm, os_isdir,
};
use crate::src::nvim::types::{
    Direction, EvalFuncData, FileInfo, VAR_NUMBER, VAR_STRING, int32_t, typval_T, uint64_t,
    uv_stat_t, uv_timespec_t, varnumber_T, xp_prefix_T,
};

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
                if file_perm & (1 as int32_t) << 8 as ::core::ffi::c_int - i != 0 {
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
        let mut file_info: FileInfo = FileInfo {
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
                st_atim: uv_timespec_t {
                    tv_sec: 0,
                    tv_nsec: 0,
                },
                st_mtim: uv_timespec_t {
                    tv_sec: 0,
                    tv_nsec: 0,
                },
                st_ctim: uv_timespec_t {
                    tv_sec: 0,
                    tv_nsec: 0,
                },
                st_birthtim: uv_timespec_t {
                    tv_sec: 0,
                    tv_nsec: 0,
                },
            },
        };
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
        let mut file_info: FileInfo = FileInfo {
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
                st_atim: uv_timespec_t {
                    tv_sec: 0,
                    tv_nsec: 0,
                },
                st_mtim: uv_timespec_t {
                    tv_sec: 0,
                    tv_nsec: 0,
                },
                st_ctim: uv_timespec_t {
                    tv_sec: 0,
                    tv_nsec: 0,
                },
                st_birthtim: uv_timespec_t {
                    tv_sec: 0,
                    tv_nsec: 0,
                },
            },
        };
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
        let mut file_info: FileInfo = FileInfo {
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
                st_atim: uv_timespec_t {
                    tv_sec: 0,
                    tv_nsec: 0,
                },
                st_mtim: uv_timespec_t {
                    tv_sec: 0,
                    tv_nsec: 0,
                },
                st_ctim: uv_timespec_t {
                    tv_sec: 0,
                    tv_nsec: 0,
                },
                st_birthtim: uv_timespec_t {
                    tv_sec: 0,
                    tv_nsec: 0,
                },
            },
        };
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
