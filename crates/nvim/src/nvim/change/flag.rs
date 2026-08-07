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

use ::core::ffi::{c_char, c_int, c_void};

use super::*;

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
/// `buf` must be a live buffer. FileChangedRO may run arbitrary autocommands,
/// which can reload the buffer and even change `curbuf`.
pub unsafe fn change_warning(buf: *mut buf_T, col: c_int) {
    unsafe {
        if (*buf).b_did_warn || curbufIsChanged() || autocmd_busy.get() || (*buf).b_p_ro == 0 {
            return;
        }
        (*buf).b_ro_locked += 1;
        apply_autocmds(
            EVENT_FILECHANGEDRO,
            ::core::ptr::null_mut(),
            ::core::ptr::null_mut(),
            false,
            buf,
        );
        (*buf).b_ro_locked -= 1;
        if (*buf).b_p_ro == 0 {
            // An autocommand cleared 'readonly': nothing to warn about.
            return;
        }

        // What msg() does, but with a column offset.
        msg_start();
        if msg_row.get() == Rows.get() - 1 {
            msg_col.set(col);
        }
        msg_source(HLF_W);
        msg_ext_set_kind(c"wmsg".as_ptr());
        msg_puts_hl(gettext(W_READONLY), HLF_W, true);
        set_vim_var_string(VV_WARNINGMSG, gettext(W_READONLY), -1);
        msg_clr_eos();
        msg_end();
        if msg_silent.get() == 0 && !silent_mode.get() && ui_active() != 0 {
            // Give the user time to think about it.
            msg_delay(1002, true);
        }
        (*buf).b_did_warn = true;
        // Don't redraw and erase the message.
        redraw_cmdline.set(false);
        if msg_row.get() < Rows.get() - 1 {
            showmode();
        }
    }
}

/// Note that something in `buf` changed.
///
/// Most often reached through [`changed_bytes`] and [`changed_lines`], which
/// also mark the area of the display to be redrawn. `b:changedtick` is bumped
/// on *every* call, whether or not the buffer was already modified.
///
/// # Safety
/// `buf` must be a live buffer. May trigger autocommands that reload it.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn changed(buf: *mut buf_T) {
    unsafe {
        if (*buf).b_changed == 0 {
            let save_msg_scroll = msg_scroll.get();

            // May check the file out, and so change `curbuf`.
            change_warning(buf, 0);

            // Create a swap file if that is wanted; not for "nofile" and
            // "nowrite" buffers.
            if (*buf).b_may_swap && !bt_dontwrite(buf) {
                let save_need_wait_return = need_wait_return.get();
                need_wait_return.set(false);
                ml_open_file(buf);

                // ml_open_file() can produce an ATTENTION message. Wait two
                // seconds so the user reads it, and call wait_return() here
                // rather than letting a later emsg() set msg_scroll.
                if need_wait_return.get()
                    && emsg_silent.get() == 0
                    && !in_assert_fails.get()
                    && !ui_has(kUIMessages)
                {
                    msg_delay(2002, true);
                    wait_return(true as c_int);
                    msg_scroll.set(save_msg_scroll);
                } else {
                    need_wait_return.set(save_need_wait_return);
                }
            }
            changed_internal(buf);
        }
        buf_inc_changedtick(buf);
        highlight_match.set(false);
    }
}

/// Set `b_changed` and everything that displays it, without the warning, the
/// swap file or the `b:changedtick` bump [`changed`] also does.
///
/// # Safety
/// `buf` must be a live buffer.
pub unsafe fn changed_internal(buf: *mut buf_T) {
    unsafe {
        (*buf).b_changed = true as c_int;
        (*buf).b_changed_invalid = true;
        ml_setflags(buf);
        redraw_buf_status_later(buf);
        redraw_tabline.set(true);
        need_maketitle.set(true);
    }
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
/// # Safety
/// `buf` must be a live buffer.
pub unsafe fn unchanged(buf: *mut buf_T, ff: bool, always_inc_changedtick: bool) {
    unsafe {
        if (*buf).b_changed != 0 || (ff && file_ff_differs(buf, false)) {
            (*buf).b_changed = false as c_int;
            (*buf).b_changed_invalid = true;
            ml_setflags(buf);
            if ff {
                save_file_ff(buf);
            }
            redraw_buf_status_later(buf);
            redraw_tabline.set(true);
            need_maketitle.set(true);
            buf_inc_changedtick(buf);
        } else if always_inc_changedtick {
            buf_inc_changedtick(buf);
        }
    }
}

/// Remember `buf`'s 'fileformat', 'fileencoding', end-of-line, end-of-file
/// and BOM as they are on disk, so that [`file_ff_differs`] can tell later
/// whether the user changed one.
///
/// # Safety
/// `buf` must be a live buffer.
pub unsafe fn save_file_ff(buf: *mut buf_T) {
    unsafe {
        (*buf).b_start_ffc = c_int::from(*(*buf).b_p_ff as u8);
        (*buf).b_start_eof = (*buf).b_p_eof;
        (*buf).b_start_eol = (*buf).b_p_eol;
        (*buf).b_start_bomb = (*buf).b_p_bomb;

        // Only free and allocate when the value actually changed.
        if (*buf).b_start_fenc.is_null() || strcmp((*buf).b_start_fenc, (*buf).b_p_fenc) != 0 {
            xfree((*buf).b_start_fenc as *mut c_void);
            (*buf).b_start_fenc = xstrdup((*buf).b_p_fenc);
        }
    }
}

/// Whether any of the options [`save_file_ff`] recorded has since changed.
///
/// `ignore_empty` is for `:w`: an unmodified, still-empty new buffer is not
/// worth reporting, because the values it carries were never read off a file.
///
/// # Safety
/// `buf` must be a live buffer.
pub unsafe fn file_ff_differs(buf: *mut buf_T, ignore_empty: bool) -> bool {
    unsafe {
        // Handle a file that was never loaded as "not changed": the recorded
        // values are the defaults, not the file's.
        if (*buf).b_flags & BF_NEVERLOADED != 0 {
            return false;
        }
        if ignore_empty
            && (*buf).b_flags & BF_NEW != 0
            && (*buf).b_ml.ml_line_count == 1
            && c_int::from(*ml_get_buf(buf, 1)) == NUL
        {
            return false;
        }
        if (*buf).b_start_ffc != c_int::from(*(*buf).b_p_ff) {
            return true;
        }
        // 'endofline' and 'endoffile' only matter with 'binary' set or
        // 'fixendofline' off: otherwise the writer normalises them anyway.
        if ((*buf).b_p_bin != 0 || (*buf).b_p_fixeol == 0)
            && ((*buf).b_start_eof != (*buf).b_p_eof || (*buf).b_start_eol != (*buf).b_p_eol)
        {
            return true;
        }
        if (*buf).b_p_bin == 0 && (*buf).b_start_bomb != (*buf).b_p_bomb {
            return true;
        }
        if (*buf).b_start_fenc.is_null() {
            return c_int::from(*(*buf).b_p_fenc) != NUL;
        }
        strcmp((*buf).b_start_fenc, (*buf).b_p_fenc) != 0
    }
}
