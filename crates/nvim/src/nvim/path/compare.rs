//! Deciding whether two names mean the same file.
//!
//! [`path_full_compare`] is the one with an answer for every case: same file,
//! different files, one missing, both missing — resolving both names and
//! consulting the file system when it has to. [`pathcmp`] is the text-only
//! comparison that sorts and matches names, treating a path separator as
//! less than any other byte so `"foo/bar"` sorts before `"foo-bar"`.
//! [`path_fix_case`] replaces a name's last component with the spelling the
//! file system actually uses, which is what makes completion look right on a
//! case-insensitive volume.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;

#[unsafe(no_mangle)]
pub unsafe extern "C" fn path_full_compare(
    s1: *mut ::core::ffi::c_char,
    s2: *mut ::core::ffi::c_char,
    checkname: bool,
    expandenv: bool,
) -> FileComparison {
    unsafe {
        let mut expanded1: [::core::ffi::c_char; 4096] = [0; 4096];
        let mut full1: [::core::ffi::c_char; 4096] = [0; 4096];
        let mut full2: [::core::ffi::c_char; 4096] = [0; 4096];
        let mut file_id_1: FileID = FileID {
            inode: 0,
            device_id: 0,
        };
        let mut file_id_2: FileID = FileID {
            inode: 0,
            device_id: 0,
        };
        if expandenv {
            expand_env(s1, &raw mut expanded1 as *mut ::core::ffi::c_char, MAXPATHL);
        } else {
            xstrlcpy(
                &raw mut expanded1 as *mut ::core::ffi::c_char,
                s1,
                MAXPATHL as size_t,
            );
        }
        let mut id_ok_1: bool = os_fileid(
            &raw mut expanded1 as *mut ::core::ffi::c_char,
            &raw mut file_id_1,
        );
        let mut id_ok_2: bool = os_fileid(s2, &raw mut file_id_2);
        if !id_ok_1 && !id_ok_2 {
            if checkname {
                vim_FullName(
                    &raw mut expanded1 as *mut ::core::ffi::c_char,
                    &raw mut full1 as *mut ::core::ffi::c_char,
                    MAXPATHL as size_t,
                    false_0 != 0,
                );
                vim_FullName(
                    s2,
                    &raw mut full2 as *mut ::core::ffi::c_char,
                    MAXPATHL as size_t,
                    false_0 != 0,
                );
                if path_fnamecmp(
                    &raw mut full1 as *mut ::core::ffi::c_char,
                    &raw mut full2 as *mut ::core::ffi::c_char,
                ) == 0 as ::core::ffi::c_int
                {
                    return kEqualFileNames;
                }
            }
            return kBothFilesMissing;
        }
        if !id_ok_1 || !id_ok_2 {
            return kOneFileMissing;
        }
        if os_fileid_equal(&raw mut file_id_1, &raw mut file_id_2) {
            return kEqualFiles;
        }
        return kDifferentFiles;
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn path_fix_case(mut name: *mut ::core::ffi::c_char) {
    unsafe {
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
        if !os_fileinfo_link(name, &raw mut file_info) {
            return;
        }
        let mut slash: *mut ::core::ffi::c_char = strrchr(name, '/' as ::core::ffi::c_int);
        let mut tail: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
        let mut dir: Directory = Directory {
            request: uv_fs_t {
                data: ::core::ptr::null_mut::<::core::ffi::c_void>(),
                type_0: UV_UNKNOWN_REQ,
                reserved: [::core::ptr::null_mut::<::core::ffi::c_void>(); 6],
                fs_type: UV_FS_CUSTOM,
                loop_0: ::core::ptr::null_mut::<uv_loop_t>(),
                cb: None,
                result: 0,
                ptr: ::core::ptr::null_mut::<::core::ffi::c_void>(),
                path: ::core::ptr::null::<::core::ffi::c_char>(),
                statbuf: uv_stat_t {
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
                new_path: ::core::ptr::null::<::core::ffi::c_char>(),
                file: 0,
                flags: 0,
                mode: 0,
                nbufs: 0,
                bufs: ::core::ptr::null_mut::<uv_buf_t>(),
                off: 0,
                uid: 0,
                gid: 0,
                atime: 0.,
                mtime: 0.,
                work_req: uv__work {
                    work: None,
                    done: None,
                    loop_0: ::core::ptr::null_mut::<uv_loop_s>(),
                    wq: uv__queue {
                        next: ::core::ptr::null_mut::<uv__queue>(),
                        prev: ::core::ptr::null_mut::<uv__queue>(),
                    },
                },
                bufsml: [uv_buf_t {
                    base: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                    len: 0,
                }; 4],
            },
            ent: uv_dirent_t {
                name: ::core::ptr::null::<::core::ffi::c_char>(),
                type_0: UV_DIRENT_UNKNOWN,
            },
        };
        let mut ok: bool = false;
        if slash.is_null() {
            ok = os_scandir(&raw mut dir, b".\0".as_ptr() as *const ::core::ffi::c_char);
            tail = name;
        } else {
            *slash = NUL as ::core::ffi::c_char;
            ok = os_scandir(&raw mut dir, name);
            *slash = '/' as ::core::ffi::c_char;
            tail = slash.offset(1 as ::core::ffi::c_int as isize);
        }
        if !ok {
            return;
        }
        let mut taillen: size_t = strlen(tail);
        let mut entry: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
        loop {
            entry = os_scandir_next(&raw mut dir);
            if entry.is_null() {
                break;
            }
            if !(strcasecmp(tail, entry as *mut ::core::ffi::c_char) == 0 as ::core::ffi::c_int
                && taillen == strlen(entry))
            {
                continue;
            }
            let mut newname: [::core::ffi::c_char; 4097] = [0; 4097];
            xstrlcpy(
                &raw mut newname as *mut ::core::ffi::c_char,
                name,
                (MAXPATHL + 1 as ::core::ffi::c_int) as size_t,
            );
            xstrlcpy(
                (&raw mut newname as *mut ::core::ffi::c_char)
                    .offset(tail.offset_from(name) as isize),
                entry,
                (MAXPATHL as isize - tail.offset_from(name) + 1 as isize) as size_t,
            );
            let mut file_info_new: FileInfo = FileInfo {
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
            if !(os_fileinfo_link(
                &raw mut newname as *mut ::core::ffi::c_char,
                &raw mut file_info_new,
            ) as ::core::ffi::c_int
                != 0
                && os_fileinfo_id_equal(&raw mut file_info, &raw mut file_info_new)
                    as ::core::ffi::c_int
                    != 0)
            {
                continue;
            }
            strcpy(tail, entry as *mut ::core::ffi::c_char);
            break;
        }
        os_closedir(&raw mut dir);
    }
}

pub unsafe extern "C" fn same_directory(
    mut f1: *mut ::core::ffi::c_char,
    mut f2: *mut ::core::ffi::c_char,
) -> bool {
    unsafe {
        let mut ffname: [::core::ffi::c_char; 4096] = [0; 4096];
        let mut t1: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
        let mut t2: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
        if f1.is_null() || f2.is_null() {
            return false_0 != 0;
        }
        vim_FullName(
            f1,
            &raw mut ffname as *mut ::core::ffi::c_char,
            MAXPATHL as size_t,
            false_0 != 0,
        );
        t1 = path_tail_with_sep(&raw mut ffname as *mut ::core::ffi::c_char);
        t2 = path_tail_with_sep(f2);
        return t1.offset_from(&raw mut ffname as *mut ::core::ffi::c_char) == t2.offset_from(f2)
            && pathcmp(
                &raw mut ffname as *mut ::core::ffi::c_char,
                f2,
                t1.offset_from(&raw mut ffname as *mut ::core::ffi::c_char) as ::core::ffi::c_int,
            ) == 0 as ::core::ffi::c_int;
    }
}

pub unsafe extern "C" fn pathcmp(
    mut p: *const ::core::ffi::c_char,
    mut q: *const ::core::ffi::c_char,
    mut maxlen: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    unsafe {
        let mut i: ::core::ffi::c_int = 0;
        let mut j: ::core::ffi::c_int = 0;
        let mut s: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
        i = 0 as ::core::ffi::c_int;
        j = 0 as ::core::ffi::c_int;
        while maxlen < 0 as ::core::ffi::c_int || i < maxlen && j < maxlen {
            let mut c1: ::core::ffi::c_int = utf_ptr2char(p.offset(i as isize));
            let mut c2: ::core::ffi::c_int = utf_ptr2char(q.offset(j as isize));
            if c1 == NUL {
                if c2 == NUL {
                    return 0 as ::core::ffi::c_int;
                }
                s = q;
                i = j;
                break;
            } else if c2 == NUL {
                s = p;
                break;
            } else {
                if if p_fic.get() != 0 {
                    (mb_toupper(c1) != mb_toupper(c2)) as ::core::ffi::c_int
                } else {
                    (c1 != c2) as ::core::ffi::c_int
                } != 0
                {
                    if vim_ispathsep(c1) {
                        return -1 as ::core::ffi::c_int;
                    }
                    if vim_ispathsep(c2) {
                        return 1 as ::core::ffi::c_int;
                    }
                    return if p_fic.get() != 0 {
                        mb_toupper(c1) - mb_toupper(c2)
                    } else {
                        c1 - c2
                    };
                }
                i += utfc_ptr2len(p.offset(i as isize));
                j += utfc_ptr2len(q.offset(j as isize));
            }
        }
        if s.is_null() {
            return 0 as ::core::ffi::c_int;
        }
        let mut c1_0: ::core::ffi::c_int = utf_ptr2char(s.offset(i as isize));
        let mut c2_0: ::core::ffi::c_int = utf_ptr2char(
            s.offset(i as isize)
                .offset(utfc_ptr2len(s.offset(i as isize)) as isize),
        );
        if c2_0 == NUL
            && i > 0 as ::core::ffi::c_int
            && after_pathsep(s, s.offset(i as isize)) == 0
            && c1_0 == '/' as ::core::ffi::c_int
        {
            return 0 as ::core::ffi::c_int;
        }
        if s == q {
            return -1 as ::core::ffi::c_int;
        }
        return 1 as ::core::ffi::c_int;
    }
}
