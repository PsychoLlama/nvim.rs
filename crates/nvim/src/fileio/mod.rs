#![deny(unsafe_op_in_unsafe_fn)]

use crate::api::private::helpers::cstr_as_string;
use crate::ascii::ascii_isspace;
use crate::autocmd::{
    EVENT_BUFADD, EVENT_BUFDELETE, EVENT_BUFNEW, EVENT_BUFNEWFILE, EVENT_BUFREADCMD,
    EVENT_BUFREADPOST, EVENT_BUFREADPRE, EVENT_BUFWIPEOUT, EVENT_FILECHANGEDSHELL,
    EVENT_FILECHANGEDSHELLPOST, EVENT_FILEREADCMD, EVENT_FILEREADPOST, EVENT_FILEREADPRE,
    EVENT_FILETYPE, EVENT_FILTERREADPOST, EVENT_FILTERREADPRE, EVENT_STDINREADPRE, apply_autocmds,
    apply_autocmds_exarg, aucmd_prepbuf, aucmd_restbuf, augroup_exists, do_doautocmd,
};
use crate::buffer::{
    BufFlags, buf_contents_changed, buf_is_dontwrite, buf_is_empty, buf_is_nofilename,
    buf_is_normal, buflist_new, current_buf, do_modelines, setfname, wipe_buffer,
};
use crate::buffer_updates::buf_updates_unload;
use crate::change::{appended_lines_mark, save_file_ff, unchanged};
use crate::cursor::{check_cursor, check_cursor_lnum};
use crate::diff::diff_invalidate;
use crate::drawscreen::{UPD_NOT_VALID, redraw_curbuf_later, status_redraw_all};
use crate::edit::beginline;
use crate::eval::vars::{eval_charconvert, get_vim_var_str, set_vim_var_string};
use crate::event::libuv::uv_strerror;
use crate::ex_eval::aborting;
use crate::fold::{fold_update_all, foldmethod_is_manual};
use crate::garray::{ga_clear_strings, ga_grow, ga_init};
use crate::getchar::stuff_empty;
use crate::global_cell::GlobalCell;
use crate::log::{LOGLVL_DBG, LOGLVL_ERR, LOGLVL_WRN};
use crate::main::{
    State, allbuf_lock, autocmd_busy, curbuf, did_check_timestamps, e_interr, e_notopen,
    emsg_silent, ex_no_reprint, exiting, exmode_active, global_busy, got_int, in_assert_fails,
    keep_msg, msg_col, msg_listdo_overwrite, msg_scroll, msg_scrolled, msg_scrolled_ign,
    msg_silent, need_check_timestamps, need_fileinfo, need_wait_return, no_check_timestamps,
    no_wait_return, p_ar, p_ccv, p_enc, p_fencs, p_ffs, p_fic, p_ur, p_verbose, readonlymode,
    recoverymode, redraw_cmdline, redraw_tabline, restart_edit, stdin_fd, swap_exists_action,
    vim_ignored,
};
use crate::mbyte::{
    enc_canon_props, enc_canonize, my_iconv_open, utf_byte2len, utf_char2bytes, utf_char2len,
    utf_head_off, utf_ptr2char, utf_ptr2len_len,
};
use crate::memfile::mf_fullname;
use crate::memline::{
    check_need_swap, ml_append, ml_delete, ml_get, ml_get_buf, ml_get_buf_len, ml_get_len, ml_open,
};
use crate::memory::{
    memchrsub, time_to_bytes, verbose_try_malloc, xfree, xmalloc, xmallocz, xmemdupz, xstrdup,
    xstrlcat,
};
use crate::message::{
    do_dialog, emsg, msg, msg_check_for_delay, msg_clr_eos, msg_delay, msg_end, msg_may_trunc,
    msg_outtrans, msg_progress, msg_putchar, msg_puts, msg_puts_hl, msg_start, msg_trunc,
    set_keep_msg,
};
use crate::r#move::update_topline;
use crate::option::{
    copy_option_part, default_fileformat, get_fileformat, get_fileformat_force, set_fileformat,
    set_option_direct, set_options_bin, shortmess,
};
use crate::options::kOptFileencoding;
use crate::os::cshim::{getc, gettext, gettext_ptr, ngettext, putc, snprintf};
use crate::os::env::{expand_env, home_replace, home_replace_save, os_env_exists};
use crate::os::fs::{
    os_closedir, os_copy, os_dirname, os_fchown, os_file_is_writable, os_file_owned, os_fileinfo,
    os_fileinfo_id_equal, os_fileinfo_link, os_fileinfo_size, os_free_acl, os_get_acl, os_getperm,
    os_isdir, os_isrealdir, os_mkdir, os_mkdtemp, os_open, os_path_exists, os_remove, os_rename,
    os_rmdir, os_scandir, os_scandir_next, os_set_acl, os_set_cloexec, os_setperm,
};
use crate::os::input::os_breakcheck;
use crate::os::users::os_get_username;
use crate::path::{
    add_pathsep, after_pathsep, dir_of_file_exists, path_fnamecmp, path_is_absolute,
    path_shorten_fname, path_tail, path_with_url, vim_full_name,
};
use crate::pos::MAXLNUM;
use crate::regexp::{vim_regcomp, vim_regexec, vim_regfree};
use crate::sha256::Sha256;
use crate::shada::check_marks_read;
use crate::state::{MODE_CMDLINE, MODE_NORMAL_BUSY};
use crate::strings::{sort_strings, vim_strchr};
use crate::types::ui::kUIMessages;
use crate::types::{
    CheckItem, Directory, FAIL, FILE, FileInfo, IOSIZE, OK, OptInt, OptVal, OptValData, OptValType,
    OptionSetFlags, ShmFlag, aco_save_T, bln_values, buf_T, colnr_T, event_T, exarg_T, garray_T,
    iconv_t, int64_t, linenr_T, off_T, pos_T, ptrdiff_t, regmatch_T, regprog_T, scid_T, size_t,
    ssize_t, time_t, uint64_t, uintmax_t, uv_gid_t, uv_uid_t,
};
use crate::ui::{ui_flush, ui_has};
use crate::undo::{
    buf_is_changed, u_clearallandblockfree, u_clearline, u_compute_hash, u_find_first_changed,
    u_read_undo, u_savecommon, u_sync, u_unchanged, u_write_undo,
};
use crate::winlayer::{Buf, Ea};
use ::libc::{
    __errno_location, close, dup, feof, ferror, fgets, flock, fwrite, iconv, iconv_close, lseek,
    memchr, memcpy, read, readlink, strcmp, strlen, symlink, umask, write,
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
pub const kOptValTypeString: OptValType = 2;
pub const BLN_DUMMY: bln_values = 4;
pub const ENC_LATIN1: ::core::ffi::c_uint = 512;
pub const ENC_2WORD: ::core::ffi::c_uint = 256;
pub const ENC_4BYTE: ::core::ffi::c_uint = 128;
pub const ENC_2BYTE: ::core::ffi::c_uint = 64;
pub const ENC_ENDIAN_L: ::core::ffi::c_uint = 32;
pub const ENC_UNICODE: ::core::ffi::c_uint = 4;
pub const READ_NOFILE: ::core::ffi::c_uint = 256;
pub const READ_FIFO: ::core::ffi::c_uint = 64;
pub const READ_KEEP_UNDO: ::core::ffi::c_uint = 32;
pub const READ_DUMMY: ::core::ffi::c_uint = 16;
pub const READ_BUFFER: ::core::ffi::c_uint = 8;
pub const READ_STDIN: ::core::ffi::c_uint = 4;
pub const READ_FILTER: ::core::ffi::c_uint = 2;
pub const READ_NEW: ::core::ffi::c_uint = 1;
pub const FIO_ALL: ::core::ffi::c_int = -1;
pub const FIO_UCSBOM: ::core::ffi::c_int = 16384;
pub const FIO_ENDIAN_L: ::core::ffi::c_int = 128;
pub const FIO_UTF16: ::core::ffi::c_int = 16;
pub const FIO_UCS4: ::core::ffi::c_int = 8;
pub const FIO_UCS2: ::core::ffi::c_int = 4;
pub const FIO_UTF8: ::core::ffi::c_int = 2;
pub const FIO_LATIN1: ::core::ffi::c_int = 1;
pub const CONV_RESTLEN: ::core::ffi::c_uint = 30;
pub const ICONV_MULT: ::core::ffi::c_uint = 8;
pub const VIM_WARNING: ::core::ffi::c_uint = 2;
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const O_RDONLY: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const EOF: ::core::ffi::c_int = -1 as ::core::ffi::c_int;
pub const SEEK_SET: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const UV_FS_COPYFILE_EXCL: ::core::ffi::c_int = 0x1 as ::core::ffi::c_int;
pub const BASENAMELEN: ::core::ffi::c_int = NAME_MAX - 5 as ::core::ffi::c_int;
pub const NL: ::core::ffi::c_int = '\n' as ::core::ffi::c_int;
pub const CAR: ::core::ffi::c_int = '\r' as ::core::ffi::c_int;
pub const NOTDONE: ::core::ffi::c_int = 2 as ::core::ffi::c_int;

/// One of the buffer-lifecycle autocommands this file fires about the current
/// buffer: `apply_autocmds(event, NULL, NULL, false, curbuf)`.
fn autocmd_for_curbuf(event: event_T) {
    let (nofile, cb) = (ptr::null_mut(), curbuf.get());
    // SAFETY: the current buffer is live, and the event takes no file name.
    unsafe { apply_autocmds(event, nofile, nofile, false, cb) };
}
static e_auchangedbuf: GlobalCell<*const ::core::ffi::c_char> =
    GlobalCell::new(c"E812: Autocommands changed buffer or buffer name".as_ptr());
pub const NONASCII_MASK: uint64_t = (-1 as ::core::ffi::c_int as uint64_t)
    .wrapping_div(0xff as uint64_t)
    .wrapping_mul(0x80 as uint64_t);
/// Report which file is being read or written, in `IObuff`.
///
/// `s` is the note to append; an empty one means the message is progress on
/// a write that is still running.
pub unsafe fn filemess(buf: Buf, name: *mut c_char, s: *mut c_char) {
    // The report. Upstream builds it in `IObuff` and then calls
    // `msg_progress`/`msg_outtrans`, which write it again.
    let mut report = [0 as c_char; IOSIZE as usize];
    let prev_msg_col = msg_col.get();
    if msg_silent.get() != 0 {
        return;
    }
    let io = report.as_mut_ptr();
    unsafe { add_quoted_fname(io, IOSIZE as size_t - 100, buf, name) };
    // Avoid an over-long translation causing trouble.
    unsafe { xstrlcat(io, s, IOSIZE as size_t) };

    // For the first message we may have to start a new line. Further ones
    // overwrite the previous one; reset `msg_scroll` before calling this.
    let msg_scroll_save = msg_scroll.get();
    if shortmess(ShmFlag::OVERALL)
        && msg_listdo_overwrite.get() == 0
        && !exiting.get()
        && p_verbose.get() == 0
    {
        msg_scroll.set(false as c_int);
    }
    if msg_scroll.get() == 0 {
        // Wait a bit when overwriting an error message.
        unsafe { msg_check_for_delay(false) };
    }
    unsafe { msg_start() };
    if prev_msg_col != 0 && msg_col.get() == 0 {
        unsafe { msg_putchar(b'\r' as c_int) }; // overwrite any previous message
    }
    msg_scroll.set(msg_scroll_save);
    msg_scrolled_ign.set(true);
    if unsafe { *s } == 0 {
        let (id, status) = (c"bufwrite".as_ptr(), c"running".as_ptr());
        // SAFETY: `io` is this frame's report buffer, NUL-terminated by
        // `xstrlcat`; the two tags are static strings.
        unsafe { msg_progress(io, id.cast_mut(), status.cast_mut(), 0, false, true) };
    } else {
        // May truncate the message to avoid a hit-return prompt.
        unsafe { msg_outtrans(msg_may_trunc(false, io), 0, false) };
    }
    unsafe { msg_clr_eos() };
    msg_scrolled_ign.set(false);
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
    let mut lnum: linenr_T = cur_buf().b_ml.ml_line_count - linecnt + 1 as linenr_T;
    let mut s = p;
    while s < endp {
        if unsafe { *s } as ::core::ffi::c_int == '\n' as ::core::ffi::c_int {
            lnum += 1;
        }
        s = unsafe { s.offset(1) };
    }
    lnum
}
/// Set the name of the current buffer, for a `:r` or `:w` command with a file
/// name given for a buffer that has none.
pub unsafe fn set_rw_fname(fname: *mut c_char, sfname: *mut c_char) -> c_int {
    let buf = curbuf.get();

    // It's like the unnamed buffer is deleted...
    if cur_buf().b_p_bl != 0 {
        autocmd_for_curbuf(EVENT_BUFDELETE);
    }
    autocmd_for_curbuf(EVENT_BUFWIPEOUT);
    if aborting() {
        // Autocommands may abort script processing.
        return FAIL;
    }
    if curbuf.get() != buf {
        // We are in another buffer now, don't do the renaming.
        unsafe { emsg(gettext_ptr(e_auchangedbuf.get())) };
        return FAIL;
    }

    if unsafe { setfname(cur_buf(), fname, sfname, false) } == OK {
        cur_buf().b_flags |= BufFlags::NOTEDITED;
    }

    // ...and a new named one is created.
    autocmd_for_curbuf(EVENT_BUFNEW);
    if cur_buf().b_p_bl != 0 {
        autocmd_for_curbuf(EVENT_BUFADD);
    }
    if aborting() {
        return FAIL;
    }

    // Do filetype detection now if 'filetype' is empty.
    if unsafe { *cur_buf().b_p_ft } == 0 {
        if unsafe { augroup_exists(c"filetypedetect".as_ptr()) } {
            let cmd = c"filetypedetect BufRead".as_ptr().cast_mut();
            // SAFETY: a static command line.
            unsafe { do_doautocmd(cmd, false, ptr::null_mut()) };
        }
        do_modelines(OptionSetFlags::NONE);
    }
    OK
}

/// Put a file name into `ret_buf`, in quotes, with the home directory at the
/// start replaced by `~`.
pub unsafe fn add_quoted_fname(
    ret_buf: *mut c_char,
    buf_len: size_t,
    buf: Buf,
    fname: *const c_char,
) {
    let fname = if fname.is_null() {
        c"-stdin-".as_ptr()
    } else {
        fname
    };
    unsafe { *ret_buf = b'"' as c_char };
    unsafe { home_replace(buf.raw(), fname, ret_buf.add(1), buf_len - 4, true) };
    unsafe { xstrlcat(ret_buf, c"\" ".as_ptr(), buf_len) };
}

/// Append the file format to `IObuff`, unless it is the platform default.
///
/// @return  true if something was appended.
pub(crate) unsafe fn msg_add_fileformat(
    report: &mut [c_char; IOSIZE as usize],
    eol_type: c_int,
) -> bool {
    let note = match eol_type {
        EOL_DOS => c"[dos]",
        EOL_MAC => c"[mac]",
        // On a platform where CRLF is the default, EOL_UNIX would be
        // worth noting instead; upstream compiles that in with USE_CRNL.
        _ => return false,
    };
    let io = report.as_mut_ptr();
    // SAFETY: `report` holds `IOSIZE` bytes and the note is a static
    // string.
    unsafe { xstrlcat(io, gettext(note).as_ptr(), IOSIZE as size_t) };
    true
}

/// Append the line and character count to `report`.
pub(crate) unsafe fn msg_add_lines(
    report: &mut [c_char; IOSIZE as usize],
    insert_space: c_int,
    lnum: linenr_T,
    nchars: off_T,
) {
    let io = report.as_mut_ptr();
    let mut len = unsafe { strlen(io) };
    let space = if insert_space != 0 { c" " } else { c"" }.as_ptr();

    if shortmess(ShmFlag::LINES) {
        // l10n: L as in line, B as in byte.
        let fmt = c"%s%ldL, %ldB".as_ptr();
        let (l, b) = (lnum as int64_t, nchars as int64_t);
        let room = IOSIZE as size_t - len;
        // SAFETY: `io` holds `IOSIZE` bytes and `len` of them are used; the
        // three conversions match the three arguments.
        unsafe { snprintf(io.add(len), room, gettext_ptr(fmt).as_ptr(), space, l, b) };
        return;
    }

    let lines_one = c"%s%ld line, ";
    let lines_many = c"%s%ld lines, ";
    let fmt = ngettext(lines_one, lines_many, lnum as core::ffi::c_ulong);
    let (at, room) = (unsafe { io.add(len) }, IOSIZE as size_t - len);
    len += unsafe { snprintf(at, room, fmt.as_ptr(), space, lnum as int64_t) } as size_t;
    let bytes_one = c"%ld byte";
    let bytes_many = c"%ld bytes";
    let fmt = ngettext(bytes_one, bytes_many, nchars as core::ffi::c_ulong);
    let (at, room) = (unsafe { io.add(len) }, IOSIZE as size_t - len);
    unsafe { snprintf(at, room, fmt.as_ptr(), nchars as int64_t) };
}

/// Like `fgets()`, but a line longer than the buffer is truncated and the rest
/// of it thrown away.
///
/// @return  true for EOF or error
pub unsafe fn vim_fgets(buf: *mut c_char, size: c_int, fp: *mut FILE) -> bool {
    debug_assert!(size > 0);
    // The last-but-one byte tells us whether the line fitted: `fgets`
    // leaves it alone if the line was shorter than the buffer.
    let last = (size - 2) as isize;
    unsafe { *buf.offset(last) = 0 };

    let mut retval;
    loop {
        unsafe { *__errno_location() = 0 };
        retval = unsafe { fgets(buf, size, fp) };
        if !(retval.is_null()
            && unsafe { *__errno_location() } == EINTR
            && unsafe { ferror(fp) } != 0)
        {
            break;
        }
    }

    let filled = |c: c_char| c != 0 && c != b'\n' as c_char;
    if filled(unsafe { *buf.offset(last) }) {
        // SAFETY: `buf` holds `size` bytes and `last + 1` is inside them.
        unsafe { buf.offset(last + 1).write(0) }; // truncate the line

        // Now throw away the rest of the line.
        let mut tbuf = [0 as c_char; 200];
        let tlast = tbuf.len() - 2;
        loop {
            tbuf[tlast] = 0;
            unsafe { *__errno_location() = 0 };
            retval = unsafe { fgets(tbuf.as_mut_ptr(), tbuf.len() as c_int, fp) };
            if retval.is_null()
                && (unsafe { feof(fp) } != 0 || unsafe { *__errno_location() } != EINTR)
            {
                break;
            }
            if !filled(tbuf[tlast]) {
                break;
            }
        }
    }
    retval.is_null()
}

/// Read `N` bytes from `fd` and turn them into an integer, most significant
/// byte first. Returns -1 at end of file.
unsafe fn get_bytes<const N: usize>(fd: *mut FILE) -> Option<u64> {
    let mut n: u64 = 0;
    for _ in 0..N {
        let c = unsafe { getc(fd) };
        if c == EOF {
            return None;
        }
        n = (n << 8) + c as u64;
    }
    Some(n)
}

/// Read 2 bytes from `fd` and turn them into an int, MSB first.
///
/// @return  -1 when encountering EOF.
pub unsafe fn get2c(fd: *mut FILE) -> c_int {
    unsafe { get_bytes::<2>(fd).map_or(-1, |n| n as c_int) }
}

/// Read 3 bytes from `fd` and turn them into an int, MSB first.
///
/// @return  -1 when encountering EOF.
pub unsafe fn get3c(fd: *mut FILE) -> c_int {
    unsafe { get_bytes::<3>(fd).map_or(-1, |n| n as c_int) }
}

/// Read 4 bytes from `fd` and turn them into an int, MSB first.
///
/// The result wraps around rather than saturating when the top bit is set,
/// which is what upstream's unsigned accumulator gives.
///
/// @return  -1 when encountering EOF.
pub unsafe fn get4c(fd: *mut FILE) -> c_int {
    unsafe { get_bytes::<4>(fd).map_or(-1, |n| n as u32 as c_int) }
}

/// Read 8 bytes from `fd` and turn them into a `time_t`, MSB first.
///
/// @return  -1 when encountering EOF.
pub unsafe fn get8ctime(fd: *mut FILE) -> time_t {
    unsafe { get_bytes::<8>(fd).map_or(-1, |n| n as time_t) }
}

/// Read a string of length `cnt` from `fd` into allocated memory.
///
/// @return  the string, or NULL when unable to read that many bytes.
pub unsafe fn read_string(fd: *mut FILE, cnt: size_t) -> *mut c_char {
    let str = unsafe { xmallocz(cnt) }.cast::<c_char>();
    for i in 0..cnt {
        let c = unsafe { getc(fd) };
        if c == EOF {
            unsafe { xfree(str.cast()) };
            return ptr::null_mut();
        }
        unsafe { *str.add(i) = c as c_char };
    }
    str
}

/// Write `number` to `fd` in `len` bytes, most significant byte first.
///
/// @return  false in case of an error.
pub unsafe fn put_bytes(fd: *mut FILE, number: uintmax_t, len: size_t) -> bool {
    debug_assert!(len > 0);
    for i in (0..len).rev() {
        if unsafe { putc((number >> (i * 8)) as c_int, fd) } == EOF {
            return false;
        }
    }
    true
}

/// Write a `time_t` to `fd` in 8 bytes.
///
/// @return  FAIL when the write failed.
pub unsafe fn put_time(fd: *mut FILE, time_: time_t) -> c_int {
    let mut buf = [0u8; 8];
    unsafe { time_to_bytes(time_, buf.as_mut_ptr()) };
    // Upstream compares `fwrite`'s item count against 1 while asking it
    // for 8 one-byte items, so this always answers FAIL. Both callers
    // ignore the answer. Preserved.
    if unsafe { fwrite(buf.as_ptr().cast(), 1, buf.len(), fd) } == 1 {
        OK
    } else {
        FAIL
    }
}

/// Version of `read()` that retries when interrupted by a signal, which
/// `SIGWINCH` makes routine.
pub unsafe fn read_eintr(fd: c_int, buf: *mut c_void, bufsize: size_t) -> ssize_t {
    loop {
        let ret = unsafe { read(fd, buf, bufsize) };
        if ret >= 0 || unsafe { *__errno_location() } != EINTR {
            return ret;
        }
    }
}

/// Version of `write()` that retries when interrupted by a signal.
///
/// Repeats the write for as long as it doesn't fail for a reason other than
/// being interrupted; the caller compares the result against `bufsize` to see
/// whether everything got out.
pub unsafe fn write_eintr(fd: c_int, buf: *mut c_void, bufsize: size_t) -> ssize_t {
    let mut written: ssize_t = 0;
    while (written as size_t) < bufsize {
        let from = buf.cast::<c_char>().wrapping_offset(written).cast();
        let left = bufsize - written as size_t;
        // SAFETY: `buf` holds `bufsize` bytes and `written` of them have
        // gone out already.
        let wlen = unsafe { write(fd, from, left) };
        if wlen < 0 {
            if unsafe { *__errno_location() } != EINTR {
                break;
            }
        } else {
            written += wlen;
        }
    }
    written
}
pub const SEA_QUIT: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
pub const BAD_REPLACE: ::core::ffi::c_int = '?' as ::core::ffi::c_int;
pub const BAD_KEEP: ::core::ffi::c_int = -1 as ::core::ffi::c_int;
pub const BAD_DROP: ::core::ffi::c_int = -2 as ::core::ffi::c_int;
pub const FORCE_BIN: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const FORCE_NOBIN: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
pub const ENC_UCSBOM: &::core::ffi::CStr = c"ucs-bom";
pub const EOL_UNKNOWN: ::core::ffi::c_int = -1 as ::core::ffi::c_int;
pub const EOL_UNIX: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const EOL_DOS: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const EOL_MAC: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
pub const EOVERFLOW: ::core::ffi::c_int = 75 as ::core::ffi::c_int;
pub const EINTR: ::core::ffi::c_int = 4 as ::core::ffi::c_int;
pub const INT_MAX: ::core::ffi::c_int = __INT_MAX__;
pub const __S_IFMT: ::core::ffi::c_int = 0o170000 as ::core::ffi::c_int;
pub const NAME_MAX: ::core::ffi::c_int = 255 as ::core::ffi::c_int;
pub const __INT_MAX__: ::core::ffi::c_int = 2147483647 as ::core::ffi::c_int;

/// Fill `eap` so that `'fileencoding'`, `'fileformat'` and `'binary'` are
/// forced to what buffer `buf` already has. Used when calling `readfile` to
/// re-read a buffer that is already open.
pub unsafe fn prep_exarg(eap: *mut exarg_T, buf: Buf) {
    // SAFETY: the caller's command, live for the call.
    let mut ea = unsafe { Ea::new(eap) };
    // SAFETY: the buffer's own NUL-terminated 'fileencoding'.
    let cmd_len = 15 + unsafe { strlen(buf.b_p_fenc) };
    ea.cmd = unsafe { xmalloc(cmd_len) }.cast();
    unsafe { snprintf(ea.cmd, cmd_len, c"e ++enc=%s".as_ptr(), buf.b_p_fenc) };
    // Where the encoding name starts in that command.
    ea.force_enc = 8;
    ea.bad_char = buf.b_bad_char;
    // SAFETY: 'fileformat' is the buffer's own one-character option string.
    ea.force_ff = unsafe { *buf.b_p_ff } as u8 as c_int;
    ea.force_bin = if buf.b_p_bin != 0 {
        FORCE_BIN
    } else {
        FORCE_NOBIN
    };
    ea.read_edit = false as c_int;
    ea.forceit = false as c_int;
}

/// Set the default or forced `'fileformat'` and `'binary'`.
pub unsafe fn set_file_options(set_options: bool, eap: *mut exarg_T) {
    // Set the default 'fileformat'.
    if set_options {
        if !eap.is_null() && unsafe { (*eap).force_ff } != 0 {
            set_fileformat(
                unsafe { get_fileformat_force(cur_buf(), eap) },
                OptionSetFlags::LOCAL,
            );
        } else if unsafe { *p_ffs.get() } != 0 {
            set_fileformat(default_fileformat(), OptionSetFlags::LOCAL);
        }
    }

    // Set or reset 'binary'.
    if !eap.is_null() && unsafe { (*eap).force_bin } != 0 {
        let oldval = cur_buf().b_p_bin;
        cur_buf().b_p_bin = (unsafe { (*eap).force_bin } == FORCE_BIN) as c_int;
        let bin = cur_buf().b_p_bin != 0;
        set_options_bin(oldval != 0, bin, OptionSetFlags::LOCAL);
    }
}

/// Set the forced `'fileencoding'` from a `++enc=` argument.
pub unsafe fn set_forced_fenc(eap: *mut exarg_T) {
    // SAFETY: the caller's command, live for the call.
    let ea = unsafe { Ea::new(eap) };
    if ea.force_enc == 0 {
        return;
    }
    let fenc = unsafe { enc_canonize(ea.cmd.offset(ea.force_enc as isize)) };
    set_option_direct(
        kOptFileencoding,
        OptVal {
            type_0: kOptValTypeString,
            data: OptValData {
                string: unsafe { cstr_as_string(fenc) },
            },
        },
        OptionSetFlags::LOCAL,
        0 as scid_T,
    );
    unsafe { xfree(fenc.cast()) };
}

/// The buffer the editor is working in.
fn cur_buf() -> Buf {
    // SAFETY: `curbuf` is set from startup to exit.
    unsafe { Buf::current() }
}
