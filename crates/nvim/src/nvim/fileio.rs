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
    msg_outtrans, msg_progress, msg_putchar, msg_puts, msg_puts_hl, msg_start, msg_trunc,
    set_keep_msg,
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
    __errno_location, close, dup, feof, ferror, fgets, flock, fwrite, getc, gettext, iconv,
    iconv_close, lseek, memchr, memcpy, ngettext, putc, read, readlink, snprintf, strcmp, strlen,
    symlink, umask, write,
};
use crate::src::nvim::os::users::os_get_username;
use crate::src::nvim::path::{
    add_pathsep, after_pathsep, dir_of_file_exists, path_fnamecmp, path_is_absolute,
    path_shorten_fname, path_tail, path_with_url, vim_FullName,
};
use crate::src::nvim::pos::MAXLNUM;
use crate::src::nvim::regexp::{vim_regcomp, vim_regexec, vim_regfree};
use crate::src::nvim::sha256::Sha256;
use crate::src::nvim::shada::check_marks_read;
use crate::src::nvim::state::{MODE_CMDLINE, MODE_NORMAL_BUSY};
use crate::src::nvim::strings::{sort_strings, vim_strchr};
use crate::src::nvim::types::ui::kUIMessages;
use crate::src::nvim::types::{
    CheckItem, Directory, FILE, FileInfo, OptInt, OptVal, OptValData, OptValType, aco_save_T,
    bln_values, buf_T, bufref_T, colnr_T, exarg_T, garray_T, iconv_t, int64_t, linenr_T, off_T,
    pos_T, ptrdiff_t, regmatch_T, regprog_T, scid_T, size_t, ssize_t, time_t, uint64_t, uintmax_t,
    uv_gid_t, uv_uid_t,
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
mod lines;
pub(crate) use self::lines::*;
mod open;
pub(crate) use self::open::*;
mod read;
pub(crate) use self::read::*;
mod report;
pub(crate) use self::report::*;
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
}
pub type DIR = __dirstream;
pub type C2Rust_Unnamed_5 = ::core::ffi::c_int;
pub const UV_ENOENT: C2Rust_Unnamed_5 = -2;
pub const UV_EFBIG: C2Rust_Unnamed_5 = -27;
pub const kOptValTypeString: OptValType = 2;
pub const BLN_DUMMY: bln_values = 4;
pub type C2Rust_Unnamed_25 = ::core::ffi::c_uint;
pub const BL_FIX: C2Rust_Unnamed_25 = 4;
pub const BL_WHITE: C2Rust_Unnamed_25 = 1;
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
pub const BASENAMELEN: ::core::ffi::c_int = NAME_MAX - 5 as ::core::ffi::c_int;
pub const DEFAULT_MAXPATHL: ::core::ffi::c_int = 4096 as ::core::ffi::c_int;
pub const MAXPATHL: ::core::ffi::c_int = DEFAULT_MAXPATHL;
pub const NUL: ::core::ffi::c_int = '\0' as ::core::ffi::c_int;
pub const NL: ::core::ffi::c_int = '\n' as ::core::ffi::c_int;
pub const CAR: ::core::ffi::c_int = '\r' as ::core::ffi::c_int;
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
/// From the buffer's line count before this read and the bytes read since,
/// estimate the line number we are now on. Used for error messages.
///
/// @param linecnt  the line count before the extra bytes were read
/// @param p        the start of those bytes
/// @param endp     the end of them
pub(crate) unsafe fn readfile_linenr(
    linecnt: linenr_T,
    p: *const ::core::ffi::c_char,
    endp: *const ::core::ffi::c_char,
) -> linenr_T {
    unsafe {
        let mut lnum: linenr_T = (*curbuf.get()).b_ml.ml_line_count - linecnt + 1 as linenr_T;
        let mut s = p;
        while s < endp {
            if *s as ::core::ffi::c_int == '\n' as ::core::ffi::c_int {
                lnum += 1;
            }
            s = s.offset(1);
        }
        return lnum;
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
        debug_assert!(size > 0);
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
        debug_assert!(len > 0);
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
pub const __INT_MAX__: ::core::ffi::c_int = 2147483647 as ::core::ffi::c_int;

/// Fill `eap` so that `'fileencoding'`, `'fileformat'` and `'binary'` are
/// forced to what buffer `buf` already has. Used when calling `readfile` to
/// re-read a buffer that is already open.
pub unsafe extern "C" fn prep_exarg(eap: *mut exarg_T, buf: *const buf_T) {
    unsafe {
        let cmd_len = 15 + strlen((*buf).b_p_fenc);
        (*eap).cmd = xmalloc(cmd_len).cast();
        snprintf((*eap).cmd, cmd_len, c"e ++enc=%s".as_ptr(), (*buf).b_p_fenc);
        // Where the encoding name starts in that command.
        (*eap).force_enc = 8;
        (*eap).bad_char = (*buf).b_bad_char;
        (*eap).force_ff = *(*buf).b_p_ff as u8 as c_int;
        (*eap).force_bin = if (*buf).b_p_bin != 0 {
            FORCE_BIN
        } else {
            FORCE_NOBIN
        };
        (*eap).read_edit = false as c_int;
        (*eap).forceit = false as c_int;
    }
}

/// Set the default or forced `'fileformat'` and `'binary'`.
pub unsafe extern "C" fn set_file_options(set_options: bool, eap: *mut exarg_T) {
    unsafe {
        // Set the default 'fileformat'.
        if set_options {
            if !eap.is_null() && (*eap).force_ff != 0 {
                set_fileformat(get_fileformat_force(curbuf.get(), eap), OPT_LOCAL as c_int);
            } else if *p_ffs.get() != 0 {
                set_fileformat(default_fileformat(), OPT_LOCAL as c_int);
            }
        }

        // Set or reset 'binary'.
        if !eap.is_null() && (*eap).force_bin != 0 {
            let oldval = (*curbuf.get()).b_p_bin;
            (*curbuf.get()).b_p_bin = ((*eap).force_bin == FORCE_BIN) as c_int;
            set_options_bin(oldval, (*curbuf.get()).b_p_bin, OPT_LOCAL as c_int);
        }
    }
}

/// Set the forced `'fileencoding'` from a `++enc=` argument.
pub unsafe extern "C" fn set_forced_fenc(eap: *mut exarg_T) {
    unsafe {
        if (*eap).force_enc == 0 {
            return;
        }
        let fenc = enc_canonize((*eap).cmd.offset((*eap).force_enc as isize));
        set_option_direct(
            kOptFileencoding,
            OptVal {
                type_0: kOptValTypeString,
                data: OptValData {
                    string: cstr_as_string(fenc),
                },
            },
            OPT_LOCAL as c_int,
            0 as scid_T,
        );
        xfree(fenc.cast());
    }
}
