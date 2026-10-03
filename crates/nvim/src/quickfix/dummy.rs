//! The throwaway buffers `:vimgrep` searches files in.
//!
//! A file that is not already open is read into a buffer that exists only
//! for the search ([`load_dummy_buffer`]) and is then thrown away again
//! ([`wipe_dummy_buffer`]) — unless it turned out to hold the first match,
//! in which case it stays so that the jump lands in a real buffer.
//!
//! Everything here fires autocommands: listing a buffer runs `BufNew`,
//! reading the file runs the `BufRead` family, and closing a window runs
//! `WinClosed`. An autocommand can change the current directory, so every
//! entry point ends by putting it back ([`restore_start_dir`]), and every
//! buffer is re-checked through a `BufRef` rather than trusted across such a
//! call.

#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use super::*;
use crate::autocmd::AucmdBuf;
use crate::buffer::{BufFlags, BufRef, buflist_add_name};
use crate::ex_eval::CleanupGuard;
use crate::fileio::read_dummy_file;
use crate::memory::XString;
use crate::os::fs::current_dir;
use crate::types::CmdIdx;
use crate::types::{CmdLine, OK};
use crate::winlayer::{Buf, windows};
use core::ffi::CStr;

/// Change back to `dirname_start` if an autocommand moved somewhere else.
/// A window with a local directory gets `:lcd`, so that the window-local
/// setting is not silently promoted to a global one.
pub(crate) fn restore_start_dir(dirname_start: &CStr) {
    let now = current_dir().unwrap_or_default();
    if now.as_cstr() == dirname_start {
        return;
    }
    // Return to the original directory, ignoring any error.
    let mut ea = ExArg {
        line: CmdLine::from_bytes(dirname_start.to_bytes()),
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
/// could not be read. `resulting_dir` is set to the directory the read left
/// the editor in, before it is put back to `dirname_start`.
pub(crate) fn load_dummy_buffer(
    fname: &CStr,
    dirname_start: &CStr,
    resulting_dir: &mut Option<XString>,
) -> Option<Buf> {
    // Allocate a buffer without putting it in the buffer list.
    let mut newbuf = buflist_add_name(None, 1, BLN_DUMMY.cast_signed())?;

    let mut failed = true;
    let newbufref = BufRef::of(newbuf);

    // Init the options.
    buf_copy_options(newbuf, (BCO_ENTER | BCO_NOHELP).cast_signed());

    // Need to open the memfile before putting the buffer in a window.
    if ml_open(newbuf).is_ok() {
        // Make sure this buffer isn't wiped out by autocommands.
        newbuf.b_locked += 1;
        // Set curwin/curbuf to buf and save a few things.
        let aco = AucmdBuf::enter(newbuf);

        // Need to set the filename for autocommands.
        let _ = setfname(Buf::current(), Some(fname), None, false);

        // Create swap file now to avoid the ATTENTION message.
        check_need_swap(true);

        // Remove the "dummy" flag, otherwise autocommands may not
        // work.
        Buf::current().b_flags.clear(BufFlags::DUMMY);

        let mut newbuf_to_wipe = BufRef::NONE;
        let read = read_dummy_file(fname);
        newbuf.b_locked -= 1;
        if read.is_ok() && !got_int.get() && !Buf::current().b_flags.has(BufFlags::NEW) {
            failed = false;
            if Buf::current_or_none() != Some(newbuf) {
                // Bloody autocommands changed the buffer! Restore
                // the original buffer and wipe the new one later.
                newbuf_to_wipe = BufRef::of(newbuf);
                newbuf = Buf::current();
            }
        }

        // Restore curwin/curbuf and a few other things.
        drop(aco);

        if let Some(to_wipe) = newbuf_to_wipe.get() {
            block_autocmds();
            wipe_dummy_buffer(to_wipe, None);
            unblock_autocmds();
        }

        // Add back the "dummy" flag, otherwise buflist_findname_file_id()
        // won't skip it.
        newbuf.b_flags |= BufFlags::DUMMY;
    }

    // When autocommands/'autochdir' option changed directory: go back.
    // Let the caller know where it went.
    *resulting_dir = current_dir();
    restore_start_dir(dirname_start);

    if !newbufref.valid() {
        return None;
    }
    if failed {
        wipe_dummy_buffer(newbuf, Some(dirname_start));
        return None;
    }
    Some(newbuf)
}

/// Wipe out the dummy buffer, closing every window that shows it first.
/// When a window will not close, the buffer merely stops being a dummy and
/// stays around as an ordinary one.
pub(crate) fn wipe_dummy_buffer(mut buffer: Buf, dirname_start: Option<&CStr>) {
    // Note: `win_close` drops `b_nwindows` behind the handle.
    #[allow(clippy::while_immutable_condition)]
    while buffer.b_nwindows > 0 {
        // Only close the window if it is not the last one, and only when
        // closing it actually worked — otherwise this would spin.
        let mut did_one = false;
        if windows().nth(1).is_some()
            && let Some(wp) = windows().find(|wp| wp.w_buffer == buffer)
        {
            did_one = win_close(wp, false, false) == OK;
        }
        if !did_one {
            // The buffer keeps a window; it can only stop being a dummy.
            buffer.b_flags.clear(BufFlags::DUMMY);
            return;
        }
    }

    if Buf::current_or_none() != Some(buffer) && buffer.b_nwindows == 0 {
        // Delete the buffer and its swap file. `wipe_buffer` calls
        // `close_buffer`, which may run autocommands, so a pending
        // exception or `:return` has to be parked over the call.
        let cleanup = CleanupGuard::enter();
        wipe_buffer(buffer, true);
        drop(cleanup);

        // When autocommands/'autochdir' option changed directory: go back.
        if let Some(dirname_start) = dirname_start {
            restore_start_dir(dirname_start);
        }
        return;
    }

    buffer.b_flags.clear(BufFlags::DUMMY);
}

/// Unload the dummy buffer that `load_dummy_buffer` created, keeping it in
/// the buffer list so that a later `:vimgrep` finds it again.
pub(crate) fn unload_dummy_buffer(buffer: Buf, dirname_start: &CStr) {
    if Buf::current_or_none() == Some(buffer) {
        return;
    }
    close_buffer(None, buffer, DOBUF_UNLOAD.cast_signed(), false, true);

    // When autocommands/'autochdir' option changed directory: go back.
    restore_start_dir(dirname_start);
}
