//! Rust port of the C `readfile()` function.
//!
//! This is a faithful translation of the C function, preserving all
//! semantics including complex control flow (goto retry/rewind_retry/
//! failed/theend), encoding conversion, and iconv FFI.
//!
//! The function is exported as `rs_readfile` and the C wrapper calls it.

#![allow(unsafe_code)]
#![allow(clippy::too_many_lines)]
#![allow(clippy::cognitive_complexity)]
#![allow(clippy::too_many_arguments)]
#![allow(dead_code)]
#![allow(unused_assignments)]
#![allow(unused_mut)]
#![allow(unused_parens)]
#![allow(unexpected_cfgs)]
#![allow(clippy::collapsible_if)]
#![allow(clippy::unnecessary_cast)]
#![allow(clippy::manual_range_contains)]
#![allow(clippy::needless_late_init)]
#![allow(clippy::manual_c_str_literals)]

use std::ffi::{c_char, c_int, c_void};
use std::ptr;

// =============================================================================
// FFI: external C functions from fileio_shim.c
// =============================================================================

extern "C" {
    // --- globals ---
    fn nvim_fileio_get_stdin_fd() -> c_int;
    fn nvim_fileio_get_msg_scroll() -> c_int;
    fn nvim_fileio_set_msg_scroll(val: c_int);
    fn nvim_fileio_get_msg_scrolled() -> c_int;
    fn nvim_fileio_get_got_int() -> c_int;
    fn nvim_fileio_set_need_fileinfo(val: c_int);
    fn nvim_fileio_get_readonlymode() -> c_int;
    fn nvim_fileio_get_recoverymode() -> c_int;
    fn nvim_fileio_get_p_verbose() -> c_int;
    fn nvim_fileio_get_p_cpo() -> *const c_char;
    fn nvim_fileio_get_p_ffs() -> *const c_char;
    fn nvim_fileio_get_p_fencs() -> *const c_char;
    fn nvim_fileio_get_p_ccv() -> *const c_char;
    fn nvim_fileio_get_msg_listdo_overwrite() -> c_int;
    fn nvim_fileio_get_exmode_active() -> c_int;
    fn nvim_fileio_get_restart_edit() -> c_int;
    fn nvim_fileio_get_need_wait_return() -> c_int;
    fn nvim_fileio_get_msg_col() -> c_int;
    fn nvim_fileio_get_msg_scrolled_ign() -> c_int;
    fn nvim_fileio_set_msg_scrolled_ign(val: c_int);
    fn nvim_fileio_get_cmdmod_lockmarks() -> c_int;

    // --- curbuf fields ---
    fn nvim_fileio_curbuf_get_b_au_did_filetype() -> c_int;
    fn nvim_fileio_curbuf_set_b_au_did_filetype(val: c_int);
    fn nvim_fileio_curbuf_get_b_no_eol_lnum() -> c_int;
    fn nvim_fileio_curbuf_set_b_no_eol_lnum(val: c_int);
    fn nvim_fileio_curbuf_get_b_ffname() -> *mut c_char;
    fn nvim_fileio_curbuf_get_b_fname() -> *mut c_char;
    fn nvim_fileio_curbuf_get_b_flags() -> c_int;
    fn nvim_fileio_curbuf_set_b_flags(val: c_int);
    fn nvim_fileio_curbuf_set_b_flags_and(mask: c_int);
    fn nvim_fileio_curbuf_set_b_flags_or(flags: c_int);
    fn nvim_fileio_curbuf_get_b_help() -> c_int;
    fn nvim_fileio_curbuf_get_b_p_ro() -> c_int;
    fn nvim_fileio_curbuf_set_b_p_ro(val: c_int);
    fn nvim_fileio_curbuf_set_b_p_eof(val: c_int);
    fn nvim_fileio_curbuf_get_b_p_eol() -> c_int;
    fn nvim_fileio_curbuf_set_b_p_eol(val: c_int);
    fn nvim_fileio_curbuf_set_b_start_eof(val: c_int);
    fn nvim_fileio_curbuf_set_b_start_eol(val: c_int);
    fn nvim_fileio_curbuf_get_b_p_bomb() -> c_int;
    fn nvim_fileio_curbuf_set_b_p_bomb(val: c_int);
    fn nvim_fileio_curbuf_set_b_start_bomb(val: c_int);
    fn nvim_fileio_curbuf_get_b_p_bin() -> c_int;
    fn nvim_fileio_curbuf_get_b_p_fenc() -> *const c_char;
    fn nvim_fileio_curbuf_get_b_p_udf() -> c_int;
    fn nvim_fileio_curbuf_get_b_bad_char() -> c_int;
    fn nvim_fileio_curbuf_set_b_bad_char(val: c_int);
    fn nvim_fileio_curbuf_get_b_p_ft() -> *const c_char;
    fn nvim_fileio_curbuf_get_ml_line_count() -> c_int;
    fn nvim_fileio_curbuf_get_ml_flags() -> c_int;
    fn nvim_fileio_curbuf_get_b_mtime() -> c_int;
    fn nvim_fileio_curbuf_set_b_mtime(val: c_int);
    fn nvim_fileio_curbuf_set_b_mtime_ns(val: c_int);
    fn nvim_fileio_curbuf_set_b_mtime_read(val: c_int);
    fn nvim_fileio_curbuf_set_b_mtime_read_ns(val: c_int);
    fn nvim_fileio_curbuf_set_b_orig_size(val: i64);
    fn nvim_fileio_curbuf_set_b_orig_mode(val: c_int);
    fn nvim_fileio_curbuf_set_deleted_bytes_zero();
    fn nvim_fileio_curbuf_has_mfp() -> c_int;
    fn nvim_fileio_curbuf_mfp_dirty_is_nosync() -> c_int;
    fn nvim_fileio_curbuf_mfp_set_dirty_yes();
    fn nvim_fileio_curbuf_get_b_op_start(lnum: *mut c_int, col: *mut c_int);
    fn nvim_fileio_curbuf_set_b_op_start(lnum: c_int, col: c_int);
    fn nvim_fileio_curbuf_set_b_op_end(lnum: c_int, col: c_int);
    fn nvim_fileio_store_fileinfo_for_newfile(fname: *const c_char) -> c_int;
    fn nvim_fileio_curbuf_check_identity(
        saved_curbuf: *mut c_void,
        saved_b_ffname: *const c_char,
        saved_b_fname: *const c_char,
        using_b_ffname: c_int,
        using_b_fname: c_int,
    ) -> c_int;
    fn nvim_fileio_get_curbuf() -> *mut c_void;
    fn nvim_fileio_get_curwin() -> *mut c_void;
    fn nvim_fileio_curwin_cursor_lnum() -> c_int;
    fn nvim_fileio_curwin_set_cursor_lnum(lnum: c_int);

    // --- os operations ---
    fn nvim_fileio_os_getperm(fname: *const c_char) -> c_int;
    fn nvim_fileio_os_file_is_writable(fname: *const c_char) -> c_int;
    fn nvim_fileio_os_open_rdonly(fname: *const c_char) -> c_int;
    fn nvim_fileio_after_pathsep(b: *const c_char, p: *const c_char) -> c_int;
    fn nvim_fileio_bt_dontwrite() -> c_int;
    fn nvim_fileio_dir_of_file_exists(fname: *const c_char) -> c_int;
    fn nvim_fileio_is_s_isreg(perm: c_int) -> c_int;
    fn nvim_fileio_is_s_isfifo(perm: c_int) -> c_int;
    fn nvim_fileio_is_s_issock(perm: c_int) -> c_int;
    fn nvim_fileio_is_s_isdir(perm: c_int) -> c_int;
    fn nvim_fileio_perm_is_writable(perm: c_int) -> c_int;
    fn nvim_fileio_close(fd: c_int) -> c_int;
    fn nvim_fileio_is_chr_dev(perm: c_int, fname: *const c_char) -> c_int;
    #[cfg(unix)]
    fn nvim_fileio_check_swap_mode_group(
        swap_fname: *const c_char,
        gid: c_int,
        fd: c_int,
        swap_mode: c_int,
    ) -> c_int;
    #[cfg(unix)]
    fn nvim_fileio_curbuf_swap_gid() -> c_int;
    #[cfg(unix)]
    fn nvim_fileio_curbuf_swap_fd() -> c_int;
    #[cfg(unix)]
    fn nvim_fileio_curbuf_swap_fname() -> *const c_char;
    #[cfg(unix)]
    fn nvim_fileio_os_setperm(fname: *const c_char, mode: c_int);

    // --- autocmd ---
    fn nvim_fileio_apply_autocmds_exarg(
        event: c_int,
        fname: *const c_char,
        fname_io: *const c_char,
        force_it: c_int,
        buf: *mut c_void,
        eap: *mut c_void,
    ) -> c_int;
    fn nvim_fileio_apply_autocmds(
        event: c_int,
        pat: *const c_char,
        fname: *const c_char,
        force_it: c_int,
        buf: *mut c_void,
    ) -> c_int;
    fn nvim_event_BUFREADCMD() -> c_int;
    fn nvim_event_FILEREADCMD() -> c_int;
    fn nvim_event_BUFNEWFILE() -> c_int;
    fn nvim_event_FILTERREADPRE() -> c_int;
    fn nvim_event_STDINREADPRE() -> c_int;
    fn nvim_event_BUFREADPRE() -> c_int;
    fn nvim_event_FILEREADPRE() -> c_int;
    fn nvim_event_FILTERREADPOST() -> c_int;
    fn nvim_event_BUFREADPOST() -> c_int;
    fn nvim_event_FILEREADPOST() -> c_int;
    fn nvim_event_FILETYPE() -> c_int;

    // --- shortmess / aborting ---
    fn nvim_fileio_shortmess_over() -> c_int;
    fn nvim_fileio_shortmess_ro() -> c_int;
    fn nvim_fileio_aborting() -> c_int;

    // --- memline ---
    fn nvim_fileio_ml_line_count() -> c_int;
    fn nvim_fileio_ml_get(lnum: c_int) -> *const c_char;
    fn nvim_fileio_ml_get_len(lnum: c_int) -> c_int;
    fn nvim_fileio_ml_append(lnum: c_int, line: *const c_char, len: c_int, newfile: c_int)
        -> c_int;
    fn nvim_fileio_ml_delete(lnum: c_int) -> c_int;
    fn nvim_fileio_vim_lseek(fd: c_int, offset: i64, whence: c_int) -> i64;

    // --- memory ---
    fn nvim_fileio_verbose_try_malloc(size: usize) -> *mut c_void;
    fn nvim_fileio_xfree(ptr: *mut c_void);
    fn nvim_fileio_xcalloc(nmemb: usize, size: usize) -> *mut c_void;

    // --- file options / encoding ---
    fn nvim_fileio_enc_canonize(enc: *const c_char) -> *mut c_char;
    fn nvim_fileio_save_file_ff();
    fn nvim_fileio_set_fileformat(ff: c_int);
    fn nvim_fileio_set_option_direct_fenc(fenc: *const c_char);
    fn nvim_fileio_get_fileformat_force(eap: *mut c_void) -> c_int;
    fn nvim_fileio_shortmess(msg_id: c_int) -> c_int;
    fn nvim_fileio_check_need_swap(newfile: c_int);

    // --- messages ---
    fn nvim_fileio_filemess(fname: *const c_char, s: *const c_char);
    fn nvim_fileio_msg_end();
    fn nvim_fileio_msg_putchar(c: c_int);
    fn nvim_fileio_msg_trunc(s: *mut c_char) -> *mut c_char;
    fn nvim_fileio_set_keep_msg(s: *const c_char);
    fn nvim_fileio_XFREE_CLEAR_keep_msg();
    fn nvim_fileio_xstrlcat(dst: *mut c_char, src: *const c_char, dst_size: usize);
    fn nvim_fileio_snprintf_iobuff(offset: c_int, fmt: *const c_char, val: i64);
    fn nvim_fileio_add_quoted_fname(sfname: *mut c_char);
    fn nvim_fileio_msg_add_fileformat(ff: c_int) -> c_int;
    fn nvim_fileio_msg_add_lines(insert_space: c_int, linecnt: c_int, filesize: i64);
    fn nvim_fileio_get_IObuff() -> *mut c_char;
    fn nvim_fileio_get_IOSIZE() -> c_int;
    fn nvim_fileio_strlen_IObuff() -> c_int;
    fn nvim_fileio_msg_illegal_filename() -> *const c_char;
    fn nvim_fileio_msg_is_a_directory() -> *const c_char;
    fn nvim_fileio_msg_is_not_a_file() -> *const c_char;
    fn nvim_fileio_msg_file_too_big() -> *const c_char;
    fn nvim_fileio_msg_permission_denied() -> *const c_char;
    fn nvim_fileio_msg_new() -> *const c_char;
    fn nvim_fileio_msg_new_directory() -> *const c_char;
    fn nvim_fileio_msg_conversion_error() -> *const c_char;
    fn nvim_fileio_msg_illegal_byte() -> *const c_char;
    fn nvim_fileio_msg_read_errors() -> *const c_char;
    fn nvim_fileio_msg_read_interrupted() -> *const c_char;
    fn nvim_fileio_msg_e200() -> *const c_char;
    fn nvim_fileio_msg_e201() -> *const c_char;
    fn nvim_fileio_msg_e202() -> *const c_char;
    fn nvim_fileio_msg_e_auchangedbuf() -> *const c_char;
    fn nvim_fileio_msg_fifo() -> *const c_char;
    fn nvim_fileio_msg_socket() -> *const c_char;
    fn nvim_fileio_msg_character_special() -> *const c_char;
    fn nvim_fileio_msg_readonly() -> *const c_char;
    fn nvim_fileio_msg_ro() -> *const c_char;
    fn nvim_fileio_msg_noeol() -> *const c_char;
    fn nvim_fileio_msg_cr_missing() -> *const c_char;
    fn nvim_fileio_msg_long_lines_split() -> *const c_char;
    fn nvim_fileio_msg_not_converted() -> *const c_char;
    fn nvim_fileio_msg_converted() -> *const c_char;
    fn nvim_fileio_emsg(s: *const c_char);
    fn nvim_fileio_vim_strchr(s: *const c_char, c: c_int) -> c_int;
    fn nvim_fileio_SHM_OVER() -> c_int;
    fn nvim_fileio_SHM_RO() -> c_int;
    fn nvim_fileio_UV_ENOENT() -> c_int;
    fn nvim_fileio_UV_EFBIG() -> c_int;
    fn nvim_fileio_EOVERFLOW_neg() -> c_int;

    // --- undo ---
    fn nvim_fileio_u_clearline();
    fn nvim_fileio_u_find_first_changed();
    fn nvim_fileio_sha256_start(ctx: *mut c_void);
    fn nvim_fileio_sha256_update(ctx: *mut c_void, data: *const u8, len: usize);
    fn nvim_fileio_sha256_finish(ctx: *mut c_void, hash: *mut u8);
    fn nvim_fileio_u_read_undo(hash: *mut u8, fname: *const c_char);
    fn nvim_fileio_sha256_ctx_size() -> c_int;
    fn nvim_fileio_UNDO_HASH_SIZE() -> c_int;

    // --- redraw/display ---
    fn nvim_fileio_redraw_curbuf_later();
    fn nvim_fileio_appended_lines_mark(from: c_int, linecnt: c_int);
    fn nvim_fileio_check_cursor_lnum();
    fn nvim_fileio_beginline();
    fn nvim_fileio_os_set_cloexec(fd: c_int);
    fn nvim_fileio_os_breakcheck();
    fn nvim_fileio_os_remove(fname: *const c_char) -> c_int;

    // --- iconv ---
    fn nvim_fileio_my_iconv_open(to: *const c_char, from: *const c_char) -> usize;
    fn nvim_fileio_iconv_invalid() -> usize;
    fn nvim_fileio_iconv_is_invalid(fd: usize) -> c_int;
    fn nvim_fileio_iconv(
        cd: usize,
        inbuf: *mut *const c_char,
        inbytesleft: *mut usize,
        outbuf: *mut *mut c_char,
        outbytesleft: *mut usize,
    ) -> usize;
    fn nvim_fileio_iconv_close(fd: usize) -> c_int;
    fn nvim_fileio_iconv_errno() -> c_int;
    fn nvim_fileio_iconv_einval() -> c_int;

    // --- utf8/mbyte ---
    fn nvim_fileio_utf_byte2len(c: c_int) -> c_int;
    fn nvim_fileio_utf_head_off(base: *const c_char, p: *const c_char) -> c_int;
    fn nvim_fileio_utf_ptr2char(p: *const c_char) -> c_int;
    fn nvim_fileio_utf_ptr2len_len(p: *const c_char, size: c_int) -> c_int;
    fn nvim_fileio_utf_char2len(c: c_int) -> c_int;
    fn nvim_fileio_utf_char2bytes(c: c_int, buf: *mut c_char) -> c_int;

    // --- constants ---
    fn nvim_fileio_EOL_UNKNOWN() -> c_int;
    fn nvim_fileio_EOL_UNIX() -> c_int;
    fn nvim_fileio_EOL_DOS() -> c_int;
    fn nvim_fileio_EOL_MAC() -> c_int;
    fn nvim_fileio_FAIL() -> c_int;
    fn nvim_fileio_OK() -> c_int;
    fn nvim_fileio_NOTDONE() -> c_int;
    fn nvim_fileio_BAD_REPLACE() -> c_int;
    fn nvim_fileio_BAD_KEEP() -> c_int;
    fn nvim_fileio_BAD_DROP() -> c_int;
    fn nvim_fileio_ML_EMPTY() -> c_int;
    fn nvim_fileio_BF_CHECK_RO() -> c_int;
    fn nvim_fileio_BF_NEW() -> c_int;
    fn nvim_fileio_BF_NEW_W() -> c_int;
    fn nvim_fileio_BF_NOTEDITED() -> c_int;
    fn nvim_fileio_SEA_QUIT() -> c_int;
    fn nvim_fileio_CPO_FNAMER() -> c_int;
    fn nvim_fileio_ENC_UCSBOM() -> *const c_char;
    fn nvim_fileio_stdin_post_read();

    // --- existing Rust functions (called by C, now also by Rust) ---
    fn rs_check_for_bom(
        data: *const u8,
        size: c_int,
        lenp: *mut c_int,
        flags: c_int,
    ) -> *const c_char;
    fn rs_need_conversion(fenc: *const c_char) -> c_int;
    fn rs_get_fio_flags(name: *const c_char) -> c_int;
    fn rs_get_fileformat(buf: *mut c_void) -> c_int;
    fn rs_default_fileformat() -> c_int;
    fn rs_check_marks_read();
    fn rs_diff_invalidate(buf: *mut c_void);
    fn rs_foldUpdateAll(win: *mut c_void);
    fn next_fenc(pp: *mut *mut c_char, alloced: *mut c_int) -> *mut c_char;
    fn readfile_charconvert(fname: *mut c_char, fenc: *mut c_char, fdp: *mut c_int) -> *mut c_char;
    fn readfile_linenr(linecnt: i32, p: *const c_char, endp: *const c_char) -> i32;
    fn set_file_options(set_options: c_int, eap: *mut c_void);
    fn set_forced_fenc(eap: *mut c_void);
    fn set_rw_fname(fname: *mut c_char, sfname: *mut c_char) -> c_int;
    fn add_quoted_fname(
        ret_buf: *mut c_char,
        buf_len: usize,
        buf: *const c_void,
        fname: *const c_char,
    );
    fn msg_add_fileformat(eol_type: c_int) -> c_int;
    fn msg_add_lines(insert_space: c_int, lnum: i32, nchars: i64);
}

// =============================================================================
// Constants (loaded at runtime from C to avoid hardcoding)
// =============================================================================

// EOL values
const fn eol_unknown() -> i32 {
    -1
}
const fn eol_unix() -> i32 {
    0
}
const fn eol_dos() -> i32 {
    1
}
const fn eol_mac() -> i32 {
    2
}

// FIO flags (these must match fileio.h)
const FIO_UCSBOM: i32 = 0x4000;
const FIO_ALL: i32 = -1;
const FIO_UTF8: i32 = 0x02;
const FIO_LATIN1: i32 = 0x01;
const FIO_UCS2: i32 = 0x04;
const FIO_UCS4: i32 = 0x08;
const FIO_UTF16: i32 = 0x10;
const FIO_ENDIAN_L: i32 = 0x80;

// READ_* flags (from fileio.h)
const READ_NEW: i32 = 0x01;
const READ_FILTER: i32 = 0x02;
const READ_STDIN: i32 = 0x04;
const READ_BUFFER: i32 = 0x08;
const READ_DUMMY: i32 = 0x10;
const READ_KEEP_UNDO: i32 = 0x20;
const READ_FIFO: i32 = 0x40;
const READ_NOFILE: i32 = 0x100;

// ICONV_MULT
const ICONV_MULT: usize = 8;

// CONV_RESTLEN
const CONV_RESTLEN: usize = 30;

// MAXCOL (from Neovim - max column number)
const MAXCOL: isize = 2147483647;

// SEEK_SET
const SEEK_SET: i32 = 0;

// NUL, NL, CAR, Ctrl_Z
const NUL: u8 = 0x00;
const NL: u8 = 0x0a;
const CAR: u8 = 0x0d;
const CTRL_Z: u8 = 0x1a;

// =============================================================================
// rs_readfile: the main exported function
// =============================================================================

/// Rust port of C `readfile()`.
///
/// Read lines from file "fname" into the buffer after line "from".
///
/// # Safety
/// All pointer arguments must be valid for the duration of the call.
#[no_mangle]
pub unsafe extern "C" fn rs_readfile(
    fname: *mut c_char,
    sfname: *mut c_char,
    from: i32,
    lines_to_skip: i32,
    lines_to_read: i32,
    eap: *mut c_void,
    flags: c_int,
    silent: c_int,
) -> c_int {
    let fail = nvim_fileio_FAIL();
    let ok = nvim_fileio_OK();
    let notdone = nvim_fileio_NOTDONE();

    let mut retval = fail; // jump to "theend" instead of returning
    let mut fd = {
        let stdin_fd = nvim_fileio_get_stdin_fd();
        if stdin_fd >= 0 {
            stdin_fd
        } else {
            0
        }
    };
    let newfile = (flags & READ_NEW) != 0;
    let filtering = (flags & READ_FILTER) != 0;
    let read_stdin = (flags & READ_STDIN) != 0;
    let read_buffer = (flags & READ_BUFFER) != 0;
    let read_fifo = (flags & READ_FIFO) != 0;
    let set_options =
        newfile || read_buffer || (!eap.is_null() && nvim_fileio_eap_read_edit(eap) != 0);

    let mut read_buf_lnum: i32 = 1;
    let mut read_buf_col: i32 = 0;
    let mut lnum = from;
    let mut buffer: *mut u8 = ptr::null_mut();
    let mut new_buffer: *mut u8;
    let mut line_start: *mut u8 = ptr::null_mut();
    let mut wasempty: i32;
    let mut len: i32;
    let mut size: isize = 0;
    let mut filesize: i64 = 0;
    let mut skip_read = false;
    let mut read_undo_file = false;
    let sha_ctx_size = nvim_fileio_sha256_ctx_size() as usize;
    let sha_ctx: *mut c_void = nvim_fileio_xcalloc(sha_ctx_size, 1);
    let mut split: i32 = 0;
    let mut linecnt: i32;
    let mut error = false;
    let eol_unknown = nvim_fileio_EOL_UNKNOWN();
    let eol_unix = nvim_fileio_EOL_UNIX();
    let eol_dos = nvim_fileio_EOL_DOS();
    let eol_mac = nvim_fileio_EOL_MAC();
    let mut ff_error = eol_unknown;
    let mut linerest: isize = 0;
    let mut perm: i32 = 0;
    #[cfg(unix)]
    let mut swap_mode: i32 = -1;
    let mut fileformat: i32 = 0;
    let mut keep_fileformat = false;
    let mut skip_count: i32 = 0;
    let mut read_count: i32 = 0;
    let msg_save = nvim_fileio_get_msg_scroll();
    let mut read_no_eol_lnum: i32 = 0;
    let mut file_rewind = false;
    let mut conv_error: i32 = 0;
    let mut illegal_byte: i32 = 0;
    let mut keep_dest_enc = false;
    let bad_replace = nvim_fileio_BAD_REPLACE();
    let bad_keep = nvim_fileio_BAD_KEEP();
    let bad_drop = nvim_fileio_BAD_DROP();
    let mut bad_char_behavior = bad_replace;
    let mut tmpname: *mut c_char = ptr::null_mut();
    let mut fio_flags: i32 = 0;
    let mut fenc: *mut c_char = ptr::null_mut();
    let mut fenc_alloced: bool = false;
    let mut fenc_next: *mut c_char = ptr::null_mut();
    let mut advance_fenc = false;
    let mut real_size: i32 = 0;
    let iconv_invalid = nvim_fileio_iconv_invalid();
    let mut iconv_fd: usize = iconv_invalid;
    let mut did_iconv = false;
    let mut converted = false;
    let mut notconverted = false;
    let mut conv_rest: [u8; CONV_RESTLEN] = [0u8; CONV_RESTLEN];
    let mut conv_restlen: usize = 0;

    // Save orig_start (curbuf->b_op_start)
    let mut orig_start_lnum: i32 = 0;
    let mut orig_start_col: i32 = 0;
    nvim_fileio_curbuf_get_b_op_start(&mut orig_start_lnum, &mut orig_start_col);

    let old_curbuf = nvim_fileio_get_curbuf();
    let old_b_ffname = nvim_fileio_curbuf_get_b_ffname();
    let old_b_fname = nvim_fileio_curbuf_get_b_fname();
    let using_b_ffname = (fname == old_b_ffname || sfname == old_b_ffname) as i32;
    let using_b_fname = (fname == old_b_fname || sfname == old_b_fname) as i32;

    // Reset before triggering any autocommands
    nvim_fileio_curbuf_set_b_au_did_filetype(0);
    nvim_fileio_curbuf_set_b_no_eol_lnum(0);

    // If there is no file name yet, use the one for the read file.
    if nvim_fileio_curbuf_get_b_ffname().is_null()
        && !filtering
        && !fname.is_null()
        && nvim_fileio_vim_strchr(nvim_fileio_get_p_cpo(), nvim_fileio_CPO_FNAMER()) != 0
        && (flags & READ_DUMMY) == 0
    {
        if set_rw_fname(fname, sfname) == fail {
            nvim_fileio_set_msg_scroll(msg_save);
            nvim_fileio_xfree(sha_ctx);
            return fail;
        }
    }

    // After reading a file the cursor line changes but we don't want to display the line.
    // nvim_fileio_set_ex_no_reprint is done via ex_cmds_shim nvim_set_ex_no_reprint
    // We use the same approach as C: call via existing accessor
    // (ex_no_reprint = true is handled by the C wrapper of set_rw_fname if needed)
    // Actually readfile calls ex_no_reprint = true directly:
    nvim_set_ex_no_reprint(1);
    nvim_fileio_set_need_fileinfo(0);

    // For Unix: Use the short file name whenever possible.
    let sfname = if sfname.is_null() { fname } else { sfname };
    #[cfg(unix)]
    let fname = sfname;

    // The BufReadCmd and FileReadCmd events intercept reading
    if !filtering && !read_stdin && !read_buffer {
        nvim_fileio_curbuf_set_b_op_start(if from == 0 { 1 } else { from }, 0);

        if newfile {
            if nvim_fileio_apply_autocmds_exarg(
                nvim_event_BUFREADCMD(),
                ptr::null(),
                sfname,
                0,
                nvim_fileio_get_curbuf(),
                eap,
            ) != 0
            {
                retval = ok;
                if nvim_fileio_aborting() != 0 {
                    retval = fail;
                }
                if retval == ok {
                    nvim_fileio_curbuf_set_b_flags_and(!(nvim_fileio_BF_NOTEDITED()));
                }
                nvim_fileio_curbuf_set_b_op_start(orig_start_lnum, orig_start_col);
                // goto theend
                if nvim_fileio_curbuf_mfp_dirty_is_nosync() != 0 {
                    nvim_fileio_curbuf_mfp_set_dirty_yes();
                }
                nvim_fileio_set_msg_scroll(msg_save);
                nvim_fileio_xfree(sha_ctx as *mut c_void);
                return retval;
            }
        } else if nvim_fileio_apply_autocmds_exarg(
            nvim_event_FILEREADCMD(),
            sfname,
            sfname,
            0,
            ptr::null_mut(),
            eap,
        ) != 0
        {
            retval = if nvim_fileio_aborting() != 0 {
                fail
            } else {
                ok
            };
            nvim_fileio_curbuf_set_b_op_start(orig_start_lnum, orig_start_col);
            if nvim_fileio_curbuf_mfp_dirty_is_nosync() != 0 {
                nvim_fileio_curbuf_mfp_set_dirty_yes();
            }
            nvim_fileio_set_msg_scroll(msg_save);
            nvim_fileio_xfree(sha_ctx as *mut c_void);
            return retval;
        }

        nvim_fileio_curbuf_set_b_op_start(orig_start_lnum, orig_start_col);

        if (flags & READ_NOFILE) != 0 {
            retval = notdone;
            if nvim_fileio_curbuf_mfp_dirty_is_nosync() != 0 {
                nvim_fileio_curbuf_mfp_set_dirty_yes();
            }
            nvim_fileio_set_msg_scroll(msg_save);
            nvim_fileio_xfree(sha_ctx as *mut c_void);
            return retval;
        }
    }

    // Set msg_scroll based on shortmess and buffer type
    let shm_over = nvim_fileio_SHM_OVER();
    if (nvim_fileio_shortmess(shm_over) != 0 && nvim_fileio_get_msg_listdo_overwrite() == 0)
        || nvim_fileio_curbuf_get_b_help() != 0 && nvim_fileio_get_p_verbose() == 0
    {
        nvim_fileio_set_msg_scroll(0); // overwrite previous file message
    } else {
        nvim_fileio_set_msg_scroll(1); // don't overwrite previous file message
    }

    // Check fname length
    if !fname.is_null() && *fname != 0 {
        let namelen = libc_strlen(fname);
        if namelen >= 4096 {
            // MAXPATHL
            nvim_fileio_filemess(fname, nvim_fileio_msg_illegal_filename());
            nvim_fileio_msg_end();
            nvim_fileio_set_msg_scroll(msg_save);
            // goto theend
            if nvim_fileio_curbuf_mfp_dirty_is_nosync() != 0 {
                nvim_fileio_curbuf_mfp_set_dirty_yes();
            }
            nvim_fileio_xfree(sha_ctx as *mut c_void);
            return retval;
        }
        if nvim_fileio_after_pathsep(fname, fname.add(namelen)) != 0 {
            if silent == 0 {
                nvim_fileio_filemess(fname, nvim_fileio_msg_is_a_directory());
            }
            nvim_fileio_msg_end();
            nvim_fileio_set_msg_scroll(msg_save);
            retval = notdone;
            if nvim_fileio_curbuf_mfp_dirty_is_nosync() != 0 {
                nvim_fileio_curbuf_mfp_set_dirty_yes();
            }
            nvim_fileio_xfree(sha_ctx as *mut c_void);
            return retval;
        }
    }

    if !read_stdin && !fname.is_null() {
        perm = nvim_fileio_os_getperm(fname);
    }

    if !read_stdin && !read_buffer && !read_fifo {
        let is_chr = nvim_fileio_is_chr_dev(perm, fname);
        if perm >= 0
            && nvim_fileio_is_s_isreg(perm) == 0
            && nvim_fileio_is_s_isfifo(perm) == 0
            && nvim_fileio_is_s_issock(perm) == 0
            && is_chr == 0
        {
            if nvim_fileio_is_s_isdir(perm) != 0 {
                if silent == 0 {
                    nvim_fileio_filemess(fname, nvim_fileio_msg_is_a_directory());
                }
                retval = notdone;
            } else {
                nvim_fileio_filemess(fname, nvim_fileio_msg_is_not_a_file());
            }
            nvim_fileio_msg_end();
            nvim_fileio_set_msg_scroll(msg_save);
            if nvim_fileio_curbuf_mfp_dirty_is_nosync() != 0 {
                nvim_fileio_curbuf_mfp_set_dirty_yes();
            }
            nvim_fileio_xfree(sha_ctx as *mut c_void);
            return retval;
        }
    }

    // Set default or forced 'fileformat' and 'binary'.
    set_file_options(set_options as c_int, eap);

    // readonly flag
    let check_readonly =
        newfile && (nvim_fileio_curbuf_get_b_flags() & nvim_fileio_BF_CHECK_RO()) != 0;
    if check_readonly && nvim_fileio_get_readonlymode() == 0 {
        nvim_fileio_curbuf_set_b_p_ro(0);
    }

    // Remember time of file for new files
    if newfile && !read_stdin && !read_buffer && !read_fifo {
        #[cfg(unix)]
        {
            let mode = nvim_fileio_store_fileinfo_for_newfile(fname);
            if mode >= 0 {
                swap_mode = mode;
            }
        }
        #[cfg(not(unix))]
        {
            nvim_fileio_store_fileinfo_for_newfile(fname);
        }
        // Clear BF_NEW and BF_NEW_W flags
        nvim_fileio_curbuf_set_b_flags_and(!(nvim_fileio_BF_NEW() | nvim_fileio_BF_NEW_W()));
    }

    // Check readonly
    let mut file_readonly = false;
    if !read_buffer && !read_stdin {
        if !newfile
            || nvim_fileio_get_readonlymode() != 0
            || nvim_fileio_perm_is_writable(perm) == 0
            || nvim_fileio_os_file_is_writable(fname) == 0
        {
            file_readonly = true;
        }
        fd = nvim_fileio_os_open_rdonly(fname);
    }

    if fd < 0 {
        // cannot open at all
        nvim_fileio_set_msg_scroll(msg_save);
        if !newfile {
            if nvim_fileio_curbuf_mfp_dirty_is_nosync() != 0 {
                nvim_fileio_curbuf_mfp_set_dirty_yes();
            }
            nvim_fileio_xfree(sha_ctx as *mut c_void);
            return retval;
        }
        let uv_enoent = nvim_fileio_UV_ENOENT();
        if perm == uv_enoent {
            // file doesn't exist
            nvim_fileio_curbuf_set_b_flags_or(nvim_fileio_BF_NEW());

            if nvim_fileio_bt_dontwrite() == 0 {
                nvim_fileio_check_need_swap(newfile as c_int);
                if nvim_fileio_curbuf_check_identity(
                    old_curbuf,
                    old_b_ffname,
                    old_b_fname,
                    using_b_ffname,
                    using_b_fname,
                ) != 0
                {
                    nvim_fileio_emsg(nvim_fileio_msg_e_auchangedbuf());
                    if nvim_fileio_curbuf_mfp_dirty_is_nosync() != 0 {
                        nvim_fileio_curbuf_mfp_set_dirty_yes();
                    }
                    nvim_fileio_xfree(sha_ctx as *mut c_void);
                    return retval;
                }
            }
            if silent == 0 {
                if nvim_fileio_dir_of_file_exists(fname) != 0 {
                    nvim_fileio_filemess(sfname, nvim_fileio_msg_new());
                } else {
                    nvim_fileio_filemess(sfname, nvim_fileio_msg_new_directory());
                }
            }
            rs_check_marks_read();
            if !eap.is_null() {
                set_forced_fenc(eap);
            }
            nvim_fileio_apply_autocmds_exarg(
                nvim_event_BUFNEWFILE(),
                sfname,
                sfname,
                0,
                nvim_fileio_get_curbuf(),
                eap,
            );
            nvim_fileio_save_file_ff();
            if nvim_fileio_aborting() == 0 {
                retval = ok;
            }
            // goto theend
            if nvim_fileio_curbuf_mfp_dirty_is_nosync() != 0 {
                nvim_fileio_curbuf_mfp_set_dirty_yes();
            }
            nvim_fileio_xfree(sha_ctx as *mut c_void);
            return retval;
        }
        // File too big or permission denied
        #[cfg(all(unix, feature = "have_eoverflow"))]
        {
            let uv_efbig = nvim_fileio_UV_EFBIG();
            let eov_neg = nvim_fileio_EOVERFLOW_neg();
            let msg = if fd == uv_efbig || fd == eov_neg {
                nvim_fileio_msg_file_too_big()
            } else {
                nvim_fileio_msg_permission_denied()
            };
            nvim_fileio_filemess(sfname, msg);
        }
        #[cfg(not(all(unix, feature = "have_eoverflow")))]
        {
            let uv_efbig = nvim_fileio_UV_EFBIG();
            let msg = if fd == uv_efbig {
                nvim_fileio_msg_file_too_big()
            } else {
                nvim_fileio_msg_permission_denied()
            };
            nvim_fileio_filemess(sfname, msg);
        }
        nvim_fileio_curbuf_set_b_p_ro(1);
        if nvim_fileio_curbuf_mfp_dirty_is_nosync() != 0 {
            nvim_fileio_curbuf_mfp_set_dirty_yes();
        }
        nvim_fileio_xfree(sha_ctx as *mut c_void);
        return retval;
    }

    // Set readonly flag
    if (check_readonly && file_readonly) || nvim_fileio_curbuf_get_b_help() != 0 {
        nvim_fileio_curbuf_set_b_p_ro(1);
    }

    if set_options {
        if !read_buffer {
            nvim_fileio_curbuf_set_b_p_eof(0);
            nvim_fileio_curbuf_set_b_start_eof(0);
            nvim_fileio_curbuf_set_b_p_eol(1);
            nvim_fileio_curbuf_set_b_start_eol(1);
        }
        nvim_fileio_curbuf_set_b_p_bomb(0);
        nvim_fileio_curbuf_set_b_start_bomb(0);
    }

    // Create swap file
    if nvim_fileio_bt_dontwrite() == 0 {
        nvim_fileio_check_need_swap(newfile as c_int);
        if !read_stdin
            && nvim_fileio_curbuf_check_identity(
                old_curbuf,
                old_b_ffname,
                old_b_fname,
                using_b_ffname,
                using_b_fname,
            ) != 0
        {
            nvim_fileio_emsg(nvim_fileio_msg_e_auchangedbuf());
            if !read_buffer {
                nvim_fileio_close(fd);
            }
            if nvim_fileio_curbuf_mfp_dirty_is_nosync() != 0 {
                nvim_fileio_curbuf_mfp_set_dirty_yes();
            }
            nvim_fileio_xfree(sha_ctx as *mut c_void);
            return retval;
        }
        #[cfg(unix)]
        if swap_mode > 0 {
            let swap_fname = nvim_fileio_curbuf_swap_fname();
            if !swap_fname.is_null() {
                let gid = nvim_fileio_curbuf_swap_gid();
                let sfd = nvim_fileio_curbuf_swap_fd();
                if gid >= 0 && sfd >= 0 {
                    let new_mode =
                        nvim_fileio_check_swap_mode_group(swap_fname, gid, sfd, swap_mode);
                    nvim_fileio_os_setperm(swap_fname, new_mode);
                }
            }
        }
    }

    // If "Quit" selected at ATTENTION dialog
    if nvim_fileio_get_swap_exists_action() == nvim_fileio_SEA_QUIT() {
        if !read_buffer && !read_stdin {
            nvim_fileio_close(fd);
        }
        if nvim_fileio_curbuf_mfp_dirty_is_nosync() != 0 {
            nvim_fileio_curbuf_mfp_set_dirty_yes();
        }
        nvim_fileio_xfree(sha_ctx as *mut c_void);
        return retval;
    }

    nvim_set_no_wait_return(nvim_get_no_wait_return() + 1);

    // Set '[ mark
    nvim_fileio_curbuf_get_b_op_start(&mut orig_start_lnum, &mut orig_start_col);
    nvim_fileio_curbuf_set_b_op_start(if from == 0 { 1 } else { from }, 0);

    let mut try_mac = nvim_fileio_vim_strchr(nvim_fileio_get_p_ffs(), b'm' as i32) != 0;
    let mut try_dos = nvim_fileio_vim_strchr(nvim_fileio_get_p_ffs(), b'd' as i32) != 0;
    let mut try_unix = nvim_fileio_vim_strchr(nvim_fileio_get_p_ffs(), b'x' as i32) != 0;

    if !read_buffer {
        let m = nvim_fileio_get_msg_scroll();
        let n = nvim_fileio_get_msg_scrolled();

        if !read_stdin {
            nvim_fileio_close(fd);
        }

        nvim_fileio_set_msg_scroll(1);
        if filtering {
            nvim_fileio_apply_autocmds_exarg(
                nvim_event_FILTERREADPRE(),
                ptr::null(),
                sfname,
                0,
                nvim_fileio_get_curbuf(),
                eap,
            );
        } else if read_stdin {
            nvim_fileio_apply_autocmds_exarg(
                nvim_event_STDINREADPRE(),
                ptr::null(),
                sfname,
                0,
                nvim_fileio_get_curbuf(),
                eap,
            );
        } else if newfile {
            nvim_fileio_apply_autocmds_exarg(
                nvim_event_BUFREADPRE(),
                ptr::null(),
                sfname,
                0,
                nvim_fileio_get_curbuf(),
                eap,
            );
        } else {
            nvim_fileio_apply_autocmds_exarg(
                nvim_event_FILEREADPRE(),
                sfname,
                sfname,
                0,
                ptr::null_mut(),
                eap,
            );
        }

        // autocommands may have changed p_ffs
        try_mac = nvim_fileio_vim_strchr(nvim_fileio_get_p_ffs(), b'm' as i32) != 0;
        try_dos = nvim_fileio_vim_strchr(nvim_fileio_get_p_ffs(), b'd' as i32) != 0;
        try_unix = nvim_fileio_vim_strchr(nvim_fileio_get_p_ffs(), b'x' as i32) != 0;
        nvim_fileio_curbuf_set_b_op_start(orig_start_lnum, orig_start_col);

        if nvim_fileio_get_msg_scrolled() == n {
            nvim_fileio_set_msg_scroll(m);
        }

        if nvim_fileio_aborting() != 0 {
            nvim_set_no_wait_return(nvim_get_no_wait_return() - 1);
            nvim_fileio_set_msg_scroll(msg_save);
            nvim_fileio_curbuf_set_b_p_ro(1);
            if nvim_fileio_curbuf_mfp_dirty_is_nosync() != 0 {
                nvim_fileio_curbuf_mfp_set_dirty_yes();
            }
            nvim_fileio_xfree(sha_ctx as *mut c_void);
            return retval;
        }

        if !read_stdin {
            if nvim_fileio_curbuf_check_identity(
                old_curbuf,
                old_b_ffname,
                old_b_fname,
                using_b_ffname,
                using_b_fname,
            ) != 0
            {
                // curbuf changed by autocmd
                nvim_set_no_wait_return(nvim_get_no_wait_return() - 1);
                nvim_fileio_set_msg_scroll(msg_save);
                nvim_fileio_emsg(nvim_fileio_msg_e201());
                nvim_fileio_curbuf_set_b_p_ro(1);
                if nvim_fileio_curbuf_mfp_dirty_is_nosync() != 0 {
                    nvim_fileio_curbuf_mfp_set_dirty_yes();
                }
                nvim_fileio_xfree(sha_ctx as *mut c_void);
                return retval;
            }
            // Try to re-open the file
            fd = nvim_fileio_os_open_rdonly(fname);
            if fd < 0 {
                nvim_set_no_wait_return(nvim_get_no_wait_return() - 1);
                nvim_fileio_set_msg_scroll(msg_save);
                nvim_fileio_emsg(nvim_fileio_msg_e200());
                nvim_fileio_curbuf_set_b_p_ro(1);
                if nvim_fileio_curbuf_mfp_dirty_is_nosync() != 0 {
                    nvim_fileio_curbuf_mfp_set_dirty_yes();
                }
                nvim_fileio_xfree(sha_ctx as *mut c_void);
                return retval;
            }
        }
    }

    wasempty = ((nvim_fileio_curbuf_get_ml_flags() & nvim_fileio_ML_EMPTY()) != 0) as i32;

    if nvim_fileio_get_recoverymode() == 0 && !filtering && (flags & READ_DUMMY) == 0 && silent == 0
    {
        if !read_stdin && !read_buffer {
            let empty: [u8; 1] = [0u8];
            nvim_fileio_filemess(sfname, empty.as_ptr() as *const c_char);
        }
    }

    nvim_fileio_set_msg_scroll(0); // overwrite file message

    linecnt = nvim_fileio_curbuf_get_ml_line_count();

    // "++bad=" argument
    if !eap.is_null() && nvim_fileio_eap_bad_char(eap) != 0 {
        bad_char_behavior = nvim_fileio_eap_bad_char(eap);
        if set_options {
            nvim_fileio_curbuf_set_b_bad_char(bad_char_behavior);
        }
    } else {
        nvim_fileio_curbuf_set_b_bad_char(0);
    }

    // Decide which encoding to use
    let enc_ucsbom = nvim_fileio_ENC_UCSBOM();
    if !eap.is_null() && nvim_fileio_eap_force_enc(eap) != 0 {
        fenc = nvim_fileio_enc_canonize(nvim_fileio_eap_force_enc_str(eap));
        fenc_alloced = true;
        keep_dest_enc = true;
    } else if nvim_fileio_curbuf_get_b_p_bin() != 0 {
        fenc = b"\0".as_ptr() as *mut c_char; // binary: don't convert
        fenc_alloced = false;
    } else if nvim_fileio_curbuf_get_b_help() != 0 {
        fenc_next = b"latin1\0".as_ptr() as *mut c_char;
        fenc = b"utf-8\0".as_ptr() as *mut c_char;
        fenc_alloced = false;
    } else if *nvim_fileio_get_p_fencs() == 0 {
        fenc = nvim_fileio_curbuf_get_b_p_fenc() as *mut c_char;
        fenc_alloced = false;
    } else {
        fenc_next = nvim_fileio_get_p_fencs() as *mut c_char;
        let mut fenc_alloced_c: c_int = 0;
        fenc = next_fenc(&mut fenc_next as *mut *mut c_char, &mut fenc_alloced_c);
        fenc_alloced = fenc_alloced_c != 0;
    }

    // =========================================================================
    // RETRY LOOP
    // This loop repeats when we need to try a different encoding.
    // =========================================================================
    'retry: loop {
        if file_rewind {
            if read_buffer {
                read_buf_lnum = 1;
                read_buf_col = 0;
            } else if read_stdin || nvim_fileio_vim_lseek(fd, 0, SEEK_SET) != 0 {
                error = true;
                break 'retry; // goto failed
            }
            // Delete the previously read lines
            while lnum > from {
                nvim_fileio_ml_delete(lnum);
                lnum -= 1;
            }
            file_rewind = false;
            if set_options {
                nvim_fileio_curbuf_set_b_p_bomb(0);
                nvim_fileio_curbuf_set_b_start_bomb(0);
            }
            conv_error = 0;
        }

        // Set fileformat
        if keep_fileformat {
            keep_fileformat = false;
        } else if !eap.is_null() && nvim_fileio_eap_force_ff(eap) != 0 {
            fileformat = nvim_fileio_get_fileformat_force(eap);
            try_unix = false;
            try_dos = false;
            try_mac = false;
        } else if nvim_fileio_curbuf_get_b_p_bin() != 0 {
            fileformat = eol_unix;
        } else if *nvim_fileio_get_p_ffs() == 0 {
            fileformat = rs_get_fileformat(nvim_fileio_get_curbuf());
        } else {
            fileformat = eol_unknown;
        }

        if nvim_fileio_iconv_is_invalid(iconv_fd) == 0 {
            nvim_fileio_iconv_close(iconv_fd);
            iconv_fd = iconv_invalid;
        }

        if advance_fenc {
            advance_fenc = false;

            if !eap.is_null() && nvim_fileio_eap_force_enc(eap) != 0 {
                notconverted = true;
                conv_error = 0;
                if fenc_alloced {
                    nvim_fileio_xfree(fenc as *mut c_void);
                }
                fenc = b"\0".as_ptr() as *mut c_char;
                fenc_alloced = false;
            } else {
                if fenc_alloced {
                    nvim_fileio_xfree(fenc as *mut c_void);
                }
                if !fenc_next.is_null() {
                    let mut fenc_alloced_c: c_int = 0;
                    fenc = next_fenc(&mut fenc_next as *mut *mut c_char, &mut fenc_alloced_c);
                    fenc_alloced = fenc_alloced_c != 0;
                } else {
                    fenc = b"\0".as_ptr() as *mut c_char;
                    fenc_alloced = false;
                }
            }
            if !tmpname.is_null() {
                nvim_fileio_os_remove(tmpname);
                nvim_fileio_xfree(tmpname as *mut c_void);
                tmpname = ptr::null_mut();
            }
        }

        // Conversion may be required
        fio_flags = 0;
        converted = rs_need_conversion(fenc) != 0;
        if converted {
            if strcmp_c(fenc, enc_ucsbom) {
                fio_flags = FIO_UCSBOM;
            } else {
                fio_flags = rs_get_fio_flags(fenc);
            }

            if fio_flags == 0 && !did_iconv {
                iconv_fd = nvim_fileio_my_iconv_open(b"utf-8\0".as_ptr() as *const c_char, fenc);
            }

            if fio_flags == 0
                && !read_stdin
                && !read_buffer
                && *nvim_fileio_get_p_ccv() != 0
                && !read_fifo
                && nvim_fileio_iconv_is_invalid(iconv_fd) != 0
            {
                did_iconv = false;
                if tmpname.is_null() {
                    tmpname = readfile_charconvert(fname, fenc, &mut fd as *mut c_int);
                    if tmpname.is_null() {
                        advance_fenc = true;
                        if fd < 0 {
                            nvim_fileio_emsg(nvim_fileio_msg_e202());
                            error = true;
                            break 'retry; // goto failed
                        }
                        continue 'retry; // goto retry
                    }
                }
            } else if fio_flags == 0 && nvim_fileio_iconv_is_invalid(iconv_fd) != 0 {
                advance_fenc = true;
                continue 'retry; // goto retry
            }
        }

        let can_retry = *fenc != 0 && !read_stdin && !keep_dest_enc && !read_fifo;

        if !skip_read {
            linerest = 0;
            filesize = 0;
            skip_count = lines_to_skip;
            read_count = lines_to_read;
            conv_restlen = 0;
            read_undo_file = newfile
                && (flags & READ_KEEP_UNDO) == 0
                && !nvim_fileio_curbuf_get_b_ffname().is_null()
                && nvim_fileio_curbuf_get_b_p_udf() != 0
                && !filtering
                && !read_fifo
                && !read_stdin
                && !read_buffer;
            if read_undo_file {
                nvim_fileio_sha256_start(sha_ctx);
            }
        }

        // =====================================================================
        // INNER READ LOOP
        // =====================================================================
        'read_loop: while !error && nvim_fileio_get_got_int() == 0 {
            // Allocate/reuse buffer
            if !skip_read {
                size = isize::min(0x10000 + linerest, 0x100000);
            }

            // Protect against size going negative
            if size < 0 || size + linerest + 1 < 0 || linerest >= MAXCOL {
                split += 1;
                // insert a NL to split the line
                *buffer.add(linerest as usize) = NL;
                size = 1;
            } else if !skip_read {
                // Allocate new buffer
                let alloc_size = size as usize + linerest as usize + 1;
                let mut alloc_size_try = alloc_size;
                new_buffer = ptr::null_mut();
                while alloc_size_try >= 10 {
                    new_buffer = nvim_fileio_verbose_try_malloc(alloc_size_try) as *mut u8;
                    if !new_buffer.is_null() {
                        break;
                    }
                    alloc_size_try /= 2;
                    size = alloc_size_try as isize - linerest;
                }
                if new_buffer.is_null() {
                    error = true;
                    break 'read_loop;
                }
                if linerest != 0 && !buffer.is_null() {
                    // Copy unprocessed bytes from end of previous buffer
                    // In C: memmove(new_buffer, ptr - linerest, linerest)
                    // where ptr = buffer + linerest, so ptr - linerest = buffer
                    std::ptr::copy(buffer, new_buffer, linerest as usize);
                }
                nvim_fileio_xfree(buffer as *mut c_void);
                buffer = new_buffer;
                let cur_ptr = buffer.add(linerest as usize);
                line_start = buffer;

                // Adjust size for iconv/encoding conversion
                real_size = size as i32;
                if nvim_fileio_iconv_is_invalid(iconv_fd) == 0 {
                    size /= ICONV_MULT as isize;
                } else if (fio_flags & FIO_LATIN1) != 0 {
                    size /= 2;
                } else if (fio_flags & (FIO_UCS2 | FIO_UTF16)) != 0 {
                    size = (size * 2 / 3) & !1;
                } else if (fio_flags & FIO_UCS4) != 0 {
                    size = (size * 2 / 3) & !3;
                } else if fio_flags == FIO_UCSBOM {
                    size /= ICONV_MULT as isize;
                }

                // Insert unconverted bytes from previous read
                let cur_ptr = if conv_restlen > 0 {
                    std::ptr::copy_nonoverlapping(conv_rest.as_ptr(), cur_ptr, conv_restlen);
                    cur_ptr.add(conv_restlen)
                    // Actually we need to track ptr manually here
                } else {
                    cur_ptr
                };
                let mut ptr = cur_ptr;
                if conv_restlen > 0 {
                    size -= conv_restlen as isize;
                }

                if read_buffer {
                    // Read from curbuf
                    if read_buf_lnum > from {
                        size = 0;
                    } else {
                        let mut tlen: isize = 0;
                        'buf_read: loop {
                            let line_ptr = nvim_fileio_ml_get(read_buf_lnum) as *const u8;
                            let n = nvim_fileio_ml_get_len(read_buf_lnum) - read_buf_col;
                            if tlen + n as isize + 1 > size {
                                let n = (size - tlen) as i32;
                                for ni in 0..n {
                                    let byte = *line_ptr.add((read_buf_col + ni) as usize);
                                    *ptr.add(tlen as usize + ni as usize) =
                                        if byte == NL { NUL } else { byte };
                                }
                                tlen += n as isize;
                                read_buf_col += n;
                                break 'buf_read;
                            }
                            for ni in 0..n {
                                let byte = *line_ptr.add((read_buf_col + ni) as usize);
                                *ptr.add(tlen as usize + ni as usize) =
                                    if byte == NL { NUL } else { byte };
                            }
                            tlen += n as isize;
                            *ptr.add(tlen as usize) = NL;
                            tlen += 1;
                            read_buf_col = 0;
                            read_buf_lnum += 1;
                            if read_buf_lnum > from {
                                if nvim_fileio_curbuf_get_b_p_eol() == 0 {
                                    tlen -= 1;
                                }
                                size = tlen;
                                break 'buf_read;
                            }
                        }
                    }
                } else {
                    // Read from file
                    use std::slice;
                    let read_buf = slice::from_raw_parts_mut(ptr, size as usize);
                    let n = read_eintr_raw(fd, read_buf);
                    size = n;
                }

                // Note: ptr pointing is managed differently now - we continue with buffer + linerest
                // We need to restructure: the C code uses ptr to track position
                // For simplicity, track ptr explicitly
                // Reset ptr to where we just read into
                ptr = buffer.add(linerest as usize);

                if size <= 0 {
                    if size < 0 {
                        error = true;
                    } else if conv_restlen > 0 {
                        // EOF with trailing unconverted bytes
                        if (fio_flags != 0 || nvim_fileio_iconv_is_invalid(iconv_fd) == 0) {
                            if can_retry {
                                // goto rewind_retry
                                if *nvim_fileio_get_p_ccv() != 0
                                    && nvim_fileio_iconv_is_invalid(iconv_fd) == 0
                                {
                                    did_iconv = true;
                                } else {
                                    advance_fenc = true;
                                }
                                file_rewind = true;
                                continue 'retry;
                            }
                            if conv_error == 0 {
                                conv_error = nvim_fileio_curbuf_get_ml_line_count() - linecnt + 1;
                            }
                        } else if illegal_byte == 0 {
                            illegal_byte = nvim_fileio_curbuf_get_ml_line_count() - linecnt + 1;
                        }
                        if bad_char_behavior == bad_drop {
                            *ptr.offset(-(conv_restlen as isize)) = NUL;
                            conv_restlen = 0;
                        } else {
                            if bad_char_behavior != bad_keep
                                && (fio_flags != 0 || nvim_fileio_iconv_is_invalid(iconv_fd) == 0)
                            {
                                let mut cr = conv_restlen as isize;
                                while cr > 0 {
                                    cr -= 1;
                                    *ptr.offset(-(cr + 1)) = bad_char_behavior as u8;
                                    conv_restlen -= 1;
                                }
                            }
                            fio_flags = 0;
                            if nvim_fileio_iconv_is_invalid(iconv_fd) == 0 {
                                nvim_fileio_iconv_close(iconv_fd);
                                iconv_fd = iconv_invalid;
                            }
                        }
                    }
                }
            } // end if !skip_read

            // Re-establish ptr for this iteration
            let mut ptr = buffer.add(linerest as usize);
            skip_read = false;

            // Include not converted bytes
            if conv_restlen > 0 {
                ptr = ptr.offset(-(conv_restlen as isize));
                size += conv_restlen as isize;
                conv_restlen = 0;
            }

            if size <= 0 {
                break 'read_loop;
            }

            // BOM detection at start of file
            if filesize == 0
                && (fio_flags == FIO_UCSBOM
                    || (nvim_fileio_curbuf_get_b_p_bomb() == 0
                        && tmpname.is_null()
                        && (*fenc == b'u' as i8 || *fenc == 0)))
            {
                let ccname: *const c_char;
                let mut blen: i32 = 0;

                if size < 2 || nvim_fileio_curbuf_get_b_p_bin() != 0 {
                    ccname = ptr::null();
                } else {
                    let check_flags = if fio_flags == FIO_UCSBOM {
                        FIO_ALL
                    } else {
                        rs_get_fio_flags(fenc)
                    };
                    ccname = rs_check_for_bom(ptr, size as i32, &mut blen, check_flags);
                }

                if !ccname.is_null() {
                    filesize += blen as i64;
                    size -= blen as isize;
                    std::ptr::copy(ptr.add(blen as usize), ptr, size as usize);
                    if set_options {
                        nvim_fileio_curbuf_set_b_p_bomb(1);
                        nvim_fileio_curbuf_set_b_start_bomb(1);
                    }
                }

                if fio_flags == FIO_UCSBOM {
                    if ccname.is_null() {
                        advance_fenc = true;
                    } else {
                        if fenc_alloced {
                            nvim_fileio_xfree(fenc as *mut c_void);
                        }
                        fenc = ccname as *mut c_char;
                        fenc_alloced = false;
                    }
                    skip_read = true;
                    continue 'retry;
                }
            }

            // iconv conversion
            if nvim_fileio_iconv_is_invalid(iconv_fd) == 0 {
                let fromp = ptr as *const c_char;
                let mut fromp_mut = fromp;
                let mut from_size = size as usize;
                let top = ptr.add(size as usize) as *mut c_char;
                let mut top_mut = top;
                let mut to_size = (real_size as usize).saturating_sub(size as usize);

                // iconv conversion loop
                loop {
                    let result = nvim_fileio_iconv(
                        iconv_fd,
                        &mut fromp_mut as *mut *const c_char,
                        &mut from_size,
                        &mut top_mut,
                        &mut to_size,
                    );
                    let iconv_err = nvim_fileio_iconv_errno();
                    let iconv_einval = nvim_fileio_iconv_einval();
                    if result != usize::MAX || iconv_err == iconv_einval {
                        if from_size <= CONV_RESTLEN {
                            break;
                        }
                    }
                    if can_retry {
                        // goto rewind_retry
                        if *nvim_fileio_get_p_ccv() != 0
                            && nvim_fileio_iconv_is_invalid(iconv_fd) == 0
                        {
                            did_iconv = true;
                        } else {
                            advance_fenc = true;
                        }
                        file_rewind = true;
                        // free buffer and continue retry
                        nvim_fileio_xfree(buffer as *mut c_void);
                        buffer = ptr::null_mut();
                        continue 'retry;
                    }
                    if conv_error == 0 {
                        conv_error = readfile_linenr(
                            linecnt,
                            ptr as *const c_char,
                            top_mut as *const c_char,
                        );
                    }
                    // Skip bad byte
                    fromp_mut = fromp_mut.add(1);
                    from_size -= 1;
                    if bad_char_behavior == bad_keep {
                        *top_mut = *fromp_mut.sub(1);
                        top_mut = top_mut.add(1);
                        to_size -= 1;
                    } else if bad_char_behavior != bad_drop {
                        *top_mut = bad_char_behavior as u8 as c_char;
                        top_mut = top_mut.add(1);
                        to_size -= 1;
                    }
                }

                if from_size > 0 {
                    std::ptr::copy_nonoverlapping(
                        fromp_mut as *const u8,
                        conv_rest.as_mut_ptr(),
                        from_size,
                    );
                    conv_restlen = from_size;
                }

                // Move linerest before converted chars
                line_start = (top_mut as *mut u8).sub(linerest as usize);
                std::ptr::copy(buffer, line_start, linerest as usize);
                size = (top_mut as isize) - (ptr.add(size as usize) as isize);
                // ptr stays as is (we'll recalculate below)
            }

            // FIO_* conversion (Unicode/Latin1 to UTF-8)
            if fio_flags != 0 {
                let (new_size, new_ptr, new_line_start) = fio_convert(
                    ptr,
                    size as usize,
                    real_size as usize,
                    linerest as usize,
                    fio_flags,
                    buffer,
                    bad_char_behavior,
                    bad_keep,
                    bad_drop,
                    can_retry,
                    linecnt,
                    &mut conv_rest,
                    &mut conv_restlen,
                    &mut conv_error,
                );
                match new_size {
                    None => {
                        // rewind_retry triggered
                        if *nvim_fileio_get_p_ccv() != 0
                            && nvim_fileio_iconv_is_invalid(iconv_fd) == 0
                        {
                            did_iconv = true;
                        } else {
                            advance_fenc = true;
                        }
                        file_rewind = true;
                        nvim_fileio_xfree(buffer as *mut c_void);
                        buffer = ptr::null_mut();
                        continue 'retry;
                    }
                    Some(sz) => {
                        size = sz as isize;
                        ptr = new_ptr;
                        line_start = new_line_start;
                    }
                }
            } else if nvim_fileio_curbuf_get_b_p_bin() == 0 {
                // UTF-8 validation
                let mut incomplete_tail = false;
                let mut pp = ptr;
                let end_ptr = ptr.add(size as usize);

                'utf8_check: loop {
                    if pp >= end_ptr {
                        break 'utf8_check;
                    }
                    let todo = (end_ptr as isize - pp as isize) as i32;
                    if *pp >= 0x80 {
                        let l = nvim_fileio_utf_ptr2len_len(pp as *const c_char, todo);
                        if l > todo && !incomplete_tail {
                            if pp > ptr || filesize > 0 {
                                incomplete_tail = true;
                            }
                            if pp > ptr {
                                conv_restlen = todo as usize;
                                std::ptr::copy_nonoverlapping(
                                    pp,
                                    conv_rest.as_mut_ptr(),
                                    conv_restlen,
                                );
                                size -= conv_restlen as isize;
                                break 'utf8_check;
                            }
                        }
                        if l == 1 || l > todo {
                            if can_retry && !incomplete_tail {
                                // goto rewind_retry
                                if *nvim_fileio_get_p_ccv() != 0
                                    && nvim_fileio_iconv_is_invalid(iconv_fd) == 0
                                {
                                    did_iconv = true;
                                } else {
                                    advance_fenc = true;
                                }
                                file_rewind = true;
                                nvim_fileio_xfree(buffer as *mut c_void);
                                buffer = ptr::null_mut();
                                continue 'retry;
                            }

                            if nvim_fileio_iconv_is_invalid(iconv_fd) == 0 && conv_error == 0 {
                                conv_error = readfile_linenr(
                                    linecnt,
                                    ptr as *const c_char,
                                    pp as *const c_char,
                                );
                            }

                            if conv_error == 0 && illegal_byte == 0 {
                                illegal_byte = readfile_linenr(
                                    linecnt,
                                    ptr as *const c_char,
                                    pp as *const c_char,
                                );
                            }

                            if bad_char_behavior == bad_drop {
                                std::ptr::copy(pp.add(1), pp, (todo - 1) as usize);
                                pp = pp.sub(1);
                                size -= 1;
                            } else if bad_char_behavior != bad_keep {
                                *pp = bad_char_behavior as u8;
                            }
                        } else {
                            pp = pp.add(l as usize - 1);
                        }
                    }
                    pp = pp.add(1);
                }

                if pp < end_ptr && !incomplete_tail {
                    // Detected UTF-8 error - goto rewind_retry
                    if *nvim_fileio_get_p_ccv() != 0 && nvim_fileio_iconv_is_invalid(iconv_fd) == 0
                    {
                        did_iconv = true;
                    } else {
                        advance_fenc = true;
                    }
                    file_rewind = true;
                    nvim_fileio_xfree(buffer as *mut c_void);
                    buffer = ptr::null_mut();
                    continue 'retry;
                }
            }

            filesize += size as i64;

            // EOL format detection
            if fileformat == eol_unknown {
                if try_dos || try_unix {
                    if try_mac {
                        // Reset CR counter
                        try_mac = true; // 1 == true, will be used as counter below
                    }

                    let mut pp = ptr;
                    while pp < ptr.add(size as usize) {
                        if *pp == NL {
                            if !try_unix || (try_dos && pp > ptr && *pp.sub(1) == CAR) {
                                fileformat = eol_dos;
                            } else {
                                fileformat = eol_unix;
                            }
                            break;
                        } else if *pp == CAR && try_mac {
                            // Count CRs
                        }
                        pp = pp.add(1);
                    }

                    // Don't give in to EOL_UNIX if EOL_MAC is more likely
                    if fileformat == eol_unix && try_mac {
                        // Recount
                        let mut pp = ptr.add(size as usize);
                        // scan backwards for CR
                        while pp > ptr && *pp.sub(1) != CAR {
                            pp = pp.sub(1);
                        }
                        if pp > ptr {
                            // Found a CR, recount
                            let mut mac_count = 1i32;
                            let mut unix_count = 0i32;
                            let mut pp2 = ptr;
                            while pp2 < ptr.add(size as usize) {
                                if *pp2 == NL {
                                    unix_count += 1;
                                } else if *pp2 == CAR {
                                    mac_count += 1;
                                }
                                pp2 = pp2.add(1);
                            }
                            if mac_count > unix_count {
                                fileformat = eol_mac;
                            }
                        }
                    } else if fileformat == eol_unknown && !try_mac {
                        // No CR found
                        fileformat = rs_default_fileformat();
                    }
                }

                if fileformat == eol_unknown && try_mac {
                    // Scan for CR
                    let mut has_cr = false;
                    let mut pp = ptr;
                    while pp < ptr.add(size as usize) {
                        if *pp == CAR {
                            has_cr = true;
                            break;
                        }
                        pp = pp.add(1);
                    }
                    if has_cr {
                        fileformat = eol_mac;
                    }
                }

                if fileformat == eol_unknown {
                    fileformat = rs_default_fileformat();
                }

                if set_options {
                    nvim_fileio_set_fileformat(fileformat);
                }
            }

            // Process lines
            // pp_end will hold the position one past the last processed byte (C's `ptr` after the loop)
            let mut pp_end: *mut u8 = ptr; // initialized to something; overwritten in each branch
            if fileformat == eol_mac {
                let mut pp = ptr.sub(1);
                loop {
                    pp = pp.add(1);
                    size -= 1;
                    if size < 0 {
                        pp_end = pp;
                        break;
                    }
                    let c = *pp;
                    if c == NUL {
                        *pp = NL;
                        continue;
                    } else if c == NL {
                        *pp = CAR;
                        continue;
                    } else if c == CAR {
                        if skip_count == 0 {
                            *pp = NUL;
                            len = (pp as isize - line_start as isize + 1) as i32;
                            if nvim_fileio_ml_append(
                                lnum,
                                line_start as *const c_char,
                                len,
                                newfile as c_int,
                            ) == nvim_fileio_FAIL()
                            {
                                error = true;
                                pp_end = pp;
                                break;
                            }
                            if read_undo_file {
                                nvim_fileio_sha256_update(sha_ctx, line_start, len as usize);
                            }
                            lnum += 1;
                            read_count -= 1;
                            if read_count == 0 {
                                error = true;
                                line_start = pp;
                                pp_end = pp;
                                break;
                            }
                        } else {
                            skip_count -= 1;
                        }
                        line_start = pp.add(1);
                    }
                }
            } else {
                // Unix and DOS processing
                let mut pp = ptr.sub(1);
                loop {
                    pp = pp.add(1);
                    size -= 1;
                    if size < 0 {
                        pp_end = pp;
                        break;
                    }
                    let c = *pp;
                    if c == NUL {
                        *pp = NL;
                        continue;
                    } else if c != NL {
                        continue;
                    }
                    // Found NL
                    if skip_count == 0 {
                        *pp = NUL;
                        len = (pp as isize - line_start as isize + 1) as i32;
                        if fileformat == eol_dos {
                            if pp > line_start && *pp.sub(1) == CAR {
                                *pp.sub(1) = NUL;
                                len -= 1;
                            } else if ff_error != eol_dos {
                                if try_unix
                                    && !read_stdin
                                    && (read_buffer || nvim_fileio_vim_lseek(fd, 0, SEEK_SET) == 0)
                                {
                                    fileformat = eol_unix;
                                    if set_options {
                                        nvim_fileio_set_fileformat(eol_unix);
                                    }
                                    file_rewind = true;
                                    keep_fileformat = true;
                                    // goto retry
                                    nvim_fileio_xfree(buffer as *mut c_void);
                                    buffer = ptr::null_mut();
                                    continue 'retry;
                                }
                                ff_error = eol_dos;
                            }
                        }
                        if nvim_fileio_ml_append(
                            lnum,
                            line_start as *const c_char,
                            len,
                            newfile as c_int,
                        ) == nvim_fileio_FAIL()
                        {
                            error = true;
                            pp_end = pp;
                            break;
                        }
                        if read_undo_file {
                            nvim_fileio_sha256_update(sha_ctx, line_start, len as usize);
                        }
                        lnum += 1;
                        read_count -= 1;
                        if read_count == 0 {
                            error = true;
                            line_start = pp;
                            pp_end = pp;
                            break;
                        }
                    } else {
                        skip_count -= 1;
                    }
                    line_start = pp.add(1);
                }
            }

            // linerest = bytes remaining in unprocessed partial last line.
            // pp_end is the position one past the last processed byte (set in each loop branch).
            // In C: linerest = (char_u *)ptr - line_start (where ptr advanced through the loop).
            linerest = pp_end as isize - line_start as isize;
            nvim_fileio_os_breakcheck();
        } // end 'read_loop

        break 'retry;
    } // end 'retry loop

    // ==========================================================================
    // "failed:" label
    // ==========================================================================
    // not an error if max lines reached
    if error && read_count == 0 {
        error = false;
    }

    // Ctrl-Z at end of file in DOS format
    if linerest != 0 && nvim_fileio_curbuf_get_b_p_bin() == 0 && fileformat == eol_dos {
        let last_byte = *buffer.add((linerest - 1) as usize);
        if last_byte == CTRL_Z {
            linerest -= 1;
            if set_options {
                nvim_fileio_curbuf_set_b_p_eof(1);
            }
        }
    }

    // EOF in middle of a line
    if !error && nvim_fileio_get_got_int() == 0 && linerest != 0 {
        if set_options {
            nvim_fileio_curbuf_set_b_p_eol(0);
        }
        *buffer.add(linerest as usize) = NUL;
        len = linerest as i32 + 1;
        let line_start_ptr = buffer;
        if nvim_fileio_ml_append(lnum, line_start_ptr as *const c_char, len, newfile as c_int)
            == nvim_fileio_FAIL()
        {
            error = true;
        } else {
            if read_undo_file {
                nvim_fileio_sha256_update(sha_ctx, line_start_ptr, len as usize);
            }
            lnum += 1;
            read_no_eol_lnum = lnum;
        }
    }

    if set_options {
        nvim_fileio_save_file_ff();
        nvim_fileio_set_option_direct_fenc(fenc);
    }
    if fenc_alloced {
        nvim_fileio_xfree(fenc as *mut c_void);
    }
    if nvim_fileio_iconv_is_invalid(iconv_fd) == 0 {
        nvim_fileio_iconv_close(iconv_fd);
    }

    if !read_buffer && !read_stdin {
        nvim_fileio_close(fd);
    } else {
        nvim_fileio_os_set_cloexec(fd);
    }
    nvim_fileio_xfree(buffer as *mut c_void);

    if read_stdin {
        nvim_fileio_close(fd);
        if nvim_fileio_get_stdin_fd() < 0 {
            nvim_fileio_stdin_post_read();
        }
    }

    if !tmpname.is_null() {
        nvim_fileio_os_remove(tmpname);
        nvim_fileio_xfree(tmpname as *mut c_void);
    }
    nvim_set_no_wait_return(nvim_get_no_wait_return() - 1);

    // Recovery mode skips most post-read processing
    if nvim_fileio_get_recoverymode() == 0 {
        if newfile
            && wasempty != 0
            && (nvim_fileio_curbuf_get_ml_flags() & nvim_fileio_ML_EMPTY()) == 0
        {
            nvim_fileio_ml_delete(nvim_fileio_curbuf_get_ml_line_count());
            linecnt -= 1;
        }
        nvim_fileio_curbuf_set_deleted_bytes_zero();
        let new_linecnt = nvim_fileio_curbuf_get_ml_line_count() - linecnt;
        linecnt = if filesize == 0 { 0 } else { new_linecnt };

        if newfile || read_buffer {
            nvim_fileio_redraw_curbuf_later();
            rs_diff_invalidate(nvim_fileio_get_curbuf());
            rs_foldUpdateAll(nvim_fileio_get_curwin());
        } else if linecnt != 0 {
            nvim_fileio_appended_lines_mark(from, linecnt);
        }

        if nvim_fileio_get_got_int() != 0 {
            if (flags & READ_DUMMY) == 0 {
                nvim_fileio_filemess(sfname, nvim_fileio_msg_read_interrupted());
                if newfile {
                    nvim_fileio_curbuf_set_b_p_ro(1);
                }
            }
            nvim_fileio_set_msg_scroll(msg_save);
            rs_check_marks_read();
            retval = ok;
            // goto theend
            nvim_fileio_curbuf_set_b_no_eol_lnum(read_no_eol_lnum);
            if nvim_fileio_curbuf_mfp_dirty_is_nosync() != 0 {
                nvim_fileio_curbuf_mfp_set_dirty_yes();
            }
            nvim_fileio_xfree(sha_ctx as *mut c_void);
            return retval;
        }

        if !filtering && (flags & READ_DUMMY) == 0 && silent == 0 {
            nvim_fileio_add_quoted_fname(sfname);

            let iobuff = nvim_fileio_get_IObuff();
            let iosize = nvim_fileio_get_IOSIZE() as usize;
            let mut c_bool = false;

            // Fifo/socket/chr detection (Unix)
            #[cfg(unix)]
            {
                if nvim_fileio_is_s_isfifo(perm) != 0 {
                    nvim_fileio_xstrlcat(iobuff, nvim_fileio_msg_fifo(), iosize);
                    c_bool = true;
                }
                if nvim_fileio_is_s_issock(perm) != 0 {
                    nvim_fileio_xstrlcat(iobuff, nvim_fileio_msg_socket(), iosize);
                    c_bool = true;
                }
                // OPEN_CHR_FILES check is skipped in Rust (handled by C shim)
            }

            if nvim_fileio_curbuf_get_b_p_ro() != 0 {
                let shm_ro = nvim_fileio_SHM_RO();
                let ro_msg = if nvim_fileio_shortmess(shm_ro) != 0 {
                    nvim_fileio_msg_ro()
                } else {
                    nvim_fileio_msg_readonly()
                };
                nvim_fileio_xstrlcat(iobuff, ro_msg, iosize);
                c_bool = true;
            }
            if read_no_eol_lnum != 0 {
                nvim_fileio_xstrlcat(iobuff, nvim_fileio_msg_noeol(), iosize);
                c_bool = true;
            }
            if ff_error == eol_dos {
                nvim_fileio_xstrlcat(iobuff, nvim_fileio_msg_cr_missing(), iosize);
                c_bool = true;
            }
            if split != 0 {
                nvim_fileio_xstrlcat(iobuff, nvim_fileio_msg_long_lines_split(), iosize);
                c_bool = true;
            }
            if notconverted {
                nvim_fileio_xstrlcat(iobuff, nvim_fileio_msg_not_converted(), iosize);
                c_bool = true;
            } else if converted {
                nvim_fileio_xstrlcat(iobuff, nvim_fileio_msg_converted(), iosize);
                c_bool = true;
            }
            if conv_error != 0 {
                let offset = nvim_fileio_strlen_IObuff();
                nvim_fileio_snprintf_iobuff(
                    offset,
                    nvim_fileio_msg_conversion_error(),
                    conv_error as i64,
                );
                c_bool = true;
            } else if illegal_byte > 0 {
                let offset = nvim_fileio_strlen_IObuff();
                nvim_fileio_snprintf_iobuff(
                    offset,
                    nvim_fileio_msg_illegal_byte(),
                    illegal_byte as i64,
                );
                c_bool = true;
            } else if error {
                nvim_fileio_xstrlcat(iobuff, nvim_fileio_msg_read_errors(), iosize);
                c_bool = true;
            }
            if nvim_fileio_msg_add_fileformat(fileformat) != 0 {
                c_bool = true;
            }

            nvim_fileio_msg_add_lines(c_bool as c_int, linecnt, filesize);

            nvim_fileio_XFREE_CLEAR_keep_msg();
            nvim_fileio_set_msg_scrolled_ign(1);

            let msg_p: *mut c_char;
            if !read_stdin && !read_buffer {
                if nvim_fileio_get_msg_col() > 0 {
                    nvim_fileio_msg_putchar(b'\r' as c_int);
                }
                msg_p = nvim_fileio_msg_trunc(iobuff);
            } else {
                msg_p = ptr::null_mut();
            }

            if read_stdin
                || read_buffer
                || nvim_fileio_get_restart_edit() != 0
                || (nvim_fileio_get_msg_scrolled() != 0 && nvim_fileio_get_need_wait_return() == 0)
            {
                nvim_fileio_set_keep_msg(msg_p);
            }
            nvim_fileio_set_msg_scrolled_ign(0);
        }

        // With errors, writing requires ":w!"
        if newfile
            && (error || conv_error != 0 || (illegal_byte > 0 && bad_char_behavior != bad_keep))
        {
            nvim_fileio_curbuf_set_b_p_ro(1);
        }

        nvim_fileio_u_clearline();

        if nvim_fileio_get_exmode_active() != 0 {
            nvim_fileio_curwin_set_cursor_lnum(from + linecnt);
        } else {
            nvim_fileio_curwin_set_cursor_lnum(from + 1);
        }
        nvim_fileio_check_cursor_lnum();
        nvim_fileio_beginline();

        if nvim_fileio_get_cmdmod_lockmarks() == 0 {
            nvim_fileio_curbuf_set_b_op_start(from + 1, 0);
            nvim_fileio_curbuf_set_b_op_end(from + linecnt, 0);
        }
    }

    nvim_fileio_set_msg_scroll(msg_save);

    rs_check_marks_read();

    nvim_fileio_curbuf_set_b_no_eol_lnum(read_no_eol_lnum);

    if (flags & READ_KEEP_UNDO) != 0 {
        nvim_fileio_u_find_first_changed();
    }

    if read_undo_file {
        let hash_size = nvim_fileio_UNDO_HASH_SIZE() as usize;
        let hash = nvim_fileio_xcalloc(hash_size, 1) as *mut u8;
        nvim_fileio_sha256_finish(sha_ctx, hash);
        nvim_fileio_u_read_undo(hash, fname);
        nvim_fileio_xfree(hash as *mut c_void);
    }

    if !read_stdin && !read_fifo && (!read_buffer || !sfname.is_null()) {
        let m = nvim_fileio_get_msg_scroll();
        let n = nvim_fileio_get_msg_scrolled();

        if set_options {
            nvim_fileio_save_file_ff();
        }

        nvim_fileio_set_msg_scroll(1);
        if filtering {
            nvim_fileio_apply_autocmds_exarg(
                nvim_event_FILTERREADPOST(),
                ptr::null(),
                sfname,
                0,
                nvim_fileio_get_curbuf(),
                eap,
            );
        } else if newfile || (read_buffer && !sfname.is_null()) {
            nvim_fileio_apply_autocmds_exarg(
                nvim_event_BUFREADPOST(),
                ptr::null(),
                sfname,
                0,
                nvim_fileio_get_curbuf(),
                eap,
            );
            if nvim_fileio_curbuf_get_b_au_did_filetype() == 0
                && *nvim_fileio_curbuf_get_b_p_ft() != 0
            {
                nvim_fileio_apply_autocmds(
                    nvim_event_FILETYPE(),
                    nvim_fileio_curbuf_get_b_p_ft(),
                    nvim_fileio_curbuf_get_b_fname(),
                    1,
                    nvim_fileio_get_curbuf(),
                );
            }
        } else {
            nvim_fileio_apply_autocmds_exarg(
                nvim_event_FILEREADPOST(),
                sfname,
                sfname,
                0,
                ptr::null_mut(),
                eap,
            );
        }
        if nvim_fileio_get_msg_scrolled() == n {
            nvim_fileio_set_msg_scroll(m);
        }
        if nvim_fileio_aborting() != 0 {
            // Note: C code uses `return FAIL` here without going to theend
            nvim_fileio_curbuf_mfp_set_dirty_yes();
            nvim_fileio_xfree(sha_ctx as *mut c_void);
            return fail;
        }
    }

    if !(nvim_fileio_get_recoverymode() != 0 && error) {
        retval = ok;
    }

    // "theend:" label
    if nvim_fileio_curbuf_mfp_dirty_is_nosync() != 0 {
        nvim_fileio_curbuf_mfp_set_dirty_yes();
    }

    nvim_fileio_xfree(sha_ctx as *mut c_void);
    retval
}

// =============================================================================
// Helper: EINTR-safe read (raw version that returns isize)
// =============================================================================

unsafe fn read_eintr_raw(fd: c_int, buf: &mut [u8]) -> isize {
    use std::io::Read;
    use std::os::unix::io::FromRawFd;
    let mut file = std::mem::ManuallyDrop::new(std::fs::File::from_raw_fd(fd));
    loop {
        match file.read(buf) {
            Ok(n) => return n as isize,
            Err(e) if e.kind() == std::io::ErrorKind::Interrupted => continue,
            Err(_) => return -1,
        }
    }
}

// =============================================================================
// Helper: FIO_* conversion (Latin1, UCS-2, UCS-4, UTF-16 -> UTF-8)
// Returns None if rewind_retry needed, Some(new_size) otherwise.
// Also returns new ptr and line_start.
// =============================================================================

unsafe fn fio_convert(
    ptr: *mut u8,
    size: usize,
    real_size: usize,
    linerest: usize,
    fio_flags: i32,
    buffer: *mut u8,
    bad_char_behavior: i32,
    bad_keep: i32,
    bad_drop: i32,
    can_retry: bool,
    linecnt: i32,
    conv_rest: &mut [u8; CONV_RESTLEN],
    conv_restlen: &mut usize,
    conv_error: &mut i32,
) -> (Option<usize>, *mut u8, *mut u8) {
    let dest = ptr.add(real_size); // write backwards from here
    let mut dest = dest;
    let mut p: *mut u8;
    let mut tail: *mut u8 = ptr::null_mut();

    // Determine end of valid input based on encoding
    if (fio_flags & (FIO_LATIN1 | FIO_UTF8)) != 0 {
        p = ptr.add(size);
        if (fio_flags & FIO_UTF8) != 0 {
            // Check for trailing incomplete UTF-8 sequence
            let mut tp = p.sub(1);
            while tp > ptr && (*tp & 0xc0) == 0x80 {
                tp = tp.sub(1);
            }
            let byte_len = nvim_fileio_utf_byte2len(*tp as c_int) as usize;
            if tp.add(byte_len) <= ptr.add(size) {
                // tail is None
            } else {
                p = tp;
                tail = tp;
            }
        }
    } else if (fio_flags & (FIO_UCS2 | FIO_UTF16)) != 0 {
        p = ptr.add(size & !1);
        if size & 1 != 0 {
            tail = p;
        }
        if (fio_flags & FIO_UTF16) != 0 && p > ptr {
            // Check for trailing leading surrogate word
            let u8c = if (fio_flags & FIO_ENDIAN_L) != 0 {
                let b1 = *p.sub(1) as u32;
                let b0 = *p.sub(2) as u32;
                (b1 << 8) | b0
            } else {
                let b0 = *p.sub(1) as u32;
                let b1 = *p.sub(2) as u32;
                (b1 << 8) | b0
            };
            if u8c >= 0xd800 && u8c <= 0xdbff {
                tail = p.sub(2);
                p = p.sub(2);
            }
        }
    } else {
        // FIO_UCS4
        p = ptr.add(size & !3);
        if size & 3 != 0 {
            tail = p;
        }
    }

    // Move trailing incomplete sequence to conv_rest[]
    if !tail.is_null() {
        *conv_restlen = (ptr.add(size) as isize - tail as isize) as usize;
        std::ptr::copy_nonoverlapping(tail, conv_rest.as_mut_ptr(), *conv_restlen);
        // size is adjusted by caller based on conv_restlen
    }

    // Convert backwards from p down to ptr
    'conv_loop: while p > ptr {
        let u8c: u32 = if (fio_flags & FIO_LATIN1) != 0 {
            p = p.sub(1);
            *p as u32
        } else if (fio_flags & (FIO_UCS2 | FIO_UTF16)) != 0 {
            p = p.sub(2);
            let u8c_val = if (fio_flags & FIO_ENDIAN_L) != 0 {
                (*p.add(1) as u32) << 8 | (*p as u32)
            } else {
                (*p as u32) << 8 | (*p.add(1) as u32)
            };
            if (fio_flags & FIO_UTF16) != 0 && u8c_val >= 0xdc00 && u8c_val <= 0xdfff {
                // Found trailing surrogate, get leading
                if p == ptr {
                    // Missing leading word
                    if can_retry {
                        return (None, ptr, buffer);
                    }
                    if *conv_error == 0 {
                        *conv_error =
                            readfile_linenr(linecnt, ptr as *const c_char, p as *const c_char);
                    }
                    if bad_char_behavior == bad_drop {
                        continue 'conv_loop;
                    }
                    if bad_char_behavior != bad_keep {
                        // Use bad_char_behavior as the char
                        let u8c_bad = bad_char_behavior as u32;
                        let char_len = nvim_fileio_utf_char2len(u8c_bad as c_int) as usize;
                        dest = dest.sub(char_len);
                        nvim_fileio_utf_char2bytes(u8c_bad as c_int, dest as *mut c_char);
                        continue 'conv_loop;
                    }
                    continue 'conv_loop;
                }
                p = p.sub(2);
                let u16c = if (fio_flags & FIO_ENDIAN_L) != 0 {
                    (*p.add(1) as u32) << 8 | (*p as u32)
                } else {
                    (*p as u32) << 8 | (*p.add(1) as u32)
                };
                // Check leading word validity
                if u16c < 0xd800 || u16c > 0xdbff {
                    if can_retry {
                        return (None, ptr, buffer);
                    }
                    if *conv_error == 0 {
                        *conv_error =
                            readfile_linenr(linecnt, ptr as *const c_char, p as *const c_char);
                    }
                    if bad_char_behavior == bad_drop {
                        continue 'conv_loop;
                    }
                    if bad_char_behavior != bad_keep {
                        let u8c_bad = bad_char_behavior as u32;
                        let char_len = nvim_fileio_utf_char2len(u8c_bad as c_int) as usize;
                        dest = dest.sub(char_len);
                        nvim_fileio_utf_char2bytes(u8c_bad as c_int, dest as *mut c_char);
                        continue 'conv_loop;
                    }
                    continue 'conv_loop;
                }
                0x10000 + ((u16c & 0x3ff) << 10) + (u8c_val & 0x3ff)
            } else {
                u8c_val
            }
        } else if (fio_flags & FIO_UCS4) != 0 {
            p = p.sub(4);
            if (fio_flags & FIO_ENDIAN_L) != 0 {
                (*p.add(3) as u32) << 24
                    | (*p.add(2) as u32) << 16
                    | (*p.add(1) as u32) << 8
                    | (*p as u32)
            } else {
                (*p as u32) << 24
                    | (*p.add(1) as u32) << 16
                    | (*p.add(2) as u32) << 8
                    | (*p.add(3) as u32)
            }
        } else {
            // FIO_UTF8 - go backwards
            p = p.sub(1);
            if *p < 0x80 {
                *p as u32
            } else {
                let head_off =
                    nvim_fileio_utf_head_off(ptr as *const c_char, p as *const c_char) as usize;
                p = p.sub(head_off);
                let u8c_val = nvim_fileio_utf_ptr2char(p as *const c_char) as u32;
                if head_off == 0 {
                    // Invalid UTF-8
                    if can_retry {
                        return (None, ptr, buffer);
                    }
                    if *conv_error == 0 {
                        *conv_error =
                            readfile_linenr(linecnt, ptr as *const c_char, p as *const c_char);
                    }
                    if bad_char_behavior == bad_drop {
                        continue 'conv_loop;
                    }
                    if bad_char_behavior != bad_keep {
                        let u8c_bad = bad_char_behavior as u32;
                        let char_len = nvim_fileio_utf_char2len(u8c_bad as c_int) as usize;
                        dest = dest.sub(char_len);
                        nvim_fileio_utf_char2bytes(u8c_bad as c_int, dest as *mut c_char);
                        continue 'conv_loop;
                    }
                    continue 'conv_loop;
                }
                u8c_val
            }
        };

        // Replace chars > INT_MAX (UCS-4 only)
        let u8c = if u8c > i32::MAX as u32 { 0xfffd } else { u8c };

        let char_len = nvim_fileio_utf_char2len(u8c as c_int) as usize;
        dest = dest.sub(char_len);
        nvim_fileio_utf_char2bytes(u8c as c_int, dest as *mut c_char);
    }

    // Move linerest before converted chars
    let new_line_start = dest.sub(linerest);
    std::ptr::copy(buffer, new_line_start, linerest);

    let new_size = (ptr.add(real_size) as isize - dest as isize) as usize;

    (Some(new_size), dest, new_line_start)
}

// =============================================================================
// Helper: C string length
// =============================================================================

unsafe fn libc_strlen(s: *const c_char) -> usize {
    let mut n = 0usize;
    while *s.add(n) != 0 {
        n += 1;
    }
    n
}

// =============================================================================
// Helper: string comparison with C string
// =============================================================================

unsafe fn strcmp_c(a: *const c_char, b: *const c_char) -> bool {
    let mut i = 0;
    loop {
        if *a.add(i) != *b.add(i) {
            return false;
        }
        if *a.add(i) == 0 {
            return true;
        }
        i += 1;
    }
}

// =============================================================================
// Additional FFI for functions used internally
// =============================================================================

extern "C" {
    fn nvim_set_ex_no_reprint(val: c_int);
    fn nvim_get_no_wait_return() -> c_int;
    fn nvim_set_no_wait_return(val: c_int);
    fn nvim_get_swap_exists_action() -> c_int;
    fn nvim_fileio_eap_read_edit(eap: *mut c_void) -> c_int;
    fn nvim_fileio_eap_bad_char(eap: *mut c_void) -> c_int;
    fn nvim_fileio_eap_force_enc(eap: *mut c_void) -> c_int;
    fn nvim_fileio_eap_force_ff(eap: *mut c_void) -> c_int;
    fn nvim_fileio_eap_force_enc_str(eap: *mut c_void) -> *const c_char;
    fn nvim_fileio_get_swap_exists_action() -> c_int;
}
