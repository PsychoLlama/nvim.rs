use crate::src::nvim::ascii::ascii_isdigit;
use crate::src::nvim::charset::{backslash_halve, backslash_halve_save, rem_backslash, skipwhite};
use crate::src::nvim::cmdexpand::globpath;
use crate::src::nvim::eval::eval_to_string;
use crate::src::nvim::ex_docmd::eval_vars;
use crate::src::nvim::fileio::{file_pat_to_reg_pat, match_file_list};
use crate::src::nvim::garray::{
    ga_clear_strings, ga_concat_strings, ga_grow, ga_init, ga_remove_duplicate_strings,
};
use crate::src::nvim::global_cell::GlobalCell;
use crate::src::nvim::main::{
    NameBuff, curbuf, emsg_off, emsg_silent, got_int, p_cdpath, p_fic, p_path, p_su, p_wig,
};
use crate::src::nvim::mbyte::{
    mb_isalpha, mb_strcmp_ic, mb_strnicmp, mb_toupper, utf_head_off, utf_ptr2char, utfc_ptr2len,
};
use crate::src::nvim::memory::{
    xcalloc, xfree, xmalloc, xmemcpyz, xmemdupz, xrealloc, xstrdup, xstrlcat, xstrlcpy,
};
use crate::src::nvim::option::copy_option_part;
use crate::src::nvim::os::env::{expand_env, expand_env_save_opt, os_getenv, vim_env_iter};
use crate::src::nvim::os::fs::{
    os_can_exe, os_closedir, os_dirname, os_file_is_readable, os_fileid, os_fileid_equal,
    os_fileinfo, os_fileinfo_id_equal, os_fileinfo_link, os_isdir, os_path_exists, os_realpath,
    os_scandir, os_scandir_next,
};
use crate::src::nvim::os::input::os_breakcheck;
use crate::src::nvim::os::libc::{
    __assert_fail, memcpy, memmove, qsort, strcasecmp, strchr, strcmp, strcpy, strlen, strncmp,
    strrchr,
};
use crate::src::nvim::os::shell::{get_cmd_output, os_expand_wildcards};
use crate::src::nvim::strings::{concat_str, vim_snprintf, vim_strchr};
use crate::src::nvim::types::{
    Directory, FileComparison, FileID, FileInfo, colnr_T, file_comparison, garray_T, linenr_T,
    regmatch_T, regprog_T, size_t, uint8_t, uv__queue, uv__work, uv_buf_t, uv_dirent_t,
    uv_dirent_type_t, uv_fs_t, uv_fs_type, uv_loop_s, uv_loop_t, uv_req_type, uv_stat_t,
    uv_timespec_t,
};

// The carve of the transpiled module; see each child's docs.
mod names;
pub use self::names::*;
mod compare;
pub use self::compare::*;
mod unique;
pub use self::unique::*;
mod glob;
pub use self::glob::*;
mod expand;
pub use self::expand::*;
unsafe extern "C" {
    fn vim_regcomp(
        expr_arg: *const ::core::ffi::c_char,
        re_flags: ::core::ffi::c_int,
    ) -> *mut regprog_T;
    fn vim_regfree(prog: *mut regprog_T);
    fn vim_regexec(rmp: *mut regmatch_T, line: *const ::core::ffi::c_char, col: colnr_T) -> bool;
}
pub const UV_UNKNOWN_REQ: uv_req_type = 0;
pub const UV_DIRENT_UNKNOWN: uv_dirent_type_t = 0;
pub const UV_FS_CUSTOM: uv_fs_type = 0;
pub type C2Rust_Unnamed_18 = ::core::ffi::c_uint;
pub const WILD_ICASE: C2Rust_Unnamed_18 = 256;
pub const WILD_ADD_SLASH: C2Rust_Unnamed_18 = 16;
pub type C2Rust_Unnamed_19 = ::core::ffi::c_uint;
pub const kShellOptSilent: C2Rust_Unnamed_19 = 8;
pub type C2Rust_Unnamed_20 = ::core::ffi::c_uint;
pub const EW_NOBREAK: C2Rust_Unnamed_20 = 262144;
pub const EW_CDPATH: C2Rust_Unnamed_20 = 131072;
pub const EW_NOTENV: C2Rust_Unnamed_20 = 65536;
pub const EW_EMPTYOK: C2Rust_Unnamed_20 = 32768;
pub const EW_DODOT: C2Rust_Unnamed_20 = 16384;
pub const EW_SHELLCMD: C2Rust_Unnamed_20 = 8192;
pub const EW_ALLLINKS: C2Rust_Unnamed_20 = 4096;
pub const EW_KEEPDOLLAR: C2Rust_Unnamed_20 = 2048;
pub const EW_NOTWILD: C2Rust_Unnamed_20 = 1024;
pub const EW_NOERROR: C2Rust_Unnamed_20 = 512;
pub const EW_ICASE: C2Rust_Unnamed_20 = 256;
pub const EW_PATH: C2Rust_Unnamed_20 = 128;
pub const EW_EXEC: C2Rust_Unnamed_20 = 64;
pub const EW_SILENT: C2Rust_Unnamed_20 = 32;
pub const EW_KEEPALL: C2Rust_Unnamed_20 = 16;
pub const EW_ADDSLASH: C2Rust_Unnamed_20 = 8;
pub const EW_NOTFOUND: C2Rust_Unnamed_20 = 4;
pub const EW_FILE: C2Rust_Unnamed_20 = 2;
pub const EW_DIR: C2Rust_Unnamed_20 = 1;
pub const kEqualFileNames: file_comparison = 7;
pub const kOneFileMissing: file_comparison = 6;
pub const kBothFilesMissing: file_comparison = 4;
pub const kDifferentFiles: file_comparison = 2;
pub const kEqualFiles: file_comparison = 1;
pub const URL_BACKSLASH: C2Rust_Unnamed_21 = 2;
pub const URL_SLASH: C2Rust_Unnamed_21 = 1;
pub type C2Rust_Unnamed_21 = ::core::ffi::c_uint;
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const DEFAULT_MAXPATHL: ::core::ffi::c_int = 4096 as ::core::ffi::c_int;
pub const MAXPATHL: ::core::ffi::c_int = DEFAULT_MAXPATHL;
pub const NUL: ::core::ffi::c_int = '\0' as ::core::ffi::c_int;
pub const PATHSEP: ::core::ffi::c_int = '/' as ::core::ffi::c_int;
pub const PATHSEPSTR: [::core::ffi::c_char; 2] =
    unsafe { ::core::mem::transmute::<[u8; 2], [::core::ffi::c_char; 2]>(*b"/\0") };
pub const OK: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const FAIL: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub unsafe extern "C" fn FullName_save(
    mut fname: *const ::core::ffi::c_char,
    mut force: bool,
) -> *mut ::core::ffi::c_char {
    if fname.is_null() {
        return ::core::ptr::null_mut::<::core::ffi::c_char>();
    }
    let mut buf: *mut ::core::ffi::c_char = xmalloc(MAXPATHL as size_t) as *mut ::core::ffi::c_char;
    if vim_FullName(fname, buf, MAXPATHL as size_t, force) == FAIL {
        xfree(buf as *mut ::core::ffi::c_void);
        return xstrdup(fname);
    }
    return buf;
}
pub unsafe extern "C" fn save_abs_path(
    mut name: *const ::core::ffi::c_char,
) -> *mut ::core::ffi::c_char {
    if !path_is_absolute(name) {
        return FullName_save(name, true_0 != 0);
    }
    return xstrdup(name);
}
pub unsafe extern "C" fn simplify_filename(mut filename: *mut ::core::ffi::c_char) -> size_t {
    let mut components: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut stripping_disabled: bool = false_0 != 0;
    let mut relative: bool = true_0 != 0;
    let mut p: *mut ::core::ffi::c_char = filename;
    if vim_ispathsep(*p as ::core::ffi::c_int) {
        relative = false_0 != 0;
        loop {
            p = p.offset(1);
            if !vim_ispathsep(*p as ::core::ffi::c_int) {
                break;
            }
        }
    }
    let mut start: *mut ::core::ffi::c_char = p;
    let mut p_end: *mut ::core::ffi::c_char = p.offset(strlen(p) as isize);
    if start > filename.offset(2 as ::core::ffi::c_int as isize) {
        memmove(
            filename.offset(1 as ::core::ffi::c_int as isize) as *mut ::core::ffi::c_void,
            p as *const ::core::ffi::c_void,
            (p_end.offset_from(p) as size_t).wrapping_add(1 as size_t),
        );
        p_end = p_end.offset(
            -(p.offset_from(filename.offset(1 as ::core::ffi::c_int as isize)) as size_t as isize),
        );
        p = filename.offset(1 as ::core::ffi::c_int as isize);
        start = p;
    }
    loop {
        if vim_ispathsep(*p as ::core::ffi::c_int) {
            memmove(
                p as *mut ::core::ffi::c_void,
                p.offset(1 as ::core::ffi::c_int as isize) as *const ::core::ffi::c_void,
                (p_end.offset_from(p.offset(1 as ::core::ffi::c_int as isize)) as size_t)
                    .wrapping_add(1 as size_t),
            );
            p_end = p_end.offset(-1);
        } else if *p.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
            == '.' as ::core::ffi::c_int
            && (vim_ispathsep(*p.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int)
                as ::core::ffi::c_int
                != 0
                || *p.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == NUL)
        {
            if p == start && relative as ::core::ffi::c_int != 0 {
                p = p.offset(
                    (1 as ::core::ffi::c_int
                        + (*p.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int != NUL)
                            as ::core::ffi::c_int) as isize,
                );
            } else {
                let mut tail: *mut ::core::ffi::c_char = p.offset(1 as ::core::ffi::c_int as isize);
                if *p.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int != NUL {
                    while vim_ispathsep(*tail as ::core::ffi::c_int) {
                        tail = tail.offset(utfc_ptr2len(tail) as isize);
                    }
                } else if p > start {
                    p = p.offset(-1);
                }
                memmove(
                    p as *mut ::core::ffi::c_void,
                    tail as *const ::core::ffi::c_void,
                    (p_end.offset_from(tail) as size_t).wrapping_add(1 as size_t),
                );
                p_end = p_end.offset(-(tail.offset_from(p) as size_t as isize));
            }
        } else if *p.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
            == '.' as ::core::ffi::c_int
            && *p.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                == '.' as ::core::ffi::c_int
            && (vim_ispathsep(*p.offset(2 as ::core::ffi::c_int as isize) as ::core::ffi::c_int)
                as ::core::ffi::c_int
                != 0
                || *p.offset(2 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == NUL)
        {
            let mut tail_0: *mut ::core::ffi::c_char = p.offset(2 as ::core::ffi::c_int as isize);
            while vim_ispathsep(*tail_0 as ::core::ffi::c_int) {
                tail_0 = tail_0.offset(utfc_ptr2len(tail_0) as isize);
            }
            if components > 0 as ::core::ffi::c_int {
                let mut do_strip: bool = false_0 != 0;
                if !stripping_disabled {
                    let mut saved_char: ::core::ffi::c_char =
                        *p.offset(-1 as ::core::ffi::c_int as isize);
                    *p.offset(-1 as ::core::ffi::c_int as isize) = NUL as ::core::ffi::c_char;
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
                    if !os_fileinfo_link(filename, &raw mut file_info) {
                        do_strip = true_0 != 0;
                    }
                    *p.offset(-1 as ::core::ffi::c_int as isize) = saved_char;
                    p = p.offset(-1);
                    while p > start && after_pathsep(start, p) == 0 {
                        p = p.offset(
                            -((utf_head_off(start, p.offset(-(1 as ::core::ffi::c_int as isize)))
                                + 1 as ::core::ffi::c_int) as isize),
                        );
                    }
                    if !do_strip {
                        saved_char = *tail_0;
                        *tail_0 = NUL as ::core::ffi::c_char;
                        if os_fileinfo(filename, &raw mut file_info) {
                            do_strip = true_0 != 0;
                        } else {
                            stripping_disabled = true_0 != 0;
                        }
                        *tail_0 = saved_char;
                        if do_strip {
                            let mut new_file_info: FileInfo = FileInfo {
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
                            if p == start && relative as ::core::ffi::c_int != 0 {
                                os_fileinfo(
                                    b".\0".as_ptr() as *const ::core::ffi::c_char,
                                    &raw mut new_file_info,
                                );
                            } else {
                                saved_char = *p;
                                *p = NUL as ::core::ffi::c_char;
                                os_fileinfo(filename, &raw mut new_file_info);
                                *p = saved_char;
                            }
                            if !os_fileinfo_id_equal(&raw mut file_info, &raw mut new_file_info) {
                                do_strip = false_0 != 0;
                            }
                        }
                    }
                }
                if !do_strip {
                    p = tail_0;
                    components = 0 as ::core::ffi::c_int;
                } else {
                    if p == start
                        && relative as ::core::ffi::c_int != 0
                        && *tail_0.offset(-1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                            == '.' as ::core::ffi::c_int
                    {
                        let c2rust_fresh4 = p;
                        p = p.offset(1);
                        *c2rust_fresh4 = '.' as ::core::ffi::c_char;
                        *p = NUL as ::core::ffi::c_char;
                    } else {
                        if p > start
                            && *tail_0.offset(-1 as ::core::ffi::c_int as isize)
                                as ::core::ffi::c_int
                                == '.' as ::core::ffi::c_int
                        {
                            p = p.offset(-1);
                        }
                        memmove(
                            p as *mut ::core::ffi::c_void,
                            tail_0 as *const ::core::ffi::c_void,
                            (p_end.offset_from(tail_0) as size_t).wrapping_add(1 as size_t),
                        );
                        p_end = p_end.offset(-(tail_0.offset_from(p) as size_t as isize));
                    }
                    components -= 1;
                }
            } else if p == start && !relative {
                memmove(
                    p as *mut ::core::ffi::c_void,
                    tail_0 as *const ::core::ffi::c_void,
                    (p_end.offset_from(tail_0) as size_t).wrapping_add(1 as size_t),
                );
                p_end = p_end.offset(-(tail_0.offset_from(p) as size_t as isize));
            } else {
                if p == start.offset(2 as ::core::ffi::c_int as isize)
                    && *p.offset(-2 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                        == '.' as ::core::ffi::c_int
                {
                    memmove(
                        p.offset(-(2 as ::core::ffi::c_int as isize)) as *mut ::core::ffi::c_void,
                        p as *const ::core::ffi::c_void,
                        (p_end.offset_from(p) as size_t).wrapping_add(1 as size_t),
                    );
                    p_end = p_end.offset(-(2 as ::core::ffi::c_int as isize));
                    tail_0 = tail_0.offset(-(2 as ::core::ffi::c_int as isize));
                }
                p = tail_0;
            }
        } else {
            components += 1;
            p = path_next_component(p) as *mut ::core::ffi::c_char;
        }
        if *p as ::core::ffi::c_int == NUL {
            break;
        }
    }
    return p_end.offset_from(filename) as size_t;
}
#[unsafe(no_mangle)]
pub unsafe extern "C" fn vim_FullName(
    mut fname: *const ::core::ffi::c_char,
    mut buf: *mut ::core::ffi::c_char,
    mut len: size_t,
    mut force: bool,
) -> ::core::ffi::c_int {
    *buf = NUL as ::core::ffi::c_char;
    if fname.is_null() {
        return FAIL;
    }
    if strlen(fname) > len.wrapping_sub(1 as size_t) {
        xstrlcpy(buf, fname, len);
        return FAIL;
    }
    if path_with_url(fname) != 0 {
        xstrlcpy(buf, fname, len);
        return OK;
    }
    let mut rv: ::core::ffi::c_int = path_to_absolute(fname, buf, len, force as ::core::ffi::c_int);
    if rv == FAIL {
        xstrlcpy(buf, fname, len);
    }
    return rv;
}
pub unsafe extern "C" fn fix_fname(
    mut fname: *const ::core::ffi::c_char,
) -> *mut ::core::ffi::c_char {
    return FullName_save(fname, true_0 != 0);
}
pub const MAXSUFLEN: ::core::ffi::c_int = 30 as ::core::ffi::c_int;
#[unsafe(no_mangle)]
pub unsafe extern "C" fn path_full_dir_name(
    mut directory: *mut ::core::ffi::c_char,
    mut buffer: *mut ::core::ffi::c_char,
    mut len: size_t,
) -> ::core::ffi::c_int {
    if strlen(directory) == 0 as size_t {
        return os_dirname(buffer, len);
    }
    if !os_realpath(directory, buffer, len).is_null() {
        return OK;
    }
    if path_is_absolute(directory) {
        return FAIL;
    }
    let mut old_dir: [::core::ffi::c_char; 4096] = [0; 4096];
    if os_dirname(
        &raw mut old_dir as *mut ::core::ffi::c_char,
        MAXPATHL as size_t,
    ) == FAIL
    {
        return FAIL;
    }
    xstrlcpy(buffer, &raw mut old_dir as *mut ::core::ffi::c_char, len);
    if append_path(buffer, directory, len) == FAIL {
        return FAIL;
    }
    return OK;
}
#[unsafe(no_mangle)]
pub unsafe extern "C" fn append_path(
    mut path: *mut ::core::ffi::c_char,
    mut to_append: *const ::core::ffi::c_char,
    mut max_len: size_t,
) -> ::core::ffi::c_int {
    let mut current_length: size_t = strlen(path);
    let mut to_append_length: size_t = strlen(to_append);
    if to_append_length == 0 as size_t
        || strcmp(to_append, b".\0".as_ptr() as *const ::core::ffi::c_char)
            == 0 as ::core::ffi::c_int
    {
        return OK;
    }
    if current_length > 0 as size_t
        && !vim_ispathsep_nocolon(
            *path.offset(current_length.wrapping_sub(1 as size_t) as isize) as ::core::ffi::c_int,
        )
    {
        if current_length
            .wrapping_add(
                ::core::mem::size_of::<[::core::ffi::c_char; 2]>().wrapping_sub(1 as size_t),
            )
            .wrapping_add(1 as size_t)
            > max_len
        {
            return FAIL;
        }
        xstrlcpy(
            path.offset(current_length as isize),
            PATHSEPSTR.as_ptr(),
            max_len.wrapping_sub(current_length),
        );
        current_length = (current_length as ::core::ffi::c_ulong).wrapping_add(
            ::core::mem::size_of::<[::core::ffi::c_char; 2]>().wrapping_sub(1 as usize)
                as ::core::ffi::c_ulong,
        ) as size_t;
    }
    if current_length
        .wrapping_add(to_append_length)
        .wrapping_add(1 as size_t)
        > max_len
    {
        return FAIL;
    }
    xstrlcpy(
        path.offset(current_length as isize),
        to_append,
        max_len.wrapping_sub(current_length),
    );
    return OK;
}
unsafe extern "C" fn path_to_absolute(
    mut fname: *const ::core::ffi::c_char,
    mut buf: *mut ::core::ffi::c_char,
    mut len: size_t,
    mut force: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let mut p: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    *buf = NUL as ::core::ffi::c_char;
    let mut relative_directory: *mut ::core::ffi::c_char = xmalloc(len) as *mut ::core::ffi::c_char;
    let mut end_of_path: *const ::core::ffi::c_char = fname;
    if force != 0 || !path_is_absolute(fname) {
        p = strrchr(fname, '/' as ::core::ffi::c_int);
        if p.is_null()
            && strcmp(fname, b"..\0".as_ptr() as *const ::core::ffi::c_char)
                == 0 as ::core::ffi::c_int
        {
            p = fname.offset(2 as ::core::ffi::c_int as isize);
        }
        if !p.is_null() {
            if vim_ispathsep(*p as ::core::ffi::c_int) as ::core::ffi::c_int != 0
                && strcmp(
                    p.offset(1 as ::core::ffi::c_int as isize),
                    b"..\0".as_ptr() as *const ::core::ffi::c_char,
                ) == 0 as ::core::ffi::c_int
            {
                p = p.offset(3 as ::core::ffi::c_int as isize);
            }
            '_c2rust_label: {
                if p >= fname {
                } else {
                    __assert_fail(
                        b"p >= fname\0".as_ptr() as *const ::core::ffi::c_char,
                        b"src/nvim/path.rs\0".as_ptr() as *const ::core::ffi::c_char,
                        2403 as ::core::ffi::c_uint,
                        b"int path_to_absolute(const char *, char *, size_t, int)\0".as_ptr()
                            as *const ::core::ffi::c_char,
                    );
                }
            };
            memcpy(
                relative_directory as *mut ::core::ffi::c_void,
                fname as *const ::core::ffi::c_void,
                (p.offset_from(fname) + 1 as isize) as size_t,
            );
            *relative_directory.offset((p.offset_from(fname) + 1 as isize) as isize) =
                NUL as ::core::ffi::c_char;
            end_of_path = if vim_ispathsep(*p as ::core::ffi::c_int) as ::core::ffi::c_int != 0 {
                p.offset(1 as ::core::ffi::c_int as isize)
            } else {
                p
            };
        } else {
            *relative_directory.offset(0 as ::core::ffi::c_int as isize) =
                NUL as ::core::ffi::c_char;
        }
        if FAIL == path_full_dir_name(relative_directory, buf, len) {
            xfree(relative_directory as *mut ::core::ffi::c_void);
            return FAIL;
        }
    }
    xfree(relative_directory as *mut ::core::ffi::c_void);
    return append_path(buf, end_of_path, len);
}
#[unsafe(no_mangle)]
pub unsafe extern "C" fn path_guess_exepath(
    mut argv0: *const ::core::ffi::c_char,
    mut buf: *mut ::core::ffi::c_char,
    mut bufsize: size_t,
) {
    let mut path: *mut ::core::ffi::c_char =
        os_getenv(b"PATH\0".as_ptr() as *const ::core::ffi::c_char);
    if path.is_null() || path_is_absolute(argv0) as ::core::ffi::c_int != 0 {
        xstrlcpy(buf, argv0, bufsize);
    } else if *argv0.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
        == '.' as ::core::ffi::c_int
        || !strchr(argv0, PATHSEP).is_null()
    {
        if os_dirname(buf, MAXPATHL as size_t) != OK {
            *buf.offset(0 as ::core::ffi::c_int as isize) = NUL as ::core::ffi::c_char;
        }
        xstrlcat(buf, PATHSEPSTR.as_ptr(), bufsize);
        xstrlcat(buf, argv0, bufsize);
    } else {
        let mut iter: *const ::core::ffi::c_void = ::core::ptr::null::<::core::ffi::c_void>();
        loop {
            let mut dir: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
            let mut dir_len: size_t = 0;
            iter = vim_env_iter(
                ENV_SEPCHAR as ::core::ffi::c_char,
                path,
                iter,
                &raw mut dir,
                &raw mut dir_len,
            );
            if dir.is_null() || dir_len == 0 as size_t {
                break;
            }
            if dir_len.wrapping_add(1 as size_t)
                <= ::core::mem::size_of::<[::core::ffi::c_char; 4096]>()
            {
                xmemcpyz(
                    NameBuff.ptr() as *mut ::core::ffi::c_char as *mut ::core::ffi::c_void,
                    dir as *const ::core::ffi::c_void,
                    dir_len,
                );
                xstrlcat(
                    NameBuff.ptr() as *mut ::core::ffi::c_char,
                    PATHSEPSTR.as_ptr(),
                    ::core::mem::size_of::<[::core::ffi::c_char; 4096]>(),
                );
                xstrlcat(
                    NameBuff.ptr() as *mut ::core::ffi::c_char,
                    argv0,
                    ::core::mem::size_of::<[::core::ffi::c_char; 4096]>(),
                );
                if os_can_exe(
                    NameBuff.ptr() as *mut ::core::ffi::c_char,
                    ::core::ptr::null_mut::<*mut ::core::ffi::c_char>(),
                    false_0 != 0,
                ) {
                    xstrlcpy(buf, NameBuff.ptr() as *mut ::core::ffi::c_char, bufsize);
                    return;
                }
            }
            if iter.is_null() {
                break;
            }
        }
        xstrlcpy(buf, argv0, bufsize);
    }
    xfree(path as *mut ::core::ffi::c_void);
}
pub const true_0: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const false_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const SPECIAL_WILDCHAR: [::core::ffi::c_char; 4] =
    unsafe { ::core::mem::transmute::<[u8; 4], [::core::ffi::c_char; 4]>(*b"`'{\0") };
pub const ENV_SEPCHAR: ::core::ffi::c_int = ':' as ::core::ffi::c_int;
pub const RE_MAGIC: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const RE_STRING: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
pub const RE_NOBREAK: ::core::ffi::c_int = 16 as ::core::ffi::c_int;
