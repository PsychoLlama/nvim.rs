//! Reading a file into a value -- `readfile()` and `readblob()`.
//!
//! `read_file_or_blob` is the shared body: it opens the path (or reads stdin
//! for `-`), and either fills a Blob with the raw bytes or splits the text into
//! a List of lines, honouring the `b` flag's "no trailing newline", the
//! embedded-NUL-becomes-NL convention, CRLF stripping and a maximum line
//! count that may be counted from the end of the file.  `read_blob` is the
//! Blob half, including the offset/size arguments that read a slice of the
//! file rather than all of it.
//!
//! Original: `src/nvim/eval/fs.c`, Vim/Neovim, Vim license.

#![deny(unsafe_op_in_unsafe_fn)]

use super::{__S_IFMT, FAIL, NUL, OK, READBIN, SEEK_END, SEEK_SET, false_0, true_0};
use crate::semsg_c;
use crate::src::nvim::eval::typval::{
    tv_blob_alloc_ret, tv_blob_free, tv_get_number, tv_get_string, tv_list_alloc_ret,
    tv_list_append_owned_tv, tv_list_item_remove,
};
use crate::src::nvim::eval::typval::{tv_list_first, tv_list_len};
use crate::src::nvim::garray::ga_grow;
use crate::src::nvim::main::{e_cant_read_file_str, e_isadir2, e_notopen};
use crate::src::nvim::memory::{xfree, xmemdupz, xrealloc};
use crate::src::nvim::os::fs::{os_fileinfo_fd, os_fileinfo_size, os_fopen, os_isdir};
use crate::src::nvim::os::libc::{fclose, fileno, fread, fseeko, gettext, memcpy, memmove, strcmp};
use crate::src::nvim::pos::MAXLNUM;
use crate::src::nvim::types::{
    __off_t, EvalFuncData, FILE, FileInfo, VAR_STRING, VAR_UNKNOWN, VAR_UNLOCKED, blob_T, int64_t,
    kListLenUnknown, list_T, off_T, ptrdiff_t, size_t, typval_T, typval_vval_union, uint8_t,
    uint64_t, uv_stat_t, uv_timespec_t,
};

unsafe extern "C" fn read_blob(
    fd: *mut FILE,
    mut rettv: *mut typval_T,
    mut offset: off_T,
    mut size_arg: off_T,
) -> ::core::ffi::c_int {
    unsafe {
        let blob: *mut blob_T = (*rettv).vval.v_blob;
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
        if !os_fileinfo_fd(fileno(fd), &raw mut file_info) {
            return FAIL;
        }
        let mut whence: ::core::ffi::c_int = 0;
        let mut size: off_T = size_arg;
        let file_size: off_T = os_fileinfo_size(&raw mut file_info) as off_T;
        if offset >= 0 as off_T {
            if size == -1 as off_T
                || size > file_size - offset
                    && !(file_info.stat.st_mode & __S_IFMT as uint64_t == 0o20000 as uint64_t)
            {
                size = os_fileinfo_size(&raw mut file_info) as off_T - offset;
            }
            whence = SEEK_SET;
        } else {
            if -offset > file_size
                && !(file_info.stat.st_mode & __S_IFMT as uint64_t == 0o20000 as uint64_t)
            {
                offset = -file_size;
            }
            if size == -1 as off_T || size > -offset {
                size = -offset;
            }
            whence = SEEK_END;
        }
        if size <= 0 as off_T {
            return OK;
        }
        if offset != 0 as off_T && fseeko(fd, offset as __off_t, whence) != 0 as ::core::ffi::c_int
        {
            return OK;
        }
        ga_grow(&raw mut (*blob).bv_ga, size as ::core::ffi::c_int);
        (*blob).bv_ga.ga_len = size as ::core::ffi::c_int;
        if (fread(
            (*blob).bv_ga.ga_data,
            1 as size_t,
            (*blob).bv_ga.ga_len as size_t,
            fd,
        ) as size_t)
            < (*blob).bv_ga.ga_len as size_t
        {
            tv_blob_free((*rettv).vval.v_blob);
            (*rettv).vval.v_blob = ::core::ptr::null_mut::<blob_T>();
            return FAIL;
        }
        return OK;
    }
}

unsafe extern "C" fn read_file_or_blob(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut always_blob: bool,
) {
    unsafe {
        let mut binary: bool = false_0 != 0;
        let mut blob: bool = always_blob;
        let mut fd: *mut FILE = ::core::ptr::null_mut::<FILE>();
        let mut buf: [::core::ffi::c_char; 1024] = [0; 1024];
        let mut io_size: ::core::ffi::c_int =
            ::core::mem::size_of::<[::core::ffi::c_char; 1024]>() as ::core::ffi::c_int;
        let mut prev: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
        let mut prevlen: ptrdiff_t = 0 as ptrdiff_t;
        let mut prevsize: ptrdiff_t = 0 as ptrdiff_t;
        let mut maxline: int64_t = MAXLNUM as ::core::ffi::c_int as int64_t;
        let mut offset: off_T = 0 as off_T;
        let mut size: off_T = -1 as off_T;
        if (*argvars.offset(1 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
            != VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            if always_blob {
                offset = tv_get_number(argvars.offset(1 as ::core::ffi::c_int as isize)) as off_T;
                if (*argvars.offset(2 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
                    != VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
                {
                    size = tv_get_number(argvars.offset(2 as ::core::ffi::c_int as isize)) as off_T;
                }
            } else {
                if strcmp(
                    tv_get_string(argvars.offset(1 as ::core::ffi::c_int as isize)),
                    c"b".as_ptr(),
                ) == 0 as ::core::ffi::c_int
                {
                    binary = true_0 != 0;
                } else if strcmp(
                    tv_get_string(argvars.offset(1 as ::core::ffi::c_int as isize)),
                    c"B".as_ptr(),
                ) == 0 as ::core::ffi::c_int
                {
                    blob = true_0 != 0;
                }
                if (*argvars.offset(2 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
                    != VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
                {
                    maxline =
                        tv_get_number(argvars.offset(2 as ::core::ffi::c_int as isize)) as int64_t;
                }
            }
        }
        if blob {
            tv_blob_alloc_ret(rettv);
        } else {
            tv_list_alloc_ret(rettv, kListLenUnknown as ::core::ffi::c_int as ptrdiff_t);
        }
        let fname: *const ::core::ffi::c_char =
            tv_get_string(argvars.offset(0 as ::core::ffi::c_int as isize));
        if os_isdir(fname) {
            semsg_c!(
                gettext(&raw const e_isadir2 as *const ::core::ffi::c_char),
                fname,
            );
            return;
        }
        if *fname as ::core::ffi::c_int == NUL || {
            fd = os_fopen(fname, READBIN.as_ptr());
            fd.is_null()
        } {
            semsg_c!(
                gettext(&raw const e_notopen as *const ::core::ffi::c_char),
                if *fname as ::core::ffi::c_int == NUL {
                    gettext(c"<empty>".as_ptr()) as *const ::core::ffi::c_char
                } else {
                    fname
                },
            );
            return;
        }
        if blob {
            if read_blob(fd, rettv, offset, size) == FAIL {
                semsg_c!(
                    gettext(&raw const e_cant_read_file_str as *const ::core::ffi::c_char),
                    fname,
                );
            }
            fclose(fd);
            return;
        }
        let l: *mut list_T = (*rettv).vval.v_list;
        while maxline < 0 as int64_t || (tv_list_len(l) as int64_t) < maxline {
            let mut readlen: ::core::ffi::c_int = fread(
                &raw mut buf as *mut ::core::ffi::c_char as *mut ::core::ffi::c_void,
                1 as size_t,
                io_size as size_t,
                fd,
            ) as ::core::ffi::c_int;
            let mut p: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
            let mut start: *mut ::core::ffi::c_char =
                ::core::ptr::null_mut::<::core::ffi::c_char>();
            p = &raw mut buf as *mut ::core::ffi::c_char;
            start = &raw mut buf as *mut ::core::ffi::c_char;
            while p < (&raw mut buf as *mut ::core::ffi::c_char).offset(readlen as isize)
                || readlen <= 0 as ::core::ffi::c_int
                    && (prevlen > 0 as ptrdiff_t || binary as ::core::ffi::c_int != 0)
            {
                if readlen <= 0 as ::core::ffi::c_int
                    || *p as ::core::ffi::c_int == '\n' as ::core::ffi::c_int
                {
                    let mut s: *mut ::core::ffi::c_char =
                        ::core::ptr::null_mut::<::core::ffi::c_char>();
                    let mut len: size_t = p.offset_from(start) as size_t;
                    if readlen > 0 as ::core::ffi::c_int && !binary {
                        while len > 0 as size_t
                            && *start.add(len.wrapping_sub(1 as size_t)) as ::core::ffi::c_int
                                == '\r' as ::core::ffi::c_int
                        {
                            len = len.wrapping_sub(1);
                        }
                        if len == 0 as size_t {
                            while prevlen > 0 as ptrdiff_t
                                && *prev.offset((prevlen - 1 as ptrdiff_t) as isize)
                                    as ::core::ffi::c_int
                                    == '\r' as ::core::ffi::c_int
                            {
                                prevlen -= 1;
                            }
                        }
                    }
                    if prevlen == 0 as ptrdiff_t {
                        debug_assert!(
                            len < 2147483647 as ::core::ffi::c_int as size_t,
                            "len < INT_MAX"
                        );
                        s = xmemdupz(start as *const ::core::ffi::c_void, len)
                            as *mut ::core::ffi::c_char;
                    } else {
                        s = xrealloc(
                            prev as *mut ::core::ffi::c_void,
                            (prevlen as size_t)
                                .wrapping_add(len)
                                .wrapping_add(1 as size_t),
                        ) as *mut ::core::ffi::c_char;
                        memcpy(
                            s.offset(prevlen as isize) as *mut ::core::ffi::c_void,
                            start as *const ::core::ffi::c_void,
                            len,
                        );
                        *s.add((prevlen as size_t).wrapping_add(len)) = NUL as ::core::ffi::c_char;
                        prev = ::core::ptr::null_mut::<::core::ffi::c_char>();
                        prevsize = 0 as ptrdiff_t;
                        prevlen = prevsize;
                    }
                    tv_list_append_owned_tv(
                        l,
                        typval_T {
                            v_type: VAR_STRING,
                            v_lock: VAR_UNLOCKED,
                            vval: typval_vval_union { v_string: s },
                        },
                    );
                    start = p.offset(1 as ::core::ffi::c_int as isize);
                    if maxline < 0 as int64_t {
                        if tv_list_len(l) as int64_t > -maxline {
                            debug_assert!(
                                tv_list_len(l) as int64_t == 1 as int64_t + -maxline,
                                "tv_list_len(l) == 1 + (-maxline)"
                            );
                            tv_list_item_remove(l, tv_list_first(l));
                        }
                    } else if tv_list_len(l) as int64_t >= maxline {
                        debug_assert!(
                            tv_list_len(l) as int64_t == maxline,
                            "tv_list_len(l) == maxline"
                        );
                        break;
                    }
                    if readlen <= 0 as ::core::ffi::c_int {
                        break;
                    }
                } else if *p as ::core::ffi::c_int == NUL {
                    *p = '\n' as ::core::ffi::c_char;
                } else if *p as uint8_t as ::core::ffi::c_int == 0xbf as ::core::ffi::c_int
                    && !binary
                {
                    let mut back1: ::core::ffi::c_char = (if p
                        >= (&raw mut buf as *mut ::core::ffi::c_char)
                            .offset(1 as ::core::ffi::c_int as isize)
                    {
                        *p.offset(-1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                    } else if prevlen >= 1 as ptrdiff_t {
                        *prev.offset((prevlen - 1 as ptrdiff_t) as isize) as ::core::ffi::c_int
                    } else {
                        NUL
                    })
                        as ::core::ffi::c_char;
                    let mut back2: ::core::ffi::c_char = (if p
                        >= (&raw mut buf as *mut ::core::ffi::c_char)
                            .offset(2 as ::core::ffi::c_int as isize)
                    {
                        *p.offset(-2 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                    } else if p
                        == (&raw mut buf as *mut ::core::ffi::c_char)
                            .offset(1 as ::core::ffi::c_int as isize)
                        && prevlen >= 1 as ptrdiff_t
                    {
                        *prev.offset((prevlen - 1 as ptrdiff_t) as isize) as ::core::ffi::c_int
                    } else if prevlen >= 2 as ptrdiff_t {
                        *prev.offset((prevlen - 2 as ptrdiff_t) as isize) as ::core::ffi::c_int
                    } else {
                        NUL
                    })
                        as ::core::ffi::c_char;
                    if back2 as uint8_t as ::core::ffi::c_int == 0xef as ::core::ffi::c_int
                        && back1 as uint8_t as ::core::ffi::c_int == 0xbb as ::core::ffi::c_int
                    {
                        let mut dest: *mut ::core::ffi::c_char =
                            p.offset(-(2 as ::core::ffi::c_int as isize));
                        if start == dest {
                            start = p.offset(1 as ::core::ffi::c_int as isize);
                        } else {
                            let mut adjust_prevlen: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                            if dest < &raw mut buf as *mut ::core::ffi::c_char {
                                adjust_prevlen = (&raw mut buf as *mut ::core::ffi::c_char)
                                    .offset_from(dest)
                                    as ::core::ffi::c_int;
                                dest = &raw mut buf as *mut ::core::ffi::c_char;
                            }
                            if readlen as isize
                                > p.offset_from(&raw mut buf as *mut ::core::ffi::c_char) + 1_isize
                            {
                                memmove(
                                    dest as *mut ::core::ffi::c_void,
                                    p.offset(1 as ::core::ffi::c_int as isize)
                                        as *const ::core::ffi::c_void,
                                    (readlen as size_t)
                                        .wrapping_sub(
                                            p.offset_from(&raw mut buf as *mut ::core::ffi::c_char)
                                                as size_t,
                                        )
                                        .wrapping_sub(1 as size_t),
                                );
                            }
                            readlen -= 3 as ::core::ffi::c_int - adjust_prevlen;
                            prevlen -= adjust_prevlen as ptrdiff_t;
                            p = dest.offset(-(1 as ::core::ffi::c_int as isize));
                        }
                    }
                }
                p = p.offset(1);
            }
            if maxline >= 0 as int64_t && tv_list_len(l) as int64_t >= maxline
                || readlen <= 0 as ::core::ffi::c_int
            {
                break;
            }
            if start < p {
                if p.offset_from(start) + prevlen as isize >= prevsize {
                    if prevsize == 0 as ptrdiff_t {
                        prevsize = p.offset_from(start) as ptrdiff_t;
                    } else {
                        let mut grow50pc: ptrdiff_t = prevsize * 3 as ptrdiff_t / 2 as ptrdiff_t;
                        let mut growmin: ptrdiff_t =
                            p.offset_from(start) * 2 as ptrdiff_t + prevlen;
                        prevsize = if grow50pc > growmin {
                            grow50pc
                        } else {
                            growmin
                        };
                    }
                    prev = xrealloc(prev as *mut ::core::ffi::c_void, prevsize as size_t)
                        as *mut ::core::ffi::c_char;
                }
                memmove(
                    prev.offset(prevlen as isize) as *mut ::core::ffi::c_void,
                    start as *const ::core::ffi::c_void,
                    p.offset_from(start) as size_t,
                );
                prevlen = (prevlen as ::core::ffi::c_long
                    + p.offset_from(start) as ::core::ffi::c_long)
                    as ptrdiff_t;
            }
        }
        xfree(prev as *mut ::core::ffi::c_void);
        fclose(fd);
    }
}

pub unsafe extern "C" fn f_readblob(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    unsafe {
        read_file_or_blob(argvars, rettv, true_0 != 0);
    }
}

pub unsafe extern "C" fn f_readfile(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    unsafe {
        read_file_or_blob(argvars, rettv, false_0 != 0);
    }
}
