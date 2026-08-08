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

use ::core::ffi::{c_char, c_int};

use super::*;

/// Bytes, words and characters in one line, up to `limit` bytes.
///
/// Answers the bytes consumed and *adds* the words and characters to `wc` and
/// `cc`, because every caller is accumulating. A word is a run of
/// non-white-space, which is what `wc(1)` counts too. Reaching the end of the
/// line before `limit` adds `eol_size` for the line break itself.
///
/// # Safety
/// `line` must be NUL-terminated; `wc` and `cc` must be writable.
unsafe fn line_count_info(
    line: *mut c_char,
    wc: *mut varnumber_T,
    cc: *mut varnumber_T,
    limit: varnumber_T,
    eol_size: c_int,
) -> varnumber_T {
    unsafe {
        let mut words = 0;
        let mut chars = 0;
        let mut is_word = false;

        let mut i: varnumber_T = 0;
        while i < limit && *line.offset(i as isize) as c_int != NUL {
            if is_word {
                if ascii_isspace(*line.offset(i as isize) as c_int) {
                    words += 1;
                    is_word = false;
                }
            } else if !ascii_isspace(*line.offset(i as isize) as c_int) {
                is_word = true;
            }
            chars += 1;
            i += varnumber_T::from(utfc_ptr2len(line.offset(i as isize)));
        }

        if is_word {
            words += 1;
        }
        *wc += words;

        // The end of the line was reached before `limit`: count the break.
        if i < limit && *line.offset(i as isize) as c_int == NUL {
            i += varnumber_T::from(eol_size);
            chars += varnumber_T::from(eol_size);
        }
        *cc += chars;
        i
    }
}

/// The six running totals `g CTRL-G` reports.
///
/// The `_cursor` three are "up to the cursor" outside Visual mode and "over
/// the selection" inside it, which is the whole difference between the two
/// messages [`report_counts`] can print.
#[derive(Default)]
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
    mode: c_int,
}

/// `g CTRL-G`, and `wordcount()` when `dict` is not null.
///
/// # Safety
/// `dict`, when not null, must point to a live dictionary.
pub unsafe fn cursor_pos_info(dict: *mut dict_T) {
    unsafe {
        let visual_active = VIsual_active.get();
        let visual_mode = VIsual_mode.get();
        let mut counts = PosCounts::default();
        let mut bom_count: varnumber_T = 0;

        if (*curbuf.get()).b_ml.ml_flags & ML_EMPTY != 0 {
            if dict.is_null() {
                msg(gettext(no_lines_msg.ptr() as *mut c_char), 0);
                return;
            }
        } else {
            let mut selection = if visual_active {
                Some(measure_selection(visual_mode))
            } else {
                None
            };

            if !count_buffer(&mut counts, selection.as_mut()) {
                // Interrupted part way through.
                return;
            }

            if dict.is_null() {
                report_counts(&counts, selection.as_ref(), visual_mode);
            }

            bom_count = varnumber_T::from(bomb_size());
            if dict.is_null() && bom_count > 0 {
                let len = strlen(IObuff.ptr() as *mut c_char);
                vim_snprintf(
                    (IObuff.ptr() as *mut c_char).offset(len as isize),
                    IOSIZE as size_t - len,
                    gettext(c"(+%ld for BOM)".as_ptr()),
                    bom_count,
                );
            }

            if dict.is_null() {
                // 'shortmess' must not truncate this one.
                let saved_shm = p_shm.get();
                p_shm.set(c"".as_ptr() as *mut c_char);
                if p_ch.get() < 1 {
                    msg_start();
                    msg_scroll.set(true_0);
                }
                msg(IObuff.ptr() as *mut c_char, 0);
                p_shm.set(saved_shm);
            }
        }

        if !dict.is_null() {
            store_counts(dict, &counts, bom_count, visual_active);
        }
    }
}

/// Work out the Visual selection's corners and, for a blockwise one, the
/// column pair `block_prep` needs.
///
/// # Safety
/// A Visual selection must be active in the current window.
unsafe fn measure_selection(visual_mode: c_int) -> Selection {
    unsafe {
        let (mut min, mut max) = if lt(VIsual.get(), (*curwin.get()).w_cursor) {
            (VIsual.get(), (*curwin.get()).w_cursor)
        } else {
            ((*curwin.get()).w_cursor, VIsual.get())
        };
        if *p_sel.get() as c_int == 'e' as c_int && max.col > 0 {
            max.col -= 1;
        }

        let mut oparg = oparg_T::ZERO;
        if visual_mode == Ctrl_V {
            // 'showbreak' would move the columns `getvcols` answers.
            let saved_sbr = p_sbr.get();
            let saved_w_sbr = (*curwin.get()).w_onebuf_opt.wo_sbr;
            p_sbr.set(empty_string_option.ptr() as *mut c_char);
            (*curwin.get()).w_onebuf_opt.wo_sbr = empty_string_option.ptr() as *mut c_char;

            oparg.is_VIsual = true;
            oparg.motion_type = kMTBlockWise;
            oparg.op_type = OP_NOP;
            getvcols(
                curwin.get(),
                &raw mut min,
                &raw mut max,
                &raw mut oparg.start_vcol,
                &raw mut oparg.end_vcol,
            );

            p_sbr.set(saved_sbr);
            (*curwin.get()).w_onebuf_opt.wo_sbr = saved_w_sbr;

            if (*curwin.get()).w_curswant == MAXCOL {
                // `$`: the block has no right edge.
                oparg.end_vcol = MAXCOL;
            }
            if oparg.end_vcol < oparg.start_vcol {
                ::core::mem::swap(&mut oparg.start_vcol, &mut oparg.end_vcol);
            }
        }

        Selection {
            line_count: (max.lnum - min.lnum + 1) as c_int,
            mode: visual_mode,
            min,
            max,
            oparg,
        }
    }
}

/// Walk the whole buffer, filling both sets of totals.
///
/// Answers false when the user interrupted it.
///
/// # Safety
/// `selection`, when given, must describe the current window's Visual
/// selection.
unsafe fn count_buffer(counts: &mut PosCounts, mut selection: Option<&mut Selection>) -> bool {
    unsafe {
        let eol_size = if get_fileformat(curbuf.get()) == EOL_DOS {
            2
        } else {
            1
        };
        let mut bd = block_def::ZERO;
        let mut last_check: varnumber_T = 100_000;

        for lnum in 1..=(*curbuf.get()).b_ml.ml_line_count {
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
            } else if lnum == (*curwin.get()).w_cursor.lnum {
                // Outside Visual mode the `_cursor` totals are the running
                // ones up to this line, plus this line up to the cursor.
                counts.words_cursor += counts.words;
                counts.chars_cursor += counts.chars;
                counts.bytes_cursor = counts.bytes
                    + line_count_info(
                        ml_get(lnum),
                        &raw mut counts.words_cursor,
                        &raw mut counts.chars_cursor,
                        varnumber_T::from((*curwin.get()).w_cursor.col) + 1,
                        eol_size,
                    );
            }

            counts.bytes += line_count_info(
                ml_get(lnum),
                &raw mut counts.words,
                &raw mut counts.chars,
                varnumber_T::from(MAXCOL),
                eol_size,
            );
        }

        // The last line has no EOL, so it was counted one byte too long.
        if (*curbuf.get()).b_p_eol == 0
            && ((*curbuf.get()).b_p_bin != 0 || (*curbuf.get()).b_p_fixeol == 0)
        {
            counts.bytes -= varnumber_T::from(eol_size);
        }
        true
    }
}

/// Add one line of the Visual selection to the `_cursor` totals.
///
/// # Safety
/// `sel` must describe the current selection and `lnum` be inside it.
unsafe fn count_selected_line(
    counts: &mut PosCounts,
    sel: &mut Selection,
    bd: &mut block_def,
    lnum: linenr_T,
    eol_size: c_int,
) {
    unsafe {
        let mut s: *mut c_char = ::core::ptr::null_mut();
        let mut len = 0;
        if sel.mode == Ctrl_V {
            virtual_op.set(virtual_active(curwin.get()) as TriState);
            block_prep(&raw mut sel.oparg, &raw mut *bd, lnum, false);
            virtual_op.set(kNone);
            s = bd.textstart;
            len = bd.textlen;
        } else if sel.mode == 'V' as c_int {
            s = ml_get(lnum);
            len = MAXCOL;
        } else if sel.mode == 'v' as c_int {
            let start_col = if lnum == sel.min.lnum { sel.min.col } else { 0 };
            let end_col = if lnum == sel.max.lnum {
                sel.max.col - start_col + 1
            } else {
                MAXCOL
            };
            s = ml_get(lnum).offset(start_col as isize);
            len = end_col;
        }

        if s.is_null() {
            return;
        }
        counts.bytes_cursor += line_count_info(
            s,
            &raw mut counts.words_cursor,
            &raw mut counts.chars_cursor,
            varnumber_T::from(len),
            eol_size,
        );
        // The last line has no EOL, and the selection reaches its end.
        if lnum == (*curbuf.get()).b_ml.ml_line_count
            && (*curbuf.get()).b_p_eol == 0
            && ((*curbuf.get()).b_p_bin != 0 || (*curbuf.get()).b_p_fixeol == 0)
            && (strlen(s) as c_int) < len
        {
            counts.bytes_cursor -= varnumber_T::from(eol_size);
        }
    }
}

/// Build the message into `IObuff`.
///
/// Four spellings: with or without a selection, and with or without a
/// character count -- which is left out when it would equal the byte count,
/// so that an ASCII buffer reports the shorter message.
///
/// # Safety
/// `selection`, when given, must describe the current selection.
unsafe fn report_counts(counts: &PosCounts, selection: Option<&Selection>, visual_mode: c_int) {
    unsafe {
        let same_as_bytes =
            counts.chars_cursor == counts.bytes_cursor && counts.chars == counts.bytes;
        let mut buf1: [c_char; 50] = [0; 50];

        let Some(sel) = selection else {
            let mut buf2: [c_char; 40] = [0; 40];
            let p = get_cursor_line_ptr();
            validate_virtcol(curwin.get());
            col_print(
                &raw mut buf1 as *mut c_char,
                buf1.len(),
                (*curwin.get()).w_cursor.col + 1,
                (*curwin.get()).w_virtcol + 1,
            );
            col_print(
                &raw mut buf2 as *mut c_char,
                buf2.len(),
                get_cursor_line_len(),
                linetabsize_str(p),
            );
            if same_as_bytes {
                vim_snprintf(
                    IObuff.ptr() as *mut c_char,
                    IOSIZE as size_t,
                    gettext(
                        c"Col %s of %s; Line %ld of %ld; Word %ld of %ld; Byte %ld of %ld".as_ptr(),
                    ),
                    &raw mut buf1 as *mut c_char,
                    &raw mut buf2 as *mut c_char,
                    (*curwin.get()).w_cursor.lnum as int64_t,
                    (*curbuf.get()).b_ml.ml_line_count as int64_t,
                    counts.words_cursor,
                    counts.words,
                    counts.bytes_cursor,
                    counts.bytes,
                );
            } else {
                vim_snprintf(
                    IObuff.ptr() as *mut c_char,
                    IOSIZE as size_t,
                    gettext(c"Col %s of %s; Line %ld of %ld; Word %ld of %ld; Char %ld of %ld; Byte %ld of %ld".as_ptr()),
                    &raw mut buf1 as *mut c_char,
                    &raw mut buf2 as *mut c_char,
                    (*curwin.get()).w_cursor.lnum as int64_t,
                    (*curbuf.get()).b_ml.ml_line_count as int64_t,
                    counts.words_cursor,
                    counts.words,
                    counts.chars_cursor,
                    counts.chars,
                    counts.bytes_cursor,
                    counts.bytes,
                );
            }
            return;
        };

        // A blockwise selection with a right edge also reports its width.
        if visual_mode == Ctrl_V && (*curwin.get()).w_curswant < MAXCOL {
            let mut min = sel.min;
            let mut max = sel.max;
            getvcols(
                curwin.get(),
                &raw mut min,
                &raw mut max,
                &raw mut min.col,
                &raw mut max.col,
            );
            // Both vcols are `c_int`, so the difference cannot overflow an
            // `int64_t` (upstream computes it under STRICT_SUB and aborts).
            let cols = int64_t::from(sel.oparg.end_vcol) + 1 - int64_t::from(sel.oparg.start_vcol);
            vim_snprintf(
                &raw mut buf1 as *mut c_char,
                buf1.len(),
                gettext(c"%ld Cols; ".as_ptr()),
                cols,
            );
        } else {
            buf1[0] = NUL as c_char;
        }

        if same_as_bytes {
            vim_snprintf(
                IObuff.ptr() as *mut c_char,
                IOSIZE as size_t,
                gettext(
                    c"Selected %s%ld of %ld Lines; %ld of %ld Words; %ld of %ld Bytes".as_ptr(),
                ),
                &raw mut buf1 as *mut c_char,
                int64_t::from(sel.line_count),
                (*curbuf.get()).b_ml.ml_line_count as int64_t,
                counts.words_cursor,
                counts.words,
                counts.bytes_cursor,
                counts.bytes,
            );
        } else {
            vim_snprintf(
                IObuff.ptr() as *mut c_char,
                IOSIZE as size_t,
                gettext(c"Selected %s%ld of %ld Lines; %ld of %ld Words; %ld of %ld Chars; %ld of %ld Bytes".as_ptr()),
                &raw mut buf1 as *mut c_char,
                int64_t::from(sel.line_count),
                (*curbuf.get()).b_ml.ml_line_count as int64_t,
                counts.words_cursor,
                counts.words,
                counts.chars_cursor,
                counts.chars,
                counts.bytes_cursor,
                counts.bytes,
            );
        }
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
    unsafe {
        let mut add = |key: &::core::ffi::CStr, value: varnumber_T| {
            tv_dict_add_nr(dict, key.as_ptr(), key.count_bytes(), value);
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
}

/// Bytes between two positions, line breaks included.
///
/// Does not look at the text: only at the line lengths, which is why the
/// buffer API and quickfix use it to size a splice. A range past the end of
/// the buffer is clipped rather than refused.
///
/// # Safety
/// `buf` must point to a live buffer.
pub unsafe fn get_region_bytecount(
    buf: *mut buf_T,
    start_lnum: linenr_T,
    end_lnum: linenr_T,
    start_col: colnr_T,
    end_col: colnr_T,
) -> bcount_t {
    unsafe {
        let max_lnum = (*buf).b_ml.ml_line_count;
        if start_lnum > max_lnum {
            return 0;
        }
        if start_lnum == end_lnum {
            return (end_col - start_col) as bcount_t;
        }

        // The rest of the first line, its break included.
        let mut bytes = (ml_get_buf_len(buf, start_lnum) - start_col + 1) as bcount_t;
        for i in 1..=end_lnum - start_lnum - 1 {
            if start_lnum + i > max_lnum {
                return bytes;
            }
            bytes += (ml_get_buf_len(buf, start_lnum + i) + 1) as bcount_t;
        }
        if end_lnum > max_lnum {
            return bytes;
        }
        bytes + end_col as bcount_t
    }
}
