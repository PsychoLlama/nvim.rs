//! The throwaway buffers `:vimgrep` searches files in.
//!
//! A file that is not already open is read into a buffer that exists only
//! for the search ([`load_dummy_buffer`]) and is then thrown away again
//! ([`wipe_dummy_buffer`]) — unless it turned out to hold the first match,
//! in which case it stays so that the jump lands in a real buffer.
//!
//! Everything here fires autocommands: `buflist_new` runs `BufNew`, reading
//! the file runs the `BufRead` family, and closing a window runs `WinClosed`.
//! An autocommand can change the current directory, so every entry point
//! ends by putting it back ([`restore_start_dir`]), and every buffer pointer
//! is re-checked through a `bufref_T` rather than trusted across such a
//! call.

#![deny(unsafe_op_in_unsafe_fn)]

use super::*;
use crate::buffer::BufFlags;
use crate::types::{CMD_cd, CMD_lcd, MAXPATHL, OK};
use crate::winlayer::Buf;
use core::ffi::{c_char, c_int};
use core::ptr;

/// Change back to `dirname_start` if an autocommand moved somewhere else.
/// A window with a local directory gets `:lcd`, so that the window-local
/// setting is not silently promoted to a global one.
///
/// # Safety
///
/// `dirname_start` must be NUL-terminated.
pub(crate) unsafe fn restore_start_dir(dirname_start: *const c_char) {
    let mut dirname_now = [0 as c_char; MAXPATHL as usize];
    // SAFETY: the caller's directory name, and one owned MAXPATHL buffer.
    unsafe { os_dirname(dirname_now.as_mut_ptr(), MAXPATHL as size_t) };
    if unsafe { strcmp(dirname_start, dirname_now.as_ptr()) } == 0 {
        return;
    }
    // Return to the original directory, ignoring any error.
    let mut ea = exarg_T {
        arg: dirname_start.cast_mut(),
        cmdidx: if cur_win().w_localdir.is_null() {
            CMD_cd
        } else {
            CMD_lcd
        },
        ..Default::default()
    };
    unsafe { ex_cd(&raw mut ea) };
}

/// Load `fname` into a dummy buffer and answer it, or null when the file
/// could not be read. `resulting_dir` is filled with the directory the read
/// left the editor in, before it is put back to `dirname_start`.
///
/// # Safety
///
/// `fname` and `dirname_start` must be NUL-terminated, and `resulting_dir`
/// must have room for MAXPATHL bytes.
pub(crate) unsafe fn load_dummy_buffer(
    fname: *mut c_char,
    dirname_start: *const c_char,
    resulting_dir: *mut c_char,
) -> *mut buf_T {
    // SAFETY: forwarded from the caller.
    // Allocate a buffer without putting it in the buffer list.
    let mut newbuf =
        unsafe { buflist_new(ptr::null_mut(), ptr::null_mut(), 1, BLN_DUMMY as c_int) };
    if newbuf.is_null() {
        return ptr::null_mut();
    }

    let mut failed = true;
    let mut newbufref = bufref_T::default();
    unsafe { set_bufref(&raw mut newbufref, newbuf) };

    // Init the options.
    unsafe { buf_copy_options(newbuf, (BCO_ENTER | BCO_NOHELP) as c_int) };

    // Need to open the memfile before putting the buffer in a window.
    if unsafe { ml_open(newbuf) } == OK {
        // Make sure this buffer isn't wiped out by autocommands.
        unsafe { (*newbuf).b_locked += 1 };
        // Set curwin/curbuf to buf and save a few things.
        let mut aco = aco_save_T::default();
        unsafe { aucmd_prepbuf(&raw mut aco, newbuf) };

        // Need to set the filename for autocommands.
        unsafe { setfname(curbuf.get(), fname, ptr::null_mut(), false) };

        // Create swap file now to avoid the ATTENTION message.
        unsafe { check_need_swap(true) };

        // Remove the "dummy" flag, otherwise autocommands may not
        // work.
        cur_buf().b_flags.clear(BufFlags::DUMMY);

        let mut newbuf_to_wipe = bufref_T::default();
        let sfname = ptr::null_mut();
        let lines_to_read = MAXLNUM as linenr_T;
        let eap = ptr::null_mut();
        let flags = (READ_NEW | READ_DUMMY) as c_int;
        let readfile_result =
            unsafe { readfile(fname, sfname, 0, 0, lines_to_read, eap, flags, false) };
        unsafe { (*newbuf).b_locked -= 1 };
        if readfile_result == OK && !got_int.get() && !cur_buf().b_flags.has(BufFlags::NEW) {
            failed = false;
            if !ptr::eq(curbuf.get(), newbuf) {
                // Bloody autocommands changed the buffer! Restore
                // the original buffer and wipe the new one later.
                unsafe { set_bufref(&raw mut newbuf_to_wipe, newbuf) };
                newbuf = curbuf.get();
            }
        }

        // Restore curwin/curbuf and a few other things.
        unsafe { aucmd_restbuf(&raw mut aco) };

        if !newbuf_to_wipe.br_buf.is_null() && unsafe { bufref_valid(&raw mut newbuf_to_wipe) } {
            unsafe { block_autocmds() };
            unsafe { wipe_dummy_buffer(newbuf_to_wipe.br_buf, ptr::null()) };
            unsafe { unblock_autocmds() };
        }

        // Add back the "dummy" flag, otherwise buflist_findname_file_id()
        // won't skip it.
        unsafe { (*newbuf).b_flags |= BufFlags::DUMMY };
    }

    // When autocommands/'autochdir' option changed directory: go back.
    // Let the caller know where it went.
    unsafe { os_dirname(resulting_dir, MAXPATHL as size_t) };
    unsafe { restore_start_dir(dirname_start) };

    if !unsafe { bufref_valid(&raw mut newbufref) } {
        return ptr::null_mut();
    }
    if failed {
        unsafe { wipe_dummy_buffer(newbuf, dirname_start) };
        return ptr::null_mut();
    }
    newbuf
}

/// Wipe out the dummy buffer, closing every window that shows it first.
/// When a window will not close, the buffer merely stops being a dummy and
/// stays around as an ordinary one.
///
/// # Safety
///
/// `buf` must be a live buffer; `dirname_start` must be null or
/// NUL-terminated.
pub(crate) unsafe fn wipe_dummy_buffer(buf: *mut buf_T, dirname_start: *const c_char) {
    // SAFETY: the caller's promise -- a live `buf_T`.
    let mut buf = unsafe { Buf::new(buf) };
    // SAFETY: forwarded from the caller.
    // Note: `win_close` drops `b_nwindows` behind the raw pointer.
    #[allow(clippy::while_immutable_condition)]
    while buf.b_nwindows > 0 {
        // Only close the window if it is not the last one, and only when
        // closing it actually worked — otherwise this would spin.
        let mut did_one = false;
        if !unsafe { (*firstwin.get()).w_next.is_null() } {
            let mut wp = firstwin.get();
            while !wp.is_null() {
                if ptr::eq(unsafe { (*wp).w_buffer }, buf.raw().cast_const()) {
                    did_one = unsafe { win_close(wp, false, false) } == OK;
                    break;
                }
                wp = unsafe { (*wp).w_next };
            }
        }
        if !did_one {
            // The buffer keeps a window; it can only stop being a dummy.
            buf.b_flags.clear(BufFlags::DUMMY);
            return;
        }
    }

    if !ptr::eq(curbuf.get(), buf.raw().cast_const()) && buf.b_nwindows == 0 {
        // Delete the buffer and its swap file. `wipe_buffer` calls
        // `close_buffer`, which may run autocommands, so a pending
        // exception or `:return` has to be parked over the call.
        let mut cs = cleanup_T {
            pending: 0,
            exception: ptr::null_mut(),
        };
        unsafe { enter_cleanup(&raw mut cs) };
        unsafe { wipe_buffer(buf.raw(), true) };
        unsafe { leave_cleanup(&raw mut cs) };

        // When autocommands/'autochdir' option changed directory: go back.
        if !dirname_start.is_null() {
            unsafe { restore_start_dir(dirname_start) };
        }
        return;
    }

    buf.b_flags.clear(BufFlags::DUMMY);
}

/// Unload the dummy buffer that `load_dummy_buffer` created, keeping it in
/// the buffer list so that a later `:vimgrep` finds it again.
///
/// # Safety
///
/// `buf` must be a live buffer and `dirname_start` NUL-terminated.
pub(crate) unsafe fn unload_dummy_buffer(buf: *mut buf_T, dirname_start: *const c_char) {
    // SAFETY: forwarded from the caller.
    if ptr::eq(curbuf.get(), buf) {
        return;
    }
    unsafe { close_buffer(ptr::null_mut(), buf, DOBUF_UNLOAD as c_int, false, true) };

    // When autocommands/'autochdir' option changed directory: go back.
    unsafe { restore_start_dir(dirname_start) };
}
