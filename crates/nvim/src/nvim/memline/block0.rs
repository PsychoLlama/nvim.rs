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

use core::ffi::{c_char, c_int, c_long};

#[allow(unused_imports)]
use super::*;

impl ZeroBlock {
    /// Upstream keeps two more fields in the last two bytes of the name:
    /// `#define b0_dirty b0_fname[B0_FNAME_SIZE_ORG - 1]` and
    /// `#define b0_flags b0_fname[B0_FNAME_SIZE_ORG - 2]`. That is why the
    /// name itself is only ever written up to `B0_FNAME_SIZE_NOCRYPT`.
    const DIRTY: usize = B0_FNAME_SIZE_ORG as usize - 1;
    const FLAGS: usize = B0_FNAME_SIZE_ORG as usize - 2;

    /// [`B0_SAME_DIR`], [`B0_HAS_FENC`] and the `'fileformat'` in
    /// [`B0_FF_MASK`].
    fn flags(&self) -> c_int {
        self.b0_fname[Self::FLAGS] as c_int
    }

    fn set_flags(&mut self, flags: c_int) {
        self.b0_fname[Self::FLAGS] = flags as c_char;
    }

    fn set_flag(&mut self, flag: c_int, on: bool) {
        let flags = self.flags();
        self.set_flags(if on { flags | flag } else { flags & !flag });
    }

    /// Whether the buffer had unsaved changes when this was last written.
    /// It is what makes `:recover` worth offering.
    fn set_dirty(&mut self, dirty: bool) {
        self.b0_fname[Self::DIRTY] = if dirty { B0_DIRTY } else { 0 } as c_char;
    }
}

/// Record the file's timestamp in the swap file, after it has been written.
pub unsafe fn ml_timestamp(buf: *mut buf_T) {
    unsafe { ml_upd_block0(buf, UB_FNAME) }
}

/// Whether the two bytes that identify a swap file are at the head of this
/// block. They are the first thing read back, and a mismatch means the file
/// is not a swap file at all.
pub(crate) unsafe fn ml_check_b0_id(b0p: *mut ZeroBlock) -> bool {
    unsafe {
        (*b0p).b0_id[0] as c_int == BLOCK0_ID0 as c_int
            && (*b0p).b0_id[1] as c_int == BLOCK0_ID1 as c_int
    }
}

/// Whether every string in the block is terminated inside its own field.
/// A swap file is read from disk, so nothing may be assumed about it: an
/// unterminated name would be read past by everything downstream.
pub(crate) unsafe fn ml_check_b0_strings(b0p: *mut ZeroBlock) -> bool {
    unsafe {
        let b0 = &*b0p;
        b0.b0_version.contains(&0)
            && b0.b0_uname.contains(&0)
            && b0.b0_hname.contains(&0)
            && b0.b0_fname[..B0_FNAME_SIZE_CRYPT as usize].contains(&0)
    }
}

/// Bring block zero up to date with the buffer: either the file name and
/// timestamp ([`UB_FNAME`]), or the "swap file is beside the file" flag
/// ([`UB_SAME_DIR`]).
pub(crate) unsafe fn ml_upd_block0(buf: *mut buf_T, what: upd_block0_T) {
    unsafe {
        let mfp = (*buf).b_ml.ml_mfp;
        if mfp.is_null() {
            return;
        }
        let hp = mf_get(mfp, 0, 1);
        if hp.is_null() {
            return;
        }

        let b0p = (*hp).bh_data as *mut ZeroBlock;
        if !ml_check_b0_id(b0p) {
            iemsg(gettext(
                c"E304: ml_upd_block0(): Didn't get block 0??".as_ptr(),
            ));
        } else if what == UB_FNAME {
            set_b0_fname(b0p, buf);
        } else {
            set_b0_dir_flag(b0p, buf);
        }
        mf_put(mfp, hp, true, false);
    }
}

/// Write the file's name, timestamp and inode into block zero, and set
/// `buf->b_mtime` from the same `stat`.
///
/// Must not use `NameBuff`: it is in use by some of the callers.
pub(crate) unsafe fn set_b0_fname(b0p: *mut ZeroBlock, buf: *mut buf_T) {
    unsafe {
        if (*buf).b_ffname.is_null() {
            (*b0p).b0_fname[0] = NUL as c_char;
        } else {
            // A file under the current user's home directory is recorded as
            // "~user/...", so that the same file opened from another machine
            // over a network is still recognised as the same file.
            // `home_replace` writes "~/", and the user name is spliced in
            // after the tilde.
            let name = &mut (*b0p).b0_fname;
            home_replace(
                core::ptr::null::<buf_T>(),
                (*buf).b_ffname,
                name.as_mut_ptr(),
                B0_FNAME_SIZE_CRYPT as size_t,
                true,
            );
            if name[0] as c_int == '~' as c_int {
                let mut uname: [c_char; B0_UNAME_SIZE as usize] = [0; B0_UNAME_SIZE as usize];
                let named = os_get_username(uname.as_mut_ptr(), B0_UNAME_SIZE as size_t);
                let ulen = strlen(uname.as_ptr()) as usize;
                // `flen` counts the bytes after the tilde *including* the
                // terminator, which is exactly what has to shift right.
                let flen = strlen(name.as_ptr()) as usize;
                if named == FAIL || ulen + flen > B0_FNAME_SIZE_CRYPT as usize - 1 {
                    // No user name, or no room for one: keep the path as it
                    // was given.
                    xstrlcpy(
                        name.as_mut_ptr(),
                        (*buf).b_ffname,
                        B0_FNAME_SIZE_CRYPT as size_t,
                    );
                } else {
                    name.copy_within(1..1 + flen, 1 + ulen);
                    name[1..1 + ulen].copy_from_slice(&uname[..ulen]);
                }
            }

            let mut file_info: FileInfo = core::mem::zeroed();
            if os_fileinfo((*buf).b_ffname, &raw mut file_info) {
                b0_store_number(
                    file_info.stat.st_mtim.tv_sec,
                    (&raw mut (*b0p).b0_mtime).cast::<c_char>(),
                );
                b0_store_number(
                    os_fileinfo_inode(&raw mut file_info) as c_long,
                    (&raw mut (*b0p).b0_ino).cast::<c_char>(),
                );
                buf_store_file_info(buf, &raw mut file_info);
                (*buf).b_mtime_read = (*buf).b_mtime;
                (*buf).b_mtime_read_ns = (*buf).b_mtime_ns;
            } else {
                b0_store_number(0, (&raw mut (*b0p).b0_mtime).cast::<c_char>());
                b0_store_number(0, (&raw mut (*b0p).b0_ino).cast::<c_char>());
                (*buf).b_mtime = 0;
                (*buf).b_mtime_ns = 0;
                (*buf).b_mtime_read = 0;
                (*buf).b_mtime_read_ns = 0;
                (*buf).b_orig_size = 0;
                (*buf).b_orig_mode = 0;
            }
        }

        // Upstream passes `curbuf` here, not `buf`. Preserved: the two are
        // the same for every reachable caller.
        add_b0_fenc(b0p, curbuf.get());
    }
}

/// Record whether the file and its swap file are in the same directory.
///
/// Fail safe: anything short of proof leaves the flag clear.
pub(crate) unsafe fn set_b0_dir_flag(b0p: *mut ZeroBlock, buf: *mut buf_T) {
    unsafe {
        let same = same_directory((*(*buf).b_ml.ml_mfp).mf_fname, (*buf).b_ffname);
        (*b0p).set_flag(B0_SAME_DIR, same);
    }
}

/// Append the buffer's `'fileencoding'` to block zero, if it fits.
///
/// It goes at the *end* of the name field with a NUL in front of it, so a
/// reader that does not know about [`B0_HAS_FENC`] still sees a terminated
/// name and never reaches the encoding.
pub(crate) unsafe fn add_b0_fenc(b0p: *mut ZeroBlock, buf: *mut buf_T) {
    unsafe {
        let size = B0_FNAME_SIZE_NOCRYPT as usize;
        let fenc = (*buf).b_p_fenc;
        let n = strlen(fenc) as usize;
        let name = &mut (*b0p).b0_fname;
        let fits = strlen(name.as_ptr()) as usize + n + 1 <= size;
        if fits {
            let at = size - n;
            core::ptr::copy_nonoverlapping(fenc, name[at..].as_mut_ptr(), n);
            name[at - 1] = NUL as c_char;
        }
        (*b0p).set_flag(B0_HAS_FENC, fits);
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
            b0_read_number(&raw const (*b0p).b0_pid as *const ::core::ffi::c_char)
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
                } else if b0_magic_wrong(&raw mut b0) {
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
                        b0_read_number(&raw mut b0.b0_mtime as *mut ::core::ffi::c_char)
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
                        b0_read_number(&raw mut b0.b0_ino as *mut ::core::ffi::c_char)
                            as varnumber_T,
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
                    if b0_read_number(&raw mut b0.b0_pid as *mut ::core::ffi::c_char)
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
                            b0_read_number(&raw mut b0.b0_pid as *mut ::core::ffi::c_char)
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
                    if b0_magic_wrong(&raw mut b0) {
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
        if ml_check_b0_id(&raw mut b0) as ::core::ffi::c_int == FAIL || b0_magic_wrong(&raw mut b0)
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
        if b0_read_number(&raw mut b0.b0_pid as *mut ::core::ffi::c_char)
            == 0 as ::core::ffi::c_long
            || swapfile_proc_running(&raw mut b0, fname) != 0
        {
            ret = false_0 != 0;
        }
        close(fd);
        return ret;
    }
}

/// Whether the four magic numbers say this swap file was written by a
/// different build: they are one value per width, so a change of
/// endianness or of a type's size shows up as a mismatch.
pub(crate) unsafe fn b0_magic_wrong(b0p: *mut ZeroBlock) -> bool {
    unsafe {
        (*b0p).b0_magic_long != B0_MAGIC_LONG as c_long
            || (*b0p).b0_magic_int != B0_MAGIC_INT as c_int
            // Truncated to 16 bits on the way in, so compare it truncated.
            || (*b0p).b0_magic_short != B0_MAGIC_SHORT as int16_t
            || (*b0p).b0_magic_char as c_int != B0_MAGIC_CHAR as c_int
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

/// Store a number in block zero: four bytes, little-endian.
///
/// Only the low 32 bits survive — the format has no room for more, and a
/// timestamp or inode wider than that is simply truncated, as upstream's
/// `(unsigned)n >> 8` stepping did.
pub(crate) unsafe fn b0_store_number(n: c_long, dest: *mut c_char) {
    let bytes = (n as u32).to_le_bytes();
    // SAFETY: every field written through here is four bytes wide, and the
    // block is a byte array, so alignment is not in question.
    unsafe { core::ptr::copy_nonoverlapping(bytes.as_ptr().cast::<c_char>(), dest, 4) };
}

/// Read back a number stored by [`b0_store_number`]. The result is the
/// unsigned 32-bit value widened, never negative.
pub(crate) unsafe fn b0_read_number(src: *const c_char) -> c_long {
    let mut bytes = [0u8; 4];
    // SAFETY: as [`b0_store_number`].
    unsafe { core::ptr::copy_nonoverlapping(src.cast::<u8>(), bytes.as_mut_ptr(), 4) };
    u32::from_le_bytes(bytes) as c_long
}

/// Update the flags block zero carries about the buffer — whether it has
/// unsaved changes, its `'fileformat'` and its `'fileencoding'` — and push
/// block zero alone to disk.
pub unsafe fn ml_setflags(buf: *mut buf_T) {
    unsafe {
        let mfp = (*buf).b_ml.ml_mfp;
        if mfp.is_null() {
            return;
        }
        // Only if block zero is already in memory; there is nothing here
        // worth a read for.
        let hp = mf_find(mfp, 0);
        if hp.is_null() {
            return;
        }

        let b0p = (*hp).bh_data as *mut ZeroBlock;
        (*b0p).set_dirty((*buf).b_changed != 0);
        let fileformat = (get_fileformat(buf) + 1) as uint8_t as c_int;
        (*b0p).set_flags((*b0p).flags() & !B0_FF_MASK | fileformat);
        add_b0_fenc(b0p, buf);
        (*hp).bh_flags |= BH_DIRTY;
        mf_sync(mfp, MFS_ZERO as c_int);
    }
}
