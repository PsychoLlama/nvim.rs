//! What the read ended up doing, in one message.
//!
//! Every note the message can carry — `[readonly]`, `[noeol]`,
//! `[converted]`, the line the first bad byte was on — is one field of
//! [`Outcome`], and `report_read` turns them into the line Nvim prints
//! after a file is read.

#![deny(unsafe_op_in_unsafe_fn)]

use crate::winlayer::Buf;
use core::ffi::{c_char, c_int};

use crate::bufwrite::translate;

use super::*;
use crate::types::{IOSIZE, ShmFlag};

/// Everything the closing message needs to know about how the read went.
pub(crate) struct Outcome {
    pub perm: c_int,
    pub read_no_eol_lnum: linenr_T,
    pub ff_error: c_int,
    pub split: c_int,
    pub notconverted: bool,
    pub converted: bool,
    pub conv_error: linenr_T,
    pub illegal_byte: linenr_T,
    pub error: bool,
    pub fileformat: c_int,
    pub linecnt: linenr_T,
    pub filesize: off_T,
}

/// `snprintf` a plain note into the report at `buflen`, answering how many
/// bytes it wrote.
fn note_text(io: *mut c_char, buflen: c_int, text: *const c_char) -> c_int {
    // SAFETY: `io` is the caller's `IOSIZE`-byte report with `buflen` bytes
    // already in it, and `text` is a NUL-terminated message.
    let at = unsafe { io.offset(buflen as isize) };
    unsafe { snprintf(at, (IOSIZE - buflen) as size_t, text) }
}

/// [`note_text`] for a note carrying one `%ld`.
fn note_num(io: *mut c_char, buflen: c_int, fmt: *const c_char, n: int64_t) {
    // SAFETY: as [`note_text`]; the one conversion matches the one argument.
    let at = unsafe { io.offset(buflen as isize) };
    unsafe { snprintf(at, (IOSIZE - buflen) as size_t, fmt, n) };
}

/// Report what was read.
pub(crate) unsafe fn report_read(sfname: *mut c_char, how: How, out: &Outcome) {
    // The report. Upstream assembles it in `IObuff`, which `msg_trunc` and
    // `set_keep_msg` reach the message machinery through.
    let mut report = [0 as c_char; IOSIZE as usize];
    let io = report.as_mut_ptr();
    unsafe { add_quoted_fname(io, IOSIZE as size_t, Buf::current(), sfname) };
    let mut noted = false;
    let mut buflen = unsafe { strlen(io) } as c_int;
    let mut note = |text: *const c_char| {
        buflen += note_text(io, buflen, text);
        noted = true;
    };

    if out.perm & __S_IFMT == 0o10000 {
        note(translate(c"[fifo]").as_ptr());
    }
    if out.perm & __S_IFMT == 0o140000 {
        note(translate(c"[socket]").as_ptr());
    }
    if cur_buf().b_p_ro != 0 {
        note(
            if shortmess(ShmFlag::RO) {
                translate(c"[RO]")
            } else {
                translate(c"[readonly]")
            }
            .as_ptr(),
        );
    }
    if out.read_no_eol_lnum != 0 {
        note(translate(c"[noeol]").as_ptr());
    }
    if out.ff_error == EOL_DOS {
        note(translate(c"[CR missing]").as_ptr());
    }
    if out.split != 0 {
        note(translate(c"[long lines split]").as_ptr());
    }
    if out.notconverted {
        note(translate(c"[NOT converted]").as_ptr());
    } else if out.converted {
        note(translate(c"[converted]").as_ptr());
    }
    // These three are last, so their `buflen` is never used again.
    if out.conv_error != 0 {
        let fmt = translate(c"[CONVERSION ERROR in line %ld]").as_ptr();
        note_num(io, buflen, fmt, out.conv_error as int64_t);
        noted = true;
    } else if out.illegal_byte > 0 {
        let fmt = translate(c"[ILLEGAL BYTE in line %ld]").as_ptr();
        note_num(io, buflen, fmt, out.illegal_byte as int64_t);
        noted = true;
    } else if out.error {
        note_text(io, buflen, translate(c"[READ ERRORS]").as_ptr());
        noted = true;
    }
    if unsafe { msg_add_fileformat(&mut report, out.fileformat) } {
        noted = true;
    }
    unsafe { msg_add_lines(&mut report, noted as c_int, out.linecnt, out.filesize) };

    unsafe { xfree(keep_msg.get().cast()) };
    keep_msg.set(ptr::null_mut());
    let mut shown: *mut c_char = ptr::null_mut();
    msg_scrolled_ign.set(true);

    if !how.stdin && !how.buffer {
        if msg_col.get() > 0 {
            unsafe { msg_putchar(b'\r' as c_int) }; // overwrite previous message
        }
        shown = unsafe { msg_trunc(io, false, 0) };
    }

    // The message has to be repeated after redrawing when reading from
    // stdin (the screen is cleared next), when `restart_edit` is set
    // (otherwise there is a delay before redrawing), and when the screen
    // scrolled but there is no wait-return prompt.
    if how.stdin
        || how.buffer
        || restart_edit.get() != 0
        || (msg_scrolled.get() != 0 && !need_wait_return.get())
    {
        unsafe { set_keep_msg(shown, 0) };
    }
    msg_scrolled_ign.set(false);
}

/// The buffer the editor is working in.
fn cur_buf() -> Buf {
    // SAFETY: `curbuf` is set from startup to exit.
    unsafe { Buf::current() }
}
