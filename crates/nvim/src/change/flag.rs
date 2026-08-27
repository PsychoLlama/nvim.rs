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

use core::ffi::{c_char, c_int, c_void};

use super::*;
use crate::buffer::BufFlags;
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
/// and even change `curbuf`; `buf` is read again after they have run, so it
/// must survive them.
pub unsafe fn change_warning(mut buf: Buf, col: c_int) {
    if buf.b_did_warn || curbuf_is_changed() || autocmd_busy.get() || buf.b_p_ro == 0 {
        return;
    }
    buf.b_ro_locked += 1;
    // SAFETY: a live buffer, and the event takes no file name.
    unsafe {
        apply_autocmds(
            EVENT_FILECHANGEDRO,
            ::core::ptr::null_mut(),
            ::core::ptr::null_mut(),
            false,
            buf.raw(),
        )
    };
    buf.b_ro_locked -= 1;
    if buf.b_p_ro == 0 {
        // An autocommand cleared 'readonly': nothing to warn about.
        return;
    }

    // What msg() does, but with a column offset.
    // SAFETY: every string here is this file's own static message or the
    // catalogue's translation of it.
    unsafe {
        msg_start();
        if msg_row.get() == Rows.get() - 1 {
            msg_col.set(col);
        }
        msg_source(HLF_W);
        msg_ext_set_kind(c"wmsg".as_ptr());
        msg_puts_hl(gettext(W_READONLY), HLF_W, true);
        set_vim_var_string(Vv::Warningmsg, gettext(W_READONLY), -1);
        msg_clr_eos();
        msg_end();
        if msg_silent.get() == 0 && !silent_mode.get() && ui_active() != 0 {
            // Give the user time to think about it.
            msg_delay(1002, true);
        }
    }
    buf.b_did_warn = true;
    // Don't redraw and erase the message.
    redraw_cmdline.set(false);
    if msg_row.get() < Rows.get() - 1 {
        // SAFETY: redrawing the mode message on the last line.
        unsafe { showmode() };
    }
}

/// Note that something in `buf` changed.
///
/// Most often reached through [`changed_bytes`] and [`changed_lines`], which
/// also mark the area of the display to be redrawn. `b:changedtick` is bumped
/// on *every* call, whether or not the buffer was already modified.
///
/// # Safety
/// May trigger autocommands that reload the buffer, and notifies the
/// `b:changedtick` watchers, which can re-enter; `buf` is used after both, so
/// it must survive them.
pub unsafe fn changed(buf: Buf) {
    if buf.b_changed == 0 {
        let save_msg_scroll = msg_scroll.get();

        // May check the file out, and so change `curbuf`.
        // SAFETY: the caller's promise -- `buf` survives FileChangedRO.
        unsafe { change_warning(buf, 0) };

        // Create a swap file if that is wanted; not for "nofile" and
        // "nowrite" buffers.
        // SAFETY: a live buffer.
        if buf.b_may_swap && !unsafe { bt_dontwrite(buf.raw()) } {
            let save_need_wait_return = need_wait_return.get();
            need_wait_return.set(false);
            // SAFETY: a live buffer.
            unsafe { ml_open_file(buf.raw()) };

            // ml_open_file() can produce an ATTENTION message. Wait two
            // seconds so the user reads it, and call wait_return() here
            // rather than letting a later emsg() set msg_scroll.
            if need_wait_return.get()
                && emsg_silent.get() == 0
                && !in_assert_fails.get()
                && !ui_has(kUIMessages)
            {
                // SAFETY: waiting on the message just shown.
                unsafe {
                    msg_delay(2002, true);
                    wait_return(true as c_int);
                }
                msg_scroll.set(save_msg_scroll);
            } else {
                need_wait_return.set(save_need_wait_return);
            }
        }
        changed_internal(buf);
    }
    // SAFETY: a live buffer.
    unsafe { buf_inc_changedtick(buf.raw()) };
    highlight_match.set(false);
}

/// Set `b_changed` and everything that displays it, without the warning, the
/// swap file or the `b:changedtick` bump [`changed`] also does.
///
/// Safe: [`Buf`] carries the only promise this needs, that the buffer is live.
pub fn changed_internal(mut buf: Buf) {
    buf.b_changed = true as c_int;
    buf.b_changed_invalid = true;
    // SAFETY: a live buffer, which is all either asks.
    unsafe {
        ml_setflags(buf.raw());
        redraw_buf_status_later(buf.raw());
    }
    redraw_tabline.set(true);
    need_maketitle.set(true);
}

/// Note that `buf` is no longer modified -- `:w`, `:e!`, and undoing back to
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
pub fn unchanged(mut buf: Buf, ff: bool, always_inc_changedtick: bool) {
    if buf.b_changed != 0 || (ff && file_ff_differs(buf, false)) {
        buf.b_changed = false as c_int;
        buf.b_changed_invalid = true;
        // SAFETY: a live buffer, which is all it asks.
        unsafe { ml_setflags(buf.raw()) };
        if ff {
            save_file_ff(buf);
        }
        // SAFETY: a live buffer, which is all it asks.
        unsafe { redraw_buf_status_later(buf.raw()) };
        redraw_tabline.set(true);
        need_maketitle.set(true);
        // SAFETY: a live buffer, which is all it asks.
        unsafe { buf_inc_changedtick(buf.raw()) };
    } else if always_inc_changedtick {
        // SAFETY: a live buffer, which is all it asks.
        unsafe { buf_inc_changedtick(buf.raw()) };
    }
}

/// Remember `buf`'s 'fileformat', 'fileencoding', end-of-line, end-of-file
/// and BOM as they are on disk, so that [`file_ff_differs`] can tell later
/// whether the user changed one.
///
/// Safe: [`Buf`] carries the only promise this needs, that the buffer is live.
pub fn save_file_ff(mut buf: Buf) {
    // SAFETY: 'fileformat' is the buffer's own one-character option string.
    buf.b_start_ffc = c_int::from(unsafe { *buf.b_p_ff } as u8);
    buf.b_start_eof = buf.b_p_eof;
    buf.b_start_eol = buf.b_p_eol;
    buf.b_start_bomb = buf.b_p_bomb;

    // Only free and allocate when the value actually changed.
    let (recorded, current) = (buf.b_start_fenc, buf.b_p_fenc);
    // SAFETY: both are NUL-terminated option strings, and `b_start_fenc` is
    // this buffer's own allocation to replace.
    if recorded.is_null() || unsafe { strcmp(recorded, current) } != 0 {
        unsafe { xfree(recorded as *mut c_void) };
        buf.b_start_fenc = unsafe { xstrdup(current) };
    }
}

/// Whether any of the options [`save_file_ff`] recorded has since changed.
///
/// `ignore_empty` is for `:w`: an unmodified, still-empty new buffer is not
/// worth reporting, because the values it carries were never read off a file.
///
/// Safe: [`Buf`] carries the only promise this needs, that the buffer is live.
pub fn file_ff_differs(buf: Buf, ignore_empty: bool) -> bool {
    // Handle a file that was never loaded as "not changed": the recorded
    // values are the defaults, not the file's.
    if buf.b_flags.has(BufFlags::NEVERLOADED) {
        return false;
    }
    if ignore_empty
        && buf.b_flags.has(BufFlags::NEW)
        && buf.b_ml.ml_line_count == 1
        // SAFETY: the line the count just promised, NUL-terminated.
        && c_int::from(unsafe { *ml_get_buf(buf.raw(), 1) }) == NUL
    {
        return false;
    }
    // SAFETY: 'fileformat' is the buffer's own one-character option string.
    if buf.b_start_ffc != c_int::from(unsafe { *buf.b_p_ff }) {
        return true;
    }
    // 'endofline' and 'endoffile' only matter with 'binary' set or
    // 'fixendofline' off: otherwise the writer normalises them anyway.
    if (buf.b_p_bin != 0 || buf.b_p_fixeol == 0)
        && (buf.b_start_eof != buf.b_p_eof || buf.b_start_eol != buf.b_p_eol)
    {
        return true;
    }
    if buf.b_p_bin == 0 && buf.b_start_bomb != buf.b_p_bomb {
        return true;
    }
    let (recorded, current) = (buf.b_start_fenc, buf.b_p_fenc);
    if recorded.is_null() {
        // SAFETY: the buffer's own NUL-terminated option string.
        return c_int::from(unsafe { *current }) != NUL;
    }
    // SAFETY: both are the buffer's own NUL-terminated option strings.
    unsafe { strcmp(recorded, current) != 0 }
}
