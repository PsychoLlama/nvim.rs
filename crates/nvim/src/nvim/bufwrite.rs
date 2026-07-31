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

// The carve of the transpiled module; see each child's docs.
mod convert;
pub(crate) use self::convert::*;
mod backup;
pub use self::backup::*;
mod autocmds;
pub(crate) use self::autocmds::*;
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
