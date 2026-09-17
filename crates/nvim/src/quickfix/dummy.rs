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
//! is re-checked through a `BufferRef` rather than trusted across such a
//! call.

#![deny(unsafe_op_in_unsafe_fn)]
#![allow(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use super::*;
use crate::buffer::BufFlags;
use crate::buffer::BufRef;
use crate::cstr;
use crate::types::CmdIdx;
use crate::types::{CmdLine, MAXPATHL, OK};
use crate::winlayer::{Buf, windows};
use core::ffi::c_char;
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
    // SAFETY (both): one owned MAXPATHL buffer, and the caller's directory
    // name, which is NUL-terminated.
    let (now, start) = unsafe {
        let _ = os_dirname(dirname_now.as_mut_ptr(), MAXPATHL as size_t);
        (
            cstr::bytes_at(dirname_now.as_ptr()),
            cstr::bytes_at(dirname_start),
        )
    };
    if now == start {
        return;
    }
    // Return to the original directory, ignoring any error.
    let mut ea = ExArg {
        line: CmdLine::from_bytes(start),
        cmdidx: if Win::current().w_localdir.is_null() {
            CmdIdx::cd
        } else {
            CmdIdx::lcd
        },
        ..Default::default()
    };
    ex_cd(&mut ea);
}

/// Load `fname` into a dummy buffer and answer it, or `None` when the file
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
) -> Option<Buf> {
    // SAFETY: forwarded from the caller.
    // Allocate a buffer without putting it in the buffer list.
    let mut newbuf =
        unsafe { buflist_new(ptr::null_mut(), ptr::null_mut(), 1, BLN_DUMMY.cast_signed()) }?;

    let mut failed = true;
    // SAFETY: `buflist_new` answered this buffer a moment ago.
    let newbufref = BufRef::of(newbuf);

    // Init the options.
    buf_copy_options(newbuf, (BCO_ENTER | BCO_NOHELP).cast_signed());

    // Need to open the memfile before putting the buffer in a window.
    if ml_open(newbuf).is_ok() {
        // Make sure this buffer isn't wiped out by autocommands.
        newbuf.b_locked += 1;
        // Set curwin/curbuf to buf and save a few things.
        let mut aco = AcoSave::default();
        unsafe { aucmd_prepbuf(&raw mut aco, newbuf) };

        // Need to set the filename for autocommands.
        let _ = unsafe { setfname(Buf::current(), fname, ptr::null_mut(), false) };

        // Create swap file now to avoid the ATTENTION message.
        check_need_swap(true);

        // Remove the "dummy" flag, otherwise autocommands may not
        // work.
        Buf::current().b_flags.clear(BufFlags::DUMMY);

        let mut newbuf_to_wipe = BufRef::NONE;
        let sfname = ptr::null_mut();
        let lines_to_read = MAXLNUM;
        let flags = (READ_NEW | READ_DUMMY).cast_signed();
        let readfile_result =
            unsafe { readfile(fname, sfname, 0, 0, lines_to_read, None, flags, false) };
        newbuf.b_locked -= 1;
        if readfile_result.is_ok() && !got_int.get() && !Buf::current().b_flags.has(BufFlags::NEW) {
            failed = false;
            if Buf::current_or_none() != Some(newbuf) {
                // Bloody autocommands changed the buffer! Restore
                // the original buffer and wipe the new one later.
                newbuf_to_wipe = BufRef::of(newbuf);
                newbuf = Buf::current();
            }
        }

        // Restore curwin/curbuf and a few other things.
        unsafe { aucmd_restbuf(&raw mut aco) };

        if let Some(to_wipe) = newbuf_to_wipe.get() {
            block_autocmds();
            // SAFETY: `BufRef::get` just established the buffer.
            unsafe { wipe_dummy_buffer(to_wipe, ptr::null()) };
            unblock_autocmds();
        }

        // Add back the "dummy" flag, otherwise buflist_findname_file_id()
        // won't skip it.
        newbuf.b_flags |= BufFlags::DUMMY;
    }

    // When autocommands/'autochdir' option changed directory: go back.
    // Let the caller know where it went.
    let _ = unsafe { os_dirname(resulting_dir, MAXPATHL as size_t) };
    unsafe { restore_start_dir(dirname_start) };

    if !newbufref.valid() {
        return None;
    }
    // SAFETY: `BufRef::valid` just established the buffer.
    let newbuf = newbuf;
    if failed {
        // SAFETY: `dirname_start` is the caller's NUL-terminated string.
        unsafe { wipe_dummy_buffer(newbuf, dirname_start) };
        return None;
    }
    Some(newbuf)
}

/// Wipe out the dummy buffer, closing every window that shows it first.
/// When a window will not close, the buffer merely stops being a dummy and
/// stays around as an ordinary one.
///
/// # Safety
///
/// `buffer` must be a live buffer; `dirname_start` must be null or
/// NUL-terminated.
pub(crate) unsafe fn wipe_dummy_buffer(mut buffer: Buf, dirname_start: *const c_char) {
    // Note: `win_close` drops `b_nwindows` behind the raw pointer.
    #[allow(clippy::while_immutable_condition)]
    while buffer.b_nwindows > 0 {
        // Only close the window if it is not the last one, and only when
        // closing it actually worked — otherwise this would spin.
        let mut did_one = false;
        if windows().nth(1).is_some()
            && let Some(wp) = windows().find(|wp| ptr::eq(wp.w_buffer, buffer.raw()))
        {
            did_one = win_close(wp, false, false) == OK;
        }
        if !did_one {
            // The buffer keeps a window; it can only stop being a dummy.
            buffer.b_flags.clear(BufFlags::DUMMY);
            return;
        }
    }

    if !ptr::eq(Buf::current_raw(), buffer.raw().cast_const()) && buffer.b_nwindows == 0 {
        // Delete the buffer and its swap file. `wipe_buffer` calls
        // `close_buffer`, which may run autocommands, so a pending
        // exception or `:return` has to be parked over the call.
        let mut cs = Cleanup {
            pending: 0,
            exception: ptr::null_mut(),
        };
        unsafe { enter_cleanup(&raw mut cs) };
        wipe_buffer(buffer, true);
        unsafe { leave_cleanup(&raw mut cs) };

        // When autocommands/'autochdir' option changed directory: go back.
        if !dirname_start.is_null() {
            unsafe { restore_start_dir(dirname_start) };
        }
        return;
    }

    buffer.b_flags.clear(BufFlags::DUMMY);
}

/// Unload the dummy buffer that `load_dummy_buffer` created, keeping it in
/// the buffer list so that a later `:vimgrep` finds it again.
///
/// # Safety
///
/// `dirname_start` must be NUL-terminated.
pub(crate) unsafe fn unload_dummy_buffer(buffer: Buf, dirname_start: *const c_char) {
    if ptr::eq(Buf::current_raw(), buffer.raw()) {
        return;
    }
    close_buffer(None, buffer, DOBUF_UNLOAD.cast_signed(), false, true);

    // When autocommands/'autochdir' option changed directory: go back.
    unsafe { restore_start_dir(dirname_start) };
}
