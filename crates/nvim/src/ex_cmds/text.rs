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

#![forbid(unsafe_code)]

use super::say;
use super::{CAR, EOL_MAC, NL, TAB};
use crate::ascii::ascii_iswhite;
use crate::change::changed_lines;
use crate::charset::{transchar, transchar_nonprint_form, vim_isprintc};
use crate::cstr;
use crate::digraph::get_digraph_for_char;
use crate::edit::{BeginlineOpts, beginline};
use crate::indent::{get_indent, set_indent};
use crate::mbyte::{char_at, char_len, cluster_len, encode_char, utf_iscomposing_first};
use crate::memline::Lines;
use crate::message::{msg, msg_multiline, msg_sb_eol};
use crate::option::get_fileformat;
use crate::os::cshim::gettext;
use crate::plines::linetabsize_text;
use crate::types::CmdIdx;
use crate::types::MB_MAXCHAR;
use crate::types::{ExArg, IOSIZE, NUL};
use crate::undo::u_save;
use crate::winlayer::Buf;
use crate::winlayer::Win;
use core::ffi::{CStr, c_int};

/// `:ascii`.
pub fn do_ascii(_excmd: &mut ExArg) {
    describe_cursor_char();
}

/// Describe the code point under the cursor -- what `:ascii` and Normal
/// mode's `ga` both do.
///
/// The first line describes the base character, and one further line each
/// combining character stacked on it. Split from the handler because `ga`
/// has no Ex command to hand it.
pub(crate) fn describe_cursor_char() {
    let cursor = Win::current().w_cursor;
    let mut lines = Lines::current();
    let line = lines.line(cursor.lnum);
    let data = &line[(cursor.col.max(0) as usize).min(line.len())..];
    let len = cluster_len(data);
    // A copy of the character and its composing ones: the messages below
    // may redraw, which reads other lines through the same cache.
    let data = data[..len].to_vec();
    let data = data.as_slice();
    if len == 0 {
        msg(c"NUL", 0);
        return;
    }

    let mut need_clear = true;
    msg_sb_eol();
    say::start();

    let mut c = char_at(data);
    let mut off = 0;

    // TODO(bfredl): merge this with the main loop
    if c < 0x80 {
        if c == NL {
            // NUL is stored as NL.
            c = NUL;
        }
        // NL is stored as CR.
        let mac = c == CAR && get_fileformat(Buf::current()) == EOL_MAC;
        describe_byte(c, if mac { NL } else { c }, &mut need_clear);
        // needed for overlong ascii?
        off += char_len(data);
    }

    // Repeat for combining characters, also handle multibyte here.
    while off < len {
        // `off` is a character boundary short of the sequence's end.
        c = char_at(&data[off..]);
        describe_char(c, off > 0, &mut need_clear);
        off += char_len(&data[off..]);
    }

    if need_clear {
        say::clear_eos();
    }
    say::end();
}

/// The `:ascii` line for a single-byte character: its `<x>` rendering, the
/// value decimal, hex and octal, and its digraph if it has one.
///
/// `cval` is the value to report, which differs from `c` only for a CR in a
/// 'fileformat' of "mac". Upstream assembles the line in `IObuff`; it is cut
/// where that would cut it.
fn describe_byte(c: c_int, cval: c_int, need_clear: &mut bool) {
    // The `  <x>` form of a printable character outside ASCII, in the
    // twenty bytes upstream gives it.
    let nonprint = if vim_isprintc(c) && !(' ' as c_int..='~' as c_int).contains(&c) {
        let raw = transchar_nonprint_form(Buf::current_or_none(), c);
        crate::formatted!(20, b"  <%s>", cstr::in_chars(&raw))
    } else {
        Vec::new()
    };

    // Upstream keeps a second, permanently empty buffer here and prints it
    // between the two; it has been dead since the multi-byte rewrite.
    let empty: &[u8] = b"";
    // The digraph is always passed: the format without a `Digr %s` simply
    // never reads that argument, as any unused trailing vararg.
    let digraph = get_digraph_for_char(cval);
    let fmt = match digraph {
        Some(_) => c"<%s>%s%s  %d,  Hex %02x,  Oct %03o, Digr %s",
        None => c"<%s>%s%s  %d,  Hex %02x,  Octal %03o",
    };
    let digraph = digraph.unwrap_or([0; 3]);
    let shown = transchar(c);
    let line = crate::formatted!(
        IOSIZE as usize,
        gettext(fmt).to_bytes(),
        cstr::in_chars(&shown),
        &nonprint[..],
        empty,
        cval,
        cval,
        cval,
        cstr::in_bytes(&digraph),
    );
    emit_line(&line, need_clear);
}

/// The `:ascii` line for one character of a multi-byte or combining sequence.
///
/// `spaced` asks for the separating space upstream writes before every
/// character but the first.
fn describe_char(c: c_int, spaced: bool, need_clear: &mut bool) {
    // This assumes every multi-byte char is printable...
    let mut line = Vec::with_capacity(64);
    if spaced {
        line.push(b' ');
    }
    line.push(b'<');
    if utf_iscomposing_first(c) {
        // Draw composing char on top of a space.
        line.push(b' ');
    }
    let mut ch = [0u8; MB_MAXCHAR];
    let len = encode_char(c, &mut ch);
    line.extend_from_slice(&ch[..len]);

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
    let room = IOSIZE as usize - line.len();
    let fmt = gettext(fmt).to_bytes();
    let rest = crate::formatted!(room, fmt, c, c, c, cstr::in_bytes(&digraph));
    line.extend_from_slice(&rest);
    emit_line(&line, need_clear);
}

/// Print `line` as one `:ascii` line; message state must be started.
fn emit_line(line: &[u8], need_clear: &mut bool) {
    msg_multiline(line, 0, true, false, need_clear);
}

/// `:left`, `:center` and `:right` -- re-indent every line of the range.
pub fn ex_align(excmd: &mut ExArg) {
    let (mut cmdidx, line1, line2) = (excmd.cmdidx, excmd.line1, excmd.line2);

    if Win::current().w_onebuf_opt.wo_rl != 0 {
        // Switch left and right aligning.  Upstream rewrites the command
        // itself, and that outlives the call.
        cmdidx = match cmdidx {
            CmdIdx::right => CmdIdx::left,
            CmdIdx::left => CmdIdx::right,
            other => other,
        };
        excmd.cmdidx = cmdidx;
    }

    // `atoi`: `strtol` cut to an `int`.
    let arg_width = crate::charset::skip::decimal(excmd.line.arg()).0 as c_int;
    let mut indent = 0;
    let mut width = 0;
    if cmdidx == CmdIdx::left {
        // The argument is the new indent.
        indent = arg_width.max(0);
    } else {
        // If 'textwidth' is set use it, else if 'wrapmargin' is set use it;
        // on an invalid value use 80.
        width = if arg_width > 0 {
            arg_width
        } else {
            Buf::current().b_p_tw as c_int
        };
        if width == 0 && Buf::current().b_p_wm > 0 {
            width = Win::current().w_view_width - Buf::current().b_p_wm as c_int;
        }
        if width <= 0 {
            width = 80;
        }
    }

    let save_curpos = Win::current().w_cursor;
    if u_save(line1 - 1, line2 + 1).is_err() {
        return;
    }

    let mut lnum = line1;
    while lnum <= line2 {
        Win::current().w_cursor.lnum = lnum;
        if let Some(new_indent) = aligned_indent(cmdidx, indent, width) {
            set_indent(new_indent.max(0), 0);
        }
        lnum += 1;
    }

    changed_lines(Buf::current(), line1, 0, line2 + 1, 0, true);
    Win::current().w_cursor = save_curpos;
    beginline(BeginlineOpts::WHITE | BeginlineOpts::FIX);
}

/// The indent the cursor's line should get, or `None` for a blank line
/// `:center`/`:right` skips.
fn aligned_indent(cmdidx: CmdIdx, indent: c_int, width: c_int) -> Option<c_int> {
    if cmdidx == CmdIdx::left {
        return Some(indent);
    }
    let (linewidth, has_tab) = linelen();
    let len = linewidth - get_indent();
    if len <= 0 {
        // Skip blank lines.
        return None;
    }
    if cmdidx == CmdIdx::center {
        return Some((width - len) / 2);
    }
    if has_tab {
        return Some(fit_right_indent(width - len, width));
    }
    Some(width - len)
}

/// `:right` on a line holding a TAB: the width the line ends up with is not a
/// function of the indent alone, so upstream searches for the largest indent
/// that still fits, one column at a time.
fn fit_right_indent(mut indent: c_int, width: c_int) -> c_int {
    while indent > 0 {
        set_indent(indent, 0);
        if linelen().0 > width {
            indent -= 1;
            continue;
        }
        // It fits: now move it as far right as it will go.
        loop {
            indent += 1;
            set_indent(indent, 0);
            if linelen().0 > width {
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
fn linelen() -> (c_int, bool) {
    // Get the line.  If it's empty bail out early (could be the empty string
    // for an unloaded buffer).
    let mut lines = Lines::current();
    let text = lines.line(Win::current().w_cursor.lnum);
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

    // Measure with the trailing white space cut off. Upstream writes a NUL
    // into the buffer line for that and puts the byte back; a copy of the
    // head is measured instead.
    let mut head = text[..last].to_vec();
    head.push(NUL as u8);
    let head = CStr::from_bytes_until_nul(&head).expect("just terminated");
    let len = linetabsize_text(0, head);
    (len, has_tab)
}
