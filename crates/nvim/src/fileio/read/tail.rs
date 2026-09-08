//! The tail of a read: what happens once the bytes are in the buffer.
//!
//! `readfile`'s last third is not about reading at all. It reports what was
//! read, decides whether the buffer has to be written with `:w!`, puts the
//! cursor and the `'[`/`']` marks on the new lines, and fires the
//! `*ReadPost` autocommands -- which is where the read can still be called
//! off, because an autocommand may abort script processing.
//!
//! Splitting it off keeps [`readfile`](super::readfile) itself to the loop
//! that actually reads.

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

/// Report what was read, and leave the cursor and the `'[`/`']` marks on the
/// new lines.
///
/// `bad_char` is the conversion's `'fileencoding'` replacement setting; it
/// only matters for deciding whether an illegal byte makes the buffer
/// read-only.
///
/// # Safety
/// `sfname` must be the name the read used, or null.
pub(crate) unsafe fn report_and_place(
    sfname: *mut c_char,
    how: How,
    silent: bool,
    out: &Outcome,
    bad_char: c_int,
    from: LineNr,
) {
    if !how.filtering && !how.dummy && !silent {
        // SAFETY: the caller's file name.
        unsafe { report_read(sfname, how, out) };
    }

    // With errors, writing the file requires ":w!".
    let bad_bytes = out.illegal_byte > 0 && bad_char != BAD_KEEP;
    if how.newfile && (out.error || out.conv_error != 0 || bad_bytes) {
        Buf::current().b_p_ro = c_int::from(true);
    }

    // SAFETY: the current buffer and window are live.
    u_clearline(Buf::current()); // "U" cannot be used after adding lines

    // In Ex mode the cursor goes on the last new line, otherwise on the
    // first one.
    Win::current().w_cursor.lnum = if exmode_active.get() {
        from + out.linecnt
    } else {
        from + 1
    };
    check_cursor_lnum(Win::current());
    beginline(BeginlineOpts::WHITE | BeginlineOpts::FIX); // first non-blank

    if !cmdmod_has(CmdModFlags::LOCKMARKS) {
        // Set the '[ and '] marks to the newly read lines.
        Buf::current().b_op_start.lnum = from + 1;
        Buf::current().b_op_start.col = 0;
        Buf::current().b_op_end.lnum = from + out.linecnt;
        Buf::current().b_op_end.col = 0;
    }
}

/// The `*ReadPost` autocommands, and the `FileType` one they may still owe.
///
/// Answers false when an autocommand aborted script processing, which makes
/// [`readfile`](super::readfile) answer `FAIL` at once -- skipping the
/// swap-file sync, as upstream does.
///
/// # Safety
/// `sfname` must be the name the read used, or null, and `args` the caller's
/// command or null.
pub(crate) unsafe fn run_read_autocmds(
    sfname: *mut c_char,
    args: *mut ExArg,
    how: How,
    set_options: bool,
) -> bool {
    let m = msg_scroll.get();
    let n = msg_scrolled.get();

    // Save the fileformat now, or the buffer would be considered modified
    // because the format or encoding was auto-detected.
    if set_options {
        save_file_ff(Buf::current());
    }

    // The output from the autocommands should neither overwrite anything nor
    // be overwritten: set msg_scroll, and restore it if no output was done.
    msg_scroll.set(c_int::from(true));
    // A `BufReadPost` also owes a `FileType` when the buffer already has one
    // and nothing triggered it.
    let buf_read = !how.filtering && (how.newfile || (how.buffer && !sfname.is_null()));
    let (ev, iofile, buf) = if how.filtering {
        (
            AutoEvent::FilterReadPost,
            ptr::null_mut(),
            Buf::current_or_none(),
        )
    } else if buf_read {
        (
            AutoEvent::BufReadPost,
            ptr::null_mut(),
            Buf::current_or_none(),
        )
    } else {
        (AutoEvent::FileReadPost, sfname, None)
    };
    // SAFETY: the current buffer is live and `args` is the caller's command.
    unsafe { apply_autocmds_exarg(ev, iofile, sfname, false, buf, args) };
    // SAFETY: `b_p_ft` is the buffer's own `'filetype'` string.
    if buf_read && !Buf::current().b_au_did_filetype && unsafe { *Buf::current().b_p_ft } != 0 {
        let (ft, fname) = (Buf::current().b_p_ft, Buf::current().b_fname);
        // SAFETY: the buffer's own option and file name; `curbuf` is re-read
        // because `BufReadPost` may have moved us.
        unsafe { apply_autocmds(AutoEvent::FileType, ft, fname, true, Buf::current_or_none()) };
    }
    if msg_scrolled.get() == n {
        msg_scroll.set(m);
    }
    !aborting()
}
