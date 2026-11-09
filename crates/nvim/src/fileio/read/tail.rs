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

#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use super::*;
use crate::optionstr::OptString;
use crate::types::ColNr;

/// Report what was read, and leave the cursor and the `'[`/`']` marks on the
/// new lines.
///
/// `bad_char` is the conversion's `'fileencoding'` replacement setting; it
/// only matters for deciding whether an illegal byte makes the buffer
/// read-only.
pub(crate) fn report_and_place(
    sfname: Option<&CStr>,
    how: &How,
    silent: bool,
    out: &Outcome,
    bad_char: c_int,
    from: LineNr,
) {
    if !how.filtering && !how.dummy && !silent {
        report_read(sfname, how, out);
    }

    // With errors, writing the file requires ":w!".
    let bad_bytes = out.illegal_byte > LineNr::ZERO && bad_char != BAD_KEEP;
    if how.newfile && (out.error || out.conv_error != LineNr::ZERO || bad_bytes) {
        Buf::current().b_p_ro = c_int::from(true);
    }

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
        Buf::current().b_op_start.col = ColNr::ZERO;
        Buf::current().b_op_end.lnum = from + out.linecnt;
        Buf::current().b_op_end.col = ColNr::ZERO;
    }
}

/// The `*ReadPost` autocommands, and the `FileType` one they may still owe.
///
/// Answers false when an autocommand aborted script processing, which makes
/// [`readfile`](super::readfile) answer `FAIL` at once -- skipping the
/// swap-file sync, as upstream does.
pub(crate) fn run_read_autocmds(
    sfname: Option<&CStr>,
    excmd: Option<&mut ExArg>,
    how: &How,
    set_options: bool,
) -> bool {
    let m = MSG_SCROLL.get();
    let n = MSG_SCROLLED.get();

    // Save the fileformat now, or the buffer would be considered modified
    // because the format or encoding was auto-detected.
    if set_options {
        save_file_ff(Buf::current());
    }

    // The output from the autocommands should neither overwrite anything nor
    // be overwritten: set msg_scroll, and restore it if no output was done.
    MSG_SCROLL.set(c_int::from(true));
    // A `BufReadPost` also owes a `FileType` when the buffer already has one
    // and nothing triggered it.
    let buf_read = !how.filtering && (how.newfile || (how.buffer && sfname.is_some()));
    let (ev, iofile, buf) = if how.filtering {
        (AutoEvent::FilterReadPost, None, Buf::current_or_none())
    } else if buf_read {
        (AutoEvent::BufReadPost, None, Buf::current_or_none())
    } else {
        (AutoEvent::FileReadPost, sfname, None)
    };
    let (iofile, afile) = (
        iofile.map(XString::from_cstr),
        sfname.map(XString::from_cstr),
    );
    apply_autocmds_exarg(ev, iofile, afile, false, buf, excmd);
    if buf_read && !Buf::current().b_au_did_filetype && Buf::current().b_p_ft.first_byte() != 0 {
        // `curbuf` is re-read because `BufReadPost` may have moved us.
        let ft = Buf::current().b_p_ft.get();
        let fname = Buf::current().name.shown().map(XString::from_cstr);
        fire_autocmds_for(
            AutoEvent::FileType,
            Some(ft),
            fname,
            true,
            Buf::current_or_none(),
            None,
        );
    }
    if MSG_SCROLLED.get() == n {
        MSG_SCROLL.set(m);
    }
    !aborting()
}
