use crate::src::nvim::autocmd::{
    EVENT_BUFWRITECMD, EVENT_BUFWRITEPOST, EVENT_BUFWRITEPRE, EVENT_FILEAPPENDCMD,
    EVENT_FILEAPPENDPOST, EVENT_FILEAPPENDPRE, EVENT_FILEWRITECMD, EVENT_FILEWRITEPOST,
    EVENT_FILEWRITEPRE, EVENT_FILTERWRITEPOST, EVENT_FILTERWRITEPRE, apply_autocmds_exarg,
    aucmd_prepbuf, aucmd_restbuf,
};
use crate::src::nvim::buffer::buf_get_changedtick;
use crate::src::nvim::buffer::{bt_nofilename, buf_set_file_id, bufref_valid, set_bufref};
use crate::src::nvim::change::unchanged;
use crate::src::nvim::drawscreen::status_redraw_all;
use crate::src::nvim::eval::vars::eval_charconvert;
use crate::src::nvim::event::libuv::uv_strerror;
use crate::src::nvim::ex_cmds::check_secure;
use crate::src::nvim::ex_eval::{aborting, should_abort};
use crate::src::nvim::fileio::{
    add_quoted_fname, buf_store_file_info, filemess, get_fio_flags, match_file_list, modname,
    msg_add_fileformat, msg_add_lines, need_conversion, set_rw_fname, time_differs, vim_rename,
    vim_tempname, write_eintr,
};
use crate::src::nvim::global_cell::GlobalCell;
use crate::src::nvim::input::ask_yesno;
use crate::src::nvim::main::{
    IObuff, cmdmod, curbuf, e_empty_buffer, e_fsync, e_interr, e_longname, ex_no_reprint, exiting,
    got_int, msg_scroll, msg_silent, need_maketitle, no_wait_return, p_bdir, p_bex, p_bk, p_bsk,
    p_ccv, p_cpo, p_fs, p_pm, p_wb,
};
use crate::src::nvim::mbyte::{enc_canonize, my_iconv_open, utf_ptr2char, utf_ptr2len_len};
use crate::src::nvim::memline::{
    get_file_in_dir, make_percent_swname, ml_get_buf, ml_preserve, ml_timestamp,
};
use crate::src::nvim::memory::{verbose_try_malloc, xfree, xmalloc, xmemcpyz, xstrlcat};
use crate::src::nvim::message::{emsg, msg, msg_progress, msg_puts_hl, semsg, set_keep_msg};
use crate::src::nvim::option::{copy_option_part, get_bkc_flags, get_fileformat_force, shortmess};
use crate::src::nvim::options::{
    kOptBkcFlagAuto, kOptBkcFlagBreakhardlink, kOptBkcFlagBreaksymlink, kOptBkcFlagYes,
};
use crate::src::nvim::os::fs::{
    os_chown, os_close, os_copy, os_copy_xattr, os_fchown, os_file_is_writable, os_file_settime,
    os_fileinfo, os_fileinfo_hardlinks, os_fileinfo_id_equal, os_fileinfo_link, os_free_acl,
    os_fsync, os_get_acl, os_getperm, os_isdir, os_mkdir_recurse, os_nodetype, os_open,
    os_path_exists, os_remove, os_set_acl, os_setperm,
};
use crate::src::nvim::os::input::os_breakcheck;
use crate::src::nvim::os::libc::{
    __assert_fail, __errno_location, close, getgid, gettext, getuid, iconv, iconv_close, memmove,
    snprintf, strlen,
};
use crate::src::nvim::path::{after_pathsep, path_fnamecmp, path_tail};
use crate::src::nvim::sha256::Sha256;
use crate::src::nvim::strings::{vim_snprintf, vim_snprintf_add, vim_strchr};
use crate::src::nvim::types::{
    FileInfo, aco_save_T, buf_T, bufref_T, colnr_T, exarg_T, iconv_t, int32_t, int64_t, linenr_T,
    off_T, pos_T, size_t, uint8_t, uint64_t, uv_gid_t, uv_stat_t, uv_timespec_t, uv_uid_t,
    varnumber_T, vim_acl_T,
};
use crate::src::nvim::ui::ui_flush;
use crate::src::nvim::undo::{curbufIsChanged, u_unchanged, u_update_save_nr, u_write_undo};
pub type C2Rust_Unnamed = ::core::ffi::c_int;
pub const UV_ENOTSUP: C2Rust_Unnamed = -95;
pub type C2Rust_Unnamed_13 = ::core::ffi::c_uint;
pub const HLF_E: C2Rust_Unnamed_13 = 6;
pub type C2Rust_Unnamed_15 = ::core::ffi::c_uint;
pub const CMOD_LOCKMARKS: C2Rust_Unnamed_15 = 2048;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct Error_T {
    pub num: *const ::core::ffi::c_char,
    pub msg: *mut ::core::ffi::c_char,
    pub arg: ::core::ffi::c_int,
    pub alloc: bool,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct bw_info {
    pub bw_fd: ::core::ffi::c_int,
    pub bw_buf: *mut ::core::ffi::c_char,
    pub bw_len: ::core::ffi::c_int,
    pub bw_flags: ::core::ffi::c_int,
    pub bw_first: ::core::ffi::c_int,
    pub bw_conv_buf: *mut ::core::ffi::c_char,
    pub bw_conv_buflen: size_t,
    pub bw_conv_error: ::core::ffi::c_int,
    pub bw_conv_error_lnum: linenr_T,
    pub bw_start_lnum: linenr_T,
    pub bw_iconv_fd: iconv_t,
}
pub const WRITEBUFSIZE: C2Rust_Unnamed_17 = 8192;
pub const SHM_WRI: C2Rust_Unnamed_20 = 119;
pub const SHM_WRITE: C2Rust_Unnamed_20 = 87;
pub const FIO_LATIN1: C2Rust_Unnamed_16 = 1;
pub const FIO_ENDIAN_L: C2Rust_Unnamed_16 = 128;
pub const FIO_UTF16: C2Rust_Unnamed_16 = 16;
pub const FIO_UCS2: C2Rust_Unnamed_16 = 4;
pub const FIO_UCS4: C2Rust_Unnamed_16 = 8;
pub const FIO_NOCONVERT: C2Rust_Unnamed_16 = 8192;
pub const FIO_UTF8: C2Rust_Unnamed_16 = 2;
pub const ICONV_MULT: C2Rust_Unnamed_18 = 8;
pub const SHM_OVER: C2Rust_Unnamed_20 = 111;
pub type C2Rust_Unnamed_16 = ::core::ffi::c_int;
pub type C2Rust_Unnamed_17 = ::core::ffi::c_uint;
pub type C2Rust_Unnamed_18 = ::core::ffi::c_uint;
pub type C2Rust_Unnamed_20 = ::core::ffi::c_uint;
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const O_WRONLY: ::core::ffi::c_int = 0o1 as ::core::ffi::c_int;
pub const O_CREAT: ::core::ffi::c_int = 0o100 as ::core::ffi::c_int;
pub const O_EXCL: ::core::ffi::c_int = 0o200 as ::core::ffi::c_int;
pub const O_TRUNC: ::core::ffi::c_int = 0o1000 as ::core::ffi::c_int;
pub const O_APPEND: ::core::ffi::c_int = 0o2000 as ::core::ffi::c_int;
pub const __O_NOFOLLOW: ::core::ffi::c_int = 0o400000 as ::core::ffi::c_int;
pub const O_NOFOLLOW: ::core::ffi::c_int = __O_NOFOLLOW;
pub const UV_FS_COPYFILE_FICLONE: ::core::ffi::c_int = 0x2 as ::core::ffi::c_int;
pub const DEFAULT_MAXPATHL: ::core::ffi::c_int = 4096 as ::core::ffi::c_int;
pub const MAXPATHL: ::core::ffi::c_int = DEFAULT_MAXPATHL;
pub const NUL: ::core::ffi::c_int = '\0' as ::core::ffi::c_int;
pub const NL: ::core::ffi::c_int = '\n' as ::core::ffi::c_int;
pub const CAR: ::core::ffi::c_int = '\r' as ::core::ffi::c_int;
pub const BF_NOTEDITED: ::core::ffi::c_int = 0x8 as ::core::ffi::c_int;
pub const BF_NEW: ::core::ffi::c_int = 0x10 as ::core::ffi::c_int;
pub const BF_READERR: ::core::ffi::c_int = 0x40 as ::core::ffi::c_int;
pub const BF_WRITE_MASK: ::core::ffi::c_int = BF_NOTEDITED + BF_NEW + BF_READERR;
pub const ML_EMPTY: ::core::ffi::c_int = 0x1 as ::core::ffi::c_int;
pub const NODE_WRITABLE: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const OK: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const FAIL: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const NOTDONE: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
pub const __ASSERT_FUNCTION: [::core::ffi::c_char; 126] = unsafe {
    ::core::mem::transmute::<
        [u8; 126],
        [::core::ffi::c_char; 126],
    >(
        *b"int buf_write_make_backup(char *, _Bool, FileInfo *, vim_acl_T, int, unsigned int, _Bool, _Bool, _Bool *, char **, Error_T *)\0",
    )
};
static err_readonly: GlobalCell<*const ::core::ffi::c_char> = GlobalCell::new(
    b"is read-only (cannot override: \"W\" in 'cpoptions')\0".as_ptr()
        as *const ::core::ffi::c_char,
);
static e_patchmode_cant_touch_empty_original_file: GlobalCell<[::core::ffi::c_char; 49]> =
    GlobalCell::new(unsafe {
        ::core::mem::transmute::<[u8; 49], [::core::ffi::c_char; 49]>(
            *b"E206: Patchmode: can't touch empty original file\0",
        )
    });
static e_write_error_conversion_failed_make_fenc_empty_to_override: GlobalCell<
    [::core::ffi::c_char; 69],
> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 69], [::core::ffi::c_char; 69]>(
        *b"E513: Write error, conversion failed (make 'fenc' empty to override)\0",
    )
});
static e_write_error_conversion_failed_in_line_nr_make_fenc_empty_to_override: GlobalCell<
    [::core::ffi::c_char; 80],
> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 80], [::core::ffi::c_char; 80]>(
        *b"E513: Write error, conversion failed in line %d (make 'fenc' empty to override)\0",
    )
});
static e_write_error_file_system_full: GlobalCell<[::core::ffi::c_char; 38]> =
    GlobalCell::new(unsafe {
        ::core::mem::transmute::<[u8; 38], [::core::ffi::c_char; 38]>(
            *b"E514: Write error (file system full?)\0",
        )
    });
static e_no_matching_autocommands_for_buftype_str_buffer: GlobalCell<[::core::ffi::c_char; 53]> =
    GlobalCell::new(unsafe {
        ::core::mem::transmute::<[u8; 53], [::core::ffi::c_char; 53]>(
            *b"E676: No matching autocommands for buftype=%s buffer\0",
        )
    });
pub const SMALLBUFSIZE: ::core::ffi::c_int = 256 as ::core::ffi::c_int;
unsafe extern "C" fn ucs2bytes(
    mut c: ::core::ffi::c_uint,
    mut pp: *mut *mut ::core::ffi::c_char,
    mut flags: ::core::ffi::c_int,
) -> bool {
    let mut p: *mut uint8_t = *pp as *mut uint8_t;
    let mut error: bool = false_0 != 0;
    if flags & FIO_UCS4 as ::core::ffi::c_int != 0 {
        if flags & FIO_ENDIAN_L as ::core::ffi::c_int != 0 {
            let c2rust_fresh3 = p;
            p = p.offset(1);
            *c2rust_fresh3 = c as uint8_t;
            let c2rust_fresh4 = p;
            p = p.offset(1);
            *c2rust_fresh4 = (c >> 8 as ::core::ffi::c_int) as uint8_t;
            let c2rust_fresh5 = p;
            p = p.offset(1);
            *c2rust_fresh5 = (c >> 16 as ::core::ffi::c_int) as uint8_t;
            let c2rust_fresh6 = p;
            p = p.offset(1);
            *c2rust_fresh6 = (c >> 24 as ::core::ffi::c_int) as uint8_t;
        } else {
            let c2rust_fresh7 = p;
            p = p.offset(1);
            *c2rust_fresh7 = (c >> 24 as ::core::ffi::c_int) as uint8_t;
            let c2rust_fresh8 = p;
            p = p.offset(1);
            *c2rust_fresh8 = (c >> 16 as ::core::ffi::c_int) as uint8_t;
            let c2rust_fresh9 = p;
            p = p.offset(1);
            *c2rust_fresh9 = (c >> 8 as ::core::ffi::c_int) as uint8_t;
            let c2rust_fresh10 = p;
            p = p.offset(1);
            *c2rust_fresh10 = c as uint8_t;
        }
    } else if flags & (FIO_UCS2 as ::core::ffi::c_int | FIO_UTF16 as ::core::ffi::c_int) != 0 {
        if c >= 0x10000 as ::core::ffi::c_int as ::core::ffi::c_uint {
            if flags & FIO_UTF16 as ::core::ffi::c_int != 0 {
                c = c.wrapping_sub(0x10000 as ::core::ffi::c_int as ::core::ffi::c_uint);
                if c >= 0x100000 as ::core::ffi::c_int as ::core::ffi::c_uint {
                    error = true_0 != 0;
                }
                let mut cc: ::core::ffi::c_int = (c >> 10 as ::core::ffi::c_int
                    & 0x3ff as ::core::ffi::c_uint)
                    .wrapping_add(0xd800 as ::core::ffi::c_uint)
                    as ::core::ffi::c_int;
                if flags & FIO_ENDIAN_L as ::core::ffi::c_int != 0 {
                    let c2rust_fresh11 = p;
                    p = p.offset(1);
                    *c2rust_fresh11 = cc as uint8_t;
                    let c2rust_fresh12 = p;
                    p = p.offset(1);
                    *c2rust_fresh12 = (cc >> 8 as ::core::ffi::c_int) as uint8_t;
                } else {
                    let c2rust_fresh13 = p;
                    p = p.offset(1);
                    *c2rust_fresh13 = (cc >> 8 as ::core::ffi::c_int) as uint8_t;
                    let c2rust_fresh14 = p;
                    p = p.offset(1);
                    *c2rust_fresh14 = cc as uint8_t;
                }
                c = (c & 0x3ff as ::core::ffi::c_uint).wrapping_add(0xdc00 as ::core::ffi::c_uint);
            } else {
                error = true_0 != 0;
            }
        }
        if flags & FIO_ENDIAN_L as ::core::ffi::c_int != 0 {
            let c2rust_fresh15 = p;
            p = p.offset(1);
            *c2rust_fresh15 = c as uint8_t;
            let c2rust_fresh16 = p;
            p = p.offset(1);
            *c2rust_fresh16 = (c >> 8 as ::core::ffi::c_int) as uint8_t;
        } else {
            let c2rust_fresh17 = p;
            p = p.offset(1);
            *c2rust_fresh17 = (c >> 8 as ::core::ffi::c_int) as uint8_t;
            let c2rust_fresh18 = p;
            p = p.offset(1);
            *c2rust_fresh18 = c as uint8_t;
        }
    } else if c >= 0x100 as ::core::ffi::c_uint {
        error = true_0 != 0;
        let c2rust_fresh19 = p;
        p = p.offset(1);
        *c2rust_fresh19 = 0xbf as uint8_t;
    } else {
        let c2rust_fresh20 = p;
        p = p.offset(1);
        *c2rust_fresh20 = c as uint8_t;
    }
    *pp = p as *mut ::core::ffi::c_char;
    return error;
}
unsafe extern "C" fn buf_write_convert_with_iconv(
    mut ip: *mut bw_info,
    mut bufp: *mut *mut ::core::ffi::c_char,
    mut lenp: *mut ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let mut len: ::core::ffi::c_int = *lenp;
    let mut from: *const ::core::ffi::c_char = *bufp;
    let mut fromlen: size_t = len as size_t;
    let mut tolen: size_t = (*ip).bw_conv_buflen;
    let mut to: *mut ::core::ffi::c_char = (*ip).bw_conv_buf;
    if (*ip).bw_first != 0 {
        let mut save_len: size_t = tolen;
        iconv(
            (*ip).bw_iconv_fd,
            ::core::ptr::null_mut::<*mut ::core::ffi::c_char>(),
            ::core::ptr::null_mut::<size_t>(),
            &raw mut to,
            &raw mut tolen,
        );
        if to.is_null() {
            to = (*ip).bw_conv_buf;
            tolen = save_len;
        }
        (*ip).bw_first = false_0;
    }
    if iconv(
        (*ip).bw_iconv_fd,
        &raw mut from as *mut ::core::ffi::c_void as *mut *mut ::core::ffi::c_char,
        &raw mut fromlen,
        &raw mut to,
        &raw mut tolen,
    ) == -1 as ::core::ffi::c_int as size_t
        && *__errno_location() != ICONV_EINVAL
    {
        (*ip).bw_conv_error = true_0;
        return -1 as ::core::ffi::c_int;
    }
    *bufp = (*ip).bw_conv_buf;
    *lenp = to.offset_from((*ip).bw_conv_buf) as ::core::ffi::c_int;
    return len - fromlen as ::core::ffi::c_int;
}
unsafe extern "C" fn buf_write_convert(
    mut ip: *mut bw_info,
    mut bufp: *mut *mut ::core::ffi::c_char,
    mut lenp: *mut ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let mut flags: ::core::ffi::c_int = (*ip).bw_flags;
    let mut wlen: ::core::ffi::c_int = *lenp;
    if flags
        & (FIO_UCS4 as ::core::ffi::c_int
            | FIO_UTF16 as ::core::ffi::c_int
            | FIO_UCS2 as ::core::ffi::c_int
            | FIO_LATIN1 as ::core::ffi::c_int)
        != 0
    {
        let mut c: ::core::ffi::c_uint = 0;
        let mut n: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        let mut p: *mut ::core::ffi::c_char = if flags & FIO_LATIN1 as ::core::ffi::c_int != 0 {
            *bufp
        } else {
            (*ip).bw_conv_buf
        };
        wlen = 0 as ::core::ffi::c_int;
        while wlen < *lenp {
            n = utf_ptr2len_len((*bufp).offset(wlen as isize), *lenp - wlen);
            if n > *lenp - wlen {
                break;
            }
            c = if n > 1 as ::core::ffi::c_int {
                utf_ptr2char((*bufp).offset(wlen as isize)) as ::core::ffi::c_uint
            } else {
                *(*bufp).offset(wlen as isize) as uint8_t as ::core::ffi::c_uint
            };
            if flags & FIO_LATIN1 as ::core::ffi::c_int == 0 {
                let mut need: size_t = (if flags & FIO_UCS4 as ::core::ffi::c_int != 0 {
                    4 as ::core::ffi::c_int
                } else {
                    2 as ::core::ffi::c_int
                }) as size_t;
                if flags & FIO_UTF16 as ::core::ffi::c_int != 0
                    && c >= 0x10000 as ::core::ffi::c_int as ::core::ffi::c_uint
                {
                    need = 4 as size_t;
                }
                if (p.offset_from((*ip).bw_conv_buf) as size_t).wrapping_add(need)
                    > (*ip).bw_conv_buflen
                {
                    return FAIL;
                }
            }
            if ucs2bytes(c, &raw mut p, flags) as ::core::ffi::c_int != 0
                && (*ip).bw_conv_error == 0
            {
                (*ip).bw_conv_error = true_0;
                (*ip).bw_conv_error_lnum = (*ip).bw_start_lnum;
            }
            if c == NL as ::core::ffi::c_uint {
                (*ip).bw_start_lnum += 1;
            }
            wlen += n;
        }
        if flags & FIO_LATIN1 as ::core::ffi::c_int != 0 {
            *lenp = p.offset_from(*bufp) as ::core::ffi::c_int;
        } else {
            *bufp = (*ip).bw_conv_buf;
            *lenp = p.offset_from((*ip).bw_conv_buf) as ::core::ffi::c_int;
        }
    }
    if (*ip).bw_iconv_fd
        != ::core::ptr::with_exposed_provenance_mut::<::core::ffi::c_void>(
            -1 as ::core::ffi::c_int as usize,
        )
    {
        return buf_write_convert_with_iconv(ip, bufp, lenp);
    }
    return wlen;
}
unsafe extern "C" fn buf_write_bytes(mut ip: *mut bw_info) -> ::core::ffi::c_int {
    let mut buf: *mut ::core::ffi::c_char = (*ip).bw_buf;
    let mut len: ::core::ffi::c_int = (*ip).bw_len;
    let mut flags: ::core::ffi::c_int = (*ip).bw_flags;
    let mut converted: ::core::ffi::c_int = len;
    let mut remaining: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    if flags & FIO_NOCONVERT as ::core::ffi::c_int == 0 {
        converted = buf_write_convert(ip, &raw mut buf, &raw mut len);
        if converted < 0 as ::core::ffi::c_int {
            return FAIL;
        }
        remaining = (*ip).bw_len - converted;
    }
    (*ip).bw_len = remaining;
    if (*ip).bw_fd >= 0 as ::core::ffi::c_int {
        let mut wlen: ::core::ffi::c_int =
            write_eintr((*ip).bw_fd, buf as *mut ::core::ffi::c_void, len as size_t)
                as ::core::ffi::c_int;
        if wlen < len {
            return FAIL;
        }
    }
    if remaining > 0 as ::core::ffi::c_int {
        memmove(
            (*ip).bw_buf as *mut ::core::ffi::c_void,
            (*ip).bw_buf.offset(converted as isize) as *const ::core::ffi::c_void,
            remaining as size_t,
        );
    }
    return OK;
}
unsafe extern "C" fn check_mtime(
    mut buf: *mut buf_T,
    mut file_info: *mut FileInfo,
) -> ::core::ffi::c_int {
    if (*buf).b_mtime_read != 0 as int64_t
        && time_differs(file_info, (*buf).b_mtime_read, (*buf).b_mtime_read_ns)
            as ::core::ffi::c_int
            != 0
    {
        msg_scroll.set(true_0);
        msg_silent.set(0 as ::core::ffi::c_int);
        msg(
            gettext(
                b"WARNING: The file has been changed since reading it!!!\0".as_ptr()
                    as *const ::core::ffi::c_char,
            ),
            HLF_E as ::core::ffi::c_int,
        );
        if ask_yesno(gettext(
            b"Do you really want to write to it\0".as_ptr() as *const ::core::ffi::c_char
        )) == 'n' as ::core::ffi::c_int
        {
            return FAIL;
        }
        msg_scroll.set(false_0);
    }
    return OK;
}
unsafe extern "C" fn make_bom(
    mut buf_in: *mut ::core::ffi::c_char,
    mut name: *mut ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    let mut buf: *mut uint8_t = buf_in as *mut uint8_t;
    let mut flags: ::core::ffi::c_int = get_fio_flags(name);
    if flags == FIO_LATIN1 as ::core::ffi::c_int || flags == 0 as ::core::ffi::c_int {
        return 0 as ::core::ffi::c_int;
    }
    if flags == FIO_UTF8 as ::core::ffi::c_int {
        *buf.offset(0 as ::core::ffi::c_int as isize) = 0xef as uint8_t;
        *buf.offset(1 as ::core::ffi::c_int as isize) = 0xbb as uint8_t;
        *buf.offset(2 as ::core::ffi::c_int as isize) = 0xbf as uint8_t;
        return 3 as ::core::ffi::c_int;
    }
    let mut p: *mut ::core::ffi::c_char = buf as *mut ::core::ffi::c_char;
    ucs2bytes(0xfeff as ::core::ffi::c_uint, &raw mut p, flags);
    return (p as *mut uint8_t).offset_from(buf) as ::core::ffi::c_int;
}
unsafe extern "C" fn buf_write_do_autocmds(
    mut buf: *mut buf_T,
    mut fnamep: *mut *mut ::core::ffi::c_char,
    mut sfnamep: *mut *mut ::core::ffi::c_char,
    mut ffnamep: *mut *mut ::core::ffi::c_char,
    mut start: linenr_T,
    mut endp: *mut linenr_T,
    mut eap: *mut exarg_T,
    mut append: bool,
    mut filtering: bool,
    mut reset_changed: bool,
    mut overwriting: bool,
    mut whole: bool,
    orig_start: pos_T,
    orig_end: pos_T,
) -> ::core::ffi::c_int {
    let mut old_line_count: linenr_T = (*buf).b_ml.ml_line_count;
    let mut msg_save: ::core::ffi::c_int = msg_scroll.get();
    let mut aco: aco_save_T = aco_save_T {
        use_aucmd_win_idx: 0,
        save_curwin_handle: 0,
        new_curwin_handle: 0,
        save_prevwin_handle: 0,
        new_curbuf: bufref_T {
            br_buf: ::core::ptr::null_mut::<buf_T>(),
            br_fnum: 0,
            br_buf_free_count: 0,
        },
        tp_localdir: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        globaldir: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        save_VIsual_active: false,
        save_prompt_insert: 0,
    };
    let mut did_cmd: bool = false_0 != 0;
    let mut nofile_err: bool = false_0 != 0;
    let mut empty_memline: bool = (*buf).b_ml.ml_mfp.is_null();
    let mut bufref: bufref_T = bufref_T {
        br_buf: ::core::ptr::null_mut::<buf_T>(),
        br_fnum: 0,
        br_buf_free_count: 0,
    };
    let mut sfname: *mut ::core::ffi::c_char = *sfnamep;
    let mut buf_ffname: bool = *ffnamep == (*buf).b_ffname;
    let mut buf_sfname: bool = sfname == (*buf).b_sfname;
    let mut buf_fname_f: bool = *fnamep == (*buf).b_ffname;
    let mut buf_fname_s: bool = *fnamep == (*buf).b_sfname;
    aucmd_prepbuf(&raw mut aco, buf);
    set_bufref(&raw mut bufref, buf);
    if append {
        did_cmd = apply_autocmds_exarg(
            EVENT_FILEAPPENDCMD,
            sfname,
            sfname,
            false_0 != 0,
            curbuf.get(),
            eap,
        );
        if !did_cmd {
            if overwriting as ::core::ffi::c_int != 0
                && bt_nofilename(curbuf.get()) as ::core::ffi::c_int != 0
            {
                nofile_err = true_0 != 0;
            } else {
                apply_autocmds_exarg(
                    EVENT_FILEAPPENDPRE,
                    sfname,
                    sfname,
                    false_0 != 0,
                    curbuf.get(),
                    eap,
                );
            }
        }
    } else if filtering {
        apply_autocmds_exarg(
            EVENT_FILTERWRITEPRE,
            ::core::ptr::null_mut::<::core::ffi::c_char>(),
            sfname,
            false_0 != 0,
            curbuf.get(),
            eap,
        );
    } else if reset_changed as ::core::ffi::c_int != 0 && whole as ::core::ffi::c_int != 0 {
        let mut was_changed: bool = curbufIsChanged();
        did_cmd = apply_autocmds_exarg(
            EVENT_BUFWRITECMD,
            sfname,
            sfname,
            false_0 != 0,
            curbuf.get(),
            eap,
        );
        if did_cmd {
            if was_changed as ::core::ffi::c_int != 0 && !curbufIsChanged() {
                u_unchanged(curbuf.get());
                u_update_save_nr(curbuf.get());
            }
        } else if overwriting as ::core::ffi::c_int != 0
            && bt_nofilename(curbuf.get()) as ::core::ffi::c_int != 0
        {
            nofile_err = true_0 != 0;
        } else {
            apply_autocmds_exarg(
                EVENT_BUFWRITEPRE,
                sfname,
                sfname,
                false_0 != 0,
                curbuf.get(),
                eap,
            );
        }
    } else {
        did_cmd = apply_autocmds_exarg(
            EVENT_FILEWRITECMD,
            sfname,
            sfname,
            false_0 != 0,
            curbuf.get(),
            eap,
        );
        if !did_cmd {
            if overwriting as ::core::ffi::c_int != 0
                && bt_nofilename(curbuf.get()) as ::core::ffi::c_int != 0
            {
                nofile_err = true_0 != 0;
            } else {
                apply_autocmds_exarg(
                    EVENT_FILEWRITEPRE,
                    sfname,
                    sfname,
                    false_0 != 0,
                    curbuf.get(),
                    eap,
                );
            }
        }
    }
    aucmd_restbuf(&raw mut aco);
    if !bufref_valid(&raw mut bufref) {
        buf = ::core::ptr::null_mut::<buf_T>();
    }
    if buf.is_null()
        || (*buf).b_ml.ml_mfp.is_null() && !empty_memline
        || did_cmd as ::core::ffi::c_int != 0
        || nofile_err as ::core::ffi::c_int != 0
        || aborting() as ::core::ffi::c_int != 0
    {
        if !buf.is_null() && (*cmdmod.ptr()).cmod_flags & CMOD_LOCKMARKS as ::core::ffi::c_int != 0
        {
            (*buf).b_op_start = orig_start;
            (*buf).b_op_end = orig_end;
        }
        (*no_wait_return.ptr()) -= 1;
        msg_scroll.set(msg_save);
        if nofile_err {
            semsg(
                gettext(
                    (e_no_matching_autocommands_for_buftype_str_buffer.ptr() as *const _)
                        as *const ::core::ffi::c_char,
                ),
                (*curbuf.get()).b_p_bt,
            );
        }
        if nofile_err as ::core::ffi::c_int != 0 || aborting() as ::core::ffi::c_int != 0 {
            return FAIL;
        }
        if did_cmd {
            if buf.is_null() {
                return OK;
            }
            if overwriting {
                ml_timestamp(buf);
                if append {
                    (*buf).b_flags &= !BF_NEW;
                } else {
                    (*buf).b_flags &= !BF_WRITE_MASK;
                }
            }
            if reset_changed as ::core::ffi::c_int != 0
                && (*buf).b_changed != 0
                && !append
                && (overwriting as ::core::ffi::c_int != 0
                    || !vim_strchr(p_cpo.get(), CPO_PLUS).is_null())
            {
                return FAIL;
            }
            return OK;
        }
        if !aborting() {
            emsg(gettext(
                b"E203: Autocommands deleted or unloaded buffer to be written\0".as_ptr()
                    as *const ::core::ffi::c_char,
            ));
        }
        return FAIL;
    }
    if (*buf).b_ml.ml_line_count != old_line_count {
        if whole {
            *endp = (*buf).b_ml.ml_line_count;
        } else if (*buf).b_ml.ml_line_count > old_line_count {
            *endp += (*buf).b_ml.ml_line_count - old_line_count;
        } else {
            *endp -= old_line_count - (*buf).b_ml.ml_line_count;
            if *endp < start {
                (*no_wait_return.ptr()) -= 1;
                msg_scroll.set(msg_save);
                emsg(gettext(
                    b"E204: Autocommand changed number of lines in unexpected way\0".as_ptr()
                        as *const ::core::ffi::c_char,
                ));
                return FAIL;
            }
        }
    }
    if buf_ffname {
        *ffnamep = (*buf).b_ffname;
    }
    if buf_sfname {
        *sfnamep = (*buf).b_sfname;
    }
    if buf_fname_f {
        *fnamep = (*buf).b_ffname;
    }
    if buf_fname_s {
        *fnamep = (*buf).b_sfname;
    }
    return NOTDONE;
}
unsafe extern "C" fn buf_write_do_post_autocmds(
    mut buf: *mut buf_T,
    mut fname: *mut ::core::ffi::c_char,
    mut eap: *mut exarg_T,
    mut append: bool,
    mut filtering: bool,
    mut reset_changed: bool,
    mut whole: bool,
) {
    let mut aco: aco_save_T = aco_save_T {
        use_aucmd_win_idx: 0,
        save_curwin_handle: 0,
        new_curwin_handle: 0,
        save_prevwin_handle: 0,
        new_curbuf: bufref_T {
            br_buf: ::core::ptr::null_mut::<buf_T>(),
            br_fnum: 0,
            br_buf_free_count: 0,
        },
        tp_localdir: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        globaldir: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        save_VIsual_active: false,
        save_prompt_insert: 0,
    };
    (*curbuf.get()).b_no_eol_lnum = 0 as ::core::ffi::c_int as linenr_T;
    aucmd_prepbuf(&raw mut aco, buf);
    if append {
        apply_autocmds_exarg(
            EVENT_FILEAPPENDPOST,
            fname,
            fname,
            false_0 != 0,
            curbuf.get(),
            eap,
        );
    } else if filtering {
        apply_autocmds_exarg(
            EVENT_FILTERWRITEPOST,
            ::core::ptr::null_mut::<::core::ffi::c_char>(),
            fname,
            false_0 != 0,
            curbuf.get(),
            eap,
        );
    } else if reset_changed as ::core::ffi::c_int != 0 && whole as ::core::ffi::c_int != 0 {
        apply_autocmds_exarg(
            EVENT_BUFWRITEPOST,
            fname,
            fname,
            false_0 != 0,
            curbuf.get(),
            eap,
        );
    } else {
        apply_autocmds_exarg(
            EVENT_FILEWRITEPOST,
            fname,
            fname,
            false_0 != 0,
            curbuf.get(),
            eap,
        );
    }
    aucmd_restbuf(&raw mut aco);
}
#[inline]
unsafe extern "C" fn set_err_num(
    mut num: *const ::core::ffi::c_char,
    mut msg_0: *const ::core::ffi::c_char,
) -> Error_T {
    return Error_T {
        num: num,
        msg: msg_0 as *mut ::core::ffi::c_char,
        arg: 0 as ::core::ffi::c_int,
        alloc: false,
    };
}
#[inline]
unsafe extern "C" fn set_err(mut msg_0: *const ::core::ffi::c_char) -> Error_T {
    return Error_T {
        num: ::core::ptr::null::<::core::ffi::c_char>(),
        msg: msg_0 as *mut ::core::ffi::c_char,
        arg: 0 as ::core::ffi::c_int,
        alloc: false,
    };
}
#[inline]
unsafe extern "C" fn set_err_arg(
    mut msg_0: *const ::core::ffi::c_char,
    mut arg: ::core::ffi::c_int,
) -> Error_T {
    return Error_T {
        num: ::core::ptr::null::<::core::ffi::c_char>(),
        msg: msg_0 as *mut ::core::ffi::c_char,
        arg: arg,
        alloc: false,
    };
}
unsafe extern "C" fn emit_err(mut e: *mut Error_T) {
    if !(*e).num.is_null() {
        if (*e).arg != 0 as ::core::ffi::c_int {
            semsg(
                b"%s: %s%s: %s\0".as_ptr() as *const ::core::ffi::c_char,
                (*e).num,
                IObuff.ptr() as *mut ::core::ffi::c_char,
                (*e).msg,
                uv_strerror((*e).arg),
            );
        } else {
            semsg(
                b"%s: %s%s\0".as_ptr() as *const ::core::ffi::c_char,
                (*e).num,
                IObuff.ptr() as *mut ::core::ffi::c_char,
                (*e).msg,
            );
        }
    } else if (*e).arg != 0 as ::core::ffi::c_int {
        semsg((*e).msg, uv_strerror((*e).arg));
    } else {
        emsg((*e).msg);
    }
    if (*e).alloc {
        xfree((*e).msg as *mut ::core::ffi::c_void);
    }
}
unsafe extern "C" fn get_fileinfo_os(
    mut fname: *mut ::core::ffi::c_char,
    mut file_info_old: *mut FileInfo,
    mut _overwriting: bool,
    mut perm: *mut ::core::ffi::c_int,
    mut device: *mut bool,
    mut newfile: *mut bool,
    mut err: *mut Error_T,
) -> ::core::ffi::c_int {
    *perm = -1 as ::core::ffi::c_int;
    if !os_fileinfo(fname, file_info_old) {
        *newfile = true_0 != 0;
    } else {
        *perm = (*file_info_old).stat.st_mode as ::core::ffi::c_int;
        if !((*file_info_old).stat.st_mode & __S_IFMT as uint64_t == 0o100000 as uint64_t) {
            if (*file_info_old).stat.st_mode & __S_IFMT as uint64_t == 0o40000 as uint64_t {
                *err = set_err_num(
                    b"E502\0".as_ptr() as *const ::core::ffi::c_char,
                    gettext(b"is a directory\0".as_ptr() as *const ::core::ffi::c_char),
                );
                return FAIL;
            }
            if os_nodetype(fname) != NODE_WRITABLE {
                *err = set_err_num(
                    b"E503\0".as_ptr() as *const ::core::ffi::c_char,
                    gettext(b"is not a file or writable device\0".as_ptr()
                        as *const ::core::ffi::c_char),
                );
                return FAIL;
            }
            *device = true_0 != 0;
            *newfile = true_0 != 0;
            *perm = -1 as ::core::ffi::c_int;
        }
    }
    return OK;
}
unsafe extern "C" fn get_fileinfo(
    mut buf: *mut buf_T,
    mut fname: *mut ::core::ffi::c_char,
    mut overwriting: bool,
    mut forceit: bool,
    mut file_info_old: *mut FileInfo,
    mut perm: *mut ::core::ffi::c_int,
    mut device: *mut bool,
    mut newfile: *mut bool,
    mut readonly: *mut bool,
    mut err: *mut Error_T,
) -> ::core::ffi::c_int {
    if get_fileinfo_os(
        fname,
        file_info_old,
        overwriting,
        perm,
        device,
        newfile,
        err,
    ) == FAIL
    {
        return FAIL;
    }
    *readonly = false_0 != 0;
    if !*device && !*newfile {
        *readonly = os_file_is_writable(fname) == 0;
        if !forceit && *readonly as ::core::ffi::c_int != 0 {
            if !vim_strchr(p_cpo.get(), CPO_FWRITE).is_null() {
                *err = set_err_num(
                    b"E504\0".as_ptr() as *const ::core::ffi::c_char,
                    gettext(err_readonly.get()),
                );
            } else {
                *err = set_err_num(
                    b"E505\0".as_ptr() as *const ::core::ffi::c_char,
                    gettext(b"is read-only (add ! to override)\0".as_ptr()
                        as *const ::core::ffi::c_char),
                );
            }
            return FAIL;
        }
        if overwriting as ::core::ffi::c_int != 0 && !forceit {
            let mut retval: ::core::ffi::c_int = check_mtime(buf, file_info_old);
            if retval == FAIL {
                return FAIL;
            }
        }
    }
    return OK;
}
pub unsafe extern "C" fn buf_get_backup_name(
    mut fname: *mut ::core::ffi::c_char,
    mut dirp: *mut *mut ::core::ffi::c_char,
    mut no_prepend_dot: bool,
    mut backup_ext: *mut ::core::ffi::c_char,
) -> *mut ::core::ffi::c_char {
    let mut backup: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut dir_len: size_t = copy_option_part(
        dirp,
        IObuff.ptr() as *mut ::core::ffi::c_char,
        IOSIZE as size_t,
        b",\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
    );
    let mut p: *mut ::core::ffi::c_char =
        (IObuff.ptr() as *mut ::core::ffi::c_char).offset(dir_len as isize);
    if **dirp as ::core::ffi::c_int == NUL && !os_isdir(IObuff.ptr() as *mut ::core::ffi::c_char) {
        let mut ret: ::core::ffi::c_int = 0;
        let mut failed_dir: *mut ::core::ffi::c_char =
            ::core::ptr::null_mut::<::core::ffi::c_char>();
        ret = os_mkdir_recurse(
            IObuff.ptr() as *mut ::core::ffi::c_char,
            0o755 as int32_t,
            &raw mut failed_dir,
            ::core::ptr::null_mut::<*mut ::core::ffi::c_char>(),
        );
        if ret != 0 as ::core::ffi::c_int {
            semsg(
                gettext(
                    b"E303: Unable to create directory \"%s\" for backup file: %s\0".as_ptr()
                        as *const ::core::ffi::c_char,
                ),
                failed_dir,
                uv_strerror(ret),
            );
            xfree(failed_dir as *mut ::core::ffi::c_void);
        }
    }
    if dir_len > 1 as size_t
        && after_pathsep(IObuff.ptr() as *mut ::core::ffi::c_char, p) != 0
        && *p.offset(-1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
            == *p.offset(-2 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
    {
        p = make_percent_swname(IObuff.ptr() as *mut ::core::ffi::c_char, p, fname);
        if !p.is_null() {
            backup = modname(p, backup_ext, no_prepend_dot);
            xfree(p as *mut ::core::ffi::c_void);
        }
    }
    if backup.is_null() {
        let mut rootname: *mut ::core::ffi::c_char =
            get_file_in_dir(fname, IObuff.ptr() as *mut ::core::ffi::c_char);
        if !rootname.is_null() {
            backup = modname(rootname, backup_ext, no_prepend_dot);
            xfree(rootname as *mut ::core::ffi::c_void);
        }
    }
    return backup;
}
unsafe extern "C" fn buf_write_make_backup(
    mut fname: *mut ::core::ffi::c_char,
    mut append: bool,
    mut file_info_old: *mut FileInfo,
    mut acl: vim_acl_T,
    mut perm: ::core::ffi::c_int,
    mut bkc: ::core::ffi::c_uint,
    mut file_readonly: bool,
    mut forceit: bool,
    mut backup_copyp: *mut bool,
    mut backupp: *mut *mut ::core::ffi::c_char,
    mut err: *mut Error_T,
) -> ::core::ffi::c_int {
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
    let no_prepend_dot: bool = false_0 != 0;
    if bkc & kOptBkcFlagYes as ::core::ffi::c_int as ::core::ffi::c_uint != 0
        || append as ::core::ffi::c_int != 0
    {
        *backup_copyp = true_0 != 0;
    } else if bkc & kOptBkcFlagAuto as ::core::ffi::c_int as ::core::ffi::c_uint != 0 {
        if os_fileinfo_hardlinks(file_info_old) > 1 as uint64_t
            || !os_fileinfo_link(fname, &raw mut file_info)
            || !os_fileinfo_id_equal(&raw mut file_info, file_info_old)
        {
            *backup_copyp = true_0 != 0;
        } else {
            let mut dirlen: size_t = path_tail(fname).offset_from(fname) as size_t;
            '_c2rust_label: {
                if dirlen < 4096 as size_t {
                } else {
                    __assert_fail(
                        b"dirlen < MAXPATHL\0".as_ptr() as *const ::core::ffi::c_char,
                        b"src/nvim/bufwrite.rs\0".as_ptr() as *const ::core::ffi::c_char,
                        743 as ::core::ffi::c_uint,
                        __ASSERT_FUNCTION.as_ptr(),
                    );
                }
            };
            let mut tmp_fname: [::core::ffi::c_char; 4096] = [0; 4096];
            xmemcpyz(
                &raw mut tmp_fname as *mut ::core::ffi::c_char as *mut ::core::ffi::c_void,
                fname as *const ::core::ffi::c_void,
                dirlen,
            );
            let mut i: ::core::ffi::c_int = 4913 as ::core::ffi::c_int;
            loop {
                snprintf(
                    (&raw mut tmp_fname as *mut ::core::ffi::c_char).offset(dirlen as isize),
                    ::core::mem::size_of::<[::core::ffi::c_char; 4096]>().wrapping_sub(dirlen),
                    b"%d\0".as_ptr() as *const ::core::ffi::c_char,
                    i,
                );
                if !os_fileinfo_link(
                    &raw mut tmp_fname as *mut ::core::ffi::c_char,
                    &raw mut file_info,
                ) {
                    break;
                }
                i += 123 as ::core::ffi::c_int;
            }
            let mut fd: ::core::ffi::c_int = os_open(
                &raw mut tmp_fname as *mut ::core::ffi::c_char,
                O_CREAT | O_WRONLY | O_EXCL | O_NOFOLLOW,
                perm,
            );
            if fd < 0 as ::core::ffi::c_int {
                *backup_copyp = true_0 != 0;
            } else {
                os_fchown(
                    fd,
                    (*file_info_old).stat.st_uid as uv_uid_t,
                    (*file_info_old).stat.st_gid as uv_gid_t,
                );
                if !os_fileinfo(
                    &raw mut tmp_fname as *mut ::core::ffi::c_char,
                    &raw mut file_info,
                ) || file_info.stat.st_uid != (*file_info_old).stat.st_uid
                    || file_info.stat.st_gid != (*file_info_old).stat.st_gid
                    || file_info.stat.st_mode as ::core::ffi::c_int != perm
                {
                    *backup_copyp = true_0 != 0;
                }
                close(fd);
                os_remove(&raw mut tmp_fname as *mut ::core::ffi::c_char);
            }
        }
    }
    if bkc & kOptBkcFlagBreaksymlink as ::core::ffi::c_int as ::core::ffi::c_uint != 0
        || bkc & kOptBkcFlagBreakhardlink as ::core::ffi::c_int as ::core::ffi::c_uint != 0
    {
        let mut file_info_link_ok: bool = os_fileinfo_link(fname, &raw mut file_info);
        if bkc & kOptBkcFlagBreaksymlink as ::core::ffi::c_int as ::core::ffi::c_uint != 0
            && file_info_link_ok as ::core::ffi::c_int != 0
            && !os_fileinfo_id_equal(&raw mut file_info, file_info_old)
        {
            *backup_copyp = false_0 != 0;
        }
        if bkc & kOptBkcFlagBreakhardlink as ::core::ffi::c_int as ::core::ffi::c_uint != 0
            && os_fileinfo_hardlinks(file_info_old) > 1 as uint64_t
            && (!file_info_link_ok
                || os_fileinfo_id_equal(&raw mut file_info, file_info_old) as ::core::ffi::c_int
                    != 0)
        {
            *backup_copyp = false_0 != 0;
        }
    }
    let mut backup_ext: *mut ::core::ffi::c_char = (if *p_bex.get() as ::core::ffi::c_int == NUL {
        b".bak\0".as_ptr() as *const ::core::ffi::c_char
    } else {
        p_bex.get() as *const ::core::ffi::c_char
    }) as *mut ::core::ffi::c_char;
    if *backup_copyp {
        let mut some_error: bool = false_0 != 0;
        let mut dirp: *mut ::core::ffi::c_char = p_bdir.get();
        while *dirp != 0 {
            *backupp = buf_get_backup_name(fname, &raw mut dirp, no_prepend_dot, backup_ext);
            if (*backupp).is_null() {
                some_error = true_0 != 0;
                break;
            } else {
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
                if os_fileinfo(*backupp, &raw mut file_info_new) {
                    if os_fileinfo_id_equal(&raw mut file_info_new, file_info_old) {
                        let mut ptr_: *mut *mut ::core::ffi::c_void =
                            backupp as *mut *mut ::core::ffi::c_void;
                        xfree(*ptr_);
                        *ptr_ = NULL;
                        let _ = *ptr_;
                    } else if p_bk.get() == 0 {
                        let mut wp: *mut ::core::ffi::c_char = (*backupp)
                            .offset(strlen(*backupp) as isize)
                            .offset(-(1 as ::core::ffi::c_int as isize))
                            .offset(-(strlen(backup_ext) as isize));
                        wp = if wp > *backupp { wp } else { *backupp };
                        *wp = 'z' as ::core::ffi::c_char;
                        while *wp as ::core::ffi::c_int > 'a' as ::core::ffi::c_int
                            && os_fileinfo(*backupp, &raw mut file_info_new) as ::core::ffi::c_int
                                != 0
                        {
                            *wp -= 1;
                        }
                        if *wp as ::core::ffi::c_int == 'a' as ::core::ffi::c_int {
                            let mut ptr__0: *mut *mut ::core::ffi::c_void =
                                backupp as *mut *mut ::core::ffi::c_void;
                            xfree(*ptr__0);
                            *ptr__0 = NULL;
                            let _ = *ptr__0;
                        }
                    }
                }
                if (*backupp).is_null() {
                    continue;
                }
                os_remove(*backupp);
                if os_copy(fname, *backupp, UV_FS_COPYFILE_FICLONE) != 0 as ::core::ffi::c_int {
                    *err = set_err(gettext(
                        b"E509: Cannot create backup file (add ! to override)\0".as_ptr()
                            as *const ::core::ffi::c_char,
                    ));
                    let mut ptr__1: *mut *mut ::core::ffi::c_void =
                        backupp as *mut *mut ::core::ffi::c_void;
                    xfree(*ptr__1);
                    *ptr__1 = NULL;
                    let _ = *ptr__1;
                    *backupp = ::core::ptr::null_mut::<::core::ffi::c_char>();
                } else {
                    os_setperm(*backupp, perm & 0o777 as ::core::ffi::c_int);
                    if file_info_new.stat.st_gid != (*file_info_old).stat.st_gid
                        && os_chown(
                            *backupp,
                            -1 as ::core::ffi::c_int as uv_uid_t,
                            (*file_info_old).stat.st_gid as uv_gid_t,
                        ) != 0 as ::core::ffi::c_int
                    {
                        os_setperm(
                            *backupp,
                            perm & 0o707 as ::core::ffi::c_int
                                | (perm & 0o7 as ::core::ffi::c_int) << 3 as ::core::ffi::c_int,
                        );
                    }
                    os_file_settime(
                        *backupp,
                        (*file_info_old).stat.st_atim.tv_sec as ::core::ffi::c_double,
                        (*file_info_old).stat.st_mtim.tv_sec as ::core::ffi::c_double,
                    );
                    os_set_acl(*backupp, acl);
                    os_copy_xattr(fname, *backupp);
                    *err = set_err(::core::ptr::null::<::core::ffi::c_char>());
                    break;
                }
            }
        }
        if (*backupp).is_null() && (*err).msg.is_null() {
            *err = set_err(gettext(
                b"E509: Cannot create backup file (add ! to override)\0".as_ptr()
                    as *const ::core::ffi::c_char,
            ));
        }
        if (some_error as ::core::ffi::c_int != 0 || !(*err).msg.is_null()) && !forceit {
            return FAIL;
        }
        *err = set_err(::core::ptr::null::<::core::ffi::c_char>());
    } else {
        if file_readonly as ::core::ffi::c_int != 0
            && !vim_strchr(p_cpo.get(), CPO_FWRITE).is_null()
        {
            *err = set_err_num(
                b"E504\0".as_ptr() as *const ::core::ffi::c_char,
                gettext(err_readonly.get()),
            );
            return FAIL;
        }
        let mut dirp_0: *mut ::core::ffi::c_char = p_bdir.get();
        while *dirp_0 != 0 {
            *backupp = buf_get_backup_name(fname, &raw mut dirp_0, no_prepend_dot, backup_ext);
            if !(*backupp).is_null() {
                if p_bk.get() == 0 && os_path_exists(*backupp) as ::core::ffi::c_int != 0 {
                    let mut p: *mut ::core::ffi::c_char = (*backupp)
                        .offset(strlen(*backupp) as isize)
                        .offset(-(1 as ::core::ffi::c_int as isize))
                        .offset(-(strlen(backup_ext) as isize));
                    p = if p > *backupp { p } else { *backupp };
                    *p = 'z' as ::core::ffi::c_char;
                    while *p as ::core::ffi::c_int > 'a' as ::core::ffi::c_int
                        && os_path_exists(*backupp) as ::core::ffi::c_int != 0
                    {
                        *p -= 1;
                    }
                    if *p as ::core::ffi::c_int == 'a' as ::core::ffi::c_int {
                        let mut ptr__2: *mut *mut ::core::ffi::c_void =
                            backupp as *mut *mut ::core::ffi::c_void;
                        xfree(*ptr__2);
                        *ptr__2 = NULL;
                        let _ = *ptr__2;
                    }
                }
            }
            if (*backupp).is_null() {
                continue;
            }
            if vim_rename(fname, *backupp) == 0 as ::core::ffi::c_int {
                break;
            }
            let mut ptr__3: *mut *mut ::core::ffi::c_void =
                backupp as *mut *mut ::core::ffi::c_void;
            xfree(*ptr__3);
            *ptr__3 = NULL;
            let _ = *ptr__3;
        }
        if (*backupp).is_null() && !forceit {
            *err = set_err(gettext(
                b"E510: Can't make backup file (add ! to override)\0".as_ptr()
                    as *const ::core::ffi::c_char,
            ));
            return FAIL;
        }
    }
    return OK;
}
pub unsafe extern "C" fn buf_write(
    mut buf: *mut buf_T,
    mut fname: *mut ::core::ffi::c_char,
    mut sfname: *mut ::core::ffi::c_char,
    mut start: linenr_T,
    mut end: linenr_T,
    mut eap: *mut exarg_T,
    mut append: bool,
    mut forceit: bool,
    mut reset_changed: bool,
    mut filtering: bool,
) -> ::core::ffi::c_int {
    let mut fenc: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut converted: bool = false;
    let mut wb_flags: ::core::ffi::c_int = 0;
    let mut notconverted: bool = false;
    let mut no_eol: bool = false;
    let mut nchars: ::core::ffi::c_int = 0;
    let mut lnum: linenr_T = 0;
    let mut fileformat: ::core::ffi::c_int = 0;
    let mut checking_conversion: bool = false;
    let mut fd: ::core::ffi::c_int = 0;
    let mut fflags: ::core::ffi::c_int = 0;
    let mut mode: ::core::ffi::c_int = 0;
    let mut dobackup: bool = false;
    let mut backup_copy: bool = false;
    let mut made_writable: bool = false;
    let mut wfname: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut retval: ::core::ffi::c_int = OK;
    let mut msg_save: ::core::ffi::c_int = msg_scroll.get();
    let mut prev_got_int: bool = got_int.get();
    let mut whole: bool = start == 1 as linenr_T && end == (*buf).b_ml.ml_line_count;
    let mut write_undo_file: bool = false_0 != 0;
    let mut sha_ctx = Sha256::new();
    let mut bkc: ::core::ffi::c_uint = get_bkc_flags(buf);
    if fname.is_null() || *fname as ::core::ffi::c_int == NUL {
        return FAIL;
    }
    if (*buf).b_ml.ml_mfp.is_null() {
        emsg(gettext(
            &raw const e_empty_buffer as *const ::core::ffi::c_char,
        ));
        return FAIL;
    }
    if check_secure() {
        return FAIL;
    }
    if strlen(fname) >= MAXPATHL as size_t {
        emsg(gettext(&raw const e_longname as *const ::core::ffi::c_char));
        return FAIL;
    }
    let mut write_info: bw_info = bw_info {
        bw_fd: 0,
        bw_buf: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        bw_len: 0,
        bw_flags: 0,
        bw_first: 0,
        bw_conv_buf: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        bw_conv_buflen: 0,
        bw_conv_error: 0,
        bw_conv_error_lnum: 0,
        bw_start_lnum: 0,
        bw_iconv_fd: ::core::ptr::null_mut::<::core::ffi::c_void>(),
    };
    write_info.bw_conv_buf = ::core::ptr::null_mut::<::core::ffi::c_char>();
    write_info.bw_conv_error = false_0;
    write_info.bw_conv_error_lnum = 0 as ::core::ffi::c_int as linenr_T;
    write_info.bw_iconv_fd = ::core::ptr::with_exposed_provenance_mut::<::core::ffi::c_void>(
        -1 as ::core::ffi::c_int as usize,
    );
    ex_no_reprint.set(true_0 != 0);
    if (*buf).b_ffname.is_null()
        && reset_changed as ::core::ffi::c_int != 0
        && whole as ::core::ffi::c_int != 0
        && buf == curbuf.get()
        && !bt_nofilename(buf)
        && !filtering
        && (!append || !vim_strchr(p_cpo.get(), CPO_FNAMEAPP).is_null())
        && !vim_strchr(p_cpo.get(), CPO_FNAMEW).is_null()
    {
        if set_rw_fname(fname, sfname) == FAIL {
            return FAIL;
        }
        buf = curbuf.get();
    }
    if sfname.is_null() {
        sfname = fname;
    }
    let mut ffname: *mut ::core::ffi::c_char = fname;
    fname = sfname;
    let mut overwriting: bool = !(*buf).b_ffname.is_null()
        && path_fnamecmp(ffname, (*buf).b_ffname) == 0 as ::core::ffi::c_int;
    (*no_wait_return.ptr()) += 1;
    let orig_start: pos_T = (*buf).b_op_start;
    let orig_end: pos_T = (*buf).b_op_end;
    (*buf).b_op_start.lnum = start;
    (*buf).b_op_start.col = 0 as ::core::ffi::c_int as colnr_T;
    (*buf).b_op_end.lnum = end;
    (*buf).b_op_end.col = 0 as ::core::ffi::c_int as colnr_T;
    let mut res: ::core::ffi::c_int = buf_write_do_autocmds(
        buf,
        &raw mut fname,
        &raw mut sfname,
        &raw mut ffname,
        start,
        &raw mut end,
        eap,
        append,
        filtering,
        reset_changed,
        overwriting,
        whole,
        orig_start,
        orig_end,
    );
    if res != NOTDONE {
        return res;
    }
    if (*cmdmod.ptr()).cmod_flags & CMOD_LOCKMARKS as ::core::ffi::c_int != 0 {
        (*buf).b_op_start = orig_start;
        (*buf).b_op_end = orig_end;
    }
    if shortmess(SHM_OVER as ::core::ffi::c_int) as ::core::ffi::c_int != 0 && !exiting.get() {
        msg_scroll.set(false_0);
    } else {
        msg_scroll.set(true_0);
    }
    if !filtering {
        filemess(
            buf,
            fname,
            b"\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        );
    }
    msg_scroll.set(false_0);
    let mut buffer: *mut ::core::ffi::c_char =
        verbose_try_malloc(WRITEBUFSIZE as ::core::ffi::c_int as size_t)
            as *mut ::core::ffi::c_char;
    let mut bufsize: ::core::ffi::c_int = 0;
    let mut smallbuf: [::core::ffi::c_char; 256] = [0; 256];
    if buffer.is_null() {
        buffer = &raw mut smallbuf as *mut ::core::ffi::c_char;
        bufsize = SMALLBUFSIZE;
    } else {
        bufsize = WRITEBUFSIZE as ::core::ffi::c_int;
    }
    let mut err: Error_T = Error_T {
        num: ::core::ptr::null::<::core::ffi::c_char>(),
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        arg: 0,
        alloc: false,
    };
    let mut perm: ::core::ffi::c_int = 0;
    let mut newfile: bool = false_0 != 0;
    let mut device: bool = false_0 != 0;
    let mut file_readonly: bool = false_0 != 0;
    let mut backup: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut fenc_tofree: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut file_info_old: FileInfo = FileInfo {
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
    let mut acl: vim_acl_T = NULL;
    '_nofail: {
        '_fail: {
            if get_fileinfo(
                buf,
                fname,
                overwriting,
                forceit,
                &raw mut file_info_old,
                &raw mut perm,
                &raw mut device,
                &raw mut newfile,
                &raw mut file_readonly,
                &raw mut err,
            ) != FAIL
            {
                if !newfile {
                    acl = os_get_acl(fname);
                }
                dobackup =
                    p_wb.get() != 0 || p_bk.get() != 0 || *p_pm.get() as ::core::ffi::c_int != NUL;
                if dobackup as ::core::ffi::c_int != 0
                    && *p_bsk.get() as ::core::ffi::c_int != NUL
                    && match_file_list(p_bsk.get(), sfname, ffname) as ::core::ffi::c_int != 0
                {
                    dobackup = false_0 != 0;
                }
                backup_copy = false_0 != 0;
                prev_got_int = got_int.get();
                got_int.set(false_0 != 0);
                (*buf).b_saving = true_0 != 0;
                if !(append as ::core::ffi::c_int != 0 && *p_pm.get() as ::core::ffi::c_int == NUL)
                    && !filtering
                    && perm >= 0 as ::core::ffi::c_int
                    && dobackup as ::core::ffi::c_int != 0
                {
                    if buf_write_make_backup(
                        fname,
                        append,
                        &raw mut file_info_old,
                        acl,
                        perm,
                        bkc,
                        file_readonly,
                        forceit,
                        &raw mut backup_copy,
                        &raw mut backup,
                        &raw mut err,
                    ) == FAIL
                    {
                        retval = FAIL;
                        break '_fail;
                    }
                }
                made_writable = false_0 != 0;
                if forceit as ::core::ffi::c_int != 0
                    && perm >= 0 as ::core::ffi::c_int
                    && perm & 0o200 as ::core::ffi::c_int == 0
                    && file_info_old.stat.st_uid == getuid() as uint64_t
                    && vim_strchr(p_cpo.get(), CPO_FWRITE).is_null()
                {
                    perm |= 0o200 as ::core::ffi::c_int;
                    os_setperm(fname, perm);
                    made_writable = true_0 != 0;
                }
                if forceit as ::core::ffi::c_int != 0
                    && overwriting as ::core::ffi::c_int != 0
                    && vim_strchr(p_cpo.get(), CPO_KEEPRO).is_null()
                {
                    (*buf).b_p_ro = false_0;
                    need_maketitle.set(true_0 != 0);
                    status_redraw_all();
                }
                end = if end < (*buf).b_ml.ml_line_count {
                    end
                } else {
                    (*buf).b_ml.ml_line_count
                };
                if (*buf).b_ml.ml_flags & ML_EMPTY != 0 {
                    start = end + 1 as linenr_T;
                }
                wfname = ::core::ptr::null_mut::<::core::ffi::c_char>();
                '_restore_backup: {
                    if reset_changed as ::core::ffi::c_int != 0
                        && !newfile
                        && overwriting as ::core::ffi::c_int != 0
                        && !(exiting.get() as ::core::ffi::c_int != 0 && !backup.is_null())
                    {
                        ml_preserve(
                            buf,
                            false_0 != 0,
                            if (*buf).b_p_fs >= 0 as ::core::ffi::c_int {
                                (*buf).b_p_fs
                            } else {
                                p_fs.get()
                            } != 0,
                        );
                        if got_int.get() {
                            err =
                                set_err(gettext(&raw const e_interr as *const ::core::ffi::c_char));
                            break '_restore_backup;
                        }
                    }
                    wfname = fname;
                    fenc = ::core::ptr::null_mut::<::core::ffi::c_char>();
                    if !eap.is_null() && (*eap).force_enc != 0 as ::core::ffi::c_int {
                        fenc = (*eap).cmd.offset((*eap).force_enc as isize);
                        fenc = enc_canonize(fenc);
                        fenc_tofree = fenc;
                    } else {
                        fenc = (*buf).b_p_fenc;
                    }
                    converted = need_conversion(fenc);
                    wb_flags = 0 as ::core::ffi::c_int;
                    if converted {
                        wb_flags = get_fio_flags(fenc);
                        if wb_flags
                            & (FIO_UCS2 as ::core::ffi::c_int
                                | FIO_UCS4 as ::core::ffi::c_int
                                | FIO_UTF16 as ::core::ffi::c_int
                                | FIO_UTF8 as ::core::ffi::c_int)
                            != 0
                        {
                            if wb_flags
                                & (FIO_UCS2 as ::core::ffi::c_int
                                    | FIO_UTF16 as ::core::ffi::c_int
                                    | FIO_UTF8 as ::core::ffi::c_int)
                                != 0
                            {
                                write_info.bw_conv_buflen =
                                    (bufsize as size_t).wrapping_mul(2 as size_t);
                            } else {
                                write_info.bw_conv_buflen =
                                    (bufsize as size_t).wrapping_mul(4 as size_t);
                            }
                            write_info.bw_conv_buf = verbose_try_malloc(write_info.bw_conv_buflen)
                                as *mut ::core::ffi::c_char;
                            if write_info.bw_conv_buf.is_null() {
                                end = 0 as ::core::ffi::c_int as linenr_T;
                            }
                        }
                    }
                    if converted as ::core::ffi::c_int != 0 && wb_flags == 0 as ::core::ffi::c_int {
                        write_info.bw_iconv_fd = my_iconv_open(
                            fenc,
                            b"utf-8\0".as_ptr() as *const ::core::ffi::c_char
                                as *mut ::core::ffi::c_char,
                        );
                        if write_info.bw_iconv_fd
                            != ::core::ptr::with_exposed_provenance_mut::<::core::ffi::c_void>(
                                -1 as ::core::ffi::c_int as usize,
                            )
                        {
                            write_info.bw_conv_buflen = (bufsize as size_t)
                                .wrapping_mul(ICONV_MULT as ::core::ffi::c_int as size_t);
                            write_info.bw_conv_buf = verbose_try_malloc(write_info.bw_conv_buflen)
                                as *mut ::core::ffi::c_char;
                            if write_info.bw_conv_buf.is_null() {
                                end = 0 as ::core::ffi::c_int as linenr_T;
                            }
                            write_info.bw_first = true_0;
                        } else if *p_ccv.get() as ::core::ffi::c_int != NUL {
                            wfname = vim_tempname();
                            if wfname.is_null() {
                                err = set_err(gettext(
                                    b"E214: Can't find temp file for writing\0".as_ptr()
                                        as *const ::core::ffi::c_char,
                                ));
                                break '_restore_backup;
                            }
                        }
                    }
                    notconverted = false_0 != 0;
                    if converted as ::core::ffi::c_int != 0
                        && wb_flags == 0 as ::core::ffi::c_int
                        && write_info.bw_iconv_fd
                            == ::core::ptr::with_exposed_provenance_mut::<::core::ffi::c_void>(
                                -1 as ::core::ffi::c_int as usize,
                            )
                        && wfname == fname
                    {
                        if !forceit {
                            err = set_err(gettext(
                                b"E213: Cannot convert (add ! to write without conversion)\0"
                                    .as_ptr()
                                    as *const ::core::ffi::c_char,
                            ));
                            break '_restore_backup;
                        } else {
                            notconverted = true_0 != 0;
                        }
                    }
                    no_eol = false_0 != 0;
                    nchars = 0;
                    lnum = 0;
                    fileformat = 0;
                    checking_conversion = false;
                    fd = 0;
                    checking_conversion = true_0 != 0;
                    loop {
                        if !converted || dobackup as ::core::ffi::c_int != 0 {
                            checking_conversion = false_0 != 0;
                        }
                        's_777: {
                            if checking_conversion {
                                fd = -1 as ::core::ffi::c_int;
                                write_info.bw_fd = fd;
                            } else {
                                fflags = O_WRONLY
                                    | (if append as ::core::ffi::c_int != 0 {
                                        if forceit as ::core::ffi::c_int != 0 {
                                            O_APPEND | O_CREAT
                                        } else {
                                            O_APPEND
                                        }
                                    } else {
                                        O_CREAT | O_TRUNC
                                    });
                                mode = if perm < 0 as ::core::ffi::c_int {
                                    0o666 as ::core::ffi::c_int
                                } else {
                                    perm & 0o777 as ::core::ffi::c_int
                                };
                                loop {
                                    fd = os_open(wfname, fflags, mode);
                                    if fd < 0 as ::core::ffi::c_int {
                                        if !err.msg.is_null() {
                                            break '_restore_backup;
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
                                        if !newfile
                                            && os_fileinfo_hardlinks(&raw mut file_info_old)
                                                > 1 as uint64_t
                                            || os_fileinfo_link(fname, &raw mut file_info)
                                                as ::core::ffi::c_int
                                                != 0
                                                && !os_fileinfo_id_equal(
                                                    &raw mut file_info,
                                                    &raw mut file_info_old,
                                                )
                                        {
                                            err = set_err(gettext(
                                                b"E166: Can't open linked file for writing\0"
                                                    .as_ptr()
                                                    as *const ::core::ffi::c_char,
                                            ));
                                            break '_restore_backup;
                                        } else {
                                            err = set_err_arg(
                                                gettext(
                                                    b"E212: Can't open file for writing: %s\0"
                                                        .as_ptr()
                                                        as *const ::core::ffi::c_char,
                                                ),
                                                fd,
                                            );
                                            if !(forceit as ::core::ffi::c_int != 0
                                                && vim_strchr(p_cpo.get(), CPO_FWRITE).is_null()
                                                && perm >= 0 as ::core::ffi::c_int)
                                            {
                                                break '_restore_backup;
                                            }
                                            if perm & 0o200 as ::core::ffi::c_int == 0 {
                                                made_writable = true_0 != 0;
                                            }
                                            perm |= 0o200 as ::core::ffi::c_int;
                                            if file_info_old.stat.st_uid != getuid() as uint64_t
                                                || file_info_old.stat.st_gid != getgid() as uint64_t
                                            {
                                                perm &= 0o777 as ::core::ffi::c_int;
                                            }
                                            if !append {
                                                os_remove(wfname);
                                            }
                                        }
                                    } else {
                                        write_info.bw_fd = fd;
                                        break 's_777;
                                    }
                                }
                            }
                        }
                        err = set_err(::core::ptr::null::<::core::ffi::c_char>());
                        write_info.bw_buf = buffer;
                        nchars = 0 as ::core::ffi::c_int;
                        let mut write_bin: ::core::ffi::c_int = 0;
                        if !eap.is_null() && (*eap).force_bin != 0 as ::core::ffi::c_int {
                            write_bin = ((*eap).force_bin == FORCE_BIN) as ::core::ffi::c_int;
                        } else {
                            write_bin = (*buf).b_p_bin;
                        }
                        if (*buf).b_p_bomb != 0
                            && write_bin == 0
                            && (!append || perm < 0 as ::core::ffi::c_int)
                        {
                            write_info.bw_len = make_bom(buffer, fenc);
                            if write_info.bw_len > 0 as ::core::ffi::c_int {
                                write_info.bw_flags =
                                    FIO_NOCONVERT as ::core::ffi::c_int | wb_flags;
                                if buf_write_bytes(&raw mut write_info) == FAIL {
                                    end = 0 as ::core::ffi::c_int as linenr_T;
                                } else {
                                    nchars += write_info.bw_len;
                                }
                            }
                        }
                        write_info.bw_start_lnum = start;
                        write_undo_file = (*buf).b_p_udf != 0
                            && overwriting as ::core::ffi::c_int != 0
                            && !append
                            && !filtering
                            && reset_changed as ::core::ffi::c_int != 0
                            && !checking_conversion;
                        if write_undo_file {
                            sha_ctx = Sha256::new();
                        }
                        write_info.bw_len = 0 as ::core::ffi::c_int;
                        write_info.bw_flags = wb_flags;
                        fileformat = get_fileformat_force(buf, eap);
                        let mut s: *mut ::core::ffi::c_char = buffer;
                        lnum = start;
                        while lnum <= end {
                            let mut ptr: *mut ::core::ffi::c_char =
                                ml_get_buf(buf, lnum).offset(-(1 as ::core::ffi::c_int as isize));
                            if write_undo_file {
                                let line = ptr.offset(1 as ::core::ffi::c_int as isize);
                                // Include the terminating NUL as a line separator.
                                sha_ctx.update(::core::slice::from_raw_parts(
                                    line as *const u8,
                                    strlen(line) + 1,
                                ));
                            }
                            let mut c: ::core::ffi::c_char = 0;
                            loop {
                                ptr = ptr.offset(1);
                                c = *ptr;
                                if c as ::core::ffi::c_int == NUL {
                                    break;
                                }
                                if c as ::core::ffi::c_int == NL {
                                    *s = NUL as ::core::ffi::c_char;
                                } else if c as ::core::ffi::c_int == CAR && fileformat == EOL_MAC {
                                    *s = NL as ::core::ffi::c_char;
                                } else {
                                    *s = c;
                                }
                                s = s.offset(1);
                                write_info.bw_len += 1;
                                if write_info.bw_len != bufsize {
                                    continue;
                                }
                                if buf_write_bytes(&raw mut write_info) == FAIL {
                                    end = 0 as ::core::ffi::c_int as linenr_T;
                                    break;
                                } else {
                                    nchars += bufsize - write_info.bw_len;
                                    s = buffer.offset(write_info.bw_len as isize);
                                    write_info.bw_start_lnum = lnum;
                                }
                            }
                            if end == 0 as linenr_T
                                || lnum == end
                                    && (write_bin != 0 || (*buf).b_p_fixeol == 0)
                                    && (write_bin != 0 && lnum == (*buf).b_no_eol_lnum
                                        || lnum == (*buf).b_ml.ml_line_count && (*buf).b_p_eol == 0)
                            {
                                lnum += 1;
                                no_eol = true_0 != 0;
                                break;
                            } else {
                                if fileformat == EOL_UNIX {
                                    let c2rust_fresh0 = s;
                                    s = s.offset(1);
                                    *c2rust_fresh0 = NL as ::core::ffi::c_char;
                                } else {
                                    let c2rust_fresh1 = s;
                                    s = s.offset(1);
                                    *c2rust_fresh1 = CAR as ::core::ffi::c_char;
                                    if fileformat == EOL_DOS {
                                        write_info.bw_len += 1;
                                        if write_info.bw_len == bufsize {
                                            if buf_write_bytes(&raw mut write_info) == FAIL {
                                                end = 0 as ::core::ffi::c_int as linenr_T;
                                                break;
                                            } else {
                                                nchars += bufsize - write_info.bw_len;
                                                s = buffer.offset(write_info.bw_len as isize);
                                            }
                                        }
                                        let c2rust_fresh2 = s;
                                        s = s.offset(1);
                                        *c2rust_fresh2 = NL as ::core::ffi::c_char;
                                    }
                                }
                                write_info.bw_len += 1;
                                if write_info.bw_len == bufsize {
                                    if buf_write_bytes(&raw mut write_info) == FAIL {
                                        end = 0 as ::core::ffi::c_int as linenr_T;
                                        break;
                                    } else {
                                        nchars += bufsize - write_info.bw_len;
                                        s = buffer.offset(write_info.bw_len as isize);
                                        os_breakcheck();
                                        if got_int.get() {
                                            end = 0 as ::core::ffi::c_int as linenr_T;
                                            break;
                                        }
                                    }
                                }
                                lnum += 1;
                            }
                        }
                        if write_info.bw_len > 0 as ::core::ffi::c_int && end > 0 as linenr_T {
                            let mut remaining: ::core::ffi::c_int = write_info.bw_len;
                            if buf_write_bytes(&raw mut write_info) == FAIL {
                                end = 0 as ::core::ffi::c_int as linenr_T;
                            }
                            nchars += remaining - write_info.bw_len;
                        }
                        if end != 0 as linenr_T && write_info.bw_len > 0 as ::core::ffi::c_int {
                            write_info.bw_conv_error = true_0;
                            write_info.bw_conv_error_lnum = end;
                            end = 0 as ::core::ffi::c_int as linenr_T;
                        }
                        if (*buf).b_p_fixeol == 0 && (*buf).b_p_eof != 0 {
                            write_eintr(
                                write_info.bw_fd,
                                b"\x1A\0".as_ptr() as *const ::core::ffi::c_char
                                    as *mut ::core::ffi::c_void,
                                1 as size_t,
                            );
                        }
                        if !checking_conversion || end == 0 as linenr_T {
                            if !checking_conversion {
                                let mut error: ::core::ffi::c_int = 0;
                                if (if (*buf).b_p_fs >= 0 as ::core::ffi::c_int {
                                    (*buf).b_p_fs
                                } else {
                                    p_fs.get()
                                }) != 0
                                    && {
                                        error = os_fsync(fd);
                                        error != 0 as ::core::ffi::c_int
                                    }
                                    && error != UV_ENOTSUP as ::core::ffi::c_int
                                    && !device
                                {
                                    err = set_err_arg(
                                        &raw const e_fsync as *const ::core::ffi::c_char,
                                        error,
                                    );
                                    end = 0 as ::core::ffi::c_int as linenr_T;
                                }
                                if !backup_copy {
                                    os_copy_xattr(backup, wfname);
                                }
                                if !backup.is_null() && !backup_copy {
                                    let mut file_info_0: FileInfo = FileInfo {
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
                                    if !os_fileinfo(wfname, &raw mut file_info_0)
                                        || file_info_0.stat.st_uid != file_info_old.stat.st_uid
                                        || file_info_0.stat.st_gid != file_info_old.stat.st_gid
                                    {
                                        os_fchown(
                                            fd,
                                            file_info_old.stat.st_uid as uv_uid_t,
                                            file_info_old.stat.st_gid as uv_gid_t,
                                        );
                                        if perm >= 0 as ::core::ffi::c_int {
                                            os_setperm(wfname, perm);
                                        }
                                    }
                                    buf_set_file_id(buf);
                                } else if !(*buf).file_id_valid {
                                    buf_set_file_id(buf);
                                }
                                error = os_close(fd);
                                if error != 0 as ::core::ffi::c_int {
                                    err = set_err_arg(
                                        gettext(b"E512: Close failed: %s\0".as_ptr()
                                            as *const ::core::ffi::c_char),
                                        error,
                                    );
                                    end = 0 as ::core::ffi::c_int as linenr_T;
                                }
                                if made_writable {
                                    perm &= !(0o200 as ::core::ffi::c_int);
                                }
                                if perm >= 0 as ::core::ffi::c_int {
                                    os_setperm(wfname, perm);
                                }
                                if !backup_copy {
                                    os_set_acl(wfname, acl);
                                }
                                if wfname != fname {
                                    if end != 0 as linenr_T {
                                        if eval_charconvert(
                                            b"utf-8\0".as_ptr() as *const ::core::ffi::c_char,
                                            fenc,
                                            wfname,
                                            fname,
                                        ) == FAIL
                                        {
                                            write_info.bw_conv_error = true_0;
                                            end = 0 as ::core::ffi::c_int as linenr_T;
                                        }
                                    }
                                    os_remove(wfname);
                                    xfree(wfname as *mut ::core::ffi::c_void);
                                }
                            }
                            if end == 0 as linenr_T {
                                if err.msg.is_null() {
                                    if write_info.bw_conv_error != 0 {
                                        if write_info.bw_conv_error_lnum == 0 as linenr_T {
                                            err = set_err(
                                                gettext(
                                                    (e_write_error_conversion_failed_make_fenc_empty_to_override.ptr() as *const _)
                                                        as *const ::core::ffi::c_char,
                                                ),
                                            );
                                        } else {
                                            err = set_err(xmalloc(300 as size_t)
                                                as *const ::core::ffi::c_char);
                                            err.alloc = true_0 != 0;
                                            vim_snprintf(
                                                err.msg,
                                                300 as size_t,
                                                gettext(
                                                    (e_write_error_conversion_failed_in_line_nr_make_fenc_empty_to_override.ptr() as *const _)
                                                        as *const ::core::ffi::c_char,
                                                ),
                                                write_info.bw_conv_error_lnum,
                                            );
                                        }
                                    } else if got_int.get() {
                                        err = set_err(gettext(
                                            &raw const e_interr as *const ::core::ffi::c_char,
                                        ));
                                    } else {
                                        err = set_err(gettext(
                                            (e_write_error_file_system_full.ptr() as *const _)
                                                as *const ::core::ffi::c_char,
                                        ));
                                    }
                                }
                                if !backup.is_null() {
                                    if backup_copy {
                                        if got_int.get() {
                                            msg(
                                                gettext(
                                                    &raw const e_interr
                                                        as *const ::core::ffi::c_char,
                                                ),
                                                0 as ::core::ffi::c_int,
                                            );
                                            ui_flush();
                                        }
                                        if os_copy(backup, fname, UV_FS_COPYFILE_FICLONE)
                                            == 0 as ::core::ffi::c_int
                                        {
                                            end = 1 as ::core::ffi::c_int as linenr_T;
                                        }
                                    } else if vim_rename(backup, fname) == 0 as ::core::ffi::c_int {
                                        end = 1 as ::core::ffi::c_int as linenr_T;
                                    }
                                }
                                break '_fail;
                            } else {
                                lnum -= start;
                                (*no_wait_return.ptr()) -= 1;
                                if !filtering {
                                    add_quoted_fname(
                                        IObuff.ptr() as *mut ::core::ffi::c_char,
                                        IOSIZE as size_t,
                                        buf,
                                        fname,
                                    );
                                    let mut insert_space: bool = false_0 != 0;
                                    if write_info.bw_conv_error != 0 {
                                        xstrlcat(
                                            IObuff.ptr() as *mut ::core::ffi::c_char,
                                            gettext(b" CONVERSION ERROR\0".as_ptr()
                                                as *const ::core::ffi::c_char),
                                            IOSIZE as size_t,
                                        );
                                        insert_space = true_0 != 0;
                                        if write_info.bw_conv_error_lnum != 0 as linenr_T {
                                            vim_snprintf_add(
                                                IObuff.ptr() as *mut ::core::ffi::c_char,
                                                IOSIZE as size_t,
                                                gettext(b" in line %ld;\0".as_ptr()
                                                    as *const ::core::ffi::c_char),
                                                write_info.bw_conv_error_lnum as int64_t,
                                            );
                                        }
                                    } else if notconverted {
                                        xstrlcat(
                                            IObuff.ptr() as *mut ::core::ffi::c_char,
                                            gettext(b"[NOT converted]\0".as_ptr()
                                                as *const ::core::ffi::c_char),
                                            IOSIZE as size_t,
                                        );
                                        insert_space = true_0 != 0;
                                    } else if converted {
                                        xstrlcat(
                                            IObuff.ptr() as *mut ::core::ffi::c_char,
                                            gettext(b"[converted]\0".as_ptr()
                                                as *const ::core::ffi::c_char),
                                            IOSIZE as size_t,
                                        );
                                        insert_space = true_0 != 0;
                                    }
                                    if device {
                                        xstrlcat(
                                            IObuff.ptr() as *mut ::core::ffi::c_char,
                                            gettext(b"[Device]\0".as_ptr()
                                                as *const ::core::ffi::c_char),
                                            IOSIZE as size_t,
                                        );
                                        insert_space = true_0 != 0;
                                    } else if newfile {
                                        xstrlcat(
                                            IObuff.ptr() as *mut ::core::ffi::c_char,
                                            gettext(
                                                b"[New]\0".as_ptr() as *const ::core::ffi::c_char
                                            ),
                                            IOSIZE as size_t,
                                        );
                                        insert_space = true_0 != 0;
                                    }
                                    if no_eol {
                                        xstrlcat(
                                            IObuff.ptr() as *mut ::core::ffi::c_char,
                                            gettext(
                                                b"[noeol]\0".as_ptr() as *const ::core::ffi::c_char
                                            ),
                                            IOSIZE as size_t,
                                        );
                                        insert_space = true_0 != 0;
                                    }
                                    if msg_add_fileformat(fileformat) {
                                        insert_space = true_0 != 0;
                                    }
                                    msg_add_lines(
                                        insert_space as ::core::ffi::c_int,
                                        lnum,
                                        nchars as off_T,
                                    );
                                    if !shortmess(SHM_WRITE as ::core::ffi::c_int) {
                                        if append {
                                            xstrlcat(
                                                IObuff.ptr() as *mut ::core::ffi::c_char,
                                                if shortmess(SHM_WRI as ::core::ffi::c_int)
                                                    as ::core::ffi::c_int
                                                    != 0
                                                {
                                                    gettext(b" [a]\0".as_ptr()
                                                        as *const ::core::ffi::c_char)
                                                } else {
                                                    gettext(b" appended\0".as_ptr()
                                                        as *const ::core::ffi::c_char)
                                                },
                                                IOSIZE as size_t,
                                            );
                                        } else {
                                            xstrlcat(
                                                IObuff.ptr() as *mut ::core::ffi::c_char,
                                                if shortmess(SHM_WRI as ::core::ffi::c_int)
                                                    as ::core::ffi::c_int
                                                    != 0
                                                {
                                                    gettext(b" [w]\0".as_ptr()
                                                        as *const ::core::ffi::c_char)
                                                } else {
                                                    gettext(b" written\0".as_ptr()
                                                        as *const ::core::ffi::c_char)
                                                },
                                                IOSIZE as size_t,
                                            );
                                        }
                                    }
                                    set_keep_msg(
                                        msg_progress(
                                            IObuff.ptr() as *mut ::core::ffi::c_char,
                                            b"bufwrite\0".as_ptr() as *const ::core::ffi::c_char
                                                as *mut ::core::ffi::c_char,
                                            b"success\0".as_ptr() as *const ::core::ffi::c_char
                                                as *mut ::core::ffi::c_char,
                                            0 as ::core::ffi::c_int,
                                            true_0 != 0,
                                            true_0 != 0,
                                        ),
                                        0 as ::core::ffi::c_int,
                                    );
                                }
                                if reset_changed as ::core::ffi::c_int != 0
                                    && whole as ::core::ffi::c_int != 0
                                    && !append
                                    && write_info.bw_conv_error == 0
                                    && (overwriting as ::core::ffi::c_int != 0
                                        || !vim_strchr(p_cpo.get(), CPO_PLUS).is_null())
                                {
                                    unchanged(buf, true_0 != 0, false_0 != 0);
                                    let changedtick: varnumber_T = buf_get_changedtick(buf);
                                    if (*buf).b_last_changedtick + 1 as varnumber_T == changedtick {
                                        (*buf).b_last_changedtick = changedtick;
                                    }
                                    u_unchanged(buf);
                                    u_update_save_nr(buf);
                                }
                                if overwriting {
                                    ml_timestamp(buf);
                                    if append {
                                        (*buf).b_flags &= !BF_NEW;
                                    } else {
                                        (*buf).b_flags &= !BF_WRITE_MASK;
                                    }
                                }
                                if *p_pm.get() as ::core::ffi::c_int != 0
                                    && dobackup as ::core::ffi::c_int != 0
                                {
                                    let org: *mut ::core::ffi::c_char =
                                        modname(fname, p_pm.get(), false_0 != 0);
                                    if !backup.is_null() {
                                        if org.is_null() {
                                            emsg(gettext(
                                                b"E205: Patchmode: can't save original file\0"
                                                    .as_ptr()
                                                    as *const ::core::ffi::c_char,
                                            ));
                                        } else if !os_path_exists(org) {
                                            vim_rename(backup, org);
                                            let mut ptr_: *mut *mut ::core::ffi::c_void =
                                                &raw mut backup as *mut *mut ::core::ffi::c_void;
                                            xfree(*ptr_);
                                            *ptr_ = NULL;
                                            let _ = *ptr_;
                                            os_file_settime(
                                                org,
                                                file_info_old.stat.st_atim.tv_sec
                                                    as ::core::ffi::c_double,
                                                file_info_old.stat.st_mtim.tv_sec
                                                    as ::core::ffi::c_double,
                                            );
                                        }
                                    } else {
                                        let mut empty_fd: ::core::ffi::c_int = 0;
                                        if org.is_null() || {
                                            empty_fd = os_open(
                                                org,
                                                O_CREAT | O_EXCL | O_NOFOLLOW,
                                                if perm < 0 as ::core::ffi::c_int {
                                                    0o666 as ::core::ffi::c_int
                                                } else {
                                                    perm & 0o777 as ::core::ffi::c_int
                                                },
                                            );
                                            empty_fd < 0 as ::core::ffi::c_int
                                        } {
                                            emsg(gettext(
                                                (e_patchmode_cant_touch_empty_original_file.ptr()
                                                    as *const _)
                                                    as *const ::core::ffi::c_char,
                                            ));
                                        } else {
                                            close(empty_fd);
                                        }
                                    }
                                    if !org.is_null() {
                                        os_setperm(
                                            org,
                                            os_getperm(fname) as ::core::ffi::c_int
                                                & 0o777 as ::core::ffi::c_int,
                                        );
                                        xfree(org as *mut ::core::ffi::c_void);
                                    }
                                }
                                if p_bk.get() == 0
                                    && !backup.is_null()
                                    && write_info.bw_conv_error == 0
                                    && os_remove(backup) != 0 as ::core::ffi::c_int
                                {
                                    emsg(gettext(b"E207: Can't delete backup file\0".as_ptr()
                                        as *const ::core::ffi::c_char));
                                }
                                break '_nofail;
                            }
                        } else {
                            checking_conversion = false_0 != 0;
                        }
                    }
                }
                if !backup.is_null() && wfname == fname {
                    if backup_copy {
                        if !os_path_exists(fname) {
                            vim_rename(backup, fname);
                        }
                        if os_path_exists(fname) {
                            os_remove(backup);
                        }
                    } else {
                        vim_rename(backup, fname);
                    }
                }
                if !newfile && !os_path_exists(fname) {
                    end = 0 as ::core::ffi::c_int as linenr_T;
                }
                if wfname != fname {
                    xfree(wfname as *mut ::core::ffi::c_void);
                }
            }
        }
        (*no_wait_return.ptr()) -= 1;
    }
    (*buf).b_saving = false_0 != 0;
    xfree(backup as *mut ::core::ffi::c_void);
    if buffer != &raw mut smallbuf as *mut ::core::ffi::c_char {
        xfree(buffer as *mut ::core::ffi::c_void);
    }
    xfree(fenc_tofree as *mut ::core::ffi::c_void);
    xfree(write_info.bw_conv_buf as *mut ::core::ffi::c_void);
    if write_info.bw_iconv_fd
        != ::core::ptr::with_exposed_provenance_mut::<::core::ffi::c_void>(
            -1 as ::core::ffi::c_int as usize,
        )
    {
        iconv_close(write_info.bw_iconv_fd);
        write_info.bw_iconv_fd = ::core::ptr::with_exposed_provenance_mut::<::core::ffi::c_void>(
            -1 as ::core::ffi::c_int as usize,
        );
    }
    os_free_acl(acl);
    if !err.msg.is_null() {
        add_quoted_fname(
            IObuff.ptr() as *mut ::core::ffi::c_char,
            (IOSIZE - 100 as ::core::ffi::c_int) as size_t,
            buf,
            fname,
        );
        emit_err(&raw mut err);
        retval = FAIL;
        if end == 0 as linenr_T {
            let hl_id: ::core::ffi::c_int = HLF_E as ::core::ffi::c_int;
            msg_puts_hl(
                gettext(
                    b"\nWARNING: Original file may be lost or damaged\n\0".as_ptr()
                        as *const ::core::ffi::c_char,
                ),
                hl_id,
                true_0 != 0,
            );
            msg_puts_hl(
                gettext(
                    b"don't quit the editor until the file is successfully written!\0".as_ptr()
                        as *const ::core::ffi::c_char,
                ),
                hl_id,
                true_0 != 0,
            );
            if os_fileinfo(fname, &raw mut file_info_old) {
                buf_store_file_info(buf, &raw mut file_info_old);
                (*buf).b_mtime_read = (*buf).b_mtime;
                (*buf).b_mtime_read_ns = (*buf).b_mtime_ns;
            }
        }
    }
    msg_scroll.set(msg_save);
    if retval == OK && write_undo_file as ::core::ffi::c_int != 0 {
        let mut hash: [uint8_t; 32] = sha_ctx.finish();
        u_write_undo(
            ::core::ptr::null::<::core::ffi::c_char>(),
            false_0 != 0,
            buf,
            &raw mut hash as *mut uint8_t,
        );
    }
    if !should_abort(retval) {
        buf_write_do_post_autocmds(buf, fname, eap, append, filtering, reset_changed, whole);
        if aborting() {
            retval = false_0;
        }
    }
    got_int.set(got_int.get() as ::core::ffi::c_int | prev_got_int as ::core::ffi::c_int != 0);
    return retval;
}
pub const IOSIZE: ::core::ffi::c_int = 1024 as ::core::ffi::c_int + 1 as ::core::ffi::c_int;
pub const FORCE_BIN: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const EOL_UNIX: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const EOL_DOS: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const EOL_MAC: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
pub const CPO_FNAMEW: ::core::ffi::c_int = 'F' as ::core::ffi::c_int;
pub const CPO_FNAMEAPP: ::core::ffi::c_int = 'P' as ::core::ffi::c_int;
pub const CPO_FWRITE: ::core::ffi::c_int = 'W' as ::core::ffi::c_int;
pub const CPO_KEEPRO: ::core::ffi::c_int = 'Z' as ::core::ffi::c_int;
pub const CPO_PLUS: ::core::ffi::c_int = '+' as ::core::ffi::c_int;
pub const __S_IFMT: ::core::ffi::c_int = 0o170000 as ::core::ffi::c_int;
pub const true_0: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const false_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const EINVAL: ::core::ffi::c_int = 22 as ::core::ffi::c_int;
pub const ICONV_EINVAL: ::core::ffi::c_int = EINVAL;
