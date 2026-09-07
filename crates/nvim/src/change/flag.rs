//! Whether the buffer counts as modified.
//!
//! [`changed`] is the front door every edit goes through: it flips
//! `b_changed`, warns once about a 'readonly' file (after giving
//! FileChangedRO a chance to clear it), makes sure a swap file exists, and
//! bumps `b:changedtick`. [`unchanged`] is the other direction -- `:w` and
//! `:e!` -- and [`save_file_ff`] / [`file_ff_differs`] are the pair that
//! remembers a buffer's 'fileformat', 'fileencoding', end-of-line and BOM at
//! load time so that `:w` can tell a real change from one the reader made.

#![deny(unsafe_op_in_unsafe_fn)]
#![allow(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use crate::cstr;
use core::ffi::{c_char, c_int, c_void};

use super::*;
use crate::buffer::BufFlags;
use crate::os::cshim::gettext_ptr;
use crate::types::NUL;
use crate::winlayer::Buf;

/// The message [`change_warning`] gives, once per buffer.
const W_READONLY: *const c_char = c"W10: Warning: Changing a readonly file".as_ptr();

/// Warn about the first change to a 'readonly' file.
///
/// Not `emsg()`, which would flush the macro buffer, and not at all while
/// autocommands are running. `b_did_warn` is what makes it once-per-buffer:
/// undoing every change clears `b_changed` again but not that flag. `col` is
/// where to put the message, non-zero in Insert mode with 'showmode' on so
/// that it lands after the mode message.
///
/// # Safety
/// FileChangedRO may run arbitrary autocommands, which can reload the buffer
/// and even change `curbuf`; `buffer` is read again after they have run, so it
/// must survive them.
pub unsafe fn change_warning(mut buffer: Buf, col: c_int) {
    if buffer.b_did_warn || curbuf_is_changed() || autocmd_busy.get() || buffer.b_p_ro == 0 {
        return;
    }
    buffer.b_ro_locked += 1;
    // SAFETY: a live buffer, and the event takes no file name.
    unsafe {
        apply_autocmds(
            AutoEvent::FileChangedRO,
            ::core::ptr::null_mut(),
            ::core::ptr::null_mut(),
            false,
            Some(buffer),
        )
    };
    buffer.b_ro_locked -= 1;
    if buffer.b_p_ro == 0 {
        // An autocommand cleared 'readonly': nothing to warn about.
        return;
    }

    // What msg() does, but with a column offset.
    msg_start();
    if msg_row.get() == Rows.get() - 1 {
        msg_col.set(col);
    }
    unsafe { msg_source(HLF_W) };
    unsafe { msg_ext_set_kind(c"wmsg".as_ptr()) };
    unsafe { msg_puts_hl(gettext_ptr(W_READONLY).as_ptr(), HLF_W, true) };
    unsafe { set_vim_var_string(Vv::Warningmsg, gettext_ptr(W_READONLY).as_ptr(), -1) };
    unsafe { msg_clr_eos() };
    msg_end();
    if msg_silent.get() == 0 && !silent_mode.get() && ui_active() != 0 {
        // Give the user time to think about it.
        unsafe { msg_delay(1002, true) };
    }
    buffer.b_did_warn = true;
    // Don't redraw and erase the message.
    redraw_cmdline.set(false);
    if msg_row.get() < Rows.get() - 1 {
        // SAFETY: redrawing the mode message on the last line.
        unsafe { showmode() };
    }
}

/// Note that something in `buffer` changed.
///
/// Most often reached through [`changed_bytes`] and [`changed_lines`], which
/// also mark the area of the display to be redrawn. `b:changedtick` is bumped
/// on *every* call, whether or not the buffer was already modified.
///
/// # Safety
/// May trigger autocommands that reload the buffer, and notifies the
/// `b:changedtick` watchers, which can re-enter; `buffer` is used after both, so
/// it must survive them.
pub unsafe fn changed(buffer: Buf) {
    if buffer.b_changed == 0 {
        let save_msg_scroll = msg_scroll.get();

        // May check the file out, and so change `curbuf`.
        // SAFETY: the caller's promise -- `buffer` survives FileChangedRO.
        unsafe { change_warning(buffer, 0) };

        // Create a swap file if that is wanted; not for "nofile" and
        // "nowrite" buffers.
        if buffer.b_may_swap && !buf_is_dontwrite(Some(buffer)) {
            let save_need_wait_return = need_wait_return.get();
            need_wait_return.set(false);
            // SAFETY: a live buffer.
            unsafe { ml_open_file(buffer) };

            // ml_open_file() can produce an ATTENTION message. Wait two
            // seconds so the user reads it, and call wait_return() here
            // rather than letting a later emsg() set msg_scroll.
            if need_wait_return.get()
                && emsg_silent.get() == 0
                && !in_assert_fails.get()
                && !ui_has(kUIMessages)
            {
                // SAFETY: waiting on the message just shown.
                unsafe { msg_delay(2002, true) };
                unsafe { wait_return(c_int::from(true)) };
                msg_scroll.set(save_msg_scroll);
            } else {
                need_wait_return.set(save_need_wait_return);
            }
        }
        changed_internal(buffer);
    }
    buf_inc_changedtick(buffer);
    highlight_match.set(false);
}

/// Set `b_changed` and everything that displays it, without the warning, the
/// swap file or the `b:changedtick` bump [`changed`] also does.
///
/// Safe: [`Buf`] carries the only promise this needs, that the buffer is live.
pub fn changed_internal(mut buffer: Buf) {
    buffer.b_changed = c_int::from(true);
    buffer.b_changed_invalid = true;
    ml_setflags(buffer);
    redraw_buf_status_later(buffer);
    redraw_tabline.set(true);
    need_maketitle.set(true);
}

/// Note that `buffer` is no longer modified -- `:w`, `:e!`, and undoing back to
/// the last write.
///
/// With `ff` set, the buffer's 'fileformat' and friends are re-recorded as
/// the on-disk state, and a buffer whose only "change" was one of those still
/// counts as having been changed. `always_inc_changedtick` bumps
/// `b:changedtick` even when nothing moved, which is what `:w` wants: the
/// file on disk is new even if the text is not.
///
/// Safe: [`Buf`] carries the only promise this needs, that the buffer is live.
/// The `b:changedtick` bump notifies the `b:` watchers, which may re-enter,
/// but nothing here reads the buffer after it.
pub fn unchanged(mut buffer: Buf, ff: bool, always_inc_changedtick: bool) {
    if buffer.b_changed != 0 || (ff && file_ff_differs(buffer, false)) {
        buffer.b_changed = c_int::from(false);
        buffer.b_changed_invalid = true;
        ml_setflags(buffer);
        if ff {
            save_file_ff(buffer);
        }
        redraw_buf_status_later(buffer);
        redraw_tabline.set(true);
        need_maketitle.set(true);
        buf_inc_changedtick(buffer);
    } else if always_inc_changedtick {
        buf_inc_changedtick(buffer);
    }
}

/// Remember `buffer`'s 'fileformat', 'fileencoding', end-of-line, end-of-file
/// and BOM as they are on disk, so that [`file_ff_differs`] can tell later
/// whether the user changed one.
///
/// Safe: [`Buf`] carries the only promise this needs, that the buffer is live.
pub fn save_file_ff(mut buffer: Buf) {
    // SAFETY: 'fileformat' is the buffer's own one-character option string.
    buffer.b_start_ffc = c_int::from(unsafe { *buffer.b_p_ff }.cast_unsigned());
    buffer.b_start_eof = buffer.b_p_eof;
    buffer.b_start_eol = buffer.b_p_eol;
    buffer.b_start_bomb = buffer.b_p_bomb;

    // Only free and allocate when the value actually changed.
    let (recorded, current) = (buffer.b_start_fenc, buffer.b_p_fenc);
    // SAFETY: both are NUL-terminated option strings, and `b_start_fenc` is
    // this buffer's own allocation to replace.
    if recorded.is_null() || !unsafe { cstr::eq(recorded, current) } {
        unsafe { xfree(recorded.cast::<c_void>()) };
        buffer.b_start_fenc = unsafe { xstrdup(current) };
    }
}

/// Whether any of the options [`save_file_ff`] recorded has since changed.
///
/// `ignore_empty` is for `:w`: an unmodified, still-empty new buffer is not
/// worth reporting, because the values it carries were never read off a file.
///
/// Safe: [`Buf`] carries the only promise this needs, that the buffer is live.
pub fn file_ff_differs(buffer: Buf, ignore_empty: bool) -> bool {
    // Handle a file that was never loaded as "not changed": the recorded
    // values are the defaults, not the file's.
    if buffer.b_flags.has(BufFlags::NEVERLOADED) {
        return false;
    }
    if ignore_empty
        && buffer.b_flags.has(BufFlags::NEW)
        && buffer.b_ml.ml_line_count == 1
        // SAFETY: the line the count just promised, NUL-terminated.
        && c_int::from(unsafe { *ml_get_buf(buffer, 1) }) == NUL
    {
        return false;
    }
    // SAFETY: 'fileformat' is the buffer's own one-character option string.
    if buffer.b_start_ffc != c_int::from(unsafe { *buffer.b_p_ff }) {
        return true;
    }
    // 'endofline' and 'endoffile' only matter with 'binary' set or
    // 'fixendofline' off: otherwise the writer normalises them anyway.
    if (buffer.b_p_bin != 0 || buffer.b_p_fixeol == 0)
        && (buffer.b_start_eof != buffer.b_p_eof || buffer.b_start_eol != buffer.b_p_eol)
    {
        return true;
    }
    if buffer.b_p_bin == 0 && buffer.b_start_bomb != buffer.b_p_bomb {
        return true;
    }
    let (recorded, current) = (buffer.b_start_fenc, buffer.b_p_fenc);
    if recorded.is_null() {
        // SAFETY: the buffer's own NUL-terminated option string.
        return c_int::from(unsafe { *current }) != NUL;
    }
    // SAFETY: both are the buffer's own NUL-terminated option strings.
    unsafe { !cstr::eq(recorded, current) }
}
