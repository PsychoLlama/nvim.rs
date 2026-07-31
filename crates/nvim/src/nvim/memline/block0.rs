//! Block zero: the swap file's first page.
//!
//! Everything else in a swap file is addressed by block number; block zero is
//! found at offset zero and says what the rest means — the page size, the magic
//! numbers that prove the endianness and type sizes match, and the name,
//! timestamp and inode of the file being edited. `swapfile_info` and its
//! friends read one back to describe it, which is all the `swapinfo()` builtin
//! and the ATTENTION message do.
//!
//! `long_to_char`/`char_to_long` are the little-endian pair every number in
//! block zero goes through.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;

pub unsafe extern "C" fn ml_timestamp(mut buf: *mut buf_T) {
    unsafe {
        ml_upd_block0(buf, UB_FNAME);
    }
}

pub(crate) unsafe extern "C" fn ml_check_b0_id(mut b0p: *mut ZeroBlock) -> bool {
    unsafe {
        return (*b0p).b0_id[0 as ::core::ffi::c_int as usize] as ::core::ffi::c_int
            == BLOCK0_ID0 as ::core::ffi::c_int
            && (*b0p).b0_id[1 as ::core::ffi::c_int as usize] as ::core::ffi::c_int
                == BLOCK0_ID1 as ::core::ffi::c_int;
    }
}

pub(crate) unsafe extern "C" fn ml_check_b0_strings(mut b0p: *mut ZeroBlock) -> bool {
    unsafe {
        return !memchr(
            &raw mut (*b0p).b0_version as *mut ::core::ffi::c_char as *const ::core::ffi::c_void,
            NUL,
            10 as size_t,
        )
        .is_null()
            && !memchr(
                &raw mut (*b0p).b0_uname as *mut ::core::ffi::c_char as *const ::core::ffi::c_void,
                NUL,
                B0_UNAME_SIZE as ::core::ffi::c_int as size_t,
            )
            .is_null()
            && !memchr(
                &raw mut (*b0p).b0_hname as *mut ::core::ffi::c_char as *const ::core::ffi::c_void,
                NUL,
                B0_HNAME_SIZE as ::core::ffi::c_int as size_t,
            )
            .is_null()
            && !memchr(
                &raw mut (*b0p).b0_fname as *mut ::core::ffi::c_char as *const ::core::ffi::c_void,
                NUL,
                B0_FNAME_SIZE_CRYPT as ::core::ffi::c_int as size_t,
            )
            .is_null();
    }
}

pub(crate) unsafe extern "C" fn ml_upd_block0(mut buf: *mut buf_T, mut what: upd_block0_T) {
    unsafe {
        let mut hp: *mut bhdr_T = ::core::ptr::null_mut::<bhdr_T>();
        let mut mfp: *mut memfile_T = (*buf).b_ml.ml_mfp;
        if mfp.is_null() || {
            hp = mf_get(mfp, 0 as blocknr_T, 1 as ::core::ffi::c_uint);
            hp.is_null()
        } {
            return;
        }
        let mut b0p: *mut ZeroBlock = (*hp).bh_data as *mut ZeroBlock;
        if ml_check_b0_id(b0p) as ::core::ffi::c_int == FAIL {
            iemsg(gettext(
                b"E304: ml_upd_block0(): Didn't get block 0??\0".as_ptr()
                    as *const ::core::ffi::c_char,
            ));
        } else if what as ::core::ffi::c_uint
            == UB_FNAME as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            set_b0_fname(b0p, buf);
        } else {
            set_b0_dir_flag(b0p, buf);
        }
        mf_put(mfp, hp, true_0 != 0, false_0 != 0);
    }
}

pub(crate) unsafe extern "C" fn set_b0_fname(mut b0p: *mut ZeroBlock, mut buf: *mut buf_T) {
    unsafe {
        if (*buf).b_ffname.is_null() {
            (*b0p).b0_fname[0 as ::core::ffi::c_int as usize] = NUL as ::core::ffi::c_char;
        } else {
            let mut uname: [::core::ffi::c_char; 40] = [0; 40];
            home_replace(
                ::core::ptr::null::<buf_T>(),
                (*buf).b_ffname,
                &raw mut (*b0p).b0_fname as *mut ::core::ffi::c_char,
                B0_FNAME_SIZE_CRYPT as ::core::ffi::c_int as size_t,
                true_0 != 0,
            );
            if (*b0p).b0_fname[0 as ::core::ffi::c_int as usize] as ::core::ffi::c_int
                == '~' as ::core::ffi::c_int
            {
                let mut retval: ::core::ffi::c_int = os_get_username(
                    &raw mut uname as *mut ::core::ffi::c_char,
                    B0_UNAME_SIZE as ::core::ffi::c_int as size_t,
                );
                let mut ulen: size_t = strlen(&raw mut uname as *mut ::core::ffi::c_char);
                let mut flen: size_t = strlen(&raw mut (*b0p).b0_fname as *mut ::core::ffi::c_char);
                if retval == FAIL
                    || ulen.wrapping_add(flen)
                        > (B0_FNAME_SIZE_CRYPT as ::core::ffi::c_int - 1 as ::core::ffi::c_int)
                            as size_t
                {
                    xstrlcpy(
                        &raw mut (*b0p).b0_fname as *mut ::core::ffi::c_char,
                        (*buf).b_ffname,
                        B0_FNAME_SIZE_CRYPT as ::core::ffi::c_int as size_t,
                    );
                } else {
                    memmove(
                        (&raw mut (*b0p).b0_fname as *mut ::core::ffi::c_char)
                            .offset(ulen as isize)
                            .offset(1 as ::core::ffi::c_int as isize)
                            as *mut ::core::ffi::c_void,
                        (&raw mut (*b0p).b0_fname as *mut ::core::ffi::c_char)
                            .offset(1 as ::core::ffi::c_int as isize)
                            as *const ::core::ffi::c_void,
                        flen,
                    );
                    memmove(
                        (&raw mut (*b0p).b0_fname as *mut ::core::ffi::c_char)
                            .offset(1 as ::core::ffi::c_int as isize)
                            as *mut ::core::ffi::c_void,
                        &raw mut uname as *mut ::core::ffi::c_char as *const ::core::ffi::c_void,
                        ulen,
                    );
                }
            }
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
            if os_fileinfo((*buf).b_ffname, &raw mut file_info) {
                long_to_char(
                    file_info.stat.st_mtim.tv_sec,
                    &raw mut (*b0p).b0_mtime as *mut ::core::ffi::c_char,
                );
                long_to_char(
                    os_fileinfo_inode(&raw mut file_info) as ::core::ffi::c_long,
                    &raw mut (*b0p).b0_ino as *mut ::core::ffi::c_char,
                );
                buf_store_file_info(buf, &raw mut file_info);
                (*buf).b_mtime_read = (*buf).b_mtime;
                (*buf).b_mtime_read_ns = (*buf).b_mtime_ns;
            } else {
                long_to_char(
                    0 as ::core::ffi::c_long,
                    &raw mut (*b0p).b0_mtime as *mut ::core::ffi::c_char,
                );
                long_to_char(
                    0 as ::core::ffi::c_long,
                    &raw mut (*b0p).b0_ino as *mut ::core::ffi::c_char,
                );
                (*buf).b_mtime = 0 as int64_t;
                (*buf).b_mtime_ns = 0 as int64_t;
                (*buf).b_mtime_read = 0 as int64_t;
                (*buf).b_mtime_read_ns = 0 as int64_t;
                (*buf).b_orig_size = 0 as uint64_t;
                (*buf).b_orig_mode = 0 as ::core::ffi::c_int;
            }
        }
        add_b0_fenc(b0p, curbuf.get());
    }
}

pub(crate) unsafe extern "C" fn set_b0_dir_flag(mut b0p: *mut ZeroBlock, mut buf: *mut buf_T) {
    unsafe {
        if same_directory((*(*buf).b_ml.ml_mfp).mf_fname, (*buf).b_ffname) {
            (*b0p).b0_fname
                [(B0_FNAME_SIZE_ORG as ::core::ffi::c_int - 2 as ::core::ffi::c_int) as usize] =
                ((*b0p).b0_fname
                    [(B0_FNAME_SIZE_ORG as ::core::ffi::c_int - 2 as ::core::ffi::c_int) as usize]
                    as ::core::ffi::c_int
                    | B0_SAME_DIR) as ::core::ffi::c_char;
        } else {
            (*b0p).b0_fname
                [(B0_FNAME_SIZE_ORG as ::core::ffi::c_int - 2 as ::core::ffi::c_int) as usize] =
                ((*b0p).b0_fname
                    [(B0_FNAME_SIZE_ORG as ::core::ffi::c_int - 2 as ::core::ffi::c_int) as usize]
                    as ::core::ffi::c_int
                    & !B0_SAME_DIR) as ::core::ffi::c_char;
        };
    }
}

pub(crate) unsafe extern "C" fn add_b0_fenc(mut b0p: *mut ZeroBlock, mut buf: *mut buf_T) {
    unsafe {
        let size: ::core::ffi::c_int = B0_FNAME_SIZE_NOCRYPT as ::core::ffi::c_int;
        let mut n: ::core::ffi::c_int = strlen((*buf).b_p_fenc) as ::core::ffi::c_int;
        if strlen(&raw mut (*b0p).b0_fname as *mut ::core::ffi::c_char) as ::core::ffi::c_int
            + n
            + 1 as ::core::ffi::c_int
            > size
        {
            (*b0p).b0_fname
                [(B0_FNAME_SIZE_ORG as ::core::ffi::c_int - 2 as ::core::ffi::c_int) as usize] =
                ((*b0p).b0_fname
                    [(B0_FNAME_SIZE_ORG as ::core::ffi::c_int - 2 as ::core::ffi::c_int) as usize]
                    as ::core::ffi::c_int
                    & !B0_HAS_FENC) as ::core::ffi::c_char;
        } else {
            memmove(
                (&raw mut (*b0p).b0_fname as *mut ::core::ffi::c_char)
                    .offset(size as isize)
                    .offset(-(n as isize)) as *mut ::core::ffi::c_void,
                (*buf).b_p_fenc as *const ::core::ffi::c_void,
                n as size_t,
            );
            *(&raw mut (*b0p).b0_fname as *mut ::core::ffi::c_char)
                .offset(size as isize)
                .offset(-(n as isize))
                .offset(-(1 as ::core::ffi::c_int as isize)) = NUL as ::core::ffi::c_char;
            (*b0p).b0_fname
                [(B0_FNAME_SIZE_ORG as ::core::ffi::c_int - 2 as ::core::ffi::c_int) as usize] =
                ((*b0p).b0_fname
                    [(B0_FNAME_SIZE_ORG as ::core::ffi::c_int - 2 as ::core::ffi::c_int) as usize]
                    as ::core::ffi::c_int
                    | B0_HAS_FENC) as ::core::ffi::c_char;
        };
    }
}

pub(crate) unsafe extern "C" fn swapfile_proc_running(
    mut b0p: *const ZeroBlock,
    mut swap_fname: *const ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    unsafe {
        let mut st: FileInfo = FileInfo {
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
        let mut uptime: ::core::ffi::c_double = 0.;
        if os_fileinfo(swap_fname, &raw mut st) as ::core::ffi::c_int != 0
            && uv_uptime(&raw mut uptime) == 0 as ::core::ffi::c_int
            && (st.stat.st_mtim.tv_sec as Timestamp) < os_time().wrapping_sub(uptime as Timestamp)
        {
            return 0 as ::core::ffi::c_int;
        }
        let mut pid: ::core::ffi::c_int =
            char_to_long(&raw const (*b0p).b0_pid as *const ::core::ffi::c_char)
                as ::core::ffi::c_int;
        return if os_proc_running(pid) as ::core::ffi::c_int != 0 {
            pid
        } else {
            0 as ::core::ffi::c_int
        };
    }
}

pub unsafe extern "C" fn swapfile_dict(mut fname: *const ::core::ffi::c_char, mut d: *mut dict_T) {
    unsafe {
        let mut fd: ::core::ffi::c_int = 0;
        let mut b0: ZeroBlock = ZeroBlock {
            b0_id: [0; 2],
            b0_version: [0; 10],
            b0_page_size: [0; 4],
            b0_mtime: [0; 4],
            b0_ino: [0; 4],
            b0_pid: [0; 4],
            b0_uname: [0; 40],
            b0_hname: [0; 40],
            b0_fname: [0; 900],
            b0_magic_long: 0,
            b0_magic_int: 0,
            b0_magic_short: 0,
            b0_magic_char: 0,
        };
        fd = os_open(fname, O_RDONLY, 0 as ::core::ffi::c_int);
        if fd >= 0 as ::core::ffi::c_int {
            if read_eintr(
                fd,
                &raw mut b0 as *mut ::core::ffi::c_void,
                ::core::mem::size_of::<ZeroBlock>(),
            ) as usize
                == ::core::mem::size_of::<ZeroBlock>()
            {
                if ml_check_b0_id(&raw mut b0) as ::core::ffi::c_int == FAIL {
                    tv_dict_add_str(
                        d,
                        b"error\0".as_ptr() as *const ::core::ffi::c_char,
                        ::core::mem::size_of::<[::core::ffi::c_char; 6]>()
                            .wrapping_sub(1 as size_t),
                        b"Not a swap file\0".as_ptr() as *const ::core::ffi::c_char,
                    );
                } else if b0_magic_wrong(&raw mut b0) != 0 {
                    tv_dict_add_str(
                        d,
                        b"error\0".as_ptr() as *const ::core::ffi::c_char,
                        ::core::mem::size_of::<[::core::ffi::c_char; 6]>()
                            .wrapping_sub(1 as size_t),
                        b"Magic number mismatch\0".as_ptr() as *const ::core::ffi::c_char,
                    );
                } else {
                    tv_dict_add_str_len(
                        d,
                        b"version\0".as_ptr() as *const ::core::ffi::c_char,
                        ::core::mem::size_of::<[::core::ffi::c_char; 8]>()
                            .wrapping_sub(1 as size_t),
                        &raw mut b0.b0_version as *mut ::core::ffi::c_char,
                        10 as ::core::ffi::c_int,
                    );
                    tv_dict_add_str_len(
                        d,
                        b"user\0".as_ptr() as *const ::core::ffi::c_char,
                        ::core::mem::size_of::<[::core::ffi::c_char; 5]>()
                            .wrapping_sub(1 as size_t),
                        &raw mut b0.b0_uname as *mut ::core::ffi::c_char,
                        B0_UNAME_SIZE as ::core::ffi::c_int,
                    );
                    tv_dict_add_str_len(
                        d,
                        b"host\0".as_ptr() as *const ::core::ffi::c_char,
                        ::core::mem::size_of::<[::core::ffi::c_char; 5]>()
                            .wrapping_sub(1 as size_t),
                        &raw mut b0.b0_hname as *mut ::core::ffi::c_char,
                        B0_HNAME_SIZE as ::core::ffi::c_int,
                    );
                    tv_dict_add_str_len(
                        d,
                        b"fname\0".as_ptr() as *const ::core::ffi::c_char,
                        ::core::mem::size_of::<[::core::ffi::c_char; 6]>()
                            .wrapping_sub(1 as size_t),
                        &raw mut b0.b0_fname as *mut ::core::ffi::c_char,
                        B0_FNAME_SIZE_ORG as ::core::ffi::c_int,
                    );
                    tv_dict_add_nr(
                        d,
                        b"pid\0".as_ptr() as *const ::core::ffi::c_char,
                        ::core::mem::size_of::<[::core::ffi::c_char; 4]>()
                            .wrapping_sub(1 as size_t),
                        swapfile_proc_running(&raw mut b0, fname) as varnumber_T,
                    );
                    tv_dict_add_nr(
                        d,
                        b"mtime\0".as_ptr() as *const ::core::ffi::c_char,
                        ::core::mem::size_of::<[::core::ffi::c_char; 6]>()
                            .wrapping_sub(1 as size_t),
                        char_to_long(&raw mut b0.b0_mtime as *mut ::core::ffi::c_char)
                            as varnumber_T,
                    );
                    tv_dict_add_nr(
                        d,
                        b"dirty\0".as_ptr() as *const ::core::ffi::c_char,
                        ::core::mem::size_of::<[::core::ffi::c_char; 6]>()
                            .wrapping_sub(1 as size_t),
                        (if b0.b0_fname[(B0_FNAME_SIZE_ORG as ::core::ffi::c_int
                            - 1 as ::core::ffi::c_int)
                            as usize] as ::core::ffi::c_int
                            != 0
                        {
                            1 as ::core::ffi::c_int
                        } else {
                            0 as ::core::ffi::c_int
                        }) as varnumber_T,
                    );
                    tv_dict_add_nr(
                        d,
                        b"inode\0".as_ptr() as *const ::core::ffi::c_char,
                        ::core::mem::size_of::<[::core::ffi::c_char; 6]>()
                            .wrapping_sub(1 as size_t),
                        char_to_long(&raw mut b0.b0_ino as *mut ::core::ffi::c_char) as varnumber_T,
                    );
                }
            } else {
                tv_dict_add_str(
                    d,
                    b"error\0".as_ptr() as *const ::core::ffi::c_char,
                    ::core::mem::size_of::<[::core::ffi::c_char; 6]>().wrapping_sub(1 as size_t),
                    b"Cannot read file\0".as_ptr() as *const ::core::ffi::c_char,
                );
            }
            close(fd);
        } else {
            tv_dict_add_str(
                d,
                b"error\0".as_ptr() as *const ::core::ffi::c_char,
                ::core::mem::size_of::<[::core::ffi::c_char; 6]>().wrapping_sub(1 as size_t),
                b"Cannot open file\0".as_ptr() as *const ::core::ffi::c_char,
            );
        };
    }
}

pub(crate) unsafe extern "C" fn swapfile_info(
    mut fname: *mut ::core::ffi::c_char,
    mut msg_0: *mut StringBuilder,
) -> time_t {
    unsafe {
        '_c2rust_label: {
            if !fname.is_null() {
            } else {
                __assert_fail(
                    b"fname != NULL\0".as_ptr() as *const ::core::ffi::c_char,
                    b"src/nvim/memline.rs\0".as_ptr() as *const ::core::ffi::c_char,
                    1545 as ::core::ffi::c_uint,
                    b"time_t swapfile_info(char *, StringBuilder *)\0".as_ptr()
                        as *const ::core::ffi::c_char,
                );
            }
        };
        let mut b0: ZeroBlock = ZeroBlock {
            b0_id: [0; 2],
            b0_version: [0; 10],
            b0_page_size: [0; 4],
            b0_mtime: [0; 4],
            b0_ino: [0; 4],
            b0_pid: [0; 4],
            b0_uname: [0; 40],
            b0_hname: [0; 40],
            b0_fname: [0; 900],
            b0_magic_long: 0,
            b0_magic_int: 0,
            b0_magic_short: 0,
            b0_magic_char: 0,
        };
        let mut x: time_t = 0 as ::core::ffi::c_int as time_t;
        let mut uname: [::core::ffi::c_char; 40] = [0; 40];
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
            if os_get_uname(
                file_info.stat.st_uid as uv_uid_t,
                &raw mut uname as *mut ::core::ffi::c_char,
                B0_UNAME_SIZE as ::core::ffi::c_int as size_t,
            ) == OK
            {
                kv_do_printf(
                    msg_0,
                    b"%s%s\0".as_ptr() as *const ::core::ffi::c_char,
                    gettext(b"          owned by: \0".as_ptr() as *const ::core::ffi::c_char),
                    &raw mut uname as *mut ::core::ffi::c_char,
                );
                kv_do_printf(
                    msg_0,
                    gettext(b"   dated: \0".as_ptr() as *const ::core::ffi::c_char),
                );
            } else {
                kv_do_printf(
                    msg_0,
                    gettext(b"             dated: \0".as_ptr() as *const ::core::ffi::c_char),
                );
            }
            x = file_info.stat.st_mtim.tv_sec as time_t;
            let mut ctime_buf: [::core::ffi::c_char; 100] = [0; 100];
            kv_do_printf(
                msg_0,
                b"%s\0".as_ptr() as *const ::core::ffi::c_char,
                os_ctime_r(x, &mut ctime_buf, true),
            );
        }
        let mut fd: ::core::ffi::c_int = os_open(fname, O_RDONLY, 0 as ::core::ffi::c_int);
        if fd >= 0 as ::core::ffi::c_int {
            if read_eintr(
                fd,
                &raw mut b0 as *mut ::core::ffi::c_void,
                ::core::mem::size_of::<ZeroBlock>(),
            ) as usize
                == ::core::mem::size_of::<ZeroBlock>()
            {
                if strncmp(
                    &raw mut b0.b0_version as *mut ::core::ffi::c_char,
                    b"VIM 3.0\0".as_ptr() as *const ::core::ffi::c_char,
                    7 as size_t,
                ) == 0 as ::core::ffi::c_int
                {
                    kv_do_printf(
                        msg_0,
                        gettext(b"         [from Vim version 3.0]\0".as_ptr()
                            as *const ::core::ffi::c_char),
                    );
                } else if ml_check_b0_id(&raw mut b0) as ::core::ffi::c_int == FAIL {
                    kv_do_printf(
                        msg_0,
                        gettext(b"         [does not look like a Nvim swap file]\0".as_ptr()
                            as *const ::core::ffi::c_char),
                    );
                } else if !ml_check_b0_strings(&raw mut b0) {
                    kv_do_printf(
                        msg_0,
                        gettext(
                            b"         [garbled strings (not nul terminated)]\0".as_ptr()
                                as *const ::core::ffi::c_char,
                        ),
                    );
                } else {
                    kv_do_printf(
                        msg_0,
                        gettext(b"         file name: \0".as_ptr() as *const ::core::ffi::c_char),
                    );
                    if b0.b0_fname[0 as ::core::ffi::c_int as usize] as ::core::ffi::c_int == NUL {
                        kv_do_printf(
                            msg_0,
                            gettext(b"[No Name]\0".as_ptr() as *const ::core::ffi::c_char),
                        );
                    } else {
                        kv_do_printf(
                            msg_0,
                            b"%s\0".as_ptr() as *const ::core::ffi::c_char,
                            &raw mut b0.b0_fname as *mut ::core::ffi::c_char,
                        );
                    }
                    kv_do_printf(
                        msg_0,
                        gettext(b"\n          modified: \0".as_ptr() as *const ::core::ffi::c_char),
                    );
                    kv_do_printf(
                        msg_0,
                        if b0.b0_fname[(B0_FNAME_SIZE_ORG as ::core::ffi::c_int
                            - 1 as ::core::ffi::c_int)
                            as usize] as ::core::ffi::c_int
                            != 0
                        {
                            gettext(b"YES\0".as_ptr() as *const ::core::ffi::c_char)
                        } else {
                            gettext(b"no\0".as_ptr() as *const ::core::ffi::c_char)
                        },
                    );
                    if *(&raw mut b0.b0_uname as *mut ::core::ffi::c_char) as ::core::ffi::c_int
                        != NUL
                    {
                        kv_do_printf(
                            msg_0,
                            gettext(
                                b"\n         user name: \0".as_ptr() as *const ::core::ffi::c_char
                            ),
                        );
                        kv_do_printf(
                            msg_0,
                            b"%s\0".as_ptr() as *const ::core::ffi::c_char,
                            &raw mut b0.b0_uname as *mut ::core::ffi::c_char,
                        );
                    }
                    if *(&raw mut b0.b0_hname as *mut ::core::ffi::c_char) as ::core::ffi::c_int
                        != NUL
                    {
                        if *(&raw mut b0.b0_uname as *mut ::core::ffi::c_char) as ::core::ffi::c_int
                            != NUL
                        {
                            kv_do_printf(
                                msg_0,
                                gettext(b"   host name: \0".as_ptr() as *const ::core::ffi::c_char),
                            );
                        } else {
                            kv_do_printf(
                                msg_0,
                                gettext(b"\n         host name: \0".as_ptr()
                                    as *const ::core::ffi::c_char),
                            );
                        }
                        kv_do_printf(
                            msg_0,
                            b"%s\0".as_ptr() as *const ::core::ffi::c_char,
                            &raw mut b0.b0_hname as *mut ::core::ffi::c_char,
                        );
                    }
                    if char_to_long(&raw mut b0.b0_pid as *mut ::core::ffi::c_char)
                        != 0 as ::core::ffi::c_long
                    {
                        kv_do_printf(
                            msg_0,
                            gettext(
                                b"\n        process ID: \0".as_ptr() as *const ::core::ffi::c_char
                            ),
                        );
                        kv_do_printf(
                            msg_0,
                            b"%d\0".as_ptr() as *const ::core::ffi::c_char,
                            char_to_long(&raw mut b0.b0_pid as *mut ::core::ffi::c_char)
                                as ::core::ffi::c_int,
                        );
                        proc_running.set(swapfile_proc_running(&raw mut b0, fname));
                        if proc_running.get() != 0 {
                            kv_do_printf(
                                msg_0,
                                gettext(
                                    b" (STILL RUNNING)\0".as_ptr() as *const ::core::ffi::c_char
                                ),
                            );
                        }
                    }
                    if b0_magic_wrong(&raw mut b0) != 0 {
                        kv_do_printf(
                            msg_0,
                            gettext(b"\n         [not usable on this computer]\0".as_ptr()
                                as *const ::core::ffi::c_char),
                        );
                    }
                }
            } else {
                kv_do_printf(
                    msg_0,
                    gettext(b"         [cannot be read]\0".as_ptr() as *const ::core::ffi::c_char),
                );
            }
            close(fd);
        } else {
            kv_do_printf(
                msg_0,
                gettext(b"         [cannot be opened]\0".as_ptr() as *const ::core::ffi::c_char),
            );
        }
        kv_do_printf(msg_0, b"\n\0".as_ptr() as *const ::core::ffi::c_char);
        return x;
    }
}

pub(crate) unsafe extern "C" fn swapfile_unchanged(mut fname: *mut ::core::ffi::c_char) -> bool {
    unsafe {
        let mut b0: ZeroBlock = ZeroBlock {
            b0_id: [0; 2],
            b0_version: [0; 10],
            b0_page_size: [0; 4],
            b0_mtime: [0; 4],
            b0_ino: [0; 4],
            b0_pid: [0; 4],
            b0_uname: [0; 40],
            b0_hname: [0; 40],
            b0_fname: [0; 900],
            b0_magic_long: 0,
            b0_magic_int: 0,
            b0_magic_short: 0,
            b0_magic_char: 0,
        };
        if !os_path_exists(fname) {
            return false_0 != 0;
        }
        let mut fd: ::core::ffi::c_int = os_open(fname, O_RDONLY, 0 as ::core::ffi::c_int);
        if fd < 0 as ::core::ffi::c_int {
            return false_0 != 0;
        }
        if read_eintr(
            fd,
            &raw mut b0 as *mut ::core::ffi::c_void,
            ::core::mem::size_of::<ZeroBlock>(),
        ) as usize
            != ::core::mem::size_of::<ZeroBlock>()
        {
            close(fd);
            return false_0 != 0;
        }
        let mut ret: bool = true_0 != 0;
        if ml_check_b0_id(&raw mut b0) as ::core::ffi::c_int == FAIL
            || b0_magic_wrong(&raw mut b0) != 0
        {
            ret = false_0 != 0;
        }
        if b0.b0_fname[(B0_FNAME_SIZE_ORG as ::core::ffi::c_int - 1 as ::core::ffi::c_int) as usize]
            != 0
        {
            ret = false_0 != 0;
        }
        if *(&raw mut b0.b0_hname as *mut ::core::ffi::c_char) as ::core::ffi::c_int == NUL {
            ret = false_0 != 0;
        } else {
            let mut hostname: [::core::ffi::c_char; 40] = [0; 40];
            os_get_hostname(
                &raw mut hostname as *mut ::core::ffi::c_char,
                B0_HNAME_SIZE as ::core::ffi::c_int as size_t,
            );
            hostname[(B0_HNAME_SIZE as ::core::ffi::c_int - 1 as ::core::ffi::c_int) as usize] =
                NUL as ::core::ffi::c_char;
            b0.b0_hname[(B0_HNAME_SIZE as ::core::ffi::c_int - 1 as ::core::ffi::c_int) as usize] =
                NUL as ::core::ffi::c_char;
            if strcasecmp(
                &raw mut b0.b0_hname as *mut ::core::ffi::c_char,
                &raw mut hostname as *mut ::core::ffi::c_char,
            ) != 0 as ::core::ffi::c_int
            {
                ret = false_0 != 0;
            }
        }
        if char_to_long(&raw mut b0.b0_pid as *mut ::core::ffi::c_char) == 0 as ::core::ffi::c_long
            || swapfile_proc_running(&raw mut b0, fname) != 0
        {
            ret = false_0 != 0;
        }
        close(fd);
        return ret;
    }
}

pub(crate) unsafe extern "C" fn b0_magic_wrong(mut b0p: *mut ZeroBlock) -> ::core::ffi::c_int {
    unsafe {
        return ((*b0p).b0_magic_long != B0_MAGIC_LONG as ::core::ffi::c_int as ::core::ffi::c_long
            || (*b0p).b0_magic_int != B0_MAGIC_INT as ::core::ffi::c_int
            || (*b0p).b0_magic_short as ::core::ffi::c_int
                != B0_MAGIC_SHORT as ::core::ffi::c_int as int16_t as ::core::ffi::c_int
            || (*b0p).b0_magic_char as ::core::ffi::c_int != B0_MAGIC_CHAR as ::core::ffi::c_int)
            as ::core::ffi::c_int;
    }
}

pub(crate) unsafe extern "C" fn fnamecmp_ino(
    mut fname_c: *mut ::core::ffi::c_char,
    mut fname_s: *mut ::core::ffi::c_char,
    mut ino_block0: ::core::ffi::c_long,
) -> bool {
    unsafe {
        let mut ino_c: uint64_t = 0 as uint64_t;
        let mut ino_s: uint64_t = 0;
        let mut buf_c: [::core::ffi::c_char; 4096] = [0; 4096];
        let mut buf_s: [::core::ffi::c_char; 4096] = [0; 4096];
        let mut retval_c: ::core::ffi::c_int = 0;
        let mut retval_s: ::core::ffi::c_int = 0;
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
        if os_fileinfo(fname_c, &raw mut file_info) {
            ino_c = os_fileinfo_inode(&raw mut file_info);
        }
        if os_fileinfo(fname_s, &raw mut file_info) {
            ino_s = os_fileinfo_inode(&raw mut file_info);
        } else {
            ino_s = ino_block0 as uint64_t;
        }
        if ino_c != 0 && ino_s != 0 {
            return ino_c != ino_s;
        }
        retval_c = vim_FullName(
            fname_c,
            &raw mut buf_c as *mut ::core::ffi::c_char,
            MAXPATHL as size_t,
            true_0 != 0,
        );
        retval_s = vim_FullName(
            fname_s,
            &raw mut buf_s as *mut ::core::ffi::c_char,
            MAXPATHL as size_t,
            true_0 != 0,
        );
        if retval_c == OK && retval_s == OK {
            return strcmp(
                &raw mut buf_c as *mut ::core::ffi::c_char,
                &raw mut buf_s as *mut ::core::ffi::c_char,
            ) != 0 as ::core::ffi::c_int;
        }
        if ino_s == 0 as uint64_t && ino_c == 0 as uint64_t && retval_c == FAIL && retval_s == FAIL
        {
            return strcmp(fname_c, fname_s) != 0 as ::core::ffi::c_int;
        }
        return true_0 != 0;
    }
}

pub(crate) unsafe extern "C" fn long_to_char(
    mut n: ::core::ffi::c_long,
    mut s_in: *mut ::core::ffi::c_char,
) {
    unsafe {
        let mut s: *mut uint8_t = s_in as *mut uint8_t;
        *s.offset(0 as ::core::ffi::c_int as isize) = (n & 0xff as ::core::ffi::c_long) as uint8_t;
        n = (n as ::core::ffi::c_uint >> 8 as ::core::ffi::c_int) as ::core::ffi::c_long;
        *s.offset(1 as ::core::ffi::c_int as isize) = (n & 0xff as ::core::ffi::c_long) as uint8_t;
        n = (n as ::core::ffi::c_uint >> 8 as ::core::ffi::c_int) as ::core::ffi::c_long;
        *s.offset(2 as ::core::ffi::c_int as isize) = (n & 0xff as ::core::ffi::c_long) as uint8_t;
        n = (n as ::core::ffi::c_uint >> 8 as ::core::ffi::c_int) as ::core::ffi::c_long;
        *s.offset(3 as ::core::ffi::c_int as isize) = (n & 0xff as ::core::ffi::c_long) as uint8_t;
    }
}

pub(crate) unsafe extern "C" fn char_to_long(
    mut s_in: *const ::core::ffi::c_char,
) -> ::core::ffi::c_long {
    unsafe {
        let mut s: *const uint8_t = s_in as *mut uint8_t;
        let mut retval: ::core::ffi::c_long =
            *s.offset(3 as ::core::ffi::c_int as isize) as ::core::ffi::c_long;
        retval <<= 8 as ::core::ffi::c_int;
        retval |= *s.offset(2 as ::core::ffi::c_int as isize) as ::core::ffi::c_long;
        retval <<= 8 as ::core::ffi::c_int;
        retval |= *s.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_long;
        retval <<= 8 as ::core::ffi::c_int;
        retval |= *s.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_long;
        return retval;
    }
}

pub unsafe extern "C" fn ml_setflags(mut buf: *mut buf_T) {
    unsafe {
        if (*buf).b_ml.ml_mfp.is_null() {
            return;
        }
        let mut hp: *mut bhdr_T = mf_find((*buf).b_ml.ml_mfp, 0 as blocknr_T);
        if !hp.is_null() {
            let mut b0p: *mut ZeroBlock = (*hp).bh_data as *mut ZeroBlock;
            (*b0p).b0_fname
                [(B0_FNAME_SIZE_ORG as ::core::ffi::c_int - 1 as ::core::ffi::c_int) as usize] =
                (if (*buf).b_changed != 0 {
                    B0_DIRTY
                } else {
                    0 as ::core::ffi::c_int
                }) as ::core::ffi::c_char;
            (*b0p).b0_fname
                [(B0_FNAME_SIZE_ORG as ::core::ffi::c_int - 2 as ::core::ffi::c_int) as usize] =
                ((*b0p).b0_fname
                    [(B0_FNAME_SIZE_ORG as ::core::ffi::c_int - 2 as ::core::ffi::c_int) as usize]
                    as ::core::ffi::c_int
                    & !B0_FF_MASK
                    | (get_fileformat(buf) + 1 as ::core::ffi::c_int) as uint8_t
                        as ::core::ffi::c_int) as ::core::ffi::c_char;
            add_b0_fenc(b0p, buf);
            (*hp).bh_flags |= BH_DIRTY;
            mf_sync((*buf).b_ml.ml_mfp, MFS_ZERO as ::core::ffi::c_int);
        }
    }
}
