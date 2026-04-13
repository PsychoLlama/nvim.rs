//! Miscellaneous standalone buffer functions
//!
//! This module contains small, self-contained buffer utility functions
//! migrated from `src/nvim/buffer.c` in Phase 1.

#![allow(clippy::cast_possible_wrap)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::missing_safety_doc)]

use std::ffi::{c_char, c_int, c_ulong, c_void};
use std::ptr::addr_of_mut;

use crate::{buf_struct::buf_ref, BufHandle};

// =============================================================================
// External C Functions
// =============================================================================

extern "C" {
    // curbuf accessors
    fn nvim_get_curbuf() -> BufHandle;
    fn nvim_curbuf_get_ffname() -> *const c_char;
    fn nvim_curbuf_ml_line_count() -> c_int;
    fn nvim_get_curbuf_ml_flags() -> c_int;

    // Buffer accessors
    fn nvim_buf_channel_job_running(buf: BufHandle) -> c_int;

    // Option accessors
    fn nvim_get_p_acd() -> c_int;

    // C functions we call
    fn gettext(msgid: *const c_char) -> *const c_char;
    fn emsg(s: *const c_char) -> bool;
    fn vim_chdirfile(fname: *mut c_char, cause: c_int) -> c_int;
    fn shorten_fnames(force: bool);
    fn extmark_free_all(buf: BufHandle);
    fn ml_delete(lnum: c_int) -> c_int;
    fn deleted_lines_mark(lnum: c_int, count: c_int);
    fn text_locked() -> bool;
    fn get_text_locked_msg() -> *const c_char;

    // For curbuf_locked / allbuf_locked
    fn nvim_buf_get_b_ro_locked(buf: BufHandle) -> c_int;
    static allbuf_lock: c_int;
    static e_cannot_edit_other_buf: c_char;

    fn do_buffer_ext(action: c_int, start: c_int, dir: c_int, count: c_int, flags: c_int) -> c_int;
    fn buflist_findpat(
        pattern: *const c_char,
        pattern_end: *const c_char,
        unlisted: bool,
        diffmode: bool,
        curtab_only: bool,
    ) -> c_int;
    fn skipwhite(p: *const c_char) -> *mut c_char;
    fn skiptowhite_esc(p: *const c_char) -> *mut c_char;
    fn rs_ascii_isdigit(c: c_int) -> c_int;
    fn getdigits_int(pp: *mut *mut c_char, strict: bool, def: c_int) -> c_int;
    fn os_breakcheck();
    fn xstrlcpy(dst: *mut c_char, src: *const c_char, dstsize: usize) -> usize;
    fn smsg(hl_id: c_int, fmt: *const c_char, ...) -> c_int;
    fn ngettext(s1: *const c_char, s2: *const c_char, n: c_ulong) -> *const c_char;
    fn ex_errmsg(msg: *const c_char, arg: *const c_char) -> *mut c_char;

    // Global statics for do_bufdel
    static mut IObuff: [c_char; 1025];
    #[link_name = "got_int"]
    static mut nvim_got_int: bool;
    static p_report: i64;
}

// =============================================================================
// External C Statics
// =============================================================================

extern "C" {
    static mut starting: c_int;
    static mut last_chdir_reason: *const c_char;
}

// =============================================================================
// Constants
// =============================================================================

/// OK return value from C functions
const OK: c_int = 1;
/// `ML_EMPTY` flag value (memline has no lines)
const ML_EMPTY: c_int = 0x01;
/// kCdCauseAuto = 2 (from `vim_defs.h`: Other=-1, Manual=0, Window=1, Auto=2)
const K_CD_CAUSE_AUTO: c_int = 2;
/// `FORWARD` direction constant (from `vim_defs.h`)
const FORWARD: c_int = 1;
/// `DOBUF_CURRENT` start value (from `buffer.h`)
const DOBUF_CURRENT: c_int = 0;
/// `DOBUF_FIRST` start value (from `buffer.h`)
const DOBUF_FIRST: c_int = 1;
/// `DOBUF_WIPE` action value (from `buffer.h`)
const DOBUF_WIPE: c_int = 4;
/// `DOBUF_DEL` action value (from `buffer.h`)
const DOBUF_DEL: c_int = 3;
/// `DOBUF_UNLOAD` action value (from `buffer.h`)
const DOBUF_UNLOAD: c_int = 2;
/// `DOBUF_FORCEIT` flag value (from `buffer.h`)
const DOBUF_FORCEIT: c_int = 1;
/// Buffer for I/O size (from `globals.h`)
const IOSIZE: usize = 1024 + 1;

// =============================================================================
// bufref_T layout (must match buffer_defs.h exactly)
// =============================================================================

/// Rust mirror of `bufref_T` from `buffer_defs.h`.
///
/// # Safety
/// This struct MUST match the C layout exactly:
/// `{ buf_T *br_buf; int br_fnum; int br_buf_free_count; }`
#[repr(C)]
pub struct BufRef {
    pub br_buf: *mut c_void, // buf_T*
    pub br_fnum: c_int,
    pub br_buf_free_count: c_int,
}

// =============================================================================
// Phase 1 Implementations
// =============================================================================

/// Change to the directory of the current buffer.
///
/// Rust port of C `do_autochdir()`.
///
/// # Safety
/// Accesses global Neovim state.
#[no_mangle]
pub unsafe extern "C" fn do_autochdir() {
    if nvim_get_p_acd() != 0 {
        let ffname = nvim_curbuf_get_ffname();
        if starting == 0 && !ffname.is_null() {
            // vim_chdirfile takes a mutable pointer; const-cast is safe here
            // as the function only reads the string.
            let fname_mut = ffname.cast_mut();
            if vim_chdirfile(fname_mut, K_CD_CAUSE_AUTO) == OK {
                last_chdir_reason = c"autochdir".as_ptr();
                shorten_fnames(true);
            }
        }
    }
}

/// Emit an error for a buffer that has unsaved changes.
///
/// Rust port of C `no_write_message()`.
///
/// # Safety
/// Accesses global Neovim state.
#[no_mangle]
pub unsafe extern "C" fn no_write_message() {
    let curbuf = nvim_get_curbuf();
    if nvim_buf_channel_job_running(curbuf) != 0 {
        emsg(gettext(
            c"E948: Job still running (add ! to end the job)".as_ptr(),
        ));
    } else {
        emsg(gettext(
            c"E37: No write since last change (add ! to override)".as_ptr(),
        ));
    }
}

/// Emit an error for a buffer that has unsaved changes (no-bang variant).
///
/// Rust port of C `no_write_message_nobang()`.
///
/// # Safety
/// `buf` must be a valid buffer handle.
#[no_mangle]
pub unsafe extern "C" fn no_write_message_nobang(buf: BufHandle) {
    if nvim_buf_channel_job_running(buf) != 0 {
        emsg(gettext(c"E948: Job still running".as_ptr()));
    } else {
        emsg(gettext(c"E37: No write since last change".as_ptr()));
    }
}

/// Emit an error message when text is locked (cmdline window open, etc.).
///
/// Rust port of C `text_locked_msg()`.
///
/// # Safety
/// Accesses global Neovim state.
#[no_mangle]
pub unsafe extern "C" fn text_locked_msg() {
    emsg(gettext(get_text_locked_msg()));
}

/// Check for text, window or buffer locked.
///
/// Returns `true` and emits an error message if something is locked.
///
/// Rust port of C `text_or_buf_locked()`.
///
/// # Safety
/// Accesses global Neovim state.
#[no_mangle]
pub unsafe extern "C" fn text_or_buf_locked() -> bool {
    if text_locked() {
        text_locked_msg();
        return true;
    }
    curbuf_locked()
}

// Check if allbuf_lock is set and return true when it is and give an error message.
// Rust port of C allbuf_locked().
/// # Safety
/// Accesses global Neovim state.
#[no_mangle]
pub unsafe extern "C" fn allbuf_locked() -> bool {
    if allbuf_lock > 0 {
        emsg(c"E811: Not allowed to change buffer information now".as_ptr());
        return true;
    }
    false
}

// Check if curbuf->b_ro_locked or allbuf_lock is set and give an error message.
// Rust port of C curbuf_locked().
/// # Safety
/// Accesses global Neovim state.
#[no_mangle]
pub unsafe extern "C" fn curbuf_locked() -> bool {
    let curbuf = nvim_get_curbuf();
    if nvim_buf_get_b_ro_locked(curbuf) > 0 {
        emsg(gettext(&raw const e_cannot_edit_other_buf));
        return true;
    }
    allbuf_locked()
}

/// Clear the current buffer contents.
///
/// Deletes all lines and extmarks from `curbuf`.
///
/// Rust port of C `buf_clear()`.
///
/// # Safety
/// Accesses global `curbuf`. Must be called on the main Neovim thread.
#[no_mangle]
pub unsafe extern "C" fn buf_clear() {
    let line_count = nvim_curbuf_ml_line_count();
    let curbuf = nvim_get_curbuf();
    extmark_free_all(curbuf);
    while nvim_get_curbuf_ml_flags() & ML_EMPTY == 0 {
        ml_delete(1);
    }
    deleted_lines_mark(1, line_count);
}

/// Initialize a buffer reference (`bufref_T`).
///
/// Sets `br_buf`, `br_fnum`, and `br_buf_free_count` on the given reference.
///
/// Rust port of C `set_bufref()`.
///
/// # Safety
/// `bufref` must be a valid non-null pointer to a `bufref_T`.
/// `buf` may be null (represents no buffer).
#[no_mangle]
pub unsafe extern "C" fn set_bufref(bufref: *mut BufRef, buf: BufHandle) {
    if bufref.is_null() {
        return;
    }
    let br = &mut *bufref;
    br.br_buf = buf.as_ptr();
    br.br_fnum = if buf.is_null() {
        0
    } else {
        buf_ref(buf).handle
    };
    br.br_buf_free_count = crate::state::get_buf_free_count();
}

// =============================================================================
// Phase 2: do_bufdel
// =============================================================================

/// Delete or unload buffer(s).
///
/// Rust port of C `do_bufdel()`.
///
/// Returns an error message string, or NULL on success.
///
/// # Safety
/// All pointers must be valid. Accesses global Neovim state.
#[no_mangle]
pub unsafe extern "C" fn do_bufdel(
    command: c_int,
    arg: *mut c_char,
    addr_count: c_int,
    start_bnr: c_int,
    end_bnr: c_int,
    forceit: c_int,
) -> *mut c_char {
    let mut arg = arg;
    let mut do_current: c_int = 0; // delete current buffer?
    let mut deleted: c_int = 0; // number of buffers deleted
    let mut errormsg: *mut c_char = std::ptr::null_mut();

    let forceit_flag = if forceit != 0 { DOBUF_FORCEIT } else { 0 };
    if addr_count == 0 {
        do_buffer_ext(command, DOBUF_CURRENT, FORWARD, 0, forceit_flag);
    } else {
        let mut bnr = if addr_count == 2 {
            if !arg.is_null() && *arg != 0 {
                // Both range and argument is not allowed
                return ex_errmsg(c"E488: Trailing characters: %s".as_ptr(), arg);
            }
            start_bnr
        } else {
            // addr_count == 1
            end_bnr
        };

        while !nvim_got_int {
            os_breakcheck();

            let curbuf = nvim_get_curbuf();
            if bnr == buf_ref(curbuf).handle {
                do_current = bnr;
            } else if do_buffer_ext(command, DOBUF_FIRST, FORWARD, bnr, forceit_flag) == OK {
                deleted += 1;
            }

            // Find next buffer number to delete/unload
            if addr_count == 2 {
                bnr += 1;
                if bnr > end_bnr {
                    break;
                }
            } else {
                // addr_count == 1
                arg = skipwhite(arg);
                if arg.is_null() || *arg == 0 {
                    break;
                }
                if rs_ascii_isdigit(c_int::from(*arg)) == 0 {
                    let p = skiptowhite_esc(arg);
                    bnr = buflist_findpat(arg, p, command == DOBUF_WIPE, false, false);
                    if bnr < 0 {
                        // failed
                        break;
                    }
                    arg = p;
                } else {
                    let arg_ptr: *mut *mut c_char = &raw mut arg;
                    bnr = getdigits_int(arg_ptr, false, 0);
                }
            }
        }

        if !nvim_got_int
            && do_current != 0
            && do_buffer_ext(command, DOBUF_FIRST, FORWARD, do_current, forceit_flag) == OK
        {
            deleted += 1;
        }

        if deleted == 0 {
            let msg = if command == DOBUF_UNLOAD {
                gettext(c"E515: No buffers were unloaded".as_ptr())
            } else if command == DOBUF_DEL {
                gettext(c"E516: No buffers were deleted".as_ptr())
            } else {
                gettext(c"E517: No buffers were wiped out".as_ptr())
            };
            // Use addr_of_mut! to avoid creating a mutable reference to a mutable static
            let iobuff_ptr = addr_of_mut!(IObuff).cast::<c_char>();
            xstrlcpy(iobuff_ptr, msg, IOSIZE);
            errormsg = iobuff_ptr;
        } else if i64::from(deleted) >= p_report {
            // deleted > 0 here (non-zero, >= p_report path)
            let n = c_ulong::from(deleted.unsigned_abs());
            if command == DOBUF_UNLOAD {
                let fmt = ngettext(
                    c"%d buffer unloaded".as_ptr(),
                    c"%d buffers unloaded".as_ptr(),
                    n,
                );
                smsg(0, fmt, deleted);
            } else if command == DOBUF_DEL {
                let fmt = ngettext(
                    c"%d buffer deleted".as_ptr(),
                    c"%d buffers deleted".as_ptr(),
                    n,
                );
                smsg(0, fmt, deleted);
            } else {
                let fmt = ngettext(
                    c"%d buffer wiped out".as_ptr(),
                    c"%d buffers wiped out".as_ptr(),
                    n,
                );
                smsg(0, fmt, deleted);
            }
        }
    }

    errormsg
}

// =============================================================================
// read_buffer_into helpers
// =============================================================================

extern "C" {
    fn nvim_ml_get_buf(buf: BufHandle, lnum: c_int) -> *const c_char;
    // nvim_ml_get_buf_len: takes *mut c_void in quickfix_shim.c
    fn nvim_ml_get_buf_len(buf: *mut c_void, lnum: c_int) -> c_int;
    fn nvim_buf_ml_is_empty(buf: BufHandle) -> bool;
    fn nvim_buf_get_no_eol_lnum(buf: BufHandle) -> c_int;
    fn nvim_buf_get_b_p_fixeol(buf: BufHandle) -> bool;
    fn nvim_buf_get_b_p_eol(buf: BufHandle) -> bool;
    fn nvim_sb_push_byte(sb: *mut c_void, byte: c_char);
    fn nvim_sb_concat_len(sb: *mut c_void, ptr: *const c_char, len: usize);
}

const NL_BYTE: u8 = b'\n';
const NUL_BYTE: u8 = b'\0';
const BLN_DUMMY: c_int = 8; // from buffer.h: don't add to buffer list, don't clip name

/// Read buffer contents from lines [start, end] into a C `StringBuilder`.
///
/// Handles NL<->/dev/null translation (Vim stores NUL as NL in the memline).
/// Appends a trailing newline for each line unless suppressed by
/// `'fixeol'`, `'eol'`, `'bin'`, or the `b_no_eol_lnum` marker.
///
/// # Safety
///
/// Must be called on the Neovim main thread. `sb` must be a valid `StringBuilder *`.
#[unsafe(export_name = "read_buffer_into")]
pub unsafe extern "C" fn rs_read_buffer_into(
    buf: BufHandle,
    start: c_int,
    end: c_int,
    sb: *mut c_void,
) {
    if nvim_buf_ml_is_empty(buf) {
        return;
    }

    let ml_line_count = buf_ref(buf).ml_line_count;
    let no_eol_lnum = nvim_buf_get_no_eol_lnum(buf);
    let bin = buf_ref(buf).b_p_bin != 0;
    let fixeol = nvim_buf_get_b_p_fixeol(buf);
    let eol = nvim_buf_get_b_p_eol(buf);

    let mut lnum = start;
    let mut lp = nvim_ml_get_buf(buf, lnum);
    let mut lplen = nvim_ml_get_buf_len(buf.as_ptr(), lnum) as usize;
    let mut written: usize = 0;

    loop {
        let len: usize;
        if lplen == 0 {
            len = 0;
        } else {
            let ch = *lp.add(written) as u8;
            if ch == NL_BYTE {
                // NL -> /dev/null translation
                len = 1;
                nvim_sb_push_byte(sb, NUL_BYTE as c_char);
            } else {
                // Find next NL or end of available bytes
                let remaining = lplen - written;
                let slice = std::slice::from_raw_parts(lp.add(written).cast::<u8>(), remaining);
                let found = slice.iter().position(|&b| b == NL_BYTE);
                len = found.unwrap_or(remaining);
                nvim_sb_concat_len(sb, lp.add(written), len);
            }
        }

        if len == lplen - written {
            // Finished a line; emit a trailing NL unless suppressed.
            if lnum != end
                || (!bin && fixeol)
                || (lnum != no_eol_lnum && (lnum != ml_line_count || eol))
            {
                nvim_sb_push_byte(sb, NL_BYTE as c_char);
            }
            lnum += 1;
            if lnum > end {
                break;
            }
            lp = nvim_ml_get_buf(buf, lnum);
            lplen = nvim_ml_get_buf_len(buf.as_ptr(), lnum) as usize;
            written = 0;
        } else if len > 0 {
            written += len;
        }
    }
}

// =============================================================================
// buf_contents_changed
// =============================================================================

extern "C" {
    #[link_name = "buflist_new"]
    fn nvim_buflist_new(
        ffname: *const c_char,
        sfname: *const c_char,
        lnum: c_int,
        flags: c_int,
    ) -> BufHandle;
    fn nvim_buf_prep_exarg_alloc(buf: BufHandle) -> *mut c_void;
    fn nvim_exarg_free(ea: *mut c_void);
    fn nvim_buf_aucmd_prepbuf_alloc(buf: BufHandle) -> *mut c_void;
    fn nvim_buf_aucmd_restbuf_free(aco: *mut c_void);
    fn nvim_ml_open_curbuf() -> c_int;
    fn nvim_readfile_for_buf(buf: BufHandle, ea: *mut c_void) -> c_int;
    #[link_name = "ml_get_buf"]
    fn nvim_buf_get_line_at(buf: BufHandle, lnum: c_int) -> *const c_char;
    #[link_name = "ml_get"]
    fn nvim_curbuf_get_line_at(lnum: c_int) -> *const c_char;
    #[link_name = "block_autocmds"]
    fn nvim_block_autocmds();
    #[link_name = "unblock_autocmds"]
    fn nvim_unblock_autocmds();
}

/// Read the file for `buf` again and check whether the contents changed.
///
/// Returns `true` if the contents differ or if the check could not be performed.
///
/// Mirrors C `buf_contents_changed`.
///
/// # Safety
///
/// Must be called on the main thread with valid Neovim state.
#[must_use]
#[unsafe(export_name = "buf_contents_changed")]
pub unsafe extern "C" fn rs_buf_contents_changed(buf: BufHandle) -> bool {
    // Allocate a dummy buffer (not in the buffer list).
    let newbuf = nvim_buflist_new(std::ptr::null_mut(), std::ptr::null_mut(), 1, BLN_DUMMY);
    if newbuf.is_null() {
        return true;
    }

    // Force 'fileencoding' and 'fileformat' to be equal.
    let ea = nvim_buf_prep_exarg_alloc(buf);

    // Set curwin/curbuf to newbuf and save a few things.
    let aco = nvim_buf_aucmd_prepbuf_alloc(newbuf);

    // Block autocommands to avoid nasty side-effects (e.g. wiping buffers).
    nvim_block_autocmds();

    let mut differ = true;

    if nvim_ml_open_curbuf() == OK && nvim_readfile_for_buf(buf, ea) == OK {
        // Compare the two files line by line.
        let buf_lines = buf_ref(buf).ml_line_count;
        let curbuf_lines = nvim_curbuf_ml_line_count();
        if buf_lines == curbuf_lines {
            differ = false;
            let mut lnum = 1;
            while lnum <= curbuf_lines {
                let buf_line = nvim_buf_get_line_at(buf, lnum);
                let cur_line = nvim_curbuf_get_line_at(lnum);
                // Both pointers are valid C strings from the memline.
                let buf_bytes = std::ffi::CStr::from_ptr(buf_line).to_bytes();
                let cur_bytes = std::ffi::CStr::from_ptr(cur_line).to_bytes();
                if buf_bytes != cur_bytes {
                    differ = true;
                    break;
                }
                lnum += 1;
            }
        }
    }

    nvim_exarg_free(ea);

    // Restore curwin/curbuf.
    nvim_buf_aucmd_restbuf_free(aco);

    let curbuf = nvim_get_curbuf();
    if curbuf != newbuf {
        // safety check: only wipe if curbuf moved away from newbuf
        crate::state::rs_wipe_buffer(newbuf, false);
    }

    nvim_unblock_autocmds();

    differ
}

// =============================================================================
// read_buffer: read file into buffer for retrying (stdin/fifo encoding retry)
// =============================================================================

extern "C" {
    fn nvim_curbuf_get_fname() -> *const c_char;
    fn shortmess(x: c_int) -> bool;
    fn readfile(
        fname: *const c_char,
        sfname: *const c_char,
        from: c_int,
        lines_to_skip: c_int,
        lines_to_read: c_int,
        eap: *mut c_void,
        flags: c_int,
        silent: bool,
    ) -> c_int;
    fn nvim_curwin_set_cursor(lnum: c_int, col: c_int);
    static mut readonlymode: bool;
    fn buf_is_empty(buf: BufHandle) -> bool;
    fn changed(buf: BufHandle);
    fn unchanged(buf: BufHandle, ff: bool, always_inc_changedtick: bool);
    fn apply_autocmds_retval(
        event: c_int,
        fname: *mut c_char,
        fname_io: *mut c_char,
        force: bool,
        buf: *mut c_void,
        retval: *mut c_int,
    ) -> bool;
}

// SHM_FILEINFO = 'F' (from option_vars.h)
const SHM_FILEINFO: c_int = b'F' as c_int;
// READ_BUFFER flag (from fileio.h)
const READ_BUFFER: c_int = 0x08;
// MAXLNUM (from pos_defs.h)
const MAXLNUM: c_int = 0x7fff_ffff;
// FAIL/OK return values
const READ_FAIL: c_int = 0;
// EVENT_STDINREADPOST = 105 (from auevents_enum.generated.h)
const EVENT_STDINREADPOST: c_int = 105;

/// Read data from the current buffer for retrying with a different encoding.
///
/// Used by `open_buffer` when reading from stdin or a fifo, to re-read with
/// corrected 'fileformat'/'fileencoding' after binary pre-read.
///
/// # Safety
/// Accesses global Neovim state. Must be called on the main thread.
#[no_mangle]
pub unsafe extern "C" fn rs_read_buffer(read_stdin: bool, eap: *mut c_void, flags: c_int) -> c_int {
    // OK = 1, FAIL = 0 (from vim_defs.h)
    const OK_VAL: c_int = 1;
    let silent = shortmess(SHM_FILEINFO);

    // Read from the buffer which the text is already filled in and append at
    // the end.  This makes it possible to retry when 'fileformat' or
    // 'fileencoding' was guessed wrong.
    let line_count = nvim_curbuf_ml_line_count();
    let mut retval = readfile(
        if read_stdin {
            std::ptr::null()
        } else {
            nvim_curbuf_get_ffname()
        },
        if read_stdin {
            std::ptr::null()
        } else {
            nvim_curbuf_get_fname()
        },
        line_count,
        0,
        MAXLNUM,
        eap,
        flags | READ_BUFFER,
        silent,
    );

    if retval == OK_VAL {
        // Delete the binary lines.
        let mut lnum = line_count;
        while lnum > 0 {
            ml_delete(1);
            lnum -= 1;
        }
    } else {
        // Delete the converted lines.
        while nvim_curbuf_ml_line_count() > line_count {
            ml_delete(line_count);
        }
    }
    // Put the cursor on the first line.
    nvim_curwin_set_cursor(1, 0);

    if read_stdin {
        // Set or reset 'modified' before executing autocommands, so that
        // it can be changed there.
        let curbuf = nvim_get_curbuf();
        if !readonlymode && !buf_is_empty(curbuf) {
            changed(curbuf);
        } else if retval != READ_FAIL {
            unchanged(curbuf, false, true);
        }

        apply_autocmds_retval(
            EVENT_STDINREADPOST,
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            false,
            curbuf.as_ptr(),
            &raw mut retval,
        );
    }
    retval
}
