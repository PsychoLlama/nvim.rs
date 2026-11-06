//! `:marks` and `:delmarks`.
//!
//! The listing is a surface of its own rather than a rendering of
//! `getmarklist()`: it prints the internal 0-based column, it shows a mark
//! whose line no longer exists as `-invalid-`, and it re-orders `'<`/`'>` so
//! that the earlier end is always `'<`. A change that moves one of the two
//! surfaces and not the other is what `1787242636-jmarksweep`'s `K` and `X`
//! lines are laid out side by side to catch.

#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use crate::buffer::{buf_is_prompt, buflist_nr2name, find_buf};
use crate::charset::ptr2cells_at;
use crate::getchar::state::got_int;
use crate::global_cell::GlobalCell;
use crate::mbyte::{c_cells_at, c_cluster_len};
use crate::memory::XString;
use crate::message::{e_argreq, e_invarg};
use crate::message::{
    emsg, message_filtered, msg, msg_display, msg_display_bytes, msg_ext_set_kind, msg_putchar,
    msg_title,
};
use crate::message_fmt::msg_bytes;
use crate::os::cshim::gettext;
use crate::os::time::os_time;
use crate::pos::lt;
use crate::semsg;
use crate::ui::state::Columns;
use crate::winlayer::{Buf, Win};
use core::ffi::{CStr, c_char, c_int};

use super::store::{UNSET_POS, global_mark_names, mark_name, nth_name, slot};
use super::*;
use crate::highlight_group::HLF_D;

/// print the marks
pub fn ex_marks(excmd: &mut ExArg) {
    // A copy of the filter: listing a mark reads buffers, and runs nothing
    // that may not reach the command line. An empty argument is the same as
    // none: `:marks` with a trailing space must not filter everything out.
    let filter = excmd.line.arg().to_vec();
    let arg = (!filter.is_empty()).then_some(&filter[..]);
    let (win, buf) = (Win::current(), Buf::current());
    msg_ext_set_kind(c"list_cmd");

    // `'<` is whichever end of the Visual range comes FIRST, so a selection
    // made backwards still lists in order.
    let (start, end) = (buf.b_visual.vi_start, buf.b_visual.vi_end);
    let starts_first = (lt(start, end) || end.lnum == LineNr::ZERO) && start.lnum != LineNr::ZERO;
    let (first, second) = if starts_first {
        (start, end)
    } else {
        (end, start)
    };

    // Every row that carries no file name, in listing order. The global table
    // goes between the locals and the tick family, which is why it is not one
    // list.
    let locals: Vec<(u8, Pos)> = (0..NMARKS)
        .map(|i| (nth_name(b'a', i), buf.b_namedm[slot(i)].mark))
        .collect();
    let ticks = [
        (b'"', buf.b_last_cursor.mark),
        (b'[', buf.b_op_start),
        (b']', buf.b_op_end),
        (b'^', buf.b_last_insert.mark),
        (b'.', buf.b_last_change.mark),
    ];
    let prompt = buf_is_prompt(Some(buf)).then(|| (b':', buf.b_prompt_start.mark));
    let visual = [(b'<', first), (b'>', second)];

    show_one_mark(b'\'', arg, win.w_pcmark, None, true);
    for (c, pos) in locals {
        show_one_mark(c, arg, pos, None, true);
    }
    // The global table, whose rows carry a FILE NAME rather than the
    // line's text when the mark is in another buffer: the loaded buffer's
    // name, or the one a mark out of the shada file still carries.
    for (i, name) in global_mark_names() {
        let mark = store::global_mark(i);
        let fnum = mark.fmark.fnum;
        let file = if fnum.is_some() {
            fm_getname(&mark.fmark, 15)
        } else {
            mark.file_name().map(XString::from_cstr)
        };
        let Some(file) = file else {
            continue;
        };
        let current = fnum == Some(buf.handle);
        show_one_mark(name, arg, mark.fmark.mark, Some(file.as_cstr()), current);
    }
    for (c, pos) in ticks.into_iter().chain(prompt).chain(visual) {
        show_one_mark(c, arg, pos, None, true);
    }
    // The footer: "No marks set" (or E283) if nothing above printed a
    // title.
    finish_marks(arg);
}

/// The `-1` row of upstream's `show_one_mark`: whatever `:marks` prints when
/// it printed nothing else.
fn finish_marks(arg: Option<&[u8]>) {
    if DID_TITLE.replace(false) {
        return;
    }
    match arg {
        None => {
            msg(gettext(c"No marks set"), 0);
        }
        Some(arg) => {
            let arg = msg_bytes(arg);
            semsg!("E283: No marks matching \"{arg}\"");
        }
    }
}

/// Whether `:marks` has printed its column header. Reset by [`finish_marks`],
/// so the next `:marks` prints it again.
static DID_TITLE: GlobalCell<bool> = GlobalCell::new(false);

/// One `:marks` row.
///
/// `arg` is the filter, `name_arg` the file name a row in another file
/// shows, and `current` says the mark is in the current file -- a row there
/// with no name shows the LINE'S TEXT.
pub(super) fn show_one_mark(
    c: u8,
    arg: Option<&[u8]>,
    pos: Pos,
    name_arg: Option<&CStr>,
    current: bool,
) {
    let wanted = arg.is_none_or(|arg| arg.contains(&c));
    if got_int.get() || !wanted || pos.lnum == LineNr::ZERO {
        return;
    }

    let text;
    let name = match name_arg {
        Some(name) => Some(name),
        None if current => {
            text = mark_line(pos, 15);
            Some(text.as_cstr())
        }
        None => None,
    };
    if message_filtered(name.unwrap_or(c"")) {
        return;
    }
    if !DID_TITLE.replace(true) {
        msg_title(gettext(c"\nmark line  col file/text"));
    }
    msg_putchar(c_int::from(b'\n'));
    if got_int.get() {
        return;
    }
    // ` %c %6d %4d `
    let mut row = Row::new();
    row.push(b' ');
    row.push(c);
    row.push(b' ');
    row.number(pos.lnum.get(), 6);
    row.push(b' ');
    row.number(pos.col.get(), 4);
    row.push(b' ');
    msg_display_bytes(row.bytes(), 0, false);
    if let Some(name) = name {
        msg_display(name, if current { HLF_D } else { 0 }, false);
    }
}

/// The numbers at the start of a `:marks`, `:jumps` or `:changes` row, laid
/// out as upstream's `snprintf` into `IObuff` lays them -- by hand, in a
/// stack buffer: the crate's `printf` makes two passes, which cost a
/// `:marks` loop twice what the listing itself does.
pub(super) struct Row {
    bytes: [u8; 48],
    len: usize,
}

impl Row {
    pub(super) fn new() -> Row {
        Row {
            bytes: [0; 48],
            len: 0,
        }
    }

    pub(super) fn push(&mut self, byte: u8) {
        if let Some(at) = self.bytes.get_mut(self.len) {
            *at = byte;
            self.len += 1;
        }
    }

    /// `%<width>d` of `n`.
    pub(super) fn number(&mut self, n: c_int, width: usize) {
        let mut digits = [0u8; 11];
        let mut at = digits.len();
        let mut rest = n.unsigned_abs();
        loop {
            at -= 1;
            digits[at] = b'0' + u8::try_from(rest % 10).unwrap_or(0);
            rest /= 10;
            if rest == 0 {
                break;
            }
        }
        if n < 0 {
            at -= 1;
            digits[at] = b'-';
        }
        for _ in digits.len() - at..width {
            self.push(b' ');
        }
        for &digit in &digits[at..] {
            self.push(digit);
        }
    }

    pub(super) fn bytes(&self) -> &[u8] {
        &self.bytes[..self.len]
    }
}

/// `:delmarks[!] [marks]`
pub fn ex_delmarks(excmd: &mut ExArg) {
    // A copy, which the messages below quote from.
    let names = excmd.line.arg().to_vec();
    let forceit = excmd.forceit;
    let mut buf = Buf::current();
    let empty = names.is_empty();

    if empty && forceit {
        delmarks_all(buf);
        return;
    }
    if forceit {
        // `:delmarks!` takes no argument at all; naming one is E474 rather
        // than "clear these, forcefully".
        emsg(gettext(e_invarg));
        return;
    }
    if empty {
        emsg(gettext(e_argreq));
        return;
    }

    let timestamp = os_time();
    // The MarkSet payload's position, shared by every announcement here: a
    // deletion reports the mark at line 0, not where it used to be.
    let gone = UNSET_POS;

    let mut i = 0;
    while i < names.len() {
        let here = names[i];
        // Where the message points: the rest of the argument from here, which
        // is what upstream's `%s` prints.
        let rest = msg_bytes(&names[i..]);
        let lower = here.is_ascii_lowercase();
        let digit = here.is_ascii_digit();
        let upper = here.is_ascii_uppercase();
        if !(lower || digit || upper) {
            if !delmarks_one(&mut buf, here, timestamp) {
                semsg!("E475: Invalid argument: {rest}");
                return;
            }
            i += 1;
            continue;
        }
        // A range `x-y`: both ends must be in the same class, and the second
        // must not come before the first. The `-` form looks two bytes
        // ahead, which over a slice is a bounds check.
        let from = here;
        let to = if names.get(i + 1) == Some(&b'-') {
            let end = names.get(i + 2).copied().unwrap_or(0);
            let same_class = if lower {
                end.is_ascii_lowercase()
            } else if digit {
                end.is_ascii_digit()
            } else {
                end.is_ascii_uppercase()
            };
            if !same_class || end < from {
                semsg!("E475: Invalid argument: {rest}");
                return;
            }
            i += 2;
            end
        } else {
            from
        };
        for c in from..=to {
            if lower {
                let at = usize::from(c - b'a');
                if buf.b_namedm[at].is_set() {
                    do_markset_autocmd(mark_name(c), gone, buf);
                }
                // Only the line and the timestamp are cleared, not the whole
                // record: `:delmarks a` is not `clear_fmark`, and the shada
                // writer wants the stamp to say when.
                let mark = &mut buf.b_namedm[at];
                mark.mark.lnum = LineNr::ZERO;
                mark.timestamp = timestamp;
            } else {
                // The fourth and fifth writings of the `namedfm` index
                // formula; see `lookup::mark_global_index`.
                let n = if digit {
                    usize::from(c - b'0') + slot(NMARKS)
                } else {
                    usize::from(c - b'A')
                };
                let fnum = namedfm.with(|table| {
                    let mark = &table[n].fmark;
                    mark.is_set().then_some(mark.fnum)
                });
                if let Some(fnum) = fnum {
                    // The event is announced against the mark's OWN buffer
                    // where it still exists, so an autocommand sees the file
                    // the mark was in.
                    let owner = find_buf(fnum).unwrap_or(buf);
                    do_markset_autocmd(mark_name(c), gone, owner);
                }
                namedfm.with_mut(|table| {
                    let slot = &mut table[n];
                    slot.fmark.mark.lnum = LineNr::ZERO;
                    slot.fmark.fnum = None;
                    slot.fmark.timestamp = timestamp;
                    slot.fname = None;
                });
            }
        }
        i += 1;
    }
}

/// `:delmarks!` — every buffer-local mark, plus the tick family.
fn delmarks_all(buffer: Buf) {
    // Announced before the clearing, so an autocommand can still read the
    // buffer the mark was in. `'<`/`'>` are NOT announced and not cleared:
    // `clrallmarks` leaves the Visual range alone.
    for i in 0..NMARKS {
        if buffer.b_namedm[slot(i)].is_set() {
            do_markset_autocmd(mark_name(nth_name(b'a', i)), UNSET_POS, buffer);
        }
    }
    for (name, set) in [
        ('"', buffer.b_last_cursor.is_set()),
        ('^', buffer.b_last_insert.is_set()),
        ('.', buffer.b_last_change.is_set()),
        ('[', buffer.b_op_start.lnum != LineNr::ZERO),
        (']', buffer.b_op_end.lnum != LineNr::ZERO),
    ] {
        if set {
            do_markset_autocmd(name as c_char, UNSET_POS, buffer);
        }
    }
    clrallmarks(buffer, os_time());
}

/// One non-alphanumeric `:delmarks` name. `false` means the name is not a
/// mark at all, which is E474 in the caller.
fn delmarks_one(buffer: &mut Buf, name: u8, timestamp: Timestamp) -> bool {
    // `:` and a space are accepted and do nothing: the prompt mark is not the
    // user's to delete, and a space is how `:delmarks a b` separates names.
    let lnum = match name {
        b'"' => buffer.b_last_cursor.mark.lnum,
        b'^' => buffer.b_last_insert.mark.lnum,
        b'.' => buffer.b_last_change.mark.lnum,
        b'[' => buffer.b_op_start.lnum,
        b']' => buffer.b_op_end.lnum,
        b'<' => buffer.b_visual.vi_start.lnum,
        b'>' => buffer.b_visual.vi_end.lnum,
        b':' | b' ' => return true,
        _ => return false,
    };
    if lnum != LineNr::ZERO {
        do_markset_autocmd(mark_name(name), UNSET_POS, *buffer);
    }
    // The three fmark stores are released; the four positions are only
    // invalidated, because they own nothing.
    match name {
        b'"' => buffer.b_last_cursor.clear(timestamp),
        b'^' => buffer.b_last_insert.clear(timestamp),
        b'.' => buffer.b_last_change.clear(timestamp),
        b'[' => buffer.b_op_start.lnum = LineNr::ZERO,
        b']' => buffer.b_op_end.lnum = LineNr::ZERO,
        b'<' => buffer.b_visual.vi_start.lnum = LineNr::ZERO,
        _ => buffer.b_visual.vi_end.lnum = LineNr::ZERO,
    }
    true
}

/// Return the line at mark "mp".  Truncate to fit in window.
pub(super) fn mark_line(pos: Pos, lead_len: c_int) -> XString {
    let buf = Buf::current();
    if pos.lnum == LineNr::ZERO || pos.lnum > buf.b_ml.ml_line_count {
        return XString::from_cstr(c"-invalid-");
    }
    debug_assert!(Columns.get() >= 0, "Columns >= 0");
    let mut lines = buf.lines();
    let line = lines.line(pos.lnum);
    let start = line
        .iter()
        .position(|&b| b != b' ' && b != b'\t')
        .unwrap_or(line.len());
    // Five bytes per column is the widest a displayed cell can be, so the
    // copy can never be shorter than what the truncation below keeps.
    let room = usize::try_from(Columns.get()).unwrap_or(0) * 5;
    let s = &line[start..line.len().min(start + room)];
    // Truncate to the screen width, measured in CELLS rather than bytes so a
    // multi-byte or double-width character is not cut in half.
    let mut at = 0;
    let mut len: c_int = 0;
    while at < s.len() {
        let rest = &s[at..];
        len += if rest[0] >= 0x80 {
            c_cells_at(rest)
        } else {
            ptr2cells_at(rest)
        };
        if len >= Columns.get() - lead_len {
            break;
        }
        at += c_cluster_len(rest).max(1);
    }
    XString::from_bytes(&s[..at.min(s.len())])
}

/// The name of the file a filemark is in — or, when it is in the current
/// buffer, the text at the mark. `None` when its buffer is gone.
pub fn fm_getname(fm: &FileMark, lead_len: c_int) -> Option<XString> {
    if fm.fnum == Some(Buf::current().handle) {
        return Some(mark_line(fm.mark, lead_len));
    }
    buflist_nr2name(fm.fnum, 0, 1)
}

#[cfg(test)]
mod tests {
    use super::Row;

    fn row(fill: impl FnOnce(&mut Row)) -> String {
        let mut row = Row::new();
        fill(&mut row);
        String::from_utf8(row.bytes().to_vec()).unwrap()
    }

    /// What `%6d`, `%2d` and friends print: right-aligned, the sign counted,
    /// never cut when the number is wider than the field.
    #[test]
    fn numbers_are_laid_out_as_printf_pads_them() {
        assert_eq!(row(|r| r.number(42, 6)), "    42");
        assert_eq!(row(|r| r.number(0, 4)), "   0");
        assert_eq!(row(|r| r.number(-7, 4)), "  -7");
        assert_eq!(row(|r| r.number(1_234_567, 2)), "1234567");
        assert_eq!(row(|r| r.number(i32::MIN, 1)), "-2147483648");
    }

    #[test]
    fn a_full_row_stops_at_its_buffer() {
        let text = row(|r| {
            for _ in 0..10 {
                r.number(1_000_000, 6);
            }
        });
        assert_eq!(text.len(), 48);
    }
}
