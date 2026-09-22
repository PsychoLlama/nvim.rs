//! `g CTRL-G` -- counting what is in the buffer.
//!
//! [`cursor_pos_info`] answers two different questions with one walk of the
//! whole buffer. Outside Visual mode it reports where the cursor is: its
//! column, and its line, word, character and byte *ordinal*. Inside Visual
//! mode it reports the selection instead -- the same counts, but summed over
//! the selected text rather than over everything up to the cursor. That is
//! why [`PosCounts`] carries two sets of totals and every phase updates both.
//!
//! `:h g_CTRL-G` and `wordcount()` are the same function: with a non-null
//! `dict` the counts are stored rather than shown, which is what the API
//! calls.
//!
//! [`get_region_bytecount`] is the byte-only version quickfix and the buffer
//! API use to size a splice; it does not walk the text at all, only the line
//! lengths.

#![deny(unsafe_op_in_unsafe_fn)]
#![allow(unsafe_code)]

use crate::memline::MlFlags;
use crate::winlayer::{Buf, Win};
use core::ffi::c_int;

use super::*;
use crate::message::MSG_IOBUFF_LEN;
use crate::message_fmt::to_message;
use crate::normal::{VisualMode, VisualSelection, sel_exclusive, visual_selection};
use crate::tr;
use crate::types::NUL;

/// Bytes, words and characters in one line, up to `limit` bytes.
///
/// Answers the bytes consumed and *adds* the words and characters to `wc` and
/// `cc`, because every caller is accumulating. A word is a run of
/// non-white-space, which is what `wc(1)` counts too. Reaching the end of the
/// line before `limit` adds `eol_size` for the line break itself.
///
/// # Safety
/// `line` must be NUL-terminated.
fn line_count_info(
    line: &[u8],
    wc: &mut VarNumber,
    cc: &mut VarNumber,
    limit: VarNumber,
    eol_size: c_int,
) -> VarNumber {
    // The walk stops at the end of the line, which `byte_at` answers as the
    // NUL the pointer form stopped on.
    let byte = |i: VarNumber| c_int::from(byte_at(line, i as usize));
    let mut words = 0;
    let mut chars = 0;
    let mut is_word = false;

    let mut i: VarNumber = 0;
    while i < limit && byte(i) != NUL {
        if is_word {
            if ascii_isspace(byte(i)) {
                words += 1;
                is_word = false;
            }
        } else if !ascii_isspace(byte(i)) {
            is_word = true;
        }
        chars += 1;
        // The guard above says `i` is inside the line, so the step is at
        // least one byte and the walk terminates.
        //
        // ASCII followed by ASCII is one byte, and on a text buffer that is
        // every step: `cluster_len` opens with the same test, but it is too
        // large to inline into this loop and the call alone costs more than
        // the answer. Ask it here, and leave everything else to the callee.
        let rest = &line[i as usize..];
        i += match rest {
            [first, next, ..] if *first < 0x80 && *next < 0x80 => 1,
            _ => cluster_len(rest) as VarNumber,
        };
    }

    if is_word {
        words += 1;
    }
    *wc += words;

    // The end of the line was reached before `limit`: count the break.
    if i < limit && byte(i) == NUL {
        i += VarNumber::from(eol_size);
        chars += VarNumber::from(eol_size);
    }
    *cc += chars;
    i
}

/// The six running totals `g CTRL-G` reports.
///
/// The `_cursor` three are "up to the cursor" outside Visual mode and "over
/// the selection" inside it, which is the whole difference between the two
/// messages [`report_counts`] can print.
#[derive(Default)]
struct PosCounts {
    /// Bytes in the buffer.
    bytes: VarNumber,
    /// Characters in the buffer.
    chars: VarNumber,
    /// Words in the buffer.
    words: VarNumber,
    /// Bytes up to the cursor, or in the selection.
    bytes_cursor: VarNumber,
    /// Characters up to the cursor, or in the selection.
    chars_cursor: VarNumber,
    /// Words up to the cursor, or in the selection.
    words_cursor: VarNumber,
}

/// The Visual selection, as the counting walk needs to see it.
struct Selection {
    /// Upper-left corner.
    min: Pos,
    /// Lower-right corner.
    max: Pos,
    /// Only the two vcols and the blockwise flags are filled in; it exists so
    /// that `block_prep` can be asked where the block sits in each line.
    oparg: OpArg,
    /// Lines the selection covers.
    line_count: c_int,
    /// `v`, `V` or CTRL-V, captured before the walk starts.
    mode: VisualMode,
}

/// `g CTRL-G`, and `wordcount()` when `dict` is not null.
///
/// # Safety
/// `dict`, when not null, must point to a live dictionary.
pub unsafe fn cursor_pos_info(dict: *mut Dict) {
    // The report is assembled across two functions and shown at the end, so
    // it is owned here rather than left in the shared `IObuff`.
    let mut report = String::new();
    let visual = visual_selection();
    let mut counts = PosCounts::default();
    let mut bom_count: VarNumber = 0;

    if Buf::current().b_ml.ml_flags.has(MlFlags::EMPTY) {
        if dict.is_null() {
            msg(gettext(no_lines_msg), 0);
            return;
        }
    } else {
        let mut selection = visual.map(measure_selection);

        if !count_buffer(&mut counts, selection.as_mut()) {
            // Interrupted part way through.
            return;
        }

        if dict.is_null() {
            report = report_counts(&counts, selection.as_ref());
        }

        bom_count = VarNumber::from(bomb_size());
        if dict.is_null() && bom_count > 0 {
            report.push_str(&tr!("(+{bom_count} for BOM)"));
        }

        if dict.is_null() {
            // 'shortmess' must not truncate this one.
            let saved_shm = P_SHM.clear();
            if p_ch() < 1 {
                msg_start();
                msg_scroll.set(1);
            }
            // `IObuff` is where upstream assembled this, and it truncated
            // there.
            msg(&to_message(report, MSG_IOBUFF_LEN), 0);
            P_SHM.restore(saved_shm);
        }
    }

    if !dict.is_null() {
        unsafe { store_counts(dict, &counts, bom_count, visual.is_some()) };
    }
}

/// Work out the Visual selection's corners and, for a blockwise one, the
/// column pair `block_prep` needs.
///
/// `sel` must be the current window's selection.
fn measure_selection(sel: VisualSelection) -> Selection {
    let (mut min, mut max) = if lt(sel.anchor, Win::current().w_cursor) {
        (sel.anchor, Win::current().w_cursor)
    } else {
        (Win::current().w_cursor, sel.anchor)
    };
    if sel_exclusive() && max.col > 0 {
        max.col -= 1;
    }

    let mut oparg = OpArg::ZERO;
    if sel.mode.is_block() {
        // 'showbreak' would move the columns `getvcols` answers.
        let saved_sbr = P_SBR.clear();
        let saved_w_sbr = Win::current().w_onebuf_opt.wo_sbr.take();

        oparg.is_visual = true;
        oparg.motion_type = kMTBlockWise;
        oparg.op_type = OpType::Nop;
        // SAFETY: a live window and two live positions in its buffer.
        let (sv, ev) = (&raw mut oparg.start_vcol, &raw mut oparg.end_vcol);
        unsafe { getvcols(Win::current(), &raw mut min, &raw mut max, sv, ev) };

        P_SBR.restore(saved_sbr);
        Win::current().w_onebuf_opt.wo_sbr = saved_w_sbr;

        if Win::current().w_curswant == MAXCOL {
            // `$`: the block has no right edge.
            oparg.end_vcol = MAXCOL;
        }
        if oparg.end_vcol < oparg.start_vcol {
            ::core::mem::swap(&mut oparg.start_vcol, &mut oparg.end_vcol);
        }
    }

    Selection {
        line_count: (max.lnum - min.lnum + 1) as c_int,
        mode: sel.mode,
        min,
        max,
        oparg,
    }
}

/// Walk the whole buffer, filling both sets of totals.
///
/// Answers false when the user interrupted it.
///
/// `selection`, when given, must describe the current window's Visual
/// selection.
fn count_buffer(counts: &mut PosCounts, mut selection: Option<&mut Selection>) -> bool {
    // `lnum` walks the buffer's own line count, so every line the walk asks
    // for is one of it.
    let eol_size = if get_fileformat(Buf::current()) == EOL_DOS {
        2
    } else {
        1
    };
    let mut bd = BlockDef::ZERO;
    let mut last_check: VarNumber = 100_000;

    for lnum in 1..=Buf::current().line_count() {
        if counts.bytes > last_check {
            os_breakcheck();
            if got_int.get() {
                return false;
            }
            last_check = counts.bytes + 100_000;
        }

        if let Some(sel) = selection.as_mut() {
            if lnum >= sel.min.lnum && lnum <= sel.max.lnum {
                count_selected_line(counts, sel, &mut bd, lnum, eol_size);
            }
        } else if lnum == Win::current().w_cursor.lnum {
            // Outside Visual mode the `_cursor` totals are the running
            // ones up to this line, plus this line up to the cursor.
            counts.words_cursor += counts.words;
            counts.chars_cursor += counts.chars;
            let upto = VarNumber::from(Win::current().w_cursor.col) + 1;
            let PosCounts {
                words_cursor: wc,
                chars_cursor: cc,
                ..
            } = counts;
            let mut lines = Lines::current();
            let taken = line_count_info(lines.line(lnum), wc, cc, upto, eol_size);
            counts.bytes_cursor = counts.bytes + taken;
        }

        let PosCounts { words, chars, .. } = counts;
        let all = VarNumber::from(MAXCOL);
        let mut lines = Lines::current();
        counts.bytes += line_count_info(lines.line(lnum), words, chars, all, eol_size);
    }

    // The last line has no EOL, so it was counted one byte too long.
    if Buf::current().b_p_eol == 0
        && (Buf::current().b_p_bin != 0 || Buf::current().b_p_fixeol == 0)
    {
        counts.bytes -= VarNumber::from(eol_size);
    }
    true
}

/// Add one line of the Visual selection to the `_cursor` totals.
///
/// `sel` must describe the current selection and `lnum` be inside it.
fn count_selected_line(
    counts: &mut PosCounts,
    sel: &mut Selection,
    bd: &mut BlockDef,
    lnum: LineNr,
    eol_size: c_int,
) {
    // `lnum` is a line of the current buffer, and `start_col` a column of it.
    let span = if sel.mode.is_block() {
        virtual_op.set(Some(virtual_active(Win::current())));
        // SAFETY: `sel.oparg` and `bd` are live, and `lnum` is in the region.
        unsafe { block_prep(&raw mut sel.oparg, &raw mut *bd, lnum, false) };
        virtual_op.set(None);
        // `block_prep` puts `textstart` at `textcol` of the same line.
        Some((bd.textcol, bd.textlen))
    } else if sel.mode.is_line() {
        Some((0, MAXCOL))
    } else if sel.mode.is_char() {
        let start_col = if lnum == sel.min.lnum { sel.min.col } else { 0 };
        let end_col = if lnum == sel.max.lnum {
            sel.max.col - start_col + 1
        } else {
            MAXCOL
        };
        Some((start_col, end_col))
    } else {
        None
    };

    let Some((start_col, len)) = span else {
        return;
    };
    let PosCounts {
        words_cursor: wc,
        chars_cursor: cc,
        ..
    } = counts;
    let mut lines = Lines::current();
    let line = lines.line(lnum);
    let at = usize::try_from(start_col).unwrap_or(0).min(line.len());
    let tail = line.len() - at;
    let taken = line_count_info(&line[at..], wc, cc, VarNumber::from(len), eol_size);
    counts.bytes_cursor += taken;
    // The last line has no EOL, and the selection reaches its end.
    if lnum == Buf::current().line_count()
        && Buf::current().b_p_eol == 0
        && (Buf::current().b_p_bin != 0 || Buf::current().b_p_fixeol == 0)
        && (tail as c_int) < len
    {
        counts.bytes_cursor -= VarNumber::from(eol_size);
    }
}

/// The message the counts describe.
///
/// Four spellings: with or without a selection, and with or without a
/// character count -- which is left out when it would equal the byte count,
/// so that an ASCII buffer reports the shorter message.
///
/// `selection`, when given, must describe the current selection.
fn report_counts(counts: &PosCounts, selection: Option<&Selection>) -> String {
    let &PosCounts {
        bytes,
        chars,
        words,
        bytes_cursor: bc,
        chars_cursor: cc,
        words_cursor: wc,
    } = counts;
    let same_as_bytes = cc == bc && chars == bytes;
    let lines = Buf::current().line_count() as int64_t;

    let Some(sel) = selection else {
        let lnum = Win::current().w_cursor.lnum as int64_t;
        let (col, virtcol) = (
            Win::current().w_cursor.col + 1,
            Win::current().w_virtcol + 1,
        );
        let p = get_cursor_line_ptr();
        validate_virtcol(Win::current());
        let at = col_text(col, virtcol);
        // SAFETY: the cursor is on a line of the current buffer, which is
        // what `linetabsize_str` asks of the pointer taken above.
        let of = col_text(get_cursor_line_len(), unsafe { linetabsize_str(p) });
        return if same_as_bytes {
            tr!(
                "Col {at} of {of}; Line {lnum} of {lines}; Word {wc} of {words}; Byte {bc} of {bytes}"
            )
        } else {
            tr!(
                "Col {at} of {of}; Line {lnum} of {lines}; Word {wc} of {words}; Char {cc} of {chars}; Byte {bc} of {bytes}"
            )
        };
    };

    // A blockwise selection with a right edge also reports its width.
    let width = if sel.mode.is_block() && Win::current().w_curswant < MAXCOL {
        let mut min = sel.min;
        let mut max = sel.max;
        // Both vcols are `c_int`, so the difference cannot overflow an
        // `int64_t` (upstream computes it under STRICT_SUB and aborts).
        let cols = int64_t::from(sel.oparg.end_vcol) + 1 - int64_t::from(sel.oparg.start_vcol);
        let (minc, maxc) = (&raw mut min.col, &raw mut max.col);
        // SAFETY: the two positions are this frame's own, and so are the
        // columns written back into them.
        unsafe { getvcols(Win::current(), &raw mut min, &raw mut max, minc, maxc) };
        tr!("{cols} Cols; ")
    } else {
        String::new()
    };

    let sel_lines = int64_t::from(sel.line_count);
    if same_as_bytes {
        tr!(
            "Selected {width}{sel_lines} of {lines} Lines; {wc} of {words} Words; {bc} of {bytes} Bytes"
        )
    } else {
        tr!(
            "Selected {width}{sel_lines} of {lines} Lines; {wc} of {words} Words; {cc} of {chars} Chars; {bc} of {bytes} Bytes"
        )
    }
}

/// Store the counts in `dict` instead of showing them -- `wordcount()`.
///
/// The `visual_*` and `cursor_*` keys are the same three numbers under
/// different names, which is how a caller tells which question was answered.
///
/// # Safety
/// `dict` must point to a live dictionary.
unsafe fn store_counts(
    dict: *mut Dict,
    counts: &PosCounts,
    bom_count: VarNumber,
    visual_active: bool,
) {
    // SAFETY: the caller's promise -- a live dictionary.
    let add = |key: &::core::ffi::CStr, value: VarNumber| {
        let _ = unsafe { (*dict).add_number(key.to_bytes(), value) };
    };
    add(c"words", counts.words);
    add(c"chars", counts.chars);
    add(c"bytes", counts.bytes + bom_count);
    if visual_active {
        add(c"visual_bytes", counts.bytes_cursor);
        add(c"visual_chars", counts.chars_cursor);
        add(c"visual_words", counts.words_cursor);
    } else {
        add(c"cursor_bytes", counts.bytes_cursor);
        add(c"cursor_chars", counts.chars_cursor);
        add(c"cursor_words", counts.words_cursor);
    }
}

/// Bytes between two positions, line breaks included.
///
/// Does not look at the text: only at the line lengths, which is why the
/// buffer API and quickfix use it to size a splice. A range past the end of
/// the buffer is clipped rather than refused.
///
/// Safe: a [`Buf`] carries the whole of the promise this needs, and every
/// line asked for below is checked against its line count first.
pub fn get_region_bytecount(
    buffer: Buf,
    start_lnum: LineNr,
    end_lnum: LineNr,
    start_col: ColNr,
    end_col: ColNr,
) -> BCount {
    let max_lnum = buffer.line_count();
    if start_lnum > max_lnum {
        return 0;
    }
    if start_lnum == end_lnum {
        return (end_col - start_col) as BCount;
    }

    // The rest of the first line, its break included.
    let first_len = buffer.line_len_raw(start_lnum);
    let mut bytes = (first_len - start_col + 1) as BCount;
    for i in 1..=end_lnum - start_lnum - 1 {
        if start_lnum + i > max_lnum {
            return bytes;
        }
        bytes += (buffer.line_len_raw(start_lnum + i) + 1) as BCount;
    }
    if end_lnum > max_lnum {
        return bytes;
    }
    bytes + end_col as BCount
}

#[cfg(test)]
mod tests {
    use super::*;

    fn count(line: &[u8], limit: VarNumber, eol_size: c_int) -> (VarNumber, VarNumber, VarNumber) {
        let (mut words, mut chars) = (0, 0);
        let bytes = line_count_info(line, &mut words, &mut chars, limit, eol_size);
        (words, chars, bytes)
    }

    const ALL: VarNumber = MAXCOL as VarNumber;

    #[test]
    fn a_whole_line_counts_its_words_and_its_line_break() {
        // The break is one byte and one character when the walk reaches the
        // end of the line before the limit.
        assert_eq!(count(b"one two three", ALL, 1), (3, 14, 14));
        assert_eq!(count(b"", ALL, 1), (0, 1, 1));
        // A `dos` fileformat's break is two of each.
        assert_eq!(count(b"one", ALL, 2), (1, 5, 5));
        // Trailing white space still ends the last word.
        assert_eq!(count(b"one  ", ALL, 1), (1, 6, 6));
        assert_eq!(count(b"  one", ALL, 1), (1, 6, 6));
    }

    #[test]
    fn a_limit_stops_the_walk_before_the_break() {
        // Four bytes of "one two": the break is not reached, so it is not
        // counted -- which is how the cursor totals stop mid-line.
        assert_eq!(count(b"one two", 4, 1), (1, 4, 4));
        // Exactly the line's length: still short of the break.
        assert_eq!(count(b"one", 3, 1), (1, 3, 3));
    }

    #[test]
    fn a_character_counts_once_however_many_bytes_it_takes() {
        // Two three-byte characters and a space: three characters, seven
        // bytes, plus the break.
        let line = "\u{4e00} \u{4e8c}".as_bytes();
        assert_eq!(line.len(), 7);
        assert_eq!(count(line, ALL, 1), (2, 4, 8));
    }
}
