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

use crate::memline::MlFlags;
use crate::winlayer::{Buf, Win};
use core::ffi::{c_char, c_int};

use super::*;
use crate::normal::{VisualMode, VisualSelection, sel_exclusive, visual_selection};
use crate::optionstr::empty_option;
use crate::types::{IOSIZE, NUL};

/// Bytes, words and characters in one line, up to `limit` bytes.
///
/// Answers the bytes consumed and *adds* the words and characters to `wc` and
/// `cc`, because every caller is accumulating. A word is a run of
/// non-white-space, which is what `wc(1)` counts too. Reaching the end of the
/// line before `limit` adds `eol_size` for the line break itself.
///
/// # Safety
/// `line` must be NUL-terminated.
unsafe fn line_count_info(
    line: *mut c_char,
    wc: &mut varnumber_T,
    cc: &mut varnumber_T,
    limit: varnumber_T,
    eol_size: c_int,
) -> varnumber_T {
    // SAFETY: the caller's promise -- the walk stops at the line's NUL, so
    // every index it takes is inside the line.
    let byte = |i: varnumber_T| unsafe { *line.offset(i as isize) } as c_int;
    let mut words = 0;
    let mut chars = 0;
    let mut is_word = false;

    let mut i: varnumber_T = 0;
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
        i += varnumber_T::from(unsafe { utfc_ptr2len(line.offset(i as isize)) });
    }

    if is_word {
        words += 1;
    }
    *wc += words;

    // The end of the line was reached before `limit`: count the break.
    if i < limit && byte(i) == NUL {
        i += varnumber_T::from(eol_size);
        chars += varnumber_T::from(eol_size);
    }
    *cc += chars;
    i
}

/// The six running totals `g CTRL-G` reports.
///
/// The `_cursor` three are "up to the cursor" outside Visual mode and "over
/// the selection" inside it, which is the whole difference between the two
/// messages [`report_counts`] can print.
#[derive(Default, Clone, Copy)]
struct PosCounts {
    /// Bytes in the buffer.
    bytes: varnumber_T,
    /// Characters in the buffer.
    chars: varnumber_T,
    /// Words in the buffer.
    words: varnumber_T,
    /// Bytes up to the cursor, or in the selection.
    bytes_cursor: varnumber_T,
    /// Characters up to the cursor, or in the selection.
    chars_cursor: varnumber_T,
    /// Words up to the cursor, or in the selection.
    words_cursor: varnumber_T,
}

/// The Visual selection, as the counting walk needs to see it.
struct Selection {
    /// Upper-left corner.
    min: pos_T,
    /// Lower-right corner.
    max: pos_T,
    /// Only the two vcols and the blockwise flags are filled in; it exists so
    /// that `block_prep` can be asked where the block sits in each line.
    oparg: oparg_T,
    /// Lines the selection covers.
    line_count: c_int,
    /// `v`, `V` or CTRL-V, captured before the walk starts.
    mode: VisualMode,
}

/// `g CTRL-G`, and `wordcount()` when `dict` is not null.
///
/// # Safety
/// `dict`, when not null, must point to a live dictionary.
pub unsafe fn cursor_pos_info(dict: *mut dict_T) {
    // The report is assembled across two functions and shown at the end, so
    // it is passed down as a sink rather than left in the shared `IObuff`.
    let mut report = [0 as c_char; IOSIZE as usize];
    let visual = visual_selection();
    let mut counts = PosCounts::default();
    let mut bom_count: varnumber_T = 0;

    // SAFETY: `report` is `IOSIZE` bytes and every write to it is bounded by
    // that; the message strings are the editor's own.
    if cur_buf().b_ml.ml_flags.has(MlFlags::EMPTY) {
        if dict.is_null() {
            unsafe { msg(gettext(no_lines_msg.as_ptr().cast_mut()), 0) };
            return;
        }
    } else {
        let mut selection = visual.map(measure_selection);

        if !count_buffer(&mut counts, selection.as_mut()) {
            // Interrupted part way through.
            return;
        }

        if dict.is_null() {
            report_counts(&mut report, &counts, selection.as_ref());
        }

        bom_count = varnumber_T::from(unsafe { bomb_size() });
        if dict.is_null() && bom_count > 0 {
            let len = unsafe { strlen(report.as_ptr()) };
            let at = unsafe { report.as_mut_ptr().add(len) };
            let fmt = unsafe { gettext(c"(+%ld for BOM)".as_ptr()) };
            unsafe { vim_snprintf(at, IOSIZE as size_t - len, fmt, bom_count) };
        }

        if dict.is_null() {
            // 'shortmess' must not truncate this one.
            let saved_shm = p_shm.get();
            p_shm.set(c"".as_ptr() as *mut c_char);
            if p_ch.get() < 1 {
                unsafe { msg_start() };
                msg_scroll.set(1);
            }
            unsafe { msg(report.as_mut_ptr(), 0) };
            p_shm.set(saved_shm);
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
    let (mut min, mut max) = if lt(sel.anchor, cur_win().w_cursor) {
        (sel.anchor, cur_win().w_cursor)
    } else {
        (cur_win().w_cursor, sel.anchor)
    };
    if sel_exclusive() && max.col > 0 {
        max.col -= 1;
    }

    let mut oparg = oparg_T::ZERO;
    if sel.mode.is_block() {
        // 'showbreak' would move the columns `getvcols` answers.
        let saved_sbr = p_sbr.get();
        let saved_w_sbr = cur_win().w_onebuf_opt.wo_sbr;
        p_sbr.set(empty_option());
        cur_win().w_onebuf_opt.wo_sbr = empty_option();

        oparg.is_VIsual = true;
        oparg.motion_type = kMTBlockWise;
        oparg.op_type = OP_NOP;
        // SAFETY: a live window and two live positions in its buffer.
        let (sv, ev) = (&raw mut oparg.start_vcol, &raw mut oparg.end_vcol);
        unsafe { getvcols(cur_win(), &raw mut min, &raw mut max, sv, ev) };

        p_sbr.set(saved_sbr);
        cur_win().w_onebuf_opt.wo_sbr = saved_w_sbr;

        if cur_win().w_curswant == MAXCOL {
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
    let eol_size = if get_fileformat(cur_buf()) == EOL_DOS {
        2
    } else {
        1
    };
    let mut bd = block_def::ZERO;
    let mut last_check: varnumber_T = 100_000;

    for lnum in 1..=cur_buf().line_count() {
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
        } else if lnum == cur_win().w_cursor.lnum {
            // Outside Visual mode the `_cursor` totals are the running
            // ones up to this line, plus this line up to the cursor.
            counts.words_cursor += counts.words;
            counts.chars_cursor += counts.chars;
            let upto = varnumber_T::from(cur_win().w_cursor.col) + 1;
            let PosCounts {
                words_cursor: wc,
                chars_cursor: cc,
                ..
            } = counts;
            let line = ml_get(lnum);
            let taken = unsafe { line_count_info(line, wc, cc, upto, eol_size) };
            counts.bytes_cursor = counts.bytes + taken;
        }

        let PosCounts { words, chars, .. } = counts;
        let line = ml_get(lnum);
        let all = varnumber_T::from(MAXCOL);
        counts.bytes += unsafe { line_count_info(line, words, chars, all, eol_size) };
    }

    // The last line has no EOL, so it was counted one byte too long.
    if cur_buf().b_p_eol == 0 && (cur_buf().b_p_bin != 0 || cur_buf().b_p_fixeol == 0) {
        counts.bytes -= varnumber_T::from(eol_size);
    }
    true
}

/// Add one line of the Visual selection to the `_cursor` totals.
///
/// `sel` must describe the current selection and `lnum` be inside it.
fn count_selected_line(
    counts: &mut PosCounts,
    sel: &mut Selection,
    bd: &mut block_def,
    lnum: linenr_T,
    eol_size: c_int,
) {
    // SAFETY: `lnum` is a line of the current buffer, so `ml_get` answers a
    // live NUL-terminated line and `start_col` is a column of it.
    let mut s: *mut c_char = ::core::ptr::null_mut();
    let mut len = 0;
    if sel.mode.is_block() {
        virtual_op.set(Some(virtual_active(cur_win())));
        unsafe { block_prep(&raw mut sel.oparg, &raw mut *bd, lnum, false) };
        virtual_op.set(None);
        s = bd.textstart;
        len = bd.textlen;
    } else if sel.mode.is_line() {
        s = ml_get(lnum);
        len = MAXCOL;
    } else if sel.mode.is_char() {
        let start_col = if lnum == sel.min.lnum { sel.min.col } else { 0 };
        let end_col = if lnum == sel.max.lnum {
            sel.max.col - start_col + 1
        } else {
            MAXCOL
        };
        s = unsafe { ml_get(lnum).offset(start_col as isize) };
        len = end_col;
    }

    if s.is_null() {
        return;
    }
    let PosCounts {
        words_cursor: wc,
        chars_cursor: cc,
        ..
    } = counts;
    let taken = unsafe { line_count_info(s, wc, cc, varnumber_T::from(len), eol_size) };
    counts.bytes_cursor += taken;
    // The last line has no EOL, and the selection reaches its end.
    if lnum == cur_buf().line_count()
        && cur_buf().b_p_eol == 0
        && (cur_buf().b_p_bin != 0 || cur_buf().b_p_fixeol == 0)
        && (unsafe { strlen(s) } as c_int) < len
    {
        counts.bytes_cursor -= varnumber_T::from(eol_size);
    }
}

/// Build the message into `out`.
///
/// Four spellings: with or without a selection, and with or without a
/// character count -- which is left out when it would equal the byte count,
/// so that an ASCII buffer reports the shorter message.
///
/// `selection`, when given, must describe the current selection.
fn report_counts(
    out: &mut [c_char; IOSIZE as usize],
    counts: &PosCounts,
    selection: Option<&Selection>,
) {
    let &PosCounts {
        bytes,
        chars,
        words,
        bytes_cursor: bc,
        chars_cursor: cc,
        words_cursor: wc,
    } = counts;
    let same_as_bytes = cc == bc && chars == bytes;
    let mut buf1: [c_char; 50] = [0; 50];
    let (n1, b1) = (buf1.len(), buf1.as_mut_ptr());
    let n = IOSIZE as size_t;
    let out = out.as_mut_ptr();
    let lines = cur_buf().line_count() as int64_t;

    // SAFETY: `out` is `IOSIZE` bytes and `buf1`/`buf2` are sized beside the
    // calls that fill them; the formats are this file's own literals, whose
    // conversions match the arguments handed after them. The cursor is on a
    // line of the current buffer, which is what `col_print`'s two measures
    // and `linetabsize_str` ask for.
    let Some(sel) = selection else {
        let mut buf2: [c_char; 40] = [0; 40];
        let (n2, b2) = (buf2.len(), buf2.as_mut_ptr());
        let lnum = cur_win().w_cursor.lnum as int64_t;
        let (col, virtcol) = (cur_win().w_cursor.col + 1, cur_win().w_virtcol + 1);
        let p = get_cursor_line_ptr();
        validate_virtcol(cur_win());
        unsafe { col_print(b1, n1, col, virtcol) };
        unsafe { col_print(b2, n2, get_cursor_line_len(), linetabsize_str(p)) };
        if same_as_bytes {
            let f = c"Col %s of %s; Line %ld of %ld; Word %ld of %ld; Byte %ld of %ld";
            unsafe {
                vim_snprintf(
                    out,
                    n,
                    gettext(f.as_ptr()),
                    b1,
                    b2,
                    lnum,
                    lines,
                    wc,
                    words,
                    bc,
                    bytes,
                )
            };
        } else {
            let f =
                c"Col %s of %s; Line %ld of %ld; Word %ld of %ld; Char %ld of %ld; Byte %ld of %ld";
            let f = unsafe { gettext(f.as_ptr()) };
            unsafe {
                vim_snprintf(
                    out, n, f, b1, b2, lnum, lines, wc, words, cc, chars, bc, bytes,
                )
            };
        }
        return;
    };

    // A blockwise selection with a right edge also reports its width.
    if sel.mode.is_block() && cur_win().w_curswant < MAXCOL {
        let mut min = sel.min;
        let mut max = sel.max;
        // Both vcols are `c_int`, so the difference cannot overflow an
        // `int64_t` (upstream computes it under STRICT_SUB and aborts).
        let cols = int64_t::from(sel.oparg.end_vcol) + 1 - int64_t::from(sel.oparg.start_vcol);
        let (minc, maxc) = (&raw mut min.col, &raw mut max.col);
        unsafe { getvcols(cur_win(), &raw mut min, &raw mut max, minc, maxc) };
        unsafe { vim_snprintf(b1, n1, gettext(c"%ld Cols; ".as_ptr()), cols) };
    } else {
        buf1[0] = NUL as c_char;
    }

    let sel_lines = int64_t::from(sel.line_count);
    if same_as_bytes {
        let f = c"Selected %s%ld of %ld Lines; %ld of %ld Words; %ld of %ld Bytes";
        let f = unsafe { gettext(f.as_ptr()) };
        unsafe { vim_snprintf(out, n, f, b1, sel_lines, lines, wc, words, bc, bytes) };
    } else {
        let f =
            c"Selected %s%ld of %ld Lines; %ld of %ld Words; %ld of %ld Chars; %ld of %ld Bytes";
        let f = unsafe { gettext(f.as_ptr()) };
        unsafe {
            vim_snprintf(
                out, n, f, b1, sel_lines, lines, wc, words, cc, chars, bc, bytes,
            )
        };
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
    dict: *mut dict_T,
    counts: &PosCounts,
    bom_count: varnumber_T,
    visual_active: bool,
) {
    // SAFETY: the caller's promise -- a live dictionary.
    let mut add = |key: &::core::ffi::CStr, value: varnumber_T| {
        unsafe { tv_dict_add_nr(dict, key.as_ptr(), key.count_bytes(), value) };
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
    buf: Buf,
    start_lnum: linenr_T,
    end_lnum: linenr_T,
    start_col: colnr_T,
    end_col: colnr_T,
) -> bcount_t {
    let max_lnum = buf.line_count();
    if start_lnum > max_lnum {
        return 0;
    }
    if start_lnum == end_lnum {
        return (end_col - start_col) as bcount_t;
    }

    // The rest of the first line, its break included.
    let first_len = unsafe { buf.line_len(start_lnum) };
    let mut bytes = (first_len - start_col + 1) as bcount_t;
    for i in 1..=end_lnum - start_lnum - 1 {
        if start_lnum + i > max_lnum {
            return bytes;
        }
        bytes += (unsafe { buf.line_len(start_lnum + i) } + 1) as bcount_t;
    }
    if end_lnum > max_lnum {
        return bytes;
    }
    bytes + end_col as bcount_t
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
