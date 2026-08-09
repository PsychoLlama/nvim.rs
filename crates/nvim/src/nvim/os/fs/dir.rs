//! Creating, removing and walking directories, and moving files between
//! them.
//!
//! Same shape as the rest of `os/fs`: a `uv_fs_t` per call, `req.result`
//! read back, `uv_fs_req_cleanup` on the way out.

#![allow(unsafe_op_in_unsafe_fn)]

use super::*;
use crate::semsg_c;
#[unsafe(no_mangle)]
pub unsafe extern "C" fn os_path_exists(mut path: *const ::core::ffi::c_char) -> bool {
    let mut statbuf: uv_stat_t = uv_stat_t {
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
    };
    return os_stat(path, &raw mut statbuf) == kLibuvSuccess.get();
}
#[unsafe(no_mangle)]
pub unsafe extern "C" fn os_rename(
    mut path: *const ::core::ffi::c_char,
    mut new_path: *const ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    fs_ok(|req| {
        uv_fs_rename(
            ::core::ptr::null_mut::<uv_loop_t>(),
            req,
            path,
            new_path,
            None,
        )
    })
}
#[unsafe(no_mangle)]
pub unsafe extern "C" fn os_mkdir(
    mut path: *const ::core::ffi::c_char,
    mut mode: int32_t,
) -> ::core::ffi::c_int {
    fs_result(|req| {
        uv_fs_mkdir(
            ::core::ptr::null_mut::<uv_loop_t>(),
            req,
            path,
            mode as ::core::ffi::c_int,
            None,
        )
    })
}
#[unsafe(no_mangle)]
pub unsafe extern "C" fn os_mkdir_recurse(
    dir: *const ::core::ffi::c_char,
    mut mode: int32_t,
    failed_dir: *mut *mut ::core::ffi::c_char,
    created: *mut *mut ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    let dirlen: size_t = strlen(dir);
    let curdir: *mut ::core::ffi::c_char =
        xmemdupz(dir as *const ::core::ffi::c_void, dirlen) as *mut ::core::ffi::c_char;
    let past_head: *mut ::core::ffi::c_char = get_past_head(curdir);
    let mut e: *mut ::core::ffi::c_char = curdir.add(dirlen);
    let real_end: *const ::core::ffi::c_char = e;
    let past_head_save: ::core::ffi::c_char = *past_head;
    while !os_isdir(curdir) {
        e = path_tail_with_sep(curdir);
        if e <= past_head {
            *past_head = NUL as ::core::ffi::c_char;
            break;
        } else {
            *e = NUL as ::core::ffi::c_char;
        }
    }
    while e != real_end as *mut ::core::ffi::c_char {
        if e > past_head {
            *e = PATHSEP as ::core::ffi::c_char;
        } else {
            *past_head = past_head_save;
        }
        let component_len: size_t = strlen(e);
        e = e.add(component_len);
        if e == real_end as *mut ::core::ffi::c_char
            && memcnt(
                e.offset(-(component_len as isize)) as *const ::core::ffi::c_void,
                PATHSEP as ::core::ffi::c_char,
                component_len,
            ) == component_len
        {
            break;
        }
        let mut ret: ::core::ffi::c_int = 0;
        ret = os_mkdir(curdir, mode);
        if ret != 0 as ::core::ffi::c_int {
            *failed_dir = curdir;
            return ret;
        } else if !created.is_null() && (*created).is_null() {
            *created = FullName_save(curdir, false_0 != 0);
        }
    }
    xfree(curdir as *mut ::core::ffi::c_void);
    return 0 as ::core::ffi::c_int;
}
pub unsafe extern "C" fn os_file_mkdir(
    mut fname: *mut ::core::ffi::c_char,
    mut mode: int32_t,
) -> ::core::ffi::c_int {
    if !dir_of_file_exists(fname) {
        let mut tail: *mut ::core::ffi::c_char = path_tail_with_sep(fname);
        let mut last_char: *mut ::core::ffi::c_char = tail.add(strlen(tail)).sub(1);
        if vim_ispathsep(*last_char as ::core::ffi::c_int) {
            emsg(gettext(&raw const e_noname as *const ::core::ffi::c_char));
            return -1 as ::core::ffi::c_int;
        }
        let mut c: ::core::ffi::c_char = *tail;
        *tail = NUL as ::core::ffi::c_char;
        let mut r: ::core::ffi::c_int = 0;
        let mut failed_dir: *mut ::core::ffi::c_char =
            ::core::ptr::null_mut::<::core::ffi::c_char>();
        r = os_mkdir_recurse(
            fname,
            mode,
            &raw mut failed_dir,
            ::core::ptr::null_mut::<*mut ::core::ffi::c_char>(),
        );
        if r < 0 as ::core::ffi::c_int {
            semsg_c!(
                gettext(&raw const e_mkdir as *const ::core::ffi::c_char),
                failed_dir,
                uv_strerror(r),
            );
            xfree(failed_dir as *mut ::core::ffi::c_void);
        }
        *tail = c;
        return r;
    }
    return 0 as ::core::ffi::c_int;
}
pub unsafe extern "C" fn os_mkdtemp(
    mut templ: *const ::core::ffi::c_char,
    mut path: *mut ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    // `request.path` is the directory libuv made, and cleanup frees it.
    fs_request(
        |request| uv_fs_mkdtemp(::core::ptr::null_mut::<uv_loop_t>(), request, templ, None),
        |result, request| {
            if result == kLibuvSuccess.get() {
                xstrlcpy(path, request.path, TEMP_FILE_PATH_MAXLEN as size_t);
            }
            result
        },
    )
}
#[unsafe(no_mangle)]
pub unsafe extern "C" fn os_rmdir(mut path: *const ::core::ffi::c_char) -> ::core::ffi::c_int {
    fs_result(|req| uv_fs_rmdir(::core::ptr::null_mut::<uv_loop_t>(), req, path, None))
}
pub unsafe extern "C" fn os_scandir(
    mut dir: *mut Directory,
    mut path: *const ::core::ffi::c_char,
) -> bool {
    let mut r: ::core::ffi::c_int = uv_fs_scandir(
        ::core::ptr::null_mut::<uv_loop_t>(),
        &raw mut (*dir).request,
        path,
        0 as ::core::ffi::c_int,
        None,
    );
    if r < 0 as ::core::ffi::c_int {
        os_closedir(dir);
    }
    return r >= 0 as ::core::ffi::c_int;
}
pub unsafe extern "C" fn os_scandir_next(mut dir: *mut Directory) -> *const ::core::ffi::c_char {
    let mut err: ::core::ffi::c_int =
        uv_fs_scandir_next(&raw mut (*dir).request, &raw mut (*dir).ent);
    return if err != UV_EOF as ::core::ffi::c_int {
        (*dir).ent.name
    } else {
        ::core::ptr::null::<::core::ffi::c_char>()
    };
}
pub unsafe extern "C" fn os_closedir(mut dir: *mut Directory) {
    uv_fs_req_cleanup(&raw mut (*dir).request);
}
#[unsafe(no_mangle)]
pub unsafe extern "C" fn os_remove(mut path: *const ::core::ffi::c_char) -> ::core::ffi::c_int {
    fs_result(|req| uv_fs_unlink(::core::ptr::null_mut::<uv_loop_t>(), req, path, None))
}
