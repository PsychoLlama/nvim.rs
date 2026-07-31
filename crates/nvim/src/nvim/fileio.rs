#![deny(unsafe_op_in_unsafe_fn)]

use crate::src::nvim::api::private::helpers::cstr_as_string;
use crate::src::nvim::ascii::ascii_isspace;
use crate::src::nvim::autocmd::{
    EVENT_BUFADD, EVENT_BUFDELETE, EVENT_BUFNEW, EVENT_BUFNEWFILE, EVENT_BUFREADCMD,
    EVENT_BUFREADPOST, EVENT_BUFREADPRE, EVENT_BUFWIPEOUT, EVENT_FILECHANGEDSHELL,
    EVENT_FILECHANGEDSHELLPOST, EVENT_FILEREADCMD, EVENT_FILEREADPOST, EVENT_FILEREADPRE,
    EVENT_FILETYPE, EVENT_FILTERREADPOST, EVENT_FILTERREADPRE, EVENT_STDINREADPRE, apply_autocmds,
    apply_autocmds_exarg, aucmd_prepbuf, aucmd_restbuf, augroup_exists, do_doautocmd,
};
use crate::src::nvim::buffer::{
    bt_dontwrite, bt_nofilename, bt_normal, buf_contents_changed, buf_is_empty, buflist_new,
    bufref_valid, do_modelines, set_bufref, setfname, wipe_buffer,
};
use crate::src::nvim::buffer_updates::buf_updates_unload;
use crate::src::nvim::change::{appended_lines_mark, save_file_ff, unchanged};
use crate::src::nvim::cursor::{check_cursor, check_cursor_lnum};
use crate::src::nvim::diff::diff_invalidate;
use crate::src::nvim::drawscreen::{UPD_NOT_VALID, redraw_curbuf_later, status_redraw_all};
use crate::src::nvim::edit::beginline;
use crate::src::nvim::eval::vars::{eval_charconvert, get_vim_var_str, set_vim_var_string};
use crate::src::nvim::event::libuv::uv_strerror;
use crate::src::nvim::ex_eval::aborting;
use crate::src::nvim::fold::{foldUpdateAll, foldmethodIsManual};
use crate::src::nvim::garray::{ga_clear_strings, ga_grow, ga_init};
use crate::src::nvim::getchar::{stuff_empty, typebuf_typed};
use crate::src::nvim::global_cell::GlobalCell;
use crate::src::nvim::log::{LOGLVL_DBG, LOGLVL_ERR, LOGLVL_WRN, logmsg};
use crate::src::nvim::main::{
    IObuff, State, allbuf_lock, autocmd_busy, cmdmod, curbuf, curtab, curwin, did_check_timestamps,
    e_interr, e_notopen, emsg_silent, ex_no_reprint, exiting, exmode_active, first_tabpage,
    firstbuf, firstwin, global_busy, got_int, in_assert_fails, keep_msg, msg_col,
    msg_listdo_overwrite, msg_scroll, msg_scrolled, msg_scrolled_ign, msg_silent,
    need_check_timestamps, need_fileinfo, need_wait_return, no_check_timestamps, no_wait_return,
    p_ar, p_ccv, p_cpo, p_enc, p_fencs, p_ffs, p_fic, p_ur, p_verbose, readonlymode, recoverymode,
    redraw_cmdline, redraw_tabline, restart_edit, stdin_fd, swap_exists_action, vim_ignored,
};
use crate::src::nvim::mbyte::{
    enc_canon_props, enc_canonize, my_iconv_open, utf_byte2len, utf_char2bytes, utf_char2len,
    utf_head_off, utf_ptr2char, utf_ptr2len_len,
};
use crate::src::nvim::memfile::mf_fullname;
use crate::src::nvim::memline::{
    check_need_swap, ml_append, ml_delete, ml_get, ml_get_buf, ml_get_buf_len, ml_get_len, ml_open,
};
use crate::src::nvim::memory::{
    memchrsub, time_to_bytes, verbose_try_malloc, xfree, xmalloc, xmallocz, xmemdupz, xstrdup,
    xstrlcat,
};
use crate::src::nvim::message::{
    do_dialog, emsg, msg, msg_check_for_delay, msg_clr_eos, msg_delay, msg_end, msg_may_trunc,
    msg_outtrans, msg_progress, msg_putchar, msg_puts, msg_puts_hl, msg_schedule_semsg, msg_start,
    msg_trunc, semsg, set_keep_msg, smsg,
};
use crate::src::nvim::r#move::update_topline;
use crate::src::nvim::option::{
    copy_option_part, default_fileformat, get_fileformat, get_fileformat_force, set_fileformat,
    set_option_direct, set_options_bin, shortmess,
};
use crate::src::nvim::options::kOptFileencoding;
use crate::src::nvim::os::env::{expand_env, home_replace, home_replace_save, os_env_exists};
use crate::src::nvim::os::fs::{
    os_closedir, os_copy, os_dirname, os_fchown, os_file_is_writable, os_file_owned, os_fileinfo,
    os_fileinfo_id_equal, os_fileinfo_link, os_fileinfo_size, os_free_acl, os_get_acl, os_getperm,
    os_isdir, os_isrealdir, os_mkdir, os_mkdtemp, os_open, os_path_exists, os_remove, os_rename,
    os_rmdir, os_scandir, os_scandir_next, os_set_acl, os_set_cloexec, os_setperm,
};
use crate::src::nvim::os::input::os_breakcheck;
use crate::src::nvim::os::libc::{
    __assert_fail, __errno_location, close, dup, feof, ferror, fgets, flock, fwrite, getc, gettext,
    iconv, iconv_close, lseek, memchr, memcpy, memmove, ngettext, putc, read, readlink, snprintf,
    strcmp, strlen, symlink, umask, write,
};
use crate::src::nvim::os::users::os_get_username;
use crate::src::nvim::path::{
    add_pathsep, after_pathsep, dir_of_file_exists, path_fnamecmp, path_is_absolute,
    path_shorten_fname, path_tail, path_with_url, vim_FullName,
};
use crate::src::nvim::pos::MAXLNUM;
use crate::src::nvim::sha256::Sha256;
use crate::src::nvim::shada::check_marks_read;
use crate::src::nvim::state::{MODE_CMDLINE, MODE_NORMAL_BUSY};
use crate::src::nvim::strings::{sort_strings, vim_strchr};
use crate::src::nvim::types::ui::kUIMessages;
use crate::src::nvim::types::{
    __off_t, CMD_index, CheckItem, Directory, FILE, FileInfo, OptInt, OptVal, OptValData,
    OptValType, VimVarIndex, aco_save_T, bln_values, buf_T, bufref_T, cmd_addr_T, colnr_T, exarg_T,
    garray_T, iconv_t, int64_t, linenr_T, mfdirty_T, off_T, pos_T, ptrdiff_t, regmatch_T,
    regprog_T, scid_T, size_t, ssize_t, time_t, uint8_t, uint64_t, uintmax_t, uv_dirent_type_t,
    uv_fs_type, uv_gid_t, uv_req_type, uv_stat_t, uv_timespec_t, uv_uid_t,
};
use crate::src::nvim::ui::{ui_flush, ui_has};
use crate::src::nvim::undo::{
    bufIsChanged, u_clearallandblockfree, u_clearline, u_compute_hash, u_find_first_changed,
    u_read_undo, u_savecommon, u_sync, u_unchanged, u_write_undo,
};
use core::ffi::{c_char, c_int, c_void};
use core::ptr;

// The carve of the transpiled module; see each child's docs.
mod convert;
pub use self::convert::*;
mod timestamp;
pub use self::timestamp::*;
mod names;
pub use self::names::*;
mod tempfile;
pub use self::tempfile::*;
// Opaque C type: layout unknown here, only ever used behind a pointer.
#[repr(C)]
pub struct __dirstream {
    _data: [u8; 0],
    _marker: ::core::marker::PhantomData<(*mut u8, ::core::marker::PhantomPinned)>,
}
unsafe extern "C" {
    fn closedir(__dirp: *mut DIR) -> ::core::ffi::c_int;
    fn opendir(__name: *const ::core::ffi::c_char) -> *mut DIR;
    fn dirfd(__dirp: *mut DIR) -> ::core::ffi::c_int;
    fn vim_regcomp(
        expr_arg: *const ::core::ffi::c_char,
        re_flags: ::core::ffi::c_int,
    ) -> *mut regprog_T;
    fn vim_regfree(prog: *mut regprog_T);
    fn vim_regexec(rmp: *mut regmatch_T, line: *const ::core::ffi::c_char, col: colnr_T) -> bool;
}
pub type DIR = __dirstream;
pub type C2Rust_Unnamed_5 = ::core::ffi::c_int;
pub const UV_ENOENT: C2Rust_Unnamed_5 = -2;
pub const UV_EFBIG: C2Rust_Unnamed_5 = -27;
pub const UV_UNKNOWN_REQ: uv_req_type = 0;
pub const UV_DIRENT_UNKNOWN: uv_dirent_type_t = 0;
pub const UV_FS_CUSTOM: uv_fs_type = 0;
pub const MF_DIRTY_YES_NOSYNC: mfdirty_T = 2;
pub const MF_DIRTY_YES: mfdirty_T = 1;
pub type C2Rust_Unnamed_20 = ::core::ffi::c_uint;
pub const MAXCOL: C2Rust_Unnamed_20 = 2147483647;
pub type C2Rust_Unnamed_21 = ::core::ffi::c_uint;
pub const HLF_W: C2Rust_Unnamed_21 = 26;
pub const HLF_E: C2Rust_Unnamed_21 = 6;
pub const kOptValTypeString: OptValType = 2;
pub const CMD_append: CMD_index = 0;
pub const ADDR_LINES: cmd_addr_T = 0;
pub type C2Rust_Unnamed_23 = ::core::ffi::c_uint;
pub const CMOD_LOCKMARKS: C2Rust_Unnamed_23 = 2048;
pub const BLN_DUMMY: bln_values = 4;
pub type C2Rust_Unnamed_25 = ::core::ffi::c_uint;
pub const BL_FIX: C2Rust_Unnamed_25 = 4;
pub const BL_WHITE: C2Rust_Unnamed_25 = 1;
pub const VV_FCS_CHOICE: VimVarIndex = 39;
pub const VV_FCS_REASON: VimVarIndex = 38;
pub const VV_WARNINGMSG: VimVarIndex = 4;
pub type C2Rust_Unnamed_26 = ::core::ffi::c_uint;
pub const ENC_LATIN1: C2Rust_Unnamed_26 = 512;
pub const ENC_2WORD: C2Rust_Unnamed_26 = 256;
pub const ENC_4BYTE: C2Rust_Unnamed_26 = 128;
pub const ENC_2BYTE: C2Rust_Unnamed_26 = 64;
pub const ENC_ENDIAN_L: C2Rust_Unnamed_26 = 32;
pub const ENC_UNICODE: C2Rust_Unnamed_26 = 4;
pub type C2Rust_Unnamed_27 = ::core::ffi::c_uint;
pub const READ_NOFILE: C2Rust_Unnamed_27 = 256;
pub const READ_FIFO: C2Rust_Unnamed_27 = 64;
pub const READ_KEEP_UNDO: C2Rust_Unnamed_27 = 32;
pub const READ_DUMMY: C2Rust_Unnamed_27 = 16;
pub const READ_BUFFER: C2Rust_Unnamed_27 = 8;
pub const READ_STDIN: C2Rust_Unnamed_27 = 4;
pub const READ_FILTER: C2Rust_Unnamed_27 = 2;
pub const READ_NEW: C2Rust_Unnamed_27 = 1;
pub type C2Rust_Unnamed_28 = ::core::ffi::c_int;
pub const FIO_ALL: C2Rust_Unnamed_28 = -1;
pub const FIO_UCSBOM: C2Rust_Unnamed_28 = 16384;
pub const FIO_ENDIAN_L: C2Rust_Unnamed_28 = 128;
pub const FIO_UTF16: C2Rust_Unnamed_28 = 16;
pub const FIO_UCS4: C2Rust_Unnamed_28 = 8;
pub const FIO_UCS2: C2Rust_Unnamed_28 = 4;
pub const FIO_UTF8: C2Rust_Unnamed_28 = 2;
pub const FIO_LATIN1: C2Rust_Unnamed_28 = 1;
pub type C2Rust_Unnamed_29 = ::core::ffi::c_uint;
pub const CONV_RESTLEN: C2Rust_Unnamed_29 = 30;
pub type C2Rust_Unnamed_30 = ::core::ffi::c_uint;
pub const ICONV_MULT: C2Rust_Unnamed_30 = 8;
pub const SHM_OVERALL: C2Rust_Unnamed_35 = 79;
pub const SHM_LINES: C2Rust_Unnamed_35 = 108;
pub const SHM_RO: C2Rust_Unnamed_35 = 114;
pub const OPT_LOCAL: C2Rust_Unnamed_34 = 2;
pub const SHM_OVER: C2Rust_Unnamed_35 = 111;
pub const RELOAD_DETECT: C2Rust_Unnamed_31 = 2;
pub type C2Rust_Unnamed_31 = ::core::ffi::c_uint;
pub const RELOAD_NORMAL: C2Rust_Unnamed_31 = 1;
pub const RELOAD_NONE: C2Rust_Unnamed_31 = 0;
pub const SHM_FILEINFO: C2Rust_Unnamed_35 = 70;
pub const VIM_WARNING: C2Rust_Unnamed_33 = 2;
pub type C2Rust_Unnamed_33 = ::core::ffi::c_uint;
pub type C2Rust_Unnamed_34 = ::core::ffi::c_uint;
pub type C2Rust_Unnamed_35 = ::core::ffi::c_uint;
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const O_RDONLY: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const EOF: ::core::ffi::c_int = -1 as ::core::ffi::c_int;
pub const SEEK_SET: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const UV_FS_COPYFILE_EXCL: ::core::ffi::c_int = 0x1 as ::core::ffi::c_int;
pub const BACKSLASH_IN_FILENAME_BOOL: ::core::ffi::c_int = false_0;
pub const BASENAMELEN: ::core::ffi::c_int = NAME_MAX - 5 as ::core::ffi::c_int;
pub const DEFAULT_MAXPATHL: ::core::ffi::c_int = 4096 as ::core::ffi::c_int;
pub const MAXPATHL: ::core::ffi::c_int = DEFAULT_MAXPATHL;
pub const NUL: ::core::ffi::c_int = '\0' as ::core::ffi::c_int;
pub const NL: ::core::ffi::c_int = '\n' as ::core::ffi::c_int;
pub const CAR: ::core::ffi::c_int = '\r' as ::core::ffi::c_int;
pub const Ctrl_Z: ::core::ffi::c_int = 26 as ::core::ffi::c_int;
pub const PATHSEPSTR: [::core::ffi::c_char; 2] =
    unsafe { ::core::mem::transmute::<[u8; 2], [::core::ffi::c_char; 2]>(*b"/\0") };
pub const BF_CHECK_RO: ::core::ffi::c_int = 0x2 as ::core::ffi::c_int;
pub const BF_NOTEDITED: ::core::ffi::c_int = 0x8 as ::core::ffi::c_int;
pub const BF_NEW: ::core::ffi::c_int = 0x10 as ::core::ffi::c_int;
pub const BF_NEW_W: ::core::ffi::c_int = 0x20 as ::core::ffi::c_int;
pub const ML_EMPTY: ::core::ffi::c_int = 0x1 as ::core::ffi::c_int;
pub const OK: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const FAIL: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const NOTDONE: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
static e_auchangedbuf: GlobalCell<*const ::core::ffi::c_char> = GlobalCell::new(
    b"E812: Autocommands changed buffer or buffer name\0".as_ptr() as *const ::core::ffi::c_char,
);
pub const NONASCII_MASK: uint64_t = (-1 as ::core::ffi::c_int as uint64_t)
    .wrapping_div(0xff as uint64_t)
    .wrapping_mul(0x80 as uint64_t);
/// Report which file is being read or written, in `IObuff`.
///
/// `s` is the note to append; an empty one means the message is progress on
/// a write that is still running.
pub unsafe extern "C" fn filemess(buf: *mut buf_T, name: *mut c_char, s: *mut c_char) {
    unsafe {
        let prev_msg_col = msg_col.get();
        if msg_silent.get() != 0 {
            return;
        }
        let io = IObuff.ptr().cast::<c_char>();
        add_quoted_fname(io, IOSIZE as size_t - 100, buf, name);
        // Avoid an over-long translation causing trouble.
        xstrlcat(io, s, IOSIZE as size_t);

        // For the first message we may have to start a new line. Further ones
        // overwrite the previous one; reset `msg_scroll` before calling this.
        let msg_scroll_save = msg_scroll.get();
        if shortmess(SHM_OVERALL as c_int)
            && msg_listdo_overwrite.get() == 0
            && !exiting.get()
            && p_verbose.get() == 0
        {
            msg_scroll.set(false as c_int);
        }
        if msg_scroll.get() == 0 {
            // Wait a bit when overwriting an error message.
            msg_check_for_delay(false);
        }
        msg_start();
        if prev_msg_col != 0 && msg_col.get() == 0 {
            msg_putchar(b'\r' as c_int); // overwrite any previous message
        }
        msg_scroll.set(msg_scroll_save);
        msg_scrolled_ign.set(true);
        if *s == 0 {
            msg_progress(
                io,
                c"bufwrite".as_ptr().cast_mut(),
                c"running".as_ptr().cast_mut(),
                0,
                false,
                true,
            );
        } else {
            // May truncate the message to avoid a hit-return prompt.
            msg_outtrans(msg_may_trunc(false, io), 0, false);
        }
        msg_clr_eos();
        msg_scrolled_ign.set(false);
    }
}
pub unsafe extern "C" fn readfile(
    mut fname: *mut ::core::ffi::c_char,
    mut sfname: *mut ::core::ffi::c_char,
    mut from: linenr_T,
    mut lines_to_skip: linenr_T,
    mut lines_to_read: linenr_T,
    mut eap: *mut exarg_T,
    mut flags: ::core::ffi::c_int,
    mut silent: bool,
) -> ::core::ffi::c_int {
    unsafe {
        let mut incomplete_tail: bool = false;
        let mut can_retry: bool = false;
        let mut check_readonly: bool = false;
        let mut file_readonly: bool = false;
        let mut try_mac: ::core::ffi::c_int = 0;
        let mut try_dos: ::core::ffi::c_int = 0;
        let mut try_unix: ::core::ffi::c_int = 0;
        let mut retval: ::core::ffi::c_int = FAIL;
        let mut fd: ::core::ffi::c_int = if stdin_fd.get() >= 0 as ::core::ffi::c_int {
            stdin_fd.get()
        } else {
            0 as ::core::ffi::c_int
        };
        let mut newfile: bool = flags & READ_NEW as ::core::ffi::c_int != 0;
        let mut filtering: bool = flags & READ_FILTER as ::core::ffi::c_int != 0;
        let mut read_stdin: bool = flags & READ_STDIN as ::core::ffi::c_int != 0;
        let mut read_buffer: bool = flags & READ_BUFFER as ::core::ffi::c_int != 0;
        let mut read_fifo: bool = flags & READ_FIFO as ::core::ffi::c_int != 0;
        let mut set_options: bool = newfile as ::core::ffi::c_int != 0
            || read_buffer as ::core::ffi::c_int != 0
            || !eap.is_null() && (*eap).read_edit != 0;
        let mut read_buf_lnum: linenr_T = 1 as linenr_T;
        let mut read_buf_col: colnr_T = 0 as colnr_T;
        let mut c: ::core::ffi::c_char = 0;
        let mut lnum: linenr_T = from;
        let mut ptr: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
        let mut buffer: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
        let mut new_buffer: *mut ::core::ffi::c_char =
            ::core::ptr::null_mut::<::core::ffi::c_char>();
        let mut line_start: *mut ::core::ffi::c_char =
            ::core::ptr::null_mut::<::core::ffi::c_char>();
        let mut wasempty: ::core::ffi::c_int = 0;
        let mut len: colnr_T = 0;
        let mut size: ptrdiff_t = 0 as ptrdiff_t;
        let mut p: *mut uint8_t = ::core::ptr::null_mut::<uint8_t>();
        let mut filesize: off_T = 0 as off_T;
        let mut skip_read: bool = false_0 != 0;
        let mut sha_ctx = Sha256::new();
        let mut read_undo_file: bool = false_0 != 0;
        let mut split: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        let mut linecnt: linenr_T = 0;
        let mut error: bool = false_0 != 0;
        let mut ff_error: ::core::ffi::c_int = EOL_UNKNOWN;
        let mut linerest: ptrdiff_t = 0 as ptrdiff_t;
        let mut perm: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        let mut swap_mode: ::core::ffi::c_int = -1 as ::core::ffi::c_int;
        let mut fileformat: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        let mut keep_fileformat: bool = false_0 != 0;
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
        let mut skip_count: linenr_T = 0 as linenr_T;
        let mut read_count: linenr_T = 0 as linenr_T;
        let mut msg_save: ::core::ffi::c_int = msg_scroll.get();
        let mut read_no_eol_lnum: linenr_T = 0 as linenr_T;
        let mut file_rewind: bool = false_0 != 0;
        let mut conv_error: linenr_T = 0 as linenr_T;
        let mut illegal_byte: linenr_T = 0 as linenr_T;
        let mut keep_dest_enc: bool = false_0 != 0;
        let mut bad_char_behavior: ::core::ffi::c_int = BAD_REPLACE;
        let mut tmpname: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
        let mut fio_flags: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        let mut fenc: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
        let mut fenc_alloced: bool = false;
        let mut fenc_next: *mut ::core::ffi::c_char =
            ::core::ptr::null_mut::<::core::ffi::c_char>();
        let mut advance_fenc: bool = false_0 != 0;
        let mut real_size: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        let mut iconv_fd: iconv_t = ::core::ptr::with_exposed_provenance_mut::<::core::ffi::c_void>(
            -1 as ::core::ffi::c_int as usize,
        );
        let mut did_iconv: bool = false_0 != 0;
        let mut converted: bool = false_0 != 0;
        let mut notconverted: bool = false_0 != 0;
        let mut conv_rest: [::core::ffi::c_char; 30] = [0; 30];
        let mut conv_restlen: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        let mut orig_start: pos_T = pos_T {
            lnum: 0,
            col: 0,
            coladd: 0,
        };
        let mut old_curbuf: *mut buf_T = ::core::ptr::null_mut::<buf_T>();
        let mut old_b_ffname: *mut ::core::ffi::c_char =
            ::core::ptr::null_mut::<::core::ffi::c_char>();
        let mut old_b_fname: *mut ::core::ffi::c_char =
            ::core::ptr::null_mut::<::core::ffi::c_char>();
        let mut using_b_ffname: ::core::ffi::c_int = 0;
        let mut using_b_fname: ::core::ffi::c_int = 0;
        static msg_is_a_directory: GlobalCell<*mut ::core::ffi::c_char> =
            GlobalCell::new(b"is a directory\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char);
        (*curbuf.get()).b_au_did_filetype = false_0 != 0;
        (*curbuf.get()).b_no_eol_lnum = 0 as ::core::ffi::c_int as linenr_T;
        '_theend: {
            if (*curbuf.get()).b_ffname.is_null()
                && !filtering
                && !fname.is_null()
                && !vim_strchr(p_cpo.get(), CPO_FNAMER).is_null()
                && flags & READ_DUMMY as ::core::ffi::c_int == 0
            {
                if set_rw_fname(fname, sfname) == FAIL {
                    break '_theend;
                }
            }
            old_curbuf = curbuf.get();
            old_b_ffname = (*curbuf.get()).b_ffname;
            old_b_fname = (*curbuf.get()).b_fname;
            using_b_ffname = (fname == (*curbuf.get()).b_ffname
                || sfname == (*curbuf.get()).b_ffname)
                as ::core::ffi::c_int;
            using_b_fname = (fname == (*curbuf.get()).b_fname || sfname == (*curbuf.get()).b_fname)
                as ::core::ffi::c_int;
            ex_no_reprint.set(true_0 != 0);
            need_fileinfo.set(false_0 != 0);
            if sfname.is_null() {
                sfname = fname;
            }
            fname = sfname;
            if !filtering && !read_stdin && !read_buffer {
                orig_start = (*curbuf.get()).b_op_start;
                (*curbuf.get()).b_op_start.lnum = if from == 0 as linenr_T {
                    1 as linenr_T
                } else {
                    from
                };
                (*curbuf.get()).b_op_start.col = 0 as ::core::ffi::c_int as colnr_T;
                if newfile {
                    if apply_autocmds_exarg(
                        EVENT_BUFREADCMD,
                        ::core::ptr::null_mut::<::core::ffi::c_char>(),
                        sfname,
                        false_0 != 0,
                        curbuf.get(),
                        eap,
                    ) {
                        retval = OK;
                        if aborting() {
                            retval = FAIL;
                        }
                        if retval == OK {
                            (*curbuf.get()).b_flags &= !BF_NOTEDITED;
                        }
                        break '_theend;
                    }
                } else if apply_autocmds_exarg(
                    EVENT_FILEREADCMD,
                    sfname,
                    sfname,
                    false_0 != 0,
                    ::core::ptr::null_mut::<buf_T>(),
                    eap,
                ) {
                    retval = if aborting() as ::core::ffi::c_int != 0 {
                        FAIL
                    } else {
                        OK
                    };
                    break '_theend;
                }
                (*curbuf.get()).b_op_start = orig_start;
                if flags & READ_NOFILE as ::core::ffi::c_int != 0 {
                    retval = NOTDONE;
                    break '_theend;
                }
            }
            if (shortmess(SHM_OVER as ::core::ffi::c_int) as ::core::ffi::c_int != 0
                && msg_listdo_overwrite.get() == 0
                || (*curbuf.get()).b_help as ::core::ffi::c_int != 0)
                && p_verbose.get() == 0 as OptInt
            {
                msg_scroll.set(false_0);
            } else {
                msg_scroll.set(true_0);
            }
            if !fname.is_null() && *fname as ::core::ffi::c_int != NUL {
                let mut fnamelen: size_t = strlen(fname);
                if fnamelen >= MAXPATHL as size_t {
                    filemess(
                        curbuf.get(),
                        fname,
                        gettext(b"Illegal file name\0".as_ptr() as *const ::core::ffi::c_char),
                    );
                    msg_end();
                    msg_scroll.set(msg_save);
                    break '_theend;
                } else if after_pathsep(fname, fname.offset(fnamelen as isize)) != 0 {
                    if !silent {
                        filemess(curbuf.get(), fname, gettext(msg_is_a_directory.get()));
                    }
                    msg_end();
                    msg_scroll.set(msg_save);
                    retval = NOTDONE;
                    break '_theend;
                }
            }
            if !read_stdin && !fname.is_null() {
                perm = os_getperm(fname) as ::core::ffi::c_int;
            }
            if !read_stdin && !read_buffer && !read_fifo {
                if perm >= 0 as ::core::ffi::c_int
                    && !(perm & __S_IFMT == 0o100000 as ::core::ffi::c_int)
                    && !(perm & __S_IFMT == 0o10000 as ::core::ffi::c_int)
                    && !(perm & __S_IFMT == 0o140000 as ::core::ffi::c_int)
                    && true
                {
                    if perm & __S_IFMT == 0o40000 as ::core::ffi::c_int {
                        if !silent {
                            filemess(curbuf.get(), fname, gettext(msg_is_a_directory.get()));
                        }
                        retval = NOTDONE;
                    } else {
                        filemess(
                            curbuf.get(),
                            fname,
                            gettext(b"is not a file\0".as_ptr() as *const ::core::ffi::c_char),
                        );
                    }
                    msg_end();
                    msg_scroll.set(msg_save);
                    break '_theend;
                }
            }
            set_file_options(set_options, eap);
            check_readonly =
                newfile as ::core::ffi::c_int != 0 && (*curbuf.get()).b_flags & BF_CHECK_RO != 0;
            if check_readonly as ::core::ffi::c_int != 0 && !readonlymode.get() {
                (*curbuf.get()).b_p_ro = false_0;
            }
            if newfile as ::core::ffi::c_int != 0 && !read_stdin && !read_buffer && !read_fifo {
                if os_fileinfo(fname, &raw mut file_info) {
                    buf_store_file_info(curbuf.get(), &raw mut file_info);
                    (*curbuf.get()).b_mtime_read = (*curbuf.get()).b_mtime;
                    (*curbuf.get()).b_mtime_read_ns = (*curbuf.get()).b_mtime_ns;
                    swap_mode = file_info.stat.st_mode as ::core::ffi::c_int
                        & 0o644 as ::core::ffi::c_int
                        | 0o600 as ::core::ffi::c_int;
                } else {
                    (*curbuf.get()).b_mtime = 0 as int64_t;
                    (*curbuf.get()).b_mtime_ns = 0 as int64_t;
                    (*curbuf.get()).b_mtime_read = 0 as int64_t;
                    (*curbuf.get()).b_mtime_read_ns = 0 as int64_t;
                    (*curbuf.get()).b_orig_size = 0 as uint64_t;
                    (*curbuf.get()).b_orig_mode = 0 as ::core::ffi::c_int;
                }
                (*curbuf.get()).b_flags &= !(BF_NEW | BF_NEW_W);
            }
            file_readonly = false_0 != 0;
            if !read_buffer && !read_stdin {
                if !newfile
                    || readonlymode.get() as ::core::ffi::c_int != 0
                    || perm & 0o222 as ::core::ffi::c_int == 0
                    || os_file_is_writable(fname) == 0
                {
                    file_readonly = true_0 != 0;
                }
                fd = os_open(fname, O_RDONLY, 0 as ::core::ffi::c_int);
            }
            if fd < 0 as ::core::ffi::c_int {
                msg_scroll.set(msg_save);
                if newfile {
                    if perm == UV_ENOENT as ::core::ffi::c_int {
                        (*curbuf.get()).b_flags |= BF_NEW;
                        if !bt_dontwrite(curbuf.get()) {
                            check_need_swap(newfile);
                            if curbuf.get() != old_curbuf
                                || using_b_ffname != 0 && old_b_ffname != (*curbuf.get()).b_ffname
                                || using_b_fname != 0 && old_b_fname != (*curbuf.get()).b_fname
                            {
                                emsg(gettext(e_auchangedbuf.get()));
                                break '_theend;
                            }
                        }
                        if !silent {
                            if dir_of_file_exists(fname) {
                                filemess(
                                    curbuf.get(),
                                    sfname,
                                    gettext(b"[New]\0".as_ptr() as *const ::core::ffi::c_char),
                                );
                            } else {
                                filemess(
                                    curbuf.get(),
                                    sfname,
                                    gettext(
                                        b"[New DIRECTORY]\0".as_ptr() as *const ::core::ffi::c_char
                                    ),
                                );
                            }
                        }
                        check_marks_read();
                        if !eap.is_null() {
                            set_forced_fenc(eap);
                        }
                        apply_autocmds_exarg(
                            EVENT_BUFNEWFILE,
                            sfname,
                            sfname,
                            false_0 != 0,
                            curbuf.get(),
                            eap,
                        );
                        save_file_ff(curbuf.get());
                        if !aborting() {
                            retval = OK;
                        }
                    } else {
                        filemess(
                            curbuf.get(),
                            sfname,
                            if fd == UV_EFBIG as ::core::ffi::c_int {
                                gettext(b"[File too big]\0".as_ptr() as *const ::core::ffi::c_char)
                            } else if fd == -EOVERFLOW {
                                gettext(b"[File too big]\0".as_ptr() as *const ::core::ffi::c_char)
                            } else {
                                gettext(
                                    b"[Permission Denied]\0".as_ptr() as *const ::core::ffi::c_char
                                )
                            },
                        );
                        (*curbuf.get()).b_p_ro = true_0;
                    }
                }
            } else {
                if check_readonly as ::core::ffi::c_int != 0
                    && file_readonly as ::core::ffi::c_int != 0
                    || (*curbuf.get()).b_help as ::core::ffi::c_int != 0
                {
                    (*curbuf.get()).b_p_ro = true_0;
                }
                if set_options {
                    if !read_buffer {
                        (*curbuf.get()).b_p_eof = false_0;
                        (*curbuf.get()).b_start_eof = false_0;
                        (*curbuf.get()).b_p_eol = true_0;
                        (*curbuf.get()).b_start_eol = true_0;
                    }
                    (*curbuf.get()).b_p_bomb = false_0;
                    (*curbuf.get()).b_start_bomb = false_0;
                }
                if !bt_dontwrite(curbuf.get()) {
                    check_need_swap(newfile);
                    if !read_stdin
                        && (curbuf.get() != old_curbuf
                            || using_b_ffname != 0 && old_b_ffname != (*curbuf.get()).b_ffname
                            || using_b_fname != 0 && old_b_fname != (*curbuf.get()).b_fname)
                    {
                        emsg(gettext(e_auchangedbuf.get()));
                        if !read_buffer {
                            close(fd);
                        }
                        break '_theend;
                    } else if swap_mode > 0 as ::core::ffi::c_int
                        && !(*curbuf.get()).b_ml.ml_mfp.is_null()
                        && !(*(*curbuf.get()).b_ml.ml_mfp).mf_fname.is_null()
                    {
                        let mut swap_fname: *const ::core::ffi::c_char =
                            (*(*curbuf.get()).b_ml.ml_mfp).mf_fname;
                        if swap_mode & 0o44 as ::core::ffi::c_int == 0o40 as ::core::ffi::c_int {
                            let mut swap_info: FileInfo = FileInfo {
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
                            if os_fileinfo(swap_fname, &raw mut swap_info) as ::core::ffi::c_int
                                != 0
                                && file_info.stat.st_gid != swap_info.stat.st_gid
                                && os_fchown(
                                    (*(*curbuf.get()).b_ml.ml_mfp).mf_fd,
                                    -1 as ::core::ffi::c_int as uv_uid_t,
                                    file_info.stat.st_gid as uv_gid_t,
                                ) == -1 as ::core::ffi::c_int
                            {
                                swap_mode &= 0o600 as ::core::ffi::c_int;
                            }
                        }
                        os_setperm(swap_fname, swap_mode);
                    }
                }
                if swap_exists_action.get() == SEA_QUIT {
                    if !read_buffer && !read_stdin {
                        close(fd);
                    }
                } else {
                    (*no_wait_return.ptr()) += 1;
                    orig_start = (*curbuf.get()).b_op_start;
                    (*curbuf.get()).b_op_start.lnum = if from == 0 as linenr_T {
                        1 as linenr_T
                    } else {
                        from
                    };
                    (*curbuf.get()).b_op_start.col = 0 as ::core::ffi::c_int as colnr_T;
                    try_mac = !vim_strchr(p_ffs.get(), 'm' as ::core::ffi::c_int).is_null()
                        as ::core::ffi::c_int;
                    try_dos = !vim_strchr(p_ffs.get(), 'd' as ::core::ffi::c_int).is_null()
                        as ::core::ffi::c_int;
                    try_unix = !vim_strchr(p_ffs.get(), 'x' as ::core::ffi::c_int).is_null()
                        as ::core::ffi::c_int;
                    if !read_buffer {
                        let mut m: ::core::ffi::c_int = msg_scroll.get();
                        let mut n: ::core::ffi::c_int = msg_scrolled.get();
                        if !read_stdin {
                            close(fd);
                        }
                        msg_scroll.set(true_0);
                        if filtering {
                            apply_autocmds_exarg(
                                EVENT_FILTERREADPRE,
                                ::core::ptr::null_mut::<::core::ffi::c_char>(),
                                sfname,
                                false_0 != 0,
                                curbuf.get(),
                                eap,
                            );
                        } else if read_stdin {
                            apply_autocmds_exarg(
                                EVENT_STDINREADPRE,
                                ::core::ptr::null_mut::<::core::ffi::c_char>(),
                                sfname,
                                false_0 != 0,
                                curbuf.get(),
                                eap,
                            );
                        } else if newfile {
                            apply_autocmds_exarg(
                                EVENT_BUFREADPRE,
                                ::core::ptr::null_mut::<::core::ffi::c_char>(),
                                sfname,
                                false_0 != 0,
                                curbuf.get(),
                                eap,
                            );
                        } else {
                            apply_autocmds_exarg(
                                EVENT_FILEREADPRE,
                                sfname,
                                sfname,
                                false_0 != 0,
                                ::core::ptr::null_mut::<buf_T>(),
                                eap,
                            );
                        }
                        try_mac = !vim_strchr(p_ffs.get(), 'm' as ::core::ffi::c_int).is_null()
                            as ::core::ffi::c_int;
                        try_dos = !vim_strchr(p_ffs.get(), 'd' as ::core::ffi::c_int).is_null()
                            as ::core::ffi::c_int;
                        try_unix = !vim_strchr(p_ffs.get(), 'x' as ::core::ffi::c_int).is_null()
                            as ::core::ffi::c_int;
                        (*curbuf.get()).b_op_start = orig_start;
                        if msg_scrolled.get() == n {
                            msg_scroll.set(m);
                        }
                        if aborting() {
                            (*no_wait_return.ptr()) -= 1;
                            msg_scroll.set(msg_save);
                            (*curbuf.get()).b_p_ro = true_0;
                            break '_theend;
                        } else if !read_stdin
                            && (curbuf.get() != old_curbuf
                                || using_b_ffname != 0 && old_b_ffname != (*curbuf.get()).b_ffname
                                || using_b_fname != 0 && old_b_fname != (*curbuf.get()).b_fname
                                || {
                                    fd = os_open(fname, O_RDONLY, 0 as ::core::ffi::c_int);
                                    fd < 0 as ::core::ffi::c_int
                                })
                        {
                            (*no_wait_return.ptr()) -= 1;
                            msg_scroll.set(msg_save);
                            if fd < 0 as ::core::ffi::c_int {
                                emsg(gettext(
                                    b"E200: *ReadPre autocommands made the file unreadable\0"
                                        .as_ptr()
                                        as *const ::core::ffi::c_char,
                                ));
                            } else {
                                emsg(gettext(
                                    b"E201: *ReadPre autocommands must not change current buffer\0"
                                        .as_ptr()
                                        as *const ::core::ffi::c_char,
                                ));
                            }
                            (*curbuf.get()).b_p_ro = true_0;
                            break '_theend;
                        }
                    }
                    wasempty = (*curbuf.get()).b_ml.ml_flags & ML_EMPTY;
                    if !recoverymode.get()
                        && !filtering
                        && flags & READ_DUMMY as ::core::ffi::c_int == 0
                        && !silent
                    {
                        if !read_stdin && !read_buffer {
                            filemess(
                                curbuf.get(),
                                sfname,
                                b"\0".as_ptr() as *const ::core::ffi::c_char
                                    as *mut ::core::ffi::c_char,
                            );
                        }
                    }
                    msg_scroll.set(false_0);
                    linecnt = (*curbuf.get()).b_ml.ml_line_count;
                    if !eap.is_null() && (*eap).bad_char != 0 as ::core::ffi::c_int {
                        bad_char_behavior = (*eap).bad_char;
                        if set_options {
                            (*curbuf.get()).b_bad_char = (*eap).bad_char;
                        }
                    } else {
                        (*curbuf.get()).b_bad_char = 0 as ::core::ffi::c_int;
                    }
                    if !eap.is_null() && (*eap).force_enc != 0 as ::core::ffi::c_int {
                        fenc = enc_canonize((*eap).cmd.offset((*eap).force_enc as isize));
                        fenc_alloced = true_0 != 0;
                        keep_dest_enc = true_0 != 0;
                    } else if (*curbuf.get()).b_p_bin != 0 {
                        fenc = b"\0".as_ptr() as *const ::core::ffi::c_char
                            as *mut ::core::ffi::c_char;
                        fenc_alloced = false_0 != 0;
                    } else if (*curbuf.get()).b_help {
                        fenc_next = b"latin1\0".as_ptr() as *const ::core::ffi::c_char
                            as *mut ::core::ffi::c_char;
                        fenc = b"utf-8\0".as_ptr() as *const ::core::ffi::c_char
                            as *mut ::core::ffi::c_char;
                        fenc_alloced = false_0 != 0;
                    } else if *p_fencs.get() as ::core::ffi::c_int == NUL {
                        fenc = (*curbuf.get()).b_p_fenc;
                        fenc_alloced = false_0 != 0;
                    } else {
                        fenc_next = p_fencs.get();
                        fenc = next_fenc(&raw mut fenc_next, &raw mut fenc_alloced);
                    }
                    '_failed: loop {
                        if file_rewind {
                            if read_buffer {
                                read_buf_lnum = 1 as ::core::ffi::c_int as linenr_T;
                                read_buf_col = 0 as ::core::ffi::c_int as colnr_T;
                            } else if read_stdin as ::core::ffi::c_int != 0
                                || lseek(fd, 0 as __off_t, SEEK_SET) != 0 as __off_t
                            {
                                error = true_0 != 0;
                                break;
                            }
                            while lnum > from {
                                let c2rust_fresh0 = lnum;
                                lnum = lnum - 1;
                                ml_delete(c2rust_fresh0);
                            }
                            file_rewind = false_0 != 0;
                            if set_options {
                                (*curbuf.get()).b_p_bomb = false_0;
                                (*curbuf.get()).b_start_bomb = false_0;
                            }
                            conv_error = 0 as ::core::ffi::c_int as linenr_T;
                        }
                        if keep_fileformat {
                            keep_fileformat = false_0 != 0;
                        } else if !eap.is_null() && (*eap).force_ff != 0 as ::core::ffi::c_int {
                            fileformat = get_fileformat_force(curbuf.get(), eap);
                            try_mac = false_0;
                            try_dos = try_mac;
                            try_unix = try_dos;
                        } else if (*curbuf.get()).b_p_bin != 0 {
                            fileformat = EOL_UNIX;
                        } else if *p_ffs.get() as ::core::ffi::c_int == NUL {
                            fileformat = get_fileformat(curbuf.get());
                        } else {
                            fileformat = EOL_UNKNOWN;
                        }
                        if iconv_fd
                            != ::core::ptr::with_exposed_provenance_mut::<::core::ffi::c_void>(
                                -1 as ::core::ffi::c_int as usize,
                            )
                        {
                            iconv_close(iconv_fd);
                            iconv_fd =
                                ::core::ptr::with_exposed_provenance_mut::<::core::ffi::c_void>(
                                    -1 as ::core::ffi::c_int as usize,
                                );
                        }
                        if advance_fenc {
                            advance_fenc = false_0 != 0;
                            if !eap.is_null() && (*eap).force_enc != 0 as ::core::ffi::c_int {
                                notconverted = true_0 != 0;
                                conv_error = 0 as ::core::ffi::c_int as linenr_T;
                                if fenc_alloced {
                                    xfree(fenc as *mut ::core::ffi::c_void);
                                }
                                fenc = b"\0".as_ptr() as *const ::core::ffi::c_char
                                    as *mut ::core::ffi::c_char;
                                fenc_alloced = false_0 != 0;
                            } else {
                                if fenc_alloced {
                                    xfree(fenc as *mut ::core::ffi::c_void);
                                }
                                if !fenc_next.is_null() {
                                    fenc = next_fenc(&raw mut fenc_next, &raw mut fenc_alloced);
                                } else {
                                    fenc = b"\0".as_ptr() as *const ::core::ffi::c_char
                                        as *mut ::core::ffi::c_char;
                                    fenc_alloced = false_0 != 0;
                                }
                            }
                            if !tmpname.is_null() {
                                os_remove(tmpname);
                                let mut ptr_: *mut *mut ::core::ffi::c_void =
                                    &raw mut tmpname as *mut *mut ::core::ffi::c_void;
                                xfree(*ptr_);
                                *ptr_ = NULL;
                                let _ = *ptr_;
                            }
                        }
                        fio_flags = 0 as ::core::ffi::c_int;
                        converted = need_conversion(fenc);
                        if converted {
                            if strcmp(fenc, ENC_UCSBOM.as_ptr()) == 0 as ::core::ffi::c_int {
                                fio_flags = FIO_UCSBOM as ::core::ffi::c_int;
                            } else {
                                fio_flags = get_fio_flags(fenc);
                            }
                            if fio_flags == 0 as ::core::ffi::c_int && !did_iconv {
                                iconv_fd = my_iconv_open(
                                    b"utf-8\0".as_ptr() as *const ::core::ffi::c_char
                                        as *mut ::core::ffi::c_char,
                                    fenc,
                                );
                            }
                            if fio_flags == 0 as ::core::ffi::c_int
                                && !read_stdin
                                && !read_buffer
                                && *p_ccv.get() as ::core::ffi::c_int != NUL
                                && !read_fifo
                                && iconv_fd
                                    == ::core::ptr::with_exposed_provenance_mut::<::core::ffi::c_void>(
                                        -1 as ::core::ffi::c_int as usize,
                                    )
                            {
                                did_iconv = false_0 != 0;
                                if tmpname.is_null() {
                                    tmpname = readfile_charconvert(fname, fenc, &raw mut fd);
                                    if tmpname.is_null() {
                                        advance_fenc = true_0 != 0;
                                        if fd >= 0 as ::core::ffi::c_int {
                                            continue;
                                        }
                                        emsg(gettext(
                                            b"E202: Conversion made file unreadable!\0".as_ptr()
                                                as *const ::core::ffi::c_char,
                                        ));
                                        error = true_0 != 0;
                                        break;
                                    }
                                }
                            } else if fio_flags == 0 as ::core::ffi::c_int
                                && iconv_fd
                                    == ::core::ptr::with_exposed_provenance_mut::<::core::ffi::c_void>(
                                        -1 as ::core::ffi::c_int as usize,
                                    )
                            {
                                advance_fenc = true_0 != 0;
                                continue;
                            }
                        }
                        can_retry = *fenc as ::core::ffi::c_int != NUL
                            && !read_stdin
                            && !keep_dest_enc
                            && !read_fifo;
                        if !skip_read {
                            linerest = 0 as ptrdiff_t;
                            filesize = 0 as off_T;
                            skip_count = lines_to_skip;
                            read_count = lines_to_read;
                            conv_restlen = 0 as ::core::ffi::c_int;
                            read_undo_file = newfile as ::core::ffi::c_int != 0
                                && flags & READ_KEEP_UNDO as ::core::ffi::c_int
                                    == 0 as ::core::ffi::c_int
                                && !(*curbuf.get()).b_ffname.is_null()
                                && (*curbuf.get()).b_p_udf != 0
                                && !filtering
                                && !read_fifo
                                && !read_stdin
                                && !read_buffer;
                            if read_undo_file {
                                sha_ctx = Sha256::new();
                            }
                        }
                        's_1469: loop {
                            if !(!error && !got_int.get()) {
                                break '_failed;
                            }
                            if !skip_read {
                                size = if 0x10000 as ::core::ffi::c_int as ptrdiff_t + linerest
                                    < 0x100000 as ::core::ffi::c_int as ptrdiff_t
                                {
                                    0x10000 as ::core::ffi::c_int as ptrdiff_t + linerest
                                } else {
                                    0x100000 as ::core::ffi::c_int as ptrdiff_t
                                };
                            }
                            '_rewind_retry: {
                                if size < 0 as ptrdiff_t
                                    || (size + linerest + 1 as ptrdiff_t) < 0 as ptrdiff_t
                                    || linerest >= MAXCOL as ::core::ffi::c_int as ptrdiff_t - size
                                {
                                    split += 1;
                                    *ptr = NL as ::core::ffi::c_char;
                                    size = 1 as ptrdiff_t;
                                } else if !skip_read {
                                    while size >= 10 as ptrdiff_t {
                                        new_buffer = verbose_try_malloc(
                                            (size as size_t)
                                                .wrapping_add(linerest as size_t)
                                                .wrapping_add(1 as size_t),
                                        )
                                            as *mut ::core::ffi::c_char;
                                        if !new_buffer.is_null() {
                                            break;
                                        }
                                        size /= 2 as ptrdiff_t;
                                    }
                                    if new_buffer.is_null() {
                                        error = true_0 != 0;
                                        break '_failed;
                                    } else {
                                        if linerest != 0 {
                                            memmove(
                                                new_buffer as *mut ::core::ffi::c_void,
                                                ptr.offset(-(linerest as isize))
                                                    as *const ::core::ffi::c_void,
                                                linerest as size_t,
                                            );
                                        }
                                        xfree(buffer as *mut ::core::ffi::c_void);
                                        buffer = new_buffer;
                                        ptr = buffer.offset(linerest as isize);
                                        line_start = buffer;
                                        real_size = size as ::core::ffi::c_int;
                                        if iconv_fd
                                            != ::core::ptr::with_exposed_provenance_mut::<
                                                ::core::ffi::c_void,
                                            >(
                                                -1 as ::core::ffi::c_int as usize
                                            )
                                        {
                                            size = size
                                                / ICONV_MULT as ::core::ffi::c_int as ptrdiff_t;
                                        } else if fio_flags & FIO_LATIN1 as ::core::ffi::c_int != 0
                                        {
                                            size = size / 2 as ptrdiff_t;
                                        } else if fio_flags
                                            & (FIO_UCS2 as ::core::ffi::c_int
                                                | FIO_UTF16 as ::core::ffi::c_int)
                                            != 0
                                        {
                                            size = size * 2 as ptrdiff_t / 3 as ptrdiff_t
                                                & !(1 as ::core::ffi::c_int) as ptrdiff_t;
                                        } else if fio_flags & FIO_UCS4 as ::core::ffi::c_int != 0 {
                                            size = size * 2 as ptrdiff_t / 3 as ptrdiff_t
                                                & !(3 as ::core::ffi::c_int) as ptrdiff_t;
                                        } else if fio_flags == FIO_UCSBOM as ::core::ffi::c_int {
                                            size = size
                                                / ICONV_MULT as ::core::ffi::c_int as ptrdiff_t;
                                        }
                                        if conv_restlen > 0 as ::core::ffi::c_int {
                                            memmove(
                                                ptr as *mut ::core::ffi::c_void,
                                                &raw mut conv_rest as *mut ::core::ffi::c_char
                                                    as *const ::core::ffi::c_void,
                                                conv_restlen as size_t,
                                            );
                                            ptr = ptr.offset(conv_restlen as isize);
                                            size -= conv_restlen as ptrdiff_t;
                                        }
                                        if read_buffer {
                                            if read_buf_lnum > from {
                                                size = 0 as ptrdiff_t;
                                            } else {
                                                let mut ni: ::core::ffi::c_int = 0;
                                                let mut tlen: ::core::ffi::c_int =
                                                    0 as ::core::ffi::c_int;
                                                loop {
                                                    p = (ml_get(read_buf_lnum) as *mut uint8_t)
                                                        .offset(read_buf_col as isize);
                                                    let mut n_0: ::core::ffi::c_int =
                                                        ml_get_len(read_buf_lnum)
                                                            - read_buf_col as ::core::ffi::c_int;
                                                    if (tlen + n_0 + 1 as ::core::ffi::c_int)
                                                        as ptrdiff_t
                                                        > size
                                                    {
                                                        n_0 = (size - tlen as ptrdiff_t)
                                                            as ::core::ffi::c_int;
                                                        ni = 0 as ::core::ffi::c_int;
                                                        while ni < n_0 {
                                                            if *p.offset(ni as isize)
                                                                as ::core::ffi::c_int
                                                                == NL
                                                            {
                                                                let c2rust_fresh1 = tlen;
                                                                tlen = tlen + 1;
                                                                *ptr.offset(
                                                                    c2rust_fresh1 as isize,
                                                                ) = NUL as ::core::ffi::c_char;
                                                            } else {
                                                                let c2rust_fresh2 = tlen;
                                                                tlen = tlen + 1;
                                                                *ptr.offset(
                                                                    c2rust_fresh2 as isize,
                                                                ) = *p.offset(ni as isize)
                                                                    as ::core::ffi::c_char;
                                                            }
                                                            ni += 1;
                                                        }
                                                        read_buf_col += n_0;
                                                        break;
                                                    } else {
                                                        ni = 0 as ::core::ffi::c_int;
                                                        while ni < n_0 {
                                                            if *p.offset(ni as isize)
                                                                as ::core::ffi::c_int
                                                                == NL
                                                            {
                                                                let c2rust_fresh3 = tlen;
                                                                tlen = tlen + 1;
                                                                *ptr.offset(
                                                                    c2rust_fresh3 as isize,
                                                                ) = NUL as ::core::ffi::c_char;
                                                            } else {
                                                                let c2rust_fresh4 = tlen;
                                                                tlen = tlen + 1;
                                                                *ptr.offset(
                                                                    c2rust_fresh4 as isize,
                                                                ) = *p.offset(ni as isize)
                                                                    as ::core::ffi::c_char;
                                                            }
                                                            ni += 1;
                                                        }
                                                        let c2rust_fresh5 = tlen;
                                                        tlen = tlen + 1;
                                                        *ptr.offset(c2rust_fresh5 as isize) =
                                                            NL as ::core::ffi::c_char;
                                                        read_buf_col =
                                                            0 as ::core::ffi::c_int as colnr_T;
                                                        read_buf_lnum += 1;
                                                        if read_buf_lnum <= from {
                                                            continue;
                                                        }
                                                        if (*curbuf.get()).b_p_eol == 0 {
                                                            tlen -= 1;
                                                        }
                                                        size = tlen as ptrdiff_t;
                                                        break;
                                                    }
                                                }
                                            }
                                        } else {
                                            let mut read_size: size_t = size as size_t;
                                            size = read_eintr(
                                                fd,
                                                ptr as *mut ::core::ffi::c_void,
                                                read_size,
                                            )
                                                as ptrdiff_t;
                                        }
                                        if size <= 0 as ptrdiff_t {
                                            if size < 0 as ptrdiff_t {
                                                error = true_0 != 0;
                                            } else if conv_restlen > 0 as ::core::ffi::c_int {
                                                if fio_flags != 0 as ::core::ffi::c_int
                                                    || iconv_fd
                                                        != ::core::ptr::with_exposed_provenance_mut::<
                                                            ::core::ffi::c_void,
                                                        >(
                                                            -1 as ::core::ffi::c_int as usize
                                                        )
                                                {
                                                    if can_retry {
                                                        break '_rewind_retry;
                                                    } else if conv_error == 0 as linenr_T {
                                                        conv_error =
                                                            (*curbuf.get()).b_ml.ml_line_count
                                                                - linecnt
                                                                + 1 as linenr_T;
                                                    }
                                                } else if illegal_byte == 0 as linenr_T {
                                                    illegal_byte =
                                                        (*curbuf.get()).b_ml.ml_line_count
                                                            - linecnt
                                                            + 1 as linenr_T;
                                                }
                                                if bad_char_behavior == BAD_DROP {
                                                    *ptr.offset(-(conv_restlen as isize)) =
                                                        NUL as ::core::ffi::c_char;
                                                    conv_restlen = 0 as ::core::ffi::c_int;
                                                } else {
                                                    if bad_char_behavior != BAD_KEEP
                                                    && (fio_flags != 0 as ::core::ffi::c_int
                                                        || iconv_fd
                                                            != ::core::ptr::with_exposed_provenance_mut::<
                                                                ::core::ffi::c_void,
                                                            >(
                                                                -1 as ::core::ffi::c_int as usize
                                                            ))
                                                {
                                                    while conv_restlen > 0 as ::core::ffi::c_int {
                                                        ptr = ptr.offset(-1);
                                                        *ptr = bad_char_behavior
                                                            as ::core::ffi::c_char;
                                                        conv_restlen -= 1;
                                                    }
                                                }
                                                    fio_flags = 0 as ::core::ffi::c_int;
                                                    if iconv_fd
                                                        != ::core::ptr::with_exposed_provenance_mut::<
                                                            ::core::ffi::c_void,
                                                        >(
                                                            -1 as ::core::ffi::c_int as usize
                                                        )
                                                    {
                                                        iconv_close(iconv_fd);
                                                        iconv_fd =
                                                        ::core::ptr::with_exposed_provenance_mut::<
                                                            ::core::ffi::c_void,
                                                        >(
                                                            -1 as ::core::ffi::c_int as usize
                                                        );
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                                skip_read = false_0 != 0;
                                if filesize == 0 as off_T
                                    && (fio_flags == FIO_UCSBOM as ::core::ffi::c_int
                                        || (*curbuf.get()).b_p_bomb == 0
                                            && tmpname.is_null()
                                            && (*fenc as ::core::ffi::c_int
                                                == 'u' as ::core::ffi::c_int
                                                || *fenc as ::core::ffi::c_int == NUL))
                                {
                                    let mut ccname: *mut ::core::ffi::c_char =
                                        ::core::ptr::null_mut::<::core::ffi::c_char>();
                                    let mut blen: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                                    if size < 2 as ptrdiff_t || (*curbuf.get()).b_p_bin != 0 {
                                        ccname = ::core::ptr::null_mut::<::core::ffi::c_char>();
                                    } else {
                                        ccname = check_for_bom(
                                            ptr,
                                            size as ::core::ffi::c_int,
                                            &raw mut blen,
                                            if fio_flags == FIO_UCSBOM as ::core::ffi::c_int {
                                                FIO_ALL as ::core::ffi::c_int
                                            } else {
                                                get_fio_flags(fenc)
                                            },
                                        );
                                    }
                                    if !ccname.is_null() {
                                        filesize += blen as off_T;
                                        size -= blen as ptrdiff_t;
                                        memmove(
                                            ptr as *mut ::core::ffi::c_void,
                                            ptr.offset(blen as isize) as *const ::core::ffi::c_void,
                                            size as size_t,
                                        );
                                        if set_options {
                                            (*curbuf.get()).b_p_bomb = true_0;
                                            (*curbuf.get()).b_start_bomb = true_0;
                                        }
                                    }
                                    if fio_flags == FIO_UCSBOM as ::core::ffi::c_int {
                                        if ccname.is_null() {
                                            advance_fenc = true_0 != 0;
                                        } else {
                                            if fenc_alloced {
                                                xfree(fenc as *mut ::core::ffi::c_void);
                                            }
                                            fenc = ccname;
                                            fenc_alloced = false_0 != 0;
                                        }
                                        skip_read = true_0 != 0;
                                        break 's_1469;
                                    }
                                }
                                ptr = ptr.offset(-(conv_restlen as isize));
                                size += conv_restlen as ptrdiff_t;
                                conv_restlen = 0 as ::core::ffi::c_int;
                                if size <= 0 as ptrdiff_t {
                                    break '_failed;
                                }
                                if iconv_fd
                                    != ::core::ptr::with_exposed_provenance_mut::<::core::ffi::c_void>(
                                        -1 as ::core::ffi::c_int as usize,
                                    )
                                {
                                    let mut fromp: *const ::core::ffi::c_char = ptr;
                                    let mut from_size: size_t = size as size_t;
                                    ptr = ptr.offset(size as isize);
                                    let mut top: *mut ::core::ffi::c_char = ptr;
                                    let mut to_size: size_t =
                                        (real_size as ptrdiff_t - size) as size_t;
                                    while iconv(
                                        iconv_fd,
                                        &raw mut fromp as *mut ::core::ffi::c_void
                                            as *mut *mut ::core::ffi::c_char,
                                        &raw mut from_size,
                                        &raw mut top,
                                        &raw mut to_size,
                                    ) == -1 as ::core::ffi::c_int as size_t
                                        && *__errno_location() != ICONV_EINVAL
                                        || from_size > CONV_RESTLEN as ::core::ffi::c_int as size_t
                                    {
                                        if can_retry {
                                            break '_rewind_retry;
                                        }
                                        if conv_error == 0 as linenr_T {
                                            conv_error = readfile_linenr(linecnt, ptr, top);
                                        }
                                        fromp = fromp.offset(1);
                                        from_size = from_size.wrapping_sub(1);
                                        if bad_char_behavior == BAD_KEEP {
                                            let c2rust_fresh6 = top;
                                            top = top.offset(1);
                                            *c2rust_fresh6 =
                                                *fromp.offset(-(1 as ::core::ffi::c_int as isize));
                                            to_size = to_size.wrapping_sub(1);
                                        } else if bad_char_behavior != BAD_DROP {
                                            let c2rust_fresh7 = top;
                                            top = top.offset(1);
                                            *c2rust_fresh7 =
                                                bad_char_behavior as ::core::ffi::c_char;
                                            to_size = to_size.wrapping_sub(1);
                                        }
                                    }
                                    if from_size > 0 as size_t {
                                        memmove(
                                            &raw mut conv_rest as *mut ::core::ffi::c_char
                                                as *mut ::core::ffi::c_void,
                                            fromp as *const ::core::ffi::c_void,
                                            from_size,
                                        );
                                        conv_restlen = from_size as ::core::ffi::c_int;
                                    }
                                    line_start = ptr.offset(-(linerest as isize));
                                    memmove(
                                        line_start as *mut ::core::ffi::c_void,
                                        buffer as *const ::core::ffi::c_void,
                                        linerest as size_t,
                                    );
                                    size = top.offset_from(ptr) as ptrdiff_t;
                                }
                                if fio_flags != 0 as ::core::ffi::c_int {
                                    let mut u8c: ::core::ffi::c_uint = 0;
                                    let mut tail: *mut ::core::ffi::c_char =
                                        ::core::ptr::null_mut::<::core::ffi::c_char>();
                                    let mut dest: *mut ::core::ffi::c_char =
                                        ptr.offset(real_size as isize);
                                    if fio_flags == FIO_LATIN1 as ::core::ffi::c_int
                                        || fio_flags == FIO_UTF8 as ::core::ffi::c_int
                                    {
                                        p = (ptr as *mut uint8_t).offset(size as isize);
                                        if fio_flags == FIO_UTF8 as ::core::ffi::c_int {
                                            tail = ptr
                                                .offset(size as isize)
                                                .offset(-(1 as ::core::ffi::c_int as isize));
                                            while tail > ptr
                                                && *tail as ::core::ffi::c_int
                                                    & 0xc0 as ::core::ffi::c_int
                                                    == 0x80 as ::core::ffi::c_int
                                            {
                                                tail = tail.offset(-1);
                                            }
                                            if tail
                                                .offset(utf_byte2len(*tail as ::core::ffi::c_int)
                                                    as isize)
                                                <= ptr.offset(size as isize)
                                            {
                                                tail =
                                                    ::core::ptr::null_mut::<::core::ffi::c_char>();
                                            } else {
                                                p = tail as *mut uint8_t;
                                            }
                                        }
                                    } else if fio_flags
                                        & (FIO_UCS2 as ::core::ffi::c_int
                                            | FIO_UTF16 as ::core::ffi::c_int)
                                        != 0
                                    {
                                        p = (ptr as *mut uint8_t).offset(
                                            (size & !(1 as ::core::ffi::c_int) as ptrdiff_t)
                                                as isize,
                                        );
                                        if size & 1 as ptrdiff_t != 0 {
                                            tail = p as *mut ::core::ffi::c_char;
                                        }
                                        if fio_flags & FIO_UTF16 as ::core::ffi::c_int != 0
                                            && p > ptr as *mut uint8_t
                                        {
                                            if fio_flags & FIO_ENDIAN_L as ::core::ffi::c_int != 0 {
                                                p = p.offset(-1);
                                                u8c = (*p as ::core::ffi::c_uint)
                                                    << 8 as ::core::ffi::c_int;
                                                p = p.offset(-1);
                                                u8c = u8c.wrapping_add(*p as ::core::ffi::c_uint);
                                            } else {
                                                p = p.offset(-1);
                                                u8c = *p as ::core::ffi::c_uint;
                                                p = p.offset(-1);
                                                u8c = u8c.wrapping_add(
                                                    (*p as ::core::ffi::c_uint)
                                                        << 8 as ::core::ffi::c_int,
                                                );
                                            }
                                            if u8c >= 0xd800 as ::core::ffi::c_uint
                                                && u8c <= 0xdbff as ::core::ffi::c_uint
                                            {
                                                tail = p as *mut ::core::ffi::c_char;
                                            } else {
                                                p = p.offset(2 as ::core::ffi::c_int as isize);
                                            }
                                        }
                                    } else {
                                        p = (ptr as *mut uint8_t).offset(
                                            (size & !(3 as ::core::ffi::c_int) as ptrdiff_t)
                                                as isize,
                                        );
                                        if size & 3 as ptrdiff_t != 0 {
                                            tail = p as *mut ::core::ffi::c_char;
                                        }
                                    }
                                    if !tail.is_null() {
                                        conv_restlen = ptr.offset(size as isize).offset_from(tail)
                                            as ::core::ffi::c_int;
                                        memmove(
                                            &raw mut conv_rest as *mut ::core::ffi::c_char
                                                as *mut ::core::ffi::c_void,
                                            tail as *const ::core::ffi::c_void,
                                            conv_restlen as size_t,
                                        );
                                        size -= conv_restlen as ptrdiff_t;
                                    }
                                    while p > ptr as *mut uint8_t {
                                        if fio_flags & FIO_LATIN1 as ::core::ffi::c_int != 0 {
                                            p = p.offset(-1);
                                            u8c = *p as ::core::ffi::c_uint;
                                        } else if fio_flags
                                            & (FIO_UCS2 as ::core::ffi::c_int
                                                | FIO_UTF16 as ::core::ffi::c_int)
                                            != 0
                                        {
                                            if fio_flags & FIO_ENDIAN_L as ::core::ffi::c_int != 0 {
                                                p = p.offset(-1);
                                                u8c = (*p as ::core::ffi::c_uint)
                                                    << 8 as ::core::ffi::c_int;
                                                p = p.offset(-1);
                                                u8c = u8c.wrapping_add(*p as ::core::ffi::c_uint);
                                            } else {
                                                p = p.offset(-1);
                                                u8c = *p as ::core::ffi::c_uint;
                                                p = p.offset(-1);
                                                u8c = u8c.wrapping_add(
                                                    (*p as ::core::ffi::c_uint)
                                                        << 8 as ::core::ffi::c_int,
                                                );
                                            }
                                            if fio_flags & FIO_UTF16 as ::core::ffi::c_int != 0
                                                && u8c >= 0xdc00 as ::core::ffi::c_uint
                                                && u8c <= 0xdfff as ::core::ffi::c_uint
                                            {
                                                let mut u16c: ::core::ffi::c_int = 0;
                                                if p == ptr as *mut uint8_t {
                                                    if can_retry {
                                                        break '_rewind_retry;
                                                    }
                                                    if conv_error == 0 as linenr_T {
                                                        conv_error = readfile_linenr(
                                                            linecnt,
                                                            ptr,
                                                            p as *mut ::core::ffi::c_char,
                                                        );
                                                    }
                                                    if bad_char_behavior == BAD_DROP {
                                                        continue;
                                                    }
                                                    if bad_char_behavior != BAD_KEEP {
                                                        u8c = bad_char_behavior
                                                            as ::core::ffi::c_uint;
                                                    }
                                                }
                                                if fio_flags & FIO_ENDIAN_L as ::core::ffi::c_int
                                                    != 0
                                                {
                                                    p = p.offset(-1);
                                                    u16c = (*p as ::core::ffi::c_int)
                                                        << 8 as ::core::ffi::c_int;
                                                    p = p.offset(-1);
                                                    u16c += *p as ::core::ffi::c_int;
                                                } else {
                                                    p = p.offset(-1);
                                                    u16c = *p as ::core::ffi::c_int;
                                                    p = p.offset(-1);
                                                    u16c += (*p as ::core::ffi::c_int)
                                                        << 8 as ::core::ffi::c_int;
                                                }
                                                u8c = (0x10000 as ::core::ffi::c_int
                                                    as ::core::ffi::c_uint)
                                                    .wrapping_add(
                                                        (u16c as ::core::ffi::c_uint
                                                            & 0x3ff as ::core::ffi::c_uint)
                                                            << 10 as ::core::ffi::c_int,
                                                    )
                                                    .wrapping_add(
                                                        u8c & 0x3ff as ::core::ffi::c_uint,
                                                    );
                                                if u16c < 0xd800 as ::core::ffi::c_int
                                                    || u16c > 0xdbff as ::core::ffi::c_int
                                                {
                                                    if can_retry {
                                                        break '_rewind_retry;
                                                    }
                                                    if conv_error == 0 as linenr_T {
                                                        conv_error = readfile_linenr(
                                                            linecnt,
                                                            ptr,
                                                            p as *mut ::core::ffi::c_char,
                                                        );
                                                    }
                                                    if bad_char_behavior == BAD_DROP {
                                                        continue;
                                                    }
                                                    if bad_char_behavior != BAD_KEEP {
                                                        u8c = bad_char_behavior
                                                            as ::core::ffi::c_uint;
                                                    }
                                                }
                                            }
                                        } else if fio_flags & FIO_UCS4 as ::core::ffi::c_int != 0 {
                                            if fio_flags & FIO_ENDIAN_L as ::core::ffi::c_int != 0 {
                                                p = p.offset(-1);
                                                u8c = (*p as ::core::ffi::c_uint)
                                                    << 24 as ::core::ffi::c_int;
                                                p = p.offset(-1);
                                                u8c = u8c.wrapping_add(
                                                    (*p as ::core::ffi::c_uint)
                                                        << 16 as ::core::ffi::c_int,
                                                );
                                                p = p.offset(-1);
                                                u8c = u8c.wrapping_add(
                                                    (*p as ::core::ffi::c_uint)
                                                        << 8 as ::core::ffi::c_int,
                                                );
                                                p = p.offset(-1);
                                                u8c = u8c.wrapping_add(*p as ::core::ffi::c_uint);
                                            } else {
                                                p = p.offset(-1);
                                                u8c = *p as ::core::ffi::c_uint;
                                                p = p.offset(-1);
                                                u8c = u8c.wrapping_add(
                                                    (*p as ::core::ffi::c_uint)
                                                        << 8 as ::core::ffi::c_int,
                                                );
                                                p = p.offset(-1);
                                                u8c = u8c.wrapping_add(
                                                    (*p as ::core::ffi::c_uint)
                                                        << 16 as ::core::ffi::c_int,
                                                );
                                                p = p.offset(-1);
                                                u8c = u8c.wrapping_add(
                                                    (*p as ::core::ffi::c_uint)
                                                        << 24 as ::core::ffi::c_int,
                                                );
                                            }
                                            if u8c > INT_MAX as ::core::ffi::c_uint {
                                                u8c = 0xfffd as ::core::ffi::c_uint;
                                            }
                                        } else {
                                            p = p.offset(-1);
                                            if (*p as ::core::ffi::c_int)
                                                < 0x80 as ::core::ffi::c_int
                                            {
                                                u8c = *p as ::core::ffi::c_uint;
                                            } else {
                                                len = utf_head_off(
                                                    ptr,
                                                    p as *mut ::core::ffi::c_char,
                                                )
                                                    as colnr_T;
                                                p = p.offset(-(len as isize));
                                                u8c = utf_ptr2char(p as *mut ::core::ffi::c_char)
                                                    as ::core::ffi::c_uint;
                                                if len == 0 as ::core::ffi::c_int {
                                                    if can_retry {
                                                        break '_rewind_retry;
                                                    }
                                                    if conv_error == 0 as linenr_T {
                                                        conv_error = readfile_linenr(
                                                            linecnt,
                                                            ptr,
                                                            p as *mut ::core::ffi::c_char,
                                                        );
                                                    }
                                                    if bad_char_behavior == BAD_DROP {
                                                        continue;
                                                    }
                                                    if bad_char_behavior != BAD_KEEP {
                                                        u8c = bad_char_behavior
                                                            as ::core::ffi::c_uint;
                                                    }
                                                }
                                            }
                                        }
                                        '_c2rust_label: {
                                            if u8c
                                                <= 2147483647 as ::core::ffi::c_int
                                                    as ::core::ffi::c_uint
                                            {
                                            } else {
                                                __assert_fail(
                                                b"u8c <= INT_MAX\0".as_ptr() as *const ::core::ffi::c_char,
                                                b"src/nvim/fileio.rs\0"
                                                    .as_ptr() as *const ::core::ffi::c_char,
                                                1328 as ::core::ffi::c_uint,
                                                b"int readfile(char *, char *, linenr_T, linenr_T, linenr_T, exarg_T *, int, _Bool)\0"
                                                    .as_ptr() as *const ::core::ffi::c_char,
                                            );
                                            }
                                        };
                                        dest = dest.offset(
                                            -(utf_char2len(u8c as ::core::ffi::c_int) as isize),
                                        );
                                        utf_char2bytes(u8c as ::core::ffi::c_int, dest);
                                    }
                                    line_start = dest.offset(-(linerest as isize));
                                    memmove(
                                        line_start as *mut ::core::ffi::c_void,
                                        buffer as *const ::core::ffi::c_void,
                                        linerest as size_t,
                                    );
                                    size = ptr.offset(real_size as isize).offset_from(dest)
                                        as ptrdiff_t;
                                    ptr = dest;
                                } else if (*curbuf.get()).b_p_bin == 0 {
                                    incomplete_tail = false_0 != 0;
                                    p = ptr as *mut uint8_t;
                                    loop {
                                        let mut ascii_end: *mut uint8_t =
                                            (ptr as *mut uint8_t).offset(size as isize);
                                        while ascii_end.offset_from(p)
                                            >= ::core::mem::size_of::<uint64_t>() as ptrdiff_t
                                        {
                                            let mut word: uint64_t = 0;
                                            memcpy(
                                                &raw mut word as *mut ::core::ffi::c_void,
                                                p as *const ::core::ffi::c_void,
                                                ::core::mem::size_of::<uint64_t>(),
                                            );
                                            if word & NONASCII_MASK != 0 {
                                                break;
                                            }
                                            p =
                                                p.offset(
                                                    ::core::mem::size_of::<uint64_t>() as isize
                                                );
                                        }
                                        while p < ascii_end
                                            && (*p as ::core::ffi::c_int)
                                                < 0x80 as ::core::ffi::c_int
                                        {
                                            p = p.offset(1);
                                        }
                                        let mut todo: ::core::ffi::c_int = (ptr as *mut uint8_t)
                                            .offset(size as isize)
                                            .offset_from(p)
                                            as ::core::ffi::c_int;
                                        if todo <= 0 as ::core::ffi::c_int {
                                            break;
                                        }
                                        if (*p as ::core::ffi::c_int) < 0x80 as ::core::ffi::c_int {
                                            continue;
                                        }
                                        let mut l: ::core::ffi::c_int =
                                            utf_ptr2len_len(p as *mut ::core::ffi::c_char, todo);
                                        if l > todo && !incomplete_tail {
                                            if p > ptr as *mut uint8_t || filesize > 0 as off_T {
                                                incomplete_tail = true_0 != 0;
                                            }
                                            if p > ptr as *mut uint8_t {
                                                conv_restlen = todo;
                                                memmove(
                                                    &raw mut conv_rest as *mut ::core::ffi::c_char
                                                        as *mut ::core::ffi::c_void,
                                                    p as *const ::core::ffi::c_void,
                                                    conv_restlen as size_t,
                                                );
                                                size -= conv_restlen as ptrdiff_t;
                                                break;
                                            }
                                        }
                                        if l == 1 as ::core::ffi::c_int || l > todo {
                                            if can_retry as ::core::ffi::c_int != 0
                                                && !incomplete_tail
                                            {
                                                break;
                                            }
                                            if iconv_fd
                                                != ::core::ptr::with_exposed_provenance_mut::<
                                                    ::core::ffi::c_void,
                                                >(
                                                    -1 as ::core::ffi::c_int as usize
                                                )
                                                && conv_error == 0 as linenr_T
                                            {
                                                conv_error = readfile_linenr(
                                                    linecnt,
                                                    ptr,
                                                    p as *mut ::core::ffi::c_char,
                                                );
                                            }
                                            if conv_error == 0 as linenr_T
                                                && illegal_byte == 0 as linenr_T
                                            {
                                                illegal_byte = readfile_linenr(
                                                    linecnt,
                                                    ptr,
                                                    p as *mut ::core::ffi::c_char,
                                                );
                                            }
                                            if bad_char_behavior == BAD_DROP {
                                                memmove(
                                                    p as *mut ::core::ffi::c_void,
                                                    p.offset(1 as ::core::ffi::c_int as isize)
                                                        as *const ::core::ffi::c_void,
                                                    (todo - 1 as ::core::ffi::c_int) as size_t,
                                                );
                                                size -= 1;
                                            } else {
                                                if bad_char_behavior != BAD_KEEP {
                                                    *p = bad_char_behavior as uint8_t;
                                                }
                                                p = p.offset(1);
                                            }
                                        } else {
                                            p = p.offset(l as isize);
                                        }
                                    }
                                    if p < (ptr as *mut uint8_t).offset(size as isize)
                                        && !incomplete_tail
                                    {
                                        break '_rewind_retry;
                                    }
                                }
                                filesize += size as ::core::ffi::c_long;
                                if fileformat == EOL_UNKNOWN {
                                    if try_dos != 0 || try_unix != 0 {
                                        if try_mac != 0 {
                                            try_mac = 1 as ::core::ffi::c_int;
                                        }
                                        p = ptr as *mut uint8_t;
                                        while p < (ptr as *mut uint8_t).offset(size as isize) {
                                            if *p as ::core::ffi::c_int == NL {
                                                if try_unix == 0
                                                    || try_dos != 0
                                                        && p > ptr as *mut uint8_t
                                                        && *p.offset(
                                                            -1 as ::core::ffi::c_int as isize,
                                                        )
                                                            as ::core::ffi::c_int
                                                            == CAR
                                                {
                                                    fileformat = EOL_DOS;
                                                } else {
                                                    fileformat = EOL_UNIX;
                                                }
                                                break;
                                            } else {
                                                if *p as ::core::ffi::c_int == CAR && try_mac != 0 {
                                                    try_mac += 1;
                                                }
                                                p = p.offset(1);
                                            }
                                        }
                                        if fileformat == EOL_UNIX && try_mac != 0 {
                                            try_mac = 1 as ::core::ffi::c_int;
                                            try_unix = 1 as ::core::ffi::c_int;
                                            while p >= ptr as *mut uint8_t
                                                && *p as ::core::ffi::c_int != CAR
                                            {
                                                p = p.offset(-1);
                                            }
                                            if p >= ptr as *mut uint8_t {
                                                p = ptr as *mut uint8_t;
                                                while p
                                                    < (ptr as *mut uint8_t).offset(size as isize)
                                                {
                                                    if *p as ::core::ffi::c_int == NL {
                                                        try_unix += 1;
                                                    } else if *p as ::core::ffi::c_int == CAR {
                                                        try_mac += 1;
                                                    }
                                                    p = p.offset(1);
                                                }
                                                if try_mac > try_unix {
                                                    fileformat = EOL_MAC;
                                                }
                                            }
                                        } else if fileformat == EOL_UNKNOWN
                                            && try_mac == 1 as ::core::ffi::c_int
                                        {
                                            fileformat = default_fileformat();
                                        }
                                    }
                                    if fileformat == EOL_UNKNOWN && try_mac != 0 {
                                        fileformat = EOL_MAC;
                                    }
                                    if fileformat == EOL_UNKNOWN {
                                        fileformat = default_fileformat();
                                    }
                                    if set_options {
                                        set_fileformat(fileformat, OPT_LOCAL as ::core::ffi::c_int);
                                    }
                                }
                                if fileformat == EOL_MAC {
                                    ptr = ptr.offset(-1);
                                    loop {
                                        ptr = ptr.offset(1);
                                        size -= 1;
                                        if size < 0 as ptrdiff_t {
                                            break;
                                        }
                                        c = *ptr;
                                        if c as ::core::ffi::c_int != NUL
                                            && c as ::core::ffi::c_int != CAR
                                            && c as ::core::ffi::c_int != NL
                                        {
                                            continue;
                                        }
                                        if c as ::core::ffi::c_int == NUL {
                                            *ptr = NL as ::core::ffi::c_char;
                                        } else if c as ::core::ffi::c_int == NL {
                                            *ptr = CAR as ::core::ffi::c_char;
                                        } else {
                                            if skip_count == 0 as linenr_T {
                                                *ptr = NUL as ::core::ffi::c_char;
                                                len = (ptr.offset_from(line_start) + 1 as isize)
                                                    as colnr_T;
                                                if ml_append(lnum, line_start, len, newfile) == FAIL
                                                {
                                                    error = true_0 != 0;
                                                    break;
                                                } else {
                                                    if read_undo_file {
                                                        sha_ctx.update(
                                                            ::core::slice::from_raw_parts(
                                                                line_start as *const u8,
                                                                len as usize,
                                                            ),
                                                        );
                                                    }
                                                    lnum += 1;
                                                    read_count -= 1;
                                                    if read_count == 0 as linenr_T {
                                                        error = true_0 != 0;
                                                        line_start = ptr;
                                                        break;
                                                    }
                                                }
                                            } else {
                                                skip_count -= 1;
                                            }
                                            line_start =
                                                ptr.offset(1 as ::core::ffi::c_int as isize);
                                        }
                                    }
                                } else {
                                    let mut end: *mut ::core::ffi::c_char =
                                        ptr.offset(size as isize);
                                    while ptr < end {
                                        let mut nl: *mut ::core::ffi::c_char = memchr(
                                            ptr as *const ::core::ffi::c_void,
                                            NL,
                                            end.offset_from(ptr) as size_t,
                                        )
                                            as *mut ::core::ffi::c_char;
                                        let mut nul_scan: *mut ::core::ffi::c_char =
                                            ::core::ptr::null_mut::<::core::ffi::c_char>();
                                        if nl.is_null() {
                                            loop {
                                                nul_scan = memchr(
                                                    ptr as *const ::core::ffi::c_void,
                                                    NUL,
                                                    end.offset_from(ptr) as size_t,
                                                )
                                                    as *mut ::core::ffi::c_char;
                                                if nul_scan.is_null() {
                                                    break;
                                                }
                                                *nul_scan = NL as ::core::ffi::c_char;
                                                ptr = nul_scan
                                                    .offset(1 as ::core::ffi::c_int as isize);
                                            }
                                            ptr = end;
                                            break;
                                        } else {
                                            let mut scan: *mut ::core::ffi::c_char = ptr;
                                            loop {
                                                nul_scan = memchr(
                                                    scan as *const ::core::ffi::c_void,
                                                    NUL,
                                                    nl.offset_from(scan) as size_t,
                                                )
                                                    as *mut ::core::ffi::c_char;
                                                if nul_scan.is_null() {
                                                    break;
                                                }
                                                *nul_scan = NL as ::core::ffi::c_char;
                                                scan = nul_scan
                                                    .offset(1 as ::core::ffi::c_int as isize);
                                            }
                                            ptr = nl;
                                            if skip_count == 0 as linenr_T {
                                                *ptr = NUL as ::core::ffi::c_char;
                                                len = (ptr.offset_from(line_start) + 1 as isize)
                                                    as colnr_T;
                                                if fileformat == EOL_DOS {
                                                    if ptr > line_start
                                                        && *ptr.offset(
                                                            -1 as ::core::ffi::c_int as isize,
                                                        )
                                                            as ::core::ffi::c_int
                                                            == CAR
                                                    {
                                                        *ptr.offset(
                                                            -1 as ::core::ffi::c_int as isize,
                                                        ) = NUL as ::core::ffi::c_char;
                                                        len -= 1;
                                                    } else if ff_error != EOL_DOS {
                                                        if try_unix != 0
                                                            && !read_stdin
                                                            && (read_buffer as ::core::ffi::c_int
                                                                != 0
                                                                || lseek(
                                                                    fd,
                                                                    0 as __off_t,
                                                                    SEEK_SET,
                                                                ) == 0 as __off_t)
                                                        {
                                                            fileformat = EOL_UNIX;
                                                            if set_options {
                                                                set_fileformat(
                                                                    EOL_UNIX,
                                                                    OPT_LOCAL as ::core::ffi::c_int,
                                                                );
                                                            }
                                                            file_rewind = true_0 != 0;
                                                            keep_fileformat = true_0 != 0;
                                                            continue '_failed;
                                                        } else {
                                                            ff_error = EOL_DOS;
                                                        }
                                                    }
                                                }
                                                if ml_append(lnum, line_start, len, newfile) == FAIL
                                                {
                                                    error = true_0 != 0;
                                                    break;
                                                } else {
                                                    if read_undo_file {
                                                        sha_ctx.update(
                                                            ::core::slice::from_raw_parts(
                                                                line_start as *const u8,
                                                                len as usize,
                                                            ),
                                                        );
                                                    }
                                                    lnum += 1;
                                                    read_count -= 1;
                                                    if read_count == 0 as linenr_T {
                                                        error = true_0 != 0;
                                                        line_start = ptr;
                                                        break;
                                                    }
                                                }
                                            } else {
                                                skip_count -= 1;
                                            }
                                            line_start =
                                                ptr.offset(1 as ::core::ffi::c_int as isize);
                                            ptr = ptr.offset(1);
                                        }
                                    }
                                    size = -1 as ptrdiff_t;
                                }
                                linerest = ptr.offset_from(line_start) as ptrdiff_t;
                                os_breakcheck();
                                continue 's_1469;
                            }
                            if *p_ccv.get() as ::core::ffi::c_int != NUL
                                && iconv_fd
                                    != ::core::ptr::with_exposed_provenance_mut::<::core::ffi::c_void>(
                                        -1 as ::core::ffi::c_int as usize,
                                    )
                            {
                                did_iconv = true_0 != 0;
                            } else {
                                advance_fenc = true_0 != 0;
                            }
                            file_rewind = true_0 != 0;
                            break;
                        }
                    }
                    if error as ::core::ffi::c_int != 0 && read_count == 0 as linenr_T {
                        error = false_0 != 0;
                    }
                    if linerest != 0 as ptrdiff_t
                        && (*curbuf.get()).b_p_bin == 0
                        && fileformat == EOL_DOS
                        && *ptr.offset(-1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                            == Ctrl_Z
                    {
                        ptr = ptr.offset(-1);
                        linerest -= 1;
                        if set_options {
                            (*curbuf.get()).b_p_eof = true_0;
                        }
                    }
                    if !error && !got_int.get() && linerest != 0 as ptrdiff_t {
                        if set_options {
                            (*curbuf.get()).b_p_eol = false_0;
                        }
                        *ptr = NUL as ::core::ffi::c_char;
                        len = (ptr.offset_from(line_start) + 1 as isize) as colnr_T;
                        if ml_append(lnum, line_start, len, newfile) == FAIL {
                            error = true_0 != 0;
                        } else {
                            if read_undo_file {
                                sha_ctx.update(::core::slice::from_raw_parts(
                                    line_start as *const u8,
                                    len as usize,
                                ));
                            }
                            lnum += 1;
                            read_no_eol_lnum = lnum;
                        }
                    }
                    if set_options {
                        save_file_ff(curbuf.get());
                        set_option_direct(
                            kOptFileencoding,
                            OptVal {
                                type_0: kOptValTypeString,
                                data: OptValData {
                                    string: cstr_as_string(fenc),
                                },
                            },
                            OPT_LOCAL as ::core::ffi::c_int,
                            0 as scid_T,
                        );
                    }
                    if fenc_alloced {
                        xfree(fenc as *mut ::core::ffi::c_void);
                    }
                    if iconv_fd
                        != ::core::ptr::with_exposed_provenance_mut::<::core::ffi::c_void>(
                            -1 as ::core::ffi::c_int as usize,
                        )
                    {
                        iconv_close(iconv_fd);
                    }
                    if !read_buffer && !read_stdin {
                        close(fd);
                    } else {
                        os_set_cloexec(fd);
                    }
                    xfree(buffer as *mut ::core::ffi::c_void);
                    if read_stdin {
                        close(fd);
                        if stdin_fd.get() < 0 as ::core::ffi::c_int {
                            vim_ignored.set(dup(2 as ::core::ffi::c_int));
                        }
                    }
                    if !tmpname.is_null() {
                        os_remove(tmpname);
                        xfree(tmpname as *mut ::core::ffi::c_void);
                    }
                    (*no_wait_return.ptr()) -= 1;
                    if !recoverymode.get() {
                        if newfile as ::core::ffi::c_int != 0
                            && wasempty != 0
                            && (*curbuf.get()).b_ml.ml_flags & ML_EMPTY == 0
                        {
                            ml_delete((*curbuf.get()).b_ml.ml_line_count);
                            linecnt -= 1;
                        }
                        (*curbuf.get()).deleted_bytes = 0 as size_t;
                        (*curbuf.get()).deleted_bytes2 = 0 as size_t;
                        (*curbuf.get()).deleted_codepoints = 0 as size_t;
                        (*curbuf.get()).deleted_codeunits = 0 as size_t;
                        linecnt = (*curbuf.get()).b_ml.ml_line_count - linecnt;
                        if filesize == 0 as off_T {
                            linecnt = 0 as ::core::ffi::c_int as linenr_T;
                        }
                        if newfile as ::core::ffi::c_int != 0
                            || read_buffer as ::core::ffi::c_int != 0
                        {
                            redraw_curbuf_later(UPD_NOT_VALID);
                            diff_invalidate(curbuf.get());
                            foldUpdateAll(curwin.get());
                        } else if linecnt != 0 {
                            appended_lines_mark(from, linecnt as ::core::ffi::c_int);
                        }
                        if got_int.get() {
                            if flags & READ_DUMMY as ::core::ffi::c_int == 0 {
                                filemess(
                                    curbuf.get(),
                                    sfname,
                                    gettext(&raw const e_interr as *const ::core::ffi::c_char),
                                );
                                if newfile {
                                    (*curbuf.get()).b_p_ro = true_0;
                                }
                            }
                            msg_scroll.set(msg_save);
                            check_marks_read();
                            retval = OK;
                            break '_theend;
                        } else {
                            if !filtering
                                && flags & READ_DUMMY as ::core::ffi::c_int == 0
                                && !silent
                            {
                                add_quoted_fname(
                                    IObuff.ptr() as *mut ::core::ffi::c_char,
                                    IOSIZE as size_t,
                                    curbuf.get(),
                                    sfname,
                                );
                                c = false_0 as ::core::ffi::c_char;
                                let mut buflen: ::core::ffi::c_int =
                                    strlen(IObuff.ptr() as *mut ::core::ffi::c_char)
                                        as ::core::ffi::c_int;
                                if perm & __S_IFMT == 0o10000 as ::core::ffi::c_int {
                                    buflen += snprintf(
                                        (IObuff.ptr() as *mut ::core::ffi::c_char)
                                            .offset(buflen as isize),
                                        (IOSIZE - buflen) as size_t,
                                        gettext(b"[fifo]\0".as_ptr() as *const ::core::ffi::c_char),
                                    );
                                    c = true_0 as ::core::ffi::c_char;
                                }
                                if perm & __S_IFMT == 0o140000 as ::core::ffi::c_int {
                                    buflen += snprintf(
                                        (IObuff.ptr() as *mut ::core::ffi::c_char)
                                            .offset(buflen as isize),
                                        (IOSIZE - buflen) as size_t,
                                        gettext(
                                            b"[socket]\0".as_ptr() as *const ::core::ffi::c_char
                                        ),
                                    );
                                    c = true_0 as ::core::ffi::c_char;
                                }
                                if (*curbuf.get()).b_p_ro != 0 {
                                    buflen += snprintf(
                                        (IObuff.ptr() as *mut ::core::ffi::c_char)
                                            .offset(buflen as isize),
                                        (IOSIZE - buflen) as size_t,
                                        b"%s\0".as_ptr() as *const ::core::ffi::c_char,
                                        if shortmess(SHM_RO as ::core::ffi::c_int)
                                            as ::core::ffi::c_int
                                            != 0
                                        {
                                            gettext(b"[RO]\0".as_ptr() as *const ::core::ffi::c_char)
                                        } else {
                                            gettext(b"[readonly]\0".as_ptr()
                                                as *const ::core::ffi::c_char)
                                        },
                                    );
                                    c = true_0 as ::core::ffi::c_char;
                                }
                                if read_no_eol_lnum != 0 {
                                    buflen += snprintf(
                                        (IObuff.ptr() as *mut ::core::ffi::c_char)
                                            .offset(buflen as isize),
                                        (IOSIZE - buflen) as size_t,
                                        gettext(b"[noeol]\0".as_ptr() as *const ::core::ffi::c_char),
                                    );
                                    c = true_0 as ::core::ffi::c_char;
                                }
                                if ff_error == EOL_DOS {
                                    buflen += snprintf(
                                        (IObuff.ptr() as *mut ::core::ffi::c_char)
                                            .offset(buflen as isize),
                                        (IOSIZE - buflen) as size_t,
                                        gettext(b"[CR missing]\0".as_ptr()
                                            as *const ::core::ffi::c_char),
                                    );
                                    c = true_0 as ::core::ffi::c_char;
                                }
                                if split != 0 {
                                    buflen += snprintf(
                                        (IObuff.ptr() as *mut ::core::ffi::c_char)
                                            .offset(buflen as isize),
                                        (IOSIZE - buflen) as size_t,
                                        gettext(b"[long lines split]\0".as_ptr()
                                            as *const ::core::ffi::c_char),
                                    );
                                    c = true_0 as ::core::ffi::c_char;
                                }
                                if notconverted {
                                    buflen += snprintf(
                                        (IObuff.ptr() as *mut ::core::ffi::c_char)
                                            .offset(buflen as isize),
                                        (IOSIZE - buflen) as size_t,
                                        gettext(b"[NOT converted]\0".as_ptr()
                                            as *const ::core::ffi::c_char),
                                    );
                                    c = true_0 as ::core::ffi::c_char;
                                } else if converted {
                                    buflen += snprintf(
                                        (IObuff.ptr() as *mut ::core::ffi::c_char)
                                            .offset(buflen as isize),
                                        (IOSIZE - buflen) as size_t,
                                        gettext(
                                            b"[converted]\0".as_ptr() as *const ::core::ffi::c_char
                                        ),
                                    );
                                    c = true_0 as ::core::ffi::c_char;
                                }
                                if conv_error != 0 as linenr_T {
                                    snprintf(
                                        (IObuff.ptr() as *mut ::core::ffi::c_char)
                                            .offset(buflen as isize),
                                        (IOSIZE - buflen) as size_t,
                                        gettext(b"[CONVERSION ERROR in line %ld]\0".as_ptr()
                                            as *const ::core::ffi::c_char),
                                        conv_error as int64_t,
                                    );
                                    c = true_0 as ::core::ffi::c_char;
                                } else if illegal_byte > 0 as linenr_T {
                                    snprintf(
                                        (IObuff.ptr() as *mut ::core::ffi::c_char)
                                            .offset(buflen as isize),
                                        (IOSIZE - buflen) as size_t,
                                        gettext(b"[ILLEGAL BYTE in line %ld]\0".as_ptr()
                                            as *const ::core::ffi::c_char),
                                        illegal_byte as int64_t,
                                    );
                                    c = true_0 as ::core::ffi::c_char;
                                } else if error {
                                    snprintf(
                                        (IObuff.ptr() as *mut ::core::ffi::c_char)
                                            .offset(buflen as isize),
                                        (IOSIZE - buflen) as size_t,
                                        gettext(b"[READ ERRORS]\0".as_ptr()
                                            as *const ::core::ffi::c_char),
                                    );
                                    c = true_0 as ::core::ffi::c_char;
                                }
                                if msg_add_fileformat(fileformat) {
                                    c = true_0 as ::core::ffi::c_char;
                                }
                                msg_add_lines(c as ::core::ffi::c_int, linecnt, filesize);
                                let mut ptr__0: *mut *mut ::core::ffi::c_void =
                                    keep_msg.ptr() as *mut *mut ::core::ffi::c_void;
                                xfree(*ptr__0);
                                *ptr__0 = NULL;
                                let _ = *ptr__0;
                                p = ::core::ptr::null_mut::<uint8_t>();
                                msg_scrolled_ign.set(true_0 != 0);
                                if !read_stdin && !read_buffer {
                                    if msg_col.get() > 0 as ::core::ffi::c_int {
                                        msg_putchar('\r' as ::core::ffi::c_int);
                                    }
                                    p = msg_trunc(
                                        IObuff.ptr() as *mut ::core::ffi::c_char,
                                        false_0 != 0,
                                        0 as ::core::ffi::c_int,
                                    ) as *mut uint8_t;
                                }
                                if read_stdin as ::core::ffi::c_int != 0
                                    || read_buffer as ::core::ffi::c_int != 0
                                    || restart_edit.get() != 0 as ::core::ffi::c_int
                                    || msg_scrolled.get() != 0 as ::core::ffi::c_int
                                        && !need_wait_return.get()
                                {
                                    set_keep_msg(
                                        p as *mut ::core::ffi::c_char,
                                        0 as ::core::ffi::c_int,
                                    );
                                }
                                msg_scrolled_ign.set(false_0 != 0);
                            }
                            if newfile as ::core::ffi::c_int != 0
                                && (error as ::core::ffi::c_int != 0
                                    || conv_error != 0 as linenr_T
                                    || illegal_byte > 0 as linenr_T
                                        && bad_char_behavior != BAD_KEEP)
                            {
                                (*curbuf.get()).b_p_ro = true_0;
                            }
                            u_clearline(curbuf.get());
                            if exmode_active.get() {
                                (*curwin.get()).w_cursor.lnum = from + linecnt;
                            } else {
                                (*curwin.get()).w_cursor.lnum = from + 1 as linenr_T;
                            }
                            check_cursor_lnum(curwin.get());
                            beginline(
                                BL_WHITE as ::core::ffi::c_int | BL_FIX as ::core::ffi::c_int,
                            );
                            if (*cmdmod.ptr()).cmod_flags & CMOD_LOCKMARKS as ::core::ffi::c_int
                                == 0 as ::core::ffi::c_int
                            {
                                (*curbuf.get()).b_op_start.lnum = from + 1 as linenr_T;
                                (*curbuf.get()).b_op_start.col = 0 as ::core::ffi::c_int as colnr_T;
                                (*curbuf.get()).b_op_end.lnum = from + linecnt;
                                (*curbuf.get()).b_op_end.col = 0 as ::core::ffi::c_int as colnr_T;
                            }
                        }
                    }
                    msg_scroll.set(msg_save);
                    check_marks_read();
                    (*curbuf.get()).b_no_eol_lnum = read_no_eol_lnum;
                    if flags & READ_KEEP_UNDO as ::core::ffi::c_int != 0 {
                        u_find_first_changed();
                    }
                    if read_undo_file {
                        let mut hash: [uint8_t; 32] = sha_ctx.finish();
                        u_read_undo(
                            ::core::ptr::null_mut::<::core::ffi::c_char>(),
                            &raw mut hash as *mut uint8_t,
                            fname,
                        );
                    }
                    if !read_stdin && !read_fifo && (!read_buffer || !sfname.is_null()) {
                        let mut m_0: ::core::ffi::c_int = msg_scroll.get();
                        let mut n_1: ::core::ffi::c_int = msg_scrolled.get();
                        if set_options {
                            save_file_ff(curbuf.get());
                        }
                        msg_scroll.set(true_0);
                        if filtering {
                            apply_autocmds_exarg(
                                EVENT_FILTERREADPOST,
                                ::core::ptr::null_mut::<::core::ffi::c_char>(),
                                sfname,
                                false_0 != 0,
                                curbuf.get(),
                                eap,
                            );
                        } else if newfile as ::core::ffi::c_int != 0
                            || read_buffer as ::core::ffi::c_int != 0 && !sfname.is_null()
                        {
                            apply_autocmds_exarg(
                                EVENT_BUFREADPOST,
                                ::core::ptr::null_mut::<::core::ffi::c_char>(),
                                sfname,
                                false_0 != 0,
                                curbuf.get(),
                                eap,
                            );
                            if !(*curbuf.get()).b_au_did_filetype
                                && *(*curbuf.get()).b_p_ft as ::core::ffi::c_int != NUL
                            {
                                apply_autocmds(
                                    EVENT_FILETYPE,
                                    (*curbuf.get()).b_p_ft,
                                    (*curbuf.get()).b_fname,
                                    true_0 != 0,
                                    curbuf.get(),
                                );
                            }
                        } else {
                            apply_autocmds_exarg(
                                EVENT_FILEREADPOST,
                                sfname,
                                sfname,
                                false_0 != 0,
                                ::core::ptr::null_mut::<buf_T>(),
                                eap,
                            );
                        }
                        if msg_scrolled.get() == n_1 {
                            msg_scroll.set(m_0);
                        }
                        if aborting() {
                            return FAIL;
                        }
                    }
                    if !(recoverymode.get() as ::core::ffi::c_int != 0
                        && error as ::core::ffi::c_int != 0)
                    {
                        retval = OK;
                    }
                }
            }
        }
        if !(*curbuf.get()).b_ml.ml_mfp.is_null()
            && (*(*curbuf.get()).b_ml.ml_mfp).mf_dirty as ::core::ffi::c_uint
                == MF_DIRTY_YES_NOSYNC as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            (*(*curbuf.get()).b_ml.ml_mfp).mf_dirty = MF_DIRTY_YES;
        }
        return retval;
    }
}
unsafe extern "C" fn readfile_linenr(
    mut linecnt: linenr_T,
    mut p: *mut ::core::ffi::c_char,
    mut endp: *const ::core::ffi::c_char,
) -> linenr_T {
    unsafe {
        let mut lnum: linenr_T = (*curbuf.get()).b_ml.ml_line_count - linecnt + 1 as linenr_T;
        let mut s: *mut ::core::ffi::c_char = p;
        while s < endp as *mut ::core::ffi::c_char {
            if *s as ::core::ffi::c_int == '\n' as ::core::ffi::c_int {
                lnum += 1;
            }
            s = s.offset(1);
        }
        return lnum;
    }
}
pub unsafe extern "C" fn prep_exarg(mut eap: *mut exarg_T, mut buf: *const buf_T) {
    unsafe {
        let cmd_len: size_t = (15 as size_t).wrapping_add(strlen((*buf).b_p_fenc));
        (*eap).cmd = xmalloc(cmd_len) as *mut ::core::ffi::c_char;
        snprintf(
            (*eap).cmd,
            cmd_len,
            b"e ++enc=%s\0".as_ptr() as *const ::core::ffi::c_char,
            (*buf).b_p_fenc,
        );
        (*eap).force_enc = 8 as ::core::ffi::c_int;
        (*eap).bad_char = (*buf).b_bad_char;
        (*eap).force_ff = *(*buf).b_p_ff as ::core::ffi::c_uchar as ::core::ffi::c_int;
        (*eap).force_bin = if (*buf).b_p_bin != 0 {
            FORCE_BIN
        } else {
            FORCE_NOBIN
        };
        (*eap).read_edit = false_0;
        (*eap).forceit = false_0;
    }
}
pub unsafe extern "C" fn set_file_options(mut set_options: bool, mut eap: *mut exarg_T) {
    unsafe {
        if set_options {
            if !eap.is_null() && (*eap).force_ff != 0 as ::core::ffi::c_int {
                set_fileformat(
                    get_fileformat_force(curbuf.get(), eap),
                    OPT_LOCAL as ::core::ffi::c_int,
                );
            } else if *p_ffs.get() as ::core::ffi::c_int != NUL {
                set_fileformat(default_fileformat(), OPT_LOCAL as ::core::ffi::c_int);
            }
        }
        if !eap.is_null() && (*eap).force_bin != 0 as ::core::ffi::c_int {
            let mut oldval: ::core::ffi::c_int = (*curbuf.get()).b_p_bin;
            (*curbuf.get()).b_p_bin = ((*eap).force_bin == FORCE_BIN) as ::core::ffi::c_int;
            set_options_bin(
                oldval,
                (*curbuf.get()).b_p_bin,
                OPT_LOCAL as ::core::ffi::c_int,
            );
        }
    }
}
pub unsafe extern "C" fn set_forced_fenc(mut eap: *mut exarg_T) {
    unsafe {
        if (*eap).force_enc == 0 as ::core::ffi::c_int {
            return;
        }
        let mut fenc: *mut ::core::ffi::c_char =
            enc_canonize((*eap).cmd.offset((*eap).force_enc as isize));
        set_option_direct(
            kOptFileencoding,
            OptVal {
                type_0: kOptValTypeString,
                data: OptValData {
                    string: cstr_as_string(fenc),
                },
            },
            OPT_LOCAL as ::core::ffi::c_int,
            0 as scid_T,
        );
        xfree(fenc as *mut ::core::ffi::c_void);
    }
}
/// Set the name of the current buffer, for a `:r` or `:w` command with a file
/// name given for a buffer that has none.
pub unsafe extern "C" fn set_rw_fname(fname: *mut c_char, sfname: *mut c_char) -> c_int {
    unsafe {
        let buf = curbuf.get();

        // It's like the unnamed buffer is deleted...
        if (*curbuf.get()).b_p_bl != 0 {
            apply_autocmds(
                EVENT_BUFDELETE,
                ptr::null_mut(),
                ptr::null_mut(),
                false,
                curbuf.get(),
            );
        }
        apply_autocmds(
            EVENT_BUFWIPEOUT,
            ptr::null_mut(),
            ptr::null_mut(),
            false,
            curbuf.get(),
        );
        if aborting() {
            // Autocommands may abort script processing.
            return FAIL;
        }
        if curbuf.get() != buf {
            // We are in another buffer now, don't do the renaming.
            emsg(gettext(e_auchangedbuf.get()));
            return FAIL;
        }

        if setfname(curbuf.get(), fname, sfname, false) == OK {
            (*curbuf.get()).b_flags |= BF_NOTEDITED;
        }

        // ...and a new named one is created.
        apply_autocmds(
            EVENT_BUFNEW,
            ptr::null_mut(),
            ptr::null_mut(),
            false,
            curbuf.get(),
        );
        if (*curbuf.get()).b_p_bl != 0 {
            apply_autocmds(
                EVENT_BUFADD,
                ptr::null_mut(),
                ptr::null_mut(),
                false,
                curbuf.get(),
            );
        }
        if aborting() {
            return FAIL;
        }

        // Do filetype detection now if 'filetype' is empty.
        if *(*curbuf.get()).b_p_ft == 0 {
            if augroup_exists(c"filetypedetect".as_ptr()) {
                do_doautocmd(
                    c"filetypedetect BufRead".as_ptr().cast_mut(),
                    false,
                    ptr::null_mut(),
                );
            }
            do_modelines(0);
        }
        OK
    }
}

/// Put a file name into `ret_buf`, in quotes, with the home directory at the
/// start replaced by `~`.
pub unsafe extern "C" fn add_quoted_fname(
    ret_buf: *mut c_char,
    buf_len: size_t,
    buf: *const buf_T,
    fname: *const c_char,
) {
    unsafe {
        let fname = if fname.is_null() {
            c"-stdin-".as_ptr()
        } else {
            fname
        };
        *ret_buf = b'"' as c_char;
        home_replace(buf, fname, ret_buf.add(1), buf_len - 4, true);
        xstrlcat(ret_buf, c"\" ".as_ptr(), buf_len);
    }
}

/// Append the file format to `IObuff`, unless it is the platform default.
///
/// @return  true if something was appended.
pub unsafe extern "C" fn msg_add_fileformat(eol_type: c_int) -> bool {
    unsafe {
        let note = match eol_type {
            EOL_DOS => c"[dos]",
            EOL_MAC => c"[mac]",
            // On a platform where CRLF is the default, EOL_UNIX would be
            // worth noting instead; upstream compiles that in with USE_CRNL.
            _ => return false,
        };
        xstrlcat(
            IObuff.ptr().cast::<c_char>(),
            gettext(note.as_ptr()),
            IOSIZE as size_t,
        );
        true
    }
}

/// Append the line and character count to `IObuff`.
pub unsafe extern "C" fn msg_add_lines(insert_space: c_int, lnum: linenr_T, nchars: off_T) {
    unsafe {
        let io = IObuff.ptr().cast::<c_char>();
        let mut len = strlen(io);
        let space = if insert_space != 0 { c" " } else { c"" }.as_ptr();

        if shortmess(SHM_LINES as c_int) {
            // l10n: L as in line, B as in byte.
            snprintf(
                io.add(len),
                IOSIZE as size_t - len,
                gettext(c"%s%ldL, %ldB".as_ptr()),
                space,
                lnum as int64_t,
                nchars as int64_t,
            );
            return;
        }

        len += snprintf(
            io.add(len),
            IOSIZE as size_t - len,
            ngettext(
                c"%s%ld line, ".as_ptr(),
                c"%s%ld lines, ".as_ptr(),
                lnum as core::ffi::c_ulong,
            ),
            space,
            lnum as int64_t,
        ) as size_t;
        snprintf(
            io.add(len),
            IOSIZE as size_t - len,
            ngettext(
                c"%ld byte".as_ptr(),
                c"%ld bytes".as_ptr(),
                nchars as core::ffi::c_ulong,
            ),
            nchars as int64_t,
        );
    }
}

/// Like `fgets()`, but a line longer than the buffer is truncated and the rest
/// of it thrown away.
///
/// @return  true for EOF or error
pub unsafe extern "C" fn vim_fgets(buf: *mut c_char, size: c_int, fp: *mut FILE) -> bool {
    unsafe {
        assert!(size > 0);
        // The last-but-one byte tells us whether the line fitted: `fgets`
        // leaves it alone if the line was shorter than the buffer.
        let last = (size - 2) as isize;
        *buf.offset(last) = 0;

        let mut retval;
        loop {
            *__errno_location() = 0;
            retval = fgets(buf, size, fp);
            if !(retval.is_null() && *__errno_location() == EINTR && ferror(fp) != 0) {
                break;
            }
        }

        let filled = |c: c_char| c != 0 && c != b'\n' as c_char;
        if filled(*buf.offset(last)) {
            buf.offset(last + 1).write(0); // truncate the line

            // Now throw away the rest of the line.
            let mut tbuf = [0 as c_char; 200];
            let tlast = tbuf.len() - 2;
            loop {
                tbuf[tlast] = 0;
                *__errno_location() = 0;
                retval = fgets(tbuf.as_mut_ptr(), tbuf.len() as c_int, fp);
                if retval.is_null() && (feof(fp) != 0 || *__errno_location() != EINTR) {
                    break;
                }
                if !filled(tbuf[tlast]) {
                    break;
                }
            }
        }
        retval.is_null()
    }
}

/// Read `N` bytes from `fd` and turn them into an integer, most significant
/// byte first. Returns -1 at end of file.
unsafe fn get_bytes<const N: usize>(fd: *mut FILE) -> Option<u64> {
    unsafe {
        let mut n: u64 = 0;
        for _ in 0..N {
            let c = getc(fd);
            if c == EOF {
                return None;
            }
            n = (n << 8) + c as u64;
        }
        Some(n)
    }
}

/// Read 2 bytes from `fd` and turn them into an int, MSB first.
///
/// @return  -1 when encountering EOF.
pub unsafe extern "C" fn get2c(fd: *mut FILE) -> c_int {
    unsafe { get_bytes::<2>(fd).map_or(-1, |n| n as c_int) }
}

/// Read 3 bytes from `fd` and turn them into an int, MSB first.
///
/// @return  -1 when encountering EOF.
pub unsafe extern "C" fn get3c(fd: *mut FILE) -> c_int {
    unsafe { get_bytes::<3>(fd).map_or(-1, |n| n as c_int) }
}

/// Read 4 bytes from `fd` and turn them into an int, MSB first.
///
/// The result wraps around rather than saturating when the top bit is set,
/// which is what upstream's unsigned accumulator gives.
///
/// @return  -1 when encountering EOF.
pub unsafe extern "C" fn get4c(fd: *mut FILE) -> c_int {
    unsafe { get_bytes::<4>(fd).map_or(-1, |n| n as u32 as c_int) }
}

/// Read 8 bytes from `fd` and turn them into a `time_t`, MSB first.
///
/// @return  -1 when encountering EOF.
pub unsafe extern "C" fn get8ctime(fd: *mut FILE) -> time_t {
    unsafe { get_bytes::<8>(fd).map_or(-1, |n| n as time_t) }
}

/// Read a string of length `cnt` from `fd` into allocated memory.
///
/// @return  the string, or NULL when unable to read that many bytes.
pub unsafe extern "C" fn read_string(fd: *mut FILE, cnt: size_t) -> *mut c_char {
    unsafe {
        let str = xmallocz(cnt).cast::<c_char>();
        for i in 0..cnt {
            let c = getc(fd);
            if c == EOF {
                xfree(str.cast());
                return ptr::null_mut();
            }
            *str.add(i) = c as c_char;
        }
        str
    }
}

/// Write `number` to `fd` in `len` bytes, most significant byte first.
///
/// @return  false in case of an error.
pub unsafe extern "C" fn put_bytes(fd: *mut FILE, number: uintmax_t, len: size_t) -> bool {
    unsafe {
        assert!(len > 0);
        for i in (0..len).rev() {
            if putc((number >> (i * 8)) as c_int, fd) == EOF {
                return false;
            }
        }
        true
    }
}

/// Write a `time_t` to `fd` in 8 bytes.
///
/// @return  FAIL when the write failed.
pub unsafe extern "C" fn put_time(fd: *mut FILE, time_: time_t) -> c_int {
    unsafe {
        let mut buf = [0u8; 8];
        time_to_bytes(time_, buf.as_mut_ptr());
        // Upstream compares `fwrite`'s item count against 1 while asking it
        // for 8 one-byte items, so this always answers FAIL. Both callers
        // ignore the answer. Preserved.
        if fwrite(buf.as_ptr().cast(), 1, buf.len(), fd) == 1 {
            OK
        } else {
            FAIL
        }
    }
}

/// Version of `read()` that retries when interrupted by a signal, which
/// `SIGWINCH` makes routine.
pub unsafe extern "C" fn read_eintr(fd: c_int, buf: *mut c_void, bufsize: size_t) -> ssize_t {
    unsafe {
        loop {
            let ret = read(fd, buf, bufsize);
            if ret >= 0 || *__errno_location() != EINTR {
                return ret;
            }
        }
    }
}

/// Version of `write()` that retries when interrupted by a signal.
///
/// Repeats the write for as long as it doesn't fail for a reason other than
/// being interrupted; the caller compares the result against `bufsize` to see
/// whether everything got out.
pub unsafe extern "C" fn write_eintr(fd: c_int, buf: *mut c_void, bufsize: size_t) -> ssize_t {
    unsafe {
        let mut written: ssize_t = 0;
        while (written as size_t) < bufsize {
            let wlen = write(
                fd,
                buf.cast::<c_char>().offset(written).cast(),
                bufsize - written as size_t,
            );
            if wlen < 0 {
                if *__errno_location() != EINTR {
                    break;
                }
            } else {
                written += wlen;
            }
        }
        written
    }
}
pub const IOSIZE: ::core::ffi::c_int = 1024 as ::core::ffi::c_int + 1 as ::core::ffi::c_int;
pub const SEA_QUIT: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
pub const BAD_REPLACE: ::core::ffi::c_int = '?' as ::core::ffi::c_int;
pub const BAD_KEEP: ::core::ffi::c_int = -1 as ::core::ffi::c_int;
pub const BAD_DROP: ::core::ffi::c_int = -2 as ::core::ffi::c_int;
pub const FORCE_BIN: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const FORCE_NOBIN: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
pub const ENC_UCSBOM: [::core::ffi::c_char; 8] =
    unsafe { ::core::mem::transmute::<[u8; 8], [::core::ffi::c_char; 8]>(*b"ucs-bom\0") };
pub const EOL_UNKNOWN: ::core::ffi::c_int = -1 as ::core::ffi::c_int;
pub const EOL_UNIX: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const EOL_DOS: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const EOL_MAC: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
pub const CPO_FNAMER: ::core::ffi::c_int = 'f' as ::core::ffi::c_int;
pub const EOVERFLOW: ::core::ffi::c_int = 75 as ::core::ffi::c_int;
pub const EINTR: ::core::ffi::c_int = 4 as ::core::ffi::c_int;
pub const EINVAL: ::core::ffi::c_int = 22 as ::core::ffi::c_int;
pub const INT_MAX: ::core::ffi::c_int = __INT_MAX__;
pub const __S_IFMT: ::core::ffi::c_int = 0o170000 as ::core::ffi::c_int;
pub const NAME_MAX: ::core::ffi::c_int = 255 as ::core::ffi::c_int;
pub const true_0: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const false_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const ICONV_EINVAL: ::core::ffi::c_int = EINVAL;
pub const RE_MAGIC: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const __INT_MAX__: ::core::ffi::c_int = 2147483647 as ::core::ffi::c_int;
