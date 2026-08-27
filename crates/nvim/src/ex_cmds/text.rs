//! Rearranging the text of a line in place -- `:left`, `:right`, `:center` and
//! the read-only `:ascii`.
//!
//! [`ex_align`] is the whole of the three alignment commands: it measures the
//! line with [`linelen`] (which is also `:sort`'s width oracle), works out the
//! new indent against 'textwidth'/'shiftwidth' and the `:right` argument, and
//! rewrites the leading whitespace.  [`do_ascii`] is `ga`: the code point under
//! the cursor spelled decimal, hex, octal and by digraph.
//!
//! Original: `src/nvim/ex_cmds.c`, Vim/Neovim, Vim license.

#![deny(unsafe_op_in_unsafe_fn)]

use super::{CAR, CMD_center, CMD_left, CMD_right, EOL_MAC, FAIL, NL, TAB};
use crate::api::private::helpers::cstr_as_string;
use crate::ascii::ascii_iswhite;
use crate::change::changed_lines;
use crate::charset::{transchar, transchar_nonprint, vim_isprintc};
use crate::cursor::{get_cursor_line_ptr, get_cursor_pos_ptr};
use crate::digraph::get_digraph_for_char;
use crate::edit::{BeginlineOpts, beginline};
use crate::indent::{get_indent, set_indent};
use crate::main::curbuf;
use crate::mbyte::{
    utf_char2bytes, utf_iscomposing_first, utf_ptr2char, utf_ptr2len, utfc_ptr2len,
};
use crate::message::{msg, msg_clr_eos, msg_end, msg_multiline, msg_sb_eol, msg_start};
use crate::option::get_fileformat;
use crate::os::cshim::gettext;
use crate::plines::linetabsize_str;
use crate::strings::vim_snprintf;
use crate::types::{IOSIZE, NUL, cmdidx_T, exarg_T};
use crate::undo::u_save;
use crate::winlayer::{Buf, Win};
use ::libc::atoi;
use core::ffi::{CStr, c_char, c_int};

/// `:ascii` and `ga` -- describe the code point under the cursor.
///
/// The first line describes the base character, and one further line each
/// combining character stacked on it.  `eap` is unused: `ga` calls this with
/// no Ex command at all.
///
/// # Safety
/// The cursor must be on a valid position of the current buffer.
pub unsafe fn do_ascii(_eap: *mut exarg_T) {
    // SAFETY: caller's contract; the cursor is on a live line.
    let data = get_cursor_pos_ptr();
    // SAFETY: `data` points into a NUL-terminated buffer line.
    let len = unsafe { utfc_ptr2len(data) } as usize;
    if len == 0 {
        // SAFETY: a literal.
        unsafe { msg(c"NUL".as_ptr(), 0) };
        return;
    }

    let mut need_clear = true;
    // The line being described. Upstream assembles it in `IObuff`, which
    // `msg_multiline` reads again as it re-enters the message machinery.
    let mut line = [0 as c_char; IOSIZE as usize];
    // SAFETY: message state, main thread.
    unsafe { msg_sb_eol() };
    unsafe { msg_start() };

    // SAFETY: `data` is a live, NUL-terminated line position.
    let mut c = unsafe { utf_ptr2char(data) };
    let mut off = 0;

    // TODO(bfredl): merge this with the main loop
    if c < 0x80 {
        if c == NL {
            // NUL is stored as NL.
            c = NUL;
        }
        // NL is stored as CR.
        // SAFETY: `curbuf` is the live current buffer.
        let mac = c == CAR && unsafe { get_fileformat(curbuf.get()) } == EOL_MAC;
        // SAFETY: `c` came out of the buffer.
        unsafe { describe_byte(c, if mac { NL } else { c }, &mut need_clear, &mut line) };
        // needed for overlong ascii?
        // SAFETY: as above.
        off += unsafe { utf_ptr2len(data) } as usize;
    }

    // Repeat for combining characters, also handle multibyte here.
    while off < len {
        // SAFETY: `off` is a character boundary short of the sequence's end.
        c = unsafe { utf_ptr2char(data.add(off)) };
        // SAFETY: `c` came out of the buffer.
        unsafe { describe_char(c, off > 0, &mut need_clear, &mut line) };
        // SAFETY: as above.
        off += unsafe { utf_ptr2len(data.add(off)) } as usize;
    }

    if need_clear {
        // SAFETY: message state, main thread.
        unsafe { msg_clr_eos() };
    }
    // SAFETY: message state, main thread.
    unsafe { msg_end() };
}

/// The `:ascii` line for a single-byte character: its `<x>` rendering, the
/// value decimal, hex and octal, and its digraph if it has one.
///
/// `cval` is the value to report, which differs from `c` only for a CR in a
/// 'fileformat' of "mac".
///
/// # Safety
/// Message state must be started; `need_clear` must be live.
unsafe fn describe_byte(
    c: c_int,
    cval: c_int,
    need_clear: &mut bool,
    line: &mut [c_char; IOSIZE as usize],
) {
    let mut nonprint: [c_char; 20] = [0; 20];
    // SAFETY: `nonprint` is 20 bytes and `transchar_nonprint` writes at most
    // seven into `raw` before the `  <%s>` wrapper takes it.
    if unsafe { vim_isprintc(c) } && !(' ' as c_int..='~' as c_int).contains(&c) {
        let mut raw: [c_char; 7] = [0; 7];
        unsafe { transchar_nonprint(curbuf.get(), raw.as_mut_ptr(), c) };
        unsafe {
            vim_snprintf(
                nonprint.as_mut_ptr(),
                nonprint.len(),
                c"  <%s>".as_ptr(),
                raw.as_ptr(),
            )
        };
    }

    // Upstream keeps a second, permanently empty buffer here and prints it
    // between the two; it has been dead since the multi-byte rewrite.
    let empty: [c_char; 1] = [0];
    // The digraph is always passed: the format without a `Digr %s` simply
    // never reads that argument, as any unused trailing vararg.
    let digraph = get_digraph_for_char(cval);
    let fmt = match digraph {
        Some(_) => c"<%s>%s%s  %d,  Hex %02x,  Oct %03o, Digr %s",
        None => c"<%s>%s%s  %d,  Hex %02x,  Octal %03o",
    };
    let digraph = digraph.unwrap_or([0; 3]);
    // SAFETY: `line` is `IOSIZE` bytes and `vim_snprintf` bounds itself by
    // the length it is given; `transchar` returns a static NUL-terminated
    // string.
    unsafe {
        vim_snprintf(
            line.as_mut_ptr(),
            IOSIZE as usize,
            gettext(fmt.as_ptr()),
            transchar(c).as_ptr(),
            nonprint.as_ptr(),
            empty.as_ptr(),
            cval,
            cval,
            cval,
            digraph.as_ptr(),
        )
    };
    // SAFETY: `line` now holds a NUL-terminated string.
    unsafe { emit_line(line, need_clear) };
}

/// The `:ascii` line for one character of a multi-byte or combining sequence.
///
/// `spaced` asks for the separating space upstream writes before every
/// character but the first.
///
/// # Safety
/// Message state must be started; `need_clear` must be live.
unsafe fn describe_char(
    c: c_int,
    spaced: bool,
    need_clear: &mut bool,
    line: &mut [c_char; IOSIZE as usize],
) {
    // This assumes every multi-byte char is printable...
    let used = {
        let buf = &mut *line;
        let mut len = 0;
        if spaced {
            buf[len] = b' ' as c_char;
            len += 1;
        }
        buf[len] = b'<' as c_char;
        len += 1;
        if utf_iscomposing_first(c) {
            // Draw composing char on top of a space.
            buf[len] = b' ' as c_char;
            len += 1;
        }
        // SAFETY: at most four bytes go in, with 1020 left.
        len + unsafe { utf_char2bytes(c, buf.as_mut_ptr().add(len)) } as usize
    };

    // Four formats: with and without a digraph, and hex in four or eight
    // digits.  The digraph argument goes out either way (see `describe_byte`).
    let digraph = get_digraph_for_char(c);
    let fmt = match (digraph.is_some(), c < 0x10000) {
        (true, true) => c"> %d, Hex %04x, Oct %o, Digr %s",
        (true, false) => c"> %d, Hex %08x, Oct %o, Digr %s",
        (false, true) => c"> %d, Hex %04x, Octal %o",
        (false, false) => c"> %d, Hex %08x, Octal %o",
    };
    let digraph = digraph.unwrap_or([0; 3]);
    // SAFETY: `used` bytes of `line` are written and `vim_snprintf` bounds
    // itself by the room reported left.
    unsafe {
        vim_snprintf(
            line.as_mut_ptr().add(used),
            IOSIZE as usize - used,
            gettext(fmt.as_ptr()),
            c,
            c,
            c,
            digraph.as_ptr(),
        )
    };
    // SAFETY: `line` now holds a NUL-terminated string.
    unsafe { emit_line(line, need_clear) };
}

/// Print what `line` holds as one `:ascii` line.
///
/// # Safety
/// `line` must hold a NUL-terminated string and message state must be
/// started.
unsafe fn emit_line(line: &mut [c_char; IOSIZE as usize], need_clear: &mut bool) {
    // SAFETY: caller's contract.
    unsafe {
        msg_multiline(
            cstr_as_string(line.as_mut_ptr()),
            0,
            true,
            false,
            need_clear,
        )
    };
}

/// `:left`, `:center` and `:right` -- re-indent every line of the range.
///
/// # Safety
/// `eap` must be a live Ex command whose range is inside the current buffer.
pub unsafe fn ex_align(eap: *mut exarg_T) {
    // SAFETY: caller's contract.
    let (mut cmdidx, arg, line1, line2) =
        unsafe { ((*eap).cmdidx, (*eap).arg, (*eap).line1, (*eap).line2) };

    // SAFETY: `curwin` is the live current window.
    if cur_win().w_onebuf_opt.wo_rl != 0 {
        // Switch left and right aligning.  Upstream rewrites the command
        // itself, and that outlives the call.
        cmdidx = match cmdidx {
            CMD_right => CMD_left,
            CMD_left => CMD_right,
            other => other,
        };
        // SAFETY: caller's contract.
        unsafe { (*eap).cmdidx = cmdidx };
    }

    // SAFETY: `arg` is the command's NUL-terminated argument.
    let arg_width = unsafe { atoi(arg) };
    let mut indent = 0;
    let mut width = 0;
    if cmdidx == CMD_left {
        // The argument is the new indent.
        indent = arg_width.max(0);
    } else {
        // If 'textwidth' is set use it, else if 'wrapmargin' is set use it;
        // on an invalid value use 80.
        width = if arg_width > 0 {
            arg_width
        } else {
            cur_buf().b_p_tw as c_int
        };
        if width == 0 && cur_buf().b_p_wm > 0 {
            width = cur_win().w_view_width - cur_buf().b_p_wm as c_int;
        }
        if width <= 0 {
            width = 80;
        }
    }

    // SAFETY: `curwin` is live; `u_save` takes the range's guard lines.
    let save_curpos = cur_win().w_cursor;
    // SAFETY: as above.
    if u_save(line1 - 1, line2 + 1) == FAIL {
        return;
    }

    let mut lnum = line1;
    while lnum <= line2 {
        // SAFETY: `lnum` is inside the range `u_save` just guarded, and
        // nothing in the body adds or removes a line.
        cur_win().w_cursor.lnum = lnum;
        if let Some(new_indent) = unsafe { aligned_indent(cmdidx, indent, width) } {
            // SAFETY: the cursor is on `lnum`.
            unsafe { set_indent(new_indent.max(0), 0) };
        }
        lnum += 1;
    }

    // SAFETY: the range is still the one that was just rewritten.
    changed_lines(cur_buf(), line1, 0, line2 + 1, 0, true);
    cur_win().w_cursor = save_curpos;
    beginline(BeginlineOpts::WHITE | BeginlineOpts::FIX);
}

/// The indent the cursor's line should get, or `None` for a blank line
/// `:center`/`:right` skips.
///
/// # Safety
/// The cursor must be on the line to measure.
unsafe fn aligned_indent(cmdidx: cmdidx_T, indent: c_int, width: c_int) -> Option<c_int> {
    if cmdidx == CMD_left {
        return Some(indent);
    }
    // SAFETY: caller's contract.
    let (linewidth, has_tab) = unsafe { linelen() };
    // SAFETY: as above.
    let len = linewidth - get_indent();
    if len <= 0 {
        // Skip blank lines.
        return None;
    }
    if cmdidx == CMD_center {
        return Some((width - len) / 2);
    }
    if has_tab {
        // SAFETY: as above.
        return Some(unsafe { fit_right_indent(width - len, width) });
    }
    Some(width - len)
}

/// `:right` on a line holding a TAB: the width the line ends up with is not a
/// function of the indent alone, so upstream searches for the largest indent
/// that still fits, one column at a time.
///
/// # Safety
/// The cursor must be on the line being aligned.
unsafe fn fit_right_indent(mut indent: c_int, width: c_int) -> c_int {
    while indent > 0 {
        // SAFETY: caller's contract.
        unsafe { set_indent(indent, 0) };
        // SAFETY: as above.
        if unsafe { linelen().0 } > width {
            indent -= 1;
            continue;
        }
        // It fits: now move it as far right as it will go.
        loop {
            indent += 1;
            // SAFETY: as above.
            unsafe { set_indent(indent, 0) };
            // SAFETY: as above.
            if unsafe { linelen().0 } > width {
                return indent - 1;
            }
        }
    }
    indent
}

/// The display width of the cursor's line ignoring trailing white space, and
/// whether the text between the first and last non-blank holds a TAB.
///
/// Upstream asks for the second answer through an out-parameter it passes
/// NULL for when it does not want it; computing it costs a memchr, so this
/// always answers both.
///
/// # Safety
/// The cursor must be on a live line of the current buffer.
unsafe fn linelen() -> (c_int, bool) {
    // Get the line.  If it's empty bail out early (could be the empty string
    // for an unloaded buffer).
    // SAFETY: caller's contract.
    let line = get_cursor_line_ptr();
    // SAFETY: buffer lines are NUL-terminated.
    let text = unsafe { CStr::from_ptr(line) }.to_bytes();
    if text.is_empty() {
        return (0, false);
    }

    // The first non-blank, and the character after the last non-blank.
    let first = text
        .iter()
        .position(|&b| !ascii_iswhite(b as c_int))
        .unwrap_or(text.len());
    let last = text
        .iter()
        .rposition(|&b| !ascii_iswhite(b as c_int))
        .map_or(first, |i| i + 1);
    let has_tab = text[first..last].contains(&(TAB as u8));

    // Measure with the trailing white space cut off, then put it back.
    // SAFETY: `last` indexes the line's own bytes, and `linetabsize_str` only
    // reads up to the NUL just written.
    let len = unsafe {
        let end = line.add(last);
        let saved = *end;
        *end = NUL as c_char;
        let len = linetabsize_str(line);
        *end = saved;
        len
    };
    (len, has_tab)
}

/// The buffer the editor is working in.
fn cur_buf() -> Buf {
    // SAFETY: `curbuf` is set from startup to exit.
    unsafe { Buf::current() }
}

/// The window the editor is working in.
fn cur_win() -> Win {
    // SAFETY: `curwin` is set from startup to exit.
    unsafe { Win::current() }
}
