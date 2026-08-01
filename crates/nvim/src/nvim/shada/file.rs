//! The ShaDa file itself: where it lives, and opening it.
//!
//! `shada_get_default_file` works out `$NVIM_SHADA_FILE` or the state
//! directory's `shada/main.shada`; `shada_filename` expands the name a
//! `:rshada`/`:wshada` argument gave. `shada_write_file` is the whole
//! write-a-file dance — a temporary file next to the target, renamed over it
//! once it is complete, with the permissions and ownership of the old one
//! carried across. `shada_removable` is what decides that a file on a
//! removable medium should not have its marks remembered.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;

#[inline(always)]
pub(crate) unsafe extern "C" fn file_eof(fp: *const FileDescriptor) -> bool {
    unsafe {
        return (*fp).eof as ::core::ffi::c_int != 0 && (*fp).read_pos == (*fp).write_pos;
    }
}

#[inline(always)]
pub(crate) unsafe extern "C" fn file_fd(fp: *const FileDescriptor) -> ::core::ffi::c_int {
    unsafe {
        return (*fp).fd;
    }
}

#[inline]
pub(crate) unsafe extern "C" fn file_space(mut fp: *mut FileDescriptor) -> size_t {
    unsafe {
        return (*fp)
            .buffer
            .offset(ARENA_BLOCK_SIZE as isize)
            .offset_from((*fp).write_pos) as size_t;
    }
}

pub(crate) unsafe extern "C" fn close_file(mut cookie: *mut FileDescriptor) {
    unsafe {
        let error: ::core::ffi::c_int = file_close(cookie, p_fs.get() != 0);
        if error != 0 as ::core::ffi::c_int {
            semsg(
                gettext(
                    b"E886: System error while closing ShaDa file: %s\0".as_ptr()
                        as *const ::core::ffi::c_char,
                ),
                uv_strerror(error),
            );
        }
    }
}

pub(crate) unsafe extern "C" fn shada_read_file(
    file: *const ::core::ffi::c_char,
    flags: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    unsafe {
        let fname: *mut ::core::ffi::c_char = shada_filename(file);
        if fname.is_null() {
            return FAIL;
        }
        let mut sd_reader: FileDescriptor = FileDescriptor {
            fd: 0,
            buffer: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            read_pos: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            write_pos: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            wr: false,
            eof: false,
            non_blocking: false,
            bytes_read: 0,
        };
        let mut of_ret: ::core::ffi::c_int = file_open(
            &raw mut sd_reader,
            fname,
            kFileReadOnly as ::core::ffi::c_int,
            0 as ::core::ffi::c_int,
        );
        if p_verbose.get() > 1 as OptInt {
            verbose_enter();
            smsg(
                0 as ::core::ffi::c_int,
                gettext(
                    b"Reading ShaDa file \"%s\"%s%s%s%s\0".as_ptr() as *const ::core::ffi::c_char
                ),
                fname,
                if flags & kShaDaWantInfo as ::core::ffi::c_int != 0 {
                    gettext(b" info\0".as_ptr() as *const ::core::ffi::c_char)
                        as *const ::core::ffi::c_char
                } else {
                    b"\0".as_ptr() as *const ::core::ffi::c_char
                },
                if flags & kShaDaWantMarks as ::core::ffi::c_int != 0 {
                    gettext(b" marks\0".as_ptr() as *const ::core::ffi::c_char)
                        as *const ::core::ffi::c_char
                } else {
                    b"\0".as_ptr() as *const ::core::ffi::c_char
                },
                if flags & kShaDaGetOldfiles as ::core::ffi::c_int != 0 {
                    gettext(b" oldfiles\0".as_ptr() as *const ::core::ffi::c_char)
                        as *const ::core::ffi::c_char
                } else {
                    b"\0".as_ptr() as *const ::core::ffi::c_char
                },
                if of_ret != 0 as ::core::ffi::c_int {
                    gettext(b" FAILED\0".as_ptr() as *const ::core::ffi::c_char)
                        as *const ::core::ffi::c_char
                } else {
                    b"\0".as_ptr() as *const ::core::ffi::c_char
                },
            );
            verbose_leave();
        }
        if of_ret != 0 as ::core::ffi::c_int {
            if of_ret != UV_ENOENT as ::core::ffi::c_int
                || flags & kShaDaMissingError as ::core::ffi::c_int != 0
            {
                semsg(
                    gettext(
                        b"E886: System error while opening ShaDa file %s for reading: %s\0".as_ptr()
                            as *const ::core::ffi::c_char,
                    ),
                    fname,
                    uv_strerror(of_ret),
                );
            }
            xfree(fname as *mut ::core::ffi::c_void);
            return FAIL;
        }
        xfree(fname as *mut ::core::ffi::c_void);
        shada_read(&raw mut sd_reader, flags);
        close_file(&raw mut sd_reader);
        return OK;
    }
}

pub(crate) unsafe extern "C" fn shada_get_default_file() -> *const ::core::ffi::c_char {
    unsafe {
        if (*default_shada_file.ptr()).is_null() {
            let mut shada_dir: *mut ::core::ffi::c_char = stdpaths_user_state_subpath(
                b"shada\0".as_ptr() as *const ::core::ffi::c_char,
                0 as size_t,
                false_0 != 0,
            );
            default_shada_file.set(concat_fnames_realloc(
                shada_dir,
                b"main.shada\0".as_ptr() as *const ::core::ffi::c_char,
                true_0 != 0,
            ));
        }
        return default_shada_file.get();
    }
}

pub(crate) unsafe extern "C" fn shada_filename(
    mut file: *const ::core::ffi::c_char,
) -> *mut ::core::ffi::c_char {
    unsafe {
        if file.is_null() || *file as ::core::ffi::c_int == NUL {
            if !(*p_shadafile.ptr()).is_null() && *p_shadafile.get() as ::core::ffi::c_int != NUL {
                if !strequal(
                    p_shadafile.get(),
                    b"NONE\0".as_ptr() as *const ::core::ffi::c_char,
                ) {
                    file = p_shadafile.get();
                } else {
                    return ::core::ptr::null_mut::<::core::ffi::c_char>();
                }
            } else {
                file = find_shada_parameter('n' as ::core::ffi::c_int);
                if file.is_null() || *file as ::core::ffi::c_int == NUL {
                    file = shada_get_default_file();
                }
                let mut len: size_t = expand_env(
                    file as *mut ::core::ffi::c_char,
                    (NameBuff.ptr() as *mut ::core::ffi::c_char)
                        .offset(0 as ::core::ffi::c_int as isize),
                    MAXPATHL,
                );
                file = (NameBuff.ptr() as *mut ::core::ffi::c_char)
                    .offset(0 as ::core::ffi::c_int as isize);
                return xmemdupz(file as *const ::core::ffi::c_void, len)
                    as *mut ::core::ffi::c_char;
            }
        }
        return xstrdup(file);
    }
}

pub unsafe extern "C" fn shada_write_file(
    file: *const ::core::ffi::c_char,
    mut nomerge: bool,
) -> ::core::ffi::c_int {
    unsafe {
        let fname: *mut ::core::ffi::c_char = shada_filename(file);
        if fname.is_null() {
            return FAIL;
        }
        let mut tempname: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
        let mut sd_writer: FileDescriptor = FileDescriptor {
            fd: 0,
            buffer: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            read_pos: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            write_pos: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            wr: false,
            eof: false,
            non_blocking: false,
            bytes_read: 0,
        };
        let mut sd_reader: FileDescriptor = FileDescriptor {
            fd: 0,
            buffer: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            read_pos: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            write_pos: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            wr: false,
            eof: false,
            non_blocking: false,
            bytes_read: 0,
        };
        let mut did_open_writer: bool = false_0 != 0;
        let mut did_open_reader: bool = false_0 != 0;
        's_240: {
            's_163: {
                's_154: {
                    if !nomerge {
                        let mut error: ::core::ffi::c_int = 0;
                        error = file_open(
                            &raw mut sd_reader,
                            fname,
                            kFileReadOnly as ::core::ffi::c_int,
                            0 as ::core::ffi::c_int,
                        );
                        if error != 0 as ::core::ffi::c_int {
                            if error != UV_ENOENT as ::core::ffi::c_int {
                                semsg(
                                gettext(
                                    b"E886: System error while opening ShaDa file %s for reading to merge before writing it: %s\0"
                                        .as_ptr() as *const ::core::ffi::c_char,
                                ),
                                fname,
                                uv_strerror(error),
                            );
                            }
                            nomerge = true_0 != 0;
                            break 's_163;
                        } else {
                            did_open_reader = true_0 != 0;
                            tempname = modname(
                                fname,
                                b".tmp.a\0".as_ptr() as *const ::core::ffi::c_char,
                                false_0 != 0,
                            );
                            if tempname.is_null() {
                                nomerge = true_0 != 0;
                                break 's_163;
                            } else {
                                let mut perm: ::core::ffi::c_int =
                                    os_getperm(fname) as ::core::ffi::c_int;
                                perm = if perm >= 0 as ::core::ffi::c_int {
                                    perm & 0o777 as ::core::ffi::c_int | 0o600 as ::core::ffi::c_int
                                } else {
                                    0o600 as ::core::ffi::c_int
                                };
                                loop {
                                    error = file_open(
                                        &raw mut sd_writer,
                                        tempname,
                                        kFileCreateOnly as ::core::ffi::c_int
                                            | kFileNoSymlink as ::core::ffi::c_int,
                                        perm,
                                    );
                                    if error != 0 {
                                        if error == UV_EEXIST as ::core::ffi::c_int
                                            || error == UV_ELOOP as ::core::ffi::c_int
                                        {
                                            let wp: *mut ::core::ffi::c_char = tempname
                                                .offset(strlen(tempname) as isize)
                                                .offset(-(1 as ::core::ffi::c_int as isize));
                                            if *wp as ::core::ffi::c_int
                                                == 'z' as ::core::ffi::c_int
                                            {
                                                semsg(
                                                gettext(
                                                    b"E138: All %s.tmp.X files exist, cannot write ShaDa file!\0"
                                                        .as_ptr() as *const ::core::ffi::c_char,
                                                ),
                                                fname,
                                            );
                                                xfree(fname as *mut ::core::ffi::c_void);
                                                xfree(tempname as *mut ::core::ffi::c_void);
                                                if did_open_reader {
                                                    close_file(&raw mut sd_reader);
                                                }
                                                return FAIL;
                                            }
                                            *wp += 1;
                                        } else {
                                            semsg(
                                            gettext(
                                                b"E886: System error while opening temporary ShaDa file %s for writing: %s\0"
                                                    .as_ptr() as *const ::core::ffi::c_char,
                                            ),
                                            tempname,
                                            uv_strerror(error),
                                        );
                                            break 's_154;
                                        }
                                    } else {
                                        did_open_writer = true_0 != 0;
                                        break 's_154;
                                    }
                                }
                            }
                        }
                    }
                }
                if !nomerge {
                    break 's_240;
                }
            }
            let tail: *mut ::core::ffi::c_char = path_tail_with_sep(fname);
            if tail != fname {
                let tail_save: ::core::ffi::c_char = *tail;
                *tail = NUL as ::core::ffi::c_char;
                if !os_isdir(fname) {
                    let mut ret: ::core::ffi::c_int = 0;
                    let mut failed_dir: *mut ::core::ffi::c_char =
                        ::core::ptr::null_mut::<::core::ffi::c_char>();
                    ret = os_mkdir_recurse(
                        fname,
                        0o700 as int32_t,
                        &raw mut failed_dir,
                        ::core::ptr::null_mut::<*mut ::core::ffi::c_char>(),
                    );
                    if ret != 0 as ::core::ffi::c_int {
                        semsg(
                            gettext(
                                b"E886: Failed to create directory %s for writing ShaDa file: %s\0"
                                    .as_ptr()
                                    as *const ::core::ffi::c_char,
                            ),
                            failed_dir,
                            uv_strerror(ret),
                        );
                        xfree(fname as *mut ::core::ffi::c_void);
                        xfree(failed_dir as *mut ::core::ffi::c_void);
                        return FAIL;
                    }
                }
                *tail = tail_save;
            }
            let mut error_0: ::core::ffi::c_int = file_open(
                &raw mut sd_writer,
                fname,
                kFileCreate as ::core::ffi::c_int | kFileTruncate as ::core::ffi::c_int,
                0o600 as ::core::ffi::c_int,
            );
            if error_0 != 0 {
                semsg(
                    gettext(
                        b"E886: System error while opening ShaDa file %s for writing: %s\0".as_ptr()
                            as *const ::core::ffi::c_char,
                    ),
                    fname,
                    uv_strerror(error_0),
                );
            } else {
                did_open_writer = true_0 != 0;
            }
        }
        if !did_open_writer {
            xfree(fname as *mut ::core::ffi::c_void);
            xfree(tempname as *mut ::core::ffi::c_void);
            if did_open_reader {
                close_file(&raw mut sd_reader);
            }
            return FAIL;
        }
        if p_verbose.get() > 1 as OptInt {
            verbose_enter();
            smsg(
                0 as ::core::ffi::c_int,
                gettext(b"Writing ShaDa file \"%s\"\0".as_ptr() as *const ::core::ffi::c_char),
                fname,
            );
            verbose_leave();
        }
        let sw_ret: ShaDaWriteResult = shada_write(
            &raw mut sd_writer,
            if nomerge as ::core::ffi::c_int != 0 {
                ::core::ptr::null_mut::<FileDescriptor>()
            } else {
                &raw mut sd_reader
            },
        );
        '_c2rust_label: {
            if sw_ret as ::core::ffi::c_uint
                != kSDWriteIgnError as ::core::ffi::c_int as ::core::ffi::c_uint
            {
            } else {
                __assert_fail(
                    b"sw_ret != kSDWriteIgnError\0".as_ptr() as *const ::core::ffi::c_char,
                    b"src/nvim/shada.rs\0".as_ptr() as *const ::core::ffi::c_char,
                    2802 as ::core::ffi::c_uint,
                    b"int shada_write_file(const char *const, _Bool)\0".as_ptr()
                        as *const ::core::ffi::c_char,
                );
            }
        };
        if !nomerge {
            if did_open_reader {
                close_file(&raw mut sd_reader);
            }
            let mut did_remove: bool = false_0 != 0;
            's_417: {
                '_shada_write_file_did_not_remove: {
                    if sw_ret as ::core::ffi::c_uint
                        == kSDWriteSuccessful as ::core::ffi::c_int as ::core::ffi::c_uint
                    {
                        let mut old_info: FileInfo = FileInfo {
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
                        if !os_fileinfo(fname, &raw mut old_info)
                            || old_info.stat.st_mode & __S_IFMT as uint64_t == 0o40000 as uint64_t
                            || getuid() != ROOT_UID as __uid_t
                                && (if old_info.stat.st_uid == getuid() as uint64_t {
                                    old_info.stat.st_mode & 0o200 as uint64_t
                                } else {
                                    if old_info.stat.st_gid == getgid() as uint64_t {
                                        old_info.stat.st_mode & 0o20 as uint64_t
                                    } else {
                                        old_info.stat.st_mode & 0o2 as uint64_t
                                    }
                                }) == 0
                        {
                            semsg(
                                gettext(b"E137: ShaDa file is not writable: %s\0".as_ptr()
                                    as *const ::core::ffi::c_char),
                                fname,
                            );
                            break '_shada_write_file_did_not_remove;
                        } else {
                            if getuid() == ROOT_UID as __uid_t {
                                if old_info.stat.st_uid != ROOT_UID as uint64_t
                                    || old_info.stat.st_gid != getgid() as uint64_t
                                {
                                    let old_uid: uv_uid_t = old_info.stat.st_uid as uv_uid_t;
                                    let old_gid: uv_gid_t = old_info.stat.st_gid as uv_gid_t;
                                    let fchown_ret: ::core::ffi::c_int =
                                        os_fchown(file_fd(&raw mut sd_writer), old_uid, old_gid);
                                    if fchown_ret != 0 as ::core::ffi::c_int {
                                        semsg(
                                        gettext(
                                            b"E136: Failed setting uid and gid for file %s: %s\0"
                                                .as_ptr()
                                                as *const ::core::ffi::c_char,
                                        ),
                                        tempname,
                                        uv_strerror(fchown_ret),
                                    );
                                        break '_shada_write_file_did_not_remove;
                                    }
                                }
                            }
                            if vim_rename(tempname, fname) == -1 as ::core::ffi::c_int {
                                semsg(
                                    gettext(
                                        b"E136: Can't rename ShaDa file from %s to %s!\0".as_ptr()
                                            as *const ::core::ffi::c_char,
                                    ),
                                    tempname,
                                    fname,
                                );
                            } else {
                                did_remove = true_0 != 0;
                                os_remove(tempname);
                            }
                        }
                    } else if sw_ret as ::core::ffi::c_uint
                        == kSDWriteReadNotShada as ::core::ffi::c_int as ::core::ffi::c_uint
                    {
                        semsg(
                        gettext(
                            b"E136: Did not rename %s because %s does not look like a ShaDa file\0"
                                .as_ptr() as *const ::core::ffi::c_char,
                        ),
                        tempname,
                        fname,
                    );
                    } else {
                        semsg(
                        gettext(
                            b"E136: Did not rename %s to %s because there were errors during writing it\0"
                                .as_ptr() as *const ::core::ffi::c_char,
                        ),
                        tempname,
                        fname,
                    );
                    }
                    if did_remove {
                        break 's_417;
                    }
                }
                semsg(
                    gettext(
                        b"E136: Do not forget to remove %s or rename it manually to %s.\0".as_ptr()
                            as *const ::core::ffi::c_char,
                    ),
                    tempname,
                    fname,
                );
            }
            xfree(tempname as *mut ::core::ffi::c_void);
        }
        close_file(&raw mut sd_writer);
        xfree(fname as *mut ::core::ffi::c_void);
        return OK;
    }
}

pub unsafe extern "C" fn shada_read_marks() -> ::core::ffi::c_int {
    unsafe {
        return shada_read_file(
            ::core::ptr::null::<::core::ffi::c_char>(),
            kShaDaWantMarks as ::core::ffi::c_int,
        );
    }
}

pub unsafe extern "C" fn shada_read_everything(
    fname: *const ::core::ffi::c_char,
    forceit: bool,
    missing_ok: bool,
) -> ::core::ffi::c_int {
    unsafe {
        return shada_read_file(
            fname,
            kShaDaWantInfo as ::core::ffi::c_int
                | kShaDaWantMarks as ::core::ffi::c_int
                | kShaDaGetOldfiles as ::core::ffi::c_int
                | (if forceit as ::core::ffi::c_int != 0 {
                    kShaDaForceit as ::core::ffi::c_int
                } else {
                    0 as ::core::ffi::c_int
                })
                | (if missing_ok as ::core::ffi::c_int != 0 {
                    0 as ::core::ffi::c_int
                } else {
                    kShaDaMissingError as ::core::ffi::c_int
                }),
        );
    }
}

pub(crate) unsafe extern "C" fn shada_removable(mut name: *const ::core::ffi::c_char) -> bool {
    unsafe {
        let mut part: [::core::ffi::c_char; 4097] = [0; 4097];
        let mut retval: bool = false_0 != 0;
        let mut new_name: *mut ::core::ffi::c_char =
            home_replace_save(::core::ptr::null_mut::<buf_T>(), name);
        let mut p: *mut ::core::ffi::c_char = p_shada.get();
        while *p != 0 {
            copy_option_part(
                &raw mut p,
                &raw mut part as *mut ::core::ffi::c_char,
                ::core::mem::size_of::<[::core::ffi::c_char; 4097]>()
                    .wrapping_div(::core::mem::size_of::<::core::ffi::c_char>())
                    .wrapping_div(
                        (::core::mem::size_of::<[::core::ffi::c_char; 4097]>()
                            .wrapping_rem(::core::mem::size_of::<::core::ffi::c_char>())
                            == 0) as ::core::ffi::c_int as size_t,
                    ),
                b", \0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            );
            if part[0 as ::core::ffi::c_int as usize] as ::core::ffi::c_int
                != 'r' as ::core::ffi::c_int
            {
                continue;
            }
            home_replace(
                ::core::ptr::null::<buf_T>(),
                (&raw mut part as *mut ::core::ffi::c_char)
                    .offset(1 as ::core::ffi::c_int as isize),
                NameBuff.ptr() as *mut ::core::ffi::c_char,
                MAXPATHL as size_t,
                true_0 != 0,
            );
            let mut n: size_t = strlen(NameBuff.ptr() as *mut ::core::ffi::c_char);
            if mb_strnicmp(NameBuff.ptr() as *mut ::core::ffi::c_char, new_name, n)
                != 0 as ::core::ffi::c_int
            {
                continue;
            }
            retval = true_0 != 0;
            break;
        }
        xfree(new_name as *mut ::core::ffi::c_void);
        return retval;
    }
}

pub unsafe extern "C" fn get_shada_parameter(mut type_0: ::core::ffi::c_int) -> ::core::ffi::c_int {
    unsafe {
        let mut p: *mut ::core::ffi::c_char = find_shada_parameter(type_0);
        if !p.is_null() && ascii_isdigit(*p as ::core::ffi::c_int) as ::core::ffi::c_int != 0 {
            return atoi(p);
        }
        return -1 as ::core::ffi::c_int;
    }
}

pub unsafe extern "C" fn find_shada_parameter(
    mut type_0: ::core::ffi::c_int,
) -> *mut ::core::ffi::c_char {
    unsafe {
        let mut p: *mut ::core::ffi::c_char = p_shada.get();
        while *p != 0 {
            if *p as ::core::ffi::c_int == type_0 {
                return p.offset(1 as ::core::ffi::c_int as isize);
            }
            if *p as ::core::ffi::c_int == 'n' as ::core::ffi::c_int {
                break;
            } else {
                p = vim_strchr(p, ',' as ::core::ffi::c_int);
                if p.is_null() {
                    break;
                }
                p = p.offset(1);
            }
        }
        return ::core::ptr::null_mut::<::core::ffi::c_char>();
    }
}

pub unsafe extern "C" fn check_marks_read() {
    unsafe {
        if !(*curbuf.get()).b_marks_read
            && get_shada_parameter('\'' as ::core::ffi::c_int) > 0 as ::core::ffi::c_int
            && !(*curbuf.get()).b_ffname.is_null()
        {
            shada_read_marks();
        }
        (*curbuf.get()).b_marks_read = true_0 != 0;
    }
}
