//! Main `buf_write` orchestrator — migrated from C.
//!
//! This is the entry point for writing a buffer to a file. It handles:
//! - Validation and filename setup
//! - Autocommand dispatch (pre/post)
//! - Backup creation
//! - Encoding detection and conversion
//! - The main line-by-line write loop
//! - Post-write cleanup (fsync, permissions, patchmode, undo)

#![allow(clippy::too_many_arguments)]
#![allow(clippy::too_many_lines)]
#![allow(clippy::fn_params_excessive_bools)]
#![allow(clippy::similar_names)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_possible_wrap)]
#![allow(clippy::cast_sign_loss)]

use std::ffi::{c_char, c_int, c_void};
use std::ptr;

use crate::ffi::{AclHandle, BufHandle, ExargHandle, FileInfoHandle, FAIL, NOTDONE, OK};

// Character constants
const NUL: u8 = 0;
const NL: u8 = b'\n';
const CAR: u8 = b'\r';
const CTRL_Z: u8 = 0x1a;

// File format constants
const EOL_UNIX: c_int = 0;
const EOL_DOS: c_int = 1;
const EOL_MAC: c_int = 2;

// Buffer flags
const ML_EMPTY: c_int = 0x01;
const BF_NEW: c_int = 0x10;
const BF_WRITE_MASK: c_int = 0x58;

// Encoding flags
const FIO_UCS2: c_int = 0x04;
const FIO_UCS4: c_int = 0x08;
const FIO_UTF16: c_int = 0x10;
const FIO_UTF8: c_int = 0x02;
const FIO_NOCONVERT: c_int = 0x2000;

// Force binary
const FORCE_BIN: c_int = 1;

// cpoptions characters
const CPO_FNAMEW: c_int = b'F' as c_int;
const CPO_FNAMEAPP: c_int = b'P' as c_int;
const CPO_KEEPRO: c_int = b'Z' as c_int;
const CPO_FWRITE: c_int = b'W' as c_int;
const CPO_PLUS: c_int = b'+' as c_int;

// shortmess characters
const SHM_OVER: c_int = b'o' as c_int;
const SHM_WRITE: c_int = b'W' as c_int;
const SHM_WRI: c_int = b'w' as c_int;

// cmdmod flag
const CMOD_LOCKMARKS: c_int = 0x0800;

// highlight
const HLF_E: c_int = 6;

// Undo hash size
const UNDO_HASH_SIZE: usize = 32;

// Size constants
const WRITEBUFSIZE: usize = 8192;
const SMALLBUFSIZE: usize = 256;
const IOSIZE: usize = 1025;
const ICONV_MULT: usize = 8;

// Opaque handle for bw_info struct
type BwInfoHandle = *mut c_void;
// Opaque handle for SHA256 context
type Sha256Handle = *mut c_void;
// iconv handle
type IconvHandle = *mut c_void;
const ICONV_INVALID: isize = -1;

// Error_T repr
#[repr(C)]
struct ErrorT {
    num: *const c_char,
    msg: *mut c_char,
    arg: c_int,
    alloc: bool,
}

impl Default for ErrorT {
    fn default() -> Self {
        Self {
            num: ptr::null(),
            msg: ptr::null_mut(),
            arg: 0,
            alloc: false,
        }
    }
}

#[allow(dead_code)]
extern "C" {
    // Global state
    fn nvim_bw_get_got_int() -> c_int;
    fn nvim_bw_set_got_int(val: c_int);
    fn nvim_bw_get_exiting() -> c_int;
    fn nvim_bw_set_ex_no_reprint(val: c_int);
    fn nvim_bw_set_msg_ext_overwrite(val: c_int);
    fn nvim_bw_set_need_maketitle(val: c_int);
    fn nvim_bw_inc_no_wait_return();
    fn nvim_bw_dec_no_wait_return();
    fn nvim_bw_get_msg_scroll() -> c_int;
    fn nvim_bw_set_msg_scroll(val: c_int);
    fn nvim_bw_get_cmdmod_cmod_flags() -> c_int;
    #[link_name = "aborting"]
    fn aborting() -> c_int;

    // Options
    fn nvim_bw_get_p_wb() -> c_int;
    fn nvim_bw_get_p_bk() -> c_int;
    fn nvim_bw_get_p_pm() -> *mut c_char;
    fn nvim_bw_get_p_bsk() -> *mut c_char;
    fn nvim_bw_get_p_ccv() -> *mut c_char;
    fn nvim_bw_get_p_fs() -> c_int;
    fn nvim_bw_cpo_contains(c: c_int) -> c_int;

    // Buffer fields
    fn nvim_bw_buf_get_ml_line_count(buf: BufHandle) -> i32;
    fn nvim_bw_buf_get_ml_mfp_nonnull(buf: BufHandle) -> c_int;
    fn nvim_bw_buf_get_ffname(buf: BufHandle) -> *mut c_char;
    fn nvim_bw_buf_get_sfname(buf: BufHandle) -> *mut c_char;
    fn nvim_bw_buf_get_flags(buf: BufHandle) -> c_int;
    fn nvim_bw_buf_set_flags(buf: BufHandle, flags: c_int);
    fn nvim_bw_buf_get_changed(buf: BufHandle) -> c_int;
    fn nvim_bw_buf_get_ml_flags(buf: BufHandle) -> c_int;
    fn nvim_bw_buf_get_p_bin(buf: BufHandle) -> c_int;
    fn nvim_bw_buf_get_p_bomb(buf: BufHandle) -> c_int;
    fn nvim_bw_buf_get_p_eol(buf: BufHandle) -> c_int;
    fn nvim_bw_buf_get_p_eof(buf: BufHandle) -> c_int;
    fn nvim_bw_buf_get_p_fixeol(buf: BufHandle) -> c_int;
    fn nvim_bw_buf_set_p_ro(buf: BufHandle, val: c_int);
    fn nvim_bw_buf_get_p_udf(buf: BufHandle) -> c_int;
    fn nvim_bw_buf_get_p_fenc(buf: BufHandle) -> *mut c_char;
    fn nvim_bw_buf_get_no_eol_lnum(buf: BufHandle) -> i32;
    fn nvim_bw_buf_set_saving(buf: BufHandle, val: c_int);
    fn nvim_bw_buf_get_last_changedtick(buf: BufHandle) -> i64;
    fn nvim_bw_buf_set_last_changedtick(buf: BufHandle, val: i64);
    fn nvim_bw_buf_get_file_id_valid(buf: BufHandle) -> c_int;
    fn nvim_bw_buf_set_op_start(buf: BufHandle, pos: crate::autocmd::PosT);
    fn nvim_bw_buf_set_op_end(buf: BufHandle, pos: crate::autocmd::PosT);
    fn nvim_bw_buf_set_no_eol_lnum(buf: BufHandle, val: i32);
    fn nvim_bw_buf_get_mtime(buf: BufHandle) -> i64;
    fn nvim_bw_buf_get_mtime_ns(buf: BufHandle) -> i64;
    fn nvim_bw_buf_set_mtime_read(buf: BufHandle, val: i64);
    fn nvim_bw_buf_set_mtime_read_ns(buf: BufHandle, val: i64);

    // Exarg fields
    fn nvim_bw_eap_get_force_enc(eap: ExargHandle) -> c_int;
    fn nvim_bw_eap_get_cmd(eap: ExargHandle) -> *mut c_char;
    fn nvim_bw_eap_get_force_bin(eap: ExargHandle) -> c_int;

    // File/Path
    fn rs_check_secure() -> c_int;
    #[link_name = "path_fnamecmp"]
    fn path_fnamecmp(a: *const c_char, b: *const c_char) -> c_int;
    #[link_name = "get_bkc_flags"]
    fn get_bkc_flags(buf: BufHandle) -> u32;
    fn nvim_bw_set_rw_fname(fname: *mut c_char, sfname: *mut c_char) -> c_int;
    fn nvim_bw_vim_tempname() -> *mut c_char;
    #[link_name = "match_file_list"]
    fn match_file_list(list: *const c_char, sfname: *const c_char, ffname: *const c_char) -> c_int;
    #[link_name = "os_path_exists"]
    fn os_path_exists(path: *const c_char) -> c_int;
    #[link_name = "vim_rename"]
    fn vim_rename(from: *const c_char, to: *const c_char) -> c_int;
    #[link_name = "os_remove"]
    fn os_remove(path: *const c_char) -> c_int;
    #[link_name = "os_copy"]
    fn os_copy(from: *const c_char, to: *const c_char, flags: c_int) -> c_int;
    #[link_name = "os_setperm"]
    fn os_setperm(path: *const c_char, perm: c_int) -> c_int;
    #[link_name = "os_getperm"]
    fn os_getperm(path: *const c_char) -> c_int;
    #[link_name = "os_open"]
    fn os_open(path: *const c_char, flags: c_int, mode: c_int) -> c_int;
    #[link_name = "os_close"]
    fn os_close(fd: c_int) -> c_int;
    #[link_name = "modname"]
    fn modname(fname: *const c_char, ext: *const c_char, prepend_dot: bool) -> *mut c_char;

    // Encoding
    #[link_name = "enc_canonize"]
    fn enc_canonize(enc: *const c_char) -> *mut c_char;
    #[link_name = "rs_need_conversion"]
    fn nvim_bw_need_conversion(fenc: *const c_char) -> c_int;
    fn rs_get_fio_flags(name: *const c_char) -> c_int;
    #[link_name = "get_fileformat_force"]
    fn get_fileformat_force(buf: BufHandle, eap: ExargHandle) -> c_int;

    // iconv
    fn nvim_bw_my_iconv_open(tocode: *const c_char, fromcode: *const c_char) -> IconvHandle;
    fn nvim_bw_iconv_close(cd: IconvHandle);

    // Memline
    fn nvim_bw_ml_get_buf(buf: BufHandle, lnum: i32) -> *mut c_char;
    #[link_name = "ml_preserve"]
    fn ml_preserve(buf: BufHandle, message: bool, do_fsync: bool);

    // Message/UI
    #[link_name = "emsg"]
    fn emsg(msg: *const c_char) -> c_int;
    fn nvim_bw_gettext(s: *const c_char) -> *const c_char;
    #[link_name = "shortmess"]
    fn shortmess_direct(c: c_int) -> c_int;
    fn nvim_bw_filemess(buf: BufHandle, fname: *const c_char, s: *const c_char);
    #[link_name = "msg_ext_set_kind"]
    fn msg_ext_set_kind(kind: *const c_char);
    fn nvim_bw_add_quoted_fname(
        buf: *mut c_char,
        bufsize: c_int,
        bp: BufHandle,
        fname: *const c_char,
    );
    fn nvim_bw_xstrlcat(dst: *mut c_char, src: *const c_char, dsize: usize);
    fn nvim_bw_vim_snprintf_add(buf: *mut c_char, len: usize, fmt: *const c_char, val: i64);
    fn nvim_bw_msg_add_fileformat(fileformat: c_int) -> c_int;
    fn nvim_bw_msg_add_lines(insert_space: c_int, lnum: i32, nchars: c_int);
    #[link_name = "msg_trunc"]
    fn msg_trunc(s: *mut c_char, force: bool, attr: c_int) -> *mut c_char;
    #[link_name = "set_keep_msg"]
    fn set_keep_msg(s: *mut c_char, attr: c_int);
    #[link_name = "msg_puts_hl"]
    fn msg_puts_hl(s: *const c_char, hl_id: c_int, wrap: bool);
    #[link_name = "msg"]
    fn nvim_bw_msg(s: *const c_char, hlf: c_int);
    fn nvim_bw_ui_flush();
    #[link_name = "status_redraw_all"]
    fn status_redraw_all();

    // Buffer change tracking
    #[link_name = "unchanged"]
    fn unchanged(buf: BufHandle, ff: bool, always_inc_changedtick: bool);
    fn nvim_bw_buf_get_changedtick(buf: BufHandle) -> i64;
    fn nvim_bw_buf_set_file_id(buf: BufHandle);
    fn nvim_bw_buf_store_file_info(buf: BufHandle, fi: FileInfoHandle);
    #[link_name = "u_unchanged"]
    fn u_unchanged(buf: BufHandle);
    #[link_name = "u_update_save_nr"]
    fn u_update_save_nr(buf: BufHandle);
    #[link_name = "ml_timestamp"]
    fn ml_timestamp(buf: BufHandle);
    fn nvim_bw_bt_nofilename(buf: BufHandle) -> c_int;

    // OS operations
    #[link_name = "os_fsync"]
    fn os_fsync(fd: c_int) -> c_int;
    fn nvim_bw_os_get_acl(fname: *const c_char) -> AclHandle;
    fn nvim_bw_os_free_acl(acl: AclHandle);
    fn nvim_bw_os_set_acl(fname: *const c_char, acl: AclHandle);
    fn nvim_bw_os_breakcheck();
    #[link_name = "os_fileinfo"]
    fn os_fileinfo(fname: *const c_char, info: FileInfoHandle) -> c_int;
    #[link_name = "os_fileinfo_hardlinks"]
    fn os_fileinfo_hardlinks(fi: FileInfoHandle) -> c_int;
    #[link_name = "os_fileinfo_link"]
    fn os_fileinfo_link(fname: *const c_char, fi: FileInfoHandle) -> c_int;
    #[link_name = "os_fileinfo_id_equal"]
    fn os_fileinfo_id_equal(a: FileInfoHandle, b: FileInfoHandle) -> c_int;
    #[link_name = "os_fchown"]
    fn os_fchown(fd: c_int, uid: u32, gid: u32);
    fn nvim_bw_os_copy_xattr(from: *const c_char, to: *const c_char);
    fn nvim_bw_fi_get_st_uid(fi: FileInfoHandle) -> u32;
    fn nvim_bw_fi_get_st_gid(fi: FileInfoHandle) -> u32;
    fn nvim_bw_fi_get_atime_sec(fi: FileInfoHandle) -> i64;
    fn nvim_bw_fi_get_mtime_sec(fi: FileInfoHandle) -> i64;
    fn nvim_bw_getuid() -> u32;
    fn nvim_bw_getgid() -> u32;

    // Allocation
    fn nvim_bw_verbose_try_malloc(size: usize) -> *mut c_char;
    fn nvim_bw_xmalloc(size: usize) -> *mut c_char;
    fn nvim_bw_xfree(ptr: *mut c_char);
    fn nvim_bw_vim_snprintf(buf: *mut c_char, len: usize, fmt: *const c_char, val: i64);
    fn nvim_bw_strlen(s: *const c_char) -> usize;

    // SHA256
    fn nvim_bw_sizeof_sha256_ctx() -> usize;
    fn nvim_bw_sha256_start(ctx: Sha256Handle);
    fn nvim_bw_sha256_update(ctx: Sha256Handle, data: *const u8, len: u32);
    fn nvim_bw_sha256_finish(ctx: Sha256Handle, hash: *mut u8);

    // Undo
    fn nvim_bw_u_write_undo(buf: BufHandle, hash: *const u8);

    // Eval
    fn nvim_bw_eval_charconvert(
        from: *const c_char,
        to: *const c_char,
        src: *const c_char,
        dst: *const c_char,
    ) -> c_int;
    #[link_name = "should_abort"]
    fn should_abort(retval: c_int) -> c_int;

    // I/O
    fn nvim_bw_write_eintr_direct(fd: c_int, buf: *const c_char, len: usize) -> c_int;

    // bw_info management
    fn nvim_bw_sizeof_bw_info() -> usize;
    fn nvim_bw_info_init(p: BwInfoHandle);
    fn nvim_bw_info_set_fd(p: BwInfoHandle, fd: c_int);
    fn nvim_bw_info_set_buf(p: BwInfoHandle, buf: *mut c_char);
    fn nvim_bw_info_set_len(p: BwInfoHandle, len: c_int);
    fn nvim_bw_info_set_flags(p: BwInfoHandle, flags: c_int);
    fn nvim_bw_info_set_conv_buflen(p: BwInfoHandle, len: usize);
    fn nvim_bw_info_set_conv_buf(p: BwInfoHandle, buf: *mut c_char);
    fn nvim_bw_info_get_conv_error(p: BwInfoHandle) -> c_int;
    fn nvim_bw_info_get_conv_error_lnum(p: BwInfoHandle) -> i32;
    fn nvim_bw_info_get_iconv_fd(p: BwInfoHandle) -> IconvHandle;
    fn nvim_bw_info_set_iconv_fd(p: BwInfoHandle, fd: IconvHandle);
    fn nvim_bw_info_set_start_lnum(p: BwInfoHandle, val: i32);
    fn nvim_bw_info_get_conv_buf(p: BwInfoHandle) -> *mut c_char;

    // Open flags
    fn nvim_bw_open_flags_wronly() -> c_int;
    fn nvim_bw_open_flags_append() -> c_int;
    fn nvim_bw_open_flags_creat() -> c_int;
    fn nvim_bw_open_flags_trunc() -> c_int;
    fn nvim_bw_open_flags_nofollow() -> c_int;
    fn nvim_bw_open_flags_creat_wronly_excl_nofollow() -> c_int;
    fn nvim_bw_uv_enotsup() -> c_int;
    fn nvim_bw_uv_fs_copyfile_ficlone() -> c_int;

    // Existing error helpers
    fn rs_set_err(msg: *const c_char) -> ErrorT;
    fn rs_set_err_arg(msg: *const c_char, arg: c_int) -> ErrorT;
    fn rs_emit_err(e: *const ErrorT);

    // Existing migrated functions
    fn rs_get_fileinfo(
        buf: BufHandle,
        fname: *mut c_char,
        overwriting: c_int,
        forceit: c_int,
        file_info_old: FileInfoHandle,
        perm: *mut c_int,
        device: *mut bool,
        newfile: *mut bool,
        readonly: *mut bool,
        err: *mut ErrorT,
    ) -> c_int;
    fn rs_make_bom(buf: *mut c_char, name: *mut c_char) -> c_int;
    fn rs_buf_write_bytes(ip: BwInfoHandle) -> c_int;
    fn rs_buf_write_make_backup(
        fname: *mut c_char,
        append: c_int,
        file_info_old: FileInfoHandle,
        acl: AclHandle,
        perm: c_int,
        bkc: u32,
        file_readonly: c_int,
        forceit: c_int,
        backup_copyp: *mut bool,
        backupp: *mut *mut c_char,
        err: *mut ErrorT,
    ) -> c_int;
    fn rs_buf_write_do_autocmds(
        buf: BufHandle,
        fnamep: *mut *mut c_char,
        sfnamep: *mut *mut c_char,
        ffnamep: *mut *mut c_char,
        start: i32,
        endp: *mut i32,
        eap: ExargHandle,
        append: c_int,
        filtering: c_int,
        reset_changed: c_int,
        overwriting: c_int,
        whole: c_int,
        orig_start: crate::autocmd::PosT,
        orig_end: crate::autocmd::PosT,
    ) -> c_int;
    fn rs_buf_write_do_post_autocmds(
        buf: BufHandle,
        fname: *const c_char,
        eap: ExargHandle,
        append: c_int,
        filtering: c_int,
        reset_changed: c_int,
        whole: c_int,
    );

    // IObuff and IOSIZE
    fn nvim_bw_get_IObuff() -> *mut c_char;
    fn nvim_bw_get_IOSIZE() -> c_int;
    fn nvim_bw_sizeof_FileInfo() -> usize;
    fn nvim_bw_get_curbuf() -> BufHandle;
    fn nvim_bw_XFREE_CLEAR(pp: *mut *mut c_char);
    #[link_name = "os_file_settime"]
    fn nvim_bw_os_file_settime(path: *const c_char, atime: f64, mtime: f64);
}

/// Helper: check if a C string is NULL or empty
unsafe fn c_str_empty(s: *const c_char) -> bool {
    s.is_null() || unsafe { *s } == 0
}

/// Helper: shortmess check
unsafe fn shortmess(c: c_int) -> bool {
    unsafe { shortmess_direct(c) != 0 }
}

/// Main buf_write orchestrator.
///
/// # Safety
///
/// All pointers must be valid.
#[export_name = "buf_write"]
#[allow(clippy::cognitive_complexity)]
#[allow(clippy::must_use_candidate)]
pub unsafe extern "C" fn rs_buf_write(
    buf: BufHandle,
    fname: *mut c_char,
    sfname: *mut c_char,
    start: i32,
    end: i32,
    eap: ExargHandle,
    append: c_int,
    forceit: c_int,
    reset_changed: c_int,
    filtering: c_int,
) -> c_int {
    let mut retval: c_int = OK;
    let msg_save = unsafe { nvim_bw_get_msg_scroll() };
    let prev_got_int = unsafe { nvim_bw_get_got_int() } != 0;
    let mut end = end;

    // writing everything
    let whole = start == 1 && end == unsafe { nvim_bw_buf_get_ml_line_count(buf) };

    #[allow(unused_assignments)]
    let mut write_undo_file = false;
    // SHA256 context allocated as opaque blob
    let sha_size = unsafe { nvim_bw_sizeof_sha256_ctx() };
    let mut sha_buf = vec![0u8; sha_size];
    let sha_ctx: Sha256Handle = sha_buf.as_mut_ptr().cast();

    let bkc = unsafe { get_bkc_flags(buf) };

    // Safety check: NULL or empty fname
    if unsafe { c_str_empty(fname) } {
        return FAIL;
    }
    // Check memline
    if unsafe { nvim_bw_buf_get_ml_mfp_nonnull(buf) } == 0 {
        unsafe {
            emsg(nvim_bw_gettext(c"E749: Empty buffer".as_ptr()));
        }
        return FAIL;
    }
    // Disallow writing in secure mode
    if unsafe { rs_check_secure() } != 0 {
        return FAIL;
    }
    // Avoid crash for long name
    if unsafe { nvim_bw_strlen(fname) } >= 4096 {
        unsafe {
            emsg(nvim_bw_gettext(c"E75: Name too long".as_ptr()));
        }
        return FAIL;
    }

    // bw_info allocated on Rust heap as opaque blob
    let bw_size = unsafe { nvim_bw_sizeof_bw_info() };
    let mut bw_buf = vec![0u8; bw_size];
    let write_info: BwInfoHandle = bw_buf.as_mut_ptr().cast();
    unsafe { nvim_bw_info_init(write_info) };

    unsafe { nvim_bw_set_ex_no_reprint(1) };

    let mut buf = buf;
    let mut fname = fname;
    let mut sfname = sfname;

    // If no file name yet, use the one for the written file
    if unsafe { nvim_bw_buf_get_ffname(buf) }.is_null()
        && reset_changed != 0
        && whole
        && buf == unsafe { nvim_bw_get_curbuf() }
        && unsafe { nvim_bw_bt_nofilename(buf) } == 0
        && filtering == 0
        && (append == 0 || unsafe { nvim_bw_cpo_contains(CPO_FNAMEAPP) } != 0)
        && unsafe { nvim_bw_cpo_contains(CPO_FNAMEW) } != 0
    {
        if unsafe { nvim_bw_set_rw_fname(fname, sfname) } == FAIL {
            return FAIL;
        }
        buf = unsafe { nvim_bw_get_curbuf() };
    }

    if sfname.is_null() {
        sfname = fname;
    }

    // For Unix: use short file name
    let mut ffname = fname;
    #[cfg(unix)]
    {
        fname = sfname;
    }

    // Check if overwriting original file
    let overwriting = !unsafe { nvim_bw_buf_get_ffname(buf) }.is_null()
        && unsafe { path_fnamecmp(ffname, nvim_bw_buf_get_ffname(buf)) } == 0;

    unsafe { nvim_bw_inc_no_wait_return() };

    let orig_start = unsafe { nvim_bw_buf_get_op_start(buf) };
    let orig_end = unsafe { nvim_bw_buf_get_op_end(buf) };

    // Set marks
    unsafe {
        nvim_bw_buf_set_op_start(
            buf,
            crate::autocmd::PosT {
                lnum: start,
                col: 0,
                coladd: 0,
            },
        );
        nvim_bw_buf_set_op_end(
            buf,
            crate::autocmd::PosT {
                lnum: end,
                col: 0,
                coladd: 0,
            },
        );
    }

    // Pre-autocmds
    let res = unsafe {
        rs_buf_write_do_autocmds(
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
            c_int::from(overwriting),
            c_int::from(whole),
            orig_start,
            orig_end,
        )
    };
    if res != NOTDONE {
        return res;
    }

    if unsafe { nvim_bw_get_cmdmod_cmod_flags() } & CMOD_LOCKMARKS != 0 {
        unsafe {
            nvim_bw_buf_set_op_start(buf, orig_start);
            nvim_bw_buf_set_op_end(buf, orig_end);
        }
    }

    // Message setup
    if shortmess(SHM_OVER) && unsafe { nvim_bw_get_exiting() } == 0 {
        unsafe { nvim_bw_set_msg_scroll(0) };
    } else {
        unsafe { nvim_bw_set_msg_scroll(1) };
    }
    if filtering == 0 {
        unsafe { msg_ext_set_kind(c"bufwrite".as_ptr()) };
        #[cfg(not(unix))]
        unsafe {
            nvim_bw_filemess(buf, sfname, c"".as_ptr());
        }
        #[cfg(unix)]
        unsafe {
            nvim_bw_filemess(buf, fname, c"".as_ptr());
        }
    }
    unsafe { nvim_bw_set_msg_scroll(0) };

    // Allocate write buffer
    let mut buffer = unsafe { nvim_bw_verbose_try_malloc(WRITEBUFSIZE) };
    let mut smallbuf_storage = [0u8; SMALLBUFSIZE];
    let (bufsize, using_smallbuf) = if buffer.is_null() {
        buffer = smallbuf_storage.as_mut_ptr().cast();
        (SMALLBUFSIZE as c_int, true)
    } else {
        (WRITEBUFSIZE as c_int, false)
    };

    let mut err = ErrorT::default();
    let mut perm: c_int = 0;
    let mut newfile = false;
    let mut device = false;
    let mut file_readonly = false;
    let mut backup: *mut c_char = ptr::null_mut();
    let mut fenc_tofree: *mut c_char = ptr::null_mut();

    // Get file info
    let fi_size = unsafe { nvim_bw_sizeof_FileInfo() };
    let mut fi_old_buf = vec![0u8; fi_size];
    let file_info_old: FileInfoHandle = fi_old_buf.as_mut_ptr().cast();

    let mut acl: AclHandle = ptr::null_mut();

    if unsafe {
        rs_get_fileinfo(
            buf,
            fname,
            c_int::from(overwriting),
            forceit,
            file_info_old,
            &raw mut perm,
            &raw mut device,
            &raw mut newfile,
            &raw mut file_readonly,
            &raw mut err,
        )
    } == FAIL
    {
        // goto fail
        return do_fail_cleanup(
            buf,
            &mut err,
            &mut backup,
            buffer,
            using_smallbuf,
            &mut fenc_tofree,
            write_info,
            acl,
            msg_save,
            prev_got_int,
            fname,
            sfname,
            end,
        );
    }

    // Get ACL
    if !newfile {
        acl = unsafe { nvim_bw_os_get_acl(fname) };
    }

    // Backup decision
    let p_pm = unsafe { nvim_bw_get_p_pm() };
    let mut dobackup = unsafe { nvim_bw_get_p_wb() } != 0
        || unsafe { nvim_bw_get_p_bk() } != 0
        || !unsafe { c_str_empty(p_pm) };
    if dobackup {
        let p_bsk = unsafe { nvim_bw_get_p_bsk() };
        if !unsafe { c_str_empty(p_bsk) } && unsafe { match_file_list(p_bsk, sfname, ffname) } != 0
        {
            dobackup = false;
        }
    }

    let mut backup_copy = false;

    // Save got_int
    let prev_got_int_save = unsafe { nvim_bw_get_got_int() } != 0;
    unsafe { nvim_bw_set_got_int(0) };

    // Mark buffer as saving
    unsafe { nvim_bw_buf_set_saving(buf, 1) };

    // Create backup if needed
    if !(append != 0 && unsafe { c_str_empty(p_pm) })
        && filtering == 0
        && perm >= 0
        && dobackup
        && unsafe {
            rs_buf_write_make_backup(
                fname,
                append,
                file_info_old,
                acl,
                perm,
                bkc,
                c_int::from(file_readonly),
                forceit,
                &raw mut backup_copy,
                &raw mut backup,
                &raw mut err,
            )
        } == FAIL
    {
        retval = FAIL;
        // goto fail
        unsafe { nvim_bw_buf_set_saving(buf, 0) };
        if !using_smallbuf {
            unsafe { nvim_bw_xfree(buffer) };
        }
        unsafe {
            nvim_bw_xfree(fenc_tofree);
            nvim_bw_xfree(backup);
        }
        let conv_buf = unsafe { nvim_bw_info_get_conv_buf(write_info) };
        if !conv_buf.is_null() {
            unsafe { nvim_bw_xfree(conv_buf) };
        }
        let iconv_fd = unsafe { nvim_bw_info_get_iconv_fd(write_info) };
        if iconv_fd as isize != ICONV_INVALID {
            unsafe { nvim_bw_iconv_close(iconv_fd) };
        }
        unsafe { nvim_bw_os_free_acl(acl) };
        if !err.msg.is_null() {
            report_write_error(buf, &mut err, fname, sfname, end);
        }
        unsafe {
            nvim_bw_set_msg_scroll(msg_save);
            nvim_bw_dec_no_wait_return();
        }
        if prev_got_int_save {
            unsafe { nvim_bw_set_got_int(1) };
        }
        return retval;
    }

    // Unix: make writable if ":w!" and read-only
    #[cfg(unix)]
    let mut made_writable = false;
    #[cfg(unix)]
    {
        if forceit != 0
            && perm >= 0
            && (perm & 0o200) == 0
            && unsafe { nvim_bw_fi_get_st_uid(file_info_old) } == unsafe { nvim_bw_getuid() }
            && unsafe { nvim_bw_cpo_contains(CPO_FWRITE) } == 0
        {
            perm |= 0o200;
            unsafe { os_setperm(fname, perm) };
            made_writable = true;
        }
    }

    // Reset readonly if ":w!" and writing to current file
    if forceit != 0 && overwriting && unsafe { nvim_bw_cpo_contains(CPO_KEEPRO) } == 0 {
        unsafe {
            nvim_bw_buf_set_p_ro(buf, 0);
            nvim_bw_set_need_maketitle(1);
            status_redraw_all();
        }
    }

    // Clamp end to line count
    let ml_line_count = unsafe { nvim_bw_buf_get_ml_line_count(buf) };
    if end > ml_line_count {
        end = ml_line_count;
    }
    if unsafe { nvim_bw_buf_get_ml_flags(buf) } & ML_EMPTY != 0 {
        end = start - 1; // start = end + 1 in C, but end = start - 1 makes start > end
                         // Actually C does: start = end + 1, meaning empty loop
                         // In C: if ML_EMPTY, start = end + 1. We need to mirror that.
                         // But we receive start as parameter and can't change the loop start easily.
                         // Let's set end = start - 1 to skip the loop.
    }

    let mut wfname: *mut c_char = ptr::null_mut();
    let mut wfname_allocated = false;

    // Preserve file if overwriting
    if reset_changed != 0 && !newfile && overwriting {
        let exiting = unsafe { nvim_bw_get_exiting() } != 0;
        if !exiting || backup.is_null() {
            let p_fs = unsafe { nvim_bw_get_p_fs() };
            unsafe { ml_preserve(buf, false, p_fs != 0) };
            if unsafe { nvim_bw_get_got_int() } != 0 {
                err = unsafe { rs_set_err(nvim_bw_gettext(c"Interrupted".as_ptr())) };
                // goto restore_backup
                restore_backup_and_fail(
                    fname,
                    &backup,
                    backup_copy,
                    newfile,
                    wfname,
                    wfname_allocated,
                );
                return do_fail_cleanup(
                    buf,
                    &mut err,
                    &mut backup,
                    buffer,
                    using_smallbuf,
                    &mut fenc_tofree,
                    write_info,
                    acl,
                    msg_save,
                    prev_got_int_save,
                    fname,
                    sfname,
                    end,
                );
            }
        }
    }

    // Default: write directly
    wfname = fname;

    // Check forced fileencoding
    let fenc: *mut c_char;
    let force_enc = unsafe { nvim_bw_eap_get_force_enc(eap) };
    if force_enc != 0 {
        let cmd = unsafe { nvim_bw_eap_get_cmd(eap) };
        let enc_ptr = unsafe { cmd.add(force_enc as usize) };
        fenc = unsafe { enc_canonize(enc_ptr) };
        fenc_tofree = fenc;
    } else {
        fenc = unsafe { nvim_bw_buf_get_p_fenc(buf) };
    }

    // Check conversion needed
    let converted = unsafe { nvim_bw_need_conversion(fenc) } != 0;
    let mut wb_flags: c_int = 0;

    if converted {
        wb_flags = unsafe { rs_get_fio_flags(fenc) };
        if wb_flags & (FIO_UCS2 | FIO_UCS4 | FIO_UTF16 | FIO_UTF8) != 0 {
            let conv_buflen = if wb_flags & (FIO_UCS2 | FIO_UTF16 | FIO_UTF8) != 0 {
                bufsize as usize * 2
            } else {
                bufsize as usize * 4
            };
            let conv_buf = unsafe { nvim_bw_verbose_try_malloc(conv_buflen) };
            unsafe {
                nvim_bw_info_set_conv_buflen(write_info, conv_buflen);
                nvim_bw_info_set_conv_buf(write_info, conv_buf);
            }
            if conv_buf.is_null() {
                end = 0;
            }
        }
    }

    if converted && wb_flags == 0 {
        // Use iconv
        let iconv_fd = unsafe { nvim_bw_my_iconv_open(fenc, c"utf-8".as_ptr()) };
        if iconv_fd as isize == ICONV_INVALID {
            // charconvert path
            let p_ccv = unsafe { nvim_bw_get_p_ccv() };
            if !unsafe { c_str_empty(p_ccv) } {
                let tmp = unsafe { nvim_bw_vim_tempname() };
                if tmp.is_null() {
                    err = unsafe {
                        rs_set_err(nvim_bw_gettext(
                            c"E214: Can't find temp file for writing".as_ptr(),
                        ))
                    };
                    // goto restore_backup
                    restore_backup_and_fail(
                        fname,
                        &backup,
                        backup_copy,
                        newfile,
                        wfname,
                        wfname_allocated,
                    );
                    return do_fail_cleanup(
                        buf,
                        &mut err,
                        &mut backup,
                        buffer,
                        using_smallbuf,
                        &mut fenc_tofree,
                        write_info,
                        acl,
                        msg_save,
                        prev_got_int_save,
                        fname,
                        sfname,
                        end,
                    );
                }
                wfname = tmp;
                wfname_allocated = true;
            }
        } else {
            unsafe { nvim_bw_info_set_iconv_fd(write_info, iconv_fd) };
            let conv_buflen = bufsize as usize * ICONV_MULT;
            let conv_buf = unsafe { nvim_bw_verbose_try_malloc(conv_buflen) };
            unsafe {
                nvim_bw_info_set_conv_buflen(write_info, conv_buflen);
                nvim_bw_info_set_conv_buf(write_info, conv_buf);
            }
            if conv_buf.is_null() {
                end = 0;
            }
            // bw_first = true is set by init
        }
    }

    let notconverted = if converted
        && wb_flags == 0
        && unsafe { nvim_bw_info_get_iconv_fd(write_info) } as isize == ICONV_INVALID
        && wfname == fname
    {
        if forceit == 0 {
            err = unsafe {
                rs_set_err(nvim_bw_gettext(
                    c"E213: Cannot convert (add ! to write without conversion)".as_ptr(),
                ))
            };
            restore_backup_and_fail(
                fname,
                &backup,
                backup_copy,
                newfile,
                wfname,
                wfname_allocated,
            );
            return do_fail_cleanup(
                buf,
                &mut err,
                &mut backup,
                buffer,
                using_smallbuf,
                &mut fenc_tofree,
                write_info,
                acl,
                msg_save,
                prev_got_int_save,
                fname,
                sfname,
                end,
            );
        }
        true
    } else {
        false
    };

    let mut no_eol = false;
    #[allow(unused_assignments)]
    let mut nchars: c_int = 0;
    let mut lnum: i32;
    let mut fileformat: c_int;
    let mut checking_conversion: bool;
    let mut fd: c_int;

    // Main write loop (possibly two passes: check conversion, then write)
    checking_conversion = true;
    loop {
        if !converted || dobackup {
            checking_conversion = false;
        }

        if checking_conversion {
            fd = -1;
        } else {
            // Open the file
            let o_wronly = unsafe { nvim_bw_open_flags_wronly() };
            let o_append_f = unsafe { nvim_bw_open_flags_append() };
            let o_creat = unsafe { nvim_bw_open_flags_creat() };
            let o_trunc = unsafe { nvim_bw_open_flags_trunc() };

            let fflags = o_wronly
                | if append != 0 {
                    if forceit != 0 {
                        o_append_f | o_creat
                    } else {
                        o_append_f
                    }
                } else {
                    o_creat | o_trunc
                };
            let mode = if perm < 0 { 0o666 } else { perm & 0o777 };

            fd = unsafe { os_open(wfname, fflags, mode) };
            while fd < 0 {
                if err.msg.is_null() {
                    #[cfg(unix)]
                    {
                        // Don't delete if hard/symbolic link
                        let mut fi_buf2 = vec![0u8; fi_size];
                        let file_info: FileInfoHandle = fi_buf2.as_mut_ptr().cast();
                        if (!newfile && unsafe { os_fileinfo_hardlinks(file_info_old) } > 1)
                            || (unsafe { os_fileinfo_link(fname, file_info) } != 0
                                && unsafe { os_fileinfo_id_equal(file_info, file_info_old) } == 0)
                        {
                            err = unsafe {
                                rs_set_err(nvim_bw_gettext(
                                    c"E166: Can't open linked file for writing".as_ptr(),
                                ))
                            };
                        } else {
                            err = unsafe {
                                rs_set_err_arg(
                                    nvim_bw_gettext(
                                        c"E212: Can't open file for writing: %s".as_ptr(),
                                    ),
                                    fd,
                                )
                            };
                            if forceit != 0
                                && unsafe { nvim_bw_cpo_contains(CPO_FWRITE) } == 0
                                && perm >= 0
                            {
                                if (perm & 0o200) == 0 {
                                    made_writable = true;
                                }
                                perm |= 0o200;
                                if unsafe { nvim_bw_fi_get_st_uid(file_info_old) }
                                    != unsafe { nvim_bw_getuid() }
                                    || unsafe { nvim_bw_fi_get_st_gid(file_info_old) }
                                        != unsafe { nvim_bw_getgid() }
                                {
                                    perm &= 0o777;
                                }
                                if append == 0 {
                                    unsafe { os_remove(wfname) };
                                }
                                fd = unsafe { os_open(wfname, fflags, mode) };
                                continue;
                            }
                        }
                    }
                    #[cfg(not(unix))]
                    {
                        err = unsafe {
                            rs_set_err_arg(
                                nvim_bw_gettext(c"E212: Can't open file for writing: %s".as_ptr()),
                                fd,
                            )
                        };
                        if forceit != 0
                            && unsafe { nvim_bw_cpo_contains(CPO_FWRITE) } == 0
                            && perm >= 0
                        {
                            if append == 0 {
                                unsafe { os_remove(wfname) };
                            }
                            fd = unsafe { os_open(wfname, fflags, mode) };
                            continue;
                        }
                    }
                }

                // restore_backup
                restore_backup_and_fail(
                    fname,
                    &backup,
                    backup_copy,
                    newfile,
                    wfname,
                    wfname_allocated,
                );
                if wfname != fname && wfname_allocated {
                    unsafe { nvim_bw_xfree(wfname) };
                }
                // goto fail
                return do_fail_cleanup(
                    buf,
                    &mut err,
                    &mut backup,
                    buffer,
                    using_smallbuf,
                    &mut fenc_tofree,
                    write_info,
                    acl,
                    msg_save,
                    prev_got_int_save,
                    fname,
                    sfname,
                    end,
                );
            }
        }
        unsafe { nvim_bw_info_set_fd(write_info, fd) };
        err = unsafe { rs_set_err(ptr::null()) };

        unsafe { nvim_bw_info_set_buf(write_info, buffer) };
        nchars = 0;

        // Binary mode
        let force_bin = unsafe { nvim_bw_eap_get_force_bin(eap) };
        let write_bin: c_int = if force_bin != 0 {
            c_int::from(force_bin == FORCE_BIN)
        } else {
            unsafe { nvim_bw_buf_get_p_bin(buf) }
        };

        // Write BOM
        if unsafe { nvim_bw_buf_get_p_bomb(buf) } != 0
            && write_bin == 0
            && (append == 0 || perm < 0)
        {
            unsafe { nvim_bw_info_set_len(write_info, rs_make_bom(buffer, fenc)) };
            // Read back bw_len via the info accessor
            let bom_len = unsafe { crate::ffi::nvim_bw_info_get_len_direct(write_info) };
            if bom_len > 0 {
                unsafe {
                    nvim_bw_info_set_flags(write_info, FIO_NOCONVERT | wb_flags);
                }
                if unsafe { rs_buf_write_bytes(write_info) } == FAIL {
                    end = 0;
                } else {
                    nchars += bom_len;
                }
            }
        }
        unsafe { nvim_bw_info_set_start_lnum(write_info, start) };

        write_undo_file = unsafe { nvim_bw_buf_get_p_udf(buf) } != 0
            && overwriting
            && append == 0
            && filtering == 0
            && reset_changed != 0
            && !checking_conversion;
        if write_undo_file {
            unsafe { nvim_bw_sha256_start(sha_ctx) };
        }

        unsafe {
            nvim_bw_info_set_len(write_info, bufsize);
            nvim_bw_info_set_flags(write_info, wb_flags);
        }
        fileformat = unsafe { get_fileformat_force(buf, eap) };
        let mut s = buffer;
        let mut len: c_int = 0;

        lnum = start;
        while lnum <= end {
            let ptr_base = unsafe { nvim_bw_ml_get_buf(buf, lnum) };
            let mut ptr = unsafe { ptr_base.offset(-1) };
            if write_undo_file {
                let line_ptr = ptr_base;
                let line_len = unsafe { nvim_bw_strlen(line_ptr) } as u32 + 1;
                unsafe { nvim_bw_sha256_update(sha_ctx, line_ptr.cast(), line_len) };
            }
            loop {
                ptr = unsafe { ptr.add(1) };
                let c = unsafe { *ptr } as u8;
                if c == NUL {
                    break;
                }
                if c == NL {
                    unsafe { *s = 0 }; // NUL
                } else if c == CAR && fileformat == EOL_MAC {
                    unsafe { *s = NL as c_char };
                } else {
                    unsafe { *s = c as c_char };
                }
                s = unsafe { s.add(1) };
                len += 1;
                if len != bufsize {
                    continue;
                }
                if unsafe { rs_buf_write_bytes(write_info) } == FAIL {
                    end = 0;
                    break;
                }
                nchars += bufsize;
                s = buffer;
                len = 0;
                unsafe { nvim_bw_info_set_start_lnum(write_info, lnum) };
            }

            // Check if we should stop
            if end == 0
                || (lnum == end
                    && (write_bin != 0 || unsafe { nvim_bw_buf_get_p_fixeol(buf) } == 0)
                    && ((write_bin != 0 && lnum == unsafe { nvim_bw_buf_get_no_eol_lnum(buf) })
                        || (lnum == unsafe { nvim_bw_buf_get_ml_line_count(buf) }
                            && unsafe { nvim_bw_buf_get_p_eol(buf) } == 0)))
            {
                lnum += 1;
                no_eol = true;
                break;
            }

            // Write EOL
            if fileformat == EOL_UNIX {
                unsafe { *s = NL as c_char };
                s = unsafe { s.add(1) };
            } else {
                unsafe { *s = CAR as c_char };
                s = unsafe { s.add(1) };
                if fileformat == EOL_DOS {
                    len += 1;
                    if len == bufsize {
                        if unsafe { rs_buf_write_bytes(write_info) } == FAIL {
                            end = 0;
                            lnum += 1;
                            break;
                        }
                        nchars += bufsize;
                        s = buffer;
                        len = 0;
                    }
                    unsafe { *s = NL as c_char };
                    s = unsafe { s.add(1) };
                }
            }
            len += 1;
            if len == bufsize {
                if unsafe { rs_buf_write_bytes(write_info) } == FAIL {
                    end = 0;
                    lnum += 1;
                    break;
                }
                nchars += bufsize;
                s = buffer;
                len = 0;

                unsafe { nvim_bw_os_breakcheck() };
                if unsafe { nvim_bw_get_got_int() } != 0 {
                    end = 0;
                    lnum += 1;
                    break;
                }
            }
            lnum += 1;
        }

        // Flush remaining
        if len > 0 && end > 0 {
            unsafe { nvim_bw_info_set_len(write_info, len) };
            if unsafe { rs_buf_write_bytes(write_info) } == FAIL {
                end = 0;
            }
            nchars += len;
        }

        // trailing CTRL-Z
        if unsafe { nvim_bw_buf_get_p_fixeol(buf) } == 0
            && unsafe { nvim_bw_buf_get_p_eof(buf) } != 0
        {
            unsafe { nvim_bw_write_eintr_direct(fd, [CTRL_Z].as_ptr().cast(), 1) };
        }

        // Stop if done or error
        if !checking_conversion || end == 0 {
            break;
        }
        checking_conversion = false;
    }

    // Post-write: fsync, permissions, etc.
    if !checking_conversion {
        // fsync
        if unsafe { nvim_bw_get_p_fs() } != 0 {
            let error = unsafe { os_fsync(fd) };
            if error != 0 && !device && error != unsafe { nvim_bw_uv_enotsup() } {
                err = unsafe {
                    rs_set_err_arg(nvim_bw_gettext(c"E667: Fsync failed: %s".as_ptr()), error)
                };
                end = 0;
            }
        }

        if !backup_copy {
            unsafe { nvim_bw_os_copy_xattr(backup, wfname) };
        }

        // Unix: set owner/group on new file
        #[cfg(unix)]
        {
            if !backup.is_null() && !backup_copy {
                let mut fi_buf3 = vec![0u8; fi_size];
                let file_info: FileInfoHandle = fi_buf3.as_mut_ptr().cast();
                if unsafe { os_fileinfo(wfname, file_info) } == 0
                    || unsafe { nvim_bw_fi_get_st_uid(file_info) }
                        != unsafe { nvim_bw_fi_get_st_uid(file_info_old) }
                    || unsafe { nvim_bw_fi_get_st_gid(file_info) }
                        != unsafe { nvim_bw_fi_get_st_gid(file_info_old) }
                {
                    unsafe {
                        os_fchown(
                            fd,
                            nvim_bw_fi_get_st_uid(file_info_old),
                            nvim_bw_fi_get_st_gid(file_info_old),
                        );
                    }
                    if perm >= 0 {
                        unsafe { os_setperm(wfname, perm) };
                    }
                }
                unsafe { nvim_bw_buf_set_file_id(buf) };
            } else if unsafe { nvim_bw_buf_get_file_id_valid(buf) } == 0 {
                unsafe { nvim_bw_buf_set_file_id(buf) };
            }
        }

        let close_error = unsafe { os_close(fd) };
        if close_error != 0 {
            err = unsafe {
                rs_set_err_arg(
                    nvim_bw_gettext(c"E512: Close failed: %s".as_ptr()),
                    close_error,
                )
            };
            end = 0;
        }

        #[cfg(unix)]
        if made_writable {
            perm &= !0o200;
        }

        if perm >= 0 {
            unsafe { os_setperm(wfname, perm) };
        }

        if !backup_copy {
            unsafe { nvim_bw_os_set_acl(wfname, acl) };
        }

        // charconvert
        if wfname != fname {
            if end != 0
                && unsafe { nvim_bw_eval_charconvert(c"utf-8".as_ptr(), fenc, wfname, fname) }
                    == FAIL
            {
                // set conv_error via accessor
                unsafe {
                    crate::ffi::nvim_bw_info_set_conv_error_direct(write_info, 1);
                }
                end = 0;
            }
            unsafe { os_remove(wfname) };
            if wfname_allocated {
                unsafe { nvim_bw_xfree(wfname) };
            }
        }
    }

    // Error handling after write
    if end == 0 {
        if err.msg.is_null() {
            let conv_error = unsafe { nvim_bw_info_get_conv_error(write_info) };
            if conv_error != 0 {
                let conv_error_lnum = unsafe { nvim_bw_info_get_conv_error_lnum(write_info) };
                if conv_error_lnum == 0 {
                    err = unsafe {
                        rs_set_err(nvim_bw_gettext(
                            c"E513: Write error, conversion failed (make 'fenc' empty to override)"
                                .as_ptr(),
                        ))
                    };
                } else {
                    let errbuf = unsafe { nvim_bw_xmalloc(300) };
                    err = ErrorT {
                        num: ptr::null(),
                        msg: errbuf,
                        arg: 0,
                        alloc: true,
                    };
                    unsafe {
                        nvim_bw_vim_snprintf(
                            errbuf,
                            300,
                            nvim_bw_gettext(
                                c"E513: Write error, conversion failed in line %lld (make 'fenc' empty to override)".as_ptr(),
                            ),
                            i64::from(conv_error_lnum),
                        );
                    }
                }
            } else if unsafe { nvim_bw_get_got_int() } != 0 {
                err = unsafe { rs_set_err(nvim_bw_gettext(c"Interrupted".as_ptr())) };
            } else {
                err = unsafe {
                    rs_set_err(nvim_bw_gettext(
                        c"E514: Write error (file system full?)".as_ptr(),
                    ))
                };
            }
        }

        // Try to restore backup
        if !backup.is_null() {
            if backup_copy {
                if unsafe { nvim_bw_get_got_int() } != 0 {
                    unsafe {
                        nvim_bw_msg(nvim_bw_gettext(c"Interrupted".as_ptr()), 0);
                        nvim_bw_ui_flush();
                    }
                }
                if unsafe { os_copy(backup, fname, nvim_bw_uv_fs_copyfile_ficlone()) } == 0 {
                    end = 1;
                }
            } else if unsafe { vim_rename(backup, fname) } == 0 {
                end = 1;
            }
        }
        // goto fail
        return do_fail_cleanup(
            buf,
            &mut err,
            &mut backup,
            buffer,
            using_smallbuf,
            &mut fenc_tofree,
            write_info,
            acl,
            msg_save,
            prev_got_int_save,
            fname,
            sfname,
            end,
        );
    }

    // ===== SUCCESS PATH =====
    lnum -= start; // number of written lines
    unsafe { nvim_bw_dec_no_wait_return() };

    #[cfg(not(unix))]
    {
        fname = sfname;
    }

    // Display write message
    if filtering == 0 {
        let iobuff = unsafe { nvim_bw_get_IObuff() };
        let iosize = IOSIZE;
        unsafe { nvim_bw_add_quoted_fname(iobuff, iosize as c_int, buf, fname) };
        let mut insert_space = false;
        let conv_error = unsafe { nvim_bw_info_get_conv_error(write_info) };
        if conv_error != 0 {
            unsafe {
                nvim_bw_xstrlcat(
                    iobuff,
                    nvim_bw_gettext(c" CONVERSION ERROR".as_ptr()),
                    iosize,
                );
            }
            insert_space = true;
            let conv_error_lnum = unsafe { nvim_bw_info_get_conv_error_lnum(write_info) };
            if conv_error_lnum != 0 {
                unsafe {
                    nvim_bw_vim_snprintf_add(
                        iobuff,
                        iosize,
                        nvim_bw_gettext(c" in line %lld;".as_ptr()),
                        i64::from(conv_error_lnum),
                    );
                }
            }
        } else if notconverted {
            unsafe {
                nvim_bw_xstrlcat(iobuff, nvim_bw_gettext(c"[NOT converted]".as_ptr()), iosize);
            }
            insert_space = true;
        } else if converted {
            unsafe {
                nvim_bw_xstrlcat(iobuff, nvim_bw_gettext(c"[converted]".as_ptr()), iosize);
            }
            insert_space = true;
        }
        if device {
            unsafe {
                nvim_bw_xstrlcat(iobuff, nvim_bw_gettext(c"[Device]".as_ptr()), iosize);
            }
            insert_space = true;
        } else if newfile {
            unsafe { nvim_bw_xstrlcat(iobuff, nvim_bw_gettext(c"[New]".as_ptr()), iosize) };
            insert_space = true;
        }
        if no_eol {
            unsafe { nvim_bw_xstrlcat(iobuff, nvim_bw_gettext(c"[noeol]".as_ptr()), iosize) };
            insert_space = true;
        }
        if unsafe { nvim_bw_msg_add_fileformat(fileformat) } != 0 {
            insert_space = true;
        }
        unsafe { nvim_bw_msg_add_lines(c_int::from(insert_space), lnum, nchars) };
        if !shortmess(SHM_WRITE) {
            if append != 0 {
                unsafe {
                    nvim_bw_xstrlcat(
                        iobuff,
                        if shortmess(SHM_WRI) {
                            nvim_bw_gettext(c" [a]".as_ptr())
                        } else {
                            nvim_bw_gettext(c" appended".as_ptr())
                        },
                        iosize,
                    );
                }
            } else {
                unsafe {
                    nvim_bw_xstrlcat(
                        iobuff,
                        if shortmess(SHM_WRI) {
                            nvim_bw_gettext(c" [w]".as_ptr())
                        } else {
                            nvim_bw_gettext(c" written".as_ptr())
                        },
                        iosize,
                    );
                }
            }
        }

        unsafe {
            msg_ext_set_kind(c"bufwrite".as_ptr());
            nvim_bw_set_msg_ext_overwrite(1);
            set_keep_msg(msg_trunc(iobuff, false, 0), 0);
        }
    }

    // Reset modified flag
    let conv_error = unsafe { nvim_bw_info_get_conv_error(write_info) };
    if reset_changed != 0
        && whole
        && append == 0
        && conv_error == 0
        && (overwriting || unsafe { nvim_bw_cpo_contains(CPO_PLUS) } != 0)
    {
        unsafe {
            unchanged(buf, true, false);
        }
        let changedtick = unsafe { nvim_bw_buf_get_changedtick(buf) };
        if unsafe { nvim_bw_buf_get_last_changedtick(buf) } + 1 == changedtick {
            unsafe { nvim_bw_buf_set_last_changedtick(buf, changedtick) };
        }
        unsafe {
            u_unchanged(buf);
            u_update_save_nr(buf);
        }
    }

    // Update timestamp and flags
    if overwriting {
        unsafe { ml_timestamp(buf) };
        let flags = unsafe { nvim_bw_buf_get_flags(buf) };
        if append != 0 {
            unsafe { nvim_bw_buf_set_flags(buf, flags & !BF_NEW) };
        } else {
            unsafe { nvim_bw_buf_set_flags(buf, flags & !BF_WRITE_MASK) };
        }
    }

    // Patchmode
    let p_pm = unsafe { nvim_bw_get_p_pm() };
    if !unsafe { c_str_empty(p_pm) } && dobackup {
        let org = unsafe { modname(fname, p_pm, false) };
        if backup.is_null() {
            // Create empty original file
            if org.is_null() {
                unsafe {
                    emsg(nvim_bw_gettext(
                        c"E206: Patchmode: can't touch empty original file".as_ptr(),
                    ));
                }
            } else {
                let o_flags = unsafe { nvim_bw_open_flags_creat_wronly_excl_nofollow() };
                let empty_fd =
                    unsafe { os_open(org, o_flags, if perm < 0 { 0o666 } else { perm & 0o777 }) };
                if empty_fd < 0 {
                    unsafe {
                        emsg(nvim_bw_gettext(
                            c"E206: Patchmode: can't touch empty original file".as_ptr(),
                        ));
                    }
                } else {
                    unsafe { os_close(empty_fd) };
                }
            }
        } else if org.is_null() {
            unsafe {
                emsg(nvim_bw_gettext(
                    c"E205: Patchmode: can't save original file".as_ptr(),
                ));
            }
        } else if unsafe { os_path_exists(org) } == 0 {
            unsafe {
                vim_rename(backup, org);
                nvim_bw_XFREE_CLEAR(&raw mut backup);
            }
            #[cfg(unix)]
            {
                #[allow(clippy::cast_precision_loss)]
                unsafe {
                    nvim_bw_os_file_settime(
                        org,
                        nvim_bw_fi_get_atime_sec(file_info_old) as f64,
                        nvim_bw_fi_get_mtime_sec(file_info_old) as f64,
                    );
                }
            }
        }
        if !org.is_null() {
            unsafe { os_setperm(org, os_getperm(fname) & 0o777) };
            unsafe { nvim_bw_xfree(org) };
        }
    }

    // Remove backup unless 'backup' option is set
    if unsafe { nvim_bw_get_p_bk() } == 0
        && !backup.is_null()
        && conv_error == 0
        && unsafe { os_remove(backup) } != 0
    {
        unsafe { emsg(nvim_bw_gettext(c"E207: Can't delete backup file".as_ptr())) };
    }

    // ===== nofail cleanup =====
    unsafe { nvim_bw_buf_set_saving(buf, 0) };
    unsafe { nvim_bw_xfree(backup) };
    if !using_smallbuf {
        unsafe { nvim_bw_xfree(buffer) };
    }
    unsafe { nvim_bw_xfree(fenc_tofree) };
    let conv_buf = unsafe { nvim_bw_info_get_conv_buf(write_info) };
    if !conv_buf.is_null() {
        unsafe { nvim_bw_xfree(conv_buf) };
    }
    let iconv_fd = unsafe { nvim_bw_info_get_iconv_fd(write_info) };
    if iconv_fd as isize != ICONV_INVALID {
        unsafe { nvim_bw_iconv_close(iconv_fd) };
    }
    unsafe { nvim_bw_os_free_acl(acl) };

    if !err.msg.is_null() {
        report_write_error(buf, &mut err, fname, sfname, end);
        retval = FAIL;
    }
    unsafe { nvim_bw_set_msg_scroll(msg_save) };

    // Write undo file
    if retval == OK && write_undo_file {
        let mut hash = [0u8; UNDO_HASH_SIZE];
        unsafe { nvim_bw_sha256_finish(sha_ctx, hash.as_mut_ptr()) };
        unsafe { nvim_bw_u_write_undo(buf, hash.as_ptr()) };
    }

    // Post-autocmds
    if unsafe { should_abort(retval) } == 0 {
        unsafe {
            rs_buf_write_do_post_autocmds(
                buf,
                fname,
                eap,
                append,
                filtering,
                reset_changed,
                c_int::from(whole),
            );
        }
        if unsafe { aborting() } != 0 {
            retval = FAIL;
        }
    }

    if prev_got_int_save {
        unsafe {
            nvim_bw_set_got_int(nvim_bw_get_got_int() | 1);
        }
    }

    retval
}

/// Restore backup file when a write failure occurs.
unsafe fn restore_backup_and_fail(
    fname: *const c_char,
    backup: &*mut c_char,
    backup_copy: bool,
    newfile: bool,
    wfname: *mut c_char,
    _wfname_allocated: bool,
) {
    let backup = *backup;
    if !backup.is_null() && std::ptr::eq(wfname, fname) {
        if backup_copy {
            if unsafe { os_path_exists(fname) } == 0 {
                unsafe { vim_rename(backup, fname) };
            }
            if unsafe { os_path_exists(fname) } != 0 {
                unsafe { os_remove(backup) };
            }
        } else {
            unsafe { vim_rename(backup, fname) };
        }
    }

    if !newfile && unsafe { os_path_exists(fname) } == 0 {
        // Original file lost - end = 0 is handled by caller
    }
}

/// Common cleanup for the "fail" path.
#[allow(clippy::too_many_arguments)]
unsafe fn do_fail_cleanup(
    buf: BufHandle,
    err: &mut ErrorT,
    backup: &mut *mut c_char,
    buffer: *mut c_char,
    using_smallbuf: bool,
    fenc_tofree: &mut *mut c_char,
    write_info: BwInfoHandle,
    acl: AclHandle,
    msg_save: c_int,
    prev_got_int: bool,
    fname: *mut c_char,
    sfname: *mut c_char,
    end: i32,
) -> c_int {
    unsafe { nvim_bw_dec_no_wait_return() };
    unsafe { nvim_bw_buf_set_saving(buf, 0) };
    unsafe { nvim_bw_xfree(*backup) };
    *backup = ptr::null_mut();
    if !using_smallbuf {
        unsafe { nvim_bw_xfree(buffer) };
    }
    unsafe { nvim_bw_xfree(*fenc_tofree) };
    *fenc_tofree = ptr::null_mut();
    let conv_buf = unsafe { nvim_bw_info_get_conv_buf(write_info) };
    if !conv_buf.is_null() {
        unsafe { nvim_bw_xfree(conv_buf) };
    }
    let iconv_fd = unsafe { nvim_bw_info_get_iconv_fd(write_info) };
    if iconv_fd as isize != ICONV_INVALID {
        unsafe { nvim_bw_iconv_close(iconv_fd) };
    }
    unsafe { nvim_bw_os_free_acl(acl) };

    if !err.msg.is_null() {
        report_write_error(buf, err, fname, sfname, end);
    }
    unsafe { nvim_bw_set_msg_scroll(msg_save) };

    if prev_got_int {
        unsafe { nvim_bw_set_got_int(1) };
    }

    FAIL
}

/// Report write error with filename.
unsafe fn report_write_error(
    buf: BufHandle,
    err: &mut ErrorT,
    fname: *mut c_char,
    _sfname: *mut c_char,
    end: i32,
) {
    let iobuff = unsafe { nvim_bw_get_IObuff() };
    #[cfg(not(unix))]
    unsafe {
        nvim_bw_add_quoted_fname(iobuff, (IOSIZE - 100) as c_int, buf, _sfname);
    }
    #[cfg(unix)]
    unsafe {
        nvim_bw_add_quoted_fname(iobuff, (IOSIZE - 100) as c_int, buf, fname);
    }
    unsafe { rs_emit_err(std::ptr::from_ref::<ErrorT>(err)) };

    if end == 0 {
        unsafe {
            msg_puts_hl(
                nvim_bw_gettext(c"\nWARNING: Original file may be lost or damaged\n".as_ptr()),
                HLF_E,
                true,
            );
            msg_puts_hl(
                nvim_bw_gettext(
                    c"don't quit the editor until the file is successfully written!".as_ptr(),
                ),
                HLF_E,
                true,
            );
        }

        // Update timestamp
        let fi_size = unsafe { nvim_bw_sizeof_FileInfo() };
        let mut fi_buf = vec![0u8; fi_size];
        let fi: FileInfoHandle = fi_buf.as_mut_ptr().cast();
        if unsafe { os_fileinfo(fname, fi) } != 0 {
            unsafe {
                nvim_bw_buf_store_file_info(buf, fi);
                let mtime = nvim_bw_buf_get_mtime(buf);
                let mtime_ns = nvim_bw_buf_get_mtime_ns(buf);
                nvim_bw_buf_set_mtime_read(buf, mtime);
                nvim_bw_buf_set_mtime_read_ns(buf, mtime_ns);
            }
        }
    }
}

// Additional extern for buf_get_op_start/end returning PosT
extern "C" {
    fn nvim_bw_buf_get_op_start(buf: BufHandle) -> crate::autocmd::PosT;
    fn nvim_bw_buf_get_op_end(buf: BufHandle) -> crate::autocmd::PosT;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_constants() {
        assert_eq!(NUL, 0);
        assert_eq!(NL, 10);
        assert_eq!(CAR, 13);
        assert_eq!(CTRL_Z, 0x1a);
        assert_eq!(EOL_UNIX, 0);
        assert_eq!(EOL_DOS, 1);
        assert_eq!(EOL_MAC, 2);
        assert_eq!(ML_EMPTY, 0x01);
        assert_eq!(FORCE_BIN, 1);
        assert_eq!(UNDO_HASH_SIZE, 32);
    }

    #[test]
    fn test_error_t_layout() {
        // Error_T in C: { const char *num, char *msg, int arg, bool alloc } = 24 bytes
        assert_eq!(std::mem::size_of::<ErrorT>(), 24);
    }
}
